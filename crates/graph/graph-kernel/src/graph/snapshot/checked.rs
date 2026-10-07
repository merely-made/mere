// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Validate explicit resource columns before materializing a graph.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use uuid::Uuid;

use crate::graph::Graph;
use crate::persistence::{GraphSnapshot, PersistedSemanticStatement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceSnapshotError {
    InvalidUuid {
        column: &'static str,
        index: usize,
        role: &'static str,
        value: String,
    },
    NonCanonicalUuid {
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
        index: usize,
        surface_id: Uuid,
    },
    LegacySemanticAggregate {
        edge_index: usize,
    },
    ActiveSurfaceAssertion {
        edge_index: usize,
        statement_id: String,
    },
    ConflictingAssertion {
        statement_id: String,
        first_pair: (Uuid, Uuid),
        pair: (Uuid, Uuid),
    },
    ConflictingShownResource {
        surface_id: Uuid,
        first_resource: Uuid,
        resource_id: Uuid,
    },
}

impl fmt::Display for ResourceSnapshotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUuid {
                column,
                index,
                role,
                value,
            } => write!(f, "{column}[{index}] has invalid {role} UUID {value:?}"),
            Self::NonCanonicalUuid {
                edge_index,
                role,
                value,
            } => write!(
                f,
                "resource_edges[{edge_index}] has noncanonical {role} UUID {value:?}"
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
            Self::MissingSurface { index, surface_id } => write!(
                f,
                "shown_resources[{index}] references absent surface {surface_id}"
            ),
            Self::LegacySemanticAggregate { edge_index } => write!(
                f,
                "resource_edges[{edge_index}] has semantic data without exact assertion handles"
            ),
            Self::ActiveSurfaceAssertion {
                edge_index,
                statement_id,
            } => write!(
                f,
                "resource_edges[{edge_index}] reuses active surface assertion {statement_id:?}"
            ),
            Self::ConflictingAssertion {
                statement_id,
                first_pair,
                pair,
            } => write!(
                f,
                "assertion {statement_id:?} conflicts between resource pairs {first_pair:?} and {pair:?}"
            ),
            Self::ConflictingShownResource {
                surface_id,
                first_resource,
                resource_id,
            } => write!(
                f,
                "surface {surface_id} shows conflicting resources {first_resource} and {resource_id}"
            ),
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

fn validate_resource_columns(snapshot: &GraphSnapshot) -> Result<(), ResourceSnapshotError> {
    let mut resources: BTreeMap<Uuid, (usize, &str, BTreeMap<&str, serde_json::Value>)> =
        BTreeMap::new();
    for (index, record) in snapshot.resources.iter().enumerate() {
        let id = chartulary::resource_id_from_canonical_iri(&record.canonical_iri);
        crate::graph::predicate_declarations::predicate_declarations_from_record(record).map_err(
            |error| ResourceSnapshotError::InvalidFacet {
                resource_id: id,
                facet: crate::graph::predicate_declarations::PREDICATE_DECLARATIONS_FACET.into(),
                detail: error.to_string(),
            },
        )?;
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

    let surfaces = snapshot
        .nodes
        .iter()
        .filter_map(|node| Uuid::parse_str(&node.node_id).ok())
        .collect::<BTreeSet<_>>();
    let surface_assertions = snapshot
        .edges
        .iter()
        .filter(|edge| {
            [&edge.from_node_id, &edge.to_node_id]
                .into_iter()
                .all(|id| {
                    Uuid::parse_str(id)
                        .ok()
                        .is_some_and(|id| surfaces.contains(&id))
                })
        })
        .filter_map(|edge| edge.semantic.as_ref())
        .flat_map(|semantic| &semantic.statements)
        .map(|statement| statement.statement_id.as_str())
        .collect::<BTreeSet<_>>();
    let mut assertions: BTreeMap<&str, ((Uuid, Uuid), &PersistedSemanticStatement)> =
        BTreeMap::new();
    for (index, edge) in snapshot.resource_edges.iter().enumerate() {
        let from = parse_id(&edge.from_node_id, "resource_edges", index, "source")?;
        let to = parse_id(&edge.to_node_id, "resource_edges", index, "target")?;
        for (role, value, id) in [
            ("source", &edge.from_node_id, from),
            ("target", &edge.to_node_id, to),
        ] {
            if value != &id.to_string() {
                return Err(ResourceSnapshotError::NonCanonicalUuid {
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
        let Some(semantic) = &edge.semantic else {
            continue;
        };
        if semantic.statements.is_empty()
            && (!semantic.sub_kinds.is_empty() || semantic.predicate.is_some())
        {
            return Err(ResourceSnapshotError::LegacySemanticAggregate { edge_index: index });
        }
        for statement in &semantic.statements {
            if surface_assertions.contains(statement.statement_id.as_str()) {
                return Err(ResourceSnapshotError::ActiveSurfaceAssertion {
                    edge_index: index,
                    statement_id: statement.statement_id.clone(),
                });
            }
            if let Some((first_pair, previous)) = assertions.get(statement.statement_id.as_str()) {
                if *first_pair != (from, to) || *previous != statement {
                    return Err(ResourceSnapshotError::ConflictingAssertion {
                        statement_id: statement.statement_id.clone(),
                        first_pair: *first_pair,
                        pair: (from, to),
                    });
                }
            } else {
                assertions.insert(&statement.statement_id, ((from, to), statement));
            }
        }
    }

    let mut shown = BTreeMap::new();
    for (index, mapping) in snapshot.shown_resources.iter().enumerate() {
        let surface = parse_id(&mapping.surface_id, "shown_resources", index, "surface")?;
        let resource = parse_id(&mapping.resource_id, "shown_resources", index, "resource")?;
        if !surfaces.contains(&surface) {
            return Err(ResourceSnapshotError::MissingSurface {
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
    /// Check all explicit resource columns before loading either graph stratum.
    /// Legacy surface conversion retains its existing compatibility behavior.
    pub fn try_from_snapshot(snapshot: &GraphSnapshot) -> Result<Self, ResourceSnapshotError> {
        validate_resource_columns(snapshot)?;
        Ok(Self::from_snapshot_unchecked(snapshot))
    }
}
