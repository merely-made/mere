// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Multi-protocol identity vault.
//!
//! Phase 2C v0 skeleton per
//! [`mere/design_docs/mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md`](../../../../../../design_docs/mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md)
//! §3 — extends the existing [`crate::IdentityProvider`] surface (which
//! continues to work unchanged) with a multi-protocol [`Profile`] holding
//! [`IdentitySlot`]s over a pluggable [`IdentityStorage`] backend.
//!
//! ## Direct vs Bootstrap slots
//!
//! Per §3.1: not all protocol identities are vault-modelable bytes.
//!
//! - **Direct** slots store the credential *itself* — Nostr `nsec`,
//!   Cable cabal keys, IRC SASL, X.509 client certs, SSH keys. The bytes
//!   in the slot are the identity.
//! - **Bootstrap** slots store a *bootstrap secret* plus a reference to an
//!   external SDK's state directory — Matrix's `matrix-rust-sdk`
//!   `StateStore` (per-device keys, cross-signing keys, pickled
//!   Olm/Megolm sessions), ATproto's session state, ActivityPub-with-
//!   server-state. The vault does not model what the SDK stores; it
//!   provides an encrypted-at-rest directory the SDK uses as its own
//!   storage root.
//!
//! ## Lineage
//!
//! [`CredentialLineage`] captures the recovery semantics of a slot. A
//! single "recovery phrase recovers everything" mental model is wrong:
//! locally-derived slots reconstruct from master, but locally-generated-
//! externally-registered slots (Matrix device keys) require re-verification
//! from another trusted device. See plan §3.4.
//!
//! ## Unlock tiers
//!
//! [`UnlockTier`] is declared per slot at registration time, and the
//! unlock UX falls out of those declarations rather than being a global
//! setting. v0 carries the type but does not enforce TTL; enforcement is
//! a follow-up. See plan §3.6.
//!
//! ## Isolation
//!
//! v0 ships at threat-model level 0 — single-process trust. A compromised
//! mod can read any slot once unlocked. Per-mod capability tokens (level 1)
//! land with the Matrix mod (first Bootstrap-category consumer). See plan
//! §3.7.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::unlock::{UnlockMethod, UnlockMethods};
use crate::{Ed25519Keypair, Ed25519PublicKey, IdentityError, IdentityProvider};
use insigne::DerivedKeyAttestation;

/// Stable identifier for a profile within a vault.
///
/// Profiles are independent: per-profile master keys do not derive from
/// each other. A user with multiple profiles (work / personal / alt)
/// switches between them at the vault level; mods hold per-profile
/// sub-instances (Element-style).
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProfileId(pub String);

/// Compound key for a slot within a profile.
///
/// `mod_id` identifies the protocol mod owning the slot ("nostr",
/// "matrix", "irc", "cable"). `instance` distinguishes multiple slots of
/// the same kind — e.g., two Matrix accounts on different homeservers, or
/// per-cabal Cable keys.
#[derive(Clone, Debug, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct ProtocolKey {
    /// The protocol mod's id.
    pub mod_id: String,
    /// Optional instance discriminator within the mod.
    pub instance: Option<String>,
}

impl ProtocolKey {
    /// Construct a new protocol key.
    pub fn new(mod_id: impl Into<String>, instance: Option<String>) -> Self {
        Self {
            mod_id: mod_id.into(),
            instance,
        }
    }
}

/// Recovery semantics for a slot. See plan §3.4.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CredentialLineage {
    /// Derived locally from master via deterministic salt.
    /// Recoverable from master alone (Cable cabal keys, Mere-native).
    LocallyDerived,
    /// Generated locally and registered with an external authority.
    /// Lost = re-register; recovery phrase does NOT regenerate it
    /// (Matrix device keys).
    LocallyGeneratedExternallyRegistered,
    /// Issued by external authority. Rotates / expires; not meaningfully
    /// backupable (access tokens, refresh JWTs).
    ExternallyIssued,
    /// Externally rooted (CA, identity provider) but locally held. CA
    /// owns revocation (X.509 client certs).
    ExternallyRootedLocallyHeld,
}

