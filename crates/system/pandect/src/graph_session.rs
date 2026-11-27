// Copyright 2026 Mark Alan Boykin
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.
// SPDX-License-Identifier: MPL-2.0

//! A mere's sessions: their schema over one muniment store, the live session
//! ([`GraphSession`]) and the lifecycle ([`MereSessions`]).
//!
//! Reservoir plan V2 (`design_docs/archive_docs/2026-10-28/2026-09-23_reservoir_plan.md`).
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
use std::sync::{Arc, Mutex, PoisonError};
use std::time::SystemTime;

use catchment::{GraphId, SessionId};
use mere::model::author::Author;
use mere::model::clock::wall_clock_now;
use mere::model::Attributed;
use mere::session::{ReplayPoint, SessionGraph, SessionJournal};
use muniment::journal::DigestChain;
use muniment::{
    Backend, Hash, Journal, JsonCodec, JsonSlots, LogId, Provenance, Seq, StoreError, WriteOp,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use uuid::Uuid;

use crate::facet_store::{NODE_FACETS_FILE, NodeFacetStore};
use crate::graph_placement::{
    FrozenGraph, LegacyTranslationReceipt, PlacementProfile, SessionPlacement, source_digest,
};
use crate::manifest::{GraphSessionManifest, Lineage, MergedMark, TrashMark};
use crate::view_state::ViewState;

/// Product-owned validation of a complete candidate before graph/history installation.
pub type GraphValidator<G> = Arc<dyn Fn(&G) -> Result<(), String> + Send + Sync>;

/// Owner policy for newly introduced references. Unlike replay validation, this
/// sees the previously admitted graph and may consult current owner authority.
pub type GraphAdmissionValidator<G> =
    Arc<dyn Fn(&G, &G) -> Result<(), String> + Send + Sync>;

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

mod page_claims;
pub use legacy_hook::LegacyHook;
mod legacy_hook;
mod publication;
pub use publication::{
    Publication, PublicationHook, PublicationWrites, PublishedGeneration, SessionBackend,
};

/// What a session's host installs on it: how it publishes its graph and how
/// it reads a stored legacy translation (engine split D252, D257). The
/// `_with` constructors install what their [`MereSessions`] carries, and a
/// bare one none.
struct SessionHooks<G: SessionGraph> {
    legacy: Option<&'static LegacyHook<G>>,
    publication: Option<&'static PublicationHook<G>>,
    /// The schema ids and redaction of the graph's codicils (engine split
    /// D265, D266).
    codicil: Option<&'static crate::graph_codicil::CodicilHook<G>>,
}

impl<G: SessionGraph> Clone for SessionHooks<G> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<G: SessionGraph> Copy for SessionHooks<G> {}

impl<G: SessionGraph> SessionHooks<G> {
    const NONE: Self = Self {
        legacy: None,
        publication: None,
        codicil: None,
    };
}

const MANIFEST: &str = "manifest.json";
const BASELINE: &str = "baseline.json";
const PLACEMENT: &str = "placement.json";
const TRANSLATION: &str = "legacy-translation.json";
const PENDING_LINKS: &str = "pending-links.json";
/// A merged session's mark (merge-in note, M6), kept beside its manifest so a
/// target's undo of the merge can clear it without reading the source first.
/// It is the authority for [`GraphSessionManifest::merged`] whenever a session
/// is read.
const MERGED: &str = "merged.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PendingLinksSlot<H> {
    version: u32,
    state: H,
}
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
    ReadOnly(SessionId),
    /// This handle only observes the session; another holds its one writer.
    Observing(SessionId),
    Validation(String),
    Store(StoreError),
    /// A merge-in cannot go ahead: the source is not a branch or picked-off
    /// part of the target, was already merged, or changed since the plan.
    NotMergeable(String),
    /// A pending merge was applied without a choice for every conflict.
    Unresolved(usize),
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
            Self::ReadOnly(id) => write!(
                f,
                "session {} is read-only (a codicil thaw, or merged); fork or branch before editing",
                id.as_uuid()
            ),
            Self::Observing(id) => write!(
                f,
                "session {} is open here to view only; edit it where its writer is open",
                id.as_uuid()
            ),
            Self::Validation(reason) => write!(f, "candidate graph refused: {reason}"),
            Self::Store(error) => write!(f, "store: {error}"),
            Self::NotMergeable(why) => write!(f, "cannot merge in: {why}"),
            Self::Unresolved(count) => write!(f, "{count} merge conflicts have no choice"),
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
    Archived {
        codicil: String,
    },
    OpenedCodicil {
        codicil: String,
    },
    /// Picked off session `from` at its journal cursor `at`: the component
    /// around one node, as a new session. Not a fork.
    Picked {
        from: SessionId,
        at: Seq,
    },
    /// Branched from session `from` at its journal cursor `at`: a copy of the
    /// whole session meant to merge back into it.
    Branched {
        from: SessionId,
        at: Seq,
    },
    /// Merged in session `from`, a branch or picked-off part, through its
    /// journal cursor `through`. The change carries the merge's edits, so
    /// undo takes the whole merge back in one step.
    MergedIn {
        from: SessionId,
        through: Seq,
    },
    /// This session was merged into session `into` through this session's
    /// own journal cursor `through`. It is read-only from then on.
    MergedInto {
        into: SessionId,
        through: Seq,
    },
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
    pub state: ViewState,
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
pub struct Reverted<G: SessionGraph> {
    /// The change undone, or the undo redone, as an index into the change log.
    pub of: Seq,
    pub applied: Applied,
    pub kept: Vec<Kept<G>>,
}

/// A part an undo or redo left as it is, and who changed it since.
pub struct Kept<G: SessionGraph> {
    pub part: G::Part,
    pub by: Vec<Author>,
}

// By hand, so they ask nothing of the graph type itself (D253).
impl<G: SessionGraph> Clone for Reverted<G> {
    fn clone(&self) -> Self {
        Self {
            of: self.of,
            applied: self.applied,
            kept: self.kept.clone(),
        }
    }
}

impl<G: SessionGraph> std::fmt::Debug for Reverted<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Reverted")
            .field("of", &self.of)
            .field("applied", &self.applied)
            .field("kept", &self.kept)
            .finish()
    }
}

impl<G: SessionGraph> PartialEq for Reverted<G> {
    fn eq(&self, other: &Self) -> bool {
        (self.of, self.applied) == (other.of, other.applied) && self.kept == other.kept
    }
}

impl<G: SessionGraph> Clone for Kept<G> {
    fn clone(&self) -> Self {
        Self {
            part: self.part.clone(),
            by: self.by.clone(),
        }
    }
}

impl<G: SessionGraph> std::fmt::Debug for Kept<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kept")
            .field("part", &self.part)
            .field("by", &self.by)
            .finish()
    }
}

impl<G: SessionGraph> PartialEq for Kept<G> {
    fn eq(&self, other: &Self) -> bool {
        self.part == other.part && self.by == other.by
    }
}

#[derive(Serialize, Deserialize)]
struct Baseline<S> {
    graph: S,
    facets: NodeFacetStore,
}

#[derive(Serialize, Deserialize)]
struct Checkpoint {
    cursor: Seq,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    translated_receipt_digest: Option<String>,
    /// The muniment digest (hex) of the journal's first `cursor` entries: what
    /// ties the checkpoint to the journal it was written beside.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    journal_digest: Option<String>,
    /// Which muniment journal digest `journal_digest` is: `None` is v1
    /// (`prefix_digest`), as every checkpoint before engine-split D154 wrote
    /// it; 2 is the chained v2 every checkpoint writes now.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    journal_digest_version: Option<u8>,
    /// The muniment hash (hex) of the stored bytes of `graph.json` and
    /// `facets.json` as this checkpoint wrote them: what ties the checkpoint to
    /// the files it summarizes with. Open reads a checkpoint only if it names
    /// both; one that names neither, or that the journal or the files no longer
    /// match, is a cache the journal outranks and is replayed past. (A session
    /// with a retained translation compares the files with it instead.)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    contents_digest: Option<String>,
}

/// The version of the journal digest a checkpoint writes.
const JOURNAL_DIGEST_VERSION: u8 = 2;

/// The v2 digest a checkpoint at `cursor` stores for `journal`: a lookup in
/// the session's chain, once the chain has taken the entries since it last
/// looked.
fn journal_digest<G: SessionGraph>(
    digests: &Mutex<DigestChain>,
    journal: &SessionJournal<G>,
    cursor: Seq,
) -> Result<Option<String>, SessionError> {
    let mut chain = digests.lock().unwrap_or_else(PoisonError::into_inner);
    chain.catch_up(journal.log())?;
    Ok(chain.at(cursor.index()).map(|digest| digest.to_hex()))
}

/// Whether `checkpoint` names `journal`'s first `cursor` entries, by the
/// digest version it was written with: a v1 record is checked by recomputing
/// v1, and an unknown version names nothing.
fn names_journal<G: SessionGraph>(
    checkpoint: &Checkpoint,
    digests: &Mutex<DigestChain>,
    journal: &SessionJournal<G>,
) -> Result<bool, SessionError> {
    let Some(stored) = &checkpoint.journal_digest else {
        return Ok(false);
    };
    let digest = match checkpoint.journal_digest_version {
        None => journal
            .log()
            .prefix_digest(checkpoint.cursor.index())?
            .map(|digest| digest.to_hex()),
        Some(JOURNAL_DIGEST_VERSION) => journal_digest(digests, journal, checkpoint.cursor)?,
        Some(_) => None,
    };
    Ok(digest.as_ref() == Some(stored))
}

/// The hash a checkpoint stores for the bytes of its graph and facets files.
fn contents_digest(graph: &[u8], facets: &[u8]) -> String {
    let mut framed = Vec::with_capacity(graph.len() + facets.len() + 16);
    for bytes in [graph, facets] {
        framed.extend((bytes.len() as u64).to_le_bytes());
        framed.extend(bytes);
    }
    Hash::of(&framed).to_hex()
}

/// The three records of a checkpoint at `cursor`: the graph, its facets, and the
/// record naming the journal prefix and the exact bytes of the other two.
fn checkpoint_ops<G: SessionGraph>(
    keys: &Keys,
    graph: &G,
    digests: &Mutex<DigestChain>,
    journal: &SessionJournal<G>,
    cursor: Seq,
    translated_receipt_digest: Option<String>,
) -> Result<[WriteOp; 3], SessionError> {
    let graph_bytes = pretty_bytes(&graph.to_snapshot())?;
    let facets_bytes = pretty_bytes(graph.facets())?;
    let record = Checkpoint {
        cursor,
        translated_receipt_digest,
        journal_digest: journal_digest(digests, journal, cursor)?,
        journal_digest_version: Some(JOURNAL_DIGEST_VERSION),
        contents_digest: Some(contents_digest(&graph_bytes, &facets_bytes)),
    };
    Ok([
        WriteOp::Put {
            key: keys.at(GRAPH),
            value: graph_bytes,
        },
        WriteOp::Put {
            key: keys.at(NODE_FACETS_FILE),
            value: facets_bytes,
        },
        pretty(keys.at(CHECKPOINT), &record)?,
    ])
}

/// Whether the stored graph and facets files are the ones `checkpoint` wrote.
async fn stored_contents_match<B: Backend>(
    slots: &JsonSlots<B>,
    keys: &Keys,
    checkpoint: &Checkpoint,
) -> Result<bool, SessionError> {
    let Some(expected) = &checkpoint.contents_digest else {
        return Ok(false);
    };
    let Some(graph) = slots.backend().get(&keys.at(GRAPH)).await? else {
        return Ok(false);
    };
    let facets = slots
        .backend()
        .get(&keys.at(NODE_FACETS_FILE))
        .await?
        .unwrap_or_default();
    Ok(contents_digest(&graph, &facets) == *expected)
}

