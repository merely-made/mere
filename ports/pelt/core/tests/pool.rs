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
use workbench::{ContentSource, DocumentRef, Tile, TileEvent, TileId, TileTree};

#[derive(Default)]
struct Probe {
    spawns: Vec<String>,
    pumps: HashMap<String, usize>,
    hidden: HashMap<String, Vec<bool>>,
    inputs: Vec<(String, u32)>,
    focus: Vec<bool>,
    frames: usize,
    input_scroll: Vec<f32>,
    async_scroll: Option<f32>,
    keys: Vec<(inker::SessionKey, SessionButtonState, SessionModifiers)>,
    ime: Vec<inker::SessionIme>,
    points: Vec<(f32, f32)>,
    wheels: Vec<(f32, f32, f32, f32)>,
    timer_ms: Option<u64>,
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
            laid_out_scroll: 0.0,
            width: request.viewport.0,
            probe: self.0.clone(),
        }))
    }
}

struct FakeSession {
    address: String,
    scroll: f32,
    laid_out_scroll: f32,
    width: u32,
    probe: Arc<Mutex<Probe>>,
}

impl DocumentSession<String> for FakeSession {
    fn pump(&mut self, _now_ms: f64) {
        if let Some(scroll) = self.probe.lock().unwrap().async_scroll.take() {
            self.scroll = scroll;
        }
        *self
            .probe
            .lock()
            .unwrap()
            .pumps
            .entry(self.address.clone())
            .or_default() += 1;
    }

    fn settled(&mut self) -> bool {
        self.probe
            .lock()
            .unwrap()
            .pumps
            .get(&self.address)
            .copied()
            .unwrap_or(0)
            >= 3
    }

    fn pending_work(&mut self) -> inker::session_engine::SessionPendingWork {
        self.probe
            .lock()
            .unwrap()
            .timer_ms
            .map(|milliseconds| {
                inker::session_engine::SessionPendingWork::timer(std::time::Duration::from_millis(
                    milliseconds,
                ))
            })
            .unwrap_or_default()
    }

    fn frame(&mut self, width: u32, height: u32) -> String {
        self.probe.lock().unwrap().frames += 1;
        self.width = width;
        self.laid_out_scroll = self.scroll;
        format!("{}@{width}x{height}+{}", self.address, self.scroll)
    }

    fn scroll_by(&mut self, _dx: f32, dy: f32) -> bool {
        self.scroll += dy;
        true
    }

    fn scroll_for_key(&mut self, _key: SessionScrollKey) -> bool {
        self.probe
            .lock()
            .unwrap()
            .inputs
            .push(("scroll".into(), self.width));
        self.scroll += 40.0;
        true
    }

    fn click_at(&mut self, x: f32, y: f32) -> SessionClick {
        self.probe.lock().unwrap().points.push((x, y));
        self.probe
            .lock()
            .unwrap()
            .input_scroll
            .push(self.laid_out_scroll);
        self.probe
            .lock()
            .unwrap()
            .inputs
            .push(("pointer".into(), self.width));
        if x < 10.0 {
            SessionClick::Navigate("next.html".to_owned())
        } else {
            SessionClick::Handled
        }
    }

    fn pointer_move(&mut self, x: f32, y: f32) -> bool {
        self.probe.lock().unwrap().points.push((x, y));
        true
    }

    fn pointer_up(&mut self, x: f32, y: f32) -> SessionClick {
        self.probe.lock().unwrap().points.push((x, y));
        SessionClick::Handled
    }

    fn scroll_at(&mut self, x: f32, y: f32, dx: f32, dy: f32) -> bool {
        self.probe.lock().unwrap().wheels.push((x, y, dx, dy));
        self.scroll_by(dx, dy)
    }

    fn links(&self) -> Vec<inker::SessionLink> {
        Vec::new()
    }

    fn dispatch_accessibility_action(
        &mut self,
        request: &inker::DocumentA11yActionRequest,
    ) -> bool {
        if request.action != inker::DocumentA11yAction::ScrollIntoView {
            return false;
        }
        self.scroll = 57.0;
        true
    }

