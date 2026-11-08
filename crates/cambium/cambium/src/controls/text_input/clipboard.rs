/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Copy, cut and paste through the host's [`Clipboard`] (backstep's contract).

use backstep::{Clipboard, ClipboardError};

use crate::{Key, KeyEvent};

use super::TextInput;

/// A clipboard intent, applied by [`TextInput::apply_clipboard`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClipboardCommand {
    Copy,
    Cut,
    Paste,
}

/// Ctrl/Cmd+C/X/V as a [`ClipboardCommand`]; `None` for any other event.
pub fn clipboard_command_from_key(event: &KeyEvent) -> Option<ClipboardCommand> {
    let primary = event.mods.ctrl || event.mods.meta;
    let Key::Character(value) = &event.key else {
        return None;
    };
    if !primary || event.mods.alt || event.mods.shift {
        return None;
    }
    match value.to_ascii_lowercase().as_str() {
        "c" => Some(ClipboardCommand::Copy),
        "x" => Some(ClipboardCommand::Cut),
        "v" => Some(ClipboardCommand::Paste),
        _ => None,
    }
}

impl TextInput {
    /// Apply a clipboard command through the host's `clipboard`. Returns
    /// whether the buffer changed. Copy and cut with nothing selected do not
    /// touch the clipboard; a cut whose write fails keeps the selection; a
    /// paste is its own undo step.
    pub fn apply_clipboard(
        &mut self,
        command: ClipboardCommand,
        clipboard: &mut dyn Clipboard,
    ) -> Result<bool, ClipboardError> {
        match command {
            ClipboardCommand::Copy | ClipboardCommand::Cut if !self.has_selection() => Ok(false),
            ClipboardCommand::Copy => clipboard.write_text(self.selected_text()).map(|()| false),
            ClipboardCommand::Cut => {
                clipboard.write_text(self.selected_text())?;
                Ok(self.apply_committed_edit(false, |input| input.backspace()))
            },
            ClipboardCommand::Paste => {
                let text = clipboard.read_text()?;
                Ok(self.apply_committed_edit(false, move |input| {
                    input.clear_preedit();
                    input.insert_str(&text);
                }))
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use backstep::NoClipboard;

    use super::*;
    use crate::{Modifiers, TextCommand};

    /// An in-memory host clipboard.
    struct Memory(Option<String>);

    impl Clipboard for Memory {
        fn read_text(&mut self) -> Result<String, ClipboardError> {
            self.0.clone().ok_or(ClipboardError::Empty)
        }
        fn write_text(&mut self, text: &str) -> Result<(), ClipboardError> {
            self.0 = Some(text.to_owned());
            Ok(())
        }
    }

    fn primary(key: &str) -> KeyEvent {
        let ctrl = Modifiers {
            ctrl: true,
            ..Modifiers::default()
        };
        KeyEvent::with_mods(Key::Character(key.to_owned()), ctrl)
    }

    #[test]
    fn primary_c_x_v_map_to_clipboard_commands() {
        assert_eq!(
            clipboard_command_from_key(&primary("c")),
            Some(ClipboardCommand::Copy)
        );
        assert_eq!(
            clipboard_command_from_key(&primary("X")),
            Some(ClipboardCommand::Cut)
        );
        assert_eq!(
            clipboard_command_from_key(&primary("v")),
            Some(ClipboardCommand::Paste)
        );
        assert_eq!(clipboard_command_from_key(&primary("a")), None);
        let plain = KeyEvent::with_mods(Key::Character("c".to_owned()), Modifiers::default());
        assert_eq!(clipboard_command_from_key(&plain), None);
    }

    #[test]
    fn copy_cut_paste_round_trip_through_the_host_clipboard() {
        let mut clipboard = Memory(None);
        let mut input = TextInput::new("hello world");
        input.apply(TextCommand::SelectAll);
        assert_eq!(
            input.apply_clipboard(ClipboardCommand::Copy, &mut clipboard),
            Ok(false)
        );
        assert_eq!(clipboard.0.as_deref(), Some("hello world"));
        assert_eq!(
            input.apply_clipboard(ClipboardCommand::Cut, &mut clipboard),
            Ok(true)
        );
        assert_eq!(input.text(), "");
        for _ in 0..2 {
            assert_eq!(
                input.apply_clipboard(ClipboardCommand::Paste, &mut clipboard),
                Ok(true)
            );
        }
        assert_eq!(input.text(), "hello worldhello world");
        // Each paste is its own undo step.
        assert!(input.apply(TextCommand::Undo));
        assert_eq!(input.text(), "hello world");
    }

    #[test]
    fn a_refused_clipboard_leaves_the_buffer_alone() {
        let mut input = TextInput::new("keep");
        input.apply(TextCommand::SelectAll);
        assert_eq!(
            input.apply_clipboard(ClipboardCommand::Cut, &mut NoClipboard),
            Err(ClipboardError::Refused)
        );
        assert_eq!(input.text(), "keep");
        assert!(input.has_selection());
        assert_eq!(
            input.apply_clipboard(ClipboardCommand::Paste, &mut NoClipboard),
            Err(ClipboardError::Refused)
        );
        assert_eq!(input.text(), "keep");
    }

    #[test]
    fn copy_without_a_selection_does_not_touch_the_clipboard() {
        let mut clipboard = Memory(Some("before".to_owned()));
        let mut input = TextInput::new("text");
        assert_eq!(
            input.apply_clipboard(ClipboardCommand::Copy, &mut clipboard),
            Ok(false)
        );
        assert_eq!(clipboard.0.as_deref(), Some("before"));
    }
}
