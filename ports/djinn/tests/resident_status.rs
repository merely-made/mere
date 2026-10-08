// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Harness plan rulings 7 to 9, in process: only djinn's own label reads the
//! resident's status or stops it, the status route takes no intent, the stop
//! reaches the run loop's signal, and a non-installed start names every
//! default endpoint it would take. The harness reads djinn's wire forms with
//! its own types, so they are held together here.

use std::sync::Arc;
use std::time::Duration;

use djinn::resident_events::EVENTS_SCHEMA;
use djinn::resident_status::{
    self, AgentListenerV1, LockStateV1, RESIDENT_APP, RESIDENT_CONTROL_ROUTE,
    RESIDENT_STATUS_ROUTE, ResidentControlEndpoint, ResidentEndpointsV1, ResidentStatusEndpoint,
    ResidentStatusSource, ResidentStatusV1, STATUS_SCHEMA, StartupUnlockV1, StopSignal,
    SyncStatusV1, default_endpoints_taken,
};
use graphshell::identity::VaultProtectionView;
use graphshell::native::app_admission::{
    AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId, default_app_endpoint,
};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::device_broker::default_device_endpoint;
use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointRoute};
use graphshell::native::personae_host::PersonaeHost;
use personae::{Ed25519Keypair, IdentityVault, InMemoryStorage, Profile, ProfileId};
use uuid::Uuid;

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-resident-status-test-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!(
            "djinn-resident-status-test-{}.sock",
            Uuid::new_v4()
        ))
        .display()
        .to_string();
}

fn fixture() -> ResidentStatusV1 {
    ResidentStatusV1 {
        schema: STATUS_SCHEMA.into(),
        pid: 4242,
        started_ms: 1_791_213_240_000,
        installed: false,
        ready: true,
        startup_unlock: StartupUnlockV1::Passphrase,
        protection: VaultProtectionView::Passphrase.into(),
        lock: LockStateV1::Unlocked,
        endpoints: ResidentEndpointsV1 {
            agent: "agent".into(),
            agent_listener: AgentListenerV1::Receipt,
            browser: "browser".into(),
            app: "app".into(),
        },
        sync: Some(SyncStatusV1 {
            node_id: "ab".repeat(32),
            ticket: "ticket".into(),
        }),
    }
}

fn resident_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("default".into()),
        "Default",
        Ed25519Keypair::from_seed([0x5e; 32]),
    );
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

async fn open(endpoint: &str, app: &str, route: &str) -> Result<AppBrokerClient, AppClientError> {
    AppBrokerClient::open_route_at(endpoint, AppId::new(app), AppRouteId::new(route).unwrap()).await
}

