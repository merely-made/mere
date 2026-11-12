// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use crate::graph::apply::assert_semantic_relation_in_scope;
use crate::graph::{SemanticStatementSpec, SemanticSubKind};
use crate::types::{GraphScope, NodeProperty};
use mere::query::*;
use oxrdf::Term;

#[test]
fn hash_functions_return_known_digests() {
    let graph = Graph::new();
    for (function, expected) in [
        ("MD5", "900150983cd24fb0d6963f7d28e17f72"),
        ("SHA1", "a9993e364706816aba3e25717850c26c9cd0d89d"),
        (
            "SHA256",
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        ),
        (
            "SHA384",
            "cb00753f45a35e8bb5a03d699ac65007272c32ab0eded1631a8b605a43ff5bed8086072ba1e7cc2358baeca134c825a7",
        ),
        (
            "SHA512",
            "ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f",
        ),
    ] {
        let query = format!("SELECT ({function}(\"abc\") AS ?hash) WHERE {{}}");
        let rows = sparql(&graph, &query).expect(function);
        assert_eq!(rows.variables, ["hash"]);
        assert_eq!(
            rows.rows,
            vec![vec![Some(expected.to_string())]],
            "{function}"
        );
    }
}

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
            basis: Default::default(),
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

/// The borrowed adapter returns the same solutions as the materialized
/// spareval dataset on every representative query.
#[test]
fn adapter_rows_match_materialized_dataset() {
    let graph = rich_graph();
    for query in PARITY_QUERIES {
        let mainline = sorted(sparql(&graph, query).expect(query));
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
fn feature_compat_adjust_preserves_timezone_conversion() {
    let graph = Graph::new();
    let query = r#"PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
            SELECT (ADJUST("2026-10-09T12:00:00Z"^^xsd:dateTime,
                           "PT2H"^^xsd:dayTimeDuration) AS ?adjusted)
            WHERE {}"#;
    let expected = QueryRows {
        variables: vec!["adjusted".to_string()],
        rows: vec![vec![Some("2026-10-09T14:00:00+02:00".to_string())]],
        coverage: graph.coverage_note(),
    };
    for (path, actual) in [
        ("adapter", sparql(&graph, query)),
        ("materialized", sparql_materialized(&graph, query)),
    ] {
        assert_eq!(actual, Ok(expected.clone()), "{path} ADJUST result");
    }
}

#[test]
fn feature_compat_lateral_preserves_per_row_correlation() {
    let graph = Graph::new();
    let query = "SELECT ?value ?next WHERE {
            VALUES ?value { 1 2 }
            LATERAL { SELECT ?value (?value + 1 AS ?next) WHERE {} LIMIT 1 }
        } ORDER BY ?value";
    let expected = QueryRows {
        variables: vec!["value".to_string(), "next".to_string()],
        rows: vec![
            vec![Some("1".to_string()), Some("2".to_string())],
            vec![Some("2".to_string()), Some("3".to_string())],
        ],
        coverage: graph.coverage_note(),
    };
    for (path, actual) in [
        ("adapter", sparql(&graph, query)),
        ("materialized", sparql_materialized(&graph, query)),
    ] {
        assert_eq!(actual, Ok(expected.clone()), "{path} LATERAL result");
    }
}

#[test]
fn feature_compat_calendar_preserves_year_month_extraction() {
    let graph = Graph::new();
    let query = r#"PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>
            SELECT (YEAR("2026-10"^^xsd:gYearMonth) AS ?year)
                   (MONTH("2026-10"^^xsd:gYearMonth) AS ?month)
            WHERE {}"#;
    let expected = QueryRows {
        variables: vec!["year".to_string(), "month".to_string()],
        rows: vec![vec![Some("2026".to_string()), Some("10".to_string())]],
        coverage: graph.coverage_note(),
    };
    for (path, actual) in [
        ("adapter", sparql(&graph, query)),
        ("materialized", sparql_materialized(&graph, query)),
    ] {
        assert_eq!(actual, Ok(expected.clone()), "{path} calendar result");
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
    let indexed = IndexedDataset::for_graph(&graph);
    let adapter = indexed.read(View::Live);
    let materialized: oxrdf::Dataset = mere::dataset_quads(&graph).into_iter().collect();
    let oracle = &materialized;
    let user: Term = NamedNode::new(mere::GRAPH_SCOPE_USER).unwrap().into();
    let source: Term = NamedNode::new(mere::GRAPH_SCOPE_SOURCE).unwrap().into();
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
    let reified = mere::dataset_quads(&graph)
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
    }
}

#[test]
fn explicit_resource_assertions_keep_canonical_pages_exact_terms_and_scopes() {
    use crate::graph::resource::{ResourceNode, TAGGED_WITH_IRI};
    use crate::persistence::{
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
            assertions: Some(
                authors
                    .into_iter()
                    .enumerate()
                    .map(|(index, author)| PersistedSemanticStatement {
                        statement_id: format!("tag-{author}"),
                        predicate: TAGGED_WITH_IRI.to_string(),
                        recognized_sub_kind: None,
                        graph_scope: scope.clone(),
                        label: Some(format!("tag asserted by {author}")),
                        basis: Default::default(),
                        provenance_iri: Some(format!("https://people.test/{author}")),
                        asserted_at_ms: Some(42 + index as u64),
                        first_asserted_ms: Some(42 + index as u64),
                        withdrawn: None,
                    })
                    .collect(),
            ),
            from_node_id: page.id().to_string(),
            to_node_id: target.id().to_string(),
            families: vec![PersistedEdgeFamily::Semantic],
            semantic: None,
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
    let quads = mere::dataset_quads(&graph);
    let reifiers: std::collections::HashSet<_> = quads
        .iter()
        .filter(|quad| quad.predicate.as_str() == mere::RDF_REIFIES)
        .map(|quad| quad.subject.clone())
        .collect();
    assert_eq!(reifiers.len(), 3);
}
