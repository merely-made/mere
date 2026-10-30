// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Synthetic native/wire fixture. Never opens the installed user's vault.
//!
//! The credential catalog is the custodian's (custody isolation plan, H2),
//! so this host serves a fixture custodian from a thread over a new
//! synthetic vault and composes Djinn's keeper as its client.
#![cfg_attr(windows, allow(unused))]
use mere_resident::custody_host::{CredentialCall, ResidentCustody};
use mere_resident::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use mere_resident::door::browser_carrier::NativeIdentityFailure;
use mere_resident::{
    identity_ui::{NativeIdentityUi, SystemNativeIdentityUi},
    keeper::Keeper,
};
use djinn::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use djinn::door::custody_client::CustodyClient;
use personae::PersonaId;
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::Duration};
const PASSPHRASE: &str = "synthetic Dramatis receipt";
struct ReceiptUi;
impl NativeIdentityUi for ReceiptUi {
    fn pick_ssh_private_key(&self) -> Result<Option<PathBuf>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn prompt_ssh_private_key_passphrase(
        &self,
    ) -> Result<Option<zeroize::Zeroizing<String>>, NativeIdentityFailure> {
        Err(NativeIdentityFailure::UiUnavailable)
    }
    fn prompt_vault_passphrase(
        &self,
    ) -> Result<Option<zeroize::Zeroizing<String>>, NativeIdentityFailure> {
        Ok(Some(zeroize::Zeroizing::new(PASSPHRASE.into())))
    }
    fn confirm_quarantine(
        &self,
        _profile: &str,
        _title: &str,
        _action: &str,
    ) -> Result<bool, NativeIdentityFailure> {
        Ok(false)
    }
}
#[cfg(windows)]
fn main() {
    eprintln!("dramatis-review-host serves its fixture custodian on a Unix socket only");
    std::process::exit(2);
}
#[cfg(unix)]
#[tokio::main]
async fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    assert_eq!(
        args.len(),
        3,
        "usage: dramatis-review-host <endpoint> <new-root> <unlocked|locked|native>"
    );
    let endpoint = args[0].clone();
    let root = PathBuf::from(&args[1]);
    let mode = &args[2];
    assert!(matches!(mode.as_str(), "unlocked" | "locked" | "native"));
    assert!(!root.exists(), "fixture root must be new");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(
        root.join(".dramatis-generated"),
        "synthetic Dramatis custody fixture",
    )
    .unwrap();
    let fixture = djinn_custodian::fixture::start(
        &root.join("custodian"),
        djinn_custodian::fixture::Spec {
            passphrase: PASSPHRASE,
            profiles: &[("family", "Family", [0x71; 32]), ("work", "Work", [0x72; 32])],
            grants: json!([
                {"operation": "status"}, {"operation": "lock"}, {"operation": "unlock"}, {"operation": "snapshot"},
                {"operation": "manage", "consent": "none"}, {"operation": "credentials", "consent": "none"}
            ]),
            consent: "deny",
            mechanism: None,
            agent: false,
        },
    )
    .unwrap();
    let installed = mere_resident::custodian::InstalledCustodian::from_path(&fixture.client_file, false).unwrap();
    let host = mere_resident::custody_host::remote::RemoteHost::connect(installed).unwrap();
    host.unlock_with_passphrase(PASSPHRASE.as_bytes()).unwrap();
    let persona = PersonaId::new();
    let source = json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Receipt","email":"fixture@example.invalid","collections":[],"items":[
        {"id":"Yw","title":"Personal payment card","credentials":[{"type":"credit-card","number":{"fieldType":"concealed-string","value":"4111111111111111"}}]},
        {"id":"b3Rw","title":"Imported OTP with 11 digits","credentials":[{"type":"totp","secret":"MY======","algorithm":"sha1","period":30,"digits":11}]},
        {"id":"dW5r","title":"Credential from a newer format","credentials":[{"type":"future-credential","secret":"synthetic-never-render-this"}]}]}]});
    let source = serde_json::to_vec(&source).unwrap();
    let preview = host.credentials(CredentialCall::CxfPreview(source.clone())).unwrap();
    host.credentials(CredentialCall::CxfImport {
        source,
        destinations: json!({"YQ": {"persona": persona, "profile": "family"}}),
        preview_sha256: preview["source_sha256"].as_str().unwrap().into(),
    })
    .unwrap();
    let ui: Arc<dyn NativeIdentityUi> = if mode == "native" {
        Arc::new(SystemNativeIdentityUi::default())
    } else {
        Arc::new(ReceiptUi)
    };
    let keeper = Arc::new(Keeper::new(host.clone()).with_ui(ui));
    if mode == "locked" {
        host.lock_vault().unwrap();
    }
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    mere_resident::custody::grant(&grants).unwrap();
    let serving = endpoint.clone();
    let server = tokio::spawn(async move {
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
    for _ in 0..200 {
        if CustodyClient::open_at(&endpoint, AppId::new("dramatis"))
            .await
            .is_ok()
        {
            std::fs::write(root.join("ready"), mode).unwrap();
            server.await.unwrap().unwrap();
            return;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("fixture broker did not bind");
}
