// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{WebFingerEndpoint, parse_document};
use gaz::intake::NewLocalContact;
use gaz::{Anchor, ContactBook, LocalId, PersonaScope, TrustState};

fn resolved(json: &str) -> WebFingerImport {
    WebFingerImport::from_document(&parse_document(json).unwrap())
}

fn local_contact() -> NewLocalContact {
    NewLocalContact {
        id: LocalId::from_random([7; 16]),
        petname: "Alice".into(),
    }
}

#[test]
fn selected_account_uses_gaz_normalization_and_retains_source() {
    let source = resolved(
        r#"{
        "subject":"acct:Alice@EXAMPLE.org",
        "aliases":["acct:Alice@other.example","did:web:example.org","unknown:opaque"]
    }"#,
    );
    let intake = WebFingerIntake::from_import("ACCT:%41lice@example.ORG", source.clone()).unwrap();
    assert_eq!(intake.resource(), "acct:Alice@example.org");
    assert_eq!(intake.addresses().handle.kind, HandleKind::Acct);
    assert_eq!(intake.addresses().handle.value, intake.resource());
    assert_eq!(intake.source(), &source);
    assert_eq!(intake.addresses().handles.len(), 2);
    assert_eq!(
        intake.addresses().handles[0].value,
        "acct:Alice@other.example"
    );
    assert_eq!(intake.addresses().handles[1].kind, HandleKind::Did);
}

#[test]
fn bracketed_ipv6_account_hosts_use_the_same_core_normalization() {
    let source = resolved(r#"{"subject":"acct:alice@[2001:DB8::1]"}"#);
    let intake = WebFingerIntake::from_import("alice@[2001:db8::1]", source).unwrap();
    assert_eq!(intake.resource(), "acct:alice@[2001:db8::1]");
}

#[test]
fn changing_the_subject_or_user_case_is_explicitly_refused() {
    for subject in [
        "acct:bob@example.org",
        "acct:Alice@example.org",
        "acct:alice@other.example",
    ] {
        let source = WebFingerImport {
            subject: subject.into(),
            aliases: vec!["acct:alice@example.org".into()],
            ..WebFingerImport::default()
        };
        assert!(matches!(
            WebFingerIntake::from_import("alice@example.org", source),
            Err(WebFingerIntakeError::SubjectMismatch { .. })
        ));
    }
}

#[test]
fn invalid_query_or_subject_never_falls_back_to_an_alias() {
    for query in [
        "",
        "acct:alice@",
        "acct:alice@host/path",
        "https://example.org/alice",
    ] {
        let source = resolved(
            r#"{"subject":"acct:alice@example.org","aliases":["acct:alice@example.org"]}"#,
        );
        assert!(matches!(
            WebFingerIntake::from_import(query, source),
            Err(WebFingerIntakeError::InvalidAccount(_))
        ));
    }
    for subject in ["", "did:web:example.org", "acct:alice@host?query"] {
        let source = WebFingerImport {
            subject: subject.into(),
            aliases: vec!["acct:alice@example.org".into()],
            ..WebFingerImport::default()
        };
        assert!(WebFingerIntake::from_import("alice@example.org", source).is_err());
    }
}

#[test]
fn returned_subject_and_account_aliases_require_explicit_uris() {
    let source = WebFingerImport {
        subject: "alice@example.org".into(),
        ..WebFingerImport::default()
    };
    assert_eq!(
        WebFingerIntake::from_import("alice@example.org", source).unwrap_err(),
        WebFingerIntakeError::SubjectNotAccount
    );
    let source = resolved(
        r#"{"subject":"acct:alice@example.org","aliases":["bob@example.org","acct:bob@example.org"]}"#,
    );
    let intake = WebFingerIntake::from_import("alice@example.org", source).unwrap();
    assert_eq!(intake.addresses().handles.len(), 1);
    assert_eq!(intake.addresses().handles[0].value, "acct:bob@example.org");
    assert_eq!(intake.source().aliases.len(), 2);
}

#[test]
fn protocol_groups_and_catch_all_metadata_are_projected_without_keys() {
    let source = resolved(
        r#"{
        "subject":"acct:alice@example.org",
        "aliases":["did:key:z6Mkunverifiedclaim","did:plc:unverifiedclaim","https://example.org/alice"],
        "links":[
            {"rel":"self","href":"https://example.org/alice"},
            {"rel":"alternate","href":"gemini://example.org/alice"},
            {"rel":"alternate","href":"gopher://example.org/1/alice"},
            {"rel":"alternate","href":"misfin://alice@example.org"},
            {"rel":"self","type":"application/activity+json","href":"https://example.org/actor"},
            {"rel":"urn:example:directory","type":"application/example","href":"sftp://example.org/alice"}
        ]
    }"#,
    );
    let intake = WebFingerIntake::from_import("alice@example.org", source.clone()).unwrap();
    let kinds: Vec<_> = intake
        .addresses()
        .endpoints
        .iter()
        .map(|endpoint| endpoint.kind.clone())
        .collect();
    assert_eq!(
        kinds,
        vec![
            EndpointKind::Http,
            EndpointKind::Gemini,
            EndpointKind::Gopher,
            EndpointKind::Misfin,
            EndpointKind::ActivityPub,
            EndpointKind::Other("urn:example:directory".into())
        ]
    );
    assert_eq!(intake.source().other_endpoints, source.other_endpoints);

    let mut book = ContactBook::new(PersonaScope::new("work"));
    let outcome = book
        .intake_addresses(intake.addresses(), Some(local_contact()))
        .unwrap();
    let contact = book.get(&outcome.anchor).unwrap();
    assert!(matches!(contact.anchor(), Anchor::Local(_)));
    assert!(contact.root_line().is_empty());
    assert!(contact.attested().is_empty());
    assert!(
        contact
            .handles
            .iter()
            .all(|handle| handle.binding == TrustState::Unverified)
    );
    assert!(contact.endpoints.iter().all(
        |endpoint| endpoint.trust == TrustState::Unverified && endpoint.last_used_ms.is_none()
    ));
}

