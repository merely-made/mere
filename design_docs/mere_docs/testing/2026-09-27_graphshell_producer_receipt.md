# Graphshell canvas producer: browser rendering and culling

Date: 2026-09-27. Phase 3 of the [one-tree plan](../implementation_strategy/2026-09-25_graphshell_one_tree_plan.md).
This receipt describes the `reservoir-v2` branch. Integration with main's
Mesquite integration is separate from these historical measurements. Mark
approved proceeding to phase 4 on 2026-09-27; performance and live physics
remain open.

## Source and build

- Vello 0.10.1, netrender `9607d16f1907f6c2085648ae96abcaa30d7c3d41`,
  Genet `92b249af5b200f78d036d8e9b0cb4fe025610740`. The dependency-only
  Mere update is already on main as `815279cf`.
- `95cd5f27` fits buffered analytic positions before the first canvas frame.
- `c190bcb6` culls offscreen underlay and node paint before scene lowering.
- The phase-3 tree continuously rasterizes each requested frame, as the
  presenter does. It previously cached Vello's incomplete first texture
  after the layout settled. Continuous rendering allows the asynchronous
  buffer-size readback to repair that frame. An event-driven host still
  needs an explicit renderer-completion contract before caching textures.
- Locked offline wasm32 dev build, debug information disabled, the existing
  five physics-crate optimization overrides retained, wasm-bindgen 0.2.127.
  This is not a release performance receipt.
- Bundle SHA256:
  `2fe1a3ec69ebe5ac58b3643c96ca8cbda1b2fc3edd65b9388492b1a2dda53a70`.
  Full artifact and scenario hashes:
  `Code/testing/mere/mere-p3-culling-build.json`.

## Verification

`cargo test --offline --locked -j 2 -p pictograph --features canvas,vello
--lib --test vello_headless -- --test-threads=1`: 254 unit tests and two GPU
tests pass. The unit suite includes a GPU comparison that preserves every
pixel after culling and detects a deliberately removed crossing edge.
The fit regression covers an analytic layout buffered before the first
frame. The standalone wasm build, JavaScript syntax check and new-file
formatting check pass. Pictograph Clippy completes with existing warnings;
it reports none in the new culling module or fit change.

Culling leaves simulation, DOM layout, hit testing and resource tables in
place. It considers each caption's paint bounds separately from its body,
retains edges that cross the viewport even when both endpoints are outside,
and preserves transform stacks and uncertain filter/shadow/fragment extents.
The 2,000-offscreen-rectangle case retains only the visible rectangle before
scene lowering. A graph fitted entirely into view has little to cull.

## Headed runs

Runner: `Code/testing/mere/scripts/run-graphshell-web-scenario.ps1`, with
`-Web Code/worktrees/mere-reservoir/ports/graphshell/web`, port 8747, a
dedicated headed Chrome profile, and seed 7 for each 2,000-node graph.
The final four receipts and captures are under
`Code/testing/mere/scenarios/graphshell-web/p3_culling/`.

Both pages use a paused analytic layout and continuously render. The windows
are startup and steady rendering; they do not measure moving physics. Each
large window reports 32 frames around the ruled 30-frame wait, with 60
frames between windows. Fixture windows report 242 frames around 240-frame
waits, with 600 frames between windows.

The logical browser viewport is 1282 by 722. The presenter rasterizes and
captures at 1282 by 722. The tree's full-page capture is 2564 by 1444; its
graph slot is 1282 by 667 logical, 2564 by 1334 physical. These are actual
host configurations, not an equal-resolution isolation of host overhead.
GPU timestamp spans include queue idle between submissions; they are not
isolated raster execution times.

All four scenarios passed. Both tree runs picked the requested node through
the host pointer path. All eight captures were verified: the tree captures
were inspected whole-frame; each presenter's startup and steady files are
byte-identical and its whole frame was inspected. The graph is visible in
every capture. Every timing window reports `hidden: false`, with no page
errors. Each browser and sink closed after its run.

Frame interval in milliseconds:

| Page and graph | Startup p50 | Startup p95 | Steady p50 | Steady p95 |
|---|---:|---:|---:|---:|
| Presenter, 11-node fixture | 14.1 | 20.3 | 10.3 | 11.1 |
| Tree, 11-node fixture | 18.6 | 27.4 | 24.1 | 58.3 |
| Presenter, 2,000 nodes | 1468.2 | 1540.9 | 1493.8 | 1595.9 |
| Tree, 2,000 nodes | 1506.2 | 1569.9 | 1553.3 | 1622.5 |

The large graph remains slow in this debug build, about 1.5 seconds per
frame. CPU work accounts for most of that interval: steady CPU medians are
1491.8 ms for the presenter and 1553.6 ms for the tree; the tree's producer
accounts for 1552.0 ms of CPU work. Culling does not eliminate DOM layout.
The tree fixture has pacing outliers (steady maximum 460.6 ms despite CPU
p95 13.7 ms); the receipt does not establish their cause. These are single
headed runs on the shared machine, not a statistically isolated overhead
benchmark. The source/runtime correctness gates pass. Mark accepted this structural
proof to proceed to Cambium, while explicitly retaining stack performance
and live physics as work to do.

## Earlier evidence

The earlier blank 2,000-node receipts are invalid rendering evidence. The
first rerun with Vello 0.10.1 still failed on the tree because of its cached
texture and stale camera fit. Its generic nonblank check counted the heading.
The short `tree_recovery/p3_tree_health` receipt then passed, with the whole
graph visible and node 0 picked through the host pointer path. Its full
pre-culling tree run passed at startup/steady interval medians of
1499.0/1494.0 ms. Those earlier window labels say `moving`/`idle` despite
the paused layout; the files are retained as historical evidence.

These measurements preceded the Mesquite integration below.
They do not claim a main-branch build. Phase 4 is now
authorized; these paused measurements do not establish physics readiness.


## Mesquite integration, 2026-09-27

With Mark's approval, `reservoir-v2` integrates main at `815279cf` and
replaces its unpublished rootstock scenario runner with Mesquite. Mesquite
owns capture scheduling, validation and completion. The web host supplies
nonblocking readback; Graphshell publishes pixels and the finished receipt.
`ProducerContext.core` remains the shared renderer access for the canvas.

Native verification passed 51 tests: 16 winit-host unit tests, 17 scenario
integration tests, 17 Mesquite tests, and the new delayed-capture regression.
The regression waits beyond the native eight-frame default and proves that
completion waits for pixels, happens once, and fails on an injected map
error. The complete standalone wasm build and bundle generation passed.

Headed `p3_tree_health` (2,000 nodes) and `p3_tree_fixture` both pass under
Mesquite, with zero page errors, visible pages and two captures each. All
four images were inspected for graph content; both node picks pass. Fixture
startup/steady frame interval medians are 18.1/18.3 ms, p95 50.6/58.4 ms.
These are integration checks, not a new isolated overhead comparison.
The large health run does not collect timing windows. Physics stays paused.

Evidence is under `Code/testing/mere/scenarios/graphshell-web/p3_mesquite`.
`mesquite-browser-build.json` records artifact/scenario hashes;
`mesquite-browser-native.log`, `mesquite-browser-async.log` and
`mesquite-browser-wasm.log` record build/test results. Existing dependency
warnings remain. The browser sinks and test instances closed after the runs.
