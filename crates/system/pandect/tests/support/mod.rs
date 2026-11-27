// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mere's shared test graph (engine split D245, D272) with pandect's hooks for
//! it (D257): a publication that names the surface's snapshot by its hash.
//! The lib tests include this file too, where `pandect` names the crate itself.

// Each test binary uses part of the surface.
#![allow(dead_code)]

pub use mere_test_surface::*;

#[cfg(all(feature = "personal-sync", not(target_arch = "wasm32")))]
mod sync;
#[cfg(all(feature = "personal-sync", not(target_arch = "wasm32")))]
#[allow(unused_imports)]
pub use sync::StatementFold;

use mere::session::SessionGraph;
use muniment::{Backend, WriteOp};
use pandect::SessionError;
use pandect::graph_codicil::CodicilHook;
use pandect::graph_session::{Publication, PublicationHook, PublishedGeneration, SessionBackend};
use uuid::Uuid;

/// A surface the publication hook refuses to publish while it shows anything.
pub const UNPUBLISHABLE: Uuid = Uuid::from_u128(0xdead);

/// Where the surface's publication keeps its head, per container.
pub fn head_key(container: Uuid) -> String {
    format!("surfaces/{container}/head")
}

/// The generation the surface publishes as: its snapshot's hash.
pub fn generation_of(graph: &Surface) -> String {
    let bytes = serde_json::to_vec(&graph.to_snapshot()).expect("serializable snapshot");
    blake3::hash(&bytes).to_hex().to_string()
}

fn published(backend: SessionBackend<'_>, container: Uuid) -> PublishedGeneration<'_> {
    Box::pin(async move {
        let head = backend
            .get(&head_key(container))
            .await?
            .ok_or_else(|| SessionError::Corrupt("no surface publication".into()))?;
        String::from_utf8(head).map_err(|error| SessionError::Corrupt(error.to_string()))
    })
}

/// The surface's publication hook: one head write naming the snapshot.
pub static PUBLICATION: PublicationHook<Surface> = PublicationHook {
    check: |graph| match graph.shown(UNPUBLISHABLE) {
        Some(_) => Err(SessionError::Validation(
            "surface refuses publication".into(),
        )),
        None => Ok(()),
    },
    prepare: |graph, _backend, container| {
        let generation = generation_of(graph);
        let (key, value) = (head_key(container), generation.clone().into_bytes());
        Ok(Publication {
            generation,
            writes: Box::new(move |_reader| vec![WriteOp::Put { key, value }]),
        })
    },
    revision: |graph, _backend, _container| Ok(generation_of(graph)),
    published,
};

/// The surface's codicil hook: its own schemas, and no redaction (D265).
pub static CODICIL: CodicilHook<Surface> = CodicilHook {
    graph_schema: b"p8.surface-snapshot/v1",
    profiled_schema: b"p8.surface-snapshot-profiled/v1",
    v3_profiled_schema: b"p8.surface-snapshot/v0-profiled",
    v2_schema: b"p8.surface-snapshot/v0-facets",
    v1_schema: b"p8.surface-snapshot/v0",
    redact: |_, _| {},
};
