// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Snapshot-level merge for codicil compose (Alembic tail B7 / decision #1).
//!
//! Unions surfaces by stable id and resources by their prepared identity IRI.
//! Pure and deterministic. It lives here, not in `graph-kernel`, so it works
//! directly on the public `GraphSnapshot` structs without the kernel graph API:
//! [`PersistedEdge`] keys its endpoints by `from_node_id` / `to_node_id`
//! (stable `String`s), so a merge is plain Vec surgery (the audited
//! `codicil_compose_merge_plan`). A kernel `Graph::merge_from` is a later, optional
//! promotion. This is what first populates the codicil lineage
//! (`ProvenanceRecord.upstream`, via [`crate::graph_codicil::compose_graph_codicils`])
//! that the consolidation pass will read.

use std::collections::{HashMap, HashSet};

use eidetic::{Error, Result};
use kernel::graph::Graph;
use kernel::graph::predicate_declarations::{
    PREDICATE_DECLARATIONS_FACET, merge_predicate_declarations,
};
use kernel::graph::resource_classifications::merge_classification_variants;
use kernel::graph::resource_content::{RESOURCE_CLASSIFICATIONS, RESOURCE_PROPERTIES};
use kernel::graph::resource_properties::merge_resource_properties;
use kernel::persistence::{GraphSnapshot, PersistedEdge};

/// What [`merge_snapshots`] did, for the Athanor proposal / diagnostics. Never a
/// placebo: every count reflects a real change to the merged snapshot.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MergeReport {
    /// Surface identities added from `b`.
    pub added_nodes: usize,
    /// Same-id surfaces layered onto `a`'s surface.
    pub layered_nodes: usize,
    /// Surface edges added from `b` with their endpoints intact.
    pub added_edges: usize,
    /// Identical surface edges already held by `a`.
    pub deduped_edges: usize,
    /// `b` fields not carried (first cut keeps only `a`'s field layer).
    pub dropped_fields: usize,
    /// `b` couplings not carried.
    pub dropped_couplings: usize,
}

/// Merge `b` into `a`, preserving independent surfaces at the same URL.
///
/// Same-id surfaces retain the existing context layering rule. Resources share
/// their prepared IRI identity; distinct assertion handles remain distinct.
/// `a`'s field layer and navigation remain canonical. Invalid explicit resource
/// data panics here; fallible consumers use [`try_merge_snapshots`].
pub fn merge_snapshots(a: &GraphSnapshot, b: &GraphSnapshot) -> (GraphSnapshot, MergeReport) {
    try_merge_snapshots(a, b)
        .unwrap_or_else(|error| panic!("invalid snapshot composition: {error}"))
}

/// A checked merge; conflicting resource data never selects an insertion-order survivor.
pub fn try_merge_snapshots(
    a: &GraphSnapshot,
    b: &GraphSnapshot,
) -> Result<(GraphSnapshot, MergeReport)> {
    let (snapshot, report, _) = merge_snapshots_with_remap(a, b)?;
    Ok((snapshot, report))
}

