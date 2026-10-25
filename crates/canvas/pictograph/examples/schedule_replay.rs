// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native line-oriented direct Canvas two-stage schedule replay.

#[path = "support/schedule_replay.rs"]
mod schedule_replay;

use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--fixture") {
        println!("{}", schedule_replay::canonical_case()?);
        return Ok(());
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.trim().is_empty() {
        input = schedule_replay::canonical_case()?;
    }
    println!("{}", schedule_replay::run(&input)?);
    Ok(())
}
