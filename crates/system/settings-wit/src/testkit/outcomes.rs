// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

/// One transcript entry as a participant logs it.
pub fn log_line((label, result): &(String, String)) -> String {
    format!("{label}{SEPARATOR}{result}")
}

/// A participant's logged transcript, back as entries.
pub fn from_log(lines: &[String]) -> Vec<(String, String)> {
    lines
        .iter()
        .filter_map(|l| l.split_once(SEPARATOR))
        .map(|(a, b)| (a.to_string(), b.to_string()))
        .collect()
}

/// What [`run`] must have seen, whichever binding carried it.
pub fn assert_outcomes(transcript: &[(String, String)]) {
    assert_eq!(transcript.len(), CALLS, "{transcript:#?}");
    let expect = |label: &str, needle: &str| {
        let (_, result) = transcript
            .iter()
            .find(|(l, _)| l == label)
            .unwrap_or_else(|| panic!("no `{label}` in {transcript:#?}"));
        assert!(
            result.contains(needle),
            "`{label}`: expected `{needle}` in {result}"
        );
    };
    expect("snapshot", "Ok(Ok(SettingsSnapshot");
    expect("snapshot missing", "Unavailable(\"no such settings page\")");
    expect(
        "snapshot private",
        "Denied(\"no grant for product/private\")",
    );
    expect("snapshot about", "1.0 — ✓");
    expect("commit", "Committed(");
    expect("commit", "revision: 5");
    expect("stale guard", "Conflict(");
    expect("other domain", "Committed(");
    expect("other domain", "/tmp/ä — ✓");
    expect("integer edge", "Integer(-9223372036854775808)");
    expect("foreign owner", "Conflict(");
    expect("wrong kind", "Validation(");
    expect("nan", "zoom out of range");
    expect("canonical value", "Number(1.100000023841858)");
    expect("storage", "Storage(");
    expect("unknown setting", "unknown setting");
    expect("denied write", "Denied(");
    expect("snapshot after", "Boolean(true)");
    expect("snapshot after", "revision: 6");
    let no_disconnect = transcript.iter().all(|(_, r)| !r.starts_with("Err("));
    assert!(no_disconnect, "{transcript:#?}");
}

/// Values at the edges, for crossing a component unchanged.
pub fn edge_snapshot() -> SettingsSnapshot {
    let owner = FixtureOwner {
        zoom: f64::MIN_POSITIVE,
        history: i64::MAX,
        theme: String::new(),
        appearance_rev: u64::MAX,
        ..FixtureOwner::default()
    };
    let s = owner
        .snapshot(&SettingsRef(APPEARANCE.into()))
        .expect("fixture page");
    crate::wire_snapshot(&s)
}

/// Every write outcome case, for crossing a component unchanged.
pub fn edge_outcomes() -> Vec<WriteOutcome> {
    use crate::bindings::{CommittedWrite, Conflict, Refusal, ValidationFailure};
    let guard = Guard {
        owner: "ä".into(),
        domain: String::new(),
        revision: u64::MAX,
    };
    let refusal = || Refusal {
        request_id: "r".into(),
        message: "x".repeat(300),
    };
    vec![
        WriteOutcome::Committed(CommittedWrite {
            request_id: "r".into(),
            reference: APPEARANCE.into(),
            setting_id: "ui.zoom".into(),
            value: Value::Number(f64::NEG_INFINITY),
            guard: guard.clone(),
        }),
        WriteOutcome::Conflict(Conflict {
            request_id: String::new(),
            current: guard,
        }),
        WriteOutcome::Unavailable(refusal()),
        WriteOutcome::Validation(ValidationFailure {
            request_id: "r".into(),
            setting_id: "s".into(),
            message: "✓".into(),
        }),
        WriteOutcome::Storage(refusal()),
        WriteOutcome::Denied(refusal()),
    ]
}

/// The failure cases, for crossing a component unchanged.
pub fn edge_failures() -> Vec<Failure> {
    vec![
        Failure::Denied("ä — ✓".into()),
        Failure::Unavailable(String::new()),
    ]
}
