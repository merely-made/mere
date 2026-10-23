# System Diagnostics and Accessibility Plan

## Producer activation and child lifetimes, 2026-10-23

**Status (2026-10-23):** bounded checked activation implemented; automated receiving qualified within the boundaries below.
Review and remaining migrations: `support/family-promotion/diagnostics-activation-review.json`.
Exact gates and source: `support/family-promotion/diagnostics-activation-qualification.json`.

### Phases and done-conditions

1. Refuse expired child tokens in publication, queue and host admission. Pointer
   equality of two cached weak tokens does not establish a live authored child.
2. Freeze raw operation-entry context before contributor callbacks. Preserve all
   earlier publication/queue bindings. Pass that admitted child token to checked
   activation rather than resolving a new child life for an old request.
3. Delegate checked activation under one mutable producer borrow. Rc adapters
   lend the same P across validation/dispatch; ContentProducer holds its actual
   ContentSlot borrow through token validation and revision-qualified activation.
   Raw and checked retained-content entry share one owner-borrowed implementation.
4. Receive expired cached tokens, raw/bound late admission replacement, shared
   mutable borrow, actual retained-content activation/retirement, canvas controls,
   native hosts, Wasm, embedding and a standalone consumer. Dispatch and refusal
   remain distinct from publication, terminal actor completion and durability.

### Findings and progress

Expired child tokens previously compared equal to cached copies. Admission now
checks liveness; full publication refuses ContributorEnded. The registry forwards
checked activation to the contributor instead of calling raw act after admission.
A replacement during the final admission callback cannot gain the old request's
child life. Refused activation and producer decline do not run after-dispatch
hooks or move focus as though an action dispatched.

The default checked method covers a directly owned model and observational child
reads. Adapters with deeper shared owners must override under that real model's
borrow/transaction. Optional None still requires never-reused authored keys;
a cached expired Some is not equivalent to None. Action IDs retain their meaning
within a child life, or need an owner-checked action revision/new child life.

### Remaining ownership migrations, in order

1. Canvas same-call removal/restoration needs a mutation-time lifetime witness.
   ingest_graph reconciles after an arbitrary closure; endpoint UUID comparison
   cannot observe a removal/recreation hidden inside it. Existing separate-call
   removal/restoration controls do not establish this guarantee. Track the red
   reproduction separately from passing acceptance gates; do not invalidate all
   unrelated live keys merely to avoid tracking the actual mutation.
2. Receive checked activation in every custom/deeper shared producer and explicit
   action-ID reuse. A trait default cannot make arbitrary interior state atomic.
3. Promote the AccessKit hook and coordinate standalone native consumer intake.
4. Receive guest namespaces/selection ingress and headed platform/AT handoff,
   installation, recovery and late events. This pass adds no such runtime claim.
5. Complete Rootstock intrinsic sizing/Path-B raster and owner-reported terminal
   actor outcomes through their separately qualified paths.

The lane retains at most 10 MB, reuses the stable Mere target, and preserves
pre-existing staged changes and concurrent receipt edits.

## Shared publication outcomes and collectors, 2026-10-23

**Status (2026-10-23):** implemented; automated receiving qualified within the boundaries below.
Continuation review: `support/family-promotion/diagnostics-outcomes-review.json`.

### Phases and done-conditions

1. Return a shared publisher observation alongside drained requests. Report
   physical adapter state separately from skipped/refused/native-submitted/
   browser-mirrored publication. Count producer contribution issues explicitly.
2. Rootstock stamps sync observations with the host and an independent sequence.
   Missing adapter/runner/layout and ended owners replace prior success. Disposal
   records an ended-owner observation; its notification runs from the disposal
   guard after the host borrow ends. Reads neither reproject nor advance sequence.
3. Expose copied observations to application contexts and an opt-in observer.
   Products retain redaction, bounded history, clock and export policy. Mesquite
   samples the latest observation in both receipt formats, separately from
   product diagnostics and acceptance. A failed/absent diagnostic read clears
   the previous exported batch.
4. Receive transition and independent-history controls, the real Mesquite finish
   path, native detached/ended-owner controls, native cohort and Wasm/embedding/
   standalone consumer compilation. Keep platform/runtime qualification separate.

### Findings and progress

`BridgeStatus` describes physical adapter installation. The pinned bridge's
`update` returns no acceptance acknowledgement, so native publication reports
**submitted**, never consumed or announced. Browser **mirrored** records completed
DOM writes; focus transfer and human AT behavior remain separate. Failed browser
writes clear proxy routes and cached elements so a retry cannot reuse partial
written state. Detached canvas/mirror pairs refuse publication.

The shared `A11ySync` contract replaces the action-only return value. Rootstock
owns the latest copied observation and notification, rather than each collector
inferring success from an installed adapter or an empty action vector. This is
not a semantic tree, target lease, presented-frame receipt, action-admission
result or actor-completion authority. Exported errors use fixed refusal codes;
contributor content and platform error strings are excluded.

### Remaining ownership migrations, in order

1. Promote the AccessKit traversal API and coordinate root patch/lock intake in
   external native consumers before advancing those consumer pins.
2. Make producer activation and child/action lifetime validation atomic at the
   model owner. Receive same-call canvas replacement and legacy authored nonreuse.
3. Give guest compositors owner-qualified namespaces and install/reset/replay/
   selection ingress receiving. This publisher remains ROOT-only.
4. Qualify native bridge acceptance, actual old platform events, partial install
   recovery, browser DOM failure/focus/disposal, and physical host handoff through
   headed platform controls. Submitted/mirrored observations do not close these.
5. Receive full Rootstock intrinsic sizing and Path-B raster integration.

Collector history retention/export stays product-owned. The resident lifecycle
collector does not gain UI authority or receive semantic contributor content.
This lane retains at most 10 MB and reuses the shared Mere target.
Exact source, gates, retained failed attempts and qualification boundaries are
in `support/family-promotion/diagnostics-outcomes-qualification.json`.

## Native publication ID remapping, 2026-10-23

**Status (2026-10-23):** native ROOT remapping implemented. Exact receiving
source, gates and limits are recorded in
`support/family-promotion/diagnostics-native-ids-qualification.json`.
The ownership and supplier-intake continuation is in
`support/family-promotion/diagnostics-native-ids-review.json`.

### Phases and done-conditions

1. Freeze complete authored node identity, including producer structural nodes
   without actions. The shared `PublicationBindings` contains weak identities
   and routes from one composition; it is not another semantic tree. Unknown
   nodes, duplicate IDs and inconsistent source/route metadata refuse capture.
2. Assign native IDs from one process-wide nonreused sequence. Ordinary updates
   preserve an unchanged life; child/leaf/registration/host replacement and
   withdrawal receive new IDs. Draft preparation does not modify installed
   bindings. Routes and namespace advance only after installation succeeds or
   the validated full ROOT update is sent to the bridge.
3. Rewrite every node-reference property through AccessKit's schema-owned
   `Node::try_map_node_ids`, then root and focus through the same map. The local
   pinned 0.25.1 patch matches typed property values exhaustively; it has no
   second list of relation properties in Mere. Malformed child graphs, dangling
   references and foreign tree/graft publication refuse before native routing.
4. Receive producer/leaf replacement, ordinary updates, pending and late IDs,
   structural withdrawal, host recreation, typed references and consumer tree
   updates. Receive native cohort, Wasm, embedding, retained-content and an
   independent-workspace native consumer. Platform runtime remains a separate gate.

### Findings, 2026-10-23

`semantics.rs` now records the authored source for structural producer children
as well as action buttons. `publication_bindings` replaces the action-only
`publication_requests` API; callers consume its fallible result. It freezes
request bindings from the same node identities used by native publication.
The browser consumes this contract too and clears invalid proxy routes.

`cambium-winit-a11y/src/publication.rs` owns native IDs; Rootstock owns authored
identity; AccessKit owns typed reference traversal; the Genet bridge owns
installation/replay. The pinned supplier lowers DOM projection nodes one-to-one
from opaque DOM IDs. An action-route map alone was incomplete for producer
structural nodes. Source projection helpers still return source IDs for
composition and now say so explicitly; they do not expose installed native IDs.

The native publisher drains pending events through its last committed bindings.
A late old numeric ID after replacement no longer resolves to a new target.
Rejected publications clear routes and the current namespace; failed first
installation never commits the draft. Recovery cannot recycle the draft/old IDs.
Publication refusal reveals the application window, matching install failure,
so malformed accessibility data cannot strand the first frame hidden. This is
a source-confirmed guard, with headed visibility/recovery still unqualified.
The old source-ID-only mapper fixture was retired; its pending-drain assertion
now uses the real namespace and AccessKit consumer.

### Remaining ownership migrations

The AccessKit hook is a local pinned supplier patch with upstream provenance and
licenses under `support/patches/accesskit`. Cargo patch tables do not propagate
through dependencies: a standalone native consumer receiving this Mere source
must declare the same patch at its own root. The independent
`support/family-promotion/native-id-consumer` fixture checks that recipe.
Upstream API promotion and coordinated external product patch/lock intake remain
open; existing external pins are not repinned by this pass.

This publisher owns ROOT only and explicitly refuses guest grafts/trees. Existing
TreeId-qualified compositor/guest paths remain separately owned. Selection
reference properties are rewritten, but SetTextSelection ingress is unsupported.
Publication refusal is logged; BridgeStatus describes physical installation,
not acceptance of the latest tree. The shared-outcome continuation above
receives typed publication observations and collector wiring; actor completion
remains separately owned.
Guest namespaces and guest install/reset/replay, physical window/canvas handoff,
actual late platform events, partial-install recovery and human AT need their
own receiving. Producer activation is not an atomic model transaction, and
legacy missing child tokens still require authored nonreuse. Same-call canvas
replacement and full leaf sizing/Path-B raster work also remain open.

### Progress, 2026-10-23

Schema traversal, complete weak identity, native namespace and source/route
validation are implemented and covered by named gates. Earlier native-ID-open
notes are historical for the ROOT remapper; they do not establish guest or
headed platform acceptance. The lane retains at most 10 MB, reuses the existing
Mere Cargo target, and preserves concurrent staged/receipt work.

## Native publication IDs and host ownership, 2026-10-22

**Status:** bounded host-owner and disposal repairs; native ID remapping remains
open. Exact source, gates and limits are in
`support/family-promotion/diagnostics-owner-qualification.json`; the pinned
native reference inventory and migration conditions are in
`support/family-promotion/diagnostics-owner-review.json`.

Published requests now capture a weak `A11yOwner` issued from the existing host
marker, alongside document/mount and contributor lives. The shared queue uses
that binding instead of retaining a separate owner-identity field. Moving the
same live runner, document and mount to a different host cannot transfer a
published request's authority; re-enqueueing never rebinds it. Raw constructors
continue to name the current live context. Direct raw and bound requests after
host disposal return `Cancelled`, and disposed hosts stop publishing even while
CPU runner/layout state remains inspectable.

`Accessibility::sync`, `publication_requests` and `bind_publication` now require
an owner argument. All four workspace implementations are migrated. Native and
browser publishers freeze it through the same projection contract. A custom
host must retain its marker while live, end it on disposal and use a fresh
marker for another lifetime. The weak token does not retain the host, validate
an installed physical window/canvas, or establish an atomic producer transaction.

The native numeric-ID gap is still open. A late OS event after map replacement
can select the new binding for the same number. The projection's action maps do
not identify every producer structural child, so allocating IDs from those maps
alone is incomplete. Complete publication identity must precede a native
namespace that preserves ordinary updates and never recycles ended lives.
Its schema-owned remapper must cover eight vector relations, seven scalar
relations, both text-selection positions, tree root and focus. Host/guest TreeId
ownership and install/replay remain separate obligations; bridge activation
epochs do not identify the originating target life. A children-only rewrite or
another maintained property list in Mere does not finish this migration.

Native `A11yHost::attach` now returns a result: a different window handle is
refused without replacing the original; attaching the same handle is idempotent.
Native host creation propagates that refusal. This headless handle guard does
not qualify actual platform installation or partial-install recovery.

Native install/reattach, late-event behavior, actual browser proxy/focus/disposal
and human assistive technology remain separately unqualified. Earlier host-owner
open lists are historical for the bounded request-binding repair, not evidence
that native publication IDs or physical surface ownership have been resolved.

## Main integration receiving, 2026-10-22

The child/leaf and producer/publication repairs are committed in `137c4fd4c`.
Fetched main `3dc450a6b` is integrated by `09145847b`, retaining the Djinn lane's
committed source and its separate uncommitted documentation/runtime work.
All 336 targeted tests passed with seven existing ignores; browser-host Wasm
and embedding compilation passed. The global D2 audit retains 40 pre-existing
errors. Concurrent Djinn commit `121a17b23` was also received without changing
the tested source or lockfile; see `diagnostics-audit-integration-final-receiving.json`.
The exact combined source and receiving gates are recorded in
`support/family-promotion/diagnostics-audit-integration-qualification.json`.
This receiving section supersedes earlier latest-main-pending notes below;
earlier receipts remain historical and are not relabeled.

A missing `retinue` entry in Djinn's lockfile dependency list was repaired
without changing package versions. Original staged receiving edits and receipt
removals remain separate from this audit. The temporary integration worktree
uses the existing Mere Cargo target; retained receipt cap is 10 MB.

The remaining ownership work is unchanged: native late-event numeric IDs need
complete publication/reference remapping, and host transfer needs host-issued
owner life. Optional producer child tokens do not establish an atomic actor
transaction or prove legacy nonreuse. Canvas raw callbacks can hide same-UUID
replacement. Rootstock intrinsic sizing and Path-B raster receiving remain
unfinished. Wasm compilation and headless tests do not qualify browser proxy,
focus/disposal, native install/reattach recovery or human assistive technology.

## Producer children and publication bindings, 2026-10-22

**Status:** bounded repairs qualified on this checkout: 287 distinct targeted
passes, seven existing ignores, browser-host Wasm and embedding compilation.
Receipts: `support/family-promotion/diagnostics-publication-qualification.json`.
The findings and larger migrations are recorded in
`support/family-promotion/diagnostics-publication-review.json`. The lane retains
at most 10 MB of receipts and reuses the existing Mere Cargo target.

Publication and queue admission now share one contributor binding. It records
the document root, window mount, target, producer registration, optional authored
child token and applicable leaf registration. Native and browser adapters freeze
these requests from the same semantic projection. Re-enqueueing preserves an
existing binding; editing the target cannot transplant its authority. Raw
constructors continue to name the current context. Existing fields stay readable;
external struct-literal callers must use `A11yRequest::new/node/produced` because
the provenance field is private. The two workspace literal callers are migrated.

`TextureProducer::child_lifetime` observes optional model-owned child lives.
`None` relies on the existing nonreused-key contract, not a guarantee that an
arbitrary producer satisfies it. Retained-content key entries own these tokens:
frames and producer recreation preserve them; close/source replacement retires
them. A weak derived lookup avoids scanning the full key map for each action.
Host admission rechecks after semantic composition, catching child replacement
by an impure read without claiming an atomic callback transaction.

