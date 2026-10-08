// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V5 qualification uses actual child processes and isolated identities.
//! The standalone application and competing owners run this test binary in
//! separate address spaces; the client path is given an unusable storage root
//! and an identity factory that panics if called. Receipts name the checkout
//! revision; a dirty checkout is explicitly diagnostic evidence only.

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use chirograph::{CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult};
use djinn::embedded_reservoir::{
    ReservoirAttachment, ReservoirAttachmentError, ReservoirAttachmentOptions,
};
use djinn::resident_mere::MereRoutes;
use djinn::resident_reservoir::ResidentReservoir;
use graphshell::identity::VaultProtectionView;
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::AppBrokerClient;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell::native::personae_host::PersonaeHost;
use graphshell::native::tasks::ResidentTasks;
use graphshell::session_item::{APPLY_EDITS_INTENT, ApplyEditsV1};
use pandect::{CapturedDelta, DomainId};
use personae::{
    Ed25519Keypair, IdentityStorage, IdentityVault, InMemoryStorage, PersonaId, Profile, ProfileId,
    SealedProfileStorage,
};
use sceno::InstanceId;
use uuid::Uuid;

const CHILD: &str = "child_runs_the_standalone_application";
const ROOT: &str = "DJINN_V5_TEST_ROOT";
const ENDPOINT: &str = "DJINN_V5_TEST_ENDPOINT";
const ROLE: &str = "DJINN_V5_TEST_ROLE";
const NODES: &str = "DJINN_V5_TEST_NODES";
const PATIENCE: Duration = Duration::from_secs(30);

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x5635_7265))
}

fn endpoint() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-v5-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return format!("/tmp/dv5-{}.sock", Uuid::new_v4());
}

fn options(root: &Path, endpoint: &str) -> ReservoirAttachmentOptions {
    ReservoirAttachmentOptions {
        endpoint: endpoint.into(),
        application: AppId::new("cleromancy"),
        applications: vec![
            AppId::new("cleromancy"),
            AppId::new("knot-editor"),
            AppId::new("denied-fixture"),
        ],
        shared_root: root.into(),
        persona: persona(),
        domain: DomainId::new("divination").unwrap(),
        session_duration: Duration::from_secs(60),
    }
}

fn identity() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("v5-fixture".into()),
        "V5 fixture",
        Ed25519Keypair::from_seed([0x56; 32]),
    );
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

fn revision() -> String {
    let output = Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut revision = String::from_utf8(output.stdout).unwrap().trim().to_owned();
    let dirty = Command::new("git")
        .args(["-C", env!("CARGO_MANIFEST_DIR"), "status", "--porcelain"])
        .output()
        .unwrap();
    if !dirty.stdout.is_empty() {
        revision.push_str("+dirty-diagnostic");
    }
    revision
}

