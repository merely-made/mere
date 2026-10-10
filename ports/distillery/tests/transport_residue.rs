// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a bound Distillery lane keeps of the persona's master (vault lock
//! rulings 2, 24, 44 and 92): castellan's residue tracker, included by path, in one
//! process.
//!
//! Since DR-C (dramatis repo plan, D5, D11) Distillery opens no vault: djinn
//! releases the lane's derived keys, and the installed authority binds the
//! resident over them. The armed window covers rebuilding that release and
//! binding; after it, nothing may hold the master, live or freed uncleared.
//! The transport identity, the mesh author key since ruling 92, must still
//! be live: that shows the instrument sees the transport's key, and that the
//! transport is the released key and not the master.
//!
//! No libtest harness: the allocator is process-wide.

#[path = "../../castellan/tests/residue/mod.rs"]
mod residue;

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use distillery::mesh_host::HostConfig;
use distillery::{InstalledAuthority, ResidentSettings, RetentionSettings};
use insigne::DerivedKeyAttestation;
use mesh::{
    AvailabilityPolicy, ErasurePolicy, KeepBound, LeasePolicy, MESH_AUTHOR_SALT,
    MeshRetentionPolicy, MeshStore, PolicyRevision,
};
use personae::vault::ProfileId;
use personae::{
    Ed25519Keypair, Ed25519PublicKey, IdentityProvider, InMemoryProvider, RetainedKeys,
};
use residue::*;

const MESH: [u8; 32] = [0xD1; 32];

/// What djinn sends for the lane's salts: the master's public key, and each
/// salt's derived seed with its attestation. Computed before arming, as djinn
/// computes it in its own process.
struct Release {
    master: Ed25519PublicKey,
    keys: Vec<(Vec<u8>, [u8; 32], DerivedKeyAttestation)>,
}

fn provision(root: &Path, seed: [u8; 32]) -> Release {
    InstalledAuthority::configure(root, ProfileId("research".into())).unwrap();
    let persona = InMemoryProvider::from_seed(seed);
    Release {
        master: persona.master_public_key(),
        keys: InstalledAuthority::release_salts()
            .into_iter()
            .map(|salt| {
                let seed = persona.derive_keypair(&salt).unwrap().to_seed();
                let attestation = persona.attest_derived_key(&salt).unwrap();
                (salt, seed, attestation)
            })
            .collect(),
    }
}

async fn bind(
    root: &Path,
    release: Release,
) -> distillery::ResidentAuthority<muniment::RedbBackend> {
    let keys = RetainedKeys::from_released(
        release.master,
        release
            .keys
            .into_iter()
            .map(|(salt, seed, attestation)| (salt, Ed25519Keypair::from_seed(seed), attestation)),
    )
    .unwrap();
    let authority = InstalledAuthority::open(root, Arc::new(keys), "released by djinn").unwrap();
    let author = authority.mesh_author().unwrap();
    let policy = MeshRetentionPolicy {
        revision: PolicyRevision([0x41; 32]),
        checkpoint_authority: author.public_key().to_bytes(),
        availability: AvailabilityPolicy {
            promised_floor: KeepBound::Forever,
        },
        erasure: ErasurePolicy {
            privacy_ceiling: KeepBound::UntilCheckpoint,
            terminal_job_payload: mesh::PayloadRule::EraseTerminalAtCheckpoint,
        },
        lease: LeasePolicy { max_skew_ms: 0 },
    };
    drop(author);
    let paths = authority.paths(MESH);
    paths.prepare().unwrap();
    let store = MeshStore::at_path_with_retention(paths.mesh_store_path(), policy).unwrap();
    let settings = ResidentSettings {
        tick_every: Duration::from_secs(1),
        maintenance_every: None,
        blob_gc_every: Duration::from_secs(1),
        retention: RetentionSettings::default(),
    };
    authority
        .bind_resident(MESH, store, settings, |space| HostConfig::supervised(space))
        .await
        .unwrap()
}

fn main() {
    let master: [u8; 32] = canary_bytes(0);
    positive_control(&master);
    clear_canaries();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let release = provision(dir.path(), master);
    let transport = Ed25519Keypair::from_seed(master)
        .derive_child(MESH_AUTHOR_SALT)
        .to_seed();

    plant(0, "master seed", &master, true);
    plant(1, "transport seed (mesh author)", &transport, false);
    arm();
    phase(1);
    let root = dir.path().to_path_buf();
    // Spawned, so the task's frames are heap allocations the tracker sees.
    let task = runtime.spawn(async move { bind(&root, release).await });
    let resident = runtime.block_on(task).unwrap();
    phase(2);
    let (hits, overflow) = disarm();
    assert!(!overflow, "allocation table overflowed");

    let master_hits: Vec<_> = hits.iter().filter(|hit| hit.canary == 0).copied().collect();
    let transport_live = hits.iter().any(|hit| hit.canary == 1 && hit.live);
    let mut report = Report { failures: 0 };
    report.check(
        "bound lane holds no master",
        &["setup", "release and bind", "bound"],
        &master_hits,
        overflow,
    );
    println!(
        "transport seed live after bind: {}",
        if transport_live { "yes" } else { "no" }
    );

    // The lane still works without the vault: the host answers as the author.
    assert_eq!(
        resident.authority().host().me(),
        Ed25519Keypair::from_seed(transport).public_key().to_bytes(),
        "the bound host speaks as the mesh author"
    );
    runtime.block_on(resident.shutdown()).unwrap();
    clear_canaries();

    if report.failures > 0 || !transport_live {
        println!("transport-residue: FAILED");
        std::process::exit(1);
    }
    println!("transport-residue: ok");
}
