/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Host-agnostic editing helpers: a generic undo/redo [`History`] over any
//! cloneable document snapshot, the text field's [`EditHistory`] built on it,
//! and an auto-pair [`wrap_selection`]. Any Genet host gets undoable editors
//! and bracket-wrapping fields for free — the omnibar, a note editor, a form
//! field, a projection editor. Prose- or grammar-specific editing (list
//! continuation, structural selection over a djot container tree) stays with
//! the host that knows the grammar.

use crate::controls::TextInput;
use crate::controls::TextSnapshot;

/// Undo/redo over whole-document snapshots, never inverse commands.
///
/// The caller records the pre-edit snapshot *before* each mutation. A record
/// carrying the same coalescing key as the run in progress joins that run
/// instead of pushing, so a drag or a run of keystrokes is one step. With a
/// window set, the run also ends once `window_ms` passes between records, on
/// a clock the host supplies; Cambium keeps none. Any push clears redo.
///
/// The host marks the position it saved with [`mark_saved`](Self::mark_saved);
/// [`is_dirty`](Self::is_dirty) compares against it, so undoing back to the
/// save reads clean. A saved entry that falls off the depth cap can never be
/// reached again, and the history reads dirty until the next save.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History<S, K = ()> {
    undo: Vec<S>,
    redo: Vec<S>,
    /// The coalescing run in progress: its key and the time of its last record.
    run: Option<(K, u64)>,
    /// Depth cap; the oldest entry is dropped past it. `0` means unbounded.
    cap: usize,
    /// Milliseconds between records after which a run ends. `None` never ends
    /// a run by time.
    window_ms: Option<u64>,
    /// The undo depth at the last save, or `None` once that state is unreachable.
    saved: Option<usize>,
}

impl<S, K> Default for History<S, K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S, K> History<S, K> {
    /// A fresh, clean history with a depth cap of 200 and no time window.
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            run: None,
            cap: 200,
            window_ms: None,
            saved: Some(0),
        }
    }

    /// Set the depth `cap` (`0` = unbounded).
    pub fn with_cap(mut self, cap: usize) -> Self {
        self.cap = cap;
        self
    }

    /// End a coalescing run once `window_ms` passes between its records.
    pub fn with_window(mut self, window_ms: u64) -> Self {
        self.window_ms = Some(window_ms);
        self
    }

    /// Record the pre-edit `snapshot`, to call *before* a mutation. `key`
    /// names the edit for coalescing (`None` is always its own step); `now_ms`
    /// is the host's clock, read only when a window is set.
    pub fn record(&mut self, snapshot: S, key: Option<K>, now_ms: u64)
    where
        K: PartialEq,
    {
        if let (Some(key), Some((run_key, last))) = (&key, &mut self.run)
            && key == run_key
            && self
                .window_ms
                .is_none_or(|window| now_ms.saturating_sub(*last) <= window)
        {
            *last = now_ms;
            return;
        }
        if self.saved.is_some_and(|saved| saved > self.undo.len()) {
            // The saved state was on the redo stack, which this push discards.
            self.saved = None;
        }
        self.undo.push(snapshot);
        self.redo.clear();
        if self.cap != 0 && self.undo.len() > self.cap {
            self.undo.remove(0);
            self.saved = match self.saved {
                Some(0) | None => None,
                Some(saved) => Some(saved - 1),
            };
        }
        self.run = key.map(|key| (key, now_ms));
    }

    /// End the run in progress without recording, so the next edit is its own
    /// step (a caret move, a selection change, the end of a drag).
    pub fn break_run(&mut self) {
        self.run = None;
    }

    /// Step back: return the previous snapshot to restore, keeping `current`
    /// for redo. `None` when there is nothing to undo.
    pub fn undo(&mut self, current: S) -> Option<S> {
        let previous = self.undo.pop()?;
        self.redo.push(current);
        self.run = None;
        Some(previous)
    }

    /// Step forward: return the next snapshot to restore, keeping `current`
    /// for undo. `None` when there is nothing to redo.
    pub fn redo(&mut self, current: S) -> Option<S> {
        let next = self.redo.pop()?;
        self.undo.push(current);
        self.run = None;
        Some(next)
    }

    /// Drop all history and read clean (a fresh document never undoes into a
    /// prior one).
    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
        self.run = None;
        self.saved = Some(0);
    }

    /// Mark the current position as saved.
    pub fn mark_saved(&mut self) {
        self.saved = Some(self.undo.len());
    }

    /// Whether the document differs from its last save.
    pub fn is_dirty(&self) -> bool {
        self.saved != Some(self.undo.len())
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
}

/// Undo/redo for a [`TextInput`]: a [`History`] of committed text, caret, and
/// selection snapshots. Composition and completion text are transient and are
/// cleared when a snapshot is restored. A run of consecutive character inserts
/// coalesces into one entry, while a delete, newline, or caret move starts a
/// fresh group.
///
/// This companion is for hosts that need transaction-level grouping and do not
/// use [`TextInput::apply`](crate::TextInput::apply)'s built-in journal:
///
/// ```ignore
/// // before a mutating edit:
/// history.snapshot(&field, /* coalesce_insert = */ true);
/// field.insert_str("x");
/// // on Ctrl+Z / Ctrl+Y:
/// history.undo(&mut field);
/// history.redo(&mut field);
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EditHistory(History<TextSnapshot>);

