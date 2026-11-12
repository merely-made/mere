// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Durable session manifest types — `GraphSessionManifest` is the
//! per-session record on disk that anchors session identity,
//! references its root graph + sub-graphs, and carries the host's
//! own fields in an opaque extension slot (engine split D43).
//!
//! This module ships only the **types**. Lifecycle (load-all-at-
//! startup, persist-on-mutation, archive-on-kill) lands in the
//! host's `manifest_store` (Turnstone) per the 2026-05-11 manifest plan.
//!
//! v0 fields are minimal; later plans (`SessionServiceRunner`,
//! engine profile boundary, capability gates, fork-on-divergence)
//! populate the reserved fields. Every new field uses
//! `#[serde(default)]` so pre-existing manifests load unchanged.

use std::path::PathBuf;
use std::time::SystemTime;

use catchment::{GraphId, SessionId};
pub use identity::PersonaId;
use kernel::time::wall_clock_now;
use serde::{Deserialize, Serialize};

/// Current manifest schema version. Bump on incompatible changes
/// (field renames, semantic shifts). Adding `#[serde(default)]`
/// fields does **not** require a bump.
pub const MANIFEST_SCHEMA_VERSION: u32 = 1;

/// Reference to a consolidated codicil (eidetic). v0 placeholder —
/// eidetic's surface for codicil identity will land when the
/// consolidation gesture is wired. String form keeps it
/// hand-inspectable in the meantime.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CodicilId(pub String);

/// The durable session record. One file on disk per session
/// (`<data_dir>/mere/host/sessions/<session_id>/manifest.json`).
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(remote = "Self")]
pub struct GraphSessionManifest {
    /// Schema version. Bumped on incompatible changes.
    pub schema_version: u32,
    /// Durable session identity. Stable across restarts.
    pub session_id: SessionId,
    /// The session's primary graph. v0: every session has one;
    /// `sub_graph_refs` is empty.
    pub root_graph_id: GraphId,
    /// Additional graphs referenced by this session (e.g. subgraphs
    /// promoted to first-class sub-graphs). v0 empty.
    #[serde(default)]
    pub sub_graph_refs: Vec<GraphId>,
    /// User-set display name. `None` = host derives from graph
    /// contents (first non-intro URL or short UUID).
    #[serde(default)]
    pub display_name: Option<String>,
    /// Owning persona. v0 always `PersonaId::default_persona()`.
    #[serde(default)]
    pub persona_id: PersonaId,
    /// When the session was minted.
    pub created_at: SystemTime,
    /// Most recent mutation observed by the manifest store. Used
    /// for "sort by recent" affordances + idle-consolidation
    /// policies.
    pub updated_at: SystemTime,
    /// Parent session this one was forked from, if any. Phase 3's
    /// `Fork` operation populates this; the reference is **weak**
    /// (parent is allowed to be killed without affecting this
    /// child).
    #[serde(default)]
    pub parent_session: Option<SessionId>,
    /// Codicils produced from this session by consolidation. See
    /// `design_docs/mere_docs/research/2026-05-11_memory_tiers_brief.md`.
    #[serde(default, alias = "consolidated_engrams")]
    pub consolidated_codicils: Vec<CodicilId>,
    /// Most recent consolidation timestamp. Drives "consolidate on
    /// idle" policies (when opted in). `None` = never consolidated.
    #[serde(default)]
    pub last_consolidated_at: Option<SystemTime>,
    /// The host's own fields, which pandect stores and round-trips without
    /// reading (engine split D43): worker declarations, the engine profile
    /// binding and session policy live here under keys their owners name.
    /// Files written before the split carried them as the top-level keys
    /// `active_workers`, `engine_profile` and `policy`; reading folds those
    /// in under the same names (see [`LEGACY_HOST_KEYS`]).
    #[serde(default, skip_serializing_if = "serde_json::Map::is_empty")]
    pub extension: serde_json::Map<String, serde_json::Value>,
    /// On-disk root for this session's data. Stored explicitly so
    /// the manifest is self-describing if it's ever copied out of
    /// its host directory.
    #[serde(default)]
    pub storage_path: Option<PathBuf>,
    /// For a fork, the journal cursor in [`parent_session`](Self::parent_session)
    /// it started from. The fork's journal starts empty there, over a baseline
    /// of the parent's graph at that cursor (reservoir plan V2).
    #[serde(default)]
    pub forked_at: Option<u64>,
    /// Set while the session is in the trash: who put it there, and when.
    /// Its keys stay where they are; restoring clears the mark.
    #[serde(default)]
    pub trashed: Option<TrashMark>,
    /// Immutable archive this session was opened from. Retained by edit forks.
    #[serde(default)]
    pub source_codicil: Option<CodicilId>,
    /// A thaw is browsed read-only; its first edit must create a separate fork.
    #[serde(default)]
    pub codicil_read_only: bool,
    /// For a part picked off another session (the component around one node,
    /// Turnstone's tear-out), where it was picked from. A pick is not a fork:
    /// it leaves [`parent_session`](Self::parent_session) and
    /// [`forked_at`](Self::forked_at) unset. Parts picked before this record
    /// existed carry those fork fields instead, and still load.
    #[serde(default)]
    pub picked_from: Option<PickedFrom>,
    /// What a whole-session copy is for, set when it is made (session merge-in
    /// note, M1): a [`Lineage::Fork`] goes its own way, a [`Lineage::Branch`]
    /// is meant to merge back into its [`parent_session`](Self::parent_session).
    /// Sessions made before branches existed read as forks.
    #[serde(default)]
    pub lineage: Lineage,
    /// Set once this session, a branch or a picked-off part, has been merged
    /// into another (M6). It is kept with its history intact and is read-only
    /// from then on; an edit makes a new branch from it.
    #[serde(default)]
    pub merged: Option<MergedMark>,
}

