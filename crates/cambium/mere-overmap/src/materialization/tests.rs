// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use chirograph::{
    BoundsRelationship, PresentationBinding, PresentationCapability, PresentationCodec,
    PresentationKey, PresentationManifest, PresentationOffer, PresentationSemantics,
    ProjectionOffer, ProtocolVersion, SemanticRole,
};
use sceno::{
    Arrangement, Footprint, ProjectedItem, Representation, Scene, Score, SourceRef, Transform2,
};
use scenotime::{Revision, SceneEpoch, SceneSnapshot};
use std::{
    collections::BTreeMap,
    future::Future,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
};

// Fake operations are immediately ready, but this executor also handles a
// future that yields and wakes without unsafe code or a runtime dependency.
fn block_on<T>(future: impl Future<Output = T>) -> T {
    struct ThreadWake(std::thread::Thread);
    impl Wake for ThreadWake {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
    }
    let waker = Waker::from(Arc::new(ThreadWake(std::thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => std::thread::park(),
        }
    }
}

struct Fake {
    descriptor: EndpointDescriptor,
    snapshots: HashMap<String, ProjectionSnapshot>,
    resources: HashMap<ContentHash, Vec<u8>>,
    fetched: Vec<(ProjectionSession, ContentHash)>,
    opens: usize,
    fail: bool,
}
impl MaterializationSource for Fake {
    async fn open_descriptor(&mut self) -> Result<EndpointDescriptor, String> {
        self.opens += 1;
        Ok(self.descriptor.clone())
    }
    async fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        self.snapshots
            .get(&request.session.0)
            .cloned()
            .ok_or("Missing snapshot".into())
    }
    async fn resource(
        &mut self,
        session: ProjectionSession,
        hash: ContentHash,
    ) -> Result<Vec<u8>, String> {
        self.fetched.push((session, hash));
        if self.fail {
            return Err("Read failed".into());
        }
        self.resources
            .get(&hash)
            .cloned()
            .ok_or("Missing resource".into())
    }
}
fn envelope() -> wire::SessionGraphV1 {
    wire::SessionGraphV1 {
        schema: wire::SESSION_GRAPH_SCHEMA.into(),
        application: "test".into(),
        views: BTreeMap::new(),
        session: pandect::SessionId(uuid::Uuid::from_u128(7)),
        read_only: true,
        epoch: SceneEpoch(1),
        revision: Revision(1),
        graph: kernel::graph::Graph::new().to_snapshot(),
        facets: Default::default(),
    }
}
fn snapshot_fixture(
    name: &str,
    entries: &[(&str, String, Vec<ContentHash>)],
    resources: &mut HashMap<ContentHash, Vec<u8>>,
) -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let mut presentation = PresentationManifest::default();
    for (index, (adapter, id, media)) in entries.iter().enumerate() {
        let source = scene.intern_source(SourceRef::new(*adapter, id));
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
        let bytes = serde_json::to_vec(&PortableCardV1 {
            title: id.clone(),
            values: vec![],
            badges: vec![],
            media: media.clone(),
        })
        .unwrap();
        let hash = ContentHash::of(&bytes);
        resources.insert(hash, bytes.clone());
        let key = PresentationKey(format!("card:{index}"));
        presentation.bindings.push(PresentationBinding {
            instance: InstanceId(index as u32),
            key: key.clone(),
        });
        presentation.offers.insert(
            key,
            vec![PresentationOffer {
                codec: PresentationCodec::PortableCardV1,
                resource: hash,
                byte_size: bytes.len() as u64,
                requires: PresentationCapability::PortableCard,
                semantics: PresentationSemantics {
                    label: id.clone(),
                    role: SemanticRole::Graphic,
                    bounds: BoundsRelationship::FillFootprint,
                    actions: vec![],
                },
            }],
        );
    }
    ProjectionSnapshot {
        version: ProtocolVersion::V1,
        session: ProjectionSession(name.into()),
        scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(),
        presentation,
        cache_policy: Default::default(),
    }
}
fn fixture(material: &wire::SessionGraphV1) -> (Fake, ContentHash) {
    let bytes = serde_json::to_vec(material).unwrap();
    let hash = ContentHash::of(&bytes);
    let mut resources = HashMap::from([(hash, bytes)]);
    let sessions = snapshot_fixture(
        wire::MERE_SESSIONS,
        &[(wire::SESSIONS_SOURCE, "mere".into(), vec![])],
        &mut resources,
    );
    let graph = snapshot_fixture(
        wire::MERE_GRAPH,
        &[(
            wire::SESSION_ITEM_SOURCE,
            envelope().session.as_uuid().to_string(),
            vec![hash],
        )],
        &mut resources,
    );
    let archive = snapshot_fixture(wire::MERE_ARCHIVE, &[], &mut resources);
    let snapshots = [sessions, graph, archive]
        .into_iter()
        .map(|s| (s.session.0.clone(), s))
        .collect();
    let descriptor = EndpointDescriptor {
        label: "test".into(),
        projections: [wire::MERE_SESSIONS, wire::MERE_GRAPH, wire::MERE_ARCHIVE]
            .into_iter()
            .map(|name| ProjectionOffer {
                label: name.into(),
                request: ProjectionRequest {
                    version: ProtocolVersion::V1,
                    session: ProjectionSession(name.into()),
                    score: Score::new(Arrangement::Grid(Default::default())),
                },
            })
            .collect(),
    };
    (
        Fake {
            descriptor,
            snapshots,
            resources,
            fetched: vec![],
            opens: 0,
            fail: false,
        },
        hash,
    )
}
fn read_complete(fake: &mut Fake) -> Result<MaterializedMere, String> {
    block_on(read(fake, "test", GraphRead::Complete))
}

