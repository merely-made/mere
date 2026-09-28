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

## Open gates

- Genet commit `27d20d3fc51ac5fcd2a2db231e035a3e06013ae1` admits safe retained
  layout for already transformed positioned boxes changing only 2D transform
  or numeric z-index. All 283 Livery tests pass, including four new regressions;
  disabling the fast path makes its unchanged-layout assertion fail. It has
  not been pushed or measured downstream in these browser receipts.
- Restyle code repeatedly computes sibling counts for dirty sibling roots and
  scans existing invalidation roots. These are possible quadratic costs found
  in source, not yet isolated by measurement.
- Browser physics advances one fixed step per rendered frame on the rendering
  thread. Slow rendering therefore slows simulated time and input response.
  Default pairwise exclusion is quadratic. GPU force parity is a separate
  unresolved lane; these browser runs use the CPU path.
- Actual IndexedDB application state, remote sessions and product panels still
  belong to the old presenter. Ctrl+wheel modifiers and middle-button parity,
  all five public wrappers and their product scenarios remain migration work.
  Continuous rasterization still protects asynchronous Vello buffer recovery.

The existing Mere worktree and stable native targets are retained for this
unintegrated work. The existing web target is reused. No new Cargo home,
worktree or build-target directory was created for this slice.
