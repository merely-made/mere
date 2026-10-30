// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::{ProducedTexture, ProducerContext, ProducerNode, ScriptedDom, TextureProducer};
use layout_dom_api::{LayoutDomMut, LocalName, Namespace};

struct GeometryOnly;
impl TextureProducer for GeometryOnly {
    fn render(&mut self, _: &ProducerContext<'_>) -> Option<ProducedTexture> {
        None
    }

    fn semantics(&self) -> Option<ProducerSemantics> {
        Some(ProducerSemantics {
            role: Some(ProducerRole::Group),
            name: Some("Geometry".into()),
            children: vec![ProducerNode {
                key: 1,
                role: ProducerRole::ListItem,
                name: "Child".into(),
                rect: [4.0, 5.0, 10.0, 20.0],
                actions: Vec::new(),
            }],
        })
    }
}

#[test]
fn batch_producer_geometry_preserves_content_edges_translation_and_hidden_ancestor_admission() {
    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root, "<div id='scroll' style='display:block;width:100px;height:100px;overflow:auto;transform:translate(13px,17px)'><custom-leaf key='7' style='display:block;width:80px;height:30px;padding:3px 5px;border:2px solid;visibility:visible;transform:translate(7px,11px)'></custom-leaf><div style='width:500px;height:500px'></div></div><div style='width:700px;height:800px'></div>");
    let scroll = flat_nodes(&dom)
        .find(|node| {
            dom.attribute(*node, &Namespace::default(), &LocalName::from("id")) == Some("scroll")
        })
        .unwrap();
    let mut layout = OwnedLayout::new(&dom, &[""], 300.0, 200.0, &[], &HashMap::new());
    let slot = layout.custom_leaf_nodes(&dom)[0].1;
    layout.set_element_scroll(&dom, [(scroll, (20.0, 25.0))].into());
    layout.set_viewport_scroll((3.0, 5.0));
    let mut producers = ProducerRegistry::new();
    producers.register(7, GeometryOnly, &[]).unwrap();
    let projected = project_semantics(&dom, &layout, &LeafRegistry::new(), &producers, None);
    let access = &projected
        .tree
        .nodes
        .iter()
        .find(|(id, _)| id.0 == dom.opaque_id(slot))
        .unwrap()
        .1;
    let bounds = access.bounds().unwrap();
    let fragment = layout.fragments().get(slot).unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0),
        (f64::from(fragment.x - 3.0), f64::from(fragment.y - 2.0))
    );
    let child = &projected
        .tree
        .nodes
        .iter()
        .find(|(id, _)| *id == access.children()[0])
        .unwrap()
        .1;
    assert_eq!(
        child.bounds(),
        Some(Rect::new(
            bounds.x0 + 11.0,
            bounds.y0 + 10.0,
            bounds.x0 + 21.0,
            bounds.y0 + 30.0
        ))
    );
    assert_eq!(layout.custom_leaf_boxes(&dom), vec![(7, (80.0, 30.0))]);

    let mut mutations = Vec::new();
    dom.drain_mutations(&mut mutations);
    mutations.clear();
    dom.set_attribute(scroll, cambium::attr_qual("style"),
        "display:block;width:100px;height:100px;overflow:auto;visibility:hidden;transform:translate(13px,17px)");
    dom.drain_mutations(&mut mutations);
    layout.rebuild(&dom, 300.0, 200.0, &mutations);
    let projected = project_semantics(&dom, &layout, &LeafRegistry::new(), &producers, None);
    let access = &projected
        .tree
        .nodes
        .iter()
        .find(|(id, _)| id.0 == dom.opaque_id(slot))
        .unwrap()
        .1;
    assert!(
        access.bounds().is_some(),
        "semantic visibility override stays visible"
    );
    let child = &projected
        .tree
        .nodes
        .iter()
        .find(|(id, _)| *id == access.children()[0])
        .unwrap()
        .1;
    assert!(
        child.bounds().is_none(),
        "producer placement preserves its hidden-ancestor refusal"
    );
    assert!(layout.custom_leaf_boxes(&dom).is_empty());
}

#[test]
fn publication_producer_origin_requires_a_current_window_member_with_the_same_key() {
    use std::{cell::Cell, rc::Rc};

    for remove in [false, true] {
        let mut dom = ScriptedDom::new();
        let root = dom.document();
        dom.set_inner_html(
            root,
            "<custom-leaf key='7' style='display:block;width:80px;height:30px'></custom-leaf>",
        );
        let layout = OwnedLayout::new(&dom, &[""], 300.0, 200.0, &[], &HashMap::new());
        let slot = layout.custom_leaf_nodes(&dom)[0].1;
        let slot_id = A11yNodeId(dom.opaque_id(slot));
        let mut producers = ProducerRegistry::new();
        producers.register(7, GeometryOnly, &[]).unwrap();
        let leaves = LeafRegistry::new();
        let mut projected = project_semantics(&dom, &layout, &leaves, &producers, None);
        let child = projected
            .tree
            .nodes
            .iter()
            .find(|(id, _)| *id == slot_id)
            .unwrap()
            .1
            .children()[0];
        let marker = Rc::new(Cell::new(true));
        let owner = crate::A11yOwner::new(&marker);
        let window = crate::WindowDom::document(&dom);
        assert!(
            projected
                .publication_bindings(&owner, &window, &leaves, &producers)
                .is_ok()
        );

        if remove {
            dom.remove(slot);
            assert!(!dom.is_live(slot));
            // Even a retained producer-only tree cannot use a dead origin.
            projected.tree.nodes.retain(|(id, _)| *id != slot_id);
            projected.dom_targets.remove(&slot_id);
        } else {
            dom.set_attribute(slot, cambium::attr_qual("key"), "8");
        }
        let window = crate::WindowDom::document(&dom);
        let issue = projected.publication_bindings(&owner, &window, &leaves, &producers);
        assert!(
            matches!(issue,
            Err(PublicationIssue::UnknownNode(id)) if remove && id == child)
                || matches!(issue,
            Err(PublicationIssue::RouteMismatch(id)) if !remove && id == child)
        );
    }
}
