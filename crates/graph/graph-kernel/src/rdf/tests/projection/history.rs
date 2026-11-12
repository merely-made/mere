// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[test]
fn default_export_omits_withdrawn_surface_and_resource_assertions() {
    use crate::graph::{RelationKey, SemanticStatementSpec, predicate_iri};

    let mut graph = Graph::new();
    let a = graph.add_node("https://a.test".to_string(), Default::default());
    let b = graph.add_node("https://b.test".to_string(), Default::default());
    let surface_predicate = predicate_iri(SemanticSubKind::UserGrouped);
    let (_, surface) = graph
        .try_assert_semantic_statement(
            a,
            b,
            SemanticStatementSpec {
                predicate: surface_predicate.into(),
                recognized_sub_kind: Some(SemanticSubKind::UserGrouped),
                provenance_iri: Some("urn:author:surface".into()),
                ..Default::default()
            },
        )
        .unwrap();
    let resource_predicate = predicate_iri(SemanticSubKind::Cites);
    let (resource_owner, resource) = graph
        .try_assert_semantic_statement(
            a,
            b,
            SemanticStatementSpec {
                predicate: resource_predicate.into(),
                recognized_sub_kind: Some(SemanticSubKind::Cites),
                provenance_iri: Some("urn:author:resource".into()),
                ..Default::default()
            },
        )
        .unwrap();
    assert!(matches!(resource_owner, RelationKey::Resource(_)));

    assert!(graph.withdraw_semantic_assertion(
        &surface.statement_id,
        Some(10),
        Some("urn:author:surface".into()),
    ));
    assert!(graph.withdraw_semantic_assertion(
        &resource.statement_id,
        Some(11),
        Some("urn:author:resource".into()),
    ));

    let (a, node_a) = graph.get_node_by_url("https://a.test").expect("node A");
    let direct = node_direct_quads(&graph, a, node_a);
    assert!(
        !direct
            .iter()
            .any(|quad| quad.predicate.as_str() == surface_predicate)
    );
    let dataset = dataset_quads(&graph);
    assert!(!dataset.iter().any(|quad| {
        quad.predicate.as_str() == surface_predicate
            || quad.predicate.as_str() == resource_predicate
    }));
    let surface_reifier = NamedNode::new(statement_reifier_id(&surface.statement_id)).unwrap();
    assert!(
        dataset
            .iter()
            .all(|quad| { quad.subject != oxrdf::NamedOrBlankNode::from(surface_reifier.clone()) })
    );
    assert!(dataset.iter().all(|quad| {
        !matches!(&quad.object, Term::Triple(triple)
            if triple.predicate.as_str() == surface_predicate
                || triple.predicate.as_str() == resource_predicate)
    }));
    assert!(
        dataset
            .iter()
            .all(|quad| quad.predicate.as_str() != RDF_REIFIES)
    );

    let history = dataset_quads_with_history(&graph);
    let history_graph = GraphName::NamedNode(NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI).unwrap());
    let resource_reifier = NamedNode::new(statement_reifier_id(&resource.statement_id)).unwrap();
    for (reifier, expected_predicate) in [
        (surface_reifier.clone(), surface_predicate),
        (resource_reifier.clone(), resource_predicate),
    ] {
        assert!(history.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == RDF_REIFIES
                && quad.graph_name == history_graph
                && matches!(&quad.object, Term::Triple(triple)
                    if triple.predicate.as_str() == expected_predicate)
        }));
        assert!(history.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(reifier.clone())
                || quad.graph_name == history_graph
        }));
    }
}

