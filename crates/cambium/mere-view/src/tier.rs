/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The overmap tier: a mere's sessions and archives as nodes on the graph
//! canvas itself, in a tier across its top, one level up from the graph.
//!
//! Each kind of tier node has its own widget:
//! - a session node, and the archive shelf's node, opens in place into its
//!   card (a session's steps, Save to archive and its archives; the shelf's
//!   Compose and its archives), declared to the canvas as the node's
//!   footprint so the routes that reach it end at the card's border. One card
//!   is open at a time, and the tier grows to hold it while the graph's own
//!   nodes are laid out again below.
//! - an archive node peeks its detail on hover and pins a popover on click
//!   (Open as session, Select for composition, Compose), through Cambium's
//!   [`detail_popover_panel`]; Escape or an outside click dismisses it.
//! - past the host's [`MereViewModel::max_archive_nodes`] archives (by
//!   default [`DEFAULT_MAX_ARCHIVE_NODES`]), a session's or the shelf's archives
//!   collapse into one "+N" node that opens the card holding them.
//!
//! New session and Compose are controls of the tier itself, at its right end.
//! The canvas's own targets, gestures and paint carry every tier node; the
//! tier adds nodes, relations, a footprint and per-node attributes to the
//! swatch, never a second hit layer under them.

use cambium::{
    DetailPopoverEvent, DetailPopoverMode, GraphCanvasNode, GraphCanvasRelation, GraphCanvasSwatch,
    GraphLabelPlacement, Key, Keyed, NamedKey, OverlaySurface, Placement, button,
    detail_popover_panel, el, on_key, request_focus,
};
use incipit::SessionId;
use uuid::Uuid;

use crate::model::{ArchiveEntry, MereNodeKind, MereViewModel, SessionEntry, TierCard};
use crate::view::{
    ArchiveRequest, CONTAINER_PAD, LINE, MereView, MereViewElement, MereViewEvent, MereViewRequest,
    archives_on, can_save, compose_button, control_rows, has_session_content, request_button,
    session_steps, session_words,
};

/// The tier's resting height across the canvas's top, and the most of the
/// canvas it takes at rest when the canvas is short.
const TIER_HEIGHT: u32 = 112;
const TIER_SHARE: f32 = 0.45;
/// The most of the canvas the tier grows to while a card is open.
const OPEN_TIER_SHARE: f32 = 0.7;
/// The width the tier's own controls take at its right end.
const TIER_CONTROLS_WIDTH: u32 = 148;
/// An open card's width, its gap from the canvas's top, and one archive
/// chip's row in it.
const CARD_WIDTH: u32 = 232;
const CARD_TOP: u32 = 8;
const CHIP_ROW: u32 = 28;
/// The heights [`MERE_VIEW_CSS`](crate::MERE_VIEW_CSS) pins for an open card
/// and an archive popover, so their computed heights are their rows exactly:
/// the card's border, its head (name and Close), a control button, the gap
/// between and below rows of controls, and the popover panel's border.
pub(crate) const CARD_BORDER: u32 = 2;
pub(crate) const CARD_HEAD: u32 = 34;
pub(crate) const BUTTON_HEIGHT: u32 = 30;
pub(crate) const ROW_GAP: u32 = 4;
pub(crate) const PANEL_BORDER: u32 = 1;
/// An archive popover's width.
const POPOVER_WIDTH: f32 = 232.0;
/// The gap between an open card and the archive nodes it moves aside, and the
/// room an archive node's target takes below its centre.
const CARD_CLEARANCE: u32 = 22;
const ARCHIVE_ROOM: u32 = 24;

/// The most archives a session or the shelf shows one by one unless the host
/// says otherwise ([`MereViewModel::max_archive_nodes`]); past it they
/// collapse into one "+N" node that opens the card holding them.
pub const DEFAULT_MAX_ARCHIVE_NODES: usize = 3;

fn max_archive_nodes(model: &MereViewModel) -> usize {
    model.max_archive_nodes.unwrap_or(DEFAULT_MAX_ARCHIVE_NODES)
}

