// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::super::{
    Author, EdgeAssertion, GraphJournal, ImportedSubKind, ProvenanceSubKind, SemanticStatement,
    SemanticSubKind, Seq, predicate_iri, replay_captured_deltas_onto,
};
use super::*;
use crate::types::GraphScope;
use euclid::default::Point2D;

fn baseline(urls: &[&str]) -> Graph {
    let mut graph = Graph::new();
    for (index, url) in urls.iter().enumerate() {
        graph.add_node_with_id(
            Uuid::from_u128(index as u128 + 1),
            (*url).into(),
            Point2D::zero(),
        );
    }
    graph
}

fn payload(id: &str, predicate: &str, source: Option<&str>) -> EdgePayload {
    let mut payload = EdgePayload::new();
    payload.push_persisted_semantic_statement(SemanticStatement {
        statement_id: id.into(),
        predicate: predicate.into(),
        recognized_sub_kind: super::super::sub_kind_from_iri(predicate),
        label: Some(format!("label-{id}")),
        graph_scope: GraphScope::User,
        provenance_iri: source.map(str::to_owned),
        asserted_at_ms: Some(100),
        first_asserted_ms: Some(100),
        withdrawn: None,
        basis: Default::default(),
    });
    payload
}

fn exact(from: u128, to: u128, payload: &EdgePayload) -> CapturedDelta {
    let pair = (Uuid::from_u128(from), Uuid::from_u128(to));
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: pair.0.to_string(),
        to_id: pair.1.to_string(),
        edges: vec![persisted_edge_for_ids(pair.0, pair.1, payload)],
    }
}

fn legacy_sidecar(from: Uuid, to: Uuid, kinds: &[super::super::RelationKind]) -> PersistedEdge {
    use super::super::RelationKind;
    use crate::persistence::{
        PersistedContainmentSubKind as C, PersistedImportedSubKind as I,
        PersistedProvenanceSubKind as P,
    };

    let mut containment = Vec::new();
    let mut imported = Vec::new();
    let mut provenance = Vec::new();
    for kind in kinds {
        match kind {
            RelationKind::Containment(kind) => containment.push(match kind {
                super::super::ContainmentSubKind::UrlPath => C::UrlPath,
                super::super::ContainmentSubKind::Domain => C::Domain,
                super::super::ContainmentSubKind::FileSystem => C::FileSystem,
                super::super::ContainmentSubKind::UserFolder => C::UserFolder,
                super::super::ContainmentSubKind::ClipSource => C::ClipSource,
                super::super::ContainmentSubKind::NotebookSection => C::NotebookSection,
                super::super::ContainmentSubKind::CollectionMember => C::CollectionMember,
            }),
            RelationKind::Imported(kind) => imported.push(match kind {
                super::super::ImportedSubKind::BookmarkFolder => I::BookmarkFolder,
                super::super::ImportedSubKind::HistoryImport => I::HistoryImport,
                super::super::ImportedSubKind::SessionImport => I::SessionImport,
                super::super::ImportedSubKind::RssMembership => I::RssMembership,
                super::super::ImportedSubKind::FileSystemImport => I::FileSystemImport,
                super::super::ImportedSubKind::ArchiveMembership => I::ArchiveMembership,
                super::super::ImportedSubKind::SharedCollection => I::SharedCollection,
            }),
            RelationKind::Provenance(kind) => provenance.push(match kind {
                super::super::ProvenanceSubKind::ClippedFrom => P::ClippedFrom,
                super::super::ProvenanceSubKind::ExcerptedFrom => P::ExcerptedFrom,
                super::super::ProvenanceSubKind::SummarizedFrom => P::SummarizedFrom,
                super::super::ProvenanceSubKind::TranslatedFrom => P::TranslatedFrom,
                super::super::ProvenanceSubKind::RewrittenFrom => P::RewrittenFrom,
                super::super::ProvenanceSubKind::GeneratedFrom => P::GeneratedFrom,
                super::super::ProvenanceSubKind::ExtractedFrom => P::ExtractedFrom,
                super::super::ProvenanceSubKind::ImportedFromSource => P::ImportedFromSource,
                super::super::ProvenanceSubKind::CopiedFrom => P::CopiedFrom,
                super::super::ProvenanceSubKind::MergedFrom => P::MergedFrom,
            }),
            _ => panic!("test legacy fixture only encodes the three old family sidecars"),
        }
    }
    PersistedEdge {
        from_node_id: from.to_string(),
        to_node_id: to.to_string(),
        families: Vec::new(),
        semantic: None,
        traversal: None,
        containment: (!containment.is_empty()).then_some(PersistedContainmentEdgeData {
            sub_kinds: containment,
        }),
        arrangement: None,
        imported: (!imported.is_empty()).then_some(PersistedImportedEdgeData {
            sub_kinds: imported,
        }),
        provenance: (!provenance.is_empty()).then_some(PersistedProvenanceEdgeData {
            sub_kinds: provenance,
        }),
        assertions: None,
    }
}

