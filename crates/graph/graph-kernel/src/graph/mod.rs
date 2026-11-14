// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Graph data structures for the spatial browser.
//!
//! Core structures:
//! - `Graph`: Main graph container backed by petgraph::StableGraph
//! - `Node`: Webpage node with a transient projected position and metadata
//! - `EdgePayload`: Edge semantics and traversal events between nodes
//!
//! This module is WASM-clean: it must compile to `wasm32-unknown-unknown`.

use euclid::default::Point2D;
// `MemoryEntryPrivacy` / `OwnerScopedMemory` / `GraphMemorySnapshot` /
// `MemoryTransitionKind` were used by the node-history types that
// moved to `history.rs` (2026-05-11 decomposition pass); `Graph` itself
// doesn't reach into node-lineage directly.
use petgraph::algo::{astar, dijkstra, has_path_connecting, kosaraju_scc};
#[cfg(test)]
use petgraph::stable_graph::NodeIndex;
use petgraph::visit::{EdgeRef, IntoEdgeReferences};
use rkyv::{Archive, Archived, Deserialize, Place, Resolver, Serialize, rancor::Fallible};
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

pub use hodograph::TransitionKind as NodeHistoryTransitionKind;
/// The stable address of a node's out-of-line representation.
pub use muniment::Hash as ContentHash;

use crate::address::{Address, AddressKind, address_from_url, cached_host_from_url, detect_mime};
use crate::persistence::{
    GraphSnapshot, PersistedAddress, PersistedArrangementEdgeData, PersistedArrangementSubKind,
    PersistedContainmentEdgeData, PersistedContainmentSubKind, PersistedEdge, PersistedEdgeFamily,
    PersistedImportedEdgeData, PersistedImportedSubKind, PersistedNavigationTrigger, PersistedNode,
    PersistedNodeSessionState, PersistedProvenanceEdgeData, PersistedProvenanceSubKind,
    PersistedSemanticEdgeData, PersistedSemanticSubKind, PersistedTraversalEdgeData,
    PersistedTraversalMetrics, PersistedTraversalRecord,
};
#[allow(unused_imports)] // Some used only in #[cfg(test)]
use crate::types::{
    ClassificationProvenance, ClassificationScheme, ClassificationStatus, DominantEdge,
    FrameLayoutHint, FrameLayoutNodeId, GraphScope, ImportRecord, ImportRecordMembership,
    NodeClassification, NodeImportProvenance, NodeImportRecordSummary, NodeTagPresentationState,
    SplitOrientation, format_imported_at_secs,
};

pub mod apply;
pub mod assertion_handles;
pub mod capture;
/// Cross-graph node copy (tear-out fork): mints a node in this graph from a
/// donor node in another graph, recording cross-graph derivation provenance.
pub mod cross_graph;
/// The edit spine: mere's captured-delta stream as a muniment journal, replayed into
/// the graph (the substrate's append-only-log primitive over mere's own edit
/// vocabulary). See `graph/journal.rs`.
pub mod journal;
pub mod legacy_content_migration;
pub(crate) mod legacy_family_attribution;
pub mod legacy_resource_migration;
#[cfg(test)]
mod legacy_resource_migration_copies_tests;
#[cfg(test)]
mod legacy_resource_migration_malformed_tests;
#[cfg(test)]
mod legacy_resource_migration_reason_tests;
pub use cross_graph::ComponentCopy;
mod assertion_write;
pub mod pending_links;
pub use pending_links::{PendingLink, PendingLinkRetention, PendingLinkRetry, PendingLinkState};
pub mod frozen_selection;
pub use frozen_selection::OpenedFrozenSelection;
pub mod frozen_link;
mod handle_index;
#[cfg(test)]
mod handle_index_tests;
pub use frozen_link::{FrozenLink, FrozenLinkSource, FrozenSourceRevision, OpenedFrozenLink};
#[cfg(test)]
mod assertion_handles_tests;
pub mod coverage;
#[cfg(test)]
mod frozen_link_tests;
#[cfg(test)]
mod frozen_selection_tests;
mod residency;
pub use coverage::{CoverageLayer, CoverageLimit, CoverageNote};
mod edge_admission;
/// Moved to `mere::model` (engine split P5 L3). The names other repositories
/// name at this path stay re-exported here until the kernel moves at P8.
pub mod edge_data {
    pub use mere::model::edge_data::{Traversal, UNKNOWN_LEGACY_ASSERTER_IRI};
}
/// Moved to `mere::model` (engine split P5 L3); see [`edge_data`].
pub mod edge_taxonomy {
    pub use mere::model::edge_taxonomy::NavigationTrigger;
}
pub mod facet_projection;
pub mod filter;
/// Test-fixture escape hatch over the `pub(crate)` mutators (write-path
/// migration). Feature-gated: enable `fixtures` in dev-dependencies only.
#[cfg(feature = "fixtures")]
pub mod fixtures;
pub mod history;
pub mod identity;
mod input_key;
pub use input_key::GraphInputKey;
mod membership;
pub use membership::SurfaceNodeLifetime;
pub mod import_records;
pub mod merge;
mod navigation_check;
pub mod node;
pub mod node_facets;
pub mod node_props;
pub mod predicate_declarations;
mod relation_read;
#[cfg(test)]
mod replay_admission_tests;
pub mod resource;
pub mod resource_classifications;
pub mod resource_content;
#[cfg(test)]
mod resource_content_edge_tests;
#[cfg(test)]
mod resource_content_handle_tests;
#[cfg(test)]
mod resource_content_tests;
mod resource_pair_bounds;
pub mod resource_properties;
pub mod resource_tag_admission;
pub mod resource_tags;
/// Reverting one change: undo's edits and the parts it keeps (reservoir plan V2).
pub mod revert;
pub mod source_time;
// chartulary capability-trait impls for `Node` (graph re-base, G5).
mod chart;

