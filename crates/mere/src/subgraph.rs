// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Per-session subgraph derivation and persistence for Mere's graph family.
//!
//! This crate owns the pure half of the subgraph seam: the per-session subgraph
//! index, linked-subgraph derivation/reconciliation, and the shape classifier in
//! [`classifier`]. SPARQL saved specs use the opt-in `query` capability.
//!
//! A `SubgraphKind::Sparql` carries SELECT text and the variable whose named
//! IRI cells identify member Resources. [`try_derive_members`] returns coverage;
//! [`SessionSubgraphs::try_record_linked`] and [`SessionSubgraphs::try_reconcile`]
//! retain the local roster and report refusals. Sharing uses
//! [`SessionSubgraphs::shared_spec`] only, so a receiver derives its own members.
//! A new Resource from [`SessionSubgraphs::freeze_linked_with_nonce`] bears an
//! immutable nested graph of member references and exact relation copies. Its
//! record travels separately from the local subgraph index; ordinary Resource
//! annotations and Surfaces refer to that owner. Opening it with
//! [`Graph::open_frozen_selection`] reports missing references through coverage.
//! The portable freeze takes a fresh host UUID; native hosts also have
//! `freeze_linked`. Old shape wire formats and infallible calls remain supported;
//! new saved-query consumers should use the checked paths.

use std::collections::{HashMap, HashSet};
use std::path::Path;

use forme::{
    GraphMemberId, SubgraphBinding, SubgraphId, SubgraphKind, SubgraphMemberDelta, SubgraphRef,
    SubgraphSpec,
};
use kernel::graph::{CoverageNote, EdgeFamily, Graph, RelationSelector};
use serde::{Deserialize, Serialize};

pub mod classifier;
mod saved;

/// Crate version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Lifecycle stage marker.
pub const STAGE: &str = "pre-alpha";

/// Sidecar filename for a session's subgraph index, beside `graph.json`.
pub const SUBGRAPHS_FILE: &str = "subgraphs.json";

/// The subgraphs belonging to one session's graph: named sub-structures (a tear-out
/// **branch**, later document-groups / relational-browse neighborhoods) over the same
/// kernel nodes the orrery + workbench render. One per session, persisted beside the
/// graph. (Subgraph-wiring Phase 1; plan recommendation B.)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SessionSubgraphs {
    subgraphs: Vec<SubgraphRef<GraphMemberId>>,
    /// Monotonic id source. `SubgraphId` is forme's lightweight `u32` index, unique
    /// within this session's index (not globally).
    next_id: SubgraphId,
    /// The kernel [`Graph::revision`] each `Linked` subgraph's roster was last derived
    /// against. A **cache key, not content**: it is never persisted (`serde(skip)`), so a
    /// freshly loaded index has no entries and every Linked subgraph reads as stale until
    /// its first reconcile. An entry is dropped whenever something other than the graph
    /// changes what the derivation would produce (a spec edit, a roster edit, a rebind).
    #[serde(skip)]
    reconciled_revision: HashMap<SubgraphId, u64>,
    #[serde(skip)]
    observations: HashMap<SubgraphId, DerivationObservation>,
    /// Test-only observable: how many times [`derive_members`] has run through this
    /// index's reconcile paths, so tests can assert the revision gate skipped the walk.
    #[cfg(test)]
    #[serde(skip)]
    derive_calls: std::cell::Cell<usize>,
}

#[derive(Clone, Debug, Default)]
struct DerivationObservation {
    error: Option<String>,
    missing_resources: Vec<GraphMemberId>,
}

/// One derivation, with coverage separate from the local member cache.
#[derive(Clone, Debug)]
pub struct DerivedMembers {
    /// Surface UUIDs for shape walks; Resource UUIDs for SPARQL selections.
    /// Resolve through the spec kind: UUID values can overlap across strata.
    pub members: Vec<GraphMemberId>,
    pub missing_resources: Vec<GraphMemberId>,
    pub coverage: CoverageNote,
}

impl SessionSubgraphs {
    /// An empty index. A freshly-created session seeds one default `Session`-kind
    /// subgraph via [`with_default_session`](Self::with_default_session).
    pub fn new() -> Self {
        Self::default()
    }

    /// The empty index plus the default whole-session subgraph (the grouping every
    /// session starts with, before any branch). The seed for a new session.
    pub fn with_default_session(mut self) -> Self {
        let id = self.mint_id();
        self.subgraphs.push(SubgraphRef::new_session(id));
        self
    }

    /// The subgraphs in this session, in creation order.
    pub fn subgraphs(&self) -> &[SubgraphRef<GraphMemberId>] {
        &self.subgraphs
    }

