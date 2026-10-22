# Foundation family promotion

**Date:** 2026-10-19
**Status:** Source promotion and consumer cutover completed. Execution/shutdown slice qualified on 2026-10-19. A contained observations/action-evidence continuation qualified on 2026-10-20 in the diagnostics plan; broader semantic/collector migrations remain open. Custody process design and source-home follow-up remain scoped. Source extraction, behavioral consolidation and process isolation have separate acceptance gates.
**Receipt budget:** 100 MB total across this migration. Keep compact command summaries and source hashes; do not copy model weights, build trees or historical audit receipts.

## Decision

This section records the completed extraction. The boundary scope below
supersedes its assumption that Djinn must remain in Mere and leaves the eventual
repository grouping open until the dependency cuts are qualified.

Mere remains the graph application platform. Graphshell, Djinn, Cambium, Conatus,
the graph kernel, canvas and application-specific integration stay in Mere.
Promote the reusable foundations without changing their public APIs, crate names,
licenses, stored formats, authority rules or default feature semantics.

The Eidetic repository owns Muniment, Chartulary, Eidetic, Hagiograph,
Stickleback, Luggage, `mere-proofs` and Servitor as separate crates. Storage-only
consumers do not acquire replication, an updater or participant execution.
Stickleback retains its policy-before-insert contract; domain consumers supply
operation meaning and authorization. Luggage retains a portable release identity
core and a separately selected native updater. Servitor evaluates host-supplied
authority, gates attributed graph petitions and reduces run observations; hosts
still execute work and own lifecycle persistence and effect reconciliation.

Armillary gets a separate repository. Its actor boundary, message correlation,
worker pool and wake contract remain host neutral. ESP gets a separate repository
for model execution, with optional Armillary and Eidetic integrations. Gemot gets
a separate repository for the governance/community family, including Mien,
Mooting, Moothold and Commons. The Mere Moot port remains an application adapter.

Errand becomes the continuing home of the Smolweb repository and its thirteen
protocol crates. Import Errand's implementation history into that existing
repository, preserving protocol crate identities and licenses. This move does
not invent a universal transport interface or transfer protocol semantics to
Murm. Reusable carriage factoring into Murm remains consumer-driven development.

## Phases and done-conditions

### 1. Establish sources and extract

Update clean main checkouts before extraction. Retain the source revision and
hashes for every moved file. Preserve filtered Git history in new repositories;
preserve the continuing Smolweb history when importing Errand. Existing dirty
work is outside the migration unless explicitly identified as part of it.

Done when extracted crate source matches the donor, with documented manifest,
documentation and integration-placement exceptions, and each new workspace has
explicit package metadata, dependencies, licenses, toolchain and CI.

### 2. Remove integration back edges

ESP's lexical/BM25 fusion test belongs with Mere's `eidetic-search` integration.
Gemot's capsule-library example composes Mere's native WIT host and belongs with
Mere's Moot port. Preserve both checks rather than making promoted libraries
depend back on Mere merely to exercise the composition. Protocol conformance and
standalone library tests remain with their owning families.

Done when promoted production libraries have no dependency on Mere's aggregate
library or ports, and integration targets have a maintained receiving location.

### 3. Validate and publish suppliers

Run independent focused tests for Armillary, Eidetic, ESP, Gemot and Errand.
Exercise portable core feature configurations where supported. Preserve real
network, large model and headed acceptance as separately scoped checks; this
source-home change does not establish new behavior or erase existing failures.
Publish suppliers and consume one immutable revision per family. Retain the
existing shared dependency source identities and owned p2panda patches.

Done when named independent gates pass and canonical repository URLs and exact
supplier revisions are recorded. No registry release is implied.

### 4. Cut over consumers and retire copies

Update Mere's workspace membership, direct path dependencies and root patches.
Redirect immutable older suppliers that still name moved packages through
`mere.git` to the new canonical family. Update Murm's foundation dependencies,
then other affected sibling manifests without disturbing unrelated work.
Refresh lockfiles and check representative consuming workspaces. Remove the
donor implementation directories only after the receiving source and history
are secured. Do not leave two maintained copies or local-path-only receipts.

Done when no live workspace member or path dependency points to a removed
implementation, selected package graphs resolve a single canonical source,
consumer checks pass, source and documentation changes are committed and pushed,
and all remaining gates and retained build locations are stated.

## Findings

- 2026-10-19: Mere was clean at `24c669d2` and fast-forwarded to `23541321`
  before extraction. Smolweb was clean and synchronized with its remote.
- 2026-10-19: Murm consumes Muniment, Stickleback, `mere-proofs` and Luggage
  through Mere. All four belong in the new foundation cutover.
- 2026-10-19: ESP's only `eidetic-search` dependency is a cross-library test.
  Gemot's native capsule example depends on Mere's app-host and WIT assets.
  These are integration targets rather than production-library dependencies.

## Progress

Supplier extraction and publication completed. All 301 moved Rust source files
match the donor after line-ending normalization and the explicitly recorded
integration-target receiving-package/WIT changes. Source inventory and compact
receipts live in `support/family-promotion/`. The 395 donor files matched their
recorded hashes before retirement. Donor implementation directories have been
removed. All 21 consuming workspace roots resolve and revalidate their lockfiles
with one canonical source for every promoted package they consume.

| Family | Published revision |
|---|---|
| armillary | `6827559937f07f78a6a830dfbfcaa54210dd093c` |
| eidetic | `de6b1dce75dd6725d72c79b829e4878c7903a381` |
| errand | `dce1f08546b8475e42a5a01ca1ea18794c6e9b97` |
| esp | `a2561b0b9766a4ad4d9924c02194bb414667b9ef` |
| gemot | `6a5023a9ca478ebcc342dd9b7c9e418ccacfc6b5` |

Independent gates passed: Armillary unit/doc tests; the full Eidetic workspace;
Luggage's Wasm core; Errand default and all-feature protocol workspaces; ESP's
portable actor/persistence and CPU decoder/model-session configurations; the
Gemot workspace against canonical Git suppliers. External model, GPU, headed
and physical acceptance are unchanged and outside this source-home receipt.

