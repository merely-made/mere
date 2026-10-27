// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! [`ResidentCustody`] over Castellan's host in this process, for the test
//! suites and receipt hosts built with `in-process-custody`. A production
//! build has no such feature and links no castellan. Wallet acts run the
//! custodian's own implementation (`djinn_custodian::wallet`), so there is
//! one copy of each. The credential catalog is custodian-only.

use std::path::Path;
use std::sync::Arc;

use castellan::authority::{PersonaeHost, VaultCommandOutput};
use castellan::custody::{IdentityStorage, UnlockMethod};
use custody_wire::contract::PayloadOperation;
use custody_wire::secret::Bytes;
use dramatis::authentication::PassphraseChangeFailureV1;
use dramatis::intents::{CreateProfileIntentV1, ImportSshKeyNativeIntentV1, SwitchProfileIntentV1};
use graphshell_endpoint::door::custody::{CustodyRefusal, EpochKeyRequest, KeySource, ReleasedEpochKey, ReleasedKey, StationGrantRequest};
use graphshell_endpoint::door::custody_payload::{PayloadBytes, PayloadOperation as AppPayload, PayloadRequest, PayloadResult};
use graphshell_endpoint::door::identity::{IdentitySurfaceSnapshot, VaultLockView};
use personae::{IdentityError, IdentityProvider, InMemoryProvider, PersonaId, ProfileId, RetainedKeys};

use super::{CredentialCall, ImportedSshKey, LockHolder, ResidentCustody, VaultOutput, refusal};
use crate::door::browser_carrier::NativeIdentityFailure;
use crate::identity_ui::NativeIdentityUi;

fn failed(error: impl std::fmt::Display) -> CustodyRefusal {
    CustodyRefusal::Failed {
        reason: error.to_string(),
    }
}

fn identity(error: IdentityError) -> CustodyRefusal {
    match error {
        IdentityError::Locked => CustodyRefusal::Locked,
        other => failed(other),
    }
}

/// A Djinn holder as Castellan's lock sees it.
struct Holder(Arc<dyn LockHolder>);

impl castellan::lock::VaultLockHolder for Holder {
    fn name(&self) -> &str {
        self.0.name()
    }

    fn key_profile(&self) -> Option<&ProfileId> {
        self.0.key_profile()
    }

    fn lock(&self) {
        self.0.lock()
    }

    fn unlock(&self, vault: &dyn IdentityProvider) -> Result<(), IdentityError> {
        self.0.unlock(vault)
    }
}

