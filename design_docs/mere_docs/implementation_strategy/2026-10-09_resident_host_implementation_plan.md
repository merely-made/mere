# Initial Resident Host Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (the user-selected orchestration method) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Root serializes compilation; independent runtime tests may overlap. One writer owns each shared module.

**Goal:** Give Graphshell/Djinn's owned complete GraphSession a configurable partial presentation, typed live edits, explicit durability outcomes, staged checkpoint publication, refresh and portable coverage.

**Architecture:** GraphSession remains the complete writer and history authority. ResidentGraph supplies immutable presentation data; typed stable IDs cross back to the writer under observation preconditions. Owned hosts prepare checkpoint and addressable writes together, release browser borrows for I/O, acknowledge confirmed storage, then install a lifecycle-checked resident candidate.

**Tech Stack:** Rust, existing graph-kernel/Pandect/Muniment, Graphshell/Djinn, Chirograph/graphshell-client, native Directory/redb and browser IndexedDB. Reuse existing event pumps and projection notices.

**Spec:** [Approved initial resident host design](../technical_architecture/2026-10-09_resident_host_design.md). Read it alongside the [parent follow-ons plan](2026-10-09_graph_semantics_followons_plan.md).

**Date:** 2026-10-09
**Status (2026-10-09):** Written spec and concrete plan approved by Mark. Tasks 1–2 are complete and locally committed at a61dfb5c9 and 652acbb72 after independent reviews. Task 3 starts combined persistence staging; Tasks 4–8 remain pending. Root serializes compilation and preserves separate consumer qualification; independent runtime tests may overlap.

## Global Constraints

- “Full remains the compatibility setting.” Neighborhood settings belong to the graph, with selected authored projection, hops and manual or host-driven refresh.
- “ResidentGraph remains immutable.” It never supplies writer NodeKeys or a complete checkpoint.
- “Do not add a general multiwriter lock service, authority epochs or persistent request-receipt log for this milestone.” Reuse session ownership, Applied/Pending and existing host lifecycle identity.
- “A store error is not evidence that an edit never happened.” Pause edits during the first combined staged commit; retry the retained batch, never the command.
- “Keep remains its existing per-view control, separate from graph pinning.” Every pane's transient focus participates without creating a durable pin.
- “Do not collect generations in this milestone.” Turnstone authority migration, genuinely bounded writers and detailed GC/scoped-transaction design remain separate reviewed work.
- “Report process memory separately from reader-record savings; this milestone makes no bounded-process-memory claim.” Full writer/baseline/history and candidate/replay peaks remain.
- Preserve transport modularity, data projections, the shared wgpu device and existing Author, qualification, validation, replay, journal, undo and kept-part controls.
- No backend implementation changes, consumer repins or consumer authority changes belong to these tasks. Unsupported or unproven ownership/backend composition returns a typed refusal.
- Each code sample below is either a proposed compile-ready declaration/test after its named task, or explicitly labelled implementation pseudocode. Proposed signatures are decisions of this plan, not claims that those APIs already exist.
- Changes and commits stay local until the user reviews a qualified candidate. Root owns canonical index, D2 judgment and status updates. Protect the separate Dramatis writer and A1 consumer worktrees.

## Review Focus

1. An unrelated hidden incident change may conservatively invalidate a record observation; an unrelated source revision with identical observed records must still allow an edit. Task 2 tests both.
2. A later command in a batch may target an earlier creation, or be invalid after that creation; refusal must leave graph, recorder, journal and reader untouched. Task 2 tests both.
3. A write may commit and report an error; retry must recognize matching writes and acknowledge once, while a mixed state refuses. Task 3 tests both.
4. A pane may have an empty or missing authored roster; it must stay scoped and release only its own transient roots on close. Task 4 tests both.
5. A coverage update or newer publication may arrive during refresh; latest context survives installation and the newer notice remains pending. Task 5 tests both, and Task 6 carries context-only updates through the client.

---

## Verified interface map and file ownership

These facts were read from source at `a8889f60e8eb11a72ad86498405d091f9ed8be7e` on 2026-10-09. RDF supplier `2976821783cee6b24d13cacb046c1d99b1890fb4` is the source-qualified supplier; the root owns current publication receipts. These are source facts, not qualification of this host work.

| Existing path | Verified responsibility/interface | Planned owner |
|---|---|---|
| `crates/system/pandect/src/addressable_graph/mod.rs` | GraphAddress::{Surface,Resource}(Uuid); ResidencyPolicy::{Full,Neighborhood { hops: u16 }}; roots have projection_surfaces, pinned and **one** focused; status Loaded/NotLoaded/Absent; set_host_coverage | Tasks 1, 4 and 5, sequential |
| `crates/system/pandect/src/addressable_graph/write.rs` | async write_recorded_graph(&self, &Graph) -> Result<String, StoreError>; builds StoredNode incident copies, catalog and head then immediately applies | Task 1 |
| `crates/system/pandect/src/addressable_graph/schema.rs` | IndexRow::{address,hash,shown,pinned,declaration}; StoredNode Surface/Resource; JSON encoding and blake3 hash | Task 1 |
| `crates/system/pandect/src/addressable_graph/residency.rs` | reconcile(ResidencyRoots), refresh(), demand(GraphAddress), set_policy(ResidencyPolicy); candidate constructed before install; current refresh reads catalog even if head unchanged | Tasks 4 and 5, sequential |
| `crates/system/pandect/src/graph_session.rs` | edit_now<R>(&mut self, Author, impl FnOnce(&mut Graph)->R) -> Result<(R, Applied), SessionError>; clones candidate, validates and compares captured replay before installation; pending(SystemTime), stored(Pending); pending_with(at,true) is private | Tasks 2 and 3, sequential |
| `crates/system/pandect/src/graph_session/residency.rs` | publish_resident_checkpoint currently checkpoints then publishes separately; placement/Residency refusal; MereSessions::open_resident_graph | Task 3 |
| `ports/graphshell/src/mere_host.rs` | owns GraphSession and retired sessions; Staged writes each Pending separately; prepare_store(u64), staged(Staged), graph(), graph_session(), store(); graph() remains Full authority | Tasks 3, 4 and 6, sequential |
| `ports/graphshell/src/product.rs` | create_address, edit_node, set_product_facet, assert_product_relation, create_file_metadata; current file intake deliberately records several host changes | Task 7 |
| `ports/graphshell/src/web_session.rs` | Rc/RefCell prepare/write/install with one write in flight, but current host accepts edits while it runs | Tasks 3 and 7, sequential |
| `ports/djinn/src/resident_mere.rs` | MereShared::hosts caches Arc<Mutex<MereHost<DirectoryBackend>>> by SessionId; host opens checked once; application identity comes from admitted endpoint; notice compares host projection epoch/revision | Tasks 4, 6 and 7, sequential |
| `crates/chirograph/src/lib.rs` | ProjectionSnapshot has version, session, scene, presentation and cache_policy; no coverage; context currently cannot travel through this boundary | Task 6 |
| `crates/graphshell/graphshell-client/src/lib.rs` | MountedScene holds scene/presentation/cache_policy/status; ClientState::apply_snapshot validates and installs; persisted projection embeds MountedScene | Task 6 |
| `ports/graphshell/src/web_product.rs` | sync_canvas replaces canvas from host.graph(); empty apply_filter clears to Show all | Task 7 |
| `crates/canvas/pictograph/src/canvas/view.rs` | scope_to_members resolves resident members and clears an empty set; clear_scope explicitly shows all | Task 4 |

New files, each annotated at its exact planned occurrence:

