// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Stage a journaled candidate; install it only after atomic storage acknowledgement.
use super::*;

/// Writes a commit carries beside the session's own, and the stored bytes
/// they were decided against.
#[derive(Clone, Debug, Default)]
pub struct CommitExtra {
    /// Written in the same atomic batch, after the session's own.
    pub ops: Vec<WriteOp>,
    /// Keys the extra writes depend on, with the bytes read (`None`: absent).
    /// The commit lands only if each key still holds exactly those bytes.
    pub expect: Vec<(String, Option<Vec<u8>>)>,
}

enum Rejection {
    ReadOnly(SessionId),
    Validation(String),
}

fn refused(error: StoreError) -> Rejection {
    Rejection::Validation(error.to_string())
}

/// What the store must still hold for a commit from one handle to land: no
/// merged mark, the stored length and last stored entry of each log this
/// handle appends to, and the caller's own keys. Every commit of a session
/// rechecks through here, inside its backend transaction.
struct Recheck {
    session: SessionId,
    merged: String,
    logs: Vec<(String, Seq, &'static str)>,
    tails: Vec<WriteOp>,
    keys: Vec<(String, Option<Vec<u8>>)>,
}

impl Recheck {
    fn run(&self, reader: &dyn muniment::TransactionReader) -> Result<(), Rejection> {
        // The merged marker can change independently of the source logs when
        // another handle undoes or redoes a merge.
        if reader.get(&self.merged).map_err(refused)?.is_some() {
            return Err(Rejection::ReadOnly(self.session));
        }
        for (log, stored, message) in &self.logs {
            let held = reader.list(&format!("{log}/")).map_err(refused)?.len() as u64;
            if held != stored.0 {
                return Err(Rejection::Validation((*message).into()));
            }
        }
        for tail in &self.tails {
            if let WriteOp::Put { key, value } = tail {
                if reader.get(key).map_err(refused)?.as_ref() != Some(value) {
                    return Err(Rejection::Validation(
                        "Session log head changed before commit; draft retained".into(),
                    ));
                }
            }
        }
        for (key, bytes) in &self.keys {
            if &reader.get(key).map_err(refused)? != bytes {
                return Err(Rejection::Validation(format!(
                    "{key} changed before commit; draft retained"
                )));
            }
        }
        Ok(())
    }
}

/// The write of the last entry a handle stored: what a log must still hold
/// at its end for the handle's next append to follow it.
fn last_stored<T: Serialize>(
    log: &Journal<T>,
    prefix: &str,
    stored: Seq,
) -> Result<Option<WriteOp>, StoreError> {
    let Some(last) = stored.0.checked_sub(1) else {
        return Ok(None);
    };
    Ok(log
        .entry_writes::<JsonCodec>(prefix, Seq(last))?
        .into_iter()
        .next())
}

impl<B: LegacyTransactionBackend> GraphSession<B> {
    /// Store everything not stored yet, stamping new changes `at`, in one
    /// backend transaction. It lands only if the store still holds what this
    /// handle was opened or last stored at (log lengths, last entries, no
    /// merge mark): a stale handle is refused with "Session changed before
    /// commit" and keeps its pending edits for the caller to rebase.
    pub async fn flush(&mut self, at: SystemTime) -> Result<(), SessionError> {
        let pending = self.pending(at)?;
        if pending.is_empty() {
            self.stored(pending);
            return Ok(());
        }
        self.commit(pending, Vec::new(), Vec::new()).await
    }

    /// Write `pending` and `extra` atomically if the store passes this
    /// handle's recheck; mark `pending` stored once it has landed.
    async fn commit(
        &mut self,
        pending: Pending,
        extra: Vec<WriteOp>,
        expect: Vec<(String, Option<Vec<u8>>)>,
    ) -> Result<(), SessionError> {
        let check = self.recheck(expect)?;
        let mut ops = pending.ops().to_vec();
        ops.extend(extra);
        let rejection = Arc::new(Mutex::new(None));
        let rejected = rejection.clone();
        let transaction =
            Box::new(
                move |reader: &dyn muniment::TransactionReader| match check.run(reader) {
                    Ok(()) => ops,
                    Err(error) => {
                        *rejected.lock().expect("transaction rejection") = Some(error);
                        Vec::new()
                    },
                },
            );
        self.slots.backend().transact(transaction).await?;
        match rejection.lock().expect("transaction rejection").take() {
            Some(Rejection::ReadOnly(id)) => return Err(SessionError::ReadOnly(id)),
            Some(Rejection::Validation(error)) => return Err(SessionError::Validation(error)),
            None => {},
        }
        self.stored(pending);
        Ok(())
    }

