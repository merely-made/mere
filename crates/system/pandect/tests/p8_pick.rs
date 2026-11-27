// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! pandect's pick over a graph with no kernel (engine split lane 4): a host
//! copies the part, and pandect makes the session, its name and its
//! `picked_from` record, as it did for mere-kernel's component fork.

mod support;

use std::sync::Arc;

use mere::model::Author;
use muniment::{Backend, MemoryBackend};
use pandect::{GraphSession, MereSessions, SessionError, SessionId};
use support::{Surface, show_edits};
use uuid::Uuid;

type Sessions = MereSessions<MemoryBackend, Surface>;

async fn minted(sessions: &Sessions) -> (SessionId, GraphSession<MemoryBackend, Surface>) {
    let id = sessions
        .mint(Author::person("p8-pick"), None)
        .await
        .unwrap()
        .session_id;
    (id, sessions.open_with(id).await.unwrap())
}

async fn stored(backend: &MemoryBackend) -> Vec<(String, Vec<u8>)> {
    let mut out = Vec::new();
    for key in backend.list("").await.unwrap() {
        let value = backend.get(&key).await.unwrap().unwrap();
        out.push((key, value));
    }
    out.sort();
    out
}

/// A part a host copied out of a session is picked off as its own session,
/// naming where it came from; a part the parent's validator refuses is not.
#[test]
fn a_part_is_picked_off_as_its_own_session_over_any_graph() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let sessions = Sessions::new_with(backend.clone());
        let (id, mut parent) = minted(&sessions).await;
        let author = Author::person("p8-pick");
        let (kept_surface, left_surface) = (Uuid::from_u128(0xc1), Uuid::from_u128(0xc2));
        let (kept, mut edits) = show_edits(kept_surface, "https://p8.test/kept");
        edits.extend(show_edits(left_surface, "https://p8.test/left").1);
        parent.apply(author.clone(), edits).await.unwrap();

        let mut part = Surface::default();
        part.show(kept_surface, "https://p8.test/kept");
        let container = pandect::GraphId::new();
        let picked = sessions
            .pick(&parent, container, part.clone(), author.clone())
            .await
            .unwrap();
        let from = picked
            .picked_from
            .as_ref()
            .expect("a pick names its source");
        assert_eq!(from.session, id);
        assert_eq!(from.at, parent.journal().live_cursor().0);
        assert_eq!(from.container, *parent.manifest().root_graph_id.as_uuid());
        assert_eq!(picked.root_graph_id, container);
        assert!(
            picked
                .display_name
                .as_deref()
                .is_some_and(|name| name.contains("(picked from")),
            "{:?}",
            picked.display_name
        );
        let opened = sessions.open_with(picked.session_id).await.unwrap();
        assert_eq!(opened.graph().shown(kept_surface), Some(kept));
        assert_eq!(opened.graph().shown(left_surface), None);

        // Control: a parent whose validator wants the left surface refuses
        // the part, and nothing is written.
        let parent = parent
            .with_validator(Arc::new(move |graph: &Surface| {
                graph
                    .shown(left_surface)
                    .map(|_| ())
                    .ok_or_else(|| "a part must keep the left surface".to_string())
            }))
            .unwrap();
        let before = stored(&backend).await;
        let refused = sessions
            .pick(&parent, pandect::GraphId::new(), part, author)
            .await;
        assert!(
            matches!(&refused, Err(SessionError::Validation(why)) if why.contains("left surface")),
            "{refused:?}"
        );
        assert_eq!(stored(&backend).await, before);
    });
}
