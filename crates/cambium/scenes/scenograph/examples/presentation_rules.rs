// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Resolve a study request from stdin, including traces, as deterministic JSON.
use scenograph::presentation::{Context, RuleSet};
use serde::Deserialize;
use std::io::{self, Read};

#[derive(Deserialize)]
struct Request {
    rule_set: RuleSet,
    contexts: Vec<Context>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    let request: Request = serde_json::from_str(&input)?;
    request
        .rule_set
        .validate()
        .map_err(|issues| format!("{issues:?}"))?;
    let results: Result<Vec<_>, _> = request
        .contexts
        .iter()
        .map(|context| request.rule_set.resolve(context))
        .collect();
    println!(
        "{}",
        serde_json::to_string(&results.map_err(|issues| format!("{issues:?}"))?)?
    );
    Ok(())
}
