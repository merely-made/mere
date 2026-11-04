// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use chirograph::{
    BoundsRelationship, CarrierOutput, CarrierRequest, CarrierResponse, CarrierResponseBody,
    PresentationBinding, PresentationCapability, PresentationCodec, PresentationKey,
    PresentationManifest, PresentationOffer, PresentationSemantics, ProjectionOffer,
    ProjectionRequest, SemanticRole,
};
use sceno::{
    Arrangement, Footprint, ProjectedItem, Representation, Scene, Score, SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};

fn panel() -> MereRouteSession {
    panel_with_archive(false)
}

fn panel_with_archive(include_archive: bool) -> MereRouteSession {
    let names: Vec<_> = if include_archive {
        vec![MERE_SESSIONS, MERE_GRAPH, crate::wire::MERE_ARCHIVE]
    } else {
        vec![MERE_SESSIONS, MERE_GRAPH]
    };
    let descriptor = EndpointDescriptor {
        label: "Resident mere".into(),
        projections: names
            .into_iter()
            .map(|name| ProjectionOffer {
                label: name.into(),
                request: ProjectionRequest {
                    version: ProtocolVersion::V1,
                    session: ProjectionSession(name.into()),
                    score: Score::new(Arrangement::Spiral(Default::default())),
                },
            })
            .collect(),
    };
    MereRouteSession::from_descriptor(
        descriptor,
        CapabilityProfile::new([PresentationCapability::PortableCard]),
        "graphshell",
    )
    .unwrap()
}

fn closed_panel() -> MereRouteSession {
    let mut panel = panel();
    panel.fail("The resident route closed.".into());
    assert!(!panel.in_flight());
    assert_eq!(panel.status(), "unavailable");
    panel
}

fn seed_partial_state(panel: &mut MereRouteSession) {
    let hash = ContentHash::of(b"partial card");
    panel.cards.insert(hash, b"partial card".to_vec());
    panel.records.insert(
        "partial".into(),
        chirograph::PortableCardV1 {
            title: "partial".into(),
            values: vec![],
            badges: vec![],
            media: vec![],
        },
    );
    panel.inspected = Some("partial".into());
    panel.transient_layout = Some("grid.default".into());
    panel
        .merge_choices
        .insert("partial".into(), MergeOption::Source);
}

fn send_response(panel: &mut MereRouteSession, body: CarrierResponseBody) {
    let outgoing = panel.take_outgoing();
    assert_eq!(outgoing.len(), 1);
    let request: CarrierRequest = serde_json::from_str(&outgoing[0]).unwrap();
    let line = chirograph::line::encode(&CarrierOutput::Response(CarrierResponse {
        id: request.id,
        body: Ok(body),
    }))
    .unwrap();
    panel.on_line(&line);
}

fn graph_snapshot(card_bytes: &[u8]) -> ProjectionSnapshot {
    let session_id = uuid::Uuid::from_u128(9).to_string();
    let mut scene = Scene::new();
    let source = scene.intern_source(SourceRef::new(
        crate::wire::SESSION_ITEM_SOURCE,
        &session_id,
    ));
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
    let key = PresentationKey("session-card".into());
    let card_hash = ContentHash::of(card_bytes);
    let presentation = PresentationManifest {
        bindings: vec![PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        }],
        offers: [(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: card_hash,
                byte_size: card_bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "session".into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: vec![],
                },
            }],
        )]
        .into_iter()
        .collect(),
    };
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession(MERE_GRAPH.into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
        presentation,
        cache_policy: Default::default(),
    }
}