    /// Look a subgraph up by id.
    pub fn get(&self, id: SubgraphId) -> Option<&SubgraphRef<GraphMemberId>> {
        self.subgraphs.iter().find(|g| g.id == id)
    }

    /// Preview a Linked subgraph's drift without mutating its roster. This powers
    /// the Roster Subgraph Card's dry diff; [`reconcile`](Self::reconcile) uses
    /// the same derivation and then applies the truth set. Subject to the same
    /// revision gate: `None` when the graph has not changed since the last reconcile.
    pub fn preview_reconcile(
        &self,
        graph: &Graph,
        id: SubgraphId,
    ) -> Option<SubgraphMemberDelta<GraphMemberId>> {
        self.reconcile_delta(graph, id)
            .ok()
            .flatten()
            .and_then(|(_, delta)| (!delta.is_empty()).then_some(delta))
    }

    /// The kernel revision subgraph `id`'s roster was last derived against, or `None`
    /// when it is stale (never reconciled, loaded from disk, or invalidated by an edit).
    pub fn reconciled_revision(&self, id: SubgraphId) -> Option<u64> {
        self.reconciled_revision.get(&id).copied()
    }

    /// Forget every recorded reconcile revision, so the next [`reconcile`](Self::reconcile)
    /// / [`reconcile_all`](Self::reconcile_all) re-derives every Linked subgraph in full.
    /// For callers that swapped the underlying graph (a revision counter is per `Graph`
    /// instance) or otherwise cannot trust the gate.
    pub fn clear_reconciled_revisions(&mut self) {
        self.reconciled_revision.clear();
    }

    /// [`reconcile_all`](Self::reconcile_all) with the revision gate bypassed: every
    /// Linked subgraph is re-derived and diffed regardless of the recorded revision.
    pub fn force_reconcile_all(&mut self, graph: &Graph) -> bool {
        self.clear_reconciled_revisions();
        self.reconcile_all(graph)
    }

    fn derive_members_counted(
        &self,
        graph: &Graph,
        spec: &SubgraphSpec,
    ) -> Result<DerivedMembers, String> {
        #[cfg(test)]
        self.derive_calls.set(self.derive_calls.get() + 1);
        try_derive_members(graph, spec)
    }

    fn mint_id(&mut self) -> SubgraphId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Record a tear-out **branch** (G3): a new subgraph anchored on the torn node,
    /// bound `Branched` back to the donor's spec. Returns the new `SubgraphId` the torn
    /// window carries. The branch and the donor share kernel nodes and diverge in this
    /// subgraph's lineage (brief §4.2).
    ///
    /// `parent_spec` is the donor subgraph's spec if it had one, else a default derived
    /// from the anchor (see [`default_spec_for`]).
    pub fn record_branch(
        &mut self,
        anchor: GraphMemberId,
        parent_spec: SubgraphSpec,
    ) -> SubgraphId {
        let id = self.mint_id();
        let mut subgraph = SubgraphRef::new_session(id).with_anchor(anchor);
        subgraph.binding = SubgraphBinding::Branched {
            parent_spec,
            reason: "tearout-branch".to_string(),
        };
        self.subgraphs.push(subgraph);
        id
    }

    /// Add `node` to subgraph `id`'s roster, returning whether it was newly added (not
    /// already present). We reuse forme's `anchors` list as the live member set, since
    /// the superseded `GraphTree` member map is not in play (Phase 3 may distinguish
    /// seed anchors from a derived roster). A tear-out **branch** grows this as its
    /// window navigates, so it diverges from the donor. (Subgraph wiring Phase 2.)
    pub fn add_member(&mut self, id: SubgraphId, node: GraphMemberId) -> bool {
        if let Some(g) = self.subgraphs.iter_mut().find(|g| g.id == id)
            && !g.anchors.contains(&node)
        {
            g.anchors.push(node);
            // A hand edit to the roster is drift the gate cannot see.
            self.reconciled_revision.remove(&id);
            return true;
        }
        false
    }

    /// Record a **Linked** subgraph (Phase 3): one whose roster is *derived* from the
    /// graph by its [`SubgraphSpec`] (kind + seed), not hand-built like a branch. The
    /// initial roster is derived now; [`reconcile`](Self::reconcile) re-derives it when the
    /// graph drifts. `anchors` holds the live derived set; `spec.primary_anchor` holds the
    /// seed (the anchors-vs-members split). Returns the new id. (Subgraph wiring Phase 3.)
    pub fn record_linked(&mut self, graph: &Graph, spec: SubgraphSpec) -> SubgraphId {
        let derived = self.derive_members_counted(graph, &spec);
        self.insert_linked(graph, spec, derived)
    }

