// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The Secret Service projected from a persona's item store (ruling 17).
//!
//! A collection is a chatelaine collection in the persona's index, its
//! members and the aliases beside it (ruling 46). An item is a chatelaine
//! item holding one `Secret` credential: its label is the item's title, its
//! content type and lookup attributes are the credential's metadata, and its
//! bytes are the credential's sealed payload. Listing, search and every
//! property read are metadata reads; only [`SecretServiceStore::secret`]
//! opens a payload, by exercising it. A secret is replaced copy-on-write
//! (ruling 48), so a replace never tears.

use std::collections::BTreeMap;

use chatelaine::{
    Credential, CredentialId, CredentialKind, Item, ItemId, ItemState, Link,
};
use personae::{IdentityError, PersonaId};
use zeroize::Zeroize;
#[cfg(any(test, target_os = "linux"))]
use zeroize::Zeroizing;

use crate::items::{ItemStore, ItemStoreError, Payload, StoredIndex, Transaction, random_id_bytes};

mod collections;
mod snapshot;
mod validate;

pub use chatelaine::MetadataSnapshot;

/// Resource limits applied before any Secret Service value reaches storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SecretServiceLimits {
    /// Collections allowed for one persona.
    pub max_collections: usize,
    /// Items allowed in one collection.
    pub max_items_per_collection: usize,
    /// Lookup attributes allowed on one item.
    pub max_attributes: usize,
    /// Maximum UTF-8 bytes in a label, attribute key, or content type.
    pub max_name_bytes: usize,
    /// Maximum UTF-8 bytes in one attribute value.
    pub max_attribute_value_bytes: usize,
    /// Maximum bytes in one secret value.
    pub max_secret_bytes: usize,
    /// Concurrent D-Bus transfer sessions retained by the adapter.
    pub max_sessions: usize,
}

impl Default for SecretServiceLimits {
    fn default() -> Self {
        Self {
            max_collections: 32,
            max_items_per_collection: 4_096,
            max_attributes: 64,
            max_name_bytes: 1_024,
            max_attribute_value_bytes: 4_096,
            max_secret_bytes: 1024 * 1024,
            max_sessions: 1_024,
        }
    }
}

// The secret-free metadata types live in chatelaine (dramatis repo plan,
// ruling D30), re-exported at their old paths.
pub use chatelaine::{SecretCollection, SecretCollectionId, SecretItem, SecretItemId};

/// Values required to create or exact-attribute-replace one secret item.
pub struct NewSecretItem {
    /// Collection that will own the item.
    pub collection: SecretCollectionId,
    /// Human-readable item label.
    pub label: String,
    /// Exact-match lookup attributes.
    pub attributes: BTreeMap<String, String>,
    /// Secret bytes to seal.
    pub secret: Vec<u8>,
    /// MIME-style content type for the secret bytes.
    pub content_type: String,
    /// Replace an item in the collection with identical attributes.
    pub replace: bool,
    /// Host-supplied Unix timestamp for creation or replacement.
    pub unix_secs: u64,
}

impl Drop for NewSecretItem {
    fn drop(&mut self) {
        self.secret.zeroize();
    }
}

#[cfg(any(test, target_os = "linux"))]
pub(super) struct SecretValue {
    pub(super) bytes: Zeroizing<Vec<u8>>,
    pub(super) content_type: String,
}

