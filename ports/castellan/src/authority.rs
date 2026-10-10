// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Castellan's authority half: the resident Personae host.
//!
//! Custody without ownership. The vault and its truth are `personae`'s; this
//! is the keeper that holds them for the resident, serves the SSH agent,
//! brokers signing approvals, and applies the typed intents the projection
//! offers. Applications (graphshell first) compose it; apps talk to a pipe
//! and never see the key.
//!
//! This host deliberately starts in `StandaloneRetained`: H4 may exercise the
//! shared vault and approval boundary without stealing the user's standard SSH
//! agent endpoint before restart and real-login proofs exist.
//!
//! ## The lock (vault lock plan, rulings 1, 9 to 14, 27, 31)
//!
//! [`PersonaeHost::lock_vault`] locks the vault, then runs every registered
//! [`VaultLockHolder`] before it returns; observers watch
//! [`PersonaeHost::lock_state`]. The agent's `ssh-add -x` engages the same
//! lock. While locked, every secret-reaching surface refuses with
//! [`IdentityIntentError::Locked`], and the snapshot shows the public view kept
//! at lock time. Unlocking takes [`PersonaeHost::unlock_vault`], a native call:
//! no intent carries the credential.

use std::io;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use insigne::DerivedKeyAttestation;
use pandect::{DeviceId, PersonaId, WalletEpochSealer};
use personae::signing::{DecisionError, RememberApproval, SigningDecision};
use personae::{
    CredentialLineage, Ed25519Keypair, Ed25519PublicKey, IdentityError, IdentityProvider,
    ProfileId, ProtocolKey, RetainedKeys, UnlockTier,
};

use crate::custody::agent::{VaultAgent, VaultLockRequest};
use crate::custody::wallet::{epoch_sealer_for_persona, revoke_remote_auth_device};
use crate::custody::{
    ApprovalBroker, IdentityStorage, IdentityVault, UnlockMethod, roster, ssh_slot,
};
use ssh_key::{Algorithm, PrivateKey, PublicKey};
use tokio::sync::watch;
use uuid::Uuid;

use crate::lock::VaultLockHolder;

use crate::projection::{
    CreateProfileIntentV1, DEVICE_REVOKE_INTENT, GenerateSshKeyIntentV1,
    ImportSshKeyNativeIntentV1, PROFILE_CREATE_INTENT, PROFILE_SWITCH_INTENT, RemoveSshKeyIntentV1,
    RevokeDeviceIntentV1, LockVaultIntentV1, VAULT_LOCK_INTENT, VAULT_UNLOCK_INTENT,
    SIGNING_APPROVE_IDLE_INTENT, SIGNING_APPROVE_ONCE_INTENT,
    SIGNING_DENY_INTENT, SSH_GENERATE_INTENT, SSH_IMPORT_NATIVE_INTENT, SSH_REMOVE_INTENT,
    SigningDecisionIntentV1, SshUnlockPolicyIntentV1, SwitchProfileIntentV1,
};
use crate::view::{
    AgentListenerView, CarryView, IdentitySurfaceSnapshot, ProfileView, SshKeyView, VaultLockView,
    VaultProtectionView, VaultView, load_carry_view,
};

const MAX_SHORT_TTL_SECONDS: u32 = 24 * 60 * 60;
#[cfg(windows)]
pub const STANDARD_WINDOWS_AGENT_ENDPOINT: &str = r"\\.\pipe\openssh-ssh-agent";

/// Whether `endpoint` is this user's `SSH_AUTH_SOCK`.
#[cfg(not(windows))]
fn is_standard_unix_agent(endpoint: &str) -> bool {
    same_socket(endpoint, std::env::var_os("SSH_AUTH_SOCK").as_deref())
}

/// The same path, or the same file once links resolve.
#[cfg(any(not(windows), test))]
fn same_socket(endpoint: &str, standard: Option<&std::ffi::OsStr>) -> bool {
    let Some(standard) = standard.filter(|s| !s.is_empty()) else {
        return false;
    };
    let (endpoint, standard) = (
        std::path::Path::new(endpoint),
        std::path::Path::new(standard),
    );
    endpoint == standard
        || matches!(
            (endpoint.canonicalize(), standard.canonicalize()),
            (Ok(a), Ok(b)) if a == b
        )
}

/// Rejected Graphshell identity action.
#[derive(Debug, thiserror::Error)]
pub enum IdentityIntentError {
    #[error("unknown identity intent")]
    UnknownIntent,
    #[error("invalid signing decision payload: {0}")]
    InvalidPayload(#[from] serde_json::Error),
    #[error("signing decision rejected: {0}")]
    Decision(#[from] DecisionError),
    #[error("identity vault operation failed: {0}")]
    Identity(IdentityError),
    /// Refused while the vault is locked (ruling 10).
    #[error("the vault is locked")]
    Locked,
    /// Unlock is native: no intent carries the credential (ruling 9).
    #[error("unlocking takes the resident's own surface; no intent carries the credential")]
    UnlockNativeOnly,
    #[error("SSH key generation failed")]
    KeyGeneration,
    #[error("SSH public key encoding failed")]
    PublicEncoding,
    #[error("SSH key comment must be at most 256 printable characters")]
    InvalidComment,
    #[error(
        "a persona id must be 1-64 characters of letters, digits, hyphen or underscore, so it \
         survives becoming a filename unchanged"
    )]
    InvalidProfileId,
    #[error("a persona name must be 1-256 printable characters")]
    InvalidProfileName,
    #[error("short idle approval must be between 1 and 86400 seconds")]
    InvalidIdleWindow,
    #[error("SSH key removal requires explicit confirmation")]
    ConfirmationRequired,
    #[error("SSH key fingerprint is not present in the selected profile")]
    KeyNotFound,
    #[error("SSH import requires a native private-key handoff")]
    NativeHandoffRequired,
    #[error("device revocation requires explicit confirmation")]
    DeviceRevocationConfirmationRequired,
    #[error("carry authority is not configured")]
    CarryUnavailable,
    #[error("device revocation failed ({0:?})")]
    DeviceRevocation(std::io::ErrorKind),
    /// Refused at import, with personae's reason (ruling 56).
    #[error("the agent cannot sign this SSH key: {0}")]
    UnsignableKey(String),
}

impl From<IdentityError> for IdentityIntentError {
    fn from(error: IdentityError) -> Self {
        match error {
            IdentityError::Locked => Self::Locked,
            error => Self::Identity(error),
        }
    }
}

// The intent receipts, at their old paths (dramatis repo plan, ruling D31).
pub use dramatis::receipts::*;

/// Native owner of one shared Personae vault and its SSH adapter.
pub struct PersonaeHost<S: IdentityStorage> {
    vault: Arc<Mutex<IdentityVault<S>>>,
    agent: VaultAgent<S>,
    approval: ApprovalBroker,
    data_root: Option<PathBuf>,
    /// Where the vault lives on disk, when it lives anywhere. A profile
    /// switch writes the family's remembered choice beside it
    /// ([`personae::roster::remember_profile`]); an ephemeral host has no
    /// beside, so `None` switches without remembering.
    vault_dir: Option<PathBuf>,
    protection: VaultProtectionView,
    lock: Arc<ResidentLock<S>>,
    listener: Arc<Mutex<AgentListenerView>>,
    /// Keys the lock leaves in place, one set per purpose (rulings 40, 46).
    retained: Mutex<Vec<Arc<RetainedKeys>>>,
}

/// What stays visible while locked (ruling 11): the profile and SSH key
/// views, kept when the lock is taken. Secret-free by their types.
#[derive(Clone)]
struct KeptView {
    profiles: Vec<ProfileView>,
    ssh_keys: Vec<SshKeyView>,
}

/// The resident's lock: the shared vault, the holders that obey it, the
/// state observers watch, and the view kept at lock time.
struct ResidentLock<S: IdentityStorage> {
    vault: Arc<Mutex<IdentityVault<S>>>,
    holders: Mutex<Vec<Arc<dyn VaultLockHolder>>>,
    state: watch::Sender<VaultLockView>,
    kept: Mutex<Option<KeptView>>,
    /// The vault directory whose persisted lock this resident keeps (rulings
    /// 5, 80), when it keeps one.
    persisted: Mutex<Option<PathBuf>>,
}

