// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Experiment E1a, the tree half: two guests joined under one host as
//! AccessKit subtrees, read back through the consumer layer under each pinned
//! adapter (app composition brief §8 and AC3, ruled 2026-10-07).
//!
//! The guests here are small models that emit their own trees and take their
//! own actions, so this file checks the join itself: one tree in host reading
//! order, focus through the graft chain, actions routed by tree, bounds offset
//! by the pane origin, and a retired guest leaving nothing behind. The two
//! guests number their nodes alike, as two fresh document arenas do, so every
//! check also proves the subtrees' id spaces stay apart. Running Cambium
//! sessions, Tab handing focus across, a caught panic and the leaf key
//! namespace are the session half of E1a.
//!
//! Each check runs against `accesskit_consumer` 0.35 (under
//! `accesskit_windows` 0.32), 0.36 (under `accesskit_unix` 0.21) and 0.38
//! (under `accesskit_macos` 0.26).

use accesskit::{Action, ActionRequest, Node, NodeId, Rect, Role, Tree, TreeId, TreeUpdate};
use uxtree::{
    ActionTarget, Composition, GraftError, GraftTable, Grafts, graft_node, tree_id_for_path,
};

/// Guest A: a document with one editable field.
struct FieldGuest {
    text: String,
}

impl FieldGuest {
    fn update(&self, focused: bool) -> TreeUpdate {
        let mut root = Node::new(Role::Document);
        root.set_label("A document");
        root.set_bounds(Rect::new(0.0, 0.0, 200.0, 100.0));
        root.set_children(vec![NodeId(2)]);
        let mut field = Node::new(Role::TextInput);
        field.set_label("A field");
        field.set_value(self.text.clone());
        field.set_bounds(Rect::new(10.0, 10.0, 190.0, 30.0));
        field.add_action(Action::Focus);
        field.add_action(Action::SetValue);
        TreeUpdate {
            nodes: vec![(NodeId(1), root), (NodeId(2), field)],
            tree: Some(Tree::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: if focused { NodeId(2) } else { NodeId(1) },
        }
    }
}

/// Guest B: a list with one item and a counting button. Its ids coincide
/// with A's.
struct CounterGuest {
    count: u32,
}

impl CounterGuest {
    fn update(&self, focused: bool) -> TreeUpdate {
        let mut root = Node::new(Role::List);
        root.set_label("B list");
        root.set_bounds(Rect::new(0.0, 0.0, 200.0, 100.0));
        root.set_children(vec![NodeId(2), NodeId(3)]);
        let mut item = Node::new(Role::ListItem);
        item.set_label("B item");
        item.set_bounds(Rect::new(10.0, 40.0, 190.0, 60.0));
        let mut button = Node::new(Role::Button);
        button.set_label(format!("B count {}", self.count));
        button.set_bounds(Rect::new(10.0, 10.0, 90.0, 30.0));
        button.add_action(Action::Click);
        button.add_action(Action::Focus);
        TreeUpdate {
            nodes: vec![(NodeId(1), root), (NodeId(2), button), (NodeId(3), item)],
            tree: Some(Tree::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: if focused { NodeId(2) } else { NodeId(1) },
        }
    }

    fn act(&mut self, request: &ActionRequest) {
        if request.action == Action::Click && request.target_node == NodeId(2) {
            self.count += 1;
        }
    }
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

const A_ORIGIN: (f64, f64) = (0.0, 40.0);
const B_ORIGIN: (f64, f64) = (200.0, 40.0);
const A_GRAFT: NodeId = NodeId(10);
const B_GRAFT: NodeId = NodeId(11);

struct Host {
    a: FieldGuest,
    b: Option<CounterGuest>,
    focus: Focus,
    a_tree: TreeId,
    b_tree: TreeId,
    table: GraftTable<Pane>,
}

impl Host {
    fn new() -> Self {
        let (a_tree, b_tree) = (
            tree_id_for_path("e1a/pane/a#1"),
            tree_id_for_path("e1a/pane/b#1"),
        );
        let mut table = GraftTable::new(TreeId::ROOT);
        table.insert(a_tree, Pane::A);
        table.insert(b_tree, Pane::B);
        Self {
            a: FieldGuest {
                text: "hello".into(),
            },
            b: Some(CounterGuest { count: 0 }),
            focus: Focus::Host,
            a_tree,
            b_tree,
            table,
        }
    }

    fn admit_b(&mut self) {
        self.b = Some(CounterGuest { count: 0 });
        self.table.insert(self.b_tree, Pane::B);
    }

    fn retire_b(&mut self) {
        self.b = None;
        self.table.remove(self.b_tree);
        if self.focus == Focus::Pane(Pane::B) {
            self.focus = Focus::Host;
        }
    }

    fn host_update(&self) -> TreeUpdate {
        let mut children = vec![NodeId(2), A_GRAFT];
        if self.b.is_some() {
            children.push(B_GRAFT);
        }
        children.push(NodeId(3));
        let mut window = Node::new(Role::Window);
        window.set_label("host");
        window.set_bounds(Rect::new(0.0, 0.0, 400.0, 160.0));
        window.set_children(children);
        let mut menu = Node::new(Role::Button);
        menu.set_label("Host menu");
        menu.set_bounds(Rect::new(0.0, 0.0, 80.0, 30.0));
        let mut status = Node::new(Role::Label);
        status.set_label("Host status");
        status.set_bounds(Rect::new(0.0, 140.0, 400.0, 160.0));
        let mut a_graft = graft_node(Role::Pane, self.a_tree, A_ORIGIN, (200.0, 100.0));
        a_graft.set_label("A pane");
        let mut nodes = vec![
            (NodeId(1), window),
            (NodeId(2), menu),
            (NodeId(3), status),
            (A_GRAFT, a_graft),
        ];
        if self.b.is_some() {
            let mut b_graft = graft_node(Role::Pane, self.b_tree, B_ORIGIN, (200.0, 100.0));
            b_graft.set_label("B pane");
            nodes.push((B_GRAFT, b_graft));
        }
        TreeUpdate {
            nodes,
            tree: Some(Tree::new(NodeId(1))),
            tree_id: TreeId::ROOT,
            focus: match self.focus {
                Focus::Host => NodeId(1),
                Focus::Pane(Pane::A) => A_GRAFT,
                Focus::Pane(Pane::B) => B_GRAFT,
            },
        }
    }

    fn composition(&self) -> Composition {
        let mut guests = vec![(
            self.a_tree,
            self.a.update(self.focus == Focus::Pane(Pane::A)),
        )];
        if let Some(b) = &self.b {
            guests.push((self.b_tree, b.update(self.focus == Focus::Pane(Pane::B))));
        }
        Composition::new(self.host_update(), guests).expect("the host composes a whole frame")
    }

    fn dispatch(&mut self, request: &ActionRequest) {
        match self.table.route(request) {
            ActionTarget::Guest(Pane::B) => {
                if let Some(b) = &mut self.b {
                    b.act(request);
                }
            },
            ActionTarget::Guest(Pane::A) | ActionTarget::Host | ActionTarget::Unknown(_) => {},
        }
    }
}

/// The guests' raw trees spliced into the host's one id space, with no graft:
/// what a host gets if it publishes each session's ids as they are.
fn unjoined(host: &Host) -> TreeUpdate {
    let mut update = host.host_update();
    update
        .nodes
        .retain(|(id, _)| *id != A_GRAFT && *id != B_GRAFT);
    let window = &mut update.nodes[0].1;
    window.set_children(vec![NodeId(2), NodeId(1), NodeId(1), NodeId(3)]);
    update.nodes.extend(host.a.update(false).nodes);
    update
        .nodes
        .extend(host.b.as_ref().expect("B is live").update(false).nodes);
    update
}

macro_rules! e1a_against {
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

            /// Activation takes the host alone; the first frame follows.
            fn open(host: &Host) -> (ConsumerTree, Grafts) {
                let composition = host.composition();
                let mut grafts = Grafts::new();
                let mut tree = ConsumerTree::new(grafts.activate(&composition), true);
                push(&mut tree, &mut grafts, composition);
                (tree, grafts)
            }

            fn push(tree: &mut ConsumerTree, grafts: &mut Grafts, composition: Composition) {
                for update in grafts.frame(composition) {
                    tree.update_and_process_changes(update, &mut Ignore);
                }
            }

            /// Records where the consumer says focus moved.
            #[derive(Default)]
            struct FocusLog(Vec<Option<String>>);

            impl TreeChangeHandler for FocusLog {
                fn node_added(&mut self, _: &Read) {}
                fn node_updated(&mut self, _: &Read, _: &Read) {}
                fn focus_moved(&mut self, _: Option<&Read>, new: Option<&Read>) {
                    self.0.push(new.and_then(|node| node.label()));
                }
                fn node_removed(&mut self, _: &Read) {}
            }

            fn reading_order(tree: &ConsumerTree) -> Vec<String> {
                fn walk(node: Read<'_>, out: &mut Vec<String>) {
                    out.push(node.label().unwrap_or_default());
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

            fn focus_label(tree: &ConsumerTree) -> Option<String> {
                tree.state().focus().and_then(|node| node.label())
            }

            /// Done-condition 1: one tree, in host reading order, with both
            /// guests' coinciding ids kept apart.
            #[test]
            fn one_tree_in_host_reading_order() {
                let (tree, _) = open(&Host::new());
                assert_eq!(
                    reading_order(&tree),
                    [
                        "host",
                        "Host menu",
                        "A pane",
                        "A document",
                        "A field",
                        "B pane",
                        "B list",
                        "B count 0",
                        "B item",
                        "Host status",
                    ]
                );
            }

            /// Done-condition 2: host focus on a graft, guest focus on its
            /// node, gives the guest's node as the effective focus.
            #[test]
            fn focus_follows_the_graft_chain() {
                let mut host = Host::new();
                let (mut tree, mut grafts) = open(&host);
                assert_eq!(focus_label(&tree).as_deref(), Some("host"));

                host.focus = Focus::Pane(Pane::A);
                push(&mut tree, &mut grafts, host.composition());
                let focus = tree.state().focus().expect("a focused node");
                assert_eq!(focus.label().as_deref(), Some("A field"));
                assert_eq!(focus.locate(), (NodeId(2), host.a_tree));

                host.focus = Focus::Pane(Pane::B);
                push(&mut tree, &mut grafts, host.composition());
                let focus = tree.state().focus().expect("a focused node");
                assert_eq!(focus.label().as_deref(), Some("B count 0"));
                assert_eq!(focus.locate(), (NodeId(2), host.b_tree));
            }

            /// A guest grafted and focused in the same frame gets the focus
            /// in one move, with no stop at the host's root on the way.
            #[test]
            fn focus_into_a_new_guest_is_one_move() {
                let mut host = Host::new();
                host.retire_b();
                host.focus = Focus::Pane(Pane::A);
                let (mut tree, mut grafts) = open(&host);
                assert_eq!(focus_label(&tree).as_deref(), Some("A field"));

                host.admit_b();
                host.focus = Focus::Pane(Pane::B);
                let mut log = FocusLog::default();
                for update in grafts.frame(host.composition()) {
                    tree.update_and_process_changes(update, &mut log);
                }
                assert_eq!(log.0, [Some("B count 0".to_string())]);
            }

            /// Focus moving from one held guest to another, whose own focus
            /// changes in the same frame, is one move: no stop at the
            /// destination's stale focus, and none at the guest being left.
            #[test]
            fn focus_between_held_guests_is_one_move() {
                let mut host = Host::new();
                host.focus = Focus::Pane(Pane::A);
                let (mut tree, mut grafts) = open(&host);
                assert_eq!(focus_label(&tree).as_deref(), Some("A field"));

                host.focus = Focus::Pane(Pane::B);
                let mut log = FocusLog::default();
                for update in grafts.frame(host.composition()) {
                    tree.update_and_process_changes(update, &mut log);
                }
                assert_eq!(log.0, [Some("B count 0".to_string())]);

                host.focus = Focus::Host;
                let mut log = FocusLog::default();
                for update in grafts.frame(host.composition()) {
                    tree.update_and_process_changes(update, &mut log);
                }
                assert_eq!(log.0, [Some("host".to_string())]);
            }

            /// Done-condition 3: a Click addressed to B's tree reaches B's
            /// button; A's node with the same id is untouched.
            #[test]
            fn actions_route_by_tree() {
                let mut host = Host::new();
                let (mut tree, mut grafts) = open(&host);
                let (target_node, target_tree) = find(&tree, "B count 0").locate();
                assert_eq!(target_node, NodeId(2));
                host.dispatch(&ActionRequest {
                    action: Action::Click,
                    target_tree,
                    target_node,
                    data: None,
                });
                push(&mut tree, &mut grafts, host.composition());
                find(&tree, "B count 1");
                assert_eq!(find(&tree, "A field").value().as_deref(), Some("hello"));
                assert_eq!(
                    host.table.route(&ActionRequest {
                        action: Action::Click,
                        target_tree: host.a_tree,
                        target_node: NodeId(2),
                        data: None,
                    }),
                    ActionTarget::Guest(&Pane::A)
                );
            }

            /// Done-condition 4: a guest node's bounds are its own rectangle
            /// offset by its pane's origin.
            #[test]
            fn bounds_offset_by_the_pane_origin() {
                let (tree, _) = open(&Host::new());
                assert_eq!(
                    find(&tree, "B count 0").bounding_box(),
                    Some(Rect::new(210.0, 50.0, 290.0, 70.0))
                );
                assert_eq!(
                    find(&tree, "A field").bounding_box(),
                    Some(Rect::new(10.0, 50.0, 190.0, 70.0))
                );
                assert_eq!(
                    find(&tree, "B pane").bounding_box(),
                    Some(Rect::new(200.0, 40.0, 400.0, 140.0))
                );
            }

            /// Done-condition 5, tree half: a retired guest's graft leaves
            /// with its whole subtree, and the host and A carry on.
            #[test]
            fn a_retired_guest_leaves_nothing_behind() {
                let mut host = Host::new();
                host.focus = Focus::Pane(Pane::B);
                let (mut tree, mut grafts) = open(&host);
                let b_tree = host.b_tree;
                host.retire_b();
                push(&mut tree, &mut grafts, host.composition());
                assert_eq!(
                    reading_order(&tree),
                    [
                        "host",
                        "Host menu",
                        "A pane",
                        "A document",
                        "A field",
                        "Host status"
                    ]
                );
                assert_eq!(tree.state().subtree_root(b_tree), None);
                assert_eq!(focus_label(&tree).as_deref(), Some("host"));
                assert_eq!(
                    host.table.route(&ActionRequest {
                        action: Action::Click,
                        target_tree: b_tree,
                        target_node: NodeId(2),
                        data: None,
                    }),
                    ActionTarget::Unknown(b_tree)
                );
            }

            /// Control: the guests' raw ids published into the host's one
            /// tree collide, and the consumer refuses the update.
            #[test]
            fn control_raw_ids_collide() {
                let host = Host::new();
                let outcome = std::panic::catch_unwind(|| {
                    let tree = ConsumerTree::new(unjoined(&host), true);
                    reading_order(&tree)
                });
                match outcome {
                    Err(_) => {},
                    Ok(order) => assert!(
                        !order.iter().any(|label| label == "A field")
                            || !order.iter().any(|label| label == "B list"),
                        "raw ids kept both guests apart: {order:?}"
                    ),
                }
            }

            /// Control: a subtree pushed without its graft is refused by the
            /// consumer, and by `Composition` before it gets there.
            #[test]
            fn control_graft_omitted() {
                let host = Host::new();
                let mut bare = host.host_update();
                bare.nodes.retain(|(id, _)| *id != B_GRAFT);
                bare.nodes[0]
                    .1
                    .set_children(vec![NodeId(2), A_GRAFT, NodeId(3)]);
                let guests = vec![
                    (host.a_tree, host.a.update(false)),
                    (host.b_tree, host.b.as_ref().unwrap().update(false)),
                ];
                assert_eq!(
                    Composition::new(bare.clone(), guests).err(),
                    Some(GraftError::MissingGraft(host.b_tree))
                );

                let outcome = std::panic::catch_unwind(|| {
                    let mut tree = ConsumerTree::new(bare, true);
                    let b = uxtree::as_subtree(host.b.as_ref().unwrap().update(false), host.b_tree);
                    tree.update_and_process_changes(b, &mut Ignore);
                });
                assert!(
                    outcome.is_err(),
                    "the consumer took a subtree with no graft"
                );
            }
        }
    };
}

e1a_against!(windows_consumer_0_35, accesskit_consumer_windows);
e1a_against!(unix_consumer_0_36, accesskit_consumer_unix);
e1a_against!(macos_consumer_0_38, accesskit_consumer_macos);
