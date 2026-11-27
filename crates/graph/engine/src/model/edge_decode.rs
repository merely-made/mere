// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An edge's runtime payload from its persisted form, the inverse of
//! [`edge_encode`](super::edge_encode). Moved from the kernel's snapshot
//! reader so the resource store (engine split D243) checks records itself.
//! A legacy sidecar's names read as IRIs through [`legacy`] (D17).

use super::relation::{Vocabulary, legacy};
use super::{
    AssertionMethod, EdgePayload, GraphScope, GraphStratum, PersistedEdge, SemanticStatement,
    SemanticStatementSpec, edge_data,
};
use crate::profile::core::{self, ids};
use uuid::Uuid;

/// Before canonical assertion records existed, Surface UrlPath and Domain
/// sidecars were compatibility projections of current URLs, not authored
/// claims. Mark just those legacy Surface rows so the ordinary session loader
/// can replace them from current source data without treating canonical or
/// unknown Resource assertions as rule output.
pub fn mark_legacy_containment_sidecars_as_derived(
    payload: &mut EdgePayload,
    edge: &PersistedEdge,
) {
    let derived = [
        core::iri(ids::REL_CONTAINMENT_URL_PATH),
        core::iri(ids::REL_CONTAINMENT_DOMAIN),
    ];
    let Some(containment) = &edge.containment else {
        return;
    };
    let has_derived_sidecar = containment
        .sub_kinds
        .iter()
        .any(|name| derived.contains(&legacy::predicate_iri("Containment", name).as_str()));
    if !has_derived_sidecar {
        return;
    }
    for statement in &mut payload.assertions.statements {
        if statement.provenance_iri.as_deref() != Some(edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
            || statement.basis.method.is_some()
            || !derived.contains(&statement.predicate.as_str())
        {
            continue;
        }
        statement.basis.method = Some(AssertionMethod {
            iri: "urn:mere:method:derived-containment".into(),
            version: Some("1".into()),
        });
    }
    payload.assertions.rebuild_compat();
}

/// Decode a standalone pair without changing carried assertion metadata.
pub fn payload_from_persisted(pedge: &PersistedEdge) -> EdgePayload {
    payload_from_persisted_at(pedge, GraphStratum::Surface, 0)
}

pub fn payload_from_persisted_at(
    pedge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
) -> EdgePayload {
    payload_from_persisted_in(pedge, stratum, ordinal, Vocabulary::core())
}

/// [`payload_from_persisted_at`] into a payload that classifies by `vocabulary`.
pub fn payload_from_persisted_in(
    pedge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
    vocabulary: Vocabulary,
) -> EdgePayload {
    let mut payload = EdgePayload::with_vocabulary(vocabulary);
    restore_payload_at(&mut payload, pedge, stratum, ordinal);
    payload
}

/// Merge persisted sidecars; report structural insertions for surface revisions.
pub fn restore_persisted_payload(payload: &mut EdgePayload, pedge: &PersistedEdge) -> usize {
    restore_payload_at(payload, pedge, GraphStratum::Surface, 0)
}

pub fn restore_payload_at(
    payload: &mut EdgePayload,
    pedge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
) -> usize {
    let mut structural_changes = 0;
    if let Some(records) = &pedge.assertions {
        let mut ids: std::collections::BTreeSet<_> = payload
            .assertions()
            .iter()
            .map(|s| s.statement_id.clone())
            .collect();
        for record in records {
            if ids.insert(record.statement_id.clone()) {
                let mut record = runtime_assertion(record);
                record.normalize_basis_compat();
                payload.assertions.statements.push(record);
                structural_changes += 1;
            }
        }
    }
    let preserved_ids: std::collections::BTreeSet<_> = payload
        .assertions()
        .iter()
        .map(|s| s.statement_id.clone())
        .chain(
            pedge
                .semantic
                .as_ref()
                .into_iter()
                .flat_map(|data| &data.statements)
                .map(|s| s.statement_id.clone()),
        )
        .collect();
    // Canonical records are authoritative. Checked load refuses a nonempty
    // legacy sidecar alongside them instead of inventing a merge precedence.
    if pedge.assertions.is_none() {
        structural_changes += restore_legacy_sidecars(payload, pedge);
    }
    for statement in &mut payload.assertions.statements {
        if !preserved_ids.contains(&statement.statement_id) {
            statement.statement_id =
                legacy_assertion_id(pedge, stratum, ordinal, &statement.predicate);
            statement.normalize_legacy_asserter();
        }
    }
    payload.assertions.rebuild_compat();
    structural_changes
}

/// The sidecars engines wrote before assertion records, by name.
fn restore_legacy_sidecars(payload: &mut EdgePayload, pedge: &PersistedEdge) -> usize {
    let mut structural_changes = 0;
    if let Some(semantic) = &pedge.semantic {
        if !semantic.statements.is_empty() {
            for statement in &semantic.statements {
                let _ = payload.push_persisted_semantic_statement(runtime_assertion(statement));
            }
        } else {
            for name in &semantic.sub_kinds {
                payload.assert_semantic_statement(SemanticStatementSpec {
                    predicate: legacy::predicate_iri("Semantic", name),
                    label: semantic.label.clone(),
                    graph_scope: GraphScope::Default,
                    provenance_iri: Some(edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into()),
                    asserted_at_ms: None,
                    basis: Default::default(),
                });
            }
            if semantic.predicate.is_some() {
                payload.set_semantic_predicate(semantic.predicate.clone());
            }
        }
    }
    // Every stored sub-kind is restored: `UrlPath` and `Domain` are re-derived
    // by the session loader, but the authored ones exist only here.
    for (family, names) in pedge.legacy_sub_kinds() {
        for name in names {
            let predicate = legacy::predicate_iri(family, name);
            structural_changes += usize::from(payload.assert_relation(&predicate));
        }
    }
    structural_changes
}

/// Stable one-time legacy identity: UTF-8 JSON array of version/domain,
/// stratum, original directed endpoint strings, bucket ordinal and predicate;
/// RFC 9562 UUIDv5 in the URL namespace. No clock or platform hash enters it.
pub fn legacy_assertion_id(
    edge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
    predicate: &str,
) -> String {
    let bytes = serde_json::to_vec(&(
        "mere.legacy-relation-assertion",
        1u8,
        match stratum {
            GraphStratum::Surface => "surface",
            GraphStratum::Resource => "resource",
        },
        &edge.from_node_id,
        &edge.to_node_id,
        ordinal,
        predicate,
    ))
    .expect("string tuple JSON");
    format!(
        "urn:mere:legacy-assertion:v1:{}",
        Uuid::new_v5(&Uuid::NAMESPACE_URL, &bytes)
    )
}

pub fn runtime_assertion(statement: &super::PersistedSemanticStatement) -> SemanticStatement {
    SemanticStatement {
        statement_id: statement.statement_id.clone(),
        predicate: statement.predicate.clone(),
        label: statement.label.clone(),
        basis: statement.basis.clone(),
        graph_scope: statement.graph_scope.clone(),
        provenance_iri: statement.provenance_iri.clone(),
        first_asserted_ms: statement.first_asserted_ms,
        asserted_at_ms: statement.asserted_at_ms,
        withdrawn: statement.withdrawn.clone(),
    }
}
