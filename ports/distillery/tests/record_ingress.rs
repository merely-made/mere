// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The admitted service mounts on the resident's existing endpoint/store.

extern crate personae as identity;

use distillery::mesh_host::{HostConfig, MeshHost};
use distillery::{ResidentAuthority, ResidentSettings, ResidentStorage};
use identity::{IdentityProvider, InMemoryProvider};
use mesh::ingress::{read_exact_inventory, submit_original};
use mesh::{
    BlobRef, DeterminismClass, JobSpec, MESH_AUTHOR_SALT, MeshEvent, MeshStore, ResourceId,
    SyncedMesh,
};
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use std::time::Duration;
use transport::{P2pandaTransport, Transport};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
const MESH: [u8; 32] = [0x4d; 32];
fn require(ok: bool, detail: &str) -> Result<()> {
    if ok {
        Ok(())
    } else {
        Err(detail.to_string().into())
    }
}

#[path = "../../../crates/mesh/mesh/examples/inventory_ingress/network.rs"]
mod network;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn resident_ingress_preserves_owner_expiry_and_shutdown() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let db = dir.path().join("mesh.redb");
    let settings = ResidentSettings {
        tick_every: Duration::from_millis(5),
        maintenance_every: None,
        blob_gc_every: Duration::from_secs(1),
        retention: distillery::RetentionSettings::default(),
    };
    let storage =
        ResidentStorage::open(dir.path().join("blobs"), MESH, settings.blob_gc_every).await?;
    let provider = InMemoryProvider::from_seed([31; 32]);
    let author = provider.derive_keypair(MESH_AUTHOR_SALT)?;
    let endpoint = Arc::new(
        P2pandaTransport::builder_for(&provider.derived_keypair(MESH_AUTHOR_SALT)?)
            .gossip()
            .blobs(&storage.blobs())
            .bind()
            .await?,
    );
    let peer_before = endpoint.local_peer_id();
    let (ep, gossip) = endpoint.sync_parts().ok_or("sync parts")?;
    let store = MeshStore::at_path(&db)?;
    let synced = SyncedMesh::join(ep, gossip, store, MESH).await?;
    let host = MeshHost::new(
        synced,
        author.clone(),
        HostConfig::supervised(storage.space()),
    );
    let mut resident = ResidentAuthority::new(host, endpoint.clone(), storage, settings)?;
    let clock = Arc::new(AtomicU64::new(1000));
    let mut config = network::config();
    config.retained_for = Duration::from_millis(10);
    let owner_clock = clock.clone();
    config.now_ms = Arc::new(move || owner_clock.load(Ordering::Relaxed));
    let mut too_short = network::config();
    too_short.retained_for = Duration::from_nanos(1);
    assert!(resident.enable_record_ingress(too_short).await.is_err());
    resident.enable_record_ingress(config).await?;
    assert_eq!(peer_before, endpoint.local_peer_id());
    assert!(
        resident
            .enable_record_ingress(network::config())
            .await
            .is_err()
    );
    let mut client = network::Client::bind(&endpoint).await?;
    let spec = JobSpec::simple(
        ResourceId::parse("mesh.echo/v1")?,
        "payload",
        BlobRef::blake3(b"fixture"),
        "result",
        1024,
        DeterminismClass::Exact,
    );
    let op = mesh::to_operation(
        &author,
        MESH,
        &MeshEvent::JobPostedV2 {
            spec: Box::new(spec),
            nonce: 1,
            at_ms: 1000,
        },
        0,
        None,
    );
    let bytes = p2panda_core::cbor::encode_cbor(&stickleback::operation_record(&op, true))?;
    for outcome in ["inserted", "duplicate"] {
        let hello = client.hello("submit", &["submit"])?;
        let mut stream = client.connect().await?;
        assert_eq!(
            submit_original(&mut stream, &hello, &bytes, network::CALL)
                .await?
                .outcome,
            outcome
        );
    }
    let existing_owner = resident.authority().host().synced().store();
    assert_eq!(existing_owner.ops(MESH).await?.len(), 1);
    assert_eq!(existing_owner.board(MESH).await?.jobs().count(), 1);
    let stored = existing_owner.operation(&op.hash).await?.unwrap();
    assert_eq!(stored.header.encode(), op.header.encode());
    assert_eq!(stored.body.unwrap().to_bytes(), op.body.unwrap().to_bytes());
    for (now, expected) in [(1000, 1), (1010, 0)] {
        clock.store(now, Ordering::Relaxed);
        let hello = client.hello("inventory", &["inventory"])?;
        let mut stream = client.connect().await?;
        let summary = read_exact_inventory(&mut stream, &hello, MESH, network::CALL).await?;
        let stickleback::inventory::Summary::Exact(page) = summary else {
            panic!("exact snapshot");
        };
        assert_eq!(page.ids().len(), expected);
    }
    assert_eq!(resident.record_ingress_stats().unwrap().owner_calls, 2);
    let mut stalled = client.connect().await?;
    tokio::io::AsyncWriteExt::write_all(&mut stalled, &[0]).await?;
    tokio::time::timeout(Duration::from_secs(1), async {
        while resident.record_ingress_stats().unwrap().active == 0 {
            tokio::time::sleep(Duration::from_millis(5)).await;
        }
    })
    .await?;
    tokio::time::timeout(Duration::from_secs(10), resident.shutdown()).await??;
    drop(stalled);
    client.transport.close().await?;
    drop(endpoint);
    let reopened = MeshStore::at_path(&db)?;
    assert_eq!(reopened.ops(MESH).await?.len(), 1);
    assert_eq!(reopened.board(MESH).await?.jobs().count(), 1);
    let new_service = mesh::ingress::RecordIngress::new(reopened, MESH, network::config())?;
    assert!(
        new_service.retained_snapshot()?.is_empty(),
        "volatile availability is rebuilt by later accepted originals"
    );
    Ok(())
}