impl CredentialLineage {
    /// What losing this device means for a slot of this lineage.
    ///
    /// Plan §3.4 requires surfacing this per slot: a single "the recovery
    /// phrase brings everything back" story is wrong and would mislead
    /// users into thinking externally-registered credentials survive a
    /// device loss. Any vault UI (CLI, pane) shows this verbatim.
    pub fn device_loss_note(&self) -> &'static str {
        match self {
            Self::LocallyDerived => {
                "Recoverable: the master key re-derives this slot deterministically."
            },
            Self::LocallyGeneratedExternallyRegistered => {
                "Not recoverable from the vault alone: register a replacement with the service. \
                 A recovery phrase unlocks the vault; it does not regenerate this key."
            },
            Self::ExternallyIssued => {
                "Not backupable: credentials of this kind rotate and expire by design. \
                 Re-authenticate with the issuer."
            },
            Self::ExternallyRootedLocallyHeld => {
                "Upstream's call: the issuing authority revokes and reissues."
            },
        }
    }
}

/// Unlock tier declared at slot registration. See plan §3.6.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum UnlockTier {
    /// Unlocked once at vault open; stays unlocked for the app session.
    Session,
    /// Unlocked on first use; auto-relocks after the configured idle
    /// window in seconds.
    ShortTtl {
        /// Idle window in seconds before the slot relocks.
        idle_seconds: u32,
    },
    /// Re-prompt on every credential read.
    PerUse,
}

/// Bytes that zero on drop. Used for slot payloads.
#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct SecretBytes(Vec<u8>);

impl SecretBytes {
    /// Wrap raw bytes (move).
    pub fn new(bytes: Vec<u8>) -> Self {
        Self(bytes)
    }

    /// Borrow as a slice.
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Length in bytes.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Whether the secret is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Debug for SecretBytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SecretBytes")
            .field("len", &self.0.len())
            .finish_non_exhaustive()
    }
}

/// A protocol-identity slot.
///
/// Two categories: vault-modelable [`Direct`] credentials (the slot bytes
/// *are* the identity), and SDK-delegated [`Bootstrap`] credentials (the
/// slot holds a bootstrap secret; the SDK owns its own encrypted state
/// directory).
///
/// `kind` is a stringly-typed protocol identifier ("nostr", "matrix",
/// "irc", "cable", …) so this enum stays free of protocol-specific
/// dependencies. Mods that want stronger typing wrap their slot into the
/// generic shape at the vault boundary.
///
/// [`Direct`]: IdentitySlot::Direct
/// [`Bootstrap`]: IdentitySlot::Bootstrap
#[derive(Debug)]
pub enum IdentitySlot {
    /// Vault-modelable credential. The slot IS the identity.
    Direct {
        /// Protocol id ("nostr", "cable", "irc", "tls", "ssh", "misfin").
        kind: String,
        /// The credential bytes.
        payload: SecretBytes,
        /// Recovery semantics.
        lineage: CredentialLineage,
        /// When the slot must be unlocked.
        unlock_tier: UnlockTier,
    },
    /// SDK-delegated credential. Vault holds a bootstrap secret + an
    /// encrypted state directory the SDK uses as its own storage root.
    Bootstrap {
        /// Protocol id ("matrix", "atproto", "activitypub-server").
        kind: String,
        /// Bootstrap secret (login flow seed, recovery key, etc.).
        bootstrap: SecretBytes,
        /// SDK-owned state-dir path. The vault wraps this directory with
        /// at-rest encryption; the SDK reads and writes as if it were a
        /// plain directory.
        state_dir: PathBuf,
        /// Recovery semantics.
        lineage: CredentialLineage,
        /// When the bootstrap secret must be unlocked.
        unlock_tier: UnlockTier,
    },
}

impl IdentitySlot {
    /// The protocol kind ("nostr", "matrix", ...).
    pub fn kind(&self) -> &str {
        match self {
            Self::Direct { kind, .. } | Self::Bootstrap { kind, .. } => kind,
        }
    }

