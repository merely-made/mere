// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
//! Native line-oriented entry point for the shared occurrence replay case.

#[path = "support/occurrence_replay.rs"]
mod occurrence_replay;

use pictograph::canvas::dynamics_recipe::bind_slot;
use pictograph::canvas::projection_dynamics::ProjectionDynamics;
use std::io::{self, Read};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if std::env::args().nth(1).as_deref() == Some("--fixture") {
        println!("{}", occurrence_replay::canonical_case()?);
        return Ok(());
    }
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    if input.trim().is_empty() {
        input = occurrence_replay::canonical_case()?;
    }
    println!("{}", occurrence_replay::run(&input)?);
    Ok(())
}
