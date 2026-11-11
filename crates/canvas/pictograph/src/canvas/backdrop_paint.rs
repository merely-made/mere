// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use emblem::{Palette, Rgba, ViewBox};
use paint_list_api::{
    ColorF, LayoutPoint, LayoutTransform, PaintCmd, TransformKind, TransformSpec,
};

use crate::backdrop::{CLEAR, LATTICE, MOTIF, OTHER, PALETTE_ENTRIES, ViewDigest, derive_backdrop};
use crate::flat_icon::wash_commands;

use super::{Canvas, scene_paint::Camera, territory::TIER_ALPHA};

const WASH_ALPHA: u8 = (TIER_ALPHA[2] * 255.0) as u8;
const MOTIF_ALPHA: u8 = WASH_ALPHA / 2;
// sRGB's linearization slope is at most 2.275, so limiting each channel
// to this interval caps relative luminance distance at 0.183.
const CHANNEL_DISTANCE: f32 = 0.08;

impl Canvas {
    /// Paint the held view's content wash beneath territory. Off by default.
    pub fn set_content_backdrop(&mut self, enabled: bool) {
        self.content_backdrop = enabled;
    }

    pub(super) fn content_backdrop_commands(&self) -> Vec<PaintCmd> {
        if !self.content_backdrop || !self.camera.is_valid() {
            return Vec::new();
        }
        let Some(digest) = self.backdrop_digest() else {
            return Vec::new();
        };
        let (transform, height) = wash_projection(&digest, self.camera);
        let Ok(file) = derive_backdrop(&digest).inspect_err(|error| {
            tracing::debug!(%error, "could not derive content backdrop");
        }) else {
            return Vec::new();
        };
        let Ok(mut commands) = wash_commands(&file, &self.content_backdrop_palette(), height)
            .inspect_err(|error| tracing::debug!(%error, "could not paint content backdrop"))
        else {
            return Vec::new();
        };
        commands.insert(
            0,
            PaintCmd::PushTransform(TransformSpec {
                origin: LayoutPoint::zero(),
                transform,
                kind: TransformKind::Standard,
            }),
        );
        commands.push(PaintCmd::PopTransform);
        commands
    }

    fn content_backdrop_palette(&self) -> Palette {
        let edge = self.style.edge_color;
        let neutral = [edge.r, edge.g, edge.b];
        let mut colors = [neutral; PALETTE_ENTRIES];
        for (index, color) in colors[..usize::from(OTHER)].iter_mut().enumerate() {
            let (r, g, b) = self.category_rgb(index);
            *color = [r, g, b];
        }
        wash_palette(self.backdrop, colors)
    }
}

fn wash_projection(digest: &ViewDigest, camera: Camera) -> (LayoutTransform, f32) {
    let [left, top, right, bottom] = digest.bounds();
    let lattice = f64::from(LATTICE);
    let width = ((f64::from(right) - f64::from(left)) * lattice) as f32;
    let height = ((f64::from(bottom) - f64::from(top)) * lattice) as f32;
    let view = ViewBox::default();
    let sx = width / (view.max_x - view.min_x);
    let sy = height / (view.max_y - view.min_y);
    let x = (f64::from(left) * lattice) as f32 - view.min_x * sx;
    let y = (f64::from(top) * lattice) as f32 - view.min_y * sy;
    let transform = LayoutTransform::scale(sx, sy, 1.0)
        .then(&LayoutTransform::translation(x, y, 0.0))
        .then(&camera.ground_transform());
    // Project the view box's vertical side to get its raster height, including
    // the ground plane's yaw and foreshortening.
    let projected_height = transform.m21.hypot(transform.m22) * (view.max_y - view.min_y);
    (transform, projected_height)
}

