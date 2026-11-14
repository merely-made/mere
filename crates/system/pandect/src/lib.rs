// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! # graphshell session runtime
//!
//! Portable runtime state + types Mere hosts attach to. Per the
//! multiplexer framing thesis:
//!
//! > **Mere multiplexes durable graph sessions. Engines are
//! > replaceable content producers. Hosts are attach clients.
//! > Graph truth is never browser profile state.**
//!
//! This crate is the **session** half of that picture — durable
//! session manifests, session graph/view sidecars, and service-runner
//! state. Everything here compiles wasm32-clean: no gpui, no
//! iced, no winit, no wgpu.
//!
//! Ownership split:
//!
//! - `graphshell/shell/system/control-plane` owns the action bus.
//! - this crate owns session manifests, session lifecycle storage,
//!   graph/view sidecars, and session worker declarations.
//!
//! Adjacent crates:
//!
//! - [`host-contract`](../host_ports/index.html) —
//!   the **vocabulary** half: port traits hosts satisfy
//!   (HostPaintPort, HostInputPort, HostSurfacePort, HostAccessibility
//!   Port, …) + frame-projection inputs.
//! - `graphshell-control-plane` owns the action-bus vocabulary.
//! - `host` and future host adapters consume these portable surfaces
//!   without owning their schemas.
//!
//! See:
//! - `design_docs/mere_docs/research/2026-05-11_browser_multiplexer_framing.md`
//! - `design_docs/mere_docs/implementation_strategy/2026-05-11_graph_session_manifest_plan.md`

#![doc(html_root_url = "https://docs.rs/host-runtime/0.0.1")]

// Application-scoped preferences live under the host's data root rather than
// in one session's settings sidecar.
// Portable address-access history (engine split D69, from the reference app).
pub mod access;
pub mod addressable_graph;
pub mod application_settings_store;
// Durable content-addressed store for node preview imagery (favicons, previews,
// snapshots). Eidetic owns the content-addressed bytes; this adapter owns
// ImageRef conversion and graph-aware retention decisions.
pub mod image_store;
// The family-shared root moved to catchment with the reservoir (engine split
// P4); the module path stays for its callers.
pub use catchment::shared_root;
// Per-node facet-store sidecar (facets.json): the runtime tier of the one-node
// facet system — typed per-node metadata keyed by node UUID, persisted beside
// graph.json. The durable home the bespoke per-node sidecars (browser/participant/
// arrangement) converge onto. Wraps chartulary's FacetStore.
pub mod facet_store;
#[cfg(feature = "furnace")]
pub mod furnace;
// The compare-and-replace over a muniment slot moved to catchment (engine
// split D84); the module path stays for its callers.
pub use catchment::slot_guard;
pub mod graph_codicil;
pub mod graph_placement;
// A mere's sessions over one muniment store: the session schema, the live
// session core and the lifecycle (reservoir plan V2).
pub mod graph_session;
// What the session's replay check compares, as bytes (engine split D156).
mod replay_state;
// Observed stable-ID live commands over the complete session writer.
pub mod live_graph;
// Producer-owned, content-addressed recipes for reopening a source at a
// cursor with existing durable curation. Source resolution stays at the host.
// Snapshot-level merge for codicil compose (Alembic tail B7): union two graph
// snapshots by URL identity, retaining per-member provenance. Pure; the codicil
// compose op (`graph_codicil::compose_graph_codicils`) layers on top.
#[cfg(test)]
mod resource_content_merge_tests;
pub mod snapshot_merge;
// The merge rules Compose and session merge-in share (session merge-in note, M8).
pub mod merge_rules;
// The frame.json pane-layout store moved OUT with the pane model at
// meerkat's deletion (2026-07-18): it lives in turnstone's `frisket::store`
// now — the pane-coupled half of this crate, split exactly as the
// boundary-pass plan parked it.
pub mod manifest;
pub mod memory_levels;
// The store-side memory program, now Eidetic's (engine split D45): no kernel or
// pandect types.
/// The retention passes live in Eidetic (engine split D45, D87); the path stays.
pub use eidetic::memory as retention;
// The session graph file's name and path (graph.json; GraphSession writes it);
// native-only so the crate stays wasm-clean (wasm hosts use a different storage
// backend).
#[cfg(not(target_arch = "wasm32"))]
pub mod session_graph_store;
// The chosen theme as stored values; the host converts to its theming type
// (engine split D79).
pub mod stored_theme;
// The personal graph's device synchronization over Stickleback and p2panda
// (engine split D69, from the reference app, as it was).
#[cfg(all(feature = "personal-sync", not(target_arch = "wasm32")))]
pub mod personal_sync;
// The graph-selection codicil and its checked decode, and the records a
// transfer writes (engine split D70, from the reference app). Ungated: the
// app's export and transfer build them for the browser too.
pub mod product_codicil;
// The saved scene's stored shapes: the grammar the sync lane and the codicil
// carry (D69); capture and restore are the canvas's, in mere-cartography.
pub mod cartography_geometry;
pub mod saved_scene;
pub mod transfer;
// Transfer offers over the personal graph (D70), beside the sync they ride.
#[cfg(all(feature = "personal-sync", not(target_arch = "wasm32")))]
pub mod transfer_offer;
// The tear-out payload types (PaneDragPayload/TileDragPayload) moved out
// with the pane model at meerkat's deletion: they name frisket::PaneId, so
// they live in turnstone's `frisket::tearout` now.
// A view's state as the session stores it: a schema id and an opaque value
// (engine split D44). ViewIntent itself is incipit's `view` feature (D78).
pub mod view_state;

