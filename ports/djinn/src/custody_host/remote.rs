// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [`ResidentCustody`] over the independently launched custodian: the only
//! implementation a production build has.
//!
//! Each method is one typed custody operation. The client verified the
//! custodian's OS identity before its first byte; a refusal maps to the
//! custody route's refusal and a lost custodian to "identity pending". A
//! follower thread turns the custodian's lock facts into the lock watch and
//! the holders' lock and re-derive, and treats a lost custodian as a lock:
//! honest holders forget their keys, and nothing local replaces it.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use custody_wire::contract::{Answer, CredentialOperation, Outcome, PayloadOperation, Purpose, Source};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::{Client, Operation};
use dramatis::authentication::PassphraseChangeFailureV1;
use dramatis::intents::{CreateProfileIntentV1, ImportSshKeyNativeIntentV1, PROFILE_CREATE_INTENT, PROFILE_SWITCH_INTENT, SwitchProfileIntentV1};
use djinn::door::custody::{CustodyRefusal, EpochKeyRequest, KeySource, ReleasedEpochKey, ReleasedKey, StationGrantRequest};
use djinn::door::custody_payload::{PayloadBytes, PayloadOperation as AppPayload, PayloadRequest, PayloadResult};
use dramatis::view::{IdentitySurfaceSnapshot, VaultLockView};
use personae::{Ed25519Keypair, Ed25519PublicKey, IdentityError, IdentityProvider, PersonaId, ProfileId, RetainedKeys};

use super::{CredentialCall, ImportedSshKey, LockHolder, ResidentCustody, VaultOutput, WalletPublic, refusal};
use crate::custodian::InstalledCustodian;
use crate::door::browser_carrier::NativeIdentityFailure;

const NONINTERACTIVE: Duration = Duration::from_secs(30);
/// Long enough for a consent petition the owner answers.
const PROMPTED: Duration = Duration::from_secs(150);

struct State {
    master: [u8; 32],
    profile: ProfileId,
    locked: bool,
}

/// Custody held by the custodian.
pub struct RemoteHost {
    installed: InstalledCustodian,
    client: Mutex<Option<Arc<Client>>>,
    state: Mutex<State>,
    lock: tokio::sync::watch::Sender<VaultLockView>,
    holders: Mutex<Vec<Arc<dyn LockHolder>>>,
    retained: Mutex<Vec<Arc<RetainedKeys>>>,
    /// The custodian instance, generation and lock last applied to the
    /// holders.
    applied: Mutex<Option<(String, u64, bool)>>,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|e| e.into_inner())
}

fn unavailable(reason: &str) -> CustodyRefusal {
    CustodyRefusal::Failed {
        reason: format!("the custodian is unavailable ({reason}); identity is pending"),
    }
}

fn unexpected() -> CustodyRefusal {
    CustodyRefusal::Failed {
        reason: "the custodian's answer did not check".into(),
    }
}

fn source(key: KeySource) -> Source {
    match key {
        KeySource::Persona => Source::Persona,
        KeySource::Wallet => Source::Wallet,
    }
}

impl RemoteHost {
    /// Connect to the installed custodian and learn its public state. The
    /// vault may be locked; [`Self::wait_unlocked`] waits for an identity.
    pub fn connect(installed: InstalledCustodian) -> Result<Arc<Self>, String> {
        let host = Arc::new(Self {
            installed,
            client: Mutex::new(None),
            state: Mutex::new(State {
                master: [0; 32],
                profile: ProfileId(String::new()),
                locked: true,
            }),
            lock: tokio::sync::watch::channel(VaultLockView::Locked).0,
            holders: Mutex::new(Vec::new()),
            retained: Mutex::new(Vec::new()),
            applied: Mutex::new(None),
        });
        host.client().map_err(|r| format!("{r:?}"))?;
        host.refresh().map_err(|r| format!("{r:?}"))?;
        let follower = Arc::clone(&host);
        std::thread::Builder::new()
            .name("custody-follower".into())
            .spawn(move || follower.follow())
            .map_err(|e| e.to_string())?;
        Ok(host)
    }

