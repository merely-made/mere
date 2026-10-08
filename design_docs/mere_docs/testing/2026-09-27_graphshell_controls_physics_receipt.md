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

## Current-main integration and local editor (2026-09-29)

The elapsed host slice is rebased onto published Mere `ca2351b3`, preserving
Genet `19c206873ab08ae227217892d9e74d0df18b349a`, the shared text-boundary
changes and Apparatus/Mesquite observations. The native library gate passes
578 tests: Cambium 232, winit host 16, Rootstock 45, Mesquite 19 and Pictograph
266. The separate winit scenario suite passes 24 tests. The previously
qualified scroll failure now passes on this baseline. Logs are
`Code/testing/mere/elapsed-integrated-native.log` and
`elapsed-integrated-scenario.log`.

Four focused local-editor tests also pass, bringing the native total to 606.
They verify reopening the same session/member with normalized metadata,
rejecting a stale selection without changing graph or stored bytes, retaining
durable prior values on a refused write and retrying successfully, and
refreshing Canvas metadata without changing geometry, camera, selection or
play state. Their log is `Code/testing/mere/saved-edit-native.log`.

The standalone Wasm build passes offline and locked. Updating its ignored
lock for the current manifests adds the existing `genet-text` and
`mere-apparatus` edges; it also re-resolves several Windows dependency edges
without changing the Genet revision. The primary checkout and its ignored
lock remain untouched. Bundle SHA256:
`d433cd0ab2b1c3ea1025ccfdebe0d37575322f5975e303d5c5185476f89e4045`.
Build logs are `elapsed-integrated-wasm.log` and
`elapsed-integrated-wasm-locked.log` under `Code/testing/mere/`.

The opt-in `tree.html?app=local` editor reopens the existing IndexedDB graph,
selects a stable member and provides Cambium Title/Tags fields with host caret
and IME routing. Save awaits storage acknowledgement; metadata-only canvas
refresh preserves geometry, camera, selection and play state. The migration
inventory records its bounded scope. Paired browser scripts export session
and member IDs for an independent-load comparison.

At this 2026-09-29 checkpoint, headed checks were **unverified**. The Chrome profile mounted the
11-node saved graph and its accessibility mirror while `document.hidden`
was true, with no recorded page errors. Frame withholding prevented the
scenario from advancing; the initial 120-second attempts timed out.
`tree_local_edit/progress.json` under the browser receipt directory records
that state. Windows Computer Use then stopped because it could not establish
the browser URL with sufficient confidence for its policy. No actual tab
hide/show, background-to-visible resume, completed save/reopen or new live
performance receipt is claimed.

The visibility probe and intentional hidden-timing failure scenario are
implemented, but require a real visible browser window to finish. A hidden
event counter now invalidates timing windows even when no hidden frame was
rendered. The active hide/show probe requires preserved geometry and zero
first-resume steps; an initially hidden mount permits analytic resize on its
first draw while requiring its preexisting accessibility mirror.

## Current-main refresh and headed local editor (2026-09-30)

Published Mere main `da2940b6` is merged into the elapsed-host branch at
`650f8541`, preserving the newer Gaz, Apparatus and generated-name changes.
Both manifests use Genet `c5470fcbc12805f0369c70f34a18178158fbe2d5`.
NetRender `9607d16f1907f6c2085648ae96abcaa30d7c3d41` and Vello 0.10.1
remain the rendering dependencies. The standalone web build uses the exact
published generated-name lock receipt, copied into this worktree's ignored
lock; the primary checkout's lock remains untouched.

Fresh offline locked gates pass 578 library tests, 25 winit scenarios and
four focused local-editor tests, for **607 native tests**. The formerly
qualified scroll failure remains green. Logs are
`Code/testing/mere/tree-final-native.log`, `tree-final-scenario.log` and
`tree-final-storage.log`. The standalone Wasm build passes; its log is
`Code/testing/mere/tree-final-wasm.log`.

The tested bundle includes the merged source plus the detail-panel key/value
spacing correction and `scenarios/visibility_entry.html` fixture. Bundle SHA256:
`a0fa9dbcf2bdf4b97208d7f18702457cf2e10861bd69a096e269ec3f68cd654b`.
Standalone web-lock SHA256:
`f9aaf5929ecf9dc37157d641d09a17384b22d34bc7abbaafa16fa6614c264560`.
These hashes identify the browser artifact at this checkpoint; they do not
claim that the remaining integration commit has already been published.

### Saved metadata and controls

The rendered `p4_tree_saved_edit` and separate-load `p4_tree_saved_reopen`
both pass with no page errors. They retain session
`a6c5e37f-1f15-482e-8dbb-9374feada441` and selected member
`81b91a6b-52ef-5751-b497-15f711cf9c70`, title `TreeSavedTitle`, and tags
`alpha, beta`. The save reports durable success, and the new load reports
reopening the existing graph. Receipts are `tree_local_edit/` and
`tree_local_reopen/` under `Code/testing/mere/scenarios/graphshell-web/`.
This closes the bounded headed Title/Tags persistence gate; it does not
complete the broader product migration.

Fresh `p4_tree_controls` and `p4_tree_elapsed` scenarios pass with no page
errors, in `tree_final_controls/` and `tree_final_elapsed/`. Their behavioral
checks cover captured dragging, focused keyboard controls, frozen Pause,
explicit Restore and bounded elapsed stepping. The small detail-panel spacing
correction was also checked in the rendered editor.

### Actual background-tab initialization

The visibility entry fixture opens the graph in a real background tab. After
32,004.3 ms hidden, the probe reports zero hidden producer calls, a preexisting
accessibility mirror, and a first resumed draw with zero physics steps and
zero discarded time. The next 12 producer frames complete successfully.
This paused initial-mount case requires the mirror and baseline reset; it
permits first-draw analytic resizing. It does not establish moving-graph
geometry preservation through hide/show. Raw receipt: `tree_visibility_initial/`.
Moving-graph hide/show and the intentional hidden-timing failure remain
pending at this checkpoint.

### Fresh live diagnostic

The 512-node generated graph, seed 7, completes a visible 14-frame live
window with no page errors. Logical graph size is 1282 by 627, physical
2564 by 1254. All 512 nodes are visible; paint counts remain 3,588 before
and after culling. The elapsed configuration permits three steps per frame,
and every measured frame uses three steps. Discarded debt ranges from
216,600 to 428,900 microseconds per frame.

