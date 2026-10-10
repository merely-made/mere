# Graph semantics follow-ons: adoption, RDF cleanup and residency

**Date:** 2026-10-09
**Status (2026-10-10):** In progress. RDF retirement is qualified and published;
source `2976821783cee6b24d13cacb046c1d99b1890fb4` passes all six gates. Consumer
adoption is qualified, reviewed and published after Mark's approval: Knot
`ab191d4d`, Redshank `fea7f0f7` and Turnstone `bf1e6a2f` are their confirmed main
and adoption-branch tips. Turnstone includes the reviewed current-main document
pump and passes 759 default and 792 Piccolo tests (nine inherited skips each),
plus both all-target checks. Fresh canonical HTTPS fetches reproduce all three
qualified file maps; all five locked Cargo metadata checks pass with a fresh Git cache and unchanged
source/locks. Consumer adoption (A1) is complete.
All 108 owned Mere dependency rows select the qualified source. This establishes
source compatibility; actual Turnstone residency authority remains A2.
The approved initial Graphshell/Djinn resident-host plan has Tasks 1–4 reviewed
and locally committed at `a61dfb5c9`, `652acbb72`, `6b2f2860b` and `29ab75c56`.
Task 4 settings, pane roots and exact scope gates pass; Task 5 lifecycle-safe refresh is in progress. The initial host still retains its complete writer and
history; native/product/browser host qualification and publication remain pending.
Mere's last published checkpoint remains `a8889f60e`.

## Authority and intent

Mark authorized orchestrating three follow-ons after the
[bounded graph semantics lane](2026-10-04_graph_semantics_plan.md) completed:
consumer adoption, the deferred RDF cleanup, and partial-view editing,
automatic refresh and obsolete-generation cleanup. This is a separate lane;
it does not reopen P1–P5 or erase their qualified publication receipts.

Bring the established Mere behavior into the actual browser hosts, keep query
behavior while removing redundant test dependencies, and make resident views
editable and current without losing recorded truth or history. Reuse existing
graph/session authority, projection, storage, event-loop and backend interfaces.
Transport modularity, data projections and the shared wgpu device remain intact.

Local investigation, changes, tests and commits are authorized. Mark reviews
concrete qualified candidates before push or merge. Original worktrees,
unpublished compatibility commits, dirty primary checkout receipts and other
active writers remain protected. Downloads are allowed; no offline restriction
is part of this lane.

## Findings (2026-10-09)

- Mere source `7628ab689` and publication `443201a7c` complete P1–P5. The original
  3088-file source receipt and native/wasm/real IndexedDB evidence are retained.
- Turnstone compatibility `93f1e45a` already incorporates current remote main
  `22361c9d`. The initial survey found it 16 commits ahead; fresh origin fetch
  now confirms it published. Turnstone then advanced to documentation/receipt
  update `e86906a3`; the isolated successor incorporates that without changing
  the prepared repin. Knot `802238cb` and Redshank `e08bf4b9` are likewise
  published on their current mains. Preserve their
  resource adoption and refusal protections in isolated successors.
- These consumed workspaces select Mere `3b3afa289`. Prepared manifests
  move from rejected initial RDF candidate `3c9abef1b` to repaired, source-qualified
  `297682178`. Consumer lock resolution and compilation follow that repin. Knot and Redshank must receive qualified immutable
  candidate hashes before Turnstone changes its provider references.
- Turnstone owns an editable Canvas and custom snapshot/facet persistence,
  rather than GraphSession/MereSessions. Graphshell's MereHost already owns
  GraphSession, and Djinn shares that host by session and emits revision notices.
  P5 resident-open/publication APIs currently have only test callers.
- A partial reader beside a complete writer retains the whole graph in the
  host process. Temporary complete-session opening reduces idle retention but
  still has a Full memory peak during edits. This is distinct from a bounded
  writer, which requires an explicit supported-operation/dependency contract.
