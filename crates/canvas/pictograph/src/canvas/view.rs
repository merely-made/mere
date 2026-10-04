// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Camera / viewport / orbit control and scope (curated-subset) lensing.

use super::*;

impl Canvas {
    /// The session graph, for the host to persist (`to_snapshot` → `graph.json`).
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    /// The graph's single live atomic-facet store. Hosts persist this beside
    /// `graph.json` as `facets.json`.
    pub fn facets(&self) -> &kernel::graph::NodeFacetStore {
        self.graph.facets()
    }

    /// Mutable access for host-defined and unknown-forward facet namespaces.
    pub fn facets_mut(&mut self) -> &mut kernel::graph::NodeFacetStore {
        self.graph.facets_mut()
    }

    /// Overlay the canonical sidecar after the graph imported any legacy
    /// snapshot columns.
    pub fn overlay_facets(&mut self, facets: kernel::graph::NodeFacetStore) {
        self.graph.overlay_facets(facets);
    }

    /// The current camera (pan + zoom), for the host to persist as view-intent.
    pub fn camera(&self) -> CameraView {
        CameraView {
            offset: self.camera.offset,
            zoom: self.camera.zoom,
        }
    }

    /// Restore the camera from persisted view-intent. A non-finite or
    /// non-positive zoom falls back to `1.0`; the zoom is clamped to the canvas's
    /// range. The host suppresses its own first-frame recenter when it restores a
    /// camera, so this value is not immediately overwritten.
    pub fn set_camera(&mut self, view: CameraView) {
        // An explicit placement is a pan or zoom: the camera stops following.
        self.follow = false;
        self.camera.zoom = if view.zoom.is_finite() && view.zoom > 0.0 {
            view.zoom.clamp(MIN_ZOOM, MAX_ZOOM)
        } else {
            1.0
        };
        self.camera.offset = view.offset;
    }

    /// The full per-pane [`Viewport`] (camera + pan inertia), for the host to stash on
    /// the owning (window, graph) pane and reinstall before the next render / input
    /// pass via [`set_viewport`](Self::set_viewport). Carries yaw / tilt too (unlike
    /// [`camera`](Self::camera)), so an isometric pane round-trips losslessly. This is
    /// the seam that moves the camera off the shared authority onto the view: the
    /// canvas's own `camera` / `pan_velocity` become per-pass working state the host
    /// drives, so two windows on one graph hold distinct viewports, not a mirror.
    pub fn viewport(&self) -> Viewport {
        Viewport {
            offset: self.camera.offset,
            zoom: self.camera.zoom,
            yaw: self.camera.yaw,
            tilt: self.camera.tilt,
            pan_velocity: self.pan_velocity,
            view: (self.view_w, self.view_h),
        }
    }

    /// Install a per-pane [`Viewport`] before rendering or handling input for the pane
    /// that owns it. Clamps as [`set_camera`](Self::set_camera) (zoom) and
    /// [`set_tilt`](Self::set_tilt) (tilt) do, so a host-stashed value is sanitized on
    /// the way back in.
    pub fn set_viewport(&mut self, v: Viewport) {
        self.camera.offset = v.offset;
        self.camera.zoom = if v.zoom.is_finite() && v.zoom > 0.0 {
            v.zoom.clamp(MIN_ZOOM, MAX_ZOOM)
        } else {
            1.0
        };
        self.camera.yaw = v.yaw;
        self.camera.tilt = v.tilt.clamp(0.05, 1.0);
        self.pan_velocity = v.pan_velocity;
        // The size the camera was framed for, installed WITH it and without
        // re-centring: this is a swap between views, not a resize of one. Use
        // `resize` when a window actually changed size and the centre should
        // hold; use this to install a pane's stashed viewport.
        self.view_w = v.view.0.max(1);
        self.view_h = v.view.1.max(1);
    }

    /// Toggle the isometric (foreshortened-ground) projection. `on` reclines the
    /// ground by foreshortening the vertical, so the graph reads as a tilted floor
    /// while the gnodes stay upright billboards; `off` restores the plain
    /// top-down view. The free-cam yaw orbit + persistence are P2. (Isometric camera P1.)
    pub fn set_isometric(&mut self, on: bool) {
        /// Vertical foreshorten for the isometric preset (a dimetric squash); becomes
        /// a setting once the projection picker lands (P2).
        const ISO_TILT: f32 = 0.55;
        self.camera.tilt = if on { ISO_TILT } else { 1.0 };
    }