fn provider<'a, S: IdentityStorage + 'static>(host: &'a PersonaeHost<S>, source: KeySource, profile: Option<&ProfileId>, wallet_root: Option<&Path>) -> Result<Box<dyn IdentityProvider + 'a>, CustodyRefusal> {
    match source {
        KeySource::Persona => match profile {
            Some(id) if *id != host.current_profile_id() => host.profile_provider(id).map(|p| Box::new(p) as Box<dyn IdentityProvider + 'a>).map_err(identity),
            _ => Ok(Box::new(Borrowed(host))),
        },
        KeySource::Wallet => {
            let root = wallet_root.ok_or(CustodyRefusal::NoWallet)?;
            let seed = castellan::custody::wallet::load_identity_seed(root).map_err(failed)?.ok_or(CustodyRefusal::NoWallet)?;
            Ok(Box::new(InMemoryProvider::from_seed(seed)))
        },
    }
}

/// A borrowed host as a boxed provider for the length of one act.
struct Borrowed<'a, S: IdentityStorage + 'static>(&'a PersonaeHost<S>);

impl<S: IdentityStorage + 'static> IdentityProvider for Borrowed<'_, S> {
    fn master_public_key(&self) -> personae::Ed25519PublicKey {
        self.0.master_public_key()
    }

    fn derive_keypair(&self, salt: &[u8]) -> Result<personae::Ed25519Keypair, IdentityError> {
        self.0.derive_keypair(salt)
    }

    fn attest_derived_key(&self, salt: &[u8]) -> Result<insigne::DerivedKeyAttestation, IdentityError> {
        self.0.attest_derived_key(salt)
    }
}

fn wallet_refusal(code: djinn_custodian::Refusal) -> CustodyRefusal {
    refusal(code.0)
}

impl<S: IdentityStorage + 'static> ResidentCustody for PersonaeHost<S> {
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        PersonaeHost::snapshot(self)
    }

    fn is_locked(&self) -> bool {
        PersonaeHost::is_locked(self)
    }

    fn lock_state(&self) -> tokio::sync::watch::Receiver<VaultLockView> {
        PersonaeHost::lock_state(self)
    }

    fn lock_vault(&self) -> Result<(), String> {
        PersonaeHost::lock_vault(self).map_err(|e| e.to_string())
    }

    fn unlock_with_passphrase(&self, passphrase: &[u8]) -> Result<(), String> {
        self.unlock_vault(UnlockMethod::Passphrase(passphrase)).map_err(|e| e.to_string())
    }

    fn unlock_with_presence(&self, ui: &dyn NativeIdentityUi) -> Option<Result<(), String>> {
        let proof = ui.verify_presence().ok().flatten()?;
        Some(self.unlock_vault(UnlockMethod::OsPresence(proof)).map_err(|e| e.to_string()))
    }

    fn register_lock_holder(&self, holder: Arc<dyn LockHolder>) {
        PersonaeHost::register_lock_holder(self, Arc::new(Holder(holder)));
    }

    fn retained_keys(&self, salts: &[Vec<u8>]) -> Result<Arc<RetainedKeys>, IdentityError> {
        PersonaeHost::retained_keys(self, salts)
    }

    fn current_profile_id(&self) -> ProfileId {
        PersonaeHost::current_profile_id(self)
    }

    fn has_profile(&self, profile: &ProfileId) -> Result<bool, IdentityError> {
        PersonaeHost::has_profile(self, profile)
    }

    fn apply_intent(&self, intent: &str, payload: &[u8]) -> Result<(), String> {
        PersonaeHost::apply_intent(self, intent, payload).map(|_| ()).map_err(|e| e.to_string())
    }

    fn switch_profile(&self, intent: SwitchProfileIntentV1) -> Result<(), String> {
        PersonaeHost::switch_profile(self, intent).map(|_| ()).map_err(|e| e.to_string())
    }

    fn create_profile(&self, intent: CreateProfileIntentV1) -> Result<(), String> {
        PersonaeHost::create_profile(self, intent).map(|_| ()).map_err(|e| e.to_string())
    }

    fn release(&self, source: KeySource, profile: Option<&ProfileId>, salts: &[Vec<u8>]) -> Result<([u8; 32], Vec<ReleasedKey>), CustodyRefusal> {
        let root = provider(self, source, profile, self.data_root())?;
        let master = root.master_public_key().to_bytes();
        let keys = salts
            .iter()
            .map(|salt| {
                Ok(ReleasedKey {
                    seed: root.derive_keypair(salt).map_err(identity)?.to_seed(),
                    attestation: root.attest_derived_key(salt).map_err(identity)?,
                    salt: salt.clone(),
                })
            })
            .collect::<Result<_, CustodyRefusal>>()?;
        Ok((master, keys))
    }

    fn attest(&self, source: KeySource, profile: Option<&ProfileId>, salt: &[u8]) -> Result<insigne::DerivedKeyAttestation, CustodyRefusal> {
        provider(self, source, profile, self.data_root())?.attest_derived_key(salt).map_err(identity)
    }

    fn sign(&self, source: KeySource, profile: Option<&ProfileId>, salt: &[u8], message: &[u8]) -> Result<([u8; 32], Vec<u8>), CustodyRefusal> {
        let key = provider(self, source, profile, self.data_root())?.derive_keypair(salt).map_err(identity)?;
        Ok((key.public_key().to_bytes(), key.sign(message).to_bytes().to_vec()))
    }

    fn sign_delegation(&self, _audience: &str, salt: &[u8], message: &[u8]) -> Result<([u8; 32], Vec<u8>), CustodyRefusal> {
        self.sign(KeySource::Persona, None, salt, message)
    }

    fn root_public_key(&self, source: KeySource, profile: Option<&ProfileId>) -> Result<[u8; 32], CustodyRefusal> {
        provider(self, source, profile, self.data_root()).map(|p| p.master_public_key().to_bytes())
    }

    fn vault_command(&self, profile: &ProfileId, command: &str, args: &[String]) -> Result<VaultOutput, CustodyRefusal> {
        match PersonaeHost::vault_command(self, profile, command, args) {
            Ok(VaultCommandOutput::Text(text)) => Ok(VaultOutput::Text(text)),
            Ok(VaultCommandOutput::Enrollment { target, script, confirmation }) => Ok(VaultOutput::Enrollment { target, script, confirmation }),
            Err(castellan::authority::IdentityIntentError::Locked) => Err(CustodyRefusal::Locked),
            Err(other) => Err(CustodyRefusal::Refused { reason: other.to_string() }),
        }
    }

    fn change_passphrase(&self, old: &[u8], new: &[u8]) -> Result<(), PassphraseChangeFailureV1> {
        use castellan::custody::PassphraseChangeError as E;
        PersonaeHost::change_passphrase(self, old, new).map_err(|e| match e {
            E::Locked => PassphraseChangeFailureV1::Locked,
            E::Unsupported => PassphraseChangeFailureV1::Unsupported,
            E::IncorrectCurrent => PassphraseChangeFailureV1::CurrentPassphraseRejected,
            E::InvalidNew => PassphraseChangeFailureV1::InvalidNewPassphrase,
            E::Storage => PassphraseChangeFailureV1::StorageFailure,
        })
    }

    fn import_ssh_key(&self, openssh: &[u8], passphrase: Option<&[u8]>, options: ImportSshKeyNativeIntentV1) -> Result<ImportedSshKey, NativeIdentityFailure> {
        let mut private = ssh_key::PrivateKey::from_openssh(openssh).map_err(|_| NativeIdentityFailure::InvalidPrivateKey)?;
        if private.is_encrypted() {
            let passphrase = passphrase.ok_or(NativeIdentityFailure::IncorrectPassphrase)?;
            private = private.decrypt(passphrase).map_err(|_| NativeIdentityFailure::IncorrectPassphrase)?;
        }
        self.import_ssh_private(private, options).map_err(|_| NativeIdentityFailure::ImportRejected)
    }

    fn epoch_keys(&self, wallet_root: &Path, persona: PersonaId, device_label: Option<&str>, requests: &[EpochKeyRequest]) -> Result<Vec<ReleasedEpochKey>, CustodyRefusal> {
        if PersonaeHost::is_locked(self) {
            return Err(CustodyRefusal::Locked);
        }
        let wire: Vec<_> = requests
            .iter()
            .map(|r| custody_wire::EpochKeyRequest {
                context: r.context.clone(),
                device_bound: r.device_bound,
                public_only: r.public_only,
            })
            .collect();
        let keys = djinn_custodian::wallet::epoch_keys(wallet_root, persona, device_label, &wire).map_err(wallet_refusal)?;
        keys.into_iter()
            .zip(requests)
            .map(|(key, request)| {
                Ok(ReleasedEpochKey {
                    request: request.clone(),
                    key: <[u8; 32]>::try_from(key.key.expose()).map_err(failed)?,
                })
            })
            .collect()
    }

    fn station_grant(&self, wallet_root: &Path, request: &StationGrantRequest, issuer: [u8; 32]) -> Result<personae::carry::DeviceGrantSet, CustodyRefusal> {
        if PersonaeHost::is_locked(self) {
            return Err(CustodyRefusal::Locked);
        }
        let wire = djinn_custodian::wallet::StationGrantWire {
            device_id: request.device_id,
            station_ed25519_public_key: request.station_ed25519_public_key,
            label: request.label.clone(),
            issued_at_ms: request.issued_at_ms,
            expires_at_ms: request.expires_at_ms,
        };
        djinn_custodian::wallet::station_grant(wallet_root, &wire, issuer).map_err(|reason| match reason.as_str() {
            "no-wallet" => CustodyRefusal::NoWallet,
            _ => CustodyRefusal::Refused { reason },
        })
    }

    fn revoke_device(&self, wallet_root: &Path, device: uuid::Uuid) -> Result<pandect::RemoteAuthRevocationOutcome, CustodyRefusal> {
        if PersonaeHost::is_locked(self) {
            return Err(CustodyRefusal::Locked);
        }
        djinn_custodian::wallet::revoke_device(wallet_root, device).map_err(failed)
    }

    fn payload(&self, wallet_root: &Path, persona: PersonaId, request: PayloadRequest, scope: Vec<u8>) -> Result<PayloadResult, CustodyRefusal> {
        let (operation, sealing) = match request.operation {
            AppPayload::Seal { bytes } => (PayloadOperation::Seal { bytes: Bytes(bytes.0) }, true),
            AppPayload::Unseal { bytes, marker } => (
                PayloadOperation::Unseal {
                    bytes: Bytes(bytes.0),
                    marker: serde_json::to_value(marker).map_err(failed)?,
                },
                false,
            ),
        };
        let hash = request.content_hash;
        let (bytes, marker) = self
            .with_current_profile_authority(&request.profile, &request.master, || djinn_custodian::wallet::payload(wallet_root, persona, &hash, &scope, &operation))
            .map_err(identity)?
            .map_err(wallet_refusal)?;
        Ok(match (sealing, marker) {
            (true, Some(marker)) => PayloadResult::Sealed {
                bytes: PayloadBytes(bytes),
                marker: serde_json::from_value(marker).map_err(failed)?,
            },
            _ => PayloadResult::Clear { bytes: PayloadBytes(bytes) },
        })
    }

    fn credentials(&self, _call: CredentialCall) -> Result<serde_json::Value, CustodyRefusal> {
        Err(CustodyRefusal::Refused {
            reason: "the credential catalog is held by the custodian".into(),
        })
    }

    fn adopt_hocket(&self) -> Result<(), String> {
        crate::hocket_adoption::adopt(self).map(|_| ()).map_err(|error| error.to_string())
    }

    fn confirm_signing(&self, ui: &dyn NativeIdentityUi, intent: &str, payload: &[u8]) -> Result<(), String> {
        crate::keeper::signing::confirm(self, ui, intent, payload)
    }
}
