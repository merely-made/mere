// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! Bounded, value-free input snapshots for retained composition scheduling.
//!
//! This module records named owner revisions and per-component local
//! generations. It does not read providers, persist values, or schedule user
//! commands. A captured snapshot is immutable for one synchronous composition
//! rebuild; queue acknowledgements remove only the exact invalidation
//! generation that was captured.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

use crate::composition::ComponentKey;
use crate::reactive_diagnostics::{
    CauseRelation, DiagnosticDetails, DiagnosticKind, DiagnosticReason, Producer, ProducerError,
    ProducerRef,
};

const MAX_COMPONENTS: usize = 256;
const MAX_CAUSES: usize = 8;
const MAX_NAME_BYTES: usize = 64;
const MAX_PUBLISHED_CAUSES: usize = 256;

/// Upstream causal event reference. Values are numeric producer identity and
/// sequence only; authored keys and setting values are never diagnostics.
pub type CompositionCauseRef = ProducerRef;

/// Latest nonempty DOM mutation range caused by one component update.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MutationRecord {
    pub reference: ProducerRef,
    pub subject: crate::reactive_diagnostics::Subject,
    pub mutation_cursor_start: u64,
    pub mutation_cursor_end: u64,
    /// Current accepted visibility, updated on every retained reconciliation.
    pub visible: bool,
}

/// A committed revision of one named domain from one owner incarnation.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DependencyStamp {
    source_id: u64,
    owner_incarnation: u128,
    domain: String,
    revision: u64,
}

impl DependencyStamp {
    pub fn new(
        source_id: u64,
        owner_incarnation: u128,
        domain: impl Into<String>,
        revision: u64,
    ) -> Result<Self, CompositionInputError> {
        let domain = bounded_name(domain.into())?;
        Ok(Self {
            source_id,
            owner_incarnation,
            domain,
            revision,
        })
    }

    pub fn source_id(&self) -> u64 {
        self.source_id
    }

    pub fn owner_incarnation(&self) -> u128 {
        self.owner_incarnation
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn revision(&self) -> u64 {
        self.revision
    }
}

/// A named local editor generation for one component instance.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LocalInputStamp {
    name: String,
    generation: u64,
}

impl LocalInputStamp {
    pub fn new(name: impl Into<String>, generation: u64) -> Result<Self, CompositionInputError> {
        Ok(Self {
            name: bounded_name(name.into())?,
            generation,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
}

/// Sorted, immutable input identity for one live component.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct InputVector {
    dependencies: Vec<DependencyStamp>,
    local: Vec<LocalInputStamp>,
}

impl InputVector {
    pub fn new(
        mut dependencies: Vec<DependencyStamp>,
        mut local: Vec<LocalInputStamp>,
    ) -> Result<Self, CompositionInputError> {
        dependencies.sort();
        let mut dependency_axes = BTreeSet::new();
        for dependency in &dependencies {
            if !dependency_axes.insert((dependency.source_id, dependency.domain.as_str())) {
                return Err(CompositionInputError::DuplicateDependency);
            }
        }
        local.sort();
        if local.windows(2).any(|pair| pair[0].name == pair[1].name) {
            return Err(CompositionInputError::DuplicateLocalInput);
        }
        Ok(Self {
            dependencies,
            local,
        })
    }

    pub fn dependencies(&self) -> &[DependencyStamp] {
        &self.dependencies
    }

    pub fn local(&self) -> &[LocalInputStamp] {
        &self.local
    }
}

/// One complete input table, captured from one coherent product flush.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CompositionInputs {
    by_component: BTreeMap<ComponentKey, InputVector>,
}

impl CompositionInputs {
    pub fn new(
        by_component: BTreeMap<ComponentKey, InputVector>,
    ) -> Result<Self, CompositionInputError> {
        if by_component.len() > MAX_COMPONENTS {
            return Err(CompositionInputError::TooManyComponents);
        }
        Ok(Self { by_component })
    }

