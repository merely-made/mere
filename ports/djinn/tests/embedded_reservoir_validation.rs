// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V3/V5 regression: one domain validator is installed by `open_with` before
//! serving and reused by daemon composition. Real process ownership changes
//! retain the valid graph; a forbidden complete batch changes no projection,
//! session file or journal. The test harness reads durable bytes as evidence;
//! the attached application uses only admitted carrier requests.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, ProjectionSnapshot,
};
use djinn::embedded_reservoir::{ReservoirAttachment, ReservoirAttachmentOptions};
use djinn::resident_mere::MereRoutes;
use djinn::resident_reservoir::ResidentReservoir;
use graphshell::identity::VaultProtectionView;
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::AppBrokerClient;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell::native::tasks::ResidentTasks;
use graphshell::session_item::{APPLY_EDITS_INTENT, ApplyEditsV1};
use pandect::{CapturedDelta, DomainId, GraphValidator, MereId, mere_dir};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use sceno::InstanceId;
use uuid::Uuid;

const CHILD: &str = "child_serves_the_registered_domain";
const DOMAIN: &str = "validated-fixture";
const ROOT: &str = "DJINN_V35_ROOT";
const ENDPOINT: &str = "DJINN_V35_ENDPOINT";
const MODE: &str = "DJINN_V35_MODE";
const PATIENCE: Duration = Duration::from_secs(30);

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x5633_5635))
}
fn endpoint() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-v35-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return format!("/tmp/dv35-{}.sock", Uuid::new_v4());
}
fn options(root: &Path, address: &str) -> ReservoirAttachmentOptions {
    ReservoirAttachmentOptions {
        endpoint: address.into(),
        application: AppId::new("validator-fixture"),
        applications: vec![AppId::new("validator-fixture")],
        shared_root: root.into(),
        persona: persona(),
        domain: DomainId::new(DOMAIN).unwrap(),
        session_duration: Duration::from_secs(60),
    }
}
fn identity() -> Arc<PersonaeHost<InMemoryStorage>> {
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(
            InMemoryStorage::new(),
            Profile::new(
                ProfileId("v35-fixture".into()),
                "V3/V5 fixture",
                Ed25519Keypair::from_seed([0x35; 32]),
            ),
        ),
        None,
        VaultProtectionView::Ephemeral,
    ))
}
fn validator(calls: Arc<AtomicUsize>) -> GraphValidator {
    Arc::new(move |graph| {
        calls.fetch_add(1, Ordering::SeqCst);
        if graph.nodes().any(|(_, node)| node.title == "Forbidden") {
            Err("fixture forbids the complete candidate".into())
        } else {
            Ok(())
        }
    })
}
fn revision() -> String {
    let git = |arguments: &[&str]| {
        Command::new("git")
            .args(["-C", env!("CARGO_MANIFEST_DIR")])
            .args(arguments)
            .output()
            .unwrap()
            .stdout
    };
    let mut revision = String::from_utf8(git(&["rev-parse", "HEAD"]))
        .unwrap()
        .trim()
        .to_owned();
    if !git(&["status", "--porcelain"]).is_empty() {
        revision.push_str("+dirty-diagnostic");
    }
    revision
}

