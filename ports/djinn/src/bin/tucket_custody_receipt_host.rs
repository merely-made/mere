//! Explicit synthetic Djinn custody fixture. Never opens the user's vault.
use castellan::{
    authority::PersonaeHost,
    custody::{
        IdentityStorage, IdentityVault, Profile, SealedProfileStorage,
        wallet::bootstrap_wallet_state,
    },
};
use djinn::{
    door::app_broker::{AppEndpointCatalog, serve_app_broker},
    keeper::Keeper,
};
use graphshell_endpoint::door::{
    app_admission::{AllowedAppRoutes, AppId, AppRouteGrants},
    custody_client::CustodyClient,
    identity::VaultProtectionView,
};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use std::{path::PathBuf, sync::Arc, time::Duration};

#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        args.len(),
        3,
        "usage: tucket-custody-receipt-host <endpoint> <new-root> <unlocked|locked|no-wallet>"
    );
    let endpoint = args[0].clone();
    let root = PathBuf::from(&args[1]);
    assert!(!root.exists(), "fixture root must be new");
    assert!(matches!(
        args[2].as_str(),
        "unlocked" | "locked" | "no-wallet"
    ));
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".tucket-custody-generated"),
        b"isolated synthetic Djinn fixture",
    )
    .unwrap();
    let storage = SealedProfileStorage::open_with_key(root.join("vault"), [0x3d; 32]);
    storage
        .enroll_passphrase(b"Tucket synthetic custody receipt")
        .unwrap();
    let profile = Profile::new(
        ProfileId("default".into()),
        "Tucket custody receipt",
        Ed25519Keypair::from_seed([0x5c; 32]),
    );
    storage.save_profile(&profile).unwrap();
    let wallet_root = root.join("wallet");
    let has_wallet = args[2] != "no-wallet";
    if has_wallet {
        bootstrap_wallet_state(&wallet_root, PersonaId::new(), "Tucket custody fixture").unwrap();
    }
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::open(storage, &profile.id).unwrap(),
        has_wallet.then(|| wallet_root.clone()),
        VaultProtectionView::Passphrase,
    ));
    if args[2] == "locked" {
        host.lock_vault().unwrap();
    }
    let keeper =
        Arc::new(Keeper::new(host.clone()).with_wallet_root(has_wallet.then_some(wallet_root)));
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
    let serving = endpoint.clone();
    let mut server = tokio::spawn(async move {
        serve_app_broker(
            &serving,
            keeper,
            grants,
            60_000,
            None,
            AppEndpointCatalog::default(),
        )
        .await
    });
    let mut ready = false;
    for _ in 0..200 {
        if CustodyClient::open_at(&endpoint, AppId::new("signalman"))
            .await
            .is_ok()
        {
            ready = true;
            break;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    assert!(ready, "custody broker did not bind");
    std::fs::write(root.join("ready"), args[2].as_bytes()).unwrap();
    let mut locked = args[2] == "locked";
    loop {
        tokio::select! {
            result = &mut server => { result.unwrap().unwrap(); break; },
            _ = tokio::time::sleep(Duration::from_millis(25)) => {
                if !locked && root.join("lock").is_file() { host.lock_vault().unwrap(); locked = true; }
                if root.join("stop").is_file() { server.abort(); let _ = server.await; break; }
            }
        }
    }
}
