// SPDX-License-Identifier: MPL-2.0
//! Acceptance preparation through ordinary admitted application routes.
//! No direct store writes and no co-op invitation or edit API is used here.
use chirograph::*;
use graphshell_endpoint::door::{
    app_admission::{AppId, AppRouteId},
    app_client::AppBrokerClient,
};
use mere_overmap::wire::*;
use pandect::CapturedDelta;
use sceno::InstanceId;
use serde_json::json;

async fn invoke(
    c: &mut AppBrokerClient,
    s: &ProjectionSnapshot,
    target: InstanceId,
    intent: &str,
    payload: Vec<u8>,
) {
    let reply = c
        .request_body(CarrierRequestBody::Intent(IntentInvocation {
            session: s.session.clone(),
            target,
            observed_epoch: s.scene.epoch,
            observed_revision: s.scene.revision,
            intent: intent.into(),
            payload,
        }))
        .await
        .unwrap();
    assert!(
        matches!(reply, CarrierResponseBody::Intent(IntentResult::Accepted)),
        "{reply:?}"
    );
}
async fn open(route: &str) -> (AppBrokerClient, EndpointDescriptor) {
    let mut c =
        AppBrokerClient::open_route(AppId::new("graphshell"), AppRouteId::new(route).unwrap())
            .await
            .unwrap();
    let d = c.open_session().await.unwrap().descriptor;
    (c, d)
}
async fn attach(c: &mut AppBrokerClient, d: &EndpointDescriptor, id: pandect::SessionId) {
    let s = c.snapshot(d.projections[0].request.clone()).await.unwrap();
    let intent = mere_overmap::route::intent(
        &mere_view::MereViewRequest::Session(id, mere_view::SessionStep::Switch),
        &s,
    )
    .unwrap();
    let reply = c
        .request_body(CarrierRequestBody::Intent(intent))
        .await
        .unwrap();
    assert!(
        matches!(reply, CarrierResponseBody::Intent(IntentResult::Accepted)),
        "{reply:?}"
    );
}
#[tokio::main]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let domain = args.get(1).map(String::as_str).unwrap_or("coop-acceptance");
    if args[0] == "seed" {
        let (mut reservoir, descriptor) =
            open(djinn::resident_reservoir::RESIDENT_RESERVOIR_ROUTE).await;
        let s = reservoir
            .snapshot(descriptor.projections[0].request.clone())
            .await
            .unwrap();
        invoke(
            &mut reservoir,
            &s,
            InstanceId(0),
            djinn::resident_reservoir::RESERVOIR_ENSURE_MERE_INTENT,
            serde_json::to_vec(&djinn::resident_reservoir::EnsureMereV1::new(domain)).unwrap(),
        )
        .await;
        let (mut c, d) = open(&format!("mere/{domain}")).await;
        let mut ids = vec![];
        for (i, url) in [
            "https://private.invalid/owner-only",
            "https://shared.invalid/existing",
        ]
        .iter()
        .enumerate()
        {
            if i > 0 {
                let s = c.snapshot(d.projections[0].request.clone()).await.unwrap();
                let intent =
                    mere_overmap::route::intent(&mere_view::MereViewRequest::Mint, &s).unwrap();
                let r = c
                    .request_body(CarrierRequestBody::Intent(intent))
                    .await
                    .unwrap();
                assert!(matches!(
                    r,
                    CarrierResponseBody::Intent(IntentResult::Accepted)
                ));
            }
            let s = c.snapshot(d.projections[1].request.clone()).await.unwrap();
            let session_item = s
                .scene
                .active_items_in_order()
                .into_iter()
                .find_map(|(instance, item)| {
                    s.scene.tables.sources[item.source.0 as usize]
                        .as_ref()
                        .filter(|source| source.adapter == SESSION_ITEM_SOURCE)
                        .map(|source| (instance, source.id.clone()))
                })
                .unwrap();
            let id = uuid::Uuid::parse_str(&session_item.1).unwrap();
            ids.push(id);
            let edits = vec![CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id: uuid::Uuid::new_v4().to_string(),
                url: url.to_string(),
                position: [i as f32 * 150., 0.],
            }];
            invoke(
                &mut c,
                &s,
                session_item.0,
                APPLY_EDITS_INTENT,
                serde_json::to_vec(&ApplyEditsV1::new(edits)).unwrap(),
            )
            .await;
        }
        println!("{}", json!({"private":ids[0],"chosen":ids[1]}));
    } else if args[0] == "read" {
        let id = pandect::SessionId(uuid::Uuid::parse_str(&args[2]).unwrap());
        let (mut c, d) = open(&format!("mere/{domain}")).await;
        attach(&mut c, &d, id).await;
        let s = c.snapshot(d.projections[1].request.clone()).await.unwrap();
        let mut cards = vec![];
        let mut graphs = vec![];
        for offer in s.presentation.offers.values().flatten() {
            let bytes = c.resource(s.session.clone(), offer.resource).await.unwrap();
            let card = serde_json::from_slice::<PortableCardV1>(&bytes).unwrap();
            for media in &card.media {
                let bytes = c.resource(s.session.clone(), *media).await.unwrap();
                if let Ok(graph) = serde_json::from_slice::<SessionGraphV1>(&bytes) {
                    graphs.push(graph);
                }
            }
            cards.push(card);
        }
        println!("{}", json!({"snapshot":s,"cards":cards,"graphs":graphs}));
    } else if args[0] == "ordinary-add" {
        let id = pandect::SessionId(uuid::Uuid::parse_str(&args[2]).unwrap());
        let (mut c, d) = open(&format!("mere/{domain}")).await;
        attach(&mut c, &d, id).await;
        let s = c.snapshot(d.projections[1].request.clone()).await.unwrap();
        let target = s
            .scene
            .active_items_in_order()
            .into_iter()
            .find_map(|(instance, item)| {
                s.scene.tables.sources[item.source.0 as usize]
                    .as_ref()
                    .filter(|source| source.adapter == SESSION_ITEM_SOURCE)
                    .map(|_| instance)
            })
            .unwrap();
        let from = s
            .scene
            .tables
            .sources
            .iter()
            .flatten()
            .find(|source| source.adapter == "mere.graph")
            .unwrap()
            .id
            .clone();
        let added = uuid::Uuid::new_v4();
        invoke(
            &mut c,
            &s,
            target,
            APPLY_EDITS_INTENT,
            serde_json::to_vec(&ApplyEditsV1::new(vec![
                CapturedDelta::ReplayAddNodeWithIdIfMissing {
                    id: uuid::Uuid::new_v4().to_string(),
                    url: args[3].clone(),
                    position: [150., 150.],
                },
            ]))
            .unwrap(),
        )
        .await;
        println!("saved");
    } else {
        panic!("seed DOMAIN | read DOMAIN UUID | ordinary-add DOMAIN UUID URL")
    }
}
