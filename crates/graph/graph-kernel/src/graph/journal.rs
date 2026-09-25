// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! The edit spine: mere's captured-delta stream over the substrate log.
//!
//! The graph's durable mutations already exist as a stream of [`CapturedDelta`]s
//! (stable-id, serializable), emitted through the capture hook as each live
//! [`GraphDelta`](super::apply::GraphDelta) applies (see `capture.rs`). This module
//! gives that stream its principled home: a [`muniment::Journal`] of captured
//! deltas, the append-only log primitive shared across the Merely apps. The
//! materialized graph is the *replay* of the journal, and because live editing and
//! replay both funnel through `apply_graph_delta`, the two cannot diverge.
//!
//! This is the edit spine over the substrate. mere keeps its own rich edit
//! vocabulary — `CapturedDelta` carries content edits (title, tags, body,
//! navigation) that chartulary's topology-only `GraphEdit` deliberately cannot
//! express, and muniment supplies the ordered, forkable, persistable log beneath
//! it. Checkpointing reuses mere's existing `GraphSnapshot`: load a snapshot, then
//! [`replay_from`](GraphJournal::replay_from) the journal tail past its sequence,
//! mirroring `chartulary::GraphLog::load_checkpointed`.
//!
//! WASM-clean: muniment is the substrate's portable persistence primitive (the
//! same ones the browser persists through), so this module compiles to
//! `wasm32-unknown-unknown` like the rest of `graph/`.

use std::sync::{Arc, Mutex};

use muniment::{Backend, Codec, SlotStore, StoreError};
use muniment::{Journal, LogId, Provenance, Seq};

use rkyv::{Archive, Deserialize, Serialize};

use super::Graph;
use super::capture::{
    CapturedDelta, DeltaRecorder, replay_captured_deltas, replay_captured_deltas_onto,
};
use super::source_time::{SourceExtent, SourceTime};

/// The id the trusted UI's person records under when the host has no persona
/// to name: see [`Author::user`].
pub const USER_AUTHOR: &str = "user";

/// What kind of author made a change: a person, or a rule, script or engine
/// acting at some version (the ambiance design's moves between keeping levels
/// record the same four).
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum AuthorKind {
    Person,
    Rule,
    Script,
    Engine,
}

/// Who or what made a journal entry, and through which application.
///
/// Every field is always written, since a positional codec cannot read back a
/// field written conditionally.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    Hash,
    Archive,
    Serialize,
    Deserialize,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Author {
    pub kind: AuthorKind,
    /// A persona or subject id for a person; the rule's, script's or engine's
    /// name otherwise.
    pub id: String,
    /// The rule's, script's or engine's version. A person has none.
    pub version: Option<String>,
    /// The application the change came through. A resident fills this in
    /// from the admitted route rather than taking the client's word for it.
    pub via: Option<String>,
}

impl Author {
    /// A person, by persona or subject id.
    pub fn person(id: impl Into<String>) -> Self {
        Self::new(AuthorKind::Person, id, None)
    }

    /// The person at the trusted UI, when the host has no persona to name.
    pub fn user() -> Self {
        Self::person(USER_AUTHOR)
    }

    /// A rule at a version.
    pub fn rule(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(AuthorKind::Rule, id, Some(version.into()))
    }

    /// A script at a version.
    pub fn script(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(AuthorKind::Script, id, Some(version.into()))
    }

    /// An engine at a version.
    pub fn engine(id: impl Into<String>, version: impl Into<String>) -> Self {
        Self::new(AuthorKind::Engine, id, Some(version.into()))
    }

    fn new(kind: AuthorKind, id: impl Into<String>, version: Option<String>) -> Self {
        Self {
            kind,
            id: id.into(),
            version,
            via: None,
        }
    }

    /// The same author, arriving through `application`.
    pub fn via(mut self, application: impl Into<String>) -> Self {
        self.via = Some(application.into());
        self
    }
}

impl std::fmt::Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match self.kind {
            AuthorKind::Person => "person",
            AuthorKind::Rule => "rule",
            AuthorKind::Script => "script",
            AuthorKind::Engine => "engine",
        };
        write!(f, "{kind} {}", self.id)?;
        if let Some(version) = &self.version {
            write!(f, " {version}")?;
        }
        if let Some(via) = &self.via {
            write!(f, " via {via}")?;
        }
        Ok(())
    }
}

