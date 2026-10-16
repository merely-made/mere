// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph grammar's view state that travels with a saved scene: the
//! saved seed and the active arrangement's regions. Member-keyed on the way
//! out, so a restored scene finds its nodes by identity; members the graph
//! no longer holds are skipped on the way in.

use forme::GraphMemberId;
use kernel::geometry::PortablePoint;

use super::Canvas;
use super::geometry::{CartographyGeometry, SavedRegion};

impl Canvas {
    /// The saved seed by member, or `None` for the recipe seed.
    pub(crate) fn saved_seed(&self) -> Option<Vec<(GraphMemberId, (f32, f32))>> {
        let saved = self.grammar.seed_book().saved()?;
        Some(
            saved
                .iter()
                .filter_map(|(key, at)| Some((self.graph.get_node(*key)?.id, (at.x, at.y))))
                .collect(),
        )
    }

    pub(crate) fn saved_regions(&self) -> Vec<SavedRegion> {
        self.strategy_regions
            .iter()
            .map(|region| SavedRegion {
                contour: region.contour.iter().map(|p| (p.x, p.y)).collect(),
                label: region.label.clone(),
                members: region
                    .members
                    .iter()
                    .filter_map(|key| Some(self.graph.get_node(*key)?.id))
                    .collect(),
            })
            .collect()
    }

    /// Restore a saved scene's seed and regions. Call after the scene's graph,
    /// arrangement and positions are in place.
    pub fn apply_cartography_scene_state(&mut self, geometry: &CartographyGeometry) {
        let key_of = |id: &GraphMemberId| self.graph.get_node_key_by_id(*id);
        let seed: Option<Vec<_>> = geometry.seed().map(|seed| {
            seed.iter()
                .filter_map(|(id, (x, y))| Some((key_of(id)?, sceno::Vec2::new(*x, *y))))
                .collect()
        });
        let regions = geometry
            .regions()
            .iter()
            .map(|region| cartography::ProjectedRegion {
                contour: region
                    .contour
                    .iter()
                    .map(|(x, y)| PortablePoint::new(*x, *y))
                    .collect(),
                label: region.label.clone(),
                members: region.members.iter().filter_map(&key_of).collect(),
            })
            .collect();
        match seed {
            Some(seed) => self.grammar.seed_book_mut().set(seed),
            None => {
                self.grammar.seed_book_mut().restore_recipe();
            },
        }
        self.strategy_regions = regions;
    }
}
