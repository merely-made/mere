// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{
    ProducedTexture, ProducerAction, ProducerContext, ProducerNode, ScriptedDom, TextureProducer,
};
use layout_dom_api::LayoutDomMut;
use sprigging::{Leaf, PaintCx, Size, SizeHint};
use std::{cell::RefCell, rc::Rc};

struct Describer(Rc<RefCell<usize>>);
impl Leaf for Describer {
    fn measure(&mut self, _: SizeHint, _: SizeHint) -> Size {
        Size {
            width: 80.0,
            height: 30.0,
        }
    }
    fn paint(&mut self, _: &mut PaintCx<'_>) {}
    fn paint_dirty(&self) -> bool {
        false
    }
    fn accessibility(&mut self, node: &mut accesskit::Node) {
        *self.0.borrow_mut() += 1;
        assert_eq!(node.label(), Some("Author"), "a fully seeded callback");
        assert!(node.bounds().is_some());
        // A contributor can replace its role, but cannot erase host restrictions,
        // geometry, DOM edges or the author's explicit name.
        *node = accesskit::Node::new(Role::Slider);
        node.set_label("Contributor");
        node.set_bounds(Rect::new(900.0, 900.0, 901.0, 901.0));
        node.set_transform(accesskit::Affine::scale(2.0));
        node.push_child(A11yNodeId(999));
        node.set_numeric_value(0.5);
        node.add_action(Action::SetValue);
        node.add_action(Action::Click);
    }
}
struct Board;
impl TextureProducer for Board {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
    fn semantics(&mut self) -> Option<ProducerSemantics> {
        Some(board())
    }
}
struct Described(ProducerSemantics);
impl TextureProducer for Described {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }
    fn semantics(&mut self) -> Option<ProducerSemantics> {
        Some(self.0.clone())
    }
}
fn board() -> ProducerSemantics {
    ProducerSemantics {
        role: Some(ProducerRole::List),
        name: Some("Board".into()),
        children: vec![ProducerNode {
            key: 40,
            role: ProducerRole::ListItem,
            name: "Card".into(),
            rect: [4.0, 5.0, 10.0, 20.0],
            actions: vec![ProducerAction {
                id: "pin".into(),
                label: "Pin".into(),
                description: "Keep card".into(),
            }],
        }],
    }
}
fn fixture(extra: &str) -> (ScriptedDom, OwnedLayout, NodeId) {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root, &format!("<custom-leaf key='7' aria-label='Author' {extra} style='display:block;width:80px;height:30px'></custom-leaf>"));
    let layout = OwnedLayout::new(&dom, &[""], 320.0, 200.0, &[], &HashMap::new());
    let slot = layout.custom_leaf_nodes(&dom)[0].1;
    (dom, layout, slot)
}
#[test]
fn seeded_leaf_cannot_erase_host_name_geometry_or_restrictions() {
    for (extra, disabled) in [
        ("aria-disabled='true'", true),
        ("aria-readonly='true'", false),
    ] {
        let (dom, layout, slot) = fixture(extra);
        let calls = Rc::new(RefCell::new(0));
        let mut leaves = LeafRegistry::new();
        leaves.insert(7, Box::new(Describer(calls.clone())));
        let projection = project_semantics(
            &dom,
            &layout,
            &mut leaves,
            &mut ProducerRegistry::new(),
            None,
        );
        let node = &projection
            .tree
            .nodes
            .iter()
            .find(|(id, _)| id.0 == dom.opaque_id(slot))
            .unwrap()
            .1;
        assert_eq!(*calls.borrow(), 1);
        assert_eq!(node.role(), Role::Slider);
        assert_eq!(node.label(), Some("Author"));
        assert_eq!(node.bounds().unwrap().width(), 80.0);
        assert!(node.transform().is_none());
        assert!(node.children().is_empty());
        assert_eq!(node.numeric_value(), Some(0.5));
        assert!(!node.supports_action(Action::SetValue));
        if disabled {
            assert!(node.is_disabled());
            assert!(!node.supports_action(Action::Click));
        } else {
            assert!(node.is_read_only());
            assert!(node.supports_action(Action::Click));
        }
    }
}
#[test]
fn producer_precedes_leaf_and_disabled_slot_has_no_action_targets() {
    for disabled in [false, true] {
        let (dom, layout, slot) = fixture(if disabled { "aria-disabled='true'" } else { "" });
        let calls = Rc::new(RefCell::new(0));
        let mut leaves = LeafRegistry::new();
        leaves.insert(7, Box::new(Describer(calls.clone())));
        let mut producers = ProducerRegistry::new();
        producers.register(7, Board, &[]).unwrap();
        let projection = project_semantics(&dom, &layout, &mut leaves, &mut producers, None);
        let access = &projection
            .tree
            .nodes
            .iter()
            .find(|(id, _)| id.0 == dom.opaque_id(slot))
            .unwrap()
            .1;
        assert_eq!(*calls.borrow(), 0);
        assert_eq!(access.label(), Some("Board"));
        assert_eq!(projection.producer_slots.get(&7), Some(&slot));
        let child = &projection
            .tree
            .nodes
            .iter()
            .find(|(id, _)| *id == access.children()[0])
            .unwrap()
            .1;
        let button = &projection
            .tree
            .nodes
            .iter()
            .find(|(id, _)| *id == child.children()[0])
            .unwrap()
            .1;
        assert_eq!(child.is_disabled(), disabled);
        assert_eq!(button.is_disabled(), disabled);
        assert_eq!(button.supports_action(Action::Click), !disabled);
        assert_eq!(projection.produced_targets.len(), usize::from(!disabled));
    }
}
#[test]
fn producer_without_slot_geometry_does_not_invent_screen_coordinates() {
    let slot = A11yNodeId(3);
    let mut tree = TreeUpdate {
        nodes: vec![(slot, accesskit::Node::new(Role::Image))],
        tree: Some(accesskit::TreeInfo::new(slot)),
        tree_id: accesskit::TreeId::ROOT,
        focus: slot,
    };
    add_producer_semantics(
        &mut tree,
        slot,
        7,
        board(),
        Some((0.0, 0.0)),
        &mut HashMap::new(),
        &mut HashSet::from([slot]),
    )
    .unwrap();
    assert!(tree.nodes.iter().all(|(_, node)| node.bounds().is_none()));
}
#[test]
fn invalid_producer_rectangle_does_not_reach_accessibility_bounds() {
    let (dom, layout, slot) = fixture("");
    for rect in [
        [f32::NAN, 0.0, 10.0, 20.0],
        [0.0, 0.0, -1.0, 20.0],
        [0.0, f32::INFINITY, 10.0, 20.0],
    ] {
        let mut tree =
            SemanticProjection::from_document(crate::document_projection(&dom, &layout, None)).tree;
        let mut semantics = board();
        semantics.children[0].rect = rect;
        add_producer_semantics(
            &mut tree,
            A11yNodeId(dom.opaque_id(slot)),
            7,
            semantics,
            Some((0.0, 0.0)),
            &mut HashMap::new(),
            &mut HashSet::from([A11yNodeId(dom.opaque_id(slot))]),
        )
        .unwrap();
        assert!(
            tree.nodes
                .iter()
                .filter(|(id, _)| id.0 >= 1 << 63)
                .all(|(_, node)| node.bounds().is_none())
        );
    }
}

