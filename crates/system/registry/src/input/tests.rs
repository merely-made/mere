// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::action_id;
use super::binding_id;
use super::defaults::*;
use super::*;
use command_menu::catalogue::{self, ids};
use std::collections::HashMap;

#[test]
fn input_registry_resolves_toolbar_submit_binding() {
    let registry = InputRegistry::default();
    let resolution = registry.resolve(&toolbar_submit_binding(), InputContext::OmnibarOpen);

    assert!(resolution.matched);
    assert_eq!(
        resolution.action_id.as_deref(),
        Some(action_id::toolbar::SUBMIT)
    );
}

#[test]
fn input_registry_reports_missing_binding() {
    let registry = InputRegistry::default();
    let resolution = registry.resolve_binding_id("input.unknown.binding");

    assert!(!resolution.matched);
    assert!(!resolution.conflicted);
    assert_eq!(resolution.action_id, None);
}

#[test]
fn input_registry_resolves_toolbar_nav_bindings() {
    let registry = InputRegistry::default();

    let back = registry.resolve(&toolbar_nav_back_binding(), InputContext::DetailView);
    assert!(back.matched);
    assert_eq!(back.action_id.as_deref(), Some(ids::NAV_BACK));

    let forward = registry.resolve(&toolbar_nav_forward_binding(), InputContext::DetailView);
    assert!(forward.matched);
    assert_eq!(forward.action_id.as_deref(), Some(ids::NAV_FORWARD));

    let reload = registry.resolve(&toolbar_nav_reload_binding(), InputContext::DetailView);
    assert!(reload.matched);
    assert_eq!(reload.action_id.as_deref(), Some(ids::NAV_RELOAD));
}

#[test]
fn input_registry_resolves_enter_differently_by_context() {
    let registry = InputRegistry::default();

    let omnibar = registry.resolve(&toolbar_submit_binding(), InputContext::OmnibarOpen);
    assert_eq!(
        omnibar.action_id.as_deref(),
        Some(action_id::toolbar::SUBMIT)
    );

    let graph_view = registry.resolve(&graph_view_confirm_binding(), InputContext::GraphView);
    assert_eq!(
        graph_view.action_id.as_deref(),
        Some(action_id::graph::VIEW_CONFIRM)
    );
}

#[test]
fn input_registry_detects_same_binding_conflict_within_context() {
    let mut registry = InputRegistry {
        bindings: HashMap::new(),
    };

    registry.register_binding(
        toolbar_submit_binding(),
        action_id::toolbar::SUBMIT,
        InputContext::OmnibarOpen,
    );
    registry.register_binding(
        toolbar_submit_binding(),
        action_id::graph::VIEW_CONFIRM,
        InputContext::OmnibarOpen,
    );

    let resolution = registry.resolve(&toolbar_submit_binding(), InputContext::OmnibarOpen);
    assert!(!resolution.matched);
    assert!(resolution.conflicted);
    assert_eq!(resolution.action_id, None);
}

#[test]
fn input_registry_legacy_binding_ids_resolve_through_typed_map() {
    let registry = InputRegistry::default();

    let resolution = registry.resolve_binding_id(binding_id::toolbar::NAV_RELOAD);
    assert!(resolution.matched);
    assert_eq!(resolution.context, InputContext::DetailView);
    assert_eq!(resolution.action_id.as_deref(), Some(ids::NAV_RELOAD));
}

#[test]
fn input_binding_remap_round_trips_through_string_encoding() {
    let remap = InputBindingRemap {
        old: toolbar_nav_back_binding(),
        new: InputBinding::Key {
            modifiers: ModifierMask::ALT,
            keycode: Keycode::Char('b'),
        },
        context: InputContext::GraphView,
    };

    let decoded = InputBindingRemap::decode(&remap.encode()).expect("remap should decode");
    assert_eq!(decoded, remap);
}