- Turnstone's current identity Forme roots every Surface; it cannot demonstrate
  neighborhood savings. Authored scoped projection roots must precede resident
  filtering. The existing Canvas scope helper clears an empty resolved set to
  Show all, so an exact empty/unloaded scope needs a separate preserved contract.
- Keep is a per-view control, distinct from graph pinning. Multiple pane focuses
  must all be retained without inventing durable pins or changing Keep semantics.
- Layout coverage currently drops at consumer boundaries. Resident loading must
  transmit and present coverage, including context-only updates.
- The old Oxigraph Store oracle shares the locked spareval/oxrdf implementation
  with the retained evaluator. Its independent-engine/newer-model comments are
  obsolete. Removing its feature also removes implicitly enabled query extensions;
  unchanged behavior requires explicit feature retention and exact-result tests.
- Muniment transactions can protect collection decisions, but the current
  IndexedDB implementation snapshots every object-store value. Scoped transaction
  reads are necessary before claiming bounded browser maintenance memory.

## Phases and done conditions

### A1. Coherent consumed-family repin

Work in isolated successors of Turnstone `93f1e45a`, Knot `802238cb` and
Redshank `e08bf4b9`. Update every Mere Git row in consumed workspaces, including
actual Mere patch rows. Retain unrelated Genet/engine pins and Woodshed root
rows outside the consumed Redshank port workspace. Preserve all unpublished
ancestors. Resolve and qualify providers first, then choose their exact hashes
in Turnstone. A local candidate is not proof of remote clean-checkout availability.

Done: sibling scoped tests and all-target checks pass; Turnstone default and
Piccolo library/all-target checks pass; dependency metadata resolves one coherent
Mere source; existing capture admission, identity, persistence/refusal, fork and
journal-origin controls remain intact. Publish approved suppliers and providers
before consumers, then verify a clean remote resolution.

### A2. Actual host adoption and coverage

First wire the existing Graphshell/Djinn session authority to qualified checkpoint
publication, resident readers, explicit roots and postcommit notices. Then adopt
that same authority boundary in Turnstone's custom lifecycle. Preserve legacy
save/migration/refusal behavior; never let two persistence authorities write the
same graph. Carry product-neutral, defaultable coverage through portable snapshots
and client presentation. Missing old-wire metadata does not independently prove
completeness.

Full remains the compatibility setting. Neighborhood is graph-scoped and uses
configured hops plus an authored scoped Forme. Preserve the authored roster when
members are unloaded or missing; an exact empty roster stays empty. Retain the
union of all open projection roots, explicit pins and every pane's transient
focus. Keep retains its existing separate meaning. Settings, projection choice,
manual refresh and coverage presentation must be usable through the host shell.

Done: two panes retain their union; closing one releases only its unnecessary
neighborhood; empty scopes do not show all; NotLoaded reaches the rendered/client
view; demand restores exact parallel/self statements; Full removes Residency
limits. Edit, store, publish, notice, refresh and close/reopen preserve identities,
authors and controls through a real endpoint and a real Turnstone profile. Failed
publication/refresh and stale completion preserve the previous view. Report
process memory separately from a resident-record probe.

### B. Conservative RDF retirement

Remove the old Store module/comparisons/timing-only test and optional Oxigraph
query dependency in `crates/graph/linked-data`. Keep the complete borrowed-adapter
versus materialized-spareval query battery, exact terms/scopes/reifiers/assertions,
coverage and saved-query controls. Preserve previously available SEP-0002,
SEP-0006 and calendar support explicitly on spareval, and keep the JSON-LD
RDF 1.2 feature previously enabled by Oxigraph through `query`. Three exact-output controls
exercise ADJUST, correlated LATERAL and calendar extraction on both query paths;
shared wrong answers cannot pass through parity alone.

The lock removes only unreachable Oxigraph packages and feature-induced edges;
no unrelated package version/checksum changes are accepted. Root runs Cargo
resolution to confirm a fixed point rather than assuming a textual prune is
sufficient. Vocabulary identities and exported alignment quads remain unchanged.

