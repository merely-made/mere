// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Session graph store — a session's graph persisted as hand-inspectable JSON
//! (`graph.json`).
//!
//! The graph is session truth (per the multiplexer framing), persisted through
//! its serde [`GraphSnapshot`] — the URL-stable form, since petgraph NodeIndex
//! keys are not stable across sessions. This is the "serde `graph.json` as the
//! live store" cut; a content-addressed eidetic layer (blobs / manifests /
//! codicils) sits behind it later for media + history.
//!
//! Native-only: this is filesystem persistence. wasm hosts persist through a
//! different backend (IndexedDB / OPFS), so the module is excluded from
//! wasm32 builds to keep the crate wasm-clean.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use incipit::SessionId;
use kernel::graph::Graph;
use kernel::persistence::GraphSnapshot;

use crate::engine_profile_store::SESSIONS_DIR;
use crate::graph_placement::{PlacementProfile, materialize_snapshot};

/// Snapshot input with explicit placement; absent metadata stays unqualified.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ProfiledGraphSnapshot {
    #[serde(flatten)]
    pub snapshot: GraphSnapshot,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement: Option<PlacementProfile>,
}

/// The graph sidecar file name under a session directory.
pub const GRAPH_FILE: &str = "graph.json";

/// The conventional per-session graph path
/// `<data_root>/sessions/<session_id>/graph.json`, matching the `ManifestStore`
/// and engine-profile layout. Used once manifests thread the session id; a
/// single-session host may persist a default graph at a flat path instead.
pub fn session_graph_path(data_root: &Path, session_id: SessionId) -> PathBuf {
    data_root
        .join(SESSIONS_DIR)
        .join(session_id.as_uuid().to_string())
        .join(GRAPH_FILE)
}

/// Persist `graph` to `path` as pretty JSON, creating parent directories. The
/// graph is written through its [`GraphSnapshot`] (the URL-stable persistence
/// form), so it round-trips across sessions where NodeIndex keys would not.
pub fn save(path: &Path, graph: &Graph) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let snapshot = graph.to_snapshot();
    let json = serde_json::to_string_pretty(&snapshot)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, json)
}

/// Save caller-qualified placement with its snapshot in one recoverable replacement.
pub fn save_profiled(
    path: &Path,
    graph: &Graph,
    placement: Option<PlacementProfile>,
) -> io::Result<()> {
    let input = ProfiledGraphSnapshot {
        snapshot: graph.to_snapshot(),
        placement,
    };
    let bytes = serde_json::to_vec_pretty(&input)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    crate::atomic_file::write_bytes_with_backup(path, &bytes)
}

/// Load a graph from `path`, or `Ok(None)` when the file does not exist (a fresh
/// session). A malformed file surfaces as an error so the host can decide whether
/// to fall back to a fresh graph rather than silently discard the session.
pub fn load(path: &Path) -> io::Result<Option<Graph>> {
    load_profiled_snapshot(path)?
        .map(|input| {
            materialize_snapshot(&input.snapshot, input.placement)
                .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
        })
        .transpose()
}

/// Read the persisted snapshot without converting it into a [`Graph`].
///
/// The seam a host needs when something must happen *between* deserializing
/// and materializing — specifically externalizing pre-phase-2 inline imagery,
/// since conversion keeps image references only and would drop those pixels
/// (see `GraphSnapshot::legacy_image_count`). Hosts with nothing to do in
/// between should call [`load`].
pub fn load_snapshot(path: &Path) -> io::Result<Option<GraphSnapshot>> {
    load_profiled_snapshot(path)?
        .map(|input| {
            if input.placement == Some(PlacementProfile::LegacySurfaceV1) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "legacy graph placement requires qualified replay",
                ));
            }
            Ok(input.snapshot)
        })
        .transpose()
}

