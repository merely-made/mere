# Distillery Projection Walk Plan

**Date:** 2026-09-02
**Status:** W0 complete. W1's endpoint, admitted catalog seam, Graphshell
mount, session-local resume by diff, detailed frozen table, machine-readable
headless receipt, admitted `ResidentProjectionHost` carrier path, and headed
WebRTC fixture receipt are green 2026-09-06. A readable three-card layout and
live Distillery-resident route remain before W1 is closed. Continuation
across a fresh admission is a separate protocol question because it receives
a fresh transcript-derived projection session (Progress). §2 was read at mere
`77a3701f052` and corrected at
`3ce750f5` by the W0 implementation, which read the code rather than this
plan.
**Scope:** the first end-to-end scene binding for a port — dataset → scene →
Scenograph → host — walked on Distillery. This plan owns the walk; the
[Distillery v0 plan](2026-08-12_distillery_v0_plan.md) owns the works
(resident authority, installed boundary, trainer, host-policy composition) and
nothing here changes what that plan rules.
**Companions:** the
[port GUI composition and comparable stacks brief](../research/2026-09-02_port_gui_composition_and_comparable_stacks_brief.md)
(why this port, and why a walk at all), the
[projection/scenes direction note](../../2026-08-23_projection_scenes_and_graph_native_platform.md)
(the nine questions this plan answers), the
[scenograph content catalog](../research/2026-08-18_scenograph_content_catalog.md)
(the Chronicle, Circuit, and Loom recipes), the
[projection grammar catalog](../research/2026-08-15_projection_grammar_catalog.md)
(promotion rules, should a recipe need a primitive the grammar lacks), the
[projection grammar adoption plan](2026-08-15_projection_grammar_adoption_plan.md)
(A3 and A5 gates this walk may supply evidence to, never close), and the
[suite census](../../2026-08-22_turnstone_suite_composition_and_capability_census.md)
(§6, the contribution seam Distillery already provides through; §3, the
Graphshell charter; §7.4, the Alembic re-ruling).

## 1. Why Distillery

Three reasons, each a fact rather than a preference:

- It is the one port with **models, work, and a Cambium surface** in one
  place: the model works by charter, Alembic's work-runs by the 2026-09-02
  re-ruling, and a read-only installed surface (`distillery.installed.v1`)
  that Turnstone already admits as the contribution seam's second provider.
- It has **no projection endpoint**. Nothing in `ports/distillery` implements
  `ProjectionCatalog` or `ProjectionSource`, so the walk forces the whole
  path rather than the half the receipts plan already proved.
- Its **second host already exists.** The family thesis's anti-shell test says
  the receipt for a platform capability is a second host that keeps its own
  identity. Distillery's own surface and Turnstone's admission of it are two
  such hosts, before Graphshell is counted.

## 2. What Distillery owns, as data

Read from `crates/mesh/mesh/src/board.rs` and `ports/distillery/src/`.

**The job board.** `Job { id: JobId([u8; 32]), kind | spec, payload, posted_by:
[u8; 32], state, lease: LeaseRecord, next_claimants: Vec<Claimant> }` with
`JobState::{Posted, Claimed { winner }, Done { winner, result }, Committed {
winner, output }}`. Two wire generations (M1 inline, V2 content-addressed)
share one state machine; the claim-race winner is deterministic; leases are
clock-free admissible facts with epochs. This is event-shaped data with
spans: a job is posted, claimed, and terminated, and a lease is a span with an
epoch.

**The resident receipt stream.** `ResidentReceipt::{Tick { steps },
MaintenanceCompleted(report), MaintenanceIdle, MaintenanceFailed { error },
SupervisorFailed { error }, StopRequested }` — one exact observation per
supervisor turn, the ticks the v0 plan's done-condition runs on `SystemClock`.
This is the clock the board's events hang from.

**Installed facts.** `DistilleryInstalledSnapshotV1 { profile, protection,
mesh_id, mesh_root, mesh_store_path, blob_store_root, resident:
Option<DistilleryResidentSnapshotV1> }` — the last a newtype over
`ResidentSettings`, which W4's prop shape will feel — plus the latest receipt:
what the surface renders today. These are status facts, not a scene (§3).