/// Failure while applying a Secret Service storage operation.
#[derive(Debug, thiserror::Error)]
pub enum SecretServiceError {
    /// Personae could not authenticate or update a sealed record.
    #[error("sealed Secret Service storage: {0}")]
    Storage(#[from] IdentityError),
    /// A requested collection is absent.
    #[error("Secret Service collection {0} does not exist")]
    CollectionNotFound(SecretCollectionId),
    /// A requested item is absent.
    #[error("Secret Service item {0} does not exist")]
    ItemNotFound(SecretItemId),
    /// A sealed record was created by an unsupported format.
    #[error("unsupported Secret Service record version {0}")]
    UnsupportedRecordVersion(u8),
    /// Stored records disagree with one another.
    #[error("Secret Service catalog is inconsistent: {0}")]
    InconsistentCatalog(String),
    /// The item store refused a record, such as one filed for another persona.
    #[error("Secret Service item store: {0}")]
    Items(ItemStoreError),
    /// A caller-provided value exceeds the configured resident limits.
    #[error("Secret Service input exceeds configured limit: {0}")]
    Limit(&'static str),
    /// A label, content type, or attribute key is empty or contains controls.
    #[error("Secret Service input is not a presentable {0}")]
    InvalidText(&'static str),
}

impl From<ItemStoreError> for SecretServiceError {
    fn from(error: ItemStoreError) -> Self {
        match error {
            ItemStoreError::Storage(error) => Self::Storage(error),
            ItemStoreError::Locked => Self::Storage(IdentityError::Locked),
            ItemStoreError::UnsupportedRecordVersion(version) => {
                Self::UnsupportedRecordVersion(version)
            },
            ItemStoreError::ItemNotFound(id) => Self::ItemNotFound(SecretItemId(id)),
            ItemStoreError::CollectionNotFound(id) => {
                Self::CollectionNotFound(SecretCollectionId(id))
            },
            ItemStoreError::Inconsistent(detail) => Self::InconsistentCatalog(detail),
            other => Self::Items(other),
        }
    }
}

impl From<chatelaine::MetadataLookupError> for SecretServiceError {
    fn from(error: chatelaine::MetadataLookupError) -> Self {
        match error {
            chatelaine::MetadataLookupError::CollectionNotFound(id) => Self::CollectionNotFound(id),
            chatelaine::MetadataLookupError::ItemNotFound(id) => Self::ItemNotFound(id),
        }
    }
}

/// Whether `item` carries every supplied attribute exactly.
fn matches(item: &SecretItem, attributes: &BTreeMap<String, String>) -> bool {
    item.matches_attributes(attributes)
}

/// Persona-scoped Secret Service collections held by one resident authority.
#[derive(Clone)]
pub struct SecretServiceStore {
    items: ItemStore,
    limits: SecretServiceLimits,
}

impl SecretServiceStore {
    pub(crate) fn new(items: ItemStore, limits: SecretServiceLimits) -> Self {
        Self { items, limits }
    }

    /// Persona whose collections this store serves.
    pub fn persona(&self) -> PersonaId {
        self.items.persona()
    }

    #[cfg(any(test, target_os = "linux"))]
    pub(super) fn limits(&self) -> SecretServiceLimits {
        self.limits
    }

    /// Return every item in a collection. No payload is read.
    pub fn items(
        &self,
        collection: SecretCollectionId,
    ) -> Result<Vec<SecretItem>, SecretServiceError> {
        let transaction = self.items.begin();
        let index = transaction.index()?;
        let mut found = Vec::new();
        for link in &collections::find(&index, collection)?.items {
            if let Some(item) = project(&transaction.indexed(link.item)?, collection) {
                found.push(item);
            }
        }
        Ok(found)
    }

    /// Search all collections using exact matches for every supplied
    /// attribute. Metadata only: no payload is read.
    pub fn search(
        &self,
        attributes: &BTreeMap<String, String>,
    ) -> Result<Vec<SecretItem>, SecretServiceError> {
        self.validate_attributes(attributes)?;
        let transaction = self.items.begin();
        let index = transaction.index()?;
        let mut found = Vec::new();
        for collection in &index.collections {
            for link in &collection.items {
                let item = transaction.indexed(link.item)?;
                let Some(item) = project(&item, SecretCollectionId(collection.id)) else {
                    continue;
                };
                if matches(&item, attributes) {
                    found.push(item);
                }
            }
        }
        found.sort_by_key(|item| item.id);
        Ok(found)
    }

    /// Create or exact-attribute-replace one item.
    ///
    /// The whole call, replace included, holds the persona's transaction
    /// lock, and a replace is copy-on-write (ruling 48).
    pub fn create_item(
        &self,
        mut request: NewSecretItem,
    ) -> Result<SecretItem, SecretServiceError> {
        self.validate_name(&request.label, "item label")?;
        self.validate_name(&request.content_type, "content type")?;
        self.validate_attributes(&request.attributes)?;
        self.validate_secret(&request.secret)?;
        let transaction = self.items.begin();
        let index = transaction.index()?;
        let owner = request.collection;
        let collection = collections::find(&index, owner)?;
        if request.replace {
            for link in &collection.items {
                let mut current = transaction.indexed(link.item)?;
                let Some(old) = matching(&current, &request.attributes) else {
                    continue;
                };
                current.title = std::mem::take(&mut request.label);
                current.modified_at = Some(request.unix_secs);
                set_content_type(&mut current, std::mem::take(&mut request.content_type));
                let payload = Payload::Secret {
                    bytes: std::mem::take(&mut request.secret),
                };
                let item = transaction.replace_payload(current, old, payload)?;
                return shown(&item, owner);
            }
        }
        if collection.items.len() >= self.limits.max_items_per_collection {
            return Err(SecretServiceError::Limit("items per collection"));
        }
        let credential = CredentialId::from_random(random_id_bytes());
        let item = Item {
            id: ItemId::from_random(random_id_bytes()),
            source_id: None,
            title: std::mem::take(&mut request.label),
            subtitle: None,
            scope: None,
            tags: Vec::new(),
            favorite: false,
            created_at: Some(request.unix_secs),
            modified_at: Some(request.unix_secs),
            credentials: vec![Credential {
                id: credential,
                kind: CredentialKind::Secret {
                    content_type: std::mem::take(&mut request.content_type),
                    attributes: std::mem::take(&mut request.attributes),
                },
            }],
            state: ItemState::Vault,
        };
        let payload = Payload::Secret {
            bytes: std::mem::take(&mut request.secret),
        };
        let (id, unix_secs) = (item.id, request.unix_secs);
        let item = transaction.insert(&index, item, vec![(credential, payload)], |index| {
            let collection = index
                .collections
                .iter_mut()
                .find(|candidate| candidate.id == owner.0)
                .ok_or(ItemStoreError::CollectionNotFound(owner.0))?;
            collection.items.push(Link { item: id });
            collection.modified_at = Some(unix_secs);
            Ok(())
        })?;
        shown(&item, owner)
    }

    /// Read one item's secret-free metadata.
    pub fn item(&self, id: SecretItemId) -> Result<SecretItem, SecretServiceError> {
        let transaction = self.items.begin();
        let (item, collection) = require(&transaction, &transaction.index()?, id)?;
        shown(&item, collection)
    }

    /// Change an item's lookup attributes.
    pub fn set_item_attributes(
        &self,
        id: SecretItemId,
        attributes: BTreeMap<String, String>,
        unix_secs: u64,
    ) -> Result<(), SecretServiceError> {
        self.validate_attributes(&attributes)?;
        self.update_metadata(id, |item| {
            if let [credential] = item.credentials.as_mut_slice()
                && let CredentialKind::Secret {
                    attributes: held, ..
                } = &mut credential.kind
            {
                *held = attributes;
            }
            item.modified_at = Some(unix_secs);
        })
    }

    /// Change an item's label.
    pub fn set_item_label(
        &self,
        id: SecretItemId,
        label: &str,
        unix_secs: u64,
    ) -> Result<(), SecretServiceError> {
        self.validate_name(label, "item label")?;
        self.update_metadata(id, |item| {
            item.title = label.to_string();
            item.modified_at = Some(unix_secs);
        })
    }

    /// Replace an item's bytes and content type, copy-on-write (ruling 48).
    #[cfg(any(test, target_os = "linux"))]
    pub(super) fn set_secret(
        &self,
        id: SecretItemId,
        secret: Vec<u8>,
        content_type: &str,
        unix_secs: u64,
    ) -> Result<(), SecretServiceError> {
        let mut secret = Zeroizing::new(secret);
        self.validate_secret(&secret)?;
        self.validate_name(content_type, "content type")?;
        let transaction = self.items.begin();
        let (mut item, _) = require(&transaction, &transaction.index()?, id)?;
        let old = item.credentials[0].id;
        set_content_type(&mut item, content_type.to_string());
        item.modified_at = Some(unix_secs);
        let payload = Payload::Secret {
            bytes: std::mem::take(&mut *secret),
        };
        transaction.replace_payload(item, old, payload)?;
        Ok(())
    }

    /// Release an item's bytes by exercising its payload, the one read that
    /// opens one.
    #[cfg(any(test, target_os = "linux"))]
    pub(super) fn secret(&self, id: SecretItemId) -> Result<SecretValue, SecretServiceError> {
        let transaction = self.items.begin();
        let (item, _) = require(&transaction, &transaction.index()?, id)?;
        let credential = &item.credentials[0];
        let CredentialKind::Secret { content_type, .. } = &credential.kind else {
            return Err(SecretServiceError::ItemNotFound(id));
        };
        let bytes = transaction.exercise(&item, credential.id, |_, _, payload| match payload {
            Payload::Secret { bytes } => Ok((Zeroizing::new(bytes.clone()), false)),
            Payload::Otp { .. } => Err(SecretServiceError::InconsistentCatalog(format!(
                "item {id} holds a payload that is not a secret"
            ))),
        })?;
        Ok(SecretValue {
            bytes,
            content_type: content_type.clone(),
        })
    }

    /// Delete an item: its index entry and membership, then its records.
    pub fn delete_item(&self, id: SecretItemId) -> Result<(), SecretServiceError> {
        let transaction = self.items.begin();
        require(&transaction, &transaction.index()?, id)?;
        transaction.delete(id.0)?;
        Ok(())
    }

    fn update_metadata(
        &self,
        id: SecretItemId,
        change: impl FnOnce(&mut Item),
    ) -> Result<(), SecretServiceError> {
        let transaction = self.items.begin();
        let (mut item, _) = require(&transaction, &transaction.index()?, id)?;
        change(&mut item);
        transaction.save_metadata(item)?;
        Ok(())
    }
}

/// The Secret Service item `id` names, with its collection. An item outside
/// every collection, or not holding one `Secret` credential, is not one.
fn require(
    transaction: &Transaction<'_>,
    index: &StoredIndex,
    id: SecretItemId,
) -> Result<(Item, SecretCollectionId), SecretServiceError> {
    let collection = index
        .items
        .contains(&id.0)
        .then(|| {
            index
                .collections
                .iter()
                .find(|collection| collection.items.contains(&Link { item: id.0 }))
        })
        .flatten()
        .ok_or(SecretServiceError::ItemNotFound(id))?;
    let item = transaction.indexed(id.0)?;
    if project(&item, SecretCollectionId(collection.id)).is_none() {
        return Err(SecretServiceError::ItemNotFound(id));
    }
    Ok((item, SecretCollectionId(collection.id)))
}

/// The item as the Secret Service shows it, if it holds one `Secret`
/// credential.
fn project(item: &Item, collection: SecretCollectionId) -> Option<SecretItem> {
    let [credential] = item.credentials.as_slice() else {
        return None;
    };
    let CredentialKind::Secret { attributes, .. } = &credential.kind else {
        return None;
    };
    Some(SecretItem {
        id: SecretItemId(item.id),
        collection,
        label: item.title.clone(),
        attributes: attributes.clone(),
        created: item.created_at.unwrap_or(0),
        modified: item.modified_at.unwrap_or(0),
    })
}

fn shown(item: &Item, collection: SecretCollectionId) -> Result<SecretItem, SecretServiceError> {
    project(item, collection).ok_or_else(|| {
        SecretServiceError::InconsistentCatalog(format!(
            "item {} holds no single secret credential",
            item.id
        ))
    })
}

/// The credential to replace when `item` is a secret with exactly these
/// attributes.
fn matching(item: &Item, attributes: &BTreeMap<String, String>) -> Option<CredentialId> {
    let [credential] = item.credentials.as_slice() else {
        return None;
    };
    match &credential.kind {
        CredentialKind::Secret {
            attributes: held, ..
        } if held == attributes => Some(credential.id),
        _ => None,
    }
}

fn set_content_type(item: &mut Item, content_type: String) {
    if let [credential] = item.credentials.as_mut_slice()
        && let CredentialKind::Secret {
            content_type: held, ..
        } = &mut credential.kind
    {
        *held = content_type;
    }
}

#[cfg(test)]
#[path = "store_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "store_limit_tests.rs"]
mod limit_tests;

#[cfg(test)]
#[path = "store_replace_tests.rs"]
mod replace_tests;
