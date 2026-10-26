// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Castellan's resident authority, held only here (custody isolation plan,
//! H2). The same `PersonaeHost` Djinn used to build in its own process now
//! lives in the custodian with the wallet and the credential catalog, over
//! Djinn's unchanged vault, wallet and credential formats. Every start is
//! locked; the host is built at the first unlock and thereafter locks and
//! unlocks in place, its holders following. Nothing here ever mints a
//! profile or a replacement root, and no master seed leaves: Djinn receives
//! public views, results, and the derived keys a grant names.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use castellan::authority::{PersonaeHost, VaultCommandOutput};
use castellan::custody::bootstrap::{self, PASSPHRASE_VAULT_FILE, Unlock};
use castellan::custody::{AUTO_UNLOCK_ROOT_FILE, IdentityStorage, IdentityVault, UnlockMethod};
use castellan::resident::{CastellanResident, CredentialSalts};
use custody_wire::contract::{CredentialOperation, EpochKeyRequest, PayloadOperation, ReleasedEpochKey, Source};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::AttestationParts;
use dramatis::view::{CarryView, IdentitySurfaceSnapshot, VaultLockView, VaultProtectionView, VaultView};
use personae::{Ed25519Keypair, IdentityProvider, InMemoryProvider, PersonaId, ProfileId};
use zeroize::Zeroizing;

use crate::Refusal;

type Host = PersonaeHost<Box<dyn IdentityStorage>>;

/// The credential catalog's two salts, exactly as Djinn derived them.
const CREDENTIAL_SALTS: CredentialSalts = CredentialSalts {
    record: b"mere.djinn/castellan/records/v1",
    freshness: b"mere.djinn/castellan/freshness/v1",
};

/// One released derived key, seed in a clearing buffer.
pub struct Released {
    pub salt: Vec<u8>,
    pub seed: Zeroizing<[u8; 32]>,
    pub attestation: AttestationParts,
}

/// A signature made inside custody.
pub struct Signed {
    pub public_key: [u8; 32],
    pub signature: [u8; 64],
    pub attestation: Option<AttestationParts>,
}

/// What a vault command answered.
pub enum CommandOutput {
    Text(String),
    Enrollment { target: String, script: String, confirmation: String },
}

/// The vault, wallet and credential catalog as the custodian holds them.
pub struct HostCustody {
    directory: PathBuf,
    data_root: Option<PathBuf>,
    configured: Option<ProfileId>,
    host: Option<Arc<Host>>,
    credentials: BTreeMap<ProfileId, CastellanResident>,
    known_profile: Option<ProfileId>,
    known_master: Option<[u8; 32]>,
}

fn storage(_: impl std::fmt::Display) -> Refusal {
    Refusal("vault-storage")
}

fn identity(error: personae::IdentityError) -> Refusal {
    match error {
        personae::IdentityError::Locked => Refusal("locked"),
        _ => Refusal("identity-refused"),
    }
}

impl HostCustody {
    /// Hold `directory` locked. `data_root` holds the wallet (`identity/`)
    /// and the credential catalog (`castellan/`) in Djinn's layout.
    pub fn locked(directory: &Path, profile: Option<&str>, data_root: Option<&Path>) -> Result<Self, Refusal> {
        if !directory.is_absolute() || data_root.is_some_and(|root| !root.is_absolute()) {
            return Err(Refusal("vault-directory-not-absolute"));
        }
        if !directory.join(PASSPHRASE_VAULT_FILE).is_file() {
            if directory.join(AUTO_UNLOCK_ROOT_FILE).exists() {
                return Err(Refusal("os-sealed-vault-needs-migration"));
            }
            return Err(Refusal("vault-missing"));
        }
        Ok(Self {
            directory: directory.to_path_buf(),
            data_root: data_root.map(Path::to_path_buf),
            configured: profile.map(|p| ProfileId(p.to_string())),
            host: None,
            credentials: BTreeMap::new(),
            known_profile: None,
            known_master: None,
        })
    }

    pub fn is_locked(&self) -> bool {
        self.host.as_ref().is_none_or(|host| host.is_locked())
    }

    pub fn profile(&self) -> Option<&str> {
        self.known_profile.as_ref().map(|p| p.0.as_str())
    }

    pub fn master_public_key(&self) -> Option<[u8; 32]> {
        self.known_master
    }

