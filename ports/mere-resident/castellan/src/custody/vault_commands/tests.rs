// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
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

#[test]
fn generate_ssh_returns_only_the_public_half_and_preserves_the_master() {
    let (storage, id) = empty_vault();
    let before = load(&storage, &id).unwrap().master.public_key();
    let mut out = Vec::new();
    execute(
        &storage,
        &id,
        "InMemory",
        "generate-ssh",
        &["surface-to-windows".into()],
        &mut out,
    )
    .unwrap();
    let profile = load(&storage, &id).unwrap();
    assert_eq!(profile.master.public_key(), before);
    let slots = ssh_slot::ssh_slots(&profile);
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].tier, UnlockTier::Session);
    let text = String::from_utf8(out).unwrap();
    assert_eq!(text.trim(), slots[0].public().to_openssh().unwrap());
    assert!(!text.contains("PRIVATE KEY"));
    assert_eq!(slots[0].private.comment(), "surface-to-windows");
}

#[test]
fn generate_ssh_refuses_bad_comments_without_mutating_the_vault() {
    let (storage, id) = empty_vault();
    for args in [
        vec![],
        vec!["".into()],
        vec!["bad\ncomment".into()],
        vec!["x".repeat(257)],
        vec!["first".into(), "second".into()],
    ] {
        assert!(
            execute(
                &storage,
                &id,
                "InMemory",
                "generate-ssh",
                &args,
                &mut Vec::new()
            )
            .is_err()
        );
    }
    assert!(load(&storage, &id).unwrap().slots.is_empty());
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

fn mint_fixture(
    policy: &personae::ssh_face::FacePolicy,
) -> (crate::custody::InMemoryStorage, ProfileId) {
    let (storage, id) = empty_vault();
    add_ssh_key(&storage, &id, &fixture("ed25519"), UnlockTier::Session).unwrap();
    let mut profile = load(&storage, &id).unwrap();
    crate::custody::ssh_face::store_policy(&mut profile, policy).unwrap();
    storage.save_profile(&profile).unwrap();
    (storage, id)
}

fn mint_text(
    storage: &dyn IdentityStorage,
    id: &ProfileId,
    flags: &[&str],
) -> Result<String, String> {
    let mut args = vec![
        "ssh".to_string(),
        "--principal".to_string(),
        "t".to_string(),
    ];
    args.extend(flags.iter().map(|arg| arg.to_string()));
    let mut out = Vec::new();
    execute(storage, id, "InMemory", "mint", &args, &mut out)?;
    Ok(String::from_utf8(out).unwrap())
}

#[test]
fn mint_refuses_replacing_face_restrictions() {
    let mut policy = personae::ssh_face::FacePolicy::burner("t", "uptime");
    policy.source_address = Some("192.0.2.0/24".into());
    let (storage, id) = mint_fixture(&policy);
    let mut accepted = Vec::new();
    for flags in [
        ["--force-command", "/bin/sh"],
        ["--force-command", ""],
        ["--source-address", "0.0.0.0/0"],
        ["--source-address", ""],
    ] {
        match mint_text(&storage, &id, &flags) {
            Ok(_) => accepted.push(flags),
            Err(err) => assert!(err.contains(flags[0]), "{err}"),
        }
    }
    assert!(
        accepted.is_empty(),
        "accepted conflicting restrictions: {accepted:?}"
    );
    for flags in [
        vec![],
        vec![
            "--force-command",
            "uptime",
            "--source-address",
            "192.0.2.0/24",
        ],
    ] {
        let text = mint_text(&storage, &id, &flags).unwrap();
        let cert = ssh_key::Certificate::from_openssh(text.lines().next().unwrap()).unwrap();
        assert_eq!(
            cert.critical_options().get("force-command").unwrap(),
            "uptime"
        );
        assert_eq!(
            cert.critical_options().get("source-address").unwrap(),
            "192.0.2.0/24"
        );
    }
    let (storage, id) = mint_fixture(&personae::ssh_face::FacePolicy::work("t"));
    let text = mint_text(
        &storage,
        &id,
        &[
            "--force-command",
            "uptime",
            "--source-address",
            "192.0.2.0/24",
        ],
    )
    .unwrap();
    let cert = ssh_key::Certificate::from_openssh(text.lines().next().unwrap()).unwrap();
    assert_eq!(
        cert.critical_options().get("force-command").unwrap(),
        "uptime"
    );
    assert_eq!(
        cert.critical_options().get("source-address").unwrap(),
        "192.0.2.0/24"
    );
}

#[test]
fn mint_reports_the_encoded_lifetime() {
    let (storage, id) = mint_fixture(&personae::ssh_face::FacePolicy::work("t"));
    let mut incorrect = Vec::new();
    for hours in ["1", "24"] {
        let text = mint_text(&storage, &id, &["--hours", hours]).unwrap();
        let cert = ssh_key::Certificate::from_openssh(text.lines().next().unwrap()).unwrap();
        let lifetime = cert.valid_before() - cert.valid_after();
        assert!(lifetime <= personae::ssh_ca::MAX_CERT_TTL_MS / 1000);
        if !text.contains(&format!("valid {lifetime}s")) {
            incorrect.push((
                hours.to_string(),
                lifetime,
                text.lines().last().unwrap().to_string(),
            ));
        }
    }
    assert!(
        incorrect.is_empty(),
        "incorrect validity reports: {incorrect:?}"
    );
}

#[test]
fn mint_refuses_duration_overflow() {
    let (storage, id) = mint_fixture(&personae::ssh_face::FacePolicy::work("t"));
    assert!(
        mint_text(&storage, &id, &["--hours", &u64::MAX.to_string()])
            .unwrap_err()
            .contains("--hours")
    );
}

#[test]
fn load_reports_locked_storage_and_preserves_missing_profile_guidance() {
    let (memory, id) = empty_vault();
    let dir = tempfile::tempdir().unwrap();
    let storage = crate::custody::SealedProfileStorage::open_with_key(dir.path(), [0x63; 32]);
    storage.save_profile(&load(&memory, &id).unwrap()).unwrap();
    storage.lock();
    let err = load(&storage, &id).err().unwrap();
    assert!(err.contains("locked"), "{err}");
    assert!(!err.contains("no profiles"), "{err}");
    let storage = crate::custody::InMemoryStorage::new();
    assert!(load(&storage, &id).err().unwrap().contains("new-profile"));
}
