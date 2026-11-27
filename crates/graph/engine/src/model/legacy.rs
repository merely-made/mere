// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a stored legacy translation names: the placement's endpoint choices,
//! the source mere's identity and the content origins a migration keeps.
//! These are the shapes mere-kernel's legacy migrations wrote, held here
//! unchanged (engine split lane 4) so stored placements, receipts and their
//! digests keep reading after the kernel leaves Mere. The migrations
//! themselves are the surface graph's.

use super::Author;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// An explicit endpoint choice for an ambiguous carried handle. No raw handle is aliased.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyResourceOriginResolution {
    pub statement_id: String,
    pub from_resource_id: Uuid,
    pub to_resource_id: Uuid,
}

/// Stable identity of the source mere, required only for unattributed tag concepts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyContentContext {
    pub original_mere_iri: String,
}

/// Where a legacy surface's content came from, kept on the resource it moved to.
pub const LEGACY_CONTENT_ORIGINS: &str = "semantic.legacy-content-origins/v1";

/// One legacy surface record's origin, as the legacy content migration keeps it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LegacyContentOrigin {
    pub kind: String,
    pub record_id: String,
    pub surface_id: Uuid,
    pub original_url: String,
    pub author: Option<Author>,
    pub uncertainty: Option<String>,
    pub original: serde_json::Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The stored shapes, field for field: a placement's origins, a receipt's
    /// content context and a resource's content origins are written and
    /// hashed as these JSON objects.
    #[test]
    fn the_stored_legacy_shapes_are_the_kernels() {
        let id = Uuid::from_u128(1);
        let origin = LegacyResourceOriginResolution {
            statement_id: "s".into(),
            from_resource_id: id,
            to_resource_id: id,
        };
        let one = "00000000-0000-0000-0000-000000000001";
        assert_eq!(
            serde_json::to_string(&origin).unwrap(),
            format!(
                r#"{{"statement_id":"s","from_resource_id":"{one}","to_resource_id":"{one}"}}"#
            )
        );
        let context = LegacyContentContext {
            original_mere_iri: "urn:mere:a".into(),
        };
        assert_eq!(
            serde_json::to_string(&context).unwrap(),
            r#"{"original_mere_iri":"urn:mere:a"}"#
        );
        let note = LegacyContentOrigin {
            kind: "tag".into(),
            record_id: "r".into(),
            surface_id: id,
            original_url: "https://a.test/".into(),
            author: None,
            uncertainty: None,
            original: serde_json::json!({"n": 1}),
        };
        assert_eq!(
            serde_json::to_string(&note).unwrap(),
            format!(
                r#"{{"kind":"tag","record_id":"r","surface_id":"{one}","original_url":"https://a.test/","author":null,"uncertainty":null,"original":{{"n":1}}}}"#
            )
        );
        // Control: the comparison sees a changed field.
        let moved = LegacyContentContext {
            original_mere_iri: "urn:mere:b".into(),
        };
        assert_ne!(
            serde_json::to_string(&moved).unwrap(),
            serde_json::to_string(&context).unwrap()
        );
    }
}
