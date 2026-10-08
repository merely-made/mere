// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

#![cfg(all(target_os = "linux", feature = "secret-service"))]

use std::collections::{HashMap, VecDeque};
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use castellan::resident::CastellanResident;
use castellan::secret_service::{
    SecretServiceAccessPolicy, SecretServiceCaller, SecretServiceLimits, SecretServiceOperation,
    SecretServiceVault, serve,
};
use futures_util::StreamExt;
use personae::PersonaId;
use tempfile::tempdir;
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};

const RECORD_KEY: [u8; 32] = [0x91; 32];
const FRESHNESS_KEY: [u8; 32] = [0x92; 32];

/// The resident's vault as the receipt drives it: the castellan resident
/// locked and unlocked with its own keys, the prompt answered from a script
/// (`true` unlocks, `false` cancels) and counted.
struct TestVault {
    resident: Arc<CastellanResident>,
    lock: tokio::sync::watch::Sender<bool>,
    answers: Mutex<VecDeque<bool>>,
    prompts: AtomicUsize,
}

impl TestVault {
    fn new(resident: Arc<CastellanResident>) -> Arc<Self> {
        Arc::new(Self {
            lock: tokio::sync::watch::Sender::new(resident.is_locked()),
            resident,
            answers: Mutex::new(VecDeque::new()),
            prompts: AtomicUsize::new(0),
        })
    }

    fn answer(&self, unlock: bool) {
        self.answers.lock().unwrap().push_back(unlock);
    }

    fn prompts(&self) -> usize {
        self.prompts.load(Ordering::SeqCst)
    }
}

impl SecretServiceVault for TestVault {
    fn is_locked(&self) -> bool {
        self.resident.is_locked()
    }

    fn watch(&self) -> tokio::sync::watch::Receiver<bool> {
        self.lock.subscribe()
    }

    fn lock(&self) -> Result<(), String> {
        self.resident.lock();
        self.lock.send_replace(true);
        Ok(())
    }

    fn prompt_unlock(&self) -> Result<bool, String> {
        self.prompts.fetch_add(1, Ordering::SeqCst);
        if self.answers.lock().unwrap().pop_front() != Some(true) {
            return Ok(false);
        }
        self.resident
            .unlock(RECORD_KEY, FRESHNESS_KEY)
            .map_err(|error| error.to_string())?;
        self.lock.send_replace(false);
        Ok(true)
    }
}

fn claim(dir: &std::path::Path) -> Arc<CastellanResident> {
    Arc::new(
        CastellanResident::claim(
            dir.join("records"),
            RECORD_KEY,
            dir.join("freshness"),
            FRESHNESS_KEY,
        )
        .unwrap(),
    )
}

fn executable_on_path(name: &str) -> PathBuf {
    let path = std::env::var_os("PATH").unwrap_or_else(|| OsString::from(""));
    std::env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .and_then(|candidate| std::fs::canonicalize(candidate).ok())
        .unwrap_or_else(|| panic!("{name} must be installed for this interoperability receipt"))
}

