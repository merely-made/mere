// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn live_carrier_accepts_required_cards_and_unbound_optional_items() {
    for (target, adapter) in [
        (MERE_SESSIONS, crate::wire::SESSIONS_SOURCE),
        (MERE_SESSIONS, crate::wire::MERGE_SOURCE),
        (crate::wire::MERE_ARCHIVE, crate::wire::ARCHIVE_SOURCE),
    ] {
        let panel = mount_required_card_route(target, adapter, true, true);
        assert!(!panel.closed, "{} / {} should be healthy", target, adapter);
        assert!(matches!(panel.model.status, ViewStatus::Ready));
        match adapter {
            crate::wire::SESSIONS_SOURCE => assert!(
                panel
                    .model
                    .sessions
                    .iter()
                    .any(|entry| entry.name == format!("title-{adapter}"))
            ),
            crate::wire::MERGE_SOURCE => assert!(panel.merge.is_some()),
            crate::wire::ARCHIVE_SOURCE => {
                assert!(panel.model.archive.as_ref().is_some_and(|archive| {
                    archive.entries.iter().any(|entry| {
                        entry.key == "required-entry" && entry.name == format!("title-{adapter}")
                    })
                }))
            },
            _ => unreachable!(),
        }
    }

    let mut panel = panel();
    let mut optional = empty_snapshot(MERE_SESSIONS);
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new("optional-unrelated", "optional"));
    scene.items.push(ProjectedItem {
        source,
        space: Scene::WORLD,
        transform: Transform2::translation(0.0, 0.0),
        footprint: Footprint::Circle { radius: 8.0 },
        representation: Representation::Card,
        layer: 0,
        visible: true,
        hit: None,
        channels: vec![],
    });
    optional.scene = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap();
    seed_partial_state(&mut panel);
    // Empty graph and archive follow the normal carrier request sequence.
    for _ in 0..8 {
        if panel.closed || panel.outbox.is_empty() {
            break;
        }
        let outgoing = panel.take_outgoing();
        for line in outgoing {
            let request: CarrierRequest = serde_json::from_str(&line).unwrap();
            let name = match &request.body {
                chirograph::CarrierRequestBody::Snapshot(snapshot) => snapshot.session.0.as_str(),
                _ => panic!("unexpected request"),
            };
            let snapshot = if name == MERE_SESSIONS {
                optional.clone()
            } else {
                empty_snapshot(name)
            };
            let response = chirograph::line::encode(&CarrierOutput::Response(CarrierResponse {
                id: request.id,
                body: Ok(CarrierResponseBody::Snapshot(Box::new(snapshot))),
            }))
            .unwrap();
            panel.on_line(&response);
        }
    }
    assert!(
        !panel.closed,
        "unbound optional active item remains allowed"
    );
    assert!(matches!(panel.model.status, ViewStatus::Ready));
}

#[test]
fn carrier_admits_zero_or_one_pending_merge_with_first_media() {
    let pending_bytes = pending_merge_bytes();
    let pending_hash = ContentHash::of(&pending_bytes);

    let zero = mount_empty_route_with_merges(&[], &pending_bytes);
    assert!(!zero.closed, "no pending merge should be healthy");
    assert!(matches!(zero.model.status, ViewStatus::Ready));
    assert!(zero.merge.is_none());

    let ignored_second = ContentHash::of(b"ignored second merge media");
    let one = mount_empty_route_with_merges(&[vec![pending_hash, ignored_second]], &pending_bytes);
    assert!(
        !one.closed,
        "one pending merge with first media should be healthy"
    );
    assert!(matches!(one.model.status, ViewStatus::Ready));
    assert_eq!(one.merge, serde_json::from_slice(&pending_bytes).ok());
}

#[test]
fn carrier_rejects_pending_merge_without_media_and_purges_partial_state() {
    let pending_bytes = pending_merge_bytes();
    let missing = mount_empty_route_with_merges(&[vec![]], &pending_bytes);
    assert!(
        missing.closed,
        "a pending merge without media must be rejected"
    );
    assert!(
        matches!(&missing.model.status, ViewStatus::Unavailable(note) if note.message.contains("media resource"))
    );
    assert!(missing.cards.is_empty());
    assert!(missing.records.is_empty());
    assert!(missing.inspected.is_none());
    assert!(missing.transient_layout.is_none());
    assert!(missing.merge_choices.is_empty());
    assert!(missing.merge.is_none());
    assert!(missing.material.is_none());
}

#[test]
fn carrier_rejects_multiple_pending_merge_cards_and_purges_partial_state() {
    let pending_bytes = pending_merge_bytes();
    let pending_hash = ContentHash::of(&pending_bytes);
    let multiple =
        mount_empty_route_with_merges(&[vec![pending_hash], vec![pending_hash]], &pending_bytes);
    assert!(
        multiple.closed,
        "multiple pending merge cards must be rejected"
    );
    assert!(
        matches!(&multiple.model.status, ViewStatus::Unavailable(note) if note.message.contains("Multiple pending merge cards"))
    );
    assert!(multiple.cards.is_empty());
    assert!(multiple.records.is_empty());
    assert!(multiple.inspected.is_none());
    assert!(multiple.transient_layout.is_none());
    assert!(multiple.merge_choices.is_empty());
    assert!(multiple.merge.is_none());
    assert!(multiple.material.is_none());
}

#[test]
fn live_route_validates_session_media_before_fetching_it() {
    for count in [0, 1, 2] {
        let mut panel = panel();
        seed_partial_state(&mut panel);
        let empty = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new()).unwrap();
        send_response(
            &mut panel,
            CarrierResponseBody::Snapshot(Box::new(ProjectionSnapshot {
                version: ProtocolVersion::V1,
                session: ProjectionSession(MERE_SESSIONS.into()),
                scene: empty,
                presentation: PresentationManifest::default(),
                cache_policy: Default::default(),
            })),
        );

        let first_media = ContentHash::of(b"scoped graph envelope");
        let media = match count {
            0 => vec![],
            1 => vec![first_media],
            _ => vec![first_media, ContentHash::of(b"unexpected second resource")],
        };
        let card_bytes = serde_json::to_vec(&chirograph::PortableCardV1 {
            title: "attached session".into(),
            values: vec![],
            badges: vec![],
            media,
        })
        .unwrap();
        let graph = graph_snapshot(&card_bytes);
        send_response(&mut panel, CarrierResponseBody::Snapshot(Box::new(graph)));
        send_response(
            &mut panel,
            CarrierResponseBody::Resource(chirograph::ResourceResponse::new(
                ProjectionSession(MERE_GRAPH.into()),
                card_bytes,
            )),
        );

        if count == 1 {
            assert!(!panel.closed);
            let outgoing = panel.take_outgoing();
            assert_eq!(outgoing.len(), 1);
            let request: CarrierRequest = serde_json::from_str(&outgoing[0]).unwrap();
            assert!(matches!(
                request.body,
                chirograph::CarrierRequestBody::Resource(resource)
                    if resource.session.0 == MERE_GRAPH && resource.resource == first_media
            ));
        } else {
            assert!(panel.closed);
            assert!(
                matches!(&panel.model.status, ViewStatus::Unavailable(note) if note.message.contains("exactly one media"))
            );
            assert!(panel.take_outgoing().is_empty());
            assert!(panel.cards.is_empty());
            assert!(panel.records.is_empty());
            assert!(panel.inspected.is_none());
            assert!(panel.transient_layout.is_none());
            assert!(panel.merge_choices.is_empty());
            assert!(panel.material.is_none());
        }
    }
}
