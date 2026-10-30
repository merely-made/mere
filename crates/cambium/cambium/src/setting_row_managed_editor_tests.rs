/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, rc::Rc};

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use mere_surface_api::settings::{
    SettingControl, SettingMovement, SettingMutability, SettingOption, SettingScope,
    SettingSecurity, SettingSpec, SettingValue,
};
use meristem::AnyView;

use crate::{
    CompositionEvent, DomHandle, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent, NamedKey,
    PointerEvent, PointerPhase, SettingRowEvent, SettingRowSession, ValueEvent,
    setting_row_managed,
};

#[derive(Default)]
struct App {
    session: Option<SettingRowSession>,
    events: Vec<SettingRowEvent>,
}

type AppView = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Runner = GenetAppRunner<App, fn(&App) -> AppView, AppView, ()>;

fn view(app: &App) -> AppView {
    let session = app.session.as_ref().unwrap().clone();
    let event_session = session.clone();
    Box::new(setting_row_managed(
        &session,
        "Managed",
        move |app: &mut App, event| {
            if matches!(&event, SettingRowEvent::Discard) {
                event_session.reset_to_observed();
            }
            app.events.push(event);
        },
    ))
}

fn spec(control: SettingControl, value: SettingValue) -> SettingSpec {
    SettingSpec {
        id: "managed.control".into(),
        label: "Managed control".into(),
        scope: SettingScope::Application,
        movement: SettingMovement::LocalOnly,
        mutability: SettingMutability::Live,
        security: SettingSecurity::Ordinary,
        control,
        value,
    }
}

fn runner(spec: SettingSpec) -> (DomHandle, Runner) {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut app = App::default();
    app.session = Some(SettingRowSession::new(spec));
    let runner = GenetAppRunner::new(dom.clone(), view as fn(&App) -> AppView, app);
    (dom, runner)
}

fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if dom.kind(root) == NodeKind::Element
        && dom.attribute(root, &Namespace::from(""), &LocalName::from(name)) == Some(value)
    {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_attr(dom, child, name, value))
}

fn assert_number(session: &SettingRowSession, expected: f64) {
    match session.snapshot().draft_value {
        SettingValue::Number(actual) => assert!((actual - expected).abs() < 0.0001),
        value => panic!("expected numeric setting value, got {value:?}"),
    }
}

#[test]
fn managed_slider_uses_shared_page_navigation_and_accessibility_value_path() {
    let (dom, mut runner) = runner(spec(
        SettingControl::Number {
            min: Some(0.0),
            max: Some(100.0),
            step: Some(10.0),
        },
        SettingValue::Number(50.0),
    ));
    let session = runner.state().session.as_ref().unwrap().clone();
    let slider = find_attr(&dom.borrow(), runner.root(), "role", "slider").unwrap();
    assert_eq!(
        dom.borrow().attribute(
            slider,
            &Namespace::from(""),
            &LocalName::from("aria-valuenow")
        ),
        Some("0.5")
    );

    runner.set_focus(Some(slider));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::PageUp)));
    assert_number(&session, 100.0);

    runner.dispatch_value(slider, ValueEvent { value: 0.37 });
    assert_number(&session, 37.0);
    assert_eq!(session.snapshot().edit_generation, 2);
}

#[test]
fn managed_slider_cancel_does_not_create_a_second_edit() {
    let (dom, mut runner) = runner(spec(
        SettingControl::Number {
            min: Some(0.0),
            max: Some(1.0),
            step: Some(0.1),
        },
        SettingValue::Number(0.5),
    ));
    let session = runner.state().session.as_ref().unwrap().clone();
    let slider = find_attr(&dom.borrow(), runner.root(), "role", "slider").unwrap();
    runner.dispatch_pointer_down(
        slider,
        PointerEvent::new(PointerPhase::Down, (30.0, 0.0), (100.0, 10.0)),
    );
    assert_eq!(session.snapshot().edit_generation, 1);
    assert_number(&session, 0.3);

    runner.dispatch_pointer_up(PointerEvent::new(
        PointerPhase::Cancel,
        (90.0, 0.0),
        (100.0, 10.0),
    ));
    assert_eq!(session.snapshot().edit_generation, 1);
    assert_number(&session, 0.3);
}

#[test]
fn managed_radio_uses_shared_keyboard_focus_and_choice_mapping() {
    let (dom, mut runner) = runner(spec(
        SettingControl::Choice {
            options: vec![
                SettingOption {
                    value: "one".into(),
                    label: "One".into(),
                },
                SettingOption {
                    value: "two".into(),
                    label: "Two".into(),
                },
            ],
        },
        SettingValue::Text("one".into()),
    ));
    let session = runner.state().session.as_ref().unwrap().clone();
    let first = find_attr(&dom.borrow(), runner.root(), "aria-label", "One").unwrap();
    let second = find_attr(&dom.borrow(), runner.root(), "aria-label", "Two").unwrap();

    runner.set_focus(Some(first));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowRight)));
    assert_eq!(runner.focus(), Some(second));
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("two".into())
    );
    assert!(
        matches!(runner.state().events.last(), Some(SettingRowEvent::Edited { value: SettingValue::Text(value), .. }) if value == "two")
    );
}

