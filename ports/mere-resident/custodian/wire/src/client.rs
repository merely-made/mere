// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custody client. It verifies the custodian's OS identity before
//! sending its hello, binds every request to the session's instance and
//! the latest generation it has observed, and reports a lost connection or
//! a missed deadline as [`Outcome::Unknown`]. One reader thread routes
//! outcomes to their callers and folds lock facts into [`Observed`], which
//! honest holders watch to forget released keys. A lost custodian is never
//! replaced by anything local: the client stays dead until its owner opens
//! a new one, which must pass current admission.

use std::collections::{HashMap, VecDeque};
use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::contract::{ClientMessage, Fact, Operation, Outcome, Petition, ServerMessage, Session};
use crate::frame::{read_message, write_message};
use crate::peer::{Peer, ServerExpectation};
use crate::transport::{Connection, Connector, Duplex};

/// Why a session could not be opened.
#[derive(Debug)]
pub enum ClientError {
    Connect(std::io::Error),
    /// The endpoint is served by something other than the installed
    /// custodian. Nothing was sent.
    WrongServer(Peer),
    /// The custodian refused admission.
    Refused(String),
    Protocol(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Connect(error) => write!(f, "cannot reach the custodian: {error}"),
            Self::WrongServer(peer) => write!(f, "the custody endpoint is not the installed custodian ({})", peer.account()),
            Self::Refused(code) => write!(f, "the custodian refused admission: {code}"),
            Self::Protocol(reason) => write!(f, "custody protocol error: {reason}"),
        }
    }
}

impl std::error::Error for ClientError {}

/// What the client currently knows about the custodian.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    /// False once the connection is gone; it never comes back.
    pub alive: bool,
    pub instance: String,
    pub generation: u64,
    pub locked: bool,
    pub last: Option<Fact>,
}

struct Shared {
    writer: Mutex<Box<dyn Duplex>>,
    control: Box<dyn Duplex>,
    pending: Mutex<HashMap<u64, mpsc::Sender<Outcome>>>,
    decided: Mutex<HashMap<u64, mpsc::Sender<bool>>>,
    petitions: Mutex<VecDeque<Petition>>,
    observed: Mutex<Observed>,
    changed: Condvar,
    next: AtomicU64,
}

/// One admitted custody session.
pub struct Client {
    shared: Arc<Shared>,
    session: Session,
    reader: Option<std::thread::JoinHandle<()>>,
}

impl Client {
    /// Connect, verify the server and open a session.
    pub fn open(connector: &dyn Connector, expect: &ServerExpectation) -> Result<Self, ClientError> {
        let connection = connector.connect().map_err(ClientError::Connect)?;
        Self::over(connection, expect)
    }

    /// Open a session over an already-connected stream.
    pub fn over(connection: Connection, expect: &ServerExpectation) -> Result<Self, ClientError> {
        let Connection { mut stream, peer } = connection;
        if !expect.admits(&peer) {
            stream.shutdown_both();
            return Err(ClientError::WrongServer(peer));
        }
        let control = stream.try_clone_duplex().map_err(ClientError::Connect)?;
        let nonce = correlation_nonce();
        let session = {
            let watchdog = Watchdog::arm(control.try_clone_duplex().map_err(ClientError::Connect)?, crate::limits::HANDSHAKE);
            write_message(&mut stream, &ClientMessage::Hello { version: crate::VERSION, nonce: nonce.clone() })
                .map_err(|e| ClientError::Protocol(e.to_string()))?;
            let reply: Option<ServerMessage> = read_message(&mut stream).map_err(|e| ClientError::Protocol(e.to_string()))?;
            watchdog.disarm();
            match reply {
                Some(ServerMessage::Opened(session)) if session.nonce == nonce && session.version == crate::VERSION => session,
                Some(ServerMessage::Opened(_)) => return Err(ClientError::Protocol("the session answered another hello".into())),
                Some(ServerMessage::Refused { code }) => return Err(ClientError::Refused(code)),
                Some(_) => return Err(ClientError::Protocol("unexpected first message".into())),
                None => return Err(ClientError::Protocol("the custodian closed during the handshake".into())),
            }
        };
        let mut reader_stream = stream.try_clone_duplex().map_err(ClientError::Connect)?;
        let shared = Arc::new(Shared {
            writer: Mutex::new(stream),
            control,
            pending: Mutex::new(HashMap::new()),
            decided: Mutex::new(HashMap::new()),
            petitions: Mutex::new(VecDeque::new()),
            observed: Mutex::new(Observed {
                alive: true,
                instance: session.instance.clone(),
                generation: session.generation,
                locked: session.locked,
                last: None,
            }),
            changed: Condvar::new(),
            next: AtomicU64::new(1),
        });
        let reader = {
            let shared = Arc::clone(&shared);
            std::thread::Builder::new()
                .name("custody-client".into())
                .spawn(move || read_loop(&shared, &mut *reader_stream))
                .map_err(ClientError::Connect)?
        };
        Ok(Self {
            shared,
            session,
            reader: Some(reader),
        })
    }

