// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use std::collections::VecDeque;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefreshOutcome {
    Unchanged,
    SourceOnly,
    ObservationOnly,
    TruthChanged,
}

/// Owned inputs with private reader provenance. Loading holds no reader borrow.
pub struct ResidentRefreshRequest<B> {
    store: AddressableGraphStore<B>,
    inputs: RefreshInputs,
}
struct RefreshInputs {
    identity: Arc<()>,
    installation: u64,
    previous_generation: String,
    previous_catalog: Option<Catalog>,
    previous_records: BTreeMap<GraphAddress, StoredNode>,
    previous_roots: ResidencyRoots,
    previous_demands: BTreeSet<GraphAddress>,
    previous_policy: ResidencyPolicy,
    roots: ResidencyRoots,
    demands: BTreeSet<GraphAddress>,
    policy: ResidencyPolicy,
}
/// Validated bytes can only be installed by the reader that requested them,
/// before any intervening installation. Host context is deliberately excluded.
pub struct ResidentRefreshCandidate {
    inputs: RefreshInputs,
    generation: String,
    replacement: Option<Replacement>,
}
struct Replacement {
    catalog: Catalog,
    records: BTreeMap<GraphAddress, StoredNode>,
    snapshot: kernel::persistence::GraphSnapshot,
    facets: NodeFacetStore,
    coverage: kernel::graph::CoverageNote,
    projection_coverage: kernel::graph::CoverageNote,
    footprint: ResidentFootprint,
    pending: kernel::graph::PendingLinkState,
    available_digest: String,
}
impl<B: Backend> ResidentRefreshRequest<B> {
    pub fn with_demand(mut self, address: GraphAddress) -> Self {
        self.inputs.demands.insert(address);
        self
    }
    pub async fn load(self) -> Result<ResidentRefreshCandidate, StoreError> {
        self.inputs.load(&self.store).await
    }
}
impl<B: Backend + Clone> ResidentGraph<B> {
    pub fn refresh_request(
        &self,
        roots: ResidencyRoots,
        policy: ResidencyPolicy,
    ) -> ResidentRefreshRequest<B> {
        ResidentRefreshRequest {
            store: AddressableGraphStore::new(self.store.backend.clone(), self.store.graph_id),
            inputs: self.refresh_inputs(roots, self.demands.clone(), policy),
        }
    }
    pub fn demand_request(&self, address: GraphAddress) -> ResidentRefreshRequest<B> {
        let mut request = self.refresh_request(self.roots.clone(), self.policy);
        request.inputs.demands.insert(address);
        request
    }
}
impl<B: Backend> ResidentGraph<B> {
    fn refresh_inputs(
        &self,
        roots: ResidencyRoots,
        demands: BTreeSet<GraphAddress>,
        policy: ResidencyPolicy,
    ) -> RefreshInputs {
        RefreshInputs {
            identity: self.identity.clone(),
            installation: self.installation,
            previous_generation: self.generation.clone(),
            previous_catalog: self.catalog.clone(),
            previous_records: self.records.clone(),
            previous_roots: self.roots.clone(),
            previous_demands: self.demands.clone(),
            previous_policy: self.policy,
            roots,
            demands,
            policy,
        }
    }
    pub async fn prepare_refresh(
        &self,
        roots: ResidencyRoots,
        policy: ResidencyPolicy,
    ) -> Result<ResidentRefreshCandidate, StoreError> {
        self.refresh_inputs(roots, self.demands.clone(), policy)
            .load(&self.store)
            .await
    }
    pub async fn refresh_if_changed(&mut self) -> Result<RefreshOutcome, StoreError> {
        let candidate = self
            .prepare_refresh(self.roots.clone(), self.policy)
            .await?;
        self.install_refresh(candidate)
    }
    pub async fn reconcile(&mut self, roots: ResidencyRoots) -> Result<(), StoreError> {
        self.reconcile_with_policy(roots, self.policy).await
    }
    pub async fn reconcile_with_policy(
        &mut self,
        roots: ResidencyRoots,
        policy: ResidencyPolicy,
    ) -> Result<(), StoreError> {
        let candidate = self
            .refresh_inputs(roots, BTreeSet::new(), policy)
            .load(&self.store)
            .await?;
        self.install_refresh(candidate).map(|_| ())
    }
    pub async fn refresh(&mut self) -> Result<(), StoreError> {
        self.refresh_if_changed().await.map(|_| ())
    }
    pub async fn demand(&mut self, address: GraphAddress) -> Result<ResidencyStatus, StoreError> {
        if self.status(address) == ResidencyStatus::Absent {
            return Ok(ResidencyStatus::Absent);
        }
        let mut demands = self.demands.clone();
        demands.insert(address);
        let candidate = self
            .refresh_inputs(self.roots.clone(), demands, self.policy)
            .load(&self.store)
            .await?;
        self.install_refresh(candidate)?;
        Ok(self.status(address))
    }
    pub async fn set_policy(&mut self, policy: ResidencyPolicy) -> Result<(), StoreError> {
        let candidate = self.prepare_refresh(self.roots.clone(), policy).await?;
        self.store.set_policy(policy).await?;
        self.install_refresh(candidate).map(|_| ())
    }
    pub fn install_refresh(
        &mut self,
        candidate: ResidentRefreshCandidate,
    ) -> Result<RefreshOutcome, StoreError> {
        if !Arc::ptr_eq(&self.identity, &candidate.inputs.identity)
            || self.installation != candidate.inputs.installation
        {
            return Err(fail(
                "refresh candidate belongs to a different or changed reader",
            ));
        }
        let previous_observation = self.graph.semantic_observation_revision();
        let source_changed = self.generation != candidate.generation;
        let mut outcome = if source_changed {
            RefreshOutcome::SourceOnly
        } else {
            RefreshOutcome::Unchanged
        };
        if let Some(next) = candidate.replacement {
            let mut composed = next.coverage.clone();
            for limit in self
                .host_coverage
                .limits
                .iter()
                .chain(&next.projection_coverage.limits)
            {
                composed.push(limit.clone());
            }
            if next.available_digest != self.available_digest {
                // This operation validates its replacement before touching self.
                self.graph
                    .replace_loaded_recorded_snapshot(&next.snapshot, &next.facets, composed)
                    .map_err(|e| fail(e.to_string()))?;
                outcome = RefreshOutcome::TruthChanged;
            } else {
                self.graph.set_known_coverage(composed);
            }
            self.graph.restore_pending_link_state(next.pending);
            self.catalog = Some(next.catalog);
            self.records = next.records;
            self.available_digest = next.available_digest;
            self.residency_coverage = next.coverage;
            self.projection_coverage = next.projection_coverage;
            self.footprint = next.footprint;
        }
        if outcome != RefreshOutcome::TruthChanged
            && self.graph.semantic_observation_revision() != previous_observation
        {
            outcome = RefreshOutcome::ObservationOnly;
        }
        self.generation = candidate.generation;
        self.roots = candidate.inputs.roots;
        self.demands = candidate.inputs.demands;
        self.policy = candidate.inputs.policy;
        self.installation = self.installation.wrapping_add(1);
        Ok(outcome)
    }
}
impl RefreshInputs {
    async fn load<B: Backend>(
        self,
        store: &AddressableGraphStore<B>,
    ) -> Result<ResidentRefreshCandidate, StoreError> {
        let generation = store.head().await?;
        if generation == self.previous_generation
            && self.roots == self.previous_roots
            && self.demands == self.previous_demands
            && self.policy == self.previous_policy
        {
            return Ok(ResidentRefreshCandidate {
                inputs: self,
                generation,
                replacement: None,
            });
        }
        let catalog = if generation == self.previous_generation {
            self.previous_catalog
                .clone()
                .ok_or_else(|| fail("missing previous catalog"))?
        } else {
            store.catalog_at(&generation).await?.0
        };
        if self
            .demands
            .iter()
            .any(|address| catalog.row(*address).is_none())
        {
            return Err(fail("demanded graph item is absent"));
        }
        let records = self
            .select(store, &catalog, &self.roots, &self.demands, self.policy)
            .await?;
        let (snapshot, facets, coverage, mut footprint) = read::materialize(
            store,
            &catalog,
            &records,
            self.policy == ResidencyPolicy::Full,
        )
        .await?;
        Graph::try_from_recorded_snapshot(&snapshot).map_err(|e| fail(e.to_string()))?;
        let addresses = records.keys().copied().collect();
        let (pending, bytes) = pending::load(
            store,
            &catalog.pending,
            &addresses,
            self.policy == ResidencyPolicy::Full || records.len() == catalog.entries.len(),
        )
        .await?;
        footprint.pending_bytes = bytes;
        let mut comparable = snapshot.clone();
        comparable.fields.sort_by(|a, b| a.id.cmp(&b.id));
        comparable.couplings.sort_by(|a, b| a.id.cmp(&b.id));
        let available_digest = hash(&encode(&(comparable, &facets))?);
        let mut projection_coverage = kernel::graph::CoverageNote::default();
        let absent: Vec<_> = self
            .roots
            .projection_surfaces
            .iter()
            .copied()
            .filter(|id| catalog.row(GraphAddress::Surface(*id)).is_none())
            .collect();
        if !absent.is_empty() {
            let mut limit = kernel::graph::CoverageLimit::new(
                kernel::graph::CoverageLayer::Projection,
                "authored projection members are absent from the catalog",
            );
            limit.count = Some(absent.len());
            limit.surfaces = absent;
            projection_coverage.push(limit);
        }
        Ok(ResidentRefreshCandidate {
            inputs: self,
            generation,
            replacement: Some(Replacement {
                catalog,
                records,
                snapshot,
                facets,
                coverage,
                projection_coverage,
                footprint,
                pending,
                available_digest,
            }),
        })
    }
    async fn select<B: Backend>(
        &self,
        store: &AddressableGraphStore<B>,
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
                        .filter(|id| catalog.row(GraphAddress::Surface(**id)).is_some())
                        .map(|id| (GraphAddress::Surface(*id), 0, true)),
                );
                queue.extend(
                    roots
                        .pinned
                        .iter()
                        .copied()
                        .chain(roots.focused)
                        .chain(roots.transient.iter().copied())
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
                    .previous_catalog
                    .as_ref()
                    .and_then(|c| c.row(address))
                    .filter(|old| old.hash == row.hash)
                    .and_then(|_| self.previous_records.get(&address))
                {
                    old.clone()
                } else {
                    store.node(row).await?
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
