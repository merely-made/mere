// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::HostTree as _;
use cambium::{
    AnyView, GenetCtx, GenetElement, HoverEvent, el, on_click, on_hover, on_pointer, on_wheel,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

struct App {
    left: u32,
    show: bool,
    clicks: usize,
    background_clicks: usize,
    pointers: Vec<(PointerPhase, (f32, f32))>,
    hovers: Vec<(cambium::HoverPhase, (f32, f32))>,
    wheels: Vec<(f32, f32)>,
    text: cambium::TextInput,
}

type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;
type TestHost = Host<App, Logic, Child>;

fn view(app: &App) -> Child {
    let target = app.show.then(|| on_hover(on_wheel(on_pointer(on_click(
        el("button", "target").attr("id", "target").attr("style", format!(
            "position:absolute;left:{}px;top:10px;width:80px;height:40px;padding:0;border:0;", app.left)),
        |app: &mut App, _: PointerClick| { app.clicks += 1; }),
        |app: &mut App, event: PointerEvent| { app.pointers.push((event.phase, event.local)); }),
        |app: &mut App, event: WheelEvent| { event.prevent_default(); app.wheels.push(event.local); }),
        |app: &mut App, event: HoverEvent| { app.hovers.push((event.phase, event.local)); }));
    Box::new(el("main", (
        on_click(el("button", "underneath").attr("id", "background").attr("style",
            "position:absolute;left:20px;top:10px;width:80px;height:40px;padding:0;border:0;"),
            |app: &mut App, _: PointerClick| { app.background_clicks += 1; }),
        target,
        el("div", app.text.text().to_owned()).attr("id", "field").attr("style",
            "position:absolute;left:20px;top:80px;width:180px;height:40px;font-size:16px;line-height:24px;"),
        el("div", el("div", "").attr("style", "height:400px;width:60px;")).attr("id", "scroller").attr("style",
            "position:absolute;left:220px;top:10px;width:60px;height:70px;overflow:auto;"),
    )))
}

fn fixture(native: bool) -> TestHost {
    let mut state: crate::HostState<App, Logic, Child> = crate::HostState::new();
    state.runner = Some(crate::Runner::new(
        Rc::new(RefCell::new(crate::ScriptedDom::new())),
        view as Logic,
        App {
            left: 20,
            show: true,
            clicks: 0,
            background_clicks: 0,
            pointers: Vec::new(),
            hovers: Vec::new(),
            wheels: Vec::new(),
            text: cambium::TextInput::new("displayed words"),
        },
    ));
    let mut hooks: crate::HostHooks<App, Logic, Child> = crate::HostHooks::inert();
    hooks.focused_text = Box::new(|runner| {
        let node = runner.focus()?;
        let dom = runner.dom();
        let dom = dom.borrow();
        if !dom.is_live(node)
            || dom.attribute(node, &Default::default(), &"id".into()) != Some("field")
        {
            return None;
        }
        Some(crate::FocusedTextSlot {
            node,
            get: Box::new(|app: &App| &app.text),
            get_mut: Box::new(|app: &mut App| &mut app.text),
        })
    });
    let wake = crate::HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(crate::HostOptions::default(), None, hooks, state, wake);
    host.relayout(300.0, 200.0);
    host.s.native_receiving = native;
    host
}

fn node(host: &TestHost, id: &str) -> NodeId {
    let runner = host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let dom = dom.borrow();
    let window = crate::WindowDom::new(&dom, runner.mount());
    let found = crate::owned_layout::flat_nodes(&window)
        .find(|node| dom.attribute(*node, &Default::default(), &"id".into()) == Some(id))
        .unwrap();
    found
}

// CPU fixture publication: exercise the exact accepted-snapshot field without
// claiming that this windowless control acquired or displayed a GPU surface.
fn publish(host: &mut TestHost) {
    let input = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        host.s
            .layout
            .as_ref()
            .unwrap()
            .presented_input(&crate::WindowDom::new(&dom, runner.mount()))
    };
    host.s.displayed_input = Some(Rc::new(input));
}

fn move_target(host: &mut TestHost) {
    host.s.runner.as_mut().unwrap().update(|app| app.left = 160);
    host.relayout(300.0, 200.0);
}

#[test]
fn native_spatial_input_waits_for_first_accepted_snapshot() {
    let mut host = fixture(true);
    host.pointer_moved(30.0, 20.0);
    assert_eq!(host.hit_at_cursor(), None);
    host.click();
    host.pointer_press(PointerButton::Secondary);
    host.wheel(0.0, 20.0);
    let app = host.s.runner.as_ref().unwrap().state();
    assert_eq!((app.clicks, app.background_clicks), (0, 0));
    assert!(app.pointers.is_empty() && app.hovers.is_empty() && app.wheels.is_empty());
    publish(&mut host);
    host.click();
    assert_eq!(host.s.runner.as_ref().unwrap().state().clicks, 1);
    assert_eq!(
        host.s.runner.as_ref().unwrap().state().pointers,
        vec![(PointerPhase::Down, (10.0, 10.0))]
    );
}

