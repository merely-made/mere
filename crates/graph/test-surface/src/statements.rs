// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Statements of every relation family between the resources two surfaces
//! show, written whole through the store: what a host fold of the personal
//! graph writes (engine split D259).

use mere::model::{
    AssertionBasis, AssertionWithdrawal, EdgePayload, GraphScope, JournalDelta, RelationSelector,
    SemanticStatement, Vocabulary,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::Surface;

impl Surface {
    /// Rewrite the pair between the resources surfaces `from` and `to` show
    /// by `edit`, recorded; whether it changed.
    pub(crate) fn rewrite(
        &mut self,
        from: Uuid,
        to: Uuid,
        mut edit: impl FnMut(&mut EdgePayload) -> bool,
    ) -> bool {
        let Some((a, b)) = self.shown(from).zip(self.shown(to)) else {
            return false;
        };
        let mut payloads = self.payloads(a, b);
        // Every payload is offered the edit; none short-circuits the rest.
        let mut changed = false;
        for payload in &mut payloads {
            changed |= edit(payload);
        }
        if !changed {
            return false;
        }
        match self.resources.replace_edges(a, b, payloads) {
            Some(written) => {
                self.emit(JournalDelta::Resource(written));
                true
            },
            None => false,
        }
    }

    /// Hold `statement` between the resources surfaces `from` and `to` show,
    /// replacing a held one of its handle.
    ///
    /// # Errors
    /// When either surface shows nothing, or another pair holds the handle.
    pub fn upsert_statement(
        &mut self,
        from: Uuid,
        to: Uuid,
        statement: SemanticStatement,
    ) -> Result<(), String> {
        let (a, b) = self
            .shown(from)
            .zip(self.shown(to))
            .ok_or("exact relation assertion references a missing endpoint")?;
        if self
            .statement(&statement.statement_id)
            .is_some_and(|pair| pair != (a, b))
        {
            return Err(format!(
                "assertion handle {:?} is held on another pair",
                statement.statement_id
            ));
        }
        let mut statement = Some(statement);
        self.rewrite(from, to, |payload| {
            statement
                .take()
                .is_some_and(|statement| payload.upsert_persisted_semantic_statement(statement))
        });
        Ok(())
    }

    /// The legacy membership ensure: `predicate` held once (a semantic one
    /// with `label`), named by `asserter`, with no recorded time.
    pub fn assert_relation(
        &mut self,
        from: Uuid,
        to: Uuid,
        predicate: &str,
        label: Option<String>,
        asserter: &str,
    ) -> bool {
        let mut label = Some(label);
        self.rewrite(from, to, |payload| {
            label.take().is_some_and(|label| {
                payload.assert_relation_with_context(
                    predicate,
                    label,
                    Some(asserter.to_owned()),
                    None,
                )
            })
        })
    }

    /// Withdraw statement `id` wherever it is held, at `at_ms`, by `by`.
    pub fn withdraw_statement(&mut self, id: &str, at_ms: Option<u64>, by: Option<String>) -> bool {
        let Some((a, b)) = self.statement(id) else {
            return false;
        };
        let surfaces = |resource| {
            self.shown
                .iter()
                .find(|(_, shown)| **shown == resource)
                .map(|(surface, _)| *surface)
        };
        let Some((from, to)) = surfaces(a).zip(surfaces(b)) else {
            return false;
        };
        self.rewrite(from, to, |payload| {
            payload.withdraw_semantic_statement(
                id,
                AssertionWithdrawal {
                    at_ms,
                    by: by.clone(),
                },
            )
        })
    }

    /// Withdraw every statement `asserter` holds between surfaces `from` and
    /// `to` that `selector` matches, with no recorded time.
    pub fn withdraw_matching(
        &mut self,
        from: Uuid,
        to: Uuid,
        selector: &RelationSelector,
        asserter: &str,
    ) {
        let ids: Vec<String> = self
            .held_statements(from, to)
            .into_iter()
            .filter(|statement| {
                statement.provenance_iri.as_deref() == Some(asserter)
                    && selector.matches(&statement.predicate, Vocabulary::core())
            })
            .map(|statement| statement.statement_id)
            .collect();
        for id in ids {
            self.withdraw_statement(&id, None, Some(asserter.to_owned()));
        }
    }

    /// The held statements between the resources surfaces `from` and `to`
    /// show.
    pub fn held_statements(&self, from: Uuid, to: Uuid) -> Vec<SemanticStatement> {
        let Some((a, b)) = self.shown(from).zip(self.shown(to)) else {
            return Vec::new();
        };
        self.payloads(a, b)
            .iter()
            .flat_map(|payload| payload.held_assertions().cloned().collect::<Vec<_>>())
            .collect()
    }

    /// Statement `id` as retained, held or withdrawn.
    pub fn find_statement(&self, id: &str) -> Option<SemanticStatement> {
        let (a, b) = self.statement(id)?;
        self.payloads(a, b)
            .iter()
            .flat_map(|payload| payload.assertions().to_vec())
            .find(|statement| statement.statement_id == id)
    }

    /// Every retained statement, held or withdrawn, as declared-lens D18
    /// compares them, in a stable order.
    pub fn retained_statements(&self) -> Vec<HeldRow> {
        let mut rows: Vec<HeldRow> = self
            .resources
            .resource_edges()
            .flat_map(|(a, b, payload)| {
                payload.assertions().iter().map(|s| HeldRow {
                    from: a.canonical_iri().to_owned(),
                    to: b.canonical_iri().to_owned(),
                    statement_id: s.statement_id.clone(),
                    predicate: s.predicate.clone(),
                    label: s.label.clone(),
                    basis: s.basis.clone(),
                    graph_scope: s.graph_scope.clone(),
                    provenance_iri: s.provenance_iri.clone(),
                    first_asserted_ms: s.first_asserted_ms,
                    asserted_at_ms: s.asserted_at_ms,
                    withdrawn: s.withdrawn.clone(),
                })
            })
            .collect();
        rows.sort_by(|x, y| {
            (&x.from, &x.to, &x.statement_id).cmp(&(&y.from, &y.to, &y.statement_id))
        });
        rows
    }
}

/// One retained statement: the resource pair it joins and its record.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HeldRow {
    pub from: String,
    pub to: String,
    pub statement_id: String,
    pub predicate: String,
    pub label: Option<String>,
    pub basis: AssertionBasis,
    pub graph_scope: GraphScope,
    pub provenance_iri: Option<String>,
    pub first_asserted_ms: Option<u64>,
    pub asserted_at_ms: Option<u64>,
    pub withdrawn: Option<AssertionWithdrawal>,
}