    fn key_input(
        &mut self,
        key: inker::SessionKey,
        state: SessionButtonState,
        modifiers: SessionModifiers,
        _repeat: bool,
    ) -> inker::SessionEffect {
        self.probe
            .lock()
            .unwrap()
            .keys
            .push((key, state, modifiers));
        inker::SessionEffect::Handled
    }

    fn ime_input(&mut self, ime: inker::SessionIme) -> bool {
        self.probe.lock().unwrap().ime.push(ime);
        true
    }

    fn focus_input(&mut self, focused: bool) {
        self.probe.lock().unwrap().focus.push(focused);
        if focused {
            self.probe
                .lock()
                .unwrap()
                .inputs
                .push(("focus".into(), self.width));
        }
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

use pelt_core::{PeltControllerPool, PeltPlacement};

fn pool() -> PeltControllerPool<u64, u64, String> {
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, Arc::new(Mutex::new(Probe::default())))
    })
    .unwrap();
    let mut pool = PeltControllerPool::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool
}

fn placement(window: u64, width: f32) -> PeltPlacement<u64> {
    PeltPlacement {
        content: 1,
        window,
        rect: WorkspaceRect {
            x: 20.0,
            y: 30.0,
            width,
            height: 100.0,
        },
        visible: true,
        focused: false,
        scale_factor: 1.0,
    }
}

#[test]
fn final_placement_close_retains_content_until_explicit_removal() {
    let mut pool = pool();
    let identity = pool.content(&1).unwrap().session_identity();
    pool.set_placements([(10, placement(0, 200.0)), (11, placement(1, 400.0))]);
    pool.set_placements([]);
    assert_eq!(pool.content(&1).unwrap().session_identity(), identity);
    pool.set_placements([(12, placement(2, 300.0))]);
    assert_eq!(pool.content(&1).unwrap().session_identity(), identity);
    pool.remove_content(&1);
    assert!(pool.content(&1).is_none());
    assert!(pool.frame(&12).is_none());
}

#[test]
fn placement_generation_invalidates_resized_and_reused_action_geometry() {
    let mut pool = pool();
    pool.set_placements([(10, placement(0, 200.0))]);
    let stamp = pool.placement_generation(&10).unwrap();
    pool.set_placements([(10, placement(0, 400.0))]);
    assert!(!pool.admits(&10, stamp));
    let stamp = pool.placement_generation(&10).unwrap();
    assert!(pool.admits(&10, stamp));
    pool.set_placements([]);
    pool.set_placements([(10, placement(0, 400.0))]);
    assert!(!pool.admits(&10, stamp));
}

#[test]
fn hidden_and_invalid_placements_never_frame() {
    let mut pool = pool();
    let mut hidden = placement(0, 200.0);
    hidden.visible = false;
    pool.set_placements([
        (10, hidden),
        (11, placement(1, f32::NAN)),
        (12, placement(2, 0.0)),
    ]);
    for id in [10, 11, 12] {
        assert!(pool.frame(&id).is_none());
    }
}

#[test]
fn repeated_content_placements_keep_their_own_geometry() {
    let mut pool = pool();
    pool.set_placements([(10, placement(0, 200.0)), (11, placement(1, 400.0))]);
    for (id, size) in [(10, "200x100"), (11, "400x100")] {
        let Some(pelt_core::PeltLayer::Document(frame)) = pool.frame(&id) else {
            panic!("document frame");
        };
        assert!(frame.contains(size), "{frame}");
    }
}

#[test]
fn workspace_reopens_closed_content_with_scroll_history_and_identity() {
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, Arc::new(Mutex::new(Probe::default())))
    })
    .unwrap();
    workspace.command(SessionNavigationCommand::Address("next.html".to_owned()));
    let identity = workspace.controller(TileId(1)).unwrap().session_identity();
    workspace
        .controller_mut(TileId(1))
        .unwrap()
        .scroll_by(0.0, 57.0);
    workspace.apply(&TileEvent::Closed(TileId(1)));
    assert!(workspace.controller(TileId(1)).is_none());
    assert!(workspace.reopen_tile(TileId(1)));
    let content = workspace.controller_mut(TileId(1)).unwrap();
    assert_eq!(content.session_identity(), identity);
    assert!(content.can_go_back());
    assert!(content.address().ends_with("next.html"));
    assert!(content.frame(200, 100).ends_with("+57"));
    workspace.remove_content(TileId(1));
    assert!(!workspace.reopen_tile(TileId(1)));
}