#[test]
fn complete_accepts_read_only_and_uses_actual_session_source() {
    let (mut fake, hash) = fixture(&envelope());
    let unrelated = ContentHash::of(b"not graph material");
    let graph = snapshot_fixture(
        wire::MERE_GRAPH,
        &[
            (
                cartography::MERE_GRAPH_ADAPTER,
                "node".into(),
                vec![unrelated],
            ),
            (
                wire::SESSION_ITEM_SOURCE,
                envelope().session.as_uuid().to_string(),
                vec![hash],
            ),
        ],
        &mut fake.resources,
    );
    fake.snapshots.insert(wire::MERE_GRAPH.into(), graph);
    let material = read_complete(&mut fake).unwrap();
    assert!(material.graph_material.unwrap().read_only);
    assert!(fake.fetched.iter().any(|(_, h)| *h == hash));
    assert!(!fake.fetched.iter().any(|(_, h)| *h == unrelated));
}

#[test]
fn projection_only_does_not_fetch_graph_cards_or_media() {
    let (mut fake, _) = fixture(&envelope());
    let material = block_on(read(&mut fake, "test", GraphRead::ProjectionOnly)).unwrap();
    assert!(material.graph_material.is_none());
    assert!(
        !fake
            .fetched
            .iter()
            .any(|(session, _)| session.0 == wire::MERE_GRAPH)
    );
}

#[test]
fn corruption_and_read_failure_return_no_result() {
    let (mut fake, hash) = fixture(&envelope());
    fake.resources.insert(hash, b"corrupt".to_vec());
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("content hash")
    );
    let (mut fake, _) = fixture(&envelope());
    let card = route::card_requests(&fake.snapshots[wire::MERE_SESSIONS])[0].resource;
    fake.resources.insert(card, b"corrupt card".to_vec());
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("content hash")
    );
    let (mut fake, _) = fixture(&envelope());
    fake.fail = true;
    assert_eq!(read_complete(&mut fake).unwrap_err(), "Read failed");
    let (mut fake, hash) = fixture(&envelope());
    fake.resources.remove(&hash);
    assert_eq!(read_complete(&mut fake).unwrap_err(), "Missing resource");
    // Earlier cards succeeded, but the later media failure still returns no result.
    assert!(fake.fetched.len() >= 3);
}

