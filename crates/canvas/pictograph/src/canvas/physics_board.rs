// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A physics board: the catalog's laws over items that are not a graph.
//!
//! A remote projection arrives as a scene — items with positions the
//! endpoint's arrangement produced — and is drawn from that score. This is
//! the seam that lets the physics catalog move those items too: a seiche
//! simulation with one body per item, the score's positions as the items'
//! **arrangement positions**, each playing a role ([`seiche::Role`]: seeded
//! by default, anchored items returning on [`seiche::AnchorSpring`] and, at
//! rest, gliding exactly home, pinned items held; dynamics grammar plan, G7),
//! and the chosen law and overlays
//! built through the same attribute builders the canvas uses, graph-free. An
//! axis the recipe encodes from a data field is held on that axis (F28). The
//! host syncs the board whenever the scene changes (a new item spawns at its
//! slot and enters with a settle burst; existing items keep their simulated
//! positions, save on an encoded axis, and only their slots move), ticks it
//! each frame, and reads positions back for drawing. The score itself is
//! never written: the board is the viewer's physics over the endpoint's
//! truth. (Physics catalog — P3.) What a person may do to a card is
//! advertised as chirograph actions, and the drag and the pin read them
//! ([`PhysicsBoard::advertised_actions`]; dynamics grammar plan, G9).
//!
//! The simulation runs behind [`seiche::Physics`], the stack's inline/actor
//! backend, so a board ticks in the frame loop on wasm and can be
//! [`offload`](PhysicsBoard::offload)ed onto an actor thread on native — the
//! same choice the canvas makes. (2026-09-04.)

use std::collections::{HashMap, HashSet};

use euclid::default::Point2D;
use kernel::graph::NodeKey;
use seiche::{
    AnchorSpring, Axes, DEFAULT_ANCHOR_STIFFNESS, LayoutView, Physics, Role, RoleTable, Simulation,
};

use crate::canvas::SETTLE_TICKS;
use crate::canvas::actions::{self, AdvertisedAction, ArrangementAction, PermittedActions};
use crate::canvas::at_rest::AtRest;
use crate::canvas::physics_catalog::{
    LawInputs, LawSources, PhysicsDepthSource, PhysicsKindSource, PhysicsLaw, PhysicsMassSource,
    PhysicsOverlay,
};

/// How firmly an anchored card returns to its slot. Gentle by design, a
/// twenty-fourth of the canvas's [`DEFAULT_ANCHOR_STIFFNESS`] (the board's
/// former pull), so an anchored score still shows the chosen law. A tunable;
/// cards are seeded unless a role says otherwise (F23).
pub const DEFAULT_BOARD_ANCHOR_STIFFNESS: f32 = DEFAULT_ANCHOR_STIFFNESS / 24.0;

/// The physics choice a board runs: what the host's own canvas runs, mirrored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PhysicsChoice {
    pub law: PhysicsLaw,
    pub overlays: Vec<PhysicsOverlay>,
    pub kind: PhysicsKindSource,
    pub mass: PhysicsMassSource,
    pub depth: PhysicsDepthSource,
}

impl Default for PhysicsChoice {
    fn default() -> Self {
        Self {
            law: PhysicsLaw::Springs,
            overlays: Vec::new(),
            kind: PhysicsKindSource::Site,
            mass: PhysicsMassSource::Degree,
            depth: PhysicsDepthSource::Roots,
        }
    }
}

/// One item the board holds: its stable id, its slot (the score position)
/// and its site (the grouping the Kinds law and the group overlay read).
#[derive(Clone, Debug, PartialEq)]
pub struct BoardItem {
    pub id: String,
    pub slot: (f32, f32),
    pub site: String,
}

/// The catalog's physics over a scene's items. See the module docs.
pub struct PhysicsBoard {
    physics: Physics,
    /// What the backend writes positions into, and `position` reads.
    view: LayoutView,
    keys: HashMap<String, NodeKey>,
    next_key: usize,
    items: Vec<BoardItem>,
    choice: PhysicsChoice,
    /// The roles the slots play; groups are sites.
    roles: RoleTable,
    anchor_stiffness: f32,
    /// The axes the recipe encodes from data fields, held on every card.
    encoded: Axes,
    /// The item currently held by the pointer. This is transient view state;
    /// score slots and `items` remain the arrangement authority.
    dragging: Option<NodeKey>,
    /// The settle record and anchored cards' at-rest return (F45, F46).
    rest: AtRest,
    /// Halted by the host: no settle is noted and nothing glides.
    halted: bool,
    /// What the binding withdraws from the cards' advertisements (G9).
    actions: PermittedActions,
    /// The host's device, when the board's repulsion is staged on it.
    #[cfg(feature = "gpu")]
    physics_device: Option<crate::canvas::PhysicsDevice>,
}

