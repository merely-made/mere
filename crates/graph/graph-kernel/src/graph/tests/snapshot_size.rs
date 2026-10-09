// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Snapshot-size gate (petgraph-RDF plan, the common-case-bloat risk).
//!
//! The projection profile added per-statement `graph_scope`, `provenance_iri`,
//! `asserted_at_ms`, and `label` to node properties and semantic statements. The
//! plan's guard is that absent metadata stays cheap on an Author-attributed,
//! default-graph assertion. This is the "before / after"
//! tripwire that keeps that guard honest.
//!
//! A [`NodeProperty`] is the record that carries the profile metadata *without*
//! the heavy `PersistedEdge` wrapper (traversal / arrangement / containment
//! payloads) that would otherwise swamp the signal, so the property lane is
//! where the metadata cost is measured cleanly: an unpopulated common-case
//! property is pinned under a tight ceiling, and populating optional metadata must
//! cost strictly more (pay-per-use). A separate, coarse ceiling on a
//! semantic-statement chain guards the edge lane from gross regression.
//!
//! Size is the compact-JSON snapshot — the actual on-disk form
//! (`store::save_graph`), minus the pretty-printer's whitespace so the number
//! reflects field/key cost, not formatting.

use super::super::*;
use crate::graph::SemanticStatementSpec;
use crate::persistence::{
    GraphSnapshot, PersistedEdge, PersistedResourceRecord, PersistedShownResource,
};
use crate::types::{GraphScope, NodeProperty};
use euclid::default::Point2D;

/// Compact-JSON byte length of a graph's snapshot (the on-disk persistence form).
fn snapshot_len(graph: &Graph) -> usize {
    let snapshot = graph.to_snapshot();
    serde_json::to_vec(&snapshot)
        .expect("a graph snapshot is always serializable")
        .len()
}

fn facet_len(graph: &Graph) -> usize {
    serde_json::to_vec(graph.resource_facets())
        .expect("a facet store is always serializable")
        .len()
}

/// `count` nodes with sequential test URLs, no edges, no properties.
fn bare_nodes(count: usize) -> (Graph, Vec<NodeKey>) {
    let mut graph = Graph::new();
    let keys = (0..count)
        .map(|i| graph.add_node(format!("https://n{i}.test/"), Point2D::new(0.0, 0.0)))
        .collect();
    (graph, keys)
}

#[test]
fn common_case_property_metadata_stays_cheap_in_the_facet_sidecar() {
    const N: usize = 50;

    // Resource content facets carry these properties; Surface facets remain separate.
    // Baseline: N isolated nodes, no properties.
    let (bare, _) = bare_nodes(N);
    let bare_len = facet_len(&bare);

    let author = Author::person("https://author.test/").asserter_iri();
    // Common case: a default-scope property carrying its required source.
    let (mut plain, keys) = bare_nodes(N);
    for &key in &keys {
        plain.append_node_property(
            key,
            NodeProperty::new(
                "https://schema.org/datePublished".to_string(),
                "2026-07-04".to_string(),
            )
            .with_metadata(Some(author.clone()), None),
        );
    }
    let plain_len = facet_len(&plain);

    // Fully annotated: the same property, now scoped + typed + attributed + timed.
    let (mut rich, keys) = bare_nodes(N);
    for &key in &keys {
        let mut property = NodeProperty::new(
            "https://schema.org/datePublished".to_string(),
            "2026-07-04".to_string(),
        )
        .with_graph_scope(GraphScope::User)
        .with_metadata(Some(author.clone()), Some(1_720_000_000_000));
        property.datatype = Some("http://www.w3.org/2001/XMLSchema#date".to_string());
        rich.append_node_property(key, property);
    }
    let rich_len = facet_len(&rich);

    let per_plain = (plain_len - bare_len) as f64 / N as f64;
    let per_rich = (rich_len - bare_len) as f64 / N as f64;
    println!(
        "facet-size gate (property): bare={bare_len} plain={plain_len} rich={rich_len} \
         (common-case {per_plain:.0} B/property, annotated {per_rich:.0} B/property)"
    );

    // Keep the existing ceiling while retaining the Author on every property.
    assert!(
        per_plain < 280.0,
        "common-case property facet cost regressed: {per_plain:.0} B/property (bare={bare_len}, plain={plain_len})"
    );

    // Metadata is pay-per-use: populating the optional fields costs strictly more
    // than leaving them empty. If this fails, the common case is paying for
    // metadata it does not carry.
    assert!(
        rich_len > plain_len,
        "annotated snapshot ({rich_len}) must exceed the common-case one ({plain_len})"
    );
    for &key in &keys {
        let common = plain.node_properties(key).unwrap();
        assert_eq!(common.len(), 1);
        assert_eq!(common[0].provenance_iri.as_deref(), Some(author.as_str()));
        assert_eq!(common[0].asserted_at_ms, None);
        assert_eq!(common[0].graph_scope, GraphScope::Default);
        let properties = rich.node_properties(key).unwrap();
        assert_eq!(properties.len(), 1);
        assert_eq!(
            properties[0].provenance_iri.as_deref(),
            Some(author.as_str())
        );
        assert_eq!(properties[0].asserted_at_ms, Some(1_720_000_000_000));
        assert_eq!(properties[0].graph_scope, GraphScope::User);
    }
}

