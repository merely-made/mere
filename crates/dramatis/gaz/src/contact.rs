// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The record itself: one person, however many addresses and keys.

use core::fmt;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::anchor::{Anchor, LocalId, PlcDid};
use crate::endpoint::Endpoint;
use crate::handle::Handle;
use crate::key::TypedKey;
use crate::trust::ProofMethod;

/// How close a contact is.
///
/// Set by you, not derived from trust. Verification is a property of an
/// address; kith and kin describe a relationship, and folding one into the
/// other would mean a peer becomes close because their certificate checked
/// out. The two axes stay separate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ContactTier {
    /// Known to you. The default for anyone you have merely met.
    #[default]
    Kith,
    /// Close: the people a permission can safely default to.
    Kin,
}

/// A key on a contact's root line: one of the keys that has stood for them
/// as a whole, in the order they succeeded each other.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RootKey {
    /// The key.
    pub key: TypedKey,
    /// What the caller checked before recording it: for a rotation, a
    /// signature by the previous root; for a `did:plc` account, a valid PLC
    /// operation. `None` for the key a record was started with, and for a
    /// change nobody proved.
    pub proof: Option<ProofMethod>,
}

/// A key that speaks for a root without being one: derived per protocol, or
/// delegated to a device. Many can be live at once.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestedKey {
    /// The key.
    pub key: TypedKey,
    /// What it was minted for, such as `mesh-author` or a station scope.
    pub scope: String,
    /// The root key that attested it, which is on this contact's root line.
    pub root: TypedKey,
    /// How the caller checked the attestation.
    pub proof: Option<ProofMethod>,
}

/// The local rollup that says these addresses and keys are all the same
/// person.
///
/// Filed under an [`Anchor`] that never moves, labelled with a petname you
/// chose, carrying handles and endpoints that each hold their own trust state.
/// Local only: a contact record is your view of someone and never goes onto a
/// wire.
///
/// Keys come in two shapes. The **root line** holds the keys that have stood
/// for the person as a whole, each succeeding the last. **Attested keys** are
/// concurrent: a personae peer shows a different derived key per protocol and
/// a key per device, all live at once, each vouched for by a root.
///
/// The rules are enforced rather than documented, on construction and on
/// load: a key anchor is the first key of its root line, a key or `did:plc`
/// anchor always holds a key, every attested key names a root this record
/// holds, and no key appears twice.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ContactRecord")]
pub struct Contact {
    /// Your name for them. Not their claim about themselves.
    pub petname: String,
    anchor: Anchor,
    root_line: Vec<RootKey>,
    attested: Vec<AttestedKey>,
    /// Names they go by, each with its own binding state.
    pub handles: Vec<Handle>,
    /// Addresses they are reachable at, each with its own trust state.
    pub endpoints: Vec<Endpoint>,
    /// How close they are.
    pub tier: ContactTier,
    /// When you last exchanged anything, unix milliseconds.
    pub last_contact_ms: Option<u64>,
    /// A private note to yourself.
    pub note: Option<String>,
}

/// A contact as stored, before its rules are checked. Field order is the
/// binary format, so it matches [`Contact`].
#[derive(Deserialize)]
struct ContactRecord {
    petname: String,
    anchor: Anchor,
    root_line: Vec<RootKey>,
    attested: Vec<AttestedKey>,
    handles: Vec<Handle>,
    endpoints: Vec<Endpoint>,
    tier: ContactTier,
    last_contact_ms: Option<u64>,
    note: Option<String>,
}

impl TryFrom<ContactRecord> for Contact {
    type Error = ContactError;

    fn try_from(record: ContactRecord) -> Result<Self, ContactError> {
        let contact = Self {
            petname: record.petname,
            anchor: record.anchor,
            root_line: record.root_line,
            attested: record.attested,
            handles: record.handles,
            endpoints: record.endpoints,
            tier: record.tier,
            last_contact_ms: record.last_contact_ms,
            note: record.note,
        };
        contact.check()?;
        Ok(contact)
    }
}

