// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Graph {
    /// Edit one exact held assertion without selecting a content-key survivor.
    pub fn update_assertion_metadata(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> bool {
        let Some((_, statement)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        if statement.withdrawn.is_some() {
            return false;
        }
        let mut basis = statement.basis.clone();
        basis.note = label.clone();
        if statement.basis == basis && statement.asserted_at_ms == asserted_at_ms {
            return false;
        }
        let owners: Vec<_> = self
            .inner
            .inner()
            .edge_references()
            .map(|edge| (RelationKey::Surface(edge.id()), edge.weight()))
            .chain(
                self.resource_relations()
                    .map(|(key, _, _, payload)| (RelationKey::Resource(key), payload)),
            )
            .filter(|(_, payload)| {
                payload
                    .assertions()
                    .iter()
                    .any(|statement| statement.statement_id == statement_id)
            })
            .map(|(key, _)| key)
            .collect();
        for owner in &owners {
            let semantic = &mut self.get_relation_mut(*owner).unwrap().assertions;
            for statement in semantic
                .statements
                .iter_mut()
                .filter(|statement| statement.statement_id == statement_id)
            {
                statement.label = label.clone();
                statement.basis = basis.clone();
                statement.asserted_at_ms = asserted_at_ms;
            }
            semantic.rebuild_compat();
        }
        self.bump_revision();
        for owner in owners {
            self.capture_statement_bucket(owner);
        }
        true
    }

    pub(crate) fn update_assertion_metadata_if_requested(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
        basis: Option<crate::types::AssertionBasis>,
    ) -> bool {
        let Some((_, current)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        let desired_basis = basis.unwrap_or_else(|| crate::types::AssertionBasis {
            note: label.clone(),
            ..Default::default()
        });
        if current.basis == desired_basis && asserted_at_ms.is_none() {
            return false;
        }
        let Some((_, current)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        self.update_assertion_record(
            statement_id,
            desired_basis.note.clone(),
            asserted_at_ms.or(current.asserted_at_ms),
            current.first_asserted_ms,
            current.withdrawn.clone(),
            desired_basis,
        )
    }

    pub(super) fn update_assertion_record(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
        first_asserted_ms: Option<u64>,
        withdrawn: Option<AssertionWithdrawal>,
        basis: crate::types::AssertionBasis,
    ) -> bool {
        let owners = self.semantic_statement_owner_keys(statement_id);
        if owners.is_empty() {
            return false;
        }
        let mut changed_owners = Vec::new();
        for owner in owners {
            let Some(data) = self
                .get_relation_mut(owner)
                .map(|payload| &mut payload.assertions)
            else {
                continue;
            };
            let mut changed = false;
            for statement in data
                .statements
                .iter_mut()
                .filter(|statement| statement.statement_id == statement_id)
            {
                if statement.label == label
                    && statement.asserted_at_ms == asserted_at_ms
                    && statement.first_asserted_ms == first_asserted_ms
                    && statement.withdrawn == withdrawn
                    && statement.basis == basis
                {
                    continue;
                }
                statement.label = label.clone();
                statement.asserted_at_ms = asserted_at_ms;
                statement.first_asserted_ms = first_asserted_ms;
                statement.withdrawn = withdrawn.clone();
                statement.basis = basis.clone();
                changed = true;
            }
            if changed {
                data.rebuild_compat();
                changed_owners.push(owner);
            }
        }
        if changed_owners.is_empty() {
            return false;
        }
        self.bump_revision();
        for owner in changed_owners {
            self.capture_statement_bucket(owner);
        }
        true
    }

    /// Ensure an assertion by its L13 reason key. Method version and note may
    /// update on the same ID; a changed method IRI or evidence set creates a
    /// new independently withdrawable assertion.
    pub fn try_assert_semantic_reason(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        mut spec: SemanticStatementSpec,
    ) -> Result<(RelationKey, StatementAssert), StatementWriteError> {
        spec.asserted_at_ms.get_or_insert_with(unix_epoch_millis);
        self.try_assert_semantic_statement(from, to, spec)
    }

    /// Confirm one exact held ID. Missing caller time is stamped once here;
    /// an unknown or withdrawn ID is never selected by matching its content.
    pub fn reconfirm_semantic_assertion(
        &mut self,
        statement_id: &str,
        label: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> bool {
        let Some((key, current)) = self.find_semantic_statement(statement_id) else {
            return false;
        };
        if current.withdrawn.is_some() {
            return false;
        }
        self.update_assertion_record(
            statement_id,
            label.clone(),
            Some(asserted_at_ms.unwrap_or_else(unix_epoch_millis)),
            current.first_asserted_ms,
            None,
            crate::types::AssertionBasis {
                note: label,
                ..current.basis.clone()
            },
        )
    }
}