- `crates/system/pandect/src/live_graph.rs` *(planned target)* <!-- doc-audit: planned-path -->: portable typed command/precondition vocabulary and preflight/application helpers.
- `crates/system/pandect/tests/live_graph.rs` *(planned target)* <!-- doc-audit: planned-path -->: complete-writer live/observation parity tests.
- `ports/graphshell/src/resident_host.rs` *(planned target)* <!-- doc-audit: planned-path -->: host settings, pane roots, refresh scheduling and staged state, beside MereHost.
- `ports/graphshell/src/web_resident.rs` *(planned target)* <!-- doc-audit: planned-path -->: browser controls, foreground/idle pump and lifecycle-safe async work.
- `ports/graphshell/tests/resident_host.rs` *(planned target)* <!-- doc-audit: planned-path -->: native owned-host failure/reopen and memory fixture.
- `ports/graphshell/tests/resident_host_browser.rs` *(planned target)* <!-- doc-audit: planned-path -->: wasm/browser host integration over real IndexedDB.

Do not create a second writer, backend service, persistence log or a competing projection definition format. Module exports change in existing `crates/system/pandect/src/lib.rs` and `ports/graphshell/src/lib.rs`; browser wiring uses existing `ports/graphshell/src/web.rs`.

## Phases and done conditions

| Phase | Tasks | Done condition |
|---|---|---|
| Pure publication and observed live edits | 1–2 | Identical record encoding; complete-candidate edits are atomic and observation-checked; named live operations preserve placement/Author/capture |
| Owned durable publication | 3 | Supported owned hosts write checkpoint and publication in one batch; explicit outcomes and fixed-batch recovery prevent duplicate edits |
| Scoped presentation and refresh | 4–5 | Two-pane union, exact roster and graph settings work; head-only unchanged checks and token-checked candidate install preserve current view/context |
| Endpoint and shell adoption | 6–7 | Coverage and actions traverse a real Djinn endpoint/client; host controls and browser edit/refresh/storage status are usable |
| Qualification and review | 8 | Native reopen/undo and actual Firefox IndexedDB pass; memory/IO evidence is separate; fresh whole-candidate review has no unresolved required finding |

Dependency chain: Task 1 → Task 2; Tasks 1–2 → Task 3; Task 3 → Task 4 → Task 5; Tasks 4–5 → Task 6 → Task 7 → Task 8. Task 6 wire types could be prepared independently, but snapshot constructors and host modules remain single-writer tasks. Root assigns tasks and all Cargo runs after plan approval.

### Task 1: Separate preparation from publication and expose observations

**Files:** Modify addressable_graph `mod.rs`, `write.rs`, `schema.rs`; test in existing `crates/system/pandect/tests/graph_residency.rs`.

**Consumes:** existing Graph::to_snapshot_at(0), StoredNode encoding/hash, WriteOp::{Put,Delete}.

**Produces (proposed Rust API notation; method bodies belong to implementation):**

```rust
pub struct PreparedGraphPublication {
    ops: Vec<muniment::WriteOp>,
    generation: String,
    catalog: schema::Catalog,
}
// AddressableGraphStore<B>, B: Backend
pub fn prepare_recorded_graph(&self, graph: &Graph)
    -> Result<PreparedGraphPublication, StoreError>;
// PreparedGraphPublication
pub fn ops(&self) -> &[muniment::WriteOp];
pub fn generation(&self) -> &str;
pub fn record_digest(&self, address: GraphAddress) -> Option<&str>;
pub fn shown_resource(&self, surface: uuid::Uuid) -> Option<uuid::Uuid>;
// ResidentGraph<B>
pub fn record_digest(&self, address: GraphAddress) -> Option<&str>;
pub fn shown_resource(&self, surface: uuid::Uuid) -> Option<uuid::Uuid>;
```

The private Catalog uses the existing schema type inside addressable_graph. Use the existing catalog's complete incident records for observations. Never derive these hashes from ResidentGraph::graph().

- [x] Add this compile-ready test to the existing graph_residency integration test, which already supplies source(), roots(), Backend and MemoryBackend:

```rust
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn preparation_has_no_writes_and_observations_use_complete_records() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let (graph, surfaces, _) = source(8);
        let graph_id = Uuid::from_u128(1400);
        let store = AddressableGraphStore::new(backend.clone(), graph_id);
        let prepared = store.prepare_recorded_graph(&graph).unwrap();
        assert!(backend.list(&format!("graphs/{graph_id}/resident/v1/")).await.unwrap().is_empty());
        let address = GraphAddress::Surface(surfaces[0]);
        let expected = prepared.record_digest(address).unwrap().to_string();
        backend.apply(prepared.ops()).await.unwrap();
        store.set_policy(ResidencyPolicy::Neighborhood { hops: 0 }).await.unwrap();
        let view = store.open(roots(surfaces[0])).await.unwrap();
        assert_eq!(view.record_digest(address), Some(expected.as_str()));
        assert_eq!(view.generation(), prepared.generation());
        assert!(view.graph().node_count() < graph.node_count());
    });
}
```

- [x] Root runs `cargo test --locked -p pandect --test graph_residency preparation_has_no_writes_and_observations_use_complete_records -- --exact`; expect RED because preparation/accessors do not exist.
- [x] Extract the current serializer into the pure preparation function without changing JSON shape, edge indexes/order, navigation/pending ownership or digest rules. Preserve the existing write_recorded_graph public API by having it prepare, apply once, return generation. Add tests comparing prepared ops with current complete publication and rejecting Residency-limited/unrecorded input before writes.
- [x] Add a Surface with the same UUID as a Resource to the existing identity controls; assert separate digests and shown mapping. Add an incident edge whose other endpoint is unloaded; assert observed digest matches the complete publication even though the resident edge is filtered.
- [x] Root runs the named RED test, `cargo test --locked -p pandect --test graph_residency`, and the existing Pandect library tests. Require GREEN and exact recorded-snapshot round-trip.
- [x] Reviewer checks unchanged encoding and no partial-record hashing; root commits this task locally after review.

**Done:** preparation performs zero backend writes and provides complete record observations, with old publication behavior intact.

### Task 2: Typed live batches with mandatory observation checks

**Files:** Create the planned live_graph module/test; modify existing Pandect lib exports and GraphSession only for the facade. Do not change kernel live/replay semantics.

**Consumes:** Task 1 preparation/digests; existing GraphSession::edit_now; Graph::set_recorder(None), write_as, with_isolated_capture, node_content_tags and shown_resource_id; GraphDelta live variants and Author::asserter_iri().

**Produces (compile-ready vocabulary; method signature is proposed API notation):**

```rust
#[derive(Clone, Debug)]
pub enum LiveCommand {
    CreateAddress { id: uuid::Uuid, address: String, title: String },
    EditNode { surface: uuid::Uuid, title: String, tags: std::collections::BTreeSet<String> },
    SetFacet { surface: uuid::Uuid, facet: String, value: serde_json::Value },
    SetPinned { surface: uuid::Uuid, pinned: bool },
    SetMimeHint { surface: uuid::Uuid, mime_hint: Option<String> },
    AssertRelation { from: uuid::Uuid, to: uuid::Uuid, assertion: kernel::graph::EdgeAssertion },
}
#[derive(Clone, Debug)]
pub enum LiveCheck {
    Exists(crate::addressable_graph::GraphAddress),
    Absent(crate::addressable_graph::GraphAddress),
    RecordUnchanged { address: crate::addressable_graph::GraphAddress, digest: String },
    ShownUnchanged { surface: uuid::Uuid, resource: uuid::Uuid },
    ContentTagsUnchanged { surface: uuid::Uuid, tags: std::collections::BTreeSet<String> },
}
#[derive(Clone, Debug)]
pub struct ObservedCommand { pub command: LiveCommand, pub checks: Vec<LiveCheck> }
#[derive(Debug)]
pub enum LiveEditError {
    Session(crate::SessionError),
    Missing(crate::addressable_graph::GraphAddress),
    Stale(crate::addressable_graph::GraphAddress),
    MissingObservation(crate::addressable_graph::GraphAddress),
    InvalidCommand(String),
    NeedsCompleteOperation,
}
#[derive(Debug)]
pub struct LiveApplied { pub applied: crate::Applied, pub created: Vec<uuid::Uuid> }
// GraphSession<B>, B: Backend + Clone
pub fn edit_live_now(&mut self, author: kernel::graph::Author,
    commands: Vec<ObservedCommand>) -> Result<LiveApplied, LiveEditError>;
```

