// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::endpoint::EndpointKind;
use crate::trust::TrustState;

fn key(seed: u8) -> TypedKey {
    TypedKey::ed25519([seed; 32])
}

fn alice() -> Contact {
    Contact::new("Alice", key(1))
}

fn bsky() -> PlcDid {
    PlcDid::parse("did:plc:z72i7hdynmk6r22z27h6tvur").unwrap()
}

#[test]
fn a_new_contact_is_kith_and_anchored_on_its_first_root() {
    let contact = alice();
    assert_eq!(contact.tier, ContactTier::Kith);
    assert_eq!(contact.anchor(), &Anchor::Key(key(1)));
    assert_eq!(contact.root(), Some(&key(1)));
    assert_eq!(contact.root_line()[0].proof, None);
    assert_eq!(contact.last_contact_ms, None);
}

#[test]
fn rotation_keeps_the_anchor_and_records_its_proof() {
    let mut contact = alice();
    assert!(contact.rotate_to(key(2), Some(ProofMethod::Signature)));

    assert_eq!(
        contact.anchor(),
        &Anchor::Key(key(1)),
        "the anchor must never move"
    );
    assert_eq!(contact.root(), Some(&key(2)));
    assert_eq!(contact.root_line()[1].proof, Some(ProofMethod::Signature));
    assert!(contact.knows_key(&key(1)), "a retired key still resolves");
}

#[test]
fn rotating_to_a_known_key_is_a_no_op() {
    let mut contact = alice();
    contact.rotate_to(key(2), None);
    assert!(!contact.rotate_to(key(2), None));
    assert!(!contact.rotate_to(key(1), None));
    assert_eq!(contact.root_line().len(), 2);
}

#[test]
fn a_plc_account_keeps_its_did_while_its_signing_key_moves() {
    let mut contact = Contact::new_plc("Bluesky", bsky(), key(1));
    contact.rotate_to(key(2), Some(ProofMethod::DidAuth));

    assert_eq!(contact.anchor(), &Anchor::Plc(bsky()));
    assert_eq!(contact.root(), Some(&key(2)));
    assert_eq!(contact.root_line()[1].proof, Some(ProofMethod::DidAuth));
}

#[test]
fn a_local_contact_starts_keyless_and_pins_its_first_key() {
    let id = LocalId::from_random([5; 16]);
    let mut contact = Contact::new_local("Mum", id);
    assert_eq!(contact.root(), None);

    assert!(contact.rotate_to(key(3), Some(ProofMethod::OutOfBand)));
    assert_eq!(
        contact.anchor(),
        &Anchor::Local(id),
        "pinning a key does not move the anchor"
    );
    assert_eq!(contact.root(), Some(&key(3)));
}

#[test]
fn attested_keys_are_concurrent_under_a_root() {
    let mut contact = alice();
    assert_eq!(
        contact.attest(key(10), "mesh-author", key(1), Some(ProofMethod::Signature)),
        Ok(true)
    );
    assert_eq!(
        contact.attest(
            key(11),
            "station/north",
            key(1),
            Some(ProofMethod::Signature)
        ),
        Ok(true)
    );

    assert_eq!(
        contact.root(),
        Some(&key(1)),
        "attesting never moves the root"
    );
    assert_eq!(
        contact.keys_for("mesh-author").collect::<Vec<_>>(),
        vec![&key(10)]
    );
    assert!(contact.knows_key(&key(11)));
    assert_eq!(
        contact.attest(key(10), "mesh-author", key(1), None),
        Ok(false)
    );
}

#[test]
fn an_attestation_needs_a_known_root_and_a_non_root_key() {
    let mut contact = alice();
    assert_eq!(
        contact.attest(key(10), "mesh-author", key(9), None),
        Err(AttestError::UnknownRoot)
    );
    assert_eq!(
        contact.attest(key(1), "mesh-author", key(1), None),
        Err(AttestError::IsRoot)
    );
}

#[test]
fn an_attested_key_cannot_become_a_root() {
    let mut contact = alice();
    contact
        .attest(key(10), "mesh-author", key(1), None)
        .unwrap();
    assert!(!contact.rotate_to(key(10), None));
    assert_eq!(contact.root(), Some(&key(1)));
}

#[test]
fn reachable_skips_revoked_addresses() {
    let contact = alice()
        .with_endpoint(Endpoint::new(EndpointKind::Misfin, "a@x.org"))
        .with_endpoint(
            Endpoint::new(EndpointKind::Gemini, "gemini://x.org/~a")
                .with_trust(TrustState::Revoked { at_ms: 1 }),
        );

    let reachable: Vec<_> = contact.reachable().map(|e| e.address.as_str()).collect();
    assert_eq!(reachable, vec!["a@x.org"]);
}

