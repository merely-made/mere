// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::authority::PersonaeHost;
use crate::custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage};
use crate::items::ItemStoreError;
use crate::resident::CastellanResident;
use base64::Engine;
use chatelaine::{CredentialKind, ItemState};
use dramatis::view::VaultProtectionView;
use personae::{Ed25519Keypair, SealedRecordStorage};
use rsa::pkcs8::EncodePrivateKey;
use serde_json::{Value, json};

const SECRET: &str = "synthetic-cxf-password-marker";

#[test]
fn unsupported_otp_counts_survive_import_and_cannot_be_exercised() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let counts = [0, 5, 11, 65535];
    let credentials = counts
        .into_iter()
        .map(|digits| {
            json!({"type":"totp","secret":"MY======",
        "algorithm":"sha1","period":300,"digits":digits})
        })
        .collect();
    let receipt = Import::read(document(vec![account(
        "YQ",
        vec![item("aQ", "OTP", credentials)],
    )]))
    .unwrap()
    .commit(choices(&["YQ"], persona), &resident, &host)
    .unwrap();
    let store = resident.items(persona);
    let item = store.list().unwrap().remove(0);
    assert_eq!(item.state, ItemState::Quarantined);
    assert_eq!(item.credentials.len(), counts.len());
    let storage = SealedRecordStorage::open_with_key(dir.path().join("records"), [0x61; 32]);
    for (credential, digits) in item.credentials.iter().zip(counts) {
        assert!(matches!(&credential.kind,CredentialKind::Otp {
            code_style:chatelaine::OtpCodeStyle::Decimal {digits:found},
            mode:chatelaine::OtpMode::Totp {period:300,..},.. } if *found==digits));
        assert!(
            receipt.notices.iter().any(|n| n.reason
                == format!("unsupported OTP digit count {digits} preserved in quarantine"))
        );
        let payload: Value = storage
            .load_record(store.payload_path(credential.id))
            .unwrap()
            .unwrap();
        assert_eq!(payload["payload"]["Otp"]["secret"], json!([102]));
        let released: Result<(), ItemStoreError> =
            store.exercise(item.id, credential.id, |_, _, _| Ok(((), false)));
        assert!(matches!(released, Err(ItemStoreError::Quarantined(_))));
    }
}

#[test]
fn nested_collections_resolve_aliases_and_report_dangling_links() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let mut a = account(
        "YQ",
        vec![
            item(
                "aQ",
                "Target",
                vec![json!({"type":"note","content":field(SECRET)})],
            ),
            item(
                "YQ",
                "Alias",
                vec![json!({"type":"item-reference","reference":{"item":"aQ"}})],
            ),
            item(
                "Yg",
                "Dangling",
                vec![json!({"type":"item-reference","reference":{"item":"Yw"}})],
            ),
        ],
    );
    a["collections"] = json!([{"id":"Yw","title":"Parent","creationAt":4,"modifiedAt":5,
        "items":[{"item":"YQ"},{"item":"Yg"}], "subCollections":[{"id":"ZA","title":"Child","items":[{"item":"aQ"}]}]}]);
    let receipt = Import::read(document(vec![a]))
        .unwrap()
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    assert_eq!(
        receipt.items, 1,
        "reference-only inputs must never mint items"
    );
    assert!(
        receipt
            .notices
            .iter()
            .filter(|n| n.reason == "dangling item link omitted")
            .count()
            >= 2
    );
    let store = resident.items(persona);
    let target = store.list().unwrap().remove(0).id;
    let index = store.begin().index().unwrap();
    let parent = &index.collections[0];
    assert_eq!(parent.source_id.as_ref().unwrap().to_string(), "Yw");
    assert_eq!((parent.created_at, parent.modified_at), (Some(4), Some(5)));
    assert_eq!(parent.items[0].item, target);
    assert_eq!(parent.items.len(), 1);
    assert_eq!(parent.sub_collections[0].items[0].item, target);
}

