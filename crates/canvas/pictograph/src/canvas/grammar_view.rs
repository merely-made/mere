// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar's node and link presentation (Merely Design System
//! DECISIONS 2026-10-16, *Graph grammar*), on when a host asks for it.
//!
//! Forward nodes (selected or foreground-pinned) are bodies: round, ringed,
//! lifted. Everything else is a site: square, filled with its category (its
//! site's colour, the territory's). Retained nodes carry a dashed underline;
//! suggestions are the smallest mark. Labels show only for forward nodes,
//! their neighbours, retained and hovered nodes. Links stay quiet until they
//! touch a forward or hovered node. Colour carries category, not activation.

use std::collections::{BTreeMap, HashMap, HashSet};

use kernel::graph::NodeKey;
use scenograph::attention::Attention;

use super::Canvas;

/// A quiet link's alpha, as a share of its loud one (`--g-link-quiet`).
pub const LINK_QUIET: f32 = 0.22;
/// A suggestion's face, as a share of a site's.
pub const SUGGESTION_SCALE: f32 = 0.45;

/// One frame's reading of the grammar.
#[derive(Clone, Debug, Default)]
pub(crate) struct GrammarFrame {
    /// Forward and hovered nodes: the links they touch are loud.
    pub loud: HashSet<NodeKey>,
    /// Nodes whose caption shows.
    pub labelled: HashSet<NodeKey>,
    /// Each node's category index (its site, in sorted order).
    pub category: HashMap<NodeKey, usize>,
}

impl Canvas {
    /// Present the graph by the grammar (bodies, sites, quiet links), or by
    /// the activation colours it supersedes.
    pub fn set_graph_grammar(&mut self, on: bool) {
        self.graph_grammar = on;
    }

    pub fn graph_grammar(&self) -> bool {
        self.graph_grammar
    }

    fn attention_of_key(&self, key: NodeKey) -> Attention {
        self.grammar
            .attention_book()
            .attention_with(&key, self.selected.contains(&key))
    }

    /// This frame's grammar, or `None` when the grammar is off.
    pub(crate) fn grammar_frame(&mut self) -> Option<GrammarFrame> {
        if !self.graph_grammar {
            return None;
        }
        let forward: HashSet<NodeKey> = self
            .graph
            .nodes()
            .map(|(key, _)| key)
            .filter(|key| self.attention_of_key(*key) == Attention::Forward)
            .collect();
        let hovered = self.pick_at(self.cursor);
        let mut loud = forward.clone();
        loud.extend(hovered);
        let mut labelled = loud.clone();
        for (_, relation) in self.graph.projected_relations() {
            if forward.contains(&relation.from) {
                labelled.insert(relation.to);
            }
            if forward.contains(&relation.to) {
                labelled.insert(relation.from);
            }
        }
        labelled.extend(
            self.graph
                .nodes()
                .map(|(key, _)| key)
                .filter(|key| self.attention_of_key(*key) == Attention::Retained),
        );
        let sites = self.channels.sites(&self.graph).clone();
        let mut names: Vec<&String> = sites.values().collect();
        names.sort();
        names.dedup();
        let index: BTreeMap<&String, usize> =
            names.into_iter().enumerate().map(|(i, n)| (n, i)).collect();
        let category = sites
            .iter()
            .map(|(key, site)| (*key, index[site]))
            .collect();
        Some(GrammarFrame {
            loud,
            labelled,
            category,
        })
    }

    /// The gnode's classes under the grammar, before its representation class.
    pub(crate) fn grammar_class(&self, key: NodeKey, frame: &GrammarFrame) -> String {
        let attention = self.attention_of_key(key);
        let mut class = match attention {
            Attention::Forward => "gnode-body".to_string(),
            _ => format!(
                "gnode-site gnode-cat-{}",
                frame.category.get(&key).copied().unwrap_or(0) % super::territory::CATEGORY_COUNT
            ),
        };
        match attention {
            Attention::Retained => class.push_str(" gnode-retained"),
            Attention::Suggestion => class.push_str(" gnode-suggestion"),
            _ => {},
        }
        if !frame.labelled.contains(&key) {
            class.push_str(" gnode-quiet");
        }
        class
    }

    /// A node's face scale under the grammar: suggestions are smallest.
    pub(crate) fn grammar_face_scale(&self, key: NodeKey) -> f32 {
        if self.attention_of_key(key) == Attention::Suggestion {
            SUGGESTION_SCALE
        } else {
            1.0
        }
    }
}

/// A link's alpha share: loud when it touches a loud node, else quiet.
pub(crate) fn link_tone(frame: Option<&GrammarFrame>, from: NodeKey, to: NodeKey) -> f32 {
    match frame {
        Some(frame) if !frame.loud.contains(&from) && !frame.loud.contains(&to) => LINK_QUIET,
        _ => 1.0,
    }
}
