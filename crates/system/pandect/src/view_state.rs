// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A view's state as the session stores it: a schema id and a value that
//! pandect keeps beside the journal without reading (engine split D44). The
//! host defines and edits the value (Graphshell's `incipit::view::ViewIntent`
//! is one, under `mere.view-intent/v1`).
//!
//! Stored as `{"schema": …, "value": …}`. Entries written before the split
//! are a bare view object with no schema; they read as
//! [`UNTAGGED_VIEW_SCHEMA`], the schema every such entry had.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};

/// The schema of a view entry stored before view states carried one.
pub const UNTAGGED_VIEW_SCHEMA: &str = "mere.view-intent/v1";

/// One view's state: the schema its value follows, and the value.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
pub struct ViewState {
    pub schema: String,
    pub value: serde_json::Value,
}

impl ViewState {
    pub fn new(schema: impl Into<String>, value: serde_json::Value) -> Self {
        Self {
            schema: schema.into(),
            value,
        }
    }

    /// Encode `value` under `schema`.
    pub fn encode<T: Serialize>(
        schema: impl Into<String>,
        value: &T,
    ) -> Result<Self, serde_json::Error> {
        Ok(Self::new(schema, serde_json::to_value(value)?))
    }

    /// The value as `T` when it follows `schema`; `None` for another schema
    /// or a value that does not decode.
    pub fn decode<T: DeserializeOwned>(&self, schema: &str) -> Option<T> {
        (self.schema == schema)
            .then(|| serde_json::from_value(self.value.clone()).ok())
            .flatten()
    }
}

impl<'de> Deserialize<'de> for ViewState {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_json::Value::deserialize(deserializer)?;
        if let serde_json::Value::Object(object) = &value
            && object.len() == 2
            && let (Some(serde_json::Value::String(schema)), Some(inner)) =
                (object.get("schema"), object.get("value"))
        {
            return Ok(Self::new(schema.clone(), inner.clone()));
        }
        Ok(Self::new(UNTAGGED_VIEW_SCHEMA, value))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_tagged_state_round_trips() {
        let state = ViewState::new("test.view/v1", json!({"focus": "n1"}));
        let stored = serde_json::to_string(&state).unwrap();
        assert_eq!(
            stored,
            r#"{"schema":"test.view/v1","value":{"focus":"n1"}}"#
        );
        assert_eq!(serde_json::from_str::<ViewState>(&stored).unwrap(), state);
    }

    #[test]
    fn a_pre_split_entry_reads_under_the_untagged_schema() {
        let legacy = r#"{"hidden_relations":[],"focus":"n1","mirror_tiles":false}"#;
        let state: ViewState = serde_json::from_str(legacy).unwrap();
        assert_eq!(state.schema, UNTAGGED_VIEW_SCHEMA);
        assert_eq!(state.value["focus"], "n1");
        let again: ViewState =
            serde_json::from_str(&serde_json::to_string(&state).unwrap()).unwrap();
        assert_eq!(again, state);
    }

    #[test]
    fn decode_checks_the_schema() {
        let state = ViewState::encode("a/v1", &json!({"x": 1})).unwrap();
        assert_eq!(
            state.decode::<serde_json::Value>("a/v1"),
            Some(json!({"x": 1}))
        );
        assert_eq!(state.decode::<serde_json::Value>("b/v1"), None);
    }
}
