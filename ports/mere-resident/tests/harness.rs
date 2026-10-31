// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The djinn test harness against the real binary (harness plan H1 to H3):
//! the guard refuses before spawning (C1, C4), one resident is ready by its
//! pipes and status route, killed, stopped gracefully and restarted on the
//! same roots, a killed test process leaves no resident (C3), and a canary
//! catches a deliberately unguarded root (C4).
//!
//! The live tests are ignored by default; they run real residents:
//!
//! ```text
//! DJINN_RECEIPTS_ROOT=C:\t\receipts cargo test -p djinn --test harness -- --ignored --nocapture
//! ```

mod resident_harness;

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use djinn_testkit::walls::{CanaryRoot, standard_endpoints};
use djinn_testkit::{Bound, HarnessError, Run, RunConfig, sys, verify};
use resident_harness::*;
use serde_json::json;

const PASSPHRASE: &str = "djinn-harness-receipt";
const ROLE: &str = "DJINN_HARNESS_ROLE";

/// The refusals, recorded as controls: every one must fail before a spawn.
#[test]
fn the_guard_refuses_every_wall_crossing_before_spawning() {
    if std::env::var_os(ROLE).is_some() {
        return;
    }
    let run = begin(
        "djinn-harness",
        "the_guard_refuses_every_wall_crossing_before_spawning",
    );
    let real_local = djinn_testkit::walls::RealRoots::from_environment()
        .local_data
        .display()
        .to_string();
    let mut cases: Vec<(String, djinn_testkit::Resident)> = Vec::new();
    for (index, standard) in standard_endpoints().into_iter().enumerate() {
        for door in ["agent", "browser", "app"] {
            let name = format!("std{index}{door}");
            cases.push((
                format!("{door} door on {standard}"),
                resident(&run, &name, "h", PASSPHRASE).endpoint_override(door, &standard),
            ));
        }
    }
    for variable in [
        "MERE_ROOT",
        "LOCALAPPDATA",
        "XDG_DATA_HOME",
        "GRAPHSHELL_APP_ENDPOINT",
    ] {
        cases.push((
            format!("{variable} left out"),
            resident(
                &run,
                &format!("no{}", variable.to_lowercase()),
                "h",
                PASSPHRASE,
            )
            .without(variable),
        ));
    }
    cases.push((
        "LOCALAPPDATA left at the real one".into(),
        resident(&run, "leak", "h", PASSPHRASE).env("LOCALAPPDATA", &real_local),
    ));
    cases.push((
        "SSH_AUTH_SOCK passed".into(),
        resident(&run, "sock", "h", PASSPHRASE).env("SSH_AUTH_SOCK", "agent.sock"),
    ));
    for (what, mut resident) in cases {
        let refused = resident.start(&[]);
        let spawned = resident.root().join("stderr-1.txt").exists();
        run.control(
            &format!("c1: {what}"),
            "refused before spawning",
            &format!("{refused:?}; spawned: {spawned}"),
            matches!(refused, Err(HarnessError::Refused(_))) && !spawned,
        );
    }
    run.finish();
}