    pub fn protection(&self) -> String {
        format!(
            "passphrase-encrypted vault at {}, held by the custodian",
            self.directory.join(PASSPHRASE_VAULT_FILE).display()
        )
    }

    fn unlocked(&self) -> Result<&Arc<Host>, Refusal> {
        match &self.host {
            Some(host) if !host.is_locked() => Ok(host),
            _ => Err(Refusal("locked")),
        }
    }

    fn wallet(&self) -> Result<&Path, Refusal> {
        self.data_root.as_deref().ok_or(Refusal("no-wallet"))
    }

    /// The wallet root's public key, while unlocked and present.
    pub fn wallet_public_key(&self) -> Option<[u8; 32]> {
        self.unlocked().ok()?;
        let seed = castellan::custody::wallet::load_identity_seed(self.data_root.as_deref()?).ok()??;
        Some(InMemoryProvider::from_seed(seed).master_public_key().to_bytes())
    }

    /// Open with the passphrase: build the host at the first unlock, unlock
    /// it in place afterwards. Any refusal leaves everything locked.
    pub fn unlock(&mut self, passphrase: &[u8]) -> Result<(), Refusal> {
        if let Some(host) = &self.host {
            if !host.is_locked() {
                return Err(Refusal("already-unlocked"));
            }
            return host
                .unlock_vault(UnlockMethod::Passphrase(passphrase))
                .map_err(|_| Refusal("unlock-refused"));
        }
        let mut owned = Zeroizing::new(Vec::with_capacity(passphrase.len()));
        owned.extend_from_slice(passphrase);
        let opened = bootstrap::open_storage(&self.directory, Unlock::Passphrase(owned)).map_err(|_| Refusal("unlock-refused"))?;
        let profile = self.resolve(&*opened.storage)?;
        let vault = IdentityVault::open(opened.storage, &profile).map_err(|_| Refusal("profile-unavailable"))?;
        let master = vault.master_public_key().to_bytes();
        if self.known_master.is_some_and(|known| known != master) {
            return Err(Refusal("vault-identity-changed"));
        }
        let host = Arc::new(
            PersonaeHost::new(vault, self.data_root.clone(), VaultProtectionView::Passphrase)
                .with_vault_dir(self.directory.clone())
                .with_persisted_lock(self.directory.clone()),
        );
        if castellan::custody::clear_persisted_lock(&self.directory).is_err() {
            let _ = host.lock_vault();
            return Err(Refusal("lock-marker-not-cleared"));
        }
        if let Some(root) = &self.data_root {
            self.credentials = claim_credentials(&host, root, &profile)?;
        }
        self.known_master = Some(master);
        self.known_profile = Some(profile);
        self.host = Some(host);
        Ok(())
    }

    /// The configured profile, else the remembered choice read directly
    /// (no environment), else the vault's sole profile.
    fn resolve(&self, opened: &dyn IdentityStorage) -> Result<ProfileId, Refusal> {
        let summaries = opened.list_profiles().map_err(storage)?;
        let present = |id: &ProfileId| summaries.iter().any(|s| &s.id == id);
        if let Some(configured) = &self.configured {
            return present(configured).then(|| configured.clone()).ok_or(Refusal("profile-missing"));
        }
        let remembered = std::fs::read_to_string(castellan::custody::roster::choice_path(&self.directory))
            .ok()
            .map(|s| ProfileId(s.trim().to_string()))
            .filter(|id| !id.0.is_empty());
        if let Some(remembered) = remembered {
            return present(&remembered).then_some(remembered).ok_or(Refusal("profile-missing"));
        }
        match summaries.as_slice() {
            [only] => Ok(only.id.clone()),
            _ => Err(Refusal("profile-unresolved")),
        }
    }

    /// Lock the host and every holder; `true` when the restart barrier was
    /// written and synced. Memory locks either way.
    pub fn lock(&mut self) -> bool {
        match &self.host {
            Some(host) => {
                let durable = host.lock_vault().is_ok();
                // A failed barrier still locks memory (C07); persist again
                // so the report reflects the disk, not the attempt.
                durable && castellan::custody::lock_persisted(&self.directory)
            },
            None => castellan::custody::persist_lock(&self.directory).is_ok(),
        }
    }