    /// Whether the isometric projection is active (the ground is foreshortened).
    pub fn is_isometric(&self) -> bool {
        self.camera.tilt < 1.0
    }

    /// Orbit the view by `d_radians` about the vertical (the free-cam yaw). The
    /// ground rotates while the gnodes stay upright billboards; pair with the
    /// isometric `tilt` for the 2.5D orbit (at `tilt == 1` it spins the flat layout).
    /// The host's orbit gesture / projection picker drives this. (Isometric camera P2.)
    pub fn orbit_by(&mut self, d_radians: f32) {
        self.camera.yaw += d_radians;
    }

    /// Set the orbit yaw (radians) directly. (Isometric camera P2.)
    pub fn set_yaw(&mut self, yaw: f32) {
        self.camera.yaw = yaw;
    }

    /// Set the vertical foreshorten directly, clamped to a sane `(0, 1]`. `1.0` is
    /// top-down; lower reclines the ground further. (Isometric camera P2.)
    pub fn set_tilt(&mut self, tilt: f32) {
        self.camera.tilt = tilt.clamp(0.05, 1.0);
    }

    /// The current orbit yaw (radians), for a host control to display / persist.
    pub fn yaw(&self) -> f32 {
        self.camera.yaw
    }

    /// The current vertical foreshorten (`tilt`), for a host control to display / persist.
    pub fn tilt(&self) -> f32 {
        self.camera.tilt
    }

    /// Whether a scope lens is active (the canvas is showing a curated subset, not the
    /// whole graph). The host offers "Show all" when this is true. (Curated canvas.)
    pub fn is_scoped(&self) -> bool {
        self.scope.is_some()
    }

    /// Focus the canvas on the current selection: scope it to the selected nodes plus
    /// their immediate (undirected) neighbors, so the selection shows as its own
    /// neighborhood projected through a curated arrangement. A no-op with no
    /// selection. (Curated canvas.)
    pub fn isolate_selection(&mut self) {
        if self.selected.is_empty() {
            return;
        }
        let mut scope: HashSet<NodeKey> = self.selected.clone();
        for &key in &self.selected {
            scope.extend(self.graph.neighbors_undirected(key));
        }
        self.scope = Some(scope.into_iter().collect());
    }

    /// Scope the canvas to a host-supplied member set (by UUID), e.g. the workbench's
    /// open tiles, so the *same* arrangement renders as both a tiled workbench and a
    /// spatial map (the spine's "two projections of one arrangement"). Members absent
    /// from the graph are skipped; an empty set clears the lens (shows the whole
    /// graph). (Curated canvas — workbench mirror.)
    pub fn scope_to_members(&mut self, members: impl IntoIterator<Item = uuid::Uuid>) {
        let keys: Vec<NodeKey> = members
            .into_iter()
            .filter_map(|id| self.graph.get_node_by_id(id).map(|(key, _)| key))
            .collect();
        self.scope = (!keys.is_empty()).then_some(keys);
    }

    /// Drop the scope lens — show the whole graph again. (Curated canvas.)
    pub fn clear_scope(&mut self) {
        self.scope = None;
    }

    /// The current scope lens as member uuids, or `None` when unscoped. The inverse of
    /// [`scope_to_members`](Self::scope_to_members) — a host saves this before a transient
    /// per-window scope override (a branch window scoping to its subgraph) and restores it
    /// after. (Per-window branch scope.)
    pub fn scope_members(&self) -> Option<Vec<uuid::Uuid>> {
        self.scope.as_ref().map(|keys| {
            keys.iter()
                .filter_map(|&k| self.graph.get_node(k).map(|n| n.id))
                .collect()
        })
    }

    /// Whether `key` is individually visible in the current Canvas view. The
    /// scope lens is applied first; an active fold then substitutes its source
    /// members with a synthetic summary body. The host's gnode builder uses this
    /// too, so its DOM path matches the scene paint path. (Graph curation C3.)
    pub fn node_in_scope(&self, key: NodeKey) -> bool {
        self.scope.as_ref().is_none_or(|s| s.contains(&key))
            && self
                .active_fold_projection()
                .is_none_or(|projection| !projection.members.contains(&key))
    }

