// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Arrangement roles on the canvas (dynamics grammar plan, G7).
//!
//! The active arrangement's positions play the roles of a
//! [`seiche::RoleTable`]: seeded by default (F23), anchored items return by an
//! anchor spring while playing and jump back when a paused drag ends (F19),
//! pinned items are kinematic at their position. An explicit pin
//! (`pin_focused`, a nudge) is the per-item pinned case (F22). A pick while
//! playing keeps playing from the landed positions, and a stop returns the
//! anchored items while seeded and pinned ones stay (F24). "Settled" is the
//! latest settle's positions, picked like any other arrangement (F30).

use super::*;
use seiche::{Role, RoleTable};

/// The arrangement id of the latest settle (F30). Not a cartography adapter:
/// the canvas supplies its positions.
pub const SETTLED_ARRANGEMENT: &str = "settled";

/// Kinetic energy below which a playing graph counts as come to rest: P2's
/// floor, the figure `never_rests` was measured against (G1, F10).
pub const SETTLE_ENERGY_FLOOR: f32 = 1.0;

/// A stop's return of anchored items: where they were and where they go.
/// A host that adopts transitions animates between the two
/// ([`Canvas::preview_paused_positions`], [`Canvas::land_paused_positions`]);
/// the canvas has already placed `to`, so a host that does not still lands.
#[derive(Clone, Debug, PartialEq)]
pub struct StopReturn {
    pub from: Vec<(NodeKey, PortablePoint)>,
    pub to: Vec<(NodeKey, PortablePoint)>,
}

/// The canvas's role state, kept in one field so it moves as one piece.
#[derive(Clone, Debug)]
pub(crate) struct ArrangementRoles {
    pub(crate) table: RoleTable,
    /// The anchored role's return stiffness ([`seiche::AnchorSpring`]).
    pub(crate) stiffness: f32,
    /// Bodies the table holds kinematic, so a role change can free them.
    pinned: HashSet<NodeKey>,
    /// A pick made while playing resumes once its final placement lands.
    resume_after_pick: bool,
    settled: Option<Vec<(NodeKey, PortablePoint)>>,
    settle_armed: bool,
    settles: u64,
    stop_return: Option<StopReturn>,
}

impl Default for ArrangementRoles {
    fn default() -> Self {
        Self {
            table: RoleTable::default(),
            stiffness: seiche::DEFAULT_ANCHOR_STIFFNESS,
            pinned: HashSet::new(),
            resume_after_pick: false,
            settled: None,
            settle_armed: false,
            settles: 0,
            stop_return: None,
        }
    }
}

impl Canvas {
    /// The roles the active arrangement's positions play.
    pub fn arrangement_roles(&self) -> &RoleTable {
        &self.roles.table
    }

    /// Replace the role table wholesale (a scene open).
    pub fn set_arrangement_roles(&mut self, table: RoleTable) {
        self.roles.table = table;
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
    }

    /// The recipe's role, which every item takes unless a group or the item
    /// says otherwise.
    pub fn set_arrangement_role(&mut self, role: Role) {
        self.roles.table.default = role;
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
    }

    /// A group's role (`None` clears it). Groups are sites: the URL host,
    /// [`Self::role_group_of`].
    pub fn set_group_role(&mut self, group: &str, role: Option<Role>) {
        match role {
            Some(role) => self.roles.table.groups.insert(group.to_string(), role),
            None => self.roles.table.groups.remove(group),
        };
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
    }

