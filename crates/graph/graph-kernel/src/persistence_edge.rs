// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Edge persistence types — family/sub-kind enums, per-family data structs,
//! and the edge container. Re-exported through [`crate::persistence`] to keep
//! `persistence.rs` under the per-file ceiling.

use rkyv::{Archive, Deserialize, Serialize};

use crate::types::GraphScope;

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
}

#[derive(Archive, Serialize, Deserialize, Clone, Debug, PartialEq, serde::Deserialize)]
#[rkyv(derive(Debug, PartialEq))]
pub struct PersistedSemanticStatement {
    pub statement_id: String,
    pub predicate: String,
    #[serde(default)]
    pub recognized_sub_kind: Option<PersistedSemanticSubKind>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub graph_scope: GraphScope,
    #[serde(default)]
    pub provenance_iri: Option<String>,
    #[serde(default)]
    pub asserted_at_ms: Option<u64>,
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
        } else {
            7
        };
        // Binary snapshots retain their complete legacy sequence shape.
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
        assert_eq!(rich.as_object().unwrap().len(), 7);
        assert_eq!(rich["statement_id"], "");
        assert_eq!(rich["label"], "");
        assert_eq!(rich["provenance_iri"], "");
        assert_eq!(rich["asserted_at_ms"], 0);
        for value in cases {
            let bytes = serde_json::to_vec(&value).unwrap();
            assert_eq!(
                serde_json::from_slice::<PersistedSemanticStatement>(&bytes).unwrap(),
                value
            );
        }
    }

    #[test]
    fn statement_binary_bytes_match_the_complete_legacy_shape() {
        #[derive(serde::Serialize)]
        struct Complete<'a> {
            statement_id: &'a String,
            predicate: &'a String,
            recognized_sub_kind: &'a Option<PersistedSemanticSubKind>,
            label: &'a Option<String>,
            graph_scope: &'a GraphScope,
            provenance_iri: &'a Option<String>,
            asserted_at_ms: &'a Option<u64>,
        }
        for value in cases() {
            let old = Complete {
                statement_id: &value.statement_id,
                predicate: &value.predicate,
                recognized_sub_kind: &value.recognized_sub_kind,
                label: &value.label,
                graph_scope: &value.graph_scope,
                provenance_iri: &value.provenance_iri,
                asserted_at_ms: &value.asserted_at_ms,
            };
            let bytes = postcard::to_allocvec(&value).unwrap();
            assert_eq!(bytes, postcard::to_allocvec(&old).unwrap());
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
}
