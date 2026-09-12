// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Root-canvas projection of one view-local [`forme::FoldRecord`].
//!
//! This is deliberately a proof layer, not a renderer. It makes the source
//! membership and relation accounting explicit before a synthetic summary node
//! is introduced into Canvas paint, hit-testing, or physics.

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use forme::{FOLD_RECORD_VERSION, FoldId, FoldRecord};
use kernel::geometry::PortablePoint;
use kernel::graph::{EdgeFamily, Graph, NodeKey};

use crate::{Canvas, FoldViewState};

/// The synthetic summary body's radius in Canvas world pixels. It deliberately
/// reads larger than an ordinary node because it stands for several members.
pub(crate) const FOLD_SUMMARY_RADIUS: f32 = 30.0;

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

/// Cells crossing a fold boundary, bundled only by their outside endpoint,
/// direction, and relation family.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldBoundaryBundle {
    pub outside: NodeKey,
    pub direction: FoldBoundaryDirection,
    pub family: EdgeFamily,
    pub count: usize,
}

/// The source accounting a Canvas summary node will render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FoldProjection {
    pub fold_id: FoldId,
    /// Source nodes replaced by the synthetic summary in this projection.
    pub members: BTreeSet<NodeKey>,
    /// Source nodes still individually visible beside the summary.
    pub visible_members: Vec<NodeKey>,
    /// Relation cells with both endpoints inside the fold.
    pub internal_relation_count: usize,
    /// Relation cells crossing the boundary, exact after family bundling.
    pub boundary_bundles: Vec<FoldBoundaryBundle>,
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
}

impl Canvas {
    fn fold_view_state(&self) -> FoldViewState {
        FoldViewState {
            fold: self.fold.clone(),
            selected: self.selected.clone(),
            selected_edges: self.selected_edges.clone(),
        }
    }

    fn push_fold_undo(&mut self) {
        self.fold_undo.push(self.fold_view_state());
        self.fold_redo.clear();
    }

    fn apply_fold_view_state(&mut self, state: FoldViewState) {
        self.fold = state.fold;
        self.fold_press = None;
        self.selected = state.selected;
        self.selected_edges = state.selected_edges;
    }

    fn fold_view_state_is_current(&self, state: &FoldViewState) -> bool {
        state
            .fold
            .as_ref()
            .is_none_or(|record| project_fold(&self.graph, record).is_some())
            && state
                .selected
                .iter()
                .all(|&key| self.graph.get_node(key).is_some())
    }

    /// Collapse the current multi-selection into one synthetic Canvas summary.
    ///
    /// The returned id is durable view-state identity. This method never adds,
    /// removes, or rewrites a graph node or relation.
    pub fn fold_selected(&mut self, source_scope: impl Into<String>) -> Option<FoldId> {
        if self.fold.is_some() {
            return None;
        }
        let members = self
            .selected
            .iter()
            .filter_map(|&key| self.graph.get_node(key).map(|node| node.id));
        let record = FoldRecord::from_selection(source_scope, members)?;
        let id = record.id;
        self.push_fold_undo();
        self.fold = Some(record);
        self.selected.clear();
        self.selected_edges.clear();
        Some(id)
    }

    /// Fold one root and every node reachable through the explicitly supplied
    /// relation family and direction. This does not guess hierarchy from the
    /// rest of the graph: callers must choose both the family and traversal.
    pub fn collapse_descendants(
        &mut self,
        root: NodeKey,
        hierarchy_family: EdgeFamily,
        direction: FoldTraversalDirection,
        source_scope: impl Into<String>,
    ) -> Option<FoldId> {
        if self.fold.is_some() || self.graph.get_node(root).is_none() {
            return None;
        }
        let members = hierarchy_closure(&self.graph, root, hierarchy_family, direction);
        let record = FoldRecord::from_selection(
            source_scope,
            members
                .iter()
                .filter_map(|&key| self.graph.get_node(key).map(|node| node.id)),
        )?;
        let id = record.id;
        self.push_fold_undo();
        self.fold = Some(record);
        self.selected.clear();
        self.selected_edges.clear();
        Some(id)
    }

