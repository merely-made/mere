/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{LayoutDom, NodeKind};
use mere_surface_api::{
    ProviderId, SourceKindId, SurfaceId, SurfaceSourceShape, SurfaceUnavailableReason,
};

use crate::{
    DomHandle, El, FocusExit, GenetAppRunner, Key, KeyEvent, Modifiers, NamedKey, OnClick,
    PointerClick, el, on_click, on_key,
};

use super::*;

struct First {
    count: u32,
    width: f32,
}

struct Second {
    count: u32,
    width: f32,
}

type FirstView = OnClick<El<String, First, ()>, First, (), fn(&mut First, PointerClick)>;

type SecondView = OnClick<El<String, Second, ()>, Second, (), fn(&mut Second, PointerClick)>;

fn first_click(state: &mut First, _event: PointerClick) {
    state.count += 1;
}

fn second_click(state: &mut Second, _event: PointerClick) {
    state.count += 10;
}

fn first_view(state: &First) -> FirstView {
    on_click(
        el::<_, First, ()>(
            "button",
            format!("first:{}:{:.0}", state.count, state.width),
        ),
        first_click as fn(&mut First, PointerClick),
    )
}

fn second_view(state: &Second) -> SecondView {
    on_click(
        el::<_, Second, ()>("div", format!("second:{}:{:.0}", state.count, state.width)),
        second_click as fn(&mut Second, PointerClick),
    )
}

fn descriptor(id: &str) -> SurfaceDescriptor {
    SurfaceDescriptor {
        provider_id: ProviderId::from("example"),
        surface_id: SurfaceId::from(id),
        label: id.to_owned(),
        accepted_source: SurfaceSourceShape::One(SourceKindId::from("example.source")),
    }
}

fn fresh_dom() -> DomHandle {
    Rc::new(RefCell::new(ScriptedDom::new()))
}

fn root_text(dom: &DomHandle, root: NodeId) -> String {
    let dom = dom.borrow();
    let text = dom
        .dom_children(root)
        .find(|node| dom.kind(*node) == NodeKind::Text)
        .expect("surface root has a text child");
    dom.text(text).expect("text child has text").to_owned()
}

#[test]
fn erased_sessions_retain_distinct_runner_state_and_dispatch_independently() {
    let first_dom = fresh_dom();
    let first_runner = GenetAppRunner::new(
        first_dom.clone(),
        first_view,
        First {
            count: 0,
            width: 0.0,
        },
    );
    let first = RunnerSurfaceSession::new(
        descriptor("example.first"),
        first_runner,
        |state: &First| {
            if state.count == 0 {
                SurfaceAvailability::Available
            } else {
                SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Locked)
            }
        },
        |state: &mut First, viewport| state.width = viewport.width,
        |_action: ()| Vec::new(),
    );
    let first_root = first.root();

    let second_dom = fresh_dom();
    let second_runner = GenetAppRunner::new(
        second_dom.clone(),
        second_view,
        Second {
            count: 10,
            width: 0.0,
        },
    );
    let second = RunnerSurfaceSession::new(
        descriptor("example.second"),
        second_runner,
        |_state: &Second| SurfaceAvailability::Available,
        |state: &mut Second, viewport| state.width = viewport.width,
        |_action: ()| Vec::new(),
    );
    let second_root = second.root();

    let mut sessions: Vec<Box<dyn RetainedSurfaceSession>> =
        vec![Box::new(first), Box::new(second)];

    assert_eq!(sessions[0].root(), first_root);
    assert_eq!(sessions[1].root(), second_root);
    assert_eq!(
        sessions[0].dispatch(ResolvedSurfaceEvent::Click {
            target: first_root,
            event: PointerClick::at((0.0, 0.0)),
        }),
        vec![SurfaceEffect::Redraw]
    );
    assert_eq!(root_text(&first_dom, first_root), "first:1:0");
    assert_eq!(root_text(&second_dom, second_root), "second:10:0");
    assert_eq!(
        sessions[0].availability(),
        SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Locked)
    );

    assert_eq!(
        sessions[1].sync_viewport(SurfaceViewport {
            width: 240.0,
            height: 160.0,
            scale_factor: 1.0,
        }),
        vec![SurfaceEffect::Redraw]
    );
    assert_eq!(root_text(&first_dom, first_root), "first:1:0");
    assert_eq!(root_text(&second_dom, second_root), "second:10:240");
    assert_eq!(
        sessions[1].sync_viewport(SurfaceViewport {
            width: 240.0,
            height: 160.0,
            scale_factor: 1.0,
        }),
        Vec::<SurfaceEffect>::new()
    );

    sessions[1].dispatch(ResolvedSurfaceEvent::Click {
        target: second_root,
        event: PointerClick::at((0.0, 0.0)),
    });
    assert_eq!(root_text(&first_dom, first_root), "first:1:0");
    assert_eq!(root_text(&second_dom, second_root), "second:20:240");
}

