// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use std::collections::{HashMap, HashSet};

use mere::formats::canon::canonicalize_quads;
use oxrdf::vocab::{rdf, xsd};
use oxrdf::{BlankNode, Graph, GraphName, NamedNode, NamedOrBlankNode, Quad, Term};

use super::{Result, iri, objects, one};

const SH: &str = "http://www.w3.org/ns/shacl#";

pub(super) fn conforms(graph: &Graph, report: &NamedOrBlankNode) -> Result<bool> {
    match one(graph, report, &iri(SH, "conforms"))? {
        Term::Literal(value) if value.datatype() == xsd::BOOLEAN => match value.value() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err("invalid xsd:boolean in report".into()),
        },
        _ => Err("sh:conforms was not an xsd:boolean".into()),
    }
}

pub(super) fn compare(
    expected: &Graph,
    expected_root: &NamedOrBlankNode,
    actual: &Graph,
) -> Result<Option<String>> {
    let roots: Vec<_> = actual
        .subjects_for_predicate_object(rdf::TYPE, iri(SH, "ValidationReport").as_ref())
        .map(|n| n.into_owned())
        .collect();
    let [actual_root] = roots.as_slice() else {
        return Ok(Some(format!(
            "expected one validation report, got {}",
            roots.len()
        )));
    };
    let result_predicate = iri(SH, "result");
    let message_predicate = iri(SH, "resultMessage");
    let mut messages = HashSet::new();
    for result in objects(expected, expected_root, &result_predicate) {
        messages.extend(objects(
            expected,
            &NamedOrBlankNode::try_from(result)?,
            &message_predicate,
        ));
    }
    let expected = project(expected, expected_root, &messages)?;
    let actual = project(actual, actual_root, &messages)?;
    let expected = canonicalize_quads(&expected, &Default::default())?.nquads;
    let actual = canonicalize_quads(&actual, &Default::default())?.nquads;
    Ok((expected != actual).then(|| format!("expected:\n{expected}actual:\n{actual}")))
}

/// Project the W3C comparison surface, retaining all result nodes including duplicates.
/// Report and result identifiers are normalized to blank nodes. Each result path
/// gets an independent copy of its blank-node structure.
fn project(
    graph: &Graph,
    report: &NamedOrBlankNode,
    messages: &HashSet<Term>,
) -> Result<Vec<Quad>> {
    let normalized_report = BlankNode::default();
    let sh_result = iri(SH, "result");
    let sh_conforms = iri(SH, "conforms");
    let sh_report_type = Term::from(iri(SH, "ValidationReport"));
    let sh_result_type = Term::from(iri(SH, "ValidationResult"));
    let sh_message = iri(SH, "resultMessage");
    let sh_path = iri(SH, "resultPath");
    let result_fields = [
        "focusNode",
        "resultPath",
        "resultSeverity",
        "sourceConstraint",
        "sourceConstraintComponent",
        "sourceShape",
        "value",
        "resultMessage",
    ]
    .map(|term| iri(SH, term));
    let mut quads = Vec::new();
    for triple in graph.triples_for_subject(report.as_ref()) {
        if triple.predicate == sh_conforms
            || (triple.predicate == rdf::TYPE && triple.object == sh_report_type.as_ref())
        {
            quads.push(Quad::new(
                normalized_report.clone(),
                triple.predicate,
                triple.object,
                GraphName::DefaultGraph,
            ));
        }
    }
    for node in objects(graph, report, &sh_result) {
        let node = NamedOrBlankNode::try_from(node)?;
        let normalized_result = BlankNode::default();
        quads.push(Quad::new(
            normalized_report.clone(),
            sh_result.clone(),
            normalized_result.clone(),
            GraphName::DefaultGraph,
        ));
        for triple in graph.triples_for_subject(node.as_ref()) {
            if triple.predicate == rdf::TYPE {
                if triple.object != sh_result_type.as_ref() {
                    continue;
                }
            } else if !result_fields.iter().any(|field| triple.predicate == *field)
                || (triple.predicate == sh_message
                    && !messages.contains(&triple.object.into_owned()))
            {
                continue;
            }
            let mut value = triple.object.into_owned();
            if triple.predicate == sh_path {
                value = copy_path(graph, value, &mut quads);
            }
            quads.push(Quad::new(
                normalized_result.clone(),
                triple.predicate,
                value,
                GraphName::DefaultGraph,
            ));
        }
    }
    Ok(quads)
}

fn copy_path(graph: &Graph, root: Term, quads: &mut Vec<Quad>) -> Term {
    let Term::BlankNode(root) = root else {
        return root;
    };
    let fields: [NamedNode; 7] = [
        rdf::FIRST.into_owned(),
        rdf::REST.into_owned(),
        iri(SH, "inversePath"),
        iri(SH, "alternativePath"),
        iri(SH, "zeroOrMorePath"),
        iri(SH, "oneOrMorePath"),
        iri(SH, "zeroOrOnePath"),
    ];
    let mut copies = HashMap::from([(root.clone(), BlankNode::default())]);
    let mut pending = vec![root.clone()];
    while let Some(node) = pending.pop() {
        let new_subject = copies[&node].clone();
        for triple in graph.triples_for_subject(node.as_ref()) {
            if !fields.iter().any(|field| triple.predicate == *field) {
                continue;
            }
            let value = match triple.object.into_owned() {
                Term::BlankNode(child) => copies
                    .entry(child.clone())
                    .or_insert_with(|| {
                        pending.push(child);
                        BlankNode::default()
                    })
                    .clone()
                    .into(),
                other => other,
            };
            quads.push(Quad::new(
                new_subject.clone(),
                triple.predicate,
                value,
                GraphName::DefaultGraph,
            ));
        }
    }
    copies.remove(&root).unwrap().into()
}
