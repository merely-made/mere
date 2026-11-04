/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// A graph-canvas Swatch that owns its own emphasis.
///
/// Selection stays application truth and arrives on the `swatch` props (a node
/// is selected because the graph says so). Hover, keyboard focus emphasis, and
/// relation hover are pointer-and-focus presentation state, so they live in the
/// component: an application no longer stores a hover field purely to route it
/// back into the view on the next rebuild. The `hovered` and `focus` fields of
/// the passed `swatch` are ignored for that reason.
///
/// The parent receives only [`GraphCanvasEvent`]s. For the callback-per-axis
/// form (an application that genuinely wants to own emphasis, e.g. to mirror it
/// across two views), the `graph_canvas_swatch*` family remains. Woodshed's
/// Related panel is that case: its hover is shared with a neighbor list, and
/// the two cross-highlight.
///
/// This is a state-ownership boundary, not a compile-time one. Measured with
/// `cargo llvm-lines`, the swatch costs the same either way (~1.6k lines, 11
/// instantiations); the boundary moves that instantiation from the app's state
/// type onto `GraphCanvasLocal<Id>` rather than removing it.
pub fn graph_canvas<State, A, Output, Id, Kind, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, Id, Kind, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    F: Fn(&mut State, GraphCanvasEvent<Id>) -> Output + 'static,
{
    graph_canvas_core(
        swatch,
        |event| event,
        |event: WheelEvent| {
            // A notch toward the top zooms in, matching every map surface; the
            // host's own scroll default is suppressed so the page does not
            // scroll under the zoom.
            if event.delta.1 == 0.0 {
                return None;
            }
            event.prevent_default();
            Some(GraphCanvasEvent::Zoom {
                factor: if event.delta.1 > 0.0 { 1.2 } else { 1.0 / 1.2 },
            })
        },
        on_event,
    )
}

/// [`graph_canvas`] with the wheel passed through whole, for a consumer that
/// applies its own wheel policy (a wheel-mode choice, pointer-anchored zoom, a
/// modifier that always zooms) instead of taking a fixed zoom factor.
///
/// Every other event is the same [`GraphCanvasEvent`] [`graph_canvas`] reports,
/// wrapped in [`GraphCanvasInput::Event`].
pub fn graph_canvas_with_wheel<State, A, Output, Id, Kind, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, Id, Kind, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    F: Fn(&mut State, GraphCanvasInput<Id>) -> Output + 'static,
{
    graph_canvas_core(
        swatch,
        GraphCanvasInput::Event,
        |event: WheelEvent| Some(GraphCanvasInput::Wheel(event)),
        on_event,
    )
}

/// The graph canvas, generic over the event a child reports to its own
/// component boundary: `lift` wraps each [`GraphCanvasEvent`] and `wheel` turns
/// a wheel notch into one (or none).
pub(super) fn graph_canvas_core<State, A, Output, Id, Kind, E, Lift, Wheel, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    lift: Lift,
    wheel: Wheel,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement>
