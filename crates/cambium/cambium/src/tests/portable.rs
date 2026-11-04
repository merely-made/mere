/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::{DomMutation, LayoutDom, LayoutDomMut};
use meristem::{MessageCtx, MessageResult, Mut, View, ViewMarker};

use crate::{
    El, GenetAppRunner, GenetCtx, GenetElement, OnClick, PointerClick, PortableKeyed, el, on_click,
};

#[derive(Default)]
struct Stats {
    builds: Vec<&'static str>,
    teardowns: Vec<&'static str>,
}

struct MoveDemo {
    left: Vec<&'static str>,
    right: Vec<&'static str>,
    clicks: usize,
    stats: Rc<RefCell<Stats>>,
}

/// A count-the-click handler as a plain fn so the inner view type stays
/// nameable (a capturing closure would make `Tile::ViewState` unnameable).
fn note_click(demo: &mut MoveDemo, _click: PointerClick) {
    demo.clicks += 1;
}

type Inner = OnClick<El<String, MoveDemo, ()>, MoveDemo, (), fn(&mut MoveDemo, PointerClick)>;

/// A portable tile: a clickable `<p>` labeled with its id, delegating to a
/// reconstructed inner view (the `TaggedText` pattern) so it can be `Clone`.
#[derive(Clone)]
struct Tile {
    id: &'static str,
    stats: Rc<RefCell<Stats>>,
}

impl Tile {
    fn inner(&self) -> Inner {
        on_click(
            el::<_, MoveDemo, ()>("p", self.id.to_string()),
            note_click as fn(&mut MoveDemo, PointerClick),
        )
    }
}

impl ViewMarker for Tile {}

impl View<MoveDemo, (), GenetCtx> for Tile {
    type Element = GenetElement;
    type ViewState = <Inner as View<MoveDemo, (), GenetCtx>>::ViewState;

