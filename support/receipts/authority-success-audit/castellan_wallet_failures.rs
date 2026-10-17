//! Windows fixture-only diagnostics of interrupted custody and legacy reading.
#![cfg(windows)]
use castellan::custody::wallet::*;
use pandect::{wallet_grant::*, wallet_store::*};
use personae::PersonaId;

#[test]
fn retry_after_refused_persona_write_reports_revoked_without_rotating_the_exposed_epoch() {
    use std::os::windows::fs::OpenOptionsExt;
    let root = tempfile::tempdir().unwrap();
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad71));
    ensure_wallet_state(root.path(), persona, "audit fixture").unwrap();
    let old_epoch = load_current_private_epoch(root.path(), persona).unwrap().unwrap();
    let local = LocalDeviceIdentity::new(DeviceId::from_uuid(uuid::Uuid::from_u128(0xad72)), [0x74;32], "fixture".into());
    let set = issue_remote_auth_device_grant(root.path(), &RemoteAuthGrantSpec {
        device_id: local.device_id, delegatee_pubkey: local.public_key(), label: "fixture".into(),
        exposure: DeviceExposure::HiddenClient, issued_at_ms: 1_770_000_000_000,
        expires_at_ms: Some(4_102_444_800_000), personas: vec![persona],
        scopes: vec!["identity.act".into(), "private.read".into()], attenuations: vec![],
        wrapped_private_epochs: vec![EpochCarriage {
            persona_id: persona,
            material: wrap_private_epoch_material(persona, old_epoch.epoch_id, &old_epoch.epoch_secret, [0x78;32]).unwrap(),
        }],
    }).unwrap();
    let certificate = set.personas[&persona].certificate.id();
    let path = persona_wallet_path(root.path(), persona);
    let before = std::fs::read(&path).unwrap();
    let temporary = path.with_extension("json.tmp");
    std::fs::write(&temporary, b"fixture obstacle").unwrap();
    let obstacle = std::fs::OpenOptions::new().read(true).share_mode(0).open(&temporary).unwrap();
    assert!(revoke_remote_auth_device(root.path(), local.device_id).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), before);
    assert!(load_wrapped_epoch_record(root.path(), certificate).unwrap().is_none(), "carriage deleted before refused write");
    drop(obstacle);
    std::fs::remove_file(&temporary).unwrap();
    let retry = revoke_remote_auth_device(root.path(), local.device_id).unwrap();
    assert!(retry.already_revoked);
    assert!(retry.rotated_personas.is_empty());
    assert!(device_is_fully_revoked(root.path(), local.device_id).unwrap());
    assert_eq!(load_current_private_epoch(root.path(), persona).unwrap().unwrap(), old_epoch, "the withdrawn device's known secret is still the current epoch after successful retry");
}

#[test]
fn legacy_read_only_seed_load_fails_only_when_an_auto_unlock_root_already_exists() {
    let root = tempfile::tempdir().unwrap();
    let legacy = [0x6a; 32];
    let path = identity_seed_path(root.path());
    save_bytes_atomic(&path, &legacy).unwrap();
    assert_eq!(load_identity_seed_read_only(root.path()).unwrap(), Some(legacy), "same fresh-root control as the existing unit test");
    save_identity_seed(root.path(), [0x5b;32]).unwrap();
    assert!(identity_auto_unlock_root_path(root.path()).is_file());
    std::fs::write(&path, legacy).unwrap();
    assert!(load_identity_seed_read_only(root.path()).is_err());
    assert_eq!(std::fs::read(&path).unwrap(), legacy, "failed read leaves original raw bytes recoverable");
    assert_eq!(load_identity_seed(root.path()).unwrap(), Some(legacy), "ordinary loader supports and migrates the same bytes");
}