#[test]
fn input_registry_remap_binding_replaces_existing_binding() {
    let mut registry = InputRegistry::default();
    let old = toolbar_nav_back_binding();
    let new = InputBinding::Key {
        modifiers: ModifierMask::ALT,
        keycode: Keycode::Char('b'),
    };

    registry
        .remap_binding(old.clone(), new.clone(), InputContext::DetailView)
        .expect("remap should succeed");

    assert_eq!(
        registry.resolve(&old, InputContext::DetailView).action_id,
        None
    );
    assert_eq!(
        registry
            .resolve(&new, InputContext::DetailView)
            .action_id
            .as_deref(),
        Some(ids::NAV_BACK)
    );
}

#[test]
fn input_registry_remap_binding_detects_target_conflicts() {
    let mut registry = InputRegistry::default();
    let result = registry.remap_binding(
        toolbar_nav_back_binding(),
        toolbar_nav_forward_binding(),
        InputContext::DetailView,
    );

    assert!(matches!(result, Err(InputConflict::TargetConflict { .. })));
    assert_eq!(
        registry
            .resolve(&toolbar_nav_back_binding(), InputContext::DetailView)
            .action_id
            .as_deref(),
        Some(ids::NAV_BACK)
    );
}

#[test]
fn input_registry_with_remaps_replays_on_top_of_defaults() {
    let remaps = [InputBindingRemap {
        old: toolbar_nav_back_binding(),
        new: InputBinding::Key {
            modifiers: ModifierMask::ALT,
            keycode: Keycode::Char('b'),
        },
        context: InputContext::DetailView,
    }];
    let registry = InputRegistry::with_remaps(&remaps).expect("remaps should apply");

    assert_eq!(
        registry
            .resolve(&remaps[0].new, InputContext::DetailView)
            .action_id
            .as_deref(),
        Some(ids::NAV_BACK)
    );
}

#[test]
fn input_binding_display_label_uses_human_shortcut_format() {
    let binding = InputBinding::Key {
        modifiers: ModifierMask(ModifierMask::CTRL.0 | ModifierMask::SHIFT.0),
        keycode: Keycode::Char('g'),
    };

    assert_eq!(binding.display_label(), "Ctrl+Shift+G");
}

#[test]
fn input_registry_describes_bindable_actions_with_current_and_default_bindings() {
    let registry = InputRegistry::default();
    let descriptors = registry.describe_bindable_actions();
    let command_palette = descriptors
        .iter()
        .find(|entry| entry.action_id == ids::PALETTE_OPEN)
        .expect("command palette binding descriptor should exist");

    assert_eq!(command_palette.display_name, "Open command palette");
    assert_eq!(command_palette.context, InputContext::GraphView);
    assert_eq!(
        command_palette
            .current_binding
            .as_ref()
            .map(InputBinding::display_label)
            .as_deref(),
        Some("F2")
    );
    assert_eq!(
        command_palette
            .default_binding
            .as_ref()
            .map(InputBinding::display_label)
            .as_deref(),
        Some("F2")
    );
}

#[test]
fn input_registry_uses_ctrl_modified_default_zoom_bindings() {
    let registry = InputRegistry::default();
    let descriptors = registry.describe_bindable_actions();
    let zoom_in = descriptors
        .iter()
        .find(|entry| entry.action_id == ids::VIEW_ZOOM_IN)
        .expect("zoom-in binding descriptor should exist");
    let zoom_out = descriptors
        .iter()
        .find(|entry| entry.action_id == ids::VIEW_ZOOM_OUT)
        .expect("zoom-out binding descriptor should exist");

    assert_eq!(
        zoom_in
            .default_binding
            .as_ref()
            .map(InputBinding::display_label)
            .as_deref(),
        Some("Ctrl++")
    );
    assert_eq!(
        zoom_out
            .default_binding
            .as_ref()
            .map(InputBinding::display_label)
            .as_deref(),
        Some("Ctrl+-")
    );
}

#[test]
fn every_default_binding_names_a_catalogue_command() {
    let specs = default_binding_specs();
    assert_eq!(
        specs.len(),
        21,
        "SE50: the bindings that land on shared ids"
    );
    for spec in &specs {
        assert!(
            catalogue::label(spec.action_id).is_some(),
            "{} is not a shared command",
            spec.action_id
        );
    }
    for input_action in [action_id::toolbar::SUBMIT, action_id::graph::VIEW_CONFIRM] {
        assert!(catalogue::is_well_formed(input_action), "{input_action}");
    }
}
