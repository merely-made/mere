// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pointer, keyboard, IME, and wheel routing. Extracted from the
//! woodshed-genet donor, then completed against Cambium's full native event
//! vocabulary (drag capture, wheel handlers, routed Tab).
//!
//! The routing rule everywhere below is the browser's: **the view layer sees
//! the event first, the host default runs second, and a handler that calls
//! `prevent_default` cancels the host default**. That is why Tab goes through
//! [`dispatch_key`](cambium::GenetAppRunner::dispatch_key) rather than straight
//! to `focus_traverse`, and why click-to-caret and wheel scrolling are gated on
//! [`default_prevented`](cambium::GenetAppRunner::default_prevented).

use cambium::{
    CaretPosition, CaretSelection, PointerButton, PointerClick, PointerEvent, PointerPhase,
    TextCommand, WheelEvent,
};
use cambium_winit::wheel_axes;
use genet_render::{VisualAffinity, VisualCaret, VisualSelection};
use genet_scripted_dom::NodeId;

use crate::meristem_bounds::RootView;
use crate::{Host, Key, KeyPress, NamedKey};

mod presented;
mod trace;

pub(crate) fn to_visual_caret(caret: CaretPosition) -> VisualCaret {
    VisualCaret {
        byte: caret.byte,
        affinity: match caret.affinity {
            cambium::CaretAffinity::Downstream => VisualAffinity::Downstream,
            cambium::CaretAffinity::Upstream => VisualAffinity::Upstream,
        },
    }
}

pub(crate) fn from_visual_caret(caret: VisualCaret) -> CaretPosition {
    CaretPosition {
        byte: caret.byte,
        affinity: match caret.affinity {
            VisualAffinity::Downstream => cambium::CaretAffinity::Downstream,
            VisualAffinity::Upstream => cambium::CaretAffinity::Upstream,
        },
    }
}

fn to_visual_selection(selection: CaretSelection) -> VisualSelection {
    VisualSelection {
        anchor: to_visual_caret(selection.anchor),
        focus: to_visual_caret(selection.focus),
    }
}

fn from_visual_selection(selection: VisualSelection) -> CaretSelection {
    CaretSelection {
        anchor: from_visual_caret(selection.anchor),
        focus: from_visual_caret(selection.focus),
    }
}

