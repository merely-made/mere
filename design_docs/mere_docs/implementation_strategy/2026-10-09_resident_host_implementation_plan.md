# Initial Resident Host Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (the user-selected orchestration method) to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Root serializes compilation; independent runtime tests may overlap. One writer owns each shared module.

**Goal:** Give Graphshell/Djinn's owned complete GraphSession a configurable partial presentation, typed live edits, explicit durability outcomes, staged checkpoint publication, refresh and portable coverage.

**Architecture:** GraphSession remains the complete writer and history authority. ResidentGraph supplies immutable presentation data; typed stable IDs cross back to the writer under observation preconditions. Owned hosts prepare checkpoint and addressable writes together, release browser borrows for I/O, acknowledge confirmed storage, then install a lifecycle-checked resident candidate.

**Tech Stack:** Rust, existing graph-kernel/Pandect/Muniment, Graphshell/Djinn, Chirograph/graphshell-client, native Directory/redb and browser IndexedDB. Reuse existing event pumps and projection notices.

**Spec:** [Approved initial resident host design](../technical_architecture/2026-10-09_resident_host_design.md). Read it alongside the [parent follow-ons plan](2026-10-09_graph_semantics_followons_plan.md).

**Date:** 2026-10-09
**Status (2026-10-10):** Written spec and concrete plan approved by Mark. Tasks 1–6 are complete and locally committed at a61dfb5c9, 652acbb72, 6b2f2860b, 29ab75c56, 45239482c and bb9025752 after independent reviews. Task 3 includes combined persistence staging and a bounded navigation snapshot ordering repair. Task 5 includes head-only refresh, lifecycle/notice checks and the reviewed browser continuation repair. Task 6 includes portable coverage, exclusive Directory cache ownership and retained endpoint recovery. The isolated current-main candidate passes ten fresh qualification gates and independent integration review; Task7 test controls are in progress and Task8 remains pending. Consumer A1 is separately qualified, published and remotely resolved. Host source stays local pending whole-host qualification and publication review.

## Global Constraints

- “Full remains the compatibility setting.” Neighborhood settings belong to the graph, with selected authored projection, hops and manual or host-driven refresh.
- “ResidentGraph remains immutable.” It never supplies writer NodeKeys or a complete checkpoint.
- “Do not add a general multiwriter lock service, authority epochs or persistent request-receipt log for this milestone.” Reuse session ownership, Applied/Pending and existing host lifecycle identity.
- “A store error is not evidence that an edit never happened.” Pause edits during the first combined staged commit; retry the retained batch, never the command.
- “Keep remains its existing per-view control, separate from graph pinning.” Every pane's transient focus participates without creating a durable pin.
- “Do not collect generations in this milestone.” Turnstone authority migration, genuinely bounded writers and detailed GC/scoped-transaction design remain separate reviewed work.
- “Report process memory separately from reader-record savings; this milestone makes no bounded-process-memory claim.” Full writer/baseline/history and candidate/replay peaks remain.
- Preserve transport modularity, data projections, the shared wgpu device and existing Author, qualification, validation, replay, journal, undo and kept-part controls.
- Backend work is limited to Task7's concrete IndexedDB lifetime admission and version2 cutover correction, required to exclude independently opened and legacy v1 browser writers. It adds no general Backend trait capability, lock service, authority epoch or persistent receipt log. Consumer authority migration remains excluded. Consumer repins are limited to the Task6/current-main Knot compatibility revisions, preserving each baseline's standalone pins/defaults. Unsupported or unproven ownership/backend composition returns a typed refusal.
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

New files; unfinished targets are annotated at their exact planned occurrence:

- `crates/system/pandect/src/live_graph.rs` *(implemented in Task 2)*: portable typed command/precondition vocabulary and preflight/application helpers.
- `crates/system/pandect/tests/live_graph.rs` *(implemented in Task 2)*: complete-writer live/observation parity tests.
- `ports/graphshell/src/resident_host.rs` *(staging/settings and asynchronous refresh implemented in Tasks 3–5)*: host settings, pane roots, refresh scheduling and staged state, beside MereHost.
- `ports/graphshell/src/web_resident.rs` *(private facade draft exists; Task 7 route/control wiring remains pending)*: browser controls, foreground/idle pump and lifecycle-safe async work.
- `ports/graphshell/tests/resident_host.rs` *(native ownership/reopen controls implemented in Task 3; memory qualification in Task 8)*: native owned-host failure/reopen and memory fixture.
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

**Files:** Modify GraphSession residency helper, MereHost staging and web_session. Add staging state in the resident_host module and native host test. Preserve current Staged's retired-session handling. The 2026-10-10 diagnosis additionally permits a bounded repair to `crates/eidetic/chartulary/src/stemma/snapshot.rs` and its regression controls: emit unordered owned visits and bindings deterministically, preserving all ordered navigation data and metadata.

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

- [x] Write the compile-ready RED test in existing graph_residency tests:

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

- [x] Root runs the exact named graph_residency test; expect RED for missing staging API.
- [x] Expose forced-checkpoint preparation through GraphSession's residency module; verify qualification and reject Residency-limited graph before preparation. Make acknowledge refuse changed session/root graph/cursor/revision before calling stored. Keep existing public publish_resident_checkpoint as a compatibility path; the owned host adopts combined staging rather than pretending its two writes are atomic.
- [x] Admit resident combined writes through concrete host factories, not a caller-supplied bool or generic Backend label. Djinn's native composition is completed by Task6's private fresh-Directory cache and restricted session facade. Browser composition's one host and concrete IndexedDbBackend remain the required Task7 activation path; Task3 supplies the private admission/staging APIs, not proof of exclusive browser ownership. MemoryBackend is test-only admission; native redb admission requires its opened exclusive handle. Keep constructors granting admission private to these composition paths. Arbitrary Backend, Fjall, Zip, unsupported wrappers or an independent writer receive OwnershipUnproven/UnsupportedStorage before resident edits. No new universal backend capability interface is needed.
- [x] Pause writer edits, undo/redo, view writes, session switch and detach publication while the current combined batch is prepared/in flight or uncertain. Allow cancellation of a refresh candidate, not cancellation that loses an uncertain write. Read-only last coherent presentation remains usable. Preparation/storage failure after a successful edit reports AppliedButUnstored with its existing Applied identity, retaining the exact batch if prepared. A confirmed batch is Durable; failed reader refresh is PendingPublication and retries only refresh. Settings-only publication may also be pending independently of an edit.
- [x] Retain per-key pre-write values before the first apply, under the existing exclusive writer ownership, alongside the in-memory batch. On uncertain completion compare all intended writes and preimages: all final values match → acknowledge once; all values equal their preimages or their unchanged final values → retry the **same** ops; any partial/conflicting mix → MixedCommit, stop writing and reopen/recover. Directory poison requires dropping all owner clones and reopening redo recovery before comparison. No persistent receipt log is added. After host loss, reopen authoritative history and show current truth; never resubmit the old command.
- [x] Add fault-backend tests for preparation codec failure, failure before apply, apply-success/error response, mixed keys, stale acknowledgement, and retry twice. Assert journal cursor/change count do not grow on retry, old reader survives, Busy prevents a second edit and recovered reopen retains exact navigation/facets/Author/undo. Use Backend wrappers confined to tests around MemoryBackend; do not qualify them as production storage.
- [x] Root runs named test, complete Pandect tests, Graphshell library tests and native resident_host integration tests serially. Reviewer checks write identity, retired-session acknowledgement, backend admission, browser borrow lifetime and outcomes. Root commits locally.

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

- [x] Add this compile-ready unit test inside resident_host, importing its proposed types plus BTreeSet, ViewKey, GraphAddress and Uuid:

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

- [x] Root runs `cargo test --locked -p graphshell --features web --lib resident_host::tests::closing_one_pane_releases_only_its_transient_roots -- --exact`; expect RED for absent module/transient roots.
- [x] Persist graph settings in existing JsonSlots under `graphs/{root_graph_id}/resident/v1/host-settings`; Full, Manual and publication_batch_ms=0 are compatibility defaults. A positive minimum interval is required for HostDriven; reject zero rather than busy-polling. Validate a selected authored projection before Neighborhood is enabled. Keep P5's existing policy key coherent with host settings by staging both together, without two competing settings authorities.
- [x] Add transient focus union to P5 selection, with zero-hop shown dependencies; it retains individual focus/pins without treating them as projection-walk roots. Keep authored rosters intact in HostRoots even when catalog members are absent or not resident. Selection skips absent requested roster members, reports their absence as host Projection context and never changes the authored roster. Known catalog members outside residency remain NotLoaded. Do not turn an absent item into an empty successful demand.
- [x] Add a separate private `exact_scope: Option<BTreeSet<Uuid>>` beside the current runtime `scope: Option<Vec<NodeKey>>` in Canvas. scope_to_exact_members stores authored IDs and resolves runtime keys, preserving Some(empty) even when nothing is resident. Re-resolve it in lifecycle::set_graph; scope_members returns the authored IDs when exact_scope is present. Explicit scope_to_members/isolate_selection clear exact_scope, and clear_scope clears both. This preserves empty/missing IDs across source changes; clear_scope remains explicit Show all. Do not call existing scope_to_members for this path. Add scope tests: empty roster yields zero visible nodes with a nonempty graph; unloaded/missing roster survives a graph swap and becomes visible after demand; clear_scope alone restores all. Keep's view state is unchanged.
- [x] Make presentation_graph choose installed resident data when residency is active, otherwise the Full writer. Leave MereHost::graph() Full. Session close releases the closing pane's roots; shared projections/focus/pins remain. Give each admitted endpoint a distinct existing ViewKey rather than the untrusted application payload's claimed identity.
- [x] Root runs Graphshell resident_host tests, `cargo test --locked -p pictograph --features canvas --lib scope`, and graph_residency tests. Review graph-settings persistence, exact scope, pin/Keep separation and session/pane ownership; root commits locally.

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