#[test]
fn all_family_assertions_export_distinct_reifiers_and_withdrawal_history() {
    use crate::graph::{ContainmentSubKind, ImportedSubKind, RelationKind, SemanticStatement};
    use crate::types::{AssertionBasis, AssertionMethod};

    let mut graph = Graph::new();
    let from = graph.add_node("https://family.test/from".into(), Default::default());
    let to = graph.add_node("https://family.test/to".into(), Default::default());
    let predicate = RelationKind::Containment(ContainmentSubKind::CollectionMember)
        .predicate_iri()
        .expect("canonical containment predicate")
        .to_owned();
    let actor = "urn:family:actor";
    let make_statement =
        |id: &str, note: &str, method: &str, evidence: &str, at| SemanticStatement {
            statement_id: id.into(),
            predicate: predicate.clone(),
            recognized_sub_kind: None,
            label: Some(note.into()),
            basis: AssertionBasis {
                note: Some(note.into()),
                method: Some(AssertionMethod {
                    iri: method.into(),
                    version: Some("1".into()),
                }),
                evidence: vec![evidence.into()],
            },
            graph_scope: GraphScope::Custom("urn:family:scope".into()),
            provenance_iri: Some(actor.into()),
            first_asserted_ms: Some(at),
            asserted_at_ms: Some(at),
            withdrawn: None,
        };
    for statement in [
        make_statement(
            "family:primary",
            "primary reason",
            "urn:method:primary",
            "urn:evidence:primary",
            10,
        ),
        make_statement(
            "family:independent",
            "independent reason",
            "urn:method:independent",
            "urn:evidence:independent",
            11,
        ),
        make_statement(
            "family:withdrawn",
            "withdrawn reason",
            "urn:method:withdrawn",
            "urn:evidence:withdrawn",
            12,
        ),
    ] {
        graph
            .try_assert_persisted_relation_statement(from, to, statement)
            .expect("distinct exact family assertions are retained");
    }
    let resource_id = "family:resource-control";
    let resource_kind = RelationKind::Imported(ImportedSubKind::SharedCollection);
    graph
        .try_assert_persisted_relation_statement(
            from,
            to,
            SemanticStatement {
                statement_id: resource_id.into(),
                predicate: resource_kind
                    .predicate_iri()
                    .expect("canonical imported predicate")
                    .into(),
                recognized_sub_kind: None,
                label: Some("resource family control".into()),
                basis: AssertionBasis {
                    note: Some("resource family control".into()),
                    method: None,
                    evidence: vec!["urn:evidence:resource-family".into()],
                },
                graph_scope: GraphScope::Custom("urn:family:scope".into()),
                provenance_iri: Some(actor.into()),
                first_asserted_ms: Some(13),
                asserted_at_ms: Some(13),
                withdrawn: None,
            },
        )
        .expect("exact resource-family assertion is retained");
    assert!(graph.withdraw_semantic_assertion(
        "family:withdrawn",
        Some(20),
        Some("urn:family:withdrawer".into()),
    ));

    let held = dataset_quads(&graph);
    let expanded = to_jsonld(&graph).to_string();
    assert!(expanded.contains(&statement_reifier_id("family:primary")));
    assert!(expanded.contains(&statement_reifier_id("family:independent")));
    assert!(expanded.contains(&statement_reifier_id(resource_id)));
    assert!(!expanded.contains(&statement_reifier_id("family:withdrawn")));
    #[cfg(feature = "query")]
    {
        let rows = mere::query::sparql(
            &graph,
            &format!(
                "SELECT ?target WHERE {{ GRAPH <urn:family:scope> {{ <https://family.test/from> <{predicate}> ?target }} }}"
            ),
        )
        .expect("SPARQL reads the canonical family assertion projection");
        assert_eq!(rows.rows, vec![vec![Some("https://family.test/to".into())]]);
    }
    let scope = GraphName::NamedNode(NamedNode::new("urn:family:scope").unwrap());
    let from_id = NamedNode::new("https://family.test/from").unwrap();
    let to_id = NamedNode::new("https://family.test/to").unwrap();
    assert!(held.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(from_id.clone())
            && quad.predicate.as_str() == predicate
            && quad.object == Term::NamedNode(to_id.clone())
            && quad.graph_name == scope
    }));
    // RDF facts are a set, while independent source reasons retain one
    // reifier each with their own immutable IDs and metadata.
    for (id, note, method, evidence) in [
        (
            "family:primary",
            "primary reason",
            "urn:method:primary",
            "urn:evidence:primary",
        ),
        (
            "family:independent",
            "independent reason",
            "urn:method:independent",
            "urn:evidence:independent",
        ),
    ] {
        let reifier = NamedNode::new(statement_reifier_id(id)).unwrap();
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == RDF_REIFIES
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == RDFS_LABEL
                && matches!(&quad.object, Term::Literal(value) if value.value() == note)
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_GENERATED_BY
                && quad.object == Term::NamedNode(NamedNode::new(method).unwrap())
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == MERE_METHOD_VERSION
                && matches!(&quad.object, Term::Literal(value) if value.value() == "1")
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_DERIVED_FROM
                && quad.object == Term::NamedNode(NamedNode::new(evidence).unwrap())
        }));
        assert!(held.iter().any(|quad| {
            quad.subject == oxrdf::NamedOrBlankNode::from(reifier.clone())
                && quad.predicate.as_str() == PROV_WAS_ATTRIBUTED_TO
                && quad.object == Term::NamedNode(NamedNode::new(actor).unwrap())
        }));
    }
    let resource_reifier = NamedNode::new(statement_reifier_id(resource_id)).unwrap();
    let resource_predicate = resource_kind.predicate_iri().unwrap();
    assert!(held.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(from_id.clone())
            && quad.predicate.as_str() == resource_predicate
            && quad.object == Term::NamedNode(to_id.clone())
            && quad.graph_name == scope
    }));
    assert!(held.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(resource_reifier.clone())
            && quad.predicate.as_str() == RDF_REIFIES
    }));
    assert!(held.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(resource_reifier.clone())
            && quad.predicate.as_str() == PROV_WAS_ATTRIBUTED_TO
            && quad.object == Term::NamedNode(NamedNode::new(actor).unwrap())
    }));
    let withdrawn_reifier = NamedNode::new(statement_reifier_id("family:withdrawn")).unwrap();
    assert!(
        held.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
        })
    );

    let history = dataset_quads_with_history(&graph);
    assert!(
        to_jsonld_with_history(&graph)
            .to_string()
            .contains(&statement_reifier_id("family:withdrawn"))
    );
    let history_graph = GraphName::NamedNode(NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI).unwrap());
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == RDF_REIFIES
            && quad.graph_name == history_graph
            && matches!(&quad.object, Term::Triple(triple)
                if triple.predicate.as_str() == predicate)
    }));
    let activity = NamedNode::new(withdrawal_activity_id("family:withdrawn")).unwrap();
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == PROV_WAS_INVALIDATED_BY
            && quad.object == Term::NamedNode(activity.clone())
            && quad.graph_name == history_graph
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == PROV_WAS_ATTRIBUTED_TO
            && quad.object == Term::NamedNode(NamedNode::new(actor).unwrap())
            && quad.graph_name == history_graph
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == PROV_INVALIDATED_AT_TIME
            && quad.graph_name == history_graph
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(activity.clone())
            && quad.predicate.as_str() == PROV_WAS_ASSOCIATED_WITH
            && quad.object == Term::NamedNode(NamedNode::new("urn:family:withdrawer").unwrap())
            && quad.graph_name == history_graph
    }));
    assert!(history.iter().all(|quad| {
        !(quad.subject == oxrdf::NamedOrBlankNode::from(from_id.clone())
            && quad.predicate.as_str() == predicate
            && quad.object == Term::NamedNode(to_id.clone())
            && quad.graph_name == history_graph)
    }));
}

