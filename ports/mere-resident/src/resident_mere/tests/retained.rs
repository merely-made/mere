// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Native retained page text over a live mere route (Eidetic S6): the
//! reservoir-owned store behind the door's bounded read and the two
//! owner-checked keep intents.

use super::*;
use chirograph::{
    CarrierRequest, CarrierRequestBody, CarrierResponseBody, RetainedKeeping, RetainedPagesLimits,
    RetainedPagesRequest, RetainedPagesResponse,
};
use graphshell::kept_text::{
    KEEP_PAGES_INTENT, KEEP_SETTING_INTENT, KeepPagesV1, KeepSettingV1, KeptPage,
};
use std::sync::atomic::AtomicBool;

fn graph_intent(session: &ProjectionSnapshot, intent: &str, payload: Vec<u8>) -> IntentInvocation {
    IntentInvocation {
        session: session.session.clone(),
        target: InstanceId(0),
        observed_epoch: session.scene.epoch,
        observed_revision: session.scene.revision,
        intent: intent.into(),
        payload,
    }
}

fn setting(enabled: bool) -> Vec<u8> {
    serde_json::to_vec(&KeepSettingV1 {
        enabled,
        ..KeepSettingV1::default()
    })
    .unwrap()
}

fn keep(pages: &[(&str, &str)]) -> Vec<u8> {
    serde_json::to_vec(&KeepPagesV1 {
        pages: pages
            .iter()
            .map(|(url, text)| KeptPage {
                url: (*url).into(),
                text: (*text).into(),
                at_ms: 0,
                private: false,
            })
            .collect(),
    })
    .unwrap()
}

/// The reply through the same dispatch the door runs, so the verb itself is
/// exercised and not only the endpoint method.
fn read(
    session: &mut ResidentEndpointSession,
    graph: &ProjectionSnapshot,
    limits: RetainedPagesLimits,
) -> Result<RetainedPagesResponse, String> {
    let response = graphshell_endpoint::dispatch_common(
        session,
        CarrierRequest {
            id: 9,
            body: CarrierRequestBody::RetainedPages(RetainedPagesRequest {
                session: graph.session.clone(),
                limits,
            }),
        },
        &mut |endpoint, request| endpoint.resume(request),
    )
    .map_err(|_| "session plane".to_string())?;
    assert_eq!(response.id, 9);
    match response.body {
        Ok(CarrierResponseBody::RetainedPages(pages)) => Ok(*pages),
        Ok(other) => panic!("unexpected reply {other:?}"),
        Err(failure) => Err(failure.message),
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn keeping_is_off_by_default_and_a_refused_keep_writes_nothing() {
    let fixture = fixture(0x5336_01, "browsing").await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    let none = read(&mut app, &graph, RetainedPagesLimits::default()).unwrap();
    assert_eq!(none.keeping, RetainedKeeping::Off);
    assert!(none.pages.is_empty() && none.truncated.is_complete());
    let refused = app
        .invoke(graph_intent(
            &graph,
            KEEP_PAGES_INTENT,
            keep(&[("https://a.test/", "should never be written")]),
        ))
        .unwrap();
    assert!(
        matches!(&refused, IntentResult::Rejected { reason } if reason.contains("off")),
        "{refused:?}"
    );
    assert!(read(&mut app, &graph, Default::default()).unwrap().pages.is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn kept_pages_come_back_bounded_with_versions_and_omissions_counted() {
    let fixture = fixture(0x5336_02, "browsing").await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true)))
            .unwrap(),
        IntentResult::Accepted
    );
    for (url, text) in [
        ("https://a.test/", "alpha body about lighthouses"),
        ("https://b.test/", "bravo body about harbours"),
        ("https://c.test/", "charlie body about tides"),
    ] {
        assert_eq!(
            app.invoke(graph_intent(&graph, KEEP_PAGES_INTENT, keep(&[(url, text)])))
                .unwrap(),
            IntentResult::Accepted
        );
        // Distinct stored_at so "most recent first" is deterministic.
        tokio::time::sleep(Duration::from_millis(3)).await;
    }
    // A changed page adds a version; the current one is the new text.
    assert_eq!(
        app.invoke(graph_intent(
            &graph,
            KEEP_PAGES_INTENT,
            keep(&[("https://a.test/", "alpha body, revised: about lamps")]),
        ))
        .unwrap(),
        IntentResult::Accepted
    );
    let all = read(&mut app, &graph, Default::default()).unwrap();
    assert_eq!(all.keeping, RetainedKeeping::On);
    assert_eq!(all.pages.len(), 3);
    assert!(all.truncated.is_complete());
    let a = all.pages.iter().find(|p| p.address.contains("a.test")).unwrap();
    assert_eq!(a.versions.len(), 2);
    assert_eq!(a.versions[0].version, a.current);
    assert_eq!(a.versions[0].text, "alpha body, revised: about lamps");
    assert_eq!(
        a.current,
        eidetic::Hash::of(a.versions[0].text.as_bytes()).to_hex(),
        "the version is the BLAKE3 of the text"
    );
    // Explicit limits, honestly counted.
    let two = read(
        &mut app,
        &graph,
        RetainedPagesLimits {
            max_pages: 2,
            max_versions: 1,
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(two.pages.len(), 2);
    assert_eq!(two.truncated.pages_omitted, 1);
    assert!(two.pages.iter().all(|p| p.versions.len() == 1));
    let tiny = read(
        &mut app,
        &graph,
        RetainedPagesLimits {
            max_bytes: 40,
            ..Default::default()
        },
    )
    .unwrap();
    let carried: usize = tiny
        .pages
        .iter()
        .flat_map(|p| &p.versions)
        .map(|v| v.text.len())
        .sum();
    assert!(carried <= 40, "{carried} bytes carried");
    assert!(tiny.truncated.pages_omitted + tiny.truncated.versions_omitted > 0);
}

#[tokio::test(flavor = "multi_thread")]
async fn a_graph_the_client_does_not_hold_is_refused_and_so_is_the_remote_and_ambient_reach() {
    let fixture = fixture(0x5336_03, "browsing").await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    for foreign in ["mere:sessions", "mere:archive", "00000000-not-held"] {
        let mut asked = graph.clone();
        asked.session = ProjectionSession(foreign.into());
        let refusal = read(&mut app, &asked, Default::default()).unwrap_err();
        assert!(refusal.contains("attached graph"), "{foreign}: {refusal}");
    }
    // An application without the route grant never reaches the endpoint.
    let mere = fixture.reservoir.meres().await.remove(0);
    fixture
        .routes
        .set_access(
            &fixture.reservoir,
            mere.id,
            AppId::new("knot-editor"),
            false,
            true,
            AppId::new("turnstone"),
        )
        .await
        .unwrap();
    app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true)))
        .unwrap();
    let mut ambient = fixture
        .catalog
        .update(|catalog| {
            catalog.open(
                &ambient_mere_route_id("browsing"),
                &context("knot-editor"),
            )
        })
        .await
        .unwrap();
    let crossing = ambient.describe().projections[0].request.clone();
    let seen = ambient.snapshot(crossing).unwrap();
    let refusal = read(&mut ambient, &seen, Default::default()).unwrap_err();
    assert!(refusal.contains("explicit local route"), "{refusal}");
    assert!(matches!(
        ambient
            .invoke(graph_intent(&seen, KEEP_PAGES_INTENT, keep(&[("https://a.test/", "x")])))
            .unwrap(),
        IntentResult::Rejected { .. }
    ));
}