/// The relation kinds joining an archive node to the session it was saved
/// from, and to the shelf: each relation cell's `data-kind`.
pub const SAVED_FROM: &str = "saved-from";
pub const ON_SHELF: &str = "on-shelf";

/// The key prefixes and keys of tier nodes, so a tier node's `data-key`
/// never reads as a graph node's.
pub const SESSION_NODE_PREFIX: &str = "mere-session:";
pub const ARCHIVE_NODE_PREFIX: &str = "mere-archive:";
pub const MORE_NODE_PREFIX: &str = "mere-more:";
pub const SHELF_NODE_KEY: &str = "mere-shelf";

/// The `data-key` of a session's tier node.
pub fn session_node_key(id: SessionId) -> String {
    format!("{SESSION_NODE_PREFIX}{}", id.as_uuid())
}

/// The `data-key` of an archive's tier node.
pub fn archive_node_key(key: &str) -> String {
    format!("{ARCHIVE_NODE_PREFIX}{key}")
}

/// The `data-key` of the "+N" node standing for a card's archives.
pub fn more_node_key(card: TierCard) -> String {
    format!("{MORE_NODE_PREFIX}{}", card_token(card))
}

/// The `data-key` of the node a card opens on.
pub fn card_node_key(card: TierCard) -> String {
    match card {
        TierCard::Session(id) => session_node_key(id),
        TierCard::Shelf => SHELF_NODE_KEY.to_string(),
    }
}

fn card_token(card: TierCard) -> String {
    match card {
        TierCard::Session(id) => id.as_uuid().to_string(),
        TierCard::Shelf => "shelf".to_string(),
    }
}

/// The card a tier node opens, and whether activating it toggles that card
/// (a session's or the shelf's own node) or only opens it (a "+N" node).
pub(crate) fn card_of_node(key: &str) -> Option<(TierCard, bool)> {
    let session = |id: &str| Uuid::parse_str(id).ok().map(SessionId::from_uuid);
    if key == SHELF_NODE_KEY {
        return Some((TierCard::Shelf, true));
    }
    if let Some(id) = key.strip_prefix(SESSION_NODE_PREFIX) {
        return session(id).map(|id| (TierCard::Session(id), true));
    }
    let more = key.strip_prefix(MORE_NODE_PREFIX)?;
    if more == "shelf" {
        Some((TierCard::Shelf, false))
    } else {
        session(more).map(|id| (TierCard::Session(id), false))
    }
}

/// The archive a tier node key names, if it names one.
pub(crate) fn archive_of_node(key: &str) -> Option<&str> {
    key.strip_prefix(ARCHIVE_NODE_PREFIX)
}

/// Whether the sessions show as a tier on the graph's own canvas.
pub(crate) fn tier_shown(model: &MereViewModel) -> bool {
    model.placement == crate::model::SessionPlacement::Tier
        && !model.hide_graph
        && has_session_content(model)
}

/// Whether the tier shows `card`'s node: a session the model lists, or the
/// shelf while any archive sits on it.
pub(crate) fn card_listed(model: &MereViewModel, card: TierCard) -> bool {
    match card {
        TierCard::Session(id) => model.sessions.iter().any(|session| session.id == id),
        TierCard::Shelf => !archives_on(model, None).is_empty(),
    }
}

/// The archives on a card: a session's, or the shelf's.
fn card_archives(model: &MereViewModel, card: TierCard) -> Vec<&ArchiveEntry> {
    match card {
        TierCard::Session(id) => model
            .sessions
            .iter()
            .find(|session| session.id == id)
            .map(|session| archives_on(model, Some(session)))
            .unwrap_or_default(),
        TierCard::Shelf => archives_on(model, None),
    }
}

fn can_compose(model: &MereViewModel) -> bool {
    model
        .archive
        .as_ref()
        .is_some_and(|archive| archive.can_compose)
}

/// The tier's resting height in a graph area `height` tall.
fn resting_height(height: u32) -> u32 {
    TIER_HEIGHT.min((height as f32 * TIER_SHARE) as u32)
}

