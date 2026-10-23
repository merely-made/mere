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
        self.transact_change(at, false, move |candidate| {
            candidate
                .edit_as(author, ChangeKind::Edit, |graph| {
                    for delta in deltas {
                        apply_graph_delta(graph, delta);
                    }
                })
                .map(|(_, applied)| applied)
        })
        .await
        .map(|(applied, _)| applied)
    }

    /// [`transact_deltas`](Self::transact_deltas), with the addressable
    /// publication of the resulting graph in the same atomic write. Returns the
    /// published generation. A reader never sees an edit without its
    /// publication, and a refused write leaves neither.
    pub async fn transact_deltas_published(
        &mut self,
        author: Author,
        deltas: Vec<GraphDelta>,
        at: SystemTime,
    ) -> Result<(Applied, String), SessionError> {
        self.transact_change(at, true, move |candidate| {
            candidate
                .edit_as(author, ChangeKind::Edit, |graph| {
                    for delta in deltas {
                        apply_graph_delta(graph, delta);
                    }
                })
                .map(|(_, applied)| applied)
        })
        .await
        .map(|(applied, generation)| (applied, generation.expect("published write")))
    }

    /// Preflight observed stable-ID commands against this complete writer, then
    /// apply their live deltas and the resulting addressable publication in one
    /// atomic write. Every refusal comes before anything is written: a stale
    /// observation, a missing target, an unqualified session or a storage
    /// refusal leaves graph, history, capture and storage as they were. Returns
    /// the published generation with the application.
    pub async fn transact_live(
        &mut self,
        author: Author,
        commands: Vec<crate::live_graph::ObservedCommand>,
        at: SystemTime,
    ) -> Result<(crate::live_graph::LiveApplied, String), crate::live_graph::LiveEditError> {
        use crate::live_graph::{LiveApplied, preflight};
        self.ensure_writable()?;
        // The complete current records are what external observations were
        // taken against; the resident graph's filtered copy is never hashed.
        let source = self.prepare_resident_publication()?;
        let program = preflight(&self.graph, &source, &author, commands)?;
        let created = program.created.clone();
        let (applied, generation) = self
            .transact_change(at, true, move |candidate| {
                candidate
                    .edit_as(author.clone(), ChangeKind::Edit, |graph| {
                        program.apply(graph, &author)
                    })
                    .map(|(_, applied)| applied)
            })
            .await?;
        Ok((
            LiveApplied { applied, created },
            generation.expect("published write"),
        ))
    }

    /// The same staged/acknowledged boundary for this author's saved history.
    pub async fn transact_history(
        &mut self,
        author: Author,
        redo: bool,
        at: SystemTime,
    ) -> Result<Option<Reverted>, SessionError> {
        self.transact_change(at, false, move |candidate| {
            if redo {
                candidate.redo_now(author)
            } else {
                candidate.undo_now(author)
            }
        })
        .await
        .map(|(reverted, _)| reverted)
    }

    /// [`transact_history`](Self::transact_history) with the addressable
    /// publication of the resulting graph in the same atomic write.
    pub async fn transact_history_published(
        &mut self,
        author: Author,
        redo: bool,
        at: SystemTime,
    ) -> Result<(Option<Reverted>, String), SessionError> {
        self.transact_change(at, true, move |candidate| {
            if redo {
                candidate.redo_now(author)
            } else {
                candidate.undo_now(author)
            }
        })
        .await
        .map(|(reverted, generation)| (reverted, generation.expect("published write")))
    }

    /// Stage the application's view and its parallel history together.
    pub async fn transact_view(
        &mut self,
        author: Author,
        key: ViewKey,
        state: ViewIntent,
        at: SystemTime,
    ) -> Result<(), SessionError> {
        self.transact_change(at, false, move |candidate| {
            candidate.set_view_now(author, key, state)
        })
        .await
        .map(|(result, _)| result)
    }

    pub(crate) async fn transact_change<R>(
        &mut self,
        at: SystemTime,
        publish: bool,
        change: impl FnOnce(&mut Self) -> Result<R, SessionError>,
    ) -> Result<(R, Option<String>), SessionError> {
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
        // A view has its own log, so a graph head alone cannot detect a
        // competing view append. Include existing and newly requested keys.
        let mut view_heads = Vec::new();
        for (key, view) in &candidate.views {
            let log = self.keys.view_log(key);
            view_heads.push((log.clone(), view.saved));
            if let Some(held) = self.views.get(key).filter(|held| held.saved.0 > 0) {
                tails.extend(
                    held.log
                        .entry_writes::<JsonCodec>(&log, Seq(held.saved.0 - 1))?,
                );
            }
        }
        let pending = candidate.pending(at)?;
        let mut ops = pending.ops().to_vec();
        // The publication describes the candidate's graph and joins its write.
        // Preparation refuses before anything is written or installed.
        let generation = if publish {
            let publication = candidate.prepare_resident_publication()?;
            ops.extend_from_slice(publication.ops());
            Some(publication.generation().to_owned())
        } else {
            None
        };
        enum TransactionRejection {
            ReadOnly(SessionId),
            Validation(String),
        }

        let session_id = self.id();
        let merged = self.keys.at(MERGED);
        let rejection = Arc::new(Mutex::new(None));
        let rejected = rejection.clone();
        let transaction = Box::new(move |reader: &dyn muniment::TransactionReader| {
            let check = || -> Result<(), TransactionRejection> {
                // The merged marker can change independently of the source
                // logs when another handle undoes or redoes a merge. Read it
                // in this same backend transaction as the candidate writes.
                match reader.get(&merged) {
                    Ok(Some(_)) => return Err(TransactionRejection::ReadOnly(session_id)),
                    Ok(None) => {},
                    Err(error) => {
                        return Err(TransactionRejection::Validation(error.to_string()));
                    },
                }
                if reader
                    .list(&format!("{journal}/"))
                    .map_err(|e| TransactionRejection::Validation(e.to_string()))?
                    .len() as u64
                    != expected_journal.0
                    || reader
                        .list(&format!("{changes}/"))
                        .map_err(|e| TransactionRejection::Validation(e.to_string()))?
                        .len() as u64
                        != expected_changes.0
                {
                    return Err(TransactionRejection::Validation(
                        "Session changed before commit; draft retained".into(),
                    ));
                }
                for (log, expected) in view_heads {
                    if reader
                        .list(&format!("{log}/"))
                        .map_err(|e| TransactionRejection::Validation(e.to_string()))?
                        .len() as u64
                        != expected.0
                    {
                        return Err(TransactionRejection::Validation(
                            "Session view changed before commit; draft retained".into(),
                        ));
                    }
                }
                for op in tails {
                    if let WriteOp::Put { key, value } = op {
                        if reader
                            .get(&key)
                            .map_err(|e| TransactionRejection::Validation(e.to_string()))?
                            .as_ref()
                            != Some(&value)
                        {
                            return Err(TransactionRejection::Validation(
                                "Session log head changed before commit; draft retained".into(),
                            ));
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
        if let Some(rejection) = rejection.lock().expect("transaction rejection").take() {
            return Err(match rejection {
                TransactionRejection::ReadOnly(session_id) => SessionError::ReadOnly(session_id),
                TransactionRejection::Validation(error) => SessionError::Validation(error),
            });
        }
        candidate.stored(pending);
        *self = candidate;
        Ok((result, generation))
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
            admission_validator: self.admission_validator.clone(),
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
            merge_marks: self.merge_marks.clone(),
        }
    }
}