struct Gate(AtomicBool);
impl pandect::RetainedCustody for Gate {
    fn writable(&self) -> Result<(), String> {
        if self.0.load(std::sync::atomic::Ordering::SeqCst) {
            Ok(())
        } else {
            Err("The vault is locked.".into())
        }
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn locked_custody_pauses_keeping_visibly_and_unlocking_resumes() {
    let gate = Arc::new(Gate(AtomicBool::new(true)));
    let fixture = fixture_inner(
        0x5336_04,
        "browsing",
        None,
        None,
        HandlerRegistry::default(),
        Some(gate.clone()),
        None,
    )
    .await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true)))
        .unwrap();
    gate.0.store(false, std::sync::atomic::Ordering::SeqCst);
    let paused = read(&mut app, &graph, Default::default()).unwrap();
    assert_eq!(
        paused.keeping,
        RetainedKeeping::Paused {
            reason: "The vault is locked.".into()
        }
    );
    let refused = app
        .invoke(graph_intent(
            &graph,
            KEEP_PAGES_INTENT,
            keep(&[("https://a.test/", "never written while locked")]),
        ))
        .unwrap();
    assert!(
        matches!(&refused, IntentResult::Rejected { reason } if reason.contains("paused") && reason.contains("locked")),
        "{refused:?}"
    );
    gate.0.store(true, std::sync::atomic::Ordering::SeqCst);
    let resumed = read(&mut app, &graph, Default::default()).unwrap();
    assert_eq!(resumed.keeping, RetainedKeeping::On);
    assert!(resumed.pages.is_empty(), "the paused attempt wrote nothing");
    assert_eq!(
        app.invoke(graph_intent(
            &graph,
            KEEP_PAGES_INTENT,
            keep(&[("https://a.test/", "kept after unlock")]),
        ))
        .unwrap(),
        IntentResult::Accepted
    );
    assert_eq!(read(&mut app, &graph, Default::default()).unwrap().pages.len(), 1);
}

#[tokio::test(flavor = "multi_thread")]
async fn kept_text_and_the_setting_survive_a_cold_reopen_of_the_resident() {
    let fixture = fixture(0x5336_05, "browsing").await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true)))
        .unwrap();
    assert_eq!(
        app.invoke(graph_intent(
            &graph,
            KEEP_PAGES_INTENT,
            keep(&[("https://a.test/", "a body that must outlive the process")]),
        ))
        .unwrap(),
        IntentResult::Accepted
    );
    drop(app);
    let Fixture {
        _root,
        reservoir,
        catalog,
        grants,
        routes,
        ..
    } = fixture;
    // The resident exits: every owner of the reservoir and its stores goes.
    drop((reservoir, catalog, grants, routes));
    let reopened = fixture_inner(
        0x5336_05,
        "browsing",
        None,
        None,
        HandlerRegistry::default(),
        None,
        Some(_root),
    )
    .await;
    let mut app = reopened.attach("graphshell").await;
    let reopened_graph = super::graph(&mut app);
    let graph = reopened_graph;
    let kept = read(&mut app, &graph, Default::default()).unwrap();
    assert_eq!(kept.keeping, RetainedKeeping::On);
    assert_eq!(kept.pages.len(), 1);
    assert_eq!(
        kept.pages[0].versions[0].text,
        "a body that must outlive the process"
    );
}
