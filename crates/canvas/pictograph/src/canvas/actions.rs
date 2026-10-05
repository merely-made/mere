// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Permitted actions (dynamics grammar plan, G9; stack seams plan, S3).
//!
//! What a person may do to an item's arrangement position is advertised as
//! chirograph's [`AdvertisedAction`], the one model accessibility and
//! permission surfaces read (S3, "One model: AdvertisedAction"). Drag and pin
//! carry [`IntentEffect::Curation`]: positions are view state, not graph
//! truth. A gesture asks its item's advertisement and nothing else, so a
//! refusal is an action left out, never a flag of its own:
//!
//! - the canvas advertises drag for every item, a pinned one included, whose
//!   drag moves its pin (F47);
//! - the board leaves drag out for a pinned card, which stays in its
//!   arrangement (F47), and its drag says which encoded axes return to their
//!   values on release (F28; [`PhysicsBoard::advertised_actions`]);
//! - both advertise pin, which every per-item pin reads;
//! - a binding may withdraw either ([`PermittedActions`]), and a withdrawal
//!   while a drag is under way ends the drag at once, as a release.
//!
//! [`PhysicsBoard::advertised_actions`]: super::PhysicsBoard::advertised_actions

use std::collections::BTreeSet;

pub use chirograph::{AdvertisedAction, IntentEffect, IntentReference};
use kernel::graph::NodeKey;
use seiche::{Axes, Role};

use super::Canvas;

/// Drag: move an item by its arrangement position.
pub const DRAG_INTENT: &str = "mere.arrangement.drag";
/// A pointer carries the drag itself, so nothing composes this payload yet;
/// a surface that invokes the action without one defines it here.
pub const DRAG_SCHEMA: &str = "mere.arrangement.drag/v1";
/// Pin: hold an item in place.
pub const PIN_INTENT: &str = "mere.arrangement.pin";
/// Pin takes no payload.
pub const PIN_SCHEMA: &str = "mere.arrangement.pin/v1";

/// The actions a binding may permit on an item's arrangement position.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ArrangementAction {
    Drag,
    Pin,
}

impl ArrangementAction {
    pub const ALL: [Self; 2] = [Self::Drag, Self::Pin];

    pub fn intent(self) -> &'static str {
        match self {
            Self::Drag => DRAG_INTENT,
            Self::Pin => PIN_INTENT,
        }
    }

    pub fn payload_schema(self) -> &'static str {
        match self {
            Self::Drag => DRAG_SCHEMA,
            Self::Pin => PIN_SCHEMA,
        }
    }

    /// The plain label a surface shows.
    pub fn label(self) -> &'static str {
        match self {
            Self::Drag => "Drag",
            Self::Pin => "Pin",
        }
    }

    /// This action as an item advertises it, with the item's explanation.
    pub fn advertise(self, explanation: impl Into<String>) -> AdvertisedAction {
        AdvertisedAction {
            intent: IntentReference(self.intent().to_string()),
            label: self.label().to_string(),
            explanation: explanation.into(),
            payload_schema: self.payload_schema().to_string(),
            input_form: None,
            effect: IntentEffect::Curation,
        }
    }

    /// Whether `actions` carries this action: the one test a gesture makes.
    pub fn advertised_in(self, actions: &[AdvertisedAction]) -> bool {
        actions
            .iter()
            .any(|action| action.intent.0 == self.intent())
    }
}

/// What a binding withdraws. Nothing by default, so each action is
/// advertised wherever the item's role allows it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PermittedActions {
    withdrawn: BTreeSet<ArrangementAction>,
}

impl PermittedActions {
    /// Every action permitted.
    pub fn all() -> Self {
        Self::default()
    }

    /// Every action but `action`.
    pub fn without(action: ArrangementAction) -> Self {
        let mut permitted = Self::default();
        permitted.withdraw(action);
        permitted
    }

    pub fn withdraw(&mut self, action: ArrangementAction) {
        self.withdrawn.insert(action);
    }

    pub fn restore(&mut self, action: ArrangementAction) {
        self.withdrawn.remove(&action);
    }

    pub fn allows(&self, action: ArrangementAction) -> bool {
        !self.withdrawn.contains(&action)
    }
}

/// The canvas's pin, by any per-item path: an explicit pin or the item's
/// pinned role.
const CANVAS_PIN: &str =
    "Pin this item; it stays held in place, and a drag moves where it is held.";

/// The board's pin: the card's pinned role.
pub(crate) const BOARD_PIN: &str =
    "Pin this card at its arrangement position; a pinned card refuses a drag.";

/// What a canvas drag does on release, by the item's role (F19, F47).
fn canvas_drag(role: Role, returns_home: bool) -> &'static str {
    match role {
        Role::Pinned => "Drag this item's pin; it stays held where you drop it.",
        Role::Anchored if returns_home => {
            "Drag this item; it returns to its arrangement position when you let go."
        },
        _ => "Drag this item; nothing returns it.",
    }
}

/// What a board drag does on release, by the card's role and the encoded
/// axes, which return to their values at once (F28). Not for a pinned card,
/// which advertises no drag.
pub(crate) fn board_drag(role: Role, encoded: Axes) -> String {
    let free = match role {
        Role::Anchored => "it returns to its arrangement position",
        _ => "nothing returns it",
    };
    let axis = match (encoded.x, encoded.y) {
        (true, true) => {
            return "Drag this card; when you let go, x and y return to their values.".into();
        },
        (true, false) => "x",
        (false, true) => "y",
        (false, false) => return format!("Drag this card; {free} when you let go."),
    };
    format!(
        "Drag this card; when you let go, {axis} returns to its value, and on the other axis {free}."
    )
}

impl Canvas {
    /// The actions `member` advertises now, drag then pin: what a surface
    /// lists and what the canvas's gestures read. Empty for a member the
    /// graph lacks.
    pub fn advertised_actions(&self, member: uuid::Uuid) -> Vec<AdvertisedAction> {
        self.graph
            .get_node_key_by_id(member)
            .map(|key| self.actions_of(key))
            .unwrap_or_default()
    }

    pub(crate) fn actions_of(&self, key: NodeKey) -> Vec<AdvertisedAction> {
        let permitted = &self.roles.actions;
        let mut actions = Vec::new();
        if permitted.allows(ArrangementAction::Drag) {
            let returns_home = self.arrangement_slot(key).is_some();
            actions.push(
                ArrangementAction::Drag
                    .advertise(canvas_drag(self.arrangement_role_of(key), returns_home)),
            );
        }
        if permitted.allows(ArrangementAction::Pin) {
            actions.push(ArrangementAction::Pin.advertise(CANVAS_PIN));
        }
        actions
    }

    /// Whether `key` advertises `action`.
    pub(crate) fn permits(&self, key: NodeKey, action: ArrangementAction) -> bool {
        action.advertised_in(&self.actions_of(key))
    }

    /// What the binding withdraws.
    pub fn permitted_actions(&self) -> &PermittedActions {
        &self.roles.actions
    }

    /// Replace what the binding withdraws. A drag under way whose item no
    /// longer advertises drag ends at once, as a release by its role.
    pub fn set_permitted_actions(&mut self, permitted: PermittedActions) {
        self.roles.actions = permitted;
        if let Some(drag) = self.drag
            && drag.moved
            && !self.permits(drag.node, ArrangementAction::Drag)
        {
            self.drag = None;
            self.physics.set_dragging(false);
            self.release_dragged(drag.node);
        }
    }
}