pub use application_settings_store::{
    APPLICATION_SETTINGS_DIR, APPLICATION_SETTINGS_FILENAME, ApplicationSettings, ShellbarEdge,
    application_settings_exist, application_settings_path, load_application_settings,
    save_application_settings,
};
/// Compatibility names for published consumers. Schema content and validation
/// are implemented by Eidetic; new callers use `eidetic::schema_facets`.
pub use eidetic::schema_facets::{
    ContentClassCodicil, SchemaFacetValidator, content_class_schema_definition,
    content_class_schema_ref, load_content_class, save_content_class,
};
/// Top-level subdirectory holding per-persona state, under a data root.
pub use sporran::PERSONAS_DIR;
/// Top-level subdirectory holding per-session state. Matches the
/// layout convention of the host's manifest store.
pub const SESSIONS_DIR: &str = "sessions";
pub use facet_store::{
    AcceptAll, ExpiringFacet, FacetError, FacetId, FacetValidator, NODE_FACETS_FILE,
    NodeFacetStore, NodeFacets, copy_node_facets, load_node_facets, node_facets_path,
    read_expiring_facet, save_node_facets,
};
#[cfg(not(target_arch = "wasm32"))]
pub use graph_session::fork_component_graph;
pub use graph_session::{
    Applied, Change, ChangeKind, DEFAULT_CHECKPOINT_INTERVAL, DroppedChoice,
    GraphAdmissionValidator, GraphSession, GraphValidator, Kept, MereSessions, MergeChoice,
    MergeConflict, MergeMatch, Pending, PendingMerge, Reverted, SESSIONS_PREFIX, SessionError,
    ViewEntry, ViewKey, branched_session_name, forked_session_name, picked_session_name,
};
pub use identity::StartupUnlockMode;
// The ids and author the session schema names, so a host can name them
// through pandect.
pub use catchment::{GraphId, SessionId};
pub use kernel::graph::{Author, AuthorKind, CapturedDelta, Part};
pub use manifest::{
    CodicilId, GraphSessionManifest, LEGACY_HOST_KEYS, Lineage, MANIFEST_SCHEMA_VERSION,
    MergedMark, PersonaId, PickedFrom, TrashMark,
};
// The reservoir service is catchment's (engine split P4, D3, D9): every mere a
// persona holds, one per data domain. These names stay for cutover; access
// decisions are attributed with the journal's `Author`.
pub use catchment::{
    DEFAULT_KEPT_BYTES, DomainId, KEEP_SETTING_SCHEMA, KEEP_SETTING_SLOT, KeepPageText, Keeping,
    MAX_KEPT_BYTES, MERE_RECORD_SCHEMA, MERES_DIR, MereId, MereRecord, PlainCustody,
    RESERVOIR_DB_FILENAME, RESERVOIR_DIR, RETAINED_DB_FILENAME, ReservoirError, RetainedCustody,
    RetainedText, SealedValues, SharedSealer, mere_dir, reservoir_dir, retained_path,
};
#[cfg(not(target_arch = "wasm32"))]
pub use catchment::{
    open_mere_archive_backend, open_mere_backend, open_mere_retained_backend,
    open_reservoir_backend,
};
/// A persona's reservoir index, with access decisions attributed by [`Author`].
pub type ReservoirStore<B> = catchment::ReservoirStore<B, Author>;
/// An owner's access decision for one application and one mere, attributed by
/// [`Author`].
pub type MereApplicationAccess = catchment::MereApplicationAccess<Author>;
pub use cartography_geometry::{CartographyGeometry, SavedMosaic, SavedRegion};
pub use saved_scene::{
    SAVED_SCENE_FACET, SAVED_SCENE_FACET_V2, SAVED_SCENE_FACET_V3, SavedDynamics, SavedRolesV1,
    SavedSceneV2, SavedSceneV3,
};
pub use stored_theme::{FontStacks, StoredTheme};
pub use view_state::{UNTAGGED_VIEW_SCHEMA, ViewState};
