// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The headless inspector's core over its committed history fixture, which is
//! real `to_trig_with_history` output (engine split P5 L6a). The example is
//! included, not exported, so the engine gains no public API.

#[path = "../examples/inspect.rs"]
mod inspect;

use inspect::{Tally, inspect as run, parse};

const FIXTURE: &str = include_str!("../examples/fixtures/history.trig");
const ADA: &str = "urn:author:ada";
const GRACE: &str = "urn:author:grace";
const NOTES: &str = "https://example.test/graph/notes";
const IMPORTS: &str = "https://example.test/graph/imports";

#[test]
fn the_fixture_reports_asserters_withdrawals_graphs_and_reasons() {
    let ds = parse("trig", FIXTURE, None).unwrap();
    let report = run(&ds, None).unwrap();

    assert_eq!(report.asserters.len(), 2);
    assert_eq!(
        report.asserters[ADA],
        Tally {
            held: 2,
            withdrawn: 0
        }
    );
    assert_eq!(
        report.asserters[GRACE],
        Tally {
            held: 1,
            withdrawn: 1
        }
    );
    assert_eq!(report.assertions.len(), 4);

    assert_eq!(report.graphs.len(), 4);
    assert_eq!(report.graphs[NOTES], 13);
    assert_eq!(report.graphs[IMPORTS], 7);
    assert!(
        report
            .graphs
            .contains_key(mere::ASSERTION_HISTORY_GRAPH_IRI)
    );

    assert_eq!(report.reasons["urn:method:manual-review"], 1);
    assert_eq!(report.reasons["urn:method:extractor"], 2);

    // Alice and bob are subjects, so resources; reifiers, activities and the
    // vocabulary alignment graph are not.
    for who in ["alice", "bob"] {
        assert!(
            report
                .resources
                .contains_key(&format!("https://example.test/{who}"))
        );
    }
    assert_eq!(report.resources.len(), 2);
}

#[cfg(feature = "query")]
#[test]
fn a_query_runs() {
    let ds = parse("trig", FIXTURE, None).unwrap();
    let q = "SELECT ?s ?o WHERE { GRAPH ?g { ?s <https://mere.computer/ns/rel#cites> ?o } }";
    let rows = run(&ds, Some(q)).unwrap().query.unwrap();
    assert_eq!(rows.rows.len(), 3);
}

#[cfg(not(feature = "query"))]
#[test]
fn a_query_without_the_feature_is_refused() {
    let ds = parse("trig", FIXTURE, None).unwrap();
    assert!(run(&ds, Some("ASK {}")).is_err());
}

#[test]
fn a_plain_turtle_file_has_no_asserters() {
    let plain = parse("ttl", "<a> <http://p/> <b> .", Some("http://x/")).unwrap();
    let report = run(&plain, None).unwrap();
    assert!(report.asserters.is_empty());
    assert_eq!(report.resources.len(), 1);
    assert!(parse("xyz", "", None).is_err());
}
