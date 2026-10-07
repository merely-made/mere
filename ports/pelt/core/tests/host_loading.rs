// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A host-loading controller: the host's transport fetches, the controller
//! owns request identity, supersede, Stop, streaming and typed outcomes, and
//! the document engine only ever receives held bodies.

use std::any::Any;
use std::sync::{Arc, Mutex};

use inker::{
    DocumentSession, SessionButtonState, SessionClick, SessionEngine, SessionError, SessionInput,
    SessionModifiers, SessionNavigationCommand, SessionPointerButton, SessionRegistry,
    SessionScrollKey, SessionSpawnRequest, SurfaceEngineRegistry,
};
use pelt_core::{
    FetchFailure, FetchOutcome, FetchRequestId, Fetched, LoadPhase, PageProgress, PeltClock,
    PeltController, PeltControllerConfig, PeltDocumentState, PeltLoadCommand, PeltLoadMode,
};

type Spawns = Arc<Mutex<Vec<SessionSpawnRequest>>>;

struct BodyEngine {
    spawns: Spawns,
}

impl SessionEngine<String> for BodyEngine {
    fn engine_id(&self) -> &str {
        "body"
    }

    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        self.spawns.lock().unwrap().push(request.clone());
        Ok(Box::new(BodySession {
            address: request.address.clone(),
            body: request.body.clone().unwrap_or_default(),
        }))
    }
}

struct BodySession {
    address: String,
    body: String,
}

impl DocumentSession<String> for BodySession {
    fn frame(&mut self, _width: u32, _height: u32) -> String {
        format!("{}|{}", self.address, self.body)
    }

    fn scroll_by(&mut self, _dx: f32, _dy: f32) -> bool {
        false
    }

    fn scroll_for_key(&mut self, _key: SessionScrollKey) -> bool {
        false
    }

    fn click_at(&mut self, _x: f32, _y: f32) -> SessionClick {
        SessionClick::Navigate("next.gmi".to_owned())
    }

    fn links(&self) -> Vec<inker::SessionLink> {
        Vec::new()
    }

    fn as_any_ref(&self) -> &dyn Any {
        self
    }

    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

fn replace_body(session: &mut dyn DocumentSession<String>, _url: &str, body: &str) -> bool {
    match session.as_any().downcast_mut::<BodySession>() {
        Some(session) => {
            session.body = body.to_owned();
            true
        },
        None => false,
    }
}

struct TestClock;

impl PeltClock for TestClock {
    fn now_ms(&self) -> f64 {
        0.0
    }
}

const START: &str = "gemini://capsule.test/index.gmi";
const NEXT: &str = "gemini://capsule.test/next.gmi";

fn controller(spawns: &Spawns) -> PeltController<String> {
    let mut sessions = SessionRegistry::new();
    sessions.register(Box::new(BodyEngine {
        spawns: spawns.clone(),
    }));
    PeltController::new(
        sessions,
        SurfaceEngineRegistry::new(),
        PeltControllerConfig::new("body", START, (640, 480)).with_host_loading(),
        TestClock,
    )
    .unwrap()
}

fn only_fetch(controller: &mut PeltController<String>, url: &str) -> FetchRequestId {
    match controller.take_load_commands().as_slice() {
        [
            PeltLoadCommand::Fetch {
                request,
                url: fetched,
            },
        ] if fetched == url => *request,
        other => panic!("expected one fetch of {url}, got {other:?}"),
    }
}

fn page(request: FetchRequestId, url: &str, body: &str) -> FetchOutcome {
    FetchOutcome {
        request,
        url: url.to_owned(),
        result: Ok(Fetched::text(Some("text/gemini".to_owned()), body)),
    }
}

fn fragment(request: FetchRequestId, url: &str, bytes: &[u8]) -> PageProgress {
    PageProgress {
        request,
        url: url.to_owned(),
        response_url: url.to_owned(),
        content_type: Some("text/gemini".to_owned()),
        bytes: bytes.to_vec(),
    }
}

fn click() -> SessionInput {
    SessionInput::PointerButton {
        x: 1.0,
        y: 1.0,
        button: SessionPointerButton::Primary,
        state: SessionButtonState::Pressed,
        modifiers: SessionModifiers::default(),
    }
}

/// Open the start page through the host and present it.
fn opened(spawns: &Spawns) -> PeltController<String> {
    let mut controller = controller(spawns);
    let request = only_fetch(&mut controller, START);
    controller.accept_outcome(page(request, START, "# Start"), 1);
    controller.mark_document_presented();
    controller
}

#[test]
fn the_engine_only_ever_receives_held_bodies() {
    let spawns = Spawns::default();
    let mut controller = controller(&spawns);
    assert_eq!(controller.load_mode(), PeltLoadMode::Host);
    let request = only_fetch(&mut controller, START);
    assert_eq!(
        controller.document_state(),
        &PeltDocumentState::Fetching {
            address: START.to_owned(),
            request,
        }
    );
    assert_eq!(
        controller.frame(10, 10),
        format!("{START}|"),
        "an empty document while fetching"
    );

    let opened = controller.accept_outcome(page(request, START, "# Start"), 1);
    assert!(opened.handled && opened.redraw);
    assert!(matches!(
        controller.document_state(),
        PeltDocumentState::Loading { .. }
    ));
    controller.mark_document_presented();
    assert_eq!(controller.document_state(), &PeltDocumentState::Ready);
    assert_eq!(controller.frame(10, 10), format!("{START}|# Start"));
    assert_eq!(controller.session_generation(), 2);
    assert!(
        !controller.can_go_back(),
        "the first load replaces the opening entry"
    );
    assert!(
        spawns
            .lock()
            .unwrap()
            .iter()
            .all(|request| request.body.is_some()),
        "no spawn asks the engine to fetch"
    );
}

#[test]
fn a_superseded_load_cannot_surface() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    controller.input(click());
    let first = only_fetch(&mut controller, NEXT);
    controller.command(SessionNavigationCommand::Address("later.gmi".to_owned()));
    let later = "gemini://capsule.test/later.gmi";
    let commands = controller.take_load_commands();
    let [
        PeltLoadCommand::Cancel { request: cancelled },
        PeltLoadCommand::Fetch {
            request: second,
            url,
        },
    ] = commands.as_slice()
    else {
        panic!("expected cancel then fetch, got {commands:?}");
    };
    assert_eq!((*cancelled, url.as_str()), (first, later));

