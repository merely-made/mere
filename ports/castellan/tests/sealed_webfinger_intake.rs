// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! M2 address intake through Gazette and the host's sealed, persona-scoped store.
#![cfg(all(feature = "keeper", not(target_arch = "wasm32")))]

use std::path::Path;

use castellan::custody::wallet::{ensure_wallet_state, stage_persona_private_epoch};
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use castellan::{authority::PersonaeHost, view::VaultProtectionView};
use gaz::{
    Anchor, Contact, ContactBook, ContactTier, Endpoint, EndpointKind, Handle, KeyProof, LocalId,
    PersistenceError, PersonaScope, ProofMethod, TrustState, intake::NewLocalContact, load_book,
    save_book,
};
use gazette::{WebFingerImport, intake::WebFingerIntake};
use muniment::{Codec, JsonCodec, PostcardCodec, RedbBackend, SlotStore, StoreError};
use pandect::{PersonaId, wallet_store::load_persona_wallet};
use personae::{Ed25519Keypair, IdentityProvider, InMemoryProvider, ProfileId};

const PRIVATE_NAME: &str = "Private intake petname: Zoë / sealed M2 receipt";
const PROFILE: &str = "https://example.org/alice";
type Host = PersonaeHost<InMemoryStorage>;

fn host(root: &Path) -> Host {
    let profile = Profile::new(
        ProfileId("webfinger-intake".into()),
        "Intake receipt",
        Ed25519Keypair::from_seed([42; 32]),
    );
    PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        Some(root.to_path_buf()),
        VaultProtectionView::Ephemeral,
    )
}

fn stage(root: &Path, persona: PersonaId) {
    ensure_wallet_state(root, persona, "Intake receipt host").unwrap();
    let epoch = load_persona_wallet(root, persona)
        .unwrap()
        .unwrap()
        .private_epoch_head;
    // Deliberately shared epoch material: isolation must bind the persona,
    // rather than rely on accidentally distinct fixture secrets.
    stage_persona_private_epoch(root, persona, epoch, b"webfinger-intake-epoch").unwrap();
}

fn fixture() -> WebFingerIntake {
    let document = gazette::parse_document(
        r#"{
            "subject": "acct:alice@example.org",
            "aliases": ["https://example.org/alice", "acct:alice@other.example"],
            "links": [
                {"rel":"self", "href":"https://example.org/alice"},
                {"rel":"alternate", "href":"gemini://example.org/alice"},
                {"rel":"alternate", "href":"gopher://example.org/1/alice"},
                {"rel":"alternate", "href":"misfin://alice@example.org"},
                {"rel":"self", "type":"application/activity+json",
                 "href":"https://example.org/users/alice"},
                {"rel":"alternate", "href":"sftp://example.org/alice"}
            ]
        }"#,
    )
    .unwrap();
    WebFingerIntake::from_import(
        "alice@example.org",
        WebFingerImport::from_document(&document),
    )
    .unwrap()
}

fn trusted_contact(trust: TrustState) -> Contact {
    let issuer = InMemoryProvider::from_seed([17; 32]);
    let rotation_salt = b"gaz/webfinger-intake/root";
    let rotation = issuer.attest_derived_key(rotation_salt).unwrap();
    let root = rotation.master_key();
    let next = InMemoryProvider::from_seed(issuer.derive_keypair(rotation_salt).unwrap().to_seed());
    let device_salt = b"gaz/webfinger-intake/device";
    let device = next.attest_derived_key(device_salt).unwrap();
    let mut contact = Contact::new(PRIVATE_NAME, root)
        .with_note("Private relationship context stays local")
        .with_tier(ContactTier::Kin)
        .with_handle(Handle::acct("acct:alice@Example.org").with_binding(trust.clone()))
        .with_endpoint(Endpoint::new(EndpointKind::Http, PROFILE).with_trust(trust));
    contact
        .rotate_to(
            rotation.derived_key(),
            Some(KeyProof::Attestation {
                attestation: rotation,
                salt: rotation_salt.to_vec(),
            }),
        )
        .unwrap();
    contact
        .attest(
            device.derived_key(),
            "device",
            device.master_key(),
            Some(KeyProof::Attestation {
                attestation: device,
                salt: device_salt.to_vec(),
            }),
        )
        .unwrap();
    contact.endpoints[0].mark_used(123);
    contact.mark_contacted(456);
    contact
}