    /// The slot's recovery lineage.
    pub fn lineage(&self) -> CredentialLineage {
        match self {
            Self::Direct { lineage, .. } | Self::Bootstrap { lineage, .. } => *lineage,
        }
    }

    /// The slot's unlock tier.
    pub fn unlock_tier(&self) -> UnlockTier {
        match self {
            Self::Direct { unlock_tier, .. } | Self::Bootstrap { unlock_tier, .. } => *unlock_tier,
        }
    }
}

/// User-level identity profile.
///
/// One vault holds one or more profiles. The master keypair is per-
/// profile; profiles do not derive from each other.
pub struct Profile {
    /// Stable id within the vault.
    pub id: ProfileId,
    /// User-facing display name.
    pub display_name: String,
    /// Master keypair. Used for Mere-native derivation
    /// ([`IdentityProvider::derive_keypair`]) and for transport identity.
    pub master: Ed25519Keypair,
    /// Per-protocol slots keyed by [`ProtocolKey`], wiped whole when freed
    /// (vault lock ruling 71).
    pub slots: SlotMap,
}

impl Profile {
    /// Build a new profile with the given master keypair and no slots.
    pub fn new(id: ProfileId, display_name: impl Into<String>, master: Ed25519Keypair) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            master,
            slots: SlotMap::new(),
        }
    }

    /// Whether this profile holds a slot for the given protocol key.
    pub fn has_slot(&self, key: &ProtocolKey) -> bool {
        self.slots.contains_key(key)
    }
}

/// Lightweight summary of a profile, for listing without unlocking the
/// full keypair material.
#[derive(Clone, Debug)]
pub struct ProfileSummary {
    /// The profile id.
    pub id: ProfileId,
    /// Display name.
    pub display_name: String,
    /// Number of slots in the profile (no slot contents).
    pub slot_count: usize,
}

/// Storage backend for an [`IdentityVault`].
///
/// Shipped backends: [`InMemoryStorage`] (tests/ephemeral), the on-disk
/// [`crate::PassphraseEncryptedStorage`] (portable passphrase vault), and
/// [`crate::SealedProfileStorage`] (sealed records keyed by the
/// [`crate::startup_unlock`] `AutoOs` ladder — the desktop default where
/// that ladder exists; DPAPI on Windows today).
pub trait IdentityStorage: Send + Sync {
    /// Load a profile by id.
    fn load_profile(&self, id: &ProfileId) -> Result<Profile, IdentityError>;

    /// Persist a profile (creates or overwrites).
    fn save_profile(&self, profile: &Profile) -> Result<(), IdentityError>;

    /// Delete a profile by id.
    fn delete_profile(&self, id: &ProfileId) -> Result<(), IdentityError>;

    /// List all profiles in the storage.
    fn list_profiles(&self) -> Result<Vec<ProfileSummary>, IdentityError>;

    /// Which unlock methods could re-key this storage on this device now.
    fn unlock_methods(&self) -> UnlockMethods;

    /// Forget the at-rest key. Until [`Self::unlock`], every profile call
    /// returns [`IdentityError::Locked`]. Locking twice is harmless.
    fn lock(&self);

    /// Whether the at-rest key is forgotten.
    fn is_locked(&self) -> bool;

    /// Re-key from a user act. A wrong credential is an error and leaves
    /// the storage locked.
    fn unlock(&self, method: UnlockMethod<'_>) -> Result<(), IdentityError>;

    /// Enrol a passphrase that can unlock this storage (ruling 39). Needs
    /// the storage unlocked; an existing enrolment is refused, not replaced.
    /// Backends that hold no OS-wrapped root refuse.
    fn enroll_passphrase(&self, _passphrase: &[u8]) -> Result<(), IdentityError> {
        Err(IdentityError::Backend(
            "this storage cannot enrol a passphrase".to_string(),
        ))
    }
}

/// Borrowed storage delegates, so a vault can be opened over a backend
/// the caller keeps (`IdentityVault<&dyn IdentityStorage>`) without
/// giving up ownership.
impl<T: IdentityStorage + ?Sized> IdentityStorage for &T {
    fn load_profile(&self, id: &ProfileId) -> Result<Profile, IdentityError> {
        (**self).load_profile(id)
    }

