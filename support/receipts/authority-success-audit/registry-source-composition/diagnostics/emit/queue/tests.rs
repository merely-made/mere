// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

fn event() -> DiagnosticEvent {
    DiagnosticEvent::MessageSent {
        channel_id: "test.started",
        byte_len: usize::MAX,
    }
}
fn limits(count: usize) -> IngressLimits {
    IngressLimits {
        max_records: count,
        max_bytes: 4096,
        max_age: Duration::from_secs(10),
    }
}
fn rejected(outcome: EmitOutcome, expected: EmissionRejection, sequence: Option<u64>) {
    assert_eq!(
        outcome,
        EmitOutcome::Rejected {
            sequence,
            reason: expected
        }
    );
}

#[test]
fn full_queue_rejects_newest_and_drains_are_bounded() {
    let (tx, rx) = diagnostic_channel(limits(2));
    assert_eq!(tx.try_send(event()), EmitOutcome::Accepted { sequence: 1 });
    assert_eq!(tx.try_send(event()), EmitOutcome::Accepted { sequence: 2 });
    rejected(tx.try_send(event()), EmissionRejection::Full, Some(3));
    assert!(rx.try_drain(0).unwrap().packets.is_empty());
    assert_eq!(tx.try_stats().unwrap().retained_records, 2);
    assert_eq!(rx.try_drain(1).unwrap().packets[0].sequence, 1);
    assert_eq!(rx.try_drain(usize::MAX).unwrap().packets[0].sequence, 2);
    assert_eq!(tx.loss().full, 1);
    assert_eq!(tx.try_send(event()), EmitOutcome::Accepted { sequence: 4 });
}

#[test]
fn any_zero_disables_and_runtime_limits_release_records() {
    for policy in [
        IngressLimits {
            max_records: 0,
            ..limits(2)
        },
        IngressLimits {
            max_bytes: 0,
            ..limits(2)
        },
        IngressLimits {
            max_age: Duration::ZERO,
            ..limits(2)
        },
    ] {
        let (tx, rx) = diagnostic_channel(limits(2));
        tx.try_send(event());
        let stats = rx.try_set_limits(policy).unwrap();
        assert_eq!(stats.retained_records, 0);
        assert_eq!(stats.retained_bytes, 0);
        rejected(tx.try_send(event()), EmissionRejection::Disabled, Some(2));
        assert_eq!(tx.loss().disabled, 1);
    }
}

#[test]
fn accounting_measures_payload_and_envelope_not_message_byte_len() {
    let bytes = accounted_packet_bytes(&event(), None).unwrap();
    assert_eq!(bytes, 8 + "test.started".len() + 8 + 22);
    let key = DiagnosticCorrelation {
        run: "run".into(),
        source: "worker".into(),
        operation: "op".into(),
    };
    assert_eq!(
        accounted_packet_bytes(&event(), Some(&key)),
        Some(bytes + 24 + 3 + 6 + 2)
    );
    let (tx, rx) = diagnostic_channel(IngressLimits {
        max_bytes: bytes,
        ..limits(2)
    });
    assert!(matches!(tx.try_send(event()), EmitOutcome::Accepted { .. }));
    rejected(tx.try_send(event()), EmissionRejection::Full, Some(2));
    rx.try_drain(1).unwrap();
    rejected(
        tx.try_send_correlated(event(), key),
        EmissionRejection::Oversized,
        Some(3),
    );
    assert_eq!(tx.loss().oversized, 1);
    let large = DiagnosticEvent::Event {
        target: "test",
        level: "INFO",
        message: "x".repeat(1000),
        fields: vec![super::super::StructuredPayloadField {
            name: "value",
            value: "y".repeat(1000),
        }],
    };
    rejected(tx.try_send(large), EmissionRejection::Oversized, Some(4));
    assert!(rx.try_drain(1).unwrap().packets.is_empty());
}