| Moving nodes | Interval p50 / p95 ms | Physics p50 ms | Mutation/restyle p50 ms | DOM frame p50 ms |
| --- | --- | --- | --- | --- |
| 512 | 312.5 / 416.0 | 6.6 | 179.5 | 99.9 |

Raw receipt: `tree_final_live_512/`. The 2,000-node run is still in progress.
This dev-build diagnostic records the present workload. It is not a
controlled improvement/regression comparison against the earlier runs, which
used different physics advancement and earlier stack revisions. GPU timestamp
spans continue to include queue idle.

*Annotation, 2026-10-06 (S14 pass):* the 2,000-node run had finished when
this section was committed (`108647cb`, 2026-09-30 07:26). Its raw receipt,
`tree_final_live_2000/` (written 02:42, with `scenario.done` reading
`RESULT ok`), records a visible 14-frame live window with no errors, the same
seed and logical and physical sizes as the 512-node run, all 2,000 nodes
visible, 14,003 paint items before and after culling, three steps in every
frame, and discarded debt of 1,483,300 to 2,483,300 microseconds per frame.
The figures are read from that `result.json`, not rerun, and carry the same
dev-build qualifications as the row above.

| Moving nodes | Interval p50 / p95 ms | Physics p50 ms | Mutation/restyle p50 ms | DOM frame p50 ms |
| --- | --- | --- | --- | --- |
| 2,000 | 2217.3 / 2486.2 | 81.5 | 1535.7 | 461.8 |

Mark reports that current physics feels slightly laggy but acceptable. His
observation is separate from the generated-graph timings above. His hypothesis
that different layouts and physics laws can yield different results at
different scales remains untested here. Future comparisons should identify
layout/law, node and edge counts, visible count and elapsed-step settings
before drawing broader conclusions.

## Physics panel on the tree (2026-10-01)

Branch `tree-physics-panel` in `worktrees/mere-tree-physics`, based on main
`d91a49f0`. This carries out the plan's §1 phase-4 physics-panel rulings for
14 of the 16 scenarios; `physics_remote_board` and `c4b1_live_board` belong to
the separate remote-session slice and are not covered here.

### What changed

- Pictograph `Canvas::set_physics_choice` applies sources, overlays and law
  with one rebuild and one settle, sources first and the law last;
  `Canvas::physics_choice` reads the live choice back as the existing
  `PhysicsChoice`. A test-only rebuild counter shows one rebuild per apply,
  the same force set as the old law-last setter sequence, and custom
  detection.
- `graphshell::canvas_physics` (beside `canvas_controls.rs`, DOM-free) holds
  the typed actions: `apply_arrangement`, `advance_arrangement`,
  `apply_physics`, `apply_profile`, `profile_id` (`custom` when no profile
  names the pair), `ticked_overlays` and `arrangement_choices`, which is
  `CANVAS_LAYOUT_STRATEGIES` plus free. The arrangement transition moved here
  from `web.rs`. The old page's `apply_arrangement_from_form`,
  `apply_physics_from_form` and `apply_profile_from_form` now read the DOM
  and call these, so there is one implementation of each Apply.
- The tree has a docked "Graph tools" region beside the canvas whose first
  section is "Arrangement and physics": Cambium selects for Arrangement,
  Physics law, Kinds, Mass, Depth and Profile, an "Overlays" group of eight
  labelled checkboxes, and the Apply arrangement, Apply physics and Apply
  profile buttons, with a status line. After each Apply the controls follow
  the canvas; the profile picker shows "Custom (no profile)" when the pair
  names none. The same component mounts on the fixture route and on
  `tree.html?app=local`. The law choice is not persisted.
