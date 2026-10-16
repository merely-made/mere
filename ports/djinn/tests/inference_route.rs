// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The resident inference lane through Djinn's application door (resident
//! inference and recall plan, I1): a resident with the lane on answers an ask
//! over `inference.ask` to Turnstone's label alone, and shuts the lane down;
//! settings naming the lane without a Distillery lane are refused before
//! anything opens.

#![cfg(feature = "inference")]

mod common;
#[path = "../../../crates/intel/esp/tests/support/decoder.rs"]
mod decoder_fixture;

use std::sync::Arc;
use std::time::Duration;

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, PortableCardV1,
    ProjectionRequest,
};
use common::{
    lane, lending, open_vault, owner, profile, refusal_from, unlock, write_device_settings,
};
use distillery::inference::{
    ASK_INTENT, AskRequestV1, CANCEL_INTENT, CancelRequestV1, INFERENCE_ASK_ROUTE,
    INFERENCE_RUNS_SCHEMA, InferenceProvider, InferenceRunsV1, RunStateV1,
};
use djinn::resident::DjinnResident;
use djinn::settings::{InferenceLaneSettings, InferenceModelSettings, OwnerSettings};
use graphshell::identity::VaultProtectionView;
use graphshell::native::app_admission::{AllowedAppRoutes, AppId, AppRouteGrants, AppRouteId};
use graphshell::native::app_broker::{AppEndpointCatalog, serve_app_broker};
use graphshell::native::app_client::{AppBrokerClient, AppClientError};
use graphshell::native::endpoint_catalog::{ResidentEndpointCatalog, ResidentEndpointRoute};
use personae::{Ed25519Keypair, ProfileId};
use sceno::InstanceId;
use scenotime::{Revision, SceneEpoch};
use uuid::Uuid;

fn stub_lane() -> Option<InferenceLaneSettings> {
    Some(InferenceLaneSettings {
        model: InferenceModelSettings::Stub,
    })
}

fn endpoint_name() -> String {
    #[cfg(windows)]
    return format!(r"\\.\pipe\djinn-inference-route-test-{}", Uuid::new_v4());
    #[cfg(not(windows))]
    return std::env::temp_dir()
        .join(format!(
            "djinn-inference-route-test-{}.sock",
            Uuid::new_v4()
        ))
        .display()
        .to_string();
}

fn keeper_host() -> Arc<PersonaeHost<InMemoryStorage>> {
    let profile = Profile::new(
        ProfileId("default".into()),
        "Default",
        Ed25519Keypair::from_seed([0x5e; 32]),
    );
    Arc::new(PersonaeHost::new(
        IdentityVault::with_profile(InMemoryStorage::new(), profile),
        None,
        VaultProtectionView::Ephemeral,
    ))
}

async fn open(endpoint: &str, app: &str) -> Result<AppBrokerClient, AppClientError> {
    AppBrokerClient::open_route_at(
        endpoint,
        AppId::new(app),
        AppRouteId::new(INFERENCE_ASK_ROUTE).unwrap(),
    )
    .await
}

/// Read the run table the route serves.
async fn read_runs(client: &mut AppBrokerClient, request: &ProjectionRequest) -> InferenceRunsV1 {
    let snapshot = client.snapshot(request.clone()).await.unwrap();
    let offer = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .next()
        .expect("the snapshot has a card")
        .clone();
    let card: PortableCardV1 = serde_json::from_slice(
        &client
            .resource(snapshot.session.clone(), offer.resource)
            .await
            .unwrap(),
    )
    .unwrap();
    serde_json::from_slice(
        &client
            .resource(snapshot.session, card.media[0])
            .await
            .unwrap(),
    )
    .unwrap()
}

