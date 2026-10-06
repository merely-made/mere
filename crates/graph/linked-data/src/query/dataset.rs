// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use kernel::graph::Graph;
use oxrdf::{GraphName, Term};
use spareval::{InternalQuad, QueryableDataset};
use std::collections::HashSet;
use std::convert::Infallible;

/// Borrows kernel authority and projects matching RDF quads as they are read.
/// Each pattern scans the projection; only matched quads need set deduplication.
pub(super) struct GraphDataset<'a> {
    graph: &'a Graph,
}

impl<'a> GraphDataset<'a> {
    pub(super) fn new(graph: &'a Graph) -> Self {
        Self { graph }
    }
}

impl<'a> QueryableDataset<'a> for GraphDataset<'a> {
    type InternalTerm = Term;
    type Error = Infallible;

    fn internal_quads_for_pattern(
        &self,
        subject: Option<&Term>,
        predicate: Option<&Term>,
        object: Option<&Term>,
        graph_name: Option<Option<&Term>>,
    ) -> impl Iterator<Item = Result<InternalQuad<Term>, Infallible>> + use<'a> {
        let subject = subject.cloned();
        let predicate = predicate.cloned();
        let object = object.cloned();
        let graph_name = graph_name.map(|name| name.cloned());
        let mut seen = HashSet::new();
        crate::dataset_quad_iter(self.graph).filter_map(move |quad| {
            let quad_subject: Term = quad.subject.clone().into();
            let quad_predicate: Term = quad.predicate.clone().into();
            let quad_graph = match &quad.graph_name {
                GraphName::DefaultGraph => None,
                GraphName::NamedNode(name) => Some(Term::NamedNode(name.clone())),
                GraphName::BlankNode(name) => Some(Term::BlankNode(name.clone())),
            };
            let graph_matches = match &graph_name {
                None => quad_graph.is_some(),
                Some(None) => quad_graph.is_none(),
                Some(Some(name)) => quad_graph.as_ref() == Some(name),
            };
            if !graph_matches
                || subject.as_ref().is_some_and(|term| term != &quad_subject)
                || predicate
                    .as_ref()
                    .is_some_and(|term| term != &quad_predicate)
                || object.as_ref().is_some_and(|term| term != &quad.object)
                || !seen.insert(quad.clone())
            {
                return None;
            }
            Some(Ok(InternalQuad {
                subject: quad_subject,
                predicate: quad_predicate,
                object: quad.object,
                graph_name: quad_graph,
            }))
        })
    }

    fn internalize_term(&self, term: Term) -> Result<Term, Infallible> {
        Ok(term)
    }

    fn externalize_term(&self, term: Term) -> Result<Term, Infallible> {
        Ok(term)
    }
}
