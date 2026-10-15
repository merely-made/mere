/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The overmap tier: a mere's sessions and archives as nodes on the graph
//! canvas itself, in a tier across its top, one level up from the graph.
//!
//! Each kind of tier node has its own widget:
//! - a session node opens in place into its card (its steps, Save to archive
//!   and its archives), declared to the canvas as the node's footprint so the
//!   routes that reach the session end at the card's border. One card is open
//!   at a time.
//! - an archive node peeks its detail on hover and pins a popover on click
//!   (Open as session, Select for composition, Compose), through Cambium's
//!   [`detail_popover_panel`]; Escape or an outside click dismisses it.
//!
//! New session is a control of the tier itself, at its right end. The canvas's
//! own targets, gestures and paint carry every tier node; the tier adds nodes,
//! relations, a footprint and per-node attributes to the swatch, never a
//! second hit layer under them.

use cambium::{
    DetailPopoverEvent, DetailPopoverMode, GraphCanvasNode, GraphCanvasRelation, GraphCanvasSwatch,
    Key, Keyed, NamedKey, OverlaySurface, Placement, button, detail_popover_panel, el, on_key,
    request_focus,
};
use incipit::SessionId;
use uuid::Uuid;

use crate::model::{ArchiveEntry, MereViewModel, NodeState, SessionEntry};
use crate::view::{
    ArchiveRequest, CONTAINER_PAD, CONTROL_ROW, LINE, MereView, MereViewElement, MereViewEvent,
    MereViewRequest, archives_on, can_save, compose_button, control_rows, has_session_content,
    request_button, session_steps, session_words,
};

/// The tier's height across the canvas's top, and the most of the canvas it
/// takes when the canvas is short.
const TIER_HEIGHT: u32 = 112;
const TIER_SHARE: f32 = 0.45;
/// The width the tier's own controls take at its right end.
const TIER_CONTROLS_WIDTH: u32 = 148;
/// An open card's width, and one archive chip's row in it.
const CARD_WIDTH: u32 = 232;
const CHIP_ROW: u32 = 28;
/// An archive popover's width.
const POPOVER_WIDTH: f32 = 232.0;
/// The gap between an open card and the archive nodes it moves aside.
const CARD_CLEARANCE: f32 = 22.0;

/// The relation kind joining an archive node to the session it was saved
/// from: the relation cell's `data-kind`.
pub const SAVED_FROM: &str = "saved-from";

/// The key prefixes of tier nodes, so a tier node's `data-key` never reads as
/// a graph node's.
pub const SESSION_NODE_PREFIX: &str = "mere-session:";
pub const ARCHIVE_NODE_PREFIX: &str = "mere-archive:";

/// The `data-key` of a session's tier node.
pub fn session_node_key(id: SessionId) -> String {
    format!("{SESSION_NODE_PREFIX}{}", id.as_uuid())
}

/// The `data-key` of an archive's tier node.
pub fn archive_node_key(key: &str) -> String {
    format!("{ARCHIVE_NODE_PREFIX}{key}")
}

