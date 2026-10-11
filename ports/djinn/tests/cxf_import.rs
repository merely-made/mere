// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real custody-wire import over disposable synthetic credentials.
#![cfg(feature = "cxf")]

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage};
use castellan::resident::{CastellanResident, CredentialSalts};
use djinn::keeper::Keeper;
use graphshell::identity::VaultProtectionView;
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::custody::{CustodyAnswer, CustodyCall, CustodyRefusal};
use graphshell::native::custody_client::{CustodyClient, CustodyClientError};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use serde_json::{Value, json};
use std::sync::Arc;

fn endpoint() -> String {
    #[cfg(windows)]
    {
        format!(r"\\.\pipe\djinn-cxf-{}", uuid::Uuid::new_v4())
    }
    #[cfg(not(windows))]
    {
        std::env::temp_dir()
            .join(format!("djinn-cxf-{}.sock", uuid::Uuid::new_v4()))
            .display()
            .to_string()
    }
}
fn command(name: &str, args: Vec<String>) -> CustodyCall {
    CustodyCall::VaultCommand {
        profile: ProfileId("family".into()),
        command: name.into(),
        args,
    }
}
fn output(answer: CustodyAnswer) -> String {
    match answer {
        CustodyAnswer::VaultOutput { text } => text,
        _ => panic!("expected public vault output"),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn preview_commit_and_locked_refusal_use_the_residents_existing_authority() {
    let dir = tempfile::tempdir().unwrap();
    let storage = SealedProfileStorage::open_with_key(dir.path().join("vault"), [0x55; 32]);
    storage
        .enroll_passphrase(b"synthetic custody wire receipt")
        .unwrap();
    let profile = Profile::new(
        ProfileId("family".into()),
        "Family",
        Ed25519Keypair::from_seed([0x45; 32]),
    );
    storage.save_profile(&profile).unwrap();
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::OsProtected,
    ));
    let salts = CredentialSalts {
        record: b"cxf-receipt/records",
        freshness: b"cxf-receipt/freshness",
    };
    let resident = CastellanResident::claim_derived(
        dir.path().join("records"),
        dir.path().join("freshness"),
        host.as_ref(),
        salts,
    )
    .unwrap();
    host.register_lock_holder(resident.lock_holder(salts));
    let keeper = Arc::new(Keeper::new(Arc::clone(&host)));
    keeper.attach_credentials(resident.clone()).unwrap();
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
    let endpoint = endpoint();
    let server_endpoint = endpoint.clone();
    let server = tokio::spawn(async move {
        serve_app_broker(
            &server_endpoint,
            keeper,
            grants,
            60_000,
            None,
            AppEndpointCatalog::default(),
        )
        .await
    });
    let mut client = None;
    for _ in 0..100 {
        if let Ok(opened) = CustodyClient::open_at(&endpoint, AppId::new("personae-vault")).await {
            client = Some(opened);
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    }
    let mut client = client.expect("custody broker opens");
    let persona = PersonaId::new();
    let source = dir.path().join("credentials.cxf");
    std::fs::write(&source,serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Account","email":"receipt@example.invalid","collections":[],"items":[{"id":"aQ","title":"Imported","credentials":[{"type":"generated-password","password":"synthetic-wire-secret-marker"}]}]}]})).unwrap()).unwrap();
    let preview = output(
        client
            .call(command("cxf-preview", vec![source.display().to_string()]))
            .await
            .unwrap(),
    );
    assert!(!preview.contains("synthetic-wire-secret-marker"));
    assert!(resident.items(persona).list().unwrap().is_empty());
    let preview: Value = serde_json::from_str(&preview).unwrap();
    let hash = preview["source_sha256"].as_str().unwrap().to_string();
    let choices = dir.path().join("destinations.json");
    std::fs::write(
        &choices,
        serde_json::to_vec(&json!({"YQ":{"persona":persona,"profile":"family"}})).unwrap(),
    )
    .unwrap();
    let args = |hash: String| {
        vec![
            source.display().to_string(),
            choices.display().to_string(),
            hash,
        ]
    };
    let mut ordinary = CustodyClient::open_at(&endpoint, AppId::new("turnstone"))
        .await
        .unwrap();
    assert!(matches!(
        ordinary
            .call(command("cxf-import", args(hash.clone())))
            .await,
        Err(CustodyClientError::Refused(CustodyRefusal::NotServed))
    ));
    assert!(
        client
            .call(command("cxf-import", args("changed-source-hash".into())))
            .await
            .is_err()
    );
    assert!(resident.items(persona).list().unwrap().is_empty());
    let receipt = output(
        client
            .call(command("cxf-import", args(hash)))
            .await
            .unwrap(),
    );
    assert!(!receipt.contains("synthetic-wire-secret-marker"));
    let receipt: Value = serde_json::from_str(&receipt).unwrap();
    assert_eq!(receipt["items"], 1);
    assert_eq!(receipt["delete_plaintext_source"], true);
    assert_eq!(resident.items(persona).list().unwrap().len(), 1);
    host.lock_vault().unwrap();
    assert!(resident.is_locked());
    // A nonexistent source proves the Locked path refuses before file I/O.
    assert!(matches!(
        client
            .call(command(
                "cxf-preview",
                vec![dir.path().join("absent.cxf").display().to_string()]
            ))
            .await,
        Err(CustodyClientError::Refused(CustodyRefusal::Locked))
    ));
    assert_eq!(
        client.roster().await.unwrap().chosen,
        ProfileId("family".into())
    );
    server.abort();
    let _ = server.await;
}
