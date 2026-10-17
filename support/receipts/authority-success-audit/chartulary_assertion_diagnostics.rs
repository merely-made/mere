// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
// Diagnostic assertions deliberately describe observed defects, not desired behavior.

use chartulary::{Author, AssertionBasis, Container, EditSpec, GraphLog, LogId, Relation};
use chartulary::taxonomy::{Recognized, RelationClass};
use muniment::{JsonSlots, MemoryBackend};

fn relation() -> Relation {
    Relation::new(RelationClass::recognized(Recognized::Cites))
}

#[test]
fn accepted_lifecycle_operations_after_remove_reinsert_are_silently_ignored() {
    for withdraw in [false, true] {
        let author = Author::new("fixture");
        let mut graph = GraphLog::new();
        graph.insert_node(&author, Container::new("a"));
        graph.insert_node(&author, Container::new("b"));
        let id = graph.assert_relation(author.clone(), "a".into(), "b".into(),
            relation(), None, Some(10), AssertionBasis::default()).unwrap();
        let rev = graph.revision();
        let lifecycle = if withdraw {
            EditSpec::WithdrawAssertion { id, at_ms: Some(20) }
        } else {
            EditSpec::ConfirmAssertion { id, at_ms: Some(20) }
        };
        graph.commit_batch(author, rev, vec![
            EditSpec::RemoveNode("a".into()),
            EditSpec::InsertNode(Container::new("a")),
            lifecycle,
        ]).expect("preflight incorrectly accepts lifecycle on implicitly withdrawn assertion");
        let record = graph.assertion(id).unwrap();
        assert_eq!(graph.revision(), rev + 1);
        assert_eq!(record.span.last_asserted_ms, Some(10));
        assert_eq!(record.span.withdrawn.as_ref().unwrap().at_ms, None,
            "explicit requested withdrawal time was ignored");
        assert_eq!(graph.graph().edge_count(), 0);
    }
}

#[test]
fn equal_length_foreign_checkpoint_overrides_the_durable_journal() {
    pollster::block_on(async {
        let slots = JsonSlots::new(MemoryBackend::new());
        let author = Author::new("fixture");
        let mut durable = GraphLog::<Container, Relation>::with_id(LogId::new("durable"));
        durable.insert_node(&author, Container::new("durable-node"));
        durable.save_log(&slots, "log").await.unwrap();
        durable.snapshot(&slots, "correct").await.unwrap();
        let control = GraphLog::<Container, Relation>::load_checkpointed(&slots, "log", "correct").await.unwrap();
        assert!(control.graph().key_of(&"durable-node".into()).is_some());

        let mut foreign = GraphLog::<Container, Relation>::with_id(LogId::new("foreign"));
        foreign.insert_node(&author, Container::new("foreign-node"));
        assert_eq!(foreign.revision(), durable.revision());
        foreign.snapshot(&slots, "foreign").await.unwrap();
        let full = GraphLog::<Container, Relation>::load_full(&slots, "log").await.unwrap();
        let loaded = GraphLog::<Container, Relation>::load_checkpointed(&slots, "log", "foreign").await.unwrap();
        assert_eq!(loaded.id(), full.id());
        assert!(loaded.graph().key_of(&"durable-node".into()).is_none());
        assert!(loaded.graph().key_of(&"foreign-node".into()).is_some());
        assert!(full.graph().key_of(&"durable-node".into()).is_some());
        assert!(full.graph().key_of(&"foreign-node".into()).is_none());

        foreign.insert_node(&author, Container::new("ahead"));
        foreign.snapshot(&slots, "ahead").await.unwrap();
        assert!(GraphLog::<Container, Relation>::load_checkpointed(&slots, "log", "ahead").await.is_err(),
            "new upstream ahead-of-journal refusal is a positive control");
    });
}

#[test]
fn disconnect_of_an_edge_reaped_earlier_in_the_batch_still_reports_success() {
    let author = Author::new("fixture");
    let mut graph = GraphLog::new();
    graph.insert_node(&author, Container::new("a"));
    graph.insert_node(&author, Container::new("b"));
    let edge = graph.connect(&author, &"a".into(), &"b".into(), relation()).unwrap();
    let rev = graph.revision();
    assert!(graph.commit_batch(author, rev, vec![
        EditSpec::RemoveNode("a".into()), EditSpec::Disconnect(edge)
    ]).is_ok());
    assert_eq!(graph.revision(), rev + 1);
    assert_eq!(graph.graph().edge_count(), 0);
}
