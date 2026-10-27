// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The H2 operations at the core: the public model and profile intents,
//! derived keys of either root, the wallet's epochs, retained payloads, the
//! credential catalog and per-use SSH signing, each checked against the
//! vault or wallet opened directly and each refused without its grant.

use std::path::{Path, PathBuf};
use std::time::Duration;

use castellan::custody::bootstrap::{self, Unlock};
use castellan::custody::{IdentityStorage, Profile};
use custody_wire::contract::{Answer, CredentialOperation, EpochKeyRequest, Outcome, PayloadOperation, Source};
use custody_wire::secret::{Bytes, Secret};
use custody_wire::Operation;
use personae::{Ed25519Keypair, IdentityProvider, PersonaId, ProfileId};
use serde_json::{Value, json};

use super::{DJINN_UID, Harness, PASSPHRASE, code, consent_helper, djinn, petition_in};
use crate::core::Decider;
use crate::policy::Config;

const WORK: [u8; 32] = [0x31; 32];
const HOME: [u8; 32] = [0x32; 32];

/// A passphrase vault with `work` and `home`, and a data root beside it.
pub(super) fn vault() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let opened = bootstrap::open_storage(&dir.path().join("vault"), Unlock::passphrase(PASSPHRASE)).unwrap();
    for (id, seed) in [("work", WORK), ("home", HOME)] {
        opened.storage.save_profile(&Profile::new(ProfileId(id.into()), id, Ed25519Keypair::from_seed(seed))).unwrap();
    }
    std::fs::create_dir(dir.path().join("data")).unwrap();
    dir
}

pub(super) fn data(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("data")
}

pub(super) fn config(dir: &tempfile::TempDir, consent: &str, grants: Value) -> Config {
    let json = json!({
        "vault": {"directory": dir.path().join("vault"), "profile": "work", "data_root": data(dir)},
        "owner": {"unix_uid": DJINN_UID}, "consent": consent, "development": true,
        "principals": [
            {"name": "djinn", "match": {"unix-uid": DJINN_UID}, "grants": grants},
            {"name": "consent", "match": {"signed": {"team": "TEAM", "identifier": "com.merely.djinn.consent"}}, "consent_channel": true}
        ]
    });
    Config::parse(&serde_json::to_vec(&json).unwrap()).unwrap()
}

pub(super) fn base(extra: Value) -> Value {
    let mut grants = vec![json!({"operation": "status"}), json!({"operation": "lock"}), json!({"operation": "unlock"})];
    grants.extend(extra.as_array().unwrap().iter().cloned());
    Value::Array(grants)
}

fn direct(dir: &Path, profile: &str) -> castellan::custody::IdentityVault<Box<dyn IdentityStorage>> {
    let opened = bootstrap::open_storage(&dir.join("vault"), Unlock::passphrase(PASSPHRASE)).unwrap();
    castellan::custody::IdentityVault::open(opened.storage, &ProfileId(profile.into())).unwrap()
}

pub(super) fn answer(outcome: Outcome) -> Answer {
    match outcome {
        Outcome::Success { answer } => answer,
        other => panic!("refused: {other:?}"),
    }
}

fn intent(name: &str, payload: Value) -> Operation {
    Operation::Intent {
        intent: name.into(),
        payload: Bytes(serde_json::to_vec(&payload).unwrap()),
    }
}

#[test]
fn the_public_model_answers_while_locked_and_follows_profile_intents() {
    let dir = vault();
    let mut h = Harness::new(config(&dir, "deny", base(json!([{"operation": "snapshot"}, {"operation": "manage", "consent": "none"}]))));
    let session = h.open(djinn());
    let Answer::Snapshot { snapshot } = answer(h.outcome(session, Operation::Snapshot {})) else { panic!() };
    assert_eq!(snapshot.vault.lock, dramatis::view::VaultLockView::Locked);
    assert!(snapshot.profiles.is_empty(), "nothing about the vault before it opens");
    assert_eq!(code(&h.outcome(session, Operation::HasProfile { profile: "home".into() })), "locked");
    h.unlock(session);
    let Answer::Snapshot { snapshot } = answer(h.outcome(session, Operation::Snapshot {})) else { panic!() };
    assert!(["work", "home"].iter().all(|id| snapshot.profiles.iter().any(|p| p.id == *id)));

    let created = intent(dramatis::intents::PROFILE_CREATE_INTENT, json!({"id": "studio", "display_name": "Studio"}));
    assert!(matches!(answer(h.outcome(session, created)), Answer::Done {}));
    assert!(matches!(answer(h.outcome(session, Operation::HasProfile { profile: "studio".into() })), Answer::Bool { value: true }));
    let switched = intent(dramatis::intents::PROFILE_SWITCH_INTENT, json!({"profile": "home"}));
    assert!(matches!(answer(h.outcome(session, switched)), Answer::Done {}));
    let status = h.core.status();
    assert_eq!(status.profile.as_deref(), Some("home"), "the status follows the switch");
    assert_eq!(status.master_public_key, Some(direct(dir.path(), "home").master_public_key().to_bytes()));
    let decision = intent(dramatis::intents::SIGNING_DENY_INTENT, json!({}));
    assert!(code(&h.outcome(session, decision)).starts_with("intent-refused"), "signing decisions are not intents");

    let mut bare = Harness::new(config(&dir, "deny", base(json!([]))));
    let session = bare.open(djinn());
    assert_eq!(code(&bare.outcome(session, Operation::Snapshot {})), "operation-denied");
}

