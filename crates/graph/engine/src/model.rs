// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Model stage: the assertion model and the resource graph (engine split P5 L3).
//!
//! The assertion rules live on [`EdgePayload`] and [`SemanticData`]: separate
//! assertions per asserter, withdrawal retained until compaction, no revival by
//! a stale assert, and reason keys ([`AssertionBasis::same_reason_as`]).
//! [`ResourceGraph`] holds resources, the edges between them and their facets.
//!
//! These types moved here from `mere-kernel`, which composes its surface graph
//! over them and records every write to them through its journal. The kernel
//! re-exports, at their old paths, the names other repositories use, until it
//! moves at P8. Every serialized shape is unchanged.

pub mod author;
pub mod clock;
pub mod coverage;
pub mod edge_data;
pub mod edge_payload;
pub mod edge_taxonomy;
pub mod frozen;
pub mod persistence_edge;
pub mod predicate_registry;
pub mod resource;
pub mod resource_content;
pub mod resource_graph;
pub mod resource_record;
pub mod types;

pub use author::{Author, AuthorKind, USER_AUTHOR};
pub use coverage::{CoverageLayer, CoverageLimit, CoverageNote};
pub use edge_data::{
    ArrangementData, AssertionData, ContainmentData, EdgeMetrics, ImportedData, ProvenanceData,
    REL_VOCAB, RelationAssertion, RelationAssertionSpec, SemanticData, SemanticStatement,
    SemanticStatementSpec, StatementAssert, Traversal, TraversalData, UNKNOWN_LEGACY_ASSERTER_IRI,
    all_semantic_sub_kinds, predicate_iri, sub_kind_from_iri,
};
pub use edge_payload::{EdgePayload, SemanticDataView, selector_matches_kind};
pub use edge_taxonomy::{
    ArrangementSubKind, ContainmentSubKind, EdgeAssertion, EdgeFamily, ImportedSubKind,
    NavigationTrigger, ProvenanceSubKind, RelationDurability, RelationKind, RelationSelector,
    SemanticSubKind,
};
pub use frozen::{
    FROZEN_LINK, FROZEN_SELECTION, FrozenLink, FrozenLinkEndpoint, FrozenLinkSource,
    FrozenSelection, FrozenSourceBinding, FrozenSourceRevision,
};
pub use persistence_edge::{
    PersistedArrangementEdgeData, PersistedArrangementSubKind, PersistedContainmentEdgeData,
    PersistedContainmentSubKind, PersistedEdge, PersistedEdgeAssertion, PersistedEdgeFamily,
    PersistedImportedEdgeData, PersistedImportedSubKind, PersistedNavigationTrigger,
    PersistedProvenanceEdgeData, PersistedProvenanceSubKind, PersistedRelationSelector,
    PersistedSemanticEdgeData, PersistedSemanticStatement, PersistedSemanticSubKind,
    PersistedTraversalEdgeData, PersistedTraversalMetrics, PersistedTraversalRecord,
};
pub use predicate_registry::{
    GraphStratum, built_in_predicate_stratum, built_in_relation_stratum, default_predicate_stratum,
};
pub use resource::{ResourceNode, TAGGED_WITH_IRI};
pub use resource_content::{
    ClassificationOrigin, ClassificationVariant, RESOURCE_CLASSIFICATIONS, RESOURCE_PROPERTIES,
    TAG_CONCEPT, TagConcept, tag_concept_iri,
};
pub use resource_graph::ResourceGraph;
pub use resource_record::{PersistedResourceFacet, PersistedResourceRecord};
pub use types::{
    AssertionBasis, AssertionMethod, AssertionWithdrawal, ClassificationProvenance,
    ClassificationScheme, ClassificationStatus, GraphScope, NodeClassification, NodeProperty,
    mint_local_statement_id, seed_statement_minter,
};
