// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Host-owned sealing of mutable Muniment slots under a persona's wallet epoch.
//!
//! Values are sealed before reaching the supplied backend. Keys remain visible
//! for listing and ordered scans. There is no cleartext fallback or migration.
//! Transactions expose authenticated cleartext reads and seal every write before
//! committing. Backends without transactions retain their typed refusal. Authentication detects
//! tampering and transplantation; it does not detect replay of an older valid
//! value at the same key. The host owns epoch history and freshness policy.

use async_trait::async_trait;
use eidetic::{Hash, PayloadSealer, SealedBlobRef};
use muniment::{Backend, StoreError, TransactFn, TransactionReader, WriteOp};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

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
    sealer: Arc<WalletEpochSealer>,
}

impl<B: Clone> Clone for WalletSealedBackend<B> {
    fn clone(&self) -> Self {
        Self {
            backend: self.backend.clone(),
            sealer: Arc::clone(&self.sealer),
        }
    }
}

impl<B> WalletSealedBackend<B> {
    /// Wrap a backend with a required, already-loaded persona wallet sealer.
    pub fn new(backend: B, sealer: WalletEpochSealer) -> Self {
        Self {
            backend,
            sealer: Arc::new(sealer),
        }
    }

    fn seal(&self, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        seal_value(&self.sealer, key, bytes)
    }

    fn unseal(&self, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
        unseal_value(&self.sealer, key, bytes)
    }
}

fn seal_value(sealer: &WalletEpochSealer, key: &str, bytes: &[u8]) -> Result<Vec<u8>, StoreError> {
    let mut salt = [0; 16];
    getrandom::fill(&mut salt).map_err(|_| failure("wallet slot entropy unavailable"))?;
    let cleartext = serde_json::to_vec(&Value {
        key: key.to_owned(),
        salt,
        bytes: bytes.to_vec(),
    })
    .map_err(|_| failure("encode wallet slot value"))?;
    let hash = Hash::of(&cleartext);
    let (ciphertext, seal) = sealer
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

fn unseal_value(
    sealer: &WalletEpochSealer,
    key: &str,
    bytes: &[u8],
) -> Result<Vec<u8>, StoreError> {
    let envelope: Envelope =
        serde_json::from_slice(bytes).map_err(|_| failure("decode wallet slot envelope"))?;
    if envelope.format != FORMAT {
        return Err(failure("unsupported wallet slot format"));
    }
    let cleartext = sealer
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

#[cfg(not(target_arch = "wasm32"))]
type SealFn = Box<dyn FnMut(&str, &[u8]) -> Result<Vec<u8>, StoreError> + Send>;
#[cfg(target_arch = "wasm32")]
type SealFn = Box<dyn FnMut(&str, &[u8]) -> Result<Vec<u8>, StoreError>>;

struct SealedReader<'a> {
    raw: &'a dyn TransactionReader,
    sealer: &'a WalletEpochSealer,
    error: &'a Mutex<Option<StoreError>>,
}

impl SealedReader<'_> {
    fn checked<T>(&self, result: Result<T, StoreError>) -> Result<T, StoreError> {
        if let Err(error) = &result {
            self.error
                .lock()
                .expect("wallet transaction error")
                .get_or_insert_with(|| error.clone());
        }
        result
    }
}

impl TransactionReader for SealedReader<'_> {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.checked(self.raw.get(key).and_then(|value| {
            value
                .map(|bytes| unseal_value(self.sealer, key, &bytes))
                .transpose()
        }))
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.checked(self.raw.list(prefix))
    }
}

fn prepare_transaction(
    sealer: Arc<WalletEpochSealer>,
    caller: TransactFn,
    mut seal: SealFn,
) -> (TransactFn, Arc<Mutex<Option<StoreError>>>) {
    let error = Arc::new(Mutex::new(None));
    let rejected = Arc::clone(&error);
    let transaction = Box::new(move |raw: &dyn TransactionReader| {
        let reader = SealedReader {
            raw,
            sealer: &sealer,
            error: &rejected,
        };
        let ops = caller(&reader);
        // A caller catching a failed read cannot commit against unauthenticated data.
        if rejected.lock().expect("wallet transaction error").is_some() {
            return Vec::new();
        }
        let sealed = ops
            .into_iter()
            .map(|op| match op {
                WriteOp::Put { key, value } => Ok(WriteOp::Put {
                    value: seal(&key, &value)?,
                    key,
                }),
                delete @ WriteOp::Delete { .. } => Ok(delete),
            })
            .collect::<Result<Vec<_>, StoreError>>();
        match sealed {
            Ok(ops) => ops,
            Err(error) => {
                *rejected.lock().expect("wallet transaction error") = Some(error);
                Vec::new()
            },
        }
    });
    (transaction, error)
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

            async fn transact(&self, caller: TransactFn) -> Result<(), StoreError> {
                let sealer = Arc::clone(&self.sealer);
                let writer = Arc::clone(&sealer);
                let (transaction, rejected) = prepare_transaction(
                    sealer, caller, Box::new(move |key, bytes| seal_value(&writer, key, bytes)),
                );
                self.backend.transact(transaction).await?;
                let error = rejected.lock().expect("wallet transaction error").take();
                error.map_or(Ok(()), Err)
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
