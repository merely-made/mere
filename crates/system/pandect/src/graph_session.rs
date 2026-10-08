// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's sessions: their schema over one muniment store, the live session
//! ([`GraphSession`]) and the lifecycle ([`MereSessions`]).
//!
//! Reservoir plan V2 (`design_docs/mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md`).
//! A mere is one muniment store, and each of its sessions lives at keys under
//! `sessions/<id>/`:
//!
//! ```text
//! manifest.json                    the session's manifest
//! baseline.json                    the graph its journal starts from (absent: empty)
//! graph.json, facets.json          the latest checkpoint's graph and node facets
//! checkpoint.json                  the journal cursor that checkpoint reflects
//! journal.jsonl/<seq>              one attributed graph edit per entry
//! changes.jsonl/<seq>              one change per entry: who made which edits, and how
//! views/<app>/<view>.json          one view's current state
//! views/<app>/<view>.jsonl/<seq>   that view's changes, each at a journal cursor
//! ```
//!
//! On muniment's directory backend the keys are Turnstone's files, and each
//! `.jsonl` log is one file with a line per entry; on redb or IndexedDB they are
//! rows. The journal is the authority: a session is its baseline plus its
//! journal, and a checkpoint only makes loading cheap. View changes run as their
//! own streams beside the journal, never merged into it (Alembic decision #5).

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use incipit::{GraphId, SessionId};
use kernel::graph::apply::{GraphDelta, apply_graph_delta};
#[cfg(not(target_arch = "wasm32"))]
use kernel::graph::node_facets::{PROVENANCE_DERIVATIONS, PROVENANCE_IMPORT};
use kernel::graph::{
    AttributedDelta, Author, CapturedDelta, Graph, GraphJournal, LogId, Part, Seq, Touched,
    revert_change,
};
use kernel::persistence::GraphSnapshot;
use kernel::time::wall_clock_now;
use muniment::{Backend, Journal, JsonCodec, JsonSlots, Provenance, StoreError, WriteOp};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

#[cfg(not(target_arch = "wasm32"))]
use crate::facet_store::{AcceptAll, FacetId, copy_node_facets};
use crate::facet_store::{NODE_FACETS_FILE, NodeFacetStore};
use crate::graph_placement::{
    FrozenGraph, LegacyTranslationReceipt, PlacementProfile, SessionPlacement, source_digest,
};
use crate::manifest::{GraphSessionManifest, TrashMark};
#[cfg(not(target_arch = "wasm32"))]
use crate::scene_facets::copy_scene_facets;
use crate::view_intent_store::ViewIntent;

/// Where a mere keeps its sessions.
pub const SESSIONS_PREFIX: &str = "sessions";
/// How many journal entries may accrue before the session writes a
/// checkpoint (reservoir plan §7 item 16).
pub const DEFAULT_CHECKPOINT_INTERVAL: u64 = 1_000;

/// Backend bound needed only when qualifying a legacy session transactionally.
/// Native transaction futures require `Sync`; browser backends keep their local bound.
#[cfg(not(target_arch = "wasm32"))]
pub trait LegacyTransactionBackend: Backend + Sync {}
#[cfg(not(target_arch = "wasm32"))]
impl<T: Backend + Sync + ?Sized> LegacyTransactionBackend for T {}
/// Browser counterpart of the native qualification backend bound.
#[cfg(target_arch = "wasm32")]
pub trait LegacyTransactionBackend: Backend {}
#[cfg(target_arch = "wasm32")]
impl<T: Backend + ?Sized> LegacyTransactionBackend for T {}

const MANIFEST: &str = "manifest.json";
const BASELINE: &str = "baseline.json";
const PLACEMENT: &str = "placement.json";
const TRANSLATION: &str = "legacy-translation.json";
const GRAPH: &str = "graph.json";
const CHECKPOINT: &str = "checkpoint.json";
const JOURNAL: &str = "journal.jsonl";
const CHANGES: &str = "changes.jsonl";
const VIEWS: &str = "views";

/// Why a session operation failed.
#[derive(Debug)]
pub enum SessionError {
    /// The mere holds no session with this id.
    Missing(SessionId),
    /// A stored document or log did not read back as what was written.
    Corrupt(String),
    /// An edit has no stable-id replay form, so it cannot be journaled.
    NotReplayable(String),
    /// A view key is not a portable name.
    InvalidViewKey(String),
    /// A fork's cursor lies past the parent's journal.
    CursorOutOfRange {
        cursor: Seq,
        live: Seq,
    },
    /// A component fork's seed names no node.
    NoSuchNode(Uuid),
    Store(StoreError),
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(id) => write!(f, "no session {} in this mere", id.as_uuid()),
            Self::Corrupt(what) => write!(f, "a stored session does not read back: {what}"),
            Self::NotReplayable(what) => write!(f, "an edit has no replay form: {what}"),
            Self::InvalidViewKey(what) => write!(f, "invalid view key: {what}"),
            Self::CursorOutOfRange { cursor, live } => {
                write!(f, "cursor {} lies past the journal's {}", cursor.0, live.0)
            },
            Self::NoSuchNode(id) => write!(f, "no node {id} to fork from"),
            Self::Store(error) => write!(f, "store: {error}"),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<StoreError> for SessionError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

/// One change to a session: who made it, what kind it was, and the journal
/// entries it wrote, `[first, end)`. A lifecycle change writes none.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Change {
    pub author: Author,
    pub kind: ChangeKind,
    pub first: Seq,
    pub end: Seq,
}

/// What a change did.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeKind {
    /// Edits made through [`GraphSession::apply`].
    Edit,
    /// Reverted change `of` (an index into the change log).
    Undo {
        of: Seq,
    },
    /// Re-applied what undo `of` reverted.
    Redo {
        of: Seq,
    },
    Minted,
    /// Forked from session `from` at its journal cursor `at`.
    Forked {
        from: SessionId,
        at: Seq,
    },
    Trashed,
    Restored,
}

/// One view of one application over a session: `views/<app>/<view>`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ViewKey {
    app: String,
    view: String,
}

impl ViewKey {
    /// A view key. Both parts become key segments, so each is 1 to 64 of
    /// `a-z`, `0-9`, `-`, `_` and `.`, not starting or ending with `.`.
    pub fn new(app: impl Into<String>, view: impl Into<String>) -> Result<Self, SessionError> {
        let (app, view) = (app.into(), view.into());
        for part in [&app, &view] {
            let portable = (1..=64).contains(&part.len())
                && !part.starts_with('.')
                && !part.ends_with('.')
                && part
                    .bytes()
                    .all(|byte| matches!(byte, b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.'));
            if !portable {
                return Err(SessionError::InvalidViewKey(format!("{app}/{view}")));
            }
        }
        Ok(Self { app, view })
    }

    pub fn app(&self) -> &str {
        &self.app
    }

    pub fn view(&self) -> &str {
        &self.view
    }
}

/// One entry of a view's stream: who changed the view, the journal cursor it
/// was changed at, and the state it changed to.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ViewEntry {
    pub author: Author,
    pub cursor: Seq,
    pub state: ViewIntent,
}

/// What [`GraphSession::apply`] wrote.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Applied {
    /// The journal entries the change wrote, `[first, end)`.
    pub first: Seq,
    pub end: Seq,
    /// The session's revision after the change.
    pub revision: u64,
}

/// What an undo or redo did: the change it reverted or re-applied, what it
/// wrote, and the parts it kept because another author changed them since.
#[derive(Clone, Debug, PartialEq)]
pub struct Reverted {
    /// The change undone, or the undo redone, as an index into the change log.
    pub of: Seq,
    pub applied: Applied,
    pub kept: Vec<Kept>,
}

/// A part an undo or redo left as it is, and who changed it since.
#[derive(Clone, Debug, PartialEq)]
pub struct Kept {
    pub part: Part,
    pub by: Vec<Author>,
}

#[derive(Serialize, Deserialize)]
struct Baseline {
    graph: GraphSnapshot,
    facets: NodeFacetStore,
}

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    cursor: Seq,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    translated_receipt_digest: Option<String>,
}

#[derive(Default)]
struct View {
    current: ViewIntent,
    log: Journal<ViewEntry>,
    saved: Seq,
}

/// Where a session's keys live.
struct Keys(String);

impl Keys {
    fn new(id: SessionId) -> Self {
        Self(format!("{SESSIONS_PREFIX}/{}", id.as_uuid()))
    }

    fn at(&self, name: &str) -> String {
        format!("{}/{name}", self.0)
    }

    fn view(&self, key: &ViewKey) -> String {
        format!("{}/{VIEWS}/{}/{}.json", self.0, key.app, key.view)
    }

    fn view_log(&self, key: &ViewKey) -> String {
        format!("{}/{VIEWS}/{}/{}.jsonl", self.0, key.app, key.view)
    }
}

fn pretty<T: Serialize>(key: String, value: &T) -> Result<WriteOp, SessionError> {
    let value =
        serde_json::to_vec_pretty(value).map_err(|error| StoreError::Codec(error.to_string()))?;
    Ok(WriteOp::Put { key, value })
}

async fn read<B: Backend, T: DeserializeOwned>(
    slots: &JsonSlots<B>,
    key: &str,
) -> Result<Option<T>, SessionError> {
    match slots.load(key).await {
        Err(StoreError::Codec(error)) => Err(SessionError::Corrupt(format!("{key}: {error}"))),
        loaded => Ok(loaded?),
    }
}

/// A stored graph and its facets, exactly as written. The facets replace
/// rather than overlay what the snapshot's columns import: a session stores
/// its whole facet store, and an overlay would add default-valued facets the
/// graph never held.
fn graph_of(
    snapshot: &GraphSnapshot,
    facets: NodeFacetStore,
) -> Result<Graph, kernel::graph::snapshot::ResourceSnapshotError> {
    let mut graph = Graph::try_from_snapshot(snapshot)?;
    *graph.facets_mut() = facets;
    Ok(graph)
}

fn baseline_of(graph: &Graph) -> Baseline {
    Baseline {
        graph: graph.to_snapshot(),
        facets: graph.facets().clone(),
    }
}

async fn retained_source_digest<B: Backend>(
    slots: &JsonSlots<B>,
    keys: &Keys,
    placement: &SessionPlacement,
) -> Result<String, SessionError> {
    let baseline = slots.backend().get(&keys.at(BASELINE)).await?;
    let mut entries = Vec::new();
    for index in 0..placement.cutoff().0 {
        let key = muniment::journal::entry_key(&keys.at(JOURNAL), Seq(index));
        entries.push(slots.backend().get(&key).await?.ok_or_else(|| {
            SessionError::Corrupt(format!("{key}: missing retained legacy entry"))
        })?);
    }
    source_digest(placement, baseline.as_deref(), &entries).map_err(SessionError::Corrupt)
}

#[derive(Clone)]
struct RetainedSource {
    baseline: Option<Vec<u8>>,
    entries: Vec<Vec<u8>>,
}

fn translation_transaction(
    keys: &Keys,
    source: &RetainedSource,
    ops: Vec<WriteOp>,
) -> (muniment::TransactFn, Arc<Mutex<Option<String>>>) {
    let baseline_key = keys.at(BASELINE);
    let journal_prefix = keys.at(JOURNAL);
    let profile_key = keys.at(PLACEMENT);
    let receipt_key = keys.at(TRANSLATION);
    let source = source.clone();
    let error = Arc::new(Mutex::new(None));
    let rejected = Arc::clone(&error);
    let transaction = Box::new(move |reader: &dyn muniment::TransactionReader| {
        let check = || -> Result<(), String> {
            if reader
                .get(&baseline_key)
                .map_err(|error| error.to_string())?
                != source.baseline
            {
                return Err("legacy baseline changed before qualification commit".into());
            }
            let keys = reader
                .list(&format!("{journal_prefix}/"))
                .map_err(|error| error.to_string())?;
            if keys.len() != source.entries.len() {
                return Err("legacy journal changed before qualification commit".into());
            }
            for (index, expected) in source.entries.iter().enumerate() {
                let key = muniment::journal::entry_key(&journal_prefix, Seq(index as u64));
                if reader
                    .get(&key)
                    .map_err(|error| error.to_string())?
                    .as_ref()
                    != Some(expected)
                {
                    return Err("retained journal entry changed before qualification commit".into());
                }
            }
            if reader
                .get(&profile_key)
                .map_err(|error| error.to_string())?
                .is_some()
                || reader
                    .get(&receipt_key)
                    .map_err(|error| error.to_string())?
                    .is_some()
            {
                return Err("session placement changed before qualification commit".into());
            }
            Ok(())
        };
        if let Err(reason) = check() {
            *rejected.lock().expect("translation rejection") = Some(reason);
            Vec::new()
        } else {
            ops
        }
    });
    (transaction, error)
}

impl RetainedSource {
    fn digest(&self, placement: &SessionPlacement) -> Result<String, SessionError> {
        let entries = self
            .entries
            .get(..placement.cutoff().index())
            .ok_or_else(|| {
                SessionError::Corrupt(
                    "legacy qualification requires reopening the durable prefix".into(),
                )
            })?;
        source_digest(placement, self.baseline.as_deref(), entries).map_err(SessionError::Corrupt)
    }
}

async fn load_retained_source<B: Backend>(
    slots: &JsonSlots<B>,
    keys: &Keys,
    manifest: &GraphSessionManifest,
) -> Result<(Graph, GraphJournal, RetainedSource), SessionError> {
    let baseline = slots.backend().get(&keys.at(BASELINE)).await?;
    let journal_prefix = keys.at(JOURNAL);
    let entry_keys = slots
        .backend()
        .scan(&format!("{journal_prefix}/"), &format!("{journal_prefix}0"))
        .await?;
    let mut entries = Vec::new();
    let mut writes = Vec::new();
    for key in entry_keys {
        let value = slots
            .backend()
            .get(&key)
            .await?
            .ok_or_else(|| SessionError::Corrupt(format!("{key}: vanished retained entry")))?;
        entries.push(value.clone());
        writes.push(WriteOp::Put { key, value });
    }
    let source = muniment::MemoryBackend::new();
    source.apply(&writes).await?;
    let journal = GraphJournal::from_log(
        Journal::<AttributedDelta>::load_entries(
            &JsonSlots::new(source),
            &journal_prefix,
            Some(LogId::new(manifest.session_id.as_uuid().to_string())),
            provenance(manifest),
        )
        .await?,
    );
    let graph = baseline
        .as_deref()
        .map(|bytes| {
            let baseline: Baseline = serde_json::from_slice(bytes).map_err(|error| {
                SessionError::Corrupt(format!("{}: {error}", keys.at(BASELINE)))
            })?;
            graph_of(&baseline.graph, baseline.facets)
                .map_err(|error| SessionError::Corrupt(format!("{}: {error}", keys.at(BASELINE))))
        })
        .transpose()?
        .unwrap_or_default();
    Ok((graph, journal, RetainedSource { baseline, entries }))
}