- [x] Add this compile-ready test inside resident_host tests:

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

- [x] Root runs the exact Graphshell test; expect RED. Add a counter-backend graph_residency test for refresh_if_changed: unchanged generation increments only head get, not catalog/record/navigation/pending reads; roots/policy changes still reconcile even with identical head.
- [x] Split candidate load/install while preserving checksum/placement/incident-copy validation. New head with unchanged available_digest is SourceOnly unless context/pending changes make it ObservationOnly; changing available truth is TruthChanged. Host context changes use set_host_coverage independently. Failures discard candidates, keep current generation/view and leave pending check. Policy writes occur only after a valid candidate and use staged settings, not speculative persistence.
- [x] Drive one in-flight request from manual refresh, confirmed local publication, foreground/attachment and elapsed minimum_check_ms. Store publication_batch_ms separately; publish at existing idle/store boundaries, explicit save and detach flush. Notices coalesce by a monotonic host sequence; completion consumes only the sequence it started with. Newer notices queue another check. Ignore backward/nonfinite browser clocks; use host monotonic elapsed input, not a library timer/thread.
- [x] Invalidate token on session/root/policy/lifecycle changes. Preserve latest host coverage when a coherent earlier source candidate installs; do not clear a new notice while acknowledging old work. Demand is inspection only and uses the same token discipline. A demand adds no durable pin. Suspended browser tabs perform no work until resume, then check immediately.
- [x] Add controlled-await tests for roots/policy/session switches, newer notice, context change during load, read failure, demand failure, notice bursts and foreground catch-up. Extend existing P5 unloaded-only/context cache controls so successful query evaluation count and truth revision remain stable for SourceOnly/Pending/context updates, but increase when loaded truth changes.
- [x] Root runs Graphshell resident_host tests and full graph_residency native suite. Reviewer checks single-flight/sequence lifecycle and graph/cache retention; root commits locally.

**Done:** unchanged polling reads only head; failed/stale candidates never replace the coherent view; latest observation and notice state survives.

### Task 6: Product-neutral coverage through endpoint and client

**Files:** Modify Chirograph lib, Graphshell snapshot producers, Djinn snapshot/notice logic, graphshell-client lib/persistence and existing portable client tests. Update all ProjectionSnapshot literals in this workspace explicitly; no unrelated consumer source edits.

**Consumes:** kernel CoverageLayer::{Possession,Residency,Disclosure,Synchronization,Projection} and CoverageNote; presentation_graph and host projection revision; existing snapshot/client/carrier interfaces.

**Integration amendment (2026-10-10):** The admitted endpoint needs concrete native cached-owner composition in this task. Its first source review found that a supplied cloneable Directory handle and caller contract cannot prove one writer. Replace that seam with a fresh Directory-owning cache and a restricted cached-session facade. Neither backend, raw admitted MereHost nor MereSessions may escape; ordinary hosts retain their detached-store API. Preserve session uniqueness, validators before epoch writes, legacy Full behavior and metadata lifecycle guards. A separate sealed fault fixture reuses that boundary; plain real-Directory cache and actual endpoint controls must qualify production composition. This is the minimum ownership repair, with product/browser controls still Task7.

Accepted edit/undo/view intents retain their accepted outcome when subsequent storage or reader publication fails. Expose unstored, durable and presentation-pending status using existing portable cards, pause further mutations, and retry the exact retained batch or publication through the admitted owner without replay. Both false-rejection paths have actual endpoint RED receipts. The five final forward gates and the original reviewer's scoped fix review passed on the committed Task6 source. Actual product/browser runtime remains Tasks7–8.

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

- [x] Add this compile-ready Chirograph unit test (existing lib tests already import super::*):

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

- [x] Root runs `cargo test --locked -p chirograph coverage_preserves_stratum_and_unreported_is_not_empty_note`; expect RED. Add a real ProjectionSnapshot deserialization control removing the coverage field from existing fixture JSON; assert None. Round-trip supplied empty/all-five-layer notes and same UUID in both strata.
- [x] Produce snapshots from presentation_graph and carry coverage_note, not a truth facet or fake graph edge. Context-only updates advance the host's **projection** revision/notice, not the resident truth revision. Djinn must notice changes in coverage and Pending observation even when truth is unchanged; reset per-endpoint seen state on attach/switch. Preserve existing admitted/ambient policy checks.
- [x] Store coverage in MountedScene and default it in persisted projection decoding; clear it with the existing session lifecycle. A context-only notice forces a fresh snapshot through existing resnapshot/resume handling so coverage need not be invented as a scene diff. Retain successful scene resources/cache when unchanged and update the observation/presentation status. No new carrier transport or protocol version is required for the defaultable snapshot field.
- [x] Add this compile-ready test to existing graphshell-client lib tests, where snapshot_with_offer and semantics are verified fixture helpers:

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

- [x] Root runs `cargo test --locked -p graphshell-client context_only_coverage_updates_preserve_cached_resources`; expect RED before wire/client fields and GREEN after. Add persisted old-snapshot, close/revoke and ambient-action controls.
- [x] In Djinn's existing resident_mere tests, reuse fixture(), attach() and graph() to serve two admitted endpoints sharing one host. Configure scoped residency via the owner path; feed the actual snapshot to ClientState::apply_snapshot; assert NotLoaded coverage, demand action and later coverage-only notice/client update. Keep existing session ownership/admission/refusal tests green.
- [x] Root runs `cargo test --locked -p chirograph`, `cargo test --locked -p graphshell-client`, `cargo test --locked -p djinn --lib resident_mere`, and Graphshell library tests. Review old-wire meaning and observation/truth separation; root commits locally.

**Done:** endpoint clients receive and retain the five coverage meanings, including context-only changes, without claiming missing metadata proves completeness.

### Task 7: Wire usable partial-display edits and host controls

**Files:** Modify product.rs, MereHost, web_product.rs, web_main.rs, web_session.rs, web_view.rs, web.rs and Djinn resident_mere; create planned web_resident module. Add controls in existing `ports/graphshell/web/tree.html`, `ports/graphshell/web/styles.css`, `ports/graphshell/src/view.rs` and `ports/graphshell/src/web_tree/product.rs` where their respective shell renderers produce product controls.

**Consumes:** all preceding APIs; existing ProductError/EditableRelation/LocalFileMetadata/content-reference validation; existing session_item intent routing/Author-via admission.

**Produces (proposed host API):** `MereHost::apply_observed_commands(&mut self, commands: Vec<pandect::live_graph::ObservedCommand>) -> Result<pandect::live_graph::LiveApplied, ResidentHostError>`. It uses the selected host Author/via and never trusts payload provenance.

**Produces:** named partial operations from the approved spec, settings/demand/refresh endpoint actions, visible coverage and persistence state. New intent payloads live beside the existing session_item vocabulary, using decoded typed UUID/JSON at admission. They do not contain Author, NodeKey or storage capability claims.

- [x] First establish a test-only admitted, recorded-placement host and publish its complete reader. Run the fixture control alone before adding missing-API tests; an ordinary unadmitted Memory host cannot prove observation validation. No production admission constructor is added.

```rust
async fn observed_product_owner() -> (MereHost<MemoryBackend>, Uuid) {
    let backend = MemoryBackend::new();
    let mut host = open_on(&backend).await;
    host.begin_profiled_session(Graph::new(), Some(PlacementProfile::RecordedStrataV1))
        .expect("recorded placement is required");
    let id = host
        .create_address("https://product.test/observed", "Original")
        .expect("seed complete product path");
    // Explicit test-only admission; no backend or raw host factory is exported.
    host.admit_resident();
    host.persist_owned(1).await.expect("publish initial reader");
    assert!(!host.resident_busy());
    assert!(host.resident_reader().is_some());
    assert!(!host.has_unstored());
    (host, id)
}

fn observed_pin_command(
    host: &MereHost<MemoryBackend>,
    id: Uuid,
) -> pandect::live_graph::ObservedCommand {
    use pandect::addressable_graph::GraphAddress;
    use pandect::live_graph::{LiveCheck, LiveCommand, ObservedCommand};
    let address = GraphAddress::Surface(id);
    let digest = host
        .resident_reader()
        .expect("published reader")
        .record_digest(address)
        .expect("complete Surface observation")
        .to_owned();
    ObservedCommand {
        command: LiveCommand::SetPinned { surface: id, pinned: true },
        checks: vec![LiveCheck::RecordUnchanged { address, digest }],
    }
}

#[test]
fn observed_product_fixture_accepts_complete_observation() {
    pollster::block_on(async {
        let (mut host, id) = observed_product_owner().await;
        let command = observed_pin_command(&host, id);
        let cursor = host.graph_session().journal().live_cursor();
        host.edit_resident("task7.fixture-control", vec![command])
            .expect("existing admitted edit must accept this observation");
        assert!(host.graph_session().journal().live_cursor() > cursor);
        let key = host.graph().get_node_key_by_id(id).unwrap();
        assert_eq!(host.graph().node_is_pinned(key), Some(true));
        host.persist_owned(2).await.expect("persist fixture control");
        let reader = host.resident_reader().unwrap();
        let key = reader.graph().get_node_key_by_id(id).unwrap();
        assert_eq!(reader.graph().node_is_pinned(key), Some(true));
    });
}
```