    /// Freeze a multi-selection as a **Session** subgraph (the 2026-06-13 crystallize default): an
    /// `UnlinkedSession` subgraph whose roster is exactly `members`, tagged with the classifier's
    /// `kind` for display. Unlike a Linked subgraph it does not derive or drift — it is the frozen
    /// selection, so it works for any shape (including the disconnected Loose / Session grab-bag the
    /// kind-derivation cannot). Returns the new id. (Swatch primitive — P3b crystallize.)
    pub fn record_session(
        &mut self,
        kind: SubgraphKind,
        members: Vec<GraphMemberId>,
    ) -> SubgraphId {
        let id = self.mint_id();
        let mut g = SubgraphRef::new_session(id);
        g.kind = Some(kind);
        g.anchors = members;
        self.subgraphs.push(g);
        id
    }

    /// Re-derive a **Linked** subgraph `id` from the current graph and reconcile its live
    /// roster to graph truth, returning the delta (added / removed members) or `None` when
    /// the subgraph is not Linked or nothing drifted. v0 **auto-applies** (the live set
    /// tracks truth); the user-choice proposal path (keep-linked / unlink / save-as-branch)
    /// is a later sub-slice. The diff is the harvested `compute_roster_delta` over the
    /// kernel-derived truth and the stored roster — no `GraphTree`. (Subgraph wiring P3.)
    ///
    /// **Revision gate:** when `graph.revision()` equals the revision this subgraph was
    /// last reconciled against, the derivation is skipped and `None` is returned (the
    /// roster already equals truth). Otherwise the walk runs, the roster is applied, and
    /// the new revision is recorded. See [`force_reconcile_all`](Self::force_reconcile_all).
    pub fn reconcile(
        &mut self,
        graph: &Graph,
        id: SubgraphId,
    ) -> Option<SubgraphMemberDelta<GraphMemberId>> {
        self.try_reconcile(graph, id).ok().flatten()
    }

    /// Checked reconciliation. Refusals preserve the prior roster and gate.
    pub fn try_reconcile(
        &mut self,
        graph: &Graph,
        id: SubgraphId,
    ) -> Result<Option<SubgraphMemberDelta<GraphMemberId>>, String> {
        let derived = match self.reconcile_delta(graph, id) {
            Ok(derived) => derived,
            Err(error) => {
                self.observations.entry(id).or_default().error = Some(error.clone());
                return Err(error);
            },
        };
        let Some((truth, delta)) = derived else {
            return Ok(None);
        };
        self.reconciled_revision.insert(id, graph.revision());
        self.observations.insert(
            id,
            DerivationObservation {
                error: None,
                missing_resources: truth.missing_resources,
            },
        );
        if let Some(g) = self.subgraphs.iter_mut().find(|g| g.id == id) {
            g.anchors = truth.members;
        }
        Ok((!delta.is_empty()).then_some(delta))
    }

    fn reconcile_delta(
        &self,
        graph: &Graph,
        id: SubgraphId,
    ) -> Result<Option<(DerivedMembers, SubgraphMemberDelta<GraphMemberId>)>, String> {
        let Some(g) = self.get(id) else {
            return Ok(None);
        };
        let SubgraphBinding::Linked { spec } = &g.binding else {
            return Ok(None);
        };
        if self.reconciled_revision.get(&id) == Some(&graph.revision()) {
            return Ok(None);
        }
        let truth = self.derive_members_counted(graph, spec)?;
        let truth_set: HashSet<_> = truth.members.iter().collect();
        let current_set: HashSet<_> = g.anchors.iter().collect();
        let delta = SubgraphMemberDelta {
            added: truth
                .members
                .iter()
                .filter(|m| !current_set.contains(m))
                .copied()
                .collect(),
            removed: g
                .anchors
                .iter()
                .filter(|m| !truth_set.contains(m))
                .copied()
                .collect(),
            rebased_seeds: vec![],
        };
        Ok(Some((truth, delta)))
    }

    /// Convert a linked/branched subgraph to an unlinked session grouping without
    /// changing its current member roster.
    pub fn keep_as_session(&mut self, id: SubgraphId) -> bool {
        let Some(g) = self.subgraphs.iter_mut().find(|g| g.id == id) else {
            return false;
        };
        if matches!(g.binding, SubgraphBinding::UnlinkedSession) {
            return false;
        }
        g.binding = SubgraphBinding::UnlinkedSession;
        self.reconciled_revision.remove(&id);
        true
    }

