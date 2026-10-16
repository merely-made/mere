// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Canonical link appearance geometry for canvas display and picking.

use std::collections::HashSet;

use euclid::default::{Box2D, Point2D};
use kernel::graph::{EdgeFamily, Graph, NodeKey, RelationKind, RelationSelector};
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, StrokeCap,
    StrokeItem, StrokeJoin,
};
use scenograph::link::{LinkAssertion, LinkKey, LinkLegacyMember, LinkSource, LinkStratum};
use seiche::LayoutView;

use crate::canvas::{EdgeCell, LinkAppearance};

/// Build one geometry item per canonical link appearance. A Resource link may
/// be routed over more than one displayed surface pair; those appearances
/// share `key` and therefore share selection/open/hide identity.
pub(crate) fn visible_link_appearances(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    node_size: impl Fn(NodeKey) -> f32,
) -> Vec<LinkAppearanceSegment> {
    visible_link_appearances_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        &HashSet::new(),
        node_size,
    )
}

pub(crate) fn visible_link_appearances_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    node_size: impl Fn(NodeKey) -> f32,
) -> Vec<LinkAppearanceSegment> {
    let source = LinkSource::Unavailable {
        reason:
            "Pictograph receives a graph without its owning source authority or public revision"
                .into(),
    };
    let pairs: std::collections::BTreeSet<_> = graph
        .projected_relations()
        .map(|(_, row)| endpoint_pair(row.from, row.to))
        .collect();
    let mut appearances = std::collections::BTreeMap::new();
    for (a, b) in pairs {
        let (Some(a_node), Some(b_node)) = (graph.get_node(a), graph.get_node(b)) else {
            continue;
        };
        let Ok(pair) = super::link_reading::read_link(graph, a_node.id, b_node.id, source.clone())
        else {
            continue;
        };
        for mut reading in pair.links {
            if hidden_links.contains(&reading.key) {
                continue;
            }
            let mut filtered = reading.clone();
            filtered.buckets.retain_mut(|bucket| {
                let (Ok(from_id), Ok(to_id)) = (
                    bucket.displayed_from.parse::<uuid::Uuid>(),
                    bucket.displayed_to.parse::<uuid::Uuid>(),
                ) else {
                    return false;
                };
                let (Some(display_from), Some(display_to)) = (
                    graph.get_node_key_by_id(from_id),
                    graph.get_node_key_by_id(to_id),
                ) else {
                    return false;
                };
                bucket.assertions.retain(|assertion| {
                    if hidden_assertions.contains(&assertion.id) {
                        return false;
                    }
                    let Some(selector) = selector_for_link_assertion(assertion) else {
                        return true;
                    };
                    !hidden_legacy_cells.contains(&EdgeCell {
                        from: display_from,
                        to: display_to,
                        selector,
                    })
                });
                bucket.legacy_members.retain(|member| {
                    let Some(selector) = selector_for_legacy_member(member) else {
                        return true;
                    };
                    !hidden_legacy_cells.contains(&EdgeCell {
                        from: display_from,
                        to: display_to,
                        selector,
                    })
                });
                !bucket.assertions.is_empty() || !bucket.legacy_members.is_empty()
            });
            if filtered.buckets.is_empty() {
                continue;
            }
            reading.buckets = filtered.buckets;
            for bucket in &reading.buckets {
                let (Ok(from_id), Ok(to_id)) = (
                    bucket.displayed_from.parse::<uuid::Uuid>(),
                    bucket.displayed_to.parse::<uuid::Uuid>(),
                ) else {
                    continue;
                };
                let (Some(bucket_from), Some(bucket_to)) = (
                    graph.get_node_key_by_id(from_id),
                    graph.get_node_key_by_id(to_id),
                ) else {
                    continue;
                };
                let (display_from, display_to) = if bucket_from <= bucket_to {
                    (bucket_from, bucket_to)
                } else {
                    (bucket_to, bucket_from)
                };
                let from_id = graph
                    .get_node(display_from)
                    .expect("resolved display endpoint")
                    .id;
                let to_id = graph
                    .get_node(display_to)
                    .expect("resolved display endpoint")
                    .id;
                let summary = reading.rest_summary(&from_id.to_string(), &to_id.to_string());
                let appearance = LinkAppearance {
                    key: reading.key.clone(),
                    display_from,
                    display_to,
                    marks: summary.marks,
                    strength: summary.strength,
                };
                appearances.insert(
                    (appearance.key.clone(), display_from, display_to),
                    appearance,
                );
            }
        }
    }
    let mut strata_by_pair = std::collections::BTreeMap::<
        (NodeKey, NodeKey),
        std::collections::BTreeSet<LinkStratum>,
    >::new();
    for appearance in appearances.values() {
        strata_by_pair
            .entry(endpoint_pair(
                appearance.display_from,
                appearance.display_to,
            ))
            .or_default()
            .insert(appearance.key.stratum);
    }
    let mut segments = Vec::new();
    for appearance in appearances.into_values() {
        let (Some(mut from), Some(mut to)) = (
            view.position_of(appearance.display_from),
            view.position_of(appearance.display_to),
        ) else {
            continue;
        };
        let points = if appearance.display_from == appearance.display_to {
            self_link_points(
                from,
                node_size(appearance.display_from),
                appearance.key.stratum,
            )
        } else {
            if strata_by_pair
                .get(&endpoint_pair(
                    appearance.display_from,
                    appearance.display_to,
                ))
                .is_some_and(|strata| strata.len() > 1)
            {
                let dx = to.x - from.x;
                let dy = to.y - from.y;
                let len = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
                let sign = if appearance.key.stratum == LinkStratum::Surface {
                    -1.0
                } else {
                    1.0
                };
                let offset = sign * 3.0;
                let nx = -dy / len * offset;
                let ny = dx / len * offset;
                from = Point2D::new(from.x + nx, from.y + ny);
                to = Point2D::new(to.x + nx, to.y + ny);
            }
            vec![from, to]
        };
        segments.push(LinkAppearanceSegment {
            appearance,
            from,
            to,
            points,
        });
    }
    segments
}

