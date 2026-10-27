// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use kernel::graph::apply::add_node;
use muniment::MemoryBackend;

const PARENT: &str = "https://example.org/parent";
const CHILD: &str = "https://example.org/child";
const LATER: &str = "https://example.org/later";

/// A recorded session over a parent node and a child branched from its history,
/// the shape whose shared navigation sets used to serialize in hash order.
fn session_with_branched_child() -> (GraphSession<MemoryBackend>, kernel::graph::NodeKey, Author) {
    let mut graph = Graph::new();
    let parent = add_node(
        &mut graph,
        Some(Uuid::from_u128(1)),
        PARENT.into(),
        Default::default(),
    );
    apply_graph_delta(
        &mut graph,
        GraphDelta::NavigateNode {
            key: parent,
            url: PARENT.into(),
        },
    );
    let child = add_node(
        &mut graph,
        Some(Uuid::from_u128(2)),
        CHILD.into(),
        Default::default(),
    );
    apply_graph_delta(&mut graph, GraphDelta::BranchHistory { child, parent });
    apply_graph_delta(
        &mut graph,
        GraphDelta::NavigateNode {
            key: child,
            url: CHILD.into(),
        },
    );
    let manifest =
        GraphSessionManifest::new(SessionId(Uuid::from_u128(3)), GraphId(Uuid::from_u128(4)));
    let author = Author::person("navigation");
    let session = GraphSession::new_recorded(
        MemoryBackend::new(),
        manifest,
        Some(graph),
        author.clone(),
        ChangeKind::Minted,
    );
    (session, parent, author)
}

fn navigate(
    session: &mut GraphSession<MemoryBackend>,
    parent: kernel::graph::NodeKey,
    author: &Author,
) -> Result<(), SessionError> {
    session
        .edit_now(author.clone(), |graph| {
            apply_graph_delta(
                graph,
                GraphDelta::NavigateNode {
                    key: parent,
                    url: LATER.into(),
                },
            );
        })
        .map(|_| ())
}

#[test]
fn ordinary_navigations_are_never_refused_as_non_replayable() {
    // Each session hydrates its shared navigation sets into fresh hash tables, so
    // before the sets were written in index order a navigation was refused about
    // two times in three.
    for _ in 0..16 {
        let (mut session, parent, author) = session_with_branched_child();
        navigate(&mut session, parent, &author).expect("the navigation replays");
    }
}

#[test]
fn undoing_a_parent_navigation_keeps_the_branched_childs_history() {
    let (mut session, parent, author) = session_with_branched_child();
    navigate(&mut session, parent, &author).unwrap();

    let reverted = session
        .undo_now(author)
        .expect("undo is fallible, not a panic");

    assert!(reverted.is_some());
    assert_eq!(session.graph().get_node(parent).unwrap().url(), PARENT);
    assert!(session.graph().get_node_by_id(Uuid::from_u128(2)).is_some());
}
