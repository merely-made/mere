// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Additional receiver-transition controls through the public workspace API.
use inker::{
    DocumentSession, SessionButtonState, SessionClick, SessionEffect, SessionEngine, SessionError,
    SessionFormMethod, SessionFormSubmission, SessionInput, SessionInputResult, SessionModifiers,
    SessionNavigationCommand, SessionPointerButton, SessionRegistry, SessionScrollKey,
    SessionSpawnRequest, SurfaceEngineRegistry,
};
use pelt_core::{
    FetchOutcome, FetchRequestId, Fetched, PageProgress, PeltClock, PeltController,
    PeltControllerConfig, PeltLoadCommand, PeltSessionIdentity, PeltWorkspace, WorkspaceRect,
};
use std::any::Any;
use std::sync::{Arc, Mutex};
use workbench::{ContentSource, DocumentRef, Tile, TileId, TileTree};

#[derive(Clone, Debug, PartialEq)]
enum Call {
    Input(usize, SessionInput),
    Body(usize, String),
    Drop(usize),
}
#[derive(Default)]
struct Probe {
    next: usize,
    calls: Vec<Call>,
    on_down: Option<SessionEffect>,
    fail_spawn: bool,
}
struct Engine(Arc<Mutex<Probe>>);
struct Receiver {
    serial: usize,
    body: String,
    probe: Arc<Mutex<Probe>>,
}
impl Drop for Receiver {
    fn drop(&mut self) {
        self.probe
            .lock()
            .unwrap()
            .calls
            .push(Call::Drop(self.serial));
    }
}
impl SessionEngine<String> for Engine {
    fn engine_id(&self) -> &str {
        "retirement"
    }
    fn spawn(
        &self,
        request: &SessionSpawnRequest,
    ) -> Result<Box<dyn DocumentSession<String>>, SessionError> {
        let mut probe = self.0.lock().unwrap();
        if probe.fail_spawn {
            return Err(SessionError::SpawnFailed("replacement rejected".to_owned()));
        }
        probe.next += 1;
        Ok(Box::new(Receiver {
            serial: probe.next,
            body: request.body.clone().unwrap_or_default(),
            probe: self.0.clone(),
        }))
    }
}
impl DocumentSession<String> for Receiver {
    fn frame(&mut self, _: u32, _: u32) -> String {
        self.body.clone()
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
        let mut probe = self.probe.lock().unwrap();
        let capture = if down {
            Some(true)
        } else if matches!(&input, SessionInput::Cancel) {
            Some(false)
        } else {
            None
        };
        probe.calls.push(Call::Input(self.serial, input));
        let effect = if down {
            probe.on_down.take().unwrap_or(SessionEffect::Handled)
        } else {
            SessionEffect::Handled
        };
        SessionInputResult {
            effect,
            capture,
            cursor: None,
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
struct Clock;
impl PeltClock for Clock {
    fn now_ms(&self) -> f64 {
        1.0
    }
}
const START: &str = "https://capture.test/start";
const NEXT: &str = "https://capture.test/next";
fn page(request: FetchRequestId, body: &str) -> FetchOutcome {
    FetchOutcome {
        request,
        url: NEXT.to_owned(),
        result: Ok(Fetched::text(Some("text/plain".to_owned()), body)),
    }
}
fn fragment(request: FetchRequestId, body: &str) -> PageProgress {
    PageProgress {
        request,
        url: NEXT.to_owned(),
        response_url: NEXT.to_owned(),
        content_type: Some("text/plain".to_owned()),
        bytes: body.as_bytes().to_vec(),
    }
}
fn replace_body(session: &mut dyn DocumentSession<String>, _: &str, body: &str) -> bool {
    let receiver = session.as_any().downcast_mut::<Receiver>().unwrap();
    receiver.body = body.to_owned();
    receiver
        .probe
        .lock()
        .unwrap()
        .calls
        .push(Call::Body(receiver.serial, body.to_owned()));
    true
}
fn workspace(host_load: bool, host_history: bool) -> (PeltWorkspace<String>, Arc<Mutex<Probe>>) {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let tile = Tile {
        id: TileId(1),
        title: START.to_owned(),
        content: ContentSource::Document(DocumentRef(START.to_owned())),
        accent: None,
    };
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile), |_| {
        let mut engines = SessionRegistry::new();
        engines.register(Box::new(Engine(probe.clone())));
        let request = SessionSpawnRequest::new(START)
            .with_viewport(100, 80)
            .with_body("start body");
        let mut config = PeltControllerConfig::from_request("retirement", request);
        if host_load {
            config = config.with_host_loading();
        }
        if host_history {
            config = config.with_host_history();
        }
        let mut controller =
            PeltController::new(engines, SurfaceEngineRegistry::new(), config, Clock)?;
        controller.set_body_replacer(replace_body);
        Ok(controller)
    })
    .unwrap();
    workspace.set_content_rects([(TileId(1), WorkspaceRect::new(20.0, 30.0, 100.0, 80.0))]);
    probe.lock().unwrap().calls.clear();
    (workspace, probe)
}
fn button(state: SessionButtonState, x: f32) -> SessionInput {
    SessionInput::PointerButton {
        x,
        y: 45.0,
        button: SessionPointerButton::Primary,
        state,
        modifiers: SessionModifiers::default(),
    }
}
fn calls(probe: &Arc<Mutex<Probe>>) -> Vec<Call> {
    probe.lock().unwrap().calls.clone()
}
fn identity(workspace: &PeltWorkspace<String>) -> PeltSessionIdentity {
    workspace.document_session_identity(TileId(1)).unwrap()
}
fn assert_old_gesture_rejected(
    workspace: &mut PeltWorkspace<String>,
    probe: &Arc<Mutex<Probe>>,
    old: usize,
) {
    let before = calls(probe);
    let cancels: Vec<_> = before
        .iter()
        .enumerate()
        .filter(|(_, call)| **call == Call::Input(old, SessionInput::Cancel))
        .map(|(i, _)| i)
        .collect();
    assert_eq!(
        cancels.len(),
        1,
        "old receiver must receive exactly one Cancel"
    );
    let drop = before
        .iter()
        .position(|call| *call == Call::Drop(old))
        .expect("old receiver must be dropped");
    assert!(cancels[0] < drop, "old Cancel must precede receiver drop");
    assert!(!workspace.has_active_pointer_capture());
    assert_eq!(workspace.pool().captured_placement(0), None);
    // Terminal rejection is checked INSIDE the current rect too. An outside
    // coordinate alone could conceal accidental re-admission to the new session.
    assert!(
        !workspace
            .input(button(SessionButtonState::Released, 35.0))
            .handled
    );
    assert_eq!(
        calls(probe),
        before,
        "old Up must not reach the new receiver"
    );
    assert!(
        !workspace
            .input(SessionInput::PointerMoved {
                x: 250.0,
                y: 45.0,
                modifiers: SessionModifiers::default()
            })
            .handled
    );
    assert_eq!(
        calls(probe),
        before,
        "old outside Move must not reach the new receiver"
    );
    let new = probe.lock().unwrap().next;
    assert!(
        workspace
            .input(button(SessionButtonState::Pressed, 35.0))
            .handled
    );
    assert!(
        workspace
            .input(button(SessionButtonState::Released, 35.0))
            .handled
    );
    let mut down = button(SessionButtonState::Pressed, 15.0);
    let mut up = button(SessionButtonState::Released, 15.0);
    if let SessionInput::PointerButton { y, .. } = &mut down {
        *y = 15.0;
    }
    if let SessionInput::PointerButton { y, .. } = &mut up {
        *y = 15.0;
    }
    assert_eq!(
        &calls(probe)[before.len()..],
        &[Call::Input(new, down), Call::Input(new, up)]
    );
    assert!(!workspace.has_active_pointer_capture());
}
fn assert_retained(
    workspace: &PeltWorkspace<String>,
    probe: &Arc<Mutex<Probe>>,
    before: PeltSessionIdentity,
    old: usize,
) {
    assert_eq!(identity(workspace), before);
    assert_eq!(workspace.pool().captured_placement(0), Some(&TileId(1)));
    assert!(workspace.has_active_pointer_capture());
    assert!(!calls(probe).iter().any(|call| *call==Call::Input(old,SessionInput::Cancel)||*call==Call::Drop(old)));
}
fn start_pending(workspace: &mut PeltWorkspace<String>) -> FetchRequestId {
    let effect = workspace.command_for(
        TileId(1),
        SessionNavigationCommand::Address(NEXT.to_owned()),
    );
    assert!(effect.handled && !effect.navigated);
    match workspace
        .controller_mut(TileId(1))
        .unwrap()
        .take_load_commands()
        .as_slice()
    {
        [PeltLoadCommand::Fetch { request, url }] if url == NEXT => *request,
        other => panic!("expected only new pending fetch, got {other:?}"),
    }
}
fn input_navigation(effect: SessionEffect) {
    let (mut workspace, probe) = workspace(false, false);
    let before = identity(&workspace);
    probe.lock().unwrap().on_down = Some(effect);
    let effect = workspace.input(button(SessionButtonState::Pressed, 35.0));
    assert!(effect.handled && effect.navigated);
    assert_ne!(identity(&workspace), before);
    assert_eq!(
        effect.pointer_capture,
        Some(false),
        "navigating Down must not capture the replacement"
    );
    assert_old_gesture_rejected(&mut workspace, &probe, 1);
}
#[test]
fn review_workspace_retirement_same_down_navigate_cancels_before_drop() {
    input_navigation(SessionEffect::Navigate("next".to_owned()));
}
#[test]
fn review_workspace_retirement_same_down_form_get_cancels_before_drop() {
    input_navigation(SessionEffect::Submit(SessionFormSubmission {
        action: "next".to_owned(),
        method: SessionFormMethod::Get,
        fields: vec![("q".to_owned(), "elm & ash".to_owned())],
    }));
}
#[test]
fn review_workspace_retirement_pending_then_accepted_outcome() {
    let (mut workspace, probe) = workspace(true, false);
    workspace.input(button(SessionButtonState::Pressed, 35.0));
    let before = identity(&workspace);
    let request = start_pending(&mut workspace);
    assert_retained(&workspace, &probe, before, 1);
    // Pump, preparation and frame are normal mutable work, not replacement.
    workspace.pump_at(2.0);
    let _ = workspace.frame();
    assert_retained(&workspace, &probe, before, 1);
    assert!(
        workspace
            .input(SessionInput::PointerMoved {
                x: 250.0,
                y: 45.0,
                modifiers: SessionModifiers::default()
            })
            .handled
    );
    let effect = workspace
        .content_mut(TileId(1))
        .unwrap()
        .accept_outcome(page(request, "new body"), 3);
    assert!(effect.navigated);
    assert_ne!(identity(&workspace), before);
    // Query capture before sync_tile to prove direct mutable replacement
    // cannot borrow its old gesture even while its placement is unchanged.
    assert!(!workspace.has_active_pointer_capture());
    assert_old_gesture_rejected(&mut workspace, &probe, 1);
    workspace.sync_tile(TileId(1));
}
#[test]
fn review_workspace_retirement_pending_then_first_prefix_only() {
    let (mut workspace, probe) = workspace(true, false);
    workspace.input(button(SessionButtonState::Pressed, 35.0));
    let before = identity(&workspace);
    let request = start_pending(&mut workspace);
    assert_retained(&workspace, &probe, before, 1);
    let effect = workspace
        .content_mut(TileId(1))
        .unwrap()
        .accept_progress(fragment(request, "prefix"), 3);
    assert!(effect.navigated);
    assert_ne!(identity(&workspace), before);
    assert_old_gesture_rejected(&mut workspace, &probe, 1);
    let new = probe.lock().unwrap().next;
    workspace.input(button(SessionButtonState::Pressed, 35.0));
    let current = identity(&workspace);
    let marker = calls(&probe).len();
    let effect = workspace
        .content_mut(TileId(1))
        .unwrap()
        .accept_progress(fragment(request, " more"), 4);
    assert!(effect.handled && !effect.navigated);
    assert_retained(&workspace, &probe, current, new);
    let effect = workspace
        .content_mut(TileId(1))
        .unwrap()
        .accept_outcome(page(request, "prefix more complete"), 5);
    assert!(effect.handled && !effect.navigated);
    assert_retained(&workspace, &probe, current, new);
    assert!(
        calls(&probe)[marker..]
            .iter()
            .all(|call| matches!(call,Call::Body(serial,_) if *serial==new))
    );
    assert!(
        workspace
            .input(button(SessionButtonState::Released, 250.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
}
#[test]
fn review_workspace_retirement_failed_host_install_preserves_capture() {
    let (mut workspace, probe) = workspace(true, false);
    workspace.input(button(SessionButtonState::Pressed, 35.0));
    let before = identity(&workspace);
    let request = start_pending(&mut workspace);
    assert_retained(&workspace, &probe, before, 1);
    probe.lock().unwrap().fail_spawn = true;
    let effect = workspace
        .content_mut(TileId(1))
        .unwrap()
        .accept_progress(fragment(request, "prefix"), 3);
    assert!(effect.error.is_some() && !effect.navigated);
    assert_retained(&workspace, &probe, before, 1);
    let effect = workspace
        .content_mut(TileId(1))
        .unwrap()
        .accept_outcome(page(request, "complete"), 4);
    assert!(effect.error.is_some() && !effect.navigated);
    assert_retained(&workspace, &probe, before, 1);
    assert!(
        workspace
            .input(button(SessionButtonState::Released, 250.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
}
#[test]
fn review_workspace_retirement_host_history_request_preserves_capture() {
    let (mut workspace, probe) = workspace(false, true);
    let before = identity(&workspace);
    probe.lock().unwrap().on_down = Some(SessionEffect::Navigate("next".to_owned()));
    let effect = workspace.input(button(SessionButtonState::Pressed, 35.0));
    assert!(effect.navigation.is_some() && !effect.navigated);
    assert_retained(&workspace, &probe, before, 1);
    assert!(
        workspace
            .input(button(SessionButtonState::Released, 250.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
}
#[test]
fn review_workspace_retirement_failed_input_navigation_preserves_capture() {
    let (mut workspace, probe) = workspace(false, false);
    let before = identity(&workspace);
    {
        let mut state = probe.lock().unwrap();
        state.fail_spawn = true;
        state.on_down = Some(SessionEffect::Navigate("next".to_owned()));
    }
    let effect = workspace.input(button(SessionButtonState::Pressed, 35.0));
    assert!(effect.error.is_some() && !effect.navigated);
    assert_retained(&workspace, &probe, before, 1);
    assert!(
        workspace
            .input(button(SessionButtonState::Released, 250.0))
            .handled
    );
    assert!(!workspace.has_active_pointer_capture());
}
