# Coupling effect evaluation draft, October 12, 2026

**Status:** implemented engine draft; this receipt qualifies a point-evaluation
API and its reuse by the existing executor. The tracing continuation below
qualifies exact compiled bindings and actual execution observations. Visible
host inspection and coordinator-owned adoption remain separate. The owning
[dynamics plan](../../../implementation_strategy/2026-10-02_dynamics_grammar_plan.md#coupling-effect-evaluation-draft-2026-10-12)
and [handoff](../../../research/2026-10-09_scenograph_codex_handoff.md#fieldeditor-runtime-seam-review-2026-10-10)
retain the field membership, clock, conflict and script checkpoints.

The original executor is published Mere `e6192a18`. The source checkpoint is
`5b2d67a2`, rebased onto published `3f26d1f3`, including `db26bb52`: upstream changed
Pictograph presentation, receiving-host docs and packaging, but no Seiche/Numen
source. The root lock adds three unrelated facade dependency edges; locked
metadata passes, with no changed Seiche/Numen dependency input. The engine
keeps Rapier 0.36 with enhanced determinism. No manifest, dependency or version
changes accompany this draft. Qualification uses the existing
`C:/t/cargo-targets/mere` target with two build jobs and four test threads.

The point-evaluation and tracing sections below retain the facts qualified at
their named source checkpoints. The latest field-runtime continuation implements
F203 to F205; earlier t=0/selector-only limits are historical checkpoint limits.

## What is implemented

[CouplingForce](../../../../../crates/conatus/seiche/src/coupling_force.rs)
can evaluate one response at caller-supplied node/body inputs. The report
distinguishes additive force, velocity scaling, velocity assignment and
displacement. It explains inert results caused by an absent captured target,
wrong field type, being outside the strictly positive region, a non-dynamic
advection body, or an unsupported physical response. The report's configured
sampling time is explicitly zero. That is the existing evaluator's policy,
not a new simulation clock or claim that a field was sampled for every result.

The ordinary executor consumes the same private evaluation after resolving the
body. It preserves the original field sample-point snapshot, current-body writes,
target multiplicity, response ordering and hold behavior. Its execution path
does not scan the complete target list again for every body. Damping continues
to use its factor without multiplying by coupling strength. The wall remains
a depth-scaled additive force. Advection's hold test is not incorrectly applied
to every velocity-writing response.

The [runnable example](../../../../../crates/conatus/seiche/examples/coupling_effects.rs)
seeds a moving body through the real integrator, prints the proposed effect,
then installs the coupling and records motion after 60 actual ticks. It exercises
wall force, damping, alignment and advection. Proposed commands and observed
motion are printed separately; damping acts on a moving body instead of producing
a misleading zero-speed demonstration.

Run from the Mere repository with the reusable target configured:

```text
cargo run --locked -p seiche --no-default-features --example coupling_effects -j 2
```

## Qualification

The [focused gate](focused.log) passes ten coupling tests, seven added by this
draft. [Original-executor measurements](baseline-trajectory.log) precede the
production refactor. Thirteen fixtures hash the float words of position and
velocity for three bodies over 120 ticks, with two integration step sizes.
They cover every closed response, open and mismatched-field controls, a pinned
body, an untargeted moving body, the positive-region guard and duplicate-target
force/damping/advection. All thirteen measured digests remain unchanged. These
are bounded regression fixtures, not proof for every field, platform or solver.

The [negative control](negative-dedup.log) deliberately deduplicates captured
targets. It fails the trajectory receipt on all three duplicate-target cases;
the [restoration record](negative-control.json) verifies the exact production
source was restored.
Git checkout subsequently normalized line endings; the qualification manifest
records raw and canonical-LF hashes, and the canonical source still matches the
restoration record. The public-probe tests check exact simple-field effects,
strict region boundaries, distinct inert reasons, legacy damping strength,
configured time and registry behavior. Another test queries an installed coupling
2,400 times while comparing its unchanged state and subsequent 120-tick motion
against an unqueried control simulation.

| Gate | Result |
| --- | --- |
| Default Seiche | 168 library passed, 10 ignored; integration 5 + 4 passed |
| Inline Seiche, no default features | 162 library passed, 10 ignored; integration 5 + 4 passed |
| Serde-enabled Seiche | 175 library passed, 10 ignored; integration 5 + 4 passed |
| Wasm inline library | compile check passed |
| Runnable example | passed; real before/after state recorded |
| Existing Pictograph graph bridge | four passed, zero failed; 430 other library tests filtered |
| Scoped Clippy | passed with 14 Seiche warnings and two Numen warnings |
| Scoped license/header checks | passed |

The example's damping leaves velocity near zero after the run, alignment keeps
the supplied direction, and advection changes position separately from its
remaining inertial velocity. The logs retain the complete values. Wasm has a
compile receipt here, not a new native/wasm float-identity or browser receipt.
GPU execution and full-workspace tests are not claimed by these CPU-path gates.
The initial fixture's ambiguous `Into<String>` call was corrected before its
baseline measurement; that diagnostic is retained.
The three coupling/example source files pass scoped formatting. Seiche's root
`lib.rs` retains one pre-existing export-order formatting difference;
[baseline comparison](format-baseline.json) proves this amendment introduces
none. No unrelated exports were reordered to make that file appear clean.

## Remaining boundary

The point evaluation is read-only and hypothetical. It does not prove that a
body exists, the inputs are finite, the registry is fully resolved, or any write
was applied. Missing field references retain numen's current zero fallback;
the returned zero must not imply valid backing. It does not aggregate duplicate
targets, other couplings, contacts, encoded-axis holds or anchored returns.
Body presence and field/coupling/occurrence provenance stay with the caller.
The result is session data and has no serialized source-authority schema.

A production inspector still needs a stable owner-held provenance map and an
actual execution record before labeling contributions as applied. This draft
supplies neither spatial extent membership nor a new arbitration rule for
state writers. Clocks, entry/exit actions, applet execution and host adoption
remain open. Firefox stays parked while its browser-policy block persists;
this engine draft introduces no browser UI change or new browser qualification.

Raw logs and the reproducible fixture remain under
`C:/Users/mark_/Code/testing/mere/coupling-effects`. The worktree is temporary
because the primary checkout contains concurrent unpublished work. No isolated
Cargo target/home was created. Source-bearing receipts and other owners' work
are preserved.


## Exact bindings and applied-effect tracing (2026-10-12)

The engine checkpoint is `feba92092caa56139d6af2f3425893dc46b5e4c9`,
built on published Mere `e0dae9c9`. Final Canvas fixture/module-order corrections
are `899edbe3`; Seiche source is identical across these checkpoints. No dependency or manifest change belongs to
this continuation. It uses `C:/t/cargo-targets/mere`, two Cargo jobs and four
test threads. The frozen source hashes and gate commands accompany the raw logs
in [trace/qualification.json](trace/qualification.json). This is implementing-agent
qualification, with external consumer adoption and coordinator review separate.

The exact compiled input snapshot retains the physical response, strength,
scalar/vector definition, registry and ordered target slots. The live Canvas
graph bridge now supplies exact field/coupling IDs and stable owner member IDs.
Graph structural revision is a hint, not proof of unchanged backing: field
writes need not advance it. No occurrence ID is fabricated for graph-node
bodies. A body owner with an occurrence adapter may supply an explicit occurrence
ID for each target slot. Anonymous callers remain visibly unprovenanced.

Observation is opt-in through `set_coupling_trace_limit`; the caller supplies a
positive row cap, with `None` disabling capture. Each row records the exact
binding instance and target slot, execution order, sample point, configured
sample time, proposed response and actual outcome. Immediate before/after state
covers position, linear velocity, user-force accumulator, dynamic state and
sleeping state. `Write` means the call executed, not that Rapier accepted a state
change. Inert results invoke no write; a missing body has no evaluation. The
report discloses omitted rows while physics executes every application.

Only the most recent valid tick is retained. A process-local simulation token
prevents equal step/binding-generation numbers from conflating separate worlds.
Force rebuild and observation configuration clear old evidence. Actor commands
use barriers to reject delayed snapshots from an old configuration. The actor
publishes through a single replaceable trace slot shared by queued layout
packets. A superseded packet yields unavailable evidence, so an unread queue
cannot accumulate trace history. Hosts can deliberately retain returned reports;
that caller-owned history is outside the runtime retention guarantee.

The trace covers the dedicated coupling slot before integration. Generic boxed
forces, anchor/affinity writes, collision and fluid response, and final integrated
motion are outside its contribution claim. The original field sampling clock
remains t=0, even where an inert evaluation does not sample the field. Captured
query targets are not filtered through field extent. Source IDs are supplied by
the owner adapter; Seiche does not authenticate them or infer content resource
identity from an owner member ID. A host must rebuild bindings when its target
mapping or source backing changes.

The [runnable trace example](../../../../../crates/conatus/seiche/examples/coupling_trace.rs)
uses two separately identified couplings, each with a repeated target. A three-row
cap records only three of four actual displacement calls, reports one omitted
row, and still arrives at x=58 from x=50. Repeated slots sample the original
per-coupling point while their writes accumulate on current body state. Run:

```text
cargo run --locked --offline -p seiche --no-default-features --example coupling_trace -j 2
```

The final source/API gates pass:

| Gate | Result |
|---|---|
| [Focused coupling tests](trace/focused.log) | 24 passed; all 13 original digests agree with tracing off/on |
| [Default engine](trace/default.log) | 182 library passed, 10 ignored; integrations 5 and 4 passed |
| [Inline engine](trace/inline.log) | 173 library passed, 10 ignored; integrations 5 and 4 passed |
| [Serde engine](trace/serde.log) | 189 library passed, 10 ignored; integrations 5 and 4 passed |
| [Wasm inline library](trace/wasm.log) | Compile passed |
| [Canvas bridge](trace/host-bridge.log) | 10 passed, 425 other library tests filtered |
| [Runnable example](trace/example.log) | Four writes execute with three recorded and one omitted |
| [Clippy all targets](trace/clippy.log) | Passed; 16 library warnings, 18 test warnings including 16 duplicates; Numen 2 warnings |

The [cap control](trace/negative-cap-skips-physics.log) deliberately skips
physical applications when recording is capped; the test fails on the wrong
position. The [identity control](trace/negative-local-index-identity.log)
substitutes a local node index for the stable member ID; the Canvas test fails.
The [restoration record](trace/negative-controls.json) verifies both production
files are restored byte for byte, followed by passing focused and bridge gates.
These are deliberately wrong source controls, not accepted implementation.

Two implementation failures are retained for clarity. The first
[Canvas compile](trace/host-fixture-import-failure.log) lacked the existing
`GraphFixtures` trait import; `899edbe3` fixes the test import and module placement.
A [sleeping-state test](trace/sleep-assumption-failure.log) initially expected an
identical zero velocity assignment to wake a sleeping body. The current Rapier
source and observed behavior contradict that assumption: it changes neither
velocity nor sleeping state. The corrected test verifies the ignored assignment
and a later changed assignment that does wake the body. The executor was not
changed to satisfy the mistaken expectation.

All fifteen owned Rust source files pass the
[header and baseline-aware format check](trace/source-format-headers.json).
Existing format differences in Canvas and Seiche module/export blocks and the
runtime's obstacle command line are preserved without introducing new format
debt. The index-wide [header check](trace/headers-seiche.log), despite its prefix
argument, reports one inherited missing header in the renderer receipt's
`qualify_final_graphs.py`, already present at `e0dae9c9`. This continuation does
not repair that unrelated owner file or claim a clean repository-wide header
check. Dependency patch warnings and Clippy diagnostics remain in the raw logs.

The mechanical audit self-test passed. The
[comparison against published e0dae9c9](trace/docs-comparison.json) introduces
zero findings across the five amended documents; inherited findings remain.
[D2 verification](trace/docs-d2.json) accepts this continuation's five records
in batch 73, with the same three unrelated missing coverage records for the
classic-browser and producer receipts. These are bounded documentation results,
not a clean whole-tree audit.
 The earlier point-evaluation receipt
and its counts remain frozen above. Clock and membership answers are recorded verbatim as F203 to F205 in the
owning plan. Their defaults are simulation time (Pause freezes, Reset rewinds),
per-appearance reference-point membership including the boundary, and initial
matches reported without entry actions. Their runtime adapters remain pending;
this trace still reports current t=0 sampling and captured selector targets. No Firefox, native UI, GPU, whole-stack adoption or persistent
trace format is claimed by this source/API draft.


## Field clock and membership runtime (2026-10-12)

**Status:** implemented F203 to F205 source/API and Canvas adapter. Source
checkpoint `bb2591c08032226506b986f19fd9ab1ec131b69e` is based on published Mere
`050e7d54`. The shared target is `C:/t/cargo-targets/mere`, with two build jobs
and four test threads. Dependency/manifest changes do not belong to this slice.
Frozen source hashes, commands and outcomes are in
[runtime/qualification.json](runtime/qualification.json). Qualification is
implementing-agent evidence, with coordinator review and external adoption
separate.

The field clock accumulates valid tick intervals in f64. Each physical evaluation
and trace retains the actual f32 value consumed by Numen; field time is separate
from the monotonically increasing step counter used to age lagged answers.
Pause freezes it, including pointer-driven body ticks, and positive settle/Play
resumes. Reset restores supplied node positions, rewinds field time on a new
epoch and clears old traces and membership baselines. It preserves paused state.
Ordinary seeding and source rebuilds do not rewind time. Grouped force contexts
carry the same clock. Recognized Time-dependent couplings request continued
ticks, while unsupported physical responses make no animation-execution claim.

Each candidate body is tested at its reference point in the bound world extent.
Region and polygon boundaries are inclusive; collinear/empty polygon behavior
remains empty, and no epsilon/hysteresis policy is invented. Camera transforms
and rendered footprints do not drive this test. Extent and available selector
metadata are retained in the exact input snapshot. Scope eligibility remains
separate from the response's positive-scalar and dynamic-body guards. Attached
extents refuse physical execution until an owner supplies resolved geometry.
Missing bodies, invalid reference points/sample time and unresolved scope do
not become a fabricated outside-to-inside transition.

First observations report initial membership without invoking entry callbacks.
Known appearances survive valid binding changes. Reports retain field, query,
source-identity/revision, actual point-movement and Reset causes; several may
coincide. Candidate removal/restoration is a later transition for a known live
appearance. A new/reused body generation or changed identity starts initial
reporting. Repeated physical target slots keep their original multiplicity and
sample-point snapshot, but one appearance receives one membership observation
and one eligible entry callback. Separate bodies with a shared member ID and
different occurrence IDs can have different membership.

Entry callbacks are explicitly owner-registered and run synchronously on the
simulation thread. Only later observed entries invoke them. Results distinguish
not requested, unregistered, completed and refused. Completion means the callback
returned success; the domain owner still owns any pending work, source validation
and durable transaction. Duplicate named coupling bindings have distinct compiled
histories and refuse ambiguous entry dispatch. Physical effects remain separately
observed. No serialized applet or general script engine is executed here.

The trace's row cap still discloses omitted rows while executing all physical
applications and eligible membership callbacks. Reports are latest diagnostics,
not a retained event log. The actor shares one latest membership frame, so queued
layout packets cannot accumulate diagnostic history. Callbacks execute at the
observation rather than on report folding; reading/dropping a report cannot
replay or suppress an already eligible callback. A reset barrier rejects delayed
clock, membership and physical-trace evidence. Historical reports retain their
original binding and clock epoch instead of resolving today's source by index.

The live Canvas force bridge now passes extent and selector plus the existing
stable field/coupling/member IDs, without fabricating occurrence IDs for graph
bodies. It excludes retired fields at initial construction as well as rebuild.
Canvas's existing node-reseed and graph-replacement paths call the explicit
field-runtime reset; this is not a complete backdrop/media/law-state reset.
Visible scripting controls, app-specific receivers, UI/platform acceptance,
Firefox, GPU execution and external consumer pins are not qualified by this
backend adapter.

The [runnable example](../../../../../crates/conatus/seiche/examples/field_runtime.rs)
shows real Time-driven displacement, an initial inside match with zero callbacks,
an observed exit, a field edit admitting a known appearance with one callback,
clock freezing and a reset that re-establishes initial membership without replay.
Run from Mere:

```text
cargo run --locked --offline -p seiche --no-default-features --example field_runtime -j 2
```

Gradient declarations follow the new execution. Global autonomous fields retain
their existing energy path. Timed or extent-clipped gradient terms conservatively
declare nonautonomous N and expose no timeless global potential; a bounded field
cannot certify the unbounded potential when it stops applying outside scope.
State-writing responses retain K. The original trajectory fixtures use global
static fields and remain the direct regression baseline.

The final gates pass:

| Gate | Result |
|---|---|
| [New runtime cases](runtime/focused.log) | 19 passed, including three actor cases |
| [Original trajectories](runtime/trajectory.log) | All 13 original digests agree; fixture checks tracing off/on |
| [Default engine](runtime/default.log) | 201 library passed, 10 ignored; integrations 5 and 4 passed |
| [Inline engine](runtime/inline.log) | 189 library passed, 10 ignored; integrations 5 and 4 passed |
| [Serde engine](runtime/serde.log) | 208 library passed, 10 ignored; integrations 5 and 4 passed |
| [Numen](runtime/numen.log) | 53 passed, including every polygon edge/vertex, reversed winding and collinear controls |
| [Wasm inline library](runtime/wasm.log) | Compile passed |
| [Example](runtime/example.log) | Real clock, scope, entry and reset assertions passed; one owner entry callback |
| [Canvas runtime](runtime/host-field-runtime.log) | 4 passed; 435 other library tests filtered |
| [Canvas coupling bridge](runtime/host-coupling.log) | 14 passed; 425 other tests filtered |
| [Canvas live physics](runtime/host-live-physics.log) | 10 passed; 429 other tests filtered |
| [Clippy all targets](runtime/clippy.log) | Passed; 15 Seiche library warnings, 17 test warnings including 15 duplicates; inherited diagnostics recorded |

Three source controls prove the tests reject broken rulings. The
[zero-clock control](runtime/negative-zero-clock.log) samples a timed force at
zero instead of accepted time; the real-motion test fails. The
[exclusive-boundary control](runtime/negative-exclusive-boundary.log) excludes
a region's lower boundary; the physical-write test fails. The
[initial-entry control](runtime/negative-initial-entry.log) invokes a registered
callback on an initial inside match; the initial-suppression test fails.
[Restoration](runtime/negative-controls.json) verifies every mutated source is
restored byte for byte. Focused, original-trajectory and Canvas runtime gates
then pass again on the restored source.

Prototype failures are retained. An
[import failure](runtime/tick-demand-import-failure.log) was corrected by importing
the existing force trait for tick demand. A
[gradient assertion](runtime/gradient-exactness-assumption-failure.log) incorrectly
expected an analytical 0.25 gradient where Numen's unchanged Mul/Sample path
uses finite differences and returns about 0.24986266 at this scale. The corrected
fixture checks an explicit 1e-3 numerical tolerance, while clock/displacement
fixtures retain exact power-of-two assertions. A
[camera fixture](runtime/camera-no-new-tick-assumption-failure.log) expected an
unchanged membership event after rendering an idle world without taking a new
tick; the accessor correctly retained its historical initial report. The fixture
now takes a new sample after the camera operations. Product behavior was not
changed to fit either mistaken assertion.

All 25 owned source files pass the
[header/baseline-aware formatting check](runtime/source-format-headers.json).
The current index-wide [header audit](runtime/headers.log) passes; the earlier
tracing phase's header finding is historical at its own checkpoint. Existing
Seiche export/runtime-line format debt is preserved. Documentation self-test,
comparison against the published base and D2 coverage results accompany this
phase. The [mechanical comparison](runtime/docs-comparison.json) against
published `050e7d54` introduces zero findings across five amended documents.
[D2 verification](runtime/docs-d2.json) accepts batch 76's five bounded records,
with the same three unrelated missing coverage records for the classic-browser
and producer receipts. These outcomes do not requalify the whole document tree.