Native sync drains against the last publication before replacing its maps.
Failed installation leaves the installed flag false and withholds new routes.
Browser routing checks exact proxy identity, replaces proxies on lifetime change,
preserves them for ordinary updates, restores held focus after proxy replacement,
and clears routes on disposal. Browser behavior is Wasm-compiled, not newly
qualified in a browser or with human assistive technology.

The earlier open-target lists below are historical for these bounded repairs.
A late native event still has no generation when a numeric target ID recurs;
this requires an authoritative ID and node-reference publication migration.
Direct publication requests moved with the exact document/mount to a foreign
host still lack host-owner provenance. Generic child/action activation needs an
owner-side transaction, and native failed-install/reattach recovery needs headed
receiving. Canvas same-call mutation visibility and full Sprigging host receiving
remain separate. Do not substitute a frame epoch, observed-description counter
or hand-rewritten subset of AccessKit node references for these owners.

Latest-main receiving remains open: fetched main advanced 50 commits to
`931a4ec8d`, including retained-content methods, tests and supplier pins. The
relevant upstream differences were inspected, but the shared staged/dirty
checkout was preserved. These receipts qualify the recorded local source, not
that newer dependency and consumer cohort.

## Child and leaf lifetime continuation, 2026-10-21

**Status:** bounded repairs qualified by 317 native/canvas/product passes with
seven existing ignores, plus browser-host Wasm and embedding compilation.
The final 30-test Sprigging rerun is a subset, not additional unique passes.
Receipts and source boundaries:
`support/family-promotion/diagnostics-child-leaf-qualification.json`.
The additive findings and remaining gates are in
`support/family-promotion/diagnostics-child-leaf-review.json`; retained receipt
cap: 10 MB, using the existing Mere Cargo target.

Sprigging now owns one insertion lifetime, observed weakly by both its paint
cache and Rootstock's semantic queue. Replacement at the same key drops former
commands and scenes before size/dirty gating, including without layout size.
Removal/reinsertion or a detached live registry cannot authorize an old queued
leaf value. Ordinary mutable updates keep the registration. Paint epochs remain
fresh after cache removal, so host fragments cannot alias a reset generation.

Canvas child keys are now assigned and pruned at construction/model
reconciliation rather than during semantic reads under a mutex. Removal and
restoration between reads receive a fresh key; graph switches and replacement
canvases also end old child authority. Metadata refresh and ordinary reads keep
live keys. Graphshell retained-document keys retain their existing model owner.

The earlier remaining-target list below is historical for these repaired leaf
and canvas portions. Generic producer child compliance, atomic activation and
native/browser publication provenance remain open. A raw canvas mutation
callback can still hide same-UUID removal/recreation inside one admitted call.
Private queue binding cannot recover provenance lost before enqueue.

A source scan also finds unfinished Sprigging host integration: Rootstock does
not receive `Leaf::measure`/`layout_dirty`, and `RenderedLeaves::scenes` has only
test consumers in this repository. The cache controls do not qualify full
custom-leaf sizing or Path-B rasterization. Maintained supplier contracts and
actual host receiving remain separate work, as do headed/browser-runtime and
human assistive-technology acceptance.

## Queued owner and registration lifetimes, 2026-10-21

**Status:** bounded repairs on `8008eba7d`, received with site-publication main
`a845b0b22`: 270 targeted native tests pass with seven existing ignores. Browser
Wasm and embedding compilation pass. Receipts, source seals and their limits are
recorded in `support/family-promotion/diagnostics-lifetimes-qualification.json`. This slice retains the existing Mere Cargo target and a
10 MB receipt cap, with gates named in
`support/family-promotion/diagnostics-lifetimes-brief.json`.

**Current main receiving:** implementation `8d4864d91` is integrated over
`49f0e197e` at merge `60daf11dc`. The merged cohort again passes 270 native tests
with seven existing ignores, browser Wasm and embedding compilation. Its exact
source and receipt hashes are in
`support/family-promotion/diagnostics-lifetimes-integrated-qualification.json`.
No diagnostic runtime source changed during the merge. The temporary receiving
worktree preserves concurrent Djinn/lock work in the primary checkout.

Later main receiving at `e1412c46d` includes the Q013 graph-snapshot repair.
The lifetime implementation and its lock inputs remain unchanged; six focused
native lifetime controls plus browser Wasm and embedding checks pass on that
merge. `support/family-promotion/diagnostics-lifetimes-main-qualification.json`
records this delta. The upstream graph owner's 474-test receipt remains
separately attributable; the six lifetime controls are a subset of the earlier
270-test cohorts, not additional unique tests or broad product acceptance.

Queued AppCtx actions now bind the originating host, the document allocation,
the window mount and, for generated actions, the logical producer registration.
These are weak bindings: an acknowledgement keeps neither the document nor a
contributor alive. Producer registration lives are allocated by `register`,
independent of GPU/image/device identities. Admission compares against the
current registry, so even a still-live detached registry cannot lend its old
authority to a replacement. Ordinary semantic reads, layout and state updates
do not allocate or revoke these lives. Current semantic admission still runs
for every request after its binding passes.

Dropping a pending queue completes its acknowledgements as `Cancelled`.
Independent-host `take_disposal` cancels pending requests immediately and closes
that host's semantic queue; later enqueue attempts also acknowledge cancellation.
A completed routing result survives subsequent disposal. A changed owner,
document, mount or producer registration returns `StaleTarget`. The queue owner
is logical input authority; existing CaptureOwner continues to own readbacks
and appearance/context invalidation. Host disposal closes both, while graphics
context recovery does not replace the logical semantic registration.

The native mapper now refuses non-root tree requests before numeric ID lookup.
The pinned Genet bridge's single-update `install`/`update` contract accepts only
`TreeId::ROOT`; guest trees use its separate ordered-batch API. Both drained and
public mapper requests use this same check. This repairs foreign-tree aliases,
without claiming that a ROOT tree ID identifies a publication or document life.
Browser mirror lookup also requires the event element to remain a descendant
of that particular mirror; its numeric `data-node` attribute alone is insufficient.
Browser runtime and human assistive-technology acceptance remain separate gates.

Pinned Genet already namespaces DOM identities: NodeId's high 24 bits name
an allocation arena and its low 40 bits are a non-reused serial. Independent
DOMs do not produce equal complete node IDs. The initial test incorrectly
assumed otherwise; its failed receipt is retained. The corrected owner controls
use the same live DOM/runner transferred between hosts, and the same live
producer registration with equal local slot/child keys across document
replacement. Do not infer current ownership from the birth arena: supplier
cross-document adoption preserves complete node identity.

### Remaining target-life owners

- **Authored producer children:** `ProducerNode::key` is unique within a
  description, but carries no lifetime. A child removed and reintroduced between
  descriptions can reuse its key. Allocate any child lifetime at the authored
  model transition, with a matching activation check; observation must not
  allocate generations. Graphshell retained content already owns keys over
  session instance/generation/node identity in `refresh_semantic_keys`; preserve
  that distinction and ordinary producer recreation. Remote board card keys are
  scene IDs (or ordinal fallbacks), with no action buttons today; those are not
  evidence of actionable child lifetime guarantees.
- **Leaf replacement:** `LeafRegistry::insert` replaces the contributor at a key,
  while `get_mut_as` updates a retained contributor. A DOM-target request can
  still reach a replacement leaf at the same DOM slot. A future registration
  contract must distinguish those operations at the registry, rather than
  turning every value update into replacement. The paint cache has the same
  missing distinction: `LeafRegistry::render_into` gates on key/size/dirty and
  carries a prior cached scene through `take_scene().or(prior_scene)` without
  checking contributor replacement. Source inspection therefore identifies a
  stale-cache path when a different leaf reuses a slot, especially Path B to
  Path A. Runtime qualification is pending. One registry-owned replacement life
  should serve paint reuse and action admission; test both clean replacements
  and changed render paths at unchanged size, while retained value updates
  keep their existing cache behavior.
- **Platform ingress/publication:** native maps are rebuilt before draining
  requests. Native ROOT IDs and browser current maps still lack the originating
  document and contributor life. Raw `A11yRequest` constructors and direct
  `apply_a11y_requests` remain current-context APIs; private queue binding cannot
  recover provenance already lost before enqueue. Extend the shared request /
  publication seam with owner-issued target lives, then receive it through both
  native and browser ingress and automation. Do not create separate platform
  lifetime catalogues or substitute a global presentation revision.
- **Native installation/rebinding:** failed installation still sets the private
  installed guard, and `attach` documents one-window use without an explicit
  replacement protocol. Keep install success, permanent unsupported status,
  bounded recovery and window replacement as distinct states. Actual headed
  failure/recovery qualification is required before a retry policy is claimed.

The broader target-life migration remains open until old platform requests,
removed/reintroduced children and replaced leaves refuse across their actual
receivers while ordinary updates remain valid. Wider Hest actions and supplier
forward geometry remain queued after that migration.

## Shared selector receiving, 2026-10-21

**Status:** shared observation/selector receiving qualified on the sealed
current family: 264 affected native tests, 46 supplier tests and 12 retained
product tests pass, with seven existing ignores. Core browser, actual Graphshell
and Distillery probe Wasm builds, embedding and resident lib/binary checks pass.
Owner/target lifetime admission is the next migration. This continuation
starts on Mere `3af00fc90`; the final receiving seal uses integrated main
`5573500dc` and Genet `206b80c2d62`. It retains the 10 MB receipt cap
and the existing Mere/Genet Cargo targets.

Taproot owns one matcher over borrowed semantic lookup views. Its DOM adapter
uses flattened traversal; its composed adapter reads the complete published
semantic tree and skips hidden subtrees. Exact-name preference, supported role
aliases, shallow class matching and DOM attribute filters retain their stated
contracts. Generated semantic objects are distinct targets. They do not inherit
arbitrary DOM attributes from the producer slot.

Harness and Mesquite now read the existing shared contributed composition,
including leaf-only roles and producer action buttons. DOM target coordinates
retain the existing product hook. Virtual action buttons have no invented
pointer or secondary-click route. Point-only queries may miss while an explicit
semantic click is available; querying must not record a false interaction failure.
Generated activation uses the owner-supplied
`ProducedAction` through the existing host accessibility admission path. The
Harness observes the dispatch result directly. Mesquite queues the request and
checks an immutable routing acknowledgement before spending another scenario
step. Producer refusal and a withdrawn offer are failures; queueing is not an
application-completion claim. Requests made by post-dispatch hooks wait for a
later drain rather than recursively consuming the semantic queue.

The browser mirror's independent role-spelling table is retired. The optional
`document-session-api/accesskit` bridge supplies the same spellings to Taproot
and `genet-render`'s host export. Taproot's normal dependencies do not acquire a
renderer. The default document contract remains neutral. This is a spelling
bridge for existing contributor vocabulary, not a new universal semantic model.

`support/family-promotion/diagnostics-selectors-qualification.json` records the
final gates and their exact source. Temporary Cargo checkout/update attempts and
the corrected scenario-test tick assumption remain preparation evidence. Normal
scenario completion already takes another tick after the last command; this
continuation makes no claim of repairing an existing last-step completion bug.

This observation/selector slice does not bind an invocation to a provider life.
Queued targets still use the existing DOM or `(slot, child, action)` identities.
The next migration must audit platform tree/window identity, host queue disposal,
provider replacement and key reuse before choosing lifetime tokens. Ordinary
republication must keep valid targets, and replacement must refuse old requests.
Broader Hest actions and full supplier-owned transform/scroll/clip placement
remain subsequent gates. This receiving does not establish new headed, browser
runtime, human AT or other external product acceptance.

### Next: owner and target lifetime admission

The native mapper still rebuilds its maps before translating queued requests.
The bounded continuation above now checks the platform target tree, but a ROOT
request still carries no originating publication life. Browser events are
restricted to their owning mirror, yet resolve DOM or `(slot, child, action)`
targets through current maps without a publication or contributor lifetime.
`LeafRegistry::insert` replaces the boxed contributor at a key. Producer keys
are unique within a description; the contract does not yet distinguish removal
and replacement from an ordinary state update. Renderer registration identities
exist, but are not automatically semantic target lifetimes.

The remaining done-condition covers originating native/browser requests across
document/window/provider replacement and removed-and-reintroduced child or leaf
keys. The private queue repairs above do not recover lost ingress provenance.
Old requests must be refused while current requests and ordinary republication
remain valid. Audit actual caller replacement/update
patterns before choosing tokens; do not allocate lifetimes from observation
reads or substitute a global presentation revision. Native tree filtering is now qualified against the pinned bridge's
root/embedded-tree contract; ROOT publication lifetime remains open.

The native initial-install branch also sets its private installed guard after
an error, preventing a later install retry in that object. This is separately
tracked as a state/recovery gap, not new evidence of a successful native adapter.
Transient failure, permanent unsupported status and bounded retry/fallback need
an actual headed failure/recovery gate before a policy repair is claimed.

## Ordered ownership migrations, 2026-10-21

**Foundation cohort:** provider/accessor work was implemented on refreshed main
`4232d5622`. Its qualification below leaves selector adoption open at that basis;
the later shared-selector continuation above records its own receiving gates.
Proceed in order: read-only observations, target generations, broader actions,
then supplier-owned forward geometry. Existing receipts retain their bases;
this continuation keeps the 10 MB cap and reusable Cargo target.

First done-condition: contributor descriptions use shared borrows; reading a
description does not allocate target identities, rebuild activation routes,
render or advance application state. Graphshell retained-content identities
move to the content owner's frame update; activation resolves against the
current document. AppCtx and Harness expose the same contributed composition
through read-only accessors. This is a borrowing/observation contract, not a
read/act transaction or presentation-revision lease. Selector catalogue adoption
and immutable observation receiving must be named explicitly before this first
migration is called complete. Preserve the DOM observation API's narrower
contract rather than inventing another role/name lowering.

The public Sprigging callback change is breaking. Prepare a new Sprigging minor
and matching Cambium minor in source; publication and receiving against existing
external product pins remain separate gates. Do not introduce a mutable
compatibility callback or hide identity allocation behind interior mutation.

### Read-only foundation and receiving

Sprigging descriptions and `TextureProducer::semantics` now take `&self`.
The registry, shared composition, native/browser adapters and embedding view
preserve shared provider borrows. `AppCtx::semantic_projection` and
`Harness::semantic_projection` return that same contributed tree, owner maps
and refusal report. Their existing `a11y_projection` remains the DOM seed.
A shared receiver is a provider contract, not proof against arbitrary interior
mutation by a third-party implementation or an atomic read/act transaction.

Graphshell's retained content owner allocates keys after an admitted document
frame, preserving live keys across ordinary frames and producer recreation and
retiring keys absent from the current session projection. Description reads no
longer allocate keys or rebuild activation routes. An unobserved producer can
activate a current advertised target through the owner; retired sources refuse.

