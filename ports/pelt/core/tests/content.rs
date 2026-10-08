// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! One routed piece of content: every new load is routed, a document lane
//! changes engine as its addresses and media types require, and a load that
//! routes to a surface swaps lanes.

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use inker::routing::{EngineRoutePolicy, EngineRouteRule, SurfaceContractMode};
use inker::{
    DocumentSession, EngineProfileBinding, FocusReason, KeyboardEvent, MouseEvent, SessionClick,
    SessionEngine, SessionError, SessionNavigationCommand, SessionRegistry, SessionScrollKey,
    SessionSpawnRequest, SurfaceEngine, SurfaceEngineRegistry, SurfaceError, SurfaceFrame,
    SurfaceProducer, SurfaceSettings, SurfaceSpawnRequest,
};
use pelt_core::{
    FetchOutcome, Fetched, PeltClock, PeltContent, PeltLane, PeltLoadCommand, PeltRegistries,
    PeltRouteSource, PeltRouteState, PeltTileRequest,
};

type Spawns = Arc<Mutex<Vec<(String, String, Option<String>)>>>;

struct Engine {
    id: &'static str,
    spawns: Spawns,
}

impl SessionEngine<String> for Engine {
    fn engine_id(&self) -> &str {
        self.id
    }

    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        self.spawns.lock().unwrap().push((
            self.id.to_owned(),
            request.address.clone(),
            request.body.clone(),
        ));
        Ok(Box::new(Document {
            id: self.id,
            address: request.address.clone(),
        }))
    }
}

struct Document {
    id: &'static str,
    address: String,
}

impl DocumentSession<String> for Document {
    fn frame(&mut self, _width: u32, _height: u32) -> String {
        format!("{}:{}", self.id, self.address)
    }

    fn scroll_by(&mut self, _dx: f32, _dy: f32) -> bool {
        false
    }

    fn scroll_for_key(&mut self, _key: SessionScrollKey) -> bool {
        false
    }

