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
//! Ignored by default: it runs two real residents for a few minutes.
//! Everything it starts lives under `DJINN_D1B_ROOT` (a fresh temporary
//! directory when unset), on endpoint names unique to the run.
//!
//! ```text
//! DJINN_D1B_ROOT=C:\t\pairing-d1b\run cargo test -p djinn --test mdns_first_contact_two_instance -- --ignored --nocapture
//! ```

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

use djinn::resident_devices::{AddrKindV1, DeviceDirectoryV1, PairedDeviceV1};
use djinn::settings::OwnerSettings;
use personae::ProfileId;
use transport::PeerAddr;

const DJINN: &str = env!("CARGO_BIN_EXE_djinn");
const DEVICES: &str = env!("CARGO_BIN_EXE_djinn-devices");
const PASSPHRASE: &str = "djinn-mdns-first-contact-receipt";
const GRAPH: &str = "pairing-d1b-receipt";
/// Long enough for several pairing polls and mDNS query rounds; D1 saw no
/// contact in 45 s, and a ticket connects in under 4 s.
const PATIENCE: Duration = Duration::from_secs(90);

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
        format!("d1b{}", self.name)
    }

    fn endpoint(&self, door: &str) -> String {
        #[cfg(windows)]
        return format!(r"\\.\pipe\djinn-d1b-{}-{}-{door}", self.run, self.name);
        #[cfg(not(windows))]
        return self.root.join(format!("{door}.sock")).display().to_string();
    }

    fn local(&self) -> PathBuf {
        self.root.join("local")
    }

    fn settings_file(&self) -> PathBuf {
        let app_dir = self.local().join(if cfg!(windows) {
            "Graphshell"
        } else {
            "graphshell"
        });
        djinn::settings::settings_path(&app_dir, &ProfileId(self.profile()))
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
        let output = self
            .command(DJINN)
            .arg("--dir")
            .arg(self.root.join("vault"))
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
        let path = self.settings_file();
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

    /// Remove every saved dial hint, so the next start has only mDNS.
    fn forget_hints(&self) -> usize {
        let path = self.settings_file();
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

    fn log(&self) -> PathBuf {
        self.root.join(format!("djinn-{}.log", self.starts))
    }

    fn start(&mut self) -> Instant {
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

    /// The ports this start listens on, from the ticket its resident logs.
    fn listening_ports(&self) -> BTreeSet<u16> {
        let (ticket, _) = wait_for(&format!("{} listening", self.name), PATIENCE, || {
            let log = std::fs::read_to_string(self.log()).ok()?;
            log.lines()
                .filter(|line| line.contains("personal graph sync listening"))
                .find_map(|line| {
                    line.split_whitespace()
                        .find_map(|t| t.strip_prefix("ticket="))
                })
                .map(str::to_string)
        });
        let (_, addrs) = transport::decode_peer_ticket(&ticket).unwrap();
        addrs
            .iter()
            .filter_map(|addr| match addr {
                PeerAddr::Direct(addr) => Some(addr.port()),
                _ => None,
            })
            .collect()
    }

    /// The directory as `djinn-devices --json` prints it.
    fn directory(&self) -> Option<DeviceDirectoryV1> {
        let output = self.command(DEVICES).arg("--json").output().ok()?;
        output
            .status
            .success()
            .then(|| serde_json::from_slice(&output.stdout).expect("djinn-devices prints JSON"))
    }

    fn sees(&self, node: &str) -> Option<PairedDeviceV1> {
        self.directory()?
            .devices
            .into_iter()
            .find(|device| device.node_id == node)
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
        std::thread::sleep(Duration::from_millis(250));
    }
}

fn active_direct_ports(device: &PairedDeviceV1) -> BTreeSet<u16> {
    device
        .path
        .iter()
        .filter(|addr| addr.active && addr.kind == AddrKindV1::Direct)
        .filter_map(|addr| addr.addr.parse::<std::net::SocketAddr>().ok())
        .map(|addr| addr.port())
        .collect()
}

/// Contact after `fresh` (re)started: the fresh process sees its peer
/// connected, and the other side's live path is one of the fresh process's
/// own ports, so a path left over from an earlier process cannot pass.
fn contact(
    label: &str,
    (fresh, fresh_node, fresh_ports, spawned): (&Instance, &str, &BTreeSet<u16>, Instant),
    (other, other_node): (&Instance, &str),
) {
    let watching = Instant::now();
    let (_, took) = wait_for(&format!("{label}: contact"), PATIENCE, || {
        let mine = fresh.sees(other_node)?.connected;
        let theirs = other.sees(fresh_node)?;
        let on_new_path = !active_direct_ports(&theirs).is_disjoint(fresh_ports);
        (mine && theirs.connected && on_new_path).then_some(())
    });
    println!(
        "RECEIPT {label}: both connected {:?} after {} was spawned \
         (watched from {:?} after spawn, for {took:?}); {} listens on {fresh_ports:?}",
        spawned.elapsed(),
        fresh.name,
        watching.duration_since(spawned),
        fresh.name
    );
}

/// Both sides hold a saved hint for the other, so removing them is a change.
fn wait_for_hints(a: &Instance, a_node: &str, b: &Instance, b_node: &str) {
    let (_, took) = wait_for("hints saved on both sides", PATIENCE, || {
        let on_a = a.sees(b_node)?.hint.is_some();
        let on_b = b.sees(a_node)?.hint.is_some();
        (on_a && on_b).then_some(())
    });
    println!("RECEIPT hints saved on both sides after {took:?}");
}

/// Stop `side`, remove its saved hints, and start it again.
fn restart_without_hints(side: &mut Instance) -> (BTreeSet<u16>, Instant) {
    side.stop();
    let forgotten = side.forget_hints();
    assert_eq!(forgotten, 1, "{} held one saved hint", side.name);
    let spawned = side.start();
    let ports = side.listening_ports();
    println!(
        "RECEIPT {} restarted without its hint, listening {:?} after spawn",
        side.name,
        spawned.elapsed()
    );
    (ports, spawned)
}

#[test]
#[ignore = "runs two real djinn residents; see the module note"]
fn two_paired_residents_meet_by_mdns_alone_and_again_after_restarts() {
    let scratch = tempfile::tempdir().unwrap();
    let base = std::env::var_os("DJINN_D1B_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| scratch.path().to_path_buf());
    let run = uuid::Uuid::new_v4().simple().to_string()[..8].to_string();
    let base = base.join(&run);
    println!("RECEIPT root {}", base.display());
    let mut a = Instance::new(&base, &run, "a");
    let mut b = Instance::new(&base, &run, "b");

    // Pair with the ordinary commands, both ways, and no ticket anywhere.
    a.configure();
    b.configure();
    let (a_node, a_root) = a.facts();
    let (b_node, b_root) = b.facts();
    a.pair(&b_node, &b_root, "d1b-b");
    b.pair(&a_node, &a_root, "d1b-a");
    println!("RECEIPT a {a_node}\nRECEIPT b {b_node}");

    // First contact: no ticket, no hint, no `--sync-peer`.
    let a_spawned = a.start();
    let a_ports = a.listening_ports();
    println!("RECEIPT a listening {:?} after spawn", a_spawned.elapsed());
    let b_spawned = b.start();
    let b_ports = b.listening_ports();
    println!("RECEIPT b listening {:?} after spawn", b_spawned.elapsed());
    contact(
        "first contact",
        (&b, &b_node, &b_ports, b_spawned),
        (&a, &a_node),
    );

    // Each restart removes the restarted side's saved hint first, so mDNS
    // stays its only way back: first contact again, not the cached-hint rung.
    wait_for_hints(&a, &a_node, &b, &b_node);
    let (b_ports, b_spawned) = restart_without_hints(&mut b);
    contact(
        "b restarted",
        (&b, &b_node, &b_ports, b_spawned),
        (&a, &a_node),
    );

    wait_for_hints(&a, &a_node, &b, &b_node);
    let (a_ports_now, a_spawned) = restart_without_hints(&mut a);
    assert!(
        a_ports_now.is_disjoint(&a_ports),
        "a came back on new ports"
    );
    contact(
        "a restarted",
        (&a, &a_node, &a_ports_now, a_spawned),
        (&b, &b_node),
    );

    a.stop();
    b.stop();
}