struct Process {
    child: Child,
    lines: Receiver<String>,
    seen: Vec<String>,
}
impl Process {
    fn spawn(root: &Path, endpoint: &str, role: &str, nodes: usize) -> Self {
        let mut child = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                CHILD,
                "--ignored",
                "--nocapture",
                "--test-threads",
                "1",
            ])
            .env(ROOT, root)
            .env(ENDPOINT, endpoint)
            .env(ROLE, role)
            .env(NODES, nodes.to_string())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .unwrap();
        let (send, lines) = channel();
        let stdout = child.stdout.take().unwrap();
        std::thread::spawn(move || {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let _ = send.send(line);
            }
        });
        Self {
            child,
            lines,
            seen: Vec::new(),
        }
    }
    fn receive(&mut self, marker: &str) -> String {
        loop {
            let line = self
                .lines
                .recv_timeout(PATIENCE)
                .unwrap_or_else(|error| panic!("waiting for {marker}: {error}; {:?}", self.seen));
            self.seen.push(line.clone());
            if line.contains(marker) {
                println!("{line}");
                return line;
            }
        }
    }
    fn signal(&mut self) {
        let _ = writeln!(self.child.stdin.as_mut().unwrap(), "continue");
    }
    fn finish(mut self) {
        let status = self.child.wait().unwrap();
        assert!(status.success(), "child failed: {:?}", self.seen);
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn graph(client: &mut AppBrokerClient) -> chirograph::ProjectionSnapshot {
    let opened = client.open_session().await.unwrap();
    client
        .snapshot(opened.descriptor.projections[1].request.clone())
        .await
        .unwrap()
}
fn nodes(snapshot: &chirograph::ProjectionSnapshot) -> usize {
    snapshot.scene.active_items_in_order().len() - 1
}
async fn edit(client: &mut AppBrokerClient, id: u128) {
    let snapshot = graph(client).await;
    let edit = ApplyEditsV1::new(vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.v5.test/"),
        position: [0.0, 0.0],
    }]);
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session,
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&edit).unwrap(),
        }))
        .await
        .unwrap();
    assert!(
        matches!(
            response,
            CarrierResponseBody::Intent(IntentResult::Accepted)
        ),
        "{response:?}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn standalone_and_client_processes_share_one_owner_and_reopen() {
    let root = tempfile::tempdir().unwrap();
    let address = endpoint();
    let mut standalone = Process::spawn(root.path(), &address, "owner", 1);
    let owned = standalone.receive("RECEIPT OWNED");
    assert!(owned.contains(&format!("pid={}", standalone.child.id())));
    assert_ne!(standalone.child.id(), std::process::id());
    standalone.signal();
    standalone.receive("RECEIPT RELEASED");
    standalone.finish();
    // A separate process composes Djinn's ordinary resident source/route/door
    // directly, bypassing the application's ownership selector.
    let mut daemon = Process::spawn(root.path(), &address, "daemon", 1);
    daemon.receive("RECEIPT DAEMON");

    // A client could not open either reservoir or mere files under this root.
    let forbidden = root.path().join("not-a-directory");
    std::fs::write(&forbidden, b"storage cannot open beneath a regular file").unwrap();
    let mut client_options = options(&forbidden, &address);
    client_options.application = AppId::new("knot-editor");
    let forbidden_identity = || async {
        panic!("a live-resident client must never open local identity or reservoir");
        #[allow(unreachable_code)]
        Ok::<_, String>(identity())
    };
    let mut client = ReservoirAttachment::open(client_options.clone(), forbidden_identity)
        .await
        .unwrap();
    assert!(!client.is_embedded());
    assert_eq!(nodes(&graph(client.client_mut()).await), 1);
    edit(client.client_mut(), 2).await;
    assert_eq!(nodes(&graph(client.client_mut()).await), 2);
    client.close().await.unwrap();
    println!(
        "RECEIPT CLIENT revision={} pid={} local-store-root-unusable=true identity-factory-called=false",
        revision(),
        std::process::id()
    );

    let mut wrong = client_options.clone();
    wrong.persona = PersonaId::from_uuid(Uuid::from_u128(0xdead));
    assert!(matches!(
        ReservoirAttachment::open(wrong, forbidden_identity).await,
        Err(ReservoirAttachmentError::Persona { .. })
    ));
    let mut recorded_denial = client_options.clone();
    recorded_denial.application = AppId::new("denied-fixture");
    let denied_result = ReservoirAttachment::open(recorded_denial, forbidden_identity).await;
    assert!(
        matches!(denied_result, Err(ReservoirAttachmentError::Ensure(reason)) if reason.contains("denied"))
    );
    let mut denied = client_options;
    denied.application = AppId::new("unadmitted-fixture");
    assert!(matches!(
        ReservoirAttachment::open(denied, forbidden_identity).await,
        Err(ReservoirAttachmentError::Client(_))
    ));
    println!(
        "RECEIPT REFUSALS revision={} wrong-persona=true unadmitted-app=true persisted-denial=true no-fallback=true",
        revision()
    );

    let mut competitor = Process::spawn(root.path(), &endpoint(), "refused", 0);
    competitor.receive("RECEIPT REFUSED");
    competitor.finish();
    daemon.signal();
    daemon.receive("RECEIPT RELEASED");
    daemon.finish();

    let mut reopened = Process::spawn(root.path(), &address, "owner", 2);
    reopened.receive("RECEIPT OWNED");
    let mut denied_reopen = options(&forbidden, &address);
    denied_reopen.application = AppId::new("denied-fixture");
    assert!(
        matches!(ReservoirAttachment::open(denied_reopen, forbidden_identity).await,
        Err(ReservoirAttachmentError::Ensure(reason)) if reason.contains("denied"))
    );
    reopened.signal();
    reopened.receive("RECEIPT RELEASED");
    reopened.finish();
}

