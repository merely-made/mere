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
//! minute or two. Everything it starts lives under `DJINN_D1_ROOT` (a fresh
//! temporary directory when unset), on endpoint names unique to the run.
//!
//! ```text
//! DJINN_D1_ROOT=C:\t\pairing-d1\run cargo test -p djinn --test device_directory_two_instance -- --ignored --nocapture
//! ```

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use djinn::resident_devices::{AddrKindV1, DeviceDirectoryV1, PairedDeviceV1};
use personae::ProfileId;
use transport::PeerAddr;

const DJINN: &str = env!("CARGO_BIN_EXE_djinn");
const DEVICES: &str = env!("CARGO_BIN_EXE_djinn-devices");
const PASSPHRASE: &str = "djinn-device-directory-receipt";
const GRAPH: &str = "pairing-d1-receipt";
/// The resident's pairing poll (`personal_sync::PAIRING_POLL`).
const POLL: Duration = Duration::from_secs(5);
/// One CLI round trip on top of a poll.
const SLACK: Duration = Duration::from_secs(2);
const PATIENCE: Duration = Duration::from_secs(120);

/// One resident under the run's root: its own vault, settings, data and doors.
struct Instance {
    name: &'static str,
    root: PathBuf,
    run: String,
    starts: u32,
    child: Option<Child>,
}

impl Instance {
    fn new(base: &Path, run: &str, name: &'static str) -> Self {
        let root = base.join(name);
        std::fs::create_dir_all(&root).unwrap();
        Self {
            name,
            root,
            run: run.to_string(),
            starts: 0,
            child: None,
        }
    }

    fn profile(&self) -> String {
        format!("d1{}", self.name)
    }

    fn endpoint(&self, door: &str) -> String {
        #[cfg(windows)]
        return format!(r"\\.\pipe\djinn-d1-{}-{}-{door}", self.run, self.name);
        #[cfg(not(windows))]
        return self.root.join(format!("{door}.sock")).display().to_string();
    }

    fn local(&self) -> PathBuf {
        self.root.join("local")
    }

    /// Every location the binaries would otherwise take from the real user
    /// profile points under this instance's root.
    fn command(&self, program: &str) -> Command {
        let mut command = Command::new(program);
        command
            .env("PERSONAE_PASSPHRASE", PASSPHRASE)
            .env("LOCALAPPDATA", self.local())
            .env("APPDATA", self.root.join("roaming"))
            .env("XDG_DATA_HOME", self.local())
            .env("MERE_ROOT", self.root.join("mere"))
            .env("GRAPHSHELL_APP_ENDPOINT", self.endpoint("app"))
            .env_remove("DJINN_DATA_ROOT")
            .env_remove("GRAPHSHELL_PROFILE")
            .env_remove("SSH_AUTH_SOCK");
        command
    }

    fn djinn(&self, args: &[&str]) -> String {
        let vault = self.root.join("vault");
        let output = self
            .command(DJINN)
            .arg("--dir")
            .arg(&vault)
            .args(["--profile", &self.profile()])
            .args(args)
            .output()
            .unwrap();
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        assert!(
            output.status.success(),
            "djinn {args:?} on {}: {}\n{stdout}",
            self.name,
            String::from_utf8_lossy(&output.stderr)
        );
        stdout
    }

    /// The owner's statement: personal sync on one named graph, reservoir off.
    fn configure(&self) {
        let app_dir = self.local().join(if cfg!(windows) {
            "Graphshell"
        } else {
            "graphshell"
        });
        let path = djinn::settings::settings_path(&app_dir, &ProfileId(self.profile()));
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(
            &path,
            format!(r#"{{"sync":{{"graph":"{GRAPH}"}},"reservoir":{{"enabled":false}}}}"#),
        )
        .unwrap();
    }

    /// `(node_id, root)` from `--pairing-facts`.
    fn facts(&self) -> (String, String) {
        let facts = self.djinn(&["--pairing-facts"]);
        let field = |name: &str| {
            facts
                .lines()
                .find_map(|line| line.strip_prefix(name))
                .map(|value| value.trim().to_string())
                .unwrap_or_else(|| panic!("no {name} in {facts}"))
        };
        (field("node_id "), field("root "))
    }

    fn pair(&self, node: &str, root: &str, label: &str) {
        let said = self.djinn(&[
            "--pair-node",
            node,
            "--pair-root",
            root,
            "--pair-label",
            label,
        ]);
        assert!(said.starts_with("paired "), "{said}");
    }

    fn log(&self) -> PathBuf {
        self.root.join(format!("djinn-{}.log", self.starts))
    }

    fn start(&mut self, extra: &[&str]) -> Instant {
        self.starts += 1;
        let stderr =
            std::fs::File::create(self.root.join(format!("stderr-{}.txt", self.starts))).unwrap();
        let child = self
            .command(DJINN)
            .arg("--dir")
            .arg(self.root.join("vault"))
            .args(["--profile", &self.profile()])
            // The isolated receipt agent: the standard listener only ever
            // binds the OpenSSH pipe, which belongs to the installed resident.
            .args(["--receipt-agent-endpoint", &self.endpoint("agent")])
            .args(["--browser-endpoint", &self.endpoint("browser")])
            .args(["--app-endpoint", &self.endpoint("app")])
            .arg("--log-file")
            .arg(self.log())
            .args(extra)
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()
            .unwrap();
        self.child = Some(child);
        Instant::now()
    }

    fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }

