// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Receiver bookkeeping must follow admitted pool custody, including failed terminals.
use inker::{
    DocumentSession, PointerPhase, SessionButtonState, SessionClick, SessionEngine, SessionError,
    SessionInput, SessionInputResult, SessionModifiers, SessionNavigationCommand, SessionRegistry,
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
}
#[derive(Default)]
struct Probe {
    next: usize,
    calls: Vec<Call>,
    failed_phase: Option<PointerPhase>,
    failed_document_down: bool,
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
fn document_cancels(probe: &Arc<Mutex<Probe>>) -> usize {
    calls(probe)
        .iter()
        .filter(|call| **call == Call::Session(1, SessionInput::Cancel))
        .count()
}
fn replace_surface(workspace: &mut PeltWorkspace<String>) {
    assert!(
        workspace
            .set_route_override(TileId(1), Some("terminal.document".to_owned()))
            .unwrap()
    );
}
fn reload_document(workspace: &mut PeltWorkspace<String>) {
    assert!(
        workspace
            .command_for(TileId(1), SessionNavigationCommand::Reload)
            .navigated
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
#[test]
fn review_capture_terminal_surface_failed_down_and_hover_never_admit() {
    use inker::{MouseButton::Left, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    assert!(
        workspace
            .input_event(pointer(37, Move, None, 0, 35.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
    probe.lock().unwrap().failed_phase = Some(Down);
    let effect = workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    assert!(!effect.handled && effect.error.is_some());
    assert!(!workspace.has_active_pointer_capture());
    replace_surface(&mut workspace);
    assert_dropped(&probe);
    assert!(
        surface_cancels(&probe).is_empty(),
        "failed Down and hover must not manufacture retirement input"
    );
}
#[test]
fn review_capture_terminal_surface_failed_cancel_not_repeated_on_replace() {
    use inker::{MouseButton::Left, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    assert!(
        workspace
            .input_event(pointer(37, Down, Some(Left), 1, 35.0))
            .handled
    );
    probe.lock().unwrap().failed_phase = Some(Cancel);
    let effect = workspace.input_event(pointer(37, Cancel, None, 0, 75.0));
    assert!(!effect.handled && effect.error.is_some());
    assert_eq!(effect.pointer_capture, Some(false));
    assert_eq!(workspace.pool().captured_placement(37), None);
    assert_eq!(surface_cancels(&probe).len(), 1);
    replace_surface(&mut workspace);
    assert_dropped(&probe);
    assert_eq!(
        surface_cancels(&probe).len(),
        1,
        "failed terminal already retired custody; replacement must not retry Cancel"
    );
}
#[test]
fn review_capture_terminal_surface_failed_final_up_not_cancelled_on_replace() {
    use inker::{MouseButton::Left, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    probe.lock().unwrap().failed_phase = Some(Up);
    let effect = workspace.input_event(pointer(37, Up, Some(Left), 0, 75.0));
    assert!(!effect.handled && effect.error.is_some());
    assert_eq!(effect.pointer_capture, Some(false));
    assert!(!workspace.has_active_pointer_capture());
    replace_surface(&mut workspace);
    assert_dropped(&probe);
    assert!(
        surface_cancels(&probe).is_empty(),
        "failed final Up must not be followed by retirement Cancel"
    );
}
#[test]
fn review_capture_terminal_surface_failed_secondary_down_never_extends_admission() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    probe.lock().unwrap().failed_phase = Some(Down);
    assert!(
        workspace
            .input_event(pointer(37, Down, Some(Middle), 5, 55.0))
            .error
            .is_some()
    );
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    probe.lock().unwrap().failed_phase = Some(Up);
    let effect = workspace.input_event(pointer(37, Up, Some(Left), 4, 75.0));
    assert_eq!(effect.pointer_capture, Some(false));
    assert!(!workspace.has_active_pointer_capture());
    replace_surface(&mut workspace);
    assert_dropped(&probe);
    assert!(
        surface_cancels(&probe).is_empty(),
        "physical unadmitted Middle must not extend a retired Left gesture"
    );
}
#[test]
fn review_capture_terminal_document_global_cancel_clears_nonzero_receivers() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = workspace(false);
    workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    workspace.input_event(pointer(49, Down, Some(Right), 2, 55.0));
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    assert_eq!(workspace.pool().captured_placement(49), Some(&TileId(1)));
    workspace
        .controller_mut(TileId(1))
        .unwrap()
        .input(SessionInput::Cancel);
    assert_eq!(document_cancels(&probe), 1);
    // This direct controller operation does not assert a new pool-wide Cancel
    // policy. Replacement subsequently invalidates both captured identities.
    reload_document(&mut workspace);
    assert_dropped(&probe);
    assert_eq!(
        document_cancels(&probe),
        1,
        "global Session Cancel already retired receiver bookkeeping for every pointer id"
    );
    assert!(!workspace.has_active_pointer_capture());
}
#[test]
fn review_capture_terminal_document_pointer_cancel_keeps_other_id_retirement() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = workspace(false);
    workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    workspace.input_event(pointer(49, Down, Some(Right), 2, 55.0));
    assert!(
        workspace
            .input_event(pointer(37, Cancel, None, 0, 75.0))
            .handled
    );
    assert_eq!(workspace.pool().captured_placement(37), None);
    assert_eq!(workspace.pool().captured_placement(49), Some(&TileId(1)));
    assert_eq!(document_cancels(&probe), 1);
    reload_document(&mut workspace);
    assert_dropped(&probe);
    assert_eq!(
        document_cancels(&probe),
        2,
        "per-Pointer Cancel must not erase another id's retained admission"
    );
    let calls = calls(&probe);
    assert!(
        calls
            .iter()
            .rposition(|call| *call == Call::Session(1, SessionInput::Cancel))
            .unwrap()
            < calls
                .iter()
                .position(|call| *call == Call::Drop(1))
                .unwrap()
    );
    assert!(!workspace.has_active_pointer_capture());
}
#[test]
fn review_capture_terminal_document_failed_down_and_hover_never_admit() {
    use inker::{MouseButton::Left, PointerPhase::*};
    let (mut workspace, probe) = workspace(false);
    assert!(
        workspace
            .input_event(pointer(37, Move, None, 0, 35.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
    probe.lock().unwrap().failed_document_down = true;
    assert!(
        !workspace
            .input_event(pointer(37, Down, Some(Left), 1, 35.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
    reload_document(&mut workspace);
    assert_dropped(&probe);
    assert_eq!(document_cancels(&probe), 0);
}
#[test]
fn review_capture_terminal_surface_failed_partial_up_keeps_remaining_admitted_mask() {
    use inker::{MouseButton::*, PointerPhase::*};
    let (mut workspace, probe) = workspace(true);
    workspace.input_event(pointer(37, Down, Some(Left), 1, 35.0));
    workspace.input_event(pointer(37, Down, Some(Middle), 5, 55.0));
    probe.lock().unwrap().failed_phase = Some(Up);
    let effect = workspace.input_event(pointer(37, Up, Some(Left), 4, 75.0));
    assert!(effect.error.is_some());
    assert_eq!(effect.pointer_capture, Some(true));
    assert_eq!(workspace.pool().captured_placement(37), Some(&TileId(1)));
    replace_surface(&mut workspace);
    assert_dropped(&probe);
    let cancels = surface_cancels(&probe);
    assert_eq!(
        cancels.len(),
        1,
        "remaining admitted Middle still needs one retirement Cancel"
    );
    assert_eq!(cancels[0].pointer_id, 37);
    assert_eq!(
        cancels[0].position,
        inker::PhysicalPosition { x: 110.0, y: 30.0 }
    );
    assert_eq!(cancels[0].buttons, inker::PointerButtons::NONE);
    assert_eq!(cancels[0].button, None);
    let calls = calls(&probe);
    assert!(
        calls
            .iter()
            .position(|call| matches!(call,Call::Pointer(1,event)if event.phase==Cancel))
            .unwrap()
            < calls
                .iter()
                .position(|call| *call == Call::Drop(1))
                .unwrap()
    );
    assert!(!workspace.has_active_pointer_capture());
}