    /// Zoom by `factor`, keeping the world point under `anchor` (screen px) fixed.
    pub(crate) fn zoom_at(&mut self, anchor: (f32, f32), factor: f32) {
        // Keep the world point currently under `anchor` fixed across the zoom: read it
        // before, then shift `offset` so it lands back under `anchor` after. Correct for
        // any projection (top-down or isometric), and identical to the old
        // `world*zoom+offset` formula at the default camera. (Isometric camera P0.)
        let world_under = self.camera.to_world(anchor);
        self.camera.zoom = (self.camera.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        let landed = self.camera.to_screen(world_under);
        self.camera.offset.0 += anchor.0 - landed.0;
        self.camera.offset.1 += anchor.1 - landed.1;
    }

    /// Where `node` sits in screen px — the space pointer input arrives in.
    /// `None` when the node has no position. The inverse of
    /// [`screen_to_world`](Self::screen_to_world).
    pub fn screen_position_of(&self, node: NodeKey) -> Option<(f32, f32)> {
        let world = self.view.position_of(node)?;
        Some(
            self.camera
                .to_screen(kernel::geometry::PortablePoint::new(world.x, world.y)),
        )
    }

    /// Where the single focused node sits in screen px, if exactly one is
    /// focused. What a host reports so a receipt can aim at it.
    pub fn focused_screen_position(&self) -> Option<(f32, f32)> {
        self.screen_position_of(self.focused_key()?)
    }

    /// Where the single focused node sits in world units, so a receipt can
    /// measure a law's motion whatever the zoom.
    pub fn focused_world_position(&self) -> Option<(f32, f32)> {
        let p = self.view.position_of(self.focused_key()?)?;
        Some((p.x, p.y))
    }

    /// The world point under screen px `screen`, through the camera.
    pub fn world_point_at(&self, screen: (f32, f32)) -> (f32, f32) {
        let p = self.camera.to_world(screen);
        (p.x, p.y)
    }

    /// Where world point `world` falls in screen px, through the camera.
    pub fn screen_point_of(&self, world: (f32, f32)) -> (f32, f32) {
        self.camera
            .to_screen(kernel::geometry::PortablePoint::new(world.0, world.1))
    }

    /// Map a screen-px point back to world space through the camera projector
    /// (the inverse of `Camera::to_screen`; at the default camera this is
    /// `world = (screen - offset) / zoom`).
    pub(crate) fn screen_to_world(&self, screen: (f32, f32)) -> Point2D<f32> {
        let w = self.camera.to_world(screen);
        Point2D::new(w.x, w.y)
    }

    /// The screen viewport mapped to world space — the region seiche culls against
    /// to decide which nodes are on screen.
    pub(crate) fn world_viewport(&self) -> Box2D<f32> {
        // Bound all four screen corners in world space: under an isometric yaw the
        // screen rectangle maps to a rotated world quad, so two corners under-cover.
        // At the default top-down camera this is the same box as before. (Isometric P0.)
        let (w, h) = (self.view_w as f32, self.view_h as f32);
        let corners = [
            self.screen_to_world((0.0, 0.0)),
            self.screen_to_world((w, 0.0)),
            self.screen_to_world((0.0, h)),
            self.screen_to_world((w, h)),
        ];
        let mut min = corners[0];
        let mut max = corners[0];
        for p in &corners[1..] {
            min.x = min.x.min(p.x);
            min.y = min.y.min(p.y);
            max.x = max.x.max(p.x);
            max.y = max.y.max(p.y);
        }
        Box2D::new(min, max)
    }

    /// Follow the layout: while physics plays, the camera eases toward
    /// fit-to-content each frame. Any pan, zoom or node drag turns it off; the
    /// host turns it on (Graphshell: a law, profile or Free switch, and Fit
    /// graph).
    pub fn set_view_follow(&mut self, on: bool) {
        self.follow = on;
    }

    /// Whether the camera is following the layout.
    pub fn view_follows(&self) -> bool {
        self.follow
    }

    /// One frame of following over `dt` seconds: zoom geometrically and the
    /// world point at the viewport centre linearly toward the fit. Holds while
    /// paused and while a node or the camera is being dragged. Whether the
    /// camera moved.
    pub(crate) fn follow_step(&mut self, dt: f32) -> bool {
        if !self.follow
            || self.physics_paused
            || self.drag.is_some()
            || self.middle_drag.is_some()
            || self.orbit_drag.is_some()
        {
            return false;
        }
        let Some(fit) = self.content_fit() else {
            return false;
        };
        let k = 1.0 - (-dt.max(0.0) / FOLLOW_EASE_SECONDS).exp();
        let (w, h) = (self.view_w as f32 / 2.0, self.view_h as f32 / 2.0);
        let (z, o) = (self.camera.zoom, self.camera.offset);
        let centre = ((w - o.0) / z, (h - o.1) / z);
        let goal = ((w - fit.offset.0) / fit.zoom, (h - fit.offset.1) / fit.zoom);
        let zoom = (z * (fit.zoom / z).powf(k)).clamp(MIN_ZOOM, MAX_ZOOM);
        let centre = (
            centre.0 + (goal.0 - centre.0) * k,
            centre.1 + (goal.1 - centre.1) * k,
        );
        let offset = (w - centre.0 * zoom, h - centre.1 * zoom);
        let moved = (offset.0 - o.0).abs() > 0.01
            || (offset.1 - o.1).abs() > 0.01
            || (zoom - z).abs() > z * 1e-4;
        self.camera.zoom = zoom;
        self.camera.offset = offset;
        moved
    }

    /// Whether every node's centre is on screen: each position through the
    /// camera against the viewport inset by `margin` px, and the layout's
    /// world extent beside the viewport's.
    pub fn layout_framing(&self, margin: f32) -> LayoutFraming {
        let (w, h) = (self.view_w as f32, self.view_h as f32);
        let view = self.world_viewport();
        let mut framing = LayoutFraming {
            view: [view.min.x, view.min.y, view.max.x, view.max.y],
            extent: [f32::INFINITY, f32::INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY],
            ..LayoutFraming::default()
        };
        for (_, p) in self.view.positions() {
            framing.nodes += 1;
            if !p.x.is_finite() || !p.y.is_finite() {
                framing.outside += 1;
                continue;
            }
            let e = &mut framing.extent;
            *e = [e[0].min(p.x), e[1].min(p.y), e[2].max(p.x), e[3].max(p.y)];
            let (x, y) = self
                .camera
                .to_screen(kernel::geometry::PortablePoint::new(p.x, p.y));
            if x < margin || y < margin || x > w - margin || y > h - margin {
                framing.outside += 1;
            }
        }
        if framing.extent[0] > framing.extent[2] {
            // No finite position: an empty extent, not an inverted one.
            framing.extent = [0.0; 4];
        }
        framing
    }
}

#[cfg(test)]
mod tests {
    use crate::canvas::{Canvas, PointerButton};