fn sessions_snapshot(merge_cards: &[Vec<ContentHash>]) -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let mut bindings = Vec::new();
    let mut offers = BTreeMap::new();
    for (index, media) in merge_cards.iter().enumerate() {
        let source = scene.intern_source(SourceRef::new(
            crate::wire::MERGE_SOURCE,
            &format!("pending-{index}"),
        ));
        scene.items.push(ProjectedItem {
            source,
            space: Scene::WORLD,
            transform: Transform2::translation(index as f32 * 16.0, 0.0),
            footprint: Footprint::Circle { radius: 8.0 },
            representation: Representation::Card,
            layer: 0,
            visible: true,
            hit: None,
            channels: vec![],
        });
        let key = PresentationKey(format!("merge-card-{index}"));
        let card = chirograph::PortableCardV1 {
            title: format!("pending merge {index}"),
            values: vec![],
            badges: vec![],
            media: media.clone(),
        };
        let bytes = serde_json::to_vec(&card).unwrap();
        let hash = ContentHash::of(&bytes);
        bindings.push(PresentationBinding {
            instance: InstanceId(index as u32),
            key: key.clone(),
        });
        offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: hash,
                byte_size: bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "pending merge".into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: vec![],
                },
            }],
        );
    }
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession(MERE_SESSIONS.into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
        presentation: PresentationManifest { bindings, offers },
        cache_policy: Default::default(),
    }
}

fn empty_snapshot(name: &str) -> ProjectionSnapshot {
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession(name.into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new()).unwrap(),
        presentation: PresentationManifest::default(),
        cache_policy: Default::default(),
    }
}

fn required_card_snapshot(
    session: &str,
    adapter: &str,
    with_binding: bool,
    with_offer: bool,
) -> (ProjectionSnapshot, HashMap<ContentHash, Vec<u8>>) {
    let mut scene = Scene::new();
    let id = if adapter == crate::wire::SESSIONS_SOURCE {
        "00000000-0000-0000-0000-000000000011"
    } else {
        "required-entry"
    };
    let source = scene.intern_source(SourceRef::new(adapter, id));
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
    let pending_bytes = if adapter == crate::wire::MERGE_SOURCE {
        Some(pending_merge_bytes())
    } else {
        None
    };
    let card_bytes = serde_json::to_vec(&chirograph::PortableCardV1 {
        title: format!("title-{adapter}"),
        values: vec![],
        badges: vec!["detail".into()],
        media: pending_bytes
            .as_ref()
            .map(|bytes| vec![ContentHash::of(bytes)])
            .unwrap_or_default(),
    })
    .unwrap();
    let hash = ContentHash::of(&card_bytes);
    let key = PresentationKey(format!("required-{adapter}"));
    let bindings = if with_binding {
        vec![PresentationBinding {
            instance: InstanceId(0),
            key: key.clone(),
        }]
    } else {
        vec![]
    };
    let offers = if with_offer {
        [(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: hash,
                byte_size: card_bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: "required".into(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: vec![],
                },
            }],
        )]
        .into_iter()
        .collect()
    } else {
        Default::default()
    };
    (
        ProjectionSnapshot {
            version: ProtocolVersion::V1,
            session: ProjectionSession(session.into()),
            scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
            presentation: PresentationManifest { bindings, offers },
            cache_policy: Default::default(),
        },
        {
            let mut resources = [(hash, card_bytes)].into_iter().collect::<HashMap<_, _>>();
            if let Some(bytes) = pending_bytes {
                resources.insert(ContentHash::of(&bytes), bytes);
            }
            resources
        },
    )
}

