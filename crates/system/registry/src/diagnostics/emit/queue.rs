// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::DiagnosticEvent;
use crate::diagnostics::DiagnosticCorrelation;
use std::{
    collections::VecDeque,
    sync::{
        Arc, Mutex, MutexGuard, TryLockError,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

/// Independent ingress policy. Any zero disables buffering. Full queues reject
/// the newest attempt, preserving the order of already admitted observations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IngressLimits {
    pub max_records: usize,
    pub max_bytes: usize,
    pub max_age: Duration,
}
impl Default for IngressLimits {
    fn default() -> Self {
        Self {
            max_records: 256,
            max_bytes: 262_144,
            max_age: Duration::from_secs(300),
        }
    }
}
impl IngressLimits {
    fn enabled(self) -> bool {
        self.max_records > 0 && self.max_bytes > 0 && !self.max_age.is_zero()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(usize)]
pub enum EmissionRejection {
    NoSink,
    Disabled,
    Full,
    Oversized,
    AccountingOverflow,
    Disconnected,
    Contended,
    Poisoned,
    SequenceExhausted,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitOutcome {
    Accepted {
        sequence: u64,
    },
    Rejected {
        sequence: Option<u64>,
        reason: EmissionRejection,
    },
}

/// Saturating counts describe actual observed loss. Contention, poisoning and
/// no-sink loss have no packet sequence; never invent record gaps for them.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EmissionLoss {
    pub no_sink: u64,
    pub disabled: u64,
    pub full: u64,
    pub oversized: u64,
    pub accounting_overflow: u64,
    pub disconnected: u64,
    pub contended: u64,
    pub poisoned: u64,
    pub sequence_exhausted: u64,
    pub expired: u64,
    pub evicted: u64,
    pub discarded_on_disconnect: u64,
}
pub(super) struct Counters([AtomicU64; 12]);
impl Counters {
    pub(super) const fn new() -> Self {
        Self([const { AtomicU64::new(0) }; 12])
    }
    fn add(&self, index: usize, count: u64) {
        let _ = self.0[index].fetch_update(Ordering::Relaxed, Ordering::Relaxed, |old| {
            Some(old.saturating_add(count))
        });
    }
    pub(super) fn reject(&self, reason: EmissionRejection) {
        self.add(reason as usize, 1);
    }
    pub(super) fn snapshot(&self) -> EmissionLoss {
        let get = |index: usize| self.0[index].load(Ordering::Relaxed);
        EmissionLoss {
            no_sink: get(0),
            disabled: get(1),
            full: get(2),
            oversized: get(3),
            accounting_overflow: get(4),
            disconnected: get(5),
            contended: get(6),
            poisoned: get(7),
            sequence_exhausted: get(8),
            expired: get(9),
            evicted: get(10),
            discarded_on_disconnect: get(11),
        }
    }
}

#[derive(Clone, Debug)]
pub struct DiagnosticPacket {
    pub sequence: u64,
    pub receipt_time: Duration,
    pub correlation: Option<DiagnosticCorrelation>,
    pub event: DiagnosticEvent,
    pub accounted_bytes: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IngressStats {
    pub connected: bool,
    pub retained_records: usize,
    pub retained_bytes: usize,
    /// None means the sequence identity space is exhausted, never wrapped.
    pub next_sequence: Option<u64>,
    pub loss: EmissionLoss,
}
#[derive(Clone, Debug)]
pub struct IngressBatch {
    pub packets: Vec<DiagnosticPacket>,
    pub stats: IngressStats,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IngressAccessError {
    Contended,
    Poisoned,
}

struct State {
    queue: VecDeque<DiagnosticPacket>,
    bytes: usize,
    next_sequence: Option<u64>,
    limits: IngressLimits,
}
struct Shared {
    state: Mutex<State>,
    started: Instant,
    connected: AtomicBool,
    loss: Counters,
}
/// Cloneable producer. Admission uses try_lock and never waits for a consumer
/// or a queue lock. Payload construction/redaction remain producer obligations.
#[derive(Clone)]
pub struct DiagnosticSender(Arc<Shared>);
/// One bounded staging consumer. Independent inspection/receipt cursors belong
/// on the downstream Apparatus store, rather than competing queue drains.
pub struct DiagnosticReceiver(Arc<Shared>);

pub fn diagnostic_channel(limits: IngressLimits) -> (DiagnosticSender, DiagnosticReceiver) {
    let shared = Arc::new(Shared {
        state: Mutex::new(State {
            queue: VecDeque::new(),
            bytes: 0,
            next_sequence: Some(1),
            limits,
        }),
        started: Instant::now(),
        connected: AtomicBool::new(true),
        loss: Counters::new(),
    });
    (DiagnosticSender(shared.clone()), DiagnosticReceiver(shared))
}

impl Shared {
    fn lock(&self) -> Result<MutexGuard<'_, State>, IngressAccessError> {
        self.state.try_lock().map_err(|error| match error {
            TryLockError::WouldBlock => IngressAccessError::Contended,
            TryLockError::Poisoned(_) => IngressAccessError::Poisoned,
        })
    }
    fn expire(&self, state: &mut State, now: Duration) {
        while state
            .queue
            .front()
            .is_some_and(|record| now.saturating_sub(record.receipt_time) >= state.limits.max_age)
        {
            if let Some(record) = state.queue.pop_front() {
                state.bytes -= record.accounted_bytes;
            }
            self.loss.add(9, 1);
        }
    }
    fn stats(&self, state: &State) -> IngressStats {
        IngressStats {
            connected: self.connected.load(Ordering::Acquire),
            retained_records: state.queue.len(),
            retained_bytes: state.bytes,
            next_sequence: state.next_sequence,
            loss: self.loss.snapshot(),
        }
    }
}

impl DiagnosticSender {
    pub fn loss(&self) -> EmissionLoss {
        self.0.loss.snapshot()
    }
    pub fn try_stats(&self) -> Result<IngressStats, IngressAccessError> {
        let mut state = self.0.lock()?;
        if !self.0.connected.load(Ordering::Acquire) {
            self.0.loss.add(11, state.queue.len() as u64);
            state.queue.clear();
            state.bytes = 0;
        }
        self.0.expire(&mut state, self.0.started.elapsed());
        Ok(self.0.stats(&state))
    }
    pub fn try_send(&self, event: DiagnosticEvent) -> EmitOutcome {
        self.send(event, None, None)
    }
    pub fn try_send_correlated(
        &self,
        event: DiagnosticEvent,
        key: DiagnosticCorrelation,
    ) -> EmitOutcome {
        self.send(event, Some(key), None)
    }
    fn rejected(&self, sequence: Option<u64>, reason: EmissionRejection) -> EmitOutcome {
        self.0.loss.reject(reason);
        EmitOutcome::Rejected { sequence, reason }
    }
    fn send(
        &self,
        event: DiagnosticEvent,
        correlation: Option<DiagnosticCorrelation>,
        supplied_time: Option<Duration>,
    ) -> EmitOutcome {
        if !self.0.connected.load(Ordering::Acquire) {
            return self.rejected(None, EmissionRejection::Disconnected);
        }
        let mut state = match self.0.lock() {
            Ok(state) => state,
            Err(IngressAccessError::Contended) => {
                return self.rejected(None, EmissionRejection::Contended);
            },
            Err(IngressAccessError::Poisoned) => {
                return self.rejected(None, EmissionRejection::Poisoned);
            },
        };
        if !self.0.connected.load(Ordering::Acquire) {
            return self.rejected(None, EmissionRejection::Disconnected);
        }
        let Some(sequence) = state.next_sequence else {
            return self.rejected(None, EmissionRejection::SequenceExhausted);
        };
        state.next_sequence = sequence.checked_add(1);
        let now = supplied_time.unwrap_or_else(|| self.0.started.elapsed());
        self.0.expire(&mut state, now);
        if !state.limits.enabled() {
            return self.rejected(Some(sequence), EmissionRejection::Disabled);
        }
        let Some(bytes) = accounted_packet_bytes(&event, correlation.as_ref()) else {
            return self.rejected(Some(sequence), EmissionRejection::AccountingOverflow);
        };
        if bytes > state.limits.max_bytes {
            return self.rejected(Some(sequence), EmissionRejection::Oversized);
        }
        if state.queue.len() >= state.limits.max_records
            || bytes > state.limits.max_bytes - state.bytes
        {
            return self.rejected(Some(sequence), EmissionRejection::Full);
        }
        state.bytes += bytes;
        state.queue.push_back(DiagnosticPacket {
            sequence,
            receipt_time: now,
            correlation,
            event,
            accounted_bytes: bytes,
        });
        EmitOutcome::Accepted { sequence }
    }
}

impl DiagnosticReceiver {
    /// A zero read does not consume or expire the staging queue.
    pub fn try_drain(&self, max_records: usize) -> Result<IngressBatch, IngressAccessError> {
        self.drain(max_records, None)
    }
    fn drain(
        &self,
        max_records: usize,
        supplied_time: Option<Duration>,
    ) -> Result<IngressBatch, IngressAccessError> {
        let mut state = self.0.lock()?;
        if max_records == 0 {
            return Ok(IngressBatch {
                packets: Vec::new(),
                stats: self.0.stats(&state),
            });
        }
        self.0.expire(
            &mut state,
            supplied_time.unwrap_or_else(|| self.0.started.elapsed()),
        );
        let count = max_records.min(state.queue.len());
        let packets: Vec<_> = state.queue.drain(..count).collect();
        state.bytes -= packets
            .iter()
            .map(|packet| packet.accounted_bytes)
            .sum::<usize>();
        Ok(IngressBatch {
            packets,
            stats: self.0.stats(&state),
        })
    }
    pub fn try_set_limits(
        &self,
        limits: IngressLimits,
    ) -> Result<IngressStats, IngressAccessError> {
        let mut state = self.0.lock()?;
        state.limits = limits;
        if limits.enabled() {
            self.0.expire(&mut state, self.0.started.elapsed());
        }
        while !limits.enabled()
            || state.queue.len() > limits.max_records
            || state.bytes > limits.max_bytes
        {
            let Some(packet) = state.queue.pop_front() else {
                break;
            };
            state.bytes -= packet.accounted_bytes;
            self.0.loss.add(10, 1);
        }
        Ok(self.0.stats(&state))
    }
}
impl Drop for DiagnosticReceiver {
    fn drop(&mut self) {
        self.0.connected.store(false, Ordering::Release);
        if let Ok(mut state) = self.0.lock() {
            self.0.loss.add(11, state.queue.len() as u64);
            state.queue.clear();
            state.bytes = 0;
        }
    }
}

/// Canonical accounted encoding: one-byte variant/option tags, u64 integer and
/// string/vector lengths, UTF-8 string bytes, duration as u64 seconds/u32 nanos.
/// byte_len describes a message, not allocated diagnostic bytes. The packet
/// adds u64 sequence, duration and optional correlation. This is not allocator
/// memory or JSON export size; event strings have already been constructed.
pub fn accounted_packet_bytes(
    event: &DiagnosticEvent,
    key: Option<&DiagnosticCorrelation>,
) -> Option<usize> {
    fn string(value: &str) -> Option<usize> {
        8usize.checked_add(value.len())
    }
    fn fields(values: &[super::StructuredPayloadField]) -> Option<usize> {
        values.iter().try_fold(8usize, |sum, field| {
            sum.checked_add(string(field.name)?)?
                .checked_add(string(&field.value)?)
        })
    }
    let event_bytes = match event {
        DiagnosticEvent::Span {
            name, duration_us, ..
        } => string(name)?.checked_add(2 + if duration_us.is_some() { 8 } else { 0 })?,
        DiagnosticEvent::MessageSent { channel_id, .. }
        | DiagnosticEvent::MessageReceived { channel_id, .. } => {
            string(channel_id)?.checked_add(8)?
        },
        DiagnosticEvent::MessageSentStructured {
            channel_id,
            fields: values,
            ..
        }
        | DiagnosticEvent::MessageReceivedStructured {
            channel_id,
            fields: values,
            ..
        } => string(channel_id)?
            .checked_add(8)?
            .checked_add(fields(values)?)?,
        DiagnosticEvent::Event {
            target,
            level,
            message,
            fields: values,
        } => string(target)?
            .checked_add(string(level)?)?
            .checked_add(string(message)?)?
            .checked_add(fields(values)?)?,
    };
    // Event tag + sequence + receipt duration + correlation option tag.
    event_bytes
        .checked_add(22)?
        .checked_add(key.map_or(Some(0), DiagnosticCorrelation::accounted_bytes)?)
}

#[cfg(test)]
mod tests;
