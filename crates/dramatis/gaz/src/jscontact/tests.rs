// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use personae::{IdentityProvider, InMemoryProvider};
use serde_json::json;

use super::*;
use crate::{ContactTier, Endpoint, Handle, KeyProof, LocalId, PlcDid, ProofMethod, TrustState};

fn format() -> JsContactFormat {
    JsContactFormat::new("example.org").unwrap()
}

fn key(seed: u8) -> TypedKey {
    TypedKey::ed25519([seed; 32])
}

fn local() -> LocalId {
    LocalId::from_random([9; 16])
}

fn rich(mut contact: Contact) -> Contact {
    contact.tier = ContactTier::Kin;
    contact.note = Some("PRIVATE NOTE".into());
    contact.last_contact_ms = Some(u64::MAX);
    contact
        .handles
        .push(
            Handle::acct("acct:Alice@example.org").with_binding(TrustState::Verified {
                method: ProofMethod::BackClaim,
                at_ms: u64::MAX,
            }),
        );
    let mut endpoint =
        Endpoint::new(EndpointKind::Misfin, "alice@example.org").with_trust(TrustState::Pinned {
            first_seen_ms: u64::MAX,
        });
    endpoint.last_used_ms = Some(u64::MAX);
    contact.endpoints.push(endpoint);
    contact.endpoints.push(
        Endpoint::new(EndpointKind::Http, "https://example.org/alice")
            .with_trust(TrustState::Revoked { at_ms: 12 }),
    );
    contact
}

fn public(contact: &Contact) -> PublicCard {
    PublicCard {
        name: "PUBLIC NAME".into(),
        identity: IdentityClaim {
            anchor: contact.anchor().clone(),
            root_line: contact.root_line().to_vec(),
            attested: contact.attested().to_vec(),
        },
        handles: contact
            .handles
            .iter()
            .map(|h| PublicHandle {
                kind: h.kind.clone(),
                value: h.value.clone(),
            })
            .collect(),
        endpoints: contact
            .endpoints
            .iter()
            .map(|e| PublicEndpoint {
                kind: e.kind.clone(),
                address: e.address.clone(),
            })
            .collect(),
    }
}

fn unverified(contact: &Contact) {
    assert_eq!(contact.tier, ContactTier::Kith);
    assert_eq!(contact.note, None);
    assert_eq!(contact.last_contact_ms, None);
    for handle in &contact.handles {
        assert_eq!(handle.binding, TrustState::Unverified);
    }
    for endpoint in &contact.endpoints {
        assert_eq!(endpoint.trust, TrustState::Unverified);
        assert_eq!(endpoint.last_used_ms, None);
    }
}

#[test]
fn private_round_trip_preserves_all_anchor_families_and_exact_local_history() {
    let contacts = [
        Contact::new("private petname", key(1)),
        Contact::new("reticulum", TypedKey::reticulum([2; 64])),
        Contact::new("secp", TypedKey::secp256k1([3; 33]).unwrap()),
        Contact::new("p256", TypedKey::p256([2; 33]).unwrap()),
        Contact::new_local("local", local()),
        Contact::new_plc(
            "plc",
            PlcDid::parse("did:plc:abcdefghijklmnopqrstuvwx").unwrap(),
            key(5),
        ),
    ];
    for contact in contacts {
        let contact = rich(contact);
        let card = format().export_contact(&contact).unwrap();
        let bytes = card.to_json().unwrap();
        let parsed = Card::parse(&bytes).unwrap();
        assert_eq!(format().restore_contact(&parsed).unwrap(), contact);
        assert!(parsed.as_value()["example.org:gazLocal"]["contactJson"].is_string());
        for (_, key) in super::card::entries(parsed.as_value(), "cryptoKeys").unwrap() {
            assert!(super::card::uri(key["uri"].as_str().unwrap()));
        }
        unverified(&format().import(&parsed, None).unwrap().contact);
    }
}

