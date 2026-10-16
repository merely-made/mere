// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Partition: shares of a measured whole, subdividing the viewport.
//!
//! Top-level shares are the groups (an item's categorical axis), ungrouped
//! items, and the remainder of the whole the items do not account for. Tiles
//! lays them out as a squarified treemap and each group's members again
//! inside it; Rings gives each share an angle, groups on the inner ring and
//! their members outside them. An item's weight is its measured size; one
//! without a usable weight counts as one.

use std::f32::consts::{FRAC_PI_2, TAU};

use sceno::{Footprint, Partition, PartitionForm, Rect, ScoreItem, Size2, Vec2};

use super::{Shape, ShapedRegion, categorical_axis};
use crate::math::sin_cos;

/// The label on the region for what the items leave of the whole.
pub(crate) const REMAINDER_LABEL: &str = "remainder";

enum Share {
    Group { name: String, members: Vec<usize> },
    Single(usize),
    Remainder,
}

fn size_of(item: &ScoreItem) -> f32 {
    item.weight
        .filter(|w| w.is_finite() && *w > 0.0)
        .unwrap_or(1.0)
}

/// The top-level shares in first-appearance order, each with its size.
fn shares(config: &Partition, items: &[&ScoreItem]) -> Vec<(Share, f32)> {
    let mut shares: Vec<(Share, f32)> = Vec::new();
    for (index, item) in items.iter().enumerate() {
        let size = size_of(item);
        match categorical_axis(item) {
            Some(name) => {
                let found = shares
                    .iter_mut()
                    .find(|(share, _)| matches!(share, Share::Group { name: n, .. } if n == name));
                match found {
                    Some((Share::Group { members, .. }, total)) => {
                        members.push(index);
                        *total += size;
                    },
                    _ => shares.push((
                        Share::Group {
                            name: name.to_owned(),
                            members: vec![index],
                        },
                        size,
                    )),
                }
            },
            None => shares.push((Share::Single(index), size)),
        }
    }
    let total: f32 = shares.iter().map(|(_, size)| size).sum();
    if let Some(whole) = config.whole.filter(|w| w.is_finite() && *w > total) {
        shares.push((Share::Remainder, whole - total));
    }
    shares
}

pub(super) fn shape(config: &Partition, items: &[&ScoreItem]) -> Shape {
    let mut shape = Shape::unplaced(items.len());
    let shares = shares(config, items);
    match config.form {
        PartitionForm::Tiles => tiles(config, items, &shares, &mut shape),
        PartitionForm::Rings => rings(config, items, &shares, &mut shape),
    }
    shape
}

fn tiles(config: &Partition, items: &[&ScoreItem], shares: &[(Share, f32)], shape: &mut Shape) {
    let sizes: Vec<f32> = shares.iter().map(|(_, size)| *size).collect();
    let rects = squarify(&sizes, config.bounds);
    for ((share, _), rect) in shares.iter().zip(rects) {
        match share {
            Share::Single(index) => place_tile(shape, *index, inset(rect, config.gap / 2.0)),
            Share::Remainder => shape.regions.push(ShapedRegion {
                contour: rect_points(rect),
                label: Some(REMAINDER_LABEL.into()),
                members: Vec::new(),
            }),
            Share::Group { name, members } => {
                // A band along the top leaves room for the group's label.
                let inner = inset(rect, config.gap);
                let inner = Rect::new(
                    Vec2::new(inner.origin.x, inner.origin.y + config.gap),
                    Size2::new(inner.size.w, (inner.size.h - config.gap).max(0.0)),
                );
                let sizes: Vec<f32> = members.iter().map(|i| size_of(items[*i])).collect();
                for (index, tile) in members.iter().zip(squarify(&sizes, inner)) {
                    place_tile(shape, *index, inset(tile, config.gap / 4.0));
                }
                shape.regions.push(ShapedRegion {
                    contour: rect_points(rect),
                    label: Some(name.clone()),
                    members: members.clone(),
                });
            },
        }
    }
}

fn place_tile(shape: &mut Shape, index: usize, tile: Rect) {
    shape.positions[index] = Vec2::new(
        tile.origin.x + tile.size.w / 2.0,
        tile.origin.y + tile.size.h / 2.0,
    );
    shape.footprints[index] = Some(Footprint::Rect { size: tile.size });
}