#[test]
fn same_epoch_deduplicates_unchanged_windows_but_not_changed_placements() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool.set_placements([(10, placement(0, 200.0))]);
    pool.pump_at(123.0);
    pool.pump_at(123.0);
    assert_eq!(probe.lock().unwrap().pumps["a/index.html"], 1);
    pool.set_placements([(10, placement(0, 300.0)), (11, placement(1, 400.0))]);
    pool.pump_at(123.0);
    assert_eq!(
        probe.lock().unwrap().pumps["a/index.html"],
        2,
        "new same-epoch arrangement work must be admitted"
    );
    pool.content_mut(&1)
        .unwrap()
        .document_mut()
        .unwrap()
        .scroll_by(0.0, 12.0);
    pool.pump_at(123.0);
    assert_eq!(
        probe.lock().unwrap().pumps["a/index.html"],
        3,
        "completion or mutation invalidates the epoch cache"
    );
}

#[test]
fn invalid_geometry_is_hidden_and_mutable_iteration_admits_new_same_epoch_work() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool.set_placements([(10, placement(0, 0.0))]);
    assert_eq!(
        probe.lock().unwrap().hidden["a/index.html"].last(),
        Some(&true)
    );
    pool.pump_at(123.0);
    for (_, content) in pool.iter_mut() {
        content.document_mut().unwrap().scroll_by(0.0, 12.0);
    }
    pool.pump_at(123.0);
    assert_eq!(probe.lock().unwrap().pumps["a/index.html"], 2);
}

#[test]
fn hidden_pending_work_advances_without_requesting_a_visible_frame() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool.set_placements([]);
    assert!(
        pool.pump_at(123.0),
        "hidden work still needs a scheduling turn"
    );
    assert_eq!(probe.lock().unwrap().pumps["a/index.html"], 1);
    assert!(!pool.visible_needs_frame());
    pool.set_placements([(10, placement(0, 200.0)), (11, placement(1, 300.0))]);
    assert!(pool.pump_at(124.0));
    assert!(pool.visible_needs_frame());
    assert_eq!(
        probe.lock().unwrap().pumps["a/index.html"],
        2,
        "two appearances pump one retained session once"
    );
    assert!(!pool.pump_at(125.0));
    assert!(!pool.visible_needs_frame(), "finite work can settle");
}

#[test]
fn repeated_placement_input_prepares_the_addressed_layout() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool.set_placements([(10, placement(0, 200.0)), (11, placement(1, 400.0))]);
    pool.frame(&10);
    pool.frame(&11);
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(
        probe.lock().unwrap().inputs.last(),
        Some(&("pointer".into(), 200)),
        "the other window's last viewport cannot determine this hit"
    );
    pool.frame(&11);
    pool.input(&10, SessionInput::Focus(true).into());
    assert_eq!(
        probe.lock().unwrap().inputs.last(),
        Some(&("focus".into(), 200)),
        "caret and focus navigation use the addressed viewport"
    );
    pool.prepare(&10).unwrap();
    let frames = probe.lock().unwrap().frames;
    let _ = pool.content(&1).unwrap().document().unwrap().inspect();
    pool.prepare(&10).unwrap();
    assert_eq!(
        probe.lock().unwrap().frames,
        frames,
        "unchanged geometry does not reframe and invalidate semantic snapshots"
    );
}

#[test]
fn mutable_iteration_invalidates_same_epoch_completion_observation() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool.pump_at(123.0);
    for (_, content) in pool.iter_mut() {
        content.document_mut().unwrap().scroll_by(0.0, 12.0);
    }
    pool.pump_at(123.0);
    assert_eq!(probe.lock().unwrap().pumps["a/index.html"], 2);
}