#[test]
fn peer_import_ignores_private_trust_even_when_the_backup_claims_it() {
    let contact = rich(Contact::new("Alice", key(1)));
    let card = format().export_contact(&contact).unwrap();
    let imported = format().import(&card, None).unwrap();
    unverified(&imported.contact);
    assert_eq!(imported.source, card);
    assert_eq!(imported.contact.anchor(), contact.anchor());
    assert_eq!(imported.contact.handles[0].value, contact.handles[0].value);
    assert_eq!(imported.contact.endpoints.len(), contact.endpoints.len());
}

#[test]
fn publication_contains_only_explicit_public_fields() {
    let contact = rich(Contact::new("PRIVATE PETNAME", key(1)));
    let card = format().publish(&public(&contact)).unwrap();
    let text = String::from_utf8(card.to_json().unwrap()).unwrap();
    for private in [
        "PRIVATE PETNAME",
        "PRIVATE NOTE",
        "gazLocal",
        "Verified",
        "Pinned",
        "Revoked",
        "last_used_ms",
        "last_contact_ms",
        "\"Kin\"",
    ] {
        assert!(!text.contains(private), "leaked {private}");
    }
    assert_eq!(card.as_value()["name"]["full"], "PUBLIC NAME");
    let imported = format().import(&card, None).unwrap();
    unverified(&imported.contact);
    assert_eq!(imported.contact.handles[0].kind, HandleKind::Acct);
    assert!(
        imported
            .contact
            .endpoints
            .iter()
            .any(|e| e.kind == EndpointKind::Misfin && e.address == "alice@example.org")
    );
    assert!(format().restore_contact(&card).is_err());
}

#[test]
fn public_claims_keep_rotation_and_attestation_artifacts_for_caller_rechecking() {
    let issuer = InMemoryProvider::from_seed([1; 32]);
    let root = TypedKey::ed25519(issuer.master_public_key().to_bytes());
    let salt = b"device\0\xff";
    let attestation = issuer.attest_derived_key(salt).unwrap();
    let derived = attestation.derived_key();
    let mut contact = Contact::new("Alice", root);
    contact
        .attest(
            derived,
            "device",
            root,
            Some(KeyProof::Attestation {
                attestation,
                salt: salt.to_vec(),
            }),
        )
        .unwrap();
    contact
        .rotate_to(
            key(3),
            Some(KeyProof::Other {
                method: ProofMethod::DidAuth,
                format: "caller-operation/v1".into(),
                bytes: vec![0, 255, 13, 10],
            }),
        )
        .unwrap();
    let card = format().publish(&public(&contact)).unwrap();
    let imported = format()
        .import(&Card::parse(&card.to_json().unwrap()).unwrap(), None)
        .unwrap();
    assert_eq!(imported.contact.root_line(), contact.root_line());
    assert_eq!(imported.contact.attested(), contact.attested());
    let KeyProof::Attestation { attestation, salt } =
        imported.contact.attested()[0].proof.as_ref().unwrap()
    else {
        panic!("lost artifact")
    };
    assert_eq!(attestation.check(salt).unwrap().derived_key(), derived);
    unverified(&imported.contact);
    let mut corrupt = card.as_value().clone();
    corrupt["example.org:gazIdentity"]["identity"]["attested"][0]["key"] =
        serde_json::to_value(key(8)).unwrap();
    assert!(
        format()
            .import(&Card::from_value(corrupt).unwrap(), None)
            .is_err()
    );
}

