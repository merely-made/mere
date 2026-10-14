// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(all(target_arch = "wasm32", feature = "indexeddb"))]

use muniment::{Backend, IndexedDbBackend, IndexedDbOpenAttempt, StoreError};
use wasm_bindgen_test::wasm_bindgen_test;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn database_name(control: &str) -> String {
    format!(
        "task7-disposable-{control}-{}-{}",
        js_sys::Date::now(),
        js_sys::Math::random(),
    )
}

#[wasm_bindgen_test]
async fn indexeddb_first_open_and_shared_clone_can_write() {
    let name = database_name("clone");
    let first = IndexedDbBackend::open(&name, "records")
        .await
        .expect("first IndexedDB open must work");
    let clone = first.clone();
    clone.put("task7/sentinel", b"shared-owner").await.unwrap();
    assert_eq!(
        first.get("task7/sentinel").await.unwrap(),
        Some(b"shared-owner".to_vec()),
    );
    drop(first);
    assert!(matches!(
        IndexedDbBackend::open(&name, "records").await,
        Err(StoreError::OwnershipBusy)
    ));
    clone.close().await.unwrap();
    assert!(matches!(
        clone.get("task7/sentinel").await,
        Err(StoreError::OwnershipLost(_))
    ));
    let reopened = IndexedDbBackend::open(&name, "records").await.unwrap();
    assert_eq!(
        reopened.get("task7/sentinel").await.unwrap(),
        Some(b"shared-owner".to_vec())
    );
    reopened.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_independent_second_open_is_refused() {
    let name = database_name("independent");
    let first = IndexedDbBackend::open(&name, "records")
        .await
        .expect("first IndexedDB open must work");
    first.put("task7/sentinel", b"first-owner").await.unwrap();
    let second = IndexedDbBackend::open(&name, "records").await;
    if let Ok(competing) = &second {
        assert_eq!(
            competing.get("task7/sentinel").await.unwrap(),
            Some(b"first-owner".to_vec()),
            "the independent handle must see the same durable store",
        );
    }
    // Retain and exercise the first handle after the second open attempt.
    assert_eq!(
        first.get("task7/sentinel").await.unwrap(),
        Some(b"first-owner".to_vec()),
    );
    assert!(
        matches!(second, Err(StoreError::OwnershipBusy)),
        "independent open succeeded while the first writer remained live",
    );
}

#[wasm_bindgen::prelude::wasm_bindgen(module = "/tests/indexeddb_controls.js")]
extern "C" {
    #[wasm_bindgen(js_name = cancelAtUpgrade)]
    fn cancel_at_upgrade(callback: &js_sys::Function);
    #[wasm_bindgen(js_name = watchTransactions)]
    fn watch_transactions() -> wasm_bindgen::JsValue;
    #[wasm_bindgen(js_name = finishWatching)]
    fn finish_watching(observation: &wasm_bindgen::JsValue) -> String;
    #[wasm_bindgen(js_name = uniqueLegacy, catch)]
    async fn unique_legacy(name: &str) -> Result<(), wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = changeVersion, catch)]
    async fn change_version(name: &str) -> Result<(), wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = competingFrame, catch)]
    async fn competing_frame(name: &str) -> Result<bool, wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = legacy, catch)]
    async fn legacy(name: &str) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = closeLegacy)]
    fn close_legacy(value: &wasm_bindgen::JsValue);
    #[wasm_bindgen(js_name = task, catch)]
    async fn task() -> Result<(), wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = inspect, catch)]
    async fn inspect(name: &str, version: u32) -> Result<String, wasm_bindgen::JsValue>;
    #[wasm_bindgen(js_name = oldVersionError, catch)]
    async fn old_version_error(name: &str) -> Result<String, wasm_bindgen::JsValue>;
}

