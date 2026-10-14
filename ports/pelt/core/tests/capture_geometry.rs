// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! A failed geometry preparation cannot preserve a physically released gesture.
use inker::{
    DocumentSession, PointerPhase, SessionButtonState, SessionClick, SessionEngine, SessionError,
    SessionInput, SessionInputResult, SessionModifiers, SessionRegistry,
    SessionScrollKey, SessionSpawnRequest, SurfaceEngineRegistry,
};
use pelt_core::{PeltClock, PeltInput, PeltWorkspace, WorkspaceRect};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use workbench::{ContentSource, DocumentRef, Tile, TileId, TileTree};
#[derive(Clone, Debug, PartialEq)]
enum Call {
    Session(usize, SessionInput),
    Pointer(usize, inker::PointerEvent),
    Drop(usize),
    Focus(usize, bool),
}
#[derive(Default)]
struct Probe {
    next: usize,
    calls: Vec<Call>,
    failed_phase: Option<PointerPhase>,
    failed_document_down: bool,
    failed_geometry: Option<&'static str>,
}
struct DocumentEngine(Arc<Mutex<Probe>>);
struct Document {
    serial: usize,
    probe: Arc<Mutex<Probe>>,
}
impl Drop for Document {
    fn drop(&mut self) {
        self.probe
            .lock()
            .unwrap()
            .calls
            .push(Call::Drop(self.serial));
    }
}
impl SessionEngine<String> for DocumentEngine {
    fn engine_id(&self) -> &str {
        "terminal.document"
    }
    fn spawn(
        &self,
        _: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        let mut probe = self.0.lock().unwrap();
        probe.next += 1;
        Ok(Box::new(Document {
            serial: probe.next,
            probe: self.0.clone(),
        }))
    }
}
impl DocumentSession<String> for Document {
    fn frame(&mut self, _: u32, _: u32) -> String {
        String::new()
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
    fn input(&mut self, input: SessionInput) -> SessionInputResult {
        let down = matches!(
            &input,
            SessionInput::PointerButton {
                state: SessionButtonState::Pressed,
                ..
            }
        );
        let cancel = matches!(&input, SessionInput::Cancel);
        let mut probe = self.probe.lock().unwrap();
        probe.calls.push(Call::Session(self.serial, input));
        let failed = down && probe.failed_document_down;
        SessionInputResult {
            effect: if failed {
                inker::SessionEffect::Ignored
            } else {
                inker::SessionEffect::Handled
            },
            cursor: None,
            editable: false,
            capture: if failed {
                None
            } else if down {
                Some(true)
            } else if cancel {
                Some(false)
            } else {
                None
            },
        }
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
struct SurfaceEngine(Arc<Mutex<Probe>>);
struct Surface {
    serial: usize,
    probe: Arc<Mutex<Probe>>,
}
impl Drop for Surface {
    fn drop(&mut self) {
        self.probe
            .lock()
            .unwrap()
            .calls
            .push(Call::Drop(self.serial));
    }
}
impl inker::SurfaceEngine for SurfaceEngine {
    fn engine_id(&self) -> &str {
        "terminal.surface"
    }
    fn spawn(
        &self,
        _: &inker::SurfaceSpawnRequest,
    ) -> Result<Box<dyn inker::SurfaceProducer>, inker::SurfaceError> {
        let mut probe = self.0.lock().unwrap();
        probe.next += 1;
        Ok(Box::new(Surface {
            serial: probe.next,
            probe: self.0.clone(),
        }))
    }
}
impl inker::SurfaceProducer for Surface {
    fn resize(&mut self, _: u32, _: u32) -> Result<(), inker::SurfaceError> {
        if self.probe.lock().unwrap().failed_geometry == Some("resize") {
            return Err(inker::SurfaceError::Unsupported("resize rejected".into()));
        }
        Ok(())
    }
    fn set_offset(&mut self, _: i32, _: i32) -> Result<(), inker::SurfaceError> {
        if self.probe.lock().unwrap().failed_geometry == Some("offset") {
            return Err(inker::SurfaceError::Unsupported("offset rejected".into()));
        }
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
        let mut probe = self.probe.lock().unwrap();
        let fail = probe.failed_phase == Some(event.phase);
        probe.calls.push(Call::Pointer(self.serial, event));
        if fail {
            Err(inker::SurfaceError::Unsupported(
                "terminal delivery rejected".to_owned(),
            ))
        } else {
            Ok(())
        }
    }
    fn send_keyboard_input(&mut self, _: inker::KeyboardEvent) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn move_focus(&mut self, _: inker::FocusReason) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
    fn set_focus(&mut self, focused: bool) -> Result<(), inker::SurfaceError> {
        self.probe.lock().unwrap().calls.push(Call::Focus(self.serial, focused));
        Ok(())
    }
    fn poll_cursor_shape(&mut self) -> Option<inker::CursorShape> {
        None
    }
    fn apply_settings(&mut self, _: &inker::SurfaceSettings) -> Result<(), inker::SurfaceError> {
        Ok(())
    }
}
struct Clock;
impl PeltClock for Clock {
    fn now_ms(&self) -> f64 {
        1.0
    }
}
fn workspace(surface: bool) -> (PeltWorkspace<String>, Arc<Mutex<Probe>>) {
    use inker::routing::{EngineRoutePolicy, EngineRouteRule, SurfaceContractMode};
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut sessions = SessionRegistry::new();
    sessions.register(Box::new(DocumentEngine(probe.clone())));
    let mut surfaces = SurfaceEngineRegistry::new();
    surfaces.register(Box::new(SurfaceEngine(probe.clone())));
    let registries = pelt_core::PeltRegistries::new(
        sessions,
        surfaces,
        EngineRoutePolicy {
            rules: Vec::new(),
            fallback: EngineRouteRule::new(
                std::iter::empty::<&str>(),
                "terminal.document",
                SurfaceContractMode::CompositedTexture,
            ),
            per_host_overrides: HashMap::new(),
        },
        "terminal-test",
        "terminal.document",
        inker::EngineProfileBinding {
            user_data_dir: "terminal-test".to_owned(),
        },
    );
    let tile = Tile {
        id: TileId(1),
        title: "terminal".to_owned(),
        content: ContentSource::Document(DocumentRef("https://terminal.test/".to_owned())),
        accent: None,
    };
    let mut workspace = PeltWorkspace::try_routed(
        TileTree::single(tile),
        registries,
        |_| {
            Ok(
                pelt_core::PeltTileRequest::new("https://terminal.test/", (100, 80))
                    .with_engine_override(if surface {
                        "terminal.surface"
                    } else {
                        "terminal.document"
                    }),
            )
        },
        || Box::new(Clock),
    )
    .unwrap();
    workspace.set_content_rects([(TileId(1), WorkspaceRect::new(20.0, 30.0, 100.0, 80.0))]);
    if surface {
        workspace.set_surface_scale_factor(2.0);
    }
    probe.lock().unwrap().calls.clear();
    (workspace, probe)
}
fn pointer(
    id: i32,
    phase: PointerPhase,
    button: Option<inker::MouseButton>,
    buttons: u16,
    x: f32,
) -> PeltInput {
    PeltInput::Pointer(pelt_core::mouse_pointer(
        id,
        phase,
        inker::PhysicalPosition { x, y: 45.0 },
        button,
        inker::PointerButtons(buttons),
        SessionModifiers::default(),
    ))
}
fn calls(probe: &Arc<Mutex<Probe>>) -> Vec<Call> {
    probe.lock().unwrap().calls.clone()
}
fn surface_cancels(probe: &Arc<Mutex<Probe>>) -> Vec<inker::PointerEvent> {
    calls(probe)
        .into_iter()
        .filter_map(|call| match call {
            Call::Pointer(1, event) if event.phase == PointerPhase::Cancel => Some(event),
            _ => None,
        })
        .collect()
}
fn replace_surface(workspace: &mut PeltWorkspace<String>) {
    assert!(
        workspace
            .set_route_override(TileId(1), Some("terminal.document".to_owned()))
            .unwrap()
    );
}
fn assert_dropped(probe: &Arc<Mutex<Probe>>) {
    assert_eq!(
        calls(probe)
            .iter()
            .filter(|call| **call == Call::Drop(1))
            .count(),
        1
    );
}

fn change_geometry(workspace: &mut PeltWorkspace<String>) {
    workspace.set_surface_scale_factor(3.0);
    workspace.set_content_rects([(TileId(1), WorkspaceRect::new(40.0, 50.0, 200.0, 100.0))]);
}
fn failed_final_release(geometry: &'static str, eager_focus: bool) {
    use inker::{MouseButton::Left, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    let mut initial = workspace.pool().placement(&TileId(1)).unwrap().clone();
    initial.focused = eager_focus;
    workspace.pool_mut().set_placements([(TileId(1), initial)]);
    // Direct production pool dispatch avoids workspace focus mutation in the
    // lazy case; the eager case exercises set_placements -> sync_visibility.
    assert!(workspace.pool_mut().input_at(0, pointer(37, Down, Some(Left), 1, 35.0)).handled);
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    probe.lock().unwrap().failed_geometry = Some(geometry);
    let mut p = workspace.pool().placement(&TileId(1)).unwrap().clone();
    p.scale_factor = 3.0;
    p.rect = WorkspaceRect::new(40.0, 50.0, 200.0, 100.0);
    workspace.pool_mut().set_placements([(TileId(1), p)]);
    if eager_focus {
        assert_eq!(workspace.pool().captured_placement(37), None,
            "sync_visibility preparation must retire before any physical Up");
        let history = calls(&probe);
        let cancelled = history.iter().position(|call| matches!(call,
            Call::Pointer(1, event) if event.phase == Cancel)).unwrap();
        let blurred = history.iter().rposition(|call| *call == Call::Focus(1, false)).unwrap();
        assert!(cancelled < blurred, "Cancel must precede the failed appearance's focus drop");
        let error = workspace.pool_mut().prepare(&TileId(1)).unwrap_err();
        assert!(error.contains(geometry), "explicit preparation must retain the error");
    } else {
        assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    }
    let effect = workspace.pool_mut().input_at(0, pointer(37, Up, Some(Left), 0, 70.0));
    if !eager_focus {
        assert!(effect.error.as_deref().is_some_and(|error| error.contains(geometry)));
        assert_eq!(effect.pointer_capture, Some(false));
    }
    assert!(!effect.handled, "failed geometry must not manufacture successful Up");
    assert_eq!(workspace.pool().captured_placement(37), None);
    assert!(!workspace.has_active_pointer_capture());
    let cancels = surface_cancels(&probe);
    assert_eq!(cancels.len(), 1);
    assert_eq!(cancels[0].pointer_id, 37);
    assert_eq!(cancels[0].position, inker::PhysicalPosition { x: 30.0, y: 30.0 },
        "Cancel must use the accepted Down's local envelope and old scale2, not failed scale3 geometry");
    assert_eq!(cancels[0].buttons, inker::PointerButtons::NONE);
    assert_eq!(cancels[0].button, None);
    assert!(!calls(&probe).iter().any(|call| matches!(call,
        Call::Pointer(_, event) if event.phase == Up)));
    probe.lock().unwrap().failed_geometry = None;
    let marker = calls(&probe).len();
    assert!(!workspace.pool_mut().input_at(0, pointer(37, Move, None, 0, 500.0)).handled);
    assert!(!workspace.pool_mut().input_at(0, pointer(37, Up, Some(Left), 0, 70.0)).handled);
    assert_eq!(calls(&probe).len(), marker, "recovered geometry cannot revive the released gesture");
    replace_surface(&mut workspace);
    assert_dropped(&probe);
    assert_eq!(surface_cancels(&probe).len(), 1, "replacement must not repeat retirement Cancel");
    let history = calls(&probe);
    let cancelled = history.iter().position(|call| matches!(call,
        Call::Pointer(1, event) if event.phase == Cancel)).unwrap();
    let dropped = history.iter().position(|call| *call == Call::Drop(1)).unwrap();
    assert!(cancelled < dropped);
    // A fresh gesture reaches the new document receiver after replacement.
    workspace.set_content_rects([(TileId(1), WorkspaceRect::new(40.0, 50.0, 200.0, 100.0))]);
    let mut down = pointer(37, Down, Some(Left), 1, 70.0);
    let mut up = pointer(37, Up, Some(Left), 0, 70.0);
    down.translate(0.0, 20.0);
    up.translate(0.0, 20.0);
    assert!(workspace.input_event(down).handled);
    assert!(workspace.input_event(up).handled);
    assert!(!workspace.has_active_pointer_capture());
    assert!(calls(&probe).iter().any(|call| matches!(call, Call::Session(2,
        SessionInput::PointerButton { state: SessionButtonState::Pressed, .. }))));
}
#[test]
fn review_capture_geometry_failed_offset_visibility_cancels_original_receiver() {
    failed_final_release("offset", true);
}
#[test]
fn review_capture_geometry_failed_resize_visibility_cancels_original_receiver() {
    failed_final_release("resize", true);
}
#[test]
fn review_capture_geometry_failed_offset_final_up_cancels_original_receiver() {
    failed_final_release("offset", false);
}
#[test]
fn review_capture_geometry_failed_resize_final_up_cancels_original_receiver() {
    failed_final_release("resize", false);
}
#[test]
fn review_capture_geometry_failed_partial_up_preserves_foreign_appearance() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    let (mut other, foreign_probe) = workspace_fixture();
    let foreign = other.pool_mut().remove_content(&TileId(1)).unwrap();
    workspace.pool_mut().insert(TileId(2), foreign);
    let mut primary = workspace.pool().placement(&TileId(1)).unwrap().clone();
    primary.focused = false;
    let mut foreign_placement = primary.clone();
    foreign_placement.content = TileId(2);
    foreign_placement.window = 1;
    workspace.pool_mut().set_placements([
        (TileId(1), primary.clone()), (TileId(2), foreign_placement.clone()),
    ]);
    assert!(workspace.pool_mut().input_at(0, pointer(37, Down, Some(Left), 1, 35.0)).handled);
    assert!(workspace.pool_mut().input_at(0, pointer(37, Down, Some(Middle), 5, 55.0)).handled);
    assert!(workspace.pool_mut().input_at(1, pointer(49, Down, Some(Right), 2, 65.0)).handled);
    probe.lock().unwrap().failed_geometry = Some("offset");
    primary.scale_factor = 3.0;
    primary.rect = WorkspaceRect::new(40.0, 50.0, 200.0, 100.0);
    workspace.pool_mut().set_placements([
        (TileId(1), primary), (TileId(2), foreign_placement),
    ]);
    let effect = workspace.pool_mut().input_at(0, pointer(37, Up, Some(Left), 4, 70.0));
    assert!(effect.error.is_some() && !effect.handled);
    assert_eq!(effect.pointer_capture, Some(false));
    assert_eq!(workspace.pool().captured_placement(37), None,
        "geometry failure cancels the whole affected gesture, including its remaining Middle");
    assert_eq!(workspace.pool().captured_placement(49), Some(&TileId(2)));
    assert!(surface_cancels(&foreign_probe).is_empty());
    let cancels = surface_cancels(&probe);
    assert_eq!(cancels.len(), 1);
    assert_eq!(cancels[0].pointer_id, 37);
    assert_eq!(cancels[0].position, inker::PhysicalPosition { x: 70.0, y: 30.0 });
    probe.lock().unwrap().failed_geometry = None;
    let marker = calls(&probe).len();
    assert!(!workspace.pool_mut().input_at(0, pointer(37, Up, Some(Middle), 0, 70.0)).handled);
    assert_eq!(calls(&probe).len(), marker);
    assert!(workspace.pool_mut().input_at(1, pointer(49, Up, Some(Right), 0, 65.0)).handled);
    assert_eq!(workspace.pool().captured_placement(49), None);
    assert!(surface_cancels(&foreign_probe).is_empty());
    assert!(calls(&foreign_probe).iter().any(|call| matches!(call,
        Call::Pointer(1, event) if event.phase == Up && event.pointer_id == 49)));
    assert!(!workspace.has_active_pointer_capture());
}
// Avoid shadowing the fixture constructor inside the two-content case.
fn workspace_fixture() -> (PeltWorkspace<String>, Arc<Mutex<Probe>>) {
    workspace(true)
}
#[test]
fn review_capture_geometry_successful_prepare_keeps_remaining_chord() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    workspace.input_event(pointer(37, Down, Some(Middle), 5, 55.0));
    change_geometry(&mut workspace);
    let effect = workspace.input_event(pointer(37, Up, Some(Left), 4, 70.0));
    assert!(effect.handled && effect.error.is_none());
    assert_eq!(effect.pointer_capture, Some(true));
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    assert!(surface_cancels(&probe).is_empty());
    let effect = workspace.input_event(pointer(37, Up, Some(Middle), 0, 70.0));
    assert!(effect.handled && effect.error.is_none());
    assert_eq!(effect.pointer_capture, Some(false));
    assert!(!workspace.has_active_pointer_capture());
    assert!(surface_cancels(&probe).is_empty());
}
