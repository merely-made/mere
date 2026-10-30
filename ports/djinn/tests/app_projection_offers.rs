// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A real app-owned Knot directory is served through the existing door using
//! arbitrary provider and viewer labels. No application-specific registration
//! is installed in the host catalog. Owner policy, denial, revocation and
//! provider disconnect are exercised over native local streams.

use std::sync::Arc;
use std::time::Duration;

use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker_on};
use djinn::door::identity_endpoint::IdentityAuthority;
use djinn::door::local_session::DoorIdentity;
use djinn::door::projection_offers::ProviderPermission;
use djinn::door::resident_identity::ResidentIdentity;
use graphshell_endpoint::door::app_admission::{
    AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId,
};
use graphshell_endpoint::door::identity::{
    AgentListenerView, CarryView, IdentitySurfaceSnapshot, VaultLockView, VaultProtectionView,
    VaultView,
};
use graphshell_endpoint::door::local_endpoint::LocalListener;
use graphshell_endpoint::door::projection::AppBrokerClient;
use graphshell_endpoint::door::projection_offer::PROJECTION_PROVIDER_ROUTE;
use graphshell_endpoint::door::{app_client::AppConnection, projection_provider};
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use personae::{IdentityError, InMemoryProvider, RetainedKeys};

#[cfg(not(windows))]
struct SocketPath(std::path::PathBuf);

#[cfg(not(windows))]
impl Drop for SocketPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// A resident identity with fixed door keys and no custody route: the route
/// under test is Knot's, and only djinn links the real keeper (dramatis D4).
struct FixtureResident(InMemoryProvider);

impl DoorIdentity for FixtureResident {
    fn door_keys(&self) -> Result<Arc<RetainedKeys>, IdentityError> {
        self.0.door_keys()
    }
}

impl ResidentIdentity for FixtureResident {}

impl IdentityAuthority for FixtureResident {
    fn snapshot(&self) -> std::io::Result<IdentitySurfaceSnapshot> {
        Ok(IdentitySurfaceSnapshot {
            vault: VaultView {
                protection: VaultProtectionView::Ephemeral,
                lock: VaultLockView::Unlocked,
                agent: AgentListenerView::StandaloneRetained,
            },
            profiles: Vec::new(),
            ssh_keys: Vec::new(),
            carry: CarryView::default(),
            pending_signing: Vec::new(),
            signing_history: Vec::new(),
        })
    }

    fn apply_intent(&self, _intent: &str, _payload: &[u8]) -> Result<(), String> {
        Err("the fixture resident takes no identity intents".into())
    }
}

fn resident_host() -> Arc<FixtureResident> {
    Arc::new(FixtureResident(InMemoryProvider::from_seed([0xA1; 32])))
}

const PROVIDER: &str = "independent-directory-app";
const VIEWER: &str = "independent-viewer";
const ROUTE: &str = "independent/documents";

fn permission(viewer: &str) -> ProviderPermission {
    ProviderPermission {
        provider: AppId::new(PROVIDER),
        route: AppRouteId::new(ROUTE).unwrap(),
        viewers: vec![AppId::new(viewer)],
        max_sessions: 2,
        timeout: Duration::from_secs(3),
    }
}

struct Door {
    endpoint: String,
    catalog: AppEndpointCatalog,
    grants: AppRouteGrants,
    server: tokio::task::JoinHandle<()>,
    #[cfg(not(windows))]
    _path: SocketPath,
}
impl Drop for Door {
    fn drop(&mut self) {
        self.server.abort();
    }
}

async fn door() -> Door {
    #[cfg(windows)]
    let endpoint = format!(r"\\.\pipe\app-offer-{}", uuid::Uuid::new_v4());
    #[cfg(not(windows))]
    let path =
        SocketPath(std::env::temp_dir().join(format!("app-offer-{}.sock", uuid::Uuid::new_v4())));
    #[cfg(not(windows))]
    let endpoint = path.0.display().to_string();
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    catalog
        .set_provider_permissions(vec![permission(VIEWER)])
        .await
        .unwrap();
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    let listener = LocalListener::bind(&endpoint, "projection offer test")
        .await
        .unwrap();
    let serving_catalog = catalog.clone();
    let serving_grants = grants.clone();
    let server = tokio::spawn(async move {
        let _ = serve_app_broker_on(
            listener,
            resident_host(),
            serving_grants,
            60_000,
            None,
            serving_catalog,
        )
        .await;
    });
    Door {
        endpoint,
        catalog,
        grants,
        server,
        #[cfg(not(windows))]
        _path: path,
    }
}

