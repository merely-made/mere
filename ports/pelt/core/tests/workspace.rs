// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use inker::{
    ContentReport, DocumentSession, SessionButtonState, SessionClick, SessionEngine, SessionError,
    SessionInput, SessionModifiers, SessionNavigationCommand, SessionPointerButton,
    SessionRegistry, SessionScrollKey, SessionSpawnRequest, SurfaceEngineRegistry,
};
use pelt_core::{PeltClock, PeltController, PeltControllerConfig, PeltWorkspace, WorkspaceRect};
use workbench::{
    ContentSource, DocumentRef, DropTarget, Edge, SplitAxis, Tile, TileBranch, TileEvent, TileId,
    TilePath, TileTree,
};

#[derive(Default)]
struct Probe {
    spawns: Vec<String>,
    pumps: HashMap<String, usize>,
    hidden: HashMap<String, Vec<bool>>,
}

struct FakeEngine(Arc<Mutex<Probe>>);

impl SessionEngine<String> for FakeEngine {
    fn engine_id(&self) -> &str {
        "fake"
    }

    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        self.0.lock().unwrap().spawns.push(request.address.clone());
        Ok(Box::new(FakeSession {
            address: request.address.clone(),
            scroll: 0.0,
            probe: self.0.clone(),
        }))
    }
}

struct FakeSession {
    address: String,
    scroll: f32,
    probe: Arc<Mutex<Probe>>,
}

impl DocumentSession<String> for FakeSession {
    fn pump(&mut self, _now_ms: f64) {
        *self
            .probe
            .lock()
            .unwrap()
            .pumps
            .entry(self.address.clone())
            .or_default() += 1;
    }

    fn frame(&mut self, width: u32, height: u32) -> String {
        format!("{}@{width}x{height}+{}", self.address, self.scroll)
    }

    fn scroll_by(&mut self, _dx: f32, dy: f32) -> bool {
        self.scroll += dy;
        true
    }

    fn scroll_for_key(&mut self, _key: SessionScrollKey) -> bool {
        self.scroll += 40.0;
        true
    }

    fn click_at(&mut self, x: f32, _y: f32) -> SessionClick {
        if x < 10.0 {
            SessionClick::Navigate("next.html".to_owned())
        } else {
            SessionClick::Handled
        }
    }

    fn links(&self) -> Vec<inker::SessionLink> {
        Vec::new()
    }

    fn set_hidden(&mut self, hidden: bool) {
        self.probe
            .lock()
            .unwrap()
            .hidden
            .entry(self.address.clone())
            .or_default()
            .push(hidden);
    }

    fn inspect(&self) -> Option<ContentReport> {
        Some(ContentReport {
            title: Some(format!("title:{}", self.address)),
            ..Default::default()
        })
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
        1.0
    }
}

fn tile(id: u64, address: &str) -> Tile {
    Tile {
        id: TileId(id),
        title: address.to_owned(),
        content: ContentSource::Document(DocumentRef(address.to_owned())),
        accent: None,
    }
}

fn controller(tile: &Tile, probe: Arc<Mutex<Probe>>) -> Result<PeltController<String>, String> {
    let ContentSource::Document(DocumentRef(address)) = &tile.content else {
        return Err("not a document".to_owned());
    };
    let mut sessions = SessionRegistry::new();
    sessions.register(Box::new(FakeEngine(probe)));
    PeltController::new(
        sessions,
        SurfaceEngineRegistry::new(),
        PeltControllerConfig::new("fake", address, (800, 600)),
        TestClock,
    )
}

fn press(x: f32, y: f32) -> SessionInput {
    SessionInput::PointerButton {
        x,
        y,
        button: SessionPointerButton::Primary,
        state: SessionButtonState::Pressed,
        modifiers: SessionModifiers::default(),
    }
}

