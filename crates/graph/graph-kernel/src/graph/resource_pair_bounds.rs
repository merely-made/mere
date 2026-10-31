// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded live capture of a resource pair for exact replacement.

use std::collections::BTreeSet;
use std::io::{self, Write};

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Serialize, Serializer};
use uuid::Uuid;

use super::{EdgeFamily, EdgePayload, Graph, SemanticStatement};
use crate::graph::edge_data::TraversalData;
use crate::persistence::PersistedEdge;
use crate::types::GraphScope;

/// A bounded pair capture refused without returning any partial rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceEdgesBoundError {
    MissingEndpoint,
    EdgeLimitExceeded,
    ByteLimitExceeded,
    Serialization,
}

impl std::fmt::Display for ResourceEdgesBoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingEndpoint => "resource pair endpoint missing",
            Self::EdgeLimitExceeded => "resource pair edge limit exceeded",
            Self::ByteLimitExceeded => "resource pair byte limit exceeded",
            Self::Serialization => "resource pair serialization failed",
        };
        f.write_str(message)
    }
}

impl std::error::Error for ResourceEdgesBoundError {}

impl Graph {
    /// Capture all live rows for a directed resource pair, including session
    /// assertions omitted from durable snapshots. This is suitable for exact
    /// replacement, not a substitute for a durable graph snapshot.
    ///
    /// `max_edges` limits pair rows. `max_bytes` independently limits each of
    /// the raw payload and returned row JSON arrays, including array framing.
    /// Both limits are checked before cloning any persisted rows. Pair lookup
    /// scans source adjacency metadata; unrelated payloads are not serialized.
    pub fn persisted_resource_edges_between_bounded(
        &self,
        from: Uuid,
        to: Uuid,
        max_edges: usize,
        max_bytes: usize,
    ) -> Result<Vec<PersistedEdge>, ResourceEdgesBoundError> {
        let (Some(source), Some(target)) =
            (self.resources.key_of(&from), self.resources.key_of(&to))
        else {
            return Err(ResourceEdgesBoundError::MissingEndpoint);
        };
        let pair = || self.resources.inner().edges_connecting(source, target);
        let mut count = 0usize;
        for _ in pair() {
            if count == max_edges {
                return Err(ResourceEdgesBoundError::EdgeLimitExceeded);
            }
            count += 1;
        }

        measure_array(pair().map(|edge| edge.weight()), max_bytes)?;
        measure_array(
            pair().map(|edge| BorrowedEdge::new(from, to, edge.weight())),
            max_bytes,
        )?;

        Ok(pair()
            .map(|edge| {
                let payload = edge.weight();
                let mut persisted = super::snapshot::persisted_edge_for_ids(from, to, payload);
                persisted.assertions = Some(
                    payload
                        .assertions()
                        .iter()
                        .map(super::snapshot::to::persisted_assertion)
                        .collect(),
                );
                persisted
            })
            .collect())
    }
}

struct BoundedWriter {
    bytes: usize,
    limit: usize,
    exceeded: bool,
}

impl Write for BoundedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        match self.bytes.checked_add(buffer.len()) {
            Some(total) if total <= self.limit => {
                self.bytes = total;
                Ok(buffer.len())
            },
            _ => {
                self.exceeded = true;
                Err(io::Error::other("byte limit exceeded"))
            },
        }
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn measure_array<I, T>(items: I, limit: usize) -> Result<(), ResourceEdgesBoundError>
where
    I: IntoIterator<Item = T>,
    T: Serialize,
{
    let mut writer = BoundedWriter {
        bytes: 0,
        limit,
        exceeded: false,
    };
    let result = (|| -> Result<(), serde_json::Error> {
        writer.write_all(b"[").map_err(serde_json::Error::io)?;
        let mut first = true;
        for item in items {
            if !first {
                writer.write_all(b",").map_err(serde_json::Error::io)?;
            }
            first = false;
            serde_json::to_writer(&mut writer, &item)?;
        }
        writer.write_all(b"]").map_err(serde_json::Error::io)?;
        Ok(())
    })();
    match result {
        Ok(()) => Ok(()),
        Err(_) if writer.exceeded => Err(ResourceEdgesBoundError::ByteLimitExceeded),
        Err(_) => Err(ResourceEdgesBoundError::Serialization),
    }
}

// Borrow the live data while matching PersistedEdge's JSON shape. The parity
// test detects any change in the canonical conversion or serialized shape.
#[derive(Serialize)]
pub(super) struct BorrowedEdge<'a> {
    from_node_id: String,
    to_node_id: String,
    families: BTreeSet<EdgeFamily>,
    semantic: Option<()>,
    traversal: Option<&'a TraversalData>,
    containment: Option<()>,
    arrangement: Option<()>,
    imported: Option<()>,
    provenance: Option<()>,
    assertions: BorrowedAssertions<'a>,
}

impl<'a> BorrowedEdge<'a> {
    pub(super) fn new(from: Uuid, to: Uuid, payload: &'a EdgePayload) -> Self {
        Self {
            from_node_id: from.to_string(),
            to_node_id: to.to_string(),
            families: payload.families(),
            semantic: None,
            traversal: payload.traversal_data(),
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
            assertions: BorrowedAssertions(payload.assertions()),
        }
    }
}

struct BorrowedAssertions<'a>(&'a [SemanticStatement]);

impl Serialize for BorrowedAssertions<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for statement in self.0 {
            seq.serialize_element(&BorrowedStatement(statement))?;
        }
        seq.end()
    }
}

struct BorrowedStatement<'a>(&'a SemanticStatement);

impl Serialize for BorrowedStatement<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let statement = self.0;
        let mut state = serializer.serialize_struct("PersistedSemanticStatement", 10)?;
        state.serialize_field("statement_id", &statement.statement_id)?;
        state.serialize_field("predicate", &statement.predicate)?;
        if statement.recognized_sub_kind.is_some() {
            state.serialize_field("recognized_sub_kind", &statement.recognized_sub_kind)?;
        }
        if statement.label.is_some() {
            state.serialize_field("label", &statement.label)?;
        }
        if statement.graph_scope != GraphScope::Default {
            state.serialize_field("graph_scope", &statement.graph_scope)?;
        }
        if statement.provenance_iri.is_some() {
            state.serialize_field("provenance_iri", &statement.provenance_iri)?;
        }
        if statement.asserted_at_ms.is_some() {
            state.serialize_field("asserted_at_ms", &statement.asserted_at_ms)?;
        }
        if statement.first_asserted_ms.is_some() || statement.asserted_at_ms.is_some() {
            state.serialize_field("first_asserted_ms", &statement.first_asserted_ms)?;
        }
        if statement.withdrawn.is_some() {
            state.serialize_field("withdrawn", &statement.withdrawn)?;
        }
        if !statement.basis.is_empty() || statement.label.is_some() {
            state.serialize_field("basis", &statement.basis)?;
        }
        state.end()
    }
}
