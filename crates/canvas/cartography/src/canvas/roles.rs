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
use super::law_inputs::{
    law_inputs_without_relation_edges, with_link_hides, with_meaning, with_registry,
};
use super::physics_catalog::PhysicsKindSource;
use super::reader::KeyMove;
use super::*;
use seiche::{Role, RoleTable};

/// The arrangement id of the latest settle (F30). Not a cartography adapter:
/// the canvas supplies its positions.
pub const SETTLED_ARRANGEMENT: &str = "settled";

/// A stop's return of anchored items, or a reset's return to the seed: where
/// they were and where they go. A host that adopts transitions animates
/// between the two ([`Canvas::preview_paused_positions`],
/// [`Canvas::land_paused_positions`]); the canvas has already placed `to`, so a
/// host that does not still lands.
#[derive(Clone, Debug, PartialEq)]
pub struct StopReturn {
    pub(super) source: super::host_results::LayoutResultSource,
    pub from: Vec<(NodeKey, PortablePoint)>,
    pub to: Vec<(NodeKey, PortablePoint)>,
    /// The ease's length; `None` is the arrangement transition's default.
    pub duration_ms: Option<u32>,
}

impl StopReturn {
    /// Capture the actual Canvas owner and structural inputs before handing a
    /// return to a delayed host. Ordinary title edits preserve this source.
    pub fn new(
        canvas: &Canvas,
        from: Vec<(NodeKey, PortablePoint)>,
        to: Vec<(NodeKey, PortablePoint)>,
        duration_ms: Option<u32>,
    ) -> Self {
        Self {
            source: super::host_results::LayoutResultSource::capture(canvas),
            from,
            to,
            duration_ms,
        }
    }
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
    /// Translation axes that the occurrence adapter binds directly to data.
    pub(crate) encoded: seiche::Axes,
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
            encoded: seiche::Axes::NONE,
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

impl ArrangementRoles {
    /// Retain runtime role state only for memberships that survived. Returns
    /// whether a keyboard move was cancelled, without releasing its old slot.
    pub(super) fn retain_members(&mut self, keep: impl Fn(&NodeKey) -> bool) -> bool {
        self.table.items.retain(|key, _| keep(key));
        self.pinned.retain(&keep);
        if let Some(settled) = &mut self.settled {
            settled.retain(|(key, _)| keep(key));
        }
        if let Some(ret) = &mut self.stop_return {
            ret.from.retain(|(key, _)| keep(key));
            ret.to.retain(|(key, _)| keep(key));
        }
        self.rest.retain_members(&keep);
        let cancelled = self.key_move.is_some_and(|move_| !keep(&move_.node));
        if cancelled {
            self.key_move = None;
        }
        if self.last_key_move.is_some_and(|move_| !keep(&move_.node)) {
            self.last_key_move = None;
        }
        cancelled
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
        let inputs = law_inputs_without_relation_edges(
            &self.graph,
            self.channels.community_held(),
            self.channels.sites_fresh(&self.graph),
        );
        let inputs = with_link_hides(
            inputs,
            &self.graph,
            &self.hidden_edges,
            &self.hidden_links,
            &self.hidden_assertions,
        );
        let inputs = with_meaning(inputs, self.meaning.snapshot());
        let inputs = with_registry(inputs, &self.channels, self.physics_view_key());
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

    pub(crate) fn arrangement_pin_installed(&self, key: NodeKey) -> bool {
        self.roles.pinned.contains(&key)
    }

    pub(crate) fn arrangement_axis_locks(&self) -> seiche::Axes {
        if self.roles.encoded == seiche::Axes::BOTH {
            seiche::Axes::NONE
        } else {
            self.roles.encoded
        }
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
        let held = self.arrangement_axis_locks();
        self.physics
            .set_axis_locks(self.view.positions().map(|(key, _)| (key, held)).collect());
        let force = (playing && !anchors.is_empty())
            .then(|| seiche::AnchorSpring::new(anchors).with_stiffness(self.roles.stiffness));
        self.physics.set_anchor_force(force);
    }
}

#[path = "roles/motion.rs"]
mod motion;
