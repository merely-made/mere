// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//!
//! Bounded presentation geometry for evaluated responsive paths.

use euclid::default::Point2D;

use super::edge_cells::LinkAppearanceSegment;

const CORNER_CUT_PASSES: usize = 2;
const MAX_EVALUATOR_POINTS: usize = numen::MAX_CONTROL_POINTS;
const MAX_PRESENTATION_POINTS: usize = 128;
const MIN_VISIBLE_LENGTH: f32 = 1.0e-3;
const OFFSET_TOLERANCE: f32 = 1.0e-3;

/// Smooth and clip one evaluated polyline while preserving its resting stratum
/// translation. `None` means the caller should keep the resting segment as-is.
pub(super) fn responsive_path_geometry(
    samples: &[(f32, f32)],
    resting: &LinkAppearanceSegment,
    from_size: f32,
    to_size: f32,
) -> Option<Vec<Point2D<f32>>> {
    if !(2..=MAX_EVALUATOR_POINTS).contains(&samples.len())
        || !from_size.is_finite()
        || !to_size.is_finite()
        || from_size < 0.0
        || to_size < 0.0
        || samples
            .iter()
            .any(|(x, y)| !x.is_finite() || !y.is_finite())
        || !finite_point(resting.from)
        || !finite_point(resting.to)
    {
        return None;
    }

    let start_offset = (resting.from.x - samples[0].0, resting.from.y - samples[0].1);
    let last = samples.len() - 1;
    let end_offset = (
        resting.to.x - samples[last].0,
        resting.to.y - samples[last].1,
    );
    if !start_offset.0.is_finite()
        || !start_offset.1.is_finite()
        || !end_offset.0.is_finite()
        || !end_offset.1.is_finite()
        || (start_offset.0 - end_offset.0).abs() > OFFSET_TOLERANCE
        || (start_offset.1 - end_offset.1).abs() > OFFSET_TOLERANCE
    {
        return None;
    }

    let mut points: Vec<_> = samples
        .iter()
        .map(|&(x, y)| Point2D::new(x + start_offset.0, y + start_offset.1))
        .collect();
    if points.iter().any(|point| !finite_point(*point)) {
        return None;
    }
    for _ in 0..CORNER_CUT_PASSES {
        points = corner_cut(&points)?;
    }
    if points.len() > MAX_PRESENTATION_POINTS {
        return None;
    }

    // Evaluator endpoints identify the actual unshifted node centers. The
    // appearance polyline carries a stratum offset, but that offset must not
    // move the faces that paint the nodes.
    let start_center = Point2D::new(samples[0].0, samples[0].1);
    let end_center = Point2D::new(samples[last].0, samples[last].1);
    let start_half = from_size * 0.5;
    let end_half = to_size * 0.5;
    if !start_half.is_finite()
        || !end_half.is_finite()
        || boxes_overlap(start_center, start_half, end_center, end_half)
    {
        return None;
    }

    if !clip_start_to_square(&mut points, start_center, start_half) {
        return None;
    }
    if polyline_reenters_square(&points, start_center, start_half) {
        return None;
    }
    points.reverse();
    let clipped_end = clip_start_to_square(&mut points, end_center, end_half);
    let end_reenters = clipped_end && polyline_reenters_square(&points, end_center, end_half);
    points.reverse();
    if !clipped_end || end_reenters || points.len() < 2 {
        return None;
    }
    let length = polyline_length(&points);
    if points.iter().any(|point| !finite_point(*point))
        || !length.is_finite()
        || length <= MIN_VISIBLE_LENGTH
    {
        return None;
    }

    Some(points)
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

fn clip_start_to_square(
    points: &mut Vec<Point2D<f32>>,
    center: Point2D<f32>,
    half_extent: f32,
) -> bool {
    if points.len() < 2 || !inside_square(points[0], center, half_extent) {
        return false;
    }
    let Some(outside_index) = points
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(index, point)| (!inside_square(*point, center, half_extent)).then_some(index))
    else {
        return false;
    };
    let Some(crossing) = square_exit(
        points[outside_index - 1],
        points[outside_index],
        center,
        half_extent,
    ) else {
        return false;
    };
    // Retain the first outside point. The crossing replaces the last inside
    // point so the actual turn immediately after the face remains represented.
    points.drain(..outside_index - 1);
    points[0] = crossing;
    true
}

