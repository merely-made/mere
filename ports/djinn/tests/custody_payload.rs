// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! Actual admitted IPC with a wallet epoch. No keys cross the custody wire.
use castellan::authority::PersonaeHost;
use castellan::custody::{
    IdentityStorage, IdentityVault, InMemoryStorage, Profile, SealedProfileStorage,
};
use mere_resident::{
    door::app_broker::{AppEndpointCatalog, serve_app_broker},
    keeper::Keeper,
};
use eidetic::{Hash, PayloadSealer};
use djinn::door::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants};
use djinn::door::custody::{CustodyAnswer, CustodyCall, CustodyRefusal};
use djinn::door::custody_client::{CustodyClient, CustodyClientError};
use djinn::door::custody_identity::CustodyIdentity;
use djinn::door::custody_payload::{
        MAX_PAYLOAD_BYTES, PayloadBytes, PayloadOperation, PayloadRequest, PayloadResult,
    };
use dramatis::view::{VaultLockView, VaultProtectionView};
use personae::{Ed25519Keypair, IdentityProvider, PersonaId, ProfileId};
use std::{sync::Arc, time::Duration};

#[tokio::test(flavor = "multi_thread")]
async fn private_payloads_use_bound_custody_and_survive_epoch_rotation() {
    let root = tempfile::tempdir().unwrap();
    let persona = PersonaId::new();
    castellan::custody::wallet::ensure_wallet_state(root.path(), persona, "Retained fixture")
        .unwrap();
    let profile = Profile::new(
        ProfileId("family".into()),
        "Family",
        Ed25519Keypair::from_seed([0x51; 32]),
    );
    let storage = SealedProfileStorage::open_with_key(root.path().join("vault"), [0x62; 32]);
    storage
        .enroll_passphrase(b"isolated retained-content fixture")
        .unwrap();
    storage.save_profile(&profile).unwrap();
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::OsProtected,
    ));
    let keeper = Arc::new(Keeper::new(host.clone()));
    keeper
        .configure_payload_custody(
            ProfileId("family".into()),
            persona,
            root.path().to_path_buf(),
        )
        .unwrap();
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    mere_resident::custody::grant(&grants).unwrap();
    #[cfg(unix)]
    let endpoint = root
        .path()
        .join("custody.sock")
        .to_string_lossy()
        .to_string();
    #[cfg(windows)]
    let endpoint = format!(r"\\.\pipe\retained-custody-{}", uuid::Uuid::new_v4());
    let server = tokio::spawn({
        let endpoint = endpoint.clone();
        async move {
            serve_app_broker(
                &endpoint,
                keeper,
                grants,
                60_000,
                None,
                AppEndpointCatalog::default(),
            )
            .await
        }
    });
    let mut client = None;
    for _ in 0..100 {
        if let Ok(opened) = CustodyClient::open_at(&endpoint, AppId::new("turnstone")).await {
            client = Some(opened);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.expect("custody starts");
    let clear = b"private retained cafe passage";
    let request = PayloadRequest {
        custody_epoch: None,
        profile: ProfileId("family".into()),
        master: host.master_public_key().to_bytes(),
        content_hash: Hash::of(clear),
        operation: PayloadOperation::Seal {
            bytes: PayloadBytes(clear.to_vec()),
        },
    };
    let CustodyAnswer::Payload {
        result: PayloadResult::Sealed { bytes, marker },
    } = client
        .call(CustodyCall::Payload(request.clone()))
        .await
        .unwrap()
    else {
        panic!("sealed response expected")
    };
    assert_ne!(bytes.0, clear);
    // Both product hosts retain private sources through custody. They use
    // distinct authenticated app scopes even under the same wallet/profile.
    let mut graphshell = CustodyClient::open_at(&endpoint, AppId::new("graphshell"))
        .await
        .unwrap();
    let CustodyAnswer::Payload {
        result:
            PayloadResult::Sealed {
                bytes: graphshell_bytes,
                marker: graphshell_marker,
            },
    } = graphshell
        .call(CustodyCall::Payload(request.clone()))
        .await
        .expect("Graphshell has its own retained-content custody scope")
    else {
        panic!("Graphshell sealed response expected")
    };
    let graphshell_read = PayloadRequest {
        operation: PayloadOperation::Unseal {
            bytes: graphshell_bytes,
            marker: graphshell_marker,
        },
        ..request.clone()
    };
    assert!(matches!(
        graphshell.call(CustodyCall::Payload(graphshell_read.clone())).await.unwrap(),
        CustodyAnswer::Payload { result: PayloadResult::Clear { bytes } } if bytes.0 == clear
    ));
    assert!(
        client
            .call(CustodyCall::Payload(graphshell_read))
            .await
            .is_err(),
        "Turnstone cannot unseal Graphshell's retained bytes"
    );
    let turnstone_read = PayloadRequest {
        operation: PayloadOperation::Unseal {
            bytes: bytes.clone(),
            marker: marker.clone(),
        },
        ..request.clone()
    };
    assert!(
        graphshell
            .call(CustodyCall::Payload(turnstone_read))
            .await
            .is_err(),
        "Graphshell cannot unseal Turnstone's retained bytes"
    );
    let ordinary_wallet_sealer =
        castellan::custody::wallet::epoch_sealer_for_persona(root.path(), persona)
            .unwrap()
            .unwrap();
    assert!(
        ordinary_wallet_sealer
            .unseal(&Hash::of(clear), &marker, &bytes.0)
            .is_err(),
        "retained-content ciphertext is bound to its app/profile lane"
    );
    // Compatibility input uses the pre-existing Turnstone AAD and the actual
    // wallet cipher independently of the broker's current scope selection.
    let legacy_scope = serde_json::to_vec(&(
        "turnstone.retained-clips/custody-v1",
        "family",
        &request.master,
    ))
    .unwrap();
    let (legacy_bytes, legacy_marker) = ordinary_wallet_sealer
        .with_scope(legacy_scope)
        .seal(&Hash::of(clear), clear)
        .unwrap();
    let legacy_read = PayloadRequest {
        operation: PayloadOperation::Unseal {
            bytes: PayloadBytes(legacy_bytes),
            marker: legacy_marker,
        },
        ..request.clone()
    };
    assert!(
        matches!(client.call(CustodyCall::Payload(legacy_read)).await.unwrap(),
        CustodyAnswer::Payload { result: PayloadResult::Clear { bytes } } if bytes.0 == clear),
        "existing Turnstone-scoped ciphertext remains readable"
    );
    let sealed_epoch = marker.epoch;
    let read = PayloadRequest {
        custody_epoch: None,
        operation: PayloadOperation::Unseal {
            bytes: bytes.clone(),
            marker: marker.clone(),
        },
        ..request.clone()
    };
    assert!(
        matches!(client.call(CustodyCall::Payload(read.clone())).await.unwrap(), CustodyAnswer::Payload { result: PayloadResult::Clear { bytes } } if bytes.0 == clear)
    );
    let mut other = CustodyClient::open_at(&endpoint, AppId::new("woodshed"))
        .await
        .unwrap();
    assert!(matches!(
        other.call(CustodyCall::Payload(read.clone())).await,
        Err(CustodyClientError::Refused(CustodyRefusal::NotServed))
    ));
    for bad in [
        PayloadRequest {
            custody_epoch: Some(djinn::door::custody::CustodyEpoch {
                instance: "a-different-custodian".into(),
                generation: 1,
            }),
            ..read.clone()
        },
        PayloadRequest {
            custody_epoch: None,
            profile: ProfileId("foreign".into()),
            ..read.clone()
        },
        PayloadRequest {
            custody_epoch: None,
            master: [3; 32],
            ..read.clone()
        },
        PayloadRequest {
            custody_epoch: None,
            content_hash: Hash::of(b"wrong"),
            ..read.clone()
        },
        PayloadRequest {
            custody_epoch: None,
            operation: PayloadOperation::Unseal {
                bytes: PayloadBytes(b"altered".to_vec()),
                marker: marker.clone(),
            },
            ..read.clone()
        },
        PayloadRequest {
            custody_epoch: None,
            operation: PayloadOperation::Seal {
                bytes: PayloadBytes(vec![0; MAX_PAYLOAD_BYTES + 1]),
            },
            ..request.clone()
        },
    ] {
        assert!(client.call(CustodyCall::Payload(bad)).await.is_err());
    }
    // Reconnect through the actual synchronous identity adapter on its actor.
    let identity_endpoint = endpoint.clone();
    let reader = tokio::task::spawn_blocking(move || {
        CustodyIdentity::connect_at(&identity_endpoint, "turnstone").unwrap()
    })
    .await
    .unwrap();
    let mut wallet = pandect::wallet_store::load_persona_wallet(root.path(), persona)
        .unwrap()
        .unwrap();
    let previous_head = wallet.private_epoch_head;
    wallet.private_epoch_head = pandect::KeyEpochId::new();
    pandect::wallet_store::save_persona_wallet(root.path(), &wallet).unwrap();
    assert!(matches!(
        client.call(CustodyCall::Payload(request.clone())).await,
        Err(CustodyClientError::Refused(CustodyRefusal::Locked))
    ));
    assert!(
        matches!(client.call(CustodyCall::Payload(read.clone())).await.unwrap(), CustodyAnswer::Payload { result: PayloadResult::Clear { bytes } } if bytes.0 == clear),
        "historical reads do not require an available writing head"
    );
    wallet.private_epoch_head = previous_head;
    pandect::wallet_store::save_persona_wallet(root.path(), &wallet).unwrap();
    wallet.private_epoch_head = pandect::KeyEpochId::new();
    castellan::custody::wallet::stage_persona_private_epoch(
        root.path(),
        persona,
        wallet.private_epoch_head,
        b"rotated retained fixture epoch",
    )
    .unwrap();
    pandect::wallet_store::save_persona_wallet(root.path(), &wallet).unwrap();
    let (reader, new_marker) = tokio::task::spawn_blocking(move || {
        assert_eq!(
            reader.unseal(&Hash::of(clear), &marker, &bytes.0).unwrap(),
            clear
        );
        let (_, new_marker) = reader.seal(&Hash::of(clear), clear).unwrap();
        (reader, new_marker)
    })
    .await
    .unwrap();
    assert_ne!(new_marker.epoch, sealed_epoch);
    host.lock_vault().unwrap();
    // A fresh custody connection must not need projection admission or door
    // keys after the vault locks. It may read status, but cannot release data.
    let mut locked_client = CustodyClient::open_at(&endpoint, AppId::new("turnstone"))
        .await
        .expect("custody remains reachable while locked");
    assert_eq!(
        locked_client.status().await.unwrap().lock,
        VaultLockView::Locked
    );
    assert!(matches!(
        locked_client.call(CustodyCall::Payload(read.clone())).await,
        Err(CustodyClientError::Refused(CustodyRefusal::Locked))
    ));
    assert!(matches!(
        client.call(CustodyCall::Payload(read)).await,
        Err(CustodyClientError::Refused(CustodyRefusal::Locked))
    ));
    tokio::task::spawn_blocking(move || assert!(reader.seal(&Hash::of(clear), clear).is_err()))
        .await
        .unwrap();
    server.abort();
}

/// Explicit process fixture only. The caller owns and removes this marked root.
#[test]
#[ignore]
fn retained_custody_receipt_host() {
    let root = std::path::PathBuf::from(
        std::env::var_os("EIDETIC_CUSTODY_FIXTURE_ROOT").expect("fixture root"),
    );
    assert_eq!(
        std::fs::read_to_string(root.join("fixture-marker")).unwrap(),
        "eidetic-retention-20261019\n"
    );
    let persona = PersonaId::from_uuid(uuid::Uuid::from_u128(0xe1de_2026_1019));
    castellan::custody::wallet::ensure_wallet_state(&root, persona, "Retained process fixture")
        .unwrap();
    let profile = Profile::new(
        ProfileId("family".into()),
        "Family",
        Ed25519Keypair::from_seed([0x51; 32]),
    );
    let storage = InMemoryStorage::new();
    storage.save_profile(&profile).unwrap();
    let host = Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(storage, profile),
        None,
        VaultProtectionView::Ephemeral,
    ));
    let keeper = Arc::new(Keeper::new(host));
    keeper
        .configure_payload_custody(ProfileId("family".into()), persona, root.clone())
        .unwrap();
    let grants = AppRouteGrants::new(AllowedAppRoutes::none());
    mere_resident::custody::grant(&grants).unwrap();
    let endpoint = std::env::var("EIDETIC_CUSTODY_FIXTURE_ENDPOINT").expect("fixture endpoint");
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(serve_app_broker(
            &endpoint,
            keeper,
            grants,
            60_000,
            None,
            AppEndpointCatalog::default(),
        ))
        .unwrap();
}
