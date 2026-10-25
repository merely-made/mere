// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `graphshell-mcp` over a live mere route: the real reservoir, the real route
//! grants and the real session item, reached through the same endpoint catalog
//! the application door serves. The adapter holds no key; what it may do is
//! what the door admitted its app id for.

use super::*;
use chirograph::{
    Carrier, CarrierError, CarrierNotice, CarrierRequest, CarrierRequestBody, CarrierResponseBody,
    SessionOpened, SessionStatus,
};
use graphshell_endpoint::dispatch_common;
use graphshell_mcp::{Config, Server};
use mere_overmap::wire::{APPLY_EDITS_INTENT, ApplyEditsV1, StepV1, UNDO_INTENT};
use serde_json::{Value, json};

const AGENT: &str = "mcp.claude-code";

/// An admitted endpoint session as a `Carrier`: what `AppRouteCarrier` is over
/// the socket, without the socket.
struct Admitted {
    session: ResidentEndpointSession,
    next: u64,
}

impl Carrier for Admitted {
    fn request(&mut self, body: CarrierRequestBody) -> Result<CarrierResponseBody, CarrierError> {
        if let CarrierRequestBody::Open(_) = body {
            return Ok(CarrierResponseBody::Opened(Box::new(SessionOpened {
                version: chirograph::ProtocolVersion::V1,
                descriptor: self.session.describe(),
                status: SessionStatus::Live,
                expires_at_ms: None,
            })));
        }
        self.next += 1;
        let request = CarrierRequest {
            id: self.next,
            body,
        };
        let mut resume = |endpoint: &mut ResidentEndpointSession, request| endpoint.resume(request);
        match dispatch_common(&mut self.session, request, &mut resume) {
            Ok(response) => response
                .body
                .map_err(|failure| CarrierError::Refused(failure.message)),
            Err(plane) => Err(CarrierError::Refused(format!("session plane: {plane:?}"))),
        }
    }

    fn take_notice(&mut self) -> Option<CarrierNotice> {
        None
    }

    fn wait_for_notice(&mut self) -> Result<CarrierNotice, CarrierError> {
        Err(CarrierError::Disconnected("no notices".into()))
    }

    fn shutdown(&mut self) -> Result<(), CarrierError> {
        Ok(())
    }
}

fn modern(id: u64, method: &str, params: Value) -> String {
    let mut params = params;
    params["_meta"] = json!({"io.modelcontextprotocol/protocolVersion": "2026-07-28"});
    json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}).to_string()
}

fn ask(server: &mut Server<Admitted>, method: &str, params: Value) -> Value {
    let reply = server.handle(&modern(1, method, params)).expect("a reply");
    serde_json::from_str(&reply).unwrap()
}

fn tool(server: &mut Server<Admitted>, name: &str, arguments: Value) -> Value {
    ask(
        server,
        "tools/call",
        json!({"name": name, "arguments": arguments}),
    )
}

fn read(server: &mut Server<Admitted>, uri: &str) -> Value {
    let reply = ask(server, "resources/read", json!({ "uri": uri }));
    serde_json::from_str(reply["result"]["contents"][0]["text"].as_str().unwrap()).unwrap()
}

async fn server_for(fixture: &Fixture, route: &str, app: &str, writes: bool) -> Server<Admitted> {
    let session = fixture
        .catalog
        .update(|catalog| catalog.open(route, &context(app)))
        .await
        .unwrap();
    let config = Config {
        scope: route.into(),
        writes,
        ..Config::default()
    };
    Server::new(Admitted { session, next: 0 }, config)
}

