/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::cell::RefCell;
use std::rc::Rc;

use genet_scripted_dom::{NodeId, ScriptedDom};
use workbench::{ContentSource, DocumentRef, DropTarget, Tile, TileBranch};

use super::*;
use crate::{DomHandle, GenetAppRunner, PointerClick};

type TestView = Box<dyn AnyView<State, (), GenetCtx, GenetElement>>;
/// A runner over one of the module's frames, plus the DOM it drew into.
type Harness = (
    DomHandle,
    GenetAppRunner<State, fn(&State) -> TestView, TestView, ()>,
);

struct State {
    tree: TileTree,
    events: Vec<TileEvent>,
}

fn tile(id: u64, title: &str) -> Tile {
    Tile {
        id: TileId(id),
        title: title.to_string(),
        content: ContentSource::Document(DocumentRef(format!("doc:{id}"))),
        accent: None,
    }
}

/// A row split of two stacks; the left one holds two tabs, second active.
fn sample() -> TileTree {
    TileTree::Split {
        axis: SplitAxis::Row,
        children: vec![
            TileBranch {
                fraction: 0.7,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(1, "First"), tile(2, "Second")],
                    active: 1,
                }),
            },
            TileBranch {
                fraction: 0.3,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(3, "Third")],
                    active: 0,
                }),
            },
        ],
    }
}

fn view(state: &State) -> TestView {
    Box::new(frisket(&state.tree, |state: &mut State, event| {
        state.events.push(event)
    }))
}

