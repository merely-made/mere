// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Accessibility focus follows the runner, independently of hover restyling.

mod activation_tests;
mod outcome_tests;
mod publication_tests;

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use cambium::{AnyView, GenetCtx, GenetElement, button, el};
use layout_dom_api::LayoutDom as _;

use crate::{
    A11yRequest, Accessibility, Host, HostHooks, HostOptions, HostState, HostTree as _, HostWake,
    OwnedLayout, ProducerRegistry, Runner, ScriptedDom, WindowDom,
};

type Child = Box<dyn AnyView<(), (), GenetCtx, GenetElement>>;
type Logic = fn(&()) -> Child;

fn view(_: &()) -> Child {
    Box::new(el(
        "main",
        (
            button("Before", |_: &mut (), _| {}),
            button("After", |_: &mut (), _| {}),
        ),
    ))
}

#[derive(Debug, PartialEq)]
struct FocusSample {
    reported: Option<u64>,
    projected: Vec<u64>,
}

struct Recording(Rc<RefCell<Vec<FocusSample>>>);

impl Accessibility for Recording {
    fn adapter_state(&self) -> crate::A11yAdapterState {
        crate::A11yAdapterState::Unavailable
    }

    fn sync(
        &mut self,
        _: &crate::A11yOwner,
        dom: &WindowDom<'_>,
        layout: &OwnedLayout,
        _: &sprigging::LeafRegistry<u64>,
        _: &ProducerRegistry,
        focus: Option<u64>,
        _: f64,
    ) -> crate::A11ySync {
        let projection = crate::document_projection(dom, layout, focus);
        self.0.borrow_mut().push(FocusSample {
            reported: focus,
            projected: projection
                .nodes()
                .iter()
                .filter(|node| node.state.focused)
                .map(|node| node.id.get())
                .collect(),
        });
        crate::A11ySync {
            requests: Vec::new(),
            publication: crate::A11yPublication::default(),
        }
    }
}

#[test]
fn accessibility_sync_tracks_focus_changes_and_clear_without_hover() {
    let samples = Rc::new(RefCell::new(Vec::new()));
    let mut state: HostState<(), Logic, Child> = HostState::new();
    state.shared.sheet =
        "main { display: block; } button { display: block; width: 100px; height: 30px; }".into();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        view as Logic,
        (),
    ));
    state.a11y = Some(Box::new(Recording(samples.clone())));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.relayout(320.0, 200.0);
    let buttons = host.s.runner.as_ref().unwrap().focusables();
    assert_eq!(buttons.len(), 2);
    let ids = {
        let runner = host.s.runner.as_ref().unwrap();
        let dom = runner.dom();
        let dom = dom.borrow();
        let window = WindowDom::new(&dom, runner.mount());
        buttons
            .iter()
            .map(|&node| window.opaque_id(node))
            .collect::<Vec<_>>()
    };

    // Prime the restyle cache, then change and clear focus as a scenario or
    // programmatic request can, with no hover event to refresh that cache.
    host.s.runner.as_mut().unwrap().set_focus(Some(buttons[0]));
    host.hover();
    assert_eq!(host.s.last_focus, Some(ids[0]));
    host.sync_a11y();
    host.s.runner.as_mut().unwrap().set_focus(Some(buttons[1]));
    host.sync_a11y();
    assert_eq!(
        host.s.last_focus,
        Some(ids[0]),
        "sync leaves restyling pending"
    );
    host.s.runner.as_mut().unwrap().set_focus(None);
    host.sync_a11y();
    assert_eq!(
        host.s.last_focus,
        Some(ids[0]),
        "clear also leaves the cache alone"
    );

    assert_eq!(
        *samples.borrow(),
        vec![
            FocusSample {
                reported: Some(ids[0]),
                projected: vec![ids[0]]
            },
            FocusSample {
                reported: Some(ids[1]),
                projected: vec![ids[1]]
            },
            FocusSample {
                reported: None,
                projected: Vec::new()
            },
        ]
    );
}

