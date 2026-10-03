// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Persona-scoped contact books over host-supplied Muniment slots.
//!
//! Enabled by `muniment`. The host chooses the backend, codec and at-rest
//! sealing. Gaz neither encrypts bytes nor verifies retained signatures.
//!
//! The embedding host also enables its chosen Muniment codec (`json` here).
//!
//! ```
//! use gaz::{Contact, ContactBook, PersonaScope, TypedKey, load_book, save_book};
//! use muniment::{JsonCodec, MemoryBackend, SlotStore};
//!
//! # pollster::block_on(async {
//! let slots = SlotStore::<_, JsonCodec>::new(MemoryBackend::new());
//! let scope = PersonaScope::new("work");
//! let mut book = ContactBook::new(scope.clone());
//! book.insert(Contact::new("Alice", TypedKey::ed25519([1; 32])));
//! save_book(&slots, &book).await.unwrap();
//! let loaded = load_book(&slots, &scope).await.unwrap();
//! assert_eq!(loaded, book);
//! # });
//! ```

use core::fmt;

use muniment::{Backend, Codec, SlotStore, StoreError};

use crate::{ContactBook, PersonaScope, ScopeMismatch};

/// Why a contact book could not be saved or loaded.
#[derive(Debug, Clone, PartialEq)]
pub enum PersistenceError {
    /// Backend or codec failure, including refusal of malformed records.
    Store(StoreError),
    /// The stored book belongs to a different persona than its slot.
    Scope(ScopeMismatch),
}

impl fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Store(error) => error.fmt(f),
            Self::Scope(error) => error.fmt(f),
        }
    }
}

impl core::error::Error for PersistenceError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Store(error) => Some(error),
            Self::Scope(error) => Some(error),
        }
    }
}

impl From<StoreError> for PersistenceError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<ScopeMismatch> for PersistenceError {
    fn from(error: ScopeMismatch) -> Self {
        Self::Scope(error)
    }
}

fn slot_key(scope: &PersonaScope) -> String {
    format!("personas/{}/contacts", scope.as_str())
}

/// Save the book at `personas/<scope>/contacts`, replacing its previous value.
///
/// The scope is the book's own opaque label. The backend must support that
/// slot key; backend restrictions surface as [`PersistenceError::Store`].
/// Hosts holding private contacts should supply a backend that seals values.
pub async fn save_book<B: Backend, C: Codec>(
    slots: &SlotStore<B, C>,
    book: &ContactBook,
) -> Result<(), PersistenceError> {
    slots.save(&slot_key(book.scope()), book).await?;
    Ok(())
}

/// Load the expected persona's book, or an empty book if its slot is missing.
///
/// Deserialization enforces the complete contact model before scope is
/// checked. Malformed records, storage failures and mis-filed books are
/// errors, never an empty or partial success. Retained proof artifacts still
/// need the caller's signature checks and current authority policy.
pub async fn load_book<B: Backend, C: Codec>(
    slots: &SlotStore<B, C>,
    expected_scope: &PersonaScope,
) -> Result<ContactBook, PersistenceError> {
    let book = slots
        .load::<ContactBook>(&slot_key(expected_scope))
        .await?
        .unwrap_or_else(|| ContactBook::new(expected_scope.clone()));
    book.verify_scope(expected_scope)?;
    Ok(book)
}

#[cfg(test)]
#[path = "persistence_tests.rs"]
mod tests;
