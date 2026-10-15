// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Synthetic native/wire fixture. Never opens the installed user's vault.
use castellan::{
    authority::PersonaeHost,
    custody::{IdentityStorage, IdentityVault, Profile, SealedProfileStorage},
    resident::{CastellanResident, CredentialSalts},
};
use djinn::door::app_broker::{AppEndpointCatalog, serve_app_broker};
use djinn::{
    identity_ui::{NativeIdentityUi, SystemNativeIdentityUi},
    keeper::Keeper,
};
use graphshell::browser_carrier::NativeIdentityFailure;
use graphshell_endpoint::door::{
    app_admission::{AllowedAppRoutes, AppId, AppRouteGrants},
    custody_client::CustodyClient,
    identity::VaultProtectionView,
};
use personae::{Ed25519Keypair, PersonaId, ProfileId};
use serde_json::json;
use std::{collections::BTreeMap, path::PathBuf, sync::Arc, time::Duration};
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
        Ok(Some(zeroize::Zeroizing::new(
            "synthetic Dramatis receipt".into(),
        )))
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
    let storage = SealedProfileStorage::open_with_key(root.join("vault"), [0x73; 32]);
    storage
        .enroll_passphrase(b"synthetic Dramatis receipt")
        .unwrap();
    for (id, name, seed) in [("family", "Family", 0x71), ("work", "Work", 0x72)] {
        storage
            .save_profile(&Profile::new(
                ProfileId(id.into()),
                name,
                Ed25519Keypair::from_seed([seed; 32]),
            ))
            .unwrap();
    }
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::open(storage, &ProfileId("family".into())).unwrap(),
        None,
        VaultProtectionView::Passphrase,
    ));
    let salts = CredentialSalts {
        record: b"dramatis-receipt/records",
        freshness: b"dramatis-receipt/freshness",
    };
    let resident = CastellanResident::claim_derived(
        root.join("records"),
        root.join("freshness"),
        host.as_ref(),
        salts,
    )
    .unwrap();
    host.register_lock_holder(resident.lock_holder_for_profile(salts, ProfileId("family".into())));
    let persona = PersonaId::new();
    let source = json!({"version":{"major":1,"minor":0},"exporterRpId":"example.invalid","exporterDisplayName":"Receipt","timestamp":1,"accounts":[{"id":"YQ","username":"Receipt","email":"fixture@example.invalid","collections":[],"items":[
        {"id":"Yw","title":"Personal payment card","credentials":[{"type":"credit-card","number":{"fieldType":"concealed-string","value":"4111111111111111"}}]},
        {"id":"b3Rw","title":"Imported OTP with 11 digits","credentials":[{"type":"totp","secret":"MY======","algorithm":"sha1","period":30,"digits":11}]},
        {"id":"dW5r","title":"Credential from a newer format","credentials":[{"type":"future-credential","secret":"synthetic-never-render-this"}]}]}]});
    let choices = BTreeMap::from([(
        "YQ".into(),
        castellan::cxf::Destination {
            persona,
            profile: ProfileId("family".into()),
            ssh_unlock_policy: dramatis::intents::SshUnlockPolicyIntentV1::Session,
        },
    )]);
    castellan::cxf::Import::read(serde_json::to_vec(&source).unwrap())
        .unwrap()
        .commit(choices, &resident, &host)
        .unwrap();
    let ui: Arc<dyn NativeIdentityUi> = if mode == "native" {
        Arc::new(SystemNativeIdentityUi::default())
    } else {
        Arc::new(ReceiptUi)
    };
    let keeper = Arc::new(Keeper::new(host.clone()).with_ui(ui));
    keeper.attach_credentials(resident).unwrap();
    if mode == "locked" {
        host.lock_vault().unwrap();
    }
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    djinn::custody::grant(&grants).unwrap();
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
