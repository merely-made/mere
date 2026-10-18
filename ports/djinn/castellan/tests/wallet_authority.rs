// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#![cfg(feature = "wallet")]

mod enrollment {
    //! Regression coverage of public custody flows. All keys are temp fixtures.
    use castellan::custody::wallet::*;
    use pandect::{wallet_grant::*, wallet_store::*};
    use personae::PersonaId;

    fn spec(device: &LocalDeviceIdentity, persona: PersonaId) -> RemoteAuthGrantSpec {
        RemoteAuthGrantSpec {
            device_id: device.device_id,
            delegatee_pubkey: device.public_key(),
            label: "audit fixture".into(),
            exposure: DeviceExposure::HiddenClient,
            issued_at_ms: 1_770_000_000_000,
            expires_at_ms: Some(4_102_444_800_000),
            personas: vec![persona],
            scopes: vec!["identity.act".into()],
            attenuations: vec!["no-subdelegation".into()],
            wrapped_private_epochs: vec![],
        }
    }

    #[test]
    fn successful_narrower_issuance_round_trips_the_stored_set_and_refs() {
        let root = tempfile::tempdir().unwrap();
        let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad31));
        ensure_wallet_state(root.path(), persona, "audit fixture").unwrap();
        let local = LocalDeviceIdentity::new(
            DeviceId::from_uuid(uuid::Uuid::from_u128(0xad32)),
            [0x3b; 32],
            "fixture".into(),
        );
        let mut request = spec(&local, persona);
        request.scopes.push("transport.egress".into());
        issue_remote_auth_device_grant(root.path(), &request).unwrap();
        request.personas.clear();
        request.scopes = vec!["transport.egress".into()];
        let issued = issue_remote_auth_device_grant(root.path(), &request).unwrap();
        assert!(issued.personas.is_empty());
        let restored = load_device_grant_set(root.path(), local.device_id).unwrap();
        assert_eq!(restored, issued);
        let expected_ref = device_grant_set_ref(&issued);
        assert_eq!(device_grant_set_ref(&restored), expected_ref);
        let roster = load_device_roster(root.path()).unwrap().unwrap();
        assert_eq!(
            roster
                .devices
                .iter()
                .find(|d| d.device_id == local.device_id)
                .unwrap()
                .grant_ref,
            Some(expected_ref)
        );
        let wallet = load_identity_wallet(root.path()).unwrap().unwrap();
        assert_eq!(
            wallet
                .grant_index
                .iter()
                .find(|d| d.device_id == local.device_id)
                .unwrap()
                .grant_ref,
            Some(expected_ref)
        );
        let empty = personae::carry::DeviceGrantSet {
            device: None,
            personas: Default::default(),
        };
        assert_eq!(
            save_device_grant_set(root.path(), local.device_id, &empty).unwrap(),
            device_grant_set_ref(&empty)
        );
        assert!(
            load_device_grant_set(root.path(), local.device_id)
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn invalid_wrapping_keys_refuse_before_persisting_the_enrollment_payload() {
        let issuer = tempfile::tempdir().unwrap();
        let recipient = tempfile::tempdir().unwrap();
        let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad41));
        ensure_wallet_state(issuer.path(), persona, "audit fixture").unwrap();
        let local = ensure_local_device_identity(recipient.path(), "audit fixture").unwrap();
        let mut request = spec(&local, persona);
        request.scopes.push("private.read".into());
        let epoch = load_current_private_epoch(issuer.path(), persona)
            .unwrap()
            .unwrap();
        let wrapping_key = [0x46; 32];
        request.wrapped_private_epochs.push(EpochCarriage {
            persona_id: persona,
            material: wrap_private_epoch_material(
                persona,
                epoch.epoch_id,
                &epoch.epoch_secret,
                wrapping_key,
            )
            .unwrap(),
        });
        issue_remote_auth_device_grant(issuer.path(), &request).unwrap();
        let bundle = build_remote_auth_enrollment_bundle(issuer.path(), local.device_id).unwrap();
        assert!(
            load_device_grant_set(recipient.path(), local.device_id)
                .unwrap()
                .is_empty()
        );
        assert!(
            load_persona_wallet(recipient.path(), persona)
                .unwrap()
                .is_none()
        );
        let error = install_remote_auth_enrollment_bundle(recipient.path(), &bundle).unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(error.to_string().contains("wrapping key"));
        let error = install_remote_auth_enrollment_bundle_with_wrapping_key(
            recipient.path(),
            &bundle,
            [0x47; 32],
        )
        .unwrap_err();
        assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
        assert!(
            load_device_grant_set(recipient.path(), local.device_id)
                .unwrap()
                .is_empty()
        );
        assert!(
            load_persona_wallet(recipient.path(), persona)
                .unwrap()
                .is_none()
        );
        assert!(load_device_roster(recipient.path()).unwrap().is_none());
        assert!(load_identity_wallet(recipient.path()).unwrap().is_none());
        assert!(
            load_current_private_epoch(recipient.path(), persona)
                .unwrap()
                .is_none()
        );
        // Positive control: supplying the required key finishes the installation.
        install_remote_auth_enrollment_bundle_with_wrapping_key(
            recipient.path(),
            &bundle,
            wrapping_key,
        )
        .unwrap();
        assert_eq!(
            load_current_private_epoch(recipient.path(), persona)
                .unwrap()
                .unwrap(),
            epoch
        );
    }
}
mod revocation {
    //! All roots, keys and device records here are isolated fixtures.
    use castellan::custody::wallet::*;
    use pandect::{wallet_grant::*, wallet_store::*};
    use personae::{IdentityProvider, PersonaId, carry::revoke_device_grant_set};

