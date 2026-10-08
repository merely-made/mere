// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! V4 durable access decisions reloaded by a fresh resident process.

use std::path::Path;
use std::process::Command;

use chirograph::ProjectionSession;
use djinn::resident_mere::{MereRoutes, ambient_mere_route_id, mere_route_id};
use djinn::resident_reservoir::ResidentReservoir;
use graphshell::lifecycle::AdmittedEndpointContext;
use graphshell::native::app_admission::{
    AllowedAppRoutes, AppId, AppRequest, AppRouteGrants, AppRouteId,
};
use graphshell::native::app_broker::AppEndpointCatalog;
use graphshell::native::endpoint_catalog::ResidentEndpointCatalog;
use graphshell_endpoint::{ProjectionCatalog, ProjectionSource};
use pandect::{
    Author, DomainId, MereApplicationAccess, MereSessions, ReservoirStore, open_mere_backend,
    open_reservoir_backend,
};
use personae::PersonaId;
use uuid::Uuid;

const ROOT: &str = "DJINN_V4_REOPEN_ROOT";
const PARENT: &str = "DJINN_V4_PARENT_PID";
const CHILD: &str = "fresh_resident_enforces_recorded_access";

fn persona() -> PersonaId {
    PersonaId::from_uuid(Uuid::from_u128(0x7634_6772_616e_7473))
}

#[tokio::test(flavor = "multi_thread")]
async fn access_survives_a_fresh_resident_process() {
    let root = tempfile::tempdir().unwrap();
    let backend = open_reservoir_backend(root.path(), persona()).unwrap();
    let mut store = ReservoirStore::open(backend, persona()).await.unwrap();
    for domain in ["divination", "notes"] {
        let (mere, _) = store
            .ensure(DomainId::new(domain).unwrap(), 1)
            .await
            .unwrap();
        MereSessions::new(open_mere_backend(root.path(), persona(), mere.id).unwrap())
            .mint(
                Author::person(persona().as_uuid().to_string()).via("turnstone"),
                None,
            )
            .await
            .unwrap();
        if domain == "divination" {
            for (app, denied) in [("turnstone", true), ("knot-editor", false)] {
                store
                    .set_application_access(
                        mere.id,
                        app.into(),
                        MereApplicationAccess {
                            denied,
                            ambient: true,
                            recorded_at_ms: 2,
                            author: Author::person(persona().as_uuid().to_string())
                                .via("turnstone"),
                        },
                    )
                    .await
                    .unwrap();
            }
        }
    }
    drop(store);
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            CHILD,
            "--ignored",
            "--nocapture",
            "--test-threads",
            "1",
        ])
        .env(ROOT, root.path())
        .env(PARENT, std::process::id().to_string())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("RECEIPT V4"), "{stdout}");
    for line in stdout.lines().filter(|line| line.contains("RECEIPT")) {
        println!("{line}");
    }
    // The fresh resident did not erase the denial or turn ambient consent
    // into a reservoir-wide consent for another mere/application.
    let store = ReservoirStore::open(
        open_reservoir_backend(root.path(), persona()).unwrap(),
        persona(),
    )
    .await
    .unwrap();
    assert_eq!(store.access().count(), 2);
}

#[tokio::test(flavor = "multi_thread")]
#[ignore = "spawned by the parent with an isolated reservoir"]
async fn fresh_resident_enforces_recorded_access() {
    let Some(root) = std::env::var_os(ROOT) else {
        return;
    };
    assert_ne!(
        std::process::id().to_string(),
        std::env::var(PARENT).unwrap()
    );
    let root = Path::new(&root);
    let reservoir = ResidentReservoir::open(root, Some(persona()))
        .await
        .unwrap();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    let routes = MereRoutes::new(
        root.into(),
        persona(),
        catalog.clone(),
        grants.clone(),
        ["turnstone", "knot-editor", "cleromancy"]
            .into_iter()
            .map(AppId::new)
            .collect(),
    );
    // Serve fails closed if the composition forgets to load policy.
    let meres = reservoir.meres().await;
    assert!(routes.serve(&meres[0]).await.is_err());
    routes.load_access(&reservoir).await.unwrap();
    for mere in &meres {
        routes.serve(mere).await.unwrap();
    }
    let request = |app: &str, domain: &str, ambient: bool| AppRequest {
        app: AppId::new(app),
        route: AppRouteId::new(if ambient {
            ambient_mere_route_id(domain)
        } else {
            mere_route_id(domain)
        })
        .unwrap(),
    };
    for ambient in [false, true] {
        let denied = request("turnstone", "divination", ambient);
        assert!(
            grants
                .current()
                .admit(&denied)
                .unwrap_err()
                .to_string()
                .contains("explicitly denied")
        );
        let context = AdmittedEndpointContext::new(ProjectionSession("v4:reopen".into()), [4; 32])
            .with_application("turnstone");
        assert!(
            catalog
                .update(|catalog| catalog.open(denied.route.as_str(), &context))
                .await
                .is_err()
        );
    }
    for app in ["cleromancy", "knot-editor"] {
        for domain in ["divination", "notes"] {
            assert!(grants.current().admit(&request(app, domain, false)).is_ok());
        }
    }
    assert!(
        grants
            .current()
            .admit(&request("knot-editor", "divination", true))
            .is_ok()
    );
    assert!(
        grants
            .current()
            .admit(&request("knot-editor", "notes", true))
            .is_err()
    );
    assert!(
        grants
            .current()
            .admit(&request("cleromancy", "divination", true))
            .is_err()
    );
    assert!(
        grants
            .current()
            .admit(&request("unknown", "divination", false))
            .is_err()
    );
    let context = AdmittedEndpointContext::new(ProjectionSession("v4:ambient".into()), [4; 32])
        .with_application("knot-editor");
    let mut endpoint = catalog
        .update(|catalog| catalog.open(&ambient_mere_route_id("divination"), &context))
        .await
        .unwrap();
    let request = endpoint.describe().projections[0].request.clone();
    assert!(endpoint.snapshot(request).is_ok());
    println!(
        "RECEIPT V4 pid={} parent={} revision={} denial-before-open=true ambient-per-app-per-mere=true explicit-without-ambient=true fresh-owner-reopen=true",
        std::process::id(),
        std::env::var(PARENT).unwrap(),
        std::env::var("MERE_RECEIPT_REVISION").unwrap_or_else(|_| "working-tree".into())
    );
}
