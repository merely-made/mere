// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Declared-lens D18: a journal and a sync record the engine wrote before L7
//! changed any type reopen with identical held assertions (pair, statement
//! id, predicate IRI, scope, basis, times, withdrawal). The fixtures and how
//! they were made are in `fixtures/README.md`.

mod support;

use std::path::Path;

use muniment::DirectoryBackend;
use pandect::{MereSessions, SessionId};
use support::{HeldRow, Surface};
use uuid::Uuid;

const FIXTURE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/l7");

#[derive(serde::Deserialize)]
struct Expected {
    session_id: Uuid,
    pin_digest: String,
    journal_held: Vec<HeldRow>,
    #[cfg_attr(not(feature = "personal-sync"), allow(dead_code))]
    sync_held: Vec<HeldRow>,
}

fn expected() -> Expected {
    let text = std::fs::read(Path::new(FIXTURE).join("expected.json")).unwrap();
    serde_json::from_slice(&text).unwrap()
}

fn copy_dir(from: &Path, to: &Path) {
    for entry in std::fs::read_dir(from).unwrap() {
        let path = entry.unwrap().path();
        let target = to.join(path.file_name().unwrap());
        if path.is_dir() {
            std::fs::create_dir_all(&target).unwrap();
            copy_dir(&path, &target);
        } else {
            std::fs::copy(&path, &target).unwrap();
        }
    }
}

#[test]
fn a_journal_written_before_l7_reopens_with_identical_held_assertions() {
    pollster::block_on(async {
        let expected = expected();
        // Opening may write; the fixture stays as written.
        let dir = tempfile::tempdir().unwrap();
        copy_dir(&Path::new(FIXTURE).join("session"), dir.path());
        let sessions = MereSessions::<DirectoryBackend, Surface>::new_with(
            DirectoryBackend::open(dir.path()).unwrap(),
        );
        let session = sessions
            .open_with(SessionId::from_uuid(expected.session_id))
            .await
            .unwrap();
        let held = session.graph().retained_statements();
        assert!(held.len() >= 9, "the fixture holds every case: {held:?}");
        assert_eq!(held, expected.journal_held);
        // The pin's v2 journal digest is over each entry as it re-serializes,
        // and new writes drop the enum-named fields, so it moves once (D17).
        let pin = session.pin(session.journal().live_cursor()).unwrap();
        assert_ne!(pin.digest(), expected.pin_digest);
    });
}

#[cfg(feature = "personal-sync")]
#[test]
fn a_sync_record_written_before_l7_folds_to_identical_held_assertions() {
    use pandect::personal_sync::{PersonalFold, PersonalGraphRecord};
    use support::StatementFold;

    let bytes = std::fs::read(Path::new(FIXTURE).join("sync_record.cbor")).unwrap();
    let record: PersonalGraphRecord =
        p2panda_core::cbor::decode_cbor_strict(&bytes[..]).expect("the old record decodes");
    let mut graph = Surface::default();
    for (i, event) in record.events.iter().enumerate() {
        StatementFold
            .apply(
                &mut graph,
                "urn:l7:asserter",
                &format!("urn:l7:fallback:{i}"),
                event,
            )
            .unwrap();
    }
    let held = minted_masked(graph.retained_statements());
    assert!(held.len() >= 4, "the record holds every case: {held:?}");
    assert_eq!(held, minted_masked(expected().sync_held));
}

/// A legacy event names no statement, so the fold mints a fresh id each time;
/// the ids the record carries (`urn:`) are compared, minted ones are not.
#[cfg(feature = "personal-sync")]
fn minted_masked(rows: Vec<HeldRow>) -> Vec<HeldRow> {
    rows.into_iter()
        .map(|mut row| {
            if !row.statement_id.starts_with("urn:") {
                row.statement_id = "minted".into();
            }
            row
        })
        .collect()
}