#[test]
fn accessibility_bounds_follow_nested_and_document_scroll() {
    use layout_dom_api::{LayoutDomMut as _, LocalName, Namespace};

    let mut dom = ScriptedDom::new();
    let root = dom.document();
    dom.set_inner_html(root,
        "<main id=\"scroll\" style=\"display:block;width:280px;height:100px;overflow:auto\"><div style=\"height:50px\"></div><button aria-label=\"Nested\" style=\"display:block;width:80px;height:30px\">Nested</button><div style=\"height:200px\"></div></main><button aria-label=\"Document\" style=\"display:block;width:80px;height:30px\">Document</button><div style=\"width:600px;height:500px\"></div>");
    let find = |attr: &str, value: &str| {
        let mut pending = vec![root];
        while let Some(node) = pending.pop() {
            if dom.attribute(node, &Namespace::default(), &LocalName::from(attr)) == Some(value) {
                return node;
            }
            pending.extend(dom.dom_children(node));
        }
        panic!("missing fixture node {attr}={value}");
    };
    let scroll = find("id", "scroll");
    let nested = find("aria-label", "Nested");
    let document = find("aria-label", "Document");
    let window = WindowDom::document(&dom);
    let mut layout = OwnedLayout::new(&window, &[""], 320.0, 200.0, &[], &Default::default());
    let before = crate::document_projection(&window, &layout, Some(dom.opaque_id(nested)));
    layout.set_element_scroll(&window, [(scroll, (0.0, 24.0))].into());
    layout.set_viewport_scroll((13.0, 29.0));
    assert_eq!(layout.element_scroll()[&scroll], (0.0, 24.0));
    assert_eq!(layout.viewport_scroll(), (13.0, 29.0));
    let after = crate::document_projection(&window, &layout, Some(dom.opaque_id(nested)));
    for (target, expected_y_shift) in [(nested, 53.0), (document, 29.0)] {
        let id = dom.opaque_id(target);
        let prior = before
            .nodes()
            .iter()
            .find(|node| node.id.get() == id)
            .unwrap();
        let current = after
            .nodes()
            .iter()
            .find(|node| node.id.get() == id)
            .unwrap();
        let old = prior.bounds.unwrap();
        let bounds = current.bounds.unwrap();
        assert_eq!(bounds.x, old.x - 13.0);
        assert_eq!(bounds.y, old.y - expected_y_shift);
        let painted = layout.painted_rect(&window, target).unwrap();
        assert_eq!((bounds.x, bounds.y, bounds.width, bounds.height), painted);
        assert_eq!(current.name, prior.name);
        assert_eq!(current.role, prior.role);
        assert_eq!(current.actions, prior.actions);
        assert_eq!(current.state.focused, prior.state.focused);
    }
}

// The first activation disables the control. Admission of the second request
// must use that new DOM, even though layout and platform maps are still retained.
type CounterChild = Box<dyn AnyView<u32, (), GenetCtx, GenetElement>>;
type CounterLogic = fn(&u32) -> CounterChild;
fn counter_view(count: &u32) -> CounterChild {
    Box::new(el(
        "main",
        (
            button("Once", |count: &mut u32, _| {
                *count += 1;
            })
            .attr("aria-disabled", (*count > 0).to_string()),
            el("div", "Plain").attr("id", "plain"),
            cambium::on_value(
                cambium::custom_leaf::<u32, ()>(7, 80, 30),
                |count: &mut u32, event| {
                    *count = event.value as u32;
                },
            ),
        ),
    ))
}
fn counter_host() -> Host<u32, CounterLogic, CounterChild> {
    let mut state: HostState<u32, CounterLogic, CounterChild> = HostState::new();
    state.shared.sheet =
        "main, button, div, custom-leaf { display:block; width:100px; height:30px; }".into();
    state.runner = Some(Runner::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        counter_view as CounterLogic,
        0,
    ));
    let wake = HostWake::new(state.wake_pending.clone(), Arc::new(|| {}));
    let mut host = Host::new(
        HostOptions::default(),
        None,
        HostHooks::inert(),
        state,
        wake,
    );
    host.relayout(320.0, 240.0);
    host
}

#[test]
fn readonly_tests_app_context_observes_the_contributed_catalogue_without_activation() {
    let mut host = counter_host();
    host.s.shared.leaves.insert(
        7,
        Box::new(sprigging::Knob::new(sprigging::Size {
            width: 80.0,
            height: 30.0,
        })),
    );
    let reads = Rc::new(RefCell::new(Vec::new()));
    let output = reads.clone();
    host.hooks.after_frame = Box::new(move |ctx| {
        let ctx = &*ctx;
        let observed = ctx.semantic_projection().unwrap();
        assert!(
            observed
                .tree
                .nodes
                .iter()
                .any(|(_, node)| node.role() == accesskit::Role::Slider)
        );
        output.borrow_mut().push(observed.tree);
        assert_eq!(*ctx.runner.state(), 0);
    });
    host.with_ctx(crate::Hook::AfterFrame);
    host.with_ctx(crate::Hook::AfterFrame);
    let reads = reads.borrow();
    assert_eq!(reads.len(), 2);
    assert_eq!(reads[0], reads[1]);
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 0);
}

#[test]
fn selector_tests_queued_actions_acknowledge_current_admission_per_request() {
    let mut host = counter_host();
    let replies = Rc::new(RefCell::new(Vec::new()));
    let saved = replies.clone();
    host.hooks.after_frame = Box::new(move |ctx| {
        let p = ctx.semantic_projection().unwrap();
        let id = p
            .tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Once"))
            .unwrap()
            .0;
        let node = p.dom_node(id.0).unwrap();
        for _ in 0..2 {
            let receipt = ctx.queue_a11y(crate::A11yRequest::node(crate::A11yAction::Click, node));
            assert_eq!(receipt.result(), None, "queueing is not routing acceptance");
            saved.borrow_mut().push(receipt);
        }
        assert_eq!(*ctx.runner.state(), 0);
    });
    host.with_ctx(crate::Hook::AfterFrame);
    assert_eq!(*host.s.runner.as_ref().unwrap().state(), 1);
    let replies = replies.borrow();
    assert_eq!(replies[0].result(), Some(crate::A11yDispatch::Dispatched));
    assert_eq!(
        replies[1].result(),
        Some(crate::A11yDispatch::Rejected(
            crate::A11yRejection::Disabled
        ))
    );
}

