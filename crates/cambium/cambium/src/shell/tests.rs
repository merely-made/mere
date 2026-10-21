use super::*;
use crate::{DomHandle, GenetAppRunner, Key, KeyEvent, NamedKey};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace};
use std::{cell::RefCell, rc::Rc};

fn attribute(dom: &DomHandle, node: NodeId, name: &str) -> Option<String> {
    dom.borrow()
        .attribute(node, &Namespace::from(""), &LocalName::from(name))
        .map(|value| value.to_string())
}

#[test]
fn disabled_shell_control_never_dispatches_even_synthetic_clicks() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |_: &u32| {
            shell_control(
                "Read",
                "Read selected document",
                ShellControl {
                    disabled: true,
                    ..Default::default()
                },
                |count: &mut u32, _| *count += 1,
            )
        },
        0u32,
    );
    let root = runner.root();
    assert_eq!(
        attribute(&dom, root, "aria-disabled").as_deref(),
        Some("true")
    );
    runner.dispatch_click(root, PointerClick::at((0.0, 0.0)));
    assert_eq!(*runner.state(), 0);
}

#[test]
fn projection_choices_invoke_host_actions_and_retain_pressed_state() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |map: &bool| shell_projection_switch(*map, false, |map: &mut bool, next| *map = next),
        true,
    );
    let children = dom.borrow().dom_children(runner.root()).collect::<Vec<_>>();
    assert_eq!(children.len(), 2);
    assert_eq!(
        attribute(&dom, children[0], "aria-pressed").as_deref(),
        Some("true")
    );
    runner.set_focus(Some(children[1]));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert!(!*runner.state());
    assert_eq!(
        attribute(&dom, children[1], "aria-pressed").as_deref(),
        Some("true")
    );
}

#[test]
fn constrained_projection_switch_offers_the_other_projection() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        |map: &bool| shell_projection_switch(*map, true, |map: &mut bool, next| *map = next),
        false,
    );
    let children = dom.borrow().dom_children(runner.root()).collect::<Vec<_>>();
    assert_eq!(children.len(), 1);
    assert_eq!(
        attribute(&dom, children[0], "aria-label").as_deref(),
        Some("Map")
    );
    runner.dispatch_click(children[0], PointerClick::at((0.0, 0.0)));
    assert!(*runner.state());
}

#[test]
fn density_tracks_caption_reservations_and_zoom() {
    for (width, density) in [
        (299.0, ShellDensity::Tiny),
        (300.0, ShellDensity::Compact),
        (540.0, ShellDensity::Regular),
        (700.0, ShellDensity::Regular),
        (701.0, ShellDensity::Wide),
    ] {
        assert_eq!(ShellDensity::for_width(width, 1.0), density);
        assert_eq!(ShellDensity::for_width(width * 2.0, 2.0), density);
    }
}

#[test]
fn activity_buttons_keep_keyboard_activation_and_independent_tool_state() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), |state: &(bool, bool)| {
        shell_activity_rail(vec![shell_activity_control("◇", "Map", "Map", ShellControl { pressed: Some(state.0), ..Default::default() }, |s: &mut (bool, bool), _| s.0 = !s.0)],
            vec![shell_activity_control("ⓘ", "Inspect", "Inspector", ShellControl { pressed: Some(state.1), ..Default::default() }, |s: &mut (bool, bool), _| s.1 = !s.1)], vec![])
    }, (true, false));
    let buttons = dom.borrow().all_with_class(runner.root(), "shell-activity-control");
    runner.set_focus(Some(buttons[1]));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert_eq!(*runner.state(), (true, true), "tools do not deselect the main projection");
    assert_eq!(attribute(&dom, buttons[1], "aria-pressed").as_deref(), Some("true"));
    let marks = dom.borrow().all_with_class(runner.root(), "shell-activity-mark");
    assert_eq!(attribute(&dom, marks[0], "aria-hidden").as_deref(), Some("true"));
}

#[test]
fn document_rail_keeps_one_selected_member_per_tile_and_dispatches_exact_target() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), |selected: &[usize; 2]| {
        shell_document_rail((0..2).map(|tile| (format!("Tile {}", tile + 1), (0..2).map(|tab| {
            shell_control(format!("Document {tile}:{tab}"), format!("Document {tile}:{tab}"), ShellControl { pressed: Some(selected[tile] == tab), ..Default::default() }, move |s: &mut [usize; 2], _| s[tile] = tab)
        }).collect())).collect())
    }, [0, 0]);
    let buttons = dom.borrow().all_with_class(runner.root(), "shell-control");
    runner.dispatch_click(buttons[1], PointerClick::at((0.0, 0.0)));
    assert_eq!(*runner.state(), [1, 0]);
    assert_eq!(attribute(&dom, buttons[1], "aria-pressed").as_deref(), Some("true"));
    assert_eq!(attribute(&dom, buttons[2], "aria-pressed").as_deref(), Some("true"));
}