    /// Toggle a relation-family selector on a **Linked** subgraph's spec (Subgraph
    /// Card selector/family editing): adds the family projection string if absent,
    /// removes it if present. Only `Linked` carries a spec whose selectors drive
    /// live re-derivation (`reconcile` / `preview_reconcile`); a no-op elsewhere
    /// (`Branched`'s `parent_spec` is lineage, not an active derivation rule).
    /// Returns whether the toggle applied.
    pub fn toggle_family_selector(&mut self, id: SubgraphId, family: EdgeFamily) -> bool {
        let Some(g) = self.subgraphs.iter_mut().find(|g| g.id == id) else {
            return false;
        };
        let SubgraphBinding::Linked { spec } = &mut g.binding else {
            return false;
        };
        let name = edge_family_str(family);
        match spec
            .selectors
            .iter()
            .position(|s| s.eq_ignore_ascii_case(name))
        {
            Some(pos) => {
                spec.selectors.remove(pos);
            },
            None => spec.selectors.push(name.to_string()),
        }
        // The spec changed, so the derivation changes without the graph moving.
        self.reconciled_revision.remove(&id);
        true
    }

    /// Branch an existing subgraph's current seed/roster into a new local subgraph.
    /// The parent spec is preserved when present; otherwise we derive a default spec
    /// from the first available member.
    pub fn branch_from_subgraph(&mut self, id: SubgraphId) -> Option<SubgraphId> {
        let parent = self.subgraphs.iter().find(|g| g.id == id)?.clone();
        let anchor = parent
            .primary_anchor
            .or_else(|| parent.anchors.first().copied())?;
        let parent_spec = match parent.binding {
            SubgraphBinding::Linked { spec } => spec,
            SubgraphBinding::Branched { parent_spec, .. } => parent_spec,
            SubgraphBinding::UnlinkedSession => default_spec_for(anchor),
        };
        Some(self.record_branch(anchor, parent_spec))
    }

    /// Whether any subgraph is `Linked` (so worth reconciling against graph truth). The
    /// host gates the per-save reconcile on this. (Subgraph wiring Phase 3 slice 2+.)
    pub fn has_linked(&self) -> bool {
        self.subgraphs
            .iter()
            .any(|g| matches!(g.binding, SubgraphBinding::Linked { .. }))
    }

    /// Reconcile every `Linked` subgraph against the current graph (re-derive + auto-apply
    /// each), returning whether any roster changed (so the caller persists). Drift at the
    /// data level: a Linked subgraph's persisted roster tracks the graph (the window
    /// already tracks it live via re-derive). (Subgraph wiring Phase 3 slice 2+.)
    ///
    /// Revision-gated per subgraph (see [`reconcile`](Self::reconcile)): a subgraph whose
    /// recorded revision equals `graph.revision()` is skipped without a walk. Use
    /// [`force_reconcile_all`](Self::force_reconcile_all) to bypass the gate.
    pub fn reconcile_all(&mut self, graph: &Graph) -> bool {
        let ids: Vec<SubgraphId> = self
            .subgraphs
            .iter()
            .filter(|g| matches!(g.binding, SubgraphBinding::Linked { .. }))
            .map(|g| g.id)
            .collect();
        let mut changed = false;
        for id in ids {
            if self.reconcile(graph, id).is_some() {
                changed = true;
            }
        }
        changed
    }

    /// Persist this index to `subgraphs.json` in the session dir. Best-effort, like the
    /// other per-session sidecars (graph / workbench / cartography).
    pub fn save(&self, session_dir: &Path) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        std::fs::write(session_dir.join(SUBGRAPHS_FILE), json)
    }

    /// Load a session's subgraph index from its sidecar, falling back to a fresh
    /// default-session index on a missing or corrupt file (the same forgiving posture
    /// as sibling persisted graph sidecars.
    pub fn load(session_dir: &Path) -> Self {
        std::fs::read_to_string(session_dir.join(SUBGRAPHS_FILE))
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| Self::new().with_default_session())
    }
}

/// Derive a Linked subgraph's member set from the graph per its [`SubgraphSpec`]
/// (Phase 3). The kind selects the kernel-graph algorithm; the seed is `primary_anchor`;
/// the spec's `selectors` are the **edge projection** (which relation families the walk
/// follows — empty = all). `GraphTree`-free — the derivation lives on the kernel graph
/// (where the primitives + the relation vocabulary are), forme supplies only the spec type.
/// Corridor / Loop / Frontier / Facet kinds are later sub-slices; they fall back to the
/// seed alone for now.
pub fn derive_members(graph: &Graph, spec: &SubgraphSpec) -> Vec<GraphMemberId> {
    // Compatibility for old shape callers. Saved-query consumers should use the
    // checked API or SessionSubgraphs, which retains errors and coverage.
    try_derive_members(graph, spec)
        .map(|result| result.members)
        .unwrap_or_default()
}

