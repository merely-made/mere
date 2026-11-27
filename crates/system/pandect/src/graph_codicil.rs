// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph codicils — freezing a live graph into an immutable, content-addressed
//! eidetic codicil, and thawing one back into a live graph.
//!
//! This is the spine of the Alembic memory subsystem (slice A of
//! `design_docs/archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md`):
//! "Save as graph codicil" freezes a session graph's snapshot plus its atomic
//! facet store, redacts private / heavy state, and writes it through eidetic's
//! runtime-schema payload layer under the schema ids a host's [`CodicilHook`]
//! names; "Open as session" thaws a codicil back into a graph. Browsing a
//! codicil is read-only (immutability holds); editing forks a thaw.
//!
//! The payloads, schema ids and the generic save, open and list paths are
//! here, over any [`SessionGraph`]; a host's graph supplies its snapshot and
//! redaction through its hook (engine split D265, D266). It is
//! store-backend-agnostic (it speaks the eidetic [`Store`] trait), so a wasm
//! host's OPFS store works the same way.

pub use eidetic::ManifestId;
use eidetic::{ProvenanceOrigin, ProvenanceRecord, Result, SchemaRef, Store, Timestamp};
use mere::session::SessionGraph;
/// Complete candidate graph passed to product-owned validators.
pub type GraphCandidate<G> = G;

use crate::graph_placement::PlacementProfile;
use crate::{Change, GraphSessionManifest, NodeFacetStore, ViewEntry, ViewState};

/// Schema id bytes for the graph-snapshot codicil schema. The [`SchemaRef`] is the
/// BLAKE3 of these bytes, so it is stable across builds and machines.
pub const GRAPH_SNAPSHOT_SCHEMA_ID: &[u8] = b"mere.graph-snapshot/v4";
/// The input adapters' schema ids, which a host's [`CodicilHook`] names too.
pub const LEGACY_GRAPH_SNAPSHOT_SCHEMA_ID: &[u8] = b"mere.graph-snapshot/v1";
pub const V2_GRAPH_SNAPSHOT_SCHEMA_ID: &[u8] = b"mere.graph-snapshot/v2";
pub const V3_PROFILED_GRAPH_SNAPSHOT_SCHEMA_ID: &[u8] = b"mere.graph-snapshot/v3";
pub const PROFILED_GRAPH_SNAPSHOT_SCHEMA_ID: &[u8] = b"mere.graph-snapshot-profiled/v4";

pub fn profiled_graph_snapshot_schema_ref() -> SchemaRef {
    SchemaRef::from_id(ManifestId::of_blob(PROFILED_GRAPH_SNAPSHOT_SCHEMA_ID))
}

/// The content-addressed schema reference every graph codicil is tagged with.
///
/// The eidetic typed layer uses this as an identity tag — checked on load so a
/// mistyped read errors rather than deserializing garbage. Registering a full
/// schema-definition codicil (with validators, for cross-language / federation
/// reads) is a later, optional step; the freeze/thaw spine does not need it.
pub fn graph_snapshot_schema_ref() -> SchemaRef {
    SchemaRef::from_id(ManifestId::of_blob(GRAPH_SNAPSHOT_SCHEMA_ID))
}

/// A graph snapshot and its one live facet store, bound to the graph-codicil
/// schema.
///
/// Version 1 carried a bare snapshot. Version 2 added facets, version 3
/// added an explicit placement profile, and version 4 stores assertion records
/// as the canonical edge representation.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct GraphCodicil<G: SessionGraph> {
    pub snapshot: G::Snapshot,
    #[serde(default)]
    pub facets: NodeFacetStore,
    /// Optional reservoir history. Graph-only v2 codicils decode unchanged.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sessions: Vec<ArchivedSession<G>>,
}

/// One immutable source session's complete graph and parallel histories.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ArchivedSession<G: SessionGraph> {
    pub manifest: GraphSessionManifest,
    pub baseline: G::Snapshot,
    pub baseline_facets: NodeFacetStore,
    pub journal: Vec<mere::model::Attributed<G::Delta>>,
    pub changes: Vec<Change>,
    pub views: Vec<ArchivedView>,
}

// Debug by hand, so it asks nothing of the graph type itself (D244).
impl<G: SessionGraph> std::fmt::Debug for GraphCodicil<G>
where
    G::Snapshot: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GraphCodicil")
            .field("snapshot", &self.snapshot)
            .field("facets", &self.facets)
            .field("sessions", &self.sessions)
            .finish()
    }
}

impl<G: SessionGraph> std::fmt::Debug for ArchivedSession<G>
where
    G::Snapshot: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchivedSession")
            .field("manifest", &self.manifest)
            .field("baseline", &self.baseline)
            .field("baseline_facets", &self.baseline_facets)
            .field("journal", &self.journal)
            .field("changes", &self.changes)
            .field("views", &self.views)
            .finish()
    }
}

