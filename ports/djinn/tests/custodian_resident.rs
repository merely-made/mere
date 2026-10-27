// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident with a custodian (custody isolation plan, H2): a real
//! `djinn` with no vault of its own reaches a custodian through the client
//! file it is handed, unlocks it with the start's passphrase, serves its
//! doors from custody, and follows a lock taken on the custodian's side.

#![cfg(unix)]

mod resident_harness;

use std::time::{Duration, Instant};

use djinn::custody_host::ResidentCustody;
use djinn_testkit::Bound;
use resident_harness::*;
use serde_json::json;

const PASSPHRASE: &str = "custodian resident receipt";

fn wait_lock(r: &djinn_testkit::Resident, want: &str) -> String {
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        let lock = r.status().map(|s| s.lock).unwrap_or_default();
        if lock == want || Instant::now() > deadline {
            return lock;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

#[test]
#[ignore = "runs a real djinn resident against a custodian"]
fn a_resident_with_a_custodian_opens_no_vault_and_follows_its_lock() {
    let run = begin("custody-isolation", "a_resident_with_a_custodian_opens_no_vault_and_follows_its_lock");
    let r = resident(&run, "client", "work", PASSPHRASE);
    let settings = djinn::settings::settings_path(&r.app_dir(), &personae::ProfileId("work".into()));
    std::fs::create_dir_all(settings.parent().unwrap()).unwrap();
    std::fs::write(&settings, r#"{"reservoir":{"enabled":false}}"#).unwrap();

    let grants: serde_json::Value = serde_json::from_str(
        &include_str!("../custodian/packaging/djinn-grants.json.in")
            .replace("@CONSENT@", "none")
            .replace("@SIGN_CONSENT@", "none"),
    )
    .unwrap();
    let fixture = djinn_custodian::fixture::start(
        &r.root().join("custodian"),
        djinn_custodian::fixture::Spec {
            passphrase: PASSPHRASE,
            profiles: &[("work", "Work", [0x77; 32])],
            grants,
            consent: "deny",
            mechanism: None,
            agent: true,
        },
    )
    .unwrap();
    let client_file = fixture.client_file.display().to_string();
    let agent = fixture.agent.clone().unwrap().display().to_string();

    // The custodian serves the SSH agent; the resident binds none.
    let mut r = r.endpoint_override("agent", &agent);
    r.start(&["--custodian", &client_file]).unwrap();
    let ready = r.wait_ready(READY);
    let vault_files = std::fs::read_dir(r.vault_dir()).map(|d| d.count()).unwrap_or(0);
    run.require(
        "the resident starts from custody and opens no vault of its own",
        Bound::State,
        json!({ "lock": "unlocked", "protection": "passphrase", "vault_files": 0, "agent": agent }),
        json!({ "lock": ready.lock, "protection": ready.protection, "vault_files": vault_files, "agent": ready.endpoints.agent }),
        ready.lock == "unlocked" && ready.protection == "passphrase" && vault_files == 0 && ready.endpoints.agent == agent,
    );
    let listed = r
        .ssh_client(std::path::Path::new("/usr/bin/ssh-add"), &["-L"], b"")
        .expect("ssh-add runs against the custodian's agent");
    run.require(
        "the custodian's agent answers the owner's SSH client",
        Bound::State,
        json!({ "answered": true }),
        json!({ "answered": String::from_utf8_lossy(&listed.stdout).contains("no identities") || String::from_utf8_lossy(&listed.stderr).contains("no identities") }),
        String::from_utf8_lossy(&listed.stdout).contains("no identities") || String::from_utf8_lossy(&listed.stderr).contains("no identities"),
    );

    fixture.service.lock_now();
    let locked = wait_lock(&r, "locked");
    run.require(
        "a lock taken by the custodian reaches the resident",
        Bound::State,
        json!({ "lock": "locked" }),
        json!({ "lock": locked }),
        locked == "locked",
    );

    let owner = djinn::custody_host::remote::RemoteHost::connect(
        djinn::custodian::InstalledCustodian::from_path(&fixture.client_file, false).unwrap(),
    )
    .unwrap();
    owner.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    let unlocked = wait_lock(&r, "unlocked");
    run.require(
        "an unlock on the custodian reaches the resident",
        Bound::State,
        json!({ "lock": "unlocked" }),
        json!({ "lock": unlocked }),
        unlocked == "unlocked",
    );
    let exit = r.stop(STOP);
    run.require(
        "the resident stops cleanly",
        Bound::State,
        json!({ "ok": true }),
        json!({ "ok": exit.is_some_and(|s| s.success()) }),
        exit.is_some_and(|s| s.success()),
    );
}