- [x] Add these compile-ready test functions to the planned integration test. Imports are `kernel::graph::{Author, Graph, apply::add_node}`, `muniment::MemoryBackend`, `pandect::{MereSessions, addressable_graph::GraphAddress, live_graph::*}`, and `uuid::Uuid`.

```rust
#[test]
fn batch_can_edit_an_earlier_creation_without_resident_keys() {
    let mut session = MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), None);
    let id = Uuid::from_u128(2000);
    let result = session.edit_live_now(Author::person("alice"), vec![
        ObservedCommand {
            command: LiveCommand::CreateAddress { id, address: "https://live.test/new".into(), title: " New ".into() },
            checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
        },
        ObservedCommand {
            command: LiveCommand::SetPinned { surface: id, pinned: true },
            checks: vec![LiveCheck::Exists(GraphAddress::Surface(id))],
        },
    ]).unwrap();
    assert_eq!(result.created, vec![id]);
    assert_eq!(session.graph().node_is_pinned(session.graph().get_node_key_by_id(id).unwrap()), Some(true));
    assert!(result.applied.end > result.applied.first);
}

#[test]
fn later_missing_target_rejects_entire_batch_and_capture() {
    let mut session = MereSessions::new(MemoryBackend::new()).begin_recorded(Author::user(), None);
    let before = serde_json::to_vec(&session.graph().to_snapshot_at(0)).unwrap();
    let cursor = session.journal().live_cursor();
    let revision = session.revision();
    let id = Uuid::from_u128(2001);
    let missing = Uuid::from_u128(2002);
    assert!(matches!(session.edit_live_now(Author::user(), vec![
        ObservedCommand {
            command: LiveCommand::CreateAddress { id, address: "https://live.test/refused".into(), title: String::new() },
            checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
        },
        ObservedCommand {
            command: LiveCommand::SetPinned { surface: missing, pinned: true },
            checks: vec![LiveCheck::Exists(GraphAddress::Surface(missing))],
        },
    ]), Err(LiveEditError::Missing(GraphAddress::Surface(value))) if value == missing));
    assert_eq!(serde_json::to_vec(&session.graph().to_snapshot_at(0)).unwrap(), before);
    assert_eq!(session.journal().live_cursor(), cursor);
    assert_eq!(session.revision(), revision);
    // A following valid edit must not collect the refused creation's captures.
    let result = session.edit_live_now(Author::user(), vec![ObservedCommand {
        command: LiveCommand::CreateAddress { id, address: "https://live.test/accepted".into(), title: String::new() },
        checks: vec![LiveCheck::Absent(GraphAddress::Surface(id))],
    }]).unwrap();
    assert_eq!(result.created, vec![id]);
    assert_eq!(session.graph().node_count(), 1);
}
```

- [x] Root runs `cargo test --locked -p pandect --test live_graph`; expect RED for missing module/facade.
- [x] Implement preflight on a complete scratch clone with its per-graph recorder cleared and thread capture isolated. Check externally supplied record/shown/tag observations against the original complete source first. Then validate target existence, earlier creations and normalized operations in sequence on scratch; references to records created in the batch use scratch existence rather than invented original digests. Return a prevalidated stable-ID command program. Reject missing targets before mutation. Reject omitted mandatory checks for existing records; an Exists-only check is permitted for a record created within this batch. Do not accept an arbitrary closure, replay record or raw record replacement through this API.
- [x] Run the prevalidated program under one GraphSession::edit_now call, resolving keys anew **inside its complete candidate**. Preflight and application have no await, external callbacks or writer interleaving. Do not return a fallible Result from edit_now's closure and assume Err rolls it back: the closure's result is only payload. All input-driven refusal happens in preflight; product validator/replay errors still occur before session installation. The application program contains only already-validated operations and writer-local IDs.
- [x] Preserve URL lookup before absence checking for CreateAddress. A hit returns the existing UUID and creates no new Surface. Later batch references use actual stable IDs, not an implicit alias from a requested UUID to that hit. A miss uses AddNode { id: Some(id), url, position: Default::default() } and trimmed nonempty SetNodeTitle.
- [x] EditNode requires Surface existence, unchanged shown binding and unchanged observed Surface/Resource digests plus content tags. Normalize/trim tags; compute removals from complete node_content_tags. Use SetNodeTitle, RemoveNodeTag and InsertNodeTag. SetFacet/SetPinned/SetMimeHint require Surface record checks. Require qualified session placement for this partial-command facade, including recorded sessions and legacy sessions admitted through their validated durable translation receipt, in addition to existing session writability. SetMimeHint uses the existing metadata operation. Relation checks distinguish Resource placement for Cites/Hyperlink from Surface placement for UserGrouped/CollectionMember/FrameMember; use the existing effective placement authority for other relations. Emit AssertRelation with writer keys and current Author's asserter IRI.
- [x] Add table-driven parity tests for all six commands against direct complete live deltas, with two authors' tags, parallel/self assertions and UUID-stratum overlap. Compare full snapshots, facets, navigation and exact captured statements; flush/reopen and undo retain identical history. Test unqualified placement, read-only codicil, validator refusal, stale shown binding, changed hidden incident record, invalid decoded command semantics, unsupported Complete-operation dependencies, and a positive unrelated-source edit with unchanged observations. This facade takes decoded UUID/JSON values and never admits raw replay. Malformed external UUID/JSON text is tested at Task 7's actual product boundary before any captured replay parser; imports remain Complete-only.
- [x] Root runs `cargo test --locked -p pandect --test live_graph`, `cargo test --locked -p pandect --lib`, and graph_residency tests. Reviewer checks live/replay differences and capture isolation. Root commits locally.

**Done:** no resident NodeKey crosses the adapter; rejected batches leave writer/history/reader untouched; exact live semantics survive replay/undo.

### Task 3: One retained checkpoint/publication batch under owned storage

**Files:** Modify GraphSession residency helper, MereHost staging and web_session. Add staging state in the planned resident_host module and native host test. Preserve current Staged's retired-session handling.

**Consumes:** Task 1 pure publication; Pending::ops, GraphSession::pending_with(at,true)/stored, Applied { first,end,revision }; existing BrowserHost prepare/write/install and Djinn host cache.

**Produces (proposed Rust API notation; method bodies belong to implementation):**

```rust
pub struct PreparedResidentCheckpoint {
    pending: crate::Pending,
    publication: crate::addressable_graph::PreparedGraphPublication,
    ops: Vec<muniment::WriteOp>,
    session: crate::SessionId,
    graph: uuid::Uuid,
    revision: u64,
    cursor: u64,
}
// GraphSession<B>, B: Backend + Clone
pub fn prepare_resident_checkpoint(&self, at: std::time::SystemTime)
    -> Result<PreparedResidentCheckpoint, crate::SessionError>;
pub fn acknowledge_resident_checkpoint(&mut self, prepared: PreparedResidentCheckpoint)
    -> Result<String, crate::SessionError>;
// PreparedResidentCheckpoint
pub fn ops(&self) -> &[muniment::WriteOp];
pub fn session_id(&self) -> crate::SessionId;
pub fn source_revision(&self) -> u64;
pub fn source_cursor(&self) -> u64;
pub fn generation(&self) -> &str;

pub enum ResidentPersistence {
    Rejected,
    AppliedButUnstored { first: u64, end: u64, revision: u64 },
    Durable { generation: String },
    PendingPublication { generation: String, reason: String },
}
pub enum ResidentHostError {
    Live(pandect::live_graph::LiveEditError),
    Host(crate::mere_host::MereHostError),
    Busy,
    OwnershipUnproven,
    UnsupportedStorage,
    StaleCompletion,
    MixedCommit,
}
```