/// Return a small arch that leaves and rejoins the node's top boundary.
/// Surface and Resource self-links use separate horizontal slots so both
/// strata remain visible and pickable.
fn self_link_points(
    center: Point2D<f32>,
    node_size: f32,
    stratum: LinkStratum,
) -> Vec<Point2D<f32>> {
    const SEGMENTS: usize = 20;
    let size = node_size.max(1.0);
    let horizontal_radius = size * 0.25;
    let vertical_radius = 16.0_f32.max(size * 0.25);
    let side = match stratum {
        LinkStratum::Surface => -horizontal_radius,
        LinkStratum::Resource => horizontal_radius,
    };
    let boundary_y = center.y - size * 0.5;
    let loop_center = Point2D::new(center.x + side, boundary_y);
    (0..=SEGMENTS)
        .map(|index| {
            if index == 0 {
                return Point2D::new(loop_center.x - horizontal_radius, boundary_y);
            }
            if index == SEGMENTS {
                return Point2D::new(loop_center.x + horizontal_radius, boundary_y);
            }
            let angle =
                std::f32::consts::PI + std::f32::consts::PI * index as f32 / SEGMENTS as f32;
            Point2D::new(
                loop_center.x + angle.cos() * horizontal_radius,
                loop_center.y + angle.sin() * vertical_radius,
            )
        })
        .collect()
}

fn selector_for_link_assertion(assertion: &LinkAssertion) -> Option<RelationSelector> {
    Some(
        RelationKind::from_predicate_iri(&assertion.predicate)
            .map(super::relationship_topology::selector_for_relation_kind)
            .unwrap_or(RelationSelector::Family(EdgeFamily::Semantic)),
    )
}