#[test]
fn hiding_singleton_strips_removes_their_close_and_activation_targets_only() {
    fn adaptive(state: &State) -> TestView {
        Box::new(frisket_with_tab_policy(
            &state.tree,
            None,
            &|_| None,
            crate::TabStripPolicy::Multiple,
            |state: &mut State, event| state.events.push(event),
            fill_by_id,
        ))
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(
        dom.clone(),
        adaptive as fn(&State) -> TestView,
        State {
            tree: sample(),
            events: Vec::new(),
        },
    );
    let root = runner.root();
    let dom = dom.borrow();
    assert_eq!(count_class(&dom, root, "frisket-tabbar"), 1);
    assert_eq!(count_class(&dom, root, "frisket-close"), 2);
    assert!(find_attr(&dom, root, ATTR_TABKEY, "3").is_none());
    assert!(
        find_attr(&dom, root, FRISKET_TILE_ATTR, "3").is_some(),
        "its real content appearance remains mounted"
    );
    assert!(find_class(&dom, root, "slot-child").is_some());
}

/// Only the tile the host marks shows a mark, on its own tab.
#[test]
fn a_host_mark_reaches_the_tab_of_its_tile_only() {
    fn marked(state: &State) -> TestView {
        Box::new(frisket_with_marks(
            &state.tree,
            None,
            &|id| (id == TileId(2)).then_some(TabMark::Attention),
            |state: &mut State, event| state.events.push(event),
            |_: &Tile| Slot::Hole,
        ))
    }
    fn find(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> Option<NodeId> {
        if attr(dom, node, name).as_deref() == Some(value) {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| find(dom, child, name, value))
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let state = State {
        tree: sample(),
        events: Vec::new(),
    };
    let runner = GenetAppRunner::new(dom.clone(), marked as fn(&State) -> TestView, state);
    let dom = dom.borrow();
    let root = runner.root();
    let marks = dom.outer_html(root).matches("data-mark=").count();
    assert_eq!(marks, 1, "one tile marked, one mark drawn");
    let tab = find(&dom, root, "data-tabkey", "2").expect("tile 2's tab");
    assert_eq!(
        attr(&*dom, tab, "aria-description").as_deref(),
        Some("Needs attention")
    );
    let unmarked = find(&dom, root, "data-tabkey", "1").expect("tile 1's tab");
    assert_eq!(attr(&*dom, unmarked, "aria-description"), None);
}

fn harness() -> Harness {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let state = State {
        tree: sample(),
        events: Vec::new(),
    };
    let runner = GenetAppRunner::new(dom.clone(), view as fn(&State) -> TestView, state);
    (dom, runner)
}

fn attr_of<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
    dom.attribute(node, &Namespace::default(), &LocalName::from(name))
}

fn find_attr(dom: &ScriptedDom, root: NodeId, name: &str, value: &str) -> Option<NodeId> {
    if attr_of(dom, root, name) == Some(value) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_attr(dom, child, name, value))
}

fn find_class(dom: &ScriptedDom, root: NodeId, class: &str) -> Option<NodeId> {
    if has_class(dom, root, class) {
        return Some(root);
    }
    dom.dom_children(root)
        .find_map(|child| find_class(dom, child, class))
}

fn count_class(dom: &ScriptedDom, root: NodeId, class: &str) -> usize {
    usize::from(has_class(dom, root, class))
        + dom
            .dom_children(root)
            .map(|child| count_class(dom, child, class))
            .sum::<usize>()
}

/// A slot per tile id: 2 gets a child view, 3 a surface, everything else a
/// hole. A plain `fn` so the fill is `Clone + 'static` without ceremony.
fn fill_by_id(tile: &Tile) -> Slot<State, ()> {
    match tile.id.0 {
        2 => Slot::View(Box::new(
            el::<_, State, ()>("p", "child").attr("class", "slot-child"),
        )),
        3 => Slot::Surface,
        _ => Slot::Hole,
    }
}

fn slot_view(state: &State) -> TestView {
    Box::new(frisket_with(
        &state.tree,
        |state: &mut State, event| state.events.push(event),
        fill_by_id,
    ))
}

fn slot_harness() -> Harness {
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let state = State {
        tree: sample(),
        events: Vec::new(),
    };
    let runner = GenetAppRunner::new(dom.clone(), slot_view as fn(&State) -> TestView, state);
    (dom, runner)
}

#[test]
fn a_view_slot_inlines_its_child_and_the_empty_slots_stay_empty() {
    let (dom, runner) = slot_harness();
    let root = runner.root();
    let dom = dom.borrow();

    // Tile 2 is the left stack's active tab and fills with a view: the
    // child is inside the content element, which says so.
    let view_slot = find_attr(&dom, root, FRISKET_TILE_ATTR, "2").expect("view content");
    assert_eq!(attr_of(&dom, view_slot, "data-slot"), Some("view"));
    assert_eq!(attr_of(&dom, view_slot, "class"), Some("frisket-content"));
    let child = dom.dom_children(view_slot).next().expect("the slot child");
    assert!(has_class(&*dom, child, "slot-child"));
    assert_eq!(dom.dom_children(view_slot).count(), 1);

    // Tile 3's stack asked for a surface: marked, and left empty for the
    // host to composite into.
    let surface = find_attr(&dom, root, FRISKET_TILE_ATTR, "3").expect("surface content");
    assert_eq!(attr_of(&dom, surface, "data-slot"), Some("surface"));
    assert_eq!(dom.dom_children(surface).count(), 0);

    // And the default frame is all holes, also empty.
    let (hole_dom, hole_runner) = harness();
    let hole_dom = hole_dom.borrow();
    let hole =
        find_attr(&hole_dom, hole_runner.root(), FRISKET_TILE_ATTR, "2").expect("hole content");
    assert_eq!(attr_of(&hole_dom, hole, "data-slot"), Some("hole"));
    assert_eq!(hole_dom.dom_children(hole).count(), 0);
}

#[test]
fn a_content_element_reports_its_slot_and_still_names_its_tile() {
    let (dom, runner) = slot_harness();
    let root = runner.root();
    let dom = dom.borrow();

    let view_slot = find_attr(&dom, root, FRISKET_TILE_ATTR, "2").expect("view content");
    let surface = find_attr(&dom, root, FRISKET_TILE_ATTR, "3").expect("surface content");
    assert_eq!(slot_kind(&*dom, view_slot), Some(SlotKind::View));
    assert_eq!(slot_kind(&*dom, surface), Some(SlotKind::Surface));
    // A descendant of a view slot answers for the element it is in.
    let child = dom.dom_children(view_slot).next().expect("the slot child");
    assert_eq!(slot_kind(&*dom, child), Some(SlotKind::View));
    // The tab bar is not in any content element, and neither is the root.
    let bar = find_class(&dom, root, "frisket-tabbar").expect("tab bar");
    assert_eq!(slot_kind(&*dom, bar), None);
    assert_eq!(slot_kind(&*dom, root), None);

    // The slot mark rides beside the tile mark; neither displaces the other.
    assert_eq!(content_target(&*dom, child), Some(TileId(2)));
    assert_eq!(content_target(&*dom, surface), Some(TileId(3)));

    let (hole_dom, hole_runner) = harness();
    let hole_dom = hole_dom.borrow();
    let hole = find_class(&hole_dom, hole_runner.root(), "frisket-content").expect("hole");
    assert_eq!(slot_kind(&*hole_dom, hole), Some(SlotKind::Hole));
}

/// The tear-out contract, through the frame: a tile dragged into another
/// stack takes its content child's DOM node with it, rather than the child
/// being torn down here and rebuilt there. (`PortableKeyed`, moveBefore S5.)
#[test]
fn a_view_slot_keeps_its_element_when_its_tile_moves_to_another_stack() {
    fn every_tile(tile: &Tile) -> Slot<State, ()> {
        Slot::View(Box::new(
            el::<_, State, ()>("p", format!("tile {}", tile.id.0)).attr("class", "slot-child"),
        ))
    }
    fn view(state: &State) -> TestView {
        Box::new(frisket_with(
            &state.tree,
            |state: &mut State, event| state.events.push(event),
            every_tile,
        ))
    }

    // Two stacks, both keeping a spare tab so neither empties and collapses
    // the split out from under the move.
    let tree = TileTree::Split {
        axis: SplitAxis::Row,
        children: vec![
            TileBranch {
                fraction: 0.5,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(1, "First"), tile(4, "Fourth")],
                    active: 0,
                }),
            },
            TileBranch {
                fraction: 0.5,
                tree: TileTree::Stack(TabStack {
                    tabs: vec![tile(3, "Third")],
                    active: 0,
                }),
            },
        ],
    };
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner = GenetAppRunner::new(
        dom.clone(),
        view as fn(&State) -> TestView,
        State {
            tree,
            events: Vec::new(),
        },
    );
    let root = runner.root();
    let (left_content, right_content, moved) = {
        let dom = dom.borrow();
        let left = find_attr(&dom, root, FRISKET_TILE_ATTR, "1").expect("left content");
        let right = find_attr(&dom, root, FRISKET_TILE_ATTR, "3").expect("right content");
        let moved = dom.dom_children(left).next().expect("tile 1's slot child");
        (left, right, moved)
    };

    // Drag tile 1 into the right-hand stack. The left stack rebuilds first
    // in tree order, so it parks the child and the right stack adopts it.
    runner.update(|state| {
        state.tree.apply(&TileEvent::Dragged {
            tile: TileId(1),
            to: DropTarget::Stack {
                stack: TilePath(vec![1]),
                index: 0,
            },
        });
    });

    let dom = dom.borrow();
    assert_eq!(
        attr_of(&dom, right_content, FRISKET_TILE_ATTR),
        Some("1"),
        "the dropped tile is active in the stack it landed in"
    );
    assert_eq!(
        attr_of(&dom, left_content, FRISKET_TILE_ATTR),
        Some("4"),
        "the stack it left falls back to its other tab"
    );
    assert_eq!(
        dom.dom_children(right_content).next(),
        Some(moved),
        "the same node moved; the child was not rebuilt in its new stack"
    );
    assert_eq!(dom.parent(moved), Some(right_content));
    assert_eq!(
        dom.dom_children(left_content).count(),
        1,
        "the vacated stack shows its new active tile's own child"
    );
    assert_ne!(
        dom.dom_children(left_content).next(),
        Some(moved),
        "and that child is a different element"
    );
}

