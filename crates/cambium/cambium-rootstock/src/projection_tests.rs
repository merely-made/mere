// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::cell::Cell;

use layout_dom_api::{
    AttributeView, LayoutDom, LayoutDomMut, LocalName, Namespace, NodeKind, QualName,
};

use crate::{NodeId, OwnedLayout, ScriptedDom, document_projection};

struct CountedDom<'a> {
    dom: &'a ScriptedDom,
    parents: Cell<usize>,
    children: Cell<usize>,
}

impl LayoutDom for CountedDom<'_> {
    type NodeId = NodeId;

    fn document(&self) -> NodeId {
        self.dom.document()
    }

    fn parent(&self, node: NodeId) -> Option<NodeId> {
        self.parents.set(self.parents.get() + 1);
        self.dom.parent(node)
    }

    fn prev_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.dom.prev_sibling(node)
    }

    fn next_sibling(&self, node: NodeId) -> Option<NodeId> {
        self.dom.next_sibling(node)
    }

    fn dom_children(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.children.set(self.children.get() + 1);
        self.dom.dom_children(node)
    }

    fn flat_children(&self, node: NodeId) -> impl Iterator<Item = NodeId> + '_ {
        self.children.set(self.children.get() + 1);
        self.dom.flat_children(node)
    }

    fn kind(&self, node: NodeId) -> NodeKind {
        self.dom.kind(node)
    }

    fn opaque_id(&self, node: NodeId) -> u64 {
        self.dom.opaque_id(node)
    }

    fn element_name(&self, node: NodeId) -> Option<&QualName> {
        self.dom.element_name(node)
    }

    fn attribute(&self, node: NodeId, ns: &Namespace, local: &LocalName) -> Option<&str> {
        self.dom.attribute(node, ns, local)
    }

    fn attributes(&self, node: NodeId) -> impl Iterator<Item = AttributeView<'_>> + '_ {
        self.dom.attributes(node)
    }

    fn text(&self, node: NodeId) -> Option<&str> {
        self.dom.text(node)
    }
}

fn element(dom: &mut ScriptedDom, parent: NodeId, tag: &str, style: &str) -> NodeId {
    let node = dom.create_element(QualName::new(
        None,
        Namespace::from("http://www.w3.org/1999/xhtml"),
        LocalName::from(tag),
    ));
    dom.append_child(parent, node);
    attr(dom, node, "style", style);
    node
}

fn attr(dom: &mut ScriptedDom, node: NodeId, name: &str, value: &str) {
    dom.set_attribute(
        node,
        QualName::new(None, Namespace::default(), LocalName::from(name)),
        value,
    );
}

#[test]
fn semantic_geometry_traversal_scales_with_nodes_instead_of_ancestor_pairs() {
    let mut work = Vec::new();
    for count in [24, 48] {
        let mut dom = ScriptedDom::new();
        let mut parent = dom.document();
        for _ in 0..count {
            parent = element(
                &mut dom,
                parent,
                "div",
                "display:block;width:40px;height:40px",
            );
            // Explicit names isolate geometry/style traversal from name-from-content work.
            attr(&mut dom, parent, "aria-label", "Named");
        }
        let layout = OwnedLayout::new(&dom, &[""], 320.0, 200.0, &[], &Default::default());
        let counted = CountedDom {
            dom: &dom,
            parents: Cell::new(0),
            children: Cell::new(0),
        };
        let projection = document_projection(&counted, &layout, Some(dom.opaque_id(parent)));
        assert_eq!(projection.nodes().len(), count + 1);
        let visits = counted.parents.get() + counted.children.get();
        eprintln!(
            "{count} nodes: {visits} structure queries ({} parents)",
            counted.parents.get()
        );
        work.push(visits);
        assert!(
            visits <= 12 * (count + 1),
            "{count} nodes required {visits} structure queries ({} parents)",
            counted.parents.get()
        );
    }
    assert!(
        work[1] <= 2 * work[0] + 12,
        "work must scale linearly: {work:?}"
    );
}

