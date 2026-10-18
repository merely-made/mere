# Ordinary graph bundle legibility — 2026-10-18

Qualified source: `487f370966630a5c95f64ce4246d79c3d12cd91b`, based on refreshed
Mere `dd9d5c523c092baf72be31cdbc1d28c4ae179fa0`. The owning scope and backlog
remain in the [link-model plan](../../../design_docs/mere_docs/implementation_strategy/2026-10-15_link_model_plan.md#bundle-legibility-and-one-geometry-handoff--2026-10-18).
The publication commit adds docs and evidence; it does not change compiled code.

| Claim | Evidence |
| --- | --- |
| Shared canvas checks | [Process](canvas-final.json), [actual log](canvas-final.log): 546 library and five integration tests passed; 18 library and one doctest ignores remain. Includes 25 responsive-path and three paint regressions. |
| Graphshell library | [Process](graphshell-final.json), [actual log](graphshell-final.log): 312 passed, six ignored. |
| Product browser build | [Wasm process](wasm-final.json), [log](wasm-final.log), [bindgen process](bindgen-final.json), [log](bindgen-final.log), [package/source hashes](build.json). |
| Product-disabled viewer and boundaries | [Viewer process](viewer-final.json), [log](viewer-final.log), [boundary process](boundaries-final.json), [log](boundaries-final.log): passed. |
| Normal graph browser run | [Seed result](browser-qualified/p4_tree_link_paths/result.json): eight captures, zero errors or gate failures. The route is the ordinary product canvas with a deterministic graph. |
| Fresh OS-process reopen | [Reopen result](browser-qualified/p4_tree_link_paths_reopen/result.json), [process observations](browser-processes.json): seed PID 87304 exited; reopen PID 3395 passed two captures. Both final exits were observed. |
| Image review | [All ten image observations and hashes](browser-review.json). Selected blue Resource cores, forward/reverse arrows and opposition ticks remain visible above the halo; selected gold Surface arrows remain visible through the field presets. |
| Exact source continuity | [Source-file hashes](source-continuity.json), [summary](receipt.json): 15,699 source-fact bytes and runtime graph revision 21 match across processes. This is not an immutable public revision claim. |
| Evidence verification | Reused [existing verifier](../link-model-ordinary-paths-20261017/verify.py); [actual process](verification-final.json), [log](verification-final.log): passed exits, Cargo summaries, artifact hashes, image sizes/dimensions, scenario gates and source equality. |

The implementation keeps one evaluated segment list for paint, hit testing,
rectangle selection and requested inspection reports. Paint/pick no longer
construct an unused provenance report or two intermediate point maps. Source
readings are filtered before cloning, and empty readings do not consume the
bounded evaluator cohort. Each centerline uses one joined polyline stroke.
These are source-verified reductions in duplicated work; no frame-time benchmark
or broader performance gain is claimed.

Responsive presentation rounds corners with two bounded convex passes, up to
128 samples, and clips against node-square bounds. Invalid attachment, overlap
or re-entry keeps resting geometry and increments an explicit fallback count.
Selected aliases share canonical identity; all halos precede stratum glyphs.
Arrows and opposition ticks use arc-length locations, independent of sampling
density. Canonical readings, exact directed assertion buckets, explicit
compatibility membership and endpoint force calculations keep their owners.

The scenario reuses the existing `p4_tree_link_paths.scn` and reopen flow. It now
also checks selected Surface identity, zero presentation fallbacks in the
qualified fixture, and the Resource forward/reverse/opposed source marks.
Field-only, compatible-only, combined, both-disabled, curved-path picking,
source inspection and saved hides pass. After Chrome exit/relaunch the hide is
retained, local response state resets, and Show hidden links restores all six
resting appearances. [Chrome/Metal capability facts](browser-qualified/probe/environment.json)
were collected in the same isolated loopback origin, `127.0.0.1:38118`.

The [original failed focused run](iterations/focused-1.json) and
[log](iterations/focused-1.log) are retained. It exposed incorrect test
assumptions: tight sample nodes can overlap, smoothing can remove a raw
control-polygon re-entry, and a strongly bent curve need not be monotone along
its original chord. Corrected checks use distinct endpoint faces, actual
presented-path re-entry, face-boundary/tangent properties and real unfiltered
picking. The [subsequent focused run](iterations/focused-2.json) passed before the
clean-source full suite. No assertion or test budget was disabled.

This is qualified for the ordinary 2D square-node view. Independent source
review found that world-space square attachment does not match tilted
billboards, rounded/circular silhouettes, scaled suggestion faces or lifted
nodes. Those profiles remain open in the owning backlog. Central bundle
crossings can still crowd marks; larger cohorts and zoom combinations need
separate qualification. Quiet unselected links retain existing grammar
emphasis. This is not a general accessibility, endpoint-dynamics, Woodshed
receiving, audio/physical or Knot storage qualification.

The full generated package and frozen web site remain at
`/Users/markik/Code/testing/mere/bundle-legibility-20261018`. The large Wasm bundle
is not copied into Git. Run the reused verifier against that complete directory:

```sh
python3 support/receipts/link-model-ordinary-paths-20261017/verify.py \
  /Users/markik/Code/testing/mere/bundle-legibility-20261018
```

The archived receipt alone lacks `pkg/`; verifying it without restoring the
manifest-matching artifacts must fail. The qualification server was stopped
intentionally with SIGINT (exit 130) after both successful scenarios; Chrome
was quit through its normal menu. Existing extension notices were dismissed
without accepting permissions.

The [documentation comparison](doc-audit/comparison.json) records zero new
link/path or judgment findings against the qualified source checkpoint. Global
audits still exit 1 with existing debt, including 19 judgment errors; this
receipt does not claim those audits are green. Raw full inventories remain at
`/Users/markik/Code/testing/mere/bundle-legibility-20261018/archived-doc-audit-inputs`;
the archive retains their hashes, process exits, counts and finding comparison
without duplicating the complete existing global inventories.
