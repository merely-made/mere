// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The task scope djinn's graceful stop relies on (harness plan F1): a
//! scoped task, and the task it spawns, are aborted and dropped before
//! `cancel_and_join` returns, so what they held is released by then. A task
//! spawned outside the scope is the control: it keeps its hold.
//!
//! Here rather than in graphshell's own tests, which need a sibling checkout
//! a worktree does not have.

use std::sync::Arc;
use std::time::Duration;

use armillary::tasks::{ResidentTasks, spawn_tracked};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_and_join_releases_what_scoped_tasks_hold() {
    let held = Arc::new(());
    let tasks = ResidentTasks::new();
    let outer = Arc::clone(&held);
    let (ready, started) = tokio::sync::oneshot::channel();
    tasks
        .scope(async move {
            spawn_tracked(async move {
                let inner = Arc::clone(&outer);
                spawn_tracked(async move {
                    let _inner = inner;
                    ready.send(()).unwrap();
                    std::future::pending::<()>().await;
                });
                let _outer = outer;
                std::future::pending::<()>().await;
            });
        })
        .await;
    tokio::time::timeout(Duration::from_secs(5), started)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(Arc::strong_count(&held), 3, "both tasks hold it");
    assert_eq!(tasks.cancel_and_join().await.joined(), 2);
    assert_eq!(Arc::strong_count(&held), 1, "both let go");

    let loose = Arc::clone(&held);
    let (ready, started) = tokio::sync::oneshot::channel();
    spawn_tracked(async move {
        let _loose = loose;
        ready.send(()).unwrap();
        std::future::pending::<()>().await;
    });
    tokio::time::timeout(Duration::from_secs(5), started)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(tasks.cancel_and_join().await.joined(), 0);
    assert_eq!(
        Arc::strong_count(&held),
        2,
        "an untracked task keeps its hold"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resident_shutdown_reports_a_panicked_tracked_task() {
    let tasks = ResidentTasks::new();
    let (send, receive) = tokio::sync::oneshot::channel();
    tasks
        .scope(async {
            spawn_tracked(async move {
                send.send(()).unwrap();
                panic!("synthetic resident task failure");
            });
        })
        .await;
    receive.await.unwrap();
    tokio::task::yield_now().await;
    let report = tasks.close_and_join().await;
    assert_eq!(report.panicked, 1);
    assert!(!report.is_clean());
    assert_eq!(tasks.close_and_join().await, report);
}

#[tokio::test]
async fn open_drain_is_reusable_but_terminal_close_refuses_work() {
    use armillary::tasks::spawn_tracked_with_handle;

    let tasks = ResidentTasks::new();
    for answer in [1, 2] {
        let receive = tasks
            .scope(async { spawn_tracked_with_handle(async move { answer }) })
            .await;
        assert_eq!(receive.await.unwrap(), answer);
        let report = tasks.cancel_and_join().await;
        assert_eq!(report.completed, 1);
        assert!(report.is_clean());
        assert!(!tasks.is_closed());
    }
    assert!(tasks.close_and_join().await.is_clean());
    let held = Arc::new(());
    let future_hold = held.clone();
    let receive = tasks
        .scope(async {
            spawn_tracked_with_handle(async move {
                let _held = future_hold;
                3
            })
        })
        .await;
    assert!(
        receive.await.is_err(),
        "rejected work cannot return a value"
    );
    assert_eq!(Arc::strong_count(&held), 1);
    let report = tasks.close_and_join().await;
    assert_eq!(report.rejected, 1);
    assert!(!report.is_clean());
    assert_eq!(tasks.close_and_join().await, report);
}

#[tokio::test]
async fn a_reaped_panic_remains_in_the_resident_shutdown_report() {
    let tasks = ResidentTasks::new();
    let (dropped, observed) = tokio::sync::oneshot::channel::<()>();
    tasks.spawn(async move {
        let _held_until_unwind = dropped;
        panic!("controlled resident task failure");
    });
    assert!(observed.await.is_err());
    // Admission reaps completed tasks; doing so must retain their failures.
    tasks.spawn(async {});
    let report = tasks.close_and_join().await;
    assert_eq!(report.panicked, 1);
    assert!(!report.is_clean());
    assert_eq!(tasks.close_and_join().await, report);
}
