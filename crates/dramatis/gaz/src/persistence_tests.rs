// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use muniment::{JsonCodec, MemoryBackend, PostcardCodec};
use personae::{IdentityProvider, InMemoryProvider};
use serde::Serialize;

use super::*;
use crate::{
    Anchor, AttestedKey, Contact, ContactTier, Endpoint, Handle, KeyProof, RootKey, TypedKey,
};

fn book(scope: &str, name: &str, seed: u8) -> ContactBook {
    let issuer = InMemoryProvider::from_seed([seed; 32]);
    let salt = b"protocol/device\0\xff";
    let attestation = issuer.attest_derived_key(salt).unwrap();
    let root = attestation.master_key();
    let mut contact = Contact::new(name, root)
        .with_handle(Handle::acct(format!("{name}@example.org")))
        .with_note("private context");
    contact
        .attest(
            attestation.derived_key(),
            "display scope",
            root,
            Some(KeyProof::Attestation {
                attestation,
                salt: salt.to_vec(),
            }),
        )
        .unwrap();
    let mut book = ContactBook::new(PersonaScope::new(scope));
    book.insert(contact);
    book
}

fn recheck(book: &ContactBook) {
    let contact = book.iter().next().unwrap();
    let device = &contact.attested()[0];
    let Some(KeyProof::Attestation { attestation, salt }) = &device.proof else {
        panic!("lost the retained artifact");
    };
    let checked = attestation.check(salt).unwrap();
    assert_eq!(checked.derived_key(), device.key);
    assert_eq!(checked.master_key(), device.root);
}

// Deliberately unchecked storage data. The layout is Contact's serde format;
// unlike Contact's public API, this lets the tests put broken records on disk.
#[derive(Serialize)]
struct StoredContact<'a> {
    petname: &'a str,
    anchor: &'a Anchor,
    root_line: &'a [RootKey],
    attested: &'a [AttestedKey],
    handles: &'a [Handle],
    endpoints: &'a [Endpoint],
    tier: ContactTier,
    last_contact_ms: Option<u64>,
    note: &'a Option<String>,
}

impl<'a> From<&'a Contact> for StoredContact<'a> {
    fn from(contact: &'a Contact) -> Self {
        Self {
            petname: &contact.petname,
            anchor: contact.anchor(),
            root_line: contact.root_line(),
            attested: contact.attested(),
            handles: &contact.handles,
            endpoints: &contact.endpoints,
            tier: contact.tier,
            last_contact_ms: contact.last_contact_ms,
            note: &contact.note,
        }
    }
}

#[derive(Serialize)]
struct StoredBook<'a> {
    scope: &'a PersonaScope,
    contacts: Vec<StoredContact<'a>>,
}

