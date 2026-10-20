> Copied 2026-10-19 from the M4-local `~/Code/testing/source-edit-publication-20261018/REPORT.md`; see the [receipts index](../README.md).

# Shared source-edit Git publication and receiving qualification

Date: 2026-10-18. Supplier was fetched, scoped-committed, rebased onto
Mere origin/main 908283728479cf3b9a666a68b9e360418b46301b and pushed first:
https://github.com/merely-made/mere/commit/70deef2c8cf3d3410ae35b496a9396514070b24f

The published supplier passes six focused tests, strict all-target Clippy,
and all six tests on Rust 1.97.1. Exact source/log hashes are in
core/qualification.json. The repository-wide documentation audit has exactly
the same baseline diagnostics as the preceding deduplication qualification;
core/doc-audit-comparison.json records that comparison.

Consumer receiving checks use the canonical Git main source and Cargo-generated
locks without local source patches. Per-consumer reports and receipts below
record exact source resolution, dependency movement, commands and workflows.
Previous local-override receipts remain unchanged in the sibling
shared-source-edit-20261018 and source-edit-dedup-20261018 directories.

Publication and receiving checks do not establish reviewed native GUI acceptance
or whole-product qualification. Host parser/schema, editor/undo, explicit save
and admission authority remain unchanged.

Published consumers:

- Isocosm: https://github.com/merely-made/isocosm/commit/276157b776c59394923550b0b42d543b317210d9
- Knot: https://github.com/merely-made/knot-editor/commit/d2d5c32942d40af47223790ed3e16bc88fa59aa6

Isocosm receives canonical main locked at the supplier commit. Knot receives the
same commit via an exact Git rev for only the new contract. This deliberate
exception avoids Cargo unifying its existing Mere/rendering dependencies at a
new revision. The rejected broader-resolution diagnostics remain in knot/;
no unrelated product API was adapted. Both consumers preserve all existing
package versions and supplier revisions. Knot's ignored standalone document
lock was also regenerated and qualified; its tracked root lock is committed.

Isocosm passes 75 feature/58 default tests, all-feature/all-target checking,
scoped Clippy/MSRV/build, and a 59-command/five-process workflow with 64 identical
normalized protocol records. Knot passes 53 default/66 engine/53 MSRV document
tests (one existing ignore each), 11 link-model cases, four desktop workflows,
two child reopens and the menu/focus case; strict library and scoped desktop
Clippy pass with baseline warnings. See per-consumer receipts for exact checks.
Parent changed only publication wording after receiving checks; runtime sources
and dependency locks are the qualified bytes.