fn wash_palette(ground: ColorF, colors: [[f32; 3]; PALETTE_ENTRIES]) -> Palette {
    let ground = [ground.r, ground.g, ground.b].map(|channel| channel.max(0.0).min(1.0));
    let mut palette = Palette::default();
    for (index, color) in colors.into_iter().enumerate() {
        let alpha = match index as u8 {
            MOTIF => MOTIF_ALPHA,
            CLEAR => 0,
            _ => WASH_ALPHA,
        };
        let a = f32::from(alpha);
        // Clamp in the premultiplied byte domain so quantization also respects
        // the interval. A transparent slot resolves to [0, 0, 0, 0].
        let channel = |i: usize| {
            let low = ((ground[i] - CHANNEL_DISTANCE).max(0.0) * a).ceil();
            let high = ((ground[i] + CHANNEL_DISTANCE).min(1.0) * a).floor();
            (color[i] * a).round().clamp(low, high) as u8
        };
        palette.0[index] = Rgba::new(channel(0), channel(1), channel(2), alpha);
    }
    palette
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backdrop::{BackdropKind, BackdropNode, LOD_HEIGHT};
    use kernel::geometry::PortablePoint;
    use kernel::graph::{Graph, fixtures::GraphFixtures};
    use netrender::SceneOp;

    #[test]
    fn one_backdrop_file_recolors_and_selects_motif_by_raster_height() {
        let digest = ViewDigest::from_held([BackdropNode {
            position: [1.0, 2.0],
            kind: BackdropKind::Document,
            community: None,
            site: None,
            visit_seconds: None,
        }])
        .unwrap();
        let file = derive_backdrop(&digest).unwrap();
        let ground = ColorF::new(0.45, 0.45, 0.45, 1.0);
        let red = wash_palette(ground, [[1.0, 0.0, 0.0]; PALETTE_ENTRIES]);
        let blue = wash_palette(ground, [[0.0, 0.0, 1.0]; PALETTE_ENTRIES]);
        let small = wash_commands(&file, &red, LOD_HEIGHT - 1.0).unwrap();
        let red = wash_commands(&file, &red, LOD_HEIGHT).unwrap();
        let blue = wash_commands(&file, &blue, LOD_HEIGHT).unwrap();
        let [
            PaintCmd::DrawRadialGradient(red_wash),
            PaintCmd::DrawPath(red_motif),
        ] = red.as_slice()
        else {
            panic!("expected a radial wash and a flat motif");
        };
        let [
            PaintCmd::DrawRadialGradient(blue_wash),
            PaintCmd::DrawPath(blue_motif),
        ] = blue.as_slice()
        else {
            panic!("expected a radial wash and a flat motif");
        };
        assert_eq!(small, red[..1]);
        assert_eq!(red_wash.placement, blue_wash.placement);
        assert_eq!(red_wash.gradient.center, blue_wash.gradient.center);
        assert_eq!(red_wash.gradient.radius, blue_wash.gradient.radius);
        assert_ne!(red_wash.gradient.stops, blue_wash.gradient.stops);
        assert_eq!(red_motif.path, blue_motif.path);
        assert_ne!(red_motif.fill, blue_motif.fill);
    }

    #[test]
    fn wash_palette_caps_relative_luminance_distance_and_alpha() {
        fn luminance(rgb: [f32; 3]) -> f32 {
            let [r, g, b] = rgb.map(|channel| {
                if channel <= 0.04045 {
                    channel / 12.92
                } else {
                    ((channel + 0.055) / 1.055).powf(2.4)
                }
            });
            0.2126 * r + 0.7152 * g + 0.0722 * b
        }

        for ground in [[0.06, 0.08, 0.10], [0.92, 0.95, 0.98], [1.0, 1.0, 1.0]] {
            let [r, g, b] = ground;
            let background = ColorF::new(r, g, b, 1.0);
            for seed in [[0.0; 3], [1.0; 3], [1.0, 0.0, 0.4]] {
                let palette = wash_palette(background, [seed; PALETTE_ENTRIES]);
                for (index, entry) in palette.0[..PALETTE_ENTRIES].iter().enumerate() {
                    let alpha = f32::from(entry.a) / 255.0;
                    if index == 8 {
                        assert_eq!([entry.r, entry.g, entry.b, entry.a], [0; 4]);
                        continue;
                    }
                    let ceiling = if index == 7 { 0.085 } else { 0.17 };
                    assert!(alpha > 0.0 && alpha <= ceiling);
                    let ink = [entry.r, entry.g, entry.b]
                        .map(|channel| f32::from(channel) / f32::from(entry.a));
                    assert!((luminance(ink) - luminance(ground)).abs() <= 0.183);
                    let composited =
                        std::array::from_fn(|i| ink[i] * alpha + ground[i] * (1.0 - alpha));
                    assert!((luminance(composited) - luminance(ground)).abs() <= 0.032);
                }
            }
        }
    }

    #[test]
    fn wash_follows_the_ground_camera_and_keeps_non_square_bounds() {
        let digest = ViewDigest::from_held([-82.0, 130.0].map(|x| BackdropNode {
            position: [x, 71.0],
            kind: BackdropKind::Other,
            community: None,
            site: None,
            visit_seconds: None,
        }))
        .unwrap();
        let camera = Camera {
            offset: (83.0, 47.0),
            zoom: 1.25,
            yaw: 0.51,
            tilt: 0.55,
        };
        let (transform, height) = wash_projection(&digest, camera);
        let [left, top, right, bottom] = digest.bounds().map(|cell| (cell * LATTICE) as f32);
        for (icon, world) in [
            (
                LayoutPoint::new(-32.0, -32.0),
                PortablePoint::new(left, top),
            ),
            (
                LayoutPoint::new(32.0, 32.0),
                PortablePoint::new(right, bottom),
            ),
        ] {
            let actual = transform.transform_point2d(icon).unwrap();
            let expected = camera.to_screen(world);
            assert!((actual.x - expected.0).abs() < 1e-4);
            assert!((actual.y - expected.1).abs() < 1e-4);
        }
        let a = camera.to_screen(PortablePoint::new(left, top));
        let b = camera.to_screen(PortablePoint::new(left, bottom));
        assert!((height - (a.0 - b.0).hypot(a.1 - b.1)).abs() < 1e-4);
    }

    fn paused_canvas() -> Canvas {
        let mut graph = Graph::new();
        let anchor = PortablePoint::new(120.0, 120.0);
        let node = graph.add_node("https://example.test/".into(), anchor);
        let mut canvas = Canvas::with_graph(graph);
        canvas.set_physics_paused(true);
        canvas.land_paused_positions(&[(node, anchor)]);
        canvas
    }

    #[test]
    fn frame_places_content_backdrop_between_ground_and_territory() {
        let mut canvas = paused_canvas();
        canvas.set_territory_field(true);

        let (off, _) = canvas.frame(320, 320);
        assert!(!off.ops.iter().any(|op| matches!(op, SceneOp::Gradient(_))));
        canvas.set_content_backdrop(true);
        let (on, _) = canvas.frame(320, 320);
        assert!(canvas.last_translation_error().is_none());
        let wash = on
            .ops
            .iter()
            .position(|op| matches!(op, SceneOp::Gradient(_)))
            .expect("the enabled canvas frame paints the wash");
        assert!(matches!(on.ops.first(), Some(SceneOp::Rect(rect)) if rect.color[3] == 1.0));
        let territory = on
            .ops
            .iter()
            .position(|op| matches!(op, SceneOp::Rect(rect) if rect.color[3] == TIER_ALPHA[2]))
            .expect("territory paints its top tier");
        assert!(0 < wash && wash < territory);
    }

    #[cfg(feature = "backdrop-cpu")]
    #[test]
    fn cpu_raster_shows_a_soft_wash_on_light_and_dark_grounds() {
        use netrender::vello_backends::{CpuSession, SparseResourceLimits};

        let mut canvas = paused_canvas();
        canvas.set_render_gnodes_as_dom(true);
        canvas.set_content_backdrop(true);
        let mut cpu = CpuSession::new(SparseResourceLimits::default());
        for (ground, ink) in [(0.1, 255), (0.9, 0)] {
            let edge = f32::from(ink) / 255.0;
            canvas.set_palette([ground, ground, ground, 1.0], [edge, edge, edge, 1.0]);
            canvas.set_category_colors([[ink; 3]; 5]);
            let (scene, _) = canvas.frame(320, 320);
            let pixmap = cpu.render(&scene).unwrap();
            let pixels = pixmap.data_as_u8_slice();
            // The held anchor falls in cell (1, 1). Its snapped centre is (96, 96).
            let sample = |x: usize, y: usize| -> u8 { pixels[(y * 320 + x) * 4] };
            let clear = sample(170, 96);
            let delta = |x| sample(x, 96).abs_diff(clear);
            assert!(delta(96) > delta(128));
            assert!(delta(128) > 0);
            assert!(delta(96) <= 5, "wash centre should stay close to ground");
            assert_eq!(clear, (ground * 255.0).round() as u8);
        }
    }
}