impl Default for PhysicsBoard {
    fn default() -> Self {
        Self::new()
    }
}

impl PhysicsBoard {
    pub fn new() -> Self {
        Self {
            physics: Physics::inline(Simulation::new(), 0),
            view: LayoutView::new(),
            keys: HashMap::new(),
            next_key: 0,
            items: Vec::new(),
            choice: PhysicsChoice::default(),
            roles: RoleTable::default(),
            anchor_stiffness: DEFAULT_BOARD_ANCHOR_STIFFNESS,
            encoded: Axes::NONE,
            dragging: None,
            rest: AtRest::default(),
            halted: false,
            actions: PermittedActions::default(),
            #[cfg(feature = "gpu")]
            physics_device: None,
        }
    }

    /// Move the board's simulation onto an actor thread (native hosts; a
    /// no-op once offloaded). `wake` pokes the host's event loop.
    pub fn offload(&mut self, wake: armillary::Wake) {
        self.physics.offload(wake);
    }

    /// Stage the board's repulsion on the host's device, or (`None`) return
    /// it to the CPU; see [`crate::canvas::Canvas::set_physics_device`].
    #[cfg(feature = "gpu")]
    pub fn set_physics_device(&mut self, device: Option<crate::canvas::PhysicsDevice>) {
        crate::canvas::physics_device::install(&mut self.physics, device.as_ref());
        self.physics_device = device;
    }

    /// The lane's counts (inline backend).
    #[cfg(feature = "gpu")]
    pub fn repulsion_stats(&self) -> Option<seiche::LaggedStats> {
        self.physics.repulsion_stats()
    }

    /// The live choice.
    pub fn choice(&self) -> &PhysicsChoice {
        &self.choice
    }

    /// The roles the slots play.
    pub fn roles(&self) -> &RoleTable {
        &self.roles
    }

    /// Set the roles (recipe default and site groups; items by id through
    /// [`Self::set_item_role`]); the anchors and pins follow.
    pub fn set_roles(&mut self, roles: RoleTable) {
        self.roles = roles;
        self.sync_roles();
        self.settle_for_choice();
        self.end_withdrawn_drag();
    }

    /// One card's role by id (`None` clears it). Returns whether it was set:
    /// not for an unknown card, nor a pin the card does not advertise (G9).
    pub fn set_item_role(&mut self, id: &str, role: Option<Role>) -> bool {
        let Some(&key) = self.keys.get(id) else {
            return false;
        };
        if role == Some(Role::Pinned) && !self.permits(id, ArrangementAction::Pin) {
            return false;
        }
        match role {
            Some(role) => self.roles.items.insert(key, role),
            None => self.roles.items.remove(&key),
        };
        self.sync_roles();
        self.settle_for_choice();
        self.end_withdrawn_drag();
        true
    }

    /// The actions card `id` advertises now, drag then pin (G9). A pinned
    /// card stays in its arrangement, so it advertises no drag (F47); a drag
    /// says which encoded axes return to their values on release (F28).
    /// What the binding withdraws is left out. Empty for an unknown card.
    pub fn advertised_actions(&self, id: &str) -> Vec<AdvertisedAction> {
        let Some(role) = self.role_of(id) else {
            return Vec::new();
        };
        let mut advertised = Vec::new();
        if self.actions.allows(ArrangementAction::Drag) && role != Role::Pinned {
            advertised
                .push(ArrangementAction::Drag.advertise(actions::board_drag(role, self.encoded)));
        }
        if self.actions.allows(ArrangementAction::Pin) {
            advertised.push(ArrangementAction::Pin.advertise(actions::BOARD_PIN));
        }
        advertised
    }

    /// Whether card `id` advertises `action`: what the drag and the pin ask.
    pub fn permits(&self, id: &str, action: ArrangementAction) -> bool {
        action.advertised_in(&self.advertised_actions(id))
    }

    /// What the binding withdraws.
    pub fn permitted_actions(&self) -> &PermittedActions {
        &self.actions
    }