/// Whether `CAMBIUM_HOST_KEY_TRACE` asks for a per-keypress trace. Read once:
/// a keyboard path is not the place to hit the environment per event.
fn key_trace() -> bool {
    use std::sync::OnceLock;
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| {
        std::env::var("CAMBIUM_HOST_KEY_TRACE").is_ok_and(|v| v != "0" && !v.is_empty())
    })
}

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// The topmost node under the cursor, in the retained layout's own
    /// coordinates. The one hit-test every pointer path goes through.
    pub fn hit_at_cursor(&self) -> Option<NodeId> {
        let (x, y) = self.s.cursor;
        self.hit_at(x, y)
    }

    /// The pointer moved to `(x, y)` in logical coordinates.
    ///
    /// One call, because the order of what follows is host policy rather than
    /// event-source policy: a captured drag must get the move before the text
    /// selection, so an `on_pointer` element that took the press owns the
    /// gesture. An event source that had to reproduce that sequence would be
    /// one divergence away from a subtle input bug, and there is about to be a
    /// second event source.
    pub fn pointer_moved(&mut self, x: f32, y: f32) {
        let _trace = trace::InputSpan::start("pointer-move");
        self.s.cursor = (x, y);
        let captured = self
            .s
            .runner
            .as_ref()
            .is_some_and(|runner| runner.pointer_capture().is_some());
        // Pointer capture owns the gesture target. Resolving ordinary hover
        // against whatever lies under the cursor would dispatch boundary
        // events to unrelated views and rebuild their DOM while a captured
        // high-frequency leaf drag is deliberately retaining its target.
        if !captured {
            self.hover();
            self.hover_dispatch();
        }
        self.pointer_move();
        self.drag_text_selection();
    }

    /// The pointer position the input path tracks, in logical coordinates.
    pub fn cursor(&self) -> (f32, f32) {
        self.s.cursor
    }

    /// A left-button press in the content area: click, then drag capture, then
    /// the host's own click-to-caret default.
    pub fn click(&mut self) {
        self.s.text_drag = None;
        let Some(node) = self.hit_at_cursor() else {
            return;
        };
        if !self.producer_admits_pointer(node) {
            return;
        }
        // 1. The click, carrying the real hit point in the target's space.
        let Some((local, _)) = self.input_local(node) else {
            return;
        };
        let mut prevented = {
            let Some(runner) = self.s.runner.as_mut() else {
                return;
            };
            runner.dispatch_click(node, PointerClick::at(local));
            runner.default_prevented()
        };
        // 2. Begin a drag when the press lands under an `on_pointer` element.
        //    `pointer_target` resolves the element that *would* capture without
        //    starting the drag, so the coordinates are measured against the
        //    capturing element rather than the deeper node the cursor hit.
        let capture = self
            .s
            .runner
            .as_ref()
            .and_then(|runner| runner.pointer_target(node));
        if let Some(target) = capture
            && let Some((local, size)) = self.input_local(target)
        {
            if let Some(runner) = self.s.runner.as_mut() {
                runner.dispatch_pointer_down(
                    node,
                    PointerEvent::new(PointerPhase::Down, local, size)
                        .with_pointer_id(self.s.pointer_id)
                        .with_modifiers(self.s.modifiers.into()),
                );
                prevented |= runner.default_prevented();
            }
        }
        // 3. Click-to-caret: a host default, so a handler that prevented the
        //    default keeps its own press semantics (a drag handle does not want
        //    the caret moved out from under it).
        if !prevented {
            self.place_caret_at_cursor();
        }
        self.after_dispatch();
    }

    /// A secondary (right) press in the content area: the same `on_pointer`
    /// element a primary press would capture receives a `Down` marked
    /// [`PointerButton::Secondary`].
    ///
    /// Three deliberate differences from [`click`](Self::click):
    ///
    /// - **No `dispatch_click`.** A right press is not a click; a button that
    ///   activates on the left must not activate on the right.
    /// - **No capture.** The press is one-shot: the runner leaves
    ///   `pointer_capture` alone (see
    ///   [`PointerButton`](cambium::PointerButton)), so a right press during a
    ///   drag does not steal the gesture, and a later left release cannot
    ///   deliver an `Up` for a capture that never began. A context menu opens
    ///   on the press and lives in the view's own state afterwards; it needs no
    ///   move or release.
    /// - **No click-to-caret.** The host default belongs to the primary
    ///   button.
    ///
    /// The dispatch happens for any hit node, even one with no `on_pointer`
    /// ancestor, so `default_prevented` reports *this* press rather than a
    /// stale value from an earlier one — which is what the window frame's
    /// system-menu default is gated on.
    pub fn secondary_press(&mut self) {
        self.pointer_press(PointerButton::Secondary);
    }

    /// Non-primary native button press, preserving button and pointer identity.
    pub fn pointer_press(&mut self, button: PointerButton) {
        let Some(node) = self.hit_at_cursor() else {
            return;
        };
        if !self.producer_admits_pointer(node) {
            return;
        }
        // Measure against the element that would capture, exactly as `click`
        // does, so `local`/`size` are the handler's own box rather than the
        // deeper node the cursor happened to hit.
        let target = self
            .s
            .runner
            .as_ref()
            .and_then(|runner| runner.pointer_target(node));
        let Some((local, size)) = self.input_local(target.unwrap_or(node)) else {
            return;
        };
        if let Some(runner) = self.s.runner.as_mut() {
            runner.dispatch_pointer_down(
                node,
                PointerEvent::new(PointerPhase::Down, local, size)
                    .with_button(button)
                    .with_pointer_id(self.s.pointer_id)
                    .with_modifiers(self.s.modifiers.into()),
            );
        }
        self.after_dispatch();
    }

    /// A pointer move while a drag is captured. Routed to the capturing
    /// element even when the cursor has left its box — that is what capture
    /// means, and it is why the coordinates come from the captured element's
    /// rect rather than the hit test.
    pub fn pointer_move(&mut self) {
        let Some(target) = self
            .s
            .runner
            .as_ref()
            .and_then(|runner| runner.pointer_capture())
        else {
            return;
        };
        let Some((local, size)) = self.input_local(target) else {
            return;
        };
        if let Some(runner) = self.s.runner.as_mut() {
            let button = runner
                .pointer_capture_owner()
                .map_or(PointerButton::Primary, |(_, button)| button);
            runner.dispatch_pointer_move(
                PointerEvent::new(PointerPhase::Move, local, size)
                    .with_button(button)
                    .with_pointer_id(self.s.pointer_id)
                    .with_modifiers(self.s.modifiers.into()),
            );
        }
        self.after_dispatch();
    }

    /// A left-button release: end any captured drag, then the text-drag anchor.
    pub fn release(&mut self) {
        self.release_button(PointerButton::Primary);
    }

    pub fn release_button(&mut self, button: PointerButton) {
        let capture = self
            .s
            .runner
            .as_ref()
            .and_then(|runner| runner.pointer_capture());
        if let Some(target) = capture
            && let Some((local, size)) = self.input_local(target)
        {
            if let Some(runner) = self.s.runner.as_mut() {
                runner.dispatch_pointer_up(
                    PointerEvent::new(PointerPhase::Up, local, size)
                        .with_button(button)
                        .with_pointer_id(self.s.pointer_id)
                        .with_modifiers(self.s.modifiers.into()),
                );
            }
            self.after_dispatch();
        }
        self.s.text_drag = None;
    }

    /// Cancel capture without delivering an activating release.
    pub fn cancel_pointer(&mut self) {
        let capture = self
            .s
            .runner
            .as_ref()
            .and_then(|r| r.pointer_capture().zip(r.pointer_capture_owner()));
        if let Some((target, (pointer_id, button))) = capture
            && let Some((local, size)) = self.input_local(target)
        {
            if let Some(runner) = self.s.runner.as_mut() {
                runner.dispatch_pointer_cancel(
                    PointerEvent::new(PointerPhase::Cancel, local, size)
                        .with_button(button)
                        .with_pointer_id(pointer_id)
                        .with_modifiers(self.s.modifiers.into()),
                );
            }
            self.after_dispatch();
        }
        self.s.text_drag = None;
    }
}

#[cfg(test)]
#[path = "input/key_layout_regression.rs"]
mod key_layout_regression;

#[path = "input/keyboard.rs"]
mod keyboard;
