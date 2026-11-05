// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Spatial admission against the last accepted frame on the native path.

use super::*;
use layout_dom_api::LayoutDom as _;

#[cfg(test)]
mod tests;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// The topmost node at an arbitrary point, in the retained layout's own
    /// coordinates.
    ///
    /// Hit testing is host machinery, not window machinery: a browser host asks
    /// the same question of the same layout. Client-side decorations compose
    /// this with their own reading of the node, rather than repeating the walk.
    pub fn hit_at(&self, x: f32, y: f32) -> Option<NodeId> {
        let runner = self.s.runner.as_ref()?;
        let dom = runner.dom();
        let dom_ref = dom.borrow();
        let view = crate::WindowDom::new(&dom_ref, runner.mount());
        if self.s.native_receiving {
            // Refuse a removed displayed hit. Never hit-test the pending
            // layout for an alternative target underneath it.
            self.s
                .displayed_input
                .as_ref()?
                .hit_at(x, y)
                .filter(|node| view.contains(*node))
        } else {
            self.s.layout.as_ref()?.hit_test(&view, x, y)
        }
    }

    /// The retained layout, for callers that read it without re-hit-testing.
    pub fn layout(&self) -> Option<&crate::OwnedLayout> {
        self.s.layout.as_ref()
    }

    /// The cursor in `node`'s **local** coordinate space, plus `node`'s box
    /// size — the pair a Cambium pointer or wheel handler normalizes with
    /// (`local.0 / size.0` is a slider's value) without knowing layout.
    ///
    /// Use Genet's inverse accumulated paint transform and scroll geometry.
    /// Custom leaves use their content box; ordinary controls use their border box.
    /// `((0, 0), (0, 0))` when the node has no laid-out box.
    pub(crate) fn local_in(&self, node: NodeId) -> ((f32, f32), (f32, f32)) {
        self.input_local(node).unwrap_or(((0.0, 0.0), (0.0, 0.0)))
    }

    /// Native dispatch needs positive geometry admission; the legacy path
    /// keeps its existing zero-size fallback when no layout box is available.
    pub(crate) fn input_local(&self, node: NodeId) -> Option<((f32, f32), (f32, f32))> {
        if self.s.native_receiving {
            let runner = self.s.runner.as_ref()?;
            let dom = runner.dom();
            let dom = dom.borrow();
            let view = crate::WindowDom::new(&dom, runner.mount());
            if !view.contains(node) {
                return None;
            }
            return self
                .s
                .displayed_input
                .as_ref()?
                .local_in(node, self.s.cursor);
        }
        let local =
            self.s
                .runner
                .as_ref()
                .zip(self.s.layout.as_ref())
                .and_then(|(runner, layout)| {
                    let dom = runner.dom();
                    let dom_ref = dom.borrow();
                    let view = crate::WindowDom::new(&dom_ref, runner.mount());
                    layout.local_coordinates(&view, node, self.s.cursor)
                });
        Some(local.unwrap_or(((0.0, 0.0), (0.0, 0.0))))
    }

    pub(super) fn producer_admits_pointer(&self, node: NodeId) -> bool {
        let Some(runner) = self.s.runner.as_ref() else {
            return false;
        };
        let dom = runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, runner.mount());
        if self.s.native_receiving {
            view.contains(node)
                && self.s.displayed_input.as_ref().is_some_and(|displayed| {
                    displayed.admits_producer_pointer(node, self.s.cursor, &self.s.shared.producers)
                })
        } else {
            self.s.layout.as_ref().is_some_and(|layout| {
                layout.producer_admits_pointer(&view, node, self.s.cursor, &self.s.shared.producers)
            })
        }
    }

    /// Proof for geometry defaults not yet exposed on PresentedInput. It
    /// expires on DOM mutation, a new layout session/rebuild, or either scroll
    /// plane moving, and becomes usable again after the frame is published.
    pub(crate) fn input_layout_is_displayed(&self) -> bool {
        if !self.s.native_receiving {
            return true;
        }
        let Some((runner, layout)) = self.s.runner.as_ref().zip(self.s.layout.as_ref()) else {
            return false;
        };
        let dom = runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, runner.mount());
        self.s.displayed_input.as_ref().is_some_and(|displayed| {
            displayed.matches_dom(&view) && displayed.matches_layout(layout)
        })
    }

    fn input_caret(&self, node: NodeId) -> Option<VisualCaret> {
        let runner = self.s.runner.as_ref()?;
        let dom = runner.dom();
        let dom = dom.borrow();
        let view = crate::WindowDom::new(&dom, runner.mount());
        let (x, y) = self.s.cursor;
        if self.s.native_receiving {
            let displayed = self.s.displayed_input.as_ref()?;
            if !view.contains(node) || !displayed.matches_dom(&view) {
                return None;
            }
            displayed.caret_position(node, x, y)
        } else {
            self.s
                .layout
                .as_ref()?
                .caret_position_at_point(&view, node, x, y)
        }
    }

    /// Place the caret at the cursor when the press focused a text field, and
    /// anchor a drag selection there.
    pub(super) fn place_caret_at_cursor(&mut self) {
        let Some(runner) = self.s.runner.as_ref() else {
            return;
        };
        let Some(slot) = (self.hooks.focused_text)(runner) else {
            return;
        };
        let Some(caret) = self.input_caret(slot.node) else {
            return;
        };
        let Some(runner) = self.s.runner.as_mut() else {
            return;
        };
        runner.update(|state| {
            let input = (slot.get_mut)(state);
            let position = from_visual_caret(caret);
            input.apply(TextCommand::SetSelection(CaretSelection {
                anchor: position,
                focus: position,
            }));
        });
        self.s.text_drag = Some(slot.node);
    }

    pub fn drag_text_selection(&mut self) {
        let Some(drag_node) = self.s.text_drag else {
            return;
        };
        let Some(runner) = self.s.runner.as_ref() else {
            return;
        };
        let Some(slot) = (self.hooks.focused_text)(runner) else {
            self.s.text_drag = None;
            return;
        };
        if slot.node != drag_node {
            self.s.text_drag = None;
            return;
        }
        let Some(caret) = self.input_caret(slot.node) else {
            return;
        };
        let Some(runner) = self.s.runner.as_mut() else {
            return;
        };
        runner.update(|state| {
            let input = (slot.get_mut)(state);
            let anchor = input.caret_selection().anchor;
            input.apply(TextCommand::SetSelection(CaretSelection {
                anchor,
                focus: from_visual_caret(caret),
            }));
        });
        self.after_dispatch();
    }

    /// The focused text field's node and selection, before dispatch.
    pub(super) fn focused_text_selection(&self) -> Option<(NodeId, CaretSelection)> {
        let runner = self.s.runner.as_ref()?;
        let slot = (self.hooks.focused_text)(runner)?;
        Some((slot.node, (slot.get)(runner.state()).caret_selection()))
    }

    /// The host's caret default: move the focused field's selection **visually**
    /// — through the laid-out lines, so wrapped and bidi text move the way they
    /// look — starting from `base`, the selection as it stood before dispatch.
    ///
    /// The field's own key handler has already applied its *logical* move by
    /// the time this runs (arrow keys route like any other key now), so this
    /// deliberately recomputes from `base` and overwrites it. The layout-aware
    /// answer wins; the field keeps working unchanged in a host that has no
    /// layout to consult.
    pub(super) fn move_focused_text_visual(
        &mut self,
        press: &KeyPress,
        base: Option<(NodeId, CaretSelection)>,
    ) {
        // Keep the field's ordinary logical key handling while a newer DOM or
        // geometry is pending. Never override it using undisplayed lines.
        if !self.input_layout_is_displayed() {
            return;
        }
        let word = press.modifiers.ctrl || press.modifiers.alt;
        let Some(movement) = press.caret_movement(word) else {
            return;
        };
        let Some((base_node, base_selection)) = base else {
            return;
        };
        let (Some(runner), Some(layout)) = (self.s.runner.as_mut(), self.s.layout.as_ref()) else {
            return;
        };
        let Some(slot) = (self.hooks.focused_text)(runner) else {
            return;
        };
        // Focus moved during dispatch: there is no caret context to continue.
        if slot.node != base_node {
            return;
        }
        // The borrow ends with the move: the update below rebuilds the tree.
        let moved = {
            let dom = runner.dom();
            let dom_ref = dom.borrow();
            layout.selection_visual_move(
                &crate::WindowDom::new(&dom_ref, runner.mount()),
                slot.node,
                to_visual_selection(base_selection),
                movement,
                press.modifiers.shift,
            )
        };
        let Some(moved) = moved else {
            return;
        };
        runner.update(|state| {
            (slot.get_mut)(state).apply(TextCommand::SetSelection(from_visual_selection(moved)));
        });
    }

    /// A wheel notch, in logical pixels. The event source normalizes lines
    /// versus pixels; both winit and the DOM report both kinds.
    pub fn wheel(&mut self, dx: f32, dy: f32) {
        let _trace = trace::InputSpan::start("wheel");
        if self.s.native_receiving && self.s.displayed_input.is_none() {
            return;
        }
        // The notch as it arrived, before the Shift axis swap: Ctrl+wheel
        // zooms on the vertical gesture whatever Shift is doing.
        let raw_dy = dy;
        // Desktop convention (shared cambium-winit policy): Shift + vertical
        // wheel scrolls horizontally.
        let (dx, dy) = wheel_axes(dx, dy, self.s.modifiers.shift);
        let hit = self.hit_at_cursor();
        // 1. The view layer first: the nearest `on_wheel` ancestor of the hit
        //    node gets the notch, with cursor-local coordinates so it can
        //    anchor a zoom under the pointer.
        let mut prevented = false;
        if let Some(node) = hit {
            let target = self
                .s
                .runner
                .as_ref()
                .and_then(|runner| runner.wheel_target(node));
            if let Some(target) = target
                && let Some((local, size)) = self.input_local(target)
            {
                if let Some(runner) = self.s.runner.as_mut() {
                    runner.dispatch_wheel(
                        node,
                        WheelEvent::new((dx, dy), local, size)
                            .with_modifiers(self.s.modifiers.into()),
                    );
                    prevented = runner.default_prevented();
                }
                self.after_dispatch();
            }
        }
        // 2. The host default: scroll the nearest overflow container under the
        //    cursor (the engine hit-tests, clamps, and chains). Suppressed when
        //    a handler consumed the notch, so a canvas that pans on the wheel
        //    does not also scroll the page behind it.
        if prevented {
            return;
        }
        // 2a. Ctrl+wheel steps the zoom instead of scrolling — a host default
        //     like the scrolling one, and gated the same way, so a canvas that
        //     runs its own Ctrl+wheel zoom keeps it by preventing the default.
        //     Wheeling *up* (toward the start of the document) enlarges, which
        //     is every browser's direction.
        if self.s.modifiers.ctrl && raw_dy != 0.0 {
            self.step_ui_zoom(raw_dy < 0.0);
            self.after_dispatch();
            return;
        }
        // scroll_at_target performs another engine hit test. Use it only
        // while the mutable layout still exactly matches the displayed one.
        // A moved scroll plane waits for publication before another default.
        if !self.input_layout_is_displayed() {
            return;
        }
        let (x, y) = self.s.cursor;
        let scrolled = if let (Some(runner), Some(layout)) =
            (self.s.runner.as_ref(), self.s.layout.as_mut())
        {
            let dom = runner.dom();
            let dom_ref = dom.borrow();
            let view = crate::WindowDom::new(&dom_ref, runner.mount());
            layout.scroll_at_target(&view, x, y, dx, dy)
        } else {
            None
        };
        if let Some(target) = scrolled {
            // Wake the scrolled target's overlay bar; the fade clock keeps
            // redraws coming until it hides again.
            self.s.scrollbar_fade.note(target, crate::Instant::now());
            if let Some(window) = self.s.window.as_ref() {
                window.request_redraw();
            }
        }
    }
}
