// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn rejects_every_envelope_binding_mismatch() {
    for field in ["schema", "application", "session", "epoch", "revision"] {
        let mut bad = envelope();
        match field {
            "schema" => bad.schema = "unknown".into(),
            "application" => bad.application = "another".into(),
            "session" => bad.session = pandect::SessionId(uuid::Uuid::from_u128(8)),
            "epoch" => bad.epoch = SceneEpoch(2),
            _ => bad.revision = Revision(2),
        }
        let (mut fake, _) = fixture(&bad);
        assert!(
            read_complete(&mut fake)
                .unwrap_err()
                .contains("acknowledged position"),
            "{field}"
        );
    }
}

#[test]
fn rejects_missing_and_duplicate_projection_advertisements() {
    for name in [wire::MERE_SESSIONS, wire::MERE_GRAPH] {
        let (mut fake, _) = fixture(&envelope());
        fake.descriptor
            .projections
            .retain(|p| p.request.session.0 != name);
        assert!(
            read_complete(&mut fake)
                .unwrap_err()
                .contains("Missing projection")
        );
    }
    for index in 0..3 {
        let (mut fake, _) = fixture(&envelope());
        fake.descriptor
            .projections
            .push(fake.descriptor.projections[index].clone());
        assert!(
            read_complete(&mut fake)
                .unwrap_err()
                .contains("Duplicate projection")
        );
        assert!(fake.fetched.is_empty());
    }
}

#[test]
fn rejects_wrong_snapshot_session() {
    for name in [wire::MERE_SESSIONS, wire::MERE_GRAPH, wire::MERE_ARCHIVE] {
        let (mut fake, _) = fixture(&envelope());
        fake.snapshots.get_mut(name).unwrap().session = ProjectionSession("wrong".into());
        assert!(
            read_complete(&mut fake)
                .unwrap_err()
                .contains("another projection session")
        );
    }
}

#[test]
fn rejects_missing_or_unexpected_session_media() {
    for media in [
        vec![],
        vec![ContentHash::of(b"one"), ContentHash::of(b"two")],
    ] {
        let (mut fake, _) = fixture(&envelope());
        let graph = snapshot_fixture(
            wire::MERE_GRAPH,
            &[(
                wire::SESSION_ITEM_SOURCE,
                envelope().session.as_uuid().to_string(),
                media,
            )],
            &mut fake.resources,
        );
        fake.snapshots.insert(wire::MERE_GRAPH.into(), graph);
        assert!(
            read_complete(&mut fake)
                .unwrap_err()
                .contains("exactly one media")
        );
    }
}

#[test]
fn rejects_hash_valid_cards_with_wrong_codec_or_capability_at_media_sites() {
    for merge in [false, true] {
        for codec in [false, true] {
            let (mut fake, _) = fixture(&envelope());
            let name = if merge {
                add_merge(&mut fake, wire::PENDING_MERGE_SCHEMA, 1);
                wire::MERE_SESSIONS
            } else {
                wire::MERE_GRAPH
            };
            let offer = fake
                .snapshots
                .get_mut(name)
                .unwrap()
                .presentation
                .offers
                .values_mut()
                .next()
                .unwrap()
                .first_mut()
                .unwrap();
            if codec {
                offer.codec = PresentationCodec::NativeGlyphV1;
            } else {
                offer.requires = PresentationCapability::NativeGlyph;
            }
            assert!(
                read_complete(&mut fake)
                    .unwrap_err()
                    .contains("codec and capability")
            );
        }
    }
}

#[test]
fn deserialized_dangling_item_source_is_refused_before_enumeration() {
    for mode in [GraphRead::ProjectionOnly, GraphRead::Complete] {
        for name in [wire::MERE_SESSIONS, wire::MERE_GRAPH, wire::MERE_ARCHIVE] {
            let (mut fake, _) = fixture(&envelope());
            let snapshot = snapshot_fixture(
                name,
                &[(wire::SESSIONS_SOURCE, "invalid".into(), vec![])],
                &mut fake.resources,
            );
            let mut serialized = serde_json::to_value(snapshot).unwrap();
            serialized["scene"]["tables"]["items"][0]["source"] = serde_json::json!(999999);
            let invalid: ProjectionSnapshot = serde_json::from_value(serialized).unwrap();
            fake.snapshots.insert(name.into(), invalid);
            let error = block_on(read(&mut fake, "test", mode)).unwrap_err();
            assert!(
                error.contains("Invalid projection scene"),
                "{name}: {error}"
            );
            assert!(fake.fetched.is_empty());
        }
    }
}