- [ ] Root runs `cargo test --locked -j1 -p graphshell --features web --lib product::tests::observed_product_fixture_accepts_complete_observation -- --exact`; require exactly one passing test. Then add the following negative and positive adapter controls, and run the `product::tests::observed_product_edit_` prefix. Require the specific missing `apply_observed_commands` API error for the initial RED; after implementation both tests must execute and pass.

```rust
#[test]
fn observed_product_edit_requires_the_observed_record() {
    pollster::block_on(async {
        use crate::resident_host::ResidentHostError;
        use pandect::addressable_graph::GraphAddress;
        use pandect::live_graph::{LiveCheck, LiveCommand, LiveEditError, ObservedCommand};
        let (mut host, id) = observed_product_owner().await;
        let before = serde_json::to_vec(&host.graph().to_snapshot_at(0)).unwrap();
        let reader_before = serde_json::to_vec(
            &host.resident_reader().unwrap().graph().to_snapshot_at(0),
        ).unwrap();
        let cursor = host.graph_session().journal().live_cursor();
        let changes = host.graph_session().changes().to_vec();
        let result = host.apply_observed_commands(vec![ObservedCommand {
            command: LiveCommand::SetPinned { surface: id, pinned: true },
            checks: vec![LiveCheck::Exists(GraphAddress::Surface(id))],
        }]);
        assert!(matches!(
            result,
            Err(ResidentHostError::Live(LiveEditError::MissingObservation(address)))
                if address == GraphAddress::Surface(id)
        ));
        assert_eq!(serde_json::to_vec(&host.graph().to_snapshot_at(0)).unwrap(), before);
        assert_eq!(host.graph_session().journal().live_cursor(), cursor);
        assert_eq!(host.graph_session().changes(), changes.as_slice());
        assert_eq!(serde_json::to_vec(
            &host.resident_reader().unwrap().graph().to_snapshot_at(0),
        ).unwrap(), reader_before);
        assert!(!host.resident_busy());
        assert!(!host.has_unstored());
    });
}

#[test]
fn observed_product_edit_accepts_the_observed_record_with_host_author() {
    pollster::block_on(async {
        let (mut host, id) = observed_product_owner().await;
        let command = observed_pin_command(&host, id);
        let reader_before = serde_json::to_vec(
            &host.resident_reader().unwrap().graph().to_snapshot_at(0),
        ).unwrap();
        let cursor = host.graph_session().journal().live_cursor();
        let changes = host.graph_session().changes().len();
        let applied: pandect::live_graph::LiveApplied = host
            .through("task7.observed".into(), |host| {
                host.apply_observed_commands(vec![command])
            })
            .expect("fully observed admitted edit must succeed");
        assert!(applied.created.is_empty());
        assert_eq!(applied.applied.first, cursor);
        assert_eq!(applied.applied.end, host.graph_session().journal().live_cursor());
        assert!(applied.applied.end > cursor);
        assert_eq!(host.graph_session().changes().len(), changes + 1);
        assert_eq!(
            host.graph_session().changes().last().unwrap().author,
            pandect::Author::person(FIXTURE_PERSONA_ADDRESS).via("task7.observed"),
        );
        let key = host.graph().get_node_key_by_id(id).unwrap();
        assert_eq!(host.graph().node_is_pinned(key), Some(true));
        assert!(host.has_unstored());
        assert_eq!(serde_json::to_vec(
            &host.resident_reader().unwrap().graph().to_snapshot_at(0),
        ).unwrap(), reader_before);
        host.persist_owned(2).await.expect("store the accepted edit");
        assert!(!host.has_unstored());
        let reader = host.resident_reader().unwrap();
        let key = reader.graph().get_node_key_by_id(id).unwrap();
        assert_eq!(reader.graph().node_is_pinned(key), Some(true));
    });
}
```

- [x] Add target-wasm `wasm-bindgen-test = "=0.3.79"` test support to Muniment and inspect only the required existing-family lock edge. Using the existing `IndexedDbBackend::open`, run an actual Firefox positive clone/write control and a separate independent-second-open control while the first handle remains live. The initial runtime RED must show the second handle sees the same sentinel and succeeds where refusal is required. Tool startup, timeout, compilation failure and zero tests do not count as that RED. Qualification uses disposable names and a fresh profile, then exact typed admission errors replace the initial generic refusal assertion.

- [ ] Write failing integration assertions in the planned resident_host tests: open owned host with Full, create three addresses, choose an authored one-member projection and Neighborhood(0), confirm presentation has one Surface plus shown Resource. Invoke observed title/tag/facet/pin/relation commands through the host, assert complete writer truth changes and the old reader stays coherent until publication/refresh. Reopen/undo must match the direct Complete product path. Root runs `cargo test --locked -j1 -p graphshell --no-default-features --features web,resident-storage --test resident_host`; expect RED for absent host integration.
- [ ] Adapt existing product operations to build ObservedCommand with Task 1 digests and the correct checks. Expose only supported partial commands; unsupported closure/global/import/delete/withdraw/history/fields/couplings operations return NeedsCompleteOperation or the explicit Full owned-writer path. Keep existing global undo/redo on that same host with their kept-part semantics. Require explicit Complete operation setting instead of silently opening another writer.
- [ ] Keep file content-reference validation and address reuse first. Preserve existing create_file_metadata change grouping: address/title, MIME and the two facets remain the existing separate changes. Hold the host operation boundary over the sequence, stage once at its existing store boundary and do not infer byte custody from resident metadata. Input validation completes before any phase; report already-applied state if a later existing phase refuses, rather than pretending the sequence rolled back.
- [ ] Parse UUID strings and facet JSON fallibly at the external boundary; retain distinct invalid-ID, invalid-JSON, missing, stale, unqualified/read-only/product-validation errors. The typed adapter uses complete lookup for address reuse and tag diffs. Preserve supplied exact relation assertions and capture handles; no selector-family substitute for exact withdrawal is added.
- [ ] Render graph controls: Full/Neighborhood, authored projection selector, hops, Manual/Host-driven, minimum check interval, publication batch interval, explicit Save/Refresh. Render coverage with Unreported/Not loaded/Missing wording and Inspect/Demand actions. Show Applied but unstored, Stored, and Stored; presentation pending distinctly; only the latter retries publication/refresh. Disable mutation controls during combined staging while the coherent reader remains inspectable.
- [ ] Switch canvas/portable scene source from host.graph() to presentation_graph() at source boundaries. Preserve stable authored scope with scope_to_exact_members; all UI targets are stable UUIDs. Return Full compatibility defaults when settings absent. Keep per-view Keep UI unchanged. Foreground/attachment and existing idle/frame events drive Task 5 scheduler; detach/save flush publication cadence and cannot discard an uncertain write.
- [ ] Add endpoint controls to the existing admitted Djinn path with the admitted application as via; ambient route remains read-only. Closing an endpoint releases only its pane roots. Invoke both endpoints through the actual route to prove one writer and union roots, with context-only notice reaching the portable client.
- [ ] Root runs Graphshell/Djinn/client tests and `cargo check --locked -p graphshell --features web`, `cargo check --locked -p djinn`. Review user-visible state/actions, Full fallback and explicit operation refusals; root commits locally.

**Done:** the shell actually exposes residency, edits, manual/host refresh and clear recovery actions. Endpoint/client behavior and local browser behavior are exercised, not inferred from library tests.


**Ownership correction (controller ruling, 2026-10-10):** Inspection proves no current IndexedDB lifetime mechanism excludes another ordinary Full writer or browser tab. Task7 must implement mandatory database-name lifetime admission for every revised writable open, then schema/open version2 before writer boot. Version2 preserves existing stores/indexes/record bytes; it blocks existing v1 connections and prevents fresh inspected old-v1 bundles from reopening. A blocked attempt refuses admission and drains its submitted request under the retained lease, aborting a later upgrade before application actions. Cancellation is a request, never immediate evidence of release or absent writes. Supervise open/upgrade/transaction callbacks through actual terminal events; keep clones and detached I/O inside a private browser owner facade. Replace raw capture/workbench backend escapes with named operations. Full remains the presentation default after admission. No user database is modified during qualification; legacy cutover controls use disposable data and fresh browser profiles. Actual browser tests must cover both construction routes, old v1 blockers/fresh v1 refusal, cancellation/close/transaction drain and exact uncertain-write recovery. This corrects the plan's backend exclusion to meet the approved spec's explicit independently-opened-writer refusal; it does not qualify browser ownership before those gates.

