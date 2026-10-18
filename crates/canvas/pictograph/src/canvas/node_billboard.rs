// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Shared screen-space geometry for the painted outer node body.
//!
//! The body is an upright CSS billboard. It is distinct from inset face pixels,
//! physics colliders, and custom sprite hulls. `ground_anchor` stays on the
//! projected graph plane; `center` includes the current visual lift.

use super::Canvas;
use euclid::default::{Box2D, Point2D};
use kernel::geometry::PortablePoint;
use kernel::graph::NodeKey;

/// The current projected silhouette of one painted graph node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NodeBillboard {
    /// The projected graph-plane position, also the foot of a raised stem.
    pub ground_anchor: Point2D<f32>,
    /// The visual body center in screen pixels, including the height lift.
    pub center: Point2D<f32>,
    /// CSS body side before camera zoom (includes grammar suggestion scale).
    pub local_side: f32,
    /// Camera zoom applied to `local_side` and `corner_radius`.
    pub zoom: f32,
    /// Projected ground depth used for painter ordering.
    pub depth: f32,
    /// Effective CSS corner radius in local pixels, clamped to half the side.
    /// Zero is square and half-side is circular.
    pub corner_radius: f32,
}

impl NodeBillboard {
    /// Whether every stored input and derived screen-space extent is finite and
    /// consistent. Public fields let hosts inspect or construct this value, so
    /// query methods validate it before using values in floating-point clamps.
    pub fn is_valid(self) -> bool {
        let side = self.local_side * self.zoom;
        let half = side * 0.5;
        let screen_radius = self.corner_radius * self.zoom;
        self.ground_anchor.x.is_finite()
            && self.ground_anchor.y.is_finite()
            && self.center.x.is_finite()
            && self.center.y.is_finite()
            && self.local_side.is_finite()
            && self.local_side > 0.0
            && self.zoom.is_finite()
            && self.zoom > 0.0
            && self.depth.is_finite()
            && self.corner_radius.is_finite()
            && self.corner_radius >= 0.0
            && self.corner_radius <= self.local_side * 0.5
            && side.is_finite()
            && side > 0.0
            && half.is_finite()
            && screen_radius.is_finite()
            && screen_radius >= 0.0
            && screen_radius <= half
            && (self.center.x - half).is_finite()
            && (self.center.y - half).is_finite()
            && (self.center.x + half).is_finite()
            && (self.center.y + half).is_finite()
    }

    /// Outer axis-aligned screen rectangle of the upright body.
    pub fn screen_rect(self) -> Box2D<f32> {
        let half = self.local_side * self.zoom * 0.5;
        Box2D::new(
            Point2D::new(self.center.x - half, self.center.y - half),
            Point2D::new(self.center.x + half, self.center.y + half),
        )
    }

