// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::io::Cursor;

use super::*;
use crate::frame::{read_frame, read_message, write_message};

#[test]
fn frames_round_trip_on_the_door_wire_and_refuse_oversize() {
    let message = ClientMessage::Hello {
        version: VERSION,
        nonce: "n".into(),
    };
    let mut bytes = Vec::new();
    write_message(&mut bytes, &message).unwrap();
    let body = serde_json::to_vec(&message).unwrap();
    assert_eq!(&bytes[..4], &(body.len() as u32).to_ne_bytes());
    let back: ClientMessage = read_message(&mut Cursor::new(bytes)).unwrap().unwrap();
    assert_eq!(back, message);

    let oversized = (limits::MAX_FRAME_BYTES + 1).to_ne_bytes().to_vec();
    assert!(matches!(
        read_frame(&mut Cursor::new(oversized)),
        Err(frame::FrameError::TooLarge(_))
    ));
    assert!(read_frame(&mut Cursor::new(Vec::new())).unwrap().is_none());
    assert!(read_frame(&mut Cursor::new(vec![1, 0])).is_err());
}

#[test]
fn decoders_refuse_unknown_fields_and_any_client_approval() {
    let forged = br#"{"type":"request","id":1,"instance":"i","generation":1,"operation":{"op":"sign","profile":"p","purpose":{"purpose":"root"},"salt":"","message":"","approved":true}}"#;
    assert!(frame::decode::<ClientMessage>(forged).is_err());
    let approve = br#"{"type":"request","id":1,"instance":"i","generation":1,"operation":{"op":"approve","ticket":"t"}}"#;
    assert!(frame::decode::<ClientMessage>(approve).is_err());
    let principal = br#"{"type":"hello","version":1,"nonce":"n","principal":"djinn"}"#;
    assert!(frame::decode::<ClientMessage>(principal).is_err());
}

#[test]
fn secrets_round_trip_redacted() {
    let secret = Secret::copy_from(b"correct horse");
    assert_eq!(format!("{secret:?}"), "Secret([redacted; 13])");
    let operation = Operation::Unlock {
        passphrase: secret.clone(),
    };
    let json = serde_json::to_vec(&operation).unwrap();
    let back: Operation = serde_json::from_slice(&json).unwrap();
    assert_eq!(back, operation);
}

#[test]
fn the_server_expectation_admits_only_the_installed_identity() {
    let expect = ServerExpectation {
        unix_uids: vec![0, 990],
        windows_sids: vec!["S-1-5-80-1".into()],
    };
    assert!(expect.admits(&Peer::Unix { uid: 990, gid: 990, pid: None }));
    assert!(!expect.admits(&Peer::Unix { uid: 1000, gid: 1000, pid: None }));
    assert!(expect.admits(&Peer::Windows { sid: "S-1-5-80-1".into(), pid: 4 }));
    assert!(!expect.admits(&Peer::Windows { sid: "S-1-5-21-9".into(), pid: 4 }));
    assert!(!expect.admits(&Peer::Signed { team: "T".into(), identifier: "i".into() }));
}

#[cfg(unix)]
mod unix {
    use std::os::fd::AsRawFd;
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    use super::*;
    use crate::transport::Connection;

    fn connection(stream: UnixStream) -> Connection {
        let peer = peer::unix_peer(stream.as_raw_fd()).unwrap();
        Connection {
            stream: Box::new(stream),
            peer,
        }
    }

    #[test]
    fn the_os_reports_the_peer_not_the_message() {
        let (a, _b) = UnixStream::pair().unwrap();
        match peer::unix_peer(a.as_raw_fd()).unwrap() {
            Peer::Unix { uid, pid, .. } => {
                assert_eq!(uid, peer::effective_uid());
                if let Some(pid) = pid {
                    assert_eq!(pid as u32, std::process::id());
                }
            },
            other => panic!("unexpected peer {other:?}"),
        }
    }

    #[test]
    fn a_client_sends_nothing_to_an_unexpected_server() {
        let (client, mut server) = UnixStream::pair().unwrap();
        let expect = ServerExpectation {
            unix_uids: vec![peer::effective_uid().wrapping_add(1)],
            windows_sids: vec![],
        };
        let refused = Client::over(connection(client), &expect);
        assert!(matches!(refused, Err(ClientError::WrongServer(_))));
        // The client shut its end down without writing: end of stream.
        let mut byte = [0u8; 1];
        assert_eq!(std::io::Read::read(&mut server, &mut byte).unwrap_or(0), 0);
    }

    #[test]
    fn a_lost_connection_is_an_unknown_outcome_never_success() {
        let (client, mut server) = UnixStream::pair().unwrap();
        let fake = std::thread::spawn(move || {
            let hello: ClientMessage = read_message(&mut server).unwrap().unwrap();
            let ClientMessage::Hello { nonce, .. } = hello else {
                panic!("expected hello")
            };
            let session = Session {
                version: VERSION,
                instance: "instance".into(),
                generation: 3,
                locked: false,
                principal: "djinn".into(),
                peer: Peer::Unix { uid: 1, gid: 1, pid: None },
                scopes: vec![],
                consent_channel: false,
                limits: Limits {
                    max_frame_bytes: limits::MAX_FRAME_BYTES,
                    max_queued_operations: 16,
                    noninteractive_ms: 30_000,
                    consent_ms: 120_000,
                },
                nonce,
            };
            write_message(&mut server, &ServerMessage::Opened(session)).unwrap();
            let request: ClientMessage = read_message(&mut server).unwrap().unwrap();
            let ClientMessage::Request { instance, generation, .. } = request else {
                panic!("expected request")
            };
            assert_eq!((instance.as_str(), generation), ("instance", 3));
            write_message(
                &mut server,
                &ServerMessage::Fact(Fact {
                    instance: "instance".into(),
                    generation: 4,
                    cause: FactCause::Lock,
                    locked: true,
                    report: None,
                    revoked: None,
                }),
            )
            .unwrap();
            // Drop the connection with the request unanswered.
        });
        let expect = ServerExpectation {
            unix_uids: vec![peer::effective_uid()],
            windows_sids: vec![],
        };
        let client = Client::over(connection(client), &expect).unwrap();
        assert_eq!(client.session().generation, 3);
        let outcome = client.call(Operation::Lock {}, Duration::from_secs(5));
        assert_eq!(outcome, Outcome::unknown("custodian-lost"));
        fake.join().unwrap();
        let observed = client.wait_for_change(3, Duration::from_secs(1));
        assert_eq!(observed.generation, 4, "the lock fact was folded before the loss");
        assert!(observed.locked);
        assert!(!client.observed().alive);
        assert_eq!(client.call(Operation::Status {}, Duration::from_secs(1)), Outcome::unknown("custodian-lost"));
    }
}
