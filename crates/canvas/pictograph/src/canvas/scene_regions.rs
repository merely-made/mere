// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The regions a bounded arrangement draws beneath its nodes: Partition's
//! group enclosures and remainder, Sets' circles and cores. They arrive with
//! the arrangement ([`super::physics_controls::apply_arrangement`]) and leave
//! with it.

use cartography::ProjectedRegion;
use paint_list_api::{
    ColorF, CommonPlacement, DashPattern, LayoutPoint, LayoutRect, PaintCmd, PathCommand, PathData,
    PathItem, StrokeCap, StrokeJoin, StrokeStyle,
};

use super::Canvas;

/// The lift accent: the grammar's ochre.
const LIFT: ColorF = ColorF {
    r: 0.78,
    g: 0.53,
    b: 0.23,
    a: 1.0,
};
/// How far a lifted tile stands out, in screen px.
const LIFT_PX: f32 = 4.0;

/// How a region reads: fill alpha, line alpha, line width, dashed. A share
/// fills, a core and the hatched-in-spirit remainder dash, a Mosaic tile (no
/// label) is a faint pane.
fn treatment(region: &ProjectedRegion) -> (Option<f32>, f32, f32, bool) {
    match region.label.as_deref() {
        Some("remainder") => (Some(0.04), 0.35, 1.0, true),
        Some(label) if label.ends_with("· core") => (None, 0.4, 1.0, true),
        None => (Some(0.05), 0.3, 1.0, false),
        _ => (Some(0.09), 0.45, 1.0, false),
    }
}

impl Canvas {
    /// The active arrangement's regions, replacing the last set.
    pub fn set_strategy_regions(&mut self, regions: Vec<ProjectedRegion>) {
        self.strategy_regions = regions;
    }

    pub fn strategy_regions(&self) -> &[ProjectedRegion] {
        &self.strategy_regions
    }

    /// A Mosaic tile whose node is selected stands out of the mosaic: an ochre
    /// outline a little larger, drawn last. Its neighbours do not move.
    fn lifted(&self, region: &ProjectedRegion) -> bool {
        region.label.is_none()
            && !region.members.is_empty()
            && region.members.iter().all(|key| self.selected.contains(key))
    }

    pub(crate) fn strategy_region_commands(&self) -> Vec<PaintCmd> {
        let ink = self.style.edge_color;
        let mut regions: Vec<&ProjectedRegion> = self
            .strategy_regions
            .iter()
            .filter(|region| region.contour.len() >= 3)
            .collect();
        regions.sort_by_key(|region| self.lifted(region));
        regions
            .into_iter()
            .map(|region| {
                let lifted = self.lifted(region);
                let mut points: Vec<LayoutPoint> = region
                    .contour
                    .iter()
                    .map(|p| {
                        let (x, y) = self.camera.to_screen(*p);
                        LayoutPoint::new(x, y)
                    })
                    .collect();
                if lifted {
                    lift(&mut points);
                }
                let mut commands: Vec<PathCommand> = points
                    .iter()
                    .enumerate()
                    .map(|(i, p)| {
                        if i == 0 {
                            PathCommand::MoveTo(*p)
                        } else {
                            PathCommand::LineTo(*p)
                        }
                    })
                    .collect();
                commands.push(PathCommand::Close);
                let (fill, line, width, dashed) = if lifted {
                    (Some(0.16), 1.0, 2.5, false)
                } else {
                    treatment(region)
                };
                let tone = if lifted { LIFT } else { ink };
                let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - 1.0;
                let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - 1.0;
                let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) + 1.0;
                let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) + 1.0;
                PaintCmd::DrawPath(PathItem {
                    placement: CommonPlacement::new(LayoutRect::new(
                        LayoutPoint::new(min_x, min_y),
                        LayoutPoint::new(max_x, max_y),
                    )),
                    path: PathData { commands },
                    fill: fill.map(|alpha| ColorF::new(tone.r, tone.g, tone.b, alpha)),
                    stroke: Some(StrokeStyle {
                        color: ColorF::new(tone.r, tone.g, tone.b, line),
                        width,
                        cap: StrokeCap::Round,
                        join: StrokeJoin::Round,
                        dash: dashed.then(|| DashPattern {
                            intervals: vec![5.0, 4.0],
                            offset: 0.0,
                        }),
                    }),
                })
            })
            .collect()
    }
}

/// Push every point `LIFT_PX` further from the region's centre on each axis.
fn lift(points: &mut [LayoutPoint]) {
    let (x0, y0, x1, y1) = points.iter().fold(
        (
            f32::INFINITY,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::NEG_INFINITY,
        ),
        |(x0, y0, x1, y1), p| (x0.min(p.x), y0.min(p.y), x1.max(p.x), y1.max(p.y)),
    );
    let (cx, cy) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
    let (hx, hy) = (((x1 - x0) / 2.0).max(1.0), ((y1 - y0) / 2.0).max(1.0));
    for p in points {
        p.x = cx + (p.x - cx) * (hx + LIFT_PX) / hx;
        p.y = cy + (p.y - cy) * (hy + LIFT_PX) / hy;
    }
}
