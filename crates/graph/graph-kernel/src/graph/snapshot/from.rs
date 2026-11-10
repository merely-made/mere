// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `Graph::from_snapshot` — reconstruct the runtime graph from a
//! persisted [`GraphSnapshot`].
//!
//! Extracted from `graph/snapshot.rs` per the 2026-05-11 kernel
//! decomposition pass.

use euclid::default::Point2D;
use std::collections::BTreeMap;
use uuid::Uuid;

use super::super::import_records::import_records_from_legacy_provenance;
use super::super::*;
use crate::address::{address_from_url, detect_mime};
use crate::persistence::{
    GraphSnapshot, PersistedArrangementSubKind, PersistedContainmentSubKind,
    PersistedCouplingResponse, PersistedEdge, PersistedFieldExtent, PersistedFieldLifecycle,
    PersistedImportedSubKind, PersistedNavigationTrigger, PersistedNodeSelector,
    PersistedProvenanceSubKind, PersistedSemanticSubKind,
};

fn semantic_sub_kind(sub_kind: PersistedSemanticSubKind) -> SemanticSubKind {
    match sub_kind {
        PersistedSemanticSubKind::Hyperlink => SemanticSubKind::Hyperlink,
        PersistedSemanticSubKind::UserGrouped => SemanticSubKind::UserGrouped,
        PersistedSemanticSubKind::AgentDerived => SemanticSubKind::AgentDerived,
        PersistedSemanticSubKind::Cites => SemanticSubKind::Cites,
        PersistedSemanticSubKind::Quotes => SemanticSubKind::Quotes,
        PersistedSemanticSubKind::Summarizes => SemanticSubKind::Summarizes,
        PersistedSemanticSubKind::Elaborates => SemanticSubKind::Elaborates,
        PersistedSemanticSubKind::ExampleOf => SemanticSubKind::ExampleOf,
        PersistedSemanticSubKind::Supports => SemanticSubKind::Supports,
        PersistedSemanticSubKind::Contradicts => SemanticSubKind::Contradicts,
        PersistedSemanticSubKind::Questions => SemanticSubKind::Questions,
        PersistedSemanticSubKind::SameEntityAs => SemanticSubKind::SameEntityAs,
        PersistedSemanticSubKind::DuplicateOf => SemanticSubKind::DuplicateOf,
        PersistedSemanticSubKind::CanonicalMirrorOf => SemanticSubKind::CanonicalMirrorOf,
        PersistedSemanticSubKind::DependsOn => SemanticSubKind::DependsOn,
        PersistedSemanticSubKind::Blocks => SemanticSubKind::Blocks,
        PersistedSemanticSubKind::NextStep => SemanticSubKind::NextStep,
    }
}

/// Before canonical assertion records existed, Surface UrlPath and Domain
/// sidecars were compatibility projections of current URLs, not authored
/// claims. Mark just those legacy Surface rows so the ordinary session loader
/// can replace them from current source data without treating canonical or
/// unknown Resource assertions as rule output.
fn mark_legacy_containment_sidecars_as_derived(payload: &mut EdgePayload, edge: &PersistedEdge) {
    let Some(containment) = &edge.containment else {
        return;
    };
    let has_derived_sidecar = containment.sub_kinds.iter().any(|kind| {
        matches!(
            kind,
            PersistedContainmentSubKind::UrlPath | PersistedContainmentSubKind::Domain
        )
    });
    if !has_derived_sidecar {
        return;
    }
    for statement in &mut payload.assertions.statements {
        if statement.provenance_iri.as_deref() != Some(edge_data::UNKNOWN_LEGACY_ASSERTER_IRI)
            || statement.basis.method.is_some()
            || !matches!(
                statement.relation_kind(),
                RelationKind::Containment(ContainmentSubKind::UrlPath | ContainmentSubKind::Domain)
            )
        {
            continue;
        }
        statement.basis.method = Some(crate::types::AssertionMethod {
            iri: "urn:mere:method:derived-containment".into(),
            version: Some("1".into()),
        });
    }
    payload.assertions.rebuild_compat();
}