#[test]
fn a_pane_path_round_trips() {
    for path in [vec![], vec![0], vec![1, 0, 2]] {
        let encoded = encode_pane_path(&path);
        assert_eq!(decode_pane_path(&encoded), TilePath(path));
    }
}

#[test]
fn the_frame_draws_splits_dividers_stacks_and_one_content_hole_per_stack() {
    let (dom, runner) = harness();
    let root = runner.root();
    let dom = dom.borrow();

    assert_eq!(count_class(&dom, root, "frisket-stack"), 2);
    // N children means N-1 dividers, never one per child.
    assert_eq!(count_class(&dom, root, "frisket-divider"), 1);
    assert_eq!(count_class(&dom, root, "frisket-tabbar"), 2);
    assert_eq!(count_class(&dom, root, "frisket-content"), 2);
    assert_eq!(count_class(&dom, root, "frisket-tab"), 3);

    // The content hole names the ACTIVE tile, the second tab here.
    let hole = find_attr(&dom, root, FRISKET_TILE_ATTR, "2").expect("active content hole");
    assert!(has_class(&*dom, hole, "frisket-content"));

    // Fractions ride on the branch as flex-grow, so layout sizes the panes.
    let split = find_class(&dom, root, "frisket-split").expect("split");
    let branch = dom
        .dom_children(split)
        .find(|child| has_class(&*dom, *child, "frisket-branch"))
        .expect("first branch");
    assert!(
        attr_of(&dom, branch, "style").is_some_and(|s| {
            s.contains("flex-grow: 0.7")
                && s.contains("flex-shrink: 0.7")
                && s.contains("flex-basis: 0px")
        }),
        "style {:?}",
        attr_of(&dom, branch, "style")
    );
}

