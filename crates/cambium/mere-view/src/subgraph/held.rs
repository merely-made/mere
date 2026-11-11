// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The indexed query dataset a [`super::SessionSubgraphs`] holds across its
//! reconcile paths, so saved SPARQL subgraphs at one graph revision share one build.
use std::sync::{Mutex, PoisonError};

use incipit::forme::{SubgraphKind, SubgraphSpec};
use kernel::graph::Graph;
use linked_data::query::{IndexedDataset, sparql_resource_members_on};

use super::DerivedMembers;

/// A cache, not content: never serialized, cloned as empty.
#[derive(Default)]
pub(super) struct HeldDataset {
    dataset: Mutex<Option<IndexedDataset>>,
    /// Test-only observable: how many times the dataset was built.
    #[cfg(test)]
    pub(super) builds: std::sync::atomic::AtomicUsize,
}

impl Clone for HeldDataset {
    fn clone(&self) -> Self {
        Self::default()
    }
}

impl std::fmt::Debug for HeldDataset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("HeldDataset(..)")
    }
}

impl HeldDataset {
    /// Derive a `Sparql` spec's members over the held dataset, rebuilding it
    /// when it is not current for `graph`. `None` for any other kind.
    pub(super) fn derive(
        &self,
        graph: &Graph,
        spec: &SubgraphSpec,
    ) -> Option<Result<DerivedMembers, String>> {
        let SubgraphKind::Sparql {
            query,
            member_variable,
        } = &spec.kind
        else {
            return None;
        };
        let mut held = self.dataset.lock().unwrap_or_else(PoisonError::into_inner);
        if !held.as_ref().is_some_and(|d| d.is_current(graph)) {
            #[cfg(test)]
            self.builds
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            *held = Some(IndexedDataset::for_graph(graph));
        }
        let dataset = held.as_ref()?;
        Some(
            sparql_resource_members_on(dataset, graph, query, member_variable).map(|r| {
                DerivedMembers {
                    members: r.members,
                    missing_resources: r.missing_resources,
                    coverage: r.coverage,
                }
            }),
        )
    }
}