struct Provider(
    tokio::task::JoinHandle<Result<(), graphshell_endpoint::door::app_client::AppClientError>>,
);
impl Drop for Provider {
    fn drop(&mut self) {
        self.0.abort();
    }
}

async fn provider(door: &Door, root: &std::path::Path) -> Provider {
    provider_with_limit(door, root, 2).await
}

async fn provider_with_limit(door: &Door, root: &std::path::Path, limit: usize) -> Provider {
    let connection = AppConnection::open_route_at(
        &door.endpoint,
        AppId::new(PROVIDER),
        AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap(),
    )
    .await
    .unwrap();
    let root = root.to_owned();
    let running = Provider(tokio::spawn(async move {
        projection_provider::serve(
            connection,
            AppRouteId::new(ROUTE).unwrap(),
            "Documents".into(),
            limit,
            move |_| knot_editor::KnotEndpoint::open(&root).map_err(|error| error.to_string()),
        )
        .await
    }));
    wait_offer(&door.catalog, true).await;
    running
}

async fn wait_offer(catalog: &AppEndpointCatalog, present: bool) {
    tokio::time::timeout(Duration::from_secs(4), async {
        loop {
            if catalog
                .update(|catalog| catalog.offers().iter().any(|offer| offer.id == ROUTE))
                .await
                == present
            {
                return;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await
    .expect("provider catalog lifecycle completed within its bound");
}

async fn viewer(
    door: &Door,
    app: &str,
) -> Result<AppBrokerClient, graphshell_endpoint::door::app_client::AppClientError> {
    tokio::time::timeout(
        Duration::from_secs(5),
        AppBrokerClient::open_route_at(
            &door.endpoint,
            AppId::new(app),
            AppRouteId::new(ROUTE).unwrap(),
        ),
    )
    .await
    .expect("viewer admission is bounded")
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn real_directory_offer_has_separate_viewer_permission_and_revocation() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("field.knot"), "# Field\n").unwrap();
    let door = door().await;
    door.grants.update(|grants| {
        grants.set_denied(
            AppId::new(PROVIDER),
            AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap(),
            true,
        )
    });
    assert!(
        AppConnection::open_route_at(
            &door.endpoint,
            AppId::new(PROVIDER),
            AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap()
        )
        .await
        .is_err(),
        "provider policy cannot override an explicit owner denial"
    );
    door.grants.update(|grants| {
        grants.set_denied(
            AppId::new(PROVIDER),
            AppRouteId::new(PROJECTION_PROVIDER_ROUTE).unwrap(),
            false,
        )
    });
    let _provider = provider(&door, directory.path()).await;
    assert!(viewer(&door, "ungranted-viewer").await.is_err());
    door.grants.update(|grants| {
        grants.set_denied(AppId::new(VIEWER), AppRouteId::new(ROUTE).unwrap(), true)
    });
    assert!(
        viewer(&door, VIEWER).await.is_err(),
        "policy cannot override an explicit owner denial"
    );
    door.grants.update(|grants| {
        grants.set_denied(AppId::new(VIEWER), AppRouteId::new(ROUTE).unwrap(), false)
    });
    let mut client = viewer(&door, VIEWER).await.unwrap();
    let opened = client.open_session().await.unwrap();
    assert_eq!(opened.descriptor.label, "Knot");
    let request = opened.descriptor.projections[0].request.clone();
    let snapshot = client.snapshot(request.clone()).await.unwrap();
    assert!(snapshot.scene.active_item_count() > 0);
    let resource = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .next()
        .expect("the directory supplies presentation content")
        .resource;
    assert!(
        !client
            .resource(snapshot.session.clone(), resource)
            .await
            .unwrap()
            .is_empty()
    );
    // Provider permission never writes admission into the shared viewer grant map.
    assert!(door.grants.current().iter().next().is_none());
    door.catalog
        .set_provider_permissions(Vec::new())
        .await
        .unwrap();
    wait_offer(&door.catalog, false).await;
    assert!(
        tokio::time::timeout(Duration::from_secs(4), client.snapshot(request))
            .await
            .expect("live viewer is cancelled on withdrawal")
            .is_err()
    );
    assert!(viewer(&door, VIEWER).await.is_err());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn idle_provider_disconnect_withdraws_and_replacement_does_not_inherit_viewers() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("field.knot"), "# Field\n").unwrap();
    let door = door().await;
    let running = provider(&door, directory.path()).await;
    let mut client = viewer(&door, VIEWER).await.unwrap();
    let opened = client.open_session().await.unwrap();
    let request = opened.descriptor.projections[0].request.clone();
    drop(running);
    wait_offer(&door.catalog, false).await;
    assert!(
        tokio::time::timeout(Duration::from_secs(4), client.snapshot(request))
            .await
            .expect("disconnect cancels viewer")
            .is_err()
    );
    door.catalog
        .set_provider_permissions(vec![permission("replacement-viewer")])
        .await
        .unwrap();
    let _replacement = provider(&door, directory.path()).await;
    assert!(viewer(&door, VIEWER).await.is_err());
    let mut replacement_viewer = viewer(&door, "replacement-viewer").await.unwrap();
    let opened = replacement_viewer.open_session().await.unwrap();
    assert!(
        replacement_viewer
            .snapshot(opened.descriptor.projections[0].request.clone())
            .await
            .is_ok()
    );
    replacement_viewer.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn withdrawing_an_offer_closes_an_idle_viewer_socket() {
    use base64::Engine;
    use graphshell_endpoint::door::{
        app_admission::AppHello,
        frame::{read_native_message_async, write_native_message_async},
        local_endpoint::connect_local,
        wire::{APP_CONNECT_SCHEMA, AppHostMessage, AppMessage},
    };
    use tokio::io::AsyncReadExt;
    type Reply = AppHostMessage<(), (), ()>;
    let directory = tempfile::tempdir().unwrap();
    let door = door().await;
    let _provider = provider(&door, directory.path()).await;
    let mut socket = connect_local(&door.endpoint).await.unwrap();
    write_native_message_async(
        &mut socket,
        &AppHello::for_route(AppId::new(VIEWER), AppRouteId::new(ROUTE).unwrap()),
    )
    .await
    .unwrap();
    let Reply::Challenge { challenge } = read_native_message_async::<_, Reply>(&mut socket)
        .await
        .unwrap()
        .unwrap()
    else {
        panic!("expected shared door challenge")
    };
    write_native_message_async(
        &mut socket,
        &AppMessage::<(), ()>::Connect {
            schema: APP_CONNECT_SCHEMA.into(),
            host_nonce: challenge.host_nonce,
            client_nonce: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode([0x13; 32]),
        },
    )
    .await
    .unwrap();
    assert!(matches!(
        read_native_message_async::<_, Reply>(&mut socket)
            .await
            .unwrap(),
        Some(Reply::Connected { .. })
    ));
    door.catalog
        .set_provider_permissions(Vec::new())
        .await
        .unwrap();
    // No request after revocation: socket closure must wake an idle viewer.
    let mut byte = [0];
    let count = tokio::time::timeout(Duration::from_secs(4), socket.read(&mut byte))
        .await
        .expect("withdrawal closes idle viewer transport")
        .unwrap();
    assert_eq!(count, 0);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn provider_capacity_refusal_preserves_its_existing_viewer() {
    let directory = tempfile::tempdir().unwrap();
    std::fs::write(directory.path().join("field.knot"), "# Field\n").unwrap();
    let door = door().await; // host permits two sessions
    let _provider = provider_with_limit(&door, directory.path(), 1).await;
    let mut first = viewer(&door, VIEWER).await.unwrap();
    let opened = first.open_session().await.unwrap();
    assert!(
        viewer(&door, VIEWER).await.is_err(),
        "provider's own limit applies"
    );
    assert!(
        first
            .snapshot(opened.descriptor.projections[0].request.clone())
            .await
            .is_ok(),
        "a refused Open cannot close an uncreated session or withdraw the provider"
    );
    first.close().await.unwrap();
}