#[test]
fn a_tab_is_an_aria_tab_and_only_the_active_one_is_selected() {
    let (dom, runner) = harness();
    let root = runner.root();
    let dom = dom.borrow();
    let first = find_attr(&dom, root, ATTR_TABID, "1").expect("first tab");
    let second = find_attr(&dom, root, ATTR_TABID, "2").expect("second tab");
    assert_eq!(attr_of(&dom, first, "role"), Some("tab"));
    assert_eq!(attr_of(&dom, first, "aria-selected"), Some("false"));
    assert_eq!(attr_of(&dom, second, "aria-selected"), Some("true"));
    assert_eq!(attr_of(&dom, first, "aria-label"), Some("First"));
    // The divider announces itself as a separator with an orientation.
    let divider = find_class(&dom, root, "frisket-divider").expect("divider");
    assert_eq!(attr_of(&dom, divider, "role"), Some("separator"));
    assert_eq!(attr_of(&dom, divider, "aria-orientation"), Some("vertical"));
}

/// The current tile marks the bar of the stack that holds it, and only
/// that one; the plain doors mark nothing.
#[test]
fn the_current_tile_marks_its_stack_s_bar_and_no_other() {
    fn current_view(state: &State) -> TestView {
        Box::new(frisket_with_current(
            &state.tree,
            // Tile 1 is the left stack's FIRST tab, not its active one:
            // the mark is about the stack, not the tab.
            Some(TileId(1)),
            |state: &mut State, event| state.events.push(event),
            |_: &Tile| Slot::Hole,
        ))
    }
    let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let runner = GenetAppRunner::new(
        dom.clone(),
        current_view as fn(&State) -> TestView,
        State {
            tree: sample(),
            events: Vec::new(),
        },
    );
    let root = runner.root();
    let dom = dom.borrow();

    let left = find_attr(&dom, root, ATTR_STACK, "0").expect("left bar");
    let right = find_attr(&dom, root, ATTR_STACK, "1").expect("right bar");
    assert_eq!(attr_of(&dom, left, "aria-current"), Some("true"));
    assert!(has_class(&*dom, left, "current"));
    assert_eq!(attr_of(&dom, right, "aria-current"), None);
    assert!(!has_class(&*dom, right, "current"));

    // `frisket` and `frisket_with` are unchanged: no bar is current.
    let (plain_dom, plain_runner) = harness();
    let plain_dom = plain_dom.borrow();
    let plain = find_attr(&plain_dom, plain_runner.root(), ATTR_STACK, "0").expect("bar");
    assert_eq!(attr_of(&plain_dom, plain, "aria-current"), None);
    assert!(!has_class(&*plain_dom, plain, "current"));
}

