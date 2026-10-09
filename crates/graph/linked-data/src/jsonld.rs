// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! JSON-LD's classic reification representation of the RDF projection.

use std::collections::{BTreeMap, HashMap, HashSet};

use kernel::graph::{Graph, sub_kind_from_iri};
use oxrdf::{GraphName, NamedNode, NamedOrBlankNode, Quad, Term, Triple};
use serde_json::{Map, Value, json};

use crate::{RDF_REIFIES, RDF_TYPE, SCHEMA_KEYWORDS, SCHEMA_NAME};

pub(crate) const RDF_STATEMENT: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#Statement";
pub(crate) const RDF_SUBJECT: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#subject";
pub(crate) const RDF_PREDICATE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#predicate";
pub(crate) const RDF_OBJECT: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#object";

pub(crate) fn is_statement_metadata(predicate: &str) -> bool {
    matches!(
        predicate,
        crate::RDFS_LABEL | crate::PROV_WAS_ATTRIBUTED_TO | crate::PROV_GENERATED_AT_TIME
    )
}

pub(crate) fn representable_metadata(predicate: &str, object: &Term) -> bool {
    match (predicate, object) {
        (crate::RDFS_LABEL, Term::Literal(literal)) => {
            literal.language().is_none() && literal.datatype().as_str() == crate::XSD_STRING
        },
        (crate::PROV_WAS_ATTRIBUTED_TO, Term::NamedNode(_)) => true,
        (crate::PROV_GENERATED_AT_TIME, Term::Literal(literal)) => {
            literal.datatype().as_str() == crate::XSD_DATETIME
                && crate::ingest::parse_xsd_datetime_ms(literal.value()).is_some()
        },
        _ => false,
    }
}

fn resource_id(node: &NamedOrBlankNode) -> String {
    match node {
        NamedOrBlankNode::NamedNode(node) => node.as_str().to_string(),
        NamedOrBlankNode::BlankNode(node) => format!("_:{}", node.as_str()),
    }
}

fn term_value(term: &Term, compact: bool) -> Option<Value> {
    match term {
        Term::NamedNode(node) => Some(json!({ "@id": node.as_str() })),
        Term::BlankNode(node) => Some(json!({ "@id": format!("_:{}", node.as_str()) })),
        Term::Literal(literal) => Some(if compact {
            crate::compact_literal_json_value(literal)
        } else {
            crate::literal_json_value(literal)
        }),
        Term::Triple(_) => None,
    }
}

fn classic_quads(quads: impl Iterator<Item = Quad>) -> Vec<Quad> {
    let mut result = Vec::new();
    for quad in quads {
        if quad.predicate.as_str() == RDF_REIFIES
            && let Term::Triple(triple) = &quad.object
        {
            for (predicate, object) in [
                (RDF_TYPE, NamedNode::new_unchecked(RDF_STATEMENT).into()),
                (RDF_SUBJECT, triple.subject.clone().into()),
                (RDF_PREDICATE, triple.predicate.clone().into()),
                (RDF_OBJECT, triple.object.clone()),
            ] {
                result.push(Quad::new(
                    quad.subject.clone(),
                    NamedNode::new_unchecked(predicate),
                    object,
                    quad.graph_name.clone(),
                ));
            }
        } else {
            result.push(quad);
        }
    }
    let mut seen = HashSet::new();
    result.retain(|quad| seen.insert(quad.clone()));
    result.sort_by_cached_key(ToString::to_string);
    result
}

type Subjects = BTreeMap<String, BTreeMap<String, Vec<Term>>>;

