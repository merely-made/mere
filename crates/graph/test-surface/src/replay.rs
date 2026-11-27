// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Replay: each statement's source carried from entry to entry (engine split
//! D251), and the checks a replayed or stored graph must pass.

use std::collections::{BTreeMap, BTreeSet};

use mere::model::{Author, JournalDelta, NodeProperty, PersistedEdge, ResourceDelta};
use mere::session::{OnRefusal, ReplayRefused};
use portolan::{FacetId, FacetStore};
use uuid::Uuid;

use super::{Delta, PartKey, SURFACE_PROPERTIES, Surface, SurfaceDelta};

/// Refuse stored resource records a snapshot cannot hold: a facet whose value
/// is not JSON, or two records for one resource that differ.
pub(crate) fn check_records(resources: &[ResourceDelta]) -> Result<(), String> {
    let mut seen = BTreeMap::new();
    for delta in resources {
        let ResourceDelta::ReplaySetResourceRecordById {
            resource_id,
            record: Some(record),
        } = delta
        else {
            continue;
        };
        for facet in &record.facets {
            serde_json::from_str::<serde_json::Value>(&facet.value_json).map_err(|error| {
                format!(
                    "resource {resource_id} facet {}: invalid JSON: {error}",
                    facet.facet
                )
            })?;
        }
        if let Some(first) = seen.insert(resource_id.clone(), record)
            && first != record
        {
            return Err(format!(
                "resource {resource_id} conflicts with another record"
            ));
        }
    }
    Ok(())
}

/// What a replay carries from entry to entry: each statement handle's source,
/// which outlives the handle's removal so a later restoration keeps it.
pub type Carry = BTreeMap<String, String>;

/// `delta` with each statement that names no source given the carried one,
/// else `author`'s.
pub(crate) fn attributed(delta: &Delta, carry: &Carry, author: Option<&Author>) -> Delta {
    let mut delta = delta.clone();
    if let JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds { edges, .. }) =
        &mut delta
    {
        for row in edges
            .iter_mut()
            .flat_map(|edge| edge.assertions.iter_mut().flatten())
        {
            if row.provenance_iri.is_none() {
                row.provenance_iri = carry
                    .get(&row.statement_id)
                    .cloned()
                    .or_else(|| author.map(Author::asserter_iri));
            }
        }
    }
    delta
}

/// Remember the source of every statement `delta` writes.
pub(crate) fn remember(carry: &mut Carry, delta: &Delta) {
    if let JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds { edges, .. }) = delta
    {
        for row in edges
            .iter()
            .flat_map(|edge| edge.assertions.iter().flatten())
        {
            if let Some(source) = &row.provenance_iri {
                carry.insert(row.statement_id.clone(), source.clone());
            }
        }
    }
}

/// The edit that writes `rows` between resources `from` and `to`.
pub fn edges_edit(from: Uuid, to: Uuid, rows: Vec<PersistedEdge>) -> Delta {
    JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds {
        from_resource_id: from.to_string(),
        to_resource_id: to.to_string(),
        edges: rows,
    })
}

