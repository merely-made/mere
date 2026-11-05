// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Product-free analytic score realization.

use crate::families;
use sceno::{
    Arrangement, Footprint, Geographic, Grid, Hold, HonoredHold, Hulls, InstanceId, Placement,
    ProjectedItem, Rect, Region, Scene, Score, ScoreItem, Spiral, SpiralCurve, Transform2, Vec2,
};

/// The golden-angle spiral's nearest neighbours are roughly 0.9 of the scale
/// constant apart. `sqrt(2) / 0.9` clears equal axis-aligned squares at every
/// angle; a little headroom keeps the receipt legible.
const SPACING_PER_EXTENT: f32 = 1.6;

/// Realize a persisted score into a scene. The solver only sees the score's
/// opaque refs, footprints, placements, and selected representations.
///
/// An [`Arrangement::Custom`] score realizes nothing here; use
/// [`solve_with`] with a planner that can resolve it.
pub fn solve(score: &Score) -> Scene {
    solve_with(score, |_| None)
}

/// [`solve`], with a planner consulted for [`Arrangement::Custom`].
///
/// The planner receives the items in arrangement order and returns one position
/// per item, in that same order. It exists so a registry living above this crate
/// — `scenograph`'s — can supply a solver without this crate learning what a
/// registry is, and without anyone re-implementing scene construction: interning,
/// bounds, the hold report and the Hulls partition all stay here, where the
/// built-in families already exercise them.
///
/// A planner that returns the wrong number of positions is refused rather than
/// indexed into. Placing three items from a five-item plan would be an arbitrary
/// mismatch resolved silently, and a solver that miscounts is a solver whose
/// positions should not be trusted anyway.
pub fn solve_with(
    score: &Score,
    plan_custom: impl FnOnce(&[&ScoreItem]) -> Option<Vec<Vec2>>,
) -> Scene {
    let mut scene = Scene::new();
    scene.generation = score.generation;

    let mut order: Vec<(usize, &ScoreItem)> = score.items.iter().enumerate().collect();
    order.sort_by_key(|(index, item)| (item.ordinal, *index));

    let effective_spacing = match &score.arrangement {
        Arrangement::Spiral(spiral) => spiral.spacing.max(
            order
                .iter()
                .filter_map(|(_, item)| footprint_side(&item.footprint))
                .max_by(f32::total_cmp)
                .unwrap_or(0.0)
                * SPACING_PER_EXTENT,
        ),
        _ => 0.0,
    };

    // Families that need to see the whole score at once — a ring has to know
    // who else is on it, a column has to know the column exists, a tiling has to
    // be grown large enough — plan every position up front. The per-item
    // families keep their original closed form below.
    let ordered: Vec<&ScoreItem> = order.iter().map(|(_, item)| *item).collect();
    // Bounded families (Partition, Sets) also size items and draw regions.
    let shaped = families::shape(&score.arrangement, &ordered);
    let arranged = match &shaped {
        Some(shape) => Some(shape.positions.clone()),
        None => families::arrange(&score.arrangement, &ordered),
    };
    let planned = match arranged {
        Some(planned) => Some(planned),
        // The four closed-form per-item families keep their original path below.
        None if families::is_per_item(&score.arrangement) => None,
        // `Arrangement::Custom`. Without a planner that resolves it, inventing
        // positions for an arrangement this crate cannot name would place every
        // item somewhere wrong without saying so. The holds still report, so the
        // caller learns what was asked for even though nothing was placed.
        None => match plan_custom(&ordered) {
            Some(planned) if planned.len() == ordered.len() => Some(planned),
            _ => return report_holds(score, scene),
        },
    };

    let mut bounds: Option<Rect> = None;
    for (rank, (_, item)) in order.into_iter().enumerate() {
        // An authored hold outranks the arrangement, in every family. Without
        // this, a Coordinate placement was honored by Geographic and Hulls and
        // silently discarded by Spiral and Grid, which is the same bug as
        // moving a pin: the person said where, and the layout answered
        // somewhere else without saying so.
        let position = match score.hold_for(&item.source) {
            Some(held) => held.at,
            None => match &planned {
                Some(planned) => planned[rank],
                None => match &score.arrangement {
                    Arrangement::Spiral(spiral) => spiral_position(spiral, effective_spacing, rank),
                    Arrangement::Grid(grid) => grid_position(grid, item, rank),
                    Arrangement::Geographic(geographic) => geographic_position(geographic, item),
                    Arrangement::Hulls(hulls) => hulls_position(hulls, item),
                    // Unreachable: `arrange` returns `Some` for every family
                    // that is not one of the four above, and `Custom` returned
                    // early. Spelled out rather than caught by a wildcard so a
                    // twelfth family fails to compile here instead of silently
                    // landing at the origin.
                    other => unreachable!("{other:?} has no per-item placement"),
                },
            },
        };
        let source = scene.intern_source(item.source.clone());
        let instance = ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(position.x, position.y),
            footprint: shaped
                .as_ref()
                .and_then(|shape| shape.footprints[rank].clone())
                .unwrap_or_else(|| item.footprint.clone()),
            representation: item.representation.clone(),
            layer: item.layer,
            visible: item.visible,
            hit: None,
            channels: Vec::new(),
        };
        if let Some(item_bounds) = placed_bounds(position, &instance.footprint) {
            bounds = Some(match bounds {
                Some(old) => old.union(item_bounds),
                None => item_bounds,
            });
        }
        scene.items.push(instance);
    }

    // Hulls emits the partition after every site is placed, because a cell is
    // defined against all the others: the part of the bounds nearer this site
    // than any of them.
    if let Arrangement::Hulls(hulls) = &score.arrangement {
        partition(hulls, &mut scene);
        bounds = Some(match bounds {
            Some(old) => old.union(hulls.bounds),
            None => hulls.bounds,
        });
    }

    // Items were pushed in rank order, so a rank is its instance.
    if let Some(shape) = shaped {
        for region in shape.regions {
            scene.regions.push(Region {
                members: region
                    .members
                    .into_iter()
                    .map(|rank| InstanceId(rank as u32))
                    .collect(),
                space: Scene::WORLD,
                contour: Some(Footprint::Polygon {
                    points: region.contour,
                }),
                label: region.label,
                confidence: 1.0,
            });
        }
        let area = match &score.arrangement {
            Arrangement::Partition(config) => Some(config.bounds),
            Arrangement::Setscape(config) => Some(config.bounds),
            _ => None,
        };
        if let Some(area) = area {
            bounds = Some(bounds.map_or(area, |old| old.union(area)));
        }
    }

    let mut scene = report_holds(score, scene);
    scene.bounds = bounds.unwrap_or_default();
    scene
}

