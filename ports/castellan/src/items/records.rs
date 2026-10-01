// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The three sealed record shapes of an item store, and where each is filed.

use std::collections::BTreeMap;

use chatelaine::{Collection, CollectionId, CredentialId, Item, ItemId};
use personae::PersonaId;
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use super::{ItemStore, ItemStoreError};

const RECORD_DIRECTORY: &str = "castellan/items/v1";
const RECORD_VERSION: u8 = 1;

/// The item ids a persona's store shows, in insertion order. Collections and
/// aliases are kept for the Secret Service, which fills them.
#[derive(Serialize, Deserialize)]
pub(crate) struct StoredIndex {
    pub(crate) version: u8,
    pub(crate) persona: PersonaId,
    pub(crate) items: Vec<ItemId>,
    pub(crate) collections: Vec<Collection>,
    pub(crate) aliases: BTreeMap<String, CollectionId>,
}

impl StoredIndex {
    pub(crate) fn empty(persona: PersonaId) -> Self {
        Self {
            version: RECORD_VERSION,
            persona,
            items: Vec::new(),
            collections: Vec::new(),
            aliases: BTreeMap::new(),
        }
    }
}

/// One item's secret-free metadata.
#[derive(Serialize, Deserialize)]
pub(crate) struct StoredItem {
    pub(crate) version: u8,
    pub(crate) persona: PersonaId,
    pub(crate) item: Item,
}

impl StoredItem {
    pub(crate) fn new(persona: PersonaId, item: Item) -> Self {
        Self {
            version: RECORD_VERSION,
            persona,
            item,
        }
    }
}

/// One credential's sealed secret, naming the item that owns it.
#[derive(Serialize, Deserialize)]
pub(crate) struct StoredPayload {
    pub(crate) version: u8,
    pub(crate) persona: PersonaId,
    pub(crate) item: ItemId,
    pub(crate) payload: Payload,
}

impl StoredPayload {
    pub(crate) fn new(persona: PersonaId, item: ItemId, payload: Payload) -> Self {
        Self {
            version: RECORD_VERSION,
            persona,
            item,
            payload,
        }
    }
}

/// A credential's secret, which no chatelaine type holds. No `Debug`, and the
/// bytes are zeroized on drop.
#[derive(Serialize, Deserialize)]
pub(crate) enum Payload {
    /// An OTP seed, and the next HOTP counter for a counter-based credential.
    Otp {
        secret: Vec<u8>,
        counter: Option<u64>,
    },
}

impl Drop for Payload {
    fn drop(&mut self) {
        let Payload::Otp { secret, .. } = self;
        secret.zeroize();
    }
}

impl ItemStore {
    /// Refuse a record filed for another persona, naming both (ruling 34).
    pub(super) fn check_scope(&self, found: PersonaId) -> Result<(), ItemStoreError> {
        if found == self.persona {
            Ok(())
        } else {
            Err(ItemStoreError::PersonaMismatch {
                expected: self.persona,
                found,
            })
        }
    }

    pub(super) fn checked_index(
        &self,
        index: Option<StoredIndex>,
    ) -> Result<StoredIndex, ItemStoreError> {
        let Some(index) = index else {
            return Ok(StoredIndex::empty(self.persona));
        };
        check_version(index.version)?;
        self.check_scope(index.persona)?;
        Ok(index)
    }

    pub(super) fn checked_item(
        &self,
        id: ItemId,
        record: StoredItem,
    ) -> Result<Item, ItemStoreError> {
        check_version(record.version)?;
        self.check_scope(record.persona)?;
        if record.item.id != id {
            return Err(ItemStoreError::Inconsistent(format!(
                "the record filed for item {id} holds item {}",
                record.item.id
            )));
        }
        Ok(record.item)
    }

    pub(super) fn check_payload(
        &self,
        item: ItemId,
        credential: CredentialId,
        record: &StoredPayload,
    ) -> Result<(), ItemStoreError> {
        check_version(record.version)?;
        self.check_scope(record.persona)?;
        if record.item != item {
            return Err(ItemStoreError::Inconsistent(format!(
                "the payload of credential {credential} belongs to item {}, not {item}",
                record.item
            )));
        }
        Ok(())
    }

    fn base_path(&self) -> String {
        format!("{RECORD_DIRECTORY}/{}", self.persona.as_uuid())
    }

    pub(crate) fn index_path(&self) -> String {
        format!("{}/index.json", self.base_path())
    }

    pub(crate) fn item_path(&self, id: ItemId) -> String {
        format!("{}/items/{id}.json", self.base_path())
    }

    pub(crate) fn payload_path(&self, id: CredentialId) -> String {
        format!("{}/payloads/{id}.json", self.base_path())
    }
}

fn check_version(version: u8) -> Result<(), ItemStoreError> {
    if version == RECORD_VERSION {
        Ok(())
    } else {
        Err(ItemStoreError::UnsupportedRecordVersion(version))
    }
}
