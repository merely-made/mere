// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pairing plan D1, the local receipt (ruling 14): two djinn residents on one
//! machine, each with its own vault, profile, data and settings, paired with
//! the ordinary `--pairing-facts` and `--pair-node` commands, and each read
//! through `djinn-devices` over its own application door.
//!
//! It proves three things: the directory lists the peer, connected, on a path;
//! a peer restarted on a new endpoint shows that endpoint as its path and in
//! its saved hint within one pairing poll; a stopped peer reads as not
//! connected (the negative control).
//!
//! Ignored by default: it runs two real residents with local discovery for a
//! minute or two, on `djinn_testkit` (harness plan ruling 10). The record and
//! everything the residents write land under `DJINN_RECEIPTS_ROOT`:
//!
//! ```text
//! DJINN_RECEIPTS_ROOT=C:\t\receipts cargo test -p djinn --test device_directory_two_instance -- --ignored --nocapture
//! ```

mod resident_harness;

use std::time::Duration;

use djinn_testkit::Bound;
use resident_harness::*;

const PASSPHRASE: &str = "djinn-device-directory-receipt";
const GRAPH: &str = "pairing-d1-receipt";
/// The resident's pairing poll (`personal_sync::PAIRING_POLL`).
const POLL: Duration = Duration::from_secs(5);
/// One CLI round trip on top of a poll.
const SLACK: Duration = Duration::from_secs(2);
const PATIENCE: Duration = Duration::from_secs(120);

#[test]
#[ignore = "runs two real djinn residents; see the module note"]
fn two_paired_residents_report_each_other_through_the_directory() {
    let run = begin(
        "pairing-d1",
        "two_paired_residents_report_each_other_through_the_directory",
    );
    println!("RECEIPT root {}", run.dir().display());
    let mut a = resident(&run, "a", "d1a", PASSPHRASE);
    let mut b = resident(&run, "b", "d1b", PASSPHRASE);

    // Pair with the ordinary commands, both ways.
    configure(&a, "d1a", GRAPH);
    configure(&b, "d1b", GRAPH);
    let (a_node, a_root) = facts(&a);
    let (b_node, b_root) = facts(&b);
    pair(&a, &b_node, &b_root, "d1-b");
    pair(&b, &a_node, &a_root, "d1-a");
    println!("RECEIPT a {a_node}\nRECEIPT b {b_node}");

    // The first contact needs an address to dial. Local discovery resolves
    // each peer's address on this machine but nothing dials on it (see the
    // D1 report), so b's first start carries a's ticket, the existing
    // bootstrap. Every later start dials from the saved hints alone.
    a.start(&[]).unwrap();
    let a_ticket = ticket(&a.wait_ready(READY));
    let b_started = b.start(&["--sync-peer", &a_ticket]).unwrap();
    let b_ports = listening_ports(&b.wait_ready(READY));
    let (first, took) = run.wait_for("b connected in a's directory", PATIENCE, || {
        let device = sees(&a, &b_node)?;
        (device.connected && !active_direct_ports(&device).is_empty()).then_some(device)
    });
    println!(
        "RECEIPT connected after {:?} (b started {:?} ago); b listens on {b_ports:?}",
        took,
        b_started.elapsed()
    );
    assert_eq!(first.label, "d1-b");
    assert_eq!(first.root.as_deref(), Some(b_root.as_str()));
    assert!(first.pairing_id.is_some() && first.added_ms > 0);
    run.require(
        "a's live path to b is one of b's endpoints",
        Bound::State,
        &b_ports,
        active_direct_ports(&first),
        !active_direct_ports(&first).is_disjoint(&b_ports),
    );
    let (_, took) = run.wait_for("a connected in b's directory", PATIENCE, || {
        sees(&b, &a_node)?.connected.then_some(())
    });
    println!("RECEIPT b sees a connected after a further {took:?}");
    println!("RECEIPT human output from a:\n{}", human(&a));
    // The route agrees with the resident's own peer directory, which logs at
    // its next poll. An assertion on the log's content, not a readiness wait.
    let (_, took) = run.wait_for("a's resident logging b connected", POLL + SLACK, || {
        let logged = std::fs::read_to_string(a.log_path()).ok()?;
        logged
            .contains(&format!("\"{b_node}\", true, true"))
            .then_some(())
    });
    println!("RECEIPT a's resident logged b connected {took:?} later");

    // Restart b on a new endpoint.
    // Both sides hold a saved hint for the other before b goes away.
    run.wait_for("hints saved on both sides", PATIENCE, || {
        let on_a = !hint_ports(&sees(&a, &b_node)?).is_empty();
        let on_b = !hint_ports(&sees(&b, &a_node)?).is_empty();
        (on_a && on_b).then_some(())
    });
    b.kill();
    let restarted = b.start(&[]).unwrap();
    let new_ports = listening_ports(&b.wait_ready(READY));
    let listening = restarted.elapsed();
    run.require(
        "b came back on new ports",
        Bound::State,
        &b_ports,
        &new_ports,
        new_ports.is_disjoint(&b_ports),
    );
    let (moved, path_took) = run.wait_for("b's new path in a's directory", PATIENCE, || {
        let device = sees(&a, &b_node)?;
        (device.connected && !active_direct_ports(&device).is_disjoint(&new_ports))
            .then_some(device)
    });
    let (hinted, hint_took) = run.wait_for("b's new hint in a's directory", PATIENCE, || {
        let device = sees(&a, &b_node)?;
        (!hint_ports(&device).is_disjoint(&new_ports)).then_some(device)
    });
    println!(
        "RECEIPT restart: b listening {listening:?} after spawn on {new_ports:?}; \
         new path {path_took:?} later; saved hint a further {hint_took:?}"
    );
    println!(
        "RECEIPT moved path {:?}\nRECEIPT moved hint {:?}",
        moved.path, hinted.hint
    );
    run.require(
        "the saved hint follows the live path within one poll",
        Bound::Relative,
        (POLL + SLACK).as_millis() as u64,
        hint_took.as_millis() as u64,
        hint_took <= POLL + SLACK,
    );

    // The negative control: a stopped peer reads as not connected.
    b.kill();
    let (stopped, took) = run.wait_for("b disconnected in a's directory", PATIENCE, || {
        let device = sees(&a, &b_node)?;
        (!device.connected).then_some(device)
    });
    run.assertion(
        "a stopped peer reads not connected (after b's kill)",
        Bound::Relative,
        "about 10 to 18 s",
        took.as_millis() as u64,
        true,
    );
    println!("RECEIPT stopped: not connected after {took:?}; {stopped:?}");
    println!("RECEIPT human output from a:\n{}", human(&a));
    a.kill();
    run.finish();
}