    /// Swapping between two panes' viewports must move NEITHER camera, at any
    /// size. The camera and the size it was framed for install together, so a
    /// host never reaches for `resize` (which re-centres) to set the size of a
    /// camera it is only borrowing.
    ///
    /// The regression: turnstone's lens installed a camera and then resized to
    /// the lens rect, so the re-centring shift landed on the borrowed camera,
    /// was read back, and was stored. Both windows' graphs walked off-screen a
    /// little per frame.
    #[test]
    fn swapping_viewports_between_sizes_moves_neither_camera() {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        let primary = canvas.viewport();

        // A second view at a different size, framed its own way.
        canvas.resize(400, 900);
        canvas.wheel(37.0, -11.0);
        let lens = canvas.viewport();
        assert_ne!(primary.view, lens.view, "the two views differ in size");

        // Swap back and forth the way a two-window host does, many times.
        for _ in 0..64 {
            canvas.set_viewport(primary);
            assert_eq!(
                canvas.viewport(),
                primary,
                "the primary is installed exactly"
            );
            canvas.set_viewport(lens);
            assert_eq!(canvas.viewport(), lens, "the lens is installed exactly");
        }

        // Neither drifted, and each still carries its own size.
        canvas.set_viewport(primary);
        assert_eq!(canvas.viewport(), primary);
        assert_eq!(primary.view, (800, 600));
        assert_eq!(lens.view, (400, 900));
    }

    /// The framing instrument: a fitted graph has every centre on screen, and
    /// a camera panned one viewport-width past it (the planted off-screen
    /// case) has none; a margin wider than half the view counts every node.
    #[test]
    fn layout_framing_counts_centres_off_the_viewport() {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        canvas.fit_to_content();
        let fitted = canvas.layout_framing(0.0);
        assert!(fitted.nodes > 0, "the sample graph has nodes");
        assert_eq!(fitted.outside, 0, "fitted: {fitted:?}");
        let [x0, y0, x1, y1] = fitted.extent;
        let [v0, w0, v1, w1] = fitted.view;
        assert!(x0 >= v0 && y0 >= w0 && x1 <= v1 && y1 <= w1, "{fitted:?}");

        let mut camera = canvas.camera();
        camera.offset.0 += 800.0 + (x1 - x0) * camera.zoom;
        canvas.set_camera(camera);
        let planted = canvas.layout_framing(0.0);
        assert_eq!(planted.outside, planted.nodes, "panned away: {planted:?}");
        assert_eq!(planted.extent, fitted.extent, "the layout did not move");

        canvas.fit_to_content();
        assert_eq!(canvas.layout_framing(301.0).outside, fitted.nodes);
    }