/// Test every segment after the initial face crossing. Checking segments as
/// well as vertices catches a path that crosses a node between samples.
fn polyline_reenters_square(
    points: &[Point2D<f32>],
    center: Point2D<f32>,
    half_extent: f32,
) -> bool {
    points
        .windows(2)
        .skip(1)
        .any(|pair| segment_enters_square(pair[0], pair[1], center, half_extent))
}

/// Return true when a segment passes through the square's strict interior.
/// Boundary-only contact, including a corner touch, is allowed.
fn segment_enters_square(
    start: Point2D<f32>,
    end: Point2D<f32>,
    center: Point2D<f32>,
    half_extent: f32,
) -> bool {
    let min_x = center.x - half_extent;
    let max_x = center.x + half_extent;
    let min_y = center.y - half_extent;
    let max_y = center.y + half_extent;
    if ![min_x, max_x, min_y, max_y]
        .iter()
        .all(|value| value.is_finite())
    {
        return true;
    }

    let (dx, dy) = (end.x - start.x, end.y - start.y);
    let mut enter = 0.0_f32;
    let mut exit = 1.0_f32;
    for (origin, delta, low, high) in [(start.x, dx, min_x, max_x), (start.y, dy, min_y, max_y)] {
        if delta == 0.0 {
            if origin <= low || origin >= high {
                return false;
            }
            continue;
        }
        let a = (low - origin) / delta;
        let b = (high - origin) / delta;
        enter = enter.max(a.min(b));
        exit = exit.min(a.max(b));
        if enter >= exit {
            return false;
        }
    }
    if !enter.is_finite() || !exit.is_finite() || enter >= exit {
        return false;
    }
    let midpoint = (enter + exit) * 0.5;
    let point = Point2D::new(start.x + dx * midpoint, start.y + dy * midpoint);
    point.x > min_x && point.x < max_x && point.y > min_y && point.y < max_y
}

fn square_exit(
    inside: Point2D<f32>,
    outside: Point2D<f32>,
    center: Point2D<f32>,
    half_extent: f32,
) -> Option<Point2D<f32>> {
    let min_x = center.x - half_extent;
    let max_x = center.x + half_extent;
    let min_y = center.y - half_extent;
    let max_y = center.y + half_extent;
    if ![min_x, max_x, min_y, max_y]
        .iter()
        .all(|value| value.is_finite())
    {
        return None;
    }
    let dx = outside.x - inside.x;
    let dy = outside.y - inside.y;
    let mut crossing = 1.0_f32;
    let mut found = false;
    if outside.x < min_x && dx < 0.0 {
        crossing = crossing.min((min_x - inside.x) / dx);
        found = true;
    } else if outside.x > max_x && dx > 0.0 {
        crossing = crossing.min((max_x - inside.x) / dx);
        found = true;
    }
    if outside.y < min_y && dy < 0.0 {
        crossing = crossing.min((min_y - inside.y) / dy);
        found = true;
    } else if outside.y > max_y && dy > 0.0 {
        crossing = crossing.min((max_y - inside.y) / dy);
        found = true;
    }
    if !found || !crossing.is_finite() || !(0.0..=1.0).contains(&crossing) {
        return None;
    }
    let point = Point2D::new(inside.x + dx * crossing, inside.y + dy * crossing);
    finite_point(point).then_some(point)
}

fn inside_square(point: Point2D<f32>, center: Point2D<f32>, half_extent: f32) -> bool {
    point.x >= center.x - half_extent
        && point.x <= center.x + half_extent
        && point.y >= center.y - half_extent
        && point.y <= center.y + half_extent
}

fn boxes_overlap(
    first: Point2D<f32>,
    first_half: f32,
    second: Point2D<f32>,
    second_half: f32,
) -> bool {
    (first.x - second.x).abs() <= first_half + second_half
        && (first.y - second.y).abs() <= first_half + second_half
}

fn finite_point(point: Point2D<f32>) -> bool {
    point.x.is_finite() && point.y.is_finite()
}