    /// Whether the custodian has opened the vault at least once, so the
    /// identity's public key is known.
    pub fn knows_identity(&self) -> bool {
        lock(&self.state).master != [0; 32]
    }

    fn client(&self) -> Result<Arc<Client>, CustodyRefusal> {
        let mut slot = lock(&self.client);
        if let Some(client) = slot.as_ref()
            && client.observed().alive
        {
            return Ok(Arc::clone(client));
        }
        #[cfg(unix)]
        let connector = custody_wire::transport::UnixSocketConnector::new(&self.installed.endpoint).with_signed(self.installed.server.signed.clone());
        #[cfg(windows)]
        let connector = custody_wire::transport::NamedPipeConnector::new(&self.installed.endpoint);
        let client = Arc::new(Client::open(&connector, &self.installed.server).map_err(|e| unavailable(&e.to_string()))?);
        *slot = Some(Arc::clone(&client));
        Ok(client)
    }

    fn call(&self, operation: Operation, deadline: Duration) -> Result<Answer, CustodyRefusal> {
        let client = self.client()?;
        let seen = client.observed().generation;
        let mut outcome = client.call(operation.clone(), deadline);
        // A refusal ran nothing; if the generation moved before its fact
        // reached us, wait for the fact and ask once more.
        if outcome.code() == Some("stale-generation") {
            client.wait_for_change(seen, Duration::from_secs(2));
            outcome = client.call(operation, deadline);
        }
        match outcome {
            Outcome::Success { answer } => Ok(answer),
            Outcome::Refused { code } => Err(refusal(&code)),
            Outcome::Degraded { code, .. } => Err(CustodyRefusal::Failed {
                reason: format!("the custodian reported a degraded outcome: {code}"),
            }),
            Outcome::Unknown { code } => Err(unavailable(&code)),
        }
    }

    /// Re-read public status: lock, profile and master.
    fn refresh(&self) -> Result<(), CustodyRefusal> {
        let Answer::Status { status } = self.call(Operation::Status {}, NONINTERACTIVE)? else {
            return Err(unexpected());
        };
        let mut state = lock(&self.state);
        state.locked = status.locked;
        if let Some(master) = status.master_public_key {
            state.master = master;
        }
        if let Some(profile) = status.profile {
            state.profile = ProfileId(profile);
        }
        Ok(())
    }

    /// Follow lock facts and custodian loss for the lock watch and holders.
    fn follow(self: Arc<Self>) {
        loop {
            match self.client() {
                Ok(client) if client.observed().alive => {
                    self.sync(&client);
                    let seen = client.observed().generation;
                    client.wait_for_change(seen, Duration::from_secs(5));
                },
                _ => {
                    // Custodian loss is a lock: honest holders forget keys.
                    *lock(&self.applied) = None;
                    self.apply_locked();
                    std::thread::sleep(Duration::from_secs(2));
                },
            }
        }
    }

    /// Apply the custodian's current state once: refresh status, then lock
    /// the holders or re-derive them through the custodian. The status is
    /// read every time, because an outcome can reach this client before the
    /// fact that names its generation.
    fn sync(&self, client: &Client) {
        let mut applied = lock(&self.applied);
        if self.refresh().is_err() {
            return;
        }
        let observed = client.observed();
        let locked = lock(&self.state).locked;
        let key = (observed.instance.clone(), observed.generation, locked);
        if applied.as_ref() == Some(&key) {
            return;
        }
        match locked {
            true => self.apply_locked(),
            false => self.apply_unlocked(),
        }
        *applied = Some(key);
    }

    fn apply_locked(&self) {
        lock(&self.state).locked = true;
        for holder in lock(&self.holders).iter() {
            holder.lock();
        }
        self.lock.send_replace(VaultLockView::Locked);
    }

    fn apply_unlocked(&self) {
        let holders: Vec<_> = lock(&self.holders).clone();
        for holder in holders {
            if let Err(error) = holder.unlock(self) {
                tracing::warn!(holder = holder.name(), %error, "holder stayed locked after the custodian unlocked");
                holder.lock();
            }
        }
        self.lock.send_replace(VaultLockView::Unlocked);
    }

