# G4b2: bounded grouped-composition authoring

Implemented and qualified on 2026-10-21 from Mere
`67ed725ae908925b6381c583b9775594f16ee2f7`, on branch
`codex/g4b2-grouping-authoring-20261021`. The first packet ran at
`6b8eb1d6967204911035c7bb793bf4461ca1d102`. Upstream then added browser
shell/source-binding controls without changing these feature files; the
isolated checkout fast-forwarded and the host and headed gates ran again.
The initial packet remains under `before-upstream/` in local evidence.

The tree Inspector now has a Grouped composition editor: Grouping partition,
Outer law, Outer share, Inner law and Apply grouping. The existing grouped
runner uses the outer law's repulsion between groups and the inner law within
each group. The outer share is a force multiplier; it is not normalized.

The typed `TwoLawGrouping` action reads the canonical DynamicsSpec, replaces
only its root, carries its root overlays, and submits it through the existing
Canvas admission/binding path. Root and inner weights remain one. No schema,
new interpreter, runtime binding or supplier dependency was added. Numeric
share validation is shared with the two-law mix writer, including finite f32
range and nonzero underflow checks. Grammar admission owns negative weights
and incompatible law refusals.

The editor reads exact authored f64 shares, rather than reconstructing them
from runtime f32 weights. Readback accepts only the flat grouped shape the
controls represent: unit root and inner weights, plain preset children, and
an admitted Groups channel. Unsupported shapes carry an explicit note that
the editor values are not readback. Grouped roots show Custom in the profile
picker. Successful actions update the live record and dirty state; refusals
leave the live and saved record intact.

Three sources remain independent: the grouped root's partition, Group pull's
`channels.groups`, and the role target's `groups.channel`. Source-only physics
edits retain the authored grouping. Use single law explicitly replaces it.

## Qualification

| Gate | Result |
| --- | --- |
| Shared physics controls | 13 passed |
| Existing binding and grouped-runtime tests | 11 passed, 1 diagnostic ignored |
| Graphshell Web library, including actual v4 capture/restore | 327 passed, 6 ignored |
| Current product/WebGPU Wasm build | Passed |
| Headed grouped authoring/refusal/save/reopen/resave | Passed |
| Headed existing role-target and two-law mix regressions | Passed |
| Deliberately wrong partition after successful Apply grouping | Failed at its single expected mismatch |

The core tests preserve exact `0.10000000000000002`, root overlays, seed, bars,
realization, channels and both role maps. They check source edits, reopen and
explicit replacement, plus atomic refusal of Anneal in either law slot and
negative, nonfinite, overflowing and underflowing shares. The actual host test
captures, stores, restores and resaves the canonical v4 scene facet with exact
`0.1`, real group/item overrides, and independent source channels.

The headed fixture imports eight nodes with an existing Mix, Site group-role
overrides and a distinct Pinned item. It authors a Cluster grouping with Charge
at share `0.1` outside and Springs inside, changes Group pull to Component, and
keeps Role groups on Site. It verifies role precedence and exact saved dynamics,
refuses Anneal without changing the live/saved record, explicitly switches to
a single law, then reopens the saved grouping. Both the controls and actual
canonical record return to the saved values, and resaving preserves the exact
dynamics. The final save state is `saved`. Reopen adds the host's synthetic
workbench document, so the final image shows nine documents after importing
eight.

The negative scenario applies a real Cluster grouping, then deliberately
expects `groups.site`. Its only failure is
`grouping-partition Eq 'groups.site': got 'groups.cluster'`. It has no page
errors or receipt-gate failures and is retained as an expected control.

## Receipts and reproduction

`software-gates.json` records commands and settings; compressed logs retain
their outputs. `browser-package.json` pins the raw and generated Wasm package.
`source-manifest.json.gz` hashes the relevant Canvas, Seiche, host and scenario
sources and manifests. `headed-results.json.gz` contains final browser results.
The PNGs show the current grouped controls after reopen and after refusal.

Builds used Rust 1.99.1, `x86_64-apple-darwin` and
`wasm32-unknown-unknown`, four jobs, debug information off, incremental
compilation off, and `--locked --offline`. The final focused build was
repackaged with wasm-bindgen into a separate output directory. The shared
`web/pkg` was not replaced. The local qualification server uses
`Cache-Control: no-store`.

Serve the standalone Web package with
`ports/graphshell/web/scenario_qualification_server.py --package /path/to/package
--output /path/to/results --port PORT`, then open
`tree.html?app=local&scenario=scenarios/p4_tree_grouping.scn&sink=/scenario-receipt`.
For `p4_tree_grouping_wrong_partition.scn`, expect the disclosed mismatch.

Full local evidence, generated package, resolved diagnostic attempts and
primary-checkout preservation checks are under
`/Users/markik/Code/output/mere-g4b2-grouping-20261021`.

## Boundary

This qualifies bounded authoring through the existing native test path and
current headed browser/Wasm controls. Reopen is within the running application;
it is not a browser-restart durability test. Existing binding/grouped tests
exercise the runner, while the headed fixture keeps physics paused. There is
no new trajectory replay, cross-OS, native UI or downstream adoption claim.
Raw-term execution, checkpoint design, nested execution and render dependency
narrowing remain separate.
