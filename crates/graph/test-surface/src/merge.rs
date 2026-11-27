// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The surface as a graph a session merges over (engine split D264).

use std::collections::{BTreeMap, BTreeSet};

use mere::model::{JournalDelta, ResourceDelta};
use mere::session::{MergeGraph, MergePart, MergeTouch, OnRefusal};
use uuid::Uuid;

use portolan::FacetId;

use super::{COPIED_FROM, Delta, Part, PartKey, Surface, SurfaceDelta};
/// A surface merges by its bindings: a surface is the node a merge moves
/// whole, its resources and their pairs ride as parts, and it records no
/// derivations (engine split D264).
impl MergeGraph for Surface {
    fn part_of(edit: &Delta) -> Option<Part> {
        PartKey::of(edit).map(|key| key.part())
    }

    fn part_node(part: &Part) -> Option<Uuid> {
        match part {
            Part::Surface(id) => Some(*id),
            _ => None,
        }
    }

    fn part_kind(part: &Part) -> MergePart {
        match part {
            Part::Surface(id) => MergePart::Node(*id),
            _ => MergePart::Other,
        }
    }

    fn part_value(&self, part: &Part) -> Option<serde_json::Value> {
        match part {
            Part::Surface(id) => self.shown(*id).map(|resource| resource.to_string().into()),
            Part::Resource(id) => self
                .resources
                .resource_record(*id)
                .map(|record| serde_json::to_value(record).expect("serializable record")),
            Part::Pair(a, b) => {
                let held = self.resources.persisted_edges_between(*a, *b);
                (!held.is_empty()).then(|| serde_json::to_value(held).expect("serializable edges"))
            },
        }
    }

    fn part_edits(
        &self,
        part: &Part,
        value: Option<&serde_json::Value>,
    ) -> Result<Vec<Delta>, String> {
        let error = |error: serde_json::Error| error.to_string();
        Ok(vec![match part {
            Part::Surface(id) => JournalDelta::Surface(match value {
                Some(value) => SurfaceDelta::ShowResource {
                    surface_id: *id,
                    resource_id: serde_json::from_value(value.clone()).map_err(error)?,
                },
                None => SurfaceDelta::HideSurface { surface_id: *id },
            }),
            Part::Resource(id) => {
                JournalDelta::Resource(ResourceDelta::ReplaySetResourceRecordById {
                    resource_id: id.to_string(),
                    record: value
                        .map(|value| serde_json::from_value(value.clone()))
                        .transpose()
                        .map_err(error)?,
                })
            },
            Part::Pair(a, b) => {
                JournalDelta::Resource(ResourceDelta::ReplaySetResourceEdgesByIds {
                    from_resource_id: a.to_string(),
                    to_resource_id: b.to_string(),
                    edges: value
                        .map(|value| serde_json::from_value(value.clone()))
                        .transpose()
                        .map_err(error)?
                        .unwrap_or_default(),
                })
            },
        }])
    }

    fn node_ids(&self) -> BTreeSet<Uuid> {
        self.shown.keys().copied().collect()
    }

    fn has_node(&self, id: Uuid) -> bool {
        self.shown.contains_key(&id)
    }

    fn copied_from(&self, id: Uuid, container: &str) -> Option<Uuid> {
        let held = self.facets.get(&id, &FacetId::new(COPIED_FROM))?;
        (held["container"].as_str() == Some(container))
            .then(|| held["origin"].as_str()?.parse().ok())
            .flatten()
    }

    fn recreate_node_edits(&self, id: Uuid) -> Option<Vec<Delta>> {
        self.has_node(id)
            .then(|| vec![self.restore(&PartKey::Surface(id))])
    }

    fn incident_edge_edits(&self, _nodes: &[Uuid]) -> Vec<Delta> {
        Vec::new()
    }

    fn merged_from_edit(_node: Uuid, _origin: Uuid, _container: Option<String>) -> Option<Delta> {
        None
    }

    fn remap_ids(edit: &Delta, names: &BTreeMap<String, String>) -> Delta {
        let swap = |id: &Uuid| {
            names
                .get(&id.to_string())
                .and_then(|mapped| Uuid::parse_str(mapped).ok())
                .unwrap_or(*id)
        };
        match edit {
            JournalDelta::Surface(SurfaceDelta::ShowResource {
                surface_id,
                resource_id,
            }) => JournalDelta::Surface(SurfaceDelta::ShowResource {
                surface_id: swap(surface_id),
                resource_id: *resource_id,
            }),
            JournalDelta::Surface(SurfaceDelta::HideSurface { surface_id }) => {
                JournalDelta::Surface(SurfaceDelta::HideSurface {
                    surface_id: swap(surface_id),
                })
            },
            other => other.clone(),
        }
    }

    fn touched(edits: &[Delta]) -> MergeTouch {
        let mut touch = MergeTouch::default();
        for key in Self::touched(edits) {
            if let PartKey::Surface(id) = key {
                touch.nodes.insert(id);
                touch.shown_surfaces.insert(id);
            }
        }
        touch
    }

    fn replay_scratch(&mut self, edits: &[Delta]) {
        let _ = self.replay_all(edits, OnRefusal::Drop);
    }
}