    #[test]
    fn rotating_another_device_withholds_private_keys_for_a_revoked_persona_certificate() {
        let root = tempfile::tempdir().unwrap();
        let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad51));
        let master_seed = ensure_wallet_state(root.path(), persona, "audit fixture").unwrap();
        let old_epoch = load_current_private_epoch(root.path(), persona)
            .unwrap()
            .unwrap();
        let mut enrolled = Vec::new();
        for n in [1u8, 2] {
            let local = LocalDeviceIdentity::new(
                DeviceId::from_uuid(uuid::Uuid::from_u128(0xad60 + n as u128)),
                [n; 32],
                "fixture".into(),
            );
            let (set, material) = issue_remote_auth_device_grant_from_pairing(
                root.path(),
                &PairedRemoteAuthGrantSpec {
                    device_id: local.device_id,
                    delegatee_pubkey: local.public_key(),
                    label: "audit fixture".into(),
                    exposure: DeviceExposure::HiddenClient,
                    issued_at_ms: 1_770_000_000_000,
                    expires_at_ms: Some(4_102_444_800_000),
                    personas: vec![persona],
                    scopes: vec![
                        "transport.egress".into(),
                        "identity.act".into(),
                        "private.read".into(),
                    ],
                    attenuations: vec!["no-subdelegation".into()],
                    pairing_secret: vec![0x53; 16],
                    private_epochs: vec![PrivateEpochPlaintext {
                        persona_id: persona,
                        epoch_id: old_epoch.epoch_id,
                        epoch_secret: old_epoch.epoch_secret.clone(),
                    }],
                },
            )
            .unwrap();
            enrolled.push((local, set, material));
        }
        let (withdrawn, set, material) = &enrolled[1];
        let persona_id = set.personas[&persona].certificate.id();
        let statements: Vec<_> = revoke_device_grant_set(master_seed, set, 1_780_000_000_000)
            .unwrap()
            .into_iter()
            .filter(|s| s.revocation.certificate == persona_id)
            .collect();
        assert_eq!(fold_revocations(root.path(), &statements).unwrap(), 1);
        assert_eq!(revoked_certificate_count(root.path(), set).unwrap(), 1);
        assert!(
            !device_is_fully_revoked(root.path(), withdrawn.device_id).unwrap(),
            "transport still stands"
        );
        let master = personae::InMemoryProvider::from_seed(master_seed)
            .master_public_key()
            .to_bytes();
        let standing = assess_device_grant(
            root.path(),
            withdrawn.device_id,
            withdrawn.public_key(),
            master,
            1_790_000_000_000,
        )
        .unwrap();
        assert!(standing.device.unwrap().is_ok());
        assert!(
            standing.personas[&persona].is_err(),
            "the shared trust evaluator sees the withdrawal"
        );

        let previous_record = load_wrapped_epoch_record(root.path(), persona_id)
            .unwrap()
            .unwrap();
        let outcome = revoke_remote_auth_device(root.path(), enrolled[0].0.device_id).unwrap();
        assert!(!outcome.refreshed_devices.contains(&withdrawn.device_id));
        let new_epoch = load_current_private_epoch(root.path(), persona)
            .unwrap()
            .unwrap();
        assert_ne!(new_epoch.epoch_id, old_epoch.epoch_id);
        let record = load_wrapped_epoch_record(root.path(), persona_id)
            .unwrap()
            .unwrap();
        assert_eq!(record, previous_record);
        assert_eq!(record.epochs.len(), 1);
        let decrypted = unwrap_private_epoch_material(
            &record.epochs[0],
            persona,
            old_epoch.epoch_id,
            material.wrapping_key,
        )
        .unwrap();
        assert_eq!(decrypted, old_epoch.epoch_secret);
        let bundle = build_remote_auth_enrollment_bundle(root.path(), withdrawn.device_id).unwrap();
        assert!(!bundle.grant.personas.contains_key(&persona));
        assert!(!bundle.epochs.iter().any(|r| r == &record));
    }
}
mod interruptions {
    //! Windows fixture-only diagnostics of interrupted custody and legacy reading.
    #![cfg(windows)]
    use castellan::custody::wallet::*;
    use pandect::{wallet_grant::*, wallet_store::*};
    use personae::PersonaId;