The first ESP test run exposed two decoder tests lacking feature requirements;
the promoted manifest now declares them. A smaller Gemot build exposed an
Errand protocol feature inherited from Mere that the initial extraction omitted;
the corrected manifest retains Finger's explicit client feature. Failed receipts
remain beside the passing follow-ups.

The normal-library graph is the layering contract. Eidetic's existing carrier
integration tests may consume Murm, and Murm's separate Signalman application
workspace may consume Mere UI. Those test/application edges do not make the
Eidetic or Murm production libraries depend on Mere application composition.

## Consumer qualification and limits

The source graph audit covers Mere, Graphshell web, three Distillery roots,
Cleromancy and its core, Knot, mer3ly and repo-graph, Murm native and Signalman,
Turnstone, Woodshed and three application/proof roots, and four Isocosm roots.
The exact roots and immutable package sources are in
`support/family-promotion/consumer-source-validation.json`.

Executed tests include 539 Eidetic, 10 Armillary, 334 default-feature and 416
all-feature Errand, 105 portable and 198 decoder ESP, 242 Gemot, 762 Mere
kernel/Seiche/Graphshell-client, 17 Cleromancy core and two relocated search
tests. These configurations overlap; their counts are not a unique-test total.
Ignored tests remain recorded in `verification-summary.json`.

Mere's platform library, Wasm Graphshell, Luggage's browser core, Woodshed's
core/graph, Knot document and Isocosm domain compile. The relocated capsule
example and its WASI guest also compile; this does not claim native/WIT/browser
end-to-end execution. The lexical integration test stays with `eidetic-search`;
the capsule application composition stays with Mere's Moot port.

The unfiltered Murm run hung on
`a_killed_peer_off_the_overlay_reads_not_connected_when_its_connection_closes`.
The follow-up executed 169 tests with that single shutdown case explicitly
filtered and one child fixture ignored. Both receipts are retained. This gate
remains open; the source move does not claim to repair transport shutdown.

Mere's whole-tree license-header audit still reports 658 missing Exhibit A
headers. Comparing the original donor revision with the final tracked sources
finds the same 658 and zero newly missing headers. Existing licensing grants and
third-party notices travelled with Eidetic and Errand. No registry release or
license-policy change is part of this migration.

Active Eidetic, Armillary and Gemot documents followed their source owners.
Mere retains its application plans and historical checkpoints with owner links.
Transport factoring into Murm remains a separate follow-up. Broad ownership
migrations inside the retained graph runtime are not declared complete here.

All Cargo checks reused `C:\t\cargo-targets\mere`; no isolated Cargo home was
created. An Isocosm worktree was necessary to publish the adoption without its
unrelated unpushed source commit. The adoption was integrated back into the
primary checkout, its original product paths were verified unchanged, and the
worktree was removed. The unrelated product commit remains unpublished. Retained compact receipts, including ignored
local preservation backups, remain below the 100 MB budget.

## Publication

All five supplier repositories are published. Their tested source cohort is
listed above; later documentation-only commits do not change those pins. Seven
consumer adoptions are pushed, with exact commits in
`support/family-promotion/consumer-publication.json`. This Mere commit retires the
donor implementations and records the canonical source graph and integration moves.
The local Errand checkout remains named `repos/smolweb`; the GitHub repository is
now `merely-made/errand`, retaining the existing repository identity and history.

The document-judgment check covers this new plan and recognizes moved documents.
It still fails on the original 19 missing records and an unchanged snapshot
whose checkout digest differs from the checker constant. The baseline comparison
records zero newly missing documents; the historic snapshot was not edited.

## Boundary consolidation scope, 2026-10-19

The owner clarified that names and umbrella repositories are secondary. The
objective is one implementation per shared responsibility, with explicit
ownership of state, permissions, execution, lifetime and reported outcomes.
The scope below defines separate acceptance gates. Execution/shutdown progress
and qualification are recorded at the end; the diagnostics plan records the
2026-10-20 contained observations/action-evidence continuation and remaining
owner migrations. Custody/process and source-home lanes remain scoped. The
completed source promotion above remains its own checkpoint.

### Responsibility boundaries

| Responsibility | Owns | Boundary to preserve |
|---|---|---|
| Execution | Actor/thread completion, typed messaging, task scopes, cancellation and shutdown mechanics | The caller supplies resource order, policy, deadlines and domain meaning. Optional Tokio support must not become mandatory for synchronous consumers. |
| Service supervision | Enabled services, configuration, process/service lifetime, restart policy and dependency order | A supervisor composes domain owners; it does not duplicate their journals, job grammar, authority or retention decisions. |
| Domain services | Compute/model work, publication, synchronization and their outcomes | Distillery retains its own jobs, leases and scheduling. A job scheduler and a task executor have different responsibilities. |
| Secret custody | Vault authority, root signing, credential release, scoped derivation, lock and revocation | General workers and diagnostics receive only the operations or material their admitted contract requires. Current mixed derived-key release remains unchanged pending custody design. |
| Observability | Shared observation envelope, bounded retention, causal references, independent readers, explicit loss and inspection data | Producers own redaction and outcome truth. Ingress buffering and retained history keep their different lifetimes and admission policies. |
| Application semantics and actions | Domain identities, semantic projections, retained geometry, permitted actions and actual action results | Accessibility, automation and inspectors consume the owner's projection and admission path. Observing an operation does not authorize controlling it. |

Existing homes are implementation starting points: Armillary for execution,
Djinn for supervision, Distillery for compute, Castellan for custody, Apparatus
for observations, and Genet/Cambium/Mere adapters for application semantics.
Repository membership is not an authority boundary. No rename or source-home
move is required to finish the first behavioral slice.

### First implementation slice: execution and truthful shutdown

