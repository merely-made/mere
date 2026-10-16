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

async fn exercise_route(library_model: bool) {
    let directory = tempfile::tempdir().expect("temporary resident root");
    let data_root = directory.path().join("data");
    let (vault_dir, identity) = open_vault(directory.path());
    write_device_settings(&data_root, Some(lending()));
    let (inference, question, model_id, expected_text, expected_session) = if library_model {
        use distillery::inference::{
            InferenceDevice, InferenceModelRow, PromptTemplate, load_library_session,
        };
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
