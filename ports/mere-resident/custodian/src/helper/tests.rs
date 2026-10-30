// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Windows consent helper against the real service. The listener tells
//! each accepted connection it is a Windows peer (the OS fact a Windows
//! pipe would report); everything else, the policy, the core, the driver and
//! the client, is the code that runs on Windows.

#![cfg(unix)]

use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use custody_wire::contract::{Answer, Outcome};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::transport::{Connection, Listener, UnixSocketConnector, UnixSocketListener};
use custody_wire::{Client, ClientError, Operation, Peer, Petition, ServerExpectation};

use super::{Retry, Verifier, message, run, serve_session};
use crate::core::Core;
use crate::driver::Service;
use crate::policy::Config;
use crate::tests::{PASSPHRASE, vault};

const OWNER: &str = "S-1-5-21-1000-2000-3000-1001";
const HELPER_IMAGE: &str = r"C:\Program Files\Merely\djinn-custodian\djinn-custodian.exe";
const WAIT: Duration = Duration::from_secs(20);

fn windows(image: &str) -> Peer {
    Peer::Windows {
        sid: OWNER.into(),
        pid: 4242,
        image: Some(image.into()),
    }
}

/// A Unix listener whose connections report the next scripted Windows peer.
struct Pretend {
    inner: UnixSocketListener,
    peers: Mutex<VecDeque<Peer>>,
}

impl Listener for Pretend {
    fn accept(&self) -> std::io::Result<Connection> {
        let mut connection = self.inner.accept()?;
        if let Some(peer) = self.peers.lock().unwrap().pop_front() {
            connection.peer = peer;
        }
        Ok(connection)
    }
}

