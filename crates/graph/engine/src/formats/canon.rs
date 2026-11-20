// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! RDF Dataset Canonicalization, RDFC-1.0 (engine split P6 L2, D143 F-R7).
//!
//! The algorithm is `mere-rdfc`, carried from `oxrdf`'s `rdfc-10` with the spec's poison-dataset
//! budget, every mention kept and lazy permutations (see that crate's header and
//! `testing/engine-split/rdfc_comparison.md`). This module takes a dataset or a quad slice.
//!
//! The work limit is a count of Hash N-Degree Quads calls and permutations, not of blank nodes: a
//! blank-node count cannot tell a poison clique of ten from a legitimate graph of nineteen.
//!
//! RDFC-1.0 defines no RDF 1.2 triple terms, so a dataset carrying one is refused.
//!
//! [`sorted_nquads`] is the other canonical form here (engine-split D190): deduplicated, sorted
//! RDF 1.2 N-Quads that keep triple terms and refuse blank nodes, the text the Resource digest
//! hashes.
//!
//! Available without the `query` feature and on wasm32.

use oxrdf::{GraphName, NamedOrBlankNode, Quad, Term};

use crate::dataset::{IndexedDataset, View};

pub use mere_rdfc::{
    Canonicalized as Canonical, DEFAULT_MAX_HNDQ_CALLS, DEFAULT_MAX_PERMUTATIONS,
    Error as CanonError, HashAlgorithm as CanonHash, Options as CanonOptions,
};

/// Canonicalize the rows of `dataset` visible in `view`.
///
/// # Errors
///
/// [`CanonError`] for a triple term or an exhausted work budget.
pub fn canonicalize(
    dataset: &IndexedDataset,
    view: View,
    options: &CanonOptions,
) -> Result<Canonical, CanonError> {
    let quads: Vec<Quad> = dataset.quads(view).collect();
    canonicalize_quads(&quads, options)
}

/// Canonicalize a slice of quads (a repeated quad counts once, as in a dataset).
///
/// # Errors
///
/// [`CanonError`] for a triple term or an exhausted work budget.
pub fn canonicalize_quads(quads: &[Quad], options: &CanonOptions) -> Result<Canonical, CanonError> {
    mere_rdfc::canonicalize(quads, options)
}

/// Why quads have no [`sorted_nquads`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortedNQuadsError {
    /// A quad holds a blank node, anywhere including inside a triple term; sorting alone cannot
    /// label one canonically. `quad` is the first seen, as N-Quads.
    BlankNode { quad: String },
}

impl std::fmt::Display for SortedNQuadsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BlankNode { quad } => write!(f, "a blank node has no sorted N-Quads: {quad}"),
        }
    }
}

impl std::error::Error for SortedNQuadsError {}

/// The deduplicated, sorted RDF 1.2 N-Quads of `quads`, triple terms included (engine-split
/// D190): one `\n`-ended line per distinct quad, in code point order, each term written by the
/// same `oxrdf` `Display` RDFC-1.0's serialization uses. A blank-free set without triple terms
/// gives the same bytes as [`canonicalize_quads`]'s `nquads`.
///
/// # Errors
///
/// [`SortedNQuadsError::BlankNode`] if any quad holds a blank node.
pub fn sorted_nquads(quads: impl IntoIterator<Item = Quad>) -> Result<String, SortedNQuadsError> {
    let mut lines = Vec::new();
    for quad in quads {
        if quad_has_blank(&quad) {
            return Err(SortedNQuadsError::BlankNode {
                quad: format!("{quad} ."),
            });
        }
        lines.push(format!("{quad} .\n"));
    }
    lines.sort_unstable();
    lines.dedup();
    Ok(lines.concat())
}

fn quad_has_blank(quad: &Quad) -> bool {
    matches!(quad.subject, NamedOrBlankNode::BlankNode(_))
        || term_has_blank(&quad.object)
        || matches!(quad.graph_name, GraphName::BlankNode(_))
}