fn selector_for_legacy_member(member: &LinkLegacyMember) -> Option<RelationSelector> {
    if member.family == "Traversal" {
        return Some(RelationSelector::Family(EdgeFamily::Traversal));
    }
    let sub_kind = member.sub_kind.as_ref()?;
    let variant = match member.family.as_str() {
        "Semantic" => "Semantic",
        "Containment" => "Containment",
        "Arrangement" => "Arrangement",
        "Imported" => "Imported",
        "Provenance" => "Provenance",
        _ => return None,
    };
    let kind: RelationKind =
        serde_json::from_value(serde_json::json!({ (variant): sub_kind })).ok()?;
    Some(selector_for_relation_kind(kind))
}

#[derive(Clone, Debug)]
pub(crate) struct LinkAppearanceSegment {
    pub appearance: LinkAppearance,
    pub from: Point2D<f32>,
    pub to: Point2D<f32>,
    pub points: Vec<Point2D<f32>>,
}

impl LinkAppearanceSegment {
    fn is_self_link(&self) -> bool {
        self.appearance.display_from == self.appearance.display_to
    }
}

fn polyline_strokes(points: &[Point2D<f32>], color: ColorF, width: f32) -> Vec<PaintCmd> {
    points
        .windows(2)
        .map(|pair| stroke(pair[0], pair[1], color, width))
        .collect()
}

fn polyline_distance(point: Point2D<f32>, points: &[Point2D<f32>]) -> f32 {
    points
        .windows(2)
        .map(|pair| point_segment_distance(point, pair[0], pair[1]))
        .fold(f32::INFINITY, f32::min)
}

fn polyline_intersects_box(points: &[Point2D<f32>], region: Box2D<f32>) -> bool {
    points
        .windows(2)
        .any(|pair| segment_intersects_box(pair[0], pair[1], region))
}

fn loop_arrowhead(points: &[Point2D<f32>], reverse: bool, color: ColorF) -> Vec<PaintCmd> {
    let mut path = points.to_vec();
    if reverse {
        path.reverse();
    }
    let index = (path.len().saturating_sub(2) * 2 / 3).min(path.len().saturating_sub(2));
    if path.len() < 2 {
        return Vec::new();
    }
    arrowhead(path[index], path[index + 1], color)
}

fn polyline_midpoint_normal(points: &[Point2D<f32>]) -> (Point2D<f32>, Point2D<f32>) {
    let index = points.len().saturating_sub(2) / 2;
    let (Some(&from), Some(&to)) = (points.get(index), points.get(index + 1)) else {
        return (Point2D::new(0.0, 0.0), Point2D::new(0.0, 0.0));
    };
    let midpoint = Point2D::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5);
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    (midpoint, Point2D::new(-dy / len, dx / len))
}

pub(crate) fn link_hit_test(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Option<LinkAppearance> {
    link_hit_test_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        &HashSet::new(),
        point,
        tolerance,
        node_visible,
        node_size,
    )
}

pub(crate) fn link_hit_test_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Option<LinkAppearance> {
    link_hit_test_where(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        point,
        tolerance,
        node_visible,
        node_size,
        |_| true,
    )
}

pub(crate) fn self_link_hit_test(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Option<LinkAppearance> {
    self_link_hit_test_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        &HashSet::new(),
        point,
        tolerance,
        node_visible,
        node_size,
    )
}

pub(crate) fn self_link_hit_test_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Option<LinkAppearance> {
    link_hit_test_where(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        point,
        tolerance,
        node_visible,
        node_size,
        |segment| segment.appearance.display_from == segment.appearance.display_to,
    )
}

fn link_hit_test_where(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
    include: impl Fn(&LinkAppearanceSegment) -> bool,
) -> Option<LinkAppearance> {
    visible_link_appearances_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        node_size,
    )
    .into_iter()
    .filter(|segment| {
        include(segment)
            && node_visible(segment.appearance.display_from)
            && node_visible(segment.appearance.display_to)
    })
    .filter_map(|segment| {
        let distance = polyline_distance(point, &segment.points);
        (distance <= tolerance).then_some((segment.appearance, distance))
    })
    .min_by(|a, b| a.1.total_cmp(&b.1))
    .map(|(appearance, _)| appearance)
}