/// Read placement without inferring it from columns, claims or declarations.
/// Materialization and any legacy migration remain the caller's qualified step.
pub fn load_profiled_snapshot(path: &Path) -> io::Result<Option<ProfiledGraphSnapshot>> {
    let json = match fs::read_to_string(path) {
        Ok(json) => json,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let snapshot: ProfiledGraphSnapshot =
        serde_json::from_str(&json).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(Some(snapshot))
}

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;
    use kernel::graph::fixtures::GraphFixtures;
    use uuid::Uuid;

    #[test]
    fn native_recorded_profile_preserves_held_edges_with_legacy_and_absent_controls() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("graph.json");
        let graph = crate::graph_placement::placement_test_graph();
        let snapshot = graph.to_snapshot();
        save_profiled(&path, &graph, Some(PlacementProfile::RecordedStrataV1)).unwrap();
        let loaded = load(&path).unwrap().unwrap().to_snapshot();
        assert_eq!(loaded.edges, snapshot.edges);
        assert_eq!(loaded.resource_edges, snapshot.resource_edges);
        assert_eq!(
            load_profiled_snapshot(&path).unwrap().unwrap().placement,
            Some(PlacementProfile::RecordedStrataV1)
        );
        save_profiled(&path, &graph, Some(PlacementProfile::LegacySurfaceV1)).unwrap();
        let untouched = fs::read(&path).unwrap();
        let error = match load(&path) {
            Err(error) => error,
            Ok(_) => panic!("declared legacy session was loaded without qualification"),
        };
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        assert_eq!(fs::read(&path).unwrap(), untouched);
        assert_eq!(
            load_profiled_snapshot(&path).unwrap().unwrap().placement,
            Some(PlacementProfile::LegacySurfaceV1)
        );
        save(&path, &graph).unwrap();
        assert_eq!(
            load_profiled_snapshot(&path).unwrap().unwrap().placement,
            None
        );
        let unqualified = load(&path).unwrap().unwrap().to_snapshot();
        assert!(
            unqualified.edges.len() + unqualified.resource_edges.len()
                > snapshot.edges.len() + snapshot.resource_edges.len()
        );
    }

    #[test]
    fn explicit_file_profiles_and_bare_resource_inputs_remain_distinct() {
        use kernel::persistence::PersistedResourceRecord;
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("graph.json");
        let graph = Graph::new();
        for placement in [
            None,
            Some(PlacementProfile::RecordedStrataV1),
            Some(PlacementProfile::LegacySurfaceV1),
        ] {
            save_profiled(&path, &graph, placement).unwrap();
            assert_eq!(
                load_profiled_snapshot(&path).unwrap().unwrap().placement,
                placement
            );
            if placement == Some(PlacementProfile::LegacySurfaceV1) {
                assert_eq!(
                    load_snapshot(&path).err().unwrap().kind(),
                    io::ErrorKind::InvalidData
                );
                assert_eq!(
                    load(&path).err().unwrap().kind(),
                    io::ErrorKind::InvalidData
                );
            } else {
                assert!(load_snapshot(&path).unwrap().unwrap().nodes.is_empty());
                assert!(load(&path).unwrap().is_some());
            }
        }
        let mut bare = graph.to_snapshot();
        bare.resources.push(PersistedResourceRecord {
            canonical_iri: "https://resource.test/".into(),
            facets: vec![],
        });
        fs::write(&path, serde_json::to_vec(&bare).unwrap()).unwrap();
        let input = load_profiled_snapshot(&path).unwrap().unwrap();
        assert_eq!(
            input.placement, None,
            "resources do not establish placement history"
        );
        assert_eq!(input.snapshot.resources, bare.resources);
        assert!(
            load(&path)
                .unwrap()
                .unwrap()
                .resource_nodes()
                .next()
                .is_some()
        );
        let mut invalid = serde_json::to_value(&bare).unwrap();
        invalid["placement"] = serde_json::json!("future-placement-v9");
        let bytes = serde_json::to_vec(&invalid).unwrap();
        fs::write(&path, &bytes).unwrap();
        assert_eq!(
            load_profiled_snapshot(&path).unwrap_err().kind(),
            io::ErrorKind::InvalidData
        );
        assert_eq!(fs::read(&path).unwrap(), bytes);
        save(&path, &graph).unwrap();
        assert_eq!(
            load_profiled_snapshot(&path).unwrap().unwrap().placement,
            None
        );
    }

    #[test]
    fn round_trips_a_graph_through_json() {
        let mut graph = Graph::new();
        graph.add_node("https://a.example".to_string(), Point2D::new(1.0, 2.0));
        graph.add_node("https://b.example".to_string(), Point2D::new(3.0, 4.0));

        let dir = std::env::temp_dir().join("mere_session_graph_store_roundtrip");
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join("graph.json");

        save(&path, &graph).expect("save");
        let loaded = load(&path).expect("load ok").expect("a graph is present");
        assert_eq!(
            loaded.nodes().count(),
            2,
            "both nodes survive the round trip"
        );
        assert!(
            loaded.get_node_by_url("https://a.example").is_some(),
            "the URL index rebuilds from the snapshot",
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn absent_file_loads_to_none() {
        let path = std::env::temp_dir().join("mere_session_graph_store_absent_zzz.json");
        let _ = fs::remove_file(&path);
        assert!(
            load(&path).expect("load ok").is_none(),
            "no file means a fresh session"
        );
    }

    #[test]
    fn checked_snapshot_file_load_rejects_invalid_resources_and_preserves_input() {
        use kernel::persistence::{PersistedResourceFacet, PersistedResourceRecord};

        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("graph.json");
        let mut graph = Graph::new();
        graph.add_node("https://surface.test".into(), Point2D::new(0.0, 0.0));
        let mut valid = graph.to_snapshot();
        valid.resources.push(PersistedResourceRecord {
            canonical_iri: "urn:mere:test:pandect-store".into(),
            facets: vec![PersistedResourceFacet {
                facet: "foreign.metadata".into(),
                value_json: "true".into(),
            }],
        });
        fs::write(&path, serde_json::to_vec(&valid).unwrap()).unwrap();
        assert_eq!(
            load(&path).unwrap().unwrap().to_snapshot().resources,
            valid.resources
        );
        for conflict in [false, true] {
            let mut invalid = valid.clone();
            if conflict {
                let mut record = invalid.resources[0].clone();
                record.facets[0].value_json = "false".into();
                invalid.resources.push(record);
            } else {
                invalid.resources[0].facets[0].value_json = "{".into();
            }
            let bytes = serde_json::to_vec(&invalid).unwrap();
            fs::write(&path, &bytes).unwrap();
            let error = match load(&path) {
                Err(error) => error,
                Ok(_) => panic!("invalid resource snapshot was loaded"),
            };
            assert_eq!(error.kind(), io::ErrorKind::InvalidData);
            assert_eq!(fs::read(&path).unwrap(), bytes);
            assert_eq!(
                load_snapshot(&path).unwrap().unwrap().resources,
                invalid.resources
            );
        }
        let mut legacy = serde_json::to_value(&valid).unwrap();
        for column in ["resources", "resource_edges", "shown_resources"] {
            legacy.as_object_mut().unwrap().remove(column);
        }
        fs::write(&path, serde_json::to_vec(&legacy).unwrap()).unwrap();
        let loaded = load(&path).unwrap().unwrap();
        assert_eq!(loaded.node_count(), 1);
        assert!(loaded.to_snapshot().resources.is_empty());
    }

    #[test]
    fn session_graph_path_follows_the_sessions_convention() {
        let id = SessionId(Uuid::from_u128(0x4242));
        let path = session_graph_path(Path::new("/data"), id);
        let expected = Path::new("/data")
            .join("sessions")
            .join(id.as_uuid().to_string())
            .join("graph.json");
        assert_eq!(path, expected);
    }
}
