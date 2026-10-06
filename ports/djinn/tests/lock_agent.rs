// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The vault lock's agent receipt (vault lock plan L2, rulings 8 and 9;
//! harness plan H4): a real djinn resident on an isolated pipe, driven by
//! the system's own OpenSSH tools. `ssh-add -x` locks the whole resident
//! and its status route says so; while locked nothing is listed and sign,
//! add and remove fail, and `ssh-add -X` is refused. `djinn --unlock`'s
//! route then refuses a wrong passphrase and takes the right one, after
//! which the same identities are listed and a signature verifies; locked
//! again, the resident stops gracefully (rulings 40, 41, 46).
//!
//! ```text
//! cargo test -p djinn --test lock_agent -- --ignored --nocapture
//! ```

#![cfg(windows)]

mod resident_harness;

use std::path::{Path, PathBuf};
use std::process::Output;

use djinn::resident_status::{self, RESIDENT_APP, RESIDENT_CONTROL_ROUTE};
use djinn_testkit::{Bound, verify};
use graphshell::native::app_admission::{AppId, AppRouteId};
use graphshell::native::app_client::AppBrokerClient;
use resident_harness::*;
use serde_json::json;

const PASSPHRASE: &str = "djinn-lock-receipt";
const NAMESPACE: &str = "lock-receipt";

/// The system's OpenSSH, by absolute path: never whatever `PATH` finds.
fn openssh(tool: &str) -> PathBuf {
    let root = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".into());
    Path::new(&root)
        .join("System32")
        .join("OpenSSH")
        .join(format!("{tool}.exe"))
}

/// An identity line as `ssh-add -L` prints it, minus what is minted per
/// listing: a certificate is fresh each time, so it is compared by type.
fn key_of(line: &str) -> String {
    let mut fields = line.split_whitespace();
    let kind = fields.next().unwrap_or_default();
    match kind.ends_with("-cert-v01@openssh.com") {
        true => format!("{kind} (certificate)"),
        false => format!("{kind} {}", fields.next().unwrap_or_default()),
    }
}

