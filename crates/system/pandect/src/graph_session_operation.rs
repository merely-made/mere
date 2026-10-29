// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! An owner's operation record, retained atomically with the published edit it
//! names, through the session's one commit.
use super::transaction::{CommitExtra, Decide};
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

    /// [`transact_operation_published`](Self::transact_operation_published)
    /// with `validate` run on the edited graph before the commit. The record
    /// is decided inside the commit's transaction: absent, the receipt is
    /// written with the edit; held by an identical request that landed first,
    /// that receipt is the answer and nothing is edited twice; held by a
    /// different request, the identity is refused.
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
        let session = self.id().as_uuid().to_string();
        let same = |held: &OperationReceipt| {
            held.request == request && held.result == result && held.session == session
        };
        let elsewhere = || {
            SessionError::Validation("Operation identity already belongs to another request".into())
        };
        if let Some(held) = self.operation_receipt(operation).await? {
            return if same(&held) {
                Ok(held)
            } else {
                Err(elsewhere())
            };
        }
        let key = self.operation_key(operation);
        let receipt_of = |generation: &str| OperationReceipt {
            session: session.clone(),
            request: request.clone(),
            result: result.clone(),
            generation: generation.to_owned(),
        };
        // The record the transaction found under the key when it refused.
        let found: Arc<Mutex<Option<OperationReceipt>>> = Arc::default();
        let record = |generation: Option<&str>| {
            let receipt = receipt_of(generation.expect("operation publishes"));
            let (key, seen) = (key.clone(), found.clone());
            let decide: Decide = Box::new(move |reader: &dyn muniment::TransactionReader| {
                let Some(bytes) = reader.get(&key).map_err(|e| e.to_string())? else {
                    let value = serde_json::to_vec(&receipt).expect("receipt serializes");
                    return Ok(vec![WriteOp::Put { key, value }]);
                };
                let held = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
                *seen.lock().expect("operation record") = Some(held);
                Err("Operation identity already recorded".into())
            });
            CommitExtra {
                decide: Some(decide),
                ..CommitExtra::default()
            }
        };
        let committed = self
            .transact_change_with(at, true, record, move |candidate| {
                let (checked, applied) = candidate.edit_as(author, ChangeKind::Edit, |graph| {
                    for delta in deltas {
                        apply_graph_delta(graph, delta);
                    }
                    validate(graph)
                })?;
                checked.map_err(SessionError::Validation)?;
                Ok(applied)
            })
            .await;
        match committed {
            Ok((_, generation)) => Ok(receipt_of(&generation.expect("operation publishes"))),
            // The commit was refused. A record found under the identity decides:
            // the same request's receipt is the answer, another request is not.
            Err(SessionError::Validation(error)) => {
                match found.lock().expect("operation record").take() {
                    Some(held) if same(&held) => Ok(held),
                    Some(_) => Err(elsewhere()),
                    None => Err(SessionError::Validation(error)),
                }
            },
            Err(error) => Err(error),
        }
    }
}
