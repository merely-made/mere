// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The one compare-and-replace over a muniment slot.
//!
//! A holder of a slot's bytes (a rule workspace, a block document, a Forme
//! layout) writes back only if the slot still holds what it loaded. The read
//! and the write share one `Backend::transact`, so no other writer lands
//! between them; the holders keep their drafts when this refuses.

use std::sync::{Arc, Mutex};

use muniment::{Backend, StoreError, WriteOp};

/// A backend the guard can drive. Native `Backend::transact` futures are `Send`,
/// which needs the backend `Sync`; the browser's backends are not, and need not be.
#[cfg(not(target_arch = "wasm32"))]
pub trait SlotBackend: Backend + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<B: Backend + Sync> SlotBackend for B {}
#[cfg(target_arch = "wasm32")]
pub trait SlotBackend: Backend {}
#[cfg(target_arch = "wasm32")]
impl<B: Backend> SlotBackend for B {}

/// What a compare-and-replace did to the slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The slot held the baseline; it now holds the new bytes.
    Replaced,
    /// The slot held something else; nothing was written.
    Conflict,
}

/// Replace `key` with `bytes` if it still holds `baseline` (`None`: empty).
///
/// A read error inside the transaction is an `Err`, never a `Conflict`, and a
/// commit failure is an `Err` even where the comparison matched.
pub async fn replace<B: SlotBackend>(
    backend: &B,
    key: &str,
    baseline: Option<&[u8]>,
    bytes: &[u8],
) -> Result<Outcome, StoreError> {
    let seen = Arc::new(Mutex::new(None));
    let record = seen.clone();
    let (key, baseline, value) = (key.to_owned(), baseline.map(<[u8]>::to_vec), bytes.to_vec());
    backend
        .transact(Box::new(move |reader| {
            let held = reader.get(&key).map(|held| held == baseline);
            let writes = match held {
                Ok(true) => vec![WriteOp::Put { key, value }],
                _ => Vec::new(),
            };
            *record.lock().expect("slot guard outcome") = Some(held);
            writes
        }))
        .await?;
    let held = seen.lock().expect("slot guard outcome").take();
    match held {
        Some(Ok(true)) => Ok(Outcome::Replaced),
        Some(Ok(false)) => Ok(Outcome::Conflict),
        Some(Err(error)) => Err(error),
        None => Err(StoreError::Backend(
            "backend did not run the transaction".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use muniment::{MemoryBackend, TransactFn, TransactionReader};

    const KEY: &str = "slot/guard";

    #[test]
    fn a_replace_with_a_stale_baseline_refuses_and_leaves_the_slot() {
        pollster::block_on(async {
            let backend = MemoryBackend::new();
            backend.put(KEY, b"one").await.unwrap();
            let outcome = replace(&backend, KEY, Some(b"one"), b"two").await;
            assert_eq!(outcome, Ok(Outcome::Replaced));
            let stale = replace(&backend, KEY, Some(b"one"), b"stale").await;
            assert_eq!(stale, Ok(Outcome::Conflict));
            assert_eq!(backend.get(KEY).await.unwrap(), Some(b"two".to_vec()));
        });
    }

    #[test]
    fn an_empty_baseline_replaces_only_an_empty_slot() {
        pollster::block_on(async {
            let backend = MemoryBackend::new();
            let claimed = replace(&backend, KEY, Some(b"claimed"), b"early").await;
            assert_eq!(claimed, Ok(Outcome::Conflict));
            assert_eq!(backend.get(KEY).await.unwrap(), None);
            let first = replace(&backend, KEY, None, b"first").await;
            assert_eq!(first, Ok(Outcome::Replaced));
            let second = replace(&backend, KEY, None, b"second").await;
            assert_eq!(second, Ok(Outcome::Conflict));
            assert_eq!(backend.get(KEY).await.unwrap(), Some(b"first".to_vec()));
        });
    }

    /// A memory backend whose transaction cannot read, or cannot run at all.
    struct Broken {
        inner: MemoryBackend,
        transactional: bool,
    }
    #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
    #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
    impl Backend for Broken {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
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
            self.inner.apply(ops).await
        }
        async fn transact(&self, f: TransactFn) -> Result<(), StoreError> {
            struct Unreadable;
            impl TransactionReader for Unreadable {
                fn get(&self, _: &str) -> Result<Option<Vec<u8>>, StoreError> {
                    Err(StoreError::Backend("unreadable".into()))
                }
                fn list(&self, _: &str) -> Result<Vec<String>, StoreError> {
                    Ok(Vec::new())
                }
            }
            if !self.transactional {
                return Err(StoreError::NotTransactional);
            }
            let ops = f(&Unreadable);
            self.inner.apply(&ops).await
        }
    }

    #[test]
    fn a_read_error_or_a_missing_transaction_is_an_error_not_a_conflict() {
        pollster::block_on(async {
            let cases = [
                (true, StoreError::Backend("unreadable".into())),
                (false, StoreError::NotTransactional),
            ];
            for (transactional, expected) in cases {
                let backend = Broken {
                    inner: MemoryBackend::new(),
                    transactional,
                };
                let outcome = replace(&backend, KEY, None, b"never").await;
                assert_eq!(outcome, Err(expected));
                assert_eq!(backend.get(KEY).await.unwrap(), None);
            }
        });
    }
}