#[test]
fn recursive_workspace_retains_each_tile_across_activation_drag_and_resize() {
    let tree = TileTree::split(
        SplitAxis::Row,
        vec![
            TileBranch::new(
                0.5,
                TileTree::stack(vec![tile(1, "a/index.html"), tile(2, "b/index.html")], 0),
            ),
            TileBranch::new(
                0.5,
                TileTree::split(
                    SplitAxis::Column,
                    vec![
                        TileBranch::new(0.5, TileTree::single(tile(3, "c/index.html"))),
                        TileBranch::new(0.5, TileTree::single(tile(4, "d/index.html"))),
                    ],
                ),
            ),
        ],
    );
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace =
        PeltWorkspace::try_new(tree, |tile| controller(tile, probe.clone())).unwrap();

    assert_eq!(workspace.focused_tile(), Some(TileId(1)));
    assert_eq!(probe.lock().unwrap().spawns.len(), 4);
    assert_eq!(
        probe.lock().unwrap().hidden["b/index.html"].last(),
        Some(&true),
        "the inactive tab is hidden without being discarded"
    );

    workspace.set_content_rects([
        (TileId(1), WorkspaceRect::new(0.0, 44.0, 390.0, 556.0)),
        (TileId(3), WorkspaceRect::new(400.0, 44.0, 400.0, 251.0)),
        (TileId(4), WorkspaceRect::new(400.0, 349.0, 400.0, 251.0)),
    ]);
    assert!(workspace.scroll_at(410.0, 360.0, 0.0, 75.0));
    let first = workspace.frame();
    assert_eq!(first.tiles.len(), 3);
    assert_eq!(first.tiles[2].frame, "d/index.html@400x251+75");

    // The pointer is translated through tile 3's content-hole origin, so an
    // x=405 workspace press reaches the session at local x=5 and follows its
    // relative link. Tile 4's navigation remains independent.
    assert!(workspace.input(press(405.0, 50.0)).navigated);
    assert_eq!(
        workspace.controller(TileId(3)).unwrap().address(),
        "c/next.html"
    );
    assert_eq!(
        workspace.controller(TileId(4)).unwrap().address(),
        "d/index.html"
    );
    assert!(
        workspace
            .command_for(
                TileId(4),
                SessionNavigationCommand::Address("other.html".to_owned()),
            )
            .navigated
    );
    assert!(workspace.controller(TileId(3)).unwrap().can_go_back());
    assert!(workspace.controller(TileId(4)).unwrap().can_go_back());

    assert!(workspace.apply(&TileEvent::Activated(TileId(2))));
    assert_eq!(workspace.focused_tile(), Some(TileId(2)));
    assert_eq!(probe.lock().unwrap().spawns.len(), 6);
    assert_eq!(
        probe.lock().unwrap().hidden["a/index.html"].last(),
        Some(&true)
    );
    workspace.set_content_rects([
        (TileId(2), WorkspaceRect::new(0.0, 44.0, 390.0, 556.0)),
        (TileId(3), WorkspaceRect::new(400.0, 44.0, 400.0, 251.0)),
        (TileId(4), WorkspaceRect::new(400.0, 349.0, 400.0, 251.0)),
    ]);
    assert!(workspace.scroll_at(410.0, 360.0, 0.0, 75.0));

    // Divider resize and tab drag only change arrangement. All four retained
    // controllers and the hidden tile's history survive.
    assert!(workspace.apply(&TileEvent::DividerMoved {
        split: TilePath(Vec::new()),
        fractions: vec![0.65, 0.35],
    }));
    assert!(workspace.apply(&TileEvent::Dragged {
        tile: TileId(2),
        to: DropTarget::Edge {
            tile: TileId(4),
            edge: Edge::Left,
        },
    }));
    assert_eq!(probe.lock().unwrap().spawns.len(), 6);
    assert!(workspace.controller(TileId(3)).unwrap().can_go_back());
    assert_eq!(
        workspace.controller(TileId(4)).unwrap().address(),
        "d/other.html"
    );

    // Reactivating tile 4 proves its live session retained the pre-tab scroll.
    assert!(workspace.apply(&TileEvent::Activated(TileId(4))));
    workspace.set_content_rects([
        (TileId(1), WorkspaceRect::new(0.0, 44.0, 300.0, 556.0)),
        (TileId(3), WorkspaceRect::new(310.0, 44.0, 240.0, 251.0)),
        (TileId(4), WorkspaceRect::new(560.0, 349.0, 240.0, 251.0)),
    ]);
    let frames = workspace.frame();
    let tile4 = frames
        .tiles
        .iter()
        .find(|frame| frame.tile == TileId(4))
        .expect("tile 4 is visible");
    assert_eq!(tile4.frame, "d/other.html@240x251+75");

    assert!(workspace.apply(&TileEvent::Closed(TileId(3))));
    assert!(workspace.controller(TileId(3)).is_none());
    assert!(workspace.controller(TileId(1)).is_some());
    assert!(workspace.controller(TileId(2)).is_some());
    assert!(workspace.controller(TileId(4)).is_some());
}