impl Surface {
    /// The first handle `edges` carry that another resource pair holds: one
    /// handle, one row.
    pub(crate) fn handle_held_elsewhere(
        &self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> Option<String> {
        let incoming: BTreeSet<String> = edges
            .iter()
            .flat_map(PersistedEdge::retained_assertions)
            .map(|row| row.statement_id.clone())
            .collect();
        self.resources
            .resource_edges()
            .filter(|(a, b, _)| (a.id(), b.id()) != (from, to))
            .flat_map(|(_, _, payload)| payload.assertions())
            .map(|statement| statement.statement_id.clone())
            .find(|handle| incoming.contains(handle))
    }

    /// Every resource pair holding a relation.
    pub(crate) fn pairs(&self) -> BTreeSet<(Uuid, Uuid)> {
        self.resources
            .resource_edges()
            .map(|(a, b, _)| (a.id(), b.id()))
            .collect()
    }

    /// The resource pair holding statement `handle`, held or withdrawn.
    pub fn statement(&self, handle: &str) -> Option<(Uuid, Uuid)> {
        self.resources
            .resource_edges()
            .find(|(_, _, payload)| {
                payload
                    .assertions()
                    .iter()
                    .any(|statement| statement.statement_id == handle)
            })
            .map(|(a, b, _)| (a.id(), b.id()))
    }

    pub(crate) fn replay_all<'a>(
        &mut self,
        deltas: impl IntoIterator<Item = &'a Delta>,
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused> {
        let mut carry = Carry::new();
        let entries = deltas.into_iter().map(|delta| (delta, None));
        self.replay_attributed(&mut carry, entries, on_refusal)
    }

    /// Replay `entries` quietly, giving a statement that names no source the
    /// source `carry` remembers for its handle, else its entry's author, and
    /// remembering every source written (engine split D251).
    pub(crate) fn replay_attributed<'a>(
        &mut self,
        carry: &mut Carry,
        entries: impl IntoIterator<Item = (&'a Delta, Option<&'a Author>)>,
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused> {
        let recorder = self.recorder.take();
        let mut outcome = Ok(());
        for (position, (delta, author)) in entries.into_iter().enumerate() {
            let delta = attributed(delta, carry, author);
            if let Err(reason) = self.apply_delta(&delta) {
                if on_refusal == OnRefusal::Stop {
                    outcome = Err(ReplayRefused { position, reason });
                    break;
                }
                continue;
            }
            remember(carry, &delta);
        }
        self.recorder = recorder;
        outcome
    }

    /// Refuse a handle held twice among the surfaces' own statements
    /// ([`SURFACE_PROPERTIES`]) under `facets` and the resources' literals.
    pub(crate) fn check_handles(&self, facets: &FacetStore<Uuid>) -> Result<(), String> {
        let mut held = BTreeMap::new();
        let literals = self.resources.resource_nodes().flat_map(|resource| {
            let id = resource.id();
            self.resources
                .resource_properties(id)
                .into_iter()
                .map(move |property| (format!("resource {id}"), property.statement_id))
        });
        let surfaces = facets.iter().flat_map(|(surface, node)| {
            node.get(&FacetId::new(SURFACE_PROPERTIES))
                .and_then(|value| serde_json::from_value::<Vec<NodeProperty>>(value.clone()).ok())
                .unwrap_or_default()
                .into_iter()
                .map(move |property| (format!("surface {surface}"), property.statement_id))
        });
        for (holder, handle) in literals.chain(surfaces) {
            if let Some(first) = held.insert(handle.clone(), holder.clone())
                && first != holder
            {
                return Err(format!(
                    "assertion handle {handle:?} repeats: held on {first} and again on {holder}"
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn touched(edits: &[Delta]) -> BTreeSet<PartKey> {
        edits.iter().filter_map(PartKey::of).collect()
    }

    pub(crate) fn state_of(&self, part: &PartKey) -> String {
        let value = match part {
            PartKey::Surface(id) => serde_json::to_string(&self.shown(*id)),
            PartKey::Resource(id) => serde_json::to_string(&self.resources.record_delta(*id)),
            PartKey::Pair(a, b) => serde_json::to_string(&self.resources.pair_delta(*a, *b)),
        };
        value.expect("serializable state")
    }

    pub(crate) fn restore(&self, part: &PartKey) -> Delta {
        match part {
            PartKey::Surface(id) => JournalDelta::Surface(match self.shown(*id) {
                Some(resource_id) => SurfaceDelta::ShowResource {
                    surface_id: *id,
                    resource_id,
                },
                None => SurfaceDelta::HideSurface { surface_id: *id },
            }),
            PartKey::Resource(id) => JournalDelta::Resource(self.resources.record_delta(*id)),
            PartKey::Pair(a, b) => JournalDelta::Resource(self.resources.pair_delta(*a, *b)),
        }
    }
}