/// The tier's height in a graph area `height` tall with `open` open; zero
/// with no tier. An open card grows the tier to hold it and the archive nodes
/// below it, up to its share of the canvas.
pub(crate) fn tier_height(model: &MereViewModel, open: Option<TierCard>, height: u32) -> u32 {
    if !tier_shown(model) {
        return 0;
    }
    let rest = resting_height(height);
    match open.filter(|card| card_listed(model, *card)) {
        Some(card) => {
            let needed = CARD_TOP + card_height(model, card) + CARD_CLEARANCE + ARCHIVE_ROOM;
            rest.max(needed.min((height as f32 * OPEN_TIER_SHARE) as u32))
        },
        None => rest,
    }
}

/// How far below an open card's top its archive chips start: its head (name
/// and Close), and a session's state and steps or the shelf's Compose.
fn chips_top(model: &MereViewModel, card: TierCard) -> u32 {
    CARD_BORDER
        + CONTAINER_PAD
        + CARD_HEAD
        + match card {
            TierCard::Session(id) => model
                .sessions
                .iter()
                .find(|session| session.id == id)
                .map_or(0, |session| {
                    let steps = session.steps.len() + usize::from(can_save(model, session));
                    LINE + controls_height(control_rows(steps) as usize)
                }),
            TierCard::Shelf => {
                if can_compose(model) {
                    controls_height(1)
                } else {
                    0
                }
            },
        }
}

/// A block of `rows` rows of controls: each [`BUTTON_HEIGHT`] tall, with
/// [`ROW_GAP`] between rows and below the last, as the sheet pins them.
fn controls_height(rows: usize) -> u32 {
    ROW_GAP + rows as u32 * BUTTON_HEIGHT + rows.saturating_sub(1) as u32 * ROW_GAP
}

/// An open card's height, exactly its rows: its border and padding, its head,
/// its controls and a row per archive. Every row's height is pinned by
/// [`MERE_VIEW_CSS`](crate::MERE_VIEW_CSS), so no host's button style changes it.
pub(crate) fn card_height(model: &MereViewModel, card: TierCard) -> u32 {
    chips_top(model, card)
        + card_archives(model, card).len() as u32 * CHIP_ROW
        + CONTAINER_PAD
        + CARD_BORDER
}

/// A visible label's width as the canvas estimates it, at its 10 px size.
fn label_width(text: &str) -> f32 {
    (text.chars().count() as f32 * 5.6 + 2.0).min(160.0)
}

/// A tier node's accessible name: a session's in words, an archive's own.
fn session_label(session: &SessionEntry) -> String {
    let mut label = format!("Session {}", session.name);
    if session.attached {
        label.push_str(", shown here");
    }
    if session.trashed {
        label.push_str(", in the trash");
    }
    label
}

fn archive_label(entry: &ArchiveEntry, held: bool) -> String {
    if held {
        entry.name.clone()
    } else {
        format!("{}, in the archive", entry.name)
    }
}

/// Normalized canvas coordinates for a leaf-local point, the inverse of the
/// canvas's projection at its default viewport.
fn unit(point: (f32, f32), size: (f32, f32), inset: f32) -> (f32, f32) {
    let span = |extent: f32| (extent - 2.0 * inset).max(1.0);
    (
        ((point.0 - inset) / span(size.0)).clamp(0.0, 1.0),
        ((point.1 - inset) / span(size.1)).clamp(0.0, 1.0),
    )
}

/// The canvas position of a graph node the layout placed at `laid` within
/// the area below the tier.
fn below_tier(laid: (f32, f32), size: (f32, f32), tier: f32, inset: f32) -> (f32, f32) {
    let below = (size.0, (size.1 - tier).max(1.0));
    let span = |extent: f32| (extent - 2.0 * inset).max(0.0);
    let point = (
        inset + laid.0 * span(below.0),
        tier + inset + laid.1 * span(below.1),
    );
    unit(point, size, inset)
}