- New tree-lane observations: `ready`, `layout`, `physics-law`,
  `physics-overlays`, `physics-profile`, the three `physics-*-source` fields,
  `panel-law`, `panel-overlays`, `panel-profile`, `panel-status`,
  `checked-overlays` (read from the rendered checkboxes' `aria-checked`),
  `canvas-nodes`, `dragging-node` and `drag-return`. The `layout-spread`,
  `layout-overlaps` and `layout-stretch` fields are computed only up to 512
  nodes, because they are pairwise. New verbs: `add-node x y url`,
  `press-focused`, `center-node url` (a camera pan that keeps zoom) and
  `release-at`, which now records the drop point.
- The standalone web manifest's seven Genet pins moved from `69a2383b` to
  root's `b1eb3af1`. Main `c6707958` had repinned root and left this
  manifest behind, so the web graph resolved two Genet copies and the locked
  build could not use main's lock. The re-resolved ignored lock has one Genet
  revision (SHA256 `c8cdb567…`, copied to
  `Code/testing/mere/tree-physics/web-Cargo.lock`).

### Native gates

All runs were offline and locked, with target `C:/t/cargo-targets/mere/tree-physics`.
Logs are under `Code/testing/mere/tree-physics/`.

- `cargo test -p pictograph -p cambium-rootstock -p graphshell` (the merge
  gate): 261 passed, 0 failed (`native-merge-gate.log`).
- `cargo test -p graphshell --features web --lib`: 228 passed, including the
  four `canvas_physics` tests for ordering, profile sync and custom detection
  (`native-graphshell-web.log`).
- `cargo test -p pictograph --features canvas --lib`: 260 passed, including
  `a_whole_choice_applies_with_one_rebuild_and_reads_back`
  (`native-pictograph-canvas.log`).
- The standalone wasm build (`CARGO_PROFILE_DEV_DEBUG=0`,
  `getrandom_backend="wasm_js"`) and wasm-bindgen 0.2.127 passed
  (`wasm-build-final.log`). Bundle SHA256:
  `8d7849a4ec716dc519e9cf2df5057f7d059506f0bbf4024900cf616a5cd28b1f`.

### Headed scenarios

The runner is a copy of `run-graphshell-web-scenario.ps1` with its own Chrome
profile (`.tree-physics-browser`), port 8761, and a sink sweep limited to that
port, so it cannot stop a concurrent lane's browser or sink. The page is
`tree.html`, the window 1400 by 900, and every run used the bundle above.
Receipts are in `Code/testing/mere/scenarios/graphshell-web/<name>/` and the
run log is `Code/testing/mere/tree-physics/run-final.log`.

| Scenario (fixture route) | Result |
| --- | --- |
| `p4_tree_physics_springs`, `_charge`, `_stress`, `_energy`, `_orbit`, `_kinds`, `_flock`, `_sync`, `_flow`, `_anneal`, `_still` | all 11 ok |
| `p4_tree_physics_profiles` | ok |
| `p4_tree_physics_add` | ok |
| `p4_tree_physics_drag` | **fail**: one release-window assertion (see below) |
| `p4_tree_physics_keys` (supplementary keyboard receipt) | ok |
| `p4_tree_physics_springs_local`, `p4_tree_physics_profiles_local` (`app=local`) | both ok, on the saved graph ("IndexedDB reopened") |

The tree scenarios keep the originals' assertions and thresholds. Controls are
chosen through `click role:combobox <name>` and `click role:option <label>`.
The old `capture; wait` became `capture; settle 2`, because the tree's `wait`
also holds for motion while the old page's held only for the capture.
`assert title` and `assert attr` became `assert snap ready`,
`checked-overlays` and `panel-*`, and `assert dom` became `assert text`.

The old page still passes `physics_drag`, `physics_profiles` and
`physics_springs` on this bundle (`p4_tree_physics_oldpage_*`).

**Drag release window (open).** The drag scenario's assertions one frame after
release (`drag-return <= 20` for resting laws, `<= 60` for moving ones) are
unstable on the tree:

| Run | Stress (≤ 20) | Anneal (≤ 60) |
| --- | --- | --- |
| Default elapsed stepping, run 1 | pass | 61 |
| Default elapsed stepping, run 2 | 31 | 68 |
| Default elapsed stepping, final bundle | 21 | pass |
| `physics_max_steps=1` | pass | 292 |

Every 300-frame reclaim, hold and overlap assertion passed in each run. The
tree advances bounded elapsed time (up to 3 steps per frame) on a 982-px
canvas, where the old page took one step per frame on a full-width canvas.
Anneal writes positions through a seeded random walk whose step shrinks with a
temperature reset at each apply. The cause is not isolated. The threshold
choice is returned to Mark; this receipt does not resolve it.

Earlier, Energy carried the web node off-screen, where the host pointer path
cannot press it (the old page pressed off-screen coordinates directly). The
tree scenario now runs `center-node` before each press. This was not ruled
and is part of the same review.

### Inspection, accessibility and controls

- **A first passing receipt was wrong.** The first Springs run passed every
  assertion, but its captures showed the tools region laid out about 1,100 px
  wide over the canvas. The producer drew at 982 by 627, yet no graph was
  visible. After the region received an explicit width, the graph and the
  docked panel both draw. Every capture of the final runs was inspected
  whole-frame. Energy, Stress, Kinds and Sync carry nodes out of the fitted
  view, which the old page's Energy capture also shows.
- **Positive control.** A temporary scenario chose Orbit and then asserted
  `physics-law == spring.rapier` and, after 300 frames,
  `physics-energy <= 1`. It failed on both, reading `orbit.gravity` and
  82479.62 (`p4_tree_physics_control/`). The scenario file was removed.
- **Accessibility mirror.** `read_page` in the Browser pane (1400 by 900,
  hidden) lists:
  - region "Graph tools";
  - section "Arrangement and physics" with its heading;
  - comboboxes Arrangement, Physics law, Kinds, Mass, Depth and Profile,
    each showing its chosen value;
  - group "Overlays" with checkboxes Hub room, Group pull, Hub pull, Depth,
    Grid, Centre, Tide and Skeleton;
  - the three Apply buttons and the status.
- **Keyboard.** `p4_tree_physics_keys` uses Cambium key dispatch:
  - ArrowDown/Enter on Physics law selects Charge;
  - Tab reaches Hub room and Space ticks it;
  - eleven Tabs reach Apply physics, and Enter applies Charge with Hub room
    (`physics-profile == custom`).
- **Status line.** The page status keeps the boot node count after `add-node`
  (it reads "11 nodes" with 22 on the canvas). This is cosmetic: the
  `canvas-nodes` observation is correct.
- **Not ruled.** The region is a fixed 300 px (279 px content plus padding).
  Its narrow-viewport behaviour is not ruled.

### Release window measured in physics steps (2026-10-01)

Mark ruled "Measure in physics steps" (plan §1, follow-up rulings).

**What changed.**
- `release-at` arms a watch at the drop point, in canvas-local px.
- The producer records the focused node's distance from that point on the
  first frame where the node is no longer held and the elapsed report shows
  at least one executed step. It also records how many steps that frame ran.
- The lane exposes the result as `drag-return-steps` and `drag-return-step`.
- Each recording also goes into the receipt log, with the distance from the
  press point and the zoom.
- `p4_tree_physics_drag` now asserts `drag-return-step <= 20` for resting
  laws and `<= 60` for moving laws, the original thresholds. Its 300-frame
  checks are unchanged.

The bundle was built with the same wasm settings
(`Code/testing/mere/tree-physics/wasm-build-steps.log`). Receipts are under
`Code/testing/mere/scenarios/graphshell-web/` in
`p4_tree_physics_drag_steps_run{1,2,3}/`.

| Law | Run 1 (px / steps) | Run 2 | Run 3 |
| --- | --- | --- | --- |
| Springs | 0.8 / 1 | 0.8 / 1 | 0.8 / 1 |
| Charge | 0.8 / 1 | 0.7 / 1 | 0.7 / 1 |
| Stress | 2.6 / 1 | 2.5 / 1 | 8.2 / 2 |
| Energy | 0.5 / 1 | 1.2 / 2 | 1.1 / 1 |
| Orbit, Kinds, Flock | ≤ 0.1 / 1 | ≤ 0.1 / 1 | ≤ 0.1 / 1 |
| Sync | 1.1 / 1 | 0.5 / 1 | 1.1 / 1 |
| Flow | 2.1 / 1 | 2.1 / 1 | 2.1 / 1 |
| **Anneal** (≤ 60) | **255.2 / 1** | **190.8 / 1** | 0.0 / 1 |
| Still | 0.0 / 2 | 0.0 / 1 | 0.0 / 1 |

Runs 1 and 2 fail only the Anneal check. Run 3 passes. Every 300-frame
reclaim, hold and overlap check passed in all three runs.

**Why Anneal is not noise.** In run 2, one step after release, the node was
190.8 px from the drop but 48.1 px from the point where it was pressed, at
zoom 1.00. Run 1 has no press-point reading: that diagnostic was added after
it. Anneal's walk moves a body at most 80 px per tick
(`seiche/src/laws/anneal.rs`, `step` 80 scaled by temperature). A
190 px jump in one tick is therefore not the walk: the node is snapping back
to where the walk left its body. The likely mechanism is that `Anneal::apply`
writes `set_translation` for every body, the held one included. The body
would then never follow the drag, and the released node returns to it.

This is not isolated. The threshold is unchanged and the case is returned as
a fork, as the ruling directs.

**Positive control.** A temporary scenario released Springs and asserted
`drag-return-step >= 50` and `drag-return-steps == 0`. It failed on both,
reading 0.8 and 1 (`p4_tree_physics_stepcontrol/`). The file was removed.

### Anneal leaves pinned bodies alone (2026-10-01)

Mark ruled "Seiche: skip non-dynamic bodies". The coordinator confirmed the
cause. `Anneal::apply` called `set_translation` on every accepted body,
including one that `Simulation::pin` had made kinematic. In rapier 0.33 that
call also rewrites `next_position`, the kinematic target, so a dragged node's
body never followed the drag.

**The fix.** `Anneal::apply` now skips bodies that are not dynamic. They
remain in the energy as neighbours.

**The audit.** In `seiche/src/laws`, `seiche/src/overlays` and Hold, no
other position write exists.
- Every other law and overlay uses `add_force`, which rapier applies to
  dynamic bodies only.
- Orbit's one-time kick, Hold and Anneal's velocity reset use `set_linvel`,
  which rapier ignores on kinematic position-based bodies.
- Outside the audit, `CouplingForce`'s FlowAdvect response
  (`seiche/src/coupling_force.rs`) has the same `set_translation` pattern.
  It is recorded, not changed. `sync.rs` writes authority positions by
  design.

**Tests.** Logs are under `Code/testing/mere/tree-physics/`.
- `a_pinned_body_stays_at_its_kinematic_target` pins one of eight nodes and
  runs 120 ticks. Run before the fix, it failed: the pinned body was at
  (58.4, 93.9) against a target of (400, −300)
  (`seiche-anneal-before-fix.log`). It passes with the fix.
- Seiche passes 98 tests with default features (`seiche-default.log`) and 94
  with `--no-default-features` (`seiche-no-default.log`).
- Pictograph `--features canvas --lib` passes 260
  (`pictograph-canvas-anneal.log`).
- All runs were offline and locked. The wasm bundle builds
  (`wasm-build-anneal.log`).

**Headed runs.** `p4_tree_physics_drag` passes three runs in a row
(`p4_tree_physics_drag_pinned_run{1,2,3}/`). Across all eleven laws the
first-step release readings are 0.0–3.0 px. Anneal reads 0.7, 0.0 and 0.0 px,
with the node about 220 px from the press point, where it was dropped. Every
300-frame check passes. `p4_tree_physics_anneal` passes
(`p4_tree_physics_anneal_pinned/`), and its cooling capture was inspected
whole-frame.

### FlowAdvect, narrow viewports and the storage line (2026-10-01)

These carry out the rulings at `ea604bf4`.

**FlowAdvect.** `CouplingForce`'s FlowAdvect response now skips non-dynamic
bodies (commit `a3a455db`). The test
`flow_advect_leaves_a_pinned_body_at_its_kinematic_target` failed before the
fix, with the pinned body held at (120, 0) against a target of (−200, 50)
(`seiche-flowadvect-before-fix.log`). It passes after. Seiche passes 99
tests with default features and 95 with `--no-default-features`
(`seiche-default-flow.log`, `seiche-no-default-flow.log`).

**Narrow viewports.** The breakpoint is `TOOLS_DOCK_MIN_WIDTH`, 900
logical px: the 300 px region plus a 600 px minimum canvas. That minimum
comes from two measurements:
- a node centred by `center-node` and dragged 220 px needs 476 px;
- the fitted fixture spans about 320 px.

Below the breakpoint the region is not rendered. A "Graph tools" button with
`aria-expanded` joins the Graph controls row, and opening it puts the region
in an absolute overlay on the right of the canvas row, with the canvas taking
the full width.

`p4_tree_tools_narrow` runs in a 700 by 900 window, giving a 687 px logical
tree, and passes:
- the region starts collapsed;
- the toggle opens it;
- Charge applies through the overlay;
- clicking the toggle closes it;
- Enter on the focused toggle opens and closes it.

The collapsed and overlay captures were inspected whole-frame. The overlay
box ends at its content height rather than stretching to the bottom inset,
the Genet trait already noted for absolutely placed boxes.

**Storage line.** On `app=local` the region begins with a status line,
"Storage: IndexedDB reopened · persistent". The floating panel over the canvas
shows only save feedback, and only after a save.

**Runs and checks.**
- The wide fixture run `p4_tree_physics_springs_tools/` passes, docked.
- The wide `app=local` run `p4_tree_physics_springs_local_tools/` passes,
  with the storage line in the region and none on the canvas. Both were
  inspected whole-frame.
- The mirror, read through `find` in the Browser pane, lists button "Graph
  tools" with `aria-expanded="false"` at 700 px, where the region is absent,
  and status "Storage: IndexedDB reopened · …" at 1400 px. The pane's
  profile reported not persistent.
- Graphshell `--features web --lib` passes 228
  (`native-graphshell-web-tools.log`). The wasm build passes
  (`wasm-build-narrow.log`).

### The remote session on the tree (2026-10-01)

Branch `tree-remote-session` in `worktrees/mere-tree-remote`, from main
`f7c5873c`, with main `f4e4726c` merged in as `fcf74f47`. This carries out
the plan's §1 remote-session rulings, through the follow-up rulings of
`f4e4726c` (one leaf whose producer picks the scene; 24 px board margins).
Logs, receipts and the runner copy's output are under
`Code/testing/mere/tree-remote/`.

**What moved.**
- `graphshell_client::remote` holds the op sequencing that was the old
  page's `BrowserHost`: `RemoteSession` (the op in flight, accept → poll,
  reject → resnapshot, bells → resume by diff, the disconnect, reconnect
  and nudge lifecycle) and `ActionForm` (status, count, draft, target). It
  never sends; the host drains its outbox and its receipt events. The
  existing `SessionDriver` API is unchanged (`840c63de`).