#[test]
fn expiry_at_equality_precedes_admission() {
    let (tx, rx) = diagnostic_channel(IngressLimits {
        max_age: Duration::from_secs(1),
        ..limits(1)
    });
    tx.send(event(), None, Some(Duration::ZERO));
    assert_eq!(
        tx.send(event(), None, Some(Duration::from_secs(1))),
        EmitOutcome::Accepted { sequence: 2 }
    );
    let batch = rx.drain(1, Some(Duration::from_secs(1))).unwrap();
    assert_eq!(batch.packets[0].sequence, 2);
    assert_eq!(batch.stats.loss.expired, 1);
    assert_eq!(batch.stats.loss.full, 0);
}

#[test]
fn disconnected_and_contended_producers_do_not_wait_or_allocate_identity() {
    let (tx, rx) = diagnostic_channel(limits(1));
    let held = tx.0.state.lock().unwrap();
    let worker = std::thread::spawn({
        let tx = tx.clone();
        move || tx.try_send(event())
    });
    rejected(worker.join().unwrap(), EmissionRejection::Contended, None);
    assert_eq!(tx.loss().contended, 1);
    assert_eq!(held.next_sequence, Some(1));
    drop(held);
    tx.try_send(event());
    drop(rx);
    rejected(tx.try_send(event()), EmissionRejection::Disconnected, None);
    assert_eq!(tx.loss().disconnected, 1);
    assert_eq!(tx.loss().discarded_on_disconnect, 1);
}

#[test]
fn poisoned_queue_returns_loss_without_panicking() {
    let (tx, _rx) = diagnostic_channel(limits(1));
    let _ = std::panic::catch_unwind(|| {
        let _held = tx.0.state.lock().unwrap();
        panic!("deliberately poison queue fixture");
    });
    rejected(tx.try_send(event()), EmissionRejection::Poisoned, None);
    assert_eq!(tx.loss().poisoned, 1);
    assert_eq!(tx.try_stats(), Err(IngressAccessError::Poisoned));
}

#[test]
fn exhausted_sequence_never_wraps_or_reuses_an_identity() {
    let (tx, rx) = diagnostic_channel(limits(2));
    tx.0.state.lock().unwrap().next_sequence = Some(u64::MAX);
    assert_eq!(
        tx.try_send(event()),
        EmitOutcome::Accepted { sequence: u64::MAX }
    );
    rejected(
        tx.try_send(event()),
        EmissionRejection::SequenceExhausted,
        None,
    );
    let batch = rx.try_drain(2).unwrap();
    assert_eq!(batch.packets.len(), 1);
    assert_eq!(batch.stats.next_sequence, None);
    assert_eq!(batch.stats.loss.sequence_exhausted, 1);
}

#[test]
fn scoped_sender_restores_previous_sink_and_no_sink_is_observable() {
    use super::super::{scoped_thread_sender, try_emit_event, uninstalled_loss};
    let before = uninstalled_loss().no_sink;
    rejected(try_emit_event(event()), EmissionRejection::NoSink, None);
    assert!(uninstalled_loss().no_sink >= before.saturating_add(1));
    let (outer, outer_rx) = diagnostic_channel(limits(2));
    let (inner, inner_rx) = diagnostic_channel(limits(2));
    let outer_scope = scoped_thread_sender(outer);
    {
        let _inner_scope = scoped_thread_sender(inner);
        try_emit_event(event());
    }
    try_emit_event(event());
    assert_eq!(inner_rx.try_drain(2).unwrap().packets.len(), 1);
    assert_eq!(outer_rx.try_drain(2).unwrap().packets.len(), 1);
    drop(outer_scope);
    rejected(try_emit_event(event()), EmissionRejection::NoSink, None);
}

#[test]
fn disconnected_queue_cleanup_can_follow_a_contended_drop() {
    let (tx, rx) = diagnostic_channel(limits(1));
    tx.try_send(event());
    let held = tx.0.state.lock().unwrap();
    drop(rx);
    assert_eq!(tx.loss().discarded_on_disconnect, 0);
    drop(held);
    let stats = tx.try_stats().unwrap();
    assert!(!stats.connected);
    assert_eq!(stats.retained_records, 0);
    assert_eq!(stats.loss.discarded_on_disconnect, 1);
    assert_eq!(tx.try_stats().unwrap().loss.discarded_on_disconnect, 1);
}
