// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::custody::vault::{CredentialLineage, SecretBytes};
use personae::Ed25519Keypair;

fn profile_with(keys: &[ProtocolKey]) -> Profile {
    let mut profile = Profile::new(
        ProfileId("t".into()),
        "t",
        Ed25519Keypair::from_seed([1; 32]),
    );
    for key in keys {
        profile.slots.insert(
            key.clone(),
            IdentitySlot::Direct {
                kind: key.mod_id.clone(),
                payload: SecretBytes::new(vec![0; 4]),
                lineage: CredentialLineage::LocallyDerived,
                unlock_tier: UnlockTier::Session,
            },
        );
    }
    profile
}

#[test]
fn exact_key_resolves_including_colons_in_the_instance() {
    let key = ProtocolKey::new("ssh", Some("SHA256:abcdef".into()));
    let profile = profile_with(std::slice::from_ref(&key));
    assert_eq!(resolve_key(&profile, "ssh:SHA256:abcdef").unwrap(), key);
}

#[test]
fn unique_prefix_resolves() {
    let key = ProtocolKey::new("ssh", Some("SHA256:abcdef".into()));
    let profile = profile_with(std::slice::from_ref(&key));
    assert_eq!(resolve_key(&profile, "ssh:SHA256:abc").unwrap(), key);
    assert_eq!(resolve_key(&profile, "ssh").unwrap(), key);
}

#[test]
fn ambiguous_prefix_lists_candidates() {
    let profile = profile_with(&[
        ProtocolKey::new("ssh", Some("SHA256:aaa".into())),
        ProtocolKey::new("ssh", Some("SHA256:aab".into())),
    ]);
    let err = resolve_key(&profile, "ssh:SHA256:aa").unwrap_err();
    assert!(err.contains("ambiguous"), "got: {err}");
    assert!(
        err.contains("SHA256:aaa") && err.contains("SHA256:aab"),
        "got: {err}"
    );
}

#[test]
fn unknown_key_is_an_error() {
    let profile = profile_with(&[ProtocolKey::new("ssh", Some("SHA256:aaa".into()))]);
    assert!(resolve_key(&profile, "nostr").is_err());
}

#[test]
fn instanceless_key_resolves_exactly() {
    let key = ProtocolKey::new("nostr", None);
    let profile = profile_with(std::slice::from_ref(&key));
    assert_eq!(resolve_key(&profile, "nostr").unwrap(), key);
}

/// personae's test-only fixtures.
fn fixture(name: &str) -> PrivateKey {
    let text = match name {
        "ed25519" => include_str!("../../../tests/fixtures/ssh/ed25519"),
        "rsa1024" => include_str!("../../../tests/fixtures/ssh/rsa1024"),
        "rsa2048" => include_str!("../../../tests/fixtures/ssh/rsa2048"),
        "rsa2048e3" => include_str!("../../../tests/fixtures/ssh/rsa2048e3"),
        "rsa2560" => include_str!("../../../tests/fixtures/ssh/rsa2560"),
        "rsa8192" => include_str!("../../../tests/fixtures/ssh/rsa8192"),
        "ecdsa256" => include_str!("../../../tests/fixtures/ssh/ecdsa256"),
        "ecdsa521" => include_str!("../../../tests/fixtures/ssh/ecdsa521"),
        "dsa" => include_str!("../../../tests/fixtures/ssh/dsa"),
        other => panic!("no fixture {other}"),
    };
    PrivateKey::from_openssh(text).unwrap()
}

fn empty_vault() -> (crate::custody::InMemoryStorage, ProfileId) {
    let storage = crate::custody::InMemoryStorage::new();
    let id = ProfileId("t".into());
    storage
        .save_profile(&Profile::new(
            id.clone(),
            "t",
            Ed25519Keypair::from_seed([2; 32]),
        ))
        .unwrap();
    (storage, id)
}

fn slot_parts(
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    key: &ProtocolKey,
) -> (String, Vec<u8>, CredentialLineage, UnlockTier) {
    match load(storage, id)
        .unwrap()
        .slots
        .get(key)
        .expect("slot held")
    {
        IdentitySlot::Direct {
            kind,
            payload,
            lineage,
            unlock_tier,
        } => (
            kind.clone(),
            payload.as_slice().to_vec(),
            *lineage,
            *unlock_tier,
        ),
        _ => panic!("ssh slots are Direct"),
    }
}

/// Ruling 60 (54): `add-ssh` of a held key, asking for another tier with
/// another comment, writes nothing and says it is held.
#[test]
fn add_ssh_leaves_a_held_key_untouched() {
    let (storage, id) = empty_vault();
    for name in ["ed25519", "rsa2048", "ecdsa256"] {
        let mut first = fixture(name);
        first.set_comment("as first added");
        let key = ssh_slot::protocol_key_for(&first);
        assert_eq!(
            add_ssh_key(&storage, &id, &first, UnlockTier::PerUse).unwrap(),
            AddSsh::Imported
        );
        let before = slot_parts(&storage, &id, &key);
        let mut again = fixture(name);
        again.set_comment("a different comment");
        assert_eq!(
            add_ssh_key(&storage, &id, &again, UnlockTier::Session).unwrap(),
            AddSsh::AlreadyHeld(UnlockTier::PerUse),
            "{name}"
        );
        assert_eq!(slot_parts(&storage, &id, &key), before, "{name}");
    }
}

/// Ruling 60 (55, 56): the same door and reasons as `ssh-add` and import.
#[test]
fn add_ssh_refuses_unsignable_keys_with_the_reason() {
    let (storage, id) = empty_vault();
    for name in [
        "rsa1024",
        "rsa2560",
        "rsa8192",
        "rsa2048e3",
        "dsa",
        "ecdsa521",
    ] {
        let reason = add_ssh_key(&storage, &id, &fixture(name), UnlockTier::Session).unwrap_err();
        let gate = crate::custody::ssh_sign::check_signable(fixture(name).key_data())
            .unwrap_err()
            .to_string();
        assert_eq!(reason, format!("refused: {gate}"), "{name}");
    }
    assert!(load(&storage, &id).unwrap().slots.is_empty());
}
