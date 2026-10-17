// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar's attention and seed on the canvas (Merely Design
//! System DECISIONS 2026-10-16; Scenograph editor plan SE91).
//!
//! `selected` stays the canvas's selection; the [`AttentionBook`] follows it,
//! so a deselected node is retained until dismissed, a foreground pin holds a
//! node forward while the selection moves, and the host marks suggestions.
//! None of it moves a body or touches the explicit position pins.
//!
//! Reset eases every unheld node back to its seed: the saved seed if the
//! person set one, else the active arrangement's position, else a point
//! derived from the node's id. The canvas places the seeds at once; while
//! paused it queues the return a host eases over
//! [`scenograph::seed::RESET_EASE_MS`].

use scenograph::attention::{Attention, AttentionBook};
use scenograph::seed::{RESET_EASE_MS, SeedBook, SeedMove, identity_seed, reset_moves};

use super::*;

/// Recipe seeds without an arrangement fill a disc this many body radii
/// across per square root of the node count.
const SEED_DISC_RADII: f32 = 4.0;

/// The canvas's attention and seed, kept in one field.
#[derive(Clone, Debug, Default)]
pub(crate) struct GrammarState {
    attention: AttentionBook<NodeKey>,
    seeds: SeedBook<NodeKey>,
    /// The host's suggestion settings (`suggestions.rs`); off by default.
    pub(crate) suggestions: scenograph::suggestion::SuggestionSettings,
    /// The last offer, kept so a settings change re-admits from it.
    pub(crate) offered: Vec<(NodeKey, String)>,
    /// Unvisited links offered by the pages they appear on (`link_suggestions.rs`).
    pub(crate) links: super::link_suggestions::LinkSuggestions,
}

impl GrammarState {
    pub(crate) fn attention_book(&self) -> &AttentionBook<NodeKey> {
        &self.attention
    }

    pub(crate) fn attention_book_mut(&mut self) -> &mut AttentionBook<NodeKey> {
        &mut self.attention
    }

    pub(crate) fn seed_book(&self) -> &SeedBook<NodeKey> {
        &self.seeds
    }

    pub(crate) fn seed_book_mut(&mut self) -> &mut SeedBook<NodeKey> {
        &mut self.seeds
    }

    /// Keep only the nodes `live` still holds.
    pub(crate) fn retain(&mut self, mut live: impl FnMut(&NodeKey) -> bool) {
        self.attention.retain(&mut live);
        self.offered.retain(|(key, _)| live(key));
        self.seeds.retain(live);
    }
}

fn vec2(point: PortablePoint) -> sceno::Vec2 {
    sceno::Vec2::new(point.x, point.y)
}

fn point(v: sceno::Vec2) -> PortablePoint {
    PortablePoint::new(v.x, v.y)
}

impl Canvas {
    /// Fold the live selection into the attention book: whatever left it is
    /// retained. Called each frame and before every attention change.
    pub(crate) fn sync_attention(&mut self) {
        self.grammar.attention.select(self.selected.iter().copied());
    }

    /// `member`'s attention, reading the live selection, or `None` when it is
    /// not in the graph.
    pub fn member_attention(&self, member: uuid::Uuid) -> Option<Attention> {
        let key = self.graph.get_node_key_by_id(member)?;
        Some(
            self.grammar
                .attention
                .attention_with(&key, self.selected.contains(&key)),
        )
    }

