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

impl serde::Serialize for PersistedSemanticStatement {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let human = serializer.is_human_readable();
        let scoped = self.graph_scope != GraphScope::Default;
        let fields = if human {
            2 + usize::from(self.recognized_sub_kind.is_some())
                + usize::from(self.label.is_some())
                + usize::from(scoped)
                + usize::from(self.provenance_iri.is_some())
                + usize::from(self.asserted_at_ms.is_some())
                + usize::from(self.first_asserted_ms.is_some() || self.asserted_at_ms.is_some())
                + usize::from(self.withdrawn.is_some())
                + usize::from(!self.basis.is_empty() || self.label.is_some())
        } else {
            10
        };
        // Positional serializers receive every field in the current shape.
        let mut state = serializer.serialize_struct("PersistedSemanticStatement", fields)?;
        state.serialize_field("statement_id", &self.statement_id)?;
        state.serialize_field("predicate", &self.predicate)?;
        if !human || self.recognized_sub_kind.is_some() {
            state.serialize_field("recognized_sub_kind", &self.recognized_sub_kind)?;
        }
        if !human || self.label.is_some() {
            state.serialize_field("label", &self.label)?;
        }
        if !human || scoped {
            state.serialize_field("graph_scope", &self.graph_scope)?;
        }
        if !human || self.provenance_iri.is_some() {
            state.serialize_field("provenance_iri", &self.provenance_iri)?;
        }
        if !human || self.asserted_at_ms.is_some() {
            state.serialize_field("asserted_at_ms", &self.asserted_at_ms)?;
        }
        if !human || self.first_asserted_ms.is_some() || self.asserted_at_ms.is_some() {
            state.serialize_field("first_asserted_ms", &self.first_asserted_ms)?;
        }
        if !human || self.withdrawn.is_some() {
            state.serialize_field("withdrawn", &self.withdrawn)?;
        }
        if !human || !self.basis.is_empty() || self.label.is_some() {
            state.serialize_field("basis", &self.basis)?;
        }
        state.end()
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
        self.assertions.as_deref().unwrap_or_else(|| self.semantic.as_ref()
            .map(|data| data.statements.as_slice()).unwrap_or(&[]))
    }
    pub fn retained_assertions_mut(&mut self) -> &mut [PersistedSemanticStatement] {
        match self.assertions.as_mut() {
            Some(records) => records.as_mut_slice(),
            None => self.semantic.as_mut().map(|data| data.statements.as_mut_slice()).unwrap_or(&mut []),
        }
    }
    pub fn has_legacy_assertion_data(&self) -> bool {
        self.semantic.as_ref().is_some_and(|data| !data.statements.is_empty() || !data.sub_kinds.is_empty()
            || data.predicate.is_some() || data.label.is_some())
            || self.containment.as_ref().is_some_and(|data| !data.sub_kinds.is_empty())
            || self.arrangement.as_ref().is_some_and(|data| !data.sub_kinds.is_empty())
            || self.imported.as_ref().is_some_and(|data| !data.sub_kinds.is_empty())
            || self.provenance.as_ref().is_some_and(|data| !data.sub_kinds.is_empty())
    }
}

#[cfg(test)]
mod statement_serialization_tests {
    use super::*;

    fn cases() -> Vec<PersistedSemanticStatement> {
        let plain = PersistedSemanticStatement {
            statement_id: "held\nassertion".into(),
            predicate: "urn:predicate:held".into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: GraphScope::Default,
            provenance_iri: Some("https://author.test/".into()),
            asserted_at_ms: None,
            first_asserted_ms: None,
            withdrawn: None,
            basis: Default::default(),
        };
        let mut legacy = plain.clone();
        legacy.provenance_iri = None;
        let mut marker = plain.clone();
        marker.provenance_iri = Some(crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());
        let mut rich = plain.clone();
        rich.statement_id = String::new();
        rich.recognized_sub_kind = Some(PersistedSemanticSubKind::Cites);
        rich.label = Some(String::new());
        rich.graph_scope = GraphScope::Custom("urn:graph:held".into());
        rich.provenance_iri = Some(String::new());
        rich.asserted_at_ms = Some(0);
        rich.first_asserted_ms = Some(0);
        rich.basis.note = Some(String::new());
        vec![plain, legacy, marker, rich]
    }