#[derive(Clone, Default)]
struct View {
    current: ViewState,
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

fn pretty_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, SessionError> {
    Ok(serde_json::to_vec_pretty(value).map_err(|error| StoreError::Codec(error.to_string()))?)
}

fn pretty<T: Serialize>(key: String, value: &T) -> Result<WriteOp, SessionError> {
    Ok(WriteOp::Put {
        key,
        value: pretty_bytes(value)?,
    })
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
fn graph_of<G: SessionGraph>(
    snapshot: &G::Snapshot,
    facets: NodeFacetStore,
    recorded: bool,
) -> Result<G, String> {
    let mut graph = G::from_snapshot(snapshot, recorded)?;
    graph.install_facets(facets)?;
    Ok(graph)
}

fn baseline_of<G: SessionGraph>(graph: &G) -> Baseline<G::Snapshot> {
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

async fn load_retained_source<B: Backend, G: SessionGraph>(
    slots: &JsonSlots<B>,
    keys: &Keys,
    manifest: &GraphSessionManifest,
    recorded: bool,
) -> Result<(G, SessionJournal<G>), SessionError> {
    let baseline = slots.backend().get(&keys.at(BASELINE)).await?;
    let journal_prefix = keys.at(JOURNAL);
    let entry_keys = slots
        .backend()
        .scan(&format!("{journal_prefix}/"), &format!("{journal_prefix}0"))
        .await?;
    let mut writes = Vec::new();
    for key in entry_keys {
        let value = slots
            .backend()
            .get(&key)
            .await?
            .ok_or_else(|| SessionError::Corrupt(format!("{key}: vanished retained entry")))?;
        writes.push(WriteOp::Put { key, value });
    }
    let source = muniment::MemoryBackend::new();
    source.apply(&writes).await?;
    let journal = SessionJournal::from_log(
        Journal::<Attributed<G::Delta>>::load_entries(
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
            let baseline: Baseline<G::Snapshot> =
                serde_json::from_slice(bytes).map_err(|error| {
                    SessionError::Corrupt(format!("{}: {error}", keys.at(BASELINE)))
                })?;
            let mut graph = G::from_snapshot(&baseline.graph, recorded).map_err(|error| {
                SessionError::Corrupt(format!("{}: {error}", keys.at(BASELINE)))
            })?;
            graph.install_facets(baseline.facets).map_err(|error| {
                SessionError::Corrupt(format!("{}: {error}", keys.at(BASELINE)))
            })?;
            Ok::<_, SessionError>(graph)
        })
        .transpose()?
        .unwrap_or_default();
    Ok((graph, journal))
}

/// The provenance a session's journal was recorded with: a fork's starts at
/// its parent's cursor.
fn provenance(manifest: &GraphSessionManifest) -> Option<Provenance> {
    Some(Provenance {
        source: Some(LogId::new(manifest.parent_session?.as_uuid().to_string())),
        at: Seq(manifest.forked_at?),
    })
}

/// Whether the replayed clone and the candidate have equal replay states.
/// Both are clones of the live graph, so only the entries either wrote are
/// compared, as bytes (engine split D164).
fn replay_matches<G: SessionGraph>(replay: &G, candidate: &G) -> Result<bool, SessionError> {
    #[cfg(test)]
    if std::env::var_os("PANDECT_REPLAY_CHECK_OFF").is_some() {
        // The control for the escape tests: with no check they must fail.
        return Ok(true);
    }
    G::replay_matches(replay, candidate).map_err(SessionError::NotReplayable)
}

/// Route what `graph` records into `buffer`, for the session to journal.
fn record_into<G: SessionGraph>(graph: &mut G, buffer: &Arc<Mutex<Vec<G::Delta>>>) {
    let buffer = Arc::clone(buffer);
    graph.set_recorder(Some(Arc::new(move |delta: &G::Delta| {
        buffer.lock().expect("recorder buffer").push(delta.clone());
    })));
}

/// What a session has not stored yet, as one batch, and what storing it
/// covers. A host that writes through a batch of its own applies
/// [`ops`](Self::ops) there and hands this back to [`GraphSession::stored`]
/// once that batch commits.
pub struct Pending<G: SessionGraph> {
    ops: Vec<WriteOp>,
    journal: Seq,
    changes: Seq,
    checkpoint: Option<Seq>,
    updated_at: Option<SystemTime>,
    /// Views written, with the log position each was written through.
    views: Vec<(ViewKey, Seq)>,
    pending_links: Option<(u64, G::Held)>,
    /// How many queued merged marks the batch writes.
    merge_marks: usize,
}

impl<G: SessionGraph> std::fmt::Debug for Pending<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Pending")
            .field("ops", &self.ops)
            .field("journal", &self.journal)
            .field("changes", &self.changes)
            .field("checkpoint", &self.checkpoint)
            .field("updated_at", &self.updated_at)
            .field("views", &self.views)
            .field("pending_links", &self.pending_links)
            .field("merge_marks", &self.merge_marks)
            .finish()
    }
}

impl<G: SessionGraph> Pending<G> {
    /// Commit batches in creation order. Storage receipts may arrive later.
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
pub struct GraphSession<B, G: SessionGraph> {
    slots: JsonSlots<B>,
    validator: Option<GraphValidator<G>>,
    admission_validator: Option<GraphAdmissionValidator<G>>,
    keys: Keys,
    manifest: GraphSessionManifest,
    baseline: G,
    placement: Option<SessionPlacement>,
    translation: Option<LegacyTranslationReceipt<G>>,
    /// The host's hooks: the legacy translation's reads over `G`, and the
    /// publication of `G` (engine split D252, D257).
    hooks: SessionHooks<G>,
    graph: G,
    journal: SessionJournal<G>,
    /// Every journal prefix's v2 digest (engine-split D154), built at open and
    /// caught up whenever a checkpoint names a prefix, so naming one is a
    /// lookup. Behind a lock so a batch built through `&self` can catch it up.
    digests: Mutex<DigestChain>,
    pending: Arc<Mutex<Vec<G::Delta>>>,
    changes: Journal<Change>,
    views: BTreeMap<ViewKey, View>,
    saved: Seq,
    changes_saved: Seq,
    checkpointed: Seq,
    /// Whether the stored checkpoint was read with a v1 journal digest: the
    /// next [`checkpoint_if_behind`](Self::checkpoint_if_behind) rewrites it
    /// as v2 even when the journal has not moved.
    checkpoint_v1: bool,
    /// Where a baseline replay stands at the stored checkpoint, which
    /// [`graph_at`](Self::graph_at) starts from (engine-split D161). A cache:
    /// shared with a detached candidate, and none for a legacy translation.
    replay_point: Option<Arc<HeldPoint<G>>>,
    checkpoint_interval: u64,
    revision: u64,
    /// Whether the manifest and baseline are in the store yet.
    head_stored: bool,
    /// Whether the baseline is written with the head: a session begun from
    /// no graph stores none, and opens empty.
    write_baseline: bool,
    saved_pending_links: G::Held,
    saved_pending_links_revision: u64,
    retained_pending_links_revision: u64,
    /// Merged marks to write on other sessions: an undone merge clears its
    /// source's, a redone one sets it again.
    merge_marks: Vec<(SessionId, Option<MergedMark>)>,
    /// Whether this handle only observes: every edit and write refuses, and
    /// nothing records it, so the stored session never learns of it.
    observing: bool,
}

#[path = "graph_session_history.rs"]
mod history;
use history::{HeldPoint, apply_all};
#[path = "graph_session_operation.rs"]
mod operation;
#[path = "graph_session_pin.rs"]
mod pin;
pub use pin::{
    GraphRevision, PinnedView, RESOURCE_DIGEST_ALGORITHM, RESOURCE_DIGEST_VERSION, ResourceDigest,
};
#[path = "graph_session_transaction.rs"]
mod transaction;
pub use operation::OperationReceipt;

#[path = "graph_session_merge.rs"]
mod merge;
pub use merge::{
    DroppedChoice, MergeChoice, MergeConflict, MergeMatch, Merged, PendingMerge, Refused,
};
pub use transaction::CommitExtra;

impl<B: Backend, G: SessionGraph> GraphSession<B, G> {
    /// Begin a new session over any graph in memory, from `baseline` or empty, with `kind` as
    /// its first change. Nothing is written until the first flush, which
    /// stores the manifest and baseline with everything since.
    pub fn new_with(
        backend: B,
        manifest: GraphSessionManifest,
        baseline: Option<G>,
        author: Author,
        kind: ChangeKind,
    ) -> Self {
        let id = LogId::new(manifest.session_id.as_uuid().to_string());
        let journal = match provenance(&manifest) {
            Some(provenance) => SessionJournal::starting_from(id, provenance),
            None => SessionJournal::with_id(id),
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
        let mut baseline = baseline.unwrap_or_default();
        let mut graph = baseline.clone();
        baseline.clear_semantic_context();
        let pending = Arc::default();
        record_into(&mut graph, &pending);
        Self {
            slots: JsonSlots::new(backend),
            validator: None,
            admission_validator: None,
            keys: Keys::new(manifest.session_id),
            manifest,
            baseline,
            placement,
            translation: None,
            hooks: SessionHooks::NONE,
            graph,
            journal,
            digests: Mutex::default(),
            pending,
            changes,
            views: BTreeMap::new(),
            saved: Seq(0),
            changes_saved: Seq(0),
            checkpointed: Seq(0),
            checkpoint_v1: false,
            replay_point: None,
            checkpoint_interval: DEFAULT_CHECKPOINT_INTERVAL,
            revision: 1,
            head_stored: false,
            write_baseline,
            saved_pending_links: Default::default(),
            saved_pending_links_revision: 0,
            retained_pending_links_revision: 1,
            merge_marks: Vec::new(),
            observing: false,
        }
    }

    /// [`new_recorded`](GraphSession::new_recorded) over any graph.
    pub fn new_recorded_with(
        backend: B,
        manifest: GraphSessionManifest,
        baseline: Option<G>,
        author: Author,
        kind: ChangeKind,
    ) -> Self {
        let mut session = Self::new_with(backend, manifest, baseline, author, kind);
        session.placement = Some(SessionPlacement::recorded());
        session
    }

    /// Open session `id` with no hooks: a session with a stored legacy
    /// translation is refused, since only a legacy hook reads one.
    pub async fn open_with(backend: B, id: SessionId) -> Result<Self, SessionError> {
        Self::open_inner(backend, id, SessionHooks::NONE).await
    }

    /// Open session `id` with `hooks`, reading a stored legacy translation
    /// through its legacy hook and refusing one without it (engine split
    /// D252).
    async fn open_inner(
        backend: B,
        id: SessionId,
        hooks: SessionHooks<G>,
    ) -> Result<Self, SessionError> {
        let legacy = hooks.legacy;
        let slots = JsonSlots::new(backend);
        let keys = Keys::new(id);
        let mut manifest: GraphSessionManifest = read(&slots, &keys.at(MANIFEST))
            .await?
            .ok_or(SessionError::Missing(id))?;
        manifest.merged = read(&slots, &keys.at(MERGED)).await?;
        let placement: Option<SessionPlacement> = read(&slots, &keys.at(PLACEMENT)).await?;
        let recorded = placement
            .as_ref()
            .is_some_and(|profile| profile.baseline == PlacementProfile::RecordedStrataV1);
        let (baseline, journal) =
            load_retained_source::<B, G>(&slots, &keys, &manifest, recorded).await?;
        let changes = Journal::load_entries(&slots, &keys.at(CHANGES), None, None).await?;
        let live = journal.live_cursor();
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

        // A stored receipt is read only through the host's legacy hook, and
        // refused before it is decoded without one.
        if legacy.is_none() && slots.backend().get(&keys.at(TRANSLATION)).await?.is_some() {
            return Err(SessionError::Corrupt(format!(
                "{}: a legacy translation opens only through its graph's legacy hook",
                keys.at(TRANSLATION)
            )));
        }
        let translation: Option<LegacyTranslationReceipt<G>> =
            read(&slots, &keys.at(TRANSLATION)).await?;
        let hook = legacy;
        match (&placement, &translation) {
            (Some(profile), Some(receipt))
                if profile.baseline == PlacementProfile::LegacySurfaceV1 =>
            {
                let digest = retained_source_digest(&slots, &keys, profile).await?;
                let hook = hook.expect("a receipt is read with its hook");
                receipt
                    .validate_with(hook, profile, &digest, journal.entries())
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
        // A checkpoint is read only beside the journal prefix and the files it
        // names. Any other is a cache the journal outranks: it is replayed past,
        // and the next checkpoint names what it was written with. A session with
        // a retained translation compares its checkpoint with that translation
        // below instead, and refuses a difference. A v1 digest is checked as
        // v1; the session's chain gives v2 from here on (engine-split D154).
        let digests = Mutex::new(DigestChain::of(journal.log())?);
        let checkpoint = match checkpoint {
            Some(checkpoint)
                if checkpoint.cursor <= live
                    && names_journal(&checkpoint, &digests, &journal)?
                    && (translation.is_some()
                        || stored_contents_match(&slots, &keys, &checkpoint).await?) =>
            {
                Some(checkpoint)
            },
            _ => None,
        };
        let checkpoint_v1 = checkpoint
            .as_ref()
            .is_some_and(|checkpoint| checkpoint.journal_digest_version.is_none());
        let (mut graph, checkpointed) = match checkpoint {
            Some(checkpoint)
                if checkpoint.cursor <= live
                    && translation.as_ref().is_none_or(|receipt| {
                        checkpoint.cursor >= receipt.placement.cutoff()
                            && checkpoint.translated_receipt_digest.as_deref()
                                == Some(receipt.translation_digest.as_str())
                    }) =>
            {
                let snapshot: G::Snapshot = read(&slots, &keys.at(GRAPH))
                    .await?
                    .ok_or_else(|| SessionError::Corrupt(keys.at(GRAPH)))?;
                let facets: NodeFacetStore = read(&slots, &keys.at(NODE_FACETS_FILE))
                    .await?
                    .unwrap_or_default();
                // A translated or recorded checkpoint materializes by the
                // recorded rules, as `FrozenGraph::materialize` does.
                let graph = graph_of::<G>(&snapshot, facets, translation.is_some() || recorded)
                    .map_err(|error| {
                        SessionError::Corrupt(format!("{}: {error}", keys.at(GRAPH)))
                    })?;
                if let Some(receipt) = &translation {
                    let hook = hook.expect("a receipt is read with its hook");
                    let expected = receipt
                        .graph_at(journal.entries(), checkpoint.cursor)
                        .map_err(SessionError::Corrupt)?
                        .ok_or_else(|| SessionError::Corrupt(keys.at(CHECKPOINT)))?;
                    if !FrozenGraph::of(&graph).same(&FrozenGraph::of(&expected), hook) {
                        return Err(SessionError::Corrupt(format!(
                            "{}: checkpoint differs from retained translation",
                            keys.at(CHECKPOINT)
                        )));
                    }
                }
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
        // The prefix replay that recovers attribution reaches the checkpoint's
        // replay point, which the session keeps for `graph_at`.
        let replay_point = if let Some(receipt) = &translation {
            let translated = receipt
                .journal(journal.entries(), live)
                .expect("receipt validated against journal");
            let baseline = receipt
                .baseline
                .materialize()
                .map_err(SessionError::Corrupt)?;
            translated
                .replay_from_with_baseline(
                    receipt.effect_cursor(checkpointed),
                    &mut graph,
                    &baseline,
                )
                .map(|()| None)
        } else {
            journal
                .replay_point_from(&baseline, checkpointed)
                .and_then(|point| {
                    let point = point.expect("an opened checkpoint lies within its journal");
                    journal.replay_from_point(&point, &mut graph)?;
                    Ok((checkpointed > Seq(0)).then_some(point))
                })
        }
        .map_err(|error| {
            SessionError::Corrupt(format!("{}: replayed graph: {error}", keys.at(JOURNAL)))
        })?;
        let replay_point = match replay_point {
            Some(point) => Some(Arc::new(HeldPoint {
                digest: journal_digest(&digests, &journal, point.cursor())?,
                point,
            })),
            None => None,
        };
        graph.validate_handles().map_err(|error| {
            SessionError::Corrupt(format!("{}: replayed graph: {error}", keys.at(JOURNAL)))
        })?;

        let saved_pending_links =
            match read::<B, PendingLinksSlot<G::Held>>(&slots, &keys.at(PENDING_LINKS)).await? {
                Some(slot) if slot.version == 1 => slot.state,
                Some(slot) => {
                    return Err(SessionError::Corrupt(format!(
                        "{}: unsupported pending-links version {}",
                        keys.at(PENDING_LINKS),
                        slot.version,
                    )));
                },
                None => Default::default(),
            };
        let saved_pending_links = G::retained(saved_pending_links);
        graph.restore_held(saved_pending_links.clone());

        let mut session = Self {
            slots,
            validator: None,
            admission_validator: None,
            keys,
            manifest,
            baseline,
            placement,
            translation,
            hooks: SessionHooks {
                legacy: hook,
                ..hooks
            },
            graph,
            journal,
            digests,
            pending: Arc::default(),
            changes_saved: changes.next_seq(),
            changes,
            views: BTreeMap::new(),
            saved: live,
            checkpointed,
            checkpoint_v1,
            replay_point,
            checkpoint_interval: DEFAULT_CHECKPOINT_INTERVAL,
            revision: 1,
            head_stored: true,
            write_baseline: false,
            saved_pending_links,
            saved_pending_links_revision: 1,
            retained_pending_links_revision: 1,
            merge_marks: Vec::new(),
            observing: false,
        };
        session.load_views().await?;
        record_into(&mut session.graph, &session.pending);
        Ok(session)
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

    /// The store this session reads and writes, for a host's own stores over
    /// the same backend (engine split D268).
    pub fn backend(&self) -> &B {
        self.slots.backend()
    }

    /// Explicit baseline placement. Absence remains unqualified.
    pub fn placement(&self) -> Option<&SessionPlacement> {
        self.placement.as_ref()
    }

    /// The live graph. Edits go through [`apply`](Self::apply), so every one is
    /// journaled.
    /// Attach product authority, refusing an already invalid graph.
    pub fn with_validator(mut self, validator: GraphValidator<G>) -> Result<Self, SessionError> {
        validator(&self.graph).map_err(SessionError::Validation)?;
        self.validator = Some(validator);
        Ok(self)
    }

    /// Attach admission policy to an already admitted/replayed baseline. Pure
    /// replay uses `with_validator`; history does not require live referents.
    pub fn with_admission_validator(mut self, validator: GraphAdmissionValidator<G>) -> Self {
        self.admission_validator = Some(validator);
        self
    }

    fn validate(&self, graph: &G) -> Result<(), SessionError> {
        if let Some(validator) = &self.validator {
            validator(graph).map_err(SessionError::Validation)?;
        }
        if let Some(validator) = &self.admission_validator {
            validator(&self.graph, graph).map_err(SessionError::Validation)?;
        }
        Ok(())
    }

    /// Make this handle an observer: it reads the stored session and refuses
    /// every edit before it runs. One view of a profile writes; others observe.
    pub fn observe(mut self) -> Self {
        self.observing = true;
        self
    }

    pub fn is_observing(&self) -> bool {
        self.observing
    }

    pub fn ensure_writable(&self) -> Result<(), SessionError> {
        if self.observing {
            return Err(SessionError::Observing(self.id()));
        }
        if self.manifest.codicil_read_only || self.manifest.merged.is_some() {
            return Err(SessionError::ReadOnly(self.id()));
        }
        Ok(())
    }

    pub fn graph(&self) -> &G {
        &self.graph
    }

    /// Install the host's launch number: the graph stamps it as the
    /// `last_session_visited` of every node navigated from now on. Per-launch
    /// host wiring, not graph truth, so it is neither journaled nor compared;
    /// the candidates an edit clones carry it.
    pub fn set_current_session(&mut self, launch: u64) {
        self.graph.set_current_session(launch);
    }

    /// Install the host's browsing idle gap (engine-split D202): traversals
    /// recorded after a gap this long open a new browsing session. Host wiring
    /// like the launch number, neither journaled nor compared.
    pub fn set_browsing_idle_gap_ms(&mut self, idle_gap_ms: u64) {
        self.graph.set_browsing_idle_gap_ms(idle_gap_ms);
    }

    pub fn journal(&self) -> &SessionJournal<G> {
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
        edits: Vec<G::Delta>,
    ) -> Result<Applied, SessionError> {
        let applied = self.apply_now(author, edits)?;
        self.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(applied)
    }

    /// Apply edits as one change under `author`, journal what they did, and
    /// store it before returning. A change that alters nothing records
    /// nothing; one with an edit admission refuses commits nothing, and the
    /// error names that edit (engine-split D163).
    pub async fn apply_deltas(
        &mut self,
        author: Author,
        deltas: Vec<G::Edit>,
    ) -> Result<Applied, SessionError> {
        self.apply_as(author, ChangeKind::Edit, deltas).await
    }

    /// [`apply`](Self::apply), journaled but not stored until the next flush:
    /// for a host whose edits arrive on a synchronous path.
    pub fn apply_now(
        &mut self,
        author: Author,
        edits: Vec<G::Delta>,
    ) -> Result<Applied, SessionError> {
        self.ensure_writable()?;
        let deltas = edits
            .iter()
            .map(|edit| {
                G::edit_of(edit, &author)
                    .map_err(|why| SessionError::NotReplayable(format!("{why}: {edit:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(self
            .try_edit_as(author, ChangeKind::Edit, |graph| apply_all(graph, deltas))?
            .1)
    }

    /// Run `edit` on the live graph as one change under `author`, journaled
    /// but not stored until the next flush. Every writer the closure uses,
    /// `apply_graph_delta` or a public assertion or declaration writer, emits
    /// captured deltas that `edit_as` records, replays onto a fresh clone and
    /// compares, refusing a candidate that differs
    /// ([`SessionError::NotReplayable`]). That check is the write boundary;
    /// `GraphDelta` is one of its writers.
    pub fn edit_now<R>(
        &mut self,
        author: Author,
        edit: impl FnOnce(&mut G) -> R,
    ) -> Result<(R, Applied), SessionError> {
        self.ensure_writable()?;
        self.edit_as(author, ChangeKind::Edit, edit)
    }

    fn edit_as<R>(
        &mut self,
        author: Author,
        kind: ChangeKind,
        edit: impl FnOnce(&mut G) -> R,
    ) -> Result<(R, Applied), SessionError> {
        self.try_edit_as(author, kind, |graph| Ok(edit(graph)))
    }

    /// [`edit_as`](Self::edit_as) whose edit can refuse: an `Err` from it
    /// leaves the graph, journal and changes as they were.
    fn try_edit_as<R>(
        &mut self,
        author: Author,
        kind: ChangeKind,
        edit: impl FnOnce(&mut G) -> Result<R, SessionError>,
    ) -> Result<(R, Applied), SessionError> {
        self.ensure_writable()?;
        let observation = self.graph.observation_revision();
        let mut candidate = self.graph.clone();
        let captured = Arc::default();
        record_into(&mut candidate, &captured);
        // The graph's write runs with only its own recorder.
        let result = candidate.write_as(author.clone(), edit)?;
        self.validate(&candidate)?;
        let recorded = std::mem::take(&mut *captured.lock().expect("candidate recorder buffer"));
        let mut replay = self.graph.clone();
        replay.replay_as(&author, &recorded);
        if !replay_matches(&replay, &candidate)? {
            return Err(SessionError::NotReplayable(
                "candidate changed graph truth outside the journaled mutation path".into(),
            ));
        }
        if candidate.held() != self.graph.held() {
            self.retained_pending_links_revision += 1;
        }
        self.graph = candidate;
        record_into(&mut self.graph, &self.pending);
        let first = self.journal.live_cursor();
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
        } else if self.graph.observation_revision() != observation {
            self.revision += 1;
        }
        let applied = Applied {
            first,
            end,
            revision: self.revision,
        };
        Ok((result, applied))
    }

    async fn apply_as(
        &mut self,
        author: Author,
        kind: ChangeKind,
        deltas: Vec<G::Edit>,
    ) -> Result<Applied, SessionError> {
        self.ensure_writable()?;
        let (_, applied) = self.try_edit_as(author, kind, |graph| apply_all(graph, deltas))?;
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
    /// Whether anything waits for the next store, without building the batch.
    pub fn has_unstored(&self) -> bool {
        !self.head_stored
            || !self.merge_marks.is_empty()
            || self.retained_pending_links_revision > self.saved_pending_links_revision
            || self.graph.held() != self.saved_pending_links
            || self.journal.live_cursor() > self.saved
            || self.changes.next_seq() > self.changes_saved
            || self
                .views
                .values()
                .any(|view| view.log.next_seq() > view.saved)
    }

    pub fn pending(&self, at: SystemTime) -> Result<Pending<G>, SessionError> {
        self.pending_with(at, false)
    }

    fn pending_with(&self, at: SystemTime, checkpoint: bool) -> Result<Pending<G>, SessionError> {
        self.validate(&self.graph)?;
        let journal = self.journal.live_cursor();
        let changes = self.changes.next_seq();
        let updated_at = (changes > self.changes_saved).then_some(at);
        let mut ops = Vec::new();
        let retained_pending_links = self.graph.held();
        // Returning to the saved value still needs a write when an earlier
        // batch may commit an intervening value. Session-only observations
        // and host coverage do not advance this durable-cache revision.
        let pending_links = Some((
            self.retained_pending_links_revision,
            retained_pending_links.clone(),
        ));
        if !self.head_stored
            || self.retained_pending_links_revision > self.saved_pending_links_revision
            || retained_pending_links != self.saved_pending_links
        {
            ops.push(pretty(
                self.keys.at(PENDING_LINKS),
                &PendingLinksSlot {
                    version: 1,
                    state: retained_pending_links,
                },
            )?);
        }
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
            ops.extend(checkpoint_ops(
                &self.keys,
                &self.graph,
                &self.digests,
                &self.journal,
                journal,
                self.translation
                    .as_ref()
                    .map(|receipt| receipt.translation_digest.clone()),
            )?);
        }
        for (source, mark) in &self.merge_marks {
            let key = Keys::new(*source).at(MERGED);
            ops.push(match mark {
                Some(mark) => pretty(key, mark)?,
                None => WriteOp::Delete { key },
            });
        }
        let mut views = Vec::new();
        for (key, view) in &self.views {
            let through = view.log.next_seq();
            if through > view.saved {
                ops.push(pretty(self.keys.view(key), &view.current)?);
                ops.extend(
                    view.log
                        .entry_writes::<JsonCodec>(&self.keys.view_log(key), view.saved)?,
                );
                views.push((key.clone(), through));
            }
        }
        Ok(Pending {
            ops,
            journal,
            changes,
            checkpoint,
            updated_at,
            views,
            pending_links,
            merge_marks: self.merge_marks.len(),
        })
    }

    /// Mark a [`Pending`] batch as stored, once it has committed.
    pub fn stored(&mut self, pending: Pending<G>) {
        let written = pending.merge_marks.min(self.merge_marks.len());
        self.merge_marks.drain(..written);
        if let Some((revision, state)) = pending.pending_links {
            if revision >= self.saved_pending_links_revision {
                self.saved_pending_links = state;
                self.saved_pending_links_revision = revision;
            }
        }
        for (key, through) in pending.views {
            if let Some(view) = self.views.get_mut(&key) {
                view.saved = view.saved.max(through);
            }
        }
        self.head_stored = true;
        self.saved = self.saved.max(pending.journal);
        self.changes_saved = self.changes_saved.max(pending.changes);
        if let Some(cursor) = pending.checkpoint {
            self.checkpointed = self.checkpointed.max(cursor);
            self.checkpoint_v1 = false;
            self.hold_replay_point(self.checkpointed);
        }
        if let Some(at) = pending.updated_at {
            self.manifest.updated_at = at;
        }
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
    pub async fn undo(&mut self, author: Author) -> Result<Option<Reverted<G>>, SessionError> {
        let reverted = self.undo_now(author)?;
        self.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(reverted)
    }

    /// [`undo`](Self::undo), journaled but not stored until the next flush,
    /// for a host that writes its batches itself ([`pending`](Self::pending),
    /// then [`stored`](Self::stored)).
    pub fn undo_now(&mut self, author: Author) -> Result<Option<Reverted<G>>, SessionError> {
        self.ensure_writable()?;
        let Some(of) = self.stacks(&author).0.last().copied() else {
            return Ok(None);
        };
        self.revert(author, of, ChangeKind::Undo { of }).map(Some)
    }

    /// Redo `author`'s most recent undo, unless an edit of theirs since has
    /// cleared it (§7 item 18). `None` when there is nothing to redo.
    pub async fn redo(&mut self, author: Author) -> Result<Option<Reverted<G>>, SessionError> {
        let reverted = self.redo_now(author)?;
        self.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(reverted)
    }

    /// [`redo`](Self::redo), journaled but not stored until the next flush.
    pub fn redo_now(&mut self, author: Author) -> Result<Option<Reverted<G>>, SessionError> {
        self.ensure_writable()?;
        let Some(of) = self.stacks(&author).1.last().copied() else {
            return Ok(None);
        };
        self.revert(author, of, ChangeKind::Redo { of }).map(Some)
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
                ChangeKind::Edit | ChangeKind::MergedIn { .. } => {
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
    fn edits_of(&self, change: &Change) -> Vec<G::Delta> {
        self.edits_in(change.first, change.end)
    }

    /// The edits the journal entries `[first, end)` wrote.
    fn edits_in(&self, first: Seq, end: Seq) -> Vec<G::Delta> {
        self.journal.entries()[first.index()..end.index()]
            .iter()
            .enumerate()
            .flat_map(|(offset, entry)| {
                self.translation
                    .as_ref()
                    .and_then(|receipt| receipt.entry_effects.get(first.index() + offset).cloned())
                    .unwrap_or_else(|| vec![entry.delta.clone()])
            })
            .collect()
    }

    /// Revert change `of` as `kind`, recording what was kept and by whom.
    /// Journaled, not stored.
    fn revert(
        &mut self,
        author: Author,
        of: Seq,
        kind: ChangeKind,
    ) -> Result<Reverted<G>, SessionError> {
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
        let (edits, kept) = G::revert(&self.edits_of(&change), &before, &after, &self.graph);
        let kept = kept
            .into_iter()
            .map(|part| Kept {
                by: self.changed_since(of, &part),
                part,
            })
            .collect();
        let deltas = edits
            .iter()
            .map(|edit| {
                G::edit_of(edit, &author)
                    .map_err(|why| SessionError::NotReplayable(format!("{why}: {edit:?}")))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mark = self.merge_mark_after(&kind);
        let (_, applied) = self.edit_as(author, kind, |graph| {
            for delta in deltas {
                let _ = graph.apply(delta);
            }
        })?;
        self.merge_marks.extend(mark);
        Ok(Reverted { of, applied, kept })
    }

    /// The source of the merge-in the latest change undid or redid, if it
    /// did: a host holding that source open reads its mark again
    /// ([`reload_merged_mark`](Self::reload_merged_mark)).
    pub fn merge_stepped(&self) -> Option<SessionId> {
        let last = self.changes.entries().last()?;
        self.merge_mark_after(&last.kind).map(|(source, _)| source)
    }

    /// Read this session's merged mark again, after a target's undo or redo
    /// of the merge changed it (M6).
    pub async fn reload_merged_mark(&mut self) -> Result<(), SessionError> {
        self.manifest.merged = read(&self.slots, &self.keys.at(MERGED)).await?;
        Ok(())
    }

    /// The merged mark an undo or redo of a merge-in leaves on its source:
    /// undoing a `MergedIn` clears it, so the source is editable and
    /// mergeable again; redoing it sets it again.
    fn merge_mark_after(&self, kind: &ChangeKind) -> Option<(SessionId, Option<MergedMark>)> {
        let merged_in = |of: &Seq| match self.changes.get(*of).map(|change| &change.kind) {
            Some(ChangeKind::MergedIn { from, through }) => Some((*from, *through)),
            _ => None,
        };
        match kind {
            ChangeKind::Undo { of } => merged_in(of).map(|(from, _)| (from, None)),
            ChangeKind::Redo { of } => match self.changes.get(*of).map(|change| &change.kind) {
                Some(ChangeKind::Undo { of }) => merged_in(of).map(|(from, through)| {
                    (
                        from,
                        Some(MergedMark {
                            into: self.id(),
                            through: through.0,
                            at: wall_clock_now(),
                        }),
                    )
                }),
                _ => None,
            },
            _ => None,
        }
    }

    /// Who, after change `of`, made a change that reached `part`.
    fn changed_since(&self, of: Seq, part: &G::Part) -> Vec<Author> {
        let mut by: Vec<Author> = Vec::new();
        for change in &self.changes.entries()[of.index() + 1..] {
            let edits = self.edits_of(change);
            if G::reaches(&edits, part) && !by.contains(&change.author) {
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

    /// Checkpoint if the journal has moved since the last one, or if the
    /// stored one names its journal by a v1 digest: what the resident calls
    /// when the last application detaches.
    pub async fn checkpoint_if_behind(&mut self) -> Result<(), SessionError> {
        if self.checkpointed < self.journal.live_cursor() || self.checkpoint_v1 {
            self.checkpoint().await?;
        }
        Ok(())
    }

    /// The graph as it stood at journal cursor `cursor`: the baseline with the
    /// first `cursor` entries replayed. `None` past the live cursor or when
    /// replay produces conflicting active Resource handles. It starts from the
    /// stored checkpoint's replay point when that lies at or before `cursor`
    /// and its digest still names the journal there, and from the baseline
    /// otherwise, with the same result (engine-split D161).
    pub fn graph_at(&self, cursor: Seq) -> Option<G> {
        let mut graph = match &self.translation {
            Some(receipt) => receipt
                .graph_at(self.journal.entries(), cursor)
                .ok()
                .flatten(),
            None => match self.replay_point_for(cursor) {
                Some(point) => self.journal.snapshot_at_from_point(point, cursor),
                None => self.journal.snapshot_at_from(&self.baseline, cursor),
            }
            .ok()
            .flatten(),
        }?;
        graph.validate_handles().ok()?;
        graph.set_known_coverage(self.graph.known_coverage().clone());
        Some(graph)
    }

    /// Every view with state, in key order.
    pub fn views(&self) -> impl Iterator<Item = (&ViewKey, &ViewState)> {
        self.views.iter().map(|(key, view)| (key, &view.current))
    }

    /// A view's current state.
    pub fn view(&self, key: &ViewKey) -> Option<&ViewState> {
        self.views.get(key).map(|view| &view.current)
    }

    /// A view's state as it stood at journal cursor `cursor`: its latest
    /// change made at or before that cursor.
    pub fn view_at(&self, key: &ViewKey, cursor: Seq) -> Option<&ViewState> {
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
        state: ViewState,
    ) -> Result<(), SessionError> {
        self.set_view_now(author, key, state)?;
        self.store(wall_clock_now(), Vec::new(), false).await
    }

    /// [`set_view`](Self::set_view), not stored until the next batch, which
    /// writes the view with the journal it names (Scenograph editor plan, SE31).
    pub fn set_view_now(
        &mut self,
        author: Author,
        key: ViewKey,
        state: ViewState,
    ) -> Result<(), SessionError> {
        self.ensure_writable()?;
        let cursor = self.journal.live_cursor();
        let view = self.views.entry(key).or_default();
        view.log.append(ViewEntry {
            author,
            cursor,
            state: state.clone(),
        });
        view.current = state;
        Ok(())
    }

    /// Complete source archive with explicit current placement when the session is qualified.
    pub fn profiled_codicil_payload(&self) -> crate::graph_codicil::ProfiledGraphCodicil<G> {
        crate::graph_codicil::ProfiledGraphCodicil {
            codicil: self.codicil_payload(),
            // A legacy profile is admitted only with its validated translation receipt.
            placement: self
                .placement
                .as_ref()
                .map(|_| PlacementProfile::RecordedStrataV1),
        }
    }

    /// Freeze graph, facets and the source's complete parallel histories.
    pub fn codicil_payload(&self) -> crate::graph_codicil::GraphCodicil<G> {
        use crate::graph_codicil::{ArchivedSession, ArchivedView, GraphCodicil};
        GraphCodicil {
            snapshot: self.graph.to_snapshot(),
            facets: self.graph.facets().clone(),
            sessions: vec![ArchivedSession {
                manifest: self.manifest.clone(),
                baseline: self.baseline.to_snapshot(),
                baseline_facets: self.baseline.facets().clone(),
                journal: self.journal.log().entries().to_vec(),
                changes: self.changes.entries().to_vec(),
                views: self
                    .views
                    .iter()
                    .map(|(key, view)| ArchivedView {
                        app: key.app.clone(),
                        view: key.view.clone(),
                        state: view.current.clone(),
                        log: view.log.entries().to_vec(),
                    })
                    .collect(),
            }],
        }
    }

    /// Record an archive receipt under the resident-supplied author.
    pub async fn archived(
        &mut self,
        author: Author,
        codicil: eidetic::ManifestId,
    ) -> Result<(), SessionError> {
        self.manifest
            .record_consolidation(crate::manifest::CodicilId(codicil.to_string()));
        self.record(
            author,
            ChangeKind::Archived {
                codicil: codicil.to_string(),
            },
            wall_clock_now(),
        )
        .await
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

/// The suffixes this crate gives a derived session's name: a fork's, a
/// branch's, a picked-off part's and a codicil thaw's.
const DERIVED_NAME_SUFFIXES: [&str; 4] =
    [" (fork of ", " (branch of ", " (picked from ", " (from "];

/// A session name without the suffix a fork, a pick-off or a codicil thaw
/// gave it, or `Session` when it has none: the base a fork's name starts
/// from, so suffixes never double.
fn derived_name_base(name: &str) -> &str {
    name.strip_suffix(')')
        .and_then(|inner| {
            DERIVED_NAME_SUFFIXES
                .iter()
                .filter_map(|suffix| inner.find(suffix))
                .min()
        })
        .map(|at| &name[..at])
        .filter(|base| !base.is_empty())
        .unwrap_or(name)
}

/// The name a fork carries, so it never reads as its parent (the fork of
/// "Session" was a second "Session"). A fork copies the whole session at a
/// point and goes its own way. Its name is the parent's without an earlier
/// fork's, pick-off's or thaw's suffix, or `Session`, and the parent it was
/// forked from, such as `Session (fork of Session)`. A fork of a fork nests,
/// and a fork of a codicil thaw names the thaw whole:
/// `Session (fork of Session (from blake3:f612a))`. It matches how a session
/// opened from a codicil names its source
/// ([`opened_codicil_session_name`](crate::graph_codicil::opened_codicil_session_name)).
pub fn forked_session_name(parent: Option<&str>) -> String {
    let parent = parent.unwrap_or("Session");
    format!("{} (fork of {parent})", derived_name_base(parent))
}

/// The name a branch carries: like a fork's, with the parent it is meant to
/// merge back into, `Session (branch of Session)`.
pub fn branched_session_name(parent: Option<&str>) -> String {
    let parent = parent.unwrap_or("Session");
    format!("{} (branch of {parent})", derived_name_base(parent))
}

/// The name a picked-off part carries: the session made by picking off the
/// connected component around one node of another ([`MereSessions::pick`],
/// Turnstone's tear-out). Picking off is
/// not forking: the part is not the whole session and starts with no session
/// history, its nodes carrying only their derivation provenance. Its name
/// starts from the source's, as a fork's does, and names where it was picked
/// from: `Evening reading (picked from Evening reading)`.
pub fn picked_session_name(source: Option<&str>) -> String {
    let source = source.unwrap_or("Session");
    format!("{} (picked from {source})", derived_name_base(source))
}

/// A mere's sessions, over its one store, for any [`SessionGraph`] (engine
/// split D244); a host installs its hooks through the `with_` builders.
pub struct MereSessions<B, G: SessionGraph> {
    backend: B,
    /// What every session these open or begin carries (engine split D257).
    hooks: SessionHooks<G>,
}

impl<B: Backend + Clone, G: SessionGraph> MereSessions<B, G> {
    /// Sessions in `backend` with no hooks; the `with_` builders add them.
    pub fn new_with(backend: B) -> Self {
        Self {
            backend,
            hooks: SessionHooks::NONE,
        }
    }

    /// Publish every session these open or begin through `hook` (engine
    /// split D257).
    pub fn with_publication(mut self, hook: &'static PublicationHook<G>) -> Self {
        self.hooks.publication = Some(hook);
        self
    }

    /// The store these sessions live in (engine split D268).
    pub fn backend(&self) -> &B {
        &self.backend
    }

    fn slots(&self) -> JsonSlots<B> {
        JsonSlots::new(self.backend.clone())
    }

    /// [`open_codicil_checked`](MereSessions::open_codicil_checked) over any
    /// graph, through these sessions' codicil hook (engine split D265).
    pub async fn open_codicil_checked_with(
        &self,
        id: eidetic::ManifestId,
        author: Author,
        validate: impl FnOnce(&G) -> Result<(), String>,
    ) -> Result<GraphSessionManifest, SessionError>
    where
        B: eidetic::Store,
    {
        let mut store = self.backend.clone();
        self.open_codicil_from_checked_with(&mut store, id, author, validate)
            .await
    }

    /// [`open_codicil_from_checked`](MereSessions::open_codicil_from_checked)
    /// over any graph, through these sessions' codicil hook.
    pub async fn open_codicil_from_checked_with(
        &self,
        store: &mut dyn eidetic::Store,
        id: eidetic::ManifestId,
        author: Author,
        validate: impl FnOnce(&G) -> Result<(), String>,
    ) -> Result<GraphSessionManifest, SessionError> {
        self.open_codicil_hooked(store, id, author, validate, self.hooks)
            .await
    }

    /// Bind every session's codicils these open or begin through `hook`
    /// (engine split D265).
    pub fn with_codicil(mut self, hook: &'static crate::graph_codicil::CodicilHook<G>) -> Self {
        self.hooks.codicil = Some(hook);
        self
    }

    /// The owning resident supplies its Eidetic store; session files keep their
    /// existing backend and layout. The complete candidate is checked before writes.
    async fn open_codicil_hooked(
        &self,
        store: &mut dyn eidetic::Store,
        id: eidetic::ManifestId,
        author: Author,
        validate: impl FnOnce(&G) -> Result<(), String>,
        hooks: SessionHooks<G>,
    ) -> Result<GraphSessionManifest, SessionError> {
        let codicil = hooks.codicil.ok_or_else(|| {
            SessionError::Validation("this session's graph has no codicil hook".into())
        })?;
        let input =
            crate::graph_codicil::load_profiled_graph_codicil_sealed_with(store, None, codicil, id)
                .await
                .map_err(|e| SessionError::Corrupt(e.to_string()))?
                .ok_or_else(|| SessionError::Corrupt(format!("no codicil {id} in this mere")))?;
        let graph = input
            .clone()
            .into_graph_with()
            .map_err(|error| SessionError::Corrupt(error.to_string()))?;
        let recorded = input.placement == Some(PlacementProfile::RecordedStrataV1);
        let payload = input.codicil;
        validate(&graph).map_err(SessionError::Validation)?;
        let mut manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        // The thaw is a new session: it names its source and the codicil, so
        // it never reads as the session it was saved from.
        manifest.display_name = Some(crate::graph_codicil::opened_codicil_session_name(
            payload
                .sessions
                .first()
                .and_then(|source| source.manifest.display_name.as_deref()),
            &id.to_string(),
        ));
        manifest.source_codicil = Some(crate::manifest::CodicilId(id.to_string()));
        let mut session = GraphSession::new_with(
            self.backend.clone(),
            manifest,
            Some(graph),
            author.clone(),
            ChangeKind::OpenedCodicil {
                codicil: id.to_string(),
            },
        );
        session.hooks = hooks;
        if recorded {
            session.placement = Some(SessionPlacement::recorded());
        }
        // Compose keeps the first source's view for each app/name, matching its graph base.
        for source in &payload.sessions {
            for view in &source.views {
                let key = ViewKey::new(&view.app, &view.view)?;
                if session.view(&key).is_none() {
                    session.set_view_now(author.clone().via(&view.app), key, view.state.clone())?;
                }
            }
        }
        session.manifest.codicil_read_only = true;
        session.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(session.manifest().clone())
    }

    /// Every session's manifest, trashed ones included, oldest first.
    pub async fn list(&self) -> Result<Vec<GraphSessionManifest>, SessionError> {
        let slots = self.slots();
        let mut manifests = Vec::new();
        for key in slots.keys(&format!("{SESSIONS_PREFIX}/")).await? {
            if key.ends_with(&format!("/{MANIFEST}"))
                && let Some(mut manifest) = read::<B, GraphSessionManifest>(&slots, &key).await?
            {
                manifest.merged = read(&slots, &Keys::new(manifest.session_id).at(MERGED)).await?;
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

    /// [`open`](MereSessions::open) over any graph, refusing a stored legacy
    /// translation.
    pub async fn open_with(&self, id: SessionId) -> Result<GraphSession<B, G>, SessionError> {
        GraphSession::open_inner(self.backend.clone(), id, self.hooks).await
    }

    /// [`begin`](MereSessions::begin) over any graph.
    pub fn begin_with(&self, author: Author, baseline: Option<G>) -> GraphSession<B, G> {
        let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        self.hooked(GraphSession::new_with(
            self.backend.clone(),
            manifest,
            baseline,
            author,
            ChangeKind::Minted,
        ))
    }

    /// [`begin_recorded`](MereSessions::begin_recorded) over any graph.
    pub fn begin_recorded_with(&self, author: Author, baseline: Option<G>) -> GraphSession<B, G> {
        let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        self.hooked(GraphSession::new_recorded_with(
            self.backend.clone(),
            manifest,
            baseline,
            author,
            ChangeKind::Minted,
        ))
    }

    fn hooked(&self, mut session: GraphSession<B, G>) -> GraphSession<B, G> {
        session.hooks = self.hooks;
        session
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
        parent: &GraphSession<B, G>,
        at: Seq,
        author: Author,
    ) -> Result<GraphSessionManifest, SessionError> {
        let mut session = self.begin_fork_at(parent, at, author)?;
        session.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(session.manifest().clone())
    }

    /// Stage an independent fork without writing it. The caller can validate
    /// and apply its first edit, then flush the entire new session in one batch.
    pub fn begin_fork_at(
        &self,
        parent: &GraphSession<B, G>,
        at: Seq,
        author: Author,
    ) -> Result<GraphSession<B, G>, SessionError> {
        self.begin_copy_at(parent, at, author, Lineage::Fork)
    }

    /// Branch `parent` at journal cursor `at`: a copy of the whole session,
    /// like a fork, that keeps its lineage link and is meant to merge back
    /// ([`merge_in`](Self::merge_in)). Its baseline is the parent's graph there
    /// and its journal starts empty.
    pub async fn branch_at(
        &self,
        parent: &GraphSession<B, G>,
        at: Seq,
        author: Author,
    ) -> Result<GraphSessionManifest, SessionError> {
        let mut session = self.begin_branch_at(parent, at, author)?;
        session.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(session.manifest().clone())
    }

    /// Stage a branch without writing it, as [`begin_fork_at`](Self::begin_fork_at)
    /// stages a fork.
    pub fn begin_branch_at(
        &self,
        parent: &GraphSession<B, G>,
        at: Seq,
        author: Author,
    ) -> Result<GraphSession<B, G>, SessionError> {
        self.begin_copy_at(parent, at, author, Lineage::Branch)
    }

    /// Pick off `part`, a graph a host copied out of `parent` into the fresh
    /// container `container`, as a new session (Turnstone's tear-out). The
    /// copy is the host's, since it knows its surfaces; the session, its
    /// name and its `picked_from` record are these sessions'. `part` is
    /// checked as `parent` checks its own graph.
    pub async fn pick(
        &self,
        parent: &GraphSession<B, G>,
        container: GraphId,
        part: G,
        author: Author,
    ) -> Result<GraphSessionManifest, SessionError> {
        parent.validate(&part)?;
        let at = parent.journal().live_cursor();
        let mut manifest = GraphSessionManifest::new(SessionId::new(), container);
        manifest.picked_from = Some(crate::manifest::PickedFrom {
            session: parent.id(),
            at: at.0,
            container: *parent.manifest().root_graph_id.as_uuid(),
        });
        manifest.display_name = Some(picked_session_name(
            parent.manifest().display_name.as_deref(),
        ));
        manifest.source_codicil = parent.manifest().source_codicil.clone();
        let kind = ChangeKind::Picked {
            from: parent.id(),
            at,
        };
        self.create(manifest, Some(&part), author, kind).await
    }

    /// A whole-session copy of `parent` at `at`, of the given lineage.
    fn begin_copy_at(
        &self,
        parent: &GraphSession<B, G>,
        at: Seq,
        author: Author,
        lineage: Lineage,
    ) -> Result<GraphSession<B, G>, SessionError> {
        let graph = parent.graph_at(at).ok_or(SessionError::CursorOutOfRange {
            cursor: at,
            live: parent.journal().live_cursor(),
        })?;
        let mut manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
        manifest.parent_session = Some(parent.id());
        manifest.forked_at = Some(at.0);
        manifest.lineage = lineage;
        let parent_name = parent.manifest().display_name.as_deref();
        let (name, kind) = match lineage {
            Lineage::Fork => (
                forked_session_name(parent_name),
                ChangeKind::Forked {
                    from: parent.id(),
                    at,
                },
            ),
            Lineage::Branch => (
                branched_session_name(parent_name),
                ChangeKind::Branched {
                    from: parent.id(),
                    at,
                },
            ),
        };
        manifest.display_name = Some(name);
        manifest.source_codicil = parent.manifest().source_codicil.clone();
        let mut session = GraphSession::new_with(
            self.backend.clone(),
            manifest,
            Some(graph),
            author.clone(),
            kind,
        );
        // A copy publishes and reads as its parent does, as it validates.
        session.hooks = parent.hooks;
        for key in parent.views.keys() {
            if let Some(state) = parent.view_at(key, at) {
                session.set_view_now(author.clone().via(key.app()), key.clone(), state.clone())?;
            }
        }
        if let Some(validator) = &parent.validator {
            session = session.with_validator(Arc::clone(validator))?;
        }
        if let Some(validator) = &parent.admission_validator {
            session = session.with_admission_validator(Arc::clone(validator));
        }
        Ok(session)
    }

    /// Write a new session's manifest, baseline and first change as one batch.
    /// Nobody else holds it, so there is nothing to recheck.
    async fn create(
        &self,
        manifest: GraphSessionManifest,
        baseline: Option<&G>,
        author: Author,
        kind: ChangeKind,
    ) -> Result<GraphSessionManifest, SessionError> {
        let mut session = self.hooked(GraphSession::<B, G>::new_with(
            self.backend.clone(),
            manifest,
            baseline.cloned(),
            author,
            kind,
        ));
        session.store(wall_clock_now(), Vec::new(), false).await?;
        Ok(session.manifest().clone())
    }
}

#[cfg(test)]
#[path = "graph_session_pending_tests.rs"]
mod pending_tests;

#[cfg(test)]
#[path = "graph_session_checkpoint_tests.rs"]
mod checkpoint_tests;

#[cfg(test)]
#[path = "graph_session_transaction_tests.rs"]
mod transaction_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_surface::{
        CODICIL, Delta, SURFACE_PROPERTIES, Surface, SurfaceDelta, SurfaceSnapshot, edges_edit,
        show_edits,
    };
    use mere::model::{JournalDelta, NodeProperty, ResourceDelta, ResourceNode};
    use muniment::{DirectoryBackend, MemoryBackend};
    use portolan::FacetId;

    type Sessions<B = MemoryBackend> = MereSessions<B, Surface>;
    type Session<B = MemoryBackend> = GraphSession<B, Surface>;

    #[test]
    fn facet_floats_replay_and_reopen_without_changing_bits() {
        pollster::block_on(async {
            let mere = Sessions::new_with(MemoryBackend::new());
            let manifest = mere.mint(person(), None).await.unwrap();
            let mut session = mere.open_with(manifest.session_id).await.unwrap();
            session.apply(person(), add(7)).await.unwrap();
            let node = Uuid::from_u128(7);
            let facet = FacetId::new("graphshell.saved-scene/v4");
            // Exact coordinates observed in the headed scene Save refusal.
            let coordinates = [21.345783233642578_f64, -26.827743530273438_f64];
            let value = serde_json::json!({ "cartography": { "positions": coordinates } });
            session
                .edit_now(person(), |graph| {
                    graph.set_facet(node, facet.as_str(), value.clone());
                })
                .expect("a captured facet must reproduce its exact floating-point values");
            let check = |graph: &Surface| {
                let actual = graph.facets().get(&node, &facet).unwrap();
                assert_eq!(actual, &value);
                for (index, expected) in coordinates.iter().enumerate() {
                    assert_eq!(
                        actual["cartography"]["positions"][index]
                            .as_f64()
                            .unwrap()
                            .to_bits(),
                        expected.to_bits(),
                    );
                }
            };
            check(session.graph());
            check(&session.graph_at(session.journal().live_cursor()).unwrap());
            session.flush(wall_clock_now()).await.unwrap();
            drop(session);
            check(mere.open_with(manifest.session_id).await.unwrap().graph());
        });
    }

    /// Placement is explicit: `begin` names none and `begin_recorded` names
    /// the recorded profile, both reopen the same graph, and an invalid
    /// placement file is refused and left as it was.
    #[test]
    fn placement_is_explicit_and_recorded_sessions_keep_held_surface_claims() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let mut baseline = Surface::default();
            baseline.show(Uuid::from_u128(1), "https://1.test/");
            baseline.show(Uuid::from_u128(2), "https://2.test/");
            baseline.assert(
                Uuid::from_u128(1),
                Uuid::from_u128(2),
                "urn:mere:profile-test:custom",
            );
            let snapshot = baseline.to_snapshot();
            let mut unqualified = mere.begin_with(person(), Some(baseline.clone()));
            assert_eq!(unqualified.placement(), None);
            unqualified.flush(wall_clock_now()).await.unwrap();
            let id = unqualified.id();
            let opened = mere.open_with(id).await.unwrap();
            assert_eq!(
                opened.placement(),
                None,
                "columns never qualify an arbitrary baseline"
            );
            assert_eq!(opened.graph().to_snapshot(), snapshot);

            let mut recorded = mere.begin_recorded_with(person(), Some(baseline));
            assert_eq!(recorded.placement(), Some(&SessionPlacement::recorded()));
            let pending = recorded.pending(wall_clock_now()).unwrap();
            assert!(pending.ops().iter().any(
                |op| matches!(op, WriteOp::Put { key, .. } if key.ends_with("/placement.json"))
            ));
            recorded.flush(wall_clock_now()).await.unwrap();
            let recorded_id = recorded.id();
            let opened = mere.open_with(recorded_id).await.unwrap();
            assert_eq!(opened.placement(), Some(&SessionPlacement::recorded()));
            assert_eq!(
                opened.graph().to_snapshot(),
                snapshot,
                "recorded placement keeps the held claims"
            );

            let fresh = mere.begin_with(person(), None);
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
                    mere.open_with(recorded_id).await,
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
                    mere.open_with(recorded_id)
                        .await
                        .unwrap()
                        .graph()
                        .to_snapshot(),
                    snapshot
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

    /// Surface `id` showing `https://{id}.test/`: the resource, then the surface.
    fn add(id: u128) -> Vec<Delta> {
        show_edits(Uuid::from_u128(id), &format!("https://{id}.test/")).1
    }

    /// Surface `id` showing the resource named `title`.
    fn retitle(id: u128, title: &str) -> Vec<Delta> {
        show_edits(Uuid::from_u128(id), &format!("https://{title}.test/")).1
    }

    /// The title surface `id` shows, read back from its resource's IRI.
    fn title(graph: &Surface, id: u128) -> Option<String> {
        let iri = graph.iri_of(Uuid::from_u128(id))?;
        Some(
            iri.trim_start_matches("https://")
                .trim_end_matches(".test/")
                .to_owned(),
        )
    }

    /// The edit that relates the resources surfaces `from` and `to` show
    /// once added.
    fn relate(from: u128, to: u128) -> Delta {
        let (a, b) = (Uuid::from_u128(from), Uuid::from_u128(to));
        let mut scratch = Surface::default();
        let ra = scratch.show(a, &format!("https://{from}.test/"));
        let rb = scratch.show(b, &format!("https://{to}.test/"));
        scratch.assert(a, b, "https://session.test/vocab#cites");
        edges_edit(ra, rb, scratch.rows_between(a, b))
    }

    async fn stored_bytes(store: &MemoryBackend) -> Vec<(String, Vec<u8>)> {
        let mut keys = store.list("").await.unwrap();
        keys.sort();
        let mut bytes = Vec::new();
        for key in keys {
            bytes.push((key.clone(), store.get(&key).await.unwrap().unwrap()));
        }
        bytes
    }

    #[test]
    fn an_observer_reads_the_stored_session_and_refuses_every_edit() {
        pollster::block_on(async {
            let store = MemoryBackend::default();
            let sessions = Sessions::new_with(store.clone());
            let id = sessions.mint(person(), None).await.unwrap().session_id;
            let mut writer = sessions.open_with(id).await.unwrap();
            writer.apply(person(), add(1)).await.unwrap();
            let key = ViewKey::new("graphshell", "panel").unwrap();
            let mut observer = sessions.open_with(id).await.unwrap().observe();
            assert!(observer.is_observing());
            assert_eq!(
                observer.graph().surface_count(),
                1,
                "it reads what was committed"
            );
            let frozen = stored_bytes(&store).await;
            assert!(matches!(
                observer.apply(person(), add(2)).await,
                Err(SessionError::Observing(_))
            ));
            assert!(matches!(
                observer.edit_now(person(), |_| panic!("must not execute")),
                Err(SessionError::Observing(_))
            ));
            assert!(matches!(
                observer.undo(person()).await,
                Err(SessionError::Observing(_))
            ));
            assert!(matches!(
                observer.set_view(person(), key, ViewState::default()).await,
                Err(SessionError::Observing(_))
            ));
            observer.flush(wall_clock_now()).await.unwrap();
            assert_eq!(
                stored_bytes(&store).await,
                frozen,
                "an observer writes nothing"
            );
            assert_eq!(observer.graph().surface_count(), 1);

            // The writer keeps committing; a fresh observer sees it.
            writer.apply(person(), add(2)).await.unwrap();
            let reopened = sessions.open_with(id).await.unwrap().observe();
            assert_eq!(reopened.graph().surface_count(), 2);
        });
    }

    #[test]
    fn profiled_archive_retains_placement_and_redaction_omits_private_history() {
        pollster::block_on(async {
            use crate::graph_codicil::{
                RedactionPolicy, load_profiled_graph_codicil_sealed_with,
                save_profiled_graph_codicil_sealed_with,
                save_profiled_session_codicil_checked_with,
            };
            let mut store = MemoryBackend::default();
            let sessions = Sessions::new_with(store.clone()).with_codicil(&CODICIL);
            let mut graph = Surface::default();
            let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
            graph.show(one, "https://profile.test/");
            graph.show(two, "https://profile.test/child");
            graph.assert(one, two, "urn:mere:profile:held-custom");
            graph.assert(one, two, "urn:mere:profile:held-second");
            let mut live = sessions.begin_recorded_with(person(), Some(graph));
            live.set_view_now(
                person(),
                ViewKey::new("graphshell", "private").unwrap(),
                ViewState::new(
                    "test.view/v1",
                    serde_json::json!({"focus": "private source history"}),
                ),
            )
            .unwrap();
            let original = serde_json::to_value(live.graph().to_snapshot()).unwrap();
            let input = live.profiled_codicil_payload();
            assert_eq!(input.placement, Some(PlacementProfile::RecordedStrataV1));
            assert!(!input.codicil.sessions.is_empty());
            let id = save_profiled_session_codicil_checked_with(
                &mut store,
                &CODICIL,
                input.clone(),
                eidetic::Timestamp(42),
                |candidate| {
                    assert_eq!(
                        serde_json::to_value(candidate.to_snapshot()).unwrap(),
                        original
                    );
                    Ok(())
                },
            )
            .await
            .unwrap();
            let retained = load_profiled_graph_codicil_sealed_with(&mut store, None, &CODICIL, id)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                serde_json::to_value(&retained).unwrap(),
                serde_json::to_value(&input).unwrap()
            );
            let opened = sessions
                .open_codicil_from_checked_with(&mut store, id, person(), |candidate| {
                    assert_eq!(
                        serde_json::to_value(candidate.to_snapshot()).unwrap(),
                        original
                    );
                    Ok(())
                })
                .await
                .unwrap();
            let reopened = sessions.open_with(opened.session_id).await.unwrap();
            assert_eq!(
                reopened.placement().unwrap().baseline,
                PlacementProfile::RecordedStrataV1
            );
            assert_eq!(
                serde_json::to_value(reopened.graph().to_snapshot()).unwrap(),
                original
            );
            assert!(reopened.manifest().codicil_read_only);
            let redacted = save_profiled_graph_codicil_sealed_with(
                &mut store,
                None,
                &CODICIL,
                input,
                RedactionPolicy::default(),
                eidetic::Timestamp(43),
            )
            .await
            .unwrap();
            let redacted = load_profiled_graph_codicil_sealed_with(&mut store, None, &CODICIL, redacted)
                .await
                .unwrap()
                .unwrap();
            assert!(
                redacted.codicil.sessions.is_empty(),
                "private histories do not bypass redaction"
            );
        });
    }

    #[test]
    fn archive_reopens_exactly_and_public_writes_require_a_fork() {
        pollster::block_on(async {
            use crate::graph_codicil::{
                load_profiled_graph_codicil_sealed_with, save_profiled_session_codicil_checked_with,
            };
            let mut store = MemoryBackend::default();
            let sessions = Sessions::new_with(store.clone()).with_codicil(&CODICIL);
            let id = sessions
                .mint(person(), Some("Kept reading".into()))
                .await
                .unwrap()
                .session_id;
            let mut live = sessions.open_with(id).await.unwrap();
            let mut edits = add(1);
            edits.extend(add(2));
            edits.push(relate(1, 2));
            live.apply(person(), edits).await.unwrap();
            let key = ViewKey::new("cleromancy", "scene").unwrap();
            let view = ViewState::new(
                "test.view/v1",
                serde_json::json!({
                    "focus": "https://1.example",
                    "camera": {"coefficients": [1.0, 0.0, 0.0, 1.0, 42.0, 17.0]},
                }),
            );
            live.set_view(person(), key.clone(), view.clone())
                .await
                .unwrap();
            let payload = live.profiled_codicil_payload();
            let archived = save_profiled_session_codicil_checked_with(
                &mut store,
                &CODICIL,
                payload.clone(),
                eidetic::Timestamp(42),
                |_| Ok(()),
            )
            .await
            .unwrap();
            let before_browse = stored_bytes(&store).await;
            let kept = load_profiled_graph_codicil_sealed_with(&mut store, None, &CODICIL, archived)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                serde_json::to_value(&payload).unwrap(),
                serde_json::to_value(&kept).unwrap()
            );
            assert_eq!(
                stored_bytes(&store).await,
                before_browse,
                "browsing writes nothing"
            );
            let thaw_id = sessions
                .open_codicil_checked_with(archived, knot(), |_| Ok(()))
                .await
                .unwrap()
                .session_id;
            assert_ne!(thaw_id, id);
            let mut thaw = sessions.open_with(thaw_id).await.unwrap();
            assert_eq!(
                whole(live.graph()),
                whole(thaw.graph()),
                "snapshot and facets exactly retained"
            );
            assert_eq!(thaw.view(&key), Some(&view));
            assert_eq!(
                thaw.manifest().source_codicil.as_ref().unwrap().0,
                archived.to_string()
            );
            assert_eq!(
                thaw.manifest().display_name,
                Some(format!(
                    "Kept reading (from {})",
                    &archived.to_string()[..12]
                )),
                "an opened codicil is named apart from its source"
            );
            assert!(thaw.manifest().codicil_read_only);
            let frozen = stored_bytes(&store).await;
            assert!(matches!(
                thaw.apply(person(), retitle(1, "bad")).await,
                Err(SessionError::ReadOnly(_))
            ));
            assert!(matches!(
                thaw.apply_deltas(person(), Vec::new()).await,
                Err(SessionError::ReadOnly(_))
            ));
            assert!(matches!(
                thaw.edit_now(person(), |_| panic!("must not execute")),
                Err(SessionError::ReadOnly(_))
            ));
            assert!(matches!(
                thaw.undo(person()).await,
                Err(SessionError::ReadOnly(_))
            ));
            assert!(matches!(
                thaw.redo(person()).await,
                Err(SessionError::ReadOnly(_))
            ));
            assert!(matches!(
                thaw.set_view(person(), key.clone(), ViewState::default())
                    .await,
                Err(SessionError::ReadOnly(_))
            ));
            assert_eq!(stored_bytes(&store).await, frozen);
            let mut fork = sessions
                .begin_fork_at(&thaw, thaw.journal().live_cursor(), knot())
                .unwrap();
            fork.apply_now(knot(), retitle(1, "edited-fork")).unwrap();
            fork.flush(wall_clock_now()).await.unwrap();
            assert_eq!(fork.manifest().parent_session, Some(thaw_id));
            assert_eq!(
                fork.manifest().display_name,
                Some(format!(
                    "Kept reading (fork of Kept reading (from {}))",
                    &archived.to_string()[..12]
                )),
                "a fork of a thaw names the thaw whole, without a doubled from"
            );
            assert_eq!(
                fork.manifest().source_codicil,
                thaw.manifest().source_codicil
            );
            assert_eq!(fork.view(&key), Some(&view));
            assert!(!fork.manifest().codicil_read_only);
            assert_eq!(
                whole(live.graph()),
                whole(sessions.open_with(thaw_id).await.unwrap().graph())
            );
            assert_ne!(whole(fork.graph()), whole(thaw.graph()));
            let fork_codicil = save_profiled_session_codicil_checked_with(
                &mut store,
                &CODICIL,
                fork.profiled_codicil_payload(),
                eidetic::Timestamp(43),
                |_| Ok(()),
            )
            .await
            .unwrap();
            let manifest = eidetic::manifest::load_manifest(&mut store, fork_codicil)
                .await
                .unwrap()
                .unwrap();
            assert_eq!(
                manifest.provenance.upstream,
                [archived],
                "a saved edit fork retains its immutable source in Eidetic lineage"
            );
        });
    }

    #[test]
    fn admission_policy_checks_transitions_preserves_history_and_follows_forks() {
        pollster::block_on(async {
            let store = MemoryBackend::default();
            let sessions = Sessions::new_with(store.clone());
            let id = sessions.mint(person(), None).await.unwrap().session_id;
            let available = Arc::new(std::sync::atomic::AtomicBool::new(true));
            let held = available.clone();
            let policy: GraphAdmissionValidator<Surface> = Arc::new(move |before, after| {
                if after.surface_count() > before.surface_count()
                    && !held.load(std::sync::atomic::Ordering::SeqCst)
                {
                    Err("new reference unavailable".into())
                } else {
                    Ok(())
                }
            });
            let mut session = sessions
                .open_with(id)
                .await
                .unwrap()
                .with_admission_validator(policy.clone());
            session.apply(person(), add(1)).await.unwrap();
            available.store(false, std::sync::atomic::Ordering::SeqCst);
            session
                .apply(person(), retitle(1, "historical-reference-retained"))
                .await
                .unwrap();
            let before = (
                whole(session.graph()),
                session.changes().to_vec(),
                session.revision(),
                stored_bytes(&store).await,
            );
            assert!(matches!(
                session.apply(person(), add(2)).await,
                Err(SessionError::Validation(_))
            ));
            assert_eq!(
                (
                    whole(session.graph()),
                    session.changes().to_vec(),
                    session.revision(),
                    stored_bytes(&store).await
                ),
                before
            );
            let mut fork = sessions
                .begin_fork_at(&session, session.journal().live_cursor(), person())
                .unwrap();
            assert!(matches!(
                fork.apply_now(person(), add(2)),
                Err(SessionError::Validation(_))
            ));
            assert_eq!(whole(fork.graph()), whole(session.graph()));
            let mut reopened = sessions
                .open_with(id)
                .await
                .unwrap()
                .with_admission_validator(policy);
            reopened.flush(wall_clock_now()).await.unwrap();
            assert_eq!(whole(reopened.graph()), whole(session.graph()));
        });
    }

    #[test]
    fn whole_graph_validation_refuses_without_graph_history_or_storage_changes() {
        pollster::block_on(async {
            let store = MemoryBackend::default();
            let sessions = Sessions::new_with(store.clone());
            let id = sessions.mint(person(), None).await.unwrap().session_id;
            let shows = |graph: &Surface, wanted: &str| title(graph, 1).as_deref() == Some(wanted);
            let validator: GraphValidator<Surface> = Arc::new(move |graph| {
                if shows(graph, "forbidden") {
                    Err("forbidden title".into())
                } else {
                    Ok(())
                }
            });
            let mut session = sessions
                .open_with(id)
                .await
                .unwrap()
                .with_validator(validator.clone())
                .unwrap();
            session.apply(person(), add(1)).await.unwrap();
            let before = (
                whole(session.graph()),
                session.changes().to_vec(),
                session.revision(),
                stored_bytes(&store).await,
            );
            assert!(matches!(
                session.apply(person(), retitle(1, "forbidden")).await,
                Err(SessionError::Validation(_))
            ));
            assert!(matches!(
                session.edit_now(person(), |graph| {
                    graph.show(Uuid::from_u128(1), "https://forbidden.test/");
                }),
                Err(SessionError::Validation(_))
            ));
            assert!(
                session
                    .apply_now(person(), retitle(1, "forbidden"))
                    .is_err()
            );
            assert!(matches!(
                session.edit_now(person(), |graph| {
                    graph
                        .facets_mut()
                        .set(
                            Uuid::from_u128(1),
                            FacetId::new("unlogged.truth"),
                            serde_json::json!(true),
                            &portolan::AcceptAll,
                        )
                        .unwrap();
                }),
                Err(SessionError::NotReplayable(_))
            ));
            assert_eq!(
                (
                    whole(session.graph()),
                    session.changes().to_vec(),
                    session.revision(),
                    stored_bytes(&store).await
                ),
                before
            );
            session
                .apply(person(), retitle(1, "allowed"))
                .await
                .unwrap();
            // A changed product authority can refuse undo and redo too.
            let refuse_old: GraphValidator<Surface> = Arc::new(move |graph| {
                if !shows(graph, "allowed") {
                    Err("old state refused".into())
                } else {
                    Ok(())
                }
            });
            session = session.with_validator(refuse_old).unwrap();
            let before = (
                whole(session.graph()),
                session.changes().to_vec(),
                stored_bytes(&store).await,
            );
            assert!(matches!(
                session.undo(person()).await,
                Err(SessionError::Validation(_))
            ));
            assert_eq!(
                (
                    whole(session.graph()),
                    session.changes().to_vec(),
                    stored_bytes(&store).await
                ),
                before
            );
            session = session.with_validator(validator).unwrap();
            session.undo(person()).await.unwrap().unwrap();
            session = session
                .with_validator(Arc::new(move |graph| {
                    if shows(graph, "allowed") {
                        Err("redo state refused".into())
                    } else {
                        Ok(())
                    }
                }))
                .unwrap();
            let before = (
                whole(session.graph()),
                session.changes().to_vec(),
                stored_bytes(&store).await,
            );
            assert!(matches!(
                session.redo(person()).await,
                Err(SessionError::Validation(_))
            ));
            assert_eq!(
                (
                    whole(session.graph()),
                    session.changes().to_vec(),
                    stored_bytes(&store).await
                ),
                before
            );
        });
    }

    #[test]
    fn a_session_journals_its_edits_under_their_authors_and_reopens() {
        pollster::block_on(async {
            let dir = tempfile::tempdir().unwrap();
            let store = DirectoryBackend::open(dir.path()).unwrap();
            let mere = Sessions::new_with(store.clone());
            let manifest = mere.mint(person(), Some("readings".into())).await.unwrap();
            let id = manifest.session_id;

            let mut session = mere.open_with(id).await.unwrap();
            let mut edits = add(1);
            edits.extend(add(2));
            let first = session.apply(person(), edits).await.unwrap();
            // Each new surface journals its resource beside the binding.
            assert_eq!((first.first, first.end), (Seq(0), Seq(4)));
            let mut edits = vec![relate(1, 2)];
            edits.extend(retitle(1, "the-tower"));
            session.apply(knot(), edits).await.unwrap();
            let live = whole(session.graph());
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
            assert_eq!(journal.lines().count(), 7);

            let store = DirectoryBackend::open(dir.path()).unwrap();
            let reopened = Session::open_with(store, id).await.unwrap();
            assert_eq!(whole(reopened.graph()), live);
            assert_eq!(reopened.journal().len(), 7);
            assert_eq!(reopened.changes().len(), 3);
            assert_eq!(
                reopened.manifest().display_name.as_deref(),
                Some("readings")
            );
        });
    }

    /// The record of resource `iri` with one facet `value_json`, as a snapshot
    /// stores it.
    fn record(iri: &str, value_json: &str) -> ResourceDelta {
        let resource = ResourceNode::for_term(iri);
        ResourceDelta::ReplaySetResourceRecordById {
            resource_id: resource.id().to_string(),
            record: Some(mere::model::PersistedResourceRecord {
                canonical_iri: resource.canonical_iri().to_owned(),
                facets: vec![mere::model::PersistedResourceFacet {
                    facet: "foreign.metadata".into(),
                    value_json: value_json.into(),
                }],
            }),
        }
    }

    #[test]
    fn checked_snapshot_session_load_reports_baseline_and_checkpoint_corruption() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let keys = Keys::new(id);
            let mut graph = Surface::default();
            graph.show(Uuid::from_u128(1), "https://1.test/");
            let mut valid = graph.to_snapshot();
            valid
                .resources
                .push(record("urn:mere:test:checked-session", "true"));
            let baseline = |snapshot: SurfaceSnapshot| Baseline {
                graph: snapshot,
                facets: NodeFacetStore::default(),
            };
            // The checkpoint record a writer would leave beside `graph` bytes.
            let checkpoint = |graph: &[u8]| {
                pretty(
                    keys.at(CHECKPOINT),
                    &Checkpoint {
                        cursor: Seq(0),
                        translated_receipt_digest: None,
                        journal_digest: journal_digest(
                            &Mutex::default(),
                            &SessionJournal::<Surface>::default(),
                            Seq(0),
                        )
                        .unwrap(),
                        journal_digest_version: Some(JOURNAL_DIGEST_VERSION),
                        contents_digest: Some(contents_digest(graph, b"")),
                    },
                )
                .unwrap()
            };
            for column in [BASELINE, GRAPH] {
                for conflict in [false, true] {
                    store
                        .apply(&[
                            pretty(keys.at(BASELINE), &baseline(valid.clone())).unwrap(),
                            pretty(keys.at(GRAPH), &valid).unwrap(),
                            checkpoint(&pretty_bytes(&valid).unwrap()),
                        ])
                        .await
                        .unwrap();
                    let opened = Session::open_with(store.clone(), id).await.unwrap();
                    assert_eq!(opened.graph().to_snapshot().resources.len(), valid.resources.len());
                    assert_eq!(opened.graph().surface_count(), 1);
                    let mut invalid = valid.clone();
                    if conflict {
                        invalid
                            .resources
                            .push(record("urn:mere:test:checked-session", "false"));
                    } else {
                        *invalid.resources.last_mut().unwrap() =
                            record("urn:mere:test:checked-session", "{");
                    }
                    let op = if column == BASELINE {
                        pretty(keys.at(column), &baseline(invalid.clone())).unwrap()
                    } else {
                        pretty(keys.at(column), &invalid).unwrap()
                    };
                    store.apply(&[op.clone()]).await.unwrap();
                    if column == GRAPH {
                        // Written consistently, as a faulty writer would: the
                        // checkpoint names these bytes, so they are validated.
                        store
                            .apply(&[checkpoint(&pretty_bytes(&invalid).unwrap())])
                            .await
                            .unwrap();
                    }
                    match Session::open_with(store.clone(), id).await {
                        Err(SessionError::Corrupt(error)) => {
                            assert!(error.contains(&keys.at(column)), "{error}");
                            assert!(
                                error.contains(if conflict {
                                    "conflicts"
                                } else {
                                    "invalid JSON"
                                }),
                                "{error}"
                            );
                        },
                        Err(error) => panic!("unexpected load error: {error}"),
                        Ok(_) => panic!("invalid resource snapshot was loaded"),
                    }
                    let slots = JsonSlots::new(store.clone());
                    let held = if column == BASELINE {
                        read::<_, Baseline<SurfaceSnapshot>>(&slots, &keys.at(column))
                            .await
                            .unwrap()
                            .unwrap()
                            .graph
                    } else {
                        read::<_, SurfaceSnapshot>(&slots, &keys.at(column))
                            .await
                            .unwrap()
                            .unwrap()
                    };
                    assert_eq!(held.resources, invalid.resources);
                }
            }
        });
    }

    /// A statement of a surface's own, with handle `handle`.
    fn literal(handle: &str) -> NodeProperty {
        NodeProperty {
            statement_id: handle.into(),
            predicate: "urn:test:predicate".into(),
            value: "held".into(),
            datatype: None,
            lang: None,
            graph_scope: Default::default(),
            provenance_iri: None,
            asserted_at_ms: None,
        }
    }

    #[test]
    fn opening_a_session_whose_facets_steal_a_handle_is_refused() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let keys = Keys::new(id);
            let surface = Uuid::from_u128(1);
            let mut graph = Surface::default();
            let resource = graph.show(surface, "https://1.test/");
            // The resource holds the handle as a literal of its own.
            let mut owned = graph.resources().resource_record(resource).unwrap();
            owned.facets.push(mere::model::PersistedResourceFacet {
                facet: mere::model::RESOURCE_PROPERTIES.into(),
                value_json: serde_json::to_string(&[literal("owned-handle")]).unwrap(),
            });
            graph
                .write_resource_record(&person(), resource, Some(owned))
                .unwrap();
            let snapshot = graph.to_snapshot();
            let honest = graph.facets().clone();
            let mut stolen = honest.clone();
            stolen
                .set(
                    surface,
                    FacetId::new(SURFACE_PROPERTIES),
                    serde_json::to_value(vec![literal("owned-handle")]).unwrap(),
                    &portolan::AcceptAll,
                )
                .unwrap();
            // The files a faulty writer would leave: the checkpoint names the
            // very bytes of `graph.json` and `facets.json`, so it is trusted.
            let write = |baseline: &NodeFacetStore, checkpoint: &NodeFacetStore| {
                let graph_bytes = pretty_bytes(&snapshot).unwrap();
                let facet_bytes = pretty_bytes(checkpoint).unwrap();
                let record = Checkpoint {
                    cursor: Seq(0),
                    translated_receipt_digest: None,
                    journal_digest: journal_digest(
                        &Mutex::default(),
                        &SessionJournal::<Surface>::default(),
                        Seq(0),
                    )
                    .unwrap(),
                    journal_digest_version: Some(JOURNAL_DIGEST_VERSION),
                    contents_digest: Some(contents_digest(&graph_bytes, &facet_bytes)),
                };
                [
                    pretty(
                        keys.at(BASELINE),
                        &Baseline {
                            graph: snapshot.clone(),
                            facets: baseline.clone(),
                        },
                    )
                    .unwrap(),
                    WriteOp::Put {
                        key: keys.at(GRAPH),
                        value: graph_bytes,
                    },
                    WriteOp::Put {
                        key: keys.at(NODE_FACETS_FILE),
                        value: facet_bytes,
                    },
                    pretty(keys.at(CHECKPOINT), &record).unwrap(),
                ]
            };
            store.apply(&write(&honest, &honest)).await.unwrap();
            let opened = Session::open_with(store.clone(), id).await.unwrap();
            assert_eq!(opened.graph().facets(), &honest);

            for (baseline, checkpoint, refused_at) in
                [(&stolen, &honest, BASELINE), (&honest, &stolen, GRAPH)]
            {
                store.apply(&write(baseline, checkpoint)).await.unwrap();
                match Session::open_with(store.clone(), id).await {
                    Err(SessionError::Corrupt(error)) => {
                        assert!(error.contains(&keys.at(refused_at)), "{error}");
                        assert!(error.contains("facets refused"), "{error}");
                    },
                    Err(error) => panic!("unexpected load error: {error}"),
                    Ok(_) => panic!("facets that steal a handle were installed"),
                }
            }
        });
    }

    #[test]
    fn a_checkpoint_and_its_tail_load_the_same_graph_as_the_baseline_and_journal() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            session.set_checkpoint_interval(4);
            for n in 0..3 {
                session.apply(person(), add(n)).await.unwrap();
            }
            // Surface 2 shows surface 0's resource: one entry, in the tail.
            let mut point = show_edits(Uuid::from_u128(2), "https://0.test/").1;
            session
                .apply(person(), vec![point.pop().unwrap()])
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

            // Whole graphs, facets included: replay is exact.
            let reopened = Session::open_with(store.clone(), id).await.unwrap();
            assert_eq!(whole(reopened.graph()), live, "checkpoint plus tail");
            let replayed = reopened.graph_at(reopened.journal().live_cursor()).unwrap();
            assert_eq!(whole(&replayed), live, "baseline plus whole journal");
        });
    }

    /// A graph's whole persisted state: its snapshot and its facets.
    fn whole(graph: &Surface) -> (serde_json::Value, serde_json::Value) {
        (
            serde_json::to_value(graph.to_snapshot()).unwrap(),
            serde_json::to_value(graph.facets()).unwrap(),
        )
    }

    #[test]
    fn a_fork_at_a_cursor_starts_from_the_parent_there_and_diverges() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let parent_id = mere
                .mint(person(), Some("parent".into()))
                .await
                .unwrap()
                .session_id;
            let mut parent = mere.open_with(parent_id).await.unwrap();
            parent.apply(person(), add(1)).await.unwrap();
            parent.apply(person(), add(2)).await.unwrap();

            // Cursor 2 lies after the first surface's two entries.
            let fork = mere.fork_at(&parent, Seq(2), knot()).await.unwrap();
            assert_eq!(fork.parent_session, Some(parent_id));
            assert_eq!(
                fork.display_name.as_deref(),
                Some("parent (fork of parent)"),
                "a fork never reads as its parent"
            );
            let unnamed = mere.mint(person(), None).await.unwrap();
            let unnamed = mere.open_with(unnamed.session_id).await.unwrap();
            let unnamed_fork = mere
                .fork_at(&unnamed, unnamed.journal().live_cursor(), knot())
                .await
                .unwrap();
            assert_eq!(
                unnamed_fork.display_name.as_deref(),
                Some("Session (fork of Session)")
            );
            let refork = mere.open_with(unnamed_fork.session_id).await.unwrap();
            assert_eq!(
                mere.fork_at(&refork, refork.journal().live_cursor(), knot())
                    .await
                    .unwrap()
                    .display_name
                    .as_deref(),
                Some("Session (fork of Session (fork of Session))"),
                "a fork of a fork keeps one base and names its own parent"
            );
            assert_eq!(fork.forked_at, Some(2));
            let mut fork = mere.open_with(fork.session_id).await.unwrap();
            assert_eq!(
                fork.graph().surface_count(),
                1,
                "the parent's graph at cursor 2"
            );
            assert!(
                fork.journal().is_empty(),
                "the fork's journal starts at the fork point"
            );
            let provenance = fork.journal().provenance().unwrap().clone();
            assert_eq!(provenance.at, Seq(2));
            assert_eq!(
                fork.changes()[0].kind,
                ChangeKind::Forked {
                    from: parent_id,
                    at: Seq(2)
                }
            );

            fork.apply(knot(), add(3)).await.unwrap();
            assert_eq!(fork.graph().surface_count(), 2);
            assert_eq!(parent.graph().surface_count(), 2, "the parent is untouched");
            assert!(parent.graph().shown(Uuid::from_u128(3)).is_none());
            assert!(mere.fork_at(&parent, Seq(9), knot()).await.is_err());
        });
    }

    #[test]
    fn undo_steps_back_through_an_author_s_own_changes_and_redo_returns() {
        pollster::block_on(async {
            let mere = Sessions::new_with(MemoryBackend::new());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            session.apply(person(), add(1)).await.unwrap();
            session.apply(knot(), add(2)).await.unwrap();
            session.apply(person(), retitle(1, "one")).await.unwrap();

            let undone = session.undo(person()).await.unwrap().unwrap();
            assert!(undone.kept.is_empty());
            assert_ne!(title(session.graph(), 1).as_deref(), Some("one"));
            session.undo(person()).await.unwrap().unwrap();
            assert!(session.graph().shown(Uuid::from_u128(1)).is_none());
            assert!(
                session.graph().shown(Uuid::from_u128(2)).is_some(),
                "the other author's change stays"
            );
            assert!(
                session.undo(person()).await.unwrap().is_none(),
                "nothing left to undo"
            );

            session.redo(person()).await.unwrap().unwrap();
            assert!(session.graph().shown(Uuid::from_u128(1)).is_some());
            session.redo(person()).await.unwrap().unwrap();
            assert_eq!(title(session.graph(), 1).as_deref(), Some("one"));
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
            let mere = Sessions::new_with(MemoryBackend::new());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            let mut edits = add(1);
            edits.extend(retitle(1, "start"));
            session.apply(person(), edits).await.unwrap();
            // Person's change: surface 1 shows "mine", a resource it writes.
            let (mine, edits) = show_edits(Uuid::from_u128(1), "https://mine.test/");
            session.apply(person(), edits).await.unwrap();
            session.apply(knot(), retitle(1, "theirs")).await.unwrap();

            let undone = session.undo(person()).await.unwrap().unwrap();
            assert_eq!(
                undone.kept,
                [Kept {
                    part: crate::test_surface::Part::Surface(Uuid::from_u128(1)),
                    by: vec![knot()],
                }]
            );
            assert_eq!(
                title(session.graph(), 1).as_deref(),
                Some("theirs"),
                "their later edit stays"
            );
            assert!(
                session.graph().resources().resource(mine).is_none(),
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
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            session.apply(person(), add(1)).await.unwrap();
            session.undo(person()).await.unwrap().unwrap();
            drop(session);

            let mut reopened = mere.open_with(id).await.unwrap();
            assert!(reopened.graph().shown(Uuid::from_u128(1)).is_none());
            reopened.redo(person()).await.unwrap().unwrap();
            assert!(reopened.graph().shown(Uuid::from_u128(1)).is_some());
        });
    }

    #[test]
    fn derived_names_start_from_one_base_and_name_their_source_whole() {
        assert_eq!(forked_session_name(None), "Session (fork of Session)");
        assert_eq!(
            forked_session_name(Some("Session (fork of Session)")),
            "Session (fork of Session (fork of Session))",
            "a fork of a fork nests"
        );
        assert_eq!(
            forked_session_name(Some("Session (from blake3:f612a)")),
            "Session (fork of Session (from blake3:f612a))",
            "a fork of a codicil thaw never doubles its from"
        );
        assert_eq!(picked_session_name(None), "Session (picked from Session)");
        assert_eq!(
            picked_session_name(Some("Evening reading")),
            "Evening reading (picked from Evening reading)"
        );
        assert_eq!(
            picked_session_name(Some("Session (from blake3:f612a)")),
            "Session (picked from Session (from blake3:f612a))"
        );
        assert_eq!(
            forked_session_name(Some("Evening reading (picked from Evening reading)")),
            "Evening reading (fork of Evening reading (picked from Evening reading))"
        );
    }

    #[test]
    fn trash_marks_the_manifest_and_restore_clears_it() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
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
            assert_eq!(mere.open_with(id).await.unwrap().changes().len(), 3);
        });
    }

    #[test]
    fn a_session_begun_in_memory_is_stored_whole_by_its_first_flush() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mut baseline = Surface::default();
            baseline.show(Uuid::from_u128(1), "https://kept.test/");
            let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
            let id = manifest.session_id;
            let mut session = Session::new_with(
                store.clone(),
                manifest,
                Some(baseline),
                person(),
                ChangeKind::Minted,
            );
            let (resource, applied) = session
                .edit_now(person(), |graph| {
                    graph.show(Uuid::from_u128(2), "https://2.test/")
                })
                .unwrap();
            assert_eq!(session.graph().shown(Uuid::from_u128(2)), Some(resource));
            assert_eq!(applied.first, Seq(0));
            assert!(
                store.list(SESSIONS_PREFIX).await.unwrap().is_empty(),
                "nothing is written before the flush"
            );
            assert!(session.has_unstored());

            let at = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
            session.flush(at).await.unwrap();
            let reopened = Session::open_with(store, id).await.unwrap();
            assert_eq!(whole(reopened.graph()), whole(session.graph()));
            assert_eq!(reopened.changes(), session.changes());
            assert_eq!(reopened.manifest().updated_at, at);
            assert!(
                session.pending(at).unwrap().is_empty(),
                "a flush with nothing new writes nothing"
            );
            assert!(!session.has_unstored(), "and nothing waits for one");
        });
    }

    #[test]
    fn a_batch_that_never_commits_stays_pending_until_one_does() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let manifest = GraphSessionManifest::new(SessionId::new(), GraphId::new());
            let id = manifest.session_id;
            let mut session =
                Session::new_with(store.clone(), manifest, None, person(), ChangeKind::Minted);
            session.apply_now(person(), add(1)).unwrap();
            let at = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(1_800_000_000);
            // A host's batch took these writes and then failed to commit.
            let lost = session.pending(at).unwrap();
            assert!(!lost.is_empty());
            session.apply_now(knot(), retitle(1, "later")).unwrap();

            let retry = session.pending(at).unwrap();
            assert!(
                retry.ops().len() > lost.ops().len(),
                "the retry carries the lost batch and what followed"
            );
            store.apply(retry.ops()).await.unwrap();
            session.stored(retry);
            assert!(session.pending(at).unwrap().is_empty());

            let reopened = Session::open_with(store, id).await.unwrap();
            assert_eq!(whole(reopened.graph()), whole(session.graph()));
            assert_eq!(reopened.journal().len(), session.journal().len());
        });
    }

