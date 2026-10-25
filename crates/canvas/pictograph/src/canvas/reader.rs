// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! What a screen reader is told of the canvas, and how it moves an item
//! without a pointer (dynamics grammar plan, G9: F62 and F65 to F67).
//!
//! Target-neutral: a host maps [`CanvasDescription`] onto its own
//! accessibility vocabulary (Graphshell onto Cambium's producer semantics).
//! The canvas describes the items on screen, at most [`DESCRIBED_ITEMS`] of
//! them, always including the focused one, each with the actions it
//! advertises ([`Canvas::advertised_actions`]). Invoking Drag without a
//! pointer starts a keyboard move: the host's arrows nudge the item, Enter
//! drops it by its role, as a pointer's release does, and Escape puts it back
//! where the move began.

use kernel::geometry::PortablePoint;
use kernel::graph::NodeKey;

use super::Canvas;
use super::actions::{AdvertisedAction, ArrangementAction};

/// The most items the canvas describes at once (F67).
pub const DESCRIBED_ITEMS: usize = 200;

/// Nonreused action keys for kernel-authored membership lifetimes. Reconciliation
/// retains ordinary edits and graph clones, but retires a removed/recreated
/// member even when its UUID and graph slot recur inside one mutation callback.
/// Semantic reads and activation only compare witnesses; they never mint keys.
#[derive(Default)]
pub(super) struct DescribedKeyRegistry {
    pub(super) by_member: std::collections::HashMap<uuid::Uuid, u64>,
    by_key: std::collections::HashMap<u64, (uuid::Uuid, kernel::graph::SurfaceNodeLifetime)>,
}

impl DescribedKeyRegistry {
    fn key_for(&mut self, member: uuid::Uuid, life: kernel::graph::SurfaceNodeLifetime) -> u64 {
        if let Some(key) = self.by_member.get(&member) {
            if self.by_key.get(key) == Some(&(member, life)) {
                return *key;
            }
            self.by_key.remove(key);
        }
        // Zero is reserved for the initial/unset value. Exhaustion is not
        // recoverable without risking stale-action aliasing, so fail closed.
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_KEY: AtomicU64 = AtomicU64::new(1);
        let key = NEXT_KEY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |key| {
                key.checked_add(1)
            })
            .expect("described action key space exhausted");
        self.by_member.insert(member, key);
        self.by_key.insert(key, (member, life));
        key
    }

    fn retain_live(&mut self, live: &std::collections::HashSet<uuid::Uuid>) {
        self.by_member.retain(|member, _| live.contains(member));
        self.by_key.retain(|_, (member, _)| live.contains(member));
    }

    pub(super) fn reconcile(&mut self, graph: &kernel::graph::Graph) {
        let live = graph.nodes().map(|(_, node)| node.id).collect();
        self.retain_live(&live);
        for (_, node) in graph.nodes() {
            if let Some(life) = graph.surface_node_lifetime(node.id) {
                self.key_for(node.id, life);
            }
        }
    }
}

/// One item as a reader is told it.
#[derive(Clone, Debug, PartialEq)]
pub struct DescribedItem {
    /// Opaque identity for this live member. Independent of graph slots and
    /// descriptions; removal or graph replacement ends its life.
    pub key: u64,
    pub member: uuid::Uuid,
    /// The label its caption shows.
    pub name: String,
    /// Where it is drawn, `[x, y, width, height]` in the canvas's logical px
    /// from its top-left corner.
    pub rect: [f32; 4],
    pub actions: Vec<AdvertisedAction>,
    pub focused: bool,
}

/// The items a reader is told of, and how many the canvas holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CanvasDescription {
    pub items: Vec<DescribedItem>,
    /// Every item the canvas shows, on screen or not.
    pub total: usize,
}

impl CanvasDescription {
    /// The slot's name: "N of M shown" (F67).
    pub fn name(&self) -> String {
        format!("{} of {} shown", self.items.len(), self.total)
    }
}

/// A keyboard move under way: the item, and where the move began.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct KeyMove {
    node: NodeKey,
    origin: PortablePoint,
}