    fn click_at(&mut self, _x: f32, _y: f32) -> SessionClick {
        SessionClick::Miss
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

type SurfaceSpawns = Arc<Mutex<Vec<(String, String)>>>;

struct Surfaces {
    spawns: SurfaceSpawns,
    fail: bool,
}

impl SurfaceEngine for Surfaces {
    fn engine_id(&self) -> &str {
        "fake.surface"
    }

    fn spawn(
        &self,
        request: &SurfaceSpawnRequest,
    ) -> Result<Box<dyn SurfaceProducer>, SurfaceError> {
        if self.fail {
            return Err(SurfaceError::Unsupported(
                "forced surface failure".to_owned(),
            ));
        }
        self.spawns
            .lock()
            .unwrap()
            .push((request.url.clone(), request.profile.user_data_dir.clone()));
        Ok(Box::new(Surface))
    }
}

struct Surface;

impl SurfaceProducer for Surface {
    fn resize(&mut self, _width: u32, _height: u32) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn set_offset(&mut self, _x: i32, _y: i32) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn acquire_frame(&mut self) -> Result<Option<SurfaceFrame>, SurfaceError> {
        Ok(None)
    }

    fn send_mouse_input(&mut self, _event: MouseEvent) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn send_pointer_input(&mut self, _event: inker::PointerEvent) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn send_keyboard_input(&mut self, _event: KeyboardEvent) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn move_focus(&mut self, _reason: FocusReason) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn poll_cursor_shape(&mut self) -> Option<inker::CursorShape> {
        None
    }

    fn apply_settings(&mut self, _settings: &SurfaceSettings) -> Result<(), SurfaceError> {
        Ok(())
    }
}

struct TestClock;

impl PeltClock for TestClock {
    fn now_ms(&self) -> f64 {
        0.0
    }
}

struct Fixture {
    registries: PeltRegistries<String>,
    spawns: Spawns,
    surfaces: SurfaceSpawns,
}

/// Gemini addresses route to the Gemtext engine, an HTML response to the HTML
/// engine, `web:` addresses to a composited surface, and everything else falls
/// back to HTML.
fn fixture(failing_surface: bool) -> Fixture {
    let spawns = Spawns::default();
    let surfaces = SurfaceSpawns::default();
    let mut sessions = SessionRegistry::new();
    for id in ["fake.gemtext", "fake.html"] {
        sessions.register(Box::new(Engine {
            id,
            spawns: spawns.clone(),
        }));
    }
    let mut surface_engines = SurfaceEngineRegistry::new();
    surface_engines.register(Box::new(Surfaces {
        spawns: surfaces.clone(),
        fail: failing_surface,
    }));
    let mut html = EngineRouteRule::new(
        std::iter::empty::<&str>(),
        "fake.html",
        SurfaceContractMode::CompositedTexture,
    );
    html.content_types = vec!["text/html".to_owned()];
    let policy = EngineRoutePolicy {
        rules: vec![
            html,
            EngineRouteRule::new(
                ["gemini"],
                "fake.gemtext",
                SurfaceContractMode::CompositedTexture,
            ),
            EngineRouteRule::new(
                ["web"],
                "fake.surface",
                SurfaceContractMode::CompositedTexture,
            ),
        ],
        fallback: EngineRouteRule::new(
            std::iter::empty::<&str>(),
            "fake.html",
            SurfaceContractMode::CompositedTexture,
        ),
        per_host_overrides: HashMap::new(),
    };
    Fixture {
        registries: PeltRegistries::new(
            sessions,
            surface_engines,
            policy,
            "pelt-content-test",
            "fake.html",
            EngineProfileBinding {
                user_data_dir: "shared-profile".to_owned(),
            },
        ),
        spawns,
        surfaces,
    }
}

fn open(registries: PeltRegistries<String>, request: PeltTileRequest) -> PeltContent<String> {
    PeltContent::routed(registries, request, Arc::new(|| Box::new(TestClock)))
        .expect("content opens")
}

const CAPSULE: &str = "gemini://capsule.test/index.gmi";

#[test]
fn each_navigation_routes_its_engine_and_history_reopens_the_recorded_one() {
    let fixture = fixture(false);
    let mut content = open(
        fixture.registries,
        PeltTileRequest::new(CAPSULE, (640, 480)),
    );
    assert_eq!(content.lane(), PeltLane::Document);
    assert_eq!(content.route().active_engine(), "fake.gemtext");
    assert_eq!(content.route().source, PeltRouteSource::Automatic);

    let navigated = content.command(SessionNavigationCommand::Address(
        "file:///docs/page.html".to_owned(),
    ));
    assert!(navigated.navigated);
    let document = content.document().unwrap();
    assert_eq!(document.engine_id(), "fake.html");
    assert_eq!(content.route().active_engine(), "fake.html");

    assert!(content.command(SessionNavigationCommand::Back).navigated);
    assert_eq!(content.document().unwrap().engine_id(), "fake.gemtext");
    assert_eq!(content.address(), Some(CAPSULE));
    assert!(content.command(SessionNavigationCommand::Reload).navigated);
    assert_eq!(content.document().unwrap().engine_id(), "fake.gemtext");

    let engines = fixture
        .spawns
        .lock()
        .unwrap()
        .iter()
        .map(|(engine, _, _)| engine.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        engines,
        ["fake.gemtext", "fake.html", "fake.gemtext", "fake.gemtext"]
    );
}

#[test]
fn a_load_that_routes_to_a_surface_swaps_lanes_and_a_document_address_swaps_back() {
    let fixture = fixture(false);
    let profile = EngineProfileBinding {
        user_data_dir: "node-7-profile".to_owned(),
    };
    let mut content = open(
        fixture.registries,
        PeltTileRequest::new(CAPSULE, (640, 480)).with_surface_profile(profile),
    );

    let swapped = content.command(SessionNavigationCommand::Address(
        "web://site.test/".to_owned(),
    ));
    assert!(swapped.handled && swapped.navigated, "{swapped:?}");
    assert_eq!(swapped.reroute, None, "the swap is consumed, not handed on");
    assert_eq!(content.lane(), PeltLane::Surface);
    assert_eq!(content.route().state, PeltRouteState::Surface);
    assert_eq!(content.address(), Some("web://site.test/"));
    assert!(content.document().is_none());
    assert_eq!(
        fixture.surfaces.lock().unwrap().as_slice(),
        [("web://site.test/".to_owned(), "node-7-profile".to_owned())],
        "the surface opens with the content's own profile"
    );
    assert_eq!(content.session_generation(), None);

    let back = content.command(SessionNavigationCommand::Address(
        "gemini://capsule.test/again.gmi".to_owned(),
    ));
    assert!(back.navigated);
    assert_eq!(content.lane(), PeltLane::Document);
    assert_eq!(content.document().unwrap().engine_id(), "fake.gemtext");
    assert_eq!(content.address(), Some("gemini://capsule.test/again.gmi"));
}

#[test]
fn a_failed_lane_swap_keeps_the_document() {
    let fixture = fixture(true);
    let mut content = open(
        fixture.registries,
        PeltTileRequest::new(CAPSULE, (640, 480)),
    );
    let generation = content.session_generation();
    let failed = content.command(SessionNavigationCommand::Address(
        "web://site.test/".to_owned(),
    ));
    assert!(!failed.navigated);
    assert!(
        failed
            .error
            .as_deref()
            .is_some_and(|error| error.contains("could not spawn surface fake.surface"))
    );
    assert_eq!(content.lane(), PeltLane::Document);
    assert_eq!(content.session_generation(), generation);
    assert_eq!(content.address(), Some(CAPSULE));
    assert_eq!(content.route().active_engine(), "fake.gemtext");
}

#[test]
fn host_loading_routes_a_held_body_by_its_media_type() {
    let fixture = fixture(false);
    let mut content = open(
        fixture.registries.with_host_loading(),
        PeltTileRequest::new(CAPSULE, (640, 480)),
    );
    let commands: [PeltLoadCommand; 1] = content.take_load_commands().try_into().unwrap();
    let [PeltLoadCommand::Fetch { request, url }] = commands else {
        panic!("expected one fetch, got {commands:?}");
    };
    assert_eq!(url, CAPSULE);

    content.accept_outcome(
        FetchOutcome {
            request,
            url: CAPSULE.to_owned(),
            result: Ok(Fetched::text(
                Some("text/html; charset=utf-8".to_owned()),
                "<h1>Capsule mirror</h1>",
            )),
        },
        1,
    );
    assert_eq!(content.document().unwrap().engine_id(), "fake.html");
    assert_eq!(content.route().active_engine(), "fake.html");
    let spawns = fixture.spawns.lock().unwrap();
    assert_eq!(
        spawns.last().unwrap(),
        &(
            "fake.html".to_owned(),
            CAPSULE.to_owned(),
            Some("<h1>Capsule mirror</h1>".to_owned())
        )
    );
    assert!(
        spawns.iter().all(|(_, _, body)| body.is_some()),
        "no engine is asked to fetch"
    );
}

#[test]
fn an_engine_pin_holds_across_navigations() {
    let fixture = fixture(false);
    let mut content = open(
        fixture.registries,
        PeltTileRequest::new(CAPSULE, (640, 480)).with_engine_override("fake.html"),
    );
    assert_eq!(content.route().source, PeltRouteSource::UserOverride);
    content.command(SessionNavigationCommand::Address(
        "gemini://capsule.test/next.gmi".to_owned(),
    ));
    assert_eq!(content.document().unwrap().engine_id(), "fake.html");

    assert!(content.set_route_override(None).unwrap());
    assert_eq!(content.route().source, PeltRouteSource::Automatic);
    assert_eq!(content.document().unwrap().engine_id(), "fake.gemtext");
    assert_eq!(
        content.address(),
        Some("gemini://capsule.test/next.gmi"),
        "the held address survives reconstruction"
    );
}
