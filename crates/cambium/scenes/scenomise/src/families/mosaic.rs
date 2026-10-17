// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Mosaic: tiles by weight on the infinite canvas, grouped, never reflowing.
//!
//! The extent grows from the total weight (`unit_area` per unit) at the
//! configured aspect, so the mosaic is as large as what it holds rather than
//! the viewport. Groups, then each group's members, split by weighted
//! balanced bisection in their given order: the cut falls where the running
//! weight comes nearest half, across the longer side. Only weight and order
//! move a tile; selection does not.

use sceno::{Footprint, Mosaic, Rect, ScoreItem, Size2, Vec2};

use super::{Shape, ShapedRegion, categorical_axis};

/// The group of an item with no categorical axis.
pub(crate) const UNGROUPED: &str = "";

fn weight_of(item: &ScoreItem) -> f32 {
    item.weight
        .filter(|w| w.is_finite() && *w > 0.0)
        .unwrap_or(1.0)
}

/// Groups in configured order, then first appearance; members in item order.
fn groups(config: &Mosaic, items: &[&ScoreItem]) -> Vec<(String, Vec<usize>)> {
    let mut groups: Vec<(String, Vec<usize>)> = config
        .group_order
        .iter()
        .map(|name| (name.clone(), Vec::new()))
        .collect();
    for (index, item) in items.iter().enumerate() {
        let name = categorical_axis(item).unwrap_or(UNGROUPED);
        match groups.iter_mut().find(|(group, _)| group == name) {
            Some((_, members)) => members.push(index),
            None => groups.push((name.to_owned(), vec![index])),
        }
    }
    groups.retain(|(_, members)| !members.is_empty());
    groups
}

pub(super) fn shape(config: &Mosaic, items: &[&ScoreItem]) -> Shape {
    let mut shape = Shape::unplaced(items.len());
    let groups = groups(config, items);
    let total: f32 = items.iter().map(|item| weight_of(item)).sum();
    let area = total * config.unit_area.max(f32::EPSILON);
    let aspect = if config.aspect.is_finite() && config.aspect > 0.0 {
        config.aspect
    } else {
        1.6
    };
    let height = (area / aspect).sqrt();
    let width = height * aspect;
    let bounds = Rect::new(
        Vec2::new(
            config.origin.x - width / 2.0,
            config.origin.y - height / 2.0,
        ),
        Size2::new(width, height),
    );
    let group_weights: Vec<f32> = groups
        .iter()
        .map(|(_, members)| members.iter().map(|i| weight_of(items[*i])).sum())
        .collect();
    for ((name, members), rect) in groups
        .iter()
        .zip(bisect(&group_weights, bounds, config.gap))
    {
        let band = config.header.min(rect.size.h / 2.0).max(0.0);
        let inner = Rect::new(
            Vec2::new(rect.origin.x, rect.origin.y + band),
            Size2::new(rect.size.w, rect.size.h - band),
        );
        let weights: Vec<f32> = members.iter().map(|i| weight_of(items[*i])).collect();
        for (index, tile) in members.iter().zip(bisect(&weights, inner, config.gap)) {
            shape.positions[*index] = Vec2::new(
                tile.origin.x + tile.size.w / 2.0,
                tile.origin.y + tile.size.h / 2.0,
            );
            shape.footprints[*index] = Some(Footprint::Rect { size: tile.size });
            shape.regions.push(ShapedRegion {
                contour: corners(tile),
                label: None,
                members: vec![*index],
            });
        }
        shape.regions.push(ShapedRegion {
            contour: corners(rect),
            label: Some(name.clone()),
            members: members.clone(),
        });
    }
    shape
}

/// Weighted balanced bisection: one rectangle per weight, in order.
pub(crate) fn bisect(weights: &[f32], rect: Rect, gap: f32) -> Vec<Rect> {
    let mut out = Vec::with_capacity(weights.len());
    split(weights, rect, gap.max(0.0), &mut out);
    out
}