    /// The public read model; before the first unlock, a locked view with
    /// no profiles rather than an error.
    pub fn snapshot(&self) -> Result<IdentitySurfaceSnapshot, Refusal> {
        match &self.host {
            Some(host) => host.snapshot().map_err(storage),
            None => Ok(IdentitySurfaceSnapshot {
                vault: VaultView {
                    protection: VaultProtectionView::Passphrase,
                    lock: VaultLockView::Locked,
                    agent: dramatis::view::AgentListenerView::StandaloneRetained,
                },
                profiles: Vec::new(),
                ssh_keys: Vec::new(),
                carry: CarryView::default(),
                pending_signing: Vec::new(),
                signing_history: Vec::new(),
            }),
        }
    }

    /// A provider for `source` and `profile`. A named persona's master stays
    /// in this process for the length of the call.
    fn provider(&self, source: Source, profile: Option<&str>) -> Result<Box<dyn IdentityProvider>, Refusal> {
        let host = self.unlocked()?;
        match source {
            Source::Persona => match profile {
                Some(name) if name != host.current_profile_id().0 => host
                    .profile_provider(&ProfileId(name.to_string()))
                    .map(|p| Box::new(p) as Box<dyn IdentityProvider>)
                    .map_err(identity),
                _ => Ok(Box::new(HostProvider(Arc::clone(host)))),
            },
            Source::Wallet => {
                let seed = castellan::custody::wallet::load_identity_seed(self.wallet()?)
                    .map_err(storage)?
                    .ok_or(Refusal("no-wallet"))?;
                Ok(Box::new(InMemoryProvider::from_seed(seed)))
            },
        }
    }

    /// Sign with the key `salt` derives, or with the persona master for an
    /// empty salt. The key never leaves this call.
    pub fn sign(&self, source: Source, profile: Option<&str>, salt: &[u8], message: &[u8]) -> Result<Signed, Refusal> {
        if salt.is_empty() {
            if source != Source::Persona || profile.is_some_and(|p| Some(p) != self.profile()) {
                return Err(Refusal("root-signature-is-the-open-persona"));
            }
            let host = self.unlocked()?;
            let signed = host
                .with_profile_authority(&host.current_profile_id(), || ())
                .map_err(identity);
            signed?;
            return root_sign(host, message);
        }
        let provider = self.provider(source, profile)?;
        let key = provider.derive_keypair(salt).map_err(identity)?;
        let attestation = provider.attest_derived_key(salt).map_err(identity)?;
        Ok(Signed {
            public_key: key.public_key().to_bytes(),
            signature: key.sign(message).to_bytes(),
            attestation: Some(parts(&attestation)),
        })
    }

    /// The derived keys for `salts`, each with its attestation. Never a root.
    pub fn release(&self, source: Source, profile: Option<&str>, salts: &[Vec<u8>]) -> Result<([u8; 32], Vec<Released>), Refusal> {
        let provider = self.provider(source, profile)?;
        let mut released = Vec::with_capacity(salts.len());
        for salt in salts {
            if salt.is_empty() {
                return Err(Refusal("master-never-released"));
            }
            let key = provider.derive_keypair(salt).map_err(identity)?;
            let attestation = provider.attest_derived_key(salt).map_err(identity)?;
            released.push(Released {
                salt: salt.clone(),
                seed: Zeroizing::new(key.to_seed()),
                attestation: parts(&attestation),
            });
        }
        Ok((provider.master_public_key().to_bytes(), released))
    }

    pub fn attest(&self, source: Source, profile: Option<&str>, salt: &[u8]) -> Result<AttestationParts, Refusal> {
        if salt.is_empty() {
            return Err(Refusal("master-never-attested"));
        }
        let provider = self.provider(source, profile)?;
        provider.attest_derived_key(salt).map(|a| parts(&a)).map_err(identity)
    }