/// One journal entry: a captured delta in the attribution envelope — the
/// participant-gate plan's B1 adoption of chartulary's `Batch { author, edits }`
/// shape over mere's edit spine. WHO made a change rides the journal, so a
/// participant's edits read back attributed and compensable.
#[derive(
    Debug, Clone, PartialEq, Archive, Serialize, Deserialize, serde::Serialize, serde::Deserialize,
)]
pub struct AttributedDelta {
    pub author: Author,
    pub delta: CapturedDelta,
}

/// An append-only journal of a graph's captured edits: the persisted edit
/// spine. The materialized [`Graph`] is the replay of this log; forking it forks
/// the graph's whole history with provenance, and it persists whole through one
/// muniment slot.
#[derive(Clone, Debug)]
pub struct GraphJournal {
    log: Journal<AttributedDelta>,
    /// The author the next [`record`](Self::record) attributes: [`Author::user`]
    /// by default; a host scopes a participant run with
    /// [`set_author`](Self::set_author) and restores afterwards.
    author: Author,
}

impl Default for GraphJournal {
    fn default() -> Self {
        Self {
            log: Journal::default(),
            author: Author::user(),
        }
    }
}

impl GraphJournal {
    /// A fresh, empty journal, recording as [`Author::user`].
    pub fn new() -> Self {
        Self::default()
    }

    /// A journal with a stable identity, so it can later be forked with
    /// provenance pointing back at it.
    pub fn with_id(id: LogId) -> Self {
        Self {
            log: Journal::with_id(id),
            author: Author::user(),
        }
    }

    /// An empty journal that starts where `provenance` says another left off,
    /// without copying its entries: the keeper holds the graph they built as a
    /// snapshot. A session forked at a cursor starts this way.
    pub fn starting_from(id: LogId, provenance: Provenance) -> Self {
        Self::from_log(Journal::starting_from(id, provenance))
    }

    /// Adopt an existing journal of attributed deltas (e.g. one just loaded
    /// from a store or received from a peer).
    pub fn from_log(log: Journal<AttributedDelta>) -> Self {
        Self {
            log,
            author: Author::user(),
        }
    }

    /// The author subsequent [`record`](Self::record)s attribute.
    pub fn author(&self) -> &Author {
        &self.author
    }

    /// Scope the recording author (a participant run); the host restores the
    /// previous one when the run ends.
    pub fn set_author(&mut self, author: Author) {
        self.author = author;
    }

    /// Append one captured edit under the current author, returning the [`Seq`]
    /// it was stamped with. This is the write path: a live mutation's capture
    /// lands here (usually via [`journal_capture_hook`]).
    pub fn record(&mut self, delta: CapturedDelta) -> Seq {
        let author = self.author.clone();
        self.record_as(author, delta)
    }

    /// Append one captured edit under an explicit author (the gate path).
    pub fn record_as(&mut self, author: Author, delta: CapturedDelta) -> Seq {
        self.log.append(AttributedDelta { author, delta })
    }

    /// The underlying journal, for cursors, replication, and persistence.
    pub fn log(&self) -> &Journal<AttributedDelta> {
        &self.log
    }

    /// Every attributed edit, oldest first.
    pub fn entries(&self) -> &[AttributedDelta] {
        self.log.entries()
    }

    /// The number of edits recorded.
    pub fn len(&self) -> usize {
        self.log.len()
    }

    /// Whether the journal holds no edits.
    pub fn is_empty(&self) -> bool {
        self.log.is_empty()
    }

    /// The [`Seq`] the next recorded edit will receive — a durable cursor a
    /// checkpoint can store to resume replay from.
    pub fn next_seq(&self) -> Seq {
        self.log.next_seq()
    }

    /// The earliest source-time cursor: the empty graph before the first
    /// captured delta. Journal source time is a **prefix cursor**, so this is
    /// intentionally `Seq(0)` even though no entry is addressed there yet.
    pub const fn earliest_cursor(&self) -> Seq {
        Seq(0)
    }

    /// The current live source-time cursor: the prefix immediately after the
    /// newest captured delta. Appending later deltas never changes the graph a
    /// previously returned cursor replays to.
    pub fn live_cursor(&self) -> Seq {
        self.next_seq()
    }