#[tokio::test(flavor = "multi_thread")]
async fn two_starting_processes_cannot_both_own_the_reservoir() {
    let root = tempfile::tempdir().unwrap();
    let mut first = Process::spawn(root.path(), &endpoint(), "race", 0);
    let mut second = Process::spawn(root.path(), &endpoint(), "race", 0);
    first.receive("READY");
    second.receive("READY");
    first.signal();
    second.signal();
    let a = first.receive("RECEIPT RACE");
    let b = second.receive("RECEIPT RACE");
    assert_ne!(
        a.contains("owned=true"),
        b.contains("owned=true"),
        "{a}\n{b}"
    );
    first.signal();
    second.signal();
    first.finish();
    second.finish();
    ResidentReservoir::open(root.path(), Some(persona()))
        .await
        .unwrap();
}

#[cfg(unix)]
#[tokio::test(flavor = "multi_thread")]
async fn another_transport_error_never_opens_local_identity_or_storage() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("ordinary-file");
    std::fs::write(&file, b"fixture endpoint cannot exist beneath a file").unwrap();
    let shared = root.path().join("never-opened");
    let config = options(&shared, &file.join("socket").display().to_string());
    let result = ReservoirAttachment::open(config, || async {
        panic!("transport refusal must not open local identity");
        #[allow(unreachable_code)]
        Ok::<_, String>(identity())
    })
    .await;
    assert!(matches!(result, Err(ReservoirAttachmentError::Client(_))));
    assert!(!shared.exists());
}

#[tokio::test(flavor = "multi_thread")]
async fn locked_identity_never_opens_a_reservoir_and_lock_stops_an_embedded_owner() {
    let root = tempfile::tempdir().unwrap();
    let storage = SealedProfileStorage::open_with_key(root.path().join("vault"), [0x35; 32]);
    storage.enroll_passphrase(b"isolated v5 fixture").unwrap();
    let profile = Profile::new(
        ProfileId("v5-lock".into()),
        "V5 lock fixture",
        Ed25519Keypair::from_seed([0x53; 32]),
    );
    storage.save_profile(&profile).unwrap();
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::open(storage, &profile.id).unwrap(),
        None,
        VaultProtectionView::Passphrase,
    ));
    host.lock_vault().unwrap();
    let config = options(&root.path().join("meres"), &endpoint());
    let locked = ReservoirAttachment::open(config.clone(), || async { Ok(host.clone()) }).await;
    assert!(
        matches!(locked, Err(ReservoirAttachmentError::Embedded(reason)) if reason.contains("locked"))
    );
    assert!(!config.shared_root.exists());
    host.unlock_vault(personae::UnlockMethod::Passphrase(b"isolated v5 fixture"))
        .unwrap();
    let attached = ReservoirAttachment::open(config.clone(), || async { Ok(host.clone()) })
        .await
        .unwrap();
    host.lock_vault().unwrap();
    // Awaiting close also awaits cancellation and release after the lock bell.
    let _ = attached.close().await;
    ResidentReservoir::open(&config.shared_root, Some(persona()))
        .await
        .unwrap();
    println!(
        "RECEIPT LOCK revision={} locked-start-refused=true live-lock-releases-owner=true",
        revision()
    );
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "only runs in actual child processes of V5 qualification"]
async fn child_runs_the_standalone_application() {
    let (Ok(root), Ok(address), Ok(role), Ok(expected)) = (
        std::env::var(ROOT),
        std::env::var(ENDPOINT),
        std::env::var(ROLE),
        std::env::var(NODES),
    ) else {
        return;
    };
    let config = options(Path::new(&root), &address);
    if role == "daemon" {
        daemon_fixture(config).await;
        return;
    }
    if role == "race" {
        println!("READY pid={}", std::process::id());
        std::io::stdout().flush().unwrap();
        let mut line = String::new();
        std::io::stdin().read_line(&mut line).unwrap();
    }
    let attempt = ReservoirAttachment::open(config, || async { Ok(identity()) }).await;
    match (role.as_str(), attempt) {
        ("refused", Err(error)) => {
            assert!(
                error.to_string().contains("could not open the reservoir"),
                "{error}"
            );
            println!(
                "RECEIPT REFUSED revision={} pid={} reason={error}",
                revision(),
                std::process::id()
            );
        },
        ("race", Err(error)) => {
            assert!(
                error.to_string().contains("could not open the reservoir"),
                "{error}"
            );
            println!(
                "RECEIPT RACE revision={} pid={} owned=false reason={error}",
                revision(),
                std::process::id()
            );
        },
        (role @ ("owner" | "race"), Ok(mut attached)) => {
            assert!(attached.is_embedded());
            if role == "owner" {
                let expected: usize = expected.parse().unwrap();
                if nodes(&graph(attached.client_mut()).await) == 0 {
                    edit(attached.client_mut(), 1).await;
                }
                assert_eq!(nodes(&graph(attached.client_mut()).await), expected);
                println!(
                    "RECEIPT OWNED revision={} pid={} nodes={expected}",
                    revision(),
                    std::process::id()
                );
            } else {
                println!(
                    "RECEIPT RACE revision={} pid={} owned=true",
                    revision(),
                    std::process::id()
                );
            }
            std::io::stdout().flush().unwrap();
            let mut line = String::new();
            std::io::stdin().read_line(&mut line).unwrap();
            attached.close().await.unwrap();
            ResidentReservoir::open(Path::new(&root), Some(persona()))
                .await
                .unwrap();
            println!(
                "RECEIPT RELEASED revision={} pid={}",
                revision(),
                std::process::id()
            );
        },
        (role, result) => panic!(
            "unexpected {role} result: {}",
            match result {
                Ok(_) => "owned".into(),
                Err(error) => error.to_string(),
            }
        ),
    }
}