    fn released(&self, source: Source, profile: Option<&ProfileId>, salts: &[Vec<u8>]) -> Result<([u8; 32], Vec<(Vec<u8>, Ed25519Keypair, insigne::DerivedKeyAttestation)>), CustodyRefusal> {
        let operation = Operation::Derive {
            source,
            profile: profile.map(|p| p.0.clone()),
            salts: salts.iter().map(|s| Bytes(s.clone())).collect(),
        };
        let Answer::Released { master_public_key, keys } = self.call(operation, PROMPTED)? else {
            return Err(unexpected());
        };
        if keys.len() != salts.len() || keys.iter().zip(salts).any(|(k, s)| &k.salt.0 != s) {
            return Err(unexpected());
        }
        let master = Ed25519PublicKey::from_bytes(&master_public_key).map_err(|_| unexpected())?;
        let mut out = Vec::with_capacity(keys.len());
        for key in keys {
            let seed = zeroize::Zeroizing::new(<[u8; 32]>::try_from(key.seed.expose()).map_err(|_| unexpected())?);
            let attestation = insigne::DerivedKeyAttestation::from_parts(key.attestation.master, key.attestation.derived, key.attestation.signature.0);
            out.push((key.salt.0, Ed25519Keypair::from_seed(*seed), attestation));
        }
        // The same checks a retained release makes: every attestation checks
        // under its salt, names this master and names the key it came with.
        RetainedKeys::from_released(master, out.iter().map(|(s, k, a)| (s.clone(), Ed25519Keypair::from_seed(k.to_seed()), a.clone()))).map_err(|_| unexpected())?;
        Ok((master_public_key, out))
    }
}