    /// The directory as `djinn-devices --json` prints it.
    fn directory(&self) -> Option<DeviceDirectoryV1> {
        let output = self.command(DEVICES).arg("--json").output().ok()?;
        output
            .status
            .success()
            .then(|| serde_json::from_slice(&output.stdout).expect("djinn-devices prints JSON"))
    }

    fn human(&self) -> String {
        let output = self.command(DEVICES).output().unwrap();
        String::from_utf8_lossy(&output.stdout).into_owned()
    }

    /// This start's own ticket, as its resident logged it.
    fn listening_ticket(&self) -> String {
        let (ticket, _) = wait_for("the listening ticket", PATIENCE, || {
            let log = std::fs::read_to_string(self.log()).ok()?;
            log.lines()
                .filter(|line| line.contains("personal graph sync listening"))
                .find_map(|line| {
                    line.split_whitespace()
                        .find_map(|t| t.strip_prefix("ticket="))
                })
                .map(str::to_string)
        });
        ticket
    }

    /// The ports this start listens on, from its own logged ticket.
    fn listening_ports(&self) -> BTreeSet<u16> {
        let (_, addrs) = transport::decode_peer_ticket(&self.listening_ticket()).unwrap();
        addrs
            .iter()
            .filter_map(|addr| match addr {
                PeerAddr::Direct(addr) => Some(addr.port()),
                _ => None,
            })
            .collect()
    }
}

impl Drop for Instance {
    fn drop(&mut self) {
        self.stop();
    }
}

fn wait_for<T>(
    what: &str,
    patience: Duration,
    mut probe: impl FnMut() -> Option<T>,
) -> (T, Duration) {
    let started = Instant::now();
    loop {
        if let Some(found) = probe() {
            return (found, started.elapsed());
        }
        assert!(
            started.elapsed() < patience,
            "no {what} within {patience:?}"
        );
        std::thread::sleep(Duration::from_millis(200));
    }
}

fn entry<'a>(directory: &'a DeviceDirectoryV1, node: &str) -> Option<&'a PairedDeviceV1> {
    directory
        .devices
        .iter()
        .find(|device| device.node_id == node)
}

fn port_of(addr: &str) -> Option<u16> {
    addr.parse::<std::net::SocketAddr>()
        .ok()
        .map(|addr| addr.port())
}

fn active_direct_ports(device: &PairedDeviceV1) -> BTreeSet<u16> {
    device
        .path
        .iter()
        .filter(|addr| addr.active && addr.kind == AddrKindV1::Direct)
        .filter_map(|addr| port_of(&addr.addr))
        .collect()
}

fn hint_ports(device: &PairedDeviceV1) -> BTreeSet<u16> {
    device
        .hint
        .iter()
        .flat_map(|hint| &hint.addrs)
        .filter(|addr| addr.kind == AddrKindV1::Direct)
        .filter_map(|addr| port_of(&addr.addr))
        .collect()
}

