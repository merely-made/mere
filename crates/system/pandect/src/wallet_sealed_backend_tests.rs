// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use muniment::MemoryBackend;

use super::*;
use crate::{PersonaId, wallet_store::KeyEpochId};

fn sealer(persona: PersonaId, epoch: u8, secret: &[u8]) -> WalletEpochSealer {
    WalletEpochSealer::from_epoch(
        persona,
        KeyEpochId(uuid::Uuid::from_bytes([epoch; 16])),
        secret,
    )
}

#[test]
fn batches_seal_values_and_keep_the_backend_key_contract() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let sealed = WalletSealedBackend::new(raw.clone(), sealer(PersonaId::new(), 1, b"secret"));
        assert_eq!(sealed.get("missing").await.unwrap(), None);
        sealed.put("a", b"private petname").await.unwrap();
        let first = raw.get("a").await.unwrap().unwrap();
        sealed.put("a", b"private petname").await.unwrap();
        let second = raw.get("a").await.unwrap().unwrap();
        assert_ne!(first, second);
        let a: Envelope = serde_json::from_slice(&first).unwrap();
        let b: Envelope = serde_json::from_slice(&second).unwrap();
        assert_ne!(
            a.hash, b.hash,
            "the content digest is salted inside the seal"
        );
        sealed
            .apply(&[
                WriteOp::Put {
                    key: "c".into(),
                    value: b"private petname".to_vec(),
                },
                WriteOp::Put {
                    key: "b".into(),
                    value: b"other".to_vec(),
                },
                WriteOp::Delete { key: "a".into() },
            ])
            .await
            .unwrap();
        assert_eq!(sealed.get("a").await.unwrap(), None);
        assert_eq!(sealed.get("c").await.unwrap().unwrap(), b"private petname");
        assert_eq!(sealed.scan("b", "d").await.unwrap(), ["b", "c"]);
        let mut keys = sealed.list("").await.unwrap();
        keys.sort();
        assert_eq!(keys, ["b", "c"]);
        for key in keys {
            let bytes = raw.get(&key).await.unwrap().unwrap();
            assert!(
                !bytes
                    .windows(b"private petname".len())
                    .any(|w| w == b"private petname")
            );
        }
        sealed.delete("b").await.unwrap();
        sealed.delete("b").await.unwrap();
        assert_eq!(raw.list("").await.unwrap(), ["c"]);
        sealed
            .transact(Box::new(|reader| {
                assert_eq!(reader.get("c").unwrap().unwrap(), b"private petname");
                assert_eq!(reader.get("absent").unwrap(), None);
                assert_eq!(reader.list("").unwrap(), ["c"]);
                vec![
                    WriteOp::Put {
                        key: "d".into(),
                        value: b"transaction private".to_vec(),
                    },
                    WriteOp::Delete { key: "c".into() },
                ]
            }))
            .await
            .unwrap();
        assert_eq!(sealed.get("c").await.unwrap(), None);
        assert_eq!(
            sealed.get("d").await.unwrap().unwrap(),
            b"transaction private"
        );
        let ciphertext = raw.get("d").await.unwrap().unwrap();
        assert!(serde_json::from_slice::<Envelope>(&ciphertext).is_ok());
        assert!(
            !ciphertext
                .windows(b"transaction private".len())
                .any(|part| part == b"transaction private")
        );
    });
}