/// H1: ready without logs, a kill, a graceful stop and restarts, all on the
/// same roots; H3: the record verifies.
#[test]
#[ignore = "runs a real djinn resident"]
fn one_resident_is_killed_stopped_and_restarted_on_the_same_roots() {
    let run = begin(
        "djinn-harness",
        "one_resident_is_killed_stopped_and_restarted_on_the_same_roots",
    );
    let dir = run.dir().to_path_buf();
    let mut r = resident(&run, "solo", "solo", PASSPHRASE);
    configure(&r, "solo", "djinn-harness-receipt");

    r.start(&[]).unwrap();
    let first = r.wait_ready(READY);
    let endpoints = json!({
        "agent": r.endpoint("agent"),
        "browser": r.endpoint("browser"),
        "app": r.endpoint("app"),
    });
    run.require(
        "the status describes this start",
        Bound::State,
        json!({
            "pid": r.pid(), "installed": false, "unlock": "passphrase",
            "protection": "passphrase", "lock": "unlocked", "listener": "receipt",
            "endpoints": endpoints,
        }),
        json!({
            "pid": first.pid, "installed": first.installed, "unlock": first.startup_unlock,
            "protection": first.protection, "lock": first.lock,
            "listener": first.endpoints.agent_listener,
            "endpoints": {
                "agent": first.endpoints.agent, "browser": first.endpoints.browser,
                "app": first.endpoints.app,
            },
        }),
        Some(first.pid) == r.pid()
            && !first.installed
            && first.startup_unlock == "passphrase"
            && first.lock == "unlocked"
            && first.endpoints.agent_listener == "receipt"
            && first.endpoints.agent == r.endpoint("agent")
            && first.endpoints.app == r.endpoint("app"),
    );
    let node = first.sync.clone().expect("sync is configured").node_id;
    let events: Vec<String> = r.events().into_iter().map(|e| e.event).collect();
    // A passphrase vault starts locked and, on fresh roots, is created from
    // the handed-over passphrase before any door listens (rulings 63 to 65).
    let expected = ["started", "waiting-for-unlock", "vault-created", "listening", "ready"];
    run.require(
        "the event file runs started, waiting-for-unlock, vault-created, listening, ready",
        Bound::State,
        expected,
        &events,
        events == expected,
    );

    // The crash case: no stopped event, and the same roots start again.
    r.kill();
    let killed: Vec<String> = r.events().into_iter().map(|e| e.event).collect();
    run.require(
        "a killed resident writes no stopped event",
        Bound::State,
        "no stopped",
        &killed,
        !killed.iter().any(|e| e == "stopped"),
    );
    for start in 2..=3 {
        r.start(&[]).unwrap();
        let again = r.wait_ready(READY);
        let same = again.sync.as_ref().map(|s| s.node_id.as_str()) == Some(node.as_str());
        run.require(
            &format!("start {start} comes back on the same roots"),
            Bound::State,
            &node,
            again.sync.as_ref().map(|s| s.node_id.clone()),
            same && again.pid != first.pid,
        );
        let asked = Instant::now();
        let exit = r.stop(STOP);
        let took = asked.elapsed();
        let tail: Vec<(String, Option<String>)> = r
            .events()
            .into_iter()
            .rev()
            .take(2)
            .map(|e| (e.event, e.reason))
            .collect();
        let expected = vec![
            ("stopped".to_string(), Some("stop intent".to_string())),
            ("stopping".to_string(), Some("stop intent".to_string())),
        ];
        run.require(
            &format!("start {start} stopped by the stop intent"),
            Bound::State,
            &expected,
            &tail,
            tail == expected && exit.is_some(),
        );
        run.step(
            "graceful stop",
            json!({ "start": start, "took_ms": took.as_millis() as u64 }),
        );
    }
    // The same stop with personal sync off, so the shutdown's own outcome is
    // seen apart from sync's (the testkit records "shut down cleanly").
    let mut quiet = resident(&run, "nosync", "nosync", PASSPHRASE);
    quiet.start(&[]).unwrap();
    quiet.wait_ready(READY);
    quiet.stop(STOP);
    let receipt = run.finish();
    let checked = verify(&dir).unwrap();
    assert!(
        checked.ok() && checked.checked == receipt.evidence.len(),
        "{checked:?}"
    );
    println!("RECEIPT verified {} evidence files", checked.checked);
}

/// A ready event is a usable boundary: the resident must already accept a
/// connection on each endpoint, with no endpoint-existence retry in between.
#[test]
#[cfg(unix)]
#[ignore = "runs a real djinn resident"]
fn ready_follows_listener_binding_and_each_door_accepts_immediately() {
    let run = begin(
        "djinn-harness",
        "ready_follows_listener_binding_and_each_door_accepts_immediately",
    );
    let mut r = resident(&run, "ready", "ready", PASSPHRASE);
    r.start(&[]).unwrap();

    let ready = run
        .wait_for("the resident ready event", READY, || {
            r.events().into_iter().find(|event| event.event == "ready")
        })
        .0;
    let endpoints = ["agent", "browser", "app"].map(|door| (door, r.endpoint(door)));
    for (door, endpoint) in endpoints {
        let connected = std::os::unix::net::UnixStream::connect(&endpoint).is_ok();
        run.require(
            &format!("{door} accepts immediately after ready"),
            Bound::State,
            true,
            json!({ "connected": connected, "endpoint": endpoint, "ready_pid": ready.pid }),
            connected,
        );
    }
    let status = r.status().expect("the app door answers after ready");
    assert!(status.ready && status.pid == ready.pid);

    r.stop(STOP);
    run.finish();
}

