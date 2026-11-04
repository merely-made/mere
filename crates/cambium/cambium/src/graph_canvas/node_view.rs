/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// The full graph-canvas interaction surface, including independently
/// targetable relation cells.
pub fn graph_canvas_swatch_with_focus_and_drag_and_relations<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Focus,
    Drag,
    DragOut,
    RelationClick,
    RelationClickOut,
    RelationHover,
    Expand,
    ExpandOut,
    Pan,
    PanOut,
    Zoom,
    ZoomOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_focus: Focus,
    on_node_drag: Drag,
    on_relation_click: RelationClick,
    on_relation_hover: RelationHover,
    on_expand: Expand,
    on_background_pointer: Pan,
    on_wheel_event: Zoom,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    // The activating handlers may emit an action (returning `()` keeps them
    // silent, which is what every state-mutating consumer does). The hover and
    // focus handlers stay silent by construction: emphasis is presentation
    // state, never an app-facing event.
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    Focus: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    RelationClickOut: OptionalAction<AppAction>,
    RelationClick: Fn(&mut State, String) -> RelationClickOut + Clone + 'static,
    RelationHover: Fn(&mut State, Option<String>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
    // The background pointer and wheel handlers receive the raw events: pan
    // anchoring and zoom stepping are caller policy ([`graph_canvas`] turns
    // them into [`GraphCanvasEvent::Pan`] / [`GraphCanvasEvent::Zoom`]).
    // Returning `()` keeps a caller that has no viewport events silent.
    PanOut: OptionalAction<AppAction>,
    Pan: Fn(&mut State, PointerEvent) -> PanOut + Clone + 'static,
    ZoomOut: OptionalAction<AppAction>,
    Zoom: Fn(&mut State, WheelEvent) -> ZoomOut + Clone + 'static,
{
    let positions = swatch.projected_positions();
    let hit_size = swatch.hit_size.max(1.0);
    let size = Size {
        width: swatch.width as f32,
        height: swatch.height as f32,
    };
    let inset = swatch.node_radius + swatch.edge_width;
    let viewport = swatch.viewport;
    let defer_drag_rebuild = swatch.defer_drag_rebuild;
    // Visible node labels (opt-in): plain positioned text beside each node,
    // aria-hidden (the button already carries the accessible name) and
    // pointer-transparent (they must not steal the node's clicks).
    let labels: Vec<_> = if swatch.show_labels {
        let labels = swatch.resolved_labels();
        swatch
            .graph
            .nodes
            .iter()
            .zip(swatch.projected_positions())
            .enumerate()
            .filter(|(index, _)| labels[*index].is_some())
            .map(|(index, (node, _))| {
                // Same state modifiers the node buttons carry. A label layer
                // that cannot say which node is selected forces a consumer to
                // keep hand-rolling the whole layer for the sake of one
                // emphasis, which is what Isometry's pointcrawl was doing: the
                // party's location is the first thing that layer has to say.
                let mut class = String::from("graph-canvas-swatch-label");
                if swatch.selected.as_ref() == Some(&node.id) {
                    class.push_str(" selected");
                }
                if swatch.focus.as_ref() == Some(&node.id) {
                    class.push_str(" focused");
                }
                if swatch.hovered.as_ref() == Some(&node.id) {
                    class.push_str(" hovered");
                }
                let label = labels[index].expect("resolved visible label");
                el::<_, State, AppAction>("span", swatch.label_text(index).into_owned())
                    .attr("class", class)
                    .attr("aria-hidden", "true")
                    .attr("data-label-placement", label.placement.name())
                    .attr("data-node-index", index.to_string())
                    .attr("style", label.style())
            })
            .collect()
    } else {
        Vec::new()
    };
    let relation_targets: Vec<_> = swatch
        .projected_relations()
        .into_iter()
        .filter_map(|(relation, route)| {
            let ((from_x, from_y), (to_x, to_y)) = route_midpoint_segment(&route)?;
            let dx = to_x - from_x;
            let dy = to_y - from_y;
            let length = (dx * dx + dy * dy).sqrt().max(1.0);
            let midpoint_x = (from_x + to_x) / 2.0;
            let midpoint_y = (from_y + to_y) / 2.0;
            let angle = dy.atan2(dx).to_degrees();
            let mut class = String::from("graph-canvas-swatch-relation");
            if relation.emphasized {
                class.push_str(" emphasized");
            }
            let label = format!("{}: {}", relation.kind, relation.label);
            let label = if relation.link_key.is_some() {
                format!("{label}; {}{}",
                    match (relation.marks.forward, relation.marks.reverse) {
                        (true, true) => "both directions",
                        (true, false) => "forward",
                        (false, true) => "reverse",
                        _ => "direction unavailable",
                    },
                    if relation.marks.opposed { "; opposing assertions" } else { "" })
            } else {
                label
            };
            let mut target = el::<_, State, AppAction>("button", ())
                .attr("class", class)
                .attr("type", "button")
                .attr("aria-label", label)
                .attr("data-relation-id", relation.id.clone())
                .attr("data-link-direction", match (relation.marks.forward,relation.marks.reverse) {(true,true)=>"both",(true,false)=>"forward",(false,true)=>"reverse",_=>"none"})
                .attr("data-link-opposed", if relation.marks.opposed {"true"} else {"false"})
                // For the host's sheet: a line style per kind, so kinds differ
                // by more than colour.
                .attr("data-kind", relation.kind.clone())
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:{}px;top:{}px;width:{length}px;height:20px;transform:rotate({angle}deg);",
                        midpoint_x - length / 2.0,
                        midpoint_y - 10.0,
                    ),
                );
            if relation.emphasized {
                target = target.attr("aria-current", "true");
            }
            let click = on_relation_click.clone();
            let click_id = relation.id.clone();
            let hover = on_relation_hover.clone();
            let hover_id = relation.id.clone();
            Some(on_hover(
                on_click(target, move |state: &mut State, _: PointerClick| {
                    click(state, click_id.clone())
                }),
                move |state: &mut State, event: HoverEvent| match event.phase {
                    HoverPhase::Enter => hover(state, Some(hover_id.clone())),
                    HoverPhase::Leave => hover(state, None),
                    HoverPhase::Move => {}
                },
            ))
        })
        .collect();
    let targets: Vec<_> = swatch
        .graph
        .nodes
        .iter()
        .zip(positions)
        .enumerate()
        .map(|(index, (node, (_, (x, y))))| {
            let selected = swatch.selected.as_ref() == Some(&node.id);
            let focused = swatch.focus.as_ref() == Some(&node.id);
            let hovered = swatch.hovered.as_ref() == Some(&node.id);
            let mut class = String::from("graph-canvas-swatch-node");
            if selected {
                class.push_str(" selected");
            }
            if focused {
                class.push_str(" focused");
            }
            if hovered {
                class.push_str(" hovered");
            }
            let mut target = el::<_, State, AppAction>("button", ())
                .attr("class", class)
                .attr("type", "button")
                .attr("aria-label", node.label.clone())
                .attr("data-node-index", index.to_string())
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:{}px;top:{}px;width:{hit_size}px;height:{hit_size}px;",
                        x - hit_size / 2.0,
                        y - hit_size / 2.0,
                    ),
                );
            if selected {
                target = target.attr("aria-current", "true");
            }
            if let Some(key) = &node.key {
                target = target.attr("data-key", key.clone());
            }
            const OWN: [&str; 6] = [
                "class",
                "type",
                "aria-label",
                "data-node-index",
                "data-key",
                "style",
            ];
            for (_, name, value) in swatch
                .node_attrs
                .iter()
                .filter(|(id, name, _)| *id == node.id && !OWN.contains(&name.as_str()))
            {
                target = target.attr(name.clone(), value.clone());
            }
            let target = request_focus(target, swatch.focus_request.as_ref() == Some(&node.id));

            let click = on_node_click.clone();
            let click_id = node.id.clone();
            let hover = on_node_hover.clone();
            let enter_id = node.id.clone();
            let focus = on_node_focus.clone();
            let focus_id = node.id.clone();
            let drag = on_node_drag.clone();
            let drag_id = node.id.clone();
            let target_left = x - hit_size / 2.0;
            let target_top = y - hit_size / 2.0;
            on_pointer(focusable(on_focus(
                on_hover(
                    on_click(target, move |state: &mut State, _: PointerClick| {
                        click(state, click_id.clone())
                    }),
                    move |state: &mut State, event: HoverEvent| match event.phase {
                        HoverPhase::Enter => hover(state, Some(enter_id.clone())),
                        HoverPhase::Leave => hover(state, None),
                        HoverPhase::Move => {}
                    },
                ),
                move |state: &mut State, event: FocusEvent| match event.phase {
                    FocusPhase::Gained => focus(state, Some(focus_id.clone())),
                    FocusPhase::Lost => focus(state, None),
                },
            )), move |state: &mut State, event: PointerEvent| {
                if defer_drag_rebuild && matches!(event.phase, PointerPhase::Move) {
                    event.defer_rebuild();
                }
                let position = graph_position_at(
                    viewport,
                    size,
                    inset,
                    (target_left + event.local.0, target_top + event.local.1),
                );
                drag(
                    state,
                    GraphCanvasNodeDrag {
                        id: drag_id.clone(),
                        phase: event.phase,
                        position,
                    },
                )
            })
        })
        .collect();

    let expand = on_expand.clone();
    let expand_buttons: Vec<_> = if swatch.show_expand {
        vec![on_click(
            el::<_, State, AppAction>("button", "Expand")
                .attr("class", "graph-canvas-swatch-expand")
                .attr("type", "button")
                .attr("aria-label", "Expand graph")
                .attr("style", "position:absolute;right:5px;top:5px;"),
            move |state: &mut State, _: PointerClick| expand(state),
        )]
    } else {
        Vec::new()
    };

    let root = el(
        "div",
        (
            custom_leaf::<State, AppAction>(swatch.leaf_key, swatch.width, swatch.height)
                .attr("aria-hidden", "true"),
            el("div", labels)
                .attr("class", "graph-canvas-swatch-labels")
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:0;top:0;width:{}px;height:{}px;",
                        swatch.width, swatch.height
                    ),
                ),
            el("div", relation_targets)
                .attr("class", "graph-canvas-swatch-relation-targets")
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:0;top:0;width:{}px;height:{}px;",
                        swatch.width, swatch.height
                    ),
                ),
            el("div", targets)
                .attr("class", "graph-canvas-swatch-targets")
                .attr(
                    "style",
                    format!(
                        "position:absolute;left:0;top:0;width:{}px;height:{}px;",
                        swatch.width, swatch.height
                    ),
                ),
            expand_buttons,
        ),
    )
    .attr("class", "graph-canvas-swatch")
    .attr("role", "group")
    .attr("aria-label", swatch.label.clone())
    // `overflow:hidden` crops the native node, label, and relation targets a
    // panned or zoomed viewport projects outside this box; the swatch is their
    // `position:relative` containing block, so the clip applies to them. The
    // painted leaf clips itself (see sprigging's GraphCanvas paint).
    .attr(
        "style",
        format!(
            "position:relative;display:block;overflow:hidden;width:{}px;height:{}px;max-width:100%;",
            swatch.width, swatch.height
        ),
    );
    // Background drags and wheel notches reach these root handlers only when
    // no inner target claims the event: a node's own pointer handler is the
    // innermost element under a node drag, so panning never fights dragging.
    on_wheel(on_pointer(root, on_background_pointer), on_wheel_event)
}
