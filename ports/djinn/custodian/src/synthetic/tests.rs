// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use personae::{Ed25519PublicKey, Ed25519Signature};

const KEY: [u8; 32] = [0x37; 32];
fn setup() -> (tempfile::TempDir, Custodian) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join(".djinn-synthetic-custody"),
        b"0123456789abcdef0123456789abcdef",
    )
    .unwrap();
    let config = SyntheticConfig {
        run_id: "0123456789abcdef0123456789abcdef".into(),
        directory: dir.path().into(),
        profile: "h1-synthetic".into(),
        principal: "signed-fixture-controller".into(),
    };
    let core = Custodian::create_synthetic(config, Zeroizing::new(KEY)).unwrap();
    (dir, core)
}
fn call(c: &mut Custodian, value: Value) -> Result<Dispatch> {
    c.dispatch(
        "signed-fixture-controller",
        &serde_json::to_vec(&value).unwrap(),
    )
}
fn sign(c: &mut Custodian, message: &str) -> String {
    let instance = c.instance.clone();
    match call(
        c,
        json!({"op":"sign","namespace":ROOT_NAMESPACE,"challenge":message,"instance":instance}),
    )
    .unwrap()
    {
        Dispatch::Confirm(facts) => facts.ticket,
        _ => panic!("signing must require native confirmation"),
    }
}
fn issue_grant(c: &mut Custodian, uses: u16) -> Value {
    let instance = c.instance.clone();
    let profile = c.config.profile.clone();
    let ticket=match call(c,json!({"op":"open-session","namespace":SESSION_NAMESPACE,"profile":profile,"max_uses":uses,"instance":instance})).unwrap() {
        Dispatch::Confirm(facts)=>facts.ticket,
        _=>panic!("session must require native confirmation"),
    };
    c.complete_confirmation(&ticket, true).unwrap()
}
fn session_request(c: &Custodian, g: &Value) -> Value {
    json!({"op":"session-sign","capability":g["capability"],"namespace":SESSION_NAMESPACE,
        "profile":c.config.profile,"challenge":"synthetic payload","generation":g["generation"],"instance":c.instance})
}
fn verify(reply: &Value, message: &str) {
    let public: [u8; 32] = STANDARD
        .decode(reply["publicKey"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    let signature: [u8; 64] = STANDARD
        .decode(reply["signature"].as_str().unwrap())
        .unwrap()
        .try_into()
        .unwrap();
    assert!(Ed25519PublicKey::from_bytes(&public).unwrap().verify(
        message.as_bytes(),
        &Ed25519Signature::from_bytes(&signature)
    ));
}

#[test]
fn real_castellan_root_signature_survives_cold_reopen_without_rewriting_records() {
    let (dir, mut core) = setup();
    let first = core.public_key.unwrap();
    let ticket = sign(&mut core, "root payload");
    verify(
        &core.complete_confirmation(&ticket, true).unwrap(),
        "root payload",
    );
    let record = std::fs::read_dir(dir.path().join("profiles"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let before = std::fs::read(&record).unwrap();
    let config = core.config.clone();
    drop(core);
    let mut reopened = Custodian::open_synthetic(config, Zeroizing::new(KEY)).unwrap();
    assert!(reopened.vault.is_none());
    reopened
        .unlock_after_native_confirmation(Zeroizing::new(KEY))
        .unwrap();
    assert_eq!(reopened.public_key, Some(first));
    let ticket = sign(&mut reopened, "reopened");
    verify(
        &reopened.complete_confirmation(&ticket, true).unwrap(),
        "reopened",
    );
    assert_eq!(std::fs::read(record).unwrap(), before);
}

#[test]
fn missing_or_tampered_profile_never_regenerates_an_identity() {
    let (dir, core) = setup();
    let config = core.config.clone();
    drop(core);
    let record = std::fs::read_dir(dir.path().join("profiles"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    std::fs::write(&record, b"tampered").unwrap();
    let mut reopened = Custodian::open_synthetic(config.clone(), Zeroizing::new(KEY)).unwrap();
    assert!(
        reopened
            .unlock_after_native_confirmation(Zeroizing::new(KEY))
            .is_err()
    );
    std::fs::remove_file(record).unwrap();
    let mut missing = Custodian::open_synthetic(config, Zeroizing::new(KEY)).unwrap();
    assert!(
        missing
            .unlock_after_native_confirmation(Zeroizing::new(KEY))
            .is_err()
    );
}

#[test]
fn a_wrong_key_and_a_missing_synthetic_marker_are_refused() {
    let (dir, core) = setup();
    let config = core.config.clone();
    drop(core);
    let mut reopened = Custodian::open_synthetic(config.clone(), Zeroizing::new([0; 32])).unwrap();
    assert!(
        reopened
            .unlock_after_native_confirmation(Zeroizing::new([0; 32]))
            .is_err()
    );
    std::fs::remove_file(dir.path().join(".djinn-synthetic-custody")).unwrap();
    assert!(Custodian::open_synthetic(config, Zeroizing::new(KEY)).is_err());
}

#[test]
fn client_approval_fields_and_client_decision_operations_cannot_authorize() {
    let (_dir, mut core) = setup();
    let instance = core.instance.clone();
    assert!(call(&mut core,json!({"op":"sign","namespace":ROOT_NAMESPACE,"challenge":"x","instance":instance,"approved":true})).is_err());
    let ticket = sign(&mut core, "x");
    assert!(
        call(
            &mut core,
            json!({"op":"approve","ticket":ticket,"accepted":true})
        )
        .is_err()
    );
    assert!(core.complete_confirmation(&ticket, false).is_err());
    assert!(
        core.complete_confirmation(&ticket, true).is_err(),
        "a denied ticket cannot be replayed"
    );
}

#[test]
fn confirmations_are_one_use_and_bound_to_the_exact_payload() {
    let (_dir, mut core) = setup();
    let ticket = sign(&mut core, "exact");
    let reply = core.complete_confirmation(&ticket, true).unwrap();
    verify(&reply, "exact");
    assert!(core.complete_confirmation(&ticket, true).is_err());
}

#[test]
fn lock_and_native_unlock_revoke_approvals_and_continuous_capabilities() {
    let (_dir, mut core) = setup();
    let old = sign(&mut core, "pending");
    let grant = issue_grant(&mut core, 3);
    core.lock().unwrap();
    assert!(core.vault.is_none());
    assert!(core.complete_confirmation(&old, true).is_err());
    core.unlock_after_native_confirmation(Zeroizing::new(KEY))
        .unwrap();
    let req = session_request(&core, &grant);
    assert!(call(&mut core, req).is_err());
    let ticket = sign(&mut core, "fresh");
    verify(&core.complete_confirmation(&ticket, true).unwrap(), "fresh");
}

#[test]
fn persisted_lock_blocks_reopen_before_decrypting_any_profile() {
    let (_dir, mut core) = setup();
    let config = core.config.clone();
    core.lock().unwrap();
    drop(core);
    // Even a wrong key is not used before explicit native unlock.
    let mut reopened = Custodian::open_synthetic(config, Zeroizing::new([0; 32])).unwrap();
    assert!(reopened.vault.is_none());
    assert!(reopened.public_key.is_none());
    assert!(
        reopened
            .unlock_after_native_confirmation(Zeroizing::new([0; 32]))
            .is_err()
    );
    assert!(reopened.vault.is_none());
    reopened
        .unlock_after_native_confirmation(Zeroizing::new(KEY))
        .unwrap();
    let ticket = sign(&mut reopened, "after native unlock");
    verify(
        &reopened.complete_confirmation(&ticket, true).unwrap(),
        "after native unlock",
    );
}

#[test]
fn continuous_consumer_is_scoped_derived_bounded_and_revocable() {
    let (_dir, mut core) = setup();
    let root = core.public_key.unwrap();
    let grant = issue_grant(&mut core, 2);
    let mut request = session_request(&core, &grant);
    request["namespace"] = json!("other-domain");
    assert!(call(&mut core, request).is_err());
    let mut request = session_request(&core, &grant);
    request["profile"] = json!("other-persona");
    assert!(call(&mut core, request).is_err());
    for _ in 0..2 {
        let request = session_request(&core, &grant);
        let reply = match call(&mut core, request).unwrap() {
            Dispatch::Reply(v) => v,
            _ => panic!(),
        };
        verify(&reply, "synthetic payload");
        assert_ne!(reply["publicKey"], json!(STANDARD.encode(root)));
    }
    let request = session_request(&core, &grant);
    assert!(call(&mut core, request).is_err());
    let grant = issue_grant(&mut core, 2);
    let instance = core.instance.clone();
    call(
        &mut core,
        json!({"op":"revoke-session","capability":grant["capability"],"instance":instance}),
    )
    .unwrap();
    let request = session_request(&core, &grant);
    assert!(call(&mut core, request).is_err());
}

#[test]
fn a_capability_cannot_cross_authenticated_principals_or_process_instances() {
    let (_dir, mut core) = setup();
    let grant = issue_grant(&mut core, 2);
    let request = session_request(&core, &grant);
    assert!(
        core.dispatch(
            "unrelated-signed-app",
            &serde_json::to_vec(&request).unwrap()
        )
        .is_err()
    );
    let config = core.config.clone();
    drop(core);
    let mut next = Custodian::open_synthetic(config, Zeroizing::new(KEY)).unwrap();
    assert!(call(&mut next, request).is_err());
}

#[test]
fn expiry_and_capacity_limits_are_enforced_without_native_authorization() {
    let (_dir, mut core) = setup();
    let ticket = sign(&mut core, "pending");
    core.pending.get_mut(&ticket).unwrap().deadline = Instant::now() - Duration::from_secs(1);
    assert!(core.complete_confirmation(&ticket, true).is_err());
    for _ in 0..MAX_PENDING {
        sign(&mut core, "bounded");
    }
    let instance = core.instance.clone();
    assert!(
        call(
            &mut core,
            json!({"op":"sign","namespace":ROOT_NAMESPACE,"challenge":"x","instance":instance})
        )
        .is_err()
    );
    assert!(
        core.dispatch(
            "signed-fixture-controller",
            &vec![b' '; MAX_REQUEST_BYTES + 1]
        )
        .is_err()
    );
}

#[test]
fn failed_persisted_barrier_still_drops_keys_and_reports_failure_until_repaired() {
    let (dir, mut core) = setup();
    let marker = dir.path().join(castellan::custody::PERSISTED_LOCK_FILE);
    std::fs::create_dir(&marker).unwrap();
    assert!(core.lock().is_err());
    assert!(core.vault.is_none());
    assert!(core.lock().is_err());
    std::fs::remove_dir(marker).unwrap();
    core.lock().unwrap();
    assert!(castellan::custody::lock_persisted(dir.path()));
}

#[test]
fn identity_preserves_real_castellan_profile_format_and_slot_payload_on_relocation() {
    let (source, core) = setup();
    let public = core.public_key.unwrap();
    let mut config = core.config.clone();
    drop(core);
    let storage = SealedProfileStorage::open_with_key(source.path(), KEY);
    let id = ProfileId(config.profile.clone());
    let mut profile = storage.load_profile(&id).unwrap();
    profile.slots.insert(
        personae::ProtocolKey::new("synthetic.slot", None),
        castellan::custody::IdentitySlot::Direct {
            kind: "test".into(),
            payload: castellan::custody::SecretBytes::new(vec![0x51; 32]),
            lineage: personae::CredentialLineage::LocallyDerived,
            unlock_tier: personae::UnlockTier::Session,
        },
    );
    storage.save_profile(&profile).unwrap();
    drop(profile);
    drop(storage);
    let destination = tempfile::tempdir().unwrap();
    std::fs::create_dir(destination.path().join("profiles")).unwrap();
    for entry in std::fs::read_dir(source.path().join("profiles")).unwrap() {
        let entry = entry.unwrap();
        std::fs::copy(
            entry.path(),
            destination.path().join("profiles").join(entry.file_name()),
        )
        .unwrap();
    }
    std::fs::copy(
        source.path().join(".djinn-synthetic-custody"),
        destination.path().join(".djinn-synthetic-custody"),
    )
    .unwrap();
    config.directory = destination.path().into();
    let mut relocated = Custodian::open_synthetic(config, Zeroizing::new(KEY)).unwrap();
    relocated
        .unlock_after_native_confirmation(Zeroizing::new(KEY))
        .unwrap();
    assert_eq!(relocated.public_key, Some(public));
    let slot = relocated
        .vault
        .as_ref()
        .unwrap()
        .slot(&personae::ProtocolKey::new("synthetic.slot", None))
        .unwrap()
        .unwrap();
    match slot {
        castellan::custody::IdentitySlot::Direct { payload, .. } => {
            assert_eq!(payload.as_slice(), &[0x51; 32])
        },
        _ => panic!(),
    };
}

#[test]
fn removing_the_same_user_lock_marker_cannot_enable_unattended_reopen() {
    let (dir, mut core) = setup();
    let config = core.config.clone();
    let root = core.public_key;
    core.lock().unwrap();
    drop(core);
    std::fs::remove_file(dir.path().join(castellan::custody::PERSISTED_LOCK_FILE)).unwrap();
    let mut reopened = Custodian::open_synthetic(config, Zeroizing::new(KEY)).unwrap();
    assert!(reopened.vault.is_none());
    let instance = reopened.instance.clone();
    assert!(matches!(
        call(
            &mut reopened,
            json!({"op":"sign","namespace":ROOT_NAMESPACE,"challenge":"attack","instance":instance})
        ),
        Err(Refusal("locked"))
    ));
    reopened
        .unlock_after_native_confirmation(Zeroizing::new(KEY))
        .unwrap();
    assert_eq!(reopened.public_key, root);
}
