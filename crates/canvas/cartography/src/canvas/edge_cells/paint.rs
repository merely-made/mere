// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Shared link stroke presentation. These commands use the exact points the
//! ordinary picker consumes; emphasis changes paint only.

use std::collections::HashSet;

use euclid::default::Point2D;
use kernel::graph::NodeKey;
use paint_list_api::{
    ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData, StrokeCap,
    StrokeItem, StrokeJoin,
};
use scenograph::link::LinkStratum;

use crate::canvas::LinkAppearance;

use super::LinkAppearanceSegment;

const SELECTED_HALO_COLOR: ColorF = ColorF::new(0.91, 0.59, 0.16, 0.48);
const SELECTED_HALO_WIDTH: f32 = 5.0;

pub(crate) fn link_overlay_from_segments(
    segments: Vec<LinkAppearanceSegment>,
    selected_links: &HashSet<scenograph::link::LinkKey>,
    node_visible: impl Fn(NodeKey) -> bool,
    tone: impl Fn(NodeKey, NodeKey) -> f32,
) -> Vec<PaintCmd> {
    let visible = |segment: &&LinkAppearanceSegment| {
        node_visible(segment.appearance.display_from) && node_visible(segment.appearance.display_to)
    };
    let mut commands = Vec::new();

    // Emit every selection halo before any link glyph. A later alias halo must
    // never paint over an earlier alias's direction or opposition marks.
    for segment in segments.iter().filter(visible) {
        if selected_links.contains(&segment.appearance.key)
            && let Some(halo) = stroke(&segment.points, SELECTED_HALO_COLOR, SELECTED_HALO_WIDTH)
        {
            commands.push(halo);
        }
    }

    for segment in segments.iter().filter(visible) {
        let selected = selected_links.contains(&segment.appearance.key);
        let share = if selected {
            // Selection raises the link into the foreground, where its source
            // stratum and directional marks should remain readable.
            1.0
        } else {
            tone(
                segment.appearance.display_from,
                segment.appearance.display_to,
            )
        };
        commands.extend(link_strokes(segment, share));
    }
    commands
}

/// Paint a link centerline and its source-derived marks along the same path.
fn link_strokes(segment: &LinkAppearanceSegment, share: f32) -> Vec<PaintCmd> {
    let mut color = stratum_color(&segment.appearance);
    color.a *= share;
    if share < 1.0 {
        return stroke(&segment.points, color, 1.0).into_iter().collect();
    }

    let width = 1.35 + segment.appearance.strength.max(0.0).ln_1p().min(3.0) * 0.55;
    let mut strokes = stroke(&segment.points, color, width)
        .into_iter()
        .collect::<Vec<_>>();
    let marks = segment.appearance.marks;
    if marks.forward {
        let fraction = if segment.is_self_link() { 0.67 } else { 0.72 };
        strokes.extend(arrowhead_at_fraction(
            &segment.points,
            fraction,
            false,
            color,
        ));
    }
    if marks.reverse {
        let fraction = if segment.is_self_link() { 0.33 } else { 0.28 };
        strokes.extend(arrowhead_at_fraction(
            &segment.points,
            fraction,
            true,
            color,
        ));
    }
    if marks.opposed
        && let Some((midpoint, ux, uy)) = point_tangent_at_fraction(&segment.points, 0.5)
    {
        let normal = Point2D::new(-uy, ux);
        let tick = [
            Point2D::new(midpoint.x - normal.x * 3.0, midpoint.y - normal.y * 3.0),
            Point2D::new(midpoint.x + normal.x * 3.0, midpoint.y + normal.y * 3.0),
        ];
        strokes.extend(stroke(&tick, color, 2.0));
    }
    strokes
}

fn stratum_color(appearance: &LinkAppearance) -> ColorF {
    match appearance.key.stratum {
        LinkStratum::Surface => ColorF::new(0.93, 0.70, 0.34, 0.76),
        LinkStratum::Resource => ColorF::new(0.55, 0.72, 0.92, 0.82),
    }
}

fn arrowhead_at_fraction(
    points: &[Point2D<f32>],
    fraction: f32,
    reverse: bool,
    color: ColorF,
) -> Vec<PaintCmd> {
    let Some((tip, mut ux, mut uy)) = point_tangent_at_fraction(points, fraction) else {
        return Vec::new();
    };
    if reverse {
        ux = -ux;
        uy = -uy;
    }
    let back = Point2D::new(tip.x - ux * 7.0, tip.y - uy * 7.0);
    let side = Point2D::new(-uy * 3.5, ux * 3.5);
    let left = [tip, Point2D::new(back.x - side.x, back.y - side.y)];
    let right = [tip, Point2D::new(back.x + side.x, back.y + side.y)];
    let mut strokes = Vec::with_capacity(2);
    strokes.extend(stroke(&left, color, 1.8));
    strokes.extend(stroke(&right, color, 1.8));
    strokes
}