Sprigging 0.3.0 and Cambium 0.6.0 are prepared source versions, not registry
publications. The resident gate exposed a real split: Knot's range excluded
0.6 while its shared editor host returned 0.6 types. Knot `b8d0e2f` widens the
unchanged embedded widget API's range; the current Mere lock now has one
Cambium/Sprigging family. That frozen test cohort retained the existing Murm
and Dramatis source revisions.
`diagnostics-readonly-final-resident.json` preserves the original four type
errors. The later retained-cohort resident check passes both lib and binary;
other external product adoption, including Woodshed's older Leaf callback,
remains a separate receiving gate.

The qualification record is
`support/family-promotion/diagnostics-readonly-qualification.json`; it separates
initial preparation and failed integration from the final source-sealed
receiving cohort. Automated native and Wasm gates do not establish headed,
human assistive-technology or cross-product acceptance. Taproot selector intake
below is a supplier/consumer migration, so it is documented here rather than
adding a second role/name catalogue in Mere.

Publication integration `d7fd54d71` receives main's subsequent local-passphrase
work and its intentional Dramatis `7d498c27` intake. All sealed semantic sources
remain unchanged; the root lock adds the authentication dependencies and that
supplier revision. `diagnostics-readonly-integration-qualification.json` records
the affected resident, retained-product and actual Graphshell Wasm gates on the
merged basis. It supplements the original 537-test cohort rather than replacing
its source seal or claiming a repeat of every unchanged family test.

### Ordered gates and current disposition

1. The current shared observation/selector receiving is described above.
   Other external product intake and registry publication remain separate;
   neither is inferred from this shared-host qualification.
2. Bind requests to the originating owner and target life. Receiving must refuse
   stale requests after window/session/provider/child replacement, while retaining
   valid targets across ordinary republication. Audit the actual platform queue
   and source lifetimes before choosing token representation.
3. Route the broader advertised action set through its current owner/Hest
   boundary. Keep focus distinct from activation, numeric validation and
   restrictive state shared, and routing acceptance distinct from terminal
   effects. Declined or unsupported requests must not claim success.
4. Finish forward geometry in Genet, including flattened paint-scope ancestry,
   transforms, scroll and explicit clipping semantics. Native/browser lowering
   must consume that mapping and qualify real producer pixels and input against
   semantic bounds. The current untransformed content-origin repair is not this
   full mapping.

## Producer semantic identities, 2026-10-21

**Status:** contained identity validation qualified on frozen main `8942e3050`, preserving Hest H4
receiving qualification. Receipt cap remains 10 MB across this continuation;
builds reuse `C:\t\cargo-targets\mere`.

Scope: validate child keys, per-child action IDs and generated platform IDs at
shared composition before publishing a producer contribution. Done when duplicate
descriptions and deterministic collision controls cannot emit repeated nodes or
leave actionable partial maps; valid siblings, reorderings and same action IDs
on different child keys must keep working through native/browser receiving and
current-target admission. Keep key generations, cross-window ownership and
read-only contributor contracts as separate migrations.

