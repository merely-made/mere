# Batch 18 — recursive query experiments plan

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-09-11_recursive_query_experiments_plan.md | current | yes | 6 | 6 | 0 | 0 |
| **Totals** |  |  | **6** | **6** | **0** | **0** |

**Totals: 1 doc, 6 claims checked (6 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `55e21c81` (2026-09-11), working tree. `archive_docs/` is excluded.

## mere_docs/implementation_strategy/2026-09-11_recursive_query_experiments_plan.md

- disposition: current
- status line: "E1–E3 merged to main 2026-09-12 (...); Q1/Q2 scoped below, awaiting Mark's ruling; the graphlets → subgraph rename unparked" — accurate: yes
- claims checked: 6 — holds: 6, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none

### Notes

- Claims verified against the tree at write time: `collapse_descendants`
  rescans `relations()` per frontier node
  (`crates/canvas/canvas/src/fold_projection.rs`); `reconcile_delta` calls
  `derive_members` unconditionally (`crates/graph/graphlets/src/lib.rs`);
  `Graph::revision` exists with no delta journal
  (`crates/graph/graph-kernel/src/graph/mod.rs`, `apply.rs`); no criterion in
  the workspace; `ascent` 0.8.1 on crates.io; the 2026-06-18 petgraph_rdf_plan
  benchmark figures as cited.