#[test]
fn derived_keys_of_either_root_are_answered_where_the_key_is() {
    let dir = vault();
    let grants = base(json!([
        {"operation": "release", "profile": "any", "salts": [{"prefix": "app/"}], "consent": "none"},
        {"operation": "release", "profile": "current", "source": "wallet", "salts": [{"prefix": "app/"}], "consent": "none"}
    ]));
    let mut h = Harness::new(config(&dir, "deny", grants));
    let session = h.open(djinn());
    h.unlock(session);
    let derive = |source, profile: Option<&str>, salt: &str| Operation::Derive {
        source,
        profile: profile.map(str::to_string),
        salts: vec![Bytes(salt.as_bytes().to_vec())],
    };
    for (profile, id) in [(None, "work"), (Some("home"), "home")] {
        let Answer::Released { master_public_key, keys } = answer(h.outcome(session, derive(Source::Persona, profile, "app/a"))) else { panic!() };
        let vault = direct(dir.path(), id);
        assert_eq!(master_public_key, vault.master_public_key().to_bytes());
        assert_eq!(keys[0].seed.expose(), vault.derive_keypair(b"app/a").unwrap().to_seed());
    }
    for refused in ["other", ""] {
        assert_eq!(code(&h.outcome(session, derive(Source::Persona, None, refused))), "operation-or-scope");
    }

    let attest = Operation::Attest { source: Source::Persona, profile: None, salt: Bytes(b"app/a".to_vec()) };
    let Answer::Attestation { attestation } = answer(h.outcome(session, attest)) else { panic!() };
    let work = direct(dir.path(), "work");
    let derived = work.derive_keypair(b"app/a").unwrap().public_key().to_bytes();
    let parts = insigne::DerivedKeyAttestation::from_parts(attestation.master, attestation.derived, attestation.signature.0);
    assert_eq!(parts.check(b"app/a").unwrap().derived(), &derived);

    let sign = Operation::SignDerived { source: Source::Persona, profile: None, salt: Bytes(b"app/a".to_vec()), message: Bytes(b"act".to_vec()) };
    let Answer::Signature { public_key, signature, .. } = answer(h.outcome(session, sign)) else { panic!() };
    assert_eq!(public_key, derived);
    assert!(personae::Ed25519PublicKey::from_bytes(&public_key).unwrap().verify(b"act", &personae::Ed25519Signature::from_bytes(&signature)));
    let home_key = Operation::PublicKey { source: Source::Persona, profile: Some("home".into()) };
    assert!(matches!(answer(h.outcome(session, home_key)), Answer::PublicKey { key } if key == direct(dir.path(), "home").master_public_key().to_bytes()));

    // The wallet root answers only once a wallet exists.
    assert_eq!(code(&h.outcome(session, derive(Source::Wallet, None, "app/w"))), "no-wallet");
    let seed = castellan::custody::wallet::ensure_wallet_state(&data(&dir), PersonaId::new(), "fixture").unwrap();
    let Answer::Released { master_public_key, keys } = answer(h.outcome(session, derive(Source::Wallet, None, "app/w"))) else { panic!() };
    let wallet = personae::InMemoryProvider::from_seed(seed);
    assert_eq!(master_public_key, wallet.master_public_key().to_bytes());
    assert_eq!(keys[0].seed.expose(), wallet.derive_keypair(b"app/w").unwrap().to_seed());
}