/// Whether a whole-session copy goes its own way or is meant to merge back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Lineage {
    /// A copy that goes its own way. Never merged in. Every session made
    /// before lineage was recorded reads as one.
    #[default]
    Fork,
    /// A copy at a point, kept linked to its parent, meant to merge back.
    Branch,
}

/// Where a merged session went: the session it was merged into, its own
/// journal cursor the merge brought in, and when.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergedMark {
    pub into: SessionId,
    pub through: u64,
    pub at: SystemTime,
}

/// Where a picked-off part came from: the source session, its journal cursor
/// when the part was picked, and the source graph's container that each
/// picked node's `CopiedFrom` derivation names as its `source_graph`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PickedFrom {
    pub session: SessionId,
    pub at: u64,
    pub container: uuid::Uuid,
}

/// Who put a session in the trash, and when.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrashMark {
    pub by: kernel::graph::Author,
    pub at: SystemTime,
}

impl GraphSessionManifest {
    /// Build a fresh manifest for a brand-new session. `created_at`
    /// and `updated_at` are stamped now; everything else takes its
    /// default. `persona_id` defaults to the v0 single persona.
    pub fn new(session_id: SessionId, root_graph_id: GraphId) -> Self {
        let now = wall_clock_now();
        Self {
            schema_version: MANIFEST_SCHEMA_VERSION,
            session_id,
            root_graph_id,
            sub_graph_refs: Vec::new(),
            display_name: None,
            persona_id: PersonaId::default_persona(),
            created_at: now,
            updated_at: now,
            parent_session: None,
            consolidated_codicils: Vec::new(),
            last_consolidated_at: None,
            extension: serde_json::Map::new(),
            storage_path: None,
            forked_at: None,
            trashed: None,
            source_codicil: None,
            codicil_read_only: false,
            picked_from: None,
            lineage: Lineage::Fork,
            merged: None,
        }
    }

    /// Mark this manifest dirty by advancing `updated_at` to now.
    /// Called by the manifest store on any mutation that should
    /// reset idle-consolidation timers.
    pub fn touch(&mut self) {
        self.updated_at = wall_clock_now();
    }