/// Record both halves of the hold story on a scene.
///
/// Shared by the placed path and the early return, so an arrangement this crate
/// cannot solve still tells the caller which pins were asked for. A scene that
/// placed nothing *and* reported nothing would be indistinguishable from an
/// empty score.
fn report_holds(score: &Score, mut scene: Scene) -> Scene {
    // The positive half, bound to instances. Recorded for every instance a
    // held source reached, because a source may be placed more than once and
    // each placement is equally held.
    scene.honored_holds = scene
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let source = scene.sources.get(item.source.0 as usize)?;
            let held = score.hold_for(source)?;
            matches!(held.hold, Hold::Pinned).then(|| HonoredHold {
                instance: InstanceId(index as u32),
                placement: held.clone(),
            })
        })
        .collect();

    // A hold naming a source this score never placed is an unmet pin. Before
    // this it was dropped in silence, which is the same failure as moving a pin
    // and saying nothing. Seeded and anchored holds are excluded on purpose:
    // neither must be honored, so one that goes unplaced is not a violation.
    scene.unmet_holds = score
        .holds
        .iter()
        .filter(|held| matches!(held.hold, Hold::Pinned))
        .filter(|held| !scene.sources.iter().any(|source| *source == held.source))
        .cloned()
        .collect();

    scene
}

/// The instances a score pins, ready to hand to [`crate::relax_holding`].
///
/// Resolves through the scene's interned sources, so it stays correct when a
/// source appears more than once: every instance of a pinned source is pinned.
pub fn pinned_instances(score: &Score, scene: &Scene) -> Vec<InstanceId> {
    scene
        .items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let source = scene.sources.get(item.source.0 as usize)?;
            matches!(score.hold_for(source)?.hold, Hold::Pinned).then_some(InstanceId(index as u32))
        })
        .collect()
}

/// One scene [`Region`] per placed site: the bounds clipped by the
/// perpendicular-bisector half-plane against every other site. Exact rather
/// than sampled, and quadratic in sites, which place-graph counts never
/// stress.
fn partition(hulls: &Hulls, scene: &mut Scene) {
    let sites: Vec<(sceno::InstanceId, Vec2)> = scene
        .items
        .iter()
        .enumerate()
        .filter(|(_, item)| item.visible)
        .map(|(index, item)| {
            (
                sceno::InstanceId(index as u32),
                Vec2::new(item.transform.translate.x, item.transform.translate.y),
            )
        })
        .collect();

    for (id, site) in &sites {
        let mut cell = rect_polygon(hulls.bounds);
        for (other_id, other) in &sites {
            if other_id == id {
                continue;
            }
            cell = clip_nearer(cell, *site, *other);
            if cell.len() < 3 {
                break;
            }
        }
        if cell.len() < 3 {
            // Coincident sites can starve a cell to nothing. An empty region
            // would be a lie about coverage, so it is simply not emitted.
            continue;
        }
        scene.regions.push(Region {
            members: vec![*id],
            space: Scene::WORLD,
            contour: Some(Footprint::Polygon { points: cell }),
            label: None,
            confidence: 1.0,
        });
    }
}

