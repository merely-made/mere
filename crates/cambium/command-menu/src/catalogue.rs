/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The stack's shared command ids (Scenograph editor plan, SE49 and SE50).
//!
//! A verb gets an id here when a second host offers it or ux-events reasons
//! about it; every other command keeps a host-owned id in the open set. Ids
//! are `namespace:name`, the namespace naming what the verb acts on. Hosts
//! register these ids for these verbs, graph-kernel's `ActionId` is their
//! typed form, and registry's default key bindings name them.

use crate::Command;

/// The ids, one constant each.
pub mod ids {
    pub const NODE_NEW: &str = "node:new";
    pub const NODE_DELETE: &str = "node:delete";
    pub const NODE_EDIT: &str = "node:edit";
    pub const NODE_PIN: &str = "node:pin";
    pub const NODE_UNPIN: &str = "node:unpin";
    pub const NODE_PIN_TOGGLE: &str = "node:pin_toggle";
    pub const NODE_VIEWER_AUTO: &str = "node:viewer_auto";
    pub const RELATION_ADD: &str = "relation:add";
    pub const RELATION_ADD_BOTH: &str = "relation:add_both";
    pub const RELATION_RETRACT: &str = "relation:retract";
    pub const IDENTITY_WEBFINGER: &str = "identity:webfinger";
    pub const IDENTITY_ACTIVITYPUB: &str = "identity:activitypub";
    pub const IDENTITY_NIP05: &str = "identity:nip05";
    pub const WORKBENCH_SPLIT_BESIDE: &str = "workbench:split_beside";
    pub const WORKBENCH_SPLIT_OUT: &str = "workbench:split_out";
    pub const WORKBENCH_STACK_ONTO: &str = "workbench:stack_onto";
    pub const VIEW_FIT: &str = "view:fit";
    pub const VIEW_ZOOM_IN: &str = "view:zoom_in";
    pub const VIEW_ZOOM_OUT: &str = "view:zoom_out";
    pub const PHYSICS_TOGGLE: &str = "physics:toggle";
    pub const PHYSICS_SETTINGS: &str = "physics:settings";
    pub const SESSION_UNDO: &str = "session:undo";
    pub const SESSION_REDO: &str = "session:redo";
    pub const SESSION_SAVE: &str = "session:save";
    pub const PANE_SETTINGS: &str = "pane:settings";
    pub const PANE_TRAIL: &str = "pane:trail";
    pub const PANE_WORKBENCH: &str = "pane:workbench";
    pub const PALETTE_OPEN: &str = "palette:open";
    pub const IMPORT_BOOKMARKS: &str = "import:bookmarks";
    pub const NAV_BACK: &str = "nav:back";
    pub const NAV_FORWARD: &str = "nav:forward";
    pub const NAV_RELOAD: &str = "nav:reload";
    pub const NAV_STOP: &str = "nav:stop";
}

/// A shared id and its label.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub id: &'static str,
    pub label: &'static str,
}

const fn entry(id: &'static str, label: &'static str) -> Entry {
    Entry { id, label }
}

/// Every shared id with its label, in the plan's order (F13).
pub const CATALOGUE: &[Entry] = &[
    entry(ids::NODE_NEW, "New node"),
    entry(ids::NODE_DELETE, "Delete node"),
    entry(ids::NODE_EDIT, "Edit node"),
    entry(ids::NODE_PIN, "Pin"),
    entry(ids::NODE_UNPIN, "Unpin"),
    entry(ids::NODE_PIN_TOGGLE, "Pin or unpin"),
    entry(ids::NODE_VIEWER_AUTO, "Choose viewer automatically"),
    entry(ids::RELATION_ADD, "Add relation"),
    entry(ids::RELATION_ADD_BOTH, "Add relation both ways"),
    entry(ids::RELATION_RETRACT, "Retract relation"),
    entry(ids::IDENTITY_WEBFINGER, "Import from WebFinger"),
    entry(ids::IDENTITY_ACTIVITYPUB, "Import ActivityPub actor"),
    entry(ids::IDENTITY_NIP05, "Resolve NIP-05"),
    entry(ids::WORKBENCH_SPLIT_BESIDE, "Open beside"),
    entry(ids::WORKBENCH_SPLIT_OUT, "Split out"),
    entry(ids::WORKBENCH_STACK_ONTO, "Stack onto"),
    entry(ids::VIEW_FIT, "Fit to view"),
    entry(ids::VIEW_ZOOM_IN, "Zoom in"),
    entry(ids::VIEW_ZOOM_OUT, "Zoom out"),
    entry(ids::PHYSICS_TOGGLE, "Play or pause physics"),
    entry(ids::PHYSICS_SETTINGS, "Physics settings"),
    entry(ids::SESSION_UNDO, "Undo"),
    entry(ids::SESSION_REDO, "Redo"),
    entry(ids::SESSION_SAVE, "Save session"),
    entry(ids::PANE_SETTINGS, "Open Settings pane"),
    entry(ids::PANE_TRAIL, "Open Trail pane"),
    entry(ids::PANE_WORKBENCH, "Open Workbench pane"),
    entry(ids::PALETTE_OPEN, "Open command palette"),
    entry(ids::IMPORT_BOOKMARKS, "Import bookmarks"),
    entry(ids::NAV_BACK, "Back"),
    entry(ids::NAV_FORWARD, "Forward"),
    entry(ids::NAV_RELOAD, "Reload"),
    entry(ids::NAV_STOP, "Stop loading"),
];

/// The label for a shared id.
pub fn label(id: &str) -> Option<&'static str> {
    CATALOGUE
        .iter()
        .find(|entry| entry.id == id)
        .map(|entry| entry.label)
}

/// What an id acts on: the part before its `:`.
pub fn namespace(id: &str) -> &str {
    id.split_once(':').map_or(id, |(namespace, _)| namespace)
}

/// A shared id as a command to register, its category its namespace.
pub fn command(id: &str) -> Option<Command> {
    let entry = CATALOGUE.iter().find(|entry| entry.id == id)?;
    Some(Command::new(entry.id, entry.label, namespace(entry.id)))
}

/// `true` when `id` is `namespace:name`, both parts lowercase ASCII letters,
/// digits and `_`, with one `:`.
pub fn is_well_formed(id: &str) -> bool {
    let part = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
    };
    match id.split_once(':') {
        Some((namespace, name)) => part(namespace) && part(name),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_id_is_well_formed_unique_and_labelled() {
        assert_eq!(CATALOGUE.len(), 33, "F13's count");
        for (index, entry) in CATALOGUE.iter().enumerate() {
            assert!(is_well_formed(entry.id), "{}", entry.id);
            assert!(!entry.label.is_empty(), "{}", entry.id);
            assert!(
                CATALOGUE[index + 1..]
                    .iter()
                    .all(|other| other.id != entry.id && other.label != entry.label),
                "{} is defined twice",
                entry.id
            );
        }
    }

    #[test]
    fn a_shared_id_registers_under_its_namespace() {
        let fit = command(ids::VIEW_FIT).expect("in the catalogue");
        assert_eq!(fit.label, "Fit to view");
        assert_eq!(fit.category, "view");
        assert!(command("host:own").is_none());
        assert_eq!(label(ids::NAV_STOP), Some("Stop loading"));
    }

    #[test]
    fn malformed_ids_are_refused() {
        for bad in [
            "",
            "node",
            ":new",
            "node:",
            "Node:new",
            "node:new:x",
            "node new",
        ] {
            assert!(!is_well_formed(bad), "{bad:?}");
        }
        assert!(is_well_formed("identity:nip05"));
    }
}
