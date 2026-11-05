// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl<State, Logic, V, T> Host<State, Logic, V, T>
where
    T: crate::HostTree<State>,
    State: 'static,
    Logic: FnMut(&State) -> V + 'static,
    V: RootView<State>,
{
    /// Release the named key in the host's platform-neutral input state.
    /// Releasing Tab ends held-Tab spatial navigation; other releases have no
    /// host default. This does not synthesize an application key press/action.
    pub fn key_released(&mut self, key: &Key) {
        if matches!(key, Key::Named(NamedKey::Tab)) {
            self.s.tab_held = false;
        }
    }

    pub fn key(&mut self, press: &KeyPress) {
        // `CAMBIUM_HOST_KEY_TRACE=1` reports every key the host was handed and
        // what became of it. It exists because "typing does not work" has three
        // very different causes — the window never got the event, winit could
        // not name the key, or the tree dropped it — and from outside they look
        // identical. One line per press tells them apart.
        let trace = key_trace();
        if trace {
            eprintln!(
                "[cambium-host] key {:?} text={:?} mods={:?} focus={:?}",
                press.key,
                press.text,
                press.modifiers,
                self.s.runner.as_ref().and_then(|r| r.focus()),
            );
        }
        // Hold-Tab spatial navigation, before anything else can claim the keys.
        //
        // "Held" means physically down, not "down long enough": you press Tab
        // and arrow immediately, the way a chord feels, rather than waiting out
        // the OS repeat delay. The press itself still traverses — a tap is
        // exactly what it always was — so the mode costs nothing until an arrow
        // arrives, and the initial traversal is simply where the steering
        // starts from.
        if self.options.spatial_focus {
            if let Key::Named(NamedKey::Tab) = press.key {
                if press.repeat {
                    // Already held: swallow the repeat rather than walking
                    // document order sixty times while steering.
                    return;
                }
                if trace {
                    eprintln!("[cambium-host]   Tab down: spatial focus armed");
                }
                self.s.tab_held = true;
                // Fall through: this press traverses as it always did.
            } else if self.s.tab_held
                && let Some(dir) = press.direction()
            {
                let moved = self.input_layout_is_displayed() && self.focus_spatial(dir);
                if trace {
                    eprintln!("[cambium-host]   spatial {dir:?} moved={moved}");
                }
                return;
            }
        }
        // The application's own policy first (an Escape that closes a popover, a
        // global shortcut): it can consume the event before the tree sees it.
        {
            let Some(runner) = self.s.runner.as_mut() else {
                return;
            };
            if (self.hooks.key_intercept)(runner, press) {
                if trace {
                    eprintln!("[cambium-host]   consumed by key_intercept");
                }
                self.after_dispatch();
                return;
            }
        }
        if self.zoom_chord(press) {
            if trace {
                eprintln!(
                    "[cambium-host]   zoom chord: ui_zoom now {}",
                    self.ui_zoom()
                );
            }
            return;
        }
        let Some(kev) = press.to_runner_key() else {
            // No runner key and no injected text: a dead key, or an
            // unidentified one the platform reported no text for.
            if trace {
                eprintln!("[cambium-host]   dropped: no Cambium key and no text");
            }
            return;
        };
        if trace {
            eprintln!("[cambium-host]   dispatching {:?}", kev.key);
        }
        // Snapshot the caret before dispatch: the host's visual movement below
        // recomputes from this rather than from whatever logical move the field
        // just applied.
        let base = self.focused_text_selection();
        let prevented = {
            let Some(runner) = self.s.runner.as_mut() else {
                return;
            };
            // Tab included: the runner routes it to the focused element's
            // handlers and applies its own traversal default only if none of
            // them prevented it.
            runner.dispatch_key(kev);
            runner.default_prevented()
        };
        if !prevented {
            self.move_focused_text_visual(press, base);
        }
        self.after_dispatch();
        // Key handlers and after-dispatch hooks can change the DOM. Apply
        // those mutations before hover/focus restyling walks the new tree;
        // inserted nodes have no computed style until layout consumes them.
        let (width, height) = self.s.layout_size;
        self.relayout(width, height);
        // Focus may have moved (Tab, or a handler's `set_focus`): refresh the
        // `:hover` / `:focus` restyle. Cheap — it returns early unless the
        // hovered/focused pair actually changed.
        self.hover();
    }

    /// The host's own zoom chords: Ctrl+plus, Ctrl+minus, Ctrl+0. Returns
    /// whether the press was one and was consumed.
    ///
    /// Deliberately *after* the application's `key_intercept` and *before*
    /// `dispatch_key`. After, because an application that binds Ctrl+0 to
    /// something of its own must be able to keep it — consuming the chord in
    /// the intercept vetoes the default, which is the same veto Escape and
    /// every other global shortcut already has. Before, because a zoom step is
    /// chrome: it belongs to the window, not to whichever control happens to
    /// hold focus, and routing it into the tree first would let a text field
    /// swallow it as typed text.
    ///
    /// Shift is ignored rather than rejected: on a US layout `Ctrl+Shift+=` is
    /// how the plus sign is actually typed, and a person pressing it means
    /// "bigger". Alt and Super are rejected, because those are somebody else's
    /// chords.
    pub(super) fn zoom_chord(&mut self, press: &KeyPress) -> bool {
        if !press.modifiers.ctrl || press.modifiers.alt || press.modifiers.meta {
            return false;
        }
        let Key::Character(character) = &press.key else {
            return false;
        };
        let stepped = match character.as_str() {
            "+" | "=" => self.step_ui_zoom(true),
            "-" | "_" => self.step_ui_zoom(false),
            // Ctrl+0 is "back to normal". With `fit_design` set, normal is
            // what fits, so this clears the user's offset rather than forcing
            // the interface to 1.0.
            "0" => self.reset_ui_zoom(),
            _ => return false,
        };
        // Consumed either way: a step that clamped at the end of the ladder is
        // still the host's key, and letting it fall through to the tree would
        // type a `+` into a text field at maximum zoom.
        let _ = stepped;
        self.after_dispatch();
        true
    }

    /// A platform IME lifecycle event, already in Cambium's neutral
    /// composition vocabulary. The event source converts; a browser reports
    /// the same four states through `compositionstart`/`update`/`end`.
    pub fn ime(&mut self, composition: cambium::CompositionEvent) {
        let Some(runner) = self.s.runner.as_mut() else {
            return;
        };
        if (self.hooks.focused_text)(runner).is_none() {
            return;
        }
        runner.dispatch_key(cambium::KeyEvent::new(cambium::Key::Composition(
            composition,
        )));
        self.after_dispatch();
    }
}