struct Process {
    child: Child,
    lines: Receiver<String>,
}
impl Process {
    fn start(root: &Path, address: &str, mode: &str) -> Self {
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
            .env(ENDPOINT, address)
            .env(MODE, mode)
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
        let process = Self { child, lines };
        process.wait_for("READY");
        process
    }
    fn wait_for(&self, marker: &str) {
        loop {
            let line = self.lines.recv_timeout(PATIENCE).unwrap();
            if line.contains(marker) {
                println!("{line}");
                break;
            }
        }
    }
    fn stop(mut self) {
        writeln!(self.child.stdin.as_mut().unwrap(), "stop").unwrap();
        self.wait_for("RELEASED");
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

async fn snapshots(client: &mut AppBrokerClient) -> Vec<ProjectionSnapshot> {
    let opened = client.open_session().await.unwrap();
    assert_eq!(
        opened.descriptor.projections.len(),
        3,
        "sessions, graph and V3 archive"
    );
    let mut snapshots = Vec::new();
    for offer in opened.descriptor.projections {
        snapshots.push(client.snapshot(offer.request).await.unwrap());
    }
    snapshots
}
fn node(id: u128) -> CapturedDelta {
    CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(id).to_string(),
        url: format!("https://{id}.validator.test/"),
        position: [0.0, 0.0],
    }
}
fn title(id: u128, title: &str) -> CapturedDelta {
    CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(id).to_string(),
        title: title.into(),
    }
}
async fn apply(
    client: &mut AppBrokerClient,
    snapshot: &ProjectionSnapshot,
    edits: Vec<CapturedDelta>,
) -> IntentResult {
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&ApplyEditsV1::new(edits)).unwrap(),
        }))
        .await
        .unwrap();
    let CarrierResponseBody::Intent(result) = response else {
        panic!("{response:?}");
    };
    result
}
fn durable_files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn collect(base: &Path, at: &Path, files: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(at).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(base, &path, files);
            } else {
                files.insert(
                    path.strip_prefix(base).unwrap().to_owned(),
                    std::fs::read(path).unwrap(),
                );
            }
        }
    }
    let domain = DomainId::new(DOMAIN).unwrap();
    let sessions = mere_dir(root, persona(), MereId::derive(persona(), &domain)).join("sessions");
    let mut files = BTreeMap::new();
    collect(&sessions, &sessions, &mut files);
    files
}

