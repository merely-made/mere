// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use std::collections::HashSet;
use std::sync::Arc;

use mere::dataset::{IndexedDataset, View};
use oxrdf::{BlankNode, GraphName, Literal, NamedNode, NamedOrBlankNode, Term, Triple};
use prefixmap::{PrefixMap, PrefixMapError};
use rudof_iri::IriS;
use rudof_rdf::rdf_core::query::{
    QueryRDF, QueryResultFormat, QuerySolution, QuerySolutions, VarName,
};
use rudof_rdf::rdf_core::{Any, Matcher, NeighsRDF, Rdf};
use spareval::{QueryEvaluator, QueryResults};
use spargebra::SparqlParser;

use crate::GraphSelector;

#[derive(Debug)]
pub(crate) struct IndexedGraph {
    dataset: Arc<IndexedDataset>,
    view: View,
    graph: Option<GraphName>,
}

impl IndexedGraph {
    pub(crate) fn new(dataset: Arc<IndexedDataset>, view: View, selector: GraphSelector) -> Self {
        let graph = match selector {
            GraphSelector::Default => Some(GraphName::DefaultGraph),
            GraphSelector::Named(name) => Some(name.into()),
            GraphSelector::Union => None,
        };
        Self {
            dataset,
            view,
            graph,
        }
    }

    fn evaluate(&self, query: &str) -> Result<QueryResults<'_>, String> {
        let query = SparqlParser::new()
            .parse_query(query)
            .map_err(|error| error.to_string())?;
        let evaluator = QueryEvaluator::new();
        let mut prepared = evaluator.prepare(&query);
        let dataset = prepared.dataset_mut();
        match &self.graph {
            Some(graph) => dataset.set_default_graph(vec![graph.clone()]),
            None => dataset.set_default_graph_as_union(),
        }
        // A SHACL data graph cannot expose any other graphs via GRAPH or FROM.
        dataset.set_available_named_graphs(Vec::new());
        prepared
            .execute(self.dataset.read(self.view))
            .map_err(|error| error.to_string())
    }
}

impl Rdf for IndexedGraph {
    type Subject = NamedOrBlankNode;
    type IRI = NamedNode;
    type Term = Term;
    type BNode = BlankNode;
    type Literal = Literal;
    type Triple = Triple;
    type Err = String;

    fn qualify_iri(&self, iri: &NamedNode) -> String {
        iri.to_string()
    }

    fn qualify_subject(&self, subject: &NamedOrBlankNode) -> String {
        subject.to_string()
    }

    fn qualify_term(&self, term: &Term) -> String {
        term.to_string()
    }

    fn prefixmap(&self) -> Option<PrefixMap> {
        None
    }

    fn resolve_prefix_local(&self, prefix: &str, local: &str) -> Result<IriS, PrefixMapError> {
        PrefixMap::new().resolve_prefix_local(prefix, local)
    }
}

impl NeighsRDF for IndexedGraph {
    fn triples(&self) -> Result<impl Iterator<Item = Triple>, String> {
        self.triples_matching(&Any, &Any, &Any)
    }

    fn triples_matching<S, P, O>(
        &self,
        subject: &S,
        predicate: &P,
        object: &O,
    ) -> Result<impl Iterator<Item = Triple> + '_, String>
    where
        S: Matcher<NamedOrBlankNode>,
        P: Matcher<NamedNode>,
        O: Matcher<Term>,
    {
        let union = self.graph.is_none();
        let mut seen = HashSet::new();
        Ok(self
            .dataset
            .quads_matching(
                self.view,
                subject.value(),
                predicate.value(),
                object.value(),
                self.graph.as_ref(),
            )
            .map(|quad| Triple::new(quad.subject, quad.predicate, quad.object))
            .filter(move |triple| !union || seen.insert(triple.clone())))
    }
}

impl QueryRDF for IndexedGraph {
    fn query_select(&self, query: &str) -> Result<QuerySolutions<Self>, String> {
        let QueryResults::Solutions(solutions) = self.evaluate(query)? else {
            return Err("Expected a SPARQL SELECT result".into());
        };
        let names: Vec<VarName> = solutions
            .variables()
            .iter()
            .map(|var| var.as_str().to_owned().into())
            .collect();
        let mut rows = Vec::new();
        for solution in solutions {
            let solution = solution.map_err(|error| error.to_string())?;
            let values = solution.values().to_vec();
            rows.push(QuerySolution::new(names.clone(), values));
        }
        Ok(QuerySolutions::new(rows, PrefixMap::new()))
    }

    fn query_ask(&self, query: &str) -> Result<bool, String> {
        let QueryResults::Boolean(answer) = self.evaluate(query)? else {
            return Err("Expected a SPARQL ASK result".into());
        };
        Ok(answer)
    }

    fn query_construct(&self, _: &str, _: &QueryResultFormat) -> Result<String, String> {
        Err("SHACL validation does not evaluate SPARQL CONSTRUCT".into())
    }
}
