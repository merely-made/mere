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
//! - `index.json`: the persona label and the item ids, with collections and
//!   aliases kept for the Secret Service.
//!
//! Listing and metadata reads never open a payload. A new item is written
//! payloads first, then metadata, then the index, so an interrupted write
//! leaves only records no read reaches; an item is visible only once indexed.
//! Deletion runs the other way. Every record carries the persona it was filed
//! for, and a load refuses one filed for another by name (ruling 34, gaz's
//! `verify_scope`). Exercising a credential is crate-private: the OTP release
//! gate is its only caller.

use std::fmt;
use std::sync::{Arc, Mutex, MutexGuard};

use chatelaine::{Credential, CredentialId, Item, ItemId, ItemState};
use personae::{IdentityError, PersonaId, SealedRecordChange, SealedRecordStorage};

mod records;

pub(crate) use records::Payload;
use records::{StoredIndex, StoredItem, StoredPayload};

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

/// The points between an insertion's writes, where a test may stop it as a
/// crash would.
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
        let _guard = self.lock();
        let index = self.load_index()?;
        index
            .items
            .iter()
            .map(|id| self.require_indexed(*id))
            .collect()
    }

    /// One item's metadata, or `None` when it is not visible. No payload is
    /// read.
    pub fn get(&self, id: ItemId) -> Result<Option<Item>, ItemStoreError> {
        let _guard = self.lock();
        self.get_locked(id)
    }

    /// Remove one item and its payloads. Removing an absent item succeeds.
    ///
    /// The index entry goes first, so an interrupted delete leaves only
    /// records no read reaches.
    pub fn delete(&self, id: ItemId) -> Result<(), ItemStoreError> {
        let _guard = self.lock();
        self.storage.update_record(
            self.index_path(),
            |index: Option<StoredIndex>| -> Result<_, ItemStoreError> {
                let mut index = self.checked_index(index)?;
                let before = index.items.len();
                index.items.retain(|candidate| *candidate != id);
                let change = if index.items.len() == before {
                    SealedRecordChange::Keep
                } else {
                    SealedRecordChange::Replace(index)
                };
                Ok(((), change))
            },
        )?;
        if let Some(item) = self.load_item(id)? {
            for credential in &item.credentials {
                self.storage
                    .delete_record(self.payload_path(credential.id))?;
            }
        }
        self.storage.delete_record(self.item_path(id))?;
        Ok(())
    }

    /// Seal a new item: payloads, then metadata, then the index entry.
    pub(crate) fn insert(
        &self,
        item: Item,
        payloads: Vec<(CredentialId, Payload)>,
    ) -> Result<Item, ItemStoreError> {
        validate(&item, &payloads)?;
        let _guard = self.lock();
        if self.load_index()?.items.contains(&item.id) {
            return Err(ItemStoreError::InvalidItem(
                "an item with this id is stored",
            ));
        }
        for (credential, payload) in payloads {
            let record = StoredPayload::new(self.persona, item.id, payload);
            self.storage
                .save_record(self.payload_path(credential), &record)?;
        }
        self.crash_point(WriteStep::Payloads)?;
        let record = StoredItem::new(self.persona, item);
        self.storage
            .save_record(self.item_path(record.item.id), &record)?;
        self.crash_point(WriteStep::Metadata)?;
        let id = record.item.id;
        self.storage.update_record(
            self.index_path(),
            |index: Option<StoredIndex>| -> Result<_, ItemStoreError> {
                let mut index = self.checked_index(index)?;
                index.items.push(id);
                Ok(((), SealedRecordChange::Replace(index)))
            },
        )?;
        Ok(record.item)
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
        let _guard = self.lock();
        let stored = self
            .get_locked(item)?
            .ok_or(ItemStoreError::ItemNotFound(item))?;
        if stored.state != ItemState::Vault {
            return Err(ItemStoreError::Quarantined(item).into());
        }
        let held = stored
            .credentials
            .iter()
            .find(|candidate| candidate.id == credential)
            .ok_or(ItemStoreError::CredentialNotFound { item, credential })?;
        self.storage.update_record(
            self.payload_path(credential),
            |record: Option<StoredPayload>| {
                let mut record = record.ok_or_else(|| {
                    ItemStoreError::Inconsistent(format!(
                        "credential {credential} of item {item} has no payload"
                    ))
                })?;
                self.check_payload(item, credential, &record)?;
                let (result, changed) = exercise(&stored, held, &mut record.payload)?;
                let change = if changed {
                    SealedRecordChange::Replace(record)
                } else {
                    SealedRecordChange::Keep
                };
                Ok((result, change))
            },
        )
    }

    fn get_locked(&self, id: ItemId) -> Result<Option<Item>, ItemStoreError> {
        if !self.load_index()?.items.contains(&id) {
            return Ok(None);
        }
        self.require_indexed(id).map(Some)
    }

    fn require_indexed(&self, id: ItemId) -> Result<Item, ItemStoreError> {
        self.load_item(id)?.ok_or_else(|| {
            ItemStoreError::Inconsistent(format!("item {id} is indexed but has no record"))
        })
    }

    fn load_index(&self) -> Result<StoredIndex, ItemStoreError> {
        let index = self.storage.load_record(self.index_path())?;
        self.checked_index(index)
    }

    fn load_item(&self, id: ItemId) -> Result<Option<Item>, ItemStoreError> {
        self.storage
            .load_record(self.item_path(id))?
            .map(|record| self.checked_item(id, record))
            .transpose()
    }

    fn lock(&self) -> MutexGuard<'_, ()> {
        self.transaction
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
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

#[cfg(test)]
mod tests;
