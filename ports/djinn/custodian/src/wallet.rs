// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The wallet's custody acts over a data root in Djinn's layout, moved here
//! from Djinn's custody route unchanged in effect: private-epoch key
//! derivation, sited station grants, device revocation and retained-payload
//! sealing. Callers own lock and profile admission; nothing returned is an
//! epoch, a seed or a wrapping key.

use std::path::Path;

mod adopt;
pub use adopt::{Adopted, adopt_files, carry_files, retire_files, wallet_files};

use custody_wire::contract::{EpochKeyRequest, PayloadOperation, ReleasedEpochKey};
use custody_wire::secret::Secret;
use personae::{Ed25519Keypair, IdentityProvider, InMemoryProvider, PersonaId};
use zeroize::Zeroizing;

use crate::Refusal;

fn storage(_: impl std::fmt::Display) -> Refusal {
    Refusal("vault-storage")
}

/// The wallet's public records: the persona index (the UUIDs that hold a
/// wallet manifest, sorted) and the device roster as stored. Neither holds
/// a seed, an epoch or a device secret, and neither needs the vault open.
pub fn public_records(root: &Path) -> Result<(Vec<String>, Option<serde_json::Value>), Refusal> {
    let personas = pandect::wallet_store::list_personas(root).map_err(storage)?;
    let roster = pandect::wallet_store::load_device_roster(root).map_err(storage)?;
    Ok((
        personas.iter().map(|persona| persona.as_uuid().to_string()).collect(),
        roster.map(|roster| serde_json::to_value(roster)).transpose().map_err(storage)?,
    ))
}

/// Keys derived from `persona`'s current private epoch: blake3's
/// `derive_key` under each context over the epoch secret, followed by this
/// device's public root when device-bound. The epoch never leaves.
pub fn epoch_keys(root: &Path, persona: PersonaId, device_label: Option<&str>, requests: &[EpochKeyRequest]) -> Result<Vec<ReleasedEpochKey>, Refusal> {
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

/// A sited station's grant request, as plain fields.
#[derive(serde::Serialize, serde::Deserialize)]
pub struct StationGrantWire {
    pub device_id: uuid::Uuid,
    pub station_ed25519_public_key: [u8; 32],
    pub label: String,
    pub issued_at_ms: u64,
    pub expires_at_ms: u64,
}

/// Issue a sited station's narrow RemoteAuth grant with the wallet seed.
pub fn station_grant(root: &Path, request: &StationGrantWire, issuer: [u8; 32]) -> Result<personae::carry::DeviceGrantSet, String> {
    use pandect::station_grant::{SitedStationGrant, SitedStationGrantRequest};
    let seed = castellan::custody::wallet::load_identity_seed(root)
        .map_err(|e| e.to_string())?
        .ok_or("no-wallet")?;
    let wallet_public_key = InMemoryProvider::from_seed(seed).master_public_key().to_bytes();
    let request = SitedStationGrantRequest::new(
        pandect::DeviceId::from_uuid(request.device_id),
        request.station_ed25519_public_key,
        request.label.clone(),
        request.issued_at_ms,
        request.expires_at_ms,
    )
    .map_err(|error| error.to_string())?;
    let grant = SitedStationGrant::issue_with(root, issuer, wallet_public_key, request, |spec| {
        castellan::custody::wallet::issue_remote_auth_device_grant(root, spec)
    })
    .map_err(|error| error.to_string())?;
    Ok(grant.signed().clone())
}

/// Revoke one delegated device in the wallet.
pub fn revoke_device(root: &Path, device: uuid::Uuid) -> Result<pandect::RemoteAuthRevocationOutcome, String> {
    castellan::custody::wallet::revoke_remote_auth_device(root, pandect::DeviceId::from_uuid(device)).map_err(|error| error.to_string())
}

/// Seal or unseal one retained payload under `persona`'s wallet epoch, bound
/// to `scope`. The caller has admitted the profile and checked the lock.
pub fn payload(root: &Path, persona: PersonaId, hash: &eidetic::Hash, scope: &[u8], operation: &PayloadOperation) -> Result<(Vec<u8>, Option<serde_json::Value>), Refusal> {
    use eidetic::PayloadSealer;
    match operation {
        PayloadOperation::Seal { bytes } => {
            let sealer = castellan::custody::wallet::epoch_sealer_for_persona(root, persona)
                .map_err(storage)?
                .ok_or(Refusal("locked"))?
                .with_scope(scope.to_vec());
            let (sealed, marker) = sealer.seal(hash, &bytes.0).map_err(storage)?;
            Ok((sealed, Some(serde_json::to_value(marker).map_err(storage)?)))
        },
        PayloadOperation::Unseal { bytes, marker } => {
            let marker: eidetic::SealedBlobRef = serde_json::from_value(marker.clone()).map_err(|_| Refusal("operation-or-scope"))?;
            let clear = castellan::custody::wallet::unseal_persona_payload(root, persona, hash, scope, &marker, &bytes.0)
                .map_err(storage)?
                .ok_or(Refusal("locked"))?;
            if eidetic::Hash::of(&clear) != *hash {
                return Err(Refusal("payload-hash-differs"));
            }
            Ok((clear, None))
        },
    }
}
