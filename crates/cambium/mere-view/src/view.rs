/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The view: a labelled region holding a bar, the sessions and the graph.

use std::collections::{BTreeMap, HashMap};

use cambium::{
    AnyView, DetailPopoverEvent, DetailPopoverMode, DetailPopoverState, GenetCtx, GenetElement,
    GraphCanvas, GraphCanvasNode, GraphCanvasNodeDrag, GraphCanvasRelation, GraphCanvasSubgraph,
    GraphCanvasSwatch, KeyEvent, Keyed, PointerEvent, RelationLine, TextCommand, TextFieldMode,
    TextInput, WheelEvent, button, caret_field_children, el,
    graph_canvas_swatch_with_focus_and_drag_and_relations, on_key, text_command_from_key,
};
use incipit::SessionId;
use sprigging::ColorF;

use crate::layout::{DEFAULT_LAYOUT, SEPARATION, lay_out};
use crate::model::{
    ArchiveEntry, GraphModel, MereNodeKind, MereViewModel, MergeConflictEntry, MergeModel,
    MergeOption, Provenance, SessionEntry, SessionPlacement, SessionStep, StatusNote, TierCard,
};
use crate::tier::{
    ON_SHELF, SAVED_FROM, archive_node_key, archive_of_node, card_listed, card_node_key,
    card_of_node, tier_card, tier_height, tier_shown, tier_widgets,
};

/// The erased view the component and its parts are made of.
pub type MereViewElement<State, Action> = Box<dyn AnyView<State, Action, GenetCtx, GenetElement>>;

/// Below this width the sessions stack above the graph.
const WIDE_AT: u32 = 560;
/// The sessions column's width beside the graph.
const SESSIONS_WIDTH: u32 = 220;
/// The bar's height above the body: one row, or two when narrow.
const BAR_HEIGHT: u32 = 40;
const NARROW_BAR_HEIGHT: u32 = 64;
/// The body's share the graph takes when the sessions stack above it.
const NARROW_GRAPH_SHARE: f32 = 0.6;
/// An overmap container's width, and the gap around and between containers.
const CONTAINER_WIDTH: u32 = 232;
const OVERMAP_GAP: u32 = 8;
/// A container's padding, one line of text, and one row of controls. A
/// container's height is counted in these, so the band, the canvas below it
/// and the layout agree before anything renders.
pub(crate) const CONTAINER_PAD: u32 = 8;
pub(crate) const LINE: u32 = 20;
pub(crate) const CONTROL_ROW: u32 = 34;
/// The most of the body the overmap band takes; past it the band scrolls.
const OVERMAP_SHARE: f32 = 0.6;

