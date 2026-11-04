// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::sync::{Arc, Mutex};

use super::*;
use crate::graph::{
    Coupling, CouplingResponse, EdgeAssertion, Field, FieldDefinition, NavigationTrigger,
    NodeSelector, ScalarField, SemanticSubKind,
};
use crate::types::{ImportRecord, ImportRecordMembership};

pub(in crate::graph) fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

pub(in crate::graph) fn add(n: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: id(n).to_string(),
        url: format!("https://{n}.test/"),
        position: [0.0, 0.0],
    }
}

pub(in crate::graph) fn title(n: u128, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: id(n).to_string(),
        title: title.to_string(),
    }
}

pub(in crate::graph) fn tag(n: u128, tag: &str) -> CapturedDelta {
    CapturedDelta::ReplayInsertNodeTagById {
        node_id: id(n).to_string(),
        tag: tag.to_string(),
    }
}

pub(in crate::graph) fn facet(n: u128, facet: &str, value: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeFacetById {
        node_id: id(n).to_string(),
        facet: facet.to_string(),
        value_json: value.to_string(),
    }
}

pub(in crate::graph) fn relate(from: u128, to: u128) -> CapturedDelta {
    CapturedDelta::ReplayAssertRelationByIds {
        from_id: id(from).to_string(),
        to_id: id(to).to_string(),
        assertion: EdgeAssertion::Semantic {
            sub_kind: SemanticSubKind::Cites,
            label: Some("cites".into()),
        },
    }
}

fn remove(n: u128) -> CapturedDelta {
    CapturedDelta::ReplayRemoveNodeById {
        node_id: id(n).to_string(),
    }
}

/// Run `edit` with a recorder on, and return what the graph recorded:
/// the change as a journal would hold it.
fn recorded(graph: &mut Graph, edit: impl FnOnce(&mut Graph)) -> Vec<CapturedDelta> {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    graph.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        sink.lock().unwrap().push(delta.clone());
    })));
    edit(graph);
    graph.set_recorder(None);
    seen.lock().unwrap().clone()
}

pub(in crate::graph) fn apply_all(graph: &mut Graph, edits: &[CapturedDelta]) {
    for edit in edits {
        let _ = apply_graph_delta(graph, edit.replay().into_write().expect("replayable"));
    }
}

type Nodes = Vec<(Uuid, Option<NodeState>)>;

/// Every node's state and every relation, so two graphs compare whole.
fn fingerprint(graph: &Graph) -> (Nodes, Vec<PersistedEdge>) {
    let mut ids: Vec<Uuid> = graph.nodes().map(|(_, node)| node.id).collect();
    ids.sort();
    let nodes = ids.iter().map(|n| (*n, node_state(graph, *n))).collect();
    let mut edges = Vec::new();
    for from in &ids {
        for to in &ids {
            edges.extend(edges_between(graph, *from, *to));
        }
    }
    (nodes, edges)
}

/// The graph before a change, the change as recorded, and the graph after.
fn scene(base: &[CapturedDelta], edits: &[CapturedDelta]) -> (Graph, Vec<CapturedDelta>, Graph) {
    let mut graph = Graph::new();
    apply_all(&mut graph, base);
    let before = graph.clone();
    let made = recorded(&mut graph, |graph| apply_all(graph, edits));
    (before, made, graph)
}

#[test]
fn a_retitle_reverts_unless_someone_retitled_since() {
    let (before, made, after) = scene(&[add(1), title(1, "A")], &[title(1, "B")]);
    let mut live = after.clone();
    let revert = revert_change(&made, &before, &after, &live);
    assert_eq!(revert.edits, [title(1, "A")]);
    assert!(revert.kept.is_empty());
    apply_all(&mut live, &revert.edits);
    assert_eq!(fingerprint(&live), fingerprint(&before));

    let mut retitled = after.clone();
    apply_all(&mut retitled, &[title(1, "C")]);
    let revert = revert_change(&made, &before, &after, &retitled);
    assert!(revert.edits.is_empty());
    assert_eq!(revert.kept, [Part::Title(id(1))]);
}