#[test]
fn wallet_epochs_are_the_derivation_djinn_made_and_need_a_wallet_grant() {
    let dir = vault();
    let persona = PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(&data(&dir), persona, "fixture").unwrap();
    let epochs = |persona: PersonaId, device_bound| Operation::EpochKeys {
        persona: persona.as_uuid().to_string(),
        device_label: Some("this device".into()),
        keys: vec![EpochKeyRequest { context: "mere.test.epoch.v1".into(), device_bound, public_only: false }],
    };
    let mut bare = Harness::new(config(&dir, "deny", base(json!([]))));
    let session = bare.open(djinn());
    bare.unlock(session);
    assert_eq!(code(&bare.outcome(session, epochs(persona, false))), "operation-denied");
    drop(bare);

    let mut h = Harness::new(config(&dir, "deny", base(json!([{"operation": "wallet", "consent": "none"}]))));
    let session = h.open(djinn());
    assert_eq!(code(&h.outcome(session, epochs(persona, false))), "locked");
    h.unlock(session);
    let epoch = castellan::custody::wallet::load_current_private_epoch(&data(&dir), persona).unwrap().unwrap().epoch_secret;
    let Answer::EpochKeys { keys } = answer(h.outcome(session, epochs(persona, false))) else { panic!() };
    assert_eq!(keys[0].key.expose(), blake3::derive_key("mere.test.epoch.v1", &epoch));
    let Answer::EpochKeys { keys } = answer(h.outcome(session, epochs(persona, true))) else { panic!() };
    let device = castellan::custody::wallet::load_local_device_identity(&data(&dir)).unwrap().unwrap();
    let mut material = epoch.to_vec();
    material.extend_from_slice(&Ed25519Keypair::from_seed(device.device_seed).public_key().to_bytes());
    assert_eq!(keys[0].key.expose(), blake3::derive_key("mere.test.epoch.v1", &material), "bound to this device's public root");
    assert_eq!(code(&h.outcome(session, epochs(PersonaId::new(), false))), "no-persona");
}

#[test]
fn retained_payloads_seal_and_unseal_only_for_the_open_profile() {
    let dir = vault();
    let persona = PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(&data(&dir), persona, "fixture").unwrap();
    let mut h = Harness::new(config(&dir, "deny", base(json!([{"operation": "payload", "consent": "none"}]))));
    let session = h.open(djinn());
    h.unlock(session);
    let master = direct(dir.path(), "work").master_public_key().to_bytes();
    let clear = b"retained clip".to_vec();
    let hash = serde_json::to_value(eidetic::Hash::of(&clear)).unwrap().as_str().unwrap().to_string();
    let payload = |profile: &str, master: [u8; 32], operation| Operation::Payload {
        profile: profile.into(),
        persona: persona.as_uuid().to_string(),
        master: Bytes(master.to_vec()),
        scope: Bytes(b"scope".to_vec()),
        content_hash: hash.clone(),
        operation,
    };
    let Answer::Payload { bytes, marker: Some(marker) } = answer(h.outcome(session, payload("work", master, PayloadOperation::Seal { bytes: Bytes(clear.clone()) }))) else { panic!() };
    assert_ne!(bytes.0, clear);
    let unseal = || PayloadOperation::Unseal { bytes: bytes.clone(), marker: marker.clone() };
    let Answer::Payload { bytes: opened, marker: None } = answer(h.outcome(session, payload("work", master, unseal()))) else { panic!() };
    assert_eq!(opened.0, clear);
    assert_eq!(code(&h.outcome(session, payload("home", master, unseal()))), "profile-changed");
    assert_eq!(code(&h.outcome(session, payload("work", [7; 32], unseal()))), "profile-changed");
}

fn import(h: &mut Harness, session: crate::core::SessionId, persona: PersonaId, profile: &str, title: &str) {
    let source = serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Account","email":"test@example.invalid","collections":[],"items":[{"id":"aQ","title":title,"credentials":[{"type":"credit-card","number":{"fieldType":"concealed-string","value":"4111111111111111"}}]}]}]})).unwrap();
    let preview = Operation::Credentials { operation: CredentialOperation::CxfPreview { source: Bytes(source.clone()) } };
    let Answer::Value { value } = answer(h.outcome(session, preview)) else { panic!() };
    let commit = Operation::Credentials {
        operation: CredentialOperation::CxfImport {
            source: Bytes(source),
            destinations: json!({"YQ": {"persona": persona, "profile": profile}}),
            preview_sha256: value["source_sha256"].as_str().unwrap().into(),
        },
    };
    let Answer::Value { value } = answer(h.outcome(session, commit)) else { panic!() };
    assert_eq!(value["items"], 1);
}