- Pictograph gains `canvas::board_scene` (`acc8920c`): `BoardScene`,
  `BoardCard`, `BoardFootprint` (from sceno's), `BoardBackdrop`, `BoardFit`.
  It paints the board at the `PhysicsBoard`'s positions; slots are read,
  never written.
- `graphshell::remote_board` maps a mounted scene to a `BoardScene` and
  keeps the board, for both pages. `web_rtc_link.rs` is the WebRTC join,
  pumps, rejoin and nudge, generic over a `RemoteHost` trait (`02a58aad`).
- The old page delegates. `web_remote.rs` went from 1152 to 438 lines and
  `web.rs` from 2117 to 1947. Its board is painted with a 50/50/116/64 fit,
  the old formula.
- Cambium's web-host mirror now writes a node's description as
  `aria-description`. Before, Genet computed it and the mirror dropped it
  (`78745a40`).
- The tree's Graph tools region gains a "Remote session" section after
  Arrangement and physics (`e9d95554`). It holds:
  - the session switch, two `aria-pressed` buttons, "Local Mere" and
    "Remote mount";
  - the active-session line, a status reading "Remote projection · N
    objects · revision R" or "Local Mere · N objects";
  - one button per advertised intent, named by its label and described by
    its explanation;
  - the draft form for an intent with inputs: one select per field, with
    "Choose…" unset, then submit and Cancel;
  - while a link exists, a "Link" group with Disconnect, Reconnect and
    Nudge host;
  - the action status.
- The one canvas leaf's producer paints the board instead of the graph
  while remote is shown, with even 24 px margins. The local law does not
  step meanwhile, as on the old page, and its frame clock resets on return.
- The tree lane busy rule follows "Moving counts only when local is
  shown": a pending capture, a remote answer in flight (join, request,
  queued bell, or link step), or local motion while local is shown.