**Artifacts and provenance.** `TrainRequest { base_model_ref, tokenizer_ref,
corpus_ref, adapter_name, prompt_template, metric: EvalMetric, settings:
LoraTrainerSettings, created_at }` → `TrainReceipt { adapter_manifest_ref,
eval_report_ref, adapter_blob, adapter_config_blob, baseline: EvalTally,
adapter: EvalTally }` — both tallies, which is what makes §3.2's
compare-two-evals intent supportable from the data. `FloraContribution {
contribution_id, manifest_ref, manifest, adapter_config_json: Vec<u8>,
adapter_safetensors: Vec<u8>, weight }` → `FloraAggregate { manifest,
adapter_config_json, adapter_safetensors, receipt }`. Training provenance is
edge-by-reference: every reference is a content-addressed `ManifestId`, so it
is a DAG whose edges are references. The Flora records are not: they carry
the adapter bytes inline beside a manifest reference. So the Circuit's blocks
are of two kinds — reference-only and payload-bearing — and W3's entrance
check decides how a payload-bearing block is represented before any of it is
drawn.

**Retention and custody.** `RetentionSettings`, `ResidentStorage`, and
`MaintenanceReport { checkpoint, candidates, collected, effects }`, per tick:
`candidates` is the count of blob refs safe to release, `collected` the
custody claims actually removed, and the gap between them is what a custodian
refused. Nothing counts what was kept; a Chronicle span for a maintenance
window can show released-versus-refused, not retained.

## 3. The scene binding, by the nine questions

The direction note's rule: identify which projection layer owns a candidate's
novelty before naming a scene; then answer the nine questions; then prove it
on a second dataset. Applied to Distillery's data, two scenes survive, one
stays a candidate, and one thing is not a scene at all.

### 3.1 Chronicle over the board and the receipt stream (founding)

