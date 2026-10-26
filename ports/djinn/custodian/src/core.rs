// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custodian's state machine, without I/O. Every transport and launcher
//! drives this one implementation of §5.3 to §5.6: the socket service on
//! Linux, macOS and Windows, and the signed macOS host through its C ABI.
//!
//! A session is opened for an OS peer the transport established; the
//! policy, not the hello, selects its principal. Each request is checked
//! for instance, generation, grant, profile, namespace and lock before it
//! runs or becomes a consent petition. A petition carries a digest the
//! custodian derived and is decided only by the trusted consent channel for
//! that digest, request and generation; the operation is rechecked when the
//! decision arrives. Lock, unlock and revocation advance one monotonic
//! generation, cancel every pending petition and broadcast a fact.

use std::collections::{BTreeSet, HashMap, HashSet, VecDeque};
use std::time::{Duration, Instant};

use custody_wire::contract::{Answer, Fact, FactCause, Limits, LockReport, Outcome, Petition, Purpose, ReleasedKey, ServerMessage, Session, Status};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::{ClientMessage, Operation, Peer, limits};

use crate::Refusal;
use crate::policy::{Config, Consent, ConsentMode, Grant, Principal, PurposeRule};
use crate::vault::VaultCustody;

/// A session handle, unique per custodian start.
pub type SessionId = u64;

/// What the driver must do after a call.
#[derive(Default)]
pub struct Effects {
    /// Messages to deliver, in order.
    pub messages: Vec<(SessionId, ServerMessage)>,
    /// Petitions for a custodian-run consent mechanism (polkit, or the
    /// macOS host's own dialog). Decide them with [`Decider::Mechanism`].
    pub prompts: Vec<Prompt>,
    /// Sessions to close after delivery.
    pub close: Vec<SessionId>,
}

/// A petition for a custodian-run consent mechanism.
#[derive(Clone, Debug)]
pub struct Prompt {
    pub petition: Petition,
    /// The OS peer that asked, for the mechanism's own subject.
    pub requester: Peer,
}

/// Who is deciding.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decider {
    /// A session; honoured only for a consent-channel principal.
    Session(SessionId),
    /// The mechanism the custodian itself ran for a [`Prompt`].
    Mechanism,
}

struct SessionState {
    principal: Principal,
    peer: Peer,
}

enum Deferred {
    Sign { salt: Vec<u8>, message: Vec<u8> },
    Release { salts: Vec<Vec<u8>> },
    Revoke { principal: String },
}

struct Pending {
    session: SessionId,
    request: u64,
    digest: String,
    generation: u64,
    deadline: Instant,
    principal: String,
    grant: usize,
    consent: Consent,
    requester: Peer,
    summary: Petition,
    deferred: Deferred,
}

/// The custodian.
pub struct Core {
    config: Config,
    vault: VaultCustody,
    instance: String,
    generation: u64,
    sessions: HashMap<SessionId, SessionState>,
    next_session: SessionId,
    revoked: BTreeSet<String>,
    consented: HashSet<(String, usize, u64)>,
    queue: VecDeque<Pending>,
    active: Option<String>,
}

const MAX_SALTS: usize = 32;
const MAX_SIGNED_BYTES: usize = 64 * 1024;
const MAX_PETITIONS_PER_SESSION: usize = 4;

impl Core {
    /// Start locked, with a fresh instance id.
    pub fn start(config: Config) -> Result<Self, Refusal> {
        let vault = VaultCustody::locked(&config.vault.directory, config.vault.profile.as_deref())?;
        Ok(Self {
            config,
            vault,
            instance: crate::nonce()?,
            generation: 1,
            sessions: HashMap::new(),
            next_session: 1,
            revoked: BTreeSet::new(),
            consented: HashSet::new(),
            queue: VecDeque::new(),
            active: None,
        })
    }