#[test]
fn repeated_resolution_deduplicates_claims_and_preserves_existing_trust() {
    let source = resolved(
        r#"{
        "subject":"acct:alice@example.org",
        "aliases":["acct:alice@other.example","acct:alice@other.example"],
        "links":[{"rel":"self","href":"https://example.org/alice"},{"rel":"self","href":"https://example.org/alice"}]
    }"#,
    );
    let intake = WebFingerIntake::from_import("alice@example.org", source).unwrap();
    let mut book = ContactBook::new(PersonaScope::new("work"));
    let initial = book
        .intake_addresses(intake.addresses(), Some(local_contact()))
        .unwrap();
    let contact = book.get_mut(&initial.anchor).unwrap();
    contact.handles[0].binding = TrustState::Pinned { first_seen_ms: 10 };
    contact.endpoints[0].trust = TrustState::Revoked { at_ms: 20 };
    contact.endpoints[0].mark_used(30);
    let retained = contact.clone();
    let repeated = book.intake_addresses(intake.addresses(), None).unwrap();
    assert!(!repeated.created);
    assert_eq!(repeated.handles_added, 0);
    assert_eq!(repeated.endpoints_added, 0);
    assert_eq!(book.len(), 1);
    assert_eq!(book.get(&initial.anchor).unwrap(), &retained);
}

#[test]
fn malformed_or_unmapped_aliases_and_links_stay_in_the_source() {
    let source = WebFingerImport {
        subject: "acct:alice@example.org".into(),
        aliases: vec![
            "acct:@example.org".into(),
            "did::empty".into(),
            "did:Web:example.org".into(),
            "did:web:".into(),
            "did:web:example.org#key".into(),
            "did:web:bad%xx".into(),
            "nostr:opaque".into(),
        ],
        profile_pages: vec![
            "https://example.org/<bad>".into(),
            "https://example.org/%zz".into(),
            "https://example.org/a b".into(),
            "https:///".into(),
            "mailto:alice@example.org".into(),
        ],
        other_endpoints: vec![WebFingerEndpoint {
            rel: "self".into(),
            media_type: Some("application/example".into()),
            href: "/relative".into(),
        }],
        ..WebFingerImport::default()
    };
    let intake = WebFingerIntake::from_import("alice@example.org", source.clone()).unwrap();
    assert!(intake.addresses().handles.is_empty());
    assert!(intake.addresses().endpoints.is_empty());
    assert_eq!(intake.source(), &source);
}

#[test]
fn valid_did_aliases_are_opaque_names_without_method_authentication() {
    for alias in [
        "did:web:example.org",
        "did:example:a-b_c.d",
        "did:example::a%2Fz",
        "did:key:opaque",
        "did:plc:opaque",
    ] {
        let source = WebFingerImport {
            subject: "acct:alice@example.org".into(),
            aliases: vec![alias.into()],
            ..WebFingerImport::default()
        };
        let intake = WebFingerIntake::from_import("alice@example.org", source).unwrap();
        assert_eq!(intake.addresses().handles.len(), 1);
        assert_eq!(intake.addresses().handles[0].kind, HandleKind::Did);
        assert_eq!(intake.addresses().handles[0].value, alias);
    }
}

#[test]
fn aliases_cannot_select_an_existing_other_contact() {
    let mut book = ContactBook::new(PersonaScope::new("work"));
    let first = WebFingerIntake::from_import(
        "bob@example.org",
        resolved(r#"{"subject":"acct:bob@example.org"}"#),
    )
    .unwrap();
    book.intake_addresses(first.addresses(), Some(local_contact()))
        .unwrap();
    let original = book.clone();
    let malicious = WebFingerIntake::from_import(
        "alice@example.org",
        resolved(r#"{"subject":"acct:alice@example.org","aliases":["acct:bob@example.org"]}"#),
    )
    .unwrap();
    assert_eq!(
        book.intake_addresses(malicious.addresses(), None)
            .unwrap_err(),
        IntakeError::LocalContactRequired
    );
    let fallback = NewLocalContact {
        id: LocalId::from_random([8; 16]),
        petname: "New Alice".into(),
    };
    assert!(matches!(
        book.intake_addresses(malicious.addresses(), Some(fallback)),
        Err(IntakeError::HandleConflict { .. })
    ));
    assert_eq!(book, original);
    assert_eq!(book.len(), 1);
}
