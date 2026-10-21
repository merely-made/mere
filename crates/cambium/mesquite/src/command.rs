// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Explicit script invocation and bounded, argument-free command receipts.

use crate::{Lane, Product};
use cambium::{EffectClass, Invocation, Outcome};
use serde::Serialize;
use std::collections::VecDeque;

/// Bounds retained command receipt metadata. The lane never copies argument
/// bytes. Each record keeps at most three UTF-8 fields of `max_field_bytes`
/// bytes (id, schema and refusal reason), plus fixed-size bookkeeping.
/// The product remains responsible for bounded decoding and execution.
#[derive(Clone, Copy, Debug, Serialize)]
pub struct CommandAuditLimits {
    pub max_records: usize,
    pub max_field_bytes: usize,
}

impl Default for CommandAuditLimits {
    fn default() -> Self {
        Self {
            max_records: 128,
            max_field_bytes: 256,
        }
    }
}

#[derive(Serialize)]
struct Record {
    sequence: u64,
    id: String,
    schema_id: String,
    effect: EffectClass,
    /// Null is explicitly pending, never a synthesized `Done`.
    outcome: Option<Outcome>,
    truncated: bool,
}

impl Record {
    fn truncate(&mut self, max_bytes: usize) {
        self.truncated |= truncate(&mut self.id, max_bytes);
        self.truncated |= truncate(&mut self.schema_id, max_bytes);
        if let Some(Outcome::Refused { reason }) = &mut self.outcome {
            self.truncated |= truncate(reason, max_bytes);
        }
    }
}

/// Records are observations at dispatch time. Pending outcomes are not polled
/// into completion, and an audit is neither durable history nor authorization.
#[derive(Serialize)]
pub(crate) struct CommandAudit {
    schema: &'static str,
    limits: CommandAuditLimits,
    dropped_records: u64,
    truncated_records: u64,
    records: VecDeque<Record>,
    #[serde(skip)]
    next_sequence: u64,
}

impl Default for CommandAudit {
    fn default() -> Self {
        Self {
            schema: "mesquite.commands/v1",
            limits: CommandAuditLimits::default(),
            dropped_records: 0,
            truncated_records: 0,
            records: VecDeque::new(),
            next_sequence: 0,
        }
    }
}

impl CommandAudit {
    fn set_limits(&mut self, limits: CommandAuditLimits) {
        self.limits = limits;
        for record in &mut self.records {
            let was_truncated = record.truncated;
            record.truncate(limits.max_field_bytes);
            if record.truncated && !was_truncated {
                self.truncated_records = self.truncated_records.saturating_add(1);
            }
        }
        self.trim();
    }

    fn trim(&mut self) {
        while self.records.len() > self.limits.max_records {
            self.records.pop_front();
            self.dropped_records = self.dropped_records.saturating_add(1);
        }
    }

    pub(crate) fn record(&mut self, invocation: &Invocation, outcome: Option<&Outcome>) {
        let record = Record {
            sequence: self.next_sequence,
            id: bounded(&invocation.id, self.limits.max_field_bytes),
            schema_id: bounded(&invocation.schema_id, self.limits.max_field_bytes),
            effect: invocation.effect,
            outcome: outcome.map(|outcome| match outcome {
                Outcome::Refused { reason } => Outcome::Refused {
                    reason: bounded(reason, self.limits.max_field_bytes),
                },
                outcome => outcome.clone(),
            }),
            truncated: invocation.id.len() > self.limits.max_field_bytes
                || invocation.schema_id.len() > self.limits.max_field_bytes
                || matches!(outcome, Some(Outcome::Refused { reason }) if reason.len() > self.limits.max_field_bytes),
        };
        // The bounded copies above never allocate in proportion to arguments
        // or an oversized refusal reason, and preserve UTF-8 boundaries.
        self.next_sequence = self.next_sequence.saturating_add(1);
        if record.truncated {
            self.truncated_records = self.truncated_records.saturating_add(1);
        }
        self.records.push_back(record);
        self.trim();
    }
}

impl<P: Product> Lane<P> {
    /// Request bounded command receipts, including an empty audit before the
    /// first invocation. Without this call, first invocation uses the defaults;
    /// lanes that never invoke keep their existing receipt shape. Tightening
    /// limits also bounds already retained records and reports resulting loss.
    pub fn set_command_audit_limits(&mut self, limits: CommandAuditLimits) {
        self.commands
            .get_or_insert_with(Default::default)
            .set_limits(limits);
    }
}

fn bounded(value: &str, max_bytes: usize) -> String {
    let mut end = value.len().min(max_bytes);
    while !value.is_char_boundary(end) {
        end -= 1;
    }
    value[..end].to_owned()
}

fn truncate(value: &mut String, max_bytes: usize) -> bool {
    if value.len() <= max_bytes {
        return false;
    }
    // Rebuild the bounded copy so tighter limits also release the old capacity.
    *value = bounded(value, max_bytes);
    true
}

pub(crate) fn parse(line: &str) -> Option<Result<Invocation, String>> {
    let (verb, payload) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
    (verb == "invoke").then(|| {
        serde_json::from_str(payload.trim()).map_err(|error| {
            // Serde's display can echo argument contents; only export location.
            format!(
                "invalid command invocation JSON at line {} column {}",
                error.line(),
                error.column()
            )
        })
    })
}

pub(crate) fn step_result(outcome: Option<&Outcome>) -> Result<(), String> {
    match outcome {
        Some(Outcome::Refused { reason }) => Err(format!("command refused: {reason}")),
        Some(Outcome::Stale) => Err("command target or observation is stale".into()),
        Some(Outcome::Done) | None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
