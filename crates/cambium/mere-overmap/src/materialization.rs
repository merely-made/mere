// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Ephemeral resident materialization. Hosts retain replay, persistence and
//! publication authority; a failed read returns no materialized state.

use std::collections::HashMap;

use chirograph::{
    ContentHash, EndpointDescriptor, PortableCardV1, ProjectionRequest, ProjectionSession,
    ProjectionSnapshot,
};
use graphshell_endpoint::door::app_client::AppBrokerClient;
use mere_view::MereViewModel;
use sceno::InstanceId;

pub use crate::validate_graph_material;
use crate::{route, wire};

/// How much of the attached graph to read.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphRead {
    ProjectionOnly,
    Complete,
}

/// An already admitted source. Each read opens a fresh descriptor. No method
/// on this boundary writes domain truth or performs product replay.
#[allow(async_fn_in_trait)]
pub trait MaterializationSource {
    async fn open_descriptor(&mut self) -> Result<EndpointDescriptor, String>;
    async fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String>;
    async fn resource(
        &mut self,
        session: ProjectionSession,
        hash: ContentHash,
    ) -> Result<Vec<u8>, String>;
}

impl MaterializationSource for AppBrokerClient {
    async fn open_descriptor(&mut self) -> Result<EndpointDescriptor, String> {
        self.open_session()
            .await
            .map(|opened| opened.descriptor)
            .map_err(|e| e.to_string())
    }
    async fn snapshot(&mut self, request: ProjectionRequest) -> Result<ProjectionSnapshot, String> {
        AppBrokerClient::snapshot(self, request)
            .await
            .map_err(|e| e.to_string())
    }
    async fn resource(
        &mut self,
        session: ProjectionSession,
        hash: ContentHash,
    ) -> Result<Vec<u8>, String> {
        AppBrokerClient::resource(self, session, hash)
            .await
            .map_err(|e| e.to_string())
    }
}

/// One fully checked read; resources remain only in memory.
#[derive(Clone, Debug)]
pub struct MaterializedMere {
    pub sessions: ProjectionSnapshot,
    pub graph: ProjectionSnapshot,
    pub archive: Option<ProjectionSnapshot>,
    pub cards: HashMap<ContentHash, Vec<u8>>,
    pub graph_material: Option<wire::SessionGraphV1>,
    pub pending_merge: Option<wire::PendingMergeV1>,
}

impl MaterializedMere {
    /// Replace the disclosed model reading, retaining host layout, status and
    /// actions. An archive or merge absent from this read clears the old one.
    pub fn fill_model(&self, model: &mut MereViewModel) {
        model.archive = None;
        model.merge = None;
        route::fill(model, &self.sessions, &self.graph, &self.cards);
        if let Some(archive) = &self.archive {
            route::fill_archive(model, &self.sessions, archive, &self.cards);
        }
    }
}

fn request(
    descriptor: &EndpointDescriptor,
    name: &str,
    mandatory: bool,
) -> Result<Option<ProjectionRequest>, String> {
    let mut offers = descriptor
        .projections
        .iter()
        .filter(|offer| offer.request.session.0 == name);
    let first = offers.next().map(|offer| offer.request.clone());
    if offers.next().is_some() {
        return Err(format!("Duplicate projection advertisement: {name}"));
    }
    if mandatory && first.is_none() {
        return Err(format!("Missing projection advertisement: {name}"));
    }
    Ok(first)
}

async fn snapshot(
    source: &mut impl MaterializationSource,
    request: ProjectionRequest,
) -> Result<ProjectionSnapshot, String> {
    let expected = request.session.clone();
    let snapshot = source.snapshot(request).await?;
    if snapshot.session != expected {
        return Err("The snapshot names another projection session.".into());
    }
    snapshot
        .scene
        .validate()
        .map_err(|error| format!("Invalid projection scene: {error:?}"))?;
    Ok(snapshot)
}

async fn resource(
    source: &mut impl MaterializationSource,
    session: &ProjectionSession,
    hash: ContentHash,
) -> Result<Vec<u8>, String> {
    let bytes = source.resource(session.clone(), hash).await?;
    if ContentHash::of(&bytes) != hash {
        return Err("The resource does not match its content hash.".into());
    }
    Ok(bytes)
}

fn source_of(snapshot: &ProjectionSnapshot, instance: InstanceId) -> Option<&sceno::SourceRef> {
    let item = snapshot
        .scene
        .tables
        .items
        .get(instance.0 as usize)?
        .as_ref()?;
    snapshot
        .scene
        .tables
        .sources
        .get(item.source.0 as usize)?
        .as_ref()
}

fn card_hash(snapshot: &ProjectionSnapshot, instance: InstanceId) -> Result<ContentHash, String> {
    let mut bindings = snapshot
        .presentation
        .bindings
        .iter()
        .filter(|binding| binding.instance == instance);
    let binding = bindings.next().ok_or("Missing material card binding.")?;
    if bindings.next().is_some() {
        return Err("Duplicate material card binding.".into());
    }
    let offer = snapshot
        .presentation
        .offers
        .get(&binding.key)
        .and_then(|offers| offers.first())
        .ok_or("Missing material card offer.")?;
    if offer.codec != chirograph::PresentationCodec::PortableCardV1
        || offer.requires != chirograph::PresentationCapability::PortableCard
    {
        return Err(
            "The mere reading requires an advertised portable card codec and capability.".into(),
        );
    }
    Ok(offer.resource)
}

