// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Real custody-wire import over disposable synthetic credentials: Djinn
//! reads the owner's source file and the custodian, which holds the
//! credential catalog, parses and commits it (custody isolation plan, H2).
#![cfg(all(unix, feature = "cxf"))]

mod custodian_fixture;

use custodian_fixture::{TestCustodian, manage_grants};
use djinn::custody_host::ResidentCustody;
use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use djinn::keeper::Keeper;
use graphshell_endpoint::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use graphshell_endpoint::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal};
use graphshell_endpoint::door::custody_client::{CustodyClient, CustodyClientError};
use personae::{PersonaId, ProfileId};
use serde_json::{Value, json};
use std::sync::Arc;

const PASSPHRASE: &str = "synthetic custody wire receipt";

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
async fn preview_commit_and_locked_refusal_go_through_the_custodians_catalog() {
    let custodian = TestCustodian::start(PASSPHRASE, &[("family", [0x45; 32])], manage_grants());
    let remote = custodian.remote();
    remote.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    let keeper = Arc::new(Keeper::new(remote.clone()));
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
    let dir = custodian.dir.path();
    let endpoint = dir.join("app.sock").display().to_string();
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
    let source = dir.join("credentials.cxf");
    std::fs::write(&source,serde_json::to_vec(&json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Account","email":"receipt@example.invalid","collections":[],"items":[{"id":"aQ","title":"Imported","credentials":[{"type":"generated-password","password":"synthetic-wire-secret-marker"}]}]}]})).unwrap()).unwrap();
    let preview = output(
        client
            .call(command("cxf-preview", vec![source.display().to_string()]))
            .await
            .unwrap(),
    );
    assert!(!preview.contains("synthetic-wire-secret-marker"));
    let preview: Value = serde_json::from_str(&preview).unwrap();
    let hash = preview["source_sha256"].as_str().unwrap().to_string();
    let choices = dir.join("destinations.json");
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
    let receipt = output(
        client
            .call(command("cxf-import", args(hash.clone())))
            .await
            .unwrap(),
    );
    assert!(!receipt.contains("synthetic-wire-secret-marker"));
    let receipt: Value = serde_json::from_str(&receipt).unwrap();
    assert_eq!(receipt["items"], 1);
    assert_eq!(receipt["delete_plaintext_source"], true);
    remote.lock_vault().unwrap();
    // A nonexistent source proves the Locked path refuses before file I/O.
    assert!(matches!(
        client
            .call(command(
                "cxf-preview",
                vec![dir.join("absent.cxf").display().to_string()]
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