#[test]
fn enum_ignore_rules_and_sharing_do_not_create_local_grants() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let mut i = item(
        "aQ",
        "Shared",
        vec![
            json!({"type":"note","content":field(SECRET)}),
            json!({"type":"totp","secret":"MY","algorithm":"future-hash","period":30,"digits":6}),
            json!({"type":"passkey","fido2Extensions":{"hmacCredentials":{"algorithm":"future-hmac"}}}),
        ],
    );
    i["extensions"] = json!([{"name":"shared","accessors":[
        {"type":"future-user","accountId":"Yg","name":"Unknown","permissions":["read"]},
        {"type":"user","accountId":"Yg","name":"Unknown permission","permissions":["future-permission"]},
        {"type":"group","accountId":"Yg","name":"Mixed","permissions":["readSecret","future-permission"]},
        {"type":"user","accountId":"Yg","name":"Empty","permissions":[]}
    ]}]);
    let import = Import::read(document(vec![account("YQ", vec![i])])).unwrap();
    assert_eq!(import.preview().accounts[0].dispositions["ignored"], 2);
    let receipt = import
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    for expected in [
        "unknown sharing accessor type ignored",
        "unknown sharing permission ignored",
        "sharing accessor with no supported permissions ignored",
        "exporter sharing grant not recreated; local destination owns the import",
        "unknown TOTP algorithm ignored",
        "unknown passkey HMAC algorithm ignored",
    ] {
        assert!(
            receipt.notices.iter().any(|n| n.reason == expected),
            "missing notice: {expected}"
        );
    }
    let index = resident.items(persona).begin().index().unwrap();
    assert_eq!(index.items.len(), 1);
    assert!(index.collections.is_empty());
    assert_eq!(
        resident.items(persona).list().unwrap()[0].credentials.len(),
        1
    );
}

#[test]
fn imported_totp_exercises_existing_release_gate_and_locked_commit_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let input = || {
        document(vec![account(
            "YQ",
            vec![item(
                "aQ",
                "OTP",
                vec![json!({
                    "type":"totp","secret":"GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ","algorithm":"sha1","period":30,"digits":8
                })],
            )],
        )])
    };
    Import::read(input())
        .unwrap()
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    let item = resident.items(persona).list().unwrap().remove(0);
    let tile = resident
        .otp_items(persona)
        .release_tile_at_unix_time(item.id, item.credentials[0].id, 59)
        .unwrap();
    assert_eq!(tile.code_at_unix_time(59), Some("94287082"));
    let before = file_bytes(dir.path());
    host.lock_vault().unwrap();
    let failed = Import::read(input())
        .unwrap()
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap_err();
    assert_eq!(failed.reason, "vault is locked");
    assert_eq!(failed.receipt.items, 0);
    assert_eq!(file_bytes(dir.path()), before);
}

