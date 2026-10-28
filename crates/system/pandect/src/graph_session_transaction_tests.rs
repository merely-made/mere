// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::GraphStratum;
use kernel::graph::fixtures::GraphFixtures;
use muniment::MemoryBackend;

fn person() -> Author {
    Author::person("persona-a").via("turnstone")
}

fn add(id: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.test/"),
        position: [0.0, 0.0],
    }
}

#[test]
fn a_transaction_carries_a_direct_writer_through_the_replay_check() {
    pollster::block_on(async {
        let store = MemoryBackend::new();
        let mere = MereSessions::new(store.clone());
        let id = mere.mint(person(), None).await.unwrap().session_id;
        let mut session = mere.open(id).await.unwrap();
        session.apply(person(), vec![add(1)]).await.unwrap();
        let predicate = "urn:mere:receipt:transacted";
        let at = SystemTime::UNIX_EPOCH;

        session
            .transact_change(at, false, |candidate| {
                candidate
                    .edit_now(person(), |graph| {
                        graph
                            .declare_predicate(predicate, GraphStratum::Surface)
                            .unwrap()
                    })
                    .map(|_| ())
            })
            .await
            .unwrap();
        let declared = |graph: &Graph| graph.effective_predicate_stratum(predicate).unwrap();
        assert_eq!(declared(session.graph()), GraphStratum::Surface);
        assert!(!session.has_unstored());
        let reopened = mere.open(id).await.unwrap();
        assert_eq!(declared(reopened.graph()), GraphStratum::Surface);
        assert_eq!(reopened.journal().len(), session.journal().len());

        // A writer that skips the capture is refused, whole, and leaves no trace.
        let key = session
            .graph()
            .get_node_key_by_id(Uuid::from_u128(1))
            .unwrap();
        let before = (session.journal().len(), session.changes().len());
        let refused = session
            .transact_change(at, false, |candidate| {
                candidate
                    .edit_now(person(), |graph| {
                        graph.set_node_title(key, "Unrecorded".into());
                    })
                    .map(|_| ())
            })
            .await;
        assert!(matches!(refused, Err(SessionError::NotReplayable(_))));
        assert_eq!(
            (session.journal().len(), session.changes().len()),
            before
        );
        assert_ne!(
            session.graph().get_node(key).unwrap().title(),
            "Unrecorded"
        );
        let reopened = mere.open(id).await.unwrap();
        assert_eq!(reopened.journal().len(), before.0);
    });
}
