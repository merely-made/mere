// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit, one-shot camera framing of the current visible placement.

use super::*;

impl Canvas {
    /// Frame the currently visible nodes and any fold summary, without applying
    /// an arrangement or moving a node. Preserves the camera's yaw and tilt;
    /// stops following and pan inertia when a finite placement can be framed.
    /// An empty view leaves the viewport unchanged and returns `false`.
    /// Like `fit_to_content`, pads by 160 projected units and never magnifies
    /// above natural size. The usual zoom limits still apply to very large views.
    pub fn fit_visible(&mut self) -> bool {
        let mut points = self.framing_points(false);
        if let Some(center) = self
            .active_fold_projection()
            .and_then(|fold| fold.summary_center(|key| self.world_position_of(key)))
        {
            points.push(center);
        }
        self.frame_points(points)
    }

    /// Frame all selected, individually visible nodes. Hidden members, selected
    /// edges, and nodes without finite placement do not supply framing bounds.
    /// With no such node, leaves the viewport unchanged and returns `false`.
    /// This is a one-shot camera action; it does not change selection or scope.
    pub fn fit_selection(&mut self) -> bool {
        self.frame_points(self.framing_points(true))
    }

    /// Whether `fit_selection` has a finite, individually visible node to frame.
    /// Hosts use this to disable the action when selection is empty or hidden.
    pub fn can_fit_selection(&self) -> bool {
        self.selected.iter().any(|&key| {
            self.node_in_scope(key)
                && self
                    .world_position_of(key)
                    .is_some_and(|p| p.x.is_finite() && p.y.is_finite())
        })
    }

    fn framing_points(&self, selected_only: bool) -> Vec<PortablePoint> {
        // Use the same scope/fold visibility as node_in_scope, but project the
        // fold once for this traversal instead of once per node.
        let fold = self.active_fold_projection();
        let scope = self
            .scope
            .as_ref()
            .map(|keys| keys.iter().copied().collect::<HashSet<_>>());
        self.graph
            .nodes()
            .filter_map(|(key, _)| {
                (scope.as_ref().is_none_or(|keys| keys.contains(&key))
                    && fold.as_ref().is_none_or(|f| !f.members.contains(&key))
                    && (!selected_only || self.selected.contains(&key)))
                .then(|| self.world_position_of(key))
                .flatten()
            })
            .collect()
    }

    fn frame_points(&mut self, points: Vec<PortablePoint>) -> bool {
        let projection = scene_paint::Camera {
            offset: (0.0, 0.0),
            zoom: 1.0,
            ..self.camera
        };
        let mut min = (f32::INFINITY, f32::INFINITY);
        let mut max = (f32::NEG_INFINITY, f32::NEG_INFINITY);
        let mut any = false;
        for point in points {
            let p = projection.to_screen(point);
            if !p.0.is_finite() || !p.1.is_finite() {
                continue;
            }
            any = true;
            min = (min.0.min(p.0), min.1.min(p.1));
            max = (max.0.max(p.0), max.1.max(p.1));
        }
        if !any {
            return false;
        }
        const MARGIN: f32 = 160.0;
        let (w, h) = (self.view_w as f32, self.view_h as f32);
        let zoom = (w / (max.0 - min.0 + 2.0 * MARGIN))
            .min(h / (max.1 - min.1 + 2.0 * MARGIN))
            .min(1.0)
            .clamp(MIN_ZOOM, MAX_ZOOM);
        let center = (min.0 * 0.5 + max.0 * 0.5, min.1 * 0.5 + max.1 * 0.5);
        self.set_camera(CameraView {
            offset: (w * 0.5 - center.0 * zoom, h * 0.5 - center.1 * zoom),
            zoom,
        });
        self.pan_velocity = (0.0, 0.0);
        true
    }
}