#[test]
fn hidden_leaf_retires_its_route_and_falls_back_from_hidden_focus() {
    struct Hidden;
    impl Leaf for Hidden {
        fn measure(&mut self, _: SizeHint, _: SizeHint) -> Size {
            Size {
                width: 80.0,
                height: 30.0,
            }
        }
        fn paint(&mut self, _: &mut PaintCx<'_>) {}
        fn paint_dirty(&self) -> bool {
            false
        }
        fn accessibility(&mut self, node: &mut accesskit::Node) {
            node.set_hidden();
        }
    }
    let (dom, layout, slot) = fixture("");
    let id = A11yNodeId(dom.opaque_id(slot));
    let mut leaves = LeafRegistry::new();
    leaves.insert(7, Box::new(Hidden));
    let projection = project_semantics(
        &dom,
        &layout,
        &mut leaves,
        &mut ProducerRegistry::new(),
        Some(id.0),
    );
    assert!(!projection.dom_targets.contains_key(&id));
    assert!(
        projection
            .tree
            .nodes
            .iter()
            .find(|(node, _)| *node == id)
            .unwrap()
            .1
            .is_hidden()
    );
    assert_eq!(
        projection.tree.focus,
        projection.tree.tree.as_ref().unwrap().root
    );
}

#[test]
fn duplicate_producer_dom_bindings_do_not_publish_actionable_children() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root,"<custom-leaf key='7' style='display:block;width:80px;height:30px'></custom-leaf><custom-leaf key='7' style='display:block;width:80px;height:30px'></custom-leaf>");
    let layout = OwnedLayout::new(&dom, &[""], 320.0, 200.0, &[], &HashMap::new());
    let mut producers = ProducerRegistry::new();
    producers.register(7, Board, &[]).unwrap();
    let projection = project_semantics(
        &dom,
        &layout,
        &mut LeafRegistry::new(),
        &mut producers,
        None,
    );
    assert!(projection.producer_slots.is_empty());
    assert!(projection.produced_targets.is_empty());
    assert!(
        !projection
            .tree
            .nodes
            .iter()
            .any(|(_, node)| node.label() == Some("Board") || node.label() == Some("Pin"))
    );
}

