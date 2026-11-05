// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! CPU admission controls; the mock adapter never installs a physical AT tree.

use super::*;
use crate::{
    A11yAdapterState, A11yOwner, A11yPublication, A11yPublicationOutcome, A11yPublicationSkip,
    A11ySync, Accessibility, HostHooks, HostOptions, HostState, HostTree as _, HostWake,
    HostWindow, LeafRegistry, NodeId, OwnedLayout, ProducerRegistry, Runner, ScriptedDom,
    WindowDom,
};
use cambium::{AnyView, CaretPosition, CaretSelection, GenetCtx, GenetElement, TextCommand, el};
use layout_dom_api::LayoutDom as _;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Arc,
};

mod wake;

struct App {
    left: u32,
    text: cambium::TextInput,
}
type Child = Box<dyn AnyView<App, (), GenetCtx, GenetElement>>;
type Logic = fn(&App) -> Child;
type TestHost = Host<App, Logic, Child>;

fn root(app: &App) -> Child {
    Box::new(el("main", (
        el("div", app.text.text().to_owned()).attr("id", "field").attr("role", "textbox").attr("aria-label", "Field").attr("style", format!(
            "position:absolute;left:{}px;top:30px;width:180px;height:40px;font-size:16px;line-height:24px;", app.left)),
        el("div", "").attr("style", "height:800px;"),
    )))
}

struct Window {
    size: Rc<Cell<(u32, u32)>>,
    ime: Rc<Cell<usize>>,
}
impl HostWindow for Window {
    fn request_redraw(&self) {}
    fn inner_size(&self) -> (u32, u32) {
        self.size.get()
    }
    fn scale_factor(&self) -> f64 {
        2.0
    }
    fn set_ime_allowed(&self, _: bool) {}
    fn set_ime_cursor_area(&self, _: f64, _: f64, _: f64, _: f64) {
        self.ime.set(self.ime.get() + 1);
    }
}

#[derive(Default)]
struct Observations {
    bootstraps: usize,
    installed: bool,
    projections: Vec<(f64, f64)>,
}
struct Adapter(Rc<RefCell<Observations>>);
impl Accessibility for Adapter {
    fn prepare_native_window(&mut self) {
        let mut seen = self.0.borrow_mut();
        if !seen.installed {
            seen.bootstraps += 1;
            seen.installed = true;
        }
    }
    fn adapter_state(&self) -> A11yAdapterState {
        if self.0.borrow().installed {
            A11yAdapterState::Installed
        } else {
            A11yAdapterState::Detached
        }
    }
    fn sync(
        &mut self,
        owner: &A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        leaves: &LeafRegistry<u64>,
        producers: &ProducerRegistry,
        focus: Option<u64>,
        scale: f64,
    ) -> A11ySync {
        let projected = crate::project_semantics(dom, layout, leaves, producers, focus);
        projected
            .publication_bindings(owner, dom, leaves, producers)
            .unwrap();
        let field = field(dom);
        let bounds = projected
            .tree
            .nodes
            .iter()
            .find(|(id, _)| id.0 == field.raw())
            .and_then(|(_, node)| node.bounds())
            .expect("real projected field geometry");
        self.0.borrow_mut().projections.push((scale, bounds.x0));
        A11ySync {
            requests: Vec::new(),
            publication: A11yPublication {
                adapter: A11yAdapterState::Installed,
                outcome: A11yPublicationOutcome::Submitted {
                    nodes: projected.tree.nodes.len(),
                    contribution_issues: 0,
                },
            },
        }
    }
}

fn field(dom: &WindowDom<'_>) -> NodeId {
    crate::owned_layout::flat_nodes(dom)
        .find(|node| dom.attribute(*node, &Default::default(), &"id".into()) == Some("field"))
        .unwrap()
}

