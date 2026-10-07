// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    Contact, ContactBook, ContactTier, Endpoint, PersonaScope, ProofMethod, TrustState, TypedKey,
};

fn handle(value: &str) -> HandleClaim {
    HandleClaim {
        kind: HandleKind::Acct,
        value: value.into(),
    }
}
fn endpoint(value: &str) -> EndpointClaim {
    EndpointClaim {
        kind: EndpointKind::Http,
        address: value.into(),
    }
}
fn claims() -> AddressIntake {
    AddressIntake {
        handle: handle("acct:Alice@example.org"),
        handles: vec![handle("acct:Alice@elsewhere.org")],
        endpoints: vec![
            endpoint("https://example.org/Alice"),
            endpoint("https://elsewhere.org/Alice"),
        ],
    }
}
fn seed(n: u8) -> NewLocalContact {
    NewLocalContact {
        id: LocalId::from_random([n; 16]),
        petname: "My petname".into(),
    }
}
fn book() -> ContactBook {
    ContactBook::new(PersonaScope::new("work"))
}

#[test]
fn account_comparison_preserves_user_case_and_normalizes_only_safe_uri_parts() {
    for (input, expected) in [
        (" ACCT:Alice@EXAMPLE.ORG ", "acct:Alice@example.org"),
        ("Alice@EXAMPLE.ORG", "acct:Alice@example.org"),
        ("acct:%41lice@example.org", "acct:Alice@example.org"),
        (
            "acct:A%6Cice@xn--exmple-cua.org",
            "acct:Alice@xn--exmple-cua.org",
        ),
        (
            "acct:juliet%40capulet.example@shop.example",
            "acct:juliet%40capulet.example@shop.example",
        ),
        ("acct:A%2fB@example.org", "acct:A%2FB@example.org"),
        ("acct:%c3%a9@example.org", "acct:%C3%A9@example.org"),
        ("acct:a@[2001:0DB8:0:0:0:0:0:1]", "acct:a@[2001:db8::1]"),
    ] {
        assert_eq!(normalize_acct_handle(input).unwrap(), expected);
    }
    assert_ne!(
        normalize_acct_handle("acct:Alice@example.org").unwrap(),
        normalize_acct_handle("acct:alice@example.org").unwrap()
    );
    assert_ne!(
        normalize_acct_handle("acct:Alice@example.org").unwrap(),
        normalize_acct_handle("acct:Alice@example.org.").unwrap()
    );
    for invalid in [
        "",
        "acct:@example.org",
        "acct:a@",
        "acct:a:b@example.org",
        "acct:a@@example.org",
        "acct:a@host:443",
        "acct:a@host/path",
        "acct:a@host?x",
        "acct:a@host#x",
        "acct:a@-host.org",
        "acct:a@host..org",
        "acct:a@é.org",
        "acct:a%00@example.org",
        "acct:a%20@example.org",
        "acct:a%7f@example.org",
        "acct:a%xx@example.org",
        "acct:a%@example.org",
        "acct:a@[not-ipv6]",
    ] {
        assert!(
            normalize_acct_handle(invalid).is_err(),
            "accepted {invalid}"
        );
    }
}

#[test]
fn missing_selector_creates_only_a_host_selected_keyless_local_contact() {
    let mut book = book();
    assert_eq!(
        book.intake_addresses(&claims(), None),
        Err(IntakeError::LocalContactRequired)
    );
    assert!(book.is_empty());
    let selected = seed(1);
    let anchor = Anchor::Local(selected.id);
    let result = book.intake_addresses(&claims(), Some(selected)).unwrap();
    assert_eq!(
        result,
        IntakeOutcome {
            anchor: anchor.clone(),
            created: true,
            handles_added: 2,
            endpoints_added: 2
        }
    );
    let contact = book.get(&anchor).unwrap();
    assert_eq!(contact.petname, "My petname");
    assert_eq!(contact.tier, ContactTier::Kith);
    assert_eq!(contact.note, None);
    assert_eq!(contact.last_contact_ms, None);
    assert!(contact.root_line().is_empty() && contact.attested().is_empty());
    assert!(
        contact
            .handles
            .iter()
            .all(|h| h.binding == TrustState::Unverified)
    );
    assert!(
        contact
            .endpoints
            .iter()
            .all(|e| e.trust == TrustState::Unverified && e.last_used_ms.is_none())
    );
    let before = book.clone();
    assert!(!book.intake_addresses(&claims(), None).unwrap().changed());
    assert_eq!(book, before);
}