#[test]
fn inconsistent_private_or_public_identity_is_refused_whole() {
    let card = format()
        .export_contact(&rich(Contact::new("Alice", key(1))))
        .unwrap();
    for field in [
        "uid",
        "name",
        "cryptoKeys",
        "onlineServices",
        "example.org:gazIdentity",
    ] {
        let mut value = card.as_value().clone();
        value.as_object_mut().unwrap().remove(field);
        if let Ok(card) = Card::from_value(value) {
            assert!(format().restore_contact(&card).is_err(), "{field}");
        }
    }
    let mut value = card.as_value().clone();
    value["uid"] = json!(key(2).to_string());
    assert!(
        format()
            .import(&Card::from_value(value).unwrap(), None)
            .is_err()
    );
    let mut value = card.as_value().clone();
    value["cryptoKeys"] = json!({});
    assert!(
        format()
            .import(&Card::from_value(value).unwrap(), None)
            .is_err()
    );
    let card = Card::from_value(
        json!({"@type":"Card","version":"1.0","uid":"did:plc:abcdefghijklmnopqrstuvwx"}),
    )
    .unwrap();
    assert!(format().import(&card, Some(local())).is_err());
}

#[test]
fn foreign_cards_preserve_original_ids_unknown_properties_and_free_text_uid() {
    let value = json!({"@type":"Card","version":"1.1","uid":"foreign user 42",
        "name":{"components":[{"kind":"given","value":"Alice"},{"kind":"surname","value":"Example"}]},
        "onlineServices":{"original_id":{"service":"AcCt","user":"alice","uri":"https://example.org/alice","elsewhere.org:opaque":{"extra":true,"~":"anything"}}},
        "emails":{"mail":{"address":"alice@example.org"}},"phones":{"phone":{"number":"+1 555 0123"}},
        "addresses":{"untouched":{"full":"somewhere"}},"futureProperty":[1,2,3],"elsewhere.org:opaque":{"extra":null}
    });
    let card = Card::from_value(value.clone()).unwrap();
    assert_eq!(
        format().import(&card, None).unwrap_err(),
        ExchangeError::LocalAnchorRequired
    );
    let imported = format().import(&card, Some(local())).unwrap();
    assert_eq!(imported.contact.anchor(), &Anchor::Local(local()));
    assert_eq!(imported.contact.petname, "Alice Example");
    assert_eq!(imported.contact.handles[0].kind, HandleKind::Acct);
    assert_eq!(imported.contact.endpoints.len(), 3);
    assert_eq!(
        Card::parse(&imported.source.to_json().unwrap())
            .unwrap()
            .as_value(),
        &value
    );
    unverified(&imported.contact);
}

#[test]
fn bare_uuid_is_local_and_malformed_known_anchors_never_use_fallback() {
    let urn = local().to_urn();
    let bare = urn.strip_prefix("urn:uuid:").unwrap();
    let card = Card::from_value(json!({"@type":"Card","version":"1.0","uid":bare})).unwrap();
    assert_eq!(
        format().import(&card, None).unwrap().contact.anchor(),
        &Anchor::Local(local())
    );
    for uid in ["did:key:bad", "did:plc:bad", "urn:uuid:bad"] {
        let card = Card::from_value(json!({"@type":"Card","version":"1.0","uid":uid})).unwrap();
        assert!(format().import(&card, Some(local())).is_err());
    }
}

#[test]
fn foreign_crypto_resources_never_invent_key_roles_or_fetch_remote_keys() {
    let value = json!({"@type":"Card","version":"1.0","uid":key(1).to_string(),"cryptoKeys":{
        "root":{"uri":key(1).to_string()},"device":{"uri":key(2).to_string()},"remote":{"uri":"https://example.org/key.asc"}
    }});
    let imported = format()
        .import(&Card::from_value(value).unwrap(), None)
        .unwrap();
    assert_eq!(imported.unbound_keys, vec![key(2)]);
    assert_eq!(imported.contact.root_line().len(), 1);
    assert!(imported.contact.attested().is_empty());
}

