/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The view: a labelled region holding a bar, the sessions and the graph.

use std::collections::HashMap;

use cambium::{
    AnyView, GenetCtx, GenetElement, GraphCanvas, GraphCanvasNode, GraphCanvasNodeDrag,
    GraphCanvasRelation, GraphCanvasSubgraph, GraphCanvasSwatch, PointerEvent, WheelEvent, button,
    el, graph_canvas_swatch_with_focus_and_drag_and_relations,
};
use incipit::SessionId;
use sprigging::ColorF;

use crate::layout::{DEFAULT_LAYOUT, lay_out};
use crate::model::{GraphModel, MereViewModel, NodeState, SessionEntry, SessionStep, StatusNote};

/// The erased view the component and its parts are made of.
pub type MereViewElement<State, Action> = Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// Below this width the sessions stack above the graph.
const WIDE_AT: u32 = 560;
/// The sessions column's width beside the graph.
const SESSIONS_WIDTH: u32 = 220;
/// The bar's height above the body.
const BAR_HEIGHT: u32 = 40;
/// The body's share the graph takes when the sessions stack above it.
const NARROW_GRAPH_SHARE: f32 = 0.6;

/// The structural sheet, over [`cambium::GRAPH_CANVAS_SWATCH_CSS`]. No colour:
/// the host's sheet adds it, keyed on the `data-*` tokens the view sets. Each
/// relation kind gets its own line style, so kinds differ by more than colour.
pub const MERE_VIEW_CSS: &str = "\
    .mere-view { display: flex; flex-direction: column; min-width: 0; min-height: 0; } \
    .mere-view-bar { display: flex; flex-wrap: wrap; align-items: center; } \
    .mere-view-title { flex-grow: 1; margin: 0; font-size: 1em; } \
    .mere-view-actions, .mere-view-layouts { display: flex; flex-wrap: wrap; } \
    .mere-view-notice { margin: 0; } \
    .mere-view-body { display: flex; flex-direction: row; min-height: 0; } \
    .mere-view[data-width=\"narrow\"] .mere-view-body { flex-direction: column; } \
    .mere-view-sessions { list-style: none; margin: 0; padding: 0; overflow: auto; flex-shrink: 0; } \
    .mere-view-session { display: flex; flex-direction: column; } \
    .mere-view-session-steps { display: flex; flex-wrap: wrap; } \
    .mere-view-graph { position: relative; flex-shrink: 0; } \
    .mere-view .graph-canvas-swatch-relation::after { content: \"\"; position: absolute; left: 30%; right: 30%; top: 9px; border-top: 2px solid; } \
    .mere-view .graph-canvas-swatch-relation[data-kind=\"extracted\"]::after { border-top-style: dotted; } \
    .mere-view .graph-canvas-swatch-relation[data-kind=\"suggested\"]::after { border-top-style: dashed; }";

/// What the person asked for. The host carries it out or declines it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MereViewRequest {
    /// Open or activate the node with this key.
    Activate(String),
    Mint,
    Session(SessionId, SessionStep),
    /// One of the host's actions, by its key.
    Host(String),
    /// Lay the graph out by this strategy id.
    Layout(String),
}

/// Presentation state the view reports and the host stores for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MereViewEvent {
    Hover(Option<String>),
    Focus(Option<String>),
}

/// The view's own state: hover, focus and the last layout. The host stores
/// it beside its own, applies [`MereViewEvent`]s to it, and calls
/// [`lay_out`](Self::lay_out) before rendering.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MereViewState {
    pub hovered: Option<String>,
    pub focused: Option<String>,
    laid: Laid,
}

#[derive(Clone, Debug, Default, PartialEq)]
struct Laid {
    layout: String,
    size: (u32, u32),
    nodes: Vec<String>,
    relations: Vec<(String, String)>,
    positions: HashMap<String, (f32, f32)>,
}

