// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Rebuildable extraction inputs, separate from graph truth and replay.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::{Graph, ResourceNode, SemanticStatementSpec};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum PendingLinkRetention {
    #[default]
    SessionOnly,
    UntilPurged,
}

/// Stable source ownership and optional original appearance context.
/// No node indices, source documents or unasserted target records are retained.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLink {
    pub source_resource: Uuid,
    pub source_surface: Option<Uuid>,
    pub target_iri: String,
    pub statement: SemanticStatementSpec,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingLinkState {
    pub retention: PendingLinkRetention,
    pub entries: Vec<PendingLink>,
}

/// A clone is a new live graph, not an escape from a replay/import scope.
#[derive(Debug, Default)]
pub(crate) struct Suppression(pub(crate) bool);

impl Clone for Suppression {
    fn clone(&self) -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PendingLinkRetry {
    pub asserted: usize,
    pub already_held: usize,
    pub remaining: usize,
}

impl Graph {
    pub fn pending_links(&self) -> &[PendingLink] {
        &self.pending_link_state.entries
    }

    pub fn pending_link_state(&self) -> &PendingLinkState {
        &self.pending_link_state
    }

    /// The durable representation excludes the default session-only entries.
    pub fn retained_pending_link_state(&self) -> PendingLinkState {
        let mut state = self.pending_link_state.clone();
        if state.retention == PendingLinkRetention::SessionOnly {
            state.entries.clear();
        }
        state
    }

    pub fn set_pending_link_retention(&mut self, retention: PendingLinkRetention) {
        if self.pending_link_state.retention != retention {
            self.pending_link_state.retention = retention;
            self.semantic_observation_revision += 1;
        }
    }

    /// Install a separate cache slot after truth replay, without deriving claims.
    pub fn restore_pending_link_state(&mut self, state: PendingLinkState) {
        if self.pending_link_state != state {
            self.pending_link_state = state;
            self.semantic_observation_revision += 1;
        }
    }

    pub fn queue_pending_link(&mut self, mut link: PendingLink) -> bool {
        if link.statement.provenance_iri.is_none() {
            link.statement.provenance_iri = Some(self.write_author().asserter_iri());
        }
        link.target_iri = ResourceNode::new(&link.target_iri).canonical_iri().into();
        if self.pending_link_state.entries.contains(&link) {
            return false;
        }
        self.pending_link_state.entries.push(link);
        self.semantic_observation_revision += 1;
        true
    }

    /// Purging changes no snapshot, source document, assertion or journal entry.
    pub fn purge_pending_links(&mut self) -> usize {
        let count = self.pending_link_state.entries.len();
        if count != 0 {
            self.pending_link_state.entries.clear();
            self.semantic_observation_revision += 1;
        }
        count
    }

    pub fn semantic_observation_revision(&self) -> u64 {
        self.semantic_observation_revision
    }

    /// Suppress ambient extraction through a complete replay or import batch.
    /// The previous scope is restored on unwind; clones do not inherit it.
    pub fn without_pending_derivation<R>(&mut self, edit: impl FnOnce(&mut Self) -> R) -> R {
        let previous = self.pending_derivation_suppressed.0;
        self.pending_derivation_suppressed.0 = true;
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| edit(self)));
        self.pending_derivation_suppressed.0 = previous;
        match result {
            Ok(result) => result,
            Err(panic) => std::panic::resume_unwind(panic),
        }
    }

    /// Retry only after completed live admission. Recorded assertions remain
    /// authoritative: a carried held claim consumes its cache input unchanged.
    pub fn retry_pending_links(&mut self) -> PendingLinkRetry {
        if self.pending_derivation_suppressed.0 {
            return PendingLinkRetry {
                remaining: self.pending_links().len(),
                ..Default::default()
            };
        }
        let entries = self.pending_link_state.entries.clone();
        let mut remaining = Vec::new();
        let mut result = PendingLinkRetry::default();
        for link in entries {
            let target = ResourceNode::new(&link.target_iri).id();
            if self.resource(link.source_resource).is_none() || self.resource(target).is_none() {
                remaining.push(link);
                continue;
            }
            match self.held_statement_by_resource_ids(link.source_resource, target, &link.statement)
            {
                Ok(Some(_)) => {
                    result.already_held += 1;
                    continue;
                },
                Err(_) => {
                    remaining.push(link);
                    continue;
                },
                Ok(None) => {},
            }
            match self.try_assert_semantic_statement_by_resource_ids(
                link.source_resource,
                target,
                link.statement.clone(),
            ) {
                Ok((_, assertion)) => result.asserted += usize::from(assertion.changed),
                Err(super::StatementWriteError::SurfaceContextRequired) => {
                    let source = link
                        .source_surface
                        .and_then(|id| self.get_node_key_by_id(id))
                        .filter(|&key| self.shown_resource_id(key) == Some(link.source_resource));
                    let targets = self.surface_ids_showing_resource(target);
                    let target_surface = (targets.len() == 1)
                        .then(|| self.get_node_key_by_id(targets[0]))
                        .flatten();
                    let asserted = source.zip(target_surface).and_then(|(source, target)| {
                        self.try_assert_semantic_statement(source, target, link.statement.clone())
                            .ok()
                    });
                    if let Some((_, assertion)) = asserted {
                        result.asserted += usize::from(assertion.changed);
                    } else {
                        remaining.push(link);
                    }
                },
                Err(_) => remaining.push(link),
            }
        }
        if remaining != self.pending_link_state.entries {
            self.pending_link_state.entries = remaining;
            self.semantic_observation_revision += 1;
        }
        result.remaining = self.pending_links().len();
        result
    }
}
