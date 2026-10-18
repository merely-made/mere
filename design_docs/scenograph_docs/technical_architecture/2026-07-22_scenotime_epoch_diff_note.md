# Scenotime Epoch and Diff Note

**Date:** 2026-07-22  
**Status:** landed and verified locally as the Scenograph half of Graphshell G2.

## Contract

`SceneSnapshot` wraps a dense `sceno::Scene` in an explicit `SceneEpoch` and
`Revision`. Its source, space, item, relation, and region tables use stable
slots. Removing a value leaves a serialized tombstone; additions append and
cannot reuse an index during the same epoch. Item order is a separate field,
so visual order can change without changing identity.

`SceneDiff` is one transactional revision transition. It can append, update,
or tombstone stable slots and change item layer/order, bounds, or generation.
The whole result is validated before it replaces the prior snapshot. Repeated
or older revisions are idempotent successful no-ops. A wrong epoch or missing
base remains explicit so a host can request replay or a full snapshot.

Presentation resources, session status, carriers, persistence, and source
authority remain outside Scenotime. Graphshell composes those beside the scene
diff rather than teaching Scenograph about remote applications.

## Receipts

- Seven Scenotime tests cover tombstones, idempotence, missing bases, epoch
  changes, transaction rollback, explicit item order/layer, serde round trips,
  and a deterministic 96-revision randomized oracle.
- `cargo test --workspace` passes across 13 Sceno, 4 Scenomise, and 7
  Scenotime tests.
- `cargo check --workspace --target wasm32-unknown-unknown` passes.
- `cargo clippy --workspace --all-targets -- -D warnings` passes.

## Remaining runtime work

Scenotime does not yet schedule incremental signal evaluation, maintain
derived caches, or resolve hits into authorized intents. Those belong to
later consumer proofs. G2 establishes only the portable identity and history
contract needed for safe remote replay.

## Source audit checkpoint, 2026-10-16

At Mere `b567660d67c4d900f1d638345c89715e1f36c575`, all 13 Scenotime source,
manifest and test files have been read at recorded hashes. The current native
package gate passed 43 library tests and 11 integration tests: six fold,
three history and two site-projection tests. Six standalone public-API
diagnostics also passed while establishing one defect and two qualified
boundary observations. These are not browser, renderer, wasm or user-archive
receipts. No production code changed in this pass. The detailed record is
`support/receipts/authority-success-audit/scenotime_review_receipt.json`.

**Q069 / A068: structural validity does not establish wire round-trip
validity.** `from_dense`, `validate` and trace construction accept nonfinite
item positions/scales, footprint dimensions and scene bounds. JSON
serialization succeeds by writing `null` for these numeric values; reading
the emitted snapshot or trace then fails. A `SetBounds` diff carrying `NaN`
also commits its revision and builds an accepted trace that cannot reopen
from its own serialized bytes. A finite trace control reopens identically.

This is a public typed-data admission/serialization mismatch. It does not
demonstrate a product solver emitting invalid geometry, a user archive being
lost, or a JSON parser accepting nonstandard `NaN` syntax. Done: shared
finite-value admission before snapshot/diff commit and trace/capture
publication, with serialization/reopening controls. Zero and degenerate
footprints, negative scales and negative dimensions need their actual
semantics preserved or explicitly decided; silently changing source
coordinates is not a repair.

**A069: live replay and stored traces intentionally differ.** Live replay
tolerates repeated/older revisions as no-ops. `SceneTrace` explicitly checks
epoch, base and forward progress before reusing that same diff application.
It is an immutable replay record, while the history integration puts complete
trace values in Backstep. This is a documented ownership split, not another
hidden undo stack. The site parity test uses the same `apply_diff` primitive
on both paths, so it checks composition/fixture parity rather than supplying
an independent proof of the diff algorithm.

**A070: coordinate traversal is still distributed.** Dense `Scene::to_world`,
sparse `SceneTables::space_to_world` and the host receipt's `world_origin`
walk parent chains separately. Shared transform and footprint math is real
reuse; no valid-input divergence was established here. Picking checks
visibility, fold hiding, explicit order/layer, hit overrides and transforms.
Its stated linear cost omits ancestor-chain depth: it resolves a chain per
candidate, while snapshot validation separately walks every space chain.
This is a source complexity qualification, not a performance measurement.

**A071: clock policy remains a host boundary.** Monotonic repeated timestamps
add no elapsed time. The diagnostic sequence `[100, 110, 105, 110]` produces
elapsed `15`: a backward sample resets the delta baseline, so already-seen
host time is charged again. Ordinary monotonic browser timestamps were not
exercised. Ignore-out-of-order versus reset-clock behavior needs a consumer
decision before changing this policy. ReturnMotion remains a local visual
offset and does not mutate source positions.

Remaining candidates are preserved in the ledger: transitions consider only
direct item operations; held-placement metadata has no diff update operation;
honored positions are compared in item-local coordinates; extreme finite
return-motion tuning has no stability bound; and deserialized schedules bypass
the spec constructor's validation. A reversed deserialized `LinkKey` also
passes snapshot validation and produces a different view token from its
canonical constructor, but no importing/grouping consumer failure was run.
The next source review is Scenograph's authoring and binding layer.
