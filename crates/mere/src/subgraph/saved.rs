// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Checked saved-query creation, sharing, coverage and freeze.
use super::*;

impl SessionSubgraphs {
    /// Checked creation. Invalid queries/unsupported capabilities allocate no id.
    pub fn try_record_linked(
        &mut self,
        graph: &Graph,
        spec: SubgraphSpec,
    ) -> Result<SubgraphId, String> {
        let derived = self.derive_members_counted(graph, &spec)?;
        Ok(self.insert_linked(graph, spec, Ok(derived)))
    }

    pub(super) fn insert_linked(
        &mut self,
        graph: &Graph,
        spec: SubgraphSpec,
        derived: Result<DerivedMembers, String>,
    ) -> SubgraphId {
        let id = self.mint_id();
        let mut g = SubgraphRef::new_session(id);
        g.kind = Some(spec.kind.clone());
        g.primary_anchor = spec.primary_anchor.as_deref().and_then(|s| s.parse().ok());
        let observation = match derived {
            Ok(derived) => {
                g.anchors = derived.members;
                self.reconciled_revision.insert(id, graph.revision());
                DerivationObservation {
                    error: None,
                    missing_resources: derived.missing_resources,
                }
            },
            Err(error) => DerivationObservation {
                error: Some(error),
                missing_resources: vec![],
            },
        };
        g.binding = SubgraphBinding::Linked { spec };
        self.subgraphs.push(g);
        self.observations.insert(id, observation);
        id
    }

    /// Last derivation refusal. Legacy infallible entry points retain diagnostics
    /// here and never cache a refusal as a successful empty roster.
    pub fn derivation_error(&self, id: SubgraphId) -> Option<&str> {
        self.observations.get(&id)?.error.as_deref()
    }

    /// Refresh coverage from host observations without rerunning a roster walk.
    pub fn coverage(&self, graph: &Graph, id: SubgraphId) -> Option<CoverageNote> {
        self.get(id)?;
        Some(
            graph.coverage_for_resource_refs(
                self.observations
                    .get(&id)
                    .map(|o| o.missing_resources.as_slice())
                    .unwrap_or_default(),
            ),
        )
    }

    /// The portable sharing payload: its definition, without a local id or roster.
    pub fn shared_spec(&self, id: SubgraphId) -> Option<SubgraphSpec> {
        match &self.get(id)?.binding {
            SubgraphBinding::Linked { spec } => Some(spec.clone()),
            _ => None,
        }
    }

    /// Edit a Linked definition, invalidating its revision gate.
    pub fn set_linked_spec(&mut self, id: SubgraphId, spec: SubgraphSpec) -> bool {
        let Some(g) = self.subgraphs.iter_mut().find(|g| g.id == id) else {
            return false;
        };
        let SubgraphBinding::Linked { spec: current } = &mut g.binding else {
            return false;
        };
        if *current == spec {
            return false;
        }
        *current = spec.clone();
        g.kind = Some(spec.kind);
        g.primary_anchor = spec.primary_anchor.as_deref().and_then(|s| s.parse().ok());
        self.reconciled_revision.remove(&id);
        self.observations.remove(&id);
        true
    }

    /// Freeze a fresh evaluation, rather than a possibly stale local roster.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn freeze_linked(
        &self,
        graph: &mut Graph,
        id: SubgraphId,
    ) -> Result<GraphMemberId, String> {
        self.freeze_linked_with_nonce(graph, id, uuid::Uuid::new_v4())
    }

    /// Portable freeze with a fresh host-supplied nonce.
    pub fn freeze_linked_with_nonce(
        &self,
        graph: &mut Graph,
        id: SubgraphId,
        nonce: uuid::Uuid,
    ) -> Result<GraphMemberId, String> {
        let spec = self.shared_spec(id).ok_or("subgraph is not Linked")?;
        // Shape walks use Surface seeds. An unavailable explicit seed is a
        // refusal, even when the walk's compatibility API returns an empty set.
        if !matches!(spec.kind, SubgraphKind::Sparql { .. }) {
            if let Some(seed) = &spec.primary_anchor {
                let id = seed
                    .parse::<uuid::Uuid>()
                    .map_err(|_| "invalid shape Surface seed")?;
                if graph.get_node_key_by_id(id).is_none() {
                    return Err("shape Surface seed is unavailable".into());
                }
            } else if matches!(
                spec.kind,
                SubgraphKind::Component | SubgraphKind::Ego { .. }
            ) {
                return Err("shape requires a Surface seed".into());
            }
        }
        let derived = try_derive_members(graph, &spec)?;
        if !derived.missing_resources.is_empty() {
            return Err("cannot freeze unavailable query members".into());
        }
        let members: Result<Vec<_>, _> = derived
            .members
            .into_iter()
            .map(|id| {
                if matches!(spec.kind, SubgraphKind::Sparql { .. }) {
                    graph
                        .resource(id)
                        .map(|_| id)
                        .ok_or("query member Resource is unavailable")
                } else {
                    graph
                        .get_node_key_by_id(id)
                        .and_then(|key| graph.shown_resource_id(key))
                        .ok_or("member has no held Resource")
                }
            })
            .collect();
        graph.freeze_resource_selection_with_nonce(
            serde_json::to_value(spec).map_err(|e| e.to_string())?,
            members?,
            nonce,
        )
    }
}