impl IdentityProvider for RemoteHost {
    fn master_public_key(&self) -> Ed25519PublicKey {
        let master = lock(&self.state).master;
        Ed25519PublicKey::from_bytes(&master).expect("composed only after the custodian opened the vault")
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, IdentityError> {
        match self.released(Source::Persona, None, &[salt.to_vec()]) {
            Ok((_, mut keys)) => Ok(keys.remove(0).1),
            Err(CustodyRefusal::Locked) => Err(IdentityError::Locked),
            Err(other) => Err(IdentityError::Backend(format!("{other:?}"))),
        }
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<insigne::DerivedKeyAttestation, IdentityError> {
        self.attest(KeySource::Persona, None, salt).map_err(|refusal| match refusal {
            CustodyRefusal::Locked => IdentityError::Locked,
            other => IdentityError::Backend(format!("{other:?}")),
        })
    }
}

impl ResidentCustody for RemoteHost {
    fn custody_epoch(
        &self,
    ) -> Result<Option<djinn::door::custody::CustodyEpoch>, CustodyRefusal> {
        let client = self.client()?;
        // Read authoritative status rather than the local boolean watch. Its
        // generation remembers transitions even before this caller subscribed.
        match client.call(Operation::Status {}, Duration::from_secs(2)) {
            Outcome::Success {
                answer: Answer::Status { status },
            } => {
                let epoch = djinn::door::custody::CustodyEpoch {
                    instance: status.instance,
                    generation: status.generation,
                };
                if epoch.valid() {
                    Ok(Some(epoch))
                } else {
                    Err(unexpected())
                }
            },
            Outcome::Refused { code } => Err(refusal(&code)),
            Outcome::Degraded { .. } | Outcome::Unknown { .. } => {
                Err(unavailable("status epoch unavailable"))
            },
            _ => Err(unexpected()),
        }
    }
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        match self.call(Operation::Snapshot {}, NONINTERACTIVE) {
            Ok(Answer::Snapshot { snapshot }) => Ok(*snapshot),
            Ok(_) => Err(std::io::Error::other("unexpected custodian answer")),
            Err(refusal) => Err(std::io::Error::other(format!("{refusal:?}"))),
        }
    }

    fn is_locked(&self) -> bool {
        lock(&self.state).locked
    }

    fn lock_state(&self) -> tokio::sync::watch::Receiver<VaultLockView> {
        self.lock.subscribe()
    }

    fn lock_vault(&self) -> Result<(), String> {
        let result = self.call(Operation::Lock {}, NONINTERACTIVE).map(|_| ()).map_err(|r| format!("{r:?}"));
        // The lock fact precedes the outcome on the same stream.
        if let Ok(client) = self.client() {
            self.sync(&client);
        }
        result
    }

    fn unlock_with_passphrase(&self, passphrase: &[u8]) -> Result<(), String> {
        let operation = Operation::Unlock {
            passphrase: Secret::copy_from(passphrase),
        };
        match self.call(operation, Duration::from_secs(60)) {
            Ok(_) => {},
            Err(CustodyRefusal::Refused { reason }) if reason.ends_with("already-unlocked") => {},
            Err(refusal) => return Err(format!("{refusal:?}")),
        }
        let client = self.client().map_err(|r| format!("{r:?}"))?;
        self.sync(&client);
        match self.is_locked() {
            true => Err("the custodian is still locked".into()),
            false => Ok(()),
        }
    }

    fn register_lock_holder(&self, holder: Arc<dyn LockHolder>) {
        if self.is_locked() {
            holder.lock();
        }
        lock(&self.holders).push(holder);
    }

    fn retained_keys(&self, salts: &[Vec<u8>]) -> Result<Arc<RetainedKeys>, IdentityError> {
        let master = self.master_public_key();
        let mut retained = lock(&self.retained);
        if let Some(kept) = retained.iter().find(|kept| kept.master_public_key() == master && kept.holds(salts)) {
            return Ok(Arc::clone(kept));
        }
        let (_, released) = self.released(Source::Persona, None, salts).map_err(|refusal| match refusal {
            CustodyRefusal::Locked => IdentityError::Locked,
            other => IdentityError::Backend(format!("{other:?}")),
        })?;
        let captured = Arc::new(RetainedKeys::from_released(master, released)?);
        retained.retain(|kept| kept.master_public_key() == master);
        retained.push(Arc::clone(&captured));
        Ok(captured)
    }

    fn current_profile_id(&self) -> ProfileId {
        lock(&self.state).profile.clone()
    }

    fn has_profile(&self, profile: &ProfileId) -> Result<bool, IdentityError> {
        match self.call(Operation::HasProfile { profile: profile.0.clone() }, NONINTERACTIVE) {
            Ok(Answer::Bool { value }) => Ok(value),
            Ok(_) => Err(IdentityError::Backend("unexpected custodian answer".into())),
            Err(CustodyRefusal::Locked) => Err(IdentityError::Locked),
            Err(other) => Err(IdentityError::Backend(format!("{other:?}"))),
        }
    }

    fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
        if intent == dramatis::intents::VAULT_LOCK_INTENT {
            return self.lock_vault();
        }
        let operation = Operation::Intent {
            intent: intent.into(),
            payload: Bytes(payload.to_vec()),
        };
        let result = self.call(operation, PROMPTED).map(|_| ()).map_err(|refusal| match refusal {
            CustodyRefusal::Refused { reason } => reason,
            other => format!("{other:?}"),
        });
        let _ = self.refresh();
        result
    }

    fn switch_profile(&self, intent: SwitchProfileIntentV1) -> Result<(), String> {
        self.apply_intent(PROFILE_SWITCH_INTENT, &serde_json::to_vec(&intent).map_err(|e| e.to_string())?)
    }

    fn create_profile(&self, intent: CreateProfileIntentV1) -> Result<(), String> {
        self.apply_intent(PROFILE_CREATE_INTENT, &serde_json::to_vec(&intent).map_err(|e| e.to_string())?)
    }

    fn release(&self, key: KeySource, profile: Option<&ProfileId>, salts: &[Vec<u8>]) -> Result<([u8; 32], Vec<ReleasedKey>), CustodyRefusal> {
        let (master, keys) = self.released(source(key), profile, salts)?;
        Ok((
            master,
            keys.into_iter()
                .map(|(salt, keypair, attestation)| ReleasedKey {
                    salt,
                    seed: keypair.to_seed(),
                    attestation,
                })
                .collect(),
        ))
    }