    #[test]
    fn the_latest_live_session_is_the_one_changed_last() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let first = mere.mint(person(), None).await.unwrap().session_id;
            let second = mere.mint(person(), None).await.unwrap().session_id;
            let latest = || async { mere.latest_live().await.unwrap().map(|m| m.session_id) };
            let base = wall_clock_now() + std::time::Duration::from_secs(60);
            let at = |seconds| base + std::time::Duration::from_secs(seconds);

            let mut one = mere.open_with(first).await.unwrap();
            one.apply_now(person(), add(1)).unwrap();
            one.flush(at(0)).await.unwrap();
            let mut two = mere.open_with(second).await.unwrap();
            two.apply_now(person(), add(2)).unwrap();
            two.flush(at(1)).await.unwrap();
            assert_eq!(latest().await, Some(second));

            one.apply_now(person(), retitle(1, "again")).unwrap();
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
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            let key = ViewKey::new("knot-editor", "main").unwrap();
            let spectral =
                ViewState::new("test.view/v1", serde_json::json!({"strategy": "spectral"}));
            session
                .set_view(knot(), key.clone(), spectral.clone())
                .await
                .unwrap();
            session.apply(person(), add(1)).await.unwrap();
            let focused = ViewState::new(
                "test.view/v1",
                serde_json::json!({"strategy": "spectral", "focus": Uuid::from_u128(1).to_string()}),
            );
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