#[test]
fn persona_batch_has_one_index_commit_after_all_payloads_and_metadata() {
    use crate::items::WriteStep;
    use chatelaine::{Credential, CredentialId, Item, ItemId};
    for step in [WriteStep::Payloads, WriteStep::Metadata] {
        let dir = tempfile::tempdir().unwrap();
        let (_host, resident) = fixture(dir.path());
        let persona = PersonaId::new();
        let store = resident.items(persona).crashing_after(step);
        let batch = (0..2)
            .map(|n| {
                let credential = CredentialId::from_random(crate::items::random_id_bytes());
                let item = Item {
                    id: ItemId::from_random(crate::items::random_id_bytes()),
                    source_id: None,
                    title: format!("Item {n}"),
                    subtitle: None,
                    scope: None,
                    tags: vec![],
                    favorite: false,
                    created_at: None,
                    modified_at: None,
                    state: ItemState::Vault,
                    credentials: vec![Credential {
                        id: credential,
                        kind: CredentialKind::Note,
                    }],
                };
                (
                    item,
                    vec![(
                        credential,
                        Payload::Secret {
                            bytes: SECRET.as_bytes().to_vec(),
                        },
                    )],
                )
            })
            .collect();
        assert!(store.begin().import_batch(batch, vec![]).is_err());
        let fresh = resident.items(persona);
        assert!(fresh.list().unwrap().is_empty());
        assert!(fresh.begin().index().unwrap().collections.is_empty());
        assert!(
            file_bytes(&dir.path().join("records"))
                .values()
                .all(|bytes| !bytes.windows(SECRET.len()).any(|w| w == SECRET.as_bytes()))
        );
    }
}
fn file_bytes(root: &std::path::Path) -> BTreeMap<std::path::PathBuf, Vec<u8>> {
    fn visit(
        root: &std::path::Path,
        dir: &std::path::Path,
        out: &mut BTreeMap<std::path::PathBuf, Vec<u8>>,
    ) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            // This live exclusive lease contains no record data and Windows
            // deliberately refuses reads while the resident owns it.
            if path
                .file_name()
                .is_some_and(|name| name == ".personae-authority.lock")
            {
                continue;
            }
            if path.is_dir() {
                visit(root, &path, out);
            } else {
                out.insert(
                    path.strip_prefix(root).unwrap().into(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut out = BTreeMap::new();
    visit(root, root, &mut out);
    out
}
fn field(value: &str) -> Value {
    json!({"fieldType":"string","value":value})
}
fn all_types() -> Vec<Value> {
    let der = ed25519_dalek::SigningKey::from_bytes(&[0x41; 32])
        .to_pkcs8_der()
        .unwrap();
    let b64 = |bytes: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
    vec![
        json!({"type":"address","streetAddress":field(SECRET)}),
        json!({"type":"api-key","key":{"fieldType":"concealed-string","value":SECRET},"username":field("api-user")}),
        json!({"type":"basic-auth","username":field("login-user"),"password":{"fieldType":"concealed-string","value":SECRET}}),
        json!({"type":"credit-card","number":{"fieldType":"concealed-string","value":"4111111111111111"}}),
        json!({"type":"custom-fields","fields":[field(SECRET)]}),
        json!({"type":"drivers-license","licenseNumber":field(SECRET)}),
        json!({"type":"file","id":"ZmlsZQ","name":"receipt.txt","decryptedSize":12,"integrityHash":b64(&[8;32])}),
        json!({"type":"generated-password","password":SECRET}),
        json!({"type":"identity-document","documentNumber":field(SECRET)}),
        json!({"type":"item-reference","reference":{"item":"dGFyZ2V0"}}),
        json!({"type":"note","content":field(SECRET)}),
        json!({"type":"passkey","credentialId":"Y3JlZA","rpId":"example.invalid","username":"passkey-user","userDisplayName":"User","userHandle":"dXNlcg","key":"a2V5"}),
        json!({"type":"passport","passportNumber":field(SECRET)}),
        json!({"type":"person-name","given":field(SECRET)}),
        json!({"type":"ssh-key","keyType":"ssh-ed25519","privateKey":b64(der.as_bytes()),"keyComment":"CXF receipt"}),
        json!({"type":"totp","secret":"GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ","period":30,"digits":8,"algorithm":"sha1"}),
        json!({"type":"wifi","ssid":field("receipt-network"),"passphrase":{"fieldType":"concealed-string","value":SECRET}}),
        json!({"type":"future-type","secret":SECRET,"future":17}),
    ]
}
fn item(id: &str, title: &str, credentials: Vec<Value>) -> Value {
    json!({"id":id,"title":title,"credentials":credentials})
}
fn account(id: &str, items: Vec<Value>) -> Value {
    json!({"id":id,"username":"receipt","email":"receipt@example.invalid","collections":[],"items":items})
}
fn document(accounts: Vec<Value>) -> Vec<u8> {
    serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":accounts})).unwrap()
}
fn fixture(root: &std::path::Path) -> (PersonaeHost<SealedProfileStorage>, CastellanResident) {
    let storage = SealedProfileStorage::open_with_key(root.join("vault"), [0x29; 32]);
    storage.enroll_passphrase(b"synthetic cxf receipt").unwrap();
    let profile = Profile::new(
        ProfileId("family".into()),
        "Family",
        Ed25519Keypair::from_seed([0x39; 32]),
    );
    storage.save_profile(&profile).unwrap();
    let host = PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::OsProtected,
    );
    let resident = CastellanResident::claim(
        root.join("records"),
        [0x61; 32],
        root.join("freshness"),
        [0x62; 32],
    )
    .unwrap();
    (host, resident)
}
fn choices(account_ids: &[&str], persona: PersonaId) -> BTreeMap<String, Destination> {
    account_ids
        .iter()
        .map(|id| {
            (
                (*id).into(),
                Destination {
                    persona,
                    profile: ProfileId("family".into()),
                    ssh_unlock_policy: dramatis::intents::SshUnlockPolicyIntentV1::Session,
                },
            )
        })
        .collect()
}

#[test]
fn all_seventeen_types_and_future_type_use_the_ruled_dispositions() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let input = document(vec![account(
        "YWNjb3VudA",
        vec![
            item("c291cmNl", "Source", all_types()),
            item(
                "dGFyZ2V0",
                "Target",
                vec![json!({"type":"note","content":field(SECRET)})],
            ),
        ],
    )]);
    let import = Import::read(input).unwrap();
    assert!(
        resident.items(persona).list().unwrap().is_empty(),
        "read must write nothing"
    );
    let summary = &import.preview().accounts[0].dispositions;
    assert_eq!(summary["stored"], 10);
    assert_eq!(summary["quarantined"], 7);
    assert_eq!(summary["ssh"], 1);
    assert_eq!(summary["link"], 1);
    assert!(
        !serde_json::to_string(import.preview())
            .unwrap()
            .contains(SECRET)
    );
    let receipt = import
        .commit(choices(&["YWNjb3VudA"], persona), &resident, &host)
        .unwrap();
    assert_eq!(receipt.items, 3);
    assert_eq!(receipt.ssh_keys, 1);
    assert!(receipt.delete_plaintext_source);
    let items = resident.items(persona).list().unwrap();
    for (ty, state) in [
        ("address", ItemState::Vault),
        ("api-key", ItemState::Vault),
        ("basic-auth", ItemState::Vault),
        ("credit-card", ItemState::Quarantined),
        ("custom-fields", ItemState::Vault),
        ("drivers-license", ItemState::Quarantined),
        ("file", ItemState::Quarantined),
        ("generated-password", ItemState::Vault),
        ("identity-document", ItemState::Quarantined),
        ("item-reference", ItemState::Vault),
        ("note", ItemState::Vault),
        ("passkey", ItemState::Quarantined),
        ("passport", ItemState::Quarantined),
        ("person-name", ItemState::Vault),
        ("totp", ItemState::Vault),
        ("wifi", ItemState::Vault),
        ("future-type", ItemState::Quarantined),
    ] {
        assert!(
            items
                .iter()
                .filter(|i| i.state == state)
                .flat_map(|i| &i.credentials)
                .any(|c| c.kind.cxf_type() == Some(ty)),
            "missing {ty} in {state:?}"
        );
    }
    assert!(
        !items
            .iter()
            .flat_map(|i| &i.credentials)
            .any(|c| c.kind.cxf_type() == Some("ssh-key")),
        "SSH belongs only in native slots"
    );
    assert_eq!(
        items
            .iter()
            .filter(|i| i.state == ItemState::Quarantined)
            .count(),
        1
    );
    let metadata = serde_json::to_string(&items).unwrap();
    assert!(!metadata.contains(SECRET));
    assert!(!metadata.contains("4111111111111111"));
    assert!(items.iter().flat_map(|i| &i.credentials).any(
        |c| matches!(&c.kind,CredentialKind::CreditCard{last_four:Some(v),..} if v.as_str()=="1111")
    ));
    assert!(
        items
            .iter()
            .flat_map(|i| &i.credentials)
            .any(|c| matches!(&c.kind,CredentialKind::Otp{account,..} if account=="Source"))
    );
    assert!(
        receipt
            .notices
            .iter()
            .any(|n| n.reason.contains("item title"))
    );
}

#[test]
fn quarantine_refuses_exercise_before_opening_payload() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    Import::read(document(vec![account("YQ",vec![item("aQ","Card",vec![json!({"type":"credit-card","number":{"fieldType":"concealed-string","value":"4111111111111111"}})])])])).unwrap().commit(choices(&["YQ"],persona),&resident,&host).unwrap();
    let store = resident.items(persona);
    let item = store.list().unwrap().remove(0);
    let credential = item.credentials[0].id;
    let exercised: Result<(), ItemStoreError> =
        store.exercise(item.id, credential, |_, _, _| Ok(((), false)));
    assert!(
        matches!(exercised, Err(ItemStoreError::Quarantined(_))),
        "quarantine must refuse before the exercising closure runs"
    );
}

