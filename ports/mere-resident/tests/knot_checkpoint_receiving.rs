// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

#![cfg(unix)]

//! Djinn custody and its existing resident route receive Knot's Unix legacy repair.

use std::path::{Path, PathBuf};

use chirograph::{
    EditableTextV1, IntentInvocation, IntentResult, PresentationCodec, ResourceRequest, SaveTextV1,
};
use mere_resident::resident_blobs::ResidentBlobCustody;
use mere_resident::resident_knot::{RESIDENT_KNOT_ROUTE, ResidentKnot};
use mere_resident::settings::{KnotResidentSettings, ResidentContentSettings};
use graphshell_endpoint::{IntentSink, PresentationSource, ProjectionCatalog, ProjectionSource};
use graphshell_host::endpoint_catalog::ResidentEndpointCatalog;
use graphshell_host::lifecycle::AdmittedEndpointContext;
use knot_editor::{
    KnotVault, PersonalVaultKeys, StartupUnlockedPersonalVault, VaultDocument, persona_vault_root,
};
use muniment::Backend;
use p2panda_core::Hash;
use personae::PersonaId;
use zeroize::{Zeroize, Zeroizing};

const DEVICE: &str = "checkpoint-receiving-fixture";
const DOCUMENT: &str = "checkpoint-private-note";
const TITLE: &str = "Djinn private checkpoint title 717b330a";
const LEGACY_BODY: &str = "# Private legacy source e046cc49\n";
const CURRENT_BODY: &str = "# Current signed source b3b9b465\n";
const CHILD_BODY: &str = "# Saved through reopened Djinn route a65bb21d\n";

fn persona() -> PersonaId {
    PersonaId::from_uuid(uuid::Uuid::from_u128(0xc022_2026_1022))
}

fn keys(root: &Path) -> (PersonalVaultKeys, Zeroizing<[u8; 32]>) {
    let mut released = mere_resident::custody::epoch_keys(
        &mere_resident::custody::ReleasePolicy::none()
            .with_declared_epochs(&knot_editor::declaration::app_declaration().custody.epoch),
        root,
        persona(),
        Some(DEVICE),
        &PersonalVaultKeys::requests(),
    )
    .unwrap();
    let vault_key = Zeroizing::new(
        released
            .iter()
            .find(|key| key.request.context == knot_editor::VAULT_KEY_CONTEXT)
            .unwrap()
            .key,
    );
    let keys = PersonalVaultKeys::from_released(&released).unwrap();
    for release in &mut released {
        release.key.zeroize();
    }
    (keys, vault_key)
}

fn document(body: &str) -> VaultDocument {
    VaultDocument {
        id: DOCUMENT.into(),
        title: TITLE.into(),
        body: body.as_bytes().to_vec(),
        media_type: "text/djot".into(),
    }
}

fn database(root: &Path) -> PathBuf {
    persona_vault_root(root, persona()).join("knot/sync.redb")
}

fn assert_sealed(path: &Path) {
    let bytes = std::fs::read(path).unwrap();
    for secret in [TITLE, LEGACY_BODY, CURRENT_BODY, CHILD_BODY] {
        assert!(
            !bytes
                .windows(secret.len())
                .any(|window| window == secret.as_bytes()),
            "plaintext source in active sync database"
        );
        let json_body = serde_json::to_vec(secret.as_bytes()).unwrap();
        assert!(
            !bytes
                .windows(json_body.len())
                .any(|window| window == json_body),
            "plaintext JSON byte array in active sync database"
        );
    }
}

