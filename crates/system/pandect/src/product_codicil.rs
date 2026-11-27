// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph-selection codicil a product export writes and a transfer carries,
//! and its checked decode (engine split D70).
//!
//! Moved from the reference app's `product` and `transfer` so the resident can
//! summarize a transfer without linking the app. Producing a codicil (export)
//! and applying one (import) stay the app's.

use std::collections::HashSet;
use std::sync::LazyLock;

use eidetic::{
    Hash, ManifestId, MereNativeFieldSpec, MereNativeSchemaBuilder, SchemaDefinition, SchemaRef,
    TypedPayload,
};
use mere::session::SessionGraph;
use serde::{Deserialize, Serialize};

use crate::graph_placement::{PlacementProfile, materialize_snapshot_with};
use crate::{NodeFacetStore, SavedSceneV3};

pub const PRODUCT_CODICIL_SCHEMA: &str = "graphshell.graph-codicil/v3";

static PRODUCT_CODICIL_SCHEMA_REF: LazyLock<SchemaRef> =
    LazyLock::new(|| schema_ref(&product_codicil_schema()));

/// The schema reference a definition is stored under.
pub fn schema_ref(definition: &SchemaDefinition) -> SchemaRef {
    let bytes = serde_json::to_vec(definition).expect("Graphshell schema always serializes");
    SchemaRef::from_id(ManifestId::from_hash(Hash::of(&bytes)))
}

pub fn product_codicil_schema() -> SchemaDefinition {
    MereNativeSchemaBuilder::new("graphshell.GraphCodicil/v3")
        .description("A closed graph or scene selection with portable facets.")
        .field("schema", MereNativeFieldSpec::String, true)
        .field(
            "scope",
            MereNativeFieldSpec::Enum {
                values: vec![
                    "object-only".to_string(),
                    "direct-relations".to_string(),
                    "selected-subgraph".to_string(),
                    "saved-scene".to_string(),
                ],
            },
            true,
        )
        .field("exported_at_ms", MereNativeFieldSpec::U64, true)
        .field("graph", MereNativeFieldSpec::Object, true)
        .field("facets", MereNativeFieldSpec::Object, true)
        .field("scene", MereNativeFieldSpec::Object, false)
        .build()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TransferScope {
    ObjectOnly,
    DirectRelations,
    SelectedSubgraph,
    SavedScene,
}

impl TransferScope {
    pub fn code(self) -> &'static str {
        match self {
            Self::ObjectOnly => "object-only",
            Self::DirectRelations => "direct-relations",
            Self::SelectedSubgraph => "selected-subgraph",
            Self::SavedScene => "saved-scene",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "object-only" => Some(Self::ObjectOnly),
            "direct-relations" => Some(Self::DirectRelations),
            "selected-subgraph" => Some(Self::SelectedSubgraph),
            "saved-scene" => Some(Self::SavedScene),
            _ => None,
        }
    }
}

/// A graph selection, with a version-4 scene when it carries one (F159).
///
/// Generic over the graph whose snapshot it carries (engine split D254); a
/// consumer names its graph through its per-crate alias.
#[derive(Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct ProductCodicilV3<G: SessionGraph> {
    pub schema: String,
    pub scope: TransferScope,
    pub exported_at_ms: u64,
    pub graph: G::Snapshot,
    pub facets: NodeFacetStore,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<SavedSceneV3>,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(bound = "")]
pub struct ProfiledProductCodicil<G: SessionGraph> {
    #[serde(flatten)]
    pub product: ProductCodicilV3<G>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<PlacementProfile>,
}

// Debug by hand, so it asks nothing of the graph type itself (D244).
impl<G: SessionGraph> std::fmt::Debug for ProductCodicilV3<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProductCodicilV3")
            .field("schema", &self.schema)
            .field("scope", &self.scope)
            .field("exported_at_ms", &self.exported_at_ms)
            .field("graph", &self.graph)
            .field("facets", &self.facets)
            .field("scene", &self.scene)
            .finish()
    }
}

