// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use crate::MemberId;
use serde::{Deserialize, Serialize};

/// Subgraph identity. Lightweight integer index within a single GraphTree.
pub type SubgraphId = u32;

/// A subgraph is a connected sub-structure within the GraphTree.
/// Multiple subgraphs exist in a graph view — like document groups
/// in a folder. Each tracks its own binding and anchor state.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct SubgraphRef<N: MemberId> {
    pub id: SubgraphId,
    pub anchors: Vec<N>,
    pub primary_anchor: Option<N>,
    pub binding: SubgraphBinding,
    pub kind: Option<SubgraphKind>,
}

impl<N: MemberId> SubgraphRef<N> {
    pub fn new_session(id: SubgraphId) -> Self {
        Self {
            id,
            anchors: Vec::new(),
            primary_anchor: None,
            binding: SubgraphBinding::UnlinkedSession,
            kind: Some(SubgraphKind::Session),
        }
    }

    pub fn with_anchor(mut self, anchor: N) -> Self {
        self.primary_anchor = Some(anchor.clone());
        self.anchors.push(anchor);
        self
    }

    pub fn with_kind(mut self, kind: SubgraphKind) -> Self {
        self.kind = Some(kind);
        self
    }
}

/// How a tile group binds to a subgraph definition.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SubgraphBinding {
    /// No link to a canonical subgraph definition. Pure session grouping.
    UnlinkedSession,
    /// Linked to a canonical subgraph spec. Roster updates from graph.
    Linked { spec: SubgraphSpec },
    /// Was linked, but a user override created a local divergence — the tear-out
    /// **branch** operation (a new grouping inside the donor's own graph). Named
    /// `Branched`, not `Forked`, to stay distinct from the host's **fork** (a new
    /// session + graph), which is the opposite operation a layer up. (See the tear-out
    /// operations brief §4.2.)
    Branched {
        parent_spec: SubgraphSpec,
        reason: String,
    },
}

/// Canonical subgraph specification (referenced by Linked bindings).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubgraphSpec {
    pub kind: SubgraphKind,
    pub anchors: Vec<String>,
    pub primary_anchor: Option<String>,
    pub selectors: Vec<String>,
}

/// The 9 canonical subgraph shapes from `subgraph_model.md`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubgraphKind {
    Ego { radius: u8 },
    Corridor,
    Component,
    Loop,
    Frontier,
    Facet,
    Session,
    Bridge,
    WorkbenchCorrespondence,
}

// ---------------------------------------------------------------------------
// Edge projection spec (consumed from subgraph_projection_binding_spec.md §3)
// ---------------------------------------------------------------------------

/// Where the active edge projection originates.
///
/// See `subgraph_projection_binding_spec.md §3.1` for the canonical shape.
/// The `graph_view_id` and `graph_id` fields are carried as opaque strings
/// because the forme crate has no dependency on Graphshell's ID types.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectionSource {
    /// Graph-level default projection.
    GraphDefault { graph_id: String },
    /// Override scoped to a single graph view.
    GraphViewOverride { graph_view_id: String },
    /// Override scoped to a specific selection within a view.
    SelectionOverride {
        graph_view_id: String,
        seed_nodes: Vec<String>,
    },
}

/// Which edges contribute to subgraph derivation.
///
/// This is the tree-side carrier for the binding spec's `EdgeProjectionSpec`.
/// Selectors are opaque strings because the tree crate doesn't own the
/// relation-selector vocabulary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeProjectionSpec {
    pub selectors: Vec<String>,
    pub source: ProjectionSource,
}

// ---------------------------------------------------------------------------
// Reconciliation types (subgraph_projection_binding_spec.md §7 + §11)
// ---------------------------------------------------------------------------

/// Difference between a linked subgraph's expected member set and the tree's
/// current member set for that subgraph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SubgraphMemberDelta<N: MemberId> {
    /// Members present in graph truth but absent from the tree.
    pub added: Vec<N>,
    /// Members present in the tree but absent from graph truth.
    pub removed: Vec<N>,
    /// Seed nodes that were rebased (still present but re-anchored).
    pub rebased_seeds: Vec<N>,
}

impl<N: MemberId> SubgraphMemberDelta<N> {
    pub fn empty() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
            rebased_seeds: Vec::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.added.is_empty() && self.removed.is_empty() && self.rebased_seeds.is_empty()
    }
}

/// Proposal produced by reconciliation for the host to present to the user.
///
/// See `subgraph_projection_binding_spec.md §7.1` for the four choices.
#[derive(Clone, Debug)]
pub struct ReconciliationProposal<N: MemberId> {
    pub subgraph_id: SubgraphId,
    pub delta: SubgraphMemberDelta<N>,
    pub reason: String,
}

/// Outcome chosen by the user or auto-applied by policy.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReconciliationChoice {
    /// Commit the delta and keep the binding linked.
    ApplyKeepLinked,
    /// Preserve the current tree roster; convert to unlinked session.
    KeepAsUnlinkedSession,
    /// Fork a new subgraph from the parent.
    SaveAsNewFork { reason: String },
    /// Discard the pending change; restore the last synced roster.
    Cancel,
}
