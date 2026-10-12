// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! The exact relation-cell input seam shared by the controller and inspector.
//! Cell multiplicity is not assertion multiplicity or a measured force value.
use kernel::graph::{
    EdgeFamily, Graph, GraphStratum, NodeKey, RelationKey, RelationKind, RelationSelector,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use uuid::Uuid;

pub fn selector_for_relation_kind(kind: RelationKind) -> RelationSelector {
    match kind {
        RelationKind::Semantic(sub) => RelationSelector::Semantic(sub),
        RelationKind::OpenPredicate => RelationSelector::Family(EdgeFamily::Semantic),
        RelationKind::Traversal => RelationSelector::Family(EdgeFamily::Traversal),
        RelationKind::Containment(sub) => RelationSelector::Containment(sub),
        RelationKind::Arrangement(sub) => RelationSelector::Arrangement(sub),
        RelationKind::Imported(sub) => RelationSelector::Imported(sub),
        RelationKind::Provenance(sub) => RelationSelector::Provenance(sub),
    }
}
/// One edge slot per visible projected relation row, including parallel buckets.
pub fn visible_edges(
    graph: &Graph,
    hidden: impl Fn(NodeKey, NodeKey, RelationSelector) -> bool,
) -> Vec<(NodeKey, NodeKey)> {
    graph
        .projected_relations()
        .filter(|(_, row)| !hidden(row.from, row.to, selector_for_relation_kind(row.kind)))
        .map(|(_, row)| (row.from, row.to))
        .collect()
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellOwner {
    pub from: Uuid,
    pub to: Uuid,
    pub stratum: GraphStratum,
    pub source_from: Uuid,
    pub source_to: Uuid,
    pub selector: RelationSelector,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellInput {
    pub owner: CellOwner,
    /// Rows sharing this selector and source owner are grouped; hiding hides this group.
    pub rows: usize,
    pub hidden: bool,
    /// Exact held handles, deduplicated across grouped parallel buckets.
    pub assertions: BTreeSet<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PairInputs {
    pub first: Uuid,
    pub second: Uuid,
    pub held_rows: usize,
    pub visible_rows: usize,
    pub cells: Vec<CellInput>,
}
pub fn inspect_pair(
    graph: &Graph,
    first: Uuid,
    second: Uuid,
    hidden: impl Fn(NodeKey, NodeKey, RelationSelector) -> bool,
) -> Result<PairInputs, String> {
    let a = graph
        .get_node_key_by_id(first)
        .ok_or_else(|| format!("Access {first} is absent from this view"))?;
    let b = graph
        .get_node_key_by_id(second)
        .ok_or_else(|| format!("Access {second} is absent from this view"))?;
    let mut cells = BTreeMap::<String, CellInput>::new();
    for (key, row) in graph
        .projected_relations()
        .filter(|(_, row)| (row.from == a && row.to == b) || (row.from == b && row.to == a))
    {
        let from = graph
            .get_node(row.from)
            .ok_or("Input endpoint departed")?
            .id;
        let to = graph.get_node(row.to).ok_or("Input endpoint departed")?.id;
        let (stratum, source_from, source_to) = match key {
            RelationKey::Surface(_) => (GraphStratum::Surface, from, to),
            RelationKey::Resource(_) => (
                GraphStratum::Resource,
                graph
                    .shown_resource_id(row.from)
                    .ok_or("Resource source binding absent")?,
                graph
                    .shown_resource_id(row.to)
                    .ok_or("Resource target binding absent")?,
            ),
        };
        let selector = selector_for_relation_kind(row.kind);
        let owner = CellOwner {
            from,
            to,
            stratum,
            source_from,
            source_to,
            selector,
        };
        let group = cells
            .entry(serde_json::to_string(&owner).unwrap())
            .or_insert_with(|| CellInput {
                owner,
                rows: 0,
                hidden: hidden(row.from, row.to, selector),
                assertions: BTreeSet::new(),
            });
        group.rows += 1;
        if let Some((_, payload)) = graph
            .projected_relations_between(row.from, row.to)
            .find(|(held, _)| *held == key)
        {
            for statement in payload.semantic_statements() {
                let matches = match row.kind {
                    RelationKind::Semantic(sub) => statement.recognized_sub_kind == Some(sub),
                    RelationKind::OpenPredicate => statement.recognized_sub_kind.is_none(),
                    _ => false,
                };
                if matches {
                    group.assertions.insert(statement.statement_id.clone());
                }
            }
        }
    }
    let cells: Vec<_> = cells.into_values().collect();
    Ok(PairInputs {
        first: first.min(second),
        second: first.max(second),
        held_rows: cells.iter().map(|cell| cell.rows).sum(),
        visible_rows: cells
            .iter()
            .filter(|cell| !cell.hidden)
            .map(|cell| cell.rows)
            .sum(),
        cells,
    })
}
#[cfg(test)]
#[path = "relationship_topology_tests.rs"]
mod tests;
