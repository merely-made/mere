// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use command_menu::catalogue::ids;

use super::{
    InputBinding, InputBindingSection, InputContext, Keycode, ModifierMask, NamedKey, binding_id,
};

pub(super) fn toolbar_submit_binding() -> InputBinding {
    InputBinding::Key {
        modifiers: ModifierMask::NONE,
        keycode: Keycode::Named(NamedKey::Enter),
    }
}

pub(super) fn graph_view_confirm_binding() -> InputBinding {
    InputBinding::Key {
        modifiers: ModifierMask::NONE,
        keycode: Keycode::Named(NamedKey::Enter),
    }
}

pub(super) fn toolbar_nav_back_binding() -> InputBinding {
    InputBinding::Key {
        modifiers: ModifierMask::ALT,
        keycode: Keycode::Named(NamedKey::ArrowLeft),
    }
}

pub(super) fn toolbar_nav_forward_binding() -> InputBinding {
    InputBinding::Key {
        modifiers: ModifierMask::ALT,
        keycode: Keycode::Named(NamedKey::ArrowRight),
    }
}

pub(super) fn toolbar_nav_reload_binding() -> InputBinding {
    InputBinding::Key {
        modifiers: ModifierMask::NONE,
        keycode: Keycode::Named(NamedKey::F5),
    }
}

pub(super) fn binding_label(binding: &InputBinding, context: InputContext) -> String {
    format!("{}@{}", binding.label(), context.label())
}

#[derive(Clone)]
pub(super) struct DefaultBindingSpec {
    pub(super) action_id: &'static str,
    pub(super) section: InputBindingSection,
    pub(super) context: InputContext,
    pub(super) binding: InputBinding,
}

pub(super) fn default_binding_specs() -> Vec<DefaultBindingSpec> {
    vec![
        DefaultBindingSpec {
            action_id: ids::NODE_EDIT,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::CTRL,
                keycode: Keycode::Char('t'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::PHYSICS_TOGGLE,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('t'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::VIEW_ZOOM_IN,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::CTRL,
                keycode: Keycode::Named(NamedKey::Plus),
            },
        },
        DefaultBindingSpec {
            action_id: ids::VIEW_ZOOM_OUT,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::CTRL,
                keycode: Keycode::Named(NamedKey::Minus),
            },
        },
        DefaultBindingSpec {
            action_id: ids::NODE_NEW,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('n'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::RELATION_ADD,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('g'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::RELATION_ADD_BOTH,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::SHIFT,
                keycode: Keycode::Char('g'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::RELATION_RETRACT,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::ALT,
                keycode: Keycode::Char('g'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::NODE_PIN,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('i'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::NODE_UNPIN,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('u'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::NODE_PIN_TOGGLE,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('l'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::NODE_DELETE,
            section: InputBindingSection::Graph,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Named(NamedKey::Delete),
            },
        },
        DefaultBindingSpec {
            action_id: ids::PALETTE_OPEN,
            section: InputBindingSection::Workbench,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Named(NamedKey::F2),
            },
        },
        DefaultBindingSpec {
            action_id: ids::PHYSICS_SETTINGS,
            section: InputBindingSection::Workbench,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Char('p'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::PANE_TRAIL,
            section: InputBindingSection::Workbench,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::CTRL,
                keycode: Keycode::Char('h'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::SESSION_UNDO,
            section: InputBindingSection::Workbench,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::CTRL,
                keycode: Keycode::Char('z'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::SESSION_REDO,
            section: InputBindingSection::Workbench,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::CTRL,
                keycode: Keycode::Char('y'),
            },
        },
        DefaultBindingSpec {
            action_id: ids::PANE_WORKBENCH,
            section: InputBindingSection::Workbench,
            context: InputContext::GraphView,
            binding: InputBinding::Key {
                modifiers: ModifierMask::NONE,
                keycode: Keycode::Named(NamedKey::F7),
            },
        },
        DefaultBindingSpec {
            action_id: ids::NAV_BACK,
            section: InputBindingSection::Navigation,
            context: InputContext::DetailView,
            binding: toolbar_nav_back_binding(),
        },
        DefaultBindingSpec {
            action_id: ids::NAV_FORWARD,
            section: InputBindingSection::Navigation,
            context: InputContext::DetailView,
            binding: toolbar_nav_forward_binding(),
        },
        DefaultBindingSpec {
            action_id: ids::NAV_RELOAD,
            section: InputBindingSection::Navigation,
            context: InputContext::DetailView,
            binding: toolbar_nav_reload_binding(),
        },
    ]
}

pub(super) fn legacy_binding(binding_id: &str) -> Option<(InputBinding, InputContext)> {
    match binding_id.to_ascii_lowercase().as_str() {
        binding_id::toolbar::SUBMIT => Some((toolbar_submit_binding(), InputContext::OmnibarOpen)),
        binding_id::toolbar::NAV_BACK => {
            Some((toolbar_nav_back_binding(), InputContext::DetailView))
        },
        binding_id::toolbar::NAV_FORWARD => {
            Some((toolbar_nav_forward_binding(), InputContext::DetailView))
        },
        binding_id::toolbar::NAV_RELOAD => {
            Some((toolbar_nav_reload_binding(), InputContext::DetailView))
        },
        _ => None,
    }
}
