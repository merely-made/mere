# Mere lane handoff and branch reconciliation, 2026-10-14

**Status, 2026-10-14:** Remote `main` is the only live line. Every branch and
worktree on the ThinkPad was landed, archived as a tag, or removed. The WebGL
presentation lane landed later the same day (below); both of its acceptance
limits were closed on 2026-10-15. The resident-host port on O-PC remains open.

## Landed on main today

- `30445d2db` Specify tenant-owned browser presentation and viewer acceptance
  (family composition brief; recovered from the ThinkPad three-lanes lane).
- `4fffbd3b5` Repair the site-export fixture digest and align Graphshell web
  p2panda tags. Main's `the_site_export_compiles_with_every_relation_drawn`
  had been failing on the stale digest; all ten `host_dataset_view` tests pass.
- `b17ed532e` Merge C1 resident divination (through `75031deb6`). Murm
  suppliers take C1's carrier fix `3d9e799f` (main's `b49df894` plus one
  commit); main's netrender patch block and Graphshell `query-views` feature
  stay. Focused tests for `mere-view`, `pandect`, `graphshell-client`,
  `mere-kernel`, `djinn` and `graphshell` (lib and integration): 63 suites,
  1,619 passed, 0 failed, 19 ignored, on Linux. C1 retires Graphshell's legacy
  browser presenter (`web_main`, `web_session`, `web_events`, `web_product`,
  `web_projection`, `web_remote`, `web_view`, `web_gpu`, `web_options`,
  `web_commands`) in favour of the one-tree route.

## Landed: WebGL presentation integration (ThinkPad)