impl EditHistory {
    /// A fresh history with the default depth cap (200 — a text field is short and
    /// each entry is a whole-buffer clone, so a bounded stack keeps memory flat).
    pub fn new() -> Self {
        Self(History::new())
    }

    /// A history with an explicit depth `cap` (`0` = unbounded).
    pub fn with_cap(cap: usize) -> Self {
        Self(History::new().with_cap(cap))
    }

    /// Record `input`'s pre-edit state, to call *before* a mutating edit. `coalesce_insert`
    /// is true for a character/space insert: a run of them coalesces into one entry (the
    /// push is skipped while already coalescing). A non-insert edit (delete, newline) passes
    /// `false`, so it is its own step and ends the run. Any push clears the redo stack.
    pub fn snapshot(&mut self, input: &TextInput, coalesce_insert: bool) {
        self.record_snapshot(input.snapshot(), coalesce_insert);
    }

    pub(crate) fn record_snapshot(&mut self, snapshot: TextSnapshot, coalesce_insert: bool) {
        self.0.record(snapshot, coalesce_insert.then_some(()), 0);
    }

    /// End the current insert-coalescing run without snapshotting — for a caret move, so
    /// the next insert starts a fresh group even though nothing was deleted.
    pub fn break_coalesce(&mut self) {
        self.0.break_run();
    }

    /// Undo the last edit: restore the top undo snapshot into `input`, moving the current
    /// buffer onto the redo stack. Returns whether anything was undone.
    pub fn undo(&mut self, input: &mut TextInput) -> bool {
        match self.undo_snapshot(input.snapshot()) {
            Some(previous) => {
                input.restore(previous);
                true
            },
            None => false,
        }
    }

    pub(crate) fn undo_snapshot(&mut self, current: TextSnapshot) -> Option<TextSnapshot> {
        self.0.undo(current)
    }

    /// Redo the last undone edit: restore the top redo snapshot into `input`, moving the
    /// current buffer back onto the undo stack. Returns whether anything was redone.
    pub fn redo(&mut self, input: &mut TextInput) -> bool {
        match self.redo_snapshot(input.snapshot()) {
            Some(next) => {
                input.restore(next);
                true
            },
            None => false,
        }
    }

    pub(crate) fn redo_snapshot(&mut self, current: TextSnapshot) -> Option<TextSnapshot> {
        self.0.redo(current)
    }

    /// Drop all history (on a field reset, so a fresh document never undoes into a prior one).
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        self.0.can_undo()
    }

    /// Whether there is anything to redo.
    pub fn can_redo(&self) -> bool {
        self.0.can_redo()
    }
}

/// The closing delimiter that pairs with `open` for an auto-pair wrap, or `None` if `open`
/// is not a wrapping delimiter. Covers brackets, quotes, and the common markup emphasis /
/// code delimiters.
pub fn pair_close(open: char) -> Option<char> {
    Some(match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        '"' => '"',
        '\'' => '\'',
        '`' => '`',
        '*' => '*',
        '_' => '_',
        '~' => '~',
        _ => return None,
    })
}

