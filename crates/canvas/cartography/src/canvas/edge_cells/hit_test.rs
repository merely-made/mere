// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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

pub(super) fn link_hit_test_where(
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
