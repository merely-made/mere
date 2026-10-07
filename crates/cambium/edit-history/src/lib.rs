/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Undo and redo over whole-document snapshots, never inverse commands.
//!
//! An editor keeps a [`History`] of the snapshots it records before each edit.
//! Records that share a key within a window, on a clock the host supplies,
//! coalesce into one step; a saved marker tells the editor whether the
//! document differs from its last save. Cambium's text field and the
//! projection editor both use it. Durable, attributed undo over saved facts
//! is a different layer and lives with the mere's session journal.

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

    /// Forget the saved position: the stored document no longer matches any
    /// state in the history, so it reads dirty until the next save.
    pub fn forget_saved(&mut self) {
        self.saved = None;
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

#[cfg(test)]
mod tests {
    use super::*;

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
}
