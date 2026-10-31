// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Synthetic private-workshop receiving harness. The custodian service runs in
//! a separate child process; this process owns only a RemoteHost and the app
//! broker. Consumers run in their own processes. This is a development fixture,
//! never an installed or signed custody host, and it opens no existing store.
//!
//! Usage: private_workshop_fixture <new-directory>
//! stdout emits redacted JSON readiness/control receipts. stdin accepts `lock`,
//! `unlock`, `status`, `lose-custodian`, and `quit`; EOF also stops the fixture.

#[cfg(unix)]
mod fixture {
    use std::{
        io::{BufRead, BufReader, Write},
        os::unix::fs::PermissionsExt,
        path::{Path, PathBuf},
        process::{Child, Command, Stdio},
        sync::{Arc, mpsc},
        time::{Duration, Instant},
    };

    use mere_resident::{
        custodian::InstalledCustodian,
        custody_host::{ResidentCustody, remote::RemoteHost},
        door::app_broker::{AppEndpointCatalog, serve_app_broker_on},
        keeper::Keeper,
    };
    use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use djinn::door::local_endpoint::LocalListener;
    use personae::{PersonaId, ProfileId};
    use serde_json::json;

    const PASSPHRASE: &str = "synthetic private workshop fixture only";
    const PROFILE: &str = "private-workshop-fixture";
    const PERSONA: u128 = 0xa1eb_1c00_2026_1029_0000_0000_0000_0001;
    const READY_TIMEOUT: Duration = Duration::from_secs(20);

