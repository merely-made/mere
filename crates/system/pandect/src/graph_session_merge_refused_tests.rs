// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Engine-split D169: a merge-in applies what it can and reports each edit
//! admission refused, with its position, part and reason.
use super::*;
use kernel::graph::node_facets::SEMANTIC_PROPERTIES;

/// Give `node` one literal whose statement handle is `handle`.
fn literal(node: Uuid, handle: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeFacetById {
        node_id: node.to_string(),
        facet: SEMANTIC_PROPERTIES.into(),
        value_json: serde_json::json!([property(handle, "x")]).to_string(),
    }
}

/// A branch that adds node 4 and gives node 3 a literal, merged into a target
/// that has since asserted a Resource statement 1 -> 2: with `collide`, the
/// literal takes that statement's handle. The handle, the plan's clean edits
/// and what the merge reports.
async fn merge_with(
    collide: bool,
) -> (
    GraphSession<MemoryBackend>,
    String,
    Vec<CapturedDelta>,
    Merged,
) {
    let mere = MereSessions::new(MemoryBackend::new());
    let mut target = parent(&mere).await;
    let mut branch = branch_of(&mere, &target).await;
    let (held, _) = target
        .edit_now(person(), |graph| {
            let key = |n| graph.get_node_key_by_id(id(n)).unwrap();
            let (from, to) = (key(1), key(2));
            let spec = kernel::graph::SemanticStatementSpec {
                predicate: "https://merge.test/vocab#next".into(),
                ..Default::default()
            };
            let (relation, _) = graph.assert_semantic_statement(from, to, spec).unwrap();
            graph.get_relation(relation).unwrap().assertions()[0]
                .statement_id
                .clone()
        })
        .unwrap();
    target.flush(SystemTime::now()).await.unwrap();
    let handle = if collide {
        held.clone()
    } else {
        "fresh".to_string()
    };
    branch
        .apply(
            other(),
            vec![add(4), literal(id(3), &handle), retitle(id(4), "Four")],
        )
        .await
        .unwrap();
    let plan = mere.merge_in(&target, &branch).await.unwrap();
    assert!(plan.is_clean(), "{:?}", plan.conflicts());
    let edits = plan.edits().to_vec();
    let merged = mere
        .apply_merge(&mut target, &mut branch, plan, &BTreeMap::new(), person())
        .await
        .unwrap();
    (target, handle, edits, merged)
}

fn holds(graph: &Graph, node: Uuid, handle: &str) -> bool {
    graph
        .facets()
        .get(&node, &FacetId::new(SEMANTIC_PROPERTIES))
        .is_some_and(|value| value.to_string().contains(handle))
}

#[test]
fn a_merge_reports_a_refused_part_and_applies_the_rest() {
    pollster::block_on(async {
        let (target, handle, edits, merged) = merge_with(true).await;
        let [refused] = merged.refused.as_slice() else {
            panic!("one refused edit: {:?}", merged.refused);
        };
        assert_eq!(
            refused.part,
            Some(Part::Facet(id(3), SEMANTIC_PROPERTIES.into()))
        );
        assert_eq!(edits[refused.position], literal(id(3), &handle));
        assert!(refused.reason.contains(&handle), "{}", refused.reason);
        assert!(
            refused.reason.contains("HandleCollision"),
            "{}",
            refused.reason
        );

        // The rest applied, as one change.
        assert!(merged.applied.end > merged.applied.first);
        assert_eq!(title(target.graph(), id(4)).as_deref(), Some("Four"));
        assert!(!holds(target.graph(), id(3), &handle), "the refused part");
        assert!(
            target.graph().find_semantic_statement(&handle).is_some(),
            "the holder keeps it"
        );
    });
}

#[test]
fn a_merge_with_nothing_refused_reports_nothing() {
    pollster::block_on(async {
        let (target, handle, _, merged) = merge_with(false).await;
        assert_eq!(merged.refused, Vec::new());
        assert_eq!(title(target.graph(), id(4)).as_deref(), Some("Four"));
        assert!(holds(target.graph(), id(3), &handle), "control: it applies");
    });
}
