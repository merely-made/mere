// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! View-local graph curation: what a view chooses to show of a graph, apart
//! from the graph itself (Scenograph editor plan, SE67 and SE73; mer3ly site
//! plan, Ruling 153).
//!
//! It holds subgraph specifications now, moved out of forme so that crates
//! without forme's layout stack (scenograph's swatch scope among them) can
//! name a subgraph; forme re-exports them, so its callers are unchanged.
//! Folds join in the native phase.

use serde::{Deserialize, Serialize};

/// Canonical subgraph specification (referenced by Linked bindings).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubgraphSpec {
    pub kind: SubgraphKind,
    pub anchors: Vec<String>,
    pub primary_anchor: Option<String>,
    pub selectors: Vec<String>,
}

/// The 9 canonical subgraph shapes from `subgraph_model.md`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubgraphKind {
    Ego { radius: u8 },
    Corridor,
    Component,
    Loop,
    Frontier,
    Facet,
    Session,
    Bridge,
    WorkbenchCorrespondence,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_spec_reads_and_writes_the_shape_forme_wrote() {
        // The wire as forme serialized it before the move.
        let wire = r#"{"kind":{"Ego":{"radius":2}},"anchors":["a"],"primary_anchor":"a","selectors":["kind:web"]}"#;
        let spec: SubgraphSpec = serde_json::from_str(wire).expect("reads");
        assert_eq!(spec.kind, SubgraphKind::Ego { radius: 2 });
        assert_eq!(serde_json::to_string(&spec).expect("writes"), wire);
        let unit: SubgraphKind =
            serde_json::from_str(r#""WorkbenchCorrespondence""#).expect("reads");
        assert_eq!(unit, SubgraphKind::WorkbenchCorrespondence);
    }
}