impl Graph {
    /// Merge one persisted edge into a surface pair, retaining revision behavior.
    pub(crate) fn restore_persisted_edge(
        &mut self,
        from: NodeKey,
        to: NodeKey,
        pedge: &PersistedEdge,
    ) {
        let existing = self.find_edge_key(from, to);
        let mut payload = existing
            .and_then(|key| self.inner.edge(key))
            .cloned()
            .unwrap_or_else(EdgePayload::new);
        let structural_changes = restore_persisted_payload(&mut payload, pedge);
        if pedge.semantic.is_some() || pedge.traversal.is_some() || !payload.is_empty() {
            match existing {
                Some(key) => *self.get_edge_mut(key).expect("existing edge") = payload,
                None => {
                    self.connect_surface(from, to, payload);
                },
            }
        }
        for _ in 0..structural_changes {
            self.bump_revision();
        }
    }
}

/// Decode a standalone pair without changing carried assertion metadata.
pub(crate) fn payload_from_persisted(pedge: &PersistedEdge) -> EdgePayload {
    payload_from_persisted_at(pedge, GraphStratum::Surface, 0)
}

pub(crate) fn payload_from_persisted_at(
    pedge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
) -> EdgePayload {
    let mut payload = EdgePayload::new();
    restore_payload_at(&mut payload, pedge, stratum, ordinal);
    payload
}

/// Merge persisted sidecars; report structural insertions for surface revisions.
pub(crate) fn restore_persisted_payload(payload: &mut EdgePayload, pedge: &PersistedEdge) -> usize {
    restore_payload_at(payload, pedge, GraphStratum::Surface, 0)
}