[AccessKit 0.25.1's TreeUpdate contract](https://docs.rs/accesskit/0.25.1/accesskit/struct.TreeUpdate.html#structfield.nodes)
uses IDs as replacement identity and requires rooted parent/child membership.
Repeated generated IDs cannot represent two distinct semantic objects. The
existing FNV allocator is stable, not collision-proof; a HashMap insertion is
not evidence of an unambiguous action binding.

At this identity-validation cohort, contributor callbacks still required
mutable access and selector observations still used the DOM seed. The later
read-only foundation and selector receiving above supersede those gaps. The
identity cohort's original test counts and source basis remain unchanged;
target lifetimes and full forward geometry remain separate migrations.

### Repair and qualification

One preflight in shared composition validates every child key and per-child
action ID, then reserves the generated IDs against DOM IDs and already accepted
producer IDs. Refusal is atomic per contribution: slot overrides, children,
action routes and ID reservations all remain untouched. The DOM seed remains
available and valid sibling providers continue to publish. `producer_issues`
returns typed refusal details from this read. Platform convenience APIs and
product collector adoption of that report remain open; it is not a new retained
diagnostic store or a lifetime-token mechanism.

Three before-repair runtime controls reproduce repeated browser action buttons,
actionable ambiguous descriptions and an actual host dispatch. Seven repair
controls pass, covering duplicate keys/actions, deterministic already-claimed
node/action ID collisions, valid siblings, stable reorderings, same action name
on different children and invalid/valid/invalid readmission without a stale
verdict. An earlier test-fixture borrow error is retained as preparation evidence,
separate from runtime failures.

Final frozen receiving passes 122 targeted native tests with five existing
ignores: producer lifecycle 41/one ignore, shared semantics 11, admission eight,
and native/browser/session receiving 62/four ignores. Browser-host Wasm
compilation passes. `support/family-promotion/diagnostics-identity-qualification.json`
records source/lock and executed-binary hashes; `diagnostics-identity-final-check.json`
records document, formatting, sealed-register and retained-byte checks. All
builds reuse the existing target; no isolated output was created.

Native projection comments no longer call mutable contributor reads pure, and
the DOM projection documentation names its actual seed role. This cohort does
not qualify immutable observations, generation-safe targets, collision-free
allocation, full geometry, browser runtime, headed interaction or human AT.

Publication preserves concurrent relay-recovery qualification and fixture tooling
at main `ad6c725e8`. Every tested source, manifest and lock hash still matches
the frozen seal; `diagnostics-identity-publication.json` records that comparison
without claiming another test cohort.

## Producer content origin, 2026-10-20

**Status:** bounded content-origin repair qualified on integrated main
`e94cb23e8`, after the initial `1797a2eed` refresh. Native Hest routing,
shared-shell/Hest scripting and concurrent co-op/replay changes are preserved.
Receipt budget remains
10 MB across this continuation; builds reuse `C:\t\cargo-targets\mere`.

Scope: use Genet's content-box origin for generated producer children and
action bounds in the host's existing untransformed, scroll-adjusted observation
space. Done when padded/bordered receiving, nested/document scroll, headless
sampled pixels, native/browser lowering and Wasm compilation pass with frozen
source evidence. Keep transform/clip and shadow/slot ancestor placement as
supplier-owned follow-up rather than recreating CSS placement in Mere.

The pinned `genet-livery::ElementGeometry` exposes content size/origin and inverse
pointer mapping but no forward semantic bounds query. Its placement query
collects ancestors through `dom.parent`, whereas paint/discovery consume the
flattened tree. Rootstock's `painted_rect` and browser mirror also retain
untransformed boxes. Completing this boundary requires supplier-owned forward
mapping and an explicit clipping representation, qualified against flattened
paint scopes and both platform consumers. That broader migration remains open.

### Repair and receiving evidence

Producer pixels and inverse pointer coordinates already use Genet's content
box, but generated semantic children and their action buttons used the DOM
slot's border-box corner. Pixel borders/padding therefore displaced semantic
bounds from the rendered producer. Rootstock now queries the supplier's content
origin and applies the existing scroll observation, with one composition shared
by publication and current-target admission. Missing slot or supplier geometry
does not invent coordinates. `ProducerNode::rect` now explicitly names content
pixels, matching `ProducerFrameInfo::logical_size`.

Before-repair runtime controls fail by twelve layout pixels in both axes,
including a headless GPU fixture whose sampled image is already painted at the
correct content position. The browser control fails by twenty-four CSS pixels
at 2x scale. Three repaired controls pass for sampled GPU pixels, both document
and ancestor scroll planes, and relative browser boxes with scale applied once.
The native fixture also includes actual border/padding and verifies child/action
rectangle equality before mapping an AccessKit request to producer dispatch.

This repair does not supply forward transforms, path clipping or flattened
ancestor placement. Existing percentage-padding interpretation is inherited
from Genet; the receiving controls qualify pixel padding and borders.

### Qualification

The final integrated cohort passes 115 targeted native tests with five existing
ignores, plus browser-host Wasm compilation: producer lifecycle 41/one ignore,
shared semantics seven, admission seven and native/browser/session receiving
60/four ignores. The initial `1797a2eed` cohort has the same results and remains
separate; it is not added to the final count. Before-repair failures remain
separate from repaired receiving.

`support/family-promotion/diagnostics-content-origin-qualification.json` records
the source/lock seals, both cohorts, executed native binary hashes and the
receiving boundary. The `before` and `before-rootstock` receipts retain the
three expected runtime failures. The `final-check` record preserves document
checks, unchanged sealed audit registers and the aggregate receipt budget.
All runs reuse the existing Cargo target; no isolated output was created.
This qualifies pixel border/padding, sampled headless RGBA, native action
mapping and browser mirror plans. It does not qualify browser runtime, headed
interaction, human AT or the supplier forward-placement migration above.

Publication integration preserves concurrent live owner-action delivery at
merge `7c9bc885e`. The tested source/manifests still match the integrated seal;
the sole later lock change adds the existing `url` dependency to the separate
`graphshell-mere` product. `diagnostics-content-origin-publication.json` records
that comparison separately; it does not replace or recount the test cohorts.

## Flattened producer discovery, 2026-10-20

**Status:** flattened discovery qualified on source-frozen main `24b079538`.
The initial refresh incorporated main `52690098b`. Hest H1/H2/H3 and
Cambium 0.5 receiving changes are preserved; accessibility's existing action
vocabulary still needs its separately tracked routing work. The diagnostics
continuation retains its 10 MB receipt cap and existing reusable Cargo target.

### Scope and done-conditions

Align producer discovery with Genet's `flat_children` traversal, shared with
semantic composition. Preserve ordinary light-DOM text walks. Prove actual
shadow/nested-shadow and assigned-slot discovery, fallback replacement,
unassigned-node exclusion, duplicate refusal and WindowDom containment through
portable capture and headless GPU receiving. Publication must name the same
current DOM owner. Keep full content-origin/transform/clip mapping, immutable
semantic observation and target generations as explicit later migrations.

### Findings

The old light-DOM finder served GPU preparation, portable capture and custom-leaf
size reads. It missed shadow producers while counting unassigned light-DOM
nodes as duplicate owners. Semantic composition independently walked the flat
tree, so an advertised action could coexist with absent rendering or capture.
GPU preparation, portable capture, leaf size reads and semantic routing now use
one Genet-based flattened traversal and the existing shared leaf-key parser.
The unrelated text-position walk retains its light-DOM text contract. Discovery
retains flat-tree members that may be CSS-hidden; existing layout geometry and
suspension rules still decide their rendering state.

The before-repair gate demonstrates the false observation path directly: the
browser semantic control passes while all six producer discovery/capture/GPU
controls fail. The repair controls pass for shadow, nested-shadow and named-slot
content, masked fallback/unassigned exclusion, duplicate refusal before callbacks,
WindowDom containment, fallback rebinding and removal retirement. Headless wgpu
RGBA samples change from the assigned image to the fallback image and then to
background after retirement. Native action mapping and browser mirror binding
also select only the assigned owner. This is sampled axis-aligned receiving,
not arbitrary padding/transform/scroll/clip or human AT acceptance.

### Qualification

The final source-frozen cohort passes 112 targeted native tests with five
existing ignores, plus browser-host Wasm compilation. Producer lifecycle,
shared semantics, current-target admission, native/browser publication and
session receiving are covered. The eight repair controls also pass separately;
they are not added to the final cohort count. The pre-repair runtime failures
remain recorded separately from repaired receiving.

Receipts are `support/family-promotion/diagnostics-flat-discovery-{before,repair,
producers,semantics,admission,platform,wasm}.json`; the `source` and `qualification`
records freeze source/manifest hashes and executed native binary hashes.
`diagnostics-flat-discovery-final-check.json` records document checks, unchanged
sealed audit registers and the aggregate diagnostics receipt budget. All builds
reuse `C:\t\cargo-targets\mere`; no isolated target, Cargo home or worktree was
created. This qualifies portable capture and sampled headless GPU receiving,
native action maps and browser mirror plans. Browser runtime, headed interaction,
human AT and full content-origin/transform/scroll/clip mapping remain open.

## Shared leaf and producer composition, 2026-10-20

This historical cohort precedes the flattened discovery continuation above.
Its light-versus-flat producer finder gap is superseded there; the coordinate,
read-only observation, action routing and target-lifetime gates remain open.

**Status:** contained host composition qualified on 2026-10-20 at integrated
main `67964c3ea`, implementation `3798e0c2d`. The refresh preserves resident
recall/readiness, portable-site extraction and Graphshell co-op work, plus
Netrender's borrowed-lowering supplier update (`ec15eb26`). Receipt
budget remains 10 MB across the diagnostics continuation; builds reuse
`C:\t\cargo-targets\mere`. Earlier cohorts below keep their original basis.

### Scope and done-conditions

Native publication, browser ARIA lowering and current-target admission consume
Rootstock's `project_semantics`. This composes the existing neutral DOM
projection through Genet's converter and then the existing AccessKit-shaped
Sprigging/producer contracts. Done when real registered contributors, native
receiving, browser mirror lowering, current-target/session controls and Wasm
compilation pass with source and lock evidence. This is host composition policy;
Genet still owns DOM extraction and products own contributor meaning and effects.

### Repairs and retired duplication

Previously native leaves received a fully seeded node, browser leaves a scratch
node and admission a scratch node containing only the requested DOM action.
Browser lowering retained a small role/name subset, dropping a real Knob's
slider role and numeric properties. Producer role/name/children/action IDs were
also constructed twice. Those independent compositions are retired. Native
publication consumes the shared tree and maps; ARIA lowers that composed tree;
admission rereads that same construction with current window membership.
Multiline role repair lives at this shared boundary. Custom-leaf keys reuse
OwnedLayout's parser. Multiple producer DOM slots for one key refuse contributed
children/action targets, matching the renderer's existing `DuplicateDomKey`
refusal rather than selecting an arbitrary semantic owner.

Producer descriptions take precedence over co-registered Sprigging leaves.
Otherwise an authored DOM name wins over the leaf's label in all consumers,
matching the browser's established policy. The leaf callback sees the seeded
node. Host geometry and tree edges and restrictive hidden/disabled/read-only
state survive callbacks. Disabled contributions cannot reintroduce actions;
read-only leaves cannot reintroduce numeric mutation. Producer action buttons
inherit disabled/hidden slot restrictions and have no admitted target map entry
when blocked. Hidden contributed subtrees lose route entries; a focused node
hidden by its contributor falls back to the document root. Missing slot geometry
no longer invents an origin at `(0,0)`;
nonfinite or negative producer rectangles produce no accessibility bounds.

### Remaining ownership migrations

- This original composition cohort left mutable callbacks and DOM-only selector
  intake open. The later shared observation/selector receiving above resolves
  those gaps. The contributor vocabulary remains AccessKit-shaped, and separate
  reads are not an atomic read/act transaction or presentation-revision lease.
- `a11y_projection` deliberately remains the DOM seed. Current consumers use
  `semantic_projection` and the shared Taproot lookup for contributed roles and
  actions; the narrow DOM API is not a second contributed catalogue.
- Requests still lack originating document/window/session/revision lifetime
  tokens. Within-window duplicate producer DOM bindings now refuse; cross-window
  producer ownership, reused registry/child keys, duplicate semantic keys/action
  IDs and generated hash collisions need explicit supplier/consumer rules.
- The existing host request enum routes only Click, Focus and numeric SetValue.
  An advertised Increment, Decrement or ScrollIntoView is not thereby routed.
  Broader command declaration/invocation/result migration belongs with Hest;
  routing acceptance remains distinct from terminal success.
- At this cohort's basis, producer render discovery used OwnedLayout's light-DOM
  walk while semantic discovery was flattened. The continuation above repairs
  that discovery gap. Render/input also use Genet's content origin,
  transforms and clips; this cohort's generated semantic bounds offset the
  projected slot rectangle. The content-origin continuation above repairs the
  border/padding offset in existing scroll-adjusted observation space. Forward
  transformed/clipped placement and shadow/slot ancestor geometry still need
  one supplier-qualified mapping and receiving controls.
  Consolidating the two old lowerings does not qualify their geometric premise.
- ARIA maps the known DOM and contributor roles. Other AccessKit roles remain
  generic; this does not qualify arbitrary leaf roles, clipping/pixel parity,
  browser runtime, installed platform delivery or human assistive technology.
- Product diagnostic final-process evidence, durable recovery, additional
  sources, A873 inspection, A961 input routing and selector grammar retain the
  open gates recorded below. This slice does not change those owners.

### Qualification

The broad native cohort passes 190 tests with seven existing ignores:
Rootstock 133/three ignores, native host 19/four ignores, native accessibility
receiving eight, browser mirror 17, Winit adapter seven and joined sessions six.
Its exact pre-integration source and binary hashes remain in
`support/family-promotion/diagnostics-contributed-pre-integration.json` and the
native gate. After merging main, 70 focused native receiving tests pass with
four existing ignores: contributor owner six, host admission seven, browser
mirror 17, native host 19/four ignores, native accessibility receiving eight,
Winit adapter seven and joined sessions six. The browser host compiles for
`wasm32-unknown-unknown` with default features disabled. No browser runtime or
human AT acceptance is claimed.

The [qualification index](../../../support/family-promotion/diagnostics-contributed-qualification.json)
records exact source/lock basis, executed binaries, gate commands, the four
upstream Netrender package-source changes and unchanged sealed registers. The
one local lock dependency addition is the existing Sprigging crate for actual
browser contributor tests. Current source hashes match the integrated seal.
Development compile failures and the fixture correction to reuse one DOM for
reordered producer IDs remain separate from passing gates. An initial
fast-forward refusal left the local lock patch intact; the tested implementation
was committed and main merged normally. The intermediate platform gate is
retained but excluded from the separately sealed 70-test current-main count.

The D2 audit reports no errors for the two updated plans and 40 existing
whole-tree errors. No isolated target, Cargo home or worktree was created;
receipts remain under the diagnostics continuation's 10 MB cap.

## Shared DOM semantics and resident collector, 2026-10-20

This historical cohort records the pre-contribution boundary. The shared leaf
and producer continuation above supersedes its contribution-lowering gaps.

**Status:** contained shared DOM projection and resident collector qualified
on 2026-10-20, based on main `6394e931`. The five intervening upstream commits cover recoverable settings
writes, Chronicle request binding, supplier inference fixtures and Mesh
provenance; they are preserved. The earlier 237-test and 178-test cohorts below
retain their original source identities. Receipt budget remains 10 MB for the
diagnostics continuation, using `C:\t\cargo-targets\mere` without isolation.

### Scope and done-conditions

Retire the native adapter's independent textbox/numeric DOM policy and consume
Rootstock's existing neutral projection through Genet's existing AccessKit
converter. Preserve native roles, focus, pruning and retained geometry. Wire one
real product lifecycle source through product sampling, bounded ingress,
redacted retained history and its existing admitted inspection route. Done when
the owner, platform/session and product receiving controls pass with source,
lock, binary and explicit remaining-gate evidence. Do not turn this into a
universal semantic or process-wide diagnostics model.

### Findings and repairs

Winit now lowers `document_projection`, the same DOM observation the browser
uses. Its separate app-textbox value/pruning implementation and unconditional
numeric marker override are retired. Routing maps contain only projected nodes.
Genet's converter currently maps multiline text fields to its single-line
AccessKit role; Winit derives the platform role from the shared `multiline`
state rather than rescanning DOM markers. Existing Sprigging and producer
contributions still follow, owned by those contributors.

The old native and browser ID walks used `dom_children`, while Genet projects
`flat_children`. This could announce a shadow control without a route to it;
Rootstock's marker and focus walks could also miss that control. All four now
use the supplier's flattened-child API. Scope membership still belongs to
`WindowDom`; traversal does not grant a foreign window's node authority.

Djinn's existing `EventLog` now accepts explicit optional diagnostic bounds.
The real executable reads `--diagnostics-config <path>` before starting its
vault/lifecycle, using the existing event source rather than a new authority.
It creates separate random nonsecret run/queue boot IDs, a local Registry
sampling policy, one explicit sender/receiver and an Apparatus collector.
No global sender or thread-local scope captures unrelated product logs.
Recognized owner phases are projected to fixed labels before queue admission;
profile, vault, endpoints, stop reasons, private fields and raw errors never
enter this diagnostic queue. A stopped event distinguishes explicit `ok=true`,
`ok=false` and absent/nonboolean outcome. Sampling refusals and unrecognized
events have separate counters, not inferred retained gaps. Zero bounds retain
the queue/store's existing disabled behavior.

Collection runs synchronously on that low-volume source, under its existing
serialization lock. Each inspection or receipt reader has its own cursor;
queue/history counters and identities remain separate. The existing owner-only
status route carries an optional second, content-addressed diagnostic resource,
with its original status resource first. The native `--resident-diagnostics`
client reads it through the same admission route. Off coverage is explicit;
diagnostic read failure produces an unavailable diagnostic resource without
breaking the resident's ordinary status. This does not alter vault access or
stop permissions. The original event-file persistence remains a distinct path,
not proof of diagnostic durability or successful application work.

A further contained authority bug was found in `EventLog::emit`: arbitrary
caller fields could overwrite schema, PID, receipt time and even event name.
Reserved fields now retain their owner values. Other event fields keep their
existing file behavior; the diagnostic copy retains only the fixed allowlist.

### Remaining ownership migrations

- **Contributed semantics:** Sprigging callbacks remain AccessKit-shaped;
  native, browser and admission still seed/lower their contributions differently.
  Browser leaf lowering currently keeps only a small role/name subset, losing
  numeric roles, values and actions. Producer semantics and generated action
  IDs are also lowered separately. A shared neutral contribution model needs
  supplier and native/browser receiving qualification, including read-only,
  disabled, labels, clipping and action lifetime. Shared DOM semantics do not
  complete that migration.
- **Targets and outcomes:** requests still lack document/window/session/revision
  lifetime tokens, and producer semantic reads and activation remain separate.
  Current projection membership and routing results do not establish terminal
  success or presentation/pixel causality. The new Hest command architecture
  plan owns the wider declaration/invocation/result consolidation; target and
  operation lifetime work should coordinate with that owner.
- **Product diagnostics:** this source covers only named resident lifecycle
  events. Unlock subevents, inference/workers, arbitrary tracing/mod-loader
  emitters and other products are not silently included. The source already
  serializes with event-file IO; adding a nonblocking ingress does not make
  that owner API nonblocking. Mutex/panic and durable recovery contracts remain
  open. History and final stopped/disconnect evidence are in memory; after
  process exit they are not available through the status route or exported by
  default. Installed-process, abrupt termination and final receipt custody
  remain separate receiving work.
- **Other owners:** A873 inspection skeleton retirement, A961 input routing,
  private selector grammar, legacy invariant-clock result loss and journal
  recovery remain open as recorded below. The sealed audit registers remain
  unchanged; the successor ledger carries new repairs and remaining migrations.

### Qualification

191 targeted native tests pass, with seven existing ignores: Rootstock 126,
native host 19/four ignores, native accessibility receiving eight, browser-host
native plan tests 17, Winit adapter seven, joined-session controls six,
resident lifecycle owner four and admitted status receiving four. Rootstock's
126 passes include three existing ignores outside that passing count. The
receiving test launches the actual compiled Djinn CLI as a separate process,
reads the redacted phase from the fixture broker, checks owner-only admission,
and verifies that inspector reads leave the independent receipt cursor intact.
The browser host compiles for `wasm32-unknown-unknown` with the flattened-tree
route walk. The source ledger and
[qualification index](../../../support/family-promotion/diagnostics-shared-qualification.json)
retain exact commands, source/lock basis and executed binaries.

The first compile checks caught a redundant `.into()` inference error and a
fixture using a private scroll setter. Both were corrected; scroll parity now
runs through the host's public wheel path. Failed development receipts remain
separate from final gates. Installed resident/vault startup and termination,
browser runtime, human AT and broader contributed semantics remain unqualified.
A client-process fixture is not an installed-resident acceptance receipt.

After qualification, upstream's Hest reservation and architecture plan
(`2fead114`, `9a173f18`) were integrated. Its reserved crate has no dependencies;
only workspace membership and the local Hest lock entry differ from the tested
manifest basis. Tested source hashes match, and locked offline no-dependency
metadata passes; the [integration receipt](../../../support/family-promotion/diagnostics-shared-integration.json)
records this separately without overwriting the original qualification. The
[Hest plan](../../cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md)
owns broader command declaration, invocation and result consolidation.


## Current-target admission and collector bridge, 2026-10-20

**Status:** contained host admission and collector bridge qualified on main
on 2026-10-20, based on `4e4a2c8c`.
The earlier 178-test continuation below retains its exact source and scope.
Receipt budget remains 10 MB across this diagnostics continuation; builds reuse
`C:\t\cargo-targets\mere`, without an isolated target, Cargo home or worktree.

### Scope and done-conditions

Revalidate each semantic request against the current host subtree, DOM semantics
and existing leaf/producer contributions immediately before dispatch. Expose
refusal and producer-declined results through the host and native harness. Wire
Registry staging to Apparatus independent readers through an explicit product
redaction function. Done when the targeted owner/receiving controls pass and
source, lock, binaries, results and unresolved migrations are recorded together.
This is a contained shared-library slice; product startup and broader source
migration remain separately owned.

### Findings and ownership

`Host::apply_a11y_requests` previously invoked producer actions even when their
slot was absent from this host, and trusted node requests beyond the runner's
live-node check. It now checks current window membership, projection presence,
disabled state, supported action, finite numeric data and current producer action
availability for every request. A callback's DOM change can refuse a later
request in the same batch. `WindowDom::contains` remains the subtree authority,
including shadow hosts; the host does not invent another ancestry walk.

`A11yDispatch` describes routing only. A producer's `false` reply is
`ProducerDeclined`, and does not move focus. DOM dispatch is not proof that a
handler ran or that downstream work completed. Rejected requests do not run
post-dispatch hooks; actual producer calls still do, including declined calls.
`sync_a11y` returns results in request order, and the native harness exposes its
single-request result. Native/browser event loops may still discard results;
automatic retained action-outcome instrumentation is not installed.

The neutral document projection now includes the existing enabled numeric
control marker. Sprigging contributions still use their existing AccessKit
callback. Admission consults that owner too so a leaf-only numeric action is not
silently broken by the stricter gate. This adds a direct Rootstock AccessKit
edge, already present transitively through Sprigging. It does not complete a
neutral semantic projection migration.

Apparatus's opt-in `registry` feature adds `RegistryCollector<P>`. Products
supply a bounded, redacted `DiagnosticProjection<P>` or explicitly filter a
packet. Raw messages, fields and producer correlation are not copied by default.
The retained envelope keeps collector run/source/sequence; payload context keeps
queue boot/sequence/receipt time separately. Product metadata is preserved,
without assigning a queue boot to a producer clock or inventing cause references.
Queue counters remain an independently scoped drain snapshot, absent before a
successful drain. Ingress rejection never becomes an invented retained sequence
gap; retained rejection/eviction still uses Apparatus's actual sequence space.

The constructor does not inspect or lock the receiver, avoiding a contended
startup dropping its sole receiver. Collection validates retained time and
sequence capacity before draining. Ingress and history retain independently
configured count/byte/age bounds. Each inspector/receipt gets its own cursor.
New queue lifetimes require fresh product-chosen run and boot identities;
old cursors observe the retained run boundary. The old sender remains the owner
of final disconnect/discard counters after collector retirement. Process-wide
`uninstalled_loss` is separate from any installed queue's loss snapshot.

The queue clock also used `std::time::Instant`, which is not a working browser
clock on `wasm32-unknown-unknown`. Registry diagnostics now uses the existing
`web-time` clock, matching the hosts. Native and Wasm compile gates are separate;
compilation does not establish browser runtime acceptance.

### Unfinished migrations and larger effects

- **Host identity and lifetime:** requests still lack an originating
  document/window/session/revision token. Current membership does not establish
  presentation freshness, disambiguate a foreign-document ID alias, or protect
  producer-key reuse across lifetimes. Producer semantic reads and activation
  also remain separate callbacks. Qualify an owner-issued target lifetime and
  atomic admission contract before claiming those guarantees.
- **Projection ownership at `65adff84`:** Winit built from Genet's AccessKit projection
  while Rootstock/browser consumed the neutral document projection, with local
  textbox/numeric/leaf overrides. This slice shares admission, not all semantic
  construction. Move the existing contributions into one supplier-qualified
  semantic model, preserving geometry, clipping, disabled/read-only behavior,
  shadow trees and actual native/browser receivers. Current AccessKit leaf
  callbacks also receive differently seeded nodes across these paths;
  context-sensitive callback equivalence remains unqualified.
- **Collector installation and policy (A1154/A1156):** the bridge is callable and
  fixture-wired; that cohort added no production `install_global_sender`, `should_emit_channel`
  or `should_emit_and_observe` caller. The continuation above adds an explicitly
  injected product sender and local sampling gate, without a global sink. Products must choose redaction,
  sink lifetime/thread routing, drain scheduling, registry policy/invariants and
  final-counter custody. A process-global OnceLock is not a per-window collector.
  Projection callbacks must not panic; panic/durable recovery is not supplied.
- **Other open owners:** A961 key/text routing, private selector grammar, A873
  skeleton adoption, journal panic recovery, legacy invariant-clock result loss
  and presentation/pixel identity remain with the previously named owners below.

### Qualification

237 targeted native tests pass: Rootstock 126 (three existing ignores), native
host library 19 (four existing ignores), native accessibility receiver seven,
Winit adapter five, browser-host native plan tests 17, Apparatus core/collector
24 and Registry diagnostics 39. The real browser host and the diagnostics-only
collector each compile for `wasm32-unknown-unknown`. The native receiving gate
includes the final read-only guard and actual sibling-window refusal.

The [qualification index](../../../support/family-promotion/diagnostics-admission-qualification.json)
records commands, source/lock and seven executed binary hashes. Named final
gates are `diagnostics-admission-final`, `diagnostics-platform-native`,
`diagnostics-collector-clock`, `diagnostics-browser-compile` and
`diagnostics-collector-wasm`. Earlier compile/CLI/fixture failures remain retained
as development attempts, not successful qualification. Native/platform/browser
checks preceded the diagnostics-only clock edge; their pins and tested sources
are unchanged, and the final combined Registry/collector gate plus collector
Wasm check cover that edge. The qualification index preserves both lock hashes.

OS/human AT, headed/browser runtime, whole-workspace acceptance and product-wide
installation remain separate. The sealed audit registers are unchanged.


## Observations and semantic action continuation, 2026-10-20

**Status:** contained continuation qualified on 2026-10-20; broader owner
migrations remain open. Continues the second boundary slice in the family
promotion plan; the completed September and October receipts below retain their
original scope. Main refreshed to `21deb0cb` before implementation.

Receipt cap: 10 MB for this continuation, within the family programme's 100 MB
total. Reuse `C:\t\cargo-targets\mere`. Keep the sealed full-source audit
registers unchanged; dated successor findings and qualification belong under
`support/family-promotion/`.

Read the ingress, retained store, UX mapper, scenario driver and their consumers
before changing shared contracts. First reproduce and repair Q326 (wheel surface
scope), Q386 (descriptor/config defaults) and Q391 (emitted channel catalogue).
Preserve configured overrides, queue/store lifetimes and producer redaction.
Follow one worker operation and one host-driven UI operation through admitted
dispatch and actual owner outcome, using independent bounded readers. Record
remaining owner migrations separately from individual defects; do not infer
terminal success, presentation or pixel causality from acceptance or quiescence.

Done when the contained regressions and selected receiving gates pass, the
worker/UI evidence and its limits are recorded, and remaining architectural
forks have named owners and concrete follow-ups. The graph/history and receipt
repair sessions in the ownership repair plan remain separate work.

### Findings and ownership, 2026-10-20

The dated successor review is
[`diagnostics-ownership-review.json`](../../../support/family-promotion/diagnostics-ownership-review.json).
It preserves the sealed audit and separately tracks individual repairs and
unfinished owner migrations. Q326 routes wheel selectors through Taproot's
surface-enforcing resolver. Q386 derives new-channel defaults from descriptors
without replacing existing configuration. Q391 generates emitted surface
channels and their catalogue from one exhaustive declaration. Further contained
repairs refuse nonfinite sampling settings, preserve fractional sampling rates,
and prevent stale inference replies from reviving or re-journaling terminal work.

| Responsibility | Current continuation | Remaining boundary |
|---|---|---|
| UX channel vocabulary (A1168) | One exhaustive declaration supplies emission and registration, including Inspector/Steward. | Browser-specific SurfaceId remains an adapter vocabulary. |
| Descriptor/config policy (A1154) | Descriptor defaults seed sampling/KeepRecent count; existing configuration retains precedence. | Registry metadata does not enforce a collector's retention. |
| Ingress and history (A1156, A870, A872) | Existing newest-rejecting ingress and oldest-evicting retained store keep their different lifetimes; inspection and receipt readers remain independent. | A collector must preserve source/boot scope and unsequenced contention/no-sink loss; ingress numbers cannot be invented as retained causal references. |
| Inference outcomes (A869, A1155) | An explicit opt-in adapter records fixed phases and generated execution IDs through the real actor and receiving run table. Worker reply, accepted outcome and journal refusal remain distinct. | Consumers choose run identity, count/byte/age limits and export policy. This does not bound the domain run table/token transport or backfill unobserved commands. |
| UI delivery | A retained-host fixture drives the actual wheel path and records its application callback, then samples the existing Mesquite attachment independently. | This is windowless CPU qualification, not a headed product or human AT receipt. |

`InferenceLane::enable_observations` adds no automatic export or daemon.
`observation_cursor` and `read_observations` use the existing Apparatus contract;
`set_observation_limits` changes retention without resetting identity or loss.
The payload omits prompts, answers, caller run names and error strings. The
producer supplies neither a semantic revision nor a presented frame it cannot
establish. Enabling twice refuses instead of silently discarding reader coverage.

The broader semantic-action migration remains open. `A11yHost::map_parts`
resolves against the last synced maps. At `4e4a2c8c`,
`Host::apply_a11y_requests` dispatched node/producer actions without a fresh
semantic admission/result boundary; the continuation above repairs that host
seam while retaining the wider identity migration.
Finishing it requires current-target and supported-action checks across platform,
local automation and producer lifetimes, with attributable refusal/outcome tests.
This is a separate API/receiving migration, beyond the contained repair pass.

Mesquite also restates Taproot's private selector grammar in `wheel::parse_selector`,
and key/input-text verbs take the narrower runner route (A961). Product `act`
and click booleans remain dispatch-handling facts, not unique-target or terminal
success receipts. The Apparatus skeleton and real inspection coexist (A873).
Presentation ordinals still require their run/host/context scope; attaching a
batch does not establish pixel correlation. These are named follow-ups, not
claims that the shared semantic application migration is finished.

The Registry gate remains an explicit API, not an installed process-wide filter.
At that earlier source, Mere had no production caller of
`should_emit_and_observe` or `should_emit_channel`, and no production
`install_global_sender` call. Djinn's new explicitly injected lifecycle source
and local sampling gate are recorded in the continuation above. The
nonblocking emitter does not consult the mutable registry automatically. The
sampling repairs therefore qualify that owner API; connecting producer policy,
bounded ingress and retained readers is still a host/collector migration.

A further source-confirmed failure-contract gap remains in
`descriptor::registry::legacy_violations`: an invariant clock error is mapped to
an empty violation vector. The keyed API exposes its typed error. Promoting that
truthful result through the legacy FIFO/public caller contract belongs with the
invariant adapter migration (A1155); an empty vector must not be treated as proof
that a valid inspection found no violation. No runtime reproduction of that
additional candidate is claimed by this pass.

The inference table also assumes its mutex is never poisoned while invoking
owner journal callbacks under that guard. A panicking journal can violate that
assumption; recovering a guard alone cannot establish whether an external write
committed. This remains a source candidate for the journal/lifecycle owner,
requiring panic and durable-recovery controls. The qualified controls here use
returned journal errors and actor panic outside that guarded callback.

### Qualification and progress, 2026-10-20

178 targeted native tests pass on the current Windows/Rust 1.99.1 source:
Apparatus core/serde 18, registry diagnostics 39, UX events 38, Mesquite 35,
Distillery library 42, inference observations 4, inference shutdown 1 and
lifecycle 1. The actor controls cover real threaded dispatch/replies using
deterministic providers, journal refusal after successful generation, cancellation,
actor loss and stale/unknown replies through the actual bridge. The retained-host
wheel control preserves request errors and observes the callback's real effect.

The three command receipts are `diagnostics-core-online.json`,
`diagnostics-mesquite-current.json` and `diagnostics-inference-current.json` under
`support/family-promotion/`. The
[qualification index](../../../support/family-promotion/diagnostics-qualification.json)
retains their exact source, lock and eight executed binary hashes. Setup attempts
remain distinct: early interrupted
Genet materialization, package-cache waits and missing offline sources executed
no qualifying tests. The completed Cargo-owned checkout is retained. All builds
reuse `C:\t\cargo-targets\mere`; this pass creates no isolated target, Cargo home
or worktree.

The selected gates all use `--locked`. `diagnostics-lock-inventory.json` records
one canonical source for each of 22 promoted packages in the lockfile; its only
task change is the Distillery-to-Apparatus edge. This is a locked-source inventory,
not all-target runtime qualification. A separate all-target metadata attempt
could not download uncached `accesskit_ios` in offline mode and is retained as
preparation evidence. Model/GPU, headed product, browser and human AT acceptance
remain separate, as do full command-refusal coverage and the migrations above.

## Current design: shared observations, 2026-09-29

**Status:** first diagnostics implementation qualified by focused Mere tests.
The bounded Apparatus core, optional Mesquite attachment and zero-capacity UX
recorder repairs are implemented. Turnstone's separate redacted observation copy
and Gloss/Inspector migration were published at Turnstone `d6b62adb`.
The combined consumer cohort is now published at Turnstone
`b2ead70a448948a1e8a9bde10b34f01524119606`, with optional Diagnostics inspection
and contributed semantic automation/platform actions qualified.
Redshank's real persistence worker now qualifies dispatch, execution and
save-reply correlation, including failed IO and retry. Its later presentation
adapter is requalified and published at Woodshed `cefc903`, with 68 desktop tests, strict
Clippy, Wasm compilation and paired/default native captures. The earlier
early-durability negative control remains qualified. Turnstone now passes 635
workspace tests with nine ignores, five exact optional participant checks and
five reviewed native Sky/Diagnostics images. Its preceding 612-test migration
cohort retains its original source identities. The September
30 continuation below separates shared presentation stamps, consumer readings,
remaining whole-application causal links and human accessibility acceptance. The June
design and receipts are historical evidence.

Share bounded observations and causal references across applications, using their
existing state and event producers. Do not introduce a universal application-event
enum, replace domain journals, or rewrite the runtime. The first implementation
priorities are a shared bounded record store and correlated Mesquite receipt
attachments, qualified by two different consumers.

### Name and ownership

**Ruled by Mark, 2026-09-29: Apparatus is the shared diagnostics library, in the
existing `mere-apparatus` package (library `apparatus`).** Reuse that home for
bounded observations and inspection beneath product-owned views. The name and
crate home are settled; the core and Mesquite attachment are implemented as
recorded below, with Turnstone and Redshank qualifying the bounded core.
The [September consolidation ruling](2026-09-23_crate_consolidation_plan.md)
requires components to remain modules and reserved homes to receive their actual
capabilities; these two consumers qualify the bounded contract and dependencies,
while two complete action/worker/pixel causal receipts remain a later gate.

**Gloss takes the operational overview.** Turnstone's configurable sections can
combine its minimap and recent visits with downloads, background work, sync and
items needing attention. Trail retains browsing history, recall and recovery;
Gloss may compose summaries from those sources. Turnstone's default is the
minimap plus Downloads, with configurable sections. Steward's download facts
move there and its separate runtime pane retires. Operational actions remain
product-owned; the current Downloads section is read-only. Background/sync
sections and retry/cancel actions are future extensions with their own gates.

This supersedes the operational-status assignment in the July 18 taxonomy
(`turnstone/design_docs/2026-07-18_meerkat_harvest.md`). At the naming ruling,
Turnstone's Apparatus pane analyzed selected graph-object facets and handling controls.
**Mark's follow-up assigns that object analysis to Inspector**, alongside its
inspection of documents and content within them, metadata and clipping. Inspector
shows the fields and controls appropriate to the selected subject; graph facets,
provenance and handling controls retain their product-owned sources and write
paths. Migrate those capabilities into Inspector before retiring the object-analysis
Apparatus pane. Application Settings remains separate. Both migrations now have
qualified source changes and native restart receipts. Within-document selection
inspection and background/sync controls remain later work.

Mere's [Apparatus crate](../../../crates/domain/apparatus/src/lib.rs) now exposes
a renderer-independent observation store. Its older empty diagnostic groups
remain behind the default `projection` feature; those placeholders do not
establish a working inspection interface.
[Alembic](../../../crates/distillery/alembic/src/lib.rs)
already owns Distillery recall/workshop scope; Eidetic owns retained artifacts;
[Armillary](https://github.com/merely-made/armillary/tree/main/crates/armillary/README.md) owns actor execution and messaging.

Products retain state, permissions, operation completion, persistence, redaction
and actionable controls. Genet owns computed DOM accessibility semantics; hosts
own surface mappings and presentation revisions. Taproot selects/drives those
semantics; Mesquite owns scenario scheduling and aggregate receipts. Registry
owns channel descriptors, configuration and invariant vocabulary. The proposed
shared store observes these owners; it neither dispatches actions nor recomputes
their truth.

### Present supply and consumers

This is a source inventory, not a new test receipt. Cross-repository references
name explicit repository paths; links stay within Mere.

| Owner and source | Existing evidence surface | Shared opportunity and limit |
|---|---|---|
| [UX events](../../../crates/system/ux-events/src/ux_observability.rs), [registry](../../../crates/system/registry/src/diagnostics/emit.rs) | Observers, probes, channel bridge, schemas and trace/message events | Reuse adapters; graph-specific action/node/surface types are not a universal app vocabulary. |
| [Cambium/Mesquite](../../../crates/cambium/mesquite/src/lib.rs), [receipt](../../../crates/cambium/mesquite/src/lane.rs) | Snapshots, string events, captures, failures, frame costs | Add optional typed attachments; CPU wall-time costs are not GPU timings. |
| Turnstone (`turnstone/src/observe.rs`), fanout (`turnstone/src/shell/effects.rs`) | Typed snapshot/AppEvent and existing 128-entry automation copy | A separate Apparatus copy records fixed redacted categories at fanout; automation strings and Trail authority remain preserved. Sealed pins, product tests and bounded native restart pass. Existing automation omission remains unreported. |
| Knot (`knot-editor/apps/desktop/src/scenario.rs`), catalog (`knot-editor/apps/desktop/src/graph/diagnostics.rs`) | Mesquite redacted document/appearance seal and optional actual Save/catalog records | Save dispatch, file-write return, request, execution, outcome and accepted catalog references remain Knot-owned. Other application domains do not acquire those causal links. |
| Woodshed (`woodshed/crates/woodshed-genet/src/scenario.rs`) | Stage/gesture snapshots, string events, drag metrics | Preserve arrangement and gesture meaning; capture-pending busy is not an operation model. |
| Redshank (`woodshed/ports/redshank/desktop/src/scenario.rs`) | Playback/transcript snapshots, seek requests, worker state | Workers report execution outcomes; desktop/session own stale-result acceptance and model application; persistence owns durable acknowledgment. |
| Cleromancy (`cleromancy/src/ui/native/scenario_driver.rs`) | Consultation/catalog state and durable first/reopen receipts | Preserve consultation authority and acceptance; useful worker-outcome pilot. |
| Mesocosm (`isometry/mesocosm/crates/mesocosm-genet/src/app/bench/probe.rs`), Eponym (`isometry/eponym/crates/eponym-client/src/bin/session/probe.rs`), Isomere host | Mesquite world snapshots, event cursors and cost observations | Keep world replay journals, rules and accepted-game-event history product-owned. |
| Hocket (`woodshed/ports/hocket/crates/hocket-genet/src/scenario.rs`) | Older local scenario/capture loop | Future Mesquite adoption; not a prerequisite for this design. |
| Signalman (`retinue/apps/signalman/src/observation.rs`), retention (`retinue/apps/signalman/src/observation/persistence.rs`) | Admission, source boot/sequence/gaps, bounded disk captures | Reuse limit/loss distinctions; radio schemas, authentication and physical receipts remain local. |

Historical [Graphshell watchdog design](../research/2026-05-17_graphshell_harvest_brief.md)
and Meerkat's bounded `HostObservability` are donors. Meerkat's cache survives in
git at `6f8e6903:crates/meerkat/src/observability/mod.rs`; its host was deleted in
`c5f01064`. Do not describe it as current Turnstone wiring. Current apps inspected
do not install the shared UX observer/registry sender as a stack-wide pipeline.

### Small contract over product payloads

`Observation<P>` carries schema version, run/source identity, monotonically
increasing record sequence, receipt time, optional source time, optional operation
identity and causal record reference, optional subject reference, and optional semantic
revision/presented-frame reference. `P` remains the producer's versioned, redacted
payload. References are scoped by run/source; sequence numbers are not globally
unique. Wall clocks do not establish causality. Keep source time, source boot,
host receipt time and monotonic elapsed time distinct; unavailable values stay
unavailable. Object, revision and causal links exist only when the producer supplies
them; absent correlation is explicit, never inferred from timing or matching text.
[Armillary correlation](https://github.com/merely-made/armillary/tree/main/crates/armillary/src/message.rs)
can map into these references without making actors mandatory.

Redshank illustrates why these boundaries matter: its playback worker replaces
a latest-value snapshot and throttles position wakeups
(`woodshed/ports/redshank/playback/src/worker.rs`). `Desktop::poll` accepts or
rejects replies against current product state, while `Persistence::acknowledge`
separately advances durable revision (`woodshed/ports/redshank/desktop/src/main.rs`).
The pilot needs hooks at those actual reply, adjudication and acknowledgment
boundaries. A snapshot-difference adapter cannot recover every outcome; missing
pre-instrumentation coverage is not a counted dropped event. Reuse real request,
load and revision identities; an item ID alone is not an operation ID.

An event records an occurrence. A snapshot describes state at a revision. A
diagnostic is an observation or validation result, not a second authoritative
event journal. A state difference may be recorded as an observed change, but must
not invent a command or cause. Action accepted, operation started, terminal
outcome, state observed, and frame presented are distinct milestones. `act = true`
or `busy = false` proves neither success nor presentation. Rejected, failed,
cancelled and stale-discarded outcomes remain distinguishable. Terminal records
come from the operation owner; generation checks remain there. Stale targets
cannot be retargeted merely because a node identifier was reused.

Registry [descriptors](../../../crates/system/registry/src/diagnostics/descriptor/types.rs)
describe schemas, retention and sampling; they do not enforce buffering or validate
every payload. Its [invariant matcher](../../../crates/system/registry/src/diagnostics/descriptor/registry.rs)
now supplies bounded run/source/operation matching alongside the explicitly
uncorrelated FIFO API. The September 30 keyed and deadline controls qualify
matching mechanics; each product still supplies its actual operation identity.

### Bounds, loss and privacy

Configure count, accounted encoded bytes and maximum age at each store/queue
boundary; expose defaults as product settings/policy. Zero count/bytes or zero
retention age disables retention. Reject an oversized record before admission;
evict oldest records until all enabled bounds hold; age uses host monotonic time.
Bound producer payload construction and ingress too: a bounded ring behind an
unbounded sender is not bounded end to end. Define and test accounting for envelope
and payload bytes rather than claiming a precise allocator-memory ceiling.

Expose retained range/bytes plus cumulative rejected, evicted, expired and dropped
counts. Independent cursors or immutable batches prevent competing destructive
drains; a slow cursor receives an explicit gap and next available sequence.
Neither missing records nor an empty disabled store means “nothing happened.”
The [RecordingObserver](../../../crates/system/ux-events/src/ux_observability.rs)
and [RecordingChannelSink](../../../crates/system/ux-events/src/ux_diagnostics.rs)
formerly appended indefinitely at capacity zero. They now return before locking
or cloning, with disabled-retention and bounded-eviction regression tests.
The September 30 registry emitter replaces the unbounded sender with bounded
nonblocking ingress and observable admission/loss. Adapters must preserve its
explicit unsequenced-loss distinction; registry catalog growth and producer
allocation remain outside this bound.

Products project and redact before buffering. Turnstone's credential-omitting
authentication observations provide a precedent, not proof that arbitrary URLs, titles or logs
are safe. Diagnostic collection does not enable screenshots, export or durable
capture. Each requires existing explicit product/run policy; secret-bearing raw
inputs must not be captured automatically. Retention, persistence and export are
separate policies. Export failure is visible and fails a requested receipt;
absence is never silently reported as success.

### Sequencing and acceptance

1. **Semantic parity first:** automated fixtures prove Taproot role/name matching
   uses the authoritative Genet computation while explicit text/class selectors
   retain their meanings. Exercise referenced/native labels, hidden/offscreen
   controls, disabled actions and disappeared/stale targets. Then native receipts
   identify the same object/revision across selector, accessibility action and
   diagnostic projection. A human Narrator/VoiceOver/Orca session is a separate
   acceptance level; neither unit tests nor native capture establishes it.
2. **Bounded records:** Turnstone's observation copy and one Mesquite worker app,
   preferably Redshank or Cleromancy, consume the same storage contract. Tests
   cover zero, byte/count/age overflow, oversized payload, slow independent readers,
   reset/new run, sequence gaps and concurrent out-of-order outcomes. Loss must be
   visible in both product inspection and receipts. Signalman informs the contract
   without migrating its evidence format.
3. **Correlated receipts:** attach optional records/loss summary to Mesquite's
   existing outcome, frame ordinal, capture and paint evidence. In two consumers,
   follow action dispatch through a real worker request/outcome to observed state
   and the exact presented frame. Prove rejected, failed, cancelled and stale
   paths, including capture/export failure. Existing product receipts and apps
   without diagnostic attachments remain compatible.

Done means owner-specific code and recorded gates satisfy those slices, with
native and human evidence labeled separately. The first storage and receipt
slice below does not close the real-worker or exact-frame acceptance gates.

### Bounded diagnostics implementation, 2026-09-29

`ObservationStore<P>` now enforces configured count, accounted encoded bytes and
monotonic age. Rejected and known dropped records consume scoped sequence
positions; independent readers receive copied batches, unavailable ranges,
per-run loss totals and explicit reset boundaries. Any zero limit disables
retention. Backward time fails before state changes. The storage core uses only
`std`, with optional serialization; renderer dependencies belong to `projection`.
Products supply already redacted payloads and truthful encoded payload lengths.
Deterministic envelope accounting is added by the store; this bounds accounted
encoding, not allocator memory or JSON export size. Payload mapping preserves
original admission accounting.

Mesquite adds default optional `Product::diagnostic_attachment`, sampled once
after `complete`. A supplied bounded `Batch<serde_json::Value>` carries retained
records, gaps and loss in the JSON receipt or text receipt line; absent hooks
preserve the existing format. Attachment errors fail the requested receipt.
`sampled_at_lane_frame` names the receipt observation point. It does not identify
the captured pixel frame or invent a causal revision: current capture fields are
sampled when asynchronous readback lands, and exact pixel/state pairing remains
open. No worker lifecycle is inferred from `act` or quiescence.

The [focused gate receipt](../testing/2026-09-29_apparatus_diagnostics_receipt.md)
records 74 passing all-feature library tests (17 Apparatus, 19 Mesquite, 38 UX),
14 passing core tests without default features and with serialization, and 75
passing host integration tests (16 host library, 24 scenario, 16 mere-view,
19 Mesquite). These are automated tests; this pass produced no new headed
diagnostics or human screen-reader receipt.

Turnstone's source adds a separate fixed-category observation store at existing
app-event fanout. URL, path, title, prompt and error fields never enter that
copy. Count/byte/age settings are configurable per run; one bounded
`diagnostics.json` is exported only for an explicit shared scenario receipt,
and export failure changes `scenario.done` to `RESULT fail`. The existing
automation event stream and Trail remain separate. This adapter and the
Gloss/Inspector pane migration pass on Mere `ca2351b3`, Genet `19c20687`, Knot
`c92ad044` and Woodshed `752c920e`. Turnstone's full workspace/all-target gate
passes 612 tests with nine ignores; five optional participant and 33 UI checks
also pass. The normal native binary and fresh/restarted scenarios pass with
three inspected nonblank captures. Initial retention of two records reports
eight evictions and gap `[1,9)`; restart retains one record without loss. Saved
Downloads/Recent order and Inspector's requested viewer override are checked.
The consumer's `turnstone/design_docs/2026-09-29_shared_diagnostics_gloss_inspector_receipt.md`
records the source graph, hashes, failures and open gates. Counts overlap and
must not be summed; requested viewer selection does not qualify Reader completion.

Redshank's published persistence pilot at Woodshed `a57085b` observes a real
save request, worker start/outcome and desktop reply handling. Successful
execution does not advance durability before the existing reply adjudication.
Its 63 desktop tests, strict Clippy, deliberate early-durability negative
control and opt-in/default/invalid-setting native receipts pass. The opt-in
native batch retains two records, reports 16 evictions and a `[1,17)` gap, and
links execution to reply handling at durable revision 3. See the consumer's
`woodshed/design_docs/2026-09-29_redshank_persistence_diagnostics_receipt.md`.
Other worker families, cancelled/stale storage paths,
exact frame correlation, custom-leaf semantic parity and human AT remain open.

### Semantic implementation progress, 2026-09-29

**Status: in progress.** Genet published Taproot's projection-aware matcher at
`19c206873ab08ae227217892d9e74d0df18b349a`; all 25 Taproot library tests passed.
Mere's integration exposes the existing Genet document projection through
`AppCtx`, uses it in Mesquite's role/name selectors, and revalidates a retained
node after scrolling before dispatching its held click. Explicit class/text
matching retains its existing behavior. The shared host fixture compares
selection and accessibility activation of the same referenced-name button and
rejects targets hidden or renamed while a click waits.

**Corrected 2026-10-06 (S14 pass):** this subsection's "in progress" status is
closed. Its own "Final integration" paragraph below closes the source
integration and machine-verification gates (`32edc2ad` merged), and the
2026-09-30 section supersedes it, qualifying contributed semantics at
Turnstone `b2ead70`. Human AT and custom-leaf parity stay open there.

Mere's isolated integration passed the standalone Graphshell Wasm check,
60 focused tests and the native smoke
(32 frames, three distinct nonblank captures, AccessKit installed with 15 nodes).
A raw-DOM negative control failed the referenced-name test, then all 21 scenario
tests passed with the exact source restored. Publication and integration with
concurrent Mere work remain pending; a human screen-reader follow-up is still
required. The current projection covers DOM semantics: Sprigging custom-leaf
contributions are subsequently merged by the native accessibility host. Its
revision is currently zero, and current DOM plus retained layout is not an
immutable presented-frame snapshot. Neither full semantic parity nor the later
correlated diagnostics acceptance is established by this slice.

The isolated integration subsequently merged the sealed Rootstock repair
`8eca3e4c` as `ced161f1`. Independent review confirmed both sides' source was
preserved. The combined Rootstock, winit host and Mesquite gate passed 199 tests
with no failures or skips; the native smoke repeated the same three capture
digests. The standalone Wasm check passed against the primary owner's accepted
web-lock baseline, with every package/version retained and only Taproot's new
document-session-api dependency added after mapping the Genet revision. Final
publication still waits for the concurrent main integration, including the
incoming Insigne migration; its broader acceptance is a separate owner's gate.

**Final integration:** the published primary baseline `32edc2ad` was subsequently
merged, preserving the Insigne migration and Knot `855cb75d` adoption. All 199
Rootstock/winit-host/Mesquite tests and the standalone Wasm check passed again.
Both locks retain every incoming package/version after the Genet substitution;
the only added dependency edge is Taproot to document-session-api. This closes
the earlier pending source-integration and machine-verification gates. The
human AT check and custom-leaf parity remain open; the first diagnostics
implementation and remaining qualification gates are recorded above.
The Apparatus name/crate home and Gloss operational overview are now ruled above;
the Gloss and Inspector pane migrations are qualified above. Contributed-pane
semantic automation is the next designed slice in Turnstone's pane registry plan;
its scoped retained-layout and admission-generation work is not implemented here.

---

## Remaining work orchestration, 2026-09-30

**Status: first remaining-work batch published, authorized by Mark.** Shared
delivery, inspection, presentation pairing and the bounded contributed-consumer
gates below are qualified. Broader owner gates and human acceptance remain open.
The September core and consumer receipts remain the baseline. Research lanes inspect current code before each
bounded implementation is promoted. Unrelated Gaz, transcript and allocator
migration work remains with its existing owners. New consumer pins are sealed
only after the shared source gates pass; the September receipts keep their
original source identities.

| Lane | Owner seam and done condition | Current state |
| --- | --- | --- |
| Contributed semantics | Genet/Taproot generic surface filter; Turnstone retained projection, visible geometry and pane/admission/node identity. Referenced-name, ambiguous, disabled, scroll and stale-target controls plus native receipt pass. | Combined Genet `69a2383b` passes 28 Taproot/40 render and two font checks. Turnstone `b2ead70` passes 635 workspace tests/nine existing ignores, five exact optional native participant tests and three reviewed Sky images. Actual Calculate request 3 is held while clipped, scrolled and delivered before the following assertion. Admission refusal fails before repair; generation and Gloss platform-arm controls fail deliberately and exact restoration passes. Built-in/live-document coverage and human AT follow separately. |
| Diagnostic delivery | Registry bounded configurable ingress with explicit disabled/full/disconnected/no-sink accounting; correlation-keyed, bounded pending outcomes and deadline handling. Concurrent out-of-order and late-terminal controls pass without an unbounded compatibility route. | Implemented; 135 tests/one existing ignore, restored lean 36 and two expected failing controls. No application pipeline or bounded catalog claim. |
| Presentation pairing | Seal a product-owned immutable reading and actual host frame identity at the render/capture boundary; carry both through asynchronous readback. Two products qualify UI state/pixel pairing and mismatch controls. Supplied worker references remain scoped to their product domain. | Mere `bd5912fb` qualifies 222 containing host/menu/key tests and actual web/Graphshell Wasm compilation; late-field failure control restored. Woodshed `cefc903` qualifies 68 desktop tests, strict Clippy, Wasm and six paired/default native images: rendered UI facts are distinct from persistence owner-time state and admission context. Knot `3dfb70b` qualifies 260 desktop/63 document tests, a restored stale-result control and nine reviewed worker/theme native images; only its visible accepted catalog carries supplied Save/outcome references. Whole-UI causal/revision authority remains unavailable. |
| Product inspection | Independent non-destructive readers show retained records, explicit unavailable correlation and gaps/loss. Shared projections do not format raw product inputs or create operation authority. | Shared inspection passes 22 all-feature/18 lean tests. Turnstone `b2ead70` qualifies optional read-only Gloss Diagnostics and two reviewed fresh/restart native images. Fresh retains two records/266 bytes with four evictions and gap `[1,5)`; restart retains one/133 without loss. Saved Diagnostics, Downloads order survives restart; default minimap plus Downloads stays unchanged. |
| Broader adoption | Qualify additional worker families and built-in/custom/document surfaces against the same contracts; preserve each product's stale, cancellation and durability adjudication. | Depends on the preceding shared and first-consumer gates; inventory is above. |
| Human accessibility | Prepare a reproducible keyboard/screen-reader checklist against the qualified native binary, then record the human observations and defects separately from machine receipts. | Turnstone checklist and Windows/Narrator preparation baseline recorded. Optional human-availability prompt is pending; actual human observations are required and no pass is claimed. |

### Findings, 2026-09-30

These are the research baseline before the implementation slices above.

- Registry's installed `Sender<DiagnosticEvent>` is unbounded and ignores
  delivery failures. Its current source installers and invariant matcher
  consumers are tests/helpers rather than an installed stack-wide pipeline.
- Mesquite currently records capture viewport when arming, but reads product
  snapshot fields when pixels are collected. Rootstock can dispatch queued
  pointers after paint, and native/web hosts can dispatch accessibility actions
  before the lane's after-frame collection. Neither current timing establishes
  that the fields describe the captured pixels.
- Turnstone contributed panes already supply retained provider DOM, layout and
  platform accessibility routes. Their scenario resolution still uses the raw
  fresh-layout route; the bounded scoped slice can use the retained owner seam.

Each lane records source identity, focused and deliberately broken controls,
restored passing gates and native evidence where required. Unavailable causes,
semantic revisions and exact image correlation remain explicitly unavailable
until their producer supplies and qualifies them. Human participation is a
separate acceptance requirement and cannot be inferred from automation.

Final source review also found an admission/result mismatch in Turnstone's
contributed delivery helper: hidden, disabled or unsupported actions could be
refused by the surface while still reported landed to Shell, allowing a rejected
Focus to change pane focus and stacking. The repaired surface returns
`Option<SurfaceRequest>` and the helper tests admission, separately from redraw.
The actual-runner refusal regression fails before repair (101), then all eight
contributed AT tests pass. A test-provider wrapper qualifies admitted actions
without host effects; the shipping Runner itself requests redraw. The final
generation failure control restores exact source bytes before the containing gate.

### Progress, 2026-09-30

The renderer-independent `apparatus::inspect_batch` now projects an independent
batch into bounded read-only rows, including explicit gaps/loss and unavailable
correlation. Scoped references are retained; arbitrary payload fields are not
formatted. The optional `project_inspection` lowers those same readings to
structural accessibility labels, with no invented bounds or actions. Its focused
all-feature library gate passes 22 tests, including five inspection controls.
Turnstone's optional Gloss Diagnostics section and read-only App mirror are
published and machine/native qualified at `b2ead70`; human acceptance remains open.
The existing minimap-plus-Downloads default remains the accepted default.
The shared lean inspection gate passes 18 tests. Registry and capture evidence,
negative controls and exact acceptance limits are recorded in the
[current gate receipt](../testing/2026-09-29_apparatus_diagnostics_receipt.md).

The next worker family is Knot's existing catalog reader. A successful Save
supplies the cause for its refresh request. Worker admission, execution,
superseded queued requests, stale completions and accepted read outcomes remain
Knot decisions; diagnostics observe those decisions. A superseded queued request
is not a cancelled worker, and an accepted catalog containing read errors is not
whole-job or durability success. Generation exhaustion must refuse further
requests rather than wrap and admit stale results. Knot's published combined
cohort at `3dfb70b` now qualifies this worker domain: actual Save and partial
read-error outcomes, count-one loss, paired/default themes and a restored
stale-acceptance failure control. Original and combined Commands/fonts receipts
retain separate source and binary hashes in the consumer plan. Other workers
do not acquire this qualification.

Turnstone's combined consumer pins are Mere `bd5912fb`, Genet `69a2383b`, Knot
`3dfb70b` and Woodshed `cefc903`. The final source audit verifies all 254
source/scenario/manifest/wrapper inputs through qualification and publication;
five native images were reviewed. Sky's lower scrolled background is black and
field values are unreadable in its final image; applied values are proved by the
scenario observation, not screenshot text. The optional native Piccolo/Wasm
checks do not qualify a Turnstone Wasm target or browser runtime. Source/binary,
failure-control, loss and publication evidence lives in
`Code/testing/turnstone/contributed-semantics/` and
`Code/testing/turnstone/diagnostic-inspection/final/`. Turnstone's custom compositor
has no immutable diagnostic-state/pixel seal yet; its behavioral captures do not
close that gate. The existing root docs were updated without a functional repin.

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_47_s14_phase_b9.md: the status holds; the
  2026-09-29 semantic subsection's "in progress" status marked closed and
  superseded by the 2026-09-30 section.

### Remaining owner gates after the first consumers

The current inventory finds real seams, rather than a need for another universal
runner or event model. Turnstone's optional Diagnostics rows are the first
built-in subtree promoted here: producer IDs, actual retained bounds, clipping
and read-only roles must pass composed-tree tests and native restart checks.
Those rows do not qualify Downloads, minimap or other built-in action routes.

The next bounded surface candidate is Inspector's viewer/handling controls.
Its existing followed-member authority and mutation path must supply semantic
identity, retained bounds and enabled actions to both automation and AT.
Switching the followed member or replacing its source must reject old requests.
Trail and active Gloss controls follow under their own product owners. Custom
paint interiors need meaningful child projections beyond Sprigging's current
single-node fallback. Chrome menus, forms and review surfaces need actual
action-route and focus checks, not just named regions.

The Inspector review found an existing Clip ownership mismatch:
`turnstone/src/shell/render.rs` checks availability against the followed member,
while `InspectorIntent::ClipToKnot` carries no member and both pointer drains
call the globally focused-document helper. Before exposing Clip through shared
semantics, bind the intent to its displayed member and revalidate the source at
dispatch, or leave Clip out of the initial semantic route. Mark has been asked
which next-slice scope he prefers; this finding is not a completed repair.
Viewer overrides have a second ownership seam: `app/node_arms.rs` writes the
member-keyed sidecar but finds the live URL in the active legacy graph. A
cross-graph follower therefore needs its own runtime lookup and respawn check.
Rows without a live member must expose disabled/no-action state. Inspector
keyboard delivery is currently absent; keyboard parity requires real runner
focus/key plumbing and tests, rather than merely focusable radio roles.

Live documents need Turnstone to consume document-session-api's engine-supplied
neutral projections and revision-guarded nonpointer actions. Genet's Livery,
scripted and reader capabilities differ; a mirrored outline cannot establish
full document accessibility or automation. Within-document selection inspection
remains an Inspector gate. Each unavailable capability must remain explicit.

Registry catalog growth and producer allocation are outside the new bounded
ingress contract. Installing a product pipeline, adding other real workers and
supplying missing operation/semantic references require separate owner gates.
Turnstone's custom compositor currently captures composed layer pixels without
a sealed product/frame token. Its finite next gate is an immutable reading at
the actual render boundary, checked run/request/frame identity through readback,
and deliberate failure, missing/mismatched token and delayed-readback controls.
A failed requested capture must propagate into the scenario result; the current
render path computes `capture_composed` success without applying that result.
The current native scenarios qualify behavior and visual output, not a causal
or diagnostic snapshot paired to those pixels.

Compilation of a Wasm host is not browser-runtime acceptance. Queue presentation
and readback do not establish compositor or physical-display acknowledgement.
Human Narrator/VoiceOver/Orca observations remain necessary for human acceptance.

---

## Historical June design and implementation record

The following text records the retired Meerkat host and its June priorities.
Its status, ownership labels, future work and test claims are historical, not the
current plan. Preserve it as evidence; use the September design above for new work.

**Date**: 2026-06-08
**Status**: Planning. Follow-on to the apparatus pane/theme switcher pass.
**Related**: [apparatus pane + runtime theme switcher](../../archive_docs/2026-10-06_completed_plans/2026-06-08_apparatus_pane_and_theme_switcher_plan.md), [frame tree in meerkat](../../archive_docs/2026-06-09_completed_plans/2026-06-08_frame_tree_in_meerkat_plan.md), [peripheral panes architecture](../technical_architecture/2026-06-06_peripheral_panes_architecture.md), [Graphshell harvest brief](../research/2026-05-17_graphshell_harvest_brief.md), [Graphshell docs full harvest](../research/2026-05-27_graphshell_docs_full_harvest.md), [spatial chrome IR brief](../../archive_docs/2026-06-09_pivot_superseded/2026-05-15_spatial_chrome_ir_brief.md), [spatial chrome modular adoption plan](../../archive_docs/2026-06-09_pivot_superseded/2026-05-15_spatial_chrome_modular_adoption_plan.md).

Build one host observability spine for **diagnostics, tracing, UX events,
UxTree/accessibility, probes, and agent harnesses**. The near-term user surface is
the **Apparatus** pane. The long-term consumers are tests, OS accessibility
bridges, and a Hermes/Burn-style agent harness that can drive the application
through typed observations and actions rather than raw pixel puppetry.

---

## Findings

### What is already live enough to use

- `crates/system/ux-events` is the right UX-event seam. It already defines
  observer/probe hooks, apparatus-facing diagnostics bridging, command-surface
  telemetry, and a stable place for user-facing event semantics distinct from
  low-level tracing.
- `registry::diagnostics` (`crates/system/registry/src/diagnostics.rs`) is useful as the channel catalog,
  descriptor/config layer, sampling policy, invariant store, and portable emit
  scaffold. It should configure and classify diagnostics; it should not become
  the live UI state store.
- `crates/forme/uxtree` already produces deterministic accessibility-shaped
  trees and can stitch subtrees into a `TreeUpdate`. This is the bridge between
  Mere's semantic surfaces and AccessKit.
- `crates/shell/frame` *(historical citation)* <!-- doc-audit: historical-path --> now gives meerkat a real pane tree. Its projected leaves
  are the natural unit for diagnostics, focus, accessibility bounds, and agent
  action targets.
- `crates/domain/apparatus` is a useful domain skeleton for an
  accessibility projection, but the current rendered pane belongs in `meerkat`
  like roster/gloss/comms. The domain crate should not be mistaken for a full UI.
- `crates/shell/chrome` still has valuable Graphshell-era shell semantics:
  toolbar, omnibar, command palette, focus authorities, host intents, routing
  hints, and the eventual `project_chrome(state) -> UxTree` direction.
- `registry::input` (`crates/system/registry/src/input.rs`) has useful action/binding/conflict
  vocabulary. The current keymap is in meerkat, but the registry gives Apparatus
  and Settings a future shape for shortcuts, command discoverability, and conflict
  diagnostics.
- The Graphshell harvest docs are useful for policy language: capability
  declaration, non-silent a11y degradation, schema-first diagnostics, watchdog
  invariants, per-frame UxTree snapshots, focus-region state, and probeable UX
  contracts.

### What should not be revived

- The old substrate-as-host plan is not the current plan. Genet/meerkat owns the
  host path now; Graphshell's substrate material is a source of concepts and OS
  plumbing warnings, not a host mandate.
- `register-viewer` / `register-renderer-types` should not become the canonical
  route selector for current meerkat content. The live content route is host
  dispatch plus `inker`/content actors. Renderer-registry events are still useful
  as diagnostic vocabulary once a second engine lane exists.
- The diagnostics channel catalog is too broad to expose wholesale. Wire a small
  live subset first, then let orphan-channel reporting tell us what deserves a
  descriptor.
- Apparatus should not swallow Inspector, Gloss, Steward, or Settings. **Axis
  refined 2026-06-09** (see [Alembic memory + engrams](../technical_architecture/2026-06-09_alembic_memory_and_engrams.md)
  §8): Apparatus is the *at-rest* plane (system diagnostics, the recorded trace read
  after the fact, health snapshot, invariant violations, plus system/subsystem
  config), Steward is the *live* plane (the process/daemon monitor: running actors
  and async jobs you can act on). So live actor lifecycle belongs to Steward;
  Apparatus keeps the recorded actor faults + health snapshot + config. The
  node's graph facets/provenance/lineage belong to Roster at node scope; addressed
  content identity/provenance/parse state belongs to Inspector; graph commentary
  belongs to Gloss.

---

## Architecture

Add a bounded `HostObservability` store owned by `meerkat::App`.

It should collect:

- `DiagnosticRecord`: registry channel, severity, payload summary, span id,
  source, timestamp.
- `TraceRecord`: tracing target/name/level, active span, elapsed time, optional
  actor or pane id.
- `UxRecord`: semantic user event from `ux-events`, including surface, command,
  focus, pane, and result when known.
- `ActorRecord`: lifecycle events for fetch/sync/comms/content/agent actors.
- `ProbeRecord`: UX or invariant probe results, including pass/fail/degraded and
  the surface that failed.
- `A11ySnapshot`: root tree metadata, focus node, node count, missing label
  count, missing bounds count, and per-surface capability state.
- `AgentObservation`: compact, typed state for harnesses: focused surface,
  visible panes, enabled actions, selected graph node, pending diagnostics,
  current modal/palette, and accessible action targets.

Sources:

1. `tracing` subscriber layer for low-level spans/events.
2. `registry::diagnostics::emit` global sender for structured diagnostic events.
3. `ux-events` observers and `UxChannelObserver` for semantic UI events.
4. Meerkat actor inbox drains in `user_event` for actor lifecycle and faults.
5. Frame-tree/layout rebuilds for pane bounds, focus targets, and accessibility
   surface summaries.
6. UxTree projection passes after layout/render state changes.

Sinks:

1. Apparatus pane sections: Overview, Events, Tracing, Probes, Accessibility,
   Agent. (Live **Actors** lifecycle moved to the Steward pane per the 2026-06-09
   axis refinement; Apparatus keeps recorded actor faults, not the live monitor.)
2. Headless tests and smoke probes.
3. OS AccessKit adapter once internal tree quality is stable.
4. Hermes/Burn agent harness API.

The store is an observation cache only. It must never become the authority for
graph truth, frame layout, actor state, command dispatch, or accessibility
semantics.

---

## Initial Channel Set

Start with a small current-channel map rather than the full donor catalog:

- `meerkat.startup.started`, `meerkat.startup.succeeded`,
  `meerkat.startup.failed`
- `meerkat.frame.layout_changed`, `meerkat.frame.pane_summoned`,
  `meerkat.frame.pane_closed`, `meerkat.frame.divider_dragged`
- `meerkat.ui.action_dispatched`, `meerkat.ui.surface_opened`,
  `meerkat.ui.surface_dismissed`, `meerkat.ui.focus_changed`
- `meerkat.theme.activated`, mapped to the existing theme registry language
- `meerkat.actor.fetch.started/succeeded/failed`
- `meerkat.actor.sync.started/succeeded/failed`
- `meerkat.actor.comms.started/succeeded/failed`
- `meerkat.actor.content.respawned`, `meerkat.actor.content.failed`
- `meerkat.a11y.tree_built`, `meerkat.a11y.bounds_missing`,
  `meerkat.a11y.label_missing`, `meerkat.a11y.focus_missing`
- `meerkat.probe.failed`, `meerkat.probe.degraded`
- `meerkat.agent.spawned`, `meerkat.agent.intent_dropped`,
  `meerkat.agent.action_applied`

Use the Graphshell convention that long-running work emits
`started -> succeeded | failed`, with descriptors declaring timeout invariants.

---

## Phases

### D0 - inventory and mapping

- Map current meerkat events, panes, actors, and commands to the initial channel
  set above.
- Identify which donor channels are directly reusable, which need `meerkat.*`
  names, and which stay latent.
- Record accessibility capability state per surface: orrery, workbench, roster,
  apparatus, gloss, comms, modal/palette.

Done when the mapping is small enough to implement without a registry migration
and explicit enough that new channels do not appear ad hoc.

### D1 - HostObservability store and apparatus upgrade

- Add `meerkat/src/observability.rs` with bounded rings and summary counters.
- Feed current apparatus diagnostics from that store instead of hand-building a
  four-row snapshot.
- Expand Apparatus into sections: Overview, Events, Probes, Accessibility, Agent.
  (The live **Actors** view belongs to Steward per the 2026-06-09 refinement;
  Apparatus shows recorded actor faults, not the live monitor.)

Done when the current theme/apparatus pane shows live recent events and actor
state without changing app authority boundaries.

### D2 - UX events and probes

- Wire command palette, pane summon/close, theme switching, roster selection,
  frame divider drag, destructive confirmations, and focus changes through
  `ux-events`.
- Add `UxProbe`s for the first real risks: modal overlap, missing destructive
  confirmation, focus lost after pane close, and unavailable action shown as
  enabled.
- Bridge UX events into diagnostics through `UxChannelObserver`.

Done when common interactions produce both semantic UX records and diagnostic
records, and at least two probes fail in tests when the guard is deliberately
broken.

### D3 - diagnostics registry bridge

- Instantiate a diagnostics registry/config in meerkat and register the initial
  channel set.
- Install the global diagnostic sender and route emitted records into
  `HostObservability`.
- Surface descriptor state, sampling, orphan channels, and invariant violations
  in Apparatus.

Done when a missing `succeeded/failed` event after a `started` event becomes a
  visible invariant warning.

### D4 - tracing ring

- Add a tracing layer that records selected `meerkat`, actor, frame, render,
  diagnostics, and a11y spans into the bounded trace ring.
- Keep verbose targets opt-in and sampled.
- Link trace records to diagnostic records when span ids are available.

Done when Apparatus can answer "what just happened?" for a pane action or actor
failure without requiring terminal logs.

### D5 - UxTree snapshot and internal a11y audit

- Build one current app UxTree snapshot by stitching chrome/frame/pane/content
  roots after layout.
- Ensure every frame leaf has stable id, role, label, bounds, focusability, and
  enabled action metadata.
- Add an internal a11y audit pass: missing label, missing bounds, duplicate id,
  invisible focused node, action without command route.

Done when Apparatus can show tree health and focused node state, and tests can
  assert the tree contains root/window/frame/pane/content nodes with stable ids.

### D6 - OS AccessKit bridge

- Only after D5, emit AccessKit `TreeUpdate`s for the host window.
- Wire focus changes from app state to AccessKit focus.
- Preserve non-silent degradation: surfaces that cannot emit meaningful a11y must
  declare degraded/unavailable with an owner and exit criterion.

Done when a Windows screen reader can traverse the shell/panes at least as named
  regions, and unavailable surfaces are reported explicitly rather than silently.

### D7 - agent harness

- Define `AgentObservation` and `AgentAction` as typed data, not screenshot
  prompts: open pane, invoke command, select node, activate focused action,
  set theme, drag divider by semantic target, request content preview.
- Expose a harness loop that can read an observation, apply one action, and
  receive the resulting observation plus diagnostics.
- Let Hermes/Burn or a Gemma-class model sit behind that typed loop.

Done when a test agent can open Apparatus, switch themes, summon Roster, select a
  node, and report any a11y/probe failures without coordinate scripting.

### D8 - inspector/steward split

- Move content-level provenance, trust, parse diagnostics, and document structure
  into Inspector.
- Move async operation management and retries into Steward.
- Keep Apparatus as the aggregate system trace and health pane.

Done when Apparatus remains legible under real browsing load instead of becoming
  an undifferentiated debug dump.

---

## Agent Readiness Answer

A Gemma-class model could plausibly puppet the application **after D5-D7**, if
the harness exposes a typed observation/action protocol. It should not be asked
to operate the app through raw pixels first. The model needs:

- a compact tree of visible surfaces and enabled actions;
- stable ids for panes, commands, nodes, and focus targets;
- recent diagnostics and probe failures;
- clear action results, including blocked/degraded reasons;
- a small action vocabulary with no hidden side effects.

Without that, it will be brittle: it may click the right pixels in a demo, but it
will not understand pane ownership, disabled commands, focus traps, or degraded
accessibility states.

---

## First Slice

1. Add `meerkat/src/observability.rs` with bounded rings for diagnostics, UX
   events, traces, probes, actors, and a11y summaries.
2. Replace `apparatus_diagnostics()` with an observability snapshot and render
   recent events in the Apparatus pane.
3. Emit `ux-events` for theme switch, apparatus summon, roster summon, pane close,
   frame divider drag, and focus changes.
4. Register the first `meerkat.*` diagnostic descriptors and install the sender.
5. Add tests for:
   - theme switch emits UX + diagnostic records;
   - apparatus summon/close preserves focus;
   - missing label/bounds appears in the a11y audit;
   - `started` without terminal event trips an invariant.

---

## Progress

- 2026-06-08: Plan written. Grounded in the live `meerkat`, `ux-events`,
  `registry::diagnostics`, `uxtree`, `frame`, `chrome`, `registry::input`, and
  Graphshell harvest docs. No code yet.
- 2026-06-08: **D1/D2 seed landed.** Added `meerkat::observability` as a bounded
  host-local observation cache, expanded the shared `SurfaceId` vocabulary for
  Roster / Gloss / Apparatus / Comms / Workbench panes, and wired pane
  open/close events through `ux-events` into diagnostic records. Apparatus now
  renders Overview, UX Events, Actors, Accessibility, Diagnostics, and Probes
  sections from the snapshot. Actor events are recorded from the kernel inbox
  drain for fetch, sync, content respawns, and comms updates. A coarse a11y
  summary records visible surfaces and explicitly marks the OS AccessKit bridge
  degraded until the real bridge lands. Full `registry::diagnostics` descriptor
  registry, tracing layer, and probe execution remain D3/D4 follow-ons.
- 2026-06-08: **D3 landed + D4 seed.** `HostObservability` now owns a local
  `DiagnosticsRegistry`, registers the first `meerkat.*` and pane UX channel
  descriptors, and runs every diagnostic through registry sampling/orphan
  tracking/invariant observation before it enters the Apparatus ring. Startup,
  fetch, and comms use the `started -> succeeded | failed` convention, with
  completion invariants registered for startup/fetch/comms. Apparatus now shows a
  Registry section (registered count, orphan channels, invariant violations) and
  a Tracing section. `meerkat` installs the portable
  `register_diagnostics::emit` sender and drains `DiagnosticEvent`s into the same
  cache, so extracted registries can feed Apparatus without depending on meerkat.
  Remaining D4 work is a real `tracing` subscriber layer over host spans.
- 2026-06-08: **D4 landed + D5 internal seed.** `meerkat` now installs a
  `tracing-subscriber` layer that mirrors selected `meerkat`, `frame`, and
  `uxtree` spans/events into the same portable diagnostics receiver used by
  `register_diagnostics::emit`, so Apparatus can show recent host trace activity
  without tailing terminal logs. The a11y refresh now builds an internal
  AccessKit-shaped `uxtree` snapshot by stitching a host window root, a chrome
  root, and the live `frame::project_frame` subtree. Apparatus reports tree root,
  focus node, node count, missing label/description count, missing bounds count,
  duplicate ids, and audit findings. The OS bridge remains explicitly degraded;
  the next D5/D6 work is attaching real bounds/content subtrees and then pushing
  `TreeUpdate`s through an AccessKit platform adapter.
- 2026-06-08: **D5 bounds/content slice landed.** The internal a11y snapshot now
  uses `frame::project_frame_with` to attach available domain subtrees under
  frame leaves: `mere-orrery` for the graph pane, `workbench` for the tiled
  workbench, the apparatus skeleton for Apparatus/System panes, and stable
  generic roots for Roster/Gloss/Comms/Tile/Custom panes. Meerkat stamps the
  host-computed frame root, pane leaf, and pane-content-root bounds into the
  AccessKit-shaped tree after projection, preserving the frame crate's
  geometry-free ownership. Focus now resolves to the active frame leaf when
  chrome does not own focus. Unit tests cover frame leaf stable ids, host bounds
  attachment, and a11y audit focus/bounds failures. Remaining D5 work is richer
  descendant bounds and pane-specific subtrees for Roster/Comms/Gloss content.
- 2026-06-08: **D5 pane subtree slice landed.** Roster, Gloss, and Comms now
  project pane-specific internal `UxTree` subtrees instead of generic content
  roots. Roster exposes member rows as list items and reuses row hit-test bounds
  when the host has rendered them; Gloss exposes graph nodes as link-like items
  with URL values and focused-node state; Comms exposes conversation list items,
  selected thread messages, and the draft text input. This gives Apparatus and
  harness code a real semantic surface to inspect before any OS AccessKit bridge
  is enabled. Remaining D5/D6 work is making descendant bounds stable across the
  first render for every pane type, then emitting platform `TreeUpdate`s through
  an AccessKit adapter.
- 2026-06-09: **D6 Windows AccessKit bridge landed.** Meerkat now factors the
  internal a11y projection into a reusable `TreeUpdate` path and installs a
  Windows `accesskit_windows::SubclassingAdapter` before the winit window is
  shown. The same stitched `uxtree` snapshot feeds Apparatus and the OS bridge;
  render, resize, and focus changes refresh the tree and push updates when the
  adapter is active. Non-Windows platforms remain explicitly degraded for this
  slice rather than silently pretending to have an OS bridge. Remaining D6 work
  is richer action handling, platform bridges beyond Windows, and visual/manual
  screen-reader verification.
- 2026-06-09: **D7 typed agent harness landed.** Meerkat now has a feature-gated
  in-process `agent_harness` module with typed `AgentObservation`,
  `AgentAction`, and one-step `AgentStep` results. The harness reads visible
  frame surfaces, active theme, focused node, enabled action descriptors,
  diagnostics/probes, and the same a11y snapshot Apparatus sees, then routes
  actions through existing host methods for opening panes, switching themes,
  invoking commands, selecting nodes by URL, and requesting focused-node
  previews. Blocked actions emit `meerkat.agent.intent_dropped`; applied actions
  emit `meerkat.agent.action_applied`. The first tests cover opening Apparatus,
  switching themes, summoning Roster/Comms, selecting nodes, and reporting
  blocked selections without coordinate scripting. Remaining D7 work is a stable
  external transport for Hermes/Burn, richer semantic divider targets, and a
  narrower public schema once the action vocabulary settles.
- 2026-06-09: **D6 AccessKit action routing landed.** The Windows bridge now
  queues `ActionRequest`s and wakes the winit loop instead of dropping them.
  Meerkat drains those requests on the kernel thread, resolves target node ids
  through a host-owned action route table generated with the a11y tree, and maps
  graph/gloss links plus roster rows to semantic node selection. Routed
  `Click`/`Focus` actions record `meerkat.agent.action_applied`; unsupported or
  stale target ids record `meerkat.agent.intent_dropped`. Remaining D6 work is
  platform bridges beyond Windows and real manual screen-reader verification.
- 2026-06-09: **D6 non-Windows bridge scaffolding and verification checklist
  landed.** Meerkat now depends on the workspace `accesskit_macos` and
  `accesskit_unix` adapters and installs the macOS AppKit subclass adapter or
  Unix AT-SPI adapter behind the same host-local snapshot/action queue used on
  Windows. Focus updates are forwarded through the platform bridge where the
  adapter exposes that hook. The manual screen-reader pass is captured in
  [2026-06-09_accesskit_screen_reader_verification.md](../../archive_docs/2026-09-02_retired_plans/2026-06-09_accesskit_screen_reader_verification.md):
  Narrator, VoiceOver, and Orca still need real local runs before D6 can be
  called screen-reader verified.
- 2026-06-09: **D8 pane split seed landed.** `frame::PaneContent` now has
  first-class `Inspector` and `Steward` variants, with matching UX surface ids,
  diagnostics channels, AccessKit descriptors, command-palette verbs, agent
  actions, a11y projection labels, and simple rendered placeholder panes.
  Inspector currently summarizes the focused node, node count, and content
  state; Steward summarizes active actors, sync chip state, and tab cap.
  Remaining D8 work is moving real content provenance/trust/parse/document
  structure into Inspector and real async operation retry/cancel/pin controls
  into Steward while keeping Apparatus as the at-rest aggregate trace/config pane.
- 2026-06-09: **D8 Inspector content + Steward operation pass landed.**
  Inspector now reads the focused graph node identity, durable import
  provenance, classifications, pinned/compat/viewer metadata, fetch state,
  content type/body size, parser lane, trust/provenance, parse diagnostics,
  outgoing links, and document-structure counts. Nematic-routed content is
  inspected through `EngineDocument`; HTML reports a Genet lane structure
  summary until Genet exposes a full document semantic tree. Steward now lists
  live content operations from the constellation and exposes host-routed retry,
  stop, and background-pin hooks through command-palette verbs and typed agent
  actions. Remaining D8 work is making those hooks clickable in the Steward pane
  UI itself, adding richer per-operation history/error affordances, and migrating
  node facets / lineage out of the focused-node Inspector into the Roster's
  node-scoped facet view.


## Boundary consolidation continuation, 2026-10-19

The [family-promotion boundary scope](2026-10-19_family_promotion_plan.md#boundary-consolidation-scope-2026-10-19)
records the execution/shutdown consolidation, shared observation/semantic-action
follow-up and separate Castellan process design. This plan retains its domain
implementation and acceptance gates. The new scope does not change existing
custody, event meanings, configured limits or action admission, and, at the scoping checkpoint, supplied no
new runtime, process-isolation or human accessibility receipt.

Execution/shutdown qualification (2026-10-19): the [owning continuation](2026-10-19_family_promotion_plan.md#executionshutdown-qualification-2026-10-19) records the shared mechanics and native receiving gates. Observation/action and custody-process gates remain open in their respective lanes.
