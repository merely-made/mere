// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Collections and aliases: chatelaine collections in the persona's index,
//! changed by one index write each.

use std::collections::BTreeSet;

use chatelaine::{Collection, ItemId};

use super::{SecretCollection, SecretCollectionId, SecretServiceError, SecretServiceStore};
use crate::items::{ItemStoreError, StoredIndex, Transaction};

impl SecretServiceStore {
    /// Ensure and return the collection behind the conventional `default` alias.
    pub fn ensure_default_collection(
        &self,
        label: &str,
        unix_secs: u64,
    ) -> Result<SecretCollection, SecretServiceError> {
        self.validate_name(label, "collection label")?;
        let transaction = self.items.begin();
        let index = transaction.index()?;
        if let Some(id) = index.aliases.get("default").copied() {
            return find(&index, SecretCollectionId(id)).map(project);
        }
        self.create_locked(&transaction, &index, label, Some("default"), unix_secs)
    }

    /// Return every collection in stable identifier order.
    pub fn collections(&self) -> Result<Vec<SecretCollection>, SecretServiceError> {
        let index = self.items.begin().index()?;
        let mut collections: Vec<_> = index.collections.iter().map(project).collect();
        collections.sort_by_key(|collection| collection.id);
        Ok(collections)
    }

    /// Read one collection's secret-free metadata.
    pub fn collection(
        &self,
        id: SecretCollectionId,
    ) -> Result<SecretCollection, SecretServiceError> {
        let index = self.items.begin().index()?;
        find(&index, id).map(project)
    }

    /// Create a collection, optionally binding one well-known alias.
    pub fn create_collection(
        &self,
        label: &str,
        alias: Option<&str>,
        unix_secs: u64,
    ) -> Result<SecretCollection, SecretServiceError> {
        self.validate_name(label, "collection label")?;
        if let Some(alias) = alias {
            self.validate_alias(alias)?;
        }
        let transaction = self.items.begin();
        let index = transaction.index()?;
        if let Some(existing) = alias.and_then(|alias| index.aliases.get(alias).copied()) {
            return relabel(&transaction, SecretCollectionId(existing), label, unix_secs);
        }
        self.create_locked(&transaction, &index, label, alias, unix_secs)
    }

    /// Resolve a well-known alias.
    pub fn read_alias(
        &self,
        alias: &str,
    ) -> Result<Option<SecretCollectionId>, SecretServiceError> {
        let index = self.items.begin().index()?;
        Ok(index.aliases.get(alias).copied().map(SecretCollectionId))
    }

    pub(in crate::secret_service) fn aliases(
        &self,
    ) -> Result<std::collections::BTreeMap<String, SecretCollectionId>, SecretServiceError> {
        let index = self.items.begin().index()?;
        Ok(index
            .aliases
            .into_iter()
            .map(|(alias, id)| (alias, SecretCollectionId(id)))
            .collect())
    }

    /// Point or remove a well-known alias.
    pub fn set_alias(
        &self,
        alias: &str,
        collection: Option<SecretCollectionId>,
    ) -> Result<(), SecretServiceError> {
        self.validate_alias(alias)?;
        self.items.begin().update_index(|index| {
            match collection {
                Some(SecretCollectionId(id)) => {
                    if !index.collections.iter().any(|candidate| candidate.id == id) {
                        return Err(ItemStoreError::CollectionNotFound(id));
                    }
                    index.aliases.insert(alias.to_string(), id);
                },
                None => {
                    index.aliases.remove(alias);
                },
            }
            Ok(())
        })?;
        Ok(())
    }

    /// Change a collection label.
    pub fn set_collection_label(
        &self,
        id: SecretCollectionId,
        label: &str,
        unix_secs: u64,
    ) -> Result<(), SecretServiceError> {
        self.validate_name(label, "collection label")?;
        relabel(&self.items.begin(), id, label, unix_secs).map(|_| ())
    }

    /// Delete a collection and every item it contained.
    ///
    /// The index write that drops the collection, its members and its
    /// aliases goes first; the members' records follow, so an interrupted
    /// delete leaves only records no read reaches.
    pub fn delete_collection(&self, id: SecretCollectionId) -> Result<(), SecretServiceError> {
        let transaction = self.items.begin();
        let members = transaction.update_index(|index| {
            let position = index
                .collections
                .iter()
                .position(|candidate| candidate.id == id.0)
                .ok_or(ItemStoreError::CollectionNotFound(id.0))?;
            let removed = index.collections.remove(position);
            let mut members = BTreeSet::new();
            gather(&removed, &mut members);
            index.items.retain(|item| !members.contains(item));
            index.aliases.retain(|_, candidate| *candidate != id.0);
            Ok(members)
        })?;
        for member in members {
            transaction.remove_records(member)?;
        }
        Ok(())
    }

    fn create_locked(
        &self,
        transaction: &Transaction<'_>,
        index: &StoredIndex,
        label: &str,
        alias: Option<&str>,
        unix_secs: u64,
    ) -> Result<SecretCollection, SecretServiceError> {
        if index.collections.len() >= self.limits.max_collections {
            return Err(SecretServiceError::Limit("collections per persona"));
        }
        let collection = Collection {
            id: chatelaine::CollectionId::from_random(crate::items::random_id_bytes()),
            source_id: None,
            title: label.to_string(),
            subtitle: None,
            created_at: Some(unix_secs),
            modified_at: Some(unix_secs),
            items: Vec::new(),
            sub_collections: Vec::new(),
        };
        let shown = project(&collection);
        transaction.update_index(|index| {
            index.collections.push(collection);
            if let Some(alias) = alias {
                index.aliases.insert(alias.to_string(), shown.id.0);
            }
            Ok(())
        })?;
        Ok(shown)
    }
}

/// The top-level collection `id` names in `index`.
pub(super) fn find(
    index: &StoredIndex,
    id: SecretCollectionId,
) -> Result<&Collection, SecretServiceError> {
    index
        .collections
        .iter()
        .find(|candidate| candidate.id == id.0)
        .ok_or(SecretServiceError::CollectionNotFound(id))
}

fn project(collection: &Collection) -> SecretCollection {
    SecretCollection {
        id: SecretCollectionId(collection.id),
        label: collection.title.clone(),
        created: collection.created_at.unwrap_or(0),
        modified: collection.modified_at.unwrap_or(0),
    }
}

fn relabel(
    transaction: &Transaction<'_>,
    id: SecretCollectionId,
    label: &str,
    unix_secs: u64,
) -> Result<SecretCollection, SecretServiceError> {
    Ok(transaction.update_index(|index| {
        let collection = index
            .collections
            .iter_mut()
            .find(|candidate| candidate.id == id.0)
            .ok_or(ItemStoreError::CollectionNotFound(id.0))?;
        collection.title = label.to_string();
        collection.modified_at = Some(unix_secs);
        Ok(project(collection))
    })?)
}

/// Every item a collection holds, nested collections included.
fn gather(collection: &Collection, members: &mut BTreeSet<ItemId>) {
    members.extend(collection.items.iter().map(|link| link.item));
    for nested in &collection.sub_collections {
        gather(nested, members);
    }
}