    fn build(
        &self,
        ctx: &mut GenetCtx,
        app_state: &mut MoveDemo,
    ) -> (Self::Element, Self::ViewState) {
        self.stats.borrow_mut().builds.push(self.id);
        self.inner().build(ctx, app_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut MoveDemo,
    ) {
        self.inner()
            .rebuild(&prev.inner(), view_state, ctx, element, app_state);
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
    ) {
        self.stats.borrow_mut().teardowns.push(self.id);
        self.inner().teardown(view_state, ctx, element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut MoveDemo,
    ) -> MessageResult<()> {
        self.inner()
            .message(view_state, message, element, app_state)
    }
}

fn pane(ids: &[&'static str], stats: &Rc<RefCell<Stats>>) -> PortableKeyed<&'static str, Tile> {
    ids.iter()
        .copied()
        .map(|id| {
            (
                id,
                Tile {
                    id,
                    stats: stats.clone(),
                },
            )
        })
        .collect()
}

fn move_logic(
    demo: &MoveDemo,
) -> impl View<MoveDemo, (), GenetCtx, Element = GenetElement> + use<> {
    el::<_, MoveDemo, ()>(
        "div",
        (
            el::<_, MoveDemo, ()>("section", pane(&demo.left, &demo.stats)),
            el::<_, MoveDemo, ()>("section", pane(&demo.right, &demo.stats)),
        ),
    )
}

/// The two `<section>` panes under the runner root.
fn sections(dom: &ScriptedDom, root: NodeId) -> (NodeId, NodeId) {
    let mut kids = dom.dom_children(root);
    let left = kids.next().expect("left section");
    let right = kids.next().expect("right section");
    (left, right)
}

fn child_texts(dom: &ScriptedDom, node: NodeId) -> Vec<String> {
    dom.dom_children(node)
        .filter_map(|p| {
            dom.dom_children(p)
                .find_map(|t| dom.text(t).map(str::to_string))
        })
        .collect()
}

fn drain(dom: &Rc<RefCell<ScriptedDom>>) -> Vec<DomMutation<NodeId>> {
    let mut out = Vec::new();
    dom.borrow_mut().drain_mutations(&mut out);
    out
}

/// Source-before-target: the tile's element, DOM node, view state, and
/// event handler all survive a cross-parent move, and the DOM observes
/// exactly one atomic `Moved` — the tear-out contract. (moveBefore S5.)
#[test]
fn cross_parent_move_preserves_element_state_and_handlers() {
    let stats = Rc::new(RefCell::new(Stats::default()));
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move_logic,
        MoveDemo {
            left: vec!["x"],
            right: vec!["y"],
            clicks: 0,
            stats: stats.clone(),
        },
    );
    stats.borrow_mut().clear_all();
    let (left, right) = sections(&dom.borrow(), runner.root());
    let x_node = dom.borrow().dom_children(left).next().expect("x tile");
    let _ = drain(&dom);

    // Move x from the left pane to the front of the right pane. The left
    // pane (source) rebuilds first in tree order, so it parks x and the
    // right pane adopts it.
    runner.update(|demo| {
        demo.left = vec![];
        demo.right = vec!["x", "y"];
    });

    {
        let dom = dom.borrow();
        assert_eq!(child_texts(&dom, left), Vec::<String>::new());
        assert_eq!(child_texts(&dom, right), vec!["x", "y"]);
        let moved = dom.dom_children(right).next().expect("moved tile");
        assert_eq!(moved, x_node, "the tile keeps its DOM node");
    }
    assert_eq!(
        drain(&dom),
        vec![DomMutation::Moved {
            node: x_node,
            from_parent: left,
            to_parent: right,
        }],
        "one atomic move; nothing removed, nothing inserted"
    );
    {
        let stats = stats.borrow();
        assert!(stats.builds.is_empty(), "no fresh build");
        assert!(stats.teardowns.is_empty(), "no teardown");
    }

    // The handler survived the move AND routes through the new position:
    // without (node, path) reconciliation this would be a stale path.
    runner.dispatch_click(x_node, PointerClick::at((0.0, 0.0)));
    assert_eq!(runner.state().clicks, 1, "the moved tile's click routes");
}

/// Target-before-source: the left pane rebuilds before the right pane has
/// parked anything, so the arrival misses the nursery and builds fresh;
/// the departed original is parked, then drained (real teardown + node
/// removal). Correct, no leak — just no preservation. The ordering caveat
/// on the module docs, exercised.
#[test]
fn target_before_source_falls_back_to_fresh_build_and_drain() {
    let stats = Rc::new(RefCell::new(Stats::default()));
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move_logic,
        MoveDemo {
            left: vec![],
            right: vec!["y"],
            clicks: 0,
            stats: stats.clone(),
        },
    );
    stats.borrow_mut().clear_all();
    let (left, right) = sections(&dom.borrow(), runner.root());
    let y_node = dom.borrow().dom_children(right).next().expect("y tile");
    let _ = drain(&dom);

    // Move y right → left: the target (left) rebuilds first.
    runner.update(|demo| {
        demo.left = vec!["y"];
        demo.right = vec![];
    });

    {
        let dom = dom.borrow();
        assert_eq!(child_texts(&dom, left), vec!["y"]);
        assert_eq!(child_texts(&dom, right), Vec::<String>::new());
        let fresh = dom.dom_children(left).next().expect("fresh tile");
        assert_ne!(fresh, y_node, "no preservation against tree order");
        assert!(!dom.is_live(y_node), "the departed original is removed");
    }
    let stats = stats.borrow();
    assert_eq!(stats.builds, vec!["y"], "one fresh build");
    assert_eq!(stats.teardowns, vec!["y"], "the parked original drained");
}

/// A key that leaves every list is parked, then drained at the end of the
/// same rebuild: real teardown, node removed — parking never leaks.
#[test]
fn removed_key_parks_then_drains_to_real_teardown() {
    let stats = Rc::new(RefCell::new(Stats::default()));
    let dom = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom.clone(),
        move_logic,
        MoveDemo {
            left: vec!["x"],
            right: vec![],
            clicks: 0,
            stats: stats.clone(),
        },
    );
    stats.borrow_mut().clear_all();
    let (left, _right) = sections(&dom.borrow(), runner.root());
    let x_node = dom.borrow().dom_children(left).next().expect("x tile");
    let _ = drain(&dom);

    runner.update(|demo| demo.left = vec![]);

    {
        let dom = dom.borrow();
        assert_eq!(child_texts(&dom, left), Vec::<String>::new());
        assert!(!dom.is_live(x_node), "drained tile's node is removed");
    }
    let stats = stats.borrow();
    assert!(stats.builds.is_empty());
    assert_eq!(stats.teardowns, vec!["x"], "drain runs the real teardown");
}

impl Stats {
    fn clear_all(&mut self) {
        self.builds.clear();
        self.teardowns.clear();
    }
}
