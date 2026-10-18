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

use super::node_billboard::NodeBillboard;
use super::scene_paint::Camera;

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

/// Return the shared flat-square self-link presentation used by graph-only
/// callers that have no projected billboard. Canvas paint replaces this with
/// the live screen-space body geometry before rendering and picking.
fn self_link_points(
    center: Point2D<f32>,
    node_size: f32,
    stratum: LinkStratum,
) -> Vec<Point2D<f32>> {
    let size = node_size.max(1.0);
    let body = NodeBillboard {
        ground_anchor: center,
        center,
        local_side: size,
        zoom: 1.0,
        depth: 0.0,
        corner_radius: 0.0,
    };
    flat_self_link_points(body, stratum).unwrap_or_else(|| vec![center, center])
}

/// Reattach resting self-links to the current projected body. Returns the number
/// of candidates that needed the finite ground-foot fallback.
/// Self-links deliberately remain non-responsive and retain their canonical
/// appearance identity and stratum.
pub(crate) fn attach_self_link_geometry(
    segments: &mut [LinkAppearanceSegment],
    camera: &Camera,
    mut billboard_for: impl FnMut(NodeKey) -> Option<NodeBillboard>,
) -> usize {
    if !camera.is_valid() {
        return 0;
    }
    let mut attachment_fallbacks = 0;
    for segment in segments.iter_mut().filter(|segment| segment.is_self_link()) {
        segment.responsive = false;
        let Some(body) =
            billboard_for(segment.appearance.display_from).filter(|body| body.is_valid())
        else {
            continue;
        };
        let raised = (body.center.y - body.ground_anchor.y).abs() >= 0.5;
        let points = if !raised {
            flat_self_link_points(body, segment.appearance.key.stratum).or_else(|| {
                attachment_fallbacks += 1;
                Some(vec![body.ground_anchor, body.ground_anchor])
            })
        } else {
            match raised_self_link_points(body, segment.appearance.key.stratum) {
                Some(points) => Some(points),
                None => {
                    attachment_fallbacks += 1;
                    Some(vec![body.ground_anchor, body.ground_anchor])
                },
            }
        };
        let Some(screen_points) = points else {
            continue;
        };
        let world_points: Vec<_> = screen_points
            .iter()
            .map(|point| camera.to_world((point.x, point.y)))
            .collect();
        if world_points
            .iter()
            .any(|point| !point.x.is_finite() || !point.y.is_finite())
        {
            continue;
        }
        let Some(first) = world_points.first().copied() else {
            continue;
        };
        let Some(last) = world_points.last().copied() else {
            continue;
        };
        segment.from = Point2D::new(first.x, first.y);
        segment.to = Point2D::new(last.x, last.y);
        segment.points = world_points
            .into_iter()
            .map(|point| Point2D::new(point.x, point.y))
            .collect();
    }
    attachment_fallbacks
}

fn flat_self_link_points(body: NodeBillboard, stratum: LinkStratum) -> Option<Vec<Point2D<f32>>> {
    const ARC_SAMPLES: usize = 16;
    let (start_angle, end_angle) = match stratum {
        LinkStratum::Surface => (
            -3.0 * std::f32::consts::FRAC_PI_4 + 0.18,
            -std::f32::consts::FRAC_PI_2 - 0.18,
        ),
        LinkStratum::Resource => (
            -std::f32::consts::FRAC_PI_2 + 0.18,
            -std::f32::consts::FRAC_PI_4 - 0.18,
        ),
    };
    let start_dir = Point2D::new(start_angle.cos(), start_angle.sin());
    let end_dir = Point2D::new(end_angle.cos(), end_angle.sin());
    let start = body.boundary_point_toward(start_dir)?;
    let end = body.boundary_point_toward(end_dir)?;
    let half = body.local_side * body.zoom * 0.5;
    let outer_radius = half * 1.65 + 8.0;
    if !outer_radius.is_finite() || outer_radius <= half {
        return None;
    }
    let outer_start = Point2D::new(
        body.center.x + start_dir.x * outer_radius,
        body.center.y + start_dir.y * outer_radius,
    );
    let outer_end = Point2D::new(
        body.center.x + end_dir.x * outer_radius,
        body.center.y + end_dir.y * outer_radius,
    );
    let mut points = Vec::with_capacity(ARC_SAMPLES + 4);
    points.push(start);
    points.push(outer_start);
    for index in 1..ARC_SAMPLES {
        let amount = index as f32 / (ARC_SAMPLES - 1) as f32;
        let angle = start_angle + (end_angle - start_angle) * amount;
        points.push(Point2D::new(
            body.center.x + angle.cos() * outer_radius,
            body.center.y + angle.sin() * outer_radius,
        ));
    }
    points.push(outer_end);
    points.push(end);
    points
        .iter()
        .all(|point| point.x.is_finite() && point.y.is_finite())
        .then_some(points)
}

