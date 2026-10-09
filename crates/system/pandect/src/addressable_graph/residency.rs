// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::VecDeque;
impl<B: Backend> ResidentGraph<B> {
    /// Reconcile against the latest published checkpoint. Failed reads never
    /// replace the previously available graph or its source generation.
    pub async fn reconcile(&mut self, roots: ResidencyRoots) -> Result<(), StoreError> {
        self.update(roots, BTreeSet::new(), self.policy, false)
            .await
    }
    pub async fn refresh(&mut self) -> Result<(), StoreError> {
        self.update(self.roots.clone(), self.demands.clone(), self.policy, false)
            .await
    }
    pub async fn demand(&mut self, address: GraphAddress) -> Result<ResidencyStatus, StoreError> {
        if self.status(address) == ResidencyStatus::Absent {
            return Ok(ResidencyStatus::Absent);
        }
        let mut demands = self.demands.clone();
        demands.insert(address);
        self.update(self.roots.clone(), demands, self.policy, false)
            .await?;
        Ok(self.status(address))
    }
    pub async fn set_policy(&mut self, policy: ResidencyPolicy) -> Result<(), StoreError> {
        self.update(self.roots.clone(), self.demands.clone(), policy, true)
            .await
    }
    async fn update(
        &mut self,
        roots: ResidencyRoots,
        demands: BTreeSet<GraphAddress>,
        policy: ResidencyPolicy,
        persist: bool,
    ) -> Result<(), StoreError> {
        let (generation, catalog, _) = self.store.catalog().await?;
        let records = self.select(&catalog, &roots, &demands, policy).await?;
        let identical = generation == self.generation
            && records.keys().eq(self.records.keys())
            && policy == self.policy;
        // All candidate construction and validation precedes policy persistence.
        // A new source generation alone is not a change to available graph truth.
        let replacement = if identical {
            None
        } else {
            let (snapshot, facets, coverage, mut footprint) = read::materialize(
                &self.store,
                &catalog,
                &records,
                policy == ResidencyPolicy::Full,
            )
            .await?;
            Graph::try_from_recorded_snapshot(&snapshot).map_err(|e| fail(e.to_string()))?;
            let addresses = records.keys().copied().collect();
            let (pending, bytes) = pending::load(
                &self.store,
                &catalog.pending,
                &addresses,
                policy == ResidencyPolicy::Full || records.len() == catalog.entries.len(),
            )
            .await?;
            footprint.pending_bytes = bytes;
            let mut comparable = snapshot.clone();
            comparable.fields.sort_by(|a, b| a.id.cmp(&b.id));
            comparable.couplings.sort_by(|a, b| a.id.cmp(&b.id));
            let digest = hash(&encode(&(comparable, &facets))?);
            Some((snapshot, facets, coverage, footprint, pending, digest))
        };
        if persist {
            self.store.set_policy(policy).await?;
        }
        if let Some((snapshot, facets, coverage, footprint, pending, digest)) = replacement {
            let composed = self.composed_coverage(&coverage);
            if digest != self.available_digest {
                self.graph
                    .replace_loaded_recorded_snapshot(&snapshot, &facets, composed)
                    .map_err(|e| fail(e.to_string()))?;
            } else {
                self.graph.set_known_coverage(composed);
            }
            self.graph.restore_pending_link_state(pending);
            self.available_digest = digest;
            self.residency_coverage = coverage;
            self.footprint = footprint;
        }
        self.catalog = Some(catalog);
        self.generation = generation;
        self.records = records;
        self.roots = roots;
        self.demands = demands;
        self.policy = policy;
        Ok(())
    }
    async fn select(
        &self,
        catalog: &Catalog,
        roots: &ResidencyRoots,
        demands: &BTreeSet<GraphAddress>,
        policy: ResidencyPolicy,
    ) -> Result<BTreeMap<GraphAddress, StoredNode>, StoreError> {
        let mut selected = BTreeMap::new();
        let mut queue = VecDeque::new();
        let mut distances: BTreeMap<GraphAddress, u16> = BTreeMap::new();
        match policy {
            ResidencyPolicy::Full => {
                queue.extend(catalog.entries.iter().map(|e| (e.address, 0, false)))
            },
            ResidencyPolicy::Neighborhood { .. } => {
                queue.extend(
                    roots
                        .projection_surfaces
                        .iter()
                        .map(|id| (GraphAddress::Surface(*id), 0, true)),
                );
                queue.extend(
                    roots
                        .pinned
                        .iter()
                        .copied()
                        .chain(roots.focused)
                        .chain(demands.iter().copied())
                        .map(|a| (a, 0, false)),
                );
                queue.extend(
                    catalog
                        .entries
                        .iter()
                        .filter(|e| e.pinned || e.declaration)
                        .map(|e| (e.address, 0, false)),
                );
            },
        }
        // The walk flag makes projection neighborhoods distinct from individual
        // pins/focus/demand. Surface dependencies always follow shown bindings.
        let mut walked: BTreeMap<GraphAddress, u16> = BTreeMap::new();
        while let Some((address, distance, walk)) = queue.pop_front() {
            let row = catalog
                .row(address)
                .ok_or_else(|| fail(format!("requested graph item is absent: {address:?}")))?;
            if !selected.contains_key(&address) {
                let record = if let Some(old) = self
                    .catalog
                    .as_ref()
                    .and_then(|c| c.row(address))
                    .filter(|old| old.hash == row.hash)
                    .and_then(|_| self.records.get(&address))
                {
                    old.clone()
                } else {
                    self.store.node(row).await?
                };
                selected.insert(address, record);
            }
            if let GraphAddress::Surface(_) = address {
                if let Some(id) = row.shown {
                    if (walk
                        && !walked
                            .get(&GraphAddress::Resource(id))
                            .is_some_and(|old| *old <= distance))
                        || !distances.contains_key(&GraphAddress::Resource(id))
                    {
                        queue.push_back((GraphAddress::Resource(id), distance, walk));
                    }
                }
            }
            distances
                .entry(address)
                .and_modify(|d| *d = (*d).min(distance))
                .or_insert(distance);
            if !walk || walked.get(&address).is_some_and(|old| *old <= distance) {
                continue;
            }
            walked.insert(address, distance);
            if let GraphAddress::Resource(id) = address {
                for alias in catalog.entries.iter().filter(|e| e.shown == Some(id)) {
                    queue.push_back((alias.address, distance, true));
                }
            }
            let max = match policy {
                ResidencyPolicy::Full => 0,
                ResidencyPolicy::Neighborhood { hops } => hops,
            };
            if distance >= max {
                continue;
            }
            for (_, edge) in selected[&address].edges() {
                for id in [&edge.from_node_id, &edge.to_node_id] {
                    let id = Uuid::parse_str(id).map_err(|e| fail(e.to_string()))?;
                    let neighbor = match address {
                        GraphAddress::Surface(_) => GraphAddress::Surface(id),
                        GraphAddress::Resource(_) => GraphAddress::Resource(id),
                    };
                    if !walked
                        .get(&neighbor)
                        .is_some_and(|old| *old <= distance + 1)
                    {
                        queue.push_back((neighbor, distance + 1, true));
                    }
                }
            }
        }
        Ok(selected)
    }
}
