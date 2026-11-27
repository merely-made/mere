// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! A stored legacy translation read over a graph with no kernel, through a
//! legacy hook its host installs (engine split D260). The receipt is written
//! here as pandect writes one, over the D245 surface: its legacy journal shows
//! one resource, and its translation shows another.

mod support;

use std::time::UNIX_EPOCH;

use mere::model::{Author, JournalDelta, PersistedResourceRecord, ResourceDelta, ResourceNode};
use mere::session::SessionGraph;
use muniment::{Backend, MemoryBackend, Seq};
use pandect::graph_placement::SessionPlacement;
use pandect::{LegacyHook, MereSessions, NodeFacetStore, SessionError, SessionId};
use serde::Serialize;
use serde_json::{Value, json};
use support::{Delta, Surface, SurfaceDelta, SurfaceSnapshot};
use uuid::Uuid;

type Sessions = MereSessions<MemoryBackend, Surface>;

/// The surface's snapshots are the current shape.
static CURRENT: LegacyHook<Surface> = LegacyHook {
    assertion_schema: |_| 1,
};
/// A hook naming a schema no reader knows: every comparison refuses.
static UNKNOWN: LegacyHook<Surface> = LegacyHook {
    assertion_schema: |_| 2,
};

const SHOWN: Uuid = Uuid::from_u128(0x5a);
const LEGACY: &str = "https://p8.test/legacy-resource";
const TRANSLATED: &str = "https://p8.test/translated-resource";

#[derive(Serialize)]
struct Frozen {
    graph: SurfaceSnapshot,
    facets: NodeFacetStore,
}

#[derive(Serialize)]
struct Receipt<'a> {
    version: u32,
    source_digest: &'a str,
    translation_digest: &'a str,
    placement: &'a SessionPlacement,
    baseline: &'a Frozen,
    entry_effects: &'a [Vec<Delta>],
    legacy_final: &'a Frozen,
}

fn frozen(graph: &Surface) -> Frozen {
    Frozen {
        graph: graph.to_snapshot(),
        facets: graph.facets().clone(),
    }
}

fn record(iri: &str) -> Delta {
    let resource = ResourceNode::for_term(iri);
    JournalDelta::Resource(ResourceDelta::ReplaySetResourceRecordById {
        resource_id: resource.id().to_string(),
        record: Some(PersistedResourceRecord {
            canonical_iri: resource.canonical_iri().to_owned(),
            facets: Vec::new(),
        }),
    })
}

fn show(iri: &str) -> Delta {
    JournalDelta::Surface(SurfaceDelta::ShowResource {
        surface_id: SHOWN,
        resource_id: ResourceNode::for_term(iri).id(),
    })
}

/// pandect's retained-source digest, over the stored bytes.
fn source_digest(
    placement: &SessionPlacement,
    baseline: Option<&[u8]>,
    entries: &[Vec<u8>],
) -> String {
    let mut hash = blake3::Hasher::new();
    hash.update(b"mere.legacy-translation-source/v1");
    let mut field = |bytes: Option<&[u8]>| {
        hash.update(&[u8::from(bytes.is_some())]);
        if let Some(bytes) = bytes {
            hash.update(&(bytes.len() as u64).to_le_bytes());
            hash.update(bytes);
        }
    };
    field(Some(&serde_json::to_vec(placement).unwrap()));
    field(baseline);
    for entry in entries {
        field(Some(entry));
    }
    hash.finalize().to_hex().to_string()
}

