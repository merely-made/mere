/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Experiment E1a, the session half: two retained Cambium sessions, each with
//! its own state type and document, laid out and projected on their own, then
//! joined under one host as AccessKit subtrees (app composition brief §8 and
//! AC3, ruled 2026-10-07).
//!
//! What the tree half (`uxtree/tests/e1a_subtrees.rs`) shows with model
//! guests, this shows with real sessions behind the seam a host uses:
//! `ContainedSession` around `RunnerSurfaceSession`. Tab past A's last
//! focusable lands on B's first through `SurfaceEffect::FocusExit`; a Click
//! routed by tree reaches B's button through B's own id map; B's bounds come
//! from B's own layout, offset by its pane; a panic in B's dispatch retires B
//! and its graft while A and the host carry on; and two sessions that place a
//! leaf under one key each keep their own leaf. Each check runs against
//! `accesskit_consumer` 0.35, 0.36 and 0.38, the consumers under the Windows,
//! AT-SPI and macOS adapters.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use accesskit::TreeUpdate;
use accesskit::{Action, ActionRequest, Node, NodeId as A11yNodeId, Rect, Role, Tree, TreeId};
use cambium::{
    AnyView, ContainedSession, FocusExit, GenetAppRunner, GenetCtx, GenetElement, Key, KeyEvent,
    NamedKey, PointerClick, ResolvedSurfaceEvent, RetainedSurfaceSession, RunnerSurfaceSession,
    SurfaceEffect, clickable, custom_leaf, el, focusable, on_key, text,
};
use cambium_rootstock::{OwnedLayout, ProducerRegistry};
use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::LayoutDom;
use mere_surface_api::{
    ProviderId, SourceKindId, SurfaceAvailability, SurfaceDescriptor, SurfaceId,
    SurfaceSourceShape, SurfaceUnavailableReason,
};
use sprigging::{Leaf, LeafRegistry, PaintCx, Size, SizeHint};
use uxtree::{ActionTarget, Composition, GraftTable, Grafts, graft_node, tree_id_for_path};

const LEAF_KEY: u64 = 0x5753_4642;
const PANE: (f32, f32) = (200.0, 120.0);
const A_ORIGIN: (f64, f64) = (0.0, 40.0);
const B_ORIGIN: (f64, f64) = (200.0, 40.0);
const A_GRAFT: A11yNodeId = A11yNodeId(10);
const B_GRAFT: A11yNodeId = A11yNodeId(11);

fn place(x: i32, y: i32, w: i32, h: i32) -> String {
    format!("position:absolute;left:{x}px;top:{y}px;width:{w}px;height:{h}px;")
}

/// Guest A's state: one editable field and a leaf.
#[derive(Default)]
struct Notes {
    typed: usize,
}

/// Guest B's state: a counter, a button that panics, and a leaf.
#[derive(Default)]
struct Counter {
    count: u32,
}

type NotesView = Box<dyn AnyView<Notes, (), GenetCtx, GenetElement>>;
type CounterView = Box<dyn AnyView<Counter, (), GenetCtx, GenetElement>>;

fn notes_view(_state: &Notes) -> NotesView {
    let typing: fn(&mut Notes, KeyEvent) = |state, _| state.typed += 1;
    Box::new(
        el(
            "div",
            (
                on_key(
                    el("input", ())
                        .attr("aria-label", "A field")
                        .attr("style", place(10, 10, 180, 30)),
                    typing,
                ),
                el("div", custom_leaf::<Notes, ()>(LEAF_KEY, 40, 20))
                    .attr("style", place(10, 60, 40, 20)),
            ),
        )
        .attr("role", "main")
        .attr("aria-label", "A document")
        .attr("style", "position:relative;width:200px;height:120px;"),
    )
}

/// B's second button: a guest bug, for the containment check.
fn explode(_: &mut Counter, _: PointerClick) {
    panic!("guest bug");
}

fn counter_view(state: &Counter) -> CounterView {
    Box::new(
        el(
            "div",
            (
                focusable(clickable(
                    el("button", text(format!("B count {}", state.count)))
                        .attr("style", place(10, 10, 80, 20)),
                    |state: &mut Counter, _| state.count += 1,
                )),
                focusable(clickable(
                    el("button", text("B explode")).attr("style", place(100, 10, 80, 20)),
                    explode,
                )),
                el("div", custom_leaf::<Counter, ()>(LEAF_KEY, 40, 20))
                    .attr("style", place(10, 60, 40, 20)),
            ),
        )
        .attr("role", "list")
        .attr("aria-label", "B list")
        .attr("style", "position:relative;width:200px;height:120px;"),
    )
}