/// The provenance a session's journal was recorded with: a fork's starts at
/// its parent's cursor.
fn provenance(manifest: &GraphSessionManifest) -> Option<Provenance> {
    Some(Provenance {
        source: Some(LogId::new(manifest.parent_session?.as_uuid().to_string())),
        at: Seq(manifest.forked_at?),
    })
}

/// Route what `graph` records into `buffer`, for the session to journal.
fn record_into(graph: &mut Graph, buffer: &Arc<Mutex<Vec<CapturedDelta>>>) {
    let buffer = Arc::clone(buffer);
    graph.set_recorder(Some(Arc::new(move |delta: &CapturedDelta| {
        buffer.lock().expect("recorder buffer").push(delta.clone());
    })));
}

/// What a session has not stored yet, as one batch, and what storing it
/// covers. A host that writes through a batch of its own applies
/// [`ops`](Self::ops) there and hands this back to [`GraphSession::stored`]
/// once that batch commits.
#[derive(Debug)]
pub struct Pending {
    ops: Vec<WriteOp>,
    journal: Seq,
    changes: Seq,
    checkpoint: Option<Seq>,
    updated_at: Option<SystemTime>,
}

impl Pending {
    pub fn ops(&self) -> &[WriteOp] {
        &self.ops
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// One session, open: its graph, journal, changes and views, with every edit
/// recorded under its author. The async edits store as they are made; the
/// `_now` ones wait for [`flush`](Self::flush).
pub struct GraphSession<B> {
    slots: JsonSlots<B>,
    keys: Keys,
    manifest: GraphSessionManifest,
    baseline: Graph,
    placement: Option<SessionPlacement>,
    translation: Option<LegacyTranslationReceipt>,
    retained_source: Option<RetainedSource>,
    graph: Graph,
    journal: GraphJournal,
    pending: Arc<Mutex<Vec<CapturedDelta>>>,
    changes: Journal<Change>,
    views: BTreeMap<ViewKey, View>,
    saved: Seq,
    changes_saved: Seq,
    checkpointed: Seq,
    checkpoint_interval: u64,
    revision: u64,
    /// Whether the manifest and baseline are in the store yet.
    head_stored: bool,
    /// Whether the baseline is written with the head: a session begun from
    /// no graph stores none, and opens empty.
    write_baseline: bool,
}

impl<B: Backend> GraphSession<B> {
    /// Begin a new session in memory, from `baseline` or empty, with `kind` as
    /// its first change. Nothing is written until the first flush, which
    /// stores the manifest and baseline with everything since.
    pub fn new(
        backend: B,
        manifest: GraphSessionManifest,
        baseline: Option<Graph>,
        author: Author,
        kind: ChangeKind,
    ) -> Self {
        let id = LogId::new(manifest.session_id.as_uuid().to_string());
        let journal = match provenance(&manifest) {
            Some(provenance) => GraphJournal::starting_from(id, provenance),
            None => GraphJournal::with_id(id),
        };
        let mut changes = Journal::new();
        changes.append(Change {
            author,
            kind,
            first: Seq(0),
            end: Seq(0),
        });
        let write_baseline = baseline.is_some();
        let placement = (!write_baseline).then(SessionPlacement::recorded);
        let baseline = baseline.unwrap_or_default();
        let mut graph = baseline.clone();
        let pending = Arc::default();
        record_into(&mut graph, &pending);
        Self {
            slots: JsonSlots::new(backend),
            keys: Keys::new(manifest.session_id),
            manifest,
            baseline,
            placement,
            translation: None,
            retained_source: None,
            graph,
            journal,
            pending,
            changes,
            views: BTreeMap::new(),
            saved: Seq(0),
            changes_saved: Seq(0),
            checkpointed: Seq(0),
            checkpoint_interval: DEFAULT_CHECKPOINT_INTERVAL,
            revision: 1,
            head_stored: false,
            write_baseline,
        }
    }

    /// Begin from explicitly recorded store placement. An arbitrary graph supplied
    /// to `new` remains unqualified; qualification must come from its caller.
    pub fn new_recorded(
        backend: B,
        manifest: GraphSessionManifest,
        baseline: Option<Graph>,
        author: Author,
        kind: ChangeKind,
    ) -> Self {
        let mut session = Self::new(backend, manifest, baseline, author, kind);
        session.placement = Some(SessionPlacement::recorded());
        session
    }

    /// Open session `id` from a mere's store: the latest checkpoint and the
    /// journal past it, or the baseline and the whole journal.
    pub async fn open(backend: B, id: SessionId) -> Result<Self, SessionError> {
        let slots = JsonSlots::new(backend);
        let keys = Keys::new(id);
        let manifest: GraphSessionManifest = read(&slots, &keys.at(MANIFEST))
            .await?
            .ok_or(SessionError::Missing(id))?;
        let (baseline, journal, retained_source) =
            load_retained_source(&slots, &keys, &manifest).await?;
        let changes = Journal::load_entries(&slots, &keys.at(CHANGES), None, None).await?;
        let live = journal.live_cursor();
        let placement: Option<SessionPlacement> = read(&slots, &keys.at(PLACEMENT)).await?;
        if let Some(profile) = &placement {
            if profile.cutoff() > live {
                return Err(SessionError::Corrupt(format!(
                    "{}: legacy cursor lies past the journal",
                    keys.at(PLACEMENT)
                )));
            }
            if profile.baseline == PlacementProfile::RecordedStrataV1
                && (profile.cutoff() != Seq(0) || !profile.origins.is_empty())
            {
                return Err(SessionError::Corrupt(format!(
                    "{}: recorded placement has legacy history metadata",
                    keys.at(PLACEMENT)
                )));
            }
        }

        let translation: Option<LegacyTranslationReceipt> =
            read(&slots, &keys.at(TRANSLATION)).await?;
        match (&placement, &translation) {
            (Some(profile), Some(receipt))
                if profile.baseline == PlacementProfile::LegacySurfaceV1 =>
            {
                let digest = retained_source_digest(&slots, &keys, profile).await?;
                receipt
                    .validate(profile, &digest, journal.entries())
                    .map_err(|error| {
                        SessionError::Corrupt(format!("{}: {error}", keys.at(TRANSLATION)))
                    })?;
            },
            (Some(profile), None) if profile.baseline == PlacementProfile::LegacySurfaceV1 => {
                return Err(SessionError::Corrupt(format!(
                    "{}: missing legacy translation receipt",
                    keys.at(TRANSLATION)
                )));
            },
            (_, Some(_)) => {
                return Err(SessionError::Corrupt(format!(
                    "{}: receipt has no legacy placement",
                    keys.at(TRANSLATION)
                )));
            },
            _ => {},
        }

        let checkpoint: Option<Checkpoint> = read(&slots, &keys.at(CHECKPOINT)).await?;
        let (mut graph, checkpointed) = match checkpoint {
            Some(checkpoint)
                if checkpoint.cursor <= live
                    && translation.as_ref().is_none_or(|receipt| {
                        checkpoint.cursor >= receipt.placement.cutoff()
                            && checkpoint.translated_receipt_digest.as_deref()
                                == Some(receipt.translation_digest.as_str())
                    }) =>
            {
                let snapshot: GraphSnapshot = read(&slots, &keys.at(GRAPH))
                    .await?
                    .ok_or_else(|| SessionError::Corrupt(keys.at(GRAPH)))?;
                let facets: NodeFacetStore = read(&slots, &keys.at(NODE_FACETS_FILE))
                    .await?
                    .unwrap_or_default();
                let graph = if translation.is_some() {
                    FrozenGraph {
                        graph: snapshot,
                        facets,
                    }
                    .materialize()
                    .map_err(|error| {
                        SessionError::Corrupt(format!("{}: {error}", keys.at(GRAPH)))
                    })?
                } else {
                    graph_of(&snapshot, facets).map_err(|error| {
                        SessionError::Corrupt(format!("{}: {error}", keys.at(GRAPH)))
                    })?
                };
                (graph, checkpoint.cursor)
            },
            _ => (
                match &translation {
                    Some(receipt) => receipt
                        .baseline
                        .materialize()
                        .map_err(SessionError::Corrupt)?,
                    None => baseline.clone(),
                },
                Seq(0),
            ),
        };
        if let Some(receipt) = &translation {
            let translated = receipt
                .journal(journal.entries(), live)
                .expect("receipt validated against journal");
            let baseline = receipt
                .baseline
                .materialize()
                .map_err(SessionError::Corrupt)?;
            translated.replay_from_with_baseline(
                receipt.effect_cursor(checkpointed),
                &mut graph,
                &baseline,
            );
        } else {
            journal.replay_from_with_baseline(checkpointed, &mut graph, &baseline);
        }

        let mut session = Self {
            slots,
            keys,
            manifest,
            baseline,
            placement,
            translation,
            retained_source: Some(retained_source),
            graph,
            journal,
            pending: Arc::default(),
            changes_saved: changes.next_seq(),
            changes,
            views: BTreeMap::new(),
            saved: live,
            checkpointed,
            checkpoint_interval: DEFAULT_CHECKPOINT_INTERVAL,
            revision: 1,
            head_stored: true,
            write_baseline: false,
        };
        session.load_views().await?;
        record_into(&mut session.graph, &session.pending);
        Ok(session)
    }

    /// Qualify an already-durable legacy input and retain its original authority.
    pub async fn open_legacy(
        backend: B,
        id: SessionId,
        cutoff: Seq,
        origins: Vec<kernel::graph::legacy_resource_migration::LegacyResourceOriginResolution>,
    ) -> Result<Self, SessionError>
    where
        B: LegacyTransactionBackend,
    {
        let mut session = Self::open(backend, id).await?;
        session.qualify_legacy(cutoff, origins).await?;
        Ok(session)
    }

    /// Freeze a translation once. Its cutoff never grows with the modern journal.
    pub async fn qualify_legacy(
        &mut self,
        cutoff: Seq,
        origins: Vec<kernel::graph::legacy_resource_migration::LegacyResourceOriginResolution>,
    ) -> Result<(), SessionError>
    where
        B: LegacyTransactionBackend,
    {
        let placement = SessionPlacement::legacy(cutoff, origins);
        if let Some(existing) = &self.placement {
            if existing == &placement && self.translation.is_some() {
                return Ok(());
            }
            return Err(SessionError::Corrupt(
                "session already has a different placement contract".into(),
            ));
        }
        let live = self.journal.live_cursor();
        if cutoff > live {
            return Err(SessionError::CursorOutOfRange {
                cursor: cutoff,
                live,
            });
        }
        if !self.head_stored
            || self.saved != live
            || self.changes_saved != self.changes.next_seq()
            || !self.pending.lock().expect("recorder buffer").is_empty()
        {
            return Err(SessionError::Corrupt(
                "legacy qualification requires an already durable session".into(),
            ));
        }
        let digest = retained_source_digest(&self.slots, &self.keys, &placement).await?;
        let retained = self.retained_source.as_ref().ok_or_else(|| {
            SessionError::Corrupt(
                "legacy qualification requires reopening the durable session".into(),
            )
        })?;
        if retained.entries.len() != live.index() || retained.digest(&placement)? != digest {
            return Err(SessionError::Corrupt(
                "legacy source changed since this session was opened; reopen before qualification"
                    .into(),
            ));
        }
        let migrated =
            kernel::graph::legacy_resource_migration::migrate_legacy_prefix_with_resolutions(
                &self.baseline,
                &self.journal.entries()[..cutoff.index()],
                &placement.origins,
            )
            .map_err(|error| SessionError::Corrupt(error.to_string()))?;
        let mut baseline = self.baseline.clone();
        kernel::graph::replay_captured_deltas_onto(&mut baseline, migrated.baseline_effects);
        let receipt = LegacyTranslationReceipt::new(
            digest.clone(),
            placement.clone(),
            FrozenGraph::of(&baseline),
            migrated.entry_effects,
            FrozenGraph::of(&migrated.graph),
        )
        .map_err(SessionError::Corrupt)?;
        receipt
            .validate(&placement, &digest, self.journal.entries())
            .map_err(SessionError::Corrupt)?;
        let mut graph = receipt
            .graph_at(self.journal.entries(), live)
            .map_err(SessionError::Corrupt)?
            .expect("cutoff validated against live journal");
        let ops = [
            pretty(self.keys.at(PLACEMENT), &placement)?,
            pretty(self.keys.at(TRANSLATION), &receipt)?,
            pretty(self.keys.at(GRAPH), &graph.to_snapshot())?,
            pretty(self.keys.at(NODE_FACETS_FILE), graph.facets())?,
            pretty(
                self.keys.at(CHECKPOINT),
                &Checkpoint {
                    cursor: live,
                    translated_receipt_digest: Some(receipt.translation_digest.clone()),
                },
            )?,
        ];
        self.commit_translation(retained, ops.to_vec()).await?;
        record_into(&mut graph, &self.pending);
        self.graph = graph;
        self.placement = Some(placement);
        self.translation = Some(receipt);
        self.checkpointed = live;
        self.revision += 1;
        Ok(())
    }

    async fn commit_translation(
        &self,
        retained: &RetainedSource,
        ops: Vec<WriteOp>,
    ) -> Result<(), SessionError>
    where
        B: LegacyTransactionBackend,
    {
        let (transaction, rejected) = translation_transaction(&self.keys, retained, ops);
        self.slots.backend().transact(transaction).await?;
        let error = rejected.lock().expect("translation rejection").take();
        error.map_or(Ok(()), |reason| Err(SessionError::Corrupt(reason)))
    }

    async fn load_views(&mut self) -> Result<(), SessionError> {
        let prefix = format!("{}/{VIEWS}/", self.keys.0);
        for key in self.slots.keys(&prefix).await? {
            let Some(name) = key.strip_prefix(&prefix) else {
                continue;
            };
            let Some((app, file)) = name.split_once('/') else {
                continue;
            };
            let Some(view) = file.strip_suffix(".json") else {
                continue;
            };
            let view_key = ViewKey::new(app, view)?;
            let current = read(&self.slots, &key).await?.unwrap_or_default();
            let log =
                Journal::load_entries(&self.slots, &self.keys.view_log(&view_key), None, None)
                    .await?;
            let saved = log.next_seq();
            self.views.insert(
                view_key,
                View {
                    current,
                    log,
                    saved,
                },
            );
        }
        Ok(())
    }

    pub fn id(&self) -> SessionId {
        self.manifest.session_id
    }

    pub fn manifest(&self) -> &GraphSessionManifest {
        &self.manifest
    }

    /// Explicit baseline placement. Absence remains unqualified.
    pub fn placement(&self) -> Option<&SessionPlacement> {
        self.placement.as_ref()
    }

    /// The live graph. Edits go through [`apply`](Self::apply), so every one is
    /// journaled.
    pub fn graph(&self) -> &Graph {
        &self.graph
    }

    pub fn journal(&self) -> &GraphJournal {
        &self.journal
    }

    /// Every change, oldest first.
    pub fn changes(&self) -> &[Change] {
        self.changes.entries()
    }

    /// Advances with every change, so an attached application knows to look.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn set_checkpoint_interval(&mut self, entries: u64) {
        self.checkpoint_interval = entries.max(1);
    }

    /// Apply edits in stable-id form, as an application sends them.
    pub async fn apply(
        &mut self,
        author: Author,
        edits: Vec<CapturedDelta>,
    ) -> Result<Applied, SessionError> {
        let applied = self.apply_now(author, edits)?;
        self.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(applied)
    }

    /// Apply edits as one change under `author`, journal what they did, and
    /// store it before returning. A change that alters nothing records
    /// nothing.
    pub async fn apply_deltas(
        &mut self,
        author: Author,
        deltas: Vec<GraphDelta>,
    ) -> Result<Applied, SessionError> {
        self.apply_as(author, ChangeKind::Edit, deltas).await
    }

    /// [`apply`](Self::apply), journaled but not stored until the next flush:
    /// for a host whose edits arrive on a synchronous path.
    pub fn apply_now(
        &mut self,
        author: Author,
        edits: Vec<CapturedDelta>,
    ) -> Result<Applied, SessionError> {
        let deltas = edits
            .iter()
            .map(|edit| {
                edit.replay_delta_as(&author)
                    .ok_or_else(|| SessionError::NotReplayable(format!("{edit:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self
            .edit_as(author, ChangeKind::Edit, |graph| {
                for delta in deltas {
                    let _ = apply_graph_delta(graph, delta);
                }
            })
            .1)
    }

    /// Run `edit` on the live graph as one change under `author`, journaled
    /// but not stored until the next flush. Every mutation goes through
    /// `apply_graph_delta`, the kernel's one write path, so the journal sees
    /// all of them.
    pub fn edit_now<R>(
        &mut self,
        author: Author,
        edit: impl FnOnce(&mut Graph) -> R,
    ) -> (R, Applied) {
        self.edit_as(author, ChangeKind::Edit, edit)
    }

    fn edit_as<R>(
        &mut self,
        author: Author,
        kind: ChangeKind,
        edit: impl FnOnce(&mut Graph) -> R,
    ) -> (R, Applied) {
        let first = self.journal.live_cursor();
        let result = self.graph.write_as(author.clone(), edit);
        let recorded = std::mem::take(&mut *self.pending.lock().expect("recorder buffer"));
        for delta in recorded {
            self.journal.record_as(author.clone(), delta);
        }
        let end = self.journal.live_cursor();
        if end > first || kind != ChangeKind::Edit {
            self.changes.append(Change {
                author,
                kind,
                first,
                end,
            });
            self.revision += 1;
        }
        let applied = Applied {
            first,
            end,
            revision: self.revision,
        };
        (result, applied)
    }

    async fn apply_as(
        &mut self,
        author: Author,
        kind: ChangeKind,
        deltas: Vec<GraphDelta>,
    ) -> Result<Applied, SessionError> {
        let (_, applied) = self.edit_as(author, kind, |graph| {
            for delta in deltas {
                let _ = apply_graph_delta(graph, delta);
            }
        });
        self.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(applied)
    }

    /// Record a change that writes no edits (a lifecycle step) and store it,
    /// stamped `at`.
    async fn record(
        &mut self,
        author: Author,
        kind: ChangeKind,
        at: SystemTime,
    ) -> Result<(), SessionError> {
        let cursor = self.journal.live_cursor();
        self.changes.append(Change {
            author,
            kind,
            first: cursor,
            end: cursor,
        });
        self.revision += 1;
        self.store(at, Vec::new(), false).await
    }

    /// Everything not stored yet, as one batch: the manifest and baseline of a
    /// session begun in memory, new journal entries and changes, and a
    /// checkpoint once enough entries have accrued. New changes stamp the
    /// manifest `updated_at` with `at` (reservoir plan §7 item 28).
    pub fn pending(&self, at: SystemTime) -> Result<Pending, SessionError> {
        self.pending_with(at, false)
    }

    fn pending_with(&self, at: SystemTime, checkpoint: bool) -> Result<Pending, SessionError> {
        let journal = self.journal.live_cursor();
        let changes = self.changes.next_seq();
        let updated_at = (changes > self.changes_saved).then_some(at);
        let mut ops = Vec::new();
        if !self.head_stored || updated_at.is_some() {
            let mut manifest = self.manifest.clone();
            manifest.updated_at = updated_at.unwrap_or(manifest.updated_at);
            ops.push(pretty(self.keys.at(MANIFEST), &manifest)?);
        }
        if !self.head_stored && self.write_baseline {
            ops.push(pretty(
                self.keys.at(BASELINE),
                &baseline_of(&self.baseline),
            )?);
        }
        if !self.head_stored {
            if let Some(placement) = &self.placement {
                ops.push(pretty(self.keys.at(PLACEMENT), placement)?);
            }
        }
        ops.extend(
            self.journal
                .log()
                .entry_writes::<JsonCodec>(&self.keys.at(JOURNAL), self.saved)?,
        );
        ops.extend(
            self.changes
                .entry_writes::<JsonCodec>(&self.keys.at(CHANGES), self.changes_saved)?,
        );
        let checkpoint = (checkpoint
            || journal.0 - self.checkpointed.0 >= self.checkpoint_interval)
            .then_some(journal);
        if checkpoint.is_some() {
            ops.extend([
                pretty(self.keys.at(GRAPH), &self.graph.to_snapshot())?,
                pretty(self.keys.at(NODE_FACETS_FILE), self.graph.facets())?,
                pretty(
                    self.keys.at(CHECKPOINT),
                    &Checkpoint {
                        cursor: journal,
                        translated_receipt_digest: self
                            .translation
                            .as_ref()
                            .map(|receipt| receipt.translation_digest.clone()),
                    },
                )?,
            ]);
        }
        Ok(Pending {
            ops,
            journal,
            changes,
            checkpoint,
            updated_at,
        })
    }

    /// Mark a [`Pending`] batch as stored, once it has committed.
    pub fn stored(&mut self, pending: Pending) {
        self.head_stored = true;
        self.saved = self.saved.max(pending.journal);
        self.changes_saved = self.changes_saved.max(pending.changes);
        if let Some(cursor) = pending.checkpoint {
            self.checkpointed = self.checkpointed.max(cursor);
        }
        if let Some(at) = pending.updated_at {
            self.manifest.updated_at = at;
        }
    }

    /// Store everything not stored yet, stamping new changes `at`.
    pub async fn flush(&mut self, at: SystemTime) -> Result<(), SessionError> {
        self.store(at, Vec::new(), false).await
    }

    /// Store what is pending, with `extra`, as one batch.
    async fn store(
        &mut self,
        at: SystemTime,
        extra: Vec<WriteOp>,
        checkpoint: bool,
    ) -> Result<(), SessionError> {
        let mut pending = self.pending_with(at, checkpoint)?;
        pending.ops.extend(extra);
        if !pending.ops.is_empty() {
            self.slots.backend().apply(&pending.ops).await?;
        }
        self.stored(pending);
        Ok(())
    }

    /// Undo `author`'s latest change still in effect. Every part nobody has
    /// changed since goes back; the rest is kept and reported with who changed
    /// it (reservoir plan §7 item 17). `None` when there is nothing to undo.
    pub async fn undo(&mut self, author: Author) -> Result<Option<Reverted>, SessionError> {
        let Some(of) = self.stacks(&author).0.last().copied() else {
            return Ok(None);
        };
        self.revert(author, of, ChangeKind::Undo { of })
            .await
            .map(Some)
    }

    /// Redo `author`'s most recent undo, unless an edit of theirs since has
    /// cleared it (§7 item 18). `None` when there is nothing to redo.
    pub async fn redo(&mut self, author: Author) -> Result<Option<Reverted>, SessionError> {
        let Some(of) = self.stacks(&author).1.last().copied() else {
            return Ok(None);
        };
        self.revert(author, of, ChangeKind::Redo { of })
            .await
            .map(Some)
    }

    /// `author`'s undo and redo stacks, rebuilt from the change log: an edit
    /// goes on the undo stack and clears redo; an undo moves its change to
    /// redo; a redo moves it back.
    fn stacks(&self, author: &Author) -> (Vec<Seq>, Vec<Seq>) {
        let (mut undo, mut redo) = (Vec::new(), Vec::new());
        for (index, change) in self.changes.entries().iter().enumerate() {
            if &change.author != author {
                continue;
            }
            let index = Seq(index as u64);
            match &change.kind {
                ChangeKind::Edit => {
                    undo.push(index);
                    redo.clear();
                },
                ChangeKind::Undo { of } => {
                    undo.retain(|seq| seq != of);
                    redo.push(index);
                },
                ChangeKind::Redo { of } => {
                    redo.retain(|seq| seq != of);
                    undo.push(index);
                },
                _ => {},
            }
        }
        (undo, redo)
    }

    /// The edits change `of` wrote.
    fn edits_of(&self, change: &Change) -> Vec<CapturedDelta> {
        self.journal.entries()[change.first.index()..change.end.index()]
            .iter()
            .enumerate()
            .flat_map(|(offset, entry)| {
                self.translation
                    .as_ref()
                    .and_then(|receipt| receipt.entry_effects.get(change.first.index() + offset))
                    .cloned()
                    .unwrap_or_else(|| vec![entry.delta.clone()])
            })
            .collect()
    }

    /// Revert change `of` as `kind`, recording what was kept and by whom.
    async fn revert(
        &mut self,
        author: Author,
        of: Seq,
        kind: ChangeKind,
    ) -> Result<Reverted, SessionError> {
        let change = self
            .changes
            .get(of)
            .cloned()
            .expect("a stacked change exists");
        let (before, after) = (
            self.graph_at(change.first)
                .expect("a change lies within its journal"),
            self.graph_at(change.end)
                .expect("a change lies within its journal"),
        );
        let revert = revert_change(&self.edits_of(&change), &before, &after, &self.graph);
        let kept = revert
            .kept
            .into_iter()
            .map(|part| Kept {
                by: self.changed_since(of, &part),
                part,
            })
            .collect();
        let deltas = revert
            .edits
            .iter()
            .map(|edit| {
                edit.replay_delta_as(&author)
                    .ok_or_else(|| SessionError::NotReplayable(format!("{edit:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let applied = self.apply_as(author, kind, deltas).await?;
        Ok(Reverted { of, applied, kept })
    }

    /// Who, after change `of`, made a change that reached `part`.
    fn changed_since(&self, of: Seq, part: &Part) -> Vec<Author> {
        let mut by: Vec<Author> = Vec::new();
        for change in &self.changes.entries()[of.index() + 1..] {
            let edits = self.edits_of(change);
            if Touched::of(&edits).reaches(part) && !by.contains(&change.author) {
                by.push(change.author.clone());
            }
        }
        by
    }

    /// Write a checkpoint: the graph, its facets and the cursor they reflect,
    /// with any unsaved entries, in one batch.
    pub async fn checkpoint(&mut self) -> Result<(), SessionError> {
        self.store(wall_clock_now(), Vec::new(), true).await
    }

    /// Checkpoint if the journal has moved since the last one: what the
    /// resident calls when the last application detaches.
    pub async fn checkpoint_if_behind(&mut self) -> Result<(), SessionError> {
        if self.checkpointed < self.journal.live_cursor() {
            self.checkpoint().await?;
        }
        Ok(())
    }

    /// The graph as it stood at journal cursor `cursor`: the baseline with the
    /// first `cursor` entries replayed. `None` past the live cursor.
    pub fn graph_at(&self, cursor: Seq) -> Option<Graph> {
        match &self.translation {
            Some(receipt) => receipt
                .graph_at(self.journal.entries(), cursor)
                .ok()
                .flatten(),
            None => self.journal.snapshot_at_from(&self.baseline, cursor),
        }
    }

    /// Every view with state, in key order.
    pub fn views(&self) -> impl Iterator<Item = (&ViewKey, &ViewIntent)> {
        self.views.iter().map(|(key, view)| (key, &view.current))
    }

    /// A view's current state.
    pub fn view(&self, key: &ViewKey) -> Option<&ViewIntent> {
        self.views.get(key).map(|view| &view.current)
    }

    /// A view's state as it stood at journal cursor `cursor`: its latest
    /// change made at or before that cursor.
    pub fn view_at(&self, key: &ViewKey, cursor: Seq) -> Option<&ViewIntent> {
        self.views
            .get(key)?
            .log
            .entries()
            .iter()
            .rev()
            .find(|entry| entry.cursor <= cursor)
            .map(|entry| &entry.state)
    }

    /// Change a view's state under `author`, stamped with the journal cursor it
    /// was made at, and store it with the view's current state and whatever
    /// the session has pending, so no stored view names an unstored cursor.
    pub async fn set_view(
        &mut self,
        author: Author,
        key: ViewKey,
        state: ViewIntent,
    ) -> Result<(), SessionError> {
        let cursor = self.journal.live_cursor();
        let mut pending = self.pending(wall_clock_now())?;
        let current = self.keys.view(&key);
        let log_key = self.keys.view_log(&key);
        let view = self.views.entry(key.clone()).or_default();
        view.log.append(ViewEntry {
            author,
            cursor,
            state: state.clone(),
        });
        pending.ops.push(pretty(current, &state)?);
        pending
            .ops
            .extend(view.log.entry_writes::<JsonCodec>(&log_key, view.saved)?);
        self.slots.backend().apply(&pending.ops).await?;
        self.stored(pending);
        let view = self.views.get_mut(&key).expect("the view was just written");
        view.current = state;
        view.saved = view.log.next_seq();
        Ok(())
    }

    /// Put the session in the trash: its manifest is marked, its keys stay.
    pub async fn trash(&mut self, author: Author, at: SystemTime) -> Result<(), SessionError> {
        self.manifest.trashed = Some(TrashMark {
            by: author.clone(),
            at,
        });
        self.record(author, ChangeKind::Trashed, at).await
    }

    /// Take the session back out of the trash.
    pub async fn restore(&mut self, author: Author, at: SystemTime) -> Result<(), SessionError> {
        self.manifest.trashed = None;
        self.record(author, ChangeKind::Restored, at).await
    }
}

/// The graph a component fork starts from: the connected component around
/// `seed`, copied with its facets through the id remap, and the donor
/// container's scene settings carried to the fork's container. The
/// product-neutral half of Turnstone's tear-out; nested worlds and resident
/// admissions stay Turnstone's to carry. Native only, as the kernel's
/// component copy is.
#[cfg(not(target_arch = "wasm32"))]
pub fn fork_component_graph(
    donor: &Graph,
    donor_container: Uuid,
    seed: Uuid,
    fork_container: Uuid,
) -> Option<Graph> {
    let mut fork = Graph::new();
    let copy = fork.copy_component_from(donor, seed, Some(donor_container.to_string()));
    if copy.new_keys.is_empty() {
        return None;
    }
    let mut facets = NodeFacetStore::new();
    copy_node_facets(donor.facets(), &mut facets, &copy.id_remap);
    copy_scene_facets(donor.facets(), &mut facets, donor_container, fork_container);
    // The copy stamped each minted node's derivation from its donor; the
    // overlay below would drop it, so keep it aside and set it again.
    let derivation = FacetId::new(PROVENANCE_DERIVATIONS);
    let derivations: Vec<(Uuid, serde_json::Value)> = copy
        .id_remap
        .iter()
        .filter_map(|(_, minted)| {
            fork.facets()
                .get(minted, &derivation)
                .cloned()
                .map(|value| (*minted, value))
        })
        .collect();
    fork.overlay_facets(facets);
    let import = FacetId::new(PROVENANCE_IMPORT);
    for (_, minted) in &copy.id_remap {
        fork.facets_mut().remove(minted, &import);
    }
    for (minted, value) in derivations {
        fork.facets_mut()
            .set(minted, derivation.clone(), value, &AcceptAll)
            .expect("AcceptAll admits every value");
    }
    Some(fork)
}

/// A mere's sessions, over its one store.
pub struct MereSessions<B> {
    backend: B,
}

impl<B: Backend + Clone> MereSessions<B> {
    pub fn new(backend: B) -> Self {
        Self { backend }
    }

    fn slots(&self) -> JsonSlots<B> {
        JsonSlots::new(self.backend.clone())
    }

    /// Every session's manifest, trashed ones included, oldest first.
    pub async fn list(&self) -> Result<Vec<GraphSessionManifest>, SessionError> {
        let slots = self.slots();
        let mut manifests = Vec::new();
        for key in slots.keys(&format!("{SESSIONS_PREFIX}/")).await? {
            if key.ends_with(&format!("/{MANIFEST}"))
                && let Some(manifest) = read::<B, GraphSessionManifest>(&slots, &key).await?
            {
                manifests.push(manifest);
            }
        }
        manifests.sort_by_key(|manifest| (manifest.created_at, *manifest.session_id.as_uuid()));
        Ok(manifests)
    }

    /// The live session changed last: untrashed, latest by `updated_at`, then
    /// by minting (reservoir plan §7 items 24 and 28). `None` when there is none.
    pub async fn latest_live(&self) -> Result<Option<GraphSessionManifest>, SessionError> {
        Ok(self
            .list()
            .await?
            .into_iter()
            .filter(|manifest| manifest.trashed.is_none())
            .max_by_key(|manifest| {
                (
                    manifest.updated_at,
                    manifest.created_at,
                    *manifest.session_id.as_uuid(),
                )
            }))
    }

    /// Open a session.
    pub async fn open(&self, id: SessionId) -> Result<GraphSession<B>, SessionError> {
        GraphSession::open(self.backend.clone(), id).await
    }

    pub async fn open_legacy(
        &self,
        id: SessionId,
        cutoff: Seq,
        origins: Vec<kernel::graph::legacy_resource_migration::LegacyResourceOriginResolution>,
    ) -> Result<GraphSession<B>, SessionError>
    where
        B: LegacyTransactionBackend,
    {
        GraphSession::open_legacy(self.backend.clone(), id, cutoff, origins).await
    }

    /// Begin a new session in memory, from `baseline` or empty. Its first
    /// flush stores it.
    pub fn begin(&self, author: Author, baseline: Option<Graph>) -> GraphSession<B> {
        let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        GraphSession::new(
            self.backend.clone(),
            manifest,
            baseline,
            author,
            ChangeKind::Minted,
        )
    }

    /// Begin from a caller-qualified recorded graph without changing its stores.
    pub fn begin_recorded(&self, author: Author, baseline: Option<Graph>) -> GraphSession<B> {
        let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        GraphSession::new_recorded(
            self.backend.clone(),
            manifest,
            baseline,
            author,
            ChangeKind::Minted,
        )
    }

    /// Mint an empty session.
    pub async fn mint(
        &self,
        author: Author,
        display_name: Option<String>,
    ) -> Result<GraphSessionManifest, SessionError> {
        let mut manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        manifest.display_name = display_name;
        self.create(manifest, None, author, ChangeKind::Minted)
            .await
    }

    /// Fork `parent` at journal cursor `at`: a new session whose baseline is
    /// the parent's graph there and whose journal starts empty.
    pub async fn fork_at(
        &self,
        parent: &GraphSession<B>,
        at: Seq,
        author: Author,
    ) -> Result<GraphSessionManifest, SessionError> {
        let live = parent.journal().live_cursor();
        let graph = parent
            .graph_at(at)
            .ok_or(SessionError::CursorOutOfRange { cursor: at, live })?;
        self.create_fork(parent, at, &graph, author).await
    }

    /// Fork the connected component around node `seed` out of `parent`
    /// (Turnstone's tear-out), as a new session.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn fork_component(
        &self,
        parent: &GraphSession<B>,
        seed: Uuid,
        author: Author,
    ) -> Result<GraphSessionManifest, SessionError> {
        let fork_graph_id = GraphId::new();
        let graph = fork_component_graph(
            parent.graph(),
            *parent.manifest().root_graph_id.as_uuid(),
            seed,
            *fork_graph_id.as_uuid(),
        )
        .ok_or(SessionError::NoSuchNode(seed))?;
        let at = parent.journal().live_cursor();
        let mut manifest = GraphSessionManifest::new(SessionId::new(), fork_graph_id);
        manifest.parent_session = Some(parent.id());
        manifest.forked_at = Some(at.0);
        let kind = ChangeKind::Forked {
            from: parent.id(),
            at,
        };
        self.create(manifest, Some(&graph), author, kind).await
    }

    async fn create_fork(
        &self,
        parent: &GraphSession<B>,
        at: Seq,
        graph: &Graph,
        author: Author,
    ) -> Result<GraphSessionManifest, SessionError> {
        let mut manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        manifest.parent_session = Some(parent.id());
        manifest.forked_at = Some(at.0);
        manifest.display_name = parent.manifest().display_name.clone();
        let kind = ChangeKind::Forked {
            from: parent.id(),
            at,
        };
        self.create(manifest, Some(graph), author, kind).await
    }

    /// Write a new session's manifest, baseline and first change as one batch.
    async fn create(
        &self,
        manifest: GraphSessionManifest,
        baseline: Option<&Graph>,
        author: Author,
        kind: ChangeKind,
    ) -> Result<GraphSessionManifest, SessionError> {
        let mut session = GraphSession::new(
            self.backend.clone(),
            manifest,
            baseline.cloned(),
            author,
            kind,
        );
        session.flush(wall_clock_now()).await?;
        Ok(session.manifest().clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kernel::graph::replay_captured_deltas_onto;

    use euclid::default::Point2D;
    use muniment::{DirectoryBackend, MemoryBackend};

    async fn legacy_receipt_fixture() -> (MemoryBackend, SessionId, Seq) {
        legacy_receipt_fixture_with_addresses(false).await
    }

    async fn legacy_receipt_fixture_with_addresses(
        same_host: bool,
    ) -> (MemoryBackend, SessionId, Seq) {
        let store = MemoryBackend::new();
        let mut baseline = kernel::graph::replay_captured_deltas([add(1), add(2)]);
        if same_host {
            replay_captured_deltas_onto(
                &mut baseline,
                [
                    CapturedDelta::ReplaySetNodeUrlById {
                        node_id: Uuid::from_u128(1).to_string(),
                        new_url: "https://same-host.test/".into(),
                    },
                    CapturedDelta::ReplaySetNodeUrlById {
                        node_id: Uuid::from_u128(2).to_string(),
                        new_url: "https://same-host.test/child".into(),
                    },
                ],
            );
            baseline = Graph::try_from_snapshot(&baseline.to_snapshot()).unwrap();
        }
        let grouped = |from: u128, to: u128| CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(from).to_string(),
            to_id: Uuid::from_u128(to).to_string(),
            assertion: kernel::graph::EdgeAssertion::Semantic {
                sub_kind: kernel::graph::SemanticSubKind::UserGrouped,
                label: Some("held surface".into()),
                decay_progress: None,
            },
        };
        replay_captured_deltas_onto(&mut baseline, [grouped(1, 2)]);
        let mut raw = baseline.to_snapshot();
        for edge in &mut raw.edges {
            if let Some(semantic) = &mut edge.semantic {
                semantic.statements.clear();
            }
        }
        let mere = MereSessions::new(store.clone());
        let mut session = mere.begin(person(), Some(baseline));
        for (author, delta) in [
            (Author::person("node-author").via("legacy-host"), add(3)),
            (person(), relate(1, 2)),
            (
                Author::person("group-author").via("legacy-editor"),
                grouped(2, 3),
            ),
        ] {
            let first = session.journal.live_cursor();
            session.journal.record_as(author.clone(), delta);
            session.changes.append(Change {
                author,
                kind: ChangeKind::Edit,
                first,
                end: session.journal.live_cursor(),
            });
        }
        session.graph = session
            .journal
            .snapshot_at_from(&session.baseline, session.journal.live_cursor())
            .unwrap();
        session
            .store(wall_clock_now(), Vec::new(), true)
            .await
            .unwrap();
        let id = session.id();
        store
            .apply(&[pretty(
                Keys::new(id).at(BASELINE),
                &Baseline {
                    graph: raw,
                    facets: session.baseline.facets().clone(),
                },
            )
            .unwrap()])
            .await
            .unwrap();
        (store, id, Seq(3))
    }

    #[test]
    fn prepared_translation_transaction_rechecks_every_retained_input() {
        pollster::block_on(async {
            let (store, id, _) = legacy_receipt_fixture().await;
            let session = GraphSession::open(store.clone(), id).await.unwrap();
            let source = session.retained_source.as_ref().unwrap();
            let keys = Keys::new(id);
            let baseline = store.get(&keys.at(BASELINE)).await.unwrap().unwrap();
            let entry_key = muniment::journal::entry_key(&keys.at(JOURNAL), Seq(0));
            let entry = store.get(&entry_key).await.unwrap().unwrap();
            let appended =
                muniment::journal::entry_key(&keys.at(JOURNAL), Seq(source.entries.len() as u64));
            let mut changed_baseline = baseline.clone();
            changed_baseline.push(b'\n');
            let mut changed_entry = entry.clone();
            changed_entry.push(b'\n');
            for (key, invalid, restore) in [
                (keys.at(BASELINE), changed_baseline, Some(baseline)),
                (entry_key, changed_entry, Some(entry)),
                (appended, b"new concurrent entry".to_vec(), None),
                (keys.at(PLACEMENT), b"new concurrent profile".to_vec(), None),
            ] {
                store.put(&key, &invalid).await.unwrap();
                let candidate = || {
                    vec![WriteOp::Put {
                        key: keys.at(TRANSLATION),
                        value: b"candidate".to_vec(),
                    }]
                };
                let (transaction, rejected) = translation_transaction(&keys, source, candidate());
                store.transact(transaction).await.unwrap();
                assert!(rejected.lock().unwrap().is_some());
                assert!(store.get(&keys.at(TRANSLATION)).await.unwrap().is_none());
                assert_eq!(store.get(&key).await.unwrap().unwrap(), invalid);
                match restore {
                    Some(bytes) => store.put(&key, &bytes).await.unwrap(),
                    None => store.delete(&key).await.unwrap(),
                }
                let (transaction, rejected) = translation_transaction(&keys, source, candidate());
                store.transact(transaction).await.unwrap();
                assert!(rejected.lock().unwrap().is_none());
                assert_eq!(
                    store.get(&keys.at(TRANSLATION)).await.unwrap().unwrap(),
                    b"candidate"
                );
                store.delete(&keys.at(TRANSLATION)).await.unwrap();
            }
        });
    }

    #[test]
    fn legacy_translation_receipt_preserves_first_handles_history_undo_and_modern_suffix() {
        pollster::block_on(async {
            let (store, id, cutoff) = legacy_receipt_fixture_with_addresses(true).await;
            let keys = Keys::new(id);
            let retained_baseline = store.get(&keys.at(BASELINE)).await.unwrap().unwrap();
            let retained_entries = {
                let mut values = Vec::new();
                for index in 0..cutoff.0 {
                    values.push(
                        store
                            .get(&muniment::journal::entry_key(&keys.at(JOURNAL), Seq(index)))
                            .await
                            .unwrap()
                            .unwrap(),
                    );
                }
                values
            };
            let mut session = GraphSession::open_legacy(store.clone(), id, cutoff, vec![])
                .await
                .unwrap();
            let original_entries = session.journal().entries()[..cutoff.index()].to_vec();
            let first = FrozenGraph::of(session.graph());
            assert!(first.graph.resource_edges.iter().any(|edge| {
                edge.containment.as_ref().is_some_and(|bucket| {
                    bucket
                        .sub_kinds
                        .contains(&kernel::persistence::PersistedContainmentSubKind::Domain)
                })
            }));
            assert!(first.graph.resource_edges.iter().any(|edge| {
                edge.containment.as_ref().is_some_and(|bucket| {
                    bucket
                        .sub_kinds
                        .contains(&kernel::persistence::PersistedContainmentSubKind::UrlPath)
                })
            }));
            assert!(
                first
                    .graph
                    .edges
                    .iter()
                    .all(|edge| edge.containment.as_ref().is_none_or(|bucket| !bucket
                        .sub_kinds
                        .contains(&kernel::persistence::PersistedContainmentSubKind::Domain)
                        && !bucket
                            .sub_kinds
                            .contains(&kernel::persistence::PersistedContainmentSubKind::UrlPath)))
            );
            let histories: Vec<_> = (0..=cutoff.0)
                .map(|cursor| FrozenGraph::of(&session.graph_at(Seq(cursor)).unwrap()))
                .collect();
            assert_eq!(histories[cutoff.index()], first);
            assert_eq!(
                histories[0]
                    .graph
                    .edges
                    .iter()
                    .filter_map(|edge| edge.semantic.as_ref())
                    .map(|bucket| bucket.statements.len())
                    .sum::<usize>(),
                1,
                "aggregate-only held baseline gains one frozen handle"
            );
            let resource_claims: Vec<_> = first
                .graph
                .resource_edges
                .iter()
                .filter_map(|edge| edge.semantic.as_ref())
                .flat_map(|bucket| bucket.statements.iter())
                .collect();
            assert_eq!(resource_claims.len(), 1);
            assert_eq!(
                resource_claims[0].provenance_iri.as_deref(),
                Some(person().asserter_iri().as_str())
            );
            let resource_handle = resource_claims[0].statement_id.clone();
            let group_author = Author::person("group-author").via("legacy-editor");
            assert!(
                first
                    .graph
                    .edges
                    .iter()
                    .filter_map(|edge| edge.semantic.as_ref())
                    .flat_map(|bucket| bucket.statements.iter())
                    .any(|claim| claim.provenance_iri.as_deref()
                        == Some(group_author.asserter_iri().as_str()))
            );
            for _ in 0..2 {
                let reopened = GraphSession::open(store.clone(), id).await.unwrap();
                assert_eq!(FrozenGraph::of(reopened.graph()), first);
                for (cursor, expected) in histories.iter().enumerate() {
                    assert_eq!(
                        FrozenGraph::of(&reopened.graph_at(Seq(cursor as u64)).unwrap()),
                        *expected
                    );
                }
                assert_eq!(
                    reopened.journal().entries()[..cutoff.index()],
                    original_entries
                );
            }
            session.undo(group_author.clone()).await.unwrap().unwrap();
            assert_eq!(
                session.graph().to_snapshot().resource_edges,
                first.graph.resource_edges,
                "undo held Surface control keeps migrated Resource claim"
            );
            session.redo(group_author).await.unwrap().unwrap();
            assert_eq!(
                FrozenGraph::of(session.graph()),
                first,
                "redo restores first Surface handles and facets"
            );
            session.undo(person()).await.unwrap().unwrap();
            assert!(
                session
                    .graph()
                    .to_snapshot()
                    .resource_edges
                    .iter()
                    .all(|edge| edge.semantic.as_ref().is_none_or(|bucket| bucket
                        .statements
                        .iter()
                        .all(|claim| claim.statement_id != resource_handle)))
            );
            assert_eq!(
                session.graph().to_snapshot().edges,
                first.graph.edges,
                "resource undo keeps held Surface controls"
            );
            session.redo(person()).await.unwrap().unwrap();
            assert_eq!(
                FrozenGraph::of(session.graph()),
                first,
                "resource redo restores exact original handles/source/time"
            );

            let modern = Author::person("modern-author").via("modern-host");
            let predicate = "urn:mere:receipt:modern-held";
            session.edit_now(modern.clone(), |graph| {
                graph
                    .declare_predicate(predicate, kernel::graph::GraphStratum::Surface)
                    .unwrap()
            });
            session.flush(wall_clock_now()).await.unwrap();
            session
                .apply(
                    modern.clone(),
                    vec![CapturedDelta::ReplayAssertSemanticPredicateByIds {
                        from_id: Uuid::from_u128(1).to_string(),
                        to_id: Uuid::from_u128(2).to_string(),
                        predicate: predicate.into(),
                    }],
                )
                .await
                .unwrap();
            session.edit_now(modern.clone(), |graph| {
                graph
                    .declare_predicate(predicate, kernel::graph::GraphStratum::Resource)
                    .unwrap()
            });
            session.checkpoint().await.unwrap();
            let final_graph = FrozenGraph::of(session.graph());
            assert!(
                final_graph
                    .graph
                    .edges
                    .iter()
                    .filter_map(|edge| edge.semantic.as_ref())
                    .flat_map(|bucket| bucket.statements.iter())
                    .any(|claim| claim.predicate == predicate
                        && claim.provenance_iri.as_deref() == Some(modern.asserter_iri().as_str()))
            );
            assert!(
                final_graph
                    .graph
                    .resource_edges
                    .iter()
                    .filter_map(|edge| edge.semantic.as_ref())
                    .flat_map(|bucket| bucket.statements.iter())
                    .all(|claim| claim.predicate != predicate),
                "later Resource declaration does not migrate modern held claims"
            );
            assert_eq!(session.placement().unwrap().cutoff(), cutoff);
            for _ in 0..2 {
                let reopened = GraphSession::open_legacy(store.clone(), id, cutoff, vec![])
                    .await
                    .unwrap();
                assert_eq!(FrozenGraph::of(reopened.graph()), final_graph);
                assert_eq!(
                    FrozenGraph::of(&reopened.graph_at(reopened.journal().live_cursor()).unwrap()),
                    final_graph
                );
                assert_eq!(
                    reopened.journal().entries()[..cutoff.index()],
                    original_entries
                );
            }
            store
                .apply(&[pretty(
                    keys.at(CHECKPOINT),
                    &Checkpoint {
                        cursor: session.journal().live_cursor(),
                        translated_receipt_digest: None,
                    },
                )
                .unwrap()])
                .await
                .unwrap();
            assert_eq!(
                FrozenGraph::of(GraphSession::open(store.clone(), id).await.unwrap().graph()),
                final_graph,
                "unqualified checkpoint falls back to the receipt, keeping first IDs"
            );
            assert_eq!(
                store.get(&keys.at(BASELINE)).await.unwrap().unwrap(),
                retained_baseline
            );
            for (index, expected) in retained_entries.iter().enumerate() {
                assert_eq!(
                    store
                        .get(&muniment::journal::entry_key(
                            &keys.at(JOURNAL),
                            Seq(index as u64)
                        ))
                        .await
                        .unwrap()
                        .unwrap(),
                    *expected
                );
            }
        });
    }

    #[test]
    fn legacy_translation_receipt_rejects_source_profile_and_receipt_damage_atomically() {
        pollster::block_on(async {
            let (store, id, cutoff) = legacy_receipt_fixture().await;
            let mut session = GraphSession::open_legacy(store.clone(), id, cutoff, vec![])
                .await
                .unwrap();
            let expected = FrozenGraph::of(session.graph());
            let keys = Keys::new(id);
            let checkpoint = store.get(&keys.at(GRAPH)).await.unwrap().unwrap();
            let receipt = store.get(&keys.at(TRANSLATION)).await.unwrap().unwrap();
            let baseline = store.get(&keys.at(BASELINE)).await.unwrap().unwrap();
            let profile = store.get(&keys.at(PLACEMENT)).await.unwrap().unwrap();
            let first_key = muniment::journal::entry_key(&keys.at(JOURNAL), Seq(0));
            let first_entry = store.get(&first_key).await.unwrap().unwrap();
            let mut changed_source = baseline.clone();
            changed_source.push(b'\n');
            let mut changed_entry: serde_json::Value =
                serde_json::from_slice(&first_entry).unwrap();
            changed_entry["entry"]["author"]["id"] = serde_json::json!("changed-author");
            let mut changed_receipt: serde_json::Value = serde_json::from_slice(&receipt).unwrap();
            changed_receipt["translation_digest"] = serde_json::json!("invalid");
            let changed_profile = SessionPlacement::legacy(Seq(0), vec![]);
            for (key, invalid, original) in [
                (keys.at(BASELINE), changed_source, baseline),
                (
                    first_key,
                    serde_json::to_vec(&changed_entry).unwrap(),
                    first_entry,
                ),
                (
                    keys.at(TRANSLATION),
                    serde_json::to_vec(&changed_receipt).unwrap(),
                    receipt.clone(),
                ),
                (
                    keys.at(PLACEMENT),
                    serde_json::to_vec(&changed_profile).unwrap(),
                    profile,
                ),
            ] {
                store
                    .apply(&[WriteOp::Put {
                        key: key.clone(),
                        value: invalid.clone(),
                    }])
                    .await
                    .unwrap();
                assert!(matches!(
                    GraphSession::open(store.clone(), id).await,
                    Err(SessionError::Corrupt(_))
                ));
                assert_eq!(store.get(&key).await.unwrap().unwrap(), invalid);
                assert_eq!(
                    store.get(&keys.at(GRAPH)).await.unwrap().unwrap(),
                    checkpoint,
                    "failed load writes no checkpoint"
                );
                assert_eq!(
                    FrozenGraph::of(session.graph()),
                    expected,
                    "existing live session stays intact"
                );
                store
                    .apply(&[WriteOp::Put {
                        key,
                        value: original,
                    }])
                    .await
                    .unwrap();
                assert_eq!(
                    FrozenGraph::of(GraphSession::open(store.clone(), id).await.unwrap().graph()),
                    expected,
                    "unchanged source positive control"
                );
            }
            store.delete(&keys.at(TRANSLATION)).await.unwrap();
            assert!(matches!(
                GraphSession::open(store.clone(), id).await,
                Err(SessionError::Corrupt(_))
            ));
            assert_eq!(
                store.get(&keys.at(GRAPH)).await.unwrap().unwrap(),
                checkpoint
            );
            store.put(&keys.at(TRANSLATION), &receipt).await.unwrap();
            assert_eq!(
                FrozenGraph::of(GraphSession::open(store.clone(), id).await.unwrap().graph()),
                expected
            );
            assert!(
                session
                    .qualify_legacy(Seq(cutoff.0 + 1), vec![])
                    .await
                    .is_err()
            );
            assert_eq!(FrozenGraph::of(session.graph()), expected);
            let mere = MereSessions::new(store.clone());
            let mut undurable = mere.begin(person(), Some(Graph::new()));
            assert!(undurable.qualify_legacy(Seq(0), vec![]).await.is_err());
            assert!(
                store
                    .get(&undurable.keys.at(TRANSLATION))
                    .await
                    .unwrap()
                    .is_none()
            );
        });
    }

    #[test]
    fn legacy_translation_qualification_rejects_a_changed_open_source_before_writing() {
        pollster::block_on(async {
            let (store, id, cutoff) = legacy_receipt_fixture().await;
            let mut session = GraphSession::open(store.clone(), id).await.unwrap();
            let expected = FrozenGraph::of(session.graph());
            let keys = Keys::new(id);
            let original = store.get(&keys.at(BASELINE)).await.unwrap().unwrap();
            let mut changed = original.clone();
            changed.push(b'\n');
            store.put(&keys.at(BASELINE), &changed).await.unwrap();
            assert!(matches!(
                session.qualify_legacy(cutoff, vec![]).await,
                Err(SessionError::Corrupt(_))
            ));
            assert_eq!(FrozenGraph::of(session.graph()), expected);
            assert!(store.get(&keys.at(TRANSLATION)).await.unwrap().is_none());
            assert!(store.get(&keys.at(PLACEMENT)).await.unwrap().is_none());
            assert_eq!(
                store.get(&keys.at(BASELINE)).await.unwrap().unwrap(),
                changed
            );
            store.put(&keys.at(BASELINE), &original).await.unwrap();
            session.qualify_legacy(cutoff, vec![]).await.unwrap();
            let qualified = FrozenGraph::of(session.graph());
            assert_eq!(
                FrozenGraph::of(GraphSession::open(store.clone(), id).await.unwrap().graph()),
                qualified
            );
            assert_eq!(
                store.get(&keys.at(BASELINE)).await.unwrap().unwrap(),
                original
            );
        });
    }

    #[derive(Clone)]
    struct QualificationBackend {
        store: MemoryBackend,
        before_commit: Option<WriteOp>,
        transactional: bool,
    }

    #[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
    #[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
    impl Backend for QualificationBackend {
        async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, StoreError> {
            self.store.get(key).await
        }
        async fn put(&self, key: &str, bytes: &[u8]) -> Result<(), StoreError> {
            self.store.put(key, bytes).await
        }
        async fn delete(&self, key: &str) -> Result<(), StoreError> {
            self.store.delete(key).await
        }
        async fn list(&self, prefix: &str) -> Result<Vec<String>, StoreError> {
            self.store.list(prefix).await
        }
        async fn scan(&self, start: &str, end: &str) -> Result<Vec<String>, StoreError> {
            self.store.scan(start, end).await
        }
        async fn apply(&self, ops: &[WriteOp]) -> Result<(), StoreError> {
            self.store.apply(ops).await
        }
        async fn transact(&self, transaction: muniment::TransactFn) -> Result<(), StoreError> {
            if !self.transactional {
                return Err(StoreError::NotTransactional);
            }
            if let Some(op) = &self.before_commit {
                self.store.apply(std::slice::from_ref(op)).await?;
            }
            self.store.transact(transaction).await
        }
    }

    #[test]
    fn qualification_commit_refuses_raced_source_and_unsupported_backend_atomically() {
        pollster::block_on(async {
            let (store, id, cutoff) = legacy_receipt_fixture().await;
            let keys = Keys::new(id);
            let baseline = store.get(&keys.at(BASELINE)).await.unwrap().unwrap();
            let checkpoint = store.get(&keys.at(GRAPH)).await.unwrap().unwrap();
            let mut changed = baseline.clone();
            changed.push(b'\n');
            for transactional in [true, false] {
                let backend = QualificationBackend {
                    store: store.clone(),
                    before_commit: Some(WriteOp::Put {
                        key: keys.at(BASELINE),
                        value: changed.clone(),
                    }),
                    transactional,
                };
                let mut session = GraphSession::open(backend, id).await.unwrap();
                let live = FrozenGraph::of(session.graph());
                let result = session.qualify_legacy(cutoff, vec![]).await;
                if transactional {
                    assert!(matches!(result, Err(SessionError::Corrupt(_))));
                    assert_eq!(
                        store.get(&keys.at(BASELINE)).await.unwrap().unwrap(),
                        changed
                    );
                } else {
                    assert!(matches!(
                        result,
                        Err(SessionError::Store(StoreError::NotTransactional))
                    ));
                    assert_eq!(
                        store.get(&keys.at(BASELINE)).await.unwrap().unwrap(),
                        baseline
                    );
                }
                assert_eq!(FrozenGraph::of(session.graph()), live);
                assert!(session.placement().is_none());
                assert!(store.get(&keys.at(TRANSLATION)).await.unwrap().is_none());
                assert!(store.get(&keys.at(PLACEMENT)).await.unwrap().is_none());
                assert_eq!(
                    store.get(&keys.at(GRAPH)).await.unwrap().unwrap(),
                    checkpoint
                );
                store.put(&keys.at(BASELINE), &baseline).await.unwrap();
            }
            let qualified = GraphSession::open_legacy(store.clone(), id, cutoff, vec![])
                .await
                .unwrap();
            assert!(qualified.placement().is_some());
            assert_eq!(
                FrozenGraph::of(qualified.graph()),
                FrozenGraph::of(GraphSession::open(store, id).await.unwrap().graph())
            );
        });
    }

    #[test]
    fn sealed_legacy_receipt_activation_history_undo_and_corruption_are_exact() {
        pollster::block_on(async {
            let (source, id, cutoff) = legacy_receipt_fixture_with_addresses(true).await;
            let raw = MemoryBackend::new();
            let epoch = crate::wallet_store::KeyEpochId(Uuid::from_bytes([7; 16]));
            let sealed = crate::WalletSealedBackend::new(
                raw.clone(),
                crate::WalletEpochSealer::from_epoch(
                    crate::PersonaId::new(),
                    epoch,
                    b"session secret",
                ),
            );
            let mut ops = Vec::new();
            for key in source.list("").await.unwrap() {
                ops.push(WriteOp::Put {
                    value: source.get(&key).await.unwrap().unwrap(),
                    key,
                });
            }
            sealed.apply(&ops).await.unwrap();
            let keys = Keys::new(id);
            let original_baseline = sealed.get(&keys.at(BASELINE)).await.unwrap().unwrap();
            let original_entries = {
                let mut entries = Vec::new();
                for index in 0..cutoff.0 {
                    let key = muniment::journal::entry_key(&keys.at(JOURNAL), Seq(index));
                    entries.push((key.clone(), sealed.get(&key).await.unwrap().unwrap()));
                }
                entries
            };
            let mut session = GraphSession::open_legacy(sealed.clone(), id, cutoff, vec![])
                .await
                .unwrap();
            let qualified = FrozenGraph::of(session.graph());
            let histories: Vec<_> = (0..=cutoff.0)
                .map(|cursor| FrozenGraph::of(&session.graph_at(Seq(cursor)).unwrap()))
                .collect();
            for _ in 0..2 {
                let reopened = GraphSession::open(sealed.clone(), id).await.unwrap();
                assert_eq!(FrozenGraph::of(reopened.graph()), qualified);
                for (index, expected) in histories.iter().enumerate() {
                    assert_eq!(
                        FrozenGraph::of(&reopened.graph_at(Seq(index as u64)).unwrap()),
                        *expected
                    );
                }
                assert_eq!(reopened.journal().entries(), session.journal().entries());
            }
            assert!(session.undo(person()).await.unwrap().is_some());
            assert_ne!(FrozenGraph::of(session.graph()), qualified);
            assert!(session.redo(person()).await.unwrap().is_some());
            assert_eq!(FrozenGraph::of(session.graph()), qualified);
            session.flush(wall_clock_now()).await.unwrap();
            session.checkpoint().await.unwrap();
            assert_eq!(
                FrozenGraph::of(
                    GraphSession::open(sealed.clone(), id)
                        .await
                        .unwrap()
                        .graph()
                ),
                qualified
            );
            // Without the checkpoint, the frozen receipt and retained Authors give the same graph.
            sealed.delete(&keys.at(CHECKPOINT)).await.unwrap();
            assert_eq!(
                FrozenGraph::of(
                    GraphSession::open(sealed.clone(), id)
                        .await
                        .unwrap()
                        .graph()
                ),
                qualified
            );
            assert_eq!(
                sealed.get(&keys.at(BASELINE)).await.unwrap().unwrap(),
                original_baseline
            );
            for (key, value) in original_entries {
                assert_eq!(sealed.get(&key).await.unwrap().unwrap(), value);
            }
            let receipt_key = keys.at(TRANSLATION);
            let original_receipt = sealed.get(&receipt_key).await.unwrap().unwrap();
            let mut corrupted: serde_json::Value =
                serde_json::from_slice(&original_receipt).unwrap();
            corrupted["translation_digest"] = serde_json::json!("damaged");
            sealed
                .put(&receipt_key, &serde_json::to_vec(&corrupted).unwrap())
                .await
                .unwrap();
            let corrupted_ciphertext = raw.get(&receipt_key).await.unwrap().unwrap();
            assert!(matches!(
                GraphSession::open(sealed.clone(), id).await,
                Err(SessionError::Corrupt(_))
            ));
            assert_eq!(
                raw.get(&receipt_key).await.unwrap().unwrap(),
                corrupted_ciphertext
            );
            assert_eq!(FrozenGraph::of(session.graph()), qualified);
            sealed.put(&receipt_key, &original_receipt).await.unwrap();
            assert_eq!(
                FrozenGraph::of(
                    GraphSession::open(sealed.clone(), id)
                        .await
                        .unwrap()
                        .graph()
                ),
                qualified
            );
            let good_ciphertext = raw.get(&receipt_key).await.unwrap().unwrap();
            raw.put(&receipt_key, b"unsealed receipt").await.unwrap();
            assert!(GraphSession::open(sealed.clone(), id).await.is_err());
            assert_eq!(
                raw.get(&receipt_key).await.unwrap().unwrap(),
                b"unsealed receipt"
            );
            raw.put(&receipt_key, &good_ciphertext).await.unwrap();
            assert_eq!(
                FrozenGraph::of(
                    GraphSession::open(sealed.clone(), id)
                        .await
                        .unwrap()
                        .graph()
                ),
                qualified
            );
            for key in raw.list("").await.unwrap() {
                let ciphertext: serde_json::Value =
                    serde_json::from_slice(&raw.get(&key).await.unwrap().unwrap()).unwrap();
                assert_eq!(ciphertext["format"], "pandect-wallet-slot-v1");
            }
        });
    }

    #[test]
    fn placement_is_explicit_and_recorded_sessions_keep_held_surface_claims() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let mut baseline =
                kernel::graph::replay_captured_deltas([add(1), add(2), relate(1, 2)]);
            let predicate = "urn:mere:profile-test:custom";
            replay_captured_deltas_onto(
                &mut baseline,
                [CapturedDelta::ReplaySetEdgeSemanticPredicateByIds {
                    from_id: Uuid::from_u128(1).to_string(),
                    to_id: Uuid::from_u128(2).to_string(),
                    predicate: Some(predicate.into()),
                }],
            );
            baseline
                .write_as(person(), |graph| {
                    graph.declare_predicate(predicate, kernel::graph::GraphStratum::Resource)
                })
                .unwrap();
            assert!(baseline.to_snapshot().resource_edges.is_empty());
            let edges = baseline.to_snapshot().edges;
            let mut unqualified = mere.begin(person(), Some(baseline.clone()));
            assert_eq!(unqualified.placement(), None);
            unqualified.flush(wall_clock_now()).await.unwrap();
            let id = unqualified.id();
            let opened = mere.open(id).await.unwrap();
            assert_eq!(
                opened.placement(),
                None,
                "columns never qualify an arbitrary baseline"
            );
            assert_eq!(opened.graph().to_snapshot().edges, edges);

            let mut recorded = mere.begin_recorded(person(), Some(baseline));
            assert_eq!(recorded.placement(), Some(&SessionPlacement::recorded()));
            let pending = recorded.pending(wall_clock_now()).unwrap();
            assert!(pending.ops().iter().any(
                |op| matches!(op, WriteOp::Put { key, .. } if key.ends_with("/placement.json"))
            ));
            recorded.flush(wall_clock_now()).await.unwrap();
            let recorded_id = recorded.id();
            let opened = mere.open(recorded_id).await.unwrap();
            assert_eq!(opened.placement(), Some(&SessionPlacement::recorded()));
            assert_eq!(
                opened.graph().to_snapshot().edges,
                edges,
                "recorded placement does not relocate held claims"
            );
            assert!(opened.graph().to_snapshot().resource_edges.is_empty());
            assert_eq!(
                opened.graph().predicate_declarations(predicate).unwrap(),
                recorded.graph().predicate_declarations(predicate).unwrap()
            );

            let fresh = mere.begin(person(), None);
            assert_eq!(fresh.placement(), Some(&SessionPlacement::recorded()));
            let key = Keys::new(recorded_id).at(PLACEMENT);
            for invalid in [
                serde_json::json!({"baseline":"future-placement-v9", "legacy_journal_until":null}),
                serde_json::json!({"baseline":"legacy-surface-v1", "legacy_journal_until":null}),
                serde_json::json!({"baseline":"recorded-strata-v1", "legacy_journal_until":9}),
                serde_json::json!({"baseline":"recorded-strata-v1", "origins":[{"statement_id":"unexpected", "from_resource_id":Uuid::nil(), "to_resource_id":Uuid::nil()}]}),
            ] {
                let bytes = serde_json::to_vec(&invalid).unwrap();
                store
                    .apply(&[WriteOp::Put {
                        key: key.clone(),
                        value: bytes.clone(),
                    }])
                    .await
                    .unwrap();
                assert!(matches!(
                    mere.open(recorded_id).await,
                    Err(SessionError::Corrupt(_))
                ));
                let slots = JsonSlots::new(store.clone());
                assert_eq!(
                    read::<_, serde_json::Value>(&slots, &key)
                        .await
                        .unwrap()
                        .unwrap(),
                    invalid
                );
                store
                    .apply(&[pretty(key.clone(), &SessionPlacement::recorded()).unwrap()])
                    .await
                    .unwrap();
                assert_eq!(
                    mere.open(recorded_id)
                        .await
                        .unwrap()
                        .graph()
                        .to_snapshot()
                        .edges,
                    edges
                );
            }
        });
    }

    fn person() -> Author {
        Author::person("persona-a").via("turnstone")
    }

    fn knot() -> Author {
        Author::person("persona-a").via("knot-editor")
    }

    fn add(id: u128) -> CapturedDelta {
        CapturedDelta::ReplayAddNodeWithIdIfMissing {
            id: Uuid::from_u128(id).to_string(),
            url: format!("https://{id}.test/"),
            position: [0.0, 0.0],
        }
    }

    fn retitle(id: u128, title: &str) -> CapturedDelta {
        CapturedDelta::ReplaySetNodeTitleById {
            node_id: Uuid::from_u128(id).to_string(),
            title: title.to_string(),
        }
    }

    fn tag(id: u128, tag: &str) -> CapturedDelta {
        CapturedDelta::ReplayInsertNodeTagById {
            node_id: Uuid::from_u128(id).to_string(),
            tag: tag.to_string(),
        }
    }

    fn relate(from: u128, to: u128) -> CapturedDelta {
        CapturedDelta::ReplayAssertRelationByIds {
            from_id: Uuid::from_u128(from).to_string(),
            to_id: Uuid::from_u128(to).to_string(),
            assertion: kernel::graph::EdgeAssertion::Semantic {
                sub_kind: kernel::graph::SemanticSubKind::Cites,
                label: None,
                decay_progress: None,
            },
        }
    }

    /// Sorted node ids with titles, and sorted relation endpoints: equal
    /// graphs regardless of petgraph keys.
    type Pairs = Vec<(String, String)>;

    fn fingerprint(graph: &Graph) -> (Pairs, Pairs) {
        let mut nodes: Vec<_> = graph
            .nodes()
            .map(|(_, node)| (node.id.to_string(), node.title.clone()))
            .collect();
        nodes.sort();
        let mut edges: Vec<_> = graph
            .relations()
            .map(|relation| {
                let from = graph.get_node(relation.from).unwrap().id.to_string();
                let to = graph.get_node(relation.to).unwrap().id.to_string();
                (from, to)
            })
            .collect();
        edges.sort();
        (nodes, edges)
    }

    #[test]
    fn a_session_journals_its_edits_under_their_authors_and_reopens() {
        pollster::block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let store = DirectoryBackend::open(dir.path()).unwrap();
            let mere = MereSessions::new(store.clone());
            let manifest = mere.mint(person(), Some("readings".into())).await.unwrap();
            let id = manifest.session_id;

            let mut session = mere.open(id).await.unwrap();
            let first = session.apply(person(), vec![add(1), add(2)]).await.unwrap();
            // Each new node journals its visit stamp beside the add.
            assert_eq!((first.first, first.end), (Seq(0), Seq(4)));
            session
                .apply(knot(), vec![relate(1, 2), retitle(1, "The Tower")])
                .await
                .unwrap();
            let live = fingerprint(session.graph());
            let authors: Vec<String> = session
                .journal()
                .entries()
                .iter()
                .map(|entry| entry.author.to_string())
                .collect();
            assert_eq!(
                authors,
                [
                    "person persona-a via turnstone",
                    "person persona-a via turnstone",
                    "person persona-a via turnstone",
                    "person persona-a via turnstone",
                    "person persona-a via knot-editor",
                    "person persona-a via knot-editor",
                ]
            );
            let kinds: Vec<&ChangeKind> = session.changes().iter().map(|c| &c.kind).collect();
            assert_eq!(
                kinds,
                [&ChangeKind::Minted, &ChangeKind::Edit, &ChangeKind::Edit]
            );
            drop(session);
            drop(mere);
            drop(store);

            // The folder is Turnstone's layout, with a line per journal entry.
            let session_dir = dir.path().join("sessions").join(id.as_uuid().to_string());
            assert!(session_dir.join("manifest.json").is_file());
            let journal = std::fs::read_to_string(session_dir.join("journal.jsonl")).unwrap();
            assert_eq!(journal.lines().count(), 6);

            let store = DirectoryBackend::open(dir.path()).unwrap();
            let reopened = GraphSession::open(store, id).await.unwrap();
            assert_eq!(fingerprint(reopened.graph()), live);
            assert_eq!(reopened.journal().len(), 6);
            assert_eq!(reopened.changes().len(), 3);
            assert_eq!(
                reopened.manifest().display_name.as_deref(),
                Some("readings")
            );
        });
    }

    #[test]
    fn checked_snapshot_session_load_reports_baseline_and_checkpoint_corruption() {
        use kernel::persistence::{PersistedResourceFacet, PersistedResourceRecord};

        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let keys = Keys::new(id);
            let graph = kernel::graph::replay_captured_deltas([add(1)]);
            let mut valid = graph.to_snapshot();
            valid.resources.push(PersistedResourceRecord {
                canonical_iri: "urn:mere:test:checked-session".into(),
                facets: vec![PersistedResourceFacet {
                    facet: "foreign.metadata".into(),
                    value_json: "true".into(),
                }],
            });
            let baseline = |snapshot: GraphSnapshot| Baseline {
                graph: snapshot,
                facets: NodeFacetStore::default(),
            };
            for column in [BASELINE, GRAPH] {
                for conflict in [false, true] {
                    store
                        .apply(&[
                            pretty(keys.at(BASELINE), &baseline(valid.clone())).unwrap(),
                            pretty(keys.at(GRAPH), &valid).unwrap(),
                            pretty(
                                keys.at(CHECKPOINT),
                                &Checkpoint {
                                    cursor: Seq(0),
                                    translated_receipt_digest: None,
                                },
                            )
                            .unwrap(),
                        ])
                        .await
                        .unwrap();
                    let opened = GraphSession::open(store.clone(), id).await.unwrap();
                    assert_eq!(opened.graph().to_snapshot().resources, valid.resources);
                    assert_eq!(opened.graph().node_count(), 1);
                    let mut invalid = valid.clone();
                    if conflict {
                        let mut record = invalid.resources[0].clone();
                        record.facets[0].value_json = "false".into();
                        invalid.resources.push(record);
                    } else {
                        invalid.resources[0].facets[0].value_json = "{".into();
                    }
                    let op = if column == BASELINE {
                        pretty(keys.at(column), &baseline(invalid.clone())).unwrap()
                    } else {
                        pretty(keys.at(column), &invalid).unwrap()
                    };
                    store.apply(&[op.clone()]).await.unwrap();
                    match GraphSession::open(store.clone(), id).await {
                        Err(SessionError::Corrupt(error)) => {
                            assert!(error.contains(&keys.at(column)));
                            assert!(error.contains(if conflict {
                                "conflicts"
                            } else {
                                "invalid JSON"
                            }));
                        },
                        Err(error) => panic!("unexpected load error: {error}"),
                        Ok(_) => panic!("invalid resource snapshot was loaded"),
                    }
                    let slots = JsonSlots::new(store.clone());
                    let held = if column == BASELINE {
                        read::<_, Baseline>(&slots, &keys.at(column))
                            .await
                            .unwrap()
                            .unwrap()
                            .graph
                    } else {
                        read::<_, GraphSnapshot>(&slots, &keys.at(column))
                            .await
                            .unwrap()
                            .unwrap()
                    };
                    assert_eq!(held.resources, invalid.resources);
                }
            }
            let mut legacy = serde_json::to_value(&valid).unwrap();
            for column in ["resources", "resource_edges", "shown_resources"] {
                legacy.as_object_mut().unwrap().remove(column);
            }
            let legacy: GraphSnapshot = serde_json::from_value(legacy).unwrap();
            store
                .apply(&[
                    pretty(keys.at(BASELINE), &baseline(legacy.clone())).unwrap(),
                    pretty(keys.at(GRAPH), &legacy).unwrap(),
                ])
                .await
                .unwrap();
            let opened = GraphSession::open(store, id).await.unwrap();
            assert_eq!(opened.graph().node_count(), 1);
            assert!(opened.graph().to_snapshot().resources.is_empty());
        });
    }

    #[test]
    fn a_checkpoint_and_its_tail_load_the_same_graph_as_the_baseline_and_journal() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            session.set_checkpoint_interval(4);
            for n in 0..3 {
                session.apply(person(), vec![add(n)]).await.unwrap();
            }
            // The second node's entries reach the interval; the rest is the tail.
            session
                .apply(person(), vec![retitle(2, "Three")])
                .await
                .unwrap();
            let checkpoint: Checkpoint = JsonSlots::new(store.clone())
                .load(&Keys::new(id).at(CHECKPOINT))
                .await
                .unwrap()
                .expect("a checkpoint was written");
            assert_eq!(checkpoint.cursor, Seq(4));
            assert_eq!(session.journal().len(), 7);
            let live = whole(session.graph());
            drop(session);

            // Whole graphs, facets and visit stamps included: replay is exact.
            let reopened = GraphSession::open(store.clone(), id).await.unwrap();
            assert_eq!(whole(reopened.graph()), live, "checkpoint plus tail");
            let replayed = reopened.graph_at(reopened.journal().live_cursor()).unwrap();
            assert_eq!(whole(&replayed), live, "baseline plus whole journal");
        });
    }

