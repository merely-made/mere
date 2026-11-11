// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The territory field (graph grammar, DECISIONS 2026-10-16; the specimen's
//! 2a, "the base scene language"): each domain claims the ground nearest its
//! nodes, tiered by how clearly it belongs, that is, by the margin by which
//! the nearest domain beats the runner-up. It is a field over wherever the
//! nodes are, so it works under forces or any arrangement, and it is drawn
//! beneath everything else. Domains are sites, coloured by category (the
//! theme's, when the host passes them; `grammar_view.rs`).

use std::collections::BTreeMap;

use kernel::geometry::PortablePoint;
use paint_list_api::{ColorF, CommonPlacement, LayoutPoint, LayoutRect, PaintCmd, RectItem};

use super::Canvas;

/// Maximum cells on either grid axis. The shorter axis follows the aspect,
/// bounding the number of samples even for a very tall or wide viewport.
const COLUMNS: usize = 48;
/// The three tiers' fill alphas (`--g-tier-1…3`).
pub const TIER_ALPHA: [f32; 3] = [0.06, 0.11, 0.17];
/// Beyond this many nodes the field is not drawn: the grid costs columns ×
/// rows × nodes each frame.
pub const MAX_TERRITORY_NODES: usize = 2000;

/// One run of grid cells in the sampling plane, claimed by one domain.
/// [`territory`] returns world coordinates; the canvas samples its viewport
/// in screen coordinates while measuring claims through the inverse camera.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerritoryRun {
    pub min: PortablePoint,
    pub max: PortablePoint,
    pub domain: usize,
    /// 1 (contested) to 3 (clearly held).
    pub tier: u8,
}

/// The territory of `sites` (position, domain index) over their bounds grown
/// by `margin` on every side, as row runs. A cell's tier is the gap between
/// its nearest domain's distance and the nearest other domain's, in units of
/// `unit`: under one is tier 1, under two tier 2, else tier 3. With one domain
/// everything it reaches is tier 3.
pub fn territory(sites: &[(PortablePoint, usize)], margin: f32, unit: f32) -> Vec<TerritoryRun> {
    let Some(first) = sites.first() else {
        return Vec::new();
    };
    let (mut x0, mut y0, mut x1, mut y1) = (first.0.x, first.0.y, first.0.x, first.0.y);
    for (p, _) in sites {
        (x0, y0, x1, y1) = (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y));
    }
    let (x0, y0, x1, y1) = (x0 - margin, y0 - margin, x1 + margin, y1 + margin);
    territory_in_rect(sites, [x0, y0, x1, y1], unit, |x, y| (x, y))
}

fn territory_in_rect(
    sites: &[(PortablePoint, usize)],
    [x0, y0, x1, y1]: [f32; 4],
    unit: f32,
    to_world: impl Fn(f32, f32) -> (f32, f32),
) -> Vec<TerritoryRun> {
    let (width, height) = (x1 - x0, y1 - y0);
    if sites.is_empty()
        || !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
    {
        return Vec::new();
    }
    let cell = width.max(height) / COLUMNS as f32;
    let columns = (width / cell).ceil().clamp(1.0, COLUMNS as f32) as usize;
    let rows = (height / cell).ceil().clamp(1.0, COLUMNS as f32) as usize;
    let unit = unit.max(f32::EPSILON);
    let domains = sites.iter().map(|(_, d)| d + 1).max().unwrap_or(1);
    let mut best = vec![f32::INFINITY; domains];
    let mut runs: Vec<TerritoryRun> = Vec::new();
    for row in 0..rows {
        let bottom = if row + 1 == rows {
            y1
        } else {
            (y0 + (row + 1) as f32 * cell).min(y1)
        };
        let top = y0 + row as f32 * cell;
        let y = (top + bottom) * 0.5;
        let mut open: Option<TerritoryRun> = None;
        for column in 0..columns {
            let left = x0 + column as f32 * cell;
            let right = if column + 1 == columns {
                x1
            } else {
                (x0 + (column + 1) as f32 * cell).min(x1)
            };
            let (x, y) = to_world((left + right) * 0.5, y);
            let claim = claim_at(sites, x, y, unit, &mut best);
            let lo = PortablePoint::new(left, top);
            let hi = PortablePoint::new(right, bottom);
            match (&mut open, claim) {
                (Some(run), (domain, tier)) if run.domain == domain && run.tier == tier => {
                    run.max = hi;
                },
                (_, (domain, tier)) => {
                    runs.extend(open.take());
                    open = Some(TerritoryRun {
                        min: lo,
                        max: hi,
                        domain,
                        tier,
                    });
                },
            }
        }
        runs.extend(open);
    }
    runs
}

/// The domain nearest `(x, y)` and its tier. `best` is scratch, one slot per
/// domain index, so a frame's grid allocates nothing per cell.
fn claim_at(
    sites: &[(PortablePoint, usize)],
    x: f32,
    y: f32,
    unit: f32,
    best: &mut [f32],
) -> (usize, u8) {
    best.fill(f32::INFINITY);
    for (p, domain) in sites {
        let d = (p.x - x).powi(2) + (p.y - y).powi(2);
        if d < best[*domain] {
            best[*domain] = d;
        }
    }
    let (mut first, mut d1, mut d2) = (0, f32::INFINITY, f32::INFINITY);
    for (domain, d) in best.iter().copied().enumerate() {
        if d < d1 {
            (d2, d1, first) = (d1, d, domain);
        } else if d < d2 {
            d2 = d;
        }
    }
    let tier = if d2.is_infinite() {
        3
    } else {
        match (d2.sqrt() - d1.sqrt()) / unit {
            m if m < 1.0 => 1,
            m if m < 2.0 => 2,
            _ => 3,
        }
    };
    (first, tier)
}