+ use<State, A, Output, Id, Kind, E, Lift, Wheel, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    E: crate::Action + 'static,
    Lift: Fn(GraphCanvasEvent<Id>) -> E + Clone + 'static,
    Wheel: Fn(WheelEvent) -> Option<E> + Clone + 'static,
    F: Fn(&mut State, E) -> Output + 'static,
{
    component(
        swatch.clone(),
        |_props: &GraphCanvasSwatch<Id, Kind>| GraphCanvasLocal {
            pan_anchor: None,
            hovered: None,
            focus: None,
            hovered_relation: None,
        },
        // Emphasis is never parent-controlled: a props change (new graph data,
        // a new selection) leaves the pointer where it is.
        |_prev: &GraphCanvasSwatch<Id, Kind>,
         _next: &GraphCanvasSwatch<Id, Kind>,
         _local: &mut GraphCanvasLocal<Id>| {},
        move |props: &GraphCanvasSwatch<Id, Kind>, local: &GraphCanvasLocal<Id>| {
            let lift = lift.clone();
            let wheel = wheel.clone();
            let mut swatch = props.clone();
            swatch.hovered = local.hovered.clone();
            swatch.focus = local.focus.clone();
            if let Some(id) = &local.hovered_relation {
                for relation in &mut swatch.relations {
                    if &relation.id == id {
                        relation.emphasized = true;
                    }
                }
            }
            Box::new(graph_canvas_swatch_with_focus_and_drag_and_relations(
                &swatch,
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>, id: Id| lift(GraphCanvasEvent::Activate(id))
                },
                |local: &mut GraphCanvasLocal<Id>, id: Option<Id>| local.hovered = id,
                |local: &mut GraphCanvasLocal<Id>, id: Option<Id>| local.focus = id,
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>, drag: GraphCanvasNodeDrag<Id>| {
                        lift(GraphCanvasEvent::Drag(drag))
                    }
                },
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>, id: String| {
                        lift(GraphCanvasEvent::RelationActivate(id))
                    }
                },
                |local: &mut GraphCanvasLocal<Id>, id: Option<String>| {
                    local.hovered_relation = id;
                },
                {
                    let lift = lift.clone();
                    move |_: &mut GraphCanvasLocal<Id>| lift(GraphCanvasEvent::Expand)
                },
                {
                    let lift = lift.clone();
                    let width = (props.width as f32).max(1.0);
                    let height = (props.height as f32).max(1.0);
                    move |local: &mut GraphCanvasLocal<Id>, event: PointerEvent| match event.phase {
                        PointerPhase::Down => {
                            local.pan_anchor = Some(event.local);
                            None
                        },
                        PointerPhase::Move => {
                            let Some(anchor) = local.pan_anchor else {
                                return None;
                            };
                            let delta = (event.local.0 - anchor.0, event.local.1 - anchor.1);
                            local.pan_anchor = Some(event.local);
                            if delta == (0.0, 0.0) {
                                return None;
                            }
                            Some(lift(GraphCanvasEvent::Pan {
                                delta: (delta.0 / width, delta.1 / height),
                            }))
                        },
                        PointerPhase::Up | PointerPhase::Cancel => {
                            local.pan_anchor = None;
                            None
                        },
                    }
                },
                move |_: &mut GraphCanvasLocal<Id>, event: WheelEvent| wheel(event),
            )) as ComponentView<GraphCanvasLocal<Id>, E>
        },
        on_event,
    )
    .memo()
}

