// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What merging one session into another needs from its graph (engine split
//! D264): parts and their values, the nodes a part names and how a picked
//! node was copied, and the edits a merge writes. A sub-trait, so a host
//! implements [`SessionGraph`] without it and merges once it does.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::Value;
use uuid::Uuid;

use super::SessionGraph;

/// The kinds of part a merge treats apart from the rest.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergePart {
    /// A node as a whole: made or removed.
    Node(Uuid),
    /// Every relation from the first node to the second.
    Edges(Uuid, Uuid),
    /// One facet of a resource record, which a merge rule may join.
    ResourceFacet(Uuid, String),
    /// Any other part.
    Other,
}

/// What a set of edits names, as a merge reads it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MergeTouch {
    pub nodes: BTreeSet<Uuid>,
    pub edges: BTreeSet<(Uuid, Uuid)>,
    pub shown_surfaces: BTreeSet<Uuid>,
    /// Whether an edit writes session-level state rather than nodes.
    pub session_level: bool,
}

/// The graph a session merges over.
pub trait MergeGraph: SessionGraph<Part: Ord + Serialize + DeserializeOwned> {
    // Parts.

    /// The part `edit` sets, or `None` for an edit a merge never writes.
    fn part_of(edit: &Self::Delta) -> Option<Self::Part>;
    /// The node a node-scoped part belongs to.
    fn part_node(part: &Self::Part) -> Option<Uuid>;
    fn part_kind(part: &Self::Part) -> MergePart;
    /// What `part` holds here, or `None` when the graph does not hold it.
    fn part_value(&self, part: &Self::Part) -> Option<Value>;
    /// The edits that make `part` hold `value` (`None`: absent).
    fn part_edits(
        &self,
        part: &Self::Part,
        value: Option<&Value>,
    ) -> Result<Vec<Self::Delta>, String>;

    // Nodes.

    fn node_ids(&self) -> BTreeSet<Uuid>;
    fn has_node(&self, id: Uuid) -> bool;
    /// The node node `id` was copied from into container `container`.
    fn copied_from(&self, id: Uuid, container: &str) -> Option<Uuid>;
    /// The edits that make node `id` again, as this graph holds it.
    fn recreate_node_edits(&self, id: Uuid) -> Option<Vec<Self::Delta>>;
    /// The edits that set every relation between each of `nodes` and every
    /// node here, either way round.
    fn incident_edge_edits(&self, nodes: &[Uuid]) -> Vec<Self::Delta>;

    // Edits.

    /// The edit recording that `node` was merged from `origin` in
    /// `container`, or `None` for a graph that records no derivations.
    fn merged_from_edit(node: Uuid, origin: Uuid, container: Option<String>)
    -> Option<Self::Delta>;
    /// `edit` with the ids `names` maps rewritten.
    fn remap_ids(edit: &Self::Delta, names: &BTreeMap<String, String>) -> Self::Delta;
    fn touched(edits: &[Self::Delta]) -> MergeTouch;
    /// Replay `edits` unattributed and quietly, dropping a refused one: the
    /// scratch replays a merge compares against.
    fn replay_scratch(&mut self, edits: &[Self::Delta]);
}
