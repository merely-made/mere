// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use oxrdf::NamedNode;

const LEGACY_PREFIX: &str = "urn:mere:statement:";
const ENCODED_PREFIX: &str = "urn:mere:statement-id:v1:";

pub(crate) fn statement_reifier_id(statement_id: &str) -> String {
    let legacy = format!("{LEGACY_PREFIX}{statement_id}");
    if NamedNode::new(legacy.clone()).is_ok() {
        return legacy;
    }
    let mut encoded = String::from(ENCODED_PREFIX);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in statement_id.bytes() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 15)]));
    }
    encoded
}

/// Distinguishes foreign reifiers from malformed handles in the reserved format.
pub(crate) fn statement_id_from_reifier(iri: &str) -> Result<Option<String>, &'static str> {
    if let Some(id) = iri.strip_prefix(LEGACY_PREFIX) {
        return Ok(Some(id.to_string()));
    }
    let Some(encoded) = iri.strip_prefix(ENCODED_PREFIX) else {
        return Ok(None);
    };
    if encoded.len() % 2 != 0 {
        return Err("encoded assertion id has an incomplete hexadecimal byte");
    }
    let mut bytes = Vec::with_capacity(encoded.len() / 2);
    for pair in encoded.as_bytes().chunks_exact(2) {
        let high = char::from(pair[0])
            .to_digit(16)
            .ok_or("encoded assertion id contains a non-hexadecimal byte")?;
        let low = char::from(pair[1])
            .to_digit(16)
            .ok_or("encoded assertion id contains a non-hexadecimal byte")?;
        bytes.push((high * 16 + low) as u8);
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| "encoded assertion id is not UTF-8")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        GRAPH_SCOPE_USER, PROV_GENERATED_AT_TIME, PROV_WAS_ATTRIBUTED_TO, RDF_REIFIES,
        asserted_at_literal,
    };
    use kernel::graph::fixtures::GraphFixtures;
    use kernel::graph::{Graph, SemanticStatement};
    use kernel::types::GraphScope;
    use oxrdf::{Quad, Term, Triple};

    fn reifier_quads(iri: &str) -> Vec<Quad> {
        let reifier = NamedNode::new(iri).unwrap();
        let scope = NamedNode::new(GRAPH_SCOPE_USER).unwrap();
        let subject = NamedNode::new("https://source.test/").unwrap();
        let predicate = NamedNode::new("https://example.test/claims").unwrap();
        let target = NamedNode::new("https://target.test/").unwrap();
        vec![
            Quad::new(
                subject.clone(),
                predicate.clone(),
                target.clone(),
                scope.clone(),
            ),
            Quad::new(
                reifier.clone(),
                NamedNode::new(RDF_REIFIES).unwrap(),
                Term::from(Triple::new(subject, predicate, target)),
                scope.clone(),
            ),
            Quad::new(
                reifier.clone(),
                NamedNode::new(PROV_WAS_ATTRIBUTED_TO).unwrap(),
                NamedNode::new("https://author.test/").unwrap(),
                scope.clone(),
            ),
            Quad::new(
                reifier,
                NamedNode::new(PROV_GENERATED_AT_TIME).unwrap(),
                asserted_at_literal(42).unwrap(),
                scope,
            ),
        ]
    }

    #[test]
    fn valid_legacy_ids_keep_their_iris_and_exact_handles() {
        for id in [
            "",
            "valid-handle",
            "猫",
            "statement-id:v1:6f70617175650a68616e646c65",
        ] {
            let iri = statement_reifier_id(id);
            assert_eq!(iri, format!("{LEGACY_PREFIX}{id}"));
            assert!(NamedNode::new(&iri).is_ok());
            assert_eq!(statement_id_from_reifier(&iri), Ok(Some(id.to_string())));
        }
    }

    #[test]
    fn unsafe_handles_have_distinct_reversible_valid_reifier_iris() {
        let handles = [
            " ",
            "  ",
            "opaque\nhandle",
            "猫\n犬",
            "\0",
            "urn:mere:statement-id:v1:20",
        ];
        let mut seen = std::collections::HashSet::new();
        for id in handles {
            let iri = statement_reifier_id(id);
            assert!(NamedNode::new(&iri).is_ok());
            assert!(
                seen.insert(iri.clone()),
                "different handles must not collide"
            );
            assert_eq!(statement_id_from_reifier(&iri), Ok(Some(id.to_string())));
            if id.contains([' ', '\n', '\0']) {
                assert!(iri.starts_with(ENCODED_PREFIX));
                assert!(!iri.starts_with(LEGACY_PREFIX));
            }
        }
        let unsafe_iri = statement_reifier_id("opaque\nhandle");
        for lookalike in [unsafe_iri.as_str(), "6f70617175650a68616e646c65"] {
            assert_ne!(statement_reifier_id(lookalike), unsafe_iri);
            assert_eq!(
                statement_id_from_reifier(&statement_reifier_id(lookalike)),
                Ok(Some(lookalike.to_string()))
            );
        }
    }

    #[test]
    fn decoder_distinguishes_foreign_and_malformed_reserved_inputs() {
        assert_eq!(
            statement_id_from_reifier("https://foreign.test/reifier"),
            Ok(None)
        );
        assert_eq!(
            statement_id_from_reifier("urn:mere:statement-id:v2:20"),
            Ok(None)
        );
        assert_eq!(
            statement_id_from_reifier("urn:mere:statement-id:v1:20"),
            Ok(Some(" ".to_string()))
        );
        for suffix in ["0", "gg", "ff"] {
            assert!(statement_id_from_reifier(&format!("{ENCODED_PREFIX}{suffix}")).is_err());
        }
    }

    #[test]
    fn production_ingest_rejects_malformed_reserved_ids_and_keeps_valid_controls() {
        for (iri, expected) in [
            ("urn:mere:statement:valid-handle", Some("valid-handle")),
            ("urn:mere:statement-id:v1:20", Some(" ")),
            ("https://foreign.test/reifier", None),
            ("urn:mere:statement-id:v2:20", None),
        ] {
            let parsed = crate::from_quads(reifier_quads(iri), "id-controls").unwrap();
            assert_eq!(parsed.edges.len(), 1);
            let edge = &parsed.edges[0];
            assert_eq!(edge.statement_id.as_deref(), expected);
            assert_eq!(edge.provenance_iri.as_deref(), Some("https://author.test/"));
            assert_eq!(edge.asserted_at_ms, Some(42));
            assert_eq!(edge.graph_scope, GraphScope::User);
        }
        for suffix in ["0", "gg", "ff"] {
            let invalid = reifier_quads(&format!("{ENCODED_PREFIX}{suffix}"));
            for invalid_first in [false, true] {
                let valid = reifier_quads("urn:mere:statement:valid-handle");
                let mixed = if invalid_first {
                    invalid.iter().cloned().chain(valid).collect::<Vec<_>>()
                } else {
                    valid.into_iter().chain(invalid.iter().cloned()).collect()
                };
                let error = crate::from_quads(mixed, "invalid-reserved-id")
                    .expect_err("mixed contribution must reject malformed reserved data");
                assert!(matches!(error, crate::IngestError::Parse(_)));
                assert!(error.to_string().contains("invalid assertion ID reifier"));
                assert!(
                    error
                        .to_string()
                        .contains(&format!("{ENCODED_PREFIX}{suffix}"))
                );
            }
        }
    }

    #[test]
    fn production_dataset_ingest_apply_preserves_arbitrary_assertion_handles() {
        use kernel::graph::resource::ResourceNode;
        use kernel::persistence::{
            PersistedEdge, PersistedEdgeFamily, PersistedResourceRecord, PersistedSemanticEdgeData,
            PersistedSemanticStatement,
        };

        let encoded_lookalike = statement_reifier_id("opaque\nhandle");
        let handles = [
            "valid-handle",
            "",
            " ",
            "  ",
            "opaque\nhandle",
            "\0",
            "猫",
            "猫\n犬",
            "urn:mere:statement-id:v1:20",
            encoded_lookalike.as_str(),
        ];
        let statements: Vec<_> = handles
            .iter()
            .enumerate()
            .map(|(index, handle)| SemanticStatement {
                statement_id: (*handle).to_string(),
                predicate: "https://example.test/claims".to_string(),
                recognized_sub_kind: None,
                label: Some(format!("assertion {index}")),
                graph_scope: GraphScope::User,
                provenance_iri: Some(format!("https://author.test/{index}")),
                asserted_at_ms: Some(42 + index as u64),
            })
            .collect();
        for resource_projection in [false, true] {
            let graph = if resource_projection {
                let from = ResourceNode::new("https://source.test/");
                let to = ResourceNode::new("https://target.test/");
                let mut snapshot = Graph::new().to_snapshot();
                snapshot.resources = [&from, &to]
                    .map(|resource| PersistedResourceRecord {
                        canonical_iri: resource.canonical_iri().to_string(),
                        facets: Vec::new(),
                    })
                    .to_vec();
                snapshot.resource_edges.push(PersistedEdge {
                    from_node_id: from.id().to_string(),
                    to_node_id: to.id().to_string(),
                    families: vec![PersistedEdgeFamily::Semantic],
                    semantic: Some(PersistedSemanticEdgeData {
                        statements: statements
                            .iter()
                            .map(|statement| PersistedSemanticStatement {
                                statement_id: statement.statement_id.clone(),
                                predicate: statement.predicate.clone(),
                                recognized_sub_kind: None,
                                label: statement.label.clone(),
                                graph_scope: statement.graph_scope.clone(),
                                provenance_iri: statement.provenance_iri.clone(),
                                asserted_at_ms: statement.asserted_at_ms,
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
                Graph::try_from_snapshot(&snapshot).unwrap_or_else(|error| panic!("{error}"))
            } else {
                let mut graph = Graph::new();
                let from = graph.add_node("https://source.test/".to_string(), Default::default());
                let to = graph.add_node("https://target.test/".to_string(), Default::default());
                for statement in &statements {
                    assert!(
                        graph
                            .assert_persisted_semantic_statement(from, to, statement.clone())
                            .is_some()
                    );
                }
                graph
            };
            let quads = crate::dataset_quads(&graph);
            let count = |predicate| {
                quads
                    .iter()
                    .filter(|quad| quad.predicate.as_str() == predicate)
                    .count()
            };
            assert_eq!(
                count("https://example.test/claims"),
                1,
                "shared base triple"
            );
            for predicate in [RDF_REIFIES, PROV_WAS_ATTRIBUTED_TO, PROV_GENERATED_AT_TIME] {
                assert_eq!(
                    count(predicate),
                    handles.len(),
                    "one metadata record per assertion"
                );
            }
            let contribution = crate::from_quads(quads.clone(), "arbitrary-id-roundtrip").unwrap();
            assert_eq!(contribution.edges.len(), handles.len());
            for statement in &statements {
                let edge = contribution
                    .edges
                    .iter()
                    .find(|edge| edge.statement_id.as_deref() == Some(&statement.statement_id))
                    .expect("exact handle survives ingest");
                assert_eq!(edge.label, statement.label);
                assert_eq!(edge.provenance_iri, statement.provenance_iri);
                assert_eq!(edge.asserted_at_ms, statement.asserted_at_ms);
                assert_eq!(edge.graph_scope, statement.graph_scope);
            }
            let text = crate::to_nquads(&graph);
            let text_contribution = crate::from_nquads(&text, "arbitrary-id-roundtrip").unwrap();
            assert_eq!(
                text_contribution, contribution,
                "encoded handles survive actual RDF file I/O"
            );
            let trig = crate::to_trig(&graph).unwrap();
            assert_eq!(
                crate::from_trig(&trig, "arbitrary-id-roundtrip").unwrap(),
                contribution,
                "encoded handles survive TriG file I/O"
            );
            #[cfg(feature = "query")]
            {
                let rows = crate::query::sparql(
                    &graph,
                    "SELECT ?stmt WHERE { GRAPH <https://mere.computer/ns/graph#user> { ?stmt <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?triple } }",
                ).unwrap();
                let actual: std::collections::BTreeSet<_> = rows
                    .rows
                    .into_iter()
                    .map(|row| row[0].clone().unwrap())
                    .collect();
                let expected: std::collections::BTreeSet<_> = handles
                    .iter()
                    .map(|handle| statement_reifier_id(handle))
                    .collect();
                assert_eq!(actual, expected, "borrowed SPARQL sees each exact reifier");
                let absent = crate::query::sparql(
                    &graph,
                    "ASK { ?stmt <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> ?triple }",
                )
                .unwrap();
                assert_eq!(
                    absent.rows,
                    vec![vec![Some("false".to_string())]],
                    "User assertions do not leak into Default scope"
                );
            }
            let mut reapplied = Graph::new();
            let outcome = crate::apply_contribution(&mut reapplied, &contribution);
            assert_eq!(outcome.edges_asserted, handles.len());
            assert_eq!(outcome.edges_skipped, 0);
            let normalized = |quads: Vec<Quad>| {
                let mut rows: Vec<_> = quads.into_iter().map(|quad| quad.to_string()).collect();
                rows.sort();
                rows
            };
            assert_eq!(
                normalized(crate::dataset_quads(&reapplied)),
                normalized(quads)
            );
        }
    }

    #[test]
    fn unsafe_literal_handles_retain_datatype_language_source_time_and_scope() {
        use kernel::types::NodeProperty;

        let mut graph = Graph::new();
        let source = graph.add_node("https://source.test/".to_string(), Default::default());
        let properties = vec![
            NodeProperty {
                statement_id: "typed\nhandle".to_string(),
                predicate: "https://schema.org/datePublished".to_string(),
                value: "2026-10-06".to_string(),
                datatype: Some("http://www.w3.org/2001/XMLSchema#date".to_string()),
                lang: None,
                graph_scope: GraphScope::User,
                provenance_iri: Some("https://author.test/date".to_string()),
                asserted_at_ms: Some(42),
            },
            NodeProperty {
                statement_id: "language\0handle".to_string(),
                predicate: "https://schema.org/abstract".to_string(),
                value: "Un article".to_string(),
                datatype: None,
                lang: Some("fr".to_string()),
                graph_scope: GraphScope::Source,
                provenance_iri: Some("https://author.test/abstract".to_string()),
                asserted_at_ms: Some(43),
            },
        ];
        assert!(graph.append_node_properties(source, properties.clone()));
        let quads = crate::dataset_quads(&graph);
        let parsed = crate::from_quads(quads, "literal-ids").unwrap();
        assert!(parsed.edges.is_empty());
        assert_eq!(parsed.nodes.len(), 1);
        assert_eq!(parsed.nodes[0].properties.len(), properties.len());
        for property in &properties {
            assert!(
                parsed.nodes[0].properties.contains(property),
                "exact literal assertion survives"
            );
        }
        for from_file in [
            crate::from_nquads(&crate::to_nquads(&graph), "literal-ids").unwrap(),
            crate::from_trig(&crate::to_trig(&graph).unwrap(), "literal-ids").unwrap(),
        ] {
            assert_eq!(from_file, parsed);
        }
        let mut reapplied = Graph::new();
        crate::apply_contribution(&mut reapplied, &parsed);
        let key = reapplied.nodes().next().expect("literal source restored").0;
        let restored = reapplied.node_properties(key).unwrap();
        assert_eq!(restored.len(), properties.len());
        for property in properties {
            assert!(restored.contains(&property));
        }
    }
}