/// A leaf that names itself, so the projection shows whose leaf it is.
struct Named(&'static str);

impl Leaf for Named {
    fn measure(&mut self, _known: SizeHint, _available: SizeHint) -> Size {
        Size {
            width: 40.0,
            height: 20.0,
        }
    }

    fn paint(&mut self, _cx: &mut PaintCx<'_>) {}

    fn accessibility(&mut self, node: &mut Node) {
        node.set_label(self.0);
    }

    fn paint_dirty(&self) -> bool {
        false
    }
}

fn leaves(name: &'static str) -> LeafRegistry<u64> {
    let mut leaves = LeafRegistry::new();
    leaves.insert(LEAF_KEY, Box::new(Named(name)));
    leaves
}

fn descriptor(id: &str) -> SurfaceDescriptor {
    SurfaceDescriptor {
        provider_id: ProviderId::from("e1a"),
        surface_id: SurfaceId::from(id),
        label: id.to_owned(),
        accepted_source: SurfaceSourceShape::One(SourceKindId::from("e1a.source")),
    }
}

fn session_a() -> Box<dyn RetainedSurfaceSession> {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom, notes_view, Notes::default());
    let mut session = ContainedSession::new(
        RunnerSurfaceSession::new(
            descriptor("e1a.a"),
            runner,
            |_: &Notes| SurfaceAvailability::Available,
            |_: &mut Notes, _| {},
            |_action: ()| Vec::new(),
        )
        .with_leaves(leaves("A leaf")),
    );
    assert!(session.set_focus_exits(true));
    Box::new(session)
}

fn session_b() -> Box<dyn RetainedSurfaceSession> {
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(dom, counter_view, Counter::default());
    let mut session = ContainedSession::new(
        RunnerSurfaceSession::new(
            descriptor("e1a.b"),
            runner,
            |_: &Counter| SurfaceAvailability::Available,
            |_: &mut Counter, _| {},
            |_action: ()| Vec::new(),
        )
        .with_leaves(leaves("B leaf")),
    );
    assert!(session.set_focus_exits(true));
    Box::new(session)
}

/// One guest's projection this frame: its own tree, and its own map from
/// AccessKit id back to its DOM.
struct Projected {
    update: TreeUpdate,
    nodes: HashMap<A11yNodeId, NodeId>,
}

/// Lay out a session's own document and project it, against its own leaves
/// (or against `shared`, for the control).
fn project(
    session: &mut dyn RetainedSurfaceSession,
    shared: Option<&mut LeafRegistry<u64>>,
) -> Projected {
    let dom = session.dom();
    let focus = session.focus();
    let dom = dom.borrow();
    let layout = OwnedLayout::new(&*dom, &[""], PANE.0, PANE.1, &[], &HashMap::new());
    let focus = focus.map(|node| dom.opaque_id(node));
    let mut empty = LeafRegistry::new();
    let leaves = match shared {
        Some(shared) => shared,
        None => session.leaves().unwrap_or(&mut empty),
    };
    let (update, nodes) = cambium_winit_a11y::project_tree(
        &*dom,
        &layout,
        leaves,
        &mut ProducerRegistry::new(),
        focus,
    );
    Projected { update, nodes }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pane {
    A,
    B,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Host,
    Pane(Pane),
}

struct Host {
    a: Box<dyn RetainedSurfaceSession>,
    b: Option<Box<dyn RetainedSurfaceSession>>,
    focus: Focus,
    a_tree: TreeId,
    b_tree: TreeId,
    table: GraftTable<Pane>,
    /// The last projection of each guest, for routing actions to DOM nodes.
    a_nodes: HashMap<A11yNodeId, NodeId>,
    b_nodes: HashMap<A11yNodeId, NodeId>,
}

impl Host {
    fn new() -> Self {
        let (a_tree, b_tree) = (tree_id_for_path("e1a/a#1"), tree_id_for_path("e1a/b#1"));
        let mut table = GraftTable::new(TreeId::ROOT);
        table.insert(a_tree, Pane::A);
        table.insert(b_tree, Pane::B);
        Self {
            a: session_a(),
            b: Some(session_b()),
            focus: Focus::Host,
            a_tree,
            b_tree,
            table,
            a_nodes: HashMap::new(),
            b_nodes: HashMap::new(),
        }
    }

    fn host_update(&self) -> TreeUpdate {
        let mut children = vec![A11yNodeId(2), A_GRAFT];
        if self.b.is_some() {
            children.push(B_GRAFT);
        }
        let mut window = Node::new(Role::Window);
        window.set_label("host");
        window.set_bounds(Rect::new(0.0, 0.0, 400.0, 160.0));
        window.set_children(children);
        let mut menu = Node::new(Role::Button);
        menu.set_label("Host menu");
        menu.set_bounds(Rect::new(0.0, 0.0, 80.0, 30.0));
        let pane = (f64::from(PANE.0), f64::from(PANE.1));
        let mut nodes = vec![
            (A11yNodeId(1), window),
            (A11yNodeId(2), menu),
            (A_GRAFT, graft_node(Role::Pane, self.a_tree, A_ORIGIN, pane)),
        ];
        if self.b.is_some() {
            nodes.push((B_GRAFT, graft_node(Role::Pane, self.b_tree, B_ORIGIN, pane)));
        }
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(A11yNodeId(1))),
            tree_id: TreeId::ROOT,
            focus: match self.focus {
                Focus::Host => A11yNodeId(1),
                Focus::Pane(Pane::A) => A_GRAFT,
                Focus::Pane(Pane::B) => B_GRAFT,
            },
        }
    }

    /// Retire B once its session reports itself unavailable.
    fn reap(&mut self) {
        let unhealthy = self
            .b
            .as_ref()
            .is_some_and(|b| !b.availability().is_available());
        if unhealthy {
            self.b = None;
            self.table.remove(self.b_tree);
            self.b_nodes.clear();
            if self.focus == Focus::Pane(Pane::B) {
                self.focus = Focus::Host;
            }
        }
    }

    fn composition(&mut self, shared: Option<&mut LeafRegistry<u64>>) -> Composition {
        self.reap();
        let (a, b) = match shared {
            Some(shared) => {
                let a = project(self.a.as_mut(), Some(&mut *shared));
                let b = self
                    .b
                    .as_mut()
                    .map(|b| project(b.as_mut(), Some(&mut *shared)));
                (a, b)
            },
            None => (
                project(self.a.as_mut(), None),
                self.b.as_mut().map(|b| project(b.as_mut(), None)),
            ),
        };
        self.a_nodes = a.nodes;
        let mut guests = vec![(self.a_tree, a.update)];
        if let Some(b) = b {
            self.b_nodes = b.nodes;
            guests.push((self.b_tree, b.update));
        }
        Composition::new(self.host_update(), guests).expect("the host composes a whole frame")
    }

    /// Apply a guest's effects: a focus exit moves the host to its next stop.
    fn follow(&mut self, from: Pane, effects: Vec<SurfaceEffect>) {
        for effect in effects {
            if let SurfaceEffect::FocusExit(exit) = effect {
                match (from, exit) {
                    (Pane::A, FocusExit::Forward) if self.b.is_some() => {
                        self.focus = Focus::Pane(Pane::B);
                        let b = self.b.as_mut().expect("B is live");
                        b.focus_traverse(true);
                    },
                    _ => self.focus = Focus::Host,
                }
            }
        }
    }

    fn key(&mut self, pane: Pane, key: KeyEvent) {
        let effects = match pane {
            Pane::A => self.a.dispatch(ResolvedSurfaceEvent::Key(key)),
            Pane::B => match &mut self.b {
                Some(b) => b.dispatch(ResolvedSurfaceEvent::Key(key)),
                None => Vec::new(),
            },
        };
        self.follow(pane, effects);
    }

    /// Route a screen reader's Click by tree, through the owning guest's own
    /// id map, as a host's adapter callback would.
    fn act(&mut self, request: &ActionRequest) {
        let (pane, nodes) = match self.table.route(request) {
            ActionTarget::Guest(Pane::A) => (Pane::A, &self.a_nodes),
            ActionTarget::Guest(Pane::B) => (Pane::B, &self.b_nodes),
            ActionTarget::Host | ActionTarget::Unknown(_) => return,
        };
        let Some(&target) = nodes.get(&request.target_node) else {
            return;
        };
        let event = ResolvedSurfaceEvent::Click {
            target,
            event: PointerClick::at((0.0, 0.0)),
        };
        let effects = match pane {
            Pane::A => self.a.dispatch(event),
            Pane::B => match &mut self.b {
                Some(b) => b.dispatch(event),
                None => Vec::new(),
            },
        };
        self.follow(pane, effects);
    }
}

