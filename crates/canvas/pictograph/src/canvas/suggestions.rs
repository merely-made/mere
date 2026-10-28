// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Suggestions on the canvas, paced by the host's settings (graph grammar,
//! DECISIONS 2026-10-16). Off by default: nothing a scene offers shows until
//! the person turns suggestions on. Then at most `at_once` show, fewer over
//! the last tenth of the things they keep track of and as their content
//! fills the plate.

use std::collections::HashSet;

use kernel::graph::NodeKey;
use scenograph::suggestion::SuggestionSettings;

use super::Canvas;
use super::link_suggestions::LINK_SOURCE;

impl Canvas {
    pub fn suggestion_settings(&self) -> &SuggestionSettings {
        &self.grammar.suggestions
    }

    /// Replace the host's suggestion settings and re-admit the last offer.
    pub fn set_suggestion_settings(&mut self, settings: SuggestionSettings) {
        self.grammar.suggestions = settings;
        self.admit_suggestions();
    }

    /// Offer `candidates` (member, source) in the scene's order, replacing
    /// the last offer. Members not in the graph are skipped.
    pub fn offer_suggestions(
        &mut self,
        candidates: impl IntoIterator<Item = (uuid::Uuid, String)>,
    ) {
        self.grammar.offered = candidates
            .into_iter()
            .filter_map(|(member, source)| Some((self.graph.get_node_key_by_id(member)?, source)))
            .collect();
        self.admit_suggestions();
    }

    /// How full the plate is: the faces' area over the viewport's, 0 to 1.
    /// An unsized canvas has no plate yet, so nothing fills it.
    pub fn plate_fill(&self) -> f32 {
        self.fill_of(self.view.positions().map(|(key, _)| key))
    }

    fn fill_of(&self, keys: impl Iterator<Item = NodeKey>) -> f32 {
        if self.view_w == 0 || self.view_h == 0 {
            return 0.0;
        }
        let viewport = (self.view_w * self.view_h) as f32;
        let zoom = self.camera.zoom;
        let faces: f32 = keys.map(|key| (self.node_size(key) * zoom).powi(2)).sum();
        (faces / viewport).clamp(0.0, 1.0)
    }

    /// What the person keeps: the held nodes neither on offer nor derived,
    /// counted and measured against the plate. Suggestions are no part of it,
    /// so minting one never changes what admits it.
    fn kept_content(&self) -> (usize, f32) {
        let offered: HashSet<NodeKey> = self.grammar.offered.iter().map(|(key, _)| *key).collect();
        let kept: HashSet<NodeKey> = self
            .graph
            .nodes()
            .filter(|(key, node)| !offered.contains(key) && !self.is_derived_member(node.id))
            .map(|(key, _)| key)
            .collect();
        let fill = self.fill_of(
            self.view
                .positions()
                .map(|(key, _)| key)
                .filter(|k| kept.contains(k)),
        );
        (kept.len(), fill)
    }

    /// The one admission. The scene's offers and the links pages carry make a
    /// single candidate set; the settings admit from it once, by source,
    /// against the count and fill of what the person keeps. Derived link
    /// members are minted or removed to match and attention reads the same
    /// set, so a link is on the canvas exactly when it is admitted.
    pub(crate) fn admit_suggestions(&mut self) {
        self.prune_derived();
        let links = self.link_candidates();
        let mut candidates: Vec<(uuid::Uuid, String)> = self
            .grammar
            .offered
            .iter()
            .filter_map(|(key, source)| Some((self.graph.get_node(*key)?.id, source.clone())))
            .collect();
        candidates.extend(
            links
                .iter()
                .map(|link| (link.member, LINK_SOURCE.to_string())),
        );
        let (kept, fill) = self.kept_content();
        let admitted = self.grammar.suggestions.admit(candidates, kept, fill);
        self.sync_link_members(&links, &admitted);
        let keys: Vec<NodeKey> = admitted
            .iter()
            .filter_map(|member| self.graph.get_node_key_by_id(*member))
            .collect();
        self.grammar.attention_book_mut().set_suggested(keys);
    }
}
