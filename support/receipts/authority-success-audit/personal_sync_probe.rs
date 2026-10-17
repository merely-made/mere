use graphshell::{
    access::{AccessAction, AccessRecord, AccessTransition},
    personal_sync::{PersonalGraphEvent as Event, PersonalGraphReplica, SyncRoster, SyncSelection},
};
use mere::kernel::graph::{EdgeAssertion, SemanticSubKind};
use muniment::MemoryBackend;
use p2panda_core::SigningKey;
use uuid::Uuid;

const A: Uuid = Uuid::from_u128(1);
const B: Uuid = Uuid::from_u128(2);
fn replica() -> PersonalGraphReplica<MemoryBackend> {
    let seed = [0x28; 32];
    let subject = *SigningKey::from_bytes(&seed).verifying_key().as_bytes();
    PersonalGraphReplica::new(MemoryBackend::new(), [0x49; 32], seed,
        SyncRoster::new([subject]), SyncSelection::default().with_access_records(true))
}
fn node(id: Uuid) -> Event {
    Event::AddNode { id, address: format!("https://sync-audit.test/{id}"), title: id.to_string() }
}
fn assertion() -> Event {
    Event::AssertRelation { from: A, to: B,
        assertion: EdgeAssertion::Semantic { sub_kind: SemanticSubKind::Cites, label: None, decay_progress: None },
        assertion_version: None, statement_id: None, asserted_at_ms: None, basis: None }
}
fn access(time: u64) -> AccessRecord {
    AccessRecord { record_id: Uuid::from_u128(7), container_id: A, address: "https://sync-audit.test/observed".into(),
        action: AccessAction::Examine, persona: "audit:person".into(), device: "audit:device".into(),
        application: "audit".into(), at_ms: time, handler: "audit".into(), dwell_ms: None,
        referring_container_id: None, referring_address: None, transition: AccessTransition::Unknown,
        capture_source: "audit".into(), source_event_id: None, privacy: eidetic::PrivacyClass::TrustedPeersOnly }
}

#[test]
fn synchronized_tag_is_legacy_surface_metadata_and_absent_from_current_content_tags() {
    pollster::block_on(async {
        let mut r = replica();
        r.author(vec![node(A), Event::AddTag { node: A, tag: "audit-tag".into() }]).await.unwrap();
        let p = r.projection().await.unwrap();
        let (key, held) = p.graph.get_node_by_id(A).unwrap();
        assert!(held.tags.contains("audit-tag"));
        assert!(p.graph.shown_resource_id(key).is_some());
        assert!(p.graph.node_content_tags(key).unwrap().is_empty());
    });
}

#[test]
fn changed_access_fact_under_the_same_id_is_accepted_then_silently_discarded() {
    pollster::block_on(async {
        let mut r = replica();
        r.author(vec![node(A), Event::AppendAccess { record: access(100) }]).await.unwrap();
        r.author(vec![Event::AppendAccess { record: access(200) }]).await.unwrap();
        let p = r.projection().await.unwrap();
        assert_eq!(p.access_records, vec![access(100)]);
        assert!(p.conflicts.is_empty());
    });
}

#[test]
fn identical_access_replay_is_a_working_idempotence_control() {
    pollster::block_on(async {
        let mut r = replica();
        r.author(vec![node(A), Event::AppendAccess { record: access(100) }]).await.unwrap();
        r.author(vec![Event::AppendAccess { record: access(100) }]).await.unwrap();
        assert_eq!(r.projection().await.unwrap().access_records, vec![access(100)]);
    });
}

#[test]
fn accepted_missing_endpoint_assertion_prevents_projection_even_after_later_nodes_arrive() {
    pollster::block_on(async {
        let mut r = replica();
        r.author(vec![assertion()]).await.unwrap();
        assert!(r.projection().await.is_err());
        r.author(vec![node(A), node(B)]).await.unwrap();
        assert!(r.projection().await.is_err());
    });
}

#[test]
fn assertion_with_prior_endpoints_is_a_working_projection_control() {
    pollster::block_on(async {
        let mut r = replica();
        r.author(vec![node(A), node(B)]).await.unwrap();
        r.author(vec![assertion()]).await.unwrap();
        let p = r.projection().await.unwrap();
        assert_eq!(p.graph.projected_relations().count(), 1);
        assert!(p.pending.is_empty());
    });
}
