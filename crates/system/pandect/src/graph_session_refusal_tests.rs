// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D163: the import-shaped paths refuse a change with a refused
//! edit whole, as `apply_now` does, and `revert` still reverts what it can.
use super::*;
use kernel::graph::RelationKey;

#[derive(Clone, Copy, Debug)]
enum Path {
    ApplyDeltas,
    Transact,
    TransactPublished,
}

fn write(edits: Vec<CapturedDelta>) -> Vec<GraphDelta> {
    edits
        .iter()
        .map(|edit| edit.replay_as(&person()).into_write().unwrap())
        .collect()
}

async fn run(
    session: &mut GraphSession<MemoryBackend>,
    path: Path,
    deltas: Vec<GraphDelta>,
) -> Result<Applied, SessionError> {
    let at = SystemTime::now();
    match path {
        Path::ApplyDeltas => session.apply_deltas(person(), deltas).await,
        Path::Transact => session.transact_deltas(person(), deltas, at).await,
        Path::TransactPublished => session
            .transact_deltas_published(person(), deltas, at)
            .await
            .map(|(applied, _)| applied),
    }
}

/// What a refused change leaves as it was.
type Held = ((serde_json::Value, serde_json::Value), Seq, usize, u64);

fn held(session: &GraphSession<MemoryBackend>) -> Held {
    (
        whole(session.graph()),
        session.journal.live_cursor(),
        session.changes().len(),
        session.revision(),
    )
}

/// A change through `path` whose middle edit puts a held handle on a Surface
/// relation is refused whole, naming the edit and the handle; the same change
/// with a fresh handle commits whole.
async fn refuses_whole(path: Path) {
    let store = MemoryBackend::new();
    let mere = MereSessions::new(store.clone());
    let (graph, handle) = related();
    let mut session = mere.begin_recorded(person(), Some(graph));
    session.flush(SystemTime::now()).await.unwrap();
    let before = held(&session);
    let stored = stored_bytes(&store).await;

    let edits = write(import(session.graph(), &handle));
    let Err(refused) = run(&mut session, path, edits).await else {
        panic!("{path:?}: an import taking a held handle is refused");
    };
    assert!(matches!(refused, SessionError::Validation(_)), "{path:?}");
    let message = refused.to_string();
    assert!(message.contains(&handle), "{path:?}: {message}");
    assert!(message.contains("edit 1"), "{path:?}: {message}");
    assert_eq!(held(&session), before, "{path:?}: nothing is applied");
    assert!(!session.has_unstored(), "{path:?}");
    assert_eq!(
        stored_bytes(&store).await,
        stored,
        "{path:?}: nothing is stored"
    );

    // Control: the same change with a fresh handle commits whole.
    let edits = write(import(session.graph(), "fresh-import"));
    let applied = run(&mut session, path, edits).await.unwrap();
    assert!(applied.end > applied.first, "{path:?}");
    assert_eq!(session.revision(), before.3 + 1, "{path:?}");
    let graph = session.graph();
    let imported = graph.get_node_key_by_id(Uuid::from_u128(7)).unwrap();
    assert_eq!(graph.get_node(imported).unwrap().title, "Imported");
    assert!(graph.find_semantic_statement("fresh-import").is_some());
    assert_ne!(stored_bytes(&store).await, stored, "{path:?}: stored");
}

#[test]
fn apply_deltas_refuses_a_change_whole() {
    pollster::block_on(refuses_whole(Path::ApplyDeltas));
}

#[test]
fn transact_deltas_refuses_a_change_whole() {
    pollster::block_on(refuses_whole(Path::Transact));
}

#[test]
fn transact_deltas_published_refuses_a_change_whole() {
    pollster::block_on(refuses_whole(Path::TransactPublished));
}

/// `ReplaySetEdgesByIds` for the Surface pair `from -> to` holding one row
/// with statement `handle`, or no rows.
fn surface(template: &Graph, from: u128, to: u128, handle: Option<&str>) -> CapturedDelta {
    let edges = handle
        .map(|handle| {
            let mut row = template.to_snapshot().resource_edges[0].clone();
            for statement in row.assertions.as_mut().unwrap() {
                statement.statement_id = handle.into();
            }
            vec![row]
        })
        .unwrap_or_default();
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(from).to_string(),
        to_id: Uuid::from_u128(to).to_string(),
        edges,
    }
}

/// Undo still reverts what it can: an undo whose restoration of a Surface
/// statement now collides with a handle another author took drops that part
/// and reverts the rest, as before D163.
#[test]
fn revert_still_reverts_what_it_can() {
    pollster::block_on(async {
        let mere = MereSessions::new(MemoryBackend::new());
        let (template, _) = related();
        let mut baseline = template.clone();
        let surface_s = surface(&template, 1, 2, Some("surface-s"));
        kernel::graph::replay_captured_deltas_onto(&mut baseline, [surface_s]);
        assert!(baseline.find_semantic_statement("surface-s").is_some());
        let mut session = mere.begin(person(), Some(baseline));

        // The person's change adds node 7 and clears the pair holding `s`.
        session
            .apply_now(person(), vec![add(7), surface(&template, 1, 2, None)])
            .unwrap();
        assert!(
            session
                .graph()
                .find_semantic_statement("surface-s")
                .is_none()
        );
        // Another author puts `s` on a new pair 3 -> 4.
        session
            .apply_now(
                restorer(),
                vec![add(3), add(4), surface(&template, 3, 4, Some("surface-s"))],
            )
            .unwrap();
        let taken = session.graph().find_semantic_statement("surface-s");
        assert!(taken.is_some(), "the other author holds `s`");

        let revision = session.revision();
        let reverted = session.undo_now(person()).unwrap().unwrap();
        assert!(reverted.applied.end > reverted.applied.first);
        assert_eq!(session.revision(), revision + 1);
        let graph = session.graph();
        assert!(
            graph.get_node_key_by_id(Uuid::from_u128(7)).is_none(),
            "the undo removes node 7"
        );
        let (one, two) = (
            graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap(),
            graph.get_node_key_by_id(Uuid::from_u128(2)).unwrap(),
        );
        let holder = graph.find_semantic_statement("surface-s").map(|(at, _)| at);
        let pair = graph.find_edge_key(one, two).map(RelationKey::Surface);
        assert!(holder.is_some(), "the other author still holds `s`");
        assert_ne!(
            holder, pair,
            "the colliding restoration is dropped, not refused"
        );
    });
}

