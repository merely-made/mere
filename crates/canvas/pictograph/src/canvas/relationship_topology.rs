// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! The exact relation-cell input seam shared by the controller and inspector.
//! Cell multiplicity is not assertion multiplicity or a measured force value.
use kernel::graph::{EdgeFamily, Graph, GraphStratum, NodeKey, RelationKind, RelationSelector};
use scenograph::link::{LinkKey, LinkLegacyMember, LinkSource};
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
    pub source: LinkSource,
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
    inspect_pair_with_link_hides(graph, first, second, &BTreeSet::new(), hidden)
}

pub fn inspect_pair_with_link_hides(
    graph: &Graph,
    first: Uuid,
    second: Uuid,
    hidden_links: &BTreeSet<LinkKey>,
    hidden: impl Fn(NodeKey, NodeKey, RelationSelector) -> bool,
) -> Result<PairInputs, String> {
    graph
        .get_node_key_by_id(first)
        .ok_or_else(|| format!("Access {first} is absent from this view"))?;
    graph
        .get_node_key_by_id(second)
        .ok_or_else(|| format!("Access {second} is absent from this view"))?;
    let mut cells = BTreeMap::<String, CellInput>::new();
    let source = LinkSource::Unavailable {
        reason: "Pictograph receives a kernel graph without its owning source authority or public revision".into(),
    };
    let pair_reading = super::link_reading::read_link(graph, first, second, source)?;
    let source = pair_reading.source.clone();
    for reading in pair_reading.links {
        let link_hidden = hidden_links.contains(&reading.key);
        let stratum = match reading.key.stratum {
            scenograph::link::LinkStratum::Surface => GraphStratum::Surface,
            scenograph::link::LinkStratum::Resource => GraphStratum::Resource,
        };
        for bucket in reading.buckets {
            let from: Uuid = bucket
                .displayed_from
                .parse()
                .map_err(|_| "Invalid displayed source identity")?;
            let to: Uuid = bucket
                .displayed_to
                .parse()
                .map_err(|_| "Invalid displayed target identity")?;
            let from_key = graph
                .get_node_key_by_id(from)
                .ok_or("Input endpoint departed")?;
            let to_key = graph
                .get_node_key_by_id(to)
                .ok_or("Input endpoint departed")?;
            let source_from = bucket
                .source_from
                .id
                .parse()
                .map_err(|_| "Invalid source endpoint identity")?;
            let source_to = bucket
                .source_to
                .id
                .parse()
                .map_err(|_| "Invalid target endpoint identity")?;
            let mut rows = Vec::<(RelationKind, Option<String>)>::new();
            rows.extend(bucket.assertions.into_iter().map(|assertion| {
                let kind = RelationKind::from_predicate_iri(&assertion.predicate)
                    .unwrap_or(RelationKind::OpenPredicate);
                (kind, Some(assertion.id))
            }));
            rows.extend(
                bucket
                    .legacy_members
                    .iter()
                    .filter_map(legacy_relation_kind)
                    .map(|kind| (kind, None)),
            );
            for (kind, assertion) in rows {
                let selector = selector_for_relation_kind(kind);
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
                        hidden: link_hidden || hidden(from_key, to_key, selector),
                        assertions: BTreeSet::new(),
                    });
                group.rows += 1;
                if let Some(assertion) = assertion {
                    group.assertions.insert(assertion);
                }
            }
        }
    }
    let cells: Vec<_> = cells.into_values().collect();
    Ok(PairInputs {
        first: first.min(second),
        second: first.max(second),
        source,
        held_rows: cells.iter().map(|cell| cell.rows).sum(),
        visible_rows: cells
            .iter()
            .filter(|cell| !cell.hidden)
            .map(|cell| cell.rows)
            .sum(),
        cells,
    })
}

fn legacy_relation_kind(member: &LinkLegacyMember) -> Option<RelationKind> {
    if member.family == "Traversal" {
        return Some(RelationKind::Traversal);
    }
    let sub_kind = member.sub_kind.as_ref()?;
    let variant = match member.family.as_str() {
        "Semantic" => "Semantic",
        "Containment" => "Containment",
        "Arrangement" => "Arrangement",
        "Imported" => "Imported",
        "Provenance" => "Provenance",
        _ => return None,
    };
    serde_json::from_value(serde_json::json!({ (variant): sub_kind })).ok()
}
#[cfg(test)]
#[path = "relationship_topology_tests.rs"]
mod tests;