fn cards(h: &mut Harness, session: crate::core::SessionId, profile: &str) -> Vec<Value> {
    let operation = Operation::Credentials { operation: CredentialOperation::Quarantine { profile: profile.into() } };
    let Answer::Value { value } = answer(h.outcome(session, operation)) else { panic!() };
    value.as_array().unwrap().clone()
}

fn review(card: &Value, profile: &str, action: &str) -> Operation {
    Operation::Credentials {
        operation: CredentialOperation::Review {
            request: json!({"store": card["store"], "profile": profile, "persona": card["persona"], "item": card["item"]["id"], "revision": card["revision"], "action": action}),
        },
    }
}

#[test]
fn the_credential_catalog_reviews_and_assigns_items_under_profile_authority() {
    let dir = vault();
    // An older import: a namespace no profile owns, written before the
    // custodian first claims the work store.
    let older = PersonaId::new();
    {
        let root = crate::host::credential_profile_root(&data(&dir), &ProfileId("work".into()));
        let store = castellan::resident::CastellanResident::claim_derived(root.join("records"), root.join("freshness"), &direct(dir.path(), "work"), crate::host::CREDENTIAL_SALTS).unwrap();
        store.otp_items(older).import_otpauth_uri("otpauth://totp/Receipt:older?secret=GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ&issuer=Receipt").unwrap();
    }
    let mut h = Harness::new(config(&dir, "deny", base(json!([{"operation": "credentials", "consent": "none"}]))));
    let session = h.open(djinn());
    h.unlock(session);
    let (accepted, deleted) = (PersonaId::new(), PersonaId::new());
    import(&mut h, session, accepted, "work", "Accepted card");
    import(&mut h, session, deleted, "work", "Deleted card");
    let held = cards(&mut h, session, "work");
    assert_eq!(held.len(), 2);
    assert!(!serde_json::to_string(&held).unwrap().contains("4111111111111111"), "cards carry no secret");
    assert!(cards(&mut h, session, "home").is_empty());
    let card = |persona: PersonaId| held.iter().find(|c| c["persona"] == json!(persona)).unwrap().clone();

    let mut stale = card(accepted);
    stale["revision"] = json!("old");
    assert!(code(&h.outcome(session, review(&stale, "work", "accept"))).starts_with("credentials-refused"));
    assert!(code(&h.outcome(session, review(&card(accepted), "home", "accept"))).starts_with("credentials-refused"));
    assert!(matches!(answer(h.outcome(session, review(&card(accepted), "work", "accept"))), Answer::Done {} | Answer::Value { .. }));
    assert!(matches!(answer(h.outcome(session, review(&card(deleted), "work", "delete"))), Answer::Done {} | Answer::Value { .. }));
    assert!(code(&h.outcome(session, review(&card(accepted), "work", "accept"))).starts_with("credentials-refused"), "no replay");
    {
        let stores = h.core.custody().credential_stores();
        let work = &stores[&ProfileId("work".into())];
        let items = work.items(accepted).list().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].state, chatelaine::ItemState::Vault);
        assert!(work.items(deleted).list().unwrap().is_empty());
    }

    let operation = Operation::Credentials { operation: CredentialOperation::UnassignedImports {} };
    let Answer::Value { value } = answer(h.outcome(session, operation)) else { panic!() };
    let import = value[0].clone();
    assert_eq!(import["persona"], json!(older));
    let before = h.core.custody().credential_stores()[&ProfileId("work".into())].items(older).list().unwrap();
    let assign = Operation::Credentials {
        operation: CredentialOperation::AssignImport {
            request: json!({"store": "work", "profile": "home", "persona": older, "revision": import["revision"]}),
        },
    };
    answer(h.outcome(session, assign.clone()));
    let work = &h.core.custody().credential_stores()[&ProfileId("work".into())];
    assert_eq!(work.items(older).list().unwrap(), before, "assignment leaves the items as they were");
    assert!(work.profile_personas(&ProfileId("home".into())).unwrap().contains(&older));
    assert!(code(&h.outcome(session, assign)).starts_with("credentials-refused"), "assigned once");
}

