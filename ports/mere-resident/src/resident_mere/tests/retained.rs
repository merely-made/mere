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
use eidetic::kept_text::{
    KEEP_PAGES_INTENT, KEEP_SETTING_INTENT, KeepPagesV1, KeepSettingV1, KeptPage,
};
use dramatis_resident::custody_host::ResidentCustody;
use personae::IdentityProvider as _;

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
        fetches: Vec::new(),
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


fn plaintext_in(root: &std::path::Path, needle: &str) -> bool {
    fn walk(dir: &std::path::Path, needle: &[u8], hit: &mut bool) {
        for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, needle, hit);
            } else if path.extension().is_some_and(|e| e == "redb")
                && path.to_string_lossy().contains("meres")
            {
                let bytes = std::fs::read(&path).unwrap();
                *hit |= bytes.windows(needle.len()).any(|w| w == needle);
            }
        }
    }
    let mut hit = false;
    walk(root, needle.as_bytes(), &mut hit);
    hit
}

#[tokio::test(flavor = "multi_thread")]
async fn nothing_is_kept_in_plaintext_when_no_custody_is_configured() {
    let fixture = fixture_inner(
        0x5336_07,
        "browsing",
        None,
        None,
        HandlerRegistry::default(),
        Some(crate::retained_custody::RetainedSealing::unconfigured()),
        None,
    )
    .await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    let state = read(&mut app, &graph, Default::default()).unwrap();
    assert!(
        matches!(&state.keeping, RetainedKeeping::Paused { reason } if reason.contains("no wallet custody")),
        "{:?}",
        state.keeping
    );
    for intent in [
        (KEEP_SETTING_INTENT, setting(true)),
        (KEEP_PAGES_INTENT, keep(&[("https://a.test/", "never in plaintext")])),
    ] {
        let refused = app.invoke(graph_intent(&graph, intent.0, intent.1)).unwrap();
        assert!(matches!(refused, IntentResult::Rejected { .. }), "{refused:?}");
    }
    assert!(!plaintext_in(fixture._root.path(), "never in plaintext"));
}

/// The custodian answers "locked" when a persona has no staged sealing key.
/// An unlocked vault must say what is missing, not that it is locked.
#[tokio::test(flavor = "multi_thread")]
async fn an_unlocked_vault_without_a_sealing_key_does_not_read_as_locked() {
    let vault_root = tempfile::tempdir().unwrap();
    let (host, _) = retained_custody(vault_root.path());
    assert!(!host.is_locked());
    let empty_wallet = tempfile::tempdir().unwrap();
    let sealing = crate::retained_custody::RetainedSealing::custodial(
        host.clone(),
        empty_wallet.path().to_path_buf(),
        PersonaId::from_uuid(Uuid::from_u128(0x5336_77)),
        personae::ProfileId("retained".into()),
    );
    let fixture = fixture_inner(
        0x5336_08,
        "browsing",
        None,
        None,
        HandlerRegistry::default(),
        Some(sealing),
        None,
    )
    .await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    let refused = app
        .invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true)))
        .unwrap();
    let IntentResult::Rejected { reason } = refused else {
        panic!("{refused:?}")
    };
    assert!(reason.contains("no sealing key"), "{reason}");
    assert!(!reason.contains("is locked"), "{reason}");
}

#[tokio::test(flavor = "multi_thread")]
async fn locking_the_vault_pauses_keeping_and_reading_and_unlocking_resumes() {
    let fixture = fixture(0x5336_04, "browsing").await;
    let host = fixture.retained_host.clone().unwrap();
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true))).unwrap(),
        IntentResult::Accepted
    );
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_PAGES_INTENT, keep(&[("https://a.test/", "kept before the lock")]))).unwrap(),
        IntentResult::Accepted
    );
    host.lock_vault().unwrap();
    let paused = read(&mut app, &graph, Default::default()).unwrap();
    assert!(
        matches!(&paused.keeping, RetainedKeeping::Paused { reason } if reason.contains("locked")),
        "{:?}",
        paused.keeping
    );
    assert!(paused.pages.is_empty(), "locked custody cannot open the text");
    let refused = app
        .invoke(graph_intent(&graph, KEEP_PAGES_INTENT, keep(&[("https://b.test/", "never written while locked")])))
        .unwrap();
    assert!(
        matches!(&refused, IntentResult::Rejected { reason } if reason.contains("paused") && reason.contains("locked")),
        "{refused:?}"
    );
    host.unlock_with_passphrase(super::VAULT_PASSPHRASE).unwrap();
    let resumed = read(&mut app, &graph, Default::default()).unwrap();
    assert_eq!(resumed.keeping, RetainedKeeping::On);
    assert_eq!(resumed.pages.len(), 1, "the locked attempt wrote nothing");
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_PAGES_INTENT, keep(&[("https://b.test/", "kept after unlock")]))).unwrap(),
        IntentResult::Accepted
    );
    assert_eq!(read(&mut app, &graph, Default::default()).unwrap().pages.len(), 2);
}