    /// Record a fork relationship — this session was created by
    /// forking off `parent`. v0 is a weak reference: nothing checks
    /// that the parent still exists.
    pub fn set_parent(&mut self, parent: SessionId) {
        self.parent_session = Some(parent);
    }

    /// Append a consolidation receipt. Bumps `last_consolidated_at`
    /// to now.
    pub fn record_consolidation(&mut self, codicil: CodicilId) {
        self.consolidated_codicils.push(codicil);
        self.last_consolidated_at = Some(wall_clock_now());
    }
}

/// The top-level keys the host's fields had before the extension slot
/// (engine split D43), and the value each held by default. A legacy file's
/// non-default values are folded into [`GraphSessionManifest::extension`]
/// under the same key; defaults are dropped, since the host defaults them.
pub const LEGACY_HOST_KEYS: [(&str, &str); 3] = [
    ("active_workers", "[]"),
    ("engine_profile", "\"PersonaScoped\""),
    ("policy", "{\"overrides\":[]}"),
];

impl GraphSessionManifest {
    /// A host field from the extension slot, if present and of type `T`.
    pub fn extension_get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.extension
            .get(key)
            .and_then(|value| serde_json::from_value(value.clone()).ok())
    }

    /// Store a host field in the extension slot under `key`.
    pub fn extension_set<T: Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), serde_json::Error> {
        self.extension
            .insert(key.to_owned(), serde_json::to_value(value)?);
        Ok(())
    }
}

