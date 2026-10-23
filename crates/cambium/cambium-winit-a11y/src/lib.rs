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

use accesskit::{Action, ActionData, Affine, NodeId as A11yNodeId, TreeUpdate};
use cambium_rootstock::{ProducedAction, ProducerRegistry};
use genet_scripted_dom::NodeId;
use genet_winit_host::{AccessKitBridge, BridgeStatus};
use layout_dom_api::LayoutDom;
use sprigging::LeafRegistry;
use winit::window::Window;

#[cfg(test)]
mod namespace_tests;
mod publication;
#[cfg(test)]
mod publication_tests;

/// The screen-reader request vocabulary, re-exported from the neutral seam.
///
/// It lives there rather than here because it never named winit or AccessKit:
/// an action and a DOM node are the same two things whichever platform asked.
/// Re-exported so callers that already import it from this crate keep working.
pub use cambium_rootstock::{A11yAction, A11yRequest, A11yTarget, Accessibility};
use cambium_rootstock::{
    A11yAdapterState, A11yPublication, A11yPublicationOutcome, A11yPublicationRefusal,
    A11yPublicationSkip, A11ySync,
};

impl Accessibility for A11yHost {
    fn adapter_state(&self) -> A11yAdapterState {
        self.adapter_state()
    }

    fn sync(
        &mut self,
        owner: &cambium_rootstock::A11yOwner,
        dom: &cambium_rootstock::WindowDom<'_>,
        layout: &cambium_rootstock::OwnedLayout,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> A11ySync {
        self.sync_inner(owner, dom, layout, leaves, producers, focus, layout_scale)
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
    /// Native publication IDs map to the last published contributor bindings.
    requests: HashMap<A11yNodeId, A11yRequest>,
    namespace: publication::NativeIds,
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
            requests: HashMap::new(),
            namespace: publication::NativeIds::default(),
        }
    }