    /// The admitted session.
    pub fn session(&self) -> &Session {
        &self.session
    }

    /// The latest observed state.
    pub fn observed(&self) -> Observed {
        lock(&self.shared.observed).clone()
    }

    /// Run one operation against the latest observed generation.
    pub fn call(&self, operation: Operation, deadline: Duration) -> Outcome {
        let generation = {
            let observed = lock(&self.shared.observed);
            if !observed.alive {
                return Outcome::unknown("custodian-lost");
            }
            observed.generation
        };
        self.call_at(operation, generation, deadline)
    }

    /// Run one operation naming an explicit expected generation.
    pub fn call_at(&self, operation: Operation, generation: u64, deadline: Duration) -> Outcome {
        let id = self.shared.next.fetch_add(1, Ordering::Relaxed);
        let (send, receive) = mpsc::channel();
        lock(&self.shared.pending).insert(id, send);
        let request = ClientMessage::Request {
            id,
            instance: self.session.instance.clone(),
            generation,
            operation,
        };
        if write_message(&mut **lock(&self.shared.writer), &request).is_err() {
            lock(&self.shared.pending).remove(&id);
            return Outcome::unknown("custodian-lost");
        }
        match receive.recv_timeout(deadline) {
            Ok(outcome) => outcome,
            Err(mpsc::RecvTimeoutError::Timeout) => {
                lock(&self.shared.pending).remove(&id);
                Outcome::unknown("deadline")
            },
            Err(mpsc::RecvTimeoutError::Disconnected) => Outcome::unknown("custodian-lost"),
        }
    }

