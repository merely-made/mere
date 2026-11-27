// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The surface as the graph a session holds.

use std::collections::BTreeSet;

use mere::model::{Attributed, Author, CoverageNote, JournalDelta, ResourceDelta, ResourceStore};
use mere::session::{OnRefusal, Recorder, ReplayRefused, SessionGraph};
use portolan::FacetStore;
use uuid::Uuid;

use super::replay::{attributed, check_records, edges_edit, remember};
use super::{Carry, Delta, Part, PartKey, PendingLinks, Surface, SurfaceDelta, SurfaceSnapshot};

impl SessionGraph for Surface {
    type Delta = Delta;
    type Edit = Delta;
    type Snapshot = SurfaceSnapshot;
    type Held = PendingLinks;
    type Part = Part;
    type Carry = Carry;

    fn to_snapshot(&self) -> SurfaceSnapshot {
        let mut resources: Vec<ResourceDelta> = self
            .resources
            .resource_nodes()
            .map(|resource| self.resources.record_delta(resource.id()))
            .collect();
        let pairs: BTreeSet<(Uuid, Uuid)> = self
            .resources
            .resource_edges()
            .map(|(from, to, _)| (from.id(), to.id()))
            .collect();
        resources.sort_by_key(|delta| serde_json::to_string(delta).expect("serializable"));
        resources.extend(
            pairs
                .into_iter()
                .map(|(a, b)| self.resources.pair_delta(a, b)),
        );
        SurfaceSnapshot {
            shown: self.shown.iter().map(|(s, r)| (*s, *r)).collect(),
            resources,
        }
    }

    fn from_snapshot(snapshot: &SurfaceSnapshot, _recorded: bool) -> Result<Self, String> {
        check_records(&snapshot.resources)?;
        let mut graph = Self::default();
        let deltas: Vec<Delta> = snapshot
            .resources
            .iter()
            .cloned()
            .map(JournalDelta::Resource)
            .chain(snapshot.shown.iter().map(|(surface_id, resource_id)| {
                JournalDelta::Surface(SurfaceDelta::ShowResource {
                    surface_id: *surface_id,
                    resource_id: *resource_id,
                })
            }))
            .collect();
        graph
            .replay_all(&deltas, OnRefusal::Stop)
            .map_err(|refused| refused.to_string())?;
        Ok(graph)
    }

    fn facets(&self) -> &FacetStore<Uuid> {
        &self.facets
    }

    fn install_facets(&mut self, facets: FacetStore<Uuid>) -> Result<(), String> {
        self.check_handles(&facets)
            .map_err(|why| format!("facets refused: {why}"))?;
        self.facets = facets;
        Ok(())
    }

    fn carry_from(baseline: &Self) -> Carry {
        let mut carry = Carry::new();
        for (from, to) in baseline.pairs() {
            let rows = baseline.resources.persisted_edges_between(from, to);
            remember(&mut carry, &edges_edit(from, to, rows));
        }
        carry
    }

    fn replay_carried(
        &mut self,
        carry: &mut Carry,
        entries: &[Attributed<Delta>],
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused> {
        let entries = entries
            .iter()
            .map(|entry| (&entry.delta, Some(&entry.author)));
        self.replay_attributed(carry, entries, on_refusal)
    }

    /// A checkpoint's snapshot holds every source it still shows; the carry
    /// rides beside it.
    fn repair_checkpoint(_carry: &Carry, _graph: &mut Self) {}

    /// With no baseline, only what the graph itself holds is carried.
    fn replay_tail(&mut self, entries: &[Attributed<Delta>]) -> Result<(), ReplayRefused> {
        let mut carry = Self::carry_from(self);
        self.replay_carried(&mut carry, entries, OnRefusal::Stop)
    }

    fn replay_as(&mut self, author: &Author, deltas: &[Delta]) {
        let mut carry = Self::carry_from(self);
        let entries = deltas.iter().map(|delta| (delta, Some(author)));
        let _ = self.replay_attributed(&mut carry, entries, OnRefusal::Drop);
    }

    fn replay_matches(replay: &Self, candidate: &Self) -> Result<bool, String> {
        Ok(replay.to_snapshot() == candidate.to_snapshot() && replay.facets == candidate.facets)
    }

    /// A live edit names its author as the source of a statement that names
    /// none, as its replay does.
    fn edit_of(delta: &Delta, author: &Author) -> Result<Delta, String> {
        if let JournalDelta::Surface(SurfaceDelta::Retired { note }) = delta {
            return Err(format!("a retired edit has no live form: {note}"));
        }
        Ok(attributed(delta, &Carry::new(), Some(author)))
    }

    fn apply(&mut self, edit: Delta) -> Result<(), String> {
        self.apply_delta(&edit)
            .map_err(|reason| reason.to_string())?;
        self.emit(edit);
        Ok(())
    }

    fn set_recorder(&mut self, recorder: Option<Recorder<Delta>>) {
        self.recorder = recorder;
    }

    fn write_as<R>(&mut self, _author: Author, write: impl FnOnce(&mut Self) -> R) -> R {
        write(self)
    }

    fn revert(
        change: &[Delta],
        before: &Self,
        after: &Self,
        live: &Self,
    ) -> (Vec<Delta>, Vec<Part>) {
        let (mut edits, mut kept) = (Vec::new(), Vec::new());
        for part in Self::touched(change) {
            if live.state_of(&part) == after.state_of(&part) {
                edits.push(before.restore(&part));
            } else {
                kept.push(part.part());
            }
        }
        (edits, kept)
    }

    fn reaches(edits: &[Delta], part: &Part) -> bool {
        Self::touched(edits).contains(&PartKey::of_part(part))
    }

    fn current_session(&self) -> u64 {
        self.launch
    }

    fn set_current_session(&mut self, launch: u64) {
        self.launch = launch;
    }

    fn set_browsing_idle_gap_ms(&mut self, _idle_gap_ms: u64) {}

    fn observation_revision(&self) -> u64 {
        self.observed
    }

    /// History starts from truth only: no pending links, no known coverage.
    fn clear_semantic_context(&mut self) {
        self.pending = PendingLinks::default();
        self.coverage = CoverageNote::default();
    }

    fn copy_semantic_context_from(&mut self, other: &Self) {
        self.pending = other.pending.clone();
        self.coverage = other.coverage.clone();
    }

    fn held(&self) -> PendingLinks {
        self.pending.clone().retained()
    }

    fn restore_held(&mut self, held: PendingLinks) {
        self.pending = held;
    }

    fn retained(held: PendingLinks) -> PendingLinks {
        held.retained()
    }

    fn known_coverage(&self) -> &CoverageNote {
        &self.coverage
    }

    fn set_known_coverage(&mut self, note: CoverageNote) {
        self.coverage = note;
    }

    fn resources(&self) -> &ResourceStore {
        &self.resources
    }

    fn validate_handles(&self) -> Result<(), String> {
        self.check_handles(&self.facets)
    }
}
