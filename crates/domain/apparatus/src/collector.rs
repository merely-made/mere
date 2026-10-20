// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Opt-in staging-to-history bridge. Products own installation, scheduling,
//! redaction, fresh run/queue identities and export policy. No raw event or
//! correlation string is copied unless the product explicitly projects it.

use crate::{
    Admission, Batch, Cursor, ObservationMetadata, ObservationStore, RetentionLimits, RunId,
    SourceBootId, SourceId, StoreError,
};
use registry::diagnostics::{
    DiagnosticPacket, DiagnosticReceiver, IngressAccessError, IngressStats,
};
use std::time::Duration;

/// Queue-local context, distinct from the retained envelope's run/source/sequence.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectedDiagnostic<P> {
    pub ingress_boot: SourceBootId,
    pub ingress_sequence: u64,
    pub ingress_receipt_time: Duration,
    pub payload: P,
}

/// A product's already-redacted projection. Byte accounting is for the chosen
/// payload encoding, excluding the two fixed queue-context fields added here.
pub struct DiagnosticProjection<P> {
    pub payload: P,
    pub encoded_bytes: usize,
    pub metadata: ObservationMetadata,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CollectError {
    Ingress(IngressAccessError),
    Store(StoreError),
}

/// Per-drain accounting. Filtering is product policy, not ingress loss.
#[derive(Clone, Debug)]
pub struct CollectionReport {
    pub ingress: IngressStats,
    pub drained: usize,
    pub filtered: usize,
    pub retained: usize,
    pub rejected: usize,
}

/// History and latest cumulative ingress counters are copied for every reader.
/// Counters are a drain-time snapshot, absent until the first successful drain.
/// They are not fabricated retained sequence gaps.
#[derive(Clone, Debug)]
pub struct CollectedBatch<P> {
    pub observations: Batch<CollectedDiagnostic<P>>,
    pub ingress_boot: SourceBootId,
    pub ingress: Option<IngressStats>,
}

pub struct RegistryCollector<P> {
    receiver: DiagnosticReceiver,
    store: ObservationStore<CollectedDiagnostic<P>>,
    boot: SourceBootId,
    ingress: Option<IngressStats>,
}

impl<P> RegistryCollector<P> {
    /// `boot` identifies this queue lifetime, including queue-local time/sequence.
    /// Choose bounded nonsecret identifiers; freshness is the installing owner's
    /// responsibility. Neither this constructor nor the bridge installs a sink.
    /// Construction never locks or drains a queue producers may already use.
    /// For a new queue lifetime construct a new collector with a fresh run and
    /// boot. An old cursor then reports a run boundary. Retain the old sender
    /// long enough to sample final disconnection losses before retiring it.
    pub fn new(
        receiver: DiagnosticReceiver,
        run: RunId,
        source: SourceId,
        boot: SourceBootId,
        limits: RetentionLimits,
    ) -> Self {
        Self {
            receiver,
            store: ObservationStore::new(run, source, limits),
            boot,
            ingress: None,
        }
    }

    pub fn cursor(&self) -> Cursor {
        self.store.cursor()
    }

    /// Validate the retained clock before consuming ingress. A product must
    /// bound/redact even fields it elects to put into metadata. Producer
    /// correlation is available to `project`; it is never reinterpreted as a
    /// retained cause reference or silently assigned to the collector's scope.
    pub fn collect(
        &mut self,
        now: Duration,
        max_records: usize,
        mut project: impl FnMut(&DiagnosticPacket) -> Option<DiagnosticProjection<P>>,
    ) -> Result<CollectionReport, CollectError> {
        self.store.expire(now).map_err(CollectError::Store)?;
        // Reserve sequence space before draining. Once the clock is checked,
        // record cannot fail partway through an already-consumed batch.
        let available = u64::MAX - self.store.stats().next_sequence;
        if max_records > 0 && available == 0 {
            return Err(CollectError::Store(StoreError::SequenceExhausted));
        }
        let max_records = max_records.min(usize::try_from(available).unwrap_or(usize::MAX));
        let batch = self
            .receiver
            .try_drain(max_records)
            .map_err(CollectError::Ingress)?;
        self.ingress = Some(batch.stats);
        let mut report = CollectionReport {
            ingress: batch.stats,
            drained: batch.packets.len(),
            filtered: 0,
            retained: 0,
            rejected: 0,
        };
        for packet in batch.packets {
            let Some(projected) = project(&packet) else {
                report.filtered += 1;
                continue;
            };
            // Queue context: sequence (8), Duration (12), boot length (8)
            // and UTF-8 boot bytes. This is admission accounting, not JSON size.
            // Overflow must reach the store as an accounting rejection, not a
            // wrapped small size. usize::MAX always fails envelope accounting.
            let bytes = projected
                .encoded_bytes
                .checked_add(28)
                .and_then(|n| n.checked_add(self.boot.0.len()))
                .unwrap_or(usize::MAX);
            let payload = CollectedDiagnostic {
                ingress_boot: self.boot.clone(),
                ingress_sequence: packet.sequence,
                ingress_receipt_time: packet.receipt_time,
                payload: projected.payload,
            };
            match self
                .store
                .record(payload, bytes, projected.metadata, now)
                .map_err(CollectError::Store)?
            {
                Admission::Retained(_) => report.retained += 1,
                Admission::Rejected { .. } => report.rejected += 1,
            }
        }
        Ok(report)
    }

    pub fn set_retention_limits(
        &mut self,
        limits: RetentionLimits,
        now: Duration,
    ) -> Result<(), StoreError> {
        self.store.set_limits(limits, now)
    }
}

impl<P: Clone> RegistryCollector<P> {
    pub fn read(
        &mut self,
        cursor: &mut Cursor,
        max_records: usize,
        now: Duration,
    ) -> Result<CollectedBatch<P>, StoreError> {
        Ok(CollectedBatch {
            observations: self.store.read(cursor, now, max_records)?,
            ingress_boot: self.boot.clone(),
            ingress: self.ingress,
        })
    }
}

#[cfg(test)]
mod tests;