    fn emit(value: serde_json::Value) -> Result<(), String> {
        let stdout = std::io::stdout();
        let mut out = stdout.lock();
        serde_json::to_writer(&mut out, &value).map_err(|e| e.to_string())?;
        out.write_all(b"\n").map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())
    }

    fn create_new_root(root: &Path) -> Result<(), String> {
        if !root.is_absolute() {
            return Err("fixture requires an absolute new-directory path".into());
        }
        // Refuse even an empty existing directory: selecting a real user's
        // store by mistake must not turn a fixture into a migration tool.
        std::fs::create_dir(root).map_err(|e| format!("fixture requires a new directory: {e}"))?;
        std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o700))
            .map_err(|e| e.to_string())
    }

    fn custodian_child(root: &Path) -> Result<(), String> {
        create_new_root(root)?;
        let custodian = djinn_custodian::fixture::start(
            root,
            djinn_custodian::fixture::Spec {
                passphrase: PASSPHRASE,
                profiles: &[(PROFILE, "Synthetic workshop", [0x91; 32])],
                grants: json!([
                    {"operation":"status"}, {"operation":"lock"}, {"operation":"unlock"},
                    {"operation":"snapshot"}, {"operation":"manage","consent":"none"},
                    {"operation":"wallet","consent":"none"},
                    {"operation":"payload","consent":"none"}
                ]),
                consent: "deny",
                mechanism: None,
                agent: false,
            },
        )?;
        castellan::custody::wallet::ensure_wallet_state(
            &custodian.data_root,
            PersonaId::from_uuid(uuid::Uuid::from_u128(PERSONA)),
            "Synthetic private workshop",
        )
        .map_err(|e| e.to_string())?;
        emit(json!({"ready":true,"role":"custodian","pid":std::process::id()}))?;
        let mut input = String::new();
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        loop {
            input.clear();
            if stdin.read_line(&mut input).map_err(|e| e.to_string())? == 0 {
                return Ok(());
            }
            if input.trim() == "quit" {
                return Ok(());
            }
        }
    }

    struct CustodianChild(Option<Child>);
    impl CustodianChild {
        fn start(root: &Path) -> Result<Self, String> {
            let child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
                .arg("--custodian-child")
                .arg(root)
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .map_err(|e| e.to_string())?;
            let mut owned = Self(Some(child));
            let stdout = owned
                .0
                .as_mut()
                .expect("owned child")
                .stdout
                .take()
                .expect("piped stdout");
            let (send, receive) = mpsc::channel();
            let reader = std::thread::spawn(move || {
                let mut line = String::new();
                let result = BufReader::new(stdout).read_line(&mut line).map(|_| line);
                let _ = send.send(result);
            });
            let result = receive.recv_timeout(READY_TIMEOUT);
            if result.is_err() {
                owned.stop();
                let _ = reader.join();
                return Err("custodian child did not become ready".into());
            }
            let line = result
                .unwrap()
                .map_err(|_| "custodian readiness read failed")?;
            let _ = reader.join();
            let ready: serde_json::Value =
                serde_json::from_str(&line).map_err(|_| "invalid custodian readiness")?;
            if ready["ready"] != true || ready["role"] != "custodian" {
                return Err("custodian child refused readiness".into());
            }
            Ok(owned)
        }
        fn pid(&self) -> Option<u32> {
            self.0.as_ref().map(Child::id)
        }
        fn stop(&mut self) {
            let Some(mut child) = self.0.take() else {
                return;
            };
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(b"quit\n");
            }
            let deadline = Instant::now() + Duration::from_secs(3);
            loop {
                if !matches!(child.try_wait(), Ok(None)) {
                    break;
                }
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    break;
                }
                std::thread::sleep(Duration::from_millis(20));
            }
            let _ = child.wait();
        }
    }
    impl Drop for CustodianChild {
        fn drop(&mut self) {
            self.stop();
        }
    }

    fn broker(root: &Path) -> Result<(), String> {
        create_new_root(root)?;
        let child_root = root.join("custodian");
        let mut child = CustodianChild::start(&child_root)?;
        let remote = RemoteHost::connect(InstalledCustodian::from_path(
            &child_root.join("client.json"),
            false,
        )?)?;
        remote.unlock_with_passphrase(PASSPHRASE.as_bytes())?;
        let keeper = Arc::new(Keeper::new(remote.clone()));
        keeper.configure_payload_custody(
            ProfileId(PROFILE.into()),
            PersonaId::from_uuid(uuid::Uuid::from_u128(PERSONA)),
            child_root.join("data"),
        )?;
        let grants = AppRouteGrants::new(AllowedAppRoutes::none());
        mere_resident::custody::grant(&grants).map_err(|e| e.to_string())?;
        let endpoint = root
            .join("app.sock")
            .to_str()
            .ok_or("non-UTF-8 fixture root")?
            .to_owned();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .map_err(|e| e.to_string())?;
        let listener = runtime
            .block_on(LocalListener::bind(&endpoint, "private-workshop-fixture"))
            .map_err(|e| e.to_string())?;
        let server = runtime.spawn(serve_app_broker_on(
            listener,
            keeper,
            grants,
            60_000,
            None,
            AppEndpointCatalog::default(),
        ));
        emit(
            json!({"ready":true,"role":"broker","pid":std::process::id(),
            "custodian_pid":child.pid(),"endpoint":endpoint,"profile":PROFILE,
            "synthetic":true,"production_qualified":false}),
        )?;
        let stdin = std::io::stdin();
        let mut stdin = stdin.lock();
        let mut input = String::new();
        loop {
            input.clear();
            if stdin.read_line(&mut input).map_err(|e| e.to_string())? == 0 {
                break;
            }
            let command = input.trim();
            if command == "quit" {
                break;
            }
            let result = match command {
                "lock" => remote.lock_vault(),
                "unlock" => remote.unlock_with_passphrase(PASSPHRASE.as_bytes()),
                "status" => Ok(()),
                "lose-custodian" => {
                    child.stop();
                    Ok(())
                },
                _ => Err("unknown fixture command".into()),
            };
            let epoch = remote.custody_epoch().ok().flatten();
            let reported_command = match command {
                "lock" | "unlock" | "status" | "lose-custodian" => command,
                _ => "unknown",
            };
            emit(
                json!({"command":reported_command,"ok":result.is_ok(),"locked":remote.is_locked(),
                "custody_epoch":epoch,"custodian_pid":child.pid()}),
            )?;
        }
        server.abort();
        runtime.block_on(async {
            let _ = server.await;
        });
        child.stop();
        Ok(())
    }

    pub fn main() -> Result<(), String> {
        let mut args = std::env::args_os().skip(1);
        let first = args
            .next()
            .ok_or("usage: private_workshop_fixture <new-directory>")?;
        if first == "--custodian-child" {
            let root = PathBuf::from(args.next().ok_or("missing child directory")?);
            if args.next().is_some() {
                return Err("unexpected child arguments".into());
            }
            custodian_child(&root)
        } else {
            if args.next().is_some() {
                return Err("unexpected fixture arguments".into());
            }
            broker(&PathBuf::from(first))
        }
    }
}

#[cfg(unix)]
fn main() -> Result<(), String> {
    fixture::main()
}
#[cfg(not(unix))]
fn main() -> Result<(), String> {
    Err("this synthetic receiving fixture requires Unix sockets".into())
}