    pub fn instance(&self) -> &str {
        &self.instance
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn consent_mode(&self) -> ConsentMode {
        self.config.consent
    }

    /// Public status.
    pub fn status(&self) -> Status {
        let mut protection = self.vault.protection();
        if self.config.development {
            protection.push_str("; development mode: same-user principals may hold unattended grants");
        }
        Status {
            instance: self.instance.clone(),
            generation: self.generation,
            locked: self.vault.is_locked(),
            profile: self.vault.profile().map(str::to_string),
            master_public_key: self.vault.master_public_key(),
            protection,
        }
    }

    /// Admit a session for `peer`. The first message must be a hello.
    pub fn open(&mut self, peer: Peer, hello: &ClientMessage) -> Result<(SessionId, ServerMessage), ServerMessage> {
        let refuse = |code: &str| ServerMessage::Refused { code: code.into() };
        let ClientMessage::Hello { version, nonce } = hello else {
            return Err(refuse("hello-required"));
        };
        if *version != custody_wire::VERSION {
            return Err(refuse("version"));
        }
        if nonce.len() > 128 {
            return Err(refuse("invalid-hello"));
        }
        let principal = self.config.principal(&peer).ok_or_else(|| refuse("principal-unknown"))?;
        if self.revoked.contains(&principal.name) {
            return Err(refuse("revoked"));
        }
        if self.sessions.len() >= limits::MAX_CONNECTIONS {
            return Err(refuse("connection-capacity"));
        }
        let account = peer.account();
        if self.sessions.values().filter(|s| s.peer.account() == account).count() >= limits::MAX_CONNECTIONS_PER_PRINCIPAL {
            return Err(refuse("principal-connection-capacity"));
        }
        let id = self.next_session;
        self.next_session += 1;
        let session = Session {
            version: custody_wire::VERSION,
            instance: self.instance.clone(),
            generation: self.generation,
            locked: self.vault.is_locked(),
            principal: principal.name.clone(),
            peer: peer.clone(),
            scopes: principal.scopes(),
            consent_channel: principal.consent_channel,
            limits: Limits {
                max_frame_bytes: limits::MAX_FRAME_BYTES,
                max_queued_operations: limits::MAX_QUEUED_OPERATIONS as u32,
                noninteractive_ms: limits::NONINTERACTIVE.as_millis() as u64,
                consent_ms: limits::CONSENT.as_millis() as u64,
            },
            nonce: nonce.clone(),
        };
        self.sessions.insert(id, SessionState { principal, peer });
        Ok((id, ServerMessage::Opened(session)))
    }

    /// A session ended. Its pending petitions are withdrawn.
    pub fn close(&mut self, session: SessionId) -> Effects {
        self.sessions.remove(&session);
        let mut effects = Effects::default();
        let before = self.queue.len();
        self.queue.retain(|p| p.session != session);
        if self.queue.len() != before {
            if self.active.as_ref().is_some_and(|d| !self.queue.iter().any(|p| &p.digest == d)) {
                self.active = None;
            }
            self.promote(&mut effects);
        }
        effects
    }

    /// Handle one message from an open session.
    pub fn handle(&mut self, session: SessionId, message: ClientMessage, now: Instant) -> Effects {
        let mut effects = self.expire(now);
        match message {
            ClientMessage::Hello { .. } => {
                effects.close.push(session);
            },
            ClientMessage::Request {
                id,
                instance,
                generation,
                operation,
            } => {
                let outcome = self.request(session, id, &instance, generation, operation, now, &mut effects);
                if let Some(outcome) = outcome {
                    effects.messages.push((session, ServerMessage::Outcome {
                        request: id,
                        generation: self.generation,
                        outcome,
                    }));
                }
            },
            ClientMessage::Decide {
                request,
                digest,
                generation,
                accept,
            } => {
                let recorded = self.decide_into(Decider::Session(session), request, &digest, generation, accept, now, &mut effects);
                effects.messages.push((session, ServerMessage::Decided {
                    request,
                    recorded: recorded.is_ok(),
                    code: recorded.err().map(|r| r.0.to_string()),
                }));
            },
        }
        effects
    }

    /// A decision from the custodian's own consent mechanism.
    pub fn decide(&mut self, decider: Decider, request: u64, digest: &str, generation: u64, accept: bool, now: Instant) -> (Result<(), Refusal>, Effects) {
        let mut effects = self.expire(now);
        let recorded = self.decide_into(decider, request, digest, generation, accept, now, &mut effects);
        (recorded, effects)
    }

    /// Expire petitions past their deadline.
    pub fn tick(&mut self, now: Instant) -> Effects {
        self.expire(now)
    }

    /// Lock from inside the custodian (a service stop or a platform event).
    pub fn lock_now(&mut self) -> (LockReport, Effects) {
        let mut effects = Effects::default();
        let report = self.lock(&mut effects);
        (report, effects)
    }

    #[allow(clippy::too_many_arguments)]
    fn request(&mut self, session: SessionId, id: u64, instance: &str, generation: u64, operation: Operation, now: Instant, effects: &mut Effects) -> Option<Outcome> {
        let Some(state) = self.sessions.get(&session) else {
            return Some(Outcome::refused("session-closed"));
        };
        let principal = state.principal.clone();
        let requester = state.peer.clone();
        if self.revoked.contains(&principal.name) {
            effects.close.push(session);
            return Some(Outcome::refused("revoked"));
        }
        if instance != self.instance {
            return Some(Outcome::refused("stale-instance"));
        }
        let generation_free = matches!(operation, Operation::Status {} | Operation::Lock {});
        if !generation_free && generation != self.generation {
            return Some(Outcome::refused("stale-generation"));
        }
        match operation {
            Operation::Status {} => Some(self.granted(&principal, |g| matches!(g, Grant::Status)).map_or_else(
                |r| Outcome::refused(r.0),
                |_| Outcome::Success { answer: Answer::Status { status: self.status() } },
            )),
            Operation::Lock {} => {
                if let Err(refusal) = self.granted(&principal, |g| matches!(g, Grant::Lock)) {
                    return Some(Outcome::refused(refusal.0));
                }
                let report = self.lock(effects);
                Some(lock_outcome(report))
            },
            Operation::Unlock { passphrase } => Some(self.unlock(&principal, passphrase, effects)),
            Operation::Revoke { principal: target } => {
                let (index, consent) = match self.granted(&principal, |g| matches!(g, Grant::Revoke { .. })) {
                    Ok((index, Grant::Revoke { consent })) => (index, consent),
                    Ok(_) => unreachable!("matched a revoke grant"),
                    Err(refusal) => return Some(Outcome::refused(refusal.0)),
                };
                if !self.config.principals.iter().any(|p| p.name == target) {
                    return Some(Outcome::refused("principal-unknown"));
                }
                let summary = ("revoke", String::new(), target.clone(), blake3::hash(target.as_bytes()).to_hex().to_string());
                self.gate(session, id, &principal, requester, index, consent, summary, Deferred::Revoke { principal: target }, now, effects)
            },
            Operation::Sign {
                profile,
                purpose,
                salt,
                message,
            } => {
                let current = self.vault.profile().unwrap_or_default().to_string();
                if self.vault.is_locked() {
                    return Some(Outcome::refused("locked"));
                }
                if message.0.len() > MAX_SIGNED_BYTES {
                    return Some(Outcome::refused("message-too-large"));
                }
                let found = self.config_grants(&principal).find_map(|(index, grant)| match (grant, &purpose) {
                    (
                        Grant::Sign {
                            profile: scope,
                            purpose: PurposeRule::Delegation { audience: granted },
                            consent,
                        },
                        Purpose::Delegation { audience },
                    ) if granted == audience && scope.covers(&profile, &current) => Some((index, *consent)),
                    (
                        Grant::Sign {
                            profile: scope,
                            purpose: PurposeRule::Root,
                            consent,
                        },
                        Purpose::Root {},
                    ) if scope.covers(&profile, &current) => Some((index, *consent)),
                    _ => None,
                });
                let Some((index, consent)) = found else {
                    return Some(Outcome::refused("operation-or-scope"));
                };
                let master = self.vault.master_public_key().unwrap_or_default();
                let checked = match &purpose {
                    Purpose::Delegation { audience } => crate::delegation::check(audience, &master, &salt.0, &message.0),
                    Purpose::Root {} if salt.0.is_empty() => Ok(()),
                    Purpose::Root {} => Err(Refusal("root-signature-takes-no-salt")),
                };
                if let Err(refusal) = checked {
                    return Some(Outcome::refused(refusal.0));
                }
                let namespace = match &purpose {
                    Purpose::Delegation { audience } => format!("delegation:{audience}"),
                    Purpose::Root {} => "root".into(),
                };
                let summary = ("sign", profile, namespace, blake3::hash(&message.0).to_hex().to_string());
                self.gate(session, id, &principal, requester, index, consent, summary, Deferred::Sign { salt: salt.0, message: message.0 }, now, effects)
            },
            Operation::Release { profile, salts } => {
                let current = self.vault.profile().unwrap_or_default().to_string();
                if self.vault.is_locked() {
                    return Some(Outcome::refused("locked"));
                }
                if salts.is_empty() || salts.len() > MAX_SALTS {
                    return Some(Outcome::refused("operation-or-scope"));
                }
                let salts: Vec<Vec<u8>> = salts.into_iter().map(|b| b.0).collect();
                let found = self.config_grants(&principal).find_map(|(index, grant)| match grant {
                    Grant::Release {
                        profile: scope,
                        salts: rules,
                        consent,
                    } if scope.covers(&profile, &current) && salts.iter().all(|s| rules.iter().any(|r| r.allows(s))) => Some((index, *consent)),
                    _ => None,
                });
                let Some((index, consent)) = found else {
                    return Some(Outcome::refused("operation-or-scope"));
                };
                let mut digest = blake3::Hasher::new();
                for salt in &salts {
                    digest.update(&(salt.len() as u64).to_le_bytes());
                    digest.update(salt);
                }
                let summary = ("release", profile, "derived-keys".to_string(), digest.finalize().to_hex().to_string());
                self.gate(session, id, &principal, requester, index, consent, summary, Deferred::Release { salts }, now, effects)
            },
        }
    }

    fn config_grants<'a>(&self, principal: &'a Principal) -> impl Iterator<Item = (usize, &'a Grant)> {
        principal.grants.iter().enumerate()
    }