fn check_preserved(contact: &Contact, before: &Contact) {
    assert_eq!(contact.anchor(), before.anchor());
    assert_eq!(contact.petname, before.petname);
    assert_eq!(contact.note, before.note);
    assert_eq!(contact.tier, ContactTier::Kin);
    assert_eq!(contact.last_contact_ms, before.last_contact_ms);
    assert_eq!(contact.root_line(), before.root_line());
    assert_eq!(contact.attested(), before.attested());
    assert!(contact.handles.contains(&before.handles[0]));
    assert!(contact.endpoints.contains(&before.endpoints[0]));
    assert_eq!(contact.endpoints.len(), 6);
    for endpoint in &contact.endpoints {
        if endpoint.address != PROFILE {
            assert_eq!(endpoint.trust, TrustState::Unverified);
            assert_eq!(endpoint.last_used_ms, None);
        }
    }
    let alias = contact.find_handle("alice@other.example").unwrap();
    assert_eq!(alias.binding, TrustState::Unverified);
    for proof in [
        contact.root_line()[1].proof.as_ref().unwrap(),
        contact.attested()[0].proof.as_ref().unwrap(),
    ] {
        let KeyProof::Attestation { attestation, salt } = proof else {
            panic!("intake or storage lost the typed proof");
        };
        attestation.check(salt).unwrap();
    }
}

fn check_local(contact: &Contact, id: LocalId) {
    assert_eq!(contact.anchor(), &Anchor::Local(id));
    assert_eq!(contact.petname, "Caller-selected burner petname");
    assert_eq!(contact.tier, ContactTier::Kith);
    assert_eq!(contact.note, None);
    assert_eq!(contact.last_contact_ms, None);
    assert!(contact.root_line().is_empty());
    assert!(contact.attested().is_empty());
    assert_eq!(contact.endpoints.len(), 6);
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
            .all(|e| { e.trust == TrustState::Unverified && e.last_used_ms.is_none() })
    );
}