/// An atlas-specific swatch interaction surface.
///
/// Geographic field buttons exist solely for keyboard focus and activation.
/// They deliberately have `pointer-events:none`: the full-size surface below
/// is the only pointer route, so a field's rectangular semantic box can never
/// turn an empty polygon corner into a hit. Pointer capture preserves the
/// selected field across a drag even after the cursor leaves its footprint.
pub fn graph_atlas_swatch<State, A, Output, Id, Kind, F>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_event: F,
) -> impl View<State, A, GenetCtx, Element = GenetElement> + use<State, A, Output, Id, Kind, F>
where
    State: 'static,
    A: 'static,
    Output: OptionalAction<A> + 'static,
    Id: Clone + PartialEq + 'static,
    Kind: Clone + PartialEq + 'static,
    F: Fn(&mut State, GraphAtlasEvent<Id>) -> Output + 'static,
{
    component(
        swatch.clone(),
        |_props: &GraphCanvasSwatch<Id, Kind>| AtlasCanvasLocal {
            captured: None,
            down: None,
        },
        |_prev, _next, _local| {},
        |props: &GraphCanvasSwatch<Id, Kind>, _local: &AtlasCanvasLocal<Id>| {
            let atlas = props.atlas.as_ref().expect("graph_atlas_swatch requires swatch.atlas");
            let view = atlas.view(props.viewport, props.width as f32, props.height as f32);
            let field_buttons: Vec<_> = atlas
                .focus_order()
                .into_iter()
                .filter_map(|field| {
                    let bounds = field.footprint.bounds()?;
                    let top_left = view.project(Vec2::new(
                        field.anchor.x + bounds.origin.x,
                        field.anchor.y + bounds.origin.y,
                    ));
                    let bottom_right = view.project(Vec2::new(
                        field.anchor.x + bounds.origin.x + bounds.size.w,
                        field.anchor.y + bounds.origin.y + bounds.size.h,
                    ));
                    let id = field.id.clone();
                    let focus_id = id.clone();
                    Some(on_focus(focusable(on_click(
                        el::<_, AtlasCanvasLocal<Id>, GraphAtlasEvent<Id>>("button", ())
                            .attr("type", "button")
                            .attr("class", "graph-canvas-atlas-field")
                            .attr("aria-label", field.label.clone())
                            .attr(
                                "style",
                                format!(
                                    "position:absolute;pointer-events:none;background:transparent;border:0;padding:0;left:{}px;top:{}px;width:{}px;height:{}px;",
                                    top_left.0.min(bottom_right.0),
                                    top_left.1.min(bottom_right.1),
                                    (bottom_right.0 - top_left.0).abs().max(1.0),
                                    (bottom_right.1 - top_left.1).abs().max(1.0),
                                ),
                            ),
                        move |_: &mut AtlasCanvasLocal<Id>, _: PointerClick| {
                            GraphAtlasEvent::Activate(id.clone())
                        },
                    )), move |_: &mut AtlasCanvasLocal<Id>, event: FocusEvent| {
                        GraphAtlasEvent::Focus(if matches!(event.phase, FocusPhase::Gained) { Some(focus_id.clone()) } else { None })
                    }))
                })
                .collect();
            let atlas_for_pointer = atlas.clone();
            let atlas_for_hover = atlas.clone();
            let viewport = props.viewport;
            let width = props.width;
            let height = props.height;
            let defer_drag = props.defer_drag_rebuild;
            let root = el(
                "div",
                (
                    custom_leaf::<AtlasCanvasLocal<Id>, GraphAtlasEvent<Id>>(
                        props.leaf_key,
                        width,
                        height,
                    )
                    .attr("aria-hidden", "true"),
                    el("div", field_buttons).attr(
                        "style",
                        format!(
                            "position:absolute;left:0;top:0;width:{width}px;height:{height}px;"
                        ),
                    ),
                ),
            )
            .attr("class", "graph-canvas-swatch graph-canvas-atlas-swatch")
            .attr("role", "group")
            .attr("aria-label", props.label.clone())
            .attr(
                "style",
                format!(
                    "position:relative;display:block;overflow:hidden;width:{width}px;height:{height}px;max-width:100%;"
                ),
            );
            Box::new(on_hover(on_pointer(root, move |local: &mut AtlasCanvasLocal<Id>, event: PointerEvent| {
                if event.button != crate::PointerButton::Primary { return None; }
                if defer_drag && matches!(event.phase, PointerPhase::Move) {
                    event.defer_rebuild();
                }
                let view = atlas_for_pointer.view(viewport, width as f32, height as f32);
                let position = view.unproject_normalized(event.local);
                match event.phase {
                    PointerPhase::Down => {
                        local.down = Some(event.local);
                        local.captured = atlas_for_pointer.pick(view, event.local).map(|field| field.id.clone());
                    }
                    PointerPhase::Move => {}
                    PointerPhase::Up => {}
                    PointerPhase::Cancel => { local.captured = None; local.down = None; return None; }
                }
                let Some(id) = local.captured.clone() else { return None; };
                let Some(position) = position else { return None; };
                let drag = GraphAtlasEvent::Drag(GraphCanvasNodeDrag { id: id.clone(), phase: event.phase, position });
                if matches!(event.phase, PointerPhase::Up) {
                    local.captured = None;
                    local.down = None;
                }
                Some(drag)
            }), move |_: &mut AtlasCanvasLocal<Id>, event: HoverEvent| {
                if defer_drag { event.defer_rebuild(); }
                let picked = if matches!(event.phase, HoverPhase::Leave) { None } else {
                    atlas_for_hover.pick(atlas_for_hover.view(viewport, width as f32, height as f32), event.local).map(|field| field.id.clone())
                };
                Some(GraphAtlasEvent::Hover(picked))
            })) as ComponentView<AtlasCanvasLocal<Id>, GraphAtlasEvent<Id>>
        },
        on_event,
    )
    .memo()
}
