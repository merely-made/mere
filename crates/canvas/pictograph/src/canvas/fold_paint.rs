// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Route boundary cells through the complete fold set, once per source cell.

use std::collections::{BTreeMap, HashMap};

use kernel::geometry::PortablePoint;
use kernel::graph::{EdgeFamily, NodeKey};
use paint_list_api::{
    CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, RectItem, StrokeCap,
    StrokeItem, StrokeJoin,
};

use super::Canvas;
use super::edge_cells::{edge_cell_for_relation, relation_family_color};
use super::fold_projection::{FOLD_SUMMARY_RADIUS, FoldProjection};

/// A source node or a stand-in indexed in the current fold projection set.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum FoldEndpoint {
    Node(NodeKey),
    Fold(usize),
}

/// One family bundle between visible fold stand-ins or ordinary nodes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FoldRoute {
    pub from: FoldEndpoint,
    pub to: FoldEndpoint,
    pub family: EdgeFamily,
    pub count: usize,
}

impl Canvas {
    /// Boundary cells routed through the complete active fold set. Fold indices
    /// refer to `fold_projections()` in the same state. Hidden and out-of-scope
    /// source cells are excluded; each directed source cell contributes once.
    pub fn fold_routes(&self) -> Vec<FoldRoute> {
        let folds = self.fold_projections();
        if folds.is_empty() {
            return Vec::new();
        }
        let membership: HashMap<_, _> = folds
            .iter()
            .enumerate()
            .flat_map(|(index, fold)| fold.members.iter().map(move |&key| (key, index)))
            .collect();
        let endpoint = |key| {
            membership
                .get(&key)
                .map_or(FoldEndpoint::Node(key), |&index| FoldEndpoint::Fold(index))
        };
        let in_scope = |key| self.scope.as_ref().is_none_or(|scope| scope.contains(&key));
        let mut bundles = BTreeMap::new();
        for (_, relation) in self.graph.projected_relations() {
            if !membership.contains_key(&relation.from) && !membership.contains_key(&relation.to) {
                continue;
            }
            if !in_scope(relation.from)
                || !in_scope(relation.to)
                || self.hidden_edges.contains(&edge_cell_for_relation(
                    relation.from,
                    relation.to,
                    relation.kind,
                ))
            {
                continue;
            }
            let (from, to) = (endpoint(relation.from), endpoint(relation.to));
            if from != to {
                *bundles
                    .entry((from, to, relation.kind.family()))
                    .or_insert(0usize) += 1;
            }
        }
        bundles
            .into_iter()
            .map(|((from, to, family), count)| FoldRoute {
                from,
                to,
                family,
                count,
            })
            .collect()
    }

