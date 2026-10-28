// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Stage a journaled candidate; install it only after atomic storage acknowledgement.
use super::*;

/// Opaque owner data retained atomically with a published graph change.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperationReceipt {
    pub session: String,
    pub request: Vec<u8>,
    pub result: Vec<u8>,
    pub generation: String,
}

impl<B: LegacyTransactionBackend + Clone> GraphSession<B> {
    /// Content-bound graph and history head, stable across a cold reopen.
    /// Partial resident graphs and pending changes are refused.
    pub fn operation_revision(&self) -> Result<String, SessionError> {
        if self.has_unstored() {
            return Err(SessionError::Validation(
                "Save pending session changes before observing a revision".into(),
            ));
        }
        // Keep the publication's completeness/placement preflight. Snapshot
        // reconstruction synthesizes visit times; replace them with the exact
        // authoritative facet sidecar before hashing.
        self.prepare_resident_publication()?;
        let mut normalized = Graph::try_from_recorded_snapshot(&self.graph.to_snapshot_at(0))
            .map_err(|error| SessionError::Validation(error.to_string()))?;
        normalized
            .install_facets(self.graph.facets().clone())
            .map_err(|error| SessionError::Validation(error.to_string()))?;
        let publication = self.resident_store().prepare_recorded_graph(&normalized)?;
        let mut hasher = blake3::Hasher::new();
        hasher.update(b"mere.graph-operation-revision/v1");
        hasher.update(publication.generation().as_bytes());
        for (name, ops) in [
            (
                "journal",
                self.journal
                    .log()
                    .entry_writes::<JsonCodec>(&self.keys.at(JOURNAL), Seq(0))?,
            ),
            (
                "changes",
                self.changes
                    .entry_writes::<JsonCodec>(&self.keys.at(CHANGES), Seq(0))?,
            ),
        ] {
            hasher.update(name.as_bytes());
            for op in ops {
                if let WriteOp::Put { value, .. } = op {
                    hasher.update(&(value.len() as u64).to_le_bytes());
                    hasher.update(&value);
                }
            }
        }
        Ok(hasher.finalize().to_hex().to_string())
    }

    fn operation_key(&self, operation: [u8; 32]) -> String {
        format!("graph-operations/{}.json", hex::encode(operation))
    }

    /// Query the durable owner record without consulting the current graph head.
    pub async fn operation_receipt(
        &self,
        operation: [u8; 32],
    ) -> Result<Option<OperationReceipt>, SessionError> {
        self.slots
            .backend()
            .get(&self.operation_key(operation))
            .await?
            .map(|bytes| {
                serde_json::from_slice(&bytes).map_err(|e| SessionError::Corrupt(e.to_string()))
            })
            .transpose()
    }

    /// Reconcile an operation after an uncertain storage acknowledgement.
    ///
    /// The owner must hold its exclusive writer lease throughout this call.
    /// If a durable receipt exists, reopen the same session and install its
    /// validated complete state only after reading the current publication.
    /// Pending local work is refused; live admission policy is retained for
    /// future edits but is not reapplied to already committed history.
    pub async fn reconcile_operation(
        &mut self,
        operation: [u8; 32],
    ) -> Result<Option<(OperationReceipt, String)>, SessionError> {
        self.ensure_writable()?;
        if self.has_unstored() {
            return Err(SessionError::Validation(
                "Save pending session changes before reconciliation".into(),
            ));
        }
        // Refuse unqualified or residency-limited writers before any reload.
        self.prepare_resident_publication()?;
        let Some(receipt) = self.operation_receipt(operation).await? else {
            return Ok(None);
        };
        if receipt.session != self.id().as_uuid().to_string() {
            return Err(SessionError::Validation(
                "Operation belongs to a different session".into(),
            ));
        }
        let launch = self.graph.current_session();
        let mut candidate = Self::open(self.slots.backend().clone(), self.id()).await?;
        candidate.ensure_writable()?;
        // This checks the reconstructed complete graph, not newly introduced
        // references: live admission does not rewrite committed history.
        candidate.prepare_resident_publication()?;
        if let Some(validator) = &self.validator {
            validator(&candidate.graph).map_err(SessionError::Validation)?;
        }
        let resident = candidate
            .resident_store()
            .open(crate::addressable_graph::ResidencyRoots::default())
            .await?;
        let generation = resident.generation().to_owned();
        candidate.graph.set_current_session(launch);
        candidate.checkpoint_interval = self.checkpoint_interval;
        candidate.validator = self.validator.clone();
        candidate.admission_validator = self.admission_validator.clone();
        candidate.revision = self.revision.wrapping_add(1);
        *self = candidate;
        Ok(Some((receipt, generation)))
    }

