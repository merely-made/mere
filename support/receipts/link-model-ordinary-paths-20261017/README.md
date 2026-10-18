# Ordinary graph responsive link paths — 2026-10-17

This receipt qualifies responsive geometry in the normal Graphshell product
canvas on source `aa0392a1ba547a55860a6988db48e2d6ba22ee48`, integrated with
published Mere `a2b4222bf`. The canonical backlog, scope, findings and follow-ups
are in the [link-model plan](../../../design_docs/mere_docs/implementation_strategy/2026-10-15_link_model_plan.md#current-backlog-and-ordinary-graph-continuation--2026-10-17).

| Claim | Evidence |
| --- | --- |
| Canvas source checks | [Pictograph process](canvas-final-v3.json), [actual log](canvas-final-v3.log): 534 library and five integration tests passed; 18 library ignores and one doctest ignore remain. All 17 new responsive-path regressions ran in the library suite. |
| Host source checks | [Graphshell process](graphshell-final.json), [actual log](graphshell-final.log): 312 passed, six existing ignores. |
| Product browser build | [Build](build-final.json), [bindgen](bindgen-final.json), [source/package hashes](build.json): locked/offline Wasm product build and wasm-bindgen 0.2.129 completed with actual zero process exits. |
| Product-disabled feature boundary | [Viewer compile](viewer-check-final.json), [log](viewer-check-final.log): the product-disabled WebGPU viewer compiles. This is compile evidence, not its own runtime acceptance. |
| Dependency authority | [Port-boundary check](port-boundary-final.json), [log](port-boundary-final.log): passed. The browser uses the Genet/Netrender suppliers already published by the integration baseline; it does not add a product-host dependency to shared graph capability. |
| Ordinary graph UI | [Seed receipt](browser-qualified/p4_tree_link_paths/result.json): all scenario checks passed, eight captures, zero page errors/gate failures. |
| Actual process reopen | [Chrome before seed](chrome-before-seed.json), [verified exit](chrome-quit.json), [new process](chrome-reopened.json), [reopen receipt](browser-qualified/p4_tree_link_paths_reopen/result.json): main PID changed from 30916 to 38280 with an observed absence between them. Two captures and all reopen checks passed. |
| Image review | [Review](browser-review.json): the integration owner and independent reviewer viewed all ten PNGs. Exact source/pick identity is established by the assertions and source readings, not inferred from images. |
| Evidence verification | [Verifier](verify.py), [actual verification process](verification-final-v2.json), [log](verification-final-v2.log), [summary](verification.json): process exits, suite summaries against logs, artifact hashes, image dimensions/encoded lengths, scenario gates and source equality passed. |

The headed route was `tree.html?app=local&fixture=link-rest` in native Chrome
155 on Apple Metal/WebGPU. It renders a deterministic graph through the ordinary
TreePage, canvas, control and inspector paths. It is not the separate
`responsive-links.html` experiment. The graph has four Surface nodes, two held
Resources and six displayed link appearances. The browser exercised labeled
view-local Attractor, Flow and Vortex presets; stored Global Vector field
selection and refusal cases are covered by CPU tests.

Field response and compatible attraction are independently opt-in. The group
contains explicit canonical LinkKeys; no predicate implies compatibility.
Ungrouped paths remain exactly at rest in compatibility-only mode. Both-off
restores the exact baseline geometry. Curved Resource alias picking uses a real
point on the emitted path and ordinary pointer input, followed by exact
selected-pair and source-backed inspector assertions. Whole-link hides remove
every appearance of the canonical key without changing source facts.

After browser exit, the saved hide survives, both response flags reset, the
temporary field and explicit group clear, and Show hidden links restores the six
resting appearances. Seed/reopen retain the same 15,699-byte source facts and
runtime graph revision 21. The live source still explicitly lacks a qualified
immutable public revision; a runtime counter is not promoted into one.

The pure evaluator remains bounded to 64 responsive appearances, eight fields,
32 points, 48 iterations and two million work units, with AST depth 16/nodes 256.
The normal controls use 17 points/24 iterations. Invalid identity, field kind,
activity, extent, cyclic/deep dependencies or work bounds refuse while retaining
complete resting geometry. Self-link arches and offscreen endpoint paths stay
at rest. Response settings and geometry are view-local; node physics authority
and source assertion storage are unchanged.

Visual follow-up: strong selected compatibility bundles are angular and overlap;
arrows, opposition marks and parallel strata are hard to distinguish at this
zoom. Later captures also scroll the long tool pane. This is recorded in the
canonical backlog. It is not a full grammar/accessibility audit, a node-force
change, a performance qualification of larger graphs, a cross-product intake,
a browser anti-eviction guarantee, physical audio acceptance or a Knot
checkpoint confidentiality fix. Knot storage work remains in its own lane.

`iterations/` retains earlier process records/logs and the first browser result.
The first browser run had three harness failures (formatted zero values and a
Pause action on an already-paused fixture). Other retained failures exposed test
fixture/discovery errors. A full canvas timing check also failed while packaging
and browser work were active; the final isolated full suite passed without a
budget change. None of those earlier attempts are counted as qualification.
The verifier initially assumed the capture metadata counted decoded PNG bytes;
`loader.js` records the PNG data URL length, which the corrected verifier checks
exactly. Failed-run captures and all generated bundles remain in the local
qualification directory.

The archived build manifest hashes the generated package and frozen web assets;
the large generated Wasm/package/site are retained outside Git at:
`/Users/markik/Code/testing/mere/ordinary-responsive-links-20261017`.
Run the full fail-closed verifier there:

```sh
python3 verify.py /Users/markik/Code/testing/mere/ordinary-responsive-links-20261017
```

The published archive contains the process logs, structured results, reviewed
captures and hashes. Running the full verifier on a clone requires restoring
the generated package/site files at the manifest paths; their absence is a
verification failure, not an implicit pass.

The [documentation comparison](doc-audit/comparison.json) records zero new link/path or judgment findings against the recorded in-progress checkpoint. Global audits remain failing: the same 19 judgment errors and existing link/path/header debt remain; this receipt does not report a global green audit.
