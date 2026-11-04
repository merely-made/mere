// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Edge persistence types — family/sub-kind enums, per-family data structs,
//! and the edge container. Re-exported through [`crate::persistence`] to keep
//! `persistence.rs` under the per-file ceiling.

use rkyv::{Archive, Deserialize, Serialize};

use crate::types::{AssertionBasis, AssertionWithdrawal, GraphScope};

// ---------------------------------------------------------------------------
// Edge persistence types
// ---------------------------------------------------------------------------

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedEdgeFamily {
    Semantic,
    Traversal,
    Containment,
    Arrangement,
    Imported,
    Provenance,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedSemanticSubKind {
    Hyperlink,
    UserGrouped,
    AgentDerived,
    Cites,
    Quotes,
    Summarizes,
    Elaborates,
    ExampleOf,
    Supports,
    Contradicts,
    Questions,
    SameEntityAs,
    DuplicateOf,
    CanonicalMirrorOf,
    DependsOn,
    Blocks,
    NextStep,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedContainmentSubKind {
    UrlPath,
    Domain,
    FileSystem,
    UserFolder,
    ClipSource,
    NotebookSection,
    CollectionMember,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedArrangementSubKind {
    FrameMember,
    TileGroup,
    SplitPair,
    TabNeighbor,
    ActiveTab,
    PinnedInFrame,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedImportedSubKind {
    BookmarkFolder,
    HistoryImport,
    SessionImport,
    RssMembership,
    FileSystemImport,
    ArchiveMembership,
    SharedCollection,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedProvenanceSubKind {
    ClippedFrom,
    ExcerptedFrom,
    SummarizedFrom,
    TranslatedFrom,
    RewrittenFrom,
    GeneratedFrom,
    ExtractedFrom,
    ImportedFromSource,
    /// Verbatim duplicate (cross-graph copy / tear-out fork). Appended last to
    /// keep existing ordinals stable. Mirrors `ProvenanceSubKind::CopiedFrom`.
    CopiedFrom,
    /// Brought in by a session merge-in. Appended last to keep ordinals
    /// stable. Mirrors `ProvenanceSubKind::MergedFrom`.
    MergedFrom,
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedSemanticStatement {
    pub statement_id: String,
    pub predicate: String,
    pub recognized_sub_kind: Option<PersistedSemanticSubKind>,
    pub label: Option<String>,
    pub graph_scope: GraphScope,
    pub provenance_iri: Option<String>,
    pub asserted_at_ms: Option<u64>,
    /// Appended after the legacy persisted fields. `asserted_at_ms` remains
    /// the last-confirmation time for stored compatibility.
    pub first_asserted_ms: Option<u64>,
    pub withdrawn: Option<AssertionWithdrawal>,
    /// Appended to preserve the existing positional field sequence.
    pub basis: AssertionBasis,
}

struct PresentOptionU64(Option<Option<u64>>);
struct PresentBasis(Option<Option<AssertionBasis>>);

impl Default for PresentOptionU64 {
    fn default() -> Self {
        Self(None)
    }
}

impl Default for PresentBasis {
    fn default() -> Self {
        Self(None)
    }
}

impl<'de> serde::Deserialize<'de> for PresentBasis {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = if deserializer.is_human_readable() {
            Option::<AssertionBasis>::deserialize(deserializer)?
        } else {
            Some(AssertionBasis::deserialize(deserializer)?)
        };
        Ok(Self(Some(value)))
    }
}

impl<'de> serde::Deserialize<'de> for PresentOptionU64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = <Option<u64> as serde::Deserialize>::deserialize(deserializer)?;
        Ok(Self(Some(value)))
    }
}

impl<'de> serde::Deserialize<'de> for PersistedSemanticStatement {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(serde::Deserialize)]
        struct Fields {
            statement_id: String,
            predicate: String,
            #[serde(default)]
            recognized_sub_kind: Option<PersistedSemanticSubKind>,
            #[serde(default)]
            label: Option<String>,
            #[serde(default)]
            graph_scope: GraphScope,
            #[serde(default)]
            provenance_iri: Option<String>,
            #[serde(default)]
            asserted_at_ms: Option<u64>,
            #[serde(default)]
            first_asserted_ms: PresentOptionU64,
            #[serde(default)]
            withdrawn: Option<AssertionWithdrawal>,
            #[serde(default)]
            basis: PresentBasis,
        }

        let fields = <Fields as serde::Deserialize>::deserialize(deserializer)?;
        let basis = fields.basis.0.flatten().unwrap_or_else(|| AssertionBasis {
            note: fields.label.clone(),
            ..Default::default()
        });
        let label = basis.note.clone();
        Ok(Self {
            statement_id: fields.statement_id,
            predicate: fields.predicate,
            recognized_sub_kind: fields.recognized_sub_kind,
            label,
            basis,
            graph_scope: fields.graph_scope,
            provenance_iri: fields.provenance_iri,
            first_asserted_ms: fields.first_asserted_ms.0.unwrap_or(fields.asserted_at_ms),
            asserted_at_ms: fields.asserted_at_ms,
            withdrawn: fields.withdrawn,
        })
    }
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedSemanticEdgeData {
    #[serde(default)]
    pub sub_kinds: Vec<PersistedSemanticSubKind>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub agent_decay_progress: Option<f32>,
    /// Open predicate IRI (statements-over-schema). `#[serde(default)]` so old
    /// graphs load with `None`.
    #[serde(default)]
    pub predicate: Option<String>,
    /// Pair-local semantic statement bucket. `#[serde(default)]` keeps old
    /// snapshots loading through the aggregate compatibility fields above.
    #[serde(default)]
    pub statements: Vec<PersistedSemanticStatement>,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedTraversalRecord {
    pub timestamp_ms: u64,
    pub trigger: PersistedNavigationTrigger,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedTraversalMetrics {
    pub total_navigations: u64,
    pub forward_navigations: u64,
    pub backward_navigations: u64,
    pub last_navigated_at: Option<u64>,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedTraversalEdgeData {
    #[serde(default)]
    pub traversals: Vec<PersistedTraversalRecord>,
    #[serde(default)]
    pub metrics: PersistedTraversalMetrics,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedContainmentEdgeData {
    #[serde(default)]
    pub sub_kinds: Vec<PersistedContainmentSubKind>,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedArrangementEdgeData {
    #[serde(default)]
    pub sub_kinds: Vec<PersistedArrangementSubKind>,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedImportedEdgeData {
    #[serde(default)]
    pub sub_kinds: Vec<PersistedImportedSubKind>,
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    Default,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedProvenanceEdgeData {
    #[serde(default)]
    pub sub_kinds: Vec<PersistedProvenanceSubKind>,
}

#[derive(
    Archive, Serialize, Deserialize, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedEdgeAssertion {
    Semantic {
        sub_kind: PersistedSemanticSubKind,
        label: Option<String>,
        agent_decay_progress: Option<f32>,
    },
    Containment {
        sub_kind: PersistedContainmentSubKind,
    },
    Arrangement {
        sub_kind: PersistedArrangementSubKind,
    },
    Imported {
        sub_kind: PersistedImportedSubKind,
    },
    Provenance {
        sub_kind: PersistedProvenanceSubKind,
    },
}

#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedRelationSelector {
    Family(PersistedEdgeFamily),
    Semantic(PersistedSemanticSubKind),
    Containment(PersistedContainmentSubKind),
    Arrangement(PersistedArrangementSubKind),
    Imported(PersistedImportedSubKind),
    Provenance(PersistedProvenanceSubKind),
}

/// Persisted traversal trigger classification.
#[derive(
    Archive,
    Serialize,
    Deserialize,
    Clone,
    Copy,
    Debug,
    PartialEq,
    Eq,
    serde::Serialize,
    serde::Deserialize,
)]
#[rkyv(derive(Debug, PartialEq))]
pub enum PersistedNavigationTrigger {
    Unknown,
    LinkClick,
    Back,
    Forward,
    AddressBarEntry,
    PanePromotion,
    Programmatic,
    /// Server redirect (3xx) or meta-refresh.
    Redirect,
    /// Navigation re-issued by session restore.
    ReopenSession,
    /// In-document anchor / fragment jump.
    JumpAnchor,
    /// Find-in-page-style within-document search jump.
    InPageSearchJump,
    /// Imported from another browser's history database.
    ImportedHistory,
}

/// Persisted edge.
#[derive(
    Archive, Serialize, Deserialize, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize,
)]
pub struct PersistedEdge {
    pub from_node_id: String,
    pub to_node_id: String,
    #[serde(default)]
    pub families: Vec<PersistedEdgeFamily>,
    #[serde(default)]
    pub semantic: Option<PersistedSemanticEdgeData>,
    #[serde(default)]
    pub traversal: Option<PersistedTraversalEdgeData>,
    #[serde(default)]
    pub containment: Option<PersistedContainmentEdgeData>,
    #[serde(default)]
    pub arrangement: Option<PersistedArrangementEdgeData>,
    #[serde(default)]
    pub imported: Option<PersistedImportedEdgeData>,
    #[serde(default)]
    pub provenance: Option<PersistedProvenanceEdgeData>,
    /// Canonical all-family records. None selects the legacy sidecar decoder;
    /// Some(empty) is an explicitly empty list, never a request to infer facts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assertions: Option<Vec<PersistedSemanticStatement>>,
}

impl PersistedEdge {
    pub fn retained_assertions(&self) -> &[PersistedSemanticStatement] {
        self.assertions.as_deref().unwrap_or_else(|| {
            self.semantic
                .as_ref()
                .map(|data| data.statements.as_slice())
                .unwrap_or(&[])
        })
    }
    pub fn retained_assertions_mut(&mut self) -> &mut [PersistedSemanticStatement] {
        match self.assertions.as_mut() {
            Some(records) => records.as_mut_slice(),
            None => self
                .semantic
                .as_mut()
                .map(|data| data.statements.as_mut_slice())
                .unwrap_or(&mut []),
        }
    }
    pub fn has_legacy_assertion_data(&self) -> bool {
        self.semantic.as_ref().is_some_and(|data| {
            !data.statements.is_empty()
                || !data.sub_kinds.is_empty()
                || data.predicate.is_some()
                || data.label.is_some()
        }) || self
            .containment
            .as_ref()
            .is_some_and(|data| !data.sub_kinds.is_empty())
            || self
                .arrangement
                .as_ref()
                .is_some_and(|data| !data.sub_kinds.is_empty())
            || self
                .imported
                .as_ref()
                .is_some_and(|data| !data.sub_kinds.is_empty())
            || self
                .provenance
                .as_ref()
                .is_some_and(|data| !data.sub_kinds.is_empty())
    }
}

#[cfg(test)]
#[path = "persistence_edge/statement_serialization_tests.rs"]
mod statement_serialization_tests;

#[path = "persistence_edge/serialization.rs"]
mod serialization;
