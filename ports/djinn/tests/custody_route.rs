// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The custody route, served as djinn serves it (dramatis repo plan, D5,
//! D11, D12): an application reads the identity, gets the derived keys the
//! policy names and nothing else, and an app the door does not know, or a
//! door that is not there, leaves the identity pending.

use std::sync::Arc;
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use djinn::keeper::Keeper;
use graphshell::identity::{VaultLockView, VaultProtectionView};
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::custody::{CustodyRefusal, KeySource};
use graphshell::native::custody_client::{CustodyClient, CustodyClientError};
use personae::{Ed25519Keypair, IdentityProvider, ProfileId};
use uuid::Uuid;

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-custody-receipt-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!("djinn-custody-receipt-{}.sock", Uuid::new_v4()))
        .display()
        .to_string();
}

fn resident_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("default".into()),
        "Default",
        Ed25519Keypair::from_seed([0x5c; 32]),
    );
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

async fn open(endpoint: &str, app: &str) -> Result<CustodyClient, CustodyClientError> {
    CustodyClient::open_at(endpoint, AppId::new(app)).await
}

#[tokio::test(flavor = "multi_thread")]
async fn an_app_reads_and_releases_through_djinn_and_nothing_else_leaves() {
    let host = resident_host();
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
    let endpoint = endpoint_name();
    {
        let (endpoint, keeper) = (endpoint.clone(), Arc::new(Keeper::new(Arc::clone(&host))));
        tokio::spawn(async move {
            let _ = serve_app_broker(
                &endpoint,
                keeper,
                grants,
                60_000,
                None,
                AppEndpointCatalog::default(),
            )
            .await;
        });
    }

    let mut client = None;
    for _ in 0..100 {
        if let Ok(opened) = open(&endpoint, "turnstone").await {
            client = Some(opened);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.expect("a first-party app is admitted to custody");

    let status = client.status().await.unwrap();
    assert_eq!(status.lock, VaultLockView::Unlocked);
    assert_eq!(status.profile, Some(ProfileId("default".into())));
    assert_eq!(
        status.persona_public_key,
        Some(host.master_public_key().to_bytes())
    );
    assert_eq!(
        status.wallet_public_key, None,
        "this resident holds no wallet"
    );

    let roster = client.roster().await.unwrap();
    assert_eq!(roster.chosen, ProfileId("default".into()));
    assert_eq!(roster.entries.len(), 1);

    // The released key is the vault's own derivation, and only it.
    let released = client
        .release(KeySource::Persona, vec![mesh::MESH_AUTHOR_SALT.to_vec()])
        .await
        .unwrap();
    assert_eq!(
        released
            .derive_keypair(mesh::MESH_AUTHOR_SALT)
            .unwrap()
            .to_seed(),
        host.derive_keypair(mesh::MESH_AUTHOR_SALT)
            .unwrap()
            .to_seed()
    );
    assert_eq!(released.master_public_key(), host.master_public_key());

    // The control: a salt outside the policy stays in djinn, and so does
    // anything from a wallet the resident does not have.
    for salt in [b"knot".to_vec(), Vec::new()] {
        assert!(matches!(
            client.release(KeySource::Persona, vec![salt]).await,
            Err(CustodyClientError::Refused(CustodyRefusal::NotReleasable))
        ));
    }
    assert!(matches!(
        client.attest(KeySource::Persona, b"knot".to_vec()).await,
        Err(CustodyClientError::Refused(CustodyRefusal::NotReleasable))
    ));
    let station = personae::reticulum::station_salts(b"device")[0].clone();
    assert!(matches!(
        client.release(KeySource::Wallet, vec![station]).await,
        Err(CustodyClientError::Refused(CustodyRefusal::NoWallet))
    ));

    // Signing happens inside djinn: the signature verifies under the derived
    // key, which never crossed.
    let session = graphshell::profile::session_salt("receipt");
    let (public_key, signature) = client
        .sign(
            KeySource::Persona,
            session.clone(),
            b"signed in djinn".to_vec(),
        )
        .await
        .unwrap();
    assert_eq!(
        public_key,
        host.derive_keypair(&session)
            .unwrap()
            .public_key()
            .to_bytes()
    );
    assert_eq!(signature.len(), 64);

    // An application the door does not know is refused before custody
    // answers, which the app shows as pending (D12).
    let unknown = open(&endpoint, "someone-elses-app").await;
    assert!(matches!(&unknown, Err(error) if error.is_pending()));
}

#[tokio::test]
async fn an_absent_djinn_is_pending_not_a_fallback() {
    let absent = open(&endpoint_name(), "turnstone").await;
    assert!(matches!(&absent, Err(error) if error.is_pending()));
}
