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

use kernel::graph::NodeKey;
use scenograph::suggestion::SuggestionSettings;

use super::Canvas;

impl Canvas {
    pub fn suggestion_settings(&self) -> &SuggestionSettings {
        &self.grammar.suggestions
    }

    /// Replace the host's suggestion settings and re-admit the last offer.
    pub fn set_suggestion_settings(&mut self, settings: SuggestionSettings) {
        self.grammar.suggestions = settings;
        self.refresh_link_suggestions();
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
        if self.view_w == 0 || self.view_h == 0 {
            return 0.0;
        }
        let viewport = (self.view_w * self.view_h) as f32;
        let zoom = self.camera.zoom;
        let faces: f32 = self
            .view
            .positions()
            .map(|(key, _)| (self.node_size(key) * zoom).powi(2))
            .sum();
        (faces / viewport).clamp(0.0, 1.0)
    }

    pub(crate) fn admit_suggestions(&mut self) {
        let mut candidates = self.grammar.offered.clone();
        candidates.extend(self.link_suggestion_members());
        let offered: Vec<NodeKey> = candidates.iter().map(|(key, _)| *key).collect();
        // What the person keeps: everything held that is not on offer.
        let kept = self
            .graph
            .nodes()
            .filter(|(key, node)| !offered.contains(key) && !self.is_derived_member(node.id))
            .count();
        let admitted = self
            .grammar
            .suggestions
            .admit(candidates, kept, self.plate_fill());
        self.grammar.attention_book_mut().set_suggested(admitted);
    }
}
