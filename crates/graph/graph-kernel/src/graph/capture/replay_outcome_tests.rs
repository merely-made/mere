// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use muniment::Seq;
use uuid::Uuid;

use super::{CapturedDelta, Replay, Unwritable};
use crate::graph::resource_content::ContentError;
use crate::graph::{
    Author, Graph, GraphJournal, ReplayRefused, replay_captured_deltas_as_onto,
    replay_captured_deltas_onto, replay_captured_deltas_onto_checked,
};

const FACET: &str = "audit/value";

fn id(n: u128) -> String {
    Uuid::from_u128(n).to_string()
}

fn add(n: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: id(n),
        url: format!("https://replay.test/{n}"),
        position: [0.0, 0.0],
    }
}

fn title(n: u128, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: id(n),
        title: title.into(),
    }
}

fn malformed(n: u128) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeFacetById {
        node_id: id(n),
        facet: FACET.into(),
        value_json: "{".into(),
    }
}

fn thumbnail(n: u128) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeThumbnailById {
        node_id: id(n),
        png_bytes: vec![1, 2, 3, 4],
        width: 2,
        height: 2,
    }
}

fn favicon(n: u128) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeFaviconById {
        node_id: id(n),
        rgba: vec![0; 4],
        width: 1,
        height: 1,
    }
}

fn journal(entries: impl IntoIterator<Item = CapturedDelta>) -> GraphJournal {
    let mut journal = GraphJournal::new();
    for entry in entries {
        journal.record(entry);
    }
    journal
}

fn title_of(graph: &Graph, n: u128) -> String {
    let key = graph.get_node_key_by_id(Uuid::from_u128(n)).expect("node");
    graph.get_node(key).expect("node").title.clone()
}

fn assert_names_the_facet(refused: &ReplayRefused, position: usize) {
    assert_eq!(refused.position, position);
    let ContentError::InvalidFacet(message) = &refused.reason else {
        panic!("expected InvalidFacet, got {:?}", refused.reason);
    };
    assert!(message.contains(FACET), "{message}");
    assert!(message.contains(&id(1)), "{message}");
}

#[test]
fn a_malformed_facet_record_replays_as_malformed_naming_node_facet_and_error() {
    let Replay::Malformed(bad) = malformed(1).replay() else {
        panic!("expected Malformed");
    };
    assert_eq!((bad.node_id, bad.facet), (id(1), FACET.to_string()));
    assert!(!bad.error.is_empty());
    assert!(matches!(
        malformed(1).replay_as(&Author::user()),
        Replay::Malformed(_)
    ));
    assert!(matches!(add(1).replay(), Replay::Apply(_)));
}

#[test]
fn obsolete_thumbnail_and_favicon_records_replay_as_an_explicit_skip() {
    for record in [thumbnail(1), favicon(1)] {
        assert!(matches!(record.replay(), Replay::Skip));
        assert!(matches!(record.replay_as(&Author::user()), Replay::Skip));
    }
}

#[test]
fn a_live_write_refuses_both_a_skip_and_a_malformed_record() {
    assert_eq!(
        thumbnail(1).replay().into_write().unwrap_err(),
        Unwritable::Skipped
    );
    assert!(matches!(
        malformed(1).replay().into_write(),
        Err(Unwritable::Malformed(_))
    ));
    assert!(add(1).replay().into_write().is_ok());
}

#[test]
fn snapshot_at_from_refuses_a_journal_entry_whose_facet_json_is_malformed() {
    let journal = journal([add(1), title(1, "before"), malformed(1), title(1, "after")]);
    let refused = journal
        .snapshot_at_from(&Graph::new(), journal.live_cursor())
        .err()
        .expect("refused");
    assert_names_the_facet(&refused, 2);
    assert!(refused.to_string().contains(FACET));
    let before = journal.snapshot_at(Seq(2)).unwrap().unwrap();
    assert_eq!(
        title_of(&before, 1),
        "before",
        "the prefix before it is sound"
    );
}

#[test]
fn a_refusal_position_counts_the_skipped_entries_before_it() {
    let journal = journal([add(1), thumbnail(1), favicon(1), malformed(1)]);
    let refused = journal
        .snapshot_at(journal.live_cursor())
        .err()
        .expect("refused");
    assert_names_the_facet(&refused, 3);
}

#[test]
fn journal_tail_replay_stops_at_a_malformed_facet_record() {
    let journal = journal([add(1), title(1, "before"), malformed(1), title(1, "after")]);
    let mut graph = journal.snapshot_at(Seq(1)).unwrap().unwrap();
    let refused = journal.replay_from(Seq(1), &mut graph).unwrap_err();
    assert_names_the_facet(&refused, 1);
    assert_eq!(
        title_of(&graph, 1),
        "before",
        "the graph holds the entries before it"
    );

    let mut graph = journal.snapshot_at(Seq(1)).unwrap().unwrap();
    let refused = journal
        .replay_from_with_baseline(Seq(1), &mut graph, &Graph::new())
        .unwrap_err();
    assert_names_the_facet(&refused, 1);
    assert_eq!(title_of(&graph, 1), "before");
}

#[test]
fn checked_replay_refuses_the_malformed_facet_record_a_tolerant_replay_skips() {
    let mut graph = Graph::new();
    replay_captured_deltas_onto(&mut graph, [add(1)]);
    let revision = graph.revision();
    let refused = replay_captured_deltas_onto_checked(&mut graph, [malformed(1)]).unwrap_err();
    assert_names_the_facet(&refused, 0);
    assert_eq!(graph.revision(), revision);
    assert!(graph.get_node_key_by_id(Uuid::from_u128(1)).is_some());
}

#[test]
fn tolerant_replay_drops_a_malformed_facet_record_and_continues() {
    let entries = [add(1), malformed(1), title(1, "after")];
    let journal = journal(entries.clone());
    assert_eq!(title_of(&journal.replay(), 1), "after");

    let mut graph = Graph::new();
    replay_captured_deltas_onto(&mut graph, entries.clone());
    assert_eq!(title_of(&graph, 1), "after");

    let mut graph = Graph::new();
    replay_captured_deltas_as_onto(&mut graph, &Author::user(), entries);
    assert_eq!(title_of(&graph, 1), "after");
}

#[test]
fn an_obsolete_thumbnail_record_is_skipped_silently_by_checked_and_tolerant_replay() {
    let entries = [add(1), thumbnail(1), favicon(1), title(1, "after")];
    let journal = journal(entries.clone());

    let checked = journal
        .snapshot_at(journal.live_cursor())
        .expect("a skip is not a refusal")
        .unwrap();
    assert_eq!(title_of(&checked, 1), "after");
    assert_eq!(title_of(&journal.replay(), 1), "after");

    let mut graph = journal.snapshot_at(Seq(1)).unwrap().unwrap();
    journal.replay_from(Seq(1), &mut graph).expect("tail skips");
    assert_eq!(title_of(&graph, 1), "after");
    let mut graph = journal.snapshot_at(Seq(1)).unwrap().unwrap();
    journal
        .replay_from_with_baseline(Seq(1), &mut graph, &Graph::new())
        .expect("attributed tail skips");
    assert_eq!(title_of(&graph, 1), "after");

    let mut graph = Graph::new();
    replay_captured_deltas_onto_checked(&mut graph, entries.clone()).expect("checked skips");
    assert_eq!(title_of(&graph, 1), "after");
    let mut graph = Graph::new();
    replay_captured_deltas_onto(&mut graph, entries);
    assert_eq!(title_of(&graph, 1), "after");
}
