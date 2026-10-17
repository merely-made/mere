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
    .mere-view-tier-rule { position: absolute; left: 0; height: 0; border-top: 1px dashed currentColor; opacity: 0.35; pointer-events: none; } \
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
    pub folds: Vec<curation::FoldRecord>,
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

impl MereViewState {
    pub fn apply(&mut self, event: MereViewEvent) {
        match event {
            MereViewEvent::Hover(key) => {
                // Hovering an archive node peeks its popover; leaving it, or
                // hovering anything else, closes a peek (never a pinned one).
                match key.as_deref().and_then(archive_of_node) {
                    Some(archive) => self.apply(MereViewEvent::ArchivePopover(
                        archive.to_string(),
                        DetailPopoverEvent::Hover(true),
                    )),
                    None => {
                        if let Some((open, _)) = self.archive_popover.clone() {
                            self.apply(MereViewEvent::ArchivePopover(
                                open,
                                DetailPopoverEvent::Hover(false),
                            ));
                        }
                    },
                }
                self.hovered = key;
            },
            MereViewEvent::Focus(key) => {
                if key.is_some() {
                    self.focus_return = None;
                }
                self.focused = key;
            },
            MereViewEvent::Expand(card) => {
                self.focus_return = match (self.expanded, card) {
                    (Some(closed), None) => Some(card_node_key(closed)),
                    _ => None,
                };
                self.expanded = card;
            },
            MereViewEvent::ArchivePopover(key, event) => {
                let mut popover = match self.archive_popover.take() {
                    Some((open, popover)) if open == key => popover,
                    // A pinned popover stays put while another node is hovered.
                    Some((open, popover))
                        if popover.mode == DetailPopoverMode::Pinned
                            && matches!(event, DetailPopoverEvent::Hover(_)) =>
                    {
                        self.archive_popover = Some((open, popover));
                        return;
                    },
                    _ => DetailPopoverState::default(),
                };
                popover.apply(event);
                if popover.return_focus {
                    popover.return_focus = false;
                    self.focus_return = Some(archive_node_key(&key));
                } else if popover.mode == DetailPopoverMode::Pinned {
                    self.focus_return = None;
                }
                self.archive_popover =
                    (popover.mode != DetailPopoverMode::Hidden).then_some((key, popover));
            },
            MereViewEvent::MergeDraft { key, seed, command } => {
                self.merge_drafts
                    .entry(key)
                    .or_insert_with(|| TextInput::new(seed))
                    .apply(command);
            },
            MereViewEvent::SelectArchive(key) => {
                if self.selected_archives.contains(&key) {
                    self.selected_archives.retain(|selected| selected != &key);
                } else {
                    self.selected_archives.push(key);
                }
            },
        }
    }

    /// Apply `event`, then lay the view out again for `model` in a tile of
    /// `width` by `height`: the one call a host makes per event, since
    /// opening or closing a tier card changes the area the graph is laid out
    /// in. Laying out again is free when nothing it reads changed.
    pub fn apply_and_lay_out(
        &mut self,
        event: MereViewEvent,
        model: &MereViewModel,
        width: u32,
        height: u32,
    ) {
        self.apply(event);
        self.lay_out(model, width, height);
    }