/// A session whose host turns on focus exits reports Tab past its last
/// focusable as an effect, so the host can move on; with exits off it
/// wraps, as standalone apps expect.
#[test]
fn a_session_reports_focus_leaving_past_its_edge() {
    let dom = fresh_dom();
    let noop: fn(&mut (), KeyEvent) = |_, _| {};
    let runner = GenetAppRunner::<_, _, _, ()>::new(
        dom,
        move |_: &()| {
            el::<_, (), ()>(
                "div",
                (
                    on_key(el::<_, (), ()>("a", ()), noop),
                    on_key(el::<_, (), ()>("b", ()), noop),
                ),
            )
        },
        (),
    );
    let mut session = RunnerSurfaceSession::new(
        descriptor("example.focus"),
        runner,
        |_: &()| SurfaceAvailability::Available,
        |_: &mut (), _| {},
        |_action: ()| Vec::new(),
    );
    let tab = |shift: bool| {
        ResolvedSurfaceEvent::Key(KeyEvent::with_mods(
            Key::Named(NamedKey::Tab),
            Modifiers {
                shift,
                ..Default::default()
            },
        ))
    };

    session.focus_traverse(true);
    session.focus_traverse(true);
    assert_eq!(
        session.dispatch(tab(false)),
        vec![SurfaceEffect::Redraw],
        "exits off: Tab wraps inside the session"
    );
    assert!(session.focus().is_some());

    assert!(session.set_focus_exits(true));
    session.focus_traverse(true);
    assert_eq!(
        session.dispatch(tab(false)),
        vec![
            SurfaceEffect::Redraw,
            SurfaceEffect::FocusExit(FocusExit::Forward)
        ]
    );
    assert_eq!(session.focus(), None);
    assert_eq!(
        session.focus_traverse(false),
        vec![SurfaceEffect::Redraw],
        "the host enters again from the far end"
    );
    assert_eq!(
        session.dispatch(tab(true)),
        vec![SurfaceEffect::Redraw],
        "from the last, Shift+Tab moves back inside"
    );
    assert_eq!(
        session.dispatch(tab(true)),
        vec![
            SurfaceEffect::Redraw,
            SurfaceEffect::FocusExit(FocusExit::Backward)
        ]
    );
}

