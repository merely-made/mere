// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The reads and writes of an item store, made under its held transaction
//! lock, so a multi-record change is never interleaved with another.

use chatelaine::{Collection, Credential, CredentialId, Item, ItemId, ItemState};
use personae::{IdentityError, SealedRecordChange};

use super::records::{Payload, StoredIndex, StoredItem, StoredPayload};
use super::{Guard, ItemStore, ItemStoreError, WriteStep, validate};

/// One persona's held transaction lock and the store it guards. Dropping it
/// releases the lock.
pub(crate) struct Transaction<'a> {
    store: &'a ItemStore,
    _guard: Guard<'a>,
}

impl<'a> Transaction<'a> {
    pub(super) fn new(store: &'a ItemStore, guard: Guard<'a>) -> Self {
        Self {
            store,
            _guard: guard,
        }
    }

    /// The persona's index, checked for version and persona.
    pub(crate) fn index(&self) -> Result<StoredIndex, ItemStoreError> {
        let index = self.store.storage.load_record(self.store.index_path())?;
        self.store.checked_index(index)
    }

    /// Change the index in one sealed write. A failed change writes nothing.
    pub(crate) fn update_index<R>(
        &self,
        change: impl FnOnce(&mut StoredIndex) -> Result<R, ItemStoreError>,
    ) -> Result<R, ItemStoreError> {
        self.store.storage.update_record(
            self.store.index_path(),
            |index: Option<StoredIndex>| -> Result<_, ItemStoreError> {
                let mut index = self.store.checked_index(index)?;
                let result = change(&mut index)?;
                Ok((result, SealedRecordChange::Replace(index)))
            },
        )
    }

    pub(crate) fn list(&self) -> Result<Vec<Item>, ItemStoreError> {
        self.index()?
            .items
            .iter()
            .map(|id| self.indexed(*id))
            .collect()
    }

    pub(crate) fn get(&self, id: ItemId) -> Result<Option<Item>, ItemStoreError> {
        if !self.index()?.items.contains(&id) {
            return Ok(None);
        }
        self.indexed(id).map(Some)
    }

    /// The metadata of an item this transaction found in the index.
    pub(crate) fn indexed(&self, id: ItemId) -> Result<Item, ItemStoreError> {
        self.load_item(id)?.ok_or_else(|| {
            ItemStoreError::Inconsistent(format!("item {id} is indexed but has no record"))
        })
    }

    /// Seal a new item: payloads, then metadata, then one index write that
    /// adds its id and applies `link`, such as a collection membership.
    /// `index` is this transaction's read of the index.
    pub(crate) fn insert(
        &self,
        index: &StoredIndex,
        item: Item,
        payloads: Vec<(CredentialId, Payload)>,
        link: impl FnOnce(&mut StoredIndex) -> Result<(), ItemStoreError>,
    ) -> Result<Item, ItemStoreError> {
        validate(&item, &payloads)?;
        if index.items.contains(&item.id) {
            return Err(ItemStoreError::InvalidItem(
                "an item with this id is stored",
            ));
        }
        for (credential, payload) in payloads {
            self.save_payload(item.id, credential, payload)?;
        }
        self.store.crash_point(WriteStep::Payloads)?;
        let record = StoredItem::new(self.store.persona, item);
        self.store
            .storage
            .save_record(self.store.item_path(record.item.id), &record)?;
        self.store.crash_point(WriteStep::Metadata)?;
        let id = record.item.id;
        self.update_index(|index| {
            index.items.push(id);
            link(index)
        })?;
        Ok(record.item)
    }

    /// Remove one item: its index entry and memberships first, then its
    /// records. Removing an absent item succeeds.
    pub(crate) fn delete(&self, id: ItemId) -> Result<(), ItemStoreError> {
        self.store.storage.update_record(
            self.store.index_path(),
            |index: Option<StoredIndex>| -> Result<_, ItemStoreError> {
                let mut index = self.store.checked_index(index)?;
                let before = index.items.len();
                index.items.retain(|candidate| *candidate != id);
                let unlinked = unlink(&mut index.collections, id);
                let change = if index.items.len() == before && !unlinked {
                    SealedRecordChange::Keep
                } else {
                    SealedRecordChange::Replace(index)
                };
                Ok(((), change))
            },
        )?;
        self.remove_records(id)
    }