// Field system: the field-layer definition types (Field / Coupling / the field AST /
// EdgePath) live in chartulary (engine-split D49), promoted out of the kernel so the
// third graph primitive sits at the same portable tier as chartulary's nodes/edges.
// The kernel keeps the parallel keyed store on `Graph` (fields/couplings are neither
// petgraph node weights nor `EdgePayload` sidecars) and re-exports chartulary's types.
/// Graph mutators + queries for the field layer (Phase 1): the parallel keyed store
/// on `Graph` + selector evaluation. The field-layer *definition* types (Field /
/// Coupling / the field AST / EdgePath) now live in chartulary,
/// re-exported below so `kernel::graph::` paths stay stable.
mod field_ops;

// Identity types and rkyv archive helpers extracted to `identity.rs`
// per the 2026-04-30 renderer plan §6.4 decomposition target. Public
// types are re-exported at this module path so external callers
// (`kernel::graph::NodeKey`, etc.) continue to resolve;
// the rkyv `with = ...` archive helpers are crate-internal and used
// only by struct field annotations in this file.
pub(crate) use identity::UuidAsBytes;
pub use identity::{
    EdgeKey, GraphDirection, GraphIndex, GraphViewId, NodeKey, RelationKey, ResourceEdgeKey,
    SurfaceNodeKey,
};

// Node + NodeLifecycle extracted to `node.rs` per the same
// decomposition target. Re-exported so `kernel::graph::Node`
// continues to resolve.
pub use assertion_write::StatementWriteError;
pub use edge_ops::Retraction;
pub(crate) use mere::model::{built_in_predicate_stratum, built_in_relation_stratum};
pub use node::{Node, SurfaceNode};
pub use node_facets::{NodeFacetStore, VisitHistoryFacet};
pub use predicate_declarations::{
    PredicateDeclaration, PredicateDeclarationError, PredicateDeclarations,
};
pub use resource::ResourceNode;

// Node navigation history extracted to `history.rs` (2026-05-11
// kernel-mod decomposition pass). Re-exported so external callers
// (`kernel::graph::NodeNavigationMemory`, etc.) keep resolving.
pub use history::{
    NodeHistoryBranchAlternative, NodeHistoryBranchProjection, NodeHistoryBranchVisit,
    NodeHistoryOwner, NodeHistoryProjection, NodeHistorySemanticSummary, SharedNavigationMemory,
};
pub use navigation_check::NavigationIndexError;

