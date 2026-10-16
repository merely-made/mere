// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The small surfaces a host implements to use the canvas's camera, command
//! and face-alignment helpers without being a [`Canvas`].
//!
//! [`Canvas`] implements all three, so existing callers only change their call
//! sites to nothing. Another host (a graph view that keeps its own camera)
//! implements them over its own state, usually through a newtype, since a
//! trait defined here cannot be implemented for a foreign camera type directly.
//! The traits are portable: they use plain numbers and no runtime.

use super::{CameraView, Canvas, MAX_ZOOM, MIN_ZOOM, WHEEL_PAN_SCALE, ZOOM_STEP};

/// A face rectangle or a body rectangle in screen px: left, top, right, bottom.
pub type ScreenRect = (f32, f32, f32, f32);

/// A top-down camera a host owns: `screen = world * zoom + offset`.
pub trait CameraHost {
    /// The current camera.
    fn camera(&self) -> CameraView;

    /// Place the camera. The host clamps the zoom to its own range.
    fn set_camera(&mut self, view: CameraView);

    /// The smallest and largest zoom the host allows.
    fn zoom_range(&self) -> (f32, f32);

    /// Stop (`false`) or resume (`true`) following a layout. A host that does
    /// not follow one ignores it.
    fn set_view_follow(&mut self, _follow: bool) {}

    /// Zoom about the screen point `at` by a wheel's signed distance in px:
    /// positive enlarges. The canvas's per-notch factor applies, so a host
    /// that takes this default zooms as the canvas does, and keeps the world
    /// point under `at` where it was.
    fn wheel_zoom_px(&mut self, at: (f32, f32), delta_px: f32) {
        let factor = ZOOM_STEP.powf(delta_px / WHEEL_PAN_SCALE);
        let before = self.camera();
        let (low, high) = self.zoom_range();
        let zoom = (before.zoom * factor).clamp(low, high);
        let scale = zoom / before.zoom;
        self.set_camera(CameraView {
            offset: (
                at.0 - (at.0 - before.offset.0) * scale,
                at.1 - (at.1 - before.offset.1) * scale,
            ),
            zoom,
        });
        self.set_view_follow(false);
    }

    /// Pan by one control or key step. The default moves the camera at once,
    /// by the distance the canvas's glide would cover; the canvas itself
    /// glides.
    fn pan_step(&mut self, dx: f32, dy: f32)
    where
        Self: Sized,
    {
        super::gestures::pan_at_once(self, dx, dy);
    }
}

/// The framing and physics verbs [`super::controls::CanvasCommand`] can ask
/// for. Each defaults to doing nothing, so a host offers the ones it has.
pub trait FitHost: CameraHost {
    fn fit_to_content(&mut self) {}
    fn fit_visible(&mut self) {}
    fn fit_selection(&mut self) {}
    fn restore_arrangement(&mut self) {}
    fn physics_paused(&self) -> bool {
        false
    }
    fn set_physics_paused(&mut self, _paused: bool) {}
    /// The graph grammar's seed and attention verbs (`attention.rs`).
    fn reset_to_seed(&mut self) {}
    fn set_seed(&mut self) {}
    fn restore_recipe_seed(&mut self) {}
    fn dismiss_all_retained(&mut self) {}
    fn toggle_selection_foreground_pin(&mut self) {}
    fn resize_selected_tiles(&mut self, _factor: f32) {}
    fn toggle_territory_field(&mut self) {}
    fn toggle_suggestions(&mut self) {}
}

/// What [`super::face_alignment::FaceAlignment`] measures: each node's face
/// rectangle with its body rectangle when the body was found.
pub trait FaceGeometry {
    /// The camera zoom the rectangles are drawn at.
    fn zoom(&self) -> f32;

    /// Every node that has a face: `(face, body)`.
    fn faces(&self) -> Vec<(ScreenRect, Option<ScreenRect>)>;
}

impl CameraHost for Canvas {
    fn camera(&self) -> CameraView {
        Canvas::camera(self)
    }

    fn set_camera(&mut self, view: CameraView) {
        Canvas::set_camera(self, view);
    }

    fn zoom_range(&self) -> (f32, f32) {
        (MIN_ZOOM, MAX_ZOOM)
    }

    fn set_view_follow(&mut self, follow: bool) {
        Canvas::set_view_follow(self, follow);
    }

    // The canvas zooms through its own wheel path (projection-aware), so its
    // behavior does not depend on the default's arithmetic.
    fn wheel_zoom_px(&mut self, at: (f32, f32), delta_px: f32) {
        self.cursor_moved(at.0, at.1);
        self.set_ctrl(true);
        self.wheel(0.0, delta_px);
        self.set_ctrl(false);
    }