#[wasm_bindgen_test]
async fn indexeddb_legacy_blocker_drains_without_upgrade_then_cutover_preserves_data() {
    let name = database_name("legacy-cutover");
    let old = legacy(&name).await.unwrap();
    let before = inspect(&name, 1).await.unwrap();
    let attempt = IndexedDbOpenAttempt::start(&name, "records").unwrap();
    for _ in 0..200 {
        if attempt.status() == "blocked-draining" {
            break;
        }
        task().await.unwrap();
    }
    assert_eq!(attempt.status(), "blocked-draining");
    assert!(matches!(
        IndexedDbBackend::open(&name, "records").await,
        Err(StoreError::OwnershipBusy)
    ));
    close_legacy(&old);
    assert!(matches!(
        attempt.finish().await,
        Err(StoreError::UpgradeBlocked)
    ));
    assert_eq!(
        inspect(&name, 1).await.unwrap(),
        before,
        "refused attempt must roll back its eventual upgrade"
    );
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    assert_eq!(owner.get("sentinel").await.unwrap(), Some(vec![3, 1, 4]));
    assert_eq!(
        inspect(&name, 2).await.unwrap(),
        before,
        "all stores, indexes and bytes survive version-only cutover"
    );
    assert_eq!(old_version_error(&name).await.unwrap(), "VersionError");
    owner.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_dropped_open_future_still_aborts_and_drains_queued_upgrade() {
    let name = database_name("dropped-open");
    let old = legacy(&name).await.unwrap();
    let before = inspect(&name, 1).await.unwrap();
    let attempt = IndexedDbOpenAttempt::start(&name, "records").unwrap();
    for _ in 0..200 {
        if attempt.status() == "blocked-draining" {
            break;
        }
        task().await.unwrap();
    }
    assert_eq!(attempt.status(), "blocked-draining");
    let waiter = attempt.finish();
    drop(waiter);
    assert!(matches!(
        IndexedDbBackend::open(&name, "records").await,
        Err(StoreError::OwnershipBusy)
    ));
    close_legacy(&old);
    // Queued behind the refused upgrade; v1 success proves no late cutover.
    assert_eq!(inspect(&name, 1).await.unwrap(), before);
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    owner.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_missing_existing_store_refuses_without_schema_change() {
    let name = database_name("missing-store");
    let old = legacy(&name).await.unwrap();
    let before = inspect(&name, 1).await.unwrap();
    close_legacy(&old);
    assert!(matches!(
        IndexedDbBackend::open(&name, "absent").await,
        Err(StoreError::InvalidSchema(_))
    ));
    assert_eq!(inspect(&name, 1).await.unwrap(), before);
}

#[wasm_bindgen_test]
async fn indexeddb_cancel_before_acquisition_has_no_committed_upgrade() {
    let name = database_name("early-cancel");
    let attempt = IndexedDbOpenAttempt::start(&name, "records").unwrap();
    attempt.cancel();
    assert!(matches!(
        attempt.finish().await,
        Err(StoreError::OpenCanceled {
            schema_upgraded: false
        })
    ));
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    owner.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_cancel_after_committed_upgrade_reports_truth_and_drains() {
    let name = database_name("committed-cancel");
    let attempt = IndexedDbOpenAttempt::start(&name, "records").unwrap();
    for _ in 0..200 {
        if attempt.status() == "opened" {
            break;
        }
        task().await.unwrap();
    }
    assert_eq!(attempt.status(), "opened");
    attempt.cancel();
    assert!(matches!(
        attempt.finish().await,
        Err(StoreError::OpenCanceled {
            schema_upgraded: true
        })
    ));
    assert_eq!(old_version_error(&name).await.unwrap(), "VersionError");
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    owner.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_dropped_transaction_waiter_drains_and_keeps_exact_committed_bytes() {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    let name = database_name("detached-transaction");
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    owner.put("counter", &[1]).await.unwrap();
    let mut write = owner.transact(Box::new(|reader| {
        let value = reader.get("counter").unwrap().unwrap()[0];
        vec![muniment::WriteOp::Put {
            key: "counter".into(),
            value: vec![value + 1],
        }]
    }));
    assert!(matches!(
        write.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    drop(write);
    // Close invalidates submission immediately but awaits the detached native
    // transaction. Reopening reads its bytes; it never replays the command.
    owner.close().await.unwrap();
    let reopened = IndexedDbBackend::open(&name, "records").await.unwrap();
    assert_eq!(reopened.get("counter").await.unwrap(), Some(vec![2]));
    reopened.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_dropped_read_and_failed_write_drain_before_reopen() {
    use std::future::Future;
    use std::task::{Context, Poll, Waker};
    let name = database_name("failed-transaction");
    unique_legacy(&name).await.unwrap();
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    let mut read = owner.get("sentinel");
    assert!(matches!(
        read.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    drop(read);
    let observation = watch_transactions();
    let mut write = owner.put("duplicate", &[7]);
    assert!(matches!(
        write.as_mut().poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    let mut closing = Box::pin(owner.close());
    assert!(matches!(
        closing
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop())),
        Poll::Pending
    ));
    // Request ConstraintError bubbles before abort. Both futures retain their
    // native callbacks until abort; a request error is never an early success.
    let written = write.await;
    closing.await.unwrap();
    let events = finish_watching(&observation);
    assert!(written.is_err());
    assert_eq!(events, "error,abort");
    let reopened = IndexedDbBackend::open(&name, "records").await.unwrap();
    assert_eq!(reopened.get("sentinel").await.unwrap(), Some(vec![7]));
    assert_eq!(reopened.get("duplicate").await.unwrap(), None);
    reopened.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_independent_realm_refuses_and_versionchange_invalidates_owner() {
    let name = database_name("realm-owner-loss");
    let owner = IndexedDbBackend::open(&name, "records").await.unwrap();
    assert!(competing_frame(&name).await.unwrap());
    change_version(&name).await.unwrap();
    assert!(!owner.is_open());
    assert!(matches!(
        owner.put("late", &[1]).await,
        Err(StoreError::OwnershipLost(_))
    ));
    owner.close().await.unwrap();
}

#[wasm_bindgen_test]
async fn indexeddb_cancel_during_upgrade_rolls_back_before_admission() {
    use std::{cell::RefCell, rc::Rc};
    use wasm_bindgen::{JsCast, closure::Closure};
    let name = database_name("upgrade-cancel");
    let old = legacy(&name).await.unwrap();
    close_legacy(&old);
    let before = inspect(&name, 1).await.unwrap();
    let attempt = Rc::new(RefCell::new(None::<IndexedDbOpenAttempt>));
    let callback_attempt = attempt.clone();
    let callback = Closure::<dyn FnMut()>::new(move || {
        callback_attempt.borrow().as_ref().unwrap().cancel();
    });
    cancel_at_upgrade(callback.as_ref().unchecked_ref());
    *attempt.borrow_mut() = Some(IndexedDbOpenAttempt::start(&name, "records").unwrap());
    for _ in 0..200 {
        if attempt.borrow().as_ref().unwrap().status() == "refused" {
            break;
        }
        task().await.unwrap();
    }
    let attempt = attempt.borrow_mut().take().unwrap();
    assert!(matches!(
        attempt.finish().await,
        Err(StoreError::OpenCanceled {
            schema_upgraded: false
        })
    ));
    drop(callback);
    assert_eq!(inspect(&name, 1).await.unwrap(), before);
}