Prepared fields are private to the session staging boundary. Prepared state records session/root graph identity, cursor, revision and generation; combined ops include the current Pending forced checkpoint plus addressable materialization. Retired session batches precede the current batch and have their own acknowledgement; never claim atomicity across those independent sessions.

- [ ] Write the compile-ready RED test in existing graph_residency tests:

```rust
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn prepared_checkpoint_does_not_acknowledge_or_reapply() {
    pollster::block_on(async {
        let backend = MemoryBackend::new();
        let mut session = pandect::MereSessions::new(backend.clone()).begin_recorded(kernel::graph::Author::user(), None);
        let id = Uuid::from_u128(3000);
        session.edit_now(kernel::graph::Author::user(), |graph| {
            apply::add_node(graph, Some(id), "https://commit.test".into(), Default::default());
        }).unwrap();
        let cursor = session.journal().live_cursor();
        let prepared = session.prepare_resident_checkpoint(std::time::SystemTime::UNIX_EPOCH).unwrap();
        assert!(session.has_unstored());
        assert!(backend.list("graphs/").await.unwrap().is_empty());
        backend.apply(prepared.ops()).await.unwrap();
        let generation = session.acknowledge_resident_checkpoint(prepared).unwrap();
        assert!(!session.has_unstored());
        assert_eq!(session.journal().live_cursor(), cursor);
        let view = pandect::MereSessions::new(backend).open_resident_graph(session.id(), Default::default()).await.unwrap();
        assert_eq!(view.generation(), generation);
        assert_eq!(view.graph().node_count(), 1);
    });
}
```

- [ ] Root runs the exact named graph_residency test; expect RED for missing staging API.
- [ ] Expose forced-checkpoint preparation through GraphSession's residency module; verify qualification and reject Residency-limited graph before preparation. Make acknowledge refuse changed session/root graph/cursor/revision before calling stored. Keep existing public publish_resident_checkpoint as a compatibility path; the owned host adopts combined staging rather than pretending its two writes are atomic.
- [ ] Admit resident combined writes through concrete host factories, not a caller-supplied bool or generic Backend label. Djinn's proven owner is MereShared::host and its cached Directory lifetime. Browser composition's one host and concrete IndexedDbBackend are the browser path. MemoryBackend is test-only admission; native redb admission requires its opened exclusive handle. Keep constructors granting admission private to these composition paths. Arbitrary Backend, Fjall, Zip, unsupported wrappers or an independent writer receive OwnershipUnproven/UnsupportedStorage before resident edits. No new universal backend capability interface is needed.
- [ ] Pause writer edits, undo/redo, view writes, session switch and detach publication while the current combined batch is prepared/in flight or uncertain. Allow cancellation of a refresh candidate, not cancellation that loses an uncertain write. Read-only last coherent presentation remains usable. Preparation/storage failure after a successful edit reports AppliedButUnstored with its existing Applied identity, retaining the exact batch if prepared. A confirmed batch is Durable; failed reader refresh is PendingPublication and retries only refresh. Settings-only publication may also be pending independently of an edit.
- [ ] Retain per-key pre-write values before the first apply, under the existing exclusive writer ownership, alongside the in-memory batch. On uncertain completion compare all intended writes and preimages: all final values match → acknowledge once; all values equal their preimages or their unchanged final values → retry the **same** ops; any partial/conflicting mix → MixedCommit, stop writing and reopen/recover. Directory poison requires dropping all owner clones and reopening redo recovery before comparison. No persistent receipt log is added. After host loss, reopen authoritative history and show current truth; never resubmit the old command.
- [ ] Add fault-backend tests for preparation codec failure, failure before apply, apply-success/error response, mixed keys, stale acknowledgement, and retry twice. Assert journal cursor/change count do not grow on retry, old reader survives, Busy prevents a second edit and recovered reopen retains exact navigation/facets/Author/undo. Use Backend wrappers confined to tests around MemoryBackend; do not qualify them as production storage.
- [ ] Root runs named test, complete Pandect tests, Graphshell library tests and native resident_host integration tests serially. Reviewer checks write identity, retired-session acknowledgement, backend admission, browser borrow lifetime and outcomes. Root commits locally.

**Done:** current session checkpoint and publication share a supported batch; storage uncertainty has a concrete recoverable state; the UI cannot apply twice.

### Task 4: Graph settings, pane-root union and exact authored scope

**Files:** Modify resident_host and MereHost, addressable_graph roots/selection, existing `crates/canvas/pictograph/src/canvas.rs`, `crates/canvas/pictograph/src/canvas/view.rs`, `crates/canvas/pictograph/src/canvas/lifecycle.rs` and `crates/canvas/pictograph/src/canvas/tests/scope_and_cartography.rs`; Djinn attaches/releases pane state through its existing endpoint lifetime.

**Consumes:** existing ViewKey, GraphAddress, ResidencyRoots and policy; selected projection's authored stable-ID roster. Do not persist transient focus into graph pins.

**Produces (proposed Rust API notation; method bodies belong to implementation):**

```rust
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ResidentRefreshMode { Manual, HostDriven { minimum_check_ms: u64 } }
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ResidentSettings {
    pub policy: pandect::addressable_graph::ResidencyPolicy,
    pub projection: Option<String>,
    pub refresh: ResidentRefreshMode,
    pub publication_batch_ms: u64,
}
#[derive(Clone, Debug, Default)]
pub struct PaneRoots {
    pub roster: std::collections::BTreeSet<uuid::Uuid>,
    pub focus: Option<pandect::addressable_graph::GraphAddress>,
}
#[derive(Default)]
pub struct HostRoots {
    panes: std::collections::BTreeMap<pandect::ViewKey, PaneRoots>,
    pins: std::collections::BTreeSet<pandect::addressable_graph::GraphAddress>,
    revision: u64,
}
// HostRoots
pub fn set_pane(&mut self, pane: pandect::ViewKey, roots: PaneRoots);
pub fn close_pane(&mut self, pane: &pandect::ViewKey);
pub fn set_pins(&mut self, pins: std::collections::BTreeSet<pandect::addressable_graph::GraphAddress>);
pub fn union(&self) -> pandect::addressable_graph::ResidencyRoots;
pub fn revision(&self) -> u64;
// Add ResidencyRoots::transient: BTreeSet<GraphAddress>, retaining focused compatibility.
// Canvas
pub fn scope_to_exact_members(&mut self, members: impl IntoIterator<Item = uuid::Uuid>);
// MereHost<B>
pub fn presentation_graph(&self) -> &mere::kernel::graph::Graph;
```

- [ ] Add this compile-ready unit test inside resident_host, importing its proposed types plus BTreeSet, ViewKey, GraphAddress and Uuid:

```rust
#[test]
fn closing_one_pane_releases_only_its_transient_roots() {
    let first = ViewKey::new("graphshell", "first").unwrap();
    let second = ViewKey::new("graphshell", "second").unwrap();
    let a = Uuid::from_u128(4000);
    let b = Uuid::from_u128(4001);
    let focus = GraphAddress::Resource(Uuid::from_u128(4002));
    let pin = GraphAddress::Surface(Uuid::from_u128(4003));
    let mut roots = HostRoots::default();
    roots.set_pins(BTreeSet::from([pin]));
    roots.set_pane(first.clone(), PaneRoots { roster: BTreeSet::from([a]), focus: Some(focus) });
    roots.set_pane(second, PaneRoots { roster: BTreeSet::from([a, b]), focus: Some(focus) });
    assert_eq!(roots.union().projection_surfaces, BTreeSet::from([a, b]));
    assert_eq!(roots.union().transient, BTreeSet::from([focus]));
    assert_eq!(roots.union().pinned, BTreeSet::from([pin]));
    roots.close_pane(&first);
    assert_eq!(roots.union().projection_surfaces, BTreeSet::from([a, b]));
    assert_eq!(roots.union().transient, BTreeSet::from([focus]));
    assert_eq!(roots.union().pinned, BTreeSet::from([pin]));
}
```