    fn attest(&self, key: KeySource, profile: Option<&ProfileId>, salt: &[u8]) -> Result<insigne::DerivedKeyAttestation, CustodyRefusal> {
        let operation = Operation::Attest {
            source: source(key),
            profile: profile.map(|p| p.0.clone()),
            salt: Bytes(salt.to_vec()),
        };
        let Answer::Attestation { attestation } = self.call(operation, PROMPTED)? else {
            return Err(unexpected());
        };
        let attestation = insigne::DerivedKeyAttestation::from_parts(attestation.master, attestation.derived, attestation.signature.0);
        attestation.check(salt).map_err(|_| unexpected())?;
        Ok(attestation)
    }

    fn sign(&self, key: KeySource, profile: Option<&ProfileId>, salt: &[u8], message: &[u8]) -> Result<([u8; 32], Vec<u8>), CustodyRefusal> {
        let operation = Operation::SignDerived {
            source: source(key),
            profile: profile.map(|p| p.0.clone()),
            salt: Bytes(salt.to_vec()),
            message: Bytes(message.to_vec()),
        };
        let Answer::Signature { public_key, signature, .. } = self.call(operation, PROMPTED)? else {
            return Err(unexpected());
        };
        verify(&public_key, &signature, message)?;
        Ok((public_key, signature.to_vec()))
    }

    fn sign_delegation(&self, audience: &str, salt: &[u8], message: &[u8]) -> Result<([u8; 32], Vec<u8>), CustodyRefusal> {
        let operation = Operation::Sign {
            profile: self.current_profile_id().0,
            purpose: Purpose::Delegation { audience: audience.into() },
            salt: Bytes(salt.to_vec()),
            message: Bytes(message.to_vec()),
        };
        let Answer::Signature { public_key, signature, attestation: Some(attestation) } = self.call(operation, PROMPTED)? else {
            return Err(unexpected());
        };
        let signer = insigne::DerivedKeyAttestation::from_parts(attestation.master, attestation.derived, attestation.signature.0);
        let checked = signer.check(salt).map_err(|_| unexpected())?;
        if checked.derived() != &public_key {
            return Err(unexpected());
        }
        verify(&public_key, &signature, message)?;
        Ok((public_key, signature.to_vec()))
    }

    fn root_public_key(&self, key: KeySource, profile: Option<&ProfileId>) -> Result<[u8; 32], CustodyRefusal> {
        let operation = Operation::PublicKey {
            source: source(key),
            profile: profile.map(|p| p.0.clone()),
        };
        match self.call(operation, NONINTERACTIVE)? {
            Answer::PublicKey { key } => Ok(key),
            _ => Err(unexpected()),
        }
    }

    fn vault_command(&self, profile: &ProfileId, command: &str, args: &[String]) -> Result<VaultOutput, CustodyRefusal> {
        let operation = Operation::VaultCommand {
            profile: profile.0.clone(),
            command: command.into(),
            args: args.to_vec(),
        };
        match self.call(operation, PROMPTED)? {
            Answer::VaultOutput { text } => Ok(VaultOutput::Text(text)),
            Answer::VaultEnrollment { target, script, confirmation } => Ok(VaultOutput::Enrollment { target, script, confirmation }),
            _ => Err(unexpected()),
        }
    }

    fn change_passphrase(&self, old: &[u8], new: &[u8]) -> Result<(), PassphraseChangeFailureV1> {
        let operation = Operation::ChangePassphrase {
            old: Secret::copy_from(old),
            new: Secret::copy_from(new),
        };
        match self.call(operation, PROMPTED) {
            Ok(_) => Ok(()),
            Err(CustodyRefusal::Locked) => Err(PassphraseChangeFailureV1::Locked),
            Err(CustodyRefusal::Refused { reason }) if reason.contains("IncorrectCurrent") => Err(PassphraseChangeFailureV1::CurrentPassphraseRejected),
            Err(CustodyRefusal::Refused { reason }) if reason.contains("InvalidNew") => Err(PassphraseChangeFailureV1::InvalidNewPassphrase),
            Err(CustodyRefusal::Refused { reason }) if reason.contains("Unsupported") => Err(PassphraseChangeFailureV1::Unsupported),
            Err(_) => Err(PassphraseChangeFailureV1::StorageFailure),
        }
    }