/// The app door binds first. A regular file at the browser endpoint forces a
/// deterministic second-listener bind failure, which must not report either
/// listening or ready.
#[test]
#[cfg(unix)]
#[ignore = "runs a real djinn resident"]
fn browser_listener_bind_failure_never_reports_listening_or_ready() {
    assert_bind_failure_never_reports_readiness("browser");
}

/// The first application-door bind can fail too; startup must not publish a
/// listening or ready event after this earlier failure.
#[test]
#[cfg(unix)]
#[ignore = "runs a real djinn resident"]
fn app_listener_bind_failure_never_reports_listening_or_ready() {
    assert_bind_failure_never_reports_readiness("app");
}

#[cfg(unix)]
fn assert_bind_failure_never_reports_readiness(door: &str) {
    let test_name = format!("{door}_listener_bind_failure_never_reports_listening_or_ready");
    let run = begin("djinn-harness", &test_name);
    let mut r = resident(&run, "blocked", "blocked", PASSPHRASE);
    let endpoint = std::path::PathBuf::from(r.endpoint(door));
    std::fs::create_dir_all(endpoint.parent().expect("endpoint has a parent")).unwrap();
    std::fs::write(&endpoint, "occupied by a regular file").unwrap();
    r.start(&[]).unwrap();

    let stopped = run
        .wait_for("the failed resident stopped event", READY, || {
            r.events()
                .into_iter()
                .find(|event| event.event == "stopped")
        })
        .0;
    let events = r.events();
    let names: Vec<&str> = events.iter().map(|event| event.event.as_str()).collect();
    let no_success_readiness = !names
        .iter()
        .any(|event| *event == "listening" || *event == "ready");
    let failed = stopped.other.get("ok").and_then(serde_json::Value::as_bool) == Some(false);
    let bind_error = stopped
        .other
        .get("error")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|error| error.contains("endpoint path exists and is not a Unix socket"));
    assert_eq!(
        std::fs::read_to_string(&endpoint).unwrap(),
        "occupied by a regular file"
    );
    if door == "browser" {
        use std::os::unix::fs::FileTypeExt;
        assert!(
            std::fs::symlink_metadata(r.endpoint("app"))
                .unwrap()
                .file_type()
                .is_socket(),
            "the browser failure follows a successful app bind"
        );
    }
    let starts_and_stops = names.first() == Some(&"started") && names.last() == Some(&"stopped");
    run.require(
        &format!("{door} bind failure exits without readiness"),
        Bound::State,
        json!({ "first_event": "started", "last_event": "stopped", "stopped_ok": false }),
        json!({ "events": names, "stopped": stopped.other }),
        starts_and_stops && no_success_readiness && failed && bind_error,
    );
    assert!(
        r.status().is_none(),
        "a failed resident has no status route"
    );
    drop(r); // Reap the stopped child before checking the run's process walls.
    run.finish();
}

/// F1: with personal sync on, a graceful stop cancels and joins every task
/// that borrows the blob store or the published-site route, and exits 0. The
/// control leaves one holder outside the task scope, and the shutdown's
/// borrower check refuses it.
#[test]
#[ignore = "runs a real djinn resident"]
fn a_graceful_stop_with_sync_on_exits_cleanly_and_a_left_holder_fails_it() {
    let run = begin(
        "djinn-harness",
        "a_graceful_stop_with_sync_on_exits_cleanly_and_a_left_holder_fails_it",
    );
    let mut r = resident(&run, "synced", "synced", PASSPHRASE);
    configure(&r, "synced", "djinn-harness-stop");
    r.start(&[]).unwrap();
    assert!(r.wait_ready(READY).sync.is_some(), "sync is on");
    r.stop(STOP);

    r.start(&["--control-unjoined-holder"]).unwrap();
    r.wait_ready(READY);
    let exit = r.stop_unjudged(STOP);
    let error = r
        .stopped_event()
        .and_then(|event| event.other.get("error").cloned())
        .unwrap_or_default();
    run.control(
        "f1: one blob-store holder left un-joined",
        "exit 1, the blob store's borrower check refusing",
        &format!("exit {:?}; {error}", exit.and_then(|e| e.code())),
        exit.and_then(|e| e.code()) == Some(1)
            && error
                .as_str()
                .is_some_and(|e| e.contains("resident blob store still has active borrowers")),
    );
    run.finish();
}

