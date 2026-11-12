// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Turtle-family file I/O for the RDF projection (petgraph-RDF plan, Phase 2's
//! "Turtle / N-Quads I/O via `oxttl`").
//!
//! [`dataset_quads`](crate::dataset_quads) is an in-memory projection; this
//! module makes it a file a standard tool can read, and reads one back. Both
//! serializations carry named graphs (the scope graphs plus the vocabulary
//! graph), so **N-Quads** and **TriG** are the faithful full-dataset formats;
//! plain Turtle/N-Triples would flatten the scopes and is not offered. N-Quads
//! is the canonical line-based interchange the plan names; TriG is its
//! Turtle-family, graph-grouped, human-readable sibling.
//!
//! Every serialization is self-describing: it appends
//! [`vocabulary_alignment_quads`](crate::vocabulary_alignment_quads) so a
//! consumer can bridge Mere's `rel#` IRIs onto standard vocabularies from the
//! file alone. On the way back in, that alignment graph is dropped (it is
//! re-derivable schema, not instance content), so `graph -> file -> contribution
//! -> graph -> file` is byte-stable under the same normalized-dataset compare as
//! the in-memory round-trip gate. RDF 1.2 triple terms (the reifier metadata)
//! ride through both formats via oxttl's `rdf-12` feature.

use oxttl::{NQuadsParser, NQuadsSerializer, TriGParser, TriGSerializer};

use crate::acquire::ingest::{GraphContribution, ImportEnvelope, IngestError, from_quads_envelope};
use crate::profile::vocab::is_vocabulary_quad;
use crate::{RdfSource, dataset_quads, dataset_quads_with_history, vocabulary_alignment_quads};

/// The whole graph as **N-Quads**: instance quads (scoped) plus the vocabulary
/// alignment graph. Deterministic (the projection is sorted), so it is safe to
/// pin in a golden test.
pub fn to_nquads<S: RdfSource + ?Sized>(graph: &S) -> String {
    to_nquads_from_quads(dataset_quads(graph))
}

/// The whole graph as N-Quads with withdrawn assertion reifiers in the
/// explicit history graph. Withdrawn base triples are never asserted.
pub fn to_nquads_with_history<S: RdfSource + ?Sized>(graph: &S) -> String {
    to_nquads_from_quads(dataset_quads_with_history(graph))
}

fn to_nquads_from_quads(quads: Vec<oxrdf::Quad>) -> String {
    let mut serializer = NQuadsSerializer::new().for_writer(Vec::<u8>::new());
    for quad in quads.iter().chain(vocabulary_alignment_quads().iter()) {
        serializer
            .serialize_quad(quad)
            .expect("writing N-Quads to an in-memory Vec is infallible");
    }
    String::from_utf8(serializer.finish()).expect("oxttl emits valid UTF-8")
}

/// The whole graph as **TriG** (Turtle with named graphs): the same content as
/// [`to_nquads`], grouped by graph and abbreviated. `Err` only on the rare
/// serializer I/O fault (the writer is an in-memory `Vec`, so effectively never).
pub fn to_trig<S: RdfSource + ?Sized>(graph: &S) -> Result<String, String> {
    to_trig_from_quads(dataset_quads(graph))
}

/// TriG serialization with withdrawn assertion reifiers in the explicit
/// history graph. Withdrawn base triples are never asserted.
pub fn to_trig_with_history<S: RdfSource + ?Sized>(graph: &S) -> Result<String, String> {
    to_trig_from_quads(dataset_quads_with_history(graph))
}

fn to_trig_from_quads(quads: Vec<oxrdf::Quad>) -> Result<String, String> {
    let mut serializer = TriGSerializer::new().for_writer(Vec::<u8>::new());
    for quad in quads.iter().chain(vocabulary_alignment_quads().iter()) {
        serializer.serialize_quad(quad).map_err(|e| e.to_string())?;
    }
    let bytes = serializer.finish().map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|e| e.to_string())
}

/// Parse **N-Quads** into a graph contribution, dropping the re-derivable
/// vocabulary-alignment graph. `namespace` scopes any skolemized blank nodes, as
/// in [`from_quads`].
pub fn from_nquads(text: &str, namespace: &str) -> Result<GraphContribution, IngestError> {
    from_nquads_envelope(text, namespace).map(ImportEnvelope::into_contribution)
}

/// Parse N-Quads with the evidence required by the faithful profile importer.
pub fn from_nquads_envelope(text: &str, namespace: &str) -> Result<ImportEnvelope, IngestError> {
    let mut quads = Vec::new();
    for quad in NQuadsParser::new().for_slice(text.as_bytes()) {
        let quad = quad.map_err(|e| IngestError::Parse(e.to_string()))?;
        if !is_vocabulary_quad(&quad) {
            quads.push(quad);
        }
    }
    from_quads_envelope(quads, namespace)
}

/// Parse **TriG** into a graph contribution, dropping the re-derivable
/// vocabulary-alignment graph.
pub fn from_trig(text: &str, namespace: &str) -> Result<GraphContribution, IngestError> {
    from_trig_envelope(text, namespace).map(ImportEnvelope::into_contribution)
}

/// Parse TriG with the evidence required by the faithful profile importer.
pub fn from_trig_envelope(text: &str, namespace: &str) -> Result<ImportEnvelope, IngestError> {
    let mut quads = Vec::new();
    for quad in TriGParser::new().for_slice(text.as_bytes()) {
        let quad = quad.map_err(|e| IngestError::Parse(e.to_string()))?;
        if !is_vocabulary_quad(&quad) {
            quads.push(quad);
        }
    }
    from_quads_envelope(quads, namespace)
}
