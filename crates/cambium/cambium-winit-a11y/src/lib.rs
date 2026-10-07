/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Accessibility host for a genet-backed Cambium app.
//!
//! Split out of `cambium-winit` on 2026-07-26, code unchanged. It lives apart
//! for one reason: it needs the laid-out genet DOM and the platform adapter,
//! which cannot be published, and holding them here leaves `cambium-winit` with
//! only `cambium` and `winit` so its key translation is reachable from a
//! registry consumer. A host that wants both takes both crates.
//!
//! Every Cambium app emits a semantic, ARIA-attributed DOM laid out by
//! Livery/Buckram, and paints its custom visuals with Sprigging leaves. This
//! module turns that into a live accessibility tree for the OS screen reader,
//! so no app has to hand-roll the wiring:
//!
//! - [`A11yHost`] owns the platform adapter and the per-frame lifecycle: project
//!   the retained layout into an AccessKit tree (leaf semantics included),
//!   install it the first frame and update it after, and drain a screen reader's
//!   actions, handing back the DOM nodes to activate so the app routes them
//!   through the same click path a mouse uses.
//!
//! The app keeps only what is app-specific: create the window hidden (the
//! adapter must attach before it is shown), drive the first frame synchronously
//! (a hidden window may not receive a deferred redraw), turn the wake callback
//! into a redraw, and dispatch the returned nodes.

use std::collections::HashMap;

use accesskit::{Action, ActionData, Affine, NodeId as A11yNodeId, Rect, Role, TreeUpdate};
use cambium_rootstock::{ProducedAction, ProducerRegistry, ProducerRole, ProducerSemantics};
use genet_scripted_dom::NodeId;
use genet_winit_host::{AccessKitBridge, BridgeStatus};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};
use sprigging::LeafRegistry;
use winit::window::Window;

/// The screen-reader request vocabulary, re-exported from the neutral seam.
///
/// It lives there rather than here because it never named winit or AccessKit:
/// an action and a DOM node are the same two things whichever platform asked.
/// Re-exported so callers that already import it from this crate keep working.
pub use cambium_rootstock::{A11yAction, A11yRequest, A11yTarget, Accessibility};

impl Accessibility for A11yHost {
    fn sync(
        &mut self,
        dom: &cambium_rootstock::WindowDom<'_>,
        layout: &cambium_rootstock::OwnedLayout,
        leaves: &mut LeafRegistry<u64>,
        producers: &mut ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> Vec<A11yRequest> {
        self.sync_inner(dom, layout, leaves, producers, focus, layout_scale)
    }
}

/// Owns the OS AccessKit adapter and the per-frame tree lifecycle for a
/// genet-backed Cambium app. Create it in `resumed` (with a wake callback that
/// nudges the event loop), then call [`A11yHost::sync`] after every frame.
pub struct A11yHost {
    /// The native handle AccessKit binds its parallel tree to. Held rather than
    /// passed per call: it is what makes this implementation winit's, and the
    /// neutral seam has no room for it.
    ///
    /// Optional so the host is constructible without one. A window cannot be
    /// made in a unit test, and requiring one here would put `map_request` and
    /// the projection out of reach of exactly the receipts that should cover
    /// them. Until [`attach`](Self::attach), `sync` installs nothing and
    /// returns no requests, which is the honest answer for a tree no reader can
    /// see yet.
    window: Option<std::sync::Arc<Window>>,
    bridge: AccessKitBridge,
    installed: bool,
    /// AccessKit node id -> its DOM node, rebuilt each frame, so a screen
    /// reader's action on a node routes back to the element it came from.
    action_map: HashMap<A11yNodeId, NodeId>,
    /// AccessKit node id -> the drawn node's action its button stands for,
    /// rebuilt each frame, so a reader's click reaches the producer.
    produced_map: HashMap<A11yNodeId, ProducedAction>,
}

impl A11yHost {
    /// Create the adapter. `wake` is called by the adapter when a screen reader
    /// acts while the app is idle; wire it to request a redraw so the queued
    /// action gets drained (e.g. set a flag honored in `about_to_wait`).
    pub fn new(wake: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            window: None,
            bridge: AccessKitBridge::new(wake),
            installed: false,
            action_map: HashMap::new(),
            produced_map: HashMap::new(),
        }
    }

    /// Whether the platform adapter is live.
    pub fn status(&self) -> BridgeStatus {
        self.bridge.status()
    }