/// The structural sheet, over [`cambium::GRAPH_CANVAS_SWATCH_CSS`]. It holds
/// the bar to the height the graph's size assumes, lays the notice over the
/// top of the graph's area rather than beside the controls, and fits the
/// graph to that area without a border of its own. Relation kinds are told
/// apart by the lines the paint leaf draws, not by this sheet. No colour: the
/// host's sheet adds it, keyed on the `data-*` tokens the view sets.
///
/// `.mere-view-main` carries no overflow clip: under the renderer pinned at
/// netrender `aba7d837`, a clipped box holding the self-clipping graph and the
/// positioned notice blanked the whole frame.
pub const MERE_VIEW_CSS: &str = "\
    .mere-view { position: relative; display: flex; flex-direction: column; overflow: hidden; } \
    .mere-view-bar { display: flex; flex-wrap: nowrap; align-items: center; height: 40px; overflow-x: auto; overflow-y: hidden; white-space: nowrap; box-sizing: border-box; flex-shrink: 0; } \
    .mere-view-title { flex-grow: 1; min-width: 0; margin: 0; font-size: 1em; overflow: hidden; text-overflow: ellipsis; } \
    .mere-view-actions, .mere-view-layouts { display: flex; flex-wrap: nowrap; flex-shrink: 0; } \
    .mere-view-body { position: relative; display: flex; flex-direction: row; min-height: 0; } \
    .mere-view[data-width=\"narrow\"] .mere-view-bar { flex-wrap: wrap; align-content: center; height: 64px; } \
    .mere-view[data-width=\"narrow\"] .mere-view-layouts { flex-basis: 100%; } \
    .mere-view[data-width=\"narrow\"] .mere-view-body { flex-direction: column; } \
    .mere-view-main { position: relative; flex-shrink: 0; } \
    .mere-view-notice { position: absolute; left: 8px; right: 8px; top: 8px; margin: 0; z-index: 2; } \
    .mere-view-sessions { list-style: none; margin: 0; padding: 0; overflow: auto; flex-shrink: 0; box-sizing: border-box; } \
    .mere-view-session { display: flex; flex-direction: column; } \
    .mere-view-session-steps { display: flex; flex-wrap: wrap; } \
    .mere-view-archive { display:flex; flex-direction:column; } \
    .mere-view-archive-steps { display:flex; flex-wrap:wrap; } \
    .mere-view[data-graph=\"hidden\"] .mere-view-body { flex-direction:column; flex:1; } \
    .mere-view[data-graph=\"hidden\"] .mere-view-sessions { width:100%; flex:1; min-height:0; max-height:none; } \
    .mere-view[data-graph=\"hidden\"] .mere-view-main { width:100%; height:32px; } \
    .mere-view-graph { position: relative; } \
    .mere-view .graph-canvas-swatch { border: 0; border-radius: 0; } \
    .mere-view-overmap { display: flex; flex-wrap: wrap; align-items: flex-start; align-content: flex-start; gap: 8px; padding: 8px; box-sizing: border-box; overflow-x: hidden; overflow-y: auto; } \
    .mere-view-container { display: flex; flex-direction: column; flex-shrink: 0; box-sizing: border-box; padding: 8px; overflow: hidden; border: 1px solid currentColor; border-radius: 10px; } \
    .mere-view-container[data-attached=\"true\"] { border-width: 2px; } \
    .mere-view-container[data-container=\"mint\"] { border-style: dashed; } \
    .mere-view-container-title, .mere-view-container .mere-view-session-name, .mere-view-container .mere-view-session-state, .mere-view-container .mere-view-archive-name { height: 20px; line-height: 20px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex-shrink: 0; } \
    .mere-view-container-title, .mere-view-container .mere-view-session-name { font-weight: bold; } \
    .mere-view-container .mere-view-session-steps, .mere-view-container .mere-view-archive-steps { display: flex; flex-wrap: wrap; gap: 4px; padding-bottom: 4px; } \
    .mere-view-container .mere-view-session-steps > button { flex: 1 1 40%; } \
    .mere-view-container .mere-view-archive-steps > button { flex: 1 1 auto; white-space: nowrap; } \
    .mere-view-container-archives { list-style: none; margin: 0; padding: 0; } \
    .mere-view-container .mere-view-archive { display: flex; flex-direction: column; } \
    .mere-view-tier-rule { position: absolute; left: 0; height: 0; border-top: 1px solid currentColor; opacity: 0.35; pointer-events: none; } \
    .mere-view-tier-controls { position: absolute; display: flex; flex-direction: column; gap: 4px; z-index: 2; } \
    .mere-view-card { position: absolute; z-index: 3; border-width: 2px; padding: 8px; box-sizing: border-box; } \
    .mere-view-card > * { flex-shrink: 0; margin: 0; } \
    .mere-view-card-head { display: flex; flex-wrap: nowrap; align-items: center; gap: 4px; height: 34px; } \
    .mere-view-card-head .mere-view-session-name { flex: 1 1 auto; min-width: 0; } \
    .mere-view-card .mere-view-card-close { height: 28px; line-height: 26px; margin: 0; padding-top: 0; padding-bottom: 0; box-sizing: border-box; } \
    .mere-view-card .mere-view-session-state { height: 20px; line-height: 20px; } \
    .mere-view-card .mere-view-session-steps { gap: 4px; padding: 0 0 4px 0; } \
    .mere-view-card .mere-view-session-steps > button { height: 30px; line-height: 28px; margin: 0; padding-top: 0; padding-bottom: 0; box-sizing: border-box; } \
    .mere-view-card .mere-view-archive { height: 28px; } \
    .mere-view-archive-chip { height: 24px; line-height: 22px; margin: 0; padding-top: 0; padding-bottom: 0; box-sizing: border-box; max-width: 100%; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; } \
    .mere-view-popover-layer { position: absolute; left: 0; top: 0; width: 0; height: 0; z-index: 4; } \
    .mere-view-popover-layer .overlay-surface-panel { box-sizing: border-box; overflow: hidden; border: 1px solid currentColor; border-radius: 8px; } \
    .mere-view-popover-layer .detail-popover-preview > *, .mere-view-popover-layer .detail-popover-content > * { margin: 0; } \
    .mere-view-popover-layer .detail-popover-preview, .mere-view-popover-layer .detail-popover-content { display: flex; flex-direction: column; padding: 8px; box-sizing: border-box; } \
    .mere-view-popover-layer .mere-view-archive-name, .mere-view-popover-layer .mere-view-archive-from, .mere-view-popover-layer .mere-view-archive-detail { height: 20px; line-height: 20px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; flex-shrink: 0; } \
    .mere-view-popover-layer .mere-view-archive-name { font-weight: bold; } \
    .mere-view-popover-layer .mere-view-archive-steps { display: flex; flex-direction: column; gap: 4px; padding: 4px 0 0 0; flex-shrink: 0; } \
    .mere-view-popover-layer .mere-view-archive-steps > button { height: 30px; line-height: 28px; margin: 0; padding-top: 0; padding-bottom: 0; box-sizing: border-box; flex-shrink: 0; } \
    .mere-view-merge { position: absolute; left: 8px; right: 8px; bottom: 8px; z-index: 5; display: flex; flex-direction: column; gap: 4px; padding: 8px; box-sizing: border-box; overflow-x: hidden; overflow-y: auto; border: 2px solid currentColor; border-radius: 10px; } \
    .mere-view-merge > * { flex-shrink: 0; margin: 0; } \
    .mere-view-merge-title { font-size: 1em; font-weight: bold; } \
    .mere-view-merge-conflicts { list-style: none; margin: 0; padding: 0; display: flex; flex-direction: column; gap: 8px; } \
    .mere-view-merge-conflict { display: flex; flex-direction: column; gap: 2px; } \
    .mere-view-merge-conflict > p { margin: 0; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; } \
    .mere-view-merge-part { font-weight: bold; } \
    .mere-view-merge-dropped { margin: 0; padding-left: 16px; } \
    .mere-view-merge-choices, .mere-view-merge-actions { display: flex; flex-wrap: wrap; gap: 4px; } \
    .mere-view-merge-hand { border: 1px solid currentColor; padding: 2px 4px; min-height: 24px; box-sizing: border-box; white-space: pre; overflow-x: auto; overflow-y: hidden; }";