    /// Atomically associate an immutable request and result with a published edit.
    /// Owners validate policy and verify that the deltas produce their intended
    /// effect. Identical requests return the original result, never a new edit.
    pub async fn transact_operation_published(
        &mut self,
        operation: [u8; 32],
        request: Vec<u8>,
        result: Vec<u8>,
        author: Author,
        deltas: Vec<GraphDelta>,
        at: SystemTime,
    ) -> Result<OperationReceipt, SessionError> {
        self.transact_operation_published_checked(
            operation,
            request,
            result,
            author,
            deltas,
            at,
            |_| Ok(()),
        )
        .await
    }

    /// As above, with an owner-supplied postcondition over the staged graph.
    pub async fn transact_operation_published_checked(
        &mut self,
        operation: [u8; 32],
        request: Vec<u8>,
        result: Vec<u8>,
        author: Author,
        deltas: Vec<GraphDelta>,
        at: SystemTime,
        validate: impl FnOnce(&Graph) -> Result<(), String>,
    ) -> Result<OperationReceipt, SessionError> {
        if let Some(receipt) = self.operation_receipt(operation).await? {
            return if receipt.request == request
                && receipt.result == result
                && receipt.session == self.id().as_uuid().to_string()
            {
                Ok(receipt)
            } else {
                Err(SessionError::Validation(
                    "Operation identity already belongs to another request".into(),
                ))
            };
        }
        let key = self.operation_key(operation);
        let (_, receipt) = self
            .transact_change_record(at, true, Some((key, request, result)), move |candidate| {
                let (checked, applied) = candidate.edit_as(author, ChangeKind::Edit, |graph| {
                    for delta in deltas {
                        apply_graph_delta(graph, delta);
                    }
                    validate(graph)
                })?;
                checked.map_err(SessionError::Validation)?;
                Ok(applied)
            })
            .await?;
        Ok(receipt.expect("operation record"))
    }

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
        let (result, _) = self
            .transact_change_record(at, publish, None, change)
            .await?;
        Ok(result.expect("ordinary transaction"))
    }

    async fn transact_change_record<R>(
        &mut self,
        at: SystemTime,
        publish: bool,
        operation: Option<(String, Vec<u8>, Vec<u8>)>,
        change: impl FnOnce(&mut Self) -> Result<R, SessionError>,
    ) -> Result<(Option<(R, Option<String>)>, Option<OperationReceipt>), SessionError> {
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
        let session = self.id().as_uuid().to_string();
        let record = operation.map(|(key, request, result)| {
            let receipt = OperationReceipt {
                session,
                request,
                result,
                generation: generation.clone().expect("operation publishes"),
            };
            (key, receipt)
        });
        let existing = Arc::new(Mutex::new(None));
        let observed = existing.clone();
        let duplicate = Arc::new(Mutex::new(false));
        let was_duplicate = duplicate.clone();
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
                if let Some((key, receipt)) = &record {
                    match reader
                        .get(key)
                        .map_err(|e| TransactionRejection::Validation(e.to_string()))?
                    {
                        Some(bytes) => {
                            let prior: OperationReceipt = serde_json::from_slice(&bytes)
                                .map_err(|e| TransactionRejection::Validation(e.to_string()))?;
                            if prior.request != receipt.request
                                || prior.result != receipt.result
                                || prior.session != receipt.session
                            {
                                return Err(TransactionRejection::Validation(
                                    "Operation identity already belongs to another request".into(),
                                ));
                            }
                            *observed.lock().expect("operation receipt") = Some(prior);
                            *was_duplicate.lock().expect("duplicate") = true;
                            return Ok(());
                        },
                        None => {},
                    }
                }
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
            } else if observed.lock().expect("operation receipt").is_some() {
                Vec::new()
            } else {
                if let Some((key, receipt)) = record {
                    ops.push(WriteOp::Put {
                        key,
                        value: serde_json::to_vec(&receipt).expect("receipt serializes"),
                    });
                    *observed.lock().expect("operation receipt") = Some(receipt);
                }
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
        let receipt = existing.lock().expect("operation receipt").take();
        // A concurrent duplicate did not write this staged candidate.
        if *duplicate.lock().expect("duplicate") {
            return Ok((None, receipt));
        }
        candidate.stored(pending);
        *self = candidate;
        Ok((Some((result, generation)), receipt))
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
