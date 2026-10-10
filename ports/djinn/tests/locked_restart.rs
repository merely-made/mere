// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A lock persists across a restart (vault lock rulings 5, 76, 80): an
//! OS-rooted vault opens unattended until `ssh-add -x` locks it; killed and
//! started again, the resident waits for a user act before anything else,
//! here the passphrase the harness hands over, and the marker clears.

#![cfg(windows)]

mod resident_harness;

use std::path::{Path, PathBuf};

use castellan::custody::{IdentityStorage, SealedProfileStorage, bootstrap};
use djinn_testkit::Bound;
use personae::ProfileId;
use resident_harness::*;
use serde_json::json;

const PASSPHRASE: &str = "djinn-restart-receipt";

fn openssh(tool: &str) -> PathBuf {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    Path::new(&root)
        .join("System32")
        .join("OpenSSH")
        .join(format!("{tool}.exe"))
}

fn events(r: &djinn_testkit::Resident) -> Vec<String> {
    r.events().into_iter().map(|e| e.event).collect()
}

#[test]
#[ignore = "runs a real djinn resident and the system's OpenSSH tools"]
fn a_lock_persists_across_a_restart_and_waits_for_a_user_act() {
    let run = begin(
        "vault-lock",
        "a_lock_persists_across_a_restart_and_waits_for_a_user_act",
    );
    let mut r = resident(&run, "restart", "restart", PASSPHRASE);

    // An OS-rooted vault in the resident's own root, with a passphrase
    // enrolled so a lock is allowed (rulings 27, 39).
    let vault = r.vault_dir();
    {
        let storage = SealedProfileStorage::open_auto_os(&vault).unwrap().unwrap();
        bootstrap::load_or_create_profile(&storage, &ProfileId("restart".into())).unwrap();
        storage.enroll_passphrase(PASSPHRASE.as_bytes()).unwrap();
    }

    r.start(&[]).unwrap();
    let first = r.wait_ready(READY);
    let opened = events(&r);
    run.require(
        "never locked, the OS vault opens unattended",
        Bound::State,
        json!({ "lock": "unlocked", "unlock": "auto_os", "asked": false }),
        json!({
            "lock": first.lock,
            "unlock": first.startup_unlock,
            "asked": opened.iter().any(|e| e == "waiting-for-unlock"),
        }),
        first.lock == "unlocked"
            && first.startup_unlock == "auto_os"
            && !opened.iter().any(|e| e == "waiting-for-unlock"),
    );

    let askpass = r.root().join("askpass-unused.bat");
    std::fs::write(&askpass, "@echo unused\r\n").unwrap();
    let askpass = askpass.display().to_string();
    let locked = r
        .ssh_client_env(
            &openssh("ssh-add"),
            &["-x"],
            b"",
            &[("SSH_ASKPASS", &askpass), ("SSH_ASKPASS_REQUIRE", "force")],
        )
        .unwrap();
    let status = r.status().expect("the status route answers");
    run.require(
        "ssh-add -x locks, and the lock persists beside the vault",
        Bound::State,
        json!({ "exit": 0, "lock": "locked", "marker": true }),
        json!({
            "exit": locked.status.code(),
            "lock": status.lock,
            "marker": castellan::custody::lock_persisted(&vault),
        }),
        locked.status.success()
            && status.lock == "locked"
            && castellan::custody::lock_persisted(&vault),
    );

    // A crash, not a stop: the launcher would start it again.
    r.kill();
    r.start(&[]).unwrap();
    let again = r.wait_ready(READY);
    let restarted = events(&r);
    let waited = restarted.iter().position(|e| e == "waiting-for-unlock");
    let unlocked = restarted.iter().position(|e| e == "unlocked-at-start");
    let listening = restarted.iter().position(|e| e == "listening");
    run.require(
        "restarted, it waits for a user act before any door, then the marker clears",
        Bound::State,
        json!({ "order": "waiting-for-unlock < unlocked-at-start < listening", "marker": false, "lock": "unlocked" }),
        json!({ "events": restarted, "marker": castellan::custody::lock_persisted(&vault), "lock": again.lock }),
        matches!((waited, unlocked, listening), (Some(w), Some(u), Some(l)) if w < u && u < l)
            && !castellan::custody::lock_persisted(&vault)
            && again.lock == "unlocked",
    );
    r.stop(STOP);
}