| Question | Answer |
| --- | --- |
| 1. Reads | `Job` (id, poster, state, lease epochs, claimants) and `ResidentReceipt` ticks and maintenance reports. |
| 2. Derivation | Order by tick; a job's span runs Posted → terminal; a lease's span runs its epoch; maintenance events group by tick. Temporal and grouping derivation only. |
| 3. Entity becomes | An **event card** (a job) or a **span meter** (a lease, a maintenance window). |
| 4. Relation becomes | An **arc over the axis**: poster → winner (the claim), job → committed output (the commit), maintenance → the jobs it retained or erased. |
| 5. Geometric law | A timeline spine; era bands are lease epochs; position is time, size is span. |
| 6. Guides | The time axis and era bands; a scrub transport over `scenotime` diffs (the recipe's native motion). |
| 7. Back-mapping | `JobId`, author keys, `ManifestId`, tick index — every mark names its source. |
| 8. Intents | v1 advertises none that change truth. Reclaim, cancel, and retention changes are the authority's petitions and stay with the works; the projection may advertise them later as typed intents the endpoint refuses or forwards, never performs. |
| 9. Second dataset | Djinn's resident job log (the same grammar, a different owner), then a heterogeneous one: the browsing trace or radio traffic, both named as Chronicle transfers in the catalog. |

Layer pre-filter: the novelty is the *reading* (events and spans from a
claim-race board) plus the arrangement (timeline with era bands); the
Chronicle recipe holds both. Nothing new enters the grammar. A3's stage-one
ladder applies unchanged — cards at close zoom, glyphs far out.

### 3.2 Circuit over training and Flora provenance (second, heterogeneous within the port)

| Question | Answer |
| --- | --- |
| 1. Reads | `TrainRequest`, `TrainReceipt`, `FloraContribution`, `FloraAggregate`, and the manifests they reference. |
| 2. Derivation | Traversal: follow `ManifestId` references into a DAG. |
| 3. Entity becomes | A **component block with ports**: corpus, base model, tokenizer, trainer run, adapter, eval report, contribution, aggregate. |
| 4. Relation becomes | An **orthogonally routed trace** from producer port to consumer port; direction always drawn. |
| 5. Geometric law | Grid-snapped placement by topological rank; bundling at density. |
| 6. Guides | A blueprint backdrop; a legend for block kinds. |
| 7. Back-mapping | `ManifestId` on every block and trace. |
| 8. Intents | Open a manifest; compare two eval reports (`EvalTally` baseline against adapter) — read-only intents the endpoint serves as resources. |
| 9. Second dataset | The workspace dependency graph, Circuit's own founding dataset — which proves the recipe is not Distillery's skin. |

Entrance check before W3: the grammar must already carry orthogonal routing
and port-anchored endpoints, or those are a promotion proof first (the
catalog's rule; the adoption plan's A5 names ELK's port and label anatomy for
exactly this). This plan does not decide that; it records the check.

### 3.3 Loom over device lanes (candidate, gated)

Lanes are claimants or devices; nodes are jobs in lane order; cross-lane edges
are leases handed off or reclaimed. The direction note lists Streams as
unresolved — it earns separation from Timeline only if lane membership sets
one axis and cross-stream crossings govern the view. Until the works runs
multi-device traffic where the crossings are the point, this is a Chronicle
variant, not a scene. Gate: real multi-device job traffic in the receipts.

### 3.4 Not a scene: the installed surface

Profile, protection, mesh id, roots, resident settings, latest receipt — this
is Steward-shaped status (census §8: status projections need exposure, not a
scene). It stays the retained surface it is, and becomes the frame the
Chronicle sits in (W4).

## 4. Phases and done-conditions

Each phase names its forcing consumer and its receipt. Nothing here closes a
gate in the adoption plan; evidence it produces is filed there.

**W0. Assemble — landed 2026-09-02, verified 2026-09-03.** Fixtures for both
second datasets, readable without Distillery, plus a headless drive of the
installed surface through its own runner (the headed `.scn` belongs to W4,
because Turnstone admits the surface through a *pinned* mere revision in
another repository).
*Done when:* both fixtures exist under test, and a bare scenario drives the
installed surface end to end. Nothing rendered yet.
*State:* complete. All three tests pass in the real crate as of 2026-09-03
(Progress).

**W1. The endpoint — software acceptance implemented 2026-09-06.** `ports/distillery` implements `ProjectionCatalog`,
`ProjectionSource`, and `ResumableProjectionSource` over the board and the
receipt stream: one offer whose score uses the timeline arrangement with
one disclosed observation-tick axis value per item; lease epoch and window
remain presentation facts because `ScoreItem` has one axis. It supplies a
presentation manifest of `PortableCardV1` per job and no intents in v1. Served through the resident
projection host like every other endpoint; admitted by Notochord; resumable
by revision.
*Done when:* Graphshell mounts Distillery's offer and renders the job
Chronicle as a served scene; snapshot, diff, reconnect, and resume are in one
machine-readable receipt; the `FrozenScene` realization lists jobs by name
with their spans in its table; a second Distillery session over the same
mesh yields byte-identical scores (determinism receipt).
*Current next step:* make the three mounted Chronicle cards visually distinct
and readable, then wire the observer to the live Distillery resident route and
repeat the headed capture there. The checked-in headless receipt
carries snapshot revision 11, same-session rediscovery and resume by a
contiguous Distillery diff to revision 12, three named job rows, and
presentation-supplied tick and lease-span detail. The admitted carrier test
separately runs `ResidentProjectionHost::accept_one` over `MemoryTransport`,
binds the factory to the admitted session, transfers the snapshot and all card
resources, rings revision 12, applies its diff in `ClientState`, freezes the
  updated table, and closes cleanly. It is a software carrier receipt. The
  separate headed fixture proves admitted WebRTC delivery and browser mounting,
  but its capture exposes overlapping card geometry rather than a readable
  Chronicle. A fresh admission mints a new
projection session, so carrying an old acknowledgement across it requires an
explicit continuity contract before this plan can claim reconnect across
admissions.

**W2. The binding, authored.** The Chronicle recipe is expressed as a
Scenograph definition — Source, Reading, Encoding, Arrangement, Interaction,
Appearance, Provenance — over Distillery's endpoint, cited by a shelfmark whose
`expects.generation` checks against the board's checkpoint hash. The same
definition is then pointed at Djinn's resident log.
*Done when:* one authored definition, two datasets, both read true in the
headed receipt; the shelfmark round-trips; changing one lever (era bands off)
yields a variant, not a new definition.

**W3. Circuit.** Training and Flora provenance as Circuit, subject to §3.2's
entrance check; the workspace dependency graph as its second dataset.
*Done when:* both render through one recipe; every block resolves to a
`ManifestId` or a crate; the eval-comparison intent is advertised and served
as a resource; the frozen realization is a navigable table of blocks and
traces.

**W4. Hosts.** Three, in order: Graphshell viewing (W1 gives it, and it is the
preeminent Scenograph host); Distillery's own Cambium surface embedding the
Chronicle beside its installed facts, through Cambium's scene-hosting rule
(props in, retained local interaction state, typed events out); Turnstone
through the contribution seam it already admits `distillery.installed.v1` by.
*Done when:* one scene realized in two hosts that keep their own identity,
workflow, and authority — the anti-shell receipt — each with a headed
genet-probe receipt, one Tab order, one AccessKit tree, and the table
alternate reachable in both. Turnstone is the third host, not the second.

**W5. Alembic's work-runs.** Once the Alembic move is ruled and executed, its
bounded-actor runs join the board Chronicle as a second job kind: same
recipe, one more entity kind, no new scene.
*Gate:* the move is executed (2026-09-02, `ports/distillery/{alembic,athanor}`
founded); what remains is Alembic's workshop implemented far enough to emit
runs onto the board. Not opened before that.

## 5. Boundaries

- The projection reads; it never performs. Reclaim, cancel, retention, and
  lending remain the works' petitions through the authority.
- No inference anywhere in the path. Derivations are ordering, grouping, and
  traversal; a model-produced derivation, if ever wanted, is a separate
  provider above the reading.
- No new grammar primitive without a promotion proof. §3.2's entrance check
  is the one place this plan expects to meet that rule.
- No change to what the v0 plan rules about the works, the installed
  boundary, or host-policy composition.
- Scenograph is built with Cambium and hosted; it does not acquire
  Distillery's authority or product policy.

## Findings

### 2026-09-02

- `ports/distillery` has no `ProjectionCatalog`/`ProjectionSource`
  implementation; its `chirograph` and `sceno` use is nil. Its surface is
  `distillery.installed.v1`, a `RetainedSurfaceSession` over
  `GenetAppRunner`, rendering installed facts and the latest receipt.
- The mesh board's `JobState` is a four-state machine with a deterministic
  claim-race winner and clock-free lease records with epochs — natively
  event-and-span shaped.
- Training and Flora records reference each other by `ManifestId` only, so
  provenance is a content-addressed DAG with no product identifiers to leak.
- `ScoreItem.axis` already exists for arrangements that place along an axis
  (the score's own doc names Timeline among them), so W1 needs no contract
  change to disclose tick and epoch.
- Turnstone admits the Distillery surface as the contribution seam's second
  provider (v0 plan status), which makes the anti-shell second host a
  composition receipt rather than new machinery. Concretely: turnstone's
  `src/shell/mod.rs` registers a built-in `DistilleryInstalledProvider`, and
  its `Cargo.toml` pins `distillery` to a mere git revision — so a headed
  scenario there sees the pinned Distillery, not this checkout.
- W0 (2026-09-02): `ResidentReceipt` does not derive `Serialize`, so the
  fixture's tick stream is a six-variant `TickRecord` mirror in the test file
  (the two failure variants included, to keep the mapping total).
  `mesh` re-exports `MeshExt` and `to_operation` but not p2panda's
  `Operation`, so a test that authors a log names `p2panda-core` as a
  dev-dependency. Tracked `.json` under the fixtures comes back CRLF unless
  `.gitattributes` says otherwise; it now does for
  `ports/distillery/tests/fixtures/**`.
- W0 (2026-09-02): in this checkout `cargo check -p distillery` fails inside
  `src/surface.rs` before any test is reached — the local genet working copy
  split `genet-host-api`'s application half out as `mere-surface-api`
  (genet `57bcc38fdae`), `.cargo/config.toml` patches cambium to that local
  genet, and mere's `surface.rs` still imports the surface contracts from
  `genet_host_api` at the pinned `eff0cb6`, so two `SurfaceDescriptor` types
  meet at `distillery_installed_surface`. Local-dev only: a clean checkout
  resolving genet from `eff0cb6` is unaffected. The migration of mere's
  side is the boundary plan's P1/P3 work, in flight in another session, and
  not this plan's to make.
- W0 refresh (2026-09-05): the boundary plan records P2 complete and the
  Cambium/engine-management P3 receiving move landed in Mere; the live
  workspace now owns `crates/cambium/*`, `crates/system/surface-api`, and
  `crates/system/{content-contract,document-lanes,errand,fetch,luggage,
  notochord,pandect,proofs,registry,resident,shell-state,ux-events}`.
  `ports/distillery/src/surface.rs` imports `mere_surface_api`, so the old
  local split mismatch is no longer an active prerequisite hold. A new locked
  `walk_fixtures` run began with an isolated target directory but was stopped
  during its first dependency build to avoid consuming the shared machine; it
  supplies no new pass or fail result. From `C:\Users\mark_`, outside the
  repository's `.cargo` directory and with no user Cargo config present,
  `cargo metadata --manifest-path C:\Users\mark_\Code\repos\mere\Cargo.toml --locked --no-deps --format-version 1`
  exited 0. This checks workspace manifests and member discovery without the
  local repository patches; `--no-deps` does not prove dependency resolution,
  compilation, or tests for Distillery.

## Progress

- 2026-09-02: plan founded from the assessment in the port GUI composition
  brief, on Mark's "walk distillery". No code opened.
- 2026-09-02: **W0 landed, one leg unverified.** Implemented by a subagent
  under a paths-only brief (new files under `ports/distillery/tests/`, one
  script, one dev-dependency line) while another session worked the tree.
  Landed: `tests/walk_fixtures.rs` (569 lines) with three tests;
  `tests/fixtures/chronicle/{distillery_board,djinn_board}.json`, two boards
  of the same grammar and different owners built by `JobBoard::fold` over
  seeded p2panda operations (3 and 4 jobs; Posted, Claimed with a lease,
  Committed V2; a second claimant so `next_claimants` is non-empty), each
  with a `TickRecord` stream; `tests/fixtures/circuit/workspace_graph.json`
  (100 packages, 230 edges, acyclic, `generated_from` the HEAD it was cut
  at) produced by `scripts/workspace_graph_fixture.py`, byte-identical on
  regeneration except for that label when HEAD moves. Verification: the two
  data tests were type-checked and run against the committed files in an
  isolated probe crate that path-depends on the same `mesh`, `personae`, and
  patched `p2panda-core`, three processes, byte-equality asserted across
  them — because `cargo test -p distillery` cannot link in this checkout
  (Findings). The headless surface drive
  `installed_surface_drives_headless_through_its_runner` is written against
  the surface's own unit-test pattern and every call checked against the
  pinned `RetainedSurfaceSession`, and it has not run. Re-run
  `cargo test -p distillery --test walk_fixtures` twice once mere's surface
  imports follow the `mere-surface-api` split. §2 corrected in five places
  by the same pass.
- 2026-09-03: **W0 verified in the real crate; W1 then held.** Mere was repointed
  to genet head after P1 (`487e18a4`), `surface.rs` now imports from
  `mere_surface_api`, `cargo check -p distillery` is clean, and
  `cargo test -p distillery --test walk_fixtures` passes 3 of 3 — the
  headless surface drive included. A second run minutes later could not
  resolve at all: the local genet checkout was mid-edit and `cambium`'s
  manifest inherited a `mere-surface-api` entry its workspace no longer
  declared. That is the boundary plan's P2/P3 (Cambium and the surface
  contracts moving toward Mere) in flight, not a regression; the determinism
  receipt for the fixtures already stands from the probe (three processes,
  byte-equality). W1's endpoint would build against exactly the crates that
  are moving, so it is held until those moves settle rather than written
  twice; its brief is ready: `ProjectionCatalog::describe`,
  `ProjectionSource::snapshot` with `SceneSnapshot::from_dense`,
  `PresentationSource::resource`, `ResumableProjectionSource::resume`, the
  `LiveEndpoint` pattern in `ports/graphshell/src/live_endpoint.rs`,
  `Arrangement::Timeline` with `ScoreItem.axis: Option<AxisValue>`, and
  registration through `ResidentEndpointCatalog::register_resumable_notifying`.
- 2026-09-05: **W0 prerequisite refresh.** The boundary plan now records P2
  and the Cambium/engine-management P3 move as landed. W1 therefore no longer
  waits on that move and is the next implementation target; its endpoint and
  joined served-scene, reconnection/resume, readable-table, and determinism
  receipt remain open. The attempted real-crate PowerShell command was
  `$env:CARGO_TARGET_DIR = 'C:\Users\mark_\Code\target-distillery-projection-walk-20260905'; cargo test -p distillery --test walk_fixtures --locked -j 1`;
  it was intentionally interrupted during initial dependency compilation, so
  the standing 2026-09-03 3/3 receipt remains the latest completed test result.
- 2026-09-06: **W1 headless path implemented and verified.** Distillery now
  owns `ChronicleEndpoint`: a caller-session-bound, solver-realized Timeline
  score over the folded `JobBoard`, deterministic content-addressed
  `PortableCardV1` resources, explicit generation/epoch/revision, one retained
  contiguous diff, notice polling, snapshot fallback outside that retained
  base, and total intent refusal. Graphshell owns `ChronicleMount`, catalog
  registration through an `AdmittedEndpointContext`, client acknowledgement,
  reconnect/resume, and the frozen realization. `FrozenInstance` gained an
  optional presentation-supplied detail field so the generic table can retain
  card facts without learning Distillery vocabulary. The checked-in
  `ports/graphshell/docs/receipts/distillery_chronicle_w1.json` records the
  revision 11 snapshot, reconnect-by-diff to 12, and three frozen job rows.
  Verification with isolated `CARGO_TARGET_DIR` and `-j 1`: `cargo test -p
  distillery --lib chronicle --offline` passed 4/4; `cargo test -p graphshell
  --lib distillery_w1 --offline` passed 4/4 (181 filtered). The latter covered
  the real Distillery endpoint, the generic diff client, exact admitted
  session/subject delivery into the factory, and the table's tick, lease epoch,
  and lease window values, and byte-compared the generated receipt with the
  checked-in JSON. A subsequent `--locked` repeat was stopped while waiting on
  the shared Cargo package-cache lock; the completed unlocked offline run had
  already updated and exercised the current lockfile graph.

  The implementation exposed two planning corrections. First, `JobBoard`
  preserves current state and lease epochs but no posting timestamp, while
  `ResidentReceipt` has no `JobId`; the present Chronicle can truthfully place
  an observation and show lease spans, but cannot reconstruct Posted-to-terminal
  job spans. A historical Chronicle needs an owner-supplied correlated event
  journal before W2. Second, a projection's accessible alternate needs readable
  presentation facts as well as scene identity. That is now a generic frozen
  contract rather than a Distillery special case.

- 2026-09-06: **W1 admitted resident-host software path verified.** Distillery
  now owns `ChronicleObserver`, which retains the materialized board, card
  resources, one contiguous diff, and notice generation without retaining any
  admitted session. Its factory creates a session-bound `ChronicleEndpoint`
  for each admitted context. The Graphshell integration test
  `distillery_chronicle_host` uses a signed delegation and real
  `MemoryTransport` dial into `ResidentProjectionHost::accept_one`; over the
  carrier it opens the Chronicle, takes revision 11, fetches and validates all
  three `PortableCardV1` resources, observes revision 12, receives the notice,
  resumes by the real diff, fetches the three changed cards, builds the frozen
  table, and closes. `cargo test -p graphshell --test
  distillery_chronicle_host --offline -j1` passed 1/1 in the isolated
  `target-distillery-w1-root` target.

  This closes admission and serving as software, not W1 as a product path.
  Four evidence layers must stay distinct: direct endpoint behavior, admitted
  carrier serving, headed realization, and a live resident feeding the route.
  At this point the first two are green; headed transport/mounting is green but
  readable realization is not, and the live resident remains open. The review
  also corrected “reconnect” in the earlier
  receipt: `ChronicleMount::reconnect` is same-session rediscovery. Ordinary
  reconnection is a new admission whose nonces and carrier binding mint a new
  `ProjectionSession`; the retained Distillery observer preserves source state
  but does not authorize Graphshell to transplant an old session's
  acknowledgement. Cross-admission continuation therefore belongs in an
  explicit protocol envelope, keyed by retained authority and source identity,
  rather than in Chronicle endpoint state.

- 2026-09-06: **W1 headed fixture transport and mount verified; readable layout
  remains open.** Graphshell's C4 host now has an opt-in
  `distillery-chronicle-fixture` feature and `--endpoint
  distillery-chronicle` launch mode. It loads Distillery's checked-in three-job
  board, creates a session-bound endpoint from the shared observer in the
  admitted route factory, and leaves the ordinary live-board endpoint as the
  default. Both the Chronicle feature configuration and the default
  `webrtc-session` configuration pass focused `cargo check`; the current
  fixture executable also completed a native build. The headed Chrome scenario
  `distillery_chronicle_w1.scn` passed 11/11 steps over the real WebRTC door:
  link `webrtc`, state `open`, revision `11`, three remote cards, selectable
  remote session, and one capture. Receipt artifacts are under
  `C:\Users\mark_\Code\testing\mere\scenarios\graphshell-web\distillery_chronicle_w1`.

  The capture is useful adverse evidence: the status and semantic receipt say
  three objects, but the canvas presents their geometry as one overlapping,
  unreadable block. This proves headed carrier delivery and mounting, not W1's
  readable Chronicle experience. W1 therefore still needs a layout/encoding
  correction, a headed capture that visibly distinguishes all three named
  jobs, and the live resident-owned route.
