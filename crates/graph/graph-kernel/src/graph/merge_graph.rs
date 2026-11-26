// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The kernel graph as the graph a session merges over (engine split D264).
//! Each method is the kernel call pandect's merge made before the split, so
//! a merge over the kernel graph does exactly what it did.

use std::collections::{BTreeMap, BTreeSet};

use mere::session::{MergeGraph, MergePart, MergeTouch};
use serde_json::Value;
use uuid::Uuid;

use super::capture::{CapturedDelta, replay_captured_deltas_onto};
use super::revert::{self, Part, Touched};
use super::{Graph, ProvenanceSubKind};
use crate::types::NodeDerivation;

impl MergeGraph for Graph {
    fn part_of(edit: &CapturedDelta) -> Option<Part> {
        revert::part_of(edit)
    }

    fn part_node(part: &Part) -> Option<Uuid> {
        revert::part_node(part)
    }

    fn part_kind(part: &Part) -> MergePart {
        match part {
            Part::Node(id) => MergePart::Node(*id),
            Part::Edges(from, to) => MergePart::Edges(*from, *to),
            Part::ResourceFacet(resource, facet) => {
                MergePart::ResourceFacet(*resource, facet.clone())
            },
            _ => MergePart::Other,
        }
    }

    fn part_value(&self, part: &Part) -> Option<Value> {
        revert::part_value(self, part)
    }

    fn part_edits(&self, part: &Part, value: Option<&Value>) -> Result<Vec<CapturedDelta>, String> {
        revert::part_edits(self, part, value)
    }

    fn node_ids(&self) -> BTreeSet<Uuid> {
        self.nodes().map(|(_, node)| node.id).collect()
    }

    fn has_node(&self, id: Uuid) -> bool {
        self.get_node_by_id(id).is_some()
    }

    fn copied_from(&self, id: Uuid, container: &str) -> Option<Uuid> {
        let (key, _) = self.get_node_by_id(id)?;
        self.node_derivations(key)?
            .into_iter()
            .find(|derivation| {
                derivation.sub_kind == ProvenanceSubKind::CopiedFrom
                    && derivation.source_graph.as_deref() == Some(container)
            })
            .and_then(|derivation| Uuid::parse_str(&derivation.source_node).ok())
    }

    fn recreate_node_edits(&self, id: Uuid) -> Option<Vec<CapturedDelta>> {
        revert::recreate_node_edits(self, id)
    }

    fn incident_edge_edits(&self, nodes: &[Uuid]) -> Vec<CapturedDelta> {
        let mut pairs = BTreeSet::new();
        let mut edits = Vec::new();
        for id in nodes {
            let Some((key, _)) = self.get_node_by_id(*id) else {
                continue;
            };
            for (other_key, other) in self.nodes() {
                for ((from_key, from), (to_key, to)) in [
                    ((key, *id), (other_key, other.id)),
                    ((other_key, other.id), (key, *id)),
                ] {
                    if !pairs.insert((from, to)) {
                        continue;
                    }
                    let edges = self.persisted_edges_between(from_key, to_key);
                    if !edges.is_empty() {
                        edits.push(CapturedDelta::ReplaySetEdgesByIds {
                            from_id: from.to_string(),
                            to_id: to.to_string(),
                            edges,
                        });
                    }
                }
            }
        }
        edits
    }

    fn merged_from_edit(
        node: Uuid,
        origin: Uuid,
        container: Option<String>,
    ) -> Option<CapturedDelta> {
        Some(CapturedDelta::ReplayRecordNodeDerivationById {
            node_id: node.to_string(),
            derivation: NodeDerivation {
                sub_kind: ProvenanceSubKind::MergedFrom,
                source_node: origin.to_string(),
                source_graph: container,
            },
        })
    }

    fn remap_ids(edit: &CapturedDelta, names: &BTreeMap<String, String>) -> CapturedDelta {
        edit.remap_ids(names)
    }

    fn touched(edits: &[CapturedDelta]) -> MergeTouch {
        let touched = Touched::of(edits);
        MergeTouch {
            session_level: !touched.fields.is_empty()
                || !touched.couplings.is_empty()
                || !touched.coupled_fields.is_empty()
                || touched.import_records,
            nodes: touched.nodes,
            edges: touched.edges,
            shown_surfaces: touched.shown_surfaces,
        }
    }

    fn replay_scratch(&mut self, edits: &[CapturedDelta]) {
        replay_captured_deltas_onto(self, edits.iter().cloned());
    }
}