impl MereViewState {
    pub fn apply(&mut self, event: MereViewEvent) {
        match event {
            MereViewEvent::Hover(key) => self.hovered = key,
            MereViewEvent::Focus(key) => self.focused = key,
        }
    }

    /// Lay the model's graph out for a tile of `width` by `height`, unless its
    /// nodes, its relations, the layout and the size are the ones last laid
    /// out. Call it before rendering whenever any of them may have changed.
    pub fn lay_out(&mut self, model: &MereViewModel, width: u32, height: u32) {
        let size = graph_size(width, height);
        let nodes: Vec<String> = model.graph.nodes.iter().map(|n| n.key.clone()).collect();
        let relations: Vec<(String, String)> = model
            .graph
            .relations
            .iter()
            .map(|r| (r.from.clone(), r.to.clone()))
            .collect();
        let laid = &self.laid;
        if laid.layout == model.layout
            && laid.size == size
            && laid.nodes == nodes
            && laid.relations == relations
        {
            return;
        }
        let positions = lay_out(&model.graph, &model.layout, size.0, size.1);
        self.laid = Laid {
            layout: model.layout.clone(),
            size,
            nodes,
            relations,
            positions,
        };
    }

    /// Where the last layout put the node with `key`, normalized to `0..=1`.
    pub fn position(&self, key: &str) -> Option<(f32, f32)> {
        self.laid.positions.get(key).copied()
    }
}

fn wide(width: u32) -> bool {
    width >= WIDE_AT
}

/// The graph's area, in pixels, inside a tile of `width` by `height`.
fn graph_size(width: u32, height: u32) -> (u32, u32) {
    let body = height.saturating_sub(BAR_HEIGHT).max(1);
    if wide(width) {
        (width.saturating_sub(SESSIONS_WIDTH).max(1), body)
    } else {
        (
            width.max(1),
            ((body as f32 * NARROW_GRAPH_SHARE) as u32).max(1),
        )
    }
}

/// One rendering of the view: the host's model and the view's state, in a
/// tile of `width` by `height` pixels, painting its graph under `leaf_key`.
#[derive(Clone, Copy, Debug)]
pub struct MereView<'a> {
    pub model: &'a MereViewModel,
    pub state: &'a MereViewState,
    pub leaf_key: u64,
    pub width: u32,
    pub height: u32,
}

impl MereView<'_> {
    /// The graph's area, in pixels.
    pub fn graph_size(&self) -> (u32, u32) {
        graph_size(self.width, self.height)
    }

    /// The layout in force: the host's, or the default when it names none.
    pub fn layout(&self) -> &str {
        if self.model.layout.is_empty() {
            DEFAULT_LAYOUT
        } else {
            &self.model.layout
        }
    }

    /// The graph as the canvas swatch the view renders and the leaf paints.
    pub fn swatch(&self) -> GraphCanvasSwatch<String, NodeState> {
        let GraphModel { nodes, relations } = &self.model.graph;
        let labels: HashMap<&str, &str> = nodes
            .iter()
            .map(|node| (node.key.as_str(), node.label.as_str()))
            .collect();
        let nodes = nodes
            .iter()
            .map(|node| GraphCanvasNode {
                id: node.key.clone(),
                kind: node.state,
                position: self.state.position(&node.key).unwrap_or((0.5, 0.5)),
                label: match node.state.word() {
                    Some(word) => format!("{}, {word}", node.label),
                    None => node.label.clone(),
                },
                key: Some(node.key.clone()),
            })
            .collect();
        let relations = relations
            .iter()
            .filter_map(|relation| {
                let from = labels.get(relation.from.as_str())?;
                let to = labels.get(relation.to.as_str())?;
                Some(GraphCanvasRelation {
                    id: relation.key.clone(),
                    from: relation.from.clone(),
                    to: relation.to.clone(),
                    kind: relation.provenance.token().to_string(),
                    label: format!("{from} to {to}"),
                    route: Vec::new(),
                    visible: true,
                    emphasized: false,
                })
            })
            .collect();
        let (width, height) = self.graph_size();
        let mut swatch = GraphCanvasSwatch::new(
            self.leaf_key,
            GraphCanvasSubgraph {
                nodes,
                edges: Vec::new(),
            },
        )
        .with_size(width, height)
        .with_label(format!("Graph of {}", self.model.title))
        .with_expand(false)
        .with_node_labels(true)
        .with_relations(relations);
        swatch.focus = self.state.focused.clone();
        swatch.hovered = self.state.hovered.clone();
        swatch
    }

    /// The paint leaf the host registers under [`leaf_key`](Self::leaf_key),
    /// with its palette for each node state.
    pub fn paint_leaf(&self, palette: impl Fn(&NodeState) -> ColorF) -> GraphCanvas {
        self.swatch().paint_leaf(palette)
    }
}

