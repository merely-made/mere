// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use kernel::graph::{Author, SemanticStatementSpec, apply::add_node};
use mere::dataset::View;
use muniment::DirectoryBackend;
use pandect::MereSessions;
use std::{collections::BTreeSet, time::UNIX_EPOCH};
use uuid::Uuid;

fn has_predicate(pin: &pandect::PinnedView, predicate: &str) -> bool {
    pin.dataset()
        .quads(View::Live)
        .any(|q| q.predicate.as_str() == predicate)
}

fn rows(pin: &pandect::PinnedView, view: View) -> BTreeSet<String> {
    pin.dataset().quads(view).map(|q| q.to_string()).collect()
}

#[test]
fn assertion_edit_survives_reopen_at_the_same_pin() {
    pollster::block_on(async {
        let dir = tempfile::tempdir().unwrap();
        let sessions = MereSessions::new(DirectoryBackend::open(dir.path()).unwrap());
        let author = Author::person("p8-probe");
        let id = sessions
            .mint(author.clone(), None)
            .await
            .unwrap()
            .session_id;
        let mut session = sessions.open(id).await.unwrap();
        let (a, b) = (Uuid::from_u128(0x801), Uuid::from_u128(0x802));
        let ((old_id, old_quad), _) = session
            .edit_now(author.clone(), |graph| {
                let a = add_node(
                    graph,
                    Some(a),
                    "https://p8.test/a".into(),
                    Default::default(),
                );
                let b = add_node(
                    graph,
                    Some(b),
                    "https://p8.test/b".into(),
                    Default::default(),
                );
                let (_, old) = graph
                    .assert_semantic_statement(
                        a,
                        b,
                        SemanticStatementSpec {
                            predicate: "https://p8.test/withdrawn".into(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
                (old.statement_id, "https://p8.test/withdrawn")
            })
            .unwrap();
        let before = session.pin(session.journal().live_cursor()).unwrap();
        assert!(has_predicate(&before, old_quad));
        session
            .edit_now(author, |graph| {
                assert!(graph.withdraw_semantic_assertion(&old_id, Some(30), None));
                let a = graph.get_node_key_by_id(a).unwrap();
                let b = graph.get_node_key_by_id(b).unwrap();
                graph
                    .assert_semantic_statement(
                        a,
                        b,
                        SemanticStatementSpec {
                            predicate: "https://p8.test/retained".into(),
                            ..Default::default()
                        },
                    )
                    .unwrap();
            })
            .unwrap();
        session.flush(UNIX_EPOCH).await.unwrap();
        let pin = session.pin(session.journal().live_cursor()).unwrap();
        let identity = (pin.session(), pin.cursor(), pin.digest().to_owned());
        let live = rows(&pin, View::Live);
        let history = rows(&pin, View::History);
        assert!(!has_predicate(&pin, old_quad));
        assert!(has_predicate(&pin, "https://p8.test/retained"));
        assert_ne!(
            rows(&before, View::Live),
            live,
            "negative control: omitted edit"
        );
        assert_ne!(
            before.digest(),
            pin.digest(),
            "negative control: wrong cursor digest"
        );
        drop(session);
        drop(sessions);
        let reopened_sessions = MereSessions::new(DirectoryBackend::open(dir.path()).unwrap());
        let reopened = reopened_sessions.open(id).await.unwrap();
        let again = reopened.pin(reopened.journal().live_cursor()).unwrap();
        assert_eq!(
            (again.session(), again.cursor(), again.digest()),
            (identity.0, identity.1, identity.2.as_str())
        );
        assert_eq!(rows(&again, View::Live), live);
        assert_eq!(rows(&again, View::History), history);
        assert!(!has_predicate(&again, old_quad));
    });
}