#[test]
fn common_case_semantic_statement_stays_cheap_in_the_snapshot() {
    const N: usize = 50;
    let statements = N - 1;

    let (bare, _) = bare_nodes(N);
    let bare_len = snapshot_len(&bare);

    let author = Author::person("https://author.test/").asserter_iri();
    // A chain of default-scope `cites` assertions retains its Author.
    let (mut plain, keys) = bare_nodes(N);
    for pair in keys.windows(2) {
        plain
            .assert_semantic_statement(
                pair[0],
                pair[1],
                SemanticStatementSpec {
                    predicate: predicate_iri(SemanticSubKind::Cites).to_string(),
                    recognized_sub_kind: Some(SemanticSubKind::Cites),
                    provenance_iri: Some(author.clone()),
                    ..Default::default()
                },
            )
            .expect("statement");
    }
    let plain_len = snapshot_len(&plain);

    let per_statement = (plain_len - bare_len) as f64 / statements as f64;
    println!(
        "snapshot-size gate (statement): bare={bare_len} plain={plain_len} \
         (common-case {per_statement:.0} B/statement, incl. edge wrapper)"
    );

    // Coarse: the whole `PersistedEdge` wrapper (defaulted edge-family payloads)
    // dominates here, so this ceiling guards the edge lane from gross regression
    // rather than isolating statement-metadata cost. Ceiling = measured (~590 B)
    // with headroom.
    assert!(
        per_statement < 760.0,
        "common-case statement snapshot cost regressed: {per_statement:.0} B/statement (bare={bare_len}, plain={plain_len})"
    );
    let snapshot = plain.to_snapshot();
    let claims: Vec<_> = snapshot
        .resource_edges
        .iter()
        .filter_map(|edge| edge.semantic.as_ref())
        .flat_map(|semantic| &semantic.statements)
        .collect();
    assert_eq!(claims.len(), statements);
    assert!(
        claims
            .iter()
            .all(|claim| claim.provenance_iri.as_deref() == Some(author.as_str()))
    );
}

