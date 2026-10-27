// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Serves the [`Core`] over any [`Listener`]. One thread per connection
//! reads frames; delivery never blocks the custodian: each session has its
//! own bounded outbox and writer thread, and a session whose outbox is full
//! is closed rather than allowed to stall a lock fact. A hello must arrive
//! within the handshake deadline. Petitions for a custodian-run consent
//! mechanism run on their own thread, one at a time, and come back through
//! [`Core::decide`]. A request that fails typed decoding is refused by id
//! when it names one; anything else closes the session.

use std::collections::HashMap;
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use custody_wire::client::Watchdog;
use custody_wire::frame::{FrameError, decode, read_frame, write_message};
use custody_wire::transport::{Connection, Duplex, Listener};
use custody_wire::{ClientMessage, Outcome, ServerMessage, limits};

use crate::consent::Mechanism;
use crate::core::{Core, Decider, Effects, Prompt, SessionId};

const OUTBOX: usize = 64;

/// A running service.
#[derive(Clone)]
pub struct Service {
    core: Arc<Mutex<Core>>,
    outboxes: Arc<Mutex<HashMap<SessionId, Outbox>>>,
    mechanism: Option<Arc<dyn Mechanism>>,
}

struct Outbox {
    send: SyncSender<ServerMessage>,
    control: Box<dyn Duplex>,
}

impl Service {
    pub fn new(core: Core, mechanism: Option<Arc<dyn Mechanism>>) -> Self {
        Self {
            core: Arc::new(Mutex::new(core)),
            outboxes: Arc::new(Mutex::new(HashMap::new())),
            mechanism,
        }
    }

    /// The core, for a platform hook that must lock from inside.
    pub fn core(&self) -> &Arc<Mutex<Core>> {
        &self.core
    }

    /// Lock from inside the custodian and tell every session.
    pub fn lock_now(&self) {
        let mut core = lock(&self.core);
        let (_, effects) = core.lock_now();
        self.deliver(&mut core, effects);
    }

    /// The SSH agent session over the vault, once it has been opened.
    pub fn agent_session(&self) -> Option<castellan::custody::agent::VaultAgent<Box<dyn castellan::custody::IdentityStorage>>> {
        lock(&self.core).custody().agent_session()
    }

    /// Accept forever. Returns only when the listener fails.
    pub fn serve(&self, listener: &dyn Listener) -> std::io::Result<()> {
        self.spawn_ticker();
        self.spawn_signing_bridge();
        loop {
            let connection = match listener.accept() {
                Ok(connection) => connection,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => return Err(error),
            };
            let service = self.clone();
            std::thread::Builder::new()
                .name("custody-session".into())
                .spawn(move || service.session(connection))?;
        }
    }

    fn spawn_ticker(&self) {
        let service = self.clone();
        std::thread::Builder::new()
            .name("custody-ticker".into())
            .spawn(move || {
                loop {
                    let wait = {
                        let mut core = lock(&service.core);
                        let effects = core.tick(Instant::now());
                        service.deliver(&mut core, effects);
                        crate::core::tick_interval(&core, Instant::now())
                    };
                    std::thread::sleep(wait);
                }
            })
            .expect("spawn the custody ticker");
    }