    /// Wait until the generation passes `seen` or the custodian is lost.
    pub fn wait_for_change(&self, seen: u64, timeout: Duration) -> Observed {
        let deadline = Instant::now() + timeout;
        let mut observed = lock(&self.shared.observed);
        while observed.alive && observed.generation <= seen {
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() {
                break;
            }
            observed = self
                .shared
                .changed
                .wait_timeout(observed, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
        observed.clone()
    }

    /// The next consent petition, for a session admitted as the trusted
    /// consent channel.
    pub fn next_petition(&self, timeout: Duration) -> Option<Petition> {
        let deadline = Instant::now() + timeout;
        let mut queue = lock(&self.shared.petitions);
        loop {
            if let Some(petition) = queue.pop_front() {
                return Some(petition);
            }
            let left = deadline.saturating_duration_since(Instant::now());
            if left.is_zero() || !lock(&self.shared.observed).alive {
                return None;
            }
            queue = self
                .shared
                .changed
                .wait_timeout(queue, left)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }

    /// Answer one petition. `Ok(true)` when the custodian recorded it.
    pub fn decide(&self, petition: &Petition, accept: bool) -> Result<bool, ClientError> {
        self.decide_raw(petition.request, &petition.digest, petition.generation, accept)
    }

    /// Send a decision with arbitrary bindings (negative controls).
    pub fn decide_raw(&self, request: u64, digest: &str, generation: u64, accept: bool) -> Result<bool, ClientError> {
        let (send, receive) = mpsc::channel();
        lock(&self.shared.decided).insert(request, send);
        let message = ClientMessage::Decide {
            request,
            digest: digest.into(),
            generation,
            accept,
        };
        write_message(&mut **lock(&self.shared.writer), &message).map_err(|e| ClientError::Protocol(e.to_string()))?;
        receive
            .recv_timeout(crate::limits::NONINTERACTIVE)
            .map_err(|_| ClientError::Protocol("no decision receipt".into()))
    }

    /// Write raw bytes as one frame body (negative controls).
    pub fn send_raw(&self, body: &[u8]) -> std::io::Result<()> {
        let mut writer = lock(&self.shared.writer);
        writer.write_all(&(body.len() as u32).to_ne_bytes())?;
        writer.write_all(body)?;
        writer.flush()
    }

    /// Close the session.
    pub fn close(&self) {
        self.shared.control.shutdown_both();
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.close();
        if let Some(reader) = self.reader.take() {
            let _ = reader.join();
        }
    }
}

fn read_loop(shared: &Shared, stream: &mut dyn Duplex) {
    loop {
        let message: ServerMessage = match read_message(stream) {
            Ok(Some(message)) => message,
            _ => break,
        };
        match message {
            ServerMessage::Outcome { request, outcome, .. } => {
                if let Some(waiter) = lock(&shared.pending).remove(&request) {
                    let _ = waiter.send(outcome);
                }
            },
            ServerMessage::Fact(fact) => {
                let mut observed = lock(&shared.observed);
                if fact.instance == observed.instance && fact.generation > observed.generation {
                    observed.generation = fact.generation;
                    observed.locked = fact.locked;
                    observed.last = Some(fact);
                }
                drop(observed);
                shared.changed.notify_all();
            },
            ServerMessage::Petition(petition) => {
                lock(&shared.petitions).push_back(petition);
                shared.changed.notify_all();
            },
            ServerMessage::Decided { request, recorded, .. } => {
                if let Some(waiter) = lock(&shared.decided).remove(&request) {
                    let _ = waiter.send(recorded);
                }
            },
            ServerMessage::Opened(_) | ServerMessage::Refused { .. } => break,
        }
    }
    lock(&shared.observed).alive = false;
    for (_, waiter) in lock(&shared.pending).drain() {
        let _ = waiter.send(Outcome::unknown("custodian-lost"));
    }
    lock(&shared.decided).clear();
    shared.changed.notify_all();
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

/// Correlation context only (§5.6): unique per hello, not a secret.
fn correlation_nonce() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}-{:x}-{:x}", std::process::id(), nanos, COUNTER.fetch_add(1, Ordering::Relaxed))
}

/// Shuts a stream down unless disarmed before its deadline.
pub struct Watchdog {
    disarm: Option<mpsc::Sender<()>>,
}

impl Watchdog {
    /// Shut `stream` down after `deadline` unless disarmed.
    pub fn arm(stream: Box<dyn Duplex>, deadline: Duration) -> Self {
        let (send, receive) = mpsc::channel::<()>();
        std::thread::spawn(move || {
            if let Err(mpsc::RecvTimeoutError::Timeout) = receive.recv_timeout(deadline) {
                stream.shutdown_both();
            }
        });
        Self { disarm: Some(send) }
    }

    /// Cancel the deadline.
    pub fn disarm(mut self) {
        if let Some(send) = self.disarm.take() {
            let _ = send.send(());
        }
    }
}

impl Drop for Watchdog {
    fn drop(&mut self) {
        if let Some(send) = self.disarm.take() {
            let _ = send.send(());
        }
    }
}
