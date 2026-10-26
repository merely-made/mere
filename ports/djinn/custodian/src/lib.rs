// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Custody core for the independently launched, signed native service.
//!
//! Uses Castellan's real sealed-profile format and IdentityVault. No keys are
//! returned to IPC clients. The native host authenticates peers and obtains
//! the storage key from its own OS group. This first integration accepts only
//! an explicitly marked synthetic vault; no product consumer is wired to it.
//! Native confirmation is a separate in-process call, never a client operation.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use castellan::custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage};
use personae::{Ed25519Keypair, IdentityProvider, ProfileId};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::HashMap,
    io::Read,
    path::PathBuf,
    time::{Duration, Instant},
};
use zeroize::Zeroizing;

#[cfg(feature = "native-host")]
mod native;

pub const MAX_REQUEST_BYTES: usize = 16 * 1024;
pub const MAX_CHALLENGE_BYTES: usize = 4096;
const ROOT_NAMESPACE: &str = "h1.root-sign.v1";
const SESSION_NAMESPACE: &str = "h1.continuous-sign.v1";
const MAX_PENDING: usize = 8;
const MAX_SESSIONS: usize = 8;
const MAX_SESSION_USES: u16 = 8;
const CONFIRMATION_TTL: Duration = Duration::from_secs(120);
const SESSION_TTL: Duration = Duration::from_secs(60);