    fn recheck(&self, keys: Vec<(String, Option<Vec<u8>>)>) -> Result<Recheck, SessionError> {
        let moved = "Session changed before commit; draft retained";
        let (journal, changes) = (self.keys.at(JOURNAL), self.keys.at(CHANGES));
        let mut tails = Vec::new();
        tails.extend(last_stored(self.journal.log(), &journal, self.saved)?);
        tails.extend(last_stored(&self.changes, &changes, self.changes_saved)?);
        let mut logs = vec![
            (journal, self.saved, moved),
            (changes, self.changes_saved, moved),
        ];
        // A view has its own log, so a graph head alone cannot detect a
        // competing view append.
        for (key, view) in &self.views {
            let log = self.keys.view_log(key);
            tails.extend(last_stored(&view.log, &log, view.saved)?);
            logs.push((
                log,
                view.saved,
                "Session view changed before commit; draft retained",
            ));
        }
        Ok(Recheck {
            session: self.id(),
            merged: self.keys.at(MERGED),
            logs,
            tails,
            keys,
        })
    }
}

impl<B: LegacyTransactionBackend + Clone> GraphSession<B> {
    /// Apply one complete, validated change against a session.
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

    /// Run `change` on a [detached candidate](Self::detached_candidate) and
    /// commit it as one change, with the handle's own unstored entries, in one
    /// backend transaction ([`commit_candidate`](Self::commit_candidate)).
    /// A refusal, from `change` or from storage, leaves this session as it was.
    ///
    /// The closure edits through [`edit_now`](Self::edit_now), so a direct
    /// write (a public assertion or declaration writer) transacts under the
    /// same capture-and-replay check as a `GraphDelta`: a candidate that
    /// changed graph truth outside the captured deltas is `NotReplayable`.
    pub async fn transact_change<R>(
        &mut self,
        at: SystemTime,
        publish: bool,
        change: impl FnOnce(&mut Self) -> Result<R, SessionError>,
    ) -> Result<(R, Option<String>), SessionError> {
        self.transact_change_with(at, publish, |_| CommitExtra::default(), change)
            .await
    }

    /// [`transact_change`](Self::transact_change) whose commit carries the
    /// writes `extra` returns, decided once the candidate's publication
    /// generation is known (`None` when not publishing).
    pub async fn transact_change_with<R>(
        &mut self,
        at: SystemTime,
        publish: bool,
        extra: impl FnOnce(Option<&str>) -> CommitExtra,
        change: impl FnOnce(&mut Self) -> Result<R, SessionError>,
    ) -> Result<(R, Option<String>), SessionError> {
        self.ensure_writable()?;
        let mut candidate = self.detached_candidate();
        let result = change(&mut candidate)?;
        let generation = self
            .commit_candidate_with(candidate, at, publish, extra)
            .await?;
        Ok((result, generation))
    }

    /// Commit `candidate`, a [`detached_candidate`](Self::detached_candidate)
    /// of this session edited since, and `extra` in one backend transaction;
    /// this session becomes the candidate on success. `publish` adds the
    /// addressable publication of the candidate's graph to the write. Storage
    /// changing under the handle, or under a key in `extra.expect`, refuses the
    /// commit before anything is written; this session is then unchanged.
    pub async fn commit_candidate(
        &mut self,
        candidate: Self,
        at: SystemTime,
        publish: bool,
        extra: CommitExtra,
    ) -> Result<Option<String>, SessionError> {
        self.commit_candidate_with(candidate, at, publish, |_| extra)
            .await
    }

    /// [`commit_candidate`](Self::commit_candidate) with the extra writes
    /// decided from the candidate's publication generation.
    pub async fn commit_candidate_with(
        &mut self,
        mut candidate: Self,
        at: SystemTime,
        publish: bool,
        extra: impl FnOnce(Option<&str>) -> CommitExtra,
    ) -> Result<Option<String>, SessionError> {
        self.ensure_writable()?;
        if candidate.id() != self.id()
            || (candidate.saved, candidate.changes_saved) != (self.saved, self.changes_saved)
        {
            return Err(SessionError::Validation(
                "Candidate was not cut from this session's stored state".into(),
            ));
        }
        let pending = candidate.pending(at)?;
        let mut ops = Vec::new();
        // The publication describes the candidate's graph and joins its write.
        // Preparation refuses before anything is written or installed.
        let generation = if publish {
            let publication = candidate.prepare_resident_publication()?;
            ops.extend_from_slice(publication.ops());
            Some(publication.generation().to_owned())
        } else {
            None
        };
        let extra = extra(generation.as_deref());
        ops.extend(extra.ops);
        candidate.commit(pending, ops, extra.expect).await?;
        *self = candidate;
        Ok(generation)
    }

    /// A private copy with its own recorder, to edit and offer to
    /// [`commit_candidate`](Self::commit_candidate). No candidate edit can reach
    /// the acknowledged graph's capture buffer through an Arc shared by
    /// Graph::clone, and nothing of this session changes until a commit lands.
    pub fn detached_candidate(&self) -> Self {
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
