// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The effects chrome asks its host to enact, as hest commands
//! (engine-split D29). Ids and the argument encoding are chrome's own and
//! stay outside hest's shared catalogue until a second host offers them.
//! The runtime issues these through [`FrameStep::invocations`]; the host
//! decodes, enacts, and answers in [`FrameHostInput::outcomes`].
//!
//! [`FrameStep::invocations`]: crate::frame_model::FrameStep::invocations
//! [`FrameHostInput::outcomes`]: crate::frame_model::FrameHostInput::outcomes

use hest::{EffectClass, Invocation, Outcome, menu::Command};

/// The argument schema for every command here. Arguments are the text's
/// UTF-8 bytes; this id is their version header.
pub const ARGUMENT_SCHEMA: &str = "chrome.effects/v1";

pub mod ids {
    pub const CLIPBOARD_WRITE_TEXT: &str = "clipboard:write_text";
}

/// The commands chrome declares, for a host that lists or admits them.
pub fn commands() -> Vec<Command> {
    vec![Command::new(
        ids::CLIPBOARD_WRITE_TEXT,
        "Copy text",
        "clipboard",
        EffectClass::ExternalEffect,
    )]
}

/// Ask the host to put `text` on the system clipboard.
pub fn clipboard_write_text(text: &str) -> Invocation {
    Invocation {
        id: ids::CLIPBOARD_WRITE_TEXT.into(),
        schema_id: ARGUMENT_SCHEMA.into(),
        arguments: text.as_bytes().to_vec(),
        effect: EffectClass::ExternalEffect,
    }
}

/// The host's side: the text a clipboard write carries, or the refusal to
/// answer with when the invocation does not match this declaration.
pub fn read_clipboard_write_text(invocation: &Invocation) -> Result<String, Outcome> {
    let refused = |reason: &str| Outcome::Refused {
        reason: reason.into(),
    };
    if invocation.id != ids::CLIPBOARD_WRITE_TEXT
        || invocation.schema_id != ARGUMENT_SCHEMA
        || invocation.effect != EffectClass::ExternalEffect
    {
        return Err(refused("not a chrome clipboard write"));
    }
    String::from_utf8(invocation.arguments.clone()).map_err(|_| refused("text is not UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clipboard_write_round_trips_and_refuses_a_foreign_schema() {
        let invocation = clipboard_write_text("https://example.org/ü");
        assert_eq!(commands()[0].effect, invocation.effect);
        assert_eq!(
            read_clipboard_write_text(&invocation).as_deref(),
            Ok("https://example.org/ü")
        );
        let foreign = Invocation {
            schema_id: "chrome.effects/v2".into(),
            ..invocation
        };
        assert!(matches!(
            read_clipboard_write_text(&foreign),
            Err(Outcome::Refused { .. })
        ));
    }
}