/// Auto-pair: wrap the current selection with the delimiter pair `open`…`close` and keep the
/// inner text selected, so wraps nest (typing `*` then `_` over a word gives `*_word_*`).
/// Returns whether it wrapped — `false` (no selection, or `open` is not a pair delimiter)
/// means the caller should insert `open` normally. Snapshot for undo before calling.
pub fn wrap_selection(input: &mut TextInput, open: char) -> bool {
    let Some(close) = pair_close(open) else {
        return false;
    };
    if !input.has_selection() {
        return false;
    }
    let (lo, hi) = input.selection_bytes();
    let inner = input.selected_text().to_owned();
    input.insert_str(&format!("{open}{inner}{close}"));
    // Re-select the inner text (between the new delimiters), so a repeat wrap nests.
    let start = lo + open.len_utf8();
    let end = start + (hi - lo);
    input.set_caret_byte(start, false);
    input.set_caret_byte(end, true);
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(text: &str, caret_byte: usize) -> TextInput {
        let mut t = TextInput::new(text);
        t.set_caret_byte(caret_byte, false);
        t
    }

    #[test]
    fn undo_redo_round_trips() {
        let mut input = TextInput::new("");
        let mut h = EditHistory::new();
        h.snapshot(&input, false);
        input.insert_str("hello");
        assert!(h.can_undo());
        assert!(h.undo(&mut input));
        assert_eq!(input.text(), "");
        assert!(h.redo(&mut input));
        assert_eq!(input.text(), "hello");
    }

    #[test]
    fn inserts_coalesce_into_one_undo() {
        let mut input = TextInput::new("");
        let mut h = EditHistory::new();
        for ch in ["h", "i"] {
            h.snapshot(&input, true); // coalescing run
            input.insert_str(ch);
        }
        // One undo removes the whole run.
        assert!(h.undo(&mut input));
        assert_eq!(input.text(), "");
        assert!(!h.can_undo());
    }

    #[test]
    fn a_new_edit_clears_redo() {
        let mut input = TextInput::new("a");
        let mut h = EditHistory::new();
        h.snapshot(&input, false);
        input.insert_str("b");
        h.undo(&mut input); // back to "a", redo has "ab"
        assert!(h.can_redo());
        h.snapshot(&input, false);
        input.insert_str("c"); // a fresh edit
        assert!(!h.can_redo(), "a new edit drops the redo stack");
    }

    /// A document history keyed by the field each edit touched, with a 400 ms window.
    fn doc() -> History<u32, &'static str> {
        History::new().with_window(400)
    }

    #[test]
    fn same_key_within_the_window_is_one_step() {
        let mut h = doc();
        h.record(0, Some("x"), 1_000);
        h.record(1, Some("x"), 1_300);
        h.record(2, Some("x"), 1_600); // each record extends the run
        assert_eq!(h.undo(3), Some(0));
        assert!(!h.can_undo());
    }

    #[test]
    fn a_new_key_starts_a_step() {
        let mut h = doc();
        h.record(0, Some("x"), 1_000);
        h.record(1, Some("y"), 1_100);
        assert_eq!(h.undo(2), Some(1));
        assert_eq!(h.undo(1), Some(0));
    }

    #[test]
    fn a_lapsed_window_starts_a_step() {
        let mut h = doc();
        h.record(0, Some("x"), 1_000);
        h.record(1, Some("x"), 1_401);
        assert_eq!(h.undo(2), Some(1));
        assert_eq!(h.undo(1), Some(0));
    }

    #[test]
    fn no_key_and_break_run_never_coalesce() {
        let mut h = doc();
        h.record(0, None, 1_000);
        h.record(1, None, 1_000);
        h.record(2, Some("x"), 1_000);
        h.break_run();
        h.record(3, Some("x"), 1_000);
        assert_eq!(h.undo(4), Some(3));
        assert_eq!(h.undo(3), Some(2));
        assert_eq!(h.undo(2), Some(1));
        assert_eq!(h.undo(1), Some(0));
    }

    #[test]
    fn undoing_back_to_the_save_reads_clean() {
        let mut h = doc();
        assert!(!h.is_dirty(), "a fresh document is clean");
        h.record(0, None, 0);
        h.mark_saved(); // saved at 1
        h.record(1, None, 0);
        assert!(h.is_dirty());
        assert_eq!(h.undo(2), Some(1));
        assert!(!h.is_dirty(), "back at the save");
        assert_eq!(h.undo(1), Some(0));
        assert!(h.is_dirty(), "before the save");
        assert_eq!(h.redo(0), Some(1));
        assert!(!h.is_dirty());
    }

    #[test]
    fn a_new_edit_after_undoing_past_the_save_stays_dirty() {
        let mut h = doc();
        h.record(0, None, 0);
        h.record(1, None, 0);
        h.mark_saved(); // saved at 2
        h.undo(2);
        h.undo(1);
        h.record(0, None, 0); // the saved state was on redo, now discarded
        h.record(5, None, 0); // back at the saved depth, but a different document
        assert!(h.is_dirty(), "the saved state is unreachable");
        h.undo(6);
        assert!(h.is_dirty());
    }

    #[test]
    fn a_saved_entry_past_the_cap_stays_dirty() {
        let mut h: History<u32> = History::new().with_cap(2);
        h.mark_saved(); // saved at the empty base
        h.record(0, None, 0);
        h.record(1, None, 0);
        h.record(2, None, 0); // drops the base state
        while h.undo(9).is_some() {}
        assert!(h.is_dirty(), "the base fell off the cap");
        h.mark_saved();
        assert!(!h.is_dirty());
    }

    #[test]
    fn saves_shift_with_the_cap() {
        let mut h: History<u32> = History::new().with_cap(2);
        h.record(0, None, 0);
        h.mark_saved(); // saved at 1
        h.record(1, None, 0);
        h.record(2, None, 0); // drops one entry; the save is now at 0
        assert_eq!(h.undo(3), Some(2));
        assert_eq!(h.undo(2), Some(1));
        assert!(!h.is_dirty(), "the saved state is still reachable");
    }

    #[test]
    fn wrap_selection_wraps_and_nests() {
        let mut input = at("hello world", 0);
        input.set_caret_byte(0, false);
        input.set_caret_byte(5, true); // select "hello"
        assert!(wrap_selection(&mut input, '*'));
        assert_eq!(input.text(), "*hello* world");
        assert!(wrap_selection(&mut input, '_')); // inner still selected → nests
        assert_eq!(input.text(), "*_hello_* world");
    }

    #[test]
    fn wrap_without_selection_or_pair_is_declined() {
        let mut input = at("hello", 5);
        assert!(!wrap_selection(&mut input, '*'), "no selection");
        input.set_caret_byte(0, false);
        input.set_caret_byte(5, true);
        assert!(!wrap_selection(&mut input, 'x'), "not a pair delimiter");
        assert_eq!(input.text(), "hello");
    }
}