fn rect_polygon(rect: Rect) -> Vec<Vec2> {
    let (x, y) = (rect.origin.x, rect.origin.y);
    let (w, h) = (rect.size.w, rect.size.h);
    vec![
        Vec2::new(x, y),
        Vec2::new(x + w, y),
        Vec2::new(x + w, y + h),
        Vec2::new(x, y + h),
    ]
}

/// Sutherland-Hodgman against the half-plane of points nearer `site` than
/// `other`: keep everything on `site`'s side of their perpendicular bisector.
fn clip_nearer(polygon: Vec<Vec2>, site: Vec2, other: Vec2) -> Vec<Vec2> {
    let mid = Vec2::new((site.x + other.x) / 2.0, (site.y + other.y) / 2.0);
    let away = Vec2::new(other.x - site.x, other.y - site.y);
    // Signed distance along `away`: negative is nearer `site`.
    let side = |p: Vec2| (p.x - mid.x) * away.x + (p.y - mid.y) * away.y;

    let mut out = Vec::with_capacity(polygon.len() + 1);
    for (index, &current) in polygon.iter().enumerate() {
        let previous = polygon[(index + polygon.len() - 1) % polygon.len()];
        let (d_prev, d_cur) = (side(previous), side(current));
        if d_prev <= 0.0 && d_cur > 0.0 || d_prev > 0.0 && d_cur <= 0.0 {
            // The edge crosses the bisector; keep the crossing point.
            let t = d_prev / (d_prev - d_cur);
            out.push(Vec2::new(
                previous.x + (current.x - previous.x) * t,
                previous.y + (current.y - previous.y) * t,
            ));
        }
        if d_cur <= 0.0 {
            out.push(current);
        }
    }
    out
}

fn hulls_position(hulls: &Hulls, item: &ScoreItem) -> Vec2 {
    let coordinate = match item.placement {
        Placement::Coordinate(coordinate) => coordinate,
        // A site is somewhere by definition; anything undisclosed sits at the
        // origin rather than being invented a position.
        _ => Vec2::ZERO,
    };
    Vec2::new(
        hulls.origin.x + coordinate.x * hulls.units_per_coordinate,
        hulls.origin.y
            + coordinate.y * hulls.units_per_coordinate * if hulls.invert_y { -1.0 } else { 1.0 },
    )
}

fn spiral_position(spiral: &Spiral, spacing: f32, ordinal: usize) -> Vec2 {
    let n = ordinal as f32;
    let radius = spacing
        * match spiral.curve {
            SpiralCurve::SquareRoot => n.sqrt(),
            SpiralCurve::Linear => n,
            SpiralCurve::Quadratic => n * n,
            SpiralCurve::Logarithmic => crate::math::ln(1.0 + n),
        };
    let angle = n * spiral.angle_radians;
    Vec2::new(
        spiral.center.x + radius * crate::math::cos(angle),
        spiral.center.y + radius * crate::math::sin(angle),
    )
}

fn grid_position(grid: &Grid, item: &ScoreItem, rank: usize) -> Vec2 {
    let columns = grid.columns.max(1) as usize;
    let (column, row) = match item.placement {
        Placement::Cell { column, row } => (column as f32, row as f32),
        _ => ((rank % columns) as f32, (rank / columns) as f32),
    };
    Vec2::new(
        grid.origin.x + column * (grid.cell.x + grid.gap),
        grid.origin.y + row * (grid.cell.y + grid.gap),
    )
}

fn geographic_position(geographic: &Geographic, item: &ScoreItem) -> Vec2 {
    let Placement::Coordinate(coordinate) = item.placement else {
        return geographic.origin;
    };
    Vec2::new(
        geographic.origin.x + coordinate.x * geographic.units_per_coordinate,
        geographic.origin.y
            + coordinate.y
                * geographic.units_per_coordinate
                * if geographic.invert_y { -1.0 } else { 1.0 },
    )
}

fn footprint_side(footprint: &Footprint) -> Option<f32> {
    footprint
        .bounds()
        .map(|bounds| bounds.size.w.max(bounds.size.h))
}

fn placed_bounds(position: Vec2, footprint: &Footprint) -> Option<Rect> {
    footprint.bounds().map(|local| {
        Rect::new(
            Vec2::new(position.x + local.origin.x, position.y + local.origin.y),
            local.size,
        )
    })
}

#[cfg(test)]
#[path = "solve/tests.rs"]
mod tests;
