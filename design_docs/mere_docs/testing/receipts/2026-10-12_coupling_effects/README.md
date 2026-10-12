# Coupling effect evaluation draft, October 12, 2026

**Status:** implemented engine draft; this receipt qualifies a point-evaluation
API and its reuse by the existing executor. Host inspection, applied-effect
tracing and coordinator-owned adoption remain separate. The owning
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