fn text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
#[ignore = "runs a real djinn resident and the system's OpenSSH tools"]
fn ssh_add_locks_the_resident_like_openssh_and_the_wire_unlock_is_refused() {
    let run = begin(
        "vault-lock",
        "ssh_add_locks_the_resident_like_openssh_and_the_wire_unlock_is_refused",
    );
    let dir = run.dir().to_path_buf();
    let (add, keygen) = (openssh("ssh-add"), openssh("ssh-keygen"));
    let mut r = resident(&run, "locked", "locked", PASSPHRASE);

    // A fresh key and a message, under the run.
    let keys = r.root().join("keys");
    std::fs::create_dir_all(&keys).unwrap();
    let key = keys.join("id_lock").display().to_string();
    let public = format!("{key}.pub");
    let message = keys.join("message.txt");
    std::fs::write(&message, b"signed through the djinn agent").unwrap();
    let made = r
        .ssh_client(
            &keygen,
            &["-q", "-t", "ed25519", "-N", "", "-C", "lock-receipt", "-f", &key],
            b"",
        )
        .unwrap();
    assert!(made.status.success(), "ssh-keygen: {}", text(&made));
    let public_line = std::fs::read_to_string(&public).unwrap();
    let public_key: String = public_line
        .split_whitespace()
        .take(2)
        .collect::<Vec<_>>()
        .join(" ");

    r.start(&[]).unwrap();
    let first = r.wait_ready(READY);
    run.require(
        "the status reports unlocked at start",
        Bound::State,
        json!({ "lock": "unlocked", "unlock": "passphrase" }),
        json!({ "lock": first.lock, "unlock": first.startup_unlock }),
        first.lock == "unlocked" && first.startup_unlock == "passphrase",
    );

    let sign = |r: &djinn_testkit::Resident| {
        let _ = std::fs::remove_file(keys.join("message.txt.sig"));
        r.ssh_client(
            &keygen,
            &[
                "-Y",
                "sign",
                "-f",
                &public,
                "-n",
                NAMESPACE,
                &message.display().to_string(),
            ],
            b"",
        )
        .unwrap()
    };

    // Unlocked: the key goes in through ssh-add, is listed, and signs.
    let added = r.ssh_client(&add, &[&key], b"").unwrap();
    run.require(
        "ssh-add adds the key while unlocked",
        Bound::State,
        "exit 0",
        text(&added),
        added.status.success(),
    );
    // The private key leaves the public key's side, so every sign below can
    // only come from the agent (ssh-keygen would read `id_lock` otherwise).
    let held = r.root().join("held");
    std::fs::create_dir_all(&held).unwrap();
    let key_path = held.join("id_lock");
    std::fs::rename(&key, &key_path).unwrap();
    let key = key_path.display().to_string();
    let listed = r.ssh_client(&add, &["-L"], b"").unwrap();
    let listed_text = String::from_utf8_lossy(&listed.stdout).into_owned();
    run.require(
        "ssh-add -L lists the certificate and the key",
        Bound::State,
        json!({ "lines": 2, "contains": public_key }),
        &listed_text,
        listed.status.success()
            && listed_text.lines().count() == 2
            && listed_text.contains(&public_key),
    );
    let signed = sign(&r);
    let checked = r
        .ssh_client(
            &keygen,
            &[
                "-Y",
                "check-novalidate",
                "-n",
                NAMESPACE,
                "-s",
                &keys.join("message.txt.sig").display().to_string(),
            ],
            &std::fs::read(&message).unwrap(),
        )
        .unwrap();
    run.require(
        "ssh-keygen -Y sign signs through the agent, and the signature checks",
        Bound::State,
        "exit 0, Good signature",
        format!("{} | {}", text(&signed), text(&checked)),
        signed.status.success()
            && checked.status.success()
            && text(&checked).contains("Good"),
    );

    // ssh-add -x: the lock password is asked twice, through an askpass
    // script (no console is attached), and never used.
    let askpass = |password: &str| {
        let script = keys.join(format!("askpass-{password}.bat"));
        std::fs::write(&script, format!("@echo {password}\r\n")).unwrap();
        script.display().to_string()
    };
    let unused = askpass("unused");
    let with_askpass = |script: &str, args: &[&str]| {
        r.ssh_client_env(
            &add,
            args,
            b"",
            &[("SSH_ASKPASS", script), ("SSH_ASKPASS_REQUIRE", "force")],
        )
        .unwrap()
    };
    let locked = with_askpass(&unused, &["-x"]);
    run.require(
        "ssh-add -x locks the agent",
        Bound::State,
        "exit 0, Agent locked",
        text(&locked),
        locked.status.success() && text(&locked).contains("locked"),
    );
    // H4's first condition: the status route, read through the door's kept
    // keys while the vault is locked (rulings 40, 46).
    let (status, waited) = run.wait_for("the status route reports locked", STOP, || {
        r.status().filter(|status| status.lock == "locked")
    });
    run.step(
        "status locked",
        json!({ "waited_ms": waited.as_millis() as u64, "lock": status.lock }),
    );

    // Locked: OpenSSH's behaviour.
    let empty = r.ssh_client(&add, &["-L"], b"").unwrap();
    run.require(
        "while locked ssh-add -L lists no identities",
        Bound::State,
        "exit 1, no identities",
        text(&empty),
        empty.status.code() == Some(1) && text(&empty).contains("no identities"),
    );
    let refused_sign = sign(&r);
    run.require(
        "while locked the sign fails",
        Bound::State,
        "nonzero exit",
        text(&refused_sign),
        !refused_sign.status.success() && !keys.join("message.txt.sig").exists(),
    );
    let refused_add = r.ssh_client(&add, &[&key], b"").unwrap();
    run.require(
        "while locked ssh-add of a key fails",
        Bound::State,
        "nonzero exit",
        text(&refused_add),
        !refused_add.status.success(),
    );
    let refused_remove = r.ssh_client(&add, &["-d", &public], b"").unwrap();
    run.require(
        "while locked ssh-add -d fails",
        Bound::State,
        "nonzero exit",
        text(&refused_remove),
        !refused_remove.status.success(),
    );
    let relock = with_askpass(&unused, &["-x"]);
    run.require(
        "a second ssh-add -x fails, as OpenSSH's does",
        Bound::State,
        "nonzero exit",
        text(&relock),
        !relock.status.success(),
    );

    // ssh-add -X is refused over the wire, whatever the password.
    for password in ["unused", PASSPHRASE] {
        let unlock = with_askpass(&askpass(password), &["-X"]);
        run.require(
            "ssh-add -X is refused",
            Bound::State,
            "nonzero exit, Failed to unlock",
            text(&unlock),
            !unlock.status.success() && text(&unlock).contains("Failed"),
        );
    }
    let still_empty = r.ssh_client(&add, &["-L"], b"").unwrap();
    let still = r.status().expect("the status route answers while locked");
    run.require(
        "after -X the resident is still locked and lists nothing",
        Bound::State,
        json!({ "lock": "locked", "listed": "no identities" }),
        json!({ "lock": still.lock, "listed": text(&still_empty) }),
        still.lock == "locked" && text(&still_empty).contains("no identities"),
    );

    // `djinn --unlock` (ruling 41): the passphrase on the owner-only control
    // route. The seam: this test calls `request_unlock`, which `--unlock`
    // calls once the terminal has the passphrase; only the read is skipped.
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let unlock = |passphrase: &[u8]| {
        runtime.block_on(async {
            let mut client = AppBrokerClient::open_route_at(
                &r.endpoint("app"),
                AppId::new(RESIDENT_APP),
                AppRouteId::new(RESIDENT_CONTROL_ROUTE).unwrap(),
            )
            .await
            .map_err(|error| error.to_string())?;
            let answer = resident_status::request_unlock(&mut client, passphrase).await;
            let _ = client.close().await;
            answer.map_err(|error| error.to_string())
        })
    };
    let wrong = unlock(b"not the passphrase");
    let after_wrong = r.status().expect("the status route answers");
    run.require(
        "djinn --unlock with a wrong passphrase leaves it locked",
        Bound::State,
        json!({ "refused": true, "lock": "locked" }),
        json!({ "answer": format!("{wrong:?}"), "lock": after_wrong.lock }),
        wrong.is_err() && after_wrong.lock == "locked",
    );
    let right = unlock(PASSPHRASE.as_bytes());
    let (opened, _) = run.wait_for("the status route reports unlocked", STOP, || {
        r.status().filter(|status| status.lock == "unlocked")
    });
    run.require(
        "djinn --unlock with the right passphrase unlocks",
        Bound::State,
        json!({ "answer": "Ok(())", "lock": "unlocked" }),
        json!({ "answer": format!("{right:?}"), "lock": opened.lock }),
        right.is_ok() && opened.lock == "unlocked",
    );
    let relisted = r.ssh_client(&add, &["-L"], b"").unwrap();
    let relisted_text = String::from_utf8_lossy(&relisted.stdout).into_owned();
    run.require(
        "after the unlock ssh-add -L lists the same identities",
        Bound::State,
        listed_text.lines().map(key_of).collect::<Vec<_>>(),
        relisted_text.lines().map(key_of).collect::<Vec<_>>(),
        relisted.status.success()
            && relisted_text.lines().map(key_of).collect::<Vec<_>>()
                == listed_text.lines().map(key_of).collect::<Vec<_>>(),
    );
    let resigned = sign(&r);
    let rechecked = r
        .ssh_client(
            &keygen,
            &[
                "-Y",
                "check-novalidate",
                "-n",
                NAMESPACE,
                "-s",
                &keys.join("message.txt.sig").display().to_string(),
            ],
            &std::fs::read(&message).unwrap(),
        )
        .unwrap();
    run.require(
        "after the unlock a signature through the agent verifies",
        Bound::State,
        "exit 0, Good signature",
        format!("{} | {}", text(&resigned), text(&rechecked)),
        resigned.status.success() && text(&rechecked).contains("Good"),
    );

    // Locked again, then the graceful stop (ruling 40: the stop route too).
    let relocked = with_askpass(&unused, &["-x"]);
    let (locked_again, _) = run.wait_for("the status route reports locked again", STOP, || {
        r.status().filter(|status| status.lock == "locked")
    });
    run.require(
        "ssh-add -x locks it again",
        Bound::State,
        json!({ "exit": 0, "lock": "locked" }),
        json!({ "exit": relocked.status.code(), "lock": locked_again.lock }),
        relocked.status.success() && locked_again.lock == "locked",
    );
    drop(runtime);
    r.stop(STOP);
    let events: Vec<String> = r.events().into_iter().map(|e| e.event).collect();
    let expected = [
        "started", "listening", "ready", "locked", "unlocked", "locked", "stopping", "stopped",
    ];
    run.require(
        "the event file runs the lock, the unlock, the relock and the stop",
        Bound::State,
        expected,
        &events,
        events == expected,
    );
    let receipt = run.finish();
    let checked = verify(&dir).unwrap();
    assert!(
        checked.ok() && checked.checked == receipt.evidence.len(),
        "{checked:?}"
    );
    println!(
        "RECEIPT {} verified {} evidence files",
        receipt.run, checked.checked
    );
}