            let reopened = mere.open_with(id).await.unwrap();
            assert_eq!(reopened.view(&key), Some(&focused));
            assert_eq!(reopened.view_at(&key, Seq(0)), Some(&spectral));
            assert!(ViewKey::new("Knot", "main").is_err());
            assert!(ViewKey::new("knot", "../main").is_err());
        });
    }

    #[test]
    fn a_view_set_now_waits_for_the_next_batch() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            assert!(!session.has_unstored());
            let key = ViewKey::new("graphshell", "commands").unwrap();
            let menu = ViewState::new(
                "test.view/v1",
                serde_json::json!({"commands": {
                    "added": ["add-address"],
                    "removed": ["zoom-out"],
                    "recent": ["zoom-in"],
                }}),
            );
            session
                .set_view_now(person(), key.clone(), menu.clone())
                .unwrap();
            assert_eq!(session.view(&key), Some(&menu));
            assert!(session.has_unstored(), "the view waits for a batch");
            assert_eq!(
                mere.open_with(id).await.unwrap().view(&key),
                None,
                "not stored yet"
            );

            let pending = session.pending(wall_clock_now()).unwrap();
            store.apply(pending.ops()).await.unwrap();
            session.stored(pending);
            assert!(!session.has_unstored());
            assert!(session.pending(wall_clock_now()).unwrap().is_empty());
            assert_eq!(mere.open_with(id).await.unwrap().view(&key), Some(&menu));
            assert_eq!(session.journal().len(), 0, "a view is not graph truth");
        });
    }

    #[test]
    fn an_edit_without_a_replay_form_is_refused_whole() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let id = mere.mint(person(), None).await.unwrap().session_id;
            let mut session = mere.open_with(id).await.unwrap();
            let retired = JournalDelta::Surface(SurfaceDelta::Retired {
                note: "a thumbnail".into(),
            });
            let mut edits = add(1);
            edits.push(retired);
            assert!(session.apply(person(), edits).await.is_err());
            assert_eq!(
                session.graph().surface_count(),
                0,
                "nothing of the batch applied"
            );
            assert!(session.graph().resources().resource_nodes().next().is_none());
            assert!(session.journal().is_empty());
        });
    }

    #[test]
    fn a_session_opens_only_if_its_mere_holds_it() {
        pollster::block_on(async {
            let mere = Sessions::new_with(MemoryBackend::new());
            let missing = SessionId::new();
            assert!(matches!(
                mere.open_with(missing).await,
                Err(SessionError::Missing(id)) if id == missing
            ));
        });
    }

    /// One statement from surface 1 to 2 at `time`, its source `source` or,
    /// when none, its author; an existing one of the same author is replaced.
    fn claim(graph: &mut Surface, author: &Author, time: u64, source: Option<&str>) -> String {
        let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
        let asserter = source
            .map(str::to_owned)
            .unwrap_or_else(|| author.asserter_iri());
        let mut rows = graph.rows_between(one, two);
        let mut scratch = Surface::default();
        scratch.show(one, "https://1.test/");
        scratch.show(two, "https://2.test/");
        scratch.assert(one, two, "https://session.test/vocab#cites");
        let template = scratch.rows_between(one, two).remove(0);
        let mut statement = template.retained_assertions()[0].clone();
        statement.statement_id = match source {
            Some(_) => format!("cites from {asserter}"),
            None => format!("cites by {asserter}"),
        };
        statement.provenance_iri = source.map(str::to_owned);
        statement.asserted_at_ms = Some(time);
        let handle = statement.statement_id.clone();
        match rows.first_mut() {
            Some(row) => {
                let held = row.assertions.get_or_insert_with(Vec::new);
                held.retain(|row| row.statement_id != handle);
                held.push(statement);
            },
            None => {
                let mut row = template;
                row.assertions = Some(vec![statement]);
                rows.push(row);
            },
        }
        let (from, to) = (graph.shown(one).unwrap(), graph.shown(two).unwrap());
        let edit = Surface::edit_of(&edges_edit(from, to, rows), author).unwrap();
        SessionGraph::apply(graph, edit).unwrap();
        handle
    }

    /// An engine's author identity names no version, so a new version updates
    /// its own assertion; an explicit source keeps a row of its own.
    #[test]
    fn assertion_author_version_and_source_survive_session_replay() {
        pollster::block_on(async {
            let mere = Sessions::new_with(MemoryBackend::default());
            let manifest = mere.mint(person(), None).await.unwrap();
            let mut session = mere.open_with(manifest.session_id).await.unwrap();
            let mut edits = add(1);
            edits.extend(add(2));
            session.apply(person(), edits).await.unwrap();
            let (v1, v2) = (Author::engine("extractor", "1"), Author::engine("extractor", "2"));
            let (first, _) = session
                .edit_now(v1.clone(), |graph| claim(graph, &v1, 10, None))
                .unwrap();
            let (second, _) = session
                .edit_now(v2.clone(), |graph| claim(graph, &v2, 20, None))
                .unwrap();
            assert_eq!(
                first, second,
                "new engine version updates its own assertion"
            );
            session
                .edit_now(v2.clone(), |graph| {
                    claim(graph, &v2, 30, Some("https://page.test/"))
                })
                .unwrap();
            let graph = session.graph();
            let rows = graph.rows_between(Uuid::from_u128(1), Uuid::from_u128(2));
            let statements = rows[0].retained_assertions();
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
            let live = whole(graph);
            let replayed = session.graph_at(session.journal().live_cursor()).unwrap();
            assert_eq!(whole(&replayed), live);
        });
    }

    #[test]
    fn replayed_session_handles_are_checked_before_open_and_historical_publication() {
        pollster::block_on(async {
            let store = MemoryBackend::new();
            let mere = Sessions::new_with(store.clone());
            let mut baseline = Surface::default();
            let (one, two) = (Uuid::from_u128(1), Uuid::from_u128(2));
            baseline.show(one, "https://1.test/");
            baseline.show(two, "https://2.test/");
            let mut held = literal("shared replay handle");
            held.predicate = "urn:replay:literal".into();
            baseline.set_facet(
                one,
                SURFACE_PROPERTIES,
                serde_json::to_value(vec![held.clone()]).unwrap(),
            );
            assert!(baseline.validate_handles().is_ok());
            let baseline_truth = whole(&baseline);
            let mut session = mere.begin_recorded_with(person(), Some(baseline.clone()));
            let id = session.manifest().session_id;
            session.flush(wall_clock_now()).await.unwrap();
            let opened = mere.open_with(id).await.unwrap();
            assert_eq!(whole(opened.graph()), baseline_truth);
            let facet = |property: NodeProperty| {
                JournalDelta::Surface(SurfaceDelta::SetFacet {
                    surface_id: two,
                    facet: SURFACE_PROPERTIES.into(),
                    value: serde_json::to_value(vec![property]).unwrap(),
                })
            };
            let mut distinct = held.clone();
            distinct.statement_id = "distinct replay handle".into();
            session.journal.record_as(person(), facet(distinct.clone()));
            session.flush(wall_clock_now()).await.unwrap();
            let mut valid = mere.open_with(id).await.unwrap();
            assert_eq!(
                valid
                    .graph()
                    .facets()
                    .get(&two, &FacetId::new(SURFACE_PROPERTIES))
                    .cloned(),
                Some(serde_json::to_value(vec![distinct]).unwrap())
            );
            valid.checkpoint().await.unwrap();
            assert!(session.graph_at(Seq(1)).is_some());
            session.journal.record_as(person(), facet(held));
            session.flush(wall_clock_now()).await.unwrap();
            assert!(session.graph_at(Seq(0)).is_some());
            assert!(session.graph_at(Seq(1)).is_some());
            assert!(
                session.graph_at(Seq(2)).is_none(),
                "historical read refuses the colliding replay result"
            );
            assert_eq!(
                whole(session.graph()),
                baseline_truth,
                "failed publication does not mutate the live graph"
            );
            let retained = stored_bytes(&store).await;
            match mere.open_with(id).await {
                Err(SessionError::Corrupt(error)) => {
                    assert!(error.contains(&session.keys.at(JOURNAL)), "{error}");
                    assert!(error.contains("shared replay handle"), "{error}");
                },
                Err(error) => panic!("unexpected error: {error}"),
                Ok(_) => panic!("colliding journal replay was published"),
            }
            assert_eq!(
                stored_bytes(&store).await,
                retained,
                "a refused open leaves the retained source as it was"
            );
        });
    }
}