    /// Materialize graph truth at a journal prefix cursor without changing the
    /// journal or its current live graph. `Seq(n)` replays exactly the first
    /// `n` entries; `Seq(0)` is empty and [`live_cursor`](Self::live_cursor)
    /// replays every current entry. A cursor beyond the known prefix is stale
    /// and is refused rather than silently clamped to live.
    pub fn snapshot_at(&self, cursor: Seq) -> Option<Graph> {
        let end = cursor.index();
        (end <= self.log.len()).then(|| {
            replay_captured_deltas(
                self.log.entries()[..end]
                    .iter()
                    .map(|entry| entry.delta.clone()),
            )
        })
    }

    /// Stable, evenly distributed prefix cursors for a compact scrubber. This
    /// uses sequence order alone: journal history has no required wall-clock
    /// timestamp. `max_points == 1` yields only the current live cursor;
    /// larger requests always include both earliest and live when distinct.
    pub fn sequence_ticks(&self, max_points: usize) -> Vec<Seq> {
        let live = self.live_cursor();
        if max_points == 0 {
            return Vec::new();
        }
        if max_points == 1 || live.0 == 0 {
            return vec![live];
        }
        let points = max_points.min(live.0 as usize + 1);
        let denominator = (points - 1) as u64;
        (0..points)
            .map(|index| Seq((index as u64 * live.0) / denominator))
            .collect()
    }

    /// This journal's log identity, if any.
    pub fn id(&self) -> Option<&LogId> {
        self.log.id()
    }

    /// Where this journal forked from, if it is a fork.
    pub fn provenance(&self) -> Option<&Provenance> {
        self.log.provenance()
    }

    /// Rebuild the whole graph by replaying every edit from empty (attribution
    /// rides the journal, not the graph — replay strips the envelope).
    pub fn replay(&self) -> Graph {
        replay_captured_deltas(self.log.entries().iter().map(|e| e.delta.clone()))
    }

    /// Advance an already-materialized `graph` by the edits from `since` onward.
    /// The checkpoint-plus-tail path: restore a `GraphSnapshot`, then apply only
    /// the journal entries recorded after the snapshot's sequence. The incremental
    /// twin of [`replay`](Self::replay).
    pub fn replay_from(&self, since: Seq, graph: &mut Graph) {
        replay_captured_deltas_onto(graph, self.log.from(since).iter().map(|e| e.delta.clone()));
    }

    /// Fork this journal under a new identity: copies the whole edit history and
    /// records where it branched from, then diverges independently. The log-level
    /// mirror of a graph fork; the fork replays to an identical graph and then
    /// edits without touching the source.
    pub fn fork(&self, new_id: LogId) -> Self {
        Self {
            log: self.log.fork(new_id),
            author: self.author.clone(),
        }
    }

    /// Write the whole journal to the muniment slot at `key`.
    pub async fn save<B: Backend, C: Codec>(
        &self,
        slots: &SlotStore<B, C>,
        key: &str,
    ) -> Result<(), StoreError> {
        self.log.save(slots, key).await
    }

    /// Load a journal from the muniment slot at `key`, or an empty journal if the
    /// slot is absent.
    pub async fn load<B: Backend, C: Codec>(
        slots: &SlotStore<B, C>,
        key: &str,
    ) -> Result<Self, StoreError> {
        Ok(Self::from_log(Journal::load(slots, key).await?))
    }
}

impl SourceTime for GraphJournal {
    type Cursor = Seq;
    type Snapshot = Graph;

    fn source_extent(&self) -> SourceExtent<Self::Cursor> {
        SourceExtent {
            earliest: self.earliest_cursor(),
            current: self.live_cursor(),
        }
    }

    fn source_ticks(&self, max_points: usize) -> Vec<Self::Cursor> {
        self.sequence_ticks(max_points)
    }

    fn source_snapshot(&self, cursor: &Self::Cursor) -> Option<Self::Snapshot> {
        self.snapshot_at(*cursor)
    }
}

