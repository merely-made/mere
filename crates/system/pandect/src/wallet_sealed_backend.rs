// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-owned sealing of mutable Muniment slots under a persona's wallet epoch.
//!
//! Values are sealed before reaching the supplied backend. Keys remain visible
//! for listing and ordered scans. There is no cleartext fallback or migration.
//! This adapter supports atomic write batches, but refuses read/write transactions
//! through Muniment's default `NotTransactional` result. It is for slots, not
//! custody operations that require transactional reads. Authentication detects
//! tampering and transplantation; it does not detect replay of an older valid
//! value at the same key. The host owns epoch history and freshness policy.

use async_trait::async_trait;
use eidetic::{Hash, PayloadSealer, SealedBlobRef};
use muniment::{Backend, StoreError, WriteOp};
use serde::{Deserialize, Serialize};

use crate::WalletEpochSealer;

const FORMAT: &str = "pandect-wallet-slot-v1";

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
struct Value {
    key: String,
    // Inside the ciphertext: the exposed content digest cannot be used to
    // guess a small record or correlate two otherwise identical writes.
    salt: [u8; 16],
    bytes: Vec<u8>,
}

/// A caller-selected byte backend with wallet-authenticated, sealed values.
///
/// Construct with an available persona sealer, for example through Castellan's
/// `PersonaeHost::sealed_backend`. Retain the original backend separately only
/// when the host needs raw storage administration. Mixing raw writes with this
/// adapter makes reads fail; unsealed bytes are never accepted as an empty slot.
pub struct WalletSealedBackend<B> {
    backend: B,
    sealer: WalletEpochSealer,
}

impl<B> WalletSealedBackend<B> {
    /// Wrap a backend with a required, already-loaded persona wallet sealer.
    pub fn new(backend: B, sealer: WalletEpochSealer) -> Self {
        Self { backend, sealer }
    }

    fn seal(&self, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        let mut salt = [0; 16];
        getrandom::fill(&mut salt).map_err(|_| failure("wallet slot entropy unavailable"))?;
        let cleartext = serde_json::to_vec(&Value {
            key: key.to_owned(),
            salt,
            bytes: bytes.to_vec(),
        })
        .map_err(|_| failure("encode wallet slot value"))?;
        let hash = Hash::of(&cleartext);
        let (ciphertext, seal) = self
            .sealer
            .seal(&hash, &cleartext)
            .map_err(|_| failure("seal wallet slot value"))?;
        serde_json::to_vec(&Envelope {
            format: FORMAT.to_owned(),
            hash,
            seal,
            ciphertext,
        })
        .map_err(|_| failure("encode wallet slot envelope"))
    }

    fn unseal(&self, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        let envelope: Envelope =
            serde_json::from_slice(bytes).map_err(|_| failure("decode wallet slot envelope"))?;
        if envelope.format != FORMAT {
            return Err(failure("unsupported wallet slot format"));
        }
        let cleartext = self
            .sealer
            .unseal(&envelope.hash, &envelope.seal, &envelope.ciphertext)
            .map_err(|_| failure("authenticate wallet slot value"))?;
        if Hash::of(&cleartext) != envelope.hash {
            return Err(failure("wallet slot content hash mismatch"));
        }
        let value: Value =
            serde_json::from_slice(&cleartext).map_err(|_| failure("decode wallet slot value"))?;
        if value.key != key {
            return Err(failure("wallet slot key mismatch"));
        }
        Ok(value.bytes)
    }
}

fn failure(message: &str) -> StoreError {
    StoreError::Backend(message.to_owned())
}

// Native futures require Sync; browser backends may hold JS objects that are
// not Sync. Keep one implementation while preserving Muniment's platform seam.
macro_rules! implement_backend {
    ($($bounds:tt)+) => {
        #[cfg_attr(not(target_arch = "wasm32"), async_trait)]
        #[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
        impl<B: $($bounds)+> Backend for WalletSealedBackend<B> {
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
                // Seal every value before issuing any writes. The selected backend
                // retains responsibility for the atomic batch contract.
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
    };
}

#[cfg(not(target_arch = "wasm32"))]
implement_backend!(Backend + Sync);
#[cfg(target_arch = "wasm32")]
implement_backend!(Backend);

#[cfg(test)]
#[path = "wallet_sealed_backend_tests.rs"]
mod tests;