impl<S: IdentityStorage + 'static> ResidentLock<S> {
    /// The vault first, so a refusal (ruling 27) changes nothing; then
    /// every holder, before anyone is told.
    fn lock(&self) -> Result<(), IdentityError> {
        let mut vault = self.vault.lock().unwrap();
        if vault.is_locked() {
            return Ok(());
        }
        let kept = public_views(&vault).ok();
        vault.lock()?;
        let holders = self.holders.lock().unwrap().clone();
        for holder in &holders {
            holder.lock();
        }
        *self.kept.lock().unwrap() = kept;
        drop(vault);
        // Every lock, the agent's `ssh-add -x` included, persists (ruling 5).
        if let Some(dir) = self.persisted.lock().unwrap().as_deref() {
            if let Err(error) = crate::custody::persist_lock(dir) {
                tracing::error!(%error, "the lock holds, but a restart could reopen the vault");
            }
        }
        self.state.send_replace(VaultLockView::Locked);
        tracing::info!(holders = holders.len(), "vault locked");
        Ok(())
    }

    /// The vault, then every holder re-derives; a holder failing relocks
    /// everything, so nothing is left half unlocked.
    fn unlock(&self, method: UnlockMethod<'_>) -> Result<(), IdentityError> {
        let mut vault = self.vault.lock().unwrap();
        if !vault.is_locked() {
            return Ok(());
        }
        vault.unlock(method)?;
        let holders = self.holders.lock().unwrap().clone();
        for holder in &holders {
            if let Err(error) = holder.unlock(&*vault) {
                tracing::error!(holder = holder.name(), %error, "holder could not re-derive; relocking");
                for holder in &holders {
                    holder.lock();
                }
                if let Err(relock) = vault.lock() {
                    tracing::error!(%relock, "the vault could not relock");
                }
                return Err(error);
            }
        }
        *self.kept.lock().unwrap() = None;
        drop(vault);
        if let Some(dir) = self.persisted.lock().unwrap().as_deref() {
            if let Err(error) = crate::custody::clear_persisted_lock(dir) {
                tracing::error!(%error, "unlocked, but the next start will still ask");
            }
        }
        self.state.send_replace(VaultLockView::Unlocked);
        tracing::info!(holders = holders.len(), "vault unlocked");
        Ok(())
    }

    /// A holder joining a locked vault drops its keys at once.
    fn register(&self, holder: Arc<dyn VaultLockHolder>) {
        let vault = self.vault.lock().unwrap();
        if vault.is_locked() {
            holder.lock();
        }
        self.holders.lock().unwrap().push(holder);
    }
}

impl<S: IdentityStorage + 'static> VaultLockRequest for ResidentLock<S> {
    fn lock_vault(&self) -> Result<(), IdentityError> {
        self.lock()
    }
}

/// The secret-free profile and SSH key views of an unlocked vault.
fn public_views<S: IdentityStorage>(vault: &IdentityVault<S>) -> io::Result<KeptView> {
    let profile = vault.current_profile().map_err(io::Error::other)?;
    let current_id = profile.id.0.clone();
    let mut profiles: Vec<_> = vault
        .storage()
        .list_profiles()
        .unwrap_or_default()
        .into_iter()
        .map(|summary| ProfileView {
            selected: summary.id == profile.id,
            id: summary.id.0,
            display_name: summary.display_name,
            slot_count: summary.slot_count,
            master_public_fingerprint: "unknown until profile is selected".to_string(),
        })
        .collect();

    let master_fingerprint = format!(
        "blake3:{}",
        blake3::hash(&profile.master.public_key().to_bytes()).to_hex()
    );
    if let Some(current) = profiles.iter_mut().find(|entry| entry.selected) {
        current.master_public_fingerprint = master_fingerprint.clone();
    } else {
        profiles.push(ProfileView {
            id: current_id.clone(),
            display_name: profile.display_name.clone(),
            selected: true,
            slot_count: profile.slots.len(),
            master_public_fingerprint: master_fingerprint,
        });
    }
    profiles.sort_by(|left, right| left.id.cmp(&right.id));

    let mut ssh_keys = Vec::new();
    for slot in ssh_slot::ssh_slots(profile) {
        let source = profile
            .slots
            .get(&slot.key)
            .expect("ssh_slots only returns profile-owned slots");
        let lineage = source.lineage();
        ssh_keys.push(SshKeyView {
            profile: current_id.clone(),
            fingerprint: slot.fingerprint(),
            comment: slot.private.comment().to_string(),
            public_openssh: slot
                .public()
                .to_openssh()
                .unwrap_or_else(|_| "public key encoding unavailable".to_string()),
            lineage: lineage_label(lineage).to_string(),
            device_loss_note: lineage.device_loss_note().to_string(),
            unlock_policy: unlock_label(source.unlock_tier()),
        });
    }
    ssh_keys.sort_by(|left, right| left.fingerprint.cmp(&right.fingerprint));
    Ok(KeptView { profiles, ssh_keys })
}

/// A locked vault with no view kept (never unlocked here): the vault's own
/// public profile, and no keys.
fn bare_view<S: IdentityStorage>(vault: &IdentityVault<S>) -> KeptView {
    let public = vault.public_profile();
    KeptView {
        profiles: vec![ProfileView {
            id: public.id.0.clone(),
            display_name: public.display_name.clone(),
            selected: true,
            slot_count: public.slots.len(),
            master_public_fingerprint: format!(
                "blake3:{}",
                blake3::hash(&public.master_public_key.to_bytes()).to_hex()
            ),
        }],
        ssh_keys: Vec::new(),
    }
}

