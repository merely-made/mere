// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Rule workspace controls. Hosts supply capabilities and acknowledge effects;
//! this component has no graph, storage, or browser owner.
use cambium::{AnyView, GenetCtx, GenetElement, el, focusable_if, on_click};

pub type Element = Box<dyn AnyView<PresentationEditor, (), GenetCtx, GenetElement>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Effect {
    Begin,
    AddNode,
    AddRelation,
    AddField,
    Apply,
    Discard,
    Undo,
    Redo,
    Reload,
}

#[derive(Clone, Debug, Default)]
pub struct Snapshot {
    pub available: bool,
    pub editing: bool,
    pub dirty: bool,
    pub busy: bool,
    pub can_undo: bool,
    pub can_redo: bool,
    pub can_reload: bool,
    pub notice: String,
}

#[derive(Default)]
pub struct PresentationEditor {
    snapshot: Snapshot,
    effect: Option<Effect>,
}
impl PresentationEditor {
    pub fn set_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshot = snapshot;
        // A host transition may revoke an action before it is drained.
        if self.effect.is_some_and(|effect| !self.permits(effect)) {
            self.effect = None;
        }
    }
    pub fn take_effect(&mut self) -> Option<Effect> {
        self.effect.take().filter(|effect| self.permits(*effect))
    }
    pub fn has_effects(&self) -> bool {
        self.effect.is_some()
    }
    fn permits(&self, effect: Effect) -> bool {
        let s = &self.snapshot;
        if s.busy || !s.available {
            return false;
        }
        match effect {
            Effect::Begin => !s.editing,
            Effect::AddNode | Effect::AddRelation | Effect::AddField | Effect::Discard => s.editing,
            Effect::Apply => s.editing && s.dirty && !s.can_reload,
            Effect::Undo => s.can_undo && (s.editing || !s.can_reload),
            Effect::Redo => s.can_redo && (s.editing || !s.can_reload),
            Effect::Reload => s.can_reload,
        }
    }
    fn push(&mut self, effect: Effect) {
        if self.effect.is_none() && self.permits(effect) {
            self.effect = Some(effect);
        }
    }
}

pub fn view(editor: &PresentationEditor) -> Element {
    let s = &editor.snapshot;
    let mut children = Vec::<Element>::new();
    if s.editing {
        for (label, effect) in [
            ("Add node rule", Effect::AddNode),
            ("Add link rule", Effect::AddRelation),
            ("Add field rule", Effect::AddField),
            ("Apply rule draft", Effect::Apply),
            ("Discard rule draft", Effect::Discard),
        ] {
            children.push(action(label, effect, editor.permits(effect)));
        }
    } else {
        children.push(action(
            "Begin rule draft",
            Effect::Begin,
            editor.permits(Effect::Begin),
        ));
    }
    for (label, effect) in [
        (
            if s.editing {
                "Undo rule draft"
            } else {
                "Undo saved rules"
            },
            Effect::Undo,
        ),
        (
            if s.editing {
                "Redo rule draft"
            } else {
                "Redo saved rules"
            },
            Effect::Redo,
        ),
    ] {
        children.push(action(label, effect, editor.permits(effect)));
    }
    if s.can_reload {
        children.push(action(
            "Reconcile saved rules",
            Effect::Reload,
            editor.permits(Effect::Reload),
        ));
    }
    children.push(Box::new(el("p", s.notice.clone()).attr("role", "status")));
    Box::new(
        el("div", children)
            .attr("class", "presentation-editor")
            .attr("aria-label", "Presentation rule controls"),
    )
}

fn action(label: &'static str, effect: Effect, enabled: bool) -> Element {
    let element = on_click(
        el("button", label),
        move |editor: &mut PresentationEditor, _| editor.push(effect),
    );
    Box::new(focusable_if(element, enabled).attr("aria-disabled", (!enabled).to_string()))
}