/// Checked shape/query derivation. A saved query is evaluated only with `query`.
pub fn try_derive_members(graph: &Graph, spec: &SubgraphSpec) -> Result<DerivedMembers, String> {
    if let SubgraphKind::Sparql {
        query,
        member_variable,
    } = &spec.kind
    {
        #[cfg(feature = "query")]
        {
            let result =
                linked_data::query::sparql_resource_members(graph, query, member_variable)?;
            return Ok(DerivedMembers {
                members: result.members,
                missing_resources: result.missing_resources,
                coverage: result.coverage,
            });
        }
        #[cfg(not(feature = "query"))]
        {
            let _ = (query, member_variable);
            return Err("saved SPARQL queries require Mere's query capability".into());
        }
    }
    let members = if let Some(seed) = spec.primary_anchor.as_deref().and_then(|s| s.parse().ok()) {
        let selectors = selectors_from_spec(spec);
        match spec.kind {
            SubgraphKind::Component => graph.component_members(seed, &selectors),
            SubgraphKind::Ego { radius } => graph.ego_members(seed, radius, &selectors),
            _ => vec![seed],
        }
    } else {
        vec![]
    };
    Ok(DerivedMembers {
        members,
        missing_resources: vec![],
        coverage: graph.coverage_note(),
    })
}

/// Map a spec's opaque `selectors` strings to kernel [`RelationSelector`]s (the edge
/// projection). A family name (`Semantic` / `Traversal` / `Containment` / `Provenance` /
/// `Imported` / `Arrangement`, case-insensitive) becomes a `Family` selector; unknown
/// strings are skipped. Empty (or all-unknown) → no projection → the derivation follows
/// every family. Sub-kind selectors (e.g. only `Cites`) are a later refinement. (Subgraph
/// derivation — selectors.)
fn selectors_from_spec(spec: &SubgraphSpec) -> Vec<RelationSelector> {
    spec.selectors
        .iter()
        .filter_map(|s| edge_family_from_str(s).map(RelationSelector::Family))
        .collect()
}

/// Parse a relation-family name to an [`EdgeFamily`] (the projection-toggle vocabulary).
fn edge_family_from_str(name: &str) -> Option<EdgeFamily> {
    Some(match name.to_ascii_lowercase().as_str() {
        "semantic" => EdgeFamily::Semantic,
        "traversal" => EdgeFamily::Traversal,
        "containment" => EdgeFamily::Containment,
        "arrangement" => EdgeFamily::Arrangement,
        "imported" => EdgeFamily::Imported,
        "provenance" => EdgeFamily::Provenance,
        _ => return None,
    })
}

/// Format a relation family to the opaque selector-string vocabulary
/// [`edge_family_from_str`] parses — its inverse.
fn edge_family_str(family: EdgeFamily) -> &'static str {
    match family {
        EdgeFamily::Semantic => "semantic",
        EdgeFamily::Traversal => "traversal",
        EdgeFamily::Containment => "containment",
        EdgeFamily::Arrangement => "arrangement",
        EdgeFamily::Imported => "imported",
        EdgeFamily::Provenance => "provenance",
    }
}

/// The relation families a Linked subgraph's spec can filter its derivation walk to
/// (Subgraph Card selector/family editing chips).
pub const EDGE_FAMILIES: [EdgeFamily; 6] = [
    EdgeFamily::Semantic,
    EdgeFamily::Traversal,
    EdgeFamily::Containment,
    EdgeFamily::Arrangement,
    EdgeFamily::Imported,
    EdgeFamily::Provenance,
];

/// Whether `family` is present in a subgraph spec's selector-string list (a Subgraph
/// Card selector chip's checked state).
pub fn spec_has_family(spec: &SubgraphSpec, family: EdgeFamily) -> bool {
    let name = edge_family_str(family);
    spec.selectors.iter().any(|s| s.eq_ignore_ascii_case(name))
}

/// A minimal `SubgraphSpec` for a branch whose donor carried no canonical spec: a
/// single-anchor `Session` grouping. Phase 2 enriches this from the donor's actual
/// subgraph once branches can carry one.
pub fn default_spec_for(anchor: GraphMemberId) -> SubgraphSpec {
    SubgraphSpec {
        kind: SubgraphKind::Session,
        anchors: vec![anchor.to_string()],
        primary_anchor: Some(anchor.to_string()),
        selectors: Vec::new(),
    }
}

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "query"))]
mod query_tests;
