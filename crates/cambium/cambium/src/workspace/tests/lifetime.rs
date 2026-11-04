/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// The acceptance: a `Component` pane keeps its element and its local
/// counter across a drag within one projection, and keeps neither across a
/// move to a second projection — where the host's `TileId`-keyed state is
/// what carries the pane instead.
#[test]
fn a_pane_keeps_its_element_and_local_state_within_a_projection_and_neither_across_one() {
    let dom_a: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let dom_b: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
    let mut runner: GenetMultiRunner<AcceptState, _, AcceptView, ()> =
        GenetMultiRunner::new(accept_state());
    let a = runner.push_projection(dom_a.clone(), projection(0));
    let b = runner.push_projection(dom_b.clone(), projection(1));

    let root_a = runner.root(a).expect("projection a root");
    let counter_node = probe(&dom_a.borrow(), root_a, "counter-1").expect("the counter pane");
    let held = dom_a.borrow().parent(counter_node).expect("its content");
    assert_eq!(
        attr_of(&dom_a.borrow(), counter_node, "data-bumps"),
        Some("0")
    );

    // Bump the component's own local state through a click.
    runner.dispatch_click(a, counter_node, PointerClick::at((2.0, 2.0)));
    runner.dispatch_click(a, counter_node, PointerClick::at((2.0, 2.0)));
    assert_eq!(
        attr_of(&dom_a.borrow(), counter_node, "data-bumps"),
        Some("2"),
        "the component owns its counter across the dispatch rebuild"
    );

    // Drag the tile into the other stack of the SAME projection.
    runner.update(|state| {
        state.spaces[0].apply(&WorkspaceEvent::Tile(TileEvent::Dragged {
            tile: TileId(1),
            to: DropTarget::Stack {
                stack: TilePath(vec![1]),
                index: 0,
            },
        }));
    });
    let moved = probe(&dom_a.borrow(), root_a, "counter-1").expect("the counter, moved");
    assert_eq!(
        moved, counter_node,
        "the same element moved; PortableKeyed carried it"
    );
    let landed = dom_a.borrow().parent(moved).expect("its content element");
    assert_ne!(landed, held, "it really did change stacks");
    assert_eq!(content_target(&*dom_a.borrow(), landed), Some(TileId(1)));
    assert_eq!(
        content_target(&*dom_a.borrow(), held),
        Some(TileId(2)),
        "and the stack it left fell back to its spare tab"
    );
    assert_eq!(
        attr_of(&dom_a.borrow(), moved, "data-bumps"),
        Some("2"),
        "and its component-local counter came with it"
    );

    // Now move the tile to the SECOND projection, the way a tear-out to a
    // window does: take it from one workspace and insert it in the other.
    runner.update(|state| {
        let tile = state.spaces[0].take_tile(TileId(1)).expect("tile 1");
        assert!(
            state.spaces[1]
                .tiled_mut()
                .insert_tab_after(TileId(9), tile)
        );
    });

    assert!(
        probe(&dom_a.borrow(), root_a, "counter-1").is_none(),
        "the tile left the first window"
    );
    let root_b = runner.root(b).expect("projection b root");
    let arrived = probe(&dom_b.borrow(), root_b, "counter-1").expect("the counter, arrived");
    // Node ids are per-document, so comparing one across the two doms
    // proves nothing; what proves the element is new is that the first
    // window no longer holds one (asserted above) and that the arrival
    // carries a freshly-initialised local state.
    assert_eq!(
        attr_of(&dom_b.borrow(), arrived, "data-bumps"),
        Some("0"),
        "and nothing view-local survived the crossing"
    );
    assert_eq!(
        attr_of(&dom_b.borrow(), arrived, "data-note"),
        Some("kept by the host"),
        "what the host keeps keyed by TileId is readable in the new window"
    );
}
