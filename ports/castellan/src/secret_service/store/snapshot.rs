// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a locked Secret Service can still answer: ruling 11's secret-free
//! snapshot of collections, labels and lookup attributes, kept in memory and
//! never written (vault lock rulings 11, 67). The snapshot type lives in
//! chatelaine (dramatis repo plan, ruling D30); the store takes it here.

use super::{MetadataSnapshot, SecretServiceError, SecretServiceStore};

impl SecretServiceStore {
    /// Take the snapshot. Refused while locked, like every metadata read.
    pub fn snapshot(&self) -> Result<MetadataSnapshot, SecretServiceError> {
        let mut snapshot = MetadataSnapshot::default();
        for collection in self.collections()? {
            let items = self.items(collection.id)?;
            snapshot.insert_collection(collection, items);
        }
        snapshot.set_aliases(self.aliases()?);
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::super::NewSecretItem;
    use super::*;
    use crate::resident::CastellanResident;
    use personae::{IdentityError, PersonaId};

    fn attributes(account: &str) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("application".into(), "turnstone".into()),
            ("account".into(), account.into()),
        ])
    }

    #[test]
    fn the_snapshot_answers_as_the_store_did_after_the_lock() {
        let dir = tempfile::tempdir().unwrap();
        let resident = CastellanResident::claim(
            dir.path().join("records"),
            [0x91; 32],
            dir.path().join("freshness"),
            [0x92; 32],
        )
        .unwrap();
        let store = resident.secret_service(PersonaId::new(), Default::default());
        let collection = store.ensure_default_collection("Castellan", 10).unwrap();
        for account in ["mark", "mayola"] {
            store
                .create_item(NewSecretItem {
                    collection: collection.id,
                    label: account.into(),
                    attributes: attributes(account),
                    secret: b"secret".to_vec(),
                    content_type: "text/plain".into(),
                    replace: false,
                    unix_secs: 11,
                })
                .unwrap();
        }
        let live_collections = store.collections().unwrap();
        let live_items = store.items(collection.id).unwrap();
        let live_search = store.search(&attributes("mark")).unwrap();
        let snapshot = store.snapshot().unwrap();

        resident.lock();
        assert!(matches!(
            store.collections(),
            Err(SecretServiceError::Storage(IdentityError::Locked))
        ));
        assert!(store.snapshot().is_err(), "no snapshot is taken while locked");

        assert_eq!(snapshot.collections(), live_collections);
        assert_eq!(snapshot.collection(collection.id).unwrap(), live_collections[0]);
        assert_eq!(snapshot.items(collection.id).unwrap(), live_items);
        assert_eq!(snapshot.search(&attributes("mark")), live_search);
        assert_eq!(snapshot.search(&BTreeMap::new()).len(), 2);
        assert_eq!(snapshot.read_alias("default"), Some(collection.id));
        assert_eq!(snapshot.item(live_items[0].id).unwrap(), live_items[0]);
    }
}
