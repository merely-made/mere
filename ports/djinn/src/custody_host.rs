// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Djinn's one view of custody (custody isolation plan, H2).
//!
//! Every vault use in Djinn goes through [`ResidentCustody`]: public reads,
//! lock and unlock, profile intents, root and root-derived acts, the
//! wallet's epochs and grants, retained-payload sealing and the credential
//! catalog. A production build has exactly one implementation,
//! [`remote::RemoteHost`], a client of the independently launched
//! custodian; it links no castellan and opens no vault. The
//! `in-process-custody` feature adds [`in_process`], Castellan's host in
//! this process, for the test suites and receipt hosts that predate the
//! custodian. It is never a fallback: a build either has the feature or it
//! does not, and the resident never chooses it because a custodian refused.
//!
//! Releases hand Djinn derived keys only, each checked against its
//! attestation; a root never crosses. Acts that must not release their key
//! (`sign`, `attest`) are performed where the key is.

use std::path::Path;
use std::sync::Arc;

use dramatis::authentication::PassphraseChangeFailureV1;
use dramatis::intents::{CreateProfileIntentV1, ImportSshKeyNativeIntentV1, SwitchProfileIntentV1};
use graphshell_endpoint::door::custody::{
    CustodyRefusal, EpochKeyRequest, KeySource, ReleasedEpochKey, ReleasedKey, StationGrantRequest,
};
use graphshell_endpoint::door::custody_payload::{PayloadRequest, PayloadResult};
use graphshell_endpoint::door::identity::{IdentitySurfaceSnapshot, VaultLockView};
use personae::{IdentityError, IdentityProvider, PersonaId, ProfileId, RetainedKeys};

use crate::door::browser_carrier::NativeIdentityFailure;
use crate::identity_ui::NativeIdentityUi;

#[cfg(feature = "in-process-custody")]
pub mod in_process;
pub mod remote;

/// Something in Djinn that keeps keys derived from the vault. It forgets
/// them when the custodian reports a lock (or is lost) and re-derives them
/// through the custodian after an unlock.
pub trait LockHolder: Send + Sync {
    /// What the log calls it.
    fn name(&self) -> &str;

    /// The persona that sealed this holder's data, when fixed.
    fn key_profile(&self) -> Option<&ProfileId> {
        None
    }

    /// Drop every vault-derived key now.
    fn lock(&self);

    /// Re-derive the keys from the identity just unlocked. An error leaves
    /// the holder locked.
    fn unlock(&self, vault: &dyn IdentityProvider) -> Result<(), IdentityError>;
}

/// What a terminal vault command answered: public text or CA installation
/// material, never a secret.
pub enum VaultOutput {
    Text(String),
    Enrollment {
        target: String,
        script: String,
        confirmation: String,
    },
}

/// An SSH key the owner imported: the public receipt.
pub type ImportedSshKey = dramatis::receipts::SshKeyMutationReceipt;

/// One credential-catalog request, answered with the catalog's public
/// records as JSON (quarantine cards, older imports, CXF preview/receipt).
pub enum CredentialCall {
    Quarantine(ProfileId),
    UnassignedImports,
    AssignImport(serde_json::Value),
    Review(serde_json::Value),
    CxfPreview(Vec<u8>),
    CxfImport {
        source: Vec<u8>,
        destinations: serde_json::Value,
        preview_sha256: String,
    },
}

