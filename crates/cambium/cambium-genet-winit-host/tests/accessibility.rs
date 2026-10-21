// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The accessibility seam: what a screen reader is told, what happens when it
//! acts, and that it can wake an idle application at all.
//!
//! The OS adapter is the one piece a test cannot supply, so it is the one piece
//! not asserted here. Everything on either side of it is: the projection the
//! adapter is handed, the mapping from a raw AccessKit request to a typed one,
//! the routing of that request into the retained DOM, and the wake callback the
//! adapter is given.

use accesskit::{Action, ActionRequest, Role};
use cambium::{
    AnyView, GenetCtx, GenetElement, RangeScrubber, RangeScrubberEvent, clickable, el, focusable,
    range_scrubber, text,
};
use cambium_genet_winit_host::{Harness, IdlePolicy};
use cambium_winit_a11y::{A11yAction, A11yHost};

#[derive(Default)]
struct App {
    activated: Vec<&'static str>,
    value: f64,
}

#[test]
fn readonly_tests_harness_observation_matches_native_tree_through_shared_reference() {
    let mut harness = Harness::new(
        "button {display:block;width:120px;height:40px}",
        App::default(),
        root as fn(&App) -> Child,
    );
    harness.layout_at(320.0, 240.0);
    let shared = &harness;
    let observed = shared.semantic_projection();
    let native = shared.a11y_tree();
    assert_eq!(observed.tree, native.0);
    assert_eq!(observed.dom_targets, native.1);
    assert!(observed.producer_issues.is_empty());
}

type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;

fn place(x: i32, y: i32, w: i32, h: i32) -> String {
    format!("position:absolute;left:{x}px;top:{y}px;width:{w}px;height:{h}px;")
}

fn root(state: &App) -> Child {
    Box::new(
        el(
            "div",
            (
                focusable(clickable(
                    el("button", text("Install")).attr("style", place(0, 0, 120, 40)),
                    |s: &mut App, _| s.activated.push("install"),
                )),
                focusable(clickable(
                    el("button", text("Cancel")).attr("style", place(0, 50, 120, 40)),
                    |s: &mut App, _| s.activated.push("cancel"),
                )),
                el(
                    "div",
                    range_scrubber(
                        RangeScrubber::new(0.0, 10.0, state.value)
                            .with_steps(1.0, 5.0)
                            .with_label("Strength"),
                        |state: &mut App, event| {
                            let (RangeScrubberEvent::Preview(value)
                            | RangeScrubberEvent::Commit(value)) = event;
                            state.value = value;
                        },
                    ),
                )
                .attr("style", place(0, 100, 200, 40)),
            ),
        )
        .attr("role", "main")
        .attr("style", "position:relative;width:300px;height:200px;"),
    )
}

fn harness() -> Harness<App, fn(&App) -> Child, Child> {
    let mut h = Harness::new("", App::default(), root as fn(&App) -> Child);
    h.layout_at(300.0, 200.0);
    h
}

/// Find the projected node for a control by its accessible name.
fn node_named<'a>(
    tree: &'a accesskit::TreeUpdate,
    name: &str,
) -> Option<(accesskit::NodeId, &'a accesskit::Node)> {
    tree.nodes
        .iter()
        .find(|(_, node)| node.label().is_some_and(|label| label == name))
        .map(|(id, node)| (*id, node))
}

/// The projection carries the roles and names a reader announces, and a box for
/// each control so a virtual cursor can land on it.
#[test]
fn the_projection_carries_roles_names_and_boxes() {
    let mut h = harness();
    let (tree, map) = h.a11y_tree();

    let (install_id, install) = node_named(&tree, "Install").expect("the Install button projects");
    assert_eq!(install.role(), Role::Button, "and announces as a button");
    let dom_node = *map
        .get(&install_id)
        .expect("the node maps back to its DOM element, or an action could not route");
    let bounds = install
        .bounds()
        .expect("a control needs a box to be reachable");
    let painted = h
        .painted_rect(dom_node)
        .expect("the same control has a laid-out box");
    // The strong form of the claim: the box a screen reader is told about is
    // the box the pointer hits. A projection that drifts from layout puts the
    // reader's cursor somewhere the mouse cannot go.
    assert_eq!(
        (
            bounds.x0 as f32,
            bounds.y0 as f32,
            (bounds.x1 - bounds.x0) as f32,
            (bounds.y1 - bounds.y0) as f32
        ),
        painted,
        "the announced box and the hit box are the same box",
    );

    assert!(
        node_named(&tree, "Cancel").is_some(),
        "every control projects, not just the first",
    );
}

