// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fold the final catalog projection, retaining the full source layout.
use crate::{MereNodeKind, NodeState};
use cambium::{GraphCanvasNode, GraphCanvasSwatch};
use curation::{FoldRecord, FoldStandIn};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Synthetic target identity. Source/catalog keys must never use this key.
pub fn fold_node_key(id: &curation::FoldId) -> String {
    format!("curation:fold:{}", id.as_str())
}

pub(super) fn project(
    mut swatch: GraphCanvasSwatch<String, MereNodeKind>,
    records: &[FoldRecord],
) -> GraphCanvasSwatch<String, MereNodeKind> {
    if records.is_empty() {
        return swatch;
    }
    let original = swatch
        .graph
        .nodes
        .iter()
        .map(|n| (n.id.clone(), n.clone()))
        .collect::<HashMap<_, _>>();
    let mut owners = HashMap::new();
    let mut used_ids = HashSet::new();
    let mut stand_ins = Vec::new();
    for record in records {
        let synthetic = fold_node_key(&record.id);
        if record.validate().is_err()
            || original.contains_key(&synthetic)
            || used_ids.contains(&record.id)
            || record
                .members
                .iter()
                .any(|m| !original.contains_key(m) || owners.contains_key(m))
        {
            continue;
        }
        let id = record
            .stand_in_member()
            .map(str::to_owned)
            .unwrap_or(synthetic);
        let mut stand_in = match &record.stand_in {
            FoldStandIn::Member(member) => original[member].clone(),
            FoldStandIn::Summary { label } => {
                let n = record.members.len() as f32;
                let position = record.members.iter().fold((0., 0.), |(x, y), member| {
                    let p = original[member].position;
                    (x + p.0 / n, y + p.1 / n)
                });
                GraphCanvasNode {
                    id: id.clone(),
                    kind: MereNodeKind::Graph(NodeState::Available),
                    position,
                    label: label.clone().unwrap_or_else(|| "Fold".into()),
                    key: Some(id.clone()),
                }
            },
        };
        stand_in.label = format!(
            "{} +{}, activate to expand",
            record.label.as_deref().unwrap_or(&stand_in.label),
            record.hidden_count()
        );
        swatch
            .node_attrs
            .push((id.clone(), "aria-expanded".into(), "false".into()));
        for member in &record.members {
            owners.insert(member.clone(), id.clone());
        }
        used_ids.insert(record.id.clone());
        stand_ins.push(stand_in);
    }
    if stand_ins.is_empty() {
        return swatch;
    }
    if let Some(key) = swatch
        .focus
        .as_ref()
        .and_then(|key| owners.get(key))
        .cloned()
    {
        swatch.focus = Some(key.clone());
        swatch.focus_request = Some(key);
    }
    swatch.graph.nodes.retain(|n| !owners.contains_key(&n.id));
    swatch.graph.nodes.extend(stand_ins);
    let mut bundles = BTreeMap::new();
    let mut untouched = Vec::new();
    for mut relation in swatch.relations {
        if !owners.contains_key(&relation.from) && !owners.contains_key(&relation.to) {
            untouched.push(relation);
            continue;
        }
        if !relation.visible {
            continue;
        }
        relation.from = owners.get(&relation.from).cloned().unwrap_or(relation.from);
        relation.to = owners.get(&relation.to).cloned().unwrap_or(relation.to);
        if relation.from == relation.to {
            continue;
        }
        relation.route.clear();
        // This is an aggregate target, not the first source cell's evidence.
        relation.link_key = None;
        relation.marks = Default::default();
        let key = (
            relation.from.clone(),
            relation.to.clone(),
            relation.kind.clone(),
        );
        let entry = bundles.entry(key).or_insert((relation, 0usize));
        entry.1 += 1;
    }
    for ((from, to, kind), (mut relation, count)) in bundles {
        relation.id = format!(
            "curation:bundle:{}:{from}{}:{to}{}:{kind}",
            from.len(),
            to.len(),
            kind.len()
        );
        relation.label = format!(
            "{count} {kind} relation{}",
            if count == 1 { "" } else { "s" }
        );
        relation.emphasized = count > 1;
        untouched.push(relation);
    }
    swatch.relations = untouched;
    swatch.graph.edges.clear();
    swatch
}

#[cfg(test)]
mod tests {
    use super::*;
    use cambium::{GraphCanvasRelation, GraphCanvasSubgraph};
    use curation::FoldId;
    fn fixture() -> GraphCanvasSwatch<String, MereNodeKind> {
        let nodes = ["a", "b", "c", "d"]
            .into_iter()
            .enumerate()
            .map(|(i, id)| GraphCanvasNode {
                id: id.into(),
                kind: MereNodeKind::Graph(NodeState::Available),
                position: (i as f32 / 4., 0.5),
                label: id.into(),
                key: Some(id.into()),
            })
            .collect();
        let relations = [("a", "b"), ("a", "c"), ("b", "d"), ("d", "a")]
            .into_iter()
            .enumerate()
            .map(|(i, (a, b))| GraphCanvasRelation {
                id: i.to_string(),
                from: a.into(),
                to: b.into(),
                kind: "extracted".into(),
                label: "source".into(),
                route: vec![],
                visible: true,
                emphasized: false,
                link_key: None,
                marks: Default::default(),
                strength: 1.0,
            })
            .collect();
        GraphCanvasSwatch::new(
            1,
            GraphCanvasSubgraph {
                nodes,
                edges: vec![],
            },
        )
        .with_relations(relations)
    }
    #[test]
    fn opaque_members_route_between_two_folds_once_and_preserve_source_geometry() {
        let source = fixture();
        let before = source.clone();
        let folds = [
            ("left", vec!["a".to_string(), "b".to_string()]),
            ("right", vec!["c".to_string(), "d".to_string()]),
        ]
        .map(|(id, members)| FoldRecord::selection(FoldId::new(id), "catalog", members).unwrap());
        let projected = project(source.clone(), &folds);
        assert_eq!(projected.graph.nodes.len(), 2);
        assert_eq!(projected.graph.nodes[0].position, (0.125, 0.5));
        assert_eq!(projected.relations.len(), 2);
        assert_eq!(projected.relations[0].label, "2 extracted relations");
        assert_eq!(projected.relations[1].label, "1 extracted relation");
        assert_eq!(source, before);
        assert_eq!(project(source.clone(), &[]), source);
    }
    #[test]
    fn partial_and_overlapping_records_never_hide_an_uncovered_member() {
        let source = fixture();
        let stale = FoldRecord::selection(
            FoldId::new("stale"),
            "catalog",
            vec!["a".to_string(), "missing".to_string()],
        )
        .unwrap();
        assert_eq!(project(source.clone(), &[stale]), source);
        let first = FoldRecord::selection(
            FoldId::new("one"),
            "catalog",
            vec!["a".to_string(), "b".to_string()],
        )
        .unwrap();
        let overlap = FoldRecord::selection(
            FoldId::new("two"),
            "catalog",
            vec!["b".to_string(), "c".to_string()],
        )
        .unwrap();
        let projected = project(source, &[first, overlap]);
        assert!(projected.graph.nodes.iter().any(|n| n.id == "c"));
        assert_eq!(projected.graph.nodes.len(), 3);
    }
}