#[test]
fn placement_focus_is_independent_of_visibility_and_replacement_keeps_arrangement() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::<u64, u64, String>::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    probe.lock().unwrap().focus.clear();
    pool.set_placements([(10, placement(0, 200.0))]);
    assert_eq!(
        probe.lock().unwrap().hidden["a/index.html"].last(),
        Some(&false)
    );
    let mut focused = placement(0, 200.0);
    focused.focused = true;
    pool.set_placements([(10, focused.clone())]);
    assert_eq!(probe.lock().unwrap().focus.last(), Some(&true));
    let old_stamp = pool.placement_generation(&10).unwrap();
    let old_identity = pool.content(&1).unwrap().session_identity();
    let replacement = pelt_core::PeltContent::from_controller(
        controller(&tile(1, "b/index.html"), probe.clone()).unwrap(),
        pool.content(&1).unwrap().route().clone(),
    );
    pool.insert(1, replacement);
    assert!(
        pool.placement(&10).is_some(),
        "replacing a retained controller must keep host arrangement"
    );
    assert!(!pool.admits(&10, old_stamp));
    assert_ne!(pool.content(&1).unwrap().session_identity(), old_identity);
    assert_eq!(
        probe.lock().unwrap().hidden["b/index.html"].last(),
        Some(&false)
    );
    focused.focused = false;
    pool.set_placements([(10, focused)]);
    assert_eq!(probe.lock().unwrap().focus.last(), Some(&false));
}

#[test]
fn replacing_content_preserves_current_placements_with_new_admission() {
    let mut pool = pool();
    pool.set_placements([(10, placement(0, 200.0))]);
    let generation = pool.placement_generation(&10).unwrap();
    let replacement = pelt_core::PeltContent::from_controller(
        controller(
            &tile(1, "replacement.html"),
            Arc::new(Mutex::new(Probe::default())),
        )
        .unwrap(),
        pool.content(&1).unwrap().route().clone(),
    );
    pool.insert(1, replacement);
    assert!(
        pool.placement(&10).is_some(),
        "replacement cannot delete host arrangement"
    );
    assert!(!pool.admits(&10, generation));
    assert!(pool.frame(&10).is_some());
}

fn layout_pool(probe: Arc<Mutex<Probe>>) -> PeltControllerPool<u64, u64, String> {
    let mut workspace = PeltWorkspace::try_new(TileTree::single(tile(1, "a/index.html")), |t| {
        controller(t, probe.clone())
    })
    .unwrap();
    let mut pool = PeltControllerPool::new();
    pool.insert(1, workspace.remove_content(TileId(1)).unwrap());
    pool.set_placements([(10, placement(0, 200.0))]);
    pool.frame(&10);
    pool
}

#[test]
fn mutable_session_access_invalidates_same_size_input_layout() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    pool.content_mut(&1)
        .unwrap()
        .document_mut()
        .unwrap()
        .scroll_by(0.0, 12.0);
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(
        probe.lock().unwrap().input_scroll.last(),
        Some(&12.0),
        "same viewport does not prove unchanged hit geometry after mutation"
    );
}

#[test]
fn async_pump_invalidates_same_size_input_layout_without_hidden_paint() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    pool.set_placements([]);
    let frames = probe.lock().unwrap().frames;
    probe.lock().unwrap().async_scroll = Some(34.0);
    pool.pump_at(1.0);
    assert_eq!(
        probe.lock().unwrap().frames,
        frames,
        "hidden completion must not paint"
    );
    pool.set_placements([(10, placement(0, 200.0))]);
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(
        probe.lock().unwrap().input_scroll.last(),
        Some(&34.0),
        "settled and same viewport do not establish current layout"
    );
}

