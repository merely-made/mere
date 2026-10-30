// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use castellan::authority::PersonaeHost;
use castellan::custody::{IdentityVault, InMemoryStorage, Profile};
use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, PortableCardV1,
    ProjectionRequest,
};
use distillery::inference::{INFERENCE_ASK_ROUTE, InferenceRunsV1};
use graphshell_endpoint::door::{
    app_admission::{AppId, AppRouteId},
    app_client::AppClientError, projection::AppBrokerClient,
    identity::VaultProtectionView,
};
use personae::{Ed25519Keypair, ProfileId};
use sceno::InstanceId;
use scenotime::{Revision, SceneEpoch};
use std::sync::Arc;
use uuid::Uuid;

pub fn endpoint_name() -> String {
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

pub fn keeper_host() -> Arc<PersonaeHost<InMemoryStorage>> {
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

pub async fn open(endpoint: &str, app: &str) -> Result<AppBrokerClient, AppClientError> {
    AppBrokerClient::open_route_at(
        endpoint,
        AppId::new(app),
        AppRouteId::new(INFERENCE_ASK_ROUTE).unwrap(),
    )
    .await
}

/// Read the run table the route serves.
pub async fn read_runs(
    client: &mut AppBrokerClient,
    request: &ProjectionRequest,
) -> InferenceRunsV1 {
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

pub async fn intent(
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
