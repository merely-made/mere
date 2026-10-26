// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reusable Cambium controls for Forme layout editing. The host performs every
//! effect and supplies the current capability/status snapshot.
use cambium::{AnyView, GenetCtx, GenetElement, el, focusable_if, on_click};
use std::collections::VecDeque;

pub type Element = Box<dyn AnyView<WorkbenchEditor, (), GenetCtx, GenetElement>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Unlock,
    Apply,
    Discard,
    Undo,
    Redo,
    ToggleVisibility,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub editing: bool,
    pub dirty: bool,
    pub busy: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub visible: bool,
    pub notice: String,
}

#[derive(Default)]
pub struct WorkbenchEditor {
    snapshot: Snapshot,
    effects: VecDeque<Effect>,
}
impl WorkbenchEditor {
    pub fn set_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshot = snapshot;
    }
    pub fn take_effect(&mut self) -> Option<Effect> {
        self.effects.pop_front()
    }
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }
    fn permits(&self, effect: Effect) -> bool {
        match effect {
            Effect::Unlock => !self.snapshot.busy && !self.snapshot.editing,
            Effect::Apply => !self.snapshot.busy && self.snapshot.editing,
            Effect::Discard => !self.snapshot.busy && self.snapshot.editing,
            Effect::Undo => !self.snapshot.busy && self.snapshot.can_undo,
            Effect::Redo => !self.snapshot.busy && self.snapshot.can_redo,
            Effect::ToggleVisibility => !self.snapshot.busy,
        }
    }
    fn push(&mut self, effect: Effect) {
        if self.effects.is_empty() && self.permits(effect) {
            self.effects.push_back(effect);
        }
    }
}

pub fn view(editor: &WorkbenchEditor) -> Element {
    let s = &editor.snapshot;
    let mut controls = Vec::<Box<dyn AnyView<WorkbenchEditor, (), GenetCtx, GenetElement>>>::new();
    let toggle = if s.editing {
        Effect::Apply
    } else {
        Effect::Unlock
    };
    controls.push(action(
        if s.editing {
            "Lock and apply"
        } else {
            "Unlock forme"
        },
        toggle,
        !s.busy,
    ));
    controls.push(action(
        "Discard changes",
        Effect::Discard,
        !s.busy && s.editing,
    ));
    controls.push(action(
        if s.editing {
            "Undo forme draft"
        } else {
            "Undo saved forme change"
        },
        Effect::Undo,
        !s.busy && s.can_undo,
    ));
    controls.push(action(
        if s.editing {
            "Redo forme draft"
        } else {
            "Redo saved forme change"
        },
        Effect::Redo,
        !s.busy && s.can_redo,
    ));
    controls.push(action(
        if s.visible {
            "Hide forme boundary"
        } else {
            "Show forme boundary"
        },
        Effect::ToggleVisibility,
        !s.busy,
    ));
    if s.editing {
        controls.push(Box::new(el(
            "p",
            if s.dirty {
                "Pending Forme changes"
            } else {
                "Forme draft matches saved arrangement"
            },
        )));
    }
    controls.push(Box::new(el("p", s.notice.clone()).attr("role", "status")));
    Box::new(
        el("div", controls)
            .attr("class", "workbench-editor")
            .attr("aria-label", "Forme layout controls"),
    )
}

fn action(label: &'static str, effect: Effect, enabled: bool) -> Element {
    let element = on_click(
        el("button", label),
        move |editor: &mut WorkbenchEditor, _| {
            editor.push(effect);
        },
    );
    Box::new(focusable_if(element, enabled).attr("aria-disabled", (!enabled).to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_emit_only_actions_allowed_by_the_current_snapshot() {
        let mut editor = WorkbenchEditor::default();
        editor.push(Effect::Unlock);
        assert!(editor.has_effects());
        assert_eq!(editor.take_effect(), Some(Effect::Unlock));
        assert!(!editor.has_effects());
        editor.push(Effect::Apply);
        editor.push(Effect::Undo);
        editor.push(Effect::Discard);
        assert_eq!(editor.take_effect(), None);
        editor.set_snapshot(Snapshot {
            editing: true,
            dirty: true,
            can_undo: true,
            ..Snapshot::default()
        });
        editor.push(Effect::Apply);
        editor.push(Effect::Undo);
        editor.push(Effect::Redo);
        assert_eq!(editor.take_effect(), Some(Effect::Apply));
        assert_eq!(editor.take_effect(), None);
        editor.set_snapshot(Snapshot {
            busy: true,
            editing: true,
            dirty: true,
            can_undo: true,
            ..Snapshot::default()
        });
        editor.push(Effect::Apply);
        assert_eq!(editor.take_effect(), None);
    }
}
