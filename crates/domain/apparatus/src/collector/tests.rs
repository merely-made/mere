// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;
use registry::diagnostics::{
    DiagnosticCorrelation, DiagnosticEvent, EmissionRejection, EmitOutcome, IngressLimits,
    diagnostic_channel, emit_span_duration, scoped_thread_sender, try_emit_correlated,
};

fn limits(records: usize) -> RetentionLimits {
    RetentionLimits {
        max_records: records,
        max_bytes: 16384,
        max_age: Duration::from_secs(60),
    }
}
fn project(packet: &DiagnosticPacket) -> Option<DiagnosticProjection<&'static str>> {
    // Actual raw text and caller correlation are deliberately not retained.
    matches!(packet.event, DiagnosticEvent::Span { .. }).then(|| DiagnosticProjection {
        payload: "span",
        encoded_bytes: 4,
        metadata: ObservationMetadata::default(),
    })
}
fn setup(
    ingress_count: usize,
    retained_count: usize,
) -> (
    registry::diagnostics::DiagnosticSender,
    RegistryCollector<&'static str>,
) {
    let (tx, rx) = diagnostic_channel(IngressLimits {
        max_records: ingress_count,
        ..Default::default()
    });
    let collector = RegistryCollector::new(
        rx,
        "run-1".into(),
        "host.registry".into(),
        "queue-1".into(),
        limits(retained_count),
    );
    (tx, collector)
}

#[test]
fn emitter_to_independent_readers_preserves_scopes_and_redaction() {
    let (tx, mut collector) = setup(8, 8);
    let _sender = scoped_thread_sender(tx);
    let mut inspector = collector.cursor();
    assert!(
        collector
            .read(&mut inspector, 0, Duration::ZERO)
            .unwrap()
            .ingress
            .is_none()
    );
    let mut receipt = collector.cursor();
    emit_span_duration("private span label", 12);
    try_emit_correlated(
        DiagnosticEvent::Event {
            target: "test",
            level: "INFO",
            message: "private text".into(),
            fields: vec![],
        },
        DiagnosticCorrelation {
            run: "private run".into(),
            source: "private source".into(),
            operation: "private operation".into(),
        },
    );
    let report = collector
        .collect(Duration::from_secs(1), 8, project)
        .unwrap();
    assert_eq!(
        (report.drained, report.filtered, report.retained),
        (2, 1, 1)
    );
    let first = collector
        .read(&mut inspector, 8, Duration::from_secs(1))
        .unwrap();
    let independent = collector
        .read(&mut receipt, 8, Duration::from_secs(1))
        .unwrap();
    assert_eq!(first.observations, independent.observations);
    let record = &first.observations.records[0];
    assert_eq!(record.envelope.reference.run, RunId::from("run-1"));
    assert_eq!(record.envelope.metadata.source_boot, None);
    assert_eq!(record.payload.ingress_boot, SourceBootId::from("queue-1"));
    assert_eq!(record.envelope.metadata.source_time, None);
    assert_eq!(record.envelope.metadata.cause, None);
    assert_eq!(record.envelope.metadata.operation, None);
    assert_eq!(record.envelope.metadata.presented_frame, None);
    assert_eq!(record.payload.ingress_sequence, 1);
    assert!(!format!("{first:?}").contains("private"));
}

#[test]
fn ingress_and_retained_overload_remain_distinct() {
    let (tx, mut collector) = setup(1, 1);
    let _sender = scoped_thread_sender(tx.clone());
    let mut cursor = collector.cursor();
    emit_span_duration("a", 1);
    assert!(matches!(
        tx.try_send(DiagnosticEvent::Span {
            name: "b",
            phase: registry::diagnostics::SpanPhase::Exit,
            duration_us: None
        }),
        EmitOutcome::Rejected {
            reason: EmissionRejection::Full,
            ..
        }
    ));
    collector
        .collect(Duration::from_secs(1), 8, project)
        .unwrap();
    emit_span_duration("c", 1);
    collector
        .collect(Duration::from_secs(2), 8, project)
        .unwrap();
    let batch = collector
        .read(&mut cursor, 8, Duration::from_secs(2))
        .unwrap();
    assert_eq!(batch.ingress.unwrap().loss.full, 1);
    assert_eq!(batch.observations.stats.loss.evicted, 1);
    assert_eq!(batch.observations.gaps.len(), 1);
    assert_eq!(
        (
            batch.observations.gaps[0].first_sequence,
            batch.observations.gaps[0].next_sequence
        ),
        (1, 2)
    );
    let last = &batch.observations.records[0];
    assert_eq!(last.envelope.reference.sequence, 2);
    assert_eq!(
        last.payload.ingress_sequence, 3,
        "ingress rejection consumes only ingress identity"
    );
}