    /// A centered square inside the body, expressed as a fraction of its CSS side.
    /// This describes face paint, not the outer body silhouette.
    pub fn inset_screen_rect(self, factor: f32) -> Box2D<f32> {
        let safe_factor = if factor.is_finite() {
            factor.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let half = self.local_side * self.zoom * safe_factor * 0.5;
        Box2D::new(
            Point2D::new(self.center.x - half, self.center.y - half),
            Point2D::new(self.center.x + half, self.center.y + half),
        )
    }

    /// Whether a screen point lies within the inclusive rounded-square silhouette.
    pub fn contains_screen(self, point: Point2D<f32>) -> bool {
        if !self.is_valid() || !point.x.is_finite() || !point.y.is_finite() {
            return false;
        }
        let side = self.local_side * self.zoom;
        let radius = self.corner_radius * self.zoom;
        rounded_square_contains(self.center, side * 0.5, radius, point)
    }

    /// Signed distance from a screen point to the outer silhouette, in pixels.
    /// Values are negative inside, zero on the boundary, and positive outside.
    pub fn signed_distance_screen(self, point: Point2D<f32>) -> Option<f32> {
        if !self.is_valid() || !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        let half = self.local_side * self.zoom * 0.5;
        let radius = self.corner_radius * self.zoom;
        let qx = (point.x - self.center.x).abs() - (half - radius);
        let qy = (point.y - self.center.y).abs() - (half - radius);
        let outside = qx.max(0.0).hypot(qy.max(0.0));
        let distance = outside + qx.max(qy).min(0.0) - radius;
        distance.is_finite().then_some(distance)
    }

    /// Inclusive parameter interval where segment `a + t*(b-a)` meets the body.
    /// A tangent may return `Some((t, t))`; callers requiring positive interior
    /// should reject intervals whose length is within their geometric epsilon.
    pub fn segment_interior_interval(self, a: Point2D<f32>, b: Point2D<f32>) -> Option<(f32, f32)> {
        if !self.is_valid()
            || !a.x.is_finite()
            || !a.y.is_finite()
            || !b.x.is_finite()
            || !b.y.is_finite()
        {
            return None;
        }
        let side = self.local_side * self.zoom;
        let half = side * 0.5;
        let radius = self.corner_radius * self.zoom;
        let local_a = Point2D::new(a.x - self.center.x, a.y - self.center.y);
        let local_b = Point2D::new(b.x - self.center.x, b.y - self.center.y);
        if !local_a.x.is_finite()
            || !local_a.y.is_finite()
            || !local_b.x.is_finite()
            || !local_b.y.is_finite()
        {
            return None;
        }
        rounded_square_segment_interval(local_a, local_b, half, radius)
    }

    /// First silhouette boundary point from the body center along `direction`.
    /// The ray is represented as a finite segment ending beyond the billboard;
    /// the canonical rounded-square interval supplies the boundary parameter.
    pub fn boundary_point_toward(self, direction: Point2D<f32>) -> Option<Point2D<f32>> {
        if !self.is_valid() || !direction.x.is_finite() || !direction.y.is_finite() {
            return None;
        }
        let length = direction.x.hypot(direction.y);
        if !length.is_finite() || length <= f32::EPSILON {
            return None;
        }
        let side = self.local_side * self.zoom;
        let distance = f64::from(side) * 2.0;
        let dx = f64::from(direction.x) / f64::from(length) * distance;
        let dy = f64::from(direction.y) / f64::from(length) * distance;
        let end_x = f64::from(self.center.x) + dx;
        let end_y = f64::from(self.center.y) + dy;
        if !end_x.is_finite()
            || !end_y.is_finite()
            || end_x.abs() > f64::from(f32::MAX)
            || end_y.abs() > f64::from(f32::MAX)
        {
            return None;
        }
        let end = Point2D::new(end_x as f32, end_y as f32);
        let (_, exit) = self.segment_interior_interval(self.center, end)?;
        let x = f64::from(self.center.x)
            + (f64::from(end.x) - f64::from(self.center.x)) * f64::from(exit);
        let y = f64::from(self.center.y)
            + (f64::from(end.y) - f64::from(self.center.y)) * f64::from(exit);
        if !x.is_finite()
            || !y.is_finite()
            || x.abs() > f64::from(f32::MAX)
            || y.abs() > f64::from(f32::MAX)
        {
            return None;
        }
        Some(Point2D::new(x as f32, y as f32))
    }
}

impl Canvas {
    /// Resolve the live graph node's projected outer body from current graph and
    /// layout state. Returns `None` when the node has no current position or the
    /// current camera/size inputs cannot be projected safely.
    pub fn node_billboard(&self, key: NodeKey) -> Option<NodeBillboard> {
        let pos = self.view.position_of(key)?;
        self.node_billboard_at(key, PortablePoint::new(pos.x, pos.y))
    }

