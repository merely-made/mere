// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::diagnostics::{DiagnosticsCapability, DiagnosticsChannelOwner, DiagnosticsRegistry};

fn key(operation: &str) -> DiagnosticCorrelation {
    DiagnosticCorrelation {
        run: "run".into(),
        source: "worker".into(),
        operation: operation.into(),
    }
}
fn at(ms: u64) -> Duration {
    Duration::from_millis(ms)
}
fn registry() -> DiagnosticsRegistry {
    let mut registry = DiagnosticsRegistry::default();
    registry.invariants.clear();
    registry
        .register_invariant(
            DiagnosticsInvariant {
                invariant_id: "test.finish".into(),
                start_channel: "test.started".into(),
                terminal_channels: vec!["test.done".into()],
                timeout_ms: 10,
                owner: DiagnosticsChannelOwner::core(),
                enabled: true,
            },
            &[DiagnosticsCapability::RegisterInvariants],
        )
        .unwrap();
    registry
}
fn start(
    registry: &mut DiagnosticsRegistry,
    key: &DiagnosticCorrelation,
    now: u64,
) -> InvariantReport {
    registry
        .observe_correlated_channel_event("test.started", key, at(now))
        .unwrap()
}
fn done(
    registry: &mut DiagnosticsRegistry,
    key: &DiagnosticCorrelation,
    now: u64,
) -> InvariantReport {
    registry
        .observe_correlated_channel_event("test.done", key, at(now))
        .unwrap()
}

#[test]
fn reverse_terminal_only_clears_its_exact_operation() {
    let mut registry = registry();
    start(&mut registry, &key("A"), 0);
    start(&mut registry, &key("B"), 1);
    assert!(done(&mut registry, &key("B"), 2).violations.is_empty());
    let report = registry.sweep_correlated_invariants(at(10)).unwrap();
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].correlation, Some(key("A")));
    assert_eq!(report.violations[0].deadline, at(10));
    assert_eq!(report.stats.pending, 0);
}

#[test]
fn late_terminal_cannot_erase_timeout_at_deadline_equality() {
    let mut registry = registry();
    start(&mut registry, &key("A"), 0);
    let report = done(&mut registry, &key("A"), 10);
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].correlation, Some(key("A")));
    assert_eq!(report.stats.loss.unmatched_terminal, 1);
    assert!(
        registry
            .sweep_correlated_invariants(at(11))
            .unwrap()
            .violations
            .is_empty()
    );
}

#[test]
fn legacy_late_terminal_preserves_the_timeout_and_uses_a_separate_clock() {
    let mut registry = registry();
    registry.observe_channel_event("test.started", 1_000);
    start(&mut registry, &key("A"), 0);
    done(&mut registry, &key("A"), 1);
    assert_eq!(registry.invariant_stats().pending, 1);
    let violations = registry.observe_channel_event("test.done", 1_010);
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].deadline_unix_ms, 1_010);
}

#[test]
fn duplicate_and_unmatched_do_not_extend_or_complete_another_operation() {
    let mut registry = registry();
    start(&mut registry, &key("A"), 0);
    assert_eq!(start(&mut registry, &key("A"), 5).stats.loss.duplicate, 1);
    assert_eq!(
        done(&mut registry, &key("missing"), 6)
            .stats
            .loss
            .unmatched_terminal,
        1
    );
    let report = registry.sweep_correlated_invariants(at(10)).unwrap();
    assert_eq!(report.violations[0].correlation, Some(key("A")));
    assert_eq!(report.violations[0].deadline, at(10));
}

#[test]
fn matching_never_crosses_run_source_or_uncorrelated_mode() {
    let mut registry = registry();
    start(&mut registry, &key("A"), 0);
    registry.observe_channel_event("test.started", 500);
    let mut wrong = key("A");
    wrong.source = "another-worker".into();
    done(&mut registry, &wrong, 1);
    wrong = key("A");
    wrong.run = "another-run".into();
    done(&mut registry, &wrong, 2);
    registry.observe_channel_event("test.done", 501);
    assert_eq!(registry.invariant_stats().pending, 1);
    let report = registry.sweep_correlated_invariants(at(10)).unwrap();
    assert_eq!(report.violations[0].correlation, Some(key("A")));
    assert_eq!(report.stats.loss.unmatched_terminal, 2);
}

