// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! `spareval`'s read of an [`IndexedDataset`] (the `query` feature).

use std::cell::RefCell;
use std::convert::Infallible;

use crate::dataset::IndexedDataset;
use oxrdf::Term;
use quadstore::{Dictionary, TermId, View};
use spareval::{InternalQuad, QueryableDataset};

impl IndexedDataset {
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
