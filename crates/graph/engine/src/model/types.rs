// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The assertion model's leaf types: graph scope, assertion basis and
//! withdrawal, classification records and literal properties, and the
//! statement-id minter. Moved from the kernel's `types` (engine split P5 L3);
//! the kernel re-exports them at their old paths.

use std::sync::atomic::{AtomicU64, Ordering};

use rkyv::{Archive, Deserialize, Serialize};
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub enum GraphScope {
    #[default]
    Default,
    Source,
    User,
    Agent,
    Moot,
    Custom(String),
}

/// Retained record-time tombstone for an assertion withdrawal.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub struct AssertionWithdrawal {
    /// Informative caller-supplied or authored record time; `None` stays
    /// undated for legacy/imported withdrawals.
    pub at_ms: Option<u64>,
    /// Actor IRI when the source supplies one.
    pub by: Option<String>,
}

/// Optional account of why an assertion exists. `occurrences` remain with
/// traversal sidecars until their P6 convergence phase.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub struct AssertionBasis {
    /// Compatibility projection of the legacy assertion label.
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub method: Option<AssertionMethod>,
    /// Source order is retained; reason-key comparison sorts a copy.
    #[serde(default)]
    pub evidence: Vec<String>,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq, Eq))]
pub struct AssertionMethod {
    pub iri: String,
    #[serde(default)]
    pub version: Option<String>,
}

static NEXT_LOCAL_STATEMENT_NONCE: AtomicU64 = AtomicU64::new(1);
static STATEMENT_MINTER_SALT: std::sync::OnceLock<u64> = std::sync::OnceLock::new();

#[cfg(test)]
#[path = "types/minting_tests.rs"]
mod minting_tests;

// ---------------------------------------------------------------------------
// Node classification — Stage A durable enrichment schema
// ---------------------------------------------------------------------------

/// Classification scheme identifier.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ClassificationScheme {
    /// Universal Decimal Classification (primary semantic taxonomy).
    #[default]
    Udc,
    /// Content-kind classification (page, article, repo, …).
    ContentKind,
    /// Custom namespaced scheme (e.g. `"myns:custom"`).
    Custom(String),
}

/// Origin of a classification or tag.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ClassificationProvenance {
    /// Explicitly authored by the user.
    #[default]
    UserAuthored,
    /// Imported from an external data source (bookmarks, history, file, …).
    Imported,
    /// Inherited from a source/parent node relationship.
    InheritedFromSource,
    /// Derived by the knowledge registry (UDC lookup, content analysis, …).
    RegistryDerived,
    /// Proposed by an agent/model; not yet accepted by the user.
    AgentSuggested,
    /// Synced from the community/Verse network.
    CommunitySynced,
}

/// Lifecycle status of a classification record.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Default,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum ClassificationStatus {
    /// User has explicitly accepted this classification.
    Accepted,
    /// Proposed but not yet reviewed (e.g. agent-suggested).
    #[default]
    Suggested,
    /// User has explicitly rejected this classification.
    Rejected,
    /// Verified by an authoritative external source.
    Verified,
    /// Imported from an external record without explicit user review.
    Imported,
}

/// A single provenance-bearing classification record on a node.
///
/// Multiple records can coexist; at most one should have `primary: true` per scheme.
#[derive(
    Debug, Clone, PartialEq, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct NodeClassification {
    pub scheme: ClassificationScheme,
    /// Scheme-specific classification value (e.g. `"udc:519.6"`, `"article"`).
    pub value: String,
    /// Human-readable label resolved from the scheme (e.g. `"Computational mathematics"`).
    pub label: Option<String>,
    /// Confidence score in `[0.0, 1.0]`; `1.0` for user-authored.
    pub confidence: f32,
    pub provenance: ClassificationProvenance,
    pub status: ClassificationStatus,
    /// Whether this is the primary presentation classification for its scheme.
    pub primary: bool,
}

/// A literal property on a node: an open predicate IRI and its value. Holds the
/// non-curated literals an ingest preserves — the kernel has no other general
/// key→value bag (`title` / `tags` stay the curated fast-paths).
#[derive(Debug, Clone, PartialEq, Eq, Archive, Serialize, Deserialize, serde::Deserialize)]
pub struct NodeProperty {
    /// Stable handle for this literal statement.
    #[serde(default = "mint_local_statement_id")]
    pub statement_id: String,
    /// The predicate IRI (e.g. `https://schema.org/datePublished`).
    pub predicate: String,
    /// The literal value.
    pub value: String,
    /// Explicit datatype IRI when the literal is not the default `xsd:string`.
    #[serde(default)]
    pub datatype: Option<String>,
    /// BCP47 language tag for an RDF language-tagged string.
    #[serde(default)]
    pub lang: Option<String>,
    /// Named-graph scope for this literal statement.
    #[serde(default)]
    pub graph_scope: GraphScope,
    /// Optional provenance agent/persona IRI for this assertion.
    #[serde(default)]
    pub provenance_iri: Option<String>,
    /// Optional assertion time in unix epoch milliseconds.
    #[serde(default)]
    pub asserted_at_ms: Option<u64>,
}

#[cfg(test)]
#[path = "types/property_serialization_tests.rs"]
mod property_serialization_tests;

#[path = "types/property.rs"]
mod property;

#[path = "types/operations.rs"]
mod operations;

#[path = "types/statement_identity.rs"]
mod statement_identity;

pub use statement_identity::seed_statement_minter;

pub use statement_identity::mint_local_statement_id;