#[test]
fn explicit_history_roundtrips_lifecycle_basis_and_original_scope() {
    use crate::graph::{Author, SemanticStatement};
    use crate::types::{AssertionBasis, AssertionMethod, AssertionWithdrawal};

    let mut graph = Graph::new();
    let from = graph.add_node("https://history.test/from".into(), Default::default());
    let to = graph.add_node("https://history.test/to".into(), Default::default());
    let predicate = crate::graph::predicate_iri(SemanticSubKind::Cites).to_string();
    let id = "rdf:history/withdrawn:one";
    let basis = AssertionBasis {
        note: Some("checked against source".into()),
        method: Some(AssertionMethod {
            iri: "urn:method:extractor".into(),
            version: Some("2.1".into()),
        }),
        evidence: vec!["https://evidence.test/item".into()],
    };
    let withdrawn = SemanticStatement {
        statement_id: id.into(),
        predicate: predicate.clone(),
        recognized_sub_kind: Some(SemanticSubKind::Cites),
        label: basis.note.clone(),
        basis: basis.clone(),
        graph_scope: GraphScope::Custom("https://scope.test/original".into()),
        provenance_iri: Some("urn:author:asserted".into()),
        first_asserted_ms: Some(10),
        asserted_at_ms: Some(20),
        withdrawn: Some(AssertionWithdrawal {
            at_ms: Some(30),
            by: Some("urn:author:withdrew".into()),
        }),
    };
    graph.write_as(Author::person("urn:author:host"), |graph| {
        graph
            .try_assert_persisted_semantic_statement(from, to, withdrawn)
            .expect("store exact tombstone");
    });

    let undated_id = "history:undated-default";
    let undated_predicate = "urn:predicate:undated-history";
    let undated = SemanticStatement {
        statement_id: undated_id.into(),
        predicate: undated_predicate.into(),
        recognized_sub_kind: None,
        label: None,
        basis: AssertionBasis::default(),
        graph_scope: GraphScope::Default,
        provenance_iri: None,
        first_asserted_ms: None,
        asserted_at_ms: None,
        withdrawn: Some(AssertionWithdrawal {
            at_ms: None,
            by: None,
        }),
    };
    graph
        .try_assert_persisted_semantic_statement(from, to, undated)
        .expect("store undated default-scope tombstone");

    let held_id = "held:partial-time";
    let mut held = SemanticStatement::new(
        predicate.clone(),
        Some(SemanticSubKind::Cites),
        None,
        GraphScope::Default,
        Some("urn:author:held".into()),
        Some(42),
    );
    held.statement_id = held_id.into();
    held.first_asserted_ms = None;
    graph
        .try_assert_persisted_semantic_statement(from, to, held)
        .expect("store held partial-time control");

    let default_dataset = dataset_quads(&graph);
    let withdrawn_reifier = NamedNode::new(statement_reifier_id(id)).unwrap();
    let from_id = node_id(graph.get_node(from).unwrap());
    let to_id = node_id(graph.get_node(to).unwrap());
    assert!(
        default_dataset.iter().all(|quad| {
            quad.subject != oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
        })
    );
    assert!(default_dataset.iter().any(|quad| {
        quad.subject.to_string() == format!("<{from_id}>")
            && quad.predicate.as_str() == predicate
            && quad.object == Term::NamedNode(NamedNode::new_unchecked(to_id.clone()))
    }));
    let held_reifier = NamedNode::new(statement_reifier_id(held_id)).unwrap();
    assert!(default_dataset.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(held_reifier.clone())
            && quad.predicate.as_str() == MERE_LIFECYCLE_VERSION
            && matches!(&quad.object, Term::Literal(literal) if literal.value() == "1")
    }));

    let history = dataset_quads_with_history(&graph);
    let history_graph = GraphName::NamedNode(NamedNode::new(ASSERTION_HISTORY_GRAPH_IRI).unwrap());
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == RDF_REIFIES
            && quad.graph_name == history_graph
    }));
    let undated_reifier = NamedNode::new(statement_reifier_id(undated_id)).unwrap();
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(undated_reifier.clone())
            && quad.predicate.as_str() == MERE_ASSERTED_GRAPH_KIND
            && matches!(&quad.object, Term::Literal(literal) if literal.value() == "default")
    }));
    assert!(history.iter().all(|quad| {
        quad.subject != oxrdf::NamedOrBlankNode::from(undated_reifier.clone())
            || !matches!(
                quad.predicate.as_str(),
                PROV_GENERATED_AT_TIME | MERE_LAST_ASSERTED_AT | PROV_INVALIDATED_AT_TIME
            )
    }));
    assert!(history.iter().all(|quad| {
        !(quad.subject.to_string() == format!("<{from_id}>")
            && quad.predicate.as_str() == predicate
            && quad.object == Term::NamedNode(NamedNode::new_unchecked(to_id.clone()))
            && quad.graph_name
                == GraphName::NamedNode(NamedNode::new("https://scope.test/original").unwrap()))
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == MERE_ASSERTED_GRAPH_KIND
            && matches!(&quad.object, Term::Literal(literal) if literal.value() == "custom")
            && quad.graph_name == history_graph
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == MERE_ASSERTED_GRAPH_NAME
            && matches!(&quad.object, Term::Literal(literal) if literal.value() == "https://scope.test/original")
            && quad.graph_name == history_graph
    }));
    let activity = NamedNode::new(withdrawal_activity_id(id)).unwrap();
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == PROV_WAS_INVALIDATED_BY
            && quad.object == Term::NamedNode(activity.clone())
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(activity.clone())
            && quad.predicate.as_str() == PROV_WAS_ASSOCIATED_WITH
            && quad.object == Term::NamedNode(NamedNode::new_unchecked("urn:author:withdrew"))
    }));
    assert!(history.iter().any(|quad| {
        quad.subject == oxrdf::NamedOrBlankNode::from(withdrawn_reifier.clone())
            && quad.predicate.as_str() == MERE_METHOD_VERSION
            && matches!(&quad.object, Term::Literal(literal) if literal.value() == "2.1")
    }));

    let nquads = to_nquads_with_history(&graph);
    let envelope = from_nquads_envelope(&nquads, "history-nquads").unwrap();
    let mut restored = Graph::new();
    apply_import(&mut restored, &envelope);
    let (_, restored_tombstone) = restored.find_semantic_statement(id).unwrap();
    assert_eq!(restored_tombstone.first_asserted_ms, Some(10));
    assert_eq!(restored_tombstone.asserted_at_ms, Some(20));
    assert_eq!(
        restored_tombstone.withdrawn.as_ref().unwrap().at_ms,
        Some(30)
    );
    assert_eq!(
        restored_tombstone.withdrawn.as_ref().unwrap().by.as_deref(),
        Some("urn:author:withdrew")
    );
    assert_eq!(
        restored_tombstone.graph_scope,
        GraphScope::Custom("https://scope.test/original".into())
    );
    assert_eq!(restored_tombstone.basis, basis);
    let (_, restored_undated) = restored.find_semantic_statement(undated_id).unwrap();
    assert_eq!(restored_undated.first_asserted_ms, None);
    assert_eq!(restored_undated.asserted_at_ms, None);
    assert_eq!(restored_undated.withdrawn.as_ref().unwrap().at_ms, None);
    assert_eq!(restored_undated.withdrawn.as_ref().unwrap().by, None);
    assert_eq!(restored_undated.graph_scope, GraphScope::Default);
    let (_, restored_held) = restored.find_semantic_statement(held_id).unwrap();
    assert_eq!(restored_held.first_asserted_ms, None);
    assert_eq!(restored_held.asserted_at_ms, Some(42));

    let trig = to_trig_with_history(&graph).expect("serialize explicit history TriG");
    let envelope = from_trig_envelope(&trig, "history-trig").unwrap();
    let mut restored_trig = Graph::new();
    apply_import(&mut restored_trig, &envelope);
    let (_, restored_trig_tombstone) = restored_trig.find_semantic_statement(id).unwrap();
    assert_eq!(restored_trig_tombstone.first_asserted_ms, Some(10));
    assert_eq!(restored_trig_tombstone.asserted_at_ms, Some(20));
    assert_eq!(
        restored_trig_tombstone.withdrawn.as_ref().unwrap().at_ms,
        Some(30)
    );
    assert_eq!(
        restored_trig_tombstone.graph_scope,
        GraphScope::Custom("https://scope.test/original".into())
    );
    assert_eq!(restored_trig_tombstone.basis, basis);

    let jsonld = to_jsonld_with_history(&graph);
    let envelope =
        mere::from_jsonld_envelope(serde_json::to_string(&jsonld).unwrap().as_bytes()).unwrap();
    let mut restored_jsonld = Graph::new();
    apply_import(&mut restored_jsonld, &envelope);
    let (_, restored_tombstone) = restored_jsonld.find_semantic_statement(id).unwrap();
    assert_eq!(
        restored_tombstone.withdrawn.as_ref().unwrap().at_ms,
        Some(30)
    );
    assert_eq!(
        restored_tombstone.graph_scope,
        GraphScope::Custom("https://scope.test/original".into())
    );
}