/// C3 with a real resident: the test process that holds the run is killed
/// mid-run, and the resident it spawned goes with it.
#[test]
#[ignore = "runs a real djinn resident"]
fn killing_the_test_process_leaves_no_resident() {
    if std::env::var(ROLE).as_deref() == Ok("driver") {
        let run = begin("djinn-harness", "c3-driver");
        let mut r = resident(&run, "orphan", "orphan", PASSPHRASE);
        r.start(&[]).unwrap();
        let status = r.wait_ready(READY);
        println!(
            "RESIDENT {} {}",
            status.pid,
            sys::started(status.pid).unwrap()
        );
        std::thread::sleep(Duration::from_secs(600));
        return;
    }
    if std::env::var_os(ROLE).is_some() {
        return;
    }
    let mut driver = Command::new(std::env::current_exe().unwrap())
        .args([
            "killing_the_test_process_leaves_no_resident",
            "--exact",
            "--ignored",
            "--nocapture",
        ])
        .env(ROLE, "driver")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(driver.stdout.take().unwrap()).lines();
    let (pid, started) = loop {
        let line = lines
            .next()
            .expect("the driver reports its resident")
            .unwrap();
        if let Some((_, rest)) = line.split_once("RESIDENT ") {
            let mut parts = rest.split_whitespace();
            break (
                parts.next().unwrap().parse::<u32>().unwrap(),
                parts.next().unwrap().parse::<u64>().unwrap(),
            );
        }
    };
    assert!(
        sys::alive(pid, started),
        "the resident runs while its driver does"
    );
    driver.kill().unwrap();
    driver.wait().unwrap();
    let killed = Instant::now();
    while sys::alive(pid, started) && killed.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(50));
    }
    let gone = !sys::alive(pid, started);
    let run = begin(
        "djinn-harness",
        "killing_the_test_process_leaves_no_resident",
    );
    run.control(
        "c3: the test process killed mid-run",
        "its resident is gone",
        &format!(
            "resident {pid} alive {} ms after the kill: {}",
            killed.elapsed().as_millis(),
            !gone
        ),
        gone,
    );
    run.finish();
}

/// C4's instrument with a real resident: `LOCALAPPDATA` aimed at a decoy and
/// exempted from the guard, and the decoy's canary catches what djinn wrote.
/// The real roots' canaries are checked by every other run's closing walls.
#[test]
#[ignore = "runs a real djinn resident"]
fn a_canary_catches_a_deliberately_unguarded_root() {
    let decoy = tempfile::tempdir().unwrap();
    let mut config = RunConfig::new(
        "djinn-harness",
        "a_canary_catches_a_deliberately_unguarded_root",
        DJINN,
    );
    config.walls.canaries.push(CanaryRoot {
        path: decoy.path().to_path_buf(),
        depth: 3,
        files: true,
    });
    let run = Run::begin(config).unwrap();
    let decoy_path = decoy.path().display().to_string();
    let mut r = resident(&run, "unguarded", "unguarded", PASSPHRASE)
        .unguarded_root("LOCALAPPDATA", &decoy_path);
    r.start(&[]).unwrap();
    r.wait_ready(READY);
    r.stop(STOP);
    let receipt = run.finish_record();
    let caught: Vec<&String> = receipt
        .walls
        .differences
        .iter()
        .filter(|d| d.contains(&decoy_path))
        .collect();
    let real: Vec<&String> = receipt
        .walls
        .differences
        .iter()
        .filter(|d| !d.contains(&decoy_path))
        .collect();
    assert!(!caught.is_empty(), "the decoy canary caught djinn's writes");
    assert!(real.is_empty(), "the real roots are unchanged: {real:?}");
    println!(
        "RECEIPT c4 caught {} decoy changes: {:?}",
        caught.len(),
        caught
    );
}
