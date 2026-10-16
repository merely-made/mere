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

#[test]
fn mailbox_polling_follows_admitted_native_window_custody() {
    use pelt_core::{PeltControllerPool, PeltPlacement, WorkspaceRect};
    let calls: Calls = Default::default();
    let mut pool: PeltControllerPool<u64, u64, String> = PeltControllerPool::new();
    pool.insert(1, hosted(&calls));
    let placement = |window, visible, width| PeltPlacement {
        content: 1,
        window,
        rect: WorkspaceRect {
            x: 0.0,
            y: 0.0,
            width,
            height: 60.0,
        },
        visible,
        focused: false,
        scale_factor: 1.0,
    };
    pool.set_placements([
        (10, placement(7, true, 80.0)),
        (11, placement(8, false, 80.0)),
        (12, placement(9, true, 0.0)),
    ]);
    let before = calls.lock().unwrap().clone();
    assert!(pool.window_has_surface(7));
    assert!(!pool.window_has_surface(8));
    assert!(!pool.window_has_surface(9));
    assert_eq!(
        *calls.lock().unwrap(),
        before,
        "poll custody observation is read-only"
    );
    pool.set_placements([(10, placement(8, true, 80.0))]);
    assert!(!pool.window_has_surface(7));
    assert!(pool.window_has_surface(8));
    pool.remove_content(&1);
    assert!(!pool.window_has_surface(8));
}

/// A web page that records every control it receives and reports one title.
struct Page {
    calls: Calls,
    pointers: Arc<Mutex<Vec<inker::PointerEvent>>>,
    acquisitions: Arc<std::sync::atomic::AtomicUsize>,
    title: Option<String>,
    fail_offset: Option<(i32, i32)>,
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

    fn set_offset(&mut self, x: i32, y: i32) -> Result<(), SurfaceError> {
        if self.fail_offset == Some((x, y)) {
            return Err(SurfaceError::InputFailed("offset rejected".into()));
        }
        Ok(())
    }

    fn acquire_frame(&mut self) -> Result<Option<SurfaceFrame>, SurfaceError> {
        self.acquisitions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(None)
    }

    fn send_mouse_input(&mut self, _event: MouseEvent) -> Result<(), SurfaceError> {
        Ok(())
    }

    fn send_pointer_input(&mut self, event: inker::PointerEvent) -> Result<(), SurfaceError> {
        self.pointers.lock().unwrap().push(event.clone());
        self.record(format!(
            "pointer {:?} {:?} ctrl={} at={},{}",
            event.phase, event.button, event.modifiers.ctrl, event.position.x, event.position.y
        ))
    }

    fn send_drag_input(&mut self, event: inker::DragEvent) -> Result<(), SurfaceError> {
        self.record(format!("drag {event:?}"))
    }

    fn send_wheel_input(&mut self, event: inker::SurfaceWheelEvent) -> Result<(), SurfaceError> {
        self.record(format!("wheel {event:?}"))
    }

