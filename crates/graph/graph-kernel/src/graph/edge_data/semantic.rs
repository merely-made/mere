// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl SemanticData {
    pub fn statements(&self) -> &[SemanticStatement] {
        &self.statements
    }

    pub fn held_statements(&self) -> impl Iterator<Item = &SemanticStatement> {
        self.statements
            .iter()
            .filter(|statement| statement.is_held())
    }

    /// Membership ensure: content-dedup on held statements by
    /// `(recognized_sub_kind, predicate, graph_scope, provenance_iri)` (same key as
    /// [`insert_statement`](Self::insert_statement)), updating the deduped
    /// statement's metadata in place; a miss mints a device-safe id and
    /// appends. Always returns the fact handle.
    pub fn assert_statement(&mut self, spec: SemanticStatementSpec) -> StatementAssert {
        let spec = spec.normalized();
        if let Some(existing) = self.statements.iter_mut().find(|statement| {
            statement.is_held()
                && statement.recognized_sub_kind == spec.recognized_sub_kind
                && statement.predicate == spec.predicate
                && statement.graph_scope == spec.graph_scope
                && statement.provenance_iri == spec.provenance_iri
                && statement.basis.same_reason_as(&spec.basis)
        }) {
            let update_time = spec.asserted_at_ms.is_some();
            let changed = existing.label != spec.label
                || existing.basis != spec.basis
                || (update_time && existing.asserted_at_ms != spec.asserted_at_ms);
            let statement_id = existing.statement_id.clone();
            if changed {
                existing.label = spec.label;
                existing.basis = spec.basis;
                if update_time {
                    existing.asserted_at_ms = spec.asserted_at_ms;
                }
                self.rebuild_compat();
            }
            return StatementAssert {
                statement_id,
                changed,
            };
        }
        let mut statement = SemanticStatement::new(
            spec.predicate,
            spec.recognized_sub_kind,
            spec.label,
            spec.graph_scope,
            spec.provenance_iri,
            spec.asserted_at_ms,
        );
        statement.basis = spec.basis;
        let statement_id = statement.statement_id.clone();
        self.statements.push(statement);
        self.rebuild_compat();
        StatementAssert {
            statement_id,
            changed: true,
        }
    }

    /// Mark one exact held assertion withdrawn without deleting its history.
    pub fn withdraw_statement(
        &mut self,
        statement_id: &str,
        withdrawal: AssertionWithdrawal,
    ) -> bool {
        let Some(statement) = self
            .statements
            .iter_mut()
            .find(|statement| statement.statement_id == statement_id)
        else {
            return false;
        };
        if statement.withdrawn.is_some() {
            return false;
        }
        statement.withdrawn = Some(withdrawal);
        self.rebuild_compat();
        true
    }

    /// Retention compaction only: physically remove an assertion after its
    /// tombstone has been captured by the owning checkpoint.
    pub(crate) fn compact_statement(&mut self, statement_id: &str) -> bool {
        let before = self.statements.len();
        self.statements
            .retain(|statement| statement.statement_id != statement_id);
        if self.statements.len() == before {
            return false;
        }
        self.rebuild_compat();
        true
    }

    pub fn insert_statement(
        &mut self,
        recognized_sub_kind: Option<SemanticSubKind>,
        predicate: String,
        label: Option<String>,
        graph_scope: GraphScope,
        provenance_iri: Option<String>,
        asserted_at_ms: Option<u64>,
    ) -> bool {
        let basis = AssertionBasis {
            note: label.clone(),
            ..Default::default()
        };
        if let Some(existing) = self.statements.iter_mut().find(|statement| {
            statement.is_held()
                && statement.recognized_sub_kind == recognized_sub_kind
                && statement.predicate == predicate
                && statement.graph_scope == graph_scope
                && statement.provenance_iri == provenance_iri
                && statement.basis.same_reason_as(&basis)
        }) {
            let update_time = asserted_at_ms.is_some();
            if existing.label != label || (update_time && existing.asserted_at_ms != asserted_at_ms)
            {
                existing.label = label;
                existing.basis.note = existing.label.clone();
                if update_time {
                    existing.asserted_at_ms = asserted_at_ms;
                }
                self.rebuild_compat();
                return true;
            }
            return false;
        }

        let mut statement = SemanticStatement::new(
            predicate,
            recognized_sub_kind,
            label,
            graph_scope,
            provenance_iri,
            asserted_at_ms,
        );
        statement.basis = basis;
        self.statements.push(statement);
        self.rebuild_compat();
        true
    }

    pub(crate) fn upsert_persisted_statement(&mut self, mut statement: SemanticStatement) -> bool {
        statement.normalize_basis_compat();
        if let Some(existing) = self
            .statements
            .iter_mut()
            .find(|existing| existing.statement_id == statement.statement_id)
        {
            // A withdrawn ID is a tombstone: stale exact-ID replay/import must
            // not revive it or overwrite its lifecycle.
            if existing.withdrawn.is_some() {
                return false;
            }
            if existing.predicate == statement.predicate
                && existing.recognized_sub_kind == statement.recognized_sub_kind
                && existing.graph_scope == statement.graph_scope
                && existing.provenance_iri == statement.provenance_iri
                && existing.label == statement.label
                && existing.basis == statement.basis
                && existing.asserted_at_ms == statement.asserted_at_ms
                && existing.withdrawn == statement.withdrawn
            {
                return false;
            }
            existing.predicate = statement.predicate;
            existing.recognized_sub_kind = statement.recognized_sub_kind;
            existing.graph_scope = statement.graph_scope;
            existing.provenance_iri = statement.provenance_iri;
            existing.label = statement.label;
            existing.basis = statement.basis;
            existing.asserted_at_ms = statement.asserted_at_ms;
            existing.withdrawn = statement.withdrawn;
            self.rebuild_compat();
            return true;
        }
        self.push_persisted_statement(statement)
    }

    /// Exact restore preserves every stored handle, including legacy records.
    pub fn push_persisted_statement(&mut self, mut statement: SemanticStatement) -> bool {
        statement.normalize_basis_compat();
        if self
            .statements
            .iter()
            .any(|existing| existing.statement_id == statement.statement_id)
        {
            return false;
        }
        self.statements.push(statement);
        self.rebuild_compat();
        true
    }

    /// Migration/retention compaction primitive. Authored graph retractions
    /// must use Graph's exact-ID withdrawal API instead.
    pub(crate) fn remove_statements_with_sub_kind(&mut self, sub_kind: SemanticSubKind) -> bool {
        let before = self.statements.len();
        self.statements.retain(|statement| {
            !statement.is_held() || statement.recognized_sub_kind != Some(sub_kind)
        });
        if self.statements.len() == before {
            return false;
        }
        self.rebuild_compat();
        true
    }

    pub fn set_statement_predicate(&mut self, predicate: Option<String>) {
        match predicate {
            Some(predicate) => {
                if !self.held_statements().any(SemanticStatement::is_semantic) {
                    self.statements.push(SemanticStatement::new(
                        predicate,
                        None,
                        self.label.clone(),
                        GraphScope::Default,
                        None,
                        None,
                    ));
                } else {
                    for statement in self
                        .statements
                        .iter_mut()
                        .filter(|statement| statement.is_held() && statement.is_semantic())
                    {
                        statement.predicate = predicate.clone();
                    }
                }
            },
            None => {
                for statement in self
                    .statements
                    .iter_mut()
                    .filter(|s| s.is_held() && s.is_semantic())
                {
                    if let Some(sub_kind) = statement.recognized_sub_kind {
                        statement.predicate = predicate_iri(sub_kind).to_string();
                    } else {
                        // This compatibility setter has no authored event
                        // envelope. Preserve an undated tombstone; callers
                        // that need record time/actor use Graph's writer API.
                        statement.withdrawn = Some(AssertionWithdrawal {
                            at_ms: None,
                            by: None,
                        });
                    }
                }
            },
        }
        self.rebuild_compat();
    }

    pub(crate) fn rebuild_compat(&mut self) {
        // Semantic-only access remains a borrowed contiguous slice, even with
        // a large non-semantic withdrawal history. Stable partitioning keeps
        // the original relative order inside both groups.
        self.statements
            .sort_by_key(|statement| !statement.is_semantic());
        self.sub_kinds = self
            .statements
            .iter()
            .filter(|statement| statement.is_held() && statement.is_semantic())
            .filter_map(|statement| match statement.relation_kind() {
                super::super::RelationKind::Semantic(kind) => Some(kind),
                _ => None,
            })
            .collect();

        let mut labels = self
            .statements
            .iter()
            .filter(|statement| statement.is_held() && statement.is_semantic())
            .filter_map(|statement| statement.label.clone())
            .collect::<Vec<_>>();
        labels.sort();
        labels.dedup();
        self.label = match labels.len() {
            0 => None,
            1 => labels.into_iter().next(),
            _ => self
                .statements
                .iter()
                .rev()
                .filter(|statement| statement.is_held() && statement.is_semantic())
                .find_map(|statement| statement.label.clone()),
        };

        let mut predicates = self
            .statements
            .iter()
            .filter(|statement| statement.is_held() && statement.is_semantic())
            .map(|statement| statement.predicate.clone())
            .collect::<Vec<_>>();
        predicates.sort();
        predicates.dedup();
        self.predicate = if predicates.len() == 1 {
            predicates.into_iter().next()
        } else {
            None
        };
    }
}