    /// Project the current layout into an AccessKit tree (with each leaf's own
    /// semantics), install it on the first call — revealing `window`, which must
    /// have been created hidden so the adapter attaches first — and update it
    /// after. Returns the screen reader's Click and Focus requests, in request
    /// order and still typed, for the caller to route each through the matching
    /// path (activation for `Click`, focus for `Focus`).
    ///
    /// `focus` is the app's currently-focused DOM node's opaque id (from
    /// `LayoutDom::opaque_id`), used as the tree's focus when it is really in the
    /// tree, so a stale id never points the reader at nothing.
    fn sync_inner(
        &mut self,
        dom: &cambium_rootstock::WindowDom<'_>,
        layout: &cambium_rootstock::OwnedLayout,
        leaves: &mut LeafRegistry<u64>,
        producers: &mut ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> Vec<A11yRequest> {
        let Some(window) = self.window.clone() else {
            // No window yet: nothing to install against, and no reader to ask.
            return Vec::new();
        };
        let (mut tree, action_map, produced_map) =
            project_tree_with_actions(dom, layout, leaves, producers, focus);
        self.action_map = action_map;
        self.produced_map = produced_map;
        scale_tree_to_window(&mut tree, dom, layout_scale);
        let node_count = tree.nodes.len();

        if !self.installed {
            match self.bridge.install(&window, tree) {
                Ok(()) => eprintln!(
                    "[cambium-winit] accessibility {:?}, {node_count} nodes projected",
                    self.bridge.status()
                ),
                Err(e) => eprintln!("[cambium-winit] accessibility install failed: {e}"),
            }
            self.installed = true;
            window.set_visible(true);
            return Vec::new();
        }

        self.bridge.update(tree);
        // Route a screen reader's requests back to their DOM nodes, or to the
        // producer whose drawn node's button was pressed, each still carrying
        // the action it asked for.
        self.bridge
            .drain_actions()
            .into_iter()
            .filter_map(|req| self.map_parts(req.action, req.data, req.target_node))
            .collect()
    }

    /// Give the host the window AccessKit installs against.
    ///
    /// Call once, as soon as the window exists. Before this the host projects
    /// nothing; after it, the first [`sync`](Accessibility::sync) installs the
    /// tree and reveals the window.
    pub fn attach(&mut self, window: std::sync::Arc<Window>) {
        self.window = Some(window);
    }

    /// Map a raw AccessKit request to a typed one against the tree that was
    /// last synced. `None` for an action this host does not route, or a target
    /// that is no longer in the tree.
    ///
    /// Public because it is the seam between "the OS asked for something" and
    /// "the app does it": a test can feed a request the same way the adapter
    /// does, without a screen reader.
    pub fn map_request(&self, request: &accesskit::ActionRequest) -> Option<A11yRequest> {
        self.map_parts(request.action, request.data.clone(), request.target_node)
    }

    /// The parts of a request, whichever type carried them.
    fn map_parts(
        &self,
        action: Action,
        data: Option<ActionData>,
        target: A11yNodeId,
    ) -> Option<A11yRequest> {
        let action = match action {
            Action::Click => A11yAction::Click,
            Action::Focus => A11yAction::Focus,
            Action::SetValue => match data {
                Some(ActionData::NumericValue(value)) if value.is_finite() => {
                    A11yAction::SetValue(value)
                },
                _ => return None,
            },
            _ => return None,
        };
        if let Some(node) = self.action_map.get(&target) {
            return Some(A11yRequest::node(action, *node));
        }
        let produced = self.produced_map.get(&target)?;
        Some(A11yRequest::produced(action, produced.clone()))
    }
}

/// Project a laid-out Cambium document into an AccessKit tree, with no window
/// and no platform adapter: the half of [`A11yHost::sync`] that is pure.
///
/// Returns the tree update and the AccessKit-id → DOM-node map a drained
/// action is resolved through. Split out so the projection is assertable in an
/// ordinary test — an accessibility regression that only a screen reader can
/// catch is one nobody catches.
pub fn project_tree<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &cambium_rootstock::OwnedLayout,
    leaves: &mut LeafRegistry<u64>,
    producers: &mut ProducerRegistry,
    focus: Option<u64>,
) -> (TreeUpdate, HashMap<A11yNodeId, NodeId>) {
    let (tree, nodes, _) = project_tree_with_actions(dom, layout, leaves, producers, focus);
    (tree, nodes)
}