impl Contact {
    /// Start a record for someone you know a root key for.
    pub fn new(petname: impl Into<String>, key: TypedKey) -> Self {
        Self::started(petname, Anchor::Key(key), Some(key))
    }

    /// Start a record for an atproto account, with the signing key its DID
    /// document names today.
    pub fn new_plc(petname: impl Into<String>, did: PlcDid, signing_key: TypedKey) -> Self {
        Self::started(petname, Anchor::Plc(did), Some(signing_key))
    }

    /// Start a record for someone who has shown you no key yet.
    pub fn new_local(petname: impl Into<String>, id: LocalId) -> Self {
        Self::started(petname, Anchor::Local(id), None)
    }

    fn started(petname: impl Into<String>, anchor: Anchor, first: Option<TypedKey>) -> Self {
        Self {
            petname: petname.into(),
            anchor,
            root_line: first
                .map(|key| RootKey { key, proof: None })
                .into_iter()
                .collect(),
            attested: Vec::new(),
            handles: Vec::new(),
            endpoints: Vec::new(),
            tier: ContactTier::default(),
            last_contact_ms: None,
            note: None,
        }
    }

    fn check(&self) -> Result<(), ContactError> {
        match &self.anchor {
            Anchor::Key(anchor) => match self.root_line.first() {
                None => return Err(ContactError::Rootless),
                Some(first) if first.key != *anchor => return Err(ContactError::AnchorNotFirst),
                Some(_) => {},
            },
            Anchor::Plc(_) if self.root_line.is_empty() => return Err(ContactError::Rootless),
            Anchor::Plc(_) | Anchor::Local(_) => {},
        }
        let mut seen = BTreeSet::new();
        let keys = self.root_line.iter().map(|root| &root.key);
        for key in keys.chain(self.attested.iter().map(|attested| &attested.key)) {
            if !seen.insert(key) {
                return Err(ContactError::DuplicateKey);
            }
        }
        if self
            .attested
            .iter()
            .any(|attested| !self.on_root_line(&attested.root))
        {
            return Err(ContactError::UnknownRoot);
        }
        Ok(())
    }

    /// What this record is filed under.
    ///
    /// Stable for the life of the record, which is why a [`ContactBook`] files
    /// them under it: a rotation never moves the record, and a message signed
    /// with a retired key still finds its way home.
    ///
    /// [`ContactBook`]: crate::ContactBook
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// The root key that stands for them now, if they have shown one.
    pub fn root(&self) -> Option<&TypedKey> {
        self.root_line.last().map(|root| &root.key)
    }

    /// Every root key, oldest first.
    pub fn root_line(&self) -> &[RootKey] {
        &self.root_line
    }

    /// Every attested key, in the order they were recorded.
    pub fn attested(&self) -> &[AttestedKey] {
        &self.attested
    }

