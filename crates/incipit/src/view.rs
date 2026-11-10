// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-pane view-intent sidecar — durable storage for view-local
//! state that doesn't belong in graph truth.
//!
//! v0 carries only the per-orrery `hidden_relations` set so a user's
//! hide gestures survive restarts. Per the
//! [`ViewIntent` sidecar plan](../../../../design_docs/mere_docs/implementation_strategy/2026-05-14_view_intent_sidecar_plan.md)
//! and the [framing brief §5.3](../../../../design_docs/mere_docs/research/2026-05-11_browser_multiplexer_framing.md#53-view-intent-persistence-viewintent-sidecar):
//!
//! ```text
//! <sessions_dir>/
//! └── <session_id>/
//!     ├── manifest.json            ← ManifestStore
//!     ├── graph.json               ← GraphSession
//!     └── views/
//!         └── <frame_id>/
//!             └── <pane_id>.json   ← this module
//! ```
//!
//! v0a (this module) is the I/O primitive: type definitions plus
//! `save_view_intent` / `load_view_intent` / `view_intent_exists`.
//! v0b host wiring (load on pane bootstrap, save on `HideRelation`
//! toggle) lands in `host` as a follow-up; see the plan's §4 for
//! the open questions around session-id resolution at pane scope.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::curation::FoldRecord;
use sceno::LinkKey;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Subdirectory under the session root holding per-frame /
/// per-pane view sidecars.
pub const VIEW_INTENT_DIR: &str = "views";

/// The schema a [`ViewIntent`] carries when a session stores it as an opaque
/// view state (engine split D44). Pandect reads view entries written before
/// the split, which carry no schema, as this one.
pub const VIEW_INTENT_SCHEMA: &str = "mere.view-intent/v1";

/// One entry in [`ViewIntent::hidden_relations`]. Keyed by stable
/// node UUIDs and the [`kernel::graph::RelationKind::tag`]
/// ordinal so a Hyperlink line and a Cites line on the same node
/// pair persist independently.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct HiddenRelationRecord {
    pub from_id: Uuid,
    pub to_id: Uuid,
    /// Round-tripped via `RelationKind::from_tag` on load.
    pub relation_tag: u32,
}

impl HiddenRelationRecord {
    pub fn new(from_id: Uuid, to_id: Uuid, relation_tag: u32) -> Self {
        Self {
            from_id,
            to_id,
            relation_tag,
        }
    }
}

/// 2D affine snapshot for serialisation. Stores the six kurbo
/// `Affine` coefficients verbatim — round-trips identity, pan,
/// zoom, and rotation.
///
/// Wire-shape kept compact (a single `[f64; 6]`) so the JSON
/// representation reads clearly and diffs cleanly:
///
/// ```text
/// "camera": { "coefficients": [1.0, 0.0, 0.0, 1.0, 100.0, 50.0] }
/// ```
///
/// (That value is a pure pan of (100, 50) with identity scale +
/// no rotation.)
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CameraSnapshot {
    /// kurbo `Affine::as_coeffs()` order: `[a, b, c, d, e, f]` for
    /// the affine `[[a, c, e], [b, d, f]]`.
    pub coefficients: [f64; 6],
}

impl CameraSnapshot {
    /// Identity transform — no pan, no zoom, no rotation.
    pub const IDENTITY: Self = Self {
        coefficients: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
    };

    /// True when this snapshot equals identity. The save path can
    /// skip persisting an identity camera (no user gesture worth
    /// recording).
    pub fn is_identity(&self) -> bool {
        *self == Self::IDENTITY
    }
}

impl Default for CameraSnapshot {
    fn default() -> Self {
        Self::IDENTITY
    }
}