/// What the person asked for. The host carries it out or declines it. More
/// requests may come, so a host matching on them keeps a wildcard arm that
/// declines what it does not carry out.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum MereViewRequest {
    /// Open or activate the node with this key.
    Activate(String),
    Mint,
    Session(SessionId, SessionStep),
    Archive(ArchiveRequest),
    /// A step on the pending merge.
    Merge(MergeRequest),
    /// One of the host's actions, by its key.
    Host(String),
    /// Lay the graph out by this strategy id.
    Layout(String),
}

/// A step on the pending merge: a choice for one conflict, by its key, then
/// Apply once every conflict has one, or Cancel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MergeRequest {
    Choose(String, MergeOption),
    Apply,
    Cancel,
}

/// Explicit requests to the archive owner. Opening creates a read-only thaw;
/// composing leaves its immutable sources untouched.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ArchiveRequest {
    Save(SessionId),
    Open(String),
    Compose(Vec<String>),
}

/// Presentation state the view reports and the host stores for it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MereViewEvent {
    Hover(Option<String>),
    Focus(Option<String>),
    SelectArchive(String),
    /// Open a session's or the shelf's card in the tier, or close the open
    /// one with `None`. One card is open at a time. The tier grows to hold
    /// it, so the host lays the view out again after applying this.
    Expand(Option<TierCard>),
    /// An archive node's popover in the tier, by archive key: hovering peeks,
    /// activating pins, Escape or an outside click dismisses.
    ArchivePopover(String, DetailPopoverEvent),
    /// An edit to a merge conflict's hand value, by conflict key. The draft
    /// starts from `seed` the first time the conflict is edited.
    MergeDraft {
        key: String,
        seed: String,
        command: TextCommand,
    },
}

