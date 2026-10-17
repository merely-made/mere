//! Bug-observation diagnostics. Passing means the defect was reproduced.
#![cfg(feature = "cxf")]
use std::collections::BTreeMap;
use castellan::{authority::PersonaeHost, custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage}, cxf::{Destination, Import}, otp::{OtpError, OtpItemError, OtpReleaseError, OtpReleaseGate, OtpReleaseParticipantClaim}, resident::CastellanResident};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use dramatis::{intents::SshUnlockPolicyIntentV1, view::VaultProtectionView};
use serde_json::{json, Value};

fn fixture(root: &std::path::Path) -> (PersonaeHost<SealedProfileStorage>, CastellanResident) {
    let storage = SealedProfileStorage::open_with_key(root.join("vault"), [0x29; 32]);
    let profile = Profile::new(ProfileId("family".into()), "Family", Ed25519Keypair::from_seed([0x39; 32]));
    storage.save_profile(&profile).unwrap();
    storage.save_profile(&Profile::new(ProfileId("work".into()), "Work", Ed25519Keypair::from_seed([0x40; 32]))).unwrap();
    let host = PersonaeHost::new(IdentityVault::with_profile(storage, profile), None, VaultProtectionView::OsProtected);
    let resident = CastellanResident::claim(root.join("records"), [0x61; 32], root.join("freshness"), [0x62; 32]).unwrap();
    (host, resident)
}
fn document(credential: Value) -> Vec<u8> {
    serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Audit","timestamp":1,"accounts":[{"id":"YQ","username":"audit","email":"audit@example.invalid","collections":[],"items":[{"id":"aQ","title":"Audit","credentials":[credential]}]}]})).unwrap()
}
fn choices(persona: PersonaId, profile: &str) -> BTreeMap<String, Destination> {
    BTreeMap::from([("YQ".into(), Destination { persona, profile: ProfileId(profile.into()), ssh_unlock_policy: SshUnlockPolicyIntentV1::Session })])
}

#[test]
fn cxf_stores_short_seed_in_vault_then_ordinary_approval_cannot_generate() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    for (seed, short) in [("MY", true), ("GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ", false)] {
        let persona = PersonaId::new();
        let import = Import::read(document(json!({"type":"totp","secret":seed,"algorithm":"sha1","period":30,"digits":6}))).unwrap();
        assert_eq!(import.preview().accounts[0].dispositions.get("stored"), Some(&1));
        let receipt = import.commit(choices(persona, "family"), &resident, &host).unwrap();
        assert_eq!(receipt.items, 1);
        let item = resident.items(persona).list().unwrap().remove(0);
        assert_eq!(item.state, chatelaine::ItemState::Vault);
        let gate = OtpReleaseGate::new(resident.otp_items(persona));
        let request = gate.petition(item.id, item.credentials[0].id, OtpReleaseParticipantClaim::unverified("local:audit", "session:audit").unwrap()).unwrap();
        let result = gate.approve(request.id);
        if short {
            assert!(matches!(result, Err(OtpReleaseError::Item(OtpItemError::Generation(OtpError::SecretTooShort { bytes: 1 })))));
        } else {
            let result = result.unwrap();
            let expires = result.tile().time_ring().unwrap().expires_at_unix_secs;
            assert!(result.tile().code_at_unix_time(expires - 1).is_some());
        }
    }
}

#[test]
fn failed_native_import_reports_zero_but_retains_a_binding_that_blocks_reassignment() {
    use base64::Engine;
    use rsa::pkcs8::EncodePrivateKey;
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let der = ed25519_dalek::SigningKey::from_bytes(&[0x41; 32]).to_pkcs8_der().unwrap();
    let credential = |comment| json!({"type":"ssh-key","keyType":"ssh-ed25519","privateKey":base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(der.as_bytes()),"keyComment":comment});
    let failed = Import::read(document(credential("invalid\ncomment"))).unwrap().commit(choices(persona, "family"), &resident, &host).unwrap_err();
    assert_eq!((failed.receipt.items, failed.receipt.ssh_keys), (0, 0));
    assert!(resident.items(persona).list().unwrap().is_empty());
    assert_eq!(resident.profile_personas(&ProfileId("family".into())).unwrap(), vec![persona]);
    let retry = Import::read(document(credential("Audit"))).unwrap().commit(choices(persona, "work"), &resident, &host).unwrap_err();
    assert_eq!(retry.reason, "item namespace belongs to another profile");
    // Positive control: the corrected key can import into the other profile
    // under a fresh namespace, proving it was the retained binding that refused.
    let fresh = PersonaId::new();
    let good = Import::read(document(credential("Audit"))).unwrap().commit(choices(fresh, "work"), &resident, &host).unwrap();
    assert_eq!(good.ssh_keys, 1);
    assert_eq!(resident.profile_personas(&ProfileId("work".into())).unwrap(), vec![fresh]);
}

#[cfg(feature = "secret-service")]
#[test]
fn deleting_one_collection_loses_a_shared_item_and_leaves_the_other_collection_broken() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    for shared in [false, true] {
        let persona = PersonaId::new();
        let mut value: Value = serde_json::from_slice(&document(json!({"type":"totp","secret":"GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ","algorithm":"sha1","period":30,"digits":6}))).unwrap();
        value["accounts"][0]["collections"] = json!([
            {"id":"Yw","title":"Delete","items":[{"item":"aQ"}]},
            {"id":"ZA","title":"Keep","items": if shared {json!([{"item":"aQ"}])} else {json!([])}}
        ]);
        Import::read(serde_json::to_vec(&value).unwrap()).unwrap().commit(choices(persona, "family"), &resident, &host).unwrap();
        let item = resident.items(persona).list().unwrap().remove(0);
        let store = resident.secret_service(persona, Default::default());
        let collections = store.collections().unwrap();
        let delete = collections.iter().find(|c| c.label == "Delete").unwrap().id;
        let keep = collections.iter().find(|c| c.label == "Keep").unwrap().id;
        // OTP items are not Secret Service secrets, so both projections are
        // initially valid empty lists despite the shared generic memberships.
        assert!(store.items(delete).unwrap().is_empty());
        assert!(store.items(keep).unwrap().is_empty());
        store.delete_collection(delete).unwrap();
        assert!(resident.items(persona).get(item.id).unwrap().is_none());
        assert!(store.collection(keep).is_ok());
        if shared {
            assert!(store.items(keep).unwrap_err().to_string().contains("indexed but has no record"));
            assert!(store.snapshot().is_err());
        } else {
            assert!(store.items(keep).unwrap().is_empty());
            assert!(store.snapshot().is_ok());
        }
    }
}