fn mount_required_card_route(
    target: &str,
    adapter: &str,
    with_binding: bool,
    with_offer: bool,
) -> MereRouteSession {
    let include_archive = target == crate::wire::MERE_ARCHIVE;
    let mut panel = panel_with_archive(include_archive);
    seed_partial_state(&mut panel);
    let (required, mut resources) =
        required_card_snapshot(target, adapter, with_binding, with_offer);
    let sessions = if target == MERE_SESSIONS {
        required.clone()
    } else {
        empty_snapshot(MERE_SESSIONS)
    };
    let archive = if include_archive {
        Some(required)
    } else {
        None
    };
    for _ in 0..16 {
        if panel.closed || panel.outbox.is_empty() {
            break;
        }
        let outgoing = panel.take_outgoing();
        for line in outgoing {
            let request: CarrierRequest = serde_json::from_str(&line).unwrap();
            let body = match request.body {
                chirograph::CarrierRequestBody::Snapshot(requested) => {
                    let name = requested.session.0.as_str();
                    let snapshot = if name == MERE_SESSIONS {
                        sessions.clone()
                    } else if name == MERE_GRAPH {
                        empty_snapshot(MERE_GRAPH)
                    } else if name == crate::wire::MERE_ARCHIVE {
                        archive.clone().unwrap_or_else(|| empty_snapshot(name))
                    } else {
                        panic!("unexpected projection {name}")
                    };
                    CarrierResponseBody::Snapshot(Box::new(snapshot))
                },
                chirograph::CarrierRequestBody::Resource(resource) => {
                    let bytes = resources
                        .remove(&resource.resource)
                        .expect("known card resource");
                    CarrierResponseBody::Resource(chirograph::ResourceResponse::new(
                        resource.session,
                        bytes,
                    ))
                },
                other => panic!("unexpected carrier request: {other:?}"),
            };
            let response = chirograph::line::encode(&CarrierOutput::Response(CarrierResponse {
                id: request.id,
                body: Ok(body),
            }))
            .unwrap();
            panel.on_line(&response);
            if panel.closed {
                break;
            }
        }
    }
    panel
}

fn assert_route_purged(panel: &MereRouteSession) {
    assert!(panel.closed);
    assert!(matches!(&panel.model.status, ViewStatus::Unavailable(_)));
    assert!(panel.cards.is_empty());
    assert!(panel.records.is_empty());
    assert!(panel.inspected.is_none());
    assert!(panel.transient_layout.is_none());
    assert!(panel.merge_choices.is_empty());
    assert!(panel.merge.is_none());
    assert!(panel.material.is_none());
}

macro_rules! required_card_rejection_test {
    ($name:ident, $target:expr, $adapter:expr, $with_binding:expr, $with_offer:expr, $expected:expr) => {
        #[test]
        fn $name() {
            let panel = mount_required_card_route(
                $target,
                $adapter,
                $with_binding,
                $with_offer,
            );
            assert_route_purged(&panel);
            assert!(panel.outbox.is_empty(), "no resolve/resource work after admission failure");
            assert!(matches!(&panel.model.status, ViewStatus::Unavailable(note) if note.message.contains($expected)), "{} / {}: {:?}", $target, $adapter, panel.model.status);
        }
    };
}

required_card_rejection_test!(
    required_card_sessions_missing_binding,
    MERE_SESSIONS,
    crate::wire::SESSIONS_SOURCE,
    false,
    true,
    "Missing material card binding"
);
required_card_rejection_test!(
    required_card_sessions_missing_offer,
    MERE_SESSIONS,
    crate::wire::SESSIONS_SOURCE,
    true,
    false,
    "Missing material card offer"
);
required_card_rejection_test!(
    required_card_merge_missing_binding,
    MERE_SESSIONS,
    crate::wire::MERGE_SOURCE,
    false,
    true,
    "Missing material card binding"
);
required_card_rejection_test!(
    required_card_merge_missing_offer,
    MERE_SESSIONS,
    crate::wire::MERGE_SOURCE,
    true,
    false,
    "Missing material card offer"
);
required_card_rejection_test!(
    required_card_archive_missing_binding,
    crate::wire::MERE_ARCHIVE,
    crate::wire::ARCHIVE_SOURCE,
    false,
    true,
    "Missing material card binding"
);
required_card_rejection_test!(
    required_card_archive_missing_offer,
    crate::wire::MERE_ARCHIVE,
    crate::wire::ARCHIVE_SOURCE,
    true,
    false,
    "Missing material card offer"
);

fn pending_merge_bytes() -> Vec<u8> {
    serde_json::to_vec(&crate::wire::PendingMergeV1 {
        schema: crate::wire::PENDING_MERGE_SCHEMA.into(),
        source: pandect::SessionId(uuid::Uuid::from_u128(9)),
        source_name: "source".into(),
        target: pandect::SessionId(uuid::Uuid::from_u128(10)),
        target_name: "target".into(),
        conflicts: vec![],
        carried: vec![],
        dropped: vec![],
    })
    .unwrap()
}