fn rings(config: &Partition, items: &[&ScoreItem], shares: &[(Share, f32)], shape: &mut Shape) {
    let b = config.bounds;
    let center = Vec2::new(b.origin.x + b.size.w / 2.0, b.origin.y + b.size.h / 2.0);
    let outer = b.size.w.min(b.size.h) / 2.0;
    let (hub, middle) = (outer * 0.22, outer * 0.6);
    let half_gap = config.gap / 2.0;
    let total: f32 = shares
        .iter()
        .map(|(_, size)| size)
        .sum::<f32>()
        .max(f32::EPSILON);
    let mut angle = -FRAC_PI_2;
    for (share, size) in shares {
        let span = TAU * size / total;
        match share {
            Share::Single(index) => {
                place_sector(
                    shape,
                    *index,
                    center,
                    (hub + half_gap, outer),
                    (angle, span),
                );
            },
            Share::Remainder => shape.regions.push(ShapedRegion {
                contour: sector(center, (hub, outer), (angle, span)),
                label: Some(REMAINDER_LABEL.into()),
                members: Vec::new(),
            }),
            Share::Group { name, members } => {
                shape.regions.push(ShapedRegion {
                    contour: sector(center, (hub, middle), (angle, span)),
                    label: Some(name.clone()),
                    members: members.clone(),
                });
                let mut at = angle;
                for index in members {
                    let part = span * size_of(items[*index]) / size;
                    place_sector(
                        shape,
                        *index,
                        center,
                        (middle + half_gap, outer),
                        (at, part),
                    );
                    at += part;
                }
            },
        }
        angle += span;
    }
}

/// Place an item at its sector's middle, its footprint the sector itself.
fn place_sector(
    shape: &mut Shape,
    index: usize,
    center: Vec2,
    radii: (f32, f32),
    (start, span): (f32, f32),
) {
    let (sin, cos) = sin_cos(start + span / 2.0);
    let r = (radii.0 + radii.1) / 2.0;
    let at = Vec2::new(center.x + cos * r, center.y + sin * r);
    let points = sector(center, radii, (start, span))
        .into_iter()
        .map(|p| Vec2::new(p.x - at.x, p.y - at.y))
        .collect();
    shape.positions[index] = at;
    shape.footprints[index] = Some(Footprint::Polygon { points });
}

/// An annular sector as a polygon: the outer arc, then the inner arc back.
fn sector(center: Vec2, (inner, outer): (f32, f32), (start, span): (f32, f32)) -> Vec<Vec2> {
    let steps = ((span / (TAU / 96.0)).ceil() as usize).max(2);
    let arc = |r: f32, k: usize| {
        let (sin, cos) = sin_cos(start + span * k as f32 / steps as f32);
        Vec2::new(center.x + cos * r, center.y + sin * r)
    };
    let mut points: Vec<Vec2> = (0..=steps).map(|k| arc(outer, k)).collect();
    points.extend((0..=steps).rev().map(|k| arc(inner, k)));
    points
}

fn inset(rect: Rect, by: f32) -> Rect {
    let by = by.min(rect.size.w / 2.0).min(rect.size.h / 2.0).max(0.0);
    Rect::new(
        Vec2::new(rect.origin.x + by, rect.origin.y + by),
        Size2::new(rect.size.w - 2.0 * by, rect.size.h - 2.0 * by),
    )
}

fn rect_points(rect: Rect) -> Vec<Vec2> {
    let (x, y, w, h) = (rect.origin.x, rect.origin.y, rect.size.w, rect.size.h);
    vec![
        Vec2::new(x, y),
        Vec2::new(x + w, y),
        Vec2::new(x + w, y + h),
        Vec2::new(x, y + h),
    ]
}

/// Squarified treemap (Bruls, Huizing and van Wijk): one rectangle per size,
/// returned in the input's order, areas in proportion, aspect ratios kept near
/// one by laying the largest first in rows along the shorter side.
pub(crate) fn squarify(sizes: &[f32], rect: Rect) -> Vec<Rect> {
    let mut out = vec![Rect::new(rect.origin, Size2::new(0.0, 0.0)); sizes.len()];
    let total: f32 = sizes.iter().sum();
    let area = rect.size.w * rect.size.h;
    // Written so a NaN total or area also lands here.
    let positive = |value: f32| value > 0.0;
    if sizes.is_empty() || !positive(total) || !positive(area) {
        return out;
    }
    let mut order: Vec<usize> = (0..sizes.len()).collect();
    order.sort_by(|a, b| sizes[*b].total_cmp(&sizes[*a]).then(a.cmp(b)));
    let areas: Vec<f32> = sizes.iter().map(|s| s / total * area).collect();
    let mut free = rect;
    let mut row: Vec<usize> = Vec::new();
    for index in order {
        let side = free.size.w.min(free.size.h);
        let mut wider = row.clone();
        wider.push(index);
        if row.is_empty() || worst(&wider, &areas, side) <= worst(&row, &areas, side) {
            row = wider;
        } else {
            free = lay_row(&row, &areas, free, &mut out);
            row = vec![index];
        }
    }
    lay_row(&row, &areas, free, &mut out);
    out
}

fn worst(row: &[usize], areas: &[f32], side: f32) -> f32 {
    let sum: f32 = row.iter().map(|i| areas[*i]).sum();
    let (s2, sum2) = (side * side, sum * sum);
    row.iter()
        .map(|i| (s2 * areas[*i] / sum2).max(sum2 / (s2 * areas[*i])))
        .fold(0.0, f32::max)
}