    #[test]
    fn retry_after_refused_persona_write_rotates_the_exposed_epoch() {
        use std::os::windows::fs::OpenOptionsExt;
        let root = tempfile::tempdir().unwrap();
        let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad71));
        ensure_wallet_state(root.path(), persona, "audit fixture").unwrap();
        let old_epoch = load_current_private_epoch(root.path(), persona)
            .unwrap()
            .unwrap();
        let local = LocalDeviceIdentity::new(
            DeviceId::from_uuid(uuid::Uuid::from_u128(0xad72)),
            [0x74; 32],
            "fixture".into(),
        );
        let set = issue_remote_auth_device_grant(
            root.path(),
            &RemoteAuthGrantSpec {
                device_id: local.device_id,
                delegatee_pubkey: local.public_key(),
                label: "fixture".into(),
                exposure: DeviceExposure::HiddenClient,
                issued_at_ms: 1_770_000_000_000,
                expires_at_ms: Some(4_102_444_800_000),
                personas: vec![persona],
                scopes: vec!["identity.act".into(), "private.read".into()],
                attenuations: vec![],
                wrapped_private_epochs: vec![EpochCarriage {
                    persona_id: persona,
                    material: wrap_private_epoch_material(
                        persona,
                        old_epoch.epoch_id,
                        &old_epoch.epoch_secret,
                        [0x78; 32],
                    )
                    .unwrap(),
                }],
            },
        )
        .unwrap();
        let certificate = set.personas[&persona].certificate.id();
        let path = persona_wallet_path(root.path(), persona);
        let before = std::fs::read(&path).unwrap();
        // The persona wallet itself is held open with no sharing. Pandect's shared
        // atomic writer (atomic_file, since fc5d2f7e) renames a fresh temporary
        // onto the target, so this is refused before any byte lands. The earlier
        // obstacle, a locked `.json.tmp`, no longer meets that write path.
        let obstacle = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0)
            .open(&path)
            .unwrap();
        assert!(revoke_remote_auth_device(root.path(), local.device_id).is_err());
        drop(obstacle);
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(
            load_wrapped_epoch_record(root.path(), certificate)
                .unwrap()
                .is_some(),
            "retry trigger survives refused write"
        );
        // The refused rename leaves its temporary behind; clear it so the retry
        // meets the directory the way a cleaned-up process would.
        let wallet_name = path.file_name().unwrap().to_os_string();
        for entry in std::fs::read_dir(path.parent().unwrap()).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name();
            if name != wallet_name
                && name.to_string_lossy().starts_with(&*wallet_name.to_string_lossy())
            {
                std::fs::remove_file(entry.path()).unwrap();
            }
        }
        let retry = revoke_remote_auth_device(root.path(), local.device_id).unwrap();
        assert!(retry.already_revoked);
        assert_eq!(retry.rotated_personas, vec![persona]);
        assert!(device_is_fully_revoked(root.path(), local.device_id).unwrap());
        let current = load_current_private_epoch(root.path(), persona)
            .unwrap()
            .unwrap();
        assert_ne!(current.epoch_id, old_epoch.epoch_id);
        assert_ne!(current.epoch_secret, old_epoch.epoch_secret);
    }

    #[test]
    fn legacy_read_only_seed_load_preserves_raw_bytes_with_an_existing_auto_unlock_root() {
        let root = tempfile::tempdir().unwrap();
        let legacy = [0x6a; 32];
        let path = identity_seed_path(root.path());
        save_bytes_atomic(&path, &legacy).unwrap();
        assert_eq!(
            load_identity_seed_read_only(root.path()).unwrap(),
            Some(legacy),
            "same fresh-root control as the existing unit test"
        );
        save_identity_seed(root.path(), [0x5b; 32]).unwrap();
        assert!(identity_auto_unlock_root_path(root.path()).is_file());
        std::fs::write(&path, legacy).unwrap();
        assert_eq!(
            load_identity_seed_read_only(root.path()).unwrap(),
            Some(legacy)
        );
        assert_eq!(
            std::fs::read(&path).unwrap(),
            legacy,
            "failed read leaves original raw bytes recoverable"
        );
        assert_eq!(
            load_identity_seed(root.path()).unwrap(),
            Some(legacy),
            "ordinary loader supports and migrates the same bytes"
        );
    }
}