fn split(weights: &[f32], r: Rect, gap: f32, out: &mut Vec<Rect>) {
    match weights.len() {
        0 => return,
        1 => {
            out.push(r);
            return;
        },
        _ => {},
    }
    let total: f32 = weights.iter().sum();
    let (mut cut, mut before, mut best, mut run) = (1, 0.0, f32::INFINITY, 0.0);
    for (i, w) in weights[..weights.len() - 1].iter().enumerate() {
        run += w;
        let distance = (run - total / 2.0).abs();
        if distance < best {
            (best, cut, before) = (distance, i + 1, run);
        }
    }
    let share = if total > 0.0 { before / total } else { 0.5 };
    let (first, second) = if r.size.w >= r.size.h {
        let w1 = ((r.size.w - gap) * share).max(0.0);
        (
            Rect::new(r.origin, Size2::new(w1, r.size.h)),
            Rect::new(
                Vec2::new(r.origin.x + w1 + gap, r.origin.y),
                Size2::new((r.size.w - w1 - gap).max(0.0), r.size.h),
            ),
        )
    } else {
        let h1 = ((r.size.h - gap) * share).max(0.0);
        (
            Rect::new(r.origin, Size2::new(r.size.w, h1)),
            Rect::new(
                Vec2::new(r.origin.x, r.origin.y + h1 + gap),
                Size2::new(r.size.w, (r.size.h - h1 - gap).max(0.0)),
            ),
        )
    };
    split(&weights[..cut], first, gap, out);
    split(&weights[cut..], second, gap, out);
}

fn corners(r: Rect) -> Vec<Vec2> {
    let (x, y, w, h) = (r.origin.x, r.origin.y, r.size.w, r.size.h);
    vec![
        Vec2::new(x, y),
        Vec2::new(x + w, y),
        Vec2::new(x + w, y + h),
        Vec2::new(x, y + h),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::families::tests::{card, items};
    use sceno::AxisValue;

    fn tile(id: u32, group: &str, weight: f32) -> ScoreItem {
        ScoreItem {
            axis: Some(AxisValue::Categorical(group.into())),
            weight: Some(weight),
            ..card(id)
        }
    }

    fn size(shape: &Shape, i: usize) -> Size2 {
        match &shape.footprints[i] {
            Some(Footprint::Rect { size }) => *size,
            other => panic!("a tile, got {other:?}"),
        }
    }

    #[test]
    fn the_extent_grows_with_weight_not_the_viewport() {
        let small = [tile(1, "a", 1.0), tile(2, "a", 1.0)];
        let large = [tile(1, "a", 4.0), tile(2, "a", 4.0)];
        let config = Mosaic::default();
        let a = shape(&config, &items(&small));
        let b = shape(&config, &items(&large));
        let area = |s: &Shape| (0..2).map(|i| size(s, i).w * size(s, i).h).sum::<f32>();
        assert!(area(&b) > area(&a) * 3.0);
    }

    #[test]
    fn groups_hold_their_members_in_order_and_area_follows_weight() {
        let owned = [
            tile(1, "turnstone", 3.0),
            tile(2, "woodshed", 1.0),
            tile(3, "turnstone", 1.0),
            tile(4, "knot", 2.0),
        ];
        let config = Mosaic {
            group_order: vec!["knot".into()],
            ..Mosaic::default()
        };
        let shape = shape(&config, &items(&owned));
        let groups: Vec<_> = shape
            .regions
            .iter()
            .filter_map(|r| r.label.clone().map(|l| (l, r.members.clone())))
            .collect();
        assert_eq!(
            groups,
            [
                ("knot".to_string(), vec![3]),
                ("turnstone".to_string(), vec![0, 2]),
                ("woodshed".to_string(), vec![1]),
            ]
        );
        let a = |i| size(&shape, i).w * size(&shape, i).h;
        assert!(
            a(0) > a(2) * 2.0,
            "three times the weight, about three times the area"
        );
        assert_eq!(
            shape.regions.iter().filter(|r| r.label.is_none()).count(),
            4,
            "a region per tile"
        );
    }

    #[test]
    fn selection_cannot_move_a_tile() {
        let mut owned = [tile(1, "a", 1.0), tile(2, "a", 2.0)];
        let before = shape(&Mosaic::default(), &items(&owned)).positions;
        owned[0].visible = false;
        owned[1].layer = 3;
        assert_eq!(shape(&Mosaic::default(), &items(&owned)).positions, before);
    }

    #[test]
    fn bisection_tiles_without_overlap() {
        let rect = Rect::new(Vec2::ZERO, Size2::new(400.0, 300.0));
        let rects = bisect(&[5.0, 1.0, 1.0, 3.0, 2.0], rect, 4.0);
        for (i, a) in rects.iter().enumerate() {
            for b in &rects[i + 1..] {
                let apart = a.origin.x + a.size.w <= b.origin.x + 1e-3
                    || b.origin.x + b.size.w <= a.origin.x + 1e-3
                    || a.origin.y + a.size.h <= b.origin.y + 1e-3
                    || b.origin.y + b.size.h <= a.origin.y + 1e-3;
                assert!(apart, "{a:?} overlaps {b:?}");
            }
        }
    }
}