#[test]
fn outside_tearout_waits_for_host_acceptance_and_retains_pelt_custody() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |tile| {
        controller(tile, probe.clone())
    })
    .unwrap();
    let before = workspace.tree().clone();
    let focused_before = workspace.focused_tile();

    let tearout = workspace.apply_outcome(&TileEvent::Dragged {
        tile: TileId(1),
        to: DropTarget::Outside,
    });
    assert_eq!(
        tearout.effect(),
        Some(workbench::WorkbenchEffect::TearOut { tile: TileId(1) })
    );
    assert!(!tearout.changed());
    assert_eq!(workspace.tree(), &before);
    assert!(workspace.controller(TileId(1)).is_some());
    assert_eq!(workspace.focused_tile(), focused_before);
    assert_eq!(probe.lock().unwrap().spawns.len(), 1);

    // A host may render a source-owned preflight frame, then discover that
    // surface configuration or presentation failed. Until it calls the
    // acceptance transfer, source tree membership, controller custody, and
    // model focus remain unchanged. Framing may transiently update viewport
    // geometry, which the native host restores from its source content hole.
    let preflight = workspace
        .frame_tile(TileId(1), WorkspaceRect::new(0.0, 0.0, 960.0, 640.0))
        .expect("live tile produces a destination preflight frame");
    assert_eq!(preflight.tiles.len(), 1);
    let destination_presented: Result<(), &str> = Err("destination present failed");
    assert!(destination_presented.is_err());
    assert_eq!(workspace.tree(), &before);
    assert!(workspace.controller(TileId(1)).is_some());
    assert_eq!(workspace.focused_tile(), focused_before);

    // A cancelled native-window request also never calls the acceptance
    // transfer, so the source tree and its live controller remain intact.
    let native_window: Result<(), &str> = Err("window creation cancelled");
    if native_window.is_ok() {
        let _ = workspace.accept_tearout(TileId(1));
    }
    assert_eq!(workspace.tree(), &before);
    assert!(workspace.controller(TileId(1)).is_some());
    assert_eq!(workspace.focused_tile(), focused_before);

    // Once the host has accepted the request, the controller moves rather
    // than being recreated. The destination keeps the original stable tile
    // identity and becomes its focused workspace.
    let detached = workspace
        .accept_tearout(TileId(1))
        .expect("known live tile transfers after host acceptance");
    assert!(workspace.tree().find(TileId(1)).is_none());
    assert!(workspace.controller(TileId(1)).is_none());
    assert_eq!(
        detached.tree().find(TileId(1)).map(|tile| tile.id),
        Some(TileId(1))
    );
    assert_eq!(detached.focused_tile(), Some(TileId(1)));
    assert!(detached.controller(TileId(1)).is_some());
    assert_eq!(probe.lock().unwrap().spawns.len(), 1);

    let unknown = workspace.apply_outcome(&TileEvent::Dragged {
        tile: TileId(99),
        to: DropTarget::Outside,
    });
    assert_eq!(unknown.effect(), None);
    assert!(!unknown.changed());
    assert!(workspace.tree().find(TileId(1)).is_none());
    assert!(workspace.controller(TileId(1)).is_none());
}