fn restore_payload_at(
    payload: &mut EdgePayload,
    pedge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
) -> usize {
    let mut structural_changes = 0;
    if let Some(records) = &pedge.assertions {
        let mut ids: std::collections::BTreeSet<_> = payload
            .assertions()
            .iter()
            .map(|s| s.statement_id.clone())
            .collect();
        for record in records {
            if ids.insert(record.statement_id.clone()) {
                let mut record = runtime_assertion(record);
                record.normalize_basis_compat();
                payload.assertions.statements.push(record);
                structural_changes += 1;
            }
        }
    }
    let preserved_ids: std::collections::BTreeSet<_> = payload
        .assertions()
        .iter()
        .map(|s| s.statement_id.clone())
        .chain(
            pedge
                .semantic
                .as_ref()
                .into_iter()
                .flat_map(|data| &data.statements)
                .map(|s| s.statement_id.clone()),
        )
        .collect();
    // Canonical records are authoritative. Checked load refuses a nonempty
    // legacy sidecar alongside them instead of inventing a merge precedence.
    if pedge.assertions.is_none() {
        if let Some(semantic) = &pedge.semantic {
            if !semantic.statements.is_empty() {
                for statement in &semantic.statements {
                    let _ = payload.push_persisted_semantic_statement(SemanticStatement {
                        statement_id: statement.statement_id.clone(),
                        predicate: statement.predicate.clone(),
                        recognized_sub_kind: statement.recognized_sub_kind.map(semantic_sub_kind),
                        label: statement.label.clone(),
                        basis: statement.basis.clone(),
                        graph_scope: statement.graph_scope.clone(),
                        provenance_iri: statement.provenance_iri.clone(),
                        first_asserted_ms: statement.first_asserted_ms,
                        asserted_at_ms: statement.asserted_at_ms,
                        withdrawn: statement.withdrawn.clone(),
                    });
                }
            } else {
                for sub_kind in &semantic.sub_kinds {
                    payload.assert_semantic_statement(SemanticStatementSpec {
                        predicate: predicate_iri(semantic_sub_kind(sub_kind.clone())).into(),
                        recognized_sub_kind: Some(semantic_sub_kind(sub_kind.clone())),
                        label: semantic.label.clone(),
                        graph_scope: crate::types::GraphScope::Default,
                        provenance_iri: Some(edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into()),
                        asserted_at_ms: None,
                        basis: Default::default(),
                    });
                }
                if semantic.predicate.is_some() {
                    payload.set_semantic_predicate(semantic.predicate.clone());
                }
            }
        }
        if let Some(arrangement) = &pedge.arrangement {
            for sub_kind in &arrangement.sub_kinds {
                let assertion = match sub_kind {
                    PersistedArrangementSubKind::FrameMember => EdgeAssertion::Arrangement {
                        sub_kind: ArrangementSubKind::FrameMember,
                    },
                    PersistedArrangementSubKind::TileGroup => EdgeAssertion::Arrangement {
                        sub_kind: ArrangementSubKind::TileGroup,
                    },
                    PersistedArrangementSubKind::SplitPair => EdgeAssertion::Arrangement {
                        sub_kind: ArrangementSubKind::SplitPair,
                    },
                    PersistedArrangementSubKind::TabNeighbor
                    | PersistedArrangementSubKind::ActiveTab
                    | PersistedArrangementSubKind::PinnedInFrame => continue,
                };
                structural_changes += usize::from(payload.assert_relation(assertion));
            }
        }
        if let Some(containment) = &pedge.containment {
            for sub_kind in &containment.sub_kinds {
                // Every persisted sub-kind is restored. `UrlPath` and
                // `Domain` are re-derived below by
                // `rebuild_derived_containment_relations`, but the
                // authored ones exist only here: dropping any of them
                // loses user data. Matched exhaustively so a new
                // variant is a compile error rather than a silent
                // omission.
                let sub_kind = match sub_kind {
                    PersistedContainmentSubKind::UrlPath => ContainmentSubKind::UrlPath,
                    PersistedContainmentSubKind::Domain => ContainmentSubKind::Domain,
                    PersistedContainmentSubKind::FileSystem => ContainmentSubKind::FileSystem,
                    PersistedContainmentSubKind::UserFolder => ContainmentSubKind::UserFolder,
                    PersistedContainmentSubKind::ClipSource => ContainmentSubKind::ClipSource,
                    PersistedContainmentSubKind::NotebookSection => {
                        ContainmentSubKind::NotebookSection
                    },
                    PersistedContainmentSubKind::CollectionMember => {
                        ContainmentSubKind::CollectionMember
                    },
                };
                structural_changes +=
                    usize::from(payload.assert_relation(EdgeAssertion::Containment { sub_kind }));
            }
        }
        if let Some(imported) = &pedge.imported {
            for sub_kind in &imported.sub_kinds {
                let assertion = match sub_kind {
                    PersistedImportedSubKind::BookmarkFolder => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::BookmarkFolder,
                    },
                    PersistedImportedSubKind::HistoryImport => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::HistoryImport,
                    },
                    PersistedImportedSubKind::SessionImport => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::SessionImport,
                    },
                    PersistedImportedSubKind::RssMembership => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::RssMembership,
                    },
                    PersistedImportedSubKind::FileSystemImport => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::FileSystemImport,
                    },
                    PersistedImportedSubKind::ArchiveMembership => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::ArchiveMembership,
                    },
                    PersistedImportedSubKind::SharedCollection => EdgeAssertion::Imported {
                        sub_kind: ImportedSubKind::SharedCollection,
                    },
                };
                structural_changes += usize::from(payload.assert_relation(assertion));
            }
        }
        if let Some(provenance) = &pedge.provenance {
            for sub_kind in &provenance.sub_kinds {
                let assertion = match sub_kind {
                    PersistedProvenanceSubKind::ClippedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::ClippedFrom,
                    },
                    PersistedProvenanceSubKind::ExcerptedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::ExcerptedFrom,
                    },
                    PersistedProvenanceSubKind::SummarizedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::SummarizedFrom,
                    },
                    PersistedProvenanceSubKind::TranslatedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::TranslatedFrom,
                    },
                    PersistedProvenanceSubKind::RewrittenFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::RewrittenFrom,
                    },
                    PersistedProvenanceSubKind::GeneratedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::GeneratedFrom,
                    },
                    PersistedProvenanceSubKind::ExtractedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::ExtractedFrom,
                    },
                    PersistedProvenanceSubKind::ImportedFromSource => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::ImportedFromSource,
                    },
                    PersistedProvenanceSubKind::CopiedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::CopiedFrom,
                    },
                    PersistedProvenanceSubKind::MergedFrom => EdgeAssertion::Provenance {
                        sub_kind: ProvenanceSubKind::MergedFrom,
                    },
                };
                structural_changes += usize::from(payload.assert_relation(assertion));
            }
        }
    }
    for statement in &mut payload.assertions.statements {
        if !preserved_ids.contains(&statement.statement_id) {
            statement.statement_id =
                legacy_assertion_id(pedge, stratum, ordinal, &statement.predicate);
            statement.normalize_legacy_asserter();
        }
    }
    payload.assertions.rebuild_compat();
    if let Some(traversal) = &pedge.traversal {
        let data = payload.traversal.get_or_insert_with(TraversalData::default);
        data.traversals = traversal
            .traversals
            .iter()
            .map(|record| Traversal {
                timestamp_ms: record.timestamp_ms,
                trigger: match record.trigger {
                    PersistedNavigationTrigger::Unknown => NavigationTrigger::Unknown,
                    PersistedNavigationTrigger::LinkClick => NavigationTrigger::LinkClick,
                    PersistedNavigationTrigger::Back => NavigationTrigger::Back,
                    PersistedNavigationTrigger::Forward => NavigationTrigger::Forward,
                    PersistedNavigationTrigger::AddressBarEntry => {
                        NavigationTrigger::AddressBarEntry
                    },
                    PersistedNavigationTrigger::PanePromotion => NavigationTrigger::PanePromotion,
                    PersistedNavigationTrigger::Programmatic => NavigationTrigger::Programmatic,
                    PersistedNavigationTrigger::Redirect => NavigationTrigger::Redirect,
                    PersistedNavigationTrigger::ReopenSession => NavigationTrigger::ReopenSession,
                    PersistedNavigationTrigger::JumpAnchor => NavigationTrigger::JumpAnchor,
                    PersistedNavigationTrigger::InPageSearchJump => {
                        NavigationTrigger::InPageSearchJump
                    },
                    PersistedNavigationTrigger::ImportedHistory => {
                        NavigationTrigger::ImportedHistory
                    },
                },
            })
            .collect();
        data.metrics = EdgeMetrics {
            total_navigations: traversal.metrics.total_navigations,
            forward_navigations: traversal.metrics.forward_navigations,
            backward_navigations: traversal.metrics.backward_navigations,
            last_navigated_at: traversal.metrics.last_navigated_at,
        };
    }
    structural_changes
}