- [ ] Root runs `cargo test --locked -p graphshell --features web --lib resident_host::tests::closing_one_pane_releases_only_its_transient_roots -- --exact`; expect RED for absent module/transient roots.
- [ ] Persist graph settings in existing JsonSlots under `graphs/{root_graph_id}/resident/v1/host-settings`; Full, Manual and publication_batch_ms=0 are compatibility defaults. A positive minimum interval is required for HostDriven; reject zero rather than busy-polling. Validate a selected authored projection before Neighborhood is enabled. Keep P5's existing policy key coherent with host settings by staging both together, without two competing settings authorities.
- [ ] Add transient focus union to P5 selection, with zero-hop shown dependencies; it retains individual focus/pins without treating them as projection-walk roots. Keep authored rosters intact in HostRoots even when catalog members are absent or not resident. Selection skips absent requested roster members, reports their absence as host Projection context and never changes the authored roster. Known catalog members outside residency remain NotLoaded. Do not turn an absent item into an empty successful demand.
- [ ] Add a separate private `exact_scope: Option<BTreeSet<Uuid>>` beside the current runtime `scope: Option<Vec<NodeKey>>` in Canvas. scope_to_exact_members stores authored IDs and resolves runtime keys, preserving Some(empty) even when nothing is resident. Re-resolve it in lifecycle::set_graph; scope_members returns the authored IDs when exact_scope is present. Explicit scope_to_members/isolate_selection clear exact_scope, and clear_scope clears both. This preserves empty/missing IDs across source changes; clear_scope remains explicit Show all. Do not call existing scope_to_members for this path. Add scope tests: empty roster yields zero visible nodes with a nonempty graph; unloaded/missing roster survives a graph swap and becomes visible after demand; clear_scope alone restores all. Keep's view state is unchanged.
- [ ] Make presentation_graph choose installed resident data when residency is active, otherwise the Full writer. Leave MereHost::graph() Full. Session close releases the closing pane's roots; shared projections/focus/pins remain. Give each admitted endpoint a distinct existing ViewKey rather than the untrusted application payload's claimed identity.
- [ ] Root runs Graphshell resident_host tests, `cargo test --locked -p pictograph --features canvas --lib scope`, and graph_residency tests. Review graph-settings persistence, exact scope, pin/Keep separation and session/pane ownership; root commits locally.

**Done:** scoped roots demonstrably retain less reader data; setting Full clears computed Residency limits while preserving other coverage layers.

### Task 5: Head-only checks and lifecycle-checked refresh installation

**Files:** Modify addressable_graph `schema.rs`/`residency.rs`, resident_host and browser pump. Extend existing graph_residency counters/cache tests rather than inventing a second query cache.

**Consumes:** Task 4 roots/settings; generation/head encoding; existing available_digest, Graph::revision(), semantic_observation_revision(), set_host_coverage and refresh candidate construction.

**Produces (proposed Rust API notation; method bodies belong to implementation):**

```rust
pub enum RefreshOutcome { Unchanged, SourceOnly, ObservationOnly, TruthChanged }
pub struct ResidentRefreshCandidate {
    generation: String,
    catalog: schema::Catalog,
    records: std::collections::BTreeMap<GraphAddress, schema::StoredNode>,
    roots: ResidencyRoots,
    demands: std::collections::BTreeSet<GraphAddress>,
    policy: ResidencyPolicy,
    snapshot: kernel::persistence::GraphSnapshot,
    facets: kernel::graph::NodeFacetStore,
    coverage: kernel::graph::CoverageNote,
    footprint: ResidentFootprint,
    pending: kernel::graph::PendingLinkState,
    available_digest: String,
}
pub struct ResidentRefreshRequest<B> {
    store: AddressableGraphStore<B>,
    previous_generation: String,
    previous_catalog: Option<schema::Catalog>,
    previous_records: std::collections::BTreeMap<GraphAddress, schema::StoredNode>,
    roots: ResidencyRoots,
    demands: std::collections::BTreeSet<GraphAddress>,
    policy: ResidencyPolicy,
}
// ResidentGraph<B>, B: Backend + Clone, creates an owned request without awaiting.
pub fn refresh_request(&self, roots: ResidencyRoots, policy: ResidencyPolicy)
    -> ResidentRefreshRequest<B>;
// ResidentRefreshRequest<B>, B: Backend
pub async fn load(self) -> Result<ResidentRefreshCandidate, muniment::StoreError>;
// ResidentGraph<B>
pub async fn refresh_if_changed(&mut self) -> Result<RefreshOutcome, muniment::StoreError>;
pub async fn prepare_refresh(&self, roots: ResidencyRoots, policy: ResidencyPolicy)
    -> Result<ResidentRefreshCandidate, muniment::StoreError>;
pub fn install_refresh(&mut self, candidate: ResidentRefreshCandidate)
    -> Result<RefreshOutcome, muniment::StoreError>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RefreshToken {
    pub lifecycle: u64,
    pub session: pandect::SessionId,
    pub graph: uuid::Uuid,
    pub roots_revision: u64,
    pub policy_revision: u64,
    pub request: u64,
}
#[derive(Default)]
pub struct RefreshGate {
    in_flight: Option<(RefreshToken, u64)>,
    newest_notice: u64,
    consumed_notice: u64,
    invalidated: bool,
}
// RefreshGate
pub fn notice(&mut self, sequence: u64);
pub fn begin(&mut self, token: RefreshToken) -> bool;
pub fn finish(&mut self, token: &RefreshToken, current: &RefreshToken) -> bool;
pub fn needs_check(&self) -> bool;
```

The candidate is owned data, with no borrow held across host callbacks. Use refresh_request with a cloned backend and owned input/catalog/old records for awaited load; `prepare_refresh(&self,...)` is the library convenience wrapper. Browser code must not hold `RefCell` borrow for its duration. install_refresh composes **current** host coverage rather than candidate-captured context, and retains the existing graph instance/cache if available truth digest is unchanged.

- [ ] Add this compile-ready test inside resident_host tests:

```rust
#[test]
fn a_new_notice_survives_completion_and_switch_rejects_install() {
    let session = pandect::MereSessions::new(muniment::MemoryBackend::new())
        .begin_recorded(mere::kernel::graph::Author::user(), None).id();
    let old = RefreshToken { lifecycle: 1, session, graph: uuid::Uuid::from_u128(5000),
        roots_revision: 1, policy_revision: 1, request: 1 };
    let mut gate = RefreshGate::default();
    gate.notice(1);
    assert!(gate.begin(old.clone()));
    assert!(!gate.begin(old.clone()));
    gate.notice(2);
    assert!(gate.finish(&old, &old));
    assert!(gate.needs_check());
    let mut switched = old.clone();
    switched.lifecycle += 1;
    switched.request += 1;
    assert!(gate.begin(old.clone()));
    assert!(!gate.finish(&old, &switched));
    assert!(gate.needs_check());
}
```

