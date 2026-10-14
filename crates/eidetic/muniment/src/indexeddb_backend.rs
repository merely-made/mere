// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Exclusive database lifetime admission and supervised IndexedDB transactions.
//! Native JS event owners retain requests and transactions independently of Rust
//! waiters. A canceled open drains before releasing its mandatory Web Lock.

use std::{collections::HashMap, rc::Rc};

use async_trait::async_trait;
use futures_channel::oneshot;
use js_sys::{Array, Promise, Uint8Array};
use wasm_bindgen::{JsCast, prelude::*};
use wasm_bindgen_futures::{JsFuture, spawn_local};

use crate::backend::{TransactFn, TransactionReader};
use crate::{Backend, StoreError, WriteOp};

#[wasm_bindgen(module = "/src/indexeddb_lifetime.js")]
extern "C" {
    #[wasm_bindgen(js_name = startOpen, catch)]
    fn start_open(name: &str, store: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = openResult)]
    fn open_result(state: &JsValue) -> Promise;
    #[wasm_bindgen(js_name = openStatus)]
    fn open_status(state: &JsValue) -> String;
    #[wasm_bindgen(js_name = cancelOpen)]
    fn cancel_open(state: &JsValue);
    #[wasm_bindgen(js_name = closeOwner)]
    fn close_owner(owner: &JsValue) -> Promise;
    #[wasm_bindgen(js_name = ownerActive)]
    fn owner_active(owner: &JsValue) -> bool;
    #[wasm_bindgen(js_name = readKey)]
    fn read_key(owner: &JsValue, key: &str) -> Promise;
    #[wasm_bindgen(js_name = readKeys)]
    fn read_keys(owner: &JsValue) -> Promise;
    #[wasm_bindgen(js_name = applyOps)]
    fn apply_ops(owner: &JsValue, ops: &Array) -> Promise;
    #[wasm_bindgen(js_name = transactOps)]
    fn transact_ops(owner: &JsValue, compute: &js_sys::Function) -> Promise;
}

/// An opening attempt, including a blocked or canceled request still draining.
/// Dropping this handle requests cancellation; it never abandons a submitted
/// request or asserts that its schema upgrade rolled back.
pub struct IndexedDbOpenAttempt {
    state: JsValue,
    delivered: bool,
}
impl IndexedDbOpenAttempt {
    pub fn start(database: &str, store: &str) -> Result<Self, StoreError> {
        Ok(Self {
            state: start_open(database, store).map_err(js_error)?,
            delivered: false,
        })
    }
    /// `blocked-draining` refuses writer admission while awaiting the old tab.
    pub fn status(&self) -> String {
        open_status(&self.state)
    }
    pub fn cancel(&self) {
        cancel_open(&self.state);
    }
    pub async fn finish(mut self) -> Result<IndexedDbBackend, StoreError> {
        let owner = JsFuture::from(open_result(&self.state))
            .await
            .map_err(js_error)?;
        self.delivered = true;
        Ok(IndexedDbBackend {
            owner: Rc::new(Owner(owner)),
        })
    }
}
impl Drop for IndexedDbOpenAttempt {
    fn drop(&mut self) {
        if !self.delivered {
            self.cancel();
        }
    }
}

struct Owner(JsValue);
impl Drop for Owner {
    fn drop(&mut self) {
        let _ = close_owner(&self.0);
    }
}
/// Internal clones share one admitted connection. Independently opening any
/// store in the same database refuses until this connection and all I/O drain.
#[derive(Clone)]
pub struct IndexedDbBackend {
    owner: Rc<Owner>,
}
impl IndexedDbBackend {
    pub async fn open(database: &str, store: &str) -> Result<Self, StoreError> {
        IndexedDbOpenAttempt::start(database, store)?.finish().await
    }
    /// Invalidation is terminal for every clone. Applications must reopen from
    /// authoritative storage, rather than replaying an uncertain command.
    pub fn is_open(&self) -> bool {
        owner_active(&self.owner.0)
    }
    pub async fn close(&self) -> Result<(), StoreError> {
        JsFuture::from(close_owner(&self.owner.0))
            .await
            .map_err(js_error)?;
        Ok(())
    }
}