    /// Resolve one node body at an explicit current layout position. This is
    /// shared by frame layout and host-side presentation inspection.
    pub(crate) fn node_billboard_at(
        &self,
        key: NodeKey,
        pos: PortablePoint,
    ) -> Option<NodeBillboard> {
        if self.graph.get_node(key).is_none() || !self.camera.is_valid() {
            return None;
        }
        let zoom = self.camera.zoom;
        let grammar_scale = if self.graph_grammar {
            self.grammar_face_scale(key)
        } else {
            1.0
        };
        let local_side = self.node_size(key) * grammar_scale;
        let lift = self.node_height(key) * zoom;
        let screen_side = local_side * zoom;
        let (x, y) = self.camera.to_screen(pos);
        let (s, c) = self.camera.yaw.sin_cos();
        let depth = (pos.x * s + pos.y * c).round();
        let center_y = y - lift;
        let half_screen = screen_side * 0.5;
        let rect_values = [
            x - half_screen,
            y - half_screen,
            x + half_screen,
            y + half_screen,
            center_y - half_screen,
            center_y + half_screen,
        ];
        if !pos.x.is_finite()
            || !pos.y.is_finite()
            || !x.is_finite()
            || !y.is_finite()
            || !depth.is_finite()
            || !local_side.is_finite()
            || local_side <= 0.0
            || !screen_side.is_finite()
            || screen_side <= 0.0
            || !lift.is_finite()
            || !center_y.is_finite()
            || rect_values.iter().any(|value| !value.is_finite())
        {
            return None;
        }
        let radius = self.node_effective_local_corner_radius(key, local_side);
        if !radius.is_finite() {
            return None;
        }
        let billboard = NodeBillboard {
            ground_anchor: Point2D::new(x, y),
            center: Point2D::new(x, center_y),
            local_side,
            zoom,
            depth,
            corner_radius: radius.clamp(0.0, local_side * 0.5),
        };
        billboard.is_valid().then_some(billboard)
    }
}

fn rounded_square_contains(
    center: Point2D<f32>,
    half: f32,
    radius: f32,
    point: Point2D<f32>,
) -> bool {
    let dx = (point.x - center.x).abs();
    let dy = (point.y - center.y).abs();
    if dx > half || dy > half {
        return false;
    }
    let corner_x = (half - radius).max(0.0);
    let corner_y = (half - radius).max(0.0);
    let ox = (dx - corner_x).max(0.0);
    let oy = (dy - corner_y).max(0.0);
    ox.hypot(oy) <= radius
}

fn rounded_square_segment_interval(
    a: Point2D<f32>,
    b: Point2D<f32>,
    half: f32,
    radius: f32,
) -> Option<(f32, f32)> {
    let delta = Point2D::new(b.x - a.x, b.y - a.y);
    if !delta.x.is_finite() || !delta.y.is_finite() {
        return None;
    }
    let inner = half - radius;
    let mut intervals = [(0.0_f32, 0.0_f32); 6];
    let mut count = 0;

    if radius <= f32::EPSILON {
        add_interval(
            &mut intervals,
            &mut count,
            rect_interval(a, delta, -half, -half, half, half),
        );
    } else {
        // The rounded square is the union of its two central rectangular strips
        // and four quarter-disks. Each primitive has constant bounded work.
        add_interval(
            &mut intervals,
            &mut count,
            rect_interval(a, delta, -inner, -half, inner, half),
        );
        add_interval(
            &mut intervals,
            &mut count,
            rect_interval(a, delta, -half, -inner, half, inner),
        );
        for sx in [-1.0_f32, 1.0] {
            for sy in [-1.0_f32, 1.0] {
                let center = Point2D::new(sx * inner, sy * inner);
                let disk = disk_interval(a, delta, center, radius);
                if let Some((lo, hi)) = disk {
                    let quadrant = rect_interval(
                        a,
                        delta,
                        if sx < 0.0 { f32::NEG_INFINITY } else { inner },
                        if sy < 0.0 { f32::NEG_INFINITY } else { inner },
                        if sx < 0.0 { -inner } else { f32::INFINITY },
                        if sy < 0.0 { -inner } else { f32::INFINITY },
                    );
                    if let Some((qlo, qhi)) = quadrant {
                        if let Some(hit) = intersect_intervals((lo, hi), (qlo, qhi)) {
                            add_interval(&mut intervals, &mut count, Some(hit));
                        }
                    }
                }
            }
        }
    }
    if count == 0 {
        return None;
    }
    // A rounded square is convex, so the union of primitive hit intervals has
    // one connected span. Taking the extrema avoids a second merge algorithm.
    let lo = intervals[..count]
        .iter()
        .map(|interval| interval.0)
        .fold(f32::INFINITY, f32::min);
    let hi = intervals[..count]
        .iter()
        .map(|interval| interval.1)
        .fold(f32::NEG_INFINITY, f32::max);
    Some((lo.clamp(0.0, 1.0), hi.clamp(0.0, 1.0)))
}

fn add_interval<const N: usize>(
    intervals: &mut [(f32, f32); N],
    count: &mut usize,
    interval: Option<(f32, f32)>,
) {
    if let Some(interval) = interval {
        if *count < N {
            intervals[*count] = interval;
            *count += 1;
        }
    }
}

fn intersect_intervals(a: (f32, f32), b: (f32, f32)) -> Option<(f32, f32)> {
    let interval = (a.0.max(b.0), a.1.min(b.1));
    (interval.0 <= interval.1).then_some(interval)
}

fn rect_interval(
    a: Point2D<f32>,
    delta: Point2D<f32>,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
) -> Option<(f32, f32)> {
    let mut interval = (0.0_f32, 1.0_f32);
    for (origin, change, min, max) in [(a.x, delta.x, min_x, max_x), (a.y, delta.y, min_y, max_y)] {
        if change.abs() <= f32::EPSILON {
            if origin < min || origin > max {
                return None;
            }
            continue;
        }
        let entry = (min - origin) / change;
        let exit = (max - origin) / change;
        let axis = (entry.min(exit), entry.max(exit));
        interval.0 = interval.0.max(axis.0);
        interval.1 = interval.1.min(axis.1);
        if interval.0 > interval.1 {
            return None;
        }
    }
    intersect_intervals(interval, (0.0, 1.0))
}

fn disk_interval(
    a: Point2D<f32>,
    delta: Point2D<f32>,
    center: Point2D<f32>,
    radius: f32,
) -> Option<(f32, f32)> {
    // Solve in f64 so large endpoint separations and small radii do not overflow
    // or lose the discriminant to f32 cancellation.
    let x = f64::from(a.x - center.x);
    let y = f64::from(a.y - center.y);
    let dx = f64::from(delta.x);
    let dy = f64::from(delta.y);
    let radius = f64::from(radius);
    let aa = dx * dx + dy * dy;
    let bb = 2.0 * (x * dx + y * dy);
    let cc = x * x + y * y - radius * radius;
    if aa <= f64::EPSILON {
        return (cc <= 0.0).then_some((0.0, 1.0));
    }
    let discriminant = bb * bb - 4.0 * aa * cc;
    if discriminant < 0.0 {
        return None;
    }
    let root = discriminant.max(0.0).sqrt();
    intersect_intervals(
        (
            ((-bb - root) / (2.0 * aa)) as f32,
            ((-bb + root) / (2.0 * aa)) as f32,
        ),
        (0.0, 1.0),
    )
}

#[cfg(test)]
#[path = "node_billboard_tests.rs"]
mod tests;