#[test]
fn routed_scroll_invalidates_same_size_input_layout() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    pool.input(
        &10,
        pelt_core::PeltInput::Wheel {
            position: inker::PhysicalPosition { x: 50.0, y: 50.0 },
            delta: (0.0, 23.0),
            modifiers: SessionModifiers::default(),
        },
    );
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(
        probe.lock().unwrap().input_scroll.last(),
        Some(&23.0),
        "scroll must update semantic hit geometry before the next pointer"
    );
}

#[test]
fn moving_focus_between_appearances_prepares_each_addressed_viewport() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    let a = placement(0, 200.0);
    let b = placement(1, 400.0);
    pool.set_placements([(10, a.clone()), (11, b.clone())]);
    pool.frame(&11);
    let mut focused_a = a.clone();
    focused_a.focused = true;
    pool.set_placements([(10, focused_a), (11, b.clone())]);
    assert_eq!(
        probe.lock().unwrap().inputs.last(),
        Some(&("focus".into(), 200)),
        "focus arrives after its own appearance layout"
    );
    let mut focused_b = b;
    focused_b.focused = true;
    pool.set_placements([(10, a), (11, focused_b)]);
    assert_eq!(
        probe.lock().unwrap().inputs.last(),
        Some(&("focus".into(), 400)),
        "content remains focused, but its newly focused appearance changed"
    );
}

#[test]
fn neutral_keyboard_lowers_document_keys_and_composed_text_explicitly() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    pool.set_placements([(10, placement(0, 200.0))]);
    let keyboard = |key_code, pressed, text: Option<&str>| {
        pelt_core::PeltInput::Keyboard(inker::KeyboardEvent {
            key_code,
            scan_code: 44,
            modifiers: inker::KeyboardModifiers {
                ctrl: true,
                shift: true,
                alt: false,
                meta: false,
            },
            pressed,
            text: text.map(str::to_owned),
        })
    };
    assert!(pool.input(&10, keyboard(37, true, None)).handled);
    assert!(pool.input(&10, keyboard(37, false, None)).handled);
    assert!(
        pool.input(&10, keyboard(0, true, Some("é JSON payload")))
            .handled
    );
    assert_eq!(
        probe.lock().unwrap().keys,
        vec![
            (
                inker::SessionKey::ArrowLeft,
                SessionButtonState::Pressed,
                SessionModifiers {
                    control: true,
                    shift: true,
                    alt: false,
                    meta: false
                }
            ),
            (
                inker::SessionKey::ArrowLeft,
                SessionButtonState::Released,
                SessionModifiers {
                    control: true,
                    shift: true,
                    alt: false,
                    meta: false
                }
            ),
        ]
    );
    assert_eq!(
        probe.lock().unwrap().ime,
        vec![inker::SessionIme::Commit("é JSON payload".into())]
    );
    let unsupported = pool.input(&10, keyboard(0xFFFF, true, None));
    assert!(!unsupported.handled);
    assert!(
        unsupported.error.is_some(),
        "unknown native keys cannot silently disappear in a document lane"
    );
}

#[test]
fn raw_session_mutation_invalidates_input_layout() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    pool.content_mut(&1)
        .unwrap()
        .document_mut()
        .unwrap()
        .session_mut()
        .scroll_by(0.0, 41.0);
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(probe.lock().unwrap().input_scroll.last(), Some(&41.0));
}

#[test]
fn handled_accessibility_action_invalidates_input_layout() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    assert!(
        pool.content_mut(&1)
            .unwrap()
            .document_mut()
            .unwrap()
            .dispatch_accessibility_action(&inker::DocumentA11yActionRequest {
                revision: 1,
                target: inker::DocumentA11yNodeId::new(1),
                action: inker::DocumentA11yAction::ScrollIntoView,
                data: None,
            })
    );
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(probe.lock().unwrap().input_scroll.last(), Some(&57.0));
}

