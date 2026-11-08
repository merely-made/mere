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
use crate::canvas::composition::PhysicsComposition;
use crate::canvas::dynamics_spec::{BindError, BoundRoot, DynamicsSpec, bind};
use crate::canvas::physics_catalog::{LawInputs, LawSources};
use crate::canvas::physics_view::PhysicsChoice;

/// How firmly an anchored card returns to its slot. Gentle by design, a
/// twenty-fourth of the canvas's [`DEFAULT_ANCHOR_STIFFNESS`] (the board's
/// former pull), so an anchored score still shows the chosen law. A tunable;
/// cards are seeded unless a role says otherwise (F23).
pub const DEFAULT_BOARD_ANCHOR_STIFFNESS: f32 = DEFAULT_ANCHOR_STIFFNESS / 24.0;

/// Why a board refuses a schedule.
const SCHEDULE_ON_A_BOARD: &str =
    "a board runs a law or a composition; the canvas drives a schedule's stages";

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
    /// Pairs of item ids the law reads as edges (stack-lift D3).
    links: Vec<(String, String)>,
    /// The stage the board runs, as the spec it was given (F150, F162).
    stage: DynamicsSpec,
    /// That stage read: the flat choice, a composition in the law slot, and
    /// the seed.
    choice: PhysicsChoice,
    composition: Option<PhysicsComposition>,
    seed: u64,
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
    /// A keyboard move under way (F67): where it began, and where the card is
    /// held now.
    key_move: Option<((f32, f32), (f32, f32))>,
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
            links: Vec::new(),
            stage: PhysicsChoice::default().into_spec(),
            choice: PhysicsChoice::default(),
            composition: None,
            seed: crate::canvas::dynamics_spec::DEFAULT_SEED,
            roles: RoleTable::default(),
            anchor_stiffness: DEFAULT_BOARD_ANCHOR_STIFFNESS,
            encoded: Axes::NONE,
            dragging: None,
            rest: AtRest::default(),
            halted: false,
            actions: PermittedActions::default(),
            key_move: None,
            #[cfg(feature = "gpu")]
            physics_device: None,
        }
    }

    /// Set the board's simulation speed (see [`seiche::Speed`]).
    pub fn set_speed(&mut self, speed: seiche::Speed) {
        self.physics.set_speed(speed);
    }

    /// Bound a frame's ticks above real time (see [`seiche::StepBudget`]).
    pub fn set_step_budget(&mut self, budget: Option<seiche::StepBudget>) {
        self.physics.set_step_budget(budget);
    }

    /// Ticks run, the effective speed reached, and whether the budget bound.
    pub fn pace(&self) -> seiche::PaceStats {
        self.physics.pace()
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

    /// The stage the board runs: a law or a composition, as a spec. A
    /// reader reads it through [`PhysicsChoice::view`].
    pub fn stage(&self) -> &DynamicsSpec {
        &self.stage
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

    /// Run `stage`: a law or a composition with its overlays, sources and
    /// seed (dynamics grammar plan, F150, F162), the canvas's
    /// [`live_stage_spec`](crate::canvas::Canvas::live_stage_spec) when the
    /// board mirrors one. The force set is replaced wholesale and a settle
    /// (or a continuous run) follows. A schedule is refused: the canvas
    /// drives one and the board follows its stage. A no-op when nothing
    /// changed, so a host may mirror its canvas every frame. The board has
    /// no partition or Meaning snapshot of its own, so a cluster or meaning
    /// source reads site.
    pub fn set_stage(&mut self, stage: &DynamicsSpec) -> Result<(), BindError> {
        if *stage == self.stage {
            return Ok(());
        }
        let bound = bind(stage)?;
        let composition = match &bound.root {
            BoundRoot::Law { .. } => None,
            BoundRoot::Composed { composition, .. } => Some(composition.clone()),
            BoundRoot::Schedule(_) => {
                return Err(BindError::Unbound {
                    place: "root".into(),
                    reason: SCHEDULE_ON_A_BOARD.into(),
                });
            },
        };
        self.seed = bound.seed;
        self.choice = PhysicsChoice::flat(bound);
        self.composition = composition;
        self.stage = stage.clone();
        self.rebuild_forces();
        self.settle_for_choice();
        Ok(())
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
        let inputs = LawInputs::from_parts(nodes, self.edges(), sites);
        let sources = LawSources {
            kind: self.choice.kind,
            groups: self.choice.groups,
            mass: self.choice.mass,
            depth: self.choice.depth,
            focus: None,
            seed: self.seed,
        };
        // The board reads no channel registry (its inputs carry none), so the law
        // goes to the backend as data: the descriptor builds the same forces
        // where the backend runs, and computes the same channels, uncached either way.
        self.physics.set_force_set(inputs.describe(
            self.choice.law,
            self.composition.as_ref(),
            &self.choice.overlays,
            sources,
        ));
    }

    /// Ask the backend for a settle: the normal burst, or — under a law or
    /// overlay that never rests — a budget that never runs out, which is how
    /// a perpetual law keeps its cards moving.
    fn settle_for_choice(&mut self) {
        self.unpark();
        self.halted = false;
        let laws = self
            .composition
            .as_ref()
            .map_or_else(|| vec![self.choice.law], PhysicsComposition::laws);
        let living = laws.iter().any(|law| law.never_rests())
            || self.choice.overlays.iter().any(|o| o.never_rests());
        self.physics
            .settle(if living { u32::MAX } else { SETTLE_TICKS });
    }
}

#[cfg(test)]
#[path = "physics_board/tests.rs"]
mod tests;

#[path = "physics_board/runtime.rs"]
mod runtime;