/// A Click request activates the control — the same path a mouse press takes.
#[test]
fn a_click_request_activates_the_control() {
    let mut h = harness();
    let (tree, _) = h.a11y_tree();
    let (install, _) = node_named(&tree, "Install").expect("Install projects");
    let dom_node = h
        .a11y_dom_node(install)
        .expect("the projected node resolves to a DOM node");

    assert_eq!(
        h.a11y_request(A11yAction::Click, dom_node),
        cambium_rootstock::A11yDispatch::Dispatched
    );
    assert_eq!(h.state().activated, vec!["install"]);
}

/// A Focus request moves focus and does **not** activate. This is the whole
/// reason the two actions stay typed: a reader's virtual cursor issues Focus as
/// it moves across controls, and collapsing that into a click would fire every
/// one it passed.
#[test]
fn a_focus_request_focuses_without_activating() {
    let mut h = harness();
    let (tree, _) = h.a11y_tree();
    let (cancel, _) = node_named(&tree, "Cancel").expect("Cancel projects");
    let dom_node = h
        .a11y_dom_node(cancel)
        .expect("Cancel resolves to a DOM node");

    assert_eq!(
        h.a11y_request(A11yAction::Focus, dom_node),
        cambium_rootstock::A11yDispatch::Dispatched
    );
    assert_eq!(
        h.state().activated,
        Vec::<&str>::new(),
        "moving the reader's cursor must not press the button",
    );
    assert_eq!(h.focus(), Some(dom_node), "but focus did move there");

    // And the projection now reports that focus, so the reader is told where
    // it is rather than being left on the root.
    let (tree, _) = h.a11y_tree();
    let (cancel_id, _) = node_named(&tree, "Cancel").expect("Cancel still projects");
    assert_eq!(tree.focus, cancel_id);
}

/// A numeric request follows the host route into the same controlled value
/// handler used by the scrubber's pointer and keyboard input.
#[test]
fn a_set_value_request_updates_a_numeric_control() {
    let mut h = harness();
    let (tree, _) = h.a11y_tree();
    let (strength, node) = node_named(&tree, "Strength").expect("Strength projects");
    assert!(node.supports_action(Action::SetValue));
    let dom_node = h
        .a11y_dom_node(strength)
        .expect("Strength resolves to a DOM node");

    assert_eq!(
        h.a11y_set_value(dom_node, 7.4),
        cambium_rootstock::A11yDispatch::Dispatched
    );
    assert_eq!(h.state().value, 7.0, "the shared step policy quantizes");
}

/// The raw-request mapping keeps the two actions apart and drops the ones this
/// host does not route, rather than guessing at them.
#[test]
fn raw_requests_map_to_their_own_actions() {
    let mut host = A11yHost::new(|| {});
    // No tree has been synced, so nothing resolves — including a Click, which
    // is the honest answer rather than a click on some default node.
    assert_eq!(
        host.map_request(&ActionRequest {
            action: Action::Click,
            target_tree: accesskit::TreeId::ROOT,
            target_node: accesskit::NodeId(1),
            data: None,
        }),
        None,
    );
    let _ = &mut host;
}

/// A screen reader acting on an idle application must be able to wake it. The
/// callback under test is the exact one the host hands the adapter.
#[test]
fn a_screen_reader_wakes_an_idle_application() {
    let mut h = harness();
    assert_eq!(
        h.idle_policy(),
        IdlePolicy::Wait,
        "an idle app with nothing pending sleeps",
    );

    h.signal_a11y_wake();
    assert_eq!(
        h.idle_policy(),
        IdlePolicy::A11yWake,
        "the adapter's wake callback turns into a repaint, so the queued action drains",
    );
    assert_eq!(
        h.idle_policy(),
        IdlePolicy::Wait,
        "and the wake is consumed once, not re-triggered every idle turn",
    );
}

#[test]
fn native_harness_exposes_current_target_refusal() {
    use cambium_rootstock::{A11yDispatch, A11yRejection};
    let mut h = harness();
    let (tree, _) = h.a11y_tree();
    let (id, _) = node_named(&tree, "Install").unwrap();
    let node = h.a11y_dom_node(id).unwrap();
    assert_eq!(
        h.a11y_set_value(node, 2.0),
        A11yDispatch::Rejected(A11yRejection::Unsupported)
    );
    assert_eq!(
        h.a11y_set_value(node, f64::NAN),
        A11yDispatch::Rejected(A11yRejection::InvalidValue)
    );
    assert!(h.state().activated.is_empty());
}

