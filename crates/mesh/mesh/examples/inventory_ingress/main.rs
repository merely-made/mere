// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

mod campaign;
mod network;
mod source;

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Result<T> = std::result::Result<T, Error>;
pub const MESH: [u8; 32] = [0x4d; 32];

pub fn require(ok: bool, detail: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(detail.to_string().into())
    }
}
pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
pub fn unhex(text: &str) -> Result<Vec<u8>> {
    require(
        text.len() <= 32768 && text.len().is_multiple_of(2),
        "hex object bound",
    )?;
    text.as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair)?, 16).map_err(Into::into))
        .collect::<Result<Vec<_>>>()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() == 4 && args[1] == "--reopen" {
        let store = mesh::MeshStore::at_path(&args[2])?;
        let records: Vec<String> = serde_json::from_slice(&std::fs::read(&args[3])?)?;
        require(
            records.len() == 4
                && store.ops(MESH).await?.len() == 4
                && store.board(MESH).await?.jobs().count() == 4,
            "fresh process owner state",
        )?;
        for encoded in records {
            let bytes = unhex(&encoded)?;
            let record: stickleback::DropRecord =
                p2panda_core::cbor::decode_cbor_strict(bytes.as_slice())?;
            let original = stickleback::decode_operation_record::<mesh::MeshExt>(&record)?
                .ok_or("operation record")?;
            let stored = store
                .operation(&original.hash)
                .await?
                .ok_or("durable original")?;
            require(
                stored.header.encode() == original.header.encode()
                    && stored.body.as_ref().map(p2panda_core::Body::to_bytes)
                        == original.body.as_ref().map(p2panda_core::Body::to_bytes),
                "fresh process signed bytes",
            )?;
        }
        return Ok(());
    }
    require(
        args.len() == 3,
        "usage: inventory_ingress RETINUE_PEER NEW_REPORT.json",
    )?;
    let peer = PathBuf::from(&args[1]).canonicalize()?;
    let before = blake3::hash(&std::fs::read(&peer)?);
    let mut report =
        tokio::time::timeout(std::time::Duration::from_secs(120), campaign::run(&peer)).await??;
    require(
        before == blake3::hash(&std::fs::read(&peer)?),
        "peer changed during run",
    )?;
    report["peer_executable_blake3"] = serde_json::json!(before.to_hex().to_string());
    report["mere_source_blake3"] = serde_json::json!(source_hash());
    report["compiled_lock_blake3"] = serde_json::json!(
        blake3::hash(include_bytes!("../../../../../Cargo.lock"))
            .to_hex()
            .to_string()
    );
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&args[2])?;
    serde_json::to_writer_pretty(&mut output, &report)?;
    output.write_all(b"\n")?;
    println!(
        "Exact inventory, actual RaptorQ recovery and admitted owner handoff: {}",
        args[2]
    );
    Ok(())
}

fn source_hash() -> String {
    let mut h = blake3::Hasher::new();
    for source in [
        include_str!("main.rs"),
        include_str!("source.rs"),
        include_str!("network.rs"),
        include_str!("campaign.rs"),
        include_str!("../../src/ingress/mod.rs"),
        include_str!("../../src/ingress/budget.rs"),
        include_str!("../../src/ingress/session.rs"),
        include_str!("../../src/ingress/lifecycle.rs"),
        include_str!("../../src/ingress/records.rs"),
        include_str!("../../src/ingress/client.rs"),
        include_str!("../../../../murm/transport/src/p2panda_transport/bounded_accept.rs"),
        include_str!("../../../../murm/transport/src/p2panda_transport.rs"),
        include_str!("../../../../../ports/distillery/src/resident.rs"),
    ] {
        h.update(&(source.len() as u64).to_le_bytes());
        h.update(source.as_bytes());
    }
    h.finalize().to_hex().to_string()
}