fn exercise<B: Backend, C: Codec>(mut reopen: impl FnMut() -> B) {
    pollster::block_on(async {
        let work = PersonaScope::new("work");
        let burner = PersonaScope::new("burner");
        let mut alice = book("work", "Alice", 1);
        let bob = book("burner", "Bob", 2);
        let slots = SlotStore::<B, C>::new(reopen());
        assert_eq!(
            load_book(&slots, &work).await.unwrap(),
            ContactBook::new(work.clone())
        );

        save_book(&slots, &alice).await.unwrap();
        save_book(&slots, &bob).await.unwrap();
        let mut keys = slots.backend().list("personas/").await.unwrap();
        keys.sort();
        assert_eq!(keys, ["personas/burner/contacts", "personas/work/contacts"]);
        // Drop the database handle before reopening. For redb this is an
        // actual disk reopen, rather than a second view of the same handle.
        drop(slots);
        let slots = SlotStore::<B, C>::new(reopen());
        let loaded = load_book(&slots, &work).await.unwrap();
        assert_eq!(loaded, alice);
        recheck(&loaded);
        assert_eq!(load_book(&slots, &burner).await.unwrap(), bob);
        recheck(&load_book(&slots, &burner).await.unwrap());

        // Overwriting one book leaves the other persona's book intact.
        let anchor = alice.iter().next().unwrap().anchor().clone();
        alice.get_mut(&anchor).unwrap().note = Some("updated private context".into());
        save_book(&slots, &alice).await.unwrap();
        assert_eq!(load_book(&slots, &work).await.unwrap(), alice);
        assert_eq!(load_book(&slots, &burner).await.unwrap(), bob);

        // A complete, valid book under the wrong slot is still refused.
        slots.save("personas/work/contacts", &bob).await.unwrap();
        let error = load_book(&slots, &work).await.unwrap_err();
        assert_eq!(
            error,
            PersistenceError::Scope(ScopeMismatch {
                expected: work.clone(),
                found: burner.clone()
            })
        );
        assert!(error.to_string().contains("burner"));
        assert!(error.to_string().contains("work"));

        let contact = alice.iter().next().unwrap();
        // Prove the unchecked test shape has the real book's wire layout
        // before using it to introduce an invariant violation.
        assert_eq!(
            C::encode(&StoredBook {
                scope: &work,
                contacts: vec![StoredContact::from(contact)],
            })
            .unwrap(),
            C::encode(&alice).unwrap(),
        );
        let mut rootless = StoredContact::from(contact);
        rootless.root_line = &[];
        rootless.attested = &[];
        let mut wrong_key = contact.attested().to_vec();
        wrong_key[0].key = TypedKey::ed25519([99; 32]);
        let mut mismatched = StoredContact::from(contact);
        mismatched.attested = &wrong_key;
        let broken = [
            (
                StoredBook {
                    scope: &work,
                    contacts: vec![StoredContact::from(contact), rootless],
                },
                "key-rooted",
            ),
            (
                StoredBook {
                    scope: &work,
                    contacts: vec![mismatched],
                },
                "proof names a different key or root",
            ),
            (
                StoredBook {
                    scope: &work,
                    contacts: vec![StoredContact::from(contact), StoredContact::from(contact)],
                },
                "two contacts share the anchor",
            ),
        ];
        for (record, reason) in broken {
            let bytes = C::encode(&record).unwrap();
            slots
                .backend()
                .put("personas/work/contacts", &bytes)
                .await
                .unwrap();
            let error = load_book(&slots, &work).await.unwrap_err();
            assert!(
                matches!(&error, PersistenceError::Store(StoreError::Codec(_))),
                "{reason}: {error}"
            );
            // A refusal preserves the stored evidence and the other persona.
            assert_eq!(
                slots.backend().get("personas/work/contacts").await.unwrap(),
                Some(bytes)
            );
            assert_eq!(load_book(&slots, &burner).await.unwrap(), bob);
        }

        slots
            .backend()
            .put("personas/work/contacts", &[0xff])
            .await
            .unwrap();
        assert!(matches!(
            load_book(&slots, &work).await,
            Err(PersistenceError::Store(StoreError::Codec(_)))
        ));
        assert_eq!(load_book(&slots, &burner).await.unwrap(), bob);

        save_book(&slots, &ContactBook::new(work.clone()))
            .await
            .unwrap();
        assert!(load_book(&slots, &work).await.unwrap().is_empty());
        assert_eq!(load_book(&slots, &burner).await.unwrap(), bob);
        slots.delete("personas/work/contacts").await.unwrap();
        assert_eq!(
            load_book(&slots, &work).await.unwrap(),
            ContactBook::new(work)
        );
    });
}

#[test]
fn memory_json_persona_isolation_and_refusals() {
    let backend = MemoryBackend::new();
    exercise::<_, JsonCodec>(|| backend.clone());
}

#[test]
fn memory_postcard_persona_isolation_and_refusals() {
    let backend = MemoryBackend::new();
    exercise::<_, PostcardCodec>(|| backend.clone());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn redb_json_reopen_persona_isolation_and_refusals() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("contacts.redb");
    exercise::<_, JsonCodec>(|| muniment::RedbBackend::open(&path).unwrap());
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn redb_postcard_reopen_persona_isolation_and_refusals() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("contacts.redb");
    exercise::<_, PostcardCodec>(|| muniment::RedbBackend::open(&path).unwrap());
}

#[test]
fn scope_labels_are_opaque_and_not_normalized() {
    pollster::block_on(async {
        let slots = SlotStore::<_, JsonCodec>::new(MemoryBackend::new());
        let book = book("team / café", "Alice", 1);
        save_book(&slots, &book).await.unwrap();
        assert!(
            slots
                .backend()
                .get("personas/team / café/contacts")
                .await
                .unwrap()
                .is_some()
        );
        assert_eq!(load_book(&slots, book.scope()).await.unwrap(), book);
        assert!(
            load_book(&slots, &PersonaScope::new("team/café"))
                .await
                .unwrap()
                .is_empty()
        );
    });
}
