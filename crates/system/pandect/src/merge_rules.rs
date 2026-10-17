// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The merge rules codicil Compose and session merge-in share (session
//! merge-in note, M8): same-id surface layering, edge signatures, resources
//! unified by canonical IRI, the checked per-facet resource merges, and the
//! assertion-handle check. A rule fixed here is fixed for both.
//!
//! Compose ([`crate::snapshot_merge`]) is a two-way union of frozen snapshots;
//! merge-in ([`crate::graph_session::MereSessions::merge_in`]) is a three-way
//! merge into a live session. Neither ever settles a conflict by insertion
//! order: a facet with no rule of its own merges only when both sides agree.

use std::collections::{HashMap, HashSet};

use kernel::graph::legacy_content_migration::{LEGACY_CONTENT_ORIGINS, LegacyContentOrigin};
use kernel::graph::predicate_declarations::{
    PREDICATE_DECLARATIONS_FACET, merge_predicate_declarations,
};
use kernel::graph::resource_classifications::{
    ClassificationVariant, merge_classification_variants,
};
use kernel::graph::resource_content::{RESOURCE_CLASSIFICATIONS, RESOURCE_PROPERTIES};
use kernel::graph::resource_properties::merge_resource_properties;
use kernel::persistence::{GraphSnapshot, PersistedEdge, PersistedNode, PersistedResourceRecord};
use kernel::types::NodeProperty;
use serde_json::Value;

/// What merging one facet's two values came to.
#[derive(Clone, Debug, PartialEq)]
pub enum FacetMerge {
    /// The facet's rule merged them, or they were equal.
    Merged(Value),
    /// The facet has no rule and the values differ: a conflict, never a
    /// survivor picked by order.
    Conflict,
}

/// Why a checked rule refused its inputs (an invalid value, or a rule's own
/// conflict such as one declaration handle with two payloads).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuleError(pub String);

impl std::fmt::Display for RuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RuleError {}

fn rule_error(error: impl std::fmt::Display) -> RuleError {
    RuleError(error.to_string())
}

/// Whether resource facet `facet` has a merge rule of its own, rather than
/// merging only when both sides agree.
pub fn resource_facet_has_rule(facet: &str) -> bool {
    facet == PREDICATE_DECLARATIONS_FACET
        || facet == RESOURCE_CLASSIFICATIONS
        || facet == RESOURCE_PROPERTIES
        || facet == LEGACY_CONTENT_ORIGINS
}

/// Merge two values of resource facet `facet` on the resource `canonical_iri`
/// names, by the facet's own checked rule: predicate declarations,
/// classifications and properties by their kernel merges, legacy content
/// origins as a sorted union. Any other facet merges only when both values are
/// equal.
pub fn merge_resource_facet(
    canonical_iri: &str,
    facet: &str,
    old: &Value,
    new: &Value,
) -> Result<FacetMerge, RuleError> {
    let merged = if facet == PREDICATE_DECLARATIONS_FACET {
        serde_json::to_value(
            merge_predicate_declarations(canonical_iri, old, new).map_err(rule_error)?,
        )
    } else if facet == RESOURCE_CLASSIFICATIONS {
        let old: Vec<ClassificationVariant> =
            serde_json::from_value(old.clone()).map_err(rule_error)?;
        let new: Vec<ClassificationVariant> =
            serde_json::from_value(new.clone()).map_err(rule_error)?;
        serde_json::to_value(merge_classification_variants(&old, &new).map_err(rule_error)?)
    } else if facet == RESOURCE_PROPERTIES {
        let old: Vec<NodeProperty> = serde_json::from_value(old.clone()).map_err(rule_error)?;
        let new: Vec<NodeProperty> = serde_json::from_value(new.clone()).map_err(rule_error)?;
        serde_json::to_value(merge_resource_properties(&old, &new).map_err(rule_error)?)
    } else if facet == LEGACY_CONTENT_ORIGINS {
        let mut notes: Vec<LegacyContentOrigin> =
            serde_json::from_value(old.clone()).map_err(rule_error)?;
        let incoming: Vec<LegacyContentOrigin> =
            serde_json::from_value(new.clone()).map_err(rule_error)?;
        for note in incoming {
            if !notes.contains(&note) {
                notes.push(note);
            }
        }
        notes.sort_by_cached_key(|note| serde_json::to_string(note).expect("origin encodes"));
        serde_json::to_value(&notes)
    } else if old != new {
        return Ok(FacetMerge::Conflict);
    } else {
        return Ok(FacetMerge::Merged(old.clone()));
    };
    merged.map(FacetMerge::Merged).map_err(rule_error)
}

/// Layer `incoming`'s facets onto `target`, two resource records with one
/// canonical IRI: a facet only one holds is kept, and a facet both hold merges
/// by [`merge_resource_facet`]. A conflict is an error naming the facet.
pub fn layer_resource_record(
    target: &mut PersistedResourceRecord,
    incoming: &PersistedResourceRecord,
) -> Result<(), RuleError> {
    for facet in &incoming.facets {
        let Some(existing) = target
            .facets
            .iter_mut()
            .find(|existing| existing.facet == facet.facet)
        else {
            target.facets.push(facet.clone());
            continue;
        };
        let old: Value = serde_json::from_str(&existing.value_json).map_err(rule_error)?;
        let new: Value = serde_json::from_str(&facet.value_json).map_err(rule_error)?;
        match merge_resource_facet(&target.canonical_iri, &facet.facet, &old, &new)? {
            // An equal value without a rule keeps its stored spelling.
            FacetMerge::Merged(_) if !resource_facet_has_rule(&facet.facet) => {},
            FacetMerge::Merged(value) => {
                existing.value_json = serde_json::to_string(&value).map_err(rule_error)?;
            },
            FacetMerge::Conflict => {
                return Err(RuleError(format!(
                    "resource {} has conflicting facet {:?}; precise variant merge is required",
                    chartulary::resource_id_from_canonical_iri(&target.canonical_iri),
                    facet.facet
                )));
            },
        }
    }
    Ok(())
}