#[test]
fn fresh_descriptor_and_fill_clear_disappearing_archive_and_merge() {
    let (mut fake, _) = fixture(&envelope());
    let mut model = MereViewModel {
        layout: "host layout".into(),
        notice: Some("host notice".into()),
        ..Default::default()
    };
    let original = model.clone();
    read_complete(&mut fake).unwrap().fill_model(&mut model);
    assert!(model.archive.is_some());
    model.merge = Some(mere_view::MergeModel {
        source: envelope().session,
        source_name: "source".into(),
        target_name: "target".into(),
        conflicts: vec![],
        dropped: vec![],
    });
    fake.descriptor
        .projections
        .retain(|p| p.request.session.0 != wire::MERE_ARCHIVE);
    read_complete(&mut fake).unwrap().fill_model(&mut model);
    assert_eq!(fake.opens, 2);
    assert!(model.archive.is_none());
    assert!(model.merge.is_none());
    assert_eq!(model.layout, original.layout);
    assert_eq!(model.status, original.status);
    assert_eq!(model.actions, original.actions);
    assert_eq!(model.notice, original.notice);
}

fn add_merge(fake: &mut Fake, schema: &str, media_count: usize) -> ContentHash {
    let pending = wire::PendingMergeV1 {
        schema: schema.into(),
        source: envelope().session,
        source_name: "source".into(),
        target: pandect::SessionId(uuid::Uuid::from_u128(8)),
        target_name: "target".into(),
        conflicts: vec![],
        carried: vec![],
        dropped: vec![],
    };
    let bytes = serde_json::to_vec(&pending).unwrap();
    let hash = ContentHash::of(&bytes);
    fake.resources.insert(hash, bytes);
    let media = (0..media_count)
        .map(|index| {
            if index == 0 {
                hash
            } else {
                ContentHash::of(b"ignored merge media")
            }
        })
        .collect();
    let sessions = snapshot_fixture(
        wire::MERE_SESSIONS,
        &[(
            wire::MERGE_SOURCE,
            pending.source.as_uuid().to_string(),
            media,
        )],
        &mut fake.resources,
    );
    fake.snapshots.insert(wire::MERE_SESSIONS.into(), sessions);
    hash
}

#[test]
fn both_modes_read_pending_merge_without_synthesizing_controls() {
    for mode in [GraphRead::ProjectionOnly, GraphRead::Complete] {
        let (mut fake, _) = fixture(&envelope());
        let hash = add_merge(&mut fake, wire::PENDING_MERGE_SCHEMA, 1);
        let material = block_on(read(&mut fake, "test", mode)).unwrap();
        assert!(material.pending_merge.is_some());
        assert!(fake.fetched.iter().any(|(_, h)| *h == hash));
        let mut model = MereViewModel::default();
        material.fill_model(&mut model);
        assert!(model.merge.is_none());
    }
}

#[test]
fn no_pending_merge_cards_is_a_valid_materialization() {
    let (mut fake, _) = fixture(&envelope());
    let material = read_complete(&mut fake).unwrap();
    assert!(material.pending_merge.is_none());
}

#[test]
fn pending_merge_uses_its_first_media_resource() {
    let (mut fake, _) = fixture(&envelope());
    let hash = add_merge(&mut fake, wire::PENDING_MERGE_SCHEMA, 2);
    let material = read_complete(&mut fake).unwrap();
    assert!(material.pending_merge.is_some());
    assert!(fake.fetched.iter().any(|(_, fetched)| *fetched == hash));
    assert_eq!(
        fake.fetched
            .iter()
            .filter(|(_, fetched)| *fetched == hash)
            .count(),
        1
    );
    assert!(
        !fake
            .fetched
            .iter()
            .any(|(_, fetched)| *fetched == ContentHash::of(b"ignored merge media"))
    );

    let (mut fake, _) = fixture(&envelope());
    add_merge(&mut fake, wire::PENDING_MERGE_SCHEMA, 0);
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("media resource")
    );

    let (mut fake, _) = fixture(&envelope());
    add_merge(&mut fake, "wrong", 1);
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("another schema")
    );
    let (mut fake, _) = fixture(&envelope());
    let hash = add_merge(&mut fake, wire::PENDING_MERGE_SCHEMA, 1);
    fake.resources.insert(hash, b"corruption".to_vec());
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("content hash")
    );
}

