// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resource-owned query definitions and read-only projections of held strata.
use forme::{SubgraphKind, SubgraphSpec};
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use kernel::graph::{Graph, ResourceNode};
use kernel::persistence::{PersistedResourceFacet, PersistedResourceRecord};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod projection;
pub use projection::*;
mod stroke;
pub use stroke::{StrokePart, stroke_parts};
#[cfg(test)]
mod tests;

pub const QUERY_VIEW_FACET: &str = "graph.query-view/v1";

/// Non-color stroke cues belong to the authored view, independently of theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StrokeCue {
    Solid,
    Dashed,
    Dotted,
    Double,
}
impl StrokeCue {
    pub const ALL: [Self; 4] = [Self::Solid, Self::Dashed, Self::Dotted, Self::Double];
    pub fn css(self) -> &'static str {
        match self {
            Self::Solid => "solid",
            Self::Dashed => "dashed",
            Self::Dotted => "dotted",
            Self::Double => "double",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelationStyles {
    pub resource_link: StrokeCue,
    pub surface_link: StrokeCue,
    pub cross_stratum_edge: StrokeCue,
}
impl Default for RelationStyles {
    fn default() -> Self {
        Self {
            resource_link: StrokeCue::Solid,
            surface_link: StrokeCue::Solid,
            cross_stratum_edge: StrokeCue::Dashed,
        }
    }
}

/// A view uses every held access to each selected Resource; no access is minted
/// or selected on behalf of the source. Undrawn members remain explicit rows.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppearancePolicy {
    AllHeldAccesses,
    ResourcesOnly,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueryViewSpec {
    pub version: u32,
    pub title: String,
    pub definition: SubgraphSpec,
    pub appearances: AppearancePolicy,
    pub styles: RelationStyles,
    pub include_tag_concepts: bool,
    /// Explicit presentation budget. Exceeding it is a refusal, never an empty
    /// or silently truncated membership. It is not an evaluator time limit.
    pub max_members: usize,
    pub max_appearances: usize,
    pub max_relations: usize,
}
impl QueryViewSpec {
    pub fn select(title: impl Into<String>, query: impl Into<String>) -> Self {
        Self {
            version: 1,
            title: title.into(),
            definition: SubgraphSpec {
                kind: SubgraphKind::Sparql {
                    query: query.into(),
                    member_variable: "member".into(),
                },
                anchors: vec![],
                primary_anchor: None,
                selectors: vec![],
            },
            appearances: AppearancePolicy::AllHeldAccesses,
            styles: RelationStyles::default(),
            include_tag_concepts: true,
            max_members: 256,
            max_appearances: 256,
            max_relations: 1024,
        }
    }
    pub fn query_text(&self) -> &str {
        match &self.definition.kind {
            SubgraphKind::Sparql { query, .. } => query,
            _ => "",
        }
    }
    pub fn set_query_text(&mut self, text: impl Into<String>) -> Result<(), String> {
        let SubgraphKind::Sparql { query, .. } = &mut self.definition.kind else {
            return Err("Query view requires SELECT membership".into());
        };
        *query = text.into();
        Ok(())
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.version != 1 {
            return Err("Unsupported query view version".into());
        }
        if self.title.trim().is_empty()
            || self.max_members == 0
            || self.max_appearances == 0
            || self.max_relations == 0
        {
            return Err("A query view needs a title and positive budgets".into());
        }
        let SubgraphKind::Sparql {
            query,
            member_variable,
        } = &self.definition.kind
        else {
            return Err("A query view requires a saved SELECT definition".into());
        };
        if query.trim().is_empty() || member_variable.is_empty() {
            return Err("A query and member column are required".into());
        }
        if !self.definition.anchors.is_empty()
            || self.definition.primary_anchor.is_some()
            || !self.definition.selectors.is_empty()
        {
            return Err("SELECT membership does not use Surface seeds or walk selectors".into());
        }
        if self.styles.cross_stratum_edge == self.styles.resource_link
            || self.styles.cross_stratum_edge == self.styles.surface_link
        {
            return Err("Cross-stratum edges need a distinct stroke cue from links".into());
        }
        Ok(())
    }
    pub fn members(&self, graph: &Graph) -> Result<crate::subgraph::DerivedMembers, String> {
        self.validate()?;
        let result = crate::subgraph::try_derive_members(graph, &self.definition)?;
        if result.members.len() + result.missing_resources.len() > self.max_members {
            return Err(format!(
                "Query membership exceeds the configured {} member budget",
                self.max_members
            ));
        }
        Ok(result)
    }
}

pub fn read_spec(graph: &Graph, owner: Uuid) -> Result<QueryViewSpec, String> {
    let record = graph
        .resource_record(owner)
        .ok_or("Query view Resource is not held")?;
    let mut facets = record
        .facets
        .iter()
        .filter(|facet| facet.facet == QUERY_VIEW_FACET);
    let facet = facets
        .next()
        .ok_or("Resource has no query view definition")?;
    if facets.next().is_some() {
        return Err("Ambiguous query view definition".into());
    }
    let raw: serde_json::Value = serde_json::from_str(&facet.value_json)
        .map_err(|e| format!("Invalid query view definition: {e}"))?;
    let spec: QueryViewSpec = serde_json::from_value(raw.clone())
        .map_err(|e| format!("Invalid query view definition: {e}"))?;
    if raw != serde_json::to_value(&spec).map_err(|e| e.to_string())? {
        return Err("Query view contains unsupported nested definition fields".into());
    }
    spec.validate()?;
    Ok(spec)
}

/// Explicit authoring writes only the view Resource. Existing graph facts,
/// Surface identities and placements remain with their original owners.
pub fn save_spec(
    graph: &mut Graph,
    owner: Option<Uuid>,
    expected: Option<&QueryViewSpec>,
    nonce: Uuid,
    spec: &QueryViewSpec,
) -> Result<Uuid, String> {
    spec.members(graph)?;
    let (id, mut record) = match owner {
        Some(id) => {
            if graph
                .resource(id)
                .is_some_and(|resource| resource.nested_selection().is_some())
            {
                return Err("Frozen selections cannot be edited as query views".into());
            }
            let held = read_spec(graph, id)?;
            if expected != Some(&held) {
                return Err("Query view changed since this draft opened".into());
            }
            (
                id,
                graph
                    .resource_record(id)
                    .ok_or("Query view Resource disappeared")?,
            )
        },
        None => {
            if expected.is_some() || nonce.is_nil() {
                return Err("New query view requires a fresh Resource nonce".into());
            }
            let iri = format!("urn:uuid:{nonce}");
            let id = ResourceNode::for_term(&iri).id();
            if graph.resource(id).is_some() {
                return Err("Query view Resource nonce was already used".into());
            }
            (
                id,
                PersistedResourceRecord {
                    canonical_iri: iri,
                    facets: vec![],
                },
            )
        },
    };
    let value = serde_json::to_string(spec).map_err(|e| e.to_string())?;
    record
        .facets
        .retain(|facet| facet.facet != QUERY_VIEW_FACET);
    record.facets.push(PersistedResourceFacet {
        facet: QUERY_VIEW_FACET.into(),
        value_json: value,
    });
    apply_graph_delta(
        graph,
        GraphDelta::ReplaySetResourceRecordById {
            resource_id: id,
            record: Some(record),
        },
    );
    if read_spec(graph, id).as_ref() != Ok(spec) {
        return Err("Query view admission refused".into());
    }
    Ok(id)
}

pub fn shared_spec(graph: &Graph, owner: Uuid) -> Result<String, String> {
    serde_json::to_string_pretty(&read_spec(graph, owner)?).map_err(|e| e.to_string())
}

pub fn freeze_view(graph: &mut Graph, owner: Uuid, nonce: Uuid) -> Result<Uuid, String> {
    let spec = read_spec(graph, owner)?;
    let members = spec.members(graph)?;
    if !members.missing_resources.is_empty() {
        return Err("Cannot freeze unavailable query members".into());
    }
    graph.freeze_resource_selection_with_nonce(
        serde_json::to_value(&spec.definition).map_err(|e| e.to_string())?,
        members.members,
        nonce,
    )
}
