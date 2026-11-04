// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

impl RelationKind {
    /// The canonical predicate IRI for a recognized relation kind.
    ///
    /// Semantic predicates retain their established byte-for-byte IRIs.
    /// The other sub-kind families use an explicit family-qualified local
    /// name so their predicate identity is stable across refactors. Traversal
    /// and open predicates do not name one fixed predicate here.
    pub fn predicate_iri(self) -> Option<&'static str> {
        use super::super::edge_data::predicate_iri as semantic_predicate_iri;

        match self {
            RelationKind::Semantic(sub_kind) => Some(semantic_predicate_iri(sub_kind)),
            RelationKind::OpenPredicate | RelationKind::Traversal => None,
            RelationKind::Containment(sub_kind) => Some(match sub_kind {
                ContainmentSubKind::UrlPath => "https://mere.computer/ns/rel#containment.url-path",
                ContainmentSubKind::Domain => "https://mere.computer/ns/rel#containment.domain",
                ContainmentSubKind::FileSystem => {
                    "https://mere.computer/ns/rel#containment.filesystem"
                },
                ContainmentSubKind::UserFolder => {
                    "https://mere.computer/ns/rel#containment.user-folder"
                },
                ContainmentSubKind::ClipSource => {
                    "https://mere.computer/ns/rel#containment.clip-source"
                },
                ContainmentSubKind::NotebookSection => {
                    "https://mere.computer/ns/rel#containment.notebook-section"
                },
                ContainmentSubKind::CollectionMember => {
                    "https://mere.computer/ns/rel#containment.collection-member"
                },
            }),
            RelationKind::Arrangement(sub_kind) => Some(match sub_kind {
                ArrangementSubKind::FrameMember => {
                    "https://mere.computer/ns/rel#arrangement.frame-member"
                },
                ArrangementSubKind::TileGroup => {
                    "https://mere.computer/ns/rel#arrangement.tile-group"
                },
                ArrangementSubKind::SplitPair => {
                    "https://mere.computer/ns/rel#arrangement.split-pair"
                },
            }),
            RelationKind::Imported(sub_kind) => Some(match sub_kind {
                ImportedSubKind::BookmarkFolder => {
                    "https://mere.computer/ns/rel#imported.bookmark-folder"
                },
                ImportedSubKind::HistoryImport => {
                    "https://mere.computer/ns/rel#imported.history-import"
                },
                ImportedSubKind::SessionImport => {
                    "https://mere.computer/ns/rel#imported.session-import"
                },
                ImportedSubKind::RssMembership => {
                    "https://mere.computer/ns/rel#imported.rss-membership"
                },
                ImportedSubKind::FileSystemImport => {
                    "https://mere.computer/ns/rel#imported.filesystem-import"
                },
                ImportedSubKind::ArchiveMembership => {
                    "https://mere.computer/ns/rel#imported.archive-membership"
                },
                ImportedSubKind::SharedCollection => {
                    "https://mere.computer/ns/rel#imported.shared-collection"
                },
            }),
            RelationKind::Provenance(sub_kind) => Some(match sub_kind {
                ProvenanceSubKind::ClippedFrom => {
                    "https://mere.computer/ns/rel#provenance.clipped-from"
                },
                ProvenanceSubKind::ExcerptedFrom => {
                    "https://mere.computer/ns/rel#provenance.excerpted-from"
                },
                ProvenanceSubKind::SummarizedFrom => {
                    "https://mere.computer/ns/rel#provenance.summarized-from"
                },
                ProvenanceSubKind::TranslatedFrom => {
                    "https://mere.computer/ns/rel#provenance.translated-from"
                },
                ProvenanceSubKind::RewrittenFrom => {
                    "https://mere.computer/ns/rel#provenance.rewritten-from"
                },
                ProvenanceSubKind::GeneratedFrom => {
                    "https://mere.computer/ns/rel#provenance.generated-from"
                },
                ProvenanceSubKind::ExtractedFrom => {
                    "https://mere.computer/ns/rel#provenance.extracted-from"
                },
                ProvenanceSubKind::ImportedFromSource => {
                    "https://mere.computer/ns/rel#provenance.imported-from-source"
                },
                ProvenanceSubKind::CopiedFrom => {
                    "https://mere.computer/ns/rel#provenance.copied-from"
                },
                ProvenanceSubKind::MergedFrom => {
                    "https://mere.computer/ns/rel#provenance.merged-from"
                },
            }),
        }
    }

    /// Decode a canonical predicate IRI produced by [`Self::predicate_iri`].
    /// Unknown and open predicates remain outside the closed relation-kind
    /// vocabulary.
    pub fn from_predicate_iri(iri: &str) -> Option<Self> {
        use super::super::edge_data::sub_kind_from_iri as semantic_sub_kind_from_iri;

        if let Some(sub_kind) = semantic_sub_kind_from_iri(iri) {
            return Some(RelationKind::Semantic(sub_kind));
        }

        [
            ContainmentSubKind::UrlPath,
            ContainmentSubKind::Domain,
            ContainmentSubKind::FileSystem,
            ContainmentSubKind::UserFolder,
            ContainmentSubKind::ClipSource,
            ContainmentSubKind::NotebookSection,
            ContainmentSubKind::CollectionMember,
        ]
        .into_iter()
        .map(RelationKind::Containment)
        .chain(
            [
                ArrangementSubKind::FrameMember,
                ArrangementSubKind::TileGroup,
                ArrangementSubKind::SplitPair,
            ]
            .into_iter()
            .map(RelationKind::Arrangement),
        )
        .chain(
            [
                ImportedSubKind::BookmarkFolder,
                ImportedSubKind::HistoryImport,
                ImportedSubKind::SessionImport,
                ImportedSubKind::RssMembership,
                ImportedSubKind::FileSystemImport,
                ImportedSubKind::ArchiveMembership,
                ImportedSubKind::SharedCollection,
            ]
            .into_iter()
            .map(RelationKind::Imported),
        )
        .chain(
            [
                ProvenanceSubKind::ClippedFrom,
                ProvenanceSubKind::ExcerptedFrom,
                ProvenanceSubKind::SummarizedFrom,
                ProvenanceSubKind::TranslatedFrom,
                ProvenanceSubKind::RewrittenFrom,
                ProvenanceSubKind::GeneratedFrom,
                ProvenanceSubKind::ExtractedFrom,
                ProvenanceSubKind::ImportedFromSource,
                ProvenanceSubKind::CopiedFrom,
                ProvenanceSubKind::MergedFrom,
            ]
            .into_iter()
            .map(RelationKind::Provenance),
        )
        .find(|kind| kind.predicate_iri() == Some(iri))
    }

    /// Project to the relation's family. Pure function — no payload
    /// access required.
    pub fn family(self) -> EdgeFamily {
        match self {
            RelationKind::Semantic(_) | RelationKind::OpenPredicate => EdgeFamily::Semantic,
            RelationKind::Traversal => EdgeFamily::Traversal,
            RelationKind::Containment(_) => EdgeFamily::Containment,
            RelationKind::Arrangement(_) => EdgeFamily::Arrangement,
            RelationKind::Imported(_) => EdgeFamily::Imported,
            RelationKind::Provenance(_) => EdgeFamily::Provenance,
        }
    }

    /// Encode this relation as an opaque `u32` tag for transport
    /// through layers that can't depend on `kernel` (e.g.
    /// `graph-canvas::CanvasEdge::tag` / `HitProxy::Edge::tag`).
    /// The top byte is the family ordinal (0..5); the bottom three
    /// bytes are the sub-kind ordinal within the family.
    /// [`OpenPredicate`](Self::OpenPredicate) keeps the Semantic family
    /// byte and uses the all-ones sub-ordinal
    /// ([`OPEN_PREDICATE_TAG_SUB`]) as its sentinel, so a consumer that
    /// reads only the family byte still sees Semantic.
    /// [`Self::from_tag`] is the inverse.
    pub fn tag(self) -> u32 {
        let (family, sub) = match self {
            RelationKind::Semantic(sk) => (0u32, sk as u32),
            RelationKind::OpenPredicate => (0, OPEN_PREDICATE_TAG_SUB),
            RelationKind::Traversal => (1, 0),
            RelationKind::Containment(sk) => (2, sk as u32),
            RelationKind::Arrangement(sk) => (3, sk as u32),
            RelationKind::Imported(sk) => (4, sk as u32),
            RelationKind::Provenance(sk) => (5, sk as u32),
        };
        (family << 24) | (sub & 0x00ff_ffff)
    }

    /// Decode a `u32` tag produced by [`Self::tag`]. Returns `None`
    /// for tags that don't correspond to a known relation (unknown
    /// family byte or sub-kind ordinal out of range).
    pub fn from_tag(tag: u32) -> Option<Self> {
        let family = tag >> 24;
        let sub = tag & 0x00ff_ffff;
        match family {
            0 if sub == OPEN_PREDICATE_TAG_SUB => Some(RelationKind::OpenPredicate),
            0 => SemanticSubKind::from_repr(sub as usize).map(RelationKind::Semantic),
            1 => (sub == 0).then_some(RelationKind::Traversal),
            2 => ContainmentSubKind::from_repr(sub as usize).map(RelationKind::Containment),
            3 => ArrangementSubKind::from_repr(sub as usize).map(RelationKind::Arrangement),
            4 => ImportedSubKind::from_repr(sub as usize).map(RelationKind::Imported),
            5 => ProvenanceSubKind::from_repr(sub as usize).map(RelationKind::Provenance),
            _ => None,
        }
    }
}
