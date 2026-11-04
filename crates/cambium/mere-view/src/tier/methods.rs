/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use super::*;

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
