// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Secret Service's secret-free metadata: collection and item ids,
//! labels and lookup attributes, and the snapshot a locked service still
//! answers from (vault lock plan, rulings 11 and 67).
//!
//! Moved from castellan (dramatis repo plan, ruling D30), which serves the
//! Secret Service, re-exports these, and alone mints ids and takes
//! snapshots from its sealed store.

use std::collections::BTreeMap;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{CollectionId, ItemId};

/// Stable identifier for one Secret Service collection: its chatelaine
/// collection's id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SecretCollectionId(
    /// The chatelaine collection's id.
    pub CollectionId,
);

impl SecretCollectionId {
    /// Recover an identifier from a D-Bus object-path UUID.
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(CollectionId::from_bytes(id.into_bytes()))
    }

    /// Return the UUID used in the D-Bus object path.
    pub fn as_uuid(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(*self.0.as_bytes())
    }
}

/// Stable identifier for one Secret Service item: its chatelaine item's id.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SecretItemId(
    /// The chatelaine item's id.
    pub ItemId,
);

impl SecretItemId {
    /// Recover an identifier from a D-Bus object-path UUID.
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(ItemId::from_bytes(id.into_bytes()))
    }

    /// Return the UUID used in the D-Bus object path.
    pub fn as_uuid(self) -> uuid::Uuid {
        uuid::Uuid::from_bytes(*self.0.as_bytes())
    }
}

/// Both ids print as their bare UUID, as they did before P3, so policy and
/// error text keep their form.
macro_rules! uuid_text {
    ($name:ident) => {
        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.as_uuid().fmt(formatter)
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(
                    formatter,
                    concat!(stringify!($name), "({:?})"),
                    self.as_uuid()
                )
            }
        }
    };
}

uuid_text!(SecretCollectionId);
uuid_text!(SecretItemId);

/// Secret-free collection metadata exposed through D-Bus properties.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretCollection {
    /// Stable collection identifier.
    pub id: SecretCollectionId,
    /// User-visible collection label.
    pub label: String,
    /// Unix creation time in seconds.
    pub created: u64,
    /// Unix modification time in seconds.
    pub modified: u64,
}

/// Secret-free item metadata exposed through D-Bus properties and search.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SecretItem {
    /// Stable item identifier.
    pub id: SecretItemId,
    /// Owning collection.
    pub collection: SecretCollectionId,
    /// User-visible item label.
    pub label: String,
    /// Exact-match lookup attributes.
    pub attributes: BTreeMap<String, String>,
    /// Unix creation time in seconds.
    pub created: u64,
    /// Unix modification time in seconds.
    pub modified: u64,
}

impl SecretItem {
    /// Whether this item carries every supplied attribute exactly, as the
    /// Secret Service's search matches.
    pub fn matches_attributes(&self, attributes: &BTreeMap<String, String>) -> bool {
        attributes
            .iter()
            .all(|(key, value)| self.attributes.get(key) == Some(value))
    }
}

/// A metadata lookup that found nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MetadataLookupError {
    /// The requested collection is absent.
    CollectionNotFound(SecretCollectionId),
    /// The requested item is absent.
    ItemNotFound(SecretItemId),
}

impl fmt::Display for MetadataLookupError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CollectionNotFound(id) => {
                write!(formatter, "Secret Service collection {id} does not exist")
            },
            Self::ItemNotFound(id) => write!(formatter, "Secret Service item {id} does not exist"),
        }
    }
}

impl std::error::Error for MetadataLookupError {}

/// Secret-free metadata of one persona's collections, taken while unlocked.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MetadataSnapshot {
    collections: BTreeMap<SecretCollectionId, SecretCollection>,
    members: BTreeMap<SecretCollectionId, Vec<SecretItemId>>,
    items: BTreeMap<SecretItemId, SecretItem>,
    aliases: BTreeMap<String, SecretCollectionId>,
}

impl MetadataSnapshot {
    /// Add one collection and its items, in the collection's own order.
    pub fn insert_collection(&mut self, collection: SecretCollection, items: Vec<SecretItem>) {
        self.members
            .insert(collection.id, items.iter().map(|item| item.id).collect());
        self.items
            .extend(items.into_iter().map(|item| (item.id, item)));
        self.collections.insert(collection.id, collection);
    }

    /// Replace the well-known aliases.
    pub fn set_aliases(&mut self, aliases: BTreeMap<String, SecretCollectionId>) {
        self.aliases = aliases;
    }

    /// Every collection in stable identifier order.
    pub fn collections(&self) -> Vec<SecretCollection> {
        self.collections.values().cloned().collect()
    }

    /// One collection.
    pub fn collection(
        &self,
        id: SecretCollectionId,
    ) -> Result<SecretCollection, MetadataLookupError> {
        self.collections
            .get(&id)
            .cloned()
            .ok_or(MetadataLookupError::CollectionNotFound(id))
    }

    /// A collection's items, in its own order.
    pub fn items(
        &self,
        collection: SecretCollectionId,
    ) -> Result<Vec<SecretItem>, MetadataLookupError> {
        let members = self
            .members
            .get(&collection)
            .ok_or(MetadataLookupError::CollectionNotFound(collection))?;
        Ok(members
            .iter()
            .filter_map(|id| self.items.get(id).cloned())
            .collect())
    }

    /// One item.
    pub fn item(&self, id: SecretItemId) -> Result<SecretItem, MetadataLookupError> {
        self.items
            .get(&id)
            .cloned()
            .ok_or(MetadataLookupError::ItemNotFound(id))
    }

    /// Exact matches for every supplied attribute, as the store searches.
    pub fn search(&self, attributes: &BTreeMap<String, String>) -> Vec<SecretItem> {
        self.items
            .values()
            .filter(|item| item.matches_attributes(attributes))
            .cloned()
            .collect()
    }

    /// The well-known aliases.
    pub fn aliases(&self) -> BTreeMap<String, SecretCollectionId> {
        self.aliases.clone()
    }

    /// One alias.
    pub fn read_alias(&self, alias: &str) -> Option<SecretCollectionId> {
        self.aliases.get(alias).copied()
    }
}
