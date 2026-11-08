/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! The clipboard contract: the host implements it, editors code against it.
//!
//! Text only, in two shapes. A host whose clipboard answers a call (a desktop
//! platform clipboard) implements [`Clipboard`]; a host with none passes
//! [`NoClipboard`]. A host whose clipboard answers later (a browser promise
//! started inside a user gesture, a toolkit that replies with an event, a
//! runtime whose side effects travel through host ports) carries a
//! [`ClipboardRequest`] and answers it with a [`ClipboardEvent`]: `Read` with
//! `Read`, `Write` with `Written`. [`ClipboardRequest::serve`] carries a request
//! through a call-shaped clipboard, so one editor can code against the pair and
//! run on either host.

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

/// A clipboard operation for a host that answers later.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ClipboardRequest {
    /// Read the clipboard's text; answered by [`ClipboardEvent::Read`].
    Read,
    /// Replace the clipboard's contents; answered by [`ClipboardEvent::Written`].
    Write(String),
}

/// A host's answer to an earlier [`ClipboardRequest`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClipboardEvent {
    /// The clipboard was read.
    Read(Result<String, ClipboardError>),
    /// The clipboard was written.
    Written(Result<(), ClipboardError>),
}

impl ClipboardRequest {
    /// The answer that refuses this request with `error`.
    pub fn fail(&self, error: ClipboardError) -> ClipboardEvent {
        match self {
            Self::Read => ClipboardEvent::Read(Err(error)),
            Self::Write(_) => ClipboardEvent::Written(Err(error)),
        }
    }

    /// Carry this request through a call-shaped `clipboard`, answering at once.
    pub fn serve(&self, clipboard: &mut dyn Clipboard) -> ClipboardEvent {
        match self {
            Self::Read => ClipboardEvent::Read(clipboard.read_text()),
            Self::Write(text) => ClipboardEvent::Written(clipboard.write_text(text)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_request_is_answered_by_its_own_event() {
        let write = ClipboardRequest::Write("x".into());
        assert_eq!(
            ClipboardRequest::Read.fail(ClipboardError::Refused),
            ClipboardEvent::Read(Err(ClipboardError::Refused))
        );
        assert_eq!(
            write.fail(ClipboardError::Unavailable),
            ClipboardEvent::Written(Err(ClipboardError::Unavailable))
        );
        assert_eq!(
            write.serve(&mut NoClipboard),
            ClipboardEvent::Written(Err(ClipboardError::Refused))
        );
        assert_eq!(
            ClipboardRequest::Read.serve(&mut NoClipboard),
            ClipboardEvent::Read(Err(ClipboardError::Refused))
        );
    }

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

    #[test]
    fn a_call_shaped_clipboard_serves_the_pair() {
        #[derive(Default)]
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
        let mut memory = Memory::default();
        assert_eq!(
            ClipboardRequest::Read.serve(&mut memory),
            ClipboardEvent::Read(Err(ClipboardError::Empty))
        );
        assert_eq!(
            ClipboardRequest::Write("copied".into()).serve(&mut memory),
            ClipboardEvent::Written(Ok(()))
        );
        assert_eq!(
            ClipboardRequest::Read.serve(&mut memory),
            ClipboardEvent::Read(Ok("copied".into()))
        );
    }
}