    /// One identity intent. Signing decisions belong to the custodian's own
    /// consent channel and are refused here (C04).
    pub fn intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
        use dramatis::intents::{SIGNING_APPROVE_IDLE_INTENT, SIGNING_APPROVE_ONCE_INTENT, SIGNING_DENY_INTENT};
        if matches!(intent, SIGNING_APPROVE_IDLE_INTENT | SIGNING_APPROVE_ONCE_INTENT | SIGNING_DENY_INTENT) {
            return Err("signing decisions are made only on the custodian's consent channel".into());
        }
        let host = self.host.as_ref().ok_or("vault is locked")?;
        host.apply_intent(intent, payload).map(|_| ()).map_err(|error| error.to_string())
    }

    pub fn has_profile(&self, profile: &str) -> Result<bool, Refusal> {
        self.unlocked()?.has_profile(&ProfileId(profile.into())).map_err(identity)
    }

    pub fn change_passphrase(&self, old: &[u8], new: &[u8]) -> Result<(), String> {
        let host = self.host.as_ref().ok_or("vault is locked")?;
        host.change_passphrase(old, new).map_err(|error| format!("{error:?}"))
    }

    /// Store an OpenSSH private key the owner picked on their own surface.
    pub fn import_ssh_key(&self, key: &[u8], passphrase: Option<&[u8]>, options: &[u8]) -> Result<serde_json::Value, String> {
        let host = self.unlocked().map_err(|r| r.0.to_string())?;
        let options: dramatis::intents::ImportSshKeyNativeIntentV1 = serde_json::from_slice(options).map_err(|_| "invalid import options")?;
        let mut private = ssh_key::PrivateKey::from_openssh(key).map_err(|_| "not an OpenSSH private key")?;
        if private.is_encrypted() {
            let passphrase = passphrase.ok_or("the key is encrypted")?;
            private = private.decrypt(passphrase).map_err(|_| "the key passphrase did not open it")?;
        }
        let receipt = host.import_ssh_private(private, options).map_err(|error| error.to_string())?;
        serde_json::to_value(receipt).map_err(|error| error.to_string())
    }

    pub fn vault_command(&self, profile: &str, command: &str, args: &[String]) -> Result<CommandOutput, Refusal> {
        let host = self.unlocked()?;
        match host.vault_command(&ProfileId(profile.into()), command, args) {
            Ok(VaultCommandOutput::Text(text)) => Ok(CommandOutput::Text(text)),
            Ok(VaultCommandOutput::Enrollment { target, script, confirmation }) => Ok(CommandOutput::Enrollment { target, script, confirmation }),
            Err(castellan::authority::IdentityIntentError::Locked) => Err(Refusal("locked")),
            Err(_) => Err(Refusal("vault-command-refused")),
        }
    }

    /// Keys derived from `persona`'s current private epoch, the device root
    /// mixed in where asked. The epoch itself never crosses.
    pub fn epoch_keys(&self, persona: &str, device_label: Option<&str>, requests: &[EpochKeyRequest]) -> Result<Vec<ReleasedEpochKey>, Refusal> {
        self.unlocked()?;
        let root = self.wallet()?;
        let persona = persona_id(persona)?;
        if requests.is_empty() || requests.iter().any(|r| r.context.is_empty()) {
            return Err(Refusal("operation-or-scope"));
        }
        if pandect::wallet_store::load_persona_wallet(root, persona).map_err(storage)?.is_none() {
            return Err(Refusal("no-persona"));
        }
        let epoch = castellan::custody::wallet::load_current_private_epoch(root, persona)
            .map_err(storage)?
            .ok_or(Refusal("locked"))?;
        let epoch = Zeroizing::new(epoch.epoch_secret);
        let device_root = match requests.iter().any(|r| r.device_bound) {
            true => Some(device_root(root, device_label)?),
            false => None,
        };
        Ok(requests
            .iter()
            .map(|request| {
                let mut material = Zeroizing::new(Vec::with_capacity(64));
                material.extend_from_slice(&*epoch);
                if let (true, Some(device_root)) = (request.device_bound, device_root.as_ref()) {
                    material.extend_from_slice(device_root);
                }
                let derived = Zeroizing::new(blake3::derive_key(&request.context, &material));
                let key = match request.public_only {
                    true => Ed25519Keypair::from_seed(*derived).public_key().to_bytes(),
                    false => *derived,
                };
                ReleasedEpochKey {
                    request: request.clone(),
                    key: Secret::copy_from(&key),
                }
            })
            .collect())
    }

    /// Issue a sited station's narrow RemoteAuth grant with the wallet seed.
    pub fn station_grant(&self, request: &serde_json::Value, issuer_public_key: &[u8]) -> Result<serde_json::Value, String> {
        use pandect::station_grant::{SitedStationGrant, SitedStationGrantRequest};
        self.unlocked().map_err(|r| r.0.to_string())?;
        let root = self.wallet().map_err(|r| r.0.to_string())?;
        let wire: StationGrantWire = serde_json::from_value(request.clone()).map_err(|_| "invalid station grant request")?;
        let issuer: [u8; 32] = issuer_public_key.try_into().map_err(|_| "issuer key is not 32 bytes")?;
        let wallet_public_key = self
            .provider(Source::Wallet, None)
            .map_err(|r| r.0.to_string())?
            .master_public_key()
            .to_bytes();
        let request = SitedStationGrantRequest::new(
            pandect::DeviceId::from_uuid(wire.device_id),
            wire.station_ed25519_public_key,
            wire.label,
            wire.issued_at_ms,
            wire.expires_at_ms,
        )
        .map_err(|error| error.to_string())?;
        let grant = SitedStationGrant::issue_with(root, issuer, wallet_public_key, request, |spec| {
            castellan::custody::wallet::issue_remote_auth_device_grant(root, spec)
        })
        .map_err(|error| error.to_string())?;
        serde_json::to_value(grant.signed()).map_err(|error| error.to_string())
    }

    pub fn revoke_device(&self, device_id: &str) -> Result<serde_json::Value, String> {
        self.unlocked().map_err(|r| r.0.to_string())?;
        let root = self.wallet().map_err(|r| r.0.to_string())?;
        let id = uuid::Uuid::parse_str(device_id).map_err(|_| "invalid device id")?;
        let outcome = castellan::custody::wallet::revoke_remote_auth_device(root, pandect::DeviceId::from_uuid(id)).map_err(|error| error.to_string())?;
        serde_json::to_value(outcome).map_err(|error| error.to_string())
    }

    /// Seal or unseal one retained payload under the open persona's wallet
    /// epoch, bound to `scope`. Plaintext returned to the caller is then in
    /// its trust boundary (§5.3).
    #[allow(clippy::too_many_arguments)]
    pub fn payload(&self, profile: &str, persona: &str, master: &[u8], scope: &[u8], content_hash: &str, operation: &PayloadOperation) -> Result<(Vec<u8>, Option<serde_json::Value>), Refusal> {
        use eidetic::PayloadSealer;
        let host = self.unlocked()?;
        let root = self.wallet()?;
        let persona = persona_id(persona)?;
        let master: [u8; 32] = master.try_into().map_err(|_| Refusal("operation-or-scope"))?;
        let hash: eidetic::Hash = serde_json::from_value(serde_json::Value::String(content_hash.to_string())).map_err(|_| Refusal("operation-or-scope"))?;
        host.with_current_profile_authority(&ProfileId(profile.into()), &master, || match operation {
            PayloadOperation::Seal { bytes } => {
                let sealer = castellan::custody::wallet::epoch_sealer_for_persona(root, persona)
                    .map_err(storage)?
                    .ok_or(Refusal("locked"))?
                    .with_scope(scope.to_vec());
                let (sealed, marker) = sealer.seal(&hash, &bytes.0).map_err(storage)?;
                Ok((sealed, Some(serde_json::to_value(marker).map_err(storage)?)))
            },
            PayloadOperation::Unseal { bytes, marker } => {
                let marker: eidetic::SealedBlobRef = serde_json::from_value(marker.clone()).map_err(|_| Refusal("operation-or-scope"))?;
                let clear = castellan::custody::wallet::unseal_persona_payload(root, persona, &hash, scope, &marker, &bytes.0)
                    .map_err(storage)?
                    .ok_or(Refusal("locked"))?;
                if eidetic::Hash::of(&clear) != hash {
                    return Err(Refusal("payload-hash-differs"));
                }
                Ok((clear, None))
            },
        })
        .map_err(|_| Refusal("profile-changed"))?
    }

    /// The credential catalog's operations, as Djinn's keeper ran them.
    pub fn credentials(&self, operation: &CredentialOperation) -> Result<serde_json::Value, String> {
        crate::credentials::run(self, operation)
    }

    pub(crate) fn credential_stores(&self) -> &BTreeMap<ProfileId, CastellanResident> {
        &self.credentials
    }

    pub(crate) fn host(&self) -> Result<&Arc<Host>, Refusal> {
        self.unlocked()
    }

    /// Pending SSH signing requests for the consent channel.
    pub fn pending_signing(&self) -> Vec<personae::signing::PendingSigningRequest> {
        match &self.host {
            Some(host) => host.snapshot().map(|s| s.pending_signing).unwrap_or_default(),
            None => Vec::new(),
        }
    }

    /// Resolve one pending SSH request after the consent channel decided.
    pub fn decide_signing(&self, request: uuid::Uuid, accept: bool) -> Result<(), Refusal> {
        let host = self.host.as_ref().ok_or(Refusal("locked"))?;
        match accept {
            true => host.approve_once(request),
            false => host.deny(request),
        }
        .map_err(|_| Refusal("signing-request-not-pending"))
    }

    /// A fresh SSH agent session over the custodian's vault.
    pub fn agent_session(&self) -> Option<castellan::custody::agent::VaultAgent<Box<dyn IdentityStorage>>> {
        self.host.as_ref().map(|host| host.agent_session())
    }
}