/// A stored session whose two legacy entries record and show `LEGACY`, with
/// a receipt translating them to `TRANSLATED`.
async fn legacy_session(backend: &MemoryBackend) -> SessionId {
    let sessions = Sessions::new_with(backend.clone());
    let author = Author::person("p8-legacy");
    let mut baseline = Surface::default();
    baseline.show(Uuid::from_u128(0x5b), "https://p8.test/baseline");
    let mut session = sessions.begin_with(author.clone(), Some(baseline.clone()));
    let id = session.id();
    session
        .apply_now(author.clone(), vec![record(LEGACY), show(LEGACY)])
        .unwrap();
    session.flush(UNIX_EPOCH).await.unwrap();
    drop(session);

    let prefix = format!("sessions/{}", id.as_uuid());
    let placement = SessionPlacement::legacy(Seq(2), Vec::new());
    let stored_baseline = backend
        .get(&format!("{prefix}/baseline.json"))
        .await
        .unwrap();
    let mut entries = Vec::new();
    for index in 0..2 {
        let key = muniment::journal::entry_key(&format!("{prefix}/journal.jsonl"), Seq(index));
        entries.push(backend.get(&key).await.unwrap().expect("a legacy entry"));
    }
    let source = source_digest(&placement, stored_baseline.as_deref(), &entries);
    let effects = vec![vec![record(TRANSLATED)], vec![show(TRANSLATED)]];
    let mut translated = baseline.clone();
    translated.replay_as(&author, &effects.concat());
    let (baseline, last) = (frozen(&baseline), frozen(&translated));
    let digest = blake3::hash(
        &serde_json::to_vec(&(1u32, &source, &placement, &baseline, &effects, &last)).unwrap(),
    )
    .to_hex()
    .to_string();
    let receipt = Receipt {
        version: 1,
        source_digest: &source,
        translation_digest: &digest,
        placement: &placement,
        baseline: &baseline,
        entry_effects: &effects,
        legacy_final: &last,
    };
    backend
        .put(
            &format!("{prefix}/placement.json"),
            &serde_json::to_vec(&placement).unwrap(),
        )
        .await
        .unwrap();
    backend
        .put(
            &format!("{prefix}/legacy-translation.json"),
            &serde_json::to_vec(&receipt).unwrap(),
        )
        .await
        .unwrap();
    id
}

#[test]
fn an_installed_legacy_hook_reads_the_translation_over_a_host_graph() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let id = legacy_session(&backend).await;
        let (legacy, translated) = (
            ResourceNode::for_term(LEGACY).id(),
            ResourceNode::for_term(TRANSLATED).id(),
        );

        let session = Sessions::new_with(backend.clone())
            .with_legacy(&CURRENT)
            .open_with(id)
            .await
            .unwrap();
        // The journal still holds the legacy entries; the graph is their
        // translation, at the live cursor and at each one before it.
        assert_eq!(session.journal().live_cursor(), Seq(2));
        assert!(matches!(
            &session.journal().entries()[1].delta,
            JournalDelta::Surface(SurfaceDelta::ShowResource { resource_id, .. }) if *resource_id == legacy
        ));
        assert_eq!(session.graph().shown(SHOWN), Some(translated));
        assert!(session.graph().resources().resource(legacy).is_none());
        let first = session.graph_at(Seq(1)).unwrap();
        assert!(first.resources().resource(translated).is_some());
        assert_eq!(first.shown(SHOWN), None);
        assert_eq!(
            session.graph_at(Seq(2)).unwrap().shown(SHOWN),
            Some(translated)
        );
        drop(session);

        // Controls: with no hook the receipt is refused before it is read, and
        // a hook naming an unknown schema refuses the comparison.
        let bare = Sessions::new_with(backend.clone()).open_with(id).await;
        assert!(
            matches!(&bare, Err(SessionError::Corrupt(why)) if why.contains("legacy hook")),
            "{:?}",
            bare.err()
        );
        let unknown = Sessions::new_with(backend.clone())
            .with_legacy(&UNKNOWN)
            .open_with(id)
            .await;
        assert!(
            matches!(&unknown, Err(SessionError::Corrupt(why)) if why.contains("do not reproduce")),
            "{:?}",
            unknown.err()
        );
        // And a translation tampered after its digest was taken is refused.
        let key = format!("sessions/{}/legacy-translation.json", id.as_uuid());
        let stored = String::from_utf8(backend.get(&key).await.unwrap().unwrap()).unwrap();
        let tampered = stored.replacen(
            &translated.to_string(),
            &Uuid::from_u128(0xbad).to_string(),
            1,
        );
        assert_ne!(tampered, stored);
        backend.put(&key, tampered.as_bytes()).await.unwrap();
        let forged = Sessions::new_with(backend)
            .with_legacy(&CURRENT)
            .open_with(id)
            .await;
        assert!(
            matches!(&forged, Err(SessionError::Corrupt(why)) if why.contains("checksum")),
            "{:?}",
            forged.err()
        );
    });
}

const BASELINE: &str = "https://p8.test/baseline";
const CITES: &str = "https://mere.computer/ns/rel#cites";

