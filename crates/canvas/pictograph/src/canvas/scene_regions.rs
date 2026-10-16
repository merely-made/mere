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

/// How a region reads: a filled share, a dashed core, or the hatched-in-spirit
/// remainder (dashed and fainter).
fn treatment(region: &ProjectedRegion) -> (Option<f32>, f32, bool) {
    match region.label.as_deref() {
        Some("remainder") => (Some(0.04), 0.35, true),
        Some(label) if label.ends_with("· core") => (None, 0.4, true),
        _ => (Some(0.09), 0.45, false),
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

    pub(crate) fn strategy_region_commands(&self) -> Vec<PaintCmd> {
        let ink = self.style.edge_color;
        self.strategy_regions
            .iter()
            .filter(|region| region.contour.len() >= 3)
            .map(|region| {
                let points: Vec<LayoutPoint> = region
                    .contour
                    .iter()
                    .map(|p| {
                        let (x, y) = self.camera.to_screen(*p);
                        LayoutPoint::new(x, y)
                    })
                    .collect();
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
                let (fill, line, dashed) = treatment(region);
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
                    fill: fill.map(|alpha| ColorF::new(ink.r, ink.g, ink.b, alpha)),
                    stroke: Some(StrokeStyle {
                        color: ColorF::new(ink.r, ink.g, ink.b, line),
                        width: 1.0,
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
