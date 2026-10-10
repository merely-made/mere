// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Retained checkpoint writes under concrete composition-owned storage.
//! A failed completion never means that the user's command should be repeated.
use crate::access::AccessContext;
use crate::handlers::HandlerRegistry;
use crate::mere_host::{MereHost, MereHostError, SelectedPersonaRef};
use muniment::{Backend, StoreError, WriteOp};
use pandect::addressable_graph::{ResidencyRoots, ResidentGraph};
use pandect::live_graph::{LiveEditError, ObservedCommand};
use pandect::{Applied, Pending, PreparedResidentCheckpoint};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResidentPersistence {
    Rejected,
    AppliedButUnstored { first: u64, end: u64, revision: u64 },
    Durable { generation: String },
    PendingPublication { generation: String, reason: String },
}
#[derive(Debug)]
pub enum ResidentHostError {
    Live(LiveEditError),
    Host(MereHostError),
    Busy,
    OwnershipUnproven,
    UnsupportedStorage,
    StaleCompletion,
    MixedCommit,
}
impl std::fmt::Display for ResidentHostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Live(e) => write!(f, "{e}"),
            Self::Host(e) => write!(f, "{e}"),
            Self::Busy => write!(f, "resident write is pending"),
            Self::OwnershipUnproven => write!(f, "resident storage ownership is unproven"),
            Self::UnsupportedStorage => write!(f, "unsupported resident storage"),
            Self::StaleCompletion => write!(f, "stale resident completion"),
            Self::MixedCommit => write!(f, "mixed resident commit; reopen storage before writing"),
        }
    }
}
impl std::error::Error for ResidentHostError {}
impl From<MereHostError> for ResidentHostError {
    fn from(e: MereHostError) -> Self {
        Self::Host(e)
    }
}
impl From<StoreError> for ResidentHostError {
    fn from(e: StoreError) -> Self {
        Self::Host(e.into())
    }
}

