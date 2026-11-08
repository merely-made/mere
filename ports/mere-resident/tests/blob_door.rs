// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The blob door route over the real application door (cycle breaks P2a).

use std::sync::Arc;
use std::time::Duration;

use mere_resident::resident_blob_door::BlobDoor;
use mere_resident::resident_blobs::ResidentBlobCustody;
use mere_resident::settings::ResidentContentSettings;
use mere_resident::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use dramatis_resident::identity::IdentityAuthority;
use mere_resident::door::local_session::DoorIdentity;
use mere_resident::door::resident_identity::ResidentIdentity;
use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use ostiary::app_admission::AppId;
use ostiary::declaration::{AppDeclaration, NamePattern};
use graphshell_door::blob_door::{BLOB_CUSTODY_ROUTE, BlobDoorClient, BlobDoorError, MigrationState};
use dramatis::view::{
    AgentListenerView, CarryView, IdentitySurfaceSnapshot, VaultLockView, VaultProtectionView,
    VaultView,
};
use ostiary::route::ResidentEndpointRoute;
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use personae::{IdentityError, InMemoryProvider, RetainedKeys};

#[cfg(not(windows))]
struct SocketPath(std::path::PathBuf);

/// Windows has no socket path to remove; the type exists so `socket()` has one shape.
#[cfg(windows)]
struct SocketPath;

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

const SCOPE: [u8; 32] = [0x77; 32];

fn socket() -> (String, Option<SocketPath>) {
    #[cfg(windows)]
    return (format!(r"\\.\pipe\blob-door-{}", uuid::Uuid::new_v4()), None);
    #[cfg(not(windows))]
    {
        let path = SocketPath(
            std::path::Path::new("/tmp").join(format!("blob-door-{}.sock", uuid::Uuid::new_v4())),
        );
        (path.0.display().to_string(), Some(path))
    }
}