/// The view's own state: hover, focus and the last layout. The host stores
/// it beside its own and hands it each [`MereViewEvent`] with
/// [`apply_and_lay_out`](Self::apply_and_lay_out), and calls
/// [`lay_out`](Self::lay_out) whenever the model or tile changes: an opened
/// or closed tier card changes the area the graph is laid out in. (A host
/// that only [`apply`](Self::apply)s sees the last layout fitted into the
/// area it has until it lays out again.)
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MereViewState {
    /// Host-owned catalog curation. Layout still uses every source member;
    /// only the final catalog Swatch is folded, before the resident tier.
    pub folds: Vec<incipit::curation::FoldRecord>,
    pub hovered: Option<String>,
    pub focused: Option<String>,
    /// Presentation-only selection for an explicit Compose request.
    pub selected_archives: Vec<String>,
    /// The tier node open as its card: a session's, or the shelf's.
    pub expanded: Option<TierCard>,
    /// The archive whose popover peeks or is pinned, in a tier.
    pub archive_popover: Option<(String, DetailPopoverState)>,
    /// The tier node a closed card or a dismissed popover hands keyboard
    /// focus back to. It clears once any node takes focus.
    pub focus_return: Option<String>,
    /// The hand values being written for a pending merge's conflicts, by
    /// conflict key. Presentation only until Edit by hand reports one.
    pub merge_drafts: BTreeMap<String, TextInput>,
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

fn wide(width: u32) -> bool {
    width >= WIDE_AT
}

/// The height left below the bar.
fn body_height(width: u32, height: u32) -> u32 {
    let bar = if wide(width) {
        BAR_HEIGHT
    } else {
        NARROW_BAR_HEIGHT
    };
    height.saturating_sub(bar).max(1)
}

pub(crate) fn has_session_content(model: &MereViewModel) -> bool {
    model.can_mint || !model.sessions.is_empty() || model.archive.is_some()
}

/// Whether the sessions show as the overmap band above the graph.
fn overmap_shown(model: &MereViewModel) -> bool {
    model.placement == SessionPlacement::Overmap && !model.hide_graph && has_session_content(model)
}

/// One container in the overmap band.
enum Container<'a> {
    Session(&'a SessionEntry),
    /// The archive shelf: Compose, and the archives no live session holds.
    Shelf,
    Mint,
}

/// The overmap's containers in order: the sessions, the shelf, then minting.
fn containers(model: &MereViewModel) -> Vec<Container<'_>> {
    model
        .sessions
        .iter()
        .map(Container::Session)
        .chain(model.archive.is_some().then_some(Container::Shelf))
        .chain(model.can_mint.then_some(Container::Mint))
        .collect()
}

/// The archives on `session`, or on the shelf for `None`. An archive sits on
/// a session only when its owner names that session and the mere still lists
/// it; every other archive sits on the shelf.
pub(crate) fn archives_on<'a>(
    model: &'a MereViewModel,
    session: Option<&SessionEntry>,
) -> Vec<&'a ArchiveEntry> {
    let Some(archive) = &model.archive else {
        return Vec::new();
    };
    archive
        .entries
        .iter()
        .filter(|entry| {
            let held = entry
                .session
                .filter(|id| model.sessions.iter().any(|session| session.id == *id));
            match session {
                Some(session) => held == Some(session.id),
                None => held.is_none(),
            }
        })
        .collect()
}

/// Rows of controls for `count` controls, two to a row.
pub(crate) fn control_rows(count: usize) -> u32 {
    count.div_ceil(2) as u32
}

/// One archive chip's height: its name and, when it offers any, its controls.
fn chip_height(model: &MereViewModel, entry: &ArchiveEntry) -> u32 {
    let can_compose = model
        .archive
        .as_ref()
        .is_some_and(|archive| archive.can_compose);
    LINE + if entry.can_open || can_compose {
        CONTROL_ROW
    } else {
        0
    }
}

