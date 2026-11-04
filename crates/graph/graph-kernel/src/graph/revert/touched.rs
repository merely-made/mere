// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Touched {
    /// What `edits` name.
    pub fn of<'a>(edits: impl IntoIterator<Item = &'a CapturedDelta>) -> Self {
        let mut touched = Self::default();
        for edit in edits {
            touched.add(edit);
        }
        touched
    }

    pub(super) fn node(&mut self, id: &str) {
        self.nodes.extend(uuid(id));
    }

    pub(super) fn edge(&mut self, from: &str, to: &str) {
        if let (Some(from), Some(to)) = (uuid(from), uuid(to)) {
            self.edges.insert((from, to));
        }
    }

    // Exhaustive on purpose: a new edit kind must say what it touches.
    pub(super) fn add(&mut self, edit: &CapturedDelta) {
        use CapturedDelta as D;
        match edit {
            D::ReplayAddNodeWithIdIfMissing { id, .. } => self.node(id),
            D::ReplayRemoveNodeById { node_id }
            | D::ReplaySetNodeTitleById { node_id, .. }
            | D::ReplaySetNodeUrlById { node_id, .. }
            | D::ReplaySetNodeImageById { node_id, .. }
            | D::ReplaySetNodeThumbnailById { node_id, .. }
            | D::ReplaySetNodeFaviconById { node_id, .. }
            | D::ReplaySetNodeMimeHintById { node_id, .. }
            | D::ReplaySetNodeNestedById { node_id, .. }
            | D::ReplaySetNodePinnedById { node_id, .. }
            | D::ReplaySetNodeFacetById { node_id, .. }
            | D::ReplayRemoveNodeFacetById { node_id, .. }
            | D::ReplayInsertNodeTagById { node_id, .. }
            | D::ReplayRemoveNodeTagById { node_id, .. }
            | D::ReplaySetNodeBodyById { node_id, .. }
            | D::ReplayNavigateNodeById { node_id, .. }
            | D::ReplayNodeHistoryBackById { node_id, .. }
            | D::ReplayNodeHistoryForwardById { node_id, .. }
            | D::ReplayAppendNodePropertyById { node_id, .. }
            | D::ReplayAddNodeClassificationById { node_id, .. }
            | D::ReplayRemoveNodeClassificationById { node_id, .. }
            | D::ReplaySetNodeClassificationStatusById { node_id, .. }
            | D::ReplaySetNodePrimaryClassificationById { node_id, .. }
            | D::ReplayRecordNodeDerivationById { node_id, .. }
            | D::ReplaySetNodeTagIconOverrideById { node_id, .. }
            | D::ReplayAppendFrameLayoutHintById { node_id, .. }
            | D::ReplayRemoveFrameLayoutHintById { node_id, .. }
            | D::ReplayMoveFrameLayoutHintById { node_id, .. }
            | D::ReplaySetFrameSplitOfferSuppressedById { node_id, .. }
            | D::ReplayUpdateNodeHistoryById { node_id, .. }
            | D::ReplayTouchNodeLastVisitedById { node_id, .. }
            | D::ReplaySetNodeContentById { node_id, .. } => self.node(node_id),
            D::ReplayBranchHistoryByIds {
                child_id,
                parent_id,
            } => {
                self.node(child_id);
                self.node(parent_id);
            },
            D::ReplayAssertRelationByIds { from_id, to_id, .. }
            | D::ReplayRetractRelationsByIds { from_id, to_id, .. }
            | D::ReplayAppendTraversalByIds { from_id, to_id, .. }
            | D::ReplaySetEdgeSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplayAssertSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplaySetEdgesByIds { from_id, to_id, .. } => self.edge(from_id, to_id),
            D::ReplaySetImportRecords { .. } => self.import_records = true,
            D::ReplayAddField { field } => self.fields.extend(uuid(&field.id)),
            D::ReplayRetireFieldById { field_id }
            | D::ReplayActivateFieldById { field_id }
            | D::ReplayRemoveFieldById { field_id } => self.fields.extend(uuid(field_id)),
            D::ReplaySetFieldCouplingStrengthByFieldId { field_id, .. } => {
                self.coupled_fields.extend(uuid(field_id));
            },
            D::ReplayAddCoupling { coupling } => self.couplings.extend(uuid(&coupling.id)),
            D::ReplayRetractCouplingById { coupling_id } => {
                self.couplings.extend(uuid(coupling_id));
            },
            D::ReplaySetResourceRecordById { resource_id, .. } => {
                self.resources.extend(uuid(resource_id));
            },
            D::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                ..
            } => {
                if let (Some(from), Some(to)) = (uuid(from_resource_id), uuid(to_resource_id)) {
                    self.resource_edges.insert((from, to));
                }
            },
            D::ReplaySetShownResourceById {
                surface_id,
                resource_id,
            } => {
                self.shown_surfaces.extend(uuid(surface_id));
                self.resources.extend(resource_id.as_deref().and_then(uuid));
            },
        }
    }

    /// Whether these edits reach `part`: how a caller learns who changed a
    /// part that undo kept.
    pub fn reaches(&self, part: &Part) -> bool {
        match part {
            Part::Node(id)
            | Part::Title(id)
            | Part::Url(id)
            | Part::Tag(id, _)
            | Part::Body(id)
            | Part::Content(id)
            | Part::MediaType(id)
            | Part::Nested(id)
            | Part::Facet(id, _)
            | Part::History(id) => {
                self.nodes.contains(id)
                    || self.shown_surfaces.contains(id)
                    || self.edges.iter().any(|(from, to)| from == id || to == id)
            },
            Part::Edges(from, to) => {
                self.edges.contains(&(*from, *to))
                    || self.nodes.contains(from)
                    || self.nodes.contains(to)
            },
            Part::Field(id) => self.fields.contains(id),
            Part::Coupling(id) => self.couplings.contains(id) || !self.coupled_fields.is_empty(),
            Part::ImportRecords => self.import_records,
            Part::Resource(id) | Part::ResourceFacet(id, _) => {
                self.resources.contains(id)
                    || self
                        .resource_edges
                        .iter()
                        .any(|(from, to)| from == id || to == id)
            },
            Part::ResourceEdges(from, to) => {
                self.resource_edges.contains(&(*from, *to))
                    || self.resources.contains(from)
                    || self.resources.contains(to)
            },
            Part::ShownResource(id) => self.shown_surfaces.contains(id) || self.nodes.contains(id),
        }
    }
}
