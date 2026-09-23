// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The persona-scoped collection of records.

use std::collections::BTreeMap;
use std::fmt;

use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize, Serializer};

use crate::anchor::Anchor;
use crate::contact::{Contact, ContactTier};
use crate::key::TypedKey;

/// Which persona a book belongs to.
///
/// An opaque label, not a parsed identity: mere passes the text of a
/// `personae::PersonaId`, and gaz never interprets it. That is what keeps this
/// crate free of a dependency on the identity stack it serves.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaScope(String);

impl PersonaScope {
    /// Label a book with the persona that owns it.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// The label as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PersonaScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A book was loaded for a persona it does not belong to.
///
/// Worth a distinct error rather than a silent accept: the whole point of
/// scoping contacts is that a burner persona cannot see the work persona's
/// people, and a mis-filed book would defeat it quietly.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopeMismatch {
    /// The persona the caller asked for.
    pub expected: PersonaScope,
    /// The persona the book actually carries.
    pub found: PersonaScope,
}

impl fmt::Display for ScopeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "contact book belongs to persona {}, not {}",
            self.found, self.expected
        )
    }
}

impl core::error::Error for ScopeMismatch {}

/// One persona's contacts.
///
/// Records are filed under their [`Anchor`], so a key rotation never moves a
/// record and an old signature still finds its owner. Stored as a list of
/// records rather than a map, so a filing key can never disagree with the
/// anchor inside its record.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(try_from = "BookRecord")]
pub struct ContactBook {
    scope: PersonaScope,
    contacts: BTreeMap<Anchor, Contact>,
}

/// A book as stored, before its anchors are checked for duplicates.
#[derive(Deserialize)]
struct BookRecord {
    scope: PersonaScope,
    contacts: Vec<Contact>,
}

/// Two stored records claimed the same anchor.
#[derive(Debug)]
struct DuplicateAnchor(Anchor);

impl fmt::Display for DuplicateAnchor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "two contacts share the anchor {}", self.0)
    }
}

impl TryFrom<BookRecord> for ContactBook {
    type Error = DuplicateAnchor;

    fn try_from(record: BookRecord) -> Result<Self, DuplicateAnchor> {
        let mut book = Self::new(record.scope);
        for contact in record.contacts {
            let anchor = contact.anchor().clone();
            if book.insert(contact).is_some() {
                return Err(DuplicateAnchor(anchor));
            }
        }
        Ok(book)
    }
}

impl Serialize for ContactBook {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ContactBook", 2)?;
        state.serialize_field("scope", &self.scope)?;
        state.serialize_field("contacts", &Records(&self.contacts))?;
        state.end()
    }
}

struct Records<'a>(&'a BTreeMap<Anchor, Contact>);

impl Serialize for Records<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(self.0.values())
    }
}

impl ContactBook {
    /// Open an empty book for a persona.
    pub fn new(scope: PersonaScope) -> Self {
        Self {
            scope,
            contacts: BTreeMap::new(),
        }
    }

    /// The persona this book belongs to.
    pub fn scope(&self) -> &PersonaScope {
        &self.scope
    }

    /// Check that a loaded book belongs where it was filed.
    ///
    /// Call this after loading, before showing anyone's contacts.
    pub fn verify_scope(&self, expected: &PersonaScope) -> Result<(), ScopeMismatch> {
        if &self.scope == expected {
            Ok(())
        } else {
            Err(ScopeMismatch {
                expected: expected.clone(),
                found: self.scope.clone(),
            })
        }
    }

    /// File a contact, replacing any record under the same anchor.
    pub fn insert(&mut self, contact: Contact) -> Option<Contact> {
        self.contacts.insert(contact.anchor().clone(), contact)
    }

    /// Look a contact up by anchor.
    pub fn get(&self, anchor: &Anchor) -> Option<&Contact> {
        self.contacts.get(anchor)
    }

    /// Borrow a contact mutably by anchor.
    pub fn get_mut(&mut self, anchor: &Anchor) -> Option<&mut Contact> {
        self.contacts.get_mut(anchor)
    }

