# Batch 96: Hest H1 local implementation and qualification

Audit date: 2026-10-20. Scoped continuation from Mere `9a173f18b` against the
local H1 source and sealed receipts. Batch 95 remains the historical judgment
of the original plan. No published release or native acceptance is inferred.

## cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md

- disposition: current
- status line: "H1 implemented and qualified locally; publication and push are explicitly deferred" - accurate: yes
- claims checked: 15 - holds: 15, stale: 0, unverifiable: 0

### Stale claims

- none

### Contradictions

- none

### Recommended action

Commit H1 locally. Retain the explicit publication gate and H2 0.2.0 boundary.

### Notes

The 15 checked claims are: Hest's version; serde-only normal dependency;
shim version; shim's 0.1.x requirement; all old public re-exports; unchanged
catalogue; exact moved implementation after module/doc path substitutions;
five active consumers repointed; `ActionId` retained in the kernel; no H2
fields or invocation types introduced; accepted H2 0.2.0 boundary; published
Cambium 0.4.0 compatibility; Graphshell's five actual host-model tests;
Turnstone's twelve actual palette tests with unchanged consumer sources and
unrelated lock entries; and the sealed 52-test total with strict Rustdoc pass.

Evidence: the two package manifests and shim source, the moved source hashes,
and `support/receipts/hest-h1-20261020/qualification.json`. Turnstone's manifest
and lock changes are confined to its isolated receiving fixture. The zero-test
Pandect discovery is excluded from the total. Existing repository-wide D2 and
mechanical findings outside this scoped plan remain outside this judgment.
