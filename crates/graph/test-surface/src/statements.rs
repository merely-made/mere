// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Statements of every relation family between the resources two surfaces
//! show, written whole through the store: what a host fold of the personal
//! graph writes (engine split D259).

use mere::model::{
    AssertionWithdrawal, EdgeAssertion, EdgePayload, JournalDelta, RelationSelector,
    SemanticStatement, selector_matches_kind,
};
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

    /// The legacy membership ensure: `assertion` held once per family kind,
    /// named by `asserter`, with no recorded time.
    pub fn assert_relation(
        &mut self,
        from: Uuid,
        to: Uuid,
        assertion: EdgeAssertion,
        asserter: &str,
    ) -> bool {
        let mut assertion = Some(assertion);
        self.rewrite(from, to, |payload| {
            assertion.take().is_some_and(|assertion| {
                payload.assert_relation_with_context(assertion, Some(asserter.to_owned()), None)
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
        selector: RelationSelector,
        asserter: &str,
    ) {
        let ids: Vec<String> = self
            .held_statements(from, to)
            .into_iter()
            .filter(|statement| {
                statement.provenance_iri.as_deref() == Some(asserter)
                    && selector_matches_kind(selector, statement.relation_kind())
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
}