    fn granted(&self, principal: &Principal, wanted: impl Fn(&Grant) -> bool) -> Result<(usize, Grant), Refusal> {
        principal
            .grants
            .iter()
            .enumerate()
            .find(|(_, g)| wanted(g))
            .map(|(i, g)| (i, g.clone()))
            .ok_or(Refusal("operation-denied"))
    }

    /// Run now, or queue a petition, according to the grant's consent.
    #[allow(clippy::too_many_arguments)]
    fn gate(&mut self, session: SessionId, request: u64, principal: &Principal, requester: Peer, grant: usize, consent: Consent, summary: (&str, String, String, String), deferred: Deferred, now: Instant, effects: &mut Effects) -> Option<Outcome> {
        let approved = match consent {
            Consent::None => true,
            Consent::PerGeneration => self.consented.contains(&(principal.name.clone(), grant, self.generation)),
            Consent::PerOperation => false,
        };
        if approved {
            return Some(self.execute(deferred, effects));
        }
        if self.config.consent == ConsentMode::Deny {
            return Some(Outcome::refused("consent-unavailable"));
        }
        if self.config.consent == ConsentMode::Channel && !self.sessions.values().any(|s| s.principal.consent_channel) {
            return Some(Outcome::refused("consent-unavailable"));
        }
        if self.queue.len() >= limits::MAX_QUEUED_OPERATIONS || self.queue.iter().filter(|p| p.session == session).count() >= MAX_PETITIONS_PER_SESSION {
            return Some(Outcome::refused("consent-busy"));
        }
        let (operation, profile, namespace, payload_digest) = summary;
        let mut hasher = blake3::Hasher::new();
        for part in [self.instance.as_bytes(), &self.generation.to_le_bytes(), principal.name.as_bytes(), &request.to_le_bytes(), operation.as_bytes(), profile.as_bytes(), namespace.as_bytes(), payload_digest.as_bytes()] {
            hasher.update(&(part.len() as u64).to_le_bytes());
            hasher.update(part);
        }
        let digest = hasher.finalize().to_hex().to_string();
        let petition = Petition {
            request,
            digest: digest.clone(),
            generation: self.generation,
            principal: principal.name.clone(),
            operation: operation.to_string(),
            profile,
            namespace,
            payload_digest,
            expires_in_ms: limits::CONSENT.as_millis() as u64,
        };
        self.queue.push_back(Pending {
            session,
            request,
            digest,
            generation: self.generation,
            deadline: now + limits::CONSENT,
            principal: principal.name.clone(),
            grant,
            consent,
            requester,
            summary: petition,
            deferred,
        });
        self.promote(effects);
        None
    }

