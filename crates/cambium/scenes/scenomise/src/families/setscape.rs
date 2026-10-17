// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Sets: fixed set regions as the backdrop; position means membership.
//!
//! Each set is a circle laid out from the count alone, so the regions never
//! follow the items. An item goes inside every circle it belongs to and
//! outside every other; its tier picks the core (forward) or the rim
//! (background). Placement is rejection sampling from the item's stable hash,
//! relaxing tier, then exclusion, when a region is too small to hit.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use sceno::{SET_CORE_WEIGHT, SET_SEPARATOR, ScoreItem, Setscape, Vec2};

use super::{Shape, ShapedRegion, categorical_axis, stable_hash};
use crate::math::sin_cos;

/// A core's radius as a share of its set's.
pub(crate) const CORE_SHARE: f32 = 0.62;
const TRIES: usize = 400;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Circle {
    pub center: Vec2,
    pub radius: f32,
}

impl Circle {
    fn holds(&self, p: Vec2, radius: f32) -> bool {
        let (dx, dy) = (p.x - self.center.x, p.y - self.center.y);
        dx * dx + dy * dy < radius * radius
    }
}

/// The circles for `count` sets inside `config.bounds`: up to four overlap
/// pairwise about the centre, as the specimen draws them; more ring it.
pub(crate) fn circles(config: &Setscape) -> Vec<Circle> {
    let b = config.bounds;
    let center = Vec2::new(b.origin.x + b.size.w / 2.0, b.origin.y + b.size.h / 2.0);
    let half = b.size.w.min(b.size.h) / 2.0;
    let n = config.sets.len();
    let polar = |distance: f32, angle: f32| {
        let (sin, cos) = sin_cos(angle);
        Vec2::new(
            center.x + cos * distance * half,
            center.y + sin * distance * half,
        )
    };
    let (points, radius): (Vec<Vec2>, f32) = match n {
        0 => (Vec::new(), 0.0),
        1 => (vec![center], 0.85),
        2 => (vec![polar(0.4, PI), polar(0.4, 0.0)], 0.7),
        3 => (
            (0..3)
                .map(|k| polar(0.38, -FRAC_PI_2 + TAU * k as f32 / 3.0))
                .collect(),
            0.6,
        ),
        4 => (
            [(-1.0, -1.0), (1.0, -1.0), (-1.0, 1.0), (1.0, 1.0)]
                .iter()
                .map(|(x, y)| Vec2::new(center.x + x * 0.33 * half, center.y + y * 0.33 * half))
                .collect(),
            0.6,
        ),
        _ => (
            (0..n)
                .map(|k| polar(0.55, -FRAC_PI_2 + TAU * k as f32 / n as f32))
                .collect(),
            (0.55 * crate::math::sin(PI / n as f32) * 1.4).min(0.45),
        ),
    };
    points
        .into_iter()
        .map(|center| Circle {
            center,
            radius: radius * half,
        })
        .collect()
}

/// The sets an item's axis names, as indices into `config.sets`.
fn memberships(config: &Setscape, item: &ScoreItem) -> Vec<usize> {
    let Some(tags) = categorical_axis(item) else {
        return Vec::new();
    };
    let mut found: Vec<usize> = tags
        .split(SET_SEPARATOR)
        .filter_map(|tag| config.sets.iter().position(|set| set == tag.trim()))
        .collect();
    found.sort_unstable();
    found.dedup();
    found
}

fn in_core(item: &ScoreItem) -> bool {
    item.weight.is_some_and(|w| w >= SET_CORE_WEIGHT)
}

pub(super) fn shape(config: &Setscape, items: &[&ScoreItem]) -> Shape {
    let circles = circles(config);
    let mut shape = Shape::unplaced(items.len());
    let mut members = vec![(Vec::new(), Vec::new()); circles.len()];
    for (index, item) in items.iter().enumerate() {
        let sets = memberships(config, item);
        let core = in_core(item);
        shape.positions[index] = place(config, &circles, &sets, core, stable_hash(item));
        for set in sets {
            members[set].0.push(index);
            if core {
                members[set].1.push(index);
            }
        }
    }
    for ((name, circle), (all, core)) in config.sets.iter().zip(&circles).zip(members) {
        shape.regions.push(ShapedRegion {
            contour: ring_points(*circle, circle.radius),
            label: Some(name.clone()),
            members: all,
        });
        shape.regions.push(ShapedRegion {
            contour: ring_points(*circle, circle.radius * CORE_SHARE),
            label: Some(format!("{name} · core")),
            members: core,
        });
    }
    shape
}

