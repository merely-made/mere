// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `Graph::from_snapshot` — reconstruct the runtime graph from a
//! persisted [`GraphSnapshot`].
//!
//! Extracted from `graph/snapshot.rs` per the 2026-05-11 kernel
//! decomposition pass.

use euclid::default::Point2D;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::super::import_records::import_records_from_legacy_provenance;
use super::super::*;
use crate::address::{address_from_url, detect_mime};
use crate::persistence::{
    GraphSnapshot, PersistedCouplingResponse, PersistedEdge, PersistedFieldExtent,
    PersistedFieldLifecycle, PersistedNodeSelector,
};

pub(crate) use mere::model::edge_decode::{
    mark_legacy_containment_sidecars_as_derived, payload_from_persisted, payload_from_persisted_at,
    restore_persisted_payload,
};

impl Graph {
    /// Merge one persisted edge into a surface pair, retaining revision behavior.
    pub(crate) fn restore_persisted_edge(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        pedge: &PersistedEdge,
    ) {
        let existing = self.find_edge_key(from, to);
        let mut payload = existing
            .and_then(|key| self.inner.edge(key))
            .map(|e| (**e).clone())
            .unwrap_or_else(EdgePayload::new);
        let structural_changes = restore_persisted_payload(&mut payload, pedge);
        if pedge.semantic.is_some() || pedge.traversal.is_some() || !payload.is_empty() {
            match existing {
                Some(key) => *self.get_edge_mut(key).expect("existing edge") = payload,
                None => {
                    self.connect_surface(from, to, payload);
                },
            }
        }
        for _ in 0..structural_changes {
            self.bump_revision();
        }
    }
}

#[path = "from/materialize.rs"]
mod materialize;