/// Layer a same-id surface's context onto `target`: its tags, and the
/// properties whose predicate `target` lacks.
pub fn layer_surface(target: &mut PersistedNode, incoming: &PersistedNode) {
    for tag in &incoming.tags {
        if !target.tags.contains(tag) {
            target.tags.push(tag.clone());
        }
    }
    for property in &incoming.properties {
        if !target
            .properties
            .iter()
            .any(|held| held.predicate == property.predicate)
        {
            target.properties.push(property.clone());
        }
    }
}

/// An edge's whole signature, so no sidecar disappears when endpoints
/// coincide: identical edges collapse, and nothing else does.
pub fn edge_signature(edge: &PersistedEdge) -> (String, String, String) {
    (
        edge.from_node_id.clone(),
        edge.to_node_id.clone(),
        format!("{edge:?}"),
    )
}

/// Every assertion handle names one assertion: one domain, one pair of
/// endpoints, one payload.
pub fn check_assertion_handles(snapshot: &GraphSnapshot) -> Result<(), RuleError> {
    let surfaces: HashSet<_> = snapshot
        .nodes
        .iter()
        .filter_map(|node| uuid::Uuid::parse_str(&node.node_id).ok())
        .collect();
    let mut handles = HashMap::new();
    for (resource, edge) in snapshot
        .edges
        .iter()
        .filter(|edge| {
            [&edge.from_node_id, &edge.to_node_id]
                .into_iter()
                .all(|id| {
                    uuid::Uuid::parse_str(id)
                        .ok()
                        .is_some_and(|id| surfaces.contains(&id))
                })
        })
        .map(|edge| (false, edge))
        .chain(snapshot.resource_edges.iter().map(|edge| (true, edge)))
    {
        for statement in edge.retained_assertions() {
            let identity = (
                resource,
                uuid::Uuid::parse_str(&edge.from_node_id).map_err(|_| edge.from_node_id.as_str()),
                uuid::Uuid::parse_str(&edge.to_node_id).map_err(|_| edge.to_node_id.as_str()),
                statement,
            );
            if let Some(previous) = handles.insert(statement.statement_id.as_str(), identity)
                && previous != identity
            {
                return Err(RuleError(format!(
                    "assertion {:?} has conflicting domain, endpoints or payload during composition",
                    statement.statement_id
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;
    use kernel::graph::Graph;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::persistence::{PersistedEdgeFamily, PersistedSemanticStatement};
    use kernel::types::{AssertionBasis, GraphScope};

    #[test]
    fn a_facet_without_a_rule_merges_only_when_both_sides_agree() {
        let iri = "https://shared.test/page";
        let a = serde_json::json!({"status": "accepted"});
        let b = serde_json::json!({"status": "rejected"});
        assert_eq!(
            merge_resource_facet(iri, "test.review", &a, &b).unwrap(),
            FacetMerge::Conflict
        );
        assert_eq!(
            merge_resource_facet(iri, "test.review", &a, &a).unwrap(),
            FacetMerge::Merged(a.clone())
        );
        assert!(!resource_facet_has_rule("test.review"));
        assert!(resource_facet_has_rule(RESOURCE_PROPERTIES));
    }

    #[test]
    fn canonical_assertion_handle_collisions_are_checked_across_pairs() {
        let mut graph = Graph::new();
        graph.add_node("https://one.test/".into(), Point2D::new(0.0, 0.0));
        graph.add_node("https://two.test/".into(), Point2D::new(1.0, 0.0));
        graph.add_node("https://three.test/".into(), Point2D::new(2.0, 0.0));
        let mut snapshot = graph.to_snapshot();
        let statement = PersistedSemanticStatement {
            statement_id: "canonical-collision".into(),
            predicate: "urn:test:claim".into(),
            recognized_sub_kind: None,
            label: Some("same reason".into()),
            graph_scope: GraphScope::User,
            provenance_iri: Some("urn:test:author".into()),
            asserted_at_ms: Some(10),
            first_asserted_ms: Some(10),
            withdrawn: None,
            basis: AssertionBasis {
                note: Some("same reason".into()),
                ..Default::default()
            },
        };
        let edge = |from: &str, to: &str| PersistedEdge {
            from_node_id: from.into(),
            to_node_id: to.into(),
            families: vec![PersistedEdgeFamily::Semantic],
            semantic: None,
            traversal: None,
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
            assertions: Some(vec![statement.clone()]),
        };
        let from = snapshot.nodes[0].node_id.clone();
        let to = snapshot.nodes[1].node_id.clone();
        let other_to = snapshot.nodes[2].node_id.clone();
        let first = edge(&from, &to);
        let identical_copy = first.clone();
        snapshot.edges = vec![first, identical_copy];
        assert!(check_assertion_handles(&snapshot).is_ok());

        snapshot.edges.push(edge(&from, &other_to));
        assert!(check_assertion_handles(&snapshot).is_err());
    }
}
