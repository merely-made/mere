// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Addressable recorded graph materializations over the existing Muniment backend.
use kernel::graph::Graph;
use muniment::{Backend, StoreError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum GraphAddress {
    Surface(Uuid),
    Resource(Uuid),
}
impl GraphAddress {
    pub fn id(self) -> Uuid {
        match self {
            Self::Surface(id) | Self::Resource(id) => id,
        }
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ResidencyPolicy {
    #[default]
    Full,
    Neighborhood {
        hops: u16,
    },
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ResidencyRoots {
    pub projection_surfaces: BTreeSet<Uuid>,
    pub pinned: BTreeSet<GraphAddress>,
    pub focused: Option<GraphAddress>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResidencyStatus {
    Loaded,
    NotLoaded,
    Absent,
}
#[derive(Clone, Debug, Default)]
pub struct ResidentFootprint {
    pub surfaces: usize,
    pub resources: usize,
    pub catalog_bytes: usize,
    pub record_bytes: usize,
    pub navigation_bytes: usize,
}
impl ResidentFootprint {
    pub fn retained_record_bytes(&self) -> usize {
        self.catalog_bytes + self.record_bytes + self.navigation_bytes
    }
}

mod navigation;
mod read;
mod residency;
mod schema;
mod write;
use kernel::graph::NodeFacetStore;
use schema::*;
use std::collections::BTreeMap;

pub struct AddressableGraphStore<B> {
    backend: B,
    graph_id: Uuid,
}
impl<B: Backend> AddressableGraphStore<B> {
    pub fn new(backend: B, graph_id: Uuid) -> Self {
        Self { backend, graph_id }
    }
    fn key(&self, suffix: &str) -> String {
        format!("graphs/{}/resident/v1/{suffix}", self.graph_id)
    }
    pub async fn set_policy(&self, policy: ResidencyPolicy) -> Result<(), StoreError> {
        self.backend
            .put(&self.key("policy"), &encode(&policy)?)
            .await
    }
    pub async fn open(self, roots: ResidencyRoots) -> Result<ResidentGraph<B>, StoreError> {
        let policy = match self.backend.get(&self.key("policy")).await? {
            Some(bytes) => decode(&bytes)?,
            None => ResidencyPolicy::Full,
        };
        let mut view = ResidentGraph {
            store: self,
            graph: Graph::new(),
            generation: String::new(),
            catalog: None,
            records: BTreeMap::new(),
            roots: Default::default(),
            demands: BTreeSet::new(),
            policy,
            footprint: Default::default(),
        };
        view.reconcile(roots).await?;
        Ok(view)
    }
}
/// A read-only available-data view. Editing/checkpointing uses the recorded session.
pub struct ResidentGraph<B> {
    store: AddressableGraphStore<B>,
    graph: Graph,
    generation: String,
    catalog: Option<Catalog>,
    records: BTreeMap<GraphAddress, StoredNode>,
    roots: ResidencyRoots,
    demands: BTreeSet<GraphAddress>,
    policy: ResidencyPolicy,
    footprint: ResidentFootprint,
}
impl<B: Backend> ResidentGraph<B> {
    pub fn graph(&self) -> &Graph {
        &self.graph
    }
    pub fn generation(&self) -> &str {
        &self.generation
    }
    pub fn policy(&self) -> ResidencyPolicy {
        self.policy
    }
    pub fn status(&self, address: GraphAddress) -> ResidencyStatus {
        if self.records.contains_key(&address) {
            ResidencyStatus::Loaded
        } else if self
            .catalog
            .as_ref()
            .is_some_and(|c| c.entries.iter().any(|e| e.address == address))
        {
            ResidencyStatus::NotLoaded
        } else {
            ResidencyStatus::Absent
        }
    }
    pub fn footprint(&self) -> ResidentFootprint {
        self.footprint.clone()
    }
}
