// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Arrangement roles on the canvas (dynamics grammar plan, G7).
//!
//! The active arrangement's positions play the roles of a
//! [`seiche::RoleTable`]: seeded by default (F23); anchored items return,
//! by an anchor spring while the graph moves and, once it rests, by a short
//! transition that takes each exactly home, where it holds until the graph is
//! next disturbed (F45), and by jumping back when a paused drag ends (F19);
//! pinned items are kinematic at their position. An explicit pin
//! (`pin_focused`, a nudge) is the per-item pinned case (F22). A pick while
//! playing keeps playing from the landed positions, and a stop returns the
//! anchored items while seeded and pinned ones stay (F24). A settle is the
//! bodies' rms speed falling under [`SETTLE_SPEED_FLOOR`] (F46). "Settled" is
//! the latest settle's positions, picked like any other arrangement (F30).

use super::actions::{ArrangementAction, PermittedActions};
use super::at_rest::AtRest;
use super::channels::Channel;
use super::physics_catalog::{LawInputs, PhysicsKindSource};
use super::reader::KeyMove;
use super::*;
use seiche::{Role, RoleTable};

/// The arrangement id of the latest settle (F30). Not a cartography adapter:
/// the canvas supplies its positions.
pub const SETTLED_ARRANGEMENT: &str = "settled";

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
    /// The `groups.*` channel a group override is read from: the site by
    /// default, any groups channel since G2 (F49, "Site, then G2 channels").
    group_source: PhysicsKindSource,
    /// The anchored role's return stiffness ([`seiche::AnchorSpring`]).
    pub(crate) stiffness: f32,
    /// Bodies the table holds kinematic, so a role change can free them.
    pinned: HashSet<NodeKey>,
    /// A pick made while playing resumes once its final placement lands.
    resume_after_pick: bool,
    settled: Option<Vec<(NodeKey, PortablePoint)>>,
    settles: u64,
    stop_return: Option<StopReturn>,
    /// The settle record and the at-rest return of anchored items.
    rest: AtRest,
    /// What the binding withdraws from the items' advertisements (G9).
    pub(crate) actions: PermittedActions,
    /// A keyboard move under way, the pointerless drag (F67).
    pub(crate) key_move: Option<KeyMove>,
    /// The latest keyboard move, kept after it ends. Receipt introspection.
    pub(crate) last_key_move: Option<KeyMove>,
}

