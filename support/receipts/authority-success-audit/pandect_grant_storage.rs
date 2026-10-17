//! Diagnostic observations, not correctness expectations.
use identity::{PersonaId, carry::{DeviceId, DevicePublicKey, DeviceGrantSet, ACTION_TRANSPORT_EGRESS, ACTION_IDENTITY_ACT}};

#[test]
fn saving_a_smaller_grant_set_keeps_old_members_and_returns_a_different_ref_from_reload() {
    let root = tempfile::tempdir().unwrap();
    let device = DeviceId::from_uuid(uuid::Uuid::from_u128(0xadd1));
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xadd2));
    let first = identity::carry::issue_device_grant_set(
        [0x6d; 32], device, DevicePublicKey([0x7a; 32]),
        &[ACTION_TRANSPORT_EGRESS, ACTION_IDENTITY_ACT], &[persona],
        60_000, 1_770_000_000_000,
    ).unwrap();
    pandect::save_device_grant_set(root.path(), device, &first).unwrap();
    assert_eq!(pandect::load_device_grant_set(root.path(), device).unwrap(), first);
    let second = DeviceGrantSet { device: first.device.clone(), personas: Default::default() };
    let saved_ref = pandect::save_device_grant_set(root.path(), device, &second).unwrap();
    let restored = pandect::load_device_grant_set(root.path(), device).unwrap();
    assert_eq!(saved_ref, pandect::device_grant_set_ref(&second));
    assert!(restored.personas.contains_key(&persona));
    assert_ne!(pandect::device_grant_set_ref(&restored), saved_ref);
    let empty = DeviceGrantSet { device: None, personas: Default::default() };
    let empty_ref = pandect::save_device_grant_set(root.path(), device, &empty).unwrap();
    assert!(!pandect::load_device_grant_set(root.path(), device).unwrap().is_empty());
    assert_ne!(pandect::device_grant_set_ref(&restored), empty_ref);
}
