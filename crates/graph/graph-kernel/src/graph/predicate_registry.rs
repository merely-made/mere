// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Fixed placement catalog. Effective custom placement must also consult the
//! mere's durable declarations; these defaults do not resolve those declarations.

use super::resource::TAGGED_WITH_IRI;
use super::{
    ArrangementSubKind, ContainmentSubKind, ImportedSubKind, ProvenanceSubKind, RelationKind,
    SemanticSubKind, sub_kind_from_iri,
};

/// The owning graph stratum of a relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GraphStratum {
    Resource,
    Surface,
}

/// Built-in placement by relation nature. `OpenPredicate` supplies only the
/// unfamiliar-predicate default, before any per-mere declaration is consulted.
pub const fn built_in_relation_stratum(kind: RelationKind) -> GraphStratum {
    match kind {
        RelationKind::Semantic(kind) => match kind {
            SemanticSubKind::UserGrouped => GraphStratum::Surface,
            SemanticSubKind::Hyperlink
            | SemanticSubKind::AgentDerived
            | SemanticSubKind::Cites
            | SemanticSubKind::Quotes
            | SemanticSubKind::Summarizes
            | SemanticSubKind::Elaborates
            | SemanticSubKind::ExampleOf
            | SemanticSubKind::Supports
            | SemanticSubKind::Contradicts
            | SemanticSubKind::Questions
            | SemanticSubKind::SameEntityAs
            | SemanticSubKind::DuplicateOf
            | SemanticSubKind::CanonicalMirrorOf
            | SemanticSubKind::DependsOn
            | SemanticSubKind::Blocks
            | SemanticSubKind::NextStep => GraphStratum::Resource,
        },
        RelationKind::Provenance(kind) => match kind {
            ProvenanceSubKind::CopiedFrom => GraphStratum::Surface,
            ProvenanceSubKind::ClippedFrom
            | ProvenanceSubKind::ExcerptedFrom
            | ProvenanceSubKind::SummarizedFrom
            | ProvenanceSubKind::TranslatedFrom
            | ProvenanceSubKind::RewrittenFrom
            | ProvenanceSubKind::GeneratedFrom
            | ProvenanceSubKind::ExtractedFrom
            | ProvenanceSubKind::ImportedFromSource => GraphStratum::Resource,
        },
        RelationKind::Imported(kind) => match kind {
            ImportedSubKind::BookmarkFolder
            | ImportedSubKind::HistoryImport
            | ImportedSubKind::SessionImport => GraphStratum::Surface,
            ImportedSubKind::RssMembership
            | ImportedSubKind::FileSystemImport
            | ImportedSubKind::ArchiveMembership
            | ImportedSubKind::SharedCollection => GraphStratum::Resource,
        },
        RelationKind::Containment(kind) => match kind {
            ContainmentSubKind::UserFolder
            | ContainmentSubKind::NotebookSection
            | ContainmentSubKind::CollectionMember => GraphStratum::Surface,
            ContainmentSubKind::UrlPath
            | ContainmentSubKind::Domain
            | ContainmentSubKind::FileSystem
            | ContainmentSubKind::ClipSource => GraphStratum::Resource,
        },
        RelationKind::Arrangement(
            ArrangementSubKind::FrameMember
            | ArrangementSubKind::TileGroup
            | ArrangementSubKind::SplitPair,
        )
        | RelationKind::Traversal => GraphStratum::Surface,
        RelationKind::OpenPredicate => GraphStratum::Resource,
    }
}

/// Fixed placement for canonical built-in predicate IRIs. Unfamiliar strings
/// return `None`; their effective placement can include a custom declaration.
pub fn built_in_predicate_stratum(iri: &str) -> Option<GraphStratum> {
    if iri == TAGGED_WITH_IRI {
        return Some(GraphStratum::Resource);
    }
    sub_kind_from_iri(iri).map(|kind| built_in_relation_stratum(RelationKind::Semantic(kind)))
}