/// Open the normal admitted route and, optionally, perform its ordinary Save.
fn visit(resident: &ResidentKnot, replacement: Option<&str>) -> String {
    let mut catalog = ResidentEndpointCatalog::new();
    resident.register(&mut catalog).unwrap();
    let context = AdmittedEndpointContext::new(
        chirograph::ProjectionSession("checkpoint:receiving".into()),
        [0x74; 32],
    );
    let mut endpoint = catalog.open(RESIDENT_KNOT_ROUTE, &context).unwrap();
    let request = endpoint.describe().projections.remove(0).request;
    let snapshot = endpoint.snapshot(request).unwrap();
    for (instance, _) in snapshot.scene.active_items_in_order() {
        let offers = snapshot.presentation.offers_for(instance).unwrap();
        let Some(offer) = offers
            .iter()
            .find(|offer| offer.codec == PresentationCodec::EditableTextV1)
        else {
            continue;
        };
        let response = endpoint
            .resource(ResourceRequest {
                session: snapshot.session.clone(),
                resource: offer.resource,
            })
            .unwrap();
        let editable: EditableTextV1 = serde_json::from_slice(&response.bytes).unwrap();
        if !editable.address.ends_with(DOCUMENT) {
            continue;
        }
        if let Some(source) = replacement {
            let action = &offer.semantics.actions[0];
            let result = endpoint
                .invoke(IntentInvocation {
                    session: snapshot.session.clone(),
                    target: instance,
                    observed_epoch: snapshot.scene.epoch,
                    observed_revision: snapshot.scene.revision,
                    intent: action.intent.0.clone(),
                    payload: serde_json::to_vec(&SaveTextV1 {
                        base_token: editable.base_token.clone(),
                        source: source.into(),
                    })
                    .unwrap(),
                })
                .unwrap();
            assert_eq!(result, IntentResult::Accepted);
        }
        return editable.source;
    }
    panic!("resident route did not disclose the signed fixture document");
}