- [ ] Root runs the exact Graphshell test; expect RED. Add a counter-backend graph_residency test for refresh_if_changed: unchanged generation increments only head get, not catalog/record/navigation/pending reads; roots/policy changes still reconcile even with identical head.
- [ ] Split candidate load/install while preserving checksum/placement/incident-copy validation. New head with unchanged available_digest is SourceOnly unless context/pending changes make it ObservationOnly; changing available truth is TruthChanged. Host context changes use set_host_coverage independently. Failures discard candidates, keep current generation/view and leave pending check. Policy writes occur only after a valid candidate and use staged settings, not speculative persistence.
- [ ] Drive one in-flight request from manual refresh, confirmed local publication, foreground/attachment and elapsed minimum_check_ms. Store publication_batch_ms separately; publish at existing idle/store boundaries, explicit save and detach flush. Notices coalesce by a monotonic host sequence; completion consumes only the sequence it started with. Newer notices queue another check. Ignore backward/nonfinite browser clocks; use host monotonic elapsed input, not a library timer/thread.
- [ ] Invalidate token on session/root/policy/lifecycle changes. Preserve latest host coverage when a coherent earlier source candidate installs; do not clear a new notice while acknowledging old work. Demand is inspection only and uses the same token discipline. A demand adds no durable pin. Suspended browser tabs perform no work until resume, then check immediately.
- [ ] Add controlled-await tests for roots/policy/session switches, newer notice, context change during load, read failure, demand failure, notice bursts and foreground catch-up. Extend existing P5 unloaded-only/context cache controls so successful query evaluation count and truth revision remain stable for SourceOnly/Pending/context updates, but increase when loaded truth changes.
- [ ] Root runs Graphshell resident_host tests and full graph_residency native suite. Reviewer checks single-flight/sequence lifecycle and graph/cache retention; root commits locally.

**Done:** unchanged polling reads only head; failed/stale candidates never replace the coherent view; latest observation and notice state survives.

### Task 6: Product-neutral coverage through endpoint and client

**Files:** Modify Chirograph lib, Graphshell snapshot producers, Djinn snapshot/notice logic, graphshell-client lib/persistence and existing portable client tests. Update all ProjectionSnapshot literals in this workspace explicitly; no unrelated consumer source edits.

**Consumes:** kernel CoverageLayer::{Possession,Residency,Disclosure,Synchronization,Projection} and CoverageNote; presentation_graph and host projection revision; existing snapshot/client/carrier interfaces.

**Produces (proposed Rust API notation; method bodies belong to implementation):**

```rust
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortableCoverageLayerV1 { Possession, Residency, Disclosure, Synchronization, Projection }
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PortableCoverageLimitV1 {
    pub layer: PortableCoverageLayerV1,
    pub reason: String,
    #[serde(default)] pub surfaces: Vec<String>,
    #[serde(default)] pub resources: Vec<String>,
    #[serde(default)] pub count: Option<u64>,
}
#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PortableCoverageV1 { pub limits: Vec<PortableCoverageLimitV1> }
// ProjectionSnapshot and MountedScene add this field:
#[serde(default, skip_serializing_if = "Option::is_none")]
pub coverage: Option<PortableCoverageV1>;
```

Use strings for portable stable IDs and u64 for counts; Pandect/kernel types do not enter Chirograph. Graphshell performs explicit checked conversion from UUIDs/counts. None means unreported; Some(default()) means no known limit **within the supplied graph**.

- [ ] Add this compile-ready Chirograph unit test (existing lib tests already import super::*):

```rust
#[test]
fn coverage_preserves_stratum_and_unreported_is_not_empty_note() {
    let id = "00000000-0000-0000-0000-000000006000".to_string();
    let note = PortableCoverageV1 { limits: vec![PortableCoverageLimitV1 {
        layer: PortableCoverageLayerV1::Residency, reason: "stored graph items are not resident".into(),
        surfaces: vec![id.clone()], resources: vec![id], count: Some(2),
    }] };
    let bytes = serde_json::to_vec(&note).unwrap();
    assert_eq!(serde_json::from_slice::<PortableCoverageV1>(&bytes).unwrap(), note);
    assert_ne!(None::<PortableCoverageV1>, Some(PortableCoverageV1::default()));
}
```

- [ ] Root runs `cargo test --locked -p chirograph coverage_preserves_stratum_and_unreported_is_not_empty_note`; expect RED. Add a real ProjectionSnapshot deserialization control removing the coverage field from existing fixture JSON; assert None. Round-trip supplied empty/all-five-layer notes and same UUID in both strata.
- [ ] Produce snapshots from presentation_graph and carry coverage_note, not a truth facet or fake graph edge. Context-only updates advance the host's **projection** revision/notice, not the resident truth revision. Djinn must notice changes in coverage and Pending observation even when truth is unchanged; reset per-endpoint seen state on attach/switch. Preserve existing admitted/ambient policy checks.
- [ ] Store coverage in MountedScene and default it in persisted projection decoding; clear it with the existing session lifecycle. A context-only notice forces a fresh snapshot through existing resnapshot/resume handling so coverage need not be invented as a scene diff. Retain successful scene resources/cache when unchanged and update the observation/presentation status. No new carrier transport or protocol version is required for the defaultable snapshot field.
- [ ] Add this compile-ready test to existing graphshell-client lib tests, where snapshot_with_offer and semantics are verified fixture helpers:

```rust
#[test]
fn context_only_coverage_updates_preserve_cached_resources() {
    use chirograph::{PortableCoverageV1, PortableCoverageLimitV1, PortableCoverageLayerV1};
    let session = ProjectionSession("resident:coverage".into());
    let bytes = serde_json::to_vec(&NativeGlyphV1 { label: "One".into(), icon: None, color: None }).unwrap();
    let mut snapshot = snapshot_with_offer(&session, PresentationCodec::NativeGlyphV1,
        PresentationCapability::NativeGlyph, semantics("One", SemanticRole::Graphic),
        &bytes, CacheRetention::MemoryOnly);
    snapshot.coverage = Some(PortableCoverageV1::default());
    let hash = ContentHash::of(&bytes);
    let mut client = ClientState::default();
    client.apply_snapshot(snapshot.clone()).unwrap();
    client.apply_resource(ResourceResponse::new(session.clone(), bytes.clone())).unwrap();
    snapshot.scene.revision = Revision(snapshot.scene.revision.0 + 1);
    snapshot.coverage = Some(PortableCoverageV1 { limits: vec![PortableCoverageLimitV1 {
        layer: PortableCoverageLayerV1::Residency, reason: "not loaded".into(),
        surfaces: vec!["00000000-0000-0000-0000-000000006001".into()],
        resources: vec![], count: Some(1),
    }] });
    client.apply_snapshot(snapshot.clone()).unwrap();
    assert_eq!(client.mounted(&session).unwrap().coverage, snapshot.coverage);
    assert_eq!(client.mounted(&session).unwrap().scene.tables, snapshot.scene.tables);
    assert_eq!(client.resources.get(&(session.clone(), hash)), Some(&bytes));
    snapshot.coverage = None;
    client.apply_snapshot(snapshot).unwrap();
    assert_eq!(client.mounted(&session).unwrap().coverage, None);
}
```

- [ ] Root runs `cargo test --locked -p graphshell-client context_only_coverage_updates_preserve_cached_resources`; expect RED before wire/client fields and GREEN after. Add persisted old-snapshot, close/revoke and ambient-action controls.
- [ ] In Djinn's existing resident_mere tests, reuse fixture(), attach() and graph() to serve two admitted endpoints sharing one host. Configure scoped residency via the owner path; feed the actual snapshot to ClientState::apply_snapshot; assert NotLoaded coverage, demand action and later coverage-only notice/client update. Keep existing session ownership/admission/refusal tests green.
- [ ] Root runs `cargo test --locked -p chirograph`, `cargo test --locked -p graphshell-client`, `cargo test --locked -p djinn --lib resident_mere`, and Graphshell library tests. Review old-wire meaning and observation/truth separation; root commits locally.

**Done:** endpoint clients receive and retain the five coverage meanings, including context-only changes, without claiming missing metadata proves completeness.

### Task 7: Wire usable partial-display edits and host controls

**Files:** Modify product.rs, MereHost, web_product.rs, web_main.rs, web_session.rs, web_view.rs, web.rs and Djinn resident_mere; create planned web_resident module. Add controls in existing `ports/graphshell/web/tree.html`, `ports/graphshell/web/styles.css`, `ports/graphshell/src/view.rs` and `ports/graphshell/src/web_tree/product.rs` where their respective shell renderers produce product controls.

