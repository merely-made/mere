# Graphshell tree controls, pause and frame cost

Date: 2026-09-27. First slice of phase 4 of the
[one-tree plan](../implementation_strategy/2026-09-25_graphshell_one_tree_plan.md).
The [migration inventory](../implementation_strategy/2026-09-27_graphshell_tree_migration_inventory.md)
defines what remains outside this slice.

## Behavior and ownership

The tree has Cambium buttons for pan, zoom, fit, Play/Pause physics and Restore
arrangement. Both browser presentations use `CanvasCommand`; Pictograph owns
the camera and simulation state. Captured primary down/move/up reaches Canvas
through the shared host pointer path. Graph keys apply only at graph focus.
Mesquite owns scenario execution; its generic `key` verb reaches host dispatch.

Mark ruled that Pause freezes visible positions. Pictograph now keeps held
positions separate from stored arrangement slots. Restore arrangement explicitly
pauses and reapplies those slots. Resume seeds from the held positions, clearing
velocity. Tests cover resumed continuity, held dragging while neighbours move,
explicit reseed, graph membership changes and graph replacement.

Seiche Halt now suppresses perpetual stepping as well as the settle budget.
Actor seed/halt barriers reject older queued snapshots, and idle actors
acknowledge commands without a simulation tick. `settle(0)` does not revive a
halted actor. A hard deadline bounds the actor acknowledgement regression.

Pictograph skips equal style/class writes, preserving Genet's retained layout
on unchanged frames. Optional host-clock profiling separates physics,
preparation, DOM mutation/restyle, DOM frame, scene lowering and rasterization.
Ordinary Canvas frames do not read that clock. Timings also record total and
visible nodes and paint-command counts before/after culling.

## Automated checks

The combined offline, locked native run passed 395 tests: 261 Pictograph unit,
two headless Vello, 80 Seiche, and 52 host/Mesquite tests. Command:

```text
cargo test --offline --locked -j2 -p seiche -p pictograph -p mesquite
  -p cambium-genet-winit-host --features pictograph/canvas,pictograph/vello
  --lib --test scenario --test async_capture --test vello_headless
  -- --test-threads=1
```

Log: `Code/testing/mere/p4-runtime-tests.log`. After the final actor-zero-budget
and bounded-wait change, all four Seiche runtime tests passed again.
The unchanged-DOM regression fails with the equality guards removed and passes
with them restored; pan, zoom, selection and actual width changes still update.
Negative/positive logs: `p4-retained-before.log`, `p4-retained-after.log`.
The standalone wasm32 build passes with the existing dev physics optimizations,
debug information disabled, and `getrandom_backend="wasm_js"`.

## Headed browser behavior

`p4_tree_controls` passes in headed Chrome via the existing scenario script,
with port 8747 and the worktree web directory. The final receipt is
`Code/testing/mere/scenarios/graphshell-web/p4_controls_focus/`.
It checks button focus, Enter activation, Tab focus movement, graph-only arrow
keys, live motion, pointer capture, held-node distance, release, frozen pause,
and restoration of the original geometry before manual dragging. All positions
remain finite. The three captures show the held, paused and restored graph.
The projected accessibility tree contains all nine named toolbar buttons.

The scenario waits for finite camera inertia before asserting that an arrow
key at button focus leaves the camera unchanged. Two earlier failed receipts
remain historical diagnostics: one used an integer-only comparison on zoom,
and one compared the camera while legitimate pan inertia was still active.
Neither is passing evidence.

Final control bundle SHA256:
`bae48b171551484c2048cd9a4e973f309be9b3675af0d1d2bd31a4bb0c01b4d6`.
Scenario SHA256:
`3b39b55d78891c6ed91f58b9e064219f18916026b58e89975d0b801f7a542326`.
These runs use Genet `92b249af5b200f78d036d8e9b0cb4fe025610740`,
netrender `9607d16f1907f6c2085648ae96abcaa30d7c3d41`, and Vello 0.10.1.

## Frame cost