fn tab() -> KeyEvent {
    KeyEvent::new(Key::Named(NamedKey::Tab))
}

macro_rules! e1a_sessions_against {
    ($module:ident, $consumer:ident) => {
        mod $module {
            use super::*;
            use $consumer::{Node as Read, Tree as ConsumerTree, TreeChangeHandler};

            struct Ignore;

            impl TreeChangeHandler for Ignore {
                fn node_added(&mut self, _: &Read) {}
                fn node_updated(&mut self, _: &Read, _: &Read) {}
                fn focus_moved(&mut self, _: Option<&Read>, _: Option<&Read>) {}
                fn node_removed(&mut self, _: &Read) {}
            }

            fn open(host: &mut Host) -> (ConsumerTree, Grafts) {
                let composition = host.composition(None);
                let mut grafts = Grafts::new();
                let mut tree = ConsumerTree::new(grafts.activate(&composition), true);
                for update in grafts.frame(composition) {
                    tree.update_and_process_changes(update, &mut Ignore);
                }
                (tree, grafts)
            }

            fn push(tree: &mut ConsumerTree, grafts: &mut Grafts, host: &mut Host) {
                for update in grafts.frame(host.composition(None)) {
                    tree.update_and_process_changes(update, &mut Ignore);
                }
            }

            fn labels(tree: &ConsumerTree) -> Vec<(String, TreeId)> {
                fn walk(node: Read<'_>, out: &mut Vec<(String, TreeId)>) {
                    if let Some(label) = node.label() {
                        out.push((label, node.locate().1));
                    }
                    for child in node.children() {
                        walk(child, out);
                    }
                }
                let mut out = Vec::new();
                walk(tree.state().root(), &mut out);
                out
            }

            fn find<'a>(tree: &'a ConsumerTree, label: &str) -> Read<'a> {
                fn walk<'a>(node: Read<'a>, label: &str) -> Option<Read<'a>> {
                    if node.label().as_deref() == Some(label) {
                        return Some(node);
                    }
                    node.children().find_map(|child| walk(child, label))
                }
                walk(tree.state().root(), label)
                    .unwrap_or_else(|| panic!("no node labelled {label:?}"))
            }

            fn focus(tree: &ConsumerTree) -> Option<(String, TreeId)> {
                tree.state()
                    .focus()
                    .and_then(|node| node.label().map(|label| (label, node.locate().1)))
            }

            /// Done-condition 1: one tree, in host reading order, holding the
            /// host and both sessions' real projections.
            ///
            /// The brief read that two fresh documents would number their
            /// nodes alike. They do not: Genet's `NodeId` carries a
            /// process-wide arena id, so in-process sessions never share
            /// AccessKit ids. The subtrees here still carry each session's
            /// focus, route its actions and retire it whole, and the tree
            /// half (`uxtree/tests/e1a_subtrees.rs`) covers coinciding ids,
            /// which trees from other sources (path-hashed host nodes, other
            /// processes) do produce.
            #[test]
            fn one_tree_from_two_real_sessions() {
                let mut host = Host::new();
                let (tree, _) = open(&mut host);
                let order: Vec<String> =
                    labels(&tree).into_iter().map(|(label, _)| label).collect();
                let at = |label: &str| {
                    order
                        .iter()
                        .position(|candidate| candidate == label)
                        .unwrap_or_else(|| panic!("{label:?} missing from {order:?}"))
                };
                assert!(at("host") < at("Host menu"));
                assert!(at("Host menu") < at("A field"));
                assert!(at("A field") < at("B count 0"));
                assert!(at("B count 0") < at("B explode"));
            }

            /// Done-condition 2: focus follows the graft chain into A's
            /// field, and Tab past A's last focusable lands on B's first.
            #[test]
            fn tab_hands_focus_from_a_to_b() {
                let mut host = Host::new();
                let field = host.a.focusables()[0];
                host.a.set_focus(Some(field));
                host.focus = Focus::Pane(Pane::A);
                let (mut tree, mut grafts) = open(&mut host);
                assert_eq!(focus(&tree), Some(("A field".into(), host.a_tree)));

                host.key(Pane::A, tab());
                assert_eq!(host.a.focus(), None, "A let go of its focus");
                push(&mut tree, &mut grafts, &mut host);
                assert_eq!(focus(&tree), Some(("B count 0".into(), host.b_tree)));
            }

            /// Done-condition 3: a Click addressed to B's tree reaches B's
            /// button through B's own id map; A is untouched.
            #[test]
            fn a_click_routed_by_tree_reaches_b() {
                let mut host = Host::new();
                let (mut tree, mut grafts) = open(&mut host);
                let (target_node, target_tree) = find(&tree, "B count 0").locate();
                host.act(&ActionRequest {
                    action: Action::Click,
                    target_tree,
                    target_node,
                    data: None,
                });
                push(&mut tree, &mut grafts, &mut host);
                find(&tree, "B count 1");
                assert_eq!(find(&tree, "A field").locate().1, host.a_tree);
            }

            /// Done-condition 4: B's button bounds are its own laid-out
            /// rectangle offset by B's pane origin.
            #[test]
            fn b_bounds_are_its_layout_offset_by_its_pane() {
                let mut host = Host::new();
                let own = project(host.b.as_mut().unwrap().as_mut(), None)
                    .update
                    .nodes
                    .into_iter()
                    .find(|(_, node)| node.label().as_deref() == Some("B count 0"))
                    .and_then(|(_, node)| node.bounds())
                    .expect("B's button has a laid-out box");
                let (tree, _) = open(&mut host);
                let expected = Rect::new(
                    own.x0 + B_ORIGIN.0,
                    own.y0 + B_ORIGIN.1,
                    own.x1 + B_ORIGIN.0,
                    own.y1 + B_ORIGIN.1,
                );
                assert_eq!(find(&tree, "B count 0").bounding_box(), Some(expected));
            }

            /// Done-condition 5: a panic in B's dispatch is caught, B reports
            /// itself unhealthy, the next frame removes B's graft and
            /// subtree, and A and the host carry on.
            #[test]
            fn a_panic_in_b_retires_b_alone() {
                let mut host = Host::new();
                let (mut tree, mut grafts) = open(&mut host);
                let (target_node, target_tree) = find(&tree, "B explode").locate();
                host.act(&ActionRequest {
                    action: Action::Click,
                    target_tree,
                    target_node,
                    data: None,
                });
                assert_eq!(
                    host.b.as_ref().map(|b| b.availability()),
                    Some(SurfaceAvailability::Unavailable(
                        SurfaceUnavailableReason::Unhealthy
                    ))
                );
                push(&mut tree, &mut grafts, &mut host);
                assert!(host.b.is_none(), "the host retired B");
                assert_eq!(tree.state().subtree_root(host.b_tree), None);
                assert!(
                    labels(&tree)
                        .iter()
                        .all(|(_, tree_id)| *tree_id != host.b_tree),
                    "no node of B's is left"
                );
                find(&tree, "A field");
                find(&tree, "Host menu");

                // A still works after its neighbour failed.
                host.key(Pane::A, KeyEvent::new(Key::Character("x".into())));
                push(&mut tree, &mut grafts, &mut host);
                find(&tree, "A field");
            }

            /// Done-condition 6: both sessions place a leaf under one key, and
            /// each projects its own. Control: one shared registry gives both
            /// panes the second leaf.
            #[test]
            fn each_session_projects_its_own_leaf_under_a_shared_key() {
                let mut host = Host::new();
                let (tree, _) = open(&mut host);
                assert_eq!(find(&tree, "A leaf").locate().1, host.a_tree);
                assert_eq!(find(&tree, "B leaf").locate().1, host.b_tree);

                let mut shared = LeafRegistry::new();
                shared.insert(LEAF_KEY, Box::new(Named("A leaf")));
                shared.insert(LEAF_KEY, Box::new(Named("B leaf")));
                let composition = host.composition(Some(&mut shared));
                let mut grafts = Grafts::new();
                let mut control = ConsumerTree::new(grafts.activate(&composition), true);
                for update in grafts.frame(composition) {
                    control.update_and_process_changes(update, &mut Ignore);
                }
                let leaf_panes: Vec<TreeId> = labels(&control)
                    .into_iter()
                    .filter(|(label, _)| label.ends_with(" leaf"))
                    .map(|(label, tree_id)| {
                        assert_eq!(label, "B leaf", "control: one registry, one leaf per key");
                        tree_id
                    })
                    .collect();
                assert_eq!(leaf_panes, [host.a_tree, host.b_tree]);
            }
        }
    };
}

e1a_sessions_against!(windows_consumer_0_35, accesskit_consumer_windows);
e1a_sessions_against!(unix_consumer_0_36, accesskit_consumer_unix);
e1a_sessions_against!(macos_consumer_0_38, accesskit_consumer_macos);
