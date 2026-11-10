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
pub mod addressable_graph;
pub mod application_settings_store;
// Durable content-addressed store for node preview imagery (favicons, previews,
// snapshots). Eidetic owns the content-addressed bytes; this adapter owns
// ImageRef conversion and graph-aware retention decisions.
pub mod image_store;
// Per-node browser-state working set (scroll / form draft / viewer override /
// compat mode / content-on / page-zoom scale) — browser-runtime state that
// doesn't belong in graph truth (boundary pass slice C). Persistence converged
// onto web.* facets (web_facets); the module's browser_nodes.json IO remains
// as legacy read-only.
pub mod browser_node_state;
// The web.* facet namespace: browser_node_state's persistence boundary — one
// atomic facet per field (web.scroll / form_draft / viewer / compat / content
// / page_scale) in facets.json, replacing the bespoke browser_nodes.json
// document.
pub mod web_facets;
// The denizen.* facet namespace: which graph nodes are participants (servitor /
// agent / peer / scenario / pack) and where each one's nested graph lives —
// a facet bundle on the node, in facets.json. Supersedes the transitional
// denizen_bindings.json sidecar (removed before any host wrote one).
pub mod denizen_facets;
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
// The arrangement.* facet namespace: cartography's per-node data (position
// first; size/sprite/hull/material/face follow) as facets in facets.json —
// born as facets, since the bespoke cartography sidecar was never wired.
pub mod arrangement_facets;
pub mod cartography_geometry;
// The compare-and-replace over a muniment slot moved to catchment (engine
// split D84); the module path stays for its callers.
pub use catchment::slot_guard;
// The scene.* facet namespace: the graph-scene's own view settings (sizing
// mode, importance metric, physics damping) as facets of the CONTAINER node
// (keyed by the session's root_graph_id) — scene-scoped, not per-node.
pub mod saved_scene;
pub mod scene_facets;
pub mod graph_codicil;
pub mod graph_placement;
// A mere's sessions over one muniment store: the session schema, the live
// session core and the lifecycle (reservoir plan V2).
pub mod graph_session;
// Observed stable-ID live commands over the complete session writer.
pub mod live_graph;
// Producer-owned, content-addressed recipes for reopening a source at a
// cursor with existing durable curation. Source resolution stays at the host.
// Snapshot-level merge for codicil compose (Alembic tail B7): union two graph
// snapshots by URL identity, retaining per-member provenance. Pure; the codicil
// compose op (`graph_codicil::compose_graph_codicils`) layers on top.
#[cfg(not(target_arch = "wasm32"))]
pub mod notochord_policy_store;
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
pub mod manifest_store;
// The session graph file's name and path (graph.json; GraphSession writes it);
// native-only so the crate stays wasm-clean (wasm hosts use a different storage
// backend).
#[cfg(not(target_arch = "wasm32"))]
pub mod session_graph_store;
// Session/dataspace settings sidecar (settings.json). A flat JSON document beside
// graph.json; the host loads it on launch and saves on change. Application and
// device preferences have their own stores and must not travel with it.
pub mod settings_store;
// The chosen theme as stored values; the host converts to its theming type
// (engine split D79).
pub mod stored_theme;
// DocumentScript origin->component bindings sidecar (script-bindings.json): the
// auto-attach list (§11.4 follow-on #2). Native-only (filesystem).
#[cfg(not(target_arch = "wasm32"))]
pub mod script_bindings_store;
// Per-persona UI settings (`personas/<id>/settings/ui.json`) — persona-scoped config
// distinct from the session/dataspace settings_store and the application/device
// stores; first field is the configurable menu.
pub mod persona_settings_store;
pub mod switcher_thumbnail;
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
pub use arrangement_facets::{
    ARRANGEMENT_FACE, ARRANGEMENT_MATERIAL, ARRANGEMENT_MOSAIC, ARRANGEMENT_POSITION,
    ARRANGEMENT_REGIONS, ARRANGEMENT_SEED, ARRANGEMENT_SIZE, ARRANGEMENT_SPRITE,
    ARRANGEMENT_SPRITE_HULL, arrangement_position_facet,
    read_arrangement_faces, read_arrangement_materials, read_arrangement_mosaic,
    read_arrangement_positions, read_arrangement_regions, read_arrangement_seed,
    read_arrangement_sizes, read_arrangement_sprite_hulls, read_arrangement_sprites,
    retain_present_nodes, write_arrangement_faces, write_arrangement_materials,
    write_arrangement_mosaic, write_arrangement_positions, write_arrangement_regions,
    write_arrangement_seed, write_arrangement_sizes, write_arrangement_sprite_hulls,
    write_arrangement_sprites, clear_arrangement, read_arrangement_geometry,
};
pub use denizen_facets::{
    DENIZEN_BINDING, DenizenBinding, DenizenKind, is_denizen, read_denizen_binding,
    read_denizen_bindings, remove_denizen_binding, write_denizen_binding,
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
/// [`ManifestStore`] layout convention.
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
pub use manifest_store::{LoadFailure, LoadReport, MANIFEST_FILE, ManifestStore, TRASH_DIR};
#[cfg(not(target_arch = "wasm32"))]
pub use notochord_policy_store::{
    NOTOCHORD_POLICY_FILENAME, load_notochord_policy, notochord_policy_path, save_notochord_policy,
};
pub use persona_settings_store::{
    PERSONA_SETTINGS_DIR, PERSONA_UI_FILENAME, PersonaSettings, load_persona_settings,
    persona_settings_path, record_launch, save_persona_settings,
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
    DEFAULT_ARRANGEMENT, SAVED_SCENE_FACET, SAVED_SCENE_FACET_V2, SAVED_SCENE_FACET_V3,
    SavedDynamics, SavedRolesV1, SavedSceneV2, SavedSceneV3, StoredScene, stored_scene,
    write_saved_scene,
};
pub use scene_facets::{
    DEFAULT_PHYSICS_DAMPING, SCENE_IMPORTANCE_METRIC, SCENE_PHYSICS_DAMPING, SCENE_SIZE_BY_DEGREE,
    SCENE_SIZE_BY_IMPORTANCE, SceneFacets, clear_scene_facets, copy_scene_facets,
    has_scene_facets, read_scene_facets, write_scene_facets,
};
#[cfg(not(target_arch = "wasm32"))]
pub use script_bindings_store::{SCRIPT_BINDINGS_FILENAME, ScriptBinding};
pub use settings_store::{
    LegacySettingsMigration, PersistedSettings, SETTINGS_FILENAME, SettingsLoad,
    load_settings_with_legacy,
};
pub use stored_theme::{FontStacks, StoredTheme};
pub use switcher_thumbnail::{
    SwitcherThumbnail, SwitcherThumbnailOptions, ThumbnailEdge, ThumbnailNode,
    build_switcher_thumbnail_with,
};
pub use view_state::{UNTAGGED_VIEW_SCHEMA, ViewState};
pub use web_facets::{
    WEB_COMPAT, WEB_CONTENT, WEB_FORM_DRAFT, WEB_PAGE_SCALE, WEB_SCROLL, WEB_VIEWER,
    read_web_states, write_web_state, write_web_states,
};