fn operations(ops: &[WriteOp]) -> Array {
    ops.iter()
        .map(|op| {
            let pair = Array::new();
            match op {
                WriteOp::Put { key, value } => {
                    pair.push(&JsValue::from_str(key));
                    pair.push(&Uint8Array::from(value.as_slice()));
                },
                WriteOp::Delete { key } => {
                    pair.push(&JsValue::from_str(key));
                    pair.push(&JsValue::NULL);
                },
            }
            JsValue::from(pair)
        })
        .collect()
}
struct SnapshotReader(HashMap<String, Vec<u8>>);
impl TransactionReader for SnapshotReader {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        Ok(self.0.get(key).cloned())
    }
    fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        Ok(self
            .0
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect())
    }
}
#[async_trait(?Send)]
impl Backend for IndexedDbBackend {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        let value = JsFuture::from(read_key(&self.owner.0, key))
            .await
            .map_err(js_error)?;
        Ok((!value.is_undefined()).then(|| Uint8Array::new(&value).to_vec()))
    }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        self.apply(&[WriteOp::Put {
            key: key.into(),
            value: bytes.to_vec(),
        }])
        .await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.apply(&[WriteOp::Delete { key: key.into() }]).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        let value = JsFuture::from(read_keys(&self.owner.0))
            .await
            .map_err(js_error)?;
        Ok(Array::from(&value)
            .iter()
            .filter_map(|key| key.as_string())
            .filter(|key| key.starts_with(prefix))
            .collect())
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        let mut keys = self
            .list("")
            .await?
            .into_iter()
            .filter(|key| key.as_str() >= start && key.as_str() < end)
            .collect::<Vec<_>>();
        keys.sort();
        Ok(keys)
    }
    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        JsFuture::from(apply_ops(&self.owner.0, &operations(ops)))
            .await
            .map_err(js_error)?;
        Ok(())
    }
    async fn transact(&self, f: TransactFn) -> Result<(), StoreError> {
        let mut compute = Some(f);
        let callback =
            Closure::<dyn FnMut(Array, Array) -> Array>::new(move |keys: Array, values: Array| {
                let snapshot = SnapshotReader(
                    keys.iter()
                        .zip(values.iter())
                        .filter_map(|(key, value)| {
                            key.as_string()
                                .map(|key| (key, Uint8Array::new(&value).to_vec()))
                        })
                        .collect(),
                );
                operations(&compute.take().expect("transaction computes once")(
                    &snapshot,
                ))
            });
        let promise = transact_ops(&self.owner.0, callback.as_ref().unchecked_ref());
        let (send, receive) = oneshot::channel();
        // The event-owned JS transaction retains its lease; this supervisor
        // retains the Rust closure through the terminal event even on cancel.
        spawn_local(async move {
            let outcome = JsFuture::from(promise).await.map(|_| ()).map_err(js_error);
            drop(callback);
            let _ = send.send(outcome);
        });
        receive
            .await
            .map_err(|_| StoreError::Backend("transaction supervisor ended".into()))?
    }
}
fn js_error(value: JsValue) -> StoreError {
    let field = |name: &str| js_sys::Reflect::get(&value, &JsValue::from_str(name)).ok();
    let kind = field("kind").and_then(|v| v.as_string());
    let message = field("message")
        .and_then(|v| v.as_string())
        .unwrap_or_else(|| format!("{value:?}"));
    let name = field("name")
        .and_then(|v| v.as_string())
        .unwrap_or_default();
    match kind.as_deref() {
        Some("busy") => StoreError::OwnershipBusy,
        Some("unavailable") => StoreError::OwnershipUnavailable(message),
        Some("blocked") => StoreError::UpgradeBlocked,
        Some("canceled") => StoreError::OpenCanceled {
            schema_upgraded: field("schemaUpgraded")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        },
        Some("schema") => StoreError::InvalidSchema(message),
        Some("lost") => StoreError::OwnershipLost(message),
        _ => StoreError::Backend(format!("{name}: {message}")),
    }
}
