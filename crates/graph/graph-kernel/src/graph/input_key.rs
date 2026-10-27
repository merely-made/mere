// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

//! Process-local input versions. Counters alone can collide across sources
//! and independently edited candidates. These keys are never persisted.
use super::Graph;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct GraphInputKey {
    source: u64,
    version: u64,
}

fn fresh() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(1);
    NEXT.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .expect("graph input key space exhausted")
}

impl GraphInputKey {
    pub(super) fn new_source() -> Self {
        let source = fresh();
        Self {
            source,
            version: source,
        }
    }
    pub(super) fn advance(&mut self) {
        self.version = fresh();
    }
    /// Clones retain their source; independently materialized graphs do not.
    /// Equality additionally requires the same input version, including forks.
    pub fn same_source(self, other: Self) -> bool {
        self.source == other.source
    }
}

impl Graph {
    pub fn structural_input_key(&self) -> GraphInputKey {
        self.runtime_input_keys[0]
    }
    pub fn content_input_key(&self) -> GraphInputKey {
        self.runtime_input_keys[1]
    }
    pub fn url_grouping_input_key(&self) -> GraphInputKey {
        self.runtime_input_keys[2]
    }
    pub fn visit_input_key(&self) -> GraphInputKey {
        self.runtime_input_keys[3]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn input_keys_preserve_clones_separate_forks_and_do_not_cross_snapshots() {
        let mut graph = Graph::new();
        let node = graph.add_node("https://one.example".into(), Default::default());
        let structural = graph.structural_input_key();
        let content = graph.content_input_key();
        let mut clone = graph.clone();
        assert_eq!(clone.structural_input_key(), structural);
        clone.set_node_title(node, "Edited".into());
        assert_eq!(clone.structural_input_key(), structural);
        assert_ne!(clone.content_input_key(), content);
        assert!(clone.content_input_key().same_source(content));
        let mut fork = graph.clone();
        fork.set_node_title(node, "Different".into());
        assert_eq!(clone.content_revision(), fork.content_revision());
        assert_ne!(clone.content_input_key(), fork.content_input_key());
        let restored = Graph::from_snapshot(&graph.to_snapshot());
        assert!(!restored.structural_input_key().same_source(structural));
        assert_ne!(
            Graph::new().structural_input_key(),
            Graph::new().structural_input_key()
        );
    }
}