    #[test]
    fn statement_json_omits_absent_metadata_and_preserves_explicit_values() {
        let cases = cases();
        let plain = serde_json::to_value(&cases[0]).unwrap();
        assert_eq!(plain.as_object().unwrap().len(), 3);
        assert_eq!(plain["statement_id"], "held\nassertion");
        assert_eq!(plain["provenance_iri"], "https://author.test/");
        for absent in [
            "recognized_sub_kind",
            "label",
            "graph_scope",
            "asserted_at_ms",
        ] {
            assert!(plain.get(absent).is_none());
        }
        let legacy = serde_json::to_value(&cases[1]).unwrap();
        assert!(legacy.get("provenance_iri").is_none());
        assert_eq!(
            serde_json::from_value::<PersistedSemanticStatement>(legacy).unwrap(),
            cases[1]
        );
        let marker = serde_json::to_value(&cases[2]).unwrap();
        assert_eq!(
            marker["provenance_iri"],
            crate::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI
        );
        let rich = serde_json::to_value(&cases[3]).unwrap();
        assert_eq!(rich.as_object().unwrap().len(), 9);
        assert_eq!(rich["statement_id"], "");
        assert_eq!(rich["label"], "");
        assert_eq!(rich["provenance_iri"], "");
        assert_eq!(rich["asserted_at_ms"], 0);
        assert_eq!(rich["first_asserted_ms"], 0);
        for value in cases {
            let bytes = serde_json::to_vec(&value).unwrap();
            assert_eq!(
                serde_json::from_slice::<PersistedSemanticStatement>(&bytes).unwrap(),
                value
            );
        }
    }

    #[test]
    fn statement_binary_current_shape_round_trips() {
        for value in cases() {
            let bytes = postcard::to_allocvec(&value).unwrap();
            assert_eq!(
                postcard::from_bytes::<PersistedSemanticStatement>(&bytes).unwrap(),
                value
            );
            let archive = rkyv::to_bytes::<rkyv::rancor::Error>(&value).unwrap();
            assert_eq!(
                rkyv::from_bytes::<PersistedSemanticStatement, rkyv::rancor::Error>(&archive)
                    .unwrap(),
                value
            );
        }
    }

    #[test]
    fn legacy_postcard_sequence_is_not_a_supported_store_format() {
        #[derive(serde::Serialize)]
        struct Legacy {
            statement_id: String,
            predicate: String,
            recognized_sub_kind: Option<PersistedSemanticSubKind>,
            label: Option<String>,
            graph_scope: GraphScope,
            provenance_iri: Option<String>,
            asserted_at_ms: Option<u64>,
        }

        let legacy = Legacy {
            statement_id: "legacy-sequence".into(),
            predicate: "urn:p".into(),
            recognized_sub_kind: None,
            label: None,
            graph_scope: GraphScope::Default,
            provenance_iri: Some("urn:source".into()),
            asserted_at_ms: Some(42),
        };
        let bytes = postcard::to_allocvec(&legacy).unwrap();
        assert!(postcard::from_bytes::<PersistedSemanticStatement>(&bytes).is_err());
    }

    #[test]
    fn legacy_time_falls_back_only_when_first_time_is_absent() {
        let legacy = serde_json::json!({
            "statement_id": "legacy",
            "predicate": "urn:p",
            "asserted_at_ms": 42
        });
        let decoded: PersistedSemanticStatement = serde_json::from_value(legacy).unwrap();
        assert_eq!(decoded.first_asserted_ms, Some(42));
        assert_eq!(decoded.asserted_at_ms, Some(42));

        let explicit_unknown = serde_json::json!({
            "statement_id": "unknown-first",
            "predicate": "urn:p",
            "first_asserted_ms": null,
            "asserted_at_ms": 42
        });
        let decoded: PersistedSemanticStatement = serde_json::from_value(explicit_unknown).unwrap();
        assert_eq!(decoded.first_asserted_ms, None);
        assert_eq!(decoded.asserted_at_ms, Some(42));
        let encoded = serde_json::to_value(decoded).unwrap();
        assert!(encoded.get("first_asserted_ms").is_some());
        assert!(encoded["first_asserted_ms"].is_null());
    }
}