#[test]
fn stable_map_ids_survive_reorder_and_unrelated_additions() {
    let mut contact = rich(Contact::new("Alice", key(1)));
    contact.handles.push(Handle::new(
        HandleKind::Other("matrix".into()),
        "@alice:example.org",
    ));
    let first = format().export_contact(&contact).unwrap();
    contact.handles.reverse();
    contact.endpoints.reverse();
    let second = format().export_contact(&contact).unwrap();
    assert_eq!(
        first.as_value()["onlineServices"],
        second.as_value()["onlineServices"]
    );
    contact
        .endpoints
        .push(Endpoint::new(EndpointKind::Gemini, "gemini://example.org/"));
    let third = format().export_contact(&contact).unwrap();
    for (id, value) in first.as_value()["onlineServices"].as_object().unwrap() {
        assert_eq!(&third.as_value()["onlineServices"][id], value);
    }
    contact.handles.push(contact.handles[0].clone());
    let duplicate = format().export_contact(&contact).unwrap();
    assert_eq!(format().restore_contact(&duplicate).unwrap(), contact);
}

#[test]
fn reticulum_resource_contains_the_whole_identity_and_uid_keeps_rnid_text() {
    let key = TypedKey::reticulum([0xab; 64]);
    let card = format()
        .publish(&public(&Contact::new("Alice", key)))
        .unwrap();
    assert_eq!(card.uid(), key.to_string());
    let uri = card.as_value()["cryptoKeys"]
        .as_object()
        .unwrap()
        .values()
        .next()
        .unwrap()["uri"]
        .as_str()
        .unwrap();
    assert_eq!(
        uri,
        format!("data:application/octet-stream,{}", "%ab".repeat(64))
    );
    assert_eq!(
        format().import(&card, None).unwrap().contact.root(),
        Some(&key)
    );
}

#[test]
fn mapped_invalid_shapes_and_duplicate_json_properties_are_refused() {
    let base = json!({"@type":"Card","version":"1.0","uid":"foreign"});
    for (name, value) in [
        ("version", json!("+1.0")),
        ("version", json!("2.0")),
        ("uid", json!(null)),
        ("Uid", json!("wrongcase")),
        ("extra", json!({})),
        ("bad/name", json!(1)),
        ("name", json!({})),
        ("name", json!({"components":[]})),
        (
            "name",
            json!({"components":[{"kind":"separator","value":" "}]}),
        ),
        ("name", json!({"full":"Alice","isOrdered":"true"})),
        ("onlineServices", json!({"bad/id":{"user":"alice"}})),
        ("onlineServices", json!({"a":{}})),
        ("onlineServices", json!(null)),
        (
            "onlineServices",
            json!({"a":{"uri":"https://example.org/a b"}}),
        ),
        (
            "onlineServices",
            json!({"a":{"uri":"https://example.org/é"}}),
        ),
        (
            "onlineServices",
            json!({"a":{"uri":"https://example.org/%zz"}}),
        ),
        ("onlineServices", json!({"a":{"user":"alice","pref":101}})),
        (
            "onlineServices",
            json!({"a":{"user":"alice","contexts":{"work":"true"}}}),
        ),
        ("cryptoKeys", json!({"a":{"uri":"bare hex"}})),
        (
            "cryptoKeys",
            json!({"a":{"@type":"OnlineService","uri":"https://example.org/key"}}),
        ),
        ("emails", json!({"a":{"address":42}})),
        ("phones", json!({"a":{"number":null}})),
    ] {
        let mut card = base.clone();
        card[name] = value;
        assert!(Card::from_value(card).is_err(), "accepted {name}");
    }
    for bytes in [
        br#"{"@type":"Card","version":"1.0","uid":"a","uid":"b"}"#.as_slice(),
        br#"{"@type":"Card","version":"1.0","uid":"a","elsewhere.org:x":{"x":1,"x":2}}"#,
        br#"{"@type":"Card","version":"1.0","uid":"a"} {}"#,
    ] {
        assert!(Card::parse(bytes).is_err());
    }
}

