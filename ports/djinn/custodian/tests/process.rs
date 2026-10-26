// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custodian as its own process: the built binary, a real vault, a
//! socket and the custody-wire client, with no GUI. Development mode binds
//! its own socket; confinement applies what the host supports.

#![cfg(unix)]

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use custody_wire::contract::{Answer, Outcome, ServerMessage};
use custody_wire::frame::{read_message, write_message};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::transport::UnixSocketConnector;
use custody_wire::{Client, ClientMessage, Operation, ServerExpectation};

const PASSPHRASE: &str = "process test passphrase";
const WAIT: Duration = Duration::from_secs(30);

struct Fixture {
    _dir: tempfile::TempDir,
    root: PathBuf,
    config: PathBuf,
    socket: PathBuf,
}

fn uid() -> u32 {
    custody_wire::peer::effective_uid()
}

fn fixture(development: bool) -> Fixture {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().to_path_buf();
    let vault = root.join("state/vault");
    std::fs::create_dir_all(&vault).unwrap();
    let sockets = root.join("run");
    std::fs::create_dir(&sockets).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&sockets, std::fs::Permissions::from_mode(0o700)).unwrap();
    let config = root.join("custodian.json");
    let consent = if development { "none" } else { "per-generation" };
    let json = serde_json::json!({
        "vault": {"directory": vault, "profile": "work"}, "owner": {"unix_uid": uid()},
        "consent": "deny", "development": development,
        "principals": [{"name": "djinn", "match": {"unix-uid": uid()}, "grants": [
            {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"},
            {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}], "consent": consent}
        ]}]
    });
    std::fs::write(&config, serde_json::to_vec_pretty(&json).unwrap()).unwrap();
    Fixture {
        _dir: dir,
        root,
        config,
        socket: sockets.join("custody.sock"),
    }
}

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_djinn-custodian")
}