fn raised_self_link_points(body: NodeBillboard, stratum: LinkStratum) -> Option<Vec<Point2D<f32>>> {
    const SAMPLES: usize = 32;
    const INTERIOR_EPSILON: f32 = 0.05;
    let half = body.local_side * body.zoom * 0.5;
    let lift = body.ground_anchor.y - body.center.y;
    let radius = (half + lift.abs() + 8.0).max(half * 2.0);
    if !radius.is_finite() || radius <= 0.0 {
        return None;
    }
    let direction = if stratum == LinkStratum::Surface {
        -1.0
    } else {
        1.0
    };
    let points: Vec<_> = (0..=SAMPLES)
        .map(|index| {
            if index == 0 || index == SAMPLES {
                return body.ground_anchor;
            }
            let t = index as f32 / SAMPLES as f32;
            let angle = std::f32::consts::PI * t;
            let sine = angle.sin();
            Point2D::new(
                body.ground_anchor.x + direction * radius * sine,
                body.ground_anchor.y + radius * sine * (1.0 + 0.5 * angle.cos()),
            )
        })
        .collect();
    if points
        .iter()
        .any(|point| !point.x.is_finite() || !point.y.is_finite())
    {
        return None;
    }
    // The two endpoint runs may cross the lifted body because the visible stem
    // meets the ground anchor. The exposed loop between the first and last
    // strictly outside samples must never re-enter the body silhouette.
    let outside: Vec<_> = points
        .iter()
        .enumerate()
        .filter_map(|(index, point)| {
            (body.signed_distance_screen(*point)? >= -INTERIOR_EPSILON).then_some(index)
        })
        .collect();
    let first_outside = *outside.first()?;
    let last_outside = *outside.last()?;
    if first_outside >= last_outside {
        return None;
    }
    for pair in points[first_outside..=last_outside].windows(2) {
        let Some((enter, exit)) = body.segment_interior_interval(pair[0], pair[1]) else {
            continue;
        };
        if exit - enter <= f32::EPSILON {
            continue;
        }
        let midpoint = Point2D::new(
            pair[0].x + (pair[1].x - pair[0].x) * (enter + exit) * 0.5,
            pair[0].y + (pair[1].y - pair[0].y) * (enter + exit) * 0.5,
        );
        if body
            .signed_distance_screen(midpoint)
            .is_some_and(|distance| distance < -INTERIOR_EPSILON)
        {
            return None;
        }
    }
    Some(points)
}

#[cfg(test)]
#[path = "edge_cells/self_link_tests.rs"]
mod self_link_tests;

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

fn polyline_distance(
    point: Point2D<f32>,
    points: &[Point2D<f32>],
    project: &impl Fn(Point2D<f32>) -> Point2D<f32>,
) -> f32 {
    points
        .windows(2)
        .map(|pair| point_segment_distance(point, project(pair[0]), project(pair[1])))
        .fold(f32::INFINITY, f32::min)
}

fn polyline_intersects_box(
    points: &[Point2D<f32>],
    region: Box2D<f32>,
    project: &impl Fn(Point2D<f32>) -> Point2D<f32>,
) -> bool {
    points
        .windows(2)
        .any(|pair| segment_intersects_box(project(pair[0]), project(pair[1]), region))
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
    link_hit_test_segments_mapped(segments, point, tolerance, node_visible, include, |point| {
        point
    })
}

/// Project samples into the query's coordinate system without allocating a
/// second path list. Pointer input uses screen pixels for a uniform hit tolerance.
pub(crate) fn link_hit_test_segments_mapped(
    segments: Vec<LinkAppearanceSegment>,
    point: Point2D<f32>,
    tolerance: f32,
    node_visible: impl Fn(NodeKey) -> bool,
    include: impl Fn(&LinkAppearanceSegment) -> bool,
    project: impl Fn(Point2D<f32>) -> Point2D<f32>,
) -> Option<LinkAppearance> {
    segments
        .into_iter()
        .filter(|segment| {
            include(segment)
                && node_visible(segment.appearance.display_from)
                && node_visible(segment.appearance.display_to)
        })
        .filter_map(|segment| {
            let distance = polyline_distance(point, &segment.points, &project);
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
    links_in_rect_segments_mapped(segments, region, node_visible, |point| point)
}

pub(crate) fn links_in_rect_segments_mapped(
    segments: Vec<LinkAppearanceSegment>,
    region: Box2D<f32>,
    node_visible: impl Fn(NodeKey) -> bool,
    project: impl Fn(Point2D<f32>) -> Point2D<f32>,
) -> Vec<LinkAppearance> {
    segments
        .into_iter()
        .filter(|segment| {
            node_visible(segment.appearance.display_from)
                && node_visible(segment.appearance.display_to)
                && polyline_intersects_box(&segment.points, region, &project)
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