/// A cites edge from `TRANSLATED` to the baseline's resource, as the kernel
/// wrote one before L7: the edge names its families and the record its
/// sub-kind, fields today's types read and no longer write.
fn pre_l7_cites() -> Value {
    let (from, to) = (
        ResourceNode::for_term(TRANSLATED).id().to_string(),
        ResourceNode::for_term(BASELINE).id().to_string(),
    );
    let mut delta = serde_json::to_value(support::edges_edit(
        ResourceNode::for_term(TRANSLATED).id(),
        ResourceNode::for_term(BASELINE).id(),
        Vec::new(),
    ))
    .unwrap();
    let edges = delta
        .pointer_mut("/ReplaySetResourceEdgesByIds/edges")
        .expect("an edges write");
    *edges = json!([{
        "from_node_id": from,
        "to_node_id": to,
        "families": ["Semantic"],
        "assertions": [{
            "statement_id": "urn:uuid:9b0c7d3e-8f43-4e0a-9d5b-6f1e2a3b4c5d",
            "predicate": CITES,
            "recognized_sub_kind": "Cites",
            "basis": { "evidence": [], "method": null, "note": null }
        }]
    }]);
    delta
}

/// The session `legacy_session` stores, its receipt rewritten so that its
/// second entry's effects also write `pre_l7_cites`, its checksum taken the
/// way a pre-L7 writer took it (over the payload it serialized), and stored
/// pretty-printed.
async fn pre_l7_receipt_session(backend: &MemoryBackend) -> (SessionId, String) {
    let id = legacy_session(backend).await;
    let key = format!("sessions/{}/legacy-translation.json", id.as_uuid());
    let mut receipt: Value =
        serde_json::from_slice(&backend.get(&key).await.unwrap().unwrap()).unwrap();
    receipt["entry_effects"][1]
        .as_array_mut()
        .unwrap()
        .push(pre_l7_cites());
    // The frozen result is what replaying those effects gives.
    let effects: Vec<Vec<Delta>> =
        serde_json::from_value(receipt["entry_effects"].clone()).unwrap();
    let mut translated = Surface::default();
    translated.show(Uuid::from_u128(0x5b), BASELINE);
    translated.replay_as(&Author::person("p8-legacy"), &effects.concat());
    receipt["legacy_final"] = serde_json::to_value(frozen(&translated)).unwrap();
    let payload = json!([
        receipt["version"],
        receipt["source_digest"],
        receipt["placement"],
        receipt["baseline"],
        receipt["entry_effects"],
        receipt["legacy_final"],
    ]);
    let digest = blake3::hash(&serde_json::to_vec(&payload).unwrap())
        .to_hex()
        .to_string();
    receipt["translation_digest"] = Value::from(digest);
    backend
        .put(&key, &serde_json::to_vec_pretty(&receipt).unwrap())
        .await
        .unwrap();
    (id, key)
}

/// Declared-lens D31 (1): a receipt's checksum covers its payload as stored.
/// A receipt written before L7 holds fields today's types drop, so a
/// checksum over the re-encoded payload refused it and the session failed
/// to open.
#[test]
fn a_receipt_written_before_l7_validates_over_its_stored_payload() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let (id, key) = pre_l7_receipt_session(&backend).await;
        let stored = String::from_utf8(backend.get(&key).await.unwrap().unwrap()).unwrap();
        assert!(stored.contains("recognized_sub_kind") && stored.contains("\n  "));

        let session = Sessions::new_with(backend.clone())
            .with_legacy(&CURRENT)
            .open_with(id)
            .await
            .unwrap();
        let cited = session.graph().resources().persisted_edges_between(
            ResourceNode::for_term(TRANSLATED).id(),
            ResourceNode::for_term(BASELINE).id(),
        );
        assert_eq!(cited.len(), 1, "the pre-L7 edge reads");
        assert_eq!(cited[0].retained_assertions()[0].predicate, CITES);
        drop(session);

        // Control: a stored field today's types never read is still covered.
        let tampered = stored.replacen("\"Cites\"", "\"Quotes\"", 1);
        assert_ne!(tampered, stored);
        backend.put(&key, tampered.as_bytes()).await.unwrap();
        let forged = Sessions::new_with(backend.clone())
            .with_legacy(&CURRENT)
            .open_with(id)
            .await;
        assert!(
            matches!(&forged, Err(SessionError::Corrupt(why)) if why.contains("checksum")),
            "{:?}",
            forged.err()
        );
        // And the untouched receipt, restored, opens again.
        backend.put(&key, stored.as_bytes()).await.unwrap();
        assert!(
            Sessions::new_with(backend)
                .with_legacy(&CURRENT)
                .open_with(id)
                .await
                .is_ok()
        );
    });
}