### Task 8: Native durable, Firefox and memory qualification

**Files:** Tests in the planned native/browser host tests and existing Djinn resident_mere tests. Root appends qualification receipts to this plan's Findings/Progress and owns index/D2/spec/parent status changes. Add target-wasm test dependencies `wasm-bindgen-test = "=0.3.79"` and `uuid` with `js` in existing `ports/graphshell/Cargo.toml`, matching Pandect's verified browser test floor; Task7 owns the separately ruled narrow IndexedDB lifetime changes. Enable `pictograph`'s existing `canvas` feature for its scope tests.

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
    let session_id = writer.id();
    let expected = serde_json::to_vec(&writer.graph().to_snapshot_at(0)).unwrap();
    // Every lease-retaining clone must drain before an independent reopen.
    drop(reader);
    drop(sessions);
    drop(writer);
    drop(backend);
    let reopened_store = muniment::IndexedDbBackend::open(&name, "records").await.unwrap();
    let reopened = pandect::GraphSession::open(reopened_store, session_id).await.unwrap();
    assert_eq!(serde_json::to_vec(&reopened.graph().to_snapshot_at(0)).unwrap(), expected);
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

- **2026-10-10:** Task 3 checkpoint control, complete Pandect tests and Graphshell library tests pass. Native integration remains unqualified: both replay-helper and live-navigation fixtures fail admission before the factory. An isolated 64-trial diagnostic confirms only unordered `owned_visits` array order differs between candidate and replay (29 combined edits and 23 separately applied second visits; zero add/first-visit mismatches). Root expands Task 3 narrowly to sort the semantically unordered visit set and binding map when creating Stemma snapshots. Replay comparison stays exact. Chartulary regression, complete Pandect/Graphshell gates, native reopen/navigation/Author/undo assertions and independent Task 3 review remain required. No host shared publication has occurred.

- **2026-10-10:** The bounded snapshot ordering repair passes 67 Chartulary tests and four native host tests, including exact Directory reopen/navigation/Author/undo and zero-mismatch replay. Fresh complete Pandect gates pass 349 library, 12 residency, eight live-command and two archive tests (one inherited ignore), plus doc tests; Graphshell passes 328 library tests (five inherited ignores). All runtime source inputs and log hashes match the frozen 12-file candidate. Independent Task 3 spec/quality review is pending; Tasks 4–8 and whole-host publication remain pending.

- **2026-10-10:** Task 3 complete at local `6b2f2860b59d2dd698b5cc8c02ac099073ce79df`. Independent spec/quality review approves all 12 frozen and gated source files without blocking findings, and committed blobs match the qualified hashes. Exact retained checkpoint/publication writes, private concrete-backend admission and native reopen controls are implemented. Task 4 starts graph settings, pane-root lifetime and exact authored scopes. Existing Djinn/browser composition activation and real poison/headed/Firefox runtime remain required in Tasks 7–8; host source is still local.

- **2026-10-10:** Task 4 is complete with coherent settings/policy batches, pane roots, one installed reader in MereHost and exact authored canvas scopes. Graphshell passes 331 tests (five inherited ignores), Pictograph passes 379 (18 inherited ignores), and residency passes 13. A test-only unsupported snapshot equality was repaired; a legacy missing-roster failure fixture now uses an absent individual focus, retaining its exact failure-preservation assertions while respecting the approved missing-roster Projection context. Djinn endpoints pass eight tests and native owner regression passes four. Independent Task4 review approves spec compliance and quality with no blocking findings. The ten source files are committed at `29ab75c56`; all committed blobs match the frozen gated candidate. Off-runtime/shutdown cleanup remains a Task5/8 lifecycle qualification boundary. Actual product recipe/roster resolution reuses saved projection definitions in Task 7.

- **2026-10-10:** Task 5 is dispatched for head-only checks, owned candidate loading, lifecycle-checked installation and notice coalescing. It must preserve current host coverage and reject foreign/stale candidates while releasing browser borrows before I/O. Host publication still awaits whole-host qualification and Mark's review.

- **2026-10-10:** Task 5 source is frozen for root gates. Graphshell passes 337 tests (five inherited ignores) and residency passes 15, including owned-load provenance, controlled lifecycle changes and head-only I/O. Query, native-owner and wasm checks plus independent review remain pending. Settings candidate validation precedes policy storage; failed/stale loads preserve dirty settings and the coherent installed reader.

- **2026-10-10:** All five Task 5 gates pass on the initial frozen candidate: Graphshell337, residency15, Mere query54, native owner4 and wasm Graphshell check. The initial native gate receipt was interrupted and replaced by a complete guarded rerun. Fresh review requires a repair to the browser settings-preflight continuation: expected stale completion currently sets a terminal store failure and may overwrite newer session status. Task5 remains unfinished until a meaningful continuation RED/GREEN and scoped re-review pass. Shared heavy-job flock and a three-GiB RAM buffer now guard future compiler/linker, packaging and headed browser jobs.

- **2026-10-10:** Task 5 fix round 1 reproduces the actual browser continuation defect: four stale-load controls fail, while a current read-error control passes. The corrected candidate adds store-attempt ownership and seven continuation controls, including retained uncertain-byte retry. Only two Graphshell files change from the initially qualified Task 5 source; relevant native/browser gates and scoped re-review remain pending. Shared lock and peak-plus-buffer checks defer compilation while another heavy job or insufficient headroom is present.

- **2026-10-10:** Task 5 complete locally at `45239482ca0e25ce371bd37f7e60e7e89267ae8a`. The original Important browser continuation finding is Addressed and scoped spec/quality review approves the fix. Graphshell passes 344 tests (five inherited skips), native owner regression passes four and the wasm browser compile passes with exact input/log guards. The unchanged Pandect residency15 and Mere query54 gates remain bound by an explicit two-Graphshell-file amendment. All nine committed source blobs match the final freeze. Seven continuation controls cover stale retry, newer session/attempt preservation, current read errors and exact uncertain-write bytes. Native test links use a recorded qualification-only one-thread/no-mmap-output policy; sampled available RAM remains above the shared three-GiB reserve. Task 6 starts endpoint/client coverage. Actual product activation, browser/Djinn runtime, async detach/shutdown and whole-process memory remain Tasks 7–8.

- **2026-10-10:** Task6 implementation is released and frozen across21 source paths, with Cargo.lock unchanged. It adds portable coverage, context-only client snapshots, admitted endpoint demand/refresh and the prerequisite native cached-owner persistence seam. Compilation and independent task review remain pending; no Task6 runtime qualification or publication claim is made. Root's own launcher probe verifies kernel MemoryMax/MemoryHigh, swap0 and memory.oom.group1 before payload release. Future heavy gates hold the shared machine lock, require at least8GiB available and expected peak plus4GiB headroom, use serial Cargo/test harnesses, and abort only their own verified job below4GiB. Gates yield between stages for the other queued chats.

- **2026-10-10:** Task6 Chirograph47 and client84 tests pass (one inherited client doctest ignore). The Djinn gate exposes a compile failure in three constructors in the pinned external Knot provider: old wire defaults do not prevent a new Rust struct field from breaking source literals. Root rules on the minimal compatibility prerequisite, keeping Knot defaults and its older Mere pins while adding a default-off coverage feature; Mere will enable it at one exact compatibility revision. This narrow dependency exception is recorded above. The original failure/freeze remains preserved; gates and review must qualify the combined amended candidate before completion or publication.

- **2026-10-10:** Task6 first wire/client gates pass (47 and84 tests, one inherited driver doctest ignore). Djinn stops at three missing coverage fields in pinned Knot; a default-off compatibility feature on the exact existing Knot base is frozen at local `3da6adefba67a4cdd388aaf0de5cbec37a35a8ac`. Its original-default check and combined forward gates remain pending. Normal Cargo resolution also reselected21 unrelated broad-range dependency edges; root rejected that result, preserved its generated lock/receipt, and prepared a source-only candidate replacing six verified Knot IDs. Locked forward gates must accept it unchanged. Older Knot Mere pins, all other locked dependency edges and the original failed Djinn evidence are preserved. No publication claim is made.

- **2026-10-10:** Knot original-default locked check passes on local compatibility revision `3da6adef`; its inherited unused-parameter warning is preserved. Task6 independent source review identifies two required fixes: existing cloneable Directory handles do not prove a unique admitted writer, and an endpoint can reject an already-applied edit after storage/publication failure while exposing no retained-write retry. Four test-only ownership and endpoint fault controls are frozen before production repair. Task6 remains unfinished; combined locked forward gates, scoped re-review and Tasks7–8 still remain. Host and compatibility changes remain local pending qualified publication approval.

- **2026-10-10:** Task6 ownership controls reproduce both required admission failures against unchanged production: 29 resident-host controls pass and two fail, specifically duplicate admission from an existing Directory handle and admission beside an ordinary live writer. The kernel-capped run completes with unchanged source/dependency inputs. Endpoint write/publication fault controls are compiling; production remains held until their runtime RED. The fix must also keep the admitted raw host and its public backend-cloning `store()` method behind the concrete owner boundary.

