/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::{cell::RefCell, rc::Rc};

use genet_scripted_dom::ScriptedDom;
use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut, LocalName, Namespace};
use meristem::ViewPathTracker;

use super::*;
use crate::{
    Action, CommandEvent, CommandItem, CommandState, DomHandle, GenetAppRunner, Key, KeyEvent,
    NamedKey, command_picker,
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct PickerProps {
    label: String,
    items: Vec<CommandItem>,
}

#[derive(Debug)]
struct AppState {
    label: String,
    activated: Vec<Vec<usize>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum AppEvent {
    Activated(Vec<usize>),
}

impl Action for AppEvent {}

fn items() -> Vec<CommandItem> {
    vec![
        CommandItem::new("Open"),
        CommandItem::new("Export"),
        CommandItem::new("Close").disabled_because("Keep one document open"),
    ]
}

fn init_picker(props: &PickerProps) -> CommandState {
    CommandState::default()
        .with_id("component-picker")
        .with_label(props.label.clone())
}

fn reconcile_picker(prev: &PickerProps, next: &PickerProps, local: &mut CommandState) {
    if prev.label != next.label {
        local.label.clone_from(&next.label);
    }
}

fn picker_body(
    props: &PickerProps,
    local: &CommandState,
) -> ComponentView<CommandState, CommandEvent> {
    Box::new(command_picker(local, &props.items))
}

fn lower_picker_event(app: &mut AppState, event: CommandEvent) -> Option<AppEvent> {
    match event {
        CommandEvent::Activate(path) => {
            app.activated.push(path.clone());
            Some(AppEvent::Activated(path))
        },
        CommandEvent::Dismiss => None,
    }
}

fn app_view(
    app: &AppState,
) -> impl View<AppState, AppEvent, GenetCtx, Element = GenetElement> + use<> {
    component(
        PickerProps {
            label: app.label.clone(),
            items: items(),
        },
        init_picker,
        reconcile_picker,
        picker_body,
        lower_picker_event,
    )
    .probe_id("primary-picker")
}

fn attr<'a>(dom: &'a ScriptedDom, node: genet_scripted_dom::NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
}

#[test]
fn component_retains_local_state_reconciles_props_and_emits_parent_events() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        app_view,
        AppState {
            label: "Actions".into(),
            activated: Vec::new(),
        },
    );
    let root = runner.root();
    assert_eq!(
        attr(&dom.borrow(), root, COMPONENT_PROBE_ATTR),
        Some("primary-picker")
    );

    runner.set_focus(Some(root));
    runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::ArrowDown)));
    assert_eq!(
        attr(&dom.borrow(), root, "aria-activedescendant"),
        Some("component-picker-item-1"),
        "component-owned selection survives the dispatch rebuild"
    );

    runner.update(|app| app.label = "Commands".into());
    assert_eq!(attr(&dom.borrow(), root, "aria-label"), Some("Commands"));
    assert_eq!(
        attr(&dom.borrow(), root, "aria-activedescendant"),
        Some("component-picker-item-1"),
        "reconciling a controlled label must not reset local selection"
    );

    let events = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Enter)));
    assert_eq!(events, [AppEvent::Activated(vec![1])]);
    assert_eq!(runner.state().activated, [vec![1]]);

    let dismissed = runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape)));
    assert!(
        dismissed.is_empty(),
        "the component may swallow a child event without minting a parent action"
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbeAction;
impl Action for ProbeAction {}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ProbeEvent;

#[derive(Debug)]
struct ProbeApp {
    probe_id: Option<String>,
}

fn probe_view(
    app: &ProbeApp,
) -> impl View<ProbeApp, ProbeAction, GenetCtx, Element = GenetElement> + use<> {
    let view = component(
        (),
        |_: &()| (),
        |_: &(), _: &(), _: &mut ()| {},
        |_: &(), _: &()| -> ComponentView<(), ProbeEvent> {
            Box::new(crate::el::<_, (), ProbeEvent>("div", "probe"))
        },
        |_: &mut ProbeApp, _: ProbeEvent| {},
    );
    match &app.probe_id {
        Some(id) => view.probe_id(id.clone()),
        None => view,
    }
}

fn drain(dom: &DomHandle) -> Vec<DomMutation<genet_scripted_dom::NodeId>> {
    let mut mutations = Vec::new();
    dom.borrow_mut().drain_mutations(&mut mutations);
    mutations
}

fn probe_old_values(mutations: &[DomMutation<genet_scripted_dom::NodeId>]) -> Vec<Option<&str>> {
    mutations
        .iter()
        .filter_map(|mutation| match mutation {
            DomMutation::AttributeChanged {
                name, old_value, ..
            } if AsRef::<str>::as_ref(&name.local) == COMPONENT_PROBE_ATTR => {
                Some(old_value.as_deref())
            },
            _ => None,
        })
        .collect()
}

#[test]
fn probe_stamp_mutates_only_when_its_value_changes() {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        probe_view,
        ProbeApp {
            probe_id: Some("primary".into()),
        },
    );
    let root = runner.root();
    assert_eq!(probe_old_values(&drain(&dom)), [None]);

    runner.update(|_| {});
    assert!(
        probe_old_values(&drain(&dom)).is_empty(),
        "an unchanged probe stamp must not enqueue a restyle mutation"
    );

    runner.update(|app| app.probe_id = Some("secondary".into()));
    assert_eq!(probe_old_values(&drain(&dom)), [Some("primary")]);
    assert_eq!(
        attr(&dom.borrow(), root, COMPONENT_PROBE_ATTR),
        Some("secondary")
    );

    runner.update(|app| app.probe_id = None);
    assert_eq!(probe_old_values(&drain(&dom)), [Some("secondary")]);
    assert_eq!(attr(&dom.borrow(), root, COMPONENT_PROBE_ATTR), None);
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Bump;
impl Action for Bump {}

#[test]
fn memo_skips_body_on_equal_props_until_a_message_dirties_local_state() {
    use std::cell::Cell;

    use crate::{PointerClick, button};

    let body_runs = Rc::new(Cell::new(0_u32));
    let seen = body_runs.clone();
    let app_view = move |step: &i64| {
        let seen = seen.clone();
        component(
            *step,
            |_: &i64| 0_i64,
            |_: &i64, _: &i64, _: &mut i64| {},
            move |_: &i64, _: &i64| -> ComponentView<i64, Bump> {
                seen.set(seen.get() + 1);
                Box::new(button("+", |count: &mut i64, _: PointerClick| {
                    *count += 1;
                }))
            },
            |_: &mut i64, _: Bump| {},
        )
        .memo()
    };

    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), app_view, 1_i64);
    assert_eq!(body_runs.get(), 1, "build runs the body once");

    runner.update(|_| {});
    assert_eq!(
        body_runs.get(),
        1,
        "equal props and clean local skip the body"
    );

    runner.update(|step| *step += 1);
    assert_eq!(body_runs.get(), 2, "changed props run the body");

    let root = runner.root();
    assert!(
        runner
            .dispatch_click(root, PointerClick::at((2.0, 2.0)))
            .is_empty(),
        "a unit-returning component mapper yields no parent action"
    );
    assert_eq!(
        body_runs.get(),
        3,
        "a message marks local dirty, so the dispatch rebuild runs the body"
    );

    runner.update(|_| {});
    assert_eq!(
        body_runs.get(),
        3,
        "the skip resumes once local is clean again"
    );
}