#[derive(Default)]
struct Progress {
    preimages: Option<BTreeMap<String, Option<Vec<u8>>>>,
    uncertain: bool,
    complete: bool,
    mixed: bool,
}
struct Part {
    ops: Vec<WriteOp>,
    final_values: BTreeMap<String, Option<Vec<u8>>>,
    progress: Mutex<Progress>,
}
impl Part {
    fn new(ops: Vec<WriteOp>) -> Self {
        let final_values = ops
            .iter()
            .map(|op| match op {
                WriteOp::Put { key, value } => (key.clone(), Some(value.clone())),
                WriteOp::Delete { key } => (key.clone(), None),
            })
            .collect();
        Self {
            ops,
            final_values,
            progress: Mutex::new(Progress::default()),
        }
    }
    async fn write<B: Backend>(&self, backend: &B) -> Result<(), ResidentHostError> {
        let (complete, mixed, uncertain, preimages) = {
            let p = self.progress.lock().unwrap();
            (p.complete, p.mixed, p.uncertain, p.preimages.clone())
        };
        if mixed {
            return Err(ResidentHostError::MixedCommit);
        }
        if complete {
            return Ok(());
        }
        if uncertain {
            let before = preimages.as_ref().expect("preimages precede apply");
            let mut all_final = true;
            let mut all_before = true;
            for (key, final_value) in &self.final_values {
                let now = backend.get(key).await?;
                all_final &= now == *final_value;
                all_before &= now == before[key];
            }
            if all_final {
                self.progress.lock().unwrap().complete = true;
                return Ok(());
            }
            // Keys unchanged by the batch satisfy both comparisons. A mixture
            // of genuinely changed old/new keys is never safe to overwrite.
            if !all_before {
                self.progress.lock().unwrap().mixed = true;
                return Err(ResidentHostError::MixedCommit);
            }
        }
        if preimages.is_none() {
            let mut before = BTreeMap::new();
            for key in self.final_values.keys() {
                before.insert(key.clone(), backend.get(key).await?);
            }
            self.progress.lock().unwrap().preimages = Some(before);
        }
        // Retained before yielding, including when this future is cancelled.
        self.progress.lock().unwrap().uncertain = true;
        backend.apply(&self.ops).await?;
        self.progress.lock().unwrap().complete = true;
        Ok(())
    }
}
struct WriteState {
    parts: Vec<Part>,
    running: Mutex<bool>,
}
/// Detached I/O work. It shares the exact retained bytes and preimages with
/// its host and never owns a mutable graph or command.
#[derive(Clone)]
pub(crate) struct ResidentWrite {
    state: Arc<WriteState>,
}
struct Running<'a>(&'a Mutex<bool>);
impl Drop for Running<'_> {
    fn drop(&mut self) {
        *self.0.lock().unwrap() = false;
    }
}
impl ResidentWrite {
    pub(crate) async fn write<B: Backend>(&self, backend: &B) -> Result<(), ResidentHostError> {
        {
            let mut running = self.state.running.lock().unwrap();
            if *running {
                return Err(ResidentHostError::Busy);
            }
            *running = true;
        }
        let _running = Running(&self.state.running);
        for part in &self.state.parts {
            part.write(backend).await?;
        }
        Ok(())
    }
}
/// Detached I/O bound to the admitted host's own backend. Callers cannot
/// substitute another store and then acknowledge the original host's batch.
pub struct ResidentIo<B> {
    work: ResidentWrite,
    backend: B,
}
impl<B: Backend + Clone> ResidentIo<B> {
    pub(crate) fn prepare(
        host: &mut MereHost<B>,
        at: SystemTime,
    ) -> Result<Self, ResidentHostError> {
        let work = host.prepare_resident_store(at)?;
        Ok(Self {
            work,
            backend: host.store(),
        })
    }
    pub async fn write(&self) -> Result<(), ResidentHostError> {
        self.work.write(&self.backend).await
    }
    pub(crate) fn work(&self) -> &ResidentWrite {
        &self.work
    }
}
pub(crate) struct ResidentStaged {
    pub(crate) current: Option<PreparedResidentCheckpoint>,
    pub(crate) retired: Vec<Option<Pending>>,
    work: ResidentWrite,
}
impl ResidentStaged {
    pub(crate) fn new(current: PreparedResidentCheckpoint, retired: Vec<Pending>) -> Self {
        let parts = retired
            .iter()
            .map(|p| Part::new(p.ops().to_vec()))
            .chain([Part::new(current.ops().to_vec())])
            .collect();
        Self {
            current: Some(current),
            retired: retired.into_iter().map(Some).collect(),
            work: ResidentWrite {
                state: Arc::new(WriteState {
                    parts,
                    running: Mutex::new(false),
                }),
            },
        }
    }
    pub(crate) fn work(&self) -> ResidentWrite {
        self.work.clone()
    }
    pub(crate) fn matches(&self, work: &ResidentWrite) -> bool {
        Arc::ptr_eq(&self.work.state, &work.state)
    }
    pub(crate) fn complete(&self, index: usize) -> bool {
        self.work.state.parts[index]
            .progress
            .lock()
            .unwrap()
            .complete
    }
    pub(crate) fn running(&self) -> bool {
        *self.work.state.running.lock().unwrap()
    }
}