impl MereView<'_> {
    /// The tier's height across the top of the graph's canvas, in pixels:
    /// taller while a card is open, zero unless the sessions show as a tier.
    pub fn tier_height(&self) -> u32 {
        tier_height(self.model, self.state.expanded, self.graph_size().1)
    }

    /// The open card, if the tier still shows its node.
    pub(crate) fn open_card(&self) -> Option<TierCard> {
        self.state
            .expanded
            .filter(|card| tier_shown(self.model) && card_listed(self.model, *card))
    }

    /// Add the tier's nodes, relations, open card and attributes to `swatch`.
    pub(crate) fn with_tier(
        &self,
        mut swatch: GraphCanvasSwatch<String, MereNodeKind>,
    ) -> GraphCanvasSwatch<String, MereNodeKind> {
        let model = self.model;
        let size = (swatch.width as f32, swatch.height as f32);
        let inset = swatch.node_radius + swatch.edge_width;
        let tier = self.tier_height() as f32;
        // The graph's own nodes were laid out in the area below the tier.
        for node in &mut swatch.graph.nodes {
            node.position = below_tier(node.position, size, tier, inset);
        }
        let rest = resting_height(swatch.height) as f32;
        let (card_y, archive_y) = (rest * 0.28, rest * 0.74);
        let controls = if model.can_mint || can_compose(model) {
            TIER_CONTROLS_WIDTH as f32
        } else {
            0.0
        };
        let open = self.open_card();
        let popover = self
            .state
            .archive_popover
            .as_ref()
            .map(|(key, popover)| (key.as_str(), popover.mode));
        let groups: Vec<TierCard> = model
            .sessions
            .iter()
            .map(|session| TierCard::Session(session.id))
            .chain(card_listed(model, TierCard::Shelf).then_some(TierCard::Shelf))
            .collect();
        let slot = (size.0 - controls).max(1.0) / groups.len().max(1) as f32;

        let push = |swatch: &mut GraphCanvasSwatch<String, MereNodeKind>,
                    key: String,
                    kind: MereNodeKind,
                    label: String,
                    at: (f32, f32),
                    attrs: Vec<(&str, String)>| {
            swatch.graph.nodes.push(GraphCanvasNode {
                id: key.clone(),
                kind,
                position: unit(at, size, inset),
                label,
                key: Some(key.clone()),
            });
            for (name, value) in attrs {
                swatch.node_attrs.push((key.clone(), name.into(), value));
            }
        };

        // Each tier node's visible label: its name alone. The accessible name
        // keeps the words ("Session", "shown here", "in the archive").
        let mut visible: Vec<(String, String)> = Vec::new();
        for (index, card) in groups.into_iter().enumerate() {
            let key = card_node_key(card);
            let opened = open == Some(card);
            let height = card_height(model, card) as f32;
            let centre = slot * (index as f32 + 0.5);
            // An open card stands on its node at the top of the grown tier.
            let at = (
                centre,
                if opened {
                    CARD_TOP as f32 + height / 2.0
                } else {
                    card_y
                },
            );
            let (kind, label, mut attrs, relation, owner) = match card {
                TierCard::Session(id) => {
                    let Some(session) = model.sessions.iter().find(|session| session.id == id)
                    else {
                        continue;
                    };
                    if session.attached {
                        // The shown session is the tier's mark: emphasis, a
                        // bold label and `aria-current` on its target.
                        swatch.selected = Some(key.clone());
                    }
                    (
                        MereNodeKind::Session {
                            attached: session.attached,
                            trashed: session.trashed,
                        },
                        session_label(session),
                        vec![
                            ("data-tier", "session".to_string()),
                            ("data-session", id.as_uuid().to_string()),
                            ("data-attached", session.attached.to_string()),
                            ("data-trashed", session.trashed.to_string()),
                        ],
                        SAVED_FROM,
                        format!("Session {}", session.name),
                    )
                },
                TierCard::Shelf => (
                    MereNodeKind::Shelf,
                    "Archive".to_string(),
                    vec![("data-tier", "shelf".to_string())],
                    ON_SHELF,
                    "Archive".to_string(),
                ),
            };
            attrs.push(("aria-expanded", opened.to_string()));
            visible.push((
                key.clone(),
                match card {
                    TierCard::Session(_) => {
                        owner.strip_prefix("Session ").unwrap_or(&owner).to_string()
                    },
                    TierCard::Shelf => "Archive".to_string(),
                },
            ));
            push(&mut swatch, key.clone(), kind, label, at, attrs);
            if opened {
                swatch = swatch.with_node_footprint(key.clone(), CARD_WIDTH as f32, height);
            }

            // The archives sit under their node, or clear below its open card.
            let archives = card_archives(model, card);
            let (row_y, left, span) = if opened {
                let width = (CARD_WIDTH as f32).min(size.0);
                let left = (centre - width / 2.0).clamp(0.0, (size.0 - width).max(0.0));
                (
                    CARD_TOP as f32 + height + CARD_CLEARANCE as f32,
                    left,
                    width,
                )
            } else {
                (archive_y, slot * index as f32, slot)
            };
            let relate = |swatch: &mut GraphCanvasSwatch<String, MereNodeKind>,
                          from: String,
                          id: &str,
                          name: &str| {
                swatch.relations.push(GraphCanvasRelation {
                    link_key: None,
                    marks: Default::default(),
                    strength: 1.0,
                    id: format!("{relation}:{id}"),
                    from,
                    to: key.clone(),
                    kind: relation.to_string(),
                    label: format!("{name} to {owner}"),
                    route: Vec::new(),
                    visible: true,
                    emphasized: false,
                });
            };
            if archives.len() > max_archive_nodes(model) {
                let more = more_node_key(card);
                let label = match card {
                    TierCard::Session(_) => format!("+{} archives of {owner}", archives.len()),
                    TierCard::Shelf => format!("+{} archives in the archive", archives.len()),
                };
                visible.push((more.clone(), format!("+{}", archives.len())));
                push(
                    &mut swatch,
                    more.clone(),
                    MereNodeKind::More,
                    label.clone(),
                    (left + span / 2.0, row_y),
                    vec![
                        ("data-tier", "more".to_string()),
                        ("data-count", archives.len().to_string()),
                        ("aria-expanded", opened.to_string()),
                    ],
                );
                relate(
                    &mut swatch,
                    more,
                    &format!("more:{}", card_token(card)),
                    &label,
                );
                continue;
            }
            let held = matches!(card, TierCard::Session(_));
            for (at, entry) in archives.iter().enumerate() {
                let node = archive_node_key(&entry.key);
                let pinned = popover == Some((entry.key.as_str(), DetailPopoverMode::Pinned));
                visible.push((node.clone(), entry.name.clone()));
                push(
                    &mut swatch,
                    node.clone(),
                    MereNodeKind::Archive { held },
                    archive_label(entry, held),
                    (
                        left + span * (at + 1) as f32 / (archives.len() + 1) as f32,
                        row_y,
                    ),
                    vec![
                        ("data-tier", "archive".to_string()),
                        ("data-archive-node", entry.key.clone()),
                        ("aria-haspopup", "dialog".to_string()),
                        ("aria-expanded", pinned.to_string()),
                    ],
                );
                relate(&mut swatch, node, &entry.key, &entry.name);
            }
        }
        // A label that would run under the tier's controls at its right end
        // goes to its node's left.
        let room = size.0 - controls;
        let clearance = swatch.node_radius + 7.0;
        let positions: Vec<(String, f32)> = swatch
            .projected_positions()
            .into_iter()
            .map(|(id, (x, _))| (id.clone(), x))
            .collect();
        for (key, text) in visible {
            let x = positions
                .iter()
                .find(|(id, _)| *id == key)
                .map_or(0.0, |(_, x)| *x);
            if controls > 0.0 && x + clearance + label_width(&text) > room {
                swatch = swatch.with_label_placement(key.clone(), GraphLabelPlacement::Left);
            }
            swatch = swatch.with_visible_label(key, text);
        }
        // Every node says what it is, the graph's own included.
        let kinds: Vec<(String, &'static str)> = swatch
            .graph
            .nodes
            .iter()
            .map(|node| (node.id.clone(), node.kind.token()))
            .collect();
        for (id, kind) in kinds {
            swatch
                .node_attrs
                .push((id, "data-node-kind".into(), kind.into()));
        }
        swatch.focus_request = self.state.focus_return.clone();
        swatch
    }
}

