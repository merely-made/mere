// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The `authority-guest` world end to end over a real component built in Mere: the claim,
//! verdicts as values, the budget cutting a runaway call off, a trap leaving the next call
//! unharmed, and the closed WASI.

use std::path::PathBuf;

use authority_host::{AuthorityComponent, Budget, LoadError, Verdict};

fn guest_bytes() -> Vec<u8> {
    let path = std::env::var_os("AUTHORITY_HOST_GUEST_WASM")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("guest/target/wasm32-wasip2/release/authority_fixture_guest.wasm"));
    std::fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "guest component missing at {} ({error}). Build it with:\n  \
             cd crates/script/authority-host/guest && cargo build --locked --target wasm32-wasip2 --release",
            path.display()
        )
    })
}

fn component(budget: Budget) -> AuthorityComponent {
    let engine = AuthorityComponent::engine().unwrap();
    AuthorityComponent::load(&engine, &guest_bytes(), budget).unwrap()
}

#[test]
fn describe_claims_one_domain_its_format_and_admission() {
    let guest = component(Budget::default());
    let claim = guest.claim();
    assert_eq!(claim.domain, "fixture");
    assert_eq!(claim.version, "v1");
    assert_eq!(claim.graph_format, authority_host::GRAPH_SNAPSHOT_AND_FACETS);
    assert!(claim.admission);
}

#[test]
fn a_candidate_is_admitted_or_refused_with_the_guests_own_text() {
    let guest = component(Budget::default());
    assert_eq!(guest.validate(b"{\"nodes\":[]}"), Verdict::Admitted);
    assert_eq!(
        guest.validate(b"has Forbidden inside"),
        Verdict::Refused("fixture forbids the complete candidate".into())
    );
    assert_eq!(guest.admit(b"before", b"after"), Verdict::Admitted);
    assert_eq!(
        guest.admit(b"before", b"after Forbidden"),
        Verdict::Refused("fixture forbids the transition".into())
    );
    assert_eq!(guest.validate(b"x").into_result(), Ok(()));
}

#[test]
fn the_guest_can_log_and_the_host_keeps_a_bounded_copy() {
    let guest = component(Budget::default());
    let (verdict, logs) = guest.validate_logged(b"LOG");
    assert_eq!(verdict, Verdict::Admitted);
    assert_eq!(logs, ["fixture saw LOG"]);
}

#[test]
fn a_runaway_call_is_cut_off_by_fuel_and_the_next_call_runs_in_a_fresh_instance() {
    let guest = component(Budget {
        fuel: 50_000_000,
        ..Budget::default()
    });
    let Verdict::Trapped(message) = guest.validate(b"SPIN") else {
        panic!("a spinning guest must trap");
    };
    assert!(message.contains("fuel"), "{message}");
    assert!(guest.validate(b"SPIN").into_result().unwrap_err().starts_with("authority guest trapped"));
    // The same component is entered again as if nothing happened.
    assert_eq!(guest.validate(b"fine"), Verdict::Admitted);
    assert_eq!(
        guest.validate(b"Forbidden"),
        Verdict::Refused("fixture forbids the complete candidate".into())
    );
}

#[test]
fn a_panic_is_a_trap_and_the_next_call_is_unharmed() {
    let guest = component(Budget::default());
    assert!(matches!(guest.validate(b"PANIC"), Verdict::Trapped(_)));
    assert_eq!(guest.validate(b"fine"), Verdict::Admitted);
}

#[test]
fn the_memory_cap_refuses_a_guest_that_asks_for_more() {
    let guest = component(Budget::default());
    let verdict = guest.validate(b"GROW");
    assert!(matches!(verdict, Verdict::Trapped(_) | Verdict::Refused(_)), "{verdict:?}");
    assert!(
        !matches!(&verdict, Verdict::Refused(text) if text.starts_with("grew")),
        "the 512 MiB allocation must not succeed under the 128 MiB cap"
    );
    assert_eq!(guest.validate(b"fine"), Verdict::Admitted);
}

#[test]
fn an_oversize_candidate_is_refused_before_the_guest_runs() {
    let guest = component(Budget {
        candidate_bytes: 8,
        ..Budget::default()
    });
    assert!(matches!(guest.validate(&[b'a'; 9]), Verdict::Trapped(_)));
    assert_eq!(guest.validate(&[b'a'; 8]), Verdict::Admitted);
    assert!(matches!(guest.admit(&[b'a'; 5], &[b'a'; 5]), Verdict::Trapped(_)));
}

#[test]
fn a_component_that_is_not_an_authority_guest_is_refused_at_load() {
    let engine = AuthorityComponent::engine().unwrap();
    assert!(matches!(
        AuthorityComponent::load(&engine, b"not wasm", Budget::default()),
        Err(LoadError::Invalid(_))
    ));
    let too_big = vec![0u8; authority_host::MAX_COMPONENT_BYTES + 1];
    assert!(matches!(
        AuthorityComponent::load(&engine, &too_big, Budget::default()),
        Err(LoadError::TooLarge(_))
    ));
}

#[test]
fn metrics_report_fuel_and_memory() {
    let guest = component(Budget::default());
    let (_, metrics) = guest.validate_measured(b"{\"nodes\":[]}");
    assert!(metrics.fuel > 0 && metrics.fuel < Budget::default().fuel);
    assert!(metrics.peak_memory > 0);
}
