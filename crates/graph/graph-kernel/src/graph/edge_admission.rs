// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Replayed edge sets meet the handle rule of `validate_active_resource_assertion_handles`
//! before they replace what a pair holds. A legacy migration passes through states in
//! which both strata hold a handle, so it stages the check instead of skipping it.

use super::Graph;
use super::resource_content::{ContentError, PendingWrite};
use crate::persistence::PersistedEdge;
use std::collections::BTreeSet;
use uuid::Uuid;

/// Set while a legacy migration stages its edge moves. A clone is a new live
/// graph, not an escape from the scope.
#[derive(Debug, Default)]
pub(crate) struct StagedEdges(bool);

impl Clone for StagedEdges {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl Graph {
    /// Begin the staged scope: until [`Self::settle_staged_edges`], replayed edge
    /// sets are stored without the handle check. Only legacy migration stages.
    pub(crate) fn stage_edges(&mut self) {
        self.staged_edges.0 = true;
    }

    /// End the staged scope and check the whole graph once, as checked open does.
    pub(crate) fn settle_staged_edges(&mut self) -> Result<(), ContentError> {
        self.staged_edges.0 = false;
        self.validate_active_resource_assertion_handles()
    }

    /// Whether storing `edges` as every Resource edge from `from` to `to` leaves
    /// the graph passing checked open. A set the write refuses anyway stores nothing.
    pub(crate) fn admit_resource_edges(
        &self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> Result<(), ContentError> {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return Ok(());
        };
        if self.staged_edges.0 || !resource_rows_wellformed(from, to, edges) {
            return Ok(());
        }
        let held = self
            .resources
            .inner()
            .edges_connecting(source, target)
            .flat_map(|edge| edge.weight().assertions())
            .map(|statement| statement.statement_id.as_str())
            .collect();
        let rows = row_handles(edges);
        if !claims_new_handle(&held, &rows) {
            return Ok(());
        }
        self.validate_handles_after(Some(PendingWrite::ResourceEdges(from, to, &rows)))
    }

    /// The same for the Surface edges between two nodes, by node id.
    pub(crate) fn admit_surface_edges(
        &self,
        from: Uuid,
        to: Uuid,
        edges: &[PersistedEdge],
    ) -> Result<(), ContentError> {
        let (Some(source), Some(target)) =
            (self.get_node_key_by_id(from), self.get_node_key_by_id(to))
        else {
            return Ok(());
        };
        if self.staged_edges.0 {
            return Ok(());
        }
        let held = self
            .inner
            .inner()
            .edges_connecting(source, target)
            .flat_map(|edge| edge.weight().assertions())
            .map(|statement| statement.statement_id.as_str())
            .collect();
        let rows = row_handles(edges);
        if !claims_new_handle(&held, &rows) {
            return Ok(());
        }
        self.validate_handles_after(Some(PendingWrite::SurfaceEdges(from, to, &rows)))
    }
}

/// The handles the replacement rows store, a row's repeats collapsing as they do on storage.
fn row_handles(edges: &[PersistedEdge]) -> Vec<String> {
    edges
        .iter()
        .flat_map(|edge| {
            let mut seen = BTreeSet::new();
            edge.retained_assertions()
                .iter()
                .filter(move |statement| seen.insert(statement.statement_id.as_str()))
                .map(|statement| statement.statement_id.clone())
        })
        .collect()
}

/// Whether `rows` put a handle on a row the pair did not already hold it on. A
/// restatement adds none, so it cannot collide and skips the scan.
fn claims_new_handle(held: &BTreeSet<&str>, rows: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    rows.iter()
        .any(|id| !seen.insert(id.as_str()) || !held.contains(id.as_str()))
}

/// Whether `edges` run exactly from `from` to `to` and none is a bare aggregate.
pub(super) fn resource_rows_wellformed(from: Uuid, to: Uuid, edges: &[PersistedEdge]) -> bool {
    edges
        .iter()
        .all(|edge| edge.from_node_id == from.to_string() && edge.to_node_id == to.to_string())
        && !edges
            .iter()
            .filter_map(|edge| edge.semantic.as_ref())
            .any(|semantic| {
                semantic.statements.is_empty()
                    && (!semantic.sub_kinds.is_empty() || semantic.predicate.is_some())
            })
}
