# Full-Source Audit Plan

**Date:** 2026-10-16 (record extracted from the scenograph editor plan on 2026-10-17)
**Status (2026-10-17):** landed as an audit. The full-source review of Mere is complete at
working HEAD `7c718105` plus the preserved shared working changes: 3,738 files, 1,043,171
lines, 39,556,704 characters, every file read contiguously against its SHA-256. The registers
hold findings Q001 to Q465 (24 repaired, the rest documented with evidence and limits),
authority entries A001 to A1536, 212 unconfirmed candidates and 23 ruled-out concerns.
At closeout 23 of the 24 repairs were uncommitted in the shared checkout; ruling R1 below
governs their landing. Repairs of the open findings and wider runtime qualification are
follow-up work, not phases of this plan.
**Scope:** the whole-Mere read-only source audit for duplicated authority and misleading
success, with the contained repairs made under the one-hour diversion limit. Evidence lives
in [`support/receipts/authority-success-audit/`](../../../support/receipts/authority-success-audit/):
the [completion receipt](../../../support/receipts/authority-success-audit/audit_completion_receipt.json),
the [findings register](../../../support/receipts/authority-success-audit/review_notes.json) and
the [authority map](../../../support/receipts/authority-success-audit/authority_map.json).

## 1. Rulings

Put to Mark on 2026-10-17 after a fresh read of the closeout, the registers and three
findings checked against source (Q453, Q455, Q465, all confirmed).

### R1. The uncommitted repair batch (2026-10-17)

Question as put: the 23 recorded repairs sit uncommitted in the shared mere checkout
(33 files, +1,574/-168, idle since 2026-10-16 18:13, gated on `b567660d` before a
16-commit fast-forward). What should happen to them?

Options: (a) re-gate and sweep in: run check and tests for the touched crates
sequentially on the shared target, then commit the whole tree with the doc addenda as
one sweep on main and push under standing permission, any red gate coming back first;
(b) re-gate, commit per lane (custody; chartulary and graph-kernel; scenograph and
scenomise); (c) leave it dirty for the owning sessions.

Mark: **"Re-gate and sweep in (Recommended)"**.

What follows: the gates are the audit's own recorded commands (chartulary plus mere-kernel
with `store`, pandect, castellan with `keeper`, scenograph default and `scripts`, scenomise
with `authoring`, script-rhai, graphshell-client all-features, the djinn `h4_identity_receipt`
bin check) rerun on the fast-forwarded base. One commit on main carries the repairs, their
three new test files and the five doc addenda. The graphshell-client all-features gate has
one pre-existing exact-byte failure (Q073) that is not this batch's. *Reading, not ruled:*
the receipts' `-j 1` is not required when the machine is otherwise idle, and `--offline`
cannot be kept because the fast-forward repinned genet to a revision absent from the local
git cache.

### R2. Where the audit record lives (2026-10-17)

Question as put: the 3,090-line audit record lives inside the scenograph editor plan, and
the completion receipt binds that path and SHA-256. Where should it live?

Options: (a) leave in place and add a pointer plan; (b) extract to its own plan and record
the move in a successor receipt, accepting that every existing receipt binding to the old
path and hash becomes historical; (c) leave as is.

Mark: **"Extract to its own plan"**.

What follows: this file. The record below is moved byte for byte from
`2026-10-07_scenograph_editor_plan.md` lines 2394 to 5481 as committed in `00db7399`;
its relative receipt links are unchanged because both plans sit in the same directory.
The [relocation receipt](../../../support/receipts/authority-success-audit/audit_record_relocation_receipt.json)
records both files' hashes before and after and the hash of the moved region. The
receipts that name the old path describe the record's location at the time they were
sealed.

### R3. Landing with main red (2026-10-17)

Question as put: re-gate verdict: the batch adds no red. 1,665 test executions passed
across the nine recorded gates. The three failures (graphshell-client Q073, two scenomise
backdrop tests, castellan's residue harness) all reproduce on clean HEAD in a separate
worktree with no batch applied, so they are main's. Castellan's is the notable one: its
files are byte-identical to the audit's green gate, and the only relevant change since is
upstream's 1.99.0 to 1.99.1 toolchain bump. Disk is at 497 GB free after the worktree and
build dir were removed, just under the 500 GB line. Land the sweep now?

Options: (a) land now, one sweep commit on main, pushed, the two main reds becoming
follow-up cards; (b) hold until main is green; (c) land after a 1.99.0 control build of
the residue harness to pin the toolchain as the cause.

Mark: **"Land now (Recommended)"**.

What follows: the sweep commit that carries this entry. The two main reds are recorded in
the [re-gate receipt](../../../support/receipts/authority-success-audit/repair_batch_regate_receipt.json)
and in the Progress entry below, and are follow-up work in their own sessions, not this
plan's. *Reading, not ruled:* the 1.99.0 control is worth running as the first step of the
castellan follow-up, since it is the cheapest way to separate a compiler change from a
latent zeroization gap in `switch_profile`.

### R4. The castellan residue fix (2026-10-18)

Question as put: the 1.99.0 control is red too, so the toolchain was never the cause. The
leaking block is the one-element `Vec<SlotSummary>` that `public_profile_of` builds in
`switch_profile`; its slot key's `Option<String>` is `None`, and the 16 uninitialised bytes
beside the `None` marker carry a stale master-seed copy from a stack temporary on the load
path, probably `Ed25519Keypair::from_seed` taking the seed by value. Stack residue through
padding, layout-dependent. Which fix?

Options: (a) B now, A logged against dramatis: build the slot summary so no uninitialised
bytes are copied, castellan only, and ledger a personae change so `from_seed` takes
`&[u8; 32]`; (b) A only, fix at the source in personae; (c) a narrow harness allowance.

Mark: **"B now, A logged against dramatis (Recommended)"**.

What follows: the castellan follow-up session carries out B in castellan and records the
personae change in the dramatis ledger (no issue or PR upstream; ledger first). B alone does
not prove the stack is clean; the plan entry says so.

### R5. Which refusal wins for a duplicated occurrence key (2026-10-18)

Question as put: the selectable-stress test was stale (`4c4ea11d` made syllable marks
selectable on purpose) and is fixed on `fix/scenomise-backdrop-gate` as `d6c82664`. The
remaining red is which refusal wins for a duplicated occurrence key: the early `validate()`
check at path `metre.occurrences` or the post-compile check at path `marks`.
`SceneInputError.path` is a free string; nothing matches on it and no ruling covers it.

Options: (a) keep `metre.occurrences` and fix the test; (b) `marks` wins, drop or reorder
the early check; (c) keep both and make `path` a documented contract.

Mark: **"Keep metre.occurrences, fix the test (Recommended)"**.

What follows: the scenomise follow-up session updates the test on its branch; the later
`marks` check stays for what the input check cannot see. *Reading, not ruled:* `path`
remains a free string.

## 2. Phases

- **A1, full-source read.** Done-condition: every live source file read contiguously
  against its current hash, with findings, authority entries and evidence registers
  reconciled. Met 2026-10-17; the completion receipt lists the six requirements and
  their results (five proved, the main refresh proved with scope).
- **A2, land the contained repairs (R1).** Done-conditions: the recorded gates pass on
  the current base, apart from the pre-existing Q073 failure; one sweep commit on main
  carries the 33 dirty files and three untracked tests; pushed; the Progress log below
  records the gate counts.

## 3. Findings and progress

The record that follows is the audit's own dated log, unchanged. It is the Findings and
Progress sections of this plan. New entries go after its closeout.

---

## Full-source audit and contained repairs, 2026-10-16

**Status:** Scenograph's 14 current source/manifest/example files are fully read.
Scenomise's manifest, arrangement helpers, authoring reducer, disclosed-backdrop model and opened-link
compiler are read; its remaining compiler/solver source is still under review.
The whole Mere audit remains active. Upstream main through `434aa49c` was merged
without conflicts at `4523cb6f`; concurrent dirty work was preserved.

Three defects have bounded repairs (Q070–Q072). Default link layout counted a
portable legacy member and its inspector summary twice: adding the summary
alone reversed Timeline to List. The opened-link compiler similarly turned six
members into seven scene items. Both now use
`LinkBucket::unrepresented_legacy_summaries`: a summary is a fallback row/count
only when its family has no portable member in that bucket. Original inspector
data and summary-only readings remain available; distinct buckets stay distinct.
This establishes one shared compatibility-copy rule for these two consumers,
not one authority for every link inspector or graph writer.

The script runner flattened runtime errors, exhausted budgets and non-bool
returns to false, allowing `Not` or `Any` to apply a failed rule. SE99 already
requires that whole rule to remain unmatched. Errors now survive boolean
composition until the rule boundary; the trace retains the failure reason and
other rules still resolve. Real Rhai tests cover all three failure kinds under
five compositions, plus genuine false negation and a working fallback rule.

Source ownership qualifications are recorded beside the bugs. Public source
revisions are durable provenance; runtime witnesses remain host-verified and
are discarded on recording. `RuntimeVerified` is not replayable freshness.
DynamicsSlot validates a canonical JSON-object envelope, leaving physics terms
to the binding host. Raw public definitions/serializers do not themselves prove
that a compiler or store admitted the definition. Option declarations are shared
through the arrangement helpers, with complete compiler/custom-registry use
still to be reviewed. View attention, seeds and suggestions retain their local
owners; newly merged product persistence is not qualified by these model tests.

The broader Q069 finite-geometry/wire-validity repair remains documented under
the user's one-hour audit-diversion limit. Its done-condition is consistent
finite admission across snapshots, committed diffs, portable captures and traces,
with successful reopening controls and downstream solver coverage. A one-field
patch would leave the misleading portable-success claim intact. Negative and
degenerate geometry policy can remain separate from finite-value rejection.

All three new regressions failed before the repairs. Final gates pass 48 default
Scenograph, 52 script-enabled Scenograph, 38 real Rhai and 226 authoring-enabled
Scenomise test executions. The separate all-feature client gate passes
91 tests, including the remote wording repair (Q066), and fails one
existing exact-byte trace test (Q073). It adds 15 groups of default relation
fields on reserialization, increasing the old trace from 10391 to 11741 bytes;
removing only those groups restores the original bytes exactly. Serializer,
fixture and failing test match merged HEAD. Decoding compatibility is not byte
identity. A versioned canonical-encoding/identity review is deferred under the
one-hour limit; neither serializer defaults nor the golden were changed merely
to turn this gate green. These are native package
receipts; headed browser, wasm and whole-product acceptance are not claimed.
The [audit receipt](../../../support/receipts/authority-success-audit/scenograph_review_receipt.json)
binds source, logs, coverage, deferred work and preservation checks.


### Read-only solver audit continuation, 2026-10-16

The built-in catalog, document/facet composition, realizer and every family
solver are now fully read. Remaining Scenomise host/compiler/history/registry/
template/integration code is still under review; package compilation gives no
reading credit. Production source and prior repairs are unchanged this pass.

Six further findings are recorded (Q074–Q079). Timeline casts its f64 span and
numerator to f32 before division: a finite, round-trippable `[0, 1e100]` score
produces NaN, and its structurally admitted snapshot cannot reopen from JSON.
`[0, 1e-100]` collapses distinct endpoints; `[0, 100]` is the passing control.
This supplies an actual producer for Q069, rather than only injecting invalid
geometry directly at snapshot admission.

Every host-named L-system grammar realizes as Hilbert without an unavailable
result, although Sceno says its rules come from the host. This fallback is
intentional in the existing test; the authored picker remains limited to the
three built-ins. Explicit L-system and Penrose depths both admit 255 for one
item, bypassing their automatic depth caps. A symbol-count recurrence over the
reviewed Hilbert rules yields 3,665,038,759,251 symbols at depth 20. No expansion
string or hazardous solve was executed. Configurable work/output budgets and
bounded refusal need a broader compiler/host pass under the one-hour diversion
rule; accepting a u8 is not resource admission.

Radial checks negativity after rounding: `-1` takes the outer fallback, whereas
`-0.1` lands at the focus. A maximum u32 ring plus an undisclosed item overflows
the unchecked outer-ring increment and panics in the native overflow-checked
library. Release/wasm behavior was not executed. Sets keeps six declared source
memberships in its regions but silently places an impossible six-way member
inside only one region. The sampler relaxes semantic constraints without an
unmet-placement report; source membership itself is not changed.

The authority map distinguishes shared declarations from duplicated parsing:
editor/refusal keys and choice tables are shared, but OptionKind and catalog
Options implement value predicates separately. Their reviewed branches agree;
no parser divergence is alleged. Common scene construction/hold reporting is
real reuse, while usable-weight and rectangle-contour helpers remain repeated.
Stateless solvers are not necessarily one linear pass or bounded work.

Eight native public-API diagnostics pass as observation tests, including the
finite Timeline control. These results reproduce defects; they are not repairs
or product acceptance. The probe executable and PDB were removed. Exact source,
library and diagnostic hashes are in the
[solver audit receipt](../../../support/receipts/authority-success-audit/scenomise_review_receipt.json).


### Read-only complete Scenomise boundary audit, 2026-10-16

All forty Scenomise source files are now fully read at recorded hashes. This
continuation adds nineteen files. Mere-wide coverage is 572/3352 source files;
the full-source audit remains active. Production code is unchanged this pass.

Six further findings Q080–Q085 have native reproductions. History joins compared
relationship text with unescaped control separators: two different permitted
label/explanation pairs produce the same signature, so both the relationship and
its incident occurrences falsely report Stable. Ordinary changes remain a
passing control. Checked history parsing also admits duplicate occurrence IDs
that the compiler rejects; its predecessor map silently retains the last one
and can report all current items Stable. Shared structural admission and typed
comparison are needed at these distinct boundaries.

Relationship routing checks a generation made only from revision text and sorted
occurrence IDs. A valid owner-b dataset routes onto an owner-a projection when
those weak inputs match, despite the documented different-dataset refusal.
Compile itself checks the stronger source binding; this is the later routing
seam. A changed-revision control refuses. This is an in-process provenance and
composition defect, not an executed remote authorization bypass.

A finite accepted units_per_coordinate override of 1e38 with finite x=1e6
produces infinite compiled geometry. Coordinate admission checks nominal spacing
instead of resolved scale, and built-in solve has no shared finite output check.
Definition draft validation does run; this is not a bypass of that validator.
The result extends Q069/Q074 with a real ProjectionCompiler producer.

Relaxation moves item transforms without moving their existing routed polylines.
It also recomputes bounds from centers, discarding a stationary 10x10 footprint
even with dt=0. The same module already computes transformed footprint corners
for collision bounds. Current Rust source search finds no non-test product
caller of this relaxer, so no live Canvas clipping or stale-edge claim follows.
Route ownership, invalidation and consistent content bounds need a named shared
seam; replacing arbitrary routes silently would not be an adequate local fix.

Three other probes remain qualified candidates rather than bugs. A different
custom registry under the same solver ID can reuse previous geometry, but stable
ID and retained-user-placement semantics need a caller contract. Entered focus
uses a visible set beyond Scene fold hiding; the actual inspected Graphshell
view consumes that set correctly. Matrix derived source IDs omit adapter identity
and can alias, while distinct instances and contributors remain; no failing
resolver or action is established.

The package suite passes 226 tests with authoring enabled. Ten native observation
tests pass: six reproduce defects, one controls ordinary history comparison,
and three qualify the boundaries above. Diagnostic success is not a repair or
product acceptance claim. No renderer, network, real store, source writes, wasm
or hazardous expansion was executed. The owned standalone executable/PDB were
removed; stable shared Cargo output was reused. Exact hashes, scope and logs are
in the [boundary audit receipt](../../../support/receipts/authority-success-audit/scenomise_boundary_review_receipt.json).


### Read-only complete Cartography audit, 2026-10-16

All twenty Cartography source/manifest files are now fully read at recorded
hashes. Combined with the Scenomise completion above, this continuation adds
39 fully read files and 482,569 characters. Mere-wide coverage is592/3352 files;
the full-source objective remains active. Production code remains unchanged.

Q086–Q090 document five further defects. AffinityScores permits asymmetric facts
but lookup finds either orientation in one pass, so reverse-first storage wins
over a present forward value. Portable scene lowering ignores ProjectedRegion:
Partition contours/members/labels disappear while a ClusterHalo control remains.
The generic score converter discards valid footprint-aware Scene bounds and
recomputes center-only bounds; a10x10 item becomes0x0 content. Lowering restores
the footprint but copies those zero bounds. Spiral uses a separate converter
which retains the solved bounds. Placement itself is genuinely shared through
Scenomise; this is duplicated, lossy conversion after solving.

ViewIntent.filter has a builder and assigns materialization to Cartography,
yet no implementation reads it. An explicit one-node filter places three nodes
with no fault. Actual Pictograph scoped layout independently builds an induced
graph and remaps stable IDs, so no live scoped-canvas leak is claimed. Partition
and Mosaic also add another plus-one to weight.degree, although the reviewed
Pictograph producer and registry already supply degree+1. A native Partition
probe given1/2/2 matches a direct solve of2/3/3: isolated share changes1/5 to2/8.
Producer/registry/dispatch were inspected as excerpts, not executed or counted
as full Pictograph reading; Mosaic's duplicate offset is source-qualified.

Two native observations remain qualified: raw projection edges default a missing
endpoint to origin, but coverage reports omission and portable lowering drops
the edge; Spiral panics on a stale order key where the generic builder skips
it, but the actual inspected host pairs a registry with its graph. Current
source search finds no non-test scene_from_projection or NodeFilter caller.
No live export, phantom edge, ranking/physics or stale-signal host failure is
alleged. Generic scene identity shifting was also narrowed out: built-in solve
places every ordered item and adapter ordinals/keys remain aligned.

The package root's contract-only/no-integration and sibling-strategy text is
stale against its own manifest and implemented adapters/Pictograph dispatch.
The graph-only dispatch table and its parity checks are actual shared code.
Missing/mistyped channel faults are real; optional Partition/Mosaic weight
fallbacks are weaker than the blanket all-required-channel wording.

All57 Cartography package tests and seven native observation diagnostics pass.
Five diagnostics reproduce defects; two qualify the boundaries above. Passing
diagnostics do not mean those defects are fixed. Bounds/filter/route ownership
decisions remain documented so they do not consume the broader source audit.
The owned executable/PDB were removed; no isolated build/worktree was created.
Exact source hashes and evidence are in the
[Cartography audit receipt](../../../support/receipts/authority-success-audit/cartography_review_receipt.json).


### Read-only Pictograph implementation audit, 2026-10-16

This checkpoint adds96 fully read Pictograph source/manifest files and
1,409,628 characters, bringing Mere coverage to688/3352 files. Pictograph is
96/163 complete. Its immediate canvas implementation files are read; the
Canvas test root/directory, signals and other package files remain unread.
The full-source goal remains active. Production code is unchanged.

Q091-Q101 record eleven further defects. Three consumers still read only the
Surface store after Resource topology was introduced: degree size/height,
pair Unrelate and Isolate selection. Native Resource Cites fixtures demonstrate
each omission; Surface UserGrouped controls show the older path still works.
The shared projected graph owner exists, but adoption remains incomplete.

Camera commands reuse the raw wheel interpreter: explicit Pan zooms with Ctrl
held, and programmatic zoom clears a physically held Ctrl flag. Cancellation
misses self_link_press, so a canceled self-link click still selects on release.
The documented single pause gate also has bypasses: resizing a paused node
starts a backend settle budget and one CPU frame advances its field clock while
Canvas still reports paused. This does not establish visible body movement.
Field deletion and affinity sync have related direct settle calls in source;
AtRest home return owns an intentional budget and needs separate qualification.

The same lifecycle/visibility facts are interpreted differently by separate
derived paths. Retired fields still paint visual coupling halos although force
admission and field picking/extent painting filter lifecycle. Fold boundary
routes ignore canonical link and exact assertion hides that the controller
honors. DynamicsInspection itself reports zero controller rows beside one
visible input row after an exact hide. Its shared exact-topology claim therefore
overstates the actual shared eligibility. Grouped partial-hidden inputs need
explicit representation rather than another independent visibility loop.

palette.rs supplies real shared CSS data, but exported node_color and
node_state_color repeat stale literals. Selected and all three state colors
disagree with the table advertised as the one palette. This is a contained
duplicate-data issue. Camera verbs and cancellation are also contained repair
candidates. Projected-neighborhood semantics, grouped visibility, scoped source
retraction and the pause/backend contract are documented for a broader repair
so they do not prolong the source audit into a redesign.

Shared Scenomise solve, Seiche derive/Physics/AtRest, Numen field evaluation and
ESP Meaning computation are real reuse. The weighted-root negative control
confirms explicit refusal, not silent acceptance. Arrangement weight.degree
and physics mass.degree intentionally have different inputs/offsets; that
distinction is not a duplication defect. Disposable occurrence bodies preserve
their original source references and do not become graph truth.

Larger qualified follow-ups are recorded without promoting unexecuted effects:
Meaning provider vector-count validation; graph replacement gesture/handle
reuse; scoped overlays/minimap membership; full historical presentation copying;
Board stage parameter application; and territory cells growing with unbounded
aspect ratio despite its node cap. Underlay prose still describes committed
position fallback, while missing live positions actually default to origin.
The ordinary main Canvas supplies live positions; no all-origin rendering claim
is made.

All13 native diagnostic cases pass: twelve observations reproduce eleven
defects, and one qualifies refusal. These are observations of unfixed behavior,
not repaired regressions. The initial12-case run had one harness mistake:
retraction retains historical buckets, so the corrected control asserts live
projected rows rather than bucket deletion. Initial evidence is retained.
One CPU frame ran; no whole-package suite, headed UI, GPU feature, network or
store was exercised. The owned executable/PDB were removed and no isolated
target/home/worktree was created. Resume with canvas/tests.rs.

Exact hashes, preservation checks and triage are in the
[Pictograph audit receipt](../../../support/receipts/authority-success-audit/pictograph_review_receipt.json).


### Read-only Pictograph signal review, 2026-10-16

All11 signal source/test files are fully read, alongside the Canvas test root,
affinity/goldens/roles/binding tests, face generator and Vello bridge. This adds
18 files and247,196 characters. Mere coverage is706/3352; Pictograph114/163.
The remaining Canvas and integration tests are still pending, beginning with
bounded_scenes.rs. The full-source audit remains active; production is unchanged.

Q102 proves that an empty authoritative affinity is not inert at runtime.
sync_affinity_force treats force existence as the processed-input flag. An empty
result has no force, so every unchanged frame repeats set(None) and settle(180).
A native paused Canvas keeps exactly180 ticks through400 CPU frames and advances
its field clock to6.666667 seconds. A disabled control stays idle; disabling
affinity allows the remaining budget to drain. The existing inert test checks
only zero force count, which does not cover scheduling. This extends Q097's
one-shot pause bypass with a distinct repeated empty-result lifecycle defect.
The contained repair is a processed-input state independent of force presence,
plus the existing pause gate; the active goal leaves production unchanged.

Q103 proves that three edgeless nodes disclose nonzero spectral coordinates
identical at0 and200 iterations. The c=0 branch skips matrix iteration but
publishes its normalized starting vectors, contradicting the documented empty
map that lets EmbeddingFallback handle absent structure. The registry correctly
caches the incorrect supplied fact once. Empty/single-node controls return
empty; a linked Resource fixture supplies coordinates and correct shared degree
importance. No actual SpectralAdapter fallback pixels or general connected-graph
eigenvector failure was executed. Handle zero structure before publishing facts,
then retain placement fallback ownership in the existing arrangement layer.

Registry centralization is real: structure/visit/URL-grouping and parameters
key separate source facts, and graph-free physics producers serve Canvas, board
and grouped subgraphs. One registry serves one graph; callers must reset on a
swap and provide truthful physics-view revisions. These are explicit caller
preconditions, not hidden runtime identity checks. No graph-identity misuse
was promoted into a product bug. The generic TopologyView and mixed-store/alias
parity tests are implemented sharing. importance.rs still claims exact agreement
with size-by-degree, which Q091's narrower Surface consumer disproves.

The reviewed tests include actual literal placement/digest goldens, one-ULP
hash controls, Resource/Surface parity, refusal cases and seeded/pinned/returning
effect controls. They were read, not freshly run. Binding's byte comparison is
between canonical serializations of typed specs; it does not prove raw input
JSON whitespace/order preservation. The versioned face generator uses shared
Emblem encoding and palette references. VelloSink retains lowering errors and
decode propagates them before returning a Graphic; no silent gradient-success
defect was found in that read module. Vello/GPU execution remains unqualified.

Two native CPU observations pass against the existing Cargo-built libraries,
with unchanged source, build-log and artifact hashes verified. Passing means
the defects were reproduced, not repaired. No package suite, new Cargo build,
headed UI, GPU, network or store was exercised. The owned executable/PDB were
removed; no isolated output/home/worktree remains.

Exact reading coverage, hashes and observations are in the
[signal audit receipt](../../../support/receipts/authority-success-audit/pictograph_signal_review_receipt.json).


### Read-only Canvas geometry and test review, 2026-10-16

Twenty more Canvas test files are fully read, from bounded_scenes through the
Meaning topic fixture:248,295 characters. Mere coverage is726/3352 files and
Pictograph134/163; the full-source goal remains active. Resume at mosaic.rs.
Production is unchanged and no whole-package or ignored release suite was run.

Q104 and Q105 expose separate pose/size calculations for the same displayed node.
With three Surface UserGrouped neighbors, degree height is48px. The CPU body
hit-readback rises24/48/96px at zoom0.5/1/2, but the derived face rectangle stays
on its ground anchor. Height-off controls align. A public pointer click at the
face center misses, while the raised body center selects the node. Body and
picking subtract visual lift; the shared face_rect_at painter/readback helper
does not. The existing face/body test uses a single edgeless unlifted node.
Resource degree omission Q091 was deliberately excluded from this fixture.

With height disabled, graph grammar makes an admitted suggestion body16.2px
wide while its derived face remains25.92px, wider than the body. Turning the
grammar off restores the intended0.72 inset. Body sizing multiplies the grammar
scale; face geometry and decoder/LOD height read the raw node size. The grammar
test checks CSS body width, and the alignment test keeps grammar disabled, so
neither covers their combination. Three native CPU observations reproduce both
defects with zoom/height/grammar controls. Actual raster pixels, headed UI,
GPU, host DOM mode and favicon decoding were not exercised.

The contained corrections belong in resolved visual center/size admission.
A broader shared display-pose seam covering CSS bodies, vector/image faces,
accessibility and input should be documented separately so the audit does not
become a coordinate-system rewrite. Physical collider size and deliberate
minimum hit targets are separate policies. reader.rs currently computes its
rectangle from the ground anchor/raw size too, but accessibility bounds were
not directly probed in this pass and remain a qualified follow-up.

The read test source contains meaningful controls as well as weaker evidence.
Density default checks differ from ignored release bars; its convergence/wander
and candidate-overlay probes are labelled print-only. Explicit thresholds and
rounding rules are recorded, not silently inferred from test names. Experimental
helpers directly install alternate laws, and the finite-budget Canvas home test
explicitly states that no current public path reaches its injected case. This
is not another production scheduling authority. AtRest home return remains an
intentional exception to ordinary settle-trigger admission.

Fold tests compare persisted graph facts after normalizing only the snapshot
wall-clock stamp. Source scrubbing retains a separate live Canvas and checks
cursor/strategy/shared-member placement across arrangements; that does not prove
full refresh-live presentation copying. Gloss has an explicitly independent
lens/selection scope and cache. Meaning tests compare partitions across consumers,
instrument a second-registry run as a control, compare sparse/dense lexical pairs,
and bound sliced completion against whole execution. These tests were read only,
not freshly passed. GPU/model and timing receipts remain separately gated.

One qualified test-output issue remains: grouped rest_on can hit6000 frames,
but its printed line always says rest afterN frames. No capped case was executed;
the separation assertions still compare actual positions. Record the terminal
reason before treating that label as convergence evidence. This is separate
from the two reproduced product geometry defects.

The native probe reused unchanged source/artifact/build-log hashes and its owned
executable/PDB were removed. No isolated target, Cargo home or worktree remains.
Exact evidence and preservation hashes are in the
[geometry audit receipt](../../../support/receipts/authority-success-audit/pictograph_geometry_review_receipt.json).


### Read-only Pictograph source completion, 2026-10-16

All 163 inventoried Pictograph source files are fully read. This continuation
adds 29 files and 317,822 characters, including the remaining Canvas and
feature-gated integration tests. Mere coverage is 755/3352 files and
11,241,938/36,854,820 characters. The repository audit remains active; Seiche
is next. Production is unchanged. Package suites and ignored receipts were
read, not freshly executed.

Q106: both layout_stats overlap implementations retain a fixed 36px diameter.
One native CPU test with four cases reports zero for two 80px bodies 50px
apart and one for two 16px bodies 30px apart; default 36px controls agree with
body geometry. The full and grid counters agree because both copy the same
constant, not because either reads resolved node sizes. The public field
describes node-body diameter without disclosing a default-size proxy. Measure
each body separately to avoid occlusion clipping in node_body_rect, then restore
full scope before reading counts. This establishes a geometric diagnostic
mismatch, not raster overlap or physical-contact truth. The initial probe's
full-body readback assumption was corrected to respect that API contract.

The contained repair is to define the metric and share its resolved radius
inputs. A grid using varying radii also needs an appropriate search range;
changing only the comparison threshold can still miss large bodies. Exact
hulls, visual lift, grammar scale and minimum pointer targets are separate
policies. No default-size Density quality-bar failure is inferred. Broader
geometry unification remains documented so it does not prolong source reading.

Evidence qualifications from the remaining tests:

- node_state's hide round-trip and one physics_registry revision check manually
  insert Surface keys for Resource-owned browse links. They check bookkeeping,
  not disappearance of those actual strokes. relations.rs supplies stronger
  canonical Resource-key disappearance checks, exact assertion filtering and
  source snapshot preservation. The localized test gap is not a claim that
  every hide implementation fails.
- The underlay identity-arrangement test promises byte identity and unchanged
  pixels, but compares node/position tuples, edge count and two bounds. It does
  not compare edge contents, serialized output or pixels. Keep this claim gap
  distinct from the earlier Q073 client failure, which remains open.
- Catalog metadata repeats enum IDs/labels, but parity tests cover the copies.
  Channel tests instrument producer reuse and compare cached/uncached results.
  Raw-slot resolution is explicitly distinct from raw-term execution refusal.
  ThroughView is a test picker facade over DynamicsSpec, not another production
  owner; its admitted partial update plus refusal return is not an atomic
  production rejection receipt.
- Meaning's device receipt has identity, allocation and second-device controls;
  its model/purity run is ignored and needs local artifacts. Physics device
  tests can explicitly skip without an adapter. CPU/GPU partition equality in
  the sweep is printed, while the purity receipt asserts score tolerance.
  Vello headless tests do inspect pixels when enabled. None ran in this pass.
- Restore, exact-scope and Forme tests check real boundary behavior: sidecar
  geometry, pre-frame buffered fitting, preserved restored scores, unresolved
  UUID rosters, explicit empty scopes, local holds and graph-swap cleanup.
  Their independent view authorities should not be mistaken for graph truth.

The native observation reused hash-verified Cargo artifacts. Its owned exe/PDB
were removed; no isolated target, Cargo home or worktree remains. Exact coverage,
preservation hashes and limits are in the
[package completion receipt](../../../support/receipts/authority-success-audit/pictograph_completion_review_receipt.json).


### Read-only Seiche force lifecycle and geometry review, 2026-10-16

Forty of Seiche's 91 inventoried source files are fully read, adding 490,432
characters. This covers the core Simulation, coupling/trace/membership/clock,
fluid and scene adapters, grouped composition, numerical instruments and laws
through Gravity, including the full Orbit diagnostic source. Mere coverage is
795/3352 files and 11,732,370/36,854,820 characters. Resume at laws/hold.rs.
The full-source objective remains active. Production is unchanged.

Q107: removing the last force source does not clear its persistent Rapier
accumulator. After one Boundary tick, clear_forces reports zero registered
forces but speed grows from 15.719 to 30.809 on the next tick. A zero-strength
replacement instead coasts down to 15.090. Removing the last anchor similarly
grows speed from 18.768 to 36.786; its zero-stiffness control decays to 18.018.
tick clears prior user forces only inside the branch for a nonempty installed
force/coupling/affinity/anchor/scene-field source. Removing the last one skips
that reset. These are direct Simulation observations; Canvas Still may retain
Hold or another installed term, so an ordinary UI Still failure is not claimed.
The contained repair concerns source removal and accumulator lifetime. Preserve
gravity/inertia and qualify any external persistent-force owner before choosing
unconditional reset versus explicit removal cleanup.

Q108: RoundedSquare half0/border0 panics on inverted clamp bounds even though
the lowerer promises a positive minimum for zero-size bodies. Ball radius0 and
Square half0 both clamp successfully. The native diagnostic catches the panic;
its passing result means the defect was observed, not that rounded admission
succeeded. Admit the half-extent before deriving border bounds, or define a
typed refusal. Canvas normal footprint controls clamp higher; their ordinary
resize path was not demonstrated to emit this input.

Q109: rigid node resizing and fluid contact use different physical geometry.
A pinned Ball radius80 leaves a radius7 particle at x60 inside the node even
while tangible. A radius4 node wrongly pushes a clear x20 particle to25, as
the default radius18 does. Tangibility-off and default-size controls isolate
the mismatch. The fluid node adapter copies NODE_BODY_RADIUS instead of the
live collider, while scene props already branch on stored shapes. The native
test disables fluid gravity, viscosity and constraint iterations to isolate
actual contact in a tick. A bounded radius repair is separate from a shared
shape reader for graph nodes, scene props, rounded/hull approximations and
deterministic contact ordering. That broader work may exceed the user's
one-hour audit diversion limit and remains documented.

One measured numerical qualification: theta0 does not eliminate minimum-cell
merging in Barnes-Hut. Two bodies 0.2 apart at epsilon0.01 produce force20
with min-cell1, versus the pairwise force5 with min-cell0.01. The primitive
is explicitly approximate; this is not a new default Charge failure. Its
default force cutoff8 exceeds min-cell1. Statements calling theta0 an exact
rung must state that remaining spatial approximation. Existing quality bars
were not rerun or invalidated by this tuned-input observation.

Authority findings also include correct boundaries. Coupling point evaluation
proposes a write. Both trace paths share evaluation and write helpers; traces
capture calls, ignored fixed-body writes, immutable inputs and omitted counts
before integration. Membership distinguishes unavailable from outside and
initial matches from later entry. Completed explicitly means the installed
owner callback returned Ok, not that a durable graph change was verified.
Duplicate named bindings refuse callback authority. Accepted field time is
separate from solver age; reset, pause and read-only probes have explicit
contracts. Reviewed tests include trajectory hashes, cap-off controls,
negative numerical controls and a distinct spectral reference for diffusion.
These test sources were read; their package suites were not freshly executed.

Several source comments retain retired designs: graph-committed positions and
tick write-back, unfinished Barnes-Hut Force wiring, unfinished fluid integration,
and a future real field clock. Current code contradicts those comments. The
old experimental Orbit counter-damping copies are test-only diagnostics with
ruled-law controls, not hidden production owners. Their Recipe::catalog label
still denotes the older Full/no-centring baseline, not the current ruled recipe.

Broader qualified follow-ups remain open: fluid contacts are collected from
HashMaps then resolved sequentially; lagged result validity binds body order
and age but not changed strength/cutoff parameters; empty Density may keep
requesting ticks before a flow can initialize; generic timed CouplingForce
placement has different pause-demand gating than its dedicated slot; and
non-finite numerical readings lack an explicit N-class admission guard.
None was promoted to an executed product failure in this pass.

The current Seiche CPU library built successfully. Five generated native
observations passed; no GPU, renderer, actor execution, full package suite or
ignored Orbit sweep ran. Both owned executables/PDBs were removed. The normal
Mere build cache remains at C:/t/cargo-targets/mere; no isolated target, Cargo
home or worktree was created. Preservation hashes and exact limits are in the
[Seiche first review receipt](../../../support/receipts/authority-success-audit/seiche_first_review_receipt.json).


### Read-only Seiche complete source review, 2026-10-16

All 91 Seiche source files are now fully read: 1,077,251 characters. This
checkpoint adds 51 files and 586,819 characters, completing the remaining laws,
overlays, backend adapters, elapsed/speed handling, portable specs and wire
formats, shared scales, scenes, geometry views and test source. Mere coverage
is 846/3352 files and 12,319,189/36,854,820 characters. The full-source objective
remains active. Production is unchanged. Resume with crates/armillary.

Q110: GravityLocus::tidal promises continuing motion but inherits the default
wants_tick=false. Both inline Physics and the actual actor park at the end of
a two-tick budget. A wrapper delegating the unchanged force calculation while
returning true for demand keeps both paths running. The existing never-settles
test calls Simulation.tick manually, bypassing scheduling. The contained repair
is the tidal demand declaration, with explicit static/zero-drive policy. These
are public runtime observations, not hosted Canvas UI execution.

Q111: empty Density has the opposite fault. apply returns before initializing
its flow, and wants_tick treats missing flow as continuing work. Native inline
Physics keeps stepping for all180 frames with no bodies and budget0. Populated
Density finishes; empty no-force stops after one budgeted tick; Halt stops the
loop. This promotes the previously qualified empty-Density candidate. Distinguish
waiting for input from queued work and re-arm on later sync. No empty-Density
actor or product empty-graph path was exercised. This differs from Q102's host
affinity resettle loop.

Runtime review confirms actual shared should_tick and step_owed helpers. The
inline and actor command surfaces do not constitute separate physics solvers.
Actor elapsed calls drain updates rather than admitting caller time; diagnostic
counts marked inline-only are not evidence an actor has no forces. Snapshot
barriers and evidence mailboxes have explicit freshness/omission boundaries.
Callbacks execute before delivery folding; missing mailbox evidence does not
certify absence of an action. Fresh actor execution is now established for Q110,
superseding only that limitation of the previous checkpoint.

Portable specs derive declarations from actual constructors, cap iterative
depth before recursive derivation and refuse unbound raw execution. Rich wire
round trips include shapes that are deliberately not runnable. The generic
settle loop uses an owner-provided runner and reports StepLimit explicitly.
These correct boundaries must survive cleanup. Raw parameter builders still
cast floats without common admission, may eagerly allocate Density grids before
refusing execution, and may change Anneal's public starting temperature after
its private state was seeded. These are source-qualified follow-ups; no large
allocation or raw runtime was attempted. General admission and allocation
ownership should not prolong this audit beyond the user's one-hour limit.

Test claims need tighter scope. Hold's moving-stop test supplies no actual shove;
Kuramoto checks within-group spread and radius without an inter-group separation
assertion; every-force reproducibility covers12 selected sets rather than fluid,
all overlays or all couplings. GPU tests explicitly print skips and return when
device admission fails. A successful test count therefore cannot itself prove
device execution. They do contain useful device counters, sign-flip and injected
submission-failure controls. None of those package tests was freshly executed.
Host-local contact-to-fact examples prove their own commitment discipline, not
a production durable graph transaction or distributed replay implementation.

The shared force scale deliberately uses a fixed reference configuration. That
is distinct from a live collider geometry authority. Tensor law defaults retain
a small literal copy of CPU defaults; no actual parameter drift was measured.
Softened all-pairs repulsion is explicitly a separate law from NodeExclusion.
Staged O(N squared) GPU timings include upload/readback and do not qualify a
resident crossover or unified host-device reuse. LayoutView really shares edge
geometry helpers, but its circle/radius map remains host-maintained and snapshots
do not carry node radii. Its deterministic-overlap wording omits equal-distance
HashMap ties; source parity tests use separated default bodies only.

Hairier issues remain documented: fluid contact order, lagged answer parameter
identity, generic coupling pause demand and non-finite diagnostic admission.
Three generated demand tests passed with the existing current CPU library;
the actual actors were halted/dropped and the owned executable/PDB removed.
No GPU, renderer, headed UI or full package suite ran. No isolated target,
Cargo home or worktree exists. Evidence and preservation hashes are in the
[Seiche completion receipt](../../../support/receipts/authority-success-audit/seiche_completion_review_receipt.json).


### Read-only Armillary actor and pool review, 2026-10-16

Armillary is fully read: six files,31,662 characters. Mere coverage is now
852/3352 files and12,350,851/36,854,820 characters. Together with Seiche's
completion this continuation adds57 full files and618,481 characters. The
full-source goal remains active; resume at crates/conatus/numen. Production
and concurrent dirty work are preserved.

Q112: Pool.submit checks idle workers without reserving one for its queued job.
Multiple admissions can see the same idle1 before that worker reacquires the
lock, enqueue two actors and notify only the same worker. The first actor then
blocks indefinitely in normal long-lived work while the other remains queued.
This contradicts the module's never-stranded/race-free claim. A bounded native
probe of the unchanged production module reproduced on its first attempt:
workers1, first actor started, second absent for150ms, then second starts when
first is released. A busy-first control grows to two workers and starts both
before either is released. Every submitted job was released and finished.
Repair admission/reservation accounting with one owner of idle transitions;
qualify overspawn and concurrent submission before calling it fixed. This is
a contained concurrency defect, separate from redesigning pool lifecycle.

Q113: source inspection confirms workers are never reclaimed when the final
public Pool handle drops. Detached worker loops retain Arc<Inner> and wait on
a Condvar forever; no stop state, Drop path or notification allows exit.
Repeated creation/use/discard of pools therefore retains resources beyond
each owner's lifetime. No dedicated native leak or host teardown test ran.
The probe process exits after completing its jobs, which ends its idle worker
threads. Public workers() is cumulative spawns, not a live count after panic.
Peak-concurrency language needs that qualification. Shutdown, draining,
long-lived actor cancellation and panic recovery are broader policy work;
document them without extending this audit beyond the user's one-hour limit.

Two other source claims need qualification. Pooled ActorHandle carries no join
handle, so join returns after dropping its command sender rather than waiting
for its actor. A panic exits the lifecycle span but bypasses the actor finished
event below run; span closure is not a normal terminal-success event. Dedicated
join discards the panic Result. These were source-reviewed, not natively probed.
Command true means channel enqueue succeeded; it does not certify application.
Emitter's silent update drop after receiver closure is explicitly documented.

KernelThread correctly makes embedding values !Send/!Sync. It does not designate
one globally authoritative thread or police separately shared state. Hosts own
canonical writes, generation acceptance and request outcome interpretation.
Correlated IDs intentionally do not define applied/rejected states. Preserve
those bounded contracts when removing overbroad boundary-drift claims.

Two generated pool observations passed; the module's own two tests were filtered
out. The executable/PDB was removed. No package, product-host, shutdown or panic
suite ran. No isolated target, Cargo home or worktree was created. Preservation
hashes and exact diagnostic limits are in the
[Armillary review receipt](../../../support/receipts/authority-success-audit/armillary_review_receipt.json).


### Read-only Numen complete source review, 2026-10-16

All16 Numen source files are fully read,150,254 characters, including portable
types, default evaluation, registry/projection, Burn lowering and its tests,
Rhai authoring, responsive-path geometry and its tests/example. Mere coverage
is868/3352 files and12,501,105/36,854,820 characters. The full-source goal remains
active. Production/concurrent work are unchanged. Resume at crates/conatus/nisus.

Q114: the default analytic gradient assumes Gaussian centres and linear normals
are spatially constant although the AST admits arbitrary vector expressions.
Linear(normal=Coord) is x*x+y*y: at(3,4) evaluation is25 but its gradient is(3,4),
half the correct(6,8). Sampling that same registered scalar selects finite
differences and returns(5.9995646,8.000374). Gaussian(center=Coord+(1,0),sigma10)
has constant value0.99501246 but returns gradient(0.009950125,0), versus measured
finite difference(0,0). Fixed-centre and constant-normal controls work. These
two native observations prove value/gradient disagreement and expression-route
dependence. Seiche's physical scalar response calls this evaluator by source;
no fresh physical coupling or hosted Canvas execution is claimed. Burn already
refuses dynamic centre/normal forms instead of applying those closed forms.
The contained repair is analytic eligibility with finite-difference fallback.
A general shared derivative/AST-analysis design is a separate larger issue.

Q115: explicit canonical IDs and named local allocation collide. insert_with_id
does not reserve against next_id. Seeding valid UUID0 with Const7 then adding
an unrelated named Const9 returns the same id, leaves len1 and changes an
existing Sample from7 to9. Ordinary distinct-name and same-name-update controls
behave correctly. One native test proves the public registry defect; a product
graph overwrite was not exercised. Skip occupied explicit IDs or admit separate
identity domains while preserving same-name updates and canonical graph IDs.
Graph-to-Seiche source seeds explicit IDs; authoring helpers allocate names,
but their coexistence in one live host was not demonstrated.

This package contains real sharing: ast is a reexport, FieldDef aliases the same
FieldDefinition, and the registry keys canonical FieldId. FieldProjection is a
view-local evaluation bundle. The old commit_to_graph bridge is gone; host graph
writes remain host-owned. Those are useful boundaries, not extra durable truth.
CPU missing/wrong-typed samples explicitly evaluate to zero; Burn returns typed
errors. Backend refusal contracts differ and cannot be flattened into one
unqualified success claim. The lowerer accepts caller tensors and mints no device.
WGPU-as-future prose is stale because the opt-in feature and parity tests exist;
that source does not prove current unified host-device integration or fusion.

The current default Numen library built successfully, and all65 default library
tests passed. They include responsive-path limits, recursive sample refusal,
hidden scalar-cycle checks, nested gradient cost, nonfinite controls and geometry
relabel/reversal. All three additional native defect observations also passed.
No Burn/Rhai/WGPU suite, renderer, hosted graph or physical coupling ran. Reading
optional backend test source is not a fresh backend qualification. The generated
diagnostic executable/PDB was removed; normal cache remains at
C:/t/cargo-targets/mere. No isolated target, Cargo home or worktree was created.

Hairier follow-ups remain documented: the responsive path's private cycle/depth/
work validator is not shared by direct CPU/Burn evaluation or every Seiche caller;
public registries can store recursive references. No deliberate stack overflow
was attempted. General admission, late registry edits and consistent diagnostics
can exceed the user's one-hour audit diversion. Rhai construction casts floats
and signed step counts without a common runtime-range gate; script success is
authored-data success only. No malformed Rhai input or field-line runtime was
freshly exercised. Its example even declares a scalar id for vector field-line
tracing, so it should not be presented as an executed rendering receipt.

Preservation hashes, build/suite output and exact diagnostic scope are in the
[Numen review receipt](../../../support/receipts/authority-success-audit/numen_review_receipt.json).


### Read-only Nisus and Tenant complete source review, 2026-10-16

Nisus2/2 and Tenant8/8 source files are fully read,63,975 additional characters.
This continuation completes26 files and214,229 characters including Numen.
Mere coverage is878/3352 files and12,565,080/36,854,820 characters. The full-source
goal remains active. Production/concurrent work are unchanged. Resume with
crates/conatus/conatus, then the remaining adjacent spatial owners.

Q116: LightBlock::is_valid promises every value is finite, but admits all three
light intensity slots with >=0 only. Positive infinity passes. Colors/directions/
positions and point radius have explicit finite checks, making the missing
intensity guards specific. set_lights trusts this result and forwards intensity
to backend lights/ambient. The source test rejects NaN ambient but does not cover
positive infinity. This is source-confirmed validation behavior; no Tenant build,
numeric probe, adapter, frame or shader failure was executed. The contained
repair is finite/non-negative intensity admission before mutation, retaining
the documented unchanged-state refusal contract.

Nisus has a clear shared owner: chunk storage/revision/edit mechanics, with
product-owned durable identity/occupancy and runtime-owned collider refresh.
It stages/validates coordinates before changes, handles duplicate cells in a
stable order, keeps noops at the same revision, and checks revision overflow
before mutation. Material-only changes do not become occupancy edits. Dirt is
excluded from record equality/serialization, and deserialization calls the
validated constructor. All six current library tests pass. These qualify value
mechanics, not product durable saves or collider refresh. Its max_edits check
follows full iterator collection, so an accepted-batch cap is not a work/allocation
cap. That distinction is documented without an infinite or huge input probe.

Tenant contains real host-device plumbing: Context::from_host, caller encoder
and target, forwarded internal-submission report, plain public arrays and wgpu
handles. It does not request a second device in its production wrapper. Pinned
kiss3d behavior owns fallback, rendering, material/animation and lower validation.
Frame success means recorded commands, not queue completion or visible pixels.
The frame test source is substantive: actual rendering/readback, palette and
split-depth controls, zero-submission check and host-device survival. It also
returns successfully with a skip message when adapter/device admission fails.
That test was read, not run here; no fresh GPU or product proof is claimed.

Hairier numeric/ownership issues remain qualified: finite quaternion components
do not ensure usable normalization; camera validation admits finite singular
projections and does not distinguish reflected orthonormal views from proper
rigid rotations; finite nonzero sun direction may normalize poorly at extreme
values. Mesh replacement also retains a former glTF body's animation player
after removing its displayed node. No transform, glTF or replacement probe ran.
Exact pinned-backend behavior and replacement semantics should be resolved in
their own work, within the user's limit on audit diversions. Do not infer absent
downstream prepass validation from a wrapper that delegates it.

The fresh execution total for this continuation is71 library tests (65 Numen,
6 Nisus) plus3 native Numen defect observations. Q114/Q115 are reproduced;
Q116 is source-confirmed. No Tenant execution or full workspace suite ran.
No isolated target, Cargo home or worktree was created; normal Cargo output
uses C:/t/cargo-targets/mere. Preservation and gate details are in the
[Nisus/Tenant receipt](../../../support/receipts/authority-success-audit/nisus_tenant_review_receipt.json).


### Read-only Conatus complete source review, 2026-10-16

All23 Conatus source files are fully read,274,139 characters: body vocabulary,
fixed clock, commands/phases, private Rapier adapter, queries/refresh, resident
allocation/kernel/exclusion/binning code and every test source. Mere coverage
is901/3352 files and12,839,219/36,854,820 characters. The full-source goal remains
active. Production/concurrent work are preserved. Resume at crates/conatus/modulus.

Q117: a system failure consumes unexecuted fixed time and discards publication
for work already applied. With one injected AfterPhysics failure, advance50000us
grants3 ticks, executes1 and returns Err; advance0 has no backlog and never runs
the remaining2. A second observation applies a queued spawn and has a pending
despawn, then fails: live state contains the spawn but its command result/change
and prior removal disappear from both the error and next successful zero-step
frame. No-error controls publish everything and execute3 steps. The primitive
clock correctly retains capped backlog; Engine consumes its grant before work
and loses local aggregates on error. Preserve unused grant and partial evidence.
Retry/resume phases, resource side effects and command outcome recovery need a
larger policy decision; no transactional rollback requirement is invented here.
That work may exceed the user's one-hour audit diversion and stays documented.

Q118: immediate impulses alter BodyState without marking publication dirty.
Native linear/angular impulses produce speeds0.47746482 and1.193662, but advance0
reports changed[]. Setting the identical already-current velocity then publishes
it. Both impulse methods bump revision without dirty_bodies insertion; set_velocity
does insert. This is a contained publication repair, independent of when a later
moving transform happens to be emitted. No renderer/network/save consumer ran.

Q119: finite components can overflow a norm and produce invalid admitted geometry.
Quaternion[f32::MAX,0,0,1] is accepted but public state returns[0,0,0,0]; moderate
nonunit input normalizes correctly. A ray at(0,4,0) through a present unit box hits
at3 units with directions(0,-1,0) and(0,-10,0), but finite(0,-f32::MAX,0) returns
Ok(None). Squared norm becomes infinity, then normalization collapses the data.
Use stable norm arithmetic or refuse unusable derived norms before admission.
Character-up and Tenant share a source pattern; they were not executed here.

The current default Conatus library built successfully.44 existing CPU tests
passed:19 original library tests and25 tests from the four original CPU integration
files, compiled directly with rustc against the current Cargo-built dependencies.
This is an explicit CPU gate, not a Cargo/package/doctest or resident feature gate.
Five generated public-API observations also pass and demonstrate the defects,
not fixes. No resident GPU or renderer was built/executed. Owned executables/PDBs
were removed; normal build cache remains at C:/t/cargo-targets/mere. No isolated
target, Cargo home or worktree was created.

Useful shared ownership is present. Conatus owns generational public IDs and
publication; one private Rapier adapter owns backend handles/conversions. Query
refresh touches colliders/broad-phase without stepping. Fresh CPU tests check
refresh/control trajectories, stale-versus-refreshed queries,25 contact shape
pairs at14 boundary cases, and canonical voxel order across reversed/edited/
carved grids. These bounded successes do not qualify extreme numeric admission.
The optional Resident integrator remains a separate n-body lane, not a current
replacement for BodyWorld's tactile/query/identity contract. Exclusion explicitly
accelerates a Rapier host; Seiche still owns its2D layout runtime.

Resident sharing has concrete source evidence: one CubeCL handle per plane,
raw views retain its lease, and Burn views use that handle. The source test
resolves Burn's actual primitive resource range independently, beyond comparing
two producer-made metadata records. Patch admission checks every range before
writes and explicitly refuses multi-plane restamping. None of these GPU tests
was run here. Adapter/device admission failures return with skip messages, so
a green count alone would not prove execution. Runtime shader compilation also
does not eliminate backend/fallback/completion questions merely because there
is no committed SPIR-V artifact.

Hairier resident issues stay qualified: n-body construction checks CSR length
but lacks complete range/target/zero-input admission; public binning/grid and
parameter arithmetic have different guards; public lease byte_len uses unchecked
multiplication despite the stronger checked plane constructor. ReadEpoch is
documented as host-issued, yet n-body leases use a local step revision and force-only
writes do not advance it. Immutable stamps do not freeze aliased GPU bytes.
No malformed GPU workload, stale consumer or overflow allocation was attempted.
Shared admission and writer/submission/read lifetime policy belong in separate
work. Atomic neighbour scatter also means exact candidate enumeration is not
bitwise GPU replay; current tests use relative error and distinct softened versus
hard-floor/cutoff laws. Existing quality bars are source-read only in this pass.

Exact source/preservation hashes, compiler commands and execution limits are in
the [Conatus review receipt](../../../support/receipts/authority-success-audit/conatus_review_receipt.json).


### Read-only Modulus and root build-source review, 2026-10-16

Modulus12/12 and12 root manifest/build/config/helper sources are fully read,
196,729 additional characters. All152 inventoried source files under
crates/conatus are now complete. Mere coverage is925/3352 files and
13,035,948/36,854,820 characters. Full-source objective remains active.
Production/concurrent work are preserved. Resume at crates/mere/Cargo.toml.

Q120: the public material lookup and traversal reader disagree. A map with
only key[0,0,0] filled7 returns7 at world cells[524288,0,0] and[-524288,0,0].
Nearby outside x8 is air, and public trace at both distant coordinates is
Clear. material_at divides by8 then narrows to i16, wrapping the distant key
to0; the traversal reader bounds pointer coordinates instead. One native test
proves the divergence. Reuse that bounded reader or check before narrowing.
This is a contained removal of duplicated lookup authority; the trace itself
does not inherit this distant alias. No GPU or product pick was executed.

Q121: at direction exactly1e-6 the box clip treats the axis as moving but the
voxel crossing treats it as parallel. Public CPU trace through air toward
solid x4 returns Clear; the adjacent representable direction above it and
unit-direction control hit[4,4,4]. Clip uses abs(d)<threshold, crossing uses
strict signed inequalities. WGSL repeats the mismatch but was not executed.
Align the boundary classification across CPU, WGSL and exact reference, with
signed endpoint/adjacent-float controls. This is a bounded numeric endpoint,
not a demonstrated product-camera or general GPU parity failure.

Q122: the recommended config example's test-web alias applies its historical
incremental workaround to absent package mere-canvas. Current package/lock
name is pictograph. The intended package therefore receives no override.
Active tracked config contains only build-dir; the stale alias is example-only.
Correct targeting is separate from determining whether today's toolchain still
needs that Windows linker workaround. No web alias or historical failure ran.

The Modulus default Cargo gate passes18 unit and8 integration tests, with one
ignored large receipt unrun and zero doctests. Both additional native defect
observations pass with controls. Four pure grouped-exporter Python tests also
pass. CPU traversal is a manually authored WGSL mirror; no actual shader
compilation or GPU execution occurred. The frozen accumulated walk is an
explicit test-only positive control, not another production authority. The
f64 reference is a useful precision instrument under the chosen offset/tie/
budget rules, not independent proof those policies are universally correct.
The ignored large receipt prints tallies without failure assertions; its exit
status alone would not establish whole-frame quality. Default sampled tests
do assert non-vacuous bars and catch the known drifting control.

Retarget/refresh ownership is mostly sound: slots are deterministic, retained
slots survive shrinking, missing payloads refuse before writes, changed
selection needs an advanced projection revision. Refresh can change content
at the same projection revision because source revision remains product-owned.
Do not collapse those two authorities. Atlas capacity is not total memory or
input work: selection collects before count refusal, pointer volume follows
the bounding box, and pointer bytes are outside max_atlas_bytes. No huge
allocation/infinite input was attempted. Total admission/memory policy and
actual GPU parity remain documented larger work within the user's audit limit.

Root declarations have consistent canonical first-party source selectors, but
branch main is not an immutable revision; Cargo.lock owns the selected commit.
The claim every member has a root path alias is false for11 of103 declared
members, though lacking an alias is not itself a runtime bug. Several comments
still describe git-sibling consumption beside current local paths. Inactive
retired vendor patches are provenance, not proof they execute. Those retained
sources remain within full-source coverage, regardless of active selection.

cargo_mode implements real local-lock isolation and portable provenance checks.
Its metadata-only option intentionally does not compile. The fixture suite was
read but not run here because it creates temporary Git repos, Cargo homes and
targets outside approved local workspace locations. No production helper or
fixture was relocated. The exporter honestly discloses declarations among
selected snapshots, including alias/target/optional/build/dev data; it does not
claim a resolved or compiling dependency graph. Rust-policy CI similarly checks
declared MSRV, with compiler lanes separate. CI and full portable verification
were read, not executed or queried for current status.

Tracked build-dir points at workspace-root/target/build; audit build commands
explicitly override it and target-dir to C:/t/cargo-targets/mere. The settings
are separate. Placing formatting or custody policy files does not execute their
gates, and no formatting/custody sweep ran. Owned probe executable/PDB removed;
no isolated target, Cargo home or worktree created. Preservation and exact
execution scopes are in the
[Modulus/manifest review receipt](../../../support/receipts/authority-success-audit/modulus_manifest_review_receipt.json).


### Read-only Mere facade source review, 2026-10-16

All18 inventoried crates/mere source/manifest files are read, adding215,322
characters. Full-source coverage is943/3352 files and13,251,270/36,854,820
characters. The complete repository objective remains active. Resume with
crates/forme/forme/Cargo.toml. Production/concurrent sources remain unchanged.

Q123: outline grouping becomes a lossy identity authority. Two graph nodes
differing by scheme, query or fragment yield only one linked outline row and
one Djot link. Distinct paths retain two. The hand-written URL splitter drops
those distinctions and one trie here slot overwrites the prior member. Gloss
then re-resolves URLs instead of carrying stable IDs, though Canvas selection
was not executed. Preserve every member independently of hierarchy grouping;
full consumer selection migration may exceed the one-hour diversion limit.

Q124: two endpoint pairs with equal titles, each with Cites and Quotes, become
four bundle starts. Labels then relation kind sort before stable endpoint IDs,
interleaving A-Cites,B-Cites,A-Quotes,B-Quotes. Distinct-title control has two
starts. This is a contained grouping correction; exact assertion IDs and
source-owned editability already survive and should remain independent.

Q125: a valid two-definition subgraph sidecar reloads both, but a truncated
copy silently loads as one fresh default. Load leaves corrupt bytes intact;
an explicit subsequent save replaces them, losing recoverable definitions.
The tested path is public load/save, not an automatic host save or crash.
Separate missing first-run state from corrupt/unreadable state and expose a
checked recovery outcome. Host recovery/adoption and crash-safe multi-file
policy are broader work and are documented rather than delaying the audit.

Authority entries A202-A210 distinguish real reuse from competing meanings.
The facade reexports actual owners; its blanket exclusion of render/host
concerns is broader than its optional/default canvas/workbench composition.
QueryView preserves Resource and Surface identity, exact source assertions,
and explicit projection refusals. Saving its spec intentionally need not pass
later appearance/relation budgets. Those budgets do not bound SPARQL runtime.
Subgraph component/ego walks delegate kernel, while other structural kinds
explicitly fall back to the seed. A classifier chip is not proof that a linked
walk for that shape is implemented. Sharing carries specs; freeze re-evaluates,
keeps strata distinct, and delegates frozen Resource ownership to kernel.

StrengthDraft stays a local candidate, with caller graph identity and journal
ownership explicit. Clip similarly reuses apply helpers but ignores their
returns and checks only node existence. No valid-input partial clip was
reproduced, so that remains a candidate. Trail is an explicit-input projection.
Routing app markers reuse Inker policy; matcher/order claims need that owner
read before promotion. No second implementation is inferred from a reexport
or UI label alone.

Fresh Cargo graph-only library gate passes45 tests. Three generated native
public-API observations pass with controls. Query/clip/canvas/workbench sources
and their feature-gated tests are read, not executed by this gate. No default
feature, renderer, GPU, hosted product or UI receipt is claimed. Owned probe
executable/PDB and sidecar fixture are removed; no isolated target, Cargo home
or worktree created. Hash preservation and evidence are in the
[Mere facade review receipt](../../../support/receipts/authority-success-audit/mere_facade_review_receipt.json).


### Read-only complete Forme subtree review, 2026-10-16

All32 inventoried files under crates/forme are read: Forme25, Curation3,
Uxtree4. This adds338,443 characters; repository coverage is975/3352 files
and13,589,713/36,854,820 characters. The full-source goal remains active.
Resume at crates/platen/platen/Cargo.toml. Production and concurrent work
remain unchanged. Authority entries A211-A224 accompany individual findings.

Q126: full parity can mean unobserved. Nonempty parent/child tree with active
and expanded root reports clean/full_match when external topology, expansion,
order and active focus are absent. Explicit topology/active mismatch controls
are detected. Skipping unsupported axes is intentional; reporting those as
matched and claiming full structural/behavioral parity loses coverage truth.
Separate unknown/checked/matched and allowed migration divergence.

Q127: valid parent1 children[2,3] accepts reorder[2,2], reports structural
change and stores[2,2,3]. Its invariant checker panics; a unique-order control
passes. Membership filtering is not deduplication. The property generator
never emits Reorder, despite the any-sequence introduction; those tests were
read but not run. Deduplicate or refuse before mutation and retain omitted
actual children once. This is a contained mutation admission correction.

Q128: failed store save stamps the live document before I/O. A file blocking
the session directory makes save return Err but changes timestamps0 to100.
First successful save at200 persists created100. Stage timestamps with the
candidate and publish them only after replacement succeeds. Native ordinary
failure and later roundtrip are executed; no automatic host save is claimed.

Q129: load_forme(B) accepts valid A JSON in B's filename and returns A. The
matching-ID control succeeds first. load_all_formes also accepts arbitrary
not-a-uuid.json and returns duplicate A records, despite its uuid-only comment.
Check file/document identity and report list refusals explicitly. Expected
graph binding remains caller-owned; no remote or authorization bypass tested.

Q130: Cold root1 with Active child2 yields active2 and a tab/UX Pane for2 but
no pane rectangle. Warm-parent control restores geometry. Geometry prunes
nonvisible ancestors before descent, while legacy accessibility independently
emits every visible member. Share an admitted visible frontier without forcing
ancestors active. Current host adoption and platform accessibility effects are
unproven; policy/consumer migration may exceed the one-hour diversion limit.

Q131: a horizontal member1/2 subtree beside root3 has true boundary x500 in
a1000px area, but outer split handle x375. First leaf1 ends at250; the code
averages that with root3's500 rather than using the direct container boundary.
Vertical-subtree control is correct. Use container geometry; nested drag intent
mapping remains a separate, unexecuted consumer question.

The four GraphTree/parity/legacy-layout findings affect parked but compiled
public APIs. Repository search found no production GraphTree/TreeTopology call
outside Forme, only comments. These are public defects, not demonstrated
current product failures. Forme's canonical Arrangement/FormeDocument do have
product consumers. Parking is a warning, not removal; deprecation needs an
API/consumer review rather than sweeping deletion. Arrangement occurrences and
graph members have distinct IDs; unique-member identity preservation is real.
Curation really owns SubgraphSpec/Kind through reexports. Old GraphTree delta
and newer Mere SessionSubgraphs delta implementations remain separate, with
different storage and automatic/proposal policies. One spec does not establish
one reconciliation implementation. Pressure remains advisory; no resource
release is proved by recommending Cold.

Uxtree's AccessKit projection is separate from parked Forme UxNodeDescriptors.
It carries original block paths through presentation wrappers and folds, while
graft retains independent TreeId namespaces and orders focus updates. Source
contains substantive modeled tests for three consumer versions, including raw-ID
and missing-graft controls. Those were read, not rerun here; they explicitly
leave actual sessions and native adapters to other receipts. Composition's
blanket panic-prevention prose exceeds its graft-pair checks: general guest
topology/root/focus checks are absent. That remains a source-only candidate.
Stitch path collisions, InPage/Submit action mapping and missing Click action
advertisement also require host tracing before any functional claim or broad
migration. Stable same-run hash IDs do not prove cross-toolchain hash stability.
No speculative production repair was made.

Direct rustc builds of current unchanged Forme source with store/petgraph pass
six native observations and120 original unit tests; four original store tests
are deliberately filtered. Current Curation source passes its nine original
unit tests. Cargo package test/build attempts failed before compilation on the
shared Windows target lock, so no Cargo gate is claimed. Dependencies are exact
hashed artifacts from the prior successful graph Cargo build; a cached kernel
fixtures library is imported by original test sources. Integration/property and
Uxtree tests, OS/headed/product/GPU paths were not executed. All owned direct
libraries/executables/PDBs and fixtures are removed; no target/home/worktree
isolation created. Preservation and exact commands appear in the
[Forme subtree review receipt](../../../support/receipts/authority-success-audit/forme_review_receipt.json).


### Read-only Platen and shared Workbench source review, 2026-10-16

All10 Platen and4 shared Workbench files are read,175,764 more characters.
Coverage is989/3352 files and13,765,477/36,854,820 characters. The full-source
goal remains active. Resume at crates/inker/document-canvas/Cargo.toml.
Production/concurrent source remains unchanged. Authority entries A225-A239
track the shared vocabulary, separate reducers and actual admission owners.

Q132: sanitization detects[-1,2] as invalid but leaves it negative after division
by total1. Both geometry sanitize and live set_weights turn finite positive
[f32::MAX,f32::MAX] into[0,0] because the f32 sum overflows. A live layout then
fails canonical_roundtrips and debug to_persisted_json panics before its Result.
Ordinary[3,1] becomes[0.75,0.25] and serializes. No release panic claimed.
Share normalization is independently repeated; use a stable finite policy
without nudging already-valid serialized fractions.

Q133: open_in_slot_of(held2,missing99) returns false but removes2 from its
existing stack first. The guarded move API preserves it on the same refusal;
valid-target control succeeds. Check/stage before detaching. No external
production caller of this specific open helper was found in repository search.

Q134: members inserted1,2,3 with StackedWith1-3 and2-3 produce two slots,
leaving explicit2-3 ungrouped. A chain aligned with insertion order produces
one slot. First-known-slot placement never merges earlier groups. Resolve
stack components independently of order while retaining occurrence identity.

Q135: geometry is a competing semantic authority. The same arrangement
1 StackedWith2 yields one cell without geometry, silently omits2 with a stale
one-member sidecar, and yields two cells with split geometry. The latter still
passes canonical_roundtrips: it re-derives both canonical inputs from its own
output, so it proves self-reconstruction, not agreement with the supplied
arrangement. Existing reconciliation removes extraneous members only. Explicit
authority/reconciliation and occurrence support may exceed the one-hour
diversion limit and are documented instead of redesigning during the audit.

Q136: closing inactive1 from[1,2,3] with active2 selects3 in BOTH Platen and
shared Workbench. Independent removal implementations clamp the old index
instead of preserving selected identity. The last-active control remains
correct through clamping. Graphshell Closed and missing-member masking use
Platen close, source-traced only; no hosted event execution was performed.

Q137: arrangement member1 plus geometry[1,1] admits two tabs, passes the
canonical check, emits duplicate AccessKit IDs, and leaves one copy after close.
Ordinary open_stack deduplicates the same input. Loaded-profile admission must
match mutation policy; real repeated appearances need occurrence IDs throughout.
No AccessKit consumer or adapter was executed on the malformed DTO.

Q138: shared Workbench drag1 onto its OWN Right edge returns Applied but
deletes1 from[1,2]. The target existed before removal, then split_at_tile cannot
find it; that failure is ignored. Unknown-target control leaves both and returns
Unchanged; other-tile edge control preserves both and succeeds. Preserve custody
and report insertion failure, or implement a properly anchored split-out.

Q139: FloatingTile::resolve says nonfinite inputs read as zero, but NaN host x
panics in clamp and NaN area width returns NaN x. Bad relative coordinates are
sanitized and finite-area control is correct. Apply or state the area admission
rule. This does not prove ordinary host viewports are invalid.

Current Graphshell has useful guards: load checks exact member set, duplicate
accesses and finite positive shares/total; edit validates a candidate before
publishing. This blocks Q132 invalid shares and Q137 duplicates in those paths,
and the missing-member Q135 case. It does not compare stacking-edge semantics.
Own-cell edge drop maps to split_out, protecting that host from Q138. These
are source-traced guard excerpts, not full-file reading credit or UI receipts.
Do not remove working guards when consolidating validation. The public-library
defects remain real without overstating their current product reach.

There are independent active reducers in Platen Pane and neutral Workbench
TileTree, plus the previously read parked GraphTree. Neutral typed vocabulary
is shared, mutation algorithms are not. The same selection bug survives in two
copies. Workbench's outside-drop Effect correctly keeps custody pending a host
decision, and Workspace station moves guard successful insertion. Public raw
restoration/serde/tiled_mut remain caller-owned admission. Transient rails and
drawers genuinely keep presentation separate from canonical organization.

Platen owns project_tile_layout returning Uxtree's AccessKit carrier; A221 is
qualified accordingly. Fixed root/slot-index namespaces, unbound tool tiles,
member-keyed geometry and distinct occurrence IDs need explicit profile claims,
not a universal lossless-Arrangement claim. Document scene construction delegates
to the actual document-canvas owner with caller viewport/style; no copied text
layout implementation appeared here. That owner is next for source review.

Fresh Platen no-default-feature Cargo library build and50 tests pass. Shared
Workbench serde Cargo gate passes22 unit tests and0 doctests. Nine native public
probes pass with controls, including two expected caught panics. Platen's default
document-layout feature and four wrapper tests were read but not run. No rendered
product, AccessKit consumer, GPU, native adapter or headed UI executed. Probe
executable/PDB removed; no isolated target, Cargo home or worktree created.
Evidence and source preservation are recorded in the
[Platen/Workbench review receipt](../../../support/receipts/authority-success-audit/platen_review_receipt.json).


### Read-only document-canvas complete source review, 2026-10-16

All13 inventoried document-canvas source/manifest files are read,250,223 more
characters. Repository coverage is1002/3352 files and14,015,700/36,854,820
characters. Full-source goal remains active. Resume at crates/inker/inker/
Cargo.toml. Production/concurrent source remains unchanged. Authority entries
A240-A253 distinguish actual shared ownership, copied projection rules and
execution boundaries.

Q140: two ordinary same-style links receive distinct semantic IDs but equal
whole-shaping-run hit rectangles. Clicking inside FIRST resolves SECOND's URL.
An intervening space does not prevent it; bold FIRST creates a shaping boundary
and the control resolves FIRST correctly. The code tests each annotation against
underlying run text_range and uses the whole run advance. Correct link identity
does not repair wrong click geometry. Per-cluster/item bounds and shared typed
hit ownership need a careful repair, potentially beyond the one-hour diversion.
Native current Windows fonts/public packet APIs are exercised, not a browser
pointer or network request.

Q141: nested source indices contradict their public type meaning. block_at on
quote0's second child returns source index1, which names unrelated top-level
outside content in EngineDocument.blocks. Quote1's first child carries1000,
outside a two-element source array. top_level_block deliberately avoids this
known collision, and its control remains correct. Smolweb uses a hierarchical
path but independently repeats the decimal packing. Preserve that adaptation
while replacing ambiguous integers with explicit source/path identity; this is
a broader contract/consumer migration. No current hosted citation failure tested.

Q142: changing only List ordered=true to false produces an exactly equal visual
packet. Portable paint-command Debug dumps are also equal, not a serialized-wire
proof. A changed-text control differs. List dispatch discards ordered; the packet
has no marker/ordering field, and paint only recurses the resulting Group. Uxtree
retains an ordered description separately. Render markers or retain the visual
distinction without putting decoration into authored link labels/source IDs.

Q143: roles.body.spacing_above80 resolves to80 but yields the same paragraph
packet as0. Body spacing_below and code spacing_above controls change geometry.
The paragraph wrapper hardcodes0 instead of passing the resolved above spacing.
This is a contained configuration-field repair; defaults remain unaffected.

Q144: hiding an earlier empty-label link's adornment renumbers a later retained
link ID in the same document. SchemeArrow makes the empty annotation nonempty;
None or an empty custom glyph removes it before document identity allocation.
A nonempty first-label control retains the later ID across styles. Allocate or
reserve identities from authored occurrences, separately from visible rectangles.
Smolweb's style setter preserves document/focus state and invalidates geometry,
source-traced only; no hosted stale action or security bypass is claimed. The
session's revision/currentness rules need full review before a consumer change.

Several ownership claims are actually implemented well. FontInterner records
the exact Parley Blob allocation plus collection index, glyph runs refer to that
sidecar, and paint ships those exact bytes. Original per-run/index tests pass.
Requested family/weight/style are not used to re-resolve render bytes. Packet
serialization explicitly omits font bytes and calls PaintList the self-contained
transport form, avoiding a hidden serde-skip success path. Netrender is a thin
shared-translator shim, not another local text renderer. The default gate does
not execute that feature or prove IPC/GPU rendering. Native font availability
does not establish browser guest font availability.

Logical link identities distinguish repeated URLs and survive wrapping/fold
changes in the existing tests. Hidden blocks reserve IDs. Q144 shows that this
policy still depends on style-derived annotation existence. Visual and semantic
projections remain distinct implementations: Uxtree projects EngineDocument,
document-canvas now retains LinkSemantics, and Smolweb builds session semantic
snapshots using source plus measured packet. The old visual-only/semantic-only
intro is too absolute; source, layout, namespace and action authority must be
named separately. Those excerpts do not count as full reading of document-lanes.

Windowing correctly filters/translates blocks and interactions in original tests,
but an intersecting Text block still clones/emits every off-band run. Band texture
extent is not a bound on glyph work for a tall single paragraph. No huge allocation
or performance benchmark ran. Image byte-count admission, premultiplied palette
defaults and per-list key/generation scope require downstream owner review;
normal image/font sidecars work, and no backend failure was inferred. Table
measurement intentionally uses a throwaway font interner and allocates no document
IDs, then final layout shares ordinary glyph/interaction machinery. Its original
ragged/overflow/alignment controls pass without establishing full CSS tables.

Fresh default Cargo library build and86 original library tests pass. Five native
public observations with controls pass. Three optional netrender shim tests were
read but not executed. No GPU, renderer, hosted session, browser/wasm, native
adapter or physical assistive technology ran. Owned probe executable/PDB removed;
no isolated target, Cargo home or worktree created. Exact hashes, consumer trace
scope and test evidence appear in the
[document-canvas review receipt](../../../support/receipts/authority-success-audit/document_canvas_review_receipt.json).


### Read-only Inker complete source review, 2026-10-16

All23 inventoried Inker files are read,335,120 more characters. Repository coverage is1025/3352 files and14,350,820/36,854,820 characters. Full-source goal remains active. Production/concurrent source remains unchanged. Resume at crates/system/document-lanes/Cargo.toml. The session contract itself is external Genet document-session-api, re-exported rather than copied into Mere; its dependency source earns no local coverage credit. Authority entries A254-A269 accompany the individual bugs.

Q145: Navigation currentness accepts unrelated same-length replacement blocks. A real two-block heading/body table with an anchor and fold reports is_current=true for unrelated Rule/body blocks. resolve still returns the old anchor. A one-block replacement is rejected. is_current compares only block_count, despite claiming indices still describe the supplied blocks and copied targets stay inert.

Native public navigation table and replacement/control. No hosted copied-target action, fold UI or stale session bypass executed. Repair scope: source identity/revision and transform contract migration may exceed one-hour diversion. Name the guard as a length check or bind navigation/targets to an explicit document identity/revision. A complete fix must cover same-count replacements, copies into another document and consumers, while preserving explicit clear/remap transforms.

Q146: Markdown/Knot export conflates literal syntax with authored structure and raw frontmatter values with fields. Literal Text("*x*") and Emphasis(Text("x")) are distinct source blocks but export exactly the same Markdown and Knot strings. HTML distinguishes them. Title "Alpha\nsource: Beta" without canonical URI exports exactly the same Knot as title "Alpha" with canonical URI "Beta". Two distinct inputs with identical bytes cannot both round-trip as the claimed equivalent document.

Native serializer non-injectivity proof with HTML control. No Nematic parser reparse, trust escalation, executable HTML/JS, or saved user note executed. Repair scope: literal escaping is contained; complete structural/frontmatter round-trip contract can exceed one-hour diversion. Escape syntax according to each output context, choose safe code fences, and encode multiline frontmatter values unambiguously. Qualify the round-trip claim and test supported field equivalence through the actual Knot parser. Do not infer authentication from a serialized trust label.

Q147: Knot semantic round-trip silently drops stable feed-entry identity. Changing only FeedEntry.guid from first to second produces exactly equal Knot output; a changed block-kind control differs. The serializer also omits published, updated, enclosures and content_address by source inspection. Public to_knot documentation promises equivalent document-level metadata and every structural/semantic block.

Native output collision for guid; other omitted fields source-qualified. No feed database identity collision or parser reparse executed. Repair scope: semantic fence schema and parser/consumer evolution may exceed one-hour diversion. Define which semantic fields Knot preserves and extend producer/parser together, or explicitly describe a lossy projection. Keep stable feed identity and offline content address distinct from display date/article navigation. Presented semantic blocks also take ordinary Markdown rather than semantic-fence serialization and need a control in that repair.

Q148: Independent authority splitters both misparse IPv6 hosts. gopher://[::1]:7070/1/notes exports host "[" and port70; a DNS host with7070 is correct. host_from_address independently returns "[" for https://[::1]:7070/, and a configured [::1] host override is missed while a DNS override works. Both split authority on the first colon.

Native string export and default policy with host overrides. No Gopher server/client, DNS, socket or live navigation executed. Repair scope: contained bracket-aware authority handling or shared URL parsing. Use an existing typed URL/authority owner or a shared validated helper rather than fixing two ad hoc splitters independently. Specify bracket conventions at override/export boundaries and retain userinfo/port controls.

Q149: Availability-filtered routing still returns an unavailable fallback engine. A filter returning false for every ID still produces external.protocol; an empty EngineRegistry does not contain that decision. An available Genet control routes correctly. The fallback bypasses is_available, contrary to the API explanation that filtering avoids decisions pointing to unregistered engines. Existing test also expects an unavailable fallback without asserting availability.

Native policy and registry membership. No actual external protocol launch, application dispatch or host registration graph executed. Repair scope: explicit no-route result and host adoption may exceed one-hour diversion. Validate fallback availability and expose explicit absence/refusal, or document a separate host-handled sentinel contract. Do not silently select another unavailable engine. EngineRegistry already refuses missing IDs, so this is a route-contract gap rather than proof of successful dispatch.

Q150: The claimed type-enforced one-hop flip invariant is an unenforced implementor convention. An ordinary downstream Secondary type implements FlipBack, FlipDonor and FlipReceiver together, compiles, then forwards its state to B and B to C through flip_forward. Public traits contain no mutual exclusion, role token or constrained primary identity, although docs claim there is no type path for secondary chaining.

Native generated implementor and two forward calls. Built-in ScryForward itself does not implement FlipDonor. No real black-box state extraction or lossy browser chain executed. Repair scope: documentation correction is contained; enforced role/provenance API may exceed one-hour diversion. State the current convention accurately. If enforcement is required, make primary/source custody an explicit API input or constrained role and review downstream implementors. Merely naming separate open traits does not enforce negative trait membership.

Q151: Scry loses host-observable cookie and restore failures while finishing the flip. A mock set_cookie refusal still yields begin Ok and navigates. A run_script refusal then yields terminal is_done. Debug warnings appear in stderr, but neither error reaches a result/event/diagnostic accessor. In non-debug source configuration tracing_warn discards its argument. Docs say cookie failures are logged by/decided by the host, yet this seam never reports them to it.

Native mock surface with default/debug Cargo library. Release suppression is source-qualified; no release build, real WebView, authentication flow or user cookie store executed. is_done explicitly includes failure and is not itself misdocumented as success. Repair scope: typed degraded outcome and asynchronous host adoption may exceed one-hour diversion. Preserve degrade-without-blocking while returning/retaining structured per-layer failures. Keep admission, load completion and restoration outcome separate. The host can then decide how to surface missing state without scraping debug stderr.

Q152: An XML declaration prevents feed sniffing and defeats the default feed route. The same Atom root without an XML declaration sniffs application/atom+xml and routes to the feed engine; adding the ordinary XML declaration sniffs application/xml and routes to Genet for HTTPS. The default policy has no application/xml feed rule. The test permits either result and claims generic XML can still dispatch without checking the actual combined policy.

Native sniff plus default policy composition on two byte fixtures. No fetch host second-pass logic, parser, RSS endpoint or actual rendering executed. Repair scope: contained sniff precedence with generic XML controls. Recognize a feed root before returning the generic XML label within the same bounded window. Test both generic XML and feed declarations through sniff plus routing; do not route all application/xml to feeds.

Q153: SVG sniffing escapes the advertised small prefix window. Two inputs with exactly equal first1024 bytes and65536 leading spaces produce different MIME values when the later marker is SVG versus HTML. SVG is detected beyond the advertised window because sniff_image trims the entire input; HTML remains limited to head_window. Existing bound test exercises HTML only.

Native65536-byte whitespace fixtures and public sniff result; no denial-of-service, huge allocation or benchmark claim. Repair scope: contained prefix-bound consistency. Apply the documented window before whitespace-dependent image sniffing, keeping fixed magic-byte checks and BOM policy explicit. Test every signature family at/after the bound, not only the structured-text branch.

The portable EngineDocument facts, sparse provenance sidecar, source presentation wrappers and host-supplied evaluation/transclusion seams are implemented shared boundaries. Default policies deny execution/fetch expansion. Successful transforms clear navigation; top-level-only evaluation does not rescan generated output. Broader host budgets and enabled max_depth0 semantics need owner review. Predicate extraction remains pure and leaves graph relation resolution/assertion to linked-data. Distinct format traversal code is sometimes appropriate lowering, but new variants can drift across those repeated walks.

Document and surface registries intentionally separate request/response from long-lived frame lifetimes, but do not enforce cross-registry ID uniqueness. Registration replaces duplicate IDs locally. Actual host dispatch ownership still needs complete reading. Pin and host override contracts default to CompositedTexture; concrete headless/native-overlay consumer consistency is not established here. Cookie/SameSite DTOs are duplicated between flip and surface seams, explicitly described as deliberate even though their payload fields currently match. Adapter maintenance and complete cookie metadata require consumer review; absence of fields alone was not promoted to a security finding.

The web-surface document capability default delegates to the shared external DocumentCapabilities, while an override can produce a separate answer. Type-level accessibility and instance status are different declarations; no contradictory backend was executed. Surface commands use one ordered event stream with request IDs, explicit unsupported defaults and clear acceptance/completion semantics. Mock tests prove the contract fixture, not all backend completions. Native frame types name format, resource epoch and custody; original owned downcast/mismatch tests pass. No GPU importer/fence receipt follows from that. HttpCredentials Debug redacts secrets, and no serde derive exports them.

Fresh default Cargo library build and123 original library tests pass;9 generated native observations pass. The serializer proof is byte equality between distinct inputs, without parser reparse. Scry debug errors were observed; release suppression is source-qualified. Optional genet-donor source/tests were read but not built/executed. No actual browser/WebView, hosted session, network, GPU or native accessibility execution. Owned probe executable/PDB removed. No isolated target, Cargo home or worktree created. Exact hashes and execution scope appear in the [Inker review receipt](../../../support/receipts/authority-success-audit/inker_review_receipt.json).


### Read-only document-lanes complete source review, 2026-10-16

All12 inventoried document-lanes files are read,243,970 more characters. Repository coverage is1037/3352 files and14,594,790/36,854,820 characters. Across this pass, Inker and document-lanes added35 complete files and579,090 characters. The full-source goal remains active. Production and concurrent source are unchanged. Resume at crates/nematic/nematic/Cargo.toml. Authority entries A270-A281 accompany the findings.

Q154: A registered Smolweb session engine ID does not select the parser that actually runs. SmolwebSessionEngine registered as nematic.text accepts an HTTPS held-body request containing literal heading syntax. Its actual document source_kind is nematic.gemtext, with a Heading and derived title. spawn never uses self.engine_id in lowering; lower independently selects by scheme, then feed prefix, then Gemtext. An explicit text pin/registration can therefore describe a different parser from the one producing content.

Native SessionEngine spawn with a NoFetch fixture, downcast, inspect and portable source facts. No host route UI, web fetch or network session executed. Repair scope: registration/route/MIME/session lowerer consolidation may exceed one-hour diversion. Pass the selected format/engine or shared registry decision into lowering and retain the actual response MIME/final identity. Avoid another independent scheme/body selector. Keep deliberate block-versus-session engine kinds while sharing parser selection.

Q155: Nested document images never enter the host subresource request set. A top-level Image produces its URL in subresources. The identical Image inside a Quote or List produces no request. provide_subresource only accepts pending URLs from that top-level scan, and replace_document similarly retains cached images only if a top-level Image still names them. The portable model, reader lowering and document-canvas support nested blocks.

Native public source/resource discovery with top-level, quote and list fixtures. No decoder/network failure or actual reader page image download executed. Repair scope: contained recursive image visitor shared by discovery/admission/cache retention. Traverse Presented, Quote and List source blocks consistently and deduplicate URLs. Reuse that visit for pending requests, response admission and cache retention so different shallow scans do not become separate resource authorities. Preserve host fetch/decode limits.

Q156: Smolweb load drops redirect-final response identity before resolving relative content. A rich mock fetch response names final.test/actual/index.gmi but load_with_inline_media asks for alias.test/old/index.gmi through the byte-only fetch method. Its retained address and promoted relative image remain on alias.test/old/image.png. Parsing the same body at the final URL gives final.test/actual/image.png. RemoteFetcher already exposes ResourceResponse with final_url/content_type, then its legacy fetch strips them.

Native mock rich ResourceFetcher, public load and inline-media promotion; direct final-URL parse control. No real redirect or live network executed. Exact external ResourceFetcher contract was traced in the current pinned Cargo checkout only. Repair scope: contained response-record plumbing; engine-selection integration broadens the repair. Consume fetch_response and preserve requested address versus final source/base identity explicitly. Pass response MIME into the same selected-parser owner instead of dropping it. Handle requested fragments deliberately while retaining final provenance.

Q157: Ragged table accessibility attaches later text to preceding padded cell geometry. For source rows [A0] then [B0,B1], document-canvas emits two cells for each row. The semantic builder advances one flat iterator only for authored cells, so B0 receives row0 column1 padding bounds and B1 receives row1 column0 bounds. Native projection shows A0 and B0 share y while B0 has the right-column x. Source text is therefore attached to the wrong presented place.

Native retained frame/semantic projection, CPU layout only. No OS screen reader, platform adapter or screenshot executed. Layout padding is intentional and separately tested; the mismatch is the consumer iteration. Repair scope: contained shared grid traversal/cell identity. Index rendered cells by the actual padded column count or carry explicit row/column/source cell identity. Preserve ragged/empty/header/alignment-only columns and interaction ownership. Avoid inferring layout row boundaries from authored row lengths.

Q158: A pointer-clickable image loses activation in the accessibility projection. A retained Image has one ordinary link rectangle and its pointer center returns Navigate(image URL). The semantic projection emits only an Image with ScrollIntoView, no node advertising Click, and accessibility_click_target(image ID) returns None. The Image branch ignores the corresponding layout focus stop instead of exposing its activation.

Native public retained session, pointer click, semantic nodes and click-target query. No decoder required; placeholder image retains the same navigation. No physical accessibility device or host input bridge executed. Repair scope: contained semantic image/link composition using retained interaction identity. Represent the image and its navigation through the shared stop identity, keeping the alt label and image role without duplicating announcements. Cover promoted gemtext images and nested images while preserving ordinary pointer/keyboard semantics.

Q159: The synchronous remote HTTP bridge panics when called inside a Tokio task. Calling RemoteFetcher.fetch for HTTP inside a current-thread Tokio runtime produces Cannot start a runtime from within a runtime. catch_unwind confirms the panic occurs at the nested multithread runtime block_on before polling the HTTP future. Unsupported-scheme control returns None. The synchronous trait has no documented runtime-context refusal and this path panics instead of returning a failed fetch.

Native controlled nested runtime and caught panic; loopback address future is never polled, so no HTTP request/response is claimed. No current product host crash executed. Repair scope: blocking/async fetch boundary ownership can exceed one-hour diversion. Keep one async fetch owner and place blocking adaptation on an appropriate worker boundary, or expose explicit context refusal rather than panic. Do not add more private runtimes or call block_on recursively. Preserve shared cache, concurrency, timeout and cleanup semantics.

Q144 consumer follow-up: Native retained SmolwebDocument focuses Second with an earlier empty-label link. After link adornment None and a new frame, focused_interaction resolves Third. Nonempty first-label control retains Second. Current accessibility revision guards still reject old queued requests; the reproduced failure is live retained keyboard focus retargeting across presentation, not a stale-request bypass.

Reader shares an immutable Arc source and lineage while retaining local layout, scroll, viewport and virtual focus per appearance. Its no-held-bytes, unreadable and scripted-empty cases explicitly refuse. Reader semantics disclose visible-links-only and unavailable activation; revision checks reject stale queued requests. Smolweb retains scene geometry, typed interactions, fold keys and current pointer targets, with presented-boundary revocation on relayout. The local frame boundary means Scene construction, not a GPU/OS presentation receipt. Native tests validate these distinctions. Currentness protection does not repair Q144 source identity derived from presentation.

The semantic projection repeats decimal-packed nested indices and table traversal conventions from the layout producer. Q157 is concrete drift at that hidden ABI. Image promotion and decode are explicitly host-configurable and bounded, but discovery/admission/cache retention share shallow top-level scans. Complete recursive source identity and shared parser/response selection are broader owner repairs; document them rather than extending this audit by over an hour.

RemoteFetcher shares HTTP cache, redirect cap, concurrency permits and one runtime across clones; current local mock cache/revalidation/limit tests pass. Permit queue time is outside request timeout. Smolweb does not retain ResourceFetchPolicy and installs process-global InMemoryTofu on first use. A host-selected durable/custom trust store may be replaced; current caller/pin behavior needs Errand/Gemini owner review, not a speculative security claim. Parser errors silently fall back to Gemtext and in-page event queues survive source replacement; those remain source-qualified candidates pending actual caller/probe evidence.

Retained clips use Fleece selected-body export/import and caller-owned custody limits. Playback cache identity is content-derived, existing bytes are checked, and new files use synced tempfile/noclobber publication. Current selected-body, sealed-refusal and corruption tests pass, without decoder/audio/cryptographic proof. Host-supplied clip presentation checks address and source_kind but is not independent source authentication. Typography reuses Tabard validation and clones the consumer baseline for reset/role overrides.

Fresh feature-enabled Cargo build and54 original library tests pass with smolweb, netfetch and retained-clips. Seven generated CPU observations pass. The original suite includes local HTTP mock servers and a negative .invalid Smolweb request. A separate GPU streaming integration test was read but not run. No GPU rendering, browser/WebView, native accessibility adapter/physical AT, playback, successful live capsule or real redirect ran. Probe HTTP nesting panic was expected and caught before polling the HTTP future. Direct Rust linking initially lacked Cargo native-library search paths; the audit runner now imports them. This harness setup failure was corrected without product edits. Owned executable/PDB/failed-link objects are absent. Stable shared Cargo target retained; no isolated target/home/worktree. Exact scope and hashes are in the [document-lanes review receipt](../../../support/receipts/authority-success-audit/document_lanes_review_receipt.json).


### Read-only Nematic complete source review, 2026-10-16

All75 inventoried Nematic files are read,452,682 more characters, including the manifest, example and checked-in Micron source fixtures. Repository coverage is1112/3352 files and15,047,472/36,854,820 characters. Full-source goal remains active. Production and original tests/concurrent work are unchanged. Resume at crates/system/errand/Cargo.toml. Authority entries A282-A299 distinguish actual shared parser reuse from competing selectors and overclaimed parity/round-trip.

Q160: HTML active-URL filtering accepts control spellings that normalize to javascript. Ordinary javascript: is refused, but HTML entity-decoded LF, TAB or CR inside java...script makes the homemade scheme recognizer treat it as no scheme and accept the link without a diagnostic. The current url::Url parser canonicalizes each admitted target to scheme javascript. An ordinary HTTPS control remains accepted. This contradicts the module claim that active URL schemes never reach the portable model.

Native HTML parser/lowerer plus actual URL canonicalization, three controls and HTTPS/refusal controls. No script, browser, export click or host navigation executed; no XSS execution or current host policy bypass is claimed. Repair scope: contained canonicalization/admission correction; host-wide URL policy review can exceed one-hour diversion. Classify the canonical URL form using the existing parser, with an explicit base/relative-reference policy, before applying the allowlist. Keep HTML entity decoding, control normalization and export/host navigation policy distinct. Do not use a second ad hoc colon-prefix check as URL authority.

Q161: HTML whitespace-only text nodes disappear between inline elements. <em>red</em> <em>fox</em> lowers to the plain text redfox. Plain red fox and red followed by a styled fox retain their source space. collapse_whitespace only appends a pending trailing space when output is already nonempty, so a whitespace-only DOM text node vanishes between the adjacent inline containers.

Native HTML lowering and public to_text with two text-bearing whitespace controls. No layout screenshot, browser rendering or full CSS whitespace claim. Repair scope: contained inline whitespace normalization/state correction. Retain an inter-node separator in normal-flow inline content and trim only at paragraph/block edges. Cover adjacent links/emphasis/strong/code and whitespace-only DOM nodes, while preserving preformatted handling.

Q162: The native-default Djot parser silently flattens lists and quotes and drops tables. Ordinary Djot quote/list fixtures retain words as top-level paragraphs without their containers. A table emitted in the writer-supported pipe shape produces blocks=[]; all three have empty diagnostics. The CommonMark compatibility quote control preserves Quote, and the default text/x-knot route selects this Djot engine. Parser container events other than its small selected subset fall through to the wildcard.

Native default parser, actual default route selection and compatibility control. No persisted user file, hosted note editor or GPU rendering executed. The source still labels parts as PoC, while the manifest/design/default route claim native parity/default migration. Repair scope: stack-based Djot event lowering and projection qualification can exceed one-hour diversion. Implement the intended structural vocabulary or surface explicit unsupported/raw content without silently losing it. Reconcile the completed parity/default claim against real constructs and diagnostics. Do not infer native grammar completion from the fence/semantic-block sample tests.

Q163: The Djot round-trip serializer erases recognized link destinations and predicates. A recognized Topic link to mere://node/topic with rel=cites parses into an outgoing link and one LinkStatement. blocks_to_djot writes only Topic newline using inline_text; reparsing keeps the label but yields no destination and no statement. The public serializer calls itself the parser dual and claims round-trip on the recognized subset, which now includes typed links and rel.

Native parse/export/reparse and actual link_statements controls. No graph ingest, edge deletion or user save executed. Repair scope: inline serializer/schema adoption can exceed one-hour diversion when preserving the full vocabulary. Serialize inline structure, destination, title and predicate with grammar-correct escaping, or explicitly qualify the projection as lossy. Keep one Djot emitter for recognized spans and exercise the actual supported parse-emission subset rather than only plain semantic sample blocks.

Q164: Per-block clip provenance is emitted but disappears instead of becoming the promised metadata. The clip builder emits block_sources containing the second source URI and anchor. Both CommonMark and Djot Knot parsers drop that information entirely: no block_sources MetadataRow, no source URI in the resulting document, and no block_sources on re-export. Document-level source remains as a control. apply_frontmatter applies only its named keys. The builder explicitly promises a metadata row even though full provenance reconstruction is deferred.

Native builder, sparse sidecar, both parsers and re-export with a differing-source override. No citation overlay, composed user clip or consumer provenance restoration executed. Repair scope: contained metadata-retention fix; full indexed provenance restoration remains a larger gated design. Preserve the advertised metadata or state accurately that parsing discards it. Do not confuse the unimplemented minimum retention with the explicitly deferred full sidecar restoration. If implementing restoration later, account for note_kind/tags prefix blocks and source-index remapping, escaping and delimiter admission.

Q165: Extension and MIME routes select incompatible grammars for the same native Knot file. The same file:///note.knot and definition-list body route through FileEngine to CommonMark when MIME is unknown, yielding no MetadataRow. Adding text/x-knot selects Djot and produces Trust=tofu metadata. FileEngine keeps a KnotEngine compatibility delegate for .knot. The governing design says Djot is the author/default grammar and compatibility remains by pin; no extension exception is recorded there.

Native default policy, full engine registry dispatch and actual document blocks. No file read, sniffing host pipeline or editor UI executed. Repair scope: contained native file default correction plus shared selection review. Resolve native .knot through the declared grammar consistently while retaining explicit compatibility selection. Share format/grammar selection across file, MIME and session routes; Q154 already demonstrates another selector ignoring the selected engine. Do not remove the intentionally supported compatibility engine.

Q166: Nested tight Markdown lists put the nested child before the parent item text. For - outer followed by an indented - inner, the outer item contains [inner List, outer Paragraph]. Inserting a blank line creates a loose item and preserves [outer Paragraph, inner List]. Tight item text stays in its inline frame until End(Item), while nested blocks attach earlier; the final flush appends the parent after them.

Native CommonMark parser/lowerer with tight and loose nested-item controls. No visual frame or screen reader executed; the portable source order itself is reversed. Repair scope: contained pending-inline flush at child-container boundaries. Flush pending tight-item inline content at the correct source boundary before attaching nested blocks. Cover mixed text, nested lists, quotes and subsequent content so a local ordering repair does not drop or duplicate tight/loose item text.

Q167: Plain-text blank-line splitting fails on CRLF input. alpha LF LF beta yields two Paragraph blocks; the same physical text with CRLF yields one Paragraph with an empty line represented by soft breaks. paragraphs splits the literal byte sequence LF LF even though paragraph_spans uses lines() and otherwise understands CRLF. The documented blank-line-separated paragraph rule therefore changes with ordinary line endings.

Native plain-text engine and paired LF/CRLF fixtures. No file load, renderer or text reflow screenshot executed. Repair scope: contained line-oriented paragraph delimiter correction. Group paragraphs by physical empty lines under the intended line-ending policy and retain within-paragraph soft breaks. Keep verbatim protocol mode separate, where original blank lines and columns are meaningful.

Q168: The feature-disabled library suite asserts HTML body structure that the declared fallback cannot provide. Fresh cargo test -p nematic --lib --no-default-features runs235 tests:234 pass and this one fails at the assertion that the article body has its own heading. With HTML disabled, the implementation explicitly emits an unsupported-body diagnostic and falls back to Short teaser, exactly the state shown by the failure. The unconditional test still requires the default-feature HTML structure. Default237 library tests plus3 example tests pass.

Fresh original feature-disabled library suite, observed shell exit1 and one named failure; no claim the fallback implementation is broken or all features pass. No production/test source edited. Repair scope: contained feature-qualified test expectations. Assert the HTML structure only for the enabled feature and qualify the disabled summary/diagnostic path explicitly. Keep the optional feature gate reproducible instead of treating the default suite as evidence for both configurations.

Q146/Q147 parser follow-up: actual KnotEngine parse/export/reparse converts an escaped literal *x* to an Emphasis span. A FeedEngine-parsed stable-id becomes guid=None after export and KnotEngine reparse. These strengthen the prior non-injectivity proofs into actual parser examples. No user save, graph edge/database mutation or trust escalation was executed.

The native-default migration is materially narrower than its completed-parity claim. Current source registers Djot and Inker MIME routing selects it; stale source comments still call it experimental/not registered. The governing polyglot design phase5 says parity and native-default landed, with CommonMark available by pin. Reusing fence expansion and inline rewrites is real, but does not implement every structural container or keep new link predicates in the serializer. The native FileEngine extension default still selects CommonMark. Q154/Q156 in the session lane already demonstrate independent selection and dropped MIME/final identity; a shared resolved parser authority needs a broader owner repair, documented rather than used to prolong the audit.

Gemtext/Gopher/Nex/Scroll/Spartan grammar reuse is present through Errand/protocol ASTs. Nematic owns portable grouping/lowering. Body wrappers for Guppy/Titan/Misfin/Finger are explicit about transport/reassembly/upload/envelope scope and keep trust Unknown; they are not parallel network implementations. The source_label field is overloaded with inner engine IDs as well as human source labels. Scroll still turns input links into ordinary links with a diagnostic claiming no input affordance, although shared Submit exists; its actual request semantics need review before mapping to a POST adapter. Spartan stale prose about Scroll signatures contradicts Scroll current explicit tests. These are recorded authority/documentation mismatches, not live protocol success claims.

Micron has a separately named compatibility subset and a richer source-preserving syntax lane. Syntax anchor admission, line-space navigation, block-index remapping around diagnostic badges, source-key fold state and measured layout are separate spaces with real reuse. Original probe-page controls validate bounded captured behavior. Form preparation checks exact source and configured budgets, rejects ambiguous/unsupported controls, redacts values in Debug and explicitly leaves destination identity/confirmation/cancellation/envelope limits with the host. No form I/O occurred. Source-preserving syntax does not make every portable export byte-faithful.

Feed XML/JSON share the Feed/block projection but JSON admits a smaller local serde field model, omitting media attachment metadata and a version discriminator. Entry rendering does not propagate feed language, and a relative article base fails absolute Url::parse before recursive resolution. Nested Djot div state and pending attribute attachment, source-index remapping for restored block_sources, and literal/extension preservation remain larger source-qualified follow-ups. Unknown-attribute and full block-provenance reconstruction were already explicitly deferred; Q164 is the narrower false claim that even a MetadataRow survived.

Fresh default Cargo library build succeeds;237 library tests plus3 example tests pass. Nine native observations pass, including actual reparses. A separate no-default-features library gate runs235 tests with234 passing and1 failing at an unconditional HTML-body assertion, while the disabled implementation explicitly falls back as designed. This optional configuration is not reported green. No browser/script execution, GPU, live network/server, OS accessibility, physical assistive technology or user persistence ran. Historical Micron JSON/wire receipts were fixture inputs, not new headed receipts. Probe executable/PDB removed; exact hashed library artifacts avoid mutable Cargo root-alias ambiguity across feature configurations. Stable shared target retained; no isolated target, Cargo home or worktree. Evidence and source hashes are in the [Nematic review receipt](../../../support/receipts/authority-success-audit/nematic_review_receipt.json).


### Read-only Errand complete source review, 2026-10-16

All 22 inventoried Errand files are read, adding 133,441 characters, including optional serving. Repository coverage is 1,134/3,352 files and 15,180,913/36,854,820 characters. The full-source goal remains active. Production and original tests/concurrent work are unchanged. Resume at crates/system/fetch/Cargo.toml. Authority entries A300-A314 track actual protocol reuse, filesystem/publication policy, wire projection admission, URI helpers, feed identity/loss, telemetry and competing trust-store initialization.

Q169: Directory root containment is lexical while reads follow junctions outside the published tree. An owned Windows junction inside the published fixture points at an owned sibling directory. Directory::get returns that outside sentinel as a Document. The /../outside/sentinel.txt control refuses. resolve rejects lexical parent escape, but tokio metadata/read follow filesystem links; the comment claims a request can never name anything above root however spelled or encoded.

Actual Windows junction and file read, entirely under the marker-identified stable Cargo target. No user document or external capsule, no public listener. Junction removed non-recursively and exact fixture files removed after verification. Repair scope: filesystem capability and reparse-point policy can exceed one-hour diversion. Define an explicit publication policy for links/reparse points and enforce containment at open/read time. A canonicalized precheck alone does not resolve check/use races. Preserve a trusted root handle or equivalent platform-safe capability rather than treating component arithmetic as filesystem authority.

Q170: Directory dotfiles are hidden from listings but directly served despite the not-published claim. The owned .audit-hidden.txt fixture is absent from the root Listing but a direct request returns its bytes as a Document. listing explicitly says dotfiles are not published and suggests renaming to publish one; get applies no corresponding admission rule.

Owned synthetic bytes only, direct Source API and root-listing control. No claim a user secret was exposed or an external server currently publishes this source. Repair scope: contained publication-policy consistency correction. Make hidden-file publication configurable and apply the chosen policy consistently to listing, direct files, intermediate components and configured index lookup. If hiding only affects discovery, describe it accurately rather than claiming publication denial.

Q171: One structural listing entry can inject additional native protocol links or rows. A newline plus gemtext link syntax in Entry.display makes one Entry become two parsed links. Tabs/CRLF in another display make one Entry become three valid Gopher rows, including an injected external destination. Actual shared protocol parsers confirm both. A benign Gopher entry remains one row. Public fields and constructors declare no validated-delimiter precondition.

Pure formatter then actual parser observations, no served network bytes or navigation. This proves structural injection, not code execution or current hostile source access. Repair scope: shared field-admission/serialization contract may exceed one-hour diversion. Centralize wire delimiter admission for all user-controlled display, target, title, preamble and host fields. Choose a visible rejection or representable normalization policy per format, because these protocols do not offer a universal escaping convention. Do not call parser-accepts-output tests proof that projection preserves source entry identity.

Q172: Absolute URL detection treats valid mailto destinations as local Gopher selectors. url::Url accepts mailto:reader@example.test as an absolute URL, while Entry::is_absolute returns false because it searches for ://. Gopher projection and reparse yield a gopher URL on local.test rather than the mailto destination. An HTTPS control remains external.

Pure public Entry/project/parser composition. No email, process launch or navigation occurred. The finding concerns the documented absolute-URL versus host-path distinction. Repair scope: contained existing URL-parser classification correction. Use the existing URL authority for absolute-reference classification with an explicit policy for local/network paths. Avoid another table of scheme punctuation or removal of intentionally supported opaque schemes.

Q173: Timeout helpers return failure before the common fetch tracing event can run. A loopback Gopher connection waiting on an unaccepted local listener times out with Error::Timeout and zero errand events. The same capture receives one event for an immediate unsupported-scheme refusal. timeout wraps and drops the inner future before trace_fetch_result after await. The common-entry prose promises one structured event for every public fetch helper.

Current Cargo-built native async API, current-thread subscriber, one owned loopback listening socket with no acceptor and no remote network. Identity/streaming variants share the source structure but were not separately executed. Repair scope: contained timeout-result tracing correction across helper variants. Place final outcome telemetry where timeout and transport results converge, preserving one event per call and caller cancellation semantics. Do not simply add an outer event while retaining inner completion events, which would double successful requests.

Q174: Fetch tracing strips query and fragment but retains URL passwords. An unsupported HTTPS URL carrying synthetic userinfo reaches the Errand warning event with the synthetic password retained. Query and fragment are absent as controls. Only boolean observations are persisted, not the captured URL. trace_url only clears query and fragment.

In-process tracing capture and unsupported-scheme refusal, no network or actual credential. The existing documentation promises query/fragment removal rather than explicitly promising password redaction; this is unnecessary credential-bearing logging, not a misquoted guarantee. Repair scope: contained userinfo redaction in the existing trace URL helper. Remove username/password from logged URLs at the shared helper and keep the current query/fragment controls. Audit structured error text separately because a redacted URL field does not automatically sanitize third-party errors.

Q175: Feed loss diagnostics disappear at the shared AST to portable-document boundary. The current XML parser records two diagnostics for an unknown extension element and invalid enclosure length. FeedEngine renders the same input into nonempty blocks with no diagnostics. It only transfers the html_stripped counter into a rendering warning and drops Feed.diagnostics.

Actual Errand XML parse and Nematic FeedEngine projection using one exact Cargo feature closure. No live feed, persistence or host degraded badge executed. Repair scope: contained diagnostic propagation at one projection boundary. Carry parser diagnostics into the existing EngineDocument diagnostic model with source/engine attribution. Keep an unknown extension warning distinct from refusal and from HTML summary stripping; successful parsing is not evidence of lossless representation.

Q176: Feed namespace identity depends on prefix spelling while foreign local names can override core facts. The same iTunes namespace URI under prefix i loses artwork/duration and gains unsupported-extension diagnostics. Prefix itunes bound to an unrelated URI is accepted and supplies both fields. A foreign x:title before the core title wins the entry title. The walker uses Reader, raw qualified strings for selected extensions and local names for core facts without namespace resolution.

Native parsing of bounded synthetic XML with equal-URI, alternate-prefix, unrelated-URI and foreign-title controls. No feed database mutation or remote spoofing claim. Repair scope: namespace-aware and context-aware parser migration can exceed one-hour diversion. Resolve bound namespace identities and validate parent/context before admitting core or extension facts. Keep known extension support separate from unknown-extension diagnostics and preserve raw source where required. Do not patch this by adding more accepted prefix strings.

Q177: Arbitrary well-formed XML returns feed success and can become an empty successful document. A generic XML document with a title returns Ok Feed, then an empty EngineDocument with no diagnostic. A generic document containing entry/title fabricates a FeedEntry outside any RSS/Atom root. The walker checks XML structure but never admits a recognized feed root/context. document-lanes also selects FeedEngine for any XML declaration, so its fallback-on-error branch is not reached for this successful empty result.

Errand and Nematic paths executed. The document-lanes XML sniff/fallback composition is source traced from its previous whole-file review, not a new hosted-session or screen reproduction. Repair scope: feed grammar/root admission plus downstream selector review can exceed one-hour diversion. Require a recognized feed root and valid entry context, distinguish non-feed XML from an empty valid feed, then make the caller retain a visible fallback reason. Reuse the parser verdict rather than another caller-side XML name table.

Q178: Nex directory base resolution appends child names inside a query. For nex://example.test/path/page?q=/wrong/ and child, the emitted URL keeps path /path/page and changes query to q=/wrong/child. The query-free control emits path /path/child. base_url uses raw ends_with and rfind slash over the full address, then Nematic concatenates the entry name.

Actual parser helper through current NexEngine and url::Url inspection, no network or directory navigation. Query semantics for a remote Nex server are not asserted; the emitted child is demonstrably not a path resolution. Repair scope: contained URL-component-aware directory resolution correction. Resolve against parsed URL path components and handle query/fragment deliberately, using the existing URL implementation. Keep entry-name admission and directory/content classification separate from address resolution.

The transport wrappers mostly delegate to their real protocol owners. Gemtext/Gopher/Scroll AST re-exports are actual shared types, and Nematic consumes them. Errand adds normalized status/error/MIME and optional native serving projections. These are useful layers rather than independent network engines. However, a shared helper can still be wrong: the Nex full-string base scan and Entry absolute-URL punctuation check duplicate semantics already present in url::Url. A benign formatter reparse confirms syntax acceptance but not structural identity when raw delimiter-bearing strings can introduce additional links/rows.

Trust initialization has competing owners. fetch::install_smolweb_tofu forwards a host-chosen store; document-lanes::remote first-use initialization installs a new InMemoryTofu. The actual registry Gemini-protocol0.1.7 setter replaces an RwLock-held Arc and explicitly says the latest call wins. This strengthens the prior candidate into a source-confirmed replacement path; it is not a TLS handshake, observed pin loss or MITM proof. Choosing session/host ownership and compatible initialization can exceed a one-hour audit diversion. Keep it documented.

Titan also contradicts the read/write overview. Scheme routing includes Titan; actual registry titan::fetch calls upload_inner with zero bytes. Introductory Errand prose says write companions are not fetchable schemes. A zero-byte payload does not make a write-shaped request read-only. No Titan request was sent and no endpoint mutation was attempted. Resource/crawler capability admission needs a broader caller review rather than another transport shortcut.

Other inherited prose has drifted: protocol dependencies are currently exact registry crates, not workspace members under protocols/; parse-feed is not an available feature and quick-xml is unconditional; the manifest license comment says MIT/Apache while the actual license and source headers say MPL-2.0. These are documentation inconsistencies, not an independent legal relicensing conclusion. Directory configured index names are host inputs; as-is Document bytes and structural Listing projections are different contracts. No hostile configuration or cross-protocol index defect was reproduced.

Fresh Cargo serve-feature library build and combined Errand/Nematic composition build succeed. Original Errand library suite: 64 passed, 1 ignored live Nex smoke test. Ten generated native observations pass. They include an actual Windows junction and owned files under a marker-identified stable Cargo target directory, actual format/reparse and feed projections, and one owned loopback listener left unaccepted to observe timeout. All fixture files, the non-recursively removed junction, executable and PDB are absent after cleanup. No remote network, TLS handshake, browser/script, GPU, user persistence or public serving ran in the generated probes. The original suite includes a separate real loopback Gopher serve/fetch control. Stable shared target reused; no isolated target, Cargo home or worktree. Evidence, exact hashed artifacts and source hashes are in the [Errand review receipt](../../../support/receipts/authority-success-audit/errand_review_receipt.json).


### Read-only Fetch complete source review, 2026-10-16

All 11 inventoried Fetch files are read, adding 137,838 characters, including optional HTTPS/Iroh adapters and their tests. Repository coverage is 1,145/3,352 files and 15,318,751/36,854,820 characters. Full-source goal remains active. Production and original tests/concurrent work are unchanged. Resume at crates/system/page-load/Cargo.toml. Authority entries A315-A330 track scoped stores versus global cookie helpers, duplicate cookie metadata, representation/budget policy, crawler paths, real protocol reuse and feature evidence.

Q179: Loading an empty persona retains the preceding persona cookie session. After a synthetic cookie is persisted for persona A, load_cookies for empty persona B leaves the same cookie readable from session_jar. Updating that inherited cookie and persisting for B writes it under B's durable prefix. load_cookies merges records into a process-wide jar rather than replacing or selecting a persona-scoped jar. Its docs say a persona with no stored cookies leaves the jar empty and keys provide a hard partition.

Current public loader/persister and real MemoryBackend; only synthetic cookies and in-process storage. No current application persona switch or real login exposed. Current Mere caller search found declarations/tests rather than a production caller of these persistence helpers. Repair scope: persona/session owner migration can exceed one-hour diversion. Bind a jar and its load/persist state to the host-selected scope, or provide an explicit safe single-persona lifecycle that refuses cross-persona reuse. Keep the real Stores scope seam rather than adding another global selected-persona variable. Define failed/partial load behavior and exclude old-persona cookies before exposing the new scope.

Q180: Incremental cookie persistence uses one shadow across all personas and stores. Persisting the same synthetic cookie for persona A writes one blob. After marking dirty, persisting for persona B writes none: shadow.get matches A's cookie although B has no durable blob. Changing the cookie then writes into B. last_persisted is documented per persona but is a single HashMap keyed only by domain/path/name; store identity is absent too.

Executed two-persona calls on one in-memory store, independent dirty marks and actual Backend list/get observations. Cross-store aliasing follows the same source structure but was not separately executed. This is distinct from the jar retention in Q179. Repair scope: scoped persistence shadow/ownership change can exceed one-hour diversion. Associate the incremental shadow with the actual jar/store/persona owner and seed/reset it consistently on successful loads. Durable prefix partitioning alone cannot make an unrelated global equality cache authoritative for whether a particular destination contains a cookie.

Q181: Expiry alone never re-arms cookie persistence, so expired durable blobs remain. A Max-Age=1 synthetic cookie is persisted, expires and disappears from usable jar records. A subsequent persist_cookies call leaves its durable blob because the dirty gate returns before comparing expirations. Explicitly marking dirty then persisting deletes it. The persister says gone/expired cookies are deleted, but elapsed time does not mark a write.

One bounded1.5-second synthetic expiry, MemoryBackend only, usable-record control and explicit-dirty deletion control. No expired authentication was accepted: the jar correctly refuses the expired cookie. The defect is durable cleanup/currentness. Repair scope: contained expiry-aware dirty/currentness correction. Account for the next expiry or perform a bounded cleanup comparison at persist boundaries. Preserve the optimization for unchanged live cookies while distinguishing no new Set-Cookie from no elapsed state change.

Q182: Untrusted Content-Range controls vector capacity and can panic before body admission. An open-ended range with Content-Range bytes0-18446744073709551614/18446744073709551615 passes ordering/request checks, then Vec::with_capacity(expected as usize) panics with capacity overflow before reading the empty body. A one-byte range control succeeds. catch_unwind records the failure without attempting a huge allocation.

Real NetFetch and current Netfetcher pipeline with a supplied one-hop wire response. This is a native public API panic from header admission, not an observed remote process crash or peak-memory/OOM test. Repair scope: contained checked capacity/bounded range admission; open-ended budget policy may be broader. Do not allocate the entire peer-declared range up front. Validate requested ranges and checked platform-size conversions, grow within an explicit caller budget and return FetchError on capacity/representation failure. Unbounded open-ended reads need a named budget policy rather than treating a syntactically ordered header as allocation authority.

Q183: The HTTPS-only blob adapter accepts plaintext HTTP as a successful HTTPS source. BlobSource::Https containing an http:// loopback URL returns the synthetic plaintext bytes successfully through HttpsFetcher. The implementation dispatches by enum variant and sends the embedded string to ureq without scheme admission. The original test named fetches_https_response_body also uses mockito's plaintext HTTP URL.

Actual local HTTP exchange and public BlobFetcher API. No TLS server, downgrade redirect, remote origin or real content credentials. Hash verification remains delegated to resolve_blob; this finding concerns transport/scheme identity rather than digest validation. Repair scope: contained initial scheme validation; redirect transport policy needs explicit owner review. Validate that the declared HTTPS source uses HTTPS, and apply a clear policy to subsequent redirect hops in the chosen agent. A source variant name and a passing test whose fixture is plaintext do not establish TLS. Keep custom-agent/proxy configuration under the caller owner.

Q184: Fetchable scheme selection rejects canonicalizable uppercase addresses that the fetch API serves. is_fetchable returns false for a valid HTTP:// loopback address while fetch_page_capped parses it through url::Url and successfully returns its bytes. The lowercase control is admitted. Prefix selection splits the raw :// string and compares case-sensitive names, while the transport parses canonical URL components.

Actual selector and local HTTP acquisition for the same address. No external navigation or host registration decision was executed. The analogous uppercase Smolweb routing disagreement is source-qualified, not separately exercised here. Repair scope: contained shared URL-component classification correction. Classify the parsed/canonical scheme using one URL implementation and pass the resolved identity across routing boundaries. Keep an is-supported predicate separate from syntactic URL validation instead of adding more raw-prefix tables.

Q185: The legacy crawler entry still uses browser cookies and automatic redirect traversal. fetch_page_crawler sends the synthetic browsing-session cookie on both hops of a local302 and returns the followed target body under the bot UA. The maintained fetch_crawler_capped control sends no cookie and returns302 without following. Anonymous-page control also sends no cookie. Current Fleece crawl code uses the maintained crawler API; the old public helper remains with prose claiming the crawl actor uses it for every page/robots request.

Actual loopback requests and headers, synthetic cookie only. This proves divergent public crawler implementations and stale ownership prose, not that current Fleece acquisition leaked a login or crossed a robots-policy gate. Repair scope: crawler entry consolidation/compatibility decision can exceed one-hour diversion. Name/deprecate the legacy contract or adapt callers to the admitted single-hop API, preserving status, response budgeting and caller redirect policy. Merely changing the User-Agent on a session fetch does not supply the maintained crawler's isolation semantics.

Q186: Smolweb page caps wait for full acquisition and rely on a nonexistent Gopher read bound. A Gopher fixture flushes eight bytes under max_bytes4 and keeps the socket open. fetch_page_capped remains pending until EOF, then returns the cap error. The dependency reads into Vec with read_to_end and no byte bound. The function discloses post-hoc Smolweb checking but justifies it by saying errand bounds its own read; that assurance is false for this actual route. The global PAGE_BODY_CAP prose also claims a hard streaming/OOM ceiling.

Eight synthetic bytes, one owned loopback connection, observed pre-EOF pending and terminal cap refusal. No large transfer, allocation exhaustion or remote denial-of-service executed. Gopher dependency excerpt is source traced, not whole-file Mere coverage. Repair scope: bounded transport API and progress/cancellation migration can exceed one-hour diversion. Carry a byte budget to the actual protocol acquisition or stream into a bounded sink that cancels on overflow. Gate progress and terminal buffering under the same request budget. A final length check and suppression of excess progress do not bound upstream allocation or stop the transfer.

Q187: Identity download silently decodes an unexpected encoded body and reports the encoded length. The handle sends Accept-Encoding identity. A supplied gzip response contains38 encoded bytes; read_into succeeds with18 decoded bytes in its sink and Facts.content_length=38. The maintained crawler rejects the same response as UnsupportedEncoding. Netfetcher decodes before the handle sees the body, and the no-store response retains encoded Content-Length. The handle comment claims identity makes the declared length equal the length written.

Current real decode pipeline with a supplied one-hop transport and bounded gzip fixture, no live HTTP/TLS wire or user file. The observed mismatch is in successful custody metadata; no arbitrary server protocol compliance is assumed. Repair scope: response-representation admission must be shared before decode; can exceed one-hour diversion. Choose whether downloads require identity/raw representation or expose explicit encoding and decoded-length facts. Refuse unexpected encodings before transformation for a raw-custody contract, or revise Facts and caller integrity/range assumptions. An outbound request header alone is not a response guarantee.

Q188: The five-redirect Smolweb limit actually allows only four redirects before a final response. A local Spartan chain with four redirects and a final success uses five requests and succeeds. With five redirects, the function performs five redirect requests and reports too many redirects before asking for the final target. The constant is documented as the most redirects it follows, but the loop counts total attempts.

Two actual loopback Spartan chains, no external origin or mutation. This is a bound-definition/off-by-one mismatch; no unbounded traversal claim. Repair scope: contained redirect-versus-attempt counting correction. Define redirects and initial request separately and test zero, boundary and overflow chains. Preserve per-hop timeout and identity scope checks when adding the terminal allowed attempt.

Q189: A matching Titan page identity is admitted by the host type but refused by its Gemini-only fetch helper. GeminiClientIdentity accepts a Titan origin and its scope predicate treats Titan as matching. A Page command for that origin with the identity routes to errand::fetch_url_timeout_with_identity, which only permits Gemini and returns unsupported scheme titan before connecting. An owned listener confirms no incoming connection; dummy key material is never used in TLS.

Actual spawned Fetch actor command and typed terminal result, matching Titan origin, supplied NoTransport for HTTP and an unaccepted loopback listener. No Titan upload, real certificate exchange or identity leak executed. Repair scope: read/write capability and identity-route decision can exceed one-hour diversion. Reconcile accepted identity families with the operations that use them. Keep explicit Titan upload-with-identity and the A311 zero-byte generic-fetch write boundary in view; do not broaden a read helper to perform more implicit writes merely to make this path return success.

Q190: Lossless flip-cookie export erases host-only versus domain scope. Two real CookieRecord states differ only in host_only. The host-only state yields no child-host cookie; the domain state does. session_cookies_for exports identical portable Cookie vectors for both. The source jar retains the distinction, but both portable cookie DTOs omit it while their comments claim session/metadata losslessness. This strengthens A262 from an absent-field concern into an executed export collision.

Synthetic source jar, actual child-host matching and public portable export comparison. No WebView import, subdomain request, real cookie or confirmed browser cookie-scope broadening. Repair scope: portable cookie schema/backend capability migration can exceed one-hour diversion. Carry host-only scope through the portable cookie vocabulary and adapter/capability boundary, or explicitly report/refuse the unrepresentable loss. Do not infer preservation from two DTOs mirroring the same incomplete field list. Keep partition-key/CHIPS decisions separately explicit.

The host-selected Stores seam works: shared Arc-backed cookie/cache/HSTS/Alt-Svc/Transport owners feed both handles and the actor, and original controls observe scoped cookies, redirects, no-store ranged reads and supplied wire use. The process-global cookie jar/shadow helpers are a separate legacy ownership model. Durable persona prefixes do not partition the jar or make one global equality cache current for each store/persona. Q179/Q180 expose that actual conflict; Q181 isolates expiration cleanup from authentication admission. The latter is correct in the source jar even when the durable blob remains. Current Mere production caller search did not establish a running persona-switch consumer of these helpers.

A262 now has a concrete follow-up: host-only and domain cookies have different native child-host matching yet the actual flip export is identical. The flip and surface Cookie DTOs both omit this distinction while claiming metadata/session losslessness. This is an executed export-loss proof, not a WebView import or a confirmed subdomain security bypass. The declared Partitioned limitation remains a different issue. Schema/backend capability migration can exceed the audit diversion limit.

The maintained crawler reuses the extracted HTTP wire implementation. Its separate rooted, retry-disabled pool is deliberate policy, not a duplicate hyper transaction. Current Fleece uses that admitted single-hop path. A public legacy helper still changes only the browser-session User-Agent and follows redirects with cookies, while claiming active crawler ownership in its prose. Q185 proves divergence without attributing the old path behavior to the current Fleece caller. Existing anonymous page fetch also passes the no-cookie control.

The common Smolweb response path genuinely shares timeouts, redirect/status handling and certificate scoping between pages and binary subresources. Typed input and certificate conversations are retained by this producer, superseding historical memory about string-only results at this layer. Q188/Q189 show bound/identity mismatches within the shared path. Generic Titan still delegates to a zero-byte upload in its supplier (A311); broadening identity dispatch without settling read/write capability admission would be a poor local repair. The final Smolweb cap and suppressed progress do not cancel upstream reads. Actual registry Gopher uses unbounded read_to_end; Q186 gives a bounded eight-byte/pending-before-EOF observation rather than attempting exhaustion.

NetFetch Range correctly obtains identity encoding through Netfetcher header assembly, so a missing local header is not a second encoding bug. Q182 is the separate vector capacity trust error. Downloads differ: an unexpected gzip response is decoded but retains its encoded length in no-store facts; the crawler explicitly refuses the same encoding. Requesting identity is not proof that the response obeyed it. ResourceResponse retains final URL/type, while legacy page/byte result projections lose some response identity needed by downstream consumers; Q156 is reinforced at its producer rather than repaired with another host sniff table. Read page-load next.

The optional HTTPS adapter has a real max+1 byte cap and a caller-configurable separate ureq agent, but its named HTTPS-only scheme admission is absent. Its async implementation blocks synchronously, and no local default timeout is set in this source. IrohFetcher reuses host BlobStore/Transport and does not create another node; it parses a narrow local hex ticket and reads the fetched content-addressed blob. Manifest verification belongs to resolve_blob when that path is used; a direct BlobFetcher call is not automatically a second manifest-level check. Blocking/timeout/whole-blob budget policy and actor concurrency/deadline completion remain source-qualified larger follow-ups, not reproduced hangs or OOM events.

Fresh all-feature library build and 36 original library tests pass. A separate no-default library gate passes16 tests. The no-default normal dependency tree names none of armillary/eidetic/pandect/inker/errand, verifying the governing D3 feature cut; this does not describe the much larger test-only dependency graph or wasm. Eleven generated public API observations pass and expose12 recorded findings. Cookies/backend contents are synthetic/in-memory, all generated HTTP/Gopher/Spartan connections are owned loopback fixtures, and the Titan dummy identity is refused before a connection or TLS use. The header capacity panic is caught without huge allocation. The original all-feature suite separately includes two Iroh peers constructed in-process and a successful synthetic blob transfer. No browser/GPU/WebView import, real credential, remote user content or durable user file was used by generated probes. Probe executable/PDB removed; stable target reused, no isolated target, Cargo home or worktree. Evidence/source hashes are in the [Fetch review receipt](../../../support/receipts/authority-success-audit/fetch_review_receipt.json).


### Read-only page-load, Framing and content-contract review, 2026-10-16

All 8 inventoried files in page-load (3), Framing (3) and content-contract (2) are read, adding 88,797 characters. Repository coverage is 1,153/3,352 files and 15,407,548/36,854,820 characters. Full-source objective remains active. Production and original tests/concurrent work are unchanged. Resume at crates/system/state-witness/Cargo.toml. Authority entries A331-A346 distinguish actual shared state/vocabulary, competing MIME/currentness/asset identity, explicit DTO compatibility, planned header adoption and browser-only verification limits.

Q191: PageLoad sends registered renderable Nex, Guppy, Titan and JSON Feed documents to download. Actual default EngineRoutePolicy plus EngineRegistry renders nonempty documents for application/x-nex, application/x-guppy, application/x-titan and application/feed+json using their registered engines. PageLoad accepts successful outcomes for the same address/MIME/body but returns Download and retains no document for all four. Text/gemini and application/gopher-menu controls remain renderable. The MIME list claims it selects types a document engine cannot render but does not consult the chosen engine or registry.

Pure current library route/registry/render/load composition, no transport, host UI, browser or Titan write. Fetch source already read assigns these fixed protocol MIME types. Pelt source excerpt actually delegates terminal outcomes to PageLoad; the Pelt controller path was not executed here. Repair scope: shared renderability/classification owner decision can exceed one-hour diversion. Let host-resolved engine capability and explicit attachment disposition determine download, or pass a resolved classification into the transport-free PageLoad. Preserve its zero-dependency role. Adding four more MIME exceptions would retain the competing authority and fail again as engines change.

Q192: A new same-address request inherits response metadata from the preceding request. A retained response at time10 with an old redirect/MIME is followed by begin2 and a new nonstreaming body at time30. The new bytes receive time10 plus the preceding effective URL and MIME. Fresh progress at time40 updates its response URL but also retains time10. begin clears the stream but retains the document; metadata lookup keys only by address and does not distinguish the new request. Explicit forget_fetched is a working control.

Public PageLoad API with synthetic timestamps and bodies. Current Pelt explicit Reload source calls forget_fetched first, so that inspected path is a real mitigating control; no bad Pelt reload, live source-capture timestamp or remote redirect was reproduced. A direct new request at the same address still violates the LoadedDocument admission-time meaning. Repair scope: request-bound observation correction; broader capture consumer review may exceed one hour. Separate the last presented document from metadata observed for the active request. Retain old content for failure/Stop presentation while associating admission time, effective URL and MIME fallback with the current request epoch. Do not erase useful old content merely to make response identity current.

Q193: Scene transfer reuses a per-frame image key as permanent asset identity and restores old image data. Two valid document image packets lower through the real PaintList translator into scenes with the same recycled key: first is red1px, second is blue2px under another source URL. Reusing the documented per-worker encoder/decoder makes the second transfer succeed with the first red bytes and width1. A fresh-cache roundtrip preserves the blue bytes and width2. The encoder HashSet remembers only ImageKey; the decoder overwrites the new stripped scene metadata with cached dimensions/blob data. Actual producer keys restart for each list, so this is not an invented mutable immutable-blob ID.

Real packet-to-paint-list-to-Scene-to-transfer/replay chain and exact resource bytes/dimensions, CPU only. No GPU upload/raster screenshot, browser worker or current product image corruption claim. Current Mere caller search finds the transfer cache definitions/tests rather than a live worker caller, so this is a public pipeline defect with consumer adoption unproven. Repair scope: contained descriptor-aware cache invalidation; shared asset identity migration may exceed one-hour diversion. Compare the full current asset descriptor/content identity and resend or invalidate changed data, or establish a genuinely stream-wide immutable asset ID distinct from frame-local draw keys. Account for size, content revision and worker/cache reset together. Never use a per-list slot alone as proof the receiver already has this image.

PageLoad is a useful transport-free shared state owner and Fetch re-exports its exact vocabulary. Original stale-request, Stop, split-UTF8, retained-conversation and final-prefix tests pass. The MIME download table is nevertheless another independent parser/format decision and disagrees with actual registered renderers. Response bytes and decoded rendering are separate views; effective URL/admission-time fallback is not request-bound merely because terminal ingress uses the right ID. The public lower-level note/surface/fail methods are trusted state setters, not authenticated ingress. No stale live surface callback misuse was demonstrated. Prefix retention after Stop/failure is explicit behavior; it should not be confused with completed acquisition when a capture consumer reads LoadedDocument. Repeated prefix cloning/decoding and capture completeness need source/caller qualification, not an unmeasured performance or persistence claim.

Framing has a real small shared helper with the chosen Wing layout. Fresh controls check magic/version before payload. Trailing bytes remain accepted and the separately selected legacy path interprets nonmatching magic as version0 raw bytes; a foreign framed blob can therefore decode as a legacy u8 prefix. Both are documented compatibility semantics, explicitly distinguished in the governing framing plan. They are not new defects or proof of schema authentication. F4/F6 require stored/sent postcard headers, but the plan accurately says only H1 is landed and H2/H3 migration is open. content-contract still uses raw positional enums and no Framing dependency. A legitimate Scroll command also decodes as a TextSelection update through the other public decoder: envelope direction/schema come from channel context, not the payload. No production misdelivery, forged authority or false completed-migration claim is asserted. Adoption needs old-byte compatibility fixtures and coordinated hosts, so it stays documented.

The content command/update mirrors are explicitly separate wire schemas. Graph properties carry full current typed literal/statement metadata. The appended ContributionV2 and retained legacy ordinal preserve multiple edges and lifecycle/basis distinctions; fresh original tests include a hand-derived fixed old byte fixture. This is compatibility duplication requiring maintenance, not an additional durable graph authority. GraphScope and permission DTOs are reused directly. Navigation/revision fields differ by update purpose; whether resource or semantic contributions survive a navigation is a host policy not authenticated by the codec. No unauthorized grant, stale graph ingest or late resource application was executed.

The image transfer failure upgrades the earlier document producer per-list key-reset candidate into actual downstream evidence. Each valid list is self-contained, but its resource slot is not permanent stream identity. A fresh cache roundtrips correctly while a reused per-worker cache silently restores earlier image bytes and dimensions on a successful later frame. Asset content revision, frame-local slot, immutable blob ID and receiver residency need distinct treatment. Document packets ship a full font table; Scene updates strip assets and rely on delta caches. These are different payload paths, so one cache claim cannot qualify both. Scene statistics count the stripped snapshot, not total outer/asset transfer bytes. Cache acknowledgements, malformed-frame state changes, restart/reset and residency limits remain larger source-only concerns; no live worker caller was found in current Mere search.

Combined current Cargo library build succeeds with content-contract/scripted enabled. Original suites pass10 page-load,11 Framing and5 content-contract tests. Five native API observations pass, including three reproduced defects and two boundary qualifications. The image path is actual packet/PaintList/Scene serialization and replay with byte/dimension comparison; it is not a GPU raster screenshot. Browser wasm helpers explicitly copy between wasm and JS and then transfer the JS ArrayBuffer, so the source does not claim a copy-free entire path. No browser/worker, native UI, GPU, script, network or user persistence ran. Pelt, renderer dependency and governing format-plan excerpts are source traces, not whole-file reading credit or live host receipts. Probe executable/PDB removed; stable shared target reused, no isolated target, Cargo home or worktree. Evidence and source hashes are in the [load/contract review receipt](../../../support/receipts/authority-success-audit/load_contract_review_receipt.json).


### Read-only state-witness, Proofs and Mesh namespace review, 2026-10-16

All 8 state-witness files,2 Proofs files and the full Mesh namespace module are read,11 files and56,519 characters added. Repository coverage is1,164/3,352 files and15,464,067/36,854,820 characters. Broader Mesh is incomplete. Full-source goal remains active; production and original tests/concurrent work are unchanged. Resume at crates/mesh/mesh/Cargo.toml. Authority entries A347-A360 distinguish replay equality, explicit extraction scope, typed proof vocabulary, backend residency, resource verification and source/test limits.

Q194: Out-of-order unique witness entries are falsely reported as duplicate labels. Witness::try_from accepts an owned list with distinct site:10 and site:9 labels and refuses it as DuplicateLabel(site:9), although that label occurs once. The list is invalid because natural order is reversed. Ordinary insertion sorts the same labels and framed reload succeeds. The Error documentation says DuplicateLabel means the witness already holds that label.

Public list admission and current strict-framed roundtrip control. Invalid entries remain refused, so this is a misleading diagnostic rather than invalid-state admission, false equality or data loss. Repair scope: contained load-error distinction, preserving the ruled insert-error contract. Distinguish ordering refusal from duplicate identity in load diagnostics. Preserve F124's existing insert errors and review consumer matches before choosing a new public load-error shape; do not silently sort malformed stored input or call distinct entries duplicates.

Q195: Mesh ambient blob presence and reads ignore the typed algorithm and declared length. After storing synthetic bytes, changing the BlobRef algorithm to Sha256 or increasing byte_len makes BlobRef::verifies false. Ambient MemoryBlobSpace::has still returns true and get returns those bytes, because only the raw32-byte array becomes a muniment BLAKE3 key. JobSpec validation also admits each reference through its width-only check. The actual JobNamespaceView then returns DigestMismatch before the resource sees either input. Stickleback's separate get_blob implementation explicitly checks BLAKE3 and reference.verifies.

Current real Mesh/Proofs/Muniment in-memory APIs with valid, wrong-algorithm and wrong-length references. The namespace file is fully read; broader Mesh/spec and Stickleback/courier excerpts are not whole-file coverage. The resource-facing guard was executed and works: no permission bypass, unverified adapter input, peer exchange or durable user blob was demonstrated. Repair scope: contained typed-reference admission correction; shared store-adapter ownership review can exceed one-hour diversion. Apply BlobRef semantics consistently at host readiness/lookup boundaries, distinguishing mere raw-key residency from a held valid typed reference. Preserve the existing resource-view verification. Reuse BlobRef::verifies and explicit supported-algorithm admission rather than treating as_32 as a verifier or creating another weaker store-specific hash rule.

State-witness is correctly scoped to labelled non-cryptographic replay equality. Natural ordering, NUL/duplicate rejection, immutable private storage, serde TryFrom validation, framed schema/version checks, first label/tick divergence and bit-preserving value hashing are real implementations with fresh original controls. Raw byte and postcard sequence hashes intentionally differ. The helper does not canonicalize arbitrary producer collections or identify a Rust type/schema inside hash_value; producers own that stable shape and iteration. The plain String label helper is an explicit F127 ruling. One slash-bearing part collides with two path parts, as the generated observation shows, but no actual Mere producer or external Isometry label collision was found/tested. This remains a producer grammar boundary, not an invented cryptographic or application identity bypass.

F119 chose State-witness as the shared FNV owner for Isometry adoption and explicitly left Mere's existing hand-written copies outside that move. Current candidate sites are retained in witness_fnv_candidate_sites.log, including Cambium, Pictograph, Scenomise, linked-data, ESP and document-host. These are repeated primitive sites, not automatically equivalent domain canonicalization contracts; the search also finds tests/constants. No completed-all-Mere-retirement claim was found and no blanket replacement is authorized by that ruling. The independent test intentionally copies the old isometer function and includes an off-prime negative control. Fresh equality against that frozen reference does not establish current external repository adoption; no such adoption was checked.

Proofs is a vocabulary plus direct BLAKE3 verification, not a universal proof engine. Whole BlobRef/Commitment checks retain their required algorithm/length/domain/scheme/version facts. Other scheme variants are tags awaiting their actual owner verifier. as_32 checks width and returns an untagged array; the original Digest still carries its tag, but the array is not an algorithm-safe identity. Q195 traces an actual weaker lookup in Mesh, and Stickleback independently implements the stricter reference check over the same backing substrate. The existing JobNamespaceView then enforces full input verification before adapter access, so the host readiness/lookup inconsistency is explicitly separated from a resource authority bypass. JobSpec currently accepts the bad references through its width-only validation. Courier conversion/tag excerpts show a related follow-up boundary, not an executed delivery/retention failure.

The namespace view exposes only named input reads and one bounded output slot after a host-owned grant. Fresh six original controls prove wrong names/size are refused before sink writes and lying input bytes are refused before adapters. The new invalid-algorithm/length controls confirm the same guard. Host authorization before grant and sink commit/durability are preconditions, not additional authority supplied by the view. No permission resolver, signed operation acceptance, network peer or user blob mutation was run here.

Current library/composition builds pass. State-witness original suites pass26 tests (formats5,hash6,trace5,witness10), Proofs passes2, and the Mesh namespace filter passes6 with117 other Mesh tests unexecuted. Four generated native observations pass: two qualified findings and two explicit format/tag boundary checks. The current normal State-witness graph excludes heapless/hash32/byteorder/spin/lock_api/scopeguard/stable_deref_trait, confirming the later dependency-floor correction. Historical wasm32/clippy/cross-repository landing receipts remain historical; this pass makes no fresh wasm32 or cross-machine replay claim. Full Mesh source/suite is not qualified by a module test. All probe storage is synthetic MemoryBackend; no filesystem/backend user state, signatures, peer/network, browser or GPU ran. Executable/PDB removed; stable target reused, no isolated target, Cargo home or worktree. Evidence and source hashes are in the [witness/proofs review receipt](../../../support/receipts/authority-success-audit/witness_proofs_review_receipt.json).


### Read-only Mesh manifest and examples review, 2026-10-16

The Mesh manifest and all 11 inventoried example files are read,12 files and 114,742 characters added. Mesh coverage is 13/43 files including the previously read namespace module. Repository coverage is 1,176/3,352 files and 15,578,809/36,854,820 characters. Full-source objective remains active; production and original example/tests/concurrent work are unchanged. Resume at crates/mesh/mesh/src/board.rs. Authority entries A361-A372 separate imported suppliers, stale provenance, scoped rehearsals, inner record authority, queue/durability/repeatability observations and actual audit gates.

Q196: Both Mesh retained-ingress receipt examples fail to compile after the Murm supplier extraction. Fresh locked/offline inventory_ingress check with admitted-ingress fails on2 deleted crates/murm/transport source includes. Fresh retained_receive check fails on4 equivalent deleted includes. All6 compile-time provenance paths resolve inside the former Mere tree and do not exist; the actual transport supplier is an external locked Murm Git package. Positive control checks the admitted-ingress library and mesh-peer example successfully under the same workspace/toolchain.

Actual native Cargo check gates:2 failures and1 positive control. No Retinue executable or network campaign ran. These are unbuildable receipt paths, not a failed codec/owner admission experiment or a claim Mesh itself cannot compile. Repair scope: contained provenance path/supplier binding correction. Bind provenance to the resolved supplier actually compiled and keep selected local owner sources distinct. Do not recreate a private crates/murm copy merely to satisfy include paths. Recheck both targets and their metadata before presenting historical composition receipts as current reproducibility.

Q197: Retained-receive provenance hardcodes a p2panda revision that differs from the locked compiled supplier. The report field is the fixed revision d8acabc96ecc59016e40edf4a47ed5eaf5360285, while both Cargo.lock and fresh locked/offline Cargo metadata select mere-p2panda-net0.7.5 at 1f69c9bcb9f2a9d798b1c25b2ec2f47c0416bd65. No assertion or lookup connects that literal to the library used by this build. Hashing Cargo.lock/source elsewhere in the report does not make this named revision field correct.

Literal source versus current lock and resolved metadata. Q196 currently prevents this example from compiling, so no newly emitted report carrying the false value is claimed. The mismatch remains after merely repairing the missing includes. Repair scope: contained supplier-derived provenance correction. Derive the reported supplier identity from the exact build closure, or explicitly label historical fixture revisions. Keep any runtime checkout observation separate from compile-bound package identity; avoid freezing dependency facts in success JSON literals.

The two old and new handoff experiments share the actual MeshStore/operation admission owner rather than maintaining an alternate graph store. Carrier/transport subject is checked distinct from signed operation author. Strict CBOR plus re-encoding, original record/header/body equality, wrong-mesh/body/signature refusals, duplicate owner behavior and redb reopen checks are explicit source controls. The retained carrier names in-process close/reopen; inventory adds an independent-process --reopen child. Native UI, physical radio, worker execution and blob delivery are explicitly outside the experiments. No such campaign was run here, and the supplied external Retinue decoder source was not independently audited. Recovered bytes/flags/executable identity are harness evidence requirements, not fresh audit execution.

The newer inventory lane uses shared admitted ingress and an exact retained-object inventory projection, with original opaque bytes submitted to the same owner. It tracks action permissions, owner-call counts, capacity refusal, duplicate identity and active/pending limits. The older QUIC carrier is finite and serialized locally and explicitly denies a global production queue bound. It validates exact framing error categories and rejects infrastructure failure as evidence; the inventory helper only observes lack of receipt/owner calls after an admitted handshake for its invalid frames. Its later healthy controls matter. These are different evidence strengths rather than interchangeable receipt labels. Active route checks reject out-of-scope routes and only retry absence of active path state.

Current code has typed namespace, resource and leased-work contracts, so the historical June M1-only description is stale as a description of this checkout. The manifest/CLI still use the M1 rehearsal label. mesh-peer deliberately posts inline legacy work because it supplies no blob delivery, and declares DevicePolicy::unsupervised because it cannot heartbeat or cancel inline execution. Unexpected Grant/Heartbeat/Reclaim is a hard failure. This is an honest transport rehearsal boundary, not a complete leased host. Its connected console line still needs supplier guarantee review: registration/topic success is not independently observed readiness in this helper, although later sync counters and Done wait use actual state. No unreachable ticket was tested.

Two provenance implementations have drifted: compile-time source catalogs still refer to deleted in-repository transport files, while the inventory Python wrapper knows how to resolve the imported supplier. The retained carrier also freezes an obsolete p2panda revision in success JSON. Source/executable/lock hashes elsewhere do not make that named field true. Runtime git observations and source metadata should remain separate from exact build-bound supplier identity. Paired runs compare a named semantic subset and explicitly exclude timing/actor observations, which is an honest repeatability scope. No current paired run exists because the example check gate fails.

The persistent Source helper uses bounded lines/lifetime bytes, monotonic request/time identity, deadlines and Tokio kill-on-drop. The older one-shot peer helper uses std child plus three pipe threads; parent timeout precedes unbounded joins and early errors have no equivalent kill-on-drop guard. Descendant-held pipes or blocking-runtime consequences were not reproduced. A shared bounded IPC lifecycle, provenance schema binding and real host scheduling are larger follow-ups to document rather than prolong this audit with an implementation rewrite.

Actual fresh locked/offline Cargo checks: inventory_ingress with admitted-ingress fails on 2 missing includes; retained_receive fails on 4 missing includes; the admitted-ingress library and mesh-peer positive control pass. Fresh Cargo metadata and the unchanged lock select p2panda 1f69c9bc, while the report literal says d8acabc9. The static mismatch is not a newly emitted false report; failed targets were not executed. No external peer, network, codec recovery, redb, child process campaign, user file or physical device ran. No owned runtime probe/output directory was created. Stable shared target reused; no isolated target, Cargo home or worktree. Evidence and source hashes are in the [Mesh examples review receipt](../../../support/receipts/authority-success-audit/mesh_examples_review_receipt.json).


### Read-only Mesh board, lease and checkpoint review, 2026-10-16

Ten whole Mesh source files add 197,076 read characters: board, directory, fold, lease, crate facade, projection, retention, spec, store and wire. Mesh is 23/43 files complete; repository coverage is 1,186/3,352 files and 15,775,885/36,854,820 characters. Full-source objective remains active. Production and original tests/concurrent WIP are unchanged. Authority entries A373-A384 map the split claims and accepted-fact/replay owners. Resume at crates/mesh/mesh/src/drop_export.rs.

Q198: Unleased completion overwrite violates the order-independent board contract. Two correctly signed sequential JobDone operations by the same valid claim winner carry A and B. Folding the identical set as post/claim/A/B returns Done(B); post/claim/B/A returns Done(A). Results are gathered by author and overwritten in input order, unlike claims keyed by operation hash. The board/module overview promises order independence.

Native direct public fold reproduced. Both operations form an ordinary signed author chain. Actual MeshStore iterates indexed author logs in sequence order, so this does not establish live store peer divergence or an accepted author-log fork. Repair scope: semantic result arbitration decision; broader than a mechanical overwrite fix. Name a deterministic rule for multiple winner outputs and mixed generations, retain the selected operation identity, and qualify permutations and checkpoint replay. Do not silently choose first/last by timestamp or hash as an unruled protocol change.

Q199: Parsed addressing and cached signed extensions disagree across verification and storage. An operation signed for foreign mesh F is initially refused by accept(M). Mutating public header.extensions.mesh_id to M leaves encoded signed bytes and cached operation hash unchanged, yet mesh::verify succeeds, direct fold(M) admits it, and real MeshStore::accept(M) reports insertion. Reading the retained header decodes F; ops(M) contains one record while board(M) is empty. The application policy indexes parsed extensions while durable bytes use the supplier cached CBOR extensions.

Native public API and real MemoryBackend store reproduction with synthetic valid signatures. No forged signature, corrupted wire acceptance, remote network ingress exploit or durable user store is claimed. Exact compiled p2panda supplier is 0.7.1 from 1f69c9b, not historical upstream API recollection. Repair scope: shared supplier/admission consistency repair, potentially over one hour. Establish one immutable or canonical decoded header boundary across p2panda typed extensions, Mere preflight, policy selection and persistence. Re-check all consumers and test features; a one-off mesh-id check would leave the shared representation problem in place.

Q200: Accepted checkpoint reconstruction loses completed leased output. A valid leased job reaches Committed. A non-live completed snapshot passes retention policy. Empty-tail reconstruction yields Claimed while lease_at still says Done. The actual store build_checkpoint/accept/board sequence reproduces the loss before any pruning, while the original completion operation remains retained. The snapshot output is seeded into the unleased results map; leased completion lookup only searches newly gathered completion facts.

Direct snapshot API and actual MemoryBackend store checkpoint path. This is a completed lease, outside the explicitly rejected live-Held checkpoint case. No network, redb reopen or physical persistence claim. Repair scope: contained restoration symptom with checkpoint contract implications. Preserve the accepted completion/output identity through snapshot reconstruction and later tails. Verify checkpoint migrations, already terminal jobs and missing completion bodies before promoting a local state fallback as a complete retention repair.

Q201: Leased result lookup publishes an output rejected by grant admission. The lease resolver validates output against the signed spec before forming a Completed end. A subsequent completed_output lookup independently chooses the first operation-hash-ordered completion matching holder/lease, without validation or matching the selected closing fact. A valid 5-byte completion closes at time400; an otherwise well-formed 17-byte output against a16-byte grant is ignored alone but published as Committed when paired with that valid completion and a lower hash. The real store accepts both structurally valid records and its board exposes the invalid output. Bounded search found the lower hash in7 attempts.

Native direct fold and actual MemoryBackend admission/board. Invalid output remains self-valid; job-specific admission is intentionally delegated to the fold. Existing single-invalid-result tests pass and do not cover coexistence with an accepted completion. No worker, blob transfer or peer execution was needed. Repair scope: bounded accepted-fact/output binding repair; retain exact resolver identity. Return or retain the selected accepted completion identity/output from lease resolution instead of performing a second weaker selection. Validate competing valid/invalid outputs, outside-window facts, equal-time tiebreaks and snapshot restoration under that one selection owner.

Q202: Checkpoint frontier removes device attestations from replay without snapshotting the directory. JobBoardSnapshot clones only jobs; fold_from_snapshot initializes an empty DeviceDirectory. A real store with one valid master-signed self-attestation reports one device. build_checkpoint includes that event in its frontier; accepting the checkpoint changes board.devices to empty even though the attestation operation remains retained. All events below the frontier are filtered out before the fold, and no snapshot directory is available to restore them.

Both snapshot-only API and actual MemoryBackend checkpoint acceptance reproduced. Device cryptographic admission remains correct; the observed loss is replay state coverage, before pruning. Consumer scheduling/UX effects and remote catch-up were not executed. Repair scope: snapshot version and replay-coverage repair, potentially over one hour. Choose whether the governed snapshot includes authenticated device facts or retention separately preserves/replays attestations outside the job frontier. Coordinate encoded checkpoint version/commitment compatibility and catch-up export; avoid adding a second private directory owner.

The strongest duplicated authority is within leased completion handling: one resolver validates the signed grant and chooses the accepted end, then another lookup selects an output using weaker conditions. Q201 executes both the direct fold and real store path. The synthetic over-grant record is correctly ignored on its own; the defect requires a valid closing completion alongside it. Selecting the accepted completion once is the repair seam, with competing facts and snapshots covered together.

Q199 is a representation split across the actual shared supplier/processor/policy/storage boundary. The supplier retains original CBOR extensions while exposing mutable typed extensions. In this production-feature build, the cached hash and already-decoded provenance assumption do not bind the current typed address. The existing wire negative changes encoded bytes, which is a different test from mutating the typed field. Exact decoded-wire signature controls are not alleged broken. Keep a supplier-level consistency repair separate from claims of a network exploit.

The shared MunimentStore is real reuse, and directory attests is one rule called by both admission and fold. Merely repeating a call is not duplicated policy. The contradictions are the output selector and incomplete checkpoint state: cryptographic checkpoint self-consistency and authorized frontier advancement do not prove that all represented state can be reconstructed. Q200/Q202 occur on the normal owner-minted checkpoint path, before physical erasure. A JobBoardSnapshot includes only jobs; the board also owns its device directory.

Declared boundaries remain distinct: live Held checkpoints are refused, signed lease facts are clock-free, liveness takes explicit observer time/skew, heartbeat silence is not worker failure, and own-device clocks are trusted rather than adversarial. Snapshot winner freezing is explicitly intended, so lower-hash late claimant behavior needs a semantic ruling rather than an arbitrary change during audit. Expired/released epoch continuation, historical heartbeat projection and shared backend mesh-id/log aliasing remain source-qualified candidates with no fresh reproduction. Their potential snapshot/log/schema repairs can exceed the one-hour diversion threshold and are documented.

Fresh gates at 4523cb6f: locked/offline default mere-mesh library build succeeds; all 123 original library tests pass; nine additional native CPU/API observations reproduce the five findings. Passing diagnostic assertions mean the defects were observed, not repaired. Synthetic signed operations and real MemoryBackend only; no network, live peer divergence, redb user store, browser/GPU, worker/blob delivery or physical radio. Probe executable/PDB removed; shared stable target reused with no isolated target, Cargo home or worktree. Required retained receipts are the compact build/test/probe logs, source/coverage hashes and [review receipt](../../../support/receipts/authority-success-audit/mesh_board_review_receipt.json), under the default 100 MB total retained-byte cap (entire audit evidence currently about 12 MB). No cited receipt is replaced by the subsequent main synchronization.


### Current main refresh and Mesh requalification, 2026-10-16

The audit was at 4523cb6f when a fresh fetch found 14 incoming origin/main commits through 641f684a, alongside 5 local-only commits. The clean merge is 04474731. Prior local history is an ancestor, and 36 unrelated dirty/untracked file hashes are unchanged. The only incoming/dirty overlap was this audit document: its append-only dirty suffix was preserved exactly around the incoming tracked additions, with the original full bytes retained as bounded recovery evidence. Only that document was temporarily stashed; the owned stash is removed. No production repair, branch/worktree, isolated Cargo home or isolated target was created.

Mesh and Stickleback source are unchanged by the merge; the workspace manifest and lock changed. Fresh locked/offline current-main library build succeeds, all 123 original Mesh library tests pass, and all 9 native public API/MemoryBackend diagnostic assertions pass again. Q198-Q202 remain reproduced. Passing these diagnostic assertions means observing the defects; neither live network divergence nor forged wire-signature admission is claimed. Prior build/probe/review receipts retain their original hashes and basis rather than being overwritten as current.

Fresh source inventory has 3,355 files and 36,897,906 characters. Thirteen previously read source files changed upstream (workspace manifest, ten Pictograph source/test files, two Pandect files; the explicit receipt list is authoritative), so their old read credit is removed pending reread. Current coverage is 1,173/3,355 complete files and 15,421,049/36,897,906 characters. Mesh remains 23/43 files and 325,472/556,938 characters read. The next Mesh file is drop_export.rs; changed earlier areas also remain in the full-source obligation. The lower current coverage reflects freshness invalidation, not discarded reading evidence.

The [current main review receipt](../../../support/receipts/authority-success-audit/current_main_review_receipt.json) binds the new gates, preserved WIP, source invalidations, original receipt chain, default 100 MB audit evidence cap and generated-output cleanup. No cited receipt is overwritten. Full-source objective remains active.


### Complete Mesh source and execution-authority review, 2026-10-16

All 43 inventoried Mesh source/manifest/example/test files are now read, 556,938 characters. This pass adds 20 previously unread Mesh files 231,466 characters and rereads the current root Cargo.toml 53,872 characters after the upstream merge. Repository current coverage is 1,194/3,355 complete files and 15,706,387/36,897,906 characters. Twelve changed Pictograph/Pandect files still need full reread; next is crates/canvas/pictograph/src/canvas.rs. Full Mere source objective remains active. Production and original tests/concurrent work are unchanged. Authority entries A386-A401 distinguish real sharing, divergent admission, cancellation/verification claims and catch-up anchors.

Q203: Legacy worker selection bypasses the configured resource allowlist. With allowed_resources containing only mesh.echo/v1, an M1 Blake3 post is still considered runnable, selected for Claim and then Execute. The equivalent V2 Blake3 spec correctly returns Idle. The legacy branch checks only payload presence, registration and adapter hardware requirements; V2 calls DevicePolicy::accepts.

Pure worker/board public APIs, valid synthetic signed operations and built-in registry. No actual excluded workload, host daemon or remote peer executed. This is a concrete configuration bypass, not a claim of untrusted remote GPU execution. Repair scope: bounded common legacy/V2 policy admission correction. Apply the configured resource rule after mapping the legacy kind to its resource. Keep any implied legacy checkpoint-class rule explicit. Verify both generations under the same policy, preserving the single actual run_legacy -> run_job executor.

Q204: The worker starts held work before checking owner withholding or capacity. A valid live lease held by this author but absent from HostOffer.running yields Execute while conservative policy reports ForegroundActivity and accepting=false. The same held-but-unstarted job yields Execute when max_concurrent_jobs=0. When the job is placed in running, the owner-withholding control correctly yields Reclaim. Reclaim only checks running jobs and the start gate occurs after the Execute loop.

Pure public decision function with an actual folded lease. The probe establishes a start decision contrary to current settings; no native OS supervision, physical workload continuation or broader host race was executed. Repair scope: bounded scheduling gate correction with a held-versus-running distinction. Separate keeping an already running job from starting a held reservation. Owner withholding must prevent a start; capacity should prevent an additional run without reclaiming work already active. Keep heartbeats and reservation release policy deliberate rather than suppressing all lease maintenance.

Q205: Cancellation during the final real-adapter await still commits output. The real DelayedResource(1,100) is manually polled to its first Pending after reporting done=1. The host cancels before the next poll, and JobControl confirms Cancel. Awaiting the run nevertheless succeeds and stores the expected output. The adapter has no cooperative point after its final sleep; the runner has no final control check before commit. Existing cancelled-run tests only cancel before execution.

Deterministic cancellation ordering through the real built-in adapter and public runner, synthetic memory blobs. No malformed adapter or timing race is required: cancellation precedes execute returning. A stronger atomic cancellation-versus-sink-commit guarantee remains a separate design question. Repair scope: bounded final cooperative check; full commit linearization can exceed one-hour diversion. Check control at the final cooperative boundary and before handing returned bytes to the sink. Define cancellation arriving during an in-flight sink write separately; do not promise rollback of a committed blob merely by adding one flag read.

Q206: Exact verification verdicts ignore the rerunning adapter weaker class. An Observed spec legitimately runs a ProducerOnly local adapter. Its ordinary output is NotCheckable, but an ExactBytes producer output for that same resource causes this local adapter to emit Reproduced for matching bytes and Diverged for different bytes. verify_output inspects only the producing output class; run_job only requires the local adapter to satisfy the weaker posted spec. The identifier contract explicitly gives weaker implementations no cross-implementation exactness promise.

Public registry APIs with a small legal ProducerOnly test adapter and memory blobs. Hash comparison itself is accurate; the unsupported part is treating its mismatch as the exact conformance verdict described by Diverged. No malicious producer, GPU or remote execution is claimed. Repair scope: bounded rerunner-class qualification or clearer verdict taxonomy. Require an appropriately qualified local class before attributing exact divergence, or distinguish an observed byte comparison from a conformance check. Preserve Unavailable for absent adapters and NotCheckable for unsupported verification semantics; do not infer a stronger rerun class from the producer declaration.

Q207: Delayed resource configuration changes exact output without changing either identity. Two legal registrations of DelayedResource with units1 and2 carry identical resource id mesh.delayed/v1, implementation id mesh.delayed.blake3-chain/v1 and ExactBytes class. The identical signed job spec and input produce different outputs. Verifying one with the other returns Diverged attributed to the same implementation id. Unit count is host constructor state and never bound into the job input/spec or either identity.

Actual shipped adapter through public run_job/verify_output; no copied hash-chain implementation. Default registries both use16 units and are not alleged to diverge. Delay-only configuration does not affect bytes; the unit count does. Repair scope: resource/input identity decision, potentially over one hour across rehearsal consumers. Bind semantic unit count into canonical input or a resource generation contract; scheduling delay can remain host policy. A different implementation id alone would not repair the promised ResourceId algorithm equivalence. Keep the rehearsal configurable without claiming one exact algorithm for distinct computations.

Q208: Checkpoint-plus-tail catch-up selection omits predecessors still required by owner admission. A real source store holds post seq0, an owner-built accepted checkpoint covering it, and a later post seq1 linked to seq0. The catch-up selector omits seq0 and selects checkpoint plus tail. An empty receiver accepts the checkpoint and reconstructs its snapshot, but rejects the selected tail as MissingPredecessor. Supplying the omitted seq0 lets that same tail enter. Checkpoint state/frontier does not initialize the shared per-author log continuity anchor.

Actual source/receiver MemoryBackend MeshStores, real checkpoint policy and public selector. No file serialization/import transport or live catch-up campaign executed. The selected set is incompatible with this unchanged authoritative owner admission path; a future authorized prune anchor is a different scenario. Repair scope: retained-frontier/bootstrap contract repair, potentially over one hour. Define an authenticated continuity anchor or require the needed signed boundary/prune operation in catch-up selection. Coordinate domain checkpoint authority with the shared processor instead of weakening MissingPredecessor for arbitrary non-root operations or bypassing MeshStore.

The seventh native observation extends Q202 rather than inventing another directory bug: archive selection gives DeviceAttested checkpoint priority, but catch-up scope omits an attestation represented by the frontier. Its snapshot contains only jobs. The privacy comment saying devices always travel is therefore false for this path. The newly read RemoteSessionClaim owner checks both client and server attestations, so lost directory state affects that admission dependency. The source checks actual poster identity, exact resource, exact active epoch/lease, offered device and authenticated endpoints; it is a real shared transport-neutral authority rather than a private Burn or Iroh identity gate. No new remote session campaign ran.

Real reuse remains visible: run_legacy binds an ephemeral namespace and uses run_job; Echo/Blake3 share preparation; the lexical adapter uses ESP plus one codec source also included by the wasm probe; SyncedMesh uses JoinedSpace and re-exports status; both admitted service actions use Notochord then unchanged MeshStore admission. The worker policy bypass occurs before the common executor, not in a second compute engine. The verifier, by contrast, has weaker local qualification than its exact verdict contract, and the delayed adapter hides semantic unit count outside that contract. Fixing either by inventing a product-private implementation would miss the owner seam.

The optional ingress source has finite active and queue limits, whole-call I/O budgets, frame limits, deadline from carrier arrival, atomic admitted-count reservation, abort/join shutdown and exact canonical original-record decoding. Its inventory/bytes window is explicitly volatile and limited to four records, with no future custody promise. Results are advisory; lost response is explicitly ambiguous and retry uses original identity. Actual code does not equate transport admission with operation acceptance or compute success. The new unit gate reaches this code, but the older example campaign is still unbuildable for Q196, and no fresh saturation/framing/network ingress campaign is claimed.

Larger follow-ups stay documented: raw availability bytes can outlive later MeshStore input-body erasure until TTL/eviction, so privacy/erasure ownership needs a rule and a dedicated carrier repro before any remote-leak claim; checkpoint_held is set by DelayedResource while its hash state is dropped and only counts escape, so current tests prove stopping/reporting rather than a resumable checkpoint; public LexicalVectors encodes unchecked row shapes, though the shipped ESP adapter supplies valid shapes. Live-lapsed owner-stop handling needs host-supervisor review. author_event can return publish failure after durable acceptance and needs a qualified outcome contract, not an inferred rollback.

All three original lease integration tests passed. Their top-level claim that nothing sleeps/races is too broad: one spawns a delayed run, waits 50 ms, then expects cancellation before completion. Different timer/runtime behaviour may change that order; no failure occurred here. Preserve this as a qualified receipt limitation and replace the handoff with a barrier if repaired. Native lexical pin passed inside the library suite; historical wasm exactness evidence was read but not freshly rerun. Default library sync tests actually bind in-process peers and its store test closes/reopens a temporary redb database. Those existing test effects are separate from the seven new CPU/MemoryBackend probes and are not a new full deployment or radio receipt.

Fresh current-main gates: seven additional native observation assertions pass (six new findings and a Q202 extension), all 3 original lease integration tests pass, and all 124 admitted-ingress library tests pass. Green observation assertions mean defects reproduced, not repaired. The current root manifest is fully reread: aliases intentionally share sources; old repository source-key patches redirect immutable consumers; the exact lock/artifact supplier matters more than stale M1 comments or retained inactive donor folders. Required compact logs/probe/coverage evidence and preserved original inputs remain under the default 100 MB retained-byte cap, with no captures or replacement of cited receipts. Probe executable/PDB removed; stable shared target only, no isolated home/target/worktree. See the [complete Mesh review receipt](../../../support/receipts/authority-success-audit/mesh_execution_review_receipt.json).


### Current Pictograph and Pandect refresh, partial Fleece review, 2026-10-17

The previous goal turn was progress: complete Mesh reading and native evidence changed the next action. All twelve source files invalidated by the main merge have now been reread in full; four additional files cover new page-link suggestions/tests and Fleece page-link/crawler adapters. This pass adds 353,818 read characters across 16 files. Current coverage is 1,210/3,355 files and 16,060,205/36,897,906 characters, with no stale coverage hashes or pending chunks. Pictograph is 165/165 files (2,244,039 characters), Pandect 76/76 (1,221,768), Fleece 2/17 (21,209/250,496). Mere objective remains active. Production and original tests/shared WIP are unchanged at 04474731. Resume at crates/fleece/Cargo.toml.

Q209: Derived suggestion creation and attention admission use inconsistent candidate sets and budgets. Two existing offers from an excluded source subtract the entire two-link minting budget, although shared SuggestionSettings::admit admits the two allowed links. Removing those excluded offers restores the links. Conversely, replacing existing offers after two links are minted removes both links from admitted Suggestion attention but leaves both derived graph nodes resident. A60x60 plate mints one link under the pre-mint fill, then immediately gives it no Suggestion attention under the post-mint fill. Minting and attention admission both call budget but use different sets/counts and different plate snapshots.

Current public Canvas API, synthetic graphs and actual shared SuggestionSettings. No rasterized screenshot, live browser or persisted owner write. The observed contract failure is a split admission decision; counts are not evidence of arbitrary unbounded allocation. Repair scope: bounded admission reconciliation; viewport-driven pacing may expand scope. Compute one admitted set after source filtering and use it for both derived membership and attention. Reconcile on every relevant offer/settings/fill change, and define which content counts toward plate occupancy so creation does not invalidate its own admission. Avoid a second private pacing implementation.

Q210: A shared suggested URL retains a page that withdrew it as its origin. Pages A and B both offer the same URL, deduplicated into one derived member. When A replaces its offer with empty, B is the only remaining source. Refresh keeps the existing member by URL, skips its origin tuple and edge update, and leaves its hyperlink from A with none from B. Selecting that member emits KeptLink.origin=A even though only B still offers it.

Actual graph projected relations and public take_kept_links after CPU Canvas frame. No browser commit or wrong durable relation was executed. Stable URL-derived identity is not itself the defect; origin/edge state is stale. Repair scope: bounded derived origin/edge reconciliation. Refresh source provenance and the owned ephemeral relation when the selected offer origin changes, preserving the stable member id. Retain enough relation identity to remove only the derived assertion; do not blanket-delete independent user relations.

Authority entries A402-A416 record a concrete split: one shared SuggestionSettings type still feeds two decisions with different candidates, counts and occupancy. Excluded other-source offers consume minting capacity; a later offer change can leave derived graph members without admitted Suggestion attention; creation itself can alter plate fill enough to revoke its own admission. The correction seam is one admitted set, not a product-private settings fork. The stale-origin case needs source tuple and owned ephemeral edge reconciliation while preserving stable URL identity.

The fifth probe records an ownership question, not a third proven durable bug: a selected link request survives replacement with a different graph that lacks its origin. Keeping link offers across ordinary owner reinstalls is explicitly intended. KeptLink has member, URL and origin but no durable graph identity; the host drains requests into a queue and uses its current create_address intake. A wrong durable host write or lost retry was not executed. Preserve a named ownership/acknowledgement decision before changing graph-switch behaviour; that can exceed a one-hour diversion.

Actual sharing remains visible. Canvas excludes derived IDs from the save geometry across all member-keyed fields; Graphshell intake owns persistence. BoardScene and BackdropSurface share BoardText and Livery rather than a private text engine; host font-role declarations reach roots. BackdropSurface uses the same presentation fit in painting and picking, including stretched template rows and ordinary instrument aspect ratio. This is source review, not a newly executed headed/template/font acceptance campaign. Older palette/body/face geometry findings remain on their original native receipts; the relevant changed source is now reread without pretending the old whole diagnostic suite was rerun.

Pandect now implements arrangement.seed/regions/mosaic alongside the existing family and exports those helpers. Sixteen current helper tests pass. The earlier snapshot describing absent helper shapes is stale; current repository call search still finds definitions/reexports rather than host adoption. Mirrored SavedRegion/ArrangementRegion and SavedMosaic/ArrangementMosaic shapes preserve a graphics-free persistence dependency but create a schema-drift seam to track. Container-specific writer names call the global rewrite_family clear, so a multi-container store would need a scope rule; current tests use one container and no production adoption/data-loss claim is established.

Fleece is partial. The entire crawl.rs and new page_links.rs are read; both choose HTML/XHTML and resolve hrefs locally while sharing the Genet Fleece raw extractor. The new feature dependency is shared, but the crawler does not call the new helper. Candidate filtering is intentionally different: page suggestions deduplicate/drop self links, acquisition delegates admission to Frontier. Duplicate MIME/parse/URL mechanics are a consolidation opportunity, not a second HTML engine. Base-element resolution remains a source-qualified question. The crawler explicitly counts raw success/failure bytes, marks cancelled/timed-out in-flight byte evidence incomplete, and checks cancellation after ready responses; its frontier/robots/sitemap/tests remain unread and no crawl ran.

Fresh checks: Pictograph canvas library build succeeds; two existing page-link tests pass (531 others filtered); sixteen Pandect arrangement tests pass (310 filtered); one Fleece page-link test passes. Five new public Canvas observations pass, meaning the two defects and the qualified pending-intent observation were reproduced. The fixtures use the public graph delta owner; a preliminary audit-fixture compile using private add_node was corrected without product changes. Probe frames compose CPU Scenes, with no GPU rasterization, browser, durable host save, application network fetch or user-file operation. Only compact build/test/probe/source evidence is retained under the default 100 MB cap; previous cited receipts remain unchanged. Probe executable/PDB removed, stable shared target reused, no isolated home/target/worktree. The [refresh receipt](../../../support/receipts/authority-success-audit/pictograph_refresh_review_receipt.json) binds all inputs and scopes.


### Full Fleece review and current upstream transition, 2026-10-17

The previous goal turn made progress and the full-source goal remains active. Fleece is now read in full: 17/17 files,250,496 characters. This checkpoint completes 15 additional Fleece files (229,287 characters) and rereads four merge-changed files (78,578 characters): Mesh lib/resource/worker and Pictograph BackdropSurface. Reading coverage is hash-bound; searches, compiler reach and historical tests do not substitute for rereading.

Fresh origin/main was 36 incoming commits beyond the previous merged basis. Authorized merge 70a15376 incorporates origin/main 8498608c. All 38 local modified/untracked file hashes survived unchanged. The inventory grew from 3,355 to3,722 source files, including the resident inference lane and cubek-matmul patch. Eighteen read hashes became stale; four are now reread, leaving 14 root/Djinn files pending. Current coverage is 1,211/3,722 files and 15,608,211/39,342,110 characters. The lower current percentage reflects newly added source and honest invalidation. Earlier receipts remain historical evidence with their original hashes.

Q211: Sitemap scanning sends XML entity spelling as a page URL and omits CDATA locations. A supplied XML sitemap loc ending search?a=1&amp;b=2 reaches the public run_crawl fetch callback literally with amp;b=2. The intended a=1&b=2 URL is never requested. Equivalent CDATA text produces no search request. Both runs finish FrontierExhausted; the entity case reports two successful page fetches through the synthetic adapter. Frontier URL validation cannot decode XML syntax.

Scope: Native public acquisition driver with a supplied in-memory fetch callback, no real HTTP request or server behavior. The scanner explicitly disclaims a full XML parser, but ordinary XML text representation still changes or drops the declared acquisition target. Repair: bounded XML location decoding. Use a bounded XML text reader or explicit supported location decoder before URL admission. Preserve request/byte/fanout/scope budgets and refusal evidence; do not expand sitemap-index recursion as part of this fix.

Q212: Visual clipping reports success for an empty intersection and includes pixels outside a partial selection. On a 100x100 PNG, an inside rectangle yields the correct 20x20 control. Rect(-50,10,100,20) attaches 100x20 although its visible intersection is 50x20. Rect(200,10,30,20) attaches a 1x20 border strip and returns true despite having no intersection. Clamping origin and length independently substitutes visible pixels for an off-image rectangle.

Scope: Native public helper, synthetic image and caller-supplied ClipRect. Repository call search found the helper definition and its original unit test, not an active product call; no browser capture, user screenshot or end-to-end selection leak executed. Repair: bounded rectangle-intersection correction. Transform both rectangle edges, intersect with image bounds, and return false for an empty/non-finite result before encoding. Keep existing resize limits and test inside, partial, outside and scaled controls.

Q213: Distinct valid canonical annotation targets alias to one exported annotation identity. Two valid rendered DOM records for the same caller-owned acquired representation, selecting characters 0..5 in Alpha and Bravo, have different canonical-text resource IRIs but the same annotation.id. A third record with a different source scope also has that id. All pass validate_integrity and annotation_json_ld; changing capture hash changes the id as a control. Saving/loading the first two through Eidetic yields distinct content-addressed records, so the alias is specifically the exported annotation identity.

Scope: Native caller-supplied DOM and in-memory storage. Actual JSON-LD envelopes share id while their targets differ; no downstream RDF merge, peer overwrite, durable-store corruption or product annotation adoption executed. Repair: potentially over one hour with exported-ID compatibility. Choose whether identity names an immutable annotation or a logical family and state update semantics. If immutable, bind canonical-text resource, source/capture evidence and selector identity. Preserve or migrate existing exported IDs explicitly; a string-format patch alone leaves historical aliases unresolved.

Authority entries A417-A434 distinguish reuse from duplication. Genet Fleece owns raw extraction and canonical-text selectors; Mere adds host-provided URL/capture facts and optional adapters. Generic supplied traversal and bounded HTTP acquisition deliberately use different frontier policies. The two page-link adapters still duplicate MIME/DOM/URL mechanics. The clip module handwrites base64 despite existing crate-based encoding in other consumers. Annotation serialization mirrors selector structs but reuses the canonical projection/resolution owner. Structured-data contribution/import-envelope adapters duplicate iteration while sharing linked-data expansion; detailed outcomes retain errors and best-effort filtering is disclosed. These are concrete sharing seams, not invented claims that every similarly named implementation must merge.

Portable clip export walks only caller-approved selected-part closure, excludes private backing and uses NoFetcher. Retention/import reuse typed sealed Eidetic storage and preserve existing receiver custody. Inputs are checked before writes, while storage failure may leave immutable payloads; atomic graph admission is expressly outside the helper. Capture evidence integrity is not caller authentication. The acquired representation hash, canonical-text resource hash and content-addressed annotation manifest are separate identities; Q213 is the exported annotation ID collapsing valid distinctions between them.

The changed Mesh resource/worker files now expose availability sampling and next_action_with_availability. The snapshot suppresses claim/grant/execute while preserving higher-priority reclaim/heartbeat obligations. Availability is not execution authority. Existing historical Mesh defects and native receipts are not silently promoted to fresh whole-suite reruns. BackdropSurface now exposes forward/inverse interaction geometry and occurrence rectangles using the same fit as paint/pick, with opt-in handles and owner-controlled edits. Its whole file is reread; no new headed rendering gate ran.

Fresh all-feature Fleece native library build passes; 56 library tests, one authenticated peer-transfer test and six portable-clip integration tests pass. The peer test actually connects authenticated local peers and reopens the receiver store; portable tests also reopen disk and exercise sealed-custody preservation with a wiring double, not a new cryptographic proof. Three synthetic public API observations pass, meaning Q211-Q213 were reproduced, not repaired. One current Mesh availability test passes. No browser/user screenshot or product graph write was executed by the audit probe. A preliminary runner ambiguity between the external and local fleece library artifacts was corrected by selecting the local target source path; product code was unchanged.

No production/original tests were edited by this review; upstream changes arrived through the authorized merge. No isolated target, Cargo home or worktree was created. The stable shared target is reused, probe executable/PDB removed, compact evidence remains under 100 MB and cited historical receipts are preserved. Resume at the changed root Cargo.toml, then the remaining thirteen Djinn invalidations, before continuing all unread Mere source. The [Fleece checkpoint receipt](../../../support/receipts/authority-success-audit/fleece_boundary_review_receipt.json) binds source, build, tests, findings and the transition.


### Current Djinn owners and partial resident-mere refresh, 2026-10-17

The previous goal turn was progress: Fleece completed, three native findings and an upstream transition changed the next action. This pass reads 19 whole current files (593,946 characters): thirteen rereads invalidated by the merge and six newly landed inference/lifecycle files. It also rereads 95,789/209,215 characters of resident_mere.rs, through the production body and a partial test prefix. Its 113,426 remaining characters are still unread. Current hash-bound coverage is 1,230/3,722 files and 16,297,946/39,342,110 characters. No stale acknowledged hash or pending chunk remains, which is not the same as completing the final invalidated file. Full Mere review remains active at 70a15376; no additional pull or production/original-test edit occurred.

Q214: Resident hint persistence can overwrite successful pair and unpair changes despite single-writer and atomic-update claims. pairing.rs says the resident never writes settings and that management pair/unpair are the only writers. refresh_dial_hint also loads, changes and saves the whole OwnerSettings file, claiming its atomic path avoids clobbering concurrent pair/unpair. The save and load sequence has no cross-process lock or revision check. A public unpair_device call returns Removed and removes the device; saving an earlier hint snapshot then restores both the paired device and its roster root. A public pair_device call returns Added; saving an earlier hint snapshot then removes that new device and root. Fresh-read controls preserve both management changes.

Scope: Explicitly ordered public API operations reproduce the possible read/write interleaving and final file contents. The fixture follows the resident hint load/record_endpoint/save critical section and uses real pair/unpair APIs. No timed race between resident processes, live roster reconciliation, key re-admission, peer connection or user settings mutation was executed. Repair: bounded shared settings transaction seam. Make all management and hint writers use the same cross-process load-modify-commit owner, or move live management through the resident. Locking only the final rename does not protect an earlier snapshot. Preserve independent concurrent fields and separately address crash-safe replacement; do not claim a rename is an atomic read-modify-write transaction.

Q215: Personal sync host installs its transfer rule by discarding caller-supplied synthetic-address policy. Startup installs the receipt synthetic-address rule, warning that the setter replaces rather than appends. The host subsequently calls with_synthetic_addresses([transfer_offer_rule()]) on that supplied selection, which clears the entire rule vector. An in-memory public replica refuses an unselected synthetic carrier under the supplied custom rule; composing the exact host builder chain admits and projects that carrier. A plain-address control is admitted/projected in both cases.

Scope: Native public SyncSelection and PersonalGraphReplica over memory backends, using the same builder chain as the host. No full host opening or network run. Current desktop startup always selects receipt facets, masking a visible receipt-filter difference there; the caller-policy overwrite is real, but no current receipt disappearance or confidentiality breach is claimed. Synthetic carrier selection is presentation, not confidentiality. Repair: bounded additive policy composition. Preserve caller rules when adding the host-owned transfer rule. Keep an intentional replacement API for callers that replace settings; define duplicate-prefix precedence rather than changing every setter into append. One shared composition seam should own the effective selector list.

Authority entries A435-A452 keep the distinctions concrete. Current dependency declarations use path-owned components while some root comments still narrate external Git siblings. Settings, models and credential roots partly share and partly duplicate profile normalization, with collision scope still qualified. The resident and management commands really are independent settings writers; a shared save function does not make their transactions serial. Startup and host really do use the same selector type but overwrite each other rather than compose the effective rule list. The current desktop always selects receipt facets, so the latter is not evidence of current receipt loss or a confidentiality failure.

There is substantial actual sharing too. Inference borrows the Distillery-held model library; deferred composition waits for the personal graph, and ordered close ends inference before Distillery. Private inference uses the existing group/keyring through a live protection handle, retaining address/domain and writer checks rather than creating another group engine. Discovery binds signed endpoint hints to the carrier, graph and current reader. Automatic settings cap candidate count, tensor frames and time bounds, including the mesh lease ceiling. The Chronicle consumes authoritative folded boards beside lifecycle receipts; it is not an independent fold. CPU/GPU fixtures share resident setup while explicitly acknowledging duplicated synthetic weight construction in another package. No GPU/trainer run is claimed from reading those tests.

Two suspicions were ruled out through source tracing: dynamic MeshHost::register_resource advances facts.gpu when a GPU stage is registered, despite older trainer-only comments; and GraphSession first-edit forks clone their parent validator, preventing the suspected loss of domain validation in the read-only-thaw path. The current resident-mere production prefix also checks live route grants and avoids minting a session for ambient open. That file is still partial, and its original tests are not counted as read or executed. Multi-catalog registration failure and live sync-disable semantics remain qualified candidates, not new proven failures.

Fresh checks: four no-inference unit tests pass, proving the default build refuses configured inference and retains strict model schemas. Seven stage-enabled unit tests pass, covering model/limit/build gates, attestation codec and signed discovery facts. Three public native observations pass: two settings interleavings and one in-memory selector/replica composition. The settings observations use actual pair_device/unpair_device and OwnerSettings APIs with synthetic profile files; no timed resident process race or live cryptographic re-admission was run. The selector observation uses the exact host builder chain with public replicas; no full host or network was opened. Preliminary probe failures came from a Windows CRLF marker assumption and treating correct local NotSelected refusal as setup success; those failed sources/logs have a separate preserved receipt. The corrected probe uses legacy node projection and checks refusal as its control. Product source was untouched.

Stable shared Cargo target reused; no isolated target/home/worktree. Probe executable/PDB and marker-identified generated fixture were removed after checks; compact retained evidence stays below 100 MB and historical cited receipts were not overwritten. Resume at resident_mere.rs character 95,789. The [Djinn owner checkpoint](../../../support/receipts/authority-success-audit/djinn_current_owner_review_receipt.json) binds source, commands, library artifacts, tests, partial coverage and remaining scope.


### Whole resident-route and personal-facts review, 2026-10-17

This checkpoint completes the current resident_mere.rs reread, all three personal-facts domain files, the Djinn personal-facts client and personal-seals adapter, all ten Dramatis app files, both persona-picker files, and the inference-door/automatic-inference fixtures. The 20 whole files contain 347,072 characters; 251,283 characters were added beyond the preceding partial resident-mere checkpoint. Current coverage is 1,250 of 3,722 source files and 16,549,229 of 39,342,110 characters. Every one of the 69 files under ports/djinn/src is now read at its current hash, totalling 1,559,634 characters. This does not complete the Djinn package or Mere audit. No stale acknowledged hash or pending chunk remains.

Q216: the original route lifecycle test retains an obsolete per-family projection assertion. A current offline Cargo run reports 22 passed and one failed: the test expects containment/semantic/semantic on one pair and observes link.surface. The host deliberately emits link.surface/link.resource from the link stratum, and the common route model forwards that disclosed kind. The link-model plan explicitly derives pair links while preserving separately attributed assertions. This is a test-contract integration defect, not evidence that three durable assertions disappeared. The failure aborts that test before its later lifecycle checks. Refresh the oracle around the current link contract and separately verify source assertion preservation, rather than restoring obsolete scene rows or declaring the entire gate green. Production and original tests were left unchanged.

The 22 passing route tests exercise actual memory-carrier admission and local broker paths, live grant revocation on idle notice intake, application-scoped views/material, ambient restrictions, archive/read-only behavior, merge choices and domain validation. Hash-valid wrong-role material and eight media-binding fault classes are refused. These are the exact named local automated checks; no physical multi-device or headed claim follows. An earlier compiled default test binary was also run while the fresh Cargo command waited; it gives the same 22/1 result and has its own artifact hash/source guard receipt. It is not counted as another independent acceptance suite.

Seven fresh personal-facts domain tests pass for revision preservation, exact subject links, replay, persona scope and tamper refusal. The domain crate transforms candidate graphs; the admitted client sends captured edits through the shared resident journal. It opens no second personal-facts file. There is still a duplicated implementation seam: the client separately traverses portable cards and validates scoped graph envelopes instead of consuming the common route decoder. Its absent-session-item path defaults the catalog to an empty graph; malformed versus legitimate empty projection behavior remains a qualified candidate, without a fabricated transport probe.

Dramatis reuses the shared roster-row builder and sends management requests through CustodyClient. Its worker serializes operations and its state drops a failed snapshot. The picker contains no vault writer. The read wire/native/package tests retain explicit ignored fixture gates; none ran here. The packaging helper delegates signed feed generation to Luggage; no install, publication or signing was performed. PersonalGraphSeals borrows current/historical keyring state from the personal-sync host, keeping current compute eligibility distinct from retained-history readability. The automatic-inference fixture is read, including synthetic CPU weights, three independent software roots on one computer, and an ignored GPU case. Reading it is not a new staged execution receipt.

Authority entries A453-A462 record these ownership and verification distinctions. The next unread Djinn fixtures are inference_route.rs (64,543 characters) and inference_gpu_reclaim.rs (11,749). Continue the wider inventory afterwards. Shared WIP and the entire prior document prefix were preserved. Stable shared Cargo output was reused; no isolated target, Cargo home, worktree or standalone generated fixture remains. Compact evidence stays within the 100 MB cap. The [checkpoint receipt](../../../support/receipts/authority-success-audit/resident_routes_personal_facts_review_receipt.json) binds coverage, sources, failed/passing gates and remaining scope.


### Shared inference runtime, journal and stage review, 2026-10-17

This checkpoint reads 15 additional whole current files, 417,348 characters: the two remaining Djinn inference integration fixtures; ESP's inference actor; Distillery's inference lane, durable journal, recall and GPU adapter; the complete stage service and its automatic/discovery/fallback/lane modules; and the durable, layered and stage integration fixtures. Current whole-source coverage is 1,265 of 3,722 files and 16,966,577 of 39,342,110 characters. No stale acknowledged hash or pending read chunk remains. The complete repository objective is still active.

Q217: a disconnected inference actor is not folded into failed run state. A fresh native probe gates an intentional provider-construction panic after a public ask has been admitted and journaled. A later ask returns Closed, demonstrating actual command-channel disconnection, but that rejected ask has already inserted and journaled another queued record. The earlier run remains queued with no load_error. Cancel and pause return success despite failed command delivery; close also returns success and retains queued table/journal state. A second public lane loaded on a user pause returns success from resume and journals a queued continuation after its actor is disconnected. The bridge exits on update-channel closure, while Armillary join discards the actor's panic result. This is a false-success and stranded-state defect across the shared lifecycle seam.

The three native checks include an ordinary returned InvalidConfig error as the control. That actor remains alive, reports load_error and makes the ask Failed. The finding therefore concerns disconnection, not every normal model-load failure. The fixtures use a public in-memory RunJournal and synthetic gated construction panics; no GPU, real model, resident process, broker, network or disk journal ran. GPU/stage generation wrappers already catch some panics. set_locked(true) has an additional source-qualified risk: it waits for queued/running state to disappear while failed Pause delivery cannot change that state. No hanging lock was invoked. Repair should share explicit actor-disconnect handling, reconcile durable transitions when command delivery fails, and expose checked completion without discarding valid checkpoints. This is documented rather than widening the read-only audit into speculative recovery policy.

There is substantial actual sharing here. Inference clones share one actor/table; synchronous checkpoint append is the durable publication barrier. The private journal borrows host-supplied group admission and authenticates domain/run addresses, writer proofs and same-run parents before group insertion. Typed Eidetic evidence is a derived content-addressed projection of those signed logs. Concurrent continuations remain visible errors rather than a silent winner. Recall borrows its source owner, retains exact passage versions/selectors, and uses one set of current-head observations per projection. Absent/stale heads remove current citations without rewriting retained evidence. Publications are owned by a live publisher session and disconnected heads are withdrawn.

Duplicated implementations remain tracked alongside those owners. RecallIndex::build repeats the same four nonzero-limit checks as RecallConfig::validate instead of calling the shared validator; current behavior agrees. LibraryStages and StageFallbackProvider separately adapt checkpoint text prefixes into streamed fragments. The layered-model BF16 fixture assigns identical result token IDs to baseline and formatted output and writes literal zero logit errors. That is self-comparison setup for runtime selection, not independently measured format quality. Its non-journaled comment is stale because the lane now receives Some(journal). These qualifications do not fabricate a numerical failure or turn every repeated adapter into a second durable authority.

Stage admission reuses current mesh leases, signed inputs and key-group readers before/after input and before output. Detached blocking work retains its runtime/busy/GPU guards through destruction. Automatic discovery and reservations carry public model metadata; only the authority's serialized drive_posts writes mesh jobs. Fallback checks the exact model/session/capability and resumes from committed token/draw state after retiring the staged provider. The reviewed integration sources contain stronger local transport, process-kill/reopen and stage checks, with separate GPU/headed gates, but those checks are not new runtime proof in this checkpoint. External fixture output freshness remains qualified: headed control/result files and merged GPU receipts need their owning driver's fresh-directory evidence.

Fresh verification: the inference-model library build passes and the three native crash observations/control pass. The first broader Djinn inference-route Cargo attempt exited 101 during compilation with memory-allocation failures, before tests ran. Its failed log is retained. A single-job retry uses the same approved shared target and remains tracked by exec session 4064 and a captured live process snapshot; poll that existing handle/process before any restart. Its result is not assumed. The probe executable/PDB and verified generated import/export linker files were removed; no isolated target, Cargo home or worktree was created. Shared WIP and the entire prior document prefix were preserved. Authority entries A463-A476 and the [runtime checkpoint](../../../support/receipts/authority-success-audit/inference_runtime_review_receipt.json) bind the exact sources, compact evidence and remaining scope. Continue at Distillery authority.rs.


### Supervisor custody and receipt truth, 2026-10-17

This pass reads 15 additional whole current files, 207,121 characters: Distillery authority, the complete mesh-host module tree, GPU ownership gate, Chronicle observer/recipe, installed authority/settings/surface, Burn Remote service, library/lifecycle modules and the authority integration fixture. Coverage is now 1,280 of 3,722 files and 17,173,698 of 39,342,110 characters. No stale acknowledged hash or pending read chunk remains. The full Mere review remains active.

Q218: successful local-input delivery does not acquire the consuming mesh's custody claim. Two mesh-scoped spaces share one collecting transport BlobStore. A stores the input; B's deliver_inputs reads it and skips the courier, so B never pins it. A's release then lets actual GC remove B's delivered input; B has zero tags to release. The positive control explicitly stages/pins the same bytes through B and keeps them across collection intervals after A releases. The one-physical-store claim remains true. The bug is treating shared local presence as ownership of its lifetime. Repair the local-hit custody adoption using the existing scoped owner, rather than inventing another blob store. This probe uses public delivery/custody/GC APIs with synthetic memory-backed data; it does not post that delivery fixture as a job or run a full checkpoint/concurrent-maintenance interleaving.

Q219: the supervisor can emit Completed for a result its authoritative board rejects. A tiny public leased BLAKE3 job finishes between ticks. Advancing ManualClock beyond the lease window before the next tick causes reap to author the late completion and emit Step::Completed before abandon_lost_leases runs. The folded job stays nonterminal (Claimed), and that same tick claims it again. The live-window control makes both receipt and board terminal. The pure fold is correct; the product receipt is false. Recheck current lease ownership before publishing a finished-run completion or reclaim fact, using the existing still_held/lost-lease seam. Do not change the fold to accept late output merely to agree with the receipt. The fixture uses one local P2panda/SyncedMesh endpoint, synthetic memory bytes and a manual clock; no physical peer race or late output accepted by the fold is claimed.

The previously pending single-job Djinn inference-route retry is terminal success: all four original tests passed, including actual app-door library inference and independently rooted keyed personal-graph continuation on one computer. The first compile attempt's allocation failure remains preserved. Its process handle 4064 is finished and needs no further polling. Four new native supervisor/custody observations and positive controls also pass against the exact successful predecessor library artifacts, whose hashes and source basis are retained. The first audit fixture failed to compile because builder_for requires DerivedKeypair while it supplied Ed25519Keypair; the exact failed probe/runner/log are preserved in a preliminary receipt. Correcting the fixture's public key wrapper changed no production/original-test source.

Authority entries A477-A491 distinguish actual sharing from overclaims. Distillery returns the MeshHost fold instead of owning a second scheduler. Run progress and running count belong to the supervisor; actual GPU occupancy belongs to the shared local gate. Ordered product shutdown uses the shared close_all helper, but this cannot make a swallowed actor panic truthful. Chronicle endpoints share one observer and retain at most one same-topology contiguous diff. Their generation is explicitly owner-supplied and not a checkpoint hash. The authored Chronicle recipe shares reading/encoding/arrangement across named source bindings, with era bands as a variant. Installed settings use Pandect's recoverable replacement; the read-only Cambium surface projects supplied bootstrap/resident facts instead of reopening a vault or folding another board.

InstalledAuthority receives an IdentityProvider from its caller. Its stored profile label does not independently verify which provider was supplied; existing tests change both label and fixture seed, so they cannot be cited as a proof that the persisted label itself selected those keys. The caller remains responsible for the binding. Old overview prose also lags implementation: H1 delivery exists in courier.rs, views/Chronicle and feature-gated trainer modules are exposed, and the supervisor is a current Distillery module despite older separate-crate language. These are concrete narrative and evidence limits, not claims that all declared feature gates have runtime receipts.

Larger effects remain documented rather than expanded speculatively. Burn Remote cleans active admission on cooperative cancellation/session end but has no aborted-future guard comparable to StageService's RunRegistration; whether transport teardown compensates is untested. Generic host shutdown explicitly waits for detached work only through a composed GPU gate, so CPU spawn_blocking drain needs the remaining trainer/resource review. Retention candidates are sampled before an awaited custody release, leaving a possible renewed-reference handoff to inspect; no intervening post/re-pin was injected. The existing replay-tail protection is real at its sampling point. These candidates do not yet prove remote authority bypass, surviving CPU work or a full maintenance race.

Shared WIP and the prior canonical document prefix were preserved. No production or original tests were edited. Probe executable, PDB and verified import/export linker outputs were removed; no isolated target, Cargo home or worktree remains. Shared Cargo outputs remain owned by their ordinary builds. Compact audit evidence remains within 100 MB. Resume at trainer.rs, then flora.rs and remaining tests/consumers. The [supervisor/custody checkpoint](../../../support/receipts/authority-success-audit/supervisor_custody_review_receipt.json) binds sources, native observations, the completed original gate and these qualifications.


### Trainer, typed identity and retirement audit, 2026-10-17

This pass reads or rereads 27 whole files (364,819 characters), including the complete remaining Distillery trainer/FLoRA source, its remaining integration fixtures, the shared ESP LoRA loader, Athanor, four Eidetic identity/tombstone modules, the current personal-facts domain, and native/Python model controls. Current coverage is 1,297 of 3,723 files and 16,929,095 of 39,370,097 characters. Seven previously read source files changed with upstream main and are excluded until reread; the lower current count of characters reflects honest invalidation, not lost historical receipts. The full Mere source review remains active.

Upstream main f8bbd092 was fetched and integrated at a6e6b94140b0b227f2ac19356571e3decfa98975. All 34 prior dirty files were preserved byte for byte, with no overlap against the 26 upstream paths. Earlier source/verification receipts retain their own commit and hashes. The successful trainer library build and its native probes use the exact premerge artifact set; reviewed trainer/Eidetic runtime sources remain unchanged through the merge. The new personal-facts domain manifest, library and original tests were reread in full. New profile ids are checked through a caller-supplied resident lookup, once per new id; historical references remain independent of current availability. The remaining changed Pandect/Djinn owners still need a current full reread.

Q220: successful host shutdown does not necessarily mean its real CPU trainer worker has stopped. A native public MeshHost runs the actual TrainerResource while the probe gates its shared artifact-store mutex. The blocking closure holds a third store Arc after shutdown returns Ok. Releasing the audit gate makes the absent synthetic model end that worker before tensor training. The drained-worker control retains only the two expected owners. Both checks pass and all audit-owned work drains. This is CPU lifetime evidence, not a model-training, GPU, installed-resident or physical peer receipt. A shared drain contract must cover the real blocking work; cancelling its awaiting async task is insufficient. The previous source-qualified CPU candidate is now reproduced.

Q221: one payload hash currently identifies both blob bytes and schema-bearing mutable manifest metadata. Distinct TypedPayload schemas with identical serialized JSON receive the same successful id; the second save overwrites the schema and makes the first typed read fail. A separate explicitly requested save changes privacy and provenance under that same id. The different-byte control preserves independent schema reads. All three native checks pass. No unauthorized privacy transition, publication or real-data exposure is claimed. The problem is the blanket immutable-manifest/no-op claim and missing schema-sensitive identity/admission. Keep physical blob deduplication while deciding artifact identity and metadata history; this is a broad compatibility/migration issue to document rather than a quick architecture rewrite. Athanor really calls the shared codicil compose path, but stable payload identity does not prove immutable provenance or a write-free reapplication.

Q222: age-based retirement reduces eligible tombstones to node ids and then calls the deliberate all-history per-node purge. An expired and an unexpired record for the same node both disappear, while a different-node control retains the fresh record. Two native checks compile the exact Athanor production module against retained Cargo-built dependencies and use actual Eidetic record/list APIs. This is not a Cargo package/resident/UI/disk/network gate. Preserve exact accepted tombstone identity or cutoff through selective retirement; leave the explicit permanent per-node deletion command's broader contract intact.

Authority entries A492-A503 also record real sharing and narrower duplication. Both trainer arms call ESP and share one explicit request enum, without feature-disabled fallback. FLoRA duplicates a private subset of the PEFT loader's config decoding and checks, while the loader identity is shared; its strict transform validation and actual resolved-model dimension validation have different jobs. Consolidate a deliberate shared subset without treating every stricter check as a bug. Training fixtures repeat tiny model/tokenizer builders but do not contain independent training engines. Gemot governs round membership/weights, while Distillery transforms supplied artifacts. Signed social fixture operations accepted into two local stores do not demonstrate a multi-process network or actual training. Rust native numerical controls reuse ESP; Python references supply independent Transformers/PyTorch calculations. Reading either is not executing parity. The walk fixture uses authored boards/maintenance receipts and a headless DOM, while host delivery/reclaim fixtures really use two local endpoints.

Image reference forgetting checks role/digest and shared image GC rechecks a supplied snapshot. URL-based cache forgetting may need an equivalent current-reference or accepted-content guard, but graph uniqueness and host acceptance contracts must be traced first; no such deletion race was injected. This remains a candidate. No production or original tests were edited by the audit. No isolated target, Cargo home or worktree was created. All three native probes' executables/PDB/import artifacts were removed after their bounded observations.

The postmerge original FLoRA unit gate first exited101 before compilation/tests because the new Murm dependency revision was unavailable offline. Its network-enabled retry is session32507, still fetching newly pinned dependencies; poll that same handle and record its terminal result before starting another retry. Do not cite the live log as a completed test receipt. The [trainer/identity checkpoint](../../../support/receipts/authority-success-audit/trainer_identity_review_receipt.json) binds coverage, the merge guard, seven native observations/controls, and the remaining verification boundary.


### Owner admission refresh and probe evidence boundaries, 2026-10-17

Current coverage is 1,328 of 3,723 files and 17,693,773 of 39,370,097 current characters. This pass emits27 whole files400446 characters, including complete current Pandect session/transaction/library sources and24 new standalone probe, launcher and fixture sources. All seven upstream-invalidated source paths now have current coverage. The full Mere review remains incomplete; five primary Distillery browser Rust/web files remain unread, and other product areas are still outstanding.

Four changed sources were refreshed through verified retained coverage rather than claiming they were emitted whole again: Cargo.toml and Djinn's daemon, embedded reservoir and resident_mere. Each old file had already been fully acknowledged. Its exact Git/checkout bytes were reconstructed at the prior hash,351600 current characters were proved identical to previously read spans, and276 changed lines were emitted with deleted lines and context, inspected and separately acknowledged. An independent verifier applies every emitted hunk against both full sources, proves every current character has exactly one coverage source, and confirms an unread-old-source attempt refuses without modifying coverage. The original linear reader is unchanged. The four immutable transition certificates and review_transition_verification_receipt.json make this coverage method explicit; it is not grep, compiler reach or an inferred whole-file read.

The admission seam is shared. Structural GraphValidator and new-reference GraphAdmissionValidator both check edit candidates. Detached transaction copies and whole-session forks retain both callbacks. Historical replay uses structural validation; current owner lookup is needed for newly introduced profile ids. Both daemon and embedded compositions supply PersonaeHost to MereRoutes, and an absent lookup refuses new profile subjects. The forwarding adapter method was inspected but is not counted as a whole receiving-host source read. Original upstream refusal tests inspect graph/history/revision/storage; they were read, not newly executed. Upstream also corrects Q216's stale projection assertion: one painted link.surface is expected, and three retained source relation kinds are separately checked, including after fork. This is a current source correction; its historical failed gate is not erased or converted into a fresh green runtime receipt.

Q223: fetch-model-matrix.ps1's purported safe model-directory guard accepts /models/.. and resolves it to the repository root, while /models/. aliases models itself. The ordinary child control stays strictly below models. The exact three production AST statements were evaluated natively with synthetic paths. No curl, download, directory creation, move or overwrite ran. This is malformed caller-supplied matrix input, not a claim of an externally reachable attack or existing matrix damage. Reject dot components and check the normalized absolute destination before creating or replacing files.

Q224: the remote fixture captures first/recovery inference elapsed time after report_stage has slept for the configured telemetry hold. The driver wrapper defaults that hold to3000ms, so both reported remote_execution values include the deliberate hold. This follows directly from the complete fixture/wrapper source and does not require a new GPU benchmark. The default direct fixture has zero hold. Capture elapsed time before emitting/holding the completed stage. This timing defect does not invalidate allocator/driver release gates by itself.

Q225: two maximum-error helpers have drifted. The session fixture folds differences with f32::max and silently discards NaN coordinates, including infinity minus itself. The remote helper refuses non-finite operands. Exact verbatim extracted helpers compiled natively reproduce the divergence; finite and unequal-length controls pass. The full session receipt separately checks adapted and independent logits for finiteness, so a complete model false-pass is not proved. Its base-logit and published-checkpoint comparisons reuse the weaker helper. Share a strict numeric-validation contract while retaining the independent CPU PEFT merge arithmetic as a useful separate numerical control. No actual model, GPU or browser was run.

Authority entries A504-A515 distinguish real shared service composition from copied evidence plumbing. The remote fixture mounts actual Distillery/RemoteSessionService and uses two local endpoints, manual time and explicit NoCustody. Active CubeCL allocator release is separated from physical driver measurement: the Rust receipt records physical measurement false, and the wrapper adds process counters/NVIDIA PID attribution and rejects missing active samples. The LoRA session fixture actually hashes its loaded bytes, round-trips through Eidetic and calls the shared ESP adapter loader, comparing a separate CPU merge. In contrast, remote MiniLM receipt fields use fixed expected model/revision/hash/size labels for a caller-supplied directory; the fetch/loader identity guarantee needs further tracing before claiming a successful wrong-artifact GPU receipt.

The two standalone browser repros copy page-error gates, worker timeout/error-scope wrappers and launcher policy. Three reviewed wasm cfg files are byte-identical at this snapshot. A self-contained minimal reproducer is an intentional boundary; copied receipt validation can still drift independently. Window error handling is not a worker-global unhandled-error/device-lost gate, but no browser failure injection was performed, so that stays a candidate. The primary probe runner also omits --locked while assuming selected package identity is unchanged, unlike the two locked repro runners; there is no selected-graph before/after proof in that script. No actual dependency drift is asserted from that source fact alone.

No production or original test source was edited. Shared WIP and the prior canonical document prefix remain byte-preserved. The numeric diagnostic's native executable/PDB/import outputs were removed, and the directory diagnostic created no model files. No isolated target, Cargo home or worktree was created. The existing original FLoRA gate is still live at session32507, now fetching Netrender/Cleromancy after Murm/Genet; it has no terminal result or executed test count yet. Continue polling that same handle before retrying. The [owner/probe checkpoint](../../../support/receipts/authority-success-audit/admission_probe_review_receipt.json) records the six new native checks, source-only timing defect, coverage proofs and remaining verification.


### Primary browser probe and Eidetic authority review, 2026-10-17

This pass reads23 additional whole files254394 characters. Coverage is 1,351 of 3,723 files and 17,948,167 of 39,370,097 current characters, without stale acknowledged hashes or pending chunks. All inventoried source beneath ports/distillery is now read. That is not the whole Mere objective: Eidetic sealing/retention/browsing/import and other product source remain outstanding. A separate path inventory found none of the additional common code-extension candidates checked; this does not replace the source denominator or add scan results as reading coverage.

Q226-Q228 are reproduced by evaluating the complete actual browser page JavaScript in a native Node VM with controlled DOM/fetch/Worker messages. Manual termination returns true and kills the synthetic worker but never settles the pending UI submission, leaving controls disabled. The worker-error control settles and reenables them. Embedding row success ignores false within-worker and across-worker repeatability fields, whereas decoder success explicitly includes them; an integrity-failure control rejects and an ordinary row passes. Finally, a controlled worker report with1 observed weight byte can produce a successful matrix conclusion advertising90000000 configured bytes as its largest successful model. No real browser, worker, model, GPU or network was exercised, and a one-byte real model was not loaded. These are actual consumer-orchestration/predicate contradictions under controlled worker contracts. ProbeConfig does not carry the external artifact sizes/SHA256 pins into acquisition. Cold-to-warm content equality is real stored identity, not evidence of agreement with the advertised model pin.

Q229: the v1 training corpus reader converts legacy fields without the current validator. Public native loading accepts empty, overlapping, duplicated and reverse-ordered partitions that current v2 serialization rejects. Valid v1/v2 controls agree. The real Distillery trainer independently calls corpus.validate before loading cases, so this does not claim an actual training/evaluation leak. Apply the same domain validation after legacy conversion rather than maintaining a weaker compatibility lane.

Q230: successful durable URL-cache replacement with content_type=None leaves the preceding response's MIME key intact. Native public save/load returns the new bytes plus old text/html; fresh-None and explicit-type replacement controls work. This is distinct from Q192's PageLoad request-observation defect. Replace body/MIME through the existing shared batch seam and delete absent replacement metadata. No live fetch, rendering or installed cache was touched.

Q231: pack verification returns on the first personae binding. A valid-plus-malformed signature multiset is Trusted or Broken solely by order, contrary to the documented existential verification description. Single valid/malformed/unsigned and tampered-manifest controls still behave correctly. All signing is with a synthetic fixture key. This is a verdict-contract problem, not a signature forgery, grant mint or installation-review bypass. Document the mixed-signature policy before changing it; the audit does not select whether one valid binding suffices or any invalid claim rejects the envelope.

Q232: actual manifest resolution can write valid canonical cache bytes yet fail its next offline resolve. This occurs for a remote-only source list or a noncanonical declared Local key, because the resolver never consults the canonical key it just populated. A canonical Local source makes the warm read succeed without another fetch, and corrupt declared cache bytes still fail integrity. The original cache test establishes key existence, not cache reuse. The native diagnostic uses a synthetic in-process fetcher, not a real network. Normal save_typed manifests already declare the canonical Local key and remain a working control.

Q233: built-in portable schema declarations have diverged from the shared schema validator and typed bindings. Exact canonical schema bytes are checked against runtime public references. A domain-valid ModelAdapterManifest is rejected because its declared mere-native body contains unsupported number/integer and nullable-union field forms; the observed first error is unknown variant number. OpaqueBlob produces raw bytes while its definition requires a JSON object carrying a bytes-string field, so its actual payload also fails. Current TrainingCorpus validates against its own definition. Meta bootstrap only seeds the meta-schema; a missing adapter definition still takes the explicitly documented tolerant skip path and can return Ok without proving validity. This is a schema/encoding compatibility repair with retained identities, not a safe in-place canonical-constant rewrite. No failed real model load or installed migration is claimed: ESP's local adapter domain checks are a separate current path.

Authority entries A516-A532 distinguish actual sharing from consolidation prose. The primary browser worker really uses IndexedDB/Muniment, Eidetic ModelLibrary and ESP providers. Its finished state means the report exists; the page owns qualification and must consume its facts correctly. Structural copy ladders and page adapter/memory samples explicitly disclaim peak GPU-memory and worker-hardware identity. Warm execute explicitly requires expected hashes despite the earlier shallow config check; a vacuous missing-hash success was ruled out.

Eidetic Store is actually a Muniment Backend reexport, and old Engram vocabulary is a compatibility alias rather than another engine. Conversely, the generic Bundle inventory remains unadopted by the reviewed model and pack paths. A crates/ports Rust name scan finds its implementation, self-tests and reexports plus unrelated Bundle names, with no other matching Eidetic consumer. ModelLibrary resolves its own config/weights/tokenizer shape, and PackManifest has another parts inventory while sharing the same leaf store. This is an in-repo consolidation gap; external published consumers were not surveyed, so removal is not justified. Codicil and BlobManifest wrap payload-hash identity with classification metadata; their higher-level temporal/authorization claims cannot be inferred from the mutable floor or bare content hash alone. Raw image loads are another namespace without the typed resolver's hash check; the consuming corruption/decoding contract remains a candidate to trace.

The central schema dispatcher has three code-defined formats, not a demonstrated runtime registry. Optional JSON Schema validation refuses when unavailable. JSON-LD now checks nominal types and required non-null properties while correctly stopping short of SHACL/vocabulary validation; its older overview still calls it parse-only. Unknown-schema tolerance is explicit and tested, so its Ok result must not be reported as conformance. Private runtime validators can succeed while portable schema definitions fail, which is the concrete shared-authority drift in Q233.

All6 Node checks and16 native Rust tests passed. The Rust diagnostics use exact retained premerge Cargo-built libraries with unchanged reviewed source; their scope is recorded per receipt, and the current postmerge gate remains separate. All diagnostic executables/PDB/import outputs were removed. Shared WIP and the canonical document prefix remain byte-preserved. No production/original-test edits or isolated target, Cargo home or worktree were created. The postmerge original FLoRA gate is terminal: exit0, 6 passed and 0 failed. Its exact scope is recorded in flora_gate_terminal.json. The [browser/Eidetic checkpoint](../../../support/receipts/authority-success-audit/browser_eidetic_review_receipt.json) binds these findings, source coverage, duplicate-implementation map and verification limits.


### Browsing/import failures and current-main review, 2026-10-17

Fetched and merged two newer main commits through4c4ea11d into local main c1b5d47156f2c940ccd67799963ed203df96362a. All34 pre-existing modified files were byte-preserved at integration. Three previously read source files changed upstream; retained-coverage certificates prove identical spans and bind the explicitly emitted/inspected changed hunks. Two new span-edit files and the complete Graphshell backdrop consumer were read normally. Earlier receipts and the successful original FLoRA gate retain their earlier exact basis; the merge does not turn them into tests of the newest span code.

Coverage is 1,381/3,725 current files and 18,239,495/39,398,069 characters, with no pending chunks or stale inventory hashes. This pass adds30 whole files288694 characters. All inventoried source under crates/eidetic is now read; the full Mere objective remains incomplete.

Q234: flush removes an owner's open events before save_trace succeeds. An injected first-write refusal returns an error and writes nothing, but consumes the only pending event copy. Retry reports no work and reopening is empty. The successful-flush control retains it in memory and the store.

Q235: forget_url takes the complete loaded corpus before deleting any target manifest. A refused middle-target deletion discards unrelated loaded traces as well. All stored manifests remain; retry returns Ok(0) because memory is empty, and reopen resurrects all three. The happy-path control removes only the target. Preserve event/trace ownership until each required durable transition succeeds and make partial progress retryable. These are actual native public APIs with synthetic MemoryBackend/fault injection, not installed history loss or a real disk failure.

Q236: recent_corridor and the chronological window read concatenate sorted trace segments and owner-keyed open events. Two individually chronological traces [10,30] and[20] produce[20,10,30], and recent(2) selects[10,30]. Two open owners named a and z at20 and10 select the older10 as recent(1). A shared event-time view is required; chronological adjacency must still not invent source navigation parentage.

Q237: current page-text reads and exact-version reads disagree on integrity. Overwriting a synthetic blob key with different valid UTF-8 makes text_for/load_all return those bytes successfully, while text_at on the same reference rejects a digest mismatch. An uncorrupted control agrees. Reuse verified decoding rather than maintain a weaker current-content path. Missing-body tolerance and encoding policy are separate from accepting mismatched content under an asserted hash.

Q238: a whitespace-named Chrome child folder is not pushed, but its raw name presence still pops the outer folder. The following sibling bookmark therefore parses at the root. Nonempty and omitted-name controls preserve its parent. Restore the path to entry depth or track whether a component was pushed. Intake source bytes survive, but source preservation does not make a wrong typed hierarchy correct.

Q239: i64::MIN is accepted as a timestamp integer, then subtracting the Chrome epoch offset panics in the retained debug Cargo-built public parser. catch_unwind catches it and the normal epoch-offset control maps to Unix zero. Release overflow behavior was not executed; a release panic is not claimed. Use checked conversion under an explicit invalid-optional-timestamp policy.

The exact native diagnostic receipts contain8 browsing checks and5 importer checks, all passed. They use exact retained Cargo-built libraries with the relevant source unchanged through both authorized merges. Their synthetic data never touches a user store/profile or live network/browser. Native diagnostic executables, PDBs and import artifacts were removed. No production or original-test code was edited and no isolated target, Cargo home or worktree was created.

Authority entries A533-A549 distinguish sharing, admission and proof. Sealing intentionally leaves low-level host policy optional, while retention requires a private sealer and real transactional quota admission. SchemaFacetValidator actually calls the common validator on preloaded schemas, with documented unknown-facet tolerance. The canonical BrowsingTrace schema also uses the unsupported integer vocabulary from Q233; this is a source-qualified extension, not a second executed schema test. ResourceCaptureRef validates resource/key identity but cannot alone prove retained bytes. Page identity reads current supplier text, not per-visit historic content. Raw encounter and typed history transition mappings remain distinct; receiving normalization is the next trace, not a presently asserted ranking bug. Restored encounter-tree validation is a partial check that needs its caller invariants examined before stronger integrity claims.

Hagiograph's max/holder-union record and prebatch feat judgement are separate from journal storage and a product simulation. Its mirrored Mesocosm serialization fixture is not live external adoption proof. Deep time has a tick ceiling; extreme or retrograde supplier counters remain a source candidate, not an executed product failure. The new Scenomise span-edit seam checks source, revision, occurrence and prior span; Graphshell uses it to author a local recipe copy and validates the receiving document. There is actual adoption here. Browser pointer cancellation normalization and the literal47 gesture/compile spacing coupling remain candidates, without a headed failure claim.

The [browsing/import checkpoint](../../../support/receipts/authority-success-audit/browsing_import_review_receipt.json) binds findings, current source hashes, integration, coverage and verification limits. Next source is the complete Graphshell capture integration. The goal remains active.


### Graphshell capture and extension authority review, 2026-10-17

This pass reads37 whole current files418956 characters at c1b5d47156f2c940ccd67799963ed203df96362a. Coverage is 1,418/3,727 files and 18,658,451/39,399,652 characters, without stale inventory hashes or pending chunks. All inventoried extension source is now read. Chromium/Firefox manifests were missing from the JSON-data filter even though they control executable entry points, identity and permissions. They are now explicitly inventoried and read; this enlarges the denominator by2, without importing scan output as reading credit. The original reader stays unchanged. Use refresh_inventory.py for future refreshes. The full Mere objective remains incomplete.

Q240-Q242 are source-confirmed, with the prepared full public capture diagnostic still unexecuted. Q240: capture snapshots/restores browsing memory and discards its local ledger/write buffer on error, but immediate MereHost graph edits remain pending. Later stage/persist includes those edits without restoring their matching encounter/access/trace writes. Existing tests redeliver the identical event, masking the different-next-delivery case prepared here. The shared resident transaction path already installs its graph candidate after storage acknowledgement; capture implements another lifecycle around it. A coherent multi-artifact candidate/commit boundary is a larger repair to document, not a production patch during this audit.

Q241: HistoryOnly forgetting retracts all incident Traversal-family assertions without capture source/author/handle selection. Preserving the addressed object does not preserve other applications' traversal evidence. The prepared native test adds an other-application incident traversal and an unrelated pair. Q242: configured segment_size reaches BrowsingMemory but ingest_batch ignores its flush-threshold result and flushes once after the delivery. This changes trace/quota granularity while leaving a visible setting ineffective. Native size1-versus64 comparison is prepared, not executed.

Q243 is reproduced by executing complete actual background.js and capture-model.js in native Node VM with controlled extension storage. Queue writes and acks use serializedQueueMutation, but configuration bypasses it. An awaited stale configuration snapshot deletes a newly queued visit. A delayed queue write can instead restore a URL after completed configuration excluded that origin. Sequential configuration is the working control. Permission-removal writes are also outside the shared serialization owner; no actual revoked-permission read was demonstrated.

Q244 is paired actual JavaScript/Rust evidence. The queue silently retains prior source facts when original_unix_micros or source_is_local changes under the same scoped visit identity/millisecond stamp. The real BrowserEncounterLedger rejects both as conflicting source facts. Exact duplicates and a URL-conflict refusal are controls. Capture's raw transition mapping difference was separately traced and qualified: NormalizedVisit::trace explicitly lowers subframes/downloads to zero-weight Restore, so a current Graphshell ranking bug is not inferred from the raw mapping alone.

Q245 is another copied implementation with weaker success gates. With exact common BLAKE3 fixtures, JavaScript ResourceAssembly accepts changed bytes under an unchanged chunk address and a validly chunk-addressed altered payload under the wrong whole-resource address. The actual shared Chirograph assembler rejects the same frames as CorruptChunk and WrongResource. Valid-frame and wrong-session controls pass. JavaScript validateHash only checks array shape. Adopt shared verified assembly rather than treating matching metadata as verified bytes. These are synthetic native tests, not a real native host, browser, transport or rendered-resource attack.

Q246 is source-confirmed package drift: POSIX prepare-extension.sh omits action-form.js even though bridge.html loads it. PowerShell copies it. bridge.js skips bounded forms when GraphshellActionForm is absent, including ordinary transfer acceptance forms. A stale destination can hide the missing file. The package source receipt compares explicit dependencies/copy lists; no shell installer, registry, installed extension or headed browser was executed.

Ten Node VM checks and6 public Rust owner tests passed. Native diagnostic executable/PDB/import outputs were removed. The first attempt to reuse the retained full Graphshell library failed to resolve its dependency graph and ran zero tests; its runner and compiler failure were preserved. A fresh current graphshell --lib --no-default-features --features web build is live in original session87735 and its artifact log is growing. Poll that same handle, preserve its terminal result, then select its exact current artifacts for the prepared6 public capture tests. Do not treat this observation timeout as terminal, restart the build, or relabel Q240-Q242 as native reproductions. The live Cargo processes were independently matched to the exact build command. No separate target, Cargo home or worktree was created.

Authority entries A550-A569 record actual sharing and its limits. Capture's ledger is graph-root scoped while browsing/access reads span the supplied backend; persona/device filters are separate. Graph observation facets are derived caches only when the caller also saves the matching Eidetic record; the base IntentSink graph-only access path still needs its effect/persistence consumer traced. The sealing API uses a real wallet in tests, while browser capture calls the unsealed alias, so crypto API qualification is not proof of product key wiring. AccessRecord's own native schema builder and omitted optional fields avoid the vocabulary/nullable-encoding drift in Q233.

Resident observed commands and coupling strength edits genuinely reuse shared GraphSession transactions and preserve their owner/session identity. The resident publication flag is a caller assertion at a trusted composition boundary, not itself authentication or an exclusive registry. A partial presentation accessor exists while portable snapshot construction reads the complete writer; adoption needs consumer evidence. Native identity UI controls and result correlation are not native custody authority. The native messaging smoke checks only its first selected card for PRIVATE KEY text, and reports session_bound without asserting it, so it is not universal private-material or session-binding proof.

Host history evaluation is shared and current checkpoint views are freshly compiled from the selected envelope. Public history_view's revision-string precondition remains narrower than proving arbitrary foreign source identity. Cambium PointerButton has no None variant and its Move phase is a captured drag, so raw browser hover/None-button reasoning does not establish a bug at that typed API. A browser/host adapter trial was not run. All909 protected prior production/test hashes and the canonical document prefix remain preserved. No production/original-test code was changed.

The [capture/extension checkpoint](../../../support/receipts/authority-success-audit/capture_extension_review_receipt.json) binds the new coverage, source-qualified and reproduced findings, implementation map, explicit pending build and diagnostic limits. The next whole source is the remaining host fixture composition; the full goal stays active.


### Product, transfer and presentation authority review, 2026-10-17

This pass completely reads33 more source/test files714200 characters at c1b5d47156f2c940ccd67799963ed203df96362a. Coverage is 1,451/3,727 files and 19,372,651/39,399,652 characters, without stale inventory hashes or pending chunks. Product/fixture composition, local edit/intake/scene owners, host datasets/presenters, transfer/source offers, rule workspace/runtime/binding and access/link/field inspection are included. The full Mere audit remains incomplete. The next whole file is personal_sync.rs; remaining Graphshell and every other unread repository area still count.

The original library build in session87735 is terminal exit0 with build-finished success=true. All6 prepared public capture tests now run against its fresh artifacts and pass: ordinary consistent capture, identical retry control, different-next-delivery after refusal, refused forget followed by another capture, segment sizes1 versus64 and other-application traversal ownership. Q240-Q242 are therefore native reproduced, not merely source-confirmed. A refused delivery's graph edits can become durable without matching ledger/trace/access authority; refused forget similarly diverges. The old failed retained-artifact attempt and its runner remain preserved. This is synthetic fault-backend evidence, not a real browser/profile, physical disk-failure or concurrent-writer receipt.

Q247: include_local_file_locations=false removes LOCAL_FILE_FACET only. An actual public export of the synthetic address file:///C:/synthetic-audit/private-folder/example.txt retains that location verbatim. The UI names the setting Include local file locations, and prepare_transfer says local locations are always removed. Picked files using ni: addresses are a working narrower path. This counterexample uses a synthetic address and no physical file. Repair must define the exclusion over graph identity/reference carriers, not blindly redact strings and silently change Resource identity.

Q248: receive-side content verification is weaker than preparation. A valid selection names b"abc" by SHA-256/ni: identity. Substituting only the manifest blob descriptor with a valid BLAKE3 address for same-length b"xyz" leaves the immutable selection intact. The actual receiver stores/attaches the different bytes and returns Completed while preserving the original content address. A normal b"abc" transfer is the successful consistency control. BLAKE3 and length checks genuinely pass; semantic content binding is missing. verify_content_facet is called at preparation but not on received bytes. Trusted caller authorization is supplied explicitly; this is not a custody or network admission bypass. Share that content-binding contract before destination mutations.

Q249: presentation_binding hardcodes family=Semantic for every exact assertion occurrence. A real CollectionMember relation has a canonical Containment predicate and is preserved by the shared inspector, but the actual binding supplies Semantic to rules. A Cites control is correctly Semantic. The underlying inspector now carries every family's canonical assertion, so retaining a compatibility SemanticStatement name does not justify the label. Derive family through the shared predicate/assertion classifier and preserve unknowns explicitly.

Eleven native diagnostics pass:6 capture,3 product/transfer and2 presentation family checks. They use exact fresh current libraries and synthetic in-memory stores/graphs. All audit-owned executables/PDB/import outputs are marker-verified and removed. These are diagnostics, not a rerun of all original tests or headed/browser acceptance. No isolated target, Cargo home or worktree was created. The stable ordinary Cargo target remains shared with its live build owners.

The full transfer tests explicitly define relations counts as Surface records. They separately verify Resource records/assertions survive replicate/copy. Consequently the earlier count concern is qualified as a protocol/reporting scope question, not promoted as lost data. Both product imports preflight composed Resource truth through shared snapshot/journal machinery. Copy remaps Surface IDs but preserves exact Resource identity and claims; scenes remap member roles and representations. Invalid opaque dynamics can be retained, while executable opens reject them. Those are real shared owners and deliberately distinct acceptance boundaries.

The current UI calls sync_canvas_metadata_from_graph, preserving exact tag handles and other authors. The older label-based sync_canvas_metadata helper remains public but has no repository caller in the current search. MereHost local edits retain pending state on write refusal and only report acknowledged saves after persistence. Optional scene view restoration reports its own error while Canvas remains attached to the accepted owner graph. The complete surrounding retained-UI lifecycle still needs source review before any broader acceptance claim.

Projection compilation reexports the Scenomise compiler rather than implementing another solver. Host folding uses shared HostDatasetFolds and retains original relationship witnesses; temporary graphs and selection are view state. Same-authority rebind/checkpoint checks are explicit. Access inspection retains missing associations without guessing from equal URLs. Link inspection uses shared canonical held/history reads with directed Surface/Resource owners; exact-owner lookup refuses conflicting retained copies. Presentation rules use Backstep history and Muniment transactional compare-and-replace before installing committed state. Node hysteresis stays per occurrence/view generation, and unavailable inputs suppress rules explicitly. Responsive path fixtures disclose fixed-endpoint geometry and state spring allocation unavailable, rather than claiming measured graph force.

Unpromoted candidates remain documented: independent transfer-stage failure/recovery boundaries, deterministic Copy presence guards, host occurrence UUIDs lacking SourceBinding, and separately implemented inspector rebuild-eligibility shape checks. None gained a new counterexample here. Offer addressing filters presentation, while the personal lane explicitly carries plaintext to the admitted roster; it is not confidentiality. A source endpoint's Accepted begin intent means disclosure is enabled, not that destination delivery succeeded.

Upstream was fetched: origin/main=161368bfd2f33c2ea4002ebd86565b2102c1a780, with 9	4 local/upstream-only commits. Four upstream commits are newer than this audit checkout. They include a changed lockfile and Djinn source. Other live Cargo builds still use the shared checkout, so no merge was made during this pass. The evidence remains bound to the stated checkout rather than claiming it contains the fetched commits. Recheck build ownership and merge safely before the next basis transition; inspect changed source and refresh coverage through the existing transition rules.

All946 prior protected production/test hashes, prior cited audit evidence and the canonical document prefix remain preserved. No production or original test source was edited. A570-A589 record both implementation splits and verified shared ownership. The [product/host checkpoint](../../../support/receipts/authority-success-audit/product_host_review_receipt.json) binds complete coverage, fresh native evidence, upstream state, candidates and remaining limits. The full-source goal remains active.


### Personal sync, retained content and Forme source review, 2026-10-17

This pass completely reads20 current files463265 characters at c1b5d47156f2c940ccd67799963ed203df96362a, including the entire153229-character personal_sync.rs and its25069-character assertion helper, their embedded tests, carriage, retained content and all its submodules/tests, Canvas accessibility adapters/tests, remote board, promoted links/tests, session card/speed helpers, and Forme workspace/session/lifecycle source. Coverage is 1,471/3,727 files and 19,835,916/39,399,652 characters, without stale hashes or pending chunks. More than half the inventoried characters are read; the full Mere objective is still incomplete. Next is forme_workspace/tests.rs, not a claim that all Graphshell or Forme source is finished.

Q250 is source-confirmed: PersonalGraphEvent AddTag/RemoveTag still writes node.tags through the explicitly legacy Surface replay setters. Live InsertNodeTag/RemoveNodeTag uses shown Resource assertions. Because synchronized AddNode supplies a shown Resource, current node_content_tags reads Resource labels rather than falling back to those raw tags. The convergence/fingerprint and Redb tests inspect node.tags directly and preserve a passing legacy representation. The prepared public native comparison has not run. A migration must version new authoring through the shared tag-concept/assertion contract while retaining old signed bytes, not rewrite stored signatures or maintain another label-derived authority.

Q251 is source-confirmed: access and blob-observation maps use first entry by record_id with no same-ID fact equality check. A causally later changed fact is admitted and silently discarded, while collect_conflicts only examines concurrent operations. Identical duplicates can be valid idempotence; changed time/device/availability under one ID is a different case. Prepared changed-time and identical-duplicate access controls remain unexecuted; blob-observation behavior is source-only. Share immutable record identity checks rather than claiming complete chronology or current availability from first-wins suppression.

Q252 is source-confirmed: exact relation operations are stored before endpoint validity is known. assert_statement later errors on a missing endpoint and materialize propagates the error for the whole projection. Adding endpoints in a later operation cannot fix the earlier replay failure. Admission success here means stored, not successfully projected. The prepared public probe includes missing-endpoint/later-node and valid-prior-endpoint controls, not yet executed. Endpoint deletion and concurrent-edit semantics need a coherent deterministic owner; blindly ignoring failed events would select a policy without evidence.

The personal-sync-feature library build remains live in original session60676, with exact Cargo processes independently confirmed and its artifact log growing after the initial wait. No new native tests completed this pass. personal_sync_probe.rs and run_personal_sync_probe.py prepare5 public tests and require an authoritative terminal build receipt before execution. Poll this same handle; after actual success save final build-finished/exit/log hashes to personal_sync_build_terminal.json and run the prepared diagnostic once. An observation timeout is not termination. Do not restart the build or relabel Q250-Q252 as native reproductions. No isolated target, Cargo home or worktree was created; the ordinary stable Mere target remains shared with its live build owners.

Personal sync genuinely shares Stickleback OperationProcessor, causal helpers, MunimentStore and LogSync composition. Its local event fold is separate product semantics. Exact assertion IDs preserve pair, stable writer root, reason and causal withdrawal knowledge; method-version/evidence-order normalization is deliberate and tested. Current legacy non-Traversal retraction uses source-authored exact withdrawals and preserves other writers, ruling out the earlier broad clearing suspicion for that path. Legacy Traversal remains event-shaped. Attested sealed operations can be retained and counted unreadable without keys; raw unattested sealed operations are refused. Some NoKey/policy comments still describe the older refusal behavior. Header and body attestation/parents have distinct uses; no adversarial consistency bypass was demonstrated.

Reader membership is a deterministic fold separate from writer admission. Only current readers change it after bootstrap, and the final reader is retained. This source does not itself qualify key rotation/distribution or actual revoked-device exclusion. Carriage separates trusted issuer lease signatures from relay transport signatures, binds exact graph/slot/issue/expiry/payload facts, and delegates destructive supersession to shared per-slot protocol prune/erasure. Purge proposal is pure and blocked without held state. Production state-refresh/reuse of its snapshot policy remains a candidate, not a newly demonstrated deletion bug.

Retained content uses Pelt/Genet with caller-admitted bytes and verified Fleece/Eidetic clip readers. Staging validates packages before live import; graph reference persistence is a separate stage with retry wording. Hidden sessions refuse input/frames, producer actions recheck session/revision/target, and export scopes bind persona/graph/access/content session. Input tests cover actual Genet cancel behavior and bounded wheel custody, but none were rerun here and no browser/GPU/OS screen reader was used. The accessibility adapter uses shared Canvas/PhysicsBoard local actions; the original headless harness includes planted missing/dead-action controls, not a universal desktop acceptance claim.

Promoted links use kernel frozen Resources and exact-source drift comparison; incomplete coverage returns unavailable rather than unchanged. The full tests preserve independent captures, personal comments, exact withdrawals, and receiver coverage. Remote reading mode explicitly clears prior bodies and follows owner slots; generic sync_scoped reuse across different identity tokens still needs caller/physics-retention qualification before becoming a product bug.

Forme has another save protocol: whole-record Backend::put, while PresentationSession uses transactional compare-and-replace for a comparable workspace. Forme tokens prevent a foreign completion from changing one live session and FormeWriteFence prevents that process reloading an owner ahead of its own retired write. Neither proves a stored-baseline guard across independent editing sessions. Stale-save overwrite remains a documented API/caller-ownership candidate with no native/multitab trial. Forme lifecycle tests do correctly distinguish actual graph ownership, graph undo/redo masking, failed-save retained edits, accepted graph versus optional view error, and parked live physics. Workspace geometry and arrangement validation/source ownership remain separate from graph truth. Remaining workspace tests and retained UI consumers are still unread.

A590-A609 track these actual shared owners, copied adapters and qualification boundaries alongside the three new source findings. All979 protected prior production/test hashes, cited prior evidence and the canonical document prefix remain preserved. No production/original-test code was edited. The four fetched upstream commits remain unmerged while other builds use the shared checkout; this evidence stays bound to the stated basis. The [personal-sync checkpoint](../../../support/receipts/authority-success-audit/personal_sync_review_receipt.json) binds coverage, source findings, candidates and the exact pending build. The full-source goal remains active.


### Retained UI and independent Forme saves, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, this pass completely reads 22 files / 440,402 characters, including web_tree.rs, product.rs, Forme UI, retained reading host, all access/link/field/dynamics inspector adapters, command composition and selected feature-off/input helpers. Coverage is 1,493/3,727 files and 20,276,318/39,399,652 characters. Full Mere review remains incomplete. Source inventory was refreshed; all acknowledged hashes remain current.

Q253 promotes the prior Forme candidate with 2 passing public native checks. Two independent sessions load one MemoryBackend slot; a first acknowledged left-bound edit disappears after a stale second top-bound save also succeeds. The equivalent PresentationSession rejects the stale baseline and keeps its draft. The retained Forme UI uses that blind proposal save. Local request tokens and its per-page retired-writer fence do not compare independent stored changes. This is public in-memory API evidence plus a traced UI caller, not a browser/multitab or physical-disk trial. The exact qualified earlier current-build libraries, source hashes, test output and marker-verified generated-output removal are recorded in forme_stale_save_probe_receipt.json. A storage-baseline guard must preserve draft/history on refusal.

Q254 is source-confirmed: Reading::poll drains a submission-only Pelt effect, displays a host-handoff message and drops the full submission. Only navigation/export address tuples reach the outer frame-hook queue. The complete current composition has no downstream submission owner, outcome or retry. This is conditional on a retained engine emitting a submission; no form/browser trial ran. Explicit refusal is appropriate until a policy-aware owner accepts the complete request.

Q255 is another live duplicate: toolbar/menu hosted-fit routing chooses FitVisible, while TreePage::command uses controls::named_command and maps the same view:fit ID to Fit unconditionally. viewer_commands::action already accepts hosted context. Fit also resumes view-follow, so these are different policies beneath the same command ID. This route divergence is source-confirmed, without a browser framing trial.

Several suspicions were ruled out. Product's caret/IME hook is reached through appearance::focused_text, which explicitly delegates to product::focused_text. TreePage filters Forme pointer gestures to Primary before calling the helper. Failed clip intake recovers the created member into pending_intake. Saved product operations wait for acknowledgement and distinguish saved data from an optional Canvas refresh failure. Link actions revalidate actual source, endpoints and exact assertion; personal withdrawal is separately checked. Field inspection distinguishes held records, visible rectangular boundaries, selector membership and recognized-but-unevaluated effects. Dynamics inspection separates stored schedules from the current stage. Feature-off adapters cannot construct a live SavedProduct and explicitly refuse remote connection. Passive owner markers and host-input requests remain instrumentation, not acceptance evidence. GPU handles are borrowed from the host RenderCore and retired on suspension; no GPU/headed trial ran here.

A610-A625 record these genuine shared owners and copied routes. The 999 prior protected production/test hashes, cited evidence and canonical document prefix remain unchanged. No production/original-test code was edited. A fresh fetch confirms origin/main at 161368bfd2f33c2ea4002ebd86565b2102c1a780, with this basis 9 ahead / 4 behind. Those upstream commits remain unmerged while live shared builds use the checkout. No isolated target, Cargo home or worktree was created. The ordinary stable Mere target remains owned by its live builds. The Forme diagnostic outputs were removed after their receipt was recorded.

Personal-sync build session60676 remains live and is independently process-confirmed. Q250-Q252 remain source-only; its five prepared tests did not execute. Continue polling that same handle, write an authoritative terminal receipt only after actual completion, then run the preserved prepared diagnostic. Next whole source is web_tree/presentation.rs and the remaining retained editor/projection/host modules; all other unread Mere areas remain in scope. The [retained UI checkpoint](../../../support/receipts/authority-success-audit/retained_ui_review_receipt.json) binds this progress. Full-source goal remains active.


### Retained editors, input and scenario completion, 2026-10-17

This pass completely reads 28 files / 481,955 characters at c1b5d47156f2c940ccd67799963ed203df96362a. Together with prior current-hash reads, web_tree.rs and every inventoried source beneath web_tree/ are now fully read (53 files). This includes all retained editor/projection submodules, node adoption, the 99,148-character scenario lane, gestures, applet/community wrappers, remote/resident adapters, physics/speed and bounded observers. Full Mere coverage is 1,521/3,727 files and 20,758,273/39,399,652 characters. The full-source goal remains incomplete.

Q256: presentation::Pane loads rules using the initial actual graph session at boot and never rebinds when the active graph is replaced. Forme and query drafts have owner-change handling; the rules pane does not. Its old saved rules can resolve against new graph facts, and successful later saves still target the initial graph's slot. Transactional baseline checks within that old slot do not establish the active owner. Add actual owner lifecycle and distinguish admitted old-slot completion from a new-owner UI. This is a source-confirmed current path, not a browser graph-switch trial.

Q257: projection saving retains only pending_save=true. While an older write is in flight, the root frame hook can accept a changed data-projection-dataset and replace the editor directly, leaving that flag set. When the older operation succeeds, projection.saved() marks the replacement current editor clean and reports a saved executable projection. Persisted content belongs to the earlier snapshot. Bind acknowledgement to exact editor/source/owner revision and keep newer state dirty. This is source-confirmed; no browser save-race trial ran.

Q258: the root touch capture listener calls host.release() for pointercancel on a host-forwarded touch. That emits Up before the canvas listener reaches the now-existing cancel_pointer API. Up can accept a Forme drop and consume capture; the later cancellation can no longer take the intended Cancel path. The comment that the host lacks cancellation is obsolete. Direct scenario pointer cancel queues HostPointer::Cancel and bypasses this DOM capture adapter, so its success cannot qualify browser touch cancellation. Use real cancellation and separately preserve the deliberate first-finger release when beginning a pinch. No browser touch trial ran.

A626-A643 track these gaps alongside actual reuse: shared rule storage, typed projection compilation, registry-owned arrangement options, one occurrence selection, shared physics controls, common presentation/export scene production, HostPresenter/grouped source identities, RemoteSession/WebRTC composition, and exact query Resource/access checks. Query drafts explicitly retire after owner changes; freeze/open actions revalidate source revisions. Promoted writes recheck pending source scope and failed saves remain unconfirmed. Their unsaved comment/selected-UUID lifetime across owner changes remains a candidate, without a demonstrated retarget.

Resident boards explicitly use sync_reading with attachment/layout identity. The generic projection path passes no additional identity to sync_scoped; endpoint/epoch replacement and transient body carryover still need qualification. Applet wrappers delegate review/lowering to CapsuleMount, retain generations and explicit keep writes, but the portable CapsuleMount/Readings source is still unread and is next. Community wrapper outcomes delegate to a supplying Host; the wrapper is not peer authority. The review does not promote those calls to fresh signature, guest, retention or network acceptance.

Scenario source distinguishes direct HostPointer steps, synthetic browser events and externally performed fixture input. Read-only overlay observations are bounded and expose disabled/incomplete states; actual visibility instrumentation requires real hidden/shown transitions. Missing semantic-tree instrumentation defaults some negative observations to empty values, a candidate pending actual scenario availability/negative-control qualification. Passive geometry and status markers are not headed acceptance. No new native, browser, GPU, storage or network tests ran this pass.

All 1,021 prior protected production/test hashes and cited prior evidence remain unchanged; the canonical document prefix is preserved. No production/original-test code changed. No isolated target, Cargo home or worktree was created. Personal-sync build session60676 remains independently confirmed live in the stable shared Mere target. Its five prepared checks and Q250-Q252 native qualification remain pending. Poll that same handle; do not restart after an observation timeout. The [retained editor checkpoint](../../../support/receipts/authority-success-audit/retained_editor_review_receipt.json) binds coverage, trace hashes, findings and pending verification. Next is capsule_applet.rs, followed by its Readings and community Host/core, with every other unread Mere area still in scope.


### Portable community owners and completed sync diagnostics, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, this pass completely reads 28 files / 320,785 characters: CapsuleMount and Readings, both Graphshell resource hosts, the portable resource model/controller and both UI realizations, three content/owner adapters, lifecycle contract, original owner/reopen tests and fixtures, examples, feature manifest and historical source harnesses. Full coverage is 1,549/3,727 files and 21,079,058/39,399,652 characters, with current hashes and no pending chunks. Full Mere review remains incomplete.

Q259 is natively reproduced: Pane::refresh guards shared_by only when old and new are both Some. Some(A) -> None -> Some(B) succeeds for the same contribution/manifest and preserves attention, while direct A -> B correctly refuses. Three public checks cover the gap, direct-refusal control and legitimate first disclosure. The 25 original portable tests also pass, including the existing direct immutability check. This is disclosed-reading identity loss, not forged signed community authority. Withholding attribution must not silently prove a conflicting stable sharing root valid; forgetting/retention semantics need an explicit owner decision.

Q260 is a verified fixture/type mismatch: the Graphshell resource unit fixture's original ResourceRead literal lacks required shared_by. Its verbatim function compiled against the current type fails E0063. Q261 separately verifies current harness drift: direct compilation of the unchanged checked-in portable source-contract harness fails E0583 resolving control. The first audit attempt hit the same module-plumbing error and ran zero tests; its source, runner, log and preliminary receipt remain preserved. A new distinct diagnostic rebases only the control/test module path attributes and successfully runs current behavior and original tests. Neither failure is a claim that an older dated receipt failed at its original source revision. The complete Graphshell feature suite was not compiled or run here.

The original personal-sync library build60676 has now returned terminal exit0 with build-finished success. Final log hashes are bound by personal_sync_build_terminal.json. Its preserved prepared runner then completed all 5 public checks. Q250 now has native evidence that the raw synchronized Surface tag is absent from current Resource content-tag reads. Q251 reproduces causally ordered conflicting access facts being accepted and silently discarded, with identical duplicate replay as a passing control; blob-observation collision remains source-only. Q252 reproduces admitted missing-endpoint assertions failing materialization even after later nodes arrive, with valid prior endpoints as the passing control. These are signed synthetic operations and in-memory stores at current source, not installed-profile, transport, encryption-distribution or physical-disk acceptance.

The 33 passing native tests comprise 25 original portable checks, 3 new Pane checks and 5 personal-sync checks. There are also two deliberately expected compile failures for Q260/Q261, not two passed product test suites. The portable diagnostic imports current source with only module path plumbing rebased; actual signed owner/Redb/browser feature tests were fully read but not rerun. Native executable/PDB outputs were marker-verified and removed; compile-failure metadata outputs do not exist.

A644-A661 distinguish actual shared owners from adapter duplication. Capsule pack signatures and exact body/inventory checks delegate to Eidetic, while local review/grants and per-instance/execution proposal binding are separate. Multiple reading IDs share one verified body; retention feedback requires the exact still-live reading/revision. The unsigned catalogue is explicitly not community history. LifecycleReport is consumer-owned view data; the harness cannot itself qualify peer presence, key rotation or an actual activity implementation.

Community Pane centralizes private request identity and feedback. Controller and Reading retain separate UI/outbox compositions over that real shared core. Browser Host tokens and native admitted Mount have distinct responsibilities; the native mount verifies service, signer, retained authority, ledger and caller clock, and saves attention rather than credentials/payloads. Accepted feedback does not synthesize custody, hosting or participation. The real collection-fork owner rechecks destination/version/current capabilities; the simpler validate_request helper is not its final authority.

Three local content preflights coexist: generic owners::local_content assumes prior caller disclosure permission; resource_owner is an admitted current-member/public-only 4 MiB Open/fork bridge; handlers::Owner uses configured powers, terms, sealed custody and signed hosting policies/ceilings. All reuse Eidetic verification, but their manifest/source/budget admission remains duplicated. They are not interchangeable policies or one uniform reader owner. Both Cambium resource views share Pane but expose different details and currently consume different owner compositions. Track shared pure preflight opportunities without merging authorization/privacy decisions or claiming richer operations are adopted everywhere.

All 1,051 prior protected/traced production/test hashes, prior cited evidence and the canonical document prefix remain preserved. No production/original-test code changed. No isolated target, Cargo home or worktree was created. The stable Mere target remains shared with other active builds, so fetched upstream commits remain unmerged at this audit basis. The personal-sync wait is resolved; no audit-owned build or test remains pending. Next is moot_resources_host.rs and remaining host/applet consumers, followed by all other unread source. The [community owner checkpoint](../../../support/receipts/authority-success-audit/community_owner_review_receipt.json) binds the full scope, findings and receipts. Full-source goal stays active.


### Community hosts, applet lifecycle and receipt completion, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, this pass fully reads 29 additional files / 178,117 characters: the native community proof, all five web applet modules, resource-owner HTTP bridge, shared mount and receipt loader, co-op page/gateway/driver, both principal receipt collectors, all remaining web test helpers and presentation fixture, viewer budget tooling and entry, and the historical native proof harness. Coverage is 1,578/3,727 files and 21,257,175/39,399,652 characters. Full Mere source review remains incomplete. Prior 1,079 protected source hashes, 24 cited evidence hashes and the complete existing document prefix are preserved.

Q262 is a source-only native proof completion defect: finite timeout/early close can return normally without final-stage qualification; reusing an existing output directory can retain a stale qualification file. This does not invalidate a historical successful run. No fresh native-headed proof ran here.

Q263 has three current-JS counterexamples: a closed handle can review/install, an in-flight review can install after close, and an old handle's repeated close can disturb a live remount and clear its root guard. Q264 reproduces rejected Auto fallback loading leaving its refusal absent while the element remains marked mounted. These use actual unchanged modules with synthetic root, Rust callbacks, fetch, Worker and observer. Existing guest-output generation checks do not establish JS host-registration ownership; receipt pages can still catch the unhandled fallback rejection.

Q265 executes the exact current loader completion listener: HTTP 500 is labelled delivered; 204, transport failure and planted page-gate controls distinguish the boundary. Q266 executes actual unchanged headed-driver main with synthetic process/OS/HTTP/file receipts: failed and successful scenario verdicts both return normally, though metadata retains the correct verdict. Q267 executes the exact current nested receipt Handler with in-memory path/write interception: an allowed-origin ../escaped capture attempts a write outside --out and returns 204; an ordinary name stays contained and a wrong origin refuses. These are adapter/status/containment proofs, not browser, server, device or actual durable receipt acceptance.

There are 17 passing focused diagnostic cases (8 lifecycle/mount, 4 delivery, 2 driver-result, 3 sink-path), four passing original Python delivery-budget tests, and all four original presentation-mount script check groups pass. The first delivery diagnostic assumed LF while source uses CRLF and failed extraction before executing any test; its original source and failure receipt remain, with a distinct successful diagnostic. No original tests or production source changed. Native timeout behavior is source-only; no full Graphshell feature build, browser, GPU, real worker process, network or two-peer runtime was rerun.

A662-A673 track duplicated ownership and genuine sharing separately: Rust and worker pack verification maintain two canonicalization/verifier implementations; host JS lifecycle registration is separate from Rust pane generation; viewer and loader really share mount.js; three receipt collectors and multiple proof drivers maintain divergent naming, origin, size, write and exit-status policies. Resource-owner HTTP transport still delegates tokens/preparation to Rust. Co-op transport still delegates signed authority to its Rust peer; the practice canvas is a disclosed projection. The packaging mount row measures viewer.js and does not measure imported mount.js, so it is not a complete transitive-payload receipt.

Upstream was fetched to 175c4148a2575c1d5095e98df7864c02502f6366; checkout/upstream divergence was 9 ahead / 12 behind. The shared dirty checkout and another live Mere build keep integration deferred. This checkpoint stays on its exact source basis; no isolated target, Cargo home or worktree was created, and no audit-owned job remains pending. The next read is the web Cargo manifest and remaining configuration/pages/scenarios, then every other unread source. The [community host checkpoint](../../../support/receipts/authority-success-audit/community_host_review_receipt.json) binds the evidence and complete file list. Goal remains active.


### Complete web-source and browser-adapter review, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, this pass fully reads 211 additional files / 524,630 characters. Every one of the 245 inventoried source files under ports/graphshell/web is now complete, including scenarios, scripts, configuration, HTML and styles. The remaining thirteen web_* / webrtc_browser source modules and Graphshell's parent feature manifest are also completely read. Full coverage is 1,789/3,727 files and 21,781,805/39,399,652 characters. Full Mere review remains incomplete. All prior 1,108 protected source paths, twelve cited evidence hashes and the entire previous document prefix remain preserved.

Q268 is a source-only embedding violation: styles.css has unscoped nav button:nth-child geometry rules at wide and narrow widths. They also match the host page's navigation; the current embed fixture tests controls inside a paragraph, so it does not qualify nav isolation. Q269 is a verified authority/gate contradiction: web.rs says graphshell-web stays a root member sharing one lockfile and patch table. The actual manifest makes it standalone and restates dependencies/patches. Two successful locked/offline/no-deps metadata commands show 104 root members without graphshell-web, then one separate web-workspace member. No target compilation or recommended-command compiler failure was run.

Q270 is source-only: reconnect pops the retained session and moves its identity/delegation into asynchronous rejoin before signaling succeeds. An early signaling failure restores neither, so another reconnect has no retired identity to use. Q271 is a broader source-only ownership problem: joins/read pumps/close notifications capture a shared TreeRemote without a connection-generation guard, while adopt replaces its current live session. Old callbacks therefore reach the current session. The thread-local retirement stack also has no logical-host/session key. These findings establish missing ownership checks, not a demonstrated live cross-peer disclosure or cryptographic admission bypass. The callback/retirement repair is documented as larger work; parking browser frames deliberately prevents late callbacks into dropped closures and must not simply be removed.

A674-A689 distinguish real reuse from duplicated local ownership. The public element aliases now enter one retained tree, so old/new presenter script names cannot recreate a two-renderer comparison at current source. Historical results retain their original basis. Mesquite/Taproot own stepping/parsing; web_scenario only publishes retained-lane results and captured pixels. Dataset parsing, reducers, compiler, PhysicsBoard and FrameBudget are shared; browser fetch/configuration, source-bound practice localStorage, CSS role evaluation, clock/worker plumbing and transport lifecycle remain adapters with their own responsibilities.

The scenarios were completely read and not rerun. They explicitly distinguish synthetic workload size from actual site data, decision previews from enabled realization, accepted action receipts from refreshed custody/hosting facts, promises from verified availability, hidden view intent from source withdrawal, and promoted frozen copies from live assertions. They do not cover Q253 independent stale writers, Q256 active-graph rule rebinding, Q257 overlapped save acknowledgement or Q258 Forme touch/pointer cancellation; those prior findings retain their original runtime/source qualifications. No new browser, network, physical input, GPU, audio or disk qualification is claimed. Timing releases device marks on suspend, and hidden epochs survive frame pauses; GPU timestamps include idle between submissions.

The browser worker creates Blob URLs without an explicit revoke, and BrowserFrames owns an unbounded input queue. These are tracked local lifecycle/budget observations; no leak magnitude or flooding result was measured. The appearance adapter reports handled candidate refusal through UI status; an Ok return alone was not promoted to a false visible-success finding. The optional connect_remote export similarly warns on immediate refusal rather than claiming an authenticated link.

No production/original tests changed, no isolated target/Cargo home/worktree was created, and no audit-owned runtime job is pending. The dirty shared main basis remains unchanged while another Mere build is live; fetched upstream has not been merged. Next is remaining historical receipt source/tooling, native Graphshell consumers and every other unread Mere source. The [web source checkpoint](../../../support/receipts/authority-success-audit/web_scenario_review_receipt.json) binds complete file coverage, four findings, ownership map and metadata evidence. Goal remains active.


### Historical receipt-tool source review, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, all 91 inventoried files / 423,707 characters under ports/graphshell/docs/receipts are now completely read. This includes frozen scripts, native module harnesses, manifests, scenarios, HTML and lock patches; 89 files / 417,532 characters are additional to the prior checkpoint. Two harness files had been read earlier. Full coverage is 1,878/3,727 files and 22,199,337/39,399,652 characters. The full Mere source review remains incomplete. All prior 1,319 protected source paths, three evidence hashes and the prior document prefix are preserved.

Q272: five host-presenter package.py copies accept the expected number of successful records and assign fixed totals of 68-88 passing CPU tests without proving those tests ran. The exact unchanged scripts, executed with entirely in-memory filesystem/process substitutes, accept zero-test fixtures and emit those positive totals. Q273: those executions also carry earlier-A binary bytes into a package labelled with newer-B source HEAD/hashes. Artifact hashing is present, but the scripts do not validate a build record connecting the bytes to the stated source. Both findings concern a rerun verification weakness, not proof that original dated test totals or artifacts were false. Five positive counterexamples and ten failed-exit/missing-record refusal controls passed. The first checkpoint recorder also stopped before mutation because it counted two previously read harness files as additional; the original recorder and failure record remain. The initial probe failed in its mock Path sorting before completing a case; it and its failure record remain, with a distinct repaired probe and receipt.

A690-A699 distinguish preserved historical copies from competing live authorities. Native receipt harnesses import original modules by path. Later driver revisions add synchronization, value assertions and input hashing; the copies do not all provide identical coverage. Static G1/G4/H4/N4 HTML is dated receipt presentation, not fresh authenticated product interaction. Physical-labelled tests call headed Playwright mouse/keyboard APIs; this source review does not qualify human or OS-device input. A fresh navigator adapter query does not establish shared production rendering handles. Project navigation is fulfilled by a test route rather than fetched from the external service.

The keyboard diagnostic is explicitly diagnosisOnly; retaining a failed scenario there is not a false acceptance. The corrected scenario uses twelve Tabs rather than eleven. Some negative tests prove only a fail result while wheel planted controls identify the exact failed assertion. The toolkit qualifier parses eight actual named test totals and treats bench/boundary commands separately; supplied same-device facts remain supplied evidence. The download verifier checks the explicit downloaded bytes and privacy assertions, not browser HTTP headers. Retained-merge custody explicitly disclaims runtime qualification. No historical script, package, log, source or original test was rewritten.

No actual build, browser, GPU, network or deployment ran. No audit-owned job is pending; no isolated target, Cargo home or worktree was created. The shared dirty main remains nine commits ahead and twelve behind the last fetched origin/main, with another Mere build live; upstream additions are not covered by this basis. The [receipt-tooling checkpoint](../../../support/receipts/authority-success-audit/receipt_tooling_review_receipt.json) binds the whole-file coverage, two findings, ten ownership entries and diagnostic evidence. Next is native Graphshell consumers and all remaining unread source. Goal remains active.


### Native examples and receipt-owner review, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, eleven further files / 101,058 characters are completely read: all four Graphshell native examples, all five receipt modules, receipt_ingest and h7_sync_peer. Current coverage is 1,889/3,727 files and 22,300,395/39,399,652 characters. All prior 1,408 protected source paths, eight evidence hashes and the preceding document prefix are preserved. Full Mere source review is still incomplete.

Q274 is a source-proven CLI contradiction: dry-run creates/opens a durable redb database and writes artifact blobs before printing nothing written. Q275 loses newer producer provenance: extra fields survive parsing but run_facet drops them from the replicated events; the preservation test only exercises parsing. Q276 accepts an empty producer SHA-256 and replaces the missing verification with a computed local digest, while declared artifact byte size is not checked. Q280 separately leaves artifact names unconstrained: absolute paths, parent traversal or escaping links can read beyond the selected receipt directory. These are source-only paths; no user database, local private file, remote producer or peer disclosure was exercised.

Q277 is runtime-qualified for the exact current timestamp function, extracted verbatim and hash-bound, using standard-library-only native tests. UTC control passes. A positive offset is ignored; a fractional negative offset is also ignored, while an integral negative-offset timestamp refuses. Unicode fractions can panic at a UTF-8 byte boundary, huge years panic under checked arithmetic, and February 30 is accepted. The first probe had five passing cases and one incorrect expectation that all negative offsets refuse. That failed original probe and output remain, followed by a distinct corrected six-case passing probe. This is not execution of the complete manifest importer or durable graph host. The probe EXE/PDB were removed from the stable approved target after exit.

Q278 renders missing exit_code as Result=unknown but adds Failed to badges. Q279 propagates one unreadable/non-UTF8 *.json entry out of pending(), so the resident host retries the same failed scan and healthy receipts do not reach its authoring loop. Ordinary JSON syntax errors are skipped, but the function gives the caller no per-file diagnostic even though its comment says the caller logs them. These findings are source-only and their existing narrow tests do not exercise the reported path.

A700-A709 distinguish importer staging, authoring, producer verification, local hashing, path policy and presentation. The resident Djinn host really stages captures before authoring one signed event turn, then marks applied only after success. This is a real single-writer boundary, not a second sync engine; host restaging cannot restore discarded provenance or an absent producer digest. Two small local hex decoders duplicate parsing beside muniment's hash parser, but no divergent accepted hash was shown. The caller-owned toolkit example passes one booted device/queue through both adapters and reads pixels; its generation bump simulates resource rehydration without physical device loss, which its qualification explicitly disclaims. Compiler timing, dynamics and retained clip examples delegate their named shared owners. H7 checks selected sync convergence, not physical blob bytes, and was not run here.

No production or original test changes, full Cargo build, browser/GPU/peer run, actual importer invocation or isolated output was made. The dirty shared main and live build remain preserved. The [native receipt checkpoint](../../../support/receipts/authority-success-audit/native_receipt_review_receipt.json) binds all source hashes, seven findings, ten authority entries and exact diagnostic limits. Next is remaining native Graphshell source and then every other unread Mere file. Goal remains active.


### Complete Graphshell subtree and shared route seam, 2026-10-17

At c1b5d47156f2c940ccd67799963ed203df96362a, every one of the 528 inventoried files / 4,259,545 characters under ports/graphshell is now completely read. This pass adds 41 native source/scenario/test/helper files, plus the two shared Mere-overmap route modules: 43 files / 461,503 characters. Current full coverage is 1,932/3,727 files and 22,761,898/39,399,652 characters. All prior 1,419 protected source paths, twelve evidence hashes and the previous document prefix remain preserved. The full Mere review remains incomplete.

Q281 is source-only: G5 returns Ok after premature EOF, including before the second/third session are attempted. Logs can reveal the incomplete exchange, but exit zero cannot qualify the declared full rehearsal. H6's copied request loop is stricter: it checks ids, variants, hashes, transfer facts and expected revocation. Both reuse shared cryptographic owners and explicitly disclose the self-issued rehearsal grant; they do not qualify real grant distribution.

Q282 is a test-coverage contradiction: C4's module says it proves a second visitor is served rather than queued behind the first, but its first helper closes the session/carrier before the second helper starts. It proves sequential reuse. The actual host spawns service tasks; no production serialization defect is claimed. Q283 is a more substantial live duplication: native graphshell_mere::Connection independently sequences the same route/material/view/merge state that MereRouteSession owns for the browser, despite the shared adapter already supporting LineDriven native pumps. Their validation differs: an explicit session card without scoped media is refused by the shared adapter, while native read skips validation and retains earlier saved-view/read-only fields. This is source-only; broader consolidation is documented as a larger seam rather than implemented during the audit.

Q284 and Q285 have scoped native counterexamples. The original current distillery_w1.rs module is compiled fresh by path against exact retained library bytes from a successful previous build. Normal replay advances revision 1 to 2. A refused wrong-session diff and an unapplied missing-base diff leave acknowledgement at 1 but stamp resumed_by_diff=true. A refused malformed snapshot stamps snapshot_revision=999 over acknowledged revision 1. The shared ClientState correctly preserves the old scene; Chronicle's adapter mutates its receipt fields too early. Separately, a correctly self-hashed but unrequested PortableCard enters requested-instance frozen details because refresh_details checks content hash validity without comparing the requested scope/address. These are adapter defects, not proof of remote admission bypass or a second Distillery job-board authority.

The first diagnostic compile failed on an audit include-path mistake. Its receipt/source/log remain. V2 confirmed four observations, then failed because the substitution fixture advertised NativeGlyph and never exercised the PortableCard-only path. V3 corrects that audit advertisement and passes one positive control plus four counterexamples. Every generated probe EXE/PDB was removed. No full-current transitive Cargo build, original module test suite, Distillery daemon, native app, browser, GPU, network or physical rehearsal was run.

A710-A722 track real reuse, repeated setup and distinct projection contracts. Tabard definitions are shared; application role/settings mapping is local and storage precedes state acceptance. FrameBudget has one worker/main inference owner and explicit fallback bounds. Practice preserves source-bound canonical disclosure and bounded navigation rather than recomputing product music. Arrangement and Dynamics matrices repeat some assembly but use shared compilers, dynamics and facet composition. Query links are a display projection over canonical assertions with explicit legacy traversal summaries, not a second stored ledger. Small receipt generators delegate shared views; personae-vault delegates custody. The reboot helper is still tied to the named legacy Graphshell installation and does not qualify current Djinn restart.

One follow-up remains qualified rather than promoted: flattened facet values are prepended to PortableCard fields, while saved-focus paths find the first Address label. The typed owner contract and consumer tests need inspection before asserting a live wrong-focus outcome. Source reading uncovered the possible collision; no owner action or data mutation was performed.

No production or original test changes, isolated Cargo target/home or worktree were created. The earlier shared Mere build handles are now absent. Fresh git fetch succeeds and origin/main remains 175c4148, twelve ahead of this nine-ahead local main. The 42 upstream-changed paths overlap neither dirty tracked paths nor untracked paths; merge feasibility can now be checked without moving shared edits aside. This [Graphshell checkpoint](../../../support/receipts/authority-success-audit/graphshell_completion_review_receipt.json) binds complete reading coverage, five findings, thirteen ownership entries and diagnostic limits. Goal remains active.


### Current upstream integration and source requalification, 2026-10-17

Authorized upstream intake is now complete at 95515a1313785513efb38a433412941da7b68ed1, a clean merge of local c1b5d471 and fetched origin/main 175c4148. All nine earlier local commits remain ancestors. The preflight merge tree equals the resulting committed tree, and none of upstream's 42 changed paths overlapped shared dirty/untracked files. All 37 such non-audit files were verified byte-identical after the merge, including the entire prior audit-document content before this append. No shared work was stashed, reset, staged, committed into the merge or pushed. The previously live Mere build handles were authoritatively absent before integration.

The source denominator is now 3,731 files / 39,427,411 characters. A new inventory wrapper preserves earlier rules and additionally includes source-bearing .diff files, avoiding omission of the incoming narrow intake patch. Logs, JSON receipt/data and binary captures still do not count as read code. Four added sources are completely read: Cambium incremental_tests.rs, the historical narrow-lock-cache.py and .diff, and Graphshell span_editing.scn. The historical helper creates cache/ref machinery under its original machine's testing path; it was read, not executed, and no isolated Cargo home was created here.

Exactly one previously reviewed source changed: ports/djinn/src/resident_mere.rs. Its old 218,638 reviewed characters are verified text-identical under the ordinary checkout conversion, and its complete 8,477-character added test hunk was emitted and explicitly acknowledged under the existing transition rule. The tests admit both natal house systems, with/without supplied observations, and reject altered cusp/motion facts through the shared resident. They are read, not rerun. The six other modified Cambium source files were already unread and remain unread; this integration does not promote their existing retained test logs to new audit execution.

All 529 current Graphshell source files / 4,260,912 characters are complete. Full current coverage is 1,936/3,731 files and 22,788,727/39,427,411 characters, with no stale hash or pending reading chunk. The new span scenario edits explicit copies and checks discard restoration. The incremental test fixture compares styles, geometry, scrolling, sampled hit tests and paint lists with a fresh layout after successive DOM mutations; two 1,000-row tests are ignored in debug builds. No tests, build, native host, browser, GPU, network or physical workflow were rerun after this merge.

Q281-Q285 source text remains unchanged; their earlier exact diagnostic limits still apply. All prior runtime receipts retain their original source/supplier/build-artifact basis. In particular, the Chronicle adapter probe remains a fresh original-module check over retained dependency bytes, not a new whole-current Cargo gate after this lock/supplier intake. A723-A725 record integration, reading provenance and incoming-source limits. The [current upstream checkpoint](../../../support/receipts/authority-success-audit/upstream_175c_review_receipt.json) preserves the premerge inventory, dirty-byte receipt and checked source transition.

No audit-authored production changes or isolated target, Cargo home or worktree were introduced. Local main contains the fetched upstream and is ten commits ahead; the full audit remains active. Next is the remainder of shared Mere-overmap and Cambium owners, then all other unread Mere source.


### Shared route and Mere-view completion, 2026-10-17

All five Mere-overmap source files and all 23 Mere-view source files are now read, together with Cambium's complete component catalog, stylesheet and manifest. This pass adds 29 complete files and 409,540 reviewed characters. Current coverage is 1,965/3,731 files and 23,198,267/39,427,411 characters at unchanged merged HEAD 95515a1313785513efb38a433412941da7b68ed1. There are no changed inventory hashes or pending chunks. The full-source objective remains active and incomplete.

Q286 records an action-identity collision: add_node_facets prepends arbitrary human-labelled facet fields, while shared and native saved-view adapters take the first Address label for ViewIntent.focus. Exact current functions over real retained facet/card types reproduce a facet shadowing the still-present canonical address; no-facet and lowercase controls pass. The saved-view payload consumer is source-traced, but no owner SetView action, durable wrong focus, GUI exploit or authorization bypass was run.

Q287 records a projection wiring defect. MereView applies its fold projector before assigning focus, then overwrites the projector's result with state.focused. A valid fold removes a focused member from the Swatch while leaving focus on that removed key and supplying no stand-in focus request. Q288 records merge draft identity loss: replacing a pending merge directly with another branch's merge sharing a conflict key retains the old hand draft. The renderer uses that draft instead of the new target seed for its hand-choice request. State/event probes verify both observations and their ordinary no-fold, visible-node, same-merge, different-key and absent-merge controls.

The [view checkpoint](../../../support/receipts/authority-success-audit/mere_view_review_receipt.json) binds three successful facet checks and seven successful public Mere-view checks. The entire original current Mere-view crate was freshly compiled, over hash-bound dependency libraries retained from an earlier successful Graphshell build. The initial downstream compile failed because the audit rlib filename lacked Rust's required lib prefix; its logs and receipt remain intact, and the corrected v2 succeeds. This is not a fresh full-current Cargo graph, rendered DOM, native input, GPU, network, persistence or owner merge-apply receipt. All diagnostic rlibs, executables and PDBs were removed after their identities were recorded.

A726-A731 distinguish shared wire vocabulary, machine address authority, model/request boundaries, final projection focus, merge draft identity and acceptance evidence. Mere-view genuinely shares a presentation component and registered layouts; that does not unify native and remote owner orchestration. Read acceptance sources cover synthetic runner input and retained paint commands. The fold acceptance example hand-builds Swatch callbacks and compares graph snapshots without external facet sidecars; those checks do not establish the actual MereView fold wiring or all durable truth. The tests, scenarios and component catalog were read, not executed here.

The possible oversized tier-card reachability issue remains a follow-up, pending current host geometry evidence. No production code was edited. Shared dirty source remains byte-identical, and the previous audit document remains an exact byte prefix. No isolated target, Cargo home or worktree was created. Next is remaining Cambium toolkit source, rootstock and native/web host ownership.


### Cambium command engines and text editing, 2026-10-17

Nineteen further toolkit sources / 239,007 characters are completely read: compatibility actions, arrangement, atlas, backdrop reading, both command engines, component/context, every basic form/text module, inert detail panels and editor history. Current coverage is 1,984/3,731 files and 23,437,274/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. The inventory has no stale hashes or pending chunk. Full Mere remains incomplete.

Q289 records incompatible command-tree interpretation. The shared context engine only stores one submenu parent, while the menubar independently stores a recursive path. With the same File -> Export -> HTML tree, context-menu Enter emits an activation for non-leaf Export and pointer disclosure never renders HTML; the menubar reaches the HTML leaf with its complete path. The context API does document a depth-one limit, but it accepts deeper recursive input without refusing or lowering that unsupported structure. Q290 records stale availability in both engines: after an open File group becomes disabled, its individually enabled child still emits an activation. The individually disabled child control remains inert. These are UI routing findings, not exercised backend authorization failures.

Q291 records broken grapheme caret arithmetic at the shared insertion splice. Appending a combining acute accent to a leaves one grapheme with caret 2, and the first Backspace deletes no text. Inserting that accent between a and b jumps the caret after b, so the next x produces a+acute+bx. Precombined-buffer and ordinary insertion controls pass. Q292 records a narrower changed-result defect: a boundary caret move clears a visible composition but returns false despite the documented logical-or-composition change contract. No redraw-gating consumer was established, so missed redraw or persistence is not claimed.

The [toolkit checkpoint](../../../support/receipts/authority-success-audit/cambium_commands_review_receipt.json) binds six passing command checks and seven passing text checks. Command modules are current original source included by path over retained real Cambium runner/ScriptedDom artifacts; the text diagnostic includes current original leaf/editor modules through an audit-only path wrapper. An initial command fixture panicked on its own RefCell borrow after three successful observations, and the initial text wrapper failed Rust module resolution. Both original attempts remain intact; distinct corrected attempts pass. No native window, OS keyboard, physical IME, GPU, network, persisted document, command backend or fresh full-current Cargo graph was exercised. All probe outputs under the stable target were identified and removed.

A732-A738 distinguish real action-list delegation from separately implemented menubar navigation; one atlas transform from caller-owned terrain/motion; component local state from parent authority; multi-listener click/key registries from single-path observer registries; shared text storage from explicit alternative history entry points; and inert fact/selection controls from source editing. Single-path nested focus composition, portable-child key collisions, host text policy and oversized Mere tier-card reachability remain qualified follow-ups. The existing source tests were read, not broadly rerun.

No production code or shared WIP was changed, the prior audit document is an exact prefix, and the Git index is unchanged. No isolated target, Cargo home or worktree was created. Next is unread toolkit source after detail_panel, followed by rootstock and native/web host ownership. The full-source goal stays active.


### Retained input, focus and pane-state preservation, 2026-10-17

Seventeen more toolkit sources / 311,125 characters are completely read, including disclosure, element and event wrappers, file/focus helpers, source-fold projection, Frisket, keyed/portable sequences, the runner, propagation, exports and legacy menus. Current coverage is 2,001/3,731 files and 23,748,399/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1, with no stale inventory hashes or pending chunk. Full Mere remains incomplete.

Q293 proves sibling event-wrapper drift: changing capture on a retained click listener is ignored, while the parallel key listener updates correctly. Initial capture and bubble controls behave as declared. Q294 proves that Frisket's cross-stack state-preservation claim exceeds PortableKeyed's documented source-before-target limit. An active tile carrying component-local count 1 keeps its node/count on an earlier-to-later move, but a later-to-earlier move retires the original node and starts at count 0. Both stacks keep spare tabs, so tree collapse is not the explanation. No persisted document loss is claimed. The existing Frisket test explicitly selects the preserving direction; that is a bounded receipt, not proof of both directions.

Q295 records accepted nested focus wrappers silently retaining only the outer observer. Q296 proves that both click and key dispatch stop all remaining paths after ordinary stop_propagation, including other listeners on the same node, contrary to the declared ordinary-versus-immediate distinction. The old at-most-one-listener comment survived the registry conversion to per-node vectors. No current JavaScript dispatcher was executed, so this is a native contract failure rather than a new cross-runtime parity result.

Q297 proves that a view-requested focus change seats focus without the Gained notification delivered by the direct setter; deferred request consumption bypasses the shared transition helper. Q298 proves that a collapsed disclosure's mounted hidden action remains in Tab traversal and Space invokes it while the panel stays closed. The expanded control behaves normally. The counterexample uses the authored hidden attribute, not a claim about every CSS/computed-style visibility rule or an authorization bypass.

The [retained-event checkpoint](../../../support/receipts/authority-success-audit/retained_events_review_receipt.json) binds a fresh original current default-feature Cambium compile against twelve exact retained dependency artifacts. The initial seven-check probe and expanded seventeen-check probe both pass; the expanded set includes those first seven, so these are seventeen distinct checks, not twenty-four independent cases. They use the actual current runner, components, wrappers, disclosure and Frisket through synthetic keyboard/pointer input and inspect DOM identity/local state. No native window, OS input, GPU, accessibility adapter, network, persisted document or fresh full-current transitive Cargo graph was exercised. Compiling source does not count as reading it; unread toolkit files remain unread in the ledger. Both fresh rlibs and their executables/PDBs were identified, hashed and removed.

A739-A745 record effective focus eligibility, source-fold provenance, frame versus tree ownership, listener reconciliation drift, transition publication, cancellation-contract limits and explicit host/compatibility boundaries. FoldProjection borrows the source and validates byte lengths/ranges; a length witness is not a revision/identity witness. Legacy menu is a declared click-only renderer with caller-owned editor keyboard state, while old action-list names actually delegate to the shared command engine. Deprecated backend names are aliases rather than duplicate implementations.

No production code or shared WIP was changed, the prior audit document remains an exact prefix, and the Git index is unchanged. No isolated target, Cargo home or worktree was created. Broad order-independent pane retention and focus-publication repairs stay documented rather than diverting the audit. Next is unread graph-canvas, grids, overlay/input wrappers and remaining toolkit code, then rootstock and native/web hosts. The full-source goal stays active.


### Graph-canvas identity and keyboard reachability, 2026-10-17

Six further toolkit sources / 192,209 characters are completely read, including every line of the 131,970-character graph_canvas module and its tests, presented Frisket, grid, optional highlighting, hover and callback action conversion. Current coverage is 2,007/3,731 files and 23,940,608/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. There are no stale hashes or pending chunks; the full-source objective remains incomplete.

Q299 verifies that graph logical identity and native focus can separate after a node reorder. Target views are positional vectors, while focus emphasis is held by logical Id. Focus node 1, swap the two nodes, and the retained focused button now represents node 2 while node 1 still carries the focused class. Enter activates node 2. Both callback-owned and component-owned graph forms reproduce it, with initial focus and explicit blur/refocus controls. The data-key still identifies the displayed node; it is automation metadata and does not key retained view state.

Q300 verifies that relation and Expand actions work by pointer but are missing from toolkit Tab traversal. Those buttons use bare on_click, while node buttons use explicit focus registration. Tab cycles between nodes and never reaches either action; direct pointer callbacks work as controls. Source tests exercise relation/Expand pointer activation without establishing keyboard reachability.

The [graph checkpoint](../../../support/receipts/authority-success-audit/graph_canvas_review_receipt.json) binds nine successful focus/reachability checks through a fresh original current default-feature Cambium compile over twelve exact retained dependency artifacts. The checks inspect actual current runner focus, DOM identity/classes, Tab traversal and emitted callbacks. They do not execute paint leaves, native input, GPU, accessibility bridges, owner persistence or a fresh full-current transitive supplier/Cargo graph. Optional highlight code was fully read but is not compiled by these feature-off diagnostics. The audit rlib, executable and PDB were recorded and removed.

A746-A752 distinguish shared graph geometry from source truth; logical ids from positional retention; pointer success from keyboard reachability; component-local emphasis from host leaf refresh; shared stack rendering from separately walked presentation structure; virtualized arrangement from row identity; and actual shared highlight/action vocabularies. Presented Frisket correctly omits closed content rather than merely hiding it, leaving a useful contrast with the executed Disclosure failure Q298. Stateful grid reuse and host paint-emphasis refresh remain qualified follow-ups. The source tests were read rather than broadly rerun.

Exact declaration locations in the mutable index were corrected for Q296 (propagation.rs:104) and Q298 (runner.rs:1506), with the prior immutable checkpoint retained and a separate location receipt. No finding outcome changed. No production code or shared WIP was edited, the prior audit document is an exact prefix, and the Git index is unchanged. No isolated target, Cargo home or worktree was introduced. Next is the remaining toolkit stylesheet, multi-runner, opened-link, overlay/input and retained-mount code, then rootstock/native/web hosts and remaining Mere source.


### Forest window custody and overlay behavior, 2026-10-17

Ten more toolkit sources / 143,052 characters are completely read: instrument styling, multi-runner, opened links, overlay geometry/surfaces, uniform DOM elements, popovers, radio/range controls and retained mounts. Current coverage is 2,017/3,731 files and 24,083,660/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1, without stale hashes or pending chunks. A retained-mount emission exceeded its output budget; that pending chunk was explicitly discarded without credit and completely re-emitted in two smaller acknowledged chunks. The full-source goal remains incomplete.

Q301 verifies a false forest-preservation claim. Forest projections share one document, but each creates its own context and drains its portable nursery before the next projection rebuilds. Moving a stateful Frisket tile between projections replaces the original node and resets component-local count 1 to 0 in both directions, including source-before-target. Shared owner truth updates correctly and unchanged-owner refresh preserves state. The separate RetainedMount path preserves the actual owner runner/node/count across a destination-first transfer. Thus a shared document and one application State do not themselves establish cross-window view custody. Existing multi-runner source tests prove sibling roots and shared counter updates, not preservation of the moved child. This is broader than Q294 single-tree ordering loss.

Q302 verifies that the public modal overlay flag advertises aria-modal=true while leaving background controls in ordinary Tab traversal; Space activates the background callback. The nonmodal control has the same reachability without the modal assertion. No .modal(true) product call site was found in the searched in-tree Cambium/ports Rust code, so this is an API/semantic gap, not a deployed product bypass or headed accessibility result.

The [forest/overlay checkpoint](../../../support/receipts/authority-success-audit/forest_overlay_review_receipt.json) binds seven successful focused checks through the original current default-feature toolkit compiled over twelve exact retained dependency artifacts. The first audit fixture failed Action type inference; its compiler logs and generated long-type text remain retained. Cleanup initially refused that extra compiler text; after the compiler exited it was marker-checked, copied into audit evidence and removed from the stable target. A distinct typed v2 succeeds. Its rlib/executable/PDB were recorded and removed. No native windows, scroll/animation preservation, layout, GPU, network, owner persistence, accessibility bridge or fresh full-current Cargo supplier graph was tested.

A753-A759 distinguish single shared app state from per-projection custody, the real independently owned RetainedMount path, repeated overlay implementations, semantic modality from input isolation, exact opened-link action context, controlled radio/range policy and one DOM element representation. General Popover separately implements anchored dismissal/Escape/focus return; DetailPopover uses OverlaySurface. The shared overlay-behavior claim is therefore narrower than the full family. RetainedMount explicitly leaves current-station input checks, scoped layout and destruction order to the host. Those host contracts remain to be read and verified.

Opened-link presentation delegates compilation to Scenomise and preserves source binding/revision, direction and assertion identity in effects requested from the host; it does not grant source-edit authority. RangeScrubber shares a real reconcile policy, with extreme finite arithmetic and cancel-preview behavior retained as unverified follow-ups. Existing source tests were read rather than broadly rerun. No production code or shared WIP was changed, prior audit text is an exact prefix and the Git index is unchanged. No isolated target, Cargo home or worktree was introduced. Broad cross-context preservation and modal focus policy remain documented under the audit-diversion rule. Next is the remaining toolkit modules/tests, then rootstock and native/web hosts and all other unread Mere source.


### List actions, tab engines and session containment, 2026-10-17

Fifteen more toolkit files / 232,028 characters are fully read, plus the first 10,463 characters of the integration-test module. Coverage is now 2,032/3,731 files and 24,326,151/39,427,411 characters, without stale source hashes or pending chunks. A fresh fetch reports this HEAD includes origin/main (ahead 10, behind 0). Full-source review remains incomplete.

Q303 verifies a misleading keyboard success path in sectioned_list: Enter and Space run the row handler and update local state but discard its returned navigation action; pointer activation returns that same action. The source explicitly claims the key action bubbles, while its existing tests only check local click mutations. No product call site beyond module tests was found in the searched in-tree Rust source. Q304 verifies a broken panic boundary: ContainedSession asks leaves() twice, guarding only the first invocation. A second-call panic escapes and leaves the session reporting Available with no recorded failure. A first-call panic control retires the guest correctly, and a stable registry control succeeds. These are current API counterexamples, not headed product failures.

The [list/session checkpoint](../../../support/receipts/authority-success-audit/list_session_review_receipt.json) binds eight focused checks through two fresh compilations of original current default-feature Cambium over exact retained dependency artifacts. Generated rlib/executable/PDB outputs were recorded and removed. No fresh complete supplier graph, native input/layout/GPU, persistence, network or assistive-technology gate was run. Existing source tests were read, not broadly rerun.

A760-A766 track the separate tab_bar_view and selection_bar roving engines alongside the separate RadioGroup path, shared reorder output versus caller persistence, resize/split geometry boundaries, the list action discrepancy, local settings drafts versus parent truth, surface/containment ownership, and genuinely shared field projection/tag constructors. The one-tab-bar claim is true inside its constructor family, but the toolkit has another independent tab engine with different disabled/manual-selection features. Consolidation must preserve those differences. An all-disabled selection-bar check required explicitly focusing its non-tabbable root; ordinary Tab exposes no seats, so this remains a qualified follow-up. Variable-row geometry, moving resize-handle coordinates, numeric/setting identity boundaries and host leaf-execution containment remain to be traced.

Production and concurrent shared files remain byte-preserved, prior document text remains an exact prefix, and the index is unchanged. No branch, worktree, isolated target or Cargo home was created. Continue at toolkit tests.rs character 10,463, then the remaining toolkit and hosts.


### Toolkit complete; floating identity and Rootstock boundaries, 2026-10-17

All 80 inventoried Cambium toolkit files / 1,456,738 characters are now fully read at current hashes. This pass finishes the 131,598-character integration-test module and the remaining text, wheel/value, title/pane chrome and workspace sources, then reads Rootstock capture/lifetime, initial embedding/GPU/egui and accessibility-test source. Seventeen files are newly complete; 248,048 additional characters were read because the integration-test prefix was already credited. Overall coverage is 2,049/3,731 files and 24,574,199/39,427,411 characters, with no stale hashes or pending chunks. Full-source review remains incomplete.

Q305 verifies floating pane state transposition. Ordinary and presented workspace entry points construct visible float views as unkeyed Vecs. Raising float 1 changes model z order; positional reconciliation then gives float 2 the original float-1 node and local count 1, while float 1 takes float 2's node/count 0. An unchanged-order control preserves identity, and another actual callback increments the inherited wrong-owner count to 2. This happens within the float station, independently of the documented tree/float or cross-projection rebuild boundary. The ordinary path was executed; the presented variant shares the construction/renderer by source trace only. The [checkpoint](../../../support/receipts/authority-success-audit/workspace_rootstock_review_receipt.json) binds three current original-toolkit checks over exact retained dependency artifacts; generated build outputs were recorded and removed. No native float windows, layout, durable pane state or whole-current supplier graph was tested.

A767-A773 record the integration tests' actual limits, workspace station identity, primitive/single-path event wrappers, presentation-only title/pane status, appearance-bound capture delegation, owner/appearance/GPU separation and egui input/callback authority. Existing source tests explicitly expect target-before-source child replacement, qualifying Q294; multi-runner tests do not establish Q301 forest preservation, and separate stacked/stop-propagation tests omit Q296's interaction. Workspace prose is honest about cross-projection loss but overstates general local preservation. Per-node wheel/value wrappers repeat registration machinery and have narrower semantics than click/key's vector registry.

CaptureScope checks disposal/generation, post-poll cancellation and stamp/pixel extent; GPU readback is delegated to render-host. Embedding uses supplied owner callbacks and GPU handles, with typed refusal paths and no storage authority in rendering callbacks. Rootstock optional egui/GPU modules and accessibility tests were read, not compiled or executed in this pass. Remaining adapter scheduling, visibility/input cancellation and finite physical-size boundaries are documented follow-ups, not product failures. Production/shared WIP remain byte-preserved, prior audit text is an exact prefix, and the index is unchanged. No isolated target, Cargo home or worktree was created. Continue with Rootstock embedding/iced.rs.


### Rootstock input scheduling and host boundaries, 2026-10-17

Twelve more Rootstock files / 269,372 characters are fully read, covering remaining embedding adapters/tests, file responses, frame/host/input code, public vocabulary/accessibility projection, shadow mount probes and multi-window turns. Coverage is now 2,061/3,731 files and 24,843,571/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1, without stale hashes or pending chunks. Full-source review remains incomplete.

Q306 verifies misleading deferred-input claims. drain_pointer takes the queued batch, but each event can run after_dispatch -> with_ctx -> apply_pending -> drain_pointer recursively. Queue A=zoom in, B=zoom out: the control observes zooms [1.1, 1.0]. If A's hook queues C=zoom in, one outer drain observes [1.1, 1.25, 1.1], proving A,C,B delivery with C already consumed. The source instead promises ordered delivery and callback events deferred to the next drain rather than looping. The fixture bounds itself to one added event: no unbounded recursion/stack overflow or native product failure is claimed. The [checkpoint](../../../support/receipts/authority-success-audit/rootstock_host_review_receipt.json) binds three passing checks through fresh original current Cambium and Rootstock compiled over 22 hash-bound retained supplier libraries. All generated rlib/executable/PDB outputs were recorded and removed. This is not a fresh whole-current Cargo supplier graph or native/GPU gate.

A774-A781 distinguish shared rendering from repeated toolkit adapter lifetime wrappers, options from one runtime state, actual Host input reuse from broken queue scheduling, logical edits from visual caret correction, node-only file responses from request authority, frame/capture identities, custom shadow probes from production stylesheet paths, and genuinely shared multi-window custody. File-response source has no request/source/handler generation, but no wrong-owner import is promoted without a supported consumer trace. MultiHost restores borrowed shared custody only on ordinary return; unwind/caller boundaries remain unverified. Producer and presentation/recovery modules are still to read.

The host explicitly documents that retained leaf fragments are registered/refreshed but not placed by paint emission; this staging layer is not a demonstrated performance feature. Shadow mount tests explicitly bypass OwnedLayout's production document-scope sheet path with resource-aware styles: their scoped test results must not be sold as production guest isolation. PresentedFrame honestly certifies queue submission, not compositor visibility; paint-envelope capture is CPU commands plus GPU references, not captured pixels. Optional Iced/egui adapters and source tests were read, not executed here.

Production/shared WIP remain byte-preserved, prior audit document text is an exact prefix and the index is unchanged. No branch, worktree, isolated target or Cargo home was created. Continue with multi_host_tests.rs and OwnedLayout, then remaining host machinery and all other unread Mere code.


### Retained caret authority, inactive embedding constructor and presentation, 2026-10-17

Eighteen more Rootstock files / 194,499 characters are fully read: multi-host tests, retained OwnedLayout and adjacent geometry/interaction/paint tests, presentation machinery and initial producer/portable lifecycle code. Overall coverage is 2,079/3,731 files and 25,038,070/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1, with no stale hashes or pending chunks. Full-source review remains incomplete.

Q307 verifies a second editor overwriting the correct toolkit result. Rootstock's visual caret correction scans Unicode scalar text rather than shaped lines, uses buffer ends for line motion and collapses from the selection focus. Eight current Host checks compare with/without layout: Home on line two goes to buffer byte 0 instead of line byte 3; End on line one goes to buffer byte 5 instead of line byte 2; Left on selection 1..3 collapses at 2 instead of 1; Right across a combining grapheme snaps back to byte 0 instead of advancing to 3. The no-layout controls preserve the toolkit answer. These are actual Host/model observations, not native injected keys, visual pixels or bidi acceptance.

Q308 verifies a stale optional constructor: embedding::View::frame omits the now-required ProducerFrameInfo::purpose. Its exact current initializer text, against freshly built current Rootstock's public type, produces only E0063 for missing purpose. The default/no-optional-feature toolkit and host build first, explaining why those builds miss it. The initial extraction fixture failed to resolve the fresh Cambium dependency (E0463); the failed receipt is preserved, then a distinct v2 explicitly loads that dependency and obtains the intended missing-field diagnostic. This is a typed initializer check, not a full embedding/egui/Iced feature graph build.

The [layout checkpoint](../../../support/receipts/authority-success-audit/rootstock_layout_review_receipt.json) binds the eight successful runtime checks and expected compile refusal over retained exact supplier libraries. Generated fresh rlib/executable/PDB outputs were recorded and removed. A782-A789 track manual-node multi-window tests versus actual portable view custody, competing caret editors, retained wrapper stubs, repeated geometry/scroll policy, interaction/paint test limits, shared presentation preparation versus duplicated completion tails, the producer-purpose migration and staged portable capture. Multi-host's manual moveBefore test does not prove Q301 forest component preservation.

OwnedLayout's animation query/tick return false, its scrollbar append body is empty and selection_style always returns None; surrounding prose still describes richer behavior. These source facts are recorded without claiming every supplier path lacks those features. Producer-local mapping delegates to ElementGeometry, while painted/caret/scroll helpers maintain separate calculations that still need consistency probes. Paint-slot tests inspect real command ordering under clips/opacity/transforms, but were read rather than rerun. Presentation correctly verifies driver stamps, keeps proposed sequence separate from successful commit and shares WgpuPresentation across native/materialized scenes; no GPU/acquisition/recovery acceptance was run.

Portable capture validates aggregate scenes and the final envelope before registry lifecycle/cache changes. Producer callbacks can nevertheless mutate their own clock before a later admission failure; policy remains a named follow-up rather than an invented rollback rule. Production/shared WIP remain byte-preserved, prior audit text is an exact prefix and the index is unchanged. No isolated target, Cargo home or worktree was created. Continue with producer/portable_tests.rs, then remaining producer/recovery/host code and all other unread Mere source.


### Rootstock completed; browser host authority and cleanup, 2026-10-17

The complete current Rootstock source area is read (55 files / 714,015 characters), as is the browser host (30 files / 273,208 characters). Since the preceding checkpoint, reading coverage advanced by 46 files / 449,720 characters to 2,125/3,731 files and 25,487,790/39,427,411 characters. One of these files already had a protected source hash from an earlier narrow finding; 45 newly guarded complete files total 437,815 characters. All current inventory hashes match. HEAD remains 95515a1313785513efb38a433412941da7b68ed1; a fresh origin/main fetch found no newer commit, and HEAD includes origin/main (ahead 10, behind 0). Full-source review remains incomplete.

Q309 records browser key conversion's scalar/grapheme contradiction: a multi-scalar grapheme input value becomes Named(Other) with no text, despite the contract promising one grapheme. This is a source-proven API conversion loss, not an observed OS/browser/IME event. Q310 records the exported standalone file chooser's forgotten closures retaining input/callback state after its Drop removes the DOM element. The actual mount uses the lifetime-owned path; no in-tree standalone constructor consumer was found. Both are documented without production changes.

The [checkpoint](../../../support/receipts/authority-success-audit/rootstock_web_review_receipt.json) binds 15 passing original current capture/hover/prepared-frame tests. Cambium and Rootstock were compiled fresh over exact retained supplier artifacts; their fake-driver receipts do not establish GPU/native presentation or a whole-current Cargo graph. Five original browser-lifetime tests also pass directly with the standard library. These establish native phase/startup/future/cleanup behavior, not actual browser microtask/listener removal. First harness failure requested the wrong filename; the second failed E0463 dependency resolution. Both failed attempts are retained with zero test credit; a distinct v3 ran all 15 successfully. Generated rlibs/executables/PDBs were recorded and removed.

A790-A804 distinguish producer-local/physical keys and validated remapping; texture-view and whole-frame admission questions; SceneProducer callback/source/purpose authority; bounded DOM/wake/command adapters; test receipt limits; native versus wasm compilation; duplicated input/file ownership policy; shared accessibility projection versus DOM mirror state; mounted versus presented status; backend fallback; portable materialization; GL turn-qualified readback; shared recovery; and the finite Workshop fixture's precise success conditions. The browser crate accurately warns that native checks omit most code. Its mount returns after a void draw, so mount success is not a PresentedFrame receipt. IME methods explicitly remain no-ops without an editable host. Browser, screen-reader, GPU and full optional-feature acceptance remain unexecuted.

Producer validation of underlying textures versus exact views, invalid-output empty commands, export-purpose callbacks, focus/aria-hidden cleanup and transformed mirror positioning remain named follow-ups rather than speculative proven bugs. Shared production/WIP hashes and index are unchanged; prior audit text is an exact prefix. No isolated target, Cargo home or worktree was introduced. Continue with the native host and every remaining unread Mere source file.


### Native host receipts and a second accessible-name authority, 2026-10-17

Fourteen native-host files / 170,606 characters are fully read: manifest, smoke/T4 examples and scenarios, caption/decorations/file adapters, the complete headless Harness and main native host module. Overall current coverage is 2,139/3,731 files and 25,658,396/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1, with no stale hashes or pending chunks. Full-source review remains active and incomplete.

Q311 records a misleading process success path: T4 only writes matched/non_vacuous booleans and raw counters. It never rejects failed/incomplete cases, and missing counters compare equal as None == None. A normal exit is therefore not an accepted receipt. Q312 verifies the unlayered reference's CSS places it at (4,4,64,48), overriding intended left:616px/top:16px with the appended local-offset declarations. This invalidates its claimed independent third-column non-vacuity control. Equal-width frame crops versus fixed 300px panel spacing remain another explicit geometry follow-up. The full GPU T4 fixture was not run.

Q313 verifies duplicated naming authority: native caption lookup claims shared accessible naming but inspects raw aria-label only. Text-only and aria-labelledby Maximize names resolve in the actual neutral accessibility projection and fail native lookup. An aria-label Maximize superseded by aria-labelledby Agrandir remains the native target despite the projection correctly announcing Agrandir. Current WinitHost::refresh_snap_layout_rect uses this helper for its Win32 maximize region. The observation proves lookup disagreement, not an observed OS Snap flyout.

The [native checkpoint](../../../support/receipts/authority-success-audit/native_host_review_receipt.json) binds four passing focused checks using five unchanged extracted source functions with the real current Cambium Runner, Rootstock OwnedLayout and neutral projection over exact retained suppliers. This is not a full native host build or OS/GPU acceptance. The initial fixture failed E0277 because its generic calls passed Ref<ScriptedDom> instead of reborrowing ScriptedDom; it is retained with zero test credit. A distinct v2 fixes only that fixture borrowing. Its raw suffix counter reports three because --nocapture inserts the geometry print inside one test line; the unchanged standard test summary says four passed, zero failed, and a separate count receipt records that discrepancy without rewriting evidence or claiming a rerun.

A805-A812 track raw receipt collection versus acceptance, duplicated caption semantics, real-core harness limits, native geometry/command policy, open/resume versus presentation success, shared-device lifecycle, file-error erasure and smoke scenario evidence limits. Remaining native tests, multi-window code and platform bridges are not counted as read by their compilation or a helper reference. Production/shared WIP, earlier audit text and Git index are preserved. Generated diagnostic rlibs/executables/PDBs were recorded and removed; no isolated target, Cargo home or worktree was introduced. Continue with headed_tests.rs and every remaining unread source file.


### Native host completed; shared accessibility base, divergent final authority, 2026-10-17

Forty-one further files / 385,557 characters are fully read. The native host is complete as source (48 files / 457,068 characters), along with cambium-winit-a11y (4 / 86,997) and cambium-winit (3 / 12,098). Overall coverage is 2,180/3,731 files and 26,043,953/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1; all inventory hashes match and no chunk is pending. Full-source review remains active and incomplete.

Q314 verifies native accessibility omits retained scroll correction. Both a viewport and a nested scroll put the actual button at y=150 and the neutral projection at y=150, while unchanged native lowering reports y=300. Q315 verifies an actual button changed to display:none disappears from the native tree but remains in its action map; the unchanged mapper returns a Click and real Host dispatch increments the hidden control state. Q316 verifies ordinary custom-leaf naming diverges: native leaf hooks overwrite Author name with Leaf name, while the browser planner gives the author label precedence. These four focused checks pass over freshly compiled current Cambium/Rootstock with exact retained suppliers. Mapping storage is fixture-owned, the mapper/lowerings are exact source extractions, and OS AT/browser DOM application/GPU behavior were not executed.

Q317 records a blind GPU invalidation assertion: the test checks external_image_key(local 807), while the registry stages a separately allocated physical renderer key. Q318 records obsolete headed negative controls that clear shared.render_core but leave shared.render_recovery available, which the current host consults first. Q319 records Windows idle predicate drift: minimized windows skip consuming their wanted redraw, yet the owed predicate still selects ControlFlow::Poll for that flag. These are source-verified test/scheduling defects, not newly run ignored GPU/headed tests or measured OS CPU usage.

The [checkpoint](../../../support/receipts/authority-success-audit/native_accessibility_review_receipt.json) binds all source and probe evidence. The first fixture failed E0046 because its test leaf omitted paint_dirty; that failed receipt is preserved with zero tests. A distinct v2 completes only the fixture trait and reports four passed, zero failed. Generated outputs were recorded and removed. Cached supplier source confirms accesskit_tree_with_style itself lowers the common generic semantic projection; the problem is the separate final Mere postprocessing, not an invented second generic ARIA engine.

A813-A823 track shared semantic base versus divergent correction/naming, action-map custody, repeated marked-textbox pruning, duplicated producer-ID hashing, attempted-versus-installed state, stale receipt controls, platform scheduling predicates, repeated key/IME conversion, the shared fade clock with an empty retained scrollbar paint endpoint, composition experiments and native integration-test scope. E1a consumer tests and E1b manual walks were read, not executed. Shared WIP/source hashes and index are unchanged; earlier audit text is an exact prefix. No isolated target, Cargo home or worktree was created. Continue with Backstep and all remaining unread source.


### Shared core primitives: save checkpoints, count metadata and stale routes, 2026-10-17

Thirty-seven further files / 209,845 characters are fully read: Backstep (2 / 9,759), command-menu (4 / 26,211), Meristem (31 / 173,875). Overall coverage is 2,217/3,731 files and 26,253,798/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. Inventory hashes match, no read chunk is pending, and full source review remains active and incomplete.

Q320 reproduces a saved-history failure: save does not end coalescing, so a subsequent edit stays falsely clean and undo skips the saved content. Current Scenomise save callers use that shared API; the editor UI/persistence were not executed. Q321 reproduces cap reduction failing to bound an already populated History; no live runtime configuration consumer is established. Eight original Backstep tests and two public API counterexamples pass using the standard library only.

Q322 proves Count::combine skips index zero and reads past all nonempty arrays. Runtime cases panic and a separate public sequence constant consumer produces expected E0080. Q323 proves Vec generations are truncated on partial regrowth: the 3->1->2->3 path goes stale, then panics, then accepts the old child's route for its replacement. Three focused current Meristem core checks pass using the original element/splice fixture. All 37 original integration tests and 9 core unit tests also pass, showing that their positive receipt does not cover these failures. Meristem itself is freshly compiled over two exact retained supplier libraries; optional features, doctests and native/application dispatch are outside this receipt.

A824-A831 track history versus caller save boundaries, command catalogue versus product execution, generic retained state versus real backend elements, repeated generation recipes, Rc/Arc policy duplication, lazy count metadata, trusted message-path contracts and original test limits. A selected disabled menu entry is expressly permitted by the shared API; product execution guards still need tracing. One Option stale test delivers through its earlier None view after building the final Some view, so its source does not establish final-view delivery coverage.

The [checkpoint](../../../support/receipts/authority-success-audit/core_primitives_review_receipt.json) binds source hashes and all three diagnostic receipts, including the expected-failure constant compiler log. Generated outputs were recorded and removed. Prior audit prose remains an exact prefix; protected source, shared WIP and index are unchanged. No production edit, new Cargo target/home or worktree. Continue with remaining Cambium scaffolding, browser smoke, Mesquite, Sprigging and all unread source.


### Capture lane and leaf cache: real shared paths, replacement failures and success limits, 2026-10-17

Thirty further files / 263,737 characters are fully read: Mesquite (15 / 146,285), Sprigging (9 / 97,113), standalone browser smoke (4 / 19,179), and two nested Cambium scaffolding files. Overall source coverage is 2,247/3,731 files and 26,517,535/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. All current inventory hashes match and no chunk is pending. Full source review remains active and incomplete.

Q324 reproduces cache lifetime drift: replacing a scene leaf with Swatch keeps the retired scene and discards the new rect commands; removing/reinserting an empty same-size SceneSlot skips painting and retains old content. Q325 reproduces an unrestricted node_radius change moving the current node hit position by ten units on both axes while retained paint remains unchanged. Three actual current Sprigging registry/cache counterexamples and all 29 original unit tests pass over a freshly compiled current crate with three exact retained suppliers. GPU rasterization and a live product mutation/replacement path were not run.

Q326 traces a surface-family bypass: wheel parsing keeps surface:NAME, but execution calls raw selector_target rather than the scope-enforcing resolver and accepts the local Hit.point. Inspected cached Taproot code confirms the DOM matcher does not own host scope, while its ordinary resolve/click methods do enforce it. This is source evidence; Mesquite/native scenarios were not built or run here. Q327 verifies the unchanged browser smoke error hooks leave SMOKE PASS after both later console and window errors, though they still log. Three JavaScript hook cases run in Node VM fixture storage; this is not browser/GPU fault observation.

A832-A848 distinguish the genuinely shared text/JSON Lane lifecycle, presentation admission and clock deadlines, paired versus legacy state sampling, shared checkpoints, intentionally independent pixel-mask oracle, copied wheel selector grammar, completion/export ordering, qualified CPU cost reports, leaf incarnation/dirty custody, GraphGlyph's real alias, inert leaf input placeholders, caller-owned virtual scroll, isolated smoke host and nested former-project CI. The actual Graphshell command executor rechecks disabled_reason, closing that prior follow-up without inventing a disabled-command bug. Pixel saved-file/alpha assumptions, serialized context generation and post-completion export ordering remain named follow-ups, with consumer proof still needed.

The [checkpoint](../../../support/receipts/authority-success-audit/capture_leaf_review_receipt.json) binds hashes, exact supplier snippets and diagnostics. All generated output was recorded and removed. Previous prose remains an exact prefix; production/test/shared-WIP hashes and index are unchanged. No production changes, target/home/worktree isolation or broad native/browser acceptance. Continue Tabard workshop, remaining Cambium source and every other unread file.


### Cambium source completed: workshop policy reset and defeated graph retention, 2026-10-17

Thirty-eight further files / 424,098 characters are fully read: Tabard workshop (33 / 356,732), Tinct (4 / 54,087), and the Pelt workshop caller (1 / 13,279). Every inventoried file under crates/cambium is now source-read: 434 files / 5,311,014 characters. Overall coverage is 2,285/3,731 files and 26,941,633/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. All current inventory hashes match and no read is pending. Full repository review remains active and incomplete.

Q328 traces a real host-policy reset: Reopen library loads a fresh WorkshopState and replaces the whole value, clearing the protected export file/directory vectors. Pelt installs its appearance-store protection once at editor open, while shared native synchronization does not restore it. Later export can therefore offer ordinary replacement of that formerly categorically protected path. Existing targets still need explicit replacement confirmation; no automatic or actual user-file overwrite was executed. Q329 traces unconditional fresh dirty graph insertion from the native frame hook. Every eligible paint pass repaints that unchanged visible graph instead of taking the retained clean gate; this is not a measured GPU/latency result or proof of continuous redraw scheduling.

All 23 unchanged current Tinct unit cases pass from a fresh direct source build over one exact retained serde artifact. The first selection preflight found two serde variants and started no compiler/tests; distinct v2 selects the rc-enabled variant. The failed preflight and later successful logs are retained separately. Generated EXE/PDB outputs were recorded and removed. Full Workshop/native/OS/GPU tests were read but not built or run; the available Graphshell retained graph lacks several required component artifacts.

A849-A865 distinguish one reusable workshop model, authored reload versus host policy, saved-choice checks, snapshotted export effects, local path-normalization rules, real isolated preview documents, declared typed fallback versus arbitrary CSS, retained producer/source identity, infallible scene fallback on render error, application versus theme typography owners, different dirty concepts, legacy versus explicit Tinct profiles, two independent contrast algorithms, repeated transfer math/external JS mirror claims, qualified native test scopes, the narrow 23-test receipt and Pelt's completion/export ordering. Current source names the resource and native acceptance limits rather than inventing whole-stack proof. The earlier Q327 source pointer was corrected in mutable review notes from nonexistent line 399 to verified queue.present line 361; its immutable diagnostic remains unchanged.

The [checkpoint](../../../support/receipts/authority-success-audit/workshop_tinct_review_receipt.json) binds source traces, test receipts and hashes. Earlier prose is preserved as an exact prefix; shared WIP, protected source/test files and index are unchanged. No production changes, isolated target/home or worktree. Continue Distillery and all remaining unread source.


### Observation custody and RDF import boundaries, 2026-10-17

Eighteen further files / 267,687 characters are fully read: Alembic (3 / 19,242), Apparatus (5 / 50,216), and ten of linked-data's twenty source files (198,229 of 430,036 characters). Ingest, exact apply, parser evidence, classic JSON-LD bridging and their test sources are read; the main projection/export/query/metadata/statement code remains. Overall coverage is 2,303/3,731 files and 27,209,320/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. Current hashes match and no chunk is pending. Full source review remains active and incomplete.

Q330 records the bounded eviction-ladder contract error: arbitrary KeepDays values below seven advance to thirty rather than the promised next seven-day rung. Q331 reproduces remote-context discovery omitting nested @context URLs inside inline term definitions, both directly and inside arrays. Four helper cases execute exact unchanged source; the policy enum is matching fixture storage and JSON scanning uses an exact retained serde_json artifact. This is not a full recall/kernel build, JSON-LD expansion or network-acquisition run, and no live non-rung policy/acquisition consumer was established.

Apparatus builds freshly in std-only and projection+serde configurations: 18 and 22 original test executions pass, with core cases repeated between configurations. Its store distinguishes typed rejection from invalid-clock/sequence failure, keeps independent readers/gaps, enforces accounting/retention limits and states that producer payload accounting/redaction remain caller-owned. It does not infer operation success from diagnostic observation. The original empty inspector skeleton remains alongside actual batch inspection and is honestly labeled a placeholder. All generated outputs were recorded and removed.

A866-A882 track Alembic's implemented recall versus unimplemented workshop, canonical tags versus supplied visit maps, timing boundaries, diagnostic admission/reader/reset/inspection custody, leftover skeleton, evidence-carrying import versus legacy DTO apply, cautious classic-reifier promotion, repeated metadata/scope/kind recipes, exact snapshot-based kernel admission, ignored aggregate refusal results, dual tag-profile recognition, asymmetric literal metadata and narrow test evidence. Hairier concerns are documented for completion of source review: per-carried-edge whole-graph snapshot validation cost, aggregate refusal/partial writes, tag-attribution scope, submillisecond negative timestamp truncation and source/profile fidelity. They are not promoted to executed full-import findings without the relevant owner/consumer proof.

The [checkpoint](../../../support/receipts/authority-success-audit/observation_ingest_review_receipt.json) binds all source/test evidence. Prior prose remains an exact prefix; protected source, concurrent WIP and index are unchanged. No production edit, isolated Cargo target/home or worktree. Continue linked-data src/lib.rs and every remaining unread source.


### RDF projection completeness and engine adapters, 2026-10-17

Twenty-five further files / 364,123 characters are fully read at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. Linked-data is now complete (20 files / 430,036 characters), along with Incipit (2 / 11,372), Graft (4 / 41,060), Scrying (5 / 63,083), and Knot editor host (4 / 16,801). Overall coverage is 2,328/3,731 files and 27,573,443/39,427,411 characters. Current source hashes match; no chunk is pending. The full repository review remains active and incomplete.

Q332 records a source-verified completeness defect: direct RDF facts traverse every parallel Surface bucket, but held metadata repeatedly selects only the first edge to each outgoing neighbor. Other held buckets lose assertion reifiers, provenance, lifecycle and basis metadata. The kernel explicitly documents the first-edge selection and provides the all-buckets API already used by direct facts. JSON-LD, file export and both borrowed/materialized query oracles share this projection, so parity between those paths does not prove completeness. The existing parallel-bucket test checks base facts and direction, not all source handles. Resource-edge projection is separate and is not accused by this finding.

Q333 records another source-verified adapter defect: Scrying maps all Pressed/Released mouse inputs to LeftButtonDown/LeftButtonUp, while using the changed button only for virtual-key flags. Right/middle/back/forward therefore retain the wrong event kind. The adapter advertises mouse support and forwards this mapping; its source test exercises left clicks only. Neither finding is labeled an executed full RDF or native-input replay. Exact excerpts and current kernel API contracts are retained in [source traces](../../../support/receipts/authority-success-audit/projection_adapter_source_traces.json).

Fresh unchanged original Incipit tests (5) and Graft adapter tests (6) pass over exact retained supplier artifacts. Graft uses explicit stub/witness surfaces; no live Servo, system-WebView, GPU import or headed result is claimed. An initial preflight refused two serde feature rows before compiling anything; its script/hash/failure is retained, and the distinct v2 runner selects the rc-enabled row. Generated executables/debug outputs were marker-checked and removed.

A883-A898 record the independent Surface metadata walker, shared-owner test oracles, canonical Resource metadata, shared reversible reifier codec, broad vocabulary-graph filtering, stale raw/type deferrals, pending-input custody, display versus identity query results, opaque shelfmark custody, Graft host seam, adapter-local Scrying limits and native-frame ownership, dual navigation translators, source-buffer/save-checkpoint ownership and host lexer overrides. Hairier follow-ups remain explicit: reserved RDF namespace admission versus silently dropping foreign quads, exact import's whole-snapshot cost/refusal reporting, attribution/literal fidelity, SourceChanged's unreachable Committed fallback, preferred CPU transport versus unsupported frame mapping, and global pending retry counts. These are documented without taking a broad redesign detour.

The [checkpoint](../../../support/receipts/authority-success-audit/projection_adapter_review_receipt.json) preserves all prior prose as an exact prefix and protects current reviewed source, shared WIP and index. No production edits, new worktree, Cargo home or isolated target. Continue the unread Weld adapter and remaining source tree.


### Weld capability claims and derived search custody, 2026-10-17

Sixteen further files / 199,524 characters are fully read: Weld (5 / 78,807) and Eidetic Search (11 / 120,717), including its rehearsal example and ignored measurement harnesses. Overall coverage is 2,344/3,731 files and 27,772,967/39,427,411 characters at unchanged HEAD 95515a1313785513efb38a433412941da7b68ed1. Hashes match; no chunk remains pending. Full source review remains active and incomplete.

Q334 is a source/trait-contract mismatch: Welding declares typed page zoom Supported, while WeldProducer inherits Inker's Unsupported set_page_zoom. Legacy apply_settings zoom is a separate route and explicitly does not satisfy the typed command contract. This feature was read without building its CEF supplier graph or claiming a headed zoom result.

Q335-Q338 are reproduced through a freshly compiled entire current search library over exact retained suppliers. A matching public BM25 search with limit zero panics; normal higher-level adapters clamp their limits and hide that core edge. A custom-stemmed persisted trail projection writes a default current sidecar and reopens successfully with different runn/running answers. Persistent rebuilding recursively deletes an unrelated marker in the supplied directory, while transient rebuilding preserves it; only disposable marker-identified fixtures under the approved existing target were touched. The 257th BM25 field wraps to field zero and silently loses its configured weight; ordinary three-field adapters are not accused of that shape. These are scoped owner/API findings, not claims of a live product crash or actual user-data loss.

Four original BM25 and five original fusion tests pass alongside five integration observations,14 total. The first audit harness failed on an incorrect relative include before tests; a distinct v2 compiled but three fixture guards rejected Windows CRLF before exercising persistence, while11othercases passed. V3 changes the fixture marker to exact bytes and all14pass. Failed scripts/logs/receipts are retained, and all generated libraries, executables/debug outputs and disposable directories were marker-checked and removed.

A899-A913 track the opt-in concrete adapter versus host runtime custody, repeated Graft/Weld forwarding, intact native frame ownership, instance capability gates, richer versus legacy input, latest-query find correlation, shared scorer with duplicated adapter recipes, core bounds, tokenizer versus saved profile, cache ownership, format versus source freshness, prefix versus ranking semantics, genuinely shared fusion, rehearsal provenance and evidence limits. Larger follow-ups remain documented: cookie callback support, find correlation across queries, cache atomic publication, sidecar profile/freshness identity, example timestamp overflow and ESP provider/model/cardinality custody. Continue ESP's full source next.

The [checkpoint](../../../support/receipts/authority-success-audit/weld_search_review_receipt.json) and [search observations](../../../support/receipts/authority-success-audit/search_boundaries_probe_v3_receipt.json) bind current source and retained supplier hashes. Prior prose remains an exact prefix; reviewed source, concurrent WIP and index are unchanged. No production edits, new worktree, isolated Cargo home or target.


### ESP embedding contracts and numerical authority, 2026-10-17

Thirty further source files / 302,676 characters are fully read. ESP's embedding subtree is complete (26 files / 270,484 characters), including BERT, the batched Burn index, typed persistence and tests. The package manifest, two inference import/qualification examples and decoder attention are also read. ESP as a whole remains32/74files and337,621/854,835characters because actor/LoRA had prior coverage and the remaining inference sources are still unread. The e5 meaning-model JSON was read/hash-bound as auxiliary runtime authority without adding it to Rust/source inventory counts. Overall coverage is 2,374/3,731files and28,075,643/39,427,411characters at unchanged HEAD95515a1313785513efb38a433412941da7b68ed1; hashes match and no chunk is pending. Full review remains active and incomplete.

Q339 traces BERT's discarded padding mask through the shared provider/model/attention/pooling route. BatchLongest produces masks, but the model receives only IDs; padding therefore participates in attention and mean pooling. Reference tests embed one text at a time and the advertised always-on reference comparison is ignored, so they do not qualify mixed-length batch behavior. Q346 records hidden_act accepted from config while GELU is always executed. Both findings are source-verified; no real or synthetic BERT numerical/model-loading run is claimed. The mask repair spans model boundaries and is documented as a larger high-effect issue rather than prolonging this full-source pass.

Q340/Q341 document actual unshared policy behind the batched index's drop-in claim: HashMap order replaces the CPU's key tie-break, and norm+1e-12 replaces the CPU's degenerate threshold. Source counterexamples include tied vectors and finite1e-8vectors. No Burn/GPU execution is claimed. Q342 reproduces accepted metric disagreement in dense and sparse SemanticSearch constructors, causing nearest and scores/metric to disagree. Q343 reproduces serde bypassing vector admission: invalid sparse indices deserialize then panic when densified, while malformed dense dimensions yield plausible zip-truncated scores. Q344 executes a cycle in the exact rank comparator with admitted NaN; it does not claim an actual sort panic. Q345 reproduces a link despite top_k0 and excess neighbors when self is outside the requested head.

Seven unchanged portable embedding modules compile in their original namespace over exact retained serde/json suppliers. Seventy original tests and seven focused audit checks pass,77total. Six checks use public APIs and one directly calls the original crate-private comparator. All generated executables/debug outputs were recorded and marker-checked before removal. This does not qualify whole ESP inference, actor, model-session, BERT/Burn, a live store/corpus, GPU or product behavior.

A914-A934 track stale loader/stub/validation prose, duplicated tensor-name/shape recipes, mask custody, intrusive diagnostic traces, actual dense/sparse sharing, duplicated accelerator/affinity policy, provider/index metric custody, explicit provider cardinality contracts, sparse capability selection, ESP's separate lexical tokenizer, typed integrity versus model-space provenance, unresolved serializer canonicalization, the specific meaning-model pin, decoder attention/cache recovery and inference example gates. The earlier zip concern is now qualified by the provider's explicit one-output-per-input contract; no conforming short-batch provider was found. Remaining source review must trace model/session/library admission and decoder error recovery before promoting broader claims.

The [checkpoint](../../../support/receipts/authority-success-audit/embedding_source_review_receipt.json), [portable checks](../../../support/receipts/authority-success-audit/embedding_core_probe_receipt.json) and [source traces](../../../support/receipts/authority-success-audit/embedding_source_traces.json) bind evidence. Prior prose remains an exact prefix; reviewed source, concurrent WIP and index are unchanged. No production edit, new worktree, isolated Cargo home or target. Continue decoder/checkpoint.rs and every remaining unread file.


### ESP decoder/session source completion and current upstream, 2026-10-17

ESP source review is complete:74files /854,835characters. This pass adds42fullyread files /517,214characters, including all remaining decoder, model-library/session/template, integration-test and Python reference source. The decoder subtree is19files /297,958characters. Both literal prompt-template JSON files were also read/hash-bound as auxiliary runtime authority, alongside the previously read meaning manifest, outside the source denominator.

The audit fetched origin/main and found two new commits,4490ae16 and8c299283. They were merged into main at609b0a60373bdb54cd899254397bb061096a3221; all37shared dirty/untracked files were preserved byte-for-byte and the index has no staged audit edits. The latest fetched origin/main is an ancestor,11local commits ahead and0behind. Imported changes adopt retained semantic projections, explicit estimated-fixture APIs, shared Arc image buffers, checked surface reconfiguration and fresh display-handoff counts. Their published validation receipts were read as upstream claims; no local whole-workspace/native/wasm/GPU run was substituted for them.

The merge invalidated44previously read files. Current valid coverage is therefore2,372/3,731files and27,543,120/39,433,387characters, lower than the pre-merge count despite new reading. Full-file rereads, starting Cargo.toml, are queued explicitly. Diff inspection does not restore whole-file credit. Historical completed areas retain their old revision receipts and affected areas are marked pending current reread. The full-source goal remains active and incomplete.

Q347 reproduces seed-independent last-token selection for tied finite logits at an admitted subnormal positive sampling temperature. The initially suspected unique-maximum failure did not reproduce; its failed test remains retained and is excluded from this finding. Q348 executes the original evaluator helper: an expected NaN logit ranks1, including an all-NaN row. The shared ranking_tally caller then counts it as passing for every positive cutoff without checking finiteness. Both trainer version receipts reuse that evaluator, so shared authority alone cannot establish truthful numerical success.

Q349 executes the exact StreamObserver and decode method with a real retained tokenizers0.20.4 ByteFallback/Fuse tokenizer. Byte tokens C3,A9,x decode first to replacement, then é, then éx. The observer emits the replacement immediately, holds every subsequent non-prefix decode forever and returns successful text containing only replacement. A Unicode stop is missed too. ASCII and split-stop controls pass. Regular generate uses this observer; durable generation separately full-decodes final completion, so identical final corruption is not claimed for that route. This is a high-effect owner repair documented without a model/GPU build detour.

Q350 uses two unchanged portable source modules through public generate_streaming_controlled. Cancelling at the first callback yields successful alpha with no delivered fragment; cancelling at the second yields alpha beta while the callback saw only alpha. Filtering the callback changes delivered text but the wrapper returns the underlying provider's unfiltered text. The actor separately emits Cancelled when its stop flag is set, so this does not claim an actor Finished error.

The decoder/helper probe passes15checks,9original plus6audit. The portable cancellation probe passes14checks,12original plus2audit. Four portable original tests occur in both binaries:29passing invocations across two binaries, not29distinct tests or a whole ESP build. Exact current source and retained suppliers are hashed. The v1 failed hypothesis and an initially too-narrow cleanup marker check are preserved; subsequent marker checks removed all generated executable/PDB/COFF/archive outputs. No model, full decoder/provider, trainer, Burn/session/store actor, native GPU or live corpus execution is claimed.

A935-A954 record actual versus duplicated tensor inventory, decoder admission, conversion sharing, checkpoint byte/memory scope, fresh-cache versus stage invalidation, two detokenization/completion routes, complete-checkpoint consistency, shared sampling and evaluator math, intentional trainer versions, partly duplicated fixtures, deferred library readiness, supplied qualification claims, inert template/session identity, cancellation ownership, oracle strength, tie-aware lexical/fusion receipts, mirrored GPU selection and upstream adoption. Ordinary generation drops failed fresh caches, narrowing the earlier attention recovery concern. Bigger numeric/trainer/cache/checkpoint/qualification issues remain documented followups rather than open-ended implementation detours.

Evidence: [source checkpoint](../../../support/receipts/authority-success-audit/decoder_source_review_receipt.json), [decoder checks](../../../support/receipts/authority-success-audit/decoder_text_math_probe_v2_receipt.json), [cancellation checks](../../../support/receipts/authority-success-audit/infer_cancellation_probe_receipt.json), [upstream transition](../../../support/receipts/authority-success-audit/presentation_main_sync_after.json). Prior document bytes remain an exact prefix. No audit-authored production edit, retained isolated target, Cargo home or worktree. Reuse remains C:/t/cargo-targets/mere.


### Current upstream Cambium reread, 2026-10-17

Twenty changed files /314,824characters were fully emitted, reviewed and acknowledged at unchanged HEAD609b0a60373bdb54cd899254397bb061096a3221. This restores all434Cambium files /5,313,101characters on the current revision, including the retained winit harness, Rootstock presentation, Mesquite and the Workshop updates. The root workspace manifest is also fully reread. Overall current coverage is2,392/3,731files and27,857,944/39,433,387characters. Twenty-four upstream-changed old files still await current full rereads; next is Pictograph canvas.rs. The full source goal remains active and incomplete.

The checked reconfiguration path has an honest no-frame result. WgpuPresentation returns Ok(None); both ordinary-native and installed-driver hosts return before advancing the presentation sequence, observing state or completing capture. WgpuDriver stamps only a real Some(output). The two host routes share the same raster/acquire/present owner. These are exact current source traces, not a headed presentation/recovery run. Native renderer access still differs from the browser frame gate, so portable try_renderer prose must not imply every native access itself checks device loss.

Direct harness resolution now builds Rootstock's retained accessibility projection and uses its semantic matcher before reading current painted bounds. The scenario tests explicitly enforce retained aria-labelledby precedence and reject stale/raw role labels while preserving class-selector text semantics. Estimated fixture APIs are named explicitly and used chiefly for test node lookup; this does not turn their names into retained layout or accessibility proof. Current Workshop native header tests resolve against the retained projection.

A955-A963 track additional authority distinctions. The root manifest's claim that only it knows internal paths is false: host/Mesquite members repeat matching path/version declarations. Their current values agree, so duplicate runtime source identity is not alleged. Current Cargo.lock has one Genet7f13b85fsource and one Netrender e8cda06esource; long comments still describe former standalone/git homes. Lock/source prose is not a fresh compiled dependency graph.

Mesquite's key/input-text verbs dispatch straight to the runner; Harness::press_key goes through Host::key, including application interception, spatial focus, zoom and visual caret work. A successful runner scenario therefore does not qualify all native host keyboard behavior. Workshop inline-style+layout checks, computed-style checks and presented-pixel proof have different scopes; source assertions must preserve those distinctions. Existing Q326 wheel-surface scope and Q328/Q329 workshop policy/cache issues are unchanged by selector-only updates and retain their earlier evidence limits. No new bug or runtime test is claimed in this reread pass.

The [reread checkpoint](../../../support/receipts/authority-success-audit/upstream_cambium_reread_receipt.json) and [source traces](../../../support/receipts/authority-success-audit/upstream_cambium_source_traces.json) bind these observations. All inventory hashes, protected sources, three auxiliary runtime JSON artifacts, prior evidence and shared WIP remain unchanged; prior document bytes remain an exact prefix. Index unchanged, diff check passes. No new/retained isolated target, Cargo home, worktree or generated build output.


### Current Pictograph image-sharing reread, 2026-10-17

Four changed files /127,605characters are fully reread at unchanged HEAD609b0a60373bdb54cd899254397bb061096a3221. All165Pictograph source files /2,249,061characters now have current matching-hash coverage. Overall valid coverage is2,396/3,731files and27,985,549/39,433,387characters. Twenty upstream-changed files remain for rereading, followed by ordinary unread source. The full audit remains active and incomplete.

A964-A967 separate actual shared image storage from broader memory claims. The decoded favicon cache converts owned pixels to Arc and face_layer clones that handle into the frame sidecar, replacing the earlier byte copy. Scene sprites now share their allocations too, but their persistent registry has a distinct lifetime and policy from the digest-keyed LRU. Host storage still owns durable bytes and decoding; a paint miss queues an ImageRef rather than fetching in the frame loop.

Four fresh checks compile the unchanged std-only cache module: two original LRU/oversize tests plus two ownership observations. Two frame handles share one allocation. Eviction to a zero-byte cache leaves a retained frame handle valid and cache usage0, so the counter is a current-entry working-set count, not all live image memory. This is expected ownership, not a leak or process-memory acceptance claim. No whole Canvas, external paint/compositor/dependency graph, image store, native renderer or GPU test was run. All generated executable/debug outputs were marker-checked, hashed and removed.

Rejected image retry policy remains a caller-trace followup: requested_images intentionally avoids repeated missing/undecodable I/O, and an oversize rejection does not itself invalidate that request marker. No permanent product image failure is asserted without tracing host invalidation/re-registration. Existing stale doc attachment/line-ceiling prose is recorded separately from functional behavior. No new bug finding is promoted in this pass.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/pictograph_shared_image_reread_receipt.json) and [fresh cache checks](../../../support/receipts/authority-success-audit/shared_image_cache_probe_receipt.json). Protected sources, auxiliary runtime artifacts, prior evidence, shared WIP and index are preserved; prior document bytes remain an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or new build output. Continue graphshell-client's changed manifest and frozen realization.


### Frozen reading and retained document ownership reread, 2026-10-17

Six upstream-changed files /210,113characters have been fully reread at HEAD 609b0a60373bdb54cd899254397bb061096a3221. Current coverage is 2,402/3,731files and 28,195,662/39,433,387characters. Graphshell-client and document-canvas now have complete matching-hash inventory coverage again. Fourteen changed files remain for whole-file rereading, followed by all ordinary unread source. The full audit remains active and incomplete.

Q351 reproduces a content mismatch behind the frozen paths' same-answer claim. Supplied flat instance detail survives rows and HTML but disappears from AccessKit labels/descriptions. Folded member detail remains in the frozen data yet disappears from every output route, including the HTML disclosure and alternate table. Member names remain present. Shared FoldReading membership is real; independently rebuilt output content is the failing seam. A968-A971 record that distinction alongside estimated DOM/test limits and the actual shared font, decoded image and portable-document ownership.

Three fresh public API cases pass against the complete current graphshell-client production library with accesskit enabled and seven exact retained suppliers. They reproduce returned-data loss and include a passing flat HTML/rows control. They do not run the original unit suite, fresh Cargo dependency graph, Tokio pump, browser DOM, OS screen reader, compositor or native GPU path. Generated library/executable/debug outputs were hashed and removed. Document-canvas and Smolweb tests were read, not executed in this pass.

The document lane shares source via Arc while keeping appearance layout/state local; decoded pixel sidecars now clone Arc handles. Concrete shaped font bytes remain the font owner. Earlier Q154 parser-selection and Q155 shallow nested-image discovery remain source-visible and were not newly rerun. Its presented flag is set after scene construction, so that local state alone cannot establish a completed surface presentation.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/frozen_document_reread_receipt.json) and [public API diagnostic](../../../support/receipts/authority-success-audit/frozen_detail_probe_receipt.json). No production edits. Protected source hashes, auxiliary artifacts, prior evidence, index and shared WIP are preserved; the prior document is an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or new build output. Next: Dramatis manifest and interaction source, then remaining changed Graphshell files.


### Current retained-host source reread, 2026-10-17

The fourteen remaining upstream-changed files /400,889characters are fully reread at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221. All prior source coverage is current again: 2,416/3,731files and 28,596,551/39,433,387characters. Graphshell port coverage is all529inventory files /4,261,985characters; Dramatis is all10 /56,730characters. The full audit remains active and incomplete, with1,315ordinary unread files remaining.

A972-A980 record actual query sharing and remaining separate authorities. Semantic role/name scenario actions now consume the retained accessibility projection and refuse a missing required projection. Authored class lookup remains a DOM identity operation. Focused text routing uses the already-focused node and its authored label/value marker; it does not need another estimated semantic query. Dramatis explicitly names estimated fixture lookups and keeps retained host interaction claims bounded.

Earlier Q255 command-route divergence, Q256 initial presentation-store owner lifetime and Q258 touch cancellation-as-release remain visible in the current complete source. No fresh runtime reproduction is claimed. Coupling edits still bind exact source/coupling and acknowledge product results. Rule persistence uses finish_save before reporting saved. Query and promoted-link writes implement separate confirmation recipes; promoted freeze retry reconciliation and dynamic field-label collisions remain caller-trace followups, not newly promoted bugs.

Browser semantic-tree JSON is instrument output, not proof of OS assistive delivery. Its missing/default counts cannot alone prove an absence assertion. Boot data-ready is mount readiness; presentation sequence is published only when a host stamp is available. The standalone web manifest necessarily restates workspace dependency policy, making configuration drift a real maintenance surface without proving two loaded runtimes. Module prose still describes the early control-free tree phase although current code composes the full product editors.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/retained_host_reread_receipt.json). This pass reads source only and executes no new Cargo, interaction, browser, native, renderer or GPU test. No production edits. Prior evidence, protected sources, auxiliary artifacts, shared WIP and Git index are preserved, and all previous document bytes remain an exact prefix. Diff check passes with line-ending warnings captured separately from errors. No retained isolated target, Cargo home, worktree or new build output. Next: unread mora-cmudict, reference-data and Moot sources.


### Lexical providers and Commons source review, 2026-10-17

Thirteen newly read source files /301,789characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221 raise current valid coverage to 2,429/3,731files and 28,898,340/39,433,387characters. Mora-cmudict and reference-data are complete. Six Commons files are fully read, including the entire call, chat, encrypted-graph and refreshable-key code plus their embedded tests. Commons root graph profile, parking and pruning remain unread; the full repository audit is active and incomplete.

Q352 is source-confirmed: EncryptedReplica::edit captures a key snapshot, loads/builds/seals against it, then self.accept captures another. Admission can park and return Ok(false), but edit discards that bool and returns the operation as success. This contradicts the one-authoring-turn key snapshot promise and stored-edit wording. A public edit callback can replace the shared GroupKeys handle between the reads. That deterministic public API reproduction remains pending; no fresh Commons compilation or observed user data loss is claimed. The contained repair candidate is one captured keyring through local seal/admission with an explicit insertion outcome.

A981-A990 separate actual sharing from broader claims. Mora owns phones; CMUdict is a distinct data/spelling provider. ValidatedPack privately owns its invariant-checked form, while SourceRegistry owns inert catalog, disabled installation, explicit enablement, exact source lookup and publication. Integrity digests do not provide source authenticity. Encrypted Commons uses the root batch/frontier/projection functions behind a distinct wire marker/lane. Chat deliberately separates retained structural admission from communal authority projection. Its joined checkpoint authority is captured at join while encryption keys refresh; host lifecycle policy remains a followup. Reusing one replay load does not by itself serialize cloned store writers. The RF fixture verifies bytes/signature/plaintext, with physical carriage requiring separate evidence.

Twenty-five unchanged original lexical tests freshly pass:23reference-data and2mora-cmudict, including the bundled dictionary parse, registry state/reopen, corruption refusal, failed publication, byte bounds and advisory lock release on child process exit. The ignored subprocess helper runs through its parent test. Compilation uses exact retained serde/serde_json/blake3/mora suppliers; no fresh Cargo graph, Unix symlink/directory-fsync, power-cut, import UI, Commons lane, radio or media run. Original Commons tests were fully read where emitted, not executed. Generated binaries/debug files and owned temporary fixtures were hashed/checked and removed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/lexical_commons_review_receipt.json) and [original lexical tests](../../../support/receipts/authority-success-audit/lexical_original_tests_receipt.json). The bilingual JSON fixture was read; the bundled dictionary was hash-bound and exercised, not given manual source-read credit. No production edits. Protected sources, auxiliary/runtime data, prior evidence, shared WIP and Git index remain preserved; prior document bytes are an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or new build output. Next: Commons root graph, parking and pruning.


### Commons source completion and concurrent parking bounds, 2026-10-17

Three new complete files /106,997characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221 complete all9Commons inventory files /341,386characters. Current repository coverage is2,432/3,731files and29,005,337/39,433,387characters, with no stale or pending read credit. The full audit remains active and incomplete.

Q353 is reproduced against the unchanged complete Parking module over exact retained suppliers and real MemoryBackend storage. Two valid signed arrivals both read an empty slot list before either batch commits. Both return true; max_records1 ends at parked2/evicted0. A byte cap admitting either record alone also ends exceeded by their actual sum. Sequential count-bound control passes. The wrapper schedules reads only and delegates every actual atomic write batch unchanged. This proves the occupancy decision needs one backend-owned transaction across writers; atomic apply alone does not close the earlier read gap. No whole Commons/live LogSync, physical carrier, disk or large-memory execution is claimed.

The v1 harness compiled but did not link because native Windows search paths were omitted. Cleanup conservatively stopped on owned COFF object suffixes; all six outputs were later identity/marker/hash checked and removed. V2 ran the count control and counterexample successfully but its equal-record-length premise failed before the byte race. CBOR records legitimately differ in encoded size. V3 preserves those attempts, sets the cap to the larger individual record, and passes all3auditcases. All generated outputs are removed.

A991-A999 record real graph batch/frontier/remove-wins sharing, duplicated chat-versus-graph attestation checks, different adapter revocation precedence, permissive compatibility materialization, parking's missing read/write transaction, shared atomic re-admission insertion, caller-owned pruning coverage, test persistence limits and the wider Commons dependency cone. The pruning helper only serializes/deserializes GroupSession in memory; its persist wording is not durable session evidence. Missing lane reports and snapshot freshness remain host obligations, not verified by this helper.

Q352 remains source-confirmed. Its deterministic public key-replacement cases and a whole current library runner are prepared but not run. A bounded retained-supplier selector could not match the selected Gemot family's named Insigne fingerprint to a current retained artifact. This is a limited selection failure, not a current Cargo build failure or a claim that every possible supplier family is unavailable. A broad dependency rebuild was not substituted for source review. No authoring test pass or live key-rotation failure is claimed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/commons_complete_review_receipt.json), [parking counterexamples](../../../support/receipts/authority-success-audit/commons_parking_probe_v3_receipt.json), [first failed attempt](../../../support/receipts/authority-success-audit/commons_parking_v1_failure_receipt.json), [second attempt](../../../support/receipts/authority-success-audit/commons_parking_probe_v2_receipt.json) and [authoring supplier qualification](../../../support/receipts/authority-success-audit/commons_authoring_supplier_gap_receipt.json). No production edits. Protected source/runtime/auxiliary hashes, prior evidence, shared WIP and Git index remain intact; prior document bytes are an exact prefix. Diff check passes. No live diagnostic process, retained isolated target, Cargo home, worktree or new generated build output. Next: unread Gemot/Moot sources and host authority consumers.


### Gemot examples and late standalone host returns, 2026-10-17

Read21new complete inventoried files /243,811characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221. This covers the manifest, all current Gemot examples and their build/check/guest files, crate/artifact introductions and the constitutional event owner. Gemot remains partial at21/51files /243,811of777,720characters. Repository coverage is2,453/3,731files /29,249,148/39,433,387characters, with no stale or pending read credit. The full audit remains active and incomplete.

Q354 reproduces a late navigation publication in the standalone capsule page. The host checks navigate permission before beginning openAddress, then awaits storage/pack/body work. Its final token check only guards newer opens. Revocation and completed Stop do not invalidate that token. A delayed body therefore updates current reader state, last_opened and the verified-success notice after either boundary. Stop really executes the unchanged shared Runtime.close and terminates its Worker double; the pending host fetch is outside that worker's pending-call owner.

Three fresh Node VM cases execute unchanged complete browser.mjs plus the shared Runtime: ordinary open control, navigate revoked while the body is deferred, and Stop completed while the body is deferred. DOM, honest Worker replies, absent-only IndexedDB, networking and signature/part verification are explicit doubles. This is host-continuation evidence, not actual Chromium/Wasm, cryptographic authenticity, real IDB, carrier or headed acceptance. It does not establish a bug in the separate Graphshell host. Its checker source explicitly covers delayed turn/body revocation and stale generations; the standalone checker revokes only after opening has completed and misses Q354.

A1000-A1011 record actual shared runtime/verification ownership, source assembly versus transitive dependencies, proof-local publication limitations, contribution versus local applet authority, the unshared host-lifetime guard, distinct checker coverage, existing signed-pack verification, overly broad native interruption evidence, supplied-Wasm provenance limits, raw-child readiness/cleanup, current constitutional grants and screenshot scope. The build actually copies shared Graphshell runtime/worker/WASI/verification source; there are no example-local fork copies of those four modules in the current inventory. The packaging-time sources map cannot alone prove that supplied prebuilt Wasm was compiled from those current sources.

Followups remain qualified: native runaway_interrupted accepts any error rather than a typed interruption cause; raw-child readiness/error exits lack the capsule example's ManagedChild cleanup; historical authority and aggregate import atomicity are explicitly open in the publication proof. No native, child, network or full Playwright proof was rerun here. Continue constitutional fold and actual authorization/storage/service consumers before claiming Gemot as a whole reviewed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/gemot_examples_review_receipt.json) and [late-return diagnostic](../../../support/receipts/authority-success-audit/capsule_late_return_probe_receipt.json). No production edits, Git index changes or isolated build output. Existing source/runtime/auxiliary and shared-WIP hashes remain intact; all prior document bytes remain an exact prefix. Diff check passes. The diagnostic is terminal; retained evidence remains under100MB. Next: unread Gemot constitutional fold.


### Gemot constitutional admission, delegation, membership and hosting, 2026-10-17

Read16new complete files /197,774characters plus25,978characters from the aggregate service at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221. Constitution7/7, delegation4/4 and group3/3 inventoried files are now fully read; FLORA and both hosting files are also read. Gemot is37/51files with the service still partial at25,978of104,697characters. Repository coverage is2,469/3,731files /29,472,900/39,433,387characters, with no stale or pending read credit. Full source review remains active and incomplete.

Q355 reproduces constitutional divergence under opposite arrival order. A valid founder genesis makes two distinct electors eligible at quorum threshold1. Each signs a different amendment of the same revision in its independent sequence-zero log. All signatures check, and the pure fold over the complete corpus deterministically selects the same law in either order. Real ConstitutionStore/MemoryBackend replicas instead accept different first arrivals, reject each other's valid proposal as StaleRevision before retaining it, and keep different laws and revisions. Repeated cross-delivery remains refused. This is not a same-signer log fork. Sequential founder amendments and retry after initially missing genesis pass as controls.

The diagnostic freshly compiles the unchanged complete event/fold/wire/store modules over ten exact retained suppliers. Real operation signatures, shared OperationProcessor and MemoryBackend execute; no fake admission or storage is substituted. This is not whole Gemot, a current Cargo graph, live LogSync, aggregate service, disk or headed acceptance. Initial retained-family selection failed before compilation because its identity fingerprint had no matching current marker/artifact pair. A targeted inventory found two other complete families; v2 used2a2dd68ccc68833a and passed all3auditcases. The initial miss is preserved. Generated executable/PDB/link objects are removed.

A1012-A1023 record the missing link between deterministic folding and actual admission convergence, live-apply versus corpus quorum scope, separate checkpoint-history reconstruction, actual delegation and membership retain/project ownership, duplicated stable-author binding checks, host-secret identifier limits, current production hosting versus the older proof-local grammar, hosting preparation/sign/serve boundaries, social FLORA declarations versus numerical execution, separate aggregate stores and signed-grant/provider intersections.

Current hosting.rs already provides the complete bounded HostingPromise grammar through signed Standing records. The older publication rehearsal's candidate HostingCommitmentV1 is a different proof-local implementation and does not exercise this owner. Preparation checks policy/authority/contribution and ceilings, commit rebuilds the preparation, and serving checks live promise/current authority/expiry/withdrawal/ceilings. Hosts explicitly serialize against ingestion across separate stores; content custody and observed availability belong elsewhere. This source review does not independently authenticate historical AuthorityCut completeness or execute durable hosting tests.

Higher-effect followups are documented rather than patched: quorum command Ok can mean one retained signature with unchanged law; checkpoint history folds sequence-sorted corpus prefixes; aggregate amend persists governance and separately switches the object policy despite atomic wording; automatic MootGroup facts default non-membership Standing fields. Trace the remaining service/authority consumers before promoting those source concerns to broader product failures. No constitutional conflict, GC or encryption-policy semantics are chosen here.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/gemot_constitution_review_receipt.json), [constitutional counterexample](../../../support/receipts/authority-success-audit/gemot_constitution_probe_v2_receipt.json) and [initial supplier miss](../../../support/receipts/authority-success-audit/gemot_constitution_v1_supplier_miss.json). No production edits. Protected source/runtime/auxiliary hashes, prior evidence, shared WIP and Git index remain intact; previous document bytes remain an exact prefix. Diff check passes. No live diagnostic process or retained isolated target/home/worktree. Evidence remains under100MB. Next: service.rs from character25,978, then remaining Gemot sources.


### Gemot source complete: Tulpa provenance and recognition authority, 2026-10-17

Finished14files, including the previously partial service, for310,157additional characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221. All51Gemot inventory files /777,720characters are fully read at matching hashes. Repository coverage is2,483/3,731files /29,783,057/39,433,387characters, with no stale/pending credit. The full source audit remains active and incomplete.

Q356 reproduces an internally inconsistent Tulpa adoption. Two accepted signed proposals bind the same Tulpa/version to different artifacts. The selected effective proposal/operation names the later artifact, but the returned adopted artifact is taken from the earlier lowest-proposal-id binding. The diagnostic retrieves and decodes the cited signed proposal to compare its actual artifact with the returned value. Both recognition decisions are accepted and all four facts remain. A distinct-version control returns a consistent citation/artifact pair.

Q357 reproduces the recognition provenance gap through the aggregate. A Moot is constitutionally founded under one identity but has zero membership records. Another signer supplies an electorate containing only itself, proposes an artifact and self-endorses. Both commands succeed and the aggregate snapshot reports adopted while still showing zero actual members and the original founder. The projection freezes and evaluates asserted recognition bytes without grounding them in retained community authority. This is not a demonstrated downstream executable activation or physical peer attack, and the fixture is explicitly zero-member. Selecting the correct governed/historical electorate is a larger design issue; raw facts should remain distinct from effective adoption.

Q358 is source-confirmed: configured availability/erasure bounds are copied into MootRetentionPolicy but never read by Gemot's production checkpoint/prune decisions. Implemented checks cover revision, single-signer authority, commitments and monotone frontier, while ForAgeMs/Forever/UntilCheckpoint do not select behavior here. No varied-bound/clock/physical-deletion case was run; this does not choose any GC/destruction semantics or prove external hosts lack their own enforcement.

The unchanged complete current Gemot production library and a public consumer were freshly compiled over15exact retained suppliers from one named Cargo fingerprint family. Four sequential CPU/MemoryBackend cases pass: distinct-version control, Q356, Q357 and pending quorum. The latter confirms aggregate amend returns Ok with unchanged accepted rules/revision when only one of two signatures is retained; its snapshot is honest, but command/prose status must distinguish a stored vote from accepted replacement law. This is not a fresh Cargo dependency graph or execution of original package, disk, live LogSync, headed, protected transport or tensor tests. Generated owner rlib/executable/PDB/link output is removed.

A1024-A1035 record actual lane-assembly ownership, raw versus authorized projections, causal collection citation, separate Tulpa resolution paths, asserted versus governed electorate, correct shared facet canonicalization, quorum result scope, inert retention configuration, shared evidence parser versus non-atomic import, intentionally different historical/live prune intake, sequential snapshot cuts and proof scope. The earlier facet suspicion is resolved: Servitor's canonical encoder turns logical facet segments into the expected slash hierarchy with segment escaping. The object wire uses shared stable_writer_subject; membership still has its local binding recipe. Protected-drop TestProtector is an XOR fixture, not confidentiality evidence. Source docs still contain stale M1/ownership milestone wording despite current capability and lane implementations.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/gemot_complete_review_receipt.json) and [whole-owner diagnostic](../../../support/receipts/authority-success-audit/gemot_owner_probe_receipt.json). No production edits or Git index changes. Prior evidence, protected source/runtime/auxiliary hashes and shared WIP remain intact; all earlier document bytes are an exact prefix. Diff check passes. No live process, retained isolated target, Cargo home or worktree; evidence remains under100MB. Next: unread Mien/Standing owner,11files /125,757characters.


### Mien and Moothold source complete: Standing authority, arithmetic and isolation, 2026-10-17

Read18new complete files /158,535characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221: all11Mien files /125,757characters and all7Moothold files /32,778characters. Repository coverage is2,501/3,731files /29,941,592/39,433,387characters, with no stale/pending credit. Full review remains active and incomplete.

Six public diagnostic cases freshly compile and execute the unchanged complete Mien production owner over8exact retained fingerprint-matched suppliers, using actual signed/attested operations, common admission and MemoryBackend. Q359: an unrelated signer can censure and pardon another root through store/fold without the governance check assumed by event/ledger prose. Raw retention is distinct from effective governance authority; no external-role or hosted gate bypass is claimed. Q360: u64::MAX Pardon gives-1 while Censure gives+1; ordinary30weight controls behave correctly. Q361: a valid same-author made-at100/completed-at99 log is sorted into completion-before-creation and scores-20 lapsed; direct causal fold and forward-time control score+10. Q362: two fresh10000bp vouches double a newcomer's boost while the voucher keeps its score, and self-vouch doubles the voucher's accrual. This is not same-hash replay. Q363: signed65535bp is admitted despite the documented10000ceiling and lends6from score1. Q364: a raw author/log0 reused across two Moot topics causes both projections to score2from one own event each; separate-store control yields1/0. Derived per-Moot signers and separate aggregate stores are distinct isolation controls.

Q365 is reproduced in the unchanged complete Reciprocity module plus actual Gemot MootId, compiled with namespace scaffolding and one retained serde supplier. u64::MAX service debt becomes-1 and allows another request at zero tolerance; maximum tolerance on an empty ledger becomes-1 and refuses. A normal debt/tolerance control works. All5original module tests and3audit tests pass. This is not whole Moothold, durable service credit or a real request router.

A1036-A1047 separate actual shared operation storage from missing moderation meaning, shared stable-author binding from deliberate legacy handling, topic indexing from log isolation, deterministic timestamp sort from causal meaning, documented numeric domains from primitive widths, vouch edge dedupe from boost accrual, host-supplied gate facts from collected rate history, explicit one-hop composition from schema/weight limits, persona roots from stable master accrual, the duplicated current-revision preflight/fold pattern in Moothold, signed federation terms from in-memory reciprocal accounting, and source-only network/durable tests from current execution. Moothold's founder-only model meansQ355's distinct-elector counterexample cannot simply be relabeled as its result. Its signed MemberTerms does validate10000bp; other raw fields do not share that bound. Current Gemot publishing also makes old receive-only Standing prose stale.

No numerical/GC/governance policy is chosen and no production code is edited. Repairs that need wider authority/lifecycle/namespace decisions are documented. No fresh Cargo dependency graph, original Mien suite, whole Moothold owner, disk, live LogSync, external-role or physical product acceptance is claimed. All generated owner rlib/executable/PDB/link outputs are removed; no process remains live.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/mien_moothold_review_receipt.json), [Mien public diagnostics](../../../support/receipts/authority-success-audit/mien_owner_probe_receipt.json) and [Reciprocity scoped tests](../../../support/receipts/authority-success-audit/moothold_reciprocity_probe_receipt.json). Prior evidence, protected source/runtime/auxiliary hashes, shared WIP and Git index remain intact; all previous document bytes remain an exact prefix. Diff check passes. No retained isolated target, Cargo home or worktree; retained receipts remain under100MB. Next: unread Mooting recognition/electorate owner.


### Mooting, Illume and probe source complete: scoped success and ownership claims, 2026-10-17

Read31new complete files /209,825characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221: Mooting3files /10,090characters, Illume9 /64,005 and all probes19 /135,730. Coverage is2,532/3,731files /30,151,417/39,433,387characters, with no stale/pending credit. Full review remains active and incomplete.

Q366 reproduces the unchanged lexical-Wasm PowerShell launcher continuing after a Cargo native double exits7, reaching the successful Node double and exiting0. Cargo0 is the control. Two audit cases, zero original tests; fresh no-profile child PowerShell and child-only PATH doubles. No real Cargo build, Node engine, Wasm module or stale artifact ran. Native failure propagation is a bounded repair candidate; production is unchanged. The native and Wasm source fixtures currently match and reuse the actual codec, but the JS wrapper prints digest/length without comparing the native pin. Historical source comments are not a fresh cross-engine receipt.

A1048-A1059 record actual pure recognition ownership versus stale storage prose; caller-supplied electorate versus signed provenance; shared Jotdown parsing versus duplicated offset/stack loops; one trusted injection registry versus host span validation; coarse Logos floors versus stale later-tier prose; entity text matching versus semantic navigation; real protocol admission versus HEADED label; duplicate signed-pack fixtures and transferred manifests versus payload/install/curation; shared graphics workload/device versus explicit B quarter-yaw differences; a partially shared geometry oracle versus independent depth checks; measurements without a fail threshold; and intentional cache contracts versus unexecuted GPU evidence. Heading breaks and A obsolete-strip cache behavior remain source candidates awaiting controlled evidence.

The standalone graphics dependency observation is qualified explicitly. Three literal nested Mesocosm manifests are absent; current parent Cargo config has no active paths override. The first locked/offline --no-deps metadata command exited0, after which its runner failed an incorrect expected-failure assertion. Full locked/offline metadata then terminated in Python TimeoutExpired after30seconds, with no Cargo exit diagnosis, full-resolution receipt or log. The original path receipt's buildability wording is overstrong and superseded by the timeout observation. This is not an established compile failure or graphics defect. No resolver process remains live; source and probe lock hashes are unchanged.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/mooting_illume_probes_review_receipt.json), [launcher diagnostic](../../../support/receipts/authority-success-audit/lexical_launcher_failure_probe_receipt.json), and [metadata qualification](../../../support/receipts/authority-success-audit/probe_dependency_resolution_timeout_observation.json). No production edits or original Rust/Node/Wasm/GPU/radio/installation tests. Prior evidence, all protected source/runtime/auxiliary hashes, shared WIP and Git index are preserved; previous report bytes remain an exact prefix. Diff check passes. No retained isolated target, Cargo home, worktree or generated build output; tiny audit command doubles remain as evidence within the100MB budget. Next: unread script/app-host source.


### All script source complete: destination parsing, component trust and divergent guards, 2026-10-17

Read36new complete files /176,097characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221. All37files /185,641characters under crates/script are now covered: app-host7 /29,051, document-host21 /97,875, Rhai8 /51,486 and shared WIT1 /7,229. Rhai scenograph_rules.rs /9,544characters was already read and remains hash-bound; it is not counted again. Repository coverage is2,568/3,731files /30,327,514/39,433,387characters, with no stale/pending credit. Full audit remains active and incomplete.

Q367 is reproduced by freshly compiling the complete unchanged net helper with one exact retained url2.5.8 supplier. The raw request https://outside.test\@allowed.test/private passes the allowed.test host gate, while the actual URL parser reads outside.test as destination; normalizing first makes the same gate refuse. Ordinary allow/deny controls work. ScriptHost source forwards the original string to its injected backend. Two audit cases, zero original tests; no Wasmtime host, production fetch backend, HTTP request or credential transfer ran. Scheme/port differences are the documented host-only policy, not an additional executed bypass.

Q368 is source-confirmed: a safe public attach path selects unsafe native-code deserialization solely by .cwasm suffix, while the SAFETY comment assumes trusted first-party serialized bytes. Exact cached Wasmtime45.0.3 contract excerpts confirm the unmodified-precompile provenance requirement; compatible config/version is not arbitrary-code validation. Registry disk discovery only accepts validated .wasm, a real mitigation for that narrower ingress. No malformed native code or exploit executed. Q369: app-host sets its epoch deadline once at attach; a continuous watchdog consumes it across idle time and successive calls, with no public rearm of the private store. DocumentScript separately rearms each guest call. This is source-confirmed with upstream deadline semantics, not a fresh timing test.

Q370: extension runtime checks requested capabilities against its separate allowed vector, then grants all imports regardless of either input. net is now live although runtime docs/tests still call fetch future work. Its absent backend and empty origin list do currently refuse actual requests; this is import-boundary/authority drift, not an executed egress bypass. Q371: both duplicated guest-build scripts warn after failure while claiming tests will fail, but leave fixed-path artifacts unbound to current source/build success. Tests can load an older compatible guest. Library-only builds may intentionally proceed; no actual failed build/stale module or original integration test ran here.

A1060-A1073 distinguish shared WIT from enforcement, accepted-for-lowering from applied actions, snapshot discovery from per-emission sink checks, RAII/per-call/unguarded containment paths, AOT compatibility from provenance, actual Servitor adapter reuse from extension policy, local host parsing from parsed request identity, async signatures from current blocking fetch, DOM liveness from scoped handles, initial preflight from complete atomic semantics, owned-page teardown/log memory from linear-memory quotas, repeated guest build/discovery, shared Rhai sandbox versus two intentional eval contracts, genuinely shared deterministic rule fixture versus its bounded recorded digest, and source tests versus fresh product evidence. Document detach dropping the owned page on teardown error, no-parent InsertBefore succeeding as a revision-only change, unbounded host logs, sequential mutation semantics, cancellation cleanup and nested Rhai error categories remain explicit source followups; no host crash, page loss, GC/teardown semantics or broader policy is chosen during audit.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/script_hosts_review_receipt.json), [net helper diagnostic](../../../support/receipts/authority-success-audit/document_net_gate_probe_receipt.json), and [exact source/contract traces](../../../support/receipts/authority-success-audit/script_host_source_traces.json). Production code, prior evidence, protected hashes, shared WIP and Git index are unchanged; all prior report bytes remain an exact prefix. Generated diagnostic executable/PDB/link outputs removed. Diff check passes. No retained isolated target, Cargo home or worktree; retained evidence remains below100MB. Next: unread Servitor authority and runtime source.


### Servitor source complete: signed audiences, author identity and wake authority, 2026-10-17

Read all12Servitor files /207,425characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221. Coverage is2,580/3,731files /30,534,939/39,433,387characters, with no stale/pending credit. Full audit remains active and incomplete.

Fresh direct compilation of the complete unchanged owner over3exact retained Cargo-fingerprint-matched suppliers passes all98original unit tests. The first separate public consumer then fails to compile because the audit omitted identity::delegation::Issue; its receipt/source/output are preserved. V2 adds that audit-only import, freshly recompiles owner/consumer and reproduces4public cases without needlessly rerunning the unchanged original suite. This is not a production compilation failure, fresh Cargo dependency graph or live host acceptance.

Q372: a genuine root-signed certificate for unrelated.application/world-Z passes the shared signature check, answers a typed trail request in DelegationTable and reaches an actual Gate/GraphLog commit. Normal-domain and outside-path controls work. Servitor does not compare the signed domain/resource against an expected audience; its provider has no resource context. No live host foreign-certificate import or wrong on-disk graph is claimed. Gemot explicitly checks is_moot_scope and is a distinct mitigation. Shared signature grammar does not supply application-owned audience checks automatically.

Q373: two distinct raw Subject values with the same first4bytes receive the same denizen:first8hex journal author. Authorized Gate commits from both have equal author fields; a watch for one suppresses the other's label as self. A different-label control wakes. This uses synthetic public Subject values, not generated colliding cryptographic keys or an identity-authentication bypass. Full-width durable attribution/self identity and legacy author migration are a broad seam to document.

Q374: a private Read watch registered at time9 still wakes and reveals matched sequence1 after the same current provider advances to expiry10 and refuses Read. Explicit removal stops it. register checks authority, but wake/would_wake/run_cascade do not accept a provider, contradicting the core containment/wake-time prose. No live product delivers a body to a revoked peer in this case. Q375: one subject's overlapping trail and trail/today subscriptions make peek name it once but drain return2Wake records for one entry. Cursor replay control is quiet. This proves duplicated scheduling inputs; no product body/external effect ran twice.

A1074-A1087 record real typed capability/signature/attenuation reuse versus separately owned chain/root/read/revocation policy; missing signed audience versus Gemot scope checks; inclusive signed versus exclusive unsigned expiry; full authority subject versus shortened authors; actual shared petition validation and exact-ID endpoint checks; projections versus proven authority; registration versus current wake permission; matcher reuse versus divergent aggregation; deferred cursors versus host-retained pending entries; deadband versus generic petitions; clock scheduling versus journal watches; public resident snapshots versus dispatch permission; and pure run reduction versus durable/executed effects. Additional parser/projection/host-routing observations remain source-qualified, with no new expiry policy, persisted identity migration or lifecycle economics chosen.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/servitor_review_receipt.json), [original unit run and preserved failed consumer](../../../support/receipts/authority-success-audit/servitor_owner_probe_receipt.json), [corrected public cases](../../../support/receipts/authority-success-audit/servitor_owner_probe_v2_receipt.json), and [source traces](../../../support/receipts/authority-success-audit/servitor_source_traces.json). No production edits. Protected source/runtime/auxiliary hashes, prior evidence, shared WIP and Git index remain intact; every previous report byte remains an exact prefix. All generated owner rlib/unit/consumer executable/PDB/link outputs removed; no process remains live. Diff check passes. No retained isolated target, Cargo home or worktree; retained evidence remains below100MB. Next: unread shell/chrome source.


### Chrome and comms source complete: configured navigation and adapter identity, 2026-10-17

Read22new complete files /192,269characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221: chrome12 /103,871 and comms10 /88,398. Coverage is2,602/3,731files /30,727,208/39,433,387characters, without stale/pending credit. Full source audit remains active and incomplete.

Q376 reproduces the mismatch between a merged multi-adapter inbox and first-by-protocol dispatch. Two same-protocol adapters both list conversations, but the second unique thread returnsNotFound for read/send; reversing registration changes reachability. A shared protocol/key produces two titled rows but selecting the second reads/fake-sends through the first owner. This runs unchanged portable Comms and its supplied InMemoryAdapter; fake sends return deterministic IDs and do not author/deliver mail. No real account or live message was exercised. Backend/account identity must survive routing or duplicate protocol registration must be visibly refused; trying successive sends is not safe owner discovery.

Q377 reproduces resolve_match discarding Bing/Google provider tags and returning DuckDuckGo through the separate hardcoded NavTarget resolver. Q378 reproduces custom colon-command classification disagreeing with suggestions using default greater-than: colon-home becomes a search row, while greater-than-home loses its search rows. The suggestion producer deliberately supports default/DuckDuckGo today; these are public shared-input/configuration disagreements, not observed live user queries or deployed preference violations. Three complete unchanged modules nav/omnibar/suggest pass32original tests plus2audit cases over exact retained kernel/url suppliers; full Chrome/focus tests did not run.

Portable Comms core passes20original unit tests, with optional Misfin/Murm adapters disabled. The first direct unit link omitted a retained Windows import-library search path; that attempt is preserved. Its portable owner/consumer and2routing cases already passed. Only original unit compilation/tests were retried with the missing native path. The total fresh execution is52scoped original tests and4audit cases, not a full package/all-feature Cargo receipt.

A1088-A1102 distinguish mutable chrome state bundles from permission decisions; extracted FrameViewModel/HostIntent from stale ownership prose; documented1-p^2 focus curve from tested(1-p)^2; portable intent vocabulary from actual host handlers; URL heuristics/history from document acquisition; duplicate provider/default-sigil decisions; linear history ranking from durable frecency; shared request-state generation from status projection; alias-shared toolbar drafts from mirrored settings; protocol from adapter/account identity; explicit delivery facts from recipient/read success; real Murm owner reads from generic text-only diagnostic loss; receive-only Misfin projection from unread/read-state uncertainty; local thread/draft guards from pane-clone/identity-switch obligations; and source completeness from executable feature scope. Misfin unread/protocol checking, clone-shared request identity, retained inbox after identity clear and blocking store calls remain source followups with their exact limits.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/shell_chrome_comms_review_receipt.json), [Comms routing and preserved first unit link](../../../support/receipts/authority-success-audit/comms_routing_probe_receipt.json), [core unit retry](../../../support/receipts/authority-success-audit/comms_core_unit_retry_receipt.json), and [Chrome module tests](../../../support/receipts/authority-success-audit/chrome_navigation_probe_receipt.json). No production edits. Prior evidence, protected source/runtime/auxiliary hashes, shared WIP and Git index remain intact; every prior report byte remains an exact prefix. Diagnostic libraries/executables/PDB/link outputs removed; no owned process live. Diff check passes. No isolated target, Cargo home or worktree retained; evidence remains below100MB. Next: unread Stickleback source.


### Stickleback drop/storage boundary checkpoint:14of30source files, 2026-10-17

Read14new complete files /200,309characters at unchanged HEAD 609b0a60373bdb54cd899254397bb061096a3221: manifest, all4examples, address book plus tests, retention-authority seam, causal helpers, drop codec/import-export, epoch-retention proposal, data keyring and public lib exports. Stickleback remains16files unread, starting group_lane.rs; full crate compilation does not grant their source-reading credit. Coverage is2,616/3,731files /30,927,517/39,433,387characters, with no stale/pending credit. Full review remains active and incomplete.

Q379 reproduces export_plain_topic_file and export_selected_plain_topic_file truncating an existing destination before returning a configured max_protected_bytes0 refusal. The two destinations are new audit-owned sentinel files and are empty afterward. A successful default-limits control round-trips one genuinely signed operation stored through the actual processor/MemoryBackend, with matching semantic drop ID and unchanged source-store count. No real user file or protected drop was exercised. File export does not explicitly promise atomic replacement; the evidence is the concrete destruction of a previous destination on a validation failure. The protected helper shares that source ordering but remains unexecuted. Repair should preserve old destinations until a valid new artifact can be published under an explicit replacement policy.

The complete unchanged current Stickleback owner and public consumer are freshly compiled over17exact retained Cargo-fingerprint-matched suppliers. First harness attempt omitted CARGO_PKG_VERSION; second compiled the owner but imported a private module instead of its public root re-export. Both failures remain preserved and neither executed file cases. Third corrects only audit setup/import and passes2cases; original tests0, no fresh Cargo graph or live transport. All30source hashes protect compile provenance only. Three tiny final fixtures are retained (0,0,557bytes); generated owner/library/executable/PDB/link outputs removed.

A1103-A1116 distinguish shared Backend address-book storage from soft directory metadata and remaining read/write races; actual causal mechanics from authenticated entries; manifest/carrier integrity from domain policy; pre/post-CBOR canonical identity migration inside version1; shared operation preflight/batch calls from still-unread atomic storage mechanics; trusted verified-record import and staged retry identity; advisory peer receipts from local durable completion; payload hydration/content-only storage from operation admission; file publication from in-memory validation; dry-run epoch facts from authenticated destruction authority; p2panda data crypto/chronology from membership control; finite inventory-model byte/replay evidence from signed/domain/physical/RF/FEC acceptance; and exact source coverage from compilation. No checkpoint/GC/destruction policy is chosen, and no caller trust assumption is presented as a verified permission.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/stickleback_drop_review_receipt.json), [first compile failure](../../../support/receipts/authority-success-audit/stickleback_export_probe_receipt.json), [second consumer failure](../../../support/receipts/authority-success-audit/stickleback_export_probe_v2_receipt.json), and [public export cases](../../../support/receipts/authority-success-audit/stickleback_export_probe_v3_receipt.json). No production edits. Prior evidence, all protected/runtime/auxiliary hashes, shared WIP and Git index remain intact; all previous report bytes remain an exact prefix. Diff check passes. No retained isolated target,Cargo home or worktree, and no owned process live; retained evidence remains below100MB. Next: unread group-key lane/session and remaining Stickleback source.


### Stickleback completed source review and ownership followup, 2026-10-17

Read the remaining16Stickleback files /295,739characters. All30Stickleback inventory files /496,048characters are now fully read and hash matched. Shared main externally advanced from 609b0a60373bdb54cd899254397bb061096a3221 to 7c71810550234ccbe11f7429cd44e369b279ec06 while this pass ran; all30Stickleback hashes remain unchanged. Twenty previously read sources changed and7new source files were added. Their old read credit is excluded until current bytes are reread. Repository coverage is2,612/3,738files /30,574,532/39,556,704characters, with zero pending chunks and20changed-hash entries. The full audit remains active and incomplete; HEAD matches origin/main at this checkpoint.

Q380 reproduces a persistence retry losing the group session transition: a real signed update advances the live session, the save callback returns an injected refusal, then retrying that same session returns success and marks the update superseded without invoking save. Reopening the old saved session against the same lane skips the settled update and cannot decrypt the new epoch. Successful-first-save and reload-before-retry controls each persist once and decrypt after reopen. Secret state stayed only in memory. This is a save-callback failure case, not real disk failure or a demonstrated host data-loss incident.

Q381 reproduces divergent retained state from two effect planners in the shared store. An explicitly authorized checkpoint both prunes two predecessors and erases the first payload. The single-operation path reports2pruned/1erased but recreates the first signed header as a bodyless log entry without its operation pointer; the subsequent payload-strip Put overwrites a prior Delete. The native-drop batch applies the identical signed corpus under the same policy and leaves only the checkpoint. Its overlay handles deletion correctly. Shared crate ownership and atomic Backend writes have not removed this duplicated implementation. Repair concerns effect composition, not a new destruction-policy choice.

A1117-A1131 distinguish cryptographic group identity from domain membership authority; in-memory transition atomicity from durable marker ordering; parked-release swallowed errors from permanent refusal; bounded advisory inventory from custody; shared join lifetime from endpoint lane-ID uniqueness and awaited shutdown; transport EOF from application acknowledgement; single/batch effect duplication; process-local clone-shared locks from batch/frontier atomicity; typed/Any compatibility views over one schema; portable completion receipts from peer trust; quiet accepted-arrival counters from convergence; and neutral boundary tests from universal ingress enforcement. Q199's cached-header/parsed-extension split is retained as an existing finding, not recounted. Parked storage faults, batch concurrency and live convergence are documented followups, not executed conclusions.

User steering prioritizes unfinished graph/pane ownership migration over file splitting. Targeted external Turnstone source at aaad30d67abc2b7454e70d33e9a84527d28f6f1e confirms with_graph_pane resolves the graph and installs/stashes pane camera/selection, while the pool's Deref still reads the active compatibility cursor. open_address visits through that boundary and then looks up the returned graph-local node key through the compatibility cursor; selection reads also retain a shared-canvas fallback. These are external source followups, with no Mere read credit, full Turnstone review, new bug reproduction or edits. User-mentioned Arrange/cross-window/first-frame failures are context rather than freshly executed evidence. Sustained migration work should carry explicit ownership through the entire operation and distinguish Frisket topology, blueprint projection and Workspace state; broad work remains documented under the audit's hour rule.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/stickleback_completion_review_receipt.json), [three group retry cases](../../../support/receipts/authority-success-audit/stickleback_group_retry_probe_receipt.json), [two retention path cases](../../../support/receipts/authority-success-audit/stickleback_prune_erase_probe_receipt.json), [external main refresh](../../../support/receipts/authority-success-audit/main_refresh_after_stickleback_receipt.json) and [preserved explicit inventory inclusions](../../../support/receipts/authority-success-audit/main_inventory_scope_correction_receipt.json). Both complete unchanged Stickleback owners and public consumers compiled over17exact retained suppliers; original tests0, live network0. This is retained-family evidence, not a fresh Cargo graph under the externally changed Cargo.lock. First checkpoint attempt stopped at its old-HEAD assertion before report/notes mutation; that terminal failure and its exact runner are preserved. The base inventory refresh omitted3earlier explicit source inclusions; all3still-present files were restored with matching old read credit, so scope does not shrink. Generated libraries/executables/PDB/link outputs removed. Prior evidence and unchanged protected hashes remain intact. External publication changed20reviewed sources,Cargo.lock,.gitattributes andGit index; the refresh records those changes rather than falsely claiming the old snapshot survived. Other protected WIP matches. Previous report bytes remain an exact prefix; diff check passes. No isolated target,Cargo home or worktree retained. Next: reread changed/new inference sources before continuing Luggage.


### Main reread, inference ownership, and completed Luggage source, 2026-10-17

At checkout HEAD 7c71810550234ccbe11f7429cd44e369b279ec06, all20externally changed sources and7new inference files /772,041current characters are now reviewed. Six changed files plus every new file were emitted wholly; fourteen other changed files use a composed review: exact old Git bytes reproduce an earlier complete read hash, each unchanged block is equal to current bytes, and every changed hunk is emitted and ACKed. These fourteen files contain542,964previously read unchanged characters plus49,864new characters. The receipt verifies reconstruction and complete hunk coverage rather than calling a diff scan a whole new read. Luggage then adds10whole files /106,957characters. Coverage is2,649/3,738files /31,453,530/39,556,704characters, zero stale/pending credit. Full review remains active and incomplete.

Q382: the new architecture reference max-error subcheck reports0for mixed/all-NaN actual logits because f32::max suppresses NaN differences. Exact-match and finite-mismatch controls pass. This is an exact expression projection; no complete architecture/model/GPU test is claimed to pass invalid output. Existing Q348 rank, Q349 Unicode streaming, Q347 sampling and Q217 actor-disconnect findings are not fixed by these ownership changes and were not rerun.

Q383: unchanged production StagedUpdate load/take_verified accepts substituted unsigned bytes after both staged-artifact and the digest in neighbouring staged.json are changed. Intact control succeeds and artifact-only tampering is refused. The record loses publisher signature/key/manifest evidence across restart and becomes its own integrity authority. Requires write access to both stage files; no remote exploit, privilege escalation or initial genuinely signed download is claimed. install_staged's unchecked native handoff is source reviewed only. Retaining authenticated publisher evidence and defining old-stage migration is a high-effect design issue documented under the hour rule, not repaired here.

Q384: unchanged discard returnsOkafter both remove_file calls fail against audit-owned empty directories; both paths remain. Q385: source-only Windows MSI handoff ignores fallback msiexec spawn failure and unconditionally exits0after PowerShell spawn failure, even when no launcher starts. No OS installer/process-exit experiment ran. Broader rollback, archive structure/path validation, cross-filesystem install and staged durability work remain documented, without inventing destruction or publisher policy.

A1132-A1148 track actual consolidation: DecoderConfig drives declared math, checkpoint/loader share tensor-name grammar, ResolvedInferenceRow::artifact_source now owns both core-layer and expert identities, Distillery supplies only store access, and template EOS reaches both loops through one provider list. Explicit F32/native/inference-only guards stay distinct from training/browser support. Resident/streamed/staged paths share kernels but their resource/placement receipts are different. Unfinished authority includes independent numerical validity checks, editable restart staging, installer handoff versus completion, CLI loose manifest grammar/signature normalization, and authenticated/query HTTP signature retrieval. Core ReleaseRefV1 remains naming-only; prose still claiming browser verification exceeds its private/native verifier API.

Fresh offline CPU gate passes27tests:24unchanged original config/portable-provider/release-core tests plus3audit tests. Staging production methods are included unchanged before cfg(test), over an audit Update facade whose install panics and is never invoked. First retained-supplier setup failed before compilation because the old build/debug rlibs had disappeared; terminal failure is preserved. The fresh graph is small and offline, not a full Luggage/ESP native graph. A local Cargo build-dir setting redirected its generated cache despite CARGO_TARGET_DIR. After terminal completion, Cargo marker and live-owner checks moved only generated output to C:/t/cargo-targets/mere/build and removed the audit test executable/PDB. No local/isolated target remains; the187,458,309byte reusable shared supplier cache is owned by this active audit for later scoped gates. Small final staging fixtures remain as evidence. Future gates must explicitly set build.build-dir too.

Fetch finds origin/main 00e4442868ee1cb014364a81ccaed657b512b218, eight commits ahead. git merge --ff-only refuses because it would overwrite the shared scenograph editor/audit report; no stash,revert,commit or production changes were made. Fourteen upstream-only diffs were read: keyboard text/paste identity, explicit producer presentation purpose, canonical Resource tags and their tests, toolchain and published welding pin. This grants no checkout read credit. Twenty-four other upstream diffs, including document/fetch/link ownership work, remain queued; do not call origin installed or completely reviewed.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/main_refresh_luggage_review_receipt.json), [current composed reread](../../../support/receipts/authority-success-audit/main_delta_plan.json), [CPU gate](../../../support/receipts/authority-success-audit/main_refresh_staging_probe_v2_receipt.json), [upstream status](../../../support/receipts/authority-success-audit/upstream_after_inference_review_receipt.json), and [generated-output relocation](../../../support/receipts/authority-success-audit/staging_build_relocation_receipt.json). Previous report bytes remain an exact prefix. Protected source, runtime, auxiliary inputs, unrelated WIP, prior evidence and Git index match. Diff check passes; receipt evidence is below100MB. Next: Registry source and remaining upstream diffs, preserving current report/WIP.


### Upstream ownership diff completion, 2026-10-17

All38changed-file diffs from local 7c71810550234ccbe11f7429cd44e369b279ec06 to upstream 00e4442868ee1cb014364a81ccaed657b512b218 are now read through exact Git objects. The initial combined tail output was truncated and earns no credit: its7file/66,001character tail was preserved, re-emitted in full as0:22000,22000:44000,44000:66001, and reviewed completely. This is an upstream diff review, not a whole upstream checkout/source receipt or fresh test run. Local full-source coverage remains2,649/3,738files with no stale/pending credit. Git still refuses fast-forward because it would overwrite the shared report; no stash,revert,commit or production edits.

A1149-A1152 track the resource and input/link ownership changes. Both HTTP adapters now use the shared host-request lowerer and preserve non-success HTTP metadata in rich responses; their response DTO lowering remains repeated. Absolute rich-request deadlines and zero-concurrency refusal are real source changes. Held HTTPS fixture bytes refuse invented response metadata. Legacy Fetch compatibility with a custom plain-only implementation needs separate checking, with only NetFetch implementing that trait locally. Kept links call one graph-owner mutation for address plus Hyperlink and preserve an existing address; this does not finish pandect facet-store adoption, explicitly still absent in all hosts in upstream's report. Input adapters separate key events from keyboard text/paste while still repeating printable/control/modifier rules. Focus/IME/AltGr and trait-compatibility followups remain documented rather than redesigning them here.

The fetched locks advance Genet7f13b85f to917f88e6 and change other supplier edges; published welding0.15.0 replaces its moving Git pin. These are not installed in this checkout. The upstream Sets qualification reports focused passes and a reproduced unchanged timing-control failure, not a full library pass. Its receiving Turnstone adoption and live cache invalidation/headed/GPU scope remain open. Resource adoption's JSON carries an earlier source/lock and external-log basis; actual current per-file hash matches/mismatches are preserved in the receipt, without treating upstream-authored tests as locally rerun proof. The documented private Isocosm authoring consumer remains app-local with shared API extraction deferred, not new shared Mere authority.

Q383's staged trust gap was already documented and independently probed in the canonical auto-update brief's2026-10-12comparison (lines374-401). This pass is a fresh scoped confirmation of an existing open trust/apply migration, not a new discovery or authority to adopt a trust provider. The brief already separates update trust, install recovery and product policy; those gates remain intact.

Evidence: [upstream completion](../../../support/receipts/authority-success-audit/upstream_ownership_review_receipt.json) and [full re-emitted tail](../../../support/receipts/authority-success-audit/upstream_tail_review.txt). Source/WIP/index hashes and previous report bytes remain intact; no process live or isolated target retained. No new tests or finding IDs beyond Q382-Q385; only four further authority entries. Next: Registry source and the rest of the full inventory.


### Registry, session wrappers, surface contracts and UX audit, 2026-10-17

Read all45Registry source/manifest files (400,034characters), all12shell-state files (8,340), all4surface-api files (14,366), and all11UX Events files (78,278). UDC seed JSON is separately read auxiliary input, not source-denominator credit. Current full-source coverage is2,721/3,738files and31,954,548/39,556,704characters, with no stale or pending chunks. The remaining full repository source is still unread, including Tabard, remaining product ports and supplier patches. A fresh fetch leaves origin unchanged at00e4442868ee1cb014364a81ccaed657b512b218; the already-reviewed38upstream diffs remain separate from current checkout7c71810550234ccbe11f7429cd44e369b279ec06. Fast-forward still overlaps the shared report. No stash/revert/commit/push or production edits.

Q386-Q391 record six issues. Q386: declared25percent tile-activity sampling is ignored by generic configdefaults, admitting8/8events; explicit config25percent admits2/8. Retention256versus100 is a metadata mismatch, not a downstream storage reproduction. Q387: absent native runtime marks the web-runtime modActive and publishes viewer capability; an explicit refusing runtime does not. The global query still returns true from its separate once-cached no-runtime registry. Q388: a failed provider does not stop its dependent callback, Active status, success diagnostic or published capability. Q389: a secondload bypasses quarantine and replaces the outstanding old extension record; later successful unload reportsUnloaded while the synthetic old effect remains. Q390: legalpipe remap / greater-than chord characters fail string round-trip; ordinary controls pass. Q391: UX mapper handles40surface channels but all_channel_ids lists36, omitting Inspector and Steward opened/dismissed IDs; the completeness-named test pins the stale38total with dispatch channels instead of exercising every surface.

Verification:89tests passed,84original and5audit, with1existing native-inventory test ignored. The initial external-path audit harness failed compilation and is preserved with zero testcredit. Corrected composition compiles a byte-identical current Registry source tree, links diagnostics/input/layout/mod_loader with their originaltests and actual command-menu, and uses only the exact data-only ViewerSubsystemCapabilities fragment needed by extension records. It does not execute viewer routing, knowledge/lens/protocol, graph-kernel, real native mods, WASM instances, host startup, browser, network orGPU. Activation/removal failure cases use the documented injected callback seam with synthetic effects; the absent-runtime case calls no native hook atall. UX catalogue reconciliation is a source-body literal comparison, not a fresh Rust/host test. Audit executable/PDB removed; reusable supplier cache stays under the stable shared Mere target/build path. Other active workspace compilers are unrelated and untouched.

A1153-A1169 track the ownership seams, including real consolidation and unfinished adoption. Default input command IDs share command-menu; lens theme and glyph types re-export Tabard/kernel. Registry's correlated bounded watchdog and nonblocking staging queue have meaningful exact-key/loss controls; the compatibility observer remains uncorrelated. Descriptor schemas/conformance metadata do not themselves validate livepayloads or guarantee runtime behavior. Native declarations, separate global readiness, topological presence and lifecycle state still divide activation authority. Viewer URI/MIME-kind/plaintext handlers repeat selection rules, and lens donor physics/config IDs have no current local runtime consumer. Those broader migrations are documented rather than choosing host policy in this pass.

Current production reference search finds document-host implementing the WASM runtime trait, but no local product ModRegistry construction, global capability query, sender installation, diagnostic observation gate, UxObservers host or UX channel bridge call. That is a bounded repository census, not a claim about external Turnstone/other consumers. Shell-state is actual re-export consolidation into Chrome/UX Events, not duplicate focus/session state; its root owns-types prose is stale. Surface/settings DTOs deliberately leave admission consistency, apply validation and storage with hosts/products. UX recording/probes remain passive: count-only rings, unbounded failure lists, missing pane/window/run correlation and no productive-pending terminal inspection are separate documented followups. Empty probe failures do not prove event completeness or runtime success.

Evidence: [source and ownership checkpoint](../../../support/receipts/authority-success-audit/registry_ownership_review_receipt.json), [scoped Registry tests](../../../support/receipts/authority-success-audit/registry_owner_probe_v2_receipt.json), [preserved compiler failure](../../../support/receipts/authority-success-audit/registry_owner_probe_receipt.json), and [UX catalogue source check](../../../support/receipts/authority-success-audit/registry_ux_catalog_check_receipt.json). Previous report bytes, guarded source, shared WIP and index remain unchanged. No isolated target, Cargo home or worktree retained. Next source: Tabard.


### Tabard authority and persistence review, 2026-10-17

All28Tabard source/manifest/test files (281,469characters) are now read at current hashes, plus its Lagrangegolden fixture as auxiliarydata. Together with Registry/session/surface/UX thisturn added100completefiles. Coverage is2,749/3,738files and32,236,017/39,556,704characters. Every sourcefile under the current crates/ inventory isread; this is not full repository completion. Executable design/receipt source, remainingports/scripts and supplierpatches still remain in thedenominator. No pending/stale readcredit. Localmain remains7c71810550234ccbe11f7429cd44e369b279ec06; fetchedorigin00e4442868ee1cb014364a81ccaed657b512b218 has no furthercommits and the previouslyreviewed8commit/38diff advance is notinstalled because of the sharedreport overlap.

Q392: rawThemeRegistry::add_user_theme can replace theme:light, force provenanceUser, and thereby allow rename/removal of a supposedlyimmutable built-in. Public actualAPI test reproduces it with a fresh-IDfork control. Workshop rejects the collision. CurrentGraphshell intake independently checks built-in/sourcecollision, while Pelt/Workshop load through the stricterThemeLibraryStore. Those are real guards; nohostimport bypass or changedusersettings is claimed. Q393: applicationFontPreferences request SERIF resolves/emits sans-serif fallback with an empty installedcatalogue, while lowercaseserif and authoredThemeFontStyle SERIF retain genericserif. Requested spelling ispreserved; the genericrecognition helpers disagree. Pelt fontCSS/notices call the affectedresolver, but noengine face/layout or headed appearance wasexecuted.

Actualcurrent Tabardroot and localTinct compiled in a standaloneCPU dependencygraph.81tests passed:37existingunit tests,42existing tests from7selectedpureCPUintegrationfiles and2public auditcases. This includes real temporaryfile load/save, busy/stale writerrefusal, failurepreservation, retry, portableversioning, authoredCSS versus derivedpalette andundo/conflict controls. The originaltheme.rs Genet/Livery integrationfile was read but notrun; neitherUnix-only permission/symlink test nor externalLagrange/runtime/font/GPU acceptance isclaimed. All18exact testexe/PDB outputs named by the completedrun were markerchecked andremoved; shareddependency cache remains in theapproved stabletarget. Noisolatedtarget/home/worktree andno productionedits.

A1170-A1177 record both actualconsolidation andremainingmigration. Sharedresolve_theme_choice preserves requestedidentity, explicitfallback diagnostics and authoredCSS authority; strictmode artifacts shareTinct/harmony math and refuse pretending sheets/customcalculators arederivedpalettes. Older noargumentexports intentionally retain rawseed normalcontrast behavior, and olderThemeTokenSet/mode_tokens APIs remain alongside them, withcustommode canonicalfallback. This is documented compatibility, not a hiddennewexport failure. Libraryandartifact exports share oneactual lock/snapshot/atomicpublication writer and theirCPUcontrols pass; activechoice has a separate simplerwriter, so relativepath/temp/stale/concurrentwriter behavior stays a documentedfollowup. Applicationfontrequests andauthoredstyles own differentdata, but genericnormalization should be shared. Customcalculators versus attachedsheets still requirehostadoption tracing. Smolweb shares palettesbut rawauthority hueparsing needs equivalencecases. Neithercontrastmetadata nor a passingselectedCPU suite meansallhostroles or rendering arequalified.

Evidence: [Tabard checkpoint](../../../support/receipts/authority-success-audit/tabard_ownership_review_receipt.json) and [actual library CPUtests](../../../support/receipts/authority-success-audit/tabard_owner_probe_receipt.json). Across RegistryandTabard thisturn170tests passed,163original plus7audit, with1existingRegistryinventorytestignored; Q391remains source-only. FirstRegistryharnesscompilerfailure remains preserved. Previousreport bytes, guardedsource, sharedWIP andindex areunchanged. Fullrepository auditcontinues with the remaining sourceinventory.


### Executable receipt source and verifier authority review, 2026-10-17

Read the remaining 47 source/manifest/style/patch files under design_docs (358,536 characters), including both complete HTML catalogues and the full historical resolved lock. All inventoried source under design_docs is now read. Markdown was consulted for specific historical/current boundaries, not credited as a complete document review. Overall coverage: 2,796/3,738 files, 32,594,553/39,556,704 characters, with no stale or pending credit. Remaining ports, scripts and vendor patches stay in the denominator. Main remains 7c71810550234ccbe11f7429cd44e369b279ec06; origin remains 00e4442868ee1cb014364a81ccaed657b512b218. The reviewed eight-commit upstream advance remains uninstalled because of the shared report overlap.

Q394: the historical custody runner records Cargo exit 101 but exits zero itself. Q395: the supplier qualifier accepts all five roots with zero recognized suppliers from a registry-only fixture; its revision and duplicate-source checks only cover names already recognized. Matched supplier and wrong-revision controls pass/refuse respectively. Historical actual custody success and nonempty supplier lists remain byte-preserved and are not overturned. Q396: the browser identity filter excludes shared CSS actually compiled through include_str!. A copied real Cambium CSS input changed after preflight, but the complete unchanged seal CLI accepted sealed=true with an identical fingerprint; a Rust-source change refused. No actual stale published product package is asserted.

These are real outer Python process and file-write outcomes over exact copied verifier source. Child Cargo/Git outputs and fixture package/build-log inputs are explicitly synthetic. Five wrapper cases and three source seal CLI cases completed with their expected controls. No Cargo compilation, browser, GPU, network or production mutation was performed. Two initial harness errors were preserved separately: stdout type mismatch in the first wrapper injection, and Windows key separators in the first seal assertion. Neither is credited as a product failure.

A1178-A1186 retain ownership distinctions: generated component catalogues versus live callback behavior; pinned research arena/identity/wake models versus current runtime; historical custody adapter versus subsequently landed Backend::transact; bounded range/cache/Iroh probes versus media playback; CPU preview equality versus GPU execution; source and served package identity versus producer pixel behavior; and archived patch provenance versus current integration. Current custody documentation already corrects the historical missing-transaction conclusion. The browser qualifier has real producer geometry/idle controls and an honest failure path, but its HEAD-length serving check depends on isolated server mapping. Source sealing and supplier census need complete input membership before agreement can support an authority claim. Coupling mutators temporarily edit shared source and were only read, never invoked here.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/design_receipt_review_receipt.json), [wrapper cases](../../../support/receipts/authority-success-audit/receipt_wrapper_probe_receipt.json), [source seal cases](../../../support/receipts/authority-success-audit/source_seal_probe_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. Evidence remains below the 100 MB cap. No new worktree, target or Cargo home. Full audit continues.


### Gazette and remaining Moot ownership review, 2026-10-17

Read 12 more source files (174,538 characters): all six Gazette source/manifest/test files, five remaining Moot files and Djinn's personal-facts real-process test. Gazette and Moot inventoried source are now completely read. Overall coverage is 2,808/3,738 files and 32,769,091/39,556,704 characters, with no stale or pending credit. This remains a partial repository audit. Main/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218; previously reviewed upstream changes are still not installed.

Q397: Ledger resolve reports an unknown view/source/contact/facet delta honored after merely parsing/counting it, and accepts malformed selection text. Generation mismatch and invalid instance JSON controls refuse. It has no projection argument or application callback, so receipt wording overstates replay. Q398: uppercase ACCT:Alice@EXAMPLE.org becomes acct:ACCT:Alice@EXAMPLE.org in resolver normalization and the actual resource query. Current Gaz-backed intake instead recognizes acct:Alice@example.org and refuses the malformed resolver result against that subject; lowercase control agrees. Q399: any application/ld+json link is classified as an ActivityPub actor without an ActivityStreams profile, and URI prefix case changes typed classification. These are supplied-claim classification errors, not evidence of authenticated identity or live federation compromise.

All 20 original Gazette unit tests and three additional public API audit cases passed against its actual unchanged crate root and current lock-seeded Gaz/Genet suppliers. Existing intake conflict, alias, trust-preservation, URI validation, declared HTML/XHTML extraction and loopback HTTP controls passed. Gazette parity integration and Moot integration files were read, not rerun. No external fetch, live contact mutation, browser/layout/GPU acceptance or production edit. Four exact completed-run exe/PDB outputs were marker-checked and removed; stable shared dependency cache remains. No isolated target/home/worktree was created.

A1187-A1195 track remaining ownership work beside the bugs. Ledger still privately constructs cells, HTML and shelfmarks; its shared matrix/grid parity tests explicitly defer retirement to the dramatis split. Gazette intake already uses shared Gaz identity, while old blocking reqwest fetch and permissive normalization await the documented sans-io protocol migration. Supplied Fleece extraction is actual reuse and does not own future feed polling/storage. Moot's pane keeps drafts/attention local, uses authority-filtered Commons snapshots and returns an intention requiring owner recheck before authoring; prepare_send is not send success. Captured search is reminted from authorized fauna and resolved capture integrity, preserving contribution provenance. The practice peer uses real governance but fixed public proof identities/clock and proof persistence, not a production owner or transport. Its self-check exit versus JSON handling is left as a caller-tracing followup. Djinn's two-app/fresh-process persistence test is source-read only.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/gazette_moot_review_receipt.json) and [Gazette actual CPU tests](../../../support/receipts/authority-success-audit/gazette_owner_probe_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. Receipt bytes stay below 100 MB. Continue with OPFS probe, remaining ports, scripts and supplier patches.


### OPFS probe source and acceptance ownership review, 2026-10-17

All 19 OPFS probe source/manifest/style/script files are read (250,796 characters). Overall coverage: 2,827/3,738 files and 33,019,887/39,556,704 characters. Every inventoried file was rehashed against current live bytes: all 3,738 match, no new standard source was found, and the three explicit diff/extension-manifest inclusions remain. No stale or pending read credit. Fetch confirms unchanged main/upstream 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218; the eight reviewed upstream commits remain uninstalled because of the shared report overlap. Full audit is not complete.

Q400: fatal cut policies advertised as none/half/all actually land none/all/all because a huge absolute torn_bytes count is clamped to buffer length. Native and browser policies both reproduce [0,8,8] on an eight-byte write; explicit4 is a real half. Original short-write error injection still does half bytes, with different post-error behavior. Q401: overall page state becomes complete after a lane returns false, and the CLI returns zero for false/thrown lanes or a failed page-error gate. Native verifier exit1/false produces round_trip_both_ways_ok=false while lane5.ok and runner lane status stay true. JSON retains those failures; aggregate success must not conceal them. Q402: digest equates a listed-but-missing key with a present empty value through unwrap_or_default. Q403: rejected persisted() query becomes persistent through truthiness of the caught error object. Q404: real offline --locked metadata for the original probe refuses with exit101; its lock records local Muniment0.1.1 while current source is0.1.2. Passing a separately resolved audit CPU graph does not fix that original runner.

Actual unchanged native root passed all17 original unit tests plus2 audit cases. The originals include464+500 cut trials, error injection, generation invariants, ASCII range arithmetic, real redb backend operations and all five shared workload digest comparisons against MemoryBackend. This verifies the cases executed, with Q400 limiting the claimed fatal-byte coverage. Four exact completed-run exe/PDB files were marker-checked and removed. The JavaScript evidence executes whole unchanged page source in Node VM and six unchanged runner CLI processes: three storage booleans/errors, three overall-run controls, six CLI/native-verifier outcomes. DOM, browser lanes and native child outputs are explicit fixtures; no real browser, OPFS/IndexedDB, worker termination, promotion, power loss, portability binary, server or network acceptance is claimed. Original source, original lock and historical receipts remain unchanged.

A1196-A1205 keep research copies and migration ownership visible. The redb4 adapter and ASCII-only range IndexedDB baseline are intentionally isolated from production redb2/shipping IndexedDB; neither overrides current transact, whose typed refusal prevents accidental racy fallback but also prevents full modern-backend substitution. OPFS token ownership stays worker-local with realm checks, including failure to release on the wrong thread; this is stronger than an unsafe thread assertion, not fresh threaded browser proof. Shared workloads are real reuse, but result flags, native verification and process status still lack one complete acceptance predicate. Staging isolation and atomic promotion remain separate; documentation already disclaims proof of a kill inside move. Performance comparison correctly distinguishes shipping versus indexed range and deliberately unbatched versus shipping batched log paths. Hold-until rejection settlement, command deadlines, holder correlation, restored receipt/build identity and source snapshot assumptions remain documented followups.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/opfs_review_receipt.json), [native CPU tests](../../../support/receipts/authority-success-audit/opfs_owner_probe_receipt.json), [injected JS cases](../../../support/receipts/authority-success-audit/opfs_browser_probe_receipt.json), [original locked metadata](../../../support/receipts/authority-success-audit/opfs_locked_metadata_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. No production edits or new target/home/worktree. Continue remaining port and supplier source.


### Pelt core receiver and history authority review, 2026-10-17

All21 Pelt core source/manifest/test files are read (441,183 characters), plus root Cargo/build and desktop Cargo/appearance:25 files,484,406 characters this pass. Overall coverage is 2,852/3,738 files and 33,504,293/39,556,704 characters. No stale or pending credit. One oversized tool emission was truncated, discarded without acknowledgment and reread in full at smaller chunks; it contributes no duplicate credit. Full Pelt desktop and the repository remain incomplete. Main/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218, with the eight reviewed upstream commits uninstalled because the shared report overlaps.

Q405: public adopted-session construction accepts a live session and empty factory registry and renders correctly, but declared capability inspection panics on an unchecked registered-engine assumption. Registry-backed adoption control returns Partial. Existing adoption/semantic fixtures use the empty registry shape without calling that accessor. Targeted current Turnstone Reader adoption supplies shared real factories, mitigating this exact trigger; no live product crash is asserted. Q406: Back restores the first engine/address after navigating to a second engine, but selected/active route metadata still reports the second; Reload retains the mismatch, Forward agrees again. Existing history test checks engine/address but omits route equality. Q407: placement SessionInput::Cancel lowers to pointer0 and is discarded with only nonzero captures, leaving those captures intact; direct controller Cancel and window Focus(false) controls deliver. Per-pointer, placement, session and window scope need an explicit shared boundary, not an inferred all-window policy.

Actual unchanged engine-free core root passed114 CPU tests:111 originals across its unit suite and all12 integration files, plus3 audit cases. Existing controls exercise receiver cancellation before destruction, geometry/scale failure, chord/sensor identity, typed host load/Stop/supersede, retained close/reopen, host tearout acceptance, shared-registry fallback, repeated appearance geometry and exact semantic publication revalidation. Supplied engines are test factories; no native document engine, real surface/input hardware, browser, desktop GPU/presentation or physical AT ran. native.rs was fully read but feature genet was not compiled. All28 exact test exe/PDB outputs named by the completed run were marker-checked and removed; shared dependency cache remains in the approved stable target. No new target/home/worktree or production edit.

A1206-A1215 record actual consolidation and unfinished ownership. Workbench owns arrangement; the pool owns retained receiver/placement admission, while the workspace's pointer0 field is a compatibility projection. Shared content keys intentionally share scroll/history; independent Reader appearances use independent keys. Host graph history and reference linear history are explicit alternatives, not a reason to merge owners. PageLoad is actual shared state and the host owns acquisition; semantic publication keeps complete current projection equality, not just counters. Native factories reuse shared engines but concrete font adaptation and forwarding of newer request/capability metadata need tracing; SessionEngine's current default is Partial. Pelt appearance already composes shared Tabard choice/library/resolution with host policy, including guards previously mitigating Q392 and the Q393 font resolver consumer. Appearance tests and full native factory behavior were not rerun. Compatibility feature names and stale default-networking prose remain tracked alongside runtime migrations.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/pelt_core_review_receipt.json) and [actual core CPU suite](../../../support/receipts/authority-success-audit/pelt_core_owner_probe_receipt.json). Prior report bytes, guarded source, shared WIP and index are preserved. Receipt bytes remain under100 MB. Continue from desktop/dx12_surface.rs.


### Pelt desktop source and success-path review, 2026-10-17

Twenty more files are completely read (334,254 characters): DX12 surface import, Frisket retained chrome and original tests, single-document viewer and original route tests, scripted/Smolweb/Scrying glue, capture, platform smokes, profile/re-exports and five appearance scenarios. The first347 characters of workspace_viewer.rs are separately credited. Coverage is 2,872/3,738 files and 33,838,894/39,556,704 characters, with no stale or pending credits. The workspace host, accessibility caller, receipts and remaining repository still require reading. Fresh fetch found unchanged origin/main 00e4442868ee1cb014364a81ccaed657b512b218; local main remains 7c71810550234ccbe11f7429cd44e369b279ec06. The eight reviewed commits remain uninstalled because of the shared report overlap. The current upstream static/Smolweb diffs were reread; upstream adds request-aware fetch and KeyboardText forwarding, but all Q408-Q410 failure bodies remain unchanged there. No upstream build receipt is implied.

Q408: window creation and SurfaceHost startup failures log and exit without setting the error checked by run_headed_with, which returns Ok with no window, frames or requested product receipt. Q409: the basic NetRender smoke submits red but accepts both blank and wrong-color readback; its CLI only logs the returned count. Six focused CPU checks passed using exact extracted executable bodies with deterministic event-loop/window/SurfaceHost/readback doubles, including successful-startup and semantic-error controls. These qualify the failure propagation, not an actual native or GPU failure. Both exact run-owned exe/PDB outputs were hashed and removed. Q410 is source-confirmed only: Windows smoke ignores resize dimensions and keeps rendering/reporting the configured extent, unlike its current-size macOS/Wayland peers. No visual resize effect was measured.

A1216-A1225 distinguish actual shared owners from remaining adapters. Frisket uses shared Livery geometry/hits and Genet semantic projection; shell controls do not establish a merged engine accessibility tree. DOM rebuilds keep semantic focus but require tracing host publication and action identity. Pelt's controller is shared beneath the single-document shell, while the ordinary direct Smolweb viewer still differs from its controller-based forcing receipt in navigation/title ownership. Input decoders remain local in both hosts. Current local Livery explicitly inserts SessionKey::Character, ruling out the proposed ordinary typing omission; the progress update's suggestion that the workspace decoder was shared was incorrect. Updated upstream committed-text behavior needs its matching supplier, not inference from older source.

Native DX12 ownership, explicit handle closure, metadata checks and tearout transfer are real. Cache reuse still needs producer-instance/invalidation tracing, and taken pending fence waits need caller-error tracing; these remain source followups, not promoted bugs. Capture reads and presents the same owned target and refuses blank frames, but required-receipt finalization after early close/acquisition failure remains a host followup. Platform smokes already retain startup errors and validate display handoff; copied loops still drift. Appearance scenarios and real WebGL sample assertions were read, not rerun. No production edits, full desktop compilation, renderer/native input, font/scenario run, physical display or AT acceptance.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/pelt_desktop_review_receipt.json) and [source-body success probes](../../../support/receipts/authority-success-audit/pelt_desktop_success_probe_receipt.json). Prior report bytes, shared WIP, source and index are preserved. Continue workspace_viewer.rs at character347.


### Pelt workspace ownership and composite semantics review, 2026-10-17

The entire239,351-character workspace parent,50,614-character accessibility owner and15,180-character native appearance proof are now read. This adds304,798 characters after the previously credited347-character parent prefix. Coverage is 2,875/3,738 files and 34,143,692/39,556,704 characters, with no stale or pending credit. Receipt implementations and original desktop tests remain unread except targeted contextual excerpts; the repository audit is incomplete. Shared source/report/index guards match. Local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218; affected upstream workspace changes were reread and only concern newer committed keyboard-text forwarding. Q411/Q412 bodies and Q413 focus handler remain unchanged upstream; those commits are still uninstalled/unbuilt.

Q411: accepted concurrent legacy receipt configurations share one completion bool. A ready capability driver skips the requested interaction driver; interaction completion can skip tearout. Output flags and tearout finalization trust the same bool. Exact admission/driver guards executed with deterministic driver/event-loop doubles reproduce both shapes; pending/refusal controls distinguish them. Q412: both the secondary child helper and copied primary lowering drop supplied neutral accessibility descriptions while preserving role/name/value/actions. Exact complete lowering bodies over current real Inker/Pelt neutral types and AccessKit0.24.1 reproduce this; direct AccessKit description control retains the text. Seven source-extracted CPU checks pass in total. These are not whole desktop/original-suite/real-DOM/native/physical-AT receipts. Both exact run-owned test exe/PDB outputs were hashed and removed from the approved shared target.

Q413 is source-confirmed only: primary host Divider/Tab/Content gesture state survives Focused(false), even though core pool captures correctly retire. With an undelivered terminal release, later movement/release can continue the old chrome drag. No native focus/mouse/visual symptom was reproduced. The user-requested migration ledger remains alongside bugs: A1226-A1237 record shared engine registries with host policy, explicit hidden preflight/typed cache custody, source rollback/indeterminate rehost, and visible stopped-renderer alerts. They also record native tile/epoch cache identity versus producer/session lifetime, late backend/importer initialization and accepted secondary lane transitions as unresolved broader seams. These are documented rather than turned into speculative native findings.

The accessibility helper comment says primary and secondary share child lowering; primary actually contains a second complete loop. Both omit description. The hosts also duplicate semantic admission instead of consuming the core's complete owned publication boundary. Session/placement/revision/action checks and per-window namespaces are real controls, and the neutral revision contract must be respected before claiming a new stale-action bypass. The prior taken-fence-wait concern is narrower after caller reading: primary errors exit and accepted secondary errors stop producer composition; hidden-preflight rollback still needs a failure witness. Ordinary pointer and accessibility tab/close handlers both clear address editing, ruling out that initial retarget suspicion. Native appearance separates saved definition from explicit application activation and verifies held-session/history/aperture invariants in source; no scenario or physical font was rerun.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/pelt_workspace_review_receipt.json) and [CPU owner probes](../../../support/receipts/authority-success-audit/pelt_workspace_owner_probe_receipt.json). Prior report bytes and concurrent WIP are preserved. No production edits, isolated target/home/worktree or running audit process. Continue remaining receipt implementations and original desktop tests.


### Pelt desktop receipt/test completion and capture recovery review, 2026-10-17

All32 inventoried Pelt desktop files are now read (985,367 characters). This pass adds the receipt dispatcher, all four receipt modules and original140,493-character workspace test source:6 files,294,587 characters. Overall coverage is 2,881/3,738 files and 34,438,279/39,556,704 characters, with no stale/pending credit. Remaining Pelt has72 source/example/reftest/root files; repository completion remains unproven. Fresh fetch found unchanged upstream 00e4442868ee1cb014364a81ccaed657b512b218; local main remains 7c71810550234ccbe11f7429cd44e369b279ec06. The eight reviewed upstream commits remain uninstalled because the shared report overlaps, and no upstream build receipt is implied.

Q414: semantic assertion sets receipt_complete before capture acquisition. A failed acquire then gets neither timeout nor recovery redraw because both gate on semantic incompletion; after any prior presentation, the separate startup retry also refuses a wake. Final required-outcome checks only help if the event loop ends. Five CPU cases execute the exact complete timeout/acquire-recovery/startup bodies with deterministic unavailable-swap/statistics/event-loop doubles. Completed Appearance/Mixed have no retry/deadline/error, while pending controls get retry or expired error. These bodies are unchanged upstream. This proves the missing bounded recovery path, not an observed native hang; a later external event may wake the window. Both exact run-owned exe/PDB outputs were hashed and removed. No whole desktop build, original desktop suite, real native/GPU/input or physical AT ran.

A1238-A1245 tie receipt claims to their actual assertion boundaries. Reader source reuse has a real original fetch-count control (one article and one neighbor), and virtual Reader accessibility deliberately withholds link activation until host-held destination source exists. Theme tests compare held instance/history/aperture and preserve explicit activation after library edits; computed CSS and supplied2x geometry are not physical font/input evidence. Mixed native proofs require importer/wait/composition progress and a producer ready message, whereas no-Scry/non-Windows tests explicitly validate fallback. Original accessibility tests cover stale scroll/session actions, namespaces, foreign trees, protected edits, capture interference, nested geometry and sibling preservation, mitigating speculative general bypass claims. They still omit neutral child descriptions and the mixed-driver/chrome-focus/acquire-after-success compositions Q411-Q414.

The test composition closures call Frisket/workspace frame and mark-visible directly, bypassing SurfaceHost acquire/present/capture. A pending-preflight test merely constructs cache=None; it is not a native pending-surface receipt. AccessibilityClick's case2 assigns3 then the common increment advances to4, making its explicit Ready stage unreachable. Current normal frame promotion can still make the destination Ready, so this remains a named maintenance candidate rather than a fresh claim of accepted non-Ready output. Step machines and shared completion state remain tracked alongside ownership migrations.

Evidence: [checkpoint](../../../support/receipts/authority-success-audit/pelt_receipt_review_receipt.json) and [capture recovery CPU witnesses](../../../support/receipts/authority-success-audit/pelt_receipt_recovery_probe_receipt.json). Prior report bytes, guarded source/shared WIP and index are preserved. No production edits, new target/home/worktree or running audit process. Continue remaining Pelt sources at ports/pelt/examples/browser-loop/index.html.


### Pelt source completion and command-line owner review, 2026-10-17

All127 inventoried current-local Pelt files are now completely read (1,646,337 characters). This pass adds the remaining72 example/HTML/CSS/JavaScript fixtures, structural Scene/scroll expectations and root lib/main/viewer sources:214,666 characters. Overall coverage is 2,953/3,738 files and 34,652,945/39,556,704 characters, with no stale/pending credit. The repository audit remains incomplete. Local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218. The entire CLI file is unchanged upstream; previously reviewed Pelt request/committed-text changes in other files remain uninstalled/unbuilt because the shared report overlaps.

Q415: smoke handlers return before another requested receipt runs and before workspace artifact/feature admission. With an available successful smoke, named product/workspace modes can be skipped; two requested smokes run only the first. Q416: workspace JS selection uses js_engine==nova and silently maps every other value to Boa, unlike the shared single-document parser, which rejects unknown values and accepts case variants. Nine actual child processes execute exact source-extracted argument/early-dispatch/parser or workspace-choice bodies with deterministic engine-capability/host/smoke doubles. They demonstrate skipped owner dispatch and absent expected PNG, the correct earlier product-artifact refusal, later-dispatch reachability without smoke, and unknown/Boa/Vano-compatibility selection controls. Neither actual desktop nor JS engine ran. The initial probe-wrapper compile used an incorrect Option<String> return annotation, was corrected to its actual borrowed Option type, and its failed log is retained. No executable production body was changed. Both exact run-owned exe/PDB outputs were hashed and removed.

A1246-A1253 record the remaining host-policy authorities. Root run-mode and backend parsing are not yet unified; bare protocol dispatch still uses a local lowercase prefix list beside workspace routing, with stale P4 migration prose. That is a followup rather than an unexecuted uppercase-protocol failure claim. Ordinary application appearance paths remain caller-owned with explicit errors and separate choice/library storage. Actual CLI wrappers consume Ok without requiring an optional requested product receipt, making the Q408 library weakness relevant there. Fixtures explicitly distinguish forced redirected source, native-ready signalling, script timer/cancel controls, held Reader source and engine limits. All tracked Scene expectations were read, but not regenerated, compared or rasterized; placeholder font blobs and structural transforms are not current physical font/rendering evidence. Their external consumer/liveness remains to be traced before calling them obsolete.

The complete Pelt read does not close its native/ownership migrations. Prior actual core tests and this pass's extracted-body probes retain their exact scopes. No full desktop/original desktop suite, real native/GPU/physical input/AT or Boa/Vano execution was performed. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/pelt_completion_review_receipt.json) and [CLI dispatch probes](../../../support/receipts/authority-success-audit/pelt_cli_dispatch_probe_receipt.json). Prior report bytes, source/shared WIP and index are preserved; no production edits, isolated target/home/worktree or live audit process. Continue Signalman source.


### Signalman and Tabard port source review, 2026-10-17

All13 Signalman files (168,279 characters) and all17 Tabard desktop files (108,946 characters) are completely read. Overall coverage: 2,983/3,738 files and 34,930,170/39,556,704 characters, with no stale/pending credit. The prior truncated Signalman emission was discarded without acknowledgement and fully reread. Fresh fetch succeeded: local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218, eight previously reviewed upstream commits uninstalled because the shared report overlaps. The full audit remains incomplete, with755 source files still to read.

Q417: first-owner counter reservation has no serialization across the read/advance/sync/rename sequence. Two callers can both read0, then each successfully commit and return1. The unchanged-body probe uses actual filesystem writes, sync and rename, delaying only read returns to select this valid interleaving. This disproves the unqualified never-reoffered claim under concurrency; no actual board replay or power-cut test ran. Q418: host-side next_event holds its station mutex throughout the radio wait. stop and successful revoke close the lease, then wait behind that same operation without signalling it. Exact method bodies with real Tokio and a pending radio double both exceed the short virtual-time timeout; an idle-stop control completes. This is host cancellation evidence, not a physical RF cessation result.

A1254-A1263 record the ownership picture. Wallet signing stays in Djinn through a real trait seam, and Sennet/Tucket now share one sticky custody watch. Older controller, host-grant and unattended sealed-head lifetimes still differ. SitedStation and RunningSitedStationHead duplicate the station operation/watchdog machinery, with different cancellation paths. Queued authorization before mutex acquisition, change-watch subscription order, persistence rollback on closure, and multiple owners restoring/provisioning one record remain documented followups requiring broader concurrency/policy evidence. Control verification has real device/key/issuer/signature/expiry checks, and the head persists before returning an acknowledgement; those controls are not erased by the unresolved lifetime migration.

Tabard's desktop host uses WorkshopState and the shared SceneProducer directly, routes text/export/close decisions to existing product owners, and refuses incomplete scripted scenarios after the event loop. Its separate one-host and independent-two-host recovery qualifiers share thresholds but repeat capture/authority validation. The native example checks real owner/image/presentation/state/pixel identities and requires Done; its injected IME is programmatic input, and controlled Device::destroy is explicitly narrower than spontaneous driver loss. All scenarios and original tests were read, not executed. No current native/GPU/physical input/AT acceptance is claimed.

Four audit CPU cases pass. Two exact run-owned exe/PDB outputs were verified and removed; the initial cleanup lookup used legacy debug/deps, was corrected to Cargo's actual build/debug/deps, and is recorded. No production edit, separate target/home/worktree or live audit process. Shared reusable compiler dependencies remain. Prior report bytes, all protected sources/shared WIP and index are preserved. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/signalman_tabard_review_receipt.json) and [CPU probes](../../../support/receipts/authority-success-audit/signalman_owner_probe_receipt.json). Continue scripts and support suppliers.


### Root scripts and qualifier ownership review, 2026-10-17

The remaining20 root scripts are fully read (176,275 characters), completing all24 inventoried scripts. Overall coverage is 3,003/3,738 files and 35,106,445/39,556,704 characters, with no stale/pending credit.735 source files remain. Local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218; no production code changed.

Q419: the smoke driver's Assert-Local certifies registry and ssh:// Git sources as local because it checks only for the rendered substring (https://. Four executions of the exact PowerShell function reproduce those false positives, with HTTPS-refusal and path-acceptance controls. Q420: Firefox metadata matching only searches four positions around the time pivot, dropping a valid fifth unclaimed visit inside tolerance. Three direct unmodified-exporter calls against synthetic in-memory SQLite reproduce it and retain ordinary/out-of-tolerance controls. Q421: both Wayland and X11 wrappers call Mere/scripts/remote-receipt.ps1, which is absent; their helper remains in Genet, while host/package/scenario ownership moved to Mere. This last finding is source/membership evidence, not an attempted remote run.

A1264-A1273 track extraction and independent authority drift. Cross-repo smoke still asks Genet to build Pelt; Windows receipt manifests label Mere HEAD as repo=genet. Native process selection is title/time-based, X11 shares fixed tmp paths, Snap success checks hit testing and saves a screenshot without testing popup visibility. These remain weaker witnesses than a bound process/image/platform result. DR-C has real refusal/mutant/source controls but overwriteable fixed logs, old output-path assumptions and no ordinary subprocess deadline. It was not executed. SSH setup/custody client, toolchain helper and LaunchServices launcher remain operational plumbing; none were used to change the system or contact another host.

The documentation detectors check path/header shape and judgment-record coverage rather than semantic truth. They share the header scanner correctly. Study rule parity checker and asset packer have separate source/binary and mutation boundaries; the support study source is next. Four original Python tests pass (one exporter, three judgment), in addition to the seven targeted classifier/exporter cases. No actual browser profile, cross-repo build, native window, physical input/AT, SSH or system setup ran. No new binary, isolated target/home/worktree or live audit process remains. Report prefix, all protected source/shared work and index are preserved. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/scripts_review_receipt.json) and [script probes](../../../support/receipts/authority-success-audit/script_owner_probe_receipt.json).


### Support studies, custody qualifiers and updater comparison review, 2026-10-17

All13 files in this slice are fully read (133,925 characters): intake attributes, complete scene study/embedded payloads, backdrop helpers, three Dramatis drivers and six standalone updater-comparison sources/manifests. Coverage is 3,016/3,738 files and 35,240,370/39,556,704 characters.722 source files remain. The entire live3,738-file inventory was reconciled against tracked/untracked source membership and every source SHA256; no additions, removals or changes. This verifies current evidence, without crediting unread source.

Q422: the package qualifier selects an exact ignored tamper test but accepts exit0 without requiring that test to execute. Its unchanged run helper accepts a synthetic0-tests log; the nonzero control refuses. Q423: the review qualifier edits production consent source in place and blindly restores saved bytes in finally. Exact AST-function probes with an entirely in-memory source show an intervening owner edit overwritten while its restoration assertion passes; no-conflict restoration control passes. The audit did not mutate product source, compile a consent bypass, sign/extract an installer, launch native dialog or operate a vault. These probes establish runner admission/custody flaws, not that a retained real package ran zero tests or any actual concurrent edit was lost.

A1274-A1281 keep intentional studies and historical evidence distinct from product authorities. Browser presentation rules are a local JS model with a Rust parity seam, and the force solver explicitly identifies itself as a bounded study. Occurrence/resource/pin distinctions are visible, while widget persistence is optional/size-limited and failures suppressed. The PNG/overlay helpers can overwrite outputs or update consumer locks and were not run. Native consent uses a synthetic process-owned dialog and programmatic button messages; source mutation belongs to the separate review-driver risk. The Luggage/Velopack/TUF comparison explicitly verifies dated known gaps and source/evidence hashes; it must not be called current safe-updater acceptance after product source changes. No comparison pipeline ran.

Four extracted-function CPU cases pass with explicit process/source doubles. All protected source/shared work, report prefix and index are preserved. No binary, isolated target/home/worktree or live audit process remains; retained evidence stays below100MB. Local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/support_qualifier_review_receipt.json) and [qualifier probes](../../../support/receipts/authority-success-audit/support_driver_probe_receipt.json). Continue supplier patches/source, starting burn-cubecl.


### Retained Burn backend source review, 2026-10-17

All119 files in support/patches/burn-cubecl are fully read (695,238 characters). Coverage is 3,135/3,738 files and 35,935,608/39,556,704 characters;603 source files remain. Full live source membership and all3,738 source hashes remain unchanged. This inventory verification awards no unread-source credit.

Q424: pointwise convolution flattening accepts a batch gap that changes addresses from[0,1,4,5] to[0,1,2,3]. Q425: the direct2D transpose-convolution integer bounds omit a valid origin contribution for a unit kernel with stride3. Q426: zero-inner-dimension matmul returns a nonempty supplied output untouched rather than zeroing its empty dot products.18 CPU cases pass:6 original registry tests,4 layout predicate/address cases,3 extracted range cases and5 extracted early-return cases. The latter use explicit scalar/tensor/allocator doubles. No CubeCL lowering, GPU execution, full backend build or actual uninitialized-device read occurred.

These are retained-copy reactivation hazards. Root selects registry burn-cubecl0.22.0, while this standalone archived copy is0.22.0-pre.4 and its path selector is retired. Current registry failures are not established by these tests. Root also selects registry cubecl-wgpu/cubek-reduce, but independent Distillery probes still select the retained reduce patch. A1282-A1297 track this consumer distinction plus placement identities, callback/lock lifetime, layout admission, repeated convolution bounds/tuning, quantized allocation offsets, reduction identities, transaction placement and source-cache identity. The template wrapper comment claims template text enters its cache identity; that is not visible in its forwarded ID and remains a runtime-cache followup. Shared reshape/fallback/error propagation and explicit unsupported cases are recorded alongside risks, without converting source-read tests into executed receipts.

All prior report bytes, protected source/shared work and Git index are preserved. Four exact owned executable/PDB outputs were verified and removed across the two probes. No audit process, isolated target/home/worktree or GPU/native run remains. Evidence stays below100MB. Local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/retained_burn_review_receipt.json), [layout/range probe](../../../support/receipts/authority-success-audit/retained_burn_layout_probe_receipt.json) and [matmul probe](../../../support/receipts/authority-success-audit/retained_burn_zero_matmul_probe_receipt.json). Continue selected burn-remote source.


Retained Burn checkpoint reference correction: seven authority-map references used file-shaped shorthand for directory modules. A1285/A1287/A1289/A1290 now point to the actual attention, forward, backward_data, backward_weight, grid_sample, fft and cast directories. Reading hashes, coverage and findings are unchanged. The original checkpoint and its cited evidence remain byte-preserved.


### Selected Burn remote client and session ownership review, 2026-10-17

The39 complete files in this slice cover both manifests, client, service/batching/response correlation, server pump/session/worker, wire definitions and telemetry. An additional2,038 characters of transport/error.rs are read, with its remainder explicitly pending. This slice is311,447 characters. Overall coverage is 3,174/3,738 files and 36,247,055/39,556,704 characters;564 files remain incomplete. All3,738 live source membership/hashes are unchanged. Burn remote remains root-selected; its transport and integration test source review is unfinished.

Q427: the server-survival test catches failed read/value assertion panics and still returns a passing completion outcome. Exact helper CPU probes distinguish panic, successful completion and bounded blocking; the actual server was not run. A separately duplicated helper in lease-close tests intentionally checks only absence of a hang, so its error/panic outcome is not called a correctness witness.

Q428: process-local transfer counters feed a shared server map keyed only by that id. Two independent counter states both issue0; the unchanged registry on actual Tokio lets a second session overwrite the first payload, then returns that second payload for the first id. Distinct-id and source-purge controls pass. Tensor payload/backend and session identity are explicit CPU doubles, and the two statics model separate processes. No real cross-process/server/GPU/credential disclosure trial occurred. The source handlers supply no additional namespace/participant check. A missing second payload can also leave transfer/reclaim waiting; that full-session effect remains to qualify. Repair needs a shared transfer/participant contract and failure lifetime, so it is documented as an ownership issue without changing production source.

A1298-A1308 track the real thin-client/service/task split, endpoint/credential identity, response disconnect protection, shared admission pump, teardown acknowledgement and projection boundaries. Native sync awaits its response; browser sync returns flush success immediately and profiling explicitly refuses. SessionManager prose still claims per-stream workers although the actual worker is one global FIFO. Telemetry may precede execution or target download, can lose events, and estimates payload savings rather than observed wire delivery. None of those views should substitute for a confirmed execution or reclaim receipt. Source review narrowed two suspicions: registry admission bounds compact IDs, and duplicate session reservation prevents two successful simultaneous bindings with one id.

Twelve CPU cases pass, including6 unchanged PendingResponses tests using actual existing Tokio. Two owned executable/PDB outputs were verified and removed. Prior report bytes, protected production/shared files and index remain preserved; no audit process, isolated target/home/worktree or live server remains. Retained evidence is below100MB. Local/upstream remain 7c71810550234ccbe11f7429cd44e369b279ec06/00e4442868ee1cb014364a81ccaed657b512b218. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/remote_session_review_receipt.json) and [CPU ownership probes](../../../support/receipts/authority-success-audit/remote_client_ownership_probe_receipt.json). Continue at transport/error.rs character2,038.


### Burn remote transport and integration source completion, 2026-10-17

All58 Burn remote files are now fully read (433,101 characters). This continuation completes19 files, adding121,654 characters after the preceding partial transport checkpoint. Overall coverage is 3,193/3,738 files and 36,368,709/39,556,704 characters.545 source files remain. Full live membership and all3,738 source hashes are unchanged.

A1309-A1315 distinguish real consolidation from overclaims: Iroh/WebSocket both use the shared session pump, but transport cfgs remain outside the transport module despite its cfg-free prose. Endpoint/node constructor scopes retain separate pools; owner cancellation starts shutdown while targeted close supplies backend-release acknowledgement. Iroh transfers use full capability/target checks and bounded rendezvous/expiry; WebSocket uses64-bit keys, ignores target and only logs pre-exposure failure. Frame length narrowing before bounds on32-bit, silent headers/handshakes without deadlines, duplicate capability/expiry generations and normal-close response drain remain source-qualified followups. None was promoted to a tested deployment failure.

Integration source includes direct/relay/app-owned endpoint, authorization, retry, port rebind, fusion/custom source/clone/upload-drop, multi-device/concurrent/local crossing and profile/lifetime checks. Their same-process concurrent transfer stress does not exercise Q428 independent-client counter collision. Iroh panic-propagating watchdog is stronger than Q427 helper; WebSocket panic watchdog fails but can mislabel the cause as deadlock. IPv6 tests return early on unsupported hosts, so green test status alone is not IPv6 evidence. No integration test, live relay/server/network, signal or GPU run occurred here; the earlier12 CPU witnesses retain their exact qualifications.

All prior report bytes, protected production/shared files and Git index remain unchanged except this append. Evidence remains below100MB, with no owned process, binary or isolation retained. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/remote_transport_review_receipt.json). Continue selected cubecl-runtime source without narrowing the full audit denominator.


### Selected CubeCL runtime client, configuration and scope review, 2026-10-17

This slice reads39 complete runtime files and22,239 characters of server/base.rs:287,578 characters total. It covers manifests/build cfg, the complete client/lazy controller, compiler/kernel/id/runtime contracts, all configuration/devices, dry-run/launch collection/observer/profiling, memory handles/guards/reports/policy and poison. Overall coverage is 3,232/3,738 files and 36,656,287/39,556,704 characters.506 source files remain incomplete, including32 runtime files. All3,738 live source hashes and membership are unchanged.

Q429: allocation_window uses ambient StreamId.current while normal client allocations honor a pinned stream. Exact-function CPU doubles show mode transitions on stream1 while the pinned client's stream2 stays Auto; an unwinding callback also skips the closing transition. Q430: RealRun is implicitly Send despite owning TLS depth. Actual std-thread tests show a foreign drop leaves the origin executing inside DryRun, and can disable the destination's own still-live guard. Q431: observer documentation promises guards dropped in taken order restore the process, but exact code loses active B when A drops first, then resurrects ended A when B drops. Reverse-order control behaves as intended. Eleven CPU cases pass, including4 unchanged dry-run controls. Allocation and observer cases use explicit device/stream/lock/trait doubles; there is no actual GPU allocation, profiling run or full runtime build.

A1316-A1327 track real boundaries alongside gaps: full ServiceId checks survive client erasure; graph enqueue acceptance, named-buffer trust and device completion are explicitly different. Dry-run verdict is issued once before actor handoff. Observers report issuance and omit graph replay; missing timing stays absent, observer-only profile refusal preserves computation, and logger timing can change overlap/method semantics. Configuration/reporting delegate shared environment and layout contracts, while mode/TLS/observer lifetimes remain unfinished seams. MemoryLayoutPolicy promises a single binding across combined layouts, resolving the retained quantization intention at contract level. Kernel source identity is explicitly author-owned: retained template-wrapper invalidation prose cannot substitute for an id hashing its text. Root's pristine-upstream claim was not independently archive-compared and is not promoted into audit provenance.

Prior report bytes, protected source/shared work and index are preserved. Two owned executable/PDB outputs were verified and removed; no audit process or isolation remains. Evidence stays below100MB. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/runtime_client_review_receipt.json) and [scope probes](../../../support/receipts/authority-success-audit/runtime_scope_probe_receipt.json). Continue server/base.rs at character22,239; the full runtime/repository review remains open.


### Runtime Server contract completion, 2026-10-17

The entire82,411-character server/base.rs is now read, adding60,172 characters beyond the prior partial. Overall coverage is 3,233/3,738 files and 36,716,459/39,556,704 characters;505 files remain incomplete. All3,738 live source membership/hashes remain unchanged. A1328-A1332 record recursive refusal/poison classification, the shared IO visibility fallback chain, graph lifecycle defaults and the explicitly unfinished collective failure path. A refused send can leave its posted peer receive waiting; the contract names that cross-device liveness gap instead of claiming successful propagation. IO reclaim classification and dispatch capacity admission remain source followups. Existing argument/error/dispatch tests were read, not executed.

This extends the prior checkpoint; Q429-Q431 and their11 scoped CPU cases retain their original qualifications. Prior report bytes, protected shared source and index are preserved. No additional binary/process/isolation or production edit; evidence stays below100MB. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/runtime_server_contract_review_receipt.json). Continue server/capture.rs; full-runtime and full-repository completion remain unproven.


### Runtime capture, storage, throughput and initial tuning-plan review, 2026-10-17

All25 files in this slice are fully read (219,281 characters). Runtime coverage is65/71 files, leaving6 driver/cache/input/helper files83,499 characters. Overall coverage is 3,258/3,738 files and 36,935,740/39,556,704 characters;480 source files remain. Full live3,738-file membership/hashes remain unchanged.

Q432: roofline time_at_peak admits a negative normal peak and panics despite promising None; tiny positive finite peaks can overflow Duration. Separate scoring can select negative/infinite-peak entries the docs say are skipped. Q433: rank divides by iterations asU32; a64-bit2^32 count panics, and an uncapped retargeted20-billion count reports7ns instead of1ns in the deterministic callback probe. Q434: candidate names are concatenated without boundaries/count before hashing, so[ab,c], [a,bc] and[abc] share the exact checksum input. These are public helper/encoding witnesses, not an observed GPU ceiling error, deployed stale-cache execution or current concrete caller reachability claim.

Twenty CPU cases pass:8 unchanged roofline controls,5 additional invalid-rate cases,4 exact rank/retarget cases with explicit duration callbacks, and3 unchanged checksum-accumulation-prefix cases. The latter inspect identical hasher input; the StableHasher crate was not replaced or run. Four exact owned executable/PDB outputs were verified and removed. No runtime/backend/GPU build, live cache load, server or integration run occurred.

A1333-A1344 track the shared capture state and region claim key, binding-backed resource lifetime, part/capacity/probe-version throughput identity, curve/model limits, group IDs versus labels, device-scoped set building versus id-scoped winner state, eviction/input ownership and separate decision/record projections. Capture publication is a logical state receipt; backend transition success still requires completion/abort. The tuning logger explicitly describes an all-failed unmeasured-winner path, which is the next driver source to examine. Shared ResourceBound arithmetic exists, but peak and threshold validity are incomplete in different ways. Remaining source suspicions stay distinct from the reproduced helper cases.

Prior report bytes, protected production/shared source and Git index remain preserved; retained evidence is below100MB. No audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/runtime_throughput_review_receipt.json), [numeric probe](../../../support/receipts/authority-success-audit/runtime_throughput_probe_receipt.json) and [checksum probe](../../../support/receipts/authority-success-audit/runtime_checksum_probe_receipt.json). Continue tune/schedule.rs. The full-source objective remains active and incomplete.


### Runtime tuning completion and retained WGPU backend/compiler review, 2026-10-17

Read another 24 complete files, 287,539 characters. All 71 CubeCL runtime source files are now read (650,530 characters). The retired WGPU copy is read through its shader formatter: 18/33 files, 204,040/313,218 characters. Overall coverage is 3,282/3,738 files and 37,223,279/39,556,704 characters. 456 files remain. Full live 3,738-file membership and hashes still match the inventory.

Q435: native adaptive tuning maps failure to acquire exclusive access to Unknown for every candidate, then treats that category as executed-but-unmeasured. Exact scheduling and commit bodies with a refusing CPU client enter the driver zero times yet produce a memory Hit and settled winner. A later actual operation execution cannot justify that earlier evidence claim. Source explicitly suppresses disk storage for unmeasured decisions, so this is not a demonstrated disk-cache defect. Missing-timestamp and fixed-strategy all-failed paths use different failure policies.

Q436-Q439 concern the retained, root-unselected cubecl-wgpu 0.10 implementation. Its 8-bit Vulkan storage predicate reads 16-bit features; helper discovery ignores switch scopes; f64 NaN/Infinity masks are swapped; Vec4 indexed assignment omits its terminator. Current root Cargo.lock selects registry cubecl-wgpu 0.11, and these findings are not promoted to current deployed failures. Exact predicates, full formatter/helper source and scoped surrounding doubles provide CPU witnesses. No actual Vulkan query, Naga/parser validation, current concrete kernel caller or GPU execution occurred.

Fifteen CPU cases pass: three exact scheduling/cache-commit cases and twelve retained-backend predicate, helper-discovery, bit-classification and emitted-string cases. Four owned executable/PDB outputs were verified and removed. A1345-A1361 track survivor selection versus result scoring, process-local fallback evidence, checksum/index admission, environment/hydration ownership, Pending cancellation, logging-gated correctness checks, anchor configuration lifetime, retired backend authority, browser validation evidence, native feature projections, compiler lifetime and metadata layout dependencies. Q434 now has a source trace through persisted checksum admission, still without a real Store-load witness.

Prior report bytes, protected source/shared files and the Git index are preserved. All current source hashes remain unchanged, and retained receipts remain below 100 MB. No production edits, audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/runtime_tuner_wgpu_review_receipt.json), [tuner probe](../../../support/receipts/authority-success-audit/runtime_tuner_probe_receipt.json) and [retained backend probe](../../../support/receipts/authority-success-audit/retained_wgpu_probe_receipt.json). Continue subgroup.rs, then the remaining backend runtime/server/storage/device/tests. The full-source objective remains active and incomplete.


### Retained WGPU completion and selected matmul definition/evaluation review, 2026-10-17

Read another 181,107 characters: 35 complete files and the first 517 characters of matmul multi_level/args.rs. All 33 retained WGPU files (313,218 characters) are now read. Twenty of 317 selected matmul files are complete. Overall coverage is 3,317/3,738 files and 37,404,386/39,556,704 characters; 421 files remain. All current source membership/hashes remain unchanged.

Q440-Q443 concern the retired, root-unselected WGPU0.10 copy: write-only flush succeeds without submitting its queued tail; mixed-adapter enumeration assigns global ordinals that type-filtered selection misinterprets; imported setups advertise adapter capabilities beyond the enabled Device; Existing(u32) IDs narrow to16 bits long before their overflow guard. Seven exact-body CPU cases qualify queue/enumeration/encoding behavior. The imported-device issue is a source/primary-API-contract finding only. No actual GPU, driver enumeration, imported low-feature device or real sync callback was exercised. The concrete server constructs a fresh compiler for each uncached kernel, ruling out A1360's suspected state leak on that path.

Q444-Q446 concern the selected matmul0.3 source and its feature-gated evidence harness. Shared broadcast_batches turns zero with singleton into one, unlike installed NumPy2.4.3. Benchmark preparation swaps strides without changing rectangular logical shapes, causing out-of-allocation views or aliasing; correctness preparation uses a separate layout-aware builder. The CPU reference uses output rank to index every operand and panics on valid rank-mismatched broadcast inputs. Thirteen exact-body CPU cases include ordinary/rank/layout controls and cached shared host-index/stride functions. No actual TestInput, kernel launch, GPU access, benchmark timing or full crate build occurred.

A1362-A1381 record compiler lifetime, native handle versus managed region ownership, per-stream pools/common queue, readback controller cleanup, polling-token lifetime, queue submission versus completion, no-work timing, imported-device authority, device ID/index spaces, selected matmul architecture vocabulary, accumulator presence, precision mappings, cost projections, vector admission and the split benchmark/reference preparation. Unfinished migration details stay alongside individual bugs rather than being replaced by file-splitting recommendations.

Twenty scoped CPU cases pass. Four exact owned executable/PDB outputs were verified and removed. Prior report bytes, source/shared files and Git index are preserved; retained receipts remain below100MB. No production edit, audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/wgpu_completion_matmul_review_receipt.json), [retained completion probe](../../../support/receipts/authority-success-audit/retained_wgpu_completion_probe_receipt.json) and [matmul definition probe](../../../support/receipts/authority-success-audit/matmul_definition_probe_receipt.json). Continue multi_level/args.rs at character517. Full-source review remains active and incomplete.


### Selected matmul argument, batch and global multi-stage review, 2026-10-17

Completed another46 files, reading230,863 new characters. Matmul coverage is66/317 files,302,792/1,590,560 characters. Overall coverage is 3,363/3,738 files and 37,635,249/39,556,704 characters;375 files remain. All live3,738-file membership and hashes still match the inventory.

Q447: shared BatchLayoutLaunch combines output-rank batch divisors with only the operand's existing batch strides. Missing leading batch axes are not padded with zero strides. Exact host payloads for unbatched/shorter-rank operands therefore have unequal list lengths; a scalar CPU interpretation of the layout body bounds-panics when it indexes them. Four same-rank/unbatched controls map correctly. The layout body's one unroll attribute was removed for interpretation, and Sequence/FastDivmod/generated DTOs are explicit doubles. This proves the host mapping mismatch, not a particular CubeDSL lowering, launcher acceptance or GPU failure mode. It extends Q446's shape migration finding into the shared launch-layout boundary.

A1382-A1400 track shared MatmulArgs versus concrete/fused/TMA factories, batch and packing identity, manually mirrored TMA layouts, delegated bounds, separate numerical fallback families, shared partition execution, accumulator/writer footprint ownership, copied GlobalConfig assembly, recoverable setup versus asynchronous compilation and barrier/event completion. Explicit accumulator-read accounting is present in the partitioned shared-memory budget. The apparent GEMM dropped-K-tail issue is ruled out for validated family launches; its family checks divisibility. Normal GEMV layout labels do not by themselves transpose physical indexing because packing is1 and supplied strides own addressing.

Six scoped CPU cases pass; two owned executable/PDB outputs were verified and removed. Existing source tests were read but are not reported as executed. No full crate/CubeDSL build, actual numerical matmul, GPU, barrier, reader or launcher run occurred. K0 loop admission, TMA broadcast/packing, accumulator loader validation and caller propagation, actual writer clamp, specialized-role consistency and shared reader behavior remain named followups.

Prior report bytes, protected source/shared files and Git index are preserved. Retained evidence stays below100MB. No production edits, audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/matmul_batch_global_review_receipt.json) and [batch-layout probe](../../../support/receipts/authority-success-audit/matmul_batch_layout_probe_receipt.json). Continue global/read/layout/mod.rs. Full-source objective remains active and incomplete.


### Selected matmul readers, components and blueprint-definition review, 2026-10-17

Read another62 complete files,267,447 characters. All multi-level component source and blueprint definitions are now read. Matmul coverage is128/317 files,570,239/1,590,560 characters. Overall coverage is 3,425/3,738 files and 37,902,696/39,556,704 characters;313 files remain. Full live3,738-file membership and source hashes remain unchanged.

Q448: sync/async partial cyclic readers each copy the same strict-workload off-by-one. A final vector start equal to the stage length is admitted because the check uses greater-than; full cyclic readers reject the same imbalance by divisibility. Eight numeric cases include balanced and further-overrun controls. Two additional cases show the ordered loader validator alone accepts MainOnly and LoadOnly flow with the same dimensions, while its kernel separately rejects specialized lhs. Other tiling/swizzle/capability checks and config methods are explicit doubles, so these tests prove the named predicates, not complete routine admission, CubeDSL lowering or an actual device fault.

A1401-A1420 track duplicated full/partial reader jobs and workload math, unused SHOULD_CLEAR, separate synchronous/async-copy/TMA completion protocols, required config/problem validation, actual role vocabulary re-exports, shared writer allocation, variant capability versus full admission, explicit register-fragment initialization responsibility and blueprint/tiling ownership. The stage selector's claim that every method is a pure dispatcher is too broad: its common constructor assembles all memory configs. Shared vocabulary does not erase the independently maintained arithmetic and admission implementations.

Source resolves three suspicions: TMA explicitly refuses mixed partial batch broadcasting when problem validation runs; actual writer allocation applies the budget's one-tile clamp; default stage construction keeps shared/global output vector widths equal, avoiding the suspicious wider-output gather branch on that path. The cooperative clear policy is dead surface with no identifier consumers, but no stale-data numerical failure is asserted without copy-intrinsic evidence.

Ten scoped CPU cases pass. Two exact owned executable/PDB outputs were verified and removed. Source tests are read, not claimed executed. No actual GPU, complete blueprint acceptance, algorithm/timestamp/barrier test or full crate build occurred. Previous report bytes, protected source/shared files and Git index remain preserved; evidence stays below100MB. No production edit, audit process or isolation remains. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/matmul_readers_components_review_receipt.json) and [reader-validator probe](../../../support/receipts/authority-success-audit/matmul_reader_validation_probe_receipt.json). Continue multi_level/eval/gemm.rs. The full-source objective remains active and incomplete.


### Selected matmul routines, stage memory and tile-operation review, 2026-10-17

Read another72 complete files,367,690 characters. Overall coverage is3,497/3,738 source files and38,270,386/39,556,704 characters;241 files remain. Matmul is200/317 files,937,929/1,590,560 characters. All3,738 working source membership and hashes remain unchanged.

Q449: DoubleVecMat shares a single-stage selector helper and omits actual buffering depth. StageK32/K96 disables K bounds, then the double-stage group validator rejects it. Q450: specialized GEMM/vector launchers still index both operands with one operand's rank; valid rank-broadcast operands panic or produce wrong host dimensions. Generic batch construction already uses each operand's rank. Strategy catches InvalidConfig and can fall back to SimpleUnit, but these Rust panics bypass it; reverse-rank misread is not claimed to be a wrong device result. Ten CPU cases compile exact relevant host fragments using explicit scalar/config/binding doubles, with equal-rank, even-group and explicit-double-bounds controls. Whole launchers, all validation and GPU execution were not run.

Q451 is experimental: Interleaved's always-true support hint makes its multirow selector always choose tileK16, rejected at plane32/64 by its separate validator. Interleaved lives in TestStrategy and is excluded from production Strategy. Actual code requires plane_sum despite a permissive type-admission flag, and accumulator load is explicitly unimplemented. Source evidence only, not a deployed default failure.

A1421-A1445 track copied evaluation preparation, stage-count/own-rank migrations, repeated routine orchestration, stale specialized heuristic, unit/plane swizzle policy, logical versus rounded memory accounting, explicit alias/free lifetimes, partial tiling admission, requested-strategy versus fallback-kernel identity, and centralized tile dispatch around independently maintained implementations. CMMA/MMA absent-accumulator copy explicitly initializes zero, resolving that suspicion for those two variants. Other software variants remain to read. Unit selector's never-over-request guarantee is broader than fit_unit_tiling: it can return an oversized minimal scheme for later launch validation to reject.

Fetched origin/main from00e44428 toa9b5dd66, eight commits. Read all six changed source diffs and four documentation diffs from Git objects, separately from working-source coverage. HTTP now reports actual completed-response provenance and keeps reload acquisition metadata request-scoped; transport facts remain separate from host retention/Keep/save. Document and element scroll extent now share an axis-aware clipping walk; optional host observation preserves missing layout as unavailable. Supplier receipts report34+12 HTTP/PageLoad passes and315 Rootstock/native-host passes with8 ignores; these were read, not rerun here. Woodshed receiving evidence retains its own owner. Existing shared report overlap prevented the earlier ff-only pull; no merge/stash/reset was attempted around it.

Previous report bytes, protected source/shared files and index are preserved; evidence remains below100MB. No production edits, live audit process, retained owned binaries or isolated outputs. Evidence: [checkpoint](../../../support/receipts/authority-success-audit/matmul_routines_tiles_review_receipt.json), [ten scoped host cases](../../../support/receipts/authority-success-audit/matmul_routine_contract_probe_receipt.json), [new upstream review](../../../support/receipts/authority-success-audit/upstream_a9b5dd66_review.json). Continue at multi_level/tile/variants/instruction/mma/writer.rs. Full-source objective remains active and incomplete.


### Remaining multi-level tile variants and first tiled routines, 2026-10-17

Read another27 complete files,240,926 characters. Overall coverage is3,524/3,738 files and38,511,312/39,556,704 characters;214 files remain. Matmul is227/317 files,1,178,855/1,590,560 characters. All3,738 working-source membership/hashes remain unchanged; HEAD7c718105 and fetched origin/maina9b5dd66 remain distinct as recorded above.

Q452: the vectorized transposed UnitTile loader uses a vector index as an element index, overwrites cells and leaves holes, and checks divisibility of the wrong axis. Four scoped CPU loop cases include vector1/single-vector controls. No explicit current true constructor was found, so helper defect and product reachability stay separate. Q453: split-cube verify accepts NaN against a finite expected result; four exact f32 predicate cases include finite/infinity controls. Neither probe executes CubeDSL or GPU code. One initial probe compile failed because the audit's comptime double omitted usize; that source/log is retained and the corrected harness passes.

Q454: atomic split-cube timings reuse a destination zeroed only once, across warmups and samples. The independent verification Bound runs once. Selected benchmark driver prepares once, then repeats profile/execute on the same args; Bound.prepare is empty. The measured additive kernel remains a raw timing, but does not include the initialization needed for the stated standalone c=AB comparison. Source-chain evidence only; no new atomic/device run or physical overflow claim.

A1446-A1463 track duplicated typed/erased stage-address math, erased vector contracts, independent register/plane/hardware I/O, copied partition event loops and four softmax protocols, mask-layout copies, root Auto versus specialized fallback errors, explicit storage/plan/delivery admission and independently stated grids. Register and PlaneVec absent accumulators explicitly initialize zero. Tiled CMMA and CPU use a shared geometry/accumulator DSL while retaining separate planning/type/setup copies; that coexistence is not one mathematical or device authority. MatrixBinding and remaining tiled source still need reading before judging apparent K/output-shape omissions. CMMA fallback's first-match-then-fit filter and storage proof accepting Skipped beside Pass remain named followups.

Eight scoped CPU cases pass. Audit-owned executables/PDBs were verified and removed; no process or isolated target/home/worktree remains. The stable existing Mere target was reused. Previous report prefix, protected shared/source files and Git index remain preserved; evidence remains below100MB. [Checkpoint](../../../support/receipts/authority-success-audit/matmul_variants_tiled_review_receipt.json), [UnitTile witness](../../../support/receipts/authority-success-audit/matmul_unit_transpose_probe_receipt.json), [NaN predicate witness](../../../support/receipts/authority-success-audit/matmul_split_verify_probe_receipt.json). Continue tiled/eval/split_k/mod.rs. Full-source objective remains active and incomplete.


### Matmul key, remaining tiled production and test ownership review, 2026-10-17

Read57 complete files,278,370 characters. All inventoried matmul production source is now read;33 matmul test files remain. Matmul is284/317 files,1,457,225/1,590,560 characters. Overall coverage is3,581/3,738 files and38,789,682/39,556,704 characters;157 files remain. All3,738 working source membership and hashes remain unchanged.

Q455: tuning-key dimensions still use LHS rank to read RHS N. Valid asymmetric-rank operands panic or produce wrong N before a key is constructed. Four exact host-fragment cases include equal-rank controls. Selected Burn caller forwards actual metadata into this key, but public frontend normalization is not fully traced; no end-to-end Tensor failure is claimed. This joins the unfinished own-rank migration already recorded in specialized launchers and CPU reference.

Q456: broadcast regression helper silently skips a numerically wrong baseline, not just an unsupported backend/layout. Its local GEMM counterpart copies that policy. Four exact-helper cases with explicit outcome/client doubles verify wrong-baseline skip and passing/failing broadcast controls. Q457: optional full-tier source still calls the pre-migration three-argument blueprint builder and imports a test-only launcher from a harness that no longer exports it. This is a static source contract mismatch, not a fresh full Cargo compiler receipt. The test include-tree remainder stays in scope.

A1464-A1483 track shared MatrixBinding/Metadata storage authority, direct QuantGemv versus Strategy exposure, scale/input contracts, one bidirectional storage relayout, separate key reconstruction/bucketing, poisoned-output reference harness, baseline outcome policy, one-shape smoke coverage, bias's narrow SimpleUnit receipt, stale comparison claims and duplicated full/extended test helpers. Split-K repeats Q453's NaN-blind predicate. QuantGemv explicitly zeroes output within every call; its reset lifetime is distinct from Q454 atomic benchmark. Interleaved inference exclusion is already acknowledged in test comments, so Q451 is a retained known tail. Historical performance comments and a green optional test label do not imply fresh numerical execution.

Eight scoped CPU cases pass; owned executable/PDB outputs were verified and removed. No live audit process, isolated target/home/worktree or production edit remains. Existing stable Mere target reused. Shared report prefix/source/index preserved; evidence remains below100MB. [Checkpoint](../../../support/receipts/authority-success-audit/matmul_key_test_review_receipt.json) and [key/broadcast witness](../../../support/receipts/authority-success-audit/matmul_key_broadcast_probe_receipt.json). Continue tests/multi_level/full/plane_accelerated/precision.rs. Full-source objective remains active and incomplete.


### Entire retained matmul source read, 2026-10-17

Read the remaining33 matmul test files,133,335 characters. All317 retained matmul source/manifests/tests are now fully read,1,590,560 characters. Overall3,614/3,738 files and38,923,017/39,556,704 characters are read;124 files remain. All3,738 live working-source membership and hashes remain unchanged.

A1484-A1494 record the remaining test ownership/evidence boundaries. Q457's static migration mismatch also appears in full unit harness imports and the full TMA MMA arm's missing MultiLevel import. Byte-identical per-family precision/stage scaffolding still surrounds shared includes. The CPU layout laboratory has a named possible stale positive case:4x4 stored inputs with a3x3 leaf conflict with current cut_by admission; actual shared Partitioning/CPU qualification is still required before reporting an executed failure. Inferred-heuristic test comments also lag current register-budget/divisor policy, so their names do not prove the claimed overhang coverage.

Newer direct QuantGemv and accumulator-precision tests use <=/< assertions that reject NaN, unlike Q453 benchmark gates. Storage tests have independent every-cell and packed-field oracles. Their source is read, not claimed freshly run. TMA refusal-versus-unavailability and output-reset contracts remain scoped; ignored timing studies and conditional CPU tests do not constitute cross-backend execution receipts.

This checkpoint adds no product edit or new execution claim. Report prefix, source/shared guards and index remain preserved, evidence below100MB, no live process or retained isolated build/home/worktree. [Checkpoint](../../../support/receipts/authority-success-audit/matmul_complete_review_receipt.json). Continue retained cubek-reduce Cargo.toml, then its full89-file tree and the remaining35 support/workflow/tool files. Full-source objective remains active and incomplete.


### Entire retained reduction source read, 2026-10-17

Read all89 retained cubek-reduce source/manifests/evaluation/test files,436,176characters. Overall3,703/3,738 files and39,359,193/39,556,704 characters are read;35files197,511characters remain. All3,738live working-source membership and hashes remain unchanged. Root uses registry cubek-reduce0.3; four independent Distillery probes select this retained patch, so its review does not become a root-backend execution claim.

Q458-Q462 record strict-axis off-by-one, finite top-k identity losing negative infinity, inverted/incomplete forced-unit idle checks, forced-cube lane/plane accumulator confusion and missing zero-k admission contract.17bounded CPU probes pass with controls: exact host validators/predicates and whole scalar top-k insertion interpreted with explicit Vector/Value doubles. These do not execute Cube DSL expansion, an actual Client, generated kernels or GPU behavior. [Probe scope and results](../../../support/receipts/authority-success-audit/reduce_contract_probe_receipt.json).

A1495-A1516 keep ownership and evidence separate from individual bugs. Fused values/indices really share each granularity's reduction body, conversion and output layout. Extrema really share NaN/identity/tie helpers. Top-k keeps separate identity/insertion/finalization networks; typed and dynamic shared allocators plus a manually mirrored footprint table remain multiple authorities. Forced/inferred admission drift accounts for two confirmed branch contradictions. Potential same-slot plane writes and vector-layout/dtype mismatches remain named qualification work, with no invented GPU corruption receipt.

Benchmark success language is broader than its gate: the fused reference path returns only values, TwoLaunch proof runs one output, small proof shapes differ from timed shapes, and Skipped is accepted. Existing TestCase integration checks both fused outputs and custom regressions reject skipped/compile-refused execution, so those stronger receipts should not be conflated with the benchmark gate. Fusedf16 traffic scaling also assumes equal output element sizes despite u32indices. Historical performance/driver comments are source evidence, not rerun measurements.

Report prefix, shared production/index and all source guards remain preserved. Probe EXE/PDB removed, retained evidence below100MB, no live audit process or isolated target/home/worktree. [Checkpoint](../../../support/receipts/authority-success-audit/reduce_complete_review_receipt.json). Continue the remaining35support/workflow/toolfiles, then final evidence/current-main reconciliation. Full-source objective remains active and incomplete.


### All working source read, completion reconciliation pending, 2026-10-17

Read the final35support/workflow/toolsource files,197,511characters. All3,738inventoried working-source files are fully read,39,556,704characters, with no pending chunk or changed SHA. Live tracked/non-ignored source membership matches the inventory exactly, including shared untracked product tests. Retained patches and source in old receipts were included. Binary assets/logs/lockfiles/JSONdata/prose and audit-generated diagnostics are outside the product-source denominator; the extension census and inertSVGasset were checked. Full-source reading is complete at HEAD7c718105plus the preserved shared working changes.

A1517-A1536 record the final ownership/evidence boundaries. Instrument freeze, capture collection and typographyverification are separate stages; they do not independently prove current compilation/rendering. Sennet/Tucket duplicate process and verdict policy; resident runners copy sampling and admission logic. Standards catalog records71specs25profiles with internally consistentcounts/references and explicitly denies conformance/physical qualification; its external factual/licensing metadata was not freshly reverified in this source pass.

Actual unchanged wrapper probes reproduced zero named-test acceptance in residentstages, extendingQ422; Q463fresh emptysoftwareJSON acceptance in streaming/formats; Q464failed childJSON returned without rejection by both copied custody functions; Q465browser_closed=true after rejected closecalls. [Exact scopes and results](../../../support/receipts/authority-success-audit/final_receipt_contract_probe_receipt.json). GPU/OS/process/browser doubles are explicitly synthetic; no model/GPU/broker/wallet/browser acceptance was run. All generatedEXE/PDB outputs were removed, and immutable compact diagnostics retained within the auditbudget.

gitfetch confirms origin/maina9b5dd66unchanged. Checkout remains16commitsbehind because earlier ff-only update overlapped sharedreportWIP; current-source coverage and separately read upstreamdiffs are distinct receipts. Final completion work is to verify that upstreamreviewchain, reconcile findings/authority/evidence and append the architectural closeout. Source reading alone is not marked as full-goal completion. Production/index/sharedWIP and exact prior report bytes remain preserved. [Checkpoint](../../../support/receipts/authority-success-audit/source_reading_complete_receipt.json).


### Full source audit closeout, 2026-10-17

The full source review is complete at working HEAD7c71810550234ccbe11f7429cd44e369b279ec06, including the preserved shared working changes:3,738files,1,043,171lines and39,556,704characters. Every file has contiguous acknowledged reading tied to its currentSHA256; live tracked/non-ignored membership was independently reconciled with the inventory. No unread source, stale hash or pending chunk remains. Current manifests/workflows/scenarios, optional/ignored tests, retained dependency patches and source-bearing historical receipts were included.

The findings register containsQ001-Q465, with individual source, evidence, status, verification limits and repair disposition. Repaired and superseded records are retained. The authority map containsA001-A1536 with the claimed owner, actual implementations and assessment. The212candidate followups remain explicitly unconfirmed;23ruled-out concerns preserve contrary evidence. Those registers distinguish source inspection, synthetic boundary probes, freshly compiled public APIs and actual product/backend execution. Reading source and preserving old receipts do not turn conditional or unexecuted tests into live acceptance.

The recurring strain is unfinished ownership migration. Graph/session history, current projections, saved/live content, custody lifetimes, transport admission and host presentation each have a legitimate owner, but callers still repeat parts of the owning layer's decisions. A local single-authority claim is often true only for a narrow field or stage. The audit records those valid boundaries alongside the remaining copies: shared output/reduction bodies and extrema helpers are real consolidation, while forced/inferred admission, vector-layout reconstruction, typed/dynamic shared allocation, response/input lowering and receipt verdict policy still have divergent implementations.

The most useful repair direction is to finish those boundaries one operation at a time: make graph/pane/session or device identity explicit at dispatch; let durable/source owners decide admission and revision; keep projections and caches tied to those identities; share pure validation and result classification where semantics match. Preserve deliberately different privacy, authorization and transport policies. Removing a duplicated owner decision is more valuable than splitting a large file while leaving the decision copied.

Misleading success is also repeated across layers. Numeric validation can skip or accept NaN, benchmark proof can check only one output or a different shape, ignored-test filters can execute zero tests, fresh files can stand in for valid software receipts, and cleanup attempts can be reported as completed resource closure. The final probes qualify those exact boundaries without claiming actual GPU/custody/browser failures. Earlier contained repairs retain their own receipts; this final read-only pass changed no production source.

Fresh gitfetch found origin/main unchanged ata9b5dd6635e7528ff3d2aec1e389075d628fcb6c. The working checkout remains16commitsbehind because a prior ff-only update would overwrite the shared report. All incoming source changes are separately covered by verified Git-object diff receipts:38changed files through00e44428, then6source files and4documentation files througha9b5dd66. Incoming receipt/log files are identified separately. This is a complete working-source audit plus an incoming-source diff review; upstream was not installed and its reported supplier/consumer runs were not rerun here.

Completion checks resolve every findings/authority receipt reference and verify1,733saved evidence-hash references with no missing or mismatched bytes. The source/index/sharedWIP guards and exact prior report prefix remain preserved. Retained audit evidence stays below100MB; new diagnosticEXE/PDB outputs were removed. No isolated target,Cargohome orworktree remains. The stableMere build cache is shared and retained for its existing owner.

[Requirement-by-requirement completion receipt](../../../support/receipts/authority-success-audit/audit_completion_receipt.json). [Findings register](../../../support/receipts/authority-success-audit/review_notes.json). [Authority and migration map](../../../support/receipts/authority-success-audit/authority_map.json). Repairs and broader runtime qualification remain named followup work; the requested full-source audit itself has no remaining required work.


### Repair batch re-gated and landed, 2026-10-17

Phase A2. The 23 repairs that the closeout left uncommitted (33 tracked files, +1,574/-168,
plus three new test files) were re-gated on the fast-forwarded base at `00db7399` under
toolchain 1.99.1, with the audit's own recorded commands, sequentially, on the audit's
shared build directory. `--offline` could not be kept: the fast-forward repinned genet to
`917f88e6`, absent from the local git cache. `--locked` was kept.

| Gate | Result |
|---|---|
| chartulary + mere-kernel (`store`) | 556 passed, 2 ignored |
| pandect | 354 passed, 1 ignored |
| castellan (`keeper`) | 309 passed; residue harness red (main's, below) |
| scenograph default / `scripts` | 48 / 52 passed |
| scenomise (`authoring`) | 217 passed, 2 failed (main's, below) |
| script-rhai | 38 passed |
| graphshell-client all-features | 91 passed, 1 failed (Q073, pre-existing) |
| djinn `h4_identity_receipt` bin | check passed |

All three reds reproduce on a clean worktree of the same HEAD with no batch applied, in a
separate build directory, both removed afterwards. The scenomise pair passed in the audit's
gate basis `4523cb6f`; upstream `4c4ea11d` (2026-10-17) put a `metre.occurrences` uniqueness
check ahead of the `marks` check in backdrops.rs, and the batch's only scenomise change is
one line in `opened_link::collect_entries`. The castellan residue harness reports one hit,
deterministic across four runs: a freed, uncleared master seed in an 88-byte block
allocated during `switch_profile` on sealed profile storage. The audit's castellan gate
recorded this harness passing with byte-identical batch files on 2026-10-16; castellan
source is unchanged upstream since; `switch_profile` is not in the batch. The one relevant
change is the toolchain bump to 1.99.1 in `e9df653a`. Both are main's defects and are
follow-up work in their own sessions; the findings register is sealed by the completion
receipt's hash and is not amended for them.

The record moved to this file the same day (R2); the
[relocation receipt](../../../support/receipts/authority-success-audit/audit_record_relocation_receipt.json)
seals both plans' hashes. Evidence for the gates: the
[re-gate receipt](../../../support/receipts/authority-success-audit/repair_batch_regate_receipt.json).
Landed as one sweep commit on main under R1 and R3 (the commit that carries this entry).
A2's done-conditions are met with the two main reds named as exceptions beside Q073.

**2026-10-18 annotation (the toolchain was not the cause).** The castellan follow-up ran
the residue harness under 1.99.0 as a control: red, with the identical hit. The block is the
one-element `Vec<SlotSummary>` that `public_profile_of` builds during `switch_profile`; its
slot key's `instance: Option<String>` is `None`, and the 16 bytes that `None` leaves
uninitialised carry a stale copy of the master seed from a stack temporary on the load path,
so this is stack residue leaking through padding, layout-dependent, not a missing `Drop`.
Why the audit's 2026-10-16 run was green is not explained. The fix choice is Mark's (R4).

**2026-10-18 Progress: the residue fix (R4, option B).** Findings: the harness was red
under 1.99.0 as well as 1.99.1 (control, clean checkout of `26a67386`, `--locked`, own
build dir), so the toolchain bump was not the cause. The 88-byte block is the one-element
`Vec<SlotSummary>` built by `public_profile_of` during `switch_profile` (a size-88
allocation trace in the switch phase shows only that Vec; the hit block's bytes show the
slot key's `instance: Option<String>` as `None` with 16 bytes of `canary_bytes(0)`, the
master seed, in the 16 uninitialised bytes beside the niche marker). Fix: castellan's
`public_profile_of` now pushes into a vector sized up front and rebuilds each key field by
field, instead of collecting clones. `cargo test -p castellan --features keeper` is green under
1.99.1 with the residue harness unchanged and its positive control still firing (uncleared
raw and decimal found, zeroized clean, live found); no other castellan test changed.
**B alone does not prove the stack is clean.** It stops this one copy of the stale bytes; the
seed still passes through the stack when a key is built, and another bulk copy could carry it
into the heap again, with presence depending on code generation. The source fix (A, personae
`from_seed` taking `&[u8; 32]`) is ledgered as item 13 of the
[upstream candidates ledger](../research/2026-10-03_upstream_candidates_ledger.md) and is not
done. Why the audit's 2026-10-16 run was green remains unexplained.


### Follow-up branches merged into main, 2026-10-18

Mark: "Just merge." `fix/castellan-slot-summary-padding` (9ccca7b7, R4) and
`fix/scenomise-backdrop-gate` (7bf3488b, R5) merged into main with plain merges. Re-gate on
the merged tree: scenomise authoring 232 passed; castellan keeper 313 passed with the residue
harness green, after one of the audit's own tests was re-aimed. Overnight commit fc5d2f7e
replaced pandect's write-tmp-then-rename with the shared `atomic_file` writer, so the Q026
test's locked `.json.tmp` no longer met the write path; it now holds the persona wallet
itself open with no sharing, which refuses the writer's rename before any byte lands, and
clears the refused rename's temporary before the retry. The property guarded is unchanged.
Both fix branches deleted after the merge; the audit's two main reds are closed. Q073 stays
as recorded.

---

*Archived 2026-10-28.* Complete; its residue (what it named as unmet or deferred) now lives in `mere_docs/implementation_strategy/2026-10-28_source_audit_repairs_plan.md`, which cites this file by line. This copy is history: its status lines, receipts and claims are as they stood, not current.
