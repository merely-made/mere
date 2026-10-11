// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::Result;
use serde_json::{Value, json};
use std::io::Read;
use std::path::Path;
use std::process::Command;

const SOURCES: &[(&str, &[u8])] = &[
    (
        "crates/stickleback/Cargo.toml",
        include_bytes!("../../../../stickleback/Cargo.toml"),
    ),
    (
        "crates/stickleback/src/lib.rs",
        include_bytes!("../../../../stickleback/src/lib.rs"),
    ),
    (
        "crates/murm/transport/src/transport.rs",
        include_bytes!("../../../../murm/transport/src/transport.rs"),
    ),
    (
        "crates/mesh/mesh/examples/retained_receive/carrier.rs",
        include_bytes!("carrier.rs"),
    ),
    (
        "crates/stickleback/src/object_stream.rs",
        include_bytes!("../../../../stickleback/src/object_stream.rs"),
    ),
    (
        "crates/murm/transport/src/p2panda_transport.rs",
        include_bytes!("../../../../murm/transport/src/p2panda_transport.rs"),
    ),
    (
        "crates/murm/transport/src/accepted.rs",
        include_bytes!("../../../../murm/transport/src/accepted.rs"),
    ),
    (
        "crates/murm/transport/src/peer_route.rs",
        include_bytes!("../../../../murm/transport/src/peer_route.rs"),
    ),
    ("Cargo.lock", include_bytes!("../../../../../Cargo.lock")),
    ("Cargo.toml", include_bytes!("../../../../../Cargo.toml")),
    (
        "crates/mesh/mesh/Cargo.toml",
        include_bytes!("../../Cargo.toml"),
    ),
    (
        "crates/mesh/mesh/examples/retained_receive/main.rs",
        include_bytes!("main.rs"),
    ),
    (
        "crates/mesh/mesh/examples/retained_receive/campaign.rs",
        include_bytes!("campaign.rs"),
    ),
    (
        "crates/mesh/mesh/examples/retained_receive/peer.rs",
        include_bytes!("peer.rs"),
    ),
    (
        "crates/mesh/mesh/examples/retained_receive/identity.rs",
        include_bytes!("identity.rs"),
    ),
    (
        "crates/mesh/mesh/src/store.rs",
        include_bytes!("../../src/store.rs"),
    ),
    (
        "crates/mesh/mesh/src/wire.rs",
        include_bytes!("../../src/wire.rs"),
    ),
    (
        "crates/mesh/mesh/src/board.rs",
        include_bytes!("../../src/board.rs"),
    ),
    (
        "crates/mesh/mesh/src/spec.rs",
        include_bytes!("../../src/spec.rs"),
    ),
    (
        "crates/stickleback/src/processor.rs",
        include_bytes!("../../../../stickleback/src/processor.rs"),
    ),
    (
        "crates/stickleback/src/store.rs",
        include_bytes!("../../../../stickleback/src/store.rs"),
    ),
    (
        "crates/stickleback/src/drop_io.rs",
        include_bytes!("../../../../stickleback/src/drop_io.rs"),
    ),
    (
        "crates/stickleback/src/drop.rs",
        include_bytes!("../../../../stickleback/src/drop.rs"),
    ),
    (
        "crates/eidetic/muniment/src/redb_backend.rs",
        include_bytes!("../../../../eidetic/muniment/src/redb_backend.rs"),
    ),
    (
        "crates/eidetic/muniment/src/backend.rs",
        include_bytes!("../../../../eidetic/muniment/src/backend.rs"),
    ),
];

pub fn file_digest(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut digest = blake3::Hasher::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let length = file.read(&mut buffer)?;
        if length == 0 {
            break;
        }
        digest.update(&buffer[..length]);
    }
    Ok(digest.finalize().to_hex().to_string())
}

pub fn mere_identity() -> Result<Value> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
    let mut digest = blake3::Hasher::new();
    let mut sources = Vec::new();
    for (path, bytes) in SOURCES {
        digest.update(&(path.len() as u64).to_be_bytes());
        digest.update(path.as_bytes());
        digest.update(&(bytes.len() as u64).to_be_bytes());
        digest.update(bytes);
        sources.push(json!({"path":path,"octets":bytes.len(),
            "blake3":blake3::hash(bytes).to_hex().as_str()}));
    }
    Ok(json!({
        "scope":"compile_time_probe_and_selected_owner_and_carrier_sources",
        "source_manifest_blake3":digest.finalize().to_hex().as_str(),
        "sources":sources,
        "executable_blake3":file_digest(&std::env::current_exe()?)?,
        "observed_git_head":git(&root, &["rev-parse","HEAD"]),
        "observed_git_status_porcelain":git(&root, &["status","--porcelain=v1"]),
        "git_observation_timing":"at_probe_execution",
        "package_version":env!("CARGO_PKG_VERSION"),
        "rust_target_arch":std::env::consts::ARCH,
        "host_os":std::env::consts::OS
    }))
}

fn git(root: &Path, args: &[&str]) -> Option<String> {
    let result = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    result
        .status
        .success()
        .then(|| String::from_utf8_lossy(&result.stdout).trim().to_owned())
}