#[test]
fn replay_preserves_every_trust_state_and_all_private_relationship_fields() {
    for state in [
        TrustState::Unverified,
        TrustState::Pinned { first_seen_ms: 10 },
        TrustState::Verified {
            method: ProofMethod::BackClaim,
            at_ms: 11,
        },
        TrustState::Mismatched { noticed_ms: 12 },
        TrustState::Revoked { at_ms: 13 },
    ] {
        let mut book = book();
        let mut contact = Contact::new("PRIVATE NAME", TypedKey::ed25519([1; 32]))
            .with_handle(Handle::acct("acct:Alice@EXAMPLE.org").with_binding(state.clone()))
            .with_endpoint(
                Endpoint::new(EndpointKind::Http, "https://example.org/Alice")
                    .with_trust(state.clone()),
            )
            .with_note("PRIVATE NOTE")
            .with_tier(ContactTier::Kin);
        contact.last_contact_ms = Some(u64::MAX);
        contact.endpoints[0].last_used_ms = Some(u64::MAX);
        let anchor = contact.anchor().clone();
        book.insert(contact.clone());
        let result = book.intake_addresses(&claims(), Some(seed(2))).unwrap();
        assert_eq!(result.anchor, anchor);
        assert!(!result.created);
        assert_eq!((result.handles_added, result.endpoints_added), (1, 1));
        let added = book.get(&anchor).unwrap();
        assert_eq!(added.handles[0], contact.handles[0]);
        assert_eq!(added.endpoints[0], contact.endpoints[0]);
        assert_eq!(added.petname, contact.petname);
        assert_eq!(added.note, contact.note);
        assert_eq!(added.tier, contact.tier);
        assert_eq!(added.last_contact_ms, contact.last_contact_ms);
        assert_eq!(added.root_line(), contact.root_line());
        assert_eq!(added.attested(), contact.attested());
        assert_eq!(added.handles[1].binding, TrustState::Unverified);
        assert_eq!(added.endpoints[1].trust, TrustState::Unverified);
        assert_eq!(added.has_alarm(), contact.has_alarm());
        let before = book.clone();
        let mut reordered = claims();
        reordered.handle.value = "ACCT:%41lice@EXAMPLE.ORG".into();
        reordered.handles.extend(reordered.handles.clone());
        reordered.endpoints.reverse();
        reordered.endpoints.extend(reordered.endpoints.clone());
        assert!(!book.intake_addresses(&reordered, None).unwrap().changed());
        assert_eq!(book, before);
        let mut loaded: ContactBook =
            serde_json::from_slice(&serde_json::to_vec(&book).unwrap()).unwrap();
        assert!(!loaded.intake_addresses(&reordered, None).unwrap().changed());
        assert_eq!(loaded, before);
    }
}

#[test]
fn ambiguous_primary_never_chooses_the_first_match() {
    let mut book = book();
    for n in [1, 2] {
        book.insert(
            Contact::new("Alice", TypedKey::ed25519([n; 32]))
                .with_handle(Handle::acct("acct:Alice@example.org")),
        );
    }
    let before = book.clone();
    let error = book.intake_addresses(&claims(), Some(seed(3))).unwrap_err();
    let IntakeError::Ambiguous(anchors) = error else {
        panic!("wrong refusal")
    };
    assert_eq!(anchors.len(), 2);
    assert_eq!(book, before);
}

#[test]
fn repeated_matching_handles_in_one_record_do_not_make_it_ambiguous() {
    let mut book = book();
    let contact = Contact::new("Alice", TypedKey::ed25519([1; 32]))
        .with_handle(Handle::acct("acct:Alice@example.org"))
        .with_handle(Handle::acct("Alice@EXAMPLE.ORG"));
    let anchor = contact.anchor().clone();
    book.insert(contact);
    assert_eq!(
        book.intake_addresses(&claims(), None).unwrap().anchor,
        anchor
    );
    assert_eq!(book.get(&anchor).unwrap().handles.len(), 3);
}

