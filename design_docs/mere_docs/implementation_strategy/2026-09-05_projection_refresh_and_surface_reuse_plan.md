# Projection refresh and surface reuse

Status: bounded implementation and focused validation complete, 2026-09-06.
Authorized by Mark on 2026-09-05 following
the historical optimization review. Preserve concurrent workspace work.

## Targets and done-conditions

1. Dependency-aware refresh: unchanged declared inputs reuse layout; a relevant
   content change invalidates it, including Kanban URL grouping. Tests must
   distinguish relevant edits, unrelated edits, and footprint changes.
2. Independent appearances: the same document can occupy differently sized
   surfaces while sharing content; resizing or focusing one preserves the other.
   Exercise an actual host path, not only a proposed representation type.
3. Bounded richness: configurable resource policy limits expensive live work
   while preserving explicit activation and useful static appearances. Verify
   budget changes, retained state, and focus/visibility distinctions.
4. Separate motion, paint, and solving: reuse resolved placement when only
   representation content changes; preserve source/instance bindings and
   invalidate when layout inputs change. Exercise the Graphshell consumer and
   retain the existing zero-recompile/text-layout drag behavior.

## Ownership and verification

Terra owns the Canvas dependency cache; a second Terra owns document appearance
isolation; Luna owns live-resource policy. Root integrates Graphshell placement
reuse, checks cross-lane compatibility, and records focused test/browser evidence.
No global dependency scheduler, new graph authority, or blanket GPU migration is
implied. Full-scene solving remains valid for globally coupled arrangements.

## Findings

Canvas now tracks URL-authority grouping separately from graph structure, and
only the by-site Kanban strategy consumes that dependency. Resolved footprint
changes invalidate analytic placement through the existing geometry update path.

Graphshell validates refreshed projection data and compares the resulting layout
inputs before solving. Label-only changes reuse placement; changed coordinates,
footprints, arrangement parameters, or occurrence identity require a solve. This
still validates and constructs the input score, so it is not a general incremental
query engine. Both authoring and practice hosts use this path.

Reader is the first independent-appearance consumer in Turnstone. Shared source
content and per-appearance document-canvas state remain distinct. Other document
engines keep their established session behavior.

Pelt bounds external surface-producer polling with fair rotation. Deferred polls
retain the last composed image. This does not cap producer background work,
texture residency, or ordinary document-controller rendering. Hosts configure
`WorkspaceViewerConfig::with_surface_resource_policy(SurfaceResourcePolicy {
max_refreshes_per_frame, refresh_every_n_frames })`; the default is four polls
on each eligible frame. This is an API setting, not yet a persisted user control.
Cadence counts poll-capable frame passes; cached composition does not advance it.

## Validation

The [combined receipt](../../../ports/graphshell/docs/receipts/projection_refresh_surface_reuse_receipt.json)
records source hashes, exact commands/logs, and scope:

- Kernel and Canvas dependency tests passed, one focused test each.
- Compiler/editor source-path tests passed: 18 tests.
- Pelt routing and policy suites passed: four tests each; desktop check passed.
- Reader/Smolweb library suite passed: 20 tests, including shared rendered
  document identity and streamed-body replacement invalidation.
- Turnstone's pure surface modules passed 13 source-path harness tests, using a
  minimal numeric PaneId. Native host compilation passed separately with 122
  local Mere overrides and Turnstone's pinned Distillery. The local Distillery
  working copy has an unrelated missing scenomise dependency; it was preserved.
- Graphshell WASM built and passed 136 headed browser scenario steps: refresh,
  authoring, practice, and fresh-page reopen. Browser error logs were empty.
- A real practice-card drag added 368 physics ticks with zero new solves,
  compiles, or text layouts. Counters stopped after settling. This observation
  preceded the final Canvas parser consolidation; browser scenarios were rerun
  against the final build.
- Five-sample compiler medians at 10,000 occurrences were 33.235 ms for full
  compile and 27.573 ms for validated label refresh. Concurrent builds were
  active; this excludes GPU work and is not a headed FPS benchmark.

Native Reader has compile and state-test coverage, not a headed interaction
receipt. Persisted resource settings, background-work/residency budgets, and
independent appearances for other engines remain outside this bounded slice.
