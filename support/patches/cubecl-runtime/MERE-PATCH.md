# Mere patch provenance

Published `cubecl-runtime 0.11.0-pre.4`, CubeCL commit
`9d1314c070b998008959db021abcc23228ab1eb8`, rebased 2026-09-27.

Ruling 375 amends ruling 355: remove `persistence` from the manifest's default
features. This is the only functional difference from published pre.4.
The old allocation-identity helper is retired; no runtime Rust source changes.
The guard now uses the public allocation descriptor in burn-cubecl.

This removes cross-process autotune/throughput persistence and the applicable
native compiled-kernel store. Within-process caches remain. Windows results
do not establish the performance cost on Unix. Drop this patch when upstream
makes persistence optional without the forcing default dependency edge.

The root and product manifests still name pre.2 during the bounded patch
checkpoint. Integration and full consumer receipts remain in migration plan
section 13; this source rebase alone is not production acceptance.

2026-10-03: the consumer manifests now pin pre.4 exactly, and this patch is
selected by the root, graphshell-web, the probe, the remote fixture and both
repros. The paragraph above describes the 2026-09-27 checkpoint.