fn polyline_length(points: &[Point2D<f32>]) -> f32 {
    points
        .windows(2)
        .map(|pair| (pair[1].x - pair[0].x).hypot(pair[1].y - pair[0].y))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::LinkAppearance;
    use kernel::graph::NodeKey;
    use scenograph::link::{LinkKey, LinkMarks, LinkStratum};

    fn segment(
        start: Point2D<f32>,
        end: Point2D<f32>,
        offset: (f32, f32),
    ) -> LinkAppearanceSegment {
        LinkAppearanceSegment {
            appearance: LinkAppearance {
                key: LinkKey::new(LinkStratum::Resource, "source:a", "source:b"),
                display_from: NodeKey::new(1),
                display_to: NodeKey::new(2),
                marks: LinkMarks::default(),
                strength: 1.0,
            },
            from: Point2D::new(start.x + offset.0, start.y + offset.1),
            to: Point2D::new(end.x + offset.0, end.y + offset.1),
            points: vec![start, end],
            responsive: false,
        }
    }

    fn xy(points: &[Point2D<f32>]) -> Vec<(f32, f32)> {
        points.iter().map(|point| (point.x, point.y)).collect()
    }

    fn max_turn(points: &[Point2D<f32>]) -> f32 {
        points
            .windows(3)
            .map(|triple| {
                let a = (triple[1].x - triple[0].x, triple[1].y - triple[0].y);
                let b = (triple[2].x - triple[1].x, triple[2].y - triple[1].y);
                let dot = a.0 * b.0 + a.1 * b.1;
                let cross = a.0 * b.1 - a.1 * b.0;
                cross.atan2(dot).abs()
            })
            .fold(0.0_f32, f32::max)
    }

    #[test]
    fn corner_cutting_reduces_a_sharp_curve_with_a_fixed_output_bound() {
        let samples = [
            (0.0, 0.0),
            (10.0, 45.0),
            (28.0, -22.0),
            (64.0, 88.0),
            (100.0, 60.0),
        ];
        let resting = segment(
            Point2D::new(0.0, 0.0),
            Point2D::new(100.0, 60.0),
            (0.0, 0.0),
        );
        let path = responsive_path_geometry(&samples, &resting, 8.0, 8.0).unwrap();
        let raw: Vec<_> = samples.iter().map(|&(x, y)| Point2D::new(x, y)).collect();
        assert!(path.len() <= MAX_PRESENTATION_POINTS);
        assert!(path.len() > samples.len());
        assert!(max_turn(&path) < max_turn(&raw));
        assert!(path.iter().all(|point| finite_point(*point)));
    }

    #[test]
    fn diagonal_path_clips_to_unequal_square_faces_and_keeps_stratum_offset() {
        let samples = [(0.0, 0.0), (20.0, 12.0), (50.0, 30.0), (100.0, 60.0)];
        let resting = segment(
            Point2D::new(0.0, 0.0),
            Point2D::new(100.0, 60.0),
            (3.0, -3.0),
        );
        let path = responsive_path_geometry(&samples, &resting, 20.0, 30.0).unwrap();
        let start = Point2D::new(0.0, 0.0);
        let end = Point2D::new(100.0, 60.0);
        assert!((path[0].x - 10.0).abs() < 1.0e-3);
        assert!(path[0].y >= -10.0 && path[0].y <= 10.0);
        assert!((path[path.len() - 1].x - 85.0).abs() < 1.0e-3);
        assert!(path[path.len() - 1].y >= 45.0 && path[path.len() - 1].y <= 75.0);
        // The start face is centered on the actual node at (0, 0), despite
        // this appearance's (3, -3) stratum translation.
        assert!((path[0].x - (start.x + 10.0)).abs() < 1.0e-3);
        assert!((path[path.len() - 1].x - (end.x - 15.0)).abs() < 1.0e-3);
        assert!(path.len() <= MAX_PRESENTATION_POINTS);
    }

    #[test]
    fn reversed_curve_and_unequal_sizes_are_symmetric() {
        let samples = [
            (0.0, 0.0),
            (18.0, 12.0),
            (43.0, 39.0),
            (72.0, 52.0),
            (100.0, 60.0),
        ];
        let forward_rest = segment(
            Point2D::new(0.0, 0.0),
            Point2D::new(100.0, 60.0),
            (2.0, 3.0),
        );
        let mut reverse_samples = samples;
        reverse_samples.reverse();
        let reverse_rest = segment(
            Point2D::new(100.0, 60.0),
            Point2D::new(0.0, 0.0),
            (2.0, 3.0),
        );
        let forward = responsive_path_geometry(&samples, &forward_rest, 14.0, 30.0).unwrap();
        let reverse =
            responsive_path_geometry(&reverse_samples, &reverse_rest, 30.0, 14.0).unwrap();
        let reverse_xy = xy(&reverse);
        for (a, b) in forward.iter().zip(reverse_xy.iter().rev()) {
            assert!((a.x - b.0).abs() < 1.0e-3);
            assert!((a.y - b.1).abs() < 1.0e-3);
        }
    }

    #[test]
    fn coincident_overlapping_and_short_paths_fall_back() {
        let coincident = [(20.0, 20.0), (20.0, 20.0), (20.0, 20.0)];
        let coincident_rest = segment(
            Point2D::new(20.0, 20.0),
            Point2D::new(20.0, 20.0),
            (0.0, 0.0),
        );
        assert!(responsive_path_geometry(&coincident, &coincident_rest, 10.0, 10.0).is_none());

        let short = [(0.0, 0.0), (2.0, 0.0), (4.0, 0.0)];
        let short_rest = segment(Point2D::new(0.0, 0.0), Point2D::new(4.0, 0.0), (0.0, 0.0));
        assert!(responsive_path_geometry(&short, &short_rest, 10.0, 10.0).is_none());

        let overlapping = [(0.0, 0.0), (11.0, 0.0), (22.0, 0.0)];
        let overlapping_rest = segment(Point2D::new(0.0, 0.0), Point2D::new(22.0, 0.0), (0.0, 0.0));
        assert!(responsive_path_geometry(&overlapping, &overlapping_rest, 24.0, 24.0).is_none());

        let separated = [(0.0, 0.0), (11.0, 0.0), (22.0, 0.0)];
        let separated_rest = segment(Point2D::new(0.0, 0.0), Point2D::new(22.0, 0.0), (0.0, 0.0));
        assert!(responsive_path_geometry(&separated, &separated_rest, 12.0, 12.0).is_some());

        let reentering = [
            (0.0, 0.0),
            (30.0, 0.0),
            (-30.0, 0.0),
            (42.0, 0.0),
            (100.0, 0.0),
        ];
        let reentering_rest = segment(Point2D::new(0.0, 0.0), Point2D::new(100.0, 0.0), (0.0, 0.0));
        assert!(
            responsive_path_geometry(&reentering, &reentering_rest, 12.0, 12.0).is_none(),
            "paths that re-enter an endpoint face must fall back to resting geometry"
        );

        let crosses_between_samples = [(0.0, 0.0), (30.0, 0.0), (-30.0, 0.0), (100.0, 0.0)];
        let crossing_rest = segment(Point2D::new(0.0, 0.0), Point2D::new(100.0, 0.0), (0.0, 0.0));
        assert!(segment_enters_square(
            Point2D::new(15.0, 0.0),
            Point2D::new(-15.0, 0.0),
            Point2D::new(0.0, 0.0),
            10.0,
        ));
        assert!(
            responsive_path_geometry(&crosses_between_samples, &crossing_rest, 20.0, 8.0).is_none(),
            "a segment crossing a node interior between sampled points must fall back"
        );
    }

    #[test]
    fn invalid_or_unbounded_samples_are_rejected() {
        let resting = segment(Point2D::new(0.0, 0.0), Point2D::new(100.0, 0.0), (0.0, 0.0));
        let nonfinite = vec![(0.0, 0.0), (50.0, f32::NAN), (100.0, 0.0)];
        assert!(responsive_path_geometry(&nonfinite, &resting, 8.0, 8.0).is_none());
        let oversized: Vec<_> = (0..=MAX_EVALUATOR_POINTS)
            .map(|index| (index as f32, 0.0))
            .collect();
        assert!(
            responsive_path_geometry(&oversized, &resting, 8.0, 8.0).is_none(),
            "more than the evaluator's bounded input sample count must be refused"
        );
    }
}
