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
//! Available without the `query` feature and on wasm32.

use oxrdf::Quad;

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
