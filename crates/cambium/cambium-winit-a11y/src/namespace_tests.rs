// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::publication::{NativeIds, Prepared, Refusal};
use super::*;
use cambium_rootstock::{
    A11yOwner, OwnedLayout, ProducerChildLifetime, ProducerNode, ProducerRole, ProducerSemantics,
    SemanticProjection, TextureProducer, WindowDom,
};
use genet_scripted_dom::ScriptedDom;
use layout_dom_api::LayoutDomMut;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

struct Fixture {
    dom: ScriptedDom,
    layout: OwnedLayout,
    leaves: LeafRegistry<u64>,
    producers: ProducerRegistry,
    marker: Rc<Cell<bool>>,
}

#[test]
fn detached_native_publisher_reports_no_window_and_revoked_owner_clears_routes() {
    let f = Fixture::new();
    let mut host = A11yHost::new(|| {});
    f.publish(&mut host);
    let sync = host.sync(
        &f.owner(),
        &WindowDom::document(&f.dom),
        &f.layout,
        &f.leaves,
        &f.producers,
        None,
        1.0,
    );
    assert!(sync.requests.is_empty());
    assert_eq!(
        sync.publication,
        A11yPublication::skipped(A11yAdapterState::Detached, A11yPublicationSkip::NoWindow)
    );
    f.marker.set(false);
    let sync = host.sync(
        &f.owner(),
        &WindowDom::document(&f.dom),
        &f.layout,
        &f.leaves,
        &f.producers,
        None,
        1.0,
    );
    assert_eq!(
        sync.publication.outcome,
        A11yPublicationOutcome::NotAttempted {
            reason: A11yPublicationSkip::OwnerEnded
        }
    );
    assert!(host.requests.is_empty());
}
impl Fixture {
    fn new() -> Self {
        let mut dom = ScriptedDom::new();
        dom.set_inner_html(
            dom.document(),
            "<custom-leaf key='7' style='display:block;width:80px;height:30px'></custom-leaf>",
        );
        let layout = OwnedLayout::new(
            &WindowDom::document(&dom),
            &[""],
            320.,
            240.,
            &[],
            &HashMap::new(),
        );
        let mut leaves = LeafRegistry::new();
        leaves.insert(7, knob());
        Self {
            dom,
            layout,
            leaves,
            producers: ProducerRegistry::new(),
            marker: Rc::new(Cell::new(true)),
        }
    }
    fn owner(&self) -> A11yOwner {
        A11yOwner::new(&self.marker)
    }
    fn projection(&self) -> SemanticProjection {
        cambium_rootstock::project_semantics(
            &WindowDom::document(&self.dom),
            &self.layout,
            &self.leaves,
            &self.producers,
            None,
        )
    }
    fn prepare(&self, ids: &NativeIds) -> Prepared {
        let projection = self.projection();
        let bindings = projection
            .publication_bindings(
                &self.owner(),
                &WindowDom::document(&self.dom),
                &self.leaves,
                &self.producers,
            )
            .unwrap();
        ids.prepare(projection.tree, bindings).unwrap()
    }
    fn publish(&self, host: &mut A11yHost) -> TreeUpdate {
        let Prepared {
            tree,
            namespace,
            requests,
        } = self.prepare(&host.namespace);
        host.namespace = namespace;
        host.requests = requests;
        tree
    }
    fn slot_id(&self, host: &A11yHost) -> A11yNodeId {
        let slot = self.dom.dom_children(self.dom.document()).next().unwrap();
        *host
            .requests
            .iter()
            .find(|(_, r)| r.target == A11yTarget::Node(slot))
            .unwrap()
            .0
    }
}
fn knob() -> Box<dyn sprigging::Leaf> {
    Box::new(sprigging::Knob::new(sprigging::Size {
        width: 80.,
        height: 30.,
    }))
}