/// [`project_tree`], also returning which drawn node's action each produced
/// button stands for, the map a reader's click on one resolves through.
pub fn project_tree_with_actions<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &cambium_rootstock::OwnedLayout,
    leaves: &mut LeafRegistry<u64>,
    producers: &mut ProducerRegistry,
    focus: Option<u64>,
) -> (
    TreeUpdate,
    HashMap<A11yNodeId, NodeId>,
    HashMap<A11yNodeId, ProducedAction>,
) {
    let root = dom.document();
    let id_of = |d: &D, n: NodeId| A11yNodeId(d.opaque_id(n));
    let focused = focus.and_then(|opaque| find_opaque(dom, root, opaque));
    let mut tree = genet_render::accesskit_tree_with_style(
        dom,
        layout.fragments(),
        focused,
        &genet_render::A11yStyleQueries {
            generated: &|node| layout.generated_text(dom, node),
            rendered: &|node| layout.rendered_visible(dom, node),
        },
    );
    let mut action_map = HashMap::new();
    let mut produced = Vec::new();
    walk(dom, root, &mut |node| {
        let id = id_of(dom, node);
        action_map.insert(id, node);
        if let Some(key) = custom_leaf_key(dom, node)
            && let Some(leaf) = leaves.get_mut(&key)
            && let Some((_, access)) = tree
                .nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == id)
        {
            leaf.accessibility(access);
        }
        if let Some(key) = custom_leaf_key(dom, node)
            && let Some(semantics) = producers.semantics(key)
        {
            produced.push((id, key, semantics));
        }
        if dom.attribute(
            node,
            &Namespace::default(),
            &LocalName::from("data-cambium-set-value"),
        ) == Some("true")
            && let Some((_, access)) = tree
                .nodes
                .iter_mut()
                .find(|(candidate, _)| *candidate == id)
        {
            access.add_action(Action::SetValue);
        }
    });
    let mut produced_map = HashMap::new();
    for (slot, key, semantics) in produced {
        add_producer_semantics(&mut tree, slot, key, semantics, &mut produced_map);
    }
    (tree, action_map, produced_map)
}

/// Write a producer's own semantics onto its slot's node, and what it draws
/// as child nodes placed where it draws them. A drawn node's actions become
/// its Button children, each taking Click (`producer` is the slot's registry
/// key, which a reader's click on one names in `produced_map`).
fn add_producer_semantics(
    tree: &mut TreeUpdate,
    slot: A11yNodeId,
    producer: u64,
    semantics: ProducerSemantics,
    produced_map: &mut HashMap<A11yNodeId, ProducedAction>,
) {
    let Some(index) = tree.nodes.iter().position(|(id, _)| *id == slot) else {
        return;
    };
    let origin = {
        let access = &mut tree.nodes[index].1;
        if let Some(role) = semantics.role {
            access.set_role(accesskit_role(role));
        }
        if let Some(name) = semantics.name {
            access.set_label(name);
        }
        access.bounds().map_or((0.0, 0.0), |bounds| (bounds.x0, bounds.y0))
    };
    let mut children = Vec::new();
    let mut buttons = Vec::new();
    for child in semantics.children {
        let id = produced_node_id(slot, child.key, None);
        let mut access = accesskit::Node::new(accesskit_role(child.role));
        access.set_label(child.name);
        let [x, y, width, height] = child.rect.map(f64::from);
        let bounds = Rect {
            x0: origin.0 + x,
            y0: origin.1 + y,
            x1: origin.0 + x + width,
            y1: origin.1 + y + height,
        };
        access.set_bounds(bounds);
        for action in child.actions {
            let button_id = produced_node_id(slot, child.key, Some(&action.id));
            let mut button = accesskit::Node::new(Role::Button);
            button.set_label(action.label);
            button.set_description(action.description);
            button.set_bounds(bounds);
            button.add_action(Action::Click);
            button.add_action(Action::Focus);
            access.push_child(button_id);
            produced_map.insert(
                button_id,
                ProducedAction {
                    slot: producer,
                    key: child.key,
                    id: action.id,
                },
            );
            buttons.push((button_id, button));
        }
        children.push((id, access));
    }
    let ids: Vec<A11yNodeId> = children.iter().map(|(id, _)| *id).collect();
    for id in &ids {
        tree.nodes[index].1.push_child(*id);
    }
    tree.nodes.extend(children);
    tree.nodes.extend(buttons);
}

/// A drawn node's id, or its action button's: stable for the slot, the
/// node's key and the action, however the producer orders its nodes, in a
/// range DOM ids never reach.
fn produced_node_id(slot: A11yNodeId, key: u64, action: Option<&str>) -> A11yNodeId {
    // FNV-1a: fixed, so an id is the same on every frame and every run.
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    let mut eat = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    eat(&slot.0.to_le_bytes());
    eat(&key.to_le_bytes());
    if let Some(action) = action {
        eat(&[1]);
        eat(action.as_bytes());
    }
    A11yNodeId((1 << 63) | (hash >> 1))
}

