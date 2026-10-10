// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Isolated receipt broker with the real djinn keeper. Never opens a user vault.

use castellan::{
    authority::PersonaeHost,
    custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage},
};
use djinn::keeper::Keeper;
use graphshell::{
    identity::VaultProtectionView,
    native::{
        app_admission::{AllowedAppRoutes, AppId, AppRouteGrants},
        app_broker::{AppEndpointCatalog, serve_app_broker},
        custody_client::CustodyClient,
    },
};
use personae::{Ed25519Keypair, ProfileId};
use std::{path::PathBuf, sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        args.len(),
        3,
        "usage: dr-c-receipt-host <endpoint> <new-root> <locked|unlocked>"
    );
    let endpoint = args[0].clone();
    let root = PathBuf::from(&args[1]);
    assert!(!root.exists(), "receipt root must be new");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join(".dr-c-generated"), b"isolated receipt vault").unwrap();
    let storage = SealedProfileStorage::open_with_key(root.join("vault"), [0x3d; 32]);
    storage
        .enroll_passphrase(b"dr-c synthetic receipt")
        .unwrap();
    let profile = Profile::new(
        ProfileId("default".into()),
        "Receipt",
        Ed25519Keypair::from_seed([0x5c; 32]),
    );
    storage.save_profile(&profile).unwrap();
    let vault = IdentityVault::open(storage, &profile.id).unwrap();
    let host = Arc::new(PersonaeHost::new(
        vault,
        None,
        VaultProtectionView::Passphrase,
    ));
    match args[2].as_str() {
        "locked" => host.lock_vault().unwrap(),
        "unlocked" => {},
        _ => panic!("mode must be locked or unlocked"),
    }
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
    let serving_endpoint = endpoint.clone();
    let server = tokio::spawn(async move {
        serve_app_broker(
            &serving_endpoint,
            Arc::new(Keeper::new(host)),
            grants,
            60_000,
            None,
            AppEndpointCatalog::default(),
        )
        .await
    });
    let mut ready = false;
    for _ in 0..200 {
        if CustodyClient::open_at(&endpoint, AppId::new("turnstone"))
            .await
            .is_ok()
        {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    assert!(ready, "isolated broker did not bind");
    std::fs::write(root.join("ready"), args[2].as_bytes()).unwrap();
    server.await.unwrap().unwrap();
}
