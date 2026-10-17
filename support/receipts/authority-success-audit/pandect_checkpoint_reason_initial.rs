//! Observations of audit defects, not desired-behavior regression tests.
use kernel::graph::{Graph, SemanticStatement, SemanticStatementSpec, SemanticSubKind, predicate_iri};
use kernel::graph::apply::add_node;
use kernel::persistence::PersistedEdge;
use kernel::types::{AssertionBasis, AssertionMethod};
use muniment::{Backend, WriteOp};
use pandect::{Author, CapturedDelta, MereSessions, Seq};
use uuid::Uuid;

fn baseline() -> Graph {
    let mut graph = Graph::new();
    for (id, url) in [(1, "https://first.audit.test/"), (2, "https://second.audit.test/")] {
        add_node(&mut graph, Some(Uuid::from_u128(id)), url.into(), Default::default());
    }
    graph
}

fn capture(edge: PersistedEdge) -> CapturedDelta {
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: edge.from_node_id.clone(),
        to_id: edge.to_node_id.clone(),
        edges: vec![edge],
    }
}

#[test]
fn recorded_session_accepts_foreign_checkpoint_at_its_own_valid_cursor() {
    pollster::block_on(async {
        let backend = muniment::MemoryBackend::new();
        let sessions = MereSessions::new(backend.clone());
        let author = Author::user();
        let mut session = sessions.begin_recorded(author.clone(), Some(baseline()));
        session.apply(author, vec![CapturedDelta::ReplaySetNodeTitleById {
            node_id: Uuid::from_u128(1).to_string(), title: "authoritative title".into(),
        }]).await.unwrap();
        session.checkpoint().await.unwrap();
        let id = session.id();
        let cursor = session.journal().live_cursor();
        let control = sessions.open(id).await.unwrap();
        assert_eq!(control.graph().get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "authoritative title");

        let mut foreign = Graph::new();
        add_node(&mut foreign, Some(Uuid::from_u128(999)), "https://foreign.audit.test/".into(), Default::default());
        let prefix = format!("sessions/{}", id.as_uuid());
        backend.apply(&[
            WriteOp::Put { key: format!("{prefix}/graph.json"), value: serde_json::to_vec(&foreign.to_snapshot()).unwrap() },
            WriteOp::Put { key: format!("{prefix}/facets.json"), value: serde_json::to_vec(foreign.facets()).unwrap() },
        ]).await.unwrap();
        let opened = sessions.open(id).await.expect("foreign checkpoint is accepted");
        assert_eq!(opened.id(), id);
        assert_eq!(opened.journal().live_cursor(), cursor);
        assert!(opened.graph().get_node_by_id(Uuid::from_u128(999)).is_some());
        assert!(opened.graph().get_node_by_id(Uuid::from_u128(1)).is_none());
        let replayed = opened.graph_at(cursor).unwrap();
        assert!(replayed.get_node_by_id(Uuid::from_u128(999)).is_none());
        assert_eq!(replayed.get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "authoritative title");

        backend.apply(&[WriteOp::Put {
            key: format!("{prefix}/checkpoint.json"),
            value: serde_json::to_vec(&serde_json::json!({"cursor":Seq(cursor.0 + 1)})).unwrap(),
        }]).await.unwrap();
        let ahead = sessions.open(id).await.unwrap();
        assert!(ahead.graph().get_node_by_id(Uuid::from_u128(999)).is_none(), "ahead checkpoints are ignored as a control");
        assert_eq!(ahead.graph().get_node_by_id(Uuid::from_u128(1)).unwrap().1.title, "authoritative title");
    });
}

#[test]
fn durable_legacy_qualification_accepts_a_changed_reason_on_a_carried_handle() {
    pollster::block_on(async {
        for change in ["method", "evidence"] {
            let base = baseline();
            let mut template = base.clone();
            let from = template.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
            let to = template.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
            template.try_assert_semantic_statement(from, to, SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
                recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                provenance_iri: Some("urn:audit:person".into()),
                asserted_at_ms: Some(10),
                basis: AssertionBasis {
                    method: Some(AssertionMethod { iri: "urn:audit:method:first".into(), version: Some("1".into()) }),
                    evidence: vec!["urn:audit:evidence:first".into()],
                    ..Default::default()
                },
                ..Default::default()
            }).unwrap();
            let original = template.to_snapshot().edges[0].clone();
            let mut changed = original.clone();
            let basis = &mut changed.assertions.as_mut().unwrap()[0].basis;
            if change == "method" { basis.method.as_mut().unwrap().iri = "urn:audit:method:changed".into(); }
            else { basis.evidence = vec!["urn:audit:evidence:changed".into()]; }
            let carried: SemanticStatement = serde_json::from_value(serde_json::to_value(&changed.retained_assertions()[0]).unwrap()).unwrap();
            assert!(template.try_assert_persisted_semantic_statement(from, to, carried).is_err(), "checked writer refuses reason change");

            let sessions = MereSessions::new(muniment::MemoryBackend::new());
            let author = Author::user();
            let mut session = sessions.begin(author.clone(), Some(base));
            session.apply(author, vec![capture(original.clone()), capture(changed.clone())]).await.unwrap();
            let id = session.id();
            let mut opened = sessions.open(id).await.unwrap();
            let cursor = opened.journal().live_cursor();
            opened.qualify_legacy(cursor, vec![]).await.expect("qualification accepts changed reason");
            let reopened = sessions.open(id).await.expect("durable translation reopens");
            let stored = reopened.graph().find_assertion(&original.retained_assertions()[0].statement_id).unwrap().1;
            assert_eq!(stored.basis, changed.retained_assertions()[0].basis);
            assert!(reopened.placement().is_some());
            assert!(reopened.graph_at(cursor).is_some());
        }
    });
}
