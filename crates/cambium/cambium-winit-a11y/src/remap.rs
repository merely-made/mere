// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! NodeId remapping over AccessKit's public `Node` API (cambium-publish D4).
//!
//! AccessKit 0.25 has no traversal API, so every NodeId-carrying property is
//! named here. `NODE_ID_PROPERTIES` is the audited list; the schema canary in
//! the tests fails when an AccessKit release adds a property it does not name.

use accesskit::{Node, NodeId, TextPosition, TextSelection};

/// The serialized names of every NodeId-carrying `Node` property, as audited
/// against AccessKit 0.25.1.
#[cfg(test)]
const NODE_ID_PROPERTIES: &[&str] = &[
    "children",
    "controls",
    "details",
    "describedBy",
    "flowTo",
    "labelledBy",
    "owns",
    "radioGroup",
    "activeDescendant",
    "errorMessage",
    "inPageLinkTarget",
    "memberOf",
    "nextOnLine",
    "previousOnLine",
    "popupFor",
    "textSelection",
];

macro_rules! remap_vecs {
    ($node:ident, $map:ident, $(($get:ident, $set:ident)),+ $(,)?) => {$(
        if !$node.$get().is_empty() {
            let ids = $node.$get().iter().map(|id| $map(*id)).collect::<Result<Vec<_>, _>>()?;
            $node.$set(ids);
        }
    )+};
}

macro_rules! remap_singles {
    ($node:ident, $map:ident, $(($get:ident, $set:ident)),+ $(,)?) => {$(
        if let Some(id) = $node.$get() {
            $node.$set($map(id)?);
        }
    )+};
}

/// Rewrites every NodeId a node refers to through `map`, stopping at the first
/// refusal. The node's own id is the caller's.
pub(crate) fn try_map_node_ids<E>(
    node: &mut Node,
    map: impl Fn(NodeId) -> Result<NodeId, E>,
) -> Result<(), E> {
    remap_vecs!(
        node,
        map,
        (children, set_children),
        (controls, set_controls),
        (details, set_details),
        (described_by, set_described_by),
        (flow_to, set_flow_to),
        (labelled_by, set_labelled_by),
        (owns, set_owns),
        (radio_group, set_radio_group),
    );
    remap_singles!(
        node,
        map,
        (active_descendant, set_active_descendant),
        (error_message, set_error_message),
        (in_page_link_target, set_in_page_link_target),
        (member_of, set_member_of),
        (next_on_line, set_next_on_line),
        (previous_on_line, set_previous_on_line),
        (popup_for, set_popup_for),
    );
    if let Some(selection) = node.text_selection() {
        let position = |p: TextPosition| -> Result<TextPosition, E> {
            Ok(TextPosition {
                node: map(p.node)?,
                character_index: p.character_index,
            })
        };
        let mapped = TextSelection {
            anchor: position(selection.anchor)?,
            focus: position(selection.focus)?,
        };
        node.set_text_selection(mapped);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use accesskit::Role;
    use std::collections::{BTreeSet, HashSet};

    const SHIFT: u64 = 1_000;

    fn every_property() -> Node {
        let mut node = Node::new(Role::GenericContainer);
        let mut next = 1u64;
        let mut id = || {
            next += 1;
            NodeId(next)
        };
        node.set_children(vec![id(), id()]);
        node.set_controls(vec![id()]);
        node.set_details(vec![id()]);
        node.set_described_by(vec![id()]);
        node.set_flow_to(vec![id()]);
        node.set_labelled_by(vec![id()]);
        node.set_owns(vec![id()]);
        node.set_radio_group(vec![id()]);
        node.set_active_descendant(id());
        node.set_error_message(id());
        node.set_in_page_link_target(id());
        node.set_member_of(id());
        node.set_next_on_line(id());
        node.set_previous_on_line(id());
        node.set_popup_for(id());
        node.set_text_selection(TextSelection {
            anchor: TextPosition { node: id(), character_index: 0 },
            focus: TextPosition { node: id(), character_index: 3 },
        });
        node
    }

    fn referenced(node: &Node) -> Vec<NodeId> {
        let mut ids: Vec<NodeId> = [
            node.children(),
            node.controls(),
            node.details(),
            node.described_by(),
            node.flow_to(),
            node.labelled_by(),
            node.owns(),
            node.radio_group(),
        ]
        .concat();
        ids.extend(
            [
                node.active_descendant(),
                node.error_message(),
                node.in_page_link_target(),
                node.member_of(),
                node.next_on_line(),
                node.previous_on_line(),
                node.popup_for(),
            ]
            .into_iter()
            .flatten(),
        );
        if let Some(selection) = node.text_selection() {
            ids.extend([selection.anchor.node, selection.focus.node]);
        }
        ids
    }

    #[test]
    fn every_reference_is_remapped() {
        let mut node = every_property();
        let before = referenced(&node);
        assert_eq!(before.len(), 18, "fixture sets every audited property");
        try_map_node_ids(&mut node, |id| Ok::<_, ()>(NodeId(id.0 + SHIFT))).unwrap();
        let after = referenced(&node);
        let expected: Vec<_> = before.iter().map(|id| NodeId(id.0 + SHIFT)).collect();
        assert_eq!(after, expected);
        assert_eq!(node.text_selection().unwrap().focus.character_index, 3);
    }

    #[test]
    fn a_refusal_stops_the_remap() {
        let mut node = every_property();
        let refused = NodeId(5);
        let result = try_map_node_ids(&mut node, |id| if id == refused { Err(id) } else { Ok(id) });
        assert_eq!(result, Err(refused));
    }

    #[test]
    fn empty_node_is_untouched() {
        let mut node = Node::new(Role::Label);
        try_map_node_ids(&mut node, |_| Err::<NodeId, _>(())).unwrap();
        assert!(referenced(&node).is_empty());
    }

    /// Every property whose schema reaches `NodeId`, read from AccessKit's own
    /// schema, must be one this module remaps.
    #[test]
    fn schema_canary_names_every_node_id_property() {
        let schema = serde_json::to_value(schemars::schema_for!(Node)).unwrap();
        let defs = schema["$defs"].as_object().expect("schema defs");
        let properties = defs["Properties"]["properties"].as_object().expect("Properties def");
        let found: BTreeSet<&str> = properties
            .iter()
            .filter(|(_, sub)| reaches_node_id(sub, defs, &mut HashSet::new()))
            .map(|(name, _)| name.as_str())
            .collect();
        let audited: BTreeSet<&str> = NODE_ID_PROPERTIES.iter().copied().collect();
        assert_eq!(found, audited, "AccessKit's NodeId properties changed: update remap.rs");
    }

    fn reaches_node_id(
        value: &serde_json::Value,
        defs: &serde_json::Map<String, serde_json::Value>,
        seen: &mut HashSet<String>,
    ) -> bool {
        match value {
            serde_json::Value::Object(map) => map.iter().any(|(key, inner)| {
                if key == "$ref" {
                    let name = inner.as_str().unwrap_or("").rsplit('/').next().unwrap_or("");
                    name == "NodeId"
                        || (seen.insert(name.to_owned())
                            && defs.get(name).is_some_and(|d| reaches_node_id(d, defs, seen)))
                } else {
                    reaches_node_id(inner, defs, seen)
                }
            }),
            serde_json::Value::Array(items) => items.iter().any(|v| reaches_node_id(v, defs, seen)),
            _ => false,
        }
    }
}