/// A shared journal plus a capture hook that records every emitted
/// [`CapturedDelta`] into it. Install the hook with
/// [`set_captured_delta_hook`](super::set_captured_delta_hook) and keep the handle
/// to replay or persist the journal:
///
/// ```ignore
/// let (journal, hook) = journal_capture_hook();
/// kernel::graph::set_captured_delta_hook(Some(hook));
/// // ... drive live mutations through apply_graph_delta ...
/// let restored = journal.lock().unwrap().replay();
/// ```
///
/// The capture hook is one per-thread slot, so this replaces any previously
/// installed hook. It is offered as the host's journal-backed persistence path.
pub fn journal_capture_hook() -> (Arc<Mutex<GraphJournal>>, DeltaRecorder) {
    let journal = Arc::new(Mutex::new(GraphJournal::new()));
    let sink = Arc::clone(&journal);
    let hook: DeltaRecorder = Arc::new(move |delta| {
        if let Ok(mut journal) = sink.lock() {
            journal.record(delta.clone());
        }
    });
    (journal, hook)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::apply::{GraphDelta, GraphDeltaResult, apply_graph_delta};
    use crate::graph::set_captured_delta_hook;
    use crate::graph::{EdgeAssertion, SemanticSubKind, SourceExtent, SourceTime};
    use euclid::default::Point2D;
    use uuid::Uuid;

    fn add(id: u128, url: &str) -> CapturedDelta {
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: Uuid::from_u128(id).to_string(),
            url: url.to_string(),
            position: [0.0, 0.0],
        }
    }

    /// A key-independent view: sorted node ids and sorted (from, to, kind) triples.
    /// Two graphs are equal iff their fingerprints are, regardless of petgraph keys.
    fn fingerprint(graph: &Graph) -> (Vec<String>, Vec<(String, String, String)>) {
        let mut nodes: Vec<String> = graph.nodes().map(|(_, n)| n.id.to_string()).collect();
        nodes.sort();
        let mut edges: Vec<(String, String, String)> = graph
            .relations()
            .map(|rel| {
                let from = graph
                    .get_node(rel.from)
                    .map(|n| n.id.to_string())
                    .unwrap_or_default();
                let to = graph
                    .get_node(rel.to)
                    .map(|n| n.id.to_string())
                    .unwrap_or_default();
                (from, to, format!("{:?}", rel.kind))
            })
            .collect();
        edges.sort();
        (nodes, edges)
    }

    /// The envelope: entries carry their author; the default is the trusted
    /// UI's person, a scoped author attributes a participant run, and replay
    /// strips the envelope.
    #[test]
    fn entries_are_attributed_and_author_scoping_works() {
        let mut journal = GraphJournal::new();
        journal.record(add(1, "https://a.test/"));
        journal.set_author(Author::person("aa11"));
        journal.record(add(2, "https://b.test/"));
        journal.set_author(Author::user());
        journal.record_as(
            Author::engine("tarot-shuffle", "1.2").via("cleromancy"),
            add(3, "https://c.test/"),
        );

        let authors: Vec<String> = journal
            .entries()
            .iter()
            .map(|e| e.author.to_string())
            .collect();
        assert_eq!(
            authors,
            [
                "person user",
                "person aa11",
                "engine tarot-shuffle 1.2 via cleromancy"
            ]
        );
        assert_eq!(journal.author(), &Author::user(), "scoping restored");
        assert_eq!(
            journal.replay().node_count(),
            3,
            "replay strips the envelope"
        );
    }

    #[test]
    fn source_time_contract_selects_immutable_journal_prefixes() {
        let mut journal = GraphJournal::new();
        journal.record(add(1, "https://one.test/"));
        journal.record(add(2, "https://two.test/"));

        assert_eq!(
            journal.source_extent(),
            SourceExtent {
                earliest: Seq(0),
                current: Seq(2),
            }
        );
        assert_eq!(journal.source_ticks(3), [Seq(0), Seq(1), Seq(2)]);
        assert_eq!(
            journal
                .source_snapshot(&Seq(1))
                .expect("known source cursor")
                .node_count(),
            1
        );
        assert!(journal.source_snapshot(&Seq(3)).is_none());
        assert_eq!(
            journal.replay().node_count(),
            2,
            "scrubbing leaves live truth intact"
        );
    }

    /// Every author field is written, so the record reads back through any
    /// codec, positional ones included.
    #[test]
    fn an_author_writes_every_field() {
        let author = Author::rule("promote-on-tag", "3").via("turnstone");
        let json = serde_json::to_string(&author).unwrap();
        assert_eq!(
            json,
            r#"{"kind":"rule","id":"promote-on-tag","version":"3","via":"turnstone"}"#
        );
        assert_eq!(serde_json::from_str::<Author>(&json).unwrap(), author);
        let person = serde_json::to_string(&Author::user()).unwrap();
        assert_eq!(
            person,
            r#"{"kind":"person","id":"user","version":null,"via":null}"#
        );
    }

    /// A graph that opted in records its own deltas; a clone of it records
    /// nothing, and neither does a graph that never opted in.
    #[test]
    fn a_graph_records_its_own_deltas_and_a_clone_does_not() {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&seen);
        let mut session = Graph::new();
        session.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
            sink.lock().unwrap().push(delta.clone());
        })));
        assert!(session.is_recording());
        let add_node = |graph: &mut Graph, id: u128| {
            apply_graph_delta(
                graph,
                GraphDelta::AddNode {
                    id: Some(Uuid::from_u128(id)),
                    url: format!("https://{id}.test/"),
                    position: Point2D::new(0.0, 0.0),
                },
            )
        };
        add_node(&mut session, 1);
        let mut copy = session.clone();
        assert!(!copy.is_recording());
        add_node(&mut copy, 2);
        add_node(&mut Graph::new(), 3);
        assert_eq!(
            seen.lock().unwrap().as_slice(),
            &[add(1, "https://1.test/")]
        );
    }

    /// Replay records nothing: a scrub or a catch-up re-applies history, it
    /// does not make new history. The same run first proves both instruments
    /// see a live edit, so the silence after is the replay's.
    #[test]
    fn replay_records_nothing() {
        let (hooked, hook) = journal_capture_hook();
        set_captured_delta_hook(Some(hook));
        let seen = Arc::new(Mutex::new(0usize));
        let sink = Arc::clone(&seen);
        let mut session = Graph::new();
        session.set_recorder(Some(Arc::new(move |_: &CapturedDelta| {
            *sink.lock().unwrap() += 1;
        })));
        apply_graph_delta(
            &mut session,
            GraphDelta::AddNode {
                id: Some(Uuid::from_u128(1)),
                url: "https://a.test/".to_string(),
                position: Point2D::new(0.0, 0.0),
            },
        );
        assert_eq!(
            hooked.lock().unwrap().len(),
            1,
            "the thread hook sees live edits"
        );
        assert_eq!(
            *seen.lock().unwrap(),
            1,
            "the graph's recorder sees live edits"
        );

        let history = hooked.lock().unwrap().clone();
        let scrubbed = history.snapshot_at(history.live_cursor()).unwrap();
        replay_captured_deltas_onto(&mut session, [add(2, "https://b.test/")]);
        assert_eq!(scrubbed.node_count(), 1);
        assert_eq!(session.node_count(), 2);
        assert_eq!(
            hooked.lock().unwrap().len(),
            1,
            "replay reached the thread hook"
        );
        assert_eq!(
            *seen.lock().unwrap(),
            1,
            "replay reached the graph's recorder"
        );

        // Both instruments are back once the replay ends.
        assert!(session.is_recording());
        apply_graph_delta(
            &mut session,
            GraphDelta::AddNode {
                id: Some(Uuid::from_u128(3)),
                url: "https://c.test/".to_string(),
                position: Point2D::new(0.0, 0.0),
            },
        );
        assert_eq!(hooked.lock().unwrap().len(), 2);
        assert_eq!(*seen.lock().unwrap(), 2);
        set_captured_delta_hook(None);
    }

    #[test]
    fn records_and_replays_reconstruct_the_graph() {
        let mut journal = GraphJournal::new();
        journal.record(add(1, "https://a.test/"));
        journal.record(add(2, "https://b.test/"));
        journal.record(CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        });
        journal.record(CapturedDelta::ReplaySetNodeTitleById {
            node_id: Uuid::from_u128(1).to_string(),
            title: "Paper A".to_string(),
        });

        let graph = journal.replay();
        assert_eq!(graph.node_count(), 2);
        assert_eq!(graph.relations().count(), 1, "the cites relation replayed");
        let (_, node) = graph
            .get_node_by_id(Uuid::from_u128(1))
            .expect("node a present");
        assert_eq!(node.title, "Paper A", "the content edit replayed too");
    }

    #[test]
    fn prefix_cursors_replay_historical_graphs_without_touching_live_history() {
        let mut journal = GraphJournal::new();
        journal.record(add(1, "https://a.test/"));
        journal.record(add(2, "https://b.test/"));
        journal.record(CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(1).to_string(),
            to_id: Uuid::from_u128(2).to_string(),
            assertion: EdgeAssertion::Semantic {
                sub_kind: SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        });

        assert_eq!(journal.earliest_cursor(), Seq(0));
        assert_eq!(journal.live_cursor(), Seq(3));
        assert_eq!(journal.snapshot_at(Seq(0)).unwrap().node_count(), 0);
        assert_eq!(journal.snapshot_at(Seq(1)).unwrap().node_count(), 1);
        let historical = journal.snapshot_at(Seq(2)).expect("known prefix");
        assert_eq!(historical.node_count(), 2);
        assert_eq!(historical.relations().count(), 0);
        assert_eq!(journal.snapshot_at(Seq(3)).unwrap().relations().count(), 1);
        assert!(
            journal.snapshot_at(Seq(4)).is_none(),
            "stale cursors refuse"
        );
        assert_eq!(
            journal.replay().relations().count(),
            1,
            "live history remains live"
        );
        assert_eq!(journal.sequence_ticks(3), vec![Seq(0), Seq(1), Seq(3)]);
    }

    #[test]
    fn fork_carries_history_then_diverges() {
        let mut source = GraphJournal::with_id(LogId::new("source"));
        source.record(add(1, "https://a.test/"));
        source.record(add(2, "https://b.test/"));

        let mut fork = source.fork(LogId::new("fork"));
        assert_eq!(
            fork.entries(),
            source.entries(),
            "the fork copies the history"
        );
        let provenance = fork.provenance().expect("a fork has provenance");
        assert_eq!(provenance.source, Some(LogId::new("source")));
        assert_eq!(provenance.at, Seq(source.len() as u64));

        // Diverging the fork leaves the source untouched.
        fork.record(add(3, "https://c.test/"));
        assert_eq!(source.replay().node_count(), 2, "source unchanged");
        assert_eq!(fork.replay().node_count(), 3);
    }

    #[test]
    fn journal_round_trips_through_a_muniment_slot() {
        use muniment::{JsonSlots, MemoryBackend};
        pollster::block_on(async {
            let slots = JsonSlots::new(MemoryBackend::new());

            let mut journal = GraphJournal::new();
            journal.record(add(1, "https://a.test/"));
            journal.record(add(2, "https://b.test/"));
            journal.save(&slots, "journal").await.unwrap();

            let reloaded = GraphJournal::load(&slots, "journal").await.unwrap();
            assert_eq!(reloaded.len(), 2, "the log survived the round trip");
            assert_eq!(
                fingerprint(&reloaded.replay()),
                fingerprint(&journal.replay()),
                "the reloaded journal replays to the same graph"
            );
        });
    }

    #[test]
    fn the_capture_hook_feeds_the_journal_and_replay_matches_live() {
        // The edit-spine invariant: a graph built by live mutation and one rebuilt
        // by replaying the journal those mutations produced are identical.
        let (journal, hook) = journal_capture_hook();
        set_captured_delta_hook(Some(hook));

        let mut live = Graph::new();
        let a = match apply_graph_delta(
            &mut live,
            GraphDelta::AddNode {
                id: Some(Uuid::from_u128(1)),
                url: "https://a.test/".to_string(),
                position: Point2D::new(0.0, 0.0),
            },
        ) {
            GraphDeltaResult::NodeAdded(key) => key,
            other => panic!("expected NodeAdded, got {other:?}"),
        };
        let b = match apply_graph_delta(
            &mut live,
            GraphDelta::AddNode {
                id: Some(Uuid::from_u128(2)),
                url: "https://b.test/".to_string(),
                position: Point2D::new(1.0, 0.0),
            },
        ) {
            GraphDeltaResult::NodeAdded(key) => key,
            other => panic!("expected NodeAdded, got {other:?}"),
        };
        apply_graph_delta(
            &mut live,
            GraphDelta::AssertRelation {
                from: a,
                to: b,
                assertion: EdgeAssertion::Semantic {
                    sub_kind: SemanticSubKind::Cites,
                    label: None,
                    decay_progress: None,
                },
            },
        );
        apply_graph_delta(
            &mut live,
            GraphDelta::SetNodeTitle {
                key: a,
                title: "Paper A".to_string(),
            },
        );

        set_captured_delta_hook(None);

        let replayed = journal.lock().unwrap().replay();
        assert_eq!(
            fingerprint(&replayed),
            fingerprint(&live),
            "replay of the captured journal reconstructs the live graph"
        );
        let (_, node) = replayed.get_node_by_id(Uuid::from_u128(1)).expect("node a");
        assert_eq!(node.title, "Paper A");
    }
}
