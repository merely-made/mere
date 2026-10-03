/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Optional rails and drawers over the canonical pane frame.

use workbench::{
    PresentationEvent, SplitAxis, Tile, TileEvent, TileId, TileTree, WorkbenchPresentation,
};

use crate::frisket::render_stack;
use crate::{
    GenetCtx, GenetElement, Key, NamedKey, OverlayRole, OverlaySurface, PaneView, Placement, Slot,
    TabMark, View, button, button_with, el, encode_pane_path, on_key, overlay_surface,
    request_focus,
};

/// Render a canonical tree with separately controlled rails, minimum widths,
/// split shares, and drawers. The host decides all give-way policy and geometry.
/// Closed stacks have no content subtree, so their controls cannot receive Tab.
/// Opening focuses the panel's close control; closing requests the configured
/// source stack (or the rail if no source was configured). Hosts can request a
/// more specific editor focus in their presentation-event callback.
pub fn frisket_presented_with<State, Action, Ev, Change, Fill>(
    tree: &TileTree,
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    presentation: &WorkbenchPresentation,
    on_event: Ev,
    on_presentation_event: Change,
    fill: Fill,
) -> impl View<State, Action, GenetCtx, Element = GenetElement>
where
    State: 'static,
    Action: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Change: Fn(&mut State, PresentationEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, Action> + Clone + 'static,
{
    let mut overlays = Vec::new();
    let frame = render_node(
        tree,
        &[],
        current,
        marks,
        presentation,
        &on_event,
        &on_presentation_event,
        &fill,
        &mut overlays,
    );
    el::<_, State, Action>("div", (frame, overlays))
        .attr("class", "frisket-body")
        .attr(
            "style",
            "display:flex;position:relative;width:100%;height:100%;min-height:0;min-width:0;",
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DomHandle, GenetAppRunner, KeyEvent, PointerClick};
    use genet_scripted_dom::{NodeId, ScriptedDom};
    use layout_dom_api::{LayoutDom, LocalName, Namespace};
    use std::{cell::RefCell, rc::Rc};
    use workbench::{
        CollapsedStack, ContentSource, DrawerGeometry, SplitPresentation, StackPresentation,
        TabStack, TileBranch, TilePath,
    };

    struct State {
        tree: TileTree,
        presentation: WorkbenchPresentation,
    }

    fn tile(id: u64) -> Tile {
        Tile {
            id: TileId(id),
            title: format!("Tile {id}"),
            content: ContentSource::Open {
                kind: "test".into(),
                id: id.to_string(),
            },
            accent: None,
        }
    }

    fn state(drawer: bool) -> State {
        State {
            tree: TileTree::Split {
                axis: SplitAxis::Row,
                children: vec![
                    TileBranch::new(
                        0.7,
                        TileTree::Stack(TabStack {
                            tabs: vec![tile(1)],
                            active: 0,
                        }),
                    ),
                    TileBranch::new(
                        0.3,
                        TileTree::Stack(TabStack {
                            tabs: vec![tile(2), tile(3)],
                            active: 1,
                        }),
                    ),
                ],
            },
            presentation: WorkbenchPresentation {
                stacks: vec![StackPresentation {
                    anchor: TileId(2),
                    min_width: 280.0,
                    collapsed: Some(CollapsedStack {
                        label: "Reading".into(),
                        rail_width: 28.0,
                        open: false,
                        drawer: drawer.then_some(DrawerGeometry {
                            trigger: (292.0, 0.0, 28.0, 0.0),
                            panel_size: (400.0, 600.0),
                            bounds: (0.0, 0.0, 320.0, 480.0),
                        }),
                    }),
                }],
                splits: vec![SplitPresentation {
                    path: TilePath(vec![]),
                    fractions: vec![0.9, 0.1],
                }],
                return_focus: Some(TileId(1)),
                focus_return_requested: false,
            },
        }
    }

    fn view(state: &State) -> PaneView<State, ()> {
        Box::new(frisket_presented_with(
            &state.tree,
            None,
            &|_| None,
            &state.presentation,
            |_: &mut State, _| {},
            |state: &mut State, event| {
                state.presentation.apply(event);
            },
            |tile: &Tile| {
                Slot::View(Box::new(
                    button("Content", |_: &mut State, _| {})
                        .attr("id", format!("content-{}", tile.id.0)),
                ))
            },
        ))
    }

    fn attr<'a>(dom: &'a ScriptedDom, node: NodeId, name: &str) -> Option<&'a str> {
        dom.attribute(node, &Namespace::from(""), &LocalName::from(name))
    }

    fn find(dom: &ScriptedDom, node: NodeId, name: &str, value: &str) -> Option<NodeId> {
        if attr(dom, node, name) == Some(value) {
            return Some(node);
        }
        dom.dom_children(node)
            .find_map(|child| find(dom, child, name, value))
    }

    #[test]
    fn rail_omits_hidden_content_and_drawer_clamps_focuses_dismisses_to_source() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let initial = state(true);
        let canonical = initial.tree.clone();
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), view, initial);
        let root = runner.root();
        let rail = find(&dom.borrow(), root, "data-rail", "2").unwrap();
        assert!(find(&dom.borrow(), root, "id", "content-3").is_none());
        assert!(find(&dom.borrow(), root, "data-tabid", "3").is_none());
        assert!(runner.focusables().contains(&rail));
        runner.dispatch_click(rail, PointerClick::at((1.0, 1.0)));
        let panel = find(&dom.borrow(), root, "role", "dialog").unwrap();
        let style = attr(&dom.borrow(), panel, "style").unwrap().to_string();
        assert!(style.contains("width: 320px"), "{style}");
        assert!(style.contains("height: 480px"), "{style}");
        let close = find(&dom.borrow(), root, "class", "frisket-panel-close").unwrap();
        assert_eq!(runner.focus(), Some(close));
        let content = find(&dom.borrow(), root, "id", "content-3").unwrap();
        runner.set_focus(Some(content));
        runner.dispatch_key(KeyEvent::new(Key::Named(NamedKey::Escape)));
        assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
        assert!(find(&dom.borrow(), root, "id", "content-3").is_none());
        let source = find(&dom.borrow(), root, "class", "frisket-presented-stack").unwrap();
        assert_eq!(runner.focus(), Some(source));
        assert_eq!(runner.state().tree, canonical);

        runner.dispatch_click(rail, PointerClick::at((1.0, 1.0)));
        let outside = find(
            &dom.borrow(),
            root,
            "class",
            "overlay-surface-dismiss-layer",
        )
        .unwrap();
        runner.dispatch_click(outside, PointerClick::at((1.0, 1.0)));
        assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
        assert_eq!(runner.focus(), Some(source));
        assert_eq!(runner.state().tree, canonical);
    }

    #[test]
    fn inline_open_obeys_minimum_and_close_preserves_active_tab_and_split() {
        let dom: DomHandle = Rc::new(RefCell::new(ScriptedDom::new()));
        let initial = state(false);
        let canonical = initial.tree.clone();
        let mut runner = GenetAppRunner::<_, _, _, ()>::new(dom.clone(), view, initial);
        let root = runner.root();
        let rail = find(&dom.borrow(), root, "data-rail", "2").unwrap();
        runner.dispatch_click(rail, PointerClick::at((1.0, 1.0)));
        assert!(find(&dom.borrow(), root, "role", "dialog").is_none());
        assert!(find(&dom.borrow(), root, "id", "content-3").is_some());
        assert!(
            find(
                &dom.borrow(),
                root,
                "style",
                "flex-grow:0.1;flex-shrink:0.1;flex-basis:0px;min-width:280px;min-height:0;"
            )
            .is_some()
        );
        let close = find(&dom.borrow(), root, "class", "frisket-panel-close").unwrap();
        runner.dispatch_click(close, PointerClick::at((1.0, 1.0)));
        assert!(find(&dom.borrow(), root, "data-rail", "2").is_some());
        assert!(find(&dom.borrow(), root, "id", "content-3").is_none());
        assert_eq!(runner.state().tree, canonical);
    }
}

