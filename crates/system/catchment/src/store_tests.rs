// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::path::Path;

use muniment::{MemoryBackend, StoreError};
use pollster::block_on;
use serde_json::{Value, json};
use uuid::Uuid;

use super::*;
use crate::catalog::MAX_DOMAIN_LEN;
#[cfg(not(target_arch = "wasm32"))]
use crate::paths::open_reservoir_backend;
use crate::paths::reservoir_dir;

/// The attribution the reservoir carries but never reads; tests use JSON.
type Store<B> = ReservoirStore<B, Value>;

fn persona(n: u128) -> PersonaId {
    PersonaId(Uuid::from_u128(n))
}

fn domain(id: &str) -> DomainId {
    DomainId::new(id).unwrap()
}

#[test]
fn domain_ids_follow_the_naming_rule() {
    for ok in ["divination", "browsing", "notes.seeds", "rng-2", "a", "9"] {
        assert!(DomainId::new(ok).is_ok(), "{ok} should be valid");
    }
    let too_long = "a".repeat(MAX_DOMAIN_LEN + 1);
    for bad in [
        "",
        "Divination",
        "-lead",
        "trail.",
        "has space",
        "under_score",
        too_long.as_str(),
    ] {
        assert!(DomainId::new(bad).is_err(), "{bad:?} should be refused");
    }
}

#[test]
fn derived_ids_are_stable_and_distinct() {
    let a = MereId::derive(persona(1), &domain("divination"));
    assert_eq!(a, MereId::derive(persona(1), &domain("divination")));
    assert_ne!(a, MereId::derive(persona(2), &domain("divination")));
    assert_ne!(a, MereId::derive(persona(1), &domain("browsing")));
    assert_eq!(a.as_uuid().get_version_num(), 5);
}

#[test]
fn a_domain_has_exactly_one_mere() {
    block_on(async {
        let mut store = Store::open(MemoryBackend::new(), persona(1)).await.unwrap();
        let (first, created) = store.ensure(domain("divination"), 10).await.unwrap();
        assert!(created);
        let (again, created) = store.ensure(domain("divination"), 20).await.unwrap();
        assert!(!created);
        assert_eq!(first, again);
        assert_eq!(again.created_at_ms, 10);
        assert_eq!(store.meres().count(), 1);
        assert_eq!(store.get_by_id(first.id), Some(&first));
    });
}

#[test]
fn the_index_survives_reopening_the_backend() {
    block_on(async {
        let backend = MemoryBackend::new();
        let mut store = Store::open(backend.clone(), persona(1)).await.unwrap();
        store.ensure(domain("divination"), 1).await.unwrap();
        store.ensure(domain("browsing"), 2).await.unwrap();
        drop(store);

        let reopened = Store::open(backend, persona(1)).await.unwrap();
        let domains: Vec<_> = reopened
            .meres()
            .map(|m| m.domain.as_str().to_string())
            .collect();
        assert_eq!(domains, ["browsing", "divination"]);
    });
}

#[test]
fn another_persona_cannot_open_the_index() {
    block_on(async {
        let backend = MemoryBackend::new();
        let mut store = Store::open(backend.clone(), persona(1)).await.unwrap();
        store.ensure(domain("divination"), 1).await.unwrap();
        let refused = Store::open(backend, persona(2)).await;
        assert!(matches!(
            refused,
            Err(ReservoirError::PersonaMismatch { .. })
        ));
    });
}

#[test]
fn a_tampered_record_is_refused() {
    block_on(async {
        let backend = MemoryBackend::new();
        let mut store = Store::open(backend.clone(), persona(1)).await.unwrap();
        store.ensure(domain("divination"), 1).await.unwrap();
        let slots = JsonSlots::new(backend.clone());
        let mut index: ReservoirIndex<Value> =
            slots.load(RESERVOIR_INDEX_SLOT).await.unwrap().unwrap();
        index.meres.get_mut(&domain("divination")).unwrap().id = MereId(Uuid::from_u128(7));
        slots.save(RESERVOIR_INDEX_SLOT, &index).await.unwrap();
        let refused = Store::open(backend, persona(1)).await;
        assert!(matches!(refused, Err(ReservoirError::Corrupt(_))));
    });
}

