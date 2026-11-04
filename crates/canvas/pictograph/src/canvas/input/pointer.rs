// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
    /// Update whether Ctrl is held (gates wheel-zoom vs wheel-pan).
    pub fn set_ctrl(&mut self, ctrl: bool) {
        self.ctrl = ctrl;
    }

    /// Update whether Shift is held. A node click with Shift down toggles that
    /// node in the selection (multi-select) rather than replacing it.
    pub fn set_shift(&mut self, shift: bool) {
        self.shift = shift;
    }

    /// Update whether Alt is held. A left-drag with Alt down orbits the camera (yaw + tilt)
    /// instead of picking / marqueeing. (Isometric camera — orbit gesture.)
    pub fn set_alt(&mut self, alt: bool) {
        self.alt = alt;
    }

    /// The pointer moved to screen px `(x, y)`: pans on a middle-drag, pins the
    /// grabbed node on a left-drag past the slop, grows an active marquee.
    pub fn cursor_moved(&mut self, x: f32, y: f32) -> bool {
        let new = (x, y);
        let mut redraw = false;
        if let Some(prev) = self.middle_drag {
            let d = (new.0 - prev.0, new.1 - prev.1);
            self.camera.offset.0 += d.0;
            self.camera.offset.1 += d.1;
            self.pan_velocity = d;
            self.middle_drag = Some(new);
            redraw = true;
        }
        if let Some(mut press) = self.empty_press {
            if !press.panning && (new.0 - press.press.0).hypot(new.1 - press.press.1) > CLICK_SLOP {
                // A left-drag on empty canvas pans, as a middle-drag does (SE23).
                press.panning = true;
                self.follow = false;
                self.pan_velocity = (0.0, 0.0);
            }
            if press.panning {
                let d = (new.0 - press.last.0, new.1 - press.last.1);
                self.camera.offset.0 += d.0;
                self.camera.offset.1 += d.1;
                self.pan_velocity = d;
                press.last = new;
                redraw = true;
            }
            self.empty_press = Some(press);
        }
        if let Some(prev) = self.orbit_drag {
            // Alt+left orbit: horizontal drag yaws the view, vertical drag reclines the tilt
            // (toward the iso foreshorten). `set_tilt` clamps, so the camera never flips. (P2.)
            let (dx, dy) = (new.0 - prev.0, new.1 - prev.1);
            self.orbit_by(dx * ORBIT_YAW_PER_PX);
            self.set_tilt(self.camera.tilt - dy * ORBIT_TILT_PER_PX);
            self.orbit_drag = Some(new);
            self.cursor = new;
            return true;
        }
        if let Some(mut d) = self.drag {
            let was_moved = d.moved;
            // A press becomes a drag only on an item that advertises drag;
            // otherwise it stays a click (G9).
            if !d.moved
                && (new.0 - d.press.0).hypot(new.1 - d.press.1) > CLICK_SLOP
                && self.permits(d.node, ArrangementAction::Drag)
            {
                d.moved = true;
            }
            if d.moved {
                // On the click→drag transition, tell the backend to keep ticking
                // so the pinned node's neighbors react through the springs.
                if !was_moved {
                    self.unpark();
                    self.physics.set_dragging(true);
                    // A node drag stops following, as a pan does ("Drag stops
                    // following", 2026-10-04): the view holds under the hand.
                    self.follow = false;
                }
                let world = self.screen_to_world(new);
                self.place_pinned_node(d.node, world);
                redraw = true;
            }
            self.drag = Some(d);
        }
        self.cursor = new;
        if self.dragging_field() {
            // A field move / resize drag follows the cursor (its box stays active);
            // the well re-aims live. (Field regions — move/resize.)
            self.drag_field_to(new);
            redraw = true;
        } else if self.update_active_field(new) {
            // Box-on-interaction: the field under the cursor shows its extent box (the
            // disk well always draws). (Field regions.)
            redraw = true;
        }
        if self.marquee.is_some() {
            redraw = true;
        }
        redraw
    }

    /// A wheel/scroll event, `(dx, dy)` already in device px (the host maps
    /// `LineDelta` by [`WHEEL_PAN_SCALE`] / `PixelDelta` straight through). Ctrl =
    /// cursor-anchored zoom; otherwise an infinite-canvas pan impulse into inertia.
    pub fn wheel(&mut self, dx: f32, dy: f32) -> bool {
        // A pan or zoom: the camera stops following the layout.
        self.follow = false;
        if self.ctrl {
            let factor = ZOOM_STEP.powf(dy / WHEEL_PAN_SCALE);
            self.zoom_at(self.cursor, factor);
        } else {
            self.pan_velocity.0 += dx;
            self.pan_velocity.1 += dy;
        }
        true
    }

    /// The visually rendered node under a screen-px cursor. Test the same rounded
    /// billboard used for drawing and responsive link attachment, then take the
    /// front-most node by projected ground depth. Physics colliders remain the
    /// physics authority and do not define visual selection geometry.
    pub(crate) fn pick_at(&self, cursor: (f32, f32)) -> Option<NodeKey> {
        let point = Point2D::new(cursor.0, cursor.1);
        self.view
            .positions()
            .filter(|(key, _)| self.node_visible_in_canvas(*key))
            .filter_map(|(key, pos)| {
                let body = self.node_billboard_at(key, PortablePoint::new(pos.x, pos.y))?;
                body.contains_screen(point).then_some((key, body.depth))
            })
            .max_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(key, _)| key)
    }

    /// A pointer button pressed at screen px `(x, y)`. Middle begins a pan; left
    /// grabs the node under the cursor (via [`pick_at`](Self::pick_at), projection-aware)
    /// or, on empty space, begins a press that becomes a pan past the slop; right
    /// begins a marquee that a release within the slop turns into a context-menu
    /// request (SE23, SE26).
    pub fn pointer_down(&mut self, button: PointerButton, x: f32, y: f32) -> bool {
        self.cursor = (x, y);
        self.self_link_press = None;
        match button {
            PointerButton::Middle => {
                self.follow = false;
                self.middle_drag = Some(self.cursor);
                self.pan_velocity = (0.0, 0.0);
            },
            PointerButton::Left => {
                if self.alt {
                    // Alt+left begins an orbit drag (yaw + tilt the camera); it owns the gesture,
                    // so no node pick / field grab / marquee starts. (Isometric camera — orbit.)
                    self.follow = false;
                    self.orbit_drag = Some(self.cursor);
                } else if let Some(fold) = self.fold_stand_in_at_screen(self.cursor) {
                    self.fold_press = Some((fold, self.cursor));
                } else if let Some(appearance) = if self.shift {
                    None
                } else {
                    link_hit_test_segments_mapped(
                        self.rendered_link_segments(),
                        Point2D::new(self.cursor.0, self.cursor.1),
                        EDGE_PICK_TOL,
                        |key| self.node_visible_in_canvas(key),
                        |segment| segment.appearance.display_from == segment.appearance.display_to,
                        |point| self.project_link_point(point),
                    )
                } {
                    self.self_link_press = Some((appearance, self.cursor));
                } else if let Some(node) = self.pick_at(self.cursor) {
                    self.drag = Some(Drag {
                        node,
                        press: self.cursor,
                        moved: false,
                    });
                } else if self.begin_mosaic_group_press() {
                    // A Mosaic group's label band: the group moves where it is dropped.
                } else if self.begin_field_drag(self.screen_to_world(self.cursor)) {
                    // Grabbed a field's box edge (move) or corner (resize) — the deep
                    // interior fell through, so no marquee starts. (Field regions.)
                } else {
                    self.empty_press = Some(EmptyPress {
                        press: self.cursor,
                        last: self.cursor,
                        panning: false,
                    });
                }
            },
            PointerButton::Right => {
                self.marquee = Some(self.cursor);
            },
        }
        false
    }

    /// Whether a left-drag on empty canvas is panning (SE23).
    pub fn left_panning(&self) -> bool {
        self.empty_press.is_some_and(|press| press.panning)
    }

    /// Take the right click waiting for the host's context menu, if any (SE26).
    pub fn take_context_request(&mut self) -> Option<ContextRequest> {
        self.context_request.take()
    }

    /// Release a canceled gesture without activating a click, selection or
    /// context menu. Motion already applied remains; a moved node returns to
    /// its arrangement role just as on release. Pan momentum stops with capture.
    pub fn pointer_cancel(&mut self) -> bool {
        let drag = self.drag.take();
        let mut changed = drag.is_some();
        if let Some(drag) = drag.filter(|drag| drag.moved) {
            self.physics.set_dragging(false);
            self.release_dragged(drag.node);
        }
        changed |= self.middle_drag.take().is_some();
        changed |= self.orbit_drag.take().is_some();
        changed |= self.empty_press.take().is_some();
        changed |= self.marquee.take().is_some();
        changed |= self.fold_press.take().is_some();
        changed |= self.mosaic.group_press.take().is_some();
        if self.end_field_drag() {
            self.settle_physics(SETTLE_TICKS / 3);
            changed = true;
        }
        changed |= self.pan_velocity != (0.0, 0.0);
        self.pan_velocity = (0.0, 0.0);
        self.ctrl = false;
        self.shift = false;
        self.alt = false;
        changed
    }

    /// A pointer button released at screen px `(x, y)`. Ends a middle-pan or a
    /// left pan; drops a dragged node (re-settling its neighborhood) or selects a
    /// clicked node; on a bare empty click, picks the nearest edge within
    /// tolerance, else clears the selection. A right release ends a marquee
    /// (rect-select), or within the slop asks for the host's context menu.
    pub fn pointer_up(&mut self, button: PointerButton, x: f32, y: f32) -> bool {
        self.cursor = (x, y);
        match button {
            PointerButton::Middle => {
                self.middle_drag = None;
                false
            },
            PointerButton::Left => {
                // End an orbit drag: the camera already moved live, nothing to settle. (P2.)
                if self.orbit_drag.take().is_some() {
                    return true;
                }
                if let Some((fold, press)) = self.fold_press.take() {
                    if (self.cursor.0 - press.0).hypot(self.cursor.1 - press.1) <= CLICK_SLOP {
                        return self.expand_fold(&fold);
                    }
                    return true;
                }
                if let Some((appearance, press)) = self.self_link_press.take() {
                    if (self.cursor.0 - press.0).hypot(self.cursor.1 - press.1) <= CLICK_SLOP {
                        self.selected.clear();
                        self.selected_links.clear();
                        self.selected_link_appearances.clear();
                        self.selected_links.insert(appearance.key.clone());
                        self.selected_link_appearances.insert(
                            appearance.key,
                            (appearance.display_from, appearance.display_to),
                        );
                    }
                    return true;
                }
                // End a field move / resize drag: re-aim already happened live; just
                // settle the layout into the new well and persist. (Field regions.)
                if self.end_field_drag() {
                    self.settle_physics(SETTLE_TICKS / 3);
                    return true;
                }
                if self.end_mosaic_group_press() {
                    return true;
                }
                if let Some(d) = self.drag.take() {
                    if d.moved {
                        self.physics.set_dragging(false);
                        self.release_dragged(d.node);
                        self.mosaic_drop(d.node);
                    } else if self.shift {
                        // Shift-click toggles the node in the selection (multi-select).
                        self.sync_attention();
                        if !self.selected.remove(&d.node) {
                            self.selected.insert(d.node);
                        }
                        self.selected_links.clear();
                        self.selected_link_appearances.clear();
                    } else {
                        self.select_only(d.node);
                    }
                    true
                } else if let Some(press) = self.empty_press.take() {
                    // A pan already moved the camera live; its momentum glides on.
                    if !press.panning && !self.shift {
                        self.bare_empty_click();
                    }
                    true
                } else {
                    false
                }
            },
            PointerButton::Right => {
                let Some(origin) = self.marquee.take() else {
                    return false;
                };
                if (self.cursor.0 - origin.0).hypot(self.cursor.1 - origin.1) > CLICK_SLOP {
                    self.rect_select(origin);
                    true
                } else {
                    self.context_request = Some(ContextRequest {
                        at: self.cursor,
                        node: self
                            .pick_at(self.cursor)
                            .and_then(|key| self.graph.get_node(key))
                            .map(|node| node.id),
                    });
                    false
                }
            },
        }
    }

    /// Select what a marquee from `origin` to the cursor covers (SE23).
    pub(super) fn rect_select(&mut self, origin: (f32, f32)) {
        // Preserve center-in-rectangle selection, but evaluate the projected
        // visible billboard center against the actual screen rectangle. Mapping
        // only two corners back to world space becomes an over-wide AABB under yaw.
        let region = Box2D::from_points([
            Point2D::new(origin.0, origin.1),
            Point2D::new(self.cursor.0, self.cursor.1),
        ]);
        let sel: std::collections::HashSet<_> = self
            .view
            .positions()
            .filter(|(key, _)| self.node_visible_in_canvas(*key))
            .filter_map(|(key, pos)| {
                let body = self.node_billboard_at(key, PortablePoint::new(pos.x, pos.y))?;
                region.contains(body.center).then_some(key)
            })
            .collect();
        self.sync_attention();
        self.selected = sel;
        let appearances = links_in_rect_segments_mapped(
            self.rendered_link_segments(),
            region,
            |key| self.node_visible_in_canvas(key),
            |point| self.project_link_point(point),
        );
        self.selected_links = appearances.iter().map(|item| item.key.clone()).collect();
        self.selected_link_appearances = appearances
            .into_iter()
            .map(|item| (item.key, (item.display_from, item.display_to)))
            .collect();
    }

    pub(super) fn project_link_point(&self, point: Point2D<f32>) -> Point2D<f32> {
        let (x, y) = self.camera.to_screen(PortablePoint::new(point.x, point.y));
        Point2D::new(x, y)
    }

    /// A bare empty click clears the selection and may pick an edge under the
    /// cursor. The caller skips it with Shift held, so a multi-select in
    /// progress is preserved.
    pub(super) fn bare_empty_click(&mut self) {
        self.sync_attention();
        self.selected.clear();
        self.selected_links.clear();
        self.selected_link_appearances.clear();
        if let Some(appearance) = link_hit_test_segments_mapped(
            self.rendered_link_segments(),
            Point2D::new(self.cursor.0, self.cursor.1),
            EDGE_PICK_TOL,
            |key| self.node_visible_in_canvas(key),
            |_| true,
            |point| self.project_link_point(point),
        ) {
            self.selected_links.insert(appearance.key.clone());
            self.selected_link_appearances.insert(
                appearance.key,
                (appearance.display_from, appearance.display_to),
            );
        }
    }
}
