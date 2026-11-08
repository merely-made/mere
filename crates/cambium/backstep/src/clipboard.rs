/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The clipboard contract: the host implements it, editors code against it.
//!
//! Text only, synchronous. A desktop host binds its platform clipboard; a host
//! with none passes [`NoClipboard`]. A clipboard that answers later (a browser
//! inside a user gesture, a toolkit that replies with an event) is not this
//! trait's shape.

use core::fmt;

/// Why a clipboard operation did not happen.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ClipboardError {
    /// There is no clipboard, or it is busy or unreachable just now.
    Unavailable,
    /// The host or the platform declined (policy, permission, no gesture).
    Refused,
    /// The clipboard holds nothing readable as text.
    Empty,
}

impl fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unavailable => "the clipboard is unavailable",
            Self::Refused => "the clipboard refused the request",
            Self::Empty => "the clipboard holds no text",
        })
    }
}

impl std::error::Error for ClipboardError {}

/// A host's text clipboard.
pub trait Clipboard {
    /// The clipboard's text.
    fn read_text(&mut self) -> Result<String, ClipboardError>;

    /// Replace the clipboard's contents with `text`.
    fn write_text(&mut self, text: &str) -> Result<(), ClipboardError>;
}

/// The default for a host without a clipboard: every request is refused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NoClipboard;

impl Clipboard for NoClipboard {
    fn read_text(&mut self) -> Result<String, ClipboardError> {
        Err(ClipboardError::Refused)
    }

    fn write_text(&mut self, _text: &str) -> Result<(), ClipboardError> {
        Err(ClipboardError::Refused)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_clipboard_refuses_both_ways() {
        let mut clipboard = NoClipboard;
        assert_eq!(clipboard.read_text(), Err(ClipboardError::Refused));
        assert_eq!(clipboard.write_text("x"), Err(ClipboardError::Refused));
    }

    #[test]
    fn the_trait_is_object_safe() {
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
        let mut memory = Memory(None);
        let clipboard: &mut dyn Clipboard = &mut memory;
        assert_eq!(clipboard.read_text(), Err(ClipboardError::Empty));
        clipboard.write_text("copied").unwrap();
        assert_eq!(clipboard.read_text().as_deref(), Ok("copied"));
    }
}
