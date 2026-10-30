// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Versioned, value-free observations for shared reactive decisions.
//!
//! Events are sent through `tracing` and do not affect view execution. A
//! product subscriber chooses whether to retain them and owns correlation to
//! its observation store.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use serde::{Deserialize, Serialize};

pub const REACTIVE_DIAGNOSTIC_SCHEMA_VERSION: u16 = 1;
pub const MAX_CAUSE_EDGES: usize = 8;

static NEXT_PRODUCER_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerRef {
    pub producer_id: u64,
    pub sequence: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub owner_id: u64,
    pub slot: u64,
    pub route_generation: u64,
    pub placement_generation: u64,
}

/// Start a value-free span for one retained component operation. Product
/// tracing layers can join these allowlisted numeric fields to event records.
pub fn diagnostic_span(
    subject: Subject,
    update_id: Option<u64>,
    operation: Option<ProducerRef>,
) -> tracing::Span {
    let span = tracing::info_span!(
        target: "cambium::reactive",
        "reactive_operation",
        schema_version = REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
        owner_id = subject.owner_id,
        slot = subject.slot,
        route_generation = subject.route_generation,
        placement_generation = subject.placement_generation,
        update_id = tracing::field::Empty,
        operation_producer_id = tracing::field::Empty,
        operation_sequence = tracing::field::Empty,
    );
    if let Some(update_id) = update_id {
        span.record("update_id", update_id);
    }
    if let Some(operation) = operation {
        span.record("operation_producer_id", operation.producer_id);
        span.record("operation_sequence", operation.sequence);
    }
    span
}

