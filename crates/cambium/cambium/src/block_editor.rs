// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0
//! The buffer a block document edits through, as messages only.
//!
//! A [`BlockEditor`] holds a source string, a caret, an undo journal and a
//! saved baseline, and answers [`TextCommand`]s. It hands out no widget: the
//! pane that shows the text owns its own [`TextInput`](crate::TextInput) and
//! mirrors the editor through [`BlockEditor::source`] and
//! [`BlockEditor::selection`], so the same boundary can later be a wire guest.

use crate::{CaretSelection, TextCommand};

/// What changed when one command was applied to a [`BlockEditor`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditOutcome {
    pub state_changed: bool,
    pub source_changed: bool,
}

/// The text buffer a block document edits through: a source string, a caret,
/// an undo journal and a saved baseline.
pub trait BlockEditor: Send {
    fn source(&self) -> &str;
    fn selection(&self) -> CaretSelection;
    fn apply(&mut self, command: TextCommand) -> EditOutcome;
    /// Whether the source differs from the last accepted (saved) source.
    fn is_dirty(&self) -> bool;
    /// Record the exact source an external authority accepted. Later local
    /// edits stay dirty.
    fn accept_saved_source(&mut self, source: &str);
}

/// Makes the editor for a document: its id and its source. A host passes one
/// to each pane it opens, so panes in one process may edit through different
/// editors; there is no process-wide registration.
pub trait EditorFactory {
    fn open(&self, id: &str, source: &str) -> Box<dyn BlockEditor>;
}

/// Any closure over an id and a source is a factory.
impl<F> EditorFactory for F
where
    F: Fn(&str, &str) -> Box<dyn BlockEditor>,
{
    fn open(&self, id: &str, source: &str) -> Box<dyn BlockEditor> {
        self(id, source)
    }
}
