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

impl CompositionInputQueue {
    pub fn new() -> Result<Self, ProducerError> {
        let producer = Producer::new()?;
        Ok(Self(Rc::new(RefCell::new(QueueState {
            generation: 0,
            latest: CompositionInputs::default(),
            pending: BTreeMap::new(),
            latest_mutations: BTreeMap::new(),
            dropped_mutation_records: 0,
            generation_exhaustions: 0,
            producer,
        }))))
    }

    /// Publish a complete coherent input table. One pending record is kept per
    /// live component, with the latest vector and bounded causal references.
    pub fn publish(
        &self,
        inputs: CompositionInputs,
        cause: Option<CompositionCauseRef>,
    ) -> Result<(), CompositionInputError> {
        let causes = cause.map_or_else(BTreeMap::new, |cause| {
            inputs
                .by_component
                .keys()
                .cloned()
                .map(|key| (key, vec![cause]))
                .collect()
        });
        self.publish_with_causes(inputs, causes)
    }

    /// Publish one coherent full input table with per-component causal
    /// parents. The cause map must address live keys in this table.
    pub fn publish_with_causes(
        &self,
        inputs: CompositionInputs,
        causes: BTreeMap<ComponentKey, Vec<CompositionCauseRef>>,
    ) -> Result<(), CompositionInputError> {
        if causes
            .keys()
            .any(|key| !inputs.by_component.contains_key(key))
        {
            return Err(CompositionInputError::CauseForUnknownComponent);
        }
        if causes
            .values()
            .any(|causes| causes.len() > MAX_PUBLISHED_CAUSES)
        {
            return Err(CompositionInputError::TooManyCauses);
        }
        let mut state = self.0.borrow_mut();
        let Some(generation) = state.generation.checked_add(1) else {
            state.generation_exhaustions = state.generation_exhaustions.saturating_add(1);
            return Err(CompositionInputError::GenerationExhausted);
        };
        let previous = state.latest.clone();
        let live: BTreeSet<_> = inputs.by_component.keys().cloned().collect();
        state.pending.retain(|key, _| live.contains(key));
        state.latest_mutations.retain(|key, _| live.contains(key));
        let producer = state.producer.clone();

        for (key, vector) in &inputs.by_component {
            let changed = previous.vector(key) != Some(vector);
            let source_causes = causes.get(key).map_or(&[][..], Vec::as_slice);
            let invalidated = changed || !source_causes.is_empty();
            let Some(mut pending) = state.pending.remove(key) else {
                if invalidated {
                    let reason = if changed {
                        DiagnosticReason::DependencyChanged
                    } else {
                        DiagnosticReason::Coalesced
                    };
                    let parents = source_causes
                        .iter()
                        .copied()
                        .map(|cause| (CauseRelation::TriggeredBy, cause))
                        .collect::<Vec<_>>();
                    let event_ref = producer
                        .emit_with_causes(
                            DiagnosticKind::CompositionInputInvalidated,
                            None,
                            details(Some(generation), Some(1), reason),
                            &parents,
                        )
                        .ok();
                    let retained_causes = source_causes
                        .iter()
                        .take(MAX_CAUSES)
                        .copied()
                        .collect::<Vec<_>>();
                    state.pending.insert(
                        key.clone(),
                        PendingInput {
                            vector: vector.clone(),
                            generation,
                            notices: 1,
                            omitted_causes: source_causes
                                .len()
                                .saturating_sub(retained_causes.len())
                                as u64,
                            causes: retained_causes,
                            event_ref,
                        },
                    );
                }
                continue;
            };

            if invalidated {
                if changed {
                    pending.vector = vector.clone();
                }
                pending.generation = generation;
                pending.notices = pending.notices.saturating_add(1);
                let mut event_causes = Vec::new();
                for cause in source_causes {
                    if pending.causes.contains(cause) || event_causes.contains(cause) {
                        continue;
                    }
                    if pending.causes.len() < MAX_CAUSES {
                        pending.causes.push(*cause);
                    } else {
                        pending.omitted_causes = pending.omitted_causes.saturating_add(1);
                    }
                    event_causes.push(*cause);
                }
                let mut parents = pending
                    .event_ref
                    .map(|reference| vec![(CauseRelation::CoalescedFrom, reference)])
                    .unwrap_or_default();
                parents.extend(
                    event_causes
                        .iter()
                        .copied()
                        .map(|cause| (CauseRelation::TriggeredBy, cause)),
                );
                pending.event_ref = producer
                    .emit_with_causes(
                        DiagnosticKind::CompositionInputMerged,
                        None,
                        details(
                            Some(generation),
                            Some(pending.notices),
                            DiagnosticReason::Coalesced,
                        ),
                        &parents,
                    )
                    .ok();
            }
            state.pending.insert(key.clone(), pending);
        }
        state.generation = generation;
        state.latest = inputs;
        Ok(())
    }