`webgl-presentation` (closed PR #8) is merged onto main. Netrender's lane
(closed Netrender PR #1, `cbdbf1e6`) was merged onto Netrender main as
`68bcddc06`; Mere pins that one revision, so the shadow-helper repair and the
direct WebGL2 session travel together. Both branches are deleted and kept as
`archive/webgl-presentation` (Mere and Netrender). `codex/site-viewer-startup`
was not merged: main already carries its root-removal cancellation and late
device destruction (`MountLifetime` with a root liveness check). It is
`archive/codex/site-viewer-startup`.

Conflict decisions: main's `MountLifetime` is the only browser disposal
authority; the lane's `Appearance` was retired and its listener, frame and
capture ownership moved onto the lifetime. `HostDisposal` now also releases an
installed presentation driver and refuses its owned captures. The lane's
tenant model, context generations, backend selection and WebGL loss/restore
are kept. Main's producer GPU-binding remap now runs once during native frame
preparation. A caller-supplied shared core still presents through the host
surface and group recovery; an installed driver restores its own surface, so
browser recovery no longer attaches a second one. Graphshell's one-tree route
keeps the lane's `dispose_tree` and boot epochs, and its startup liveness checks
both the root and the epoch.

Verified 2026-10-14 on the Renoir: 336 native host tests, 513 Graphshell, 5
WebGL portable, 28 Netrender; Chrome WebGL2, Auto without WebGPU, Firefox and
SwiftShader pass 32 viewer controls, native Vulkan WebGPU 26. Receipt:
`testing/receipts/browser_presentation_20261013/presentation/integration-20261014/`.

**Limits, closed 2026-10-15.** The 125% scale gap was fixed by aligning
composition: browser WebGPU presents the same portable scene as WebGL2, so
the comparison passes at RGB RMS 0.258/0.332/0.328. The delivery limit was
closed in three steps: supplier de-generification (Genet `e8b8606f5`, Vello
fork `276b4664`, Netrender `5fbc7b2ba`); per-route viewer artifacts loaded by
`mount.js`, with Classic Vello an opt-out in the suppliers (Netrender
`f4f9c0d87`, Genet `ae3d9f5d972`); and Mark's ruling amending R157's Wasm cap
to 3,460,300 B. Final gzip6: WebGPU 3,359,673, WebGL2 3,322,091, both passing.
Receipts: `testing/receipts/browser_presentation_20261013/presentation/limits-followup-20261014/`.
Recorded options, not tasks: slim Genet's recent browser-capture work in the
viewer, align genet-parley onto one read-fonts. Human assistive-technology and
asynchronous browser WebGPU device replacement remain separate gates.

**Re-verified 2026-10-15 on current main.** `0bb5facfe` was pushed after a
rebase without a rerun, so main was re-verified at `90ce8f12d`. That found one
break from a later lane: the Knot repin removed Graphshell's `windows-sys`
dependency without updating the viewer workspace's lockfile, so
`build-viewer.py`'s `--locked` build refused. Fixed in `137afc01e`. At that
source: hosts 337 passed / 11 ignored, portable WebGL 5, Graphshell 493 / 7
ignored (the P1 door lift moved 22 tests to `graphshell-endpoint`), build
checks and `presentation-mount.mjs` pass; `build-viewer.py` passes every cap
(WebGPU 3,359,746, WebGL2 3,322,097 gzip6); Chrome WebGL2 and Auto without
WebGPU, Firefox and SwiftShader pass 33 controls, WebGPU 27, and Auto with a
Vulkan WebGPU device loads the WebGPU artifact; same-state comparisons pass at
both scales with the figures above. Evidence:
`Code/testing/mere/verify-90ce8f12d-20261015`.

Reproduction is now committed. `ports/graphshell/web/tests/README.md` gives
the whole sequence: `build-viewer.py` with wasm-bindgen 0.2.129 and Terser
5.51.2, the six routes, then the scale 1 and 1.25 comparisons. The fixture's
`?mode=compare` and `?mode=auto-choice` replace the uncommitted per-route HTML
copies, `run-presentation-webgl.py` takes `--mode` and `--device-scale-factor`
in place of `run-review-fractional.py`, and
`compare-presentation-captures.py` holds the RMS 1.0 / max 16 / alpha 0
contract. Re-run from those pieces alone on 2026-10-15 (evidence
`Code/testing/mere/repro-committed-20261015`): all six routes pass (33, 33, 33,
33, 27 and 3 controls), Auto with a Vulkan device loads the WebGPU artifact,
and the comparisons pass at RMS 0.252/0.300/0.299 at scale 1 and
0.258/0.332/0.328 at 1.25, max 11, alpha 0. Environment: hardware GL routes
need the desktop display (`DISPLAY=:0` with its Xauthority; launched from a
bare SSH tty, ANGLE cannot open a display), and Vulkan WebGPU needs X11, so
leave `WAYLAND_DISPLAY` unset. Compare WebGL2 and WebGPU only in
`?mode=compare`; the default mode's routes run different control sequences.
The copies under `Code/testing/mere/webgl-integration-20261014/split2` are
superseded.

## Open: resident-host port (O-PC)

The Oct 9–11 resident-host implementation lane never left the ThinkPad. It is
preserved in full, including its uncommitted Task 7 WIP, at tag
`archive/resident-current-main-20261010` (the WIP is its top commit,
`fd1640d0e`; Tasks 1–6 end at `2530e44d0`). Its plan,
`design_docs/archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md`,
exists only on that tag. Main carries the
[design](mere_docs/technical_architecture/2026-10-09_resident_host_design.md),
still marked proposed, although the tag's plan records Mark's approval.

This is a port, not a merge. C1 retired the browser modules the lane's
browser wiring targets and added its own residency layer
(`pandect/src/graph_session/residency.rs`,
`pandect/src/addressable_graph/residency.rs`, `djinn/src/resident_mere.rs`,
`graphshell/src/mere_host_resident.rs`). None of the lane's core types exist on
main: `LiveCommand`/`LiveEditError` (typed live edits, `pandect/src/live_graph.rs`),
`HostRefresh*` and `OwnedPersistenceStatus` (`graphshell/src/resident_host.rs`),
`PaneRoots`/`HostRoots`, `DirectoryHostCache`
(`resident_directory_owner.rs`).

The lane had a Knot companion, also never merged: an optional
`mere-snapshot-coverage-v1` construction feature in `knot-editor`'s endpoint
for Chirograph's snapshot coverage field. It is preserved in knot-editor as
`archive/resident-coverage-compat` and `archive/resident-coverage-current-main-20261010`
(two variants against different Mere families); port it alongside if the
coverage field survives the mapping.

Prompt for the O-PC session:

> Port the resident-host lane onto current Mere main. Read
> `design_docs/2026-10-14_mere_lane_handoff.md` (this section), then the lane's
> plan from tag `archive/resident-current-main-20261010`
> (`git show archive/resident-current-main-20261010:design_docs/archive_docs/2026-10-28/2026-10-09_resident_host_implementation_plan.md`)
> and the design on main. First map each of the plan's Tasks 1–6 against what
> C1 already provides on main (its residency modules listed above) and write
> that map into the plan, deciding per behaviour: already covered, port, or
> drop. Stop and show me that map before porting. Then port the missing
> behaviour onto main's current structure (the one-tree browser route, not the
> retired `web_*` presenter), task by task with focused tests, committing
> directly to main. Treat the Task 7 WIP (`fd1640d0e`: IndexedDB lifetime
> admission, `web_resident.rs`) as reference only; it was untested. Follow
> `design_docs/DOC_README.md` working principles; no PRs, no Co-Authored-By
> trailer.

## Archived as tags (branches deleted)

`archive/codex-tabard-boundary`, `archive/wing-platform-alignment-20260909`,
`archive/woodshed-retained-probe`,
`archive/codex-scripted-host-capabilities-20260906`,
`archive/codex-0903-preference-adapter` (the unmerged ESP preference-adapter
experiment), `archive/codex-knot-editor-extraction-20260901`,
`archive/a11y-mirror-prevent-scroll`, `archive/codex-burn-022-turnstone-base`,
`archive/codex-three-lanes-upstream-20261010`,
`archive/resident-current-main-20261010`, `archive/webgl-presentation` and
`archive/codex/site-viewer-startup`. Eleven more remote branches were
already merged and were deleted outright.

## ThinkPad state

`/home/markik/Code/repos/mere` is `main`, clean, with no other worktrees. The
`mere-receipt` worktree (a clean detached checkout already on main) was
removed. Genet and Turnstone were reconciled the same day; Turnstone's lane
moved to q-pc (`turnstone/design_docs/2026-10-14_qpc_lane_handoff.md`).
