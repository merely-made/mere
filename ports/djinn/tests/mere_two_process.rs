// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Reservoir plan V2, the two-process receipt.
//!
//! Two applications, running as two processes attached through djinn's door
//! to one session of one mere, each see the other's edit through a revision
//! bell, with no stale copy left behind. The parent serves the door, as djinn
//! does, with the reservoir route and the mere routes over one catalog; the
//! two applications are this test binary re-run as children, so each holds
//! its own connection in its own address space, as Turnstone and the Knot
//! editor would.

use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, ProjectionSession,
    ProjectionSnapshot,
};
use djinn::resident_mere::{MereRoutes, mere_route_id};
use djinn::resident_reservoir::{
    EnsureMereV1, RESERVOIR_ENSURE_MERE_INTENT, RESIDENT_RESERVOIR_ROUTE, ResidentReservoir,
};
use graphshell::identity::VaultProtectionView;
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::AppBrokerClient;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell::native::personae_host::PersonaeHost;
use graphshell::session_item::{APPLY_EDITS_INTENT, ApplyEditsV1};
use graphshell_endpoint::{IntentSink, ProjectionCatalog, ProjectionSource};
use pandect::{CapturedDelta, mere_dir};
use personae::{Ed25519Keypair, IdentityVault, InMemoryStorage, PersonaId, Profile, ProfileId};
use sceno::InstanceId;
use uuid::Uuid;

const ENDPOINT_ENV: &str = "DJINN_MERE_RECEIPT_ENDPOINT";
const APP_ENV: &str = "DJINN_MERE_RECEIPT_APP";
const ROLE_ENV: &str = "DJINN_MERE_RECEIPT_ROLE";
const CHILD_TEST: &str = "child_edits_the_shared_session";
const DOMAIN: &str = "divination";
const PATIENCE: Duration = Duration::from_secs(60);

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x6d65_7265))
}

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-mere-receipt-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!("djinn-mere-receipt-{}.sock", Uuid::new_v4()))
        .display()
        .to_string();
}

fn resident_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("default".into()),
        "Default",
        Ed25519Keypair::from_seed([0x4d; 32]),
    );
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

/// Start this test binary as the child for `app`, its stdout read line by line.
fn spawn_child(endpoint: &str, app: &str, role: &str) -> (Child, Receiver<String>) {
    let mut child = Command::new(std::env::current_exe().expect("the test binary knows its path"))
        .args(["--exact", CHILD_TEST, "--ignored", "--nocapture"])
        .args(["--test-threads", "1"])
        .env(ENDPOINT_ENV, endpoint)
        .env(APP_ENV, app)
        .env(ROLE_ENV, role)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("the child process starts");
    let stdout = child.stdout.take().expect("piped stdout");
    let (lines, receiver) = channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines().map_while(Result::ok) {
            let _ = lines.send(line);
        }
    });
    (child, receiver)
}

/// Collect the child's lines until one contains `marker`, or fail at patience.
fn read_until(lines: &Receiver<String>, marker: &str, seen: &mut Vec<String>) {
    loop {
        let line = lines
            .recv_timeout(PATIENCE)
            .unwrap_or_else(|_| panic!("no {marker:?} from the child; saw {seen:#?}"));
        let found = line.contains(marker);
        seen.push(line);
        if found {
            return;
        }
    }
}

fn finish(mut child: Child, lines: Receiver<String>, mut seen: Vec<String>) -> Vec<String> {
    let status = child.wait().expect("the child ends");
    seen.extend(lines.try_iter());
    let mut stderr = String::new();
    if let Some(mut err) = child.stderr.take() {
        let _ = std::io::Read::read_to_string(&mut err, &mut stderr);
    }
    assert!(
        status.success(),
        "status {status:?}\nstdout {seen:#?}\nstderr {stderr}"
    );
    for line in &seen {
        if let Some(at) = line.find("RECEIPT") {
            println!("child {}", &line[at..]);
        }
    }
    seen
}

