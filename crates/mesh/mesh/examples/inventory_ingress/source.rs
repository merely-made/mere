// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, require};
use serde_json::{Value, json};
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::time::{Instant, timeout_at};

/// Bounded persistent IPC; a failed/uncertain command kills the child on drop.
pub struct Source {
    child: Child,
    input: Option<ChildStdin>,
    output: BufReader<ChildStdout>,
    stderr: tokio::task::JoinHandle<std::io::Result<Vec<u8>>>,
    deadline: Instant,
    next_id: u32,
    pub now: u32,
    read: usize,
    written: usize,
    pub hello: Value,
    pub exchanges: Vec<Value>,
}

impl Source {
    pub async fn start(executable: &Path) -> Result<Self> {
        let mut child = Command::new(executable)
            .arg("--retained-peer")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()?;
        let input = child.stdin.take().ok_or("peer stdin")?;
        let output = child.stdout.take().ok_or("peer stdout")?;
        let stderr = child.stderr.take().ok_or("peer stderr")?;
        let stderr = tokio::spawn(async move {
            let mut bytes = Vec::new();
            stderr.take(65537).read_to_end(&mut bytes).await?;
            Ok(bytes)
        });
        let mut source = Self {
            child,
            input: Some(input),
            output: BufReader::new(output),
            stderr,
            deadline: Instant::now() + Duration::from_secs(60),
            next_id: 1,
            now: 0,
            read: 0,
            written: 0,
            hello: Value::Null,
            exchanges: Vec::new(),
        };
        source.hello = source.read().await?;
        require(
            source.hello["schema"] == "retinue.retained-peer/v1" && source.hello["kind"] == "hello",
            "persistent peer hello",
        )?;
        Ok(source)
    }

    async fn read(&mut self) -> Result<Value> {
        let deadline = self.deadline.min(Instant::now() + Duration::from_secs(10));
        let mut line = Vec::new();
        timeout_at(
            deadline,
            (&mut self.output).take(65537).read_until(b'\n', &mut line),
        )
        .await??;
        require(
            !line.is_empty() && line.len() <= 65536 && line.last() == Some(&b'\n'),
            "peer line bound",
        )?;
        self.read += line.len();
        require(self.read <= 4 * 1024 * 1024, "peer output lifetime budget")?;
        Ok(serde_json::from_slice(&line)?)
    }

    pub async fn command(&mut self, command: Value) -> Result<Value> {
        require(self.next_id <= 64, "composition IPC request budget")?;
        let request = json!({"schema":"retinue.retained-peer/v1","request_id":self.next_id,
            "now_slot":self.now,"command":command});
        let mut bytes = serde_json::to_vec(&request)?;
        bytes.push(b'\n');
        self.written += bytes.len();
        require(
            bytes.len() <= 65536 && self.written <= 4 * 1024 * 1024,
            "peer input budget",
        )?;
        let deadline = self.deadline.min(Instant::now() + Duration::from_secs(10));
        timeout_at(deadline, async {
            let input = self.input.as_mut().ok_or("closed peer stdin")?;
            input.write_all(&bytes).await?;
            input.flush().await?;
            Ok::<_, crate::Error>(())
        })
        .await??;
        let response = self.read().await?;
        require(
            response["schema"] == "retinue.retained-peer/v1"
                && response["kind"] == "response"
                && response["request_id"] == self.next_id,
            "peer response identity",
        )?;
        let now = u32::try_from(response["now_slot"].as_u64().ok_or("peer time")?)?;
        require(now >= self.now && now <= 8192, "peer monotonic epoch")?;
        self.now = now;
        self.next_id += 1;
        self.exchanges
            .push(json!({"request":request,"response":response}));
        Ok(response)
    }

    pub async fn ok(&mut self, command: Value) -> Result<Value> {
        let response = self.command(command).await?;
        require(
            response["status"] == "ok",
            &format!("peer refusal: {}", response["refusal"]),
        )?;
        Ok(response["result"].clone())
    }

    pub async fn finish(mut self) -> Result<Value> {
        drop(self.input.take());
        let status = timeout_at(
            self.deadline.min(Instant::now() + Duration::from_secs(5)),
            self.child.wait(),
        )
        .await??;
        require(status.success(), "peer exit status")?;
        let diagnostics = self.stderr.await??;
        require(diagnostics.len() <= 65536, "peer stderr bound")?;
        Ok(
            json!({"hello":self.hello,"exchanges":self.exchanges,"final_now_slot":self.now,
            "request_octets":self.written,"response_octets":self.read,"requests":self.next_id-1,
            "exited_successfully":true,"stderr":String::from_utf8_lossy(&diagnostics)}),
        )
    }
}
