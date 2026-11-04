// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

use super::*;

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

    pub(super) fn acknowledge(
        &self,
        snapshot: &CompositionInputSnapshot,
        evaluated: &BTreeSet<ComponentKey>,
    ) {
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