fn accesskit_role(role: ProducerRole) -> Role {
    match role {
        ProducerRole::List => Role::List,
        ProducerRole::ListItem => Role::ListItem,
        ProducerRole::Group => Role::Group,
        ProducerRole::Image => Role::Image,
        ProducerRole::GraphicsObject => Role::GraphicsObject,
    }
}

/// Stamp the host's layout scale on the tree root.
///
/// Layout bounds are layout-space CSS pixels, and AccessKit expects the final
/// transformed coordinates to be the platform's physical client pixels. The
/// Windows adapter applies no DPI conversion of its own, so without this a
/// screen reader or UI Automation client at 125% is told every control sits
/// at four fifths of its true position.
///
/// `layout_scale` is the device scale **times the UI zoom**, because both
/// separate the two spaces and a reader has no way to know about either. Pass
/// [`Host::layout_scale`](cambium_rootstock::Host::layout_scale).
pub fn scale_tree_to_window<D: LayoutDom<NodeId = NodeId>>(
    tree: &mut TreeUpdate,
    dom: &D,
    layout_scale: f64,
) {
    let root = A11yNodeId(dom.opaque_id(dom.document()));
    if let Some((_, node)) = tree.nodes.iter_mut().find(|(id, _)| *id == root) {
        node.set_transform(Affine::scale(layout_scale));
    }
}

fn walk<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId, visit: &mut impl FnMut(NodeId)) {
    visit(node);
    for child in dom.dom_children(node) {
        walk(dom, child, visit);
    }
}

fn find_opaque<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    node: NodeId,
    opaque: u64,
) -> Option<NodeId> {
    if dom.opaque_id(node) == opaque {
        return Some(node);
    }
    dom.dom_children(node)
        .find_map(|child| find_opaque(dom, child, opaque))
}

fn custom_leaf_key<D: LayoutDom<NodeId = NodeId>>(dom: &D, node: NodeId) -> Option<u64> {
    if dom.kind(node) != NodeKind::Element
        || !matches!(
            dom.element_name(node)?.local.as_ref(),
            "custom-leaf" | "chisel-leaf"
        )
    {
        return None;
    }
    dom.attribute(node, &Namespace::default(), &LocalName::from("key"))?
        .parse()
        .ok()
}

#[cfg(test)]
mod dpi_tests {
    use genet_scripted_dom::ScriptedDom;

    use super::*;
    use accesskit::{ActionRequest, Node, Role, Tree, TreeId};

    /// A 125% window must report physical coordinates to the platform: the
    /// logical layout bounds ride a root transform that scales them.
    #[test]
    fn tree_root_carries_the_window_scale() {
        let dom = ScriptedDom::new();
        let root = A11yNodeId(dom.opaque_id(dom.document()));
        let mut tree = TreeUpdate {
            nodes: vec![(root, Node::new(Role::Window))],
            tree: Some(Tree::new(root)),
            tree_id: TreeId::ROOT,
            focus: root,
        };
        scale_tree_to_window(&mut tree, &dom, 1.25);
        let (_, node) = tree.nodes.iter().find(|(id, _)| *id == root).unwrap();
        assert_eq!(node.transform(), Some(&Affine::scale(1.25)));
    }

    #[test]
    fn numeric_set_value_maps_only_finite_numeric_data() {
        let dom = ScriptedDom::new();
        let node = dom.document();
        let target = A11yNodeId(44);
        let mut host = A11yHost::new(|| {});
        host.action_map.insert(target, node);
        let request = |data| ActionRequest {
            action: Action::SetValue,
            target_tree: TreeId::ROOT,
            target_node: target,
            data,
        };
        assert_eq!(
            host.map_request(&request(Some(ActionData::NumericValue(2.5)))),
            Some(A11yRequest::node(A11yAction::SetValue(2.5), node)),
        );
        assert_eq!(host.map_request(&request(None)), None);
        assert_eq!(
            host.map_request(&request(Some(ActionData::NumericValue(f64::NAN)))),
            None,
        );
    }