All measurements are dev builds, seed 7 for generated graphs, one headed
browser at a time, with no concurrent builds/tests during timing windows.
Logical viewport: 1282 by 722. Graph slot: 1282 by 627 logical, 2564 by 1254
physical. These short single runs identify cost; they are not production
performance acceptance or a controlled equal-resolution presenter comparison.

| Workload | Interval p50 / p95 ms | Physics p50 | DOM mutation/restyle p50 | DOM frame p50 |
| --- | --- | --- | --- | --- |
| 2,000 paused, before equal-write guards | 1603.3 / 1716.5 | 0.2 | 428.8 | 1087.3 |
| 2,000 paused, after guards | 107.6 / 118.0 | 0.3 | 1.2 | 0.3 |
| 128 moving, after guards | 141.7 / 158.0 | 0.4 | 64.4 | 66.1 |
| 512 moving, after guards | 872.0 / 947.5 | 2.6 | 585.4 | 235.5 |

The paired paused improvement is about 14.9 times. Remaining paused costs
include preparation (24.5 ms), faces (20.4 ms), and rasterization (35.9 ms).
The live 2,000-node run timed out after 180 seconds, without a completed timing
window or capture. Its empty page-error list does not establish responsiveness.

Paused receipts and artifact hashes are in `p4-render-profile-build.json` and
`scenarios/graphshell-web/p4_baseline/`, `p4_retained/`. Moving receipts are in
`p4_live_128/` and `p4_live_512/`. Their instrumented source preceded the final
focus-observation addition; a separate exact bundle hash was not retained.
Moving counts are 128/128 and 512/512 total/visible nodes, with 900 and 3588
paint commands respectively, unchanged by culling because the graph is fitted.
The final fixture control window has 11 visible nodes and 69 paint commands;
its captures/interaction steps introduce pacing outliers and are not a steady
performance comparison. GPU timestamp spans include queue idle and must not
be interpreted as isolated GPU execution cost.

## Genet motion adoption

Genet `27d20d3fc51ac5fcd2a2db231e035a3e06013ae1` was pushed with approval
and adopted in both Mere manifests and local locks. The root lock is tracked;
the standalone web lock remains ignored by the existing repository policy.
The repin includes earlier Genet generated-text, accessibility and positioned
layout commits. Reader accessibility nodes now supply the new optional
description field. Unrelated primary-checkout work was preserved.

All 298 targeted native tests pass after the repin: 261 Pictograph unit,
two headless Vello, 16 host unit, 18 scenario and one asynchronous capture.
The real Canvas camera test now also checks retained layout generation across
pan and zoom. The offline locked wasm build passes. Logs are
`Code/testing/mere/p4-repin-tests.log` and `p4-retained-motion-locked.log`.

`p4_motion_repin_controls/` passes every headed behavior assertion. All three
whole-frame captures were inspected. The restored capture is byte-identical
to the earlier `p4_controls_focus/` capture; the moving/held captures are not
byte-identical and are supported by their behavioral assertions. Bundle SHA256:
`af8ae8d19b5ce9804d8a53b3b83bafb41721911dcf5d4a485dd5d1ac43f6ae8c`.
Artifact/lock hashes and capture comparisons are in
`Code/testing/mere/p4-retained-motion-adoption.json`. Another release build
was active during this functional run, so its timings are not a comparable
performance receipt.

After the other jobs exited, separate 14-frame live runs passed at 128 and
512 nodes. No Cargo/compiler/test job was observed before or after those
windows; both remained visible and reported no page errors. Captures were
inspected whole-frame. Counts remain 128/128 and 512/512 total/visible, with
900 and 3588 paint commands, matching the earlier workload.

| Moving nodes | Earlier interval p50 / p95 ms | Repinned interval p50 / p95 ms | DOM frame p50 before / after | Mutation/restyle p50 after | Physics p50 after |
| --- | --- | --- | --- | --- | --- |
| 128 | 141.7 / 158.0 | 96.0 / 98.9 | 66.1 / 24.8 | 61.0 | 0.4 |
| 512 | 872.0 / 947.5 | 595.0 / 622.9 | 235.5 / 76.6 | 491.6 | 2.1 |

