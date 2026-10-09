// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A surface the host spawned and routed itself, held in the same content as
//! a document: one command API reaches its web plane, and the host keeps the
//! producer for the work the content does not wrap.

use std::sync::{Arc, Mutex};

use inker::routing::{EngineRouteDecision, SurfaceContract, SurfaceContractMode, SurfaceTargetId};
use inker::{
    Cookie, FocusReason, KeyboardEvent, MouseEvent, SessionNavigationCommand, SessionSpawnRequest,
    SurfaceError, SurfaceFrame, SurfaceProducer, SurfaceSettings, WebSurface, WebSurfaceEvent,
};
use pelt_core::{PeltContent, PeltLane, PeltRoute, PeltRouteSource, PeltRouteState};

type Calls = Arc<Mutex<Vec<String>>>;

/// A web page that records every control it receives and reports one title.
struct Page {
    calls: Calls,
    title: Option<String>,
}

impl Page {
    fn record(&self, call: impl Into<String>) -> Result<(), SurfaceError> {
        self.calls.lock().unwrap().push(call.into());
        Ok(())
    }
}

impl SurfaceProducer for Page {
    fn resize(&mut self, width: u32, height: u32) -> Result<(), SurfaceError> {
        self.record(format!("resize {width}x{height}"))
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
        self.record("focus")
    }

    fn poll_cursor_shape(&mut self) -> Option<inker::CursorShape> {
        None
    }

    fn apply_settings(&mut self, _settings: &SurfaceSettings) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn as_web_surface(&mut self) -> Option<&mut dyn WebSurface> {
        Some(self)
    }
}

impl WebSurface for Page {
    fn navigate_to_url(&mut self, url: &str) -> Result<(), SurfaceError> {
        self.record(format!("navigate {url}"))
    }

    fn navigate_to_string(&mut self, _html: &str) -> Result<(), SurfaceError> {
        self.record("navigate string")
    }

    fn reload(&mut self) -> Result<(), SurfaceError> {
        self.record("reload")
    }

    fn stop(&mut self) -> Result<(), SurfaceError> {
        self.record("stop")
    }

    fn go_back(&mut self) -> Result<(), SurfaceError> {
        self.record("back")
    }

    fn go_forward(&mut self) -> Result<(), SurfaceError> {
        self.record("forward")
    }

    fn can_go_back(&self) -> bool {
        true
    }

    fn can_go_forward(&self) -> bool {
        true
    }

    fn set_cookie(&mut self, _cookie: &Cookie) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn poll_web_event(&mut self) -> Option<WebSurfaceEvent> {
        self.title
            .take()
            .map(|title| WebSurfaceEvent::TitleChanged { title })
    }
}

fn route() -> PeltRoute {
    PeltRoute {
        decision: EngineRouteDecision {
            engine_id: "fake.web".to_owned(),
            surface_contract: SurfaceContract {
                target: SurfaceTargetId::new("node-7"),
                mode: SurfaceContractMode::CompositedTexture,
            },
        },
        source: PeltRouteSource::Automatic,
        state: PeltRouteState::Surface,
    }
}

fn hosted(calls: &Calls) -> PeltContent<String> {
    PeltContent::from_surface(
        Box::new(Page {
            calls: calls.clone(),
            title: Some("Site".to_owned()),
        }),
        (640, 480),
        route(),
    )
}

#[test]
fn a_host_spawned_surface_takes_every_navigation_command_on_its_web_plane() {
    let calls = Calls::default();
    let mut content = hosted(&calls);
    assert_eq!(content.lane(), PeltLane::Surface);
    assert!(content.document().is_none());
    assert_eq!(content.route(), &route());

    for command in [
        SessionNavigationCommand::Back,
        SessionNavigationCommand::Forward,
        SessionNavigationCommand::Reload,
        SessionNavigationCommand::Stop,
    ] {
        let effect = content.command(command);
        assert!(effect.handled && effect.error.is_none(), "{effect:?}");
    }
    assert!(
        content
            .open(SessionSpawnRequest::new("https://site.test/next"))
            .navigated,
        "with no routing of its own, an entry opens on the same surface"
    );
    assert_eq!(content.lane(), PeltLane::Surface);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [
            "back",
            "forward",
            "reload",
            "stop",
            "navigate https://site.test/next"
        ]
    );
}

#[test]
fn the_host_keeps_the_producer_and_its_event_plane() {
    let calls = Calls::default();
    let mut content = hosted(&calls);
    let producer = content
        .surface_producer_mut()
        .expect("a surface lane has a producer");
    producer.move_focus(FocusReason::Programmatic).unwrap();
    assert!(matches!(
        content.poll_web_event(),
        Ok(Some(WebSurfaceEvent::TitleChanged { title })) if title == "Site"
    ));
    assert!(matches!(content.poll_web_event(), Ok(None)));
    assert_eq!(calls.lock().unwrap().as_slice(), ["focus"]);
}

#[test]
fn a_spawned_extent_is_kept_until_the_host_lays_the_surface_out_anew() {
    let calls = Calls::default();
    let mut content = hosted(&calls);
    let rect = pelt_core::WorkspaceRect {
        x: 0.0,
        y: 0.0,
        width: 640.0,
        height: 480.0,
    };
    let _ = content.frame(rect, 1.0);
    assert!(
        calls.lock().unwrap().is_empty(),
        "the spawned extent needs no resize"
    );
    let _ = content.frame(
        pelt_core::WorkspaceRect {
            width: 800.0,
            ..rect
        },
        1.0,
    );
    assert_eq!(calls.lock().unwrap().as_slice(), ["resize 800x480"]);
}
