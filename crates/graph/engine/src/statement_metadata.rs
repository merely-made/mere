// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

#[allow(
    clippy::too_many_arguments,
    reason = "the helper mirrors the complete RDF statement metadata tuple"
)]
pub fn push_statement_metadata_quads(
    quads: &mut Vec<Quad>,
    subject: &NamedNode,
    statement_id: &str,
    predicate: &str,
    object: Term,
    graph_scope: &GraphScope,
    label: Option<&str>,
    provenance_iri: Option<&str>,
    asserted_at_ms: Option<u64>,
) {
    let Ok(reifier) = NamedNode::new(statement_reifier_id(statement_id)) else {
        return;
    };
    let (Ok(predicate_node), Some(graph_name), Ok(rdf_reifies)) = (
        NamedNode::new(predicate),
        graph_name_for_scope(graph_scope),
        NamedNode::new(RDF_REIFIES),
    ) else {
        return;
    };
    quads.push(Quad::new(
        reifier.clone(),
        rdf_reifies,
        Term::from(Triple::new(subject.clone(), predicate_node, object)),
        graph_name.clone(),
    ));
    if let Some(label) = label
        && let Ok(predicate) = NamedNode::new(RDFS_LABEL)
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(Literal::new_simple_literal(label)),
            graph_name.clone(),
        ));
    }
    if let Some(provenance_iri) = provenance_iri
        && let (Ok(predicate), Ok(agent)) = (
            NamedNode::new(PROV_WAS_ATTRIBUTED_TO),
            NamedNode::new(provenance_iri),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(agent),
            graph_name.clone(),
        ));
    }
    if let Some(asserted_at_ms) = asserted_at_ms
        && let (Ok(predicate), Some(literal)) = (
            NamedNode::new(PROV_GENERATED_AT_TIME),
            asserted_at_literal(asserted_at_ms),
        )
    {
        quads.push(Quad::new(
            reifier,
            predicate,
            Term::from(literal),
            graph_name,
        ));
    }
}

pub fn push_semantic_statement_metadata_quads(
    quads: &mut Vec<Quad>,
    subject: &NamedNode,
    predicate: &str,
    object: Term,
    statement: &SemanticStatement,
    graph_name: GraphName,
    include_original_scope: bool,
) {
    let Ok(reifier) = NamedNode::new(statement_reifier_id(&statement.statement_id)) else {
        return;
    };
    let Ok(predicate_node) = NamedNode::new(predicate) else {
        return;
    };
    let Ok(rdf_reifies) = NamedNode::new(RDF_REIFIES) else {
        return;
    };

    quads.push(Quad::new(
        reifier.clone(),
        rdf_reifies,
        Term::from(Triple::new(subject.clone(), predicate_node, object)),
        graph_name.clone(),
    ));

    if statement.first_asserted_ms != statement.asserted_at_ms {
        push_reifier_literal(
            quads,
            &reifier,
            MERE_LIFECYCLE_VERSION,
            "1".into(),
            &graph_name,
        );
    }

    if include_original_scope {
        let (kind, name) = graph_scope_marker(&statement.graph_scope);
        push_reifier_literal(
            quads,
            &reifier,
            MERE_ASSERTED_GRAPH_KIND,
            kind.into(),
            &graph_name,
        );
        if let Some(name) = name {
            push_reifier_literal(quads, &reifier, MERE_ASSERTED_GRAPH_NAME, name, &graph_name);
        }
    }

    if let Some(label) = statement.basis.note.as_deref()
        && let Ok(predicate) = NamedNode::new(RDFS_LABEL)
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(Literal::new_simple_literal(label)),
            graph_name.clone(),
        ));
    }
    if let Some(provenance_iri) = statement.provenance_iri.as_deref()
        && let (Ok(predicate), Ok(agent)) = (
            NamedNode::new(PROV_WAS_ATTRIBUTED_TO),
            NamedNode::new(provenance_iri),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(agent),
            graph_name.clone(),
        ));
    }
    if let Some(first_asserted_ms) = statement.first_asserted_ms
        && let (Ok(predicate), Some(literal)) = (
            NamedNode::new(PROV_GENERATED_AT_TIME),
            asserted_at_literal(first_asserted_ms),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(literal),
            graph_name.clone(),
        ));
    }
    if let Some(last_asserted_ms) = statement.asserted_at_ms
        && let (Ok(predicate), Some(literal)) = (
            NamedNode::new(MERE_LAST_ASSERTED_AT),
            asserted_at_literal(last_asserted_ms),
        )
    {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(literal),
            graph_name.clone(),
        ));
    }
    if let Some(method) = &statement.basis.method {
        if let (Ok(predicate), Ok(method_iri)) = (
            NamedNode::new(PROV_WAS_GENERATED_BY),
            NamedNode::new(method.iri.clone()),
        ) {
            quads.push(Quad::new(
                reifier.clone(),
                predicate,
                Term::from(method_iri),
                graph_name.clone(),
            ));
        }
        if let Some(version) = &method.version {
            push_reifier_literal(
                quads,
                &reifier,
                MERE_METHOD_VERSION,
                version.clone(),
                &graph_name,
            );
        }
    }
    for evidence in &statement.basis.evidence {
        if let (Ok(predicate), Ok(evidence)) = (
            NamedNode::new(PROV_WAS_DERIVED_FROM),
            NamedNode::new(evidence.clone()),
        ) {
            quads.push(Quad::new(
                reifier.clone(),
                predicate,
                Term::from(evidence),
                graph_name.clone(),
            ));
        }
    }
    if let Some(withdrawal) = &statement.withdrawn {
        if let Ok(predicate) = NamedNode::new(PROV_WAS_INVALIDATED_BY)
            && let Ok(activity) = NamedNode::new(withdrawal_activity_id(&statement.statement_id))
        {
            quads.push(Quad::new(
                reifier.clone(),
                predicate,
                Term::from(activity.clone()),
                graph_name.clone(),
            ));
            if let Ok(predicate) = NamedNode::new(RDF_TYPE) {
                quads.push(Quad::new(
                    activity.clone(),
                    predicate,
                    Term::from(NamedNode::new_unchecked(PROV_ACTIVITY)),
                    graph_name.clone(),
                ));
            }
            if let Some(by) = &withdrawal.by
                && let (Ok(predicate), Ok(agent)) = (
                    NamedNode::new(PROV_WAS_ASSOCIATED_WITH),
                    NamedNode::new(by.clone()),
                )
            {
                quads.push(Quad::new(
                    activity,
                    predicate,
                    Term::from(agent),
                    graph_name.clone(),
                ));
            }
        }
        if let Some(at_ms) = withdrawal.at_ms
            && let (Ok(predicate), Some(literal)) = (
                NamedNode::new(PROV_INVALIDATED_AT_TIME),
                asserted_at_literal(at_ms),
            )
        {
            quads.push(Quad::new(
                reifier,
                predicate,
                Term::from(literal),
                graph_name,
            ));
        }
    }
}