The alignment review finds generic ExampleOf broader than Schema.org's
creative-work instance relation, so a universal mapping is inappropriate.
Summarizes to CiTO describes is plausible but changes exported schema; the current
cites alignment remains during retirement. These are review conclusions, not a
silent new semantic ruling. Definitions: [Schema.org exampleOfWork](https://schema.org/exampleOfWork)
and [CiTO describes](https://sparontologies.github.io/cito/current/cito.html#describes).

Done: meaningful feature negatives turn green after explicit feature retention;
full linked-query/Mere-query suites, feature-free linked check, locked workspace
and both query wasm checks pass; one fresh source-qualified review clears the
retirement. The active materialized parity oracle remains. Record completion in
the [deferred tails](2026-07-03_archived_plan_tails_plan.md) without rewriting
historical test receipts as current evidence.

### C1. Editing from a partial display

The [initial resident host design](../technical_architecture/2026-10-09_resident_host_design.md)
is approved by Mark. The [concrete implementation plan](2026-10-09_resident_host_implementation_plan.md)
is approved and entering implementation. Its first host is the existing owned
Graphshell/Djinn writer; Turnstone authority migration follows separately.

Approved first milestone: stable typed IDs and explicit observation
preconditions go from the resident display to the existing complete-session
writer. Preserve Author, qualification, validation, capture/replay, journal and
undo. Demand aids inspection; it does not prove the edit's dependency completeness.
Keep ResidentGraph immutable, and serialize through the host's established session
owner. Refuse missing/stale targets and unproven independent writer ownership.

Prepare addressable publication separately so existing session pending writes and
publication can share a supported atomic batch. If durability and publication
remain separate, the outcome distinguishes a committed edit from pending/failed
publication; retry republishes without applying the edit twice. Prefer existing
request/change identity and host ownership over introducing another journal.

Mark approved retaining Full writer memory for the first milestone. A writer that never materializes the whole
graph is a distinct larger contract: complete operation dependency declarations,
incremental journal/materialization updates, bounded history reconstruction and
explicit refusal for unsupported/global operations. Neither a partial UI nor a
partial reader is advertised as that writer.

Done: the adopted host's intended edits work from the partial display, reopen and
undo with the same exact truth/history as complete-session edits, refuse invalid
or stale targets, and recover from failed publication without duplicate edits.
State the Full peak and retained-writer costs plainly.

### C2. Host-driven refresh

Keep manual mode; hosts supply time and drive checks from existing foreground,
event/idle and successful-publication boundaries. Cadence and publication batching
are configurable. Add a head-only unchanged fast path, coalesce notices and allow
one refresh in flight. Async candidates carry session/graph, roots, policy and
request tokens; switching invalidates installation. Preserve newer host context
and pending notices arriving during a load.

Done: bursts coalesce; foreground catches missed publications; unchanged heads
avoid catalog/record reads; stale completions do not enter another session; failed
loads preserve the view; source-only and observation-only changes preserve truth
and query caches. Native and actual Firefox IndexedDB drive the same transitions.
Automatic checks do not promise work while a browser tab is suspended.

### C3. Scoped transactions and safe generation collection

Add a default-refusing scoped transaction-read seam over existing backends:
declared exact values and key-only prefixes, with explicit key/value budgets.
Reject undeclared access and overflow rather than exposing a truncated reference
set. Preserve native Send/browser-local bounds and sealed wrapper behavior. Fetch
selected IndexedDB data inside the committing transaction; do not snapshot all
stored blobs merely to acquire a reader reference.

Protect in-flight generation loads transactionally, then release after candidate
installation/refusal. Retain current head, configured previous generations and
all live/explicit references. Reachability includes node records, navigation
entries/visits/owners and retained pending owners. Recheck publication/catalog and
references in the deletion transaction. Delete only residency materializations;
session baselines, journals, history, archives and payload blobs are outside
collection. Unsupported transactions and corrupted preservation inputs refuse.

Collection stays disabled until protected-reader adoption is established at a
safe migration boundary. Conservative nonexpiring crash references may retain
storage until proven exclusive-owner recovery; timeout is not proof of a dead
reader. Budgets constrain selected values and deletion passes without claiming
that thin catalog/reference planning is constant memory.

Done: publish/load/collect races preserve referenced records; shared immutable
records survive; navigation/pending closure survives; corruption and unsupported
capabilities delete nothing; failed/restarted cleanup is idempotent; complete
session history reopens exactly. Exercise actual redb, Directory redo/reopen and
Firefox IndexedDB, including sealed-wrapper and cancellation boundaries.

## Coordination and qualification

RDF retirement and initial family manifest preparation are independent. Host
adoption consumes the settled resident edit/refresh contracts. Collection follows
protected-reader and scoped-transaction adoption. Only one owner changes a shared
source module; root runs Cargo serially to avoid competing target locks. Reuse
qualified caches and retain source-bound logs; do not interrupt other writers.

Each independently shippable phase has its own source-qualified review and
publication checkpoint. Commits are local until Mark approves push/merge. A new
source revision or consumer pin is not called qualified from old receipts.
Library tests, host behavior, remote availability and memory probes are separate
claims, each supported by its own evidence.

## Progress

- **2026-10-09:** Three read-only surveys completed. Isolated consumed-family
  successors created after the app worktree tool refused this non-Git Code task;
  originals remain untouched. 44 Knot, 11 Redshank and 53 Turnstone Mere rows now
  select the qualified P1–P5 publication. Locks/provider hashes/builds remain pending.
- **2026-10-09:** RDF retirement patch prepared. Cargo's first locked gate refused
  the manual lock prune. Narrow unlocked resolution removed two obsolete oxrdf
  feature edges without changing package versions. All three corrected query
  extension controls failed with the features absent, then passed with the final
  explicit feature floor in the full 88-test linked suite. The first LATERAL
  fixture omitted its correlation variable from the subselect projection; that
  fixture was corrected using the SEP scope rule, without evaluator changes.
  All six initial gates passed, including 54 Mere tests and both query wasm
  checks. Fresh review then found that `oxjsonld/rdf-12` also depended on the
  removed Oxigraph edge. Without it, opposite JSON-LD directions collapse before
  classic-reifier matching and can attach false provenance. Import controls
  will reproduce that defect before the feature repair; consumer qualification
  waits for the repaired source candidate.
- **2026-10-09:** Residency design alternatives and dependencies are recorded;
  the first edit milestone's Full-memory boundary is explicit. Host adoption and
  residency changes have not been implemented or qualified yet.

- **2026-10-09:** Fresh fetch confirms the original compatibility commits on
  remote main. The Turnstone successor also pulls documentation/receipt-only
  `e86906a3`, preserving the prepared manifest changes. The consumer runner
  corrects Knot document testing to its excluded standalone workspace and uses
  a dedicated qualification cache, seeded by reflinks without changing the
  original active shared target.

- **2026-10-09:** Fresh-review direction controls all failed without the inherited
  parser feature, including false provenance on mismatched and multivalued
  descriptions. Restoring `query` to enable `oxjsonld/rdf-12` makes all three
  pass. The repaired full linked suite passes 91 tests; remaining gates continue
  under separate receipts, preserving the rejected initial candidate evidence.

- **2026-10-09:** RDF review repair completes at `297682178`. All six locked
  gates pass with unchanged source hashes: linked-data 91, Mere 54, feature-free
  linked check, workspace and both query wasm checks. The three directional
  controls have observed RED/GREEN; no kernel representation or production
  matching code changes. The lock remains the narrow previously reviewed prune.
  The rejected initial `3c9abef1b` receipt and logs remain as evidence.

- **2026-10-09:** All 108 owned Mere rows now select exact supplier
  `2976821783cee6b24d13cacb046c1d99b1890fb4` (Knot 44, Redshank 11, Turnstone 53).
  Parsed manifests differ only in the owned revisions; original checkout
  statuses and compatibility ancestry remain preserved. Provider qualification
  starts with no broad lock-change allowances, in the dedicated cache and using
  a process-only local Git route for the unpublished supplier. That route is
  local object availability, not remote publication evidence.

- **2026-10-09:** Mark approves the written resident-host spec and RDF checkpoint
  publication. Atomic push merges the qualified cleanup and documents into main
  at `a8889f60e`, preserving concurrent documentation rulings and all qualified
  source bytes. Both remote heads are verified. Knot fully qualifies and is
  locally committed at `ab191d4`; Redshank qualification continues. Concrete
  resident-host planning now follows the approved complete-writer milestone.

- **2026-10-09:** Knot `ab191d4d` and Redshank `fea7f0f7` pass provider gates and remain local. Turnstone now selects both provider commits and supplier `297682178`; its default/Piccolo qualification is running. The initial host implementation plan is written, source-checked and indexed for Mark’s review. Host code has not started.
- **2026-10-09:** Mark approves the concrete host plan. Task 1 starts with pure publication preparation and complete catalog observations. Turnstone’s one-file CoveredScene compatibility repair passes seven focused endpoint controls; full default/Piccolo qualification continues independently. Portable wire coverage and actual residency activation remain subsequent host tasks.

- **2026-10-09:** Host Tasks 1–2 complete locally at `a61dfb5c9` and `652acbb72`, with focused tests, full Pandect/residency controls and independent task reviews. Task 3 begins storage staging; reader/product/browser integration remains pending. Turnstone newer-main integration passes 759 default library tests with nine inherited skips and its scoped source review; Piccolo and all-target gates remain active. Consumer and host publication approvals remain separate.

- **2026-10-09:** Turnstone integration completes at `bf1e6a2fb9f0f4c9eba9c866de50a5505bd2404f`, a direct successor of current main `25f85bd`. All 2,968 tracked inputs match the reviewed freeze and completed gate receipts; original adoption commit `767e1613` is preserved. The updated publication request covers Knot `ab191d4d`, Redshank `fea7f0f7` and integrated Turnstone `bf1e6a2f`, including all three main fast-forwards. Provider publication/fresh HTTPS availability remains unproven until approval and execution.

- **2026-10-10:** Mark approves the concrete consumer publication request with “go ahead.” Normal atomic pushes per repository publish the exact qualified topic/main refs: Knot `ab191d4d8808c709cc8ecba093298efa6b37c1e9`, Redshank `fea7f0f797bf8ec8adccf3f842a80c680e0d235c`, Turnstone `bf1e6a2fb9f0f4c9eba9c866de50a5505bd2404f`. Remote refs are confirmed and independent fresh canonical HTTPS fetches match all 376/733/2,968 qualified tracked files. Original provider workspaces, prior Turnstone adoption commit and all qualification receipts remain preserved. Fresh Cargo Git-cache resolution subsequently passes all five metadata gates; no temporary local route is required and every source/lock hash remains unchanged.

- **2026-10-10:** A1 complete: fresh Cargo Git-cache resolution passes Knot workspace/standalone, Redshank and Turnstone default/Piccolo metadata. Canonical published Mere/Knot/Redshank sources are observed in the fresh checkout cache, with all tracked inputs and locks unchanged. The completion binding joins reviewed source, scoped execution gates, approved remote refs and independent availability proof. This does not activate Turnstone residency or qualify headed product behavior.

- **2026-10-10:** Host Task 3 complete locally at `6b2f2860b`: retained owned storage batches and the bounded deterministic navigation snapshot repair pass full scoped qualification and independent review. Task 4 begins settings/pane roots/exact scopes. Actual existing Djinn/browser activation, runtime coverage and whole-host publication remain pending. Consumer A1 and RDF publication receipts remain unchanged.
