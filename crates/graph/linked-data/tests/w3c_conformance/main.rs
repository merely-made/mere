// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! W3C conformance harness for the `linked-data` adapter (engine split plan, P0).
//!
//! Reads the vendored suites' manifests and runs two layers per test:
//!
//! - `parser`: the parsers the adapter wraps (`oxttl`, `oxjsonld`, `spargebra`), built with the
//!   adapter's own feature set and options.
//! - `adapter`: the crate's public entry points (`from_nquads`, `from_trig`,
//!   `from_jsonld_with_contexts_and_base_iri`), where one exists for the format.
//!
//! Ignored by default and a clean skip when the suites are absent:
//!
//! ```text
//! W3C_SUITES_DIR=<dir holding rdf-tests, json-ld-api, ...> \
//!   cargo test -p mere-rdf --features query --test w3c_conformance -- --ignored --nocapture
//! ```
//!
//! Failures do not fail the test; they are the ledger's record. `W3C_VERBOSE=1` lists each
//! failing test. The only assertions are the positive and negative controls, which prove the
//! instrument before any count is believed.

mod controls;
mod jsonld;
mod manifest;
mod rdf;
mod report;
#[cfg(feature = "query")]
mod sparql;

use std::path::{Path, PathBuf};

use manifest::walk;
use rdf::{Fmt, adapter_accepts, isomorphic, parse_rdf};
use report::Report;

fn suites_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("W3C_SUITES_DIR") {
        let dir = PathBuf::from(dir);
        return dir.join("rdf-tests").is_dir().then_some(dir);
    }
    // `repos/mere/crates/graph/linked-data` -> `Code/crates`.
    let guess = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../../crates");
    guess.join("rdf-tests").is_dir().then_some(guess)
}

// ---- tests ------------------------------------------------------------------------------

#[test]
fn instrument_controls() {
    // Negative control: a bad N-Triples line is rejected. Positive control: a good one parses.
    assert!(parse_rdf(Fmt::NTriples, b"<a> <b>", "").is_err());
    let good = parse_rdf(Fmt::NTriples, b"<http://a/> <http://b/> \"c\" .\n", "").unwrap();
    assert_eq!(good.len(), 1);
    // The isomorphism check tells blank-node renaming from a real difference.
    let a = parse_rdf(Fmt::NTriples, b"_:x <http://p/> _:y .\n", "").unwrap();
    let b = parse_rdf(Fmt::NTriples, b"_:q <http://p/> _:r .\n", "").unwrap();
    let c = parse_rdf(Fmt::NTriples, b"_:q <http://p/> _:q .\n", "").unwrap();
    assert!(isomorphic(a.clone(), b));
    assert!(!isomorphic(a, c));
    // The adapter's own entry points reject and accept as expected.
    assert!(adapter_accepts(Fmt::NQuads, b"<a> <b>").unwrap().is_err());
    assert!(
        adapter_accepts(Fmt::NQuads, b"<http://a/> <http://b/> <http://c/> .\n")
            .unwrap()
            .is_ok()
    );
}

#[test]
#[ignore = "needs the vendored W3C suites; see the module doc"]
fn w3c_conformance() {
    let Some(dir) = suites_dir() else {
        println!("W3C suites absent (set W3C_SUITES_DIR): skipping");
        return;
    };
    println!("SUITES\t{}", dir.display());

    let rdf_tests = dir.join("rdf-tests");
    let mut rdf_suites = vec![
        ("rdf12", "rdf/rdf12/manifest.ttl"),
        ("rdf11", "rdf/rdf11/manifest.ttl"),
    ];
    if cfg!(feature = "query") {
        rdf_suites.extend([
            (
                "sparql11-query",
                "sparql/sparql11/manifest-sparql11-query.ttl",
            ),
            (
                "sparql11-update",
                "sparql/sparql11/manifest-sparql11-update.ttl",
            ),
            ("sparql12", "sparql/sparql12/manifest.ttl"),
        ]);
    }
    let mut total_pass = 0;
    for (suite, manifest) in rdf_suites {
        let mut entries = Vec::new();
        walk(&rdf_tests.join(manifest), &mut entries).unwrap_or_else(|e| panic!("{suite}: {e}"));
        println!("SUITE\t{suite}\t{} entries", entries.len());
        let mut report = Report::default();
        if suite.starts_with("sparql") {
            #[cfg(feature = "query")]
            sparql::run_sparql(&rdf_tests, &entries, &mut report);
        } else {
            rdf::run_rdf(&rdf_tests, &entries, &mut report);
        }
        total_pass += report
            .leaf
            .iter()
            .filter(|((l, _, _), _)| l == "parser")
            .map(|(_, c)| c.pass)
            .sum::<usize>();
        report.print(suite);
    }

    let jsonld = dir.join("json-ld-api");
    if jsonld.is_dir() {
        let mut report = Report::default();
        jsonld::run_jsonld(&jsonld, &mut report).unwrap_or_else(|e| panic!("json-ld: {e}"));
        total_pass += report
            .leaf
            .iter()
            .filter(|((l, _, _), _)| l == "parser")
            .map(|(_, c)| c.pass)
            .sum::<usize>();
        report.print("jsonld11");
    }
    // The instrument found and passed tests: a zero here is a broken walker, not a verdict.
    assert!(
        total_pass > 0,
        "no test passed: the harness is not reading the suites"
    );
}