    /// Capture all latest inputs and the exact pending generations to be
    /// acknowledged by a completed rebuild.
    pub fn capture(&self) -> CompositionInputSnapshot {
        let state = self.0.borrow();
        let captured: BTreeMap<ComponentKey, CapturedInput> = state
            .pending
            .iter()
            .map(|(key, pending)| {
                (
                    key.clone(),
                    CapturedInput {
                        generation: pending.generation,
                        notices: pending.notices,
                        causes: pending.causes.clone(),
                        omitted_causes: pending.omitted_causes,
                        event_ref: pending.event_ref,
                    },
                )
            })
            .collect();
        let mut cause_refs = captured
            .values()
            .filter_map(|item| item.event_ref)
            .collect::<Vec<_>>();
        cause_refs.sort_by_key(|cause| (cause.producer_id, cause.sequence));
        cause_refs.dedup();
        let parents = cause_refs
            .into_iter()
            .map(|cause| (CauseRelation::CapturedFrom, cause))
            .collect::<Vec<_>>();
        let capture_ref = state
            .producer
            .emit_with_causes(
                DiagnosticKind::CompositionInputCaptured,
                None,
                details(Some(state.generation), None, DiagnosticReason::Accepted),
                &parents,
            )
            .ok();
        CompositionInputSnapshot {
            inputs: state.latest.clone(),
            captured,
            queue: Some(self.clone()),
            producer: Some(state.producer.clone()),
            capture_ref,
        }
    }

    pub fn pending_components(&self) -> usize {
        self.0.borrow().pending.len()
    }

    pub fn owner_id(&self) -> u64 {
        self.0.borrow().producer.producer_id()
    }

    /// Nonconsuming snapshot of the latest nonempty DOM range for each live
    /// component. The queue is bounded by the composition component limit.
    pub fn latest_mutations(&self) -> Vec<MutationRecord> {
        self.0.borrow().latest_mutations.values().copied().collect()
    }

    pub fn dropped_mutation_records(&self) -> u64 {
        self.0.borrow().dropped_mutation_records
    }

    /// Count of refused queue publications after the checked generation
    /// counter reached its terminal value. This is a coverage gap, not a
    /// frozen runtime state: the queue continues accepting no further jobs.
    pub fn generation_exhaustions(&self) -> u64 {
        self.0.borrow().generation_exhaustions
    }

    pub(crate) fn record_mutation(&self, key: ComponentKey, record: MutationRecord) {
        if record.mutation_cursor_end <= record.mutation_cursor_start {
            return;
        }
        let mut state = self.0.borrow_mut();
        if !state.latest_mutations.contains_key(&key)
            && state.latest_mutations.len() >= MAX_COMPONENTS
        {
            if let Some(oldest_key) = state.latest_mutations.keys().next().cloned() {
                state.latest_mutations.remove(&oldest_key);
                state.dropped_mutation_records = state.dropped_mutation_records.saturating_add(1);
            }
        }
        state.latest_mutations.insert(key, record);
    }

    pub(crate) fn set_mutation_visibility(&self, key: &ComponentKey, visible: bool) {
        if let Some(record) = self.0.borrow_mut().latest_mutations.get_mut(key) {
            record.visible = visible;
        }
    }

    fn acknowledge(&self, snapshot: &CompositionInputSnapshot, evaluated: &BTreeSet<ComponentKey>) {
        let mut state = self.0.borrow_mut();
        for key in evaluated {
            let Some(captured) = snapshot.captured.get(key) else {
                continue;
            };
            if state
                .pending
                .get(key)
                .is_some_and(|pending| pending.generation == captured.generation)
            {
                state.pending.remove(key);
            }
        }
    }
}

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