    /// Lay the model's graph out for a tile of `width` by `height`, unless its
    /// nodes, its relations, the layout and the size are the ones last laid
    /// out. Call it before rendering whenever any of them may have changed.
    pub fn lay_out(&mut self, model: &MereViewModel, width: u32, height: u32) {
        // A refresh removes vanished archive selections before a later Compose.
        self.selected_archives.retain(|key| {
            model
                .archive
                .as_ref()
                .is_some_and(|archive| archive.entries.iter().any(|entry| &entry.key == key))
        });
        // A card or popover whose session, shelf or archive is gone closes.
        if let Some(card) = self.expanded
            && !card_listed(model, card)
        {
            self.expanded = None;
        }
        if let Some((key, _)) = &self.archive_popover
            && !model
                .archive
                .as_ref()
                .is_some_and(|archive| archive.entries.iter().any(|entry| &entry.key == key))
        {
            self.archive_popover = None;
        }
        // A draft whose conflict is gone, or whose merge is, goes with it.
        self.merge_drafts.retain(|key, _| {
            model
                .merge
                .as_ref()
                .is_some_and(|merge| merge.conflicts.iter().any(|conflict| &conflict.key == key))
        });
        if model.hide_graph {
            // The host's editing surface owns the geometry in panel mode.
            return;
        }
        let size = catalog_area(model, self.expanded, width, height);
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

    /// Where the last layout put the node with `key`, normalized to `0..=1`
    /// within the graph's own area (below the tier, when there is one).
    pub fn position(&self, key: &str) -> Option<(f32, f32)> {
        self.laid.positions.get(key).copied()
    }
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

impl MereView<'_> {
    /// The graph's area, in pixels. A tier, when there is one, is inside it.
    pub fn graph_size(&self) -> (u32, u32) {
        graph_area(self.model, self.width, self.height)
    }

    /// The overmap band's height above the graph, in pixels; zero unless the
    /// sessions show as an overmap.
    pub fn overmap_height(&self) -> u32 {
        overmap_band(self.model, self.width, body_height(self.width, self.height))
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
    pub fn swatch(&self) -> GraphCanvasSwatch<String, MereNodeKind> {
        let GraphModel { nodes, relations } = &self.model.graph;
        let labels: HashMap<&str, &str> = nodes
            .iter()
            .map(|node| (node.key.as_str(), node.label.as_str()))
            .collect();
        let nodes = nodes
            .iter()
            .map(|node| GraphCanvasNode {
                id: node.key.clone(),
                kind: MereNodeKind::Graph(node.state),
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
                    link_key: None,
                    marks: Default::default(),
                    strength: 1.0,
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
        // A narrow tile cannot hold every label; the focused and hovered keep
        // theirs, and every node keeps its accessible name.
        .with_label_culling(true)
        .with_relations(relations)
        .with_relation_lines(
            [
                (Provenance::Extracted, RelationLine::Solid),
                (Provenance::Authored, RelationLine::Heavy),
                (Provenance::Suggested, RelationLine::Dashed),
                (Provenance::Other(SAVED_FROM.into()), RelationLine::Dotted),
                (Provenance::Other(ON_SHELF.into()), RelationLine::Dotted),
            ]
            .into_iter()
            .map(|(kind, line)| (kind.token().to_string(), line))
            .collect(),
        );
        swatch = super::folds::project(swatch, &self.state.folds);
        // Targets as wide as the layout keeps nodes apart, so none holds
        // another's centre.
        swatch.hit_size = SEPARATION;
        swatch.focus = self.state.focused.clone();
        swatch.hovered = self.state.hovered.clone();
        if self.tier_height() > 0 {
            swatch = self.with_tier(swatch);
        }
        swatch
    }

    /// The paint leaf the host registers under [`leaf_key`](Self::leaf_key),
    /// with its palette for each node state.
    /// A palette that knows only graph node states can take
    /// [`MereNodeKind::state`].
    pub fn paint_leaf(&self, palette: impl Fn(&MereNodeKind) -> ColorF) -> GraphCanvas {
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
    let band = view.overmap_height();
    let mut notice = el::<_, State, Action>("p", model.notice.clone().unwrap_or_default())
        .attr("class", "mere-view-notice")
        .attr("role", "status")
        .attr("aria-live", "polite");
    if band > 0 {
        // Below the band, over the top of the graph, where it covers no
        // container.
        notice = notice.attr("style", format!("top:{}px", band + 8));
    } else if tier_shown(model) {
        // Over the foot of the canvas, clear of the tier across its top.
        notice = notice.attr("style", "top:auto;bottom:8px");
    }
    let content = match model.status.note() {
        None if model.hide_graph => {
            Box::new(el::<_, State, Action>("div", ())) as MereViewElement<State, Action>
        },
        None => graph(&view, on_change.clone(), on_request.clone()),
        Some(note) => status_panel(model.status.token(), note, on_request.clone()),
    };
    // The notice rides over the top of the graph's area, where it covers no
    // control and needs no room of its own.
    let (width, height) = if model.hide_graph {
        (
            view.width,
            if model.status.note().is_some() {
                128
            } else {
                32
            },
        )
    } else {
        view.graph_size()
    };
    let layer: MereViewElement<State, Action> = if band > 0 {
        overmap(&view, band, on_change.clone(), on_request.clone())
    } else {
        Box::new(el::<_, State, Action>("div", ()).attr("style", "display:none"))
    };
    let main = el::<_, State, Action>("div", (notice, layer, content))
        .attr("class", "mere-view-main")
        .attr(
            "style",
            format!("width:{width}px;height:{}px", height + band),
        );
    let merge: MereViewElement<State, Action> = match &model.merge {
        Some(merge) => merge_card(&view, merge, on_change.clone(), on_request.clone()),
        None => Box::new(el::<_, State, Action>("div", ()).attr("style", "display:none")),
    };
    let body = el::<_, State, Action>(
        "div",
        (sessions(&view, on_change, on_request.clone()), main),
    )
    .attr("class", "mere-view-body");
    Box::new(
        el::<_, State, Action>("section", (bar(&view, on_request), body, merge))
            .attr("class", "mere-view")
            .attr(
                "style",
                format!("width:{}px;height:{}px", view.width, view.height),
            )
            .attr("role", "region")
            .attr("aria-label", format!("Mere: {}", model.title))
            .attr(
                "data-width",
                if wide(view.width) { "wide" } else { "narrow" },
            )
            .attr("data-status", model.status.token())
            .attr(
                "data-graph",
                if model.hide_graph { "hidden" } else { "shown" },
            )
            .attr("data-sessions", model.placement.token()),
    )
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

fn sessions<State, Action, Change, Request>(
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
    // Minting heads the list, beside the other lifecycle steps.
    let mint: Option<MereViewElement<State, Action>> = view.model.can_mint.then(|| {
        Box::new(
            el::<_, State, Action>(
                "li",
                request_button(
                    "New session",
                    MereViewRequest::Mint,
                    on_request.clone(),
                    vec![("data-request", "mint".into())],
                ),
            )
            .attr("class", "mere-view-mint"),
        ) as MereViewElement<State, Action>
    });
    let items: Vec<MereViewElement<State, Action>> = mint
        .into_iter()
        .chain(view.model.sessions.iter().map(|session| {
            session_item(
                session,
                view.model
                    .archive
                    .as_ref()
                    .is_some_and(|archive| archive.save_sessions.contains(&session.id)),
                on_request.clone(),
            )
        }))
        .chain(archive_items(view, on_change, on_request.clone()))
        .collect();
    // A band or tier holds the sessions; the hidden column then holds no
    // second copy of their controls.
    let items = if overmap_shown(view.model) || tier_shown(view.model) {
        Vec::new()
    } else {
        items
    };
    let size = if !has_session_content(view.model)
        || overmap_shown(view.model)
        || tier_shown(view.model)
    {
        "display:none".to_string()
    } else if view.model.hide_graph {
        "width:100%;flex:1;min-height:0;max-height:none".to_string()
    } else if wide(view.width) {
        format!("width:{SESSIONS_WIDTH}px")
    } else {
        let body = body_height(view.width, view.height);
        let graph = graph_size(view.width, view.height, true).1;
        format!("max-height:{}px", body.saturating_sub(graph))
    };
    Box::new(
        el::<_, State, Action>("ul", items)
            .attr("class", "mere-view-sessions")
            .attr("aria-label", "Sessions")
            .attr("style", size),
    )
}

/// A session's state in words: whether it is shown here or trashed, and the
/// host's detail.
pub(crate) fn session_words(session: &SessionEntry) -> String {
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
    words.join(", ")
}

fn session_item<State, Action, Request>(
    session: &SessionEntry,
    can_save: bool,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let name = el::<_, State, Action>("span", session.name.clone())
        .attr("class", "mere-view-session-name");
    let state = el::<_, State, Action>("span", session_words(session))
        .attr("class", "mere-view-session-state");
    let steps = el::<_, State, Action>("div", session_steps(session, can_save, on_request))
        .attr("class", "mere-view-session-steps");
    Box::new(
        el::<_, State, Action>("li", (name, state, steps))
            .attr("class", "mere-view-session")
            .attr("data-session", session.id.as_uuid().to_string())
            .attr("data-attached", session.attached.to_string())
            .attr("data-trashed", session.trashed.to_string()),
    )
}

/// The steps the host offers on `session`, and Save when the archive takes it.
pub(crate) fn session_steps<State, Action, Request>(
    session: &SessionEntry,
    can_save: bool,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let mut steps: Vec<MereViewElement<State, Action>> = session
        .steps
        .iter()
        .map(|&step| {
            request_button(
                step.label(),
                MereViewRequest::Session(session.id, step),
                on_request.clone(),
                vec![
                    ("data-step", step.token().into()),
                    (
                        "data-session-step",
                        format!("{}:{}", step.token(), session.id.as_uuid()),
                    ),
                    ("aria-label", format!("{} {}", step.label(), session.name)),
                ],
            )
        })
        .collect();
    if can_save {
        steps.push(request_button(
            "Save to archive",
            MereViewRequest::Archive(ArchiveRequest::Save(session.id)),
            on_request,
            vec![
                ("data-step", "save-archive".into()),
                ("data-attached", session.attached.to_string()),
                ("aria-label", format!("Save {} to archive", session.name)),
            ],
        ));
    }
    steps
}

fn archive_items<State, Action, Change, Request>(
    view: &MereView<'_>,
    on_change: Change,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let Some(archive) = &view.model.archive else {
        return Vec::new();
    };
    let mut result = vec![Box::new(
        el::<_, State, Action>("li", "Archive").attr("class", "mere-view-archive-heading"),
    ) as MereViewElement<State, Action>];
    if archive.can_compose {
        result.push(Box::new(el::<_, State, Action>(
            "li",
            compose_button(view, on_request.clone()),
        )));
    }
    for entry in &archive.entries {
        let actions = archive_actions(
            view,
            entry,
            "Select for composition",
            on_change.clone(),
            on_request.clone(),
        );
        result.push(Box::new(
            el::<_, State, Action>(
                "li",
                (
                    el::<_, State, Action>("span", entry.name.clone()),
                    el::<_, State, Action>("span", entry.detail.clone()),
                    el::<_, State, Action>("div", actions).attr("class", "mere-view-archive-steps"),
                ),
            )
            .attr("class", "mere-view-archive")
            .attr("data-archive", entry.key.clone()),
        ));
    }
    result
}

/// Compose, enabled once two archives are selected.
pub(crate) fn compose_button<State, Action, Request>(
    view: &MereView<'_>,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let selected = view.state.selected_archives.clone();
    let mut compose = button("Compose selected archives", move |state: &mut State, _| {
        if selected.len() >= 2 {
            on_request(
                state,
                MereViewRequest::Archive(ArchiveRequest::Compose(selected.clone())),
            )
        }
    })
    .attr(
        "aria-disabled",
        (view.state.selected_archives.len() < 2).to_string(),
    )
    .attr("data-request", "compose-archives");
    if view.state.selected_archives.len() < 2 {
        compose = compose.attr("disabled", "");
    }
    Box::new(compose)
}

/// An archive's controls: open it as a session, and select it for
/// composition, labelled `select_label` on screen.
fn archive_actions<State, Action, Change, Request>(
    view: &MereView<'_>,
    entry: &ArchiveEntry,
    select_label: &str,
    on_change: Change,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let mut actions = Vec::new();
    if entry.can_open {
        actions.push(request_button(
            "Open as session",
            MereViewRequest::Archive(ArchiveRequest::Open(entry.key.clone())),
            on_request,
            vec![
                ("data-step", "open-archive".into()),
                ("data-archive", entry.key.clone()),
                ("aria-label", format!("Open {} as session", entry.name)),
            ],
        ));
    }
    if view
        .model
        .archive
        .as_ref()
        .is_some_and(|archive| archive.can_compose)
    {
        let key = entry.key.clone();
        actions.push(Box::new(
            button(select_label, move |state: &mut State, _| {
                on_change(state, MereViewEvent::SelectArchive(key.clone()))
            })
            .attr("data-step", "select-archive")
            .attr(
                "aria-label",
                format!("Select {} for composition", entry.name),
            )
            .attr(
                "aria-pressed",
                view.state
                    .selected_archives
                    .contains(&entry.key)
                    .to_string(),
            ),
        ));
    }
    actions
}

/// The overmap band: each session a container node with the archives saved
/// from it on it, the archive shelf, and minting last.
fn overmap<State, Action, Change, Request>(
    view: &MereView<'_>,
    band: u32,
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
    let width = container_width(view.width);
    let chips = |session: Option<&SessionEntry>| -> MereViewElement<State, Action> {
        let items: Vec<MereViewElement<State, Action>> = archives_on(model, session)
            .into_iter()
            .map(|entry| {
                let steps =
                    archive_actions(view, entry, "Select", on_change.clone(), on_request.clone());
                Box::new(
                    el::<_, State, Action>(
                        "li",
                        (
                            el::<_, State, Action>("span", entry.name.clone())
                                .attr("class", "mere-view-archive-name")
                                .attr("title", entry.detail.clone()),
                            el::<_, State, Action>("div", steps)
                                .attr("class", "mere-view-archive-steps"),
                        ),
                    )
                    .attr("class", "mere-view-archive")
                    .attr("data-archive", entry.key.clone())
                    .attr("style", format!("height:{}px", chip_height(model, entry))),
                ) as MereViewElement<State, Action>
            })
            .collect();
        Box::new(
            el::<_, State, Action>("ul", items)
                .attr("class", "mere-view-container-archives")
                .attr("aria-label", "Archives"),
        )
    };
    let items: Vec<MereViewElement<State, Action>> = containers(model)
        .iter()
        .map(|container| {
            let size = format!(
                "width:{width}px;height:{}px",
                container_height(model, container)
            );
            match container {
                Container::Session(session) => Box::new(
                    el::<_, State, Action>(
                        "section",
                        (
                            el::<_, State, Action>("span", session.name.clone())
                                .attr("class", "mere-view-session-name"),
                            el::<_, State, Action>("span", session_words(session))
                                .attr("class", "mere-view-session-state"),
                            el::<_, State, Action>(
                                "div",
                                session_steps(
                                    session,
                                    can_save(model, session),
                                    on_request.clone(),
                                ),
                            )
                            .attr("class", "mere-view-session-steps"),
                            chips(Some(*session)),
                        ),
                    )
                    .attr("class", "mere-view-container mere-view-session")
                    .attr("data-container", "session")
                    .attr("role", "group")
                    .attr("aria-label", format!("Session {}", session.name))
                    .attr("data-session", session.id.as_uuid().to_string())
                    .attr("data-attached", session.attached.to_string())
                    .attr("data-trashed", session.trashed.to_string())
                    .attr("style", size),
                ) as MereViewElement<State, Action>,
                Container::Shelf => {
                    let compose: Vec<MereViewElement<State, Action>> = model
                        .archive
                        .as_ref()
                        .is_some_and(|archive| archive.can_compose)
                        .then(|| compose_button(view, on_request.clone()))
                        .into_iter()
                        .collect();
                    Box::new(
                        el::<_, State, Action>(
                            "section",
                            (
                                el::<_, State, Action>("span", "Archive")
                                    .attr("class", "mere-view-container-title"),
                                el::<_, State, Action>("div", compose)
                                    .attr("class", "mere-view-session-steps"),
                                chips(None),
                            ),
                        )
                        .attr("class", "mere-view-container mere-view-shelf")
                        .attr("data-container", "shelf")
                        .attr("role", "group")
                        .attr("aria-label", "Archive")
                        .attr("style", size),
                    )
                },
                Container::Mint => Box::new(
                    el::<_, State, Action>(
                        "div",
                        request_button(
                            "New session",
                            MereViewRequest::Mint,
                            on_request.clone(),
                            vec![("data-request", "mint".into())],
                        ),
                    )
                    .attr("class", "mere-view-container mere-view-mint")
                    .attr("data-container", "mint")
                    .attr("style", size),
                ),
            }
        })
        .collect();
    Box::new(
        el::<_, State, Action>("div", items)
            .attr("class", "mere-view-overmap")
            .attr("role", "group")
            .attr("aria-label", "Overmap: sessions and archives")
            .attr("style", format!("width:{}px;height:{band}px", view.width)),
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
    let focus = {
        let on_change = on_change.clone();
        move |state: &mut State, key: Option<String>| on_change(state, MereViewEvent::Focus(key))
    };
    // A tier node opens its own widget; every other node is the host's.
    let tier = tier_shown(view.model);
    let model = view.model;
    let cards: Vec<TierCard> = model
        .sessions
        .iter()
        .map(|session| TierCard::Session(session.id))
        .chain([TierCard::Shelf])
        .filter(|card| card_listed(model, *card))
        .collect();
    let archives: Vec<String> = view
        .model
        .archive
        .iter()
        .flat_map(|archive| archive.entries.iter().map(|entry| entry.key.clone()))
        .collect();
    let expanded = view.state.expanded;
    let activate = {
        let on_change = on_change.clone();
        let on_request = on_request.clone();
        move |state: &mut State, key: String| {
            if tier {
                if let Some((card, toggles)) =
                    card_of_node(&key).filter(|(card, _)| cards.contains(card))
                {
                    let open = (!toggles || expanded != Some(card)).then_some(card);
                    return on_change(state, MereViewEvent::Expand(open));
                }
                if let Some(archive) =
                    archive_of_node(&key).filter(|key| archives.iter().any(|a| a == key))
                {
                    return on_change(
                        state,
                        MereViewEvent::ArchivePopover(
                            archive.to_string(),
                            DetailPopoverEvent::TogglePinned,
                        ),
                    );
                }
            }
            on_request(state, MereViewRequest::Activate(key))
        }
    };
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
    let (card, widgets) = if tier {
        (
            tier_card(view, &swatch, on_change.clone(), on_request.clone()),
            tier_widgets(view, &swatch, on_change, on_request),
        )
    } else {
        (Keyed::new(Vec::new()), Vec::new())
    };
    Box::new(
        el::<_, State, Action>("div", (canvas, card, widgets))
            .attr("class", "mere-view-graph")
            .attr("data-tier", if tier { "shown" } else { "none" }),
    )
}

/// The pending merge's card, over the view below its bar: each conflict with
/// its three values, its choices and a hand value to write, then Apply and
/// Cancel.
fn merge_card<State, Action, Change, Request>(
    view: &MereView<'_>,
    merge: &MergeModel,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let top = if wide(view.width) {
        BAR_HEIGHT
    } else {
        NARROW_BAR_HEIGHT
    } + 8;
    let title = el::<_, State, Action>(
        "h2",
        format!("Merge {} into {}", merge.source_name, merge.target_name),
    )
    .attr("class", "mere-view-merge-title");
    let dropped: MereViewElement<State, Action> = if merge.dropped.is_empty() {
        Box::new(el::<_, State, Action>("div", ()).attr("style", "display:none"))
    } else {
        let items: Vec<MereViewElement<State, Action>> = merge
            .dropped
            .iter()
            .map(|words| {
                Box::new(el::<_, State, Action>("li", words.clone()))
                    as MereViewElement<State, Action>
            })
            .collect();
        Box::new(
            el::<_, State, Action>("ul", items)
                .attr("class", "mere-view-merge-dropped")
                .attr("role", "status")
                .attr("aria-label", "Dropped choices"),
        )
    };
    let conflicts: Vec<MereViewElement<State, Action>> = merge
        .conflicts
        .iter()
        .map(|conflict| merge_conflict(view, conflict, on_change.clone(), on_request.clone()))
        .collect();
    let list = el::<_, State, Action>("ul", conflicts)
        .attr("class", "mere-view-merge-conflicts")
        .attr("aria-label", "Conflicts");
    let resolved = merge.resolved();
    let apply: MereViewElement<State, Action> = if resolved {
        request_button(
            "Apply",
            MereViewRequest::Merge(MergeRequest::Apply),
            on_request.clone(),
            vec![("aria-label", "Apply merge".to_string())],
        )
    } else {
        Box::new(
            button("Apply", |_: &mut State, _| {})
                .attr("aria-label", "Apply merge")
                .attr("aria-disabled", "true"),
        )
    };
    let cancel = request_button(
        "Cancel",
        MereViewRequest::Merge(MergeRequest::Cancel),
        on_request,
        vec![("aria-label", "Cancel merge".to_string())],
    );
    let actions =
        el::<_, State, Action>("div", (apply, cancel)).attr("class", "mere-view-merge-actions");
    Box::new(
        el::<_, State, Action>("section", (title, dropped, list, actions))
            .attr("class", "mere-view-merge")
            .attr("style", format!("top:{top}px"))
            .attr("role", "group")
            .attr(
                "aria-label",
                format!(
                    "Pending merge: {} into {}",
                    merge.source_name, merge.target_name
                ),
            )
            .attr("data-resolved", resolved.to_string()),
    )
}

/// One conflict: its part, its base, source and target values, its choices
/// and the hand value field.
fn merge_conflict<State, Action, Change, Request>(
    view: &MereView<'_>,
    conflict: &MergeConflictEntry,
    on_change: Change,
    on_request: Request,
) -> MereViewElement<State, Action>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let words = |value: &str| {
        if value.is_empty() {
            "absent".to_string()
        } else {
            value.to_string()
        }
    };
    let seed = match &conflict.chosen {
        Some(MergeOption::Hand(text)) => text.clone(),
        _ => conflict.target.clone(),
    };
    let draft = view
        .state
        .merge_drafts
        .get(&conflict.key)
        .cloned()
        .unwrap_or_else(|| TextInput::new(seed.clone()));
    let label = &conflict.label;
    let mut options = vec![
        (
            MergeOption::Source,
            "Source",
            format!("Take the source's {label}"),
        ),
        (
            MergeOption::Target,
            "Target",
            format!("Keep the target's {label}"),
        ),
        (
            MergeOption::Hand(draft.text().to_string()),
            "Edit by hand",
            format!("Use the hand value for {label}"),
        ),
    ];
    if conflict.add_as_new {
        options.push((
            MergeOption::AddAsNew,
            "Add as new",
            format!("Add {label} as new"),
        ));
    }
    let chosen = conflict.chosen.as_ref().map(MergeOption::token);
    let choices: Vec<MereViewElement<State, Action>> = options
        .into_iter()
        .map(|(option, text, aria)| {
            let pressed = chosen == Some(option.token());
            request_button(
                text,
                MereViewRequest::Merge(MergeRequest::Choose(conflict.key.clone(), option.clone())),
                on_request.clone(),
                vec![
                    ("aria-label", aria),
                    ("aria-pressed", pressed.to_string()),
                    ("data-choice", option.token().to_string()),
                ],
            )
        })
        .collect();
    let key = conflict.key.clone();
    let field = on_key(
        el::<_, State, Action>("div", caret_field_children::<State, Action>(&draft, &[]))
            .attr("class", "mere-view-merge-hand")
            .attr("role", "textbox")
            .attr("aria-label", format!("Hand value for {label}"))
            .attr("data-cambium-text-value", draft.text().to_string()),
        move |state: &mut State, event: KeyEvent| {
            if let Some(command) = text_command_from_key(&event, TextFieldMode::SingleLine) {
                on_change(
                    state,
                    MereViewEvent::MergeDraft {
                        key: key.clone(),
                        seed: seed.clone(),
                        command,
                    },
                );
            }
        },
    );
    let line =
        |class: &'static str, text: String| el::<_, State, Action>("p", text).attr("class", class);
    Box::new(
        el::<_, State, Action>(
            "li",
            (
                line("mere-view-merge-part", label.clone()),
                line(
                    "mere-view-merge-base",
                    format!("Base: {}", words(&conflict.base)),
                ),
                line(
                    "mere-view-merge-source",
                    format!("Source: {}", words(&conflict.source)),
                ),
                line(
                    "mere-view-merge-target",
                    format!("Target: {}", words(&conflict.target)),
                ),
                el::<_, State, Action>("div", choices).attr("class", "mere-view-merge-choices"),
                field,
            ),
        )
        .attr("class", "mere-view-merge-conflict")
        .attr("data-key", conflict.key.clone())
        .attr("data-chosen", chosen.unwrap_or("none")),
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
