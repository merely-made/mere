// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The held dataset (no SPARQL evaluator needed; `query` reads it through `spareval`): `oxrdf` terms interned into a dense-id dictionary
//! and their quads in a `chartulary::quads::QuadIndex` (indexed dataset brief
//! D1 to D5). Built once per revision and read until the kernel's change
//! counters move, or brought forward by [`IndexedDataset::refresh`] (quad index
//! plan P4); who holds it is the consumer's (quad index plan D3).

use std::collections::HashMap;

use crate::projection::{ProjectionSource, SourceQuads};
use crate::{ProjectionKey, RdfSource};
pub use chartulary::quads::View;
use chartulary::quads::{Dictionary, Quad, QuadIndex, QuadMeta, TermId};
use oxrdf::{GraphName, NamedNode, Term};

mod refresh;
pub use refresh::Refresh;

const WITHDRAWN: QuadMeta = QuadMeta {
    held: false,
    asserter: None,
    statement: None,
};

/// A stored row: the quad's ids and whether it is held.
pub(crate) type Row = (Quad, bool);

/// An indexed, dictionary-encoded RDF dataset with held and withdrawn rows.
#[derive(Clone, Debug)]
pub struct IndexedDataset {
    pub(crate) terms: Dictionary<Term>,
    pub(crate) index: QuadIndex,
    /// The kernel counters and input keys at build or last refresh; `None`
    /// for a dataset not built from a graph.
    key: Option<ProjectionKey>,
    /// What each source contributed, when the graph's projection splits.
    parts: Option<Parts>,
}

/// The per-source record a refresh diffs against.
#[derive(Clone, Debug, Default)]
struct Parts {
    sources: HashMap<ProjectionSource, Contribution>,
    /// Rows more than one source emits, with how many; any other stored row
    /// has exactly one.
    shared: HashMap<Row, u32>,
    /// The resource graph's version, which moves on any resource change.
    resources_version: u64,
}

/// One source's rows, sorted and distinct, and the id of the IRI it is named by.
#[derive(Clone, Debug)]
struct Contribution {
    subject: Option<TermId>,
    rows: Box<[Row]>,
}

fn intern(terms: &mut Dictionary<Term>, quad: oxrdf::Quad) -> Quad {
    let graph = match quad.graph_name {
        GraphName::DefaultGraph => TermId::DEFAULT_GRAPH,
        GraphName::NamedNode(name) => terms.intern(name.into()),
        GraphName::BlankNode(name) => terms.intern(name.into()),
    };
    Quad {
        subject: terms.intern(quad.subject.into()),
        predicate: terms.intern(quad.predicate.into()),
        object: terms.intern(quad.object),
        graph,
    }
}

fn meta(held: bool) -> QuadMeta {
    if held { QuadMeta::HELD } else { WITHDRAWN }
}

fn subject_term(subject: &str) -> Option<Term> {
    NamedNode::new(subject).ok().map(Term::from)
}

impl Contribution {
    fn new(terms: &mut Dictionary<Term>, quads: SourceQuads) -> Self {
        let subject = quads
            .subject
            .as_deref()
            .and_then(subject_term)
            .map(|term| terms.intern(term));
        let mut rows: Vec<Row> = Vec::with_capacity(quads.held.len() + quads.withdrawn.len());
        for quad in quads.held {
            rows.push((intern(terms, quad), true));
        }
        for quad in quads.withdrawn {
            rows.push((intern(terms, quad), false));
        }
        rows.sort_unstable();
        rows.dedup();
        Self {
            subject,
            rows: rows.into_boxed_slice(),
        }
    }
}

impl IndexedDataset {
    /// Index any quads; the flag is whether the row is held (`false` rows are
    /// read only by [`View::History`]).
    pub fn from_quads(quads: impl IntoIterator<Item = (oxrdf::Quad, bool)>) -> Self {
        let mut terms = Dictionary::new();
        let rows: Vec<(Quad, QuadMeta)> = quads
            .into_iter()
            .map(|(quad, held)| (intern(&mut terms, quad), meta(held)))
            .collect();
        Self {
            terms,
            index: QuadIndex::build(rows),
            key: None,
            parts: None,
        }
    }

    /// Index the graph's projection: the held quads, and the withdrawn
    /// assertions' reifiers in the reserved history graph as withdrawn rows.
    /// A graph whose projection splits by source is indexed source by
    /// source, so [`refresh`](Self::refresh) can re-project one alone.
    ///
    /// # Panics
    ///
    /// If more than `u32::MAX` sources emit one row.
    pub fn for_graph<S: RdfSource + ?Sized>(graph: &S) -> Self {
        let key = Some(graph.projection_key());
        let Some(sources) = graph.sources() else {
            let held = graph.held_quads().map(|quad| (quad, true));
            let withdrawn = graph
                .withdrawn_reifier_quads()
                .into_iter()
                .map(|quad| (quad, false));
            return Self {
                key,
                ..Self::from_quads(held.chain(withdrawn))
            };
        };
        let mut terms = Dictionary::new();
        let mut parts = Parts {
            sources: HashMap::with_capacity(sources.len()),
            shared: HashMap::new(),
            resources_version: graph.resource_graph().version(),
        };
        let mut all: Vec<Row> = Vec::new();
        for source in sources {
            if let Some(quads) = graph.source_quads(source) {
                let contribution = Contribution::new(&mut terms, quads);
                all.extend_from_slice(&contribution.rows);
                parts.sources.insert(source, contribution);
            }
        }
        all.sort_unstable();
        for run in all.chunk_by(|a, b| a == b).filter(|run| run.len() > 1) {
            let count = u32::try_from(run.len()).expect("sources fit u32");
            parts.shared.insert(run[0], count);
        }
        Self::from_parts(terms, all, key, parts)
    }

    fn from_parts(
        terms: Dictionary<Term>,
        rows: Vec<Row>,
        key: Option<ProjectionKey>,
        parts: Parts,
    ) -> Self {
        let index = QuadIndex::build(rows.into_iter().map(|(quad, held)| (quad, meta(held))));
        Self {
            terms,
            index,
            key,
            parts: Some(parts),
        }
    }

    /// Whether `graph` is the graph, at the counters and input keys, this
    /// dataset was built or last refreshed at.
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

    /// The number of distinct terms interned. A refresh only adds terms, so
    /// this can count terms no row uses any more until the next rebuild.
    pub fn term_count(&self) -> usize {
        self.terms.len()
    }

    /// Bytes held by the quad index (the dictionary's terms are not counted).
    pub fn memory_bytes(&self) -> usize {
        self.index.memory_bytes()
    }
}