impl Default for ArrangementRoles {
    fn default() -> Self {
        Self {
            table: RoleTable::default(),
            group_source: PhysicsKindSource::Site,
            stiffness: seiche::DEFAULT_ANCHOR_STIFFNESS,
            pinned: HashSet::new(),
            resume_after_pick: false,
            settled: None,
            settles: 0,
            stop_return: None,
            rest: AtRest::default(),
            actions: PermittedActions::default(),
            key_move: None,
            last_key_move: None,
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

    /// A group's role (`None` clears it), by the group's label under the
    /// role group source ([`Self::role_group_of`]): a site's URL host, or
    /// `groups.<source>#<id>` for another groups channel, the id its smallest
    /// member's (F134).
    pub fn set_group_role(&mut self, group: &str, role: Option<Role>) {
        match role {
            Some(role) => self.roles.table.groups.insert(group.to_string(), role),
            None => self.roles.table.groups.remove(group),
        };
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
    }

    /// One item's role (`None` clears it), by member id. Returns whether it
    /// was set: not for a member the graph lacks, nor a pin the item does not
    /// advertise (G9).
    pub fn set_member_role(&mut self, member: uuid::Uuid, role: Option<Role>) -> bool {
        let Some(key) = self.graph.get_node_key_by_id(member) else {
            return false;
        };
        if role == Some(Role::Pinned) && !self.permits(key, ArrangementAction::Pin) {
            return false;
        }
        match role {
            Some(role) => self.roles.table.items.insert(key, role),
            None => self.roles.table.items.remove(&key),
        };
        self.sync_arrangement_roles();
        self.settle_physics(SETTLE_TICKS);
        true
    }

    /// The `groups.*` channel group overrides are read from (F49).
    pub fn role_group_source(&self) -> PhysicsKindSource {
        self.roles.group_source
    }

    /// Read group overrides from `source`'s groups (F49): `groups.site` by
    /// default, or any groups channel, `groups.cluster` and `groups.meaning`
    /// among them, as Group pull's source is chosen. Overrides keep their
    /// labels; a label the new source does not name matches no node.
    pub fn set_role_group_source(&mut self, source: PhysicsKindSource) {
        if self.roles.group_source != source {
            self.roles.group_source = source;
            self.sync_arrangement_roles();
            self.settle_physics(SETTLE_TICKS);
        }
    }

    /// Bring the role group source's partition up to the graph: the
    /// registry's Louvain partition, or a Meaning run.
    fn refresh_role_groups(&mut self) {
        match self.roles.group_source {
            PhysicsKindSource::Cluster => self.ensure_community_fresh(),
            PhysicsKindSource::Meaning => {
                self.refresh_meaning();
            },
            _ => {},
        }
    }

    /// Every node's group label under a source other than the site, from the
    /// law inputs (the registry's partition and the Meaning snapshot as they
    /// stand); `None` under the site, whose label each node reads alone.
    fn role_group_labels(&self) -> Option<HashMap<NodeKey, String>> {
        let source = self.roles.group_source;
        if source == PhysicsKindSource::Site {
            return None;
        }
        let channel = Channel::Groups(source).id();
        let inputs = LawInputs::new(
            &self.graph,
            &self.hidden_edges,
            self.channels.community_held(),
            self.channels.sites_fresh(&self.graph),
        )
        .with_meaning(self.meaning.snapshot());
        // A group is named by its smallest member's stable id (F134), which
        // survives the partition renumbering its groups.
        let groups = inputs.groups(source);
        let mut smallest: HashMap<u32, uuid::Uuid> = HashMap::new();
        for (key, group) in &groups {
            if let Some(node) = self.graph.get_node(*key) {
                let id = smallest.entry(*group).or_insert(node.id);
                *id = (*id).min(node.id);
            }
        }
        Some(
            groups
                .into_iter()
                .filter_map(|(key, group)| {
                    Some((key, format!("{channel}#{}", smallest.get(&group)?)))
                })
                .collect(),
        )
    }

    /// `key`'s group label, read from `labels` (the role group source's) or,
    /// under the site, from the node.
    fn role_group_in(
        &self,
        key: NodeKey,
        labels: &Option<HashMap<NodeKey, String>>,
    ) -> Option<String> {
        match labels {
            Some(labels) => labels.get(&key).cloned(),
            None => self
                .graph
                .get_node(key)
                .map(|node| Graph::url_grouping_key(node.url()).to_string()),
        }
    }

    /// The group a node's role override is read from, under the role group
    /// source: its site by default (F49).
    pub fn role_group_of(&self, key: NodeKey) -> Option<String> {
        self.role_group_in(key, &self.role_group_labels())
    }

    /// The role `key` plays, its group read from `labels`.
    fn role_in(&self, key: NodeKey, labels: &Option<HashMap<NodeKey, String>>) -> Role {
        if self.pinned_nodes.contains(&key) {
            return Role::Pinned;
        }
        self.roles
            .table
            .role(key, self.role_group_in(key, labels).as_deref())
    }

    /// Every slot's role, the group labels read once.
    fn roles_of(&self, keys: impl IntoIterator<Item = NodeKey>) -> HashMap<NodeKey, Role> {
        let labels = self.role_group_labels();
        keys.into_iter()
            .map(|key| (key, self.role_in(key, &labels)))
            .collect()
    }

    /// One member's own role, when it overrides its group's and the recipe's
    /// (F48, the detail panel).
    pub fn member_role(&self, member: uuid::Uuid) -> Option<Role> {
        let key = self.graph.get_node_key_by_id(member)?;
        self.roles.table.items.get(&key).copied()
    }

    /// The role `key` plays now: an explicit pin first, then the table.
    pub fn arrangement_role_of(&self, key: NodeKey) -> Role {
        self.role_in(key, &self.role_group_labels())
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
        self.refresh_role_groups();
        let playing = !self.physics_paused;
        let mut pinned = HashSet::new();
        let mut anchors = Vec::new();
        let dragged = self.drag.map(|d| d.node);
        if let Some(slots) = self.strategy_positions.clone() {
            let roles = self.roles_of(slots.iter().map(|(key, _)| *key));
            for (key, at) in slots {
                // Explicit pins and the body under the pointer are input's.
                if self.pinned_nodes.contains(&key) {
                    continue;
                }
                let role = roles[&key];
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
        let roles = self.roles_of(from.iter().map(|(key, _)| *key));
        let to: Vec<_> = from
            .iter()
            .map(|&(key, at)| {
                if roles[&key] == Role::Anchored
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

    /// The roles' share of a frame: note a settle, then glide anchored items
    /// home. Called right after the physics snapshot lands in the view.
    pub(crate) fn advance_roles(&mut self) {
        self.note_settle();
        self.advance_home();
        self.advance_schedule();
    }

    /// Fold one frame into the settle record: a playing graph whose bodies'
    /// rms speed falls under the floor has settled (F46). Once per rest, the
    /// rest replaces Settled and anchored items start home (F45). A law that
    /// never rests does neither.
    fn note_settle(&mut self) {
        let budget_ended = self.roles.rest.budget_ended(self.physics.is_settling());
        if self.physics_paused {
            return;
        }
        if self.drag.is_some_and(|d| d.moved) {
            self.roles.rest.arm();
            return;
        }
        if !self.roles.rest.rested(self.physics.rms_speed()) || self.physics_never_rests() {
            // A budget spent with the bodies still moving is no settle, but
            // anchored items still go home (F63).
            if budget_ended {
                self.start_home();
            }
            return;
        }
        let positions: Vec<_> = self.view.positions().collect();
        self.roles.settles += 1;
        if self.active_strategy.as_deref() == Some(SETTLED_ARRANGEMENT) {
            self.strategy_positions = Some(positions.clone());
            self.sync_arrangement_roles();
        }
        self.roles.settled = Some(positions);
        self.start_home();
    }

    /// At rest, every anchored item away from its position starts home.
    fn start_home(&mut self) {
        let roles = self.roles_of(self.view.positions().map(|(key, _)| key));
        let glide: Vec<_> = self
            .view
            .positions()
            .filter(|&(key, _)| {
                !self.pinned_nodes.contains(&key) && roles.get(&key) == Some(&Role::Anchored)
            })
            .filter_map(|(key, at)| Some((key, at, self.arrangement_slot(key)?)))
            .collect();
        self.roles.rest.start(&mut self.physics, glide);
    }

    /// One frame of the glide home; at its end the items hold at home.
    fn advance_home(&mut self) {
        self.roles.rest.step(&mut self.physics);
    }

    /// A disturbance (a settle request, a drag, a pause or play) releases
    /// the items held at home and stops a glide, so the spring acts again.
    pub(crate) fn unpark(&mut self) {
        let roles = self.roles_of(self.view.positions().map(|(key, _)| key));
        let keep = |key: NodeKey| roles.get(&key) == Some(&Role::Pinned);
        self.roles.rest.release(&mut self.physics, keep);
    }

    /// How many anchored items are held at home after an at-rest return.
    /// Receipt introspection.
    pub fn anchored_home_count(&self) -> usize {
        self.roles.rest.parked_count()
    }

    /// The frame's layout stage alone, without composing a scene: the
    /// physics snapshot, the paused placement, and the roles. For receipts
    /// that run thousands of frames.
    #[cfg(test)]
    pub(crate) fn step_layout(&mut self) {
        self.physics.advance_frame(&mut self.view);
        self.apply_strategy_to_view();
        self.advance_roles();
    }

    /// A placement applied while Settled is active is Settled's own (a
    /// reopened scene).
    pub(crate) fn adopt_settled(&mut self, positions: &[(NodeKey, PortablePoint)]) {
        if self.active_strategy.as_deref() == Some(SETTLED_ARRANGEMENT) {
            self.roles.settled = Some(positions.to_vec());
        }
    }
}
