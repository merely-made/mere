// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Validate explicit resource columns and navigation indices before
//! materializing a graph.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use uuid::Uuid;

use crate::graph::assertion_handles::{AssertionHandleRepeat, AssertionHandles};
use crate::graph::resource_content::ContentError;
use crate::graph::{Graph, GraphStratum, NavigationIndexError};
use crate::persistence::GraphSnapshot;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceSnapshotError {
    UnsupportedAssertionSchema {
        schema: u32,
    },
    MixedAssertionFormat {
        column: &'static str,
        edge_index: usize,
    },
    MissingCanonicalAssertions {
        column: &'static str,
        edge_index: usize,
    },
    InvalidUuid {
        column: &'static str,
        index: usize,
        role: &'static str,
        value: String,
    },
    NonCanonicalUuid {
        column: &'static str,
        edge_index: usize,
        role: &'static str,
        value: String,
    },
    InvalidFacet {
        resource_id: Uuid,
        facet: String,
        detail: String,
    },
    DuplicateFacet {
        resource_id: Uuid,
        facet: String,
    },
    ConflictingResource {
        resource_id: Uuid,
        first_index: usize,
        index: usize,
    },
    MissingResource {
        column: &'static str,
        index: usize,
        resource_id: Uuid,
    },
    MissingSurface {
        column: &'static str,
        index: usize,
        surface_id: Uuid,
    },
    LegacySemanticAggregate {
        edge_index: usize,
    },
    RepeatedAssertionHandle(AssertionHandleRepeat),
    ConflictingShownResource {
        surface_id: Uuid,
        first_resource: Uuid,
        resource_id: Uuid,
    },
    InvalidNavigation(NavigationIndexError),
    InvalidSurfaceId {
        index: usize,
        value: String,
    },
    RepeatedSurfaceId {
        surface_id: Uuid,
        first_index: usize,
        index: usize,
    },
    /// Installing facets would leave the graph failing checked open.
    InvalidFacets(ContentError),
}

impl fmt::Display for ResourceSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedAssertionSchema { schema } => {
                write!(f, "unsupported assertion schema {schema}")
            },
            Self::MixedAssertionFormat { column, edge_index } => write!(
                f,
                "{column}[{edge_index}] mixes canonical assertions with legacy assertion data"
            ),
            Self::MissingCanonicalAssertions { column, edge_index } => write!(
                f,
                "{column}[{edge_index}] has legacy assertion data without canonical assertions"
            ),
            Self::InvalidUuid {
                column,
                index,
                role,
                value,
            } => write!(f, "{column}[{index}] has invalid {role} UUID {value:?}"),
            Self::NonCanonicalUuid {
                column,
                edge_index,
                role,
                value,
            } => write!(
                f,
                "{column}[{edge_index}] has noncanonical {role} UUID {value:?}"
            ),
            Self::InvalidFacet {
                resource_id,
                facet,
                detail,
            } => write!(
                f,
                "resource {resource_id} facet {facet:?} has invalid JSON: {detail}"
            ),
            Self::DuplicateFacet { resource_id, facet } => {
                write!(f, "resource {resource_id} repeats facet {facet:?}")
            },
            Self::ConflictingResource {
                resource_id,
                first_index,
                index,
            } => write!(
                f,
                "resources[{index}] conflicts with resources[{first_index}] for {resource_id}"
            ),
            Self::MissingResource {
                column,
                index,
                resource_id,
            } => write!(
                f,
                "{column}[{index}] references absent resource {resource_id}"
            ),
            Self::MissingSurface {
                column,
                index,
                surface_id,
            } => write!(
                f,
                "{column}[{index}] references absent surface {surface_id}"
            ),
            Self::LegacySemanticAggregate { edge_index } => write!(
                f,
                "resource_edges[{edge_index}] has semantic data without exact assertion handles"
            ),
            Self::RepeatedAssertionHandle(repeat) => fmt::Display::fmt(repeat, f),
            Self::ConflictingShownResource {
                surface_id,
                first_resource,
                resource_id,
            } => write!(
                f,
                "surface {surface_id} shows conflicting resources {first_resource} and {resource_id}"
            ),
            Self::InvalidNavigation(error) => fmt::Display::fmt(error, f),
            Self::InvalidSurfaceId { index, value } => {
                write!(f, "nodes[{index}] has invalid surface UUID {value:?}")
            },
            Self::RepeatedSurfaceId {
                surface_id,
                first_index,
                index,
            } => write!(
                f,
                "nodes[{index}] repeats surface {surface_id} of nodes[{first_index}]"
            ),
            Self::InvalidFacets(error) => write!(f, "facets refused: {error}"),
        }
    }
}

