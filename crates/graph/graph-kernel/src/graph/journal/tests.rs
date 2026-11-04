// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use crate::graph::capture::{replay_captured_deltas, replay_captured_deltas_onto};
use crate::graph::set_captured_delta_hook;
use crate::graph::{EdgeAssertion, SemanticSubKind, SourceExtent, SourceTime};
use euclid::default::Point2D;
use uuid::Uuid;

#[test]
fn assertion_attribution_ignores_automated_versions_and_recording_application() {
    for kind in [AuthorKind::Rule, AuthorKind::Script, AuthorKind::Engine] {
        let v1 = Author::new(kind, "extractor", Some("1".into())).via("turnstone");
        let v2 = Author::new(kind, "extractor", Some("2".into())).via("knot");
        assert_ne!(v1, v2, "the journal keeps version and application metadata");
        assert_eq!(v1.asserter_iri(), v2.asserter_iri());
        assert_ne!(
            v1.asserter_iri(),
            Author::new(kind, "another-extractor", Some("1".into())).asserter_iri(),
            "different names still identify different asserters"
        );
    }
    let kinds = [
        Author::person("extractor"),
        Author::rule("extractor", "1"),
        Author::script("extractor", "1"),
        Author::engine("extractor", "1"),
    ];
    let identities: std::collections::BTreeSet<_> =
        kinds.iter().map(Author::asserter_iri).collect();
    assert_eq!(identities.len(), kinds.len());
}

#[test]
fn assertion_attribution_keeps_person_iris_and_encodes_opaque_ids() {
    for iri in [
        "https://people.test/alice",
        "did:key:z6MkAlice",
        "urn:uuid:058e1fe2-851b-4f1c-bbf0-0cbf74fe7727",
    ] {
        assert_eq!(Author::person(iri).asserter_iri(), iri);
    }
    let person = Author::person("alice /+?#%名字");
    assert_eq!(
        person.asserter_iri(),
        "https://mere.computer/ns/agent#person/alice+%2F%2B%3F%23%25%E5%90%8D%E5%AD%97"
    );
    assert_eq!(
        person.asserter_iri(),
        person.clone().via("knot").asserter_iri()
    );
    assert!(url::Url::parse(&person.asserter_iri()).is_ok());
    assert_ne!(
        Author::person("alice /+?#%名字").asserter_iri(),
        Author::person("alice +?#%名字").asserter_iri()
    );
    assert_ne!(
        Author::person(".").asserter_iri(),
        Author::person("..").asserter_iri(),
        "opaque dot segments are preserved in the fragment"
    );
    assert_eq!(
        Author::person(" https://people.test/alice ").asserter_iri(),
        "https://mere.computer/ns/agent#person/+https%3A%2F%2Fpeople.test%2Falice+"
    );
}

fn add(id: u128, url: &str) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: url.to_string(),
        position: [0.0, 0.0],
    }
}

fn assert_live_add_captures(deltas: &[CapturedDelta], id: u128, url: &str) {
    let resource = super::super::ResourceNode::new(url);
    assert_eq!(deltas.len(), 4);
    assert_eq!(deltas[0], add(id, url));
    assert!(
        matches!(&deltas[1], CapturedDelta::ReplayTouchNodeLastVisitedById {
        node_id, ..
    } if node_id == &Uuid::from_u128(id).to_string())
    );
    assert_eq!(
        deltas[2],
        CapturedDelta::ReplaySetResourceRecordById {
            resource_id: resource.id().to_string(),
            record: Some(crate::persistence::PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().into(),
                facets: vec![],
            }),
        }
    );
    assert_eq!(
        deltas[3],
        CapturedDelta::ReplaySetShownResourceById {
            surface_id: Uuid::from_u128(id).to_string(),
            resource_id: Some(resource.id().to_string()),
        }
    );
}