    /// Hold `member` forward while the selection moves, or release it to
    /// retained. Not a position pin: the body stays free. Returns `false` if
    /// the member is not in the graph.
    pub fn set_member_foreground_pin(&mut self, member: uuid::Uuid, pinned: bool) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        self.sync_attention();
        self.grammar.attention.set_foreground_pin(key, pinned);
        true
    }

    /// Whether `member` has a foreground pin, or `None` when it is not in the
    /// graph.
    pub fn member_foreground_pinned(&self, member: uuid::Uuid) -> Option<bool> {
        let key = self.graph.get_node_key_by_id(member)?;
        Some(self.grammar.attention.foreground_pins().contains(&key))
    }

    /// Pin every selected node forward, or release them all when every one
    /// is pinned already. Returns `false` with nothing selected.
    pub fn toggle_selection_foreground_pin(&mut self) -> bool {
        if self.selected.is_empty() {
            return false;
        }
        self.sync_attention();
        let pins = self.grammar.attention.foreground_pins();
        let pin = !self.selected.iter().all(|key| pins.contains(key));
        for key in self.selected.clone() {
            self.grammar.attention.set_foreground_pin(key, pin);
        }
        true
    }

    /// Return a retained member to ambient. Refused for a forward member or
    /// one not in the graph.
    pub fn dismiss_retained_member(&mut self, member: uuid::Uuid) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        self.sync_attention();
        self.grammar.attention.dismiss(&key)
    }

    /// Dismiss every retained member, returning how many there were.
    pub fn dismiss_all_retained(&mut self) -> usize {
        self.sync_attention();
        self.grammar.attention.dismiss_all()
    }

    /// Offer `members` as suggestions from no particular source, replacing the
    /// last offer; the suggestion settings decide which show
    /// ([`Self::offer_suggestions`]).
    pub fn set_suggested_members(&mut self, members: impl IntoIterator<Item = uuid::Uuid>) {
        self.offer_suggestions(members.into_iter().map(|member| (member, String::new())));
    }

    /// Members with `attention`, in key order.
    pub fn members_with_attention(&self, attention: Attention) -> Vec<uuid::Uuid> {
        self.graph
            .nodes()
            .filter(|(key, _)| {
                self.grammar
                    .attention
                    .attention_with(key, self.selected.contains(key))
                    == attention
            })
            .map(|(_, node)| node.id)
            .collect()
    }

    /// Set seed: the current arrangement becomes the seed for this view.
    pub fn set_seed(&mut self) {
        self.grammar
            .seeds
            .set(self.view.positions().map(|(key, at)| (key, vec2(at))));
    }

    /// Return to the recipe seed. Returns whether a saved seed was dropped.
    pub fn restore_recipe_seed(&mut self) -> bool {
        self.grammar.seeds.restore_recipe()
    }

    pub fn has_saved_seed(&self) -> bool {
        self.grammar.seeds.saved().is_some()
    }

    /// `key`'s seed: saved, else the active arrangement's position, else a
    /// point derived from its id around the origin.
    fn seed_of(&self, key: NodeKey, radius: f32) -> Option<sceno::Vec2> {
        self.grammar.seeds.seed_of(&key, || {
            self.arrangement_slot(key).map(vec2).or_else(|| {
                let id = self.graph.get_node(key)?.id;
                Some(identity_seed(&id.to_string(), sceno::Vec2::ZERO, radius))
            })
        })
    }

    /// Reset: every node not held (position pin, pinned role or drag) goes
    /// back to its seed. While paused the canvas also queues the whole
    /// placement as a return, which a host with transitions eases over
    /// [`RESET_EASE_MS`] like a stop's ([`Canvas::take_stop_return`]).
    /// Returns the moves.
    pub fn reset_to_seed(&mut self) -> Vec<SeedMove<uuid::Uuid>> {
        let count = self.graph.nodes().count() as f32;
        let radius = SEED_DISC_RADII * seiche::NODE_BODY_RADIUS * count.sqrt();
        let dragged = self.drag.map(|d| d.node);
        let current: Vec<_> = self
            .view
            .positions()
            .map(|(key, at)| (key, vec2(at)))
            .collect();
        let before: Vec<_> = current.iter().map(|&(key, at)| (key, point(at))).collect();
        let moves = reset_moves(
            current,
            |key| {
                self.pinned_nodes.contains(key)
                    || self.arrangement_pin_installed(*key)
                    || dragged == Some(*key)
            },
            |key| self.seed_of(*key, radius),
        );
        if moves.is_empty() {
            return Vec::new();
        }
        let landed: Vec<_> = moves.iter().map(|m| (m.key, point(m.to))).collect();
        for &(key, at) in &landed {
            self.view.set_position(key, at);
        }
        if let Some(positions) = self.paused_positions.as_mut() {
            let to: HashMap<_, _> = landed.iter().copied().collect();
            for (key, at) in positions.iter_mut() {
                if let Some(seed) = to.get(key) {
                    *at = *seed;
                }
            }
        }
        if self.physics_paused {
            let to: HashMap<_, _> = landed.iter().copied().collect();
            let after = before
                .iter()
                .map(|&(key, at)| (key, to.get(&key).copied().unwrap_or(at)))
                .collect();
            self.queue_return(StopReturn {
                from: before,
                to: after,
                duration_ms: Some(RESET_EASE_MS),
            });
        }
        self.reset_frame_time();
        self.physics.seed(landed);
        moves
            .into_iter()
            .filter_map(|m| {
                let id = self.graph.get_node(m.key)?.id;
                Some(SeedMove {
                    key: id,
                    from: m.from,
                    to: m.to,
                })
            })
            .collect()
    }
}
