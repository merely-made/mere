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
            2 + usize::from(self.recognized_sub_kind.is_some())
                + usize::from(self.label.is_some())
                + usize::from(scoped)
                + usize::from(self.provenance_iri.is_some())
                + usize::from(self.asserted_at_ms.is_some())
                + usize::from(self.first_asserted_ms.is_some() || self.asserted_at_ms.is_some())
                + usize::from(self.withdrawn.is_some())
                + usize::from(!self.basis.is_empty() || self.label.is_some())
        } else {
            10
        };
        // Positional serializers receive every field in the current shape.
        let mut state = serializer.serialize_struct("PersistedSemanticStatement", fields)?;
        state.serialize_field("statement_id", &self.statement_id)?;
        state.serialize_field("predicate", &self.predicate)?;
        if !human || self.recognized_sub_kind.is_some() {
            state.serialize_field("recognized_sub_kind", &self.recognized_sub_kind)?;
        }
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