#[test]
fn unknown_credential_bytes_are_preserved_exactly_in_sealed_payload() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let raw = r#"{ "type" : "future-type", "str" : "escaped\u0020value", "n":1e+02 }"#;
    let mut input = String::from_utf8(document(vec![account(
        "YQ",
        vec![item("aQ", "Future", vec![json!({"type":"placeholder"})])],
    )]))
    .unwrap();
    input = input.replace(r#"{"type":"placeholder"}"#, raw);
    Import::read(input.into_bytes())
        .unwrap()
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    let store = resident.items(persona);
    let item = store.list().unwrap().remove(0);
    let credential = item.credentials[0].id;
    let storage = SealedRecordStorage::open_with_key(dir.path().join("records"), [0x61; 32]);
    let payload: Value = storage
        .load_record(store.payload_path(credential))
        .unwrap()
        .unwrap();
    let bytes: Vec<u8> =
        serde_json::from_value(payload["payload"]["Secret"]["bytes"].clone()).unwrap();
    assert_eq!(bytes, raw.as_bytes());
}

#[test]
fn known_failures_are_not_classified_as_new_types_and_totp_uses_original_base32() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let credentials = vec![
        json!({"type":"totp","secret":"M!Y","algorithm":"sha1","period":300,"digits":8}),
        json!({"type":"totp","secret":"my======","algorithm":"sha1","period":300,"digits":8}),
        json!({"type":"file","name":"broken"}),
        json!({"type":"note"}),
    ];
    let import = Import::read(document(vec![account(
        "YQ",
        vec![item("aQ", "OTP", credentials)],
    )]))
    .unwrap();
    assert_eq!(import.preview().accounts[0].dispositions["quarantined"], 3);
    assert_eq!(import.preview().accounts[0].dispositions["stored"], 1);
    import
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    let items = resident.items(persona).list().unwrap();
    assert!(
        !items
            .iter()
            .flat_map(|i| &i.credentials)
            .any(|c| matches!(c.kind, CredentialKind::Unknown { .. }))
    );
    assert!(items.iter().flat_map(|i| &i.credentials).any(|c| matches!(
        c.kind,
        CredentialKind::Otp {
            mode: chatelaine::OtpMode::Totp { period: 300, .. },
            ..
        }
    )));
}