#[test]
fn pending_observation_is_read_only_and_redraw_is_scoped_to_visible_windows() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    let mut hidden = placement(8, 400.0);
    hidden.visible = false;
    pool.set_placements([(10, placement(7, 200.0)), (11, hidden)]);
    assert!(pool.pump_at(200.0));
    let frames = probe.lock().unwrap().frames;
    assert_eq!(
        pool.pending_contents().copied().collect::<Vec<_>>(),
        vec![1]
    );
    assert!(pool.window_needs_frame(7));
    assert!(
        !pool.window_has_surface(7),
        "a retained document is not a native mailbox producer"
    );
    assert!(!pool.window_needs_frame(8));
    assert!(!pool.window_needs_frame(9));
    assert_eq!(probe.lock().unwrap().frames, frames);
    let pumps = probe.lock().unwrap().pumps.clone();
    assert!(pool.pump_at(200.0));
    assert_eq!(
        probe.lock().unwrap().pumps,
        pumps,
        "pending observation must not invalidate pump deduplication"
    );
    pool.set_placements([]);
    assert!(!pool.window_needs_frame(7));
    assert_eq!(
        pool.pending_contents().count(),
        1,
        "hidden work remains scheduled"
    );
}

#[test]
fn scheduled_document_work_preserves_prepared_routes_and_current_receiver_custody() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    let mut hidden = placement(8, 400.0);
    hidden.visible = false;
    pool.set_placements([(10, placement(7, 200.0)), (11, hidden)]);
    pool.frame(&10);
    assert!(pool.pump_at(1.0));
    assert!(pool.pump_at(2.0));
    assert!(
        !pool.pump_at(3.0),
        "completion can settle before a future timer"
    );
    pool.frame(&10);
    probe.lock().unwrap().timer_ms = Some(1000);
    let frames = probe.lock().unwrap().frames;
    let pumps = probe.lock().unwrap().pumps.clone();
    let work = pool.document_work().collect::<Vec<_>>();
    assert_eq!(work.len(), 1);
    assert_eq!(work[0].work.timer, Some(std::time::Duration::from_secs(1)));
    assert!(pool.window_has_document_work(7, &work[0]));
    assert!(!pool.window_has_document_work(8, &work[0]));
    assert!(!pool.window_has_document_work(9, &work[0]));
    assert!(!pool.pump_at(3.0));
    assert_eq!(
        probe.lock().unwrap().pumps,
        pumps,
        "observation keeps pump deduplication"
    );
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(
        probe.lock().unwrap().frames,
        frames,
        "observation keeps prepared input geometry"
    );
    pool.set_placements([(10, placement(9, 200.0))]);
    assert!(!pool.window_has_document_work(7, &work[0]));
    assert!(pool.window_has_document_work(9, &work[0]));
    let replacement = layout_pool(Arc::new(Mutex::new(Probe::default())))
        .remove_content(&1)
        .unwrap();
    pool.insert(1, replacement);
    assert!(
        !pool.window_has_document_work(9, &work[0]),
        "same key never admits a replaced receiver's timer"
    );
    let current = pool.document_work().next().unwrap();
    assert_ne!(current.identity, work[0].identity);
    assert!(pool.window_has_document_work(9, &current));
    pool.remove_content(&1);
    assert!(!pool.window_has_document_work(9, &current));
}

#[test]
fn keyboard_scroll_prepares_its_addressed_appearance_and_retires_old_hit_layout() {
    let probe = Arc::new(Mutex::new(Probe::default()));
    let mut pool = layout_pool(probe.clone());
    pool.set_placements([(10, placement(0, 200.0)), (11, placement(1, 400.0))]);
    pool.frame(&11);
    assert!(
        pool.input(
            &10,
            pelt_core::PeltInput::ScrollKey(SessionScrollKey::PageDown)
        )
        .handled
    );
    assert_eq!(
        probe.lock().unwrap().inputs.last(),
        Some(&("scroll".into(), 200))
    );
    pool.input(&10, press(50.0, 50.0).into());
    assert_eq!(probe.lock().unwrap().input_scroll.last(), Some(&40.0));
}

