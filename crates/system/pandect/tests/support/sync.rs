// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The surface's fold of the personal graph (engine split D259): nodes become
//! surfaces showing their address, statements of every family land on the
//! resources they show, and facets and scenes ride as surface facets. The
//! protocol, its identities and its bookkeeping stay pandect's.

use mere::model::{AssertionBasis, GraphScope, JournalDelta, SemanticStatement};
use mere::session::SessionGraph;
use pandect::access::{ACCESS_HISTORY_FACET, AccessHistory};
use pandect::personal_sync::{PersonalFold, PersonalGraphError, PersonalGraphEvent, RelationClaim};
use uuid::Uuid;

use super::{Surface, SurfaceDelta};

/// The surface's fold.
pub struct StatementFold;

/// The exact statement an assertion event names, under its asserter.
fn statement(
    relation: &RelationClaim,
    asserter: &str,
    id: &str,
    at_ms: Option<u64>,
    basis: Option<AssertionBasis>,
) -> SemanticStatement {
    let label = relation.label.clone();
    let basis = basis.unwrap_or_else(|| AssertionBasis {
        note: label.clone(),
        ..Default::default()
    });
    SemanticStatement {
        statement_id: id.to_owned(),
        predicate: relation.predicate.clone(),
        label: basis.note.clone().or(label),
        basis,
        graph_scope: GraphScope::Default,
        provenance_iri: Some(asserter.to_owned()),
        asserted_at_ms: at_ms,
        first_asserted_ms: at_ms,
        withdrawn: None,
    }
}

fn set_facet(graph: &mut Surface, node: Uuid, facet: &str, value: serde_json::Value) {
    graph.set_facet(node, facet, value);
}

impl PersonalFold<Surface> for StatementFold {
    fn apply(
        &self,
        graph: &mut Surface,
        asserter_iri: &str,
        fallback_id: &str,
        event: &PersonalGraphEvent,
    ) -> Result<(), PersonalGraphError> {
        match event {
            PersonalGraphEvent::AddNode { id, address, .. } => {
                if graph.shown(*id).is_none() {
                    graph.show(*id, address);
                }
            },
            PersonalGraphEvent::RemoveNode { id } => {
                let hide = JournalDelta::Surface(SurfaceDelta::HideSurface { surface_id: *id });
                graph.apply(hide).map_err(PersonalGraphError::Excluded)?;
            },
            PersonalGraphEvent::AssertRelation {
                from,
                to,
                relation,
                assertion_version,
                statement_id,
                asserted_at_ms,
                basis,
            } => {
                if matches!(assertion_version, Some(1)) || relation.is_semantic() {
                    let id = statement_id.as_deref().unwrap_or(fallback_id);
                    let exact =
                        statement(relation, asserter_iri, id, *asserted_at_ms, basis.clone());
                    graph
                        .upsert_statement(*from, *to, exact)
                        .map_err(PersonalGraphError::Excluded)?;
                } else {
                    graph.assert_relation(
                        *from,
                        *to,
                        &relation.predicate,
                        relation.label.clone(),
                        asserter_iri,
                    );
                }
            },
            PersonalGraphEvent::RetractRelation { from, to, selector } => {
                graph.withdraw_matching(*from, *to, selector, asserter_iri);
            },
            PersonalGraphEvent::RetractAssertion {
                statement_id,
                withdrawn_at_ms,
                ..
            } => {
                graph.withdraw_statement(
                    statement_id,
                    *withdrawn_at_ms,
                    Some(asserter_iri.to_owned()),
                );
            },
            PersonalGraphEvent::SetFacet { node, facet, value } => {
                set_facet(graph, *node, facet, value.clone());
            },
            PersonalGraphEvent::SaveScene { node, scene } => {
                let value = serde_json::to_value(scene).expect("saved scene always serializes");
                set_facet(graph, *node, pandect::SAVED_SCENE_FACET_V3, value);
            },
            _ => {},
        }
        Ok(())
    }

    fn set_access_history(&self, graph: &mut Surface, node: Uuid, history: AccessHistory) {
        let value = serde_json::to_value(history).expect("access history always serializes");
        set_facet(graph, node, ACCESS_HISTORY_FACET, value);
    }
}
