// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl serde::Serialize for PersistedSemanticStatement {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let human = serializer.is_human_readable();
        let scoped = self.graph_scope != GraphScope::Default;
        let fields = if human {
            2 + usize::from(self.label.is_some())
                + usize::from(scoped)
                + usize::from(self.provenance_iri.is_some())
                + usize::from(self.asserted_at_ms.is_some())
                + usize::from(self.first_asserted_ms.is_some() || self.asserted_at_ms.is_some())
                + usize::from(self.withdrawn.is_some())
                + usize::from(!self.basis.is_empty() || self.label.is_some())
        } else {
            9
        };
        // Positional serializers receive every field in the current shape.
        let mut state = serializer.serialize_struct("PersistedSemanticStatement", fields)?;
        state.serialize_field("statement_id", &self.statement_id)?;
        state.serialize_field("predicate", &self.predicate)?;
        if !human || self.label.is_some() {
            state.serialize_field("label", &self.label)?;
        }
        if !human || scoped {
            state.serialize_field("graph_scope", &self.graph_scope)?;
        }
        if !human || self.provenance_iri.is_some() {
            state.serialize_field("provenance_iri", &self.provenance_iri)?;
        }
        if !human || self.asserted_at_ms.is_some() {
            state.serialize_field("asserted_at_ms", &self.asserted_at_ms)?;
        }
        if !human || self.first_asserted_ms.is_some() || self.asserted_at_ms.is_some() {
            state.serialize_field("first_asserted_ms", &self.first_asserted_ms)?;
        }
        if !human || self.withdrawn.is_some() {
            state.serialize_field("withdrawn", &self.withdrawn)?;
        }
        if !human || !self.basis.is_empty() || self.label.is_some() {
            state.serialize_field("basis", &self.basis)?;
        }
        state.end()
    }
}

impl serde::Serialize for PersistedEdge {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        // JSON writes the pair and its records; the legacy sidecars are read
        // only (D17). A positional format keeps each field in its place.
        let human = serializer.is_human_readable();
        let records = !human || self.assertions.is_some();
        let fields = if human { 2 + usize::from(records) } else { 7 };
        let mut state = serializer.serialize_struct("PersistedEdge", fields)?;
        state.serialize_field("from_node_id", &self.from_node_id)?;
        state.serialize_field("to_node_id", &self.to_node_id)?;
        if !human {
            state.serialize_field("semantic", &self.semantic)?;
            state.serialize_field("containment", &self.containment)?;
            state.serialize_field("imported", &self.imported)?;
            state.serialize_field("provenance", &self.provenance)?;
        }
        if records {
            state.serialize_field("assertions", &self.assertions)?;
        }
        state.end()
    }
}