#[tokio::test(flavor = "multi_thread")]
async fn two_processes_edit_one_session_through_djinn_and_hear_each_other() {
    let root = tempfile::tempdir().unwrap();
    let reservoir = ResidentReservoir::open(root.path(), Some(persona()))
        .await
        .unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    let routes = MereRoutes::new(
        root.path().to_path_buf(),
        persona(),
        catalog.clone(),
        grants.clone(),
        vec![AppId::new("turnstone"), AppId::new("knot-editor")],
    );
    catalog
        .update(|catalog| reservoir.register(catalog, Some(routes)))
        .await
        .unwrap();

    // The mere, ensured through the reservoir route: its own route opens.
    let context = AdmittedEndpointContext::new(ProjectionSession("v2:receipt".into()), [0x2b; 32])
        .with_application("turnstone");
    let mut door = catalog
        .update(|catalog| catalog.open(RESIDENT_RESERVOIR_ROUTE, &context))
        .await
        .unwrap();
    let request = door.describe().projections.remove(0).request;
    let listed = door.snapshot(request).unwrap();
    let ensured = door
        .invoke(IntentInvocation {
            session: listed.session.clone(),
            target: InstanceId(0),
            observed_epoch: listed.scene.epoch,
            observed_revision: listed.scene.revision,
            intent: RESERVOIR_ENSURE_MERE_INTENT.into(),
            payload: serde_json::to_vec(&EnsureMereV1::new(DOMAIN)).unwrap(),
        })
        .unwrap();
    assert_eq!(ensured, IntentResult::Accepted);

    let endpoint = endpoint_name();
    let server = {
        let (endpoint, grants, catalog) = (endpoint.clone(), grants.clone(), catalog.clone());
        tokio::spawn(async move {
            let _ =
                serve_app_broker(&endpoint, resident_host(), grants, 60_000, None, catalog).await;
        })
    };
    // Wait for the door on the mere's own route.
    let route = AppRouteId::new(mere_route_id(DOMAIN)).unwrap();
    let mut probe = None;
    for _ in 0..100 {
        if let Ok(client) =
            AppBrokerClient::open_route_at(&endpoint, AppId::new("knot-editor"), route.clone())
                .await
        {
            probe = Some(client);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    probe
        .expect("the door serves the mere's route")
        .close()
        .await
        .unwrap();

    // The Knot editor attaches and waits for a bell; then Turnstone edits.
    let (knot, knot_lines) = spawn_child(&endpoint, "knot-editor", "wait-then-edit");
    let mut knot_seen = Vec::new();
    read_until(&knot_lines, "READY", &mut knot_seen);
    let (turnstone, turnstone_lines) = spawn_child(&endpoint, "turnstone", "edit-then-wait");
    let turnstone_seen = finish(turnstone, turnstone_lines, Vec::new());
    let knot_seen = finish(knot, knot_lines, knot_seen);
    for (seen, app) in [(&turnstone_seen, "turnstone"), (&knot_seen, "knot-editor")] {
        assert!(
            seen.iter()
                .any(|line| line.contains(&format!("RECEIPT {app} heard"))),
            "{app} never heard the other's edit: {seen:#?}"
        );
    }

    // The stored changes name both applications, as the door admitted them.
    let mere = reservoir.meres().await.remove(0);
    let sessions = mere_dir(root.path(), persona(), mere.id).join("sessions");
    let mut changes = String::new();
    for entry in std::fs::read_dir(sessions).unwrap() {
        changes += &std::fs::read_to_string(entry.unwrap().path().join("changes.jsonl")).unwrap();
    }
    for app in ["turnstone", "knot-editor"] {
        assert!(changes.contains(&format!("\"via\":\"{app}\"")), "{changes}");
    }
    server.abort();
}

fn nodes(snapshot: &ProjectionSnapshot) -> usize {
    // Every item but the session item, which keeps instance 0.
    snapshot.scene.active_items_in_order().len() - 1
}

async fn add_node(client: &mut AppBrokerClient, snapshot: &ProjectionSnapshot, n: u128) {
    let edit = ApplyEditsV1::new(vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
        id: Uuid::from_u128(n).to_string(),
        url: format!("https://{n}.receipt.test/"),
        position: [0.0, 0.0],
    }]);
    let body = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: snapshot.session.clone(),
            target: InstanceId(0),
            observed_epoch: snapshot.scene.epoch,
            observed_revision: snapshot.scene.revision,
            intent: APPLY_EDITS_INTENT.into(),
            payload: serde_json::to_vec(&edit).unwrap(),
        }))
        .await
        .expect("the edit is answered");
    assert!(
        matches!(body, CarrierResponseBody::Intent(IntentResult::Accepted)),
        "{body:?}"
    );
}

/// One application. It runs only when the parent above starts it.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "runs only as a child process of the two-process receipt"]
async fn child_edits_the_shared_session() {
    let (Ok(endpoint), Ok(app), Ok(role)) = (
        std::env::var(ENDPOINT_ENV),
        std::env::var(APP_ENV),
        std::env::var(ROLE_ENV),
    ) else {
        return;
    };
    let route = AppRouteId::new(mere_route_id(DOMAIN)).unwrap();
    let mut client = AppBrokerClient::open_route_at(&endpoint, AppId::new(&app), route)
        .await
        .expect("attached through the door");
    let opened = client.open_session().await.unwrap();
    let graph = opened.descriptor.projections[1].request.clone();
    let first = client.snapshot(graph.clone()).await.unwrap();
    let hear = |bell: &chirograph::CarrierNotice| {
        assert_eq!(bell.session, graph.session, "the bell names the graph");
    };
    match role.as_str() {
        "wait-then-edit" => {
            let before = nodes(&first);
            println!("READY {app} attached with {before} nodes");
            use std::io::Write as _;
            std::io::stdout().flush().unwrap();
            let bell = tokio::time::timeout(PATIENCE, client.wait_for_notice())
                .await
                .expect("a bell within patience")
                .unwrap();
            hear(&bell);
            let seen = client.snapshot(graph.clone()).await.unwrap();
            assert_eq!(nodes(&seen), before + 1, "the other's node arrived");
            println!(
                "RECEIPT {app} heard the other's edit at revision {}",
                bell.revision.0
            );
            add_node(&mut client, &seen, 2).await;
            println!("RECEIPT {app} edited");
        },
        "edit-then-wait" => {
            add_node(&mut client, &first, 1).await;
            println!("RECEIPT {app} edited");
            let bell = tokio::time::timeout(PATIENCE, client.wait_for_notice())
                .await
                .expect("a bell within patience")
                .unwrap();
            hear(&bell);
            let seen = client.snapshot(graph.clone()).await.unwrap();
            assert_eq!(nodes(&seen), 2, "both nodes, and no stale copy");
            println!(
                "RECEIPT {app} heard the other's edit at revision {}",
                bell.revision.0
            );
        },
        other => panic!("no child role {other:?}"),
    }
    client.close().await.unwrap();
}
