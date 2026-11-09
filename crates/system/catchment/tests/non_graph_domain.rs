// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine split P4's reservoir done-condition: a domain that is not a graph
//! opens through the reservoir with nothing but the reservoir in its path.
//! This crate depends on no graph or host crate, so a test here that opens
//! Woodshed's `music` mere and Knot's operation store is that proof.
#![cfg(not(target_arch = "wasm32"))]

use catchment::{
    DomainId, MereId, ReservoirStore, mere_dir, open_mere_archive_backend, open_mere_backend,
    open_reservoir_backend,
};
use identity::PersonaId;
use muniment::{Backend, JsonSlots};
use pollster::block_on;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x6d75_7369_63))
}

/// A stand-in for a domain's own record: the reservoir never reads it.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
struct Practice {
    piece: String,
    bars: u32,
}

/// The attribution the access index carries; this domain records none.
type Store<B> = ReservoirStore<B, serde_json::Value>;

#[test]
fn a_music_mere_opens_writes_and_reopens_through_the_reservoir() {
    let root = tempfile::tempdir().unwrap();
    let music = DomainId::new("music").unwrap();
    let mere = block_on(async {
        let index = open_reservoir_backend(root.path(), persona()).unwrap();
        let mut reservoir = Store::open(index, persona()).await.unwrap();
        let (record, created) = reservoir.ensure(music.clone(), 1).await.unwrap();
        assert!(created);
        record.id
    });
    assert_eq!(mere, MereId::derive(persona(), &music));

    let practice = Practice {
        piece: "etude".into(),
        bars: 32,
    };
    block_on(async {
        let store = open_mere_backend(root.path(), persona(), mere).unwrap();
        JsonSlots::new(store)
            .save("practice/current", &practice)
            .await
            .unwrap();
    });
    assert!(mere_dir(root.path(), persona(), mere).is_dir());

    // A second process would be refused while the first holds the store; once
    // it is dropped, the same mere reopens with the record intact.
    block_on(async {
        let index = open_reservoir_backend(root.path(), persona()).unwrap();
        let reservoir = Store::open(index, persona()).await.unwrap();
        assert_eq!(reservoir.get(&music).map(|record| record.id), Some(mere));
        let store = open_mere_backend(root.path(), persona(), mere).unwrap();
        let back: Option<Practice> = JsonSlots::new(store)
            .load("practice/current")
            .await
            .unwrap();
        assert_eq!(back, Some(practice));
    });
}

#[test]
fn an_operation_store_opens_beside_its_mere_as_one_owner() {
    // Knot's store is a muniment redb backend of signed operations; the
    // reservoir opens it with the same lifetime rule as any mere's archive.
    let root = tempfile::tempdir().unwrap();
    let knot = block_on(async {
        let index = open_reservoir_backend(root.path(), persona()).unwrap();
        let mut reservoir = Store::open(index, persona()).await.unwrap();
        let (record, _) = reservoir
            .ensure(DomainId::new("knot").unwrap(), 2)
            .await
            .unwrap();
        record.id
    });
    let store = open_mere_archive_backend(root.path(), persona(), knot).unwrap();
    assert!(
        open_mere_archive_backend(root.path(), persona(), knot).is_err(),
        "an operation store has one owner"
    );
    block_on(async {
        store.put("op/0001", b"signed operation").await.unwrap();
        assert_eq!(
            store.get("op/0001").await.unwrap().as_deref(),
            Some(&b"signed operation"[..])
        );
    });
}
