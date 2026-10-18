// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Canonical link appearance geometry for canvas display and picking.

use std::collections::HashSet;

use euclid::default::{Box2D, Point2D};
use kernel::graph::{EdgeFamily, Graph, NodeKey, RelationKind, RelationSelector};
use paint_list_api::{ColorF, PaintCmd};
use scenograph::link::{
    LinkAssertion, LinkKey, LinkLegacyMember, LinkPairReading, LinkSource, LinkStratum,
};
use seiche::LayoutView;

use crate::canvas::{EdgeCell, LinkAppearance};

#[path = "edge_cells/paint.rs"]
mod paint;
pub(crate) use paint::link_overlay_from_segments;

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
    visible_link_segments_with_assertion_hides(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        node_size,
    )
}

/// Build resting appearance geometry without retaining source readings. This
/// is the ordinary paint/pick path while responsive paths are disabled.
pub(crate) fn visible_link_segments_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    node_size: impl Fn(NodeKey) -> f32,
) -> Vec<LinkAppearanceSegment> {
    visible_link_geometry_inner(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        node_size,
        false,
    )
    .0
}

/// Canonical visible source readings and their resting geometry. Responsive
/// paths are evaluated from these exact hide-filtered readings, then replace
/// only the returned segment points for this frame/pick operation.
pub(crate) fn visible_link_geometry_with_assertion_hides(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    node_size: impl Fn(NodeKey) -> f32,
) -> (Vec<LinkAppearanceSegment>, Vec<LinkPairReading>) {
    visible_link_geometry_inner(
        graph,
        view,
        hidden_links,
        hidden_legacy_cells,
        hidden_assertions,
        node_size,
        true,
    )
}

fn visible_link_geometry_inner(
    graph: &Graph,
    view: &LayoutView,
    hidden_links: &HashSet<LinkKey>,
    hidden_legacy_cells: &HashSet<EdgeCell>,
    hidden_assertions: &HashSet<String>,
    node_size: impl Fn(NodeKey) -> f32,
    retain_readings: bool,
) -> (Vec<LinkAppearanceSegment>, Vec<LinkPairReading>) {
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
    let mut visible_readings = Vec::new();
    for (a, b) in pairs {
        let (Some(a_node), Some(b_node)) = (graph.get_node(a), graph.get_node(b)) else {
            continue;
        };
        let Ok(pair) = super::link_reading::read_link(graph, a_node.id, b_node.id, source.clone())
        else {
            continue;
        };
        let mut visible_pair = LinkPairReading {
            source: pair.source,
            links: Vec::new(),
            coverage: pair.coverage,
        };
        for mut reading in pair.links {
            if hidden_links.contains(&reading.key) {
                continue;
            }
            reading.buckets.retain_mut(|bucket| {
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
            if reading.buckets.is_empty() {
                continue;
            }
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
            if retain_readings {
                visible_pair.links.push(reading);
            }
        }
        if retain_readings && !visible_pair.links.is_empty() {
            visible_readings.push(visible_pair);
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
            responsive: false,
        });
    }
    (segments, visible_readings)
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
    /// Whether this same-frame geometry came from the bounded response evaluator.
    pub responsive: bool,
}

impl LinkAppearanceSegment {
    fn is_self_link(&self) -> bool {
        self.appearance.display_from == self.appearance.display_to
    }
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
    link_hit_test_segments(
        visible_link_appearances_with_assertion_hides(
            graph,
            view,
            hidden_links,
            hidden_legacy_cells,
            hidden_assertions,
            node_size,
        ),
        point,
        tolerance,
        node_visible,
        include,
    )
}

pub(crate) fn link_hit_test_segments(
    segments: Vec<LinkAppearanceSegment>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    include: impl Fn(&LinkAppearanceSegment) -> bool,
) -> Option<LinkAppearance> {
    segments
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
    links_in_rect_segments(
        visible_link_appearances_with_assertion_hides(
            graph,
            view,
            hidden_links,
            hidden_legacy_cells,
            hidden_assertions,
            node_size,
        ),
        region,
        node_visible,
    )
}

pub(crate) fn links_in_rect_segments(
    segments: Vec<LinkAppearanceSegment>,
    region: Box2D<f32>,
    node_visible: impl Fn(NodeKey) -> bool,
) -> Vec<LinkAppearance> {
    segments
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
    link_overlay_from_segments(
        visible_link_appearances_with_assertion_hides(
            graph,
            view,
            hidden_links,
            hidden_legacy_cells,
            hidden_assertions,
            node_size,
        ),
        selected_links,
        node_visible,
        tone,
    )
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
