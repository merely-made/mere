// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! SPARQL query over the graph (the `query` feature).
//!
//! A borrowed, read-only kernel RDF projection evaluated with `spareval`.
//! Each triple pattern streams the projection and deduplicates its matching
//! quads. The adapter keeps kernel authority without rebuilding a whole
//! materialized dataset before each query. Broad patterns still scan all rows.
//! Materialized spareval and Oxigraph paths remain test oracles.

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

#[cfg(test)]
mod coverage_tests {
    use super::*;
    use kernel::graph::{
        CoverageLayer, CoverageLimit, CoverageNote, PendingLink, ResourceNode,
        SemanticStatementSpec,
    };

    #[test]
    fn select_empty_select_and_ask_preserve_every_known_coverage_layer() {
        let mut graph = Graph::new();
        let limits = CoverageLayer::ALL
            .into_iter()
            .map(|layer| CoverageLimit::new(layer, "host boundary"))
            .collect();
        let note = CoverageNote { limits };
        let mut truth = graph.to_snapshot();
        truth.timestamp_secs = 0;
        graph.set_known_coverage(note.clone());
        for query in ["SELECT ?s WHERE { ?s ?p ?o }", "ASK { ?s ?p ?o }"] {
            let rows = sparql(&graph, query).unwrap();
            assert_eq!(rows.coverage, note);
            assert_eq!(rows, sparql_materialized(&graph, query).unwrap());
        }
        let nonempty = sparql(
            &graph,
            "SELECT ?s WHERE { VALUES ?s { <urn:held-result> } }",
        )
        .unwrap();
        assert_eq!(nonempty.rows.len(), 1);
        assert_eq!(nonempty.coverage, note);
        let mut after = graph.to_snapshot();
        after.timestamp_secs = 0;
        assert_eq!(
            serde_json::to_value(after).unwrap(),
            serde_json::to_value(truth).unwrap()
        );
        assert!(
            sparql(&Graph::new(), "ASK { ?s ?p ?o }")
                .unwrap()
                .coverage
                .limits
                .is_empty()
        );
    }

    #[test]
    fn pending_unknown_targets_are_possession_limits_unless_known_to_be_unloaded() {
        let mut graph = Graph::new();
        let source = kernel::graph::apply::add_node(
            &mut graph,
            None,
            "https://example.org/source".into(),
            Default::default(),
        );
        let target = ResourceNode::new("https://example.org/absent");
        graph.queue_pending_link(PendingLink {
            source_resource: graph.shown_resource_id(source).unwrap(),
            source_surface: None,
            target_iri: target.canonical_iri().into(),
            statement: SemanticStatementSpec {
                predicate: "https://mere.computer/ns/rel#cites".into(),
                ..Default::default()
            },
        });
        let rows = sparql(&graph, "ASK { ?s ?p ?o }").unwrap();
        assert!(
            rows.coverage
                .limits
                .iter()
                .any(|limit| limit.layer == CoverageLayer::Possession)
        );
        for scope in [vec![target.id()], vec![]] {
            let mut limit = CoverageLimit::new(CoverageLayer::Residency, "known but not loaded");
            limit.resources = scope;
            graph.set_known_coverage(CoverageNote {
                limits: vec![limit],
            });
            let rows = sparql(&graph, "ASK { ?s ?p ?o }").unwrap();
            assert!(
                rows.coverage
                    .limits
                    .iter()
                    .any(|limit| limit.layer == CoverageLayer::Residency)
            );
            assert!(
                !rows
                    .coverage
                    .limits
                    .iter()
                    .any(|limit| limit.layer == CoverageLayer::Possession)
            );
        }
    }
}

use kernel::graph::Graph;
use oxrdf::Term;
use spareval::{QueryEvaluator, QueryResults};
use spargebra::SparqlParser;

mod dataset;

use dataset::GraphDataset;

/// The rows of a SPARQL `SELECT` (or the boolean of an `ASK`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryRows {
    /// The selected variable names, in result order.
    pub variables: Vec<String>,
    /// One entry per solution; each is the bound term per variable (display
    /// form), or `None` when the variable is unbound in that solution.
    pub rows: Vec<Vec<Option<String>>>,
    pub coverage: kernel::graph::CoverageNote,
}

