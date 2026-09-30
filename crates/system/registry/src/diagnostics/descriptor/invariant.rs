// Copyright 2026 Mark Alan Boykin
// SPDX-License-Identifier: MPL-2.0

use super::DiagnosticsInvariant;
use crate::diagnostics::DiagnosticCorrelation;
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvariantLimits {
    pub max_pending: usize,
    pub max_bytes: usize,
    pub max_age: Duration,
}
impl Default for InvariantLimits {
    fn default() -> Self {
        Self {
            max_pending: 256,
            max_bytes: 65_536,
            max_age: Duration::from_secs(300),
        }
    }
}
impl InvariantLimits {
    fn enabled(self) -> bool {
        self.max_pending > 0 && self.max_bytes > 0 && !self.max_age.is_zero()
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InvariantTrackingLoss {
    pub disabled: u64,
    pub oversized: u64,
    pub full: u64,
    /// Retention age/config eviction is lost tracking, not an operation failure.
    pub expired: u64,
    pub evicted: u64,
    pub duplicate: u64,
    pub unmatched_terminal: u64,
    pub time_rejected: u64,
    pub deadline_overflow: u64,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvariantStats {
    pub pending: usize,
    pub accounted_bytes: usize,
    pub limits: InvariantLimits,
    pub loss: InvariantTrackingLoss,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CorrelatedInvariantViolation {
    pub invariant_id: String,
    pub start_channel: String,
    /// None means the explicitly uncorrelated legacy FIFO mode.
    pub correlation: Option<DiagnosticCorrelation>,
    pub deadline: Duration,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InvariantReport {
    pub violations: Vec<CorrelatedInvariantViolation>,
    pub stats: InvariantStats,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvariantTimeError;

struct Pending {
    invariant_id: String,
    start_channel: String,
    key: Option<DiagnosticCorrelation>,
    started: Duration,
    deadline: Duration,
    bytes: usize,
}
/// The keyed monotonic and legacy epoch clocks are independent domains. Bounds
/// apply to their combined pool. A terminal never crosses those domains.
pub(super) struct InvariantTracker {
    pending: Vec<Pending>,
    bytes: usize,
    limits: InvariantLimits,
    loss: InvariantTrackingLoss,
    last_keyed: Option<Duration>,
    last_legacy: Option<Duration>,
}
impl Default for InvariantTracker {
    fn default() -> Self {
        Self {
            pending: Vec::new(),
            bytes: 0,
            limits: InvariantLimits::default(),
            loss: InvariantTrackingLoss::default(),
            last_keyed: None,
            last_legacy: None,
        }
    }
}
fn increment(counter: &mut u64) {
    *counter = counter.saturating_add(1);
}

impl InvariantTracker {
    pub(super) fn stats(&self) -> InvariantStats {
        InvariantStats {
            pending: self.pending.len(),
            accounted_bytes: self.bytes,
            limits: self.limits,
            loss: self.loss,
        }
    }
    pub(super) fn set_limits(&mut self, limits: InvariantLimits) -> InvariantStats {
        self.limits = limits;
        // Changing policy releases records immediately, without mixing clocks.
        let mut index = 0;
        while index < self.pending.len() {
            let record = &self.pending[index];
            let now = if record.key.is_some() {
                self.last_keyed
            } else {
                self.last_legacy
            };
            if limits.enabled()
                && now.is_some_and(|now| now.saturating_sub(record.started) >= limits.max_age)
            {
                self.remove(index);
                increment(&mut self.loss.expired);
            } else {
                index += 1;
            }
        }
        while !limits.enabled()
            || self.pending.len() > limits.max_pending
            || self.bytes > limits.max_bytes
        {
            if self.pending.is_empty() {
                break;
            }
            self.remove(0);
            increment(&mut self.loss.evicted);
        }
        self.stats()
    }
    fn remove(&mut self, index: usize) -> Pending {
        let record = self.pending.remove(index);
        self.bytes -= record.bytes;
        record
    }
    fn validate_time(&mut self, keyed: bool, now: Duration) -> Result<(), InvariantTimeError> {
        let slot = if keyed {
            &mut self.last_keyed
        } else {
            &mut self.last_legacy
        };
        if slot.is_some_and(|last| now < last) {
            increment(&mut self.loss.time_rejected);
            return Err(InvariantTimeError);
        }
        *slot = Some(now);
        Ok(())
    }
    pub(super) fn sweep(
        &mut self,
        keyed: bool,
        now: Duration,
    ) -> Result<InvariantReport, InvariantTimeError> {
        self.validate_time(keyed, now)?;
        let mut violations = Vec::new();
        let mut index = 0;
        // Inspect every deadline. An earlier entry can have a later deadline.
        while index < self.pending.len() {
            let record = &self.pending[index];
            if record.key.is_some() != keyed {
                index += 1;
                continue;
            }
            let retention_end = record.started.checked_add(self.limits.max_age);
            if retention_end.is_some_and(|end| end < record.deadline && end <= now) {
                self.remove(index);
                increment(&mut self.loss.expired);
            } else if record.deadline <= now {
                let record = self.remove(index);
                violations.push(CorrelatedInvariantViolation {
                    invariant_id: record.invariant_id,
                    start_channel: record.start_channel,
                    correlation: record.key,
                    deadline: record.deadline,
                });
            } else {
                index += 1;
            }
        }
        Ok(InvariantReport {
            violations,
            stats: self.stats(),
        })
    }
    pub(super) fn observe<'a>(
        &mut self,
        rules: impl Iterator<Item = &'a DiagnosticsInvariant>,
        channel: &str,
        key: Option<&DiagnosticCorrelation>,
        now: Duration,
    ) -> Result<InvariantReport, InvariantTimeError> {
        // Deadline-before-terminal: a late completion cannot erase a timeout.
        let mut report = self.sweep(key.is_some(), now)?;
        for rule in rules.filter(|rule| rule.enabled) {
            if rule
                .terminal_channels
                .iter()
                .any(|terminal| terminal == channel)
            {
                if let Some(index) = self.pending.iter().position(|pending| {
                    pending.invariant_id == rule.invariant_id && pending.key.as_ref() == key
                }) {
                    self.remove(index);
                } else {
                    increment(&mut self.loss.unmatched_terminal);
                }
            }
            if rule.start_channel != channel {
                continue;
            }
            if !self.limits.enabled() {
                increment(&mut self.loss.disabled);
                continue;
            }
            // Check before cloning opaque ids: two u64-prefixed strings,
            // two durations (u64 seconds/u32 nanos), one key option tag and
            // the key's three u64-prefixed UTF-8 strings.
            let bytes = key
                .map_or(Some(0), DiagnosticCorrelation::accounted_bytes)
                .and_then(|bytes| bytes.checked_add(25 + 16))
                .and_then(|bytes| bytes.checked_add(rule.invariant_id.len()))
                .and_then(|bytes| bytes.checked_add(rule.start_channel.len()));
            let Some(bytes) = bytes.filter(|bytes| *bytes <= self.limits.max_bytes) else {
                increment(&mut self.loss.oversized);
                continue;
            };
            if key.is_some()
                && self.pending.iter().any(|pending| {
                    pending.invariant_id == rule.invariant_id && pending.key.as_ref() == key
                })
            {
                increment(&mut self.loss.duplicate);
                continue;
            }
            if self.pending.len() >= self.limits.max_pending
                || bytes > self.limits.max_bytes - self.bytes
            {
                increment(&mut self.loss.full);
                continue;
            }
            let Some(deadline) = now.checked_add(Duration::from_millis(rule.timeout_ms)) else {
                increment(&mut self.loss.deadline_overflow);
                continue;
            };
            self.bytes += bytes;
            self.pending.push(Pending {
                invariant_id: rule.invariant_id.clone(),
                start_channel: rule.start_channel.clone(),
                key: key.cloned(),
                started: now,
                deadline,
                bytes,
            });
        }
        report.stats = self.stats();
        Ok(report)
    }
}

#[cfg(test)]
mod tests;