    /// Watch Castellan's broker for per-use SSH requests and petition the
    /// consent channel for each.
    fn spawn_signing_bridge(&self) {
        let service = self.clone();
        let _ = std::thread::Builder::new().name("custody-signing".into()).spawn(move || {
            loop {
                {
                    let mut core = lock(&service.core);
                    let effects = core.petition_signing(Instant::now());
                    service.deliver(&mut core, effects);
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }

    fn session(&self, connection: Connection) {
        let Connection { mut stream, peer } = connection;
        let Ok(control) = stream.try_clone_duplex() else {
            return;
        };
        let hello = {
            let watchdog = control.try_clone_duplex().map(|c| Watchdog::arm(c, limits::HANDSHAKE));
            let hello = read_frame(&mut *stream);
            if let Ok(watchdog) = watchdog {
                watchdog.disarm();
            }
            match hello {
                Ok(Some(body)) => decode::<ClientMessage>(&body).ok(),
                _ => None,
            }
        };
        let Some(hello) = hello else {
            stream.shutdown_both();
            return;
        };
        // The outbox exists before admission and is registered under the
        // core's lock, so no fact can be broadcast between the session's
        // admission and its first message; `Opened` goes out first.
        let (send, receive) = mpsc::sync_channel::<ServerMessage>(OUTBOX);
        let Ok(mut writer) = stream.try_clone_duplex() else {
            stream.shutdown_both();
            return;
        };
        let writer_thread = std::thread::Builder::new()
            .name("custody-writer".into())
            .spawn(move || {
                for message in receive {
                    if write_message(&mut *writer, &message).is_err() {
                        writer.shutdown_both();
                        break;
                    }
                }
            });
        let Ok(writer_thread) = writer_thread else {
            stream.shutdown_both();
            return;
        };
        let id = {
            let mut core = lock(&self.core);
            match core.open(peer, &hello) {
                Ok((id, opened)) => {
                    let _ = send.try_send(opened);
                    lock(&self.outboxes).insert(id, Outbox { send, control });
                    id
                },
                Err(refused) => {
                    drop(core);
                    let _ = send.try_send(refused);
                    drop(send);
                    let _ = writer_thread.join();
                    stream.shutdown_both();
                    return;
                },
            }
        };
        loop {
            let body = match read_frame(&mut *stream) {
                Ok(Some(body)) => body,
                Ok(None) | Err(FrameError::Io(_)) | Err(FrameError::TooLarge(_)) | Err(FrameError::Json(_)) => break,
            };
            let message = match decode::<ClientMessage>(&body) {
                Ok(message) => message,
                Err(_) => {
                    match request_id(&body) {
                        Some(request) => {
                            let mut core = lock(&self.core);
                            let generation = core.generation();
                            let effects = Effects {
                                messages: vec![(id, ServerMessage::Outcome {
                                    request,
                                    generation,
                                    outcome: Outcome::refused("invalid-request"),
                                })],
                                ..Effects::default()
                            };
                            self.deliver(&mut core, effects);
                            continue;
                        },
                        None => break,
                    }
                },
            };
            let mut core = lock(&self.core);
            let effects = core.handle(id, message, Instant::now());
            self.deliver(&mut core, effects);
        }
        stream.shutdown_both();
        let mut core = lock(&self.core);
        self.drop_outbox(id);
        let effects = core.close(id);
        self.deliver(&mut core, effects);
    }

    fn drop_outbox(&self, id: SessionId) {
        if let Some(outbox) = lock(&self.outboxes).remove(&id) {
            outbox.control.shutdown_both();
        }
    }

    /// Deliver effects while the core is held, so facts and outcomes keep
    /// the order of the state changes that produced them.
    fn deliver(&self, core: &mut Core, effects: Effects) {
        let mut pending = vec![effects];
        while let Some(effects) = pending.pop() {
            let mut closing = effects.close;
            {
                let outboxes = lock(&self.outboxes);
                for (id, message) in effects.messages {
                    if let Some(outbox) = outboxes.get(&id) {
                        match outbox.send.try_send(message) {
                            Ok(()) => {},
                            Err(TrySendError::Full(_)) | Err(TrySendError::Disconnected(_)) => closing.push(id),
                        }
                    }
                }
            }
            for prompt in effects.prompts {
                if let Some(more) = self.present(core, prompt) {
                    pending.push(more);
                }
            }
            for id in closing {
                // The writer drains what was queued before the shutdown lands.
                if let Some(outbox) = lock(&self.outboxes).remove(&id) {
                    drop(outbox.send);
                    let control = outbox.control;
                    std::thread::spawn(move || {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        control.shutdown_both();
                    });
                }
                pending.push(core.close(id));
            }
        }
    }

    /// Run the consent mechanism for one petition on its own thread. With
    /// no mechanism the petition is refused at once.
    fn present(&self, core: &mut Core, prompt: Prompt) -> Option<Effects> {
        let Some(mechanism) = self.mechanism.clone() else {
            let petition = &prompt.petition;
            let (_, effects) = core.decide(Decider::Mechanism, petition.request, &petition.digest, petition.generation, false, Instant::now());
            return Some(effects);
        };
        let service = self.clone();
        std::thread::spawn(move || {
            let accept = mechanism.present(&prompt);
            let petition = &prompt.petition;
            let mut core = lock(&service.core);
            let (_, effects) = core.decide(Decider::Mechanism, petition.request, &petition.digest, petition.generation, accept, Instant::now());
            service.deliver(&mut core, effects);
        });
        None
    }
}

/// The id of a request that failed typed decoding, so it can be refused.
/// Only two fields are read; every other value is skipped uncopied.
fn request_id(body: &[u8]) -> Option<u64> {
    #[derive(serde::Deserialize)]
    struct Probe {
        #[serde(rename = "type")]
        kind: String,
        id: Option<u64>,
    }
    let probe: Probe = serde_json::from_slice(body).ok()?;
    (probe.kind == "request").then_some(probe.id).flatten()
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}