#[test]
fn account_choices_are_complete_before_writes_and_cross_persona_links_are_omitted() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let left = PersonaId::new();
    let right = PersonaId::new();
    let input = || {
        document(vec![
            account(
                "YQ",
                vec![item(
                    "aQ",
                    "Left",
                    vec![
                        json!({"type":"note","content":field(SECRET)}),
                        json!({"type":"item-reference","reference":{"account":"Yg","item":"ag"}}),
                    ],
                )],
            ),
            account(
                "Yg",
                vec![item(
                    "ag",
                    "Right",
                    vec![json!({"type":"note","content":field(SECRET)})],
                )],
            ),
        ])
    };
    let failed = Import::read(input())
        .unwrap()
        .commit(choices(&["YQ"], left), &resident, &host)
        .unwrap_err();
    assert_eq!(failed.receipt.items, 0);
    assert!(resident.items(left).list().unwrap().is_empty());
    let mut destinations = choices(&["YQ"], left);
    destinations.extend(choices(&["Yg"], right));
    let receipt = Import::read(input())
        .unwrap()
        .commit(destinations, &resident, &host)
        .unwrap();
    assert!(
        receipt
            .notices
            .iter()
            .any(|n| n.reason == "cross-persona link omitted")
    );
    assert!(
        !resident.items(left).list().unwrap()[0]
            .credentials
            .iter()
            .any(|c| matches!(c.kind, CredentialKind::ItemReference(_)))
    );
    let same = PersonaId::new();
    let receipt = Import::read(input())
        .unwrap()
        .commit(choices(&["YQ", "Yg"], same), &resident, &host)
        .unwrap();
    assert!(
        !receipt
            .notices
            .iter()
            .any(|n| n.reason == "cross-persona link omitted")
    );
    assert!(
        resident
            .items(same)
            .list()
            .unwrap()
            .iter()
            .flat_map(|i| &i.credentials)
            .any(|c| matches!(c.kind, CredentialKind::ItemReference(_)))
    );
}