#[test]
fn shared_resource_statement_cost_and_metadata_roundtrip_stay_bounded() {
    const N: usize = 50;
    let author = Author::person("https://author.test/").asserter_iri();
    let mut baseline = Graph::new();
    let keys: Vec<_> = (0..N)
        .map(|index| {
            let key = baseline.add_node(
                format!("https://shared{}.test/page#alias{index}", index / 2),
                Point2D::zero(),
            );
            baseline.refresh_surface_resource(key);
            key
        })
        .collect();
    let base = baseline.to_snapshot();
    assert_eq!(base.nodes.len(), N);
    assert_eq!(base.resources.len(), N / 2);
    assert_eq!(base.shown_resources.len(), N);
    assert!(base.resource_edges.is_empty());
    let bare_len = snapshot_len(&baseline);
    let mut plain = baseline.clone();
    let mut rich = baseline.clone();
    for pair in keys.windows(2) {
        let spec = SemanticStatementSpec {
            predicate: predicate_iri(SemanticSubKind::Cites).into(),
            recognized_sub_kind: Some(SemanticSubKind::Cites),
            provenance_iri: Some(author.clone()),
            ..Default::default()
        };
        plain
            .try_assert_semantic_statement(pair[0], pair[1], spec.clone())
            .unwrap();
        rich.try_assert_semantic_statement(
            pair[0],
            pair[1],
            SemanticStatementSpec {
                label: Some("held annotation".into()),
                graph_scope: GraphScope::User,
                asserted_at_ms: Some(0),
                ..spec
            },
        )
        .unwrap();
    }
    let plain_len = snapshot_len(&plain);
    let rich_len = snapshot_len(&rich);
    let per_statement = (plain_len - bare_len) as f64 / (N - 1) as f64;
    println!(
        "shared-resource snapshot-size gate: surfaces={N} resources={} shown={} bare={bare_len} plain={plain_len} rich={rich_len} ({per_statement:.0} B/statement incl. edge wrapper)",
        base.resources.len(),
        base.shown_resources.len()
    );
    assert!(
        per_statement < 760.0,
        "shared-resource common-case statement cost regressed: {per_statement:.0} B/statement"
    );
    assert!(
        rich_len > plain_len,
        "populated metadata is measured in the same resource baseline"
    );
    for (graph, annotated) in [(&plain, false), (&rich, true)] {
        let snapshot = graph.to_snapshot();
        assert_eq!(snapshot.resources, base.resources);
        assert_eq!(snapshot.shown_resources, base.shown_resources);
        assert!(snapshot.edges.is_empty());
        let claims: Vec<_> = snapshot
            .resource_edges
            .iter()
            .filter_map(|edge| edge.semantic.as_ref())
            .flat_map(|semantic| &semantic.statements)
            .collect();
        assert_eq!(claims.len(), N - 1);
        for claim in claims {
            assert_eq!(claim.provenance_iri.as_deref(), Some(author.as_str()));
            assert_eq!(claim.asserted_at_ms, annotated.then_some(0));
            assert_eq!(
                claim.label.as_deref(),
                annotated.then_some("held annotation")
            );
            assert_eq!(
                claim.graph_scope,
                if annotated {
                    GraphScope::User
                } else {
                    GraphScope::Default
                }
            );
        }
        let json = serde_json::to_vec(&snapshot).unwrap();
        let decoded = serde_json::from_slice::<GraphSnapshot>(&json).unwrap();
        assert_eq!(
            serde_json::to_value(&decoded).unwrap(),
            serde_json::to_value(&snapshot).unwrap()
        );
        let archive = rkyv::to_bytes::<rkyv::rancor::Error>(&snapshot).unwrap();
        let decoded = rkyv::from_bytes::<GraphSnapshot, rkyv::rancor::Error>(&archive).unwrap();
        assert_eq!(
            serde_json::to_value(&decoded).unwrap(),
            serde_json::to_value(&snapshot).unwrap()
        );
        type Columns = (
            Vec<PersistedResourceRecord>,
            Vec<PersistedEdge>,
            Vec<PersistedShownResource>,
        );
        let columns = (
            snapshot.resources.clone(),
            snapshot.resource_edges.clone(),
            snapshot.shown_resources.clone(),
        );
        let bytes = postcard::to_allocvec(&columns).unwrap();
        assert_eq!(postcard::from_bytes::<Columns>(&bytes).unwrap(), columns);
        let restored = Graph::try_from_snapshot(&snapshot).unwrap();
        assert_eq!(
            restored.to_snapshot().resource_edges,
            snapshot.resource_edges
        );
    }
}
