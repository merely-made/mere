// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The held query dataset: `oxrdf` terms interned into a dense-id dictionary
//! and their quads in a `chartulary::quads::QuadIndex` (indexed dataset brief
//! D1 to D5). Built once per revision and read until the kernel's change
//! counters move; who holds it is the consumer's (quad index plan D3).

use std::cell::RefCell;
use std::convert::Infallible;

use crate::{ProjectionKey, RdfSource};
use chartulary::quads::{Dictionary, Quad, QuadIndex, QuadMeta, TermId, View};
use oxrdf::{GraphName, Term};
use spareval::{InternalQuad, QueryableDataset};

/// The kernel counters and input keys a dataset was built at; a move in any
/// means rebuild. The keys tell two graphs with equal counters apart.
const WITHDRAWN: QuadMeta = QuadMeta {
    held: false,
    asserter: None,
    statement: None,
};

/// An indexed, dictionary-encoded RDF dataset with held and withdrawn rows.
#[derive(Clone, Debug)]
pub struct IndexedDataset {
    terms: Dictionary<Term>,
    index: QuadIndex,
    /// The kernel counters at build; `None` for a dataset not built from a graph.
    key: Option<ProjectionKey>,
}

impl IndexedDataset {
    /// Index any quads; the flag is whether the row is held (`false` rows are
    /// read only by [`View::History`]).
    pub fn from_quads(quads: impl IntoIterator<Item = (oxrdf::Quad, bool)>) -> Self {
        let mut terms = Dictionary::new();
        let rows: Vec<(Quad, QuadMeta)> = quads
            .into_iter()
            .map(|(quad, held)| {
                let graph = match quad.graph_name {
                    GraphName::DefaultGraph => TermId::DEFAULT_GRAPH,
                    GraphName::NamedNode(name) => terms.intern(name.into()),
                    GraphName::BlankNode(name) => terms.intern(name.into()),
                };
                let row = Quad {
                    subject: terms.intern(quad.subject.into()),
                    predicate: terms.intern(quad.predicate.into()),
                    object: terms.intern(quad.object),
                    graph,
                };
                (row, if held { QuadMeta::HELD } else { WITHDRAWN })
            })
            .collect();
        Self {
            terms,
            index: QuadIndex::build(rows),
            key: None,
        }
    }

    /// Index the graph's projection: the held quads, and the withdrawn
    /// assertions' reifiers in the reserved history graph as withdrawn rows.
    pub fn for_graph<S: RdfSource + ?Sized>(graph: &S) -> Self {
        let held = graph.held_quads().map(|quad| (quad, true));
        let withdrawn = graph
            .withdrawn_reifier_quads()
            .into_iter()
            .map(|quad| (quad, false));
        Self {
            key: Some(graph.projection_key()),
            ..Self::from_quads(held.chain(withdrawn))
        }
    }

    /// Whether `graph` is the graph, at the counters and input keys, this dataset
    /// was built at.
    /// Always `false` for a dataset built by [`IndexedDataset::from_quads`].
    pub fn is_current<S: RdfSource + ?Sized>(&self, graph: &S) -> bool {
        self.key == Some(graph.projection_key())
    }

    /// Stored rows (distinct quad and held flag).
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Whether the dataset holds no rows.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// The number of distinct terms interned.
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// Bytes held by the quad index (the dictionary's terms are not counted).
    pub fn memory_bytes(&self) -> usize {
        self.index.memory_bytes()
    }

    /// One query's read of the dataset through `view`, for `spareval`.
    pub fn read(&self, view: View) -> IndexedRead<'_> {
        IndexedRead {
            dataset: self,
            view,
            transient: RefCell::default(),
        }
    }
}

/// One query's read of an [`IndexedDataset`]: the view, and the terms the
/// query brings that the dataset lacks (constants, computed values).
///
/// Those are minted in a transient dictionary above the dataset's own ids,
/// so they match no row yet externalize back to themselves, and equal terms
/// get equal ids for joins and grouping. The transient terms die with the read.
#[derive(Debug)]
pub struct IndexedRead<'a> {
    dataset: &'a IndexedDataset,
    view: View,
    transient: RefCell<Dictionary<Term>>,
}

impl IndexedRead<'_> {
    fn base(&self) -> u32 {
        u32::try_from(self.dataset.terms.len()).expect("dictionary ids are u32")
    }
}

impl<'a> QueryableDataset<'a> for IndexedRead<'a> {
    type InternalTerm = TermId;
    type Error = Infallible;

    fn internal_quads_for_pattern(
        &self,
        subject: Option<&TermId>,
        predicate: Option<&TermId>,
        object: Option<&TermId>,
        graph_name: Option<Option<&TermId>>,
    ) -> impl Iterator<Item = Result<InternalQuad<TermId>, Infallible>> + use<'a> {
        let dataset: &'a IndexedDataset = self.dataset;
        // spareval's `None` graph is any named graph, never the default one.
        let named_only = graph_name.is_none();
        dataset
            .index
            .pattern(
                subject.copied(),
                predicate.copied(),
                object.copied(),
                graph_name.map(|name| name.copied()),
                self.view,
            )
            .filter(move |(quad, _)| !(named_only && quad.graph.is_default_graph()))
            .map(|(quad, _)| {
                Ok(InternalQuad {
                    subject: quad.subject,
                    predicate: quad.predicate,
                    object: quad.object,
                    graph_name: (!quad.graph.is_default_graph()).then_some(quad.graph),
                })
            })
    }

    fn internal_named_graphs(&self) -> impl Iterator<Item = Result<TermId, Infallible>> + use<'a> {
        let dataset: &'a IndexedDataset = self.dataset;
        dataset.index.graph_names(self.view).map(Ok)
    }

    fn contains_internal_graph_name(&self, graph_name: &TermId) -> Result<bool, Infallible> {
        Ok(self.dataset.index.contains_graph(*graph_name, self.view))
    }

    fn internalize_term(&self, term: Term) -> Result<TermId, Infallible> {
        if let Some(id) = self.dataset.terms.lookup(&term) {
            return Ok(id);
        }
        let local = self.transient.borrow_mut().intern(term);
        let raw = self
            .base()
            .checked_add(local.as_u32())
            .filter(|&raw| raw < u32::MAX)
            .expect("transient term ids stay below the default graph");
        Ok(TermId::from_u32(raw).expect("raw id is at least 1"))
    }

    fn externalize_term(&self, term: TermId) -> Result<Term, Infallible> {
        if let Some(term) = self.dataset.terms.get(term) {
            return Ok(term.clone());
        }
        let local = TermId::from_u32(term.as_u32() - self.base())
            .expect("an id above the dictionary is a transient one");
        Ok(self.transient.borrow().resolve(local).clone())
    }
}