    fn save_profile(&self, profile: &Profile) -> Result<(), IdentityError> {
        (**self).save_profile(profile)
    }

    fn delete_profile(&self, id: &ProfileId) -> Result<(), IdentityError> {
        (**self).delete_profile(id)
    }

    fn list_profiles(&self) -> Result<Vec<ProfileSummary>, IdentityError> {
        (**self).list_profiles()
    }

    fn unlock_methods(&self) -> UnlockMethods {
        (**self).unlock_methods()
    }

    fn lock(&self) {
        (**self).lock()
    }

    fn is_locked(&self) -> bool {
        (**self).is_locked()
    }

    fn unlock(&self, method: UnlockMethod<'_>) -> Result<(), IdentityError> {
        (**self).unlock(method)
    }

    fn enroll_passphrase(&self, passphrase: &[u8]) -> Result<(), IdentityError> {
        (**self).enroll_passphrase(passphrase)
    }
}

/// Boxed storage delegates, so callers can pick a backend at runtime
/// (`IdentityVault<Box<dyn IdentityStorage>>`).
impl<T: IdentityStorage + ?Sized> IdentityStorage for Box<T> {
    fn load_profile(&self, id: &ProfileId) -> Result<Profile, IdentityError> {
        (**self).load_profile(id)
    }

    fn save_profile(&self, profile: &Profile) -> Result<(), IdentityError> {
        (**self).save_profile(profile)
    }

    fn delete_profile(&self, id: &ProfileId) -> Result<(), IdentityError> {
        (**self).delete_profile(id)
    }

    fn list_profiles(&self) -> Result<Vec<ProfileSummary>, IdentityError> {
        (**self).list_profiles()
    }

    fn unlock_methods(&self) -> UnlockMethods {
        (**self).unlock_methods()
    }

    fn lock(&self) {
        (**self).lock()
    }

    fn is_locked(&self) -> bool {
        (**self).is_locked()
    }

    fn unlock(&self, method: UnlockMethod<'_>) -> Result<(), IdentityError> {
        (**self).unlock(method)
    }

    fn enroll_passphrase(&self, passphrase: &[u8]) -> Result<(), IdentityError> {
        (**self).enroll_passphrase(passphrase)
    }
}

/// What stays visible while the vault is locked: no secret material
/// (vault lock ruling 11).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PublicProfile {
    /// The profile id.
    pub id: ProfileId,
    /// Display name.
    pub display_name: String,
    /// The master public key.
    pub master_public_key: Ed25519PublicKey,
    /// Each slot's description, without its payload.
    pub slots: Vec<SlotSummary>,
}

/// One slot, described without its secret.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotSummary {
    /// Where the slot lives.
    pub key: ProtocolKey,
    /// The protocol kind.
    pub kind: String,
    /// Recovery lineage.
    pub lineage: CredentialLineage,
    /// Unlock tier (consent, per ruling 12).
    pub unlock_tier: UnlockTier,
}

impl PublicProfile {
    fn of(profile: &Profile) -> Self {
        let mut slots: Vec<SlotSummary> = profile
            .slots
            .iter()
            .map(|(key, slot)| SlotSummary {
                key: key.clone(),
                kind: slot.kind().to_string(),
                lineage: slot.lineage(),
                unlock_tier: slot.unlock_tier(),
            })
            .collect();
        slots.sort_by(|a, b| a.key.cmp(&b.key));
        Self {
            id: profile.id.clone(),
            display_name: profile.display_name.clone(),
            master_public_key: profile.master.public_key(),
            slots,
        }
    }
}

