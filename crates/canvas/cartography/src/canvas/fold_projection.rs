// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Root-canvas projection of one view-local [`incipit::curation::FoldRecord`].
//!
//! [`project_fold`] is the native check (mer3ly site plan, Ruling 166): a
//! record projects only when it is current and every member resolves against
//! the live graph, and its projection carries the boundary accounting. The
//! canvas draws a [`FoldStandIn::Summary`] as the synthetic summary body and a
//! [`FoldStandIn::Member`] as that member with a "+N" badge
//! ([`super::folds`]). Members are the graph's node UUIDs as strings,
//! converted here at the canvas's edge (Ruling 163).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use incipit::curation::{FoldDirection, FoldId, FoldRecord, FoldStandIn};
use kernel::geometry::PortablePoint;
use kernel::graph::{EdgeFamily, Graph, NodeKey};

/// The synthetic summary body's radius in Canvas world pixels. It deliberately
/// reads larger than an ordinary node because it stands for several members.
pub(crate) const FOLD_SUMMARY_RADIUS: f32 = 30.0;

/// The family a collapse follows unless the person chooses another (Ruling
/// 165): the hierarchy family.
pub const DEFAULT_COLLAPSE_FAMILY: EdgeFamily = EdgeFamily::Containment;

/// Whether a boundary cell enters or leaves a folded summary.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FoldBoundaryDirection {
    Incoming,
    Outgoing,
}

/// The chosen direction through an explicitly selected hierarchy family when a
/// Canvas collapses a root with its descendants or ancestors.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FoldTraversalDirection {
    Outgoing,
    Incoming,
}

impl From<FoldTraversalDirection> for FoldDirection {
    fn from(direction: FoldTraversalDirection) -> Self {
        match direction {
            FoldTraversalDirection::Outgoing => Self::Outgoing,
            FoldTraversalDirection::Incoming => Self::Incoming,
        }
    }
}

impl From<FoldDirection> for FoldTraversalDirection {
    fn from(direction: FoldDirection) -> Self {
        match direction {
            FoldDirection::Outgoing => Self::Outgoing,
            FoldDirection::Incoming => Self::Incoming,
        }
    }
}

/// Every relation family, in the order a chooser offers them.
pub const EDGE_FAMILIES: [EdgeFamily; 6] = [
    EdgeFamily::Containment,
    EdgeFamily::Semantic,
    EdgeFamily::Traversal,
    EdgeFamily::Arrangement,
    EdgeFamily::Imported,
    EdgeFamily::Provenance,
];

/// A family's name as a fold rule records it: the name served scenes give
/// their relations' kind.
pub fn edge_family_name(family: EdgeFamily) -> &'static str {
    match family {
        EdgeFamily::Semantic => "semantic",
        EdgeFamily::Traversal => "traversal",
        EdgeFamily::Containment => "containment",
        EdgeFamily::Arrangement => "arrangement",
        EdgeFamily::Imported => "imported",
        EdgeFamily::Provenance => "provenance",
    }
}

/// The family a recorded name names.
pub fn edge_family_from_name(name: &str) -> Option<EdgeFamily> {
    EDGE_FAMILIES
        .into_iter()
        .find(|family| edge_family_name(*family) == name)
}

/// Cells crossing a fold boundary, bundled only by their outside endpoint,
/// direction, and relation family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldBoundaryBundle {
    pub outside: NodeKey,
    pub direction: FoldBoundaryDirection,
    pub family: EdgeFamily,
    pub count: usize,
}

/// The source accounting a Canvas fold renders.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldProjection {
    pub fold_id: FoldId,
    /// Every source node the fold covers, a member stand-in included.
    pub members: BTreeSet<NodeKey>,
    /// The member drawn in the fold's place; `None` for a summary.
    pub stand_in: Option<NodeKey>,
    /// Source nodes outside the fold.
    pub visible_members: Vec<NodeKey>,
    /// Relation cells with both endpoints inside the fold.
    pub internal_relation_count: usize,
    /// Relation cells crossing the boundary, exact after family bundling.
    pub boundary_bundles: Vec<FoldBoundaryBundle>,
    /// The host's own words for the fold.
    pub label: Option<String>,
}

impl FoldProjection {
    /// The summary lives at the centroid of its members' existing layout
    /// positions. Folding never moves or re-seeds those source members, so an
    /// expand restores their stable placement exactly.
    pub fn summary_center(
        &self,
        mut position_of: impl FnMut(NodeKey) -> Option<PortablePoint>,
    ) -> Option<PortablePoint> {
        let mut total = PortablePoint::zero();
        for &member in &self.members {
            let position = position_of(member)?;
            total.x += position.x;
            total.y += position.y;
        }
        let count = self.members.len() as f32;
        Some(PortablePoint::new(total.x / count, total.y / count))
    }

