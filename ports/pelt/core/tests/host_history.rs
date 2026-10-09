// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A host-history controller: the host keeps history, the controller keeps
//! only its current entry, hands its document's navigations up, and opens
//! whatever entry the host chooses.

use std::any::Any;
use std::sync::{Arc, Mutex};

use inker::{
    DocumentSession, SessionButtonState, SessionClick, SessionEngine, SessionError,
    SessionFormMethod, SessionFormSubmission, SessionInput, SessionModifiers,
    SessionNavigationCommand, SessionPointerButton, SessionRegistry, SessionScrollKey,
    SessionSpawnRequest, SurfaceEngineRegistry,
};
use pelt_core::{
    PeltClock, PeltController, PeltControllerConfig, PeltDocumentState, PeltHistoryMode,
    PeltLoadCommand, PeltNavigationCause,
};

type Spawns = Arc<Mutex<Vec<SessionSpawnRequest>>>;

struct Engine {
    spawns: Spawns,
}

impl SessionEngine<String> for Engine {
    fn engine_id(&self) -> &str {
        "fake"
    }

    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        self.spawns.lock().unwrap().push(request.clone());
        Ok(Box::new(Page {
            address: request.address.clone(),
        }))
    }
}

struct Page {
    address: String,
}

impl DocumentSession<String> for Page {
    fn frame(&mut self, _width: u32, _height: u32) -> String {
        self.address.clone()
    }

    fn scroll_by(&mut self, _dx: f32, _dy: f32) -> bool {
        false
    }

    fn scroll_for_key(&mut self, _key: SessionScrollKey) -> bool {
        false
    }

    /// Left of x=10 is a link; right of it submits a form: a GET form, or on
    /// an `upload` page a mutation endpoint the host completes (a POST).
    fn click_at(&mut self, x: f32, _y: f32) -> SessionClick {
        if x < 10.0 {
            SessionClick::Navigate("next.html".to_owned())
        } else {
            SessionClick::Submit("search.html".to_owned())
        }
    }