    /// Expand the active summary, restoring its source members at their unchanged
    /// layout positions and selecting them for immediate orientation.
    pub fn expand_fold(&mut self, id: FoldId) -> bool {
        let Some(record) = self.fold.as_ref() else {
            return false;
        };
        if record.id != id {
            return false;
        }
        let Some(projection) = project_fold(&self.graph, record) else {
            return false;
        };
        self.push_fold_undo();
        self.fold = None;
        self.fold_press = None;
        self.selected = projection.members.into_iter().collect();
        self.selected_edges.clear();
        true
    }

    /// Expand whichever fold Canvas is currently projecting.
    pub fn expand_active_fold(&mut self) -> bool {
        let Some(id) = self.fold.as_ref().map(|record| record.id) else {
            return false;
        };
        self.expand_fold(id)
    }

    /// Revert the last local fold or expand. The source graph and its physics
    /// have never changed, so this only restores a prior curation projection.
    pub fn undo_fold(&mut self) -> bool {
        let Some(previous) = self.fold_undo.pop() else {
            return false;
        };
        if !self.fold_view_state_is_current(&previous) {
            return false;
        }
        self.fold_redo.push(self.fold_view_state());
        self.apply_fold_view_state(previous);
        true
    }

    /// Reapply a previously undone fold or expand.
    pub fn redo_fold(&mut self) -> bool {
        let Some(next) = self.fold_redo.pop() else {
            return false;
        };
        if !self.fold_view_state_is_current(&next) {
            return false;
        }
        self.fold_undo.push(self.fold_view_state());
        self.apply_fold_view_state(next);
        true
    }

    /// The current fold's source accounting, if it is still valid against the
    /// live graph. A stale record remains durable state but does not partially
    /// render.
    pub fn active_fold_projection(&self) -> Option<FoldProjection> {
        self.fold
            .as_ref()
            .and_then(|record| project_fold(&self.graph, record))
    }

    /// The durable fold record a host should place in its per-pane view intent.
    /// The record names source members by stable id; it does not serialize a
    /// synthetic Canvas node or any physics state.
    pub fn fold_record(&self) -> Option<&FoldRecord> {
        self.fold.as_ref()
    }

    /// Restore a persisted fold record when it still projects completely
    /// against this graph. A stale record is refused intact so the host can
    /// offer repair rather than silently collapsing a different selection.
    pub fn restore_fold(&mut self, record: Option<FoldRecord>) -> bool {
        if let Some(record) = record {
            if project_fold(&self.graph, &record).is_none() {
                return false;
            }
            self.fold = Some(record);
            self.selected.clear();
            self.selected_edges.clear();
        } else {
            self.fold = None;
        }
        self.fold_press = None;
        true
    }

    pub(crate) fn node_visible_in_canvas(&self, key: NodeKey) -> bool {
        self.node_in_scope(key)
    }

    pub(crate) fn fold_summary_at_screen(&self, cursor: (f32, f32)) -> Option<FoldId> {
        let projection = self.active_fold_projection()?;
        let center = projection.summary_center(|key| {
            self.view
                .position_of(key)
                .map(|position| PortablePoint::new(position.x, position.y))
        })?;
        let (x, y) = self.camera.to_screen(center);
        let radius = FOLD_SUMMARY_RADIUS * self.camera.zoom;
        ((cursor.0 - x).hypot(cursor.1 - y) <= radius).then_some(projection.fold_id)
    }
}