pub(super) fn push_reifier_literal(
    quads: &mut Vec<Quad>,
    reifier: &NamedNode,
    predicate: &str,
    value: String,
    graph_name: &GraphName,
) {
    if let Ok(predicate) = NamedNode::new(predicate) {
        quads.push(Quad::new(
            reifier.clone(),
            predicate,
            Term::from(Literal::new_simple_literal(value)),
            graph_name.clone(),
        ));
    }
}

pub fn withdrawal_activity_id(statement_id: &str) -> String {
    let mut encoded = String::from(WITHDRAWAL_ACTIVITY_PREFIX);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in statement_id.bytes() {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 15)]));
    }
    encoded
}

pub(super) fn graph_scope_marker(scope: &GraphScope) -> (&'static str, Option<String>) {
    match scope {
        GraphScope::Default => ("default", None),
        GraphScope::Source => ("source", None),
        GraphScope::User => ("user", None),
        GraphScope::Agent => ("agent", None),
        GraphScope::Moot => ("moot", None),
        GraphScope::Custom(name) => ("custom", Some(name.clone())),
    }
}

pub fn property_literal(property: &NodeProperty) -> Literal {
    if let Some(lang) = property.lang.as_deref()
        && let Ok(literal) = Literal::new_language_tagged_literal(property.value.clone(), lang)
    {
        return literal;
    }
    if let Some(datatype) = property.datatype.as_deref()
        && datatype != XSD_STRING
        && let Ok(datatype) = NamedNode::new(datatype)
    {
        return Literal::new_typed_literal(property.value.clone(), datatype);
    }
    Literal::new_simple_literal(property.value.clone())
}

pub fn asserted_at_literal(asserted_at_ms: u64) -> Option<Literal> {
    let timestamp =
        OffsetDateTime::from_unix_timestamp_nanos(i128::from(asserted_at_ms) * 1_000_000).ok()?;
    let lexical = timestamp.format(&Rfc3339).ok()?;
    let datatype = NamedNode::new(XSD_DATETIME).ok()?;
    Some(Literal::new_typed_literal(lexical, datatype))
}