/// The merge plus each `b` surface id mapped to the same merged identity.
///
/// Graph codicil composition uses this to carry the separate facet store
/// alongside the graph snapshot without coalescing same-URL surfaces.
pub(crate) fn merge_snapshots_with_remap(
    a: &GraphSnapshot,
    b: &GraphSnapshot,
) -> Result<(GraphSnapshot, MergeReport, HashMap<String, String>)> {
    for snapshot in [a, b] {
        check_assertion_handles(snapshot)?;
        Graph::try_from_snapshot(snapshot)
            .map_err(|error| Error::new(format!("invalid composition source: {error}")))?;
    }
    let mut report = MergeReport::default();
    let mut merged = a.clone();

    // A URL identifies content, while each surface keeps its own stable id.
    let mut id_to_index: HashMap<String, usize> = merged
        .nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.node_id.clone(), i))
        .collect();

    let mut remap: HashMap<String, String> = HashMap::with_capacity(b.nodes.len());
    for bn in &b.nodes {
        remap.insert(bn.node_id.clone(), bn.node_id.clone());
        if let Some(&idx) = id_to_index.get(&bn.node_id) {
            let an = &mut merged.nodes[idx];
            for t in &bn.tags {
                if !an.tags.contains(t) {
                    an.tags.push(t.clone());
                }
            }
            for p in &bn.properties {
                if !an.properties.iter().any(|q| q.predicate == p.predicate) {
                    an.properties.push(p.clone());
                }
            }
            report.layered_nodes += 1;
        } else {
            id_to_index.insert(bn.node_id.clone(), merged.nodes.len());
            merged.nodes.push(bn.clone());
            report.added_nodes += 1;
        }
    }

    // Exact duplicates collapse; full sidecars and distinct handles survive.
    let mut seen: HashSet<(String, String, String)> =
        merged.edges.iter().map(edge_signature).collect();
    for be in &b.edges {
        let mut e = be.clone();
        if let Some(canon) = remap.get(&e.from_node_id) {
            e.from_node_id = canon.clone();
        }
        if let Some(canon) = remap.get(&e.to_node_id) {
            e.to_node_id = canon.clone();
        }
        if seen.insert(edge_signature(&e)) {
            merged.edges.push(e);
            report.added_edges += 1;
        } else {
            report.deduped_edges += 1;
        }
    }

    // import_records: union by record_id, remapping membership node ids so the
    // provenance still points at the canonical nodes. Graph materialization
    // projects this durable source into the provenance.import facet.
    let mut existing: HashSet<String> = merged
        .import_records
        .iter()
        .map(|r| r.record_id.clone())
        .collect();
    for r in &b.import_records {
        if !existing.insert(r.record_id.clone()) {
            continue;
        }
        let mut rec = r.clone();
        for m in &mut rec.memberships {
            if let Some(canon) = remap.get(&m.node_id) {
                m.node_id = canon.clone();
            }
        }
        merged.import_records.push(rec);
    }

    let mut resource_ids = HashSet::new();
    merged.resources.retain(|record| {
        resource_ids.insert(chartulary::resource_id_from_canonical_iri(
            &record.canonical_iri,
        ))
    });
    let mut resources: HashMap<_, _> = merged
        .resources
        .iter()
        .enumerate()
        .map(|(index, record)| {
            (
                chartulary::resource_id_from_canonical_iri(&record.canonical_iri),
                index,
            )
        })
        .collect();
    for record in &b.resources {
        let id = chartulary::resource_id_from_canonical_iri(&record.canonical_iri);
        if let Some(&index) = resources.get(&id) {
            let target = &mut merged.resources[index];
            for facet in &record.facets {
                if let Some(existing) = target
                    .facets
                    .iter_mut()
                    .find(|existing| existing.facet == facet.facet)
                {
                    let old: serde_json::Value = serde_json::from_str(&existing.value_json)
                        .map_err(|error| Error::new(error.to_string()))?;
                    let new: serde_json::Value = serde_json::from_str(&facet.value_json)
                        .map_err(|error| Error::new(error.to_string()))?;
                    if facet.facet == PREDICATE_DECLARATIONS_FACET {
                        let declarations =
                            merge_predicate_declarations(&target.canonical_iri, &old, &new)
                                .map_err(|error| Error::new(error.to_string()))?;
                        existing.value_json = serde_json::to_string(&declarations)
                            .map_err(|error| Error::new(error.to_string()))?;
                    } else if facet.facet == RESOURCE_CLASSIFICATIONS {
                        let old: Vec<
                            kernel::graph::resource_classifications::ClassificationVariant,
                        > = serde_json::from_value(old)
                            .map_err(|error| Error::new(error.to_string()))?;
                        let new: Vec<
                            kernel::graph::resource_classifications::ClassificationVariant,
                        > = serde_json::from_value(new)
                            .map_err(|error| Error::new(error.to_string()))?;
                        let variants = merge_classification_variants(&old, &new)
                            .map_err(|error| Error::new(error.to_string()))?;
                        existing.value_json = serde_json::to_string(&variants)
                            .map_err(|error| Error::new(error.to_string()))?;
                    } else if facet.facet == RESOURCE_PROPERTIES {
                        let old: Vec<kernel::types::NodeProperty> = serde_json::from_value(old)
                            .map_err(|error| Error::new(error.to_string()))?;
                        let new: Vec<kernel::types::NodeProperty> = serde_json::from_value(new)
                            .map_err(|error| Error::new(error.to_string()))?;
                        let properties = merge_resource_properties(&old, &new)
                            .map_err(|error| Error::new(error.to_string()))?;
                        existing.value_json = serde_json::to_string(&properties)
                            .map_err(|error| Error::new(error.to_string()))?;
                    } else if facet.facet
                        == kernel::graph::legacy_content_migration::LEGACY_CONTENT_ORIGINS
                    {
                        let mut notes: Vec<
                            kernel::graph::legacy_content_migration::LegacyContentOrigin,
                        > = serde_json::from_value(old)
                            .map_err(|error| Error::new(error.to_string()))?;
                        let incoming: Vec<
                            kernel::graph::legacy_content_migration::LegacyContentOrigin,
                        > = serde_json::from_value(new)
                            .map_err(|error| Error::new(error.to_string()))?;
                        for note in incoming {
                            if !notes.contains(&note) {
                                notes.push(note);
                            }
                        }
                        notes.sort_by_cached_key(|note| {
                            serde_json::to_string(note).expect("origin encodes")
                        });
                        existing.value_json = serde_json::to_string(&notes)
                            .map_err(|error| Error::new(error.to_string()))?;
                    } else if old != new {
                        return Err(Error::new(format!(
                            "resource {id} has conflicting facet {:?}; precise variant merge is required",
                            facet.facet
                        )));
                    }
                } else {
                    target.facets.push(facet.clone());
                }
            }
        } else {
            resources.insert(id, merged.resources.len());
            merged.resources.push(record.clone());
        }
    }
    for edge in &b.resource_edges {
        if !merged.resource_edges.contains(edge) {
            merged.resource_edges.push(edge.clone());
        }
    }
    for shown in &b.shown_resources {
        if !merged.shown_resources.contains(shown) {
            merged.shown_resources.push(shown.clone());
        }
    }
    check_assertion_handles(&merged)?;
    Graph::try_from_snapshot(&merged)
        .map_err(|error| Error::new(format!("invalid composed resources: {error}")))?;

    // `b`'s field layer + navigation are dropped (first cut keeps `a`'s). Report it.
    report.dropped_fields = b.fields.len();
    report.dropped_couplings = b.couplings.len();
    merged.timestamp_secs = a.timestamp_secs.max(b.timestamp_secs);

    Ok((merged, report, remap))
}

