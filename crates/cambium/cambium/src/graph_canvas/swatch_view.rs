/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

/// Render a bounded graph canvas with one native node target per painted node.
///
/// `on_node_click` owns navigation or staging. `on_node_hover` normally writes
/// the supplied id into [`GraphCanvasSwatch::hovered`] (and clears it on leave),
/// after which the host refreshes the registered leaf. `on_expand` switches to
/// the app's full-canvas route. The component does not invent those policies.
pub fn graph_canvas_swatch<State, AppAction, Id, Kind, Click, ClickOut, Hover, Expand, ExpandOut>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        |_state: &mut State, _id: Option<Id>| {},
        |_state: &mut State, _event: GraphCanvasNodeDrag<Id>| {},
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas Swatch with captured node motion. This is the
/// interaction variant of [`graph_canvas_swatch`]: click and hover retain their
/// existing meanings while `on_node_drag` receives Down/Move/Up in normalized
/// graph coordinates.
pub fn graph_canvas_swatch_with_drag<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Drag,
    DragOut,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_drag: Drag,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        |_state: &mut State, _id: Option<Id>| {},
        on_node_drag,
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas swatch whose retained paint focus follows the native
/// node button that pointer, keyboard, or programmatic focus selected.
///
/// `on_node_focus` normally writes its value into
/// [`GraphCanvasSwatch::focus`] before rebuilding the matching paint leaf.
pub fn graph_canvas_swatch_with_focus<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Focus,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_focus: Focus,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    Focus: Fn(&mut State, Option<Id>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        on_node_focus,
        |_state: &mut State, _event: GraphCanvasNodeDrag<Id>| {},
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas Swatch with captured node motion and independently
/// targetable relation cells. Relation callbacks receive a consumer-owned
/// stable relation id, so equal endpoint pairs remain separate cells.
pub fn graph_canvas_swatch_with_drag_and_relations<
    State,
    AppAction,
    Id,
    Kind,
    Click,
    ClickOut,
    Hover,
    Drag,
    DragOut,
    RelationClick,
    RelationClickOut,
    RelationHover,
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_drag: Drag,
    on_relation_click: RelationClick,
    on_relation_hover: RelationHover,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    RelationClickOut: OptionalAction<AppAction>,
    RelationClick: Fn(&mut State, String) -> RelationClickOut + Clone + 'static,
    RelationHover: Fn(&mut State, Option<String>) + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        |_state: &mut State, _id: Option<Id>| {},
        on_node_drag,
        on_relation_click,
        on_relation_hover,
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}

/// Render a graph-canvas Swatch with focus and captured node-motion callbacks.
///
/// The component emits motion only; the consumer keeps ownership of position,
/// solver, and persistence policy. Existing callers can keep using
/// [`graph_canvas_swatch`] or [`graph_canvas_swatch_with_focus`].
pub fn graph_canvas_swatch_with_focus_and_drag<
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
    Expand,
    ExpandOut,
>(
    swatch: &GraphCanvasSwatch<Id, Kind>,
    on_node_click: Click,
    on_node_hover: Hover,
    on_node_focus: Focus,
    on_node_drag: Drag,
    on_expand: Expand,
) -> impl View<State, AppAction, GenetCtx, Element = GenetElement>
where
    State: 'static,
    AppAction: 'static,
    Id: Clone + PartialEq + 'static,
    ClickOut: OptionalAction<AppAction>,
    Click: Fn(&mut State, Id) -> ClickOut + Clone + 'static,
    Hover: Fn(&mut State, Option<Id>) + Clone + 'static,
    Focus: Fn(&mut State, Option<Id>) + Clone + 'static,
    DragOut: OptionalAction<AppAction>,
    Drag: Fn(&mut State, GraphCanvasNodeDrag<Id>) -> DragOut + Clone + 'static,
    ExpandOut: OptionalAction<AppAction>,
    Expand: Fn(&mut State) -> ExpandOut + Clone + 'static,
{
    graph_canvas_swatch_with_focus_and_drag_and_relations(
        swatch,
        on_node_click,
        on_node_hover,
        on_node_focus,
        on_node_drag,
        |_state: &mut State, _id: String| {},
        |_state: &mut State, _id: Option<String>| {},
        on_expand,
        |_state: &mut State, _event: PointerEvent| {},
        |_state: &mut State, _event: WheelEvent| {},
    )
}
