// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The shared commands as a typed vocabulary.
//!
//! [`ActionId`] is the typed form of `command-menu`'s catalogue (Scenograph
//! editor plan, SE49 and SE50): one variant per shared id, its key and label
//! read from the catalogue, so a command is defined once. ux-events and
//! `HostIntent` carry it where a typed id is wanted; hosts register the same
//! ids as strings. The egui-era catalogue it replaces (68 actions, four
//! categories) was audited in the plan's F12.

use command_menu::catalogue::{self, ids};
use serde::{Deserialize, Serialize};

/// A shared command.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Serialize, Deserialize)]
pub enum ActionId {
    NodeNew,
    NodeDelete,
    NodeEdit,
    NodePin,
    NodeUnpin,
    NodePinToggle,
    NodeViewerAuto,
    RelationAdd,
    RelationAddBoth,
    RelationRetract,
    IdentityWebfinger,
    IdentityActivitypub,
    IdentityNip05,
    WorkbenchSplitBeside,
    WorkbenchSplitOut,
    WorkbenchStackOnto,
    ViewFit,
    ViewZoomIn,
    ViewZoomOut,
    PhysicsToggle,
    PhysicsSettings,
    SessionUndo,
    SessionRedo,
    SessionSave,
    PaneSettings,
    PaneTrail,
    PaneWorkbench,
    PaletteOpen,
    ImportBookmarks,
    NavBack,
    NavForward,
    NavReload,
    NavStop,
}

impl ActionId {
    /// Every variant, in the catalogue's order.
    pub const ALL: [Self; 33] = [
        Self::NodeNew,
        Self::NodeDelete,
        Self::NodeEdit,
        Self::NodePin,
        Self::NodeUnpin,
        Self::NodePinToggle,
        Self::NodeViewerAuto,
        Self::RelationAdd,
        Self::RelationAddBoth,
        Self::RelationRetract,
        Self::IdentityWebfinger,
        Self::IdentityActivitypub,
        Self::IdentityNip05,
        Self::WorkbenchSplitBeside,
        Self::WorkbenchSplitOut,
        Self::WorkbenchStackOnto,
        Self::ViewFit,
        Self::ViewZoomIn,
        Self::ViewZoomOut,
        Self::PhysicsToggle,
        Self::PhysicsSettings,
        Self::SessionUndo,
        Self::SessionRedo,
        Self::SessionSave,
        Self::PaneSettings,
        Self::PaneTrail,
        Self::PaneWorkbench,
        Self::PaletteOpen,
        Self::ImportBookmarks,
        Self::NavBack,
        Self::NavForward,
        Self::NavReload,
        Self::NavStop,
    ];

    /// The shared id, `namespace:name`.
    pub fn key(self) -> &'static str {
        match self {
            Self::NodeNew => ids::NODE_NEW,
            Self::NodeDelete => ids::NODE_DELETE,
            Self::NodeEdit => ids::NODE_EDIT,
            Self::NodePin => ids::NODE_PIN,
            Self::NodeUnpin => ids::NODE_UNPIN,
            Self::NodePinToggle => ids::NODE_PIN_TOGGLE,
            Self::NodeViewerAuto => ids::NODE_VIEWER_AUTO,
            Self::RelationAdd => ids::RELATION_ADD,
            Self::RelationAddBoth => ids::RELATION_ADD_BOTH,
            Self::RelationRetract => ids::RELATION_RETRACT,
            Self::IdentityWebfinger => ids::IDENTITY_WEBFINGER,
            Self::IdentityActivitypub => ids::IDENTITY_ACTIVITYPUB,
            Self::IdentityNip05 => ids::IDENTITY_NIP05,
            Self::WorkbenchSplitBeside => ids::WORKBENCH_SPLIT_BESIDE,
            Self::WorkbenchSplitOut => ids::WORKBENCH_SPLIT_OUT,
            Self::WorkbenchStackOnto => ids::WORKBENCH_STACK_ONTO,
            Self::ViewFit => ids::VIEW_FIT,
            Self::ViewZoomIn => ids::VIEW_ZOOM_IN,
            Self::ViewZoomOut => ids::VIEW_ZOOM_OUT,
            Self::PhysicsToggle => ids::PHYSICS_TOGGLE,
            Self::PhysicsSettings => ids::PHYSICS_SETTINGS,
            Self::SessionUndo => ids::SESSION_UNDO,
            Self::SessionRedo => ids::SESSION_REDO,
            Self::SessionSave => ids::SESSION_SAVE,
            Self::PaneSettings => ids::PANE_SETTINGS,
            Self::PaneTrail => ids::PANE_TRAIL,
            Self::PaneWorkbench => ids::PANE_WORKBENCH,
            Self::PaletteOpen => ids::PALETTE_OPEN,
            Self::ImportBookmarks => ids::IMPORT_BOOKMARKS,
            Self::NavBack => ids::NAV_BACK,
            Self::NavForward => ids::NAV_FORWARD,
            Self::NavReload => ids::NAV_RELOAD,
            Self::NavStop => ids::NAV_STOP,
        }
    }

    /// The catalogue's label.
    pub fn label(self) -> &'static str {
        catalogue::label(self.key()).expect("every variant is in the catalogue")
    }

    /// What it acts on: the id's namespace.
    pub fn namespace(self) -> &'static str {
        catalogue::namespace(self.key())
    }

    /// The variant for a shared id.
    pub fn from_key(key: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|action| action.key() == key)
    }
}