#[test]
fn transactions_refuse_caught_authentication_errors_without_any_writes() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let persona = PersonaId::new();
        let sealed = WalletSealedBackend::new(raw.clone(), sealer(persona, 1, b"secret"));
        sealed.put("a", b"protected").await.unwrap();
        sealed.put("keep", b"retained").await.unwrap();
        let original = raw.get("a").await.unwrap().unwrap();
        let keep = raw.get("keep").await.unwrap().unwrap();
        let mut invalid = vec![b"{}".to_vec(), original.clone()];
        let malformed_value = b"[]";
        let hash = Hash::of(malformed_value);
        let (ciphertext, seal) = sealed.sealer.seal(&hash, malformed_value).unwrap();
        invalid.push(
            serde_json::to_vec(&Envelope {
                format: FORMAT.into(),
                hash,
                seal,
                ciphertext,
            })
            .unwrap(),
        );
        for field in ["ciphertext", "hash", "epoch", "format"] {
            let mut envelope: Envelope = serde_json::from_slice(&original).unwrap();
            match field {
                "ciphertext" => *envelope.ciphertext.last_mut().unwrap() ^= 1,
                "hash" => envelope.hash = Hash::of(b"wrong"),
                "epoch" => envelope.seal.epoch.0[0] ^= 1,
                "format" => envelope.format.push('!'),
                _ => unreachable!(),
            }
            invalid.push(serde_json::to_vec(&envelope).unwrap());
        }
        // Even a valid envelope transplanted from a different key is refused.
        for bytes in invalid {
            raw.put("bad", &bytes).await.unwrap();
            assert!(
                sealed
                    .transact(Box::new(|reader| {
                        assert!(reader.get("bad").is_err());
                        vec![
                            WriteOp::Delete { key: "keep".into() },
                            WriteOp::Put {
                                key: "new".into(),
                                value: b"must not land".to_vec(),
                            },
                        ]
                    }))
                    .await
                    .is_err()
            );
            assert_eq!(raw.get("bad").await.unwrap().unwrap(), bytes);
            assert_eq!(raw.get("keep").await.unwrap().unwrap(), keep);
            assert_eq!(raw.get("new").await.unwrap(), None);
        }
        for wrong in [
            sealer(PersonaId::new(), 1, b"secret"),
            sealer(persona, 1, b"wrong"),
        ] {
            let wrong = WalletSealedBackend::new(raw.clone(), wrong);
            assert!(
                wrong
                    .transact(Box::new(|reader| {
                        assert!(reader.get("a").is_err());
                        vec![WriteOp::Delete { key: "keep".into() }]
                    }))
                    .await
                    .is_err()
            );
            assert_eq!(raw.get("keep").await.unwrap().unwrap(), keep);
        }
        sealed.put("bad", b"repaired").await.unwrap();
        sealed
            .transact(Box::new(|reader| {
                assert_eq!(reader.get("bad").unwrap().unwrap(), b"repaired");
                vec![WriteOp::Put {
                    key: "new".into(),
                    value: b"valid".to_vec(),
                }]
            }))
            .await
            .unwrap();
        assert_eq!(sealed.get("new").await.unwrap().unwrap(), b"valid");
        assert_eq!(raw.get("a").await.unwrap().unwrap(), original);
    });
}

#[test]
fn transaction_seal_failure_discards_the_whole_batch_with_valid_control() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let sealed = WalletSealedBackend::new(raw.clone(), sealer(PersonaId::new(), 1, b"secret"));
        sealed.put("keep", b"retained").await.unwrap();
        let original = raw.get("keep").await.unwrap().unwrap();
        for fail_second in [true, false] {
            let writer = Arc::clone(&sealed.sealer);
            let (transaction, rejected) = prepare_transaction(
                Arc::clone(&sealed.sealer),
                Box::new(|reader| {
                    assert_eq!(reader.get("keep").unwrap().unwrap(), b"retained");
                    vec![
                        WriteOp::Put {
                            key: "first".into(),
                            value: b"one".to_vec(),
                        },
                        WriteOp::Delete { key: "keep".into() },
                        WriteOp::Put {
                            key: "second".into(),
                            value: b"two".to_vec(),
                        },
                    ]
                }),
                Box::new(move |key, bytes| {
                    if fail_second && key == "second" {
                        return Err(failure("injected sealing failure"));
                    }
                    seal_value(&writer, key, bytes)
                }),
            );
            raw.transact(transaction).await.unwrap();
            if fail_second {
                assert_eq!(
                    *rejected.lock().unwrap(),
                    Some(failure("injected sealing failure"))
                );
                assert_eq!(raw.get("keep").await.unwrap().unwrap(), original);
                assert_eq!(raw.get("first").await.unwrap(), None);
                assert_eq!(raw.get("second").await.unwrap(), None);
            } else {
                assert!(rejected.lock().unwrap().is_none());
                assert_eq!(sealed.get("keep").await.unwrap(), None);
                assert_eq!(sealed.get("first").await.unwrap().unwrap(), b"one");
                assert_eq!(sealed.get("second").await.unwrap().unwrap(), b"two");
            }
        }
    });
}

struct BatchOnly(MemoryBackend);

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl Backend for BatchOnly {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
        self.0.get(key).await
    }
    async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
        self.0.put(key, bytes).await
    }
    async fn delete(&self, key: &str) -> Result<(), StoreError> {
        self.0.delete(key).await
    }
    async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
        self.0.list(prefix).await
    }
    async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
        self.0.scan(start, end).await
    }
    async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
        self.0.apply(ops).await
    }
}

#[test]
fn unsupported_transactions_do_not_run_the_caller_or_write() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let sealed = WalletSealedBackend::new(
            BatchOnly(raw.clone()),
            sealer(PersonaId::new(), 1, b"secret"),
        );
        sealed.put("keep", b"retained").await.unwrap();
        let original = raw.get("keep").await.unwrap().unwrap();
        assert_eq!(
            sealed
                .transact(Box::new(|_| panic!(
                    "unsupported backend must not run closure"
                )))
                .await,
            Err(StoreError::NotTransactional)
        );
        assert_eq!(raw.get("keep").await.unwrap().unwrap(), original);
        assert_eq!(raw.list("").await.unwrap(), ["keep"]);
    });
}