struct Fixture {
    host: TestHost,
    size: Rc<Cell<(u32, u32)>>,
    observations: Rc<RefCell<Observations>>,
    ime: Rc<Cell<usize>>,
}
fn fixture() -> Fixture {
    let size = Rc::new(Cell::new((640, 480)));
    let ime = Rc::new(Cell::new(0));
    let observations = Rc::new(RefCell::new(Observations::default()));
    let mut text = cambium::TextInput::new("retained words");
    text.apply(TextCommand::SetSelection(CaretSelection {
        anchor: CaretPosition::default(),
        focus: CaretPosition {
            byte: 4,
            ..Default::default()
        },
    }));
    let mut state: HostState<App, Logic, Child> = HostState::new();
    state.window = Some(Box::new(Window {
        size: size.clone(),
        ime: ime.clone(),
    }));
    state.a11y = Some(Box::new(Adapter(observations.clone())));
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        root as Logic,
        App { left: 20, text },
    ));
    let mut hooks: HostHooks<App, Logic, Child> = HostHooks::inert();
    hooks.focused_text = Box::new(|runner| {
        let node = runner.focus()?;
        Some(crate::FocusedTextSlot {
            node,
            get: Box::new(|app: &App| &app.text),
            get_mut: Box::new(|app: &mut App| &mut app.text),
        })
    });
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(HostOptions::default(), None, hooks, state, wake);
    relayout(&mut host);
    let node = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        field(&WindowDom::new(&dom, runner.mount()))
    };
    host.s.runner.as_mut().unwrap().set_focus(Some(node));
    host.s.native_receiving = true;
    Fixture {
        host,
        size,
        observations,
        ime,
    }
}

fn relayout(host: &mut TestHost) {
    let (width, height) = host.logical_size();
    host.relayout(width, height);
}

// Model accepted host fields directly; no GPU queue or physical presentation
// is claimed by these controls. Native GPU tests own the publication path.
fn accept(host: &mut TestHost) -> Rc<crate::owned_layout::PresentedInput> {
    let input = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        Rc::new(
            host.s
                .layout
                .as_ref()
                .unwrap()
                .presented_input(&WindowDom::new(&dom, runner.mount())),
        )
    };
    let (width, height) = host.s.window.as_ref().unwrap().inner_size();
    host.s.displayed_input = Some(input.clone());
    host.s.displayed_presentation = Some(crate::PresentedFrame {
        host: host.s.presentation_host,
        sequence: host.s.presentation_sequence + 1,
        width,
        height,
        layout_scale: host.layout_scale() as f32,
        context_generation: ContextGeneration(0),
    });
    host.s.native_dirty = false;
    input
}

fn assert_pending(host: &mut TestHost) {
    assert!(host.sync_a11y().is_empty());
    assert_eq!(
        host.a11y_publication().publication.outcome,
        A11yPublicationOutcome::NotAttempted {
            reason: A11yPublicationSkip::PendingPresentation
        }
    );
}
fn assert_projected(host: &mut TestHost) {
    assert!(host.sync_a11y().is_empty());
    assert!(matches!(
        host.a11y_publication().publication.outcome,
        A11yPublicationOutcome::Submitted { .. }
    ));
}

#[test]
fn physical_input_uses_displayed_scale_until_the_next_snapshot() {
    let mut f = fixture();
    assert_eq!(f.host.layout_scale(), 2.0);
    accept(&mut f.host);
    assert_eq!(f.host.input_scale(), 2.0);
    assert_eq!(f.host.layout_point(80.0, 40.0), (40.0, 20.0));
    f.host.set_ui_zoom(2.0);
    assert_eq!(f.host.layout_scale(), 4.0);
    assert_eq!(f.host.input_scale(), 2.0);
    assert_eq!(f.host.layout_point(80.0, 40.0), (40.0, 20.0));
    relayout(&mut f.host);
    accept(&mut f.host);
    assert_eq!(f.host.input_scale(), 4.0);
    assert_eq!(f.host.layout_point(80.0, 40.0), (20.0, 10.0));
}