/// Return the point and forward tangent at an arc-length fraction of a path.
/// Sampling density therefore does not move the semantic marks.
fn point_tangent_at_fraction(
    points: &[Point2D<f32>],
    fraction: f32,
) -> Option<(Point2D<f32>, f32, f32)> {
    if points.len() < 2 || !fraction.is_finite() {
        return None;
    }
    let total = points
        .windows(2)
        .map(|pair| (pair[1] - pair[0]).length())
        .sum::<f32>();
    if !total.is_finite() || total <= f32::EPSILON {
        return None;
    }

    let target = total * fraction.clamp(0.0, 1.0);
    let mut traversed = 0.0;
    for (index, pair) in points.windows(2).enumerate() {
        let length = (pair[1] - pair[0]).length();
        if !length.is_finite() || length <= f32::EPSILON {
            continue;
        }
        let from = pair[0];
        let to = pair[1];
        if target <= traversed + length || index + 2 == points.len() {
            let t = ((target - traversed) / length).clamp(0.0, 1.0);
            let delta = to - from;
            let point = from + delta * t;
            return Some((point, delta.x / length, delta.y / length));
        }
        traversed += length;
    }
    None
}

/// Build a single joined stroke for a whole polyline, keeping command count
/// independent of the number of sampled path segments.
fn stroke(points: &[Point2D<f32>], color: ColorF, width: f32) -> Option<PaintCmd> {
    if points.len() < 2
        || points
            .iter()
            .any(|point| !point.x.is_finite() || !point.y.is_finite())
    {
        return None;
    }
    let first = points.first()?;
    let mut min_x = first.x;
    let mut max_x = first.x;
    let mut min_y = first.y;
    let mut max_y = first.y;
    let mut path = Vec::with_capacity(points.len());
    for (index, point) in points.iter().enumerate() {
        min_x = min_x.min(point.x);
        max_x = max_x.max(point.x);
        min_y = min_y.min(point.y);
        max_y = max_y.max(point.y);
        let point = LayoutPoint::new(point.x, point.y);
        path.push(if index == 0 {
            PathCommand::MoveTo(point)
        } else {
            PathCommand::LineTo(point)
        });
    }
    Some(PaintCmd::DrawStroke(StrokeItem {
        placement: CommonPlacement::new(LayoutRect::new(
            LayoutPoint::new(min_x, min_y),
            LayoutPoint::new(max_x, max_y),
        )),
        path: PathData { commands: path },
        color,
        width,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        dash: None,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use paint_list_api::{PaintCmd, PathCommand};
    use scenograph::link::{LinkKey, LinkMarks};

    fn segment(
        stratum: LinkStratum,
        key_pair: (&str, &str),
        display_pair: (usize, usize),
        marks: LinkMarks,
        points: &[Point2D<f32>],
    ) -> LinkAppearanceSegment {
        LinkAppearanceSegment {
            appearance: LinkAppearance {
                key: LinkKey::new(stratum, key_pair.0, key_pair.1),
                display_from: NodeKey::new(display_pair.0),
                display_to: NodeKey::new(display_pair.1),
                marks,
                strength: 2.0,
            },
            from: points[0],
            to: *points.last().unwrap(),
            points: points.to_vec(),
            responsive: false,
        }
    }

    fn draw_stroke(command: &PaintCmd) -> &StrokeItem {
        let PaintCmd::DrawStroke(stroke) = command else {
            panic!("expected DrawStroke")
        };
        stroke
    }

    fn move_point(stroke: &StrokeItem) -> Point2D<f32> {
        let PathCommand::MoveTo(point) = stroke.path.commands[0] else {
            panic!("expected MoveTo")
        };
        Point2D::new(point.x, point.y)
    }

    fn line_point(stroke: &StrokeItem) -> Point2D<f32> {
        let PathCommand::LineTo(point) = stroke.path.commands[1] else {
            panic!("expected LineTo")
        };
        Point2D::new(point.x, point.y)
    }

    #[test]
    fn selected_alias_bundle_paints_all_halos_before_stratified_marks() {
        let points = [
            Point2D::new(0.0, 0.0),
            Point2D::new(25.0, 8.0),
            Point2D::new(50.0, 0.0),
            Point2D::new(75.0, -8.0),
            Point2D::new(100.0, 0.0),
        ];
        let resource = LinkKey::new(LinkStratum::Resource, "urn:r:a", "urn:r:b");
        let segments = vec![
            segment(
                LinkStratum::Resource,
                ("urn:r:a", "urn:r:b"),
                (0, 2),
                LinkMarks {
                    forward: true,
                    reverse: true,
                    opposed: true,
                },
                &points,
            ),
            // Same canonical resource key, distinct displayed alias geometry.
            segment(
                LinkStratum::Resource,
                ("urn:r:a", "urn:r:b"),
                (1, 3),
                LinkMarks {
                    reverse: true,
                    ..LinkMarks::default()
                },
                &points,
            ),
            segment(
                LinkStratum::Surface,
                ("urn:s:a", "urn:s:b"),
                (0, 2),
                LinkMarks {
                    forward: true,
                    ..LinkMarks::default()
                },
                &points,
            ),
        ];
        let commands = link_overlay_from_segments(
            segments.clone(),
            &HashSet::from([resource]),
            |_| true,
            |_, _| 0.25,
        );

        // Two halos, then the first resource core plus 5 marker strokes, the
        // second alias core plus 2 reverse-arrow strokes, then quiet surface.
        assert_eq!(commands.len(), 12);
        let halos = &commands[..2];
        assert!(halos.iter().all(|command| {
            let stroke = draw_stroke(command);
            stroke.width == SELECTED_HALO_WIDTH
                && stroke.color == SELECTED_HALO_COLOR
                && stroke.path.commands.len() == points.len()
        }));

        let first_core = draw_stroke(&commands[2]);
        assert_eq!(first_core.color, stratum_color(&segments[0].appearance));
        assert_eq!(first_core.path.commands.len(), points.len());
        assert_eq!(first_core.join, StrokeJoin::Round);
        assert_eq!(first_core.cap, StrokeCap::Round);
        // Forward, reverse, and opposition marks remain on top of the halo.
        assert_eq!(draw_stroke(&commands[3]).color, first_core.color);
        assert_eq!(draw_stroke(&commands[4]).color, first_core.color);
        assert_eq!(draw_stroke(&commands[5]).color, first_core.color);
        assert_eq!(draw_stroke(&commands[6]).color, first_core.color);
        assert_eq!(draw_stroke(&commands[7]).color, first_core.color);
        let second_core = draw_stroke(&commands[8]);
        assert_eq!(second_core.color, stratum_color(&segments[1].appearance));
        assert_eq!(second_core.path.commands.len(), points.len());
        assert_eq!(draw_stroke(&commands[9]).color, second_core.color);
        assert_eq!(draw_stroke(&commands[10]).color, second_core.color);
        let quiet_surface = draw_stroke(&commands[11]);
        assert_eq!(quiet_surface.color.a, 0.76 * 0.25);
        assert_eq!(quiet_surface.path.commands.len(), points.len());
    }

    #[test]
    fn directional_and_opposition_marks_use_arc_length_not_sample_index() {
        // Deliberately uneven sampling along a straight line.
        let uneven = [
            Point2D::new(0.0, 0.0),
            Point2D::new(5.0, 0.0),
            Point2D::new(10.0, 0.0),
            Point2D::new(90.0, 0.0),
            Point2D::new(100.0, 0.0),
        ];
        let forward = arrowhead_at_fraction(&uneven, 0.72, false, ColorF::new(1.0, 0.0, 0.0, 1.0));
        let reverse = arrowhead_at_fraction(&uneven, 0.28, true, ColorF::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(forward.len(), 2);
        assert_eq!(reverse.len(), 2);
        let forward_tip = move_point(draw_stroke(&forward[0]));
        let reverse_tip = move_point(draw_stroke(&reverse[0]));
        assert!((forward_tip.x - 72.0).abs() < 0.001);
        assert!((reverse_tip.x - 28.0).abs() < 0.001);
        assert!(line_point(draw_stroke(&forward[0])).x < forward_tip.x);
        assert!(line_point(draw_stroke(&reverse[0])).x > reverse_tip.x);

        let Some((midpoint, _, _)) = point_tangent_at_fraction(&uneven, 0.5) else {
            panic!("midpoint should be on the path")
        };
        assert!((midpoint.x - 50.0).abs() < 0.001);

        let opposed = segment(
            LinkStratum::Surface,
            ("urn:a", "urn:b"),
            (0, 1),
            LinkMarks {
                opposed: true,
                ..LinkMarks::default()
            },
            &uneven,
        );
        let opposed_strokes = link_strokes(&opposed, 1.0);
        assert_eq!(opposed_strokes.len(), 2);
        let tick_from = move_point(draw_stroke(&opposed_strokes[1]));
        let tick_to = line_point(draw_stroke(&opposed_strokes[1]));
        assert!(((tick_from.x + tick_to.x) * 0.5 - 50.0).abs() < 0.001);
    }

    #[test]
    fn selected_quiet_link_is_raised_without_changing_its_geometry() {
        let points = [
            Point2D::new(0.0, 0.0),
            Point2D::new(10.0, 2.0),
            Point2D::new(20.0, 0.0),
        ];
        let selected = segment(
            LinkStratum::Surface,
            ("urn:a", "urn:b"),
            (0, 1),
            LinkMarks {
                forward: true,
                ..LinkMarks::default()
            },
            &points,
        );
        let commands = link_overlay_from_segments(
            vec![selected.clone()],
            &HashSet::from([selected.appearance.key.clone()]),
            |_| true,
            |_, _| 0.1,
        );
        assert_eq!(commands.len(), 4); // halo, core, two arrow branches
        assert_eq!(draw_stroke(&commands[1]).color.a, 0.76);
        assert_eq!(draw_stroke(&commands[1]).path.commands.len(), points.len());
        assert_eq!(selected.points, points.to_vec());

        let expected_key = selected.appearance.key.clone();
        let hit = super::super::link_hit_test_segments(
            vec![selected.clone()],
            Point2D::new(10.0, 2.0),
            0.1,
            |_| true,
            |_| true,
        )
        .expect("same displayed path remains pickable");
        assert_eq!(hit.key, expected_key);
        assert_eq!(selected.points, points.to_vec());
    }
}
