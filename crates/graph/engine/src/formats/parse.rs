// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Format entry points that return a dataset (engine split P5 L4, F9).
//!
//! Each function parses one syntax with a base IRI and returns an
//! [`IndexedDataset`], built through [`IndexedDataset::from_quads`], so the
//! ledger's engine layer and the query path share one dataset type. The text
//! is read as written: nothing is skolemized, nothing is dropped, and no graph
//! is touched. Applying a dataset to a graph stays with the importers
//! ([`from_quads`](crate::from_quads) and the kernel's `kernel::rdf`).
//!
//! `base_iri` resolves relative IRIs. `None` leaves them unresolved, which the
//! syntaxes then reject. N-Triples and N-Quads have no relative IRIs, so they
//! take the argument for a uniform signature and ignore it. RDF/XML is parse
//! only (standards stance D142): the engine does not serialize it.
//!
//! Gated on the `query` feature because [`IndexedDataset`] is.

use oxrdf::{GraphName, Quad};
use oxttl::{NQuadsParser, NTriplesParser, TriGParser, TurtleParser};

use crate::acquire::ingest::{ContextCache, IngestError, offline_loader};
use crate::query::IndexedDataset;

fn parse_error(error: impl std::fmt::Display) -> IngestError {
    IngestError::Parse(error.to_string())
}

fn dataset(quads: Vec<Quad>) -> IndexedDataset {
    IndexedDataset::from_quads(quads.into_iter().map(|quad| (quad, true)))
}

fn in_default_graph(triples: impl IntoIterator<Item = oxrdf::Triple>) -> Vec<Quad> {
    triples
        .into_iter()
        .map(|triple| triple.in_graph(GraphName::DefaultGraph))
        .collect()
}

