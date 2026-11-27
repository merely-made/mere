// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The graph a session holds, and its journal (engine split D241, D249, D251).
//!
//! A session keeps a graph, the journal that replays to it, the state it holds
//! beside the journal, and the resource store beneath it. [`SessionGraph`] is
//! what a session needs from its graph: snapshots and facets, replay with the
//! state a replay carries, edits and their recording, revert, the session
//! clock, held state with coverage, and the resource store. Graphshell's
//! incipit-graph implements it; so can any host that names its own surface
//! delta inside [`JournalDelta`](crate::model::JournalDelta).
//!
//! [`SessionJournal`] is the engine journal over any such graph: one muniment
//! log of [`Attributed`] deltas, one cursor and one digest per prefix.

use std::fmt;
use std::sync::Arc;

use portolan::FacetStore;
use serde::Serialize;
use serde::de::DeserializeOwned;
use uuid::Uuid;

use crate::model::{
    Attributed, Author, CoverageNote, PersistedResourceRecord, ResourceDelta, ResourceStore,
};
use crate::projection::RdfSource;

mod journal;
mod merge;
mod replay_point;

pub use journal::SessionJournal;
pub use merge::{MergeGraph, MergePart, MergeTouch};
pub use replay_point::ReplayPoint;

/// Where a graph that records sends each delta it applies.
pub type Recorder<D> = Arc<dyn Fn(&D) + Send + Sync + 'static>;

/// A replayed entry the graph refused: the journal and the graph it replays to
/// disagree from `position` (an index into the entries replayed) onward.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayRefused {
    pub position: usize,
    pub reason: crate::model::resource_rules::ContentError,
}

impl fmt::Display for ReplayRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "replayed entry {} refused: {}",
            self.position, self.reason
        )
    }
}

impl std::error::Error for ReplayRefused {}

/// What a replay does with an entry the graph refuses: stop there, so the
/// graph holds exactly the entries before it, or drop it and keep going.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OnRefusal {
    Stop,
    Drop,
}

/// The graph a session holds.
pub trait SessionGraph: RdfSource + Clone + Default + Sized + 'static {
    /// One journaled edit, in stable-id replay form.
    type Delta: Clone
        + fmt::Debug
        + PartialEq
        + Serialize
        + DeserializeOwned
        + Send
        + Sync
        + From<ResourceDelta>
        + 'static;
    /// A live edit, as [`apply`](Self::apply) takes it.
    type Edit;
    /// The graph as a session stores it.
    type Snapshot: Clone + fmt::Debug + Serialize + DeserializeOwned;
    /// What the session holds beside the journal, stored with it.
    type Held: Clone + fmt::Debug + Default + PartialEq + Serialize + DeserializeOwned;
    /// A part of the graph an undo can keep.
    type Part: Clone + fmt::Debug + PartialEq;
    /// What a replay from a baseline carries from entry to entry (D251).
    type Carry: Clone;

    // Snapshot and facets.

    fn to_snapshot(&self) -> Self::Snapshot;
    /// The graph a stored snapshot holds, by its recorded or unqualified rules.
    fn from_snapshot(snapshot: &Self::Snapshot, recorded: bool) -> Result<Self, String>;
    fn facets(&self) -> &FacetStore<Uuid>;
    /// Replace the facets whole, refusing ones that break the graph's rules.
    fn install_facets(&mut self, facets: FacetStore<Uuid>) -> Result<(), String>;

    // Replay.

    /// The carry a replay from `baseline` starts with.
    fn carry_from(baseline: &Self) -> Self::Carry;
    /// Replay `entries` onto this graph, carrying `carry`, quietly: nothing
    /// replayed is recorded.
    fn replay_carried(
        &mut self,
        carry: &mut Self::Carry,
        entries: &[Attributed<Self::Delta>],
        on_refusal: OnRefusal,
    ) -> Result<(), ReplayRefused>;
    /// Give a checkpoint graph what `carry` holds that its snapshot lost.
    fn repair_checkpoint(carry: &Self::Carry, graph: &mut Self);
    /// Replay a checkpoint tail with no baseline to carry from.
    fn replay_tail(&mut self, entries: &[Attributed<Self::Delta>]) -> Result<(), ReplayRefused>;
    /// Replay `deltas` as `author` quietly, dropping a refused one.
    fn replay_as(&mut self, author: &Author, deltas: &[Self::Delta]);
    /// Whether `replay` and `candidate` hold the same replay state.
    fn replay_matches(replay: &Self, candidate: &Self) -> Result<bool, String>;

    // Edits and their recording.

    /// The live edit `delta` replays as under `author`, or why it has none.
    fn edit_of(delta: &Self::Delta, author: &Author) -> Result<Self::Edit, String>;
    /// Apply one edit, or say why the graph refused it.
    fn apply(&mut self, edit: Self::Edit) -> Result<(), String>;
    fn set_recorder(&mut self, recorder: Option<Recorder<Self::Delta>>);
    /// Run `write` under `author`, recording only to this graph's recorder.
    fn write_as<R>(&mut self, author: Author, write: impl FnOnce(&mut Self) -> R) -> R;

    // Revert.

    /// The edits that take back `change` (made from `before` to `after`) on
    /// `live`, and the parts it keeps because they changed since.
    fn revert(
        change: &[Self::Delta],
        before: &Self,
        after: &Self,
        live: &Self,
    ) -> (Vec<Self::Delta>, Vec<Self::Part>);
    /// Whether `edits` reach `part`.
    fn reaches(edits: &[Self::Delta], part: &Self::Part) -> bool;

    // The session clock: host wiring, never journaled or compared.

    fn current_session(&self) -> u64;
    fn set_current_session(&mut self, launch: u64);
    fn set_browsing_idle_gap_ms(&mut self, idle_gap_ms: u64);
    /// Advances when what a host observes changes without a journal edit.
    fn observation_revision(&self) -> u64;
    fn clear_semantic_context(&mut self);
    fn copy_semantic_context_from(&mut self, other: &Self);

    // Held state and coverage.

    /// The held state as the session stores it.
    fn held(&self) -> Self::Held;
    fn restore_held(&mut self, held: Self::Held);
    /// `held` as it is retained: what a session keeps of it across opening.
    fn retained(held: Self::Held) -> Self::Held;
    fn known_coverage(&self) -> &CoverageNote;
    fn set_known_coverage(&mut self, note: CoverageNote);

    // The resource store.

    fn resources(&self) -> &ResourceStore;
    /// Refuse a graph whose active resource handles conflict.
    fn validate_handles(&self) -> Result<(), String>;

    /// Write or remove resource `id`'s record as one recorded edit under
    /// `author`, applied as its replay would apply it, or say why the graph
    /// refused it (engine split D257). Every host reaches it through its own
    /// [`edit_of`](Self::edit_of) and [`apply`](Self::apply).
    fn write_resource_record(
        &mut self,
        author: &Author,
        id: Uuid,
        record: Option<PersistedResourceRecord>,
    ) -> Result<(), String> {
        let delta = Self::Delta::from(ResourceDelta::ReplaySetResourceRecordById {
            resource_id: id.to_string(),
            record,
        });
        self.apply(Self::edit_of(&delta, author)?)
    }
}
