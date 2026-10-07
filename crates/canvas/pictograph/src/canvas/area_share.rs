// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! How much room each node holds, and whether room follows mass: the Density
//! law's signature, measured from positions alone so it reads any law.
//!
//! A node's area share is its discrete Voronoi cell: a raster over the nodes'
//! bounding box, grown by half the mean nearest-neighbour distance, with each
//! raster cell given to its nearest node. From the shares come the Spearman
//! rank correlation of mass against area and the coefficient of variation of
//! `mass / area` (zero when density is perfectly even).

/// Raster cells along the longer side of the measured box.
const RASTER: usize = 128;

/// Each point's discrete Voronoi area, in world units², in input order.
pub(crate) fn area_shares(points: &[(f32, f32)]) -> Vec<f32> {
    area_shares_and_rim(points).0
}

/// The areas, and whether each point's cell reaches the measured box's rim
/// (its area then depends on the clip, not on its neighbours alone).
pub(crate) fn area_shares_and_rim(points: &[(f32, f32)]) -> (Vec<f32>, Vec<bool>) {
    let n = points.len();
    if n < 2 {
        return (vec![0.0; n], vec![true; n]);
    }
    let margin = mean_nearest(points) / 2.0;
    let (mut lo, mut hi) = (points[0], points[0]);
    for &(x, y) in points {
        lo = (lo.0.min(x), lo.1.min(y));
        hi = (hi.0.max(x), hi.1.max(y));
    }
    let (lo, hi) = (
        (lo.0 - margin, lo.1 - margin),
        (hi.0 + margin, hi.1 + margin),
    );
    let longest = (hi.0 - lo.0).max(hi.1 - lo.1).max(1e-3);
    let cell = longest / RASTER as f32;
    let cols = ((hi.0 - lo.0) / cell).ceil().max(1.0) as usize;
    let rows = ((hi.1 - lo.1) / cell).ceil().max(1.0) as usize;
    let buckets = Buckets::new(points, lo, hi);
    let mut area = vec![0.0f32; n];
    let mut rim = vec![false; n];
    for row in 0..rows {
        for col in 0..cols {
            let p = (
                lo.0 + (col as f32 + 0.5) * cell,
                lo.1 + (row as f32 + 0.5) * cell,
            );
            let owner = buckets.nearest(points, p);
            area[owner] += cell * cell;
            if row == 0 || col == 0 || row + 1 == rows || col + 1 == cols {
                rim[owner] = true;
            }
        }
    }
    (area, rim)
}

/// The mean distance from each point to its nearest other point.
fn mean_nearest(points: &[(f32, f32)]) -> f32 {
    let mut total = 0.0;
    for (i, a) in points.iter().enumerate() {
        let mut best = f32::MAX;
        for (j, b) in points.iter().enumerate() {
            if i != j {
                best = best.min((a.0 - b.0).hypot(a.1 - b.1));
            }
        }
        total += best;
    }
    total / points.len() as f32
}

/// A uniform bucket grid over the points, for nearest-point queries.
struct Buckets {
    lo: (f32, f32),
    size: f32,
    side: usize,
    cells: Vec<Vec<usize>>,
}

impl Buckets {
    fn new(points: &[(f32, f32)], lo: (f32, f32), hi: (f32, f32)) -> Self {
        let side = (points.len() as f32).sqrt().ceil().max(1.0) as usize;
        let size = ((hi.0 - lo.0).max(hi.1 - lo.1) / side as f32).max(1e-3);
        let mut cells = vec![Vec::new(); side * side];
        let mut buckets = Self {
            lo,
            size,
            side,
            cells: Vec::new(),
        };
        for (i, &p) in points.iter().enumerate() {
            let (c, r) = buckets.index(p);
            cells[r * side + c].push(i);
        }
        buckets.cells = cells;
        buckets
    }

    fn index(&self, p: (f32, f32)) -> (usize, usize) {
        let clamp = |v: f32| (v.max(0.0) as usize).min(self.side - 1);
        (
            clamp((p.0 - self.lo.0) / self.size),
            clamp((p.1 - self.lo.1) / self.size),
        )
    }

    /// The nearest point to `p`: rings of buckets outward until the ring is
    /// farther than the best found.
    fn nearest(&self, points: &[(f32, f32)], p: (f32, f32)) -> usize {
        let (c, r) = self.index(p);
        let mut best = (f32::MAX, 0usize);
        for ring in 0..=self.side as i32 {
            if best.0 < f32::MAX && (ring as f32 - 1.0) * self.size > best.0 {
                break;
            }
            for dr in -ring..=ring {
                for dc in -ring..=ring {
                    if dr.abs() != ring && dc.abs() != ring {
                        continue;
                    }
                    let (cc, rr) = (c as i32 + dc, r as i32 + dr);
                    if cc < 0 || rr < 0 || cc >= self.side as i32 || rr >= self.side as i32 {
                        continue;
                    }
                    for &i in &self.cells[rr as usize * self.side + cc as usize] {
                        let d = (points[i].0 - p.0).hypot(points[i].1 - p.1);
                        if d < best.0 || (d == best.0 && i < best.1) {
                            best = (d, i);
                        }
                    }
                }
            }
        }
        best.1
    }
}

