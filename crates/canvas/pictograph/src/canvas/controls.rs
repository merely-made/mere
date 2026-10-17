// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Canvas operations a host's controls and keyboard bindings share.
//!
//! Controls and keyboard bindings choose their own distances. The host owns
//! camera and physics state (the canvas, or any [`FitHost`]); this seam does
//! not read a DOM or duplicate that state.

use super::CameraView;
use super::camera_host::FitHost;

/// A presentation command that does not change graph authority.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CanvasCommand {
    Pan {
        dx: f32,
        dy: f32,
    },
    /// Zoom about the centre of the canvas's logical viewport.
    Zoom {
        delta: f32,
    },
    /// Set the zoom exactly, about the same centre: what a receipt uses to
    /// visit chosen zooms. The canvas clamps it to its range.
    SetZoom {
        zoom: f32,
    },
    Fit,
    /// One-shot framing, without enabling layout following or restoring placement.
    FitVisible,
    FitSelection,
    RestoreArrangement,
    SetPhysicsPaused(bool),
    TogglePhysics,
    /// Every unheld node eases back to its seed (graph grammar).
    ResetToSeed,
    /// The current arrangement becomes the seed.
    SetSeed,
    RestoreRecipeSeed,
    DismissRetained,
    /// Pin the selection forward, or release it if all of it is pinned.
    ToggleForegroundPin,
    /// Grow or shrink the selected Mosaic tiles by a quarter.
    GrowTiles,
    ShrinkTiles,
    /// Draw the territory field beneath the graph, or stop.
    ToggleTerritory,
    /// Turn suggestions on or off (off by default).
    ToggleSuggestions,
}

impl CanvasCommand {
    /// Carry the command out on any host that offers the verbs it needs; the
    /// canvas is one. `viewport` is the host's logical size in px.
    pub fn apply<H: FitHost>(self, host: &mut H, viewport: (u32, u32)) {
        let centre = (viewport.0 as f32 * 0.5, viewport.1 as f32 * 0.5);
        match self {
            Self::Pan { dx, dy } => host.pan_step(dx, dy),
            Self::Zoom { delta } => host.wheel_zoom_px(centre, delta),
            Self::SetZoom { zoom } => {
                // The camera is top-down: screen = world * zoom + offset.
                let before = host.camera();
                let (low, high) = host.zoom_range();
                host.set_camera(CameraView {
                    zoom: zoom.clamp(low, high),
                    ..before
                });
                let zoom = host.camera().zoom;
                let scale = zoom / before.zoom;
                host.set_camera(CameraView {
                    offset: (
                        centre.0 - (centre.0 - before.offset.0) * scale,
                        centre.1 - (centre.1 - before.offset.1) * scale,
                    ),
                    zoom,
                });
            },
            // Fitting resumes following the layout; a pan or zoom (both reach
            // the canvas's wheel) or a node drag stops it.
            Self::Fit => {
                host.fit_to_content();
                host.set_view_follow(true);
            },
            Self::FitVisible => host.fit_visible(),
            Self::FitSelection => host.fit_selection(),
            Self::RestoreArrangement => host.restore_arrangement(),
            Self::SetPhysicsPaused(paused) => host.set_physics_paused(paused),
            // An arrangement can pause the canvas independently of the UI.
            Self::TogglePhysics => {
                let paused = host.physics_paused();
                host.set_physics_paused(!paused);
            },
            Self::ResetToSeed => host.reset_to_seed(),
            Self::SetSeed => host.set_seed(),
            Self::RestoreRecipeSeed => host.restore_recipe_seed(),
            Self::DismissRetained => host.dismiss_all_retained(),
            Self::ToggleForegroundPin => host.toggle_selection_foreground_pin(),
            Self::GrowTiles => host.resize_selected_tiles(1.25),
            Self::ShrinkTiles => host.resize_selected_tiles(0.8),
            Self::ToggleTerritory => host.toggle_territory_field(),
            Self::ToggleSuggestions => host.toggle_suggestions(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::Canvas;

    #[test]
    fn set_zoom_lands_exactly_and_keeps_the_centre_world_point() {
        let mut canvas = Canvas::new();
        canvas.set_camera(CameraView {
            offset: (130.0, -40.0),
            zoom: 1.3,
        });
        let viewport = (800, 600);
        let world = |c: &Canvas| {
            let v = c.camera();
            ((400.0 - v.offset.0) / v.zoom, (300.0 - v.offset.1) / v.zoom)
        };
        let before = world(&canvas);
        for zoom in [1.0, 0.75, 0.5, 0.25, 0.1, 2.0] {
            CanvasCommand::SetZoom { zoom }.apply(&mut canvas, viewport);
            assert_eq!(canvas.camera().zoom, zoom);
            let after = world(&canvas);
            assert!((after.0 - before.0).abs() < 1e-3 && (after.1 - before.1).abs() < 1e-3);
        }
        CanvasCommand::SetZoom { zoom: 0.01 }.apply(&mut canvas, viewport);
        assert_eq!(canvas.camera().zoom, 0.1, "the canvas's range clamps it");
    }
}
