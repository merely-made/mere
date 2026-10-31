// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The sync host's residue verdict on the audit's four shapes (Q052): what it
//! judges, and that every hit it does not judge is still reported.

mod residue_verdict;

use residue_verdict::{OUTER_FRAME, Reason, Run, Unjudged, own_blocks, unjudged};
use std::collections::BTreeSet;

const RUNS: usize = 5;

fn run(live: Vec<usize>) -> Run {
    Run {
        freed: vec![],
        live,
        frame: 4096,
    }
}

fn runs(live: &[usize]) -> Vec<Run> {
    (0..RUNS).map(|_| run(live.to_vec())).collect()
}

/// Positive control: a size in every host run and no baseline is judged.
#[test]
fn persistent_unique_block_is_judged() {
    let host = runs(&[64]);
    assert_eq!(own_blocks(&host, &[]), BTreeSet::from([(true, 64)]));
    assert!(unjudged(&host, &[]).is_empty());
}

#[test]
fn intermittent_block_is_reported_unjudged() {
    let mut host = runs(&[]);
    host[4].live.push(64);
    assert!(own_blocks(&host, &[]).is_empty());
    assert_eq!(
        unjudged(&host, &[]),
        vec![Unjudged {
            live: true,
            size: 64,
            runs: 1,
            reason: Reason::Intermittent,
        }]
    );
}

#[test]
fn block_matching_one_baseline_run_is_reported_unjudged() {
    let host = runs(&[64]);
    let mut baseline = runs(&[]);
    baseline[0].live.push(64);
    assert!(own_blocks(&host, &[&baseline]).is_empty());
    assert_eq!(
        unjudged(&host, &[&baseline]),
        vec![Unjudged {
            live: true,
            size: 64,
            runs: RUNS,
            reason: Reason::BaselineSize {
                host_copies: 1,
                baseline_copies: 1,
            },
        }]
    );
}

#[test]
fn extra_copies_of_a_baseline_size_are_reported_with_counts() {
    let host = runs(&[64, 64]);
    let baseline = runs(&[64]);
    assert!(own_blocks(&host, &[&baseline]).is_empty());
    let found = unjudged(&host, &[&baseline]);
    assert_eq!(
        found,
        vec![Unjudged {
            live: true,
            size: 64,
            runs: RUNS,
            reason: Reason::BaselineSize {
                host_copies: 2,
                baseline_copies: 1,
            },
        }]
    );
    assert!(found[0].to_string().contains("up to 2 host copies against 1"));
}

#[test]
fn outer_frames_compare_by_presence() {
    let host: Vec<Run> = (0..RUNS)
        .map(|n| Run {
            freed: vec![],
            live: vec![7000 + n],
            frame: 7000 + n,
        })
        .collect();
    assert_eq!(own_blocks(&host, &[]), BTreeSet::from([(true, OUTER_FRAME)]));
    let baseline = vec![Run {
        freed: vec![],
        live: vec![6904],
        frame: 6904,
    }];
    assert!(own_blocks(&host, &[&baseline]).is_empty());
    assert_eq!(unjudged(&host, &[&baseline]).len(), 1);
}