impl std::error::Error for ResourceSnapshotError {}

fn parse_id(
    value: &str,
    column: &'static str,
    index: usize,
    role: &'static str,
) -> Result<Uuid, ResourceSnapshotError> {
    Uuid::parse_str(value).map_err(|_| ResourceSnapshotError::InvalidUuid {
        column,
        index,
        role,
        value: value.to_owned(),
    })
}

/// Everything a checked load verifies before any column is materialized.
fn validate_snapshot(snapshot: &GraphSnapshot) -> Result<(), ResourceSnapshotError> {
    snapshot
        .navigation
        .check_indices()
        .map_err(ResourceSnapshotError::InvalidNavigation)?;
    let surfaces = validate_surface_ids(snapshot)?;
    validate_resource_columns(snapshot, &surfaces)
}

/// The Surface ids of `snapshot.nodes`. A node whose id is unparseable or
/// repeats an earlier node's is refused, as a malformed facet is, so the
/// materializer never silently drops or merges one.
fn validate_surface_ids(snapshot: &GraphSnapshot) -> Result<BTreeSet<Uuid>, ResourceSnapshotError> {
    let mut first_indices = BTreeMap::new();
    for (index, node) in snapshot.nodes.iter().enumerate() {
        let surface_id = Uuid::parse_str(&node.node_id).map_err(|_| {
            ResourceSnapshotError::InvalidSurfaceId {
                index,
                value: node.node_id.clone(),
            }
        })?;
        if let Some(first_index) = first_indices.insert(surface_id, index) {
            return Err(ResourceSnapshotError::RepeatedSurfaceId {
                surface_id,
                first_index,
                index,
            });
        }
    }
    Ok(first_indices.into_keys().collect())
}