Work in the existing Armillary repository and Mere consumer workspace. Give
Armillary the generic scoped-task implementation currently at
`crates/graphshell/graphshell-endpoint/src/door/tasks.rs` and the ordered-close
mechanics currently at `ports/distillery/src/lifecycle.rs`. Keep them modules
within the existing crate, with Tokio support optional. Update Graphshell doors,
Djinn and Distillery to use the shared implementation. Each host continues to
supply its own resource names, close sequence and domain-specific cancellation.
Do not introduce a general service scheduler in this slice.

Make actor completion and failure observable for dedicated and pooled actors;
channel closure alone does not establish completion. Propagate relevant failures
through the consuming close path. Review scoped-task reaping as well: the current
implementation discards join results while reaping and while cancelling. A drain,
a cooperative stop request, a completed join, a panic, and forced future
cancellation must retain their actual meanings. Do not invent successful terminal
work records from a requested stop or an empty inbox.

The existing task scope can be drained and reused. Before changing that contract,
list the callers needing reuse and specify a separate terminal-close operation
if required. Closed scopes must have defined admission behavior under concurrent
spawn. Never claim that a timeout or dropped handle stopped a Rust thread.

Required acceptance:

1. One maintained task-scope implementation and one ordered-close implementation;
   old call sites are migrated rather than left with independent copies.
2. Dedicated and pooled actor completion are distinguishable from panic and
   disconnection. A blocked actor cannot be reported joined before it exits.
3. Nested tracked tasks release held resources before a successful close report;
   the untracked/detached control remains explicitly outside that guarantee.
4. An early close error does not prevent later resources receiving their close
   attempt, and all observed errors remain available to the owner.
5. Focused Graphshell/Djinn task tests, Distillery resident/close tests and an
   Armillary-only consumer pass against canonical Git sources. The synchronous
   Armillary feature graph remains free of Tokio and application/custody crates.
6. Selected failure controls fail on the old behavior and pass after repair.
   Exact commands, source pins, ignored cases and unresolved baseline failures
   are recorded. Heavy models, headed rendering and physical-device acceptance
   are required only if the implementation changes those behaviors.