impl<G: SessionGraph> std::fmt::Debug for ProfiledProductCodicil<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfiledProductCodicil")
            .field("product", &self.product)
            .field("placement", &self.placement)
            .finish()
    }
}

impl<G: SessionGraph> std::ops::Deref for ProfiledProductCodicil<G> {
    type Target = ProductCodicilV3<G>;
    fn deref(&self) -> &Self::Target {
        &self.product
    }
}

impl<G: SessionGraph> std::ops::DerefMut for ProfiledProductCodicil<G> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.product
    }
}

impl<G: SessionGraph> TypedPayload for ProductCodicilV3<G> {
    fn schema_ref() -> SchemaRef {
        *PRODUCT_CODICIL_SCHEMA_REF
    }
}

impl<G: SessionGraph> TypedPayload for ProfiledProductCodicil<G> {
    fn schema_ref() -> SchemaRef {
        *PRODUCT_CODICIL_SCHEMA_REF
    }
}

/// A codicil that does not read as a closed graph selection.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvalidCodicil(pub String);

impl std::fmt::Display for InvalidCodicil {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "invalid graph codicil: {}", self.0)
    }
}

impl std::error::Error for InvalidCodicil {}

/// Materialize a codicil's graph under its placement profile, with its
/// facets, over any graph (engine split D257).
pub fn profiled_codicil_graph_with<G: SessionGraph>(
    snapshot: G::Snapshot,
    facets: NodeFacetStore,
    placement: Option<PlacementProfile>,
) -> Result<G, InvalidCodicil> {
    let mut graph = materialize_snapshot_with::<G>(&snapshot, placement).map_err(InvalidCodicil)?;
    graph.install_facets(facets).map_err(InvalidCodicil)?;
    Ok(graph)
}

/// Decode a codicil's payload, refusing a foreign schema, a relation naming an
/// object outside the selection, and a graph that does not materialize, over
/// any graph (engine split D254). The selection check reads node ids and
/// relation ends from the snapshot's stored form, which every graph's
/// snapshot keeps.
pub fn decode_profiled_codicil_with<G: SessionGraph>(
    bytes: &[u8],
) -> Result<ProfiledProductCodicil<G>, InvalidCodicil> {
    #[derive(Deserialize)]
    struct Head {
        schema: String,
        #[serde(default)]
        placement: Option<PlacementProfile>,
    }
    #[derive(Deserialize)]
    struct Selection {
        nodes: Vec<SelectedNode>,
        edges: Vec<SelectedEdge>,
    }
    #[derive(Deserialize)]
    struct SelectedNode {
        node_id: String,
    }
    #[derive(Deserialize)]
    struct SelectedEdge {
        from_node_id: String,
        to_node_id: String,
    }
    let invalid = |error: serde_json::Error| InvalidCodicil(error.to_string());
    let Head { schema, placement } = serde_json::from_slice(bytes).map_err(invalid)?;
    if schema != PRODUCT_CODICIL_SCHEMA {
        return Err(InvalidCodicil(format!(
            "expected {PRODUCT_CODICIL_SCHEMA}, found {schema}"
        )));
    }
    let codicil = serde_json::from_slice::<ProductCodicilV3<G>>(bytes).map_err(invalid)?;
    let selection = serde_json::to_value(&codicil.graph)
        .and_then(serde_json::from_value::<Selection>)
        .map_err(invalid)?;
    let ids: HashSet<_> = selection
        .nodes
        .iter()
        .map(|node| node.node_id.as_str())
        .collect();
    if selection.edges.iter().any(|edge| {
        !ids.contains(edge.from_node_id.as_str()) || !ids.contains(edge.to_node_id.as_str())
    }) {
        return Err(InvalidCodicil(
            "a relation names an object outside the codicil".to_string(),
        ));
    }
    let input = ProfiledProductCodicil {
        product: codicil,
        placement,
    };
    profiled_codicil_graph_with::<G>(input.graph.clone(), input.facets.clone(), input.placement)?;
    Ok(input)
}
