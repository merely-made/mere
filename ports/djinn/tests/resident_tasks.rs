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

use graphshell::native::tasks::{ResidentTasks, spawn_tracked};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_and_join_releases_what_scoped_tasks_hold() {
    let held = Arc::new(());
    let tasks = ResidentTasks::new();
    let outer = Arc::clone(&held);
    tasks
        .scope(async move {
            spawn_tracked(async move {
                let inner = Arc::clone(&outer);
                spawn_tracked(async move {
                    let _inner = inner;
                    std::future::pending::<()>().await;
                });
                let _outer = outer;
                std::future::pending::<()>().await;
            });
        })
        .await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(Arc::strong_count(&held), 3, "both tasks hold it");
    assert_eq!(tasks.cancel_and_join().await, 2);
    assert_eq!(Arc::strong_count(&held), 1, "both let go");

    let loose = Arc::clone(&held);
    spawn_tracked(async move {
        let _loose = loose;
        std::future::pending::<()>().await;
    });
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(tasks.cancel_and_join().await, 0);
    assert_eq!(
        Arc::strong_count(&held),
        2,
        "an untracked task keeps its hold"
    );
}