/// Parse **N-Triples** (RDF 1.2 included) into a dataset of default-graph quads.
/// `base_iri` is ignored: the syntax has no relative IRIs.
///
/// # Errors
///
/// [`IngestError::Parse`] with the parser's message.
pub fn parse_ntriples(text: &str, _base_iri: Option<&str>) -> Result<IndexedDataset, IngestError> {
    let triples = NTriplesParser::new()
        .for_slice(text.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(parse_error)?;
    Ok(dataset(in_default_graph(triples)))
}

/// Parse **N-Quads** into a dataset. `base_iri` is ignored, as for N-Triples.
///
/// # Errors
///
/// [`IngestError::Parse`] with the parser's message.
pub fn parse_nquads(text: &str, _base_iri: Option<&str>) -> Result<IndexedDataset, IngestError> {
    let quads = NQuadsParser::new()
        .for_slice(text.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(parse_error)?;
    Ok(dataset(quads))
}

/// Parse **Turtle** into a dataset of default-graph quads, resolving relative
/// IRIs against `base_iri`.
///
/// # Errors
///
/// [`IngestError::Parse`] for an invalid base IRI or a syntax error.
pub fn parse_turtle(text: &str, base_iri: Option<&str>) -> Result<IndexedDataset, IngestError> {
    let mut parser = TurtleParser::new();
    if let Some(base) = base_iri {
        parser = parser.with_base_iri(base).map_err(parse_error)?;
    }
    let triples = parser
        .for_slice(text.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(parse_error)?;
    Ok(dataset(in_default_graph(triples)))
}

/// Parse **TriG** into a dataset, resolving relative IRIs against `base_iri`.
/// Unlike [`from_trig`](crate::from_trig) this keeps the vocabulary-alignment
/// graph if the text carries one.
///
/// # Errors
///
/// [`IngestError::Parse`] for an invalid base IRI or a syntax error.
pub fn parse_trig(text: &str, base_iri: Option<&str>) -> Result<IndexedDataset, IngestError> {
    let mut parser = TriGParser::new();
    if let Some(base) = base_iri {
        parser = parser.with_base_iri(base).map_err(parse_error)?;
    }
    let quads = parser
        .for_slice(text.as_bytes())
        .collect::<Result<Vec<_>, _>>()
        .map_err(parse_error)?;
    Ok(dataset(quads))
}

/// Parse **RDF/XML** into a dataset of default-graph quads, resolving relative
/// IRIs against `base_iri`. Parse only.
///
/// # Errors
///
/// [`IngestError::Parse`] for an invalid base IRI or a syntax error.
pub fn parse_rdfxml(bytes: &[u8], base_iri: Option<&str>) -> Result<IndexedDataset, IngestError> {
    let mut parser = oxrdfxml::RdfXmlParser::new();
    if let Some(base) = base_iri {
        parser = parser.with_base_iri(base).map_err(parse_error)?;
    }
    let triples = parser
        .for_slice(bytes)
        .collect::<Result<Vec<_>, _>>()
        .map_err(parse_error)?;
    Ok(dataset(in_default_graph(triples)))
}

/// Parse **JSON-LD** 1.1 into a dataset (the `toRdf` algorithm), resolving
/// relative IRIs against `base_iri`. A remote `@context` is served from
/// `contexts` and never fetched; a URL absent from the cache is an error. The
/// base is an expansion input, not a source-identity claim, as for
/// [`from_jsonld_with_contexts_and_base_iri`](crate::from_jsonld_with_contexts_and_base_iri).
///
/// # Errors
///
/// [`IngestError::Parse`] for an invalid base IRI, an unresolvable context or
/// a syntax error.
pub fn parse_jsonld(
    bytes: &[u8],
    contexts: ContextCache,
    base_iri: Option<&str>,
) -> Result<IndexedDataset, IngestError> {
    let mut parser = oxjsonld::JsonLdParser::new();
    if let Some(base) = base_iri {
        parser = parser
            .with_base_iri(base)
            .map_err(|error| IngestError::Parse(format!("invalid JSON-LD base IRI: {error}")))?;
    }
    let quads = parser
        .for_slice(bytes)
        .with_load_document_callback(offline_loader(contexts))
        .collect::<Result<Vec<_>, _>>()
        .map_err(parse_error)?;
    Ok(dataset(quads))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query::View;

    fn rows(dataset: &IndexedDataset) -> usize {
        dataset.len()
    }

    #[test]
    fn base_iri_resolves_relative_iris_and_absence_rejects_them() {
        let text = "<a> <http://p/> <b> .
";
        assert_eq!(rows(&parse_turtle(text, Some("http://x/")).unwrap()), 1);
        assert!(parse_turtle(text, None).is_err());
        let trig = "<g> { <a> <http://p/> <b> . }
";
        assert_eq!(rows(&parse_trig(trig, Some("http://x/")).unwrap()), 1);
        assert!(parse_trig(trig, None).is_err());
        assert!(parse_turtle(text, Some("not an iri")).is_err());
    }

    #[test]
    fn line_formats_take_and_ignore_a_base() {
        let nt = "<http://a/> <http://p/> \"x\" .
";
        assert_eq!(rows(&parse_ntriples(nt, Some("http://x/")).unwrap()), 1);
        let nq = "<http://a/> <http://p/> \"x\" <http://g/> .
";
        assert_eq!(rows(&parse_nquads(nq, None).unwrap()), 1);
        assert!(parse_nquads("<a> <p>", None).is_err());
    }

    #[test]
    fn rdfxml_resolves_against_the_base() {
        let xml = br#"<rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
            <rdf:Description rdf:about="a"><rdf:type rdf:resource="b"/></rdf:Description></rdf:RDF>"#;
        assert_eq!(rows(&parse_rdfxml(xml, Some("http://x/")).unwrap()), 1);
        assert!(parse_rdfxml(b"<not-xml", None).is_err());
    }

    #[test]
    fn jsonld_serves_contexts_from_the_cache_only() {
        let doc = br#"{"@context":"http://c.example/ctx","@id":"a","name":"n"}"#;
        let cache = ContextCache::new().with(
            "http://c.example/ctx",
            br#"{"@context":{"name":"http://schema.org/name"}}"#.to_vec(),
        );
        let ds = parse_jsonld(doc, cache, Some("http://x/")).unwrap();
        assert_eq!(rows(&ds), 1);
        let err = parse_jsonld(doc, ContextCache::new(), Some("http://x/")).unwrap_err();
        assert!(err.to_string().contains("refused remote @context"));
    }

    #[test]
    fn rows_are_held_and_the_default_graph_reads_live() {
        let ds = parse_turtle("<http://a/> <http://p/> <http://b/> .", None).unwrap();
        let rows = crate::query::sparql_on(
            &ds,
            "SELECT ?o WHERE { <http://a/> <http://p/> ?o }",
            View::Live,
            crate::model::CoverageNote::default(),
        )
        .unwrap();
        assert_eq!(rows.rows.len(), 1);
    }
}