fn subject_object(
    id: String,
    predicates: BTreeMap<String, Vec<Term>>,
    compact: bool,
    context: &mut Map<String, Value>,
) -> Value {
    let mut object = Map::new();
    object.insert("@id".into(), Value::String(id));
    for (predicate, terms) in predicates {
        if predicate == RDF_TYPE && terms.iter().all(|term| matches!(term, Term::NamedNode(_))) {
            object.insert(
                "@type".into(),
                Value::Array(
                    terms
                        .iter()
                        .map(|term| match term {
                            Term::NamedNode(node) => Value::String(node.as_str().into()),
                            _ => unreachable!("named types checked"),
                        })
                        .collect(),
                ),
            );
            continue;
        }
        let key = if compact
            && (predicate == SCHEMA_NAME
                || predicate == SCHEMA_KEYWORDS
                || sub_kind_from_iri(&predicate).is_some())
        {
            let term = predicate.rsplit(['#', '/']).next().unwrap_or(&predicate);
            context.insert(term.into(), Value::String(predicate.clone()));
            term.to_string()
        } else {
            predicate.clone()
        };
        let mut values: Vec<_> = terms
            .iter()
            .filter_map(|term| term_value(term, compact))
            .collect();
        let value = if compact && predicate != SCHEMA_KEYWORDS && values.len() == 1 {
            values.pop().expect("one value checked")
        } else {
            Value::Array(values)
        };
        object.insert(key, value);
    }
    Value::Object(object)
}

pub(crate) fn export(graph: &Graph, compact: bool) -> Value {
    let mut graphs: BTreeMap<Option<String>, Subjects> = BTreeMap::new();
    // Preserve the legacy presentation of otherwise undescribed surfaces.
    let default = graphs.entry(None).or_default();
    for (_, node) in graph.nodes() {
        default.entry(crate::node_id(node)).or_default();
    }
    for quad in classic_quads(crate::dataset_quad_iter(graph)) {
        let graph_id = match &quad.graph_name {
            GraphName::DefaultGraph => None,
            GraphName::NamedNode(node) => Some(node.as_str().to_string()),
            GraphName::BlankNode(node) => Some(format!("_:{}", node.as_str())),
        };
        graphs
            .entry(graph_id)
            .or_default()
            .entry(resource_id(&quad.subject))
            .or_default()
            .entry(quad.predicate.as_str().into())
            .or_default()
            .push(quad.object);
    }
    let mut context = Map::new();
    let mut objects = Vec::new();
    for (graph_id, subjects) in graphs {
        let values: Vec<_> = subjects
            .into_iter()
            .map(|(id, predicates)| subject_object(id, predicates, compact, &mut context))
            .collect();
        if let Some(id) = graph_id {
            objects.push(json!({ "@id": id, "@graph": values }));
        } else {
            objects.extend(values);
        }
    }
    if compact {
        json!({ "@context": context, "@graph": objects })
    } else {
        Value::Array(objects)
    }
}