    // A pan impulse into the canvas's inertia.
    fn pan_step(&mut self, dx: f32, dy: f32) {
        self.wheel(dx, dy);
    }
}

impl FitHost for Canvas {
    fn fit_to_content(&mut self) {
        Canvas::fit_to_content(self);
    }

    fn fit_visible(&mut self) {
        Canvas::fit_visible(self);
    }

    fn fit_selection(&mut self) {
        Canvas::fit_selection(self);
    }

    fn restore_arrangement(&mut self) {
        Canvas::restore_arrangement(self);
    }

    fn physics_paused(&self) -> bool {
        Canvas::physics_paused(self)
    }

    fn set_physics_paused(&mut self, paused: bool) {
        Canvas::set_physics_paused(self, paused);
    }

    fn reset_to_seed(&mut self) {
        Canvas::reset_to_seed(self);
    }

    fn set_seed(&mut self) {
        Canvas::set_seed(self);
    }

    fn restore_recipe_seed(&mut self) {
        Canvas::restore_recipe_seed(self);
    }

    fn dismiss_all_retained(&mut self) {
        Canvas::dismiss_all_retained(self);
    }

    fn toggle_selection_foreground_pin(&mut self) {
        Canvas::toggle_selection_foreground_pin(self);
    }

    fn resize_selected_tiles(&mut self, factor: f32) {
        Canvas::resize_selected_tiles(self, factor);
    }

    fn toggle_territory_field(&mut self) {
        let on = Canvas::territory_field(self);
        Canvas::set_territory_field(self, !on);
    }

    fn toggle_suggestions(&mut self) {
        let mut settings = Canvas::suggestion_settings(self).clone();
        settings.on = !settings.on;
        Canvas::set_suggestion_settings(self, settings);
    }
}

impl FaceGeometry for Canvas {
    fn zoom(&self) -> f32 {
        self.camera().zoom
    }

