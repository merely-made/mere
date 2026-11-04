// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl CapturedDelta {
    /// This edit with each id in `ids` (old to new, as canonical UUID text)
    /// rewritten where the edit holds an id, and every other string as it was:
    /// a title, tag, url or body that reads like an id is text.
    ///
    /// The id fields are those [`Touched`] reads, and the ids those edits carry
    /// in their payloads: the ends of a relation, an import membership, a
    /// field's attached node and a frame-layout member. A node facet's value is
    /// rewritten through [`remap_facet_value`]; a resource record and a
    /// derivation are carried as written.
    ///
    /// Exhaustive on purpose, as `Touched::add` is: a new edit kind must say
    /// which of its fields are ids.
    pub fn remap_ids(&self, ids: &BTreeMap<String, String>) -> CapturedDelta {
        use CapturedDelta as D;
        let swap = |id: &mut String| {
            if let Some(mapped) = ids.get(id.as_str()) {
                id.clone_from(mapped);
            }
        };
        let mut edit = self.clone();
        match &mut edit {
            D::ReplayAddNodeWithIdIfMissing { id, .. } => swap(id),
            D::ReplayRemoveNodeById { node_id }
            | D::ReplaySetNodeTitleById { node_id, .. }
            | D::ReplaySetNodeUrlById { node_id, .. }
            | D::ReplaySetNodeImageById { node_id, .. }
            | D::ReplaySetNodeThumbnailById { node_id, .. }
            | D::ReplaySetNodeFaviconById { node_id, .. }
            | D::ReplaySetNodeMimeHintById { node_id, .. }
            | D::ReplaySetNodeNestedById { node_id, .. }
            | D::ReplaySetNodePinnedById { node_id, .. }
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
            // A derivation's `source_node` names a node of another graph: kept.
            | D::ReplayRecordNodeDerivationById { node_id, .. }
            | D::ReplaySetNodeTagIconOverrideById { node_id, .. }
            | D::ReplayRemoveFrameLayoutHintById { node_id, .. }
            | D::ReplayMoveFrameLayoutHintById { node_id, .. }
            | D::ReplaySetFrameSplitOfferSuppressedById { node_id, .. }
            | D::ReplayUpdateNodeHistoryById { node_id, .. }
            | D::ReplayTouchNodeLastVisitedById { node_id, .. }
            | D::ReplaySetNodeContentById { node_id, .. } => swap(node_id),
            D::ReplaySetNodeFacetById {
                node_id,
                facet,
                value_json,
            } => {
                swap(node_id);
                *value_json = remap_facet_value(facet, value_json, &swap);
            },
            D::ReplayAppendFrameLayoutHintById { node_id, hint } => {
                swap(node_id);
                remap_hint(hint, &swap);
            },
            D::ReplayBranchHistoryByIds {
                child_id,
                parent_id,
            } => {
                swap(child_id);
                swap(parent_id);
            },
            D::ReplayAssertRelationByIds { from_id, to_id, .. }
            | D::ReplayRetractRelationsByIds { from_id, to_id, .. }
            | D::ReplayAppendTraversalByIds { from_id, to_id, .. }
            | D::ReplaySetEdgeSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplayAssertSemanticPredicateByIds { from_id, to_id, .. } => {
                swap(from_id);
                swap(to_id);
            },
            D::ReplaySetEdgesByIds {
                from_id,
                to_id,
                edges,
            } => {
                swap(from_id);
                swap(to_id);
                remap_ends(edges, &swap);
            },
            D::ReplaySetImportRecords { import_records } => {
                for member in import_records.iter_mut().flat_map(|r| &mut r.memberships) {
                    swap(&mut member.node_id);
                }
            },
            D::ReplayAddField { field } => {
                swap(&mut field.id);
                if let PersistedFieldExtent::AttachedToNode(node) = &mut field.extent {
                    swap(node);
                }
            },
            D::ReplayRetireFieldById { field_id }
            | D::ReplayActivateFieldById { field_id }
            | D::ReplayRemoveFieldById { field_id }
            | D::ReplaySetFieldCouplingStrengthByFieldId { field_id, .. } => swap(field_id),
            D::ReplayAddCoupling { coupling } => {
                swap(&mut coupling.id);
                swap(&mut coupling.field_id);
            },
            D::ReplayRetractCouplingById { coupling_id } => swap(coupling_id),
            D::ReplaySetResourceRecordById { resource_id, .. } => swap(resource_id),
            D::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                edges,
            } => {
                swap(from_resource_id);
                swap(to_resource_id);
                remap_ends(edges, &swap);
            },
            D::ReplaySetShownResourceById {
                surface_id,
                resource_id,
            } => {
                swap(surface_id);
                if let Some(resource) = resource_id {
                    swap(resource);
                }
            },
        }
        edit
    }
}
