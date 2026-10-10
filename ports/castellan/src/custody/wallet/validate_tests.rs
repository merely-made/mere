// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! pandect's `wallet_grant::validate` tests. They issue grants with the
//! wallet's seed, custody since the dramatis repo plan's DR-B, so they moved
//! with it.

mod tests {
    use super::super::test_support::*;
    use super::super::*;
    use std::io;

    #[test]
    fn issue_remote_auth_device_grant_rejects_unknown_persona_wallet() {
        let root = temp_data_root("remote-auth-missing-persona");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();

        let mut spec = sample_remote_auth_spec();
        spec.personas.push(second_persona());
        let err = issue_remote_auth_device_grant(&root, &spec).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn issue_remote_auth_device_grant_rejects_wrapped_epoch_outside_persona_set() {
        let root = temp_data_root("remote-auth-mismatch");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();

        let mut spec = sample_remote_auth_spec();
        spec.wrapped_private_epochs.push(EpochCarriage {
            persona_id: second_persona(),
            material: WrappedEpochMaterial {
                index: blinded_epoch_index(second_persona(), fixture_epoch(), FIXTURE_WRAPPING_KEY),
                wrap_format: "xchacha20poly1305-v1".into(),
                wrapped_key: vec![0xca, 0xfe],
            },
        });
        let err = issue_remote_auth_device_grant(&root, &spec).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn issue_remote_auth_device_grant_rejects_private_read_without_wrapped_epoch() {
        let root = temp_data_root("remote-auth-missing-wrap");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();

        let mut spec = sample_remote_auth_spec();
        spec.wrapped_private_epochs.clear();
        let err = issue_remote_auth_device_grant(&root, &spec).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn private_read_free_grant_can_skip_wrapped_epoch_material() {
        let root = temp_data_root("remote-auth-no-private");
        crate::custody::wallet::ensure_wallet_state(&root, fixture_persona(), "Studio PC").unwrap();

        let mut spec = sample_remote_auth_spec();
        spec.scopes = vec!["identity.act".into(), "transport.egress".into()];
        spec.wrapped_private_epochs.clear();
        let grant = issue_remote_auth_device_grant(&root, &spec).unwrap();
        assert!(stored_epochs_for(&root, &grant, fixture_persona()).is_empty());

        let _ = std::fs::remove_dir_all(&root);
    }
}