    fn send_keyboard_input(&mut self, event: KeyboardEvent) -> Result<(), SurfaceError> {
        self.record(format!("keyboard {event:?}"))
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
            pointers: Default::default(),
            acquisitions: Default::default(),
            fail_offset: None,
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

#[test]
fn neutral_surface_press_preserves_button_and_modifiers_in_pointer_contract() {
    let calls = Calls::default();
    let mut content = hosted(&calls);
    let effect = content.input(
        inker::SessionInput::PointerButton {
            x: 10.0,
            y: 20.0,
            button: inker::SessionPointerButton::Auxiliary,
            state: inker::SessionButtonState::Pressed,
            modifiers: inker::SessionModifiers {
                control: true,
                ..Default::default()
            },
        },
        2.0,
    );
    assert!(effect.handled);
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        ["pointer Down Some(Middle) ctrl=true at=20,40"]
    );
}

#[test]
fn pool_pointer_capture_qualifies_identity_button_and_focus_loss() {
    use inker::{
        MouseButton, PhysicalPosition, PointerButtons, PointerPhase, SessionInput, SessionModifiers,
    };
    use pelt_core::{PeltControllerPool, PeltInput, PeltPlacement, WorkspaceRect, mouse_pointer};
    let calls = Calls::default();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, hosted(&calls));
    pool.set_placements([(
        10,
        PeltPlacement {
            content: 1,
            window: 0,
            rect: WorkspaceRect::new(20.0, 30.0, 100.0, 100.0),
            visible: true,
            focused: true,
            scale_factor: 2.0,
        },
    )]);
    let event = |id, phase, button, buttons| {
        PeltInput::Pointer(mouse_pointer(
            id,
            phase,
            PhysicalPosition { x: 10.0, y: 20.0 },
            button,
            buttons,
            SessionModifiers::default(),
        ))
    };
    assert!(
        pool.input(
            &10,
            event(
                9,
                PointerPhase::Down,
                Some(MouseButton::Middle),
                PointerButtons::AUXILIARY
            )
        )
        .handled
    );
    assert_eq!(pool.captured_placement(9), Some(&10));
    assert!(
        !pool
            .input(
                &10,
                event(
                    10,
                    PointerPhase::Up,
                    Some(MouseButton::Middle),
                    PointerButtons::NONE
                )
            )
            .handled
    );
    assert!(
        !pool
            .input(
                &10,
                event(
                    9,
                    PointerPhase::Up,
                    Some(MouseButton::Left),
                    PointerButtons::AUXILIARY
                )
            )
            .handled
    );
    assert_eq!(
        calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.starts_with("pointer "))
            .count(),
        1,
        "unrelated releases cannot reach the producer"
    );
    pool.input(&10, PeltInput::Session(SessionInput::Focus(false)));
    assert!(
        !pool.has_pointer_capture(),
        "focus loss must release host custody as well as canceling the producer"
    );
    assert!(
        calls
            .lock()
            .unwrap()
            .iter()
            .any(|call| call.contains("pointer Cancel"))
    );
}

#[test]
fn neutral_drag_and_modified_wheel_preserve_payload_and_scale_once() {
    let calls = Calls::default();
    let mut content = hosted(&calls);
    let mut drag = inker::DragEvent {
        phase: inker::DragPhase::Drop,
        position: inker::PhysicalPosition { x: 10.0, y: 20.0 },
        modifiers: inker::KeyboardModifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        },
        buttons: inker::PointerButtons::PRIMARY,
        data_transfer: inker::DataTransfer {
            items: vec![inker::DataTransferItem::File {
                mime_type: "text/plain".into(),
                path: "/tmp/two words.txt".into(),
                display_name: Some("two words.txt".into()),
            }],
            allowed_operations: inker::DragOperationSet::COPY,
        },
    };
    assert!(
        content
            .input_event(pelt_core::PeltInput::Drag(drag.clone()), 2.0)
            .handled
    );
    drag.position.x *= 2.0;
    drag.position.y *= 2.0;
    let wheel = inker::SurfaceWheelEvent {
        position: inker::PhysicalPosition { x: 20.0, y: 40.0 },
        delta: (6.0, -8.0),
        modifiers: inker::KeyboardModifiers {
            ctrl: true,
            ..Default::default()
        },
    };
    assert!(
        content
            .input_event(
                pelt_core::PeltInput::Wheel {
                    position: inker::PhysicalPosition { x: 10.0, y: 20.0 },
                    delta: (3.0, -4.0),
                    modifiers: inker::SessionModifiers {
                        control: true,
                        ..Default::default()
                    },
                },
                2.0
            )
            .handled
    );
    assert_eq!(
        calls.lock().unwrap().as_slice(),
        [format!("drag {drag:?}"), format!("wheel {wheel:?}")]
    );
}