impl<S: IdentityStorage + 'static> PersonaeHost<S> {
    /// Compose a resident host without taking over the standard agent endpoint.
    pub fn new(
        vault: IdentityVault<S>,
        data_root: Option<PathBuf>,
        protection: VaultProtectionView,
    ) -> Self {
        Self::with_decision_timeout(vault, data_root, protection, Duration::from_secs(120))
    }

    /// Constructor with a bounded timeout for tests and configured hosts.
    pub fn with_decision_timeout(
        vault: IdentityVault<S>,
        data_root: Option<PathBuf>,
        protection: VaultProtectionView,
        decision_timeout: Duration,
    ) -> Self {
        let initial = match vault.is_locked() {
            true => VaultLockView::Locked,
            false => VaultLockView::Unlocked,
        };
        let vault = Arc::new(Mutex::new(vault));
        let lock = Arc::new(ResidentLock {
            vault: Arc::clone(&vault),
            holders: Mutex::new(Vec::new()),
            state: watch::channel(initial).0,
            kept: Mutex::new(None),
            persisted: Mutex::new(None),
        });
        let approval = ApprovalBroker::new(decision_timeout);
        let agent =
            VaultAgent::from_shared_vault(Arc::clone(&vault), approval.clone(), "graphshell.ssh")
                .with_vault_lock(Arc::clone(&lock) as Arc<dyn VaultLockRequest>);
        Self {
            vault,
            agent,
            approval,
            data_root,
            protection,
            lock,
            listener: Arc::new(Mutex::new(AgentListenerView::StandaloneRetained)),
            retained: Mutex::new(Vec::new()),
            vault_dir: None,
        }
    }

    /// Name where the vault lives, so a profile switch also remembers the
    /// choice for the rest of the family.
    pub fn with_vault_dir(mut self, dir: PathBuf) -> Self {
        self.vault_dir = Some(dir);
        self
    }

    /// Keep the persisted lock in `dir`: every lock writes its marker and
    /// every unlock clears it, so a restart under a lock waits for a user act
    /// (rulings 5, 76, 80). Only the resident sets this; a host without it
    /// leaves nothing on disk.
    pub fn with_persisted_lock(self, dir: PathBuf) -> Self {
        *self.lock.persisted.lock().unwrap() = Some(dir);
        self
    }

    /// Lock the vault and every registered holder (ruling 1): when this
    /// returns, no holder keeps a vault-derived key. Refused while no unlock
    /// method is available (ruling 27). Any surface may ask.
    pub fn lock_vault(&self) -> Result<(), IdentityIntentError> {
        Ok(self.lock.lock()?)
    }

    /// Unlock by a user act on this resident's own surface; never reachable
    /// through an intent or the agent protocol (ruling 9). A wrong credential
    /// leaves everything locked.
    pub fn unlock_vault(&self, method: UnlockMethod<'_>) -> Result<(), IdentityIntentError> {
        Ok(self.lock.unlock(method)?)
    }

    /// Whether the vault is locked.
    pub fn is_locked(&self) -> bool {
        self.vault.lock().unwrap().is_locked()
    }

    /// The lock state, for observers (ruling 31): a UI, the status route.
    pub fn lock_state(&self) -> watch::Receiver<VaultLockView> {
        self.lock.state.subscribe()
    }

    /// Have `holder` drop its keys with every lock and re-derive them with
    /// every unlock. Registered while locked, it locks at once.
    pub fn register_lock_holder(&self, holder: Arc<dyn VaultLockHolder>) {
        self.lock.register(holder);
    }

    /// A restricted provider for exactly `salts` that the lock leaves in
    /// place (rulings 40, 46): captured from the vault the first time it is
    /// asked for while unlocked, then served as kept, locked or not. It
    /// derives nothing else. Locked with nothing kept, it is `Locked`.
    pub fn retained_keys(&self, salts: &[Vec<u8>]) -> Result<Arc<RetainedKeys>, IdentityError> {
        let vault = self.vault.lock().unwrap();
        let master = IdentityProvider::master_public_key(&*vault);
        let mut retained = self.retained.lock().unwrap();
        if let Some(kept) = retained
            .iter()
            .find(|kept| kept.master_public_key() == master && kept.holds(salts))
        {
            return Ok(Arc::clone(kept));
        }
        let captured = Arc::new(RetainedKeys::capture(&*vault, salts)?);
        retained.retain(|kept| kept.master_public_key() == master);
        retained.push(Arc::clone(&captured));
        Ok(captured)
    }

    fn ensure_unlocked(&self) -> Result<(), IdentityIntentError> {
        match self.is_locked() {
            true => Err(IdentityIntentError::Locked),
            false => Ok(()),
        }
    }

    /// A fresh per-connection SSH agent session over the resident vault.
    pub fn agent_session(&self) -> VaultAgent<S> {
        self.agent.clone()
    }

    /// Bind an isolated Windows named pipe for acceptance testing.
    ///
    /// The standard OpenSSH endpoint is rejected here. Taking it over requires
    /// a separate cutover path with restart and real-login receipts.
    #[cfg(windows)]
    pub fn bind_receipt_listener(
        &self,
        endpoint: &str,
    ) -> std::io::Result<ssh_agent_lib::agent::NamedPipeListener> {
        if endpoint.eq_ignore_ascii_case(STANDARD_WINDOWS_AGENT_ENDPOINT) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "receipt listener cannot bind the standard SSH agent endpoint",
            ));
        }
        let listener = ssh_agent_lib::agent::NamedPipeListener::bind(endpoint)?;
        *self.listener.lock().unwrap() = AgentListenerView::ReceiptEndpoint {
            endpoint: endpoint.to_string(),
        };
        Ok(listener)
    }

    /// Bind an isolated Unix socket for acceptance testing.
    ///
    /// The user's own `SSH_AUTH_SOCK` is refused, as the OpenSSH pipe is on
    /// Windows: it belongs to the agent the user already runs.
    #[cfg(not(windows))]
    pub fn bind_receipt_listener(
        &self,
        endpoint: &str,
    ) -> std::io::Result<tokio::net::UnixListener> {
        if endpoint.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the receipt SSH agent socket path is empty",
            ));
        }
        if is_standard_unix_agent(endpoint) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "receipt listener cannot bind the standard SSH agent endpoint",
            ));
        }
        let listener = tokio::net::UnixListener::bind(endpoint)?;
        *self.listener.lock().unwrap() = AgentListenerView::ReceiptEndpoint {
            endpoint: endpoint.to_string(),
        };
        Ok(listener)
    }

    /// Bind the standard Windows OpenSSH endpoint for the resident host.
    ///
    /// This is deliberately distinct from [`Self::bind_receipt_listener`]:
    /// only a lifecycle-managed device host should call it.
    #[cfg(windows)]
    pub fn bind_standard_listener(
        &self,
        endpoint: &str,
    ) -> std::io::Result<ssh_agent_lib::agent::NamedPipeListener> {
        if !endpoint.eq_ignore_ascii_case(STANDARD_WINDOWS_AGENT_ENDPOINT) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the Windows standard listener must use the OpenSSH agent pipe",
            ));
        }
        let listener = ssh_agent_lib::agent::NamedPipeListener::bind(endpoint)?;
        *self.listener.lock().unwrap() = AgentListenerView::StandardEndpoint {
            endpoint: endpoint.to_string(),
        };
        Ok(listener)
    }

    /// Bind the configured Unix `SSH_AUTH_SOCK` for the resident host.
    #[cfg(not(windows))]
    pub fn bind_standard_listener(
        &self,
        endpoint: &str,
    ) -> std::io::Result<tokio::net::UnixListener> {
        if endpoint.trim().is_empty() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the standard SSH agent socket path is empty",
            ));
        }
        let listener = tokio::net::UnixListener::bind(endpoint)?;
        *self.listener.lock().unwrap() = AgentListenerView::StandardEndpoint {
            endpoint: endpoint.to_string(),
        };
        Ok(listener)
    }

    /// Generate an Ed25519 key entirely inside the resident authority.
    pub fn generate_ssh_key(
        &self,
        request: GenerateSshKeyIntentV1,
    ) -> Result<SshKeyMutationReceipt, IdentityIntentError> {
        validate_comment(&request.comment)?;
        let tier = unlock_tier(request.unlock_policy)?;
        let mut private = PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519)
            .map_err(|_| IdentityIntentError::KeyGeneration)?;
        private.set_comment(&request.comment);
        self.store_ssh_private(private, tier, SshKeyMutationKind::Generated)
    }

    /// Store a key received directly from a native file picker.
    ///
    /// Callers pass the parsed key object, not serialized key bytes. This API
    /// is intentionally separate from `apply_intent`.
    pub fn import_ssh_private(
        &self,
        private: PrivateKey,
        options: ImportSshKeyNativeIntentV1,
    ) -> Result<SshKeyMutationReceipt, IdentityIntentError> {
        validate_comment(private.comment())?;
        let tier = unlock_tier(options.unlock_policy)?;
        self.store_ssh_private(private, tier, SshKeyMutationKind::Imported)
    }

    fn store_ssh_private(
        &self,
        private: PrivateKey,
        tier: UnlockTier,
        operation: SshKeyMutationKind,
    ) -> Result<SshKeyMutationReceipt, IdentityIntentError> {
        let key = ssh_slot::protocol_key_for(&private);
        let public = PublicKey::from(&private);
        let fingerprint = public.fingerprint(ssh_key::HashAlg::Sha256).to_string();
        let public_openssh = public
            .to_openssh()
            .map_err(|_| IdentityIntentError::PublicEncoding)?;
        let mut vault = self.vault.lock().unwrap();
        // Ruling 54: a held key is never rewritten; the receipt describes the
        // slot as held, comment and tier included.
        if let Some(held) = vault.current_profile()?.slots.get(&key) {
            let held_private = ssh_slot::private_key_from_slot(held)?;
            return Ok(SshKeyMutationReceipt {
                operation,
                fingerprint,
                comment: held_private.comment().to_string(),
                public_openssh,
                unlock_policy: unlock_label(held.unlock_tier()),
                replaced_existing: true,
            });
        }
        // Ruling 56: only keys the agent can sign are taken in.
        crate::custody::ssh_sign::check_signable(private.key_data())
            .map_err(|refused| IdentityIntentError::UnsignableKey(refused.to_string()))?;
        let slot = ssh_slot::slot_for(&private, tier)?;
        let comment = private.comment().to_string();
        vault.add_slot(key, slot)?;
        Ok(SshKeyMutationReceipt {
            operation,
            fingerprint,
            comment,
            public_openssh,
            unlock_policy: unlock_label(tier),
            replaced_existing: false,
        })
    }

    /// Remove one SSH slot only after the local UI confirms its fingerprint.
    pub fn remove_ssh_key(
        &self,
        request: RemoveSshKeyIntentV1,
    ) -> Result<SshKeyMutationReceipt, IdentityIntentError> {
        if !request.confirmed {
            return Err(IdentityIntentError::ConfirmationRequired);
        }
        let key = ProtocolKey::new(ssh_slot::SSH_MOD_ID, Some(request.fingerprint.clone()));
        let mut vault = self.vault.lock().unwrap();
        let Some(slot) = vault.current_profile()?.slots.get(&key) else {
            return Err(IdentityIntentError::KeyNotFound);
        };
        let private = ssh_slot::private_key_from_slot(slot)?;
        let public = PublicKey::from(&private);
        let receipt = SshKeyMutationReceipt {
            operation: SshKeyMutationKind::Removed,
            fingerprint: public.fingerprint(ssh_key::HashAlg::Sha256).to_string(),
            comment: private.comment().to_string(),
            public_openssh: public
                .to_openssh()
                .map_err(|_| IdentityIntentError::PublicEncoding)?,
            unlock_policy: unlock_label(slot.unlock_tier()),
            replaced_existing: false,
        };
        vault.remove_slot(&key)?;
        Ok(receipt)
    }

    /// The private-lane payload sealer for one persona.
    ///
    /// Eidetic sits below the wallet and owns no keys, so its seal seam stays
    /// inert until a host supplies a `PayloadSealer`; its own header says
    /// landing the seam "changes no runtime behavior" for exactly that reason.
    /// This is the supply point. The keeper already holds the carry root, which
    /// is the only thing beyond the persona that building one needs, so no
    /// other component has to learn where the wallet lives.
    ///
    /// `None` twice over, and deliberately not an error either time: a host
    /// with no carry root has no wallet to seal under, and a persona with no
    /// staged epoch has nothing to seal with. Eidetic's own posture is that "a
    /// keyless host stays in the cleartext lane, which is a host policy
    /// decision, not this seam's to enforce", so the decision is returned to
    /// the caller rather than taken here. A caller that requires sealing
    /// refuses on `None`; one that does not writes cleartext, which is what
    /// every caller does today.
    pub fn payload_sealer(&self, persona: PersonaId) -> io::Result<Option<WalletEpochSealer>> {
        if self.is_locked() {
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "the vault is locked",
            ));
        }
        let Some(data_root) = self.data_root.as_deref() else {
            return Ok(None);
        };
        epoch_sealer_for_persona(data_root, persona)
    }

    /// Revoke one delegated device through pandect's live authority.
    pub fn revoke_device(
        &self,
        request: RevokeDeviceIntentV1,
    ) -> Result<DeviceRevocationReceipt, IdentityIntentError> {
        if !request.confirmed {
            return Err(IdentityIntentError::DeviceRevocationConfirmationRequired);
        }
        self.ensure_unlocked()?;
        let data_root = self
            .data_root
            .as_deref()
            .ok_or(IdentityIntentError::CarryUnavailable)?;
        let outcome = revoke_remote_auth_device(data_root, DeviceId::from_uuid(request.device_id))
            .map_err(|error| IdentityIntentError::DeviceRevocation(error.kind()))?;
        Ok(DeviceRevocationReceipt {
            device_id: outcome.device_id.as_uuid().to_string(),
            already_revoked: outcome.already_revoked,
            rotated_personas: outcome
                .rotated_personas
                .into_iter()
                .map(|persona| persona.as_uuid().to_string())
                .collect(),
            refreshed_devices: outcome
                .refreshed_devices
                .into_iter()
                .map(|device| device.as_uuid().to_string())
                .collect(),
        })
    }

    /// Secret-free Graphshell read model. While locked it reports Locked
    /// with the public view kept at lock time (ruling 11).
    pub fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        let vault = self.vault.lock().unwrap();
        let (lock, view) = match vault.is_locked() {
            true => (
                VaultLockView::Locked,
                self.lock
                    .kept
                    .lock()
                    .unwrap()
                    .clone()
                    .unwrap_or_else(|| bare_view(&vault)),
            ),
            false => (VaultLockView::Unlocked, public_views(&vault)?),
        };
        drop(vault);

        let carry = match &self.data_root {
            Some(data_root) => load_carry_view(data_root)?,
            None => CarryView {
                unavailable: vec!["carry data root is not configured".to_string()],
                ..CarryView::default()
            },
        };

        Ok(IdentitySurfaceSnapshot {
            vault: VaultView {
                protection: self.protection,
                lock,
                agent: self.listener.lock().unwrap().clone(),
            },
            profiles: view.profiles,
            ssh_keys: view.ssh_keys,
            carry,
            pending_signing: self.approval.pending(),
            signing_history: self.approval.history(),
        })
    }

    /// Approve exactly one pending operation.
    pub fn approve_once(&self, request_id: Uuid) -> Result<(), DecisionError> {
        self.approval.decide(
            request_id,
            SigningDecision::Approve {
                remember: RememberApproval::Once,
            },
        )
    }

    /// Approve under the key's configured short idle window.
    pub fn approve_until_idle(&self, request_id: Uuid) -> Result<(), DecisionError> {
        self.approval.decide(
            request_id,
            SigningDecision::Approve {
                remember: RememberApproval::UntilIdle,
            },
        )
    }

    /// Deny one pending operation.
    pub fn deny(&self, request_id: Uuid) -> Result<(), DecisionError> {
        self.approval.decide(request_id, SigningDecision::Deny)
    }

    /// Switch the resident vault to another persona, live.
    ///
    /// Everything sharing the vault follows from its next operation — the SSH
    /// agent holds the same `Arc<Mutex<IdentityVault>>`, so the next signing
    /// request is served from the new persona's slots with no restart
    /// anywhere. The choice is then remembered beside the vault so the rest
    /// of the family opens on it too; a host with no on-disk vault switches
    /// without remembering, and the receipt says which happened.
    pub fn switch_profile(
        &self,
        payload: SwitchProfileIntentV1,
    ) -> Result<ProfileSwitchReceipt, IdentityIntentError> {
        let id = ProfileId(payload.profile);
        self.vault.lock().unwrap().switch_profile(&id)?;
        // The kept sets follow the persona now speaking.
        let salts: Vec<Vec<Vec<u8>>> = self
            .retained
            .lock()
            .unwrap()
            .iter()
            .map(|kept| kept.salts())
            .collect();
        for set in salts {
            if let Err(error) = self.retained_keys(&set) {
                tracing::warn!(%error, "kept keys not recaptured after the switch");
            }
        }
        let remembered = match &self.vault_dir {
            Some(dir) => match roster::remember_profile(dir, &id) {
                Ok(()) => true,
                // The switch itself succeeded; a choice that could not be
                // written is worth saying, not worth unwinding.
                Err(error) => {
                    tracing::warn!(%error, profile = %id.0, "switched without remembering");
                    false
                },
            },
            None => false,
        };
        Ok(ProfileSwitchReceipt {
            profile: id.0,
            remembered,
        })
    }

    /// Mint a persona in the vault.
    ///
    /// **Does not switch to it.** Creating an identity and becoming it are
    /// separate decisions: a persona minted for another device or another
    /// purpose is not one the user is necessarily adopting, and the new card
    /// carries the ordinary switch action for when they are.
    ///
    /// The id is constrained rather than sanitized. It reaches a filename in
    /// `owner_settings::settings_path`, which replaces anything unsafe with
    /// `_`; accepting `a/b` here would mean it and `a_b` silently share one
    /// settings file. Refusing at the point of creation is the only place that
    /// cannot be worked around later.
    pub fn create_profile(
        &self,
        payload: CreateProfileIntentV1,
    ) -> Result<ProfileCreatedReceipt, IdentityIntentError> {
        let id = payload.id.trim();
        if id.is_empty()
            || id.chars().count() > 64
            || !id
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(IdentityIntentError::InvalidProfileId);
        }
        let display_name = payload.display_name.trim();
        if display_name.is_empty()
            || display_name.chars().count() > 256
            || display_name.chars().any(char::is_control)
        {
            return Err(IdentityIntentError::InvalidProfileName);
        }
        // `roster::create_profile` refuses a taken id, which is the guard that
        // matters: minting over a persona would replace its master key and
        // every certificate rooted on it.
        let vault = self.vault.lock().unwrap();
        let profile =
            roster::create_profile(vault.storage(), &ProfileId(id.to_string()), display_name)?;
        Ok(ProfileCreatedReceipt {
            id: profile.id.0,
            display_name: profile.display_name,
            master_public_fingerprint: format!(
                "blake3:{}",
                blake3::hash(&profile.master.public_key().to_bytes()).to_hex()
            ),
        })
    }

    /// Apply one typed action emitted by [`crate::projection`].
    pub fn apply_intent(
        &self,
        intent: &str,
        payload: &[u8],
    ) -> Result<IdentityIntentOutcome, IdentityIntentError> {
        match intent {
            SIGNING_APPROVE_ONCE_INTENT => {
                let payload: SigningDecisionIntentV1 = serde_json::from_slice(payload)?;
                self.approve_once(payload.request_id)?;
                Ok(IdentityIntentOutcome::SigningDecision)
            },
            SIGNING_APPROVE_IDLE_INTENT => {
                let payload: SigningDecisionIntentV1 = serde_json::from_slice(payload)?;
                self.approve_until_idle(payload.request_id)?;
                Ok(IdentityIntentOutcome::SigningDecision)
            },
            SIGNING_DENY_INTENT => {
                let payload: SigningDecisionIntentV1 = serde_json::from_slice(payload)?;
                self.deny(payload.request_id)?;
                Ok(IdentityIntentOutcome::SigningDecision)
            },
            SSH_GENERATE_INTENT => {
                let payload: GenerateSshKeyIntentV1 = serde_json::from_slice(payload)?;
                self.generate_ssh_key(payload)
                    .map(IdentityIntentOutcome::SshKeyMutation)
            },
            SSH_REMOVE_INTENT => {
                let payload: RemoveSshKeyIntentV1 = serde_json::from_slice(payload)?;
                self.remove_ssh_key(payload)
                    .map(IdentityIntentOutcome::SshKeyMutation)
            },
            SSH_IMPORT_NATIVE_INTENT => {
                let _: ImportSshKeyNativeIntentV1 = serde_json::from_slice(payload)?;
                Err(IdentityIntentError::NativeHandoffRequired)
            },
            DEVICE_REVOKE_INTENT => {
                let payload: RevokeDeviceIntentV1 = serde_json::from_slice(payload)?;
                self.revoke_device(payload)
                    .map(IdentityIntentOutcome::DeviceRevocation)
            },
            PROFILE_SWITCH_INTENT => {
                let payload: SwitchProfileIntentV1 = serde_json::from_slice(payload)?;
                self.switch_profile(payload)
                    .map(IdentityIntentOutcome::ProfileSwitch)
            },
            PROFILE_CREATE_INTENT => {
                let payload: CreateProfileIntentV1 = serde_json::from_slice(payload)?;
                self.create_profile(payload)
                    .map(IdentityIntentOutcome::ProfileCreated)
            },
            VAULT_LOCK_INTENT => {
                let _: LockVaultIntentV1 = serde_json::from_slice(payload)?;
                self.lock_vault()?;
                Ok(IdentityIntentOutcome::VaultLocked)
            },
            VAULT_UNLOCK_INTENT => Err(IdentityIntentError::UnlockNativeOnly),
            _ => Err(IdentityIntentError::UnknownIntent),
        }
    }
}

