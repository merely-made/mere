# Mere lane handoff and branch reconciliation, 2026-10-14

**Status, 2026-10-14:** Remote `main` is the only live line. Every branch and
worktree on the ThinkPad was landed, archived as a tag, or removed. The WebGL
presentation lane landed later the same day (below), with two acceptance limits
left for Mark. The resident-host port on O-PC remains open.

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

**Mark's decision, still open:** both acceptance limits remain unmet and
unchanged. Wasm gzip6 is 3,661,337 bytes against the 3,407,872 cap (the lane
measured 3,674,899). At 125% display scale, WebGL2 versus WebGPU RGB RMS is
1.26451 and 1.19021 against 1.0 (alpha matches; resized and normal-scale
comparisons pass). Human assistive-technology and asynchronous browser WebGPU
device replacement remain separate gates.

## Open: resident-host port (O-PC)

The Oct 9–11 resident-host implementation lane never left the ThinkPad. It is
preserved in full, including its uncommitted Task 7 WIP, at tag
`archive/resident-current-main-20261010` (the WIP is its top commit,
`fd1640d0e`; Tasks 1–6 end at `2530e44d0`). Its plan,
`design_docs/mere_docs/implementation_strategy/2026-10-09_resident_host_implementation_plan.md`,
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

Prompt for the O-PC session:

> Port the resident-host lane onto current Mere main. Read
> `design_docs/2026-10-14_mere_lane_handoff.md` (this section), then the lane's
> plan from tag `archive/resident-current-main-20261010`
> (`git show archive/resident-current-main-20261010:design_docs/mere_docs/implementation_strategy/2026-10-09_resident_host_implementation_plan.md`)
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
