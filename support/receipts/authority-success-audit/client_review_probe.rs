//! Real public client functions, generated valid scenes and protocol replies.
//! No network, browser, OS accessibility bridge or installed resident.
use std::collections::HashMap;
use chirograph::*;
use graphshell_client::{ClientState, SessionCore, Progress, Outcome, PresentationResolution, RemoteSession};
use graphshell_client::frozen::FrozenScene;
use sceno::{Scene, SourceRef, InstanceId, ProjectedItem, Transform2, Footprint, Representation};
use scenotime::{SceneSnapshot, SceneEpoch, Revision};

fn session() -> ProjectionSession { ProjectionSession("audit:client".into()) }
fn profile() -> CapabilityProfile { CapabilityProfile::new([PresentationCapability::NativeGlyph]) }
fn bytes(index: u32) -> Vec<u8> { serde_json::to_vec(&NativeGlyphV1 { label: format!("Item {index}"), icon: None, color: None }).unwrap() }
fn snapshot() -> ProjectionSnapshot {
    let mut scene = Scene::new();
    let mut presentation = PresentationManifest::default();
    for index in 0..2 {
        let source = scene.intern_source(SourceRef::new("audit", format!("item:{index}")));
        scene.items.push(ProjectedItem { source, space: Scene::WORLD, transform: Transform2::IDENTITY, footprint: Footprint::Point, representation: Representation::Glyph, layer: 0, visible: true, hit: None, channels: vec![] });
        let key = PresentationKey(format!("item:{index}"));
        presentation.bindings.push(PresentationBinding { instance: InstanceId(index), key: key.clone() });
        presentation.offers.insert(key, vec![PresentationOffer { codec: PresentationCodec::NativeGlyphV1, resource: ContentHash::of(&bytes(index)), byte_size: bytes(index).len() as u64, requires: PresentationCapability::NativeGlyph,
            semantics: PresentationSemantics { label: format!("Item {index}"), role: SemanticRole::Graphic, bounds: BoundsRelationship::FillFootprint, actions: vec![] } }]);
    }
    ProjectionSnapshot { version: ProtocolVersion::V1, session: session(), scene: SceneSnapshot::from_dense(SceneEpoch(1), Revision(1), scene).unwrap(), presentation,
        cache_policy: CachePolicy { retention: CacheRetention::MemoryOnly, expires_at_ms: None, purge_on_revocation: true } }
}
fn request() -> ProjectionRequest { ProjectionRequest { version: ProtocolVersion::V1, session: session(), score: sceno::Score::new(sceno::Arrangement::Spiral(Default::default())) } }
fn descriptor() -> EndpointDescriptor { EndpointDescriptor { label: "client audit".into(), projections: vec![ProjectionOffer { label: "two items".into(), request: request() }] } }
fn core() -> SessionCore {
    let mut core = SessionCore::from_descriptor(descriptor(), profile());
    let _ = core.mount(0).unwrap();
    assert!(matches!(core.on_response(CarrierResponseBody::Snapshot(Box::new(snapshot()))).unwrap(), Progress::Done(Outcome::Mounted(_))));
    core
}
fn expects_first(progress: Progress<Outcome>) {
    assert!(matches!(progress, Progress::Ask(CarrierRequestBody::Resource(r)) if r.session == session() && r.resource == ContentHash::of(&bytes(0))));
}
#[test] fn requested_resource_control_completes() {
    let mut core = core();
    expects_first(core.resolve(&session(), InstanceId(0)).unwrap());
    let progress = core.on_response(CarrierResponseBody::Resource(ResourceResponse::new(session(), bytes(0)))).unwrap();
    assert!(matches!(progress, Progress::Done(Outcome::Resolved(_))));
    println!("CONTROL requested-resource resolves=true");
}
#[test] fn another_advertised_resource_is_accepted_repeatedly_without_completing() {
    let mut core = core();
    expects_first(core.resolve(&session(), InstanceId(0)).unwrap());
    for _ in 0..6 {
        expects_first(core.on_response(CarrierResponseBody::Resource(ResourceResponse::new(session(), bytes(1)))).unwrap());
    }
    assert!(matches!(core.client().resolve(&session(), InstanceId(1), &profile()).unwrap(), PresentationResolution::Ready(_)));
    assert!(core.is_awaiting());
    println!("OBSERVED resource-mismatch accepted-replies=6 selected-resource-still-missing=true other-resource-cached=true");
}
fn names() -> HashMap<InstanceId, String> { HashMap::from([(InstanceId(0), "Item 0".into()), (InstanceId(1), "Item 1".into())]) }
fn orders(snapshot: ProjectionSnapshot) -> (Vec<InstanceId>, Vec<InstanceId>) {
    snapshot.scene.validate().unwrap();
    let frozen = FrozenScene::freeze_snapshot(&snapshot.scene, "audit", &names());
    let mut client = ClientState::default();
    client.apply_snapshot(snapshot).unwrap();
    let remote = client.accessibility_tree(&session(), &profile()).unwrap();
    (remote.children.into_iter().map(|i| i.instance).collect(), frozen.instances.into_iter().map(|i| i.instance).collect())
}
#[test] fn dense_order_control_agrees() {
    let (remote, frozen) = orders(snapshot());
    assert_eq!(remote, frozen);
    println!("CONTROL dense-order readers-agree=true");
}
#[test] fn frozen_reader_ignores_explicit_item_order() {
    let mut s = snapshot();
    s.scene.tables.item_order = vec![Some(1), Some(0)];
    let (remote, frozen) = orders(s);
    assert_eq!(remote, vec![InstanceId(1), InstanceId(0)]);
    assert_eq!(frozen, vec![InstanceId(0), InstanceId(1)]);
    println!("OBSERVED reordered-snapshot remote=[1,0] frozen=[0,1]");
}
#[test] fn remote_accessibility_ignores_item_visibility() {
    let mut s = snapshot();
    s.scene.tables.items[1].as_mut().unwrap().visible = false;
    assert!(!s.scene.is_shown(InstanceId(1)));
    let (remote, frozen) = orders(s);
    assert_eq!(remote, vec![InstanceId(0), InstanceId(1)]);
    assert_eq!(frozen, vec![InstanceId(0)]);
    println!("OBSERVED invisible-item remote-includes=true frozen-excludes=true");
}
fn outgoing(remote: &mut RemoteSession) -> CarrierRequest {
    let lines = remote.take_outgoing();
    assert_eq!(lines.len(), 1);
    serde_json::from_str(&lines[0]).unwrap()
}
fn answer(remote: &mut RemoteSession, id: u64, body: CarrierResponseBody) {
    remote.on_line(&serde_json::to_string(&CarrierOutput::Response(CarrierResponse { id, body: Ok(body) })).unwrap());
}
fn mounted_remote() -> RemoteSession {
    let mut remote = RemoteSession::new(profile());
    remote.joined();
    let discover = outgoing(&mut remote);
    answer(&mut remote, discover.id, CarrierResponseBody::Descriptor(descriptor()));
    let mount = outgoing(&mut remote);
    answer(&mut remote, mount.id, CarrierResponseBody::Snapshot(Box::new(snapshot())));
    assert_eq!(remote.status(), "open");
    remote
}
#[test] fn intentional_disconnect_control_clears_mount_status_and_restarts_discovery() {
    let mut remote = mounted_remote();
    remote.disconnect(); remote.channel_closed();
    assert_eq!(remote.mounted().unwrap().status, SessionStatus::Disconnected);
    assert!(!remote.in_flight());
    remote.reconnecting(); remote.rejoined();
    assert!(matches!(outgoing(&mut remote).body, CarrierRequestBody::Discover));
    println!("CONTROL intentional-disconnect mount-disconnected=true rediscovery-sent=true");
}
#[test] fn unexpected_close_leaves_mount_live_and_a_lost_request_blocks_rejoin() {
    let mut mounted = mounted_remote();
    mounted.channel_closed();
    assert_eq!(mounted.mounted().unwrap().status, SessionStatus::Live);
    assert!(mounted.status().starts_with("closed:"));
    let mut waiting = RemoteSession::new(profile());
    waiting.joined();
    outgoing(&mut waiting);
    waiting.channel_closed();
    assert!(waiting.in_flight());
    assert!(waiting.driver().is_awaiting());
    waiting.reconnecting(); waiting.rejoined();
    assert!(waiting.status().contains("still waiting on request"), "{}", waiting.status());
    assert!(waiting.take_outgoing().is_empty());
    println!("OBSERVED unexpected-close mount-still-live=true lost-request-still-awaited=true rejoin-rediscovery-refused=true");
}
#[test] fn a_snapshot_resume_is_reported_as_a_diff() {
    let mut remote = mounted_remote();
    remote.on_line(&serde_json::to_string(&CarrierOutput::Notice(CarrierNotice { session: session(), epoch: SceneEpoch(1), revision: Revision(2) })).unwrap());
    let resume = outgoing(&mut remote);
    assert!(matches!(resume.body, CarrierRequestBody::Resume(_)));
    let mut fresh = snapshot();
    fresh.scene.revision = Revision(2);
    answer(&mut remote, resume.id, CarrierResponseBody::Resume(ResumeReply::Snapshot(Box::new(fresh))));
    assert_eq!(remote.revision(), Some(2));
    assert_eq!(remote.last_resume(), "diff · 1 → 2");
    println!("OBSERVED resume-reply=Snapshot receipt-label=diff revision-updated=true");
}