/// The resource URI of the attached session's graph projection.
fn graph_resource(server: &mut Server<Admitted>) -> String {
    let listed = ask(server, "resources/list", json!({}));
    listed["result"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .find(|resource| resource["name"] == "Attached session")
        .expect("the route offers the mere's graph")["uri"]
        .as_str()
        .unwrap()
        .to_string()
}

/// The graph projection's session-item action named `intent`, with the
/// session, epoch and revision it was read at.
fn advertised(server: &mut Server<Admitted>, intent: &str) -> (Value, Value, Value) {
    let listed = tool(server, "mere_actions", json!({}));
    for projection in listed["result"]["structuredContent"]["projections"]
        .as_array()
        .unwrap()
    {
        for action in projection["actions"].as_array().unwrap() {
            if action["intent"] == intent {
                return (
                    action["instance"].clone(),
                    projection["epoch"].clone(),
                    projection["revision"].clone(),
                );
            }
        }
    }
    panic!("the route does not advertise {intent}: {listed}");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_agent_writes_a_mere_as_its_granted_app_and_the_door_names_it() {
    let fixture = fixture(41, "memory").await;
    let route_request = AppRequest {
        app: AppId::new(AGENT),
        route: AppRouteId::new(&fixture.route).unwrap(),
    };
    assert!(
        fixture.grants.current().admit(&route_request).is_err(),
        "an app the owner has not admitted reaches no mere"
    );
    fixture.grants.grant(
        AppId::new(AGENT),
        ResidentEndpointRoute::new("identity", MERE_NOTICE_POLL).unwrap(),
    );
    assert!(fixture.grants.current().admit(&route_request).is_ok());

    let mut mcp = server_for(&fixture, &fixture.route, AGENT, true).await;
    let listed = ask(&mut mcp, "resources/list", json!({}));
    let names: Vec<&str> = listed["result"]["resources"]
        .as_array()
        .unwrap()
        .iter()
        .map(|resource| resource["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["Sessions", "Attached session", "Archive"]);
    let graph_uri = graph_resource(&mut mcp);

    let before = read(&mut mcp, &graph_uri);
    let (session_item, epoch, revision) = advertised(&mut mcp, APPLY_EDITS_INTENT);
    assert_eq!(
        before["items"].as_array().unwrap().len(),
        1,
        "only the session item"
    );

    let note = ApplyEditsV1::new(vec![node(7), retitle(7, "decided: servers hold no keys")]);
    let done = tool(
        &mut mcp,
        "mere_invoke",
        json!({
            "projection": graph_uri,
            "instance": session_item,
            "intent": APPLY_EDITS_INTENT,
            "payload": serde_json::to_value(&note).unwrap(),
            "observed_epoch": epoch,
            "observed_revision": revision,
        }),
    );
    assert_eq!(done["result"]["isError"], false, "{done}");
    assert_eq!(done["result"]["structuredContent"]["result"], "accepted");

    let after = read(&mut mcp, &graph_uri);
    assert_eq!(
        after["items"].as_array().unwrap().len(),
        2,
        "the note is a node now"
    );
    assert!(after["revision"].as_u64() > before["revision"].as_u64());
    assert!(
        after.to_string().contains("decided: servers hold no keys"),
        "the retitle is readable back through the resource: {after}"
    );

    // The agent's own undo takes its edit back, and only its own.
    let (item, epoch, revision) = advertised(&mut mcp, UNDO_INTENT);
    let undone = tool(
        &mut mcp,
        "mere_invoke",
        json!({
            "projection": graph_uri,
            "instance": item,
            "intent": UNDO_INTENT,
            "payload": serde_json::to_value(StepV1::default()).unwrap(),
            "observed_epoch": epoch,
            "observed_revision": revision,
        }),
    );
    assert_eq!(undone["result"]["isError"], false, "{undone}");
    assert_eq!(
        read(&mut mcp, &graph_uri)["items"]
            .as_array()
            .unwrap()
            .len(),
        1
    );

    // The stored changes name the app the door admitted, never a client claim.
    let mere = fixture.reservoir.meres().await.remove(0);
    let sessions = mere_dir(fixture._root.path(), persona(41), mere.id).join("sessions");
    let mut changes = String::new();
    for entry in std::fs::read_dir(sessions).unwrap() {
        changes += &std::fs::read_to_string(entry.unwrap().path().join("changes.jsonl")).unwrap();
    }
    assert!(
        changes.contains(&format!("\"via\":\"{AGENT}\"")),
        "{changes}"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_malformed_edit_is_the_routes_refusal_and_leaves_the_mere_alone() {
    let fixture = fixture(42, "memory").await;
    fixture.grants.grant(
        AppId::new(AGENT),
        ResidentEndpointRoute::new("identity", MERE_NOTICE_POLL).unwrap(),
    );
    let mut mcp = server_for(&fixture, &fixture.route, AGENT, true).await;
    let uri = graph_resource(&mut mcp);
    let (item, epoch, revision) = advertised(&mut mcp, APPLY_EDITS_INTENT);
    let refused = tool(
        &mut mcp,
        "mere_invoke",
        json!({
            "projection": uri, "instance": item, "intent": APPLY_EDITS_INTENT,
            "payload": {"schema": "mere.session.apply/v1", "edits": "not a list"},
            "observed_epoch": epoch, "observed_revision": revision,
        }),
    );
    assert_eq!(refused["result"]["isError"], true, "{refused}");
    assert_eq!(read(&mut mcp, &uri)["items"].as_array().unwrap().len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn the_read_only_ambient_route_offers_an_agent_no_way_to_write() {
    let fixture = fixture(43, "memory").await;
    fixture.grants.grant(
        AppId::new(AGENT),
        ResidentEndpointRoute::new("identity", MERE_NOTICE_POLL).unwrap(),
    );
    let mere = fixture.reservoir.meres().await.remove(0);
    fixture
        .routes
        .set_access(
            &fixture.reservoir,
            mere.id,
            AppId::new(AGENT),
            false,
            true,
            AppId::new("turnstone"),
        )
        .await
        .unwrap();
    // Ambient reading needs live material, which an explicit attach creates.
    let mut explicit = server_for(&fixture, &fixture.route, AGENT, true).await;
    let uri = graph_resource(&mut explicit);
    read(&mut explicit, &uri);
    let ambient = ambient_mere_route_id("memory");
    let mut mcp = server_for(&fixture, &ambient, AGENT, true).await;
    let listed = ask(&mut mcp, "resources/list", json!({}));
    assert_eq!(listed["result"]["resources"].as_array().unwrap().len(), 1);
    let actions = tool(&mut mcp, "mere_actions", json!({}));
    let rows = actions["result"]["structuredContent"]["projections"][0]["actions"]
        .as_array()
        .unwrap();
    assert!(
        rows.iter().all(|row| row["intent"] != APPLY_EDITS_INTENT),
        "the ambient route advertises no edit: {rows:?}"
    );
    let refused = tool(
        &mut mcp,
        "mere_invoke",
        json!({
            "projection": listed["result"]["resources"][0]["uri"], "instance": 0,
            "intent": APPLY_EDITS_INTENT,
            "payload": serde_json::to_value(ApplyEditsV1::new(vec![node(9)])).unwrap(),
            "observed_epoch": 0, "observed_revision": 0,
        }),
    );
    assert_eq!(refused["result"]["isError"], true, "{refused}");
}