    let stale = controller.accept_outcome(page(first, NEXT, "# Next"), 2);
    assert_eq!(
        stale,
        Default::default(),
        "the superseded answer changes nothing"
    );
    assert_eq!(controller.address(), START);

    controller.accept_outcome(page(*second, later, "# Later"), 3);
    assert_eq!(controller.address(), later);
    assert!(controller.can_go_back());
    assert_eq!(controller.frame(10, 10), format!("{later}|# Later"));
}

#[test]
fn stop_cancels_the_exact_request_and_keeps_the_current_page() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    let stopped = controller.command(SessionNavigationCommand::Stop);
    assert!(stopped.handled);
    assert_eq!(
        controller.take_load_commands(),
        [PeltLoadCommand::Cancel { request }]
    );
    assert_eq!(controller.document_state(), &PeltDocumentState::Ready);
    assert_eq!(controller.pending_request(), None);
    assert_eq!(controller.address(), START);

    let late = FetchOutcome {
        request,
        url: NEXT.to_owned(),
        result: Err(FetchFailure::Cancelled),
    };
    assert_eq!(controller.accept_outcome(late, 4), Default::default());
    assert_eq!(controller.frame(10, 10), format!("{START}|# Start"));
}

#[test]
fn reload_is_a_new_request_for_the_same_entry() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    let before = controller.session_generation();
    controller.command(SessionNavigationCommand::Reload);
    let request = only_fetch(&mut controller, START);
    assert!(
        controller.loaded_document().is_none(),
        "the old body is not reused"
    );
    controller.accept_outcome(page(request, START, "# Start, again"), 5);
    assert_eq!(controller.session_generation(), before + 1);
    assert!(!controller.can_go_back(), "reload does not grow history");
    assert_eq!(controller.frame(10, 10), format!("{START}|# Start, again"));

    controller.command(SessionNavigationCommand::Reload);
    let again = only_fetch(&mut controller, START);
    assert_ne!(
        again, request,
        "every reload mints its own request identity"
    );
}

#[test]
fn a_streamed_prefix_opens_and_later_fragments_replace_in_place() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    controller.set_body_replacer(replace_body);
    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);

    let first = controller.accept_progress(fragment(request, NEXT, b"# Ne"), 6);
    assert!(first.navigated, "the first prefix commits the navigation");
    assert_eq!(controller.address(), NEXT);
    let generation = controller.session_generation();
    assert_eq!(controller.frame(10, 10), format!("{NEXT}|# Ne"));

    let second = controller.accept_progress(fragment(request, NEXT, b"xt"), 7);
    assert!(second.redraw && !second.navigated);
    assert_eq!(controller.frame(10, 10), format!("{NEXT}|# Next"));
    assert!(matches!(
        controller.load_phase(),
        Some(LoadPhase::Streaming {
            received_bytes: 6,
            ..
        })
    ));

    controller.accept_outcome(page(request, NEXT, "# Next\n=> more.gmi"), 8);
    assert_eq!(
        controller.session_generation(),
        generation,
        "one session throughout"
    );
    assert_eq!(
        controller.frame(10, 10),
        format!("{NEXT}|# Next\n=> more.gmi")
    );
    let (url, document) = controller.loaded_document().unwrap();
    assert_eq!(url, NEXT);
    assert_eq!(document.acquired_at_ms, 6, "first admission is kept");
    assert!(controller.can_go_back() && !controller.can_go_forward());
}