/// Custody, as Djinn reaches it.
pub trait ResidentCustody: IdentityProvider + Send + Sync + 'static {
    /// The public identity read model, answerable while locked.
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot>;
    fn is_locked(&self) -> bool;
    /// The lock as observers see it (a UI, the status route).
    fn lock_state(&self) -> tokio::sync::watch::Receiver<VaultLockView>;
    fn lock_vault(&self) -> Result<(), String>;
    /// Unlock with a passphrase the owner typed on Djinn's own surface: the
    /// documented terminal-unlock exception toward a custodian.
    fn unlock_with_passphrase(&self, passphrase: &[u8]) -> Result<(), String>;
    /// OS presence (Windows Hello) unlock, available only where the vault
    /// is OS-sealed in this process.
    fn unlock_with_presence(&self, ui: &dyn NativeIdentityUi) -> Option<Result<(), String>> {
        let _ = ui;
        None
    }
    fn register_lock_holder(&self, holder: Arc<dyn LockHolder>);
    /// A restricted provider for exactly `salts`, kept through every lock.
    fn retained_keys(&self, salts: &[Vec<u8>]) -> Result<Arc<RetainedKeys>, IdentityError>;
    fn current_profile_id(&self) -> ProfileId;
    fn has_profile(&self, profile: &ProfileId) -> Result<bool, IdentityError>;
    /// One typed identity intent; signing decisions are refused.
    fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String>;
    fn switch_profile(&self, intent: SwitchProfileIntentV1) -> Result<(), String>;
    fn create_profile(&self, intent: CreateProfileIntentV1) -> Result<(), String>;
    /// Release derived keys of `source` (the open or a named persona, or the
    /// wallet), each with the root's attestation.
    fn release(&self, source: KeySource, profile: Option<&ProfileId>, salts: &[Vec<u8>]) -> Result<([u8; 32], Vec<ReleasedKey>), CustodyRefusal>;
    fn attest(&self, source: KeySource, profile: Option<&ProfileId>, salt: &[u8]) -> Result<insigne::DerivedKeyAttestation, CustodyRefusal>;
    /// Sign with the key `salt` derives, where the key is.
    fn sign(&self, source: KeySource, profile: Option<&ProfileId>, salt: &[u8], message: &[u8]) -> Result<([u8; 32], Vec<u8>), CustodyRefusal>;
    /// Sign one delegation statement in `audience`, checked where signed.
    fn sign_delegation(&self, audience: &str, salt: &[u8], message: &[u8]) -> Result<([u8; 32], Vec<u8>), CustodyRefusal>;
    /// The public key of `source`'s root.
    fn root_public_key(&self, source: KeySource, profile: Option<&ProfileId>) -> Result<[u8; 32], CustodyRefusal>;
    fn vault_command(&self, profile: &ProfileId, command: &str, args: &[String]) -> Result<VaultOutput, CustodyRefusal>;
    fn change_passphrase(&self, old: &[u8], new: &[u8]) -> Result<(), PassphraseChangeFailureV1>;
    fn import_ssh_key(&self, openssh: &[u8], passphrase: Option<&[u8]>, options: ImportSshKeyNativeIntentV1) -> Result<ImportedSshKey, NativeIdentityFailure>;
    /// Keys derived from a persona wallet's current private epoch.
    fn epoch_keys(&self, wallet_root: &Path, persona: PersonaId, device_label: Option<&str>, requests: &[EpochKeyRequest]) -> Result<Vec<ReleasedEpochKey>, CustodyRefusal>;
    fn station_grant(&self, wallet_root: &Path, request: &StationGrantRequest, issuer: [u8; 32]) -> Result<personae::carry::DeviceGrantSet, CustodyRefusal>;
    fn revoke_device(&self, wallet_root: &Path, device: uuid::Uuid) -> Result<pandect::RemoteAuthRevocationOutcome, CustodyRefusal>;
    /// Seal or unseal one retained payload, admitted to the open profile.
    fn payload(&self, wallet_root: &Path, persona: PersonaId, request: PayloadRequest, scope: Vec<u8>) -> Result<PayloadResult, CustodyRefusal>;
    fn credentials(&self, call: CredentialCall) -> Result<serde_json::Value, CustodyRefusal>;
    /// Adopt Hocket's own sealed identity as a persona (D13). Hocket seals it
    /// under the desktop user's OS key, which a custodian running under its
    /// own account cannot open; adopting it here would open that store in
    /// Djinn. So only an in-process host adopts, and a custodian refuses
    /// until the import is moved into custody with its own envelope.
    fn adopt_hocket(&self) -> Result<(), String> {
        Err("Hocket's identity is adopted only by an in-process host; a custodian refuses it".into())
    }

    /// A projected signing decision. Only an in-process host with its own
    /// native confirmation answers; a custodian decides on its own channel.
    fn confirm_signing(&self, ui: &dyn NativeIdentityUi, intent: &str, payload: &[u8]) -> Result<(), String> {
        let _ = (ui, intent, payload);
        Err("signing decisions are made on the custodian's own consent channel".into())
    }
}

/// Map a custodian or local refusal code onto the custody route's refusal.
pub(crate) fn refusal(code: &str) -> CustodyRefusal {
    match code {
        "locked" => CustodyRefusal::Locked,
        "no-wallet" => CustodyRefusal::NoWallet,
        "no-persona" => CustodyRefusal::NoPersona,
        "operation-or-scope" | "operation-denied" => CustodyRefusal::NotReleasable,
        other => CustodyRefusal::Refused {
            reason: format!("the custodian refused: {other}"),
        },
    }
}