fn pixels(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

#[allow(clippy::too_many_arguments)]
fn render_node<State, Action, Ev, Change, Fill>(
    node: &TileTree,
    path: &[usize],
    current: Option<TileId>,
    marks: &dyn Fn(TileId) -> Option<TabMark>,
    presentation: &WorkbenchPresentation,
    on_event: &Ev,
    change: &Change,
    fill: &Fill,
    overlays: &mut Vec<PaneView<State, Action>>,
) -> PaneView<State, Action>
where
    State: 'static,
    Action: 'static,
    Ev: Fn(&mut State, TileEvent) + Clone + 'static,
    Change: Fn(&mut State, PresentationEvent) + Clone + 'static,
    Fill: Fn(&Tile) -> Slot<State, Action> + Clone + 'static,
{
    match node {
        TileTree::Split { axis, children } => {
            let fractions = presentation.fractions(path, children.len());
            let mut items: Vec<PaneView<State, Action>> = Vec::new();
            for (index, branch) in children.iter().enumerate() {
                let mut child_path = path.to_vec();
                child_path.push(index);
                let stack_presentation = match &branch.tree {
                    TileTree::Stack(stack) => presentation.stack(stack),
                    _ => None,
                };
                let rail = stack_presentation
                    .and_then(|entry| entry.collapsed.as_ref())
                    .filter(|collapsed| !collapsed.open || collapsed.drawer.is_some());
                let fraction = fractions.map_or(branch.fraction, |values| values[index]);
                let style = if let Some(rail) = rail {
                    let width = pixels(rail.rail_width);
                    match axis {
                        SplitAxis::Row => {
                            format!("flex:0 0 {width}px;min-width:{width}px;min-height:0;")
                        },
                        SplitAxis::Column => {
                            format!("flex:0 0 {width}px;min-width:0;min-height:{width}px;")
                        },
                    }
                } else {
                    let min_width = stack_presentation.map_or(0.0, |entry| pixels(entry.min_width));
                    format!(
                        "flex-grow:{fraction};flex-shrink:{fraction};flex-basis:0px;min-width:{min_width}px;min-height:0;"
                    )
                };
                let inner = render_node(
                    &branch.tree,
                    &child_path,
                    current,
                    marks,
                    presentation,
                    on_event,
                    change,
                    fill,
                    overlays,
                );
                items.push(Box::new(
                    el::<_, State, Action>("div", inner)
                        .attr("class", "frisket-branch")
                        .attr("style", style),
                ));
                if index + 1 < children.len() {
                    items.push(Box::new(
                        el::<_, State, Action>("div", ())
                            .attr("class", "frisket-divider")
                            .attr("role", "separator")
                            .attr(
                                "aria-orientation",
                                match axis {
                                    SplitAxis::Row => "vertical",
                                    SplitAxis::Column => "horizontal",
                                },
                            )
                            .attr("data-divider", encode_pane_path(path))
                            .attr("data-dindex", index.to_string()),
                    ));
                }
            }
            Box::new(
                el::<_, State, Action>("div", items)
                    .attr("class", "frisket-split")
                    .attr(
                        "style",
                        format!(
                            "display:flex;flex-direction:{};",
                            match axis {
                                SplitAxis::Row => "row",
                                SplitAxis::Column => "column",
                            }
                        ),
                    ),
            )
        },
        TileTree::Stack(stack) => {
            let entry = presentation.stack(stack);
            let collapsed = entry.and_then(|entry| entry.collapsed.as_ref());
            if let (Some(entry), Some(collapsed)) = (entry, collapsed) {
                let anchor = entry.anchor;
                if collapsed.open {
                    let close_change = change.clone();
                    let close = request_focus(
                        button(
                            format!("Close {}", collapsed.label),
                            move |state: &mut State, _| {
                                close_change(state, PresentationEvent::Closed(anchor));
                            },
                        )
                        .attr("class", "frisket-panel-close")
                        .attr("style", "flex:0 0 auto;"),
                        true,
                    );
                    let content = render_stack(stack, path, current, marks, on_event, fill);
                    let content = el::<_, State, Action>("div", content)
                        .attr("style", "flex:1 1 0px;min-width:0;min-height:0;");
                    let panel = el::<_, State, Action>("div", (close, content))
                        .attr("id", format!("frisket-panel-{}", anchor.0))
                        .attr("class", "frisket-open-panel")
                        .attr("style", "display:flex;flex-direction:column;width:100%;height:100%;min-width:0;min-height:0;overflow:auto;");
                    let dismiss_change = change.clone();
                    if let Some(geometry) = collapsed.drawer {
                        let (x0, y0, x1, y1) = geometry.bounds;
                        let size = (
                            pixels(geometry.panel_size.0).min(pixels(x1 - x0)),
                            pixels(geometry.panel_size.1).min(pixels(y1 - y0)),
                        );
                        let surface = OverlaySurface::new(geometry.trigger, size, geometry.bounds)
                            .with_placement(Placement::Below)
                            .with_role(OverlayRole::Dialog)
                            .with_label(collapsed.label.clone());
                        overlays.push(Box::new(overlay_surface(
                            &surface,
                            panel,
                            move |state: &mut State, _| {
                                dismiss_change(state, PresentationEvent::Closed(anchor));
                            },
                        )));
                    } else {
                        return Box::new(
                            on_key(panel, move |state: &mut State, event| {
                                if matches!(event.key, Key::Named(NamedKey::Escape)) {
                                    event.prevent_default();
                                    dismiss_change(state, PresentationEvent::Closed(anchor));
                                }
                            })
                            .focusable(false),
                        );
                    }
                }
                let open = collapsed.open;
                let open_change = change.clone();
                // A column of ordinary text boxes keeps the whole name visible
                // even on renderers that do not implement CSS writing-mode.
                let letters: Vec<_> = collapsed
                    .label
                    .chars()
                    .map(|letter| {
                        el::<_, State, Action>("span", letter.to_string())
                            .attr("class", "frisket-rail-letter")
                            .attr("style", "flex:0 0 auto;line-height:1.1;text-align:center;")
                    })
                    .collect();
                let label = el::<_, State, Action>("span", letters)
                    .attr("class", "frisket-rail-label")
                    .attr("aria-hidden", "true")
                    .attr("style", "display:flex;flex-direction:column;align-items:center;flex:0 1 auto;min-height:0;max-height:100%;overflow:auto;white-space:nowrap;");
                let rail = button_with(label, move |state: &mut State, _| {
                    open_change(
                        state,
                        if open {
                            PresentationEvent::Closed(anchor)
                        } else {
                            PresentationEvent::Opened(anchor)
                        },
                    );
                })
                .attr("class", "frisket-rail")
                .attr(
                    "aria-label",
                    format!(
                        "{} {}",
                        if open { "Close" } else { "Open" },
                        collapsed.label
                    ),
                )
                .attr("aria-expanded", if open { "true" } else { "false" })
                .attr("aria-controls", format!("frisket-panel-{}", anchor.0))
                .attr("data-rail", anchor.0.to_string())
                .attr(
                    "style",
                    "display:flex;align-items:center;justify-content:center;box-sizing:border-box;width:100%;height:100%;min-width:0;padding:0;border-radius:0;overflow:hidden;",
                );
                return Box::new(request_focus(
                    rail,
                    !open
                        && presentation.return_focus.is_none()
                        && presentation.focus_return_requested,
                ));
            }
            let content = render_stack(stack, path, current, marks, on_event, fill);
            let focus = presentation.focus_return_requested
                && presentation
                    .return_focus
                    .is_some_and(|id| stack.tabs.iter().any(|tile| tile.id == id));
            Box::new(request_focus(
                el::<_, State, Action>("div", content)
                    .attr("class", "frisket-presented-stack")
                    .attr("tabindex", "-1")
                    .attr("style", "width:100%;height:100%;min-width:0;min-height:0;"),
                focus,
            ))
        },
    }
}
