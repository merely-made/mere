// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! RDF-profile metadata for each shared Resource, independent of its surfaces.

use crate::{
    ClassificationScheme, Graph, GraphScope, Literal, NamedNode, Quad, RDF_TYPE, Term,
    property_literal, push_quad, push_statement_metadata_quads,
};

const SKOS_CONCEPT: &str = "http://www.w3.org/2004/02/skos/core#Concept";
const SKOS_PREF_LABEL: &str = "http://www.w3.org/2004/02/skos/core#prefLabel";

pub(crate) fn quads(graph: &Graph, resource: &kernel::graph::ResourceNode) -> Vec<Quad> {
    let mut quads = Vec::new();
    let Ok(subject) = NamedNode::new(resource.canonical_iri()) else {
        return quads;
    };
    if let Some(concept) = graph.resource_tag_concept(resource.id()) {
        push_quad(
            &mut quads,
            &subject,
            RDF_TYPE,
            NamedNode::new_unchecked(SKOS_CONCEPT).into(),
            &GraphScope::Default,
        );
        push_quad(
            &mut quads,
            &subject,
            SKOS_PREF_LABEL,
            Literal::new_simple_literal(concept.label).into(),
            &GraphScope::Default,
        );
        if let Ok(owner) = NamedNode::new(concept.owner_iri) {
            push_quad(
                &mut quads,
                &subject,
                crate::PROV_WAS_ATTRIBUTED_TO,
                owner.into(),
                &GraphScope::Default,
            );
        }
    }
    for variant in graph.resource_classification_variants(resource.id()) {
        let record = variant.classification;
        if record.status.is_affirmative()
            && matches!(&record.scheme, ClassificationScheme::Custom(scheme) if scheme == "rdf:type")
            && let Ok(value) = NamedNode::new(record.value)
        {
            push_quad(
                &mut quads,
                &subject,
                RDF_TYPE,
                value.into(),
                &GraphScope::Default,
            );
        }
    }
    for property in graph.resource_properties(resource.id()) {
        let literal: Term = property_literal(&property).into();
        push_quad(
            &mut quads,
            &subject,
            &property.predicate,
            literal.clone(),
            &property.graph_scope,
        );
        push_statement_metadata_quads(
            &mut quads,
            &subject,
            &property.statement_id,
            &property.predicate,
            literal,
            &property.graph_scope,
            None,
            property.provenance_iri.as_deref(),
            property.asserted_at_ms,
        );
    }
    let mut seen = std::collections::HashSet::new();
    quads.retain(|quad| seen.insert(quad.clone()));
    quads
}

#[cfg(test)]
#[path = "resource_metadata/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "resource_metadata/attribution_tests.rs"]
mod attribution_tests;
