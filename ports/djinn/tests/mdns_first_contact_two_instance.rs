// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pairing plan D1b (ruling 16): two djinn residents on one machine, paired
//! both ways with no ticket and no `--sync-peer`, meet by local discovery
//! alone, and meet again after either side restarts with its saved dial hints
//! removed, so mDNS stays the only way in.
//!
//! Its second restart fails until a separate overlay fix lands (pairing plan
//! ruling 36).
//!
//! Ignored by default: it runs two real residents for a few minutes, on
//! `djinn_testkit` (harness plan ruling 10). The record and everything the
//! residents write land under `DJINN_RECEIPTS_ROOT`:
//!
//! ```text
//! DJINN_RECEIPTS_ROOT=C:\t\receipts cargo test -p djinn --test mdns_first_contact_two_instance -- --ignored --nocapture
//! ```

mod resident_harness;

use std::collections::BTreeSet;
use std::time::{Duration, Instant};

use djinn::settings::OwnerSettings;
use djinn_testkit::{Bound, Resident, Run};
use personae::ProfileId;
use resident_harness::*;

const PASSPHRASE: &str = "djinn-mdns-first-contact-receipt";
const GRAPH: &str = "pairing-d1b-receipt";
/// Long enough for several pairing polls and mDNS query rounds; D1 saw no
/// contact in 45 s, and a ticket connects in under 4 s.
const PATIENCE: Duration = Duration::from_secs(90);

/// Remove every saved dial hint, so the next start has only mDNS.
fn forget_hints(side: &Resident, profile: &str) -> usize {
    let path = djinn::settings::settings_path(&side.app_dir(), &ProfileId(profile.into()));
    let mut settings = OwnerSettings::load(&path).unwrap();
    let sync = settings.sync.as_mut().expect("sync is configured");
    let forgotten = sync
        .paired_devices
        .iter_mut()
        .filter_map(|device| device.last_endpoint.take())
        .count();
    settings.save(&path).unwrap();
    forgotten
}

/// Contact after `fresh` (re)started: the fresh process sees its peer
/// connected, and the other side's live path is one of the fresh process's
/// own ports, so a path left over from an earlier process cannot pass.
fn contact(
    run: &Run,
    label: &str,
    (fresh, fresh_node, fresh_ports, spawned): (&Resident, &str, &BTreeSet<u16>, Instant),
    (other, other_node): (&Resident, &str),
) {
    let watching = Instant::now();
    let (_, took) = run.wait_for(&format!("{label}: contact"), PATIENCE, || {
        let mine = sees(fresh, other_node)?.connected;
        let theirs = sees(other, fresh_node)?;
        let on_new_path = !active_direct_ports(&theirs).is_disjoint(fresh_ports);
        (mine && theirs.connected && on_new_path).then_some(())
    });
    println!(
        "RECEIPT {label}: both connected {:?} after {} was spawned \
         (watched from {:?} after spawn, for {took:?}); {} listens on {fresh_ports:?}",
        spawned.elapsed(),
        fresh.name(),
        watching.duration_since(spawned),
        fresh.name()
    );
}

/// Both sides hold a saved hint for the other, so removing them is a change.
fn wait_for_hints(run: &Run, a: &Resident, a_node: &str, b: &Resident, b_node: &str) {
    let (_, took) = run.wait_for("hints saved on both sides", PATIENCE, || {
        let on_a = sees(a, b_node)?.hint.is_some();
        let on_b = sees(b, a_node)?.hint.is_some();
        (on_a && on_b).then_some(())
    });
    println!("RECEIPT hints saved on both sides after {took:?}");
}

/// Kill `side`, remove its saved hints, and start it again.
fn restart_without_hints(
    run: &Run,
    side: &mut Resident,
    profile: &str,
) -> (BTreeSet<u16>, Instant) {
    side.kill();
    let forgotten = forget_hints(side, profile);
    run.require(
        &format!("{} held one saved hint", side.name()),
        Bound::State,
        1,
        forgotten,
        forgotten == 1,
    );
    let spawned = side.start(&[]).unwrap();
    let ports = listening_ports(&side.wait_ready(READY));
    println!(
        "RECEIPT {} restarted without its hint, listening {:?} after spawn",
        side.name(),
        spawned.elapsed()
    );
    (ports, spawned)
}

#[test]
#[ignore = "runs two real djinn residents; see the module note"]
fn two_paired_residents_meet_by_mdns_alone_and_again_after_restarts() {
    let run = begin(
        "pairing-d1b",
        "two_paired_residents_meet_by_mdns_alone_and_again_after_restarts",
    );
    println!("RECEIPT root {}", run.dir().display());
    let mut a = resident(&run, "a", "d1ba", PASSPHRASE);
    let mut b = resident(&run, "b", "d1bb", PASSPHRASE);

    // Pair with the ordinary commands, both ways, and no ticket anywhere.
    configure(&a, "d1ba", GRAPH);
    configure(&b, "d1bb", GRAPH);
    let (a_node, a_root) = facts(&a);
    let (b_node, b_root) = facts(&b);
    pair(&a, &b_node, &b_root, "d1b-b");
    pair(&b, &a_node, &a_root, "d1b-a");
    println!("RECEIPT a {a_node}\nRECEIPT b {b_node}");

    // First contact: no ticket, no hint, no `--sync-peer`.
    let a_spawned = a.start(&[]).unwrap();
    let a_ports = listening_ports(&a.wait_ready(READY));
    println!("RECEIPT a listening {:?} after spawn", a_spawned.elapsed());
    let b_spawned = b.start(&[]).unwrap();
    let b_ports = listening_ports(&b.wait_ready(READY));
    println!("RECEIPT b listening {:?} after spawn", b_spawned.elapsed());
    contact(
        &run,
        "first contact",
        (&b, &b_node, &b_ports, b_spawned),
        (&a, &a_node),
    );

    // Each restart removes the restarted side's saved hint first, so mDNS
    // stays its only way back: first contact again, not the cached-hint rung.
    wait_for_hints(&run, &a, &a_node, &b, &b_node);
    let (b_ports, b_spawned) = restart_without_hints(&run, &mut b, "d1bb");
    contact(
        &run,
        "b restarted",
        (&b, &b_node, &b_ports, b_spawned),
        (&a, &a_node),
    );

    wait_for_hints(&run, &a, &a_node, &b, &b_node);
    let (a_ports_now, a_spawned) = restart_without_hints(&run, &mut a, "d1ba");
    run.require(
        "a came back on new ports",
        Bound::State,
        &a_ports,
        &a_ports_now,
        a_ports_now.is_disjoint(&a_ports),
    );
    contact(
        &run,
        "a restarted",
        (&a, &a_node, &a_ports_now, a_spawned),
        (&b, &b_node),
    );

    a.kill();
    b.kill();
    run.finish();
}