/// An owner keeps the complete writer private, with a last coherent resident
/// reader. Only concrete factories below can grant write admission.
pub struct ResidentHost<B> {
    host: Option<MereHost<B>>,
    reader: Option<ResidentGraph<B>>,
    roots: ResidencyRoots,
    applied: Option<Applied>,
    publication: Option<String>,
    #[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
    directory: Option<std::path::PathBuf>,
}
impl<B: Backend + Clone> ResidentHost<B> {
    fn owned(mut host: MereHost<B>) -> Self {
        host.admit_resident();
        Self {
            host: Some(host),
            reader: None,
            roots: Default::default(),
            applied: None,
            publication: None,
            #[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
            directory: None,
        }
    }
    /// Generic callers cannot certify backend semantics or writer ownership.
    pub fn try_from_host(_host: MereHost<B>) -> Result<Self, ResidentHostError> {
        Err(ResidentHostError::OwnershipUnproven)
    }
    pub fn reader(&self) -> Option<&ResidentGraph<B>> {
        self.reader.as_ref()
    }
    pub fn session(&self) -> &pandect::GraphSession<B> {
        self.host.as_ref().expect("open resident").graph_session()
    }
    pub fn is_busy(&self) -> bool {
        self.host.as_ref().is_none_or(|h| h.resident_busy()) || self.publication.is_some()
    }
    pub fn edit(
        &mut self,
        via: &str,
        commands: Vec<ObservedCommand>,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let applied = host.edit_resident(via, commands)?;
        self.applied = Some(applied);
        // Preparation failure retains the already-applied identity and Busy.
        let _ = host.prepare_resident_store(at);
        Ok(self.unstored())
    }
    pub fn undo(
        &mut self,
        via: &str,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let Some(reverted) = host.undo_now(via)? else {
            return Ok(ResidentPersistence::Rejected);
        };
        self.applied = Some(reverted.applied);
        host.hold_resident();
        let _ = host.prepare_resident_store(at);
        Ok(self.unstored())
    }
    pub fn redo(
        &mut self,
        via: &str,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.is_busy() {
            return Err(ResidentHostError::Busy);
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let Some(reverted) = host.redo_now(via)? else {
            return Ok(ResidentPersistence::Rejected);
        };
        self.applied = Some(reverted.applied);
        host.hold_resident();
        let _ = host.prepare_resident_store(at);
        Ok(self.unstored())
    }
    fn unstored(&self) -> ResidentPersistence {
        match &self.applied {
            Some(a) => ResidentPersistence::AppliedButUnstored {
                first: a.first.0,
                end: a.end.0,
                revision: a.revision,
            },
            None => ResidentPersistence::Rejected,
        }
    }
    pub async fn persist(
        &mut self,
        at: SystemTime,
    ) -> Result<ResidentPersistence, ResidentHostError> {
        if self.publication.is_some() {
            return self.refresh().await;
        }
        let host = self.host.as_mut().ok_or(ResidentHostError::Busy)?;
        let work = match host.prepare_resident_store(at) {
            Ok(w) => w,
            Err(_) => return Ok(self.unstored()),
        };
        let result = work.write(&host.store()).await;
        let generation = host.install_resident_store(&work)?;
        if let Some(generation) = generation {
            self.applied = None;
            self.publication = Some(generation);
            return self.refresh().await;
        }
        if matches!(result, Err(ResidentHostError::MixedCommit)) {
            return Err(ResidentHostError::MixedCommit);
        }
        Ok(self.unstored())
    }
    pub async fn refresh(&mut self) -> Result<ResidentPersistence, ResidentHostError> {
        let generation = self.publication.clone().ok_or(ResidentHostError::Busy)?;
        let host = self.host.as_ref().ok_or(ResidentHostError::Busy)?;
        let result = if let Some(reader) = &mut self.reader {
            reader.refresh().await.map_err(MereHostError::from)
        } else {
            match pandect::MereSessions::new(host.store())
                .open_resident_graph(host.graph_session().id(), self.roots.clone())
                .await
            {
                Ok(reader) => {
                    self.reader = Some(reader);
                    Ok(())
                },
                Err(e) => Err(e.into()),
            }
        };
        match result {
            Ok(()) => {
                self.publication = None;
                Ok(ResidentPersistence::Durable { generation })
            },
            Err(error) => Ok(ResidentPersistence::PendingPublication {
                generation,
                reason: error.to_string(),
            }),
        }
    }
}

#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
impl ResidentHost<muniment::DirectoryBackend> {
    pub async fn open_directory(
        path: impl AsRef<std::path::Path>,
        id: pandect::SessionId,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
    ) -> Result<Self, ResidentHostError> {
        let path = path.as_ref().to_owned();
        let backend = muniment::DirectoryBackend::open(&path)?;
        let host = MereHost::open_session(backend, id, persona, handlers, access).await?;
        let mut owner = Self::owned(host);
        owner.directory = Some(path);
        Ok(owner)
    }
    /// Discard every owner clone before Directory redo recovery. Reopening
    /// reads authoritative history and never resubmits the previous command.
    pub async fn reopen(
        mut self,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
    ) -> Result<Self, ResidentHostError> {
        let id = self.session().id();
        let path = self
            .directory
            .take()
            .ok_or(ResidentHostError::OwnershipUnproven)?;
        let work = self.host.as_ref().and_then(|h| h.retained_resident_work());
        drop(self.reader.take());
        drop(self.host.take());
        // Opening finishes a durable redo batch before any comparison. The
        // retained worker owns bytes/preimages only, so it cannot hold the lock.
        if let Some(work) = work {
            let recovered = muniment::DirectoryBackend::open(&path)?;
            work.write(&recovered).await?;
            drop(recovered);
        }
        Self::open_directory(path, id, persona, handlers, access).await
    }
}
#[cfg(all(feature = "resident-storage", not(target_arch = "wasm32")))]
impl ResidentHost<muniment::RedbBackend> {
    pub async fn open_redb(
        path: impl AsRef<std::path::Path>,
        id: pandect::SessionId,
        persona: SelectedPersonaRef,
        handlers: HandlerRegistry,
        access: AccessContext,
    ) -> Result<Self, ResidentHostError> {
        let backend = muniment::RedbBackend::open(path)?;
        Ok(Self::owned(
            MereHost::open_session(backend, id, persona, handlers, access).await?,
        ))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    use muniment::MemoryBackend;
    use pandect::addressable_graph::GraphAddress;
    use pandect::live_graph::{LiveCheck, LiveCommand};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    #[derive(Clone, Default)]
    struct Fault {
        inner: MemoryBackend,
        mode: Arc<AtomicUsize>,
        calls: Arc<Mutex<Vec<Vec<WriteOp>>>>,
    }
    #[async_trait::async_trait]
    impl Backend for Fault {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            if self.mode.load(Ordering::SeqCst) == 4 {
                return Err(StoreError::Backend("read fault".into()));
            }
            self.inner.get(key).await
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            self.inner.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.inner.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.inner.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.inner.scan(start, end).await
        }
        async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
            self.calls.lock().unwrap().push(ops.to_vec());
            let mode = self.mode.load(Ordering::SeqCst);
            if mode == 1 {
                return Err(StoreError::Backend("before apply".into()));
            }
            if mode == 3 {
                for op in ops {
                    let (key, value) = match op {
                        WriteOp::Put { key, value } => (key, Some(value.clone())),
                        WriteOp::Delete { key } => (key, None),
                    };
                    if self.inner.get(key).await? != value {
                        self.inner.apply(std::slice::from_ref(op)).await?;
                        break;
                    }
                }
                return Err(StoreError::Backend("mixed commit".into()));
            }
            self.inner.apply(ops).await?;
            if mode == 6 {
                std::future::pending::<()>().await;
            }
            if mode == 2 {
                return Err(StoreError::Backend("lost completion".into()));
            }
            if mode == 5 {
                self.mode.store(4, Ordering::SeqCst);
            }
            Ok(())
        }
    }
    fn persona() -> SelectedPersonaRef {
        SelectedPersonaRef {
            persona: "test-author".into(),
            profile: "test".into(),
        }
    }
    fn access() -> AccessContext {
        AccessContext {
            persona: "test-author".into(),
            device: "test".into(),
            at_ms: 0,
        }
    }
    fn command(n: u128) -> Vec<ObservedCommand> {
        let id = Uuid::from_u128(n);
        vec![ObservedCommand {
            command: LiveCommand::CreateAddress {
                id,
                address: format!("https://resident.test/{n}"),
                title: format!("node {n}"),
            },
            checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
        }]
    }
    async fn owner() -> (ResidentHost<Fault>, Fault) {
        let backend = Fault::default();
        let mut host = MereHost::empty(
            backend.clone(),
            persona(),
            HandlerRegistry::default(),
            access(),
        );
        host.begin_profiled_session(
            mere::kernel::graph::Graph::new(),
            Some(pandect::graph_placement::PlacementProfile::RecordedStrataV1),
        )
        .unwrap();
        let mut owner = ResidentHost::owned(host);
        assert!(matches!(
            owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
            ResidentPersistence::Durable { .. }
        ));
        backend.calls.lock().unwrap().clear();
        (owner, backend)
    }
    #[test]
    fn retry_twice_keeps_exact_batch_author_and_change_identity() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let applied = owner
                .edit("turnstone", command(10), SystemTime::UNIX_EPOCH)
                .unwrap();
            let cursor = owner.session().journal().live_cursor();
            let count = owner.session().changes().len();
            backend.mode.store(1, Ordering::SeqCst);
            for _ in 0..2 {
                assert_eq!(owner.persist(SystemTime::now()).await.unwrap(), applied);
                assert_eq!(owner.session().journal().live_cursor(), cursor);
                assert_eq!(owner.session().changes().len(), count);
                assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
                assert!(matches!(
                    owner.edit("turnstone", command(11), SystemTime::now()),
                    Err(ResidentHostError::Busy)
                ));
                assert!(matches!(
                    owner.undo("turnstone", SystemTime::now()),
                    Err(ResidentHostError::Busy)
                ));
                assert!(matches!(
                    owner.redo("turnstone", SystemTime::now()),
                    Err(ResidentHostError::Busy)
                ));
                let host = owner.host.as_mut().unwrap();
                assert!(matches!(
                    host.apply_edits(Vec::new()),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.set_view_now("app", "main", Default::default()),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.begin_profiled_session(Default::default(), None),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.prepare_store(0),
                    Err(MereHostError::ResidentBusy)
                ));
                assert!(matches!(
                    host.persist(0).await,
                    Err(MereHostError::ResidentBusy)
                ));
            }
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            let calls = backend.calls.lock().unwrap();
            assert_eq!(calls.len(), 3);
            assert_eq!(calls[0], calls[1]);
            assert_eq!(calls[1], calls[2]);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert_eq!(
                owner.session().changes().last().unwrap().author,
                pandect::Author::person("test-author").via("turnstone")
            );
            assert_eq!(owner.reader().unwrap().graph().node_count(), 1);
        });
    }
    #[test]
    fn applied_error_acknowledges_once_without_a_second_apply() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(20), SystemTime::UNIX_EPOCH)
                .unwrap();
            let cursor = owner.session().journal().live_cursor();
            backend.mode.store(2, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::AppliedButUnstored { .. }
            ));
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert!(!owner.session().has_unstored());
        });
    }
    #[test]
    fn cancelled_io_retains_uncertainty_and_reconciles_without_reapplying() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(25), SystemTime::UNIX_EPOCH)
                .unwrap();
            let cursor = owner.session().journal().live_cursor();
            let io = owner
                .host
                .as_mut()
                .unwrap()
                .prepare_resident_io(SystemTime::UNIX_EPOCH)
                .unwrap();
            backend.mode.store(6, Ordering::SeqCst);
            {
                let mut future = std::pin::pin!(io.write());
                let mut cx = std::task::Context::from_waker(std::task::Waker::noop());
                assert!(std::future::Future::poll(future.as_mut(), &mut cx).is_pending());
                assert!(matches!(io.write().await, Err(ResidentHostError::Busy)));
                assert!(matches!(
                    owner.host.as_mut().unwrap().install_resident_io(&io),
                    Err(ResidentHostError::Busy)
                ));
                assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
            }
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
            assert_eq!(owner.reader().unwrap().graph().node_count(), 1);
        });
    }
    #[test]
    fn mixed_keys_stop_writing_and_keep_old_reader() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(30), SystemTime::UNIX_EPOCH)
                .unwrap();
            backend.mode.store(3, Ordering::SeqCst);
            owner.persist(SystemTime::now()).await.unwrap();
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await,
                Err(ResidentHostError::MixedCommit)
            ));
            assert!(matches!(
                owner.persist(SystemTime::now()).await,
                Err(ResidentHostError::MixedCommit)
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
            assert!(owner.is_busy());
        });
    }
    #[test]
    fn preparation_codec_failure_retains_applied_identity() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner.host.as_mut().unwrap().resident_prepare_error =
                Some(StoreError::Codec("injected preparation codec fault".into()));
            let applied = owner
                .edit("turnstone", command(40), SystemTime::UNIX_EPOCH)
                .unwrap();
            assert!(matches!(
                applied,
                ResidentPersistence::AppliedButUnstored { .. }
            ));
            assert!(backend.calls.lock().unwrap().is_empty());
            assert!(owner.is_busy());
            let cursor = owner.session().journal().live_cursor();
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(owner.session().journal().live_cursor(), cursor);
        });
    }
    #[test]
    fn failed_reader_refresh_retries_only_publication() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            owner
                .edit("turnstone", command(50), SystemTime::UNIX_EPOCH)
                .unwrap();
            backend.mode.store(5, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::PendingPublication { .. }
            ));
            assert_eq!(owner.reader().unwrap().graph().node_count(), 0);
            assert!(!owner.session().has_unstored());
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.reader().unwrap().graph().node_count(), 1);
        });
    }
    #[test]
    fn foreign_and_duplicate_completion_are_refused() {
        pollster::block_on(async {
            let (mut first, backend) = owner().await;
            let (mut second, _) = owner().await;
            first
                .edit("turnstone", command(60), SystemTime::UNIX_EPOCH)
                .unwrap();
            second
                .edit("turnstone", command(61), SystemTime::UNIX_EPOCH)
                .unwrap();
            let work = first
                .host
                .as_mut()
                .unwrap()
                .prepare_resident_store(SystemTime::now())
                .unwrap();
            assert!(matches!(
                second.host.as_mut().unwrap().install_resident_store(&work),
                Err(ResidentHostError::StaleCompletion)
            ));
            work.write(&backend).await.unwrap();
            assert!(
                first
                    .host
                    .as_mut()
                    .unwrap()
                    .install_resident_store(&work)
                    .unwrap()
                    .is_some()
            );
            assert!(matches!(
                first.host.as_mut().unwrap().install_resident_store(&work),
                Err(ResidentHostError::StaleCompletion)
            ));
        });
    }
    #[test]
    fn detached_io_uses_only_its_owner_store() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let foreign = Fault::default();
            owner
                .edit("turnstone", command(65), SystemTime::UNIX_EPOCH)
                .unwrap();
            let host = owner.host.as_mut().unwrap();
            let io = host.prepare_resident_io(SystemTime::UNIX_EPOCH).unwrap();
            io.write().await.unwrap();
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert!(foreign.calls.lock().unwrap().is_empty());
            assert!(host.install_resident_io(&io).unwrap().is_some());
            // Generic preparation never grants owner admission, even for the
            // same concrete backend type used by the admitted test owner.
            let mut unowned =
                MereHost::empty(foreign, persona(), HandlerRegistry::default(), access());
            assert!(matches!(
                unowned.prepare_resident_io(SystemTime::UNIX_EPOCH),
                Err(ResidentHostError::OwnershipUnproven)
            ));
        });
    }
    #[test]
    fn publication_without_an_edit_can_remain_pending() {
        pollster::block_on(async {
            let (mut owner, backend) = owner().await;
            let cursor = owner.session().journal().live_cursor();
            backend.mode.store(5, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::UNIX_EPOCH).await.unwrap(),
                ResidentPersistence::PendingPublication { .. }
            ));
            assert!(owner.is_busy());
            backend.mode.store(0, Ordering::SeqCst);
            assert!(matches!(
                owner.persist(SystemTime::now()).await.unwrap(),
                ResidentPersistence::Durable { .. }
            ));
            assert_eq!(backend.calls.lock().unwrap().len(), 1);
            assert_eq!(owner.session().journal().live_cursor(), cursor);
        });
    }
    #[test]
    fn retired_success_is_acknowledged_when_current_batch_fails() {
        pollster::block_on(async {
            let backend = Fault::default();
            let mut host = MereHost::empty(
                backend.clone(),
                persona(),
                HandlerRegistry::default(),
                access(),
            );
            let retired_id = host.graph_session().id();
            host.begin_profiled_session(
                Default::default(),
                Some(pandect::graph_placement::PlacementProfile::RecordedStrataV1),
            )
            .unwrap();
            host.admit_resident();
            let work = host.prepare_resident_store(SystemTime::UNIX_EPOCH).unwrap();
            work.state.parts[0].write(&backend).await.unwrap();
            backend.mode.store(1, Ordering::SeqCst);
            assert!(work.write(&backend).await.is_err());
            assert!(host.install_resident_store(&work).unwrap().is_none());
            assert!(
                pandect::MereSessions::new(backend.clone())
                    .open(retired_id)
                    .await
                    .is_ok()
            );
            backend.mode.store(0, Ordering::SeqCst);
            work.write(&backend).await.unwrap();
            assert!(host.install_resident_store(&work).unwrap().is_some());
            let calls = backend.calls.lock().unwrap();
            assert_eq!(calls.len(), 3);
            assert_eq!(calls[1], calls[2]);
            assert!(!host.has_unstored());
        });
    }
}
