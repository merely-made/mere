// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
#![cfg(not(target_arch = "wasm32"))]

//! A component fork carries every facet of the donor container to its own
//! container, naming no namespace (engine split D134). The donor store in
//! `tests/fixtures/pre_p4_container_facets_store` was written by the code
//! before the change: its container holds scene.*, arrangement.regions and
//! arrangement.mosaic facets, the last two naming nodes outside the
//! component that is forked.

use std::fs;
use std::path::Path;

use kernel::graph::Graph;
use muniment::{DirectoryBackend, MemoryBackend};
use pandect::{Author, CapturedDelta, FacetId, MereSessions, SessionId};
use uuid::Uuid;

fn author() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn add(n: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: id(n).to_string(),
        url: format!("https://{n}.test/"),
        position: [0.0, 0.0],
    }
}

#[test]
fn a_store_written_before_the_change_forks_its_container_facets_whole() {
    let dir = tempfile::tempdir().unwrap();
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pre_p4_container_facets_store");
    copy_tree(&fixture, dir.path());

    pollster::block_on(async {
        let mere = MereSessions::new(DirectoryBackend::open(dir.path()).unwrap());
        let donor = mere.open(SessionId::from_uuid(id(0x5E55))).await.unwrap();
        let donor_container = *donor.manifest().root_graph_id.as_uuid();
        assert_eq!(donor_container, id(0xC0));
        let donor_facet = |name: &str| {
            donor
                .graph()
                .facets()
                .get(&donor_container, &FacetId::new(name))
                .cloned()
                .unwrap_or_else(|| panic!("the fixture container holds {name}"))
        };

        let fork = mere.fork_component(&donor, id(1), author()).await.unwrap();
        // Reopen: the fork was written, and is read back from the store.
        let fork_session = mere.open(fork.session_id).await.unwrap();
        let fork_container = *fork_session.manifest().root_graph_id.as_uuid();
        assert_ne!(fork_container, donor_container);
        let graph = fork_session.graph();
        let facets = graph.facets();

        for name in [
            "scene.size_by_degree",
            "scene.size_by_importance",
            "scene.importance_metric",
            "scene.physics_damping",
            "arrangement.regions",
            "arrangement.mosaic",
        ] {
            assert_eq!(
                facets.get(&fork_container, &FacetId::new(name)),
                Some(&donor_facet(name)),
                "{name} reaches the fork container verbatim"
            );
        }
        assert_eq!(
            facets.get(&fork_container, &FacetId::new("scene.physics_damping")),
            Some(&serde_json::json!(6.25))
        );
        assert!(
            facets.facets_of(&donor_container).is_none(),
            "the fork keys them by its own container, not the donor one"
        );

        // The regions and mosaic name nodes 3 and 99, outside the component:
        // the fork holds neither, so they do not appear, and the fork still
        // projects.
        assert_eq!(graph.node_count(), 2);
        for outside in [id(3), id(99)] {
            assert!(graph.get_node_key_by_id(outside).is_none());
            assert!(facets.facets_of(&outside).is_none());
        }
        let rebuilt = Graph::try_from_snapshot(&graph.to_snapshot()).expect("the fork projects");
        assert_eq!(rebuilt.node_count(), 2);
    });
}

#[test]
fn a_donor_with_no_container_facets_leaves_the_fork_with_none() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let manifest = mere.mint(author(), None).await.unwrap();
        let mut donor = mere.open(manifest.session_id).await.unwrap();
        donor.apply(author(), vec![add(1), add(2)]).await.unwrap();
        let fork = mere.fork_component(&donor, id(1), author()).await.unwrap();
        let fork = mere.open(fork.session_id).await.unwrap();
        let container = *fork.manifest().root_graph_id.as_uuid();
        assert!(
            fork.graph().facets().facets_of(&container).is_none(),
            "nothing is written, so a scene read gives the defaults it gave before"
        );
    });
}