#[test]
fn invalid_version_or_missing_string_type_refuses_the_document() {
    let mut v: Value = serde_json::from_slice(&document(vec![account("YQ", vec![])])).unwrap();
    v["version"]["major"] = json!(2);
    assert!(Import::read(serde_json::to_vec(&v).unwrap()).is_err());
    for credential in [json!({}), json!({"type":17})] {
        assert!(
            Import::read(document(vec![account(
                "YQ",
                vec![item("aQ", "Bad", vec![credential])]
            )]))
            .is_err()
        );
    }
}

#[test]
fn custom_fields_are_recovered_element_by_element_and_known_extra_fields_are_ignored() {
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let input = document(vec![account(
        "YQ",
        vec![item(
            "aQ",
            "Fields",
            vec![
                json!({"type":"custom-fields","label":"Section","fields":[{"fieldType":"string","label":"Good","value":SECRET},{"fieldType":"future-field","value":SECRET},{"fieldType":"date","value":"nonsense"}]}),
                json!({"type":"generated-password","password":SECRET,"futureField":SECRET}),
            ],
        )],
    )]);
    let import = Import::read(input).unwrap();
    assert_eq!(import.preview().accounts[0].dispositions["stored"], 2);
    assert_eq!(import.preview().notices.len(), 2);
    import
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    let store = resident.items(persona);
    let item = store.list().unwrap().remove(0);
    assert!(item.credentials.iter().any(|c|matches!(&c.kind,CredentialKind::CustomFields{field_labels,..} if field_labels==&vec![Some("Good".into())])));
    let password = item
        .credentials
        .iter()
        .find(|c| matches!(c.kind, CredentialKind::GeneratedPassword))
        .unwrap();
    let payload: Value = SealedRecordStorage::open_with_key(dir.path().join("records"), [0x61; 32])
        .load_record(store.payload_path(password.id))
        .unwrap()
        .unwrap();
    let bytes: Vec<u8> =
        serde_json::from_value(payload["payload"]["Secret"]["bytes"].clone()).unwrap();
    assert!(!String::from_utf8(bytes).unwrap().contains("futureField"));
}

