// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reservoir plan V1, the two-process receipt.
//!
//! While one process owns a persona's reservoir, a second *process* that
//! tries to own it is refused at once rather than left waiting on the lock.
//! Once the first lets go, the second opens it and ensures a mere through the
//! admitted route, and the next owner finds that mere. The second process is
//! this test binary re-run by the parent, so the two reservoir handles live in
//! separate address spaces, as they would for djinn and a stray second
//! resident.

use std::path::Path;
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use chirograph::{IntentInvocation, IntentResult, ProjectionSession};
use djinn::resident_reservoir::{
    EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
};
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use personae::PersonaId;
use sceno::InstanceId;
use uuid::Uuid;

const ROOT_ENV: &str = "DJINN_RESERVOIR_RECEIPT_ROOT";
const EXPECT_ENV: &str = "DJINN_RESERVOIR_RECEIPT_EXPECT";
const CHILD_TEST: &str = "child_attempts_to_own_the_reservoir";

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x5e5e_7e7e))
}

/// Re-run this test binary as a separate process, running only the child.
fn run_child(root: &Path, expect: &str) -> Output {
    Command::new(std::env::current_exe().expect("the test binary knows its own path"))
        .args(["--exact", CHILD_TEST, "--ignored", "--nocapture"])
        .args(["--test-threads", "1"])
        .env(ROOT_ENV, root)
        .env(EXPECT_ENV, expect)
        .output()
        .expect("the child process starts")
}

/// Echo the child's receipt lines so a `--nocapture` run shows them.
fn echo_receipt(output: &Output) {
    // libtest prints its own "test name ... " prefix on the same line.
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        if let Some(at) = line.find("RECEIPT") {
            println!("child {}", &line[at..]);
        }
    }
}

fn describe(output: &Output) -> String {
    format!(
        "status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[tokio::test(flavor = "multi_thread")]
async fn a_second_process_is_refused_while_the_first_owns_the_reservoir() {
    let root = tempfile::tempdir().unwrap();
    let owner = ResidentReservoir::open(root.path(), Some(persona()))
        .await
        .unwrap();

    let started = Instant::now();
    let refused = run_child(root.path(), "refused");
    let waited = started.elapsed();
    echo_receipt(&refused);
    assert!(refused.status.success(), "{}", describe(&refused));
    assert!(
        String::from_utf8_lossy(&refused.stdout).contains("RECEIPT refused:"),
        "{}",
        describe(&refused)
    );
    // Spawning a process costs seconds at most; a refusal that waited on the
    // lock would not come back at all.
    assert!(
        waited < Duration::from_secs(60),
        "the refusal took {waited:?}"
    );

    drop(owner);
    let opened = run_child(root.path(), "opened");
    echo_receipt(&opened);
    assert!(opened.status.success(), "{}", describe(&opened));
    assert!(
        String::from_utf8_lossy(&opened.stdout).contains("RECEIPT opened:"),
        "{}",
        describe(&opened)
    );

    // What the second process wrote is there for the next owner.
    let next = ResidentReservoir::open(root.path(), Some(persona()))
        .await
        .unwrap();
    let domains: Vec<String> = next
        .meres()
        .await
        .into_iter()
        .map(|mere| mere.domain.to_string())
        .collect();
    assert_eq!(domains, ["divination"]);
}

/// The second process. It runs only when the parent above starts it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "runs only as the child process of the two-process receipt"]
async fn child_attempts_to_own_the_reservoir() {
    let (Some(root), Ok(expect)) = (std::env::var_os(ROOT_ENV), std::env::var(EXPECT_ENV)) else {
        return;
    };
    let attempt = ResidentReservoir::open(Path::new(&root), Some(persona())).await;
    match (expect.as_str(), attempt) {
        ("refused", Err(reason)) => {
            assert!(reason.contains("could not open the reservoir"), "{reason}");
            println!("RECEIPT refused: {reason}");
        },
        ("opened", Ok(reservoir)) => {
            let mut catalog = ResidentEndpointCatalog::new();
            reservoir.register(&mut catalog).unwrap();
            let context =
                AdmittedEndpointContext::new(ProjectionSession("v1:receipt".into()), [0x2b; 32]);
            let mut session = catalog.open(RESIDENT_RESERVOIR_ROUTE, &context).unwrap();
            let request = session.describe().projections.remove(0).request;
            let snapshot = session.snapshot(request).unwrap();
            let result = session
                .invoke(IntentInvocation {
                    session: snapshot.session.clone(),
                    target: InstanceId(0),
                    observed_epoch: snapshot.scene.epoch,
                    observed_revision: snapshot.scene.revision,
                    intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
                    payload: serde_json::to_vec(&EnsureMereV1::new("divination")).unwrap(),
                })
                .unwrap();
            assert_eq!(result, IntentResult::Accepted);
            println!("RECEIPT opened: ensured divination as a second process");
        },
        (expect, Ok(_)) => panic!("expected {expect}, but the second process owned the reservoir"),
        (expect, Err(reason)) => panic!("expected {expect}, but the open failed: {reason}"),
    }
}