/// Lift only a complete, single-valued classic record whose base is asserted.
/// Other descriptions keep all their ordinary RDF quads.
pub(crate) fn bridge_classic_reification(quads: Vec<Quad>) -> Vec<Quad> {
    let present: HashSet<_> = quads.iter().cloned().collect();
    let mut groups: HashMap<(NamedOrBlankNode, GraphName), Vec<&Quad>> = HashMap::new();
    let mut record_graphs: HashMap<NamedOrBlankNode, HashSet<GraphName>> = HashMap::new();
    let mut triple_descriptions: HashMap<NamedOrBlankNode, Vec<&Quad>> = HashMap::new();
    for quad in &quads {
        groups
            .entry((quad.subject.clone(), quad.graph_name.clone()))
            .or_default()
            .push(quad);
        if matches!(
            quad.predicate.as_str(),
            RDF_SUBJECT | RDF_PREDICATE | RDF_OBJECT
        ) || (quad.predicate.as_str() == RDF_TYPE
            && quad.object == Term::from(NamedNode::new_unchecked(RDF_STATEMENT)))
        {
            record_graphs
                .entry(quad.subject.clone())
                .or_default()
                .insert(quad.graph_name.clone());
        }
        if quad.predicate.as_str() == RDF_REIFIES {
            triple_descriptions
                .entry(quad.subject.clone())
                .or_default()
                .push(quad);
        }
    }
    let mut candidates = HashMap::new();
    for ((subject, graph), rows) in &groups {
        if !rows.iter().any(|quad| {
            quad.predicate.as_str() == RDF_TYPE
                && quad.object == Term::from(NamedNode::new_unchecked(RDF_STATEMENT))
        }) {
            continue;
        }
        let single = |predicate: &str| -> Option<Term> {
            let values: HashSet<_> = rows
                .iter()
                .filter(|quad| quad.predicate.as_str() == predicate)
                .map(|quad| quad.object.clone())
                .collect();
            (values.len() == 1).then(|| values.into_iter().next().expect("one value"))
        };
        let (Some(base_subject), Some(Term::NamedNode(predicate)), Some(object)) = (
            single(RDF_SUBJECT),
            single(RDF_PREDICATE),
            single(RDF_OBJECT),
        ) else {
            continue;
        };
        let base_subject = match base_subject {
            Term::NamedNode(node) => NamedOrBlankNode::from(node),
            Term::BlankNode(node) => NamedOrBlankNode::from(node),
            _ => continue,
        };
        if matches!(object, Term::Triple(_)) {
            continue;
        }
        let base = Quad::new(
            base_subject.clone(),
            predicate.clone(),
            object.clone(),
            graph.clone(),
        );
        if !present.contains(&base) {
            continue;
        }
        if rows.iter().any(|quad| {
            is_statement_metadata(quad.predicate.as_str())
                && !representable_metadata(quad.predicate.as_str(), &quad.object)
        }) {
            continue;
        }
        if record_graphs
            .get(subject)
            .is_some_and(|graphs| graphs.len() > 1)
        {
            continue;
        }
        // These fields map to single-valued assertion metadata in Mere.
        if [
            crate::RDFS_LABEL,
            crate::PROV_WAS_ATTRIBUTED_TO,
            crate::PROV_GENERATED_AT_TIME,
        ]
        .iter()
        .any(|predicate| {
            rows.iter()
                .filter(|quad| quad.predicate.as_str() == *predicate)
                .map(|quad| &quad.object)
                .collect::<HashSet<_>>()
                .len()
                > 1
        }) {
            continue;
        }
        let triple = Triple::new(base_subject, predicate, object);
        // A conflicting triple-term description must not pick a survivor.
        if triple_descriptions.get(subject).is_some_and(|rows| {
            rows.iter().any(|quad| {
                quad.graph_name != *graph || quad.object != Term::Triple(Box::new(triple.clone()))
            })
        }) {
            continue;
        }
        candidates.insert((subject.clone(), graph.clone()), triple);
    }
    // One carried reifier identity cannot describe multiple assertions/scopes.
    let mut counts: HashMap<NamedOrBlankNode, usize> = HashMap::new();
    for (subject, _) in candidates.keys() {
        *counts.entry(subject.clone()).or_default() += 1;
    }
    candidates.retain(|(subject, _), _| counts.get(subject) == Some(&1));
    let mut result = Vec::new();
    let mut seen = HashSet::new();
    for quad in quads {
        let key = (quad.subject.clone(), quad.graph_name.clone());
        if let Some(triple) = candidates.get(&key)
            && (matches!(
                quad.predicate.as_str(),
                RDF_SUBJECT | RDF_PREDICATE | RDF_OBJECT
            ) || (quad.predicate.as_str() == RDF_TYPE
                && quad.object == Term::from(NamedNode::new_unchecked(RDF_STATEMENT))))
        {
            let lifted = Quad::new(
                quad.subject,
                NamedNode::new_unchecked(RDF_REIFIES),
                Term::Triple(Box::new(triple.clone())),
                quad.graph_name,
            );
            if seen.insert(lifted.clone()) {
                result.push(lifted);
            }
        } else if seen.insert(quad.clone()) {
            result.push(quad);
        }
    }
    result
}

#[cfg(test)]
mod tests;