/// Render the tier's widgets over the canvas: the rule under the tier, its
/// controls and the archive popover. They share the canvas's positioned box,
/// so the swatch's projection places them.
pub(crate) fn tier_widgets<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, MereNodeKind>,
    on_change: Change,
    on_request: Request,
) -> Vec<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let width = swatch.width;
    let tier = view.tier_height();
    let mut widgets: Vec<MereViewElement<State, Action>> = vec![Box::new(
        el::<_, State, Action>("div", ())
            .attr("class", "mere-view-tier-rule")
            .attr("aria-hidden", "true")
            .attr("style", format!("top:{tier}px;width:{width}px")),
    )];
    let mut controls: Vec<MereViewElement<State, Action>> = Vec::new();
    if model.can_mint {
        controls.push(request_button(
            "New session",
            MereViewRequest::Mint,
            on_request.clone(),
            vec![("data-request", "mint".into())],
        ));
    }
    if can_compose(model) {
        controls.push(compose_button(view, on_request.clone()));
    }
    if !controls.is_empty() {
        let left = width.saturating_sub(TIER_CONTROLS_WIDTH) + CONTAINER_PAD;
        widgets.push(Box::new(
            el::<_, State, Action>("div", controls)
                .attr("class", "mere-view-tier-controls")
                .attr("role", "group")
                .attr("aria-label", "Overmap")
                .attr(
                    "style",
                    format!(
                        "left:{left}px;top:{CONTAINER_PAD}px;width:{}px",
                        TIER_CONTROLS_WIDTH - CONTAINER_PAD * 2
                    ),
                ),
        ));
    }
    if let Some(popover) = archive_popover(view, swatch, on_change, on_request) {
        widgets.push(Box::new(
            el::<_, State, Action>("div", popover).attr("class", "mere-view-popover-layer"),
        ));
    }
    widgets
}

