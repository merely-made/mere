// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A frozen payload's statements as stored, and as today's types re-encode them.
fn keys_hold(stored: serde_json::Value) -> Result<(), String> {
    let edges: Vec<PersistedEdge> = serde_json::from_value(stored["statements"].clone()).unwrap();
    let mut known = serde_json::json!({ "statements": edges });
    known_edge_keys(&stored, &mut known);
    validate_retained_keys(&stored, &known)
}

#[test]
fn a_frozen_payload_written_before_d28_still_validates() {
    let core = crate::profile::core::iri(crate::profile::core::ids::REL_CITES);
    let edge = |record: serde_json::Value| {
        serde_json::json!({ "statements": [{
            "from_node_id": "a", "to_node_id": "b", "families": ["Semantic"],
            "assertions": [record],
        }]})
    };
    // Before D28: a core predicate by IRI alone, with its old sub-kind name.
    let before = edge(serde_json::json!({
        "statement_id": "s", "predicate": core, "recognized_sub_kind": "Cites",
    }));
    assert_eq!(keys_hold(before), Ok(()));
    // Control: a key no engine ever wrote is still refused.
    let foreign = edge(serde_json::json!({
        "statement_id": "s", "predicate": core, "invented": 1,
    }));
    assert!(keys_hold(foreign).is_err());
}