Receipts: `p4_motion_repin_128/` and `p4_motion_repin_512/`, under the same
scenario evidence directory. Artifact and timing data are also in the adoption
JSON. These single dev runs show about a 32% lower median interval after the
Genet repin. They do not isolate the motion commit from the earlier Genet
changes included in that pin. The majority of remaining time is mutation and
restyling. The separate restyle fix below is not in these measurements.

## Genet restyle adoption and elapsed-time core

Genet `f2e2850fcc49ade2a00a40c9aef220013336aa0b` is pushed and adopted in
both manifests and locks. The native gate passes 403 tests: 261 Pictograph,
two headless Vello, 88 Seiche and 52 host/Mesquite tests. The offline locked
wasm build passes. Logs: `Code/testing/mere/p4-restyle-tests.log` and
`p4-restyle-locked.log`. The initial native run regenerated the root lock;
the subsequent wasm verification used its existing lock.

The latest headed controls run, `p4_restyle_controls/`, passes every behavior
assertion, exposes all nine named toolbar buttons and reports no page errors.
Its held, paused and restored captures were inspected whole-frame. Separate
14-frame live windows pass at 128, 512 and 2,000 nodes, with all graph captures
inspected. No concurrent compiler/test job was observed during these timing
windows. These retain the same dev-build and viewport qualifications above.

| Moving nodes | Interval p50 / p95 ms | Physics p50 | Mutation/restyle p50 | DOM frame p50 |
| --- | --- | --- | --- | --- |
| 128 | 73.8 / 78.1 | 0.4 | 39.5 | 24.7 |
| 512 | 251.4 / 261.2 | 2.0 | 151.1 | 76.4 |
| 2,000 | 1365.0 / 1428.1 | 21.6 | 960.8 | 287.8 |

The 512-node median falls about 58% from the preceding 595.0 ms run. DOM
frame cost is almost unchanged; mutation/restyle falls from 491.6 to 151.1 ms.
The 2,000-node live run now completes and draws the graph, but 1.365 seconds
per frame remains too slow. Its previous timeout supplies no valid timing
ratio. Total/visible node counts match at each size; paint counts are 900,
3588 and 14003, unchanged by culling because the graphs are fitted.

Receipts are `p4_restyle_128/`, `p4_restyle_512/`, `p4_restyle_2000/` and
`p4_restyle_controls/` under `Code/testing/mere/scenarios/graphshell-web/`.
Hashes, locks, timing data and build provenance are recorded in
`Code/testing/mere/p4-restyle-adoption.json`. Bundle SHA256:
`ce13a33fdbcfa6668470790f3a8451f36af205e9d2639cac819d98995e9606c5`.

Seiche also gains the additive, caller-timed `Physics::advance_elapsed` API.
Configurable elapsed and step caps discard excess whole-step debt and retain
only a substep fraction. Seed, halt and suspension reset that fraction;
actors retain their own pacing. Twelve focused runtime tests pass with
default features, eight without default features. Those browser measurements
used deterministic `advance_frame`; elapsed-time adoption is recorded below.

## Elapsed-time host adoption (2026-09-29)

Graphshell's tree producer now uses the host's monotonic timestamp through
Pictograph `frame_at` / `frame_profiled_at`. The web host supplies animation
timestamps and uses the same performance clock for immediate input draws.
Document hiding suspends producers immediately; resuming starts a fresh
Canvas baseline. Hidden mounts still publish their accessibility mirror.
Pause, resume, reseed, restore and idle-to-active transitions cannot accumulate
catch-up debt. Backward timestamps retain the previous high-water mark.

The tree page accepts `physics_max_steps` and `physics_max_elapsed_ms`, with
defaults of three steps and 50 ms. Excess debt is discarded and reported;
one bounded physics advance is followed by one composition. The old presenter
and deterministic Canvas methods keep their existing behavior. Camera and
ambient animation pacing are outside this physics change.

