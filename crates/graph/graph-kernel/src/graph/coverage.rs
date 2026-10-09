// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Known limits of the supplied graph, not a claim of world completeness.

use super::{Graph, ResourceNode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLayer {
    Possession,
    Residency,
    Disclosure,
    Synchronization,
    Projection,
}

impl CoverageLayer {
    pub const ALL: [Self; 5] = [
        Self::Possession,
        Self::Residency,
        Self::Disclosure,
        Self::Synchronization,
        Self::Projection,
    ];
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageLimit {
    pub layer: CoverageLayer,
    pub reason: String,
    /// Empty scope means the entire supplied graph. Hosts own these observations.
    #[serde(default)]
    pub resources: Vec<Uuid>,
    /// None means the number beyond this boundary is unknown.
    #[serde(default)]
    pub count: Option<usize>,
}

impl CoverageLimit {
    pub fn new(layer: CoverageLayer, reason: impl Into<String>) -> Self {
        Self {
            layer,
            reason: reason.into(),
            resources: vec![],
            count: None,
        }
    }
}

/// An empty note means no known limit within this supplied graph.
/// It never means all relevant data exists here or anywhere else.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoverageNote {
    pub limits: Vec<CoverageLimit>,
}

impl CoverageNote {
    pub fn push(&mut self, limit: CoverageLimit) {
        if !self.limits.contains(&limit) {
            self.limits.push(limit);
        }
    }
    pub fn add_count(&mut self, layer: CoverageLayer, reason: impl Into<String>, count: usize) {
        if count == 0 {
            return;
        }
        let mut limit = CoverageLimit::new(layer, reason);
        limit.count = Some(count);
        self.push(limit);
    }
}

impl Graph {
    pub fn known_coverage(&self) -> &CoverageNote {
        &self.known_coverage
    }

    /// Runtime host context, never graph truth, a journal edit or a geometry key.
    pub fn set_known_coverage(&mut self, note: CoverageNote) {
        if self.known_coverage != note {
            self.known_coverage = note;
            self.semantic_observation_revision += 1;
        }
    }

    pub fn coverage_note(&self) -> CoverageNote {
        let mut note = self.known_coverage.clone();
        let mut unavailable = 0;
        let mut unasserted = 0;
        for link in self.pending_links() {
            let missing: Vec<_> = [
                link.source_resource,
                ResourceNode::new(&link.target_iri).id(),
            ]
            .into_iter()
            .filter(|id| self.resource(*id).is_none())
            .collect();
            if missing.is_empty() {
                unasserted += 1;
                continue;
            }
            if missing.iter().any(|id| {
                !self.known_coverage.limits.iter().any(|limit| {
                    limit.layer == CoverageLayer::Residency
                        && (limit.resources.is_empty() || limit.resources.contains(id))
                })
            }) {
                unavailable += 1;
            }
        }
        // Aggregate counts carry no private target IRIs or derived target ids.
        note.add_count(
            CoverageLayer::Possession,
            "pending source claims have unavailable endpoints",
            unavailable,
        );
        note.add_count(
            CoverageLayer::Possession,
            "pending source claims await assertion placement",
            unasserted,
        );
        note
    }

    pub fn copy_semantic_context_from(&mut self, graph: &Self) {
        self.restore_pending_link_state(graph.pending_link_state.clone());
        self.set_known_coverage(graph.known_coverage.clone());
    }

    /// History starts from truth only, never an ambient extraction input.
    pub fn clear_semantic_context(&mut self) {
        self.restore_pending_link_state(Default::default());
        self.set_known_coverage(Default::default());
    }
}
