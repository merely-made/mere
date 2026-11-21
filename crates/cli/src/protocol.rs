// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::fs::File;
use std::io::{Read, Write};
use std::net::SocketAddr;
use std::path::Path;
use std::time::Duration;

use anyhow::{Context, Result, bail, ensure};
use mere::dataset::{IndexedDataset, View};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::time::timeout;
use transport::{PeerID, security::SetupGuard};

pub const PROTOCOL: &str = "mere/sparql-rows/v1";
const MAX_QUERY: usize = 64 * 1024;
const MAX_REPLY: usize = 1024 * 1024;
const EXCHANGE: Duration = Duration::from_secs(90);

pub type Rows = Vec<Vec<Option<String>>>;

#[derive(Serialize, Deserialize)]
#[serde(tag = "via", rename_all = "lowercase", deny_unknown_fields)]
pub enum Route {
    Iroh {
        ticket: String,
    },
    Reticulum {
        address: SocketAddr,
        destination: [u8; 32],
    },
    I2p {
        destination: String,
    },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Invitation {
    pub protocol: String,
    pub owner: [u8; 32],
    pub route: Route,
    token: [u8; 32],
}

impl Invitation {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.protocol == PROTOCOL, "unsupported invitation protocol");
        PeerID::from_bytes(&self.owner)?;
        Ok(())
    }
}

pub fn read_small(path: &Path, limit: usize) -> Result<Vec<u8>> {
    let mut data = Vec::new();
    File::open(path)
        .with_context(|| format!("open {}", path.display()))?
        .take(limit as u64 + 1)
        .read_to_end(&mut data)?;
    ensure!(
        data.len() <= limit,
        "{} exceeds {limit} bytes",
        path.display()
    );
    Ok(data)
}

pub fn read_query(path: &Path) -> Result<String> {
    let text = String::from_utf8(read_small(path, MAX_QUERY)?).context("query must be UTF-8")?;
    ensure!(!text.trim().is_empty(), "query is empty");
    Ok(text.replace("\r\n", "\n"))
}

pub fn evaluate(dataset: &IndexedDataset, query: &str) -> Result<Rows> {
    Ok(
        mere::query::sparql_on(dataset, query, View::Live, Default::default())
            .map_err(anyhow::Error::msg)?
            .rows,
    )
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Reply {
    Rows(Rows),
    Error(String),
}

pub struct Service {
    dataset: IndexedDataset,
    allowed: String,
    token: [u8; 32],
}

impl Service {
    pub fn new(dataset: IndexedDataset, allowed: String) -> Result<Self> {
        let mut token = [0; 32];
        getrandom::fill(&mut token).map_err(|e| anyhow::anyhow!("generate invitation: {e}"))?;
        Ok(Self {
            dataset,
            allowed,
            token,
        })
    }

    pub fn invite(&self, file: &Path, owner: PeerID, route: Route) -> Result<()> {
        let dir = file
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut tmp = tempfile::NamedTempFile::new_in(dir)?;
        serde_json::to_writer(
            &mut tmp,
            &Invitation {
                protocol: PROTOCOL.to_string(),
                owner: owner.to_bytes(),
                route,
                token: self.token,
            },
        )?;
        tmp.flush()?;
        tmp.persist_noclobber(file)
            .context("write a new private invitation")?;
        eprintln!(
            "Ready. Share {} privately with the reader. Press Ctrl-C to stop.",
            file.display()
        );
        Ok(())
    }

    pub async fn answer<S: AsyncRead + AsyncWrite + Unpin>(
        &self,
        mut stream: S,
        guard: SetupGuard,
    ) -> Result<()> {
        timeout(EXCHANGE, async {
            let token = read_frame(&mut stream, 32).await?;
            if !bool::from(self.token.as_slice().ct_eq(&token)) {
                guard.refused();
                bail!("invalid invitation");
            }
            guard.admitted()?;
            let query = read_frame(&mut stream, MAX_QUERY).await?;
            let reply = if query != self.allowed.as_bytes() {
                Reply::Error("query is not enabled by this invitation".into())
            } else {
                match evaluate(&self.dataset, &self.allowed) {
                    Ok(rows) => Reply::Rows(rows),
                    Err(error) => Reply::Error(error.to_string()),
                }
            };
            let mut bytes = serde_json::to_vec(&reply)?;
            if bytes.len() > MAX_REPLY {
                bytes = serde_json::to_vec(&Reply::Error("result exceeds 1 MiB".into()))?;
            }
            write_frame(&mut stream, &bytes).await?;
            ensure!(
                stream.read_u8().await? == 1,
                "missing result acknowledgement"
            );
            stream.write_u8(1).await?;
            stream.shutdown().await?;
            Ok(())
        })
        .await
        .context("query session timed out")?
    }
}

async fn read_frame<S: AsyncRead + Unpin>(s: &mut S, max: usize) -> Result<Vec<u8>> {
    let len = s.read_u32().await? as usize;
    ensure!((1..=max).contains(&len), "invalid message size");
    let mut bytes = vec![0; len];
    s.read_exact(&mut bytes).await?;
    Ok(bytes)
}

async fn write_frame<S: AsyncWrite + Unpin>(s: &mut S, bytes: &[u8]) -> Result<()> {
    s.write_u32(u32::try_from(bytes.len())?).await?;
    s.write_all(bytes).await?;
    s.flush().await?;
    Ok(())
}

pub async fn request<S: AsyncRead + AsyncWrite + Unpin>(
    mut stream: S,
    invite: &Invitation,
    query: &str,
) -> Result<Rows> {
    timeout(EXCHANGE, async {
        write_frame(&mut stream, &invite.token).await?;
        write_frame(&mut stream, query.as_bytes()).await?;
        let reply: Reply = serde_json::from_slice(&read_frame(&mut stream, MAX_REPLY).await?)?;
        stream.write_u8(1).await?;
        stream.flush().await?;
        ensure!(stream.read_u8().await? == 1, "missing session completion");
        match reply {
            Reply::Rows(rows) => Ok(rows),
            Reply::Error(message) => bail!("peer: {message}"),
        }
    })
    .await
    .context("peer query timed out")?
}