async fn cards(
    source: &mut impl MaterializationSource,
    snapshot: &ProjectionSnapshot,
    result: &mut HashMap<ContentHash, Vec<u8>>,
) -> Result<(), String> {
    // Required cards come from the projection's active source items, not its
    // possibly incomplete presentation sidecar. Keep checking other advertised
    // bindings too, as before. This function is never called for graph cards
    // during ProjectionOnly reads.
    let mut instances: Vec<_> = snapshot
        .scene
        .active_items_in_order()
        .into_iter()
        .filter_map(|(instance, _)| {
            let source = source_of(snapshot, instance)?;
            let required = match snapshot.session.0.as_str() {
                wire::MERE_SESSIONS => matches!(
                    source.adapter.as_str(),
                    wire::SESSIONS_SOURCE | wire::MERGE_SOURCE
                ),
                wire::MERE_ARCHIVE => source.adapter == wire::ARCHIVE_SOURCE,
                wire::MERE_GRAPH => matches!(
                    source.adapter.as_str(),
                    wire::SESSION_ITEM_SOURCE | cartography::MERE_GRAPH_ADAPTER
                ),
                _ => false,
            };
            required.then_some(instance)
        })
        .collect();
    for binding in &snapshot.presentation.bindings {
        if !instances.contains(&binding.instance) {
            instances.push(binding.instance);
        }
    }
    for instance in instances {
        let hash = card_hash(snapshot, instance)?;
        // Fetch in each projection scope, even if another scope offered the same hash.
        let bytes = resource(source, &snapshot.session, hash).await?;
        serde_json::from_slice::<PortableCardV1>(&bytes)
            .map_err(|e| format!("Invalid portable card: {e}"))?;
        result.insert(hash, bytes);
    }
    Ok(())
}

/// Read fresh projections and validate every fetched resource before decoding.
/// Only Complete fetches graph cards and the scoped graph envelope; projection
/// reads still fetch sessions/archive cards and any advertised pending merge.
pub async fn read(
    source: &mut impl MaterializationSource,
    application: &str,
    graph_read: GraphRead,
) -> Result<MaterializedMere, String> {
    let descriptor = source.open_descriptor().await?;
    let sessions_request = request(&descriptor, wire::MERE_SESSIONS, true)?.unwrap();
    let graph_request = request(&descriptor, wire::MERE_GRAPH, true)?.unwrap();
    let archive_request = request(&descriptor, wire::MERE_ARCHIVE, false)?;
    let sessions = snapshot(source, sessions_request).await?;
    let graph = snapshot(source, graph_request).await?;
    let archive = if let Some(request) = archive_request {
        Some(snapshot(source, request).await?)
    } else {
        None
    };
    let mut result = MaterializedMere {
        sessions,
        graph,
        archive,
        cards: HashMap::new(),
        graph_material: None,
        pending_merge: None,
    };
    cards(source, &result.sessions, &mut result.cards).await?;
    if let Some(archive) = &result.archive {
        cards(source, archive, &mut result.cards).await?;
    }
    if let Some(instance) = route::unique_merge_card(&result.sessions)? {
        let card_hash = card_hash(&result.sessions, instance)?;
        let bytes = result
            .cards
            .get(&card_hash)
            .ok_or("Missing material card resource.")?;
        let card: PortableCardV1 =
            serde_json::from_slice(bytes).map_err(|error| error.to_string())?;
        let hash = route::pending_merge_media(&card)?;
        let bytes = resource(source, &result.sessions.session, hash).await?;
        result.pending_merge = Some(route::decode_pending_merge(&bytes)?);
    }
    if graph_read == GraphRead::Complete {
        cards(source, &result.graph, &mut result.cards).await?;
        let attached: Vec<_> = result
            .graph
            .scene
            .active_items_in_order()
            .into_iter()
            .filter_map(|(instance, _)| {
                source_of(&result.graph, instance)
                    .filter(|source| source.adapter == wire::SESSION_ITEM_SOURCE)
                    .map(|source| (instance, source))
            })
            .collect();
        if attached.len() > 1 {
            return Err("Multiple attached session cards.".into());
        }
        if let Some((instance, attached)) = attached.first() {
            let session = pandect::SessionId(
                uuid::Uuid::parse_str(&attached.id)
                    .map_err(|_| "The material card names no attached session.")?,
            );
            let card_hash = card_hash(&result.graph, *instance)?;
            let bytes = result
                .cards
                .get(&card_hash)
                .ok_or("Missing material card resource.")?;
            let card: PortableCardV1 = serde_json::from_slice(bytes).map_err(|e| e.to_string())?;
            let hash = wire::session_media(&card)?;
            let bytes = resource(source, &result.graph.session, hash).await?;
            let envelope: wire::SessionGraphV1 = serde_json::from_slice(&bytes)
                .map_err(|e| format!("Invalid scoped graph material: {e}"))?;
            validate_graph_material(
                &envelope,
                application,
                session,
                result.graph.scene.epoch,
                result.graph.scene.revision,
            )?;
            result.graph_material = Some(envelope);
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
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
        async fn snapshot(
            &mut self,
            request: ProjectionRequest,
        ) -> Result<ProjectionSnapshot, String> {
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
}