    fn form_submission(&mut self, action: &str) -> SessionFormSubmission {
        if self.address.contains("upload") {
            return SessionFormSubmission {
                action: action.to_owned(),
                method: SessionFormMethod::Post,
                fields: Vec::new(),
            };
        }
        SessionFormSubmission {
            action: action.to_owned(),
            method: SessionFormMethod::Get,
            fields: vec![("q".to_owned(), "cedar".to_owned())],
        }
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

struct TestClock;

impl PeltClock for TestClock {
    fn now_ms(&self) -> f64 {
        0.0
    }
}

fn controller(spawns: &Spawns, config: PeltControllerConfig) -> PeltController<String> {
    let mut sessions = SessionRegistry::new();
    sessions.register(Box::new(Engine {
        spawns: spawns.clone(),
    }));
    PeltController::new(sessions, SurfaceEngineRegistry::new(), config, TestClock).unwrap()
}

fn host_history(spawns: &Spawns) -> PeltController<String> {
    controller(
        spawns,
        PeltControllerConfig::new("fake", "docs/index.html", (640, 480)).with_host_history(),
    )
}

fn press(x: f32, modifiers: SessionModifiers) -> SessionInput {
    SessionInput::PointerButton {
        x,
        y: 1.0,
        button: SessionPointerButton::Primary,
        state: SessionButtonState::Pressed,
        modifiers,
    }
}

#[test]
fn links_and_forms_become_requests_for_the_host() {
    let spawns = Spawns::default();
    let mut controller = host_history(&spawns);
    assert_eq!(controller.history_mode(), PeltHistoryMode::Host);
    let generation = controller.session_generation();

    let modifiers = SessionModifiers {
        control: true,
        ..SessionModifiers::default()
    };
    let link = controller.input(press(1.0, modifiers));
    assert!(link.handled && !link.navigated);
    let navigation = link.navigation.expect("the link is handed to the host");
    assert_eq!(navigation.request.address, "docs/next.html");
    assert_eq!(navigation.request.viewport, (640, 480));
    assert_eq!(navigation.cause, PeltNavigationCause::Link { modifiers });

    let form = controller.input(press(20.0, SessionModifiers::default()));
    let navigation = form.navigation.expect("the GET form is handed to the host");
    assert_eq!(navigation.request.address, "docs/search.html?q=cedar");
    assert_eq!(navigation.cause, PeltNavigationCause::FormGet);

    assert_eq!(controller.address(), "docs/index.html", "nothing loaded");
    assert_eq!(controller.session_generation(), generation);
    assert_eq!(spawns.lock().unwrap().len(), 1, "only the opening spawn");
}

#[test]
fn back_and_forward_belong_to_the_host_and_open_replaces_in_place() {
    let spawns = Spawns::default();
    let mut controller = host_history(&spawns);
    let back = controller.command(SessionNavigationCommand::Back);
    assert!(!back.handled, "the host's history traverses");
    assert_eq!(back, Default::default());

    let opened = controller.open(SessionSpawnRequest::new("docs/other.html"));
    assert!(opened.handled && opened.navigated);
    assert_eq!(controller.address(), "docs/other.html");
    assert_eq!(controller.session_generation(), 2);
    assert!(!controller.can_go_back() && !controller.can_go_forward());

    // The host's own Address request opens in place too, resolved against
    // the current document.
    controller.command(SessionNavigationCommand::Address("third.html".to_owned()));
    assert_eq!(controller.address(), "docs/third.html");
    assert!(!controller.can_go_back());
    assert_eq!(
        spawns.lock().unwrap().last().unwrap().viewport,
        (640, 480),
        "an opened request takes the controller's viewport"
    );
}

#[test]
fn open_with_a_held_body_skips_the_transport_and_supersedes_a_fetch() {
    let spawns = Spawns::default();
    let mut controller = controller(
        &spawns,
        PeltControllerConfig::new("fake", "gemini://capsule.test/", (640, 480))
            .with_host_loading()
            .with_host_history(),
    );
    let [PeltLoadCommand::Fetch { request, .. }] = controller.take_load_commands()[..] else {
        panic!("expected the opening fetch");
    };

    let opened = controller
        .open(SessionSpawnRequest::new("gemini://capsule.test/held.gmi").with_body("# Held"));
    assert!(opened.navigated);
    assert_eq!(
        controller.take_load_commands(),
        [PeltLoadCommand::Cancel { request }],
        "the held body supersedes the fetch and asks for no other"
    );
    assert_eq!(controller.pending_request(), None);
    assert_eq!(controller.address(), "gemini://capsule.test/held.gmi");
    assert!(matches!(
        controller.document_state(),
        PeltDocumentState::Loading { .. }
    ));
    assert_eq!(
        spawns.lock().unwrap().last().unwrap().body.as_deref(),
        Some("# Held")
    );
}

#[test]
fn linear_history_is_unchanged() {
    let spawns = Spawns::default();
    let mut controller = controller(
        &spawns,
        PeltControllerConfig::new("fake", "docs/index.html", (640, 480)),
    );
    assert_eq!(controller.history_mode(), PeltHistoryMode::Linear);
    let link = controller.input(press(1.0, SessionModifiers::default()));
    assert!(link.navigated && link.navigation.is_none());
    assert_eq!(controller.address(), "docs/next.html");
    assert!(controller.can_go_back());

    // open() replaces the current entry even in linear mode.
    controller.open(SessionSpawnRequest::new("docs/replaced.html"));
    assert_eq!(controller.address(), "docs/replaced.html");
    assert!(controller.command(SessionNavigationCommand::Back).navigated);
    assert_eq!(controller.address(), "docs/index.html");
    assert!(
        controller
            .command(SessionNavigationCommand::Forward)
            .navigated
    );
    assert_eq!(controller.address(), "docs/replaced.html");
}

/// A POST (a smolweb mutation endpoint) is handed to a host-history host to
/// collect, confirm and send, resolved against the document; nothing loads.
#[test]
fn a_post_submission_is_handed_to_the_host() {
    let spawns = Spawns::default();
    let mut controller = controller(
        &spawns,
        PeltControllerConfig::new("fake", "titan://capsule.test/docs/upload.gmi", (640, 480))
            .with_host_history(),
    );
    let spawned = spawns.lock().unwrap().len();
    let effect = controller.input(press(20.0, SessionModifiers::default()));
    assert!(effect.handled, "{effect:?}");
    assert_eq!(effect.error, None);
    assert_eq!(effect.navigation, None, "a POST is not a navigation");
    let submission = effect.submission.expect("the submission is handed up");
    assert_eq!(submission.method, SessionFormMethod::Post);
    assert_eq!(submission.action, "titan://capsule.test/docs/search.html");
    assert!(submission.fields.is_empty());
    assert_eq!(spawns.lock().unwrap().len(), spawned, "nothing loads");
}

/// A linear-history controller still refuses a POST: it has no host to
/// collect the body.
#[test]
fn a_linear_controller_still_refuses_a_post() {
    let spawns = Spawns::default();
    let mut controller = controller(
        &spawns,
        PeltControllerConfig::new("fake", "titan://capsule.test/docs/upload.gmi", (640, 480)),
    );
    let effect = controller.input(press(20.0, SessionModifiers::default()));
    assert_eq!(effect.submission, None);
    assert!(
        effect
            .error
            .as_deref()
            .is_some_and(|error| error.contains("POST form submission"))
    );
}
