// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The reservoir-owned retained page-text store (Eidetic S6): its lifetime,
//! its off-by-default setting, and the custody gate that pauses writing
//! rather than falling back to plaintext.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use catchment::{
    DomainId, KeepPageText, Keeping, MereId, PlainCustody, RetainedCustody, RetainedText,
    open_mere_retained_backend, open_reservoir_backend, retained_path,
};
use eidetic::{PageTextStore, ResourceCaptureStore};
use identity::PersonaId;
use muniment::{Backend, MemoryBackend};
use uuid::Uuid;

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x5336))
}

fn mere() -> MereId {
    MereId::derive(persona(), &DomainId::new("browsing").unwrap())
}

#[test]
fn the_reservoir_owner_opens_the_store_once_and_it_survives_a_cold_reopen() {
    pollster::block_on(async {
        let root = tempfile::tempdir().unwrap();
        // The owner holds the reservoir lock, then opens the mere's store.
        let _reservoir = open_reservoir_backend(root.path(), persona()).unwrap();
        let backend = open_mere_retained_backend(root.path(), persona(), mere()).unwrap();
        assert!(retained_path(root.path(), persona(), mere()).exists());
        assert!(
            open_mere_retained_backend(root.path(), persona(), mere()).is_err(),
            "a second opener is refused while the owner holds the store"
        );
        let text = PageTextStore::new(backend.clone());
        text.put("https://example.test/a", "kept body about lighthouses", 10)
            .await
            .unwrap();
        // The two writers share one format: the resource version is there too.
        assert_eq!(
            ResourceCaptureStore::new(&backend)
                .for_url("https://example.test/a")
                .await
                .unwrap()
                .len(),
            1
        );
        drop(text);
        drop(backend);
        let reopened = open_mere_retained_backend(root.path(), persona(), mere()).unwrap();
        let text = PageTextStore::new(reopened);
        assert_eq!(
            text.text_for("https://example.test/a")
                .await
                .unwrap()
                .as_deref(),
            Some("kept body about lighthouses")
        );
    });
}

#[test]
fn keeping_is_off_until_chosen_and_the_choice_is_stored_with_the_store() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let store = RetainedText::new(backend.clone(), Arc::new(PlainCustody));
        assert_eq!(store.setting().await.unwrap(), KeepPageText::default());
        assert_eq!(store.keeping().await.unwrap(), Keeping::Off);
        assert_eq!(store.permit_write().await.unwrap_err(), Keeping::Off);
        let chosen = KeepPageText {
            enabled: true,
            max_bytes: 4096,
            excluded_origins: ["https://bank.test".to_string()].into(),
        };
        store.set_setting(chosen.clone()).await.unwrap();
        // A second handle on the same backend (a reopen) sees the choice.
        let again = RetainedText::new(backend, Arc::new(PlainCustody));
        assert_eq!(again.setting().await.unwrap(), chosen);
        assert_eq!(again.keeping().await.unwrap(), Keeping::On);
        assert_eq!(again.permit_write().await.unwrap(), chosen);
    });
}

struct Switch(AtomicBool);

impl RetainedCustody for Switch {
    fn writable(&self) -> Result<(), String> {
        if self.0.load(Ordering::SeqCst) {
            Ok(())
        } else {
            Err("The vault is locked; page text is not being kept.".into())
        }
    }
}

#[test]
fn locked_custody_pauses_writing_and_nothing_reaches_the_backend() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let open = Arc::new(Switch(AtomicBool::new(true)));
        let store = RetainedText::new(raw.clone(), open.clone());
        store
            .set_setting(KeepPageText {
                enabled: true,
                ..KeepPageText::default()
            })
            .await
            .unwrap();
        assert_eq!(store.keeping().await.unwrap(), Keeping::On);
        let held = raw.list("").await.unwrap();
        open.0.store(false, Ordering::SeqCst);
        assert_eq!(
            store.keeping().await.unwrap(),
            Keeping::Paused("The vault is locked; page text is not being kept.".into())
        );
        assert!(matches!(
            store.permit_write().await,
            Err(Keeping::Paused(_))
        ));
        assert!(
            store
                .set_setting(KeepPageText::default())
                .await
                .unwrap_err()
                .contains("locked"),
            "even changing the setting is a write the gate governs"
        );
        assert_eq!(raw.list("").await.unwrap(), held, "a pause writes nothing");
        open.0.store(true, Ordering::SeqCst);
        assert!(store.permit_write().await.is_ok(), "unlocking resumes");
    });
}

#[test]
fn a_sealed_backend_keeps_the_same_records_unreadable_without_custody() {
    pollster::block_on(async {
        let raw = MemoryBackend::new();
        let epoch = sporran::wallet_store::KeyEpochId(Uuid::from_bytes([5; 16]));
        let sealed = sporran::WalletSealedBackend::new(
            raw.clone(),
            sporran::WalletEpochSealer::from_epoch(persona(), epoch, b"retained secret"),
        );
        let store = RetainedText::new(sealed.clone(), Arc::new(PlainCustody));
        store
            .set_setting(KeepPageText {
                enabled: true,
                ..KeepPageText::default()
            })
            .await
            .unwrap();
        let text = PageTextStore::new(sealed.clone());
        let secret = "a sentence only the person should read";
        text.put("https://example.test/s", secret, 1).await.unwrap();
        // Same format through the seal: the records read back whole.
        assert_eq!(
            text.text_for("https://example.test/s")
                .await
                .unwrap()
                .as_deref(),
            Some(secret)
        );
        // At rest nothing under any key is the plaintext.
        for key in raw.list("").await.unwrap() {
            let value = raw.get(&key).await.unwrap().unwrap();
            assert!(
                !String::from_utf8_lossy(&value).contains("only the person"),
                "{key} holds plaintext"
            );
        }
        // Reading the raw backend as a plain store finds no page text.
        let plain = PageTextStore::new(raw);
        assert!(
            plain.text_for("https://example.test/s").await.is_err()
                || plain
                    .text_for("https://example.test/s")
                    .await
                    .unwrap()
                    .is_none()
        );
    });
}