impl Serialize for GraphSessionManifest {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        GraphSessionManifest::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for GraphSessionManifest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let mut object = serde_json::Map::<String, serde_json::Value>::deserialize(deserializer)?;
        let mut folded = serde_json::Map::new();
        for (key, default) in LEGACY_HOST_KEYS {
            let Some(value) = object.remove(key) else {
                continue;
            };
            let default: serde_json::Value =
                serde_json::from_str(default).expect("legacy default is JSON");
            if value != default && !value.is_null() {
                folded.insert(key.to_owned(), value);
            }
        }
        if !folded.is_empty() {
            let slot = object
                .entry("extension")
                .or_insert_with(|| serde_json::Value::Object(serde_json::Map::new()));
            let serde_json::Value::Object(slot) = slot else {
                return Err(D::Error::custom("manifest extension is not an object"));
            };
            for (key, value) in folded {
                slot.entry(key).or_insert(value);
            }
        }
        GraphSessionManifest::deserialize(serde_json::Value::Object(object))
            .map_err(D::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    fn fixture_session() -> SessionId {
        SessionId::from_uuid(Uuid::from_u128(0x1234_5678_90ab_cdef_1234_5678_90ab_cdef))
    }

    fn fixture_graph() -> GraphId {
        GraphId::from_uuid(Uuid::from_u128(0xabcd_ef01_2345_6789_abcd_ef01_2345_6789))
    }

    #[test]
    fn manifest_new_carries_schema_version_and_ids() {
        let m = GraphSessionManifest::new(fixture_session(), fixture_graph());
        assert_eq!(m.schema_version, MANIFEST_SCHEMA_VERSION);
        assert_eq!(m.session_id, fixture_session());
        assert_eq!(m.root_graph_id, fixture_graph());
        assert!(m.sub_graph_refs.is_empty());
        assert!(m.parent_session.is_none());
        assert!(m.consolidated_codicils.is_empty());
        assert_eq!(m.persona_id, PersonaId::default_persona());
        assert!(m.extension.is_empty());
    }

    #[test]
    fn manifest_round_trips_through_serde_json() {
        let mut m = GraphSessionManifest::new(fixture_session(), fixture_graph());
        m.display_name = Some("research-session".to_string());
        m.set_parent(SessionId::from_uuid(Uuid::from_u128(0xdeadbeef)));
        m.record_consolidation(CodicilId("codicil-hash-abc123".to_string()));
        m.extension_set("engine_profile", &"SessionScoped").unwrap();

        let json = serde_json::to_string(&m).expect("serialize");
        let restored: GraphSessionManifest = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(restored.session_id, m.session_id);
        assert_eq!(restored.root_graph_id, m.root_graph_id);
        assert_eq!(restored.display_name, m.display_name);
        assert_eq!(restored.parent_session, m.parent_session);
        assert_eq!(restored.consolidated_codicils, m.consolidated_codicils);
        assert_eq!(restored.extension, m.extension);
        assert_eq!(
            restored
                .extension_get::<String>("engine_profile")
                .as_deref(),
            Some("SessionScoped")
        );
    }

    #[test]
    fn manifest_reads_the_legacy_consolidated_engrams_field() {
        let mut manifest = GraphSessionManifest::new(fixture_session(), fixture_graph());
        manifest.record_consolidation(CodicilId("legacy-id".to_string()));
        let mut value = serde_json::to_value(&manifest).unwrap();
        let object = value.as_object_mut().unwrap();
        let codicils = object.remove("consolidated_codicils").unwrap();
        object.insert("consolidated_engrams".to_string(), codicils);

        let restored: GraphSessionManifest = serde_json::from_value(value).unwrap();
        assert_eq!(
            restored.consolidated_codicils,
            vec![CodicilId("legacy-id".to_string())]
        );
    }

    #[test]
    fn manifest_loads_minimal_legacy_shape_via_serde_default() {
        // Older manifests written before the consolidation list,
        // `last_consolidated_at`, `active_workers`, `engine_profile`,
        // `policy`, `parent_session`, `sub_graph_refs`,
        // `display_name`, `storage_path`, and `persona_id` fields
        // landed should still deserialize because every late
        // addition uses `#[serde(default)]`.
        let minimal = format!(
            r#"{{
                "schema_version": 1,
                "session_id": "{}",
                "root_graph_id": "{}",
                "created_at": {{ "secs_since_epoch": 0, "nanos_since_epoch": 0 }},
                "updated_at": {{ "secs_since_epoch": 0, "nanos_since_epoch": 0 }}
            }}"#,
            fixture_session().as_uuid(),
            fixture_graph().as_uuid(),
        );
        let restored: GraphSessionManifest =
            serde_json::from_str(&minimal).expect("legacy-shape deserialize");
        assert_eq!(restored.session_id, fixture_session());
        assert_eq!(restored.root_graph_id, fixture_graph());
        assert!(restored.sub_graph_refs.is_empty());
        assert!(restored.consolidated_codicils.is_empty());
        assert!(restored.extension.is_empty());
        assert_eq!(restored.persona_id, PersonaId::default_persona());
        assert_eq!(restored.picked_from, None);
        assert_eq!(
            restored.lineage,
            Lineage::Fork,
            "old sessions read as forks"
        );
        assert_eq!(restored.merged, None);
    }

    #[test]
    fn lineage_and_the_merged_mark_round_trip() {
        let mut manifest = GraphSessionManifest::new(fixture_session(), fixture_graph());
        manifest.lineage = Lineage::Branch;
        manifest.merged = Some(MergedMark {
            into: SessionId::from_uuid(Uuid::from_u128(5)),
            through: 4,
            at: SystemTime::UNIX_EPOCH,
        });
        let json = serde_json::to_value(&manifest).expect("serialize");
        assert_eq!(json["lineage"], "branch");
        let back: GraphSessionManifest = serde_json::from_value(json).expect("deserialize");
        assert_eq!(back.lineage, Lineage::Branch);
        assert_eq!(back.merged, manifest.merged);
    }

    #[test]
    fn a_part_picked_before_the_pick_record_still_loads_with_its_fork_fields() {
        let parent = SessionId::from_uuid(uuid::Uuid::from_u128(7));
        let legacy = format!(
            r#"{{
                "schema_version": 1,
                "session_id": "{}",
                "root_graph_id": "{}",
                "created_at": {{ "secs_since_epoch": 0, "nanos_since_epoch": 0 }},
                "updated_at": {{ "secs_since_epoch": 0, "nanos_since_epoch": 0 }},
                "parent_session": "{}",
                "forked_at": 3
            }}"#,
            fixture_session().as_uuid(),
            fixture_graph().as_uuid(),
            parent.as_uuid(),
        );
        let restored: GraphSessionManifest =
            serde_json::from_str(&legacy).expect("legacy pick deserialize");
        assert_eq!(restored.parent_session, Some(parent));
        assert_eq!(restored.forked_at, Some(3));
        assert_eq!(restored.picked_from, None);
        let mut picked = restored.clone();
        picked.picked_from = Some(PickedFrom {
            session: parent,
            at: 3,
            container: uuid::Uuid::from_u128(9),
        });
        let json = serde_json::to_string(&picked).expect("serialize");
        let back: GraphSessionManifest = serde_json::from_str(&json).expect("round trip");
        assert_eq!(back.picked_from, picked.picked_from);
    }

    #[test]
    fn legacy_host_keys_fold_into_the_extension_and_are_not_written_back() {
        // A manifest as written before D43: every host field at the top level,
        // two with values and one at its default.
        let legacy = format!(
            r#"{{
                "schema_version": 1,
                "session_id": "{}",
                "root_graph_id": "{}",
                "created_at": {{ "secs_since_epoch": 0, "nanos_since_epoch": 0 }},
                "updated_at": {{ "secs_since_epoch": 0, "nanos_since_epoch": 0 }},
                "active_workers": ["FetcherPool"],
                "engine_profile": "SessionScoped",
                "policy": {{ "overrides": [] }}
            }}"#,
            fixture_session().as_uuid(),
            fixture_graph().as_uuid(),
        );
        let restored: GraphSessionManifest = serde_json::from_str(&legacy).unwrap();
        assert_eq!(
            restored.extension_get::<Vec<String>>("active_workers"),
            Some(vec!["FetcherPool".to_string()])
        );
        assert_eq!(
            restored
                .extension_get::<String>("engine_profile")
                .as_deref(),
            Some("SessionScoped")
        );
        assert!(
            !restored.extension.contains_key("policy"),
            "a default is dropped"
        );

        let written = serde_json::to_value(&restored).unwrap();
        for (key, _) in LEGACY_HOST_KEYS {
            assert!(
                written.get(key).is_none(),
                "{key} is not written at the top level"
            );
        }
        let again: GraphSessionManifest = serde_json::from_value(written).unwrap();
        assert_eq!(again.extension, restored.extension);
    }

    #[test]
    fn a_pre_split_file_with_only_defaults_reads_with_an_empty_extension() {
        let manifest = GraphSessionManifest::new(fixture_session(), fixture_graph());
        let mut value = serde_json::to_value(&manifest).unwrap();
        let object = value.as_object_mut().unwrap();
        for (key, default) in LEGACY_HOST_KEYS {
            object.insert(key.into(), serde_json::from_str(default).unwrap());
        }
        let restored: GraphSessionManifest = serde_json::from_value(value).unwrap();
        assert!(restored.extension.is_empty());
    }

    #[test]
    fn touch_advances_updated_at() {
        let mut m = GraphSessionManifest::new(fixture_session(), fixture_graph());
        let original = m.updated_at;
        std::thread::sleep(std::time::Duration::from_millis(2));
        m.touch();
        assert!(m.updated_at > original);
    }

    #[test]
    fn record_consolidation_bumps_last_consolidated_at() {
        let mut m = GraphSessionManifest::new(fixture_session(), fixture_graph());
        assert!(m.last_consolidated_at.is_none());
        m.record_consolidation(CodicilId("e1".to_string()));
        assert!(m.last_consolidated_at.is_some());
        assert_eq!(m.consolidated_codicils.len(), 1);
    }

    #[test]
    fn default_persona_is_stable_across_calls() {
        let a = PersonaId::default_persona();
        let b = PersonaId::default_persona();
        assert_eq!(a, b);
    }
}