    fn import_ssh_key(&self, openssh: &[u8], passphrase: Option<&[u8]>, options: ImportSshKeyNativeIntentV1) -> Result<ImportedSshKey, NativeIdentityFailure> {
        let operation = Operation::ImportSshKey {
            key: Secret::copy_from(openssh),
            passphrase: passphrase.map(Secret::copy_from),
            options: Bytes(serde_json::to_vec(&options).map_err(|_| NativeIdentityFailure::ImportRejected)?),
        };
        match self.call(operation, PROMPTED) {
            Ok(Answer::Value { value }) => serde_json::from_value(value).map_err(|_| NativeIdentityFailure::ImportRejected),
            Err(CustodyRefusal::Refused { reason }) if reason.contains("did not open it") => Err(NativeIdentityFailure::IncorrectPassphrase),
            Err(CustodyRefusal::Refused { reason }) if reason.contains("not an OpenSSH private key") => Err(NativeIdentityFailure::InvalidPrivateKey),
            _ => Err(NativeIdentityFailure::ImportRejected),
        }
    }

    fn epoch_keys(&self, _wallet_root: &Path, persona: PersonaId, device_label: Option<&str>, requests: &[EpochKeyRequest]) -> Result<Vec<ReleasedEpochKey>, CustodyRefusal> {
        let operation = Operation::EpochKeys {
            persona: persona.as_uuid().to_string(),
            device_label: device_label.map(str::to_string),
            keys: requests
                .iter()
                .map(|r| custody_wire::EpochKeyRequest {
                    context: r.context.clone(),
                    device_bound: r.device_bound,
                    public_only: r.public_only,
                })
                .collect(),
        };
        let Answer::EpochKeys { keys } = self.call(operation, PROMPTED)? else {
            return Err(unexpected());
        };
        if keys.len() != requests.len() {
            return Err(unexpected());
        }
        keys.into_iter()
            .zip(requests)
            .map(|(key, request)| {
                Ok(ReleasedEpochKey {
                    request: request.clone(),
                    key: <[u8; 32]>::try_from(key.key.expose()).map_err(|_| unexpected())?,
                })
            })
            .collect()
    }

    fn station_grant(&self, _wallet_root: &Path, request: &StationGrantRequest, issuer: [u8; 32]) -> Result<personae::carry::DeviceGrantSet, CustodyRefusal> {
        let wire = serde_json::json!({
            "device_id": request.device_id,
            "station_ed25519_public_key": request.station_ed25519_public_key,
            "label": request.label,
            "issued_at_ms": request.issued_at_ms,
            "expires_at_ms": request.expires_at_ms,
        });
        let operation = Operation::StationGrant {
            request: wire,
            issuer_public_key: Bytes(issuer.to_vec()),
        };
        match self.call(operation, PROMPTED)? {
            Answer::Value { value } => serde_json::from_value(value).map_err(|_| unexpected()),
            _ => Err(unexpected()),
        }
    }

    fn revoke_device(&self, _wallet_root: &Path, device: uuid::Uuid) -> Result<personae::carry::RemoteAuthRevocationOutcome, CustodyRefusal> {
        match self.call(Operation::RevokeDevice { device_id: device.to_string() }, PROMPTED)? {
            Answer::Value { value } => serde_json::from_value(value).map_err(|_| unexpected()),
            _ => Err(unexpected()),
        }
    }

    fn wallet_public(&self, _wallet_root: &Path) -> Result<WalletPublic, CustodyRefusal> {
        let Answer::WalletPublic { personas, roster } = self.call(Operation::WalletPublic {}, NONINTERACTIVE)? else {
            return Err(unexpected());
        };
        Ok(WalletPublic {
            personas: personas
                .iter()
                .map(|persona| uuid::Uuid::parse_str(persona).map(PersonaId::from_uuid).map_err(|_| unexpected()))
                .collect::<Result<_, _>>()?,
            roster: roster.map(serde_json::from_value).transpose().map_err(|_| unexpected())?,
        })
    }