#[test]
fn choice_label_refresh_updates_presentation_without_resetting_selection() {
    let initial = spec(
        SettingControl::Choice {
            options: vec![
                SettingOption {
                    value: "one".into(),
                    label: "One".into(),
                },
                SettingOption {
                    value: "two".into(),
                    label: "Two".into(),
                },
            ],
        },
        SettingValue::Text("one".into()),
    );
    let (dom, mut runner) = runner(initial);
    let session = runner.state().session.as_ref().unwrap().clone();
    let second = find_attr(&dom.borrow(), runner.root(), "aria-label", "Two").unwrap();
    runner.dispatch_click(second, crate::PointerClick::at((1.0, 1.0)));

    let mut refreshed = session.snapshot().observed_spec;
    if let SettingControl::Choice { options } = &mut refreshed.control {
        options[0].label = "First option".into();
        options[1].label = "Second option".into();
    }
    refreshed.label = "Renamed group".into();
    session.observe(refreshed);
    runner.update(|_| {});

    let second = find_attr(&dom.borrow(), runner.root(), "aria-label", "Second option").unwrap();
    assert_eq!(
        dom.borrow().attribute(
            second,
            &Namespace::from(""),
            &LocalName::from("aria-checked")
        ),
        Some("true")
    );
    let group = find_attr(&dom.borrow(), runner.root(), "role", "radiogroup").unwrap();
    assert_eq!(
        dom.borrow()
            .attribute(group, &Namespace::from(""), &LocalName::from("aria-label")),
        Some("Renamed group")
    );
    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("two".into())
    );
    assert_eq!(session.snapshot().phase, crate::SettingRowPhase::Dirty);
}

#[test]
fn unavailable_dirty_row_exposes_explicit_discard_to_adopt_new_control() {
    let (dom, mut runner) = runner(spec(
        SettingControl::Text,
        SettingValue::Text("original".into()),
    ));
    let session = runner.state().session.as_ref().unwrap().clone();
    let input = find_attr(&dom.borrow(), runner.root(), "role", "textbox").unwrap();
    runner.set_focus(Some(input));
    runner.dispatch_key(KeyEvent::new(Key::Character("x".into())));

    let next = spec(SettingControl::Toggle, SettingValue::Boolean(false));
    assert_eq!(
        session.observe(next),
        crate::SettingRowObservation::IncompatibleDraft
    );
    session.set_unavailable("source changed");
    runner.update(|_| {});
    let discard = dom
        .borrow()
        .all_with_class(runner.root(), "setting-discard")[0];
    assert_eq!(
        dom.borrow()
            .attribute(discard, &Namespace::from(""), &LocalName::from("class")),
        Some("setting-discard")
    );
    assert_eq!(
        dom.borrow().attribute(
            discard,
            &Namespace::from(""),
            &LocalName::from("data-setting")
        ),
        Some("managed.control")
    );
    runner.dispatch_click(discard, crate::PointerClick::at((1.0, 1.0)));

    assert_eq!(session.snapshot().phase, crate::SettingRowPhase::Clean);
    assert_eq!(session.snapshot().draft_value, SettingValue::Boolean(false));
    assert!(find_attr(&dom.borrow(), runner.root(), "role", "switch").is_some());
    assert!(matches!(
        runner.state().events.last(),
        Some(SettingRowEvent::Discard)
    ));
}

#[test]
fn prepared_ime_commit_updates_managed_session_through_retained_text_route() {
    let (dom, mut runner) = runner(spec(
        SettingControl::Text,
        SettingValue::Text("before".into()),
    ));
    let row_root = runner.root();
    let session = runner.state().session.as_ref().unwrap().clone();
    let textbox = find_attr(&dom.borrow(), row_root, "role", "textbox").unwrap();
    runner.set_focus(Some(textbox));
    runner.dispatch_key(KeyEvent::new(Key::Composition(CompositionEvent::Preedit {
        text: "漢".into(),
        selection: None,
    })));

    assert_eq!(session.snapshot().phase, crate::SettingRowPhase::Clean);
    runner.prepare_subtree_move(row_root);

    assert_eq!(
        session.snapshot().draft_value,
        SettingValue::Text("before漢".into())
    );
    assert_eq!(session.snapshot().phase, crate::SettingRowPhase::Dirty);
    assert_eq!(session.snapshot().edit_generation, 1);
    assert!(matches!(
        runner.state().events.last(),
        Some(SettingRowEvent::Edited {
            value: SettingValue::Text(value),
            edit_generation: 1,
        }) if value == "before漢"
    ));
}