pub(crate) fn links_in_rect(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    region: Box2D<f32>,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Vec<LinkAppearance> {
    links_in_rect_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        &HashSet::new(),
        region,
        node_visible,
        node_size,
    )
}

pub(crate) fn links_in_rect_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    region: Box2D<f32>,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Vec<LinkAppearance> {
    visible_link_appearances_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        node_size,
    )
    .into_iter()
    .filter(|segment| {
        node_visible(segment.appearance.display_from)
            && node_visible(segment.appearance.display_to)
            && polyline_intersects_box(&segment.points, region)
    })
    .fold(
        std::collections::BTreeMap::new(),
        |mut appearances, segment| {
            appearances
                .entry(segment.appearance.key.clone())
                .or_insert(segment.appearance);
            appearances
        },
    )
    .into_values()
    .collect()
}

pub(crate) fn link_overlay(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    selected_links: &HashSet<LinkKey>,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
) -> Vec<PaintCmd> {
    link_overlay_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        &HashSet::new(),
        selected_links,
        node_visible,
        node_size,
        |_, _| 1.0,
    )
}

pub(crate) fn link_overlay_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    selected_links: &HashSet<LinkKey>,
    node_visible: impl Fn(NodeKey) -> bool,
    node_size: impl Fn(NodeKey) -> f32,
    tone: impl Fn(NodeKey, NodeKey) -> f32,
) -> Vec<PaintCmd> {
    visible_link_appearances_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        node_size,
    )
    .into_iter()
    .filter(|segment| {
        node_visible(segment.appearance.display_from) && node_visible(segment.appearance.display_to)
    })
    .flat_map(|segment| {
        let share = tone(
            segment.appearance.display_from,
            segment.appearance.display_to,
        );
        let mut strokes = link_strokes(&segment, share);
        if selected_links.contains(&segment.appearance.key) {
            strokes.extend(polyline_strokes(
                &segment.points,
                ColorF::new(0.91, 0.59, 0.16, 1.0),
                4.0,
            ));
        }
        strokes
    })
    .collect()
}

/// A link's strokes at `share` of its loud alpha. A quiet link (share below
/// one) is a hairline without marks; the grammar raises it when it touches
/// the foreground.
fn link_strokes(segment: &LinkAppearanceSegment, share: f32) -> Vec<PaintCmd> {
    let mut color = match segment.appearance.key.stratum {
        LinkStratum::Surface => ColorF::new(0.93, 0.70, 0.34, 0.76),
        LinkStratum::Resource => ColorF::new(0.55, 0.72, 0.92, 0.82),
    };
    color.a *= share;
    if share < 1.0 {
        return polyline_strokes(&segment.points, color, 1.0);
    }
    let width = 1.35 + segment.appearance.strength.max(0.0).ln_1p().min(3.0) * 0.55;
    let mut strokes = polyline_strokes(&segment.points, color, width);
    let marks = segment.appearance.marks;
    if marks.forward {
        strokes.extend(if segment.is_self_link() {
            loop_arrowhead(&segment.points, false, color)
        } else {
            arrowhead(segment.from, segment.to, color)
        });
    }
    if marks.reverse {
        strokes.extend(if segment.is_self_link() {
            loop_arrowhead(&segment.points, true, color)
        } else {
            arrowhead(segment.to, segment.from, color)
        });
    }
    if marks.opposed {
        let (midpoint, normal) = if segment.is_self_link() {
            polyline_midpoint_normal(&segment.points)
        } else {
            let midpoint = Point2D::new(
                (segment.from.x + segment.to.x) * 0.5,
                (segment.from.y + segment.to.y) * 0.5,
            );
            let dx = segment.to.x - segment.from.x;
            let dy = segment.to.y - segment.from.y;
            let len = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
            (midpoint, Point2D::new(-dy / len, dx / len))
        };
        strokes.push(stroke(
            Point2D::new(midpoint.x - normal.x * 3.0, midpoint.y - normal.y * 3.0),
            Point2D::new(midpoint.x + normal.x * 3.0, midpoint.y + normal.y * 3.0),
            color,
            2.0,
        ));
    }
    strokes
}

