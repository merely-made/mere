// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Persona-scoped sealed items: chatelaine metadata beside one sealed payload
//! per credential.
//!
//! An [`ItemStore`] keeps one persona's items as three kinds of sealed record
//! under `castellan/items/v1/<persona>/` (ruling 39, 2026-10-01):
//!
//! - `items/<item>.json`: the persona label and the chatelaine [`Item`];
//! - `payloads/<credential>.json`: the persona label, the owning item id and
//!   the credential's secret;
//! - `index.json`: the persona label, the item ids, and the collections with
//!   their members and aliases (ruling 46: one index, one commit point).
//!
//! Listing and metadata reads never open a payload. A new item is written
//! payloads first, then metadata, then the index, so an interrupted write
//! leaves only records no read reaches; an item is visible only once indexed.
//! Deletion runs the other way. A payload is replaced copy-on-write: the new
//! payload under a fresh credential id, then the metadata naming it, then the
//! old payload's removal (ruling 48), so a replace never tears. Every record
//! carries the persona it was filed for, and a load refuses one filed for
//! another by name (ruling 34, gaz's `verify_scope`). Exercising a credential
//! is crate-private: the OTP release gate and the Secret Service are its
//! callers.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use chatelaine::{CollectionId, Credential, CredentialId, Item, ItemId};
use personae::{IdentityError, PersonaId, SealedRecordStorage};

mod records;
mod transaction;

pub(crate) use records::Payload;
#[cfg(any(test, feature = "secret-service"))]
pub(crate) use records::StoredIndex;
pub(crate) use transaction::Transaction;
#[cfg(test)]
use {
    chatelaine::ItemState,
    records::{StoredItem, StoredPayload},
};

/// Failure while reading, writing or exercising a persona's sealed items.
#[derive(Debug)]
pub enum ItemStoreError {
    /// Personae could not read, write or authenticate a sealed record.
    Storage(IdentityError),
    /// A record was filed for another persona than the store's.
    PersonaMismatch {
        /// The persona the store serves.
        expected: PersonaId,
        /// The persona the record carries.
        found: PersonaId,
    },
    /// A record was written by an unsupported store format.
    UnsupportedRecordVersion(u8),
    /// The item is not in this persona's store.
    ItemNotFound(ItemId),
    /// The item holds no credential with this id.
    CredentialNotFound {
        /// The item asked for.
        item: ItemId,
        /// The credential it does not hold.
        credential: CredentialId,
    },
    /// The collection is not in this persona's index.
    CollectionNotFound(CollectionId),
    /// The item is quarantined, so it is never exercised (invariant 12).
    Quarantined(ItemId),
    /// Stored records disagree with one another.
    Inconsistent(String),
    /// An item offered for storage was malformed.
    InvalidItem(&'static str),
}

impl fmt::Display for ItemStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Storage(error) => write!(f, "sealed item storage: {error}"),
            Self::PersonaMismatch { expected, found } => write!(
                f,
                "item record belongs to persona {}, not {}",
                found.as_uuid(),
                expected.as_uuid()
            ),
            Self::UnsupportedRecordVersion(version) => {
                write!(f, "unsupported item record version {version}")
            },
            Self::ItemNotFound(id) => write!(f, "no item {id} for this persona"),
            Self::CredentialNotFound { item, credential } => {
                write!(f, "item {item} holds no credential {credential}")
            },
            Self::CollectionNotFound(id) => write!(f, "no collection {id} for this persona"),
            Self::Quarantined(id) => {
                write!(f, "item {id} is quarantined and cannot be exercised")
            },
            Self::Inconsistent(detail) => write!(f, "item records are inconsistent: {detail}"),
            Self::InvalidItem(reason) => write!(f, "invalid item: {reason}"),
        }
    }
}

impl std::error::Error for ItemStoreError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

impl From<IdentityError> for ItemStoreError {
    fn from(error: IdentityError) -> Self {
        Self::Storage(error)
    }
}

/// The points between a multi-record write's steps, where a test may stop it
/// as a crash would: after the payloads, and after the metadata.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WriteStep {
    Payloads,
    Metadata,
}

/// One persona's sealed items.
///
/// The record key comes from the already-unlocked Personae layer. Clones share
/// one transaction lock; [`crate::resident::CastellanResident`] hands every
/// store for a persona the same lock.
#[derive(Clone)]
pub struct ItemStore {
    storage: SealedRecordStorage,
    persona: PersonaId,
    transaction: Arc<Mutex<()>>,
    #[cfg(test)]
    crash_after: Option<WriteStep>,
}