impl<S: IdentityStorage + 'static> IdentityProvider for PersonaeHost<S> {
    fn master_public_key(&self) -> Ed25519PublicKey {
        IdentityProvider::master_public_key(&*self.vault.lock().unwrap())
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, IdentityError> {
        self.vault.lock().unwrap().derive_keypair(salt)
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<DerivedKeyAttestation, IdentityError> {
        self.vault.lock().unwrap().attest_derived_key(salt)
    }
}

fn validate_comment(comment: &str) -> Result<(), IdentityIntentError> {
    if comment.chars().count() > 256 || comment.chars().any(char::is_control) {
        return Err(IdentityIntentError::InvalidComment);
    }
    Ok(())
}

fn unlock_tier(policy: SshUnlockPolicyIntentV1) -> Result<UnlockTier, IdentityIntentError> {
    match policy {
        SshUnlockPolicyIntentV1::Session => Ok(UnlockTier::Session),
        SshUnlockPolicyIntentV1::ShortTtl { idle_seconds }
            if (1..=MAX_SHORT_TTL_SECONDS).contains(&idle_seconds) =>
        {
            Ok(UnlockTier::ShortTtl { idle_seconds })
        },
        SshUnlockPolicyIntentV1::ShortTtl { .. } => Err(IdentityIntentError::InvalidIdleWindow),
        SshUnlockPolicyIntentV1::PerUse => Ok(UnlockTier::PerUse),
    }
}

fn lineage_label(lineage: CredentialLineage) -> &'static str {
    match lineage {
        CredentialLineage::LocallyDerived => "locally derived",
        CredentialLineage::LocallyGeneratedExternallyRegistered => {
            "locally generated, externally registered"
        },
        CredentialLineage::ExternallyIssued => "externally issued",
        CredentialLineage::ExternallyRootedLocallyHeld => "externally rooted, locally held",
    }
}

