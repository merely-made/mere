// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl PhysicsBoard {
    /// Reconcile the bodies to `items`: departed items drop, new ones spawn
    /// at their slot, existing ones keep their simulated position; every
    /// slot is re-anchored. Returns how many items are new. New items earn a
    /// settle burst.
    pub fn sync(&mut self, items: Vec<BoardItem>) -> usize {
        let mut fresh = 0;
        for item in &items {
            if !self.keys.contains_key(&item.id) {
                let key = NodeKey::new(self.next_key);
                self.next_key += 1;
                self.keys.insert(item.id.clone(), key);
                fresh += 1;
            }
        }
        let live: HashMap<&str, NodeKey> = items
            .iter()
            .map(|item| (item.id.as_str(), self.keys[&item.id]))
            .collect();
        self.keys.retain(|id, _| live.contains_key(id.as_str()));
        if self
            .dragging
            .is_some_and(|key| !live.values().any(|live_key| *live_key == key))
        {
            self.dragging = None;
            self.key_move = None;
            self.physics.set_dragging(false);
        }
        // Every body, at its slot; `sync_nodes` leaves an existing body where
        // the simulation put it and spawns the new ones where told.
        self.physics.sync_nodes(
            items
                .iter()
                .map(|item| {
                    (
                        live[item.id.as_str()],
                        Point2D::new(item.slot.0, item.slot.1),
                    )
                })
                .collect(),
        );
        self.physics.sync_edges(self.edges());
        // An encoded axis is pinned to its value (F28), so a card whose value
        // moved goes there on that axis; its free axes keep what physics gave
        // them, as every axis of an unencoded card does.
        if self.encoded.any() {
            let old: HashMap<&str, (f32, f32)> = self
                .items
                .iter()
                .map(|item| (item.id.as_str(), item.slot))
                .collect();
            let moved: Vec<_> = items
                .iter()
                .filter_map(|item| {
                    let was = *old.get(item.id.as_str())?;
                    let key = live[item.id.as_str()];
                    let at = self.view.position_of(key)?;
                    (was != item.slot && self.dragging != Some(key)).then(|| {
                        let x = if self.encoded.x { item.slot.0 } else { at.x };
                        let y = if self.encoded.y { item.slot.1 } else { at.y };
                        (key, Point2D::new(x, y))
                    })
                })
                .collect();
            if !moved.is_empty() {
                self.physics.seed(moved);
            }
        }
        self.items = items;
        self.sync_roles();
        self.rebuild_forces();
        self.settle_for_choice();
        // A card whose site moved into a pinned group stops being dragged.
        self.end_withdrawn_drag();
        // So a host that syncs and draws in one frame sees the fresh item.
        self.physics.refresh(&mut self.view);
        fresh
    }

    /// The pairs of items the law reads as edges, so linked cards cluster
    /// (stack-lift D3). A pair naming an item the board does not hold is
    /// kept and reads once it arrives. Changed links earn a settle burst.
    pub fn set_links(&mut self, links: Vec<(String, String)>) {
        if links == self.links {
            return;
        }
        self.links = links;
        self.physics.sync_edges(self.edges());
        self.rebuild_forces();
        self.settle_for_choice();
    }

    /// The links between held items, as the backend's keys.
    pub(super) fn edges(&self) -> Vec<(NodeKey, NodeKey)> {
        self.links
            .iter()
            .filter_map(|(a, b)| Some((*self.keys.get(a)?, *self.keys.get(b)?)))
            .collect()
    }

    /// Advance one frame, folding the freshest layout into the board's view.
    /// Returns whether the board is still moving. The backend owns the settle
    /// budget, so a law that never rests simply holds a budget that never runs
    /// out (see [`settle_for_choice`](Self::settle_for_choice)).
    pub fn tick(&mut self) -> bool {
        let settling = self.physics.advance_frame(&mut self.view);
        let budget_ended = self.rest.budget_ended(self.physics.is_settling());
        if self.dragging.is_some() {
            self.rest.arm();
        } else if !self.halted && (self.rest.rested(self.physics.rms_speed()) || budget_ended) {
            // At rest, or the budget spent with the cards still moving (F63).
            self.start_home();
        }
        self.rest.step(&mut self.physics);
        settling || self.rest.gliding()
    }

    /// At rest, every anchored card away from its slot glides home (F45).
    pub(super) fn start_home(&mut self) {
        let glide: Vec<_> = self
            .items
            .iter()
            .filter(|item| self.roles.role(self.keys[&item.id], Some(&item.site)) == Role::Anchored)
            .filter_map(|item| {
                let key = self.keys[&item.id];
                let at = self.view.position_of(key)?;
                Some((key, at, Point2D::new(item.slot.0, item.slot.1)))
            })
            .collect();
        self.rest.start(&mut self.physics, glide);
    }

    /// A disturbance releases the cards held at home to their springs.
    pub(super) fn unpark(&mut self) {
        let pinned: HashSet<NodeKey> = self
            .items
            .iter()
            .filter(|item| self.roles.role(self.keys[&item.id], Some(&item.site)) == Role::Pinned)
            .map(|item| self.keys[&item.id])
            .collect();
        self.rest
            .release(&mut self.physics, |key| pinned.contains(&key));
    }

    /// How many anchored cards are held at home. Receipt introspection.
    pub fn anchored_home_count(&self) -> usize {
        self.rest.parked_count()
    }

    /// Begin a transient drag of an item. The body is pinned at its current
    /// simulated position so the first pointer move cannot jump it, while the
    /// other bodies continue responding to the board's forces. A card that
    /// does not advertise drag refuses it: a pinned card, which stays in its
    /// arrangement, or one whose drag the binding withdrew (G9).
    pub fn drag_start(&mut self, id: &str) -> bool {
        let Some(&key) = self.keys.get(id) else {
            return false;
        };
        if self.dragging.is_some() || !self.permits(id, ArrangementAction::Drag) {
            return false;
        }
        let Some(position) = self.position(id) else {
            return false;
        };
        self.unpark();
        self.dragging = Some(key);
        self.physics.pin(key, Point2D::new(position.0, position.1));
        self.physics.set_dragging(true);
        true
    }

    /// Move the currently-held item in score coordinates. This only updates
    /// the pinned body; it does not rebuild forces or reset any other body.
    pub fn drag_move(&mut self, x: f32, y: f32) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        let Some(key) = self.dragging else {
            return false;
        };
        self.physics.pin(key, Point2D::new(x, y));
        true
    }

    /// Release the held item back to the dynamic solver, by its role: a
    /// seeded card stays where it was dropped, an anchored card eases back to
    /// its slot. An encoded axis goes back to its value at once, since the
    /// data, not the drop, says where the card is on it.
    pub fn drag_end(&mut self) -> bool {
        self.key_move = None;
        let Some(key) = self.dragging.take() else {
            return false;
        };
        self.physics.unpin(key);
        self.physics.set_dragging(false);
        // Dropped at rest: the pointer's last step is not a throw.
        if let Some(item) = self.items.iter().find(|item| self.keys[&item.id] == key)
            && let Some(at) = self.view.position_of(key)
        {
            let x = if self.encoded.x { item.slot.0 } else { at.x };
            let y = if self.encoded.y { item.slot.1 } else { at.y };
            self.physics.seed(vec![(key, Point2D::new(x, y))]);
        }
        self.sync_roles();
        self.settle_for_choice();
        true
    }

    /// Start a keyboard move of card `id`, the pointerless drag (F67): a drag
    /// the host's arrows steer, refused as a drag is.
    pub fn begin_key_move(&mut self, id: &str) -> bool {
        let Some(at) = self.position(id) else {
            return false;
        };
        if !self.drag_start(id) {
            return false;
        }
        self.key_move = Some((at, at));
        true
    }

    /// The card a keyboard move holds.
    pub fn key_moving(&self) -> Option<&str> {
        self.key_move?;
        let key = self.dragging?;
        self.keys
            .iter()
            .find(|(_, k)| **k == key)
            .map(|(id, _)| id.as_str())
    }

    /// Nudge the held card by `(dx, dy)` in the score's units.
    pub fn key_move_by(&mut self, dx: f32, dy: f32) -> bool {
        let Some((origin, at)) = self.key_move else {
            return false;
        };
        let to = (at.0 + dx, at.1 + dy);
        if !self.drag_move(to.0, to.1) {
            return false;
        }
        self.key_move = Some((origin, to));
        true
    }

    /// End the keyboard move: `drop` (Enter) releases the card where it is,
    /// by its role; otherwise (Escape) it goes back where the move began,
    /// then is released.
    pub fn end_key_move(&mut self, drop: bool) -> bool {
        let Some((origin, _)) = self.key_move else {
            return false;
        };
        let held = self.dragging;
        if !self.drag_end() {
            return false;
        }
        if !drop && let Some(key) = held {
            self.physics
                .seed(vec![(key, Point2D::new(origin.0, origin.1))]);
            self.physics.refresh(&mut self.view);
        }
        true
    }

    /// Halt motion for a paused board. An active drag is cancelled and its
    /// body is returned to the dynamic solver before the halt, so a later
    /// `sync` or choice change can explicitly reawaken the board.
    pub fn halt(&mut self) {
        self.key_move = None;
        if let Some(key) = self.dragging.take() {
            self.physics.unpin(key);
            self.physics.set_dragging(false);
        }
        self.unpark();
        self.halted = true;
        self.physics.halt();
    }

    /// Whether the board still needs frames for settling, a drag, or a
    /// deliberately continuous law.
    pub fn is_settling(&self) -> bool {
        self.physics.is_settling()
    }

    /// Where the item is now, in the score's units.
    pub fn position(&self, id: &str) -> Option<(f32, f32)> {
        let key = self.keys.get(id)?;
        self.view.position_of(*key).map(|p| (p.x, p.y))
    }

    /// The bodies' kinetic energy: live inline, the last snapshot's figure
    /// offloaded.
    pub fn energy(&self) -> f32 {
        self.physics.kinetic_energy()
    }

    /// The distance between two items, in the score's units.
    pub fn gap(&self, a: &str, b: &str) -> Option<f32> {
        let (ax, ay) = self.position(a)?;
        let (bx, by) = self.position(b)?;
        Some(((ax - bx).powi(2) + (ay - by).powi(2)).sqrt())
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}
