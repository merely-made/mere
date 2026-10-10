// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use inker::*;
use pelt_core::*;
use std::{
    any::Any,
    sync::{Arc, Mutex},
};

struct Clock;
impl PeltClock for Clock {
    fn now_ms(&self) -> f64 {
        0.
    }
}
#[derive(Default)]
struct State {
    width: u32,
    revision: u64,
    focused: bool,
    value: String,
    clicks: usize,
    mutate_on_resize: bool,
}
struct Page(Arc<Mutex<State>>);
fn projection(s: &State) -> DocumentA11yProjection {
    DocumentA11yProjection::new(
        s.revision,
        DocumentA11ySupport::new(A11yCapability::Partial, ["test field"]).unwrap(),
        DocumentA11yNodeId::new(1),
        vec![DocumentA11yNode {
            id: DocumentA11yNodeId::new(1),
            parent: None,
            children: vec![],
            role: DocumentA11yRole::TextField,
            name: Some("Field".into()),
            description: None,
            value: Some(s.value.clone()),
            numeric_value: None,
            numeric_minimum: None,
            numeric_maximum: None,
            bounds: Some(DocumentA11yBounds {
                x: 0.,
                y: 0.,
                width: s.width as f32,
                height: 30.,
            }),
            state: DocumentA11yState {
                focused: s.focused,
                ..Default::default()
            },
            actions: vec![
                DocumentA11yAction::Focus,
                DocumentA11yAction::SetValue,
                DocumentA11yAction::Click,
            ],
        }],
    )
}
impl DocumentSession<String> for Page {
    fn frame(&mut self, w: u32, _h: u32) -> String {
        let mut s = self.0.lock().unwrap();
        if s.width != w {
            s.width = w;
            s.revision += 1;
            if s.mutate_on_resize {
                s.value = "frame side effect".into();
            }
        }
        format!("width={w}")
    }
    fn scroll_by(&mut self, _: f32, _: f32) -> bool {
        false
    }
    fn scroll_for_key(&mut self, _: SessionScrollKey) -> bool {
        false
    }
    fn click_at(&mut self, _: f32, _: f32) -> SessionClick {
        let mut s = self.0.lock().unwrap();
        s.clicks += 1;
        SessionClick::Navigate(format!("https://fixture.test/{}", s.width))
    }
    fn links(&self) -> Vec<SessionLink> {
        vec![]
    }
    fn accessibility_projection(&self) -> Option<DocumentA11yProjection> {
        Some(projection(&self.0.lock().unwrap()))
    }
    fn accessibility_click_target(
        &self,
        id: DocumentA11yNodeId,
    ) -> Option<DocumentA11yClickTarget> {
        let s = self.0.lock().unwrap();
        (id.get() == 1).then_some(DocumentA11yClickTarget {
            revision: s.revision,
            point: DocumentA11yPoint {
                x: s.width as f32 / 2.,
                y: 10.,
            },
        })
    }
    fn dispatch_accessibility_action(&mut self, request: &DocumentA11yActionRequest) -> bool {
        let mut s = self.0.lock().unwrap();
        if request.revision != s.revision || request.target.get() != 1 {
            return false;
        }
        match request.action {
            DocumentA11yAction::Focus => s.focused = true,
            DocumentA11yAction::SetValue => {
                let Some(DocumentA11yActionData::Value(value)) = &request.data else {
                    return false;
                };
                s.value = value.clone();
            },
            _ => return false,
        }
        s.revision += 1;
        true
    }
    fn as_any_ref(&self) -> &dyn Any {
        self
    }
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
fn content(state: &Arc<Mutex<State>>) -> PeltContent<String> {
    let controller = PeltController::from_session_shared(
        Arc::new(SessionRegistry::new()),
        Arc::new(SurfaceEngineRegistry::new()),
        PeltControllerConfig::new("fixture", "https://fixture.test/", (200, 200))
            .with_host_history(),
        Clock,
        Box::new(Page(state.clone())),
    );
    PeltContent::from_controller(
        controller,
        PeltRoute {
            decision: routing::EngineRouteDecision {
                engine_id: "fixture".into(),
                surface_contract: routing::SurfaceContract {
                    target: routing::SurfaceTargetId::new("fixture"),
                    mode: routing::SurfaceContractMode::CompositedTexture,
                },
            },
            source: PeltRouteSource::Automatic,
            state: PeltRouteState::Document,
        },
    )
}
fn placements() -> [(u8, PeltPlacement<u8>); 2] {
    [
        (
            1,
            PeltPlacement {
                content: 1,
                window: 0,
                rect: WorkspaceRect::new(0., 0., 200., 200.),
                visible: true,
                focused: true,
                scale_factor: 1.,
            },
        ),
        (
            2,
            PeltPlacement {
                content: 1,
                window: 1,
                rect: WorkspaceRect::new(0., 0., 400., 200.),
                visible: true,
                focused: false,
                scale_factor: 1.,
            },
        ),
    ]
}
fn pool() -> (PeltControllerPool<u8, u8, String>, Arc<Mutex<State>>) {
    let state = Arc::new(Mutex::new(State {
        value: "seed".into(),
        ..Default::default()
    }));
    let mut pool = PeltControllerPool::new();
    pool.insert(1, content(&state));
    pool.set_placements(placements());
    (pool, state)
}
fn request(
    snapshot: &PeltPlacementA11y<u8, u8>,
    action: DocumentA11yAction,
) -> DocumentA11yActionRequest {
    DocumentA11yActionRequest {
        revision: snapshot.projection().revision(),
        target: DocumentA11yNodeId::new(1),
        action,
        data: None,
    }
}
#[test]
fn addressed_restore_keeps_both_viewports_actionable_without_relaxing_captured_revision() {
    let (mut pool, state) = pool();
    let narrow = pool.accessibility_projection(&1).unwrap();
    let wide = pool.accessibility_projection(&2).unwrap();
    assert_ne!(narrow.projection().revision(), wide.projection().revision());
    assert!(
        pool.dispatch_accessibility_action(&narrow, &request(&narrow, DocumentA11yAction::Focus))
            .handled
    );
    assert_eq!(state.lock().unwrap().width, 200);
    assert_eq!(state.lock().unwrap().clicks, 0, "Focus is not Click");
    let narrow = pool.accessibility_projection(&1).unwrap();
    let wide = pool.accessibility_projection(&2).unwrap();
    let _ = pool.frame(&1);
    let mut set = request(&wide, DocumentA11yAction::SetValue);
    set.data = Some(DocumentA11yActionData::Value("typed".into()));
    assert!(pool.dispatch_accessibility_action(&wide, &set).handled);
    assert_eq!(state.lock().unwrap().width, 400);
    assert_eq!(state.lock().unwrap().value, "typed");
    assert!(
        !pool
            .dispatch_accessibility_action(&narrow, &request(&narrow, DocumentA11yAction::Focus))
            .handled,
        "a source mutation retires other published appearances"
    );
}
#[test]
fn neutral_click_restores_target_geometry_and_emits_one_host_navigation() {
    let (mut pool, state) = pool();
    let narrow = pool.accessibility_projection(&1).unwrap();
    let _ = pool.accessibility_projection(&2).unwrap();
    let effect =
        pool.dispatch_accessibility_action(&narrow, &request(&narrow, DocumentA11yAction::Click));
    assert_eq!(
        effect.navigation.unwrap().request.address,
        "https://fixture.test/200"
    );
    assert_eq!(state.lock().unwrap().clicks, 1);
}
#[test]
fn full_snapshot_equality_refuses_frame_side_effects() {
    let (mut pool, state) = pool();
    let narrow = pool.accessibility_projection(&1).unwrap();
    let _ = pool.accessibility_projection(&2).unwrap();
    state.lock().unwrap().mutate_on_resize = true;
    assert!(
        !pool
            .dispatch_accessibility_action(&narrow, &request(&narrow, DocumentA11yAction::Focus))
            .handled
    );
    assert!(!state.lock().unwrap().focused);
}
#[test]
fn unexpected_source_revision_even_with_restored_value_retires_every_snapshot() {
    let (mut pool, state) = pool();
    let narrow = pool.accessibility_projection(&1).unwrap();
    let wide = pool.accessibility_projection(&2).unwrap();
    state.lock().unwrap().revision += 2;
    assert!(
        !pool
            .dispatch_accessibility_action(&narrow, &request(&narrow, DocumentA11yAction::Focus))
            .handled
    );
    assert!(
        !pool
            .dispatch_accessibility_action(&wide, &request(&wide, DocumentA11yAction::Focus))
            .handled
    );
    let current = pool.accessibility_projection(&1).unwrap();
    assert!(
        pool.dispatch_accessibility_action(&current, &request(&current, DocumentA11yAction::Focus))
            .handled
    );
}
#[test]
fn old_request_revision_hidden_moved_replaced_and_reopened_placements_refuse_delivery() {
    let (mut pool, state) = pool();
    let old = pool.accessibility_projection(&1).unwrap();
    let mut wrong = request(&old, DocumentA11yAction::Focus);
    wrong.revision += 1;
    assert!(!pool.dispatch_accessibility_action(&old, &wrong).handled);
    let mut moved = placements();
    moved[0].1.rect.x = 10.;
    pool.set_placements(moved);
    assert!(
        !pool
            .dispatch_accessibility_action(&old, &request(&old, DocumentA11yAction::Focus))
            .handled
    );
    let moved = pool.accessibility_projection(&1).unwrap();
    pool.set_placements([]);
    assert!(
        !pool
            .dispatch_accessibility_action(&moved, &request(&moved, DocumentA11yAction::Focus))
            .handled
    );
    pool.set_placements(placements());
    assert!(
        !pool
            .dispatch_accessibility_action(&old, &request(&old, DocumentA11yAction::Focus))
            .handled
    );
    let replaced = pool.accessibility_projection(&1).unwrap();
    pool.insert(1, content(&state));
    pool.set_placements(placements());
    assert!(
        !pool
            .dispatch_accessibility_action(
                &replaced,
                &request(&replaced, DocumentA11yAction::Focus)
            )
            .handled
    );
    assert!(!state.lock().unwrap().focused);
}

#[test]
fn unexpected_focus_and_field_value_changes_are_not_layout_restoration() {
    for focus in [false, true] {
        let (mut pool, state) = pool();
        let first = pool.accessibility_projection(&1).unwrap();
        let _ = pool.accessibility_projection(&2).unwrap();
        {
            let mut state = state.lock().unwrap();
            if focus {
                state.focused = true;
            } else {
                state.value = "async field value".into();
            }
            state.revision += 1;
        }
        let mut action = request(&first, DocumentA11yAction::SetValue);
        action.data = Some(DocumentA11yActionData::Value("stale write".into()));
        assert!(!pool.dispatch_accessibility_action(&first, &action).handled);
        assert_ne!(state.lock().unwrap().value, "stale write");
    }
}