#[test]
fn publication_membership_uses_the_existing_window_inventory() {
    use std::rc::Rc;

    for count in [24, 48] {
        let mut dom = ScriptedDom::new();
        let mut parent = dom.document();
        for _ in 0..count {
            parent = element(
                &mut dom,
                parent,
                "div",
                "display:block;width:40px;height:40px",
            );
            attr(&mut dom, parent, "aria-label", "Named");
        }
        let window = crate::WindowDom::document(&dom);
        let layout = OwnedLayout::new(&window, &[""], 320.0, 200.0, &[], &Default::default());
        let leaves = crate::LeafRegistry::new();
        let producers = crate::ProducerRegistry::new();
        let projected = crate::project_semantics(&window, &layout, &leaves, &producers, None);
        let marker = Rc::new(Cell::new(true));
        let owner = crate::A11yOwner::new(&marker);
        crate::window_dom::take_membership_steps();
        let bindings = projected
            .publication_bindings(&owner, &window, &leaves, &producers)
            .unwrap();
        assert_eq!(bindings.identities.len(), count + 1);
        let steps = crate::window_dom::take_membership_steps();
        eprintln!("{count} publication nodes: {steps} membership ancestor steps");
        assert_eq!(steps, 0, "the window inventory already proves membership");
    }
}

fn bounds(projection: &crate::DocumentA11yProjection, id: u64) -> (f32, f32, f32, f32) {
    let rect = projection
        .nodes()
        .iter()
        .find(|node| node.id.get() == id)
        .unwrap()
        .bounds
        .unwrap();
    (rect.x, rect.y, rect.width, rect.height)
}

#[test]
fn semantic_geometry_tracks_nested_translation_scroll_windows_and_retained_changes() {
    use crate::WindowDom;

    let mut dom = ScriptedDom::new();
    let document = dom.document();
    let outside = element(
        &mut dom,
        document,
        "div",
        "transform:translate(901px,903px)",
    );
    let first = element(&mut dom, outside, "div", "transform:translate(701px,703px)");
    let second = element(&mut dom, outside, "div", "transform:translate(501px,503px)");
    let outer = element(
        &mut dom,
        first,
        "div",
        "display:block;width:120px;height:100px;overflow:auto;transform:translate(11px,13px)",
    );
    let inner = element(
        &mut dom,
        outer,
        "div",
        "display:block;width:500px;height:500px;overflow:auto;transform:translate(3px,5px)",
    );
    let target = element(
        &mut dom,
        inner,
        "button",
        "display:block;width:80px;height:30px;transform:translate(2px,4px)",
    );
    attr(&mut dom, target, "aria-label", "First window");
    element(
        &mut dom,
        inner,
        "div",
        "display:block;width:900px;height:900px",
    );
    element(
        &mut dom,
        first,
        "div",
        "display:block;width:800px;height:1000px",
    );
    let other = element(
        &mut dom,
        second,
        "button",
        "display:block;width:80px;height:30px;transform:translate(31px,37px)",
    );
    attr(&mut dom, other, "aria-label", "Second window");
    let first_view = WindowDom::new(&dom, first);
    let second_view = WindowDom::new(&dom, second);
    let mut layout = OwnedLayout::new(&first_view, &[""], 300.0, 200.0, &[], &Default::default());
    let other_layout =
        OwnedLayout::new(&second_view, &[""], 300.0, 200.0, &[], &Default::default());
    layout.set_element_scroll(
        &first_view,
        [(outer, (7.0, 9.0)), (inner, (11.0, 13.0))].into(),
    );
    layout.set_viewport_scroll((3.0, 5.0));
    assert_eq!(layout.element_scroll()[&outer], (7.0, 9.0));
    assert_eq!(layout.element_scroll()[&inner], (11.0, 13.0));
    assert_eq!(layout.viewport_scroll(), (3.0, 5.0));
    let projection = document_projection(&first_view, &layout, Some(dom.opaque_id(target)));
    let fragment = layout.fragments().get(target).unwrap();
    assert_eq!(
        bounds(&projection, dom.opaque_id(target)),
        (
            fragment.x - 5.0,
            fragment.y - 5.0,
            fragment.width,
            fragment.height
        )
    );
    let fragment = layout.fragments().get(outer).unwrap();
    assert_eq!(
        bounds(&projection, dom.opaque_id(outer)),
        (
            fragment.x + 8.0,
            fragment.y + 8.0,
            fragment.width,
            fragment.height
        ),
        "the scroller's own offset must not move its border box"
    );
    let other_before = document_projection(&second_view, &other_layout, None);
    let other_fragment = other_layout.fragments().get(other).unwrap();
    assert_eq!(
        bounds(&other_before, dom.opaque_id(other)),
        (
            other_fragment.x + 31.0,
            other_fragment.y + 37.0,
            other_fragment.width,
            other_fragment.height
        )
    );
    assert!(
        projection
            .nodes()
            .iter()
            .all(|node| node.id.get() != dom.opaque_id(other))
    );
    assert!(
        other_before
            .nodes()
            .iter()
            .all(|node| node.id.get() != dom.opaque_id(target))
    );

    let mut mutations = Vec::new();
    dom.drain_mutations(&mut mutations);
    mutations.clear();
    attr(
        &mut dom,
        target,
        "style",
        "display:block;width:80px;height:30px;transform:translate(19px,23px)",
    );
    dom.drain_mutations(&mut mutations);
    let first_view = WindowDom::new(&dom, first);
    layout.rebuild(&first_view, 300.0, 200.0, &mutations);
    let changed = document_projection(&first_view, &layout, Some(dom.opaque_id(target)));
    let fragment = layout.fragments().get(target).unwrap();
    assert_eq!(
        bounds(&changed, dom.opaque_id(target)),
        (
            fragment.x + 12.0,
            fragment.y + 14.0,
            fragment.width,
            fragment.height
        )
    );
    let mut fresh = OwnedLayout::new(&first_view, &[""], 300.0, 200.0, &[], &Default::default());
    fresh.set_element_scroll(&first_view, layout.element_scroll().clone());
    fresh.set_viewport_scroll(layout.viewport_scroll());
    assert_eq!(
        changed.nodes(),
        document_projection(&first_view, &fresh, Some(dom.opaque_id(target))).nodes()
    );

    mutations.clear();
    dom.append_child(first, target);
    dom.drain_mutations(&mut mutations);
    let first_view = WindowDom::new(&dom, first);
    layout.rebuild(&first_view, 300.0, 200.0, &mutations);
    let moved = document_projection(&first_view, &layout, Some(dom.opaque_id(target)));
    let fragment = layout.fragments().get(target).unwrap();
    assert_eq!(
        bounds(&moved, dom.opaque_id(target)),
        (
            fragment.x + 16.0,
            fragment.y + 18.0,
            fragment.width,
            fragment.height
        )
    );
    let second_view = WindowDom::new(&dom, second);
    assert_eq!(
        other_before.nodes(),
        document_projection(&second_view, &other_layout, None).nodes()
    );
}

