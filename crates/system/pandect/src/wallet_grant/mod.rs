// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Signed device-grant vocabulary for the wallet carry layer.
//!
//! This is the next slice after the wallet manifest store: a typed remote-auth
//! grant that lives under `identity/grants/<device_id>.cbor`, with canonical
//! CBOR bytes, a signed delegation payload, and verification helpers. Pairing
//! UX, wrapped-key generation, and revocation flow still layer on top.
//!
//! The flows that sign with the wallet's seed or touch its secrets (issuing,
//! refreshing and revoking remote-auth grants, the wrapping keys, installing an
//! enrollment bundle, re-issuing a legacy grant) moved to castellan in the
//! dramatis repo plan's DR-B (ruling D8). This module keeps the grant
//! vocabulary: types, certificates, codecs, pairing, trust and revocation
//! records, and the helpers those flows call.
//!
//! Split 2026-08-10 (wallet carry fold-in plan, W3). The envelope codec stays
//! here rather than moving into `personae::carry`: personae already owns a
//! delegation model, and a second one beside it would be duplication, not a
//! fold-in. See the plan's W3 ruling.

use std::collections::BTreeSet;
use std::io;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use uuid::Uuid;

use crate::wallet_store::{
    CarryRef, DeviceExposure, DeviceId, DeviceMode, DevicePublicKey, KeyEpochId,
    PersonaWalletManifest,
};

/// Canonical CBOR, byte for byte what p2panda-core's helpers wrote.
pub fn encode_cbor<T: Serialize>(value: &T) -> Result<Vec<u8>, ciborium::ser::Error<io::Error>> {
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(value, &mut bytes)?;
    Ok(bytes)
}

/// Decode canonical CBOR.
pub fn decode_cbor<T: serde::de::DeserializeOwned>(
    reader: impl io::Read,
) -> Result<T, ciborium::de::Error<io::Error>> {
    ciborium::from_reader(reader)
}

mod certificate;
mod enroll;
mod epochs;
mod errors;
mod migrate;
mod pairing;
mod records;
mod revocation;
#[cfg(test)]
mod test_support;
mod trust;
mod types;
mod validate;

/// Current schema version for typed device grants.
pub const DEVICE_GRANT_SCHEMA_VERSION: u32 = 1;
/// Current wrap format for private-epoch material in remote-auth grants.
pub const WRAPPED_PRIVATE_EPOCH_FORMAT_V1: &str = "xchacha20poly1305-v1";
/// Pairing transcript context for deriving the remote-auth wrapping key.
pub const REMOTE_AUTH_PAIRING_WRAP_CONTEXT_V1: &str = "mere.wallet.remote-auth.wrap.v1";
/// Pairing transcript context for deriving the short auth string.
pub const REMOTE_AUTH_PAIRING_SAS_CONTEXT_V1: &str = "mere.wallet.remote-auth.sas.v1";
/// Schema version for QR / code transported remote-auth pairing tickets.
pub const REMOTE_AUTH_PAIRING_TICKET_SCHEMA_VERSION: u32 = 1;
/// Random secret bytes carried by a remote-auth pairing ticket.
pub const REMOTE_AUTH_PAIRING_SECRET_LEN: usize = 16;
/// Schema version for a typed remote-auth enrollment bundle.
pub const REMOTE_AUTH_ENROLLMENT_BUNDLE_SCHEMA_VERSION: u32 = 1;

/// Milliseconds since the Unix epoch, from the system clock.
pub fn unix_time_ms() -> io::Result<u64> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|err| io::Error::other(format!("system clock before unix epoch: {err}")))?;
    u64::try_from(now.as_millis()).map_err(|_| io::Error::other("unix time overflowed u64"))
}

/// Whether `expires_at_ms` has passed at `now_ms`.
pub fn is_expired(expires_at_ms: Option<u64>, now_ms: u64) -> bool {
    matches!(expires_at_ms, Some(expires_at_ms) if expires_at_ms <= now_ms)
}

/// Check every certificate in a grant set, keeping the first fault in the
/// error so a refusal says which step failed.
pub fn check_grant_set(set: &identity::carry::DeviceGrantSet, context: &str) -> io::Result<()> {
    for certificate in set.certificates() {
        certificate.check().map_err(|fault| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("{context} failed signature verification: {fault}"),
            )
        })?;
    }
    Ok(())
}

/// The capability-slot id a remote-auth device occupies in a persona wallet.
///
/// The `device-grant:` prefix is a persisted key: it is matched when slots are
/// upserted and when a revocation clears them, so changing it would orphan
/// every slot already written.
pub fn remote_auth_capability_slot_id(device_id: DeviceId) -> String {
    format!("device-grant:{}", device_id.as_uuid())
}

pub use epochs::wrapped_epoch_aad;
pub use pairing::{derive_pairing_key_from_transcript, remote_auth_pairing_transcript};
pub use records::{
    upsert_grant_index, upsert_local_remote_auth_record, upsert_remote_auth_device_record,
};
pub use validate::{
    validate_paired_remote_auth_spec, validate_remote_auth_enrollment_bundle,
    validate_remote_auth_spec,
};

pub use certificate::{
    WrappedEpochRecord, certificate_device_id, check_epoch_carriage, decode_certificate,
    decode_device_grant_set, decode_epoch_record, device_certificate_path, device_grant_set_ref,
    device_scope_certificate_path, encode_certificate, encode_device_grant_set,
    encode_epoch_record, load_device_certificate, load_device_grant_set, load_wrapped_epoch_record,
    remove_wrapped_epoch_record, requires_epoch_material, save_device_certificate,
    save_device_grant_set, save_wrapped_epoch_record, wrapped_epoch_record_path,
};
pub use enroll::{
    build_remote_auth_enrollment_bundle, decode_remote_auth_enrollment_bundle,
    encode_remote_auth_enrollment_bundle,
};
pub use epochs::{
    BlindedEpochIndex, BlindedSlotId, blinded_epoch_index, blinded_slot_id,
    unwrap_private_epoch_material, wrap_private_epoch_material,
};
pub use errors::{
    DeviceGrantError, EnrollmentBundleError, PairingCodeError, PairingMaterialError,
    PairingTicketError, WrappedEpochError,
};
pub use migrate::{LegacyGrant, legacy_grant_hint, retire_legacy_grant, survey_legacy_grants};
pub use pairing::{
    decode_remote_auth_pairing_ticket, derive_remote_auth_pairing_material,
    encode_remote_auth_pairing_ticket, format_remote_auth_pairing_code,
    mint_remote_auth_pairing_ticket, parse_remote_auth_pairing_code,
};
pub use revocation::{
    device_is_fully_revoked, fold_revocations, load_revocation_ledger, revocation_ledger_path,
    revoke_device_certificates, revoked_certificate_count, save_revocation_ledger,
};
pub use trust::{GrantStanding, assess_device_grant, wallet_trusted_roots};
pub use types::{
    EpochCarriage, PairedRemoteAuthGrantSpec, PrivateEpochPlaintext, RemoteAuthEnrollmentBundle,
    RemoteAuthGrantSpec, RemoteAuthPairingMaterial, RemoteAuthPairingResponse,
    RemoteAuthPairingTicket, RemoteAuthPairingTicketRequest, RemoteAuthRevocationOutcome,
    WrappedEpochMaterial,
};
