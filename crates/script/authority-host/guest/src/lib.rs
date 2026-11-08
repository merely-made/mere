// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `authority-guest` test fixture. The candidate is read as text: a candidate holding
//! `Forbidden` is refused, `SPIN` never returns (the host's fuel cut-off shows it traps),
//! `PANIC` panics, `GROW` asks for far more memory than any budget, and `LOG` logs. A
//! before/after check refuses a transition whose after holds `Forbidden`.

wit_bindgen::generate!({
    path: "../../wit",
    world: "authority-guest",
});

use exports::mere::script::domain_authority::{Claim, Guest};

struct Fixture;

fn has(bytes: &[u8], needle: &str) -> bool {
    let needle = needle.as_bytes();
    bytes.windows(needle.len()).any(|window| window == needle)
}

impl Guest for Fixture {
    fn describe() -> Claim {
        Claim {
            domain: "fixture".into(),
            version: "v1".into(),
            graph_format: "mere.graph-snapshot-and-facets/json".into(),
            admission: true,
        }
    }

    fn validate(candidate: Vec<u8>) -> Result<(), String> {
        if has(&candidate, "LOG") {
            mere::script::log::log("fixture saw LOG");
        }
        if has(&candidate, "SPIN") {
            let mut n = 0u64;
            loop {
                n = std::hint::black_box(n.wrapping_add(1));
            }
        }
        if has(&candidate, "PANIC") {
            panic!("fixture panics on request");
        }
        if has(&candidate, "GROW") {
            let big = vec![1u8; 512 * 1024 * 1024];
            return Err(format!("grew {}", std::hint::black_box(big).len()));
        }
        if has(&candidate, "Forbidden") {
            return Err("fixture forbids the complete candidate".into());
        }
        Ok(())
    }

    fn admit(_before: Vec<u8>, after: Vec<u8>) -> Result<(), String> {
        if has(&after, "Forbidden") {
            Err("fixture forbids the transition".into())
        } else {
            Ok(())
        }
    }
}

export!(Fixture);