#[test]
fn content_origin_tests_padded_bounds_follow_content_and_both_scroll_planes() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root, "<main style='display:block;width:800px;height:800px'><section style='position:absolute;left:20px;top:30px;width:120px;height:70px;overflow:scroll'><custom-leaf key='7' style='position:absolute;left:50px;top:60px;display:block;width:80px;height:40px;padding:7px;border:5px solid black'></custom-leaf><div style='width:400px;height:400px'></div></section></main>");
    let mut layout = OwnedLayout::new(&dom, &[""], 160.0, 120.0, &[], &HashMap::new());
    let slot = layout.custom_leaf_nodes(&dom)[0].1;
    let scroller = dom.parent(slot).unwrap();
    layout.set_element_scroll(&dom, HashMap::from([(scroller, (3.0, 4.0))]));
    layout.set_viewport_scroll((5.0, 6.0));
    assert_eq!(layout.element_scroll()[&scroller], (3.0, 4.0));
    assert_eq!(layout.viewport_scroll(), (5.0, 6.0));
    let fragment = layout.fragments().get(slot).unwrap();
    let expected = Rect::new(
        f64::from(fragment.x + 12.0 - 3.0 - 5.0 + 4.0),
        f64::from(fragment.y + 12.0 - 4.0 - 6.0 + 5.0),
        f64::from(fragment.x + 12.0 - 3.0 - 5.0 + 14.0),
        f64::from(fragment.y + 12.0 - 4.0 - 6.0 + 25.0),
    );
    let mut producers = ProducerRegistry::new();
    producers.register(7, Board, &[]).unwrap();
    let projection = project_semantics(
        &dom,
        &layout,
        &mut LeafRegistry::new(),
        &mut producers,
        None,
    );
    let children: Vec<_> = projection
        .tree
        .nodes
        .iter()
        .filter(|(_, n)| matches!(n.label(), Some("Card" | "Pin")))
        .collect();
    assert_eq!(children.len(), 2);
    for (_, node) in children {
        assert_eq!(node.bounds(), Some(expected));
    }
    assert_eq!(projection.produced_targets.len(), 1);
    let local = layout
        .element_geometry(&dom, slot)
        .unwrap()
        .map_to_local(expected.x0 as f32 + 5.0, expected.y0 as f32 + 6.0)
        .unwrap();
    assert_eq!(local, (4.0, 5.0));
}

#[test]
fn identity_tests_duplicate_child_keys_and_action_ids_publish_no_partial_contribution() {
    let mut duplicate_key = board();
    duplicate_key
        .children
        .push(duplicate_key.children[0].clone());
    let mut duplicate_action = board();
    let action = duplicate_action.children[0].actions[0].clone();
    duplicate_action.children[0].actions.push(action);
    // A valid first child must not leak out when a later child is invalid.
    let mut later_invalid = board();
    let mut later = duplicate_action.children[0].clone();
    later.key = 41;
    later_invalid.children.push(later);
    for (semantics, issue) in [
        (
            duplicate_key,
            ProducerSemanticIssue::DuplicateChildKey {
                producer: 7,
                key: 40,
            },
        ),
        (
            duplicate_action,
            ProducerSemanticIssue::DuplicateActionId {
                producer: 7,
                key: 40,
                id: "pin".into(),
            },
        ),
        (
            later_invalid,
            ProducerSemanticIssue::DuplicateActionId {
                producer: 7,
                key: 41,
                id: "pin".into(),
            },
        ),
    ] {
        let (dom, layout, _) = fixture("");
        let mut producers = ProducerRegistry::new();
        producers.register(7, Described(semantics), &[]).unwrap();
        let projection = project_semantics(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut producers,
            None,
        );
        assert!(
            projection.produced_targets.is_empty(),
            "ambiguous declarations must not activate"
        );
        assert!(projection.producer_slots.is_empty());
        assert_eq!(projection.producer_issues, [issue]);
        assert!(
            !projection
                .tree
                .nodes
                .iter()
                .any(|(_, node)| node.label() == Some("Board"))
        );
        let ids: std::collections::HashSet<_> =
            projection.tree.nodes.iter().map(|(id, _)| id).collect();
        assert_eq!(ids.len(), projection.tree.nodes.len());
    }
}