struct Ignore;
impl accesskit_consumer::TreeChangeHandler for Ignore {
    fn node_added(&mut self, _: &accesskit_consumer::NodeRef<'_>) {}
    fn node_updated(
        &mut self,
        _: &accesskit_consumer::NodeRef<'_>,
        _: &accesskit_consumer::NodeRef<'_>,
    ) {
    }
    fn focus_moved(
        &mut self,
        _: Option<&accesskit_consumer::NodeRef<'_>>,
        _: Option<&accesskit_consumer::NodeRef<'_>>,
    ) {
    }
    fn node_removed(&mut self, _: &accesskit_consumer::NodeRef<'_>) {}
}

#[test]
fn native_namespace_retires_replaced_leaf_ids_and_routes_late_events_without_aliasing() {
    let mut f = Fixture::new();
    let mut host = A11yHost::new(|| {});
    let first = f.publish(&mut host);
    let mut consumer = accesskit_consumer::Tree::new(first.clone(), true);
    let old_id = f.slot_id(&host);
    let old_request = host
        .map_parts(Action::Focus, None, accesskit::TreeId::ROOT, old_id)
        .unwrap();
    f.leaves
        .get_mut_as::<sprigging::Knob>(&7)
        .unwrap()
        .set_value(2.);
    let ordinary = f.publish(&mut host);
    assert_eq!(old_id, f.slot_id(&host));
    assert_eq!(
        first.tree.as_ref().unwrap().root,
        ordinary.tree.as_ref().unwrap().root
    );
    consumer.update_and_process_changes(ordinary, &mut Ignore);
    f.leaves.insert(7, knob());
    let pending = host
        .map_parts(Action::Focus, None, accesskit::TreeId::ROOT, old_id)
        .unwrap();
    assert!(
        pending.same_publication_target(&old_request),
        "drain through the last actual native publication until replacement commits"
    );
    let replacement = f.publish(&mut host);
    let new_id = f.slot_id(&host);
    assert_ne!(old_id, new_id);
    // The old OS event arrives after the current publication replaced its maps.
    assert!(
        host.map_parts(Action::Focus, None, accesskit::TreeId::ROOT, old_id)
            .is_none()
    );
    let current = host
        .map_parts(
            Action::SetValue,
            Some(ActionData::NumericValue(4.)),
            accesskit::TreeId::ROOT,
            new_id,
        )
        .unwrap();
    assert!(!old_request.same_publication_target(&current));
    consumer.update_and_process_changes(replacement, &mut Ignore);
}

#[test]
fn native_namespace_changes_owner_and_never_reuses_ids_after_host_recreation() {
    let mut f = Fixture::new();
    let mut host = A11yHost::new(|| {});
    let before = f.publish(&mut host);
    let old_ids: std::collections::HashSet<_> = before.nodes.iter().map(|(id, _)| *id).collect();
    let old_marker = f.marker.clone();
    f.marker = Rc::new(Cell::new(true));
    let after = f.publish(&mut host);
    assert!(old_marker.get());
    assert!(after.nodes.iter().all(|(id, _)| !old_ids.contains(id)));
    for id in old_ids {
        assert!(
            host.map_parts(Action::Focus, None, accesskit::TreeId::ROOT, id)
                .is_none()
        );
    }
    let last_ids: std::collections::HashSet<_> = after.nodes.iter().map(|(id, _)| *id).collect();
    let mut consumer = accesskit_consumer::Tree::new(before, true);
    consumer.update_and_process_changes(after, &mut Ignore);
    drop(host);
    let recreated = f.publish(&mut A11yHost::new(|| {}));
    assert!(recreated.nodes.iter().all(|(id, _)| !last_ids.contains(id)));
}

struct Structural {
    life: Rc<RefCell<ProducerChildLifetime>>,
    shown: Rc<Cell<bool>>,
    action: bool,
}
impl TextureProducer for Structural {
    fn render(
        &mut self,
        _: &cambium_rootstock::ProducerContext<'_>,
    ) -> Option<cambium_rootstock::ProducedTexture> {
        None
    }
    fn child_lifetime(&self, _: u64) -> Option<cambium_rootstock::ProducerChildToken> {
        Some(self.life.borrow().token())
    }
    fn semantics(&self) -> Option<ProducerSemantics> {
        Some(ProducerSemantics {
            children: if self.shown.get() {
                vec![ProducerNode {
                    key: 40,
                    role: ProducerRole::ListItem,
                    name: "Structural".into(),
                    rect: [0., 0., 10., 10.],
                    actions: if self.action {
                        vec![cambium_rootstock::ProducerAction {
                            id: "pin".into(),
                            label: "Pin".into(),
                            description: "Keep child".into(),
                        }]
                    } else {
                        vec![]
                    },
                }]
            } else {
                vec![]
            },
            ..Default::default()
        })
    }
}
fn structural(tree: &TreeUpdate) -> A11yNodeId {
    tree.nodes
        .iter()
        .find(|(_, n)| n.label() == Some("Structural"))
        .unwrap()
        .0
}
#[test]
fn native_namespace_covers_structural_children_and_retires_withdrawn_publications() {
    let mut f = Fixture::new();
    let life = Rc::new(RefCell::new(ProducerChildLifetime::new()));
    let shown = Rc::new(Cell::new(true));
    f.producers
        .register(
            7,
            Structural {
                life: life.clone(),
                shown: shown.clone(),
                action: false,
            },
            &[],
        )
        .unwrap();
    let mut host = A11yHost::new(|| {});
    let first = structural(&f.publish(&mut host));
    assert!(
        !host.requests.contains_key(&first),
        "a structural node has identity without an action route"
    );
    assert_eq!(first, structural(&f.publish(&mut host)));
    let old_life = life.borrow().clone();
    *life.borrow_mut() = ProducerChildLifetime::new();
    let second = structural(&f.publish(&mut host));
    assert_ne!(
        first, second,
        "retained old child marker cannot authorize replacement"
    );
    shown.set(false);
    f.publish(&mut host);
    shown.set(true);
    assert_ne!(
        second,
        structural(&f.publish(&mut host)),
        "withdrawn IDs never return"
    );
    drop(old_life);
}

#[test]
fn native_namespace_remaps_relations_selection_root_and_focus_in_one_snapshot() {
    let f = Fixture::new();
    let mut projection = f.projection();
    let root = projection.tree.tree.as_ref().unwrap().root;
    let slot = projection
        .tree
        .nodes
        .iter()
        .find(|(id, _)| *id != root)
        .unwrap()
        .0;
    projection.tree.focus = slot;
    let node = &mut projection
        .tree
        .nodes
        .iter_mut()
        .find(|(id, _)| *id == slot)
        .unwrap()
        .1;
    node.set_controls(vec![root]);
    node.set_details(vec![root]);
    node.set_described_by(vec![root]);
    node.set_flow_to(vec![root]);
    node.set_labelled_by(vec![root]);
    node.set_owns(vec![root]);
    node.set_radio_group(vec![root]);
    node.set_active_descendant(root);
    node.set_error_message(root);
    node.set_in_page_link_target(root);
    node.set_member_of(root);
    node.set_next_on_line(root);
    node.set_previous_on_line(root);
    node.set_popup_for(root);
    node.set_text_selection(accesskit::TextSelection {
        anchor: accesskit::TextPosition {
            node: root,
            character_index: 2,
        },
        focus: accesskit::TextPosition {
            node: slot,
            character_index: 3,
        },
    });
    let bindings = projection
        .publication_bindings(
            &f.owner(),
            &WindowDom::document(&f.dom),
            &f.leaves,
            &f.producers,
        )
        .unwrap();
    let prepared = NativeIds::default()
        .prepare(projection.tree, bindings)
        .unwrap();
    let new_root = prepared.tree.tree.as_ref().unwrap().root;
    let new_slot = prepared.tree.focus;
    assert_ne!(new_root, new_slot);
    let root = &prepared
        .tree
        .nodes
        .iter()
        .find(|(id, _)| *id == new_root)
        .unwrap()
        .1;
    assert_eq!(root.children(), &[new_slot]);
    let node = &prepared
        .tree
        .nodes
        .iter()
        .find(|(id, _)| *id == new_slot)
        .unwrap()
        .1;
    for relation in [
        node.controls(),
        node.details(),
        node.described_by(),
        node.flow_to(),
        node.labelled_by(),
        node.owns(),
        node.radio_group(),
    ] {
        assert_eq!(relation, &[new_root]);
    }
    for relation in [
        node.active_descendant(),
        node.error_message(),
        node.in_page_link_target(),
        node.member_of(),
        node.next_on_line(),
        node.previous_on_line(),
        node.popup_for(),
    ] {
        assert_eq!(relation, Some(new_root));
    }
    assert_eq!(node.text_selection().unwrap().anchor.node, new_root);
    assert_eq!(node.text_selection().unwrap().focus.node, new_slot);
    assert_eq!(node.text_selection().unwrap().focus.character_index, 3);
}

#[test]
fn native_namespace_refuses_bad_references_and_foreign_trees_without_changing_current_ids() {
    let f = Fixture::new();
    let first = f.prepare(&NativeIds::default());
    let baseline: Vec<_> = first.tree.nodes.iter().map(|(id, _)| *id).collect();
    for case in 0..6 {
        let mut projection = f.projection();
        let mut bindings = projection
            .publication_bindings(
                &f.owner(),
                &WindowDom::document(&f.dom),
                &f.leaves,
                &f.producers,
            )
            .unwrap();
        let first_id = projection.tree.nodes[0].0;
        match case {
            0 => projection.tree.nodes[0]
                .1
                .set_controls(vec![A11yNodeId(u64::MAX)]),
            1 => projection.tree.tree_id = accesskit::TreeId(accesskit::Uuid::from_u128(99)),
            2 => projection.tree.nodes[0]
                .1
                .set_tree_id(accesskit::TreeId::ROOT),
            3 => projection.tree.nodes.push(projection.tree.nodes[0].clone()),
            4 => {
                bindings.identities.remove(&projection.tree.nodes[0].0);
            },
            _ => projection.tree.nodes[0].1.push_child(first_id),
        }
        let result = first.namespace.prepare(projection.tree, bindings);
        assert!(matches!(
            result,
            Err(Refusal::DanglingReference(_)
                | Refusal::ForeignTree
                | Refusal::DuplicateNode(_)
                | Refusal::MissingIdentity(_)
                | Refusal::NotATree(_))
        ));
        let after = f.prepare(&first.namespace);
        assert_eq!(
            baseline,
            after
                .tree
                .nodes
                .iter()
                .map(|(id, _)| *id)
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn shared_publication_refuses_route_metadata_with_a_different_authored_target() {
    let f = Fixture::new();
    let mut projection = f.projection();
    let id = *projection.dom_targets.keys().next().unwrap();
    let other = f.dom.dom_children(f.dom.document()).next().unwrap();
    let wrong = if id == A11yNodeId(f.dom.opaque_id(other)) {
        f.dom.document()
    } else {
        other
    };
    projection.dom_targets.insert(id, wrong);
    assert!(matches!(
        projection.publication_bindings(
            &f.owner(),
            &WindowDom::document(&f.dom),
            &f.leaves,
            &f.producers
        ),
        Err(cambium_rootstock::PublicationIssue::RouteMismatch(_))
    ));
}

#[test]
fn native_producer_action_ids_change_with_child_life_and_registration() {
    let mut f = Fixture::new();
    let life = Rc::new(RefCell::new(ProducerChildLifetime::new()));
    let shown = Rc::new(Cell::new(true));
    let producer = || Structural {
        life: life.clone(),
        shown: shown.clone(),
        action: true,
    };
    f.producers.register(7, producer(), &[]).unwrap();
    let mut host = A11yHost::new(|| {});
    f.publish(&mut host);
    let action = |host: &A11yHost| {
        *host
            .requests
            .iter()
            .find(|(_, r)| matches!(r.target, A11yTarget::Produced(_)))
            .unwrap()
            .0
    };
    let old = action(&host);
    let slot = f.slot_id(&host);
    f.publish(&mut host);
    assert_eq!(old, action(&host));
    let retained_old_child = life.borrow().clone();
    *life.borrow_mut() = ProducerChildLifetime::new();
    f.publish(&mut host);
    let current = action(&host);
    assert_ne!(old, current);
    assert_eq!(
        slot,
        f.slot_id(&host),
        "unchanged slot owner keeps its native ID"
    );
    assert!(
        host.map_parts(Action::Click, None, accesskit::TreeId::ROOT, old)
            .is_none()
    );
    let request = host
        .map_parts(Action::Click, None, accesskit::TreeId::ROOT, current)
        .unwrap();
    assert!(
        matches!(request.target,A11yTarget::Produced(ref target) if target.slot == 7 && target.key == 40 && target.id == "pin")
    );
    assert!(
        host.map_parts(
            Action::Click,
            None,
            accesskit::TreeId(accesskit::Uuid::from_u128(99)),
            current
        )
        .is_none()
    );
    f.producers.remove(7);
    f.producers.register(7, producer(), &[]).unwrap();
    f.publish(&mut host);
    assert_ne!(current, action(&host));
    assert!(
        host.map_parts(Action::Click, None, accesskit::TreeId::ROOT, current)
            .is_none()
    );
    drop(retained_old_child);
}