#[tokio::test(flavor = "multi_thread")]
async fn only_djinn_reads_the_status_and_stops_the_resident() {
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([(
        AppId::new("turnstone"),
        ResidentEndpointRoute::new("identity", Duration::from_millis(50)).unwrap(),
    )]));
    let status = ResidentStatusSource::new(fixture());
    let stop = StopSignal::default();
    let route = catalog
        .update(|catalog| ResidentStatusEndpoint::register(status.clone(), catalog))
        .await
        .unwrap();
    resident_status::grant(&grants, route);
    let route = catalog
        .update(|catalog| ResidentControlEndpoint::register(stop.clone(), catalog))
        .await
        .unwrap();
    resident_status::grant(&grants, route);

    let endpoint = endpoint_name();
    let server = {
        let (endpoint, grants, catalog) = (endpoint.clone(), grants.clone(), catalog.clone());
        tokio::spawn(async move {
            let _ =
                serve_app_broker(&endpoint, resident_host(), grants, 60_000, None, catalog).await;
        })
    };

    // The positive control: djinn's label reads the status.
    let mut client = None;
    for _ in 0..100 {
        if let Ok(opened) = open(&endpoint, RESIDENT_APP, RESIDENT_STATUS_ROUTE).await {
            client = Some(opened);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.expect("djinn's label is admitted");
    assert_eq!(
        resident_status::read_status(&mut client).await.unwrap(),
        fixture()
    );
    client.close().await.unwrap();

    // The status moves under the route, and the next read sees it.
    status.update(|status| status.ready = false);
    let mut again = open(&endpoint, RESIDENT_APP, RESIDENT_STATUS_ROUTE)
        .await
        .unwrap();
    assert!(
        !resident_status::read_status(&mut again)
            .await
            .unwrap()
            .ready
    );
    // The status route is read-only: a stop sent there is not accepted.
    assert!(resident_status::request_stop(&mut again).await.is_err());
    again.close().await.unwrap();

    // Other applications are refused both routes before a session opens.
    for route in [RESIDENT_STATUS_ROUTE, RESIDENT_CONTROL_ROUTE] {
        for app in ["turnstone", "someone-elses-app"] {
            assert!(
                matches!(
                    open(&endpoint, app, route).await,
                    Err(AppClientError::Closed)
                ),
                "{app} must be refused {route}"
            );
        }
    }

    // The stop reaches the signal the run loop waits on.
    let mut control = open(&endpoint, RESIDENT_APP, RESIDENT_CONTROL_ROUTE)
        .await
        .unwrap();
    resident_status::request_stop(&mut control).await.unwrap();
    tokio::time::timeout(Duration::from_secs(5), stop.raised())
        .await
        .expect("the stop intent raised the signal");
    server.abort();
}

/// Ruling 8, resident side: a start without `--installed` names each default
/// endpoint it would take; the installed one takes them.
#[test]
fn a_non_installed_start_names_every_default_endpoint() {
    let (browser, app) = (default_device_endpoint(), default_app_endpoint());
    let at_defaults =
        |installed| default_endpoints_taken(installed, true, (&browser, &browser), (&app, &app));
    assert_eq!(at_defaults(false).len(), 3, "{:?}", at_defaults(false));
    assert!(at_defaults(true).is_empty());
    assert!(
        default_endpoints_taken(false, false, ("mine-b", &browser), ("mine-a", &app)).is_empty()
    );
    let shouted = browser.to_uppercase();
    let one = default_endpoints_taken(false, false, (&shouted, &browser), ("mine-a", &app));
    assert_eq!(
        one.len(),
        usize::from(cfg!(windows)),
        "pipe names ignore case"
    );
}

/// The harness's standard endpoints are the ones graphshell and castellan
/// treat as the installed resident's.
#[test]
fn the_harness_walls_name_the_resident_defaults() {
    let walls = djinn_testkit::walls::standard_endpoints();
    assert!(walls.contains(&default_app_endpoint()), "{walls:?}");
    assert!(walls.contains(&default_device_endpoint()), "{walls:?}");
    #[cfg(windows)]
    assert!(
        walls.contains(
            &graphshell::native::personae_host::STANDARD_WINDOWS_AGENT_ENDPOINT.to_string()
        )
    );
}

/// The harness reads the status route and the event file with its own types.
#[test]
fn the_harness_reads_djinns_wire_forms() {
    let status = fixture();
    let read: djinn_testkit::ResidentStatus =
        serde_json::from_value(serde_json::to_value(&status).unwrap()).unwrap();
    assert_eq!(read.schema, djinn_testkit::status::STATUS_SCHEMA);
    assert_eq!((read.pid, read.ready), (4242, true));
    assert_eq!(
        (
            read.startup_unlock.as_str(),
            read.lock.as_str(),
            read.protection.as_str()
        ),
        ("passphrase", "unlocked", "passphrase")
    );
    assert_eq!(read.endpoints.agent_listener, "receipt");
    assert_eq!(read.sync.unwrap().ticket, "ticket");
    assert_eq!(EVENTS_SCHEMA, djinn_testkit::status::EVENTS_SCHEMA);
}