#[test]
fn placement_removal_cancels_the_original_pen_pointer_without_losing_sensors() {
    use inker::{MouseButton, PhysicalPosition, PointerButtons, PointerPhase, PointerType};
    use pelt_core::{PeltControllerPool, PeltInput, PeltPlacement, WorkspaceRect, mouse_pointer};
    let pointers = Arc::new(Mutex::new(Vec::new()));
    let content = PeltContent::<String>::from_surface(
        Box::new(Page {
            calls: Calls::default(),
            pointers: pointers.clone(),
            acquisitions: Default::default(),
            fail_offset: None,
            title: None,
        }),
        (100, 100),
        route(),
    );
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, content);
    pool.set_placements([(
        10,
        PeltPlacement {
            content: 1,
            window: 2,
            rect: WorkspaceRect::new(20.0, 30.0, 100.0, 100.0),
            visible: true,
            focused: true,
            scale_factor: 2.0,
        },
    )]);
    let mut pen = mouse_pointer(
        81,
        PointerPhase::Down,
        PhysicalPosition { x: 30.0, y: 50.0 },
        Some(MouseButton::Left),
        PointerButtons::PRIMARY,
        Default::default(),
    );
    pen.pointer_type = PointerType::Pen;
    pen.pressure = Some(0.3);
    pen.tilt_x = Some(-25.0);
    pen.width = 3.0;
    pen.height = 4.0;
    assert!(pool.input_at(2, PeltInput::Pointer(pen.clone())).handled);
    assert!(
        !pool.input_at(3, PeltInput::Pointer(pen.clone())).handled,
        "another window cannot steal capture"
    );
    pool.set_placements([]);
    assert!(!pool.has_pointer_capture());
    let events = pointers.lock().unwrap();
    assert_eq!(events.len(), 2);
    pen.position = PhysicalPosition { x: 20.0, y: 40.0 };
    pen.width *= 2.0;
    pen.height *= 2.0;
    assert_eq!(events[0], pen);
    pen.phase = PointerPhase::Cancel;
    pen.button = None;
    pen.buttons = PointerButtons::NONE;
    assert_eq!(
        events[1], pen,
        "cancel retains the originating identity and sensors"
    );
}

#[test]
fn preparing_surface_input_resizes_only_the_target_without_resource_polling() {
    use pelt_core::{PeltControllerPool, PeltPlacement, WorkspaceRect};
    use std::sync::atomic::{AtomicUsize, Ordering};
    let calls = Calls::default();
    let acquisitions = Arc::new(AtomicUsize::new(0));
    let content = PeltContent::<String>::from_surface(
        Box::new(Page {
            calls: calls.clone(),
            pointers: Default::default(),
            acquisitions: acquisitions.clone(),
            fail_offset: None,
            title: None,
        }),
        (200, 100),
        route(),
    );
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, content);
    let placement = |window, width| PeltPlacement {
        content: 1,
        window,
        rect: WorkspaceRect::new(0.0, 0.0, width, 100.0),
        visible: true,
        focused: false,
        scale_factor: 1.0,
    };
    pool.set_placements([(10, placement(0, 200.0)), (11, placement(1, 400.0))]);
    pool.frame(&10);
    pool.frame(&11);
    assert_eq!(acquisitions.load(Ordering::Relaxed), 2);
    pool.prepare(&10).unwrap();
    assert_eq!(calls.lock().unwrap().last().unwrap(), "resize 200x100");
    assert_eq!(
        acquisitions.load(Ordering::Relaxed),
        2,
        "input geometry is not resource polling"
    );
}

#[test]
fn window_focus_loss_cancels_each_pointer_capture_in_that_window() {
    use inker::{MouseButton, PhysicalPosition, PointerButtons, PointerPhase, SessionInput};
    use pelt_core::{PeltControllerPool, PeltInput, PeltPlacement, WorkspaceRect, mouse_pointer};
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, hosted(&Calls::default()));
    pool.insert(2, hosted(&Calls::default()));
    pool.insert(3, hosted(&Calls::default()));
    let placement = |content, focused| PeltPlacement {
        content,
        window: 7,
        rect: WorkspaceRect::new(0.0, 0.0, 100.0, 100.0),
        visible: true,
        focused,
        scale_factor: 1.0,
    };
    let mut other_window = placement(3, true);
    other_window.window = 8;
    pool.set_placements([
        (10, placement(1, true)),
        (11, placement(2, false)),
        (12, other_window),
    ]);
    for (id, placement) in [(81, 10), (82, 11), (83, 12)] {
        let pointer = mouse_pointer(
            id,
            PointerPhase::Down,
            PhysicalPosition { x: 10.0, y: 10.0 },
            Some(MouseButton::Left),
            PointerButtons::PRIMARY,
            Default::default(),
        );
        assert!(pool.input(&placement, PeltInput::Pointer(pointer)).handled);
    }
    pool.input_at(7, SessionInput::Focus(false).into());
    assert!(
        pool.captured_placement(81).is_none() && pool.captured_placement(82).is_none(),
        "host focus loss is not limited to the focused placement"
    );
    assert_eq!(
        pool.captured_placement(83),
        Some(&12),
        "another window keeps its capture"
    );
}

