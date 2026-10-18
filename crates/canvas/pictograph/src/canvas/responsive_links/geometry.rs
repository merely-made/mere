// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//!
//! Bounded presentation geometry for evaluated responsive paths.

use euclid::default::Point2D;
use kernel::geometry::PortablePoint;

use super::super::node_billboard::NodeBillboard;
use super::super::scene_paint::Camera;
use super::edge_cells::LinkAppearanceSegment;

const CORNER_CUT_PASSES: usize = 2;
const MAX_EVALUATOR_POINTS: usize = numen::MAX_CONTROL_POINTS;
const MAX_PRESENTATION_POINTS: usize = 128;
const MIN_VISIBLE_LENGTH: f32 = 1.0e-3;
const OFFSET_TOLERANCE: f32 = 1.0e-3;
const ANCHOR_TOLERANCE: f32 = 1.0e-2;
const INTERIOR_EPSILON: f32 = 1.0e-5;
const MIN_TAPER_LENGTH: f32 = 1.0;

/// Smooth and attach one evaluated polyline to the rendered endpoint bodies.
/// Evaluator samples and the stratum translation are world-space; smoothing,
/// silhouette clipping, and the lifted-node stem policy are screen-space.
/// `None` means the caller retains its resting path for this appearance.
pub(super) fn responsive_path_geometry(
    samples: &[(f32, f32)],
    resting: &LinkAppearanceSegment,
    camera: Camera,
    from_body: NodeBillboard,
    to_body: NodeBillboard,
) -> Option<Vec<Point2D<f32>>> {
    if !(2..=MAX_EVALUATOR_POINTS).contains(&samples.len())
        || !camera.is_valid()
        || !from_body.is_valid()
        || !to_body.is_valid()
        || samples
            .iter()
            .any(|(x, y)| !x.is_finite() || !y.is_finite())
        || !finite_point(resting.from)
        || !finite_point(resting.to)
    {
        return None;
    }

    let last = samples.len() - 1;
    let start_offset = (resting.from.x - samples[0].0, resting.from.y - samples[0].1);
    let end_offset = (
        resting.to.x - samples[last].0,
        resting.to.y - samples[last].1,
    );
    if !finite_pair(start_offset)
        || !finite_pair(end_offset)
        || (start_offset.0 - end_offset.0).abs() > OFFSET_TOLERANCE
        || (start_offset.1 - end_offset.1).abs() > OFFSET_TOLERANCE
    {
        return None;
    }

    let start_anchor = screen_point(camera, samples[0])?;
    let end_anchor = screen_point(camera, samples[last])?;
    if point_distance(start_anchor, from_body.ground_anchor) > ANCHOR_TOLERANCE
        || point_distance(end_anchor, to_body.ground_anchor) > ANCHOR_TOLERANCE
    {
        return None;
    }

    let bodies_overlap = rects_overlap(from_body.screen_rect(), to_body.screen_rect());
    if bodies_overlap {
        return None;
    }

    let mut points: Vec<_> = samples
        .iter()
        .map(|&(x, y)| {
            let world = PortablePoint::new(x + start_offset.0, y + start_offset.1);
            let (sx, sy) = camera.to_screen(world);
            Point2D::new(sx, sy)
        })
        .collect();
    if points.iter().any(|point| !finite_point(*point)) {
        return None;
    }

    let from_stem = has_visible_stem(from_body);
    let to_stem = has_visible_stem(to_body);
    let lane_offset = start_offset.0.abs() > f32::EPSILON
        || start_offset.1.abs() > f32::EPSILON
        || end_offset.0.abs() > f32::EPSILON
        || end_offset.1.abs() > f32::EPSILON;
    let taper_from = from_stem || lane_offset;
    let taper_to = to_stem || lane_offset;
    if taper_from || taper_to {
        let offset_world = (
            (start_offset.0 + end_offset.0) * 0.5,
            (start_offset.1 + end_offset.1) * 0.5,
        );
        let shifted_start = screen_point(
            camera,
            (samples[0].0 + offset_world.0, samples[0].1 + offset_world.1),
        )?;
        let offset_screen = Point2D::new(
            shifted_start.x - start_anchor.x,
            shifted_start.y - start_anchor.y,
        );
        taper_endpoint_offsets(
            &mut points,
            offset_screen,
            taper_from,
            taper_to,
            from_body,
            to_body,
        )?;
        if taper_from {
            points[0] = from_body.ground_anchor;
        }
        if taper_to {
            *points.last_mut()? = to_body.ground_anchor;
        }
    }

    for _ in 0..CORNER_CUT_PASSES {
        points = corner_cut(&points)?;
    }
    if points.len() > MAX_PRESENTATION_POINTS {
        return None;
    }

    // Raised gnodes visibly connect through their stem feet. Flat endpoints
    // attach to their actual rounded-square billboard silhouette.
    if from_stem {
        if !safe_stem_attachment(&points, from_body) {
            return None;
        }
    } else if !clip_to_body(&mut points, from_body) {
        return None;
    }
    points.reverse();
    if to_stem {
        if !safe_stem_attachment(&points, to_body) {
            points.reverse();
            return None;
        }
    } else if !clip_to_body(&mut points, to_body) {
        points.reverse();
        return None;
    }
    points.reverse();

    let screen_length = polyline_length(&points);
    if points.len() < 2
        || points.iter().any(|point| !finite_point(*point))
        || !screen_length.is_finite()
        || screen_length <= MIN_VISIBLE_LENGTH
    {
        return None;
    }

    let world_points: Vec<_> = points
        .iter()
        .map(|point| {
            let world = camera.to_world((point.x, point.y));
            Point2D::new(world.x, world.y)
        })
        .collect();
    if world_points.iter().any(|point| !finite_point(*point)) {
        return None;
    }
    Some(world_points)
}