fn validate_resource_columns(
    snapshot: &GraphSnapshot,
    surfaces: &BTreeSet<Uuid>,
) -> Result<(), ResourceSnapshotError> {
    if snapshot.assertion_schema > 1 {
        return Err(ResourceSnapshotError::UnsupportedAssertionSchema {
            schema: snapshot.assertion_schema,
        });
    }
    for (column, edges) in [
        ("edges", snapshot.edges.as_slice()),
        ("resource_edges", snapshot.resource_edges.as_slice()),
    ] {
        for (edge_index, edge) in edges.iter().enumerate() {
            if edge.assertions.is_some() && edge.has_legacy_assertion_data() {
                return Err(ResourceSnapshotError::MixedAssertionFormat { column, edge_index });
            }
            if snapshot.assertion_schema == 1
                && edge.assertions.is_none()
                && edge.has_legacy_assertion_data()
            {
                return Err(ResourceSnapshotError::MissingCanonicalAssertions {
                    column,
                    edge_index,
                });
            }
        }
    }

    let mut resources: BTreeMap<Uuid, (usize, &str, BTreeMap<&str, serde_json::Value>)> =
        BTreeMap::new();
    for (index, record) in snapshot.resources.iter().enumerate() {
        let id = portolan::resource_id_from_canonical_iri(&record.canonical_iri);
        crate::graph::predicate_declarations::predicate_declarations_from_record(record).map_err(
            |error| ResourceSnapshotError::InvalidFacet {
                resource_id: id,
                facet: crate::graph::predicate_declarations::PREDICATE_DECLARATIONS_FACET.into(),
                detail: error.to_string(),
            },
        )?;
        crate::graph::resource_content::validate_content_record(record).map_err(|error| {
            ResourceSnapshotError::InvalidFacet {
                resource_id: id,
                facet: "semantic content".into(),
                detail: error.to_string(),
            }
        })?;
        let mut facets = BTreeMap::new();
        for facet in &record.facets {
            let value =
                serde_json::from_str::<serde_json::Value>(&facet.value_json).map_err(|error| {
                    ResourceSnapshotError::InvalidFacet {
                        resource_id: id,
                        facet: facet.facet.clone(),
                        detail: error.to_string(),
                    }
                })?;
            if facets.insert(facet.facet.as_str(), value).is_some() {
                return Err(ResourceSnapshotError::DuplicateFacet {
                    resource_id: id,
                    facet: facet.facet.clone(),
                });
            }
        }
        if let Some((first_index, first_iri, first_facets)) = resources.get(&id) {
            if *first_iri != record.canonical_iri.as_str() || first_facets != &facets {
                return Err(ResourceSnapshotError::ConflictingResource {
                    resource_id: id,
                    first_index: *first_index,
                    index,
                });
            }
        } else {
            resources.insert(id, (index, record.canonical_iri.as_str(), facets));
        }
    }

    let mut handles = AssertionHandles::new();
    let repeated = ResourceSnapshotError::RepeatedAssertionHandle;
    if snapshot.assertion_schema == 1 {
        for (index, edge) in snapshot.edges.iter().enumerate() {
            if edge.retained_assertions().is_empty() {
                continue;
            }
            let from = parse_id(&edge.from_node_id, "edges", index, "source")?;
            let to = parse_id(&edge.to_node_id, "edges", index, "target")?;
            for (role, value, id) in [
                ("source", &edge.from_node_id, from),
                ("target", &edge.to_node_id, to),
            ] {
                if value != &id.to_string() {
                    return Err(ResourceSnapshotError::NonCanonicalUuid {
                        column: "edges",
                        edge_index: index,
                        role,
                        value: value.clone(),
                    });
                }
                if !surfaces.contains(&id) {
                    return Err(ResourceSnapshotError::MissingSurface {
                        column: "edges",
                        index,
                        surface_id: id,
                    });
                }
            }
            handles
                .admit(GraphStratum::Surface, edge)
                .map_err(repeated)?;
        }
    } else {
        // Schema 0 preserves the permissive legacy Surface loader: malformed
        // or orphaned edge endpoints are skipped by materialization.
        for edge in snapshot.edges.iter().filter(|edge| {
            [&edge.from_node_id, &edge.to_node_id]
                .into_iter()
                .all(|id| {
                    Uuid::parse_str(id)
                        .ok()
                        .is_some_and(|id| surfaces.contains(&id))
                })
        }) {
            handles.hold_legacy_surface(edge);
        }
    }
    for (index, edge) in snapshot.resource_edges.iter().enumerate() {
        let from = parse_id(&edge.from_node_id, "resource_edges", index, "source")?;
        let to = parse_id(&edge.to_node_id, "resource_edges", index, "target")?;
        for (role, value, id) in [
            ("source", &edge.from_node_id, from),
            ("target", &edge.to_node_id, to),
        ] {
            if value != &id.to_string() {
                return Err(ResourceSnapshotError::NonCanonicalUuid {
                    column: "resource_edges",
                    edge_index: index,
                    role,
                    value: value.clone(),
                });
            }
        }
        for id in [from, to] {
            if !resources.contains_key(&id) {
                return Err(ResourceSnapshotError::MissingResource {
                    column: "resource_edges",
                    index,
                    resource_id: id,
                });
            }
        }
        if edge.assertions.is_none()
            && edge.semantic.as_ref().is_some_and(|semantic| {
                semantic.statements.is_empty()
                    && (!semantic.sub_kinds.is_empty() || semantic.predicate.is_some())
            })
        {
            return Err(ResourceSnapshotError::LegacySemanticAggregate { edge_index: index });
        }
        handles
            .admit(GraphStratum::Resource, edge)
            .map_err(repeated)?;
    }

    let surface_literals: BTreeSet<_> = snapshot
        .nodes
        .iter()
        .filter(|node| Uuid::parse_str(&node.node_id).is_ok())
        .flat_map(|node| {
            node.properties
                .iter()
                .map(|property| property.statement_id.as_str())
        })
        .collect();
    for (handle, row) in handles.rows() {
        if surface_literals.contains(handle) {
            return Err(ResourceSnapshotError::InvalidFacet {
                resource_id: Uuid::parse_str(&row.from_node_id)
                    .expect("admitted rows have validated endpoints"),
                facet: crate::graph::resource_content::RESOURCE_PROPERTIES.into(),
                detail: format!(
                    "resource assertion handle {handle:?} reuses an active surface literal"
                ),
            });
        }
    }
    let mut literal_owners = BTreeMap::new();
    for record in &snapshot.resources {
        let resource = portolan::resource_id_from_canonical_iri(&record.canonical_iri);
        for facet in &record.facets {
            if facet.facet != crate::graph::resource_content::RESOURCE_PROPERTIES {
                continue;
            }
            let properties: Vec<crate::types::NodeProperty> =
                serde_json::from_str(&facet.value_json).expect("typed content validated above");
            for property in properties {
                let error = || ResourceSnapshotError::InvalidFacet {
                    resource_id: resource,
                    facet: facet.facet.clone(),
                    detail: format!(
                        "literal assertion handle {:?} has conflicting ownership or payload",
                        property.statement_id
                    ),
                };
                if handles.holds(&property.statement_id)
                    || surface_literals.contains(property.statement_id.as_str())
                {
                    return Err(error());
                }
                if let Some((owner, held)) = literal_owners.get(&property.statement_id) {
                    if *owner != resource || held != &property {
                        return Err(error());
                    }
                } else {
                    literal_owners.insert(property.statement_id.clone(), (resource, property));
                }
            }
        }
    }

    let mut shown = BTreeMap::new();
    for (index, mapping) in snapshot.shown_resources.iter().enumerate() {
        let surface = parse_id(&mapping.surface_id, "shown_resources", index, "surface")?;
        let resource = parse_id(&mapping.resource_id, "shown_resources", index, "resource")?;
        if !surfaces.contains(&surface) {
            return Err(ResourceSnapshotError::MissingSurface {
                column: "shown_resources",
                index,
                surface_id: surface,
            });
        }
        if !resources.contains_key(&resource) {
            return Err(ResourceSnapshotError::MissingResource {
                column: "shown_resources",
                index,
                resource_id: resource,
            });
        }
        if let Some(first_resource) = shown.insert(surface, resource)
            && first_resource != resource
        {
            return Err(ResourceSnapshotError::ConflictingShownResource {
                surface_id: surface,
                first_resource,
                resource_id: resource,
            });
        }
    }
    Ok(())
}

impl Graph {
    /// Check all explicit resource columns and navigation indices before loading
    /// either graph stratum. Legacy surface conversion retains its existing
    /// compatibility behavior.
    pub fn try_from_snapshot(snapshot: &GraphSnapshot) -> Result<Self, ResourceSnapshotError> {
        validate_snapshot(snapshot)?;
        Ok(Self::from_snapshot_unchecked(snapshot))
    }

    /// Load caller-qualified recorded truth without deriving relations from current URLs.
    /// Explicit resource columns and navigation receive the same validation as
    /// legacy loads.
    pub fn try_from_recorded_snapshot(
        snapshot: &GraphSnapshot,
    ) -> Result<Self, ResourceSnapshotError> {
        validate_snapshot(snapshot)?;
        Ok(Self::from_recorded_snapshot_unchecked(snapshot))
    }
}

#[cfg(test)]
#[path = "checked/assertion_schema_tests.rs"]
mod assertion_schema_tests;