    /// Physical adapter installation only; this does not report latest tree publication.
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
        owner: &cambium_rootstock::A11yOwner,
        dom: &cambium_rootstock::WindowDom<'_>,
        layout: &cambium_rootstock::OwnedLayout,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        layout_scale: f64,
    ) -> A11ySync {
        if !owner.is_live() {
            self.requests.clear();
            self.namespace.clear();
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::skipped(
                    self.adapter_state(),
                    A11yPublicationSkip::OwnerEnded,
                ),
            };
        }
        let Some(window) = self.window.clone() else {
            // No window yet: nothing to install against, and no reader to ask.
            return A11ySync {
                requests: Vec::new(),
                publication: A11yPublication::skipped(
                    A11yAdapterState::Detached,
                    A11yPublicationSkip::NoWindow,
                ),
            };
        };
        // Resolve queued events through the publication they could have seen,
        // before reading contributors or replacing the published maps.
        let requests = self
            .bridge
            .drain_actions()
            .into_iter()
            .filter_map(|req| {
                self.map_parts(req.action, req.data, req.target_tree, req.target_node)
            })
            .collect();
        let projection =
            cambium_rootstock::project_semantics(dom, layout, leaves, producers, focus);
        let contribution_issues = projection.producer_issues.len();
        let bindings = projection.publication_bindings(owner, dom, leaves, producers);
        let mut tree = projection.tree;
        scale_tree_to_window(&mut tree, dom, layout_scale);
        let publication = bindings
            .map_err(|issue| {
                (
                    A11yPublicationRefusal::InvalidBindings,
                    format!("{issue:?}"),
                )
            })
            .and_then(|bindings| {
                self.namespace
                    .prepare(tree, bindings)
                    .map_err(|issue| (A11yPublicationRefusal::InvalidTree, format!("{issue:?}")))
            });
        let publication::Prepared {
            tree,
            namespace,
            requests: next,
        } = match publication {
            Ok(candidate) => candidate,
            Err((reason, issue)) => {
                self.requests.clear();
                self.namespace.clear();
                eprintln!("[cambium-winit] accessibility publication refused: {issue}");
                // Match install failure: a rejected accessibility tree must not
                // leave the application hidden. A later sync may retry.
                window.set_visible(true);
                return A11ySync {
                    requests: Vec::new(),
                    publication: A11yPublication::refused(self.adapter_state(), reason),
                };
            },
        };
        let node_count = tree.nodes.len();
        let mut outcome = A11yPublicationOutcome::Submitted {
            nodes: node_count,
            contribution_issues,
        };
        if !self.installed {
            match self.bridge.install(&window, tree) {
                Ok(()) => {
                    self.installed = true;
                    self.requests = next;
                    self.namespace = namespace;
                    eprintln!(
                        "[cambium-winit] accessibility {:?}, {node_count} nodes projected",
                        self.bridge.status()
                    );
                },
                Err(e) => {
                    self.requests.clear();
                    self.namespace.clear();
                    outcome = A11yPublicationOutcome::Refused {
                        reason: A11yPublicationRefusal::AdapterInstall,
                    };
                    eprintln!("[cambium-winit] accessibility install failed: {e}");
                },
            }
            // Keep the app usable on adapter failure; retry installation on the
            // next sync rather than claiming that a failed install succeeded.
            window.set_visible(true);
        } else {
            self.bridge.update(tree);
            self.requests = next;
            self.namespace = namespace;
        }
        A11ySync {
            requests,
            publication: A11yPublication {
                adapter: self.adapter_state(),
                outcome,
            },
        }
    }

    fn adapter_state(&self) -> A11yAdapterState {
        if self.window.is_none() {
            return A11yAdapterState::Detached;
        }
        match self.bridge.status() {
            BridgeStatus::Installed => A11yAdapterState::Installed,
            BridgeStatus::Unavailable => A11yAdapterState::Unavailable,
        }
    }

    /// Give the host the window AccessKit installs against.
    ///
    /// Call as soon as the window exists. Reattaching the same handle is
    /// harmless; another window requires a fresh host. Refusal leaves the
    /// original window and installed adapter together. Before attachment the
    /// host projects nothing; the first [`sync`](Accessibility::sync) then
    /// installs the tree and reveals the window.
    pub fn attach(&mut self, window: std::sync::Arc<Window>) -> Result<(), String> {
        attach_window(&mut self.window, window)
    }

    /// Map a raw AccessKit request to a typed one against the tree that was
    /// last synced. `None` for an action this host does not route, or a target
    /// that is no longer in the tree.
    ///
    /// Public because it is the seam between "the OS asked for something" and
    /// "the app does it": a test can feed a request the same way the adapter
    /// does, without a screen reader.
    pub fn map_request(&self, request: &accesskit::ActionRequest) -> Option<A11yRequest> {
        self.map_parts(
            request.action,
            request.data.clone(),
            request.target_tree,
            request.target_node,
        )
    }

    /// The parts of a request, whichever type carried them.
    fn map_parts(
        &self,
        action: Action,
        data: Option<ActionData>,
        tree: accesskit::TreeId,
        target: A11yNodeId,
    ) -> Option<A11yRequest> {
        // This host publishes one ROOT tree through bridge.install/update.
        // Embedded-tree routing belongs to the bridge's separate batch API;
        // its numeric IDs cannot be resolved through these host-only maps.
        if tree != accesskit::TreeId::ROOT {
            return None;
        }
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
        let mut request = self.requests.get(&target)?.clone();
        request.action = action;
        Some(request)
    }
}

fn attach_window<W>(
    current: &mut Option<std::sync::Arc<W>>,
    requested: std::sync::Arc<W>,
) -> Result<(), String> {
    if let Some(window) = current {
        if !std::sync::Arc::ptr_eq(window, &requested) {
            return Err(
                "accessibility host already owns another window; create a fresh host".into(),
            );
        }
    } else {
        *current = Some(requested);
    }
    Ok(())
}