**Native gates.** All offline and locked, target
`C:/t/cargo-targets/mere/tree-remote`, rerun after the merge (`*-merged.log`):
- graphshell-client 59, including 13 new tests driving the session against a
  scripted board endpoint. Dropping the poll after an acceptance or the
  resnapshot after a refusal fails two of them.
- Graphshell `--features web --lib` 230; pictograph `--features canvas
  --lib` 267.
- The merge gate (`-p pictograph -p cambium-rootstock -p graphshell`) 262.
- `remote_session_live` drives `RemoteSession` against the real
  `LiveEndpoint` along the c4b1 and c4b3 path.
- `cambium-genet-web-host --lib` passes 7, including the description test.
- The standalone wasm build passes with the existing lock (SHA256
  `c8cdb567…`), seeded from `tree-physics/web-Cargo.lock`.

**Headed runs.** A runner copy with its own Chrome profile
(`.tree-remote-browser`) and sink port 8741 uses the host fixture built from
the branch. The host binds 192.168.4.36 and signals on 8788. Each scenario
ran in a 1400 by 900 window.

| Scenario | Route | Result |
| --- | --- | --- |
| `c4b1_live_board`, `physics_remote_board`, `c4b3_reconnect` | old page, unchanged | all ok |
| `p4_tree_c4b1_live_board`, `p4_tree_physics_remote_board`, `p4_tree_c4b3_reconnect` | `tree.html?signal=` | all ok |
| the same three | `tree.html?app=local&signal=` | all ok |
| positive control (the forbidden intent asserted accepted) | `tree.html?signal=` | fails, as it should |

- The tree copies keep the originals' assertions. `act` and `dom click`
  became `click role:button <label>`, and `assert dom` became `assert text`.
- The old page's board geometry matches its 2026-09-02 receipt at the same
  frame size.
- On the tree the board draws in the leaf, Charge pushes the pair apart,
  and the reconnect shows three cards.
- The positive control read "Rejected · this endpoint advertises the action
  and refuses it · revision after 1" and revision 1.
- Each receipt's semantic tree, collected from the browser mirror, lists:
  - group "Session" with both buttons and their pressed state;
  - the active-session status;
  - group "Remote actions" with both intents and their descriptions;
  - group "Link" with its three buttons;
  - the action status.

**Open.**
- On `app=local` at 1400 by 900 the region is taller than the window.
  The storage line pushes the Link group's buttons against the bottom edge,
  and the action status falls below it. The scenarios still pass; the
  captures do not meet "nothing clipped". How the region handles height is
  returned as a fork.
- The live fixture advertises no intent with inputs, so the tree's draft
  form is built but not exercised headed over WebRTC. This is returned as a
  fork.
- In the Browser pane, whose Chromium hides host candidates behind mDNS, the
  join fails as expected ("offer has no usable ICE candidates"). A data
  channel closure in `webrtc_carrier` then throws "closure invoked
  recursively or after being dropped". This path predates the slice. The
  headed Chrome receipts record no page errors.

### Graph tools overflow, the draft proof and card titles (2026-10-02)

These carry out the rulings at `bbc89994`, merged into `tree-remote-session`
as `c9145545`. The runs used the final bundle, SHA256 `1ba3905d…`. The
headed log is `Code/testing/mere/tree-remote/run-round3-final.log`.

**Draft proof.**
- `LiveEndpoint` advertises a third intent, "Append a coloured card", with
  one required choice of colour (Red, Blue, Gold). Accepted, it appends a
  card titled with the colour, "Card 1 · Blue" (`b8d56626`).
- `RemoteSession::submit_draft` now checks a draft composes before asking.
  An unset required value reads "Choose required values · …" and keeps the
  draft open. Before, it failed the whole session as a link error.
- `p4_tree_remote_draft` opens the draft and submits it unset, which is
  refused before the endpoint is asked. It cancels, opens the draft again,
  chooses Blue and submits, which is accepted. Revision, cards and the
  "Card 1 · Blue" title move.
- The form's button reads "Submit", because the action's own label is on
  its button in the actions group.

**Overflow.**
- Arrangement and physics and Remote session are each a Cambium disclosure,
  both open at first, inside a Graph tools region that scrolls vertically.