    pub(super) fn fold_overlay(
        &self,
        folds: &[FoldProjection],
        positions: &HashMap<NodeKey, PortablePoint>,
    ) -> Vec<PaintCmd> {
        if folds.is_empty() {
            return Vec::new();
        }
        let position = |key| positions.get(&key).copied();
        let geometry = |endpoint| match endpoint {
            FoldEndpoint::Node(key) => position(key).map(|p| (p, self.node_size(key) / 2.0)),
            FoldEndpoint::Fold(index) => {
                let fold = &folds[index];
                if let Some(key) = fold.stand_in {
                    position(key).map(|p| (p, self.node_size(key) / 2.0))
                } else {
                    fold.summary_center(position)
                        .map(|p| (p, FOLD_SUMMARY_RADIUS))
                }
            },
        };
        let mut commands = Vec::new();
        for FoldRoute {
            from,
            to,
            family,
            count,
        } in self.fold_routes()
        {
            let (Some((a, a_radius)), Some((b, b_radius))) = (geometry(from), geometry(to)) else {
                continue;
            };
            let (dx, dy) = (b.x - a.x, b.y - a.y);
            let length = dx.hypot(dy);
            if !length.is_finite() || length <= a_radius + b_radius {
                continue;
            }
            let (ux, uy) = (dx / length, dy / length);
            let from = LayoutPoint::new(a.x + ux * a_radius, a.y + uy * a_radius);
            let to = LayoutPoint::new(b.x - ux * b_radius, b.y - uy * b_radius);
            commands.push(PaintCmd::DrawStroke(StrokeItem {
                placement: CommonPlacement::new(LayoutRect::new(
                    LayoutPoint::new(from.x.min(to.x), from.y.min(to.y)),
                    LayoutPoint::new(from.x.max(to.x), from.y.max(to.y)),
                )),
                path: PathData {
                    commands: vec![PathCommand::MoveTo(from), PathCommand::LineTo(to)],
                },
                color: relation_family_color(family),
                width: 2.0 + count.saturating_sub(1).min(4) as f32,
                cap: StrokeCap::Round,
                join: StrokeJoin::Round,
                dash: None,
            }));
        }
        for fold in folds.iter().filter(|fold| fold.stand_in.is_none()) {
            if let Some(center) = fold.summary_center(position) {
                commands.push(PaintCmd::DrawRect(RectItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(
                            center.x - FOLD_SUMMARY_RADIUS,
                            center.y - FOLD_SUMMARY_RADIUS,
                        ),
                        LayoutPoint::new(
                            center.x + FOLD_SUMMARY_RADIUS,
                            center.y + FOLD_SUMMARY_RADIUS,
                        ),
                    )),
                    color: self.style.node_color,
                }));
            }
        }
        commands
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use curation::{FoldId, FoldRecord, FoldStandIn};
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{EdgeAssertion, Graph, RelationKind, SemanticSubKind};

    fn fixture(summary: bool) -> (Canvas, [NodeKey; 5]) {
        let mut graph = Graph::new();
        let keys = std::array::from_fn(|i| {
            graph.add_node(
                format!("https://fold-paint.test/{i}"),
                PortablePoint::zero(),
            )
        });
        let relation = || EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::UserGrouped,
            label: None,
        };
        graph.assert_relation(keys[1], keys[3], relation());
        graph.assert_relation(keys[0], keys[2], relation());
        graph.assert_relation(keys[1], keys[4], relation());
        let records = keys[..4]
            .chunks(2)
            .enumerate()
            .map(|(i, pair)| {
                let members: Vec<_> = pair
                    .iter()
                    .map(|&key| graph.get_node(key).unwrap().id.to_string())
                    .collect();
                let stand_in = if summary {
                    FoldStandIn::Summary { label: None }
                } else {
                    FoldStandIn::Member(members[0].clone())
                };
                FoldRecord::new(
                    FoldId::new(format!("fold:{i}")),
                    "paint-test",
                    members,
                    stand_in,
                    None,
                    None,
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        let mut canvas = Canvas::with_graph(graph);
        for (&key, x) in keys.iter().zip([0.0, 100.0, 500.0, 400.0, 900.0]) {
            canvas.view.set_position(key, PortablePoint::new(x, 0.0));
        }
        assert!(canvas.restore_folds(records).is_empty());
        (canvas, keys)
    }

    fn strokes(canvas: &Canvas) -> Vec<StrokeItem> {
        let positions = canvas
            .view
            .positions()
            .map(|(k, p)| (k, PortablePoint::new(p.x, p.y)))
            .collect();
        canvas
            .fold_overlay(&canvas.fold_projections(), &positions)
            .into_iter()
            .filter_map(|cmd| match cmd {
                PaintCmd::DrawStroke(stroke) => Some(stroke),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn member_boundary_cells_bundle_at_both_stand_ins_once() {
        let (mut canvas, keys) = fixture(false);
        let source_relations = canvas.graph.projected_relations().count();
        let painted = strokes(&canvas);
        assert_eq!(
            painted.len(),
            2,
            "one bundle between folds and one to the outside node"
        );
        let between = painted.iter().find(|stroke| stroke.width == 3.0).unwrap();
        let [PathCommand::MoveTo(from), PathCommand::LineTo(to)] = between.path.commands.as_slice()
        else {
            panic!("a boundary stroke");
        };
        assert!((from.x - canvas.node_size(keys[0]) / 2.0).abs() < 0.01);
        assert!((to.x - (500.0 - canvas.node_size(keys[2]) / 2.0)).abs() < 0.01);
        canvas.hidden_edges.insert(edge_cell_for_relation(
            keys[1],
            keys[3],
            RelationKind::Semantic(SemanticSubKind::UserGrouped),
        ));
        assert!(strokes(&canvas).iter().all(|stroke| stroke.width == 2.0));
        canvas.hidden_edges.insert(edge_cell_for_relation(
            keys[0],
            keys[2],
            RelationKind::Semantic(SemanticSubKind::UserGrouped),
        ));
        assert_eq!(
            strokes(&canvas).len(),
            1,
            "hiding every source cell removes its bundle"
        );
        assert_eq!(canvas.graph.projected_relations().count(), source_relations);
    }

    #[test]
    fn summary_boundary_routes_to_the_other_summary_not_its_hidden_members() {
        let (mut canvas, keys) = fixture(true);
        let painted = strokes(&canvas);
        assert_eq!(painted.len(), 2);
        let between = painted.iter().find(|stroke| stroke.width == 3.0).unwrap();
        let [PathCommand::MoveTo(from), PathCommand::LineTo(to)] = between.path.commands.as_slice()
        else {
            panic!("a boundary stroke");
        };
        assert_eq!(
            (from.x, to.x),
            (80.0, 420.0),
            "trim from centroids 50 and 450"
        );
        let roster = keys[..4]
            .iter()
            .map(|&key| canvas.graph.get_node(key).unwrap().id)
            .collect::<Vec<_>>();
        canvas.scope_to_members(roster);
        assert_eq!(
            strokes(&canvas).len(),
            1,
            "the scoped-out endpoint has no boundary stroke"
        );
    }
    #[test]
    fn boundary_bundles_keep_relation_families_and_directions_distinct() {
        let (mut canvas, keys) = fixture(false);
        canvas.graph.assert_relation(
            keys[1],
            keys[3],
            EdgeAssertion::Containment {
                sub_kind: kernel::graph::ContainmentSubKind::Domain,
            },
        );
        canvas.graph.assert_relation(
            keys[3],
            keys[1],
            EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::UserGrouped,
                label: None,
            },
        );
        let painted = strokes(&canvas);
        assert_eq!(painted.len(), 4);
        let semantic = relation_family_color(kernel::graph::EdgeFamily::Semantic);
        assert_eq!(
            painted
                .iter()
                .filter(|stroke| stroke.color == semantic)
                .count(),
            3
        );
        let reverse = painted.iter().find(|stroke| {
            matches!(stroke.path.commands.first(), Some(PathCommand::MoveTo(p)) if p.x > 400.0)
        }).unwrap();
        assert_eq!(
            reverse.width, 2.0,
            "the reverse cell is not merged into the outgoing bundle"
        );
    }
}