fn disk_receipt<C: Codec>(trust: TrustState) {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("wallet");
        let path = dir.path().join("intake.redb");
        let work = PersonaId::new();
        let burner = PersonaId::new();
        stage(&root, work);
        stage(&root, burner);
        let scope = PersonaScope::new(work.as_uuid().to_string());
        let burner_scope = PersonaScope::new(burner.as_uuid().to_string());
        let before = trusted_contact(trust);
        let anchor = before.anchor().clone();
        let mut book = ContactBook::new(scope.clone());
        book.insert(before.clone());
        let keeper = host(&root);
        let raw = RedbBackend::open(&path).unwrap();
        let slots = SlotStore::<_, C>::new(keeper.sealed_backend(work, &raw).unwrap());
        save_book(&slots, &book).await.unwrap();
        drop(slots);
        drop(raw);
        drop(keeper);

        // Begin intake from durable state, rather than merely testing an
        // in-memory merge followed by its first save.
        let keeper = host(&root);
        let raw = RedbBackend::open(&path).unwrap();
        let slots = SlotStore::<_, C>::new(keeper.sealed_backend(work, &raw).unwrap());
        let mut book = load_book(&slots, &scope).await.unwrap();
        let intake = fixture();
        book.intake_addresses(intake.addresses(), None).unwrap();
        assert_eq!(book.len(), 1);
        check_preserved(book.get(&anchor).unwrap(), &before);
        let expected = book.clone();
        book.intake_addresses(intake.addresses(), None).unwrap();
        assert_eq!(book, expected, "resolver replay must be an exact no-op");
        save_book(&slots, &book).await.unwrap();

        let burner_slots = SlotStore::<_, C>::new(keeper.sealed_backend(burner, &raw).unwrap());
        let mut local_book = load_book(&burner_slots, &burner_scope).await.unwrap();
        assert!(local_book.is_empty());
        assert!(
            local_book
                .intake_addresses(intake.addresses(), None)
                .is_err()
        );
        assert!(
            local_book.is_empty(),
            "a resolver cannot mint the host's local id"
        );
        let id = LocalId::from_random([31; 16]);
        local_book
            .intake_addresses(
                intake.addresses(),
                Some(NewLocalContact {
                    id,
                    petname: "Caller-selected burner petname".into(),
                }),
            )
            .unwrap();
        assert_eq!(local_book.len(), 1);
        check_local(local_book.get(&Anchor::Local(id)).unwrap(), id);
        save_book(&burner_slots, &local_book).await.unwrap();
        drop(burner_slots);
        drop(slots);
        drop(raw);
        drop(keeper);

        // Check every redb page, including historical pages. An encoded
        // positive control proves either codec would expose the petname if
        // the host's sealing adapter were accidentally bypassed.
        let encoded = C::encode(&expected).unwrap();
        assert!(
            encoded
                .windows(PRIVATE_NAME.len())
                .any(|w| w == PRIVATE_NAME.as_bytes())
        );
        let disk = std::fs::read(&path).unwrap();
        assert!(
            !disk
                .windows(PRIVATE_NAME.len())
                .any(|w| w == PRIVATE_NAME.as_bytes())
        );

        let keeper = host(&root);
        let raw = RedbBackend::open(&path).unwrap();
        let slots = SlotStore::<_, C>::new(keeper.sealed_backend(work, &raw).unwrap());
        let burner_slots = SlotStore::<_, C>::new(keeper.sealed_backend(burner, &raw).unwrap());
        let mut book = load_book(&slots, &scope).await.unwrap();
        assert_eq!(book, expected);
        check_preserved(book.get(&anchor).unwrap(), &before);
        book.intake_addresses(intake.addresses(), None).unwrap();
        assert_eq!(
            book, expected,
            "replay after reopening must not duplicate addresses"
        );
        let mut reloaded_local = load_book(&burner_slots, &burner_scope).await.unwrap();
        assert_eq!(reloaded_local, local_book);
        reloaded_local
            .intake_addresses(intake.addresses(), None)
            .unwrap();
        assert_eq!(reloaded_local, local_book);
        check_local(reloaded_local.get(&Anchor::Local(id)).unwrap(), id);
        assert!(matches!(
            load_book(&burner_slots, &scope).await,
            Err(PersistenceError::Store(StoreError::Backend(_)))
        ));
        assert!(matches!(
            load_book(&slots, &burner_scope).await,
            Err(PersistenceError::Store(StoreError::Backend(_)))
        ));
        assert_eq!(load_book(&slots, &scope).await.unwrap(), expected);
        assert_eq!(
            load_book(&burner_slots, &burner_scope).await.unwrap(),
            local_book
        );
    });
}

#[test]
fn json_webfinger_intake_preserves_pins_and_private_state_across_sealed_reload() {
    disk_receipt::<JsonCodec>(TrustState::Pinned { first_seen_ms: 11 });
}

#[test]
fn postcard_webfinger_intake_preserves_pins_and_private_state_across_sealed_reload() {
    disk_receipt::<PostcardCodec>(TrustState::Pinned { first_seen_ms: 11 });
}

#[test]
fn json_webfinger_intake_preserves_verified_state_across_sealed_reload() {
    disk_receipt::<JsonCodec>(TrustState::Verified {
        method: ProofMethod::BackClaim,
        at_ms: 22,
    });
}

#[test]
fn postcard_webfinger_intake_preserves_verified_state_across_sealed_reload() {
    disk_receipt::<PostcardCodec>(TrustState::Verified {
        method: ProofMethod::BackClaim,
        at_ms: 22,
    });
}