    /// Whether the fold hides `key`: a member that is not its stand-in.
    pub fn hides(&self, key: NodeKey) -> bool {
        self.stand_in != Some(key) && self.members.contains(&key)
    }

    /// The members the fold hides.
    pub fn hidden(&self) -> impl Iterator<Item = NodeKey> + '_ {
        self.members.iter().copied().filter(|key| self.hides(*key))
    }

    /// The "+N" the stand-in carries: how many members it hides.
    pub fn hidden_count(&self) -> usize {
        self.members.len() - usize::from(self.stand_in.is_some())
    }

    /// The badge text, `+N`.
    pub fn badge(&self) -> String {
        format!("+{}", self.hidden_count())
    }
}

/// A record's members and member stand-in as live keys, when the record is
/// current and every member resolves. The cheap half of [`project_fold`].
pub(crate) fn fold_keys(
    graph: &Graph,
    fold: &FoldRecord,
) -> Option<(BTreeSet<NodeKey>, Option<NodeKey>)> {
    fold.validate().ok()?;
    let key_of = |member: &str| {
        uuid::Uuid::parse_str(member)
            .ok()
            .and_then(|id| graph.get_node_key_by_id(id))
    };
    let members: BTreeSet<NodeKey> = fold
        .members
        .iter()
        .map(|member| key_of(member))
        .collect::<Option<_>>()?;
    if members.len() != fold.members.len() {
        return None;
    }
    let stand_in = match &fold.stand_in {
        FoldStandIn::Member(member) => Some(key_of(member)?),
        FoldStandIn::Summary { .. } => None,
    };
    Some((members, stand_in))
}

/// Every node reachable from `root` through relations of `hierarchy_family`
/// in `direction`, `root` included. Each frontier node visits only its own
/// incident edges on the requested side. Surface adjacency costs O(reachable ×
/// degree); lifting each incident Resource bucket additionally scans the shown
/// bindings to find its Surface aliases.
pub(crate) fn hierarchy_closure(
    graph: &Graph,
    root: NodeKey,
    hierarchy_family: EdgeFamily,
    direction: FoldTraversalDirection,
) -> BTreeSet<NodeKey> {
    let mut members = BTreeSet::from([root]);
    let mut pending = VecDeque::from([root]);
    while let Some(current) = pending.pop_front() {
        let incident: Box<dyn Iterator<Item = NodeKey>> = match direction {
            FoldTraversalDirection::Outgoing => Box::new(
                graph
                    .projected_outgoing_relations(current)
                    .filter(|(_, _, payload)| payload.families().contains(&hierarchy_family))
                    .map(|(neighbor, _, _)| neighbor),
            ),
            FoldTraversalDirection::Incoming => Box::new(
                graph
                    .projected_incoming_relations(current)
                    .filter(|(_, _, payload)| payload.families().contains(&hierarchy_family))
                    .map(|(neighbor, _, _)| neighbor),
            ),
        };
        for next in incident {
            if members.insert(next) {
                pending.push_back(next);
            }
        }
    }
    members
}

/// Project one record against its current source graph.
///
/// A stale or unsupported record is refused rather than folding a partial
/// member set. The caller can keep the record for inspection and offer an
/// explicit repair or removal action.
pub fn project_fold(graph: &Graph, fold: &FoldRecord) -> Option<FoldProjection> {
    let (members, stand_in) = fold_keys(graph, fold)?;

    let mut bundles: BTreeMap<(NodeKey, FoldBoundaryDirection, EdgeFamily), usize> =
        BTreeMap::new();
    let mut internal_relation_count = 0;
    for (_, relation) in graph.projected_relations() {
        let from_folded = members.contains(&relation.from);
        let to_folded = members.contains(&relation.to);
        match (from_folded, to_folded) {
            (true, true) => internal_relation_count += 1,
            (true, false) => {
                *bundles
                    .entry((
                        relation.to,
                        FoldBoundaryDirection::Outgoing,
                        relation.kind.family(),
                    ))
                    .or_default() += 1;
            },
            (false, true) => {
                *bundles
                    .entry((
                        relation.from,
                        FoldBoundaryDirection::Incoming,
                        relation.kind.family(),
                    ))
                    .or_default() += 1;
            },
            (false, false) => {},
        }
    }

    let visible_members = graph
        .nodes()
        .filter_map(|(key, _)| (!members.contains(&key)).then_some(key))
        .collect();
    Some(FoldProjection {
        fold_id: fold.id.clone(),
        members,
        stand_in,
        visible_members,
        internal_relation_count,
        boundary_bundles: bundles
            .into_iter()
            .map(|((outside, direction, family), count)| FoldBoundaryBundle {
                outside,
                direction,
                family,
                count,
            })
            .collect(),
        label: fold.label.clone(),
    })
}

#[cfg(test)]
#[path = "fold_projection/tests.rs"]
mod tests;
