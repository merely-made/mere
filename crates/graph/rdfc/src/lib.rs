// Copyright 2026 Mark Alan Boykin (changes)
// Copyright Tpt <thomas@pellissier-tanon.fr> (the carried algorithm)
// SPDX-License-Identifier: MIT OR Apache-2.0

//! RDF Dataset Canonicalization, RDFC-1.0 (W3C Recommendation), carried from `oxrdf`.
//!
//! # Provenance
//!
//! The algorithm is `oxrdf` 0.3.5's `rdfc-10` (crates.io, MIT OR Apache-2.0, author Tpt, repository
//! `github.com/oxigraph/oxigraph/tree/main/lib/oxrdf`), copied from its `src/dataset.rs`:
//! `canonicalize_interned_blank_nodes` (lines 602-725), `add_quad_to_blank_node_to_quads_map_*`
//! (727-774), `issue_identifier` (776-796), `hash_first_degree_quads*` (798-903),
//! `hash_related_blank_node` (905-946), `hash_n_degree_quads` (949-1101),
//! `hash_related_blank_node_on_possible_component` (1144-1173), `hash_function` (1175-1192), and the
//! `CanonicalizationState` and `IdentifierIssuer` types (1996-2020). The step numbers in the
//! comments are the spec's, as upstream's are.
//!
//! # Changes from upstream
//!
//! Each is marked `CHANGE n` where it is made.
//!
//! 1. **Every mention is kept** (4.8.3 step 3): upstream's `HashSet` for the related hash to blank
//!    node lists collapsed repeated mentions; this crate uses a `Vec`.
//! 2. **A work budget** on Hash N-Degree Quads calls and on permutations (Security Considerations,
//!    "Dataset Poisoning"); upstream has none.
//! 3. **Lazy permutations**: upstream built all n! vectors before iterating.
//! 4. **Own interner** in place of upstream's `src/interning.rs` (568 lines): dense table ids, no
//!    hashing keys, and a deterministic quad order. No triple-term support in it.
//! 5. **Triple terms are refused** at the entry (RDFC-1.0 defines none); upstream's triple-term
//!    walks are not carried.
//! 6. **Entry points**: canonical N-Quads and the issued-identifier map, the hash as a parameter, and
//!    the call counts reported. Upstream mutated a `Dataset` in place.
//! 7. **`sha2` 0.11** and no `hex` crate (the digest is hex-encoded here).
//!
//! The suite is the metric (the `rdf-canon` tests, run by Mere's conformance harness), with a
//! differential fuzz against a second implementation recorded in `testing/engine-split/rdfc_comparison.md`.

mod algorithm;
mod intern;

use std::collections::BTreeMap;

use oxrdf::Quad;

/// The hash function RDFC-1.0 runs under (4.4: SHA-256 default, SHA-384 where the suite asks).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum HashAlgorithm {
    /// SHA-256, the spec's default.
    #[default]
    Sha256,
    /// SHA-384.
    Sha384,
}

/// Work limits (Security Considerations, "Dataset Poisoning": "providing a configurable limit on
/// the number of iterations of steps performed in the algorithm, particularly recursive steps and
/// permutations of long lists").
///
/// A limit of `None` is unbounded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Options {
    /// The hash function.
    pub hash: HashAlgorithm,
    /// Most calls to Hash N-Degree Quads (4.8) in one canonicalization.
    pub max_hndq_calls: Option<usize>,
    /// Most permutations (4.8.3 step 5.4) considered in one canonicalization.
    pub max_permutations: Option<usize>,
}

/// Default Hash N-Degree Quads call budget. See [`Options::default`].
pub const DEFAULT_MAX_HNDQ_CALLS: usize = 4_000;
/// Default permutation budget.
pub const DEFAULT_MAX_PERMUTATIONS: usize = 100_000;

impl Default for Options {
    fn default() -> Self {
        Self {
            hash: HashAlgorithm::Sha256,
            max_hndq_calls: Some(DEFAULT_MAX_HNDQ_CALLS),
            max_permutations: Some(DEFAULT_MAX_PERMUTATIONS),
        }
    }
}

/// Why a dataset was not canonicalized.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The dataset holds an RDF 1.2 triple term, which RDFC-1.0 does not define.
    TripleTerm,
    /// The Hash N-Degree Quads call budget ran out: a possible poison dataset.
    HndqCallLimitExceeded(usize),
    /// The permutation budget ran out: a possible poison dataset.
    PermutationLimitExceeded(usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TripleTerm => f.write_str("RDFC-1.0 does not define RDF 1.2 triple terms"),
            Self::HndqCallLimitExceeded(n) => {
                write!(
                    f,
                    "more than {n} calls to Hash N-Degree Quads: a possible poison dataset"
                )
            },
            Self::PermutationLimitExceeded(n) => {
                write!(f, "more than {n} permutations: a possible poison dataset")
            },
        }
    }
}

impl std::error::Error for Error {}

/// A canonicalized dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Canonicalized {
    /// Canonical N-Quads (4.5 serialization): sorted by code point, one `\n`-ended line per quad.
    pub nquads: String,
    /// Input blank node label (no `_:`) to its canonical label (`c14n0`, `c14n1`, ...).
    pub issued: BTreeMap<String, String>,
    /// Calls to Hash N-Degree Quads made (the budget's measure).
    pub hndq_calls: usize,
    /// Permutations considered (the budget's second measure).
    pub permutations: usize,
}

/// Canonicalize `quads` (a repeated quad counts once, as in a dataset).
///
/// # Errors
///
/// [`Error::TripleTerm`] for an RDF 1.2 triple term, and the budget errors for a dataset that
/// exceeds [`Options`].
pub fn canonicalize(quads: &[Quad], options: &Options) -> Result<Canonicalized, Error> {
    algorithm::canonicalize(quads, options)
}