    /// A producer's slot is named by the producer, and each thing it draws
    /// is a child node placed where it is drawn. A drawn node's actions are
    /// its Button children (dynamics grammar plan, F65), and a reader's
    /// click on one reaches the producer's `act` with the node's key and the
    /// action's id (F66).
    #[test]
    fn a_producers_children_reach_the_tree() {
        use cambium_rootstock::{
            ProducedTexture, ProducerAction, ProducerContext, ProducerNode, ProducerRegistry,
            TextureProducer,
        };
        use std::cell::RefCell;
        use std::rc::Rc;
        struct Board(Rc<RefCell<Vec<(u64, String)>>>);
        impl TextureProducer for Board {
            fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
                None
            }
            fn semantics(&mut self) -> Option<ProducerSemantics> {
                Some(ProducerSemantics {
                    role: Some(ProducerRole::List),
                    name: Some("Remote board".into()),
                    children: vec![
                        ProducerNode {
                            key: 40,
                            role: ProducerRole::ListItem,
                            name: "Card 0".into(),
                            rect: [24.0, 24.0, 120.0, 80.0],
                            actions: vec![ProducerAction {
                                id: "pin".into(),
                                label: "Pin".into(),
                                description: "Hold this card".into(),
                            }],
                        },
                        ProducerNode {
                            key: 41,
                            role: ProducerRole::ListItem,
                            name: "Card 1".into(),
                            rect: [164.0, 24.0, 120.0, 80.0],
                            actions: Vec::new(),
                        },
                    ],
                })
            }
            fn act(&mut self, key: u64, id: &str) -> bool {
                self.0.borrow_mut().push((key, id.to_string()));
                key == 40 && id == "pin"
            }
        }
        let slot = A11yNodeId(7);
        let mut tree = TreeUpdate {
            nodes: vec![(slot, {
                let mut node = Node::new(Role::Image);
                node.set_bounds(Rect {
                    x0: 10.0,
                    y0: 100.0,
                    x1: 1000.0,
                    y1: 700.0,
                });
                node
            })],
            tree: Some(Tree::new(slot)),
            tree_id: TreeId::ROOT,
            focus: slot,
        };
        let acted = Rc::new(RefCell::new(Vec::new()));
        let mut producers = ProducerRegistry::new();
        producers.register(1, Board(acted.clone()), &[]).unwrap();
        let semantics = producers.semantics(1).expect("the board describes itself");
        let mut produced_map = HashMap::new();
        add_producer_semantics(&mut tree, slot, 1, semantics, &mut produced_map);

        let (_, board) = tree.nodes.iter().find(|(id, _)| *id == slot).unwrap();
        assert_eq!(board.role(), Role::List);
        assert_eq!(board.label(), Some("Remote board"));
        assert_eq!(board.children().len(), 2);
        let card = |id: A11yNodeId| tree.nodes.iter().find(|(n, _)| *n == id).unwrap().1.clone();
        let first = card(board.children()[0]);
        assert_eq!(first.role(), Role::ListItem);
        assert_eq!(first.label(), Some("Card 0"));
        assert_eq!(
            first.bounds(),
            Some(Rect { x0: 34.0, y0: 124.0, x1: 154.0, y1: 204.0 }),
            "placed where it is drawn, from the slot's corner"
        );
        assert_eq!(card(board.children()[1]).label(), Some("Card 1"));
        assert!(card(board.children()[1]).children().is_empty(), "no actions, no buttons");

        // Card 0's one action is a Button child taking Click, named by its
        // label and described by its description.
        assert_eq!(first.children().len(), 1);
        let pin_id = first.children()[0];
        let pin = card(pin_id);
        assert_eq!(pin.role(), Role::Button);
        assert_eq!(pin.label(), Some("Pin"));
        assert_eq!(pin.description(), Some("Hold this card"));
        assert!(pin.supports_action(Action::Click));

        // A reader's click on it maps to the drawn node's action and reaches
        // the producer, which carries it out; the card itself is no button.
        let mut host = A11yHost::new(|| {});
        host.produced_map = produced_map;
        let click = |target_node| ActionRequest {
            action: Action::Click,
            target_tree: TreeId::ROOT,
            target_node,
            data: None,
        };
        let request = host.map_request(&click(pin_id)).expect("the button routes");
        let produced = ProducedAction {
            slot: 1,
            key: 40,
            id: "pin".into(),
        };
        assert_eq!(request, A11yRequest::produced(A11yAction::Click, produced.clone()));
        assert!(producers.act(&produced), "the producer carried it out");
        assert_eq!(acted.borrow().as_slice(), &[(40, "pin".to_string())]);
        assert_eq!(host.map_request(&click(board.children()[0])), None);
        // An unknown slot reaches no producer.
        assert!(!producers.act(&ProducedAction { slot: 9, ..produced }));
        // A producer without semantics leaves its slot alone.
        assert_eq!(producers.semantics(2), None);
    }
}