// Edge taxonomy (family/sub-kind enums) and per-family runtime data structs
// extracted to their own modules. edge_taxonomy holds classifiers; edge_data
// holds runtime payload types (Data structs, Traversal, EdgeMetrics, IRI fns).
// Stage 4 of the 2026-05-11 relation-taxonomy plan removed `EdgeType` and
// `EdgeKind`; reads go through [`RelationKind`] + [`RelationSelector`], writes
// through [`EdgeAssertion`].
pub use capture::{
    CapturedDelta, DeltaRecorder, GraphTableStats, Malformed, Replay, ReplayRefused, Unwritable,
    replay_captured_deltas, replay_captured_deltas_as_onto, replay_captured_deltas_onto,
    replay_captured_deltas_onto_checked, set_captured_delta_hook, with_isolated_capture,
};
pub use journal::{
    AttributedDelta, Author, AuthorKind, GraphJournal, ReplayPoint, USER_AUTHOR,
    journal_capture_hook,
};
pub use merge::import_edits;
pub use revert::{
    Part, Revert, Touched, part_edits, part_node, part_of, part_value, recreate_node_edits,
    revert_change,
};
pub use source_time::{SourceExtent, SourceTime};
// The borne-graph identity type (`Node.nested`): part of the node's public
// surface, re-exported so consumers name it without a direct muniment dep.
// The assertion model moved to `mere::model` (engine split P5 L3). Names other
// repositories use are re-exported publicly; the rest only for the kernel.
#[allow(unused_imports)]
pub(crate) use mere::model::{
    ArrangementData, AssertionData, ContainmentData, EdgeMetrics, ImportedData, ProvenanceData,
    REL_VOCAB, RelationAssertion, RelationAssertionSpec, RelationDurability, SemanticData,
    StatementAssert, Traversal, TraversalData, all_semantic_sub_kinds,
};
pub use mere::model::{
    ArrangementSubKind, ContainmentSubKind, EdgeAssertion, EdgeFamily, EdgePayload, GraphStratum,
    ImportedSubKind, NavigationTrigger, ProvenanceSubKind, RelationKind, RelationSelector,
    SemanticStatement, SemanticStatementSpec, SemanticSubKind, predicate_iri, sub_kind_from_iri,
};
pub use muniment::{LogId, Seq};

// Field-system truth types (2026-05-31). Field/Coupling form a parallel field
// layer beside the node/edge graph; Seiche's numen reads them and evaluates.
pub use fieldbook::coupling::{COUPLING_VOCAB, Coupling, CouplingResponse, NodeSelector};
pub use fieldbook::edge_path::{EdgePath, EdgePathRule};
pub use fieldbook::field::{
    CouplingId, Field, FieldDefinition, FieldExtent, FieldId, FieldLifecycle,
};
pub use fieldbook::field_ast::{Falloff, ScalarField, VectorField};

/// Traversal archive payload emitted when dissolving a node.
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize)]
pub struct DissolvedTraversalRecord {
    #[rkyv(with = UuidAsBytes)]
    pub from_node_id: Uuid,
    #[rkyv(with = UuidAsBytes)]
    pub to_node_id: Uuid,
    pub traversals: Vec<Traversal>,
}

fn normalize_import_record_memberships(memberships: &mut Vec<ImportRecordMembership>) {
    memberships.sort_by(|left, right| {
        left.node_id
            .cmp(&right.node_id)
            .then_with(|| left.suppressed.cmp(&right.suppressed))
    });
    let mut deduped: Vec<ImportRecordMembership> = Vec::with_capacity(memberships.len());
    for membership in memberships.drain(..) {
        if let Some(existing) = deduped.last_mut()
            && existing.node_id == membership.node_id
        {
            existing.suppressed &= membership.suppressed;
            continue;
        }
        deduped.push(membership);
    }
    *memberships = deduped;
}

pub(crate) fn normalize_import_records(import_records: &mut Vec<ImportRecord>) {
    let mut merged = BTreeMap::<String, ImportRecord>::new();
    for mut record in import_records.drain(..) {
        normalize_import_record_memberships(&mut record.memberships);
        if record.record_id.trim().is_empty() {
            continue;
        }
        let entry = merged
            .entry(record.record_id.clone())
            .or_insert_with(|| ImportRecord {
                record_id: record.record_id.clone(),
                source_id: record.source_id.clone(),
                source_label: record.source_label.clone(),
                imported_at_secs: record.imported_at_secs,
                memberships: Vec::new(),
            });
        if entry.source_id.is_empty() {
            entry.source_id = record.source_id.clone();
        }
        if entry.source_label.is_empty() {
            entry.source_label = record.source_label.clone();
        }
        if entry.imported_at_secs == 0 {
            entry.imported_at_secs = record.imported_at_secs;
        } else if record.imported_at_secs != 0 {
            entry.imported_at_secs = entry.imported_at_secs.min(record.imported_at_secs);
        }
        entry.memberships.extend(record.memberships);
    }
    *import_records = merged.into_values().collect();
    for record in import_records.iter_mut() {
        normalize_import_record_memberships(&mut record.memberships);
    }
    import_records.sort_by(|left, right| {
        left.source_label
            .cmp(&right.source_label)
            .then_with(|| left.imported_at_secs.cmp(&right.imported_at_secs))
            .then_with(|| left.record_id.cmp(&right.record_id))
    });
}