impl Canvas {
    /// Draw the territory field beneath the graph, or stop drawing it.
    pub fn set_territory_field(&mut self, on: bool) {
        self.territory_field = on;
    }

    pub fn territory_field(&self) -> bool {
        self.territory_field
    }

    /// Cover the entire visible viewport. Runs are in screen coordinates;
    /// nearest-domain distances are measured in world coordinates, so pan,
    /// zoom and camera rotation retain the same underlying spatial field.
    pub(crate) fn territory_runs(&mut self) -> Vec<TerritoryRun> {
        if !self.territory_field || self.graph.nodes().count() > MAX_TERRITORY_NODES {
            return Vec::new();
        }
        let sites = self.channels.sites(&self.graph).clone();
        let mut names: Vec<&String> = sites.values().collect();
        names.sort();
        names.dedup();
        let index: BTreeMap<&String, usize> =
            names.into_iter().enumerate().map(|(i, n)| (n, i)).collect();
        let placed: Vec<(PortablePoint, usize)> = self
            .view
            .positions()
            .filter(|(key, _)| self.node_visible_in_canvas(*key))
            .filter_map(|(key, at)| {
                let site = sites.get(&key)?;
                Some((PortablePoint::new(at.x, at.y), index[site]))
            })
            .collect();
        let body = seiche::NODE_BODY_RADIUS;
        territory_in_rect(
            &placed,
            [0.0, 0.0, self.view_w as f32, self.view_h as f32],
            body * 2.0,
            |x, y| {
                let world = self.screen_to_world((x, y));
                (world.x, world.y)
            },
        )
    }

    pub(crate) fn territory_commands(&mut self) -> Vec<PaintCmd> {
        self.territory_runs()
            .into_iter()
            .map(|run| {
                let (r, g, b) = self.category_rgb(run.domain);
                PaintCmd::DrawRect(RectItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(run.min.x, run.min.y),
                        LayoutPoint::new(run.max.x, run.max.y),
                    )),
                    color: ColorF::new(r, g, b, TIER_ALPHA[run.tier as usize - 1]),
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_sampling_covers_all_edges_and_stays_bounded_at_extreme_aspects() {
        let sites = [
            (PortablePoint::new(-2000.0, 0.0), 0),
            (PortablePoint::new(2000.0, 0.0), 1),
        ];
        for (width, height) in [(1200.0, 300.0), (90.0, 900.0), (1.0, 10000.0)] {
            let runs = territory_in_rect(&sites, [0.0, 0.0, width, height], 20.0, |x, y| (x, y));
            assert!(runs.len() <= COLUMNS * COLUMNS);
            let area: f64 = runs
                .iter()
                .map(|r| f64::from(r.max.x - r.min.x) * f64::from(r.max.y - r.min.y))
                .sum();
            assert!((area - f64::from(width) * f64::from(height)).abs() < 0.1);
            assert!(runs.iter().all(|r| r.min.x >= 0.0
                && r.min.y >= 0.0
                && r.max.x <= width
                && r.max.y <= height));
            assert!(runs.iter().any(|r| r.max.x == width && r.max.y == height));
        }
    }

    #[test]
    fn viewport_sampling_measures_claims_through_the_inverse_camera() {
        let sites = [
            (PortablePoint::new(-100.0, 0.0), 0),
            (PortablePoint::new(100.0, 0.0), 1),
        ];
        let runs = territory_in_rect(&sites, [0.0, 0.0, 800.0, 600.0], 20.0, |x, y| {
            (x - 600.0, y - 300.0)
        });
        for (x, domain) in [(100.0, 0), (550.0, 0), (700.0, 1)] {
            let run = runs
                .iter()
                .find(|r| r.min.x <= x && x < r.max.x && r.min.y <= 300.0 && 300.0 < r.max.y)
                .unwrap();
            assert_eq!(run.domain, domain);
        }
        assert!(territory_in_rect(&sites, [0.0; 4], 20.0, |x, y| (x, y)).is_empty());
    }

    #[test]
    fn each_domain_claims_its_side_most_clearly_far_from_the_border() {
        let sites = [
            (PortablePoint::new(-100.0, 0.0), 0),
            (PortablePoint::new(-80.0, 20.0), 0),
            (PortablePoint::new(100.0, 0.0), 1),
        ];
        let runs = territory(&sites, 60.0, 20.0);
        let at = |x: f32, y: f32| {
            runs.iter()
                .find(|r| r.min.x <= x && x <= r.max.x && r.min.y <= y && y <= r.max.y)
                .map(|r| (r.domain, r.tier))
                .unwrap()
        };
        assert_eq!(at(-140.0, 0.0), (0, 3), "deep in domain 0");
        assert_eq!(at(140.0, 0.0), (1, 3), "deep in domain 1");
        let (_, border) = at(10.0, 0.0);
        assert_eq!(border, 1, "contested near the bisector");
    }

    #[test]
    fn runs_merge_along_rows_and_one_domain_holds_everything_clearly() {
        let sites = [
            (PortablePoint::new(0.0, 0.0), 4),
            (PortablePoint::new(50.0, 10.0), 4),
        ];
        let runs = territory(&sites, 30.0, 10.0);
        assert!(runs.iter().all(|r| r.domain == 4 && r.tier == 3));
        let rows = runs.len();
        assert!(rows < COLUMNS, "one run per row, {rows} runs");
        assert!(territory(&[], 10.0, 10.0).is_empty());
    }
}
