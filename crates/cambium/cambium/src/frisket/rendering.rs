/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

pub(super) fn render_node<State, AppAction, Ev, Fill>(
    node: &TileTree,
    path: &[usize],
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    policy: crate::TabStripPolicy,
    on_event: &Ev,
    fill: &Fill,
) -> PaneView<State, AppAction>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    match node {
        TileTree::Split { axis, children } => {
            let dir = match axis {
                SplitAxis::Row => "row",
                SplitAxis::Column => "column",
            };
            let path_attr = encode_pane_path(path);
            // A draggable divider between adjacent children, each carrying its
            // split's path and boundary index so a host can resolve a drag to a
            // `DividerMoved` without keeping its own map.
            let mut items: Vec<PaneView<State, AppAction>> = Vec::new();
            for (index, branch) in children.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(index);
                let inner = render_node(
                    &branch.tree,
                    &child_path,
                    current,
                    marks,
                    policy,
                    on_event,
                    fill,
                );
                items.push(Box::new(
                    el::<_, State, AppAction>("div", inner)
                        .attr("class", "frisket-branch")
                        .attr(
                            "style",
                            format!(
                                "flex-grow: {frac}; flex-shrink: {frac}; flex-basis: 0px; min-width: 0; min-height: 0;",
                                frac = branch.fraction
                            ),
                        ),
                ));
                if index + 1 < children.len() {
                    items.push(Box::new(
                        el::<_, State, AppAction>("div", ())
                            .attr("class", "frisket-divider")
                            // An ARIA separator so the divider is reachable and
                            // announced, not just draggable.
                            .attr("role", "separator")
                            .attr(
                                "aria-orientation",
                                match axis {
                                    SplitAxis::Row => "vertical",
                                    SplitAxis::Column => "horizontal",
                                },
                            )
                            .attr(ATTR_DIVIDER, path_attr.clone())
                            .attr(ATTR_DINDEX, index.to_string()),
                    ));
                }
            }
            Box::new(
                el::<_, State, AppAction>("div", items)
                    .attr("class", "frisket-split")
                    .attr("style", format!("display: flex; flex-direction: {dir};")),
            )
        },
        TileTree::Stack(stack) => {
            render_stack_with_tab_policy(stack, path, current, marks, policy, on_event, fill)
        },
    }
}

pub(in super::super) fn render_stack<State, AppAction, Ev, Fill>(
    stack: &TabStack,
    path: &[usize],
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    on_event: &Ev,
    fill: &Fill,
) -> PaneView<State, AppAction>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    render_stack_with_tab_policy(
        stack,
        path,
        current,
        marks,
        crate::TabStripPolicy::Always,
        on_event,
        fill,
    )
}

pub(super) fn render_stack_with_tab_policy<State, AppAction, Ev, Fill>(
    stack: &TabStack,
    path: &[usize],
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    policy: crate::TabStripPolicy,
    on_event: &Ev,
    fill: &Fill,
) -> PaneView<State, AppAction>
where
    State: 'static,
    AppAction: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, AppAction> + Clone + 'static,
{
    // The stack's tabs as the crate's tab items: the tile's id is the tab's
    // stable key, and a host accent is the item's accent, so a product can tint
    // a tab to match its content without forking the component's CSS.
    let items: Vec<TabItem> = stack
        .tabs
        .iter()
        .map(|tile| {
            let mut item = TabItem::new(tile.title.clone()).with_key(tile.id.0.to_string());
            if let Some(accent) = tile.accent {
                item = item.with_accent(TabAccentColors::new(accent.background, accent.foreground));
            }
            if let Some(mark) = marks(tile.id) {
                item = item.with_mark(mark);
            }
            item
        })
        .collect();
    let ids: Vec<TileId> = stack.tabs.iter().map(|tile| tile.id).collect();
    let close_ids = ids.clone();
    let activate_event = on_event.clone();
    let close_event = on_event.clone();
    // The bar carries its stack's path, so a tab dropped on it resolves to
    // `DropTarget::Stack` (merge into this stack) rather than an edge split.
    let bar_attrs = [(ATTR_STACK, encode_pane_path(path))];
    // The host's current tile marks the stack it sits in, active tab or not.
    let is_current = current.is_some_and(|id| stack.tabs.iter().any(|tile| tile.id == id));
    let bar = policy.visible(stack.tabs.len()).then(|| {
        tab_bar_view(
            TabBar::new(&items, stack.active)
                .with_names(FRISKET_TAB_NAMES)
                .with_current(is_current)
                .with_attrs(&bar_attrs),
            move |state: &mut State, index: usize| {
                if let Some(id) = ids.get(index) {
                    activate_event(state, TileEvent::Activated(*id));
                }
            },
            // The × stops propagation, so a close never also activates.
            Some(move |state: &mut State, index: usize| {
                if let Some(id) = close_ids.get(index) {
                    close_event(state, TileEvent::Closed(*id));
                }
            }),
        )
    });

    // The content element: marked with the active tile's id and with which of
    // the three slots it carries. A `View` slot nests the host's child through
    // a `PortableKeyed` keyed by the tile, so the child survives the tile being
    // dragged to another stack; `Surface` and `Hole` stay empty, and what fills
    // their rect is the host's business — which is what lets one frame serve a
    // document, an external texture, or an app's own surface.
    let active_tile = stack.tabs.get(stack.active);
    let kind = match active_tile.map(fill) {
        Some(Slot::View(_)) => SlotKind::View,
        Some(Slot::Surface) => SlotKind::Surface,
        Some(Slot::Hole) | None => SlotKind::Hole,
    };
    let children: PortableKeyed<TileId, SlotChild<State, AppAction, Fill>> = match active_tile {
        Some(tile) if kind == SlotKind::View => {
            PortableKeyed::new(vec![(tile.id, SlotChild::new(tile.clone(), fill.clone()))])
        },
        _ => PortableKeyed::new(Vec::new()),
    };
    let mut content = el::<_, State, AppAction>("div", children)
        .attr("class", "frisket-content")
        .attr(ATTR_SLOT, kind.attr_value());
    // An empty stack is an inert hole, not an appearance of tile zero.
    // Only the actual active tile can publish content/input custody.
    if let Some(tile) = active_tile {
        content = content.attr(FRISKET_TILE_ATTR, tile.id.0.to_string());
    }

    Box::new(
        el::<_, State, AppAction>("div", (bar, content))
            .attr("class", "frisket-stack")
            .attr(
                "data-tabs-visible",
                policy.visible(stack.tabs.len()).to_string(),
            )
            .attr("style", "display: flex; flex-direction: column;"),
    )
}
