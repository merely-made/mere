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

/// Grid columns across the claimed ground; rows follow the aspect.
const COLUMNS: usize = 48;
/// The three tiers' fill alphas (`--g-tier-1…3`).
pub const TIER_ALPHA: [f32; 3] = [0.06, 0.11, 0.17];
/// Beyond this many nodes the field is not drawn: the grid costs columns ×
/// rows × nodes each frame.
pub const MAX_TERRITORY_NODES: usize = 2000;

/// One run of grid cells in world space that one domain claims at one tier.
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
    let cell = ((x1 - x0) / COLUMNS as f32).max(f32::EPSILON);
    let rows = ((y1 - y0) / cell).ceil().max(1.0) as usize;
    let unit = unit.max(f32::EPSILON);
    let domains = sites.iter().map(|(_, d)| d + 1).max().unwrap_or(1);
    let mut best = vec![f32::INFINITY; domains];
    let mut runs: Vec<TerritoryRun> = Vec::new();
    for row in 0..rows {
        let y = y0 + (row as f32 + 0.5) * cell;
        let mut open: Option<TerritoryRun> = None;
        for column in 0..COLUMNS {
            let x = x0 + (column as f32 + 0.5) * cell;
            let claim = claim_at(sites, x, y, unit, &mut best);
            let lo = PortablePoint::new(x0 + column as f32 * cell, y0 + row as f32 * cell);
            let hi = PortablePoint::new(lo.x + cell, lo.y + cell);
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

    /// The field's runs for the current positions, domains as sites in
    /// sorted order (so a domain keeps its colour).
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
        territory(&placed, body * 6.0, body * 2.0)
    }

    pub(crate) fn territory_commands(&mut self) -> Vec<PaintCmd> {
        self.territory_runs()
            .into_iter()
            .map(|run| {
                let (ax, ay) = self.camera.to_screen(run.min);
                let (bx, by) = self.camera.to_screen(run.max);
                let (r, g, b) = self.category_rgb(run.domain);
                PaintCmd::DrawRect(RectItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(ax.min(bx), ay.min(by)),
                        LayoutPoint::new(ax.max(bx), ay.max(by)),
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