#[test]
fn occupied_fallback_never_overwrites_an_unrelated_local_record() {
    let mut book = book();
    let fallback = seed(1);
    let anchor = Anchor::Local(fallback.id);
    book.insert(Contact::new_local("Someone else", fallback.id));
    let before = book.clone();
    assert_eq!(
        book.intake_addresses(&claims(), Some(fallback)),
        Err(IntakeError::LocalAnchorOccupied(anchor))
    );
    assert_eq!(book, before);
}

#[test]
fn secondary_handles_never_select_or_merge_another_person() {
    let mut book = book();
    book.insert(
        Contact::new("Other", TypedKey::ed25519([1; 32]))
            .with_handle(Handle::acct("acct:Alice@elsewhere.org")),
    );
    let before = book.clone();
    assert!(matches!(
        book.intake_addresses(&claims(), Some(seed(1))),
        Err(IntakeError::HandleConflict { .. })
    ));
    assert_eq!(book, before);
    book.insert(
        Contact::new("Alice", TypedKey::ed25519([2; 32]))
            .with_handle(Handle::acct("acct:Alice@example.org")),
    );
    let before = book.clone();
    assert!(matches!(
        book.intake_addresses(&claims(), None),
        Err(IntakeError::HandleConflict { .. })
    ));
    assert_eq!(book, before);
}

#[test]
fn typed_opaque_handles_and_endpoints_preserve_case_and_distinct_roles() {
    let mut book = book();
    let mut intake = claims();
    intake.handles = vec![
        HandleClaim {
            kind: HandleKind::Did,
            value: "did:web:example.org:Alice".into(),
        },
        HandleClaim {
            kind: HandleKind::Did,
            value: "did:web:example.org:alice".into(),
        },
        HandleClaim {
            kind: HandleKind::Other("custom".into()),
            value: "did:web:example.org:Alice".into(),
        },
    ];
    intake.endpoints = vec![
        endpoint("https://example.org/Alice"),
        endpoint("https://example.org/alice"),
        EndpointClaim {
            kind: EndpointKind::ActivityPub,
            address: "https://example.org/Alice".into(),
        },
    ];
    let result = book.intake_addresses(&intake, Some(seed(1))).unwrap();
    assert_eq!((result.handles_added, result.endpoints_added), (4, 3));
    assert!(matches!(result.anchor, Anchor::Local(_)));
    assert!(book.get(&result.anchor).unwrap().root_line().is_empty());
    let before = book.clone();
    assert!(!book.intake_addresses(&intake, None).unwrap().changed());
    assert_eq!(book, before);
}

#[test]
fn account_case_never_selects_a_different_localpart_or_handle_family() {
    for (kind, value) in [
        (HandleKind::Acct, "acct:alice@example.org"),
        (HandleKind::Did, "acct:Alice@example.org"),
    ] {
        let mut book = book();
        book.insert(
            Contact::new("Other", TypedKey::ed25519([1; 32])).with_handle(Handle::new(kind, value)),
        );
        let result = book.intake_addresses(&claims(), Some(seed(1))).unwrap();
        assert!(result.created);
        assert_eq!(book.len(), 2);
    }
}

#[test]
fn invalid_claims_refuse_before_any_mutation() {
    let mut book = book();
    book.intake_addresses(&claims(), Some(seed(1))).unwrap();
    let before = book.clone();
    for which in 0..6 {
        let mut invalid = claims();
        match which {
            0 => invalid.handle.value = "acct:broken".into(),
            1 => invalid.handles[0].value = " ".into(),
            2 => invalid.endpoints[1].address = "\0".into(),
            3 => invalid.endpoints[1].kind = EndpointKind::Other(" ".into()),
            4 => invalid.handles[0].kind = HandleKind::Other(" ".into()),
            _ => invalid.handles[0].value = "acct:A%00lice@example.org".into(),
        }
        assert!(book.intake_addresses(&invalid, None).is_err());
        assert_eq!(book, before);
    }
    let mut empty = ContactBook::new(PersonaScope::new("empty"));
    let mut fallback = seed(2);
    fallback.petname.clear();
    assert!(empty.intake_addresses(&claims(), Some(fallback)).is_err());
    assert!(empty.is_empty());
}
