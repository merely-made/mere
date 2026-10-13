// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use cambium::{
    AnyView, GenetAppRunner, GenetCtx, GenetElement, PointerClick, WorkspacePaneStatus, button,
    workspace_pane_header,
};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::LayoutDom;
use std::{cell::RefCell, rc::Rc};

#[derive(Default)]
struct State {
    pane: WorkspacePaneStatus,
    opened: usize,
}
type View = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;

fn view(state: &State) -> View {
    workspace_pane_header(
        "Lyrics",
        "resource:retained-lyrics",
        state.pane,
        Box::new(button(
            "Open source",
            |state: &mut State, _: PointerClick| state.opened += 1,
        )),
    )
}

fn make() -> GenetAppRunner<State, fn(&State) -> View, View> {
    GenetAppRunner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as fn(&State) -> View,
        State::default(),
    )
}

#[test]
fn attention_and_working_state_do_not_open_or_save_the_material() {
    let mut runner = make();
    runner.update(|state| {
        state.pane = WorkspacePaneStatus {
            selected: true,
            ..Default::default()
        }
    });
    assert_eq!(runner.state().opened, 0);
    assert!(!runner.state().pane.active);
    assert!(!runner.state().pane.dirty);
    let dom = runner.dom();
    let dom = dom.borrow();
    let states = dom.all_with_class(dom.document(), "workspace-pane-state");
    assert_eq!(states.len(), 1);
    assert_eq!(
        dom.dom_children(states[0])
            .filter_map(|n| dom.text(n))
            .collect::<String>(),
        "Selected material"
    );
}

#[test]
fn updating_pane_status_preserves_action_identity_and_keyboard_focus() {
    let mut runner = make();
    let action = {
        let dom = runner.dom();
        let dom = dom.borrow();
        dom.all_with_class(dom.document(), "workspace-pane-actions")
            .into_iter()
            .flat_map(|n| dom.dom_children(n))
            .next()
            .unwrap()
    };
    runner.set_focus(Some(action));
    runner.update(|state| {
        state.pane = WorkspacePaneStatus {
            active: true,
            selected: true,
            dirty: true,
            loading: true,
        }
    });
    assert_eq!(runner.focus(), Some(action));
    assert!(runner.dom().borrow().is_live(action));
    runner.dispatch_click(action, PointerClick::at((2., 2.)));
    assert_eq!(runner.state().opened, 1);
    let dom = runner.dom();
    let dom = dom.borrow();
    assert_eq!(
        dom.all_with_class(dom.document(), "workspace-pane-state")
            .len(),
        4
    );
    drop(dom);
    runner.update(|state| state.pane = WorkspacePaneStatus::default());
    assert_eq!(runner.focus(), Some(action));
}
