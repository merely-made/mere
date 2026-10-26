// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Legacy migration keys a carried handle on the claim the checked writer keys it on.
use super::apply::add_node;
use super::legacy_resource_migration::migrate_legacy_prefix;
use super::*;
use crate::persistence::PersistedEdge;
use crate::types::{AssertionBasis, AssertionMethod};
use uuid::Uuid;

fn pair() -> (Graph, NodeKey, NodeKey) {
    let mut graph = Graph::new();
    let mut add = |id: u128, url: &str| {
        add_node(
            &mut graph,
            Some(Uuid::from_u128(id)),
            url.into(),
            Default::default(),
        )
    };
    let (a, b) = (
        add(1, "https://reason.test/a"),
        add(2, "https://reason.test/b"),
    );
    (graph, a, b)
}

fn carried() -> PersistedEdge {
    let (mut graph, a, b) = pair();
    graph
        .try_assert_semantic_statement(
            a,
            b,
            SemanticStatementSpec {
                predicate: predicate_iri(SemanticSubKind::UserGrouped).into(),
                recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                provenance_iri: Some("urn:reason:author".into()),
                asserted_at_ms: Some(10),
                basis: AssertionBasis {
                    method: Some(AssertionMethod {
                        iri: "urn:reason:method:first".into(),
                        version: Some("1".into()),
                    }),
                    evidence: vec!["urn:reason:evidence:first".into()],
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .unwrap();
    graph.to_snapshot().edges.remove(0)
}

fn statement(edge: &PersistedEdge) -> SemanticStatement {
    serde_json::from_value(serde_json::to_value(&edge.retained_assertions()[0]).unwrap()).unwrap()
}

fn replace(edge: PersistedEdge) -> CapturedDelta {
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(1).to_string(),
        to_id: Uuid::from_u128(2).to_string(),
        edges: vec![edge],
    }
}

#[test]
fn legacy_migration_and_checked_writer_agree_which_reason_changes_a_carried_claim() {
    let edge = carried();
    let id = edge.retained_assertions()[0].statement_id.clone();
    for (change, same_reason) in [("method", false), ("evidence", false), ("version", true)] {
        let mut changed = edge.clone();
        let basis = &mut changed.assertions.as_mut().unwrap()[0].basis;
        match change {
            "method" => basis.method.as_mut().unwrap().iri = "urn:reason:method:other".into(),
            "evidence" => basis.evidence = vec!["urn:reason:evidence:other".into()],
            _ => basis.method.as_mut().unwrap().version = Some("2".into()),
        }

        let (mut checked, a, b) = pair();
        checked
            .try_assert_persisted_semantic_statement(a, b, statement(&edge))
            .unwrap();
        let written = checked.try_assert_persisted_semantic_statement(a, b, statement(&changed));

        let mut journal = GraphJournal::new();
        journal.record_as(Author::user(), replace(edge.clone()));
        journal.record(replace(changed.clone()));
        let migrated = migrate_legacy_prefix(&pair().0, journal.entries());

        assert_eq!(written.is_ok(), same_reason, "{change}: checked writer");
        assert_eq!(migrated.is_ok(), same_reason, "{change}: migration");
        match migrated {
            Ok(migrated) => {
                let stored = migrated.graph.to_snapshot().edges;
                let held = &stored[0].retained_assertions()[0];
                assert_eq!(held.statement_id, id);
                assert_eq!(held.basis, changed.retained_assertions()[0].basis);
            },
            Err(error) => assert!(error.0.contains(&id), "{change}: {error}"),
        }
    }
}
