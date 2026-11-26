// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Immutable selections owned by Resources, independent of local appearances.
// Legacy edge DTOs deliberately remain permissive. This v1 carrier checks
// every nested key against what their typed serialization retains instead.

use super::Graph;
pub use mere::model::frozen::{FROZEN_SELECTION, FrozenSelection};
pub use mere::model::frozen_rules::OpenedFrozenSelection;
pub(crate) use mere::model::frozen_rules::selection_from_record;
use uuid::Uuid;

impl Graph {
    /// Mint a Resource that bears a revision-frozen nested graph of references and
    /// exact relation copies. Member payloads stay in their owning resources.
    /// The spec is opaque to the kernel; the facade supplies its portable value.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn freeze_resource_selection(
        &mut self,
        spec: serde_json::Value,
        members: Vec<Uuid>,
    ) -> Result<Uuid, String> {
        self.freeze_resource_selection_with_nonce(spec, members, Uuid::new_v4())
    }

    /// Portable freeze entry point: the host supplies a fresh UUID nonce, as it
    /// does for new Surfaces on wasm. A reused address is refused before writing.
    pub fn freeze_resource_selection_with_nonce(
        &mut self,
        spec: serde_json::Value,
        mut members: Vec<Uuid>,
        nonce: Uuid,
    ) -> Result<Uuid, String> {
        members.sort();
        members.dedup();
        if members.iter().any(|id| self.resource(*id).is_none()) {
            return Err("missing frozen member resource".into());
        }
        let set: std::collections::BTreeSet<_> = members.iter().copied().collect();
        let statements = self
            .resource_edges()
            .filter(|(from, to, _)| set.contains(&from.id()) && set.contains(&to.id()))
            .map(|(from, to, payload)| {
                super::snapshot::persisted_edge_for_ids(from.id(), to.id(), payload)
            })
            .collect();
        let selection = FrozenSelection::v1(spec, self.revision(), members, statements);
        // A fresh address gives this record its own deterministic Resource identity.
        if nonce.is_nil() {
            return Err("frozen Resource nonce must be fresh".into());
        }
        let iri = format!("urn:uuid:{nonce}");
        let resource = super::ResourceNode::for_term(&iri);
        let record = crate::persistence::PersistedResourceRecord {
            canonical_iri: iri,
            facets: vec![crate::persistence::PersistedResourceFacet {
                facet: FROZEN_SELECTION.into(),
                value_json: serde_json::to_string(&selection).map_err(|e| e.to_string())?,
            }],
        };
        selection_from_record(&record)?;
        if self.resource(resource.id()).is_some()
            || !self.set_resource_record(resource.id(), Some(record.clone()))
        {
            return Err("frozen Resource admission refused".into());
        }
        self.record_delta(&super::CapturedDelta::ReplaySetResourceRecordById {
            resource_id: resource.id().to_string(),
            record: Some(record),
        });
        Ok(resource.id())
    }

    /// Open the immutable graph without loading or inventing its referenced members.
    pub fn open_frozen_selection(&self, owner: Uuid) -> Result<OpenedFrozenSelection<'_>, String> {
        let selection = self
            .resource(owner)
            .and_then(|resource| resource.nested_selection())
            .ok_or("resource bears no frozen selection")?;
        Ok(OpenedFrozenSelection {
            selection,
            coverage: self.coverage_for_resource_refs(selection.members()),
        })
    }
}

#[cfg(test)]
mod canonical_record_tests {
    use super::*;
    use crate::persistence::{
        PersistedEdge, PersistedResourceFacet, PersistedResourceRecord, PersistedSemanticStatement,
    };
    use crate::types::{AssertionBasis, GraphScope};

    fn assertion_edge(from: Uuid, to: Uuid, id: &str) -> PersistedEdge {
        PersistedEdge {
            from_node_id: from.to_string(),
            to_node_id: to.to_string(),
            families: vec![crate::persistence::PersistedEdgeFamily::Semantic],
            semantic: None,
            traversal: None,
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
            assertions: Some(vec![PersistedSemanticStatement {
                statement_id: id.into(),
                predicate: "urn:test:claim".into(),
                recognized_sub_kind: None,
                label: Some("claim".into()),
                graph_scope: GraphScope::User,
                provenance_iri: Some("urn:test:author".into()),
                asserted_at_ms: Some(12),
                first_asserted_ms: Some(10),
                withdrawn: None,
                basis: AssertionBasis {
                    note: Some("basis note".into()),
                    ..Default::default()
                },
            }]),
        }
    }

    fn record(statements: Vec<PersistedEdge>) -> PersistedResourceRecord {
        let members = (1..=3).map(Uuid::from_u128).collect();
        let selection = FrozenSelection::v1(serde_json::json!({}), 1, members, statements);
        PersistedResourceRecord {
            canonical_iri: "urn:uuid:selection".into(),
            facets: vec![PersistedResourceFacet {
                facet: FROZEN_SELECTION.into(),
                value_json: serde_json::to_string(&selection).unwrap(),
            }],
        }
    }

    #[test]
    fn frozen_selection_rejects_conflicting_canonical_ids_across_pairs() {
        let (first, second, third) = (Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3));
        let error = selection_from_record(&record(vec![
            assertion_edge(first, second, "same-id"),
            assertion_edge(first, third, "same-id"),
        ]))
        .unwrap_err();
        assert!(error.contains("one handle means one row"), "{error}");
    }

    #[test]
    fn frozen_selection_rejects_identical_copies_on_parallel_rows() {
        let row = assertion_edge(Uuid::from_u128(1), Uuid::from_u128(2), "copied");
        assert!(selection_from_record(&record(vec![row.clone()])).is_ok());
        let error = selection_from_record(&record(vec![row.clone(), row])).unwrap_err();
        assert!(error.contains("one handle means one row"), "{error}");
    }
}
