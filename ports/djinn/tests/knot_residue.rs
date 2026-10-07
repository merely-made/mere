// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What Knot leaves of its signing seed once its lane closes (vault lock
//! ruling 48): personae's tracker, included by path, over a Knot source, a
//! session and a sync host on a temp data root, opened and closed the way
//! the resident's lane does.
//!
//! Two runs: open only (the seed must be seen live, the instrument's own
//! control for Knot's allocations), then open and close. A live copy after
//! the close fails: that would be djinn's lane holding on. Copies freed
//! uncleared inside Knot, p2panda or iroh are printed as a finding for the
//! upstream ledger; Knot is not patched.
//!
//! No libtest harness: the allocator is process-wide.

#[path = "../../../crates/dramatis/personae/tests/residue/mod.rs"]
mod residue;

use knot_editor::{
    KnotResidentSource, KnotSyncEvent, KnotSyncFileStore, KnotSyncHost, KnotSyncHostConfig,
    KnotVault, KnotWriteGrant, VaultDocument,
};
use residue::*;

const PHASES: [&str; 4] = ["setup", "open", "close", "drop"];

async fn lane(root: &std::path::Path, seed: [u8; 32], close: bool) {
    let writer = *p2panda_core::SigningKey::from_bytes(&seed)
        .verifying_key()
        .as_bytes();
    let space = [0x91; 32];
    std::fs::create_dir_all(root).unwrap();
    phase(1);
    let vault = KnotVault::open(root.join("vault"), [0xa1; 32]).unwrap();
    let store = KnotSyncFileStore::open(root.join("sync.redb"), space, [writer]).unwrap();
    store
        .author(
            seed,
            &vault,
            &KnotSyncEvent::Put(VaultDocument {
                id: "note".into(),
                title: "Note".into(),
                body: b"# Residue\n".to_vec(),
                media_type: "text/djot".into(),
            }),
        )
        .await
        .unwrap();
    let source = KnotResidentSource::from_synced_vault(vault, store.clone(), seed).unwrap();
    let session = source.session(Some(KnotWriteGrant::new(4096)));
    let host = KnotSyncHost::open(&store, seed, KnotSyncHostConfig::default())
        .await
        .unwrap();
    if !close {
        // Held until the scan: the live control.
        std::mem::forget((session, source, host, store));
        return;
    }
    phase(2);
    drop(session);
    host.close().await.unwrap();
    phase(3);
    drop(source);
    drop(store);
}

fn main() {
    let seed: [u8; 32] = canary_bytes(0);
    positive_control(&seed);
    clear_canaries();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let mut report = Report { failures: 0 };

    plant(0, "Knot signing seed", &seed, true);
    arm();
    runtime.block_on(lane(&dir.path().join("open"), seed, false));
    let (hits, overflow) = disarm();
    let live = hits.iter().filter(|h| h.live).count();
    println!("knot control: the open lane holds the seed in {live} live block(s)");
    assert!(!overflow && live > 0, "the instrument must see Knot's live seed");

    arm();
    runtime.block_on(lane(&dir.path().join("closed"), seed, true));
    // Let the runtime's workers finish what the close handed them.
    runtime.block_on(async { tokio::time::sleep(std::time::Duration::from_millis(500)).await });
    let (hits, overflow) = disarm();
    report.check("Knot lane closed", &PHASES, &hits, overflow);
    clear_canaries();
    drop(runtime);

    // djinn's own guarantee: once the lane is closed nothing holds the seed.
    // Copies freed uncleared inside knot-editor, p2panda or iroh are printed
    // above as a finding for the upstream ledger, and not failed here.
    let live = hits.iter().filter(|h| h.live).count();
    let freed = hits.len() - live;
    println!("knot residue: {live} live, {freed} freed uncleared (upstream finding)");
    if live > 0 {
        println!("knot residue: FAILED (the closed lane still holds the seed)");
        std::process::exit(1);
    }
    println!("knot residue: no live seed after the close");
}
