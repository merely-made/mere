// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl Canvas {
    /// Apply a structural mutation to the session graph and reconcile every derived
    /// view, so externally-ingested nodes/edges (e.g. a linked-data merge) join the
    /// spatial field. Kernel structural/membership changes require reconciliation
    /// even when `mutate` reports false. On a reported or observed change,
    /// the newly-added nodes are fanned out around the current selection (they are
    /// minted at the origin, which the force sim must not stack), and the settle
    /// restarts. Returns whether the graph changed.
    ///
    /// canvas-host stays free of the linked-data bridge: the host passes the merge
    /// in as a closure over [`Graph`].
    pub fn ingest_graph<F: FnOnce(&mut Graph) -> bool>(&mut self, mutate: F) -> bool {
        // A reused graph slot is not the same live member. The kernel authors
        // these witnesses during each mutation, including within this closure.
        let before: std::collections::HashMap<_, _> = self
            .graph
            .nodes()
            .map(|(key, node)| (key, self.graph.surface_node_lifetime(node.id)))
            .collect();
        let revision = self.graph.revision();
        let claimed_change = mutate(&mut self.graph);
        let memberships_changed = self.graph.node_count() != before.len()
            || self.graph.nodes().any(|(key, node)| {
                before.get(&key) != Some(&self.graph.surface_node_lifetime(node.id))
            });
        if !claimed_change && self.graph.revision() == revision && !memberships_changed {
            return false;
        }
        self.reconcile_derived();
        let anchor = self
            .selected
            .iter()
            .copied()
            .next()
            .and_then(|k| self.view.position_of(k))
            .unwrap_or(Point2D::new(0.0, 0.0));
        let seeds: Vec<_> = self
            .graph
            .nodes()
            .filter(|(key, node)| {
                before.get(key) != Some(&self.graph.surface_node_lifetime(node.id))
            })
            .map(|(key, _)| key)
            .enumerate()
            .map(|(i, k)| {
                let col = (i % 6) as f32;
                let row = (i / 6) as f32;
                (
                    k,
                    Point2D::new(anchor.x + 12.0 + col * 16.0, anchor.y + 12.0 + row * 16.0),
                )
            })
            .collect();
        for &(key, pos) in &seeds {
            self.view.set_position(key, pos);
        }
        self.reset_frame_time();
        self.physics.seed(seeds);
        self.settle_physics(SETTLE_TICKS);
        true
    }

    /// Install returned owner data when its physical topology is unchanged.
    /// Refuse additions, removals or changed projected relations before mutation;
    /// the caller uses structural ingest for those. Like recorded metadata
    /// refresh, this rebuilds derived data without restarting the settled layout.
    pub fn refresh_same_topology_graph(&mut self, source: &Graph) -> Result<(), String> {
        if self.graph.node_count() != source.node_count()
            || self.graph.nodes().any(|(key, node)| {
                source.get_node(key).is_none_or(|next| next.id != node.id)
                    || source.surface_node_lifetime(node.id)
                        != self.graph.surface_node_lifetime(node.id)
            })
        {
            return Err("owner metadata changes the canvas node topology".into());
        }
        let relations = |graph: &Graph| {
            let mut counts = std::collections::HashMap::new();
            for (key, relation) in graph.projected_relations() {
                *counts.entry((key, relation)).or_insert(0usize) += 1;
            }
            counts
        };
        if relations(&self.graph) != relations(source) {
            return Err("owner metadata changes the canvas relation topology".into());
        }
        self.graph = source.clone();
        self.reconcile_derived();
        Ok(())
    }

    /// Refresh runtime cache and known coverage independently of geometry.
    pub fn refresh_semantic_context(&mut self, source: &Graph) {
        self.graph.copy_semantic_context_from(source);
    }

    /// Refresh exact hosted content without restarting layout or physics.
    pub fn refresh_recorded_metadata(
        &mut self,
        deltas: &[kernel::graph::CapturedDelta],
    ) -> Result<bool, String> {
        use kernel::graph::CapturedDelta;
        let mut expected = self.graph.to_snapshot();
        let uuid = |value: &str| uuid::Uuid::parse_str(value).map_err(|error| error.to_string());
        for delta in deltas {
            match delta {
                CapturedDelta::ReplaySetResourceRecordById {
                    resource_id,
                    record,
                } => {
                    let id = uuid(resource_id)?;
                    let index = expected.resources.iter().position(|record| {
                        chartulary::resource_id_from_canonical_iri(&record.canonical_iri) == id
                    });
                    match (index, record) {
                        (Some(index), Some(record)) => expected.resources[index] = record.clone(),
                        (None, Some(record)) => expected.resources.push(record.clone()),
                        (Some(index), None) => {
                            expected.resources.remove(index);
                        },
                        (None, None) => {},
                    }
                    if let Some(record) = record
                        && chartulary::resource_id_from_canonical_iri(&record.canonical_iri) != id
                    {
                        return Err("recorded resource identity does not match its record".into());
                    }
                },
                CapturedDelta::ReplaySetResourceEdgesByIds {
                    from_resource_id,
                    to_resource_id,
                    edges,
                } => {
                    let from = uuid(from_resource_id)?;
                    let to = uuid(to_resource_id)?;
                    expected.resource_edges.retain(|edge| {
                        uuid(&edge.from_node_id).ok() != Some(from)
                            || uuid(&edge.to_node_id).ok() != Some(to)
                    });
                    if edges.iter().any(|edge| {
                        uuid(&edge.from_node_id).ok() != Some(from)
                            || uuid(&edge.to_node_id).ok() != Some(to)
                    }) {
                        return Err("recorded resource pair does not match its endpoints".into());
                    }
                    expected.resource_edges.extend(edges.iter().cloned());
                },
                CapturedDelta::ReplaySetShownResourceById {
                    surface_id,
                    resource_id,
                } => {
                    let surface = uuid(surface_id)?;
                    if !expected
                        .nodes
                        .iter()
                        .any(|node| uuid(&node.node_id).ok() == Some(surface))
                    {
                        return Err("recorded content names an absent Surface".into());
                    }
                    expected
                        .shown_resources
                        .retain(|shown| uuid(&shown.surface_id).ok() != Some(surface));
                    if let Some(resource) = resource_id {
                        uuid(resource)?;
                        expected.shown_resources.push(
                            kernel::persistence::PersistedShownResource {
                                surface_id: surface.to_string(),
                                resource_id: resource.clone(),
                            },
                        );
                    }
                },
                CapturedDelta::ReplaySetNodeTitleById { node_id, title } => {
                    let id = uuid(node_id)?;
                    let node = expected
                        .nodes
                        .iter_mut()
                        .find(|node| uuid(&node.node_id).ok() == Some(id))
                        .ok_or("recorded title names an absent Surface")?;
                    node.title = title.clone();
                },
                _ => {
                    return Err(
                        "canvas metadata refresh requires recorded content or title captures"
                            .into(),
                    );
                },
            }
        }
        let expected = Graph::try_from_recorded_snapshot(&expected)
            .map_err(|error| error.to_string())?
            .to_snapshot();
        let mut scratch = self.graph.clone();
        kernel::graph::replay_captured_deltas_onto(&mut scratch, deltas.iter().cloned());
        scratch
            .validate_active_resource_assertion_handles()
            .map_err(|error| error.to_string())?;
        let actual = scratch.to_snapshot();
        // The snapshot keeps Resource records, and the parallel edges of one
        // pair, in the order they were added; the shown bindings are already
        // in Surface id order.
        let content = |snapshot: &kernel::persistence::GraphSnapshot| {
            let mut records: Vec<_> = snapshot
                .resources
                .iter()
                .map(|record| serde_json::to_string(record).unwrap())
                .collect();
            let mut pairs: Vec<_> = snapshot
                .resource_edges
                .iter()
                .map(|edge| serde_json::to_string(edge).unwrap())
                .collect();
            records.sort();
            pairs.sort();
            (records, pairs)
        };
        if content(&actual) != content(&expected)
            || actual.shown_resources != expected.shown_resources
            || actual
                .nodes
                .iter()
                .map(|node| (&node.node_id, &node.title))
                .collect::<Vec<_>>()
                != expected
                    .nodes
                    .iter()
                    .map(|node| (&node.node_id, &node.title))
                    .collect::<Vec<_>>()
        {
            return Err("canvas metadata captures did not reproduce their recorded content".into());
        }
        let titles_changed = self.graph.nodes().any(|(_, node)| {
            scratch
                .get_node_by_id(node.id)
                .is_none_or(|(_, updated)| updated.title != node.title)
        });
        let changed = scratch.revision() != self.graph.revision() || titles_changed;
        if changed {
            self.graph = scratch;
            self.reconcile_derived();
        }
        Ok(changed)
    }
}