#[test]
fn sweep_checks_every_deadline_even_behind_a_later_one() {
    let mut tracker = InvariantTracker::default();
    let mut slow = DiagnosticsInvariant {
        invariant_id: "slow".into(),
        start_channel: "start".into(),
        terminal_channels: vec!["done".into()],
        timeout_ms: 1_000,
        owner: DiagnosticsChannelOwner::core(),
        enabled: true,
    };
    tracker
        .observe(std::iter::once(&slow), "start", Some(&key("slow")), at(0))
        .unwrap();
    slow.invariant_id = "fast".into();
    slow.timeout_ms = 10;
    tracker
        .observe(std::iter::once(&slow), "start", Some(&key("fast")), at(1))
        .unwrap();
    let report = tracker.sweep(true, at(11)).unwrap();
    assert_eq!(report.violations.len(), 1);
    assert_eq!(report.violations[0].correlation, Some(key("fast")));
    assert_eq!(report.stats.pending, 1);
}

#[test]
fn pending_count_bytes_zero_and_opaque_key_are_bounded() {
    let mut registry = registry();
    let defaults = InvariantLimits::default();
    registry.set_invariant_limits(InvariantLimits {
        max_pending: 1,
        ..defaults
    });
    let bytes = start(&mut registry, &key("A"), 0).stats.accounted_bytes;
    assert_eq!(start(&mut registry, &key("B"), 1).stats.loss.full, 1);
    let mut huge = key("huge");
    huge.operation = "x".repeat(defaults.max_bytes);
    assert_eq!(start(&mut registry, &huge, 2).stats.loss.oversized, 1);
    assert_eq!(registry.invariant_stats().pending, 1);
    registry.set_invariant_limits(InvariantLimits {
        max_bytes: bytes - 1,
        ..defaults
    });
    assert_eq!(registry.invariant_stats().pending, 0);
    assert_eq!(registry.invariant_stats().loss.evicted, 1);
    assert_eq!(start(&mut registry, &key("A"), 3).stats.loss.oversized, 2);
    for limits in [
        InvariantLimits {
            max_pending: 0,
            ..defaults
        },
        InvariantLimits {
            max_bytes: 0,
            ..defaults
        },
        InvariantLimits {
            max_age: Duration::ZERO,
            ..defaults
        },
    ] {
        registry.set_invariant_limits(limits);
        start(&mut registry, &key("A"), 4);
        assert_eq!(registry.invariant_stats().pending, 0);
        assert_eq!(registry.invariant_stats().accounted_bytes, 0);
    }
    assert_eq!(registry.invariant_stats().loss.disabled, 3);
}

#[test]
fn retention_expiry_is_lost_tracking_not_a_false_operation_failure() {
    let mut registry = registry();
    registry.set_invariant_limits(InvariantLimits {
        max_age: at(5),
        ..InvariantLimits::default()
    });
    start(&mut registry, &key("A"), 0);
    let report = registry.sweep_correlated_invariants(at(10)).unwrap();
    assert!(report.violations.is_empty());
    assert_eq!(report.stats.loss.expired, 1);
    assert_eq!(
        done(&mut registry, &key("A"), 10)
            .stats
            .loss
            .unmatched_terminal,
        1
    );
}

#[test]
fn time_reversal_is_visible_and_does_not_change_pending_deadlines() {
    let mut registry = registry();
    start(&mut registry, &key("A"), 5);
    assert_eq!(
        registry.observe_correlated_channel_event("test.done", &key("A"), at(4)),
        Err(InvariantTimeError)
    );
    assert_eq!(registry.invariant_stats().pending, 1);
    assert_eq!(registry.invariant_stats().loss.time_rejected, 1);
    let report = registry.sweep_correlated_invariants(at(15)).unwrap();
    assert_eq!(report.violations[0].deadline, at(15));
}

#[test]
fn channel_config_preserves_zero_retention() {
    let mut registry = registry();
    registry.set_config(
        "test.started",
        super::super::ChannelConfig {
            retention_count: 0,
            ..super::super::ChannelConfig::default()
        },
    );
    assert_eq!(registry.get_config("test.started").retention_count, 0);
}

#[test]
fn deadline_overflow_is_counted_without_retaining_a_token() {
    let mut registry = registry();
    let report = registry
        .observe_correlated_channel_event("test.started", &key("A"), Duration::MAX)
        .unwrap();
    assert_eq!(report.stats.pending, 0);
    assert_eq!(report.stats.loss.deadline_overflow, 1);
}
