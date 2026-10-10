// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Immutable selections owned by Resources, independent of local appearances.
use super::{CoverageNote, Graph};
use crate::persistence::PersistedEdge;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const FROZEN_SELECTION: &str = "graph.frozen-selection/v1";
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenSelection {
    version: u32,
    spec: serde_json::Value,
    revision: u64,
    members: Vec<Uuid>,
    statements: Vec<PersistedEdge>,
}
impl FrozenSelection {
    pub fn spec(&self) -> &serde_json::Value {
        &self.spec
    }
    pub fn revision(&self) -> u64 {
        self.revision
    }
    pub fn members(&self) -> &[Uuid] {
        &self.members
    }
    pub fn statements(&self) -> &[PersistedEdge] {
        &self.statements
    }
}
pub struct OpenedFrozenSelection<'a> {
    pub selection: &'a FrozenSelection,
    pub coverage: CoverageNote,
}
// Legacy edge DTOs deliberately remain permissive. This v1 carrier checks
// every nested key against what their typed serialization retains instead.
fn validate_retained_keys(
    input: &serde_json::Value,
    known: &serde_json::Value,
) -> Result<(), String> {
    match (input, known) {
        (serde_json::Value::Object(input), serde_json::Value::Object(known)) => {
            for (key, value) in input {
                let expected = known
                    .get(key)
                    .ok_or_else(|| format!("unknown frozen payload field {key}"))?;
                validate_retained_keys(value, expected)?;
            }
        },
        (serde_json::Value::Array(input), serde_json::Value::Array(known)) => {
            for (value, expected) in input.iter().zip(known) {
                validate_retained_keys(value, expected)?;
            }
        },
        (serde_json::Value::Object(_), _) | (serde_json::Value::Array(_), _) => {
            return Err("unrepresentable frozen payload shape".into());
        },
        _ => {},
    }
    Ok(())
}

/// Decode and check nested references without requiring member residency.
pub(crate) fn selection_from_record(
    record: &crate::persistence::PersistedResourceRecord,
) -> Result<Option<FrozenSelection>, String> {
    let Some(facet) = record
        .facets
        .iter()
        .find(|facet| facet.facet == FROZEN_SELECTION)
    else {
        return Ok(None);
    };
    let raw: serde_json::Value = serde_json::from_str(&facet.value_json)
        .map_err(|e| format!("invalid frozen selection: {e}"))?;
    let selection: FrozenSelection = serde_json::from_value(raw.clone())
        .map_err(|e| format!("invalid frozen selection: {e}"))?;
    let mut known = serde_json::to_value(&selection).map_err(|e| e.to_string())?;
    // Human-readable statement serialization omits default metadata. Their
    // known fields are still accepted explicitly, including null/default values.
    for edge in known["statements"]
        .as_array_mut()
        .expect("serialized edge array")
    {
        if let Some(statements) = edge
            .get_mut("semantic")
            .and_then(|s| s.get_mut("statements"))
            .and_then(|s| s.as_array_mut())
        {
            for statement in statements {
                let object = statement
                    .as_object_mut()
                    .expect("serialized statement object");
                for field in [
                    "recognized_sub_kind",
                    "label",
                    "graph_scope",
                    "provenance_iri",
                    "asserted_at_ms",
                ] {
                    object.entry(field).or_insert(serde_json::Value::Null);
                }
            }
        }
    }
    validate_retained_keys(&raw, &known)?;
    if selection.version != 1 {
        return Err("unsupported frozen selection version".into());
    }
    let members: std::collections::BTreeSet<_> = selection.members.iter().copied().collect();
    if members.len() != selection.members.len() || members.contains(&Uuid::nil()) {
        return Err("invalid frozen member references".into());
    }
    let mut handles = std::collections::BTreeMap::new();
    for edge in &selection.statements {
        let from = edge
            .from_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen source id")?;
        let to = edge
            .to_node_id
            .parse::<Uuid>()
            .map_err(|_| "invalid frozen target id")?;
        if from.to_string() != edge.from_node_id
            || to.to_string() != edge.to_node_id
            || !members.contains(&from)
            || !members.contains(&to)
        {
            return Err("frozen statement endpoints must be member references".into());
        }
        if let Some(semantic) = &edge.semantic {
            if semantic.statements.is_empty()
                && (!semantic.sub_kinds.is_empty() || semantic.predicate.is_some())
            {
                return Err("frozen semantic statements require exact handles".into());
            }
            for statement in &semantic.statements {
                if let Some(previous) =
                    handles.insert(&statement.statement_id, (from, to, statement))
                {
                    if previous != (from, to, statement) {
                        return Err("conflicting frozen statement handle".into());
                    }
                }
            }
        }
        // Reject malformed family/payload combinations instead of normalizing away data.
        let payload = super::snapshot::payload_from_persisted(edge);
        if super::snapshot::persisted_edge_for_ids(from, to, &payload) != *edge {
            return Err("inconsistent frozen relation payload".into());
        }
    }
    Ok(Some(selection))
}

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
        let selection = FrozenSelection {
            version: 1,
            spec,
            revision: self.revision(),
            members,
            statements,
        };
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
