// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D163: the import-shaped paths refuse a change with a refused
//! edit whole, as `apply_now` does, and `revert` still reverts what it can.
use super::*;
use crate::test_surface::{PUBLICATION, show_edits};

#[derive(Clone, Copy, Debug)]
enum Path {
    ApplyDeltas,
    Transact,
    TransactPublished,
}

fn write(edits: Vec<Delta>) -> Vec<Delta> {
    edits
        .iter()
        .map(|edit| Surface::edit_of(edit, &person()).unwrap())
        .collect()
}

fn hooked(store: &MemoryBackend) -> Sessions {
    Sessions::new_with(store.clone()).with_publication(&PUBLICATION)
}

async fn run(session: &mut Session, path: Path, deltas: Vec<Delta>) -> Result<Applied, SessionError> {
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

fn held(session: &Session) -> Held {
    (
        whole(session.graph()),
        session.journal.live_cursor(),
        session.changes().len(),
        session.revision(),
    )
}

/// A change through `path` whose middle edit puts a held handle on another
/// relation is refused whole, naming the edit and the handle; the same change
/// with a fresh handle commits whole.
async fn refuses_whole(path: Path) {
    let store = MemoryBackend::new();
    let mere = hooked(&store);
    let (graph, handle) = related();
    let mut session = mere.begin_recorded_with(person(), Some(graph));
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
    assert_eq!(
        graph.iri_of(Uuid::from_u128(7)).as_deref(),
        Some("https://imported.test/")
    );
    assert!(graph.statement("fresh-import").is_some());
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

/// The resource surface `n` shows once added.
fn resource(n: u128) -> Uuid {
    show_edits(Uuid::from_u128(n), &format!("https://{n}.test/")).0
}

/// The relation between the resources surfaces `from` and `to` show, holding
/// one row with statement `handle`, or no rows.
fn relation(template: &Surface, from: u128, to: u128, handle: Option<&str>) -> Delta {
    let rows = handle
        .map(|handle| {
            let mut row = template.rows_between(Uuid::from_u128(1), Uuid::from_u128(2))[0].clone();
            for statement in row.assertions.as_mut().unwrap() {
                statement.statement_id = handle.into();
            }
            vec![row]
        })
        .unwrap_or_default();
    edges_edit(resource(from), resource(to), rows)
}

/// Undo still reverts what it can: an undo whose restoration of a statement
/// now collides with a handle another author took drops that part and
/// reverts the rest, as before D163.
#[test]
fn revert_still_reverts_what_it_can() {
    pollster::block_on(async {
        let mere = Sessions::new_with(MemoryBackend::new());
        let (template, _) = related();
        let mut baseline = template.clone();
        let surface_s = relation(&template, 1, 2, Some("surface-s"));
        SessionGraph::apply(&mut baseline, surface_s).unwrap();
        assert!(baseline.statement("surface-s").is_some());
        let mut session = mere.begin_with(person(), Some(baseline));

        // The person's change adds surface 7 and clears the pair holding `s`.
        let mut change = add(7);
        change.push(relation(&template, 1, 2, None));
        session.apply_now(person(), change).unwrap();
        assert!(session.graph().statement("surface-s").is_none());
        // Another author puts `s` on a new pair 3 -> 4.
        let mut change = add(3);
        change.extend(add(4));
        change.push(relation(&template, 3, 4, Some("surface-s")));
        session.apply_now(restorer(), change).unwrap();
        let taken = session.graph().statement("surface-s");
        assert!(taken.is_some(), "the other author holds `s`");

        let revision = session.revision();
        let reverted = session.undo_now(person()).unwrap().unwrap();
        assert!(reverted.applied.end > reverted.applied.first);
        assert_eq!(session.revision(), revision + 1);
        let graph = session.graph();
        assert!(
            graph.shown(Uuid::from_u128(7)).is_none(),
            "the undo removes surface 7"
        );
        let holder = graph.statement("surface-s");
        let pair = Some((resource(1), resource(2)));
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
        let mere = hooked(&store);
        let (graph, handle) = related();
        let mut session = mere.begin_recorded_with(person(), Some(graph));
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
        assert!(session.graph().statement("fresh-import").is_some());
        assert_eq!(
            session.operation_receipt(operation).await.unwrap(),
            Some(receipt)
        );
    });
}