/// Fixed event vocabulary shared by Cambium producers and product collectors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticKind {
    SettingsRequested,
    SettingsCommitted,
    SettingsConflict,
    SettingsValidation,
    SettingsStorage,
    SettingsUnavailable,
    RevisionAdvanced,
    ComponentCreated,
    ComponentMoved,
    ComponentHidden,
    ComponentRevealed,
    ComponentReplaced,
    ComponentDisposed,
    CompositionRefused,
    MutationBatch,
    RendererStyleCompleted,
    RendererLayoutCompleted,
    RendererPaintCompleted,
    RendererWorkFailed,
    HostFrameSubmitted,
    HostFrameUnavailable,
    ComponentBuilt,
    ComponentMemoSkipped,
    ComponentRouteRefreshed,
    ComponentRebuildStarted,
    ComponentRebuildCompleted,
    ComponentMessageRouted,
    ComponentTornDown,
    ComponentOperationStarted,
    AnyViewReused,
    AnyViewReplaced,
    AnyViewTornDown,
    AnyViewGenerationMatched,
    AnyViewGenerationRefused,
    MessagePathAdvanced,
    MessageConsumed,
    MessageDeclined,
    MessageStale,
    SequenceChildAdded,
    SequenceChildRemoved,
    SequenceMessageMatched,
    SequenceMessageRefused,
    CompositionInputInvalidated,
    CompositionInputMerged,
    CompositionInputCaptured,
    ComponentInputSkipped,
    ComponentInputEvaluated,
    ComponentInputDeferred,
    CompositionFlushCompleted,
    CauseEdge,
    CausesOmitted,
    SequenceExhausted,
    ProducerIdUnavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagnosticReason {
    #[default]
    None,
    DependencyChanged,
    Coalesced,
    EqualAndClean,
    LocalStateDirty,
    RouteChanged,
    Hidden,
    ActiveInteraction,
    StaleRoute,
    Accepted,
    Refused,
    Replaced,
    Teardown,
    GenerationMatched,
    GenerationMismatch,
    MessageConsumed,
    MessageDeclined,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseRelation {
    TriggeredBy,
    InvalidatedBy,
    CoalescedFrom,
    CapturedFrom,
    EvaluatedFrom,
    CommittedBy,
    ProducedBy,
    RenderedFrom,
    SubmittedFrom,
    ScopeContains,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticDetails {
    pub revision: Option<u64>,
    pub expected_revision: Option<u64>,
    pub current_revision: Option<u64>,
    pub update_id: Option<u64>,
    pub mutation_cursor_start: Option<u64>,
    pub mutation_cursor_end: Option<u64>,
    pub dependency_id: Option<u64>,
    pub notice_count: Option<u64>,
    pub window_id: Option<u64>,
    pub surface_generation: Option<u64>,
    pub frame_id: Option<u64>,
    pub path_depth: Option<u16>,
    pub path_truncated: bool,
    pub reason: DiagnosticReason,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "record", rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum ReactiveDiagnosticEvent {
    Decision {
        schema_version: u16,
        reference: ProducerRef,
        kind: DiagnosticKind,
        subject: Option<Subject>,
        details: DiagnosticDetails,
    },
    CauseEdge {
        schema_version: u16,
        reference: ProducerRef,
        child: ProducerRef,
        parent: ProducerRef,
        relation: CauseRelation,
    },
    CausesOmitted {
        schema_version: u16,
        reference: ProducerRef,
        child: ProducerRef,
        omitted_count: u16,
    },
    SequenceExhausted {
        schema_version: u16,
        producer_id: u64,
        last_sequence: u64,
    },
    ProducerIdUnavailable {
        schema_version: u16,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProducerError {
    IdExhausted,
    SequenceExhausted {
        producer_id: u64,
        last_sequence: u64,
    },
}

struct ProducerState {
    producer_id: u64,
    next_sequence: AtomicU64,
    dropped_events: AtomicU64,
}

/// Cloneable event producer. Clones share a checked, process-local sequence.
#[derive(Clone)]
pub struct Producer(Arc<ProducerState>);

impl Producer {
    /// Allocate a process-unique id. IDs start at one; zero is reserved for
    /// Meristem's core-only tracing producer.
    pub fn new() -> Result<Self, ProducerError> {
        let producer_id = NEXT_PRODUCER_ID
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |next| {
                next.checked_add(1)
            })
            .map_err(|_| {
                emit_unscoped_event(ReactiveDiagnosticEvent::ProducerIdUnavailable {
                    schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
                });
                ProducerError::IdExhausted
            })?;
        Ok(Self(Arc::new(ProducerState {
            producer_id,
            next_sequence: AtomicU64::new(1),
            dropped_events: AtomicU64::new(0),
        })))
    }

    pub fn producer_id(&self) -> u64 {
        self.0.producer_id
    }

    /// Events that could not be serialized/emitted or whose sequence could
    /// not be allocated. This never changes the caller's decision.
    pub fn dropped_events(&self) -> u64 {
        self.0.dropped_events.load(Ordering::Relaxed)
    }

    pub fn emit(
        &self,
        kind: DiagnosticKind,
        subject: Option<Subject>,
        details: DiagnosticDetails,
    ) -> Result<ProducerRef, ProducerError> {
        let reference = self.next_ref()?;
        self.emit_event(ReactiveDiagnosticEvent::Decision {
            schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
            reference,
            kind,
            subject,
            details,
        });
        Ok(reference)
    }

    /// Emit one bounded edge record per admitted cause. Further edges are
    /// represented by one explicit CausesOmitted event, never silently lost.
    pub fn emit_with_causes(
        &self,
        kind: DiagnosticKind,
        subject: Option<Subject>,
        details: DiagnosticDetails,
        causes: &[(CauseRelation, ProducerRef)],
    ) -> Result<ProducerRef, ProducerError> {
        let child = self.emit(kind, subject, details)?;
        for (relation, parent) in causes.iter().take(MAX_CAUSE_EDGES) {
            let reference = self.next_ref()?;
            self.emit_event(ReactiveDiagnosticEvent::CauseEdge {
                schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
                reference,
                child,
                parent: *parent,
                relation: *relation,
            });
        }
        let omitted = causes.len().saturating_sub(MAX_CAUSE_EDGES);
        if omitted > 0 {
            let reference = self.next_ref()?;
            self.emit_event(ReactiveDiagnosticEvent::CausesOmitted {
                schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
                reference,
                child,
                omitted_count: u16::try_from(omitted).unwrap_or(u16::MAX),
            });
        }
        Ok(child)
    }

    /// Emit one causal edge after both referenced decisions exist. The edge
    /// preserves event timing when the completed operation is only known after
    /// synchronous work returns.
    pub(crate) fn emit_cause_edge(
        &self,
        child: ProducerRef,
        parent: ProducerRef,
        relation: CauseRelation,
    ) -> Result<ProducerRef, ProducerError> {
        let reference = self.next_ref()?;
        self.emit_event(ReactiveDiagnosticEvent::CauseEdge {
            schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
            reference,
            child,
            parent,
            relation,
        });
        Ok(reference)
    }

    fn next_ref(&self) -> Result<ProducerRef, ProducerError> {
        match self
            .0
            .next_sequence
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |sequence| {
                sequence.checked_add(1)
            }) {
            Ok(sequence) => Ok(ProducerRef {
                producer_id: self.0.producer_id,
                sequence,
            }),
            Err(last_sequence) => {
                increment_dropped(&self.0.dropped_events);
                self.emit_event(ReactiveDiagnosticEvent::SequenceExhausted {
                    schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
                    producer_id: self.0.producer_id,
                    last_sequence,
                });
                Err(ProducerError::SequenceExhausted {
                    producer_id: self.0.producer_id,
                    last_sequence,
                })
            },
        }
    }

    fn emit_event(&self, event: ReactiveDiagnosticEvent) {
        emit_typed_event(&event, &self.0.dropped_events);
    }
}

fn emit_typed_event(event: &ReactiveDiagnosticEvent, dropped: &AtomicU64) {
    let Ok(payload) = serde_json::to_string(event) else {
        increment_dropped(dropped);
        return;
    };
    // The payload contains only the fixed enum and numeric fields above.
    tracing::info!(target: "cambium::reactive", payload = payload.as_str(), "shared reactive decision");
}

fn increment_dropped(dropped: &AtomicU64) {
    let _ = dropped.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
        Some(value.saturating_add(1))
    });
}

fn emit_unscoped_event(event: ReactiveDiagnosticEvent) {
    let Ok(payload) = serde_json::to_string(&event) else {
        return;
    };
    tracing::warn!(target: "cambium::reactive", payload = payload.as_str(), "reactive diagnostic coverage gap");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn producer_refs_are_scoped_and_cause_edges_are_bounded_with_loss_record() {
        let producer = Producer::new().expect("producer id available");
        let causes = (0..10)
            .map(|n| {
                (
                    CauseRelation::TriggeredBy,
                    ProducerRef {
                        producer_id: 900,
                        sequence: n,
                    },
                )
            })
            .collect::<Vec<_>>();
        let child = producer
            .emit_with_causes(
                DiagnosticKind::CompositionInputCaptured,
                Some(Subject {
                    owner_id: 7,
                    slot: 11,
                    route_generation: 2,
                    placement_generation: 4,
                }),
                DiagnosticDetails::default(),
                &causes,
            )
            .expect("sequence available");

        assert_eq!(child.producer_id, producer.producer_id());
        assert_eq!(child.sequence, 1);
        // One decision, eight edges, one explicit omitted-causes record.
        let next = producer
            .emit(
                DiagnosticKind::CompositionFlushCompleted,
                None,
                DiagnosticDetails::default(),
            )
            .expect("sequence available");
        assert_eq!(next.sequence, 11);
    }

    #[test]
    fn typed_payload_rejects_unrecognized_fields() {
        let event = ReactiveDiagnosticEvent::Decision {
            schema_version: REACTIVE_DIAGNOSTIC_SCHEMA_VERSION,
            reference: ProducerRef {
                producer_id: 1,
                sequence: 1,
            },
            kind: DiagnosticKind::SettingsCommitted,
            subject: None,
            details: DiagnosticDetails::default(),
        };
        let mut json = serde_json::to_value(event).expect("serializable typed event");
        json.as_object_mut().expect("tagged event object").insert(
            "unrecognized_text".into(),
            serde_json::json!("must-not-be-accepted"),
        );
        assert!(serde_json::from_value::<ReactiveDiagnosticEvent>(json).is_err());
    }
}