/// Run `query` over `graph` and return the solution rows. The graph is
/// read through a borrowed dataset adapter. Errors (parse,
/// evaluation) are returned as their display string. `CONSTRUCT` / `DESCRIBE`
/// are not supported in this cut.
pub fn sparql(graph: &Graph, query: &str) -> Result<QueryRows, String> {
    let dataset = GraphDataset::new(graph);
    let query = SparqlParser::new()
        .parse_query(query)
        .map_err(|e| e.to_string())?;
    let evaluator = QueryEvaluator::new();
    let results = evaluator
        .prepare(&query)
        .execute(dataset)
        .map_err(|e| e.to_string())?;

    query_rows(results, graph.coverage_note())
}

fn query_rows(
    results: QueryResults<'_>,
    coverage: kernel::graph::CoverageNote,
) -> Result<QueryRows, String> {
    match results {
        QueryResults::Solutions(solutions) => {
            let variables: Vec<String> = solutions
                .variables()
                .iter()
                .map(|v| v.as_str().to_string())
                .collect();
            let mut rows = Vec::new();
            for solution in solutions {
                let solution = solution.map_err(|e| e.to_string())?;
                let row = variables
                    .iter()
                    .map(|var| solution.get(var.as_str()).map(term_to_string))
                    .collect();
                rows.push(row);
            }
            Ok(QueryRows {
                variables,
                rows,
                coverage,
            })
        },
        QueryResults::Boolean(value) => Ok(QueryRows {
            variables: vec!["result".to_string()],
            rows: vec![vec![Some(value.to_string())]],
            coverage,
        }),
        QueryResults::Graph(_) => {
            Err("CONSTRUCT / DESCRIBE results are not supported in this cut".to_string())
        },
    }
}

/// The former materialized query path, retained to verify adapter semantics.
#[cfg(test)]
fn sparql_materialized(graph: &Graph, query: &str) -> Result<QueryRows, String> {
    let dataset: oxrdf::Dataset = crate::dataset_quads(graph).into_iter().collect();
    let query = SparqlParser::new()
        .parse_query(query)
        .map_err(|error| error.to_string())?;
    let evaluator = QueryEvaluator::new();
    let results = evaluator
        .prepare(&query)
        .execute(&dataset)
        .map_err(|error| error.to_string())?;
    query_rows(results, graph.coverage_note())
}