#[test]
fn accessibility_bootstraps_then_requires_accepted_dom_layout_scale_and_extent() {
    let mut f = fixture();
    assert_pending(&mut f.host);
    assert_eq!(f.observations.borrow().bootstraps, 1);
    assert!(f.observations.borrow().projections.is_empty());
    accept(&mut f.host);
    assert_projected(&mut f.host);
    assert_eq!(f.observations.borrow().projections, vec![(2.0, 20.0)]);
    f.host
        .s
        .runner
        .as_mut()
        .unwrap()
        .update(|app| app.left = 120);
    assert_pending(&mut f.host); // DOM changed before the current layout.
    relayout(&mut f.host);
    assert_pending(&mut f.host); // A future layout is still not a publication.
    assert_eq!(f.observations.borrow().projections.len(), 1);
    accept(&mut f.host);
    assert_projected(&mut f.host);
    assert_eq!(
        f.observations.borrow().projections.last(),
        Some(&(2.0, 120.0))
    );
    f.host.set_ui_zoom(2.0);
    assert_pending(&mut f.host); // Same DOM/layout, different desired scale.
    relayout(&mut f.host);
    accept(&mut f.host);
    assert_projected(&mut f.host);
    assert_eq!(f.observations.borrow().projections.last().unwrap().0, 4.0);
    f.size.set((800, 480));
    assert_pending(&mut f.host); // Same accepted source, different client extent.
    assert_eq!(f.observations.borrow().projections.len(), 3);
    relayout(&mut f.host);
    accept(&mut f.host);
    assert_projected(&mut f.host);
    assert_eq!(f.observations.borrow().projections.len(), 4);
    assert_eq!(f.observations.borrow().bootstraps, 1);
}

#[test]
fn layout_only_scroll_invalidates_coalescing_at_and_caret_proofs_until_acceptance() {
    let mut f = fixture();
    let prepared = accept(&mut f.host);
    assert!(f.host.prepared_input_is_current(&prepared));
    assert!(f.host.input_layout_is_displayed());
    assert_projected(&mut f.host);
    assert!(f.host.focused_caret_rect().is_some());
    assert!(!f.host.focused_selection_rects().is_empty());
    f.host.sync_ime_area();
    assert_eq!(f.ime.get(), 1);
    f.host
        .s
        .layout
        .as_mut()
        .unwrap()
        .set_viewport_scroll((0.0, 12.0));
    let runner = f.host.s.runner.as_ref().unwrap();
    let dom = runner.dom();
    let dom = dom.borrow();
    assert!(
        prepared.matches_dom(&WindowDom::new(&dom, runner.mount())),
        "no DOM mutation"
    );
    drop(dom);
    assert!(!f.host.s.native_dirty, "no after-dispatch/model wake flag");
    assert!(
        !f.host.prepared_input_is_current(&prepared),
        "production coalescing predicate catches geometry only change"
    );
    assert!(!f.host.input_layout_is_displayed());
    assert_pending(&mut f.host);
    assert_eq!(f.observations.borrow().projections.len(), 1);
    assert!(f.host.focused_caret_rect().is_none());
    assert!(f.host.focused_selection_rects().is_empty());
    f.host.sync_ime_area();
    assert_eq!(
        f.ime.get(),
        1,
        "pending layout must not move the platform IME"
    );
    let accepted = accept(&mut f.host);
    assert!(f.host.prepared_input_is_current(&accepted));
    assert!(f.host.input_layout_is_displayed());
    assert_projected(&mut f.host);
    f.host.sync_ime_area();
    assert_eq!(f.ime.get(), 2);
}

#[test]
fn model_dirty_and_recreated_layout_both_refuse_the_prepared_picture_proof() {
    let mut f = fixture();
    let prepared = accept(&mut f.host);
    f.host.s.native_dirty = true;
    assert!(!f.host.prepared_input_is_current(&prepared));
    f.host.s.native_dirty = false;
    assert!(f.host.prepared_input_is_current(&prepared));
    f.host.s.layout = None;
    relayout(&mut f.host);
    assert!(
        !f.host.prepared_input_is_current(&prepared),
        "a new session can start at the old generation number"
    );
    let accepted = accept(&mut f.host);
    assert!(f.host.prepared_input_is_current(&accepted));
}
