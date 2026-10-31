// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::{HashMap, HashSet};

use document_session_api::{DocumentA11yAction, DocumentA11yBounds, DocumentA11yProjection};
use layout_dom_api::{LayoutDom, LocalName, Namespace, NodeKind};

use crate::{NodeId, OwnedLayout};

/// The renderer-neutral DOM observation used to seed host semantics, including
/// Cambium's app-owned committed text for marked textboxes. Contributed leaf
/// and producer semantics are added by [`crate::project_semantics`], which
/// native and browser publication consume. This read does not invoke those
/// contributors. `focus` is the focused node's opaque id, as
/// [`crate::Accessibility::sync`] receives it.
pub fn document_projection<D: LayoutDom<NodeId = NodeId>>(
    dom: &D,
    layout: &OwnedLayout,
    focus: Option<u64>,
) -> DocumentA11yProjection {
    let mut focused = None;
    let mut facts = HashMap::new();
    let mut painted = HashMap::new();
    let mut markers = HashMap::new();
    let mut numeric = HashSet::new();
    // Geometry, visibility and generated content share the style owner's flat
    // traversal. No per-node walk back through the scroll/style ancestors.
    layout.walk_painted_geometry(dom, |node, geometry| {
        let opaque = dom.opaque_id(node);
        if focus == Some(opaque) {
            focused = Some(node);
        }
        facts.insert(node, (geometry.rendered, geometry.generated_text));
        if let Some((x, y, width, height)) = geometry.border_rect {
            painted.insert(
                opaque,
                DocumentA11yBounds {
                    x,
                    y,
                    width,
                    height,
                },
            );
        }
        if dom.kind(node) == NodeKind::Element
            && dom
                .attribute(node, &Namespace::default(), &LocalName::from("role"))
                .is_some_and(|role| role.trim().eq_ignore_ascii_case("textbox"))
            && let Some(value) = dom.attribute(
                node,
                &Namespace::default(),
                &LocalName::from("data-cambium-text-value"),
            )
        {
            let multiline = dom.attribute(
                node,
                &Namespace::default(),
                &LocalName::from("aria-multiline"),
            ) == Some("true");
            markers.insert(opaque, (value.to_owned(), multiline));
        }
        if dom.attribute(
            node,
            &Namespace::default(),
            &LocalName::from("data-cambium-set-value"),
        ) == Some("true")
        {
            numeric.insert(opaque);
        }
    });
    let projection = genet_render::document_a11y_projection_with_style(
        dom,
        layout.fragments(),
        focused,
        0,
        None,
        &genet_render::A11yStyleQueries {
            // Name references may name an unassigned light-DOM node outside
            // the flat tree. Preserve the existing style query for that case.
            generated: &|node| {
                facts.get(&node).map_or_else(
                    || layout.generated_text(dom, node),
                    |fact| (fact.1.0.to_owned(), fact.1.1.to_owned()),
                )
            },
            rendered: &|node| {
                facts
                    .get(&node)
                    .map_or_else(|| layout.rendered_visible(dom, node), |fact| fact.0)
            },
        },
    );
    // Genet supplies semantic names and actions; this host owns retained scroll
    // planes and viewport placement. Apply the shared translated border bounds
    // before platform lowering. Action routing remains bound to node identity.
    let mut nodes = projection.nodes().to_vec();
    for node in &mut nodes {
        if numeric.contains(&node.id.get())
            && node.actions.contains(&DocumentA11yAction::Click)
            && !node.state.disabled
            && !node.state.read_only
            && !node.actions.contains(&DocumentA11yAction::SetValue)
        {
            node.actions.push(DocumentA11yAction::SetValue);
        }
        if node.bounds.is_some()
            && let Some(bounds) = painted.get(&node.id.get())
        {
            node.bounds = Some(*bounds);
        }
    }
    if !markers.is_empty() {
        // An outer committed textbox owns its entire projected subtree. Carry
        // that owner once instead of enumerating every marker's descendants.
        let children: HashMap<_, _> = nodes
            .iter()
            .map(|node| (node.id, node.children.as_slice()))
            .collect();
        let mut hidden = HashSet::new();
        let mut focused_markers = HashSet::new();
        let mut pending = vec![(projection.root(), None)];
        while let Some((id, owner)) = pending.pop() {
            if let Some(owner) = owner {
                hidden.insert(id);
                if focus == Some(id.get()) {
                    focused_markers.insert(owner);
                }
            }
            let owner = owner.or_else(|| markers.contains_key(&id.get()).then_some(id.get()));
            pending.extend(
                children
                    .get(&id)
                    .into_iter()
                    .flat_map(|ids| ids.iter())
                    .map(|child| (*child, owner)),
            );
        }
        nodes.retain(|node| !hidden.contains(&node.id));
        for node in &mut nodes {
            let Some((value, multiline)) = markers.get(&node.id.get()) else {
                continue;
            };
            node.value = Some(value.clone());
            node.state.multiline = *multiline;
            node.state.editable = !node.state.disabled && !node.state.read_only;
            node.children.clear();
            node.state.focused |= focused_markers.contains(&node.id.get());
        }
    }
    DocumentA11yProjection::new(
        projection.revision(),
        projection.support().clone(),
        projection.root(),
        nodes,
    )
}
