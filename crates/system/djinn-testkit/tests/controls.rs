// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The harness's own controls, without a resident: each must fail where it
//! should (harness plan C1, C3, C4, C5). The spawned process is this test
//! binary re-run in a role, so nothing here needs djinn.

use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use djinn_testkit::walls::{CanaryRoot, InstalledProcess, WallSpec};
use djinn_testkit::{Bound, Controls, Run, RunConfig, sys, verify};

const ROLE: &str = "DJINN_TESTKIT_ROLE";

fn config(test: &str) -> RunConfig {
    RunConfig::new("djinn-testkit", test, std::env::current_exe().unwrap())
}

/// The re-run roles: a sleeper, and a driver that spawns one and waits.
fn role() -> Option<String> {
    std::env::var(ROLE).ok()
}

fn sleeper_args() -> Vec<String> {
    [
        "role_entry",
        "--exact",
        "--nocapture",
        "--test-threads",
        "1",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect()
}

#[test]
fn role_entry() {
    match role().as_deref() {
        Some("sleeper") => std::thread::sleep(Duration::from_secs(600)),
        Some("driver") => {
            let run = Run::begin(config("c3-driver")).unwrap();
            let sleeper = run.resident("sleeper").env(ROLE, "sleeper");
            let mut child = sleeper
                .spawn(&std::env::current_exe().unwrap(), &sleeper_args())
                .unwrap();
            let pid = child.id();
            println!("SLEEPER {pid} {}", sys::started(pid).unwrap());
            let _ = child.wait();
            drop(run);
        },
        _ => {},
    }
}

/// C5: a tampered evidence file fails verification; the untouched record
/// verifies.
#[test]
fn a_tampered_evidence_file_fails_verification() {
    if role().is_some() {
        return;
    }
    let run = Run::begin(config("c5-tamper")).unwrap();
    let dir = run.dir().to_path_buf();
    run.step("evidence written", serde_json::json!({}));
    run.write_evidence("fact", &serde_json::json!({ "answer": 42 }));
    run.assertion("the fact is recorded", Bound::State, 42, 42, true);
    let receipt = run.finish();
    assert_eq!(receipt.schema, "mere.djinn.receipt/v1");
    assert!(receipt.binary.sha256.is_some() && !receipt.evidence.is_empty());
    let clean = verify(&dir).unwrap();
    assert!(clean.ok(), "{:?}", clean.problems);
    std::fs::write(dir.join("evidence").join("fact.json"), b"{\"answer\":41}").unwrap();
    let tampered = verify(&dir).unwrap();
    assert!(!tampered.ok());
    assert!(tampered.problems[0].contains("evidence/fact.json"));
}

/// C1: a guard fed a changed installed identity fails the receipt.
#[test]
fn a_changed_installed_identity_fails_the_receipt() {
    if role().is_some() {
        return;
    }
    let mut config = config("c1-identity");
    config.controls = Controls {
        perturb_installed_identity: true,
    };
    let run = Run::begin(config).unwrap();
    let before: Vec<InstalledProcess> = run.walls_before().installed.clone();
    let receipt = run.finish_record();
    assert!(!receipt.outcome.passed);
    assert!(
        receipt
            .outcome
            .failures
            .iter()
            .any(|f| f.contains("installed resident changed")),
        "{:?} (installed before: {before:?})",
        receipt.outcome.failures
    );
}

/// C4's instrument: a canary under a decoy root catches a write, and an
/// untouched decoy reads unchanged.
#[test]
fn a_canary_catches_a_write_under_its_root() {
    if role().is_some() {
        return;
    }
    let decoy = std::env::temp_dir().join(format!("djinn-testkit-decoy-{}", std::process::id()));
    std::fs::create_dir_all(decoy.join("settings")).unwrap();
    let spec = WallSpec {
        installed: Vec::new(),
        standard_endpoints: Vec::new(),
        canaries: vec![CanaryRoot {
            path: decoy.clone(),
            depth: 2,
            files: true,
        }],
    };
    let before = spec.capture();
    assert!(before.differences(&spec.capture()).is_empty());
    std::fs::write(decoy.join("settings").join("profile.json"), b"{}").unwrap();
    let after = spec.capture();
    let changed = before.differences(&after);
    assert!(
        changed.iter().any(|c| c.contains("profile.json")),
        "{changed:?}"
    );
    std::fs::remove_dir_all(&decoy).unwrap();
}

/// C3: killing the process that holds the run leaves nothing it spawned.
#[test]
fn killing_the_test_process_leaves_no_spawned_process() {
    if role().is_some() {
        return;
    }
    let mut driver = Command::new(std::env::current_exe().unwrap())
        .args(sleeper_args())
        .env(ROLE, "driver")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut lines = BufReader::new(driver.stdout.take().unwrap()).lines();
    let (pid, started) = loop {
        let line = lines
            .next()
            .expect("the driver reports its sleeper")
            .unwrap();
        // libtest may print the test's name on the same line first.
        if let Some(rest) = line.split_once("SLEEPER ").map(|(_, rest)| rest) {
            let mut parts = rest.split_whitespace();
            break (
                parts.next().unwrap().parse::<u32>().unwrap(),
                parts.next().unwrap().parse::<u64>().unwrap(),
            );
        }
    };
    assert!(
        sys::alive(pid, started),
        "the sleeper runs while its driver does"
    );
    driver.kill().unwrap();
    driver.wait().unwrap();
    let killed = Instant::now();
    while sys::alive(pid, started) && killed.elapsed() < Duration::from_secs(10) {
        std::thread::sleep(Duration::from_millis(50));
    }
    let gone = !sys::alive(pid, started);
    let run = Run::begin(config("c3-orphans")).unwrap();
    run.control(
        "c3: the driver killed mid-run",
        "its spawned sleeper is gone",
        &format!("sleeper {pid} alive after kill: {}", !gone),
        gone,
    );
    run.finish();
}
