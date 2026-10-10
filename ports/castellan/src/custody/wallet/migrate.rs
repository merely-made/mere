// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Re-issuing a legacy grant as a certificate set: custody, moved from
//! pandect's `wallet_grant::migrate` (dramatis repo plan, DR-B, D8), whose
//! survey supplies what is re-issued.

use std::io;
use std::path::Path;

use personae::carry::DeviceGrantSet;

use super::*;

/// Re-issue one legacy grant as a certificate set.
///
/// `scopes` is not optional and has no default on purpose. It is the one part
/// of the old grant this migration cannot recover, and guessing it would
/// either widen a device's authority or narrow it silently. The caller states
/// what the device is for; everything else comes from the survey.
pub fn reissue_legacy_grant(
    data_root: &Path,
    legacy: &LegacyGrant,
    scopes: &[String],
    issued_at_ms: u64,
    expires_at_ms: u64,
) -> io::Result<DeviceGrantSet> {
    if scopes.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "re-issuing the grant for device {} needs its scopes restated; \
                 the old payload is deliberately not decoded",
                legacy.device_id.as_uuid()
            ),
        ));
    }
    if legacy.mode != DeviceMode::RemoteAuth {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "device {} is enrolled as {:?}; only RemoteAuth devices carry re-issuable grants",
                legacy.device_id.as_uuid(),
                legacy.mode
            ),
        ));
    }
    issue_remote_auth_device_grant(
        data_root,
        &RemoteAuthGrantSpec {
            device_id: legacy.device_id,
            delegatee_pubkey: legacy.holder,
            label: legacy.label.clone(),
            exposure: legacy.exposure,
            issued_at_ms,
            expires_at_ms: Some(expires_at_ms),
            personas: legacy.personas.clone(),
            scopes: scopes.to_vec(),
            attenuations: Vec::new(),
            // Private-lane material cannot be recovered either: it was wrapped
            // to a pairing key inside the payload. A device that needs it is
            // re-paired, not re-issued.
            wrapped_private_epochs: Vec::new(),
        },
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_support::*;
    use super::*;
    use pandect::wallet_store::save_device_grant;

    fn stage_legacy(root: &Path, device_id: DeviceId) {
        // Opaque bytes: the point is that nothing ever decodes them.
        save_device_grant(root, device_id, b"pre-certificate grant bytes").unwrap();
    }

    #[test]
    fn a_wallet_with_no_roster_surveys_clean() {
        let root = tempfile::tempdir().unwrap();
        assert!(survey_legacy_grants(root.path()).unwrap().is_empty());
    }

    #[test]
    fn a_legacy_grant_is_found_and_reads_as_stranded() {
        let root = temp_data_root("m3-survey");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();
        let spec = sample_remote_auth_spec();
        issue_remote_auth_device_grant(&root, &spec).unwrap();
        // Simulate the pre-migration state: the stale file beside the new set.
        stage_legacy(&root, spec.device_id);

        let found = survey_legacy_grants(&root).unwrap();
        let entry = found
            .iter()
            .find(|entry| entry.device_id == spec.device_id)
            .expect("the legacy grant should be surveyed");
        assert_eq!(entry.label, spec.label);
        assert_eq!(entry.personas, vec![fixture_persona()]);
        // Certificates exist here, so it is residue rather than a stranding.
        assert!(entry.reissued);
        assert!(!entry.is_stranded());
    }

    /// The consequence of the no-decoder posture, asserted rather than
    /// described: the scopes must be restated, and omitting them fails loudly
    /// instead of minting a narrower grant.
    #[test]
    fn re_issuing_without_restated_scopes_is_refused() {
        let root = temp_data_root("m3-scopes");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();
        let legacy = LegacyGrant {
            device_id: fixture_device(),
            label: "Pocket relay".into(),
            mode: DeviceMode::RemoteAuth,
            exposure: DeviceExposure::ExposedEgress,
            holder: DevicePublicKey::from(delegatee().public_key()),
            personas: vec![fixture_persona()],
            reissued: false,
        };

        let error = reissue_legacy_grant(&root, &legacy, &[], 1_700_000_001, 1_800_000_001)
            .expect_err("empty scopes must be refused");
        assert_eq!(error.kind(), io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("scopes restated"));
    }

    #[test]
    fn a_copy_mode_device_is_not_re_issuable() {
        let root = temp_data_root("m3-copy");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();
        let legacy = LegacyGrant {
            device_id: fixture_device(),
            label: "Laptop clone".into(),
            mode: DeviceMode::Copy,
            exposure: DeviceExposure::HiddenClient,
            holder: DevicePublicKey::from(delegatee().public_key()),
            personas: vec![fixture_persona()],
            reissued: false,
        };

        let error = reissue_legacy_grant(
            &root,
            &legacy,
            &["transport.egress".to_string()],
            1_700_000_001,
            1_800_000_001,
        )
        .expect_err("a Copy device has no delegated grant to re-issue");
        assert!(error.to_string().contains("only RemoteAuth"));
    }

    #[test]
    fn re_issuing_restores_a_stranded_device() {
        let root = temp_data_root("m3-reissue");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();
        let legacy = LegacyGrant {
            device_id: fixture_device(),
            label: "Pocket relay".into(),
            mode: DeviceMode::RemoteAuth,
            exposure: DeviceExposure::ExposedEgress,
            holder: DevicePublicKey::from(delegatee().public_key()),
            personas: Vec::new(),
            reissued: false,
        };

        let set = reissue_legacy_grant(
            &root,
            &legacy,
            &["transport.egress".to_string()],
            1_700_000_001,
            1_800_000_001,
        )
        .unwrap();

        assert!(set.device.is_some());
        assert!(
            !load_device_grant_set(&root, fixture_device())
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn a_stranded_device_keeps_its_legacy_file() {
        let root = temp_data_root("m3-retire-guard");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();
        stage_legacy(&root, fixture_device());

        let error = retire_legacy_grant(&root, fixture_device())
            .expect_err("retiring before re-issue must be refused");
        assert!(error.to_string().contains("no certificates"));
        assert!(device_grant_path(&root, fixture_device()).exists());
    }

    #[test]
    fn retiring_removes_the_file_once_certificates_exist() {
        let root = temp_data_root("m3-retire");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();
        let spec = sample_remote_auth_spec();
        issue_remote_auth_device_grant(&root, &spec).unwrap();
        stage_legacy(&root, spec.device_id);

        assert!(retire_legacy_grant(&root, spec.device_id).unwrap());
        assert!(!device_grant_path(&root, spec.device_id).exists());
        // Retiring residue must not disturb the authority that replaced it.
        assert!(
            !load_device_grant_set(&root, spec.device_id)
                .unwrap()
                .is_empty()
        );
    }
}