Pool admission and worker retirement are separate named repairs: Q112's idle
reservation can strand queued actors, and Q113's workers survive all public pool
handles. Source repair and standalone Git receiving are qualified in the
[pool continuation below](#pool-admission-and-worker-retirement-2026-10-21);
the product dependency cohort remains a separate intake. Fix either when it is a prerequisite for this slice's acceptance;
otherwise retain it as an open pool gate. Do not claim bounded worker lifetime
from an actor-completion repair. The existing Q217 inference-disconnection
counterexamples are receiving gates: exposing a join error is insufficient if
the lane still journals or acknowledges work after its actor has disconnected.

### Second implementation slice: observations and semantic action evidence

Current-target/collector continuation (2026-10-20, qualified): the
[diagnostics plan](2026-06-08_system_diagnostics_and_accessibility_plan.md#current-target-admission-and-collector-bridge-2026-10-20)
records fresh host admission and explicit routing results, plus the opt-in
Registry-to-Apparatus bridge: 237 targeted native tests pass with seven existing
ignores, plus browser-host and collector Wasm compilation. Product startup
wiring, target lifetime tokens and
shared semantic construction remain open.

Shared DOM semantics/product continuation (2026-10-20, qualified):
Winit consumes Rootstock's DOM projection, and Djinn's lifecycle source wires
an opt-in product-projected collector into its admitted status reader. 191
targeted native passes/seven existing ignores and browser-host Wasm compilation
qualify the bounded supplier and product paths, including a real CLI client
process against the admitted fixture broker. Shared
leaf/producer semantics, target lifetimes and process-final evidence were
separately tracked for that historical cohort.

Shared leaf/producer continuation (2026-10-20, qualified): native,
browser and admission now consume one Rootstock host composition over the
existing AccessKit-shaped provider contracts. The
[diagnostics plan](2026-06-08_system_diagnostics_and_accessibility_plan.md#shared-leaf-and-producer-composition-2026-10-20)
tracks repairs to numeric leaf lowering, producer IDs/action maps, restrictive
state, hidden focus, duplicate producer DOM bindings and missing/invalid geometry.
Seventy affected native receiving tests pass on integrated main, with four
existing ignores, plus browser-host Wasm compilation. The earlier broad cohort
has 190 passes/seven existing ignores on its preserved pre-repin basis. Neutral contributor contracts, mutable
read/act boundaries, target lifetimes, broader Hest action routing and
process-final diagnostic evidence remain open.

Flattened producer discovery (2026-10-20, qualified): GPU preparation, portable
capture, leaf size reads and semantic routing share Genet's flattened child
selection through one Rootstock traversal. Shadow and assigned-slot producers
now reach rendering/capture; masked fallback and unassigned light nodes cannot
become competing owners. The
[diagnostics plan](2026-06-08_system_diagnostics_and_accessibility_plan.md#flattened-producer-discovery-2026-10-20)
records 112 targeted native passes/five existing ignores and browser-host Wasm
compilation on source-frozen main `24b079538`, including sampled headless GPU
pixels, fallback rebinding, retirement and native/browser action mapping.
Full producer coordinate mapping, read-only semantic observations, target
lifetimes and broader action routing remain separate migrations.

Producer content-origin repair (2026-10-20, qualified): generated children and
their action bounds now use Genet's content-box origin in the existing
untransformed, scroll-adjusted observation space. This removes the border and
padding displacement from rendered producer pixels. On integrated main
`e94cb23e8`, 115 targeted native tests pass with five existing ignores, plus
browser-host Wasm compilation. Sampled headless pixels, both scroll planes,
native action bounds and scaled browser boxes receive the repair. The
[diagnostics plan](2026-06-08_system_diagnostics_and_accessibility_plan.md#producer-content-origin-2026-10-20)
keeps full forward transform/clip and flattened-ancestor mapping with the
supplier; no duplicate CSS placement implementation is introduced in Mere.

Producer semantic identity validation (2026-10-21, qualified): one shared
preflight refuses duplicate child keys, per-child action IDs and platform-ID
collisions before mutating any contribution or route. Native/browser publication
and current-target admission receive the same policy. Frozen main `8942e3050`
passes 122 targeted native tests/five existing ignores and browser-host Wasm
compilation. The
[diagnostics plan](2026-06-08_system_diagnostics_and_accessibility_plan.md#producer-semantic-identities-2026-10-21)
keeps immutable observations, target generations and report/collector adoption
open; the stable ID hash is not a generation token or collision-free allocator.

Read-only observation and selector continuation (2026-10-21): provider callbacks
and composition use shared borrows; the content owner allocates retained keys.
Taproot's borrowed lookup now receives leaf/producer contributions in the Harness
and Mesquite, with explicit generated target identity and host-written routing
acknowledgements. The browser and selector role-spelling duplication is retired
through an optional contract bridge without adding a renderer dependency to
Taproot. Exact qualification and remaining acceptance are recorded in the
[shared selector receiving](2026-06-08_system_diagnostics_and_accessibility_plan.md#shared-selector-receiving-2026-10-21)
section. Registry publication and other product adoption remain separate. Target
lifetimes are next, followed by broader actions and supplier forward geometry.

Continue the existing
[diagnostics/accessibility plan](2026-06-08_system_diagnostics_and_accessibility_plan.md),
which remains the owner of its detailed implementation and acceptance. Reuse
Apparatus's implemented bounded store and Mesquite attachment. Inventory the
registry ingress envelope/correlation and Apparatus metadata, then share only
contracts that represent the same fact. Keep the registry's nonblocking ingress
separate from Apparatus's independent-reader retained history. Preserve each
layer's configured count/byte/age limits, rejection policy and loss accounting.
No universal event enum, global semantic revision or new diagnostics daemon.

The selected worker/UI evidence paths have already been qualified in the
[observations continuation](2026-06-08_system_diagnostics_and_accessibility_plan.md#observations-and-semantic-action-continuation-2026-10-20):
Distillery records real actor replies and journal refusals, and the retained-host
wheel fixture records its application callback. Continue from those paths and
the later DOM/collector repairs rather than repeat their initial implementation.
Their CPU receipts do not establish headed product completion or pixel causality.

**Command-contract owner:** the
[Hest plan](../../cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md)
owns shared declaration, invocation and outcome vocabulary. H1–H4 now have
their own published delivery/receiving cohorts, including H3's nonterminal
acceptance and H4's script/automation adapters. This diagnostics continuation consumes
that owner instead of defining another command catalogue or outcome protocol.
Keep Hest independent of Chirograph, Apparatus and application runtimes.

Hosts and Chirograph retain authorization, confirmation, current-target and
session/document/revision lifetime validation, dispatch and execution. Hest's
effect/destructive declarations describe commands; they do not grant permission.
An accepted invocation or handled input establishes admission/dispatch only.
A terminal completion requires the executing owner's actual reply; the current
Chirograph `IntentResult::Accepted` must not convert to terminal `Done`.
Apparatus records a redacted copy of those facts using producer-supplied
operation/source scope, with its existing independent readers and loss bounds.

The remaining action migration must carry that distinction through the Hest
owner's two-host and Chirograph round-trip gates, then through a selected current
target's automation/platform adapters. Replaced or expired targets must refuse;
unsupported, stale, ambiguous, cancelled and failed operations must retain their
actual results. Taproot and Mesquite consume the host's retained projection and
geometry. UxTree's Inker-specific projections and ux-events' browser vocabulary
remain adapters; generic users must not inherit them. Diagnostics and local
automation must continue to work without Djinn running.

Done when the two paths produce bounded, redacted evidence with independent
inspection/test readers; unsupported, stale, ambiguous, cancelled and failed
operations retain truthful results; overload/restart gaps stay visible; and
retry/cancel controls traverse the original owner's admitted command path.
Presentation stamps retain their qualified meaning of queue submission, with
separate run/host scope. Exact image/state correlation and human screen-reader
acceptance remain the existing plan's separate gates.

### Separate design pass: custody process boundary

The intended direction is a narrow Castellan custody process contacted by the
resident. The present code links Castellan into Djinn; separate crates and
cargo-deny wrappers do not establish address-space isolation. This design pass
must produce a reviewable protocol and migration, not a second vault or a
parallel secret authority. Process separation is not implemented by this scope.

Specify caller authentication and authorization independently: OS-user identity
alone does not identify the requesting application or authorize key use. Cover
app/profile/session/key scope, consent, unlock and presence UI, sticky lock and
revocation, permitted derived-key lifetime, parent/client death, restart and
reconnection, replay/idempotency, broker ownership and existing SSH/Secret
Service behavior. Include the exact policy for calls already in flight during
lock; preserve the existing absence/Locked refusal and prohibition on fallback
keys. Root material must not cross into the general resident's workers.

The design must identify Djinn's direct vault openings and adapters, including
`resident_distillery.rs`, and how each becomes a custody operation or a narrowly
permitted derived-key consumer. Start with a cross-process fixture for one root
signing operation and one scoped derived-key consumer, using actual authenticated
IPC. Wrong-app/profile requests, broker loss, lock during use, crash/restart and
revocation need negative controls. Existing source-formats and credential
identity stay stable. Packaging, launch policy and per-OS process restrictions
require explicit acceptance; IPC by itself does not prove complete isolation.
A trusted resident supervisor's ability to launch or replace the custodian also
belongs in the stated threat boundary.

### Source-home follow-up and exclusions

After the functional seams are qualified, promote the resident and its custody
implementation and choose a workspace grouping based on the actual dependency
graph. Separate public crates can share a repository without acquiring each
other's runtime dependencies. Preserve history, notices and immutable consumer
pins as in the completed promotion; qualify one external foundation consumer and
one real resident consumer. No umbrella name, Armillary relocation or mandatory
Apparatus promotion is a prerequisite for the first slice.

This continuation excludes a universal transport rewrite, a replacement async
runtime, new product panes, universal automation protocol, model/backend work,
new capability algebra, a vault-format migration and new restart/retry defaults.
Keep Mere's graph ownership migration and wider protocol/admission consolidation
in their existing plans. Respect the original instruction to document hairy,
high-impact issues that would consume the bounded audit pass; retain them as
named follow-ups with evidence instead of expanding this scope silently.

### Source findings and progress

- 2026-10-19: Read Mere `883ccdc4`, Armillary
  `def985a79f40c255af5c771e37b754b47f73a813`, and the named Genet/Taproot seam
  at local Genet `2921c95bbbc50841fe35ae993e89e03f7197fb46`. This is a local
  source review, not a fresh upstream or runtime qualification.
- 2026-10-19: `ActorHandle::join` discards dedicated-thread panic results;
  `spawn_named_on` returns a handle without a completion join. The actor-span
  comment claims a finish on panic, while the explicit finish event follows the
  run call and is skipped by unwinding. These are source confirmations; this
  scoping pass did not execute new reproductions.
- 2026-10-19: Reuse the prior
  [audit findings register](../../../support/receipts/authority-success-audit/review_notes.json):
  Q112 (pool reservation), Q113 (pool retirement), Q217 (inference actor loss),
  and its existing pooled-join/panic-finish follow-up. Their historical source
  paths predate promotion. Their statuses are not changed by this scope.
- 2026-10-19: Registry `diagnostics/emit.rs` already describes its queue as
  nonblocking ingress and explicitly delegates independent retained readers to
  Apparatus. Their similar limits are not evidence that one queue should replace
  both. Mesquite already uses Apparatus batches and retained accessibility
  projection through Taproot; this is an integration continuation.
- 2026-10-19: Scope recorded; no runtime code, custody process, dependency pins
  or source homes changed. First delivery is the execution/shutdown slice above.

Receipt budget: 100 MB total retained evidence for this continuation across all
repositories. Each implementation brief names its subset and cap; keep source
hashes, compact outcomes and required failure controls. Reuse the approved shared
Cargo target, preserve unrelated work, and use a worktree only for a real
collision. This scoping pass needs documentation checks only and creates no build
output or isolated toolchain home.


### Execution slice progress, 2026-10-19

- Armillary published `5d8754dc2a00be02d82b075441ac9ea44aa4f135`: shared
  ordered-close mechanics, optional Tokio task scopes, observable dedicated and
  pooled actor completion, distinct panic traces, reusable draining and terminal
  admission fencing. Cancelling a drain future leaves ownership/results in the
  scope. A pooled-join regression fails on the former source and passes after
  repair. Eighteen focused unit/integration/doc tests pass with Tokio; fourteen
  pass without the feature, and normal synchronous dependencies contain no Tokio.
- ESP published `ddb5037bae30bd89f9e558c7dd5c9065eedfb162`, incorporating the
  concurrent repaired-Eidetic source adoption. Its actor/persistence receiving
  configuration passes 105 tests with one existing ignore.
- Mere refreshed onto `d5c26e08` before receiving qualification; unrelated
  renderer, Djot, keyboard, standards and comms work is preserved. Its two former
  generic implementations are removed, and callers use Armillary directly.
  Receiving changes fold inference actor loss into failed run records, refuse
  late commands, preserve unrequested retained user pauses and collect close
  failures while attempting both actor and bridge joins. Native receiving gates
  pass on the final supplier cohort recorded below.
- The upstream ESP change requires its repaired Eidetic cohort across actual
  storage-sharing suppliers. Source-only cohort alignment is part of integration;
  the historical promotion receipts and pins are retained rather than rewritten.

New compact command receipts use the `execution-` prefix under
`support/family-promotion/`; they belong to this continuation's 100 MB total
budget. Pool Q112/Q113, custody process separation and later observation/action
work remain separately scoped. No success claim for those gates is implied.


### Execution/shutdown qualification, 2026-10-19

The first implementation slice is qualified. The final immutable execution
supplier is Armillary `e67c5def1c735be047492a867bcdc8b41fba22b8`, and ESP
`b20ae7db257dae83493031efd92da3a102c44a48` adopts it. The later Armillary
refinements preserve the existing handle's `Sync` property and retain cumulative
terminal task failures across repeated/concurrent closes. Reusable drains still
consume their own reports; they do not reopen a terminally closed scope.

The old task and ordered-close modules are removed from Graphshell and Distillery.
Callers import Armillary directly, with no parallel implementation or forwarding
module left behind. Host-specific close order and domain cancellation remain
with their owners. Djinn's real resident and embedded-host close paths surface
tracked-task failures while continuing resource cleanup. The receiving tests
also assert clean fixture teardown rather than discard its report.

Q217's two public counterexamples are repaired in the bounded receiving lane:
actor update EOF is processed only after draining already emitted updates;
unfinished active work becomes Failed and is written to its journal, or a journal
refusal is surfaced. Late ask/cancel/pause/resume commands refuse a disconnected
actor. An already-disconnected actor cannot append a queued continuation to an
unchanged retained user pause. Close attempts both joins, reports their failures,
and preserves the result for repeated/concurrent callers. These are native
synthetic-provider/journal receipts, not large-model or GPU qualification.

| Final gate | Observed result |
|---|---|
| Armillary with optional Tokio | 19 unit/integration/doc tests pass, including interrupted close, repeated failure reporting, pooled cleanup and panic outcomes |
| ESP actor/persistence receiver | 105 tests pass; one existing scale case ignored |
| Distillery inference library, actor-loss receiver and resident teardown | 40 tests pass |
| Graphshell endpoint with door | 35 tests pass after upstream custody integration |
| Djinn resident-task integration | Two tests pass, including retained panic reporting and the detached-work control |
| Fleece actor API adoption | One selected no-network-acquisition test passes; 36 unrelated tests filtered |
| Independent synchronous Git consumer | Runs against the final Armillary pin; normal dependency tree has no Tokio |
| Graphshell browser target | Wasm compile check passes; browser runtime and human acceptance are not claimed |
| Canonical Mere source graph | 22 consumed promoted packages each resolve one expected source; root registry package versions unchanged |
| Affected standalone lockfiles | Eight roots resolve and revalidate with `--locked`; this is not runtime qualification of every probe |

Compact final command receipts are the `execution-*-qualified.json` files and
`execution-armillary-final.json`, `execution-esp-final.json`,
`execution-fleece-adoption.json`, `execution-browser-compile.json`,
`execution-standalone-locks-qualified.json` and
`execution-source-graph-qualified.json` under `support/family-promotion/`.
Earlier passes and the failing pre-repair pooled-join control remain alongside
them. The independent consumer is `support/fixtures/armillary-consumer`.

Supplier alignment preserved the concurrently published Eidetic repair at
`543f5ff38f87a09afc04dbc59f57c2dfbc6a9943` and Gemot's adoption at
`67ce0ac3e2b48be951a677a9eca245eaabd56d63`. Murm's native source-only adoption is
published at `da453213ccce1768783ffd77e47e1bce18e80550`; its native graph passes
locked resolution and its code is compiled/used by the receiving resident gates.
No unfiltered Murm transport-suite or physical-radio pass is claimed.

**Deferred standalone renderer root:** `crates/cambium/examples/genet_web_smoke`
needed a moving Genet/renderer source refresh while resolving its historical
standalone graph. The offline failure is retained, and the later online probe
was stopped to keep this unrelated renderer work outside the execution slice.
This slice leaves that manifest untouched; later upstream supplier-cohort edits
remain preserved. See
`execution-standalone-smoke-deferred.json`; the fresh Cargo-owned partial Genet
checkout is retained, not deleted. The maintained browser target passes its own
separate compile gate. This is not a full renderer-prototype adoption receipt.

Q112/Q113 pool admission and worker retirement remain open. This work does not
establish bounded pool-worker lifetime. The previously recorded full-tree
license-header and document-judgment debt is not repaired by this slice.
Observability/action evidence, custody process separation and repository grouping
retain their scoped gates. All builds reused `C:\t\cargo-targets\mere`; no
isolated Cargo home, target or worktree was created for this implementation.


Final upstream integration (2026-10-19): rebased the receiving change onto Mere
`48fd3385` and preserved the concurrent coherent-supplier adoption, deterministic
Mesh receipts and `e122a071` custodian-bound payload sealing. Upstream manifest
policy, including Eidetic's disabled default features, remains intact; execution
suppliers stay pinned to the tested Armillary/ESP cohort above. The combined
Graphshell door gate passes 35 tests. Djinn passes both task tests and the admitted
custody/profile-rotation case; its separate child receipt-host fixture remains
ignored by the parent test runner. Distillery again passes all 40 selected tests,
and the maintained browser target again compiles. The integrated source graph
still contains 22 canonical promoted packages and all eight selected standalone
locks revalidate. `execution-*-integrated.json` and
`execution-integration-basis.json` retain this later receiving evidence without
overwriting the earlier qualification receipts.

Publishing integration also preserves `4d60510c`, the concurrent Cambium
publishing-version changes. After aligning the three local package-version rows,
Cargo regenerated the root lock and validated the full offline locked graph;
the 22 promoted supplier sources remain unchanged. See
`execution-publish-lock-qualified.json`. The earlier unseeded resolution was
stopped after sustained resolver work, and a seeded locked attempt correctly
refused the still-incomplete lock; both preparation receipts are retained.

The initial publishing graph exposed another unfinished consumer migration:
Knot's older Cambium/Illume requirements selected parallel Git and local
versions. The resulting Djinn compile failure is retained in
`execution-djinn-publish.json`; successful metadata alone did not establish
receiving compatibility. Adopted Knot's published GUI requirement correction,
then aligned its ESP intake with this slice's tested supplier in Knot
`9832691735ed63a1bbb5f65dc0f5857e6f121dbc`. Cargo regenerated and validated the
full locked graph with one local Cambium, one local Illume and the same 22
canonical promoted supplier sources. See `execution-knot-cohort-qualified.json`.
The earlier failed source comparison also remains retained. This source-only
Knot adoption is qualified through Mere's receiving gates, not an unfiltered
standalone Knot suite. Later receiving reruns have separate
`execution-*-publish-aligned.json` receipts.


### Repaired supplier receiving checkpoints on macOS

The initial promotion table above is historical. The repaired receiving cohort is
Eidetic `543f5ff3`, Gemot `67ce0ac3`, ESP `af5597c6` and Armillary `68275599`.
Mere's execution receiver advances the last two to the separately qualified
ESP `b20ae7db` and Armillary `e67c5def`; consumers without that execution seam
retain their qualified pins. Source identity is checked within each actual
consuming graph rather than inferred from declarations or unused patches.

| Consumer | Published receiving checkpoint | Receiving evidence |
|---|---|---|
| Cleromancy | `230347a9` | Personal-sync all-targets check; 17 core tests |
| Knot | `c6a442db` | Root library check; locked root, document and site graphs |
| Murm | `f9fe8507` | Six session-lane tests; Signalman app check |
| Isocosm | `028a5ef1` | Seven witnessed-lifecycle tests; five locked roots |
| merelyllc.com | `9b6bd436` | Site check; one selected repo-graph test; both locked roots |
| Turnstone | `01658c73` | Published manifest and lock checked; native evidence remains with its owner |
| Woodshed | `f1f2df08` | 331 core/graph tests; Hocket, Redshank and recipe checks; four locked roots |

Required SDK intake preserves the concurrent Cambium 0.4 requirement. Mere
`48fd3385` repairs the promoted family source identities; consumers with the new
GUI bounds use published SDK `4d60510c`. Cargo-generated lock closure changes
are retained in their receipts. The only receiving adapter changes are removal
of three obsolete semantic `decay_progress` initializers in Cleromancy and
explicit anonymous management-geometry fields in Signalman. The latter does not
invent source-link identity or link-opening authority from a management edge.

Knot's concurrent ESP alignment is preserved: its root resolves ESP `b20ae7db`
and Armillary `68275599`; the nested document/site supplier patches are unused
in their actual graphs. The root lock closure was regenerated and checked after
the alignment. This intake does not include the separate Knot checkpoint repair.

The macOS receiving extension preserves Mere `8f26d962` and its direct Armillary
imports, deleted local task/close modules, browser qualification and upstream
receipts. It adds one retained collection of journal failures to the existing
run table. Lock and close report every observed journal refusal; an explicit
host lock remains authoritative after actor loss. Failed resume admission
restores the retained pause instead of leaving a queued continuation. Buffered
completion, actor panic and repeated close results have deterministic receiving
controls. The dedicated/pooled controls extend the existing
`support/fixtures/armillary-consumer` host rather than add another fixture.

Final macOS checks pass: 41 Distillery library tests plus four shutdown/close/
resident cases; nine Djinn receiving tests with its binary target compiled;
43 Graphshell endpoint tests; four independent synchronous actor controls; eight
locked metadata roots; and the port boundary check. Both selected before-repair
controls fail at their intended assertions, including the durable-lock false
success on `8f26d962`. The stable final Distillery/Djinn reruns start from a clean
receiving checkpoint. The synchronous fixture's normal dependency graph excludes
Tokio. Counts describe their individual configurations, not a combined unique
suite or new native UI qualification.

Compact records for this receiving extension and the consumer publications are
`support/family-promotion/consumer-repaired-cohort-publication.json` and
`support/family-promotion/execution-macos-receiving.json`. They retain source
hashes, exact commands, exit results and the failed pre-repair controls. These
are CPU/source checks; native UI, physical audio/radio, heavy models and the
Knot checkpoint lane keep their own acceptance records. Q112/Q113 pool admission
and worker lifetime remain open.


### Embedding compatibility receiving cohort (2026-10-20)

The resident I10 receiving build exposed parallel exact-revision Eidetic and
Chartulary types through older Knot, Cleromancy, Gemot and Murm manifests. A
root-only pin change could resolve metadata but could not compile the actual
consumer interfaces. The retained schema is Eidetic `34105d63`; the guarded
ESP implementation is `6d371901`. The manifest intakes were published first,
then Mere `fb56a2ae1` supplied the receiving SDK, followed by standalone lock
refreshes. No local Cargo overrides remain in the qualified graph.

The canonical receipt is
`support/receipts/resident-embedding-20261020/qualification.json`; its
`qualified_git_graph` records exact tested inputs and
`supplier_lock_receiving_heads` records the subsequent lock closure. Six roots
(Knot, Gemot, Cleromancy root/core, Murm native/Signalman) pass locked metadata
with one Eidetic source per package. Those checks establish source resolution;
the compiled behavioral qualification is the resident I10 gate. They do not
assert unfiltered standalone application suites. Later manifest/lock commits
carry the same code and SDK/artifact revisions as the qualified receiving
closure. Frozen earlier promotion and native receipts remain unchanged.


### Apparatus portable-core extraction, 2026-10-21

The portable bounded observation store and renderer-independent inspection data
now have one implementation in `apparatus-core`, a sibling crate in Armillary
at `75643df1f178757bb7b6386913af1a39353f3069`. Its default library uses only
`std`; optional Serde is its only library dependency. It does not depend on
Armillary's actor package, Tokio, Mere, Registry, AccessKit or UxTree.

Mere retains `mere-apparatus` as its integration facade: existing public types
are re-exported, the `serde` feature forwards to the shared core, and the
Registry collector, structural accessibility projection and default projection
behavior remain in Mere. The old store and portable inspection implementations
are removed from Mere. Hest vocabulary, command admission/execution, producer
redaction, retention/export decisions and persistence authority are unchanged.
Armillary's existing execution package pin is independent and stays qualified;
adding this sibling crate does not require an actor-source migration.

The donor is Mere `8942e30503f2fa91fdefe879ec5268ee42a5241a`. Source hashes
prove the observation implementation/tests were moved verbatim; inspection
removes only its Mere projection blocks and the resulting blank seam. All 18
portable tests move with their implementation. Receiving proof checks shared
Rust type identity and the original serialized batch/cursor values, including
Unicode payload bytes, eviction and scoped cause metadata. The independent
Git consumer exercises the default core without the Mere facade or a runtime.

Compact source, command and publication evidence belongs in
`support/family-promotion/apparatus-core-extraction.json`. These source/CPU
checks do not establish new native UI, physical-device or durable audit-log
qualification. The shared core remains a bounded observation copy; products
retain authoritative execution results and persistence.


Apparatus extraction receiving qualification is complete. Armillary's default
and Serde configurations pass the 18 transferred core tests on Rust 1.97.1;
its existing 19 execution tests also pass. Mere passes ten retained projection/
collector tests plus two public-type/wire controls, Distillery's 42 library and
five observation/shutdown tests, Djinn's four resident-event tests, and
Mesquite's 39 library tests. The standalone Git consumer runs on Rust 1.97.1
with no normal dependency beyond `apparatus-core`. Four selected locked roots
revalidate; the optional Distillery probe does not consume the core in its
default resolved graph.

The supplier core is pushed at Armillary `75643df1`. Mere's integration is
pushed at `7edfe3b0`, preserving concurrent producer-identity refusal and native
co-op/save-recovery work. The final facade rerun passes on that integrated
source. Turnstone's actual diagnostics, redaction and projection paths pass 16
focused tests against the canonical Git supplier, and its receiving lock/receipt
are pushed at `fc7a3e49`. Its product sources and dependency declarations are
unchanged. Cargo also reselected an already-present Windows dependency in 16
lock entries; this is recorded in the receiving receipt, with no unrelated
package-version or source changes. The qualified code pins remain fixed as
later documentation-only records are published; no recursive repin is needed.


### Pool admission and worker retirement, 2026-10-21

Q112 and Q113 are repaired in Armillary
`92d5b15cf39aaac40a3f2d863ffee69c9be11bfc`. Queue demand now reserves waiting
and starting capacity under one mutex. Two submissions made before a notified
worker reacquires that mutex cannot both depend on its one unit of capacity.
Public pool handles own a separate lifetime from the workers' execution state.
The last handle wakes all waiters; admitted jobs finish and workers then retire.

Pool drop remains nonblocking. It does not cancel jobs or join OS threads.
An actor's existing completion handle still owns the completion observation;
a job that does not return, or retains a pool clone, can keep workers alive.
`workers()` counts threads ever spawned and is not a shutdown-completion signal.
No async runtime dependency or host shutdown policy was added.

The source controls fail at their intended assertions before repair: a second
queued job cannot start while the first is held, and dropping the last pool
handle does not produce worker thread-local teardown. After repair, tests also
cover all idle workers releasing shared state, sequential reuse, concurrent
jobs, immediate drop with admitted work, nonblocking drop with a live actor,
and final-handle drop from inside a worker. Supplier default coverage passes
38 tests; all features pass 43 on Rust 1.99.1 and 1.97.1, including the unchanged
18 Apparatus tests. An independent Intel Mac checkout of the published revision
also passes all 43 tests on Rust 1.99.1. The supplier receipt records existing broad strict-Clippy
findings separately from passing focused pool lint/format checks.

The existing `support/fixtures/armillary-consumer` receives that canonical Git
revision, with no local source patch. Its five tests pass on Rust 1.99.1 and
1.97.1, including actor completion after pool drop followed by actual worker
thread-local teardown. Its executable, strict Clippy and formatting pass; the
normal dependency graph remains Tokio-free. This new public consumer control
also fails against the old `e67c5def` supplier. Exact commands, source hashes and
receipt/log hashes are in `support/family-promotion/pool-lifetime-20261021.json`.

The product root remains on its previously qualified Armillary pin. Its current
source uses dedicated actors; the standalone fixture is the current pool caller.
A root-only repin would duplicate Armillary because ESP still directly pins the
old revision. Knot also directly pins that ESP revision, so a clean product
intake must proceed through ESP, Knot and then the Mere/product dependency
cohort together. This continuation does not claim that broader intake, fresh
native UI acceptance, or changes to the separate Knot checkpoint feature.


### Coordinated pool consumer rollout, 2026-10-22

The authorized receiving continuation carries Armillary `92d5b15` through ESP,
Knot, Mere and the product consumers. ESP `6d6ad84` adopts the repaired execution
source without engine changes. Its default workspace passes 94 tests with one
existing measurement ignored, and its actor library passes 88 tests on both
Rust 1.99.1 and 1.97.1. Isocosm's narrow intake is published at `3e365400`: its
three actual session-bridge cases pass with one repaired Armillary source while
its independent fixed Mere cohort stays intact.

Knot and Mere share SDK dependencies. A staged Mere SDK at `639e78fc5` carries
the new execution pins so Knot can qualify that exact canonical Git source;
final Mere receiving then incorporates qualified Knot. The final main merge
preserves the staged SDK ancestry. Package identities must converge through
Cargo-generated locks, with no source aliases, local replacement checkout or
immutable-revision/precise-commit mismatch. `apparatus-core` remains its own
package and independent `75643df` pin.

The intake exposed a live WebSocket supplier handoff: Murm's published 0.30
boundary conflicted with Genet Netfetcher's 0.29 imports. Genet `7d1d89db` aligns
that one optional dependency with the existing Murm API. All 98 existing policy
tests pass, including 15 real loopback WebSocket cases. Its bounded two-crate
source workspace was byte-verified against the supplier. Actual Knot receiving
then compiles the canonical Git supplier and passes its existing provider/index
reopen tests. This does not claim a full JS/browser or Genet workspace sweep.

Knot intake also requires its existing Scroll response contract. Mere receives
the already-qualified Errand `3d2ce3a` family as one source; the older Mere
override omitted the `Reply` and `ResponseLimits` types used by Knot. This
continuation changes receiving pins, with no Scroll implementation change.

Actual Cleromancy receiving through published Git sources also exposes a public
snapshot type split hidden by Mere's workspace self-patch. Knot's site adapter
had pinned the portable `mere-site` model to an independent exact source, while
Djinn used the SDK branch source. The Rust model and manifest are byte-identical
at those revisions, but Cargo treats their selectors as different types. The
owning correction aligns Knot's model dependency with the canonical SDK branch,
retaining its portable dependency graph and semantic version. The failed real
consumer build is retained as a control; final receiving must show one
`mere-site` package identity as well as one Armillary and ESP identity. No
snapshot conversion or product workaround is introduced.

Woodshed's temporary desktop-host snapshot prevents current Cambium/Sprigging
intake. Its audit verifies all 48 recorded upstream file hashes and 46 unchanged
originals; the only behavioral patch suppresses synthesized held-key presses at
the existing native event owner. That guard and its test move verbatim to Mere's
shared host, whose guarded library source matches the donor byte for byte.
Woodshed `80f2d4eb` receives the qualified shared revision and removes the source
patch and 52-file, 13,288-line snapshot. Real presses and the existing
release/Tab-navigation path retain their behavior. Its four receiving roots and
affected leaf libraries pass 1,086 CPU test executions, with 11 existing ignores.
Four existing read-only accessibility hooks adopt Sprigging's shared `&self`
receiver without changing their bodies. This adds no native-focus or audio
acceptance claim.

The canonical source/receiving ledger is
`support/family-promotion/pool-cohort-rollout-20261022.json`. It records exact
published code inputs, per-root source identities, commands, results and
qualification limits as each receiving checkpoint is delivered. Native UI,
physical devices, heavy models and the Knot checkpoint feature retain their
separate owners and acceptance records.

The scoped receiving checkpoints are published. The final audit parses all
package entries in 19 committed locks, including inactive targets. It closes
two byte-identical Genet package residues in Cleromancy that actual-platform
metadata omitted. Runtime-qualified code and later lock-only evidence remain
distinct; products retain their tested SDK inputs instead of following
receipt-only commits through another recursive repin.