    fn faces(&self) -> Vec<(ScreenRect, Option<ScreenRect>)> {
        self.graph()
            .nodes()
            .filter_map(|(key, _)| {
                let face = self.node_face_rect(key)?;
                Some((face, self.node_body_rect(key)))
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::controls::CanvasCommand;
    use crate::canvas::face_alignment::FaceAlignment;
    use crate::canvas::gestures::{PAN_GLIDE, two_finger, wheel_zoom};

    /// A host that is not a canvas: its own camera and a log of the verbs it
    /// was asked for.
    #[derive(Default)]
    struct Plain {
        view: Option<CameraView>,
        following: Option<bool>,
        paused: bool,
        log: Vec<&'static str>,
        faces: Vec<(ScreenRect, Option<ScreenRect>)>,
    }

    impl Plain {
        fn at(view: CameraView) -> Self {
            Self {
                view: Some(view),
                ..Self::default()
            }
        }
        fn view(&self) -> CameraView {
            self.view.unwrap_or(CameraView {
                offset: (0.0, 0.0),
                zoom: 1.0,
            })
        }
    }

    impl CameraHost for Plain {
        fn camera(&self) -> CameraView {
            self.view()
        }
        fn set_camera(&mut self, view: CameraView) {
            self.view = Some(CameraView {
                zoom: view.zoom.clamp(MIN_ZOOM, MAX_ZOOM),
                ..view
            });
        }
        fn zoom_range(&self) -> (f32, f32) {
            (MIN_ZOOM, MAX_ZOOM)
        }
        fn set_view_follow(&mut self, follow: bool) {
            self.following = Some(follow);
        }
    }

    impl FitHost for Plain {
        fn fit_to_content(&mut self) {
            self.log.push("fit");
        }
        fn fit_selection(&mut self) {
            self.log.push("fit-selection");
        }
        fn physics_paused(&self) -> bool {
            self.paused
        }
        fn set_physics_paused(&mut self, paused: bool) {
            self.paused = paused;
        }
    }

    impl FaceGeometry for Plain {
        fn zoom(&self) -> f32 {
            self.view().zoom
        }
        fn faces(&self) -> Vec<(ScreenRect, Option<ScreenRect>)> {
            self.faces.clone()
        }
    }

    fn close(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn a_plain_host_zooms_as_the_canvas_does() {
        let start = CameraView {
            offset: (130.0, -40.0),
            zoom: 1.3,
        };
        let mut plain = Plain::at(start);
        let mut canvas = Canvas::new();
        canvas.set_camera(start);
        let at = (200.0, 150.0);
        for dy in [-40.0, -40.0, 80.0, -400.0, 4000.0] {
            wheel_zoom(&mut plain, at, dy);
            wheel_zoom(&mut canvas, at, dy);
            assert!(
                (plain.camera().zoom - canvas.camera().zoom).abs() < 1e-3,
                "{dy}"
            );
            assert!(close(plain.camera().offset, canvas.camera().offset), "{dy}");
        }
        assert_eq!(plain.following, Some(false));
    }

    #[test]
    fn a_plain_hosts_zoom_stays_in_its_range_and_keeps_the_point() {
        let mut plain = Plain::at(CameraView {
            offset: (10.0, 20.0),
            zoom: 1.0,
        });
        let at = (300.0, 200.0);
        let world = |h: &Plain| {
            let v = h.camera();
            ((at.0 - v.offset.0) / v.zoom, (at.1 - v.offset.1) / v.zoom)
        };
        let before = world(&plain);
        wheel_zoom(&mut plain, at, -40.0);
        assert!(plain.camera().zoom > 1.0);
        assert!(close(world(&plain), before));
        for _ in 0..40 {
            wheel_zoom(&mut plain, at, -4000.0);
        }
        assert_eq!(plain.camera().zoom, MAX_ZOOM);
        assert!(close(world(&plain), before));
    }

    #[test]
    fn two_fingers_move_a_plain_host_as_they_move_the_canvas() {
        let start = CameraView {
            offset: (130.0, -40.0),
            zoom: 1.0,
        };
        let mut plain = Plain::at(start);
        let mut canvas = Canvas::new();
        canvas.set_camera(start);
        let from = [(300.0, 300.0), (400.0, 300.0)];
        let to = [(280.0, 300.0), (480.0, 300.0)];
        two_finger(&mut plain, from, to);
        two_finger(&mut canvas, from, to);
        assert!((plain.camera().zoom - canvas.camera().zoom).abs() < 1e-4);
        assert!(close(plain.camera().offset, canvas.camera().offset));
        assert_eq!(plain.following, Some(false));
    }

    #[test]
    fn commands_reach_a_plain_host_through_its_verbs() {
        let mut plain = Plain::at(CameraView {
            offset: (130.0, -40.0),
            zoom: 1.3,
        });
        CanvasCommand::Fit.apply(&mut plain, (800, 600));
        CanvasCommand::FitSelection.apply(&mut plain, (800, 600));
        CanvasCommand::FitVisible.apply(&mut plain, (800, 600));
        assert_eq!(
            plain.log,
            ["fit", "fit-selection"],
            "an absent verb does nothing"
        );
        assert_eq!(plain.following, Some(true), "fitting resumes following");
        CanvasCommand::TogglePhysics.apply(&mut plain, (800, 600));
        assert!(plain.paused);
        CanvasCommand::SetPhysicsPaused(false).apply(&mut plain, (800, 600));
        assert!(!plain.paused);

        let centre_world = |h: &Plain| {
            let v = h.camera();
            ((400.0 - v.offset.0) / v.zoom, (300.0 - v.offset.1) / v.zoom)
        };
        let before = centre_world(&plain);
        CanvasCommand::SetZoom { zoom: 0.5 }.apply(&mut plain, (800, 600));
        assert_eq!(plain.camera().zoom, 0.5);
        assert!(close(centre_world(&plain), before));
        CanvasCommand::SetZoom { zoom: 0.01 }.apply(&mut plain, (800, 600));
        assert_eq!(plain.camera().zoom, MIN_ZOOM, "the host's range clamps it");
        assert!(close(centre_world(&plain), before));

        let offset = plain.camera().offset;
        CanvasCommand::Pan { dx: -42.0, dy: 0.0 }.apply(&mut plain, (800, 600));
        assert!(close(
            plain.camera().offset,
            (offset.0 - 42.0 * PAN_GLIDE, offset.1)
        ));
    }

    #[test]
    fn face_alignment_reads_a_plain_hosts_rectangles() {
        let mut plain = Plain::at(CameraView {
            offset: (0.0, 0.0),
            zoom: 2.0,
        });
        let body = (100.0, 100.0, 150.0, 150.0);
        let face = (103.5, 103.5, 139.5, 139.5);
        plain.faces = vec![
            (face, Some(body)),
            ((10.0, 10.0, 20.0, 20.0), None),
            ((-5.0, 10.0, 20.0, 20.0), Some(body)),
            ((700.0, 10.0, 900.0, 20.0), Some(body)),
        ];
        let alignment = FaceAlignment::measure(&plain, (800, 600));
        assert_eq!(alignment.zoom, 2.0);
        assert_eq!(alignment.nodes, 1, "off-screen faces are not measured");
        assert_eq!(alignment.missing, 1);
        assert!((alignment.ratio_min - 0.72).abs() < 1e-4);
        assert!((alignment.offset_max - 3.5 * 2f32.sqrt()).abs() < 1e-3);
        assert_eq!(
            FaceAlignment::measure(&Plain::default(), (800, 600)).nodes,
            0
        );
    }
}