/// Project a laid-out Cambium document into an AccessKit tree, with no window
/// and no platform adapter. Providers are read through shared borrows; this
/// copied observation is not a presentation-revision snapshot or target lease.
///
/// Returns semantic source IDs and their DOM routes for composition. These
/// are not the native IDs installed by A11yHost or accepted by map_request. Split out so the projection is assertable in an
/// ordinary test — an accessibility regression that only a screen reader can
/// catch is one nobody catches.
pub fn project_tree<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &cambium_rootstock::OwnedLayout,
    leaves: &LeafRegistry<u64>,
    producers: &ProducerRegistry,
    focus: Option<u64>,
) -> (TreeUpdate, HashMap<A11yNodeId, NodeId>) {
    let (tree, nodes, _) = project_tree_with_actions(dom, layout, leaves, producers, focus);
    (tree, nodes)
}

/// [`project_tree`], also returning semantic source routes for produced actions.
/// Native publication remaps the complete tree before installing its own routes.
pub fn project_tree_with_actions<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &cambium_rootstock::OwnedLayout,
    leaves: &LeafRegistry<u64>,
    producers: &ProducerRegistry,
    focus: Option<u64>,
) -> (
    TreeUpdate,
    HashMap<A11yNodeId, NodeId>,
    HashMap<A11yNodeId, ProducedAction>,
) {
    let projection = cambium_rootstock::project_semantics(dom, layout, leaves, producers, focus);
    (
        projection.tree,
        projection.dom_targets,
        projection.produced_targets,
    )
}

/// Stamp the host's layout scale on the semantic source tree root. Apply this
/// before native ID remapping.
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

#[cfg(test)]
mod dpi_tests {
    use cambium_rootstock::{OwnedLayout, ProducerRegistry};
    use genet_scripted_dom::ScriptedDom;
    use layout_dom_api::LayoutDomMut;
    use sprigging::LeafRegistry;
    use std::collections::HashMap;

    use super::*;
    use accesskit::{ActionRequest, Node, Rect, Role, TreeId, TreeInfo};
    use cambium_rootstock::{ProducerRole, ProducerSemantics};
    use layout_dom_api::{LocalName, Namespace, NodeKind};

