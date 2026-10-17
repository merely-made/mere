//! Diagnostic observations of public custody flows. All keys are temp fixtures.
use castellan::custody::wallet::*;
use pandect::{wallet_grant::*, wallet_store::*};
use personae::PersonaId;

fn spec(device: &LocalDeviceIdentity, persona: PersonaId) -> RemoteAuthGrantSpec {
    RemoteAuthGrantSpec {
        device_id: device.device_id, delegatee_pubkey: device.public_key(),
        label: "audit fixture".into(), exposure: DeviceExposure::HiddenClient,
        issued_at_ms: 1_770_000_000_000, expires_at_ms: Some(4_102_444_800_000),
        personas: vec![persona], scopes: vec!["identity.act".into()],
        attenuations: vec!["no-subdelegation".into()], wrapped_private_epochs: vec![],
    }
}

#[test]
fn successful_narrower_issuance_writes_refs_that_disagree_with_the_stored_set() {
    let root = tempfile::tempdir().unwrap();
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad31));
    ensure_wallet_state(root.path(), persona, "audit fixture").unwrap();
    let local = LocalDeviceIdentity::new(DeviceId::from_uuid(uuid::Uuid::from_u128(0xad32)), [0x3b;32], "fixture".into());
    let mut request = spec(&local, persona);
    request.scopes.push("transport.egress".into());
    issue_remote_auth_device_grant(root.path(), &request).unwrap();
    request.personas.clear();
    request.scopes = vec!["transport.egress".into()];
    let issued = issue_remote_auth_device_grant(root.path(), &request).unwrap();
    assert!(issued.personas.is_empty());
    let restored = load_device_grant_set(root.path(), local.device_id).unwrap();
    assert!(restored.personas.contains_key(&persona));
    let expected_ref = device_grant_set_ref(&issued);
    assert_ne!(device_grant_set_ref(&restored), expected_ref);
    let roster = load_device_roster(root.path()).unwrap().unwrap();
    assert_eq!(roster.devices.iter().find(|d|d.device_id == local.device_id).unwrap().grant_ref, Some(expected_ref));
    let wallet = load_identity_wallet(root.path()).unwrap().unwrap();
    assert_eq!(wallet.grant_index.iter().find(|d| d.device_id==local.device_id).unwrap().grant_ref, Some(expected_ref));
}

#[test]
fn missing_wrapping_key_refusal_already_persisted_the_enrollment_payload() {
    let issuer = tempfile::tempdir().unwrap();
    let recipient = tempfile::tempdir().unwrap();
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xad41));
    ensure_wallet_state(issuer.path(), persona, "audit fixture").unwrap();
    let local = ensure_local_device_identity(recipient.path(), "audit fixture").unwrap();
    let mut request = spec(&local, persona);
    request.scopes.push("private.read".into());
    let epoch = load_current_private_epoch(issuer.path(), persona).unwrap().unwrap();
    let wrapping_key = [0x46; 32];
    request.wrapped_private_epochs.push(EpochCarriage {
        persona_id: persona,
        material: wrap_private_epoch_material(persona, epoch.epoch_id, &epoch.epoch_secret, wrapping_key).unwrap(),
    });
    issue_remote_auth_device_grant(issuer.path(), &request).unwrap();
    let bundle = build_remote_auth_enrollment_bundle(issuer.path(), local.device_id).unwrap();
    assert!(load_device_grant_set(recipient.path(), local.device_id).unwrap().is_empty());
    assert!(load_persona_wallet(recipient.path(), persona).unwrap().is_none());
    let error = install_remote_auth_enrollment_bundle(recipient.path(), &bundle).unwrap_err();
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);
    assert!(error.to_string().contains("wrapping key"));
    assert_eq!(load_device_grant_set(recipient.path(), local.device_id).unwrap(), bundle.grant);
    assert_eq!(load_persona_wallet(recipient.path(), persona).unwrap().unwrap(), bundle.persona_wallets[0]);
    assert!(load_device_roster(recipient.path()).unwrap().is_none());
    assert!(load_identity_wallet(recipient.path()).unwrap().is_none());
    assert!(load_current_private_epoch(recipient.path(), persona).unwrap().is_none());
    // Positive control: supplying the required key finishes the installation.
    install_remote_auth_enrollment_bundle_with_wrapping_key(recipient.path(), &bundle, wrapping_key).unwrap();
    assert_eq!(load_current_private_epoch(recipient.path(), persona).unwrap().unwrap(), epoch);
}