**Consumes:** all preceding APIs; existing ProductError/EditableRelation/LocalFileMetadata/content-reference validation; existing session_item intent routing/Author-via admission.

**Produces (proposed host API):** `MereHost::apply_observed_commands(&mut self, commands: Vec<pandect::live_graph::ObservedCommand>) -> Result<pandect::live_graph::LiveApplied, ResidentHostError>`. It uses the selected host Author/via and never trusts payload provenance.

**Produces:** named partial operations from the approved spec, settings/demand/refresh endpoint actions, visible coverage and persistence state. New intent payloads live beside the existing session_item vocabulary, using decoded typed UUID/JSON at admission. They do not contain Author, NodeKey or storage capability claims.

- [ ] Add this compile-ready RED test in existing product.rs tests, which supplies open_on(), MemoryBackend and Uuid:

```rust
#[test]
fn observed_product_edit_requires_the_observed_record() {
    pollster::block_on(async {
        use pandect::live_graph::{ObservedCommand, LiveCommand, LiveCheck};
        use pandect::addressable_graph::GraphAddress;
        let backend = MemoryBackend::new();
        let mut host = open_on(&backend).await;
        let id = host.create_address("https://product.test/observed", "Original").unwrap();
        let before = serde_json::to_vec(&host.graph().to_snapshot_at(0)).unwrap();
        let cursor = host.graph_session().journal().live_cursor();
        assert!(host.apply_observed_commands(vec![ObservedCommand {
            command: LiveCommand::SetPinned { surface: id, pinned: true },
            checks: vec![LiveCheck::Exists(GraphAddress::Surface(id))],
        }]).is_err());
        assert_eq!(serde_json::to_vec(&host.graph().to_snapshot_at(0)).unwrap(), before);
        assert_eq!(host.graph_session().journal().live_cursor(), cursor);
    });
}
```

- [ ] Root runs `cargo test --locked -p graphshell --features web --lib product::tests::observed_product_edit_requires_the_observed_record -- --exact`; expect RED for absent host adapter.
- [ ] Write failing integration assertions in the planned resident_host tests: open owned host with Full, create three addresses, choose an authored one-member projection and Neighborhood(0), confirm presentation has one Surface plus shown Resource. Invoke observed title/tag/facet/pin/relation commands through the host, assert complete writer truth changes and the old reader stays coherent until publication/refresh. Reopen/undo must match the direct Complete product path. Root runs `cargo test --locked -p graphshell --features web --test resident_host`; expect RED for absent host integration.
- [ ] Adapt existing product operations to build ObservedCommand with Task 1 digests and the correct checks. Expose only supported partial commands; unsupported closure/global/import/delete/withdraw/history/fields/couplings operations return NeedsCompleteOperation or the explicit Full owned-writer path. Keep existing global undo/redo on that same host with their kept-part semantics. Require explicit Complete operation setting instead of silently opening another writer.
- [ ] Keep file content-reference validation and address reuse first. Preserve existing create_file_metadata change grouping: address/title, MIME and the two facets remain the existing separate changes. Hold the host operation boundary over the sequence, stage once at its existing store boundary and do not infer byte custody from resident metadata. Input validation completes before any phase; report already-applied state if a later existing phase refuses, rather than pretending the sequence rolled back.
- [ ] Parse UUID strings and facet JSON fallibly at the external boundary; retain distinct invalid-ID, invalid-JSON, missing, stale, unqualified/read-only/product-validation errors. The typed adapter uses complete lookup for address reuse and tag diffs. Preserve supplied exact relation assertions and capture handles; no selector-family substitute for exact withdrawal is added.
- [ ] Render graph controls: Full/Neighborhood, authored projection selector, hops, Manual/Host-driven, minimum check interval, publication batch interval, explicit Save/Refresh. Render coverage with Unreported/Not loaded/Missing wording and Inspect/Demand actions. Show Applied but unstored, Stored, and Stored; presentation pending distinctly; only the latter retries publication/refresh. Disable mutation controls during combined staging while the coherent reader remains inspectable.
- [ ] Switch canvas/portable scene source from host.graph() to presentation_graph() at source boundaries. Preserve stable authored scope with scope_to_exact_members; all UI targets are stable UUIDs. Return Full compatibility defaults when settings absent. Keep per-view Keep UI unchanged. Foreground/attachment and existing idle/frame events drive Task 5 scheduler; detach/save flush publication cadence and cannot discard an uncertain write.
- [ ] Add endpoint controls to the existing admitted Djinn path with the admitted application as via; ambient route remains read-only. Closing an endpoint releases only its pane roots. Invoke both endpoints through the actual route to prove one writer and union roots, with context-only notice reaching the portable client.
- [ ] Root runs Graphshell/Djinn/client tests and `cargo check --locked -p graphshell --features web`, `cargo check --locked -p djinn`. Review user-visible state/actions, Full fallback and explicit operation refusals; root commits locally.

**Done:** the shell actually exposes residency, edits, manual/host refresh and clear recovery actions. Endpoint/client behavior and local browser behavior are exercised, not inferred from library tests.

### Task 8: Native durable, Firefox and memory qualification

**Files:** Tests in the planned native/browser host tests and existing Djinn resident_mere tests. Root appends qualification receipts to this plan's Findings/Progress and owns index/D2/spec/parent status changes. Add target-wasm test dependencies `wasm-bindgen-test = "=0.3.79"` and `uuid` with `js` in existing `ports/graphshell/Cargo.toml`, matching Pandect's verified browser test floor; no production dependency or backend change. Enable `pictograph`'s existing `canvas` feature for its scope tests.

**Consumes:** complete implemented candidate and reviewed Tasks 1–7. Browser tests use existing wasm_bindgen_test_configure!(run_in_browser), IndexedDbBackend::open(name,"records") and Firefox.

- [ ] Add a compile-ready browser storage test to the planned browser integration file:

```rust
#![cfg(target_arch = "wasm32")]
use mere::kernel::graph::{Author, apply::add_node};
use muniment::Backend;
use pandect::{MereSessions, addressable_graph::{GraphAddress, ResidencyPolicy, ResidencyRoots}};
use uuid::Uuid;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
async fn owned_checkpoint_resident_refresh_and_reopen_use_real_indexeddb() {
    let name = format!("resident-host-plan-{}", Uuid::new_v4());
    let backend = muniment::IndexedDbBackend::open(&name, "records").await.unwrap();
    let mut writer = MereSessions::new(backend.clone()).begin_recorded(Author::person("browser-owner"), None);
    let first = Uuid::from_u128(8000);
    writer.edit_now(Author::person("browser-owner"), |graph| {
        add_node(graph, Some(first), "https://browser.test/one".into(), Default::default());
        add_node(graph, Some(Uuid::from_u128(8001)), "https://browser.test/two".into(), Default::default());
    }).unwrap();
    let pending = writer.prepare_resident_checkpoint(std::time::SystemTime::UNIX_EPOCH).unwrap();
    backend.apply(pending.ops()).await.unwrap();
    writer.acknowledge_resident_checkpoint(pending).unwrap();
    let sessions = MereSessions::new(backend.clone());
    let mut reader = sessions.open_resident_graph(writer.id(), ResidencyRoots {
        projection_surfaces: [first].into(), ..Default::default()
    }).await.unwrap();
    reader.set_policy(ResidencyPolicy::Neighborhood { hops: 0 }).await.unwrap();
    assert_eq!(reader.graph().node_count(), 1);
    assert_eq!(reader.status(GraphAddress::Surface(Uuid::from_u128(8001))), pandect::addressable_graph::ResidencyStatus::NotLoaded);
    assert!(matches!(reader.refresh_if_changed().await.unwrap(), pandect::addressable_graph::RefreshOutcome::Unchanged));
    let reopened_store = muniment::IndexedDbBackend::open(&name, "records").await.unwrap();
    let reopened = pandect::GraphSession::open(reopened_store, writer.id()).await.unwrap();
    assert_eq!(serde_json::to_vec(&reopened.graph().to_snapshot_at(0)).unwrap(), serde_json::to_vec(&writer.graph().to_snapshot_at(0)).unwrap());
}
```