#[test]
fn a_made_node_goes_with_its_relations_unless_linked_since() {
    let (before, made, after) = scene(&[add(1)], &[add(2), title(2, "Two"), relate(1, 2)]);
    let mut live = after.clone();
    let revert = revert_change(&made, &before, &after, &live);
    assert_eq!(
        revert.edits,
        [remove(2)],
        "the node takes its relation with it"
    );
    apply_all(&mut live, &revert.edits);
    assert_eq!(fingerprint(&live), fingerprint(&before));

    let mut linked = after.clone();
    apply_all(&mut linked, &[add(3), relate(3, 2)]);
    let revert = revert_change(&made, &before, &after, &linked);
    assert!(
        revert.kept.contains(&Part::Node(id(2))),
        "{:?}",
        revert.kept
    );
    assert!(!revert.edits.contains(&remove(2)));
}

#[test]
fn a_removed_node_comes_back_whole() {
    let base = [
        add(1),
        add(2),
        title(2, "Two"),
        tag(2, "kept"),
        facet(2, "custom.note", r#"{"text":"hello"}"#),
        relate(1, 2),
    ];
    let (before, made, after) = scene(&base, &[remove(2)]);
    let mut live = after.clone();
    let revert = revert_change(&made, &before, &after, &live);
    assert!(revert.kept.is_empty(), "{:?}", revert.kept);
    apply_all(&mut live, &revert.edits);
    assert_eq!(fingerprint(&live), fingerprint(&before));
}

#[test]
fn a_restored_node_keeps_its_visit_stamp_across_a_clock_tick() {
    // Birth stamps visits with the clock. Make the node and the revert in
    // one millisecond and apply it in the next: the stamp must still be the
    // original, not the moment of the restore.
    let tick = |from: u64| while crate::time::unix_epoch_millis() == from {};
    let (before, after, revert, at) = (0..100)
        .find_map(|_| {
            tick(crate::time::unix_epoch_millis());
            let at = crate::time::unix_epoch_millis();
            let (before, made, after) = scene(&[add(1), add(2), relate(1, 2)], &[remove(2)]);
            let revert = revert_change(&made, &before, &after, &after);
            (crate::time::unix_epoch_millis() == at).then_some((before, after, revert, at))
        })
        .expect("a scene and its revert fit in one millisecond");
    tick(at);
    let mut live = after;
    apply_all(&mut live, &revert.edits);
    assert_eq!(fingerprint(&live), fingerprint(&before));
}

#[test]
fn recreation_restores_facets_when_the_birth_clock_changes() {
    for original_visit in [Some(90), None, Some(100)] {
        let mut before = Graph::new();
        apply_all(&mut before, &[add(1), add(2)]);
        let visit_facet = super::super::node_facets::VISIT_HISTORY;
        apply_all(
            &mut before,
            &[
                facet(
                    1,
                    visit_facet,
                    r#"{"last_visited_ms":42,"last_session_visited":0}"#,
                ),
                facet(2, "custom.note", r#"{"text":"original"}"#),
            ],
        );
        let original = match original_visit {
            Some(timestamp) => facet(
                2,
                visit_facet,
                &serde_json::json!({
                    "last_visited_ms": timestamp,
                    "last_session_visited": 0,
                })
                .to_string(),
            ),
            None => CapturedDelta::ReplayRemoveNodeFacetById {
                node_id: id(2).to_string(),
                facet: visit_facet.into(),
            },
        };
        apply_all(&mut before, &[original]);
        let was = node_state(&before, id(2)).unwrap();
        let mut scratch = Graph::new();
        apply_all(&mut scratch, &[add(2)]);
        let mut born = node_state(&scratch, id(2)).unwrap();
        born.facets.insert(
            visit_facet.into(),
            serde_json::json!({"last_visited_ms":100,"last_session_visited":0}),
        );
        let mut revert = Revert::default();
        revert.recreate_from_birth_state(id(2), &was, &born);

        let mut live = before.clone();
        apply_all(&mut live, &[remove(2)]);
        apply_all(&mut live, &revert.edits[..1]);
        // A later replay birth, even when original and planner both read 100.
        apply_all(
            &mut live,
            &[CapturedDelta::ReplayTouchNodeLastVisitedById {
                node_id: id(2).to_string(),
                timestamp_ms: 101,
            }],
        );
        apply_all(&mut live, &revert.edits[1..]);
        assert_eq!(
            fingerprint(&live),
            fingerprint(&before),
            "{original_visit:?}"
        );
        assert_eq!(
            node_state(&live, id(1)).unwrap().facets[visit_facet]["last_visited_ms"],
            42,
            "the untouched node keeps its own timestamp"
        );
    }
}

#[test]
fn tags_and_facets_revert_part_by_part() {
    let (before, made, after) = scene(
        &[add(1), facet(1, "custom.a", "1")],
        &[
            tag(1, "new"),
            facet(1, "custom.a", "2"),
            facet(1, "custom.b", "true"),
        ],
    );
    let mut live = after.clone();
    // Another author changes custom.b afterwards; the rest still reverts.
    apply_all(&mut live, &[facet(1, "custom.b", "false")]);
    let revert = revert_change(&made, &before, &after, &live);
    assert_eq!(revert.kept, [Part::Facet(id(1), "custom.b".into())]);
    apply_all(&mut live, &revert.edits);
    let state = node_state(&live, id(1)).unwrap();
    assert!(!state.tags.contains("new"));
    assert_eq!(state.facets.get("custom.a"), Some(&serde_json::json!(1)));
    assert_eq!(
        state.facets.get("custom.b"),
        Some(&serde_json::json!(false))
    );
    assert!(before.get_node_by_id(id(1)).is_some());
}

#[test]
fn an_undo_reverts_exactly_and_reverting_the_undo_redoes() {
    let traverse = CapturedDelta::ReplayAppendTraversalByIds {
        from_id: id(1).to_string(),
        to_id: id(2).to_string(),
        trigger: NavigationTrigger::LinkClick,
        timestamp_ms: 1_700_000_000_000,
    };
    let (before, made, after) = scene(&[add(1), add(2)], &[relate(1, 2), traverse]);
    let mut live = after.clone();
    let undo = revert_change(&made, &before, &after, &live);
    let undone = recorded(&mut live, |graph| apply_all(graph, &undo.edits));
    assert_eq!(fingerprint(&live), fingerprint(&before), "undo");

    // Redo is the undo reverted, with the undo's own before and after.
    let after_undo = live.clone();
    let redo = revert_change(&undone, &after, &after_undo, &live);
    assert!(redo.kept.is_empty(), "{:?}", redo.kept);
    apply_all(&mut live, &redo.edits);
    assert_eq!(fingerprint(&live), fingerprint(&after), "redo");
}

#[test]
fn the_field_layer_and_import_records_revert() {
    let field_id = FieldId::from_uuid(id(40));
    let coupling_id = CouplingId::from_uuid(id(41));
    let field = Field::new(field_id, FieldDefinition::Scalar(ScalarField::Const(1.0)));
    let coupling = Coupling::new(
        coupling_id,
        field_id,
        NodeSelector::Kind("paper".into()),
        CouplingResponse::DampenInside { factor: 0.3 },
        1.5,
    );
    let records = vec![ImportRecord {
        record_id: "import-record:one".into(),
        source_id: "import:one".into(),
        source_label: "One".into(),
        imported_at_secs: 1_763_500_800,
        memberships: vec![ImportRecordMembership {
            node_id: id(1).to_string(),
            suppressed: false,
        }],
    }];
    let mut graph = Graph::new();
    apply_all(&mut graph, &[add(1)]);
    let before = graph.clone();
    let made = recorded(&mut graph, |graph| {
        let _ = apply_graph_delta(graph, GraphDelta::AddField { field });
        let _ = apply_graph_delta(graph, GraphDelta::AddCoupling { coupling });
        let _ = apply_graph_delta(
            graph,
            GraphDelta::SetImportRecords {
                import_records: records,
            },
        );
    });
    let after = graph.clone();
    let revert = revert_change(&made, &before, &after, &graph);
    assert!(revert.kept.is_empty(), "{:?}", revert.kept);
    apply_all(&mut graph, &revert.edits);
    assert!(
        graph.field(field_id).is_none(),
        "the added field is gone, not retired"
    );
    assert_eq!(graph.couplings().count(), 0);
    assert!(graph.import_records().is_empty());
}

#[test]
fn touched_parts_reach_the_edits_that_name_them() {
    let touched = Touched::of(&[title(1, "A"), relate(2, 3)]);
    assert!(touched.reaches(&Part::Title(id(1))));
    assert!(touched.reaches(&Part::Edges(id(2), id(3))));
    assert!(
        touched.reaches(&Part::Node(id(2))),
        "a relation touches its ends"
    );
    assert!(!touched.reaches(&Part::Title(id(4))));
    assert!(!touched.reaches(&Part::ImportRecords));
}
