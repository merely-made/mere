// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0


//! Sealing a store's values: a muniment backend whose values are sealed by
//! the persona's custody before they reach the inner backend. A mere's
//! retained store is sealed this way when its owner composes it over custody.

use std::sync::Arc;

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use muniment::Backend;

use async_trait::async_trait;
use eidetic::{Hash, PayloadSealer, SealedBlobRef};
use muniment::{StoreError, WriteOp};

const SEALED_FORMAT: &str = "pandect-retained-value-v1";

/// A sealer shared across the store's handles. Custody-backed sealers are
/// `Send + Sync`; the seal and unseal calls may block briefly on custody.
pub type SharedSealer = Arc<dyn PayloadSealer + Send + Sync>;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    format: String,
    hash: Hash,
    seal: SealedBlobRef,
    ciphertext: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Inner {
    key: String,
    // Inside the ciphertext, so the exposed digest cannot be used to guess a
    // small page or to correlate two identical writes.
    salt: [u8; 16],
    bytes: Vec<u8>,
}

fn refusal(message: &str) -> StoreError {
    StoreError::Backend(message.to_owned())
}

/// A backend whose values are sealed by a [`PayloadSealer`] before they reach
/// the inner backend, and authenticated against their key on the way out.
///
/// Keys stay visible (the BLAKE3 of an address and of a text are visible; the
/// text, the title-bearing records and the setting are not). There is no
/// cleartext fallback: an unreadable value is an error, never an empty slot.
/// Authentication detects tampering and transplantation between keys; it does
/// not detect replay of an older valid value at the same key.
pub struct SealedValues<B> {
    backend: B,
    sealer: SharedSealer,
}

impl<B: Clone> Clone for SealedValues<B> {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            sealer: Arc::clone(&self.sealer),
        }
    }
}

impl<B> SealedValues<B> {
    pub fn new(backend: B, sealer: SharedSealer) -> Self {
        Self { backend, sealer }
    }

    fn seal(&self, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        let cleartext = serde_json::to_vec(&Inner {
            key: key.to_owned(),
            salt: *Uuid::new_v4().as_bytes(),
            bytes: bytes.to_vec(),
        })
        .map_err(|_| refusal("encode sealed value"))?;
        let hash = Hash::of(&cleartext);
        let (ciphertext, seal) = self
            .sealer
            .seal(&hash, &cleartext)
            .map_err(|error| refusal(&format!("seal value: {error}")))?;
        serde_json::to_vec(&Envelope {
            format: SEALED_FORMAT.to_owned(),
            hash,
            seal,
            ciphertext,
        })
        .map_err(|_| refusal("encode sealed envelope"))
    }

    fn unseal(&self, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        let envelope: Envelope =
            serde_json::from_slice(bytes).map_err(|_| refusal("value is not sealed"))?;
        if envelope.format != SEALED_FORMAT {
            return Err(refusal("unsupported sealed value format"));
        }
        let cleartext = self
            .sealer
            .unseal(&envelope.hash, &envelope.seal, &envelope.ciphertext)
            .map_err(|error| refusal(&format!("unseal value: {error}")))?;
        if Hash::of(&cleartext) != envelope.hash {
            return Err(refusal("sealed value content hash mismatch"));
        }
        let inner: Inner =
            serde_json::from_slice(&cleartext).map_err(|_| refusal("decode sealed value"))?;
        if inner.key != key {
            return Err(refusal("sealed value key mismatch"));
        }
        Ok(inner.bytes)
    }
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl<B: Backend + Sync> Backend for SealedValues<B> {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.backend
            .get(key)
            .await?
            .map(|bytes| self.unseal(key, &bytes))
            .transpose()
    }

    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        self.backend.put(key, &self.seal(key, bytes)?).await
    }

    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.backend.delete(key).await
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.backend.list(prefix).await
    }

    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.backend.scan(start, end).await
    }

    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        let sealed = ops
            .iter()
            .map(|op| match op {
                WriteOp::Put { key, value } => Ok(WriteOp::Put {
                    key: key.clone(),
                    value: self.seal(key, value)?,
                }),
                WriteOp::Delete { key } => Ok(WriteOp::Delete { key: key.clone() }),
            })
            .collect::<Result<Vec<_>, StoreError>>()?;
        self.backend.apply(&sealed).await
    }
}