/// The exact reusable source/route/broker composition in Djinn's startup.
/// This is a library daemon fixture, not the full installed Djinn binary or
/// its network/credential lanes. It owns real disk stores in this process.
async fn daemon_fixture(config: ReservoirAttachmentOptions) {
    let reservoir = ResidentReservoir::open(&config.shared_root, Some(config.persona))
        .await
        .unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new(
        config
            .applications
            .iter()
            .cloned()
            .map(|app| (app, ResidentReservoir::route())),
    ));
    let routes = MereRoutes::new(
        config.shared_root.clone(),
        config.persona,
        catalog.clone(),
        grants.clone(),
        config.applications.clone(),
    );
    routes.load_access(&reservoir).await.unwrap();
    for mere in reservoir.meres().await {
        routes
            .set_access(
                &reservoir,
                mere.id,
                AppId::new("denied-fixture"),
                true,
                true,
                AppId::new("cleromancy"),
            )
            .await
            .unwrap();
        routes.serve(&mere).await.unwrap();
    }
    catalog
        .update(|catalog| reservoir.register(catalog, Some(routes)))
        .await
        .unwrap();
    let tasks = ResidentTasks::new();
    let scoped = tasks.clone();
    let server_catalog = catalog.clone();
    let address = config.endpoint.clone();
    let server = tokio::spawn(async move {
        scoped
            .scope(serve_app_broker(
                &address,
                identity(),
                grants,
                60_000,
                None,
                server_catalog,
            ))
            .await
    });
    for _ in 0..100 {
        if graphshell::native::local_endpoint::connect_local(&config.endpoint)
            .await
            .is_ok()
        {
            break;
        }
        assert!(!server.is_finished(), "daemon broker failed to start");
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    println!(
        "RECEIPT DAEMON revision={} pid={} composition=djinn-library",
        revision(),
        std::process::id()
    );
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    server.abort();
    let _ = server.await;
    tasks.cancel_and_join().await;
    catalog
        .update(|catalog| *catalog = ResidentEndpointCatalog::new())
        .await;
    drop(reservoir);
    ResidentReservoir::open(&config.shared_root, Some(config.persona))
        .await
        .unwrap();
    println!(
        "RECEIPT RELEASED revision={} pid={}",
        revision(),
        std::process::id()
    );
}
