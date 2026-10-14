# Shared query Resources and distinct link/edge styling

Status: implementation qualification in progress, 2026-10-13.

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

Owner persistence, browser UI, configured stroke captures, exact empty results
and reopening remain pending in this receipt until their final logs are added.
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
versions do not change. New upstream integration is recorded separately.