#[test]
fn route_refresh_traverses_retained_component_without_running_body() {
    use std::cell::Cell;

    let body_runs = Rc::new(Cell::new(0_u32));
    let seen = body_runs.clone();
    let view = move |_: &ProbeApp| {
        let seen = seen.clone();
        component(
            (),
            |_: &()| (),
            |_: &(), _: &(), _: &mut ()| {},
            move |_: &(), _: &()| -> ComponentView<(), ProbeEvent> {
                seen.set(seen.get() + 1);
                Box::new(crate::el::<_, (), ProbeEvent>("div", "retained"))
            },
            |_: &mut ProbeApp, _: ProbeEvent| {},
        )
    };
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom, view, ProbeApp { probe_id: None });
    assert_eq!(body_runs.get(), 1);

    runner.refresh_routes();

    assert_eq!(
        body_runs.get(),
        1,
        "route refresh must retain component body state"
    );
}

struct PathScope<V> {
    route: u64,
    child: V,
}

struct PathScopeState<S> {
    child: S,
}

impl<V> ViewMarker for PathScope<V> {}

impl<State: 'static, Action: 'static, V> View<State, Action, GenetCtx> for PathScope<V>
where
    V: View<State, Action, GenetCtx, Element = GenetElement>,
{
    type Element = GenetElement;
    type ViewState = PathScopeState<V::ViewState>;

    fn build(&self, ctx: &mut GenetCtx, state: &mut State) -> (GenetElement, Self::ViewState) {
        ctx.push_id(meristem::ViewId::new(self.route));
        let (element, child) = self.child.build(ctx, state);
        ctx.pop_id();
        (element, PathScopeState { child })
    }

    fn rebuild(
        &self,
        prev: &Self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
        app_state: &mut State,
    ) {
        ctx.push_id(meristem::ViewId::new(self.route));
        self.child
            .rebuild(&prev.child, &mut state.child, ctx, element, app_state);
        ctx.pop_id();
    }

    fn teardown(
        &self,
        state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, GenetElement>,
    ) {
        self.child.teardown(&mut state.child, ctx, element);
    }

    fn message(
        &self,
        state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, GenetElement>,
        app_state: &mut State,
    ) -> MessageResult<Action> {
        self.child
            .message(&mut state.child, message, element, app_state)
    }
}

#[test]
fn dirty_memo_component_renders_when_parent_route_changes_in_same_dispatch() {
    use std::cell::Cell;

    let body_runs = Rc::new(Cell::new(0_u32));
    let seen = body_runs.clone();
    let logic = move |app: &ProbeApp| {
        let seen = seen.clone();
        PathScope {
            route: if app.probe_id.is_some() { 1 } else { 0 },
            child: component::<(), i64, Bump, ProbeApp, Bump, (), _, _, _, _>(
                (),
                |_: &()| 0_i64,
                |_: &(), _: &(), _: &mut i64| {},
                move |_: &(), _count: &i64| -> ComponentView<i64, Bump> {
                    seen.set(seen.get() + 1);
                    Box::new(crate::button(
                        "increment",
                        move |count: &mut i64, _: crate::PointerClick| {
                            *count += 1;
                            Bump
                        },
                    ))
                },
                |app: &mut ProbeApp, _: Bump| app.probe_id = Some("changed-route".into()),
            )
            .memo(),
        }
    };
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(dom, logic, ProbeApp { probe_id: None });
    let root = runner.root();
    assert_eq!(body_runs.get(), 1);

    runner.dispatch_click(root, crate::PointerClick::at((0.0, 0.0)));

    assert_eq!(
        body_runs.get(),
        2,
        "dirty local state must render despite route change"
    );
}