/// Predicate placement before per-mere declarations are applied. This fallback
/// must not replace effective placement for registered custom predicates.
pub fn default_predicate_stratum(iri: &str) -> GraphStratum {
    built_in_predicate_stratum(iri).unwrap_or(GraphStratum::Resource)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::predicate_iri;
    use strum::IntoEnumIterator;

    #[test]
    fn all_recognized_kinds_match_the_settled_placement_table() {
        let surfaces = [
            RelationKind::Semantic(SemanticSubKind::UserGrouped),
            RelationKind::Provenance(ProvenanceSubKind::CopiedFrom),
            RelationKind::Imported(ImportedSubKind::BookmarkFolder),
            RelationKind::Imported(ImportedSubKind::HistoryImport),
            RelationKind::Imported(ImportedSubKind::SessionImport),
            RelationKind::Containment(ContainmentSubKind::UserFolder),
            RelationKind::Containment(ContainmentSubKind::NotebookSection),
            RelationKind::Containment(ContainmentSubKind::CollectionMember),
            RelationKind::Arrangement(ArrangementSubKind::FrameMember),
            RelationKind::Arrangement(ArrangementSubKind::TileGroup),
            RelationKind::Arrangement(ArrangementSubKind::SplitPair),
        ];
        let kinds: Vec<_> = SemanticSubKind::iter()
            .map(RelationKind::Semantic)
            .chain(ProvenanceSubKind::iter().map(RelationKind::Provenance))
            .chain(ImportedSubKind::iter().map(RelationKind::Imported))
            .chain(ContainmentSubKind::iter().map(RelationKind::Containment))
            .chain(ArrangementSubKind::iter().map(RelationKind::Arrangement))
            .collect();
        assert_eq!(kinds.len(), 43);
        let mut resource_count = 0;
        let mut surface_count = 0;
        for kind in kinds {
            let expected = if surfaces.contains(&kind) {
                surface_count += 1;
                GraphStratum::Surface
            } else {
                resource_count += 1;
                GraphStratum::Resource
            };
            assert_eq!(built_in_relation_stratum(kind), expected, "{kind:?}");
        }
        assert_eq!((resource_count, surface_count), (32, 11));
        assert_eq!(
            built_in_relation_stratum(RelationKind::Traversal),
            GraphStratum::Surface
        );
        assert_eq!(
            built_in_relation_stratum(RelationKind::OpenPredicate),
            GraphStratum::Resource
        );
    }

    #[test]
    fn canonical_predicates_tags_and_open_defaults_have_distinct_controls() {
        assert_eq!(SemanticSubKind::iter().count(), 17);
        for kind in SemanticSubKind::iter() {
            let expected = if kind == SemanticSubKind::UserGrouped {
                GraphStratum::Surface
            } else {
                GraphStratum::Resource
            };
            assert_eq!(
                built_in_predicate_stratum(predicate_iri(kind)),
                Some(expected)
            );
            assert_eq!(default_predicate_stratum(predicate_iri(kind)), expected);
        }
        for kind in [
            SemanticSubKind::AgentDerived,
            SemanticSubKind::DependsOn,
            SemanticSubKind::Blocks,
            SemanticSubKind::NextStep,
        ] {
            assert_eq!(
                built_in_relation_stratum(RelationKind::Semantic(kind)),
                GraphStratum::Resource
            );
        }
        assert_eq!(
            built_in_relation_stratum(RelationKind::Imported(ImportedSubKind::SharedCollection)),
            GraphStratum::Resource
        );
        assert_eq!(
            built_in_predicate_stratum(TAGGED_WITH_IRI),
            Some(GraphStratum::Resource)
        );
        for unfamiliar in [
            "https://foreign.test/predicate",
            "",
            "arbitrary predicate",
            "https://mere.computer/ns/rel#User-grouped",
        ] {
            assert_eq!(built_in_predicate_stratum(unfamiliar), None);
            assert_eq!(
                default_predicate_stratum(unfamiliar),
                GraphStratum::Resource
            );
        }
    }
}