    /// The attested keys minted for one scope, such as `mesh-author`.
    pub fn keys_for<'a>(&'a self, scope: &'a str) -> impl Iterator<Item = &'a TypedKey> + 'a {
        self.attested
            .iter()
            .filter(move |attested| attested.scope == scope)
            .map(|attested| &attested.key)
    }

    /// Whether this is one of their keys: root or attested, current or
    /// retired.
    pub fn knows_key(&self, key: &TypedKey) -> bool {
        self.on_root_line(key) || self.attested.iter().any(|attested| attested.key == *key)
    }

    fn on_root_line(&self, key: &TypedKey) -> bool {
        self.root_line.iter().any(|root| root.key == *key)
    }

    /// Record the key that now stands for them as a whole, with what proved
    /// it. On a local record with no key yet, this pins the first one.
    ///
    /// Returns whether anything changed. A key they already hold is a no-op
    /// rather than an error, so replaying an event stream is safe.
    pub fn rotate_to(&mut self, key: TypedKey, proof: Option<ProofMethod>) -> bool {
        if self.knows_key(&key) {
            return false;
        }
        self.root_line.push(RootKey { key, proof });
        true
    }

    /// Record a key that one of their roots vouched for.
    ///
    /// Returns whether anything changed; recording a key twice is a no-op.
    pub fn attest(
        &mut self,
        key: TypedKey,
        scope: impl Into<String>,
        root: TypedKey,
        proof: Option<ProofMethod>,
    ) -> Result<bool, AttestError> {
        if !self.on_root_line(&root) {
            return Err(AttestError::UnknownRoot);
        }
        if self.on_root_line(&key) {
            return Err(AttestError::IsRoot);
        }
        if self.attested.iter().any(|attested| attested.key == key) {
            return Ok(false);
        }
        self.attested.push(AttestedKey {
            key,
            scope: scope.into(),
            root,
            proof,
        });
        Ok(true)
    }

    /// Add a handle, for chaining at construction.
    pub fn with_handle(mut self, handle: Handle) -> Self {
        self.handles.push(handle);
        self
    }

    /// Add an endpoint, for chaining at construction.
    pub fn with_endpoint(mut self, endpoint: Endpoint) -> Self {
        self.endpoints.push(endpoint);
        self
    }

    /// Set the tier, for chaining at construction.
    pub fn with_tier(mut self, tier: ContactTier) -> Self {
        self.tier = tier;
        self
    }

    /// Set the private note, for chaining at construction.
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.note = Some(note.into());
        self
    }

    /// The addresses still worth offering as a way to reach them.
    pub fn reachable(&self) -> impl Iterator<Item = &Endpoint> {
        self.endpoints
            .iter()
            .filter(|endpoint| endpoint.is_usable())
    }

    /// Whether anything about this record should reach a person.
    ///
    /// True when any endpoint or handle has gone mismatched or revoked. A
    /// contact list that hides this is worse than no contact list.
    pub fn has_alarm(&self) -> bool {
        self.endpoints
            .iter()
            .any(|endpoint| endpoint.trust.is_alarming())
            || self
                .handles
                .iter()
                .any(|handle| handle.binding.is_alarming())
    }

    /// Find a handle by the string a person typed.
    pub fn find_handle(&self, query: &str) -> Option<&Handle> {
        self.handles.iter().find(|handle| handle.matches(query))
    }

    /// Find an endpoint by exact address.
    pub fn find_endpoint(&self, address: &str) -> Option<&Endpoint> {
        self.endpoints
            .iter()
            .find(|endpoint| endpoint.address == address)
    }

    /// Note that you exchanged something with them just now.
    ///
    /// Monotonic, like [`Endpoint::mark_used`]: a late or replayed event never
    /// moves recency backwards.
    pub fn mark_contacted(&mut self, now_ms: u64) {
        if self.last_contact_ms.is_none_or(|last| now_ms > last) {
            self.last_contact_ms = Some(now_ms);
        }
    }
}

/// Why a stored record broke one of a contact's rules.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactError {
    /// A key or `did:plc` anchor with no key at all.
    Rootless,
    /// A key anchor that is not the first key of its root line.
    AnchorNotFirst,
    /// An attested key naming a root this record does not hold.
    UnknownRoot,
    /// The same key recorded twice.
    DuplicateKey,
}

impl fmt::Display for ContactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Rootless => "a contact is key-rooted: a key or did:plc anchor needs a key",
            Self::AnchorNotFirst => "a key-anchored contact's root line must start with its anchor",
            Self::UnknownRoot => "an attested key names a root this contact does not hold",
            Self::DuplicateKey => "a key appears twice in one contact",
        })
    }
}

impl core::error::Error for ContactError {}

/// Why an attested key was not recorded.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttestError {
    /// The attesting root is not on this contact's root line.
    UnknownRoot,
    /// The key is already one of their roots.
    IsRoot,
}

impl fmt::Display for AttestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::UnknownRoot => "the attesting key is not one of this contact's roots",
            Self::IsRoot => "the key is already one of this contact's roots",
        })
    }
}

impl core::error::Error for AttestError {}

#[cfg(test)]
#[path = "contact_tests.rs"]
mod tests;