/// Immutable native-host configuration, read only after its signed resources
/// and protected installation location have been validated.
#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyntheticConfig {
    pub run_id: String,
    pub directory: PathBuf,
    pub profile: String,
    pub principal: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal(pub &'static str);
pub type Result<T> = std::result::Result<T, Refusal>;

fn storage(_: personae::IdentityError) -> Refusal {
    Refusal("vault-storage")
}
fn nonce() -> Result<String> {
    let mut bytes = [0; 32];
    getrandom::fill(&mut bytes).map_err(|_| Refusal("randomness-unavailable"))?;
    Ok(blake3::hash(&bytes).to_hex().to_string())
}

/// All client operations. There is deliberately no approve/decision/unlock-
/// credential operation. An extra approved/confirmed field fails decoding.
#[derive(Deserialize)]
#[serde(tag = "op", deny_unknown_fields)]
enum Request {
    #[serde(rename = "status")]
    Status,
    #[serde(rename = "sign")]
    Sign {
        namespace: String,
        challenge: String,
        instance: String,
        #[serde(default)]
        lane: Option<String>,
    },
    #[serde(rename = "open-session")]
    OpenSession {
        namespace: String,
        profile: String,
        max_uses: u16,
        instance: String,
    },
    #[serde(rename = "session-sign")]
    SessionSign {
        capability: String,
        namespace: String,
        profile: String,
        challenge: String,
        generation: u64,
        instance: String,
    },
    #[serde(rename = "revoke-session")]
    RevokeSession {
        capability: String,
        instance: String,
    },
    #[serde(rename = "lock")]
    Lock { instance: String },
}

enum Action {
    RootSign(String),
    OpenSession(u16),
}
struct Pending {
    action: Action,
    generation: u64,
    principal: String,
    deadline: Instant,
}
struct Session {
    generation: u64,
    principal: String,
    remaining: u16,
    deadline: Instant,
}

/// Public-only facts shown by the native owner. Tickets cannot be submitted
/// through dispatch; only the native host can complete a confirmation.
#[derive(Serialize)]
pub struct Confirmation {
    pub ticket: String,
    pub principal: String,
    pub profile: String,
    pub operation: &'static str,
    pub payload_digest: String,
    pub generation: u64,
}

pub enum Dispatch {
    Reply(Value),
    Confirm(Confirmation),
}

pub struct Custodian {
    config: SyntheticConfig,
    vault: Option<IdentityVault<SealedProfileStorage>>,
    public_key: Option<[u8; 32]>,
    instance: String,
    generation: u64,
    pending: HashMap<String, Pending>,
    sessions: HashMap<String, Session>,
}

impl Custodian {
    /// Reopen only. Missing/corrupt records never regenerate an identity.
    /// Every existing vault starts locked before any profile is decrypted.
    /// A same-user app removing the advisory marker cannot authorize unlock.
    pub fn open_synthetic(config: SyntheticConfig, key: Zeroizing<[u8; 32]>) -> Result<Self> {
        drop(key);
        Self::open(config, Zeroizing::new([0; 32]), false)
    }

    /// First-run native composition only: the OS key provider must have just
    /// created this run's unique item. Never call this for an existing OS key.
    pub fn create_synthetic(config: SyntheticConfig, key: Zeroizing<[u8; 32]>) -> Result<Self> {
        if config.directory.join("profiles").exists() {
            return Err(Refusal("synthetic-vault-already-exists"));
        }
        Self::open(config, key, true)
    }

    fn open(config: SyntheticConfig, key: Zeroizing<[u8; 32]>, initialize: bool) -> Result<Self> {
        validate_config(&config)?;
        let mut this = Self {
            config,
            vault: None,
            public_key: None,
            instance: nonce()?,
            generation: 1,
            pending: HashMap::new(),
            sessions: HashMap::new(),
        };
        if initialize && !castellan::custody::lock_persisted(&this.config.directory) {
            this.load_key(&key, initialize)?;
        }
        Ok(this)
    }

    fn load_key(&mut self, key: &[u8; 32], initialize: bool) -> Result<()> {
        let storage = SealedProfileStorage::open_with_key(&self.config.directory, *key);
        let profiles = storage.list_profiles().map_err(storage_error)?;
        let id = ProfileId(self.config.profile.clone());
        if profiles.is_empty() && initialize {
            storage
                .save_profile(&Profile::new(
                    id.clone(),
                    "Synthetic custody receiving",
                    Ed25519Keypair::generate(),
                ))
                .map_err(storage_error)?;
        } else if !profiles.iter().any(|p| p.id == id) {
            return Err(Refusal("synthetic-profile-missing"));
        }
        let vault = IdentityVault::open(storage, &id).map_err(storage_error)?;
        let public = vault
            .current_profile()
            .map_err(storage_error)?
            .master
            .public_key()
            .to_bytes();
        if self.public_key.is_some_and(|p| p != public) {
            return Err(Refusal("vault-identity-changed"));
        }
        self.public_key = Some(public);
        self.vault = Some(vault);
        Ok(())
    }

    pub fn status(&self) -> Value {
        json!({"ok":true,"backend":"castellan-sealed-profile-v1","syntheticOnly":true,
            "realConsumersAllowed":false,"locked":self.vault.is_none(),"instance":self.instance,
            "generation":self.generation,"profile":self.config.profile,
            "publicKey":self.public_key.map(|p| STANDARD.encode(p))})
    }

    fn admitted(&self, principal: &str, instance: &str) -> Result<()> {
        if principal != self.config.principal {
            return Err(Refusal("principal-denied"));
        }
        if instance != self.instance {
            return Err(Refusal("stale-instance"));
        }
        if self.vault.is_none() {
            return Err(Refusal("locked"));
        }
        Ok(())
    }

    pub fn dispatch(&mut self, principal: &str, bytes: &[u8]) -> Result<Dispatch> {
        if bytes.len() > MAX_REQUEST_BYTES {
            return Err(Refusal("request-too-large"));
        }
        if principal != self.config.principal {
            return Err(Refusal("principal-denied"));
        }
        let request: Request =
            serde_json::from_slice(bytes).map_err(|_| Refusal("invalid-request"))?;
        self.pending.retain(|_, p| p.deadline > Instant::now());
        self.sessions
            .retain(|_, s| s.deadline > Instant::now() && s.remaining > 0);
        match request {
            Request::Status => Ok(Dispatch::Reply(self.status())),
            Request::Sign {
                namespace,
                challenge,
                instance,
                lane,
            } => {
                self.admitted(principal, &instance)?;
                if namespace != ROOT_NAMESPACE
                    || challenge.len() > MAX_CHALLENGE_BYTES
                    || lane.as_ref().is_some_and(|s| s.len() > 64)
                {
                    return Err(Refusal("operation-or-scope"));
                }
                self.petition(principal, Action::RootSign(challenge))
            },
            Request::OpenSession {
                namespace,
                profile,
                max_uses,
                instance,
            } => {
                self.admitted(principal, &instance)?;
                if namespace != SESSION_NAMESPACE
                    || profile != self.config.profile
                    || max_uses == 0
                    || max_uses > MAX_SESSION_USES
                {
                    return Err(Refusal("operation-or-scope"));
                }
                if self.sessions.len() >= MAX_SESSIONS {
                    return Err(Refusal("session-capacity"));
                }
                self.petition(principal, Action::OpenSession(max_uses))
            },
            Request::SessionSign {
                capability,
                namespace,
                profile,
                challenge,
                generation,
                instance,
            } => {
                self.admitted(principal, &instance)?;
                if namespace != SESSION_NAMESPACE
                    || profile != self.config.profile
                    || challenge.len() > MAX_CHALLENGE_BYTES
                {
                    return Err(Refusal("operation-or-scope"));
                }
                let grant = self
                    .sessions
                    .get_mut(&capability)
                    .ok_or(Refusal("capability-denied"))?;
                if grant.principal != principal
                    || grant.generation != self.generation
                    || generation != self.generation
                    || grant.remaining == 0
                    || grant.deadline <= Instant::now()
                {
                    return Err(Refusal("capability-denied"));
                }
                grant.remaining -= 1;
                let vault = self.vault.as_ref().ok_or(Refusal("locked"))?;
                let key = vault
                    .derived_keypair(SESSION_NAMESPACE.as_bytes())
                    .map_err(storage_error)?;
                Ok(Dispatch::Reply(
                    json!({"ok":true,"publicKey":STANDARD.encode(key.public_key().to_bytes()),
                    "signature":STANDARD.encode(key.sign(challenge.as_bytes()).to_bytes()),"generation":self.generation}),
                ))
            },
            Request::RevokeSession {
                capability,
                instance,
            } => {
                self.admitted(principal, &instance)?;
                self.sessions
                    .remove(&capability)
                    .ok_or(Refusal("capability-denied"))?;
                Ok(Dispatch::Reply(json!({"ok":true})))
            },
            Request::Lock { instance } => {
                // Repeating lock retries a failed persisted barrier.
                if instance != self.instance {
                    return Err(Refusal("stale-instance"));
                }
                self.lock()?;
                Ok(Dispatch::Reply(self.status()))
            },
        }
    }

    fn petition(&mut self, principal: &str, action: Action) -> Result<Dispatch> {
        if self.pending.len() >= MAX_PENDING {
            return Err(Refusal("confirmation-capacity"));
        }
        let (operation, payload_digest) = match &action {
            Action::RootSign(bytes) => (
                "root-sign-once",
                blake3::hash(bytes.as_bytes()).to_hex().to_string(),
            ),
            Action::OpenSession(uses) => (
                "open-scoped-session",
                format!("{SESSION_NAMESPACE}; max uses {uses}; 60 seconds"),
            ),
        };
        let ticket = nonce()?;
        self.pending.insert(
            ticket.clone(),
            Pending {
                action,
                generation: self.generation,
                principal: principal.into(),
                deadline: Instant::now() + CONFIRMATION_TTL,
            },
        );
        Ok(Dispatch::Confirm(Confirmation {
            ticket,
            principal: principal.into(),
            profile: self.config.profile.clone(),
            operation,
            payload_digest,
            generation: self.generation,
        }))
    }

    /// Native host only. There is no serialized client route to this method.
    pub fn complete_confirmation(&mut self, ticket: &str, accepted: bool) -> Result<Value> {
        let pending = self
            .pending
            .remove(ticket)
            .ok_or(Refusal("confirmation-not-pending"))?;
        if pending.generation != self.generation
            || pending.deadline <= Instant::now()
            || self.vault.is_none()
        {
            return Err(Refusal("stale-confirmation"));
        }
        if !accepted {
            return Err(Refusal("confirmation-denied"));
        }
        match pending.action {
            Action::RootSign(challenge) => {
                let profile = self
                    .vault
                    .as_ref()
                    .ok_or(Refusal("locked"))?
                    .current_profile()
                    .map_err(storage_error)?;
                Ok(
                    json!({"ok":true,"publicKey":STANDARD.encode(profile.master.public_key().to_bytes()),
                    "signature":STANDARD.encode(profile.master.sign(challenge.as_bytes()).to_bytes()),"instance":self.instance}),
                )
            },
            Action::OpenSession(uses) => {
                if self.sessions.len() >= MAX_SESSIONS {
                    return Err(Refusal("session-capacity"));
                }
                let capability = nonce()?;
                self.sessions.insert(
                    capability.clone(),
                    Session {
                        generation: self.generation,
                        principal: pending.principal,
                        remaining: uses,
                        deadline: Instant::now() + SESSION_TTL,
                    },
                );
                Ok(
                    json!({"ok":true,"capability":capability,"namespace":SESSION_NAMESPACE,"profile":self.config.profile,
                    "generation":self.generation,"instance":self.instance,"maxUses":uses,"expiresInSeconds":60}),
                )
            },
        }
    }

    pub fn lock(&mut self) -> Result<()> {
        // Drop the *whole* owning vault/storage: no retained key/provider clone.
        // This service owns OS-key retrieval; generic passphrase unlock is not
        // advertised or fabricated to make IdentityVault::lock accept a lock.
        self.vault.take();
        self.pending.clear();
        self.sessions.clear();
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(Refusal("generation-exhausted"))?;
        castellan::custody::persist_lock(&self.config.directory).map_err(storage_error)
    }

    /// Native OS-confirmed key handoff only, never an IPC credential. Failed
    /// unlock or marker clear leaves every operation locked and grants revoked.
    pub fn unlock_after_native_confirmation(&mut self, key: Zeroizing<[u8; 32]>) -> Result<()> {
        if self.vault.is_some() {
            return Err(Refusal("already-unlocked"));
        }
        self.load_key(&key, false)?;
        if let Err(error) = castellan::custody::clear_persisted_lock(&self.config.directory) {
            self.vault.take();
            let _ = castellan::custody::persist_lock(&self.config.directory);
            return Err(storage_error(error));
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or(Refusal("generation-exhausted"))?;
        Ok(())
    }
}

fn storage_error(error: personae::IdentityError) -> Refusal {
    storage(error)
}

fn validate_config(config: &SyntheticConfig) -> Result<()> {
    if config.run_id.len() != 32
        || !config.run_id.bytes().all(|c| c.is_ascii_hexdigit())
        || config.profile != "h1-synthetic"
        || config.principal.is_empty()
        || config.principal.len() > 256
        || !config.directory.is_absolute()
    {
        return Err(Refusal("invalid-synthetic-config"));
    }
    let marker_path = config.directory.join(".djinn-synthetic-custody");
    let metadata = std::fs::symlink_metadata(&marker_path)
        .map_err(|_| Refusal("synthetic-marker-required"))?;
    if !metadata.file_type().is_file() || metadata.len() != 32 {
        return Err(Refusal("synthetic-marker-mismatch"));
    }
    let mut marker = Vec::with_capacity(33);
    std::fs::File::open(marker_path)
        .map_err(|_| Refusal("synthetic-marker-required"))?
        .take(33)
        .read_to_end(&mut marker)
        .map_err(|_| Refusal("synthetic-marker-required"))?;
    if marker != config.run_id.as_bytes() {
        return Err(Refusal("synthetic-marker-mismatch"));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
