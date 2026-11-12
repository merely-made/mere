// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl serde::Serialize for NodeProperty {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let human = serializer.is_human_readable();
        let scoped = self.graph_scope != GraphScope::Default;
        let fields = if human {
            3 + usize::from(self.datatype.is_some())
                + usize::from(self.lang.is_some())
                + usize::from(scoped)
                + usize::from(self.provenance_iri.is_some())
                + usize::from(self.asserted_at_ms.is_some())
        } else {
            8
        };
        // Positional codecs keep the complete legacy field order.
        let mut state = serializer.serialize_struct("NodeProperty", fields)?;
        state.serialize_field("statement_id", &self.statement_id)?;
        state.serialize_field("predicate", &self.predicate)?;
        state.serialize_field("value", &self.value)?;
        if !human || self.datatype.is_some() {
            state.serialize_field("datatype", &self.datatype)?;
        }
        if !human || self.lang.is_some() {
            state.serialize_field("lang", &self.lang)?;
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
        state.end()
    }
}

impl NodeProperty {
    pub fn new(predicate: String, value: String) -> Self {
        Self {
            statement_id: mint_local_statement_id(),
            predicate,
            value,
            datatype: None,
            lang: None,
            graph_scope: GraphScope::Default,
            provenance_iri: None,
            asserted_at_ms: None,
        }
    }

    pub fn with_graph_scope(mut self, graph_scope: GraphScope) -> Self {
        self.graph_scope = graph_scope;
        self
    }

    pub fn with_metadata(
        mut self,
        provenance_iri: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> Self {
        self.provenance_iri = provenance_iri;
        self.asserted_at_ms = asserted_at_ms;
        self
    }

    pub fn content_eq(&self, other: &Self) -> bool {
        self.predicate == other.predicate
            && self.value == other.value
            && self.datatype == other.datatype
            && self.lang == other.lang
            && self.graph_scope == other.graph_scope
            && self.provenance_iri == other.provenance_iri
    }
}