#[test]
fn hidden_sessions_are_pumped_to_complete_async_work() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let tree = TileTree::stack(vec![tile(1, "a/index.html"), tile(2, "b/index.html")], 0);
    let mut workspace = PeltWorkspace::try_new(tree, |t| controller(t, probe.clone())).unwrap();
    workspace.pump();
    let probe = probe.lock().unwrap();
    assert_eq!(probe.pumps.get("a/index.html"), Some(&1));
    assert_eq!(
        probe.pumps.get("b/index.html"),
        Some(&1),
        "hidden work must reach the engine's settled decision"
    );
}

// These probes enter the public routed workspace API. Receiver serials and
// destruction order distinguish a retained placement from its live receiver.
#[derive(Clone, Debug, PartialEq)]
enum CaptureCall {
    Session(usize, SessionInput),
    Pointer(usize, inker::PointerEvent),
    Drop(usize),
}

#[derive(Default)]
struct CaptureProbe {
    next: usize,
    receivers: Vec<(usize, String, String)>,
    calls: Vec<CaptureCall>,
    failed: std::collections::HashSet<String>,
}

struct CaptureDocumentEngine {
    id: &'static str,
    probe: Arc<Mutex<CaptureProbe>>,
}

impl SessionEngine<String> for CaptureDocumentEngine {
    fn engine_id(&self) -> &str {
        self.id
    }

    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        let mut probe = self.probe.lock().unwrap();
        if probe.failed.contains(self.id) {
            return Err(SessionError::SpawnFailed(
                "capture replacement rejected".to_owned(),
            ));
        }
        probe.next += 1;
        let serial = probe.next;
        probe
            .receivers
            .push((serial, self.id.to_owned(), request.address.clone()));
        Ok(Box::new(CaptureDocument {
            serial,
            probe: self.probe.clone(),
        }))
    }
}

struct CaptureDocument {
    serial: usize,
    probe: Arc<Mutex<CaptureProbe>>,
}

impl Drop for CaptureDocument {
    fn drop(&mut self) {
        self.probe
            .lock()
            .unwrap()
            .calls
            .push(CaptureCall::Drop(self.serial));
    }
}