- **2026-10-10:** Task6 endpoint fault controls now reproduce both false-rejection cases through admitted Djinn routes: real apply followed by lost completion, and durable apply followed by resident-reader record failure. Both return Rejected despite the applied edit. The frozen, locked forward build accepts the source-only Knot lock amendment unchanged, but these intentional REDs are not qualification GREEN. The original implementer resumes the concrete private cache/facade and truthful persistence/recovery repair. Undo/view loop cases, retained-batch recovery, fresh covering gates and scoped re-review remain required before Task6 completion.


- **2026-10-10:** Task6 Fix1 passes five fresh covering gates: Djinn14, Graphshell352 (five inherited ignores), native owner4, native all-target check and wasm browser compile. Wire47/client84 evidence remains explicitly retained through the unchanged package-cone ruling. Scoped review accepts the original admission and truthful recovery fixes, but finds a further shared-host interleaving: a peer commit during a held reader refresh can leave publication permanently busy when the completion is refused before its slot is retired. Fix round2 reproduces that failure for both successful and failed actual held loads through two endpoints sharing a plain Directory cache. Both controls reach the matching-slot retirement assertion; Three foreign owner/session/attempt controls pass on that frozen test stage. The narrow production repair now retires only a matching completion during recovery, preserving the newer notice and publication work; five fresh covering gates and scoped re-review remain pending. Task6 remains unfinished; actual browser runtime and whole-process qualification remain Tasks7–8.


- **2026-10-10:** Task6 complete locally at `bb9025752a6d49176cdd071116f30aabd9e04df7`. Both held-refresh interleavings recover through real Directory endpoints; wrong owner/session/attempt controls preserve the outstanding work. Five fresh covering gates pass: Djinn16, Graphshell355 (five inherited ignores), native owner4, native all-target check and wasm compile. Chirograph47/client84 remain bound through the unchanged package-cone evidence, including one inherited client doctest ignore. Scoped spec/quality review addresses all three required findings and approves the exact26 committed source blobs. One nonblocking wasm dead-code warning is explicitly retained until the browser-composition scope is settled. The source-only Knot lock amendment is accepted unchanged by these locked gates. Task7 controls, actual browser/runtime/memory qualification, current-main integration and shared publication are still pending.

## Current upstream integration boundary (2026-10-10)

A fresh remote comparison after local Task6 source commit `bb9025752` found `origin/main` at `4f70d5362` with 161 commits absent here and 14 local commits absent from main. This count precedes the Task6 status documentation commit. Main also changes the root Cargo manifest/lock, both Graphshell/Djinn manifests and `identity_endpoint.rs`; its newer Knot pins must be reconciled with the resident snapshot compatibility amendment. Task6 keeps its frozen inputs for qualification. Current main also adds DR-C custody/startup receipt handling; preserve that boundary in the combined candidate. Reconcile the current main candidate, review the combined source and run the affected checks before shared publication; the current branch gates do not qualify that combined candidate.

The locked p2panda input is fork commit `1bec457e` at tag `mere-p2panda-net-0.7.5`; the observed upstream main has 26 commits absent from that fork, which has 20 unique commits. Iroh 1.3.0 and iroh-blobs 0.103.1 are current releases, including the blob push fix; their upstream main branches also contain 19 and 7 newer unreleased commits respectively. Dependency reconciliation is separate from the resident-host implementation and does not silently change its frozen test inputs. The p2panda counts describe ancestry, not proof that every upstream change is absent semantically; preserve the fork's exact ALPN, rejoin, topic-update and shutdown behavior during a separate upstream review.

Before Task7, prepare an isolated local candidate combining the qualified Task6 source with freshly observed Mere main. Current main changes the product/browser controls and identity boundary. Keep its Dramatis extraction and newer Knot family input. Knot `fef132ef9922ede907d80950762a7f313508547e` still needs the three default-off snapshot coverage constructor fields; carry that narrow compatibility amendment onto the newer Knot input and qualify its original defaults and the embedded forward build again. Original commits and receipts remain historical evidence. The combined candidate needs its own affected gates and integration review before controls continue. Shared push and main merge still require approval of the concrete qualified candidate.

- **2026-10-10:** Current-main source reconciliation combines host metadata parent `25cbff5be` with freshly observed main `4f70d5362` in an isolated local candidate. Both-parent inspection preserves the native custody separation, Forme hold release and exact authored scopes alongside the qualified resident host. Three manifest/lock conflicts are resolved at source level; actual Cargo lock acceptance and integration review remain pending. Knot compatibility is carried onto main's `fef132ef` family as local immutable candidate `fb415b8014c017ccd0a6dd4dddf2c003b101680c`, with original-default byte restoration proven. Fresh default, wire/client, Pandect, Pictograph, Graphshell, Djinn, owner, native all-target and wasm gates must qualify these combined inputs before Task7 production begins. Browser ownership preflight is read-only; no owned-browser qualification or shared publication is claimed.

- **2026-10-10:** Current-main integration prerequisite qualified and independently reviewed. The frozen combined source joins host parent `25cbff5be` and freshly verified upstream main `4f70d5362`; all ten gates pass: locked dependency resolution, Knot original-default check, wire47/client84, Pandect321, Pictograph395 with canvas, Graphshell402, Djinn122, native owner4, native all-target check and Wasm library check. Existing ignored probes and diagnostics remain explicit. Knot `fb415b8014c017ccd0a6dd4dddf2c003b101680c` retains its unchanged standalone pins/defaults; embedded Mere enables the compatibility construction feature. The source-only lock replaces six Knot IDs and preserves every other main dependency edge. Integration review finds no blocking issue; Root corrects the stale index summary and retains the unused Wasm helper warning for Task7. The Wasm library check does not compile the separate browser presenter, and library/fixed-authority tests do not prove the real keeper route. Task7 product/ownership controls, Task8 running native/client and actual Firefox/memory qualification, final whole-host review and shared publication remain pending. The local merge commit must retain every qualified runtime blob; compatibility remains unpublished until concrete publication approval.

- **2026-10-10:** Current-main integration is locally committed at `af44edc80a2ea371664f6f265ab3b5f919aa8181`, with every qualified runtime blob verified. Task7 begins with test-only controls. Fresh locked metadata accepts only the Muniment target-wasm test edge at the existing locked wasm-bindgen-test family. The admitted, recorded-placement Memory fixture publishes a reader, accepts a complete Surface observation through the existing edit path, persists it and confirms the refreshed reader; exactly one control passes with source/log/binary/section/kernel evidence. This is a test-only admission seam, not a production browser ownership factory. The new adapter negative/positive tests and existing-API Firefox clone/independent-open tests are staged and frozen; their named initial failures are still queued. Production backend/facade/product code remains unchanged until those source failures are demonstrated. Actual browser lifetime, both routes, legacy cutover, running keeper/client and process-memory qualification remain pending.

- **2026-10-10:** Task7 initial failures are demonstrated on the unchanged production candidate: two specific missing `apply_observed_commands` calls after the passing admitted observation fixture, and actual Firefox accepting an independent second IndexedDB handle while the first writer remains live. The independent handle reads the same sentinel; the first handle still reads afterward. The separate clone/write/read browser control passes. Exactly one test executes in each browser invocation, with disposable database names and a fresh local Firefox profile. Frozen source/dependency inputs, real driver/browser cgroup identities, logs, resource bounds, compiler flags and identical Wasm artifact are retained. These are initial headless storage controls, not browser facade, legacy cutover, two-route or headed-shell qualification. The native product integration test stage is released next; production remains held for its initial failure.

- **2026-10-10:** Task7 native integration test is staged with real recorded Directory admission, an authored one-member Neighborhood(0) display and an independent Complete product baseline. The prescribed locked gate has exactly one E0599 for the proposed named owned-host product adapter and no other compile error; its runtime assertions still await implementation. Initial adapter and browser ownership failures are now demonstrated, so the original Task7 implementer is released for the full product/UI/endpoint and corrected browser lifetime work. Root adds only the Muniment target-wasm optional wasm-bindgen-futures dependency and matching edge to the existing 0.4.79 family in the root and standalone presenter lockfiles; full normalized comparison preserves every other source/version/edge. These lock candidates still need actual Cargo acceptance. Covering native/browser/presenter gates, task review, Task8 runtime/memory qualification and final review remain pending. Shared publication is still pending concrete approval.

- **2026-10-10:** Task7 intermediate core gates accept both root and actual standalone-presenter locked metadata. The new non-Clone/no-I/O recorded-edit regression, Pandect library297/live8, observed-product5, actual Firefox lifetime5 and root Wasm library check pass with frozen source/tool/resource receipts. The browser controls cover independent-writer refusal, clone/close/reopen, missing-store refusal, blocked legacy cutover with data preservation and dropped queued-open drain. These are headless storage controls; remaining transaction/owner-loss/uncertain-write cases, both construction routes and actual presenter/headed shell remain unqualified. Native integration executes four passing tests and one failure: the fixture first needed pane registration before staged settings; after that correction it reaches edits and finds its zero-hop display lacks the separate tag-concept Resource. Explicit focus hydration controls are staged next. A separate negative/positive product test will check whether replacement of unloaded tags can silently use complete-writer observations. Full Task7 UI/endpoint work, native reopen/undo acceptance, task review and Task8 remain pending. No dependency upgrade or shared publication occurred.