- Showing the remote board opens its section. The storage line stays at
  the top of the region.
- The draft scenario closes and reopens Arrangement and physics.
- Captures follow a new tree-lane verb, `reveal <role:name|.class> [text]`,
  which scrolls the match into view as a click would. Each capture shows
  the state it records.
- The first closed-section capture showed the section's whole content
  still laid out. Genet's Cambium UA sheet has no `[hidden] { display:
  none }`. Cambium's closed disclosure, accordion and tree panels now also
  carry an inline `display: none` (`00d695df`). The rule's proper home is
  Genet's UA sheet.

**Card titles.**
- `BoardCard` carries a title, taken from the card's presentation
  semantics. `BoardScene::paint_titled` sets it in the card through Livery's
  standalone layout and paint, over a `BoardText` holding the page's own
  font (`6005ea08`).
- The Remote session section lists the titles as list "Cards".
- The fit now frames the cards' edges: the live board's bounds cover card
  centres only, so cards hung half outside the top margin. The old page
  keeps framing centres (`frame_edges: false`), so its board stays where
  its chrome expects it. Its receipts confirm the geometry is unchanged.
- The board leaf does not yet project one node per card into the mirror.
  It is returned as a fork: Cambium's leaf semantics carry one role and one
  name, with no children.

**Gates.** All offline and locked, logs `native-*-r3.log`:
- graphshell-client 59; graphshell `web` lib 230; pictograph `canvas` 269;
- the merge gate 263; Cambium 244; `cambium-genet-web-host` 7;
- `remote_session_live` with the WebRTC join loopback, 4.

The wasm build passes. The host fixture was rebuilt from the branch.

**Headed** (1400 by 900, all ok, no page errors recorded):

| Scenario | Fixture route | `app=local` |
| --- | --- | --- |
| `p4_tree_remote_draft` | ok | ok |
| `p4_tree_c4b1_live_board` | ok | ok |
| `p4_tree_physics_remote_board` | ok | ok |
| `p4_tree_c4b3_reconnect` | ok | ok |
| old page `c4b1_live_board`, `physics_remote_board`, `c4b3_reconnect` | ok, unchanged, third action present | — |

No old-page assertion counts actions. The captures were inspected
whole-frame:
- titles sit in every card, with the top margin held;
- the open draft and its status are in view on `app=local`;
- a closed physics section leaves the remote section unscrolled.

The receipts' semantic tree, from the browser mirror, lists:
- the two section triggers with `aria-expanded`;
- list "Cards" with each title;
- the three described actions;
- group Link;
- the statuses.

### The board's cards in the mirror, and Remote session closed at start (2026-10-02)

These carry out the rulings at `7248e74d`, merged as `b690a907`. Final
bundle SHA256 `7d1b4a0e…`; headed log `run-round4.log`.

**Producer semantics** (`52f6a1ec`).
- `TextureProducer` gains a defaulted `semantics()` returning
  `ProducerSemantics`: an optional role and name for the slot, and child
  `ProducerNode`s (role, name, rectangle in the slot's layout pixels).
- Roles are a small neutral set (list, listitem, group, img,
  graphics-object) that each host lowers itself.
- `ProducerRegistry::semantics` reads it by key. `Accessibility::sync` takes
  the producer registry; its call site and the winit harness follow.
- The web mirror names the slot from the producer and writes its children
  as elements placed where they are drawn.
- The winit a11y host sets the slot's role and name, and appends AccessKit
  children with bounds offset from the slot.
- Drawn nodes take ids outside the DOM's range. The graph canvas returns
  none for now.

**The board** (`9adc4415`). While the board is shown, the canvas producer
describes its slot as list "Remote board · N cards", with one list item per
card from `BoardScene::card_rects`. Remote session starts closed and opens
once when a link is established, or when remote is shown.

**Found headed.** The Browser pane's mirror first named the empty board's
slot "Graph": the producer's name only replaced the author's when it drew
children. A producer now always names its slot. A native test covers the
empty case.

**Gates** (offline, locked, `native-*-r4.log`):

| Gate | Passed |
| --- | --- |
| cambium-rootstock | 52 |
| cambium-winit-a11y, including "a producer's children reach the tree" | 3 |
| cambium-genet-web-host | 8 |
| cambium-genet-winit-host | 153 |
| cambium | 244 |
| graphshell-client | 59 |
| graphshell `web` lib | 230 |
| pictograph `canvas` | 269 |
| merge gate | 263 |
| live endpoint plus join loopback | 4 |

The wasm build passes.

**Headed** (1400 by 900; all ok; no page errors):

| Scenario | Fixture route | `app=local` |
| --- | --- | --- |
| `p4_tree_remote_draft`, `p4_tree_c4b1_live_board`, `p4_tree_physics_remote_board`, `p4_tree_c4b3_reconnect` | all ok | all ok |
| `p4_tree_remote_absent` (no link: section closed) | ok | ok |
| old page `c4b1_live_board`, `physics_remote_board`, `c4b3_reconnect` | all ok, unchanged | — |

- A positive control asserting a "Card 9" in the slot failed, reading
  `Card 0@431,24,120,80`.
- On every receipt ending with the board shown, on both routes, the
  semantic tree lists list "Remote board · N cards" with one named item
  per card.
- Each item's mirror box was checked against the capture. All four inner
  corners are card colour, and just outside the left and top edges is not.
  Every receipt matches.
- In `c4b1_appended` the cards are painted at CSS x 361–480 and 501–620,
  y 119–198. The mirror boxes are [361, 119, 120, 80] and
  [501, 119, 120, 80].
- `read_page` in the Browser pane lists list "Remote board · 0 cards" at the
  canvas slot. The pane cannot join, because its Chromium hides host
  candidates behind mDNS, so the joined state is verified through the
  headed receipts.
- The Genet gaps (`[hidden]`, `text-align`) belong to a separate Genet lane.
  Cambium's `display: none` and left-aligned titles stay meanwhile.

### Genet `bd3e8861b29`: `[hidden]` in the UA sheet, centred titles (2026-10-02)

Branch `genet-repin` in `worktrees/mere-genet-repin`, from main `f02d9d35`.
This carries out the one-tree plan's "Genet gaps" ruling: "Fix in Genet now,
separate lane". Logs, locks, the runner copy and receipts are under
`Code/testing/mere/genet-repin/`, and the target is
`C:/t/cargo-targets/mere/genet-repin`.

**Repin** (`7a0950da`).
- Genet main `bd3e8861b29` carries `4ac56bbbe0b` (`[hidden]` generates no
  box in the Cambium UA sheet) and `6fca091dc26` (non-wrapping lines follow
  `text-align`) on top of `b1eb3af1971`. Between the two only
  genet-livery's source changed; no manifest did.
- 28 root rows and the standalone web manifest's 7 rows moved. A sweep of the
  whole worktree for the old SHA found no other manifest naming it.
  `genet_web_smoke` keeps its older `5ae30cad` pin, as
  `cambium_docs/testing/local-genet-development.md` records.
- The root lock moved its 64 Genet entries and nothing else (SHA256
  `17c6f3ee…`). The ignored web lock, seeded from `c8cdb567…`, moved its 19
  entries and nothing else (SHA256 `2e28ea2f…`, copied to
  `genet-repin/web-Cargo.lock`).
- `cargo tree` finds one Genet revision in each gate crate's graph:
  cambium, cambium-rootstock, both hosts, pictograph, graphshell and
  graphshell-client. The workspace still holds a second, from before this
  change: djinn's `knot-editor` `855cb75d` pin brings `fleece` and
  `layout-dom-api` in at Genet `34626a6c`.
- The unused-patch warnings name only `boa_engine`, `boa_gc` and
  `iroh-mdns-address-lookup`, exactly as before the repin. The web build
  reports none.

**Closed panels** (`1182f642`).
- `disclosure.rs` is back to its content before `00d695df`. Closed
  disclosure, accordion and tree panels carry `hidden` alone.
- New `cambium-rootstock` tests (`owned_layout/hidden_panel_tests.rs`) lay
  each component out through the host's own Livery session. A closed panel
  must have no fragment, and the open one beside it is the positive control.
- At Genet `b1eb3af1971` they passed with the inline style and failed without
  it: each closed panel had an 18 px box. At `bd3e8861b29` they pass without
  it.

**Card titles** (`de115df8`).
- The title sheet has said `text-align: center` since `6005ea08`, so no
  style change was needed.
- `titles_are_centred_in_their_cards` reads each title's glyph origins from
  `paint_titled`. At `b1eb3af1971` it failed, with a left gap of 6 (the
  padding) against a right gap of 242 in a 280 px card. At `bd3e8861b29` it
  passes.

**The §2 label finding.** The mere view's labels before `53f625fc` were the
same shape as the titles: `.graph-canvas-swatch-label` set
`white-space: nowrap`, inside an absolute, 160 px box with `text-align`. A
temporary rootstock probe laid out that exact style, left, centre and right,
and was not committed (`genet-repin/label_align_probe.rs.txt`).
- At `b1eb3af1971` all three began at the box's left edge.
- At `bd3e8861b29` the centre label had gaps of 56.1 and 58.3 px, and the
  right label ended 2.2 px from the right edge.
- The probe's text fits its box, so `text-overflow: ellipsis` was not
  exercised.

**Native gates** (all offline and locked, `gate-*.log`):

| Gate | Passed |
| --- | --- |
| cambium | 245 |
| cambium-rootstock | 55 |
| cambium-genet-web-host | 8 |
| cambium-genet-winit-host | 153 |
| pictograph `canvas` lib | 270 |
| graphshell `web` lib | 230 |
| graphshell-client | 59 |
| merge gate | 266 |

`c4_webrtc_host` builds from the branch, and `cargo check --workspace
--all-targets` passes.

The standalone wasm build passes with `CARGO_PROFILE_DEV_DEBUG=0`,
`getrandom_backend="wasm_js"` and wasm-bindgen 0.2.127. Bundle SHA256:
`d9d7f5dab0db8f848be6fec7655dcde2970bc12daeba0a86722784f67f5ac4c6`.

**Headed** (`tree.html`). The runner copy uses its own Chrome profile
(`.genet-repin-browser`), sink 8771 and fixture signal port 8798. The log is
`run-headed.log`.

| Scenario | Window | Result |
| --- | --- | --- |
| `p4_tree_physics_springs` | 1400 by 900 | ok |
| `p4_tree_tools_narrow` | 700 by 900 | ok |
| `p4_tree_remote_absent` | 1400 by 900 | ok |
| `p4_tree_c4b1_live_board` (fixture over WebRTC) | 1400 by 900 | ok |
| `p4_tree_remote_draft` (fixture over WebRTC) | 1400 by 900 | ok |

Every capture was inspected whole-frame:
- `remote_absent_closed` is pixel-identical to the tree-remote lane's
  capture, which had the inline style. The UA rule reproduces it exactly.
- In `remote_physics_closed` the closed physics section shows only its
  trigger.
- Against the tree-remote lane's five board captures, every changed pixel
  lies in the title text band (capture y 306–327). The titles moved, and
  nothing else did.
- In `c4b1_appended`, "Card 0" is centred in its card. Measured in capture
  pixels, the card spans x 722–960 and the title's ink 805–877, so both
  centres are at 841.

**Not run here:** the `app=local` route, and the old page's three originals.

## Durable address/file intake on the current stack (2026-10-08)

**Source:** intake `da466027afe2a2a9e543962f8927a05c037c6833`, archive fingerprint
repair `f279692f9dfefbbc24c728c3159aaf5bdfa542f4`, on published Mere
`79fbbeb75b04e7aa0b2c91f9ec68d4b435909931`. This incorporates the coherent
Genet `15713014e2e23b887360471552f75f60684f5384` repin and current vault-lock L3
source. Qualification is scoped to this source, not a release or all-family
acceptance.

**Behavior:** `tree.html?app=local` adds an address and optional title through
`MereHost`, or actual chosen bytes through Cambium's existing `open_file`
callback. SHA256 drives the existing NI file identity; name, media type, byte
length and modification time remain owner facets. The same app performs its
existing persistence operation. Success selects and opens the member only
after acknowledgement and structural Canvas ingest; a failed write keeps its
pending member for Retry intake. Empty addresses/unnamed files refuse before
mutation. Repeated identities reuse members. The ingest preserves existing
positions, camera, roles and physics choice; ordinary topology settling can
continue after ingest. Selection/detail changes are guarded during the chooser
and save, including mirror reader actions, while Tab still traverses focus.

The saved-product panel now anchors inside the tree body below the wrapping
toolbar. Current Cambium single-line field markers receive explicit width and
height, preserving the shared host's text/caret path and clipping long values.
This repairs the absent empty-field box and narrow toolbar/panel overlap found
during the initial browser pass.

**Automated native gates:** locked checks at the clean source above pass:

| Gate | Passed | Scope |
| --- | ---: | --- |
| Pandect library | 299 | Existing graph/session/archive contracts |
| Kernel undo | 8 | Exact graph revert behavior |
| Djinn resident | 40 | Scoped `resident_` library tests |
| Pandect archive process | 2 | Owner-bound archive reopen/fork and fingerprint regression |
| Djinn process targets | 8 | Embedded owner (4), validator, mere, access and reservoir (1 each) |
| Graphshell local intake | 5 | Refusals, actual byte metadata, dedup/reopen, rejected-write retry, Canvas preservation |
| Graphshell local edit | 4 | Existing metadata save/refusal/reopen/geometry regressions |

Each process target's child helper is ignored in the parent invocation and
run by its fixture. All 2,266 tracked Rust/TOML/lock files retained aggregate
SHA256 `5fa8df8a900da39f3f3331a87ccf114a9eb704fb557321ba39e44f2d4c5ca1ed`
before and after every gate; source and HEAD were unchanged and clean.
`CARGO_BUILD_JOBS=2`, `TMPDIR=/tmp`; the expanded personal-sync test remains
excluded after its earlier hang. The archive regression ignores only the
generated snapshot-envelope timestamp for state comparison, with a positive
control that changed node content differs. Visit clocks remain exact; no
archive format, undo policy or product storage changed.

**Web build:** the default WASM build and no-default-features viewer check pass
with `--locked` on the source above. The ignored standalone web lock was
regenerated after the upstream Genet pin changed, then preserved and used for
both locked checks. Official wasm-bindgen CLI `0.2.129` matches the library.
The bound bundle's WASM SHA256 is
`557657df8c073771aeb5b31a153353cb5d092597e8929a197cbe905d3093097f`; this
is a large debug qualification artifact, not a release-size/performance result.

**Browser scope:** macOS arm64, Codex in-app browser, visible document,
1280×900 and 420×900 viewport overrides. Separate loopback origins isolate
fixture stores; each save/reopen pair uses the same origin and an independent
page load. Mesquite results, semantic trees and full-frame PNGs are retained
under `/Users/markik/Code/testing/mere/one-tree-product-20261008/browser`.
Final-stack receipts start at `receipt-10.json`; earlier receipts include
interim stack checks and the superseded cancellation assertion failure.



| Final browser check | 1280×900 receipt | 420×900 receipt | Result |
| --- | --- | --- | --- |
| Address refusal/create/edit/dedup | 10 | 19 | Pass; new member then repeat keeps 12 nodes |
| Independent address reopen | 11 | 20 | Pass; exact member/session/title/tags |
| Pending chooser/detail guard and DOM cancel | 12 | 21 | Pass; original selection/count/layout preserved |
| Actual file chooser bytes and reopen | 13 | 22 | Pass; 90 bytes, `text/plain`, exact NI/name/mtime |
| Repeat file choice then independent reopen | 14 | 23 | Pass; same file member/session/facets, 13 nodes |
| Existing Title/Tags saved edit | 15 | 17 | Pass; geometry/camera/physics/identity unchanged |
| Existing Title/Tags independent reopen | 16 | 18 | Pass; exact member/session/title/tags |
| Collapsed/overlay Graph tools, apply Charge, Enter toggle | — | 24 | Pass |

All 15 final scenario results are `ok`, with no errors or gate failures and
visible documents. All 18 full frames were reviewed, including both empty
intake fields, long values/file NI, wrapped toolbar clearance and Graph tools
collapse/overlay. Actual narrow keyboard typing produced `Caret proof`; Home,
Right, Shift+Right and `X` produced `CXret proof`, and Tab reached New node
without changing graph selection. This is caret/selection evidence, not an
IME-composition or screen-reader acceptance claim.

The actual CUA chooser instrument supplies the fixture through the real
shared file callback; each subsequent independent load checks its stored owner
facets against the fixture's SHA256 and filesystem modification time. DOM
cancel dispatches the actual hidden input's `cancel` listener; the earlier
headless shared callback cancellation fixture also passes (1 test). CUA's
chooser instrument refuses an empty file list, so **OS-dialog Cancel remains
unverified**. Failed durable writes/retry are proven with the native backend
fixture, not injected browser storage failure. IndexedDB reports “not
persistent, may be evicted” in these runs.

Logs, hashes, exact source snapshots, the final ignored web lock, all browser
results/captures and stable-identity pair comparisons are preserved in
`/Users/markik/Code/testing/mere/one-tree-product-20261008/qualification-receipt.json`
and its linked `native/final-stack/receipt.json` and `wasm/receipt.json`. The
pre-existing lane formatting findings and documentation audit debt were
compared with the source baseline; this slice adds no new findings.

This advances V2b step 4 only. Remaining detail/product controls, all five
public pages, the full scenario/accessibility/live-performance wall, legacy
presenter retirement, V2b step 5's mounted session panel and actual native
consumer adoption retain their independent gates.

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
- The local IndexedDB graph, Title/Tags editor and bounded address/file intake
  have passing opt-in tree save/reopen receipts. OS-dialog Cancel remains
  unverified; the cancellation fixture exercises the real DOM listener and
  the shared headless callback, not operating-system dialog interaction. The WebRTC remote session reached the tree on
  2026-10-01 (section above); the in-process canary and the other
  product panels still belong to the old presenter. Ctrl+wheel modifiers and
  middle-button parity,
  all five public wrappers and their product scenarios remain migration work.
  Continuous rasterization still protects asynchronous Vello buffer recovery.

The elapsed-time slice is isolated in `worktrees/mere-canvas-elapsed` because
the primary checkout has concurrent work. The reusable web target is
`C:/t/cargo-targets/mere/web`. No isolated Cargo home was created.
After verification, `cargo clean` removed the slice's isolated native output
at `C:/t/cargo-targets/mere/canvas-elapsed` (3.2 GiB). Logs and browser receipts
remain outside that target.