fn overlap_pool() -> (PeltControllerPool<u64, u64, String>, [Arc<Mutex<Probe>>; 2]) {
    let mut pool = pool();
    let probes = [
        Arc::new(Mutex::new(Probe::default())),
        Arc::new(Mutex::new(Probe::default())),
    ];
    let route = pool.content(&1).unwrap().route().clone();
    for (key, address, probe) in [
        (1, "https://lower.test/index", &probes[0]),
        (2, "https://upper.test/index", &probes[1]),
    ] {
        let mut sessions = SessionRegistry::new();
        sessions.register(Box::new(FakeEngine(probe.clone())));
        let controller = PeltController::new(
            sessions,
            SurfaceEngineRegistry::new(),
            PeltControllerConfig::new("fake", address, (100, 100)).with_host_history(),
            TestClock,
        )
        .unwrap();
        pool.insert(
            key,
            pelt_core::PeltContent::from_controller(controller, route.clone()),
        );
    }
    let mut lower = placement(0, 100.);
    lower.rect = WorkspaceRect::new(0., 0., 100., 100.);
    let mut upper = lower.clone();
    upper.content = 2;
    upper.rect = WorkspaceRect::new(20., 30., 100., 100.);
    pool.set_placements([(10, lower), (11, upper)]);
    (pool, probes)
}

#[test]
fn review_overlap_pool_last_placement_receives_pointer_and_navigation() {
    let (mut pool, probes) = overlap_pool();
    let effect = pool.input_at(0, press(27., 39.).into());
    assert_eq!(
        pool.captured_placement(0),
        Some(&11),
        "last presented content is topmost"
    );
    assert_eq!(probes[1].lock().unwrap().points, vec![(7., 9.)]);
    assert!(probes[0].lock().unwrap().points.is_empty());
    assert_eq!(
        effect.navigation.unwrap().request.address,
        "https://upper.test/next.html"
    );
}

#[test]
fn review_overlap_pool_capture_outside_and_unrelated_release() {
    let (mut pool, probes) = overlap_pool();
    pool.input_at(0, press(27., 39.).into());
    assert_eq!(pool.captured_placement(0), Some(&11));
    let unrelated = pool.input_at(
        0,
        SessionInput::PointerButton {
            x: 5.,
            y: 6.,
            button: SessionPointerButton::Secondary,
            state: SessionButtonState::Released,
            modifiers: Default::default(),
        }
        .into(),
    );
    assert!(!unrelated.handled);
    assert_eq!(pool.captured_placement(0), Some(&11));
    assert_eq!(probes[1].lock().unwrap().points.len(), 1);
    pool.input_at(
        0,
        SessionInput::PointerMoved {
            x: 207.,
            y: 239.,
            modifiers: Default::default(),
        }
        .into(),
    );
    pool.input_at(
        0,
        SessionInput::PointerButton {
            x: 207.,
            y: 239.,
            button: SessionPointerButton::Primary,
            state: SessionButtonState::Released,
            modifiers: Default::default(),
        }
        .into(),
    );
    assert_eq!(pool.captured_placement(0), None);
    assert_eq!(
        probes[1].lock().unwrap().points,
        vec![(7., 9.), (187., 209.), (187., 209.)]
    );
    assert!(probes[0].lock().unwrap().points.is_empty());
}

#[test]
fn review_overlap_pool_wheel_follows_current_stacking_and_skips_hidden_top() {
    let (mut pool, probes) = overlap_pool();
    let wheel = || pelt_core::PeltInput::Wheel {
        position: inker::PhysicalPosition { x: 27., y: 39. },
        delta: (3., 23.),
        modifiers: Default::default(),
    };
    assert!(pool.input_at(0, wheel()).handled);
    assert_eq!(probes[1].lock().unwrap().wheels, vec![(7., 9., 3., 23.)]);
    assert!(probes[0].lock().unwrap().wheels.is_empty());
    let lower = pool.placement(&10).unwrap().clone();
    let mut upper = pool.placement(&11).unwrap().clone();
    upper.visible = false;
    pool.set_placements([(10, lower), (11, upper)]);
    assert!(pool.input_at(0, wheel()).handled);
    assert_eq!(probes[0].lock().unwrap().wheels, vec![(27., 39., 3., 23.)]);
    assert_eq!(probes[1].lock().unwrap().wheels.len(), 1);
}