#[test]
fn passphrase_change_and_ssh_import_need_the_manage_grant() {
    let dir = vault();
    let mut bare = Harness::new(config(&dir, "deny", base(json!([]))));
    let session = bare.open(djinn());
    bare.unlock(session);
    let change = |old: &str, new: &str| Operation::ChangePassphrase { old: Secret::copy_from(old.as_bytes()), new: Secret::copy_from(new.as_bytes()) };
    assert_eq!(code(&bare.outcome(session, change(PASSPHRASE, "a new passphrase"))), "operation-denied");
    drop(bare);

    let mut h = Harness::new(config(&dir, "deny", base(json!([{"operation": "manage", "consent": "none"}, {"operation": "snapshot"}]))));
    let session = h.open(djinn());
    h.unlock(session);
    assert!(code(&h.outcome(session, change("not it", "a new passphrase"))).starts_with("passphrase-change-refused"));
    assert!(matches!(answer(h.outcome(session, change(PASSPHRASE, "a new passphrase"))), Answer::Done {}));
    assert!(bootstrap::open_storage(&dir.path().join("vault"), Unlock::passphrase("a new passphrase")).is_ok());
    assert!(bootstrap::open_storage(&dir.path().join("vault"), Unlock::passphrase(PASSPHRASE)).is_err());

    let key = ssh_key::PrivateKey::from(ssh_key::private::Ed25519Keypair::from_seed(&[0x41; 32]));
    let openssh = key.to_openssh(ssh_key::LineEnding::LF).unwrap();
    let import = Operation::ImportSshKey {
        key: Secret::copy_from(openssh.as_bytes()),
        passphrase: None,
        options: Bytes(serde_json::to_vec(&json!({"unlock_policy": {"kind": "session"}})).unwrap()),
    };
    assert!(matches!(answer(h.outcome(session, import)), Answer::Value { .. }));
    let Answer::Snapshot { snapshot } = answer(h.outcome(session, Operation::Snapshot {})) else { panic!() };
    let fingerprint = key.public_key().fingerprint(ssh_key::HashAlg::Sha256).to_string();
    assert!(snapshot.ssh_keys.iter().any(|k| serde_json::to_string(k).unwrap().contains(&fingerprint)));
    let garbage = Operation::ImportSshKey { key: Secret::copy_from(b"not a key"), passphrase: None, options: Bytes(br#"{"unlock_policy":{"kind":"session"}}"#.to_vec()) };
    assert!(code(&h.outcome(session, garbage)).starts_with("import-refused"));
}

/// One SSH signature through the custodian's agent, on its own thread.
fn sign_through_agent(mut agent: castellan::custody::agent::VaultAgent<Box<dyn IdentityStorage>>, public: ssh_key::public::KeyData) -> std::thread::JoinHandle<Result<ssh_key::Signature, String>> {
    use ssh_agent_lib::agent::Session;
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
        runtime.block_on(async move {
            agent
                .sign(ssh_agent_lib::proto::SignRequest { credential: public.into(), data: b"ssh payload".to_vec(), flags: 0 })
                .await
                .map_err(|e| e.to_string())
        })
    })
}

