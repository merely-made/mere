// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An edge's persisted form: the encoding a snapshot and a journal entry
//! share. Moved from the kernel's snapshot writer so the resource store
//! (engine split D243, D250) builds its deltas from the edges it holds.

use super::{EdgePayload, PersistedEdge, PersistedSemanticStatement, SemanticStatement};
use uuid::Uuid;

/// Encode a pair using explicit ids from either graph stratum.
pub fn persisted_edge_for_ids(from: Uuid, to: Uuid, payload: &EdgePayload) -> PersistedEdge {
    persisted_edge_with_ids(from.to_string(), to.to_string(), payload)
}

pub fn persisted_assertion(statement: &SemanticStatement) -> PersistedSemanticStatement {
    PersistedSemanticStatement {
        statement_id: statement.statement_id.clone(),
        predicate: statement.predicate.clone(),
        label: statement.label.clone(),
        basis: statement.basis.clone(),
        graph_scope: statement.graph_scope.clone(),
        provenance_iri: statement.provenance_iri.clone(),
        asserted_at_ms: statement.asserted_at_ms,
        first_asserted_ms: statement.first_asserted_ms,
        withdrawn: statement.withdrawn.clone(),
    }
}

/// The pair's records, less those whose predicate's durability axis says
/// they last only while the session does (declared-lens D27 (3)).
pub fn persisted_edge_with_ids(
    from_node_id: String,
    to_node_id: String,
    payload: &EdgePayload,
) -> PersistedEdge {
    let vocabulary = payload.vocabulary();
    let assertions = payload
        .assertions()
        .iter()
        .filter(|statement| !vocabulary.is_session_only(&statement.predicate))
        .map(persisted_assertion)
        .collect();
    PersistedEdge::of(from_node_id, to_node_id, assertions)
}