#[tokio::test]
async fn pkcs8_keys_join_native_slots_preserve_existing_keys_and_reimport_is_a_noop() {
    use crate::custody::{IdentitySlot, ssh_slot};
    use dramatis::intents::{ImportSshKeyNativeIntentV1, SshUnlockPolicyIntentV1};
    use signature::Verifier;
    use ssh_agent_lib::agent::Session;
    use ssh_agent_lib::proto::SignRequest;
    let dir = tempfile::tempdir().unwrap();
    let (host, resident) = fixture(dir.path());
    let persona = PersonaId::new();
    let existing: ssh_key::PrivateKey =
        ssh_key::private::Ed25519Keypair::from(ed25519_dalek::SigningKey::from_bytes(&[0x11; 32]))
            .into();
    let existing_key = ssh_slot::protocol_key_for(&existing);
    host.import_ssh_private(
        existing,
        ImportSshKeyNativeIntentV1 {
            unlock_policy: SshUnlockPolicyIntentV1::Session,
        },
    )
    .unwrap();
    let load = || {
        SealedProfileStorage::open_with_key(dir.path().join("vault"), [0x29; 32])
            .load_profile(&ProfileId("family".into()))
            .unwrap()
    };
    let slot_bytes = |profile: &Profile| match &profile.slots[&existing_key] {
        IdentitySlot::Direct { payload, .. } => payload.as_slice().to_vec(),
        _ => panic!("native SSH slot"),
    };
    let before = slot_bytes(&load());
    let ed = ed25519_dalek::SigningKey::from_bytes(&[0x41; 32])
        .to_pkcs8_der()
        .unwrap();
    let rsa = rsa::RsaPrivateKey::new(&mut rand_core::OsRng, 2048)
        .unwrap()
        .to_pkcs8_der()
        .unwrap();
    let p256 = p256::SecretKey::from_slice(&[0x12; 32])
        .unwrap()
        .to_pkcs8_der()
        .unwrap();
    let p384 = p384::SecretKey::from_slice(&[0x13; 48])
        .unwrap()
        .to_pkcs8_der()
        .unwrap();
    let credentials=vec![("ssh-ed25519",ed),("ssh-rsa",rsa),("ecdsa-sha2-nistp256",p256),("ecdsa-sha2-nistp384",p384)].into_iter().map(|(algorithm,der)|json!({"type":"ssh-key","keyType":algorithm,"privateKey":base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(der.as_bytes()),"keyComment":"Imported"})).collect();
    let input = document(vec![account("YQ", vec![item("aQ", "SSH", credentials)])]);
    let first = Import::read(input.clone())
        .unwrap()
        .commit(choices(&["YQ"], persona), &resident, &host)
        .unwrap();
    assert_eq!(first.ssh_keys, 4);
    assert_eq!(first.items, 0);
    assert_eq!(slot_bytes(&load()), before);
    let files_before = file_bytes(&dir.path().join("vault"));
    let mut agent = host.agent_session();
    let identities = agent.request_identities().await.unwrap();
    assert_eq!(
        identities
            .iter()
            .filter(|identity| matches!(
                identity.credential,
                ssh_agent_lib::proto::PublicCredential::Key(_)
            ))
            .count(),
        5
    );
    assert_eq!(
        identities
            .iter()
            .filter(|identity| matches!(
                identity.credential,
                ssh_agent_lib::proto::PublicCredential::Cert(_)
            ))
            .count(),
        5
    );
    for identity in identities {
        let public = identity.credential.key_data().clone();
        let flags = if matches!(public, ssh_key::public::KeyData::Rsa(_)) {
            2
        } else {
            0
        };
        let signature = agent
            .sign(SignRequest {
                credential: identity.credential,
                data: b"CXF signing receipt".to_vec(),
                flags,
            })
            .await
            .unwrap();
        assert!(public.verify(b"CXF signing receipt", &signature).is_ok());
    }
    let mut repeat_choices = choices(&["YQ"], persona);
    repeat_choices.get_mut("YQ").unwrap().ssh_unlock_policy = SshUnlockPolicyIntentV1::PerUse;
    Import::read(input)
        .unwrap()
        .commit(repeat_choices, &resident, &host)
        .unwrap();
    assert_eq!(slot_bytes(&load()), before);
    assert_eq!(file_bytes(&dir.path().join("vault")), files_before);
}