This storage control complements a headed **actual Graphshell shell** run; it alone does not qualify browser UI routing, foreground behavior or borrowed-host safety.

- [ ] Root first runs native test gates serially, then `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner WASM_BINDGEN_USE_BROWSER=1 WASM_BINDGEN_USE_FIREFOX=1 cargo test --locked -p pandect --target wasm32-unknown-unknown --test graph_residency` and the planned Graphshell browser test with the same runner environment and `--no-default-features --features web`. Set the runner explicitly as above; no checked-in workspace runner configuration was found. The Graphshell command is `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner WASM_BINDGEN_USE_BROWSER=1 WASM_BINDGEN_USE_FIREFOX=1 cargo test --locked -p graphshell --no-default-features --features web --target wasm32-unknown-unknown --test resident_host_browser`. If the runner/toolchain is unavailable, install compatible tooling or record that browser qualification remains incomplete; do not substitute Node or Chromium evidence for Firefox IndexedDB.
- [ ] Qualify DirectoryBackend::open(tempdir) and RedbBackend::open(tempfile) separately for prepare/write/reopen/undo, including a recovered Directory interrupted/poisoned batch. Drop all owner handles before reopen. Ensure unsupported Fjall/Zip/wrapper and unproven independent ownership perform no combined writes.
- [ ] Through a real running Djinn route and retained portable client, exercise two panes, scoped roots and closing one, Full clearing Residency, parallel/self statement demand, title/tag/facet/pin/file/relation operations, malformed/stale inputs, context-only notices and foreground catch-up. Record route/client identity and exact candidate commit. Repeat shell interactions in headed Firefox with fresh test IndexedDB, reload/reopen/undo and two sessions; capture visible error/pending states and stale completion refusal.
- [ ] Measure a 64-node fixture with large Surface/Resource payloads at Full and Neighborhood(0/1): retain ResidentFootprint breakdown and counted catalog/record/navigation/pending reads. Record whole process RSS separately, including the owned Full writer and peak candidate/replay work. State that reader retained-record savings are not process-bounded proof.
- [ ] Root runs final package tests/checks serially: `cargo test --locked -p pandect`, `cargo test --locked -p graphshell --features web --lib`, `cargo test --locked -p graphshell-client`, `cargo test --locked -p chirograph`, `cargo test --locked -p djinn --lib`, `cargo check --locked --workspace`, and `cargo check --locked -p graphshell --no-default-features --features web --target wasm32-unknown-unknown`. Reuse qualified caches; record commands, source hashes, target and feature sets. New code/source changes require relevant gates to be rerun.
- [ ] Fresh reviewer audits all five Review Focus classes, approved-spec coverage, exact interfaces, ownership/admission, Complete-operation boundaries and actual endpoint/browser receipts. Address required findings with observed RED/GREEN tests. Root updates canonical docs/D2 coverage, runs `python scripts/mere_doc_audit.py` and `python scripts/mere_doc_judgment_audit.py`, then presents the exact qualified candidate for user approval before any push/merge.

**Done:** source-qualified implementation, real host/client and actual Firefox evidence support each claimed outcome; all deferred scope remains explicit.

## Self-review and coverage (2026-10-09)

| Approved-spec requirement | Plan coverage |
|---|---|
| Owned complete Graphshell/Djinn writer; immutable resident display; Full memory honesty | Constraints, Tasks 3–4 and 8 |
| Typed strata, zero-hop shown/one-hop relations, pins/focus/Keep, exact authored empty/missing roster | Tasks 1, 4, 7–8 |
| Six named live commands; mandatory checks; complete candidate and earlier-in-batch references; errors | Task 2 plus product/external admission Task 7 |
| Live versus replay, Author/tag withdrawal/placement/Resource refresh, exact assertions, undo/global refusal | Tasks 2, 7–8 |
| Prepare/checkpoint/addressable combined batch, edit pause, outcomes, uncertainty/retry/reopen, backend limits | Task 3 plus durable/browser qualification Task 8 |
| Cadence distinct from polling; manual/foreground/host checks; one flight; head fast path; lifecycle/new notices/current context | Task 5 plus real host wiring Task 7 |
| All five coverage meanings, old-wire unreported, context-only client updates, Pending/query-cache distinction | Tasks 5–6 and 8 |
| Usable shell, actual endpoint/client, native durable reopen and Firefox, separate RSS/reader/IO claims | Tasks 7–8 |
| No GC; later protected-reader/scoped-transaction/Turnstone/bounded-writer contracts | Constraints and this coverage record; deliberately deferred |

Dependencies and signatures were checked: tasks consume only verified existing interfaces or declarations introduced by earlier tasks. GraphSession::edit_now does not roll back a closure's Err; preflight and capture isolation are explicit. Backend qualification comes from concrete factories and ownership, not a generic Backend assumption. Settings and observed empty scopes have one durable/one authored authority each. Fresh source/P5 receipts are not treated as new host qualification. No independent persistence receipt log, backend service or universal writer was introduced.

## Findings

- **2026-10-09:** Existing addressable write serializes complete incident copies before applying; those digests are the correct observation authority, not filtered resident edges (`addressable_graph/write.rs`, `schema.rs`, `read.rs`).
- **2026-10-09:** GraphSession candidate/replay validation happens before installation, but closure return values are not errors (`graph_session.rs:1157`). Full-batch input preflight is required for the live facade.
- **2026-10-09:** Current publication checkpoints then writes addressable storage separately; current host staging writes retired/current Pending separately. Combined atomicity must be scoped to one current session, and retired batches retain separate receipts (`graph_session/residency.rs`, `mere_host.rs:204`).
- **2026-10-09:** Djinn already supplies a single cached Directory-backed host per session and derives via from admission. Browser staging releases its RefCell borrow for IndexedDB writes; both are reuse points (`resident_mere.rs:468`, `web_session.rs:55`).
- **2026-10-09:** Existing roots allow only one focus, snapshots/client mounts lack coverage, and Canvas's current helper clears empty scopes. These are explicit interface changes in Tasks 4 and 6, not assumed capabilities.

## Progress

- **2026-10-09:** Read approved spec, parent lane, canonical policy/index and relevant source interfaces. Concrete plan written and self-reviewed, then approved by Mark. Local documentation checkpoints 59393140d and ec41bf4a2 retain that approval. Task 1 has a test-only preparation control staged; its expected-failure gate is running. No host production change has been made yet. Consumer default libraries pass 751 tests with 9 existing skips; the remaining consumer gates are separate from host qualification.
- **2026-10-09:** Mark approves the concrete plan. Root records the interface/file preflight and starts Task 1 under the preserved team method. Each task receives observed tests and an independent task review; final host qualification and publication remain pending.

- **2026-10-09:** Task 1 complete at local `a61dfb5c9bba1347965f9cfa8b0a936d4e1c1a7d`. Expected API RED followed by named GREEN (1), full native residency controls (10, including real redb) and Pandect libraries (349), with zero new skips and unchanged qualified source. Independent task review is spec compliant and quality approved. Firefox runtime controls are wired but their execution belongs Task 8. Task 2 starts with a fresh implementer and two test-only controls. All host source remains local pending whole-host qualification and publication approval.

- **2026-10-09:** Task 2 complete at local `652acbb72cecb98ec341cedea84e17b88d0e134c`. Missing-API RED, a scoped test-inference repair, then eight live-command controls, 349 Pandect library tests and ten residency tests pass with unchanged runtime inputs. Independent review approves the exact four-file candidate without blocking findings. Qualified legacy admission, Surface placement for UserGrouped, and decoded input boundaries are recorded in the task. Malformed external text and the suggested shared-Resource creation control remain assigned to product integration. Task 3 begins one retained combined checkpoint/publication batch.
