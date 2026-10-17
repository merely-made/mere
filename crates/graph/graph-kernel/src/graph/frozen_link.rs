// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Immutable, source-qualified link readings borne by Resources.

use super::{CoverageNote, Graph, GraphStratum};
use crate::persistence::{PersistedEdge, PersistedResourceFacet, PersistedResourceRecord};
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use uuid::Uuid;

pub const FROZEN_LINK: &str = "graph.frozen-link/v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenLinkEndpoint {
    pub id: Uuid,
    pub resource: Option<Uuid>,
    pub address: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenSourceBinding {
    pub authority: String,
    pub domain: String,
    pub resource: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrozenSourceRevision {
    Available { revision: String },
    Unavailable { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum FrozenLinkSource {
    Available {
        binding: FrozenSourceBinding,
        revision: FrozenSourceRevision,
    },
    Unavailable {
        reason: String,
    },
}

impl FrozenLinkSource {
    fn validate(&self) -> Result<(), String> {
        let nonempty = |value: &str| !value.trim().is_empty() && value == value.trim();
        match self {
            Self::Available { binding, revision } => {
                if !nonempty(&binding.authority)
                    || !nonempty(&binding.domain)
                    || !nonempty(&binding.resource)
                {
                    return Err("frozen source binding fields must be nonempty and trimmed".into());
                }
                match revision {
                    FrozenSourceRevision::Available { revision } if nonempty(revision) => Ok(()),
                    FrozenSourceRevision::Available { .. } => {
                        Err("frozen source revision must be nonempty and trimmed".into())
                    },
                    FrozenSourceRevision::Unavailable { reason } if nonempty(reason) => Ok(()),
                    FrozenSourceRevision::Unavailable { .. } => {
                        Err("unavailable source revision requires a reason".into())
                    },
                }
            },
            Self::Unavailable { reason } if nonempty(reason) => Ok(()),
            Self::Unavailable { .. } => Err("unavailable frozen source requires a reason".into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenLink {
    version: u32,
    stratum: GraphStratum,
    first: FrozenLinkEndpoint,
    second: FrozenLinkEndpoint,
    revision: u64,
    source: FrozenLinkSource,
    coverage: CoverageNote,
    statements: Vec<PersistedEdge>,
}

impl FrozenLink {
    pub fn stratum(&self) -> GraphStratum {
        self.stratum
    }
    pub fn first(&self) -> &FrozenLinkEndpoint {
        &self.first
    }
    pub fn second(&self) -> &FrozenLinkEndpoint {
        &self.second
    }
    /// The local structural revision observed at freeze time; not a public source revision.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn source(&self) -> &FrozenLinkSource {
        &self.source
    }
    pub fn coverage(&self) -> &CoverageNote {
        &self.coverage
    }
    pub fn statements(&self) -> &[PersistedEdge] {
        &self.statements
    }
}

pub struct OpenedFrozenLink<'a> {
    pub link: &'a FrozenLink,
    pub coverage: CoverageNote,
}

pub(super) fn validate_frozen_link_record(
    record: &PersistedResourceRecord,
) -> Result<Option<FrozenLink>, String> {
    let facets = record
        .facets
        .iter()
        .filter(|facet| facet.facet == FROZEN_LINK)
        .collect::<Vec<_>>();
    if facets.len() > 1 {
        return Err("duplicate frozen link facets".into());
    }
    let Some(facet) = facets.first() else {
        return Ok(None);
    };
    let raw: serde_json::Value = serde_json::from_str(&facet.value_json)
        .map_err(|error| format!("invalid frozen link: {error}"))?;
    let link: FrozenLink = serde_json::from_value(raw.clone())
        .map_err(|error| format!("invalid frozen link: {error}"))?;
    let mut known = serde_json::to_value(&link).map_err(|error| error.to_string())?;
    for edge in known["statements"]
        .as_array_mut()
        .expect("serialized edge array")
    {
        if let Some(statements) = edge
            .get_mut("assertions")
            .and_then(serde_json::Value::as_array_mut)
        {
            super::frozen_selection::normalize_statement_defaults(statements);
        }
        if let Some(statements) = edge
            .get_mut("semantic")
            .and_then(|semantic| semantic.get_mut("statements"))
            .and_then(serde_json::Value::as_array_mut)
        {
            super::frozen_selection::normalize_statement_defaults(statements);
        }
    }
    super::frozen_selection::validate_retained_keys(&raw, &known)?;
    if link.version != 1 {
        return Err("unsupported frozen link version".into());
    }
    link.source.validate()?;
    if link.first.id > link.second.id {
        return Err("frozen link endpoints must be in canonical order".into());
    }
    for endpoint in [&link.first, &link.second] {
        if endpoint.id.is_nil()
            || endpoint.resource.is_some_and(|id| id.is_nil())
            || endpoint
                .address
                .as_ref()
                .is_some_and(|address| address.trim().is_empty() || address != address.trim())
        {
            return Err("invalid frozen link endpoint".into());
        }
    }
    if link.stratum == GraphStratum::Resource
        && (link.first.resource != Some(link.first.id)
            || link.second.resource != Some(link.second.id))
    {
        return Err("resource link endpoints must retain resource identity".into());
    }
    if link.stratum == GraphStratum::Resource
        && [&link.first, &link.second].into_iter().any(|endpoint| {
            endpoint.address.as_ref().is_none_or(|address| {
                chartulary::resource_id_from_canonical_iri(address) != endpoint.id
            })
        })
    {
        return Err("resource endpoint address does not match its identity".into());
    }
    if link.statements.is_empty() {
        return Err("frozen link must retain at least one pair bucket".into());
    }
    let mut handles = BTreeMap::new();
    for edge in &link.statements {
        let from = edge
            .from_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen link source id")?;
        let to = edge
            .to_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen link target id")?;
        let valid_pair = (from == link.first.id && to == link.second.id)
            || (from == link.second.id && to == link.first.id);
        if from.to_string() != edge.from_node_id || to.to_string() != edge.to_node_id || !valid_pair
        {
            return Err("frozen link bucket is outside its canonical endpoint pair".into());
        }
        for statement in edge.retained_assertions() {
            if statement.statement_id.trim().is_empty() {
                return Err("frozen link assertion id must be nonempty".into());
            }
            let key = statement.statement_id.clone();
            let value = (from, to, statement.clone());
            if let Some(previous) = handles.get(&key) {
                if previous == &value {
                    return Err("duplicate frozen assertion handle".into());
                }
                return Err("conflicting frozen assertion handle".into());
            }
            handles.insert(key, value);
        }
        let payload = super::snapshot::payload_from_persisted(edge);
        if super::snapshot::persisted_edge_for_ids(from, to, &payload) != *edge {
            return Err("inconsistent frozen link bucket payload".into());
        }
    }
    Ok(Some(link))
}

impl Graph {
    fn link_coverage(&self, resources: &[Uuid], surfaces: &[Uuid]) -> CoverageNote {
        let resources = resources
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let surfaces = surfaces
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let mut note = self.known_coverage.clone();
        note.limits.retain(|limit| {
            (limit.resources.is_empty() && limit.surfaces.is_empty())
                || limit.resources.iter().any(|id| resources.contains(id))
                || limit.surfaces.iter().any(|id| surfaces.contains(id))
        });
        let missing_resources = resources
            .iter()
            .copied()
            .filter(|id| self.resource(*id).is_none())
            .filter(|id| {
                !note.limits.iter().any(|limit| {
                    limit.layer == super::CoverageLayer::Residency
                        && ((limit.resources.is_empty() && limit.surfaces.is_empty())
                            || limit.resources.contains(id))
                })
            })
            .collect::<Vec<_>>();
        if !missing_resources.is_empty() {
            let mut limit = super::CoverageLimit::new(
                super::CoverageLayer::Possession,
                "referenced resources are unavailable",
            );
            limit.count = Some(missing_resources.len());
            limit.resources = missing_resources;
            note.push(limit);
        }
        let missing_surfaces = surfaces
            .iter()
            .copied()
            .filter(|id| self.get_node_key_by_id(*id).is_none())
            .filter(|id| {
                !note.limits.iter().any(|limit| {
                    limit.layer == super::CoverageLayer::Residency
                        && ((limit.resources.is_empty() && limit.surfaces.is_empty())
                            || limit.surfaces.contains(id))
                })
            })
            .collect::<Vec<_>>();
        if !missing_surfaces.is_empty() {
            let mut limit = super::CoverageLimit::new(
                super::CoverageLayer::Possession,
                "referenced surfaces are unavailable",
            );
            limit.count = Some(missing_surfaces.len());
            limit.surfaces = missing_surfaces;
            note.push(limit);
        }
        note
    }

    /// Read a current pair without minting a Resource. An empty pair is a valid
    /// reading result for drift comparison, but cannot be frozen.
    pub fn read_link_snapshot(
        &self,
        stratum: GraphStratum,
        first: Uuid,
        second: Uuid,
        source: FrozenLinkSource,
    ) -> Result<FrozenLink, String> {
        source.validate()?;
        let (first, second) = if first <= second {
            (first, second)
        } else {
            (second, first)
        };
        let (first_endpoint, second_endpoint, statements) = match stratum {
            GraphStratum::Surface => {
                let (first_key, first_node) = self
                    .get_node_by_id(first)
                    .ok_or("missing first Surface link endpoint")?;
                let (second_key, second_node) = self
                    .get_node_by_id(second)
                    .ok_or("missing second Surface link endpoint")?;
                let endpoints = (
                    FrozenLinkEndpoint {
                        id: first,
                        resource: self.shown_resource_id(first_key),
                        address: Some(first_node.url().to_owned()),
                    },
                    FrozenLinkEndpoint {
                        id: second,
                        resource: self.shown_resource_id(second_key),
                        address: Some(second_node.url().to_owned()),
                    },
                );
                let rows = self
                    .inner
                    .inner()
                    .edge_references()
                    .filter_map(|edge| {
                        let from = self.inner.node(edge.source())?.id;
                        let to = self.inner.node(edge.target())?.id;
                        ((from == first && to == second) || (from == second && to == first)).then(
                            || super::snapshot::persisted_edge_for_ids(from, to, edge.weight()),
                        )
                    })
                    .collect::<Vec<_>>();
                (endpoints.0, endpoints.1, rows)
            },
            GraphStratum::Resource => {
                let first_resource = self
                    .resource(first)
                    .ok_or("missing first Resource endpoint")?;
                let second_resource = self
                    .resource(second)
                    .ok_or("missing second Resource endpoint")?;
                let endpoints = (
                    FrozenLinkEndpoint {
                        id: first,
                        resource: Some(first),
                        address: Some(first_resource.canonical_iri().to_owned()),
                    },
                    FrozenLinkEndpoint {
                        id: second,
                        resource: Some(second),
                        address: Some(second_resource.canonical_iri().to_owned()),
                    },
                );
                let rows = self
                    .resource_edges()
                    .filter_map(|(from, to, payload)| {
                        let (from, to) = (from.id(), to.id());
                        ((from == first && to == second) || (from == second && to == first))
                            .then(|| super::snapshot::persisted_edge_for_ids(from, to, payload))
                    })
                    .collect::<Vec<_>>();
                (endpoints.0, endpoints.1, rows)
            },
        };
        let resource_refs = [first_endpoint.resource, second_endpoint.resource]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let surface_refs = if stratum == GraphStratum::Surface {
            vec![first, second]
        } else {
            Vec::new()
        };
        Ok(FrozenLink {
            version: 1,
            stratum,
            first: first_endpoint,
            second: second_endpoint,
            revision: self.revision(),
            source,
            coverage: self.link_coverage(&resource_refs, &surface_refs),
            statements,
        })
    }

    /// Freeze a link in one graph stratum as a distinct immutable Resource.
    pub fn freeze_link_with_nonce(
        &mut self,
        stratum: GraphStratum,
        first: Uuid,
        second: Uuid,
        source: FrozenLinkSource,
        nonce: Uuid,
    ) -> Result<Uuid, String> {
        if nonce.is_nil() {
            return Err("frozen Resource nonce must be non-nil".into());
        }
        let link = self.read_link_snapshot(stratum, first, second, source)?;
        if link.statements.is_empty() {
            return Err("cannot freeze a link with no pair buckets".into());
        }
        let iri = format!("urn:uuid:{nonce}");
        let owner = super::ResourceNode::for_term(&iri);
        if self.resource(owner.id()).is_some() {
            return Err("frozen Resource nonce is already in use".into());
        }
        let record = PersistedResourceRecord {
            canonical_iri: iri,
            facets: vec![PersistedResourceFacet {
                facet: FROZEN_LINK.into(),
                value_json: serde_json::to_string(&link).map_err(|error| error.to_string())?,
            }],
        };
        validate_frozen_link_record(&record)?;
        if !self.set_resource_record(owner.id(), Some(record.clone())) {
            return Err("frozen Resource admission refused".into());
        }
        self.record_delta(&super::CapturedDelta::ReplaySetResourceRecordById {
            resource_id: owner.id().to_string(),
            record: Some(record),
        });
        Ok(owner.id())
    }

    pub fn open_frozen_link(&self, owner: Uuid) -> Result<OpenedFrozenLink<'_>, String> {
        let link = self
            .resource(owner)
            .and_then(|resource| resource.nested_link())
            .ok_or("resource bears no frozen link")?;
        let resource_refs = [link.first.resource, link.second.resource]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let surface_refs = if link.stratum == GraphStratum::Surface {
            vec![link.first.id, link.second.id]
        } else {
            Vec::new()
        };
        let coverage = self.link_coverage(&resource_refs, &surface_refs);
        Ok(OpenedFrozenLink { link, coverage })
    }
}