async fn connect(endpoint: &str, app: &str) -> Result<BlobDoorClient, BlobDoorError> {
    let declaration = AppDeclaration {
        routes: vec![NamePattern::exact(BLOB_CUSTODY_ROUTE)],
        ..AppDeclaration::default()
    };
    let mut last = None;
    for _ in 0..100 {
        match BlobDoorClient::open_declared_at(endpoint, AppId::new(app), declaration.clone()).await
        {
            Ok(client) => return Ok(client),
            // Only a socket that is not up yet is worth another try; the
            // door's answer, a refusal or a close, is final.
            Err(BlobDoorError::Door(ostiary::app_client::AppClientError::Io(error))) => {
                last = Some(BlobDoorError::Door(ostiary::app_client::AppClientError::Io(error)));
            },
            Err(error) => return Err(error),
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    Err(last.unwrap())
}

/// A granted application holds scoped leases on the resident's blob store over
/// the real door: put in parts, lease, read back, serve authority, migrate a
/// legacy store, and be cut off by revocation. An ungranted application never
/// reaches the route and a second granted one cannot take the first's scope.
#[tokio::test(flavor = "multi_thread")]
async fn a_granted_application_holds_scoped_leases_over_the_door() {
    let temp = tempfile::tempdir().unwrap();
    let custody = ResidentBlobCustody::open(
        &temp.path().join("data"),
        &ResidentContentSettings {
            gc_interval_seconds: 60,
            ..ResidentContentSettings::default()
        },
    )
    .await
    .unwrap();
    let door = BlobDoor::new(custody.clone());
    let mut catalog = ResidentEndpointCatalog::new();
    let route = door.register(&mut catalog).unwrap();
    assert_eq!(route.id(), BLOB_CUSTODY_ROUTE);
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([
        (AppId::new("blob-fixture"), route.clone()),
        (AppId::new("blob-intruder"), route),
    ]));
    let (endpoint, _socket) = socket();
    let serving = endpoint.clone();
    let server = tokio::spawn(async move {
        let _ = serve_app_broker(
            &serving,
            resident_host(),
            grants,
            60_000,
            None,
            AppEndpointCatalog::new(catalog),
        )
        .await;
    });

    let mut app = connect(&endpoint, "blob-fixture").await.unwrap();
    assert_eq!(app.bind_scope(SCOPE).await.unwrap(), 0);

    // A legacy store, migrated under the application's scope.
    let legacy_root = temp.path().join("old.blobs");
    let legacy = transport::BlobStore::open(&legacy_root).await.unwrap();
    legacy.put_bytes_named(b"old evidence".to_vec(), b"old/a").await.unwrap();
    legacy.shutdown().await.unwrap();
    let source = legacy_root.display().to_string();
    assert_eq!(
        app.migrate_legacy(SCOPE, "legacy.fixture", &source).await.unwrap(),
        (MigrationState::Copied, 1)
    );
    assert_eq!(
        app.migrate_legacy(SCOPE, "legacy.fixture", &source).await.unwrap(),
        (MigrationState::AlreadyComplete, 1)
    );

    // A put across several parts, a read through resource chunks, a lease.
    let bytes: Vec<u8> = (0..200_000u32).map(|n| (n % 253) as u8).collect();
    let hash = app.put(SCOPE, "fixture.evidence", b"one", &bytes).await.unwrap();
    assert_eq!(&hash, blake3::hash(&bytes).as_bytes());
    assert_eq!(app.read(hash).await.unwrap(), bytes);
    assert_eq!(app.lease_hash(SCOPE, "fixture.evidence", b"one").await.unwrap(), Some(hash));
    app.lease(SCOPE, "fixture.alias", b"two", hash).await.unwrap();
    assert!(app.release(SCOPE, "fixture.alias", b"two").await.unwrap());
    assert_eq!(
        app.lease_hash(SCOPE, "fixture.evidence", b"one").await.unwrap(),
        Some(hash),
        "releasing one lease leaves the other"
    );

    // The same authorizer the transport's blob server asks.
    let peer = [5u8; 32];
    assert!(app.retain(SCOPE, hash).await.unwrap());
    assert!(app.allow_reader(SCOPE, peer).await.unwrap());
    assert!(app.allows(SCOPE, peer, hash).await.unwrap());
    assert!(custody.authorizer().allows(
        transport::BlobScope::new(SCOPE),
        &peer,
        transport::BlobHash::from_bytes(hash)
    ));
    let ledger = app.ledger().await.unwrap();
    assert_eq!(ledger.app, "blob-fixture");
    assert_eq!(ledger.scopes.len(), 1);
    assert_eq!(ledger.scopes[0].hashes.len(), 2, "the migrated blob and the new one");
    assert_eq!(ledger.scopes[0].readers.len(), 1);

    // A second admitted application cannot take the scope or read its bytes.
    let mut intruder = connect(&endpoint, "blob-intruder").await.unwrap();
    let refused = intruder.bind_scope(SCOPE).await.unwrap_err();
    assert!(refused.to_string().contains("blob-fixture"), "{refused}");
    assert!(intruder.read(hash).await.is_err());

    // An application nobody granted never reaches the route.
    assert!(connect(&endpoint, "blob-stranger").await.is_err());

    // Revocation cuts the live session and withdraws serving; leases stay.
    door.revoke("blob-fixture").await.unwrap();
    assert!(app.lease_hash(SCOPE, "fixture.evidence", b"one").await.is_err());
    assert!(!custody.authorizer().allows(
        transport::BlobScope::new(SCOPE),
        &peer,
        transport::BlobHash::from_bytes(hash)
    ));
    assert_eq!(
        custody.blobs().leased_hashes(transport::BlobScope::new(SCOPE)).await.unwrap().len(),
        2
    );
    assert!(connect(&endpoint, "blob-fixture").await.is_err());
    door.restore("blob-fixture");
    let mut again = connect(&endpoint, "blob-fixture").await.unwrap();
    assert_eq!(again.bind_scope(SCOPE).await.unwrap(), 2, "serving custody is rebuilt from the leases");

    server.abort();
}