#[test]
#[ignore = "runs two real djinn residents; see the module note"]
fn two_paired_residents_report_each_other_through_the_directory() {
    let scratch = tempfile::tempdir().unwrap();
    let base = std::env::var_os("DJINN_D1_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| scratch.path().to_path_buf());
    let run = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let base = base.join(&run);
    println!("RECEIPT root {}", base.display());
    let mut a = Instance::new(&base, &run, "a");
    let mut b = Instance::new(&base, &run, "b");

    // Pair with the ordinary commands, both ways.
    a.configure();
    b.configure();
    let (a_node, a_root) = a.facts();
    let (b_node, b_root) = b.facts();
    a.pair(&b_node, &b_root, "d1-b");
    b.pair(&a_node, &a_root, "d1-a");
    println!("RECEIPT a {a_node}\nRECEIPT b {b_node}");

    // The first contact needs an address to dial. Local discovery resolves
    // each peer's address on this machine but nothing dials on it (see the
    // D1 report), so b's first start carries a's ticket, the existing
    // bootstrap. Every later start dials from the saved hints alone.
    a.start(&[]);
    let a_ticket = a.listening_ticket();
    let b_started = b.start(&["--sync-peer", &a_ticket]);
    let b_ports = b.listening_ports();
    let (first, took) = wait_for("b connected in a's directory", PATIENCE, || {
        let directory = a.directory()?;
        let device = entry(&directory, &b_node)?;
        (device.connected && !active_direct_ports(device).is_empty()).then(|| device.clone())
    });
    println!(
        "RECEIPT connected after {:?} (b started {:?} ago); b listens on {b_ports:?}",
        took,
        b_started.elapsed()
    );
    assert_eq!(first.label, "d1-b");
    assert_eq!(first.root.as_deref(), Some(b_root.as_str()));
    assert!(first.pairing_id.is_some() && first.added_ms > 0);
    assert!(
        !active_direct_ports(&first).is_disjoint(&b_ports),
        "a's live path to b is one of b's endpoints: {first:?} vs {b_ports:?}"
    );
    let (_, took) = wait_for("a connected in b's directory", PATIENCE, || {
        let directory = b.directory()?;
        entry(&directory, &a_node)?.connected.then_some(())
    });
    println!("RECEIPT b sees a connected after a further {took:?}");
    println!("RECEIPT human output from a:\n{}", a.human());
    // The route agrees with the resident's own peer directory, which logs at
    // its next poll.
    let (_, took) = wait_for("a's resident logging b connected", POLL + SLACK, || {
        let logged = std::fs::read_to_string(a.log()).ok()?;
        logged
            .contains(&format!("\"{b_node}\", true, true"))
            .then_some(())
    });
    println!("RECEIPT a's resident logged b connected {took:?} later");

    // Restart b on a new endpoint.
    // Both sides hold a saved hint for the other before b goes away.
    wait_for("hints saved on both sides", PATIENCE, || {
        let on_a = !hint_ports(entry(&a.directory()?, &b_node)?).is_empty();
        let on_b = !hint_ports(entry(&b.directory()?, &a_node)?).is_empty();
        (on_a && on_b).then_some(())
    });
    b.stop();
    let restarted = b.start(&[]);
    let new_ports = b.listening_ports();
    let listening = restarted.elapsed();
    assert!(
        new_ports.is_disjoint(&b_ports),
        "b came back on new ports: {b_ports:?} then {new_ports:?}"
    );
    let (moved, path_took) = wait_for("b's new path in a's directory", PATIENCE, || {
        let directory = a.directory()?;
        let device = entry(&directory, &b_node)?;
        (device.connected && !active_direct_ports(device).is_disjoint(&new_ports))
            .then(|| device.clone())
    });
    let (hinted, hint_took) = wait_for("b's new hint in a's directory", PATIENCE, || {
        let directory = a.directory()?;
        let device = entry(&directory, &b_node)?;
        (!hint_ports(device).is_disjoint(&new_ports)).then(|| device.clone())
    });
    println!(
        "RECEIPT restart: b listening {listening:?} after spawn on {new_ports:?}; \
         new path {path_took:?} later; saved hint a further {hint_took:?}"
    );
    println!(
        "RECEIPT moved path {:?}\nRECEIPT moved hint {:?}",
        moved.path, hinted.hint
    );
    assert!(
        hint_took <= POLL + SLACK,
        "the saved hint follows the live path within one poll ({hint_took:?})"
    );

    // The negative control: a stopped peer reads as not connected.
    b.stop();
    let (stopped, took) = wait_for("b disconnected in a's directory", PATIENCE, || {
        let directory = a.directory()?;
        let device = entry(&directory, &b_node)?;
        (!device.connected).then(|| device.clone())
    });
    println!("RECEIPT stopped: not connected after {took:?}; {stopped:?}");
    println!("RECEIPT human output from a:\n{}", a.human());
    a.stop();
}