/// A panic in one guest's dispatch is caught at the boundary: that guest
/// retires as unhealthy and is never called again, and a guest beside it
/// carries on.
#[test]
fn a_contained_session_retires_on_a_panic_and_its_neighbour_carries_on() {
    fn boom(_: &mut First, _: PointerClick) {
        panic!("guest bug");
    }
    fn failing_view(state: &First) -> FirstView {
        on_click(
            el::<_, First, ()>("button", format!("first:{}", state.count)),
            boom as fn(&mut First, PointerClick),
        )
    }

    let failing_runner = GenetAppRunner::new(
        fresh_dom(),
        failing_view,
        First {
            count: 0,
            width: 0.0,
        },
    );
    let mut failing = ContainedSession::new(RunnerSurfaceSession::new(
        descriptor("example.failing"),
        failing_runner,
        |_: &First| SurfaceAvailability::Available,
        |_: &mut First, _| {},
        |_action: ()| Vec::new(),
    ));
    let failing_root = failing.root();

    let second_dom = fresh_dom();
    let second_runner = GenetAppRunner::new(
        second_dom.clone(),
        second_view,
        Second {
            count: 0,
            width: 0.0,
        },
    );
    let mut second = ContainedSession::new(RunnerSurfaceSession::new(
        descriptor("example.second"),
        second_runner,
        |_: &Second| SurfaceAvailability::Available,
        |_: &mut Second, _| {},
        |_action: ()| Vec::new(),
    ));
    let second_root = second.root();

    assert_eq!(failing.availability(), SurfaceAvailability::Available);
    let click = |target| ResolvedSurfaceEvent::Click {
        target,
        event: PointerClick::at((0.0, 0.0)),
    };
    assert_eq!(
        failing.dispatch(click(failing_root)),
        vec![SurfaceEffect::Redraw],
        "the failing call asks for one redraw"
    );
    assert_eq!(
        failing.failure(),
        Some(SessionFailure {
            call: "dispatch",
            message: Some("guest bug".to_owned()),
        })
    );
    assert_eq!(
        failing.availability(),
        SurfaceAvailability::Unavailable(SurfaceUnavailableReason::Unhealthy)
    );
    assert_eq!(failing.focusables(), Vec::<NodeId>::new());
    assert_eq!(failing.focus(), None);
    assert!(!failing.set_focus_exits(true));
    assert_eq!(failing.dispatch(click(failing_root)), Vec::new());
    assert_eq!(failing.root(), failing_root, "the root stays readable");

    second.dispatch(click(second_root));
    assert_eq!(root_text(&second_dom, second_root), "second:10:0");
    assert_eq!(second.failure(), None);
    assert_eq!(second.availability(), SurfaceAvailability::Available);
}

/// A leaf the test can tell apart from another under the same key.
struct Tagged(&'static str);

impl sprigging::Leaf for Tagged {
    fn measure(
        &mut self,
        _known: sprigging::SizeHint,
        _available: sprigging::SizeHint,
    ) -> sprigging::Size {
        sprigging::Size {
            width: 1.0,
            height: 1.0,
        }
    }

    fn paint(&mut self, _cx: &mut sprigging::PaintCx<'_>) {}

    fn paint_dirty(&self) -> bool {
        false
    }
}

/// Two sessions that place a leaf under the same key each keep their own
/// leaf, because each owns its registry. The control: one registry shared
/// between them holds one leaf per key, so one session would paint the
/// other's.
#[test]
fn sessions_keep_their_own_leaves_under_a_shared_key() {
    const KEY: u64 = 0x5753_4642;
    let session = |name: &'static str| {
        let mut leaves = LeafRegistry::new();
        leaves.insert(KEY, Box::new(Tagged(name)));
        let runner = GenetAppRunner::new(
            fresh_dom(),
            first_view,
            First {
                count: 0,
                width: 0.0,
            },
        );
        ContainedSession::new(
            RunnerSurfaceSession::new(
                descriptor(name),
                runner,
                |_: &First| SurfaceAvailability::Available,
                |_: &mut First, _| {},
                |_action: ()| Vec::new(),
            )
            .with_leaves(leaves),
        )
    };
    let mut sessions = [session("example.a"), session("example.b")];
    let painted: Vec<&'static str> = sessions
        .iter_mut()
        .map(|session| {
            session
                .leaves()
                .and_then(|leaves| leaves.get_mut_as::<Tagged>(&KEY))
                .map(|leaf| leaf.0)
                .expect("each session holds its leaf")
        })
        .collect();
    assert_eq!(painted, ["example.a", "example.b"]);

    let mut shared = LeafRegistry::new();
    shared.insert(KEY, Box::new(Tagged("example.a")));
    shared.insert(KEY, Box::new(Tagged("example.b")));
    assert_eq!(
        shared.get_mut_as::<Tagged>(&KEY).map(|leaf| leaf.0),
        Some("example.b"),
        "control: a shared registry lets the second session's leaf replace the first's"
    );
}