    /// Remove an item's payloads, then its metadata. Only for an item no
    /// index entry reaches any longer.
    pub(crate) fn remove_records(&self, id: ItemId) -> Result<(), ItemStoreError> {
        if let Some(item) = self.load_item(id)? {
            for credential in &item.credentials {
                self.store
                    .storage
                    .delete_record(self.store.payload_path(credential.id))?;
            }
        }
        self.store.storage.delete_record(self.store.item_path(id))?;
        Ok(())
    }

    /// Rewrite an indexed item's metadata. Its payloads are untouched.
    #[cfg(feature = "secret-service")]
    pub(crate) fn save_metadata(&self, item: Item) -> Result<Item, ItemStoreError> {
        let record = StoredItem::new(self.store.persona, item);
        self.store
            .storage
            .save_record(self.store.item_path(record.item.id), &record)?;
        Ok(record.item)
    }

    /// Replace credential `old`'s payload copy-on-write (ruling 48): the new
    /// payload under a fresh credential id, then `item` (already carrying any
    /// metadata change) naming it, which is the one commit point, then the
    /// old payload's removal. Interrupted before the commit, the old item is
    /// whole; after it, the new one is; either way the leftover payload is an
    /// orphan no read reaches.
    #[cfg(feature = "secret-service")]
    pub(crate) fn replace_payload(
        &self,
        mut item: Item,
        old: CredentialId,
        payload: Payload,
    ) -> Result<Item, ItemStoreError> {
        let id = item.id;
        let fresh = CredentialId::from_random(super::random_id_bytes());
        let slot = item
            .credentials
            .iter_mut()
            .find(|candidate| candidate.id == old)
            .ok_or(ItemStoreError::CredentialNotFound {
                item: id,
                credential: old,
            })?;
        slot.id = fresh;
        self.save_payload(id, fresh, payload)?;
        self.store.crash_point(WriteStep::Payloads)?;
        let item = self.save_metadata(item)?;
        self.store.crash_point(WriteStep::Metadata)?;
        self.store
            .storage
            .delete_record(self.store.payload_path(old))?;
        Ok(item)
    }

    /// Exercise one credential of `stored`, an item this transaction read.
    /// Refuses an item that is not in the vault (invariant 12).
    pub(crate) fn exercise<R, E>(
        &self,
        stored: &Item,
        credential: CredentialId,
        exercise: impl FnOnce(&Item, &Credential, &mut Payload) -> Result<(R, bool), E>,
    ) -> Result<R, E>
    where
        E: From<ItemStoreError> + From<IdentityError>,
    {
        let item = stored.id;
        if stored.state != ItemState::Vault {
            return Err(ItemStoreError::Quarantined(item).into());
        }
        let held = stored
            .credentials
            .iter()
            .find(|candidate| candidate.id == credential)
            .ok_or(ItemStoreError::CredentialNotFound { item, credential })?;
        self.store.storage.update_record(
            self.store.payload_path(credential),
            |record: Option<StoredPayload>| {
                let mut record = record.ok_or_else(|| {
                    ItemStoreError::Inconsistent(format!(
                        "credential {credential} of item {item} has no payload"
                    ))
                })?;
                self.store.check_payload(item, credential, &record)?;
                let (result, changed) = exercise(stored, held, &mut record.payload)?;
                let change = if changed {
                    SealedRecordChange::Replace(record)
                } else {
                    SealedRecordChange::Keep
                };
                Ok((result, change))
            },
        )
    }

    fn save_payload(
        &self,
        item: ItemId,
        credential: CredentialId,
        payload: Payload,
    ) -> Result<(), ItemStoreError> {
        let record = StoredPayload::new(self.store.persona, item, payload);
        self.store
            .storage
            .save_record(self.store.payload_path(credential), &record)?;
        Ok(())
    }

    fn load_item(&self, id: ItemId) -> Result<Option<Item>, ItemStoreError> {
        self.store
            .storage
            .load_record(self.store.item_path(id))?
            .map(|record| self.store.checked_item(id, record))
            .transpose()
    }
}

/// Drop every link to `id`, nested collections included; true if any went.
fn unlink(collections: &mut [Collection], id: ItemId) -> bool {
    let mut unlinked = false;
    for collection in collections {
        let before = collection.items.len();
        collection.items.retain(|link| link.item != id);
        unlinked |= collection.items.len() != before;
        unlinked |= unlink(&mut collection.sub_collections, id);
    }
    unlinked
}