#[test]
fn a_mismatched_endpoint_raises_an_alarm() {
    assert!(!alice().has_alarm());

    let contact = alice().with_endpoint(
        Endpoint::new(EndpointKind::Misfin, "a@x.org")
            .with_trust(TrustState::Mismatched { noticed_ms: 7 }),
    );
    assert!(contact.has_alarm());
}

#[test]
fn a_mismatched_handle_binding_also_raises_an_alarm() {
    let contact =
        alice().with_handle(Handle::acct("a@x.org").with_binding(TrustState::Revoked { at_ms: 2 }));
    assert!(contact.has_alarm());
}

#[test]
fn contact_marking_is_monotonic() {
    let mut contact = alice();
    contact.mark_contacted(500);
    contact.mark_contacted(200);
    assert_eq!(contact.last_contact_ms, Some(500));
}

#[test]
fn handles_are_found_through_normalization() {
    let contact = alice().with_handle(Handle::acct("acct:Alice@Example.org"));
    assert!(contact.find_handle("alice@example.org").is_some());
    assert!(contact.find_handle("bob@example.org").is_none());
}

fn stored(anchor: &str, root_line: &str, attested: &str) -> String {
    format!(
        r#"{{"petname":"P","anchor":"{anchor}","root_line":{root_line},"attested":{attested},
           "handles":[],"endpoints":[],"tier":"Kith","last_contact_ms":null,"note":null}}"#
    )
}

fn root_json(key: &TypedKey) -> String {
    format!(r#"{{"key":"{key}","proof":null}}"#)
}

#[test]
fn a_record_that_lost_its_root_fails_to_load() {
    let error =
        serde_json::from_str::<Contact>(&stored(&key(1).to_string(), "[]", "[]")).unwrap_err();
    assert!(
        error.to_string().contains("key-rooted"),
        "the failure must name the invariant, got: {error}"
    );
}

#[test]
fn a_key_anchor_must_head_its_root_line() {
    let json = stored(
        &key(1).to_string(),
        &format!("[{}]", root_json(&key(2))),
        "[]",
    );
    let error = serde_json::from_str::<Contact>(&json).unwrap_err();
    assert!(
        error.to_string().contains("start with its anchor"),
        "got: {error}"
    );
}

#[test]
fn an_attestation_by_a_stranger_fails_to_load() {
    let attested = format!(
        r#"[{{"key":"{}","scope":"s","root":"{}","proof":null}}]"#,
        key(10),
        key(9)
    );
    let json = stored(
        &key(1).to_string(),
        &format!("[{}]", root_json(&key(1))),
        &attested,
    );
    let error = serde_json::from_str::<Contact>(&json).unwrap_err();
    assert!(error.to_string().contains("does not hold"), "got: {error}");
}

#[test]
fn a_duplicated_key_fails_to_load() {
    let line = format!("[{},{}]", root_json(&key(1)), root_json(&key(1)));
    let error =
        serde_json::from_str::<Contact>(&stored(&key(1).to_string(), &line, "[]")).unwrap_err();
    assert!(error.to_string().contains("twice"), "got: {error}");
}

#[test]
fn a_keyless_local_record_loads() {
    let id = LocalId::from_random([1; 16]);
    let contact = serde_json::from_str::<Contact>(&stored(&id.to_urn(), "[]", "[]")).unwrap();
    assert_eq!(contact.anchor(), &Anchor::Local(id));
}

fn full_records() -> Vec<Contact> {
    let mut alice = alice()
        .with_tier(ContactTier::Kin)
        .with_handle(Handle::acct("alice@example.org"))
        .with_endpoint(Endpoint::new(EndpointKind::Murm, "ff00"))
        .with_note("met at the moot");
    alice.rotate_to(key(2), Some(ProofMethod::Signature));
    alice
        .attest(key(10), "mesh-author", key(2), Some(ProofMethod::Signature))
        .unwrap();
    alice.mark_contacted(1234);

    let mut bluesky = Contact::new_plc("Bluesky", bsky(), key(20));
    bluesky.rotate_to(key(21), Some(ProofMethod::DidAuth));

    let mut mum = Contact::new_local("Mum", LocalId::from_random([7; 16]));
    mum.rotate_to(key(30), None);

    vec![alice, bluesky, mum]
}

#[test]
fn serde_round_trips_full_records_in_json_and_binary() {
    for contact in full_records() {
        let json = serde_json::to_string(&contact).unwrap();
        assert_eq!(serde_json::from_str::<Contact>(&json).unwrap(), contact);

        let bytes = postcard::to_allocvec(&contact).unwrap();
        assert_eq!(postcard::from_bytes::<Contact>(&bytes).unwrap(), contact);
    }
}