/// A bound term's display form for a result cell: the bare IRI / lexical value
/// (no angle brackets or quotes), `_:id` for a blank node.
fn term_to_string(term: &Term) -> String {
    match term {
        Term::NamedNode(n) => n.as_str().to_string(),
        Term::Literal(l) => l.value().to_string(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        Term::Triple(triple) => triple.to_string(),
    }
}

/// The retired copy-into-Oxigraph-`Store` query path, kept as the parity
/// oracle: an independently implemented SPARQL engine over the same
/// projection, diff-tested against the spareval mainline above. Oxigraph
/// carries its own RDF model (a newer `oxrdf`), so [`baseline::to_ox_quad`]
/// rebuilds each term across the two crate versions.
#[cfg(test)]
mod baseline {
    use super::{QueryRows, XSD_STRING, term_to_string};
    use kernel::graph::Graph;
    use oxigraph::model::{
        BlankNode as OxBlankNode, GraphName as OxGraphName, Literal as OxLiteral,
        NamedNode as OxNamedNode, NamedOrBlankNode as OxNamedOrBlankNode, Quad as OxQuad,
        Term as OxTerm, Triple as OxTriple,
    };
    use oxigraph::sparql::{QueryResults as OxQueryResults, SparqlEvaluator};
    use oxigraph::store::Store;

    use crate::dataset_quads;

    pub(super) fn sparql_store(graph: &Graph, query: &str) -> Result<QueryRows, String> {
        let store = Store::new().map_err(|e| e.to_string())?;
        for quad in dataset_quads(graph) {
            if let Some(oxquad) = to_ox_quad(&quad) {
                store.insert(&oxquad).map_err(|e| e.to_string())?;
            }
        }

        let results = SparqlEvaluator::new()
            .parse_query(query)
            .map_err(|e| e.to_string())?
            .on_store(&store)
            .execute()
            .map_err(|e| e.to_string())?;

        match results {
            OxQueryResults::Solutions(solutions) => {
                let variables: Vec<String> = solutions
                    .variables()
                    .iter()
                    .map(|v| v.as_str().to_string())
                    .collect();
                let mut rows = Vec::new();
                for solution in solutions {
                    let solution = solution.map_err(|e| e.to_string())?;
                    let row = variables
                        .iter()
                        .map(|var| solution.get(var.as_str()).map(ox_term_to_string))
                        .collect();
                    rows.push(row);
                }
                Ok(QueryRows {
                    variables,
                    rows,
                    coverage: graph.coverage_note(),
                })
            },
            OxQueryResults::Boolean(value) => Ok(QueryRows {
                variables: vec!["result".to_string()],
                rows: vec![vec![Some(value.to_string())]],
                coverage: graph.coverage_note(),
            }),
            OxQueryResults::Graph(_) => {
                Err("CONSTRUCT / DESCRIBE results are not supported in this cut".to_string())
            },
        }
    }

    fn to_ox_subject(subject: &oxrdf::NamedOrBlankNode) -> Option<OxNamedOrBlankNode> {
        Some(match subject {
            oxrdf::NamedOrBlankNode::NamedNode(n) => OxNamedNode::new(n.as_str()).ok()?.into(),
            oxrdf::NamedOrBlankNode::BlankNode(b) => OxBlankNode::new(b.as_str()).ok()?.into(),
        })
    }

    fn to_ox_term(term: &oxrdf::Term) -> Option<OxTerm> {
        Some(match term {
            oxrdf::Term::NamedNode(n) => OxNamedNode::new(n.as_str()).ok()?.into(),
            oxrdf::Term::BlankNode(b) => OxBlankNode::new(b.as_str()).ok()?.into(),
            oxrdf::Term::Literal(l) => {
                if let Some(language) = l.language() {
                    OxLiteral::new_language_tagged_literal(l.value(), language)
                        .ok()?
                        .into()
                } else if l.datatype().as_str() == XSD_STRING {
                    OxLiteral::new_simple_literal(l.value()).into()
                } else {
                    OxLiteral::new_typed_literal(
                        l.value(),
                        OxNamedNode::new(l.datatype().as_str()).ok()?,
                    )
                    .into()
                }
            },
            oxrdf::Term::Triple(triple) => OxTriple::new(
                to_ox_subject(&triple.subject)?,
                OxNamedNode::new(triple.predicate.as_str()).ok()?,
                to_ox_term(&triple.object)?,
            )
            .into(),
        })
    }

    fn to_ox_quad(quad: &oxrdf::Quad) -> Option<OxQuad> {
        let subject = to_ox_subject(&quad.subject)?;
        let predicate = OxNamedNode::new(quad.predicate.as_str()).ok()?;
        let object = to_ox_term(&quad.object)?;
        Some(OxQuad::new(
            subject,
            predicate,
            object,
            match &quad.graph_name {
                oxrdf::GraphName::DefaultGraph => OxGraphName::DefaultGraph,
                oxrdf::GraphName::NamedNode(node) => OxNamedNode::new(node.as_str()).ok()?.into(),
                oxrdf::GraphName::BlankNode(node) => OxBlankNode::new(node.as_str()).ok()?.into(),
            },
        ))
    }

    fn ox_term_to_string(term: &OxTerm) -> String {
        match term {
            OxTerm::NamedNode(n) => n.as_str().to_string(),
            OxTerm::Literal(l) => l.value().to_string(),
            OxTerm::BlankNode(b) => format!("_:{}", b.as_str()),
            OxTerm::Triple(triple) => triple.to_string(),
        }
    }

    // Both engines share this crate's display mapping; anchor the assumption.
    const _: fn(&super::Term) -> String = term_to_string;
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::apply::assert_semantic_relation_in_scope;
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{SemanticStatementSpec, SemanticSubKind};
    use kernel::types::{GraphScope, NodeProperty};

    /// A graph exercising the projection's full surface: curated literals,
    /// recognized and raw statements, named scopes, statement metadata
    /// (reifiers), typed and language-tagged property literals, rdf:type.
    fn rich_graph() -> Graph {
        let mut graph = Graph::new();
        let a = graph.add_node("https://a.test/".to_string(), Default::default());
        let b = graph.add_node("https://b.test/".to_string(), Default::default());
        let c = graph.add_node("https://c.test/".to_string(), Default::default());

        {
            let node = graph.get_node_mut(a).expect("node a");
            node.title = "Article A".to_string();
            node.tags.insert("research".to_string());
        }

        assert_semantic_relation_in_scope(
            &mut graph,
            a,
            b,
            SemanticSubKind::Cites,
            Some("cites".to_string()),
            GraphScope::Source,
        );
        graph.assert_semantic_statement(
            a,
            c,
            SemanticStatementSpec {
                predicate: "https://mere.computer/ns/rel#cites".to_string(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                label: Some("also cites".to_string()),
                graph_scope: GraphScope::User,
                provenance_iri: Some("https://people.test/alice".to_string()),
                asserted_at_ms: Some(1_720_000_000_123),
            },
        );
        graph.assert_semantic_statement(
            b,
            c,
            SemanticStatementSpec {
                predicate: "https://example.test/vocab#refutes".to_string(),
                ..Default::default()
            },
        );

        let mut published = NodeProperty::new(
            "https://schema.org/datePublished".to_string(),
            "2026-07-04".to_string(),
        )
        .with_graph_scope(GraphScope::User)
        .with_metadata(
            Some("https://people.test/bob".to_string()),
            Some(1_720_000_100_456),
        );
        published.datatype = Some("http://www.w3.org/2001/XMLSchema#date".to_string());
        assert!(graph.append_node_properties(a, vec![published]));
        let mut summary = NodeProperty::new(
            "https://schema.org/abstract".to_string(),
            "Un article".to_string(),
        );
        summary.lang = Some("fr".to_string());
        assert!(graph.append_node_properties(a, vec![summary]));

        graph
    }

    /// Row order is engine-dependent when the query has no ORDER BY, so
    /// parity compares row multisets.
    fn sorted(mut rows: QueryRows) -> QueryRows {
        rows.rows.sort();
        rows
    }

    const PARITY_QUERIES: &[&str] = &[
        "SELECT ?s ?p ?o WHERE { ?s ?p ?o }",
        "SELECT ?g ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } }",
        "SELECT ?name WHERE { <https://a.test/> <https://schema.org/name> ?name }",
        "SELECT ?t WHERE { GRAPH <https://mere.computer/ns/graph#user> { <https://a.test> <https://mere.computer/ns/rel#cites> ?t } }",
        "SELECT ?stmt ?prov WHERE { GRAPH ?g { ?stmt <http://www.w3.org/ns/prov#wasAttributedTo> ?prov } }",
        "SELECT ?v WHERE { <https://a.test> <https://schema.org/abstract> ?v FILTER(lang(?v) = 'fr') }",
        "SELECT (COUNT(?s) AS ?n) WHERE { ?s a <https://mere.computer/ns/core#Node> }",
        "ASK { <https://b.test> <https://example.test/vocab#refutes> <https://c.test> }",
        "ASK { <https://b.test> <https://example.test/vocab#refutes> <https://a.test> }",
    ];

    /// Phase 3 gate (a): the spareval mainline returns the same solutions as
    /// the retired Oxigraph-Store baseline on every representative query.
    #[test]
    fn spareval_rows_match_store_baseline() {
        let graph = rich_graph();
        for query in PARITY_QUERIES {
            let mainline = sorted(sparql(&graph, query).expect(query));
            let oracle = sorted(baseline::sparql_store(&graph, query).expect(query));
            assert_eq!(mainline, oracle, "row parity for: {query}");
            assert_eq!(
                mainline,
                sorted(sparql_materialized(&graph, query).expect(query)),
                "materialized dataset parity for: {query}"
            );
            assert!(
                !mainline.rows.is_empty(),
                "parity query must exercise rows: {query}"
            );
        }
        for (query, expected) in [(PARITY_QUERIES[7], "true"), (PARITY_QUERIES[8], "false")] {
            assert_eq!(
                sparql(&graph, query).unwrap().rows,
                vec![vec![Some(expected.to_string())]],
                "canonical resource ASK control: {query}"
            );
        }
    }

    #[test]
    fn adapter_patterns_preserve_graph_scopes_and_rdf_terms() {
        use oxrdf::{Literal, NamedNode};
        use spareval::QueryableDataset;

        fn rows<'a, D: QueryableDataset<'a>>(
            dataset: &D,
            subject: Option<&Term>,
            predicate: Option<&Term>,
            object: Option<&Term>,
            scope: Option<Option<&Term>>,
        ) -> Vec<(Term, Term, Term, Option<Term>)> {
            let internalize = |term: &Term| {
                dataset
                    .internalize_term(term.clone())
                    .unwrap_or_else(|error| panic!("{error}"))
            };
            let subject = subject.map(internalize);
            let predicate = predicate.map(internalize);
            let object = object.map(internalize);
            let scope = scope.map(|name| name.map(internalize));
            let mut rows: Vec<_> = dataset
                .internal_quads_for_pattern(
                    subject.as_ref(),
                    predicate.as_ref(),
                    object.as_ref(),
                    scope.as_ref().map(|name| name.as_ref()),
                )
                .map(|quad| {
                    let quad = quad.unwrap_or_else(|error| panic!("{error}"));
                    let externalize = |term| {
                        dataset
                            .externalize_term(term)
                            .unwrap_or_else(|error| panic!("{error}"))
                    };
                    (
                        externalize(quad.subject),
                        externalize(quad.predicate),
                        externalize(quad.object),
                        quad.graph_name.map(externalize),
                    )
                })
                .collect();
            rows.sort_by_key(|row| format!("{row:?}"));
            rows
        }

        let graph = rich_graph();
        let adapter = GraphDataset::new(&graph);
        let materialized: oxrdf::Dataset = crate::dataset_quads(&graph).into_iter().collect();
        let oracle = &materialized;
        let user: Term = NamedNode::new(crate::GRAPH_SCOPE_USER).unwrap().into();
        let source: Term = NamedNode::new(crate::GRAPH_SCOPE_SOURCE).unwrap().into();
        let missing: Term = NamedNode::new("https://missing.test/graph").unwrap().into();
        for scope in [Some(None), None, Some(Some(&user)), Some(Some(&source))] {
            let actual = rows(&adapter, None, None, None, scope);
            assert!(!actual.is_empty(), "positive scope control {scope:?}");
            assert_eq!(actual, rows(&oracle, None, None, None, scope));
        }
        assert!(rows(&adapter, None, None, None, Some(Some(&missing))).is_empty());
        assert!(rows(&oracle, None, None, None, Some(Some(&missing))).is_empty());

        let french: Term = Literal::new_language_tagged_literal("Un article", "fr")
            .unwrap()
            .into();
        let wrong_language: Term = Literal::new_language_tagged_literal("Un article", "en")
            .unwrap()
            .into();
        let typed: Term = Literal::new_typed_literal(
            "2026-07-04",
            NamedNode::new("http://www.w3.org/2001/XMLSchema#date").unwrap(),
        )
        .into();
        let reified = crate::dataset_quads(&graph)
            .into_iter()
            .find_map(|quad| matches!(quad.object, Term::Triple(_)).then_some(quad.object))
            .expect("fixture has an RDF 1.2 reified triple");
        for (object, scope) in [
            (&french, Some(None)),
            (&typed, Some(Some(&user))),
            (&reified, None),
        ] {
            let actual = rows(&adapter, None, None, Some(object), scope);
            assert!(!actual.is_empty(), "positive RDF term control {object}");
            assert_eq!(actual, rows(&oracle, None, None, Some(object), scope));
        }
        assert!(rows(&adapter, None, None, Some(&wrong_language), Some(None)).is_empty());
        assert!(rows(&oracle, None, None, Some(&wrong_language), Some(None)).is_empty());
    }

    #[test]
    fn adapter_deduplicates_base_quads_but_keeps_each_assertion_reifier() {
        let mut graph = Graph::new();
        let a = graph.add_node("https://same.test/".to_string(), Default::default());
        let duplicate = graph.add_node("https://same.test/".to_string(), Default::default());
        let b = graph.add_node("https://target.test/".to_string(), Default::default());
        graph.get_node_mut(a).unwrap().title = "Shared title".to_string();
        graph.get_node_mut(duplicate).unwrap().title = "Shared title".to_string();
        for asserter in ["https://people.test/alice", "https://people.test/bob"] {
            graph.assert_semantic_statement(
                a,
                b,
                SemanticStatementSpec {
                    predicate: "https://example.test/claims".to_string(),
                    graph_scope: GraphScope::User,
                    provenance_iri: Some(asserter.to_string()),
                    asserted_at_ms: Some(42),
                    ..Default::default()
                },
            );
        }
        assert!(graph.find_edge_key(a, b).is_none());
        let key = graph
            .find_resource_edge_key(
                graph.shown_resource_id(a).unwrap(),
                graph.shown_resource_id(b).unwrap(),
            )
            .expect("live claims belong to the shown resources");
        let statements = graph.get_resource_edge(key).unwrap().semantic_statements();
        assert_eq!(statements.len(), 2);
        let mut sources: Vec<_> = statements
            .iter()
            .map(|statement| {
                assert_eq!(statement.graph_scope, GraphScope::User);
                assert_eq!(statement.asserted_at_ms, Some(42));
                statement.provenance_iri.as_deref().unwrap()
            })
            .collect();
        sources.sort();
        assert_eq!(
            sources,
            ["https://people.test/alice", "https://people.test/bob"]
        );
        let controls = [
            (
                "SELECT ?title WHERE { <https://same.test/> <https://schema.org/name> ?title }",
                1,
            ),
            (
                "SELECT ?target WHERE { GRAPH <https://mere.computer/ns/graph#user> { <https://same.test> <https://example.test/claims> ?target } }",
                1,
            ),
            (
                "SELECT ?stmt ?author WHERE { GRAPH <https://mere.computer/ns/graph#user> { ?stmt <http://www.w3.org/ns/prov#wasAttributedTo> ?author } }",
                2,
            ),
        ];
        for (query, count) in controls {
            let actual = sorted(sparql(&graph, query).unwrap());
            assert_eq!(actual.rows.len(), count, "{query}");
            assert_eq!(actual, sorted(sparql_materialized(&graph, query).unwrap()));
            assert_eq!(
                actual,
                sorted(baseline::sparql_store(&graph, query).unwrap())
            );
        }
        for (query, expected) in [
            (
                "ASK { GRAPH <https://mere.computer/ns/graph#user> { <https://same.test> <https://example.test/claims> <https://target.test> } }",
                "true",
            ),
            (
                "ASK { GRAPH <https://mere.computer/ns/graph#user> { <https://same.test> <https://example.test/claims> <https://absent.test> } }",
                "false",
            ),
            (
                "ASK { GRAPH <https://mere.computer/ns/graph#user> { <https://same.test/> <https://example.test/claims> <https://target.test/> } }",
                "false",
            ),
        ] {
            assert_eq!(
                sparql(&graph, query).unwrap().rows,
                vec![vec![Some(expected.to_string())]],
                "canonical resource and raw Surface control: {query}"
            );
            assert_eq!(sparql(&graph, query), sparql_materialized(&graph, query));
            assert_eq!(sparql(&graph, query), baseline::sparql_store(&graph, query));
        }
    }

    #[test]
    fn explicit_resource_assertions_keep_canonical_pages_exact_terms_and_scopes() {
        use kernel::graph::resource::{ResourceNode, TAGGED_WITH_IRI};
        use kernel::persistence::{
            PersistedEdge, PersistedEdgeFamily, PersistedResourceRecord, PersistedSemanticEdgeData,
            PersistedSemanticStatement,
        };

        let page = ResourceNode::new("https://PAGE.test/item#first");
        let alias = ResourceNode::new("https://page.test/item#second");
        let cat = ResourceNode::for_term("https://vocabulary.test/ns#Cat");
        let dog = ResourceNode::for_term("https://vocabulary.test/ns#Dog");
        assert_eq!(page.id(), alias.id());
        assert_ne!(cat.id(), dog.id());
        let mut snapshot = Graph::new().to_snapshot();
        snapshot.resources = [&page, &cat, &dog]
            .map(|resource| PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().to_string(),
                facets: Vec::new(),
            })
            .to_vec();
        for (target, scope, authors) in [
            (&cat, GraphScope::User, vec!["alice", "bob"]),
            (&dog, GraphScope::Default, vec!["carol"]),
        ] {
            snapshot.resource_edges.push(PersistedEdge {
                from_node_id: page.id().to_string(),
                to_node_id: target.id().to_string(),
                families: vec![PersistedEdgeFamily::Semantic],
                semantic: Some(PersistedSemanticEdgeData {
                    statements: authors
                        .into_iter()
                        .enumerate()
                        .map(|(index, author)| PersistedSemanticStatement {
                            statement_id: format!("tag-{author}"),
                            predicate: TAGGED_WITH_IRI.to_string(),
                            recognized_sub_kind: None,
                            graph_scope: scope.clone(),
                            label: Some(format!("tag asserted by {author}")),
                            provenance_iri: Some(format!("https://people.test/{author}")),
                            asserted_at_ms: Some(42 + index as u64),
                        })
                        .collect(),
                    ..Default::default()
                }),
                traversal: None,
                containment: None,
                arrangement: None,
                imported: None,
                provenance: None,
            });
        }
        let graph = Graph::try_from_snapshot(&snapshot).unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            graph.node_count(),
            0,
            "resource projection needs no surface"
        );
        let controls = [
            (
                "SELECT ?target WHERE { GRAPH <https://mere.computer/ns/graph#user> { <https://page.test/item> <https://mere.computer/ns/rel#taggedWith> ?target } }",
                1,
            ),
            (
                "SELECT ?target WHERE { <https://page.test/item> <https://mere.computer/ns/rel#taggedWith> ?target }",
                1,
            ),
            (
                "SELECT ?stmt ?author ?time WHERE { GRAPH <https://mere.computer/ns/graph#user> { ?stmt <http://www.w3.org/ns/prov#wasAttributedTo> ?author ; <http://www.w3.org/ns/prov#generatedAtTime> ?time } }",
                2,
            ),
        ];
        for (query, count) in controls {
            let actual = sorted(sparql(&graph, query).unwrap());
            assert_eq!(actual.rows.len(), count, "{query}");
            assert_eq!(actual, sorted(sparql_materialized(&graph, query).unwrap()));
            assert_eq!(
                actual,
                sorted(baseline::sparql_store(&graph, query).unwrap())
            );
        }
        let default_targets = sparql(&graph, controls[1].0).unwrap();
        assert_eq!(
            default_targets.rows,
            vec![vec![Some(dog.canonical_iri().to_string())]]
        );
        let user_targets = sparql(&graph, controls[0].0).unwrap();
        assert_eq!(
            user_targets.rows,
            vec![vec![Some(cat.canonical_iri().to_string())]]
        );
        let absent = "ASK { <https://page.test/item> <https://mere.computer/ns/rel#taggedWith> <https://vocabulary.test/ns#Cat> }";
        assert_eq!(
            sparql(&graph, absent).unwrap().rows,
            vec![vec![Some("false".to_string())]]
        );
        assert_eq!(sparql(&graph, absent), sparql_materialized(&graph, absent));
        let quads = crate::dataset_quads(&graph);
        let reifiers: std::collections::HashSet<_> = quads
            .iter()
            .filter(|quad| quad.predicate.as_str() == crate::RDF_REIFIES)
            .map(|quad| quad.subject.clone())
            .collect();
        assert_eq!(reifiers.len(), 3);
    }

    /// Phase 3 gate (b): the direct evaluation path must not regress against
    /// the store-copy path it replaces. Debug-build wall clock over the
    /// parity battery; generous 3x headroom keeps this a regression tripwire,
    /// not a benchmark.
    #[test]
    fn spareval_is_not_slower_than_store_copy() {
        let graph = rich_graph();
        let battery = |run: &dyn Fn(&str) -> QueryRows| {
            let start = std::time::Instant::now();
            for _ in 0..20 {
                for query in PARITY_QUERIES {
                    run(query);
                }
            }
            start.elapsed()
        };
        let mainline = battery(&|q| sparql(&graph, q).expect(q));
        let oracle = battery(&|q| baseline::sparql_store(&graph, q).expect(q));
        println!("spareval {mainline:?} vs store-copy {oracle:?} over the parity battery x20");
        assert!(
            mainline < oracle * 3,
            "spareval path ({mainline:?}) should not be slower than 3x the store-copy path ({oracle:?})"
        );
    }
}