fn has_visible_stem(body: NodeBillboard) -> bool {
    (body.center.y - body.ground_anchor.y).abs() >= 0.5
}

fn taper_endpoint_offsets(
    points: &mut [Point2D<f32>],
    screen_offset: Point2D<f32>,
    from_stem: bool,
    to_stem: bool,
    from_body: NodeBillboard,
    to_body: NodeBillboard,
) -> Option<()> {
    let mut cumulative = Vec::with_capacity(points.len());
    cumulative.push(0.0);
    for pair in points.windows(2) {
        let next = cumulative.last().copied()? + point_distance(pair[0], pair[1]);
        if !next.is_finite() {
            return None;
        }
        cumulative.push(next);
    }
    let total = *cumulative.last()?;
    if total <= f32::EPSILON {
        return None;
    }
    let from_length = (from_body.local_side * from_body.zoom).max(MIN_TAPER_LENGTH);
    let to_length = (to_body.local_side * to_body.zoom).max(MIN_TAPER_LENGTH);
    if !from_length.is_finite() || !to_length.is_finite() {
        return None;
    }
    for (index, point) in points.iter_mut().enumerate() {
        let mut retained = 1.0_f32;
        if from_stem {
            retained = retained.min((cumulative[index] / from_length).clamp(0.0, 1.0));
        }
        if to_stem {
            retained = retained.min(((total - cumulative[index]) / to_length).clamp(0.0, 1.0));
        }
        point.x -= screen_offset.x * (1.0 - retained);
        point.y -= screen_offset.y * (1.0 - retained);
    }
    Some(())
}

fn clip_to_body(points: &mut Vec<Point2D<f32>>, body: NodeBillboard) -> bool {
    if points.len() < 2 || !body.contains_screen(points[0]) {
        return false;
    }
    let Some(outside_index) = points
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, point)| (!body.contains_screen(*point)).then_some(index))
    else {
        return false;
    };
    let Some((_, exit)) =
        body.segment_interior_interval(points[outside_index - 1], points[outside_index])
    else {
        return false;
    };
    let crossing = interpolate(points[outside_index - 1], points[outside_index], exit);
    if !finite_point(crossing) {
        return false;
    }
    points.drain(..outside_index);
    points.insert(0, crossing);
    !has_strict_body_reentry(points, body, 1)
}