/// Lay `row` along the shorter side of `free`, returning what is left.
fn lay_row(row: &[usize], areas: &[f32], free: Rect, out: &mut [Rect]) -> Rect {
    let sum: f32 = row.iter().map(|i| areas[*i]).sum();
    let (x, y, w, h) = (free.origin.x, free.origin.y, free.size.w, free.size.h);
    if w >= h {
        let thick = if h > 0.0 { sum / h } else { 0.0 };
        let mut at = y;
        for i in row {
            let len = if thick > 0.0 { areas[*i] / thick } else { 0.0 };
            out[*i] = Rect::new(Vec2::new(x, at), Size2::new(thick, len));
            at += len;
        }
        Rect::new(Vec2::new(x + thick, y), Size2::new((w - thick).max(0.0), h))
    } else {
        let thick = if w > 0.0 { sum / w } else { 0.0 };
        let mut at = x;
        for i in row {
            let len = if thick > 0.0 { areas[*i] / thick } else { 0.0 };
            out[*i] = Rect::new(Vec2::new(at, y), Size2::new(len, thick));
            at += len;
        }
        Rect::new(Vec2::new(x, y + thick), Size2::new(w, (h - thick).max(0.0)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::families::tests::{card, items};
    use sceno::AxisValue;

    fn sized(id: u32, group: Option<&str>, weight: f32) -> ScoreItem {
        ScoreItem {
            axis: group.map(|g| AxisValue::Categorical(g.into())),
            weight: Some(weight),
            ..card(id)
        }
    }

    fn area(r: &Rect) -> f32 {
        r.size.w * r.size.h
    }

    #[test]
    fn squarify_keeps_proportions_inside_the_rect() {
        let rect = Rect::new(Vec2::new(0.0, 0.0), Size2::new(600.0, 400.0));
        let sizes = [6.0, 6.0, 4.0, 3.0, 2.0, 2.0, 1.0];
        let rects = squarify(&sizes, rect);
        let total: f32 = sizes.iter().sum();
        for (size, r) in sizes.iter().zip(&rects) {
            assert!((area(r) - size / total * 240_000.0).abs() < 1.0);
            assert!(r.origin.x >= -1e-3 && r.origin.y >= -1e-3);
            assert!(r.origin.x + r.size.w <= 600.0 + 1e-2);
            assert!(r.origin.y + r.size.h <= 400.0 + 1e-2);
        }
        let worst = rects
            .iter()
            .map(|r| r.size.w.max(r.size.h) / r.size.w.min(r.size.h))
            .fold(0.0, f32::max);
        assert!(worst < 3.0, "squarified, worst aspect {worst}");
    }

    #[test]
    fn tiles_nest_groups_and_hatch_the_remainder() {
        let owned = [
            sized(1, Some("djinn"), 120.0),
            sized(2, Some("djinn"), 96.0),
            sized(3, Some("distillery"), 2100.0),
            sized(4, None, 400.0),
        ];
        let config = Partition {
            whole: Some(4000.0),
            ..Partition::default()
        };
        let shape = shape(&config, &items(&owned));
        let labels: Vec<_> = shape
            .regions
            .iter()
            .map(|r| r.label.clone().unwrap())
            .collect();
        assert_eq!(labels, ["djinn", "distillery", REMAINDER_LABEL]);
        assert_eq!(shape.regions[0].members, vec![0, 1]);
        let footprint_area = |i: usize| match &shape.footprints[i] {
            Some(Footprint::Rect { size }) => size.w * size.h,
            other => panic!("tile footprint, got {other:?}"),
        };
        assert!(
            footprint_area(2) > footprint_area(0) * 10.0,
            "area follows share"
        );
        let b = config.bounds;
        for p in &shape.positions {
            assert!(p.x > b.origin.x && p.x < b.origin.x + b.size.w);
            assert!(p.y > b.origin.y && p.y < b.origin.y + b.size.h);
        }
    }

    #[test]
    fn no_whole_means_no_remainder_and_missing_weights_count_as_one() {
        let owned = [card(1), card(2)];
        let shape = shape(&Partition::default(), &items(&owned));
        assert!(shape.regions.is_empty());
        assert!(shape.footprints.iter().all(Option::is_some));
    }

    #[test]
    fn rings_give_angles_by_share() {
        let owned = [
            sized(1, Some("a"), 1.0),
            sized(2, Some("a"), 1.0),
            sized(3, None, 2.0),
        ];
        let config = Partition {
            form: PartitionForm::Rings,
            whole: Some(8.0),
            ..Partition::default()
        };
        let shape = shape(&config, &items(&owned));
        assert_eq!(shape.regions.len(), 2, "group a and the remainder");
        assert!(matches!(
            shape.footprints[0],
            Some(Footprint::Polygon { .. })
        ));
        let c = Vec2::new(0.0, 0.0);
        let radius = |p: Vec2| ((p.x - c.x).powi(2) + (p.y - c.y).powi(2)).sqrt();
        assert!(radius(shape.positions[0]) > radius(shape.regions[0].contour[0]) * 0.5);
        assert_ne!(shape.positions[0], shape.positions[1]);
    }
}