#[test]
fn full_keyboard_payload_reaches_the_focused_surface_through_the_pool() {
    use pelt_core::{PeltControllerPool, PeltInput, PeltPlacement, WorkspaceRect};
    let calls = Calls::default();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, hosted(&calls));
    pool.set_placements([(
        10,
        PeltPlacement {
            content: 1,
            window: 2,
            rect: WorkspaceRect::new(30.0, 40.0, 200.0, 100.0),
            visible: true,
            focused: true,
            scale_factor: 1.5,
        },
    )]);
    let mut event = KeyboardEvent {
        key_code: 0x7B,
        scan_code: 0x1234,
        modifiers: inker::KeyboardModifiers {
            ctrl: true,
            shift: true,
            alt: true,
            meta: true,
        },
        pressed: true,
        text: Some("é JSON payload".into()),
    };
    let down = format!("keyboard {event:?}");
    assert!(pool.input_at(2, PeltInput::Keyboard(event.clone())).handled);
    event.pressed = false;
    event.text = None;
    let up = format!("keyboard {event:?}");
    assert!(pool.input_at(2, PeltInput::Keyboard(event)).handled);
    assert_eq!(
        calls
            .lock()
            .unwrap()
            .iter()
            .filter(|call| call.starts_with("keyboard "))
            .cloned()
            .collect::<Vec<_>>(),
        vec![down, up]
    );
}

#[test]
fn failed_other_placement_frame_cannot_leave_old_input_geometry_admitted() {
    use pelt_core::{PeltControllerPool, PeltLayer, PeltPlacement, WorkspaceRect};
    let calls = Calls::default();
    let content = PeltContent::<String>::from_surface(
        Box::new(Page {
            calls: calls.clone(),
            pointers: Default::default(),
            acquisitions: Default::default(),
            title: None,
            fail_offset: Some((100, 0)),
        }),
        (200, 100),
        route(),
    );
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, content);
    let placement = |x, width| PeltPlacement {
        content: 1,
        window: 0,
        rect: WorkspaceRect::new(x, 0.0, width, 100.0),
        visible: true,
        focused: false,
        scale_factor: 1.0,
    };
    pool.set_placements([(10, placement(0.0, 200.0)), (11, placement(100.0, 400.0))]);
    pool.frame(&10);
    assert!(matches!(pool.frame(&11), Some(PeltLayer::Surface(Err(_)))));
    assert_eq!(calls.lock().unwrap().last().unwrap(), "resize 400x100");
    pool.prepare(&10).unwrap();
    assert_eq!(
        calls.lock().unwrap().last().unwrap(),
        "resize 200x100",
        "failed B offset follows a real resize, so old prepared A must be restored"
    );
}

#[test]
fn positive_subunit_scale_uses_the_same_geometry_for_layout_and_pointer() {
    use pelt_core::{PeltControllerPool, PeltInput, PeltPlacement, WorkspaceRect, mouse_pointer};
    let calls = Calls::default();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, hosted(&calls));
    pool.set_placements([(
        10,
        PeltPlacement {
            content: 1,
            window: 0,
            rect: WorkspaceRect::new(20.0, 30.0, 200.0, 100.0),
            visible: true,
            focused: false,
            scale_factor: 0.75,
        },
    )]);
    pool.frame(&10);
    assert!(
        pool.input(
            &10,
            PeltInput::Pointer(mouse_pointer(
                5,
                inker::PointerPhase::Down,
                inker::PhysicalPosition { x: 10.0, y: 20.0 },
                Some(inker::MouseButton::Left),
                inker::PointerButtons::PRIMARY,
                Default::default()
            ))
        )
        .handled
    );
    let calls = calls.lock().unwrap();
    assert!(
        calls.contains(&"resize 150x75".into()),
        "admitted scale applies to producer extent: {calls:?}"
    );
    assert!(calls.contains(&"pointer Down Some(Left) ctrl=false at=7.5,15".into()));
}

#[test]
fn surface_ime_commit_is_delivered_and_unsupported_preedit_is_explicit() {
    let calls = Calls::default();
    let mut content = hosted(&calls);
    let commit = content.input(
        inker::SessionInput::Ime(inker::SessionIme::Commit("é compose".into())),
        1.0,
    );
    assert!(commit.handled);
    assert!(
        calls
            .lock()
            .unwrap()
            .iter()
            .any(|call| call.contains("é compose"))
    );
    let preedit = content.input(
        inker::SessionInput::Ime(inker::SessionIme::Preedit {
            text: "é".into(),
            selection: Some((0, 2)),
        }),
        1.0,
    );
    assert!(!preedit.handled);
    assert!(
        preedit.error.is_some(),
        "the surface contract cannot silently drop composition state"
    );
}