    /// Show the head of the queue if nothing is active.
    fn promote(&mut self, effects: &mut Effects) {
        if self.active.is_some() {
            return;
        }
        let Some(head) = self.queue.front() else {
            return;
        };
        self.active = Some(head.digest.clone());
        match self.config.consent {
            ConsentMode::Channel => {
                for (id, state) in &self.sessions {
                    if state.principal.consent_channel {
                        effects.messages.push((*id, ServerMessage::Petition(head.summary.clone())));
                    }
                }
            },
            ConsentMode::Polkit => effects.prompts.push(Prompt {
                petition: head.summary.clone(),
                requester: head.requester.clone(),
            }),
            ConsentMode::Deny => {},
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn decide_into(&mut self, decider: Decider, request: u64, digest: &str, generation: u64, accept: bool, _now: Instant, effects: &mut Effects) -> Result<(), Refusal> {
        let trusted = match decider {
            Decider::Session(id) => self.config.consent == ConsentMode::Channel && self.sessions.get(&id).is_some_and(|s| s.principal.consent_channel),
            Decider::Mechanism => self.config.consent == ConsentMode::Polkit,
        };
        if !trusted {
            return Err(Refusal("not-consent-channel"));
        }
        if self.active.as_deref() != Some(digest) {
            return Err(Refusal("petition-not-active"));
        }
        let position = self.queue.iter().position(|p| p.digest == digest).ok_or(Refusal("petition-not-pending"))?;
        let head = &self.queue[position];
        if head.request != request || head.generation != generation || generation != self.generation {
            return Err(Refusal("petition-binding"));
        }
        let pending = self.queue.remove(position).expect("position is in range");
        self.active = None;
        let outcome = if !accept {
            Outcome::refused("consent-denied")
        } else if self.revoked.contains(&pending.principal) || !self.sessions.contains_key(&pending.session) {
            Outcome::refused("invalidated")
        } else {
            if pending.consent == Consent::PerGeneration {
                self.consented.insert((pending.principal.clone(), pending.grant, self.generation));
            }
            self.execute(pending.deferred, effects)
        };
        effects.messages.push((pending.session, ServerMessage::Outcome {
            request: pending.request,
            generation: self.generation,
            outcome,
        }));
        self.promote(effects);
        Ok(())
    }

    fn execute(&mut self, deferred: Deferred, effects: &mut Effects) -> Outcome {
        match deferred {
            Deferred::Sign { salt, message } => match self.vault.sign(&salt, &message) {
                Ok(signed) => Outcome::Success {
                    answer: Answer::Signature {
                        public_key: signed.public_key,
                        signature: signed.signature,
                        attestation: signed.attestation,
                    },
                },
                Err(refusal) => Outcome::refused(refusal.0),
            },
            Deferred::Release { salts } => match self.vault.release(&salts) {
                Ok((master, released)) => Outcome::Success {
                    answer: Answer::Released {
                        master_public_key: master,
                        keys: released
                            .into_iter()
                            .map(|r| ReleasedKey {
                                salt: Bytes(r.salt),
                                seed: Secret::copy_from(&*r.seed),
                                attestation: r.attestation,
                            })
                            .collect(),
                    },
                },
                Err(refusal) => Outcome::refused(refusal.0),
            },
            Deferred::Revoke { principal } => {
                self.revoked.insert(principal.clone());
                self.advance();
                self.cancel(|p| p.principal == principal, effects);
                let doomed: Vec<SessionId> = self.sessions.iter().filter(|(_, s)| s.principal.name == principal).map(|(id, _)| *id).collect();
                self.broadcast(FactCause::Revoke, None, Some(principal), effects);
                effects.close.extend(doomed);
                Outcome::Success {
                    answer: Answer::Revoked { generation: self.generation },
                }
            },
        }
    }

    fn unlock(&mut self, principal: &Principal, passphrase: Secret, effects: &mut Effects) -> Outcome {
        if let Err(refusal) = self.granted(principal, |g| matches!(g, Grant::Unlock)) {
            return Outcome::refused(refusal.0);
        }
        if let Err(refusal) = self.vault.unlock(passphrase.expose()) {
            return Outcome::refused(refusal.0);
        }
        drop(passphrase);
        self.advance();
        self.broadcast(FactCause::Unlock, None, None, effects);
        Outcome::Success {
            answer: Answer::Unlocked { generation: self.generation },
        }
    }

    /// Close admission, drop the root and every grant, persist the barrier.
    fn lock(&mut self, effects: &mut Effects) -> LockReport {
        let was_unlocked = !self.vault.is_locked();
        let durable = self.vault.lock();
        self.advance();
        self.cancel(|_| true, effects);
        let report = LockReport {
            generation: self.generation,
            memory_locked: true,
            // Operations run to completion under the custodian's lock before
            // this one starts; petitions were cancelled above.
            drained: true,
            durable,
        };
        let _ = was_unlocked;
        self.broadcast(FactCause::Lock, Some(report.clone()), None, effects);
        report
    }

    fn advance(&mut self) {
        self.generation = self.generation.saturating_add(1);
        self.consented.clear();
    }

    fn cancel(&mut self, which: impl Fn(&Pending) -> bool, effects: &mut Effects) {
        let mut kept = VecDeque::new();
        for pending in self.queue.drain(..) {
            if which(&pending) {
                if self.active.as_deref() == Some(pending.digest.as_str()) {
                    self.active = None;
                }
                effects.messages.push((pending.session, ServerMessage::Outcome {
                    request: pending.request,
                    generation: self.generation,
                    outcome: Outcome::refused("invalidated"),
                }));
            } else {
                kept.push_back(pending);
            }
        }
        self.queue = kept;
        if let Some(active) = &self.active
            && !self.queue.iter().any(|p| &p.digest == active)
        {
            self.active = None;
        }
        self.promote(effects);
    }

    fn broadcast(&mut self, cause: FactCause, report: Option<LockReport>, revoked: Option<String>, effects: &mut Effects) {
        let fact = Fact {
            instance: self.instance.clone(),
            generation: self.generation,
            cause,
            locked: self.vault.is_locked(),
            report,
            revoked,
        };
        for id in self.sessions.keys() {
            effects.messages.push((*id, ServerMessage::Fact(fact.clone())));
        }
    }

    fn expire(&mut self, now: Instant) -> Effects {
        let mut effects = Effects::default();
        let mut kept = VecDeque::new();
        for pending in self.queue.drain(..) {
            if pending.deadline <= now {
                if self.active.as_deref() == Some(pending.digest.as_str()) {
                    self.active = None;
                }
                effects.messages.push((pending.session, ServerMessage::Outcome {
                    request: pending.request,
                    generation: self.generation,
                    outcome: Outcome::refused("consent-expired"),
                }));
            } else {
                kept.push_back(pending);
            }
        }
        self.queue = kept;
        self.promote(&mut effects);
        effects
    }

    /// The next petition deadline, for a driver's timer.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.queue.iter().map(|p| p.deadline).min()
    }
}

fn lock_outcome(report: LockReport) -> Outcome {
    if report.durable {
        Outcome::Success {
            answer: Answer::Lock { report },
        }
    } else {
        Outcome::Degraded {
            code: "restart-barrier-not-durable".into(),
            answer: Some(Answer::Lock { report }),
        }
    }
}

/// How long a driver may sleep before the next expiry check.
pub fn tick_interval(core: &Core, now: Instant) -> Duration {
    core.next_deadline()
        .map(|d| d.saturating_duration_since(now))
        .unwrap_or(Duration::from_secs(60))
        .clamp(Duration::from_millis(50), Duration::from_secs(60))
}