- **2026-10-10:** The native integration now passes all five tests, including the Complete comparison, reopened truth and undo. Its zero-hop tag concept is explicitly NotLoaded before focus hydration; the fixture does not broaden zero-hop membership automatically. A separate adapter control demonstrated silent replacement of unloaded tags. The scoped repair takes external binding/tag observations from the published reader, while relation assertions remain usable without unrelated tag hydration. All three replacement/relation controls pass; the Graphshell library passes402 with five inherited ignores, native ownership/integration passes5, and the root Wasm library check passes on the frozen repair. Earlier Firefox5 evidence retains its original source map and unchanged Muniment test/dependency cone; it does not qualify Graphshell routes or the presenter. Full Task7 controls, route wiring, endpoint actions, remaining lifetime cases and task review remain pending, followed by Task8 runtime and memory qualification. No dependency upgrade or shared publication occurred.

- **2026-10-10:** The full Task7 source is staged locally and frozen for qualification. Root reproduced an actual Firefox committed-upgrade cancellation failure against the preserved original controller, then restored the proposed repair. All14 covering gates pass on the same frozen source: root and presenter metadata, observed-product8, Graphshell404 (five inherited ignores), Pandect297/live8, native ownership/integration5, Djinn123, full client84 (one inherited doctest ignore), both native checks, all11 current Firefox lifetime controls, root Wasm and the separate standalone default-presenter check. Two qualification corrections changed only test expectations: the metadata editor's collection type and the approved six-intent session vocabulary; production bytes were preserved across both. The actual standalone Wasm build also passes under the same source/dependency guard, with debug symbols disabled and assertions/overflow checks retained. Its 468 recorded target compiler invocations preserve that profile. Packaging first hit its verified own cgroup hard cap; this is an infrastructure failure, with runtime and dependency source unchanged. A disposable derivative removes only the 80 MB non-executable name custom section, preserving every other complete section byte-for-byte and the original compiler artifact, for a separately recorded packaging retry, which passes with all served assets and the actual IndexedDB controller bound to the frozen source. Disposable headed routes in both two-tab open orders, remaining UI/recovery controls and independent Task7 review are still required. Task8 remains held. Source, plan and compatibility commits remain local pending concrete publication approval.

- **2026-10-11:** Fresh upstream observation finds current Mere main `4f70d5362` and graph-semantics `443201a7` both contained in the local qualified integration. The locked p2panda Merely fork `1bec457e` matches its current fork head/release but diverges from upstream by 20 fork-only and 26 upstream-only commits; none of those 26 changes p2panda-net. This observation does not qualify a dependency upgrade. Task7 headed/UI/recovery qualification and review, Task8 and concrete publication approval remain pending.

- **2026-10-11:** Actual packaging passes using a separately proven name-section-only derivative of the original standalone Wasm. Headed Firefox initially waited for the native persistence prompt, then reported no WebGPU adapter after a native default-denial control. With WebGPU explicitly enabled in disposable profiles, the reference graph renders and a second writer is refused without changing stored bytes. The first real Address edit exposes an existing shortcut defect: canvas zoom consumes the typed hyphen and the owner persists a different URL. Exact native key actions, durable records, browser/process identities and screenshot are retained. The original Task7 implementer owns the narrow form-keyboard correction; final-source covering and headed/UI/recovery acceptance, Task7 review and Task8 remain pending. This configured Firefox run does not qualify shipping graphics defaults or physical adapter selection.

- **2026-10-11:** The narrow keyboard correction changes only web_events.rs across3277runtimepaths; every other path is unchanged. Actual standalone build and packaging pass. The original native-keyboard URL control now stores/publishes the exact hyphenated address. Close releases the owner lease, and explicit reload admits the formerly refused retained page. The broader suite stops on a helper selector error: the actual retained product is present and rendered, but its mirror lacks the assumed Saved graph aria-label. Corrected helpers are being bound to actual DOM/semantic evidence while all14 covering gates refresh on the corrected source. Both full open orders, legacy cutover, remaining UI/recovery/lifecycle and independent Task7 review remain required; Task8 is held.

- **2026-10-11:** A fresh fetch advances Mere main to `1081987cd`, four commits beyond the previously qualified `4f70d5362`: scoped Forme drafts/persistent layout undo, inspector/qualification plans, draft-delivery documentation and read-only access/Resource inspection. Full Task7 WIP is preserved in a private snapshot before integration. The combined preview has two text conflicts, module declarations and the retained Forme adapter; all other overlaps merge automatically. The local candidate keeps upstream FormeSession and inspection, routes Forme persistence through the admitted WebWorkbench facade, and privately validates saved current/undo/redo membership against complete writer member IDs. Its temporary canvas region filters unloaded cells without rewriting canonical layouts or expanding the published roster. Actual native membership RED/GREEN, integration covering checks and independent review are underway. Earlier fourteen keyboard-fix1 gates are retained on their exact prior freeze; they do not qualify this new source. Actual headed partial/Forme reopen, both routes and remaining Task7 UI/lifecycle controls are still pending, followed by Task8 and concrete publication approval.

- **2026-10-11:** The first newer-main integration passes actual membership RED/GREEN, Graphshell420 (five inherited ignores), native owner5, Forme117 and both Wasm checks on its full frozen working candidate. Independent spec/quality review approves after the original worker repairs temporary Forme region reinstall on reader replacement. Main then advances to `c397f33bf`, adding canvas backdrop/static-obstacle support and documentation. This merge pass pins that checkpoint; its sole additional conflict is canvas module ordering. The combined full WIP and separate committed-parent baseline retain all 39 existing distinctions. Seven affected integration checks and a separate review are underway. Backdrop binding across graph switches remains an upstream activation boundary: the setter has no current production caller and retained geometry requires an explicit reset/rebind contract before that use. Task7 headed/Forme/UI/lifecycle qualification, Task8 and shared publication remain pending.

- **2026-10-11:** Local merge `b7b4f591851a1be0b23a4e8c4a3b1b0bd895c808` includes pinned Mere main `c397f33bfa2de1853cad25a948d7c9046ca01348`. Module/Forme conflicts are resolved and independently reviewed. Seven guarded integration checks pass on the exact 3,287-input combined working candidate: Graphshell, native ownership, Forme, Pictograph with canvas enabled, Seiche, root Wasm and actual standalone default-presenter check. The committed merge contains only the committed-parent baseline; unfinished Task7 and current plan amendments remain uncommitted, with every runtime working input preserved exactly. These passes qualify that combined working candidate, not a clean checkout of the baseline commit. Actual headed partial/Forme reopen, remaining Task7 UI/lifecycle controls, Task8 and shared publication remain pending. The inherited inactive backdrop reset/rebind boundary is recorded above.

- **2026-10-11:** Fresh actual presenter build and packaging pass on that same merged working source. The headed reference-first ownership control passes its exact native URL write, unchanged refused-contender storage, ordinary owner close and explicit retained-page reopen. Retained-first reaches the ready owner but stops when sixteen native wheel actions fail to expose Add an object. A matched native-input diagnostic establishes a source defect: the first wheel event aimed at the panel scrolls the retained document unless pointer movement first updates the host's cached cursor. The narrow fix changes only the web-host wheel handler to use actual wheel coordinates with the existing canvas/UI-zoom mapping. Its fresh build/package and unchanged browser routing assertion pass for both wheel-only and pointer-movement inputs; both controls also save and publish exact hyphenated URLs. Screenshots, geometry, storage, native actions, the failing and passing routing assertions and kernel receipts are retained. Static review also corrects the unexecuted Forme helper's workbench lookup to use the owning session UUID rather than its distinct graph UUID; the original helper is preserved. Broader corrected-source owner/cutover/UI/lifecycle/Forme checks, Task7 acceptance/review, Task8 and shared publication remain pending.

Actual headed testing after the wheel repair passes all four owner/upgrade cases and the native wheel unit. The broader reference UI check now exposes intermittent default reopen into an empty session while the fixture graph remains durable. The initial recorded fixture creates an empty retired session with tied timestamps; the live-session UUID tiebreak can choose it. Root and original implementer are capturing bounded observations and a regression before repair. Reference pointer-navigation overlap remains separately recorded; keyboard variant and broader fault/lifecycle/Forme gates have not yet passed. Task7 and Task8 remain unfinished.

- **2026-10-11:** The ordinary-reopen defect is reproduced on both actual browser routes: a recorded fixture retained an empty construction placeholder, creating two live sessions with tied timestamps; the UUID tiebreak could reopen the placeholder while the intended 11-node graph remained durable. A deterministic native regression fails at two live manifests versus one. The narrow constructor repair creates the initial recorded session directly, preserving genuine session switching, retired flushes, authority and the unchanged regression. That regression passes, followed by Graphshell421 tests (five inherited ignores), native ownership5, root Wasm and a fresh actual presenter build/package. The unchanged headed diagnostic passes fresh startup and two ordinary reopens on both routes, with11 nodes, one live manifest and no ready/Stored count mismatch. Independent narrow review finds no required issues. Previously affected test profiles remain preserved and unmigrated. Broader owner/UI/fault/lifecycle/Forme checks, full Task7 acceptance/review, Task8 and shared publication remain pending.