fn place(config: &Setscape, circles: &[Circle], sets: &[usize], core: bool, seed: u64) -> Vec2 {
    let b = config.bounds;
    let mut stream = seed;
    let mut sample = |within: (Vec2, f32, f32)| {
        let (origin, w, h) = within;
        Vec2::new(
            origin.x + unit(&mut stream) * w,
            origin.y + unit(&mut stream) * h,
        )
    };
    if sets.is_empty() {
        // Outside every set, somewhere in the bounds.
        for _ in 0..TRIES {
            let p = sample((b.origin, b.size.w, b.size.h));
            if circles.iter().all(|c| !c.holds(p, c.radius)) {
                return p;
            }
        }
        return b.origin;
    }
    let first = circles[sets[0]];
    let square = (
        Vec2::new(first.center.x - first.radius, first.center.y - first.radius),
        first.radius * 2.0,
        first.radius * 2.0,
    );
    let fits = |p: Vec2, tiered: bool, exclusive: bool| {
        let inside = sets
            .iter()
            .all(|s| circles[*s].holds(p, circles[*s].radius));
        let outside = !exclusive
            || circles
                .iter()
                .enumerate()
                .all(|(k, c)| sets.contains(&k) || !c.holds(p, c.radius));
        let in_cores = sets
            .iter()
            .all(|s| circles[*s].holds(p, circles[*s].radius * CORE_SHARE));
        let tier = !tiered || in_cores == core;
        inside && outside && tier
    };
    for (tiered, exclusive) in [(true, true), (false, true), (false, false)] {
        for _ in 0..TRIES {
            let p = sample(square);
            if fits(p, tiered, exclusive) {
                return p;
            }
        }
    }
    first.center
}

/// SplitMix64's next value as a float in [0, 1).
fn unit(state: &mut u64) -> f32 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    (z >> 40) as f32 / (1u64 << 24) as f32
}

fn ring_points(circle: Circle, radius: f32) -> Vec<Vec2> {
    (0..64)
        .map(|k| {
            let (sin, cos) = sin_cos(TAU * k as f32 / 64.0);
            Vec2::new(
                circle.center.x + cos * radius,
                circle.center.y + sin * radius,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::families::tests::{card, items};
    use sceno::AxisValue;

    fn member(id: u32, sets: &str, weight: f32) -> ScoreItem {
        ScoreItem {
            axis: Some(AxisValue::Categorical(sets.into())),
            weight: Some(weight),
            ..card(id)
        }
    }

    fn four() -> Setscape {
        Setscape {
            sets: ["song", "practice", "publishing", "neighbourhood"]
                .map(String::from)
                .to_vec(),
            ..Setscape::default()
        }
    }

    #[test]
    fn position_means_membership() {
        let config = four();
        let circles = circles(&config);
        let owned = [
            member(1, "song", 0.0),
            member(2, "song|practice", 1.0),
            member(3, "song|practice|publishing|neighbourhood", 0.0),
            member(4, "", 0.0),
            member(5, "unknown|publishing", 1.0),
        ];
        let shape = shape(&config, &items(&owned));
        let inside = |p: Vec2| -> Vec<usize> {
            (0..4)
                .filter(|k| circles[*k].holds(p, circles[*k].radius))
                .collect()
        };
        assert_eq!(inside(shape.positions[0]), vec![0]);
        assert_eq!(inside(shape.positions[1]), vec![0, 1]);
        assert_eq!(inside(shape.positions[2]), vec![0, 1, 2, 3]);
        assert!(
            inside(shape.positions[3]).is_empty(),
            "no set, outside them all"
        );
        assert_eq!(inside(shape.positions[4]), vec![2]);
        let core = |p: Vec2, k: usize| circles[k].holds(p, circles[k].radius * CORE_SHARE);
        assert!(!core(shape.positions[0], 0), "background sits in the rim");
        assert!(core(shape.positions[4], 2), "forward sits in the core");
    }

    #[test]
    fn regions_are_fixed_by_the_sets_not_the_items() {
        let config = four();
        let a = shape(&config, &items(&[member(1, "song", 1.0)]));
        let b = shape(&config, &items(&[member(9, "practice", 0.0), card(3)]));
        let contours = |s: &Shape| {
            s.regions
                .iter()
                .map(|r| r.contour.clone())
                .collect::<Vec<_>>()
        };
        assert_eq!(contours(&a), contours(&b));
        assert_eq!(a.regions.len(), 8, "a set and its core each");
        assert_eq!(a.regions[0].members, vec![0]);
        assert_eq!(a.regions[1].members, vec![0]);
    }

    #[test]
    fn placement_is_stable_and_more_than_four_sets_ring() {
        let config = Setscape {
            sets: (0..6).map(|k| format!("s{k}")).collect(),
            ..Setscape::default()
        };
        let owned = [member(1, "s0|s1", 0.0)];
        let first = shape(&config, &items(&owned)).positions[0];
        assert_eq!(first, shape(&config, &items(&owned)).positions[0]);
        assert_eq!(circles(&config).len(), 6);
    }
}
