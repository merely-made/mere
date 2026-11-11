// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// Return the shared flat-square self-link presentation used by graph-only
/// callers that have no projected billboard. Canvas paint replaces this with
/// the live screen-space body geometry before rendering and picking.
pub(super) fn self_link_points(
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

pub(super) fn flat_self_link_points(
    body: NodeBillboard,
    stratum: LinkStratum,
) -> Option<Vec<Point2D<f32>>> {
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

pub(super) fn raised_self_link_points(
    body: NodeBillboard,
    stratum: LinkStratum,
) -> Option<Vec<Point2D<f32>>> {
    const SAMPLES: usize = 32;
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
        .filter_map(|(index, point)| (!body.strictly_contains_screen(*point)).then_some(index))
        .collect();
    let first_outside = *outside.first()?;
    let last_outside = *outside.last()?;
    if first_outside >= last_outside {
        return None;
    }
    for pair in points[first_outside..=last_outside].windows(2) {
        if body.segment_crosses_interior(pair[0], pair[1]) {
            return None;
        }
    }
    Some(points)
}