    pub fn vector(&self, key: &ComponentKey) -> Option<&InputVector> {
        self.by_component.get(key)
    }

    pub fn components(&self) -> impl Iterator<Item = (&ComponentKey, &InputVector)> {
        self.by_component.iter()
    }

    fn vector_or_empty(&self, key: &ComponentKey) -> InputVector {
        self.by_component.get(key).cloned().unwrap_or_default()
    }
}

/// Input errors are deliberately static and contain no source payload.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompositionInputError {
    InvalidName,
    DuplicateDependency,
    DuplicateLocalInput,
    TooManyComponents,
    TooManyCauses,
    CauseForUnknownComponent,
    GenerationExhausted,
}

#[derive(Clone, Debug)]
struct PendingInput {
    vector: InputVector,
    generation: u64,
    notices: u64,
    causes: Vec<CompositionCauseRef>,
    omitted_causes: u64,
    event_ref: Option<ProducerRef>,
}

struct QueueState {
    generation: u64,
    latest: CompositionInputs,
    pending: BTreeMap<ComponentKey, PendingInput>,
    latest_mutations: BTreeMap<ComponentKey, MutationRecord>,
    dropped_mutation_records: u64,
    generation_exhaustions: u64,
    producer: Producer,
}

/// Cloneable invalidation ingress. Repeated input publications merge by
/// component; commands do not enter this queue.
#[derive(Clone)]
pub struct CompositionInputQueue(Rc<RefCell<QueueState>>);

#[derive(Clone, Debug)]
struct CapturedInput {
    generation: u64,
    notices: u64,
    causes: Vec<CompositionCauseRef>,
    omitted_causes: u64,
    event_ref: Option<ProducerRef>,
}

/// Immutable input snapshot for one retained composition flush.
#[derive(Clone)]
pub struct CompositionInputSnapshot {
    inputs: CompositionInputs,
    captured: BTreeMap<ComponentKey, CapturedInput>,
    queue: Option<CompositionInputQueue>,
    producer: Option<Producer>,
    capture_ref: Option<ProducerRef>,
}

impl std::fmt::Debug for CompositionInputSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CompositionInputSnapshot")
            .field("component_count", &self.inputs.by_component.len())
            .field("pending_count", &self.captured.len())
            .finish()
    }
}

impl CompositionInputSnapshot {
    /// Create a filtering snapshot without a queue. This form has no pending
    /// invalidation acknowledgements or coalescing evidence.
    pub fn detached(inputs: CompositionInputs) -> Self {
        Self {
            inputs,
            captured: BTreeMap::new(),
            queue: None,
            producer: None,
            capture_ref: None,
        }
    }

    pub fn inputs(&self) -> &CompositionInputs {
        &self.inputs
    }

    pub(crate) fn vector_or_empty(&self, key: &ComponentKey) -> InputVector {
        self.inputs.vector_or_empty(key)
    }

    pub(crate) fn pending_notices(&self, key: &ComponentKey) -> Option<u64> {
        self.captured.get(key).map(|item| item.notices)
    }

    pub(crate) fn captured_generation(&self, key: &ComponentKey) -> Option<u64> {
        self.captured.get(key).map(|item| item.generation)
    }

    pub(crate) fn pending_causes(&self, key: &ComponentKey) -> &[CompositionCauseRef] {
        self.captured
            .get(key)
            .map_or(&[], |item| item.causes.as_slice())
    }

    pub(crate) fn omitted_causes(&self, key: &ComponentKey) -> u64 {
        self.captured.get(key).map_or(0, |item| item.omitted_causes)
    }

    pub(crate) fn acknowledge(&self, evaluated: &BTreeSet<ComponentKey>) {
        if let Some(queue) = &self.queue {
            queue.acknowledge(self, evaluated);
        }
    }