    /// A 125% window must report physical coordinates to the platform: the
    /// logical layout bounds ride a root transform that scales them.
    #[test]
    fn tree_root_carries_the_window_scale() {
        let dom = ScriptedDom::new();
        let root = A11yNodeId(dom.opaque_id(dom.document()));
        let mut tree = TreeUpdate {
            nodes: vec![(root, Node::new(Role::Window))],
            tree: Some(TreeInfo::new(root)),
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
        host.requests
            .insert(target, A11yRequest::node(A11yAction::Focus, node));
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

    #[test]
    fn lifetime_tests_foreign_tree_ids_cannot_alias_host_dom_or_producer_targets() {
        let dom = ScriptedDom::new();
        let node = dom.document();
        let mut host = A11yHost::new(|| {});
        host.requests
            .insert(A11yNodeId(44), A11yRequest::node(A11yAction::Focus, node));
        let produced = ProducedAction {
            slot: 7,
            key: 40,
            id: "pin".into(),
        };
        host.requests.insert(
            A11yNodeId(45),
            A11yRequest::produced(A11yAction::Focus, produced.clone()),
        );
        for (target, expected) in [
            (44, A11yRequest::node(A11yAction::Click, node)),
            (45, A11yRequest::produced(A11yAction::Click, produced)),
        ] {
            let mut request = ActionRequest {
                action: Action::Click,
                target_tree: TreeId(accesskit::Uuid::from_u128(77)),
                target_node: A11yNodeId(target),
                data: None,
            };
            assert_eq!(host.map_request(&request), None);
            request.target_tree = TreeId::ROOT;
            assert_eq!(host.map_request(&request), Some(expected));
        }
    }

    #[test]
    fn committed_text_projects_as_an_editable_textbox_value() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(
            root,
            "<div aria-label=\"Notes\" role=\"textbox\" aria-multiline=\"true\" data-cambium-text-value=\"Café 👩🏽‍🚀\nSecond line\" style=\"width:240px;height:40px\">Café 👩🏽‍🚀<span> ghost</span><span>preedit</span><span>│</span></div><div aria-label=\"Title\" role=\"textbox\" data-cambium-text-value=\"Résumé 🇫🇷\" style=\"width:240px;height:32px\">Résumé 🇫🇷<span>ghost</span></div><div role=\"button\" data-cambium-text-value=\"not a field\">Button</div>",
        );
        let mut pending = vec![dom.document()];
        let mut field_dom = None;
        while let Some(node) = pending.pop() {
            if dom.attribute(node, &Namespace::default(), &LocalName::from("aria-label"))
                == Some("Notes")
            {
                field_dom = Some(node);
                break;
            }
            pending.extend(dom.dom_children(node));
        }
        let field_dom = field_dom.expect("the fixture has its app textbox");
        let focus = dom.opaque_id(field_dom);
        let drawn_children: Vec<_> = dom.dom_children(field_dom).collect();
        let drawn_descendants: Vec<_> = drawn_children
            .iter()
            .flat_map(|child| {
                let mut pending = vec![*child];
                let mut ids = Vec::new();
                while let Some(node) = pending.pop() {
                    ids.push(A11yNodeId(dom.opaque_id(node)));
                    pending.extend(dom.dom_children(node));
                }
                ids
            })
            .collect();
        let layout = OwnedLayout::new(&dom, &[""], 320.0, 180.0, &[], &HashMap::new());
        let mut leaves = LeafRegistry::new();
        let (tree, action_map, _) = project_tree_with_actions(
            &dom,
            &layout,
            &mut leaves,
            &mut ProducerRegistry::new(),
            Some(focus),
        );
        let field = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::MultilineTextInput)
            .map(|(_, node)| node)
            .expect("the field has multiline textbox semantics");
        assert_eq!(field.value(), Some("Café 👩🏽‍🚀\nSecond line"));
        assert_eq!(field.label(), Some("Notes"));
        assert!(!field.is_read_only(), "text fields remain editable");
        assert!(field.children().is_empty(), "app textboxes are leaves");
        assert_eq!(tree.focus, A11yNodeId(focus), "focus stays on the textbox");
        assert!(drawn_children.len() >= 4, "painting DOM remains intact");
        assert!(
            drawn_descendants
                .iter()
                .all(|id| !action_map.contains_key(id))
        );
        let title = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Title"))
            .map(|(_, node)| node)
            .expect("the single-line app field is projected");
        assert_eq!(title.role(), Role::TextInput);
        assert_eq!(title.value(), Some("Résumé 🇫🇷"));
        let button = tree
            .nodes
            .iter()
            .find(|(_, node)| node.role() == Role::Button)
            .map(|(_, node)| node)
            .expect("the non-field marker preserves its button role");
        assert_eq!(button.value(), None);
    }

