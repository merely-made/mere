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
/// under test is the door's, and only djinn links the real keeper (dramatis D4).
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

/// A supervised application that holds no store serves its scope's blobs to a
/// peer through the door (cycle breaks P2 step 3): the provider's source is a
/// `DoorBlobSource`, the verified bytes come from the resident's store, the
/// fetching peer checks them against the hash, and the serving rules hold:
/// retained hashes only, readers only, and revocation cuts it.
#[tokio::test(flavor = "multi_thread")]
async fn an_application_without_the_store_serves_verified_streams_through_the_door() {
    use graphshell_door::blob_door::ChunkInterval;
    use graphshell_door::blob_door_source::DoorBlobSource;
    use transport::{BlobHash, BlobScope, P2pandaTransport, PeerID, Transport as _};

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
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([
        (AppId::new("blob-fixture"), route.clone()),
        (AppId::new("blob-intruder"), route),
    ]));
    let (endpoint, _socket) = socket();
    let serving_endpoint = endpoint.clone();
    let server = tokio::spawn(async move {
        let _ = serve_app_broker(
            &serving_endpoint,
            resident_host(),
            grants,
            60_000,
            None,
            AppEndpointCatalog::new(catalog),
        )
        .await;
    });

    let scope = BlobScope::new(SCOPE);
    let mut app = connect(&endpoint, "blob-fixture").await.unwrap();
    app.bind_scope(SCOPE).await.unwrap();
    // The serving session is its own, so a peer's download does not hold the
    // custody calls.
    let source = DoorBlobSource::new(connect(&endpoint, "blob-fixture").await.unwrap());
    source.bind_scope(scope).await.unwrap();

    // Bound before any put: binding a scope rebuilds serving custody from every
    // durable lease (the restart rule), so a session bound later would retain
    // the leased-only blob below.
    let mut direct = connect(&endpoint, "blob-fixture").await.unwrap();
    direct.bind_scope(SCOPE).await.unwrap();

    let bytes: Vec<u8> = (0..200_000u32).map(|n| (n % 251) as u8).collect();
    let hash = app.put(SCOPE, "fixture.evidence", b"served", &bytes).await.unwrap();
    let leased_only = app
        .put(SCOPE, "fixture.evidence", b"leased-only", b"leased but never retained")
        .await
        .unwrap();
    let small = app.put(SCOPE, "fixture.evidence", b"small", b"a small blob").await.unwrap();

    let provider = P2pandaTransport::builder_from_seed_ref(&[0xA5; 32])
        .scoped_source(source.clone(), scope)
        .bind()
        .await
        .unwrap();
    let bob_blobs = transport::BlobStore::new();
    let bob = P2pandaTransport::builder_from_seed_ref(&[0xB5; 32])
        .blobs(&bob_blobs)
        .bind()
        .await
        .unwrap();
    let mallory_blobs = transport::BlobStore::new();
    let mallory = P2pandaTransport::builder_from_seed_ref(&[0xC5; 32])
        .blobs(&mallory_blobs)
        .bind()
        .await
        .unwrap();
    let provider_id: PeerID = provider.local_peer_id();
    let addr = provider.endpoint_addr().await.unwrap();
    bob.add_peer(addr.clone()).await.unwrap();
    mallory.add_peer(addr).await.unwrap();

    app.retain(SCOPE, hash).await.unwrap();
    app.retain(SCOPE, small).await.unwrap();
    app.allow_reader(SCOPE, bob.local_peer_id().to_bytes()).await.unwrap();

    // The peer fetches a multi-leaf blob and verifies it against its hash.
    bob_blobs
        .fetch_from(&bob, provider_id, BlobHash::from_bytes(hash))
        .await
        .expect("a reader fetches a retained blob served through the door");
    assert_eq!(
        bob_blobs.get_bytes(BlobHash::from_bytes(hash)).await.unwrap().to_vec(),
        bytes
    );

    // The verified read's ranges are honoured at the door itself.
    let bob_id = bob.local_peer_id().to_bytes();
    let mut whole = direct
        .bao_open(SCOPE, bob_id, hash, vec![ChunkInterval { start: 0, end: None }])
        .await
        .unwrap();
    let mut partial = direct
        .bao_open(SCOPE, bob_id, hash, vec![ChunkInterval { start: 0, end: Some(1) }])
        .await
        .unwrap();
    assert!(partial.total_len() < whole.total_len(), "one chunk streams less than the blob");
    let mut got = 0u64;
    while let Some(piece) = direct.bao_next(&mut whole).await.unwrap() {
        got += piece.len() as u64;
    }
    assert_eq!(got, whole.total_len());
    assert!(direct.bao_close(&partial).await.unwrap());
    // Streams are sequential: a closed cursor is gone.
    assert!(direct.bao_next(&mut partial).await.is_err());

    // Held under a lease but not retained: refused, at the door and on the wire.
    let refused = direct
        .bao_open(SCOPE, bob_id, leased_only, vec![ChunkInterval { start: 0, end: None }])
        .await
        .unwrap_err();
    assert!(refused.to_string().contains("not retained"), "{refused}");
    assert!(
        bob_blobs
            .fetch_from(&bob, provider_id, BlobHash::from_bytes(leased_only))
            .await
            .is_err()
    );
    assert!(!bob_blobs.has(BlobHash::from_bytes(leased_only)).await.unwrap());

    // A peer that is not a reader gets nothing.
    assert!(
        mallory_blobs
            .fetch_from(&mallory, provider_id, BlobHash::from_bytes(small))
            .await
            .is_err()
    );
    assert!(!mallory_blobs.has(BlobHash::from_bytes(small)).await.unwrap());

    // A scope the application does not own is not reachable at all.
    let mut intruder = connect(&endpoint, "blob-intruder").await.unwrap();
    assert!(
        intruder
            .bao_open(SCOPE, bob_id, hash, vec![ChunkInterval { start: 0, end: None }])
            .await
            .is_err()
    );

    // Taking the reader away cuts serving at once, without the child's help.
    app.deny_reader(SCOPE, bob_id).await.unwrap();
    let after_deny = transport::BlobStore::new();
    assert!(after_deny.fetch_from(&bob, provider_id, BlobHash::from_bytes(small)).await.is_err());
    assert!(!after_deny.has(BlobHash::from_bytes(small)).await.unwrap());
    app.allow_reader(SCOPE, bob_id).await.unwrap();

    // Revocation cuts the session: the child's source answers nothing more.
    door.revoke("blob-fixture").await.unwrap();
    let after_revoke = transport::BlobStore::new();
    assert!(after_revoke.fetch_from(&bob, provider_id, BlobHash::from_bytes(small)).await.is_err());
    assert!(!after_revoke.has(BlobHash::from_bytes(small)).await.unwrap());
    assert!(
        custody.blobs().has(BlobHash::from_bytes(small)).await.unwrap(),
        "revocation leaves the custody data"
    );

    server.abort();
}