/// The operation path is import-shaped: an operation whose middle edit takes
/// a held handle is refused whole, with no receipt; the control commits whole
/// with its receipt.
#[test]
fn transact_operation_published_refuses_an_operation_whole() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let (graph, handle) = related();
        let mut session = mere.begin_recorded(person(), Some(graph));
        session.flush(SystemTime::now()).await.unwrap();
        let before = held(&session);
        let stored = stored_bytes(&store).await;
        let operation = [7; 32];

        let edits = write(import(session.graph(), &handle));
        let refused = session
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"result".to_vec(),
                person(),
                edits,
                SystemTime::now(),
            )
            .await
            .unwrap_err();
        assert!(matches!(refused, SessionError::Validation(_)));
        let message = refused.to_string();
        assert!(message.contains(&handle), "{message}");
        assert!(message.contains("edit 1"), "{message}");
        assert_eq!(held(&session), before, "nothing is applied");
        assert_eq!(stored_bytes(&store).await, stored, "nothing is stored");
        assert!(
            session
                .operation_receipt(operation)
                .await
                .unwrap()
                .is_none()
        );

        // Control: the same operation with a fresh handle commits whole.
        let edits = write(import(session.graph(), "fresh-import"));
        let receipt = session
            .transact_operation_published(
                operation,
                b"request".to_vec(),
                b"result".to_vec(),
                person(),
                edits,
                SystemTime::now(),
            )
            .await
            .unwrap();
        assert_eq!(session.revision(), before.3 + 1);
        assert!(
            session
                .graph()
                .find_semantic_statement("fresh-import")
                .is_some()
        );
        assert_eq!(
            session.operation_receipt(operation).await.unwrap(),
            Some(receipt)
        );
    });
}

/// `transact_live` refuses a change whole (engine-split D166): commands whose
/// middle edit gives a Surface a literal holding a held handle are refused,
/// naming the edit and the handle; the same commands with a fresh handle
/// commit whole.
#[test]
fn transact_live_refuses_a_change_whole() {
    use crate::addressable_graph::GraphAddress;
    use crate::live_graph::{LiveCheck, LiveCommand, LiveEditError, ObservedCommand};
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let (graph, handle) = related();
        let mut session = mere.begin_recorded(person(), Some(graph));
        session.flush(SystemTime::now()).await.unwrap();
        let (one, nine) = (Uuid::from_u128(1), Uuid::from_u128(9));
        let digest = session
            .prepare_resident_publication()
            .unwrap()
            .record_digest(GraphAddress::Surface(one))
            .unwrap()
            .to_owned();
        let observed = vec![LiveCheck::RecordUnchanged {
            address: GraphAddress::Surface(one),
            digest,
        }];
        let commands = |statement: &str| {
            let mut property = kernel::types::NodeProperty::new(
                "https://live.test/vocab#note".to_string(),
                "held".to_string(),
            );
            property.statement_id = statement.into();
            vec![
                ObservedCommand {
                    command: LiveCommand::SetPinned {
                        surface: one,
                        pinned: true,
                    },
                    checks: observed.clone(),
                },
                ObservedCommand {
                    command: LiveCommand::SetFacet {
                        surface: one,
                        facet: kernel::graph::node_facets::SEMANTIC_PROPERTIES.into(),
                        value: serde_json::to_value([property]).unwrap(),
                    },
                    checks: observed.clone(),
                },
                ObservedCommand {
                    command: LiveCommand::CreateAddress {
                        id: nine,
                        address: "https://live.test/nine".into(),
                        title: "Nine".into(),
                    },
                    checks: vec![LiveCheck::Absent(GraphAddress::Surface(nine))],
                },
            ]
        };
        let before = held(&session);
        let stored = stored_bytes(&store).await;

        let refused = session
            .transact_live(person(), commands(&handle), SystemTime::now())
            .await
            .unwrap_err();
        assert!(
            matches!(refused, LiveEditError::Session(SessionError::Validation(_))),
            "{refused:?}"
        );
        let message = refused.to_string();
        assert!(message.contains(&handle), "{message}");
        assert!(message.contains("edit 1"), "{message}");
        assert_eq!(held(&session), before, "nothing is applied");
        assert_eq!(stored_bytes(&store).await, stored, "nothing is stored");

        // Control: the same commands with a fresh handle commit whole.
        let (live, _) = session
            .transact_live(person(), commands("fresh-live"), SystemTime::now())
            .await
            .unwrap();
        assert!(live.applied.end > live.applied.first);
        assert_eq!(live.created, vec![nine]);
        let graph = session.graph();
        let key = graph.get_node_key_by_id(one).unwrap();
        assert_eq!(graph.node_is_pinned(key), Some(true));
        assert!(graph.get_node_key_by_id(nine).is_some());
        assert_ne!(stored_bytes(&store).await, stored, "stored");
    });
}