/// User-facing identity vault.
///
/// Holds the storage backend and a currently-loaded profile. Other Mere
/// crates consume the vault either through the [`IdentityProvider`] impl
/// (for the legacy single-master surface) or through the slot-aware API
/// here (for multi-protocol consumers).
///
/// ## Locking
///
/// [`Self::lock`] drops the profile (its keys zeroize on drop) and has the
/// storage forget its at-rest key, so nothing reloads it silently (vault
/// lock plan, ruling 25). While locked, every accessor that reaches secret
/// material returns [`IdentityError::Locked`]; [`Self::public_profile`]
/// and the master public key stay readable. [`Self::unlock`] takes a user
/// act and reloads the same profile.
pub struct IdentityVault<S: IdentityStorage> {
    storage: S,
    current: Option<Profile>,
    public: PublicProfile,
}

impl<S: IdentityStorage> IdentityVault<S> {
    /// Open a vault loading the named profile from storage.
    #[tracing::instrument(level = "debug", skip(storage), fields(?id))]
    pub fn open(storage: S, id: &ProfileId) -> Result<Self, IdentityError> {
        let current = storage.load_profile(id)?;
        Ok(Self::with_profile(storage, current))
    }

    /// Open a vault with an already-loaded profile (no storage round-trip).
    ///
    /// Useful for tests and for callers that constructed a profile in-process.
    pub fn with_profile(storage: S, profile: Profile) -> Self {
        Self {
            storage,
            public: PublicProfile::of(&profile),
            current: Some(profile),
        }
    }

    /// The currently-loaded profile, or [`IdentityError::Locked`].
    pub fn current_profile(&self) -> Result<&Profile, IdentityError> {
        self.current.as_ref().ok_or(IdentityError::Locked)
    }

    /// Borrow a slot by key, or [`IdentityError::Locked`].
    pub fn slot(&self, key: &ProtocolKey) -> Result<Option<&IdentitySlot>, IdentityError> {
        Ok(self.current_profile()?.slots.get(key))
    }

    /// The profile this vault speaks as, locked or not.
    pub fn profile_id(&self) -> &ProfileId {
        &self.public.id
    }

    /// The secret-free view of the current profile, locked or not.
    pub fn public_profile(&self) -> &PublicProfile {
        &self.public
    }

    /// Add or replace a slot in the current profile, persisting the
    /// updated profile to storage.
    #[tracing::instrument(level = "debug", skip(self, slot), fields(?key))]
    pub fn add_slot(&mut self, key: ProtocolKey, slot: IdentitySlot) -> Result<(), IdentityError> {
        let current = self.current.as_mut().ok_or(IdentityError::Locked)?;
        current.slots.insert(key, slot);
        self.public = PublicProfile::of(current);
        self.storage.save_profile(current)
    }

    /// Remove a slot from the current profile, persisting.
    #[tracing::instrument(level = "debug", skip(self), fields(?key))]
    pub fn remove_slot(&mut self, key: &ProtocolKey) -> Result<bool, IdentityError> {
        let current = self.current.as_mut().ok_or(IdentityError::Locked)?;
        let removed = current.slots.remove(key).is_some();
        if removed {
            self.public = PublicProfile::of(current);
            self.storage.save_profile(current)?;
        }
        Ok(removed)
    }

    /// Whether the vault is locked.
    pub fn is_locked(&self) -> bool {
        self.current.is_none()
    }

    /// Which unlock methods this device offers for this vault now.
    pub fn unlock_methods(&self) -> UnlockMethods {
        self.storage.unlock_methods()
    }

    /// Enrol a passphrase over the vault's root (ruling 39), so a device
    /// with no OS presence can lock (ruling 27). Refused while locked.
    pub fn enroll_passphrase(&self, passphrase: &[u8]) -> Result<(), IdentityError> {
        if self.current.is_none() {
            return Err(IdentityError::Locked);
        }
        self.storage.enroll_passphrase(passphrase)
    }

    /// Lock: drop the profile and have the storage forget its key.
    ///
    /// Refused while no unlock method is available (ruling 27), so a lock
    /// is never one nobody can undo. Locking a locked vault is a no-op.
    #[tracing::instrument(level = "info", skip(self), fields(profile = %self.public.id.0))]
    pub fn lock(&mut self) -> Result<(), IdentityError> {
        if self.current.is_none() {
            return Ok(());
        }
        if !self.storage.unlock_methods().any() {
            return Err(IdentityError::Backend(
                "refusing to lock: no unlock method is available on this device".to_string(),
            ));
        }
        self.current = None;
        self.storage.lock();
        Ok(())
    }