    #[test]
    fn focus_inside_a_nested_marked_textbox_resolves_to_the_surviving_outer_leaf() {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(
            root,
            "<div id=\"outer\" aria-label=\"Outer\" role=\"textbox\" data-cambium-text-value=\"outer value\" style=\"display:block;width:240px;height:60px\"><div id=\"inner\" aria-label=\"Inner\" role=\"textbox\" data-cambium-text-value=\"inner value\" style=\"display:block;width:200px;height:32px\"><span id=\"focused-child\">preedit</span></div></div>",
        );
        let find_id = |wanted: &str| {
            let mut pending = vec![dom.document()];
            while let Some(node) = pending.pop() {
                if dom.attribute(node, &Namespace::default(), &LocalName::from("id"))
                    == Some(wanted)
                {
                    return node;
                }
                pending.extend(dom.dom_children(node));
            }
            panic!("missing fixture node with id {wanted}");
        };
        let outer = find_id("outer");
        let inner = find_id("inner");
        let focused_child = find_id("focused-child");
        let outer_id = A11yNodeId(dom.opaque_id(outer));
        let inner_id = A11yNodeId(dom.opaque_id(inner));
        let focused_opaque = dom.opaque_id(focused_child);
        let focused_id = A11yNodeId(focused_opaque);
        let outer_drawn_children: Vec<_> = dom.dom_children(outer).collect();
        let layout = OwnedLayout::new(&dom, &[""], 320.0, 180.0, &[], &HashMap::new());
        let mut leaves = LeafRegistry::new();
        let (tree, action_map, _) = project_tree_with_actions(
            &dom,
            &layout,
            &mut leaves,
            &mut ProducerRegistry::new(),
            Some(focused_opaque),
        );

        let outer_node = tree
            .nodes
            .iter()
            .find(|(id, _)| *id == outer_id)
            .map(|(_, node)| node)
            .expect("the outer textbox survives");
        assert_eq!(outer_node.value(), Some("outer value"));
        assert!(outer_node.children().is_empty());
        assert_eq!(tree.focus, outer_id);
        assert!(!tree.nodes.iter().any(|(id, _)| *id == inner_id));
        assert!(action_map.contains_key(&outer_id));
        assert!(!action_map.contains_key(&inner_id));
        assert!(!action_map.contains_key(&focused_id));
        assert_eq!(
            dom.dom_children(outer).collect::<Vec<_>>(),
            outer_drawn_children,
            "accessibility pruning leaves drawing DOM intact"
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
            fn semantics(&self) -> Option<ProducerSemantics> {
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
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(root, "<custom-leaf key='1' style='position:absolute;left:10px;top:100px;width:990px;height:600px;padding:7px;border:5px solid black'></custom-leaf>");
        let layout = OwnedLayout::new(&dom, &[""], 1200.0, 900.0, &[], &HashMap::new());
        let slot_node = {
            let mut pending = vec![root];
            loop {
                let node = pending.pop().expect("the fixture has a producer slot");
                if dom.attribute(node, &Namespace::default(), &LocalName::from("key")) == Some("1")
                {
                    break node;
                }
                pending.extend(dom.flat_children(node));
            }
        };
        let slot = A11yNodeId(dom.opaque_id(slot_node));
        let acted = Rc::new(RefCell::new(Vec::new()));
        let mut producers = ProducerRegistry::new();
        producers.register(1, Board(acted.clone()), &[]).unwrap();
        let (tree, _, produced_map) = project_tree_with_actions(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut producers,
            None,
        );

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
            Some(Rect {
                x0: 46.0,
                y0: 136.0,
                x1: 166.0,
                y1: 216.0
            }),
            "placed where it is drawn, inside the slot's border and padding"
        );
        assert_eq!(card(board.children()[1]).label(), Some("Card 1"));
        assert!(
            card(board.children()[1]).children().is_empty(),
            "no actions, no buttons"
        );

        // Card 0's one action is a Button child taking Click, named by its
        // label and described by its description.
        assert_eq!(first.children().len(), 1);
        let pin_id = first.children()[0];
        let pin = card(pin_id);
        assert_eq!(pin.role(), Role::Button);
        assert_eq!(pin.label(), Some("Pin"));
        assert_eq!(pin.description(), Some("Hold this card"));
        assert!(pin.supports_action(Action::Click));
        assert_eq!(pin.bounds(), first.bounds());

        // A reader's click on it maps to the drawn node's action and reaches
        // the producer, which carries it out; the card itself is no button.
        let mut host = A11yHost::new(|| {});
        host.requests = produced_map
            .into_iter()
            .map(|(id, action)| (id, A11yRequest::produced(A11yAction::Focus, action)))
            .collect();
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
        assert_eq!(
            request,
            A11yRequest::produced(A11yAction::Click, produced.clone())
        );
        assert!(producers.act(&produced), "the producer carried it out");
        assert_eq!(acted.borrow().as_slice(), &[(40, "pin".to_string())]);
        assert_eq!(host.map_request(&click(board.children()[0])), None);
        // An unknown slot reaches no producer.
        assert!(!producers.act(&ProducedAction {
            slot: 9,
            ..produced
        }));
        // A producer without semantics leaves its slot alone.
        assert_eq!(producers.semantics(2), None);
    }
    #[test]
    fn native_projection_uses_shared_bounds_and_disabled_numeric_policy() {
        use layout_dom_api::{LayoutDom as _, LocalName, Namespace};
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(root, "<button aria-label='Off' disabled data-cambium-set-value='true' style='display:block;width:80px;height:30px'>Off</button><button aria-label='On' data-cambium-set-value='true' style='display:block;width:80px;height:30px'>On</button><div style='height:600px'></div>");
        let layout = OwnedLayout::new(&dom, &[""], 320.0, 200.0, &[], &HashMap::new());
        let projection = cambium_rootstock::document_projection(&dom, &layout, None);
        let (tree, map, _) = project_tree_with_actions(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut ProducerRegistry::new(),
            None,
        );
        for semantic in projection.nodes() {
            let (_, native) = tree
                .nodes
                .iter()
                .find(|(id, _)| id.0 == semantic.id.get())
                .unwrap();
            if let Some(bounds) = semantic.bounds {
                let native = native.bounds().unwrap();
                assert_eq!(
                    (native.x0, native.y0, native.width(), native.height()),
                    (
                        f64::from(bounds.x),
                        f64::from(bounds.y),
                        f64::from(bounds.width),
                        f64::from(bounds.height)
                    )
                );
            }
            assert_eq!(
                native.supports_action(Action::SetValue),
                semantic
                    .actions
                    .contains(&document_session_api::DocumentA11yAction::SetValue)
            );
        }
        let disabled = tree
            .nodes
            .iter()
            .find(|(_, node)| node.label() == Some("Off"))
            .unwrap();
        assert!(!disabled.1.supports_action(Action::SetValue));
        assert!(map.values().all(|node| dom.attribute(
            *node,
            &Namespace::default(),
            &LocalName::from("aria-hidden")
        ) != Some("true")));
    }
    #[test]
    fn native_routes_and_shared_overrides_follow_the_supplier_flat_tree() {
        use layout_dom_api::{LayoutDom as _, ShadowRootInit};
        let mut dom = ScriptedDom::new();
        let document = dom.document();
        dom.set_inner_html(
            document,
            "<main style='display:block;width:100px;height:100px'></main>",
        );
        let host = dom
            .dom_children(document)
            .find(|node| dom.kind(*node) == NodeKind::Element)
            .unwrap();
        let shadow = dom.attach_shadow_unchecked(host, ShadowRootInit::default(), false);
        dom.set_inner_html(shadow, "<button aria-label='Shadow' data-cambium-set-value='true' style='display:block;width:80px;height:30px'>Shadow</button>");
        let button = dom
            .dom_children(shadow)
            .find(|node| dom.kind(*node) == NodeKind::Element)
            .unwrap();
        let opaque = dom.opaque_id(button);
        let layout = OwnedLayout::new(&dom, &[""], 320.0, 200.0, &[], &HashMap::new());
        let projection = cambium_rootstock::document_projection(&dom, &layout, Some(opaque));
        let semantic = projection
            .nodes()
            .iter()
            .find(|node| node.id.get() == opaque)
            .expect("shadow button projects");
        assert!(semantic.state.focused);
        assert!(
            semantic
                .actions
                .contains(&document_session_api::DocumentA11yAction::SetValue)
        );
        let (tree, map, _) = project_tree_with_actions(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut ProducerRegistry::new(),
            Some(opaque),
        );
        assert_eq!(map.get(&A11yNodeId(opaque)), Some(&button));
        assert_eq!(tree.focus, A11yNodeId(opaque));
    }
    #[test]
    fn flat_tree_tests_native_action_map_contains_only_the_assigned_producer() {
        use cambium_rootstock::{
            ProducedTexture, ProducerAction, ProducerContext, ProducerNode, TextureProducer,
        };
        use layout_dom_api::ShadowRootInit;
        struct Board;
        impl TextureProducer for Board {
            fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
                None
            }
            fn semantics(&self) -> Option<ProducerSemantics> {
                Some(ProducerSemantics {
                    role: Some(ProducerRole::Group),
                    name: Some("Slotted board".into()),
                    children: vec![ProducerNode {
                        key: 40,
                        role: ProducerRole::ListItem,
                        name: "Card".into(),
                        rect: [0.0, 0.0, 8.0, 6.0],
                        actions: vec![ProducerAction {
                            id: "pin".into(),
                            label: "Pin".into(),
                            description: "Keep card".into(),
                        }],
                    }],
                })
            }
            fn act(&mut self, key: u64, id: &str) -> bool {
                key == 40 && id == "pin"
            }
        }
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(root,"<section><custom-leaf id='active' key='7' slot='scene' style='display:block;width:8px;height:6px'></custom-leaf><custom-leaf id='decoy' key='7'></custom-leaf></section>");
        let host = dom
            .dom_children(root)
            .find(|node| dom.kind(*node) == NodeKind::Element)
            .unwrap();
        let leaf = dom
            .dom_children(host)
            .find(|node| {
                dom.attribute(*node, &Namespace::default(), &LocalName::from("id"))
                    == Some("active")
            })
            .unwrap();
        let decoy = dom
            .dom_children(host)
            .find(|node| {
                dom.attribute(*node, &Namespace::default(), &LocalName::from("id")) == Some("decoy")
            })
            .unwrap();
        let shadow = dom.attach_shadow_unchecked(host, ShadowRootInit::default(), false);
        dom.set_inner_html(shadow,"<slot name='scene'><custom-leaf key='7' style='display:block;width:8px;height:6px'></custom-leaf></slot>");
        let layout = OwnedLayout::new(&dom, &[""], 160.0, 120.0, &[], &HashMap::new());
        let mut producers = ProducerRegistry::new();
        producers.register(7, Board, &[]).unwrap();
        let (_, map, produced) = project_tree_with_actions(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut producers,
            None,
        );
        assert_eq!(map.get(&A11yNodeId(dom.opaque_id(leaf))), Some(&leaf));
        assert!(!map.contains_key(&A11yNodeId(dom.opaque_id(decoy))));
        assert_eq!(produced.len(), 1);
        let (&button, target) = produced.iter().next().unwrap();
        let expected = target.clone();
        let mut host = A11yHost::new(|| {});
        host.requests = produced
            .into_iter()
            .map(|(id, action)| (id, A11yRequest::produced(A11yAction::Focus, action)))
            .collect();
        let request = host
            .map_request(&ActionRequest {
                action: Action::Click,
                target_tree: TreeId::ROOT,
                target_node: button,
                data: None,
            })
            .unwrap();
        assert_eq!(
            request,
            A11yRequest::produced(A11yAction::Click, expected.clone())
        );
        assert!(producers.act(&expected));
    }