    /// Replace what the binding withdraws. A drag under way whose card no
    /// longer advertises drag ends at once, as a release by its role.
    pub fn set_permitted_actions(&mut self, permitted: PermittedActions) {
        self.actions = permitted;
        self.end_withdrawn_drag();
    }

    /// End the drag under way if its card no longer advertises drag.
    fn end_withdrawn_drag(&mut self) {
        let Some(key) = self.dragging else {
            return;
        };
        let held = self
            .items
            .iter()
            .find(|item| self.keys.get(&item.id) == Some(&key))
            .map(|item| item.id.clone());
        if held.is_some_and(|id| !self.permits(&id, ArrangementAction::Drag)) {
            self.drag_end();
        }
    }

    /// The role a card plays.
    pub fn role_of(&self, id: &str) -> Option<Role> {
        let key = self.keys.get(id)?;
        let item = self.items.iter().find(|item| item.id == id)?;
        Some(self.roles.role(*key, Some(&item.site)))
    }

    /// How firmly an anchored card returns.
    pub fn anchor_stiffness(&self) -> f32 {
        self.anchor_stiffness
    }

    pub fn set_anchor_stiffness(&mut self, stiffness: f32) {
        self.anchor_stiffness = stiffness.max(0.0);
        self.sync_roles();
        self.settle_for_choice();
    }

    /// The axes the recipe encodes from data fields.
    pub fn encoded_axes(&self) -> Axes {
        self.encoded
    }

    /// The largest distance any card sits from its value on an encoded axis:
    /// zero while the encoding holds. A receipt asserts it.
    pub fn encoded_drift(&self) -> f32 {
        self.items
            .iter()
            .filter(|item| self.dragging != Some(self.keys[&item.id]))
            .filter_map(|item| {
                let at = self.view.position_of(self.keys[&item.id])?;
                let x = if self.encoded.x {
                    (at.x - item.slot.0).abs()
                } else {
                    0.0
                };
                let y = if self.encoded.y {
                    (at.y - item.slot.1).abs()
                } else {
                    0.0
                };
                Some(x.max(y))
            })
            .fold(0.0, f32::max)
    }

    /// Declare the encoded axes (F28): a card is held on each, and physics
    /// moves only the others. With both encoded no law acts, and only an
    /// exclusion at contact range separates overlapping cards.
    pub fn set_encoded_axes(&mut self, axes: Axes) {
        self.encoded = axes;
        self.sync_roles();
        self.rebuild_forces();
        self.settle_for_choice();
    }