#[test]
fn transaction_reader_io_errors_poison_even_when_caught() {
    struct BrokenReader;
    impl TransactionReader for BrokenReader {
        fn get(&self, _: &str) -> Result<Option<Vec<u8>>, StoreError> {
            Err(failure("read failed"))
        }
        fn list(&self, _: &str) -> Result<Vec<String>, StoreError> {
            Err(failure("list failed"))
        }
    }
    for list in [true, false] {
        let (transaction, rejected) = prepare_transaction(
            Arc::new(sealer(PersonaId::new(), 1, b"secret")),
            Box::new(move |reader| {
                if list {
                    assert!(reader.list("").is_err());
                } else {
                    assert!(reader.get("a").is_err());
                }
                vec![WriteOp::Delete { key: "keep".into() }]
            }),
            Box::new(|_, _| panic!("failed reads must skip sealing")),
        );
        assert!(transaction(&BrokenReader).is_empty());
        assert_eq!(
            *rejected.lock().unwrap(),
            Some(failure(if list { "list failed" } else { "read failed" }))
        );
    }
}

#[test]
fn epoch_history_is_explicit_and_different_personas_or_keys_are_refused() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let persona = PersonaId::new();
        WalletSealedBackend::new(raw.clone(), sealer(persona, 1, b"old"))
            .put("slot", b"old record")
            .await
            .unwrap();
        assert!(
            WalletSealedBackend::new(raw.clone(), sealer(persona, 2, b"new"))
                .get("slot")
                .await
                .is_err()
        );
        let historical = sealer(persona, 2, b"new")
            .with_epoch(KeyEpochId(uuid::Uuid::from_bytes([1; 16])), b"old");
        let rotated = WalletSealedBackend::new(raw.clone(), historical);
        assert_eq!(rotated.get("slot").await.unwrap().unwrap(), b"old record");
        assert!(
            WalletSealedBackend::new(raw.clone(), sealer(PersonaId::new(), 1, b"old"))
                .get("slot")
                .await
                .is_err()
        );
        assert!(
            WalletSealedBackend::new(raw.clone(), sealer(persona, 1, b"wrong secret"))
                .get("slot")
                .await
                .is_err()
        );
        rotated.put("slot", b"new record").await.unwrap();
        assert!(
            WalletSealedBackend::new(raw.clone(), sealer(persona, 1, b"old"))
                .get("slot")
                .await
                .is_err()
        );
        assert_eq!(
            WalletSealedBackend::new(raw, sealer(persona, 2, b"new"))
                .get("slot")
                .await
                .unwrap()
                .unwrap(),
            b"new record"
        );
    });
}

#[test]
fn malformed_tampered_and_transplanted_envelopes_stay_refused_and_untouched() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let sealed = WalletSealedBackend::new(raw.clone(), sealer(PersonaId::new(), 1, b"secret"));
        sealed.put("a", b"private").await.unwrap();
        let original = raw.get("a").await.unwrap().unwrap();
        raw.put("b", &original).await.unwrap();
        assert_eq!(
            sealed.get("b").await,
            Err(failure("wallet slot key mismatch"))
        );
        for bytes in [b"private".to_vec(), vec![], b"{}".to_vec()] {
            raw.put("b", &bytes).await.unwrap();
            assert!(sealed.get("b").await.is_err());
            assert_eq!(raw.get("b").await.unwrap().unwrap(), bytes);
        }
        for field in ["ciphertext", "hash", "epoch", "format"] {
            let mut envelope: Envelope = serde_json::from_slice(&original).unwrap();
            match field {
                "ciphertext" => *envelope.ciphertext.last_mut().unwrap() ^= 1,
                "hash" => envelope.hash = Hash::of(b"wrong hash"),
                "epoch" => envelope.seal.epoch.0[0] ^= 1,
                "format" => envelope.format.push('!'),
                _ => unreachable!(),
            }
            let bytes = serde_json::to_vec(&envelope).unwrap();
            raw.put("a", &bytes).await.unwrap();
            assert!(sealed.get("a").await.is_err(), "changed {field} accepted");
            assert_eq!(raw.get("a").await.unwrap().unwrap(), bytes);
        }
        raw.put("a", &original).await.unwrap();
        assert_eq!(sealed.get("a").await.unwrap().unwrap(), b"private");
    });
}