    /// Unlock by a user act, reloading the profile that was current.
    ///
    /// A wrong credential, or a profile that will not load, leaves the vault
    /// and its storage locked. Unlocking an unlocked vault is a no-op.
    #[tracing::instrument(level = "info", skip_all, fields(profile = %self.public.id.0))]
    pub fn unlock(&mut self, method: UnlockMethod<'_>) -> Result<(), IdentityError> {
        if self.current.is_some() {
            return Ok(());
        }
        self.storage.unlock(method)?;
        match self.storage.load_profile(&self.public.id) {
            Ok(profile) => {
                self.public = PublicProfile::of(&profile);
                self.current = Some(profile);
                Ok(())
            },
            Err(error) => {
                self.storage.lock();
                Err(error)
            },
        }
    }

    /// Borrow the underlying storage.
    pub fn storage(&self) -> &S {
        &self.storage
    }

    /// Switch to another profile, live.
    ///
    /// This is the vault-level switching the [`ProfileId`] docs promise. It is
    /// deliberately **not** a restart: everything reading this vault — every
    /// consumer of a shared `Arc<Mutex<IdentityVault>>`, including a
    /// [`crate::agent::VaultAgent`] serving SSH keys — speaks as the new
    /// persona from its next operation, with no process anywhere told to die.
    ///
    /// Loads before replacing, so a profile that will not load leaves the
    /// current one untouched rather than half-switched.
    ///
    /// Refused while locked (ruling 14), whether or not the storage could
    /// load the target.
    #[tracing::instrument(level = "debug", skip(self), fields(?id))]
    pub fn switch_profile(&mut self, id: &ProfileId) -> Result<(), IdentityError> {
        if self.current.is_none() {
            return Err(IdentityError::Locked);
        }
        let next = self.storage.load_profile(id)?;
        self.public = PublicProfile::of(&next);
        self.current = Some(next);
        Ok(())
    }
}

