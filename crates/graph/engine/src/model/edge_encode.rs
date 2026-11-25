// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An edge's persisted form: the encoding a snapshot and a journal entry
//! share. Moved from the kernel's snapshot writer so the resource store
//! (engine split D243, D250) builds its deltas from the edges it holds.

use super::{
    EdgeFamily, EdgePayload, NavigationTrigger, PersistedEdge, PersistedEdgeFamily,
    PersistedNavigationTrigger, PersistedSemanticStatement, PersistedSemanticSubKind,
    PersistedTraversalEdgeData, PersistedTraversalMetrics, PersistedTraversalRecord,
    RelationDurability, RelationKind, SemanticStatement, SemanticSubKind,
};
use uuid::Uuid;

pub fn persisted_semantic_sub_kind(sub_kind: SemanticSubKind) -> PersistedSemanticSubKind {
    match sub_kind {
        SemanticSubKind::Hyperlink => PersistedSemanticSubKind::Hyperlink,
        SemanticSubKind::UserGrouped => PersistedSemanticSubKind::UserGrouped,
        SemanticSubKind::AgentDerived => PersistedSemanticSubKind::AgentDerived,
        SemanticSubKind::Cites => PersistedSemanticSubKind::Cites,
        SemanticSubKind::Quotes => PersistedSemanticSubKind::Quotes,
        SemanticSubKind::Summarizes => PersistedSemanticSubKind::Summarizes,
        SemanticSubKind::Elaborates => PersistedSemanticSubKind::Elaborates,
        SemanticSubKind::ExampleOf => PersistedSemanticSubKind::ExampleOf,
        SemanticSubKind::Supports => PersistedSemanticSubKind::Supports,
        SemanticSubKind::Contradicts => PersistedSemanticSubKind::Contradicts,
        SemanticSubKind::Questions => PersistedSemanticSubKind::Questions,
        SemanticSubKind::SameEntityAs => PersistedSemanticSubKind::SameEntityAs,
        SemanticSubKind::DuplicateOf => PersistedSemanticSubKind::DuplicateOf,
        SemanticSubKind::CanonicalMirrorOf => PersistedSemanticSubKind::CanonicalMirrorOf,
        SemanticSubKind::DependsOn => PersistedSemanticSubKind::DependsOn,
        SemanticSubKind::Blocks => PersistedSemanticSubKind::Blocks,
        SemanticSubKind::NextStep => PersistedSemanticSubKind::NextStep,
    }
}

/// Encode a pair using explicit ids from either graph stratum.
pub fn persisted_edge_for_ids(from: Uuid, to: Uuid, payload: &EdgePayload) -> PersistedEdge {
    persisted_edge_with_ids(from.to_string(), to.to_string(), payload)
}

pub fn persisted_assertion(statement: &SemanticStatement) -> PersistedSemanticStatement {
    PersistedSemanticStatement {
        statement_id: statement.statement_id.clone(),
        predicate: statement.predicate.clone(),
        recognized_sub_kind: statement
            .recognized_sub_kind
            .map(persisted_semantic_sub_kind),
        label: statement.label.clone(),
        basis: statement.basis.clone(),
        graph_scope: statement.graph_scope.clone(),
        provenance_iri: statement.provenance_iri.clone(),
        asserted_at_ms: statement.asserted_at_ms,
        first_asserted_ms: statement.first_asserted_ms,
        withdrawn: statement.withdrawn.clone(),
    }
}

pub fn persisted_edge_with_ids(
    from_node_id: String,
    to_node_id: String,
    payload: &EdgePayload,
) -> PersistedEdge {
    let assertions: Vec<_> = payload
        .assertions()
        .iter()
        .filter(|statement| {
            !matches!(statement.relation_kind(), RelationKind::Arrangement(kind)
            if kind.durability() != RelationDurability::Durable)
        })
        .map(persisted_assertion)
        .collect();
    PersistedEdge {
        from_node_id,
        to_node_id,
        families: payload
            .families()
            .into_iter()
            .map(|family| match family {
                EdgeFamily::Semantic => PersistedEdgeFamily::Semantic,
                EdgeFamily::Traversal => PersistedEdgeFamily::Traversal,
                EdgeFamily::Containment => PersistedEdgeFamily::Containment,
                EdgeFamily::Arrangement => PersistedEdgeFamily::Arrangement,
                EdgeFamily::Imported => PersistedEdgeFamily::Imported,
                EdgeFamily::Provenance => PersistedEdgeFamily::Provenance,
            })
            .collect(),
        semantic: None,
        containment: None,
        arrangement: None,
        imported: None,
        provenance: None,
        assertions: Some(assertions),
        traversal: payload
            .traversal_data()
            .map(|data| PersistedTraversalEdgeData {
                traversals: data
                    .traversals
                    .iter()
                    .map(|traversal| PersistedTraversalRecord {
                        id: traversal.id,
                        timestamp_ms: traversal.timestamp_ms,
                        recorded_by: traversal.recorded_by.clone(),
                        browsing_session: traversal.browsing_session,
                        trigger: match traversal.trigger {
                            NavigationTrigger::Unknown => PersistedNavigationTrigger::Unknown,
                            NavigationTrigger::LinkClick => PersistedNavigationTrigger::LinkClick,
                            NavigationTrigger::Back => PersistedNavigationTrigger::Back,
                            NavigationTrigger::Forward => PersistedNavigationTrigger::Forward,
                            NavigationTrigger::AddressBarEntry => {
                                PersistedNavigationTrigger::AddressBarEntry
                            },
                            NavigationTrigger::PanePromotion => {
                                PersistedNavigationTrigger::PanePromotion
                            },
                            NavigationTrigger::Programmatic => {
                                PersistedNavigationTrigger::Programmatic
                            },
                            NavigationTrigger::Redirect => PersistedNavigationTrigger::Redirect,
                            NavigationTrigger::ReopenSession => {
                                PersistedNavigationTrigger::ReopenSession
                            },
                            NavigationTrigger::JumpAnchor => PersistedNavigationTrigger::JumpAnchor,
                            NavigationTrigger::InPageSearchJump => {
                                PersistedNavigationTrigger::InPageSearchJump
                            },
                            NavigationTrigger::ImportedHistory => {
                                PersistedNavigationTrigger::ImportedHistory
                            },
                        },
                    })
                    .collect(),
                metrics: PersistedTraversalMetrics {
                    total_navigations: data.metrics.total_navigations,
                    forward_navigations: data.metrics.forward_navigations,
                    backward_navigations: data.metrics.backward_navigations,
                    last_navigated_at: data.metrics.last_navigated_at,
                },
            }),
    }
}
