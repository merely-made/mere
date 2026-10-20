// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Product policy for a local lifecycle copy, separate from event-file writes.
use apparatus::collector::{CollectedBatch, DiagnosticProjection, RegistryCollector};
use apparatus::{Cursor, ObservationMetadata, RetentionLimits};
use registry::diagnostics::{
    ChannelConfig, ChannelRegistrationPolicy, DiagnosticEvent, DiagnosticSender,
    DiagnosticsChannelOwner, DiagnosticsRegistry, IngressLimits, RuntimeChannelDescriptor,
    diagnostic_channel,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::time::{Duration, Instant};

const CHANNEL: &str = "djinn.resident.lifecycle";

/// Explicit local bounds. A zero in any limit disables its queue/store.
/// Omitted configuration leaves diagnostics off; no profile or vault IDs enter it.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DiagnosticsConfig {
    pub ingress_records: usize,
    pub ingress_bytes: usize,
    pub ingress_age_secs: u64,
    pub retained_records: usize,
    pub retained_bytes: usize,
    pub retained_age_secs: u64,
    pub sample_rate: f32,
}

/// Owner-observed phases, never inferred from a quiet queue or successful write.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LifecyclePhase {
    Started,
    Listening,
    Ready,
    Stopping,
    StoppedOk,
    StoppedFailed,
    StoppedUnknown,
}
impl LifecyclePhase {
    pub(super) fn from_event(event: &str, fields: &Value) -> Option<Self> {
        match event {
            "started" => Some(Self::Started),
            "listening" => Some(Self::Listening),
            "ready" => Some(Self::Ready),
            "stopping" => Some(Self::Stopping),
            "stopped" => Some(match fields.get("ok").and_then(Value::as_bool) {
                Some(true) => Self::StoppedOk,
                Some(false) => Self::StoppedFailed,
                None => Self::StoppedUnknown,
            }),
            _ => None,
        }
    }
    fn label(self) -> &'static str {
        match self {
            Self::Started => "started",
            Self::Listening => "listening",
            Self::Ready => "ready",
            Self::Stopping => "stopping",
            Self::StoppedOk => "stopped_ok",
            Self::StoppedFailed => "stopped_failed",
            Self::StoppedUnknown => "stopped_unknown",
        }
    }
    fn from_label(label: &str) -> Option<Self> {
        [
            Self::Started,
            Self::Listening,
            Self::Ready,
            Self::Stopping,
            Self::StoppedOk,
            Self::StoppedFailed,
            Self::StoppedUnknown,
        ]
        .into_iter()
        .find(|phase| phase.label() == label)
    }
}

pub(super) struct LifecycleDiagnostics {
    sender: DiagnosticSender,
    collector: RegistryCollector<LifecyclePhase>,
    policy: DiagnosticsRegistry,
    started: Instant,
    sampled_out: u64,
    unmapped: u64,
    failure: Option<String>,
}
impl LifecycleDiagnostics {
    pub(super) fn new(config: DiagnosticsConfig) -> std::io::Result<Self> {
        if !config.sample_rate.is_finite() || !(0.0..=1.0).contains(&config.sample_rate) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "diagnostic sample_rate must be finite and between zero and one",
            ));
        }
        let (sender, receiver) = diagnostic_channel(IngressLimits {
            max_records: config.ingress_records,
            max_bytes: config.ingress_bytes,
            max_age: Duration::from_secs(config.ingress_age_secs),
        });
        let collector = RegistryCollector::new(
            receiver,
            uuid::Uuid::new_v4().to_string().into(),
            "djinn.resident.lifecycle".into(),
            uuid::Uuid::new_v4().to_string().into(),
            RetentionLimits {
                max_records: config.retained_records,
                max_bytes: config.retained_bytes,
                max_age: Duration::from_secs(config.retained_age_secs),
            },
        );
        let mut policy = DiagnosticsRegistry::default();
        policy
            .register_runtime_channel(
                RuntimeChannelDescriptor::info(
                    CHANNEL,
                    1,
                    DiagnosticsChannelOwner::runtime(),
                    Some("Redacted resident lifecycle phases".into()),
                ),
                ChannelRegistrationPolicy::KeepExisting,
            )
            .map_err(|e| std::io::Error::other(format!("diagnostic channel: {e:?}")))?;
        policy.set_config(
            CHANNEL,
            ChannelConfig {
                enabled: true,
                sample_rate: config.sample_rate,
                retention_count: config.retained_records,
            },
        );
        Ok(Self {
            sender,
            collector,
            policy,
            started: Instant::now(),
            sampled_out: 0,
            unmapped: 0,
            failure: None,
        })
    }
    pub(super) fn observe(&mut self, phase: Option<LifecyclePhase>) {
        if let Some(phase) = phase {
            if self.policy.should_emit_channel(CHANNEL) {
                // Fixed phase only. Private event fields are excluded BEFORE queue admission.
                self.sender.try_send(DiagnosticEvent::Event {
                    target: CHANNEL,
                    level: "INFO",
                    message: phase.label().into(),
                    fields: vec![],
                });
            } else {
                self.sampled_out = self.sampled_out.saturating_add(1);
            }
        } else {
            self.unmapped = self.unmapped.saturating_add(1);
        }
        if self.failure.is_none() {
            if let Err(error) = self.collector.collect(self.started.elapsed(), 1, |packet| {
                let DiagnosticEvent::Event {
                    target: CHANNEL,
                    message,
                    ..
                } = &packet.event
                else {
                    return None;
                };
                LifecyclePhase::from_label(message).map(|phase| DiagnosticProjection {
                    payload: phase,
                    encoded_bytes: phase.label().len(),
                    metadata: ObservationMetadata::default(),
                })
            }) {
                self.failure = Some(format!("{error:?}"));
            }
        }
    }
    pub(super) fn cursor(&self) -> Cursor {
        self.collector.cursor()
    }
    pub(super) fn read(
        &mut self,
        cursor: &mut Cursor,
        max_records: usize,
    ) -> Result<CollectedBatch<LifecyclePhase>, String> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        self.collector
            .read(cursor, max_records, self.started.elapsed())
            .map_err(|error| error.to_string())
    }
    pub(super) fn snapshot(&mut self) -> Result<Value, String> {
        let mut cursor = self.cursor();
        let batch = self.read(&mut cursor, usize::MAX)?;
        let ingress = batch.ingress.map(|stats| {
            let loss = stats.loss;
            json!({ "connected": stats.connected, "retained_records": stats.retained_records,
                "retained_bytes": stats.retained_bytes, "next_sequence": stats.next_sequence,
                "loss": { "disabled": loss.disabled, "full": loss.full, "oversized": loss.oversized,
                    "accounting_overflow": loss.accounting_overflow, "disconnected": loss.disconnected,
                    "contended": loss.contended, "poisoned": loss.poisoned, "sequence_exhausted": loss.sequence_exhausted,
                    "expired": loss.expired, "evicted": loss.evicted, "discarded_on_disconnect": loss.discarded_on_disconnect } })
        });
        let observations = batch.observations.map_payload(|record| {
            json!({ "phase": record.payload,
            "ingress_boot": record.ingress_boot, "ingress_sequence": record.ingress_sequence,
            "ingress_receipt_time": record.ingress_receipt_time })
        });
        Ok(
            json!({ "schema": "djinn.lifecycle-diagnostics/v1", "observations": observations,
            "ingress_boot": batch.ingress_boot, "ingress": ingress, "sampled_out": self.sampled_out,
            "unmapped": self.unmapped }),
        )
    }
}