/// The open card, at its footprint on the canvas, keyed by the card so moving
/// it to another node builds it afresh (and its Close takes focus again).
pub(crate) fn tier_card<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, MereNodeKind>,
    on_change: Change,
    on_request: Request,
) -> Keyed<String, MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    card(view, swatch, on_change, on_request)
        .into_iter()
        .collect()
}

fn card<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, MereNodeKind>,
    on_change: Change,
    on_request: Request,
) -> Option<(String, MereViewElement<State, Action>)>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let open = view.open_card()?;
    let region = swatch.projected_node_footprint(&card_node_key(open))?;
    let session = match open {
        TierCard::Session(id) => model.sessions.iter().find(|session| session.id == id),
        TierCard::Shelf => None,
    };
    let (title, close_label) = match session {
        Some(session) => (
            session.name.clone(),
            format!("Close session {}", session.name),
        ),
        None => ("Archive".to_string(), "Close the archive".to_string()),
    };
    let close = {
        let on_change = on_change.clone();
        request_focus(
            button("Close", move |state: &mut State, _| {
                on_change(state, MereViewEvent::Expand(None))
            })
            .attr("class", "mere-view-card-close")
            .attr(
                "data-request",
                if session.is_some() {
                    "close-session"
                } else {
                    "close-shelf"
                },
            )
            .attr("aria-label", close_label),
            true,
        )
    };
    let head = el::<_, State, Action>(
        "div",
        (
            el::<_, State, Action>("span", title).attr("class", "mere-view-session-name"),
            close,
        ),
    )
    .attr("class", "mere-view-card-head");
    let popover = view.state.archive_popover.as_ref();
    let chips: Vec<MereViewElement<State, Action>> = card_archives(model, open)
        .into_iter()
        .map(|entry| {
            let key = entry.key.clone();
            let on_change = on_change.clone();
            let pinned = popover.is_some_and(|(shown, popover)| {
                *shown == entry.key && popover.mode == DetailPopoverMode::Pinned
            });
            // A chip takes focus back when its popover closes and it has no
            // node of its own (its archives collapsed into "+N").
            let returns = view.state.focus_return.as_deref()
                == Some(archive_node_key(&entry.key).as_str())
                && !swatch
                    .graph
                    .nodes
                    .iter()
                    .any(|node| node.id == archive_node_key(&entry.key));
            Box::new(
                el::<_, State, Action>(
                    "li",
                    request_focus(
                        button(entry.name.clone(), move |state: &mut State, _| {
                            on_change(
                                state,
                                MereViewEvent::ArchivePopover(
                                    key.clone(),
                                    DetailPopoverEvent::TogglePinned,
                                ),
                            )
                        })
                        .attr("class", "mere-view-archive-chip")
                        .attr("data-archive-chip", entry.key.clone())
                        .attr("aria-haspopup", "dialog")
                        .attr("aria-expanded", pinned.to_string()),
                        returns,
                    ),
                )
                .attr("class", "mere-view-archive"),
            ) as MereViewElement<State, Action>
        })
        .collect();
    let controls: Vec<MereViewElement<State, Action>> = match session {
        Some(session) => vec![
            Box::new(
                el::<_, State, Action>("span", session_words(session))
                    .attr("class", "mere-view-session-state"),
            ),
            Box::new(
                el::<_, State, Action>(
                    "div",
                    session_steps(session, can_save(model, session), on_request),
                )
                .attr("class", "mere-view-session-steps"),
            ),
        ],
        None => can_compose(model)
            .then(|| {
                Box::new(
                    el::<_, State, Action>("div", compose_button(view, on_request))
                        .attr("class", "mere-view-session-steps"),
                ) as MereViewElement<State, Action>
            })
            .into_iter()
            .collect(),
    };
    let body = (
        head,
        controls,
        el::<_, State, Action>("ul", chips)
            .attr("class", "mere-view-container-archives")
            .attr("aria-label", "Archives"),
    );
    let style = format!(
        "left:{}px;top:{}px;width:{}px;height:{}px",
        region.left, region.top, region.width, region.height
    );
    let card = match session {
        Some(session) => el::<_, State, Action>("section", body)
            .attr(
                "class",
                "mere-view-container mere-view-session mere-view-card",
            )
            .attr("data-container", "session")
            .attr("role", "group")
            .attr("aria-label", format!("Session {}", session.name))
            .attr("data-session", session.id.as_uuid().to_string())
            .attr("data-attached", session.attached.to_string())
            .attr("data-trashed", session.trashed.to_string())
            .attr("style", style),
        None => el::<_, State, Action>("section", body)
            .attr(
                "class",
                "mere-view-container mere-view-shelf mere-view-card",
            )
            .attr("data-container", "shelf")
            .attr("role", "group")
            .attr("aria-label", "Archive")
            .attr("style", style),
    };
    Some((
        card_token(open),
        Box::new(
            on_key(card, move |state: &mut State, event| {
                if matches!(event.key, Key::Named(NamedKey::Escape)) {
                    event.prevent_default();
                    on_change(state, MereViewEvent::Expand(None));
                }
            })
            .focusable(false),
        ),
    ))
}