/// A complete signature, so no sidecar disappears when endpoints coincide.
fn edge_signature(e: &PersistedEdge) -> (String, String, String) {
    (
        e.from_node_id.clone(),
        e.to_node_id.clone(),
        format!("{e:?}"),
    )
}

fn check_assertion_handles(snapshot: &GraphSnapshot) -> Result<()> {
    let surfaces: HashSet<_> = snapshot
        .nodes
        .iter()
        .filter_map(|node| uuid::Uuid::parse_str(&node.node_id).ok())
        .collect();
    let mut handles = HashMap::new();
    for (resource, edge) in snapshot
        .edges
        .iter()
        .filter(|edge| {
            [&edge.from_node_id, &edge.to_node_id]
                .into_iter()
                .all(|id| {
                    uuid::Uuid::parse_str(id)
                        .ok()
                        .is_some_and(|id| surfaces.contains(&id))
                })
        })
        .map(|edge| (false, edge))
        .chain(snapshot.resource_edges.iter().map(|edge| (true, edge)))
    {
        for statement in edge
            .semantic
            .iter()
            .flat_map(|semantic| &semantic.statements)
        {
            let identity = (
                resource,
                uuid::Uuid::parse_str(&edge.from_node_id).map_err(|_| edge.from_node_id.as_str()),
                uuid::Uuid::parse_str(&edge.to_node_id).map_err(|_| edge.to_node_id.as_str()),
                statement,
            );
            if let Some(previous) = handles.insert(statement.statement_id.as_str(), identity)
                && previous != identity
            {
                return Err(Error::new(format!(
                    "assertion {:?} has conflicting domain, endpoints or payload during composition",
                    statement.statement_id
                )));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use euclid::default::Point2D;
    use kernel::graph::Graph;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::predicate_declarations::{
        PredicateDeclaration, PredicateDeclarationError, PredicateDeclarations,
        parse_predicate_declarations,
    };
    use kernel::graph::{Author, GraphStratum};

    /// A snapshot with one node per url (ids minted by the real graph API).
    fn snap(urls: &[&str]) -> GraphSnapshot {
        let mut g = Graph::new();
        for (i, u) in urls.iter().enumerate() {
            g.add_node(u.to_string(), Point2D::new(i as f32, 0.0));
        }
        g.to_snapshot()
    }

    /// A minimal kind-less edge between two node ids.
    fn edge(from: &str, to: &str) -> PersistedEdge {
        PersistedEdge {
            from_node_id: from.to_string(),
            to_node_id: to.to_string(),
            families: Vec::new(),
            semantic: None,
            traversal: None,
            containment: None,
            arrangement: None,
            imported: None,
            provenance: None,
        }
    }

    #[test]
    fn preserves_distinct_surfaces_showing_the_same_url() {
        let a = snap(&["https://x", "https://y"]);
        let b = snap(&["https://y", "https://z"]);
        let (merged, report) = merge_snapshots(&a, &b);
        let urls: HashSet<&str> = merged.nodes.iter().map(|n| n.url.as_str()).collect();
        let want: HashSet<&str> = ["https://x", "https://y", "https://z"]
            .into_iter()
            .collect();
        assert_eq!(urls, want, "x, y, z union");
        assert_eq!(merged.nodes.len(), 4, "both y surfaces remain");
        assert_eq!(report.added_nodes, 2, "both B identities are new");
        assert_eq!(report.layered_nodes, 0);
    }

    #[test]
    fn surface_tags_stay_on_their_own_identity() {
        let mut a = snap(&["https://y"]);
        a.nodes[0].tags = vec!["from-a".into()];
        let a_id = a.nodes[0].node_id.clone();
        let mut b = snap(&["https://y"]);
        b.nodes[0].tags = vec!["from-b".into()];
        assert_ne!(a_id, b.nodes[0].node_id, "the two graphs mint distinct ids");

        let (merged, _) = merge_snapshots(&a, &b);
        assert_eq!(merged.nodes.len(), 2);
        assert_eq!(merged.nodes[0].node_id, a_id);
        assert_eq!(merged.nodes[0].tags, vec!["from-a"]);
        assert_eq!(merged.nodes[1].node_id, b.nodes[0].node_id);
        assert_eq!(merged.nodes[1].tags, vec!["from-b"]);
        b.nodes[0].node_id = a_id;
        let (same_identity, report) = merge_snapshots(&a, &b);
        assert_eq!(same_identity.nodes.len(), 1);
        assert_eq!(same_identity.nodes[0].tags, vec!["from-a", "from-b"]);
        assert_eq!(report.layered_nodes, 1, "same-id context still layers");
    }

    #[test]
    fn keeps_edge_endpoints_on_the_original_surface() {
        let a = snap(&["https://x", "https://y"]);
        let ay = a.nodes[1].node_id.clone();
        let mut b = snap(&["https://y", "https://z"]);
        let by = b.nodes[0].node_id.clone();
        let bz = b.nodes[1].node_id.clone();
        b.edges.push(edge(&by, &bz)); // b: y -> z

        let (merged, report) = merge_snapshots(&a, &b);
        assert_eq!(report.added_edges, 1);
        let e = merged
            .edges
            .iter()
            .find(|e| e.to_node_id == bz)
            .expect("the y->z edge");
        assert_eq!(
            e.from_node_id, by,
            "b's relation stays on b's browsing surface",
        );
        assert_ne!(e.from_node_id, ay);
        assert!(merged.nodes.iter().any(|node| node.node_id == ay));
    }

    #[test]
    fn dedups_a_relation_asserted_in_both() {
        let mut a = snap(&["https://x", "https://y"]);
        let (ax, ay) = (a.nodes[0].node_id.clone(), a.nodes[1].node_id.clone());
        a.edges.push(edge(&ax, &ay));
        let mut b = a.clone();
        b.edges.clear();
        let (bx, by) = (b.nodes[0].node_id.clone(), b.nodes[1].node_id.clone());
        b.edges.push(edge(&bx, &by));

        let (merged, report) = merge_snapshots(&a, &b);
        assert_eq!(
            merged.edges.len(),
            1,
            "an exact same-id relation is not doubled"
        );
        assert_eq!(report.deduped_edges, 1);
        assert_eq!(report.added_edges, 0);
        let mut independent = snap(&["https://x", "https://y"]);
        independent.edges.push(edge(
            &independent.nodes[0].node_id,
            &independent.nodes[1].node_id,
        ));
        let (separate, report) = merge_snapshots(&a, &independent);
        assert_eq!(
            separate.edges.len(),
            2,
            "independent surfaces retain their relation"
        );
        assert_eq!(report.added_edges, 1);
    }

    fn resource_record(
        iri: &str,
        facet: &str,
        value: &str,
    ) -> kernel::persistence::PersistedResourceRecord {
        kernel::persistence::PersistedResourceRecord {
            canonical_iri: iri.into(),
            facets: vec![kernel::persistence::PersistedResourceFacet {
                facet: facet.into(),
                value_json: value.into(),
            }],
        }
    }

    fn assertion(from: &str, to: &str, handle: &str) -> PersistedEdge {
        let mut edge = edge(from, to);
        edge.families = vec![kernel::persistence::PersistedEdgeFamily::Semantic];
        edge.semantic = Some(kernel::persistence::PersistedSemanticEdgeData {
            statements: vec![kernel::persistence::PersistedSemanticStatement {
                statement_id: handle.into(),
                predicate: "https://schema.org/citation".into(),
                recognized_sub_kind: None,
                label: Some("citation".into()),
                graph_scope: kernel::types::GraphScope::Default,
                provenance_iri: Some("https://author.test/".into()),
                asserted_at_ms: Some(17),
            }],
            ..Default::default()
        });
        edge
    }

    fn declaration(
        id: &str,
        stratum: GraphStratum,
        author: Author,
        declared_at_ms: u64,
    ) -> PredicateDeclaration {
        PredicateDeclaration {
            declaration_id: id.to_string(),
            stratum,
            author,
            declared_at_ms: Some(declared_at_ms),
        }
    }

    fn declaration_record(
        predicate: &str,
        variants: Vec<PredicateDeclaration>,
        selected: Option<&str>,
    ) -> kernel::persistence::PersistedResourceRecord {
        let value = serde_json::to_string(&PredicateDeclarations {
            variants,
            selected: selected.map(str::to_string),
        })
        .unwrap();
        resource_record(predicate, PREDICATE_DECLARATIONS_FACET, &value)
    }

    fn stored_declarations(snapshot: &GraphSnapshot, predicate: &str) -> PredicateDeclarations {
        let facet = snapshot
            .resources
            .iter()
            .find(|record| record.canonical_iri == predicate)
            .unwrap()
            .facets
            .iter()
            .find(|facet| facet.facet == PREDICATE_DECLARATIONS_FACET)
            .unwrap();
        parse_predicate_declarations(predicate, &serde_json::from_str(&facet.value_json).unwrap())
            .unwrap()
    }

    #[test]
    fn conflicting_declarations_retain_origins_and_claims_until_explicit_selection() {
        let predicate = "https://vocabulary.test/ns#custom";
        let resource_declaration = declaration(
            "declaration-a",
            GraphStratum::Resource,
            Author::engine("source-a", "v1").via("host-a"),
            41,
        );
        let surface_declaration = declaration(
            "declaration-b",
            GraphStratum::Surface,
            Author::person("https://person.test/b").via("host-b"),
            42,
        );
        let source = "https://source.test/";
        let target = "https://target.test/";
        let from = chartulary::resource_id_from_canonical_iri(source).to_string();
        let to = chartulary::resource_id_from_canonical_iri(target).to_string();
        let mut a = snap(&[source, target]);
        let mut b = snap(&[source, target]);
        a.resources = vec![
            declaration_record(
                predicate,
                vec![resource_declaration.clone()],
                Some("declaration-a"),
            ),
            resource_record(source, "test.source", "1"),
            resource_record(target, "test.target", "2"),
        ];
        b.resources = vec![declaration_record(
            predicate,
            vec![surface_declaration.clone()],
            Some("declaration-b"),
        )];
        let mut resource_claim = assertion(&from, &to, "held-resource");
        resource_claim.semantic.as_mut().unwrap().statements[0].predicate = predicate.to_string();
        let mut surface_claim = assertion(&b.nodes[0].node_id, &b.nodes[1].node_id, "held-surface");
        surface_claim.semantic.as_mut().unwrap().statements[0].predicate = predicate.to_string();
        a.resource_edges.push(resource_claim.clone());
        b.edges.push(surface_claim.clone());
        let before = (
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap(),
        );
        let (merged, _) = try_merge_snapshots(&a, &b).unwrap();
        let declarations = stored_declarations(&merged, predicate);
        assert_eq!(
            declarations.variants,
            vec![resource_declaration, surface_declaration]
        );
        assert_eq!(
            declarations.selected, None,
            "composition chooses no survivor"
        );
        assert_eq!(merged.resource_edges, vec![resource_claim]);
        assert_eq!(merged.edges, vec![surface_claim]);
        let (reverse, _) = try_merge_snapshots(&b, &a).unwrap();
        assert_eq!(stored_declarations(&reverse, predicate), declarations);
        assert_eq!(serde_json::to_value(&a).unwrap(), before.0);
        assert_eq!(serde_json::to_value(&b).unwrap(), before.1);

        let mut graph = Graph::try_from_snapshot(&merged).unwrap();
        assert!(matches!(
            graph.effective_predicate_stratum(predicate),
            Err(PredicateDeclarationError::Conflict { .. })
        ));
        assert_eq!(
            graph
                .effective_predicate_stratum("https://other.test/predicate")
                .unwrap(),
            GraphStratum::Resource,
            "unaffected predicates stay usable"
        );
        let held = graph.to_snapshot();
        assert!(
            graph
                .select_predicate_declaration(predicate, "declaration-b")
                .unwrap()
        );
        assert_eq!(
            graph.effective_predicate_stratum(predicate).unwrap(),
            GraphStratum::Surface
        );
        let chosen = graph.to_snapshot();
        assert_eq!(chosen.edges, held.edges);
        assert_eq!(chosen.resource_edges, held.resource_edges);
        assert_eq!(
            chosen.edges[0].semantic.as_ref().unwrap().statements,
            merged.edges[0].semantic.as_ref().unwrap().statements
        );
        assert_eq!(
            chosen.resource_edges[0]
                .semantic
                .as_ref()
                .unwrap()
                .statements,
            merged.resource_edges[0]
                .semantic
                .as_ref()
                .unwrap()
                .statements
        );
        assert_eq!(
            stored_declarations(&chosen, predicate).variants,
            declarations.variants
        );
        assert_eq!(
            stored_declarations(&chosen, predicate).selected.as_deref(),
            Some("declaration-b")
        );
    }

    #[test]
    fn equal_and_disjoint_declarations_remain_unambiguous_during_composition() {
        let predicate = "https://vocabulary.test/ns#custom";
        let other = "https://vocabulary.test/ns#other";
        let first = declaration("first", GraphStratum::Surface, Author::person("a"), 1);
        let second = declaration("second", GraphStratum::Surface, Author::rule("b", "v2"), 2);
        let mut a = snap(&[]);
        a.resources = vec![declaration_record(
            predicate,
            vec![first.clone()],
            Some("first"),
        )];
        let (identical, _) = try_merge_snapshots(&a, &a).unwrap();
        assert_eq!(
            stored_declarations(&identical, predicate).variants,
            vec![first.clone()]
        );
        assert_eq!(
            stored_declarations(&identical, predicate)
                .selected
                .as_deref(),
            Some("first")
        );
        let mut b = snap(&[]);
        b.resources = vec![declaration_record(
            predicate,
            vec![second.clone()],
            Some("second"),
        )];
        let (equal_nature, _) = try_merge_snapshots(&a, &b).unwrap();
        assert_eq!(
            stored_declarations(&equal_nature, predicate).variants,
            vec![first, second]
        );
        let graph = Graph::try_from_snapshot(&equal_nature).unwrap();
        assert_eq!(
            graph.effective_predicate_stratum(predicate).unwrap(),
            GraphStratum::Surface
        );
        b.resources = vec![declaration_record(
            other,
            vec![declaration(
                "other",
                GraphStratum::Resource,
                Author::person("c"),
                3,
            )],
            Some("other"),
        )];
        let (disjoint, _) = try_merge_snapshots(&a, &b).unwrap();
        assert_eq!(disjoint.resources.len(), 2);
        let graph = Graph::try_from_snapshot(&disjoint).unwrap();
        assert_eq!(
            graph.effective_predicate_stratum(predicate).unwrap(),
            GraphStratum::Surface
        );
        assert_eq!(
            graph.effective_predicate_stratum(other).unwrap(),
            GraphStratum::Resource
        );
    }

    #[test]
    fn conflicting_declaration_handles_reject_atomically_with_distinct_handle_control() {
        let predicate = "https://vocabulary.test/ns#custom";
        let first = declaration("stable", GraphStratum::Resource, Author::person("a"), 1);
        let mut a = snap(&[]);
        a.resources = vec![declaration_record(
            predicate,
            vec![first.clone()],
            Some("stable"),
        )];
        let mut b = a.clone();
        let mut changed = first;
        changed.author = Author::person("b");
        b.resources = vec![declaration_record(
            predicate,
            vec![changed.clone()],
            Some("stable"),
        )];
        let before = (
            serde_json::to_value(&a).unwrap(),
            serde_json::to_value(&b).unwrap(),
        );
        assert!(
            try_merge_snapshots(&a, &b).is_err(),
            "one handle cannot select an origin"
        );
        assert_eq!(serde_json::to_value(&a).unwrap(), before.0);
        assert_eq!(serde_json::to_value(&b).unwrap(), before.1);
        changed.declaration_id = "distinct".to_string();
        b.resources = vec![declaration_record(
            predicate,
            vec![changed],
            Some("distinct"),
        )];
        let (merged, _) = try_merge_snapshots(&a, &b).unwrap();
        assert_eq!(stored_declarations(&merged, predicate).variants.len(), 2);
        let graph = Graph::try_from_snapshot(&merged).unwrap();
        assert_eq!(
            graph.effective_predicate_stratum(predicate).unwrap(),
            GraphStratum::Resource
        );
    }

    #[test]
    fn shared_resources_keep_surface_associations_metadata_and_distinct_handles() {
        let iri = "https://shared.test/page";
        let target = "https://target.test/item";
        let from = chartulary::resource_id_from_canonical_iri(iri).to_string();
        let to = chartulary::resource_id_from_canonical_iri(target).to_string();
        let mut a = snap(&[iri]);
        let mut b = snap(&[iri]);
        a.resources = vec![
            resource_record(iri, "test.a", "1"),
            resource_record(target, "test.same", "{\"x\":1}"),
        ];
        b.resources = vec![
            resource_record(iri, "test.b", "2"),
            resource_record(target, "test.same", "{ \"x\": 1 }"),
        ];
        a.shown_resources = vec![kernel::persistence::PersistedShownResource {
            surface_id: a.nodes[0].node_id.clone(),
            resource_id: from.clone(),
        }];
        b.shown_resources = vec![kernel::persistence::PersistedShownResource {
            surface_id: b.nodes[0].node_id.clone(),
            resource_id: from.clone(),
        }];
        a.resource_edges = vec![assertion(&from, &to, "handle-a")];
        b.resource_edges = vec![assertion(&from, &to, "handle-b")];
        let (merged, _) = try_merge_snapshots(&a, &b).unwrap();
        assert_eq!(merged.nodes.len(), 2);
        assert_eq!(merged.resources.len(), 2);
        assert_eq!(merged.shown_resources.len(), 2);
        let shared = merged
            .resources
            .iter()
            .find(|record| record.canonical_iri == iri)
            .unwrap();
        assert_eq!(shared.facets.len(), 2, "disjoint metadata survives");
        let statements: Vec<_> = merged
            .resource_edges
            .iter()
            .flat_map(|edge| &edge.semantic.as_ref().unwrap().statements)
            .collect();
        assert_eq!(
            statements.len(),
            2,
            "same assertion key keeps both old handles"
        );
        assert_eq!(
            statements
                .iter()
                .map(|statement| statement.statement_id.as_str())
                .collect::<Vec<_>>(),
            vec!["handle-a", "handle-b"]
        );
        assert!(
            statements
                .iter()
                .all(|statement| statement.asserted_at_ms == Some(17)
                    && statement.provenance_iri.as_deref() == Some("https://author.test/"))
        );
        let (identical, _) = try_merge_snapshots(&a, &a).unwrap();
        assert_eq!(
            identical.resource_edges.len(),
            1,
            "an exact duplicate adds no assertion"
        );
        assert_eq!(identical.shown_resources.len(), 1);
        assert!(Graph::try_from_snapshot(&merged).is_ok());
    }

    #[test]
    fn resource_conflicts_return_errors_with_equal_and_disjoint_controls() {
        let iri = "https://shared.test/page";
        let mut a = snap(&[iri]);
        let mut b = snap(&[iri]);
        a.resources = vec![resource_record(
            iri,
            "test.review",
            "{\"status\":\"accepted\"}",
        )];
        b.resources = vec![resource_record(
            iri,
            "test.review",
            "{\"status\":\"rejected\"}",
        )];
        let error = try_merge_snapshots(&a, &b).unwrap_err().to_string();
        assert!(
            error.contains("test.review") && error.contains("conflicting"),
            "{error}"
        );
        assert_eq!(
            a.resources[0].facets[0].value_json,
            "{\"status\":\"accepted\"}"
        );
        assert_eq!(
            b.resources[0].facets[0].value_json,
            "{\"status\":\"rejected\"}"
        );
        b.resources = a.resources.clone();
        assert!(
            try_merge_snapshots(&a, &b).is_ok(),
            "equal metadata is accepted"
        );
        b.resources[0].facets[0].facet = "test.other-review".into();
        let (merged, _) = try_merge_snapshots(&a, &b).unwrap();
        assert_eq!(
            merged.resources[0].facets.len(),
            2,
            "disjoint metadata is retained"
        );
    }

    #[test]
    fn a_shared_handle_cannot_pick_a_payload_during_composition() {
        let from_iri = "https://from.test/item";
        let to_iri = "https://to.test/item";
        let from = chartulary::resource_id_from_canonical_iri(from_iri).to_string();
        let to = chartulary::resource_id_from_canonical_iri(to_iri).to_string();
        let mut a = snap(&[]);
        a.resources = vec![
            resource_record(from_iri, "test.a", "1"),
            resource_record(to_iri, "test.a", "1"),
        ];
        a.resource_edges = vec![assertion(&from, &to, "stable-handle")];
        let mut b = a.clone();
        b.resource_edges[0].semantic.as_mut().unwrap().statements[0].label =
            Some("conflict".into());
        assert!(try_merge_snapshots(&a, &b).is_err());
        b.resource_edges = a.resource_edges.clone();
        assert_eq!(
            try_merge_snapshots(&a, &b).unwrap().0.resource_edges.len(),
            1
        );
        b.resource_edges[0].semantic.as_mut().unwrap().statements[0].statement_id =
            "other-handle".into();
        assert_eq!(
            try_merge_snapshots(&a, &b).unwrap().0.resource_edges.len(),
            2
        );
    }

    #[test]
    fn legacy_surface_handle_conflicts_stop_with_distinct_and_duplicate_controls() {
        let mut a = snap(&["https://a.test/", "https://b.test/", "https://c.test/"]);
        let from = a.nodes[0].node_id.clone();
        let to = a.nodes[1].node_id.clone();
        a.edges = vec![assertion(&from, &to, "legacy-handle")];
        let mut b = a.clone();
        b.edges[0].to_node_id = a.nodes[2].node_id.clone();
        assert!(
            try_merge_snapshots(&a, &b).is_err(),
            "same id on a different pair stops"
        );
        b.edges = a.edges.clone();
        b.edges[0].semantic.as_mut().unwrap().statements[0].label = Some("changed".into());
        assert!(
            try_merge_snapshots(&a, &b).is_err(),
            "same id with a different payload stops"
        );
        b.edges[0].semantic.as_mut().unwrap().statements[0].statement_id = "distinct-handle".into();
        assert_eq!(try_merge_snapshots(&a, &b).unwrap().0.edges.len(), 2);
        b.edges = a.edges.clone();
        assert_eq!(try_merge_snapshots(&a, &b).unwrap().0.edges.len(), 1);
        b.edges[0].from_node_id = from.to_uppercase();
        b.edges[0].to_node_id = to.to_uppercase();
        assert!(
            try_merge_snapshots(&a, &b).is_ok(),
            "UUID spelling does not change the pair named by an assertion"
        );
        let mut cross_domain = a.clone();
        cross_domain.resource_edges = a.edges.clone();
        assert!(
            check_assertion_handles(&cross_domain).is_err(),
            "one handle cannot name assertions in both active strata"
        );
    }

    #[test]
    fn orphan_surface_rows_stay_retained_without_owning_active_assertion_handles() {
        let mut source = snap(&["https://surface.test/a", "https://surface.test/b"]);
        let from_iri = "https://resource.test/a";
        let to_iri = "https://resource.test/b";
        let from = chartulary::resource_id_from_canonical_iri(from_iri).to_string();
        let to = chartulary::resource_id_from_canonical_iri(to_iri).to_string();
        source.resources = vec![
            resource_record(from_iri, "test.origin", "1"),
            resource_record(to_iri, "test.origin", "2"),
        ];
        source.resource_edges = vec![assertion(&from, &to, "resource-handle")];
        source.edges = vec![assertion(
            &source.nodes[0].node_id,
            &uuid::Uuid::from_u128(999).to_string(),
            "resource-handle",
        )];
        assert!(Graph::try_from_snapshot(&source).is_ok());
        let before = serde_json::to_value(&source).unwrap();
        let empty = snap(&[]);
        let merged = try_merge_snapshots(&source, &empty).unwrap().0;
        assert_eq!(
            serde_json::to_value(&source).unwrap(),
            before,
            "composition leaves source truth untouched"
        );
        assert_eq!(merged.edges, source.edges, "orphan rows stay retained");
        assert_eq!(merged.resource_edges, source.resource_edges);
        assert!(Graph::try_from_snapshot(&merged).is_ok());
        assert_eq!(
            try_merge_snapshots(&empty, &source)
                .unwrap()
                .0
                .resource_edges,
            source.resource_edges,
        );

        let mut invalid_endpoint = source.clone();
        invalid_endpoint.edges[0].to_node_id = "not-a-uuid".into();
        let merged = try_merge_snapshots(&invalid_endpoint, &empty).unwrap().0;
        assert_eq!(merged.edges, invalid_endpoint.edges);
        assert_eq!(merged.resource_edges, source.resource_edges);

        let mut active = source.clone();
        active.edges[0].to_node_id = active.nodes[1].node_id.to_uppercase();
        assert!(check_assertion_handles(&active).is_err());
        assert!(try_merge_snapshots(&active, &empty).is_err());
        active.edges[0].semantic.as_mut().unwrap().statements[0].statement_id =
            "distinct-surface-handle".into();
        assert!(check_assertion_handles(&active).is_ok());
        assert!(try_merge_snapshots(&active, &empty).is_ok());
        assert_eq!(serde_json::to_value(&source).unwrap(), before);
    }
}