#[test]
fn committed_textbox_collapse_preserves_outer_focus_and_unrelated_nodes() {
    let mut dom = ScriptedDom::new();
    let document = dom.document();
    let outer = element(
        &mut dom,
        document,
        "div",
        "display:block;width:100px;height:100px",
    );
    let inner = element(
        &mut dom,
        outer,
        "div",
        "display:block;width:80px;height:80px",
    );
    let descendant = element(
        &mut dom,
        inner,
        "button",
        "display:block;width:40px;height:40px",
    );
    let sibling = element(
        &mut dom,
        document,
        "button",
        "display:block;width:40px;height:40px",
    );
    for (node, value) in [(outer, "Committed outer"), (inner, "Committed inner")] {
        attr(&mut dom, node, "role", "textbox");
        attr(&mut dom, node, "data-cambium-text-value", value);
    }
    let layout = OwnedLayout::new(&dom, &[""], 300.0, 200.0, &[], &Default::default());
    let projection = document_projection(&dom, &layout, Some(dom.opaque_id(descendant)));
    let field = projection
        .nodes()
        .iter()
        .find(|node| node.id.get() == dom.opaque_id(outer))
        .unwrap();
    assert_eq!(field.value.as_deref(), Some("Committed outer"));
    assert!(field.state.focused);
    assert!(field.children.is_empty());
    assert!(
        !projection
            .nodes()
            .iter()
            .any(|node| [dom.opaque_id(inner), dom.opaque_id(descendant)].contains(&node.id.get()))
    );
    assert!(
        projection
            .nodes()
            .iter()
            .any(|node| node.id.get() == dom.opaque_id(sibling))
    );
}