#[test]
fn gestures_report_events_and_never_mutate_the_tree() {
    let (dom, mut runner) = harness();
    let root = runner.root();
    let before = runner.state().tree.clone();

    let first = find_attr(&dom.borrow(), root, ATTR_TABID, "1").expect("first tab");
    runner.dispatch_click(first, PointerClick::at((2.0, 2.0)));
    assert_eq!(runner.state().events, [TileEvent::Activated(TileId(1))]);

    // The close x reports a close and stops propagation, so the tab's own
    // activate handler must not also fire.
    let close = find_attr(&dom.borrow(), root, "aria-label", "Close Second").expect("close");
    runner.dispatch_click(close, PointerClick::at((2.0, 2.0)));
    assert_eq!(
        runner.state().events,
        [
            TileEvent::Activated(TileId(1)),
            TileEvent::Closed(TileId(2)),
        ],
        "the close does not also activate its tab"
    );

    assert_eq!(
        runner.state().tree,
        before,
        "the component reports gestures; the caller owns the tree"
    );
}

#[test]
fn a_hit_resolves_to_the_divider_it_belongs_to() {
    let (dom, runner) = harness();
    let root = runner.root();
    let dom = dom.borrow();
    let divider = find_class(&dom, root, "frisket-divider").expect("divider");
    assert_eq!(
        divider_target(&*dom, divider),
        Some(DividerTarget {
            path: TilePath(vec![]),
            index: 0,
        }),
        "the root split's first boundary"
    );
    let tab = find_attr(&dom, root, ATTR_TABID, "1").expect("tab");
    assert_eq!(divider_target(&*dom, tab), None);
}

#[test]
fn a_hit_inside_a_tab_resolves_to_its_tile_but_the_close_does_not() {
    let (dom, runner) = harness();
    let root = runner.root();
    let dom = dom.borrow();

    // The label is a child of the tab, so the walk has to climb.
    let tab = find_attr(&dom, root, ATTR_TABID, "3").expect("third tab");
    let label = dom
        .dom_children(tab)
        .find(|child| has_class(&*dom, *child, "frisket-label"))
        .expect("label");
    assert_eq!(tab_target(&*dom, label), Some(TileId(3)));

    let close = find_attr(&dom, root, "aria-label", "Close Third").expect("close");
    assert_eq!(tab_target(&*dom, close), None);
    assert_eq!(close_target(&*dom, close), Some(TileId(3)));
    assert_eq!(close_target(&*dom, tab), None);
}

#[test]
fn a_content_hole_resolves_to_its_active_tile() {
    let (dom, runner) = harness();
    let root = runner.root();
    let dom = dom.borrow();
    let content = find_class(&dom, root, "frisket-content").expect("content hole");
    let active = attr(&*dom, content, FRISKET_TILE_ATTR)
        .and_then(|id| id.parse::<u64>().ok())
        .map(TileId)
        .expect("active tile id");
    assert_eq!(content_target(&*dom, content), Some(active));
    assert_eq!(content_target(&*dom, root), None);
}

#[test]
fn a_tab_bar_hit_resolves_to_its_stack_and_a_drop_index() {
    let (dom, runner) = harness();
    let root = runner.root();
    let dom = dom.borrow();
    let bar = find_class(&dom, root, "frisket-tabbar").expect("first tab bar");
    assert_eq!(stack_target(&*dom, bar), Some(TilePath(vec![0])));

    // Two tabs 100 wide at x=0 and x=100, so centres at 50 and 150. The
    // rects arrive through the closure: no layout engine in this crate.
    let rect_of = |node: NodeId| -> Option<(f32, f32, f32, f32)> {
        match attr_of(&dom, node, ATTR_TABID) {
            Some("1") => Some((0.0, 0.0, 100.0, 40.0)),
            Some("2") => Some((100.0, 0.0, 100.0, 40.0)),
            _ => None,
        }
    };
    assert_eq!(
        tab_drop_index(&*dom, bar, 10.0, rect_of),
        Some((TilePath(vec![0]), 0)),
        "left of both centres inserts first"
    );
    assert_eq!(
        tab_drop_index(&*dom, bar, 120.0, rect_of),
        Some((TilePath(vec![0]), 1))
    );
    assert_eq!(
        tab_drop_index(&*dom, bar, 400.0, rect_of),
        Some((TilePath(vec![0]), 2)),
        "past both centres appends"
    );
}