fn exact_legacy(from: u128, to: u128, kind: super::super::RelationKind) -> CapturedDelta {
    let (from_id, to_id) = (Uuid::from_u128(from), Uuid::from_u128(to));
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: from_id.to_string(),
        to_id: to_id.to_string(),
        edges: vec![legacy_sidecar(from_id, to_id, &[kind])],
    }
}

fn clear(from: u128, to: u128) -> CapturedDelta {
    CapturedDelta::ReplaySetEdgesByIds {
        from_id: Uuid::from_u128(from).to_string(),
        to_id: Uuid::from_u128(to).to_string(),
        edges: vec![],
    }
}

fn resource_pair(a: &str, b: &str) -> Pair {
    (ResourceNode::new(a).id(), ResourceNode::new(b).id())
}

fn aggregate_assertion(kind: super::super::RelationKind) -> super::super::EdgeAssertion {
    match kind {
        super::super::RelationKind::Containment(sub_kind) => {
            super::super::EdgeAssertion::Containment { sub_kind }
        },
        super::super::RelationKind::Imported(sub_kind) => {
            super::super::EdgeAssertion::Imported { sub_kind }
        },
        super::super::RelationKind::Provenance(sub_kind) => {
            super::super::EdgeAssertion::Provenance { sub_kind }
        },
        _ => unreachable!("fixture includes the three nonsemantic families"),
    }
}

fn persisted_aggregate_tags(edges: &[PersistedEdge]) -> Vec<u32> {
    edges
        .iter()
        .flat_map(|edge| aggregate_kinds(&payload_from_persisted(edge)))
        .map(super::super::RelationKind::tag)
        .collect()
}