#[tokio::test(flavor = "multi_thread")]
async fn registered_domain_validator_survives_ownership_and_refuses_complete_batches() {
    let root = tempfile::tempdir().unwrap();
    let unusable = root.path().join("not-a-storage-directory");
    std::fs::write(&unusable, b"attached clients cannot open stores here").unwrap();
    let forbidden_identity = || async {
        panic!("the client must not open local identity");
        #[allow(unreachable_code)]
        Ok::<_, String>(identity())
    };
    let mut original_session = None;
    for (phase, mode) in ["embedded", "daemon", "embedded"].into_iter().enumerate() {
        let address = endpoint();
        let process = Process::start(root.path(), &address, mode);
        assert_ne!(process.child.id(), std::process::id());
        let mut attached =
            ReservoirAttachment::open(options(&unusable, &address), forbidden_identity)
                .await
                .unwrap();
        assert!(!attached.is_embedded());
        let start = snapshots(attached.client_mut()).await;
        if phase == 0 {
            // A temporary forbidden intermediate is allowed: domain authority
            // checks the complete final candidate, not each individual delta.
            assert_eq!(
                apply(
                    attached.client_mut(),
                    &start[1],
                    vec![node(1), title(1, "Forbidden"), title(1, "Allowed")]
                )
                .await,
                IntentResult::Accepted
            );
        }
        let before = snapshots(attached.client_mut()).await;
        assert_eq!(
            before[1].scene.active_items_in_order().len(),
            2,
            "one valid node plus session item"
        );
        let sessions: Vec<_> = before[0]
            .scene
            .tables
            .sources
            .iter()
            .flatten()
            .filter(|source| source.adapter == graphshell::session_item::SESSIONS_SOURCE)
            .filter(|source| Uuid::parse_str(&source.id).is_ok())
            .map(|source| source.id.clone())
            .collect();
        assert_eq!(sessions.len(), 1);
        match &original_session {
            None => original_session = Some(sessions),
            Some(original) => assert_eq!(&sessions, original),
        }
        let bytes = durable_files(root.path());
        assert!(
            matches!(apply(attached.client_mut(), &before[1], vec![node(2), title(1,"Forbidden")]).await,
            IntentResult::Rejected { reason } if reason.contains("fixture forbids"))
        );
        let after = snapshots(attached.client_mut()).await;
        assert_eq!(
            serde_json::to_value(&after).unwrap(),
            serde_json::to_value(&before).unwrap(),
            "rejection changes no graph, sessions or archive projection"
        );
        assert_eq!(
            durable_files(root.path()),
            bytes,
            "rejection changes no durable session/checkpoint/journal/view bytes"
        );
        if phase == 0 {
            // The callback is bound to one domain, not every mere on the door.
            let mut control = options(&unusable, &address);
            control.domain = DomainId::new("unvalidated-control").unwrap();
            let mut control = ReservoirAttachment::open(control, forbidden_identity)
                .await
                .unwrap();
            let control_graph = snapshots(control.client_mut()).await;
            assert_eq!(
                apply(
                    control.client_mut(),
                    &control_graph[1],
                    vec![node(1), title(1, "Forbidden")]
                )
                .await,
                IntentResult::Accepted
            );
            control.close().await.unwrap();
        }
        attached.close().await.unwrap();
        println!(
            "RECEIPT VALIDATOR revision={} mode={mode} phase={phase} retained-session=true whole-batch-refused=true all-projections-and-session-bytes-unchanged=true",
            revision()
        );
        process.stop();
    }
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "runs only as the real process owner launched by the regression"]
async fn child_serves_the_registered_domain() {
    let (Ok(root), Ok(address), Ok(mode)) = (
        std::env::var(ROOT),
        std::env::var(ENDPOINT),
        std::env::var(MODE),
    ) else {
        return;
    };
    let config = options(Path::new(&root), &address);
    let calls = Arc::new(AtomicUsize::new(0));
    let registered = Arc::new(AtomicBool::new(false));
    let mut daemon = None;
    let mut attached = if mode == "embedded" {
        let (calls, registered) = (calls.clone(), registered.clone());
        ReservoirAttachment::open_with(
            config.clone(),
            || async { Ok(identity()) },
            move |routes| async move {
                registered.store(true, Ordering::SeqCst);
                Ok(routes.with_domain_validator(DOMAIN, validator(calls)))
            },
        )
        .await
        .unwrap()
    } else {
        let reservoir = ResidentReservoir::open(&config.shared_root, Some(config.persona))
            .await
            .unwrap();
        let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
        let grants = AppRouteGrants::new(AllowedAppRoutes::new([(
            config.application.clone(),
            ResidentReservoir::route(),
        )]));
        let routes = MereRoutes::new(
            config.shared_root.clone(),
            config.persona,
            catalog.clone(),
            grants.clone(),
            config.applications.clone(),
        )
        .with_domain_validator(DOMAIN, validator(calls.clone()));
        registered.store(true, Ordering::SeqCst);
        routes.load_access(&reservoir).await.unwrap();
        for mere in reservoir.meres().await {
            routes.serve(&mere).await.unwrap();
        }
        catalog
            .update(|catalog| reservoir.register(catalog, Some(routes)))
            .await
            .unwrap();
        let tasks = ResidentTasks::new();
        let (scope, door, address) = (tasks.clone(), catalog.clone(), address.clone());
        let server = tokio::spawn(async move {
            scope
                .scope(serve_app_broker(
                    &address,
                    std::sync::Arc::new(djinn::keeper::Keeper::new(identity())),
                    grants,
                    60_000,
                    None,
                    door,
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
            assert!(!server.is_finished());
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        daemon = Some((server, tasks, catalog, reservoir));
        ReservoirAttachment::open(config.clone(), || async {
            panic!("live daemon must not request identity");
            #[allow(unreachable_code)]
            Ok::<_, String>(identity())
        })
        .await
        .unwrap()
    };
    assert_eq!(attached.is_embedded(), mode == "embedded");
    snapshots(attached.client_mut()).await;
    assert!(registered.load(Ordering::SeqCst));
    assert!(
        calls.load(Ordering::SeqCst) > 0,
        "validator installed before first graph served"
    );
    println!(
        "READY mode={mode} pid={} validator-installed=true",
        std::process::id()
    );
    std::io::stdout().flush().unwrap();
    let mut line = String::new();
    std::io::stdin().read_line(&mut line).unwrap();
    attached.close().await.unwrap();
    if let Some((server, tasks, catalog, reservoir)) = daemon {
        server.abort();
        let _ = server.await;
        tasks.cancel_and_join().await;
        catalog
            .update(|catalog| *catalog = ResidentEndpointCatalog::new())
            .await;
        drop(reservoir);
    }
    ResidentReservoir::open(&config.shared_root, Some(config.persona))
        .await
        .unwrap();
    println!(
        "RELEASED mode={mode} pid={} validator-calls={}",
        std::process::id(),
        calls.load(Ordering::SeqCst)
    );
}
