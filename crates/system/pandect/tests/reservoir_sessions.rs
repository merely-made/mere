// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph domain over catchment's store lifetimes: a mere's sessions live
//! in that mere's own directory. Moved here from the reservoir module when it
//! became catchment (engine split P4), since it is the graph domain's check.
#![cfg(not(target_arch = "wasm32"))]

use kernel::graph::Author;
use pandect::manifest::PersonaId;
use pandect::{
    DomainId, MERES_DIR, MereId, MereSessions, ReservoirError, mere_dir, open_mere_backend,
    reservoir_dir,
};
use pollster::block_on;
use uuid::Uuid;

fn persona(n: u128) -> PersonaId {
    PersonaId(Uuid::from_u128(n))
}

#[test]
fn a_mere_keeps_its_sessions_in_its_own_directory() {
    let root = tempfile::tempdir().unwrap();
    let mere = MereId::derive(persona(1), &DomainId::new("divination").unwrap());
    let store = open_mere_backend(root.path(), persona(1), mere).unwrap();
    assert!(
        matches!(
            open_mere_backend(root.path(), persona(1), mere),
            Err(ReservoirError::Open { .. })
        ),
        "a mere's store has one owner"
    );
    let session = block_on(MereSessions::new(store).mint(Author::user(), None)).unwrap();
    let manifest = mere_dir(root.path(), persona(1), mere)
        .join("sessions")
        .join(session.session_id.as_uuid().to_string())
        .join("manifest.json");
    assert!(manifest.is_file(), "{} exists", manifest.display());
    assert!(manifest.starts_with(reservoir_dir(root.path(), persona(1)).join(MERES_DIR)));
}