fn secret_tool(arguments: &[&str], secret_input: Option<&str>) -> std::process::Output {
    let mut child = Command::new("secret-tool")
        .args(arguments)
        .stdin(if secret_input.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("launch secret-tool");
    if let Some(secret) = secret_input {
        child
            .stdin
            .take()
            .unwrap()
            .write_all(secret.as_bytes())
            .expect("write secret-tool input");
    }
    let output = child.wait_with_output().expect("wait for secret-tool");
    assert!(
        output.status.success(),
        "secret-tool {:?} failed: {}",
        arguments,
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// A real libsecret client exercises create, search, release, and deletion.
///
/// Run inside a disposable session bus so the receipt cannot replace the
/// desktop keyring:
/// `dbus-run-session -- cargo test -p castellan --features secret-service \
///   --test secret_service_linux -- --ignored --nocapture`
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires Linux, dbus-run-session, and secret-tool"]
async fn secret_tool_store_lookup_and_clear() {
    let dir = tempdir().unwrap();
    let resident = claim(dir.path());
    let store = resident.secret_service(PersonaId::new(), SecretServiceLimits::default());
    let vault = TestVault::new(Arc::clone(&resident));

    let admitted_executable = executable_on_path("secret-tool");
    let policy: Arc<dyn SecretServiceAccessPolicy> = Arc::new(
        move |caller: &SecretServiceCaller, _operation: &SecretServiceOperation| {
            caller.executable.as_deref() == Some(admitted_executable.as_path())
        },
    );
    let _server = serve(store, policy, "Castellan test collection", vault)
        .await
        .unwrap();

    let untrusted = zbus::Connection::session().await.unwrap();
    let proxy = zbus::Proxy::new(
        &untrusted,
        "org.freedesktop.secrets",
        "/org/freedesktop/secrets",
        "org.freedesktop.Secret.Service",
    )
    .await
    .unwrap();
    let denied = proxy
        .get_property::<Vec<zbus::zvariant::OwnedObjectPath>>("Collections")
        .await
        .unwrap_err();
    assert!(denied.to_string().contains("AccessDenied"));

    secret_tool(
        &[
            "store",
            "--label=Castellan interoperability receipt",
            "application",
            "turnstone",
            "account",
            "mark",
        ],
        Some("swordfish"),
    );
    let lookup = secret_tool(
        &["lookup", "application", "turnstone", "account", "mark"],
        None,
    );
    assert_eq!(
        String::from_utf8(lookup.stdout).unwrap().trim(),
        "swordfish"
    );

    secret_tool(
        &["clear", "application", "turnstone", "account", "mark"],
        None,
    );
    let lookup = Command::new("secret-tool")
        .args(["lookup", "application", "turnstone", "account", "mark"])
        .output()
        .expect("launch final secret-tool lookup");
    assert!(!lookup.status.success());
    assert!(lookup.stdout.is_empty());
}

/// A proxy that asks the service on every property read: a caching proxy
/// would answer from the very `PropertiesChanged` this receipt checks.
async fn uncached(
    client: &zbus::Connection,
    path: OwnedObjectPath,
    interface: &'static str,
) -> zbus::Proxy<'static> {
    zbus::proxy::Builder::<zbus::Proxy<'static>>::new(client)
        .destination("org.freedesktop.secrets")
        .unwrap()
        .path(path)
        .unwrap()
        .interface(interface)
        .unwrap()
        .cache_properties(zbus::proxy::CacheProperties::No)
        .build()
        .await
        .unwrap()
}

fn lookup() -> std::process::Output {
    Command::new("secret-tool")
        .args(["lookup", "application", "turnstone", "account", "mark"])
        .stdin(Stdio::null())
        .output()
        .expect("launch secret-tool lookup")
}

/// Vault lock rulings 10, 67 and 68 against a real libsecret client: a locked
/// vault reports every object locked, refuses secrets, answers searches from
/// the snapshot, and `Unlock` returns a Prompt that runs the resident's own
/// unlock. Run as the receipt above, inside `dbus-run-session`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "requires Linux, dbus-run-session, and secret-tool"]
async fn a_locked_vault_refuses_and_its_unlock_prompts() {
    let dir = tempdir().unwrap();
    let resident = claim(dir.path());
    let store = resident.secret_service(PersonaId::new(), SecretServiceLimits::default());
    let vault = TestVault::new(Arc::clone(&resident));
    let admitted = [
        executable_on_path("secret-tool"),
        std::fs::canonicalize(std::env::current_exe().unwrap()).unwrap(),
    ];
    let policy: Arc<dyn SecretServiceAccessPolicy> = Arc::new(
        move |caller: &SecretServiceCaller, _operation: &SecretServiceOperation| {
            caller
                .executable
                .as_deref()
                .is_some_and(|executable| admitted.iter().any(|path| path == executable))
        },
    );
    let _server = serve(
        store,
        policy,
        "Castellan test collection",
        Arc::clone(&vault) as Arc<dyn SecretServiceVault>,
    )
    .await
    .unwrap();
    secret_tool(
        &[
            "store",
            "--label=Castellan lock receipt",
            "application",
            "turnstone",
            "account",
            "mark",
        ],
        Some("swordfish"),
    );

    let client = zbus::Connection::session().await.unwrap();
    let service = uncached(
        &client,
        OwnedObjectPath::try_from("/org/freedesktop/secrets").unwrap(),
        "org.freedesktop.Secret.Service",
    )
    .await;
    let default: OwnedObjectPath = service.call("ReadAlias", &("default",)).await.unwrap();
    let collection = uncached(&client, default.clone(), "org.freedesktop.Secret.Collection").await;
    let attributes = HashMap::from([("application", "turnstone"), ("account", "mark")]);
    let (found, _): (Vec<OwnedObjectPath>, Vec<OwnedObjectPath>) =
        service.call("SearchItems", &(&attributes,)).await.unwrap();
    assert_eq!(found.len(), 1, "unlocked: the item is found unlocked");
    let item_path = found[0].clone();
    let item = uncached(&client, item_path.clone(), "org.freedesktop.Secret.Item").await;
    let (_, session): (OwnedValue, OwnedObjectPath) = service
        .call("OpenSession", &("plain", Value::from("")))
        .await
        .unwrap();
    assert!(!collection.get_property::<bool>("Locked").await.unwrap());

    // A client's Lock locks the whole vault (ruling 68), and says so.
    let properties = zbus::fdo::PropertiesProxy::builder(&client)
        .destination("org.freedesktop.secrets")
        .unwrap()
        .path(default.clone())
        .unwrap()
        .build()
        .await
        .unwrap();
    let mut changes = properties.receive_properties_changed().await.unwrap();
    let (locked, prompt): (Vec<OwnedObjectPath>, OwnedObjectPath) =
        service.call("Lock", &(vec![item_path.clone()],)).await.unwrap();
    assert_eq!((locked, prompt.as_str()), (vec![item_path.clone()], "/"));
    assert!(vault.is_locked(), "an item's Lock locked the vault");
    let change = tokio::time::timeout(Duration::from_secs(5), changes.next())
        .await
        .expect("Locked is announced")
        .unwrap();
    let args = change.args().unwrap();
    assert_eq!(args.interface_name().as_str(), "org.freedesktop.Secret.Collection");
    assert_eq!(
        bool::try_from(args.changed_properties().get("Locked").unwrap()).unwrap(),
        true
    );

    // Locked: every object says so, a search answers from the snapshot with
    // everything locked, and no secret leaves.
    assert!(collection.get_property::<bool>("Locked").await.unwrap());
    assert!(item.get_property::<bool>("Locked").await.unwrap());
    assert_eq!(
        item.get_property::<String>("Label").await.unwrap(),
        "Castellan lock receipt"
    );
    let (unlocked, locked): (Vec<OwnedObjectPath>, Vec<OwnedObjectPath>) =
        service.call("SearchItems", &(&attributes,)).await.unwrap();
    assert_eq!((unlocked, locked), (Vec::new(), vec![item_path.clone()]));
    let secrets: HashMap<OwnedObjectPath, (OwnedObjectPath, Vec<u8>, Vec<u8>, String)> = service
        .call("GetSecrets", &(vec![item_path.clone()], &session))
        .await
        .unwrap();
    assert!(secrets.is_empty(), "GetSecrets releases nothing while locked");
    let refused = item
        .call::<_, _, (OwnedObjectPath, Vec<u8>, Vec<u8>, String)>("GetSecret", &(&session,))
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("IsLocked"), "{refused}");

    // Unlock returns a Prompt; its cancel leaves the vault locked.
    vault.answer(false);
    let cancelled = lookup();
    assert!(cancelled.stdout.is_empty(), "a cancelled prompt releases nothing");
    assert_eq!(vault.prompts(), 1, "secret-tool met the prompt");
    assert!(vault.is_locked());

    // And the resident's unlock: secret-tool gets its secret.
    vault.answer(true);
    let unlocked = lookup();
    assert!(unlocked.status.success());
    assert_eq!(String::from_utf8(unlocked.stdout).unwrap().trim(), "swordfish");
    assert_eq!(vault.prompts(), 2);
    assert!(!vault.is_locked());
    assert!(!collection.get_property::<bool>("Locked").await.unwrap());
}
