//! All roots, keys and device records here are isolated fixtures.
use castellan::custody::wallet::*;
use pandect::{wallet_grant::*, wallet_store::*};
use personae::{IdentityProvider, PersonaId, carry::revoke_device_grant_set};

#[test]
fn rotating_another_device_refreshes_private_keys_for_a_revoked_persona_certificate() {
    let root = tempfile::tempdir().unwrap();
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad51));
    let master_seed = ensure_wallet_state(root.path(), persona, "audit fixture").unwrap();
    let old_epoch = load_current_private_epoch(root.path(), persona).unwrap().unwrap();
    let mut enrolled = Vec::new();
    for n in [1u8, 2] {
        let local = LocalDeviceIdentity::new(DeviceId::from_uuid(uuid::Uuid::from_u128(0xad60+n as u128)), [n;32], "fixture".into());
        let (set, material) = issue_remote_auth_device_grant_from_pairing(root.path(), &PairedRemoteAuthGrantSpec {
            device_id: local.device_id, delegatee_pubkey: local.public_key(), label: "audit fixture".into(),
            exposure: DeviceExposure::HiddenClient, issued_at_ms: 1_770_000_000_000,
            expires_at_ms: Some(4_102_444_800_000), personas: vec![persona],
            scopes: vec!["transport.egress".into(), "identity.act".into(), "private.read".into()],
            attenuations: vec!["no-subdelegation".into()], pairing_secret: vec![0x53;16],
            private_epochs: vec![PrivateEpochPlaintext {
                persona_id: persona, epoch_id: old_epoch.epoch_id, epoch_secret: old_epoch.epoch_secret.clone(),
            }],
        }).unwrap();
        enrolled.push((local, set, material));
    }
    let (withdrawn, set, material) = &enrolled[1];
    let persona_id = set.personas[&persona].certificate.id();
    let statements: Vec<_> = revoke_device_grant_set(master_seed, set, 1_780_000_000_000).unwrap()
        .into_iter().filter(|s| s.revocation.certificate == persona_id).collect();
    assert_eq!(fold_revocations(root.path(), &statements).unwrap(), 1);
    assert_eq!(revoked_certificate_count(root.path(), set).unwrap(), 1);
    assert!(!device_is_fully_revoked(root.path(), withdrawn.device_id).unwrap(), "transport still stands");
    let master = personae::InMemoryProvider::from_seed(master_seed).master_public_key().to_bytes();
    let standing = assess_device_grant(root.path(), withdrawn.device_id, withdrawn.public_key(), master, 1_790_000_000_000).unwrap();
    assert!(standing.device.unwrap().is_ok());
    assert!(standing.personas[&persona].is_err(), "the shared trust evaluator sees the withdrawal");

    let outcome = revoke_remote_auth_device(root.path(), enrolled[0].0.device_id).unwrap();
    assert!(outcome.refreshed_devices.contains(&withdrawn.device_id), "refresh consults roster-wide status, not persona revocation");
    let new_epoch = load_current_private_epoch(root.path(), persona).unwrap().unwrap();
    assert_ne!(new_epoch.epoch_id, old_epoch.epoch_id);
    let record = load_wrapped_epoch_record(root.path(), persona_id).unwrap().unwrap();
    assert_eq!(record.epochs.len(), 1);
    let decrypted = unwrap_private_epoch_material(&record.epochs[0], persona, new_epoch.epoch_id, material.wrapping_key).unwrap();
    assert_eq!(decrypted, new_epoch.epoch_secret, "revoked persona gets the newly rotated secret");
    let bundle = build_remote_auth_enrollment_bundle(root.path(), withdrawn.device_id).unwrap();
    assert!(bundle.epochs.iter().any(|r|r == &record), "the public enrollment bundle builder exports that new carriage");
}
