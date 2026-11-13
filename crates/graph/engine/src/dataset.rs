// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The held dataset (no SPARQL evaluator needed; `query` reads it through `spareval`): `oxrdf` terms interned into a dense-id dictionary
//! and their quads in a `chartulary::quads::QuadIndex` (indexed dataset brief
//! D1 to D5). Built once per revision and read until the kernel's change
//! counters move; who holds it is the consumer's (quad index plan D3).

use crate::{ProjectionKey, RdfSource};
pub use chartulary::quads::View;
use chartulary::quads::{Dictionary, Quad, QuadIndex, QuadMeta, TermId};
use oxrdf::{GraphName, Term};

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
    pub(crate) terms: Dictionary<Term>,
    pub(crate) index: QuadIndex,
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
}