- **2026-10-11:** Running the owner/upgrade cases individually after a memory-reserve stop yields four complete passes on the repaired source. The reference UI then reaches genuine keyboard commands, authored projection setup and saved metadata. Its initial title assertion targets the wrong stratum: the canonical selected Surface has the exact revised title and facet, while its Resource carries address and relationship data. Immutable helper revisions are correcting all corresponding title oracles without changing source or Resource checks. A separate strict public/durable/native oracle confirms a UI caption defect: storage and the published persistence label say Stored, but the footer retains “Applied but unstored; saving title, tags and facet.” That text remains to be repaired and checked. All original helpers, failures, screenshots, profiles and receipts remain intact.

### Continued actual UI qualification, 2026-10-11

The native policy-select diagnostic identifies an input-driver ordering issue: collapsed focus with separate trusted keys produces the expected selection and genuine native input/change events. The corrected reference workflow passes metadata/facet, pin/undo, empty and one-member Neighborhood(0), unloaded tag refusal, Cites without hydration, same-head Resource Demand without Surface expansion, hydrated edit/undo, Full restoration, host cadence and ordinary saved reopen. Those passes do not qualify every persistence status or lifecycle boundary.

Three UI defects are independently reproduced: the reference metadata caption still says applied-but-unstored after commit; successful publication recovery leaves an obsolete reference storage error; the retained route loses precise Stored; presentation pending status and Save while an owned publication read is held. Independent review catches and resolves two recovery-path gaps before application. Fix2 is applied at exactly six source paths; native Graphshell423 (five inherited ignores), ownership5, root Wasm and the actual presenter build pass. Fresh packaging passes after a separately preserved tool-cap failure. Matched headed controls pass for the stored metadata caption, reference automatic and Demand-first error retirement, and retained pending Save followed by native read abort and publication-only recovery. The full reference workflow also passes all15 checkpoints. Runtime claims are limited to these actual controls, rather than inferred from native tests.

The retained full UI reaches partial-scope/Demand positives, then its Save-scrolling helper oscillates across the clip edge. The immutable direction correction preserves all native click/hit/visibility and storage assertions; source is unchanged for that harness correction. An earlier unnecessary retained node re-click remains unclassified because the trace proves only an outer canvas hit. Reference pointer navigation has a separate actual obstruction: its HTML panel covers the old navigation rail. The unchanged native navigation discriminator reproduces Firefox click interception at (118,144) by the Address label. The reviewed two-file repair moves all eight existing commands into visible panel controls without changing dispatch. Actual presenter build and fresh packaging pass; the first repaired-site control reaches visible unobstructed button geometry, then stops before input on a test-helper event-log name collision. After the collision-only helper correction, all eight commands pass actual native pointer and keyboard controls at 1100, 740, 700 and 500px, with exact selection/detail correspondence, signed camera changes, real field typing and unchanged durable bytes. The requested 460px case stops before application controls because native Firefox window resizing clamps the observed inner viewport at500px; below480 behavior remains unqualified. Keyboard qualification does not replace pointer qualification.

A new test-only candidate exercises Task7 resident settings, observed edits, peer roots, context notices and retained resources through actual broker socket bytes and the production Directory facade. The first compile attempt exposes a test-only return-type error, preserved and corrected without changing shipping source. On the next frozen test candidate, 123 Djinn tests pass and the new socket control fails its post-close node-count assertion: two visible records versus one expected. The root-union assertion passes first. Required closure versus leaked admission is under diagnosis; the later endpoint sequence gates have not run. Existing library route evidence does not replace this transport obligation. Full Task7 acceptance/review, remaining lifecycle/Forme controls, Task8 and shared publication remain pending. All original failures, helpers, captures, profiles, source candidates and receipts remain intact.

The retained full workflow also exposes two harness timing assumptions after metadata: an immediate Facet lookup while a separate explicit Save is still doing readonly preparation, and a detached native element before that Save can be clicked. The first failure proves absent controls at that sample, not eventual return; the second run stops before the post-Save discriminator. Bounded field availability observation and exact-selector reacquisition before any native input preserve all field/value, geometry, fault and durable assertions. Neither qualifies eventual post-Save availability until the workflow actually reaches and completes that check.

The socket control is strengthened by separating retained Demand admission from strict pane-close and empty-scope checks. The reordered test passes close `{b}`/1 and context-only notices/resource retention, then observes `{b}` where the accepted empty-roster/focusNone settings intent requires an empty reader. Static tracing identifies the cached owner’s already-stored return before queued root refresh. A separate test-only discriminator checks decoded focusNone, empty live union roots, unpinned writer, unchanged graph/history/storage and the old reader before the original empty0 assertion. The actual discriminator reaches the original empty0 assertion and fails with b still Loaded. Decoded intent and live roots are empty, writer/history/generation/storage unchanged, and no unstored work or recovery is pending. This confirms the cached owner clean fast path misses the queued root-only refresh. A narrow candidate now joins the existing common publication/recovery path at the same generation, with separate real Directory regressions running before production application.

The navigation layout repair now passes all eight commands with native pointer and keyboard inputs at five actual widths: 1100, 740, 700, 500 and 460 pixels. Firefox normal-window minimum clamps the attempted 460 run to500; a separately tracked native popup reaches actual460 and passes the unchanged19 assertions. This is explicit-enabled WebGPU development qualification, with the popup limit recorded. It does not qualify shipping graphics or whole Task7.

The retained workflow actually observes Facet ID return after about3.75 seconds of native readonly work, then passes metadata and pin/undo. Bounded select availability also observes Display return and native settings input; the empty Neighborhood0 checkpoint passes. The next one-member Save attempt stops between readonly identity and first geometry, before any native wheel or click for that identity. Historical geometry reports zero or duplicate together, so multiplicity remains unresolved. A candidate distinguishes zero from duplicate and bounds only pre-first-input reacquisition; duplicates and all post-input failures remain final. Broader retained, lifecycle, Forme and transport controls remain unfinished.

The cached root-only repair is independently reviewed and applied as one native owner file, with both regression tests unchanged. Both meaningful REDs now pass: empty roots retire the old Surface; a focus-only record-read fault retains the old reader and pending publication, then read-only retry publishes the new focus at the same generation. Writer/history/cursor/revision and durable bytes stay identical, with zero apply batches. The unchanged real socket sequence and covering native gates remain pending. This native file is excluded from wasm; the exact existing browser package is retained under an explicit two-file browser-input amendment.

The real socket now passes the original empty-roster0 assertion. Its next failure is a fixture expectation: saving the authored projection creates a fourth Surface with its address/title/definition facet, but the test discarded the returned UUID and expected Full3. A reviewed test-only correction records that identity and requires Full to equal the exact complete writer set of four; empty0, close1 and later DemandABC3 remain strict. Runtime UUID provenance and whole socket GREEN are being checked.

The retained UI run exercises explicit zero-match pre-input reacquisition for Save and successfully publishes the one-member scope. It then fails native node selection: one visible node, ready/Stored state, pointer at its visible centre, but picked remains none. Internal hit detection, pointer routing and source versus helper cause are under diagnosis. The six earlier checkpoints pass; whole retained workflow remains incomplete.


The corrected socket fixture now passes its entire sequence through real broker bytes and the production Directory owner. Runtime provenance verifies the exact fourth saved projection Surface, including address, title and definition facet. Strict empty0, close1, context-only resource retention, Pending publication, read-only recovery, deferred DemandABC3 and live endpoint shutdown all pass. Covering native Graphshell287 tests (five inherited ignores), Djinn124, owner5 and the client context-only control pass on the current frozen source. The two-process and Keeper controls are still running or queued; installed Djinn qualification remains Task8.

The missed retained node click has a stronger readiness explanation: public captures immediately before and after the single native click still show the operation-busy paragraph. The pointer guard rejects selection during that state, but the old trace lacks event-time proof. A reviewed helper now waits for public interaction readiness, then records one trusted primary mouse sequence at the stable visible node centre and preserves the original selection/detail assertions. Its runtime result remains pending; neither a source collider defect nor a successful repair is claimed from static diagnosis.

- **2026-10-11:** The current source passes all six covering native gates: Graphshell287 (five inherited ignores), Djinn124, native owner5, client context1, real two-process edit exchange1 and Keeper2. Task7 remains open. Independent integration review identifies retained intake pane-root lifetime and local coverage formatting defects. Eight new formatter regressions compile and fail at the expected runtime assertions, exposing lost layer/reason/count, supplied-empty semantics and omitted composed Pending coverage; production repair is pending. A strict retained Demand byte mismatch was traced to a preceding Save completing before every Inspect input but after the baseline audit, so it does not establish a Demand write. The corrected UI check retains full-byte equality and now waits for public owned-operation completion. Forme scroll routing and native post-intake empty scope remain separate pending UI controls. All failure evidence and profiles are preserved.