#[test]
fn invalid_clock_does_not_consume_ingress_and_fresh_queue_reports_run_boundary() {
    let (tx, mut old) = setup(4, 4);
    let _sender = scoped_thread_sender(tx.clone());
    let mut cursor = old.cursor();
    emit_span_duration("a", 1);
    old.collect(Duration::from_secs(2), 4, project).unwrap();
    old.read(&mut cursor, 4, Duration::from_secs(2)).unwrap();
    emit_span_duration("b", 1);
    assert_eq!(
        old.collect(Duration::from_secs(1), 4, project).unwrap_err(),
        CollectError::Store(StoreError::TimeWentBackwards)
    );
    assert_eq!(tx.try_stats().unwrap().retained_records, 1);
    drop(old);
    assert_eq!(tx.loss().discarded_on_disconnect, 1);
    let (next_tx, rx) = diagnostic_channel(IngressLimits::default());
    let mut next = RegistryCollector::new(
        rx,
        "run-2".into(),
        "host.registry".into(),
        "queue-2".into(),
        limits(4),
    );
    let _next_sender = scoped_thread_sender(next_tx);
    emit_span_duration("new queue", 1);
    next.collect(Duration::from_secs(1), 4, project).unwrap();
    let batch = next.read(&mut cursor, 4, Duration::from_secs(1)).unwrap();
    assert_eq!(
        batch.observations.boundary.unwrap().previous_run,
        RunId::from("run-1")
    );
    assert_eq!(batch.observations.records[0].payload.ingress_sequence, 1);
    assert_eq!(
        Some(batch.observations.records[0].payload.ingress_boot.clone()),
        Some("queue-2".into())
    );
    assert_eq!(batch.ingress.unwrap().loss, Default::default());
}

#[test]
fn disabled_retention_and_overflow_accounting_are_visible_to_readers() {
    let (tx, mut collector) = setup(8, 0);
    let _sender = scoped_thread_sender(tx);
    let mut cursor = collector.cursor();
    emit_span_duration("a", 1);
    assert_eq!(
        collector
            .collect(Duration::ZERO, 8, project)
            .unwrap()
            .rejected,
        1
    );
    collector
        .set_retention_limits(limits(8), Duration::ZERO)
        .unwrap();
    emit_span_duration("b", 1);
    let report = collector
        .collect(Duration::ZERO, 8, |packet| {
            project(packet).map(|mut p| {
                p.encoded_bytes = usize::MAX;
                p
            })
        })
        .unwrap();
    assert_eq!(report.rejected, 1);
    let batch = collector.read(&mut cursor, 8, Duration::ZERO).unwrap();
    assert_eq!(batch.observations.stats.loss.rejected_disabled, 1);
    assert_eq!(batch.observations.stats.loss.rejected_accounting, 1);
    assert!(batch.observations.records.is_empty());
}

#[test]
fn producer_metadata_is_preserved_without_becoming_queue_identity() {
    let (tx, mut collector) = setup(4, 4);
    let mut cursor = collector.cursor();
    tx.try_send_correlated(
        DiagnosticEvent::Span {
            name: "owner-phase",
            phase: registry::diagnostics::SpanPhase::Exit,
            duration_us: None,
        },
        DiagnosticCorrelation {
            run: "producer-run".into(),
            source: "producer-source".into(),
            operation: "operation-2".into(),
        },
    );
    collector
        .collect(Duration::from_secs(10), 4, |packet| {
            let correlation = packet.correlation.as_ref().unwrap();
            Some(DiagnosticProjection {
                payload: "owner-phase",
                encoded_bytes: 11,
                metadata: ObservationMetadata {
                    source_boot: Some("producer-boot".into()),
                    source_time: Some(Duration::from_secs(90)),
                    operation: Some(correlation.operation.clone().into()),
                    cause: Some(crate::RecordRef {
                        run: correlation.run.clone().into(),
                        source: correlation.source.clone().into(),
                        sequence: 77,
                    }),
                    ..Default::default()
                },
            })
        })
        .unwrap();
    let batch = collector
        .read(&mut cursor, 4, Duration::from_secs(10))
        .unwrap();
    let record = &batch.observations.records[0];
    assert_eq!(record.envelope.reference.run, RunId::from("run-1"));
    assert_eq!(
        record.envelope.metadata.source_boot,
        Some("producer-boot".into())
    );
    assert_eq!(
        record.envelope.metadata.source_time,
        Some(Duration::from_secs(90))
    );
    assert_eq!(
        record.envelope.metadata.operation,
        Some("operation-2".into())
    );
    assert_eq!(
        record.envelope.metadata.cause.as_ref().unwrap().run,
        RunId::from("producer-run")
    );
    assert_eq!(
        record.envelope.metadata.cause.as_ref().unwrap().sequence,
        77
    );
    assert_eq!(record.payload.ingress_boot, SourceBootId::from("queue-1"));
    assert_eq!(record.payload.ingress_sequence, 1);
}

#[test]
fn process_no_sink_loss_is_observed_separately_from_collector_gaps() {
    use registry::diagnostics::{try_emit_event, uninstalled_loss};
    let (_tx, mut collector) = setup(4, 4);
    let mut cursor = collector.cursor();
    let before = uninstalled_loss().no_sink;
    assert_eq!(
        try_emit_event(DiagnosticEvent::Span {
            name: "uninstalled",
            phase: registry::diagnostics::SpanPhase::Exit,
            duration_us: None
        }),
        EmitOutcome::Rejected {
            sequence: None,
            reason: EmissionRejection::NoSink
        }
    );
    assert!(uninstalled_loss().no_sink > before);
    collector.collect(Duration::ZERO, 4, project).unwrap();
    let batch = collector.read(&mut cursor, 4, Duration::ZERO).unwrap();
    assert!(batch.observations.gaps.is_empty());
    assert_eq!(batch.observations.next_sequence, 1);
    assert_eq!(batch.ingress.unwrap().loss.no_sink, 0);
}
