# Shared query Resources and distinct link/edge styling

Status: merged native acceptance passed; final browser qualification in progress,
2026-10-14.

Saved definitions are `graph.query-view/v1` Mere Resources. Graphshell authors
them through the selected GraphSession journal and renders their actual
Resource and Surface identities in the Workbench. Resource links retain that
classification when lifted onto Surface appearances. Shown bindings remain
cross-stratum edges with zero hop cost. Saved stroke cues distinguish edges
from both link classes independently of theme colors.

The definition carries SELECT membership and its existing member variable,
appearance policy, tag context, and explicit member/appearance/relation budgets.
Sharing presents only that definition for receiver evaluation. Freezing creates
an existing immutable nested-selection Resource with exact selected identities,
relations and source revision. Neither operation replaces the other. Additional
strata remain a later design decision.

Native model acceptance covers two accesses to one Resource, distinct owners'
equal tag labels, undrawn tag concepts, exact parallel assertions, navigation,
stale edits, malformed definitions, empty and incomplete results, independent
freezing and budgets. Surface relation inspection uses exact stored buckets;
it never calls the global Resource-to-Surface Cartesian projection. This fixes
a budget bypass found by an independent source review. Snapshot restoration
rebuilds derived Domain links; authored assertion identities and shown bindings
are compared separately and exactly.

Merged native acceptance passes ten shared query tests, one actual owner-journal
save/freeze/undo/reopen test, and thirteen retained-content/export tests. The first
browser pass verifies save, independent freezing, configured strokes, exact empty
membership and actual IndexedDB reopening without reseeding. Its source/artifact
basis remains separate from the final browser pass.

Visual review found that Netrender's per-side border fallback renders a styled
top border solid. Authored cues now use explicit native stroke geometry, bounded
to 32 pieces per relation. Solid, dashed, dotted and double cues stay distinct
without relying on that fallback. A new model test verifies geometry, gaps and
the finite rendering budget; final browser pixels and source inspection are
recorded separately.
The block-stroke bundle in `browser-artifact-block-strokes.json` passes the
actual reopened inspection scenario. Its native PNG visibly contains dotted
shown bindings and solid links. `browser-final/query_views_inspect-span`
preserves the preceding span-based capture, whose pieces did not paint.
The accepted block children fix that failure. The card stacking and bounded
reader wheel repairs await their fresh consumer bundle; these captures do not
claim those later source changes.
Native Cargo gates use Rust 1.99.0, locked/offline suppliers, one compiler job,
and the stable Mere target and build directories. Declared MSRV remains 1.97.1;
this task does not claim a fresh MSRV qualification.

`standalone-lock-baseline.json` identifies the prior web lock by Git blob and
SHA256. The controlled lock delta preserves every existing Git supplier revision.
It adds the SELECT evaluator and brings the stale carrier lock row into step
with its published Murm b49df894 declaration. The standalone host restates the
parent workspace's Luggage override, retaining the same current Mere identity
contract. It also supplies the query evaluator's getrandom 0.2 browser entropy
feature independently of the persona stack's newer backend. Registry package
versions did not change in that initial delta. New upstream integration is
recorded separately in `combined-supplier-closure.json`: Genet is now 5618326d.
The standalone evaluator initially selected incompatible Digest major versions;
the directed repair selects sha1/sha2 0.11 for spareval while retaining older
hash consumers. `query-hash-resolution-repair.json` records that refusal and
repair. The query-owning linked-data library now enables its evaluator's browser
clock and rand 0.8 entropy backend on Wasm; native query behavior is unchanged.

The scoped host-hit supplier repair is published as Genet
`19947c3a3cb7a04fac0ed5b4ec656a3258b32dc7`, with all 42 Taproot tests passing.
It changes no supplier Cargo manifests relative to 5618326d. The directed
consumer lock update replaces only that Genet source identity, preserving the
canonical and two-package alias bridge. Fresh consumer gates remain pending.
Initial metadata attempts were interrupted while Cargo was materializing the
new 187,754-file Git checkout, before dependency resolution. They are not
failed compilation or resolver receipts.