impl DocumentSession<String> for CaptureDocument {
    fn frame(&mut self, width: u32, height: u32) -> String {
        format!("{}@{width}x{height}", self.serial)
    }
    fn scroll_by(&mut self, _: f32, _: f32) -> bool {
        false
    }
    fn scroll_for_key(&mut self, _: SessionScrollKey) -> bool {
        false
    }
    fn click_at(&mut self, _: f32, _: f32) -> SessionClick {
        SessionClick::Handled
    }
    fn links(&self) -> Vec<inker::SessionLink> {
        Vec::new()
    }
    fn input(&mut self, input: SessionInput) -> inker::SessionInputResult {
        let capture = match input {
            SessionInput::PointerButton {
                state: SessionButtonState::Pressed,
                ..
            } => Some(true),
            SessionInput::Cancel => Some(false),
            _ => None,
        };
        self.probe
            .lock()
            .unwrap()
            .calls
            .push(CaptureCall::Session(self.serial, input));
        inker::SessionInputResult {
            effect: inker::SessionEffect::Handled,
            cursor: None,
            capture,
            editable: false,
        }
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}

struct CaptureSurfaceEngine(Arc<Mutex<CaptureProbe>>);

impl inker::SurfaceEngine for CaptureSurfaceEngine {
    fn engine_id(&self) -> &str {
        "capture.surface"
    }
    fn spawn(
        &self,
        request: &inker::SurfaceSpawnRequest,
    ) -> Result<Box<dyn inker::SurfaceProducer>, inker::SurfaceError> {
        let mut probe = self.0.lock().unwrap();
        probe.next += 1;
        let serial = probe.next;
        probe
            .receivers
            .push((serial, "capture.surface".to_owned(), request.url.clone()));
        Ok(Box::new(CaptureSurface {
            serial,
            probe: self.0.clone(),
        }))
    }
}

struct CaptureSurface {
    serial: usize,
    probe: Arc<Mutex<CaptureProbe>>,
}

impl inker::SurfaceProducer for CaptureSurface {
    fn resize(&mut self, _: u32, _: u32) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn set_offset(&mut self, _: i32, _: i32) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn acquire_frame(&mut self) -> Result<Option<inker::SurfaceFrame>, inker::SurfaceError> {
        Ok(None)
    }
    fn send_mouse_input(&mut self, _: inker::MouseEvent) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn send_pointer_input(
        &mut self,
        event: inker::PointerEvent,
    ) -> Result<(), inker::SurfaceError> {
        self.probe
            .lock()
            .unwrap()
            .calls
            .push(CaptureCall::Pointer(self.serial, event));
        Ok(())
    }
    fn send_keyboard_input(&mut self, _: inker::KeyboardEvent) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn move_focus(&mut self, _: inker::FocusReason) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn poll_cursor_shape(&mut self) -> Option<inker::CursorShape> {
        None
    }
    fn apply_settings(&mut self, _: &inker::SurfaceSettings) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
}

fn capture_workspace(
    surface: bool,
    two_tiles: bool,
) -> (PeltWorkspace<String>, Arc<Mutex<CaptureProbe>>) {
    use inker::routing::{EngineRoutePolicy, EngineRouteRule, SurfaceContractMode};
    let probe = Arc::new(Mutex::new(CaptureProbe::default()));
    let mut sessions = SessionRegistry::new();
    for id in ["capture.a", "capture.b"] {
        sessions.register(Box::new(CaptureDocumentEngine {
            id,
            probe: probe.clone(),
        }));
    }
    let mut surfaces = SurfaceEngineRegistry::new();
    surfaces.register(Box::new(CaptureSurfaceEngine(probe.clone())));
    let registries = pelt_core::PeltRegistries::new(
        sessions,
        surfaces,
        EngineRoutePolicy {
            rules: Vec::new(),
            fallback: EngineRouteRule::new(
                std::iter::empty::<&str>(),
                "capture.a",
                SurfaceContractMode::CompositedTexture,
            ),
            per_host_overrides: HashMap::new(),
        },
        "capture-workspace-test",
        "capture.a",
        inker::EngineProfileBinding {
            user_data_dir: "capture-workspace-test".to_owned(),
        },
    );
    let tree = if two_tiles {
        TileTree::split(
            SplitAxis::Row,
            vec![
                TileBranch::new(0.5, TileTree::single(tile(1, "a/index.html"))),
                TileBranch::new(0.5, TileTree::single(tile(2, "b/index.html"))),
            ],
        )
    } else {
        TileTree::single(tile(1, "a/index.html"))
    };
    let mut workspace = PeltWorkspace::try_routed(
        tree,
        registries,
        |tile| {
            let ContentSource::Document(DocumentRef(address)) = &tile.content else {
                unreachable!()
            };
            Ok(
                pelt_core::PeltTileRequest::new(address, (100, 80)).with_engine_override(
                    if surface {
                        "capture.surface"
                    } else {
                        "capture.a"
                    },
                ),
            )
        },
        || Box::new(TestClock),
    )
    .unwrap();
    workspace.set_content_rects([
        (TileId(1), WorkspaceRect::new(20.0, 30.0, 100.0, 80.0)),
        (TileId(2), WorkspaceRect::new(200.0, 30.0, 100.0, 80.0)),
    ]);
    probe.lock().unwrap().calls.clear();
    (workspace, probe)
}

fn capture_session_button(x: f32, state: SessionButtonState) -> SessionInput {
    SessionInput::PointerButton {
        x,
        y: 45.0,
        button: SessionPointerButton::Primary,
        state,
        modifiers: SessionModifiers::default(),
    }
}

fn capture_calls(probe: &Arc<Mutex<CaptureProbe>>) -> Vec<CaptureCall> {
    probe.lock().unwrap().calls.clone()
}

fn assert_retired_receiver(
    workspace: &mut PeltWorkspace<String>,
    probe: &Arc<Mutex<CaptureProbe>>,
    old: usize,
) {
    let calls = capture_calls(probe);
    let marker = calls.len();
    workspace.input(SessionInput::PointerMoved {
        x: 250.0,
        y: 45.0,
        modifiers: SessionModifiers::default(),
    });
    workspace.input(capture_session_button(250.0, SessionButtonState::Released));
    let cancel = calls
        .iter()
        .position(|c| *c == CaptureCall::Session(old, SessionInput::Cancel));
    let drop = calls.iter().position(|c| *c == CaptureCall::Drop(old));
    assert!(
        cancel.is_some(),
        "accepted receiver replacement must cancel the old gesture"
    );
    assert_eq!(
        calls
            .iter()
            .filter(|c| **c == CaptureCall::Session(old, SessionInput::Cancel))
            .count(),
        1
    );
    assert!(
        cancel.unwrap() < drop.expect("old receiver dropped"),
        "Cancel must precede old receiver destruction"
    );
    assert!(!workspace.has_active_pointer_capture());
    assert_eq!(
        capture_calls(probe).len(),
        marker,
        "old outside Move/Up must not reach the replacement"
    );
    let new = probe.lock().unwrap().receivers.last().unwrap().0;
    assert!(
        workspace
            .input(capture_session_button(35.0, SessionButtonState::Pressed))
            .handled
    );
    assert!(
        workspace
            .input(capture_session_button(35.0, SessionButtonState::Released))
            .handled
    );
    assert_eq!(
        capture_calls(probe)[marker..],
        [
            CaptureCall::Session(
                new,
                capture_session_button(15.0, SessionButtonState::Pressed)
            ),
            CaptureCall::Session(
                new,
                capture_session_button(15.0, SessionButtonState::Released)
            ),
        ]
        .map(|call| match call {
            CaptureCall::Session(
                serial,
                SessionInput::PointerButton {
                    x,
                    button,
                    state,
                    modifiers,
                    ..
                },
            ) => CaptureCall::Session(
                serial,
                SessionInput::PointerButton {
                    x,
                    y: 15.0,
                    button,
                    state,
                    modifiers
                }
            ),
            _ => unreachable!(),
        })
    );
    assert!(!workspace.has_active_pointer_capture());
}

#[test]
fn review_workspace_capture_provider_replacement_cancels_before_drop() {
    let (mut workspace, probe) = capture_workspace(false, false);
    assert!(
        workspace
            .input(capture_session_button(35.0, SessionButtonState::Pressed))
            .handled
    );
    assert_eq!(workspace.pool().captured_placement(0), Some(&TileId(1)));
    let before = workspace.document_session_identity(TileId(1)).unwrap();
    assert!(
        workspace
            .set_route_override(TileId(1), Some("capture.b".to_owned()))
            .unwrap()
    );
    assert_ne!(
        workspace.document_session_identity(TileId(1)).unwrap(),
        before
    );
    assert_retired_receiver(&mut workspace, &probe, 1);
}

fn capture_accepted_command(command: SessionNavigationCommand) {
    let (mut workspace, probe) = capture_workspace(false, false);
    if command == SessionNavigationCommand::Back {
        assert!(
            workspace
                .command_for(
                    TileId(1),
                    SessionNavigationCommand::Address("next.html".to_owned())
                )
                .navigated
        );
    }
    let old = probe.lock().unwrap().receivers.last().unwrap().0;
    assert!(
        workspace
            .input(capture_session_button(35.0, SessionButtonState::Pressed))
            .handled
    );
    let before = workspace.document_session_identity(TileId(1)).unwrap();
    assert!(workspace.command_for(TileId(1), command).navigated);
    assert_ne!(
        workspace.document_session_identity(TileId(1)).unwrap(),
        before
    );
    assert_retired_receiver(&mut workspace, &probe, old);
}

#[test]
fn review_workspace_capture_reload_cancels_old_generation() {
    capture_accepted_command(SessionNavigationCommand::Reload);
}

#[test]
fn review_workspace_capture_address_cancels_old_generation() {
    capture_accepted_command(SessionNavigationCommand::Address("next.html".to_owned()));
}

#[test]
fn review_workspace_capture_history_cancels_old_generation() {
    capture_accepted_command(SessionNavigationCommand::Back);
}

fn assert_capture_survives(
    workspace: &mut PeltWorkspace<String>,
    probe: &Arc<Mutex<CaptureProbe>>,
    before: pelt_core::PeltSessionIdentity,
) {
    assert_eq!(workspace.document_session_identity(TileId(1)), Some(before));
    assert_eq!(workspace.pool().captured_placement(0), Some(&TileId(1)));
    assert!(
        workspace
            .input(SessionInput::PointerMoved {
                x: 250.0,
                y: 45.0,
                modifiers: SessionModifiers::default()
            })
            .handled
    );
    assert!(
        workspace
            .input(capture_session_button(250.0, SessionButtonState::Released))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
    assert_eq!(
        capture_calls(probe),
        [
            CaptureCall::Session(
                1,
                SessionInput::PointerButton {
                    x: 15.0,
                    y: 15.0,
                    button: SessionPointerButton::Primary,
                    state: SessionButtonState::Pressed,
                    modifiers: SessionModifiers::default()
                }
            ),
            CaptureCall::Session(
                1,
                SessionInput::PointerMoved {
                    x: 230.0,
                    y: 15.0,
                    modifiers: SessionModifiers::default()
                }
            ),
            CaptureCall::Session(
                1,
                SessionInput::PointerButton {
                    x: 230.0,
                    y: 15.0,
                    button: SessionPointerButton::Primary,
                    state: SessionButtonState::Released,
                    modifiers: SessionModifiers::default()
                }
            ),
        ]
    );
}

#[test]
fn review_workspace_capture_noop_provider_preserves_receiver() {
    let (mut workspace, probe) = capture_workspace(false, false);
    workspace.input(capture_session_button(35.0, SessionButtonState::Pressed));
    let before = workspace.document_session_identity(TileId(1)).unwrap();
    assert!(
        !workspace
            .set_route_override(TileId(1), Some("capture.a".to_owned()))
            .unwrap()
    );
    assert_capture_survives(&mut workspace, &probe, before);
}

#[test]
fn review_workspace_capture_failed_provider_preserves_receiver() {
    let (mut workspace, probe) = capture_workspace(false, false);
    workspace.input(capture_session_button(35.0, SessionButtonState::Pressed));
    let before = workspace.document_session_identity(TileId(1)).unwrap();
    probe
        .lock()
        .unwrap()
        .failed
        .extend(["capture.a".to_owned(), "capture.b".to_owned()]);
    assert!(
        workspace
            .set_route_override(TileId(1), Some("capture.b".to_owned()))
            .is_err()
    );
    assert_capture_survives(&mut workspace, &probe, before);
}

#[test]
fn review_workspace_capture_failed_reload_preserves_receiver() {
    let (mut workspace, probe) = capture_workspace(false, false);
    workspace.input(capture_session_button(35.0, SessionButtonState::Pressed));
    let before = workspace.document_session_identity(TileId(1)).unwrap();
    probe.lock().unwrap().failed.insert("capture.a".to_owned());
    let effect = workspace.command_for(TileId(1), SessionNavigationCommand::Reload);
    assert!(!effect.navigated);
    assert!(effect.error.is_some());
    assert_capture_survives(&mut workspace, &probe, before);
}

fn capture_pointer(
    id: i32,
    phase: inker::PointerPhase,
    x: f32,
    button: Option<inker::MouseButton>,
    buttons: u16,
) -> inker::PointerEvent {
    pelt_core::mouse_pointer(
        id,
        phase,
        inker::PhysicalPosition { x, y: 45.0 },
        button,
        inker::PointerButtons(buttons),
        SessionModifiers {
            shift: true,
            ..Default::default()
        },
    )
}

#[test]
fn review_workspace_capture_nonzero_pointer_chord_keeps_focus_receiver_and_masks() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = capture_workspace(true, true);
    let send = |workspace: &mut PeltWorkspace<String>, event| {
        workspace.input_event(pelt_core::PeltInput::Pointer(event))
    };
    assert!(
        send(
            &mut workspace,
            capture_pointer(37, Down, 35.0, Some(Left), 1)
        )
        .handled
    );
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    assert!(send(&mut workspace, capture_pointer(37, Move, 215.0, None, 1)).handled);
    assert!(
        send(
            &mut workspace,
            capture_pointer(37, Down, 215.0, Some(Middle), 5)
        )
        .handled
    );
    assert_eq!(
        workspace.focused_tile(),
        Some(TileId(1)),
        "a second button must not move focus away from the captured receiver"
    );
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    assert!(
        !send(
            &mut workspace,
            capture_pointer(37, Up, 215.0, Some(Right), 5)
        )
        .handled
    );
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    assert!(
        send(
            &mut workspace,
            capture_pointer(37, Up, 215.0, Some(Left), 4)
        )
        .handled
    );
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    assert!(
        send(
            &mut workspace,
            capture_pointer(37, Up, 215.0, Some(Middle), 0)
        )
        .handled
    );
    assert!(!workspace.has_active_pointer_capture());
    assert_eq!(
        capture_calls(&probe),
        [
            capture_pointer(37, Down, 15.0, Some(Left), 1),
            capture_pointer(37, Move, 195.0, None, 1),
            capture_pointer(37, Down, 195.0, Some(Middle), 5),
            capture_pointer(37, Up, 195.0, Some(Left), 4),
            capture_pointer(37, Up, 195.0, Some(Middle), 0),
        ]
        .map(|mut event| {
            event.position.y = 15.0;
            CaptureCall::Pointer(1, event)
        })
    );
    // A real appearance retirement cancels the next captured gesture once;
    // its old Up cannot be borrowed by B, which then accepts a fresh gesture.
    send(
        &mut workspace,
        capture_pointer(37, Down, 35.0, Some(Left), 1),
    );
    assert!(workspace.apply(&TileEvent::Closed(TileId(1))));
    assert!(!workspace.has_active_pointer_capture());
    let calls = capture_calls(&probe);
    assert!(
        matches!(calls.last(), Some(CaptureCall::Pointer(1, event)) if event.phase == Cancel && event.buttons.0 == 0 && event.button.is_none())
    );
    let marker = calls.len();
    assert!(
        !send(
            &mut workspace,
            capture_pointer(37, Up, 215.0, Some(Left), 0)
        )
        .handled
    );
    assert_eq!(capture_calls(&probe).len(), marker);
    assert!(
        send(
            &mut workspace,
            capture_pointer(37, Down, 215.0, Some(Left), 1)
        )
        .handled
    );
    assert_eq!(workspace.focused_tile(), Some(TileId(2)));
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(2)));
    assert!(
        send(
            &mut workspace,
            capture_pointer(37, Up, 215.0, Some(Left), 0)
        )
        .handled
    );
    assert!(!workspace.has_active_pointer_capture());
    assert!(capture_calls(&probe)[marker..].iter().all(|call| matches!(call, CaptureCall::Pointer(2, event) if event.pointer_id == 37 && event.position.x == 15.0 && event.position.y == 15.0)));
}

#[test]
fn review_workspace_capture_overlap_focus_matches_topmost_receiver() {
    let (mut workspace, probe) = capture_workspace(true, true);
    workspace.set_content_rects([
        (TileId(1), WorkspaceRect::new(20.0, 30.0, 100.0, 80.0)),
        (TileId(2), WorkspaceRect::new(25.0, 35.0, 100.0, 80.0)),
    ]);
    assert!(
        workspace
            .input_event(pelt_core::PeltInput::Pointer(capture_pointer(
                49,
                inker::PointerPhase::Down,
                35.0,
                Some(inker::MouseButton::Left),
                1
            )))
            .handled
    );
    let calls = capture_calls(&probe);
    assert!(
        matches!(calls.as_slice(), [CaptureCall::Pointer(2, event)] if event.position.x == 10.0 && event.position.y == 10.0)
    );
    assert_eq!(
        workspace.focused_tile(),
        Some(TileId(2)),
        "focus must name the topmost positional receiver"
    );
    assert_eq!(workspace.pool().captured_placement(49), Some(&TileId(2)));
}
