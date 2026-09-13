// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! First-party caller for Djinn's persistent saved-site service.
//!
//! It reads a completed Knot site once, turns it into a process-neutral
//! snapshot, and uploads bounded frames through the owner-only app broker.

use std::path::Path;

use chirograph::{
    CarrierRequestBody, CarrierResponseBody, IntentInvocation, IntentResult, ProjectionSession,
};
use djinn::resident_site::{
    GeminiListenerPolicyV1, MAX_PUBLISHED_SITE_UPLOAD_CHUNK_BYTES, PUBLISHED_SITE_CHUNK_INTENT,
    PUBLISHED_SITE_COMMIT_INTENT, PUBLISHED_SITE_PREPARE_INTENT, PUBLISHED_SITE_REMOVE_INTENT,
    PUBLISHED_SITE_ROUTE, PUBLISHED_SITE_STOP_INTENT, PublishedSiteChunkV1, PublishedSitePrepareV1,
};
use graphshell::native::app_admission::{AppId, AppRouteId};
use graphshell::native::app_client::AppBrokerClient;
use knot_site::Site;
use sceno::InstanceId;
use scenotime::{Revision, SceneEpoch};

const SESSION: &str = "djinn.published-site/v1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let command = args
        .next()
        .ok_or("usage: djinn-site publish SITE_DIR [PORT] [--resume] | status | stop | remove")?;
    let mut client = AppBrokerClient::open_route(
        AppId::new("knot-editor"),
        AppRouteId::new(PUBLISHED_SITE_ROUTE)?,
    )
    .await?;
    client.open_session().await?;
    match command.as_str() {
        "publish" => {
            let site = args.next().ok_or("djinn-site publish needs SITE_DIR")?;
            let mut port = 0;
            let mut resume_on_restart = false;
            for argument in args {
                if argument == "--resume" {
                    resume_on_restart = true;
                } else if port == 0 {
                    port = argument.parse::<u16>()?;
                } else {
                    return Err("usage: djinn-site publish SITE_DIR [PORT] [--resume]".into());
                }
            }
            publish(&mut client, Path::new(&site), port, resume_on_restart).await?;
            print_status(&mut client).await?;
        },
        "status" => print_status(&mut client).await?,
        "stop" => {
            invoke(&mut client, PUBLISHED_SITE_STOP_INTENT, Vec::new()).await?;
            print_status(&mut client).await?;
        },
        "remove" => {
            invoke(&mut client, PUBLISHED_SITE_REMOVE_INTENT, Vec::new()).await?;
            print_status(&mut client).await?;
        },
        _ => {
            return Err(
                "usage: djinn-site publish SITE_DIR [PORT] [--resume] | status | stop | remove"
                    .into(),
            );
        },
    }
    client.close().await?;
    Ok(())
}

async fn publish(
    client: &mut AppBrokerClient,
    site_dir: &Path,
    port: u16,
    resume_on_restart: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = Site::open(site_dir)?.publication()?.to_snapshot_v1()?;
    let bytes = snapshot.encode()?;
    let chunks = bytes.len().div_ceil(MAX_PUBLISHED_SITE_UPLOAD_CHUNK_BYTES) as u32;
    let prepare = PublishedSitePrepareV1 {
        snapshot_digest: *blake3::hash(&bytes).as_bytes(),
        total_bytes: bytes.len() as u64,
        chunks,
        policy: GeminiListenerPolicyV1 {
            port,
            resume_on_restart,
            ..Default::default()
        },
    };
    invoke(
        client,
        PUBLISHED_SITE_PREPARE_INTENT,
        serde_json::to_vec(&prepare)?,
    )
    .await?;
    for (index, bytes) in bytes
        .chunks(MAX_PUBLISHED_SITE_UPLOAD_CHUNK_BYTES)
        .enumerate()
    {
        invoke(
            client,
            PUBLISHED_SITE_CHUNK_INTENT,
            serde_json::to_vec(&PublishedSiteChunkV1 {
                index: index as u32,
                bytes: bytes.to_vec(),
            })?,
        )
        .await?;
    }
    invoke(client, PUBLISHED_SITE_COMMIT_INTENT, Vec::new()).await
}

async fn print_status(client: &mut AppBrokerClient) -> Result<(), Box<dyn std::error::Error>> {
    let opened = client.open_session().await?;
    let request = opened
        .descriptor
        .projections
        .first()
        .ok_or("published-site route did not offer a status projection")?
        .request
        .clone();
    let snapshot = client.snapshot(request).await?;
    let offer = snapshot
        .presentation
        .offers
        .values()
        .flatten()
        .next()
        .ok_or("published-site status snapshot has no resource")?;
    let card_bytes = client
        .resource(snapshot.session.clone(), offer.resource)
        .await?;
    let card: chirograph::PortableCardV1 = serde_json::from_slice(&card_bytes)?;
    let status_resource = card
        .media
        .first()
        .ok_or("published-site status card has no typed status resource")?;
    let bytes = client.resource(snapshot.session, *status_resource).await?;
    let status: djinn::resident_site::PublishedSiteStatusV1 = serde_json::from_slice(&bytes)?;
    println!("{}", serde_json::to_string_pretty(&status)?);
    Ok(())
}

async fn invoke(
    client: &mut AppBrokerClient,
    action: &str,
    payload: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = client
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: ProjectionSession(SESSION.into()),
            target: InstanceId(0),
            observed_epoch: SceneEpoch(1),
            observed_revision: Revision(1),
            intent: action.into(),
            payload,
        }))
        .await?;
    match response {
        CarrierResponseBody::Intent(IntentResult::Accepted) => Ok(()),
        CarrierResponseBody::Intent(IntentResult::Rejected { reason }) => Err(reason.into()),
        CarrierResponseBody::Intent(IntentResult::Stale { .. }) => {
            Err("published-site route is stale".into())
        },
        other => {
            Err(format!("published-site route returned unexpected response: {other:?}").into())
        },
    }
}