fn config(dir: &std::path::Path) -> Config {
    let json = serde_json::json!({
        "vault": {"directory": dir}, "owner": {"windows_sid": OWNER}, "consent": "channel",
        "principals": [
            {"name": "consent", "match": {"windows-image": {"sid": OWNER, "image": HELPER_IMAGE}}, "consent_channel": true,
             "grants": [{"operation": "status"}]},
            {"name": "djinn", "match": {"windows-sid": OWNER}, "grants": [
                {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"},
                {"operation": "release", "profile": "current", "salts": [{"exact": "mesh-author"}], "consent": "per-generation"}
            ]}
        ]
    });
    Config::parse(&serde_json::to_vec(&json).unwrap()).unwrap()
}

/// A running service whose next accepted connections are `peers`, in order.
fn serve(dir: &std::path::Path, peers: Vec<Peer>) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let sockets = dir.join("sockets");
    std::fs::create_dir(&sockets).unwrap();
    std::fs::set_permissions(&sockets, std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = sockets.join("custody.sock");
    let listener = Pretend {
        inner: UnixSocketListener::bind(&path).unwrap(),
        peers: Mutex::new(peers.into()),
    };
    let service = Service::new(Core::start(config(dir)).unwrap(), None);
    std::thread::spawn(move || service.serve(&listener));
    path
}

fn open(path: &std::path::Path) -> Client {
    let expect = ServerExpectation {
        unix_uids: vec![custody_wire::peer::effective_uid()],
        ..ServerExpectation::default()
    };
    Client::open(&UnixSocketConnector::new(path), &expect).unwrap()
}

fn release() -> Operation {
    Operation::Release {
        profile: "work".into(),
        salts: vec![Bytes(b"mesh-author".to_vec())],
    }
}

fn unlock(client: &Client) {
    let outcome = client.call(Operation::Unlock { passphrase: Secret::copy_from(PASSPHRASE.as_bytes()) }, WAIT);
    assert!(matches!(outcome, Outcome::Success { answer: Answer::Unlocked { .. } }), "{outcome:?}");
}

/// Hands each petition to the test and waits for the test's answer.
struct Gate {
    seen: mpsc::Sender<Petition>,
    answers: Mutex<mpsc::Receiver<bool>>,
}

impl Verifier for Gate {
    fn verify(&self, petition: &Petition) -> bool {
        self.seen.send(petition.clone()).unwrap();
        self.answers.lock().unwrap().recv_timeout(WAIT).unwrap_or(false)
    }
}

#[test]
fn the_helper_puts_each_petition_to_the_owner_and_only_it_can_decide() {
    let (dir, master) = vault();
    let path = serve(dir.path(), vec![windows(&HELPER_IMAGE.to_lowercase().replace('\\', "/")), windows(r"C:\Users\mark\Downloads\djinn.exe"), windows(r"C:\Users\mark\AppData\Local\Temp\other.exe")]);
    let helper = Arc::new(open(&path));
    assert_eq!(helper.session().principal, "consent");
    assert!(helper.session().consent_channel, "the pinned image, however the path is spelled, is the channel");
    let djinn = Arc::new(open(&path));
    assert_eq!(djinn.session().principal, "djinn");
    assert!(!djinn.session().consent_channel);
    let impostor = open(&path);
    assert_eq!(impostor.session().principal, "djinn", "another image of the same user is an ordinary requester");

    let (seen, petitions) = mpsc::channel();
    let (answer, answers) = mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let serving = {
        let (helper, stop) = (Arc::clone(&helper), Arc::clone(&stop));
        let gate = Gate { seen, answers: Mutex::new(answers) };
        std::thread::spawn(move || serve_session(&helper, &gate, &stop))
    };
    unlock(&djinn);

    let asking = {
        let djinn = Arc::clone(&djinn);
        std::thread::spawn(move || djinn.call(release(), WAIT))
    };
    let petition = petitions.recv_timeout(WAIT).expect("the helper is petitioned");
    assert_eq!((petition.principal.as_str(), petition.operation.as_str(), petition.profile.as_str()), ("djinn", "release", "work"));
    assert!(message(&petition).contains("allow djinn to release"), "{}", message(&petition));

    // Neither the requester nor another process of the same user can decide.
    assert_eq!(djinn.decide_raw(petition.request, &petition.digest, petition.generation, true).ok(), Some(false));
    assert_eq!(impostor.decide_raw(petition.request, &petition.digest, petition.generation, true).ok(), Some(false));
    assert!(!asking.is_finished(), "the petition still waits for the channel");

    answer.send(true).unwrap();
    let released = asking.join().unwrap();
    assert!(matches!(released, Outcome::Success { answer: Answer::Released { master_public_key, .. } } if master_public_key == master), "{released:?}");

    // Consent was per generation: a lock and unlock ask the owner again, and a refusal is final.
    assert!(matches!(djinn.call(Operation::Lock {}, WAIT), Outcome::Success { answer: Answer::Lock { .. } }));
    unlock(&djinn);
    let asking = {
        let djinn = Arc::clone(&djinn);
        std::thread::spawn(move || djinn.call(release(), WAIT))
    };
    petitions.recv_timeout(WAIT).expect("asked again after the lock");
    answer.send(false).unwrap();
    assert_eq!(asking.join().unwrap().code(), Some("consent-denied"));

    stop.store(true, Ordering::Relaxed);
    helper.close();
    assert!(serving.join().unwrap(), "it was a consent channel session");
}

/// A verifier for sessions that must never be petitioned.
struct Never;

impl Verifier for Never {
    fn verify(&self, _: &Petition) -> bool {
        panic!("no petition reaches this session")
    }
}

#[test]
fn a_session_not_admitted_as_the_channel_is_not_served_and_a_lost_custodian_ends_it() {
    let (dir, _) = vault();
    let path = serve(dir.path(), vec![windows(r"C:\Users\mark\Downloads\djinn.exe"), windows(HELPER_IMAGE)]);
    let ordinary = open(&path);
    assert!(!serve_session(&ordinary, &Never, &AtomicBool::new(false)));

    let helper = Arc::new(open(&path));
    let stop = Arc::new(AtomicBool::new(false));
    let serving = {
        let (helper, stop) = (Arc::clone(&helper), Arc::clone(&stop));
        std::thread::spawn(move || serve_session(&helper, &Never, &stop))
    };
    std::thread::sleep(Duration::from_millis(200));
    assert!(!serving.is_finished());
    helper.close();
    assert!(serving.join().unwrap(), "the loop ends when the session is gone, without a stop");
}

#[test]
fn the_helper_connects_again_until_the_custodian_admits_it_and_reports_once_per_streak() {
    let (dir, _) = vault();
    let path = serve(dir.path(), vec![windows(HELPER_IMAGE)]);
    let attempts = AtomicUsize::new(0);
    let stop = AtomicBool::new(false);
    let reports = Mutex::new(Vec::<String>::new());
    let connect = || {
        if attempts.fetch_add(1, Ordering::Relaxed) < 3 {
            return Err(ClientError::Connect(std::io::Error::other("the custodian is not up yet")));
        }
        Ok(open(&path))
    };
    std::thread::scope(|scope| {
        scope.spawn(|| {
            let retry = Retry { first: Duration::from_millis(10), longest: Duration::from_millis(20) };
            run(&connect, &Never, &stop, retry, &|text| reports.lock().unwrap().push(text.to_string()));
        });
        let deadline = std::time::Instant::now() + WAIT;
        while attempts.load(Ordering::Relaxed) < 4 {
            assert!(std::time::Instant::now() < deadline, "the helper never connected");
            std::thread::sleep(Duration::from_millis(10));
        }
        std::thread::sleep(Duration::from_millis(100));
        stop.store(true, Ordering::Relaxed);
    });
    assert_eq!(attempts.load(Ordering::Relaxed), 4, "admitted on the fourth try and then served");
    assert_eq!(reports.lock().unwrap().len(), 1, "three failures in a row are one report: {:?}", reports.lock().unwrap());
}
