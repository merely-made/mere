// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The portable window adapter owns its ids and argument encoding. These
//! remain outside Hest's shared catalogue until its second-host offer rule is met.

use super::WindowCommand;
use cambium::{EffectClass, Invocation, Outcome};

impl WindowCommand {
    /// No arguments except `window:resize`, whose arguments are two f64 values
    /// (width, height) encoded little endian. The event source retains its
    /// existing geometry validation and window-manager policy.
    pub const ARGUMENT_SCHEMA: &str = "cambium.window/v1";

    pub fn id(&self) -> &'static str {
        match self {
            Self::Show => "window:show",
            Self::Minimize => "window:minimize",
            Self::ToggleMaximize => "window:toggle_maximize",
            Self::Drag => "window:drag",
            Self::ShowSystemMenu => "window:system_menu",
            Self::Resize(..) => "window:resize",
            Self::Close => "window:close",
        }
    }

    fn effect(&self) -> EffectClass {
        match self {
            Self::Close => EffectClass::ExternalEffect,
            _ => EffectClass::Curation,
        }
    }

    pub fn invocation(&self) -> Invocation {
        let arguments = match self {
            Self::Resize(width, height) => [width.to_le_bytes(), height.to_le_bytes()].concat(),
            _ => Vec::new(),
        };
        Invocation {
            id: self.id().into(),
            schema_id: Self::ARGUMENT_SCHEMA.into(),
            arguments,
            effect: self.effect(),
        }
    }

    pub fn from_invocation(invocation: &Invocation) -> Result<Self, Outcome> {
        let refused = || Outcome::Refused {
            reason: "invalid window command contract or arguments".into(),
        };
        if invocation.schema_id != Self::ARGUMENT_SCHEMA {
            return Err(refused());
        }
        let command = match invocation.id.as_str() {
            "window:show" => Self::Show,
            "window:minimize" => Self::Minimize,
            "window:toggle_maximize" => Self::ToggleMaximize,
            "window:drag" => Self::Drag,
            "window:system_menu" => Self::ShowSystemMenu,
            "window:close" => Self::Close,
            "window:resize" if invocation.arguments.len() == 16 => Self::Resize(
                f64::from_le_bytes(invocation.arguments[..8].try_into().unwrap()),
                f64::from_le_bytes(invocation.arguments[8..].try_into().unwrap()),
            ),
            _ => return Err(refused()),
        };
        if command.effect() != invocation.effect
            || (!matches!(command, Self::Resize(..)) && !invocation.arguments.is_empty())
        {
            return Err(refused());
        }
        Ok(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::WindowCommands;

    #[test]
    fn every_window_verb_round_trips_and_shares_the_existing_queue() {
        let commands = WindowCommands::new();
        let verbs = [
            WindowCommand::Show,
            WindowCommand::Minimize,
            WindowCommand::ToggleMaximize,
            WindowCommand::Drag,
            WindowCommand::ShowSystemMenu,
            WindowCommand::Resize(800.5, 600.25),
            WindowCommand::Close,
        ];
        let clone = commands.clone();
        for verb in verbs {
            let invocation = verb.invocation();
            assert_eq!(WindowCommand::from_invocation(&invocation), Ok(verb));
            assert_eq!(clone.invoke(&invocation), Ok(()));
        }
        assert_eq!(commands.drain(), verbs);
        assert!(clone.is_empty());
    }

    #[test]
    fn invalid_requests_leave_the_queue_empty() {
        let commands = WindowCommands::new();
        for field in 0..4 {
            let mut invocation = WindowCommand::Minimize.invocation();
            match field {
                0 => invocation.id.push_str("-other"),
                1 => invocation.schema_id.push_str("-other"),
                2 => invocation.effect = EffectClass::DomainTruth,
                _ => invocation.arguments.push(1),
            }
            assert!(matches!(
                commands.invoke(&invocation),
                Err(Outcome::Refused { .. })
            ));
            assert!(commands.is_empty());
        }
        let mut resize = WindowCommand::Resize(1.0, 2.0).invocation();
        resize.arguments.pop();
        assert!(commands.invoke(&resize).is_err());
        assert!(commands.is_empty());
    }
}
