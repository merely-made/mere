// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::{Result, hex, require, unhex};
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

pub const MAX_PEER_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_OBJECT_BYTES: usize = 16 * 1024;
pub const CHILD_TIMEOUT_SECONDS: u64 = 60;
const MAX_STDERR_BYTES: usize = 64 * 1024;

pub struct Recovery {
    pub scenario: Value,
    pub response: Value,
}

impl Recovery {
    pub fn receiver(&self, lane: &str) -> Result<&Value> {
        self.response["report"]["receivers"]
            .as_array()
            .and_then(|receivers| receivers.iter().find(|r| r["id"] == lane))
            .ok_or_else(|| format!("missing modeled receiver {lane}").into())
    }

    pub fn recovered(&self, lane: &str, expected: &[u8]) -> Result<Vec<u8>> {
        let receiver = self.receiver(lane)?;
        let generation = &receiver["generations"][0];
        require(
            generation["completion_slot"].is_u64(),
            "object did not reconstruct",
        )?;
        let bytes = unhex(
            generation["recovered_object_hex"]
                .as_str()
                .ok_or("missing recovered opaque original")?,
            MAX_OBJECT_BYTES,
        )?;
        require(
            bytes == expected,
            "carrier changed recovered original bytes",
        )?;
        require(
            generation["reconstructed_blake3"] == blake3::hash(&bytes).to_hex().as_str(),
            "reported reconstructed digest differs from recovered bytes",
        )?;
        for counter in [
            "invalid_signatures",
            "malformed_frames",
            "reconstructed_digest_mismatches",
        ] {
            require(
                receiver[counter].as_u64() == Some(0),
                "outer carrier refused a frame",
            )?;
        }
        Ok(bytes)
    }

    pub fn observation(&self) -> Value {
        json!({"scenario":self.scenario,"peer_response":self.response})
    }
}

pub fn recover(peer: &Path, id: &str, original: &[u8], late_one: bool) -> Result<Recovery> {
    require(
        !original.is_empty() && original.len() <= MAX_OBJECT_BYTES,
        "single original byte budget",
    )?;
    let symbols = original.len().div_ceil(128);
    let source_drain_upper_bound = symbols.div_ceil(2);
    let join_slot = source_drain_upper_bound + 2;
    let receivers = if late_one {
        json!([{"id":"pinned-four","radios":4},
            {"id":"late-one","radios":1,"join_slot":join_slot}])
    } else {
        json!([{"id":"pinned-four","radios":4}])
    };
    let scenario = json!({
        "schema":"transport-conformance.simulation/v1", "id":id,
        "namespace":1701, "slots":512, "channels":4,
        "symbol_size":128, "source_blocks":1, "slot_octet_budget":512,
        "manifest_interval":7, "repair_floor":1,
        "generations":[{"id":1,"publish_slot":0,"expires_slot":512,
            "object_octets":original.len(),"original_hex":hex(original)}],
        "receivers":receivers
    });
    let response = invoke(peer, &json!({"op":"run","scenario":scenario}))?;
    require(
        response["kind"] == "report",
        "unexpected simulation response kind",
    )?;
    require(
        response["implementation"].is_object(),
        "peer omitted compiled implementation identity",
    )?;
    require(
        response["report"]["physical_throughput_claim"] == false,
        "unexpected physical-throughput claim",
    )?;
    require(
        response["report"]["transmitter"]["generations"][0]["fresh_sent"].as_u64()
            == Some(symbols as u64),
        "single source burst did not drain",
    )?;
    Ok(Recovery { scenario, response })
}

fn invoke(peer: &Path, request: &Value) -> Result<Value> {
    let bytes = serde_json::to_vec(request)?;
    require(bytes.len() <= MAX_PEER_BYTES, "peer request budget")?;
    let mut child = Command::new(peer)
        .arg("--simulation-peer")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let mut stdin = child.stdin.take().ok_or("missing child stdin")?;
    let stdout = child.stdout.take().ok_or("missing child stdout")?;
    let stderr = child.stderr.take().ok_or("missing child stderr")?;
    let writer = std::thread::spawn(move || stdin.write_all(&bytes));
    let output = std::thread::spawn(move || bounded_read(stdout, MAX_PEER_BYTES));
    let diagnostics = std::thread::spawn(move || bounded_read(stderr, MAX_STDERR_BYTES));
    let started = Instant::now();
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait()? {
            break (status, false);
        }
        if started.elapsed() >= Duration::from_secs(CHILD_TIMEOUT_SECONDS) {
            child.kill()?;
            break (child.wait()?, true);
        }
        std::thread::sleep(Duration::from_millis(10));
    };
    let written = writer.join().map_err(|_| "child writer thread failed")?;
    let output = output.join().map_err(|_| "child stdout reader failed")??;
    let diagnostics = diagnostics
        .join()
        .map_err(|_| "child stderr reader failed")??;
    require(!timed_out, "simulation child timed out")?;
    if !status.success() {
        return Err(format!(
            "simulation child {status}: {}",
            String::from_utf8_lossy(&diagnostics)
        )
        .into());
    }
    written?;
    require(output.len() <= MAX_PEER_BYTES, "peer response budget")?;
    require(diagnostics.len() <= MAX_STDERR_BYTES, "peer stderr budget")?;
    Ok(serde_json::from_slice(&output)?)
}

fn bounded_read(reader: impl Read, maximum: usize) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take((maximum + 1) as u64).read_to_end(&mut bytes)?;
    Ok(bytes)
}