    #[test]
    fn identity_tests_native_tree_omits_ambiguous_producer_action_routes() {
        use cambium_rootstock::{
            ProducedTexture, ProducerAction, ProducerContext, ProducerNode, TextureProducer,
        };
        struct Board(ProducerSemantics);
        impl TextureProducer for Board {
            fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
                None
            }
            fn semantics(&self) -> Option<ProducerSemantics> {
                Some(self.0.clone())
            }
        }
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(
            root,
            "<custom-leaf key='7' style='display:block;width:80px;height:30px'></custom-leaf>",
        );
        let layout = OwnedLayout::new(&dom, &[""], 160.0, 120.0, &[], &HashMap::new());
        let action = ProducerAction {
            id: "pin".into(),
            label: "Pin".into(),
            description: "Keep card".into(),
        };
        let child = ProducerNode {
            key: 40,
            role: ProducerRole::ListItem,
            name: "Card".into(),
            rect: [0.0, 0.0, 10.0, 10.0],
            actions: vec![action.clone(), action],
        };
        let mut producers = ProducerRegistry::new();
        producers
            .register(
                7,
                Board(ProducerSemantics {
                    children: vec![child],
                    ..Default::default()
                }),
                &[],
            )
            .unwrap();
        let (tree, _, produced) = project_tree_with_actions(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut producers,
            None,
        );
        assert!(produced.is_empty());
        assert!(
            !tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Pin"))
        );
        let ids: std::collections::HashSet<_> = tree.nodes.iter().map(|(id, _)| id).collect();
        assert_eq!(ids.len(), tree.nodes.len());
    }
}
