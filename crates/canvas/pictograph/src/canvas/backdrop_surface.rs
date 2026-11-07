// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Independently embeddable realization of Scenomise's disclosed-data scenes.
//! No graph, application session, persistence, private GPU, clock, or source
//! mutation is required. Hosts lend fonts, theme roles and a viewport.

use super::{BoardFit, BoardRect, BoardText};
use paint_list_api::{
    ColorF, CommonPlacement, DashPattern, DeviceIntSize, LayoutPoint, LayoutRect, PaintCmd,
    PaintList, PathCommand, PathData, PathItem, StrokeCap, StrokeJoin, StrokeStyle,
};
use paint_list_render::{CompositeLayer, composite_paint_layers};
use sceno::{Footprint, Vec2};
use sceno_backdrops::{
    BackdropScene, CoverageTier, MarkRole, SceneFacts, SceneInput, SceneInputError,
};
use std::collections::BTreeSet;

/// Applied appearance belongs to the embedding host. Defaults are the quiet
/// study palette, not an activation of a Tabard theme.
#[derive(Clone, Debug)]
pub struct SurfaceTheme {
    pub ground: [f32; 4],
    pub text: [f32; 4],
    pub guide: [f32; 4],
    pub category: [f32; 4],
    pub root: [f32; 4],
    pub fifth: [f32; 4],
    pub warning: [f32; 4],
}
impl Default for SurfaceTheme {
    fn default() -> Self {
        Self {
            ground: [0.13, 0.15, 0.12, 1.0],
            text: [0.88, 0.90, 0.85, 1.0],
            guide: [0.57, 0.63, 0.54, 0.28],
            category: [0.63, 0.73, 0.60, 1.0],
            root: [0.84, 0.72, 0.48, 1.0],
            fifth: [0.62, 0.74, 0.79, 1.0],
            warning: [0.86, 0.64, 0.58, 1.0],
        }
    }
}

/// Presentation transform. Sequence rows can use the available height without
/// shortening musical spans or changing the source's coordinates.
struct SurfaceFit {
    scale_x: f32,
    scale_y: f32,
    origin: (f32, f32),
}
impl SurfaceFit {
    fn to_viewport(&self, (x, y): (f32, f32)) -> (f32, f32) {
        (
            self.origin.0 + x * self.scale_x,
            self.origin.1 + y * self.scale_y,
        )
    }
}

pub struct BackdropSurface {
    compiled: BackdropScene,
    selected: Option<String>,
    retained: BTreeSet<String>,
    text: BoardText,
    font_ready: bool,
    fill_template_height: bool,
    pub show_coverage: bool,
    pub show_links: bool,
    /// Embedding owner opts into span affordances when it supplies editing.
    pub show_span_handles: bool,
}

fn alpha(mut c: [f32; 4], a: f32) -> [f32; 4] {
    c[3] *= a;
    c
}
fn stroke(c: [f32; 4], width: f32, dash: Option<DashPattern>) -> StrokeStyle {
    StrokeStyle {
        color: ColorF::new(c[0], c[1], c[2], c[3]),
        width,
        cap: StrokeCap::Round,
        join: StrokeJoin::Round,
        dash,
    }
}
fn rect(c: LayoutPoint, w: f32, h: f32) -> Vec<LayoutPoint> {
    vec![
        LayoutPoint::new(c.x - w / 2.0, c.y - h / 2.0),
        LayoutPoint::new(c.x + w / 2.0, c.y - h / 2.0),
        LayoutPoint::new(c.x + w / 2.0, c.y + h / 2.0),
        LayoutPoint::new(c.x - w / 2.0, c.y + h / 2.0),
    ]
}
fn circle(c: LayoutPoint, r: f32) -> Vec<LayoutPoint> {
    (0..40)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 40.0;
            LayoutPoint::new(c.x + r * libm::cosf(a), c.y + r * libm::sinf(a))
        })
        .collect()
}
fn path(
    points: &[LayoutPoint],
    closed: bool,
    fill: Option<[f32; 4]>,
    stroke: Option<StrokeStyle>,
) -> PaintCmd {
    let mut commands = Vec::new();
    for (i, p) in points.iter().enumerate() {
        commands.push(if i == 0 {
            PathCommand::MoveTo(*p)
        } else {
            PathCommand::LineTo(*p)
        });
    }
    if closed {
        commands.push(PathCommand::Close);
    }
    let padding = stroke.as_ref().map_or(0.0, |s| s.width / 2.0);
    let bounds = LayoutRect::new(
        LayoutPoint::new(
            points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - padding,
            points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - padding,
        ),
        LayoutPoint::new(
            points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) + padding,
            points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) + padding,
        ),
    );
    PaintCmd::DrawPath(PathItem {
        placement: CommonPlacement::new(bounds),
        path: PathData { commands },
        fill: fill.map(|c| ColorF::new(c[0], c[1], c[2], c[3])),
        stroke,
    })
}
// Even-odd clipping of diagonal hatch lines against a supplied polygon.
fn hatch(out: &mut Vec<PaintCmd>, polygon: &[LayoutPoint], color: [f32; 4]) {
    let min = polygon
        .iter()
        .map(|p| p.x + p.y)
        .fold(f32::INFINITY, f32::min);
    let max = polygon
        .iter()
        .map(|p| p.x + p.y)
        .fold(f32::NEG_INFINITY, f32::max);
    let mut k = (min / 10.0).floor() * 10.0;
    while k <= max {
        let mut hits = Vec::new();
        for i in 0..polygon.len() {
            let a = polygon[i];
            let b = polygon[(i + 1) % polygon.len()];
            let d = b.x + b.y - a.x - a.y;
            if d.abs() > 1e-6 {
                let t = (k - a.x - a.y) / d;
                if (0.0..1.0).contains(&t) {
                    hits.push(LayoutPoint::new(
                        a.x + (b.x - a.x) * t,
                        a.y + (b.y - a.y) * t,
                    ));
                }
            }
        }
        hits.sort_by(|a, b| a.x.total_cmp(&b.x));
        for pair in hits.chunks_exact(2) {
            out.push(path(pair, false, None, Some(stroke(color, 0.7, None))));
        }
        k += 10.0;
    }
}

#[cfg(test)]
#[path = "backdrop_surface/tests.rs"]
mod tests;

#[path = "backdrop_surface/operations.rs"]
mod operations;
