// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Which projection sources a run of captured edits touched, for
//! `IndexedDataset::refresh` (quad index plan P4).

use mere::ProjectionSource;
use uuid::Uuid;

use crate::graph::capture::CapturedDelta;

/// The sources the edits touched, or `None` when one of them needs a full
/// rebuild: a node removal (its in-neighbours' rows named it), import
/// records, and any id that does not parse.
///
/// A node edit touches that node; a relation edit touches both ends. Field
/// and coupling edits touch nothing, since the projection does not read them.
/// A resource record edit touches that resource, a resource edge edit both
/// ends, and showing a resource the surface and the resource. When an edit
/// changes a source's IRI, the refresh re-projects its dependents.
pub fn touched_sources<'a>(
    deltas: impl IntoIterator<Item = &'a CapturedDelta>,
) -> Option<Vec<ProjectionSource>> {
    use CapturedDelta as D;
    let node = |id: &str| Uuid::parse_str(id).ok().map(ProjectionSource::Node);
    let resource = |id: &str| Uuid::parse_str(id).ok().map(ProjectionSource::Resource);
    let mut touched = Vec::new();
    for delta in deltas {
        match delta {
            D::ReplayRemoveNodeById { .. } | D::ReplaySetImportRecords { .. } => return None,
            D::ReplayAddNodeWithIdIfMissing { id: node_id, .. }
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
            | D::ReplaySetNodeContentById { node_id, .. } => touched.push(node(node_id)?),
            D::ReplayAssertRelationByIds { from_id, to_id, .. }
            | D::ReplayRetractRelationsByIds { from_id, to_id, .. }
            | D::ReplayAppendTraversalByIds { from_id, to_id, .. }
            | D::ReplaySetEdgeSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplayAssertSemanticPredicateByIds { from_id, to_id, .. }
            | D::ReplaySetEdgesByIds { from_id, to_id, .. }
            | D::ReplayBranchHistoryByIds {
                child_id: from_id,
                parent_id: to_id,
                ..
            } => {
                touched.push(node(from_id)?);
                touched.push(node(to_id)?);
            },
            D::ReplaySetShownResourceById {
                surface_id,
                resource_id,
            } => {
                touched.push(node(surface_id)?);
                if let Some(resource_id) = resource_id {
                    touched.push(resource(resource_id)?);
                }
            },
            D::ReplaySetResourceRecordById { resource_id, .. } => {
                touched.push(resource(resource_id)?);
            },
            D::ReplaySetResourceEdgesByIds {
                from_resource_id,
                to_resource_id,
                ..
            } => {
                touched.push(resource(from_resource_id)?);
                touched.push(resource(to_resource_id)?);
            },
            D::ReplayAddField { .. }
            | D::ReplayRetireFieldById { .. }
            | D::ReplayRemoveFieldById { .. }
            | D::ReplayAddCoupling { .. }
            | D::ReplaySetFieldCouplingStrengthByFieldId { .. }
            | D::ReplayActivateFieldById { .. }
            | D::ReplayRetractCouplingById { .. } => {},
        }
    }
    touched.sort_unstable();
    touched.dedup();
    Some(touched)
}
