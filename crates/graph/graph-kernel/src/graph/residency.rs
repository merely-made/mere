// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Replacing an available-data view is a load, not a journaled edit.
use super::{CoverageNote, Graph, NodeFacetStore};
use crate::persistence::GraphSnapshot;
impl Graph {
    /// Validate the complete candidate before publishing a resident replacement.
    pub fn replace_loaded_recorded_snapshot(
        &mut self,
        snapshot: &GraphSnapshot,
        facets: &NodeFacetStore,
        coverage: CoverageNote,
    ) -> Result<(), super::snapshot::ResourceSnapshotError> {
        let mut candidate = Self::try_from_recorded_snapshot(snapshot)?;
        *candidate.facets_mut() = facets.clone();
        candidate.set_known_coverage(coverage);
        candidate.revision = self
            .revision
            .checked_add(1)
            .expect("graph revision exhausted");
        candidate.content_revision = self
            .content_revision
            .checked_add(1)
            .expect("content revision exhausted");
        candidate.url_grouping_revision = self
            .url_grouping_revision
            .checked_add(1)
            .expect("URL revision exhausted");
        candidate.visit_revision = self
            .visit_revision
            .checked_add(1)
            .expect("visit revision exhausted");
        candidate.semantic_observation_revision = self
            .semantic_observation_revision
            .checked_add(1)
            .expect("observation revision exhausted");
        candidate.appearance_admission_revision = self
            .appearance_admission_revision
            .checked_add(1)
            .expect("appearance revision exhausted");
        *self = candidate;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replacement_advances_cache_revisions_and_refuses_atomically() {
        let mut graph = Graph::new();
        let key = super::super::apply::add_node(
            &mut graph,
            None,
            "https://old.test".into(),
            Default::default(),
        );
        let old = graph.revision();
        let mut other = Graph::new();
        super::super::apply::add_node(
            &mut other,
            None,
            "https://new.test".into(),
            Default::default(),
        );
        graph
            .replace_loaded_recorded_snapshot(
                &other.to_snapshot_at(0),
                other.facets(),
                Default::default(),
            )
            .unwrap();
        assert!(graph.revision() > old);
        let before = serde_json::to_value(graph.to_snapshot_at(0)).unwrap();
        let mut broken = other.to_snapshot_at(0);
        broken.shown_resources[0].resource_id = uuid::Uuid::nil().to_string();
        assert!(
            graph
                .replace_loaded_recorded_snapshot(&broken, other.facets(), Default::default())
                .is_err()
        );
        assert_eq!(
            serde_json::to_value(graph.to_snapshot_at(0)).unwrap(),
            before
        );
        let _ = key;
    }
}
