# Mere patch provenance

Published `burn-cubecl 0.22.0-pre.4`, Burn commit
`9147c11a19a5e2d46694c54d441eee0e006e096c`, rebased 2026-09-27.

Mere carries the existing BrowserWebGpu shared-input binary guard in the
numeric, integer and float launchers. A matching view binds its allocation
once, aliases the second input and writes a distinct output. Other inputs
retain upstream's in-place selection. The numeric and float guards stay
inside pre.4's `in_memory_order` closures, after its zero-output checks;
the integer launcher keeps its existing structure.

Rulings 355 and 377 move identity comparison into the private `same_view`
helper and add full service identity. Its six predicates compare allocation
descriptor ID, both optional offsets, stream, raw underlying size and service.
The same-device/different-service-type unit control calls this actual helper;
forged test handles are never sent to a GPU. Slice controls preserve distinct
offsets and the prior None-versus-Some(0) distinction.

Manifest-only additions isolate this patch as a workspace, apply Mere's
runtime persistence patch when checking it standalone, and directly name the
already-transitive common/environment crates for constructing test handles.
Upstream source beyond the three guards and private helper is preserved.

Root/product manifests remain on pre.2 at this bounded checkpoint. These
helper controls are not the full headed numerical or two-peer acceptance
receipts, which remain migration plan section 13 gates. Remove the launcher
patch when a released unpatched Burn/CubeCL row passes the headed shared-input
reproducer and its applicable controls.

## 2026-09-27: verified alias-broadcast correction carried from pre.2

After the separate pre.2 fix `a016f86f`, ruling 380 carries the same alias-aware
broadcast helper and three guarded RHS calls into this pre.4 patch. The helper
combines input-zero alias binding with the output reference shape. It preserves
the six-field identity predicate, fresh output, zero-output returns and each
launcher's existing memory-order behavior. The fixture uses pre.4's non-generic
CubeTensor/CubeDevice API, the actual same_view predicate and allocation descriptor
IDs. Existing test-runtime dependencies suffice; no manifest change was needed.

Nine direct cases passed, the mapping-only fault failed exactly three broadcast
cases while six controls passed, and restoration passed all nine. The nine
identity tests and full Seiche gate (96 passing tests, one ignored) also passed.
Detailed binary/source and concurrency qualifications are in canonical plan
§13.22 and external `Code/testing/mere/receipts/2026-09-27/burn-pre4/pre4-carry-checkpoint.json`.
These native correctness receipts do not expand historical headed acceptance.

The earlier statement about root/product manifests remaining pre.2 describes
the initial S3/S4 checkpoint. Lane M subsequently moved its root graph to exact
pre.4 in S5–S8; this carry preserves those pins and the parked nested-lock work.
Broader migration acceptance and S13 remain open.

## 2026-09-29: selector retired under ruling 410

This vendored copy is retained for provenance and historical regression evidence.
The migration branch's root, probe, remote fixture and embedding fixture now
select pristine registry burn-cubecl 0.22.0-pre.4. The old pre.2 failure was
reproduced in the same browser where upstream pre.4 passed all 21 cases; nine
native unfused launcher controls passed identically on patched and upstream
rows, including broadcast aliases and retained-input/fresh-output checks.

The six-field same_view helper and its service rejection test describe this
retained implementation, not upstream. Ruling 410 conditionally supersedes the
requirement to carry that guard for the tested pre.4 migration. Other patches
remain independent. Plan §13.28 records four locked graph checks and the
remaining runtime/lifecycle/integration gates; main promotion is still pending.
