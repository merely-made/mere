// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Immutable, source-qualified link readings borne by Resources.

use super::{CoverageNote, Graph, GraphStratum};
use crate::persistence::{PersistedResourceFacet, PersistedResourceRecord};
pub use mere::model::frozen::{
    FROZEN_LINK, FrozenLink, FrozenLinkEndpoint, FrozenLinkSource, FrozenSourceBinding,
    FrozenSourceRevision,
};
pub use mere::model::frozen_rules::OpenedFrozenLink;
pub(crate) use mere::model::frozen_rules::validate_frozen_link_record;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use uuid::Uuid;

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
        Ok(FrozenLink::v1(
            stratum,
            first_endpoint,
            second_endpoint,
            self.revision(),
            source,
            self.link_coverage(&resource_refs, &surface_refs),
            statements,
        ))
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
        if link.statements().is_empty() {
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
        let resource_refs = [link.first().resource, link.second().resource]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();
        let surface_refs = if link.stratum() == GraphStratum::Surface {
            vec![link.first().id, link.second().id]
        } else {
            Vec::new()
        };
        let coverage = self.link_coverage(&resource_refs, &surface_refs);
        Ok(OpenedFrozenLink { link, coverage })
    }
}