/// Persistable per-pane view state. v0 carried `hidden_relations`
/// only; `camera` joined per the spatial-chrome IR brief §6's
/// pan/zoom-as-substrate-camera framing; `focus` (the focused node's
/// URL) joined when the host began restoring it on reload. Future fields
/// (`form_factor`, `filter`, `strategy`, `overlays`) land as their
/// producers wire up. Per the plan §2, bundling unwired
/// fields now would be the half-finished shape the user's memory
/// warns against.
///
/// `BTreeSet` over `HashSet` for deterministic JSON output —
/// dev-tier inspection wants stable diffs.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ViewIntent {
    #[serde(default)]
    pub hidden_relations: BTreeSet<HiddenRelationRecord>,
    /// Hidden logical links keyed by their stable unordered endpoint pair
    /// and source stratum. This is distinct from `hidden_relations`, which
    /// retains the older directed per-kind/cell visibility semantics.
    #[serde(default)]
    pub hidden_links: BTreeSet<LinkKey>,
    /// Hidden exact relation assertions by their stable source assertion ID.
    /// This composes with whole-link hides and legacy selector hides without
    /// widening either to a broader target.
    #[serde(default)]
    pub hidden_assertions: BTreeSet<String>,
    /// The view's folds (mer3ly site plan, Rulings 163 to 165): several,
    /// never sharing a member. Their members remain source graph members; a
    /// record only tells a projection to draw them as one stand-in. A saved
    /// version 1 record migrates as it loads (Ruling 164).
    #[serde(default)]
    pub folds: Vec<FoldRecord>,
    /// Fold records this host could not apply (another scope, a newer
    /// version, a member since gone), kept as written so a host that can
    /// apply them finds them intact after this one saves.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub retained_folds: Vec<serde_json::Value>,
    /// Substrate camera snapshot (pan + zoom). `None` when the pane
    /// has no saved camera (fresh pane → host falls back to
    /// `CameraSnapshot::IDENTITY`). Save path uses `None` for
    /// identity cameras too, since persisting an unmoved camera is
    /// wasted I/O.
    #[serde(default)]
    pub camera: Option<CameraSnapshot>,
    /// URL of the node this pane is focused on, so a reload re-opens its media.
    /// Keyed by URL (URL identity) rather than petgraph `NodeKey`, which isn't
    /// stable across save/load — the same reason `GraphSnapshot` is URL-keyed.
    /// `None` when nothing is focused.
    #[serde(default)]
    pub focus: Option<String>,
    /// The orrery pane's active layout strategy (a cartography adapter `projection_id`,
    /// e.g. `"phyllotaxis.default"`), or `None` for force-directed (seiche, the default).
    /// The host re-applies it on load; the positions themselves are recomputed, never
    /// persisted (analytic layouts re-derive from the node set). (Layout picker.)
    #[serde(default)]
    pub strategy: Option<String>,
    /// Whether the orrery pane is in live workbench-mirror mode: scoped each frame to
    /// the workbench's open tiles. Restored on load (the scope itself re-derives from
    /// the open tiles, never persisted). `false` by default. (Workbench mirror.)
    #[serde(default)]
    pub mirror_tiles: bool,
    /// A host's command menu as the person keeps it (Scenograph editor plan,
    /// SE31; moved from `command-menu` to Hest at H1). `None` for views that
    /// are not a menu; absent from older views.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commands: Option<hest::menu::CommandChoices>,
}

impl ViewIntent {
    pub fn new() -> Self {
        Self::default()
    }

    /// True when nothing in this `ViewIntent` is worth persisting.
    /// The save path can skip the file for the empty case; the load
    /// path returns `Ok(None)` and the host falls back to default.
    pub fn is_empty(&self) -> bool {
        self.hidden_relations.is_empty()
            && self.hidden_links.is_empty()
            && self.hidden_assertions.is_empty()
            && self.folds.is_empty()
            && self.retained_folds.is_empty()
            && self.camera.is_none()
            && self.focus.is_none()
            && self.strategy.is_none()
            && !self.mirror_tiles
            && self.commands.is_none()
    }
}

/// Filename for one pane's view-intent sidecar. The pane-id portion
/// is rendered as the decimal `PaneId(u64)`; the host computes the
/// containing frame directory.
pub fn pane_intent_filename(pane_id: u64) -> String {
    format!("{pane_id}.json")
}

/// Build the path `<session_dir>/views/<frame_id>/<pane_id>.json`.
/// `session_dir` is the per-session root (sibling to `manifest.json`
/// and `graph.json`). `frame_id_str` should be the `FrameId`'s
/// underlying UUID string (lowercase, hyphenated). Constructing the
/// path is pure — callers can use it for existence checks before
/// loading.
pub fn view_intent_path(session_dir: &Path, frame_id_str: &str, pane_id: u64) -> PathBuf {
    session_dir
        .join(VIEW_INTENT_DIR)
        .join(frame_id_str)
        .join(pane_intent_filename(pane_id))
}

/// Serialise `intent` to JSON and replace it through the shared recoverable writer
/// to the sidecar path. Creates intermediate directories as needed
/// — sessions don't usually start with a `views/` folder.
///
/// Returns `Ok(())` even when `intent.is_empty()`; callers that want
/// to skip empty writes should check via [`ViewIntent::is_empty`]
/// before calling. Saving the empty document is harmless but creates
/// a file the loader will faithfully restore as the default value.
pub fn save_view_intent(
    session_dir: &Path,
    frame_id_str: &str,
    pane_id: u64,
    intent: &ViewIntent,
) -> io::Result<()> {
    let target = view_intent_path(session_dir, frame_id_str, pane_id);
    let json = serde_json::to_string_pretty(intent)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    muniment::atomic_file::write_bytes_with_backup(&target, json.as_bytes())
}

/// Read `<session_dir>/views/<frame_id>/<pane_id>.json` and parse it
/// as `ViewIntent`. Returns `Ok(None)` when the file doesn't exist
/// (fresh pane — the host falls back to `ViewIntent::default()`).
pub fn load_view_intent(
    session_dir: &Path,
    frame_id_str: &str,
    pane_id: u64,
) -> io::Result<Option<ViewIntent>> {
    let path = view_intent_path(session_dir, frame_id_str, pane_id);
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path)?;
    let intent: ViewIntent =
        serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(intent))
}

/// True when the sidecar exists for this pane. Helper for callers
/// that want to skip the read when nothing's there to load.
pub fn view_intent_exists(session_dir: &Path, frame_id_str: &str, pane_id: u64) -> bool {
    view_intent_path(session_dir, frame_id_str, pane_id).exists()
}

#[cfg(test)]
mod tests;