#[test]
fn identity_tests_existing_node_and_action_id_collisions_preserve_the_original_tree() {
    let slot = A11yNodeId(3);
    for claimed in [
        produced_node_id(slot, 40, None),
        produced_node_id(slot, 40, Some("pin")),
    ] {
        let mut parent = accesskit::Node::new(Role::Group);
        parent.set_label("DOM owner");
        parent.set_children(vec![claimed]);
        let mut existing = accesskit::Node::new(Role::Button);
        existing.set_label("Existing target");
        let mut tree = TreeUpdate {
            nodes: vec![(slot, parent), (claimed, existing)],
            tree: Some(accesskit::TreeInfo::new(slot)),
            tree_id: accesskit::TreeId::ROOT,
            focus: claimed,
        };
        let before = tree.clone();
        let mut reserved = HashSet::from([slot, claimed]);
        let before_ids = reserved.clone();
        let mut targets = HashMap::new();
        let result = add_producer_semantics(
            &mut tree,
            slot,
            7,
            board(),
            Some((0.0, 0.0)),
            &mut targets,
            &mut reserved,
        );
        assert_eq!(
            result,
            Err(ProducerSemanticIssue::NodeIdCollision {
                producer: 7,
                id: claimed
            })
        );
        assert_eq!(tree, before);
        assert!(targets.is_empty());
        assert_eq!(
            reserved, before_ids,
            "a rejected prefix must not consume IDs"
        );
    }
}

#[test]
fn identity_tests_reordered_children_keep_distinct_targets_with_the_same_action_name() {
    let (dom, layout, _) = fixture("");
    let mut semantics = board();
    let mut child = semantics.children[0].clone();
    child.key = 41;
    child.name = "Second card".into();
    semantics.children.push(child);
    let project = |semantics| {
        let mut producers = ProducerRegistry::new();
        producers.register(7, Described(semantics), &[]).unwrap();
        project_semantics(
            &dom,
            &layout,
            &mut LeafRegistry::new(),
            &mut producers,
            None,
        )
    };
    let before = project(semantics.clone());
    semantics.children.reverse();
    let after = project(semantics);
    assert_eq!(before.produced_targets.len(), 2);
    assert_eq!(before.produced_targets, after.produced_targets);
    assert!(before.producer_issues.is_empty() && after.producer_issues.is_empty());
    let ids: HashSet<_> = after.tree.nodes.iter().map(|(id, _)| id).collect();
    assert_eq!(ids.len(), after.tree.nodes.len());
}

#[test]
fn identity_tests_invalid_producer_does_not_block_a_valid_sibling() {
    let (mut dom, _, _) = fixture("");
    let root = dom.document();
    let other = dom.create_element(cambium::attr_qual("custom-leaf"));
    dom.set_attribute(other, cambium::attr_qual("key"), "8");
    dom.set_attribute(
        other,
        cambium::attr_qual("style"),
        "display:block;width:80px;height:30px",
    );
    dom.append_child(root, other);
    let layout = OwnedLayout::new(&dom, &[""], 320.0, 200.0, &[], &HashMap::new());
    let mut invalid = board();
    let action = invalid.children[0].actions[0].clone();
    invalid.children[0].actions.push(action);
    let mut producers = ProducerRegistry::new();
    producers.register(7, Described(invalid), &[]).unwrap();
    producers.register(8, Board, &[]).unwrap();
    let projection = project_semantics(
        &dom,
        &layout,
        &mut LeafRegistry::new(),
        &mut producers,
        None,
    );
    assert_eq!(projection.producer_issues.len(), 1);
    assert_eq!(projection.producer_slots, HashMap::from([(8, other)]));
    assert_eq!(projection.produced_targets.len(), 1);
    assert!(
        projection
            .produced_targets
            .values()
            .all(|target| target.slot == 8)
    );
}
