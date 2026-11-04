/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use layout_dom_api::LayoutDom;
use meristem::{MessageCtx, MessageResult, Mut, View, ViewMarker};

use crate::{GenetAppRunner, GenetCtx, GenetElement, Keyed, el};

#[derive(Default)]
struct KeyedStats {
    builds: Vec<&'static str>,
    rebuilds: Vec<&'static str>,
    teardowns: Vec<&'static str>,
}

impl KeyedStats {
    fn clear(&mut self) {
        self.builds.clear();
        self.rebuilds.clear();
        self.teardowns.clear();
    }
}

struct KeyedDemo {
    ids: Vec<&'static str>,
    stats: Rc<RefCell<KeyedStats>>,
}

#[derive(Clone)]
struct TaggedText {
    id: &'static str,
    stats: Rc<RefCell<KeyedStats>>,
}

impl ViewMarker for TaggedText {}

impl View<KeyedDemo, (), GenetCtx> for TaggedText {
    type Element = GenetElement;
    type ViewState = <&'static str as View<KeyedDemo, (), GenetCtx>>::ViewState;

    fn build(
        &self,
        ctx: &mut GenetCtx,
        app_state: &mut KeyedDemo,
    ) -> (Self::Element, Self::ViewState) {
        let _ = app_state;
        self.stats.borrow_mut().builds.push(self.id);
        View::<KeyedDemo, (), GenetCtx>::build(&self.id, ctx, app_state)
    }

    fn rebuild(
        &self,
        prev: &Self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut KeyedDemo,
    ) {
        assert_eq!(
            self.id, prev.id,
            "keyed rebuild paired different logical children"
        );
        self.stats.borrow_mut().rebuilds.push(self.id);
        View::<KeyedDemo, (), GenetCtx>::rebuild(
            &self.id, &prev.id, view_state, ctx, element, app_state,
        );
    }

    fn teardown(
        &self,
        view_state: &mut Self::ViewState,
        ctx: &mut GenetCtx,
        element: Mut<'_, Self::Element>,
    ) {
        self.stats.borrow_mut().teardowns.push(self.id);
        View::<KeyedDemo, (), GenetCtx>::teardown(&self.id, view_state, ctx, element);
    }

    fn message(
        &self,
        view_state: &mut Self::ViewState,
        message: &mut MessageCtx,
        element: Mut<'_, Self::Element>,
        app_state: &mut KeyedDemo,
    ) -> MessageResult<()> {
        View::<KeyedDemo, (), GenetCtx>::message(&self.id, view_state, message, element, app_state)
    }
}

fn keyed_logic(
    demo: &KeyedDemo,
) -> impl View<KeyedDemo, (), GenetCtx, Element = GenetElement> + use<> {
    let children: Keyed<&'static str, TaggedText> = demo
        .ids
        .iter()
        .copied()
        .map(|id| {
            (
                id,
                TaggedText {
                    id,
                    stats: demo.stats.clone(),
                },
            )
        })
        .collect();
    el::<_, KeyedDemo, ()>("div", children)
}

fn child_texts(dom: &ScriptedDom, node: NodeId) -> Vec<String> {
    dom.dom_children(node)
        .filter_map(|child| dom.text(child).map(str::to_string))
        .collect()
}

#[test]
fn keyed_middle_insert_retains_later_child_state() {
    let stats = Rc::new(RefCell::new(KeyedStats::default()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        keyed_logic,
        KeyedDemo {
            ids: vec!["a", "c"],
            stats: stats.clone(),
        },
    );
    stats.borrow_mut().clear();

    runner.update(|demo| demo.ids = vec!["a", "b", "c"]);

    let dom = runner.dom();
    let dom = dom.borrow();
    assert_eq!(child_texts(&dom, runner.root()), vec!["a", "b", "c"]);
    drop(dom);

    let stats = stats.borrow();
    assert_eq!(stats.builds, vec!["b"]);
    assert_eq!(stats.rebuilds, vec!["a", "c"]);
    assert!(stats.teardowns.is_empty());
}

#[test]
fn keyed_middle_delete_retains_later_child_state() {
    let stats = Rc::new(RefCell::new(KeyedStats::default()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        Rc::new(RefCell::new(ScriptedDom::new())),
        keyed_logic,
        KeyedDemo {
            ids: vec!["a", "b", "c"],
            stats: stats.clone(),
        },
    );
    stats.borrow_mut().clear();

    runner.update(|demo| demo.ids = vec!["a", "c"]);

    let dom = runner.dom();
    let dom = dom.borrow();
    assert_eq!(child_texts(&dom, runner.root()), vec!["a", "c"]);
    drop(dom);

    let stats = stats.borrow();
    assert!(stats.builds.is_empty());
    assert_eq!(stats.rebuilds, vec!["a", "c"]);
    assert_eq!(stats.teardowns, vec!["b"]);
}

#[test]
fn keyed_reorder_moves_the_element_without_teardown() {
    use layout_dom_api::{DomMutation, LayoutDomMut};

    let stats = Rc::new(RefCell::new(KeyedStats::default()));
    let dom_handle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::<_, _, _, ()>::new(
        dom_handle.clone(),
        keyed_logic,
        KeyedDemo {
            ids: vec!["a", "b"],
            stats: stats.clone(),
        },
    );
    stats.borrow_mut().clear();
    let before: Vec<NodeId> = {
        let dom = dom_handle.borrow();
        dom.dom_children(runner.root()).collect()
    };
    {
        // Clear the build-phase mutation noise so the reorder's records
        // stand alone.
        let mut drained = Vec::new();
        dom_handle.borrow_mut().drain_mutations(&mut drained);
    }

    runner.update(|demo| demo.ids = vec!["b", "a"]);

    // Both children keep their DOM nodes (same NodeIds, swapped order):
    // a keyed reorder over single-element children is a move, never a
    // teardown + rebuild. (moveBefore plan S5.)
    let dom = runner.dom();
    let dom = dom.borrow();
    assert_eq!(child_texts(&dom, runner.root()), vec!["b", "a"]);
    let after: Vec<NodeId> = dom.dom_children(runner.root()).collect();
    assert_eq!(after, vec![before[1], before[0]], "same nodes, swapped");
    drop(dom);

    // The DOM observed exactly one atomic Moved — no Removed, no Inserted.
    let mut muts = Vec::new();
    dom_handle.borrow_mut().drain_mutations(&mut muts);
    let root = runner.root();
    assert_eq!(
        muts,
        vec![DomMutation::Moved {
            node: before[1],
            from_parent: root,
            to_parent: root,
        }]
    );

    let stats = stats.borrow();
    assert!(stats.builds.is_empty(), "no child rebuilt from scratch");
    assert_eq!(stats.rebuilds, vec!["b", "a"]);
    assert!(stats.teardowns.is_empty(), "no child torn down");
}