#[test]
fn per_use_ssh_signing_is_petitioned_to_the_consent_channel() {
    let dir = vault();
    let mut h = Harness::new(config(&dir, "channel", base(json!([{"operation": "manage", "consent": "none"}]))));
    let session = h.open(djinn());
    let helper = h.open(consent_helper());
    h.unlock(session);
    let key = ssh_key::PrivateKey::from(ssh_key::private::Ed25519Keypair::from_seed(&[0x42; 32]));
    let import = Operation::ImportSshKey {
        key: Secret::copy_from(key.to_openssh(ssh_key::LineEnding::LF).unwrap().as_bytes()),
        passphrase: None,
        options: Bytes(br#"{"unlock_policy":{"kind":"per_use"}}"#.to_vec()),
    };
    answer(h.outcome(session, import));
    let public = key.public_key().key_data().clone();

    for accept in [true, false] {
        let signer = sign_through_agent(h.core.custody().agent_session().unwrap(), public.clone());
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        let petition = loop {
            let effects = h.core.petition_signing(h.now);
            if let Some(petition) = petition_in(&effects, helper) {
                break petition;
            }
            assert!(std::time::Instant::now() < deadline, "no petition");
            std::thread::sleep(Duration::from_millis(20));
        };
        assert_eq!(petition.operation, "ssh-sign");
        assert!(petition_in(&h.core.petition_signing(h.now), helper).is_none(), "one petition per request");
        let (recorded, _) = h.core.decide(Decider::Session(helper), petition.request, &petition.digest, petition.generation, accept, h.now);
        recorded.unwrap();
        let signed = signer.join().unwrap();
        match accept {
            true => {
                let signature = signed.expect("approved on the consent channel");
                let key = personae::Ed25519PublicKey::from_bytes(&public.ed25519().unwrap().0).unwrap();
                let raw: [u8; 64] = signature.as_bytes().try_into().unwrap();
                assert!(key.verify(b"ssh payload", &personae::Ed25519Signature::from_bytes(&raw)));
            },
            false => assert!(signed.is_err(), "a denial reaches the waiting client at once"),
        }
    }
}

#[test]
fn the_door_rule_releases_exactly_the_open_profiles_door_keys() {
    let dir = vault();
    let grants = base(json!([{"operation": "release", "profile": "any", "salts": [{"named": "djinn-door"}], "consent": "none"}]));
    let mut h = Harness::new(config(&dir, "deny", grants));
    let session = h.open(djinn());
    h.unlock(session);
    let derive = |profile: Option<&str>, salts: Vec<Vec<u8>>| Operation::Derive {
        source: Source::Persona,
        profile: profile.map(str::to_string),
        salts: salts.into_iter().map(Bytes).collect(),
    };
    let work = direct(dir.path(), "work");
    let door = crate::policy::door_salts(&work.master_public_key().to_bytes());
    let Answer::Released { keys, .. } = answer(h.outcome(session, derive(None, door.to_vec()))) else { panic!() };
    for (key, salt) in keys.iter().zip(&door) {
        assert_eq!(key.seed.expose(), work.derive_keypair(salt).unwrap().to_seed());
    }
    let guest = insigne::delegation::delegation_signing_salt(&insigne::delegation::CapabilityScope {
        domain: "mere.graphshell".into(),
        resource: b"guest-room".to_vec(),
        path_prefix: "/".into(),
        actions: Default::default(),
    });
    assert_eq!(code(&h.outcome(session, derive(None, vec![guest]))), "operation-or-scope", "a guest scope key stays sign-only");
    let home = crate::policy::door_salts(&direct(dir.path(), "home").master_public_key().to_bytes());
    assert_eq!(code(&h.outcome(session, derive(Some("home"), home.to_vec()))), "operation-or-scope", "only the open profile's doors");
}

#[test]
fn the_installed_grants_parse_on_every_platform_and_cover_djinns_own_keys() {
    let template = include_str!("../../packaging/djinn-grants.json.in");
    for (consent, sign) in [("per-generation", "per-operation"), ("none", "none")] {
        let grants: Value = serde_json::from_str(&template.replace("@CONSENT@", consent).replace("@SIGN_CONSENT@", sign)).unwrap();
        let dir = vault();
        let mut h = Harness::new(config(&dir, "deny", grants));
        let session = h.open(djinn());
        h.unlock(session);
        if consent != "none" {
            continue;
        }
        let master = direct(dir.path(), "work").master_public_key().to_bytes();
        let graph = [0x6e; 32];
        let mut writer = b"graphshell.personal-graph.writer.v1/".to_vec();
        writer.extend_from_slice(&graph);
        let mut prekey = b"stickleback/group-prekey-identity/v1".to_vec();
        prekey.extend_from_slice(&graph);
        let batches: Vec<Vec<Vec<u8>>> = vec![
            crate::policy::door_salts(&master).to_vec(),
            vec![b"mesh-author".to_vec(), b"distillery-personal-mesh".to_vec()],
            vec![writer],
            vec![prekey],
            vec![b"graphshell/endpoint-session/coop/v1".to_vec()],
        ];
        for salts in batches {
            let derive = Operation::Derive { source: Source::Persona, profile: None, salts: salts.into_iter().map(Bytes).collect() };
            assert!(matches!(h.outcome(session, derive), Outcome::Success { .. }));
        }
        let guest = insigne::delegation::delegation_signing_salt(&insigne::delegation::CapabilityScope {
            domain: "mere.graphshell".into(),
            resource: b"guest-room".to_vec(),
            path_prefix: "/".into(),
            actions: Default::default(),
        });
        let derive = Operation::Derive { source: Source::Persona, profile: None, salts: vec![Bytes(guest)] };
        assert_eq!(code(&h.outcome(session, derive)), "operation-or-scope", "guest delegations stay sign-only");
    }
}