    /// Every contact holding this key: root or attested, current or retired.
    ///
    /// Every one, not the first. One key can belong to more than one record:
    /// atproto servers once issued a single signing key to many accounts.
    pub fn by_key<'a>(&'a self, key: &'a TypedKey) -> impl Iterator<Item = &'a Contact> + 'a {
        self.contacts
            .values()
            .filter(move |contact| contact.knows_key(key))
    }

    /// Find a contact by a handle string a person typed.
    pub fn by_handle(&self, query: &str) -> Option<&Contact> {
        self.contacts
            .values()
            .find(|contact| contact.find_handle(query).is_some())
    }

    /// Find a contact by an exact endpoint address.
    pub fn by_endpoint(&self, address: &str) -> Option<&Contact> {
        self.contacts
            .values()
            .find(|contact| contact.find_endpoint(address).is_some())
    }

    /// Remove a contact by anchor.
    pub fn remove(&mut self, anchor: &Anchor) -> Option<Contact> {
        self.contacts.remove(anchor)
    }

    /// The people you have actually exchanged something with, most recent
    /// first.
    ///
    /// Contacts you have never reached are left out entirely rather than
    /// padding the tail: a recently-contacted list that includes people you
    /// have never contacted is not answering the question.
    pub fn recent(&self, limit: usize) -> Vec<&Contact> {
        let mut contacted: Vec<&Contact> = self
            .contacts
            .values()
            .filter(|contact| contact.last_contact_ms.is_some())
            .collect();

        contacted.sort_by(|a, b| {
            b.last_contact_ms
                .cmp(&a.last_contact_ms)
                .then_with(|| a.petname.cmp(&b.petname))
        });
        contacted.truncate(limit);
        contacted
    }

    /// Everyone at a given tier.
    pub fn tier(&self, tier: ContactTier) -> impl Iterator<Item = &Contact> {
        self.contacts
            .values()
            .filter(move |contact| contact.tier == tier)
    }

    /// Everyone whose record needs a person to look at it.
    pub fn alarms(&self) -> impl Iterator<Item = &Contact> {
        self.contacts.values().filter(|contact| contact.has_alarm())
    }

    /// Note contact with whoever is filed under this anchor.
    ///
    /// Returns whether anyone was found. To mark by a key a message arrived
    /// under, resolve it with [`ContactBook::by_key`] first.
    pub fn mark_contacted(&mut self, anchor: &Anchor, now_ms: u64) -> bool {
        let Some(contact) = self.contacts.get_mut(anchor) else {
            return false;
        };
        contact.mark_contacted(now_ms);
        true
    }

    /// Every contact, ordered by anchor.
    pub fn iter(&self) -> impl Iterator<Item = &Contact> {
        self.contacts.values()
    }

    /// How many contacts the book holds.
    pub fn len(&self) -> usize {
        self.contacts.len()
    }

    /// Whether the book is empty.
    pub fn is_empty(&self) -> bool {
        self.contacts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchor::{LocalId, PlcDid};
    use crate::endpoint::{Endpoint, EndpointKind};
    use crate::handle::Handle;
    use crate::trust::{ProofMethod, TrustState};

    fn key(seed: u8) -> TypedKey {
        TypedKey::ed25519([seed; 32])
    }

    fn anchor(seed: u8) -> Anchor {
        Anchor::Key(key(seed))
    }

    fn book() -> ContactBook {
        ContactBook::new(PersonaScope::new("work"))
    }

    #[test]
    fn a_fresh_book_is_empty_and_scoped() {
        let book = book();
        assert!(book.is_empty());
        assert_eq!(book.scope().as_str(), "work");
    }

    #[test]
    fn scope_verification_catches_a_misfiled_book() {
        let book = book();
        assert!(book.verify_scope(&PersonaScope::new("work")).is_ok());

        let error = book.verify_scope(&PersonaScope::new("burner")).unwrap_err();
        assert_eq!(error.found, PersonaScope::new("work"));
        assert_eq!(error.expected, PersonaScope::new("burner"));
    }

    fn names<'a>(contacts: impl Iterator<Item = &'a Contact>) -> Vec<&'a str> {
        contacts.map(|contact| contact.petname.as_str()).collect()
    }

    #[test]
    fn a_rotated_or_attested_key_still_finds_its_owner() {
        let mut book = book();
        let mut alice = Contact::new("Alice", key(1));
        alice.rotate_to(key(2), Some(ProofMethod::Signature));
        alice.attest(key(10), "mesh-author", key(2), None).unwrap();
        book.insert(alice);

        for known in [1, 2, 10] {
            assert_eq!(
                names(book.by_key(&key(known))),
                vec!["Alice"],
                "key {known}"
            );
        }
        assert_eq!(book.by_key(&key(9)).count(), 0);
    }

    #[test]
    fn a_shared_key_finds_every_holder() {
        // atproto servers once gave many accounts one signing key.
        let mut book = book();
        let shared = key(7);
        for (name, did) in [
            ("Bluesky", "did:plc:z72i7hdynmk6r22z27h6tvur"),
            ("atproto", "did:plc:ewvi7nxzyoun6zhxrhs64oiz"),
        ] {
            book.insert(Contact::new_plc(name, PlcDid::parse(did).unwrap(), shared));
        }
        assert_eq!(names(book.by_key(&shared)), vec!["atproto", "Bluesky"]);
    }

    #[test]
    fn rotation_does_not_move_the_record() {
        let mut book = book();
        book.insert(Contact::new("Alice", key(1)));

        book.get_mut(&anchor(1)).unwrap().rotate_to(key(2), None);

        assert_eq!(book.len(), 1, "rotation must not create a second record");
        assert!(
            book.get(&anchor(1)).is_some(),
            "still filed under the anchor"
        );
    }

    #[test]
    fn a_local_contact_is_filed_under_its_id() {
        let mut book = book();
        let id = LocalId::from_random([3; 16]);
        book.insert(Contact::new_local("Mum", id));
        assert_eq!(book.get(&Anchor::Local(id)).unwrap().petname, "Mum");
    }

    #[test]
    fn lookup_by_handle_and_endpoint() {
        let mut book = book();
        book.insert(
            Contact::new("Alice", key(1))
                .with_handle(Handle::acct("acct:Alice@example.org"))
                .with_endpoint(Endpoint::new(EndpointKind::Misfin, "alice@example.org")),
        );

        assert!(book.by_handle("alice@example.org").is_some());
        assert!(book.by_endpoint("alice@example.org").is_some());
        assert!(book.by_handle("bob@example.org").is_none());
    }

    #[test]
    fn recent_is_most_recent_first_and_omits_the_never_contacted() {
        let mut book = book();
        book.insert(Contact::new("Alice", key(1)));
        book.insert(Contact::new("Bob", key(2)));
        book.insert(Contact::new("Carol", key(3)));

        book.mark_contacted(&anchor(1), 100);
        book.mark_contacted(&anchor(2), 300);

        let names: Vec<&str> = book.recent(10).iter().map(|c| c.petname.as_str()).collect();
        assert_eq!(names, vec!["Bob", "Alice"], "Carol was never contacted");
    }

    #[test]
    fn recent_respects_its_limit() {
        let mut book = book();
        for seed in 1..=5u8 {
            book.insert(Contact::new(format!("P{seed}"), key(seed)));
            book.mark_contacted(&anchor(seed), u64::from(seed) * 10);
        }
        assert_eq!(book.recent(2).len(), 2);
        assert_eq!(book.recent(2)[0].petname, "P5");
    }

    #[test]
    fn marking_an_unknown_anchor_finds_nobody() {
        let mut book = book();
        book.insert(Contact::new("Alice", key(1)));
        assert!(book.mark_contacted(&anchor(1), 500));
        assert_eq!(book.get(&anchor(1)).unwrap().last_contact_ms, Some(500));
        assert!(
            !book.mark_contacted(&anchor(9), 500),
            "nobody is filed there"
        );
    }

    #[test]
    fn tiers_and_alarms_filter() {
        let mut book = book();
        book.insert(Contact::new("Alice", key(1)).with_tier(ContactTier::Kin));
        book.insert(
            Contact::new("Bob", key(2)).with_endpoint(
                Endpoint::new(EndpointKind::Misfin, "b@x.org")
                    .with_trust(TrustState::Mismatched { noticed_ms: 4 }),
            ),
        );

        assert_eq!(names(book.tier(ContactTier::Kin)), vec!["Alice"]);
        assert_eq!(names(book.alarms()), vec!["Bob"]);
    }

    fn mixed_book() -> ContactBook {
        let mut book = book();
        book.insert(Contact::new("Alice", key(1)).with_handle(Handle::acct("a@x.org")));
        book.insert(Contact::new_plc(
            "Bluesky",
            PlcDid::parse("did:plc:z72i7hdynmk6r22z27h6tvur").unwrap(),
            key(2),
        ));
        book.insert(Contact::new_local("Mum", LocalId::from_random([4; 16])));
        book.mark_contacted(&anchor(1), 77);
        book
    }

    #[test]
    fn serde_round_trips_a_mixed_book_in_json_and_binary() {
        let book = mixed_book();

        let json = serde_json::to_string(&book).unwrap();
        assert_eq!(serde_json::from_str::<ContactBook>(&json).unwrap(), book);

        let bytes = postcard::to_allocvec(&book).unwrap();
        assert_eq!(postcard::from_bytes::<ContactBook>(&bytes).unwrap(), book);
    }

    #[test]
    fn a_stored_book_with_two_records_on_one_anchor_fails_to_load() {
        let alice = serde_json::to_string(&Contact::new("Alice", key(1))).unwrap();
        let json = format!(r#"{{"scope":"work","contacts":[{alice},{alice}]}}"#);
        let error = serde_json::from_str::<ContactBook>(&json).unwrap_err();
        assert!(
            error.to_string().contains("share the anchor"),
            "got: {error}"
        );
    }
}
