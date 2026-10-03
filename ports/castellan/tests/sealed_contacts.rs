// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Gaz M1 host receipt: actual wallet epochs, Castellan, and disk reopening.
#![cfg(all(feature = "keeper", not(target_arch = "wasm32")))]

use std::path::Path;

use castellan::{authority::PersonaeHost, view::VaultProtectionView};
use gaz::{Contact, ContactBook, KeyProof, PersistenceError, PersonaScope, load_book, save_book};
use muniment::{
    Backend, Codec, JsonCodec, MemoryBackend, PostcardCodec, RedbBackend, SlotStore, StoreError,
};
use pandect::{
    PersonaId,
    wallet_store::{ensure_wallet_state, load_persona_wallet, stage_persona_private_epoch},
};
use personae::{
    Ed25519Keypair, IdentityProvider, IdentityVault, InMemoryProvider, InMemoryStorage, Profile,
    ProfileId,
};

type Host = PersonaeHost<InMemoryStorage>;

fn host(root: Option<&Path>) -> Host {
    let profile = Profile::new(
        ProfileId("receipt".into()),
        "Receipt",
        Ed25519Keypair::from_seed([42; 32]),
    );
    PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        root.map(Path::to_path_buf),
        VaultProtectionView::Ephemeral,
    )
}

fn stage(root: &Path, persona: PersonaId) {
    ensure_wallet_state(root, persona, "Receipt host").unwrap();
    let epoch = load_persona_wallet(root, persona)
        .unwrap()
        .unwrap()
        .private_epoch_head;
    // Same fixture secret for both personas: refusal must depend on persona
    // binding as well as possession of the epoch secret.
    stage_persona_private_epoch(root, persona, epoch, b"receipt-private-epoch").unwrap();
}

fn book(scope: &PersonaScope, petname: &str) -> ContactBook {
    let issuer = InMemoryProvider::from_seed([17; 32]);
    let salt = b"gaz/host-receipt";
    let attestation = issuer.attest_derived_key(salt).unwrap();
    let root = attestation.master_key();
    let mut contact = Contact::new(petname, root).with_note("private relationship context");
    contact
        .attest(
            attestation.derived_key(),
            "device",
            root,
            Some(KeyProof::Attestation {
                attestation,
                salt: salt.to_vec(),
            }),
        )
        .unwrap();
    let mut book = ContactBook::new(scope.clone());
    book.insert(contact);
    book
}

fn refuses_book(result: Result<ContactBook, PersistenceError>) {
    assert!(
        matches!(result, Err(PersistenceError::Store(StoreError::Backend(_)))),
        "sealed storage failure must not turn into an empty book: {result:?}"
    );
}

fn disk_receipt<C: Codec>() {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("wallet");
        let path = dir.path().join("contacts.redb");
        let work = PersonaId::new();
        let burner = PersonaId::new();
        stage(&root, work);
        stage(&root, burner);
        let scope = PersonaScope::new(work.as_uuid().to_string());
        let burner_scope = PersonaScope::new(burner.as_uuid().to_string());
        let key = format!("personas/{}/contacts", scope.as_str());
        let other_key = format!("personas/{}/contacts", burner_scope.as_str());
        let name = "Petname sealed on disk: Zoë / unique M1 receipt";
        let expected = book(&scope, name);
        let other = book(&burner_scope, "Other persona private petname");
        let keeper = host(Some(&root));
        let raw = RedbBackend::open(&path).unwrap();
        let slots = SlotStore::<_, C>::new(keeper.sealed_backend(work, &raw).unwrap());
        assert_eq!(
            load_book(&slots, &scope).await.unwrap(),
            ContactBook::new(scope.clone())
        );
        save_book(&slots, &expected).await.unwrap();
        let other_slots = SlotStore::<_, C>::new(keeper.sealed_backend(burner, &raw).unwrap());
        save_book(&other_slots, &other).await.unwrap();
        drop(other_slots);
        drop(slots);
        drop(raw);
        drop(keeper);

        // Examine the complete durable file, including redb's old pages, after
        // all handles close. A JSON *and* binary positive control prevents an
        // absence assertion from passing just because the petname was encoded.
        let encoded = C::encode(&expected).unwrap();
        assert!(encoded.windows(name.len()).any(|w| w == name.as_bytes()));
        let disk = std::fs::read(&path).unwrap();
        assert!(!disk.windows(name.len()).any(|w| w == name.as_bytes()));
        let keeper = host(Some(&root));
        let raw = RedbBackend::open(&path).unwrap();
        let original = raw.get(&key).await.unwrap().unwrap();
        let other_original = raw.get(&other_key).await.unwrap().unwrap();
        let slots = SlotStore::<_, C>::new(keeper.sealed_backend(work, &raw).unwrap());
        let loaded = load_book(&slots, &scope).await.unwrap();
        assert_eq!(loaded, expected);
        let contact = loaded.iter().next().unwrap();
        let held = &contact.attested()[0];
        let Some(KeyProof::Attestation { attestation, salt }) = &held.proof else {
            panic!("stored proof lost");
        };
        assert_eq!(attestation.check(salt).unwrap().derived_key(), held.key);
        let wrong = SlotStore::<_, C>::new(keeper.sealed_backend(burner, &raw).unwrap());
        refuses_book(load_book(&wrong, &scope).await);
        assert_eq!(load_book(&wrong, &burner_scope).await.unwrap(), other);

        let mut altered: serde_json::Value = serde_json::from_slice(&original).unwrap();
        let byte = altered["ciphertext"][0].as_u64().unwrap();
        altered["ciphertext"][0] = (byte ^ 1).into();
        let altered = serde_json::to_vec(&altered).unwrap();
        for invalid in [&altered, &encoded] {
            raw.put(&key, invalid).await.unwrap();
            refuses_book(load_book(&slots, &scope).await);
            assert_eq!(raw.get(&key).await.unwrap().unwrap(), *invalid);
            assert_eq!(raw.get(&other_key).await.unwrap().unwrap(), other_original);
        }
        raw.put(&key, &original).await.unwrap();
        let alias_scope = PersonaScope::new("copied-slot");
        raw.put("personas/copied-slot/contacts", &original)
            .await
            .unwrap();
        refuses_book(load_book(&slots, &alias_scope).await);
        assert_eq!(load_book(&slots, &scope).await.unwrap(), expected);
        save_book(&slots, &ContactBook::new(scope.clone()))
            .await
            .unwrap();
        assert_eq!(
            load_book(&slots, &scope).await.unwrap(),
            ContactBook::new(scope.clone())
        );
        assert_eq!(load_book(&wrong, &burner_scope).await.unwrap(), other);
    });
}

#[test]
fn json_contacts_are_sealed_by_the_host_and_reopen_from_disk() {
    disk_receipt::<JsonCodec>();
}

#[test]
fn postcard_contacts_are_sealed_by_the_host_and_reopen_from_disk() {
    disk_receipt::<PostcardCodec>();
}

#[test]
fn absent_host_wallet_or_epoch_refuses_without_writing() {
    let dir = tempfile::tempdir().unwrap();
    let raw = MemoryBackend::new();
    for keeper in [host(None), host(Some(dir.path()))] {
        let error = keeper
            .sealed_backend(PersonaId::new(), raw.clone())
            .err()
            .unwrap();
        assert_eq!(error.kind(), std::io::ErrorKind::PermissionDenied);
        assert!(raw.is_empty());
    }
}