#[test]
fn pending_layout_keeps_hit_hover_click_capture_and_wheel_in_displayed_coordinates() {
    let mut host = fixture(true);
    publish(&mut host);
    let target = node(&host, "target");
    move_target(&mut host);
    host.pointer_moved(30.0, 20.0);
    assert_eq!(host.hit_at_cursor(), Some(target));
    assert_eq!(host.s.last_hover, Some(target.raw()));
    assert!(!host.input_layout_is_displayed());
    host.click();
    host.pointer_moved(50.0, 30.0);
    host.wheel(0.0, 20.0);
    let app = host.s.runner.as_ref().unwrap().state();
    assert_eq!((app.clicks, app.background_clicks), (1, 0));
    assert!(app.hovers.iter().any(|(_, local)| *local == (10.0, 10.0)));
    assert_eq!(
        app.pointers.last(),
        Some(&(PointerPhase::Move, (30.0, 20.0)))
    );
    assert_eq!(app.wheels, vec![(30.0, 20.0)]);
    host.release();
    publish(&mut host);
    assert_ne!(host.hit_at(30.0, 20.0), Some(target));
    assert_eq!(host.hit_at(170.0, 20.0), Some(target));
}

#[test]
fn removed_displayed_node_does_not_retarget_the_pending_control_underneath() {
    let mut host = fixture(true);
    publish(&mut host);
    host.pointer_moved(30.0, 20.0);
    let old = node(&host, "target");
    host.s
        .runner
        .as_mut()
        .unwrap()
        .update(|app| app.show = false);
    host.relayout(300.0, 200.0);
    assert!(!host.s.runner.as_ref().unwrap().dom().borrow().is_live(old));
    assert_eq!(host.hit_at_cursor(), None);
    host.click();
    host.pointer_moved(31.0, 21.0);
    assert_eq!(host.s.last_hover_hit, None);
    let app = host.s.runner.as_ref().unwrap().state();
    assert_eq!((app.clicks, app.background_clicks), (0, 0));
    publish(&mut host);
    host.click();
    assert_eq!(host.s.runner.as_ref().unwrap().state().background_clicks, 1);
}

#[test]
fn native_default_scroll_waits_after_a_plane_change_then_resumes_on_publication() {
    let mut host = fixture(true);
    publish(&mut host);
    let scroller = node(&host, "scroller");
    host.s.cursor = (230.0, 20.0);
    assert!(host.input_layout_is_displayed());
    host.wheel(0.0, 20.0);
    assert_eq!(
        host.s.layout.as_ref().unwrap().element_scroll()[&scroller].1,
        20.0
    );
    assert!(!host.input_layout_is_displayed());
    host.wheel(0.0, 20.0);
    assert_eq!(
        host.s.layout.as_ref().unwrap().element_scroll()[&scroller].1,
        20.0
    );
    publish(&mut host);
    assert!(host.input_layout_is_displayed());
    host.wheel(0.0, 20.0);
    assert_eq!(
        host.s.layout.as_ref().unwrap().element_scroll()[&scroller].1,
        40.0
    );
}

#[test]
fn native_caret_and_drag_refuse_offsets_after_pending_text_mutation() {
    let mut host = fixture(true);
    let field = node(&host, "field");
    host.s.runner.as_mut().unwrap().set_focus(Some(field));
    publish(&mut host);
    host.s.cursor = (65.0, 90.0);
    let expected = host.input_caret(field).expect("retained text frame");
    host.place_caret_at_cursor();
    assert_eq!(
        host.s
            .runner
            .as_ref()
            .unwrap()
            .state()
            .text
            .caret_selection()
            .focus,
        from_visual_caret(expected)
    );
    host.s.runner.as_mut().unwrap().update(|app| {
        app.text = cambium::TextInput::new("pending replacement");
        app.text.apply(TextCommand::SetSelection(CaretSelection {
            anchor: CaretPosition::default(),
            focus: CaretPosition::default(),
        }));
    });
    host.relayout(300.0, 200.0);
    assert!(host.input_caret(field).is_none());
    let before = host
        .s
        .runner
        .as_ref()
        .unwrap()
        .state()
        .text
        .caret_selection();
    host.s.text_drag = Some(field);
    host.drag_text_selection();
    host.place_caret_at_cursor();
    assert_eq!(
        host.s
            .runner
            .as_ref()
            .unwrap()
            .state()
            .text
            .caret_selection(),
        before
    );
    publish(&mut host);
    let expected = host.input_caret(field).expect("newly accepted text frame");
    host.place_caret_at_cursor();
    assert_eq!(
        host.s
            .runner
            .as_ref()
            .unwrap()
            .state()
            .text
            .caret_selection()
            .focus,
        from_visual_caret(expected)
    );
}

#[test]
fn legacy_input_uses_current_layout_without_an_accepted_snapshot() {
    let mut host = fixture(false);
    let target = node(&host, "target");
    move_target(&mut host);
    assert_ne!(host.hit_at(30.0, 20.0), Some(target));
    host.s.cursor = (170.0, 20.0);
    host.click();
    assert_eq!(host.s.runner.as_ref().unwrap().state().clicks, 1);
    host.release();
    let scroller = node(&host, "scroller");
    host.s.cursor = (230.0, 20.0);
    host.wheel(0.0, 20.0);
    host.wheel(0.0, 20.0);
    assert_eq!(
        host.s.layout.as_ref().unwrap().element_scroll()[&scroller].1,
        40.0
    );
    assert!(host.s.displayed_input.is_none());
}