/// Project one record against its current source graph.
///
/// A stale or unsupported record is refused rather than folding a partial
/// member set. The caller can keep the record for inspection and offer an
/// explicit repair or removal action.
/// Every node reachable from `root` through relations of `hierarchy_family`
/// in `direction`, `root` included. Each frontier node visits only its own
/// incident edges on the requested side, so the walk costs O(reachable ×
/// degree) rather than rescanning the whole relation table per node.
fn hierarchy_closure(
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
                    .outgoing_relations(current)
                    .filter(|relation| relation.kind.family() == hierarchy_family)
                    .map(|relation| relation.to),
            ),
            FoldTraversalDirection::Incoming => Box::new(
                graph
                    .incoming_relations(current)
                    .filter(|relation| relation.kind.family() == hierarchy_family)
                    .map(|relation| relation.from),
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

pub fn project_fold(graph: &Graph, fold: &FoldRecord) -> Option<FoldProjection> {
    if fold.version != FOLD_RECORD_VERSION || fold.members.len() < 2 {
        return None;
    }
    let members: BTreeSet<NodeKey> = fold
        .members
        .iter()
        .map(|member| graph.get_node_key_by_id(*member))
        .collect::<Option<_>>()?;
    if members.len() != fold.members.len() {
        return None;
    }

    let mut bundles: BTreeMap<(NodeKey, FoldBoundaryDirection, EdgeFamily), usize> =
        BTreeMap::new();
    let mut internal_relation_count = 0;
    for relation in graph.relations() {
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
        fold_id: fold.id,
        members,
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
    })
}

#[cfg(test)]
mod tests {
    use euclid::default::Point2D;
    use kernel::graph::{
        ArrangementSubKind, ContainmentSubKind, EdgeAssertion, SemanticSubKind,
        fixtures::GraphFixtures,
    };

    use super::*;

