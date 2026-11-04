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
//! A materialized spareval dataset remains the test parity oracle.

#[cfg(test)]
#[path = "query/coverage_tests.rs"]
mod coverage_tests;

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

/// Run SPARQL over an explicit retained-history projection. The reserved
/// `mere:graph#history` named graph contains withdrawn reifiers only; a query
/// must name that graph to inspect tombstones. Default [`sparql`] stays held-only.
pub fn sparql_with_history(graph: &Graph, query: &str) -> Result<QueryRows, String> {
    let dataset: oxrdf::Dataset = crate::dataset_quads_with_history(graph)
        .into_iter()
        .collect();
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

#[cfg(test)]
#[path = "query/tests.rs"]
mod tests;

/// A SELECT roster of held Resource identities, with limits of this evaluation.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceQueryMembers {
    pub members: Vec<uuid::Uuid>,
    /// Requested Resource IRIs not held here; retained for coverage refresh.
    pub missing_resources: Vec<uuid::Uuid>,
    pub coverage: kernel::graph::CoverageNote,
}

/// Evaluate the named member column as Resource references.
pub fn sparql_resource_members(
    graph: &Graph,
    query: &str,
    member_variable: &str,
) -> Result<ResourceQueryMembers, String> {
    let query = SparqlParser::new()
        .parse_query(query)
        .map_err(|e| e.to_string())?;
    let evaluator = QueryEvaluator::new();
    let results = evaluator
        .prepare(&query)
        .execute(GraphDataset::new(graph))
        .map_err(|e| e.to_string())?;
    let QueryResults::Solutions(solutions) = results else {
        return Err("saved-query membership requires SELECT".into());
    };
    if !solutions
        .variables()
        .iter()
        .any(|v| v.as_str() == member_variable)
    {
        return Err(format!("SELECT has no member variable ?{member_variable}"));
    }
    let identities: std::collections::HashMap<_, _> = graph
        .resource_nodes()
        .map(|r| (r.canonical_iri(), r.id()))
        .collect();
    let mut members = std::collections::BTreeSet::new();
    let mut missing = std::collections::BTreeSet::new();
    for solution in solutions {
        let solution = solution.map_err(|e| e.to_string())?;
        let Some(term) = solution.get(member_variable) else {
            continue;
        };
        let Term::NamedNode(iri) = term else {
            return Err(format!(
                "?{member_variable} must contain Resource IRIs, not literals or blank nodes"
            ));
        };
        if let Some(id) = identities.get(iri.as_str()) {
            members.insert(*id);
        } else {
            missing.insert(kernel::graph::ResourceNode::for_term(iri.as_str()).id());
        }
    }
    let missing_resources: Vec<_> = missing.into_iter().collect();
    Ok(ResourceQueryMembers {
        members: members.into_iter().collect(),
        coverage: graph.coverage_for_resource_refs(&missing_resources),
        missing_resources,
    })
}

#[cfg(test)]
mod resource_member_tests;

#[cfg(test)]
#[path = "query/history_projection_tests.rs"]
mod history_projection_tests;