/// Where an archive's popover anchors: its node, or, when its archives
/// collapsed into "+N", its chip in the open card.
fn popover_trigger(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, MereNodeKind>,
    entry: &ArchiveEntry,
) -> Option<((f32, f32, f32, f32), Placement)> {
    let node = archive_node_key(&entry.key);
    if let Some((_, centre)) = swatch
        .projected_positions()
        .into_iter()
        .find(|(id, _)| **id == node)
    {
        let hit = swatch.hit_size;
        return Some((
            (centre.0 - hit / 2.0, centre.1 - hit / 2.0, hit, hit),
            Placement::Below,
        ));
    }
    let open = view.open_card()?;
    let index = card_archives(view.model, open)
        .iter()
        .position(|chip| chip.key == entry.key)?;
    let region = swatch.projected_node_footprint(&card_node_key(open))?;
    let top = region.top + (chips_top(view.model, open) + index as u32 * CHIP_ROW) as f32;
    Some((
        (region.left, top, region.width, CHIP_ROW as f32),
        Placement::RightOf,
    ))
}

/// The open archive popover: its peek, or its pinned controls.
fn archive_popover<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, MereNodeKind>,
    on_change: Change,
    on_request: Request,
) -> Option<MereViewElement<State, Action>>
where
    State: 'static,
    Action: 'static,
    Change: Fn(&mut State, MereViewEvent) + Clone + 'static,
    Request: Fn(&mut State, MereViewRequest) + Clone + 'static,
{
    let model = view.model;
    let (key, popover) = view.state.archive_popover.as_ref()?;
    let archive = model.archive.as_ref()?;
    let entry = archive.entries.iter().find(|entry| &entry.key == key)?;
    let (trigger, placement) = popover_trigger(view, swatch, entry)?;
    let saved_from = entry
        .session
        .and_then(|id| model.sessions.iter().find(|session| session.id == id))
        .map_or_else(
            || "In the archive".to_string(),
            |session| format!("Saved from {}", session.name),
        );
    let can_compose = archive.can_compose;
    let rows = usize::from(entry.can_open) + 2 * usize::from(can_compose);
    let panel_height = PANEL_BORDER * 2
        + CONTAINER_PAD * 2
        + LINE * 3
        + match popover.mode {
            DetailPopoverMode::Pinned => controls_height(rows),
            _ => 0,
        };
    let surface = OverlaySurface::new(
        trigger,
        (POPOVER_WIDTH, panel_height as f32),
        (0.0, 0.0, swatch.width as f32, swatch.height as f32),
    )
    .with_placement(placement)
    .with_label(entry.name.clone());
    let words = || {
        (
            el::<_, State, Action>("span", entry.name.clone())
                .attr("class", "mere-view-archive-name"),
            el::<_, State, Action>("span", saved_from.clone())
                .attr("class", "mere-view-archive-from"),
            el::<_, State, Action>("span", entry.detail.clone())
                .attr("class", "mere-view-archive-detail"),
        )
    };
    let mut controls: Vec<MereViewElement<State, Action>> = Vec::new();
    // The first control takes focus as the popover pins, so Escape and Tab
    // work from the keyboard at once.
    if entry.can_open {
        let request = MereViewRequest::Archive(ArchiveRequest::Open(entry.key.clone()));
        let on_request = on_request.clone();
        let close = on_change.clone();
        let opened = entry.key.clone();
        controls.push(Box::new(request_focus(
            button("Open as session", move |state: &mut State, _| {
                on_request(state, request.clone());
                // Opening leaves the archive for the session it opens, so its
                // popover closes, rather than catching the next click.
                close(
                    state,
                    MereViewEvent::ArchivePopover(
                        opened.clone(),
                        DetailPopoverEvent::Dismiss(cambium::OverlayDismiss::OutsideClick),
                    ),
                )
            })
            .attr("data-step", "open-archive")
            .attr("data-archive", entry.key.clone())
            .attr("aria-label", format!("Open {} as session", entry.name)),
            true,
        )));
    }
    if can_compose {
        let archive_key = entry.key.clone();
        let select = on_change.clone();
        controls.push(Box::new(request_focus(
            button("Select for composition", move |state: &mut State, _| {
                select(state, MereViewEvent::SelectArchive(archive_key.clone()))
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
            !entry.can_open,
        )));
        controls.push(compose_button(view, on_request));
    }
    let content = (
        words(),
        el::<_, State, Action>("div", controls).attr("class", "mere-view-archive-steps"),
    );
    let key = key.clone();
    detail_popover_panel(
        *popover,
        &surface,
        words(),
        content,
        move |state: &mut State, event| {
            on_change(state, MereViewEvent::ArchivePopover(key.clone(), event))
        },
    )
}
