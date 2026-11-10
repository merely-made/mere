// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Tests for the view sidecar, moved with it from pandect (engine split D78).

use super::*;

fn temp_session_dir(label: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let dir = std::env::temp_dir().join(format!("mere-view-intent-test-{label}-{pid}-{nanos}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn fixture_frame() -> String {
    "11111111-2222-3333-4444-555555555555".to_string()
}

fn sample_intent() -> ViewIntent {
    let mut intent = ViewIntent::new();
    intent.hidden_relations.insert(HiddenRelationRecord::new(
        Uuid::from_u128(0xa1),
        Uuid::from_u128(0xa2),
        // Cites tag: Semantic family (0 << 24) + sub ordinal 3.
        3,
    ));
    intent.hidden_relations.insert(HiddenRelationRecord::new(
        Uuid::from_u128(0xb1),
        Uuid::from_u128(0xb2),
        // Hyperlink tag: family 0, sub 0.
        0,
    ));
    intent
}

#[test]
fn save_then_load_round_trips_intent() {
    let dir = temp_session_dir("round-trip");
    let frame = fixture_frame();
    let original = sample_intent();
    save_view_intent(&dir, &frame, 7, &original).unwrap();
    let restored = load_view_intent(&dir, &frame, 7)
        .unwrap()
        .expect("intent file should be present");
    assert_eq!(restored, original);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn focus_round_trips_and_counts_as_nonempty() {
    let dir = temp_session_dir("focus");
    let frame = fixture_frame();
    let mut intent = ViewIntent::new();
    intent.focus = Some("https://example.com/page".to_string());
    assert!(!intent.is_empty(), "a focus alone is worth persisting");
    save_view_intent(&dir, &frame, 1, &intent).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert_eq!(restored.focus.as_deref(), Some("https://example.com/page"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn link_hide_round_trips_and_counts_as_nonempty() {
    let dir = temp_session_dir("link-hide-round-trip");
    let frame = fixture_frame();
    let mut intent = ViewIntent::new();
    let key = LinkKey::new(sceno::LinkStratum::Surface, "surface:two", "surface:one");
    intent.hidden_links.insert(key.clone());
    intent.hidden_links.insert(LinkKey::new(
        sceno::LinkStratum::Resource,
        "surface:one",
        "surface:two",
    ));
    assert!(!intent.is_empty(), "a link hide alone is worth persisting");
    save_view_intent(&dir, &frame, 1, &intent).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert_eq!(restored.hidden_links.len(), 2);
    assert!(restored.hidden_links.contains(&key));
    assert!(restored.hidden_relations.is_empty());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn exact_assertion_hides_round_trip_independently_and_compose_with_other_hides() {
    let dir = temp_session_dir("assertion-hide-round-trip");
    let frame = fixture_frame();
    let mut intent = ViewIntent::new();
    intent.hidden_relations.insert(HiddenRelationRecord::new(
        Uuid::from_u128(1),
        Uuid::from_u128(2),
        3,
    ));
    intent.hidden_links.insert(LinkKey::new(
        sceno::LinkStratum::Surface,
        "surface:a",
        "surface:b",
    ));
    intent.hidden_assertions.insert("urn:assertion:one".into());
    assert!(!intent.is_empty());
    save_view_intent(&dir, &frame, 1, &intent).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert_eq!(restored, intent);
    assert_eq!(
        restored.hidden_assertions,
        BTreeSet::from(["urn:assertion:one".into()])
    );
    assert_eq!(restored.hidden_links.len(), 1);
    assert_eq!(restored.hidden_relations.len(), 1);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn legacy_view_without_link_hides_loads_empty_link_set() {
    let dir = temp_session_dir("legacy-without-link-hides");
    let frame = fixture_frame();
    let path = view_intent_path(&dir, &frame, 1);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(
        &path,
        r#"{"hidden_relations":[{"from_id":"00000000-0000-0000-0000-000000000001","to_id":"00000000-0000-0000-0000-000000000002","relation_tag":3}]}"#,
    )
    .unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert!(restored.hidden_links.is_empty());
    assert!(restored.hidden_assertions.is_empty());
    assert_eq!(restored.hidden_relations.len(), 1);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn folds_round_trip_as_durable_view_state() {
    let dir = temp_session_dir("folds");
    let frame = fixture_frame();
    let mut intent = ViewIntent::new();
    intent.folds.push(
        FoldRecord::selection(
            crate::curation::FoldId::new(Uuid::from_u128(1).to_string()),
            "session:alpha/graph:current",
            [Uuid::from_u128(7), Uuid::from_u128(3)].map(|id| id.to_string()),
        )
        .expect("two members form a fold"),
    );
    assert!(!intent.is_empty(), "a fold alone is worth persisting");
    save_view_intent(&dir, &frame, 1, &intent).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert_eq!(restored.folds, intent.folds);
    fs::remove_dir_all(&dir).ok();
}

/// A sidecar saved with forme's version 1 fold loads with the fold
/// migrated (Ruling 164): the same id and members, a summary stand-in,
/// no rule and no label. The next save writes version 2.
#[test]
fn a_saved_version_one_fold_migrates_on_load() {
    let dir = temp_session_dir("fold-v1");
    let frame = fixture_frame();
    let path = view_intent_path(&dir, &frame, 3);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    let (fold, a, b) = (
        Uuid::from_u128(0xf0),
        Uuid::from_u128(1),
        Uuid::from_u128(2),
    );
    fs::write(
        &path,
        format!(
            r#"{{"hidden_relations":[],"folds":[{{"version":1,"id":"{fold}","source_scope":"session:alpha","members":["{a}","{b}"],"boundary_policy":"BundleByRelationFamily"}}]}}"#
        ),
    )
    .unwrap();
    let restored = load_view_intent(&dir, &frame, 3).unwrap().unwrap();
    let migrated = &restored.folds[0];
    assert_eq!(migrated.version, crate::curation::FOLD_RECORD_VERSION);
    assert_eq!(migrated.id.as_str(), fold.to_string());
    assert_eq!(migrated.members, vec![a.to_string(), b.to_string()]);
    assert_eq!(
        migrated.stand_in,
        crate::curation::FoldStandIn::Summary { label: None }
    );
    assert_eq!((&migrated.rule, &migrated.label), (&None, &None));
    save_view_intent(&dir, &frame, 3, &restored).unwrap();
    let written = fs::read_to_string(&path).unwrap();
    assert!(written.contains(r#""version": 2"#), "{written}");
    assert_eq!(
        load_view_intent(&dir, &frame, 3).unwrap().unwrap(),
        restored
    );
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn strategy_round_trips_and_counts_as_nonempty() {
    let dir = temp_session_dir("strategy");
    let frame = fixture_frame();
    let mut intent = ViewIntent::new();
    intent.strategy = Some("phyllotaxis.default".to_string());
    assert!(
        !intent.is_empty(),
        "a layout strategy alone is worth persisting"
    );
    save_view_intent(&dir, &frame, 1, &intent).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert_eq!(restored.strategy.as_deref(), Some("phyllotaxis.default"));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn mirror_tiles_round_trip_and_count_as_nonempty() {
    let dir = temp_session_dir("mirror-tiles");
    let frame = fixture_frame();
    let mut intent = ViewIntent::new();
    intent.mirror_tiles = true;
    assert!(
        !intent.is_empty(),
        "mirror mode alone is durable pane scope"
    );
    save_view_intent(&dir, &frame, 1, &intent).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert!(restored.mirror_tiles);
    assert_eq!(restored, intent);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn load_returns_none_when_no_file() {
    let dir = temp_session_dir("no-file");
    let frame = fixture_frame();
    let result = load_view_intent(&dir, &frame, 42).unwrap();
    assert!(result.is_none());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn exists_reflects_save_state() {
    let dir = temp_session_dir("exists");
    let frame = fixture_frame();
    assert!(!view_intent_exists(&dir, &frame, 1));
    save_view_intent(&dir, &frame, 1, &sample_intent()).unwrap();
    assert!(view_intent_exists(&dir, &frame, 1));
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_overwrites_existing_intent_atomically() {
    let dir = temp_session_dir("overwrite");
    let frame = fixture_frame();
    save_view_intent(&dir, &frame, 1, &sample_intent()).unwrap();
    let mut updated = ViewIntent::new();
    updated.hidden_relations.insert(HiddenRelationRecord::new(
        Uuid::from_u128(0xc1),
        Uuid::from_u128(0xc2),
        0,
    ));
    save_view_intent(&dir, &frame, 1, &updated).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    assert_eq!(restored.hidden_relations.len(), 1);
    assert!(
        restored
            .hidden_relations
            .iter()
            .any(|r| r.from_id == Uuid::from_u128(0xc1))
    );
    // No leftover .tmp file from the atomic write.
    let tmp = view_intent_path(&dir, &frame, 1).with_extension("json.tmp");
    assert!(!tmp.exists());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn empty_intent_round_trips_identically() {
    let dir = temp_session_dir("empty");
    let frame = fixture_frame();
    let original = ViewIntent::new();
    save_view_intent(&dir, &frame, 9, &original).unwrap();
    let restored = load_view_intent(&dir, &frame, 9).unwrap().unwrap();
    assert!(restored.is_empty());
    assert_eq!(restored, ViewIntent::default());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn malformed_json_returns_invalid_data_error() {
    let dir = temp_session_dir("malformed");
    let frame = fixture_frame();
    let path = view_intent_path(&dir, &frame, 5);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, "{ not json at all").unwrap();
    match load_view_intent(&dir, &frame, 5) {
        Err(e) => assert_eq!(e.kind(), io::ErrorKind::InvalidData),
        Ok(_) => panic!("expected malformed JSON to fail parsing"),
    }
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn save_creates_views_and_frame_subdirectories() {
    let dir = temp_session_dir("dir-create");
    let frame = fixture_frame();
    assert!(!dir.join(VIEW_INTENT_DIR).exists());
    save_view_intent(&dir, &frame, 1, &sample_intent()).unwrap();
    assert!(dir.join(VIEW_INTENT_DIR).join(&frame).exists());
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn intent_path_composition_matches_spec() {
    let dir = PathBuf::from("/tmp/mere-test");
    let path = view_intent_path(&dir, "frame-abc", 42);
    let expected = dir.join("views").join("frame-abc").join("42.json");
    assert_eq!(path, expected);
}

#[test]
fn relation_tag_round_trips_through_disk() {
    // Cites tag in the file → RelationKind::Cites in memory.
    // Pins the contract: relation_tag is the
    // `RelationKind::tag()` ordinal and survives the round-trip
    // intact. If the tag ordinal scheme ever shifts, this test
    // breaks and forces an explicit decision on migration.
    let dir = temp_session_dir("tag-round-trip");
    let frame = fixture_frame();
    let mut original = ViewIntent::new();
    let from = Uuid::from_u128(1);
    let to = Uuid::from_u128(2);
    let cites_tag =
        kernel::graph::RelationKind::Semantic(kernel::graph::SemanticSubKind::Cites).tag();
    original
        .hidden_relations
        .insert(HiddenRelationRecord::new(from, to, cites_tag));
    save_view_intent(&dir, &frame, 1, &original).unwrap();
    let restored = load_view_intent(&dir, &frame, 1).unwrap().unwrap();
    let record = restored.hidden_relations.iter().next().unwrap();
    assert_eq!(
        kernel::graph::RelationKind::from_tag(record.relation_tag),
        Some(kernel::graph::RelationKind::Semantic(
            kernel::graph::SemanticSubKind::Cites,
        )),
    );
    fs::remove_dir_all(&dir).ok();
}