The locked wasm build passes. Headed Chrome scenarios `p4_tree_controls` and
`p4_tree_elapsed?nodes=128&seed=7&physics_max_steps=1` both pass, without page
errors. Captures were inspected for graph content. The elapsed scenario
checks live motion, the selected step cap, frozen Pause and exact Restore.
Its 14-frame window reports exactly one physics step per frame and discards
92,433–110,633 microseconds per frame. This is functional evidence under
concurrent builds, not a controlled performance comparison. The recorded
121.0 ms median interval is not a responsiveness claim.

Raw receipts: `Code/testing/mere/scenarios/graphshell-web/elapsed_host_controls/`
and `elapsed_host_timing/`; build log: `Code/testing/mere/elapsed-host-wasm.log`.
Bundle SHA256: `cc4a0dbc5ce607dc87bb56788bc9171c0b20b64cb93cbe98d563059e1361feb7`.
All 266 Pictograph tests pass. Rootstock passes 41 tests, including the new
timestamp delivery, untimed redraw and visibility lifecycle test. Its existing
`equal_hover_cascade_retains_geometry_text_generation_and_scroll` test fails
with a zero element-scroll offset instead of 12 px. Replacing every changed
rootstock source with its unchanged `a31b9a14` version reproduces the same
failure; all saved changes were then restored. This is not a fully green
Rootstock suite. Logs are `elapsed-host-final-native.log`,
`elapsed-host-pictograph-final.log` and `elapsed-host-baseline-scroll.log` under
`Code/testing/mere/`; `scripts/elapsed-host-baseline-control.py` records the
comparison procedure. No dependency pins or lock files changed.
The final restored-source run, `elapsed-host-qualified-native.log`, passes
307 tests with only that demonstrated baseline failure explicitly filtered.

Real browser hide/show and background-tab initialization have not received a
headed scenario receipt; the lifecycle contract is also tested at the host
and Canvas boundaries.

## Open gates

- Genet commit `27d20d3fc51ac5fcd2a2db231e035a3e06013ae1` admits safe retained
  layout for already transformed positioned boxes changing only 2D transform
  or numeric z-index. All 283 Livery tests pass, including four new regressions;
  disabling the fast path makes its unchanged-layout assertion fail. The
  adoption and bounded downstream timings are recorded above; acceptable
  large-graph responsiveness remains open.
- Genet `f2e2850fcc49ade2a00a40c9aef220013336aa0b` is a separate restyle
  fix, pushed with approval along with Mere `reservoir-v2` at `68b7c060`.
  It shares immutable hints, scopes and sibling counts
  across the batch, coalesces roots once and deduplicates sibling enumeration.
  All 283 unit and 19 focused integration tests pass, including full-style,
  mixed mutation, shadow-scope and HTML-hint comparisons. At 64/256 siblings,
  child visits are 514/2050 and parent lookups 642/2562. Restoring repeated
  parent-count walks makes the work regression fail (4546/67330 child visits).
  Independent review is clear. Browser adoption measurements above confirm
  a frame-time improvement, while large-graph responsiveness remains open.
- Tree-page physics now advances bounded elapsed steps on the rendering
  thread. When rendering exceeds the caps, discarded time still slows motion.
  Default pairwise exclusion is quadratic. GPU force parity is outside these
  browser receipts, which use the CPU path. At 2,000 nodes a single step
  already exceeds a nominal 16.7 ms frame budget; catch-up caps alone cannot
  solve that cost.
- Actual IndexedDB application state, remote sessions and product panels still
  belong to the old presenter. Ctrl+wheel modifiers and middle-button parity,
  all five public wrappers and their product scenarios remain migration work.
  Continuous rasterization still protects asynchronous Vello buffer recovery.

The elapsed-time slice is isolated in `worktrees/mere-canvas-elapsed` because
the primary checkout has concurrent work. The reusable web target is
`C:/t/cargo-targets/mere/web`. No isolated Cargo home was created.
After verification, `cargo clean` removed the slice's isolated native output
at `C:/t/cargo-targets/mere/canvas-elapsed` (3.2 GiB). Logs and browser receipts
remain outside that target.
