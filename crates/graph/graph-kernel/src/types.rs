// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Portable leaf types used by the graph model and persistence schema.
//!
//! These types are shared between `kernel` modules and must be
//! WASM-clean: no platform I/O, no UI framework dependencies.

use std::collections::HashMap;

use rkyv::{Archive, Deserialize, Serialize};

use crate::graph::ProvenanceSubKind;

// The assertion model's leaf types moved to `mere::model::types` (engine split
// P5 L3). Names other repositories use stay public here; the rest are for the
// kernel only.
pub use mere::model::{
    AssertionBasis, AssertionMethod, AssertionWithdrawal, GraphScope, NodeProperty,
};
#[allow(unused_imports)]
pub(crate) use mere::model::{
    ClassificationProvenance, ClassificationScheme, ClassificationStatus, NodeClassification,
    mint_local_statement_id, seed_statement_minter,
};

// ---------------------------------------------------------------------------
// Frame layout types
// ---------------------------------------------------------------------------

/// Durable member reference used by frame layout hints.
///
/// `NodeKey` is process-local and not stable across restart, so persistent frame
/// layout metadata uses the member node's stable UUID string instead.
pub type FrameLayoutNodeId = String;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub enum SplitOrientation {
    Vertical,
    Horizontal,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub enum DominantEdge {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub enum FrameLayoutHint {
    SplitHalf {
        first: FrameLayoutNodeId,
        second: FrameLayoutNodeId,
        orientation: SplitOrientation,
    },
    SplitPamphlet {
        members: [FrameLayoutNodeId; 3],
        orientation: SplitOrientation,
    },
    SplitTriptych {
        dominant: FrameLayoutNodeId,
        dominant_edge: DominantEdge,
        wings: [FrameLayoutNodeId; 2],
    },
    SplitQuartered {
        top_left: FrameLayoutNodeId,
        top_right: FrameLayoutNodeId,
        bottom_left: FrameLayoutNodeId,
        bottom_right: FrameLayoutNodeId,
    },
}

// ---------------------------------------------------------------------------
// Import provenance types
// ---------------------------------------------------------------------------

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NodeImportProvenance {
    pub source_id: String,
    pub source_label: String,
}

/// Cross-graph derivation provenance: a record that this node was derived from
/// a node in another graph (the tear-out "fork" / cross-graph copy).
///
/// This is the node-anchored analog of an in-graph `Provenance` edge. A
/// node→node derivation *within* one graph rides a petgraph `Provenance` edge,
/// but a cross-graph derivation has its object node in a *different* graph, so
/// it cannot be a petgraph edge — it is recorded here on the derived node
/// instead, beside [`NodeImportProvenance`] and `NodeClassification`. It
/// projects to a `<this> {provenance predicate} <source>` statement when the
/// RDF projection lands (per the petgraph-rdf projection profile).
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct NodeDerivation {
    /// Which provenance relation this is (e.g. [`ProvenanceSubKind::CopiedFrom`]).
    pub sub_kind: ProvenanceSubKind,
    /// The source node's stable id, as a string (matches the kernel's
    /// node-id-as-string convention in persisted records). The object of the
    /// derivation statement.
    pub source_node: String,
    /// The source graph's id (the constellation `GraphId` rendered as a string),
    /// or `None` when same-graph or unknown — the named-graph scope of the source.
    pub source_graph: Option<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ImportRecordMembership {
    pub node_id: String,
    pub suppressed: bool,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct ImportRecord {
    pub record_id: String,
    pub source_id: String,
    pub source_label: String,
    pub imported_at_secs: u64,
    pub memberships: Vec<ImportRecordMembership>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeImportRecordSummary {
    pub record_id: String,
    pub source_id: String,
    pub source_label: String,
    pub imported_at_secs: u64,
}

pub fn format_imported_at_secs(imported_at_secs: u64) -> String {
    time::OffsetDateTime::from_unix_timestamp(imported_at_secs as i64)
        .ok()
        .and_then(|timestamp| {
            timestamp
                .format(&time::format_description::well_known::Rfc3339)
                .ok()
        })
        .unwrap_or_else(|| format!("{}s", imported_at_secs))
}

// ---------------------------------------------------------------------------
// Preview imagery references (node image externalization plan)
// ---------------------------------------------------------------------------

/// Which preview-image role a reference fills. A node holds at most one image per
/// role. `Favicon` is the site icon on the node face; `Preview` the default
/// thumbnail; `Snapshot` the last-rendered peek the preview card shows. The set
/// is extensible (per-lane snapshots) by adding variants, which is why the node
/// keys images by role rather than holding fixed fields. `Ord` so it can key a
/// `BTreeMap` with a deterministic iteration order.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(
    compare(PartialEq, PartialOrd),
    derive(Debug, PartialEq, Eq, PartialOrd, Ord)
)]
pub enum ImageRole {
    Favicon,
    Preview,
    Snapshot,
}

/// A content-addressed reference to a preview image held in the durable blob
/// store (node image externalization plan): the BLAKE3-256 digest of the stored
/// PNG bytes plus the decoded dimensions. ~40 bytes and no pixels, so a graph of
/// 50k nodes carries references, not image data. The kernel only *carries* the
/// handle; `pandect::image_store` computes the digest and owns the blob
/// under `content/image/<hex>`, which is why the digest is a plain `[u8; 32]`
/// here (no eidetic dependency in the kernel).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub struct ImageRef {
    /// BLAKE3-256 digest of the stored PNG bytes — the blob key. Rendered as hex
    /// in JSON snapshots (compact + hand-inspectable); rkyv archives the raw bytes.
    #[serde(with = "digest_hex")]
    pub digest: [u8; 32],
    /// Decoded pixel width.
    pub width: u32,
    /// Decoded pixel height.
    pub height: u32,
}

/// Serde helpers rendering a 32-byte digest as a hex string in JSON snapshots.
#[path = "types/digest_hex.rs"]
mod digest_hex;

// ---------------------------------------------------------------------------
// Badge / tag presentation types (from badge.rs carve-out)
// ---------------------------------------------------------------------------

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub enum BadgeIcon {
    Emoji(String),
    Lucide(String),
    None,
}

#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub struct NodeTagPresentationState {
    pub ordered_tags: Vec<String>,
    pub icon_overrides: HashMap<String, BadgeIcon>,
}

impl ImageRef {
    pub fn new(digest: [u8; 32], width: u32, height: u32) -> Self {
        Self {
            digest,
            width,
            height,
        }
    }

    /// Lowercase hex of the digest — the `content/image/<hex>` blob-key suffix.
    /// Matches `eidetic::Hash::to_hex` so a ref built from a saved blob's hash
    /// reads that same blob back.
    pub fn hex(&self) -> String {
        use std::fmt::Write;
        let mut out = String::with_capacity(64);
        for byte in &self.digest {
            let _ = write!(out, "{byte:02x}");
        }
        out
    }
}
