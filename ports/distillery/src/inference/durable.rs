// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Signed, encrypted per-persona ask logs over the resident's existing
//! p2panda endpoint. Each token appends only new IDs and text; immutable prompt
//! bytes appear once. Concurrent continuations remain a visible fork.
use super::*;
use chacha20poly1305::{
    Key, XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use eidetic::models::inference::{InferenceContinuation, InferenceRunRecord, InferenceRunStatus};
use muniment::Backend;
use p2panda_core::{Body, Hash, Header, Operation, SigningKey, Topic, VerifyingKey};
use p2panda_store::topics::TopicStore;
use personae::Ed25519Keypair;
use rand_core::{OsRng, RngCore};
use std::collections::BTreeMap;
use stickleback::{
    Admission, JoinedSpace, MunimentStore, OperationPolicy, OperationProcessor, Reject, StoreTarget,
};
use zeroize::Zeroizing;

/// Signed address of one device's per-ask log; prompt text stays encrypted.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunExt {
    pub space: [u8; 32],
    pub device: [u8; 32],
    pub run: [u8; 32],
}
#[derive(Clone)]
struct Policy {
    space: [u8; 32],
    author: VerifyingKey,
}
impl OperationPolicy<RunExt> for Policy {
    type LogId = ([u8; 32], [u8; 32]);
    fn admit(&self, op: &Operation<RunExt>) -> Result<Admission<Self::LogId>, Reject> {
        if op.header.extensions.space != self.space || op.header.verifying_key != self.author {
            return Err(Reject::new(
                "wrong-persona",
                "inference log requires this persona's derived signer",
            ));
        }
        if op.header.payload_size > 1024 * 1024 || op.body.is_none() {
            return Err(Reject::new(
                "invalid-checkpoint",
                "missing or oversized encrypted checkpoint",
            ));
        }
        Ok(Admission::keep(StoreTarget::new(
            Topic::from(self.space),
            (op.header.extensions.run, op.header.extensions.device),
        )))
    }
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Delta {
    parent: Option<String>,
    /// Full state at creation and state/device transitions only.
    record: Option<RunRecord>,
    tokens: Vec<u32>,
    text: String,
    draws: u64,
    complete: bool,
    state: RunStateV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence: Option<InferenceRunRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evidence_schema: Option<eidetic::SchemaRef>,
}
#[derive(Serialize, Deserialize)]
struct Sealed {
    nonce: [u8; 24],
    bytes: Vec<u8>,
}

/// Durable encrypted log and its joined replication session. The host clears
/// the data key after pausing asks, and re-derives it at unlock.
pub struct PersonaRunJournal<B: Backend + Clone + Send + Sync + 'static> {
    store: MunimentStore<B, RunExt>,
    policy: Policy,
    signing: SigningKey,
    device: String,
    device_hash: [u8; 32],
    key: Mutex<Option<Zeroizing<[u8; 32]>>>,
    heads: Mutex<BTreeMap<String, (String, RunRecord)>>,
    joined: Mutex<Option<JoinedSpace<RunExt>>>,
    /// Serializes local append and explicit refresh, including log frontier reads.
    writer: Mutex<()>,
}
impl<B: Backend + Clone + Send + Sync + 'static> PersonaRunJournal<B> {
    /// Bind a local store to the persona signer, stable device ID and data key.
    pub fn new(
        backend: B,
        space: [u8; 32],
        author: &Ed25519Keypair,
        device: String,
        key: [u8; 32],
    ) -> Self {
        let signing = SigningKey::from_bytes(&author.to_seed());
        Self {
            store: MunimentStore::new(backend),
            policy: Policy {
                space,
                author: signing.verifying_key(),
            },
            signing,
            device_hash: *Hash::digest(device.as_bytes()).as_bytes(),
            device,
            key: Mutex::new(Some(Zeroizing::new(key))),
            heads: Mutex::new(BTreeMap::new()),
            joined: Mutex::new(None),
            writer: Mutex::new(()),
        }
    }
    /// Join the ask lane on the resident-owned endpoint and gossip overlay.
    pub async fn join(
        self: &Arc<Self>,
        endpoint: stickleback::Endpoint,
        gossip: stickleback::Gossip,
    ) -> Result<(), String> {
        let store = self.store.clone();
        let policy = self.policy.clone();
        let joined = JoinedSpace::join::<_, ([u8; 32], [u8; 32]), _, _>(
            stickleback::lane_id("inference/v1", self.policy.space),
            self.store.clone(),
            endpoint,
            gossip,
            self.policy.space,
            move |op| {
                let processor = OperationProcessor::new(store.clone(), policy.clone());
                async move {
                    processor
                        .process(&op)
                        .await
                        .is_ok_and(|outcome| outcome.inserted())
                }
            },
        )
        .await
        .map_err(|error| error.to_string())?;
        *self.joined.lock().unwrap() = Some(joined);
        Ok(())
    }
    /// End replication and await release of captured store handles.
    pub async fn shutdown(&self) -> Result<(), String> {
        let joined = self.joined.lock().unwrap().take();
        if let Some(joined) = joined {
            joined
                .leave_and_wait()
                .await
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }
    /// Clear decrypted heads and the retained data key.
    pub fn lock(&self) {
        let _writer = self.writer.lock().unwrap();
        self.heads.lock().unwrap().clear();
        self.key.lock().unwrap().take();
    }
    /// Install the freshly re-derived persona data key.
    pub fn unlock(&self, key: [u8; 32]) {
        let _writer = self.writer.lock().unwrap();
        *self.key.lock().unwrap() = Some(Zeroizing::new(key));
    }
    /// Read actual LogSync activity for this lane.
    pub fn sync_status(&self) -> Option<stickleback::SyncStatus> {
        self.joined
            .lock()
            .unwrap()
            .as_ref()
            .map(|joined| joined.sync_status())
    }
    /// Read the underlying signed-operation store.
    pub fn store(&self) -> &MunimentStore<B, RunExt> {
        &self.store
    }
    /// Verify persona authority and commit a received signed operation.
    pub async fn accept(&self, op: &Operation<RunExt>) -> Result<bool, String> {
        OperationProcessor::new(self.store.clone(), self.policy.clone())
            .process(op)
            .await
            .map(|outcome| outcome.inserted())
            .map_err(|error| error.to_string())
    }
    /// Read all retained operations under this persona topic.
    pub async fn operations(&self) -> Result<Vec<Operation<RunExt>>, String> {
        let logs: BTreeMap<VerifyingKey, Vec<([u8; 32], [u8; 32])>> = self
            .store
            .resolve(&Topic::from(self.policy.space))
            .await
            .map_err(|error| error.to_string())?;
        let mut ops = Vec::new();
        for (author, logs) in logs {
            for log in logs {
                if let Some(entries) = self
                    .store
                    .get_log_entries(&author, &log, None, None)
                    .await
                    .map_err(|error| error.to_string())?
                {
                    ops.extend(entries.into_iter().map(|(op, _)| op));
                }
            }
        }
        Ok(ops)
    }
    /// Project a retained run through Eidetic's inference evidence schema.
    pub fn retained_record(&self, run: &str) -> Result<InferenceRunRecord, String> {
        let _writer = self.writer.lock().unwrap();
        let heads = self.fold()?;
        let id = self.materialize_evidence(&heads.get(run).ok_or("unknown retained run")?.1)?;
        let mut backend = self.store.backend().clone();
        pollster::block_on(eidetic::typed::load_typed_sealed(
            &mut backend,
            &mut eidetic::NoFetcher,
            Some(self),
            id,
        ))
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "retained evidence disappeared".into())
    }
    /// Resolve an exact reference through its sealed Eidetic blob manifest.
    pub fn resolve_evidence_blob(
        &self,
        reference: eidetic::ManifestId,
    ) -> Result<Option<Vec<u8>>, String> {
        let _writer = self.writer.lock().unwrap();
        for (_, (_, record)) in self.fold()? {
            self.materialize_evidence(&record)?;
        }
        let mut backend = self.store.backend().clone();
        pollster::block_on(eidetic::typed::load_typed_sealed::<
            eidetic::models::OpaqueBlob,
        >(
            &mut backend,
            &mut eidetic::NoFetcher,
            Some(self),
            reference,
        ))
        .map(|blob| blob.map(|blob| blob.0))
        .map_err(|error| error.to_string())
    }
    fn save_evidence<T: eidetic::TypedPayload>(
        &self,
        value: &T,
    ) -> Result<eidetic::ManifestId, String> {
        let bytes = Zeroizing::new(
            value
                .serialize_to_bytes()
                .map_err(|error| error.to_string())?,
        );
        let id = eidetic::ManifestId::of_blob(&bytes);
        let mut backend = self.store.backend().clone();
        if pollster::block_on(eidetic::manifest::load_manifest(&mut backend, id))
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return Ok(id);
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis() as u64;
        pollster::block_on(eidetic::typed::save_typed_sealed(
            &mut backend,
            Some(self),
            value,
            vec![],
            eidetic::PrivacyClass::LocalOnly,
            eidetic::ProvenanceRecord::self_imported("inference"),
            eidetic::TrustEnvelope::self_asserted(),
            eidetic::Timestamp(now),
        ))
        .map_err(|error| error.to_string())
    }
    fn materialize_evidence(&self, record: &RunRecord) -> Result<eidetic::ManifestId, String> {
        self.save_evidence(&eidetic::models::OpaqueBlob(
            record.view.question.as_bytes().to_vec(),
        ))?;
        self.save_evidence(&eidetic::models::OpaqueBlob(
            record.view.text.as_bytes().to_vec(),
        ))?;
        if let Some(cp) = &record.checkpoint {
            self.save_evidence(&eidetic::models::OpaqueBlob(
                serde_json::to_vec(cp).map_err(|error| error.to_string())?,
            ))?;
        }
        self.save_evidence(&Self::evidence(record)?)
    }
    /// Attach a real uninterrupted comparison. Absence of a comparison hash
    /// means unmeasured; a hash with no differing position means equal IDs.
    pub fn record_comparison(
        &self,
        run: &str,
        reference: &GenerationCheckpoint,
    ) -> Result<Option<usize>, String> {
        let mut record = self
            .load()?
            .into_iter()
            .find(|record| record.view.run == run)
            .ok_or("unknown run")?;
        let actual = record
            .checkpoint
            .as_ref()
            .ok_or("run has no model checkpoint")?;
        if actual.session != reference.session
            || actual.request != reference.request
            || !actual.complete
            || !reference.complete
        {
            return Err(
                "comparison requires completed runs of the same exact session and request".into(),
            );
        }
        let first = actual
            .token_ids
            .iter()
            .zip(&reference.token_ids)
            .position(|(a, b)| a != b)
            .or_else(|| {
                (actual.token_ids.len() != reference.token_ids.len())
                    .then_some(actual.token_ids.len().min(reference.token_ids.len()))
            });
        let visit = record
            .continuations
            .last_mut()
            .ok_or("run has no device visit")?;
        visit.first_differing_token = first;
        visit.compared_token_ids_hash = Some(
            eidetic::Hash::of(
                &serde_json::to_vec(&reference.token_ids).map_err(|error| error.to_string())?,
            )
            .to_string(),
        );
        self.append(&record)?;
        Ok(first)
    }

    fn crypt(
        &self,
        bytes: &[u8],
        nonce: [u8; 24],
        address: &RunExt,
        encrypt: bool,
    ) -> Result<Vec<u8>, String> {
        let key = self.key.lock().unwrap();
        let key = key.as_ref().ok_or("inference data is locked")?;
        let cipher = XChaCha20Poly1305::new(&Key::try_from(&key[..]).unwrap());
        let nonce = XNonce::try_from(&nonce[..]).unwrap();
        let aad = serde_json::to_vec(address).map_err(|error| error.to_string())?;
        let payload = Payload {
            msg: bytes,
            aad: &aad,
        };
        if encrypt {
            cipher.encrypt(&nonce, payload)
        } else {
            cipher.decrypt(&nonce, payload)
        }
        .map_err(|_| "inference checkpoint authentication failed".into())
    }
    fn decode(&self, op: &Operation<RunExt>) -> Result<Delta, String> {
        let body = op.body.as_ref().ok_or("checkpoint body missing")?;
        let sealed: Sealed =
            serde_json::from_slice(body.as_bytes()).map_err(|error| error.to_string())?;
        let clear = Zeroizing::new(self.crypt(
            &sealed.bytes,
            sealed.nonce,
            &op.header.extensions,
            false,
        )?);
        serde_json::from_slice(&clear).map_err(|error| error.to_string())
    }
    fn fold(&self) -> Result<BTreeMap<String, (String, RunRecord)>, String> {
        let mut pending: BTreeMap<String, (RunExt, Delta)> = pollster::block_on(self.operations())?
            .into_iter()
            .map(|op| {
                self.decode(&op)
                    .map(|delta| (op.hash.to_string(), (op.header.extensions, delta)))
            })
            .collect::<Result<_, _>>()?;
        let mut states: BTreeMap<String, RunRecord> = BTreeMap::new();
        // Keep each parent only until all its children have been replayed.
        // A long linear ask must not retain every full token prefix in memory.
        let mut children = BTreeMap::<String, usize>::new();
        for (_, delta) in pending.values() {
            if let Some(parent) = &delta.parent {
                *children.entry(parent.clone()).or_default() += 1;
            }
        }
        while !pending.is_empty() {
            let ready: Vec<_> = pending
                .iter()
                .filter(|(_, (_, delta))| {
                    delta
                        .parent
                        .as_ref()
                        .is_none_or(|parent| states.contains_key(parent))
                })
                .map(|(id, _)| id.clone())
                .collect();
            if ready.is_empty() {
                return Err("inference checkpoint log has a missing parent".into());
            }
            for id in ready {
                let (address, delta) = pending.remove(&id).unwrap();
                let mut record = match delta.record {
                    Some(record) => record,
                    None => states
                        .get(delta.parent.as_ref().ok_or("token delta has no parent")?)
                        .ok_or("missing checkpoint parent")?
                        .clone(),
                };
                if *Hash::digest(record.view.run.as_bytes()).as_bytes() != address.run {
                    return Err("checkpoint addresses another run".into());
                }
                record.view.state = delta.state;
                if let Some(cp) = &mut record.checkpoint {
                    cp.token_ids.extend(delta.tokens);
                    cp.text.push_str(&delta.text);
                    cp.draws = delta.draws;
                    cp.complete = delta.complete;
                    record.view.text = cp.text.clone();
                }
                if let Some(parent) = delta.parent {
                    let remaining = children
                        .get_mut(&parent)
                        .ok_or("missing parent child count")?;
                    *remaining -= 1;
                    if *remaining == 0 {
                        states.remove(&parent);
                    }
                }
                states.insert(id, record);
            }
        }
        let mut heads = BTreeMap::new();
        for (id, record) in states {
            {
                if heads
                    .insert(record.view.run.clone(), (id, record))
                    .is_some()
                {
                    return Err(
                        "inference run has concurrent continuations; choose a branch explicitly"
                            .into(),
                    );
                }
            }
        }
        Ok(heads)
    }
    /// Evidence projection is typed by Eidetic, independent of the decoder's
    /// checkpoint wire. Content addresses refer to the exact bytes in this log.
    pub fn evidence(record: &RunRecord) -> Result<InferenceRunRecord, String> {
        let cp = record.checkpoint.as_ref();
        Ok(InferenceRunRecord {
            run: record.view.run.clone(),
            question_blob: eidetic::ManifestId::of_blob(record.view.question.as_bytes()),
            supplied_passages: Vec::new(),
            session_id: record
                .view
                .session
                .as_ref()
                .map(|session| session.session_id.clone()),
            prompt_template_hash: record
                .view
                .session
                .as_ref()
                .map(|session| session.prompt_template_hash.clone()),
            weight_format: Some("f32".into()),
            request: serde_json::to_value(cp.map_or(&record.request, |cp| &cp.request))
                .map_err(|error| error.to_string())?,
            output_blob: eidetic::ManifestId::of_blob(record.view.text.as_bytes()),
            checkpoint_blob: cp
                .map(|cp| serde_json::to_vec(cp).map(|bytes| eidetic::ManifestId::of_blob(&bytes)))
                .transpose()
                .map_err(|error| error.to_string())?,
            status: match record.view.state {
                RunStateV1::Queued => InferenceRunStatus::Queued,
                RunStateV1::Running => InferenceRunStatus::Running,
                RunStateV1::Paused => InferenceRunStatus::Paused,
                RunStateV1::Finished => InferenceRunStatus::Finished,
                RunStateV1::Cancelled => InferenceRunStatus::Cancelled,
                RunStateV1::Failed => InferenceRunStatus::Failed,
            },
            continuations: record
                .continuations
                .iter()
                .map(|visit| InferenceContinuation {
                    device: visit.device.clone(),
                    at_token: visit.at_token,
                    reason: visit.reason.clone(),
                    first_differing_token: visit.first_differing_token,
                    compared_token_ids_hash: visit.compared_token_ids_hash.clone(),
                })
                .collect(),
            error: record.view.error.clone(),
        })
    }
}
// The all-zero device address is reserved for Eidetic blob sealing. It
// separates content-hash AAD from every installed device's log address.
impl<B: Backend + Clone + Send + Sync + 'static> eidetic::PayloadSealer for PersonaRunJournal<B> {
    fn seal(
        &self,
        hash: &eidetic::Hash,
        clear: &[u8],
    ) -> eidetic::Result<(Vec<u8>, eidetic::SealedBlobRef)> {
        let address = RunExt {
            space: self.policy.space,
            device: [0; 32],
            run: *hash.as_bytes(),
        };
        let mut nonce = [0; 24];
        OsRng.fill_bytes(&mut nonce);
        let bytes = self
            .crypt(clear, nonce, &address, true)
            .map_err(eidetic::Error::new)?;
        let sealed = serde_json::to_vec(&Sealed { nonce, bytes })
            .map_err(|error| eidetic::Error::new(error.to_string()))?;
        Ok((
            sealed,
            eidetic::SealedBlobRef {
                epoch: eidetic::SealEpochId([1; 16]),
                format: eidetic::seal::SEAL_FORMAT_V1.into(),
            },
        ))
    }
    fn unseal(
        &self,
        hash: &eidetic::Hash,
        marker: &eidetic::SealedBlobRef,
        sealed: &[u8],
    ) -> eidetic::Result<Vec<u8>> {
        if marker.epoch != eidetic::SealEpochId([1; 16])
            || marker.format != eidetic::seal::SEAL_FORMAT_V1
        {
            return Err(eidetic::Error::new(
                "unsupported inference blob seal epoch or format",
            ));
        }
        let sealed: Sealed = serde_json::from_slice(sealed)
            .map_err(|error| eidetic::Error::new(error.to_string()))?;
        let address = RunExt {
            space: self.policy.space,
            device: [0; 32],
            run: *hash.as_bytes(),
        };
        self.crypt(&sealed.bytes, sealed.nonce, &address, false)
            .map_err(eidetic::Error::new)
    }
}

