//! Explicit synthetic Djinn custody fixture. Never opens the user's vault.
use castellan::{
    authority::PersonaeHost,
    custody::{
        IdentityStorage, IdentityVault, Profile, SealedProfileStorage,
        wallet::bootstrap_wallet_state,
    },
};
use mere_resident::{
    door::app_broker::{AppEndpointCatalog, serve_app_broker},
    keeper::Keeper,
};
use djinn::door::app_admission::{AllowedAppRoutes, AppRouteGrants};
use ostiary::app_admission::AppId;
use castellan_client::custody_client::CustodyClient;
use dramatis::view::VaultProtectionView;
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
    mere_resident::custody::grant(&grants).unwrap();
    // Signalman, granted what it declares (`station_custody::declaration`),
    // and the receipt's other application, granted the whole Signalman
    // wallet namespace, which still does not reach a station's keys.
    {
        use ostiary::declaration::{
            AppDeclaration, CustodyOperation as Op, CustodyUse, NamePattern, SaltPattern, SaltUse,
        };
        let declared = |operations: Vec<Op>, namespaces: &[&str]| AppDeclaration {
            routes: vec![NamePattern::exact("custody")],
            custody: CustodyUse {
                operations: operations.into_iter().collect(),
                salts: namespaces
                    .iter()
                    .map(|namespace| SaltUse::wallet(SaltPattern::Prefix((*namespace).into())))
                    .collect(),
                ..CustodyUse::default()
            },
            ..AppDeclaration::default()
        };
        let signalman = declared(
            vec![Op::Status, Op::WatchLock, Op::Release, Op::IssueStationGrant, Op::RevokeDevice],
            &[
                "mere.castellan.reticulum.station/v1",
                "mere.castellan.reticulum.controller/v1",
                "mere/signalman/sited-station-control/v1/",
                "mere/signalman/sited-station-storage/v1/",
                "mere.signalman.tucket.station/v1/",
                "mere.signalman.sennet.station/v1/",
            ],
        );
        let broad = declared(vec![Op::Release, Op::Attest, Op::Sign], &["mere.signalman."]);
        mere_resident::applications::install(
            &grants,
            vec![
                mere_resident::applications::grant_for(&AppId::new("signalman"), &signalman, None),
                mere_resident::applications::grant_for(&AppId::new("turnstone"), &broad, None),
            ],
        );
    }
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
        if ostiary::local_endpoint::connect_local(&endpoint)
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