#[test]
fn native_and_neutral_projections_share_the_hosts_scrolled_bounds() {
    let mut h = harness();
    h.layout_at(300.0, 100.0);
    h.wheel(0.0, 20.0);
    assert!(h.viewport_scroll().1 > 0.0);
    let neutral = h.a11y_projection();
    let (native, _) = h.a11y_tree();
    let semantic = neutral
        .nodes()
        .iter()
        .find(|node| node.name.as_deref() == Some("Cancel"))
        .unwrap();
    let (_, platform) = node_named(&native, "Cancel").unwrap();
    let bounds = semantic.bounds.unwrap();
    let native = platform.bounds().unwrap();
    assert_eq!(
        (
            native.x0,
            native.y0,
            native.x1 - native.x0,
            native.y1 - native.y0
        ),
        (
            f64::from(bounds.x),
            f64::from(bounds.y),
            f64::from(bounds.width),
            f64::from(bounds.height)
        )
    );
}

#[test]
fn selector_tests_harness_uses_leaf_roles_and_real_generated_action_admission() {
    use cambium_rootstock::{
        HostHooks, Init, ProducedTexture, ProducerAction, ProducerContext, ProducerNode,
        ProducerRole, ProducerSemantics, TextureProducer,
    };
    use std::{cell::Cell, rc::Rc};
    struct Board {
        offered: Rc<Cell<bool>>,
        accepted: Rc<Cell<bool>>,
        calls: Rc<Cell<u32>>,
    }
    impl TextureProducer for Board {
        fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
            None
        }
        fn semantics(&self) -> Option<ProducerSemantics> {
            Some(ProducerSemantics {
                children: vec![ProducerNode {
                    key: 41,
                    role: ProducerRole::ListItem,
                    name: "Card".into(),
                    rect: [0., 0., 40., 20.],
                    actions: if self.offered.get() {
                        vec![ProducerAction {
                            id: "pin".into(),
                            label: "Pin selected".into(),
                            description: "Pin".into(),
                        }]
                    } else {
                        vec![]
                    },
                }],
                ..Default::default()
            })
        }
        fn act(&mut self, key: u64, id: &str) -> bool {
            assert_eq!((key, id), (41, "pin"));
            self.calls.set(self.calls.get() + 1);
            self.accepted.get()
        }
    }
    fn view(state: &App) -> Child {
        Box::new(el(
            "main",
            (
                clickable(
                    el("button", "Pin selected in workbench").attr("data-key", "native"),
                    |s: &mut App, _| s.activated.push("native"),
                ),
                cambium::custom_leaf::<App, ()>(7, 80, 30)
                    .attr("aria-hidden", (state.value > 0.).to_string()),
                cambium::custom_leaf::<App, ()>(8, 80, 30),
            ),
        ))
    }
    let calls = Rc::new(Cell::new(0));
    let offered = Rc::new(Cell::new(true));
    let accepted = Rc::new(Cell::new(true));
    let mut hooks: HostHooks<App, fn(&App) -> Child, Child> =
        cambium_genet_winit_host::inert_hooks();
    let (c, o, a) = (calls.clone(), offered.clone(), accepted.clone());
    hooks.frame = Box::new(move |ctx| {
        if !ctx.producers.contains(7) {
            ctx.producers
                .register(
                    7,
                    Board {
                        calls: c.clone(),
                        offered: o.clone(),
                        accepted: a.clone(),
                    },
                    &[],
                )
                .unwrap();
        }
        if ctx.leaves.get(&8).is_none() {
            ctx.leaves.insert(
                8,
                Box::new(sprigging::Knob::new(sprigging::Size {
                    width: 80.,
                    height: 30.,
                })),
            );
        }
        false
    });
    let mut h = Harness::with_hooks(
        Init {
            state: App::default(),
            logic: view as fn(&App) -> Child,
            sheet: "main,button,custom-leaf{display:block;width:100px;height:30px}".into(),
            fonts: vec![],
            images: vec![],
        },
        hooks,
    );
    h.prepare_frame();
    h.layout_at(320., 240.);
    assert!(
        h.resolve(&taproot::Selector::role("slider")).is_some(),
        "leaf-only role is selectable"
    );
    let sel = taproot::Selector::role("button").containing("Pin selected");
    assert!(
        h.resolve(&sel).is_none(),
        "virtual actions have no fabricated pointer point"
    );
    assert!(
        !h.right_click_on(&sel),
        "generated action buttons have no secondary pointer route"
    );
    assert!(h.click_on(&sel));
    assert_eq!(calls.get(), 1);
    assert!(h.state().activated.is_empty());
    accepted.set(false);
    assert!(!h.click_on(&sel));
    assert_eq!(calls.get(), 2);
    assert!(h.click_on(&sel.clone().with_attr("data-key", "native")));
    assert_eq!(h.state().activated, ["native"]);
    offered.set(false);
    assert!(
        h.resolve(
            &taproot::Selector::role("button")
                .containing("Pin selected")
                .with_attr("missing", "none")
        )
        .is_none()
    );
    // The remaining prefix is the real DOM button, not a stale generated route.
    assert!(h.click_on(&sel));
    assert_eq!(calls.get(), 2);
}