    pub(crate) fn producer(&self) -> Option<&Producer> {
        self.producer.as_ref()
    }

    pub(crate) fn capture_ref(&self) -> Option<ProducerRef> {
        self.capture_ref
    }

    pub(crate) fn emit_component(
        &self,
        kind: DiagnosticKind,
        subject: crate::reactive_diagnostics::Subject,
        details: DiagnosticDetails,
        key: &ComponentKey,
    ) -> Option<ProducerRef> {
        let Some(producer) = &self.producer else {
            return None;
        };
        let mut causes = Vec::new();
        if let Some(capture_ref) = self.capture_ref {
            causes.push((CauseRelation::CapturedFrom, capture_ref));
        }
        causes.extend(
            self.pending_causes(key)
                .iter()
                .copied()
                .map(|cause| (CauseRelation::InvalidatedBy, cause)),
        );
        producer
            .emit_with_causes(kind, Some(subject), details, &causes)
            .ok()
    }

    pub(crate) fn record_mutation(&self, key: ComponentKey, record: MutationRecord) {
        if let Some(queue) = &self.queue {
            queue.record_mutation(key, record);
        }
    }

    pub(crate) fn set_mutation_visibility(&self, key: &ComponentKey, visible: bool) {
        if let Some(queue) = &self.queue {
            queue.set_mutation_visibility(key, visible);
        }
    }

    pub(crate) fn emit_flush_completed(
        &self,
        _update_id: u64,
        _evaluated: usize,
        mutation_cursor_start: u64,
        mutation_cursor_end: u64,
        evaluated_causes: &[ProducerRef],
    ) -> Option<ProducerRef> {
        let Some(producer) = &self.producer else {
            return None;
        };
        let mut causes = self
            .capture_ref
            .map(|cause| vec![(CauseRelation::CapturedFrom, cause)])
            .unwrap_or_default();
        causes.extend(
            evaluated_causes
                .iter()
                .copied()
                .map(|cause| (CauseRelation::EvaluatedFrom, cause)),
        );
        let mut summary = details(None, None, DiagnosticReason::Accepted);
        summary.mutation_cursor_start = Some(mutation_cursor_start);
        summary.mutation_cursor_end = Some(mutation_cursor_end);
        producer
            .emit_with_causes(
                DiagnosticKind::CompositionFlushCompleted,
                None,
                summary,
                &causes,
            )
            .ok()
    }

    pub(crate) fn emit_mutation_batch(
        &self,
        flush_ref: ProducerRef,
        mutation_cursor_start: u64,
        mutation_cursor_end: u64,
    ) -> Option<ProducerRef> {
        if mutation_cursor_end <= mutation_cursor_start {
            return None;
        }
        let producer = self.producer.as_ref()?;
        let mut summary = details(None, None, DiagnosticReason::Accepted);
        summary.mutation_cursor_start = Some(mutation_cursor_start);
        summary.mutation_cursor_end = Some(mutation_cursor_end);
        producer
            .emit_with_causes(
                DiagnosticKind::MutationBatch,
                None,
                summary,
                &[(CauseRelation::ProducedBy, flush_ref)],
            )
            .ok()
    }
}

pub(crate) fn details(
    revision: Option<u64>,
    notice_count: Option<u64>,
    reason: DiagnosticReason,
) -> DiagnosticDetails {
    DiagnosticDetails {
        revision,
        notice_count,
        mutation_cursor_start: None,
        mutation_cursor_end: None,
        reason,
        ..DiagnosticDetails::default()
    }
}

fn bounded_name(value: String) -> Result<String, CompositionInputError> {
    if value.is_empty() || value.len() > MAX_NAME_BYTES {
        Err(CompositionInputError::InvalidName)
    } else {
        Ok(value)
    }
}

#[cfg(test)]
#[path = "composition_scheduler_tests.rs"]
mod tests;

#[path = "composition_scheduler/queue.rs"]
mod queue;