fn term_has_blank(term: &Term) -> bool {
    match term {
        Term::BlankNode(_) => true,
        Term::Triple(triple) => {
            matches!(triple.subject, NamedOrBlankNode::BlankNode(_))
                || term_has_blank(&triple.object)
        },
        Term::NamedNode(_) | Term::Literal(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use oxrdf::{GraphName, NamedNode, Term, Triple};

    use super::*;

    /// A dataset read through `quads(View::Live)` canonicalizes, and the issued map is returned.
    #[test]
    fn a_dataset_is_canonicalized_with_its_issued_map() {
        let text = "_:e0 <http://example.org/p> <http://example.org/o> .\n";
        let dataset = crate::parse_nquads(text, None).expect("parses");
        let out = canonicalize(&dataset, View::Live, &CanonOptions::default()).expect("canonical");
        assert_eq!(
            out.nquads,
            "_:c14n0 <http://example.org/p> <http://example.org/o> .\n"
        );
        assert_eq!(out.issued.get("e0").map(String::as_str), Some("c14n0"));
    }

    /// SHA-384 changes the hash, so a tied pair is labelled by a different order or the same
    /// canonical text; either way it runs and issues both labels.
    #[test]
    fn sha384_runs() {
        let text = "_:a <urn:p> _:b .\n_:b <urn:p> _:a .\n";
        let dataset = crate::parse_nquads(text, None).expect("parses");
        let options = CanonOptions {
            hash: CanonHash::Sha384,
            ..CanonOptions::default()
        };
        let out = canonicalize(&dataset, View::Live, &options).expect("canonical");
        assert_eq!(out.issued.len(), 2);
    }

    /// RDFC-1.0 defines no triple terms: the boundary refuses them.
    #[test]
    fn a_triple_term_is_refused() {
        let n = NamedNode::new_unchecked("urn:x");
        let quad = Quad::new(
            n.clone(),
            n.clone(),
            Term::from(Triple::new(n.clone(), n.clone(), n)),
            GraphName::DefaultGraph,
        );
        assert_eq!(
            canonicalize_quads(&[quad], &CanonOptions::default()),
            Err(CanonError::TripleTerm)
        );
    }

    fn iri(s: &str) -> NamedNode {
        NamedNode::new_unchecked(s)
    }

    /// A blank-free dataset without triple terms: both graph kinds, escapes, language tags,
    /// datatypes, non-ASCII, prefix-sharing IRIs and repeats.
    fn plain_quads() -> Vec<Quad> {
        use oxrdf::Literal;
        let g = GraphName::NamedNode(iri("urn:g"));
        let mut quads = vec![
            Quad::new(
                iri("urn:b"),
                iri("urn:p"),
                iri("urn:a"),
                GraphName::DefaultGraph,
            ),
            Quad::new(iri("urn:a/b"), iri("urn:p"), iri("urn:é"), g.clone()),
            Quad::new(
                iri("urn:a"),
                iri("urn:p"),
                Literal::new_simple_literal("tab\tnl\nq\"bs\\ c\u{1}"),
                g.clone(),
            ),
            Quad::new(
                iri("urn:a"),
                iri("urn:p"),
                Literal::new_language_tagged_literal_unchecked("chat", "fr"),
                GraphName::DefaultGraph,
            ),
            Quad::new(
                iri("urn:a"),
                iri("urn:p"),
                Literal::new_typed_literal("7", iri("http://www.w3.org/2001/XMLSchema#integer")),
                g.clone(),
            ),
            Quad::new(
                iri("urn:a"),
                iri("urn:p"),
                Literal::new_typed_literal("s", iri("http://www.w3.org/2001/XMLSchema#string")),
                g,
            ),
            Quad::new(
                iri("urn:Z"),
                iri("urn:p"),
                Literal::new_simple_literal("日本"),
                GraphName::DefaultGraph,
            ),
        ];
        quads.push(quads[0].clone());
        quads.push(quads[3].clone());
        quads
    }

    /// D190: for a blank-free set without triple terms, sorted N-Quads is RDFC-1.0's output.
    #[test]
    fn sorted_nquads_equal_rdfc_on_a_blank_free_set() {
        let quads = plain_quads();
        let rdfc = canonicalize_quads(&quads, &CanonOptions::default())
            .unwrap()
            .nquads;
        let sorted = sorted_nquads(quads.clone()).unwrap();
        assert_eq!(sorted.lines().count(), 7, "the two repeats count once");
        assert_eq!(sorted, rdfc);
        let mut reversed = quads;
        reversed.reverse();
        assert_eq!(
            sorted_nquads(reversed).unwrap(),
            rdfc,
            "input order does not matter"
        );
    }

    /// Triple terms are kept, in their RDF 1.2 form, and the text does not depend on input order.
    #[test]
    fn sorted_nquads_keep_triple_terms() {
        let (s, p, o) = (iri("urn:s"), iri("urn:p"), iri("urn:o"));
        let reifies = iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies");
        let triple = Term::from(Triple::new(s.clone(), p.clone(), o.clone()));
        let mut quads = plain_quads();
        quads.push(Quad::new(
            iri("urn:r"),
            reifies.clone(),
            triple.clone(),
            GraphName::DefaultGraph,
        ));
        quads.push(Quad::new(s, p, o, GraphName::DefaultGraph));
        let text = sorted_nquads(quads.clone()).unwrap();
        assert!(text.contains(
            "<urn:r> <http://www.w3.org/1999/02/22-rdf-syntax-ns#reifies> <<( <urn:s> <urn:p> <urn:o> )>> .\n"
        ));
        let mut reversed = quads.clone();
        reversed.reverse();
        assert_eq!(sorted_nquads(reversed).unwrap(), text);
        let without = sorted_nquads(
            quads
                .into_iter()
                .filter(|q| !matches!(q.object, Term::Triple(_))),
        );
        assert_ne!(without.unwrap(), text, "the triple-term row is in the text");
    }

    /// D190: a blank node anywhere is refused, inside a triple term too.
    #[test]
    fn sorted_nquads_refuse_blank_nodes() {
        use oxrdf::BlankNode;
        let b = BlankNode::new_unchecked("b0");
        let (s, p) = (iri("urn:s"), iri("urn:p"));
        let nested = Triple::new(
            s.clone(),
            p.clone(),
            Term::from(Triple::new(b.clone(), p.clone(), s.clone())),
        );
        let cases = [
            Quad::new(b.clone(), p.clone(), s.clone(), GraphName::DefaultGraph),
            Quad::new(s.clone(), p.clone(), b.clone(), GraphName::DefaultGraph),
            Quad::new(
                s.clone(),
                p.clone(),
                s.clone(),
                GraphName::BlankNode(b.clone()),
            ),
            Quad::new(
                s.clone(),
                p.clone(),
                Term::from(Triple::new(b.clone(), p.clone(), s.clone())),
                GraphName::DefaultGraph,
            ),
            Quad::new(
                s.clone(),
                p.clone(),
                Term::from(Triple::new(s.clone(), p.clone(), b)),
                GraphName::DefaultGraph,
            ),
            Quad::new(
                s.clone(),
                p.clone(),
                Term::from(nested),
                GraphName::DefaultGraph,
            ),
        ];
        for case in cases {
            let mut quads = plain_quads();
            quads.push(case.clone());
            assert_eq!(
                sorted_nquads(quads),
                Err(SortedNQuadsError::BlankNode {
                    quad: format!("{case} .")
                }),
                "{case}"
            );
        }
        assert!(
            sorted_nquads(plain_quads()).is_ok(),
            "control: the blank-free rest passes"
        );
    }

    /// The suite's poison shape, a clique of ten blank nodes, is refused by the work budget.
    #[test]
    fn a_blank_node_clique_is_refused_by_the_budget() {
        let mut text = String::new();
        for i in 0..10 {
            for j in 0..10 {
                text.push_str(&format!("_:e{i} <http://example.com/p> _:e{j} .\n"));
            }
        }
        let dataset = crate::parse_nquads(&text, None).expect("parses");
        let options = CanonOptions {
            max_hndq_calls: Some(500),
            ..CanonOptions::default()
        };
        assert!(matches!(
            canonicalize(&dataset, View::Live, &options),
            Err(CanonError::HndqCallLimitExceeded(500) | CanonError::PermutationLimitExceeded(_))
        ));
    }
}