#[test]
fn the_reservoir_sits_beside_the_persona() {
    let root = Path::new("root");
    let dir = reservoir_dir(root, persona(1));
    assert_eq!(
        dir,
        root.join("personas")
            .join(Uuid::from_u128(1).to_string())
            .join("reservoir")
    );
}

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn a_durable_reservoir_reopens_and_refuses_a_second_owner() {
    let root = tempfile::tempdir().unwrap();
    let owner = open_reservoir_backend(root.path(), persona(1)).unwrap();
    let second = open_reservoir_backend(root.path(), persona(1));
    assert!(
        matches!(second, Err(ReservoirError::Open { .. })),
        "a second owner must be refused while the first holds the lock"
    );
    block_on(async {
        let mut store = Store::open(owner, persona(1)).await.unwrap();
        store.ensure(domain("divination"), 5).await.unwrap();
    });
    // The first owner's store dropped above; the lock is free again.
    let reopened = open_reservoir_backend(root.path(), persona(1)).unwrap();
    block_on(async {
        let store = Store::open(reopened, persona(1)).await.unwrap();
        assert_eq!(
            store.get(&domain("divination")).map(|m| m.created_at_ms),
            Some(5)
        );
    });
}

#[derive(Clone)]
struct Fallible {
    memory: MemoryBackend,
    fail: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[async_trait::async_trait]
impl Backend for Fallible {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.memory.get(key).await
    }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        if self.fail.load(std::sync::atomic::Ordering::SeqCst) {
            return Err(StoreError::Backend("injected grant write failure".into()));
        }
        self.memory.put(key, bytes).await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.memory.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.memory.list(prefix).await
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.memory.scan(start, end).await
    }
    async fn apply(&self, ops: &[muniment::WriteOp]) -> Result<(), StoreError> {
        self.memory.apply(ops).await
    }
}

#[test]
fn access_defaults_migrate_and_failed_writes_change_no_decision() {
    block_on(async {
        let backend = Fallible {
            memory: MemoryBackend::new(),
            fail: Default::default(),
        };
        let mut store = Store::open(backend.clone(), persona(1)).await.unwrap();
        let (mere, _) = store.ensure(domain("divination"), 1).await.unwrap();
        // The v1 index has no access field when no decisions exist.
        let raw = backend.get(RESERVOIR_INDEX_SLOT).await.unwrap().unwrap();
        assert!(!String::from_utf8(raw).unwrap().contains("access"));
        drop(store);
        let mut store = Store::open(backend.clone(), persona(1)).await.unwrap();
        assert!(store.application_access(mere.id, "cleromancy").is_none());
        let recorded = MereApplicationAccess {
            denied: false,
            ambient: true,
            recorded_at_ms: 2,
            author: json!({
                "kind": "person",
                "id": persona(1).as_uuid().to_string(),
                "version": null,
                "via": "turnstone",
            }),
        };
        store
            .set_application_access(mere.id, "cleromancy".into(), recorded.clone())
            .await
            .unwrap();
        backend
            .fail
            .store(true, std::sync::atomic::Ordering::SeqCst);
        let mut denied = recorded.clone();
        denied.denied = true;
        assert!(
            store
                .set_application_access(mere.id, "cleromancy".into(), denied)
                .await
                .is_err()
        );
        assert_eq!(
            store.application_access(mere.id, "cleromancy"),
            Some(&recorded)
        );
        drop(store);
        let reopened = Store::open(backend.clone(), persona(1)).await.unwrap();
        assert_eq!(
            reopened.application_access(mere.id, "cleromancy"),
            Some(&recorded)
        );
    });
}
