// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The enrollment bundle a delegator hands a newly accepted device. The
//! install path that turns it into local wallet state stages private epochs,
//! so it is custody, in castellan (dramatis repo plan, DR-B, ruling D8).

use std::io;
use std::path::Path;

use identity::PersonaId;

use crate::wallet_store::*;

use super::*;

/// Gather the bundle a delegator hands `device_id`: its grant set, wrapped
/// epochs and persona wallets.
pub fn build_remote_auth_enrollment_bundle(
    data_root: &Path,
    device_id: DeviceId,
) -> io::Result<RemoteAuthEnrollmentBundle> {
    let roster = load_device_roster(data_root)?.unwrap_or_else(DeviceRoster::new);
    if roster.revoked.contains(&device_id) {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            format!("device {} is revoked", device_id.as_uuid()),
        ));
    }
    let grant = load_device_grant_set(data_root, device_id)?;
    if grant.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            legacy_grant_hint(data_root, device_id),
        ));
    }
    check_grant_set(&grant, "device grant certificate")?;
    let granted_personas: Vec<PersonaId> = grant.personas.keys().copied().collect();
    let mut epochs = Vec::new();
    for certificate in grant.personas.values() {
        if let Some(record) = load_wrapped_epoch_record(data_root, certificate.certificate.id())? {
            epochs.push(record);
        }
    }
    let mut persona_wallets = Vec::with_capacity(granted_personas.len());
    for &persona in &granted_personas {
        let wallet = load_persona_wallet(data_root, persona)?.ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                format!("persona wallet missing for {}", persona.as_uuid()),
            )
        })?;
        persona_wallets.push(wallet);
    }
    Ok(RemoteAuthEnrollmentBundle {
        epochs,
        schema_version: REMOTE_AUTH_ENROLLMENT_BUNDLE_SCHEMA_VERSION,
        ticket_id: None,
        grant,
        persona_wallets,
    })
}

/// Canonical CBOR bytes for a remote-auth enrollment bundle.
pub fn encode_remote_auth_enrollment_bundle(
    bundle: &RemoteAuthEnrollmentBundle,
) -> Result<Vec<u8>, EnrollmentBundleError> {
    encode_cbor(bundle).map_err(|_| EnrollmentBundleError::Encode)
}

/// Decode a remote-auth enrollment bundle from canonical CBOR bytes.
pub fn decode_remote_auth_enrollment_bundle(
    bytes: &[u8],
) -> Result<RemoteAuthEnrollmentBundle, EnrollmentBundleError> {
    decode_cbor(bytes).map_err(|_| EnrollmentBundleError::Decode)
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use uuid::Uuid;

    #[test]
    fn remote_auth_enrollment_bundle_round_trips_through_cbor() {
        let chain_root =
            crate::wallet_store::derive_persona_chain_root([21; 32], fixture_persona()).unwrap();
        let bundle = RemoteAuthEnrollmentBundle {
            schema_version: REMOTE_AUTH_ENROLLMENT_BUNDLE_SCHEMA_VERSION,
            ticket_id: Some(Uuid::from_u128(0xfeed)),
            grant: identity::carry::issue_device_grant_set(
                [21; 32],
                fixture_device(),
                DevicePublicKey::from(delegatee().public_key()),
                &["identity.act", "private.read"],
                &[fixture_persona()],
                100_000,
                1_700_000_001,
            )
            .unwrap(),
            epochs: Vec::new(),
            persona_wallets: vec![PersonaWalletManifest::new(
                fixture_persona(),
                chain_root,
                fixture_epoch(),
            )],
        };

        let bytes = encode_remote_auth_enrollment_bundle(&bundle).unwrap();
        let restored = decode_remote_auth_enrollment_bundle(&bytes).unwrap();
        assert_eq!(restored, bundle);
    }
}
