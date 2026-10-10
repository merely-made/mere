// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Root-canvas projection of one view-local [`curation::FoldRecord`].
//!
//! [`project_fold`] is the native check (mer3ly site plan, Ruling 166): a
//! record projects only when it is current and every member resolves against
//! the live graph, and its projection carries the boundary accounting. The
//! canvas draws a [`FoldStandIn::Summary`] as the synthetic summary body and a
//! [`FoldStandIn::Member`] as that member with a "+N" badge
//! ([`super::folds`]). Members are the graph's node UUIDs as strings,
//! converted here at the canvas's edge (Ruling 163).

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use curation::{FoldDirection, FoldId, FoldRecord, FoldStandIn};
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
mod tests {
    use euclid::default::Point2D;
    use kernel::graph::{
        ArrangementSubKind, ContainmentSubKind, EdgeAssertion, SemanticSubKind,
        fixtures::GraphFixtures,
    };

    use super::*;
    use crate::canvas::Canvas;

    fn selection(members: impl IntoIterator<Item = uuid::Uuid>) -> FoldRecord {
        FoldRecord::selection(
            FoldId::new("fold:fixture"),
            "graph:fixture",
            members.into_iter().map(|id| id.to_string()),
        )
        .expect("two source members")
    }

    /// The canvas's one fold, when it has exactly one.
    fn only_fold(canvas: &Canvas) -> Option<FoldProjection> {
        let mut folds = canvas.fold_projections();
        assert!(folds.len() <= 1, "one fold at most");
        folds.pop()
    }

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
        let fold = selection([graph.get_node(a).unwrap().id, graph.get_node(b).unwrap().id]);

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
            graph.projected_relations().count(),
            4,
            "projection does not remove cells"
        );
    }

    #[test]
    fn stale_member_refuses_partial_fold() {
        let mut graph = Graph::new();
        let node = graph.add_node("https://only-live".into(), Point2D::new(0.0, 0.0));
        let fold = selection([graph.get_node(node).unwrap().id, uuid::Uuid::from_u128(99)]);
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
        let source_relations = graph.projected_relations().count();
        let mut canvas = Canvas::with_graph(graph);

        let id = canvas
            .collapse_descendants(
                root,
                EdgeFamily::Containment,
                FoldTraversalDirection::Outgoing,
                "canvas:hierarchy-test",
            )
            .expect("the containment descendants form a fold");
        let projection = only_fold(&canvas).expect("current fold projects");
        assert_eq!(projection.fold_id, id);
        assert_eq!(projection.stand_in, Some(root), "the root stands in");
        assert_eq!(projection.badge(), "+2");
        assert_eq!(
            projection.members,
            BTreeSet::from([root, child, grandchild])
        );
        assert_eq!(projection.internal_relation_count, 2);
        assert_eq!(projection.boundary_bundles.len(), 1);
        assert_eq!(projection.boundary_bundles[0].outside, linked);
        assert_eq!(projection.boundary_bundles[0].family, EdgeFamily::Semantic);
        assert_eq!(canvas.graph().nodes().count(), source_nodes);
        assert_eq!(
            canvas.graph().projected_relations().count(),
            source_relations
        );
        assert!(canvas.undo_fold(), "collapse is a reversible view action");
        assert!(only_fold(&canvas).is_none());
        assert!(canvas.redo_fold(), "redo restores the same hierarchy fold");
        assert_eq!(only_fold(&canvas).unwrap().fold_id, id);
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
        let relations: Vec<_> = graph.projected_relations().map(|(_, row)| row).collect();
        let mut members = BTreeSet::from([root]);
        let mut pending = VecDeque::from([root]);
        while let Some(current) = pending.pop_front() {
            for relation in &relations {
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
            graph.projected_relations().count() > 2_000,
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
            let projection = only_fold(&canvas).expect("current fold projects");
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
        assert!(only_fold(&canvas).is_none());
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
        let down_members = only_fold(&down).unwrap().members;

        let mut up = Canvas::with_graph(graph);
        up.collapse_descendants(
            mid,
            EdgeFamily::Containment,
            FoldTraversalDirection::Incoming,
            "canvas:hierarchy-up",
        )
        .expect("ancestors fold");
        let up_members = only_fold(&up).unwrap().members;

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
    #[test]
    fn hierarchy_closure_keeps_legacy_surface_and_resource_families_distinct() {
        let mut graph = Graph::new();
        let root = graph.add_node("https://fold.test/root".into(), Point2D::zero());
        let child = graph.add_node("https://fold.test/child".into(), Point2D::zero());
        let grandchild = graph.add_node("https://fold.test/grandchild".into(), Point2D::zero());
        let linked = graph.add_node("https://fold.test/linked".into(), Point2D::zero());
        let from_id = graph.get_node(root).unwrap().id.to_string();
        let to_id = graph.get_node(child).unwrap().id.to_string();
        kernel::graph::replay_captured_deltas_onto(
            &mut graph,
            [kernel::graph::CapturedDelta::ReplayAssertRelationByIds {
                from_id,
                to_id,
                assertion: EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::Domain,
                },
            }],
        );
        let resource = graph
            .assert_relation(
                child,
                grandchild,
                EdgeAssertion::Containment {
                    sub_kind: ContainmentSubKind::Domain,
                },
            )
            .unwrap();
        assert!(matches!(resource, kernel::graph::RelationKey::Resource(_)));
        graph.assert_relation(
            root,
            linked,
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        );
        assert_eq!(
            graph.relations().count(),
            1,
            "legacy Surface hierarchy remains held"
        );
        assert_eq!(graph.projected_relations().count(), 3);
        let before = graph.to_snapshot();
        let expected = BTreeSet::from([root, child, grandchild]);
        for (start, direction) in [
            (root, FoldTraversalDirection::Outgoing),
            (grandchild, FoldTraversalDirection::Incoming),
        ] {
            assert_eq!(
                hierarchy_closure(&graph, start, EdgeFamily::Containment, direction),
                expected
            );
            assert_eq!(
                brute_force_hierarchy_closure(&graph, start, EdgeFamily::Containment, direction),
                expected
            );
        }
        assert_eq!(
            hierarchy_closure(
                &graph,
                root,
                EdgeFamily::Semantic,
                FoldTraversalDirection::Outgoing
            ),
            BTreeSet::from([root, linked])
        );
        let mut after = graph.to_snapshot();
        after.timestamp_secs = before.timestamp_secs;
        assert_eq!(
            serde_json::to_value(after).unwrap(),
            serde_json::to_value(before).unwrap()
        );
    }
}
