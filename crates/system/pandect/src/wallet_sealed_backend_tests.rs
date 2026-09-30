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
        assert_eq!(
            sealed
                .transact(Box::new(|_| panic!("refusal must not run the closure")))
                .await,
            Err(StoreError::NotTransactional)
        );
    });
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