fn unlock_label(tier: UnlockTier) -> String {
    match tier {
        UnlockTier::Session => "session".to_string(),
        UnlockTier::ShortTtl { idle_seconds } => format!("{idle_seconds}s idle"),
        UnlockTier::PerUse => "every use".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use crate::custody::ssh_slot::{protocol_key_for, slot_for};
    use crate::custody::{InMemoryStorage, Profile};
    use personae::{Ed25519Keypair, ProfileId};
    use signature::Verifier;
    use ssh_agent_lib::agent::Session;
    use ssh_agent_lib::proto::SignRequest;
    use ssh_key::{Algorithm, LineEnding};

    use super::*;

    /// The Unix receipt wall: the user's `SSH_AUTH_SOCK`, by path or by a
    /// link to it, is the standard endpoint; any other socket is not.
    #[test]
    fn the_users_agent_socket_is_the_standard_endpoint() {
        let dir = tempfile::tempdir().unwrap();
        let sock = dir.path().join("agent.sock");
        std::fs::write(&sock, b"").unwrap();
        let standard = Some(sock.as_os_str());
        assert!(same_socket(&sock.display().to_string(), standard));
        assert!(!same_socket(
            &dir.path().join("receipt.sock").display().to_string(),
            standard
        ));
        assert!(!same_socket(&sock.display().to_string(), None));
        let dotted = dir.path().join(".").join("agent.sock");
        assert!(same_socket(&dotted.display().to_string(), standard));
    }

    #[test]
    fn snapshot_discloses_public_ssh_material_but_not_the_private_slot() {
        let mut private =
            ssh_key::PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
        private.set_comment("workstation");
        let private_openssh = private.to_openssh(LineEnding::LF).unwrap().to_string();
        let mut profile = Profile::new(
            ProfileId("research".to_string()),
            "Research",
            Ed25519Keypair::from_seed([0x5a; 32]),
        );
        profile.slots.insert(
            protocol_key_for(&private),
            slot_for(&private, UnlockTier::PerUse).unwrap(),
        );
        let storage = InMemoryStorage::new();
        storage.save_profile(&profile).unwrap();
        let host = PersonaeHost::new(
            IdentityVault::with_profile(storage, profile),
            None,
            VaultProtectionView::Ephemeral,
        );

        let snapshot = host.snapshot().unwrap();
        let json = snapshot.to_public_json().unwrap();
        assert_eq!(snapshot.profiles.len(), 1);
        assert_eq!(snapshot.ssh_keys.len(), 1);
        assert_eq!(snapshot.ssh_keys[0].comment, "workstation");
        assert_eq!(snapshot.ssh_keys[0].unlock_policy, "every use");
        assert!(
            snapshot.ssh_keys[0]
                .public_openssh
                .starts_with("ssh-ed25519 ")
        );
        assert!(!json.contains(&private_openssh));
        assert!(!json.contains("BEGIN OPENSSH PRIVATE KEY"));
        assert!(!json.contains("5a5a5a5a5a5a5a5a"));
        assert_eq!(snapshot.vault.agent, AgentListenerView::StandaloneRetained);
    }

    #[test]
    fn a_created_persona_joins_the_vault_and_the_host_stays_where_it_was() {
        // Creating and becoming are separate decisions: a persona minted for
        // another device is not one the user is adopting.
        let storage = InMemoryStorage::new();
        let work = Profile::new(
            ProfileId("work".into()),
            "Work",
            Ed25519Keypair::from_seed([0x31; 32]),
        );
        storage.save_profile(&work).unwrap();
        let host = PersonaeHost::new(
            IdentityVault::with_profile(storage, work),
            None,
            VaultProtectionView::Ephemeral,
        );
        let before = IdentityProvider::master_public_key(&host).to_bytes();

        let outcome = host
            .apply_intent(
                PROFILE_CREATE_INTENT,
                &serde_json::to_vec(&CreateProfileIntentV1 {
                    id: "alt".into(),
                    display_name: "Late Night Alt".into(),
                })
                .unwrap(),
            )
            .unwrap();

        match outcome {
            IdentityIntentOutcome::ProfileCreated(receipt) => {
                assert_eq!(receipt.id, "alt");
                assert_eq!(receipt.display_name, "Late Night Alt");
                assert!(receipt.master_public_fingerprint.starts_with("blake3:"));
            },
            other => panic!("expected a creation receipt, got {other:?}"),
        }

        let snapshot = host.snapshot().unwrap();
        let mut ids: Vec<&str> = snapshot.profiles.iter().map(|p| p.id.as_str()).collect();
        ids.sort();
        assert_eq!(ids, ["alt", "work"], "the new persona is in the roster");
        assert_eq!(
            snapshot
                .profiles
                .iter()
                .filter(|p| p.selected)
                .map(|p| p.id.as_str())
                .collect::<Vec<_>>(),
            ["work"],
            "creating does not switch"
        );
        assert_eq!(
            IdentityProvider::master_public_key(&host).to_bytes(),
            before,
            "and the host still speaks as the persona it had"
        );
    }

    #[test]
    fn an_id_that_would_not_survive_becoming_a_filename_is_refused() {
        // `owner_settings::settings_path` replaces anything unsafe with `_`,
        // so accepting `a/b` here would mean it and `a_b` silently share one
        // settings file. Refusing at creation is the only place that cannot be
        // worked around later.
        let host = PersonaeHost::new(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("work".into()),
                    "Work",
                    Ed25519Keypair::from_seed([0x32; 32]),
                ),
            ),
            None,
            VaultProtectionView::Ephemeral,
        );
        for bad in ["../../evil", "a/b", "with space", "", "   "] {
            assert!(
                matches!(
                    host.create_profile(CreateProfileIntentV1 {
                        id: bad.into(),
                        display_name: "Whatever".into(),
                    }),
                    Err(IdentityIntentError::InvalidProfileId)
                ),
                "{bad:?} must be refused"
            );
        }
        assert!(
            host.create_profile(CreateProfileIntentV1 {
                id: "a".repeat(65),
                display_name: "Long".into(),
            })
            .is_err()
        );
        // And a name that is only whitespace, or carries control characters.
        for bad in ["", "   ", "two\nlines"] {
            assert!(
                matches!(
                    host.create_profile(CreateProfileIntentV1 {
                        id: "fine".into(),
                        display_name: bad.into(),
                    }),
                    Err(IdentityIntentError::InvalidProfileName)
                ),
                "{bad:?} must be refused as a name"
            );
        }
        assert_eq!(
            host.snapshot().unwrap().profiles.len(),
            1,
            "nothing was minted by any of the refusals"
        );
    }

    #[test]
    fn creating_over_an_existing_persona_is_refused() {
        // The guard that matters: minting over a persona replaces its master
        // key and every certificate rooted on it.
        let storage = InMemoryStorage::new();
        let work = Profile::new(
            ProfileId("work".into()),
            "Work",
            Ed25519Keypair::from_seed([0x33; 32]),
        );
        storage.save_profile(&work).unwrap();
        let before = work.master.public_key().to_bytes();
        let host = PersonaeHost::new(
            IdentityVault::with_profile(storage, work),
            None,
            VaultProtectionView::Ephemeral,
        );

        assert!(
            host.create_profile(CreateProfileIntentV1 {
                id: "work".into(),
                display_name: "Impostor".into(),
            })
            .is_err()
        );
        assert_eq!(
            IdentityProvider::master_public_key(&host).to_bytes(),
            before,
            "the existing persona keeps its key"
        );
    }

    #[test]
    fn a_projected_switch_is_live_and_remembered_for_the_family() {
        // The gap this closes: the projection could show which persona the
        // host speaks as, and nothing could change it.
        let storage = InMemoryStorage::new();
        let work = Profile::new(
            ProfileId("work".into()),
            "Work",
            Ed25519Keypair::from_seed([0x11; 32]),
        );
        let personal = Profile::new(
            ProfileId("personal".into()),
            "Personal",
            Ed25519Keypair::from_seed([0x12; 32]),
        );
        storage.save_profile(&work).unwrap();
        storage.save_profile(&personal).unwrap();
        let vault_dir =
            std::env::temp_dir().join(format!("graphshell-switch-receipt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&vault_dir);
        let host = PersonaeHost::new(
            IdentityVault::with_profile(storage, work),
            None,
            VaultProtectionView::Ephemeral,
        )
        .with_vault_dir(vault_dir.clone());

        let before = IdentityProvider::master_public_key(&host).to_bytes();
        let outcome = host
            .apply_intent(
                PROFILE_SWITCH_INTENT,
                &serde_json::to_vec(&SwitchProfileIntentV1 {
                    profile: "personal".into(),
                })
                .unwrap(),
            )
            .unwrap();

        // Live: the host's own identity — the one the shared SSH agent
        // serves — is the new persona, and the snapshot marks it selected.
        assert_ne!(
            IdentityProvider::master_public_key(&host).to_bytes(),
            before
        );
        let snapshot = host.snapshot().unwrap();
        let selected: Vec<&str> = snapshot
            .profiles
            .iter()
            .filter(|profile| profile.selected)
            .map(|profile| profile.id.as_str())
            .collect();
        assert_eq!(selected, ["personal"]);

        // Remembered: the family's choice file now names the new persona.
        match outcome {
            IdentityIntentOutcome::ProfileSwitch(receipt) => {
                assert_eq!(receipt.profile, "personal");
                assert!(receipt.remembered);
            },
            other => panic!("expected a profile switch receipt, got {other:?}"),
        }
        assert_eq!(
            roster::chosen_profile(&vault_dir),
            Some(ProfileId("personal".into()))
        );

        // A persona that does not exist is an error, and the host still
        // speaks as the one it had.
        assert!(
            host.switch_profile(SwitchProfileIntentV1 {
                profile: "absent".into(),
            })
            .is_err()
        );
        assert_eq!(
            host.snapshot()
                .unwrap()
                .profiles
                .iter()
                .find(|p| p.selected)
                .unwrap()
                .id,
            "personal"
        );
        let _ = std::fs::remove_dir_all(&vault_dir);
    }

    #[tokio::test]
    async fn projected_approval_intent_releases_the_real_ssh_adapter() {
        let mut private =
            ssh_key::PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
        private.set_comment("approval-key");
        let public = ssh_key::PublicKey::from(&private);
        let mut profile = Profile::new(
            ProfileId("research".to_string()),
            "Research",
            Ed25519Keypair::from_seed([0x2a; 32]),
        );
        profile.slots.insert(
            protocol_key_for(&private),
            slot_for(&private, UnlockTier::PerUse).unwrap(),
        );
        let host = PersonaeHost::with_decision_timeout(
            IdentityVault::with_profile(InMemoryStorage::new(), profile),
            None,
            VaultProtectionView::Ephemeral,
            Duration::from_secs(2),
        );
        let mut agent = host.agent_session();
        let signing = tokio::spawn(async move {
            agent
                .sign(SignRequest {
                    credential: public.key_data().clone().into(),
                    data: b"native-host-approval".to_vec(),
                    flags: 0,
                })
                .await
        });

        let pending = loop {
            if let Some(pending) = host.snapshot().unwrap().pending_signing.into_iter().next() {
                break pending;
            }
            tokio::task::yield_now().await;
        };
        let payload = serde_json::to_vec(&SigningDecisionIntentV1 {
            request_id: pending.request.request_id,
        })
        .unwrap();
        host.apply_intent(SIGNING_APPROVE_ONCE_INTENT, &payload)
            .unwrap();

        let signature = signing.await.unwrap().unwrap();
        assert!(!signature.as_bytes().is_empty());
        let completed = host.snapshot().unwrap();
        assert!(completed.pending_signing.is_empty());
        assert_eq!(completed.signing_history.len(), 1);
        assert!(matches!(
            completed.signing_history[0].result,
            personae::signing::SigningRecordResult::Signed { .. }
        ));
    }

    #[cfg(windows)]
    #[tokio::test]
    async fn isolated_named_pipe_lists_and_signs_through_the_ssh_wire_protocol() {
        use ssh_agent_lib::client::Client;
        use tokio::net::windows::named_pipe::ClientOptions;

        let mut private =
            ssh_key::PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
        private.set_comment("wire-receipt");
        let public = ssh_key::PublicKey::from(&private);
        let mut profile = Profile::new(
            ProfileId("research".to_string()),
            "Research",
            Ed25519Keypair::from_seed([0x3a; 32]),
        );
        profile.slots.insert(
            protocol_key_for(&private),
            slot_for(&private, UnlockTier::PerUse).unwrap(),
        );
        let host = PersonaeHost::with_decision_timeout(
            IdentityVault::with_profile(InMemoryStorage::new(), profile),
            None,
            VaultProtectionView::Ephemeral,
            Duration::from_secs(2),
        );
        assert_eq!(
            host.bind_receipt_listener(r"\\.\pipe\openssh-ssh-agent")
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::PermissionDenied
        );
        assert_eq!(
            host.bind_standard_listener(r"\\.\pipe\graphshell-not-standard")
                .unwrap_err()
                .kind(),
            std::io::ErrorKind::InvalidInput
        );
        let endpoint = format!(r"\\.\pipe\graphshell-h4-receipt-{}", Uuid::new_v4());
        let listener = host.bind_receipt_listener(&endpoint).unwrap();
        let server = tokio::spawn(ssh_agent_lib::agent::listen(listener, host.agent_session()));

        let pipe = ClientOptions::new().open(&endpoint).unwrap();
        let mut client = Client::new(pipe);
        // One slot, two offers. A certifiable key is offered as its
        // certificate first, so a host that trusts the authority needs no
        // per-key enrollment, and then bare, so a host that has only ever seen
        // the key still works. Both resolve to the same slot when signing,
        // because a credential's key data is the key either way.
        let identities = client.request_identities().await.unwrap();
        assert_eq!(
            identities
                .iter()
                .map(|identity| identity.comment.as_str())
                .collect::<Vec<_>>(),
            ["wire-receipt (personae certificate)", "wire-receipt"],
            "the wire offers the certificate before the bare key"
        );
        assert_eq!(
            host.snapshot().unwrap().vault.agent,
            AgentListenerView::ReceiptEndpoint {
                endpoint: endpoint.clone()
            }
        );

        let data = b"graphshell-isolated-wire-receipt".to_vec();
        let verify_data = data.clone();
        let credential = identities[0].credential.clone();
        let signing = tokio::spawn(async move {
            client
                .sign(SignRequest {
                    credential,
                    data,
                    flags: 0,
                })
                .await
        });

        let pending = tokio::time::timeout(Duration::from_secs(1), async {
            loop {
                if let Some(pending) = host.snapshot().unwrap().pending_signing.into_iter().next() {
                    break pending;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .unwrap();
        host.approve_once(pending.request.request_id).unwrap();

        let signature = signing.await.unwrap().unwrap();
        public.key_data().verify(&verify_data, &signature).unwrap();
        assert!(matches!(
            host.snapshot().unwrap().signing_history[0].result,
            personae::signing::SigningRecordResult::Signed { .. }
        ));

        server.abort();
        let _ = server.await;
    }

    /// personae's test-only `ssh-keygen` fixtures.
    fn fixture(name: &str) -> PrivateKey {
        let text = match name {
            "ed25519" => {
                include_str!("../tests/fixtures/ssh/ed25519")
            },
            "rsa2048" => {
                include_str!("../tests/fixtures/ssh/rsa2048")
            },
            "rsa4096" => {
                include_str!("../tests/fixtures/ssh/rsa4096")
            },
            "ecdsa256" => {
                include_str!("../tests/fixtures/ssh/ecdsa256")
            },
            "ecdsa384" => {
                include_str!("../tests/fixtures/ssh/ecdsa384")
            },
            "ecdsa521" => {
                include_str!("../tests/fixtures/ssh/ecdsa521")
            },
            "rsa1024" => {
                include_str!("../tests/fixtures/ssh/rsa1024")
            },
            "rsa2560" => {
                include_str!("../tests/fixtures/ssh/rsa2560")
            },
            "rsa8192" => {
                include_str!("../tests/fixtures/ssh/rsa8192")
            },
            "rsa2048e3" => {
                include_str!("../tests/fixtures/ssh/rsa2048e3")
            },
            "dsa" => include_str!("../tests/fixtures/ssh/dsa"),
            other => panic!("no fixture {other}"),
        };
        PrivateKey::from_openssh(text).unwrap()
    }

    const P4A_KEYS: [&str; 4] = ["rsa2048", "rsa4096", "ecdsa256", "ecdsa384"];

    /// Ruling 54: re-importing a held key, even asking for another tier and
    /// carrying another comment, rewrites nothing and says it is held.
    #[test]
    fn reimporting_a_held_key_rewrites_nothing() {
        let host = PersonaeHost::new(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("research".to_string()),
                    "Research",
                    Ed25519Keypair::from_seed([0x4c; 32]),
                ),
            ),
            None,
            VaultProtectionView::Ephemeral,
        );
        for name in ["ed25519", "rsa2048", "ecdsa256"] {
            let mut first = fixture(name);
            first.set_comment("as first imported");
            let key = protocol_key_for(&first);
            let per_use = ImportSshKeyNativeIntentV1 {
                unlock_policy: SshUnlockPolicyIntentV1::PerUse,
            };
            let imported = host.import_ssh_private(first, per_use).unwrap();
            assert!(!imported.replaced_existing, "{name}");
            let before = slot_parts(&host, &key);

            let mut again = fixture(name);
            again.set_comment("a different comment");
            let receipt = host.import_ssh_private(again, session_import()).unwrap();
            assert!(receipt.replaced_existing, "{name}: reported as held");
            assert_eq!(receipt.comment, "as first imported", "{name}");
            assert_eq!(receipt.unlock_policy, imported.unlock_policy, "{name}");
            assert_eq!(slot_parts(&host, &key), before, "{name}: byte for byte");
            assert_eq!(slot_parts(&host, &key).3, UnlockTier::PerUse, "{name}");
        }
    }

    /// Rulings 55 and 56: import refuses what the agent cannot sign, naming
    /// why, and stores nothing.
    #[test]
    fn native_import_refuses_unsignable_keys_with_the_reason() {
        let host = PersonaeHost::new(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("research".to_string()),
                    "Research",
                    Ed25519Keypair::from_seed([0x4d; 32]),
                ),
            ),
            None,
            VaultProtectionView::Ephemeral,
        );
        for name in [
            "rsa1024",
            "rsa2560",
            "rsa8192",
            "rsa2048e3",
            "dsa",
            "ecdsa521",
        ] {
            let error = host
                .import_ssh_private(fixture(name), session_import())
                .unwrap_err();
            let IdentityIntentError::UnsignableKey(reason) = &error else {
                panic!("{name}: {error}");
            };
            assert!(
                reason.contains("2048 to 4096 bits")
                    || reason.contains("ssh-dss")
                    || reason.contains("P-521"),
                "{name}: {reason}"
            );
        }
        assert!(host.snapshot().unwrap().ssh_keys.is_empty());
    }

    fn slot_parts(
        host: &PersonaeHost<InMemoryStorage>,
        key: &ProtocolKey,
    ) -> (String, Vec<u8>, CredentialLineage, UnlockTier) {
        let vault = host.vault.lock().unwrap();
        match vault.current_profile().unwrap().slots.get(key).expect("slot held") {
            crate::custody::IdentitySlot::Direct {
                kind,
                payload,
                lineage,
                unlock_tier,
            } => (
                kind.clone(),
                payload.as_slice().to_vec(),
                *lineage,
                *unlock_tier,
            ),
            _ => panic!("ssh slots are Direct"),
        }
    }

    fn session_import() -> ImportSshKeyNativeIntentV1 {
        ImportSshKeyNativeIntentV1 {
            unlock_policy: SshUnlockPolicyIntentV1::Session,
        }
    }

    /// P4a, ruling 20: RSA and ECDSA keys land in new fingerprint-keyed slots
    /// and the Ed25519 slot already held is untouched, byte for byte.
    #[test]
    fn native_import_holds_rsa_and_ecdsa_beside_an_untouched_ed25519_slot() {
        let ed25519 = fixture("ed25519");
        let ed_key = protocol_key_for(&ed25519);
        let mut profile = Profile::new(
            ProfileId("research".to_string()),
            "Research",
            Ed25519Keypair::from_seed([0x4a; 32]),
        );
        profile.slots.insert(
            ed_key.clone(),
            slot_for(&ed25519, UnlockTier::PerUse).unwrap(),
        );
        let host = PersonaeHost::new(
            IdentityVault::with_profile(InMemoryStorage::new(), profile),
            None,
            VaultProtectionView::Ephemeral,
        );
        let before = slot_parts(&host, &ed_key);

        for name in P4A_KEYS {
            let key = fixture(name);
            let public = PublicKey::from(&key);
            let fingerprint = public.fingerprint(ssh_key::HashAlg::Sha256).to_string();
            let receipt = host.import_ssh_private(key, session_import()).unwrap();
            assert_eq!(receipt.fingerprint, fingerprint, "{name}");
            assert!(!receipt.replaced_existing, "{name} is a new slot");
            let held = slot_parts(
                &host,
                &ProtocolKey::new(ssh_slot::SSH_MOD_ID, Some(fingerprint)),
            );
            assert_eq!(held.0, ssh_slot::SSH_MOD_ID, "{name}");
            let stored = PrivateKey::from_openssh(&held.1).unwrap();
            assert_eq!(
                PublicKey::from(&stored).key_data(),
                public.key_data(),
                "{name}"
            );
        }

        assert_eq!(slot_parts(&host, &ed_key), before, "the Ed25519 slot");
        assert_eq!(host.snapshot().unwrap().ssh_keys.len(), 1 + P4A_KEYS.len());
    }

    /// The resident's agent signs each imported key over the named-pipe wire,
    /// RSA as the request's flags ask, and refuses a flagless RSA request.
    #[cfg(windows)]
    #[tokio::test]
    async fn imported_rsa_and_ecdsa_keys_sign_over_the_named_pipe_wire() {
        use ssh_agent_lib::client::Client;
        use ssh_key::{EcdsaCurve, HashAlg};
        use tokio::net::windows::named_pipe::ClientOptions;

        let host = PersonaeHost::with_decision_timeout(
            IdentityVault::with_profile(
                InMemoryStorage::new(),
                Profile::new(
                    ProfileId("research".to_string()),
                    "Research",
                    Ed25519Keypair::from_seed([0x4b; 32]),
                ),
            ),
            None,
            VaultProtectionView::Ephemeral,
            Duration::from_secs(2),
        );
        for name in P4A_KEYS {
            host.import_ssh_private(fixture(name), session_import())
                .unwrap();
        }
        let endpoint = format!(r"\\.\pipe\castellan-p4a-receipt-{}", Uuid::new_v4());
        let listener = host.bind_receipt_listener(&endpoint).unwrap();
        let server = tokio::spawn(ssh_agent_lib::agent::listen(listener, host.agent_session()));
        let mut client = Client::new(ClientOptions::new().open(&endpoint).unwrap());
        assert_eq!(
            client.request_identities().await.unwrap().len(),
            2 * P4A_KEYS.len()
        );

        let rsa = |hash| Algorithm::Rsa { hash: Some(hash) };
        let ecdsa = |curve| Algorithm::Ecdsa { curve };
        for (name, flags, expected) in [
            ("rsa2048", 0x02, rsa(HashAlg::Sha256)),
            ("rsa2048", 0x04, rsa(HashAlg::Sha512)),
            ("rsa4096", 0x02, rsa(HashAlg::Sha256)),
            ("rsa4096", 0x04, rsa(HashAlg::Sha512)),
            ("ecdsa256", 0, ecdsa(EcdsaCurve::NistP256)),
            ("ecdsa384", 0, ecdsa(EcdsaCurve::NistP384)),
        ] {
            let public = PublicKey::from(&fixture(name));
            let signature = client
                .sign(SignRequest {
                    credential: public.key_data().clone().into(),
                    data: b"castellan-p4a-wire".to_vec(),
                    flags,
                })
                .await
                .unwrap();
            assert_eq!(signature.algorithm(), expected, "{name} flags {flags}");
            public
                .key_data()
                .verify(b"castellan-p4a-wire", &signature)
                .unwrap();
        }
        let flagless = client
            .sign(SignRequest {
                credential: PublicKey::from(&fixture("rsa2048"))
                    .key_data()
                    .clone()
                    .into(),
                data: b"castellan-p4a-wire".to_vec(),
                flags: 0,
            })
            .await;
        assert!(flagless.is_err(), "ssh-rsa (SHA-1) is never signed");

        server.abort();
        let _ = server.await;
    }

    #[test]
    fn typed_generation_and_confirmed_removal_mutate_the_shared_vault() {
        let profile = Profile::new(
            ProfileId("research".to_string()),
            "Research",
            Ed25519Keypair::from_seed([0x19; 32]),
        );
        let host = PersonaeHost::new(
            IdentityVault::with_profile(InMemoryStorage::new(), profile),
            None,
            VaultProtectionView::Ephemeral,
        );
        let generate = serde_json::to_vec(&GenerateSshKeyIntentV1 {
            comment: "generated in Graphshell".to_string(),
            unlock_policy: SshUnlockPolicyIntentV1::ShortTtl { idle_seconds: 45 },
        })
        .unwrap();
        let generated = host.apply_intent(SSH_GENERATE_INTENT, &generate).unwrap();
        let IdentityIntentOutcome::SshKeyMutation(generated) = generated else {
            panic!("expected SSH key mutation");
        };
        assert_eq!(generated.operation, SshKeyMutationKind::Generated);
        assert_eq!(generated.unlock_policy, "45s idle");
        assert!(generated.public_openssh.starts_with("ssh-ed25519 "));
        assert_eq!(host.snapshot().unwrap().ssh_keys.len(), 1);

        let unconfirmed = serde_json::to_vec(&RemoveSshKeyIntentV1 {
            fingerprint: generated.fingerprint.clone(),
            confirmed: false,
        })
        .unwrap();
        assert!(matches!(
            host.apply_intent(SSH_REMOVE_INTENT, &unconfirmed),
            Err(IdentityIntentError::ConfirmationRequired)
        ));
        assert_eq!(host.snapshot().unwrap().ssh_keys.len(), 1);

        let confirmed = serde_json::to_vec(&RemoveSshKeyIntentV1 {
            fingerprint: generated.fingerprint,
            confirmed: true,
        })
        .unwrap();
        let removed = host.apply_intent(SSH_REMOVE_INTENT, &confirmed).unwrap();
        assert!(matches!(
            removed,
            IdentityIntentOutcome::SshKeyMutation(SshKeyMutationReceipt {
                operation: SshKeyMutationKind::Removed,
                ..
            })
        ));
        assert!(host.snapshot().unwrap().ssh_keys.is_empty());
    }

    #[test]
    fn native_import_never_accepts_private_key_bytes_as_an_intent() {
        let profile = Profile::new(
            ProfileId("research".to_string()),
            "Research",
            Ed25519Keypair::from_seed([0x29; 32]),
        );
        let host = PersonaeHost::new(
            IdentityVault::with_profile(InMemoryStorage::new(), profile),
            None,
            VaultProtectionView::Ephemeral,
        );
        let mut private =
            ssh_key::PrivateKey::random(&mut rand_core::OsRng, Algorithm::Ed25519).unwrap();
        private.set_comment("native import");
        let private_openssh = private.to_openssh(LineEnding::LF).unwrap().to_string();
        let receipt = host
            .import_ssh_private(
                private,
                ImportSshKeyNativeIntentV1 {
                    unlock_policy: SshUnlockPolicyIntentV1::PerUse,
                },
            )
            .unwrap();
        let public_json = serde_json::to_string(&receipt).unwrap();
        assert_eq!(receipt.operation, SshKeyMutationKind::Imported);
        assert!(!public_json.contains(&private_openssh));
        assert!(!public_json.contains("BEGIN OPENSSH PRIVATE KEY"));

        let options = serde_json::to_vec(&ImportSshKeyNativeIntentV1 {
            unlock_policy: SshUnlockPolicyIntentV1::Session,
        })
        .unwrap();
        assert!(matches!(
            host.apply_intent(SSH_IMPORT_NATIVE_INTENT, &options),
            Err(IdentityIntentError::NativeHandoffRequired)
        ));
        assert_eq!(host.snapshot().unwrap().ssh_keys.len(), 1);
    }
}

#[cfg(test)]
#[path = "authority_lock_tests.rs"]
mod lock_tests;
