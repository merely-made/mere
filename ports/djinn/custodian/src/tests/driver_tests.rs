// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The contract over real Unix sockets, with peers the OS reports.

#![cfg(unix)]

use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use custody_wire::contract::{Answer, Outcome, Purpose, ServerMessage};
use custody_wire::frame::{read_message, write_message};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::transport::{UnixSocketConnector, UnixSocketListener};
use custody_wire::{Client, ClientError, ClientMessage, Operation, ServerExpectation};
use insigne::delegation::{CapabilityScope, DelegationCertificate, DelegationParent, delegation_signing_salt};

use super::{PASSPHRASE, vault};
use crate::consent::Scripted;
use crate::core::Core;
use crate::driver::Service;
use crate::policy::Config;

const WAIT: Duration = Duration::from_secs(20);

fn euid() -> u32 {
    custody_wire::peer::effective_uid()
}

fn serve(dir: &std::path::Path, decisions: &[bool]) -> std::path::PathBuf {
    let uid = euid();
    let json = serde_json::json!({
        "vault": {"directory": dir}, "owner": {"unix_uid": uid}, "consent": "polkit", "development": true,
        "principals": [{"name": "djinn", "match": {"unix-uid": uid}, "grants": [
            {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"},
            {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}], "consent": "none"},
            {"operation": "sign", "profile": "current", "purpose": {"purpose": "delegation", "audience": "mere.graphshell"}, "consent": "per-operation"}
        ]}]
    });
    let config = Config::parse(&serde_json::to_vec(&json).unwrap()).unwrap();
    let socket = dir.join("sockets");
    std::fs::create_dir(&socket).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = socket.join("custody.sock");
    let listener = UnixSocketListener::bind(&path).unwrap();
    let mechanism = Arc::new(Scripted(Mutex::new(decisions.iter().copied().collect::<VecDeque<_>>())));
    let service = Service::new(Core::start(config).unwrap(), Some(mechanism));
    std::thread::spawn(move || service.serve(&listener));
    path
}

fn expect() -> ServerExpectation {
    ServerExpectation {
        unix_uids: vec![euid()],
        windows_sids: vec![],
    }
}

#[test]
fn the_contract_runs_end_to_end_over_a_socket_with_os_peers() {
    let (dir, master) = vault();
    let path = serve(dir.path(), &[true, false]);
    let connector = UnixSocketConnector::new(&path);
    let a = Client::open(&connector, &expect()).unwrap();
    assert!(a.session().locked);
    assert_eq!(a.session().principal, "djinn");
    let unlocked = a.call(Operation::Unlock { passphrase: Secret::copy_from(PASSPHRASE.as_bytes()) }, WAIT);
    assert!(matches!(unlocked, Outcome::Success { answer: Answer::Unlocked { .. } }), "{unlocked:?}");
    let b = Client::open(&connector, &expect()).unwrap();
    assert!(!b.session().locked);

    let released = a.call(Operation::Release { profile: "work".into(), salts: vec![Bytes(b"mesh-author".to_vec())] }, WAIT);
    assert!(matches!(released, Outcome::Success { answer: Answer::Released { master_public_key, .. } } if master_public_key == master));

    let statement = DelegationCertificate::new(
        DelegationParent::Root([1; 32]), master, [2; 32],
        CapabilityScope { domain: "mere.graphshell".into(), resource: b"room".to_vec(), path_prefix: "/".into(), actions: BTreeSet::from(["visit".to_string()]) },
        5, 5, None, 0, [4; 32],
    );
    let sign = || Operation::Sign {
        profile: "work".into(),
        purpose: Purpose::Delegation { audience: "mere.graphshell".into() },
        salt: Bytes(delegation_signing_salt(&statement.scope)),
        message: Bytes(statement.signing_bytes()),
    };
    assert!(matches!(a.call(sign(), WAIT), Outcome::Success { answer: Answer::Signature { .. } }), "the mechanism approved");
    assert_eq!(a.call(sign(), WAIT).code(), Some("consent-denied"), "the mechanism denied");

    let seen = a.observed().generation;
    let locked = b.call(Operation::Lock {}, WAIT);
    assert!(matches!(locked, Outcome::Success { answer: Answer::Lock { .. } }), "{locked:?}");
    let observed = a.wait_for_change(seen, WAIT);
    assert!(observed.locked && observed.generation > seen, "the other session sees the lock fact");
    assert_eq!(a.call_at(Operation::Release { profile: "work".into(), salts: vec![Bytes(b"mesh-author".to_vec())] }, seen, WAIT).code(), Some("stale-generation"));
    assert_eq!(a.call(Operation::Release { profile: "work".into(), salts: vec![Bytes(b"mesh-author".to_vec())] }, WAIT).code(), Some("locked"));
}

#[test]
fn a_forged_approval_field_is_an_invalid_request_and_a_wrong_server_gets_nothing() {
    let (dir, _) = vault();
    let path = serve(dir.path(), &[]);
    let mut raw = std::os::unix::net::UnixStream::connect(&path).unwrap();
    write_message(&mut raw, &ClientMessage::Hello { version: custody_wire::VERSION, nonce: "raw".into() }).unwrap();
    let Some(ServerMessage::Opened(session)) = read_message::<_, ServerMessage>(&mut raw).unwrap() else {
        panic!("not opened")
    };
    let forged = serde_json::json!({"type": "request", "id": 77, "instance": session.instance, "generation": session.generation,
        "operation": {"op": "lock", "approved": true}});
    let body = serde_json::to_vec(&forged).unwrap();
    use std::io::Write;
    raw.write_all(&(body.len() as u32).to_ne_bytes()).unwrap();
    raw.write_all(&body).unwrap();
    let reply = read_message::<_, ServerMessage>(&mut raw).unwrap().unwrap();
    assert!(matches!(reply, ServerMessage::Outcome { request: 77, outcome: Outcome::Refused { code }, .. } if code == "invalid-request"));

    let wrong = ServerExpectation { unix_uids: vec![euid().wrapping_add(1)], windows_sids: vec![] };
    assert!(matches!(Client::open(&UnixSocketConnector::new(&path), &wrong), Err(ClientError::WrongServer(_))));
}

#[test]
fn a_silent_connection_is_closed_at_the_handshake_deadline() {
    let (dir, _) = vault();
    let path = serve(dir.path(), &[]);
    let mut raw = std::os::unix::net::UnixStream::connect(&path).unwrap();
    raw.set_read_timeout(Some(custody_wire::limits::HANDSHAKE + Duration::from_secs(5))).unwrap();
    let started = std::time::Instant::now();
    let mut byte = [0u8; 1];
    let read = std::io::Read::read(&mut raw, &mut byte);
    assert!(matches!(read, Ok(0)), "closed, not timed out: {read:?}");
    assert!(started.elapsed() >= custody_wire::limits::HANDSHAKE - Duration::from_millis(200));
}
