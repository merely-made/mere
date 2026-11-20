// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! SHACL validation for one explicitly selected view of Mere's indexed dataset.

use std::sync::Arc;

use mere::dataset::{IndexedDataset, View};
use oxrdf::{Dataset, Graph, NamedNode, NamedOrBlankNode, Term};
use rudof_iri::IriS;
use rudof_rdf::rdf_core::BuildRDF;
use rudof_rdf::rdf_impl::OxigraphInMemory;
use shacl::ir::IRSchema;
use shacl::rdf::ShaclParser;
use shacl::validator::engine::{Engine, NativeEngine};
use shacl::validator::processor::ShaclProcessor;
use shacl::validator::report::ValidationResult;
use shacl::validator::{ShaclConfig, ShaclValidationMode};

mod read;
use read::IndexedGraph;

/// Which graph in the caller's chosen dataset view is the SHACL data graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphSelector {
    Default,
    Named(NamedOrBlankNode),
    Union,
}

/// A result path, with a typed predicate when it names one direct property.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyPath {
    Predicate(NamedNode),
    /// A complex expression. Its full RDF form is in the report dataset.
    Complex(String),
}

impl PropertyPath {
    pub fn predicate(&self) -> Option<&NamedNode> {
        match self {
            Self::Predicate(predicate) => Some(predicate),
            Self::Complex(_) => None,
        }
    }
}

impl std::fmt::Display for PropertyPath {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Predicate(predicate) => formatter.write_str(predicate.as_str()),
            Self::Complex(expression) => formatter.write_str(expression),
        }
    }
}

/// One result's fields a host can show without depending on Rudof's types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub focus_node: Term,
    pub path: Option<PropertyPath>,
    pub message: Option<String>,
    /// The result severity IRI.
    pub severity: String,
    pub constraint_component: Term,
    pub value: Option<Term>,
    pub source_shape: Option<Term>,
}

/// Both the standard RDF report and its typed, stable-order summary.
#[derive(Debug, Clone)]
pub struct ValidationReport {
    pub dataset: Dataset,
    pub conforms: bool,
    pub findings: Vec<Finding>,
}

#[derive(Debug)]
pub struct ValidationError(String);

impl std::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ValidationError {}

/// Validate against a caller-supplied shapes graph with no network or writes.
/// The graph selector is explicit even for a dataset with only one graph.
///
/// # Errors
///
/// Invalid shapes, evaluation failures, or failure to write the RDF report.
pub fn validate(
    dataset: impl Into<Arc<IndexedDataset>>,
    view: View,
    selector: GraphSelector,
    shapes: &Graph,
) -> Result<ValidationReport, ValidationError> {
    let mut shapes_store = OxigraphInMemory::empty();
    for triple in shapes {
        shapes_store
            .add_triple(
                triple.subject.into_owned(),
                triple.predicate.into_owned(),
                triple.object.into_owned(),
            )
            .map_err(validation_error)?;
    }
    let schema: IRSchema = ShaclParser::new(shapes_store)
        .parse()
        .map_err(validation_error)?
        .try_into()
        .map_err(validation_error)?;
    let config = ShaclConfig::new()
        .with_store_errors(true)
        .with_store_evidences(false);
    let mut validation = IndexedValidation(IndexedGraph::new(dataset.into(), view, selector));
    let report = validation
        .validate(&schema, &ShaclValidationMode::Native, &config)
        .map_err(validation_error)?;

    let mut rdf = OxigraphInMemory::empty();
    report.to_rdf(&mut rdf).map_err(validation_error)?;
    let mut findings: Vec<_> = report.results().iter().map(finding).collect();
    findings.sort_by_cached_key(|finding| {
        (
            finding.focus_node.to_string(),
            finding.path.as_ref().map(ToString::to_string),
            finding.severity.clone(),
            finding.constraint_component.to_string(),
            finding.value.as_ref().map(ToString::to_string),
            finding.source_shape.as_ref().map(ToString::to_string),
            finding.message.clone(),
        )
    });
    Ok(ValidationReport {
        dataset: rdf.quads().collect(),
        conforms: report.conforms(),
        findings,
    })
}

struct IndexedValidation(IndexedGraph);

impl ShaclProcessor<IndexedGraph> for IndexedValidation {
    fn store(&self) -> &IndexedGraph {
        &self.0
    }

    fn runner(_: &ShaclValidationMode, config: &ShaclConfig) -> Box<dyn Engine<IndexedGraph>> {
        Box::new(NativeEngine::new(config.recursion_semantics()))
    }

    fn use_class_index(&self) -> bool {
        false
    }
}

fn validation_error(error: impl std::fmt::Display) -> ValidationError {
    ValidationError(error.to_string())
}

fn finding(result: &ValidationResult) -> Finding {
    let messages = result.message();
    let message = messages
        .get(None)
        .or_else(|| {
            messages
                .iter()
                .min_by_key(|(lang, _)| *lang)
                .map(|(_, text)| text)
        })
        .cloned();
    Finding {
        focus_node: result.focus_node().clone().into(),
        path: result.path().map(|path| match path.pred() {
            Some(predicate) => PropertyPath::Predicate(predicate.named_node().clone()),
            None => PropertyPath::Complex(path.to_string()),
        }),
        message,
        severity: IriS::from(result.severity()).as_str().to_owned(),
        constraint_component: result.constraint_component().clone().into(),
        value: result.value().cloned().map(Term::from),
        source_shape: result.source().cloned().map(Term::from),
    }
}

#[cfg(test)]
mod tests;