/// The master public key stays answerable while locked; derivation and
/// attestation need the master secret and return [`IdentityError::Locked`].
impl<S: IdentityStorage> IdentityProvider for IdentityVault<S> {
    fn master_public_key(&self) -> Ed25519PublicKey {
        self.public.master_public_key
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<Ed25519Keypair, IdentityError> {
        Ok(self.current_profile()?.master.derive_child(salt))
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<DerivedKeyAttestation, IdentityError> {
        Ok(crate::provider::attest_derived_key(
            &self.current_profile()?.master,
            salt,
        ))
    }
}

// ─── In-memory storage (test fixture) ─────────────────────────────────────

/// In-memory [`IdentityStorage`] for tests and ephemeral runs.
///
/// Profiles are held in a `Mutex<HashMap>`; a clone of the storage shares
/// the same backing map so multiple consumers can see each other's
/// profile updates. Not for production — see plan §3.2 for the production
/// backends to come.
#[derive(Default, Clone)]
pub struct InMemoryStorage {
    inner: std::sync::Arc<std::sync::Mutex<HashMap<ProfileId, ProfileBytes>>>,
}

/// Wire-shaped representation of a profile for in-memory storage.
///
/// In a real backend this would be the at-rest encrypted form. Here we
/// keep it as a structural clone so tests can round-trip without
/// implementing serde plumbing yet.
struct ProfileBytes {
    display_name: String,
    master_seed: [u8; 32],
    slots: HashMap<ProtocolKey, SerializedSlot>,
}

/// Storage-shaped slot — same shape as [`IdentitySlot`] but with cloneable
/// payload bytes (so the storage map can hold and re-issue them).
struct SerializedSlot {
    kind: String,
    payload: Vec<u8>,
    bootstrap_state_dir: Option<PathBuf>,
    lineage: CredentialLineage,
    unlock_tier: UnlockTier,
    is_bootstrap: bool,
}

impl InMemoryStorage {
    /// Construct an empty in-memory storage.
    pub fn new() -> Self {
        Self::default()
    }
}

fn serialize_slot(slot: &IdentitySlot) -> SerializedSlot {
    match slot {
        IdentitySlot::Direct {
            kind,
            payload,
            lineage,
            unlock_tier,
        } => SerializedSlot {
            kind: kind.clone(),
            payload: payload.as_slice().to_vec(),
            bootstrap_state_dir: None,
            lineage: *lineage,
            unlock_tier: *unlock_tier,
            is_bootstrap: false,
        },
        IdentitySlot::Bootstrap {
            kind,
            bootstrap,
            state_dir,
            lineage,
            unlock_tier,
        } => SerializedSlot {
            kind: kind.clone(),
            payload: bootstrap.as_slice().to_vec(),
            bootstrap_state_dir: Some(state_dir.clone()),
            lineage: *lineage,
            unlock_tier: *unlock_tier,
            is_bootstrap: true,
        },
    }
}

fn deserialize_slot(s: &SerializedSlot) -> IdentitySlot {
    if s.is_bootstrap {
        IdentitySlot::Bootstrap {
            kind: s.kind.clone(),
            bootstrap: SecretBytes::new(s.payload.clone()),
            state_dir: s
                .bootstrap_state_dir
                .clone()
                .unwrap_or_else(|| PathBuf::from(".")),
            lineage: s.lineage,
            unlock_tier: s.unlock_tier,
        }
    } else {
        IdentitySlot::Direct {
            kind: s.kind.clone(),
            payload: SecretBytes::new(s.payload.clone()),
            lineage: s.lineage,
            unlock_tier: s.unlock_tier,
        }
    }
}

impl IdentityStorage for InMemoryStorage {
    fn load_profile(&self, id: &ProfileId) -> Result<Profile, IdentityError> {
        let guard = self.inner.lock().unwrap();
        let bytes = guard
            .get(id)
            .ok_or_else(|| IdentityError::Backend(format!("profile not found: {:?}", id)))?;
        let mut slots = SlotMap::with_capacity(bytes.slots.len());
        for (k, s) in &bytes.slots {
            slots.insert(k.clone(), deserialize_slot(s));
        }
        Ok(Profile {
            id: id.clone(),
            display_name: bytes.display_name.clone(),
            master: Ed25519Keypair::from_seed(bytes.master_seed),
            slots,
        })
    }

    fn save_profile(&self, profile: &Profile) -> Result<(), IdentityError> {
        let mut guard = self.inner.lock().unwrap();
        let mut slot_map = HashMap::with_capacity(profile.slots.len());
        for (k, s) in &profile.slots {
            slot_map.insert(k.clone(), serialize_slot(s));
        }
        guard.insert(
            profile.id.clone(),
            ProfileBytes {
                display_name: profile.display_name.clone(),
                master_seed: profile.master.to_seed(),
                slots: slot_map,
            },
        );
        Ok(())
    }

    fn delete_profile(&self, id: &ProfileId) -> Result<(), IdentityError> {
        self.inner.lock().unwrap().remove(id);
        Ok(())
    }

    fn list_profiles(&self) -> Result<Vec<ProfileSummary>, IdentityError> {
        let guard = self.inner.lock().unwrap();
        Ok(guard
            .iter()
            .map(|(id, bytes)| ProfileSummary {
                id: id.clone(),
                display_name: bytes.display_name.clone(),
                slot_count: bytes.slots.len(),
            })
            .collect())
    }

    /// None: the fixture holds plaintext and has no key to forget, so a
    /// vault over it refuses to lock.
    fn unlock_methods(&self) -> UnlockMethods {
        UnlockMethods::default()
    }

    fn lock(&self) {}

    fn is_locked(&self) -> bool {
        false
    }

    fn unlock(&self, _method: UnlockMethod<'_>) -> Result<(), IdentityError> {
        Ok(())
    }
}

mod slot_map;

pub use slot_map::SlotMap;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod lock_tests;