impl<B: Backend + Clone + Send + Sync + 'static> RunJournal for PersonaRunJournal<B> {
    fn remote_revision(&self) -> u64 {
        self.sync_status().map_or(0, |status| status.ops_received)
    }
    fn device(&self) -> &str {
        &self.device
    }
    fn load(&self) -> Result<Vec<RunRecord>, String> {
        let _writer = self.writer.lock().unwrap();
        let heads = self.fold()?;
        let runs = heads.values().map(|(_, record)| record.clone()).collect();
        *self.heads.lock().unwrap() = heads;
        Ok(runs)
    }
    fn append(&self, record: &RunRecord) -> Result<(), String> {
        let _writer = self.writer.lock().unwrap();
        let mut heads = self.heads.lock().unwrap();
        let previous = heads.get(&record.view.run);
        let mut delta = Delta {
            parent: previous.map(|(id, _)| id.clone()),
            record: Some(record.clone()),
            tokens: Vec::new(),
            text: String::new(),
            draws: record.checkpoint.as_ref().map_or(0, |cp| cp.draws),
            complete: record.checkpoint.as_ref().is_some_and(|cp| cp.complete),
            evidence: None,
            evidence_schema: None,
            state: record.view.state,
        };
        if let Some((_, previous)) = previous {
            if previous.view.question != record.view.question
                || previous.request != record.request
                || record.continuations.len() < previous.continuations.len()
                || !previous
                    .continuations
                    .iter()
                    .zip(&record.continuations)
                    .all(|(a, b)| {
                        a.device == b.device && a.at_token == b.at_token && a.reason == b.reason
                    })
                || previous.checkpoint.as_ref().is_some_and(|old| {
                    record.checkpoint.as_ref().is_none_or(|new| {
                        old.session != new.session
                            || old.request != new.request
                            || !new.token_ids.starts_with(&old.token_ids)
                    })
                })
            {
                return Err("stale or incompatible inference continuation; refresh the retained run before resuming".into());
            }
            if let (Some(old), Some(new)) = (&previous.checkpoint, &record.checkpoint) {
                if old.request == new.request
                    && old.session == new.session
                    && old.prompt_ids == new.prompt_ids
                    && new.token_ids.starts_with(&old.token_ids)
                    && new.text.starts_with(&old.text)
                    && previous.continuations == record.continuations
                    && previous.pause_reason == record.pause_reason
                    && record.view.state == RunStateV1::Running
                {
                    delta.record = None;
                    delta.tokens = new.token_ids[old.token_ids.len()..].to_vec();
                    delta.text = new.text[old.text.len()..].to_string();
                }
            }
        }
        if delta.record.is_some() || delta.complete {
            delta.evidence = Some(Self::evidence(record)?);
            delta.evidence_schema =
                Some(<InferenceRunRecord as eidetic::TypedPayload>::schema_ref());
        }
        let address = RunExt {
            space: self.policy.space,
            device: self.device_hash,
            run: *Hash::digest(record.view.run.as_bytes()).as_bytes(),
        };
        let mut nonce = [0; 24];
        OsRng.fill_bytes(&mut nonce);
        let clear = Zeroizing::new(serde_json::to_vec(&delta).map_err(|error| error.to_string())?);
        let sealed = Sealed {
            nonce,
            bytes: self.crypt(&clear, nonce, &address, true)?,
        };
        let body = serde_json::to_vec(&sealed).map_err(|error| error.to_string())?;
        if body.len() > 1024 * 1024 {
            return Err("checkpoint exceeds the one MiB operation cap".into());
        }
        if delta.evidence.is_some() {
            self.materialize_evidence(record)?;
        }
        let latest = pollster::block_on(
            self.store
                .get_latest_entry(&self.policy.author, &(address.run, address.device)),
        )
        .map_err(|error| error.to_string())?;
        let sequence = latest.as_ref().map_or(Ok(0), |op| {
            op.header
                .seq_num
                .checked_add(1)
                .ok_or("checkpoint log sequence exhausted")
        })?;
        let header = Header::builder()
            .body(&body)
            .seq_num(sequence)
            .backlink(latest.map(|op| op.hash))
            .build(&self.signing, address);
        let op = Operation {
            hash: header.hash(),
            header,
            body: Some(Body::from_bytes(&body)),
        };
        pollster::block_on(self.accept(&op))?;
        heads.insert(
            record.view.run.clone(),
            (op.hash.to_string(), record.clone()),
        );
        // A disconnected live lane cannot undo a durable commit. LogSync
        // catches up from this same store after reconnection.
        if let Some(joined) = self.joined.lock().unwrap().as_ref() {
            let _ = joined.publish(op);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muniment::MemoryBackend;
    fn journal(device: &str) -> PersonaRunJournal<MemoryBackend> {
        PersonaRunJournal::new(
            MemoryBackend::new(),
            [7; 32],
            &Ed25519Keypair::from_seed([2; 32]),
            device.into(),
            [9; 32],
        )
    }
    fn paused() -> RunRecord {
        RunRecord {
            view: RunV1 {
                run: "private-run".into(),
                question: "secret question".into(),
                state: RunStateV1::Paused,
                text: String::new(),
                error: None,
                session: None,
                tokens: 0,
                continuations: Vec::new(),
            },
            actor_id: 0,
            request: GenerationRequest {
                prompt: "secret question".into(),
                ..Default::default()
            },
            checkpoint: None,
            continuations: vec![Continuation {
                device: "a".into(),
                at_token: 0,
                reason: "ask".into(),
                first_differing_token: None,
                compared_token_ids_hash: None,
            }],
            pause_reason: Some("user".into()),
            executing: false,
        }
    }
    #[test]
    fn encrypted_logs_refuse_wrong_keys_signers_and_modified_bodies() {
        let a = journal("a");
        a.append(&paused()).unwrap();
        let mut backend = a.store.backend().clone();
        let question = eidetic::ManifestId::of_blob(b"secret question");
        let manifest = pollster::block_on(eidetic::manifest::load_manifest(&mut backend, question))
            .unwrap()
            .unwrap();
        assert!(eidetic::seal_marker(&manifest).unwrap().is_some());
        assert!(
            pollster::block_on(eidetic::typed::load_typed::<eidetic::models::OpaqueBlob>(
                &mut backend,
                &mut eidetic::NoFetcher,
                question
            ))
            .is_err(),
            "private evidence cannot be read without the sealer"
        );
        let operations = pollster::block_on(a.operations()).unwrap();
        let op = &operations[0];
        let encoded = op.body.as_ref().unwrap().to_bytes();
        assert!(!String::from_utf8_lossy(&encoded).contains("secret question"));
        let b = journal("b");
        assert!(pollster::block_on(b.accept(op)).unwrap());
        b.unlock([8; 32]);
        assert!(b.load().err().unwrap().contains("authentication"));
        b.unlock([9; 32]);
        assert_eq!(b.load().unwrap()[0].view.question, "secret question");
        let foreign = PersonaRunJournal::new(
            MemoryBackend::new(),
            [7; 32],
            &Ed25519Keypair::from_seed([3; 32]),
            "c".into(),
            [9; 32],
        );
        assert!(
            pollster::block_on(foreign.accept(op))
                .unwrap_err()
                .contains("wrong-persona")
        );
        let mut tampered = op.clone();
        tampered.body = Some(Body::from_bytes(b"changed ciphertext"));
        assert!(pollster::block_on(b.accept(&tampered)).is_err());
        b.lock();
        assert!(b.load().is_err());
    }
    #[test]
    fn two_continuations_are_a_visible_fork_and_never_a_silent_winner() {
        let a = journal("a");
        a.append(&paused()).unwrap();
        let operations = pollster::block_on(a.operations()).unwrap();
        let b = journal("b");
        let c = journal("c");
        for op in &operations {
            pollster::block_on(b.accept(op)).unwrap();
            pollster::block_on(c.accept(op)).unwrap();
        }
        for (journal, device) in [(&b, "b"), (&c, "c")] {
            let mut record = journal.load().unwrap().remove(0);
            record.continuations.push(Continuation {
                device: device.into(),
                at_token: 0,
                reason: "resume".into(),
                first_differing_token: None,
                compared_token_ids_hash: None,
            });
            record.view.state = RunStateV1::Queued;
            record.pause_reason = None;
            journal.append(&record).unwrap();
        }
        for op in pollster::block_on(c.operations()).unwrap() {
            pollster::block_on(b.accept(&op)).unwrap();
        }
        assert!(b.load().err().unwrap().contains("concurrent continuations"));
    }
}