/// A key-independent view: sorted node ids and sorted (from, to, kind) triples.
/// Two graphs are equal iff their fingerprints are, regardless of petgraph keys.
fn fingerprint(graph: &Graph) -> (Vec<String>, Vec<(String, String, String)>) {
    let mut nodes: Vec<String> = graph.nodes().map(|(_, n)| n.id.to_string()).collect();
    nodes.sort();
    let mut edges: Vec<(String, String, String)> = graph
        .relations()
        .map(|rel| {
            let from = graph
                .get_node(rel.from)
                .map(|n| n.id.to_string())
                .unwrap_or_default();
            let to = graph
                .get_node(rel.to)
                .map(|n| n.id.to_string())
                .unwrap_or_default();
            (from, to, format!("{:?}", rel.kind))
        })
        .collect();
    edges.sort();
    (nodes, edges)
}

/// The envelope: entries carry their author; the default is the trusted
/// UI's person, a scoped author attributes a participant run, and replay
/// strips the envelope.
#[test]
fn entries_are_attributed_and_author_scoping_works() {
    let mut journal = GraphJournal::new();
    journal.record(add(1, "https://a.test/"));
    journal.set_author(Author::person("aa11"));
    journal.record(add(2, "https://b.test/"));
    journal.set_author(Author::user());
    journal.record_as(
        Author::engine("tarot-shuffle", "1.2").via("cleromancy"),
        add(3, "https://c.test/"),
    );

    let authors: Vec<String> = journal
        .entries()
        .iter()
        .map(|e| e.author.to_string())
        .collect();
    assert_eq!(
        authors,
        [
            "person user",
            "person aa11",
            "engine tarot-shuffle 1.2 via cleromancy"
        ]
    );
    assert_eq!(journal.author(), &Author::user(), "scoping restored");
    assert_eq!(
        journal.replay().node_count(),
        3,
        "replay strips the envelope"
    );
}

#[test]
fn source_time_contract_selects_immutable_journal_prefixes() {
    let mut journal = GraphJournal::new();
    journal.record(add(1, "https://one.test/"));
    journal.record(add(2, "https://two.test/"));

    assert_eq!(
        journal.source_extent(),
        SourceExtent {
            earliest: Seq(0),
            current: Seq(2),
        }
    );
    assert_eq!(journal.source_ticks(3), [Seq(0), Seq(1), Seq(2)]);
    assert_eq!(
        journal
            .source_snapshot(&Seq(1))
            .expect("known source cursor")
            .node_count(),
        1
    );
    assert!(journal.source_snapshot(&Seq(3)).is_none());
    assert_eq!(
        journal.replay().node_count(),
        2,
        "scrubbing leaves live truth intact"
    );
}

/// Every author field is written, so the record reads back through any
/// codec, positional ones included.
#[test]
fn an_author_writes_every_field() {
    let author = Author::rule("promote-on-tag", "3").via("turnstone");
    let json = serde_json::to_string(&author).unwrap();
    assert_eq!(
        json,
        r#"{"kind":"rule","id":"promote-on-tag","version":"3","via":"turnstone"}"#
    );
    assert_eq!(serde_json::from_str::<Author>(&json).unwrap(), author);
    let person = serde_json::to_string(&Author::user()).unwrap();
    assert_eq!(
        person,
        r#"{"kind":"person","id":"user","version":null,"via":null}"#
    );
}

/// A graph that opted in records its own deltas; a clone of it records
/// nothing, and neither does a graph that never opted in.
#[test]
fn a_graph_records_its_own_deltas_and_a_clone_does_not() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let mut session = Graph::new();
    session.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    assert!(session.is_recording());
    let add_node = |graph: &mut Graph, id: u128| {
        apply_graph_delta(
            graph,
            GraphDelta::AddNode {
                id: Some(Uuid::from_u128(id)),
                url: format!("https://{id}.test/"),
                position: Point2D::new(0.0, 0.0),
            },
        )
    };
    add_node(&mut session, 1);
    let mut copy = session.clone();
    assert!(!copy.is_recording());
    add_node(&mut copy, 2);
    add_node(&mut Graph::new(), 3);
    let seen = seen.lock().unwrap();
    assert_live_add_captures(&seen, 1, "https://1.test/");
}