async fn receive_through_owner(root: &Path, replacement: Option<&str>) -> String {
    let custody = ResidentBlobCustody::open(root, &ResidentContentSettings::default())
        .await
        .unwrap();
    let epoch_root = root.to_path_buf();
    let resident = ResidentKnot::open(
        root,
        KnotResidentSettings {
            persona: persona().as_uuid().to_string(),
            device_label: DEVICE.into(),
            ..Default::default()
        },
        custody.clone(),
        std::sync::Arc::new(move |persona, label: &str, requests: &[_]| {
            mere_resident::custody::epoch_keys(&mere_resident::custody::ReleasePolicy::none()
            .with_declared_epochs(&knot_editor::declaration::app_declaration().custody.epoch), &epoch_root, persona, Some(label), requests).map_err(|r| format!("{r:?}"))
        }),
    )
    .await
    .unwrap();
    assert!(
        !resident.sync_enabled(),
        "fixture does not admit network replication"
    );
    let source = visit(&resident, replacement);
    resident.close().await.unwrap();
    custody.shutdown().await.unwrap();
    source
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn resident_startup_repairs_legacy_checkpoint_and_fresh_process_continues_signed_author() {
    let root = tempfile::tempdir().unwrap();
    castellan::custody::wallet::ensure_wallet_state(root.path(), persona(), DEVICE).unwrap();
    let (released, vault_key) = keys(root.path());
    let startup = StartupUnlockedPersonalVault::open(root.path(), persona(), released, []).unwrap();
    startup.author_document(document(LEGACY_BODY)).unwrap();
    let vault = KnotVault::open(persona_vault_root(root.path(), persona()), *vault_key).unwrap();
    let store = startup.store().clone();
    let previous = store.save_checkpoint(&vault).await.unwrap();
    let previous_digest = *blake3::hash(&serde_json::to_vec(&previous).unwrap()).as_bytes();
    startup.author_document(document(CURRENT_BODY)).unwrap();
    let projection = store.projection(&vault).await.unwrap();
    let current_head = projection.document_heads[DOCUMENT];
    let current = store
        .sync_store()
        .get_operation(&Hash::from(current_head))
        .await
        .unwrap()
        .unwrap();
    let current_seq = current.header.seq_num;

    // Actual earlier checkpoint shape through the existing opaque backend;
    // its stale frontier must not replace the later signed source above.
    let raw = store.sync_store();
    let suffix = knot_editor::hex32(&store.space_id());
    raw.backend()
        .put(
            &format!("knot-sync/checkpoint/{suffix}"),
            &serde_json::to_vec(&previous).unwrap(),
        )
        .await
        .unwrap();
    raw.backend()
        .delete(&format!("knot-sync/sealed-storage/{suffix}"))
        .await
        .unwrap();
    assert!(store.load_checkpoint(&vault).await.is_err());
    drop(raw);
    drop(store);
    drop(vault);
    drop(startup);
    let before = std::fs::read(database(root.path())).unwrap();
    assert!(
        before
            .windows(TITLE.len())
            .any(|window| window == TITLE.as_bytes()),
        "legacy plaintext before-control"
    );

    assert_eq!(receive_through_owner(root.path(), None).await, CURRENT_BODY);
    assert_sealed(&database(root.path()));
    let (released, vault_key) = keys(root.path());
    let authority =
        StartupUnlockedPersonalVault::open(root.path(), persona(), released, []).unwrap();
    assert!(
        !authority.checkpoint_repair().rewritten,
        "the owner already completed storage replacement"
    );
    let vault = KnotVault::open(persona_vault_root(root.path(), persona()), *vault_key).unwrap();
    let repaired = authority
        .store()
        .load_checkpoint(&vault)
        .await
        .unwrap()
        .unwrap();
    assert_ne!(
        *blake3::hash(&serde_json::to_vec(&repaired).unwrap()).as_bytes(),
        previous_digest
    );
    assert_eq!(
        repaired.snapshot.as_ref().unwrap().document_heads[DOCUMENT],
        current_head
    );
    assert_eq!(
        authority.store().projection(&vault).await.unwrap(),
        projection
    );
    drop(vault);
    drop(authority);

    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--ignored",
            "--exact",
            "checkpoint_receiving_child_process",
            "--nocapture",
        ])
        .env("DJINN_KNOT_CHECKPOINT_RECEIVING_ROOT", root.path())
        .env(
            "DJINN_KNOT_CHECKPOINT_PUBLIC_HEAD",
            serde_json::to_string(&(current_head, current_seq)).unwrap(),
        )
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "child stdout: {}\nchild stderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains(
        "Djinn checkpoint receiving child verified owner reopen and signed continuation"
    ));
    assert_sealed(&database(root.path()));
}

#[test]
#[ignore = "launched by the receiving test in a fresh process"]
fn checkpoint_receiving_child_process() {
    let root = PathBuf::from(
        std::env::var_os("DJINN_KNOT_CHECKPOINT_RECEIVING_ROOT").expect("receiving root"),
    );
    let (previous, seq): ([u8; 32], u32) = serde_json::from_str(
        &std::env::var("DJINN_KNOT_CHECKPOINT_PUBLIC_HEAD").expect("public signed frontier"),
    )
    .unwrap();
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            assert_eq!(
                receive_through_owner(&root, Some(CHILD_BODY)).await,
                CURRENT_BODY
            );
            let (released, vault_key) = keys(&root);
            let authority =
                StartupUnlockedPersonalVault::open(&root, persona(), released, []).unwrap();
            assert!(!authority.checkpoint_repair().rewritten);
            let vault = KnotVault::open(persona_vault_root(&root, persona()), *vault_key).unwrap();
            let projection = authority.store().projection(&vault).await.unwrap();
            assert_eq!(projection.documents[0].body, CHILD_BODY.as_bytes());
            let next_head = projection.document_heads[DOCUMENT];
            let next = authority
                .store()
                .sync_store()
                .get_operation(&Hash::from(next_head))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(next.header.seq_num, seq + 1);
            assert_eq!(
                next.header.backlink.as_ref().map(|hash| *hash.as_bytes()),
                Some(previous)
            );
            println!(
                "Djinn checkpoint receiving child verified owner reopen and signed continuation"
            );
        });
}