#[test]
fn extension_vocabulary_is_host_selected_and_other_domains_stay_opaque() {
    for domain in [
        "ietf.org",
        "jscontact.ietf.org",
        "bad/domain",
        ".example.org",
        "-bad.example.org",
        "localhost",
    ] {
        assert!(JsContactFormat::new(domain).is_err());
    }
    let card = format()
        .export_contact(&rich(Contact::new("Alice", key(1))))
        .unwrap();
    let other = JsContactFormat::new("ELSEWHERE.ORG").unwrap();
    assert!(other.restore_contact(&card).is_err());
    let imported = other.import(&card, None).unwrap();
    unverified(&imported.contact);
    assert_eq!(imported.source, card);
}

#[test]
fn every_advertised_protocol_keeps_its_kind_and_exact_address() {
    let mut card = public(&Contact::new("Alice", key(1)));
    card.handles = [
        HandleKind::Acct,
        HandleKind::Did,
        HandleKind::Nostr,
        HandleKind::Other("matrix".into()),
    ]
    .into_iter()
    .map(|kind| PublicHandle {
        kind,
        value: "non-URI user text".into(),
    })
    .collect();
    card.endpoints = [
        EndpointKind::Misfin,
        EndpointKind::Murm,
        EndpointKind::Gemini,
        EndpointKind::Gopher,
        EndpointKind::ActivityPub,
        EndpointKind::Http,
        EndpointKind::Other("custom".into()),
    ]
    .into_iter()
    .map(|kind| PublicEndpoint {
        kind,
        address: "caller protocol address".into(),
    })
    .collect();
    let imported = format()
        .import(&format().publish(&card).unwrap(), None)
        .unwrap();
    for handle in &card.handles {
        assert!(
            imported
                .contact
                .handles
                .iter()
                .any(|h| h.kind == handle.kind && h.value == handle.value)
        );
    }
    for endpoint in &card.endpoints {
        assert!(
            imported
                .contact
                .endpoints
                .iter()
                .any(|e| e.kind == endpoint.kind && e.address == endpoint.address)
        );
    }
    unverified(&imported.contact);
}

#[test]
fn ordered_name_separators_and_percent_encoded_uris_are_accepted() {
    let card=Card::from_value(json!({"@type":"Card","version":"1.0","uid":"foreign",
        "name":{"isOrdered":true,"defaultSeparator":"","components":[
            {"kind":"given","value":"Alice"},{"kind":"separator","value":", "},{"kind":"surname","value":"Example"}
        ]},"onlineServices":{"a":{"uri":"https://example.org/%C3%A9","contexts":{"work":false},"pref":100}}
    })).unwrap();
    let imported = format().import(&card, Some(local())).unwrap();
    assert_eq!(imported.contact.petname, "Alice, Example");
    assert_eq!(
        imported.contact.endpoints[0].address,
        "https://example.org/%C3%A9"
    );
}

#[test]
fn malformed_private_payload_cannot_select_restoration_on_peer_intake() {
    let mut card = format()
        .publish(&public(&Contact::new("Alice", key(1))))
        .unwrap()
        .as_value()
        .clone();
    card["example.org:gazLocal"] = json!({"version":"999","contactJson":"broken"});
    let card = Card::from_value(card).unwrap();
    unverified(&format().import(&card, None).unwrap().contact);
    assert!(format().restore_contact(&card).is_err());
}

#[test]
fn generated_ids_have_the_same_spelling_with_json_preserve_order() {
    let mut public = public(&Contact::new("Alice", key(1)));
    public.handles.push(PublicHandle {
        kind: HandleKind::Acct,
        value: "alice".into(),
    });
    let card = format().publish(&public).unwrap();
    assert_eq!(
        card.as_value()["onlineServices"]
            .as_object()
            .unwrap()
            .keys()
            .next()
            .unwrap(),
        "h61430c0b8902df9bb56bba2a7ed342d3"
    );
}