impl ItemStore {
    /// Open one persona's items over an unlocked record store.
    pub fn new(storage: SealedRecordStorage, persona: PersonaId) -> Self {
        Self::with_transaction(storage, persona, Arc::new(Mutex::new(())))
    }

    pub(crate) fn with_transaction(
        storage: SealedRecordStorage,
        persona: PersonaId,
        transaction: Arc<Mutex<()>>,
    ) -> Self {
        Self {
            storage,
            persona,
            transaction,
            #[cfg(test)]
            crash_after: None,
        }
    }

    /// The persona whose items this store serves.
    pub fn persona(&self) -> PersonaId {
        self.persona
    }

    /// Every visible item's metadata, in the order stored. No payload is read.
    pub fn list(&self) -> Result<Vec<Item>, ItemStoreError> {
        self.begin().list()
    }

    /// One item's metadata, or `None` when it is not visible. No payload is
    /// read.
    pub fn get(&self, id: ItemId) -> Result<Option<Item>, ItemStoreError> {
        self.begin().get(id)
    }

    /// Remove one item and its payloads. Removing an absent item succeeds.
    ///
    /// The index entry goes first, so an interrupted delete leaves only
    /// records no read reaches.
    pub fn delete(&self, id: ItemId) -> Result<(), ItemStoreError> {
        self.begin().delete(id)
    }

    /// Seal a new item: payloads, then metadata, then the index entry.
    pub(crate) fn insert(
        &self,
        item: Item,
        payloads: Vec<(CredentialId, Payload)>,
    ) -> Result<Item, ItemStoreError> {
        let transaction = self.begin();
        let index = transaction.index()?;
        transaction.insert(&index, item, payloads, |_| Ok(()))
    }

    /// Exercise one credential's payload, replacing it when `exercise` says
    /// so before the result is returned.
    ///
    /// Refuses an item that is not in the vault. The payload update runs
    /// inside Personae's single-record transaction, so clones of one opened
    /// store never interleave two updates of one payload.
    pub(crate) fn exercise<R, E>(
        &self,
        item: ItemId,
        credential: CredentialId,
        exercise: impl FnOnce(&Item, &Credential, &mut Payload) -> Result<(R, bool), E>,
    ) -> Result<R, E>
    where
        E: From<ItemStoreError> + From<IdentityError>,
    {
        let transaction = self.begin();
        let stored = transaction
            .get(item)?
            .ok_or(ItemStoreError::ItemNotFound(item))?;
        transaction.exercise(&stored, credential, exercise)
    }

    /// Hold this persona's transaction lock until the returned value drops.
    /// Every read and write of the store runs inside one.
    pub(crate) fn begin(&self) -> Transaction<'_> {
        let guard = self
            .transaction
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Transaction::new(self, guard)
    }

    #[cfg(test)]
    pub(crate) fn crashing_after(mut self, step: WriteStep) -> Self {
        self.crash_after = Some(step);
        self
    }

    #[cfg(test)]
    fn crash_point(&self, step: WriteStep) -> Result<(), ItemStoreError> {
        if self.crash_after == Some(step) {
            return Err(ItemStoreError::Storage(IdentityError::Backend(format!(
                "simulated crash after {step:?}"
            ))));
        }
        Ok(())
    }

    #[cfg(not(test))]
    fn crash_point(&self, _step: WriteStep) -> Result<(), ItemStoreError> {
        Ok(())
    }
}

/// Sixteen random bytes for a new item, credential or collection id.
pub(crate) fn random_id_bytes() -> [u8; 16] {
    uuid::Uuid::new_v4().into_bytes()
}

fn validate(item: &Item, payloads: &[(CredentialId, Payload)]) -> Result<(), ItemStoreError> {
    for (index, (credential, _)) in payloads.iter().enumerate() {
        if !item
            .credentials
            .iter()
            .any(|candidate| candidate.id == *credential)
        {
            return Err(ItemStoreError::InvalidItem(
                "a payload names a credential the item does not hold",
            ));
        }
        if payloads[..index]
            .iter()
            .any(|(earlier, _)| earlier == credential)
        {
            return Err(ItemStoreError::InvalidItem(
                "two payloads name one credential",
            ));
        }
    }
    Ok(())
}

/// The held transaction lock.
type Guard<'a> = MutexGuard<'a, ()>;

#[cfg(test)]
mod tests;