    /// Following the layout ("Follow while playing", 2026-10-03). The control:
    /// a camera planted off the graph stays off it while not following. Then
    /// following eases it back until every centre is on screen, a wheel pan
    /// stops it, a node drag stops it while a click does not, and a paused
    /// canvas holds the camera even while following.
    #[test]
    fn following_eases_the_camera_onto_the_layout_and_a_pan_stops_it() {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        canvas.fit_to_content();
        canvas.set_physics_paused(false);
        let nodes = canvas.layout_framing(0.0).nodes;
        let mut planted = canvas.camera();
        planted.offset.0 += 4_000.0;
        canvas.set_camera(planted);
        assert!(!canvas.view_follows(), "a placement is not followed");
        for _ in 0..60 {
            let _ = canvas.frame(800, 600);
        }
        assert_eq!(canvas.layout_framing(0.0).outside, nodes, "not following");

        canvas.set_view_follow(true);
        for _ in 0..120 {
            let _ = canvas.frame(800, 600);
        }
        let framed = canvas.layout_framing(0.0);
        assert_eq!(framed.outside, 0, "following, two seconds on: {framed:?}");
        assert!(canvas.view_follows());

        canvas.wheel(0.0, 400.0);
        assert!(!canvas.view_follows(), "a pan stops following");
        for _ in 0..120 {
            let _ = canvas.frame(800, 600);
        }
        assert!(
            canvas.layout_framing(0.0).outside > 0,
            "the pan's glide stands; nothing pulls it back"
        );

        // A node drag stops it too; a click on a node does not.
        canvas.fit_to_content();
        canvas.set_view_follow(true);
        let (key, _) = canvas.graph.nodes().next().unwrap();
        let (x, y) = canvas.screen_position_of(key).unwrap();
        canvas.pointer_down(PointerButton::Left, x, y);
        canvas.pointer_up(PointerButton::Left, x, y);
        assert!(canvas.view_follows(), "a click keeps following");
        canvas.pointer_down(PointerButton::Left, x, y);
        canvas.cursor_moved(x + 40.0, y);
        assert!(!canvas.view_follows(), "a node drag stops following");
        canvas.pointer_up(PointerButton::Left, x + 40.0, y);

        canvas.set_physics_paused(true);
        canvas.set_view_follow(true);
        let held = canvas.camera();
        for _ in 0..30 {
            let _ = canvas.frame(800, 600);
        }
        assert_eq!(canvas.camera(), held, "paused, the camera holds");
    }

    /// World and screen points round-trip through the camera at any zoom, and
    /// the focused node's world position maps to its screen position.
    #[test]
    fn world_and_screen_points_round_trip_through_the_camera() {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        canvas.set_camera(crate::canvas::CameraView {
            offset: (130.0, -40.0),
            zoom: 0.772,
        });
        let world = canvas.world_point_at((410.0, 275.0));
        let back = canvas.screen_point_of(world);
        assert!((back.0 - 410.0).abs() < 1e-3 && (back.1 - 275.0).abs() < 1e-3, "{back:?}");
        let (key, _) = canvas.graph.nodes().next().unwrap();
        canvas.selected = [key].into_iter().collect();
        let at = canvas.focused_world_position().unwrap();
        let screen = canvas.focused_screen_position().unwrap();
        let mapped = canvas.screen_point_of(at);
        assert!((mapped.0 - screen.0).abs() < 1e-3 && (mapped.1 - screen.1).abs() < 1e-3);
        // A world step of 220 is 220 x zoom screen px under the top-down camera.
        let step = canvas.screen_point_of((at.0 + 220.0, at.1));
        assert!((step.0 - screen.0 - 220.0 * 0.772).abs() < 1e-2, "{step:?}");
    }

    /// `resize` still re-centres: a genuine window resize holds whatever sits
    /// at the middle. That behaviour is the reason the swap path had to stop
    /// borrowing it, so pin it here beside its counterpart.
    #[test]
    fn resize_still_recentres() {
        let mut canvas = Canvas::with_sample_graph();
        canvas.resize(800, 600);
        let before = canvas.viewport();
        canvas.resize(1000, 600);
        let after = canvas.viewport();
        assert_eq!(
            after.offset.0 - before.offset.0,
            100.0,
            "grew 200px wide, so the centre holds by shifting half that"
        );
        assert_eq!(after.view, (1000, 600));
    }
}