/// Render the mere view.
///
/// `on_change` receives presentation events for [`MereViewState::apply`];
/// `on_request` receives what the person asked for.
pub fn mere_view<State, Action, Change, Request>(
    view: MereView<'_>,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let notice = el::<_, State, Action>("p", model.notice.clone().unwrap_or_default())
        .attr("class", "mere-view-notice")
        .attr("role", "status")
        .attr("aria-live", "polite");
    let graph = match model.status.note() {
        None => graph(&view, on_change, on_request.clone()),
        Some(note) => status_panel(model.status.token(), note, on_request.clone()),
    };
    let body = el::<_, State, Action>("div", (sessions(&view, on_request.clone()), graph))
        .attr("class", "mere-view-body");
    Box::new(
        el::<_, State, Action>("section", (bar(&view, on_request), notice, body))
            .attr("class", "mere-view")
            .attr("role", "region")
            .attr("aria-label", format!("Mere: {}", model.title))
            .attr(
                "data-width",
                if wide(view.width) { "wide" } else { "narrow" },
            )
            .attr("data-status", model.status.token()),
    )
}

/// A button that reports `request`, carrying `attrs`.
fn request_button<State, Action, Request>(
    label: impl Into<String>,
    request: MereViewRequest,
    on_request: Request,
    attrs: Vec<(&'static str, String)>,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + 'static,
{
    let mut control = button(label, move |state: &mut State, _| {
        on_request(state, request.clone())
    });
    for (name, value) in attrs {
        control = control.attr(name, value);
    }
    Box::new(control)
}

fn bar<State, Action, Request>(
    view: &MereView<'_>,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let title = el::<_, State, Action>("h2", model.title.clone()).attr("class", "mere-view-title");

    let mut actions: Vec<MereViewElement<State, Action>> = Vec::new();
    if model.can_mint {
        actions.push(request_button(
            "New session",
            MereViewRequest::Mint,
            on_request.clone(),
            vec![("data-request", "mint".into())],
        ));
    }
    for action in &model.actions {
        actions.push(request_button(
            action.label.clone(),
            MereViewRequest::Host(action.key.clone()),
            on_request.clone(),
            vec![("data-action", action.key.clone())],
        ));
    }
    let actions = el::<_, State, Action>("div", actions)
        .attr("class", "mere-view-actions")
        .attr("role", "group")
        .attr("aria-label", "Actions");

    let current = view.layout();
    let layouts: Vec<MereViewElement<State, Action>> = model
        .layouts
        .iter()
        .map(|(id, label)| {
            request_button(
                label.clone(),
                MereViewRequest::Layout(id.clone()),
                on_request.clone(),
                vec![
                    ("data-layout", id.clone()),
                    ("aria-pressed", (id == current).to_string()),
                ],
            )
        })
        .collect();
    let layouts = el::<_, State, Action>("div", layouts)
        .attr("class", "mere-view-layouts")
        .attr("role", "group")
        .attr("aria-label", "Layout");

    Box::new(
        el::<_, State, Action>("div", (title, actions, layouts)).attr("class", "mere-view-bar"),
    )
}

fn sessions<State, Action, Request>(
    view: &MereView<'_>,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let items: Vec<MereViewElement<State, Action>> = view
        .model
        .sessions
        .iter()
        .map(|session| session_item(session, on_request.clone()))
        .collect();
    let size = if wide(view.width) {
        format!("width:{SESSIONS_WIDTH}px")
    } else {
        let body = view.height.saturating_sub(BAR_HEIGHT);
        let graph = graph_size(view.width, view.height).1;
        format!("max-height:{}px", body.saturating_sub(graph))
    };
    Box::new(
        el::<_, State, Action>("ul", items)
            .attr("class", "mere-view-sessions")
            .attr("aria-label", "Sessions")
            .attr("style", size),
    )
}

fn session_item<State, Action, Request>(
    session: &SessionEntry,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let mut words: Vec<&str> = Vec::new();
    if session.attached {
        words.push("Shown here");
    }
    if session.trashed {
        words.push("In the trash");
    }
    if let Some(detail) = &session.detail {
        words.push(detail);
    }
    let name = el::<_, State, Action>("span", session.name.clone())
        .attr("class", "mere-view-session-name");
    let state =
        el::<_, State, Action>("span", words.join(", ")).attr("class", "mere-view-session-state");
    let steps: Vec<MereViewElement<State, Action>> = session
        .steps
        .iter()
        .map(|&step| {
            request_button(
                step.label(),
                MereViewRequest::Session(session.id, step),
                on_request.clone(),
                vec![
                    ("data-step", step.token().into()),
                    ("aria-label", format!("{} {}", step.label(), session.name)),
                ],
            )
        })
        .collect();
    let steps = el::<_, State, Action>("div", steps).attr("class", "mere-view-session-steps");
    Box::new(
        el::<_, State, Action>("li", (name, state, steps))
            .attr("class", "mere-view-session")
            .attr("data-session", session.id.as_uuid().to_string())
            .attr("data-attached", session.attached.to_string())
            .attr("data-trashed", session.trashed.to_string()),
    )
}

fn graph<State, Action, Change, Request>(
    view: &MereView<'_>,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let swatch = view.swatch();
    let hover = {
        let on_change = on_change.clone();
        move |state: &mut State, key: Option<String>| on_change(state, MereViewEvent::Hover(key))
    };
    let focus =
        move |state: &mut State, key: Option<String>| on_change(state, MereViewEvent::Focus(key));
    let activate =
        move |state: &mut State, key: String| on_request(state, MereViewRequest::Activate(key));
    let canvas = graph_canvas_swatch_with_focus_and_drag_and_relations(
        &swatch,
        activate,
        hover,
        focus,
        |_: &mut State, _: GraphCanvasNodeDrag<String>| {},
        |_: &mut State, _: String| {},
        |_: &mut State, _: Option<String>| {},
        |_: &mut State| {},
        |_: &mut State, _: PointerEvent| {},
        |_: &mut State, _: WheelEvent| {},
    );
    let (width, height) = view.graph_size();
    Box::new(
        el::<_, State, Action>("div", canvas)
            .attr("class", "mere-view-graph")
            .attr("style", format!("width:{width}px;height:{height}px")),
    )
}

fn status_panel<State, Action, Request>(
    token: &'static str,
    note: &StatusNote,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let message =
        el::<_, State, Action>("p", note.message.clone()).attr("class", "mere-view-state-message");
    let action: Vec<MereViewElement<State, Action>> = note
        .action
        .iter()
        .map(|action| {
            request_button(
                action.label.clone(),
                MereViewRequest::Host(action.key.clone()),
                on_request.clone(),
                vec![("data-action", action.key.clone())],
            )
        })
        .collect();
    Box::new(
        el::<_, State, Action>("div", (message, action))
            .attr("class", "mere-view-state")
            .attr("role", "status")
            .attr("data-status", token),
    )
}
