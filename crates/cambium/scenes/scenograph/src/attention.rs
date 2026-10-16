// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Attention over a view's occurrences (Scenograph editor plan SE91; graph
//! grammar, Merely Design System DECISIONS 2026-10-16).
//!
//! Four states. Forward: selected, or held there by a foreground pin.
//! Retained: was forward, then deselected; it stays until dismissed.
//! Ambient: untouched. Suggestion: offered by the scene rather than the
//! person's content. Attention is not keeping: nothing here writes durable
//! state, moves anything or implies a position pin. A foreground pin is
//! not a position pin, and recency alone never retains.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::presentation::ViewState;

/// Ordered by prominence, so `max` picks the most prominent state.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Attention {
    Suggestion,
    Ambient,
    Retained,
    Forward,
}

/// One view's attention, keyed by whatever identifies an occurrence there.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AttentionBook<K: Ord> {
    selected: BTreeSet<K>,
    foreground_pins: BTreeSet<K>,
    retained: BTreeSet<K>,
    suggested: BTreeSet<K>,
}

impl<K: Ord> Default for AttentionBook<K> {
    fn default() -> Self {
        Self {
            selected: BTreeSet::new(),
            foreground_pins: BTreeSet::new(),
            retained: BTreeSet::new(),
            suggested: BTreeSet::new(),
        }
    }
}

impl<K: Ord + Clone> AttentionBook<K> {
    /// Replace the working selection. Whatever leaves it without a
    /// foreground pin is retained; whatever enters it stops being retained.
    pub fn select(&mut self, selection: impl IntoIterator<Item = K>) {
        let selection: BTreeSet<K> = selection.into_iter().collect();
        for key in self.selected.difference(&selection) {
            if !self.foreground_pins.contains(key) {
                self.retained.insert(key.clone());
            }
        }
        for key in &selection {
            self.retained.remove(key);
        }
        self.selected = selection;
    }

    /// Hold `key` forward while the selection moves, or let it go. A released
    /// pin on an unselected occurrence leaves it retained, as a deselection
    /// would.
    pub fn set_foreground_pin(&mut self, key: K, pinned: bool) {
        if pinned {
            self.retained.remove(&key);
            self.foreground_pins.insert(key);
        } else if self.foreground_pins.remove(&key) && !self.selected.contains(&key) {
            self.retained.insert(key);
        }
    }

    /// Return a retained occurrence to ambient. Forward ones are refused.
    pub fn dismiss(&mut self, key: &K) -> bool {
        self.retained.remove(key)
    }

    /// Dismiss every retained occurrence, returning how many there were.
    pub fn dismiss_all(&mut self) -> usize {
        std::mem::take(&mut self.retained).len()
    }

    /// The scene's current offers. The person's attention outranks them: a
    /// suggestion that is selected or retained reads as such.
    pub fn set_suggested(&mut self, keys: impl IntoIterator<Item = K>) {
        self.suggested = keys.into_iter().collect();
    }

    /// Keep only the occurrences `live` still holds.
    pub fn retain(&mut self, mut live: impl FnMut(&K) -> bool) {
        for set in [
            &mut self.selected,
            &mut self.foreground_pins,
            &mut self.retained,
            &mut self.suggested,
        ] {
            set.retain(&mut live);
        }
    }

    pub fn attention(&self, key: &K) -> Attention {
        self.attention_with(key, self.selected.contains(key))
    }

    /// The attention `key` will have once a host's live selection, which may
    /// have moved since the last [`Self::select`], is folded in.
    pub fn attention_with(&self, key: &K, selected_now: bool) -> Attention {
        if selected_now || self.foreground_pins.contains(key) {
            Attention::Forward
        } else if self.retained.contains(key) || self.selected.contains(key) {
            Attention::Retained
        } else if self.suggested.contains(key) {
            Attention::Suggestion
        } else {
            Attention::Ambient
        }
    }

    /// The presentation-rule states this book supplies for `key`. Position
    /// pins, inspection and overview belong to other owners.
    pub fn view_states(&self, key: &K) -> BTreeSet<ViewState> {
        let mut states = BTreeSet::new();
        if self.selected.contains(key) {
            states.insert(ViewState::Selected);
        }
        if self.foreground_pins.contains(key) {
            states.insert(ViewState::ForegroundPinned);
        }
        match self.attention(key) {
            Attention::Retained => {
                states.insert(ViewState::Retained);
            },
            Attention::Suggestion => {
                states.insert(ViewState::Suggested);
            },
            _ => {},
        }
        states
    }