#[derive(serde::Deserialize)]
struct StationGrantWire {
    device_id: uuid::Uuid,
    station_ed25519_public_key: [u8; 32],
    label: String,
    issued_at_ms: u64,
    expires_at_ms: u64,
}

/// The open persona, as an identity provider, without exposing its master.
struct HostProvider(Arc<Host>);

impl IdentityProvider for HostProvider {
    fn master_public_key(&self) -> personae::Ed25519PublicKey {
        self.0.master_public_key()
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, personae::IdentityError> {
        self.0.derive_keypair(salt)
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<insigne::DerivedKeyAttestation, personae::IdentityError> {
        self.0.attest_derived_key(salt)
    }
}

/// The open persona's master signature, made under the host's own lock.
fn root_sign(host: &Host, message: &[u8]) -> Result<Signed, Refusal> {
    let signed = host.sign_with_master(message).map_err(identity)?;
    Ok(Signed {
        public_key: signed.0,
        signature: signed.1,
        attestation: None,
    })
}

fn persona_id(text: &str) -> Result<PersonaId, Refusal> {
    uuid::Uuid::parse_str(text).map(PersonaId::from_uuid).map_err(|_| Refusal("operation-or-scope"))
}

/// This machine's public device key; a label mints the identity when none.
fn device_root(root: &Path, label: Option<&str>) -> Result<[u8; 32], Refusal> {
    let identity = match label {
        Some(label) => castellan::custody::wallet::ensure_local_device_identity(root, label).map(Some).map_err(storage)?,
        None => castellan::custody::wallet::load_local_device_identity(root).map_err(storage)?,
    };
    match identity {
        Some(identity) => Ok(Ed25519Keypair::from_seed(identity.device_seed).public_key().to_bytes()),
        None if pandect::wallet_store::local_device_identity_path(root).is_file() => Err(Refusal("locked")),
        None => Err(Refusal("no-device-identity")),
    }
}

/// Claim every profile's credential store under its own derived keys and
/// have each follow the host's lock, as Djinn did at start.
fn claim_credentials(host: &Arc<Host>, data_root: &Path, primary: &ProfileId) -> Result<BTreeMap<ProfileId, CastellanResident>, Refusal> {
    let mut stores = BTreeMap::new();
    let snapshot = host.snapshot().map_err(storage)?;
    for profile in snapshot.profiles {
        let id = ProfileId(profile.id);
        let root = credential_profile_root(data_root, &id);
        if &id != primary && !root.join("records").is_dir() {
            continue;
        }
        let store = if &id == primary {
            CastellanResident::claim_derived(root.join("records"), root.join("freshness"), &HostProvider(Arc::clone(host)), CREDENTIAL_SALTS)
        } else {
            let provider = host.profile_provider(&id).map_err(identity)?;
            CastellanResident::claim_derived(root.join("records"), root.join("freshness"), &provider, CREDENTIAL_SALTS)
        }
        .map_err(storage)?;
        host.register_lock_holder(match &id == primary {
            true => store.lock_holder(CREDENTIAL_SALTS),
            false => store.lock_holder_for_profile(CREDENTIAL_SALTS, id.clone()),
        });
        stores.insert(id, store);
    }
    Ok(stores)
}

/// Djinn's credential layout: `castellan/profiles/<sanitized id>`.
pub(crate) fn credential_profile_root(data_root: &Path, profile: &ProfileId) -> PathBuf {
    let segment: String = profile
        .0
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || matches!(c, '-' | '_') { c } else { '_' })
        .collect();
    data_root.join("castellan").join("profiles").join(segment)
}

fn parts(attestation: &insigne::DerivedKeyAttestation) -> AttestationParts {
    AttestationParts {
        master: *attestation.master(),
        derived: *attestation.derived(),
        signature: Bytes(attestation.signature().to_vec()),
    }
}
