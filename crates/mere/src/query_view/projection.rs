// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::resource::TAGGED_WITH_IRI;
use kernel::graph::{CoverageNote, EdgePayload, GraphStratum, SemanticStatement};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryNodeId {
    pub stratum: GraphStratum,
    pub id: Uuid,
}
impl QueryNodeId {
    pub fn resource(id: Uuid) -> Self {
        Self {
            stratum: GraphStratum::Resource,
            id,
        }
    }
    pub fn surface(id: Uuid) -> Self {
        Self {
            stratum: GraphStratum::Surface,
            id,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueryNode {
    pub identity: QueryNodeId,
    pub title: String,
    pub address: String,
    pub member: bool,
    pub tags: Vec<String>,
    /// Empty means this Resource has no held access; not a new opening request.
    pub held_accesses: Vec<Uuid>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VisualRelationKind {
    ResourceLink,
    SurfaceLink,
    ShownBinding,
}
impl VisualRelationKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::ResourceLink => "Resource link",
            Self::SurfaceLink => "Surface link",
            Self::ShownBinding => "Cross-stratum edge: shown binding",
        }
    }
    pub fn cue(self, styles: &RelationStyles) -> StrokeCue {
        match self {
            Self::ResourceLink => styles.resource_link,
            Self::SurfaceLink => styles.surface_link,
            Self::ShownBinding => styles.cross_stratum_edge,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryRelation {
    pub from: QueryNodeId,
    pub to: QueryNodeId,
    pub kind: VisualRelationKind,
    /// The source identity survives lifting onto Surface endpoints.
    pub source_from: QueryNodeId,
    pub source_to: QueryNodeId,
    /// All retained assertion records from the source edge, across every
    /// relation family. `statements` remains the semantic-only compatibility
    /// API; this field carries canonical non-semantic records and tombstones.
    #[serde(default)]
    pub assertions: Option<Vec<SemanticStatement>>,
    pub statements: Vec<SemanticStatement>,
    pub families: serde_json::Value,
    /// Shown bindings keep the existing zero-hop cost and no assertion handle.
    pub hop_cost: u8,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct QueryProjection {
    pub owner: Uuid,
    pub source_revision: u64,
    pub spec: QueryViewSpec,
    pub members: Vec<Uuid>,
    pub missing_resources: Vec<Uuid>,
    pub coverage: CoverageNote,
    pub nodes: Vec<QueryNode>,
    pub relations: Vec<QueryRelation>,
}
impl QueryProjection {
    pub fn is_current(&self, graph: &Graph) -> bool {
        graph.revision() == self.source_revision
            && read_spec(graph, self.owner).as_ref() == Ok(&self.spec)
    }
    pub fn access_for(&self, graph: &Graph, resource: Uuid, access: Uuid) -> Result<(), String> {
        if !self.is_current(graph) {
            return Err("Query view changed; refresh before opening an access".into());
        }
        if !self.nodes.iter().any(|node| {
            node.identity == QueryNodeId::resource(resource) && node.held_accesses.contains(&access)
        }) {
            return Err("Access is not an appearance in this query view".into());
        }
        let key = graph
            .get_node_key_by_id(access)
            .ok_or("Held access disappeared")?;
        if graph.shown_resource_id(key) != Some(resource) {
            return Err("Access now shows a different Resource".into());
        }
        Ok(())
    }
}

fn source_relation(
    from: QueryNodeId,
    to: QueryNodeId,
    kind: VisualRelationKind,
    payload: &EdgePayload,
) -> QueryRelation {
    QueryRelation {
        from,
        to,
        kind,
        source_from: from,
        source_to: to,
        assertions: Some(payload.assertions().to_vec()),
        statements: payload.semantic_statements().to_vec(),
        families: serde_json::to_value(payload).expect("held edge payload serializes"),
        hop_cost: 1,
    }
}

/// A pure projection: it neither selects existing accesses nor invents new
/// ones, and it cannot write to source Resources, assertions or Forme positions.
pub fn project_view(graph: &Graph, owner: Uuid) -> Result<QueryProjection, String> {
    let spec = read_spec(graph, owner)?;
    let derived = spec.members(graph)?;
    let members: BTreeSet<_> = derived.members.iter().copied().collect();
    let mut resources = members.clone();
    if spec.include_tag_concepts {
        for (from, to, payload) in graph.resource_edges() {
            if members.contains(&from.id())
                && payload
                    .semantic_statements()
                    .iter()
                    .any(|s| s.predicate == TAGGED_WITH_IRI)
            {
                resources.insert(to.id());
                if resources.len() > spec.max_members {
                    return Err("Query tag context exceeds the configured member budget".into());
                }
            }
        }
    }
    if resources.len() > spec.max_members {
        return Err("Query tag context exceeds the configured member budget".into());
    }
    let mut nodes = vec![];
    let mut surfaces = BTreeSet::new();
    let mut held_access_count = 0usize;
    let mut relations = vec![];
    for id in &resources {
        let resource = graph
            .resource(*id)
            .ok_or("Selected Resource is no longer held")?;
        let accesses = graph.surface_ids_showing_resource(*id);
        held_access_count = held_access_count.saturating_add(accesses.len());
        if held_access_count > spec.max_appearances {
            return Err("Query exceeds the configured appearance budget".into());
        }
        let mut tags: Vec<_> = graph.resource_tag_labels(*id).into_iter().collect();
        tags.sort();
        let title = graph
            .resource_tag_concept(*id)
            .map(|concept| format!("{} ({})", concept.label, concept.owner_iri))
            .unwrap_or_else(|| resource.canonical_iri().to_owned());
        nodes.push(QueryNode {
            identity: QueryNodeId::resource(*id),
            title,
            address: resource.canonical_iri().to_owned(),
            member: members.contains(id),
            tags,
            held_accesses: accesses.clone(),
        });
        if spec.appearances == AppearancePolicy::ResourcesOnly {
            continue;
        }
        for access in accesses {
            if relations.len() >= spec.max_relations {
                return Err("Query exceeds the configured relation budget".into());
            }
            let (_, node) = graph
                .get_node_by_id(access)
                .ok_or("Recorded access is no longer held")?;
            surfaces.insert(access);
            nodes.push(QueryNode {
                identity: QueryNodeId::surface(access),
                title: node.title.clone(),
                address: node.url().to_owned(),
                member: members.contains(id),
                tags: vec![],
                held_accesses: vec![access],
            });
            relations.push(QueryRelation {
                from: QueryNodeId::surface(access),
                to: QueryNodeId::resource(*id),
                kind: VisualRelationKind::ShownBinding,
                source_from: QueryNodeId::surface(access),
                source_to: QueryNodeId::resource(*id),
                assertions: Some(vec![]),
                statements: vec![],
                families: serde_json::Value::Null,
                hop_cost: 0,
            });
        }
    }
    for (from, to, payload) in graph.resource_edges() {
        if resources.contains(&from.id()) && resources.contains(&to.id()) {
            let relation = source_relation(
                QueryNodeId::resource(from.id()),
                QueryNodeId::resource(to.id()),
                VisualRelationKind::ResourceLink,
                payload,
            );
            if relations.len() >= spec.max_relations {
                return Err("Query exceeds the configured relation budget".into());
            }
            relations.push(relation.clone());
            // Lift the held source link onto its explicit access pairs. This
            // remains a Resource link, independent of the displayed endpoints.
            if spec.appearances == AppearancePolicy::AllHeldAccesses {
                let from_accesses = graph.surface_ids_showing_resource(from.id());
                let to_accesses = graph.surface_ids_showing_resource(to.id());
                if relations
                    .len()
                    .saturating_add(from_accesses.len().saturating_mul(to_accesses.len()))
                    > spec.max_relations
                {
                    return Err("Query lifted links exceed the configured relation budget".into());
                }
                for a in from_accesses {
                    for b in to_accesses.iter().copied() {
                        let mut lifted = relation.clone();
                        lifted.from = QueryNodeId::surface(a);
                        lifted.to = QueryNodeId::surface(b);
                        relations.push(lifted);
                    }
                }
            }
        }
    }
    for (_, from, to, payload) in graph.surface_relation_buckets() {
        let (Some(from), Some(to)) = (graph.get_node(from), graph.get_node(to)) else {
            continue;
        };
        if surfaces.contains(&from.id) && surfaces.contains(&to.id) {
            if relations.len() >= spec.max_relations {
                return Err("Query exceeds the configured relation budget".into());
            }
            relations.push(source_relation(
                QueryNodeId::surface(from.id),
                QueryNodeId::surface(to.id),
                VisualRelationKind::SurfaceLink,
                payload,
            ));
        }
    }
    Ok(QueryProjection {
        owner,
        source_revision: graph.revision(),
        spec,
        members: derived.members,
        missing_resources: derived.missing_resources,
        coverage: derived.coverage,
        nodes,
        relations,
    })
}