    fn payload(&self, _wallet_root: &Path, persona: PersonaId, request: PayloadRequest, scope: Vec<u8>) -> Result<PayloadResult, CustodyRefusal> {
        let expected_epoch = request.custody_epoch.clone();
        let content_hash = match serde_json::to_value(request.content_hash) {
            Ok(serde_json::Value::String(text)) => text,
            _ => return Err(unexpected()),
        };
        let (operation, sealing) = match request.operation {
            AppPayload::Seal { bytes } => (PayloadOperation::Seal { bytes: Bytes(bytes.0) }, true),
            AppPayload::Unseal { bytes, marker } => (
                PayloadOperation::Unseal {
                    bytes: Bytes(bytes.0),
                    marker: serde_json::to_value(marker).map_err(|_| unexpected())?,
                },
                false,
            ),
        };
        let operation = Operation::Payload {
            profile: request.profile.0.clone(),
            persona: persona.as_uuid().to_string(),
            master: Bytes(request.master.to_vec()),
            scope: Bytes(scope),
            content_hash,
            operation,
        };
        let answer = match expected_epoch.as_ref() {
            Some(expected) => {
                let client = self.client()?;
                let observed = client.observed();
                if !observed.alive || observed.instance != expected.instance {
                    return Err(unavailable("retained payload custodian changed"));
                }
                // Exact generation admission: unlike the legacy call helper,
                // never refresh/retry this private operation after revocation.
                match client.call_at(operation, expected.generation, NONINTERACTIVE) {
                    Outcome::Success { answer } => answer,
                    Outcome::Refused { code } => return Err(refusal(&code)),
                    Outcome::Unknown { .. } | Outcome::Degraded { .. } => return Err(unavailable("retained payload outcome unresolved")),
                }
            },
            None => self.call(operation, PROMPTED)?,
        };
        if let Some(expected) = expected_epoch.as_ref() {
            if self.custody_epoch()?.as_ref() != Some(expected) {
                return Err(unavailable("retained payload custody changed after admission"));
            }
        }
        let Answer::Payload { bytes, marker } = answer else {
            return Err(unexpected());
        };
        Ok(match (sealing, marker) {
            (true, Some(marker)) => PayloadResult::Sealed {
                bytes: PayloadBytes(bytes.0),
                marker: serde_json::from_value(marker).map_err(|_| unexpected())?,
            },
            (false, None) => PayloadResult::Clear { bytes: PayloadBytes(bytes.0) },
            _ => return Err(unexpected()),
        })
    }

    fn credentials(&self, call: CredentialCall) -> Result<serde_json::Value, CustodyRefusal> {
        let operation = match call {
            CredentialCall::Quarantine(profile) => CredentialOperation::Quarantine { profile: profile.0 },
            CredentialCall::UnassignedImports => CredentialOperation::UnassignedImports {},
            CredentialCall::AssignImport(request) => CredentialOperation::AssignImport { request },
            CredentialCall::Review(request) => CredentialOperation::Review { request },
            CredentialCall::CxfPreview(source) => CredentialOperation::CxfPreview { source: Bytes(source) },
            CredentialCall::CxfImport { source, destinations, preview_sha256 } => CredentialOperation::CxfImport {
                source: Bytes(source),
                destinations,
                preview_sha256,
            },
        };
        match self.call(Operation::Credentials { operation }, PROMPTED)? {
            Answer::Value { value } => Ok(value),
            _ => Err(unexpected()),
        }
    }
}

fn verify(public_key: &[u8; 32], signature: &[u8; 64], message: &[u8]) -> Result<(), CustodyRefusal> {
    let verified = Ed25519PublicKey::from_bytes(public_key)
        .map(|key| key.verify(message, &personae::Ed25519Signature::from_bytes(signature)))
        .unwrap_or(false);
    verified.then_some(()).ok_or_else(unexpected)
}
