//! Bug-observation diagnostic: equivalent field stores refuse legacy qualification.
use kernel::graph::{Graph, Field, FieldId, FieldDefinition, ScalarField};
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
use muniment::Seq;
use pandect::{Author, MereSessions, SessionError};
use uuid::Uuid;

fn fields(count: u128) -> Graph {
    let mut graph = Graph::new();
    for n in 0..count {
        apply_graph_delta(&mut graph, GraphDelta::AddField {
            field: Field::new(FieldId::from_uuid(Uuid::from_u128(n + 1)), FieldDefinition::Scalar(ScalarField::Const(n as f32))),
        });
    }
    graph
}

fn normalized_fields(graph: &Graph) -> Vec<String> {
    let mut values: Vec<_> = graph.to_snapshot().fields.iter().map(|field| serde_json::to_string(field).unwrap()).collect();
    values.sort();
    values
}

#[test]
fn valid_field_stores_refuse_legacy_qualification_when_hash_iteration_order_changes() {
    pollster::block_on(async {
        for count in [0, 1] {
            let graph = fields(count);
            let expected = normalized_fields(&graph);
            let sessions = MereSessions::new(muniment::MemoryBackend::new());
            let mut session = sessions.begin(Author::user(), Some(graph));
            session.flush(std::time::SystemTime::UNIX_EPOCH).await.unwrap();
            let id = session.id();
            let mut opened = sessions.open(id).await.unwrap();
            opened.qualify_legacy(Seq(0), vec![]).await.expect("zero/one field positive controls");
            assert_eq!(normalized_fields(sessions.open(id).await.unwrap().graph()), expected);
        }
        let mut refused = 0;
        for _ in 0..8 {
            let graph = fields(8);
            let expected = normalized_fields(&graph);
            let sessions = MereSessions::new(muniment::MemoryBackend::new());
            let mut session = sessions.begin(Author::user(), Some(graph));
            session.flush(std::time::SystemTime::UNIX_EPOCH).await.unwrap();
            let mut opened = sessions.open(session.id()).await.unwrap();
            assert_eq!(normalized_fields(opened.graph()), expected, "valid baseline opens with all exact fields");
            match opened.qualify_legacy(Seq(0), vec![]).await {
                Err(SessionError::Corrupt(reason)) if reason.contains("legacy translation effects do not reproduce their frozen result") => {
                    refused += 1;
                    println!("valid eight-field baseline refused: {reason}");
                    assert!(opened.placement().is_none());
                    assert_eq!(normalized_fields(opened.graph()), expected, "refusal retains exact field truth");
                }
                Ok(()) => (),
                Err(error) => panic!("unexpected qualification failure: {error}"),
            }
        }
        assert!(refused > 0, "expected to observe the unordered comparison discrepancy across eight independently seeded field maps");
        println!("unordered field comparison rejected {refused}/8 valid eight-field baselines; zero/one-field controls qualify and reopen");
    });
}
