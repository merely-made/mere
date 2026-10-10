// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pairing plan D1: only the directory's granted caller reads it.
//!
//! The door is served as djinn serves it, with the grant made through the same
//! function the resident uses. The granted caller reads the directory over the
//! door (the positive control); every other caller is refused before a session
//! opens.

use std::sync::Arc;
use std::time::Duration;

use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use djinn::resident_devices::{
    self, DEVICE_DIRECTORY_APP, DEVICE_DIRECTORY_ROUTE, DeviceDirectoryEndpoint,
    DeviceDirectorySource, DeviceDirectoryV1, read_directory,
};
use graphshell::identity::VaultProtectionView;
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointRoute};
use graphshell::native::personae_host::PersonaeHost;
use personae::{Ed25519Keypair, ProfileId};
use uuid::Uuid;

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(
        r"\\.\pipe\djinn-device-directory-receipt-{}",
        Uuid::new_v4()
    );
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!(
            "djinn-device-directory-receipt-{}.sock",
            Uuid::new_v4()
        ))
        .display()
        .to_string();
}

fn resident_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("default".into()),
        "Default",
        Ed25519Keypair::from_seed([0x4e; 32]),
    );
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

fn fixture() -> DeviceDirectoryV1 {
    DeviceDirectoryV1 {
        version: 1,
        local_node: "d1".repeat(32),
        devices: Vec::new(),
    }
}

async fn open(endpoint: &str, app: &str) -> Result<AppBrokerClient, AppClientError> {
    AppBrokerClient::open_route_at(
        endpoint,
        AppId::new(app),
        AppRouteId::new(DEVICE_DIRECTORY_ROUTE).unwrap(),
    )
    .await
}

#[tokio::test(flavor = "multi_thread")]
async fn only_the_granted_caller_reads_the_device_directory() {
    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    // Turnstone holds its usual identity grant, so its refusal below is about
    // this route, not about being unknown to the door.
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([(
        AppId::new("turnstone"),
        ResidentEndpointRoute::new("identity", Duration::from_millis(50)).unwrap(),
    )]));
    let source = DeviceDirectorySource::from_fn(|| Box::pin(async { Ok(fixture()) }));
    let route = catalog
        .update(|catalog| DeviceDirectoryEndpoint::register(source, catalog))
        .await
        .unwrap();
    resident_devices::grant(&grants, route);

    let endpoint = endpoint_name();
    let server = {
        let (endpoint, grants, catalog) = (endpoint.clone(), grants.clone(), catalog.clone());
        tokio::spawn(async move {
            let _ =
                serve_app_broker(&endpoint, resident_host(), grants, 60_000, None, catalog).await;
        })
    };

    // The positive control: the granted caller reads the directory.
    let mut client = None;
    for _ in 0..100 {
        if let Ok(opened) = open(&endpoint, DEVICE_DIRECTORY_APP).await {
            client = Some(opened);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.expect("the granted caller is admitted");
    assert_eq!(read_directory(&mut client).await.unwrap(), fixture());
    client.close().await.unwrap();

    // A first-party app granted other routes, an app the device does not know,
    // and the granted label on the version-one hello are each refused.
    for app in ["turnstone", "someone-elses-app"] {
        let refused = open(&endpoint, app).await;
        assert!(
            matches!(refused, Err(AppClientError::Closed)),
            "{app} must be refused the device directory, got {:?}",
            refused.map(|_| "a session")
        );
    }
    let legacy = AppBrokerClient::open_at(&endpoint, AppId::new(DEVICE_DIRECTORY_APP)).await;
    assert!(
        matches!(legacy, Err(AppClientError::Closed)),
        "the directory's label holds no identity-route grant"
    );

    // And the door still serves the granted caller after the refusals.
    let mut again = open(&endpoint, DEVICE_DIRECTORY_APP).await.unwrap();
    assert_eq!(read_directory(&mut again).await.unwrap(), fixture());
    again.close().await.unwrap();
    server.abort();
}
