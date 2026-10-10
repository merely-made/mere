// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Opt-in endpoint fixtures over a fresh real Directory lease.
//! This is fault support, not qualification of production owner composition.

use super::*;
use std::path::Path;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DirectoryFault {
    #[default]
    None,
    AfterApplyError,
    ReadRecordsAfterApply,
    ReadRecords,
}

#[derive(Default)]
struct FaultState {
    mode: DirectoryFault,
    batches: Vec<Vec<WriteOp>>,
    intercepted_reads: Vec<String>,
}

/// Cannot be constructed from an existing backend or arbitrary implementation.
pub struct DirectoryFixture {
    cache: DirectoryHostCache<FixtureDirectory>,
    faults: DirectoryFaultControl,
}

impl DirectoryFixture {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StoreError> {
        let root = path.as_ref().to_owned();
        let faults = Arc::new(Mutex::new(FaultState::default()));
        let backend = FixtureDirectory {
            inner: muniment::DirectoryBackend::open(&root)?,
            faults: faults.clone(),
        };
        Ok(Self {
            cache: DirectoryHostCache::from_fresh_backend(backend, root),
            faults: DirectoryFaultControl { faults },
        })
    }
    pub fn into_parts(self) -> (DirectoryHostCache<FixtureDirectory>, DirectoryFaultControl) {
        (self.cache, self.faults)
    }
}

pub struct DirectoryFaultControl {
    faults: Arc<Mutex<FaultState>>,
}

impl DirectoryFaultControl {
    pub fn fault(&self, mode: DirectoryFault) {
        self.faults.lock().unwrap().mode = mode;
    }

    pub fn clear_batches(&self) {
        let mut state = self.faults.lock().unwrap();
        state.batches.clear();
        state.intercepted_reads.clear();
    }

    pub fn intercepted_reads(&self) -> Vec<String> {
        self.faults.lock().unwrap().intercepted_reads.clone()
    }

    pub fn batches(&self) -> Vec<Vec<WriteOp>> {
        self.faults.lock().unwrap().batches.clone()
    }
}

/// Only the fresh concrete fixture can create this delegated handle.
#[derive(Clone)]
pub struct FixtureDirectory {
    inner: muniment::DirectoryBackend,
    faults: Arc<Mutex<FaultState>>,
}

impl FixtureDirectory {
    pub fn root(&self) -> &Path {
        self.inner.root()
    }
}

#[async_trait::async_trait]
impl Backend for FixtureDirectory {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        {
            let mut state = self.faults.lock().unwrap();
            // A changed generation always loads its catalog. Undo can reuse
            // every remaining node record and perform no node read at all.
            if state.mode == DirectoryFault::ReadRecords
                && (key.contains("/resident/v1/nodes/") || key.contains("/resident/v1/catalog/"))
            {
                state.intercepted_reads.push(key.to_owned());
                return Err(StoreError::Backend("fixture reader record failure".into()));
            }
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
        let mode = {
            let mut state = self.faults.lock().unwrap();
            state.batches.push(ops.to_vec());
            state.mode
        };
        self.inner.apply(ops).await?;
        match mode {
            DirectoryFault::AfterApplyError => {
                self.faults.lock().unwrap().mode = DirectoryFault::None;
                Err(StoreError::Backend(
                    "fixture lost completion after apply".into(),
                ))
            },
            DirectoryFault::ReadRecordsAfterApply => {
                self.faults.lock().unwrap().mode = DirectoryFault::ReadRecords;
                Ok(())
            },
            _ => Ok(()),
        }
    }
    async fn transact(&self, f: muniment::TransactFn) -> Result<(), StoreError> {
        self.inner.transact(f).await
    }
}

/// Read-only evidence for an opaque load result, available only with the
/// explicit native test-support module. It grants no completion construction.
impl<B> HostRefreshCompletion<B> {
    pub fn failed_for_test(&self) -> bool {
        self.result.is_err()
    }
}