pub(crate) fn can_save(model: &MereViewModel, session: &SessionEntry) -> bool {
    model
        .archive
        .as_ref()
        .is_some_and(|archive| archive.save_sessions.contains(&session.id))
}

/// A container's height, in the same rows its elements render in.
fn container_height(model: &MereViewModel, container: &Container<'_>) -> u32 {
    let chips = |session| -> u32 {
        archives_on(model, session)
            .into_iter()
            .map(|entry| chip_height(model, entry))
            .sum()
    };
    CONTAINER_PAD * 2
        + match container {
            Container::Session(session) => {
                let steps = session.steps.len() + usize::from(can_save(model, session));
                LINE * 2 + control_rows(steps) * CONTROL_ROW + chips(Some(*session))
            },
            Container::Shelf => {
                let compose = model
                    .archive
                    .as_ref()
                    .is_some_and(|archive| archive.can_compose);
                LINE + if compose { CONTROL_ROW } else { 0 } + chips(None)
            },
            Container::Mint => CONTROL_ROW,
        }
}

/// A container's width in a band `width` wide.
fn container_width(width: u32) -> u32 {
    CONTAINER_WIDTH
        .min(width.saturating_sub(OVERMAP_GAP * 2))
        .max(1)
}

/// The overmap band's height: its containers wrapped into rows at `width`,
/// each row as tall as its tallest container, up to its share of `body`.
fn overmap_band(model: &MereViewModel, width: u32, body: u32) -> u32 {
    if !overmap_shown(model) {
        return 0;
    }
    let per_row = (width.saturating_sub(OVERMAP_GAP) / (container_width(width) + OVERMAP_GAP))
        .max(1) as usize;
    let heights: Vec<u32> = containers(model)
        .iter()
        .map(|container| container_height(model, container))
        .collect();
    let rows: u32 = heights
        .chunks(per_row)
        .map(|row| row.iter().copied().max().unwrap_or(0) + OVERMAP_GAP)
        .sum();
    (OVERMAP_GAP + rows).min((body as f32 * OVERMAP_SHARE) as u32)
}

/// The graph's area, in pixels, for `model` in a tile of `width` by `height`.
fn graph_area(model: &MereViewModel, width: u32, height: u32) -> (u32, u32) {
    if tier_shown(model) {
        // The tier is on the graph's own canvas, which keeps the whole body.
        (width.max(1), body_height(width, height))
    } else if overmap_shown(model) {
        let body = body_height(width, height);
        (
            width.max(1),
            body.saturating_sub(overmap_band(model, width, body)).max(1),
        )
    } else {
        graph_size(width, height, has_session_content(model))
    }
}

/// The area the graph's own nodes are laid out in: the graph's area, less the
/// tier across its top when there is one.
fn catalog_area(
    model: &MereViewModel,
    open: Option<TierCard>,
    width: u32,
    height: u32,
) -> (u32, u32) {
    let (width, height) = graph_area(model, width, height);
    (
        width,
        height
            .saturating_sub(tier_height(model, open, height))
            .max(1),
    )
}

/// The graph's area, in pixels, inside a tile of `width` by `height`.
fn graph_size(width: u32, height: u32, has_sessions: bool) -> (u32, u32) {
    let body = body_height(width, height);
    if wide(width) {
        (
            if has_sessions {
                width.saturating_sub(SESSIONS_WIDTH).max(1)
            } else {
                width.max(1)
            },
            body,
        )
    } else {
        (
            width.max(1),
            if has_sessions {
                ((body as f32 * NARROW_GRAPH_SHARE) as u32).max(1)
            } else {
                body
            },
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

/// A button that reports `request`, carrying `attrs`.
pub(crate) fn request_button<State, Action, Request>(
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

#[path = "view/state_ops.rs"]
mod state_ops;

#[path = "view/methods.rs"]
mod methods;

#[path = "view/sessions.rs"]
mod sessions;

use sessions::{archive_actions, sessions};

pub(crate) use sessions::{compose_button, session_steps, session_words};

#[path = "view/content.rs"]
mod content;

use content::{graph, merge_card, overmap, status_panel};

#[path = "view/build.rs"]
mod build;

pub use build::mere_view;