/// Stable one-time legacy identity: UTF-8 JSON array of version/domain,
/// stratum, original directed endpoint strings, bucket ordinal and predicate;
/// RFC 9562 UUIDv5 in the URL namespace. No clock or platform hash enters it.
fn legacy_assertion_id(
    edge: &PersistedEdge,
    stratum: GraphStratum,
    ordinal: usize,
    predicate: &str,
) -> String {
    let bytes = serde_json::to_vec(&(
        "mere.legacy-relation-assertion",
        1u8,
        match stratum {
            GraphStratum::Surface => "surface",
            GraphStratum::Resource => "resource",
        },
        &edge.from_node_id,
        &edge.to_node_id,
        ordinal,
        predicate,
    ))
    .expect("string tuple JSON");
    format!(
        "urn:mere:legacy-assertion:v1:{}",
        Uuid::new_v5(&Uuid::NAMESPACE_URL, &bytes)
    )
}

fn runtime_assertion(
    statement: &crate::persistence::PersistedSemanticStatement,
) -> SemanticStatement {
    SemanticStatement {
        statement_id: statement.statement_id.clone(),
        predicate: statement.predicate.clone(),
        recognized_sub_kind: statement.recognized_sub_kind.clone().map(semantic_sub_kind),
        label: statement.label.clone(),
        basis: statement.basis.clone(),
        graph_scope: statement.graph_scope.clone(),
        provenance_iri: statement.provenance_iri.clone(),
        first_asserted_ms: statement.first_asserted_ms,
        asserted_at_ms: statement.asserted_at_ms,
        withdrawn: statement.withdrawn.clone(),
    }
}

#[path = "from/materialize.rs"]
mod materialize;
