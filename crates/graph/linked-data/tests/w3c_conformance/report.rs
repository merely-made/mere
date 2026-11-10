// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Pass, fail and skip counts per layer, manifest and category.

use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Counts {
    pub(crate) pass: usize,
    pub(crate) fail: usize,
    pub(crate) skip: usize,
    pub(crate) skip_reasons: BTreeMap<String, usize>,
    pub(crate) fails: Vec<String>,
}

#[derive(Clone)]
pub(crate) enum Outcome {
    Pass,
    Fail(String),
    Skip(&'static str),
}

#[derive(Default)]
pub(crate) struct Report {
    /// `(layer, manifest, category)` -> counts.
    pub(crate) leaf: BTreeMap<(String, String, String), Counts>,
}

impl Report {
    pub(crate) fn record(
        &mut self,
        layer: &str,
        manifest: &str,
        category: &str,
        id: &str,
        out: Outcome,
    ) {
        let c = self
            .leaf
            .entry((layer.into(), manifest.into(), category.into()))
            .or_default();
        match out {
            Outcome::Pass => c.pass += 1,
            Outcome::Fail(why) => {
                c.fail += 1;
                c.fails.push(format!("{id}: {why}"));
            },
            Outcome::Skip(reason) => {
                c.skip += 1;
                *c.skip_reasons.entry(reason.into()).or_default() += 1;
            },
        }
    }

    pub(crate) fn print(&self, suite: &str) {
        let verbose = std::env::var("W3C_VERBOSE").is_ok();
        let mut rows: BTreeMap<(String, String), (usize, usize, usize)> = BTreeMap::new();
        let mut reasons: BTreeMap<(String, String, String), usize> = BTreeMap::new();
        for ((layer, manifest, category), c) in &self.leaf {
            println!(
                "LEAF\t{layer}\t{manifest}\t{category}\t{}\t{}\t{}",
                c.pass, c.fail, c.skip
            );
            if verbose {
                for f in &c.fails {
                    println!("FAIL\t{layer}\t{manifest}\t{f}");
                }
            }
            let r = rows.entry((category.clone(), layer.clone())).or_default();
            r.0 += c.pass;
            r.1 += c.fail;
            r.2 += c.skip;
            for (reason, n) in &c.skip_reasons {
                *reasons
                    .entry((category.clone(), layer.clone(), reason.clone()))
                    .or_default() += n;
            }
        }
        for ((category, layer), (p, f, s)) in &rows {
            println!("ROW\t{suite}\t{category}\t{layer}\t{p}\t{f}\t{s}");
        }
        for ((category, layer, reason), n) in &reasons {
            println!("SKIPREASON\t{suite}\t{category}\t{layer}\t{n}\t{reason}");
        }
    }
}