#[tokio::test(flavor = "multi_thread")]
async fn values_are_sealed_at_rest_and_another_lane_cannot_open_them() {
    use castellan_client::custody_payload::{PayloadBytes, PayloadOperation, PayloadRequest};
    let fixture = fixture(0x5336_08, "browsing").await;
    let host = fixture.retained_host.clone().unwrap();
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true))).unwrap();
    let secret = "a sentence only the person should read";
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_PAGES_INTENT, keep(&[("https://a.test/", secret)]))).unwrap(),
        IntentResult::Accepted
    );
    assert_eq!(read(&mut app, &graph, Default::default()).unwrap().pages.len(), 1);
    let mere = fixture.reservoir.meres().await.remove(0);
    let persona = fixture.reservoir.persona();
    let shared_root = fixture._root.path().to_path_buf();
    let Fixture { _root, reservoir, catalog, grants, routes, .. } = fixture;
    drop((app, reservoir, catalog, grants, routes));
    assert!(!plaintext_in(_root.path(), secret), "no plaintext at rest");
    assert!(!plaintext_in(_root.path(), "should read"));
    // Take a stored envelope and offer it to a different lane's scope.
    let raw = pandect::open_mere_retained_backend(&shared_root, persona, mere.id).unwrap();
    use muniment::Backend;
    let key = raw.list("blob/").await.unwrap().remove(0);
    let envelope: serde_json::Value =
        serde_json::from_slice(&raw.get(&key).await.unwrap().unwrap()).unwrap();
    let hash: eidetic::Hash = serde_json::from_value(envelope["hash"].clone()).unwrap();
    let marker: eidetic::SealedBlobRef = serde_json::from_value(envelope["seal"].clone()).unwrap();
    let ciphertext: Vec<u8> = serde_json::from_value(envelope["ciphertext"].clone()).unwrap();
    let wallet = PersonaId::from_uuid(Uuid::from_u128(0x5336_77));
    let master = host.master_public_key().to_bytes();
    let ask = |lane: &str| {
        let request = PayloadRequest {
            profile: personae::ProfileId("retained".into()),
            master,
            content_hash: hash,
            operation: PayloadOperation::Unseal {
                bytes: PayloadBytes(ciphertext.clone()),
                marker: marker.clone(),
            },
            custody_epoch: None,
        };
        let scope = serde_json::to_vec(&(lane, "retained", &master)).unwrap();
        host.payload(&_root.path().join("retained-custody"), wallet, request, scope)
    };
    assert!(ask("graphshell.retained-content/custody-v1").is_err(), "another lane is refused");
    assert!(ask("turnstone.retained-clips/custody-v1").is_err());
    assert!(
        ask(crate::retained_custody::RETAINED_LANE).is_ok(),
        "the shared retained lane opens it"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn sealed_text_and_the_setting_survive_a_cold_reopen_of_the_resident() {
    let fixture = fixture(0x5336_05, "browsing").await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true))).unwrap();
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_PAGES_INTENT, keep(&[("https://a.test/", "a body that must outlive the process")]))).unwrap(),
        IntentResult::Accepted
    );
    drop(app);
    let Fixture { _root, reservoir, catalog, grants, routes, retained_host, .. } = fixture;
    // The resident exits: every owner of the reservoir, its stores and custody goes.
    drop((reservoir, catalog, grants, routes, retained_host));
    let reopened = fixture_inner(0x5336_05, "browsing", None, None, HandlerRegistry::default(), None, Some(_root)).await;
    // A cold custodian starts locked-or-unlocked per its vault; unlock it.
    let mut app = reopened.attach("graphshell").await;
    let graph = super::graph(&mut app);
    let kept = read(&mut app, &graph, Default::default()).unwrap();
    assert_eq!(kept.keeping, RetainedKeeping::On);
    assert_eq!(kept.pages.len(), 1);
    assert_eq!(kept.pages[0].versions[0].text, "a body that must outlive the process");
}

#[tokio::test(flavor = "multi_thread")]
async fn an_explicit_fetch_is_recorded_beside_the_text_it_produced() {
    use eidetic::kept_text::{FetchRecordV1, fetch_record_key};
    let fixture = fixture(0x5336_06, "browsing").await;
    let mut app = fixture.attach("graphshell").await;
    let graph = graph(&mut app);
    app.invoke(graph_intent(&graph, KEEP_SETTING_INTENT, setting(true))).unwrap();
    let text = "a typed address, fetched once";
    let record = FetchRecordV1 {
        requested: "http://127.0.0.1:9/a".into(),
        final_url: "http://127.0.0.1:9/a".into(),
        fetched_at_ms: 1234,
        status: 200,
        content_type: Some("text/html".into()),
        html_bytes: 400,
        text_bytes: text.len() as u64,
        text_version: eidetic::Hash::of(text.as_bytes()).to_hex(),
    };
    let payload = serde_json::to_vec(&KeepPagesV1 {
        pages: vec![KeptPage { url: record.final_url.clone(), text: text.into(), at_ms: 0, private: false }],
        fetches: vec![record.clone()],
    })
    .unwrap();
    assert_eq!(
        app.invoke(graph_intent(&graph, KEEP_PAGES_INTENT, payload)).unwrap(),
        IntentResult::Accepted
    );
    let mere = fixture.reservoir.meres().await.remove(0);
    let persona = fixture.reservoir.persona();
    let (host, sealing) = (fixture.retained_host.clone().unwrap(), ());
    let _ = sealing;
    let Fixture { _root, reservoir, catalog, grants, routes, .. } = fixture;
    drop((app, reservoir, catalog, grants, routes));
    // Reading the record back goes through the same custody: sealed at rest.
    assert!(!plaintext_in(_root.path(), "fetched once"));
    let (_, sealing) = (host, crate::retained_custody::RetainedSealing::unconfigured());
    let _ = sealing;
    let raw = pandect::open_mere_retained_backend(_root.path(), persona, mere.id).unwrap();
    use muniment::Backend;
    let key = fetch_record_key(&record.final_url, record.fetched_at_ms);
    assert!(raw.list("retained-text/fetch/v1/").await.unwrap().contains(&key), "the record exists under its key");
}