#[test]
fn legacy_resource_migration_probe_reports_mixed_raw_and_carried_evidence_without_aliasing() {
    let baseline = baseline(&["https://a.test/old", "https://b.test/"]);
    let mut journal = GraphJournal::new();
    let author = Author::person("mint-author");
    journal.record_as(
        author.clone(),
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
            },
        },
    );
    journal.record(CapturedDelta::ReplaySetNodeUrlById {
        node_id: Uuid::from_u128(1).to_string(),
        new_url: "https://a.test/new".into(),
    });
    let mut carried = payload(
        "carried",
        predicate_iri(SemanticSubKind::Cites),
        Some(&author.asserter_iri()),
    );
    carried.assertions.statements[0].graph_scope = GraphScope::Default;
    journal.record_as(Author::person("later-author"), exact(1, 2, &carried));
    let source_before = serde_json::to_vec(journal.entries()).unwrap();
    let diagnostics = probe_legacy_mint_links(&baseline, journal.entries()).unwrap();
    assert_eq!(diagnostics.len(), 1);
    let diagnostic = &diagnostics[0];
    assert_eq!(
        (diagnostic.raw_entry_index, diagnostic.carried_entry_index),
        (0, 2)
    );
    assert_eq!(diagnostic.statement_id, "carried");
    assert!(diagnostic.explicit_source_matches);
    assert_eq!(
        diagnostic.earlier_resource_pair.0,
        ResourceNode::new("https://a.test/old").id().to_string()
    );
    assert_eq!(
        diagnostic.current_resource_pair.0,
        ResourceNode::new("https://a.test/new").id().to_string()
    );
    assert_eq!(
        serde_json::to_vec(journal.entries()).unwrap(),
        source_before
    );
    for candidate in [
        &diagnostic.earlier_resource_pair,
        &diagnostic.current_resource_pair,
    ] {
        let choice = LegacyResourceOriginResolution {
            statement_id: "carried".into(),
            from_resource_id: parse_id(&candidate.0).unwrap(),
            to_resource_id: parse_id(&candidate.1).unwrap(),
        };
        validate_legacy_origin_resolutions(&baseline, journal.entries(), &[choice.clone()])
            .unwrap();
        let resolved =
            migrate_legacy_prefix_with_resolutions(&baseline, journal.entries(), &[choice.clone()])
                .unwrap();
        let exact = resolved
            .graph
            .persisted_resource_edges_between(choice.from_resource_id, choice.to_resource_id);
        assert_eq!(statement_ids(&exact), BTreeSet::from(["carried".into()]));
        let claim = &exact
            .iter()
            .find(|edge| !edge.retained_assertions().is_empty())
            .unwrap()
            .retained_assertions()[0];
        assert_eq!(
            claim,
            &persisted_edge_for_ids(Uuid::from_u128(1), Uuid::from_u128(2), &carried)
                .retained_assertions()[0],
            "resolution preserves carried ID, time and source"
        );
        let prefix = journal
            .migrated_prefix_at_from_with_resolutions(&baseline, Seq(1), &[choice.clone()])
            .unwrap()
            .unwrap();
        assert!(
            !statement_ids(&prefix.graph.to_snapshot().resource_edges).contains("carried"),
            "future resolution does not invent a handle in an earlier prefix"
        );
        let mut effective = baseline.clone();
        replay_captured_deltas_onto(&mut effective, resolved.baseline_effects);
        for effects in resolved.entry_effects {
            replay_captured_deltas_onto(&mut effective, effects);
        }
        assert_eq!(
            effective.to_snapshot().resource_edges,
            resolved.graph.to_snapshot().resource_edges
        );
        let mut checkpoint = baseline.clone();
        journal
            .migrated_replay_from_with_baseline_and_resolutions(
                Seq(1),
                &mut checkpoint,
                &baseline,
                &[choice],
            )
            .unwrap();
        assert_eq!(
            statement_ids(&checkpoint.to_snapshot().resource_edges),
            BTreeSet::from(["carried".into()])
        );
    }
    let invalid = LegacyResourceOriginResolution {
        statement_id: "carried".into(),
        from_resource_id: ResourceNode::new("https://third.test/").id(),
        to_resource_id: parse_id(&diagnostic.current_resource_pair.1).unwrap(),
    };
    let mut unknown = invalid.clone();
    unknown.statement_id = "absent-handle".into();
    let earlier = LegacyResourceOriginResolution {
        statement_id: "carried".into(),
        from_resource_id: parse_id(&diagnostic.earlier_resource_pair.0).unwrap(),
        to_resource_id: parse_id(&diagnostic.earlier_resource_pair.1).unwrap(),
    };
    let later = LegacyResourceOriginResolution {
        from_resource_id: parse_id(&diagnostic.current_resource_pair.0).unwrap(),
        ..earlier.clone()
    };
    for choices in [vec![invalid], vec![unknown], vec![earlier, later]] {
        assert!(
            migrate_legacy_prefix_with_resolutions(&baseline, journal.entries(), &choices).is_err()
        );
        assert!(
            journal
                .migrated_snapshot_at_from_with_resolutions(&baseline, Seq(1), &choices)
                .is_err()
        );
        let mut checkpoint = baseline.clone();
        let original = checkpoint.to_snapshot();
        let revision = checkpoint.revision();
        assert!(
            journal
                .migrated_replay_from_with_baseline_and_resolutions(
                    Seq(0),
                    &mut checkpoint,
                    &baseline,
                    &choices
                )
                .is_err()
        );
        assert_eq!(checkpoint.to_snapshot().edges, original.edges);
        assert_eq!(checkpoint.to_snapshot().resources, original.resources);
        assert_eq!(checkpoint.revision(), revision);
    }
    let error = migrate_legacy_prefix(&baseline, journal.entries())
        .err()
        .expect("direct adapter must hold unresolved linkage");
    assert!(error.to_string().contains("checkpoint C28"));
    assert!(
        journal
            .migrated_prefix_at_from(&baseline, journal.live_cursor())
            .is_err()
    );
    assert!(
        journal
            .migrated_snapshot_at_from(&baseline, journal.live_cursor())
            .is_err()
    );
    let mut checkpoint = baseline.clone();
    let checkpoint_author = Author::person("checkpoint-author");
    checkpoint.write_author = checkpoint_author.clone();
    checkpoint.current_session = 55;
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::<CapturedDelta>::new()));
    let sink = seen.clone();
    checkpoint.set_recorder(Some(std::sync::Arc::new(move |delta| {
        sink.lock().unwrap().push(delta.clone())
    })));
    let before_checkpoint = serde_json::to_value(checkpoint.to_snapshot()).unwrap();
    let before_revision = checkpoint.revision();
    let before_url_revision = checkpoint.url_grouping_revision();
    assert!(
        journal
            .migrated_replay_from_with_baseline(Seq(0), &mut checkpoint, &baseline)
            .is_err()
    );
    let mut after_checkpoint = serde_json::to_value(checkpoint.to_snapshot()).unwrap();
    after_checkpoint["timestamp_secs"] = before_checkpoint["timestamp_secs"].clone();
    assert_eq!(
        after_checkpoint, before_checkpoint,
        "failed checkpoint replay leaves the whole graph unchanged"
    );
    assert_eq!(checkpoint.revision(), before_revision);
    assert_eq!(checkpoint.url_grouping_revision(), before_url_revision);
    assert_eq!(checkpoint.write_author, checkpoint_author);
    assert_eq!(checkpoint.current_session, 55);
    assert!(checkpoint.is_recording());
    assert!(seen.lock().unwrap().is_empty());
    assert_eq!(
        serde_json::to_vec(journal.entries()).unwrap(),
        source_before
    );
    let witness = CapturedDelta::ReplaySetNodeTitleById {
        node_id: Uuid::from_u128(1).to_string(),
        title: "recorder remains live".into(),
    };
    super::super::apply::apply_graph_delta(&mut checkpoint, witness.replay().into_write().unwrap());
    assert_eq!(
        *seen.lock().unwrap(),
        vec![witness],
        "a real edit proves the retained recorder works"
    );
    let mut marker = carried.clone();
    marker.assertions.statements[0].provenance_iri =
        Some(mere::model::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI.into());
    let mut uncertain = journal.entries()[..2].to_vec();
    uncertain.push(AttributedDelta {
        author: Author::person("later-author"),
        delta: exact(1, 2, &marker),
    });
    let unknown = probe_legacy_mint_links(&baseline, &uncertain).unwrap();
    assert_eq!(
        unknown.len(),
        1,
        "unknown legacy attribution cannot suppress earlier mint evidence"
    );
    assert!(!unknown[0].explicit_source_matches);
    assert!(migrate_legacy_prefix(&baseline, &uncertain).is_err());
    marker.assertions.statements[0].provenance_iri =
        Some(Author::person("different-author").asserter_iri());
    uncertain[2].delta = exact(1, 2, &marker);
    assert!(
        probe_legacy_mint_links(&baseline, &uncertain)
            .unwrap()
            .is_empty(),
        "a concrete different Author is a distinct claim control"
    );
    let author_control = migrate_legacy_prefix(&baseline, &uncertain).unwrap();
    assert_eq!(
        statement_ids(&author_control.graph.to_snapshot().resource_edges),
        BTreeSet::from(["carried".into()])
    );
    // A distinct claim key is an unambiguous carried handle control.
    carried.assertions.statements[0].predicate = "https://vocab.test/distinct".into();
    carried.assertions.statements[0].recognized_sub_kind = None;
    let mut independent = journal.entries()[..2].to_vec();
    independent.push(AttributedDelta {
        author,
        delta: exact(1, 2, &carried),
    });
    assert!(
        probe_legacy_mint_links(&baseline, &independent)
            .unwrap()
            .is_empty()
    );
    let predicate_control = migrate_legacy_prefix(&baseline, &independent).unwrap();
    assert_eq!(
        statement_ids(&predicate_control.graph.to_snapshot().resource_edges),
        BTreeSet::from(["carried".into()])
    );
}

#[path = "tests/effects.rs"]
mod effects;

#[path = "tests/placement.rs"]
mod placement;

#[path = "tests/identity.rs"]
mod identity;
