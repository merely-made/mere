// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Stage a journaled candidate; install it only after atomic storage acknowledgement.
use super::*;

impl<B: LegacyTransactionBackend + Clone> GraphSession<B> {
    /// Apply one complete, validated change against an already durable session.
    /// Storage failure or a competing append leaves graph and history unchanged.
    /// Append-only log lengths and their last retained entries are rechecked in
    /// the write transaction. This is not a journal corruption/tampering audit.
    pub async fn transact_deltas(
        &mut self,
        author: Author,
        deltas: Vec<GraphDelta>,
        at: SystemTime,
    ) -> Result<Applied, SessionError> {
        self.transact_change(at, move |candidate| {
            candidate
                .edit_as(author, ChangeKind::Edit, |graph| {
                    for delta in deltas {
                        apply_graph_delta(graph, delta);
                    }
                })
                .map(|(_, applied)| applied)
        })
        .await
    }

    /// The same staged/acknowledged boundary for this author's saved history.
    pub async fn transact_history(
        &mut self,
        author: Author,
        redo: bool,
        at: SystemTime,
    ) -> Result<Option<Reverted>, SessionError> {
        self.transact_change(at, move |candidate| {
            if redo {
                candidate.redo_now(author)
            } else {
                candidate.undo_now(author)
            }
        })
        .await
    }

    async fn transact_change<R>(
        &mut self,
        at: SystemTime,
        change: impl FnOnce(&mut Self) -> Result<R, SessionError>,
    ) -> Result<R, SessionError> {
        self.ensure_writable()?;
        if self.has_unstored() {
            return Err(SessionError::Validation(
                "Save pending session changes before this transaction".into(),
            ));
        }
        let journal = self.keys.at(JOURNAL);
        let changes = self.keys.at(CHANGES);
        let expected_journal = self.saved;
        let expected_changes = self.changes_saved;
        let mut tails = Vec::new();
        if self.saved.0 > 0 {
            tails.extend(
                self.journal
                    .log()
                    .entry_writes::<JsonCodec>(&journal, Seq(self.saved.0 - 1))?,
            );
        }
        if self.changes_saved.0 > 0 {
            tails.extend(
                self.changes
                    .entry_writes::<JsonCodec>(&changes, Seq(self.changes_saved.0 - 1))?,
            );
        }
        let mut candidate = self.detached_candidate();
        let result = change(&mut candidate)?;
        let pending = candidate.pending(at)?;
        let ops = pending.ops().to_vec();
        let rejection = Arc::new(Mutex::new(None));
        let rejected = rejection.clone();
        let transaction = Box::new(move |reader: &dyn muniment::TransactionReader| {
            let check = || -> Result<(), String> {
                if reader
                    .list(&format!("{journal}/"))
                    .map_err(|e| e.to_string())?
                    .len() as u64
                    != expected_journal.0
                    || reader
                        .list(&format!("{changes}/"))
                        .map_err(|e| e.to_string())?
                        .len() as u64
                        != expected_changes.0
                {
                    return Err("Session changed before commit; draft retained".into());
                }
                for op in tails {
                    if let WriteOp::Put { key, value } = op {
                        if reader.get(&key).map_err(|e| e.to_string())?.as_ref() != Some(&value) {
                            return Err(
                                "Session log head changed before commit; draft retained".into()
                            );
                        }
                    }
                }
                Ok(())
            };
            if let Err(error) = check() {
                *rejected.lock().expect("transaction rejection") = Some(error);
                Vec::new()
            } else {
                ops
            }
        });
        self.slots.backend().transact(transaction).await?;
        if let Some(error) = rejection.lock().expect("transaction rejection").take() {
            return Err(SessionError::Validation(error));
        }
        candidate.stored(pending);
        *self = candidate;
        Ok(result)
    }

    // A private copy with its own recorder. No candidate edit can reach the
    // acknowledged graph's capture buffer through an Arc shared by Graph::clone.
    fn detached_candidate(&self) -> Self {
        let pending = Arc::default();
        let mut graph = self.graph.clone();
        record_into(&mut graph, &pending);
        Self {
            slots: JsonSlots::new(self.slots.backend().clone()),
            validator: self.validator.clone(),
            keys: Keys(self.keys.0.clone()),
            manifest: self.manifest.clone(),
            baseline: self.baseline.clone(),
            placement: self.placement.clone(),
            translation: self.translation.clone(),
            retained_source: self.retained_source.clone(),
            graph,
            pending,
            journal: self.journal.clone(),
            changes: self.changes.clone(),
            views: self.views.clone(),
            saved: self.saved,
            changes_saved: self.changes_saved,
            checkpointed: self.checkpointed,
            checkpoint_interval: self.checkpoint_interval,
            revision: self.revision,
            head_stored: self.head_stored,
            write_baseline: self.write_baseline,
            saved_pending_links: self.saved_pending_links.clone(),
            saved_pending_links_revision: self.saved_pending_links_revision,
            retained_pending_links_revision: self.retained_pending_links_revision,
        }
    }
}