fn mount_empty_route_with_merges(
    merge_media: &[Vec<ContentHash>],
    pending_bytes: &[u8],
) -> MereRouteSession {
    let mut panel = panel();
    let empty = SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), Scene::new()).unwrap();
    let graph = ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession(MERE_GRAPH.into()),
        scene: empty,
        presentation: PresentationManifest::default(),
        cache_policy: Default::default(),
    };
    let mut resources = HashMap::new();
    for (index, media) in merge_media.iter().enumerate() {
        let card = chirograph::PortableCardV1 {
            title: format!("pending merge {index}"),
            values: vec![],
            badges: vec![],
            media: media.clone(),
        };
        let bytes = serde_json::to_vec(&card).unwrap();
        resources.insert(ContentHash::of(&bytes), bytes);
    }
    resources.insert(ContentHash::of(pending_bytes), pending_bytes.to_vec());
    seed_partial_state(&mut panel);
    while !panel.outbox.is_empty() && !panel.closed {
        let outgoing = panel.take_outgoing();
        assert_eq!(outgoing.len(), 1);
        let request: CarrierRequest = serde_json::from_str(&outgoing[0]).unwrap();
        let body = match request.body {
            chirograph::CarrierRequestBody::Snapshot(snapshot)
                if snapshot.session.0 == MERE_SESSIONS =>
            {
                CarrierResponseBody::Snapshot(Box::new(sessions_snapshot(merge_media)))
            },
            chirograph::CarrierRequestBody::Snapshot(snapshot)
                if snapshot.session.0 == MERE_GRAPH =>
            {
                CarrierResponseBody::Snapshot(Box::new(graph.clone()))
            },
            chirograph::CarrierRequestBody::Resource(resource) => {
                let bytes = resources.get(&resource.resource).unwrap_or_else(|| {
                    panic!("unexpected resource request: {:?}", resource.resource)
                });
                CarrierResponseBody::Resource(chirograph::ResourceResponse::new(
                    resource.session,
                    bytes.clone(),
                ))
            },
            other => panic!("unexpected carrier request: {other:?}"),
        };
        let line = chirograph::line::encode(&CarrierOutput::Response(CarrierResponse {
            id: request.id,
            body: Ok(body),
        }))
        .unwrap();
        panel.on_line(&line);
    }
    panel
}

#[test]
fn purged_panel_waits_immediately_through_reconnect_and_admission() {
    let panel = closed_panel();
    let mut transport = graphshell_client::RemoteSession::new(CapabilityProfile::new([
        PresentationCapability::PortableCard,
    ]));
    transport.channel_closed();
    assert!(!panel.in_flight_with_transport(&transport));

    transport.reconnecting();
    assert!(panel.in_flight_with_transport(&transport));
    transport.set_status("rejoining: challenge, admission");
    assert!(panel.in_flight_with_transport(&transport));
    assert!(
        !panel.in_flight(),
        "no replacement panel requests exist yet"
    );

    transport.rejoined();
    assert!(
        panel.in_flight_with_transport(&transport),
        "new discovery is pending after transport admission"
    );
}

#[test]
fn refused_reconnect_releases_wait_and_keeps_panel_purged() {
    let panel = closed_panel();
    let mut transport = graphshell_client::RemoteSession::new(CapabilityProfile::default());
    transport.reconnecting();
    assert!(panel.in_flight_with_transport(&transport));
    transport.link_failed("rejoin refused: grant revoked");
    assert!(!panel.in_flight_with_transport(&transport));
    assert_eq!(panel.status(), "unavailable");
    assert!(panel.model().sessions.is_empty());
    assert!(panel.records().is_empty());
    assert!(panel.primary().is_none());
}

#[path = "busy_tests/admission.rs"]
mod admission;
