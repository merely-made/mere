/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Linear back/forward navigation: a browser toolbar's history, moved here
//! from chrome. Not undo (that is [`crate::History`]) and not a shared browse
//! trail (the graph kernel keeps that).

/// A linear back/forward history of visited locations (resolved URLs).
///
/// Always holds at least one entry, so [`current`](Self::current) never
/// panics. A fresh [`visit`](Self::visit) truncates any forward entries (the
/// classic browser "new branch discards the forward stack" behavior); a repeat
/// of the current entry is treated as a reload and does not grow the stack.
#[derive(Debug, Clone)]
pub struct History {
    entries: Vec<String>,
    cursor: usize,
}

impl History {
    /// A history seeded with one entry (the cold-start location).
    pub fn new(initial: impl Into<String>) -> Self {
        Self {
            entries: vec![initial.into()],
            cursor: 0,
        }
    }

    /// The current entry — the URL the content root should show.
    pub fn current(&self) -> &str {
        &self.entries[self.cursor]
    }

    /// All visited URLs, oldest first. Used to source omnibar suggestions.
    pub fn entries(&self) -> &[String] {
        &self.entries
    }

    /// Whether a back step is possible.
    pub fn can_back(&self) -> bool {
        self.cursor > 0
    }

    /// Whether a forward step is possible.
    pub fn can_forward(&self) -> bool {
        self.cursor + 1 < self.entries.len()
    }

    /// Navigate to `url`. A repeat of the current entry is a reload (no
    /// growth); otherwise forward entries are discarded and `url` pushed as the
    /// new current.
    pub fn visit(&mut self, url: String) {
        if self.entries[self.cursor] == url {
            return;
        }
        self.entries.truncate(self.cursor + 1);
        self.entries.push(url);
        self.cursor = self.entries.len() - 1;
    }

    /// Step back, returning the new current entry (or `None` at the oldest).
    pub fn back(&mut self) -> Option<&str> {
        if self.cursor == 0 {
            return None;
        }
        self.cursor -= 1;
        Some(&self.entries[self.cursor])
    }

    /// Step forward, returning the new current entry (or `None` at the newest).
    pub fn forward(&mut self) -> Option<&str> {
        if self.cursor + 1 >= self.entries.len() {
            return None;
        }
        self.cursor += 1;
        Some(&self.entries[self.cursor])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_starts_with_one_entry_and_no_steps() {
        let h = History::new("mere://welcome");
        assert_eq!(h.current(), "mere://welcome");
        assert!(!h.can_back());
        assert!(!h.can_forward());
    }

    #[test]
    fn history_visit_advances_and_enables_back() {
        let mut h = History::new("a");
        h.visit("b".into());
        assert_eq!(h.current(), "b");
        assert!(h.can_back());
        assert!(!h.can_forward());
    }

    #[test]
    fn history_back_forward_round_trip() {
        let mut h = History::new("a");
        h.visit("b".into());
        assert_eq!(h.back(), Some("a"));
        assert!(!h.can_back());
        assert!(h.can_forward());
        assert_eq!(h.forward(), Some("b"));
        assert!(h.can_back());
        assert!(!h.can_forward());
        // Stepping past either end is a no-op.
        assert_eq!(h.forward(), None);
        h.back();
        assert_eq!(h.back(), None);
    }

    #[test]
    fn history_visit_truncates_forward_branch() {
        let mut h = History::new("a");
        h.visit("b".into());
        h.visit("c".into());
        h.back(); // at "b", forward = ["c"]
        h.visit("d".into()); // discards "c"
        assert_eq!(h.current(), "d");
        assert!(h.can_back());
        assert!(!h.can_forward());
        h.back();
        assert_eq!(h.current(), "b");
    }

    #[test]
    fn history_reload_does_not_grow_stack() {
        let mut h = History::new("a");
        h.visit("a".into());
        assert!(!h.can_back());
        assert_eq!(h.current(), "a");
    }
}
