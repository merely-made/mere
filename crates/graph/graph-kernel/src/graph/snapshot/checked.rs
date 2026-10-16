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
    ActiveSurfaceAssertion {
        edge_index: usize,
        statement_id: String,
    },
    ConflictingAssertion {
        statement_id: String,
        first_pair: (Uuid, Uuid),
        pair: (Uuid, Uuid),
        first_stratum: &'static str,
        stratum: &'static str,
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
                first_stratum,
                stratum,
            } => write!(
                f,
                "assertion {statement_id:?} conflicts between {first_stratum} pair {first_pair:?} and {stratum} pair {pair:?}"
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
        let id = chartulary::resource_id_from_canonical_iri(&record.canonical_iri);
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

    let surfaces = snapshot
        .nodes
        .iter()
        .filter_map(|node| Uuid::parse_str(&node.node_id).ok())
        .collect::<BTreeSet<_>>();
    let mut surface_assertions = BTreeSet::new();
    let mut assertions: BTreeMap<&str, ((Uuid, Uuid), &'static str, &PersistedSemanticStatement)> =
        BTreeMap::new();
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
            for statement in edge.retained_assertions() {
                surface_assertions.insert(statement.statement_id.as_str());
                if let Some((first_pair, first_stratum, previous)) =
                    assertions.get(statement.statement_id.as_str())
                {
                    if *first_pair != (from, to)
                        || *first_stratum != "surface"
                        || *previous != statement
                    {
                        return Err(ResourceSnapshotError::ConflictingAssertion {
                            statement_id: statement.statement_id.clone(),
                            first_pair: *first_pair,
                            pair: (from, to),
                            first_stratum: *first_stratum,
                            stratum: "surface",
                        });
                    }
                } else {
                    assertions.insert(&statement.statement_id, ((from, to), "surface", statement));
                }
            }
        }
    } else {
        // Schema 0 preserves the permissive legacy Surface loader: malformed
        // or orphaned edge endpoints are skipped by materialization.
        surface_assertions = snapshot
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
            .flat_map(|edge| edge.retained_assertions())
            .map(|statement| statement.statement_id.as_str())
            .collect();
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
        for statement in edge.retained_assertions() {
            if surface_assertions.contains(statement.statement_id.as_str()) {
                return Err(ResourceSnapshotError::ActiveSurfaceAssertion {
                    edge_index: index,
                    statement_id: statement.statement_id.clone(),
                });
            }
            if let Some((first_pair, first_stratum, previous)) =
                assertions.get(statement.statement_id.as_str())
            {
                if *first_pair != (from, to)
                    || *first_stratum != "resource"
                    || *previous != statement
                {
                    return Err(ResourceSnapshotError::ConflictingAssertion {
                        statement_id: statement.statement_id.clone(),
                        first_pair: *first_pair,
                        pair: (from, to),
                        first_stratum: *first_stratum,
                        stratum: "resource",
                    });
                }
            } else {
                assertions.insert(&statement.statement_id, ((from, to), "resource", statement));
            }
        }
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
    for (handle, (pair, _, _)) in &assertions {
        if surface_literals.contains(handle) {
            return Err(ResourceSnapshotError::InvalidFacet {
                resource_id: pair.0,
                facet: crate::graph::resource_content::RESOURCE_PROPERTIES.into(),
                detail: format!(
                    "resource assertion handle {handle:?} reuses an active surface literal"
                ),
            });
        }
    }
    let mut literal_owners = BTreeMap::new();
    for record in &snapshot.resources {
        let resource = chartulary::resource_id_from_canonical_iri(&record.canonical_iri);
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
                if surface_assertions.contains(property.statement_id.as_str())
                    || assertions.contains_key(property.statement_id.as_str())
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
    /// Check all explicit resource columns before loading either graph stratum.
    /// Legacy surface conversion retains its existing compatibility behavior.
    pub fn try_from_snapshot(snapshot: &GraphSnapshot) -> Result<Self, ResourceSnapshotError> {
        validate_resource_columns(snapshot)?;
        Ok(Self::from_snapshot_unchecked(snapshot))
    }

    /// Load caller-qualified recorded truth without deriving relations from current URLs.
    /// Explicit resource columns receive the same validation as legacy loads.
    pub fn try_from_recorded_snapshot(
        snapshot: &GraphSnapshot,
    ) -> Result<Self, ResourceSnapshotError> {
        validate_resource_columns(snapshot)?;
        Ok(Self::from_recorded_snapshot_unchecked(snapshot))
    }
}

#[cfg(test)]
mod assertion_schema_tests {
    use super::*;
    use crate::persistence::{PersistedEdge, PersistedSemanticEdgeData};
    use crate::types::{AssertionBasis, GraphScope};
    use euclid::default::Point2D;

    fn base_snapshot() -> GraphSnapshot {
        let mut graph = Graph::new();
        graph.add_node("https://one.test/".into(), Point2D::new(0.0, 0.0));
        graph.add_node("https://two.test/".into(), Point2D::new(1.0, 0.0));
        graph.to_snapshot()
    }

    fn statement(id: &str, predicate: &str) -> PersistedSemanticStatement {
        PersistedSemanticStatement {
            statement_id: id.into(),
            predicate: predicate.into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: GraphScope::User,
            provenance_iri: None,
            asserted_at_ms: None,
            first_asserted_ms: None,
            withdrawn: None,
            basis: AssertionBasis::default(),
        }
    }

    fn canonical_edge(
        from: &str,
        to: &str,
        assertions: Vec<PersistedSemanticStatement>,
    ) -> PersistedEdge {
        PersistedEdge {
            from_node_id: from.into(),
            to_node_id: to.into(),
            families: Vec::new(),
            semantic: None,
            traversal: None,
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
            assertions: Some(assertions),
        }
    }

    #[test]
    fn schema_gate_rejects_unknown_and_mixed_formats_but_accepts_explicit_empty() {
        let mut snapshot = base_snapshot();
        snapshot.assertion_schema = 9;
        assert!(matches!(
            Graph::try_from_snapshot(&snapshot),
            Err(ResourceSnapshotError::UnsupportedAssertionSchema { schema: 9 })
        ));

        let mut snapshot = base_snapshot();
        let from = snapshot.nodes[0].node_id.clone();
        let to = snapshot.nodes[1].node_id.clone();
        snapshot.assertion_schema = 1;
        let mut mixed = canonical_edge(&from, &to, vec![statement("x", "urn:p")]);
        mixed.semantic = Some(PersistedSemanticEdgeData {
            statements: Vec::new(),
            sub_kinds: vec![crate::persistence::PersistedSemanticSubKind::UserGrouped],
            predicate: None,
            label: None,
            agent_decay_progress: None,
        });
        snapshot.edges.push(mixed);
        assert!(matches!(
            Graph::try_from_snapshot(&snapshot),
            Err(ResourceSnapshotError::MixedAssertionFormat {
                column: "edges",
                edge_index: 0
            })
        ));

        let mut resource_mixed = base_snapshot();
        resource_mixed.assertion_schema = 1;
        let mut legacy_resource_edge = canonical_edge("bad-from", "bad-to", vec![]);
        legacy_resource_edge.assertions = None;
        legacy_resource_edge.semantic = Some(PersistedSemanticEdgeData {
            statements: Vec::new(),
            sub_kinds: vec![crate::persistence::PersistedSemanticSubKind::UserGrouped],
            predicate: None,
            label: None,
            agent_decay_progress: None,
        });
        resource_mixed.resource_edges.push(legacy_resource_edge);
        assert!(matches!(
            Graph::try_from_snapshot(&resource_mixed),
            Err(ResourceSnapshotError::MissingCanonicalAssertions {
                column: "resource_edges",
                edge_index: 0
            })
        ));

        let mut empty = base_snapshot();
        empty.assertion_schema = 1;
        let from = empty.nodes[0].node_id.clone();
        let to = empty.nodes[1].node_id.clone();
        let edge = canonical_edge(&from, &to, Vec::new());
        let encoded = serde_json::to_value(&edge).unwrap();
        assert_eq!(encoded["assertions"], serde_json::json!([]));
        empty.edges.push(edge);
        assert!(Graph::try_from_snapshot(&empty).is_ok());
    }

    #[test]
    fn canonical_schema_requires_assertions_and_valid_surface_endpoints() {
        let mut legacy = base_snapshot();
        legacy.assertion_schema = 1;
        let from = legacy.nodes[0].node_id.clone();
        let to = legacy.nodes[1].node_id.clone();
        let mut old = canonical_edge(&from, &to, Vec::new());
        old.assertions = None;
        old.semantic = Some(PersistedSemanticEdgeData {
            statements: Vec::new(),
            sub_kinds: vec![crate::persistence::PersistedSemanticSubKind::UserGrouped],
            predicate: None,
            label: None,
            agent_decay_progress: None,
        });
        legacy.edges.push(old);
        assert!(matches!(
            Graph::try_from_snapshot(&legacy),
            Err(ResourceSnapshotError::MissingCanonicalAssertions {
                column: "edges",
                edge_index: 0
            })
        ));

        let mut orphan = base_snapshot();
        orphan.assertion_schema = 1;
        orphan.edges.push(canonical_edge(
            &orphan.nodes[0].node_id,
            &Uuid::from_u128(99).to_string(),
            vec![statement("x", "urn:p")],
        ));
        assert!(matches!(
            Graph::try_from_snapshot(&orphan),
            Err(ResourceSnapshotError::MissingSurface { .. })
        ));

        // Schema zero intentionally keeps the historic permissive Surface load.
        let mut legacy_orphan = orphan;
        legacy_orphan.assertion_schema = 0;
        assert!(Graph::try_from_snapshot(&legacy_orphan).is_ok());
    }

    #[test]
    fn canonical_surface_assertion_ids_are_global_but_identical_owner_copies_are_allowed() {
        let mut snapshot = base_snapshot();
        snapshot.assertion_schema = 1;
        let from = snapshot.nodes[0].node_id.clone();
        let to = snapshot.nodes[1].node_id.clone();
        let same = statement("stable-id", "urn:p");
        snapshot
            .edges
            .push(canonical_edge(&from, &to, vec![same.clone()]));
        snapshot
            .edges
            .push(canonical_edge(&from, &to, vec![same.clone()]));
        assert!(Graph::try_from_snapshot(&snapshot).is_ok());

        let mut conflicting_pair = snapshot.clone();
        let third = conflicting_pair.nodes[1].node_id.clone();
        let first = conflicting_pair.nodes[0].node_id.clone();
        conflicting_pair
            .edges
            .push(canonical_edge(&third, &first, vec![same.clone()]));
        assert!(matches!(
            Graph::try_from_snapshot(&conflicting_pair),
            Err(ResourceSnapshotError::ConflictingAssertion { .. })
        ));

        let mut conflicting_claim = snapshot;
        conflicting_claim.edges[1].assertions.as_mut().unwrap()[0].predicate = "urn:other".into();
        assert!(matches!(
            Graph::try_from_snapshot(&conflicting_claim),
            Err(ResourceSnapshotError::ConflictingAssertion { .. })
        ));
    }
}
