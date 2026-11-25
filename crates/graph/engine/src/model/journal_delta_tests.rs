// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// A host's surface delta, unknown to the engine.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Toy {
    AddSurface { id: String },
    Clear,
    Rename(String),
}

fn record() -> ResourceDelta {
    ResourceDelta::ReplaySetResourceRecordById {
        resource_id: "r".into(),
        record: None,
    }
}

fn entries() -> Vec<JournalDelta<Toy>> {
    vec![
        JournalDelta::Resource(record()),
        JournalDelta::Surface(Toy::AddSurface { id: "s".into() }),
        JournalDelta::Surface(Toy::Clear),
        JournalDelta::Surface(Toy::Rename("n".into())),
        JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds {
            from_resource_id: "a".into(),
            to_resource_id: "b".into(),
            edges: Vec::new(),
        }),
    ]
}

#[test]
fn each_half_encodes_as_its_own_enum_would() {
    for entry in entries() {
        let own = match &entry {
            JournalDelta::Resource(delta) => serde_json::to_string(delta),
            JournalDelta::Surface(delta) => serde_json::to_string(delta),
        }
        .unwrap();
        assert_eq!(serde_json::to_string(&entry).unwrap(), own);
    }
}

#[test]
fn entries_route_by_tag_and_round_trip() {
    let json = serde_json::to_string(&entries()).unwrap();
    let back: Vec<JournalDelta<Toy>> = serde_json::from_str(&json).unwrap();
    assert_eq!(back, entries());
    let attributed = Attributed {
        author: Author::user(),
        delta: JournalDelta::<Toy>::Resource(record()),
    };
    let json = serde_json::to_string(&attributed).unwrap();
    assert_eq!(
        serde_json::from_str::<Attributed<JournalDelta<Toy>>>(&json).unwrap(),
        attributed
    );
}

#[test]
fn a_resource_tag_never_reaches_the_surface() {
    // A surface type that would also accept the resource tag still loses it.
    #[derive(Debug, PartialEq, Deserialize)]
    enum Greedy {
        ReplaySetResourceRecordById { resource_id: String },
    }
    let json = r#"{"ReplaySetResourceRecordById":{"resource_id":"r","record":null}}"#;
    assert_eq!(
        serde_json::from_str::<JournalDelta<Greedy>>(json).unwrap(),
        JournalDelta::Resource(record())
    );
}

#[test]
fn an_unknown_tag_is_refused_by_the_surface() {
    assert!(serde_json::from_str::<JournalDelta<Toy>>(r#"{"Unknown":{}}"#).is_err());
    // Control: a known surface tag in the same shape decodes.
    assert!(serde_json::from_str::<JournalDelta<Toy>>(r#"{"AddSurface":{"id":"s"}}"#).is_ok());
}

#[test]
fn positional_tags_are_refused() {
    let bytes = postcard::to_allocvec(&JournalDelta::<Toy>::Surface(Toy::Clear)).unwrap();
    assert!(postcard::from_bytes::<JournalDelta<Toy>>(&bytes).is_err());
    // Control: the same bytes decode as the surface enum itself.
    assert_eq!(postcard::from_bytes::<Toy>(&bytes).unwrap(), Toy::Clear);
}
