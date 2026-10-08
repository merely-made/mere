// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use command_menu::catalogue::CATALOGUE;

use crate::actions::*;

#[test]
fn the_variants_are_the_catalogue_one_for_one() {
    assert_eq!(ActionId::ALL.len(), CATALOGUE.len());
    for (action, entry) in ActionId::ALL.into_iter().zip(CATALOGUE) {
        assert_eq!(
            action.key(),
            entry.id,
            "{action:?} out of the catalogue's order"
        );
        assert_eq!(action.label(), entry.label);
        assert_eq!(ActionId::from_key(entry.id), Some(action));
    }
    assert_eq!(ActionId::from_key("frame:select"), None, "retired");
}

#[test]
fn a_variant_names_what_it_acts_on() {
    assert_eq!(ActionId::RelationRetract.namespace(), "relation");
    assert_eq!(ActionId::NavStop.namespace(), "nav");
}

#[test]
fn action_id_serde_json_round_trips() {
    for action in ActionId::ALL {
        let json = serde_json::to_string(&action).expect("serializes");
        let back: ActionId = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(back, action);
    }
}