#[test]
fn without_a_replacer_the_settled_body_reopens_once() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    controller.accept_progress(fragment(request, NEXT, b"# Ne"), 6);
    let generation = controller.session_generation();
    assert!(
        !controller
            .accept_progress(fragment(request, NEXT, b"xt"), 7)
            .redraw
    );
    let settled = controller.accept_outcome(page(request, NEXT, "# Next"), 8);
    assert!(settled.redraw && !settled.navigated);
    assert_eq!(controller.session_generation(), generation + 1);
    assert_eq!(controller.frame(10, 10), format!("{NEXT}|# Next"));
    assert!(controller.can_go_back());
}

#[test]
fn conversations_downloads_and_failures_leave_the_page_intact() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    let generation = controller.session_generation();

    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    let input = FetchFailure::InputRequired {
        url: NEXT.to_owned(),
        prompt: "Search".to_owned(),
        sensitive: false,
    };
    let asked = controller.accept_outcome(
        FetchOutcome {
            request,
            url: NEXT.to_owned(),
            result: Err(input.clone()),
        },
        9,
    );
    assert!(asked.handled);
    assert_eq!(
        controller.document_state(),
        &PeltDocumentState::Awaiting {
            address: NEXT.to_owned(),
            failure: input,
        }
    );
    assert_eq!(controller.address(), START);

    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    let archive = Fetched {
        content_type: Some("application/zip".to_owned()),
        content_disposition: None,
        bytes: vec![0x50, 0x4b],
        body: String::new(),
    };
    let download = controller.accept_outcome(
        FetchOutcome {
            request,
            url: NEXT.to_owned(),
            result: Ok(archive.clone()),
        },
        10,
    );
    let download = download.download.expect("a download is handed to the host");
    assert_eq!((download.url.as_str(), download.fetched), (NEXT, archive));
    assert_eq!(controller.document_state(), &PeltDocumentState::Ready);

    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    let failed = controller.accept_outcome(
        FetchOutcome {
            request,
            url: NEXT.to_owned(),
            result: Err(FetchFailure::Failed("51 not found".to_owned())),
        },
        11,
    );
    assert_eq!(failed.error.as_deref(), Some("51 not found"));
    assert!(matches!(
        controller.document_state(),
        PeltDocumentState::Error { .. }
    ));
    assert_eq!(controller.session_generation(), generation);
    assert_eq!(controller.address(), START);
}

#[test]
fn back_refetches_the_entry_and_commits_on_its_answer() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    controller.accept_outcome(page(request, NEXT, "# Next"), 12);
    assert!(controller.can_go_back());

    controller.command(SessionNavigationCommand::Back);
    let request = only_fetch(&mut controller, START);
    assert_eq!(
        controller.address(),
        NEXT,
        "nothing commits before the answer"
    );
    controller.accept_outcome(page(request, START, "# Start"), 13);
    assert_eq!(controller.address(), START);
    assert!(controller.can_go_forward());
}

#[test]
fn stopping_a_presented_prefix_keeps_it_ready_and_shown() {
    let spawns = Spawns::default();
    let mut controller = opened(&spawns);
    controller.set_body_replacer(replace_body);
    controller.input(click());
    let request = only_fetch(&mut controller, NEXT);
    controller.accept_progress(fragment(request, NEXT, b"# Ne"), 14);
    controller.mark_document_presented();
    assert_eq!(controller.document_state(), &PeltDocumentState::Ready);

    controller.command(SessionNavigationCommand::Stop);
    assert_eq!(
        controller.take_load_commands(),
        [PeltLoadCommand::Cancel { request }]
    );
    assert_eq!(controller.document_state(), &PeltDocumentState::Ready);
    assert_eq!(
        controller.address(),
        NEXT,
        "the shown prefix stays committed"
    );
    assert_eq!(controller.frame(10, 10), format!("{NEXT}|# Ne"));
    assert_eq!(
        controller.load_phase(),
        Some(&LoadPhase::Stopped { received_bytes: 4 })
    );
}