fn arrowhead(from: Point2D<f32>, to: Point2D<f32>, color: ColorF) -> Vec<PaintCmd> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    let ux = dx / len;
    let uy = dy / len;
    let tip = Point2D::new(from.x + dx * 0.66, from.y + dy * 0.66);
    let back = Point2D::new(tip.x - ux * 7.0, tip.y - uy * 7.0);
    let side = Point2D::new(-uy * 3.5, ux * 3.5);
    vec![
        stroke(
            tip,
            Point2D::new(back.x - side.x, back.y - side.y),
            color,
            1.8,
        ),
        stroke(
            tip,
            Point2D::new(back.x + side.x, back.y + side.y),
            color,
            1.8,
        ),
    ]
}

pub(crate) use super::relationship_topology::selector_for_relation_kind;

pub(crate) fn edge_cell_for_relation(from: NodeKey, to: NodeKey, kind: RelationKind) -> EdgeCell {
    EdgeCell {
        from,
        to,
        selector: selector_for_relation_kind(kind),
    }
}

fn endpoint_pair(a: NodeKey, b: NodeKey) -> (NodeKey, NodeKey) {
    if a <= b { (a, b) } else { (b, a) }
}

fn stroke(from: Point2D<f32>, to: Point2D<f32>, color: ColorF, width: f32) -> PaintCmd {
    let p0 = LayoutPoint::new(from.x, from.y);
    let p1 = LayoutPoint::new(to.x, to.y);
    let bounds = LayoutRect::new(
        LayoutPoint::new(p0.x.min(p1.x), p0.y.min(p1.y)),
        LayoutPoint::new(p0.x.max(p1.x), p0.y.max(p1.y)),
    );
    PaintCmd::DrawStroke(StrokeItem {
        placement: CommonPlacement::new(bounds),
        path: PathData {
            commands: vec![PathCommand::MoveTo(p0), PathCommand::LineTo(p1)],
        },
        color,
        width,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        dash: None,
    })
}

pub(crate) fn relation_family_color(family: EdgeFamily) -> ColorF {
    match family {
        EdgeFamily::Semantic => ColorF::new(0.93, 0.70, 0.34, 0.76),
        EdgeFamily::Traversal => ColorF::new(0.55, 0.72, 0.92, 0.62),
        EdgeFamily::Containment => ColorF::new(0.47, 0.80, 0.58, 0.70),
        EdgeFamily::Arrangement => ColorF::new(0.75, 0.63, 0.92, 0.70),
        EdgeFamily::Imported => ColorF::new(0.50, 0.78, 0.82, 0.68),
        EdgeFamily::Provenance => ColorF::new(0.90, 0.58, 0.68, 0.72),
    }
}

fn point_segment_distance(p: Point2D<f32>, a: Point2D<f32>, b: Point2D<f32>) -> f32 {
    let ab = b - a;
    let len2 = ab.square_length();
    if len2 <= f32::EPSILON {
        return (p - a).length();
    }
    let t = ((p - a).dot(ab) / len2).clamp(0.0, 1.0);
    let proj = a + ab * t;
    (p - proj).length()
}

fn segment_intersects_box(a: Point2D<f32>, b: Point2D<f32>, r: Box2D<f32>) -> bool {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let p = [-dx, dx, -dy, dy];
    let q = [a.x - r.min.x, r.max.x - a.x, a.y - r.min.y, r.max.y - a.y];
    let mut u1 = 0.0_f32;
    let mut u2 = 1.0_f32;
    for i in 0..4 {
        if p[i].abs() < f32::EPSILON {
            if q[i] < 0.0 {
                return false;
            }
        } else {
            let t = q[i] / p[i];
            if p[i] < 0.0 {
                if t > u2 {
                    return false;
                }
                if t > u1 {
                    u1 = t;
                }
            } else {
                if t < u1 {
                    return false;
                }
                if t < u2 {
                    u2 = t;
                }
            }
        }
    }
    u1 <= u2
}