    pub fn selected(&self) -> &BTreeSet<K> {
        &self.selected
    }

    pub fn foreground_pins(&self) -> &BTreeSet<K> {
        &self.foreground_pins
    }

    pub fn retained(&self) -> &BTreeSet<K> {
        &self.retained
    }

    pub fn suggested(&self) -> &BTreeSet<K> {
        &self.suggested
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn book() -> AttentionBook<&'static str> {
        AttentionBook::default()
    }

    #[test]
    fn deselection_retains_until_dismissed() {
        let mut book = book();
        book.select(["a", "b"]);
        book.select(["c"]);
        assert_eq!(book.attention(&"a"), Attention::Retained);
        assert_eq!(book.attention(&"c"), Attention::Forward);
        assert!(book.dismiss(&"a"));
        assert_eq!(book.attention(&"a"), Attention::Ambient);
        assert!(!book.dismiss(&"c"), "a forward occurrence is not dismissed");
        assert_eq!(book.dismiss_all(), 1);
        assert_eq!(book.attention(&"b"), Attention::Ambient);
    }

    #[test]
    fn reselection_lifts_retained_back_forward() {
        let mut book = book();
        book.select(["a"]);
        book.select([]);
        book.select(["a"]);
        assert_eq!(book.attention(&"a"), Attention::Forward);
        assert!(book.retained().is_empty());
    }

    #[test]
    fn foreground_pin_holds_forward_across_selection() {
        let mut book = book();
        book.select(["a"]);
        book.set_foreground_pin("a", true);
        book.select(["b"]);
        assert_eq!(book.attention(&"a"), Attention::Forward);
        assert!(!book.retained().contains(&"a"));
        book.set_foreground_pin("a", false);
        assert_eq!(book.attention(&"a"), Attention::Retained);
        book.set_foreground_pin("b", true);
        book.set_foreground_pin("b", false);
        assert_eq!(book.attention(&"b"), Attention::Forward, "still selected");
    }

    #[test]
    fn suggestions_yield_to_the_persons_attention() {
        let mut book = book();
        book.set_suggested(["s", "t"]);
        assert_eq!(book.attention(&"s"), Attention::Suggestion);
        book.select(["s"]);
        book.select([]);
        assert_eq!(book.attention(&"s"), Attention::Retained);
        assert_eq!(book.attention(&"t"), Attention::Suggestion);
        assert_eq!(book.attention(&"u"), Attention::Ambient);
    }

    #[test]
    fn a_pending_deselection_reads_as_retained() {
        let mut book = book();
        book.select(["a"]);
        assert_eq!(book.attention_with(&"a", false), Attention::Retained);
        assert_eq!(book.attention_with(&"b", true), Attention::Forward);
    }

    #[test]
    fn view_states_name_each_supplied_state() {
        let mut book = book();
        book.select(["a"]);
        book.set_foreground_pin("a", true);
        book.set_foreground_pin("p", true);
        book.set_suggested(["s"]);
        book.select(["b"]);
        book.set_foreground_pin("a", false);
        assert_eq!(
            book.view_states(&"a"),
            BTreeSet::from([ViewState::Retained])
        );
        assert_eq!(
            book.view_states(&"b"),
            BTreeSet::from([ViewState::Selected])
        );
        assert_eq!(
            book.view_states(&"p"),
            BTreeSet::from([ViewState::ForegroundPinned])
        );
        assert_eq!(
            book.view_states(&"s"),
            BTreeSet::from([ViewState::Suggested])
        );
        assert!(book.view_states(&"z").is_empty());
    }

    #[test]
    fn retain_prunes_every_set() {
        let mut book = book();
        book.select(["a", "b"]);
        book.set_foreground_pin("c", true);
        book.set_suggested(["d"]);
        book.select([]);
        book.retain(|key| *key == "b");
        assert_eq!(book.retained(), &BTreeSet::from(["b"]));
        assert!(book.foreground_pins().is_empty() && book.suggested().is_empty());
    }
}