    /// Switch the law, overlays and sources; the force set is replaced
    /// wholesale and a settle (or a continuous run) follows. A no-op when
    /// nothing changed, so a host may mirror its canvas every frame.
    pub fn set_choice(&mut self, choice: PhysicsChoice) {
        if choice == self.choice {
            return;
        }
        self.choice = choice;
        self.rebuild_forces();
        self.settle_for_choice();
    }

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
        self.physics.sync_edges(Vec::new());
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
    fn start_home(&mut self) {
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
    fn unpark(&mut self) {
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

    /// Halt motion for a paused board. An active drag is cancelled and its
    /// body is returned to the dynamic solver before the halt, so a later
    /// `sync` or choice change can explicitly reawaken the board.
    pub fn halt(&mut self) {
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

    /// The axes a card is held on: the encoded ones, except with both
    /// encoded, where only overlap separation moves it.
    fn held_axes(&self) -> Axes {
        if self.encoded == Axes::BOTH {
            Axes::NONE
        } else {
            self.encoded
        }
    }

    /// Realize the roles: anchored cards on springs, pinned cards kinematic
    /// at their slots, seeded cards free; and the encoded axes held.
    fn sync_roles(&mut self) {
        let mut anchors = Vec::new();
        let mut locks = Vec::new();
        let held = self.held_axes();
        for item in &self.items {
            let key = self.keys[&item.id];
            if self.dragging == Some(key) {
                continue;
            }
            match self.roles.role(key, Some(&item.site)) {
                Role::Seeded => self.physics.unpin(key),
                Role::Anchored => {
                    self.physics.unpin(key);
                    anchors.push((key, item.slot));
                },
                Role::Pinned => self
                    .physics
                    .pin(key, Point2D::new(item.slot.0, item.slot.1)),
            }
            locks.push((key, held));
        }
        self.physics.set_axis_locks(locks);
        let force = (self.anchor_stiffness > 0.0 && !anchors.is_empty())
            .then(|| AnchorSpring::new(anchors).with_stiffness(self.anchor_stiffness));
        self.physics.set_anchor_force(force);
    }

    fn rebuild_forces(&mut self) {
        let nodes: Vec<NodeKey> = self.items.iter().map(|item| self.keys[&item.id]).collect();
        let sites: HashMap<NodeKey, String> = self
            .items
            .iter()
            .map(|item| (self.keys[&item.id], item.site.clone()))
            .collect();
        if self.encoded == Axes::BOTH {
            // Both axes encoded: physics only separates overlaps (F28), by an
            // exclusion that reaches no further than contact.
            self.physics
                .set_forces(vec![Box::new(seiche::NodeExclusion {
                    cutoff: 2.0 * seiche::NODE_BODY_RADIUS,
                    ..Default::default()
                })]);
            return;
        }
        let inputs = LawInputs::from_parts(nodes, Vec::new(), sites);
        let sources = LawSources {
            kind: self.choice.kind,
            mass: self.choice.mass,
            depth: self.choice.depth,
            focus: None,
        };
        let forces = inputs.forces(self.choice.law, &self.choice.overlays, sources);
        self.physics.set_forces(forces);
    }

    /// Ask the backend for a settle: the normal burst, or — under a law or
    /// overlay that never rests — a budget that never runs out, which is how
    /// a perpetual law keeps its cards moving.
    fn settle_for_choice(&mut self) {
        self.unpark();
        self.halted = false;
        let living =
            self.choice.law.never_rests() || self.choice.overlays.iter().any(|o| o.never_rests());
        self.physics
            .settle(if living { u32::MAX } else { SETTLE_TICKS });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: &str, x: f32, y: f32) -> BoardItem {
        BoardItem {
            id: id.to_string(),
            slot: (x, y),
            site: "fixture".to_string(),
        }
    }

    fn settle(board: &mut PhysicsBoard) {
        for _ in 0..SETTLE_TICKS {
            board.tick();
        }
    }

    /// Anchored at the canvas's stiffness, items sit at their slots; a later
    /// item spawns at its slot and the earlier ones are not re-seeded. Anchored
    /// at the board's gentle stiffness the boundary force bends the shape a
    /// little (a body 300 units out drifts about thirty toward the origin), and
    /// seeded, the default, it bends further (the control).
    #[test]
    fn items_hold_their_slots_and_a_new_one_joins_without_reseeding() {
        let pair = || vec![item("a", 0.0, 0.0), item("b", 300.0, 0.0)];
        let mut gentle = PhysicsBoard::new();
        gentle.set_roles(RoleTable::uniform(Role::Anchored));
        gentle.sync(pair());
        settle(&mut gentle);
        let b = gentle.position("b").unwrap();
        assert!(
            (b.0 - 300.0).abs() < 60.0 && b.0 > 200.0,
            "the gentle anchor keeps the shape within a fifth: {b:?}"
        );
        let mut seeded = PhysicsBoard::new();
        seeded.sync(pair());
        settle(&mut seeded);
        let free = seeded.position("b").unwrap();
        assert!(free.0 < b.0 - 1.0, "seeded drifts further: {free:?} {b:?}");

        let mut board = PhysicsBoard::new();
        board.set_roles(RoleTable::uniform(Role::Anchored));
        board.set_anchor_stiffness(DEFAULT_ANCHOR_STIFFNESS);
        assert_eq!(
            board.sync(vec![item("a", 0.0, 0.0), item("b", 300.0, 0.0)]),
            2
        );
        for _ in 0..SETTLE_TICKS {
            board.tick();
        }
        let a = board.position("a").unwrap();
        let b = board.position("b").unwrap();
        assert!(
            a.0.abs() < 20.0 && (b.0 - 300.0).abs() < 20.0,
            "at the slots: {a:?} {b:?}"
        );
        // Nudge a off its slot by moving its slot, then add c: a keeps its
        // simulated position (not teleported to the new slot), c spawns at its own.
        assert_eq!(
            board.sync(vec![
                item("a", 60.0, 0.0),
                item("b", 300.0, 0.0),
                item("c", 150.0, 200.0)
            ]),
            1
        );
        let a_after = board.position("a").unwrap();
        assert!(
            (a_after.0 - a.0).abs() < 1.0,
            "a was not re-seeded: {a_after:?}"
        );
        let c = board.position("c").unwrap();
        assert!(
            (c.0 - 150.0).abs() < 1.0 && (c.1 - 200.0).abs() < 1.0,
            "c at its slot: {c:?}"
        );
        assert_eq!(board.len(), 3);
        // A departed item drops.
        board.sync(vec![item("b", 300.0, 0.0)]);
        assert!(board.position("a").is_none());
        assert_eq!(board.len(), 1);
    }

    #[test]
    fn drag_pins_one_item_and_release_returns_toward_its_slot() {
        let mut board = PhysicsBoard::new();
        board.set_roles(RoleTable::uniform(Role::Anchored));
        board.set_anchor_stiffness(DEFAULT_ANCHOR_STIFFNESS);
        board.sync(vec![item("a", 0.0, 0.0), item("b", 220.0, 0.0)]);
        for _ in 0..SETTLE_TICKS {
            board.tick();
        }

        let start = board.position("a").unwrap();
        assert!(board.drag_start("a"));
        assert!(!board.drag_start("b"));
        assert!(board.drag_move(140.0, 70.0));
        assert!(board.tick());
        let held = board.position("a").unwrap();
        assert!((held.0 - 140.0).abs() < 1.0 && (held.1 - 70.0).abs() < 1.0);
        assert!(board.is_settling());

        assert!(board.drag_end());
        assert!(!board.drag_move(20.0, 20.0));
        for _ in 0..SETTLE_TICKS {
            board.tick();
        }
        let released = board.position("a").unwrap();
        assert!(
            (released.0 - 0.0).abs() < (held.0 - 0.0).abs(),
            "release should let the item return toward its slot: start={start:?}, held={held:?}, released={released:?}"
        );
    }

    #[test]
    fn halt_cancels_drag_and_sync_removal_clears_it() {
        let mut board = PhysicsBoard::new();
        board.sync(vec![item("a", 0.0, 0.0), item("b", 120.0, 0.0)]);
        assert!(board.drag_start("a"));
        assert!(!board.drag_move(f32::NAN, 4.0));
        board.halt();
        assert!(!board.drag_move(20.0, 20.0));
        assert!(!board.is_settling());

        assert!(board.drag_start("b"));
        board.sync(vec![item("a", 0.0, 0.0)]);
        assert!(!board.drag_move(20.0, 20.0));
        assert!(!board.drag_end());
    }

    /// Under Charge, two seeded items whose slots overlap settle apart; two
    /// anchored ones are pushed apart while moving and, at rest, glide back
    /// to their slots, overlapping as their arrangement does (F45). Under
    /// Orbit the board keeps moving.
    #[test]
    fn charge_separates_overlapping_slots_and_orbit_never_rests() {
        let pair = || vec![item("a", 0.0, 0.0), item("b", 4.0, 2.0)];
        let charge = PhysicsChoice {
            law: PhysicsLaw::Charge,
            ..PhysicsChoice::default()
        };
        let mut anchored = PhysicsBoard::new();
        anchored.set_roles(RoleTable::uniform(Role::Anchored));
        anchored.sync(pair());
        anchored.set_choice(charge.clone());
        let mut widest: f32 = 0.0;
        for _ in 0..SETTLE_TICKS * 4 {
            anchored.tick();
            widest = widest.max(anchored.gap("a", "b").unwrap());
        }
        assert!(
            widest > 30.0,
            "charge pushed the anchored pair apart: {widest:.0}"
        );
        assert_eq!(anchored.anchored_home_count(), 2, "both came home at rest");
        assert_eq!(anchored.position("b"), Some((4.0, 2.0)), "exactly home");

        let mut board = PhysicsBoard::new();
        board.sync(pair());
        board.set_choice(charge);
        for _ in 0..SETTLE_TICKS {
            board.tick();
        }
        let gap = board.gap("a", "b").unwrap();
        assert!(gap > 60.0, "charge pushed the seeded pair apart: {gap:.0}");
        assert!(!board.tick(), "charge comes to rest");

        board.set_choice(PhysicsChoice {
            law: PhysicsLaw::Orbit,
            ..PhysicsChoice::default()
        });
        for _ in 0..120 {
            assert!(board.tick(), "orbit keeps ticking");
        }
        assert!(
            board.energy() > 0.0,
            "orbit carries energy: {}",
            board.energy()
        );
    }
}