/// One source application's view and its full parallel stream.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ArchivedView {
    pub app: String,
    pub view: String,
    pub state: ViewState,
    pub log: Vec<ViewEntry>,
}

/// Placement belongs to the input envelope, never to inferred graph contents.
#[derive(Clone, serde::Serialize, serde::Deserialize)]
#[serde(bound = "")]
pub struct ProfiledGraphCodicil<G: SessionGraph> {
    #[serde(flatten)]
    pub codicil: GraphCodicil<G>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<PlacementProfile>,
}

impl<G: SessionGraph> std::fmt::Debug for ProfiledGraphCodicil<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProfiledGraphCodicil")
            .field("codicil", &self.codicil)
            .field("placement", &self.placement)
            .finish()
    }
}

#[path = "graph_codicil_hook.rs"]
mod hook;
pub use hook::{
    CodicilHook, list_graph_codicils_with, load_profiled_graph_codicil_sealed_with,
    open_codicil_as_session_sealed_with, save_graph_snapshot_codicil_sealed_with,
    save_profiled_graph_codicil_sealed_with, save_profiled_session_codicil_checked_with,
};

/// What a graph codicil keeps from the live snapshot.
///
/// The default is conservative (Alembic plan open decision #7): private / heavy
/// per-node state is stripped, leaving graph structure, addresses, titles,
/// tags, classifications, provenance, properties, and other portable facets.
/// Callers opt state back in explicitly with [`RedactionPolicy::include_all`]
/// or by setting a flag, so a shareable codicil does not leak screenshots,
/// drafts, or browser-runtime facets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RedactionPolicy {
    /// Keep preview/snapshot image references. Off by default (heavy, private).
    pub include_thumbnails: bool,
    /// Keep favicon image references. Off by default (heavy).
    pub include_favicons: bool,
    /// Keep legacy session state and `web.*` runtime facets. Off by default.
    pub include_session_state: bool,
}

impl Default for RedactionPolicy {
    fn default() -> Self {
        Self {
            include_thumbnails: false,
            include_favicons: false,
            include_session_state: false,
        }
    }
}

impl RedactionPolicy {
    /// Include everything — no redaction. For a local, trusted freeze where the
    /// heavy / private fields are wanted; never for a shareable codicil.
    pub fn include_all() -> Self {
        Self {
            include_thumbnails: true,
            include_favicons: true,
            include_session_state: true,
        }
    }

    /// Drop the private runtime facets unless session state is kept.
    fn apply_facets(&self, facets: &mut NodeFacetStore) {
        if self.include_session_state {
            return;
        }
        let private_runtime_facets = [
            "web.scroll",
            "web.form_draft",
            "web.viewer",
            "web.compat",
            "web.content",
            "web.page_scale",
        ];
        let nodes = facets.iter().map(|(node, _)| *node).collect::<Vec<_>>();
        for node in nodes {
            for facet in private_runtime_facets {
                facets.remove(&node, &portolan::FacetId::new(facet));
            }
        }
    }
}

/// Provenance for a locally-frozen graph codicil: generated by this tool at the
/// freeze time, no upstream ancestry (composition / merge fills `upstream` later).
fn graph_codicil_provenance(created_at: Timestamp) -> ProvenanceRecord {
    ProvenanceRecord {
        origin: ProvenanceOrigin::Generated,
        upstream: Vec::new(),
        tooling: Some(concat!("pandect/graph-codicil@", env!("CARGO_PKG_VERSION")).to_string()),
        generated_at: created_at,
    }
}

/// The short form a person reads for a codicil id: its algorithm and the
/// first digits, such as `blake3:f612a`.
pub fn codicil_short_id(id: &str) -> &str {
    id.get(..12).unwrap_or(id)
}

/// The name a session opened from a codicil carries, so it never reads as
/// its source: the source session's name (or `Session`) and the codicil it
/// came from, such as `Session (from blake3:f612a)`.
pub fn opened_codicil_session_name(source: Option<&str>, codicil: &str) -> String {
    format!(
        "{} (from {})",
        source.unwrap_or("Session"),
        codicil_short_id(codicil)
    )
}

/// Parse Eidetic's self-describing id (and its legacy bare BLAKE3 digest).
pub fn parse_codicil_id(id: &str) -> std::result::Result<ManifestId, String> {
    eidetic::Hash::parse(id)
        .map(ManifestId)
        .map_err(|e| e.to_string())
}

/// The archive timestamp, using the engine's browser-safe clock.
pub fn archive_timestamp() -> Timestamp {
    Timestamp(
        mere::model::clock::wall_clock_now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64,
    )
}