#[test]
fn selector_tests_post_dispatch_semantic_requests_wait_for_the_next_batch() {
    let mut host = counter_host();
    let replies = Rc::new(RefCell::new(Vec::new()));
    let saved = replies.clone();
    host.hooks.after_frame = Box::new(move |ctx| {
        let p = ctx.semantic_projection().unwrap();
        let id = p
            .tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Once"))
            .unwrap()
            .0;
        saved
            .borrow_mut()
            .push(ctx.queue_a11y(crate::A11yRequest::node(
                crate::A11yAction::Focus,
                p.dom_node(id.0).unwrap(),
            )));
    });
    let saved = replies.clone();
    let once = Rc::new(std::cell::Cell::new(false));
    host.hooks.after_dispatch = Box::new(move |ctx| {
        if once.replace(true) {
            return;
        }
        let p = ctx.semantic_projection().unwrap();
        let id = p
            .tree
            .nodes
            .iter()
            .find(|(_, n)| n.label() == Some("Once"))
            .unwrap()
            .0;
        saved
            .borrow_mut()
            .push(ctx.queue_a11y(crate::A11yRequest::node(
                crate::A11yAction::Focus,
                p.dom_node(id.0).unwrap(),
            )));
    });
    host.with_ctx(crate::Hook::AfterFrame);
    assert_eq!(replies.borrow().len(), 2);
    assert_eq!(
        replies.borrow()[0].result(),
        Some(crate::A11yDispatch::Dispatched)
    );
    assert_eq!(
        replies.borrow()[1].result(),
        None,
        "post-dispatch hooks cannot recursively drain semantic actions"
    );
    host.drain_pointer();
    assert_eq!(
        replies.borrow()[1].result(),
        Some(crate::A11yDispatch::Dispatched)
    );
}

// Leave the actual hook queue pending, as when an after-dispatch hook enqueues
// work for the next event. Mutations below happen before the real host drain.
fn defer_lifetime_request(
    host: &mut Host<u32, CounterLogic, CounterChild>,
    request: A11yRequest,
) -> crate::A11yReceipt {
    let saved = Rc::new(RefCell::new(None));
    let reply = saved.clone();
    if host.s.a11y_owner.get() {
        host.hooks.after_frame = Box::new(|ctx| {
            let node = ctx.runner.focusables()[0];
            ctx.queue_a11y(A11yRequest::node(crate::A11yAction::Focus, node));
        });
        host.hooks.after_dispatch = Box::new(move |ctx| {
            if reply.borrow().is_none() {
                *reply.borrow_mut() = Some(ctx.queue_a11y(request.clone()));
            }
        });
    } else {
        host.hooks.after_frame = Box::new(move |ctx| {
            *reply.borrow_mut() = Some(ctx.queue_a11y(request.clone()));
        });
    }
    host.with_ctx(crate::Hook::AfterFrame);
    host.hooks.after_frame = Box::new(|_| {});
    host.hooks.after_dispatch = Box::new(|_| {});
    saved.borrow_mut().take().unwrap()
}

fn lifetime_click(host: &Host<u32, CounterLogic, CounterChild>) -> A11yRequest {
    A11yRequest::node(
        crate::A11yAction::Click,
        host.s.runner.as_ref().unwrap().focusables()[0],
    )
}

struct LifetimeBoard {
    calls: Rc<std::cell::Cell<u32>>,
    revision: Rc<std::cell::Cell<u32>>,
}
impl crate::TextureProducer for LifetimeBoard {
    fn render(&mut self, _: &crate::ProducerContext<'_>) -> Option<crate::ProducedTexture> {
        None
    }
    fn semantics(&self) -> Option<crate::ProducerSemantics> {
        Some(crate::ProducerSemantics {
            children: vec![crate::ProducerNode {
                key: 40,
                role: crate::ProducerRole::ListItem,
                name: format!("Card revision {}", self.revision.get()),
                rect: [0.0, 0.0, 10.0, 10.0],
                actions: vec![crate::ProducerAction {
                    id: "pin".into(),
                    label: "Pin".into(),
                    description: "Keep card".into(),
                }],
            }],
            ..Default::default()
        })
    }
    fn act(&mut self, key: u64, id: &str) -> bool {
        assert_eq!((key, id), (40, "pin"));
        self.calls.set(self.calls.get() + 1);
        true
    }
}
fn lifetime_produced() -> A11yRequest {
    A11yRequest::produced(
        crate::A11yAction::Click,
        crate::ProducedAction {
            slot: 7,
            key: 40,
            id: "pin".into(),
        },
    )
}

#[path = "accessibility_tests/admission.rs"]
mod admission;

#[path = "accessibility_tests/lifetimes.rs"]
mod lifetimes;