- **2026-10-11:** The reviewed local coverage formatter repair passes the same eight formerly failing tests, and the full Graphshell library gate passes295 tests with five inherited ignores. It now preserves composed Pending coverage, all five layers, exact reasons, typed Surface/Resource scopes, known or unknown counts, and the supplied-empty note meaning. A fresh actual Wasm presenter build and preserved package pass; the supplied-empty wording is visible in the headed browser capture. Native intake and genuine graph-background selection clearing now pass; a RAM-floor cancellation still prevents the post-intake exact-empty assertion, so intake-root runtime qualification remains open.

- **2026-10-11:** The current retained UI run passes11 functional checkpoints, including strict same-head Demand byte preservation and hydrated edit/undo. It stops before a follow-up Save activation when a native scroll overlaps a readonly host check and the old painted control ID disappears. The subsequent focused held-read control confirms a source defect: a uniquely painted and focused Hops7 field disappears during an unchanged-head readonly check, one trusted native8 is lost, and the completed view restores7 without its focus. The same-owner pending native head, explicit nonexpired fixture release, complete native terminals, exact durable/decoded storage preservation, zero observed writes and clean cleanup all pass. The reviewed repair keeps the admitted App and stable controls present during clean automatic reads, separates typed read completion from owned saves, and refuses graph submits before effects while a check remains pending. The corrected actual Wasm build and fresh preserved package pass. The unchanged original headed held-read test now passes: Hops remains painted and focused, the single trusted8 becomes78, explicit nonexpired release completes both native transactions, and exact durable/decoded bytes remain unchanged. Failed-store hidden recovery, submit-family controls and the remaining UI/lifecycle gates still require qualification. The Forme run reaches native controls but exposes a fixture error: arranging an already horizontal pair creates no undo entry. A reviewed genuine Row-to-Column fixture retains strict current/undo/redo membership checks and adds intermediate stored geometry audits. Two separately reviewed renderer-free Firefox IndexedDB tests call the real intake facade and now reproduce the hidden root: both finish with one loaded Surface when an empty Neighborhood(0) pane requires zero. One first restores Full and later clears the same pane; the other intakes directly from an already-empty Neighborhood(0) display. Both pass their persisted-intake, unchanged writer/settings/session and exact durable-byte controls before that final assertion. A reviewed three-file repair replaces the permanent intake pane with transient foreground focus, preserves authored rosters and other roots, and connects selection clearing/replacement to focus retirement. Root has applied that exact reviewed three-file candidate at the new3,287-input freeze. The full native Graphshell library passes297 tests with five inherited ignores; both unchanged real Firefox IndexedDB regressions now pass, with exact durable bytes and all original assertions retained. The actual intake-repair Wasm presenter build also passes. These renderer-free intake controls qualify facade visibility and focus retirement; the complete headed intake flow remains pending. The independent readonly-check defect is tracked separately. Task7 and Task8 remain unfinished.

- **2026-10-11:** The fresh repaired presenter passes the complete retained UI workflow with14 functional checkpoints, including strict same-head Demand byte preservation, hydrated edit/undo, both Manual and HostDriven1700 settings, actual foreground transition, and ordinary Stored close/reopen. Its dirty-settings-held-reader case remains explicitly unimplemented; the workflow is not acceptance of that boundary. A separate native finite held-read control passes pending-head Apply refusal before effects, exact unchanged durable bytes with no automatic replay after retirement, and one fresh Apply storing the intended settings exactly once. The checkpoint timestamp allowance is limited to the exact CURRENT session graph.json, every other raw byte/decoded field remains identical, and the actual native PUT timestamp lies within recorded browser save-time bounds. The native intake flow again stops at the4GiB available-memory floor after successful stored/unpinned intake, before finalempty0. That is an infrastructure hold; both unchanged real IndexedDB facade intake regressions remain separately qualified. The first failed-store hidden control stops before its fault because the fixture graph item moves between geometry sampling and native click. Its strict event-time geometry guard is retained, and a bounded public fixture preparation is under review. Task7 and Task8 remain open.

- **2026-10-11:** A separate renderer-free real Firefox IndexedDB control passes both Workbench orderings: save before preparing a graph check, and save after its native completion but before installing it. Only the bound workspace key changes (record counts199→200, then200→200), with every other stored byte, graph reader and empty layout history preserved. This qualifies sequential order and namespace separation; simultaneous pending transactions and painted Forme remain separate. A second headed intake run stops at3.79144GiB available memory before finalempty0. A read-only sampler records an owned Firefox AMD client’s GTT counter growing from618744 to3841912KiB over66seconds while global available memory falls from7.32 to4.14GiB and ordinary browser cgroup charge changes from1.03 to1.09GiB. Duplicate file-descriptor counters are deduplicated. This is a correlated graphics-memory lead, with source allocation and causality still under investigation. The strict hidden-failure fixture now checks deadline expiry before admitting selection and is running with its original recovery oracles. Task7 and Task8 remain unfinished.

The graphics counter increase is an endpoint comparison, not monotonic growth: the same client also shows two substantial reclamation drops during the sampled interval. Permanent retention is unproved. The next bounded diagnostic compares actual visible, hidden and visible-again phases on one disposable tab. The hidden-save control reaches a real53-operation aborted metadata batch and preserves the old reader and exact durable bytes, then crosses the memory floor during the required visible due interval. Its later hidden retry and reopen assertions remain unreached.


- **2026-10-11:** A shorter unchanged-source visibility and ordinary-close control completes all four six-second windows, with actual hidden/visible events, byte-identical storage, released writer lease and clean cleanup. Available memory remains above5.27GiB. The same owned graphics client falls to498520KiB while hidden, grows after foreground return, then holds2468016KiB briefly after ordinary close before falling to436344KiB with Firefox and the empty observer still alive. This qualifies delayed reclamation in this no-edit control. Allocation provenance, GPU completion and a safe repair remain unproved. A separate minimal renderer fixture is being prepared to distinguish basic raster/presentation pressure from graph physics and producer staging. The final headed intake, hidden-save recovery, Forme and lifecycle controls remain pending; Task7 and Task8 remain open.

- **2026-10-11:** The minimal renderer compiles and runs with the current locked renderer family. Its fixed rectangle performs120 raster/presentation frames; all74 native queue completion promises fulfill, including the final submitted work. The owned Firefox graphics client rises from128460 to954316KiB, then falls to430028KiB during the idle drain before tab close. This reproduces substantial transient pressure without graph physics, producer staging, text or images. The allocation responsible and a safe source repair remain unproved. A short UI control is prepared to distinguish native layout scrolling from browser scrolling of the accessibility mirror after field focus. Its first two admissions defer before execution for insufficient available memory. Task7 and Task8 remain open.

The short native UI diagnostic completes and records a concrete accessibility alignment defect: the invisible mirror scrolls331pixels after focusing Refresh mode, while the painted canvas and outer panel stay fixed. Earlier genuine wheel inputs move the native panel content normally with mirror scroll zero. Unsubmitted drafts preserve all durable bytes. A strict regression check and a narrow focus-with-preventScroll repair are being prepared; the current source remains frozen until baseline failure and review. The full intake endpoint remains unqualified.

The alignment regression and two-file preventScroll candidate pass independent static review; the repair remains unapplied. Baseline regression execution defers before opening a browser at7.515GiB available memory, and a later capacity check remains below8GiB. The ordered acceptance condition is unchanged native baseline failure after storage preservation, reviewed source apply/build, then the same regression passing with mirror root unscrolled and native boundaries aligned. A separate one-target/120-renders renderer comparison also passes static review and awaits actual execution. Both retain the existing memory floor and all original evidence.

- **2026-10-11, applied but untested:** At the user's explicit request to land the reviewed repair before runtime testing, MirrorHandle now focuses with `preventScroll: true`, with only the required `web-sys` FocusOptions feature added. Local commit `2530e44d0` contains exactly these two files; all13 Cargo.lock files and the remaining3285 source inputs are unchanged. The earlier baseline-before-apply ordering is superseded by this decision. No build or browser regression has run on the repair, so successful alignment is not claimed. The observed before-fix331-pixel mirror displacement and the unchanged native-input regression remain preserved; next verification is a fresh presenter build/package and that regression on the repaired source. Task7 and Task8 remain open.

- **Published for testing, untested:** With explicit user approval to push and request testing, the exact two-file focus repair is published on `a11y-mirror-prevent-scroll` as `d51cb3e8373a2a1a22d3f542f99d13fbaf2c96e0`, based on fetched main `04a48c2d64752d5da56e523bdd0ed93b1ef2f37e`. Both source files match local repair2530e44d0 byte-for-byte. A [commit testing note](https://github.com/merely-made/mere/commit/d51cb3e8373a2a1a22d3f542f99d13fbaf2c96e0#commitcomment-204507602) requests native field/select focus, mirror/paint alignment and ordinary panel scrolling checks. Remote branch SHA and comment content are verified. Earlier resident checkpoints and working-tree changes are outside this publication. Runtime validation and merge approval remain pending; Task7/8 remain open.