#[test]
fn pending_merge_admission_rejects_multiple_cards_and_malformed_material() {
    let (mut fake, _) = fixture(&envelope());
    let hash = add_merge(&mut fake, wire::PENDING_MERGE_SCHEMA, 1);
    let sessions = snapshot_fixture(
        wire::MERE_SESSIONS,
        &[
            (wire::MERGE_SOURCE, "merge-a".into(), vec![hash]),
            (wire::MERGE_SOURCE, "merge-b".into(), vec![hash]),
        ],
        &mut fake.resources,
    );
    fake.snapshots.insert(wire::MERE_SESSIONS.into(), sessions);
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("Multiple pending merge cards")
    );

    let (mut fake, _) = fixture(&envelope());
    let malformed = b"not json".to_vec();
    let hash = ContentHash::of(&malformed);
    let sessions = snapshot_fixture(
        wire::MERE_SESSIONS,
        &[(wire::MERGE_SOURCE, "merge".into(), vec![hash])],
        &mut fake.resources,
    );
    fake.resources.insert(hash, malformed);
    fake.snapshots.insert(wire::MERE_SESSIONS.into(), sessions);
    assert!(
        read_complete(&mut fake)
            .unwrap_err()
            .contains("Invalid pending merge material")
    );
}

#[test]
fn complete_without_attached_session_does_not_read_arbitrary_card_media() {
    let (mut fake, _) = fixture(&envelope());
    let graph = snapshot_fixture(
        wire::MERE_GRAPH,
        &[(
            cartography::MERE_GRAPH_ADAPTER,
            "node".into(),
            vec![ContentHash::of(b"unavailable node media")],
        )],
        &mut fake.resources,
    );
    fake.snapshots.insert(wire::MERE_GRAPH.into(), graph);
    assert!(read_complete(&mut fake).unwrap().graph_material.is_none());
}

#[test]
fn required_session_merge_and_archive_cards_must_have_bindings_and_offers() {
    for mode in [GraphRead::ProjectionOnly, GraphRead::Complete] {
        for (name, adapter) in [
            (wire::MERE_SESSIONS, wire::SESSIONS_SOURCE),
            (wire::MERE_SESSIONS, wire::MERGE_SOURCE),
            (wire::MERE_ARCHIVE, wire::ARCHIVE_SOURCE),
        ] {
            for missing_binding in [true, false] {
                let (mut fake, _) = fixture(&envelope());
                let mut snapshot = snapshot_fixture(
                    name,
                    &[(adapter, "required".into(), vec![])],
                    &mut fake.resources,
                );
                if missing_binding {
                    snapshot.presentation.bindings.clear();
                } else {
                    snapshot.presentation.offers.clear();
                }
                fake.snapshots.insert(name.into(), snapshot);
                let error = block_on(read(&mut fake, "test", mode)).unwrap_err();
                assert!(
                    error.contains(if missing_binding {
                        "Missing material card binding"
                    } else {
                        "Missing material card offer"
                    }),
                    "{name}: {error}"
                );
            }
        }
    }
}

#[test]
fn graph_card_binding_is_required_only_for_complete_reads() {
    for adapter in [wire::SESSION_ITEM_SOURCE, cartography::MERE_GRAPH_ADAPTER] {
        let (mut fake, _) = fixture(&envelope());
        let mut graph = snapshot_fixture(
            wire::MERE_GRAPH,
            &[(adapter, envelope().session.as_uuid().to_string(), vec![])],
            &mut fake.resources,
        );
        graph.presentation.bindings.clear();
        fake.snapshots.insert(wire::MERE_GRAPH.into(), graph);
        assert!(block_on(read(&mut fake, "test", GraphRead::ProjectionOnly)).is_ok());
        assert!(
            !fake
                .fetched
                .iter()
                .any(|(session, _)| session.0 == wire::MERE_GRAPH)
        );
        assert!(
            read_complete(&mut fake)
                .unwrap_err()
                .contains("Missing material card binding")
        );
    }
}

#[path = "tests/admission.rs"]
mod admission;