    /// One item's role (`None` clears it), by member id. Returns whether the
    /// member exists.
    pub fn set_member_role(&mut self, member: uuid::Uuid, role: Option<Role>) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        match role {
            Some(role) => self.roles.table.items.insert(key, role),
            None => self.roles.table.items.remove(&key),
        };
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
        true
    }

    /// The group a node's role override is read from: its site.
    pub fn role_group_of(&self, key: NodeKey) -> Option<String> {
        self.graph
            .get_node(key)
            .map(|node| Graph::url_grouping_key(node.url()).to_string())
    }

    /// The role `key` plays now: an explicit pin first, then the table.
    pub fn arrangement_role_of(&self, key: NodeKey) -> Role {
        if self.pinned_nodes.contains(&key) {
            return Role::Pinned;
        }
        self.roles
            .table
            .role(key, self.role_group_of(key).as_deref())
    }

    /// How firmly an anchored item returns ([`seiche::AnchorSpring`]'s
    /// stiffness).
    pub fn anchor_stiffness(&self) -> f32 {
        self.roles.stiffness
    }

    /// Set the anchored return's stiffness, re-installing the springs.
    pub fn set_anchor_stiffness(&mut self, stiffness: f32) {
        self.roles.stiffness = stiffness.max(0.0);
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
    }

    /// The active arrangement's position for `key`, if it has one.
    pub(crate) fn arrangement_slot(&self, key: NodeKey) -> Option<PortablePoint> {
        self.strategy_positions
            .as_ref()?
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, p)| *p)
    }

    /// Realize the roles over the active arrangement: pinned items kinematic
    /// at their positions, anchored items on springs while playing, seeded
    /// items left alone. With no arrangement nothing is held but explicit pins.
    pub(crate) fn sync_arrangement_roles(&mut self) {
        let playing = !self.physics_paused;
        let mut pinned = HashSet::new();
        let mut anchors = Vec::new();
        let dragged = self.drag.map(|d| d.node);
        if let Some(slots) = self.strategy_positions.clone() {
            for (key, at) in slots {
                // Explicit pins and the body under the pointer are input's.
                if self.pinned_nodes.contains(&key) {
                    continue;
                }
                let role = self.arrangement_role_of(key);
                if dragged == Some(key) {
                    if role == Role::Pinned {
                        pinned.insert(key);
                    }
                    continue;
                }
                match role {
                    Role::Seeded => {},
                    Role::Anchored => anchors.push((key, (at.x, at.y))),
                    Role::Pinned => {
                        self.physics.pin(key, at);
                        pinned.insert(key);
                    },
                }
            }
        }
        for key in self.roles.pinned.difference(&pinned) {
            if !self.pinned_nodes.contains(key) {
                self.physics.unpin(*key);
            }
        }
        self.roles.pinned = pinned;
        let force = (playing && !anchors.is_empty())
            .then(|| seiche::AnchorSpring::new(anchors).with_stiffness(self.roles.stiffness));
        self.physics.set_anchor_force(force);
    }

    /// A drag ended on an anchored item with physics paused: it jumps back to
    /// its position (F19).
    pub(crate) fn jump_home(&mut self, key: NodeKey) {
        let Some(home) = self.arrangement_slot(key) else {
            return;
        };
        self.view.set_position(key, home);
        if let Some(positions) = self.paused_positions.as_mut()
            && let Some((_, at)) = positions.iter_mut().find(|(k, _)| *k == key)
        {
            *at = home;
        }
        self.reset_frame_time();
        self.physics.seed(vec![(key, home)]);
    }

    /// A pick while playing pauses for its transition and resumes when the
    /// final placement lands (F24). Called right after the pick's pause.
    pub(crate) fn note_pick(&mut self, was_playing: bool) {
        self.roles.resume_after_pick = was_playing;
    }

    /// Any explicit pause or play cancels a pick's pending resume.
    pub(crate) fn cancel_pick_resume(&mut self) {
        self.roles.resume_after_pick = false;
    }

    /// Called when a final placement lands: resume a pick made while playing.
    pub(crate) fn land_pick(&mut self) {
        if std::mem::take(&mut self.roles.resume_after_pick) {
            self.set_physics_paused(false);
        }
    }

    /// The placement a stop leaves (F24): every item where motion left it,
    /// except anchored items, which go back to their positions.
    pub(crate) fn stop_placement(
        &mut self,
        from: &[(NodeKey, PortablePoint)],
    ) -> Vec<(NodeKey, PortablePoint)> {
        let mut moved = false;
        let to: Vec<_> = from
            .iter()
            .map(|&(key, at)| {
                if self.arrangement_role_of(key) == Role::Anchored
                    && let Some(home) = self.arrangement_slot(key)
                {
                    moved |= home != at;
                    (key, home)
                } else {
                    (key, at)
                }
            })
            .collect();
        self.roles.stop_return = moved.then(|| StopReturn {
            from: from.to_vec(),
            to: to.clone(),
        });
        to
    }

    /// Whether a stop's return waits for a host to animate it.
    pub fn has_stop_return(&self) -> bool {
        self.roles.stop_return.is_some()
    }

    /// The last stop's return, for a host that animates it. Taking it clears
    /// it; the canvas has already placed its `to`.
    pub fn take_stop_return(&mut self) -> Option<StopReturn> {
        self.roles.stop_return.take()
    }

    /// Show an in-between paused placement without seeding physics: a
    /// host-clocked stop transition's sampled frames. A no-op while playing.
    pub fn preview_paused_positions(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if self.physics_paused {
            self.paused_positions = Some(positions.to_vec());
        }
    }

    /// Land a paused placement: shown and seeded, so play starts from it.
    pub fn land_paused_positions(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if !self.physics_paused {
            return;
        }
        self.paused_positions = Some(positions.to_vec());
        self.apply_strategy_to_view();
        self.reset_frame_time();
        self.physics.seed(positions.to_vec());
    }

    /// The latest settle's positions (F30), or `None` before the first.
    pub fn settled_positions(&self) -> Option<&[(NodeKey, PortablePoint)]> {
        self.roles.settled.as_deref()
    }

    /// How many settles have been recorded. Receipt introspection.
    pub fn settle_count(&self) -> u64 {
        self.roles.settles
    }

    /// Fold one frame into the settle record: a playing graph that comes to
    /// rest (its settle ends, or its energy falls below the floor) replaces
    /// Settled, once per rest. A law that never rests never replaces it.
    pub(crate) fn note_settle(&mut self, settling: bool) {
        if self.physics_paused || self.drag.is_some_and(|d| d.moved) {
            return;
        }
        let at_rest = !settling || self.physics.kinetic_energy() < SETTLE_ENERGY_FLOOR;
        if !at_rest {
            self.roles.settle_armed = true;
            return;
        }
        if !std::mem::take(&mut self.roles.settle_armed) || self.physics_never_rests() {
            return;
        }
        let positions: Vec<_> = self.view.positions().collect();
        self.roles.settles += 1;
        if self.active_strategy.as_deref() == Some(SETTLED_ARRANGEMENT) {
            self.strategy_positions = Some(positions.clone());
            self.sync_arrangement_roles();
        }
        self.roles.settled = Some(positions);
    }

    /// A placement applied while Settled is active is Settled's own (a
    /// reopened scene).
    pub(crate) fn adopt_settled(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if self.active_strategy.as_deref() == Some(SETTLED_ARRANGEMENT) {
            self.roles.settled = Some(positions.to_vec());
        }
    }
}