    #[test]
    fn fold_projects_members_and_bundles_only_boundary_cells_by_family() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a".into(), Point2D::new(0.0, 0.0));
        let b = graph.add_node("https://b".into(), Point2D::new(1.0, 0.0));
        let outside = graph.add_node("https://outside".into(), Point2D::new(2.0, 0.0));
        graph.assert_relation(
            a,
            b,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Hyperlink,
                label: None,
                decay_progress: None,
            },
        );
        graph.assert_relation(
            a,
            outside,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        );
        graph.assert_relation(
            b,
            outside,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Quotes,
                label: None,
                decay_progress: None,
            },
        );
        graph.assert_relation(
            outside,
            b,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::Domain,
            },
        );
        let fold = FoldRecord::from_selection(
            "graph:fixture",
            [graph.get_node(a).unwrap().id, graph.get_node(b).unwrap().id],
        )
        .expect("two source members");

        let projection = project_fold(&graph, &fold).expect("current record projects");
        assert_eq!(projection.members, BTreeSet::from([a, b]));
        assert_eq!(projection.visible_members, vec![outside]);
        assert_eq!(projection.internal_relation_count, 1);
        assert_eq!(
            projection.boundary_bundles,
            vec![
                FoldBoundaryBundle {
                    outside,
                    direction: FoldBoundaryDirection::Incoming,
                    family: EdgeFamily::Containment,
                    count: 1,
                },
                FoldBoundaryBundle {
                    outside,
                    direction: FoldBoundaryDirection::Outgoing,
                    family: EdgeFamily::Semantic,
                    count: 2,
                },
            ],
            "two semantic cells bundle, while a different family and direction remain distinct"
        );
        assert_eq!(graph.nodes().count(), 3, "the source graph stays intact");
        assert_eq!(
            graph.relations().count(),
            4,
            "projection does not remove cells"
        );
    }

    #[test]
    fn stale_member_refuses_partial_fold() {
        let mut graph = Graph::new();
        let node = graph.add_node("https://only-live".into(), Point2D::new(0.0, 0.0));
        let fold = FoldRecord::from_selection(
            "graph:fixture",
            [graph.get_node(node).unwrap().id, uuid::Uuid::from_u128(99)],
        )
        .expect("two ids in the durable record");
        assert!(project_fold(&graph, &fold).is_none());
    }

    #[test]
    fn collapsing_descendants_follows_only_the_explicit_family_and_direction() {
        let mut graph = Graph::new();
        let root = graph.add_node("https://root".into(), Point2D::new(0.0, 0.0));
        let child = graph.add_node("https://child".into(), Point2D::new(1.0, 0.0));
        let grandchild = graph.add_node("https://grandchild".into(), Point2D::new(2.0, 0.0));
        let linked = graph.add_node("https://linked".into(), Point2D::new(3.0, 0.0));
        graph.assert_relation(
            root,
            child,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::Domain,
            },
        );
        graph.assert_relation(
            child,
            grandchild,
            EdgeAssertion::Containment {
                sub_kind: ContainmentSubKind::Domain,
            },
        );
        graph.assert_relation(
            child,
            linked,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        );
        let source_nodes = graph.nodes().count();
        let source_relations = graph.relations().count();
        let mut canvas = Canvas::with_graph(graph);

        let id = canvas
            .collapse_descendants(
                root,
                EdgeFamily::Containment,
                FoldTraversalDirection::Outgoing,
                "canvas:hierarchy-test",
            )
            .expect("the containment descendants form a fold");
        let projection = canvas
            .active_fold_projection()
            .expect("current fold projects");
        assert_eq!(projection.fold_id, id);
        assert_eq!(
            projection.members,
            BTreeSet::from([root, child, grandchild])
        );
        assert_eq!(projection.internal_relation_count, 2);
        assert_eq!(projection.boundary_bundles.len(), 1);
        assert_eq!(projection.boundary_bundles[0].outside, linked);
        assert_eq!(projection.boundary_bundles[0].family, EdgeFamily::Semantic);
        assert_eq!(canvas.graph().nodes().count(), source_nodes);
        assert_eq!(canvas.graph().relations().count(), source_relations);
        assert!(canvas.undo_fold(), "collapse is a reversible view action");
        assert!(canvas.active_fold_projection().is_none());
        assert!(canvas.redo_fold(), "redo restores the same hierarchy fold");
        assert_eq!(canvas.active_fold_projection().unwrap().fold_id, id);
    }

    /// The pre-E1 closure: rescan every relation row in the graph for each
    /// frontier node. Kept only as the reference the incident-edge walk must
    /// reproduce member for member.
    fn brute_force_hierarchy_closure(
        graph: &Graph,
        root: NodeKey,
        hierarchy_family: EdgeFamily,
        direction: FoldTraversalDirection,
    ) -> BTreeSet<NodeKey> {
        let mut members = BTreeSet::from([root]);
        let mut pending = VecDeque::from([root]);
        while let Some(current) = pending.pop_front() {
            for relation in graph.relations() {
                if relation.kind.family() != hierarchy_family {
                    continue;
                }
                let next = match direction {
                    FoldTraversalDirection::Outgoing if relation.from == current => relation.to,
                    FoldTraversalDirection::Incoming if relation.to == current => relation.from,
                    _ => continue,
                };
                if members.insert(next) {
                    pending.push_back(next);
                }
            }
        }
        members
    }

    /// A containment chain of `chain_len` nodes, a containment star of
    /// `star_len` leaves hung off the chain's middle node, and a dense mesh
    /// of unrelated Semantic and Arrangement relations over every node.
    /// Returns the graph and the chain in root-to-tip order.
    fn large_hierarchy_fixture(chain_len: usize, star_len: usize) -> (Graph, Vec<NodeKey>) {
        let mut graph = Graph::new();
        let chain: Vec<NodeKey> = (0..chain_len)
            .map(|i| graph.add_node(format!("https://chain/{i}"), Point2D::new(i as f32, 0.0)))
            .collect();
        for pair in chain.windows(2) {
            graph.assert_relation(
                pair[0],
                pair[1],
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::Domain,
                },
            );
        }
        let hub = chain[chain_len / 2];
        let leaves: Vec<NodeKey> = (0..star_len)
            .map(|i| graph.add_node(format!("https://leaf/{i}"), Point2D::new(i as f32, 1.0)))
            .collect();
        for &leaf in &leaves {
            graph.assert_relation(
                hub,
                leaf,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::Domain,
                },
            );
        }
        // Unrelated families in every direction, including edges that would
        // pull the whole graph into one component if the family were ignored.
        let all: Vec<NodeKey> = chain.iter().chain(leaves.iter()).copied().collect();
        let total = all.len();
        for (i, &from) in all.iter().enumerate() {
            for stride in [1usize, 7, 13, 101] {
                let to = all[(i + stride) % total];
                graph.assert_relation(
                    to,
                    from,
                    EdgeAssertion::Semantic {
                        sub_kind: SemanticSubKind::Cites,
                        label: None,
                        decay_progress: None,
                    },
                );
                graph.assert_relation(
                    from,
                    to,
                    EdgeAssertion::Arrangement {
                        sub_kind: ArrangementSubKind::TileGroup,
                    },
                );
            }
        }
        (graph, chain)
    }

    #[test]
    fn incident_edge_walk_matches_brute_force_closure_on_large_hierarchy() {
        let (graph, chain) = large_hierarchy_fixture(300, 300);
        assert!(
            graph.relations().count() > 2_000,
            "the fixture must carry many unrelated relations"
        );
        let root = chain[0];
        let mid = chain[chain.len() / 2];
        let tip = chain[chain.len() - 1];

        for (start, direction) in [
            (root, FoldTraversalDirection::Outgoing),
            (mid, FoldTraversalDirection::Outgoing),
            (mid, FoldTraversalDirection::Incoming),
            (tip, FoldTraversalDirection::Incoming),
        ] {
            let expected =
                brute_force_hierarchy_closure(&graph, start, EdgeFamily::Containment, direction);
            assert!(
                expected.len() >= 2,
                "{direction:?} from {start:?} reaches a fold"
            );
            let mut canvas = Canvas::with_graph(graph.clone());
            canvas
                .collapse_descendants(
                    start,
                    EdgeFamily::Containment,
                    direction,
                    "canvas:hierarchy-large",
                )
                .expect("a hierarchy fold with at least two members");
            let projection = canvas
                .active_fold_projection()
                .expect("current fold projects");
            assert_eq!(projection.members, expected, "{direction:?} from {start:?}");
        }

        // Sanity on the reference itself: the root's outgoing closure is the
        // whole hierarchy and nothing else.
        let whole = brute_force_hierarchy_closure(
            &graph,
            root,
            EdgeFamily::Containment,
            FoldTraversalDirection::Outgoing,
        );
        assert_eq!(whole.len(), 600);

        // The tip has no outgoing containment, so both walks stop at the root
        // alone and the fold is refused exactly as before: a durable record
        // needs at least two members.
        let alone = brute_force_hierarchy_closure(
            &graph,
            tip,
            EdgeFamily::Containment,
            FoldTraversalDirection::Outgoing,
        );
        assert_eq!(alone, BTreeSet::from([tip]));
        let mut canvas = Canvas::with_graph(graph.clone());
        assert!(
            canvas
                .collapse_descendants(
                    tip,
                    EdgeFamily::Containment,
                    FoldTraversalDirection::Outgoing,
                    "canvas:hierarchy-large",
                )
                .is_none()
        );
        assert!(canvas.active_fold_projection().is_none());
    }

    #[test]
    fn outgoing_and_incoming_from_mid_chain_yield_distinct_correct_sets() {
        let (graph, chain) = large_hierarchy_fixture(301, 50);
        let mid_index = chain.len() / 2;
        let mid = chain[mid_index];

        let mut down = Canvas::with_graph(graph.clone());
        down.collapse_descendants(
            mid,
            EdgeFamily::Containment,
            FoldTraversalDirection::Outgoing,
            "canvas:hierarchy-down",
        )
        .expect("descendants fold");
        let down_members = down.active_fold_projection().unwrap().members;

        let mut up = Canvas::with_graph(graph);
        up.collapse_descendants(
            mid,
            EdgeFamily::Containment,
            FoldTraversalDirection::Incoming,
            "canvas:hierarchy-up",
        )
        .expect("ancestors fold");
        let up_members = up.active_fold_projection().unwrap().members;

        // Outgoing: the mid node, the rest of the chain below it, and every
        // star leaf hung off it.
        let expected_down: BTreeSet<NodeKey> = chain[mid_index..].iter().copied().collect();
        assert_eq!(down_members.len(), expected_down.len() + 50);
        assert!(down_members.is_superset(&expected_down));
        // Incoming: the mid node and the chain above it, no leaves.
        let expected_up: BTreeSet<NodeKey> = chain[..=mid_index].iter().copied().collect();
        assert_eq!(up_members, expected_up);

        assert_ne!(down_members, up_members);
        let overlap: BTreeSet<NodeKey> = down_members.intersection(&up_members).copied().collect();
        assert_eq!(overlap, BTreeSet::from([mid]));
    }
}