pub(crate) fn current_unix_timestamp_secs() -> u64 {
    crate::time::unix_epoch_seconds()
}

// Node, NodeLifecycle, and impl Node moved to `node.rs` per the
// 2026-04-30 renderer plan §6.4 decomposition target. Re-exported
// here so external paths (`kernel::graph::Node`, etc.)
// resolve unchanged.

/// Canonical read-side relation view. One row per (from, to,
/// [`RelationKind`]) — a multi-relation node pair yields multiple
/// rows. Replaces the `EdgeType`-flavoured `EdgeView` removed in
/// stage 4 of the 2026-05-11 relation-taxonomy plan.
///
/// This is the **classifier shape** — it carries no per-relation
/// payload (labels, decay, traversal events). Callers needing
/// payload reach for the typed `EdgePayload` sidecar via the
/// kernel's per-family query methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RelationView {
    pub from: NodeKey,
    pub to: NodeKey,
    pub kind: RelationKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticEdgeView {
    pub from: NodeKey,
    pub to: NodeKey,
    pub sub_kind: SemanticSubKind,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ArrangementEdgeView {
    pub from: NodeKey,
    pub to: NodeKey,
    pub sub_kind: ArrangementSubKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContainmentEdgeView {
    pub from: NodeKey,
    pub to: NodeKey,
    pub sub_kind: ContainmentSubKind,
}

/// Main graph structure backed by petgraph::StableGraph
#[derive(Clone)]
pub struct Graph {
    /// Runtime-only model input versions, preserved by clones and advanced at
    /// the existing structural/content/site/visit mutation boundaries.
    runtime_input_keys: [GraphInputKey; 4],
    /// The substrate graph: a [`portolan::Graph`] over mere's [`Node`] and
    /// [`EdgePayload`] (the G5 re-base — mere's graph *is* a chartulary graph).
    /// It owns the petgraph topology and the stable id-to-key index (mere's old
    /// hand-rolled `id_to_node` is retired in its favor). `pub(crate)` to enforce
    /// the single-write-path boundary at the type level: every topology mutation
    /// goes through a `Graph` method so it advances [`revision`](Self::revision);
    /// direct mutation from outside the kernel would bypass the bump and stale the
    /// structural caches. Reads reach petgraph algorithms through
    /// [`portolan::Graph::inner`]. (Graph signals.)
    pub(crate) inner: portolan::Graph<Node, EdgePayload>,
    /// Runtime membership witnesses; cloned with candidates, never snapshotted.
    surface_node_lifetimes: HashMap<Uuid, SurfaceNodeLifetime>,

    /// Resource truth beneath surfaces, indexed by the prepared identity IRI,
    /// with the one live authority for ordinary resource metadata (its facets).
    /// The engine's model (`mere::model::ResourceGraph`); `pub(crate)` so every
    /// write goes through a recorded `Graph` method, as for `inner`.
    pub(crate) resources: mere::model::ResourceGraph,
    /// Explicit surface UUID to resource UUID associations.
    pub(crate) shown_resources: BTreeMap<Uuid, Uuid>,
    /// `shown_resources` inverted: the surfaces showing each resource. Kept
    /// in step by `set_shown_resource` and node removal, its only writers.
    pub(crate) resource_surfaces: BTreeMap<Uuid, std::collections::BTreeSet<Uuid>>,

    /// Atomic optional metadata keyed by stable node id. This is the single
    /// live authority persisted by the host as `facets.json`; snapshot columns
    /// are legacy import inputs only.
    pub(crate) facets: portolan::FacetStore<Uuid>,

    /// URL to node mapping for lookup (supports duplicate URLs).
    pub(crate) url_to_nodes: HashMap<String, Vec<NodeKey>>,

    /// Durable imported relation truth; node provenance is derived from this.
    pub(crate) import_records: Vec<ImportRecord>,

    /// Field-layer truth (field-system step 3, Phase 1): a parallel keyed store
    /// beside the node/edge petgraph. A coupling targets a *selector* over nodes,
    /// not a node→node edge, so fields/couplings cannot ride the petgraph; they
    /// are content truth that numen reads and evaluates (derived).
    ///
    /// Ordered by id (the inner UUID, which orders as its canonical string
    /// does; `FieldId` and `CouplingId` carry no `Ord`), so every read and
    /// every snapshot sees one order.
    pub(crate) fields: BTreeMap<Uuid, Field>,
    pub(crate) couplings: BTreeMap<Uuid, Coupling>,

    /// The graph's whole navigation history: one shared visit space, one owner
    /// per node (the (b) anchor design). In-place navigation extends a node's own
    /// path; a branch node is spawned anchored under its origin's current visit.
    pub(crate) nav: SharedNavigationMemory,

    /// A monotonic **structural revision**, bumped by [`bump_revision`](Self::bump_revision)
    /// whenever the node/edge structure or a pair's relation set changes (add/remove node, add/
    /// remove edge, assert/retract a relation). It does NOT bump on a re-visit's traversal append
    /// or on pure content edits (title/url/tags), since those leave the structure a derived signal
    /// reads unchanged. The single source of truth any consumer gates a cache on — community,
    /// arrangements, importance — so a recompute happens once per real change, not per frame.
    /// (Graph signals — the local counter; cross-owner caches use structural_input_key.)
    revision: u64,

    /// A monotonic revision for the URL-authority partition used by views such
    /// as site kanban. URL edits do not change topology, so they deliberately
    /// do not advance [`revision`](Self::revision); a consumer that groups
    /// nodes by authority must depend on this narrower signal instead.
    url_grouping_revision: u64,

    /// A monotonic **content revision**: bumped when a node's text may have
    /// changed (a node added, copied in or removed, its title set, its
    /// primary URL moved). Structure, visits, tags, images and facets leave it
    /// alone. A consumer keyed to node text, such as the canvas's Meaning
    /// channel, recomputes only when it moves. Not persisted, like
    /// [`revision`](Self::revision). (Dynamics grammar plan, G2, F33.)
    content_revision: u64,

    /// A monotonic **visit revision**: bumped whenever a node's visit history
    /// (`visit.history`) may have changed, through any kernel writer of it, a
    /// generic facet delta naming it, a sidecar overlay, or mutable facet
    /// access. Structure and text leave it alone. A consumer keyed to recency,
    /// such as the channel registry's recency order, recomputes only when it or
    /// the structural revision moves. Not persisted. (Dynamics grammar plan,
    /// G2b, F54.)
    visit_revision: u64,

    /// The current app-launch session number, set once by the host via
    /// [`set_current_session`](Self::set_current_session) right after construction/
    /// restore. `0` (the default) means "not wired" — [`navigate_node`](Self::navigate_node)
    /// then stamps nothing, so by-sessions eviction sees every node as undated until the
    /// host opts in. Not part of the persisted snapshot: it is per-launch host state, not
    /// graph truth. (Alembic B5 — by-sessions eviction.)
    current_session: u64,

    /// Attribution context supplied by the session performing a write.
    write_author: Author,

    /// Where this graph sends its own captured deltas, once a host opts it in
    /// with [`set_recorder`](Self::set_recorder). Not graph truth, and not
    /// carried by a clone.
    pub(crate) recorder: capture::Recorder,
    pub(crate) pending_link_state: PendingLinkState,
    pub(crate) pending_derivation_suppressed: pending_links::Suppression,
    pub(crate) semantic_observation_revision: u64,
    pub(crate) appearance_admission_revision: u64,
    pub(crate) known_coverage: CoverageNote,
    /// Set only inside a legacy migration's staged scope; see `edge_admission`.
    pub(crate) staged_edges: edge_admission::StagedEdges,
    /// Runtime index of where each statement handle is held; see `handle_index`.
    pub(crate) handle_index: handle_index::HandleIndex,
}

// Edge mutators (add_edge, assert_relation, replay_*, dissolve_*,
// remove_edges, retract_relations, get_edge*, push_traversal) live in
// `graph/edge_ops.rs` (2026-05-11 decomposition pass).
pub mod edge_ops;

// Query + traversal (node/edge lookups, neighbor iteration, BFS /
// shortest-path / connected-components / SCC, derived-edge views)
// live in `graph/query.rs` (2026-05-11 decomposition pass).
pub mod query;

// `node_display_label`: a human-meaningful label derived from a node's title /
// linked-data role / @type / address, for the canvas tiles + roster (so an
// anonymous ingested entity reads as "publisher" / "Organization", not a raw
// `urn:mere:bnode:` string).
pub mod display;

// Snapshot serialization (`to_snapshot` / `from_snapshot`), the rkyv
// trait impls that delegate through them, `containment_parent_url`,
// and `impl Default for Graph` live in `graph/snapshot.rs`
// (2026-05-11 decomposition pass).
pub mod snapshot;

// Tests live in `graph/tests.rs` to keep this file under the
// 600-LOC ceiling (kernel decomposition pass 2026-05-11).
#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "tests/resource_lifecycle.rs"]
mod resource_lifecycle_tests;

#[path = "operations.rs"]
mod operations;