impl Canvas {
    /// The items on screen in graph order, at most `cap`, the focused one
    /// always among them (it takes the last place when the cap would leave it
    /// out), each with its actions.
    pub fn describe_items(&self, cap: usize) -> CanvasDescription {
        let focused = self.focused_key();
        let (width, height) = (self.view_w as f32, self.view_h as f32);
        let mut total = 0;
        let mut shown = Vec::new();
        let mut focused_shown = false;
        for (key, _) in self.graph.nodes() {
            if !self.node_visible_in_canvas(key) {
                continue;
            }
            total += 1;
            let Some(rect) = self.item_rect(key) else {
                continue;
            };
            let [x, y, w, h] = rect;
            let on_screen = x + w > 0.0 && y + h > 0.0 && x < width && y < height;
            let is_focused = focused == Some(key);
            if (on_screen && shown.len() < cap) || is_focused {
                focused_shown |= is_focused;
                shown.push((key, rect));
            }
        }
        // The focused item came beyond the cap: it replaces the last other one.
        if shown.len() > cap && focused_shown {
            let last_other = shown.iter().rposition(|(key, _)| Some(*key) != focused);
            if let Some(index) = last_other {
                shown.remove(index);
            }
        }
        let items = shown
            .into_iter()
            .filter_map(|(key, rect)| {
                let node = self.graph.get_node(key)?;
                let action_key = *self.described_keys.by_member.get(&node.id)?;
                self.member_of_described(action_key)?;
                Some(DescribedItem {
                    key: action_key,
                    member: node.id,
                    name: self.graph.node_display_label(key),
                    rect,
                    actions: self.actions_of(key),
                    focused: focused == Some(key),
                })
            })
            .collect();
        CanvasDescription { items, total }
    }

    /// The member a described item's key names.
    pub fn member_of_described(&self, key: u64) -> Option<uuid::Uuid> {
        let (member, life) = *self.described_keys.by_key.get(&key)?;
        (self.graph.surface_node_lifetime(member) == Some(life)).then_some(member)
    }

    /// Where `key` is drawn, in the canvas's logical px.
    fn item_rect(&self, key: NodeKey) -> Option<[f32; 4]> {
        let (x, y) = self.screen_position_of(key)?;
        let side = self.node_size(key) * self.camera.zoom;
        Some([x - side / 2.0, y - side / 2.0, side, side])
    }

    /// Pin `member` where it is, as the explicit pin does. Refused unless it
    /// advertises pin.
    pub fn pin_member(&mut self, member: uuid::Uuid) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        self.pin_key(key)
    }

    /// Start a keyboard move of `member`: it is held where it is, as a
    /// pointer drag holds it, until [`end_key_move`](Self::end_key_move).
    /// Refused unless it advertises drag, or while a drag or another move is
    /// under way.
    pub fn begin_key_move(&mut self, member: uuid::Uuid) -> bool {
        let Some(node) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        if self.roles.key_move.is_some()
            || self.drag.is_some_and(|drag| drag.moved)
            || !self.permits(node, ArrangementAction::Drag)
        {
            return false;
        }
        let Some(origin) = self.view.position_of(node) else {
            return false;
        };
        self.unpark();
        self.physics.set_dragging(true);
        self.place_pinned_node(node, origin);
        self.roles.key_move = Some(KeyMove { node, origin });
        self.roles.last_key_move = self.roles.key_move;
        true
    }

    /// How far, in screen px, the latest keyboard move has carried its item
    /// from where that move began, during it or after. `None` before any.
    /// Receipt introspection.
    pub fn key_move_offset(&self) -> Option<(f32, f32)> {
        let KeyMove { node, origin } = self.roles.last_key_move?;
        let at = self.view.position_of(node)?;
        let zoom = self.camera.zoom;
        Some(((at.x - origin.x) * zoom, (at.y - origin.y) * zoom))
    }

    /// The member a keyboard move holds.
    pub fn key_moving(&self) -> Option<uuid::Uuid> {
        let node = self.roles.key_move?.node;
        self.graph.get_node(node).map(|node| node.id)
    }

    /// Nudge the held item by screen px, the host's arrow step.
    pub fn key_move_by(&mut self, dx: f32, dy: f32) -> bool {
        let Some(KeyMove { node, .. }) = self.roles.key_move else {
            return false;
        };
        let Some(at) = self.view.position_of(node) else {
            return false;
        };
        let zoom = self.camera.zoom.max(f32::EPSILON);
        self.place_pinned_node(node, PortablePoint::new(at.x + dx / zoom, at.y + dy / zoom));
        true
    }

    /// End the keyboard move: `drop` (Enter) releases the item where it is,
    /// by its role, as a pointer's release does; otherwise (Escape) it goes
    /// back where the move began, then is released.
    pub fn end_key_move(&mut self, drop: bool) -> bool {
        let Some(KeyMove { node, origin }) = self.roles.key_move.take() else {
            return false;
        };
        if !drop {
            self.place_pinned_node(node, origin);
        }
        self.physics.set_dragging(false);
        self.release_dragged(node);
        true
    }

    /// A keyboard move whose item no longer advertises drag ends at once, as
    /// a release (F60).
    pub(crate) fn end_withdrawn_key_move(&mut self) {
        if let Some(KeyMove { node, .. }) = self.roles.key_move
            && !self.permits(node, ArrangementAction::Drag)
        {
            self.end_key_move(true);
        }
    }
}