/// The session a tier node key names, if it names one.
pub(crate) fn session_of_node(key: &str) -> Option<SessionId> {
    key.strip_prefix(SESSION_NODE_PREFIX)
        .and_then(|id| Uuid::parse_str(id).ok())
        .map(SessionId::from_uuid)
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

/// The tier's height in a graph area `height` tall; zero with no tier.
pub(crate) fn tier_height(model: &MereViewModel, height: u32) -> u32 {
    if tier_shown(model) {
        TIER_HEIGHT.min((height as f32 * TIER_SHARE) as u32)
    } else {
        0
    }
}

/// An open card's height: its head (name and Close), its state, its steps and
/// one row per archive saved from it.
pub(crate) fn card_height(model: &MereViewModel, session: &SessionEntry) -> u32 {
    let steps = session.steps.len() + usize::from(can_save(model, session));
    CONTAINER_PAD * 2
        + CONTROL_ROW
        + LINE
        + control_rows(steps) * CONTROL_ROW
        + archives_on(model, Some(session)).len() as u32 * CHIP_ROW
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
    /// The tier's height across the top of the graph's canvas, in pixels;
    /// zero unless the sessions show as a tier.
    pub fn tier_height(&self) -> u32 {
        tier_height(self.model, self.graph_size().1)
    }

    /// Add the tier's nodes, relations, open card and attributes to `swatch`.
    pub(crate) fn with_tier(
        &self,
        mut swatch: GraphCanvasSwatch<String, NodeState>,
    ) -> GraphCanvasSwatch<String, NodeState> {
        let model = self.model;
        let size = (swatch.width as f32, swatch.height as f32);
        let inset = swatch.node_radius + swatch.edge_width;
        let tier = self.tier_height() as f32;
        let controls = if model.can_mint {
            TIER_CONTROLS_WIDTH as f32
        } else {
            0.0
        };
        let loose = archives_on(model, None);
        let slots = model.sessions.len() + usize::from(!loose.is_empty());
        let slot = (size.0 - controls).max(1.0) / slots.max(1) as f32;
        let session_y = tier * 0.28;
        let archive_y = tier * 0.74;
        // The graph's own nodes were laid out in the area below the tier.
        for node in &mut swatch.graph.nodes {
            node.position = below_tier(node.position, size, tier, inset);
        }
        let popover = self
            .state
            .archive_popover
            .as_ref()
            .map(|(key, popover)| (key.as_str(), popover.mode));

        let archive_node = |swatch: &mut GraphCanvasSwatch<String, NodeState>,
                            entry: &ArchiveEntry,
                            held: bool,
                            at: (f32, f32)| {
            let key = archive_node_key(&entry.key);
            swatch.graph.nodes.push(GraphCanvasNode {
                id: key.clone(),
                kind: NodeState::Available,
                position: unit(at, size, inset),
                label: archive_label(entry, held),
                key: Some(key.clone()),
            });
            let pinned = popover == Some((entry.key.as_str(), DetailPopoverMode::Pinned));
            for (name, value) in [
                ("data-tier", "archive".to_string()),
                ("data-archive-node", entry.key.clone()),
                ("aria-haspopup", "dialog".to_string()),
                ("aria-expanded", pinned.to_string()),
            ] {
                swatch.node_attrs.push((key.clone(), name.into(), value));
            }
        };
        let spread = |slot_index: usize, count: usize, index: usize| {
            slot * slot_index as f32 + slot * (index + 1) as f32 / (count + 1) as f32
        };

        for (index, session) in model.sessions.iter().enumerate() {
            let key = session_node_key(session.id);
            swatch.graph.nodes.push(GraphCanvasNode {
                id: key.clone(),
                kind: if session.trashed {
                    NodeState::Unavailable
                } else if session.attached {
                    NodeState::Open
                } else {
                    NodeState::Available
                },
                position: unit((spread(index, 1, 0), session_y), size, inset),
                label: session_label(session),
                key: Some(key.clone()),
            });
            for (name, value) in [
                ("data-tier", "session".to_string()),
                ("data-session", session.id.as_uuid().to_string()),
                ("data-attached", session.attached.to_string()),
                ("data-trashed", session.trashed.to_string()),
                (
                    "aria-expanded",
                    (self.state.expanded == Some(session.id)).to_string(),
                ),
            ] {
                swatch.node_attrs.push((key.clone(), name.into(), value));
            }
            if session.attached {
                // The shown session is the tier's mark: emphasis, a bold
                // label and `aria-current` on its target.
                swatch.selected = Some(key.clone());
            }
            let held = archives_on(model, Some(session));
            for (at, entry) in held.iter().enumerate() {
                archive_node(
                    &mut swatch,
                    entry,
                    true,
                    (spread(index, held.len(), at), archive_y),
                );
                swatch.relations.push(GraphCanvasRelation {
                    id: format!("{SAVED_FROM}:{}", entry.key),
                    from: archive_node_key(&entry.key),
                    to: key.clone(),
                    kind: SAVED_FROM.to_string(),
                    label: format!("{} to Session {}", entry.name, session.name),
                    route: Vec::new(),
                    visible: true,
                    emphasized: false,
                });
            }
        }
        for (at, entry) in loose.iter().enumerate() {
            archive_node(
                &mut swatch,
                entry,
                false,
                (spread(model.sessions.len(), loose.len(), at), archive_y),
            );
        }

        // The open card occupies its session's node; the archives saved from
        // that session move clear of it, below the card or else above it, so
        // their routes stay visible and end at its border.
        if let Some(session) = self.expanded_session() {
            let key = session_node_key(session.id);
            swatch = swatch.with_node_footprint(
                key.clone(),
                CARD_WIDTH as f32,
                card_height(model, session) as f32,
            );
            if let Some(card) = swatch.projected_node_footprint(&key) {
                let below = card.top + card.height + CARD_CLEARANCE;
                let y = if below <= size.1 - inset {
                    below
                } else {
                    (card.top - CARD_CLEARANCE).max(inset)
                };
                let held: Vec<String> = archives_on(model, Some(session))
                    .into_iter()
                    .map(|entry| archive_node_key(&entry.key))
                    .collect();
                for node in &mut swatch.graph.nodes {
                    if let Some(at) = held.iter().position(|held| *held == node.id) {
                        let x = card.left + card.width * (at + 1) as f32 / (held.len() + 1) as f32;
                        node.position = unit((x, y), size, inset);
                    }
                }
            }
        }
        swatch.focus_request = self.state.focus_return.clone();
        swatch
    }

    /// The session whose card is open, if the model still lists it.
    fn expanded_session(&self) -> Option<&SessionEntry> {
        let id = self.state.expanded?;
        self.model.sessions.iter().find(|session| session.id == id)
    }
}

/// Render the tier's widgets over the canvas: the rule under the tier, its
/// controls, the open card and the archive popover. They share the canvas's
/// positioned box, so the swatch's projection places them.
pub(crate) fn tier_widgets<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, NodeState>,
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
    if model.can_mint {
        let left = width.saturating_sub(TIER_CONTROLS_WIDTH) + CONTAINER_PAD;
        widgets.push(Box::new(
            el::<_, State, Action>(
                "div",
                request_button(
                    "New session",
                    MereViewRequest::Mint,
                    on_request.clone(),
                    vec![("data-request", "mint".into())],
                ),
            )
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

/// The open session's card, at its footprint on the canvas, keyed by its
/// session so moving the card to another session builds it afresh (and its
/// Close takes focus again).
pub(crate) fn tier_card<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, NodeState>,
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
    swatch: &GraphCanvasSwatch<String, NodeState>,
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
    let session = view.expanded_session()?;
    let region = swatch.projected_node_footprint(&session_node_key(session.id))?;
    let close = {
        let on_change = on_change.clone();
        request_focus(
            button("Close", move |state: &mut State, _| {
                on_change(state, MereViewEvent::Expand(None))
            })
            .attr("class", "mere-view-card-close")
            .attr("data-request", "close-session")
            .attr("aria-label", format!("Close session {}", session.name)),
            true,
        )
    };
    let head = el::<_, State, Action>(
        "div",
        (
            el::<_, State, Action>("span", session.name.clone())
                .attr("class", "mere-view-session-name"),
            close,
        ),
    )
    .attr("class", "mere-view-card-head");
    let popover = view.state.archive_popover.as_ref();
    let chips: Vec<MereViewElement<State, Action>> = archives_on(model, Some(session))
        .into_iter()
        .map(|entry| {
            let key = entry.key.clone();
            let on_change = on_change.clone();
            let pinned = popover.is_some_and(|(open, popover)| {
                *open == entry.key && popover.mode == DetailPopoverMode::Pinned
            });
            Box::new(
                el::<_, State, Action>(
                    "li",
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
                )
                .attr("class", "mere-view-archive"),
            ) as MereViewElement<State, Action>
        })
        .collect();
    let body = (
        head,
        el::<_, State, Action>("span", session_words(session))
            .attr("class", "mere-view-session-state"),
        el::<_, State, Action>(
            "div",
            session_steps(session, can_save(model, session), on_request),
        )
        .attr("class", "mere-view-session-steps"),
        el::<_, State, Action>("ul", chips)
            .attr("class", "mere-view-container-archives")
            .attr("aria-label", "Archives"),
    );
    let card = el::<_, State, Action>("section", body)
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
        .attr(
            "style",
            format!(
                "left:{}px;top:{}px;width:{}px;height:{}px",
                region.left, region.top, region.width, region.height
            ),
        );
    Some((
        session.id.as_uuid().to_string(),
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

/// The open archive popover: its peek, or its pinned controls.
fn archive_popover<State, Action, Change, Request>(
    view: &MereView<'_>,
    swatch: &GraphCanvasSwatch<String, NodeState>,
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
    let node = archive_node_key(key);
    let (_, center) = swatch
        .projected_positions()
        .into_iter()
        .find(|(id, _)| **id == node)?;
    let hit = swatch.hit_size;
    let trigger = (center.0 - hit / 2.0, center.1 - hit / 2.0, hit, hit);
    let saved_from = entry
        .session
        .and_then(|id| model.sessions.iter().find(|session| session.id == id))
        .map_or_else(
            || "In the archive".to_string(),
            |session| format!("Saved from {}", session.name),
        );
    let can_compose = archive.can_compose;
    let rows = usize::from(entry.can_open) + 2 * usize::from(can_compose);
    let panel_height = match popover.mode {
        DetailPopoverMode::Pinned => LINE * 3 + rows as u32 * CONTROL_ROW,
        _ => LINE * 3,
    } + CONTAINER_PAD * 2;
    let surface = OverlaySurface::new(
        trigger,
        (POPOVER_WIDTH, panel_height as f32),
        (0.0, 0.0, swatch.width as f32, swatch.height as f32),
    )
    .with_placement(Placement::Below)
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
        controls.push(Box::new(request_focus(
            button("Open as session", move |state: &mut State, _| {
                on_request(state, request.clone())
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