fn run_with_stdin(args: &[&str], stdin: &str) -> std::process::Output {
    let mut child = Command::new(binary()).args(args).stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().unwrap();
    child.stdin.take().unwrap().write_all(stdin.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

fn init(fixture: &Fixture) {
    let output = run_with_stdin(&["init", "--config", fixture.config.to_str().unwrap()], &format!("{PASSPHRASE}\n{PASSPHRASE}\n"));
    assert!(output.status.success(), "init: {}", String::from_utf8_lossy(&output.stderr));
}

struct Running(Child);

impl Drop for Running {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

fn start(fixture: &Fixture) -> Running {
    let child = Command::new(binary())
        .args(["--config", fixture.config.to_str().unwrap(), "--socket", fixture.socket.to_str().unwrap()])
        .env_clear()
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let running = Running(child);
    let deadline = Instant::now() + WAIT;
    while std::os::unix::net::UnixStream::connect(&fixture.socket).is_err() {
        assert!(Instant::now() < deadline, "the custodian never listened");
        std::thread::sleep(Duration::from_millis(50));
    }
    running
}

fn client(fixture: &Fixture) -> Client {
    let expect = ServerExpectation {
        unix_uids: vec![uid()],
        windows_sids: vec![],
    };
    Client::open(&UnixSocketConnector::new(&fixture.socket), &expect).unwrap()
}

fn unlock(client: &Client) {
    let outcome = client.call(Operation::Unlock { passphrase: Secret::copy_from(PASSPHRASE.as_bytes()) }, WAIT);
    assert!(matches!(outcome, Outcome::Success { answer: Answer::Unlocked { .. } }), "{outcome:?}");
}

fn release() -> Operation {
    Operation::Release {
        profile: "work".into(),
        salts: vec![Bytes(b"mesh-author".to_vec())],
    }
}

#[test]
fn the_custodian_process_starts_locked_and_serves_the_contract() {
    let fixture = fixture(true);
    init(&fixture);
    let _custodian = start(&fixture);
    let client = client(&fixture);
    assert!(client.session().locked, "every start is locked");
    assert_eq!(client.call(release(), WAIT).code(), Some("locked"));
    unlock(&client);
    assert!(matches!(client.call(release(), WAIT), Outcome::Success { answer: Answer::Released { .. } }));
    assert!(matches!(client.call(Operation::Lock {}, WAIT), Outcome::Success { answer: Answer::Lock { .. } }));
    assert_eq!(client.call(release(), WAIT).code(), Some("locked"));
}

#[test]
fn custodian_loss_is_unknown_and_a_restart_forces_current_admission() {
    let fixture = fixture(true);
    init(&fixture);
    let mut custodian = start(&fixture);
    let before = client(&fixture);
    unlock(&before);
    let old_instance = before.session().instance.clone();
    custodian.0.kill().unwrap();
    custodian.0.wait().unwrap();
    let lost = before.call(release(), WAIT);
    assert_eq!(lost, Outcome::unknown("custodian-lost"), "a disconnect is never success");
    assert!(!before.observed().alive);

    let _restarted = start(&fixture);
    let after = client(&fixture);
    assert_ne!(after.session().instance, old_instance, "a fresh instance");
    assert!(after.session().locked, "the restart is locked; the old unlock is gone");
    let mut raw = std::os::unix::net::UnixStream::connect(&fixture.socket).unwrap();
    write_message(&mut raw, &ClientMessage::Hello { version: custody_wire::VERSION, nonce: "replay".into() }).unwrap();
    let Some(ServerMessage::Opened(session)) = read_message::<_, ServerMessage>(&mut raw).unwrap() else {
        panic!("not opened")
    };
    write_message(&mut raw, &ClientMessage::Request { id: 1, instance: old_instance, generation: session.generation, operation: release() }).unwrap();
    let Some(ServerMessage::Outcome { outcome, .. }) = read_message::<_, ServerMessage>(&mut raw).unwrap() else {
        panic!("no outcome")
    };
    assert_eq!(outcome.code(), Some("stale-instance"));
}

#[test]
fn adopt_copies_djinns_vault_and_proves_the_same_identity_without_touching_the_source() {
    let fixture = fixture(true);
    let source = fixture.root.join("djinn-vault");
    let master = {
        use castellan::custody::bootstrap::{self, Unlock};
        let opened = bootstrap::open_storage(&source, Unlock::passphrase(PASSPHRASE)).unwrap();
        let (profile, _) = bootstrap::load_or_create_profile(&*opened.storage, &personae::ProfileId("work".into())).unwrap();
        profile.master.public_key().to_bytes()
    };
    let snapshot = |dir: &Path| -> Vec<(String, Vec<u8>)> {
        let mut entries: Vec<_> = std::fs::read_dir(dir).unwrap().map(|e| {
            let e = e.unwrap();
            (e.file_name().to_string_lossy().into_owned(), std::fs::read(e.path()).unwrap_or_default())
        }).collect();
        entries.sort();
        entries
    };
    let before = snapshot(&source);
    let output = run_with_stdin(&["adopt", "--config", fixture.config.to_str().unwrap(), "--from", source.to_str().unwrap()], &format!("{PASSPHRASE}\n"));
    assert!(output.status.success(), "adopt: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(snapshot(&source), before, "the source vault is byte-identical, no marker written");
    let _custodian = start(&fixture);
    let client = client(&fixture);
    unlock(&client);
    let Outcome::Success { answer: Answer::Status { status } } = client.call(Operation::Status {}, WAIT) else {
        panic!("no status")
    };
    assert_eq!(status.master_public_key, Some(master), "the same identity, now held by the custodian");
}

#[test]
fn a_protected_custodian_refuses_to_bind_its_own_socket() {
    let fixture = fixture(false);
    init(&fixture);
    let output = Command::new(binary())
        .args(["--config", fixture.config.to_str().unwrap(), "--socket", fixture.socket.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("service manager"));
    assert!(!fixture.socket.exists());
}