/// Ranks with ties averaged (1-based).
fn ranks(values: &[f32]) -> Vec<f32> {
    let mut order: Vec<usize> = (0..values.len()).collect();
    order.sort_by(|&a, &b| values[a].total_cmp(&values[b]));
    let mut ranks = vec![0.0; values.len()];
    let mut i = 0;
    while i < order.len() {
        let mut j = i;
        while j + 1 < order.len() && values[order[j + 1]] == values[order[i]] {
            j += 1;
        }
        let rank = (i + j) as f32 / 2.0 + 1.0;
        for &k in &order[i..=j] {
            ranks[k] = rank;
        }
        i = j + 1;
    }
    ranks
}

/// Spearman's rank correlation; zero when either side is constant.
pub(crate) fn spearman(a: &[f32], b: &[f32]) -> f32 {
    let (ra, rb) = (ranks(a), ranks(b));
    let n = ra.len() as f32;
    if n < 2.0 {
        return 0.0;
    }
    let (ma, mb) = (ra.iter().sum::<f32>() / n, rb.iter().sum::<f32>() / n);
    let (mut cov, mut va, mut vb) = (0.0, 0.0, 0.0);
    for (x, y) in ra.iter().zip(&rb) {
        cov += (x - ma) * (y - mb);
        va += (x - ma) * (x - ma);
        vb += (y - mb) * (y - mb);
    }
    if va <= 0.0 || vb <= 0.0 {
        return 0.0;
    }
    cov / (va * vb).sqrt()
}

/// `(Spearman of mass against area share, CV of mass / area)` over the
/// points; zeros with fewer than two points.
pub(crate) fn mass_area_stats(points: &[(f32, f32)], masses: &[f32]) -> (f32, f32) {
    if points.len() < 2 {
        return (0.0, 0.0);
    }
    let areas = area_shares(points);
    let rank = spearman(masses, &areas);
    let density: Vec<f32> = masses
        .iter()
        .zip(&areas)
        .map(|(m, a)| m / a.max(1e-3))
        .collect();
    let mean = density.iter().sum::<f32>() / density.len() as f32;
    let var = density.iter().map(|d| (d - mean) * (d - mean)).sum::<f32>() / density.len() as f32;
    let cv = if mean > 0.0 { var.sqrt() / mean } else { 0.0 };
    (rank, cv)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spearman_reads_order_not_scale_and_averages_ties() {
        assert!((spearman(&[1.0, 2.0, 3.0], &[10.0, 200.0, 3000.0]) - 1.0).abs() < 1e-6);
        assert!((spearman(&[1.0, 2.0, 3.0], &[3.0, 2.0, 1.0]) + 1.0).abs() < 1e-6);
        assert_eq!(spearman(&[1.0, 1.0, 1.0], &[1.0, 2.0, 3.0]), 0.0);
        assert_eq!(ranks(&[5.0, 1.0, 5.0]), vec![2.5, 1.0, 2.5]);
    }

    /// A square lattice shares its area evenly; a point given a wider gap
    /// around it gets the bigger cell.
    #[test]
    fn a_lattice_shares_evenly_and_a_gap_is_room() {
        let mut points = Vec::new();
        for r in 0..6 {
            for c in 0..6 {
                points.push((c as f32 * 50.0, r as f32 * 50.0));
            }
        }
        let areas = area_shares(&points);
        let (min, max) = areas
            .iter()
            .fold((f32::MAX, 0.0f32), |(lo, hi), a| (lo.min(*a), hi.max(*a)));
        assert!(max / min < 1.15, "even lattice: {min} .. {max}");
        let (rank, cv) = mass_area_stats(&points, &vec![1.0; points.len()]);
        assert_eq!(rank, 0.0, "uniform mass has no rank");
        assert!(cv < 0.06, "even lattice cv {cv}");
        // Push the inner neighbours of point 14 away: it gains room.
        let mut spread = points.clone();
        for &i in &[8, 13, 15, 20] {
            let (x, y) = spread[i];
            let (cx, cy) = points[14];
            spread[i] = (cx + (x - cx) * 1.4, cy + (y - cy) * 1.4);
        }
        let wide = area_shares(&spread);
        assert!(wide[14] > areas[14] * 1.5, "{} vs {}", wide[14], areas[14]);
    }
}