    #[test]
    fn legacy_exact_assertions_keep_minting_authors_across_checkpointed_withdrawals() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            let mut template = kernel::graph::replay_captured_deltas([add(1), add(2)]);
            let from = template.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
            let to = template.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
            template
                .assert_semantic_statement(
                    from,
                    to,
                    kernel::graph::SemanticStatementSpec {
                        predicate: kernel::graph::predicate_iri(
                            kernel::graph::SemanticSubKind::Cites,
                        )
                        .into(),
                        recognized_sub_kind: Some(kernel::graph::SemanticSubKind::Cites),
                        provenance_iri: Some("https://source.test/page".into()),
                        asserted_at_ms: Some(100),
                        ..Default::default()
                    },
                )
                .unwrap();
            let mut baseline_edges = template.to_snapshot().edges;
            assert_eq!(baseline_edges.len(), 1);
            let statements = &mut baseline_edges[0].semantic.as_mut().unwrap().statements;
            statements[0].statement_id = "baseline-source".into();
            let mut unknown = statements[0].clone();
            unknown.statement_id = "baseline-unknown".into();
            unknown.provenance_iri = None;
            statements.push(unknown.clone());
            let exact = |edges| CapturedDelta::ReplaySetEdgesByIds {
                from_id: Uuid::from_u128(1).to_string(),
                to_id: Uuid::from_u128(2).to_string(),
                edges,
            };
            replay_captured_deltas_onto(&mut template, [exact(baseline_edges.clone())]);
            session.baseline = template;
            session.write_baseline = true;
            session.head_stored = false;
            let restorer = Author::person("persona-b").via("knot-editor");
            let mut minted_edges = baseline_edges.clone();
            unknown.statement_id = "journal-minted".into();
            minted_edges[0]
                .semantic
                .as_mut()
                .unwrap()
                .statements
                .push(unknown);
            session
                .journal
                .record_as(person(), exact(minted_edges.clone()));
            session.graph = session.baseline.clone();
            replay_captured_deltas_onto(&mut session.graph, [exact(minted_edges.clone())]);
            session
                .store(SystemTime::now(), Vec::new(), true)
                .await
                .unwrap();
            let old_checkpoint = GraphSession::open(store.clone(), id).await.unwrap();
            let old_checkpoint_snapshot = old_checkpoint.graph().to_snapshot();
            let old_statements = &old_checkpoint_snapshot.edges[0]
                .semantic
                .as_ref()
                .unwrap()
                .statements;
            let old_source_of = |id: &str| {
                old_statements
                    .iter()
                    .find(|s| s.statement_id == id)
                    .unwrap()
                    .provenance_iri
                    .as_deref()
                    .unwrap()
            };
            assert_eq!(
                old_source_of("baseline-source"),
                "https://source.test/page",
                "legacy checkpoint explicit-source control"
            );
            assert_eq!(
                old_source_of("baseline-unknown"),
                kernel::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI,
                "legacy checkpoint baseline control"
            );
            assert_eq!(
                old_source_of("journal-minted"),
                person().asserter_iri(),
                "an empty tail still repairs legacy checkpoint attribution"
            );
            assert_eq!(
                whole(old_checkpoint.graph()),
                whole(&old_checkpoint.graph_at(Seq(1)).unwrap())
            );
            drop(old_checkpoint);
            session
                .journal
                .record_as(restorer.clone(), exact(baseline_edges));
            session.graph = session
                .journal
                .snapshot_at_from(&session.baseline, Seq(2))
                .unwrap();
            session
                .store(SystemTime::now(), Vec::new(), true)
                .await
                .unwrap();
            session
                .journal
                .record_as(restorer.clone(), exact(minted_edges));
            session.graph = session
                .journal
                .snapshot_at_from(&session.baseline, Seq(3))
                .unwrap();
            session.flush(SystemTime::now()).await.unwrap();
            drop(session);

