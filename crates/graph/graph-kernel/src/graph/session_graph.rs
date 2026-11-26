// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The kernel graph as the graph a session holds (engine split D241, D249,
//! D251). Each method is the kernel call pandect made before the split, so a
//! session over the kernel graph does exactly what it did.

use mere::model::{Attributed, CoverageNote, ResourceStore};
use mere::session::{OnRefusal, Recorder, ReplayRefused, SessionGraph};
use portolan::FacetStore;
use uuid::Uuid;

use super::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
use super::capture::{
    CapturedDelta, ReplayAttribution, replay_attributed_deltas_onto,
    replay_captured_deltas_as_onto, replay_outcomes_onto, with_isolated_capture,
};
use super::revert::{Part, Touched, revert_change};
use super::{Author, Graph, PendingLinkRetention, PendingLinkState};
use crate::persistence::GraphSnapshot;

impl SessionGraph for Graph {
    type Delta = CapturedDelta;
    type Edit = GraphDelta;
    type Snapshot = GraphSnapshot;
    type Held = PendingLinkState;
    type Part = Part;
    type Carry = ReplayAttribution;

    fn to_snapshot(&self) -> GraphSnapshot {
        Graph::to_snapshot(self)
    }

    fn from_snapshot(snapshot: &GraphSnapshot, recorded: bool) -> Result<Self, String> {
        if recorded {
            Graph::try_from_recorded_snapshot(snapshot)
        } else {
            Graph::try_from_snapshot(snapshot)
        }
        .map_err(|error| error.to_string())
    }

    fn facets(&self) -> &FacetStore<Uuid> {
        Graph::facets(self)
    }

    fn install_facets(&mut self, facets: FacetStore<Uuid>) -> Result<(), String> {
        Graph::install_facets(self, facets).map_err(|error| error.to_string())
    }

    fn carry_from(baseline: &Self) -> ReplayAttribution {
        ReplayAttribution::from_baseline(baseline)
    }

    fn replay_carried(
        &mut self,
        carry: &mut ReplayAttribution,
        entries: &[Attributed<CapturedDelta>],
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused> {
        replay_attributed_deltas_onto(self, carry, entries, on_refusal)
    }

    fn repair_checkpoint(carry: &ReplayAttribution, graph: &mut Self) {
        carry.repair_checkpoint(graph);
    }

    fn replay_tail(&mut self, entries: &[Attributed<CapturedDelta>]) -> Result<(), ReplayRefused> {
        replay_outcomes_onto(
            self,
            entries
                .iter()
                .map(|entry| entry.delta.replay_as(&entry.author)),
            OnRefusal::Stop,
        )
    }

    fn replay_as(&mut self, author: &Author, deltas: &[CapturedDelta]) {
        replay_captured_deltas_as_onto(self, author, deltas.iter().cloned());
    }

    fn replay_matches(replay: &Self, candidate: &Self) -> Result<bool, String> {
        super::replay_state::replay_equal(replay, candidate)
            .map(|(equal, _)| equal)
            .map_err(|error| error.to_string())
    }

    fn edit_of(delta: &CapturedDelta, author: &Author) -> Result<GraphDelta, String> {
        delta
            .replay_as(author)
            .into_write()
            .map_err(|why| why.to_string())
    }

    fn apply(&mut self, edit: GraphDelta) -> Result<(), String> {
        match apply_graph_delta(self, edit) {
            GraphDeltaResult::Refused(reason) => Err(reason.to_string()),
            _ => Ok(()),
        }
    }

    fn set_recorder(&mut self, recorder: Option<Recorder<CapturedDelta>>) {
        Graph::set_recorder(self, recorder);
    }

    fn write_as<R>(&mut self, author: Author, write: impl FnOnce(&mut Self) -> R) -> R {
        with_isolated_capture(|| Graph::write_as(self, author, write))
    }

    fn revert(
        change: &[CapturedDelta],
        before: &Self,
        after: &Self,
        live: &Self,
    ) -> (Vec<CapturedDelta>, Vec<Part>) {
        let revert = revert_change(change, before, after, live);
        (revert.edits, revert.kept)
    }

    fn reaches(edits: &[CapturedDelta], part: &Part) -> bool {
        Touched::of(edits).reaches(part)
    }

    fn current_session(&self) -> u64 {
        Graph::current_session(self)
    }

    fn set_current_session(&mut self, launch: u64) {
        Graph::set_current_session(self, launch);
    }

    fn set_browsing_idle_gap_ms(&mut self, idle_gap_ms: u64) {
        Graph::set_browsing_idle_gap_ms(self, idle_gap_ms);
    }

    fn observation_revision(&self) -> u64 {
        self.semantic_observation_revision()
    }

    fn clear_semantic_context(&mut self) {
        Graph::clear_semantic_context(self);
    }

    fn copy_semantic_context_from(&mut self, other: &Self) {
        Graph::copy_semantic_context_from(self, other);
    }

    fn held(&self) -> PendingLinkState {
        self.retained_pending_link_state()
    }

    fn restore_held(&mut self, held: PendingLinkState) {
        self.restore_pending_link_state(held);
    }

    fn retained(mut held: PendingLinkState) -> PendingLinkState {
        if held.retention == PendingLinkRetention::SessionOnly {
            held.entries.clear();
        }
        held
    }

    fn known_coverage(&self) -> &CoverageNote {
        Graph::known_coverage(self)
    }

    fn set_known_coverage(&mut self, note: CoverageNote) {
        Graph::set_known_coverage(self, note);
    }

    fn resources(&self) -> &ResourceStore {
        &self.resources
    }

    fn validate_handles(&self) -> Result<(), String> {
        self.validate_active_resource_assertion_handles()
            .map_err(|error| error.to_string())
    }
}