#[test]
fn fork_carries_history_then_diverges() {
    let mut source = GraphJournal::with_id(LogId::new("source"));
    source.record(add(1, "https://a.test/"));
    source.record(add(2, "https://b.test/"));

    let mut fork = source.fork(LogId::new("fork"));
    assert_eq!(
        fork.entries(),
        source.entries(),
        "the fork copies the history"
    );
    let provenance = fork.provenance().expect("a fork has provenance");
    assert_eq!(provenance.source, Some(LogId::new("source")));
    assert_eq!(provenance.at, Seq(source.len() as u64));

    // Diverging the fork leaves the source untouched.
    fork.record(add(3, "https://c.test/"));
    assert_eq!(source.replay().node_count(), 2, "source unchanged");
    assert_eq!(fork.replay().node_count(), 3);
}

#[test]
fn journal_round_trips_through_a_muniment_slot() {
    use muniment::{JsonSlots, MemoryBackend};
    pollster::block_on(async {
        let slots = JsonSlots::new(MemoryBackend::new());

        let mut journal = GraphJournal::new();
        journal.record(add(1, "https://a.test/"));
        journal.record(add(2, "https://b.test/"));
        journal.save(&slots, "journal").await.unwrap();

        let reloaded = GraphJournal::load(&slots, "journal").await.unwrap();
        assert_eq!(reloaded.len(), 2, "the log survived the round trip");
        assert_eq!(
            fingerprint(&reloaded.replay()),
            fingerprint(&journal.replay()),
            "the reloaded journal replays to the same graph"
        );
    });
}

#[test]
fn the_capture_hook_feeds_the_journal_and_replay_matches_live() {
    // The edit-spine invariant: a graph built by live mutation and one rebuilt
    // by replaying the journal those mutations produced are identical.
    let (journal, hook) = journal_capture_hook();
    set_captured_delta_hook(Some(hook));

    let mut live = Graph::new();
    let a = match apply_graph_delta(
        &mut live,
        GraphDelta::AddNode {
            id: Some(Uuid::from_u128(1)),
            url: "https://a.test/".to_string(),
            position: Point2D::new(0.0, 0.0),
        },
    ) {
        GraphDeltaResult::NodeAdded(key) => key,
        other => panic!("expected NodeAdded, got {other:?}"),
    };
    let b = match apply_graph_delta(
        &mut live,
        GraphDelta::AddNode {
            id: Some(Uuid::from_u128(2)),
            url: "https://b.test/".to_string(),
            position: Point2D::new(1.0, 0.0),
        },
    ) {
        GraphDeltaResult::NodeAdded(key) => key,
        other => panic!("expected NodeAdded, got {other:?}"),
    };
    apply_graph_delta(
        &mut live,
        GraphDelta::AssertRelation {
            asserter_iri: crate::graph::journal::Author::user().asserter_iri(),
            from: a,
            to: b,
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
            },
        },
    );
    apply_graph_delta(
        &mut live,
        GraphDelta::SetNodeTitle {
            key: a,
            title: "Paper A".to_string(),
        },
    );

    set_captured_delta_hook(None);

    let replayed = journal.lock().unwrap().replay();
    assert_eq!(
        fingerprint(&replayed),
        fingerprint(&live),
        "replay of the captured journal reconstructs the live graph"
    );
    let (_, node) = replayed.get_node_by_id(Uuid::from_u128(1)).expect("node a");
    assert_eq!(node.title, "Paper A");
}

#[path = "tests/attribution.rs"]
mod attribution;

#[path = "tests/replay.rs"]
mod replay;