            let reopened = GraphSession::open(store, id).await.unwrap();
            let full = reopened.graph_at(Seq(3)).unwrap();
            assert_eq!(
                whole(reopened.graph()),
                whole(&full),
                "checkpoint plus tail equals full replay"
            );
            let edges = full.to_snapshot().edges;
            let statements = &edges[0].semantic.as_ref().unwrap().statements;
            let source_of = |id: &str| {
                statements
                    .iter()
                    .find(|s| s.statement_id == id)
                    .unwrap()
                    .provenance_iri
                    .as_deref()
                    .unwrap()
            };
            assert_eq!(
                source_of("baseline-source"),
                "https://source.test/page",
                "explicit-source control"
            );
            assert_eq!(
                source_of("baseline-unknown"),
                kernel::graph::edge_data::UNKNOWN_LEGACY_ASSERTER_IRI,
                "baseline control"
            );
            assert_eq!(
                source_of("journal-minted"),
                person().asserter_iri(),
                "restoration keeps the minting Author"
            );
            assert_ne!(source_of("journal-minted"), restorer.asserter_iri());
            assert_eq!(statements.len(), 3);
            assert!(statements.iter().all(|s| s.asserted_at_ms == Some(100)));
        });
    }

    /// A graph's whole persisted state: its snapshot, undated, and its facets.
    fn whole(graph: &Graph) -> (serde_json::Value, serde_json::Value) {
        let mut snapshot = graph.to_snapshot();
        snapshot.timestamp_secs = 0;
        (
            serde_json::to_value(&snapshot).unwrap(),
            serde_json::to_value(graph.facets()).unwrap(),
        )
    }

    #[test]
    fn a_fork_at_a_cursor_starts_from_the_parent_there_and_diverges() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let parent_id = mere
                .mint(person(), Some("parent".into()))
                .await
                .unwrap()
                .session_id;
            let mut parent = mere.open(parent_id).await.unwrap();
            parent.apply(person(), vec![add(1)]).await.unwrap();
            parent.apply(person(), vec![add(2)]).await.unwrap();

            let fork = mere.fork_at(&parent, Seq(1), knot()).await.unwrap();
            assert_eq!(fork.parent_session, Some(parent_id));
            assert_eq!(fork.forked_at, Some(1));
            let mut fork = mere.open(fork.session_id).await.unwrap();
            assert_eq!(
                fork.graph().node_count(),
                1,
                "the parent's graph at cursor 1"
            );
            assert!(
                fork.journal().is_empty(),
                "the fork's journal starts at the fork point"
            );
            let provenance = fork.journal().provenance().unwrap().clone();
            assert_eq!(provenance.at, Seq(1));
            assert_eq!(
                fork.changes()[0].kind,
                ChangeKind::Forked {
                    from: parent_id,
                    at: Seq(1)
                }
            );

            fork.apply(knot(), vec![add(3)]).await.unwrap();
            assert_eq!(fork.graph().node_count(), 2);
            assert_eq!(parent.graph().node_count(), 2, "the parent is untouched");
            assert!(parent.graph().get_node_by_id(Uuid::from_u128(3)).is_none());
            assert!(mere.fork_at(&parent, Seq(9), knot()).await.is_err());
        });
    }

    #[test]
    fn undo_steps_back_through_an_author_s_own_changes_and_redo_returns() {
        pollster::block_on(async {
            let mere = MereSessions::new(MemoryBackend::new());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            session.apply(person(), vec![add(1)]).await.unwrap();
            session.apply(knot(), vec![add(2)]).await.unwrap();
            session
                .apply(person(), vec![retitle(1, "One")])
                .await
                .unwrap();
            let title = |session: &GraphSession<MemoryBackend>| {
                session
                    .graph()
                    .get_node_by_id(Uuid::from_u128(1))
                    .map(|(_, node)| node.title.clone())
            };

            let undone = session.undo(person()).await.unwrap().unwrap();
            assert!(undone.kept.is_empty());
            assert_ne!(title(&session).as_deref(), Some("One"));
            session.undo(person()).await.unwrap().unwrap();
            assert!(session.graph().get_node_by_id(Uuid::from_u128(1)).is_none());
            assert!(
                session.graph().get_node_by_id(Uuid::from_u128(2)).is_some(),
                "the other author's change stays"
            );
            assert!(
                session.undo(person()).await.unwrap().is_none(),
                "nothing left to undo"
            );

            session.redo(person()).await.unwrap().unwrap();
            assert!(session.graph().get_node_by_id(Uuid::from_u128(1)).is_some());
            session.redo(person()).await.unwrap().unwrap();
            assert_eq!(title(&session).as_deref(), Some("One"));
            assert!(session.redo(person()).await.unwrap().is_none());
            assert!(
                session.redo(knot()).await.unwrap().is_none(),
                "stacks are per author"
            );
        });
    }

    #[test]
    fn undo_keeps_what_another_author_changed_and_says_who() {
        pollster::block_on(async {
            let mere = MereSessions::new(MemoryBackend::new());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            session
                .apply(person(), vec![add(1), retitle(1, "Start")])
                .await
                .unwrap();
            session
                .apply(person(), vec![retitle(1, "Mine"), tag(1, "mine")])
                .await
                .unwrap();
            session
                .apply(knot(), vec![retitle(1, "Theirs")])
                .await
                .unwrap();

            let undone = session.undo(person()).await.unwrap().unwrap();
            assert_eq!(
                undone.kept,
                [Kept {
                    part: Part::Title(Uuid::from_u128(1)),
                    by: vec![knot()],
                }]
            );
            let (_, node) = session.graph().get_node_by_id(Uuid::from_u128(1)).unwrap();
            assert_eq!(node.title, "Theirs", "their later edit stays");
            assert!(
                !node.tags.contains("mine"),
                "the rest of the change went back"
            );
            assert_eq!(
                session.changes().last().unwrap().kind,
                ChangeKind::Undo { of: Seq(2) }
            );
        });
    }

    #[test]
    fn undo_and_redo_order_survives_a_reopen() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            session.apply(person(), vec![add(1)]).await.unwrap();
            session.undo(person()).await.unwrap().unwrap();
            drop(session);

            let mut reopened = mere.open(id).await.unwrap();
            assert!(
                reopened
                    .graph()
                    .get_node_by_id(Uuid::from_u128(1))
                    .is_none()
            );
            reopened.redo(person()).await.unwrap().unwrap();
            assert!(
                reopened
                    .graph()
                    .get_node_by_id(Uuid::from_u128(1))
                    .is_some()
            );
        });
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn a_component_fork_takes_only_the_seed_s_component() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut parent = mere.open(id).await.unwrap();
            parent
                .apply(person(), vec![add(1), add(2), add(3), relate(1, 2)])
                .await
                .unwrap();
            let fork = mere
                .fork_component(&parent, Uuid::from_u128(1), knot())
                .await
                .unwrap();
            let fork = mere.open(fork.session_id).await.unwrap();
            assert_eq!(
                fork.graph().node_count(),
                2,
                "nodes 1 and 2, not the lone 3"
            );
            assert!(
                mere.fork_component(&parent, Uuid::from_u128(9), knot())
                    .await
                    .is_err()
            );
        });
    }

    #[test]
    fn trash_marks_the_manifest_and_restore_clears_it() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            let at = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
            session.trash(knot(), at).await.unwrap();

            let listed = mere.list().await.unwrap();
            assert_eq!(listed.len(), 1);
            let mark = listed[0].trashed.clone().expect("trashed");
            assert_eq!(mark.by, knot());
            assert_eq!(mark.at, at);

            session.restore(person(), at).await.unwrap();
            assert!(mere.list().await.unwrap()[0].trashed.is_none());
            let kinds: Vec<&ChangeKind> = session.changes().iter().map(|c| &c.kind).collect();
            assert_eq!(
                kinds,
                [
                    &ChangeKind::Minted,
                    &ChangeKind::Trashed,
                    &ChangeKind::Restored
                ]
            );
            // The session still opens: trash never moved its keys.
            assert_eq!(mere.open(id).await.unwrap().changes().len(), 3);
        });
    }

    #[test]
    fn a_session_begun_in_memory_is_stored_whole_by_its_first_flush() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mut baseline = Graph::new();
            replay_captured_deltas_onto(&mut baseline, [add(1), retitle(1, "Kept")]);
            let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
            let id = manifest.session_id;
            let mut session = GraphSession::new(
                store.clone(),
                manifest,
                Some(baseline),
                person(),
                ChangeKind::Minted,
            );
            let (key, applied) = session.edit_now(person(), |graph| {
                kernel::graph::apply::add_node(
                    graph,
                    Some(Uuid::from_u128(2)),
                    "https://2.test/".into(),
                    Point2D::new(0.0, 0.0),
                )
            });
            assert!(session.graph().get_node(key).is_some());
            assert_eq!(applied.first, Seq(0));
            assert!(
                store.list(SESSIONS_PREFIX).await.unwrap().is_empty(),
                "nothing is written before the flush"
            );

            let at = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
            session.flush(at).await.unwrap();
            let reopened = GraphSession::open(store, id).await.unwrap();
            assert_eq!(whole(reopened.graph()), whole(session.graph()));
            assert_eq!(reopened.changes(), session.changes());
            assert_eq!(reopened.manifest().updated_at, at);
            assert!(
                session.pending(at).unwrap().is_empty(),
                "a flush with nothing new writes nothing"
            );
        });
    }

    #[test]
    fn a_batch_that_never_commits_stays_pending_until_one_does() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
            let id = manifest.session_id;
            let mut session =
                GraphSession::new(store.clone(), manifest, None, person(), ChangeKind::Minted);
            session.apply_now(person(), vec![add(1)]).unwrap();
            let at = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
            // A host's batch took these writes and then failed to commit.
            let lost = session.pending(at).unwrap();
            assert!(!lost.is_empty());
            session
                .apply_now(knot(), vec![retitle(1, "Later")])
                .unwrap();

            let retry = session.pending(at).unwrap();
            assert!(
                retry.ops().len() > lost.ops().len(),
                "the retry carries the lost batch and what followed"
            );
            store.apply(retry.ops()).await.unwrap();
            session.stored(retry);
            assert!(session.pending(at).unwrap().is_empty());

            let reopened = GraphSession::open(store, id).await.unwrap();
            assert_eq!(whole(reopened.graph()), whole(session.graph()));
            assert_eq!(reopened.journal().len(), session.journal().len());
        });
    }

    #[test]
    fn the_latest_live_session_is_the_one_changed_last() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let first = mere.mint(person(), None).await.unwrap().session_id;
            let second = mere.mint(person(), None).await.unwrap().session_id;
            let latest = || async { mere.latest_live().await.unwrap().map(|m| m.session_id) };
            let base = wall_clock_now() + std::time::Duration::from_secs(60);
            let at = |seconds| base + std::time::Duration::from_secs(seconds);

            let mut one = mere.open(first).await.unwrap();
            one.apply_now(person(), vec![add(1)]).unwrap();
            one.flush(at(0)).await.unwrap();
            let mut two = mere.open(second).await.unwrap();
            two.apply_now(person(), vec![add(2)]).unwrap();
            two.flush(at(1)).await.unwrap();
            assert_eq!(latest().await, Some(second));

            one.apply_now(person(), vec![retitle(1, "Again")]).unwrap();
            one.flush(at(2)).await.unwrap();
            assert_eq!(latest().await, Some(first), "a stored edit updates");

            one.flush(at(3)).await.unwrap();
            assert_eq!(
                one.manifest().updated_at,
                at(2),
                "a flush with nothing new leaves updated_at alone"
            );

            one.trash(person(), at(4)).await.unwrap();
            assert_eq!(
                latest().await,
                Some(second),
                "a trashed session is not live"
            );
        });
    }

    #[test]
    fn a_view_keeps_its_own_stream_scrubbable_by_journal_cursor() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            let key = ViewKey::new("knot-editor", "main").unwrap();
            let spectral = ViewIntent {
                strategy: Some("spectral".into()),
                ..ViewIntent::default()
            };
            session
                .set_view(knot(), key.clone(), spectral.clone())
                .await
                .unwrap();
            session.apply(person(), vec![add(1)]).await.unwrap();
            let focused = ViewIntent {
                focus: Some(Uuid::from_u128(1).to_string()),
                ..spectral.clone()
            };
            session
                .set_view(knot(), key.clone(), focused.clone())
                .await
                .unwrap();

            assert_eq!(session.view(&key), Some(&focused));
            assert_eq!(session.view_at(&key, Seq(0)), Some(&spectral));
            assert_eq!(session.view_at(&key, Seq(2)), Some(&focused));
            assert_eq!(
                session.journal().len(),
                2,
                "view changes stay out of the journal"
            );
            drop(session);

            let reopened = mere.open(id).await.unwrap();
            assert_eq!(reopened.view(&key), Some(&focused));
            assert_eq!(reopened.view_at(&key, Seq(0)), Some(&spectral));
            assert!(ViewKey::new("Knot", "main").is_err());
            assert!(ViewKey::new("knot", "../main").is_err());
        });
    }

    #[test]
    fn an_edit_without_a_replay_form_is_refused_whole() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = MereSessions::new(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            let legacy = CapturedDelta::ReplaySetNodeThumbnailById {
                node_id: Uuid::from_u128(1).to_string(),
                png_bytes: Vec::new(),
                width: 0,
                height: 0,
            };
            assert!(session.apply(person(), vec![add(1), legacy]).await.is_err());
            assert_eq!(
                session.graph().node_count(),
                0,
                "nothing of the batch applied"
            );
            assert!(session.journal().is_empty());
        });
    }

    #[test]
    fn a_session_opens_only_if_its_mere_holds_it() {
        pollster::block_on(async {
            let mere = MereSessions::new(MemoryBackend::new());
            let missing = SessionId::new();
            assert!(matches!(
                mere.open(missing).await,
                Err(SessionError::Missing(id)) if id == missing
            ));
        });
    }

    #[test]
    fn in_process_hosts_apply_graph_deltas_by_key() {
        pollster::block_on(async {
            let mere = MereSessions::new(MemoryBackend::new());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open(id).await.unwrap();
            session
                .apply_deltas(
                    person(),
                    vec![GraphDelta::AddNode {
                        id: Some(Uuid::from_u128(7)),
                        url: "https://seven.test/".into(),
                        position: Point2D::new(1.0, 2.0),
                    }],
                )
                .await
                .unwrap();
            let (key, _) = session.graph().get_node_by_id(Uuid::from_u128(7)).unwrap();
            let result = session
                .apply_deltas(
                    person(),
                    vec![GraphDelta::SetNodeTitle {
                        key,
                        title: "Seven".into(),
                    }],
                )
                .await
                .unwrap();
            assert_eq!(result.first, Seq(4));
            assert_eq!(result.end, Seq(5));
            assert_eq!(session.graph().get_node(key).unwrap().title, "Seven");
            let resource_id = kernel::graph::ResourceNode::new("https://seven.test/").id();
            assert_eq!(session.graph().shown_resource_id(key), Some(resource_id));
            let [added, visit, resource, shown, title] = session.journal().entries() else {
                panic!("expected surface, visit, resource, shown binding and title");
            };
            let surface_id = Uuid::from_u128(7).to_string();
            let resource_id_text = resource_id.to_string();
            assert!(
                matches!(&added.delta, CapturedDelta::ReplayAddNodeWithIdIfMissing {
                id, url, ..
            } if id == &surface_id && url == "https://seven.test/")
            );
            assert!(
                matches!(&visit.delta, CapturedDelta::ReplayTouchNodeLastVisitedById {
                node_id, ..
            } if node_id == &surface_id)
            );
            assert!(
                matches!(&resource.delta, CapturedDelta::ReplaySetResourceRecordById {
                resource_id, record: Some(record)
            } if resource_id == &resource_id_text
                && record.canonical_iri == "https://seven.test")
            );
            assert!(
                matches!(&shown.delta, CapturedDelta::ReplaySetShownResourceById {
                surface_id: captured_surface, resource_id: Some(captured_resource)
            } if captured_surface == &surface_id && captured_resource == &resource_id_text)
            );
            assert!(
                matches!(&title.delta, CapturedDelta::ReplaySetNodeTitleById {
                node_id, title
            } if node_id == &surface_id && title == "Seven")
            );
            assert!(
                session
                    .journal()
                    .entries()
                    .iter()
                    .all(|entry| entry.author == person())
            );
            drop(session);
            let reopened = mere.open(id).await.unwrap();
            let (key, node) = reopened.graph().get_node_by_id(Uuid::from_u128(7)).unwrap();
            assert_eq!(node.title, "Seven");
            assert_eq!(reopened.graph().shown_resource_id(key), Some(resource_id));
        });
    }
    #[test]
    fn assertion_author_version_and_source_survive_session_replay() {
        use kernel::graph::{SemanticStatementSpec, SemanticSubKind, predicate_iri};
        pollster::block_on(async {
            let mere = MereSessions::new(MemoryBackend::default());
            let manifest = mere.mint(person(), None).await.unwrap();
            let mut session = mere.open(manifest.session_id).await.unwrap();
            session.apply(person(), vec![add(1), add(2)]).await.unwrap();
            let claim = |graph: &mut Graph, time, source: Option<&str>| {
                let from = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
                let to = graph.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
                graph
                    .assert_semantic_statement(
                        from,
                        to,
                        SemanticStatementSpec {
                            predicate: predicate_iri(SemanticSubKind::Cites).into(),
                            recognized_sub_kind: Some(SemanticSubKind::Cites),
                            provenance_iri: source.map(str::to_owned),
                            asserted_at_ms: Some(time),
                            ..Default::default()
                        },
                    )
                    .unwrap()
                    .1
                    .statement_id
            };
            let (first, _) = session.edit_now(Author::engine("extractor", "1"), |graph| {
                claim(graph, 10, None)
            });
            let (second, _) = session.edit_now(Author::engine("extractor", "2"), |graph| {
                claim(graph, 20, None)
            });
            assert_eq!(
                first, second,
                "new engine version updates its own assertion"
            );
            session.edit_now(Author::engine("extractor", "2"), |graph| {
                claim(graph, 30, Some("https://page.test/"))
            });
            let graph = session.graph();
            let from = graph.get_node_key_by_id(Uuid::from_u128(1)).unwrap();
            let to = graph.get_node_key_by_id(Uuid::from_u128(2)).unwrap();
            let statements = graph
                .get_edge(graph.find_edge_key(from, to).unwrap())
                .unwrap()
                .semantic_statements();
            assert_eq!(statements.len(), 2);
            assert_eq!(
                statements[0].provenance_iri,
                Some(Author::engine("extractor", "1").asserter_iri())
            );
            assert_eq!(statements[0].asserted_at_ms, Some(20));
            assert_eq!(
                statements[1].provenance_iri.as_deref(),
                Some("https://page.test/")
            );
            let live = serde_json::to_value(graph.to_snapshot()).unwrap();
            let replayed = session.graph_at(session.journal().live_cursor()).unwrap();
            assert_eq!(serde_json::to_value(replayed.to_snapshot()).unwrap(), live);
        });
    }
}