/// Keep the ground-anchor endpoint for nodes with a rendered stem. If the foot
/// lies inside the raised body, allow its initial hidden run to exit the body;
/// any later strict reentry makes the candidate unsafe and falls back to rest.
fn safe_stem_attachment(points: &[Point2D<f32>], body: NodeBillboard) -> bool {
    if points.len() < 2 {
        return false;
    }
    let skip = if body.contains_screen(points[0]) {
        let Some(outside_index) = points
            .iter()
            .enumerate()
            .skip(1)
            .find_map(|(index, point)| (!body.contains_screen(*point)).then_some(index))
        else {
            return false;
        };
        outside_index
    } else {
        0
    };
    !has_strict_body_reentry(points, body, skip)
}

/// Inspect segment interiors after `skip` leading segments. Boundary contact
/// and a zero-width tangent do not count as entering the painted silhouette.
fn has_strict_body_reentry(points: &[Point2D<f32>], body: NodeBillboard, skip: usize) -> bool {
    points.windows(2).skip(skip).any(|pair| {
        body.segment_interior_interval(pair[0], pair[1])
            .is_some_and(|(enter, exit)| {
                if exit - enter <= INTERIOR_EPSILON {
                    return false;
                }
                let midpoint = interpolate(pair[0], pair[1], (enter + exit) * 0.5);
                body.signed_distance_screen(midpoint)
                    .is_some_and(|distance| distance < -INTERIOR_EPSILON)
            })
    })
}

fn corner_cut(points: &[Point2D<f32>]) -> Option<Vec<Point2D<f32>>> {
    if points.len() < 2 {
        return None;
    }
    let next_len = points.len().checked_mul(2)?;
    if next_len > MAX_PRESENTATION_POINTS {
        return None;
    }
    let mut smoothed = Vec::with_capacity(next_len);
    smoothed.push(points[0]);
    for pair in points.windows(2) {
        let a = pair[0];
        let b = pair[1];
        smoothed.push(Point2D::new(
            a.x * 0.75 + b.x * 0.25,
            a.y * 0.75 + b.y * 0.25,
        ));
        smoothed.push(Point2D::new(
            a.x * 0.25 + b.x * 0.75,
            a.y * 0.25 + b.y * 0.75,
        ));
    }
    smoothed.push(*points.last()?);
    smoothed
        .iter()
        .all(|point| finite_point(*point))
        .then_some(smoothed)
}

fn rects_overlap(first: euclid::default::Box2D<f32>, second: euclid::default::Box2D<f32>) -> bool {
    first.min.x <= second.max.x
        && second.min.x <= first.max.x
        && first.min.y <= second.max.y
        && second.min.y <= first.max.y
}

fn screen_point(camera: Camera, point: (f32, f32)) -> Option<Point2D<f32>> {
    let (x, y) = camera.to_screen(PortablePoint::new(point.0, point.1));
    let point = Point2D::new(x, y);
    finite_point(point).then_some(point)
}

fn interpolate(from: Point2D<f32>, to: Point2D<f32>, amount: f32) -> Point2D<f32> {
    from + (to - from) * amount.clamp(0.0, 1.0)
}

fn point_distance(first: Point2D<f32>, second: Point2D<f32>) -> f32 {
    (second - first).length()
}

fn polyline_length(points: &[Point2D<f32>]) -> f32 {
    points
        .windows(2)
        .map(|pair| point_distance(pair[0], pair[1]))
        .sum()
}

fn finite_pair(pair: (f32, f32)) -> bool {
    pair.0.is_finite() && pair.1.is_finite()
}

fn finite_point(point: Point2D<f32>) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

#[cfg(test)]
#[path = "geometry_tests.rs"]
mod tests;
