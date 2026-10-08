// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! djinn's side of the shared harness: the binaries, the owner's settings,
//! pairing through the ordinary commands, and the directory and ticket read
//! the way djinn serves them. Everything else is `djinn_testkit`.

#![allow(dead_code)]

use std::collections::BTreeSet;
use std::time::Duration;

use djinn::resident_devices::{AddrKindV1, DeviceDirectoryV1, PairedDeviceV1};
use djinn_testkit::{Resident, ResidentStatus, Run, RunConfig, Unlock};
use personae::ProfileId;
use transport::PeerAddr;

pub const DJINN: &str = env!("CARGO_BIN_EXE_djinn");
pub const DEVICES: &str = env!("CARGO_BIN_EXE_djinn-devices");
/// Spawn to ready, under other sessions' builds.
pub const READY: Duration = Duration::from_secs(180);
/// A graceful stop, to process exit.
pub const STOP: Duration = Duration::from_secs(60);

pub fn begin(plan: &str, test: &str) -> Run {
    Run::begin(RunConfig::new(plan, test, DJINN)).expect("the run begins")
}

/// A resident with its own profile, unlocked by passphrase.
pub fn resident(run: &Run, name: &str, profile: &str, passphrase: &str) -> Resident {
    run.resident(name)
        .profile(profile)
        .unlock(Unlock::Passphrase(passphrase.into()))
}

/// The owner's statement: personal sync on one named graph, reservoir off.
pub fn configure(resident: &Resident, profile: &str, graph: &str) {
    let path = djinn::settings::settings_path(&resident.app_dir(), &ProfileId(profile.into()));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(
        &path,
        format!(r#"{{"sync":{{"graph":"{graph}"}},"reservoir":{{"enabled":false}}}}"#),
    )
    .unwrap();
}

/// `(node_id, root)` from `--pairing-facts`.
pub fn facts(resident: &Resident) -> (String, String) {
    let facts = resident.djinn(&["--pairing-facts"]);
    let field = |name: &str| {
        facts
            .lines()
            .find_map(|line| line.strip_prefix(name))
            .map(|value| value.trim().to_string())
            .unwrap_or_else(|| panic!("no {name} in {facts}"))
    };
    (field("node_id "), field("root "))
}

pub fn pair(resident: &Resident, node: &str, root: &str, label: &str) {
    let said = resident.djinn(&[
        "--pair-node",
        node,
        "--pair-root",
        root,
        "--pair-label",
        label,
    ]);
    assert!(said.starts_with("paired "), "{said}");
}

/// The directory as `djinn-devices --json` prints it, over this resident's
/// app door.
pub fn directory(resident: &Resident) -> Option<DeviceDirectoryV1> {
    let output = resident
        .output(std::path::Path::new(DEVICES), &["--json".to_string()])
        .ok()?;
    output
        .status
        .success()
        .then(|| serde_json::from_slice(&output.stdout).expect("djinn-devices prints JSON"))
}

pub fn human(resident: &Resident) -> String {
    let output = resident.output(std::path::Path::new(DEVICES), &[]).unwrap();
    String::from_utf8_lossy(&output.stdout).into_owned()
}

pub fn sees(resident: &Resident, node: &str) -> Option<PairedDeviceV1> {
    directory(resident)?
        .devices
        .into_iter()
        .find(|device| device.node_id == node)
}

/// The start's own ticket, from its status route.
pub fn ticket(status: &ResidentStatus) -> String {
    status
        .sync
        .as_ref()
        .expect("personal sync is on, so the status names a ticket")
        .ticket
        .clone()
}

/// The ports a start listens on, from the ticket its status reports.
pub fn listening_ports(status: &ResidentStatus) -> BTreeSet<u16> {
    let (_, addrs) = transport::decode_peer_ticket(&ticket(status)).unwrap();
    addrs
        .iter()
        .filter_map(|addr| match addr {
            PeerAddr::Direct(addr) => Some(addr.port()),
            _ => None,
        })
        .collect()
}

fn port_of(addr: &str) -> Option<u16> {
    addr.parse::<std::net::SocketAddr>()
        .ok()
        .map(|addr| addr.port())
}

pub fn active_direct_ports(device: &PairedDeviceV1) -> BTreeSet<u16> {
    device
        .path
        .iter()
        .filter(|addr| addr.active && addr.kind == AddrKindV1::Direct)
        .filter_map(|addr| port_of(&addr.addr))
        .collect()
}

pub fn hint_ports(device: &PairedDeviceV1) -> BTreeSet<u16> {
    device
        .hint
        .iter()
        .flat_map(|hint| &hint.addrs)
        .filter(|addr| addr.kind == AddrKindV1::Direct)
        .filter_map(|addr| port_of(&addr.addr))
        .collect()
}