async fn intent(
    client: &mut AppBrokerClient,
    request: &ProjectionRequest,
    intent: &str,
    payload: Vec<u8>,
) -> IntentResult {
    let body = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: request.session.clone(),
            target: InstanceId(0),
            observed_epoch: SceneEpoch(1),
            observed_revision: Revision(0),
            intent: intent.into(),
            payload,
        }))
        .await
        .unwrap();
    match body {
        CarrierResponseBody::Intent(result) => result,
        other => panic!("an intent was answered with {other:?}"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_resident_answers_an_ask_on_its_route_and_closes_the_lane() {
    exercise_route(false).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_library_model_answers_through_the_resident_app_door_without_a_trainer() {
    exercise_route(true).await;
}

async fn save_library(data_root: &std::path::Path) -> eidetic::ManifestId {
    use distillery::inference::{InferenceModelRow, PromptTemplate};
    use eidetic::typed::save_typed;
    use eidetic::{ModelLibrary, PrivacyClass, ProvenanceRecord, Timestamp, TrustEnvelope};
    let path = djinn::resident_distillery::model_library_path(&data_root, &profile().0, None);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut store = muniment::RedbBackend::open(&path).unwrap();
    let model = ModelLibrary::save_model_with_components(
        &mut store,
        "test/tiny-decoder",
        "llama",
        "MIT",
        serde_json::from_str(decoder_fixture::CONFIG_JSON).unwrap(),
        &decoder_fixture::weights_bytes(),
        decoder_fixture::tokenizer_json().as_bytes(),
        vec![],
        vec![],
        PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("local"),
        TrustEnvelope::self_asserted(),
        Timestamp::ZERO,
    )
    .await
    .unwrap();
    let mut template = PromptTemplate::from_bytes(include_bytes!(
        "../../../crates/intel/esp/templates/smollm2-instruct-v1.json"
    ))
    .unwrap();
    template.system_prompt.clear();
    template.user.prefix = "t1 ".into();
    template.user.suffix = " t2 ".into();
    template.assistant.prefix = "t3 ".into();
    let template = save_typed(
        &mut store,
        &template,
        vec![],
        PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("local"),
        TrustEnvelope::self_asserted(),
        Timestamp::ZERO,
    )
    .await
    .unwrap();
    let row = save_typed(
        &mut store,
        &InferenceModelRow { model, template },
        vec![],
        PrivacyClass::LocalOnly,
        ProvenanceRecord::self_imported("local"),
        TrustEnvelope::self_asserted(),
        Timestamp::ZERO,
    )
    .await
    .unwrap();
    row
}

async fn exercise_route(library_model: bool) {
    let directory = tempfile::tempdir().expect("temporary resident root");
    let data_root = directory.path().join("data");
    let (vault_dir, identity) = open_vault(directory.path());
    write_device_settings(&data_root, Some(lending()));
    let (inference, question, model_id, expected_text, expected_session) = if library_model {
        use distillery::inference::{InferenceDevice, load_library_session};
        let row = save_library(&data_root).await;
        let path = djinn::resident_distillery::model_library_path(&data_root, &profile().0, None);
        let mut store = muniment::RedbBackend::open(&path).unwrap();
        let provider = load_library_session(
            &mut store,
            row,
            &InferenceDevice::ndarray(),
            "burn-ndarray",
            4,
        )
        .await
        .unwrap();
        let prepared = provider.prepare(&Default::default(), "");
        assert!(prepared.is_err(), "empty questions stay invalid");
        let prepared = provider
            .prepare(
                &serde_json::from_value(serde_json::json!({
                    "prompt": "t4", "max_tokens": 4, "temperature": 0.0, "stop": []
                }))
                .unwrap(),
                "",
            )
            .unwrap();
        let expected = provider.bound().generate_prepared(&prepared).unwrap();
        (
            Some(InferenceLaneSettings {
                model: InferenceModelSettings::Library {
                    row: row.to_string(),
                    device: djinn::settings::InferenceDeviceSettings::Cpu,
                    max_tokens: 4,
                },
            }),
            "t4",
            "test/tiny-decoder",
            expected,
            provider.session_identity(),
        )
    } else {
        (
            stub_lane(),
            "what does the lane say?",
            "stub/echo",
            "echo: what does the lane say?".into(),
            None,
        )
    };
    let settings = OwnerSettings {
        inference,
        ..owner(Some(lane()))
    };
    let resident = DjinnResident::open(
        &identity,
        &data_root,
        &profile(),
        settings,
        &vault_dir,
        unlock(),
    )
    .await
    .expect("open a resident with the inference lane");

    let catalog = AppEndpointCatalog::new(ResidentEndpointCatalog::new());
    let grants = AppRouteGrants::new(AllowedAppRoutes::new([(
        AppId::new("turnstone"),
        ResidentEndpointRoute::new("identity", Duration::from_millis(50)).unwrap(),
    )]));
    let route = catalog
        .update(|catalog| resident.register_inference_route(catalog))
        .await
        .unwrap()
        .expect("the lane registers its route");
    grants.grant(AppId::new("turnstone"), route);

    let endpoint = endpoint_name();
    let server = {
        let (endpoint, grants, catalog) = (endpoint.clone(), grants.clone(), catalog.clone());
        tokio::spawn(async move {
            let _ = serve_app_broker(
                &endpoint,
                Arc::new(djinn::keeper::Keeper::new(keeper_host())),
                grants,
                60_000,
                None,
                catalog,
            )
            .await;
        })
    };
    let mut client = None;
    for _ in 0..100 {
        if let Ok(opened) = open(&endpoint, "turnstone").await {
            client = Some(opened);
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let mut client = client.expect("Turnstone's label is admitted");
    let opened = client.open_session().await.unwrap();
    let request = opened.descriptor.projections[0].request.clone();

    let ask = AskRequestV1 {
        run: "first".into(),
        question: question.into(),
    };
    assert_eq!(
        intent(
            &mut client,
            &request,
            ASK_INTENT,
            serde_json::to_vec(&ask).unwrap()
        )
        .await,
        IntentResult::Accepted
    );
    let mut runs = read_runs(&mut client, &request).await;
    for _ in 0..200 {
        if runs.runs[0].state.is_terminal() {
            break;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
        runs = read_runs(&mut client, &request).await;
    }
    assert_eq!(runs.schema, INFERENCE_RUNS_SCHEMA);
    assert_eq!(runs.model.as_deref(), Some(model_id));
    assert_eq!(runs.session, expected_session);
    assert_eq!(runs.runs[0].session, expected_session);
    assert_eq!(runs.load_error, None);
    if library_model {
        assert!(resident.distillery().unwrap().model_library().is_some());
    }
    assert_eq!(runs.runs[0].run, "first");
    assert_eq!(runs.runs[0].state, RunStateV1::Finished);
    assert_eq!(runs.runs[0].text, expected_text);

    // A cancel of an ended run is accepted and changes nothing; an unknown
    // run and a repeated name are refused.
    let cancel = |run: &str| serde_json::to_vec(&CancelRequestV1 { run: run.into() }).unwrap();
    assert_eq!(
        intent(&mut client, &request, CANCEL_INTENT, cancel("first")).await,
        IntentResult::Accepted
    );
    assert!(matches!(
        intent(&mut client, &request, CANCEL_INTENT, cancel("missing")).await,
        IntentResult::Rejected { .. }
    ));
    assert!(matches!(
        intent(
            &mut client,
            &request,
            ASK_INTENT,
            serde_json::to_vec(&ask).unwrap()
        )
        .await,
        IntentResult::Rejected { .. }
    ));
    if library_model {
        let holder = resident
            .inference_lock_holder()
            .expect("library inference has a lock hook");
        holder.lock();
        assert!(
            client.snapshot(request.clone()).await.is_err(),
            "a previously admitted session cannot snapshot private runs while locked"
        );
        holder.unlock(&identity).unwrap();
        // Some carriers close after a refused request. Reopen through the
        // same app door and confirm the retained answer, not an empty lane.
        let _ = client.close().await;
        client = open(&endpoint, "turnstone").await.unwrap();
        let opened = client.open_session().await.unwrap();
        let unlocked_request = opened.descriptor.projections[0].request.clone();
        let unlocked = read_runs(&mut client, &unlocked_request).await;
        assert_eq!(unlocked.runs[0].text, expected_text);
        assert_eq!(unlocked.runs[0].state, RunStateV1::Finished);
    }
    client.close().await.unwrap();

    // Granted to Turnstone alone (ruling 8).
    for app in ["knot-editor", "djinn", "someone-elses-app"] {
        assert!(
            matches!(open(&endpoint, app).await, Err(AppClientError::Closed)),
            "{app} must be refused {INFERENCE_ASK_ROUTE}"
        );
    }

    server.abort();
    let _ = server.await;
    drop(catalog);
    resident
        .shutdown()
        .await
        .expect("the resident, inference lane first, shuts down cleanly");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn the_lane_without_a_distillery_lane_is_refused_before_anything_opens() {
    let directory = tempfile::tempdir().expect("temporary resident root");
    let data_root = directory.path().join("data");
    let (vault_dir, identity) = open_vault(directory.path());
    let settings = OwnerSettings {
        inference: stub_lane(),
        ..owner(None)
    };
    let refusal = refusal_from(
        DjinnResident::open(
            &identity,
            &data_root,
            &profile(),
            settings,
            &vault_dir,
            unlock(),
        )
        .await,
        "inference without a distillery lane must be refused",
    )
    .await;
    assert!(refusal.contains("needs the distillery lane"), "{refusal}");
}

/// Two production resident compositions, with independent vaults, one graph
/// key agreement and the actual personal-sync endpoints. The tiny decoder is
/// only the model fixture; no identity seed or data key is copied between them.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn keyed_personal_graph_devices_continue_a_seeded_ask() {
    use distillery::inference::{InferenceDevice, RunJournal, load_library_session};
    use graphshell::native::personal_sync_host::{PersonalSyncHost, PersonalSyncHostConfig};
    use graphshell::personal_sync::{SyncRoster, SyncSelection};
    use p2panda_core::{Body, Header, Operation, SigningKey};
    use personae::IdentityProvider;

    let temp = tempfile::tempdir().unwrap();
    let root_a = temp.path().join("a");
    let root_b = temp.path().join("b");
    std::fs::create_dir_all(&root_a).unwrap();
    std::fs::create_dir_all(&root_b).unwrap();
    let (vault_a, identity_a) = open_vault(&root_a);
    let (vault_b, identity_b) = open_vault(&root_b);
    assert_ne!(
        identity_a.master_public_key(),
        identity_b.master_public_key()
    );
    let data_a = root_a.join("data");
    let data_b = root_b.join("data");
    write_device_settings(&data_a, Some(lending()));
    write_device_settings(&data_b, Some(lending()));
    let row = save_library(&data_a).await;
    assert_eq!(save_library(&data_b).await, row);
    let settings = OwnerSettings {
        inference: Some(InferenceLaneSettings {
            model: InferenceModelSettings::Library {
                row: row.to_string(),
                device: djinn::settings::InferenceDeviceSettings::Cpu,
                max_tokens: 8,
            },
        }),
        ..owner(Some(lane()))
    };
    let local = DjinnResident::open(
        &identity_a,
        &data_a,
        &profile(),
        settings.clone(),
        &vault_a,
        unlock(),
    )
    .await
    .unwrap();
    local
        .inference()
        .unwrap()
        .lane()
        .ask(AskRequestV1 {
            run: "retained-local".into(),
            question: "t4".into(),
        })
        .unwrap();
    let local_finished = wait_run(
        local.inference().unwrap().lane(),
        "retained-local",
        |run| run.state.is_terminal(),
        false,
    )
    .await;
    assert_eq!(local_finished.state, RunStateV1::Finished);
    local.shutdown().await.unwrap();
    let mut a = DjinnResident::open_deferred_inference(
        &identity_a,
        &data_a,
        &profile(),
        settings.clone(),
        &vault_a,
        unlock(),
    )
    .await
    .unwrap();
    let mut b = DjinnResident::open_deferred_inference(
        &identity_b,
        &data_b,
        &profile(),
        settings.clone(),
        &vault_b,
        unlock(),
    )
    .await
    .unwrap();
    assert!(
        a.inference().is_none(),
        "no early actor can resume local history during composition"
    );
    let graph = [0x67; 32];
    let roster = SyncRoster::new([
        identity_a.master_public_key().to_bytes(),
        identity_b.master_public_key().to_bytes(),
    ]);
    let config = |root: &std::path::Path, tickets: Vec<String>| PersonalSyncHostConfig {
        graph,
        store_path: root.join("personal.redb"),
        roster: roster.clone(),
        selection: SyncSelection::default(),
        peer_tickets: tickets,
        peer_hints: Vec::new(),
        paired_nodes: Vec::new(),
        relay_urls: Vec::new(),
    };
    let custody_a = a.blobs();
    let custody_b = b.blobs();
    custody_a
        .bind_scope(transport::BlobScope::new(graph))
        .await
        .unwrap();
    custody_b
        .bind_scope(transport::BlobScope::new(graph))
        .await
        .unwrap();
    let host_a = Arc::new(
        PersonalSyncHost::open_with_blob_custody(
            &identity_a,
            config(&root_a, vec![]),
            custody_a.blobs(),
            custody_a.authorizer(),
        )
        .await
        .unwrap(),
    );
    let host_b = Arc::new(
        PersonalSyncHost::open_with_blob_custody(
            &identity_b,
            config(&root_b, vec![host_a.ticket().await.unwrap()]),
            custody_b.blobs(),
            custody_b.authorizer(),
        )
        .await
        .unwrap(),
    );
    drop(custody_a);
    drop(custody_b);
    assert_ne!(host_a.node_id(), host_b.node_id());
    // Register the private topic before pairing A's peer, exercising the live
    // append path as well as B's already-known ticket path.
    a.start_inference(&identity_a, Some(host_a.clone()))
        .await
        .unwrap();
    b.start_inference(&identity_b, Some(host_b.clone()))
        .await
        .unwrap();
    host_a.pair_node(host_b.node_id()).await.unwrap();
    let lane_a = a.inference().unwrap().lane().clone();
    let lane_b = b.inference().unwrap().lane().clone();
    let journal_a = a.inference().unwrap().journal().unwrap().clone();
    let journal_b = b.inference().unwrap().journal().unwrap().clone();
    let ask = |run: &str| AskRequestV1 {
        run: run.into(),
        question: "t4".into(),
    };
    assert!(
        lane_b
            .ask(ask("unkeyed"))
            .unwrap_err()
            .to_string()
            .contains("keyed reader")
    );
    assert!(journal_b.operations().await.unwrap().is_empty());
    host_a.enable_encryption().await.unwrap();
    host_a
        .admit_reader(identity_b.master_public_key().to_bytes(), "device b")
        .await
        .unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        host_a.key_paired_devices().await.unwrap();
        host_b.key_paired_devices().await.unwrap();
        if host_b.seals().can_write() {
            let probe = host_a.seals().seal(b"same graph epoch").unwrap();
            if host_b.seals().open(&probe).is_ok() {
                break;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the existing graph agreement did not key B"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    // Existing local history is re-sealed into the keyed graph without
    // overwriting an existing shared head or handing B the local data key.
    lane_a.refresh_retained_runs().unwrap();
    let imported = wait_run(
        &lane_b,
        "retained-local",
        |run| run.state == RunStateV1::Finished,
        true,
    )
    .await;
    assert_eq!(imported.text, local_finished.text);
    // The resident owns that redb, so use its existing handle for the reference.
    let library = a.distillery().unwrap().model_library().unwrap();
    let reference = load_library_session(
        &mut *library.lock().await,
        row,
        &InferenceDevice::ndarray(),
        "burn-ndarray",
        8,
    )
    .await
    .unwrap();
    let request = serde_json::from_value(serde_json::json!({
        "prompt":"t4", "max_tokens":8, "temperature":0.85,
        "top_p":0.95, "seed":42, "stop":[]
    }))
    .unwrap();
    let expected = reference
        .generate_checkpointed(&request, None, &mut |_| Ok(()), &mut || false)
        .unwrap();
    assert_eq!(expected.token_ids.len(), 8);
    let pause_lane = lane_a.clone();
    let pauser = tokio::task::spawn_blocking(move || {
        let deadline = std::time::Instant::now() + Duration::from_secs(15);
        loop {
            if pause_lane
                .runs()
                .runs
                .iter()
                .any(|run| run.run == "across" && run.tokens >= 3)
            {
                pause_lane.pause("across", "user").unwrap();
                break;
            }
            assert!(std::time::Instant::now() < deadline, "no committed prefix");
            std::thread::yield_now();
        }
    });
    lane_a.ask_with_generation(ask("across"), request).unwrap();
    pauser.await.unwrap();
    let paused = wait_run(
        &lane_a,
        "across",
        |run| run.state == RunStateV1::Paused,
        false,
    )
    .await;
    assert!((3..8).contains(&paused.tokens));
    // A remote materialization is locally paused even while its source still
    // reports Running. Wait for the signed pause record, not that projection,
    // so this is a sequential handoff rather than a concurrent continuation.
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        match journal_b.retained_record("across") {
            Ok(record)
                if record.status == eidetic::models::inference::InferenceRunStatus::Paused =>
            {
                break;
            },
            Ok(_) => {},
            Err(error) => assert!(
                error.contains("missing parent") || error.contains("unknown retained run"),
                "{error}"
            ),
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "the durable pause did not reach B"
        );
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    let remote = wait_run(
        &lane_b,
        "across",
        |run| run.state == RunStateV1::Paused && run.tokens == paused.tokens,
        true,
    )
    .await;
    assert_eq!(remote.text, paused.text);
    lane_b.resume("across", "device-transfer").unwrap();
    let finished = wait_run(&lane_b, "across", |run| run.state.is_terminal(), false).await;
    assert_eq!(finished.state, RunStateV1::Finished, "{:?}", finished.error);
    assert_eq!(finished.text, expected.text);
    assert_eq!(finished.session, paused.session);
    assert_eq!(
        finished.continuations.last().unwrap().device,
        journal_b.device()
    );
    assert_eq!(
        finished.continuations.last().unwrap().at_token,
        paused.tokens
    );
    let evidence = journal_b.retained_record("across").unwrap();
    let actual: serde_json::Value = serde_json::from_slice(
        &journal_b
            .resolve_evidence_blob(evidence.checkpoint_blob.unwrap())
            .unwrap()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(actual["token_ids"], serde_json::json!(expected.token_ids));
    assert_eq!(
        journal_b.record_comparison("across", &expected).unwrap(),
        None
    );

    // The selected boundary is the key group, independently of permission to
    // write the browser graph itself. Removing B from that write roster alone
    // must not revoke its still-current inference seat.
    host_a
        .set_roster(SyncRoster::new([identity_a.master_public_key().to_bytes()]))
        .await;
    let sibling_key = SigningKey::from_bytes(
        &identity_b
            .derive_keypair(&{
                let mut salt = b"mere.djinn/personal-inference/writer/v1/".to_vec();
                salt.extend_from_slice(&graph);
                salt
            })
            .unwrap()
            .to_seed(),
    );
    let sibling_operation = journal_b
        .operations()
        .await
        .unwrap()
        .into_iter()
        .filter(|op| op.header.verifying_key == sibling_key.verifying_key())
        .last()
        .unwrap();
    journal_a.accept(&sibling_operation).await.unwrap();
    host_a.set_roster(roster.clone()).await;

    wait_run(
        &lane_a,
        "across",
        |run| run.state == RunStateV1::Finished,
        true,
    )
    .await;
    let mut salt = b"mere.djinn/personal-inference/writer/v1/".to_vec();
    salt.extend_from_slice(&graph);
    let owner_key = SigningKey::from_bytes(&identity_a.derive_keypair(&salt).unwrap().to_seed());
    let run_hash = *p2panda_core::Hash::digest(b"across").as_bytes();
    let operation = journal_a
        .operations()
        .await
        .unwrap()
        .into_iter()
        .filter(|op| {
            op.header.extensions.run == run_hash
                && op.header.verifying_key == owner_key.verifying_key()
        })
        .last()
        .unwrap();
    let mut ext = operation.header.extensions.clone();
    ext.space = [0x68; 32];
    let body = operation.body.as_ref().unwrap().as_bytes();
    let header = Header::builder().body(body).build(&owner_key, ext);
    let wrong_graph = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(body)),
    };
    assert!(
        journal_b
            .accept(&wrong_graph)
            .await
            .unwrap_err()
            .contains("wrong-space")
    );
    // Possession of the endpoint or even the sealing handle cannot replace
    // this graph's admitted root and domain-bound writer attestation.
    let outsider = personae::InMemoryProvider::from_seed([0x69; 32]);
    let mut ext = operation.header.extensions.clone();
    ext.authority = djinn::resident_inference::personal::writer_authority(
        &outsider.attest_derived_key(&salt).unwrap(),
    )
    .unwrap();
    let key = SigningKey::from_bytes(&outsider.derive_keypair(&salt).unwrap().to_seed());
    let body = operation.body.as_ref().unwrap().as_bytes();
    // Re-seal the matching address, so the membership test cannot accidentally
    // pass merely because an unrelated envelope failed address binding.
    let sealed = serde_json::from_slice(body).unwrap();
    let mut protected: serde_json::Value =
        serde_json::from_slice(&host_a.seals().open(&sealed).unwrap()).unwrap();
    protected["address"] = serde_json::to_value(&ext).unwrap();
    let outside_body = serde_json::to_vec(
        &host_a
            .seals()
            .seal(&serde_json::to_vec(&protected).unwrap())
            .unwrap(),
    )
    .unwrap();
    let header = Header::builder().body(&outside_body).build(&key, ext);
    let forbidden = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(&outside_body)),
    };
    assert!(
        journal_b
            .accept(&forbidden)
            .await
            .unwrap_err()
            .contains("key group")
    );
    // A valid signer also cannot transplant an envelope to another ask.
    let mut ext = operation.header.extensions.clone();
    ext.run = [0x6a; 32];
    let header = Header::builder().body(body).build(&owner_key, ext);
    let transplanted = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(body)),
    };
    assert!(
        journal_b
            .accept(&transplanted)
            .await
            .unwrap_err()
            .contains("another domain or log")
    );

    let plaintext = host_a
        .seals()
        .open(&serde_json::from_slice(body).unwrap())
        .unwrap();
    let header = Header::builder()
        .body(&plaintext)
        .build(&owner_key, operation.header.extensions.clone());
    let unsealed = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(&plaintext)),
    };
    assert!(
        journal_b.accept(&unsealed).await.is_err(),
        "private asks have no plaintext fallback"
    );
    let mut ext = operation.header.extensions.clone();
    ext.authority = djinn::resident_inference::personal::writer_authority(
        &identity_a
            .attest_derived_key(b"wrong inference domain")
            .unwrap(),
    )
    .unwrap();
    let header = Header::builder().body(body).build(&owner_key, ext);
    let wrong_domain = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(body)),
    };
    assert!(journal_b.accept(&wrong_domain).await.is_err());

    let other_parent = journal_b
        .operations()
        .await
        .unwrap()
        .into_iter()
        .find(|op| op.header.extensions.run != run_hash)
        .unwrap();
    let mut protected: serde_json::Value = serde_json::from_slice(&plaintext).unwrap();
    let payload: Vec<u8> = serde_json::from_value(protected["payload"].clone()).unwrap();
    let mut delta: serde_json::Value = serde_json::from_slice(&payload).unwrap();
    delta["parent"] = serde_json::json!(other_parent.hash.to_string());
    protected["payload"] = serde_json::to_value(serde_json::to_vec(&delta).unwrap()).unwrap();
    let cross_body = serde_json::to_vec(
        &host_a
            .seals()
            .seal(&serde_json::to_vec(&protected).unwrap())
            .unwrap(),
    )
    .unwrap();
    let header = Header::builder()
        .body(&cross_body)
        .build(&owner_key, operation.header.extensions.clone());
    let cross_run = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(&cross_body)),
    };
    assert!(
        journal_b
            .accept(&cross_run)
            .await
            .unwrap_err()
            .contains("parent addresses another run")
    );

    let holder = b.inference_lock_holder().unwrap();
    holder.lock();
    assert!(journal_b.load().is_err());
    assert!(
        journal_b
            .accept(&operation)
            .await
            .unwrap_err()
            .contains("locked")
    );
    lane_a.ask(ask("during-lock")).unwrap();
    wait_run(&lane_a, "during-lock", |run| run.state.is_terminal(), false).await;
    for op in journal_a.operations().await.unwrap().iter().filter(|op| {
        op.header.extensions.run == *p2panda_core::Hash::digest(b"during-lock").as_bytes()
    }) {
        assert!(journal_b.accept(op).await.unwrap_err().contains("locked"));
    }
    holder.unlock(&identity_b).unwrap();
    wait_run(
        &lane_b,
        "during-lock",
        |run| run.state == RunStateV1::Finished,
        true,
    )
    .await;
    assert_eq!(
        journal_b.retained_record("across").unwrap().status,
        eidetic::models::inference::InferenceRunStatus::Finished
    );
    // Removal retains old readable history, while the graph's actual epoch
    // rotation excludes this device from the next private ask.
    host_a
        .retire_reader(identity_b.master_public_key().to_bytes())
        .await
        .unwrap();
    host_a.key_paired_devices().await.unwrap();
    lane_a.ask(ask("after-revocation")).unwrap();
    wait_run(
        &lane_a,
        "after-revocation",
        |run| run.state.is_terminal(),
        false,
    )
    .await;
    let new_ops = journal_a.operations().await.unwrap();
    let after = new_ops
        .iter()
        .find(|op| {
            op.header.extensions.run == *p2panda_core::Hash::digest(b"after-revocation").as_bytes()
        })
        .unwrap();
    assert!(
        journal_b.accept(after).await.is_err(),
        "the departed reader must not read the new epoch"
    );
    assert!(journal_b.retained_record("across").is_ok());
    host_a
        .set_roster(SyncRoster::new([identity_a.master_public_key().to_bytes()]))
        .await;
    assert!(
        journal_a.retained_record("across").is_ok(),
        "an admitted historical continuation survives removal of its writer"
    );
    // A fresh journal with the current graph keys can still admit the old
    // reader's historical epoch after that reader has left the current group.
    let late_root = temp.path().join("late-view");
    std::fs::create_dir_all(&late_root).unwrap();
    let local = Arc::new(distillery::inference::durable::PersonaRunJournal::new(
        muniment::RedbBackend::open(late_root.join("local.redb")).unwrap(),
        [0x70; 32],
        &identity_a.derive_keypair(&salt).unwrap(),
        "late-view".into(),
        [0x71; 32],
    ));
    let late = djinn::resident_inference::personal::open(
        &identity_a,
        &late_root,
        &host_a,
        local,
        [0x71; 32],
    )
    .unwrap();
    let history = journal_b.operations().await.unwrap();
    let orphan = history
        .iter()
        .find(|op| op.header.verifying_key != owner_key.verifying_key() && op.header.seq_num == 0)
        .unwrap();
    assert!(
        late.accept(orphan)
            .await
            .unwrap_err()
            .contains("missing checkpoint parent")
    );
    assert!(late.operations().await.unwrap().is_empty());
    assert!(
        late.load().unwrap().is_empty(),
        "an out-of-order child cannot poison restart"
    );
    for _ in 0..=history.len() {
        for op in &history {
            if let Err(error) = late.accept(op).await {
                assert!(error.contains("missing checkpoint parent"), "{error}");
            }
        }
        if late.operations().await.unwrap().len() == history.len() {
            break;
        }
    }
    assert_eq!(late.operations().await.unwrap().len(), history.len());
    assert_eq!(
        late.retained_record("across").unwrap().status,
        eidetic::models::inference::InferenceRunStatus::Finished
    );
    let former = history
        .iter()
        .find(|op| op.header.verifying_key != owner_key.verifying_key())
        .unwrap();
    let frame = serde_json::from_slice(former.body.as_ref().unwrap().as_bytes()).unwrap();
    let clear = host_a.seals().open(&frame).unwrap();
    let current_body = serde_json::to_vec(&host_a.seals().seal(&clear).unwrap()).unwrap();
    let former_key = SigningKey::from_bytes(&identity_b.derive_keypair(&salt).unwrap().to_seed());
    let header = Header::builder()
        .body(&current_body)
        .build(&former_key, former.header.extensions.clone());
    let forbidden_current = Operation {
        hash: header.hash(),
        header,
        body: Some(Body::from_bytes(&current_body)),
    };
    assert!(
        late.accept(&forbidden_current)
            .await
            .unwrap_err()
            .contains("key group")
    );
    late.shutdown().await.unwrap();
    drop(late);
    // An independently named local ask must not disappear behind an existing
    // shared run with the same name. Keep its source and report the collision.
    let collision_source = Arc::new(distillery::inference::durable::PersonaRunJournal::new(
        muniment::RedbBackend::open(late_root.join("collision-local.redb")).unwrap(),
        [0x70; 32],
        &identity_a.derive_keypair(&salt).unwrap(),
        "another-device".into(),
        [0x71; 32],
    ));
    let mut conflicting = serde_json::to_value(
        journal_b
            .load()
            .unwrap()
            .into_iter()
            .find(|record| serde_json::to_value(record).unwrap()["view"]["run"] == "across")
            .unwrap(),
    )
    .unwrap();
    conflicting["view"]["question"] = serde_json::json!("a different retained question");
    conflicting["request"]["prompt"] = serde_json::json!("a different retained question");
    conflicting["view"]["text"] = serde_json::json!("");
    conflicting["view"]["state"] = serde_json::json!("queued");
    conflicting["view"]["session"] = serde_json::Value::Null;
    conflicting["checkpoint"] = serde_json::Value::Null;
    conflicting["continuations"] = serde_json::json!([{
        "device":"another-device", "at_token":0, "reason":"ask",
        "first_differing_token":null, "compared_token_ids_hash":null
    }]);
    let conflicting = serde_json::from_value(conflicting).unwrap();
    collision_source.append(&conflicting).unwrap();
    let collision = djinn::resident_inference::personal::open(
        &identity_a,
        &late_root,
        &host_a,
        collision_source.clone(),
        [0x71; 32],
    )
    .unwrap();
    let collision_error = collision
        .load()
        .err()
        .expect("a conflicting local ask must remain visible");
    assert!(collision_error.contains("collides with personal graph history"));
    assert_eq!(
        serde_json::to_value(&collision_source.load().unwrap()[0]).unwrap()["view"]["question"],
        "a different retained question"
    );
    drop(collision);
    drop(collision_source);
    let receipt = serde_json::json!({
        "separate_identity_roots":true, "separate_transport_nodes":true,
        "keyed_reader_need_not_write_graph":true,
        "model_row":row.to_string(), "session":finished.session,
        "paused_tokens":paused.tokens, "completed_tokens":finished.tokens,
        "seed":42, "resumed_token_ids":actual["token_ids"],
        "reference_token_ids":expected.token_ids, "first_differing_token":null,
        "continuations":finished.continuations,
        "retained_evidence":journal_b.retained_record("across").unwrap(),
        "unkeyed_refused":true, "wrong_graph_refused":true,
        "outside_key_group_refused":true, "transplanted_envelope_refused":true,
        "plaintext_fallback_refused":true, "wrong_writer_domain_refused":true,
        "locked_refused":true, "sync_catches_up_after_unlock":true,
        "new_epoch_excludes_retired_reader":true,
        "retired_reader_retains_history":true,
        "local_history_imported":true, "conflicting_local_history_preserved":true,
        "retained_history_survives_roster_removal":true,
        "late_history_after_revocation":true, "former_reader_current_epoch_write_refused":true,
        "out_of_order_child_refused_before_persistence":true, "cross_run_parent_refused":true,
        "a_sync_ops_received":journal_a.sync_status().unwrap().ops_received,
        "b_sync_ops_received":journal_b.sync_status().unwrap().ops_received,
        "boundary":"two independently rooted resident compositions on one computer; actual personal graph key agreement and endpoints; tiny CPU model; no installed binary or physical second computer"
    });
    drop(reference);
    drop(library);
    drop(holder);
    drop(lane_a);
    drop(lane_b);
    drop(host_a);
    drop(host_b);
    a.shutdown().await.unwrap();
    b.shutdown().await.unwrap();
    journal_a.shutdown().await.unwrap();
    journal_b.shutdown().await.unwrap();
    drop(journal_a);
    drop(journal_b);
    drop(muniment::RedbBackend::open(root_a.join("personal.redb")).unwrap());
    drop(muniment::RedbBackend::open(root_b.join("personal.redb")).unwrap());
    let mut receipt = receipt;
    receipt["shared_custody_shutdown_drained"] = serde_json::json!(true);
    if let Some(path) = std::env::var_os("DJINN_PERSONAL_INFERENCE_RECEIPT") {
        std::fs::write(path, serde_json::to_vec_pretty(&receipt).unwrap()).unwrap();
    }
}

async fn wait_run(
    lane: &distillery::inference::InferenceLane,
    id: &str,
    test: impl Fn(&distillery::inference::RunV1) -> bool,
    refresh: bool,
) -> distillery::inference::RunV1 {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        if refresh {
            if let Err(error) = lane.refresh_retained_runs() {
                assert!(error.contains("missing parent"), "{error}");
            }
        }
        if let Some(run) = lane.runs().runs.into_iter().find(|run| run.run == id) {
            if test(&run) {
                return run;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out: {:?}",
            lane.runs()
        );
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}
