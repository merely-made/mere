# Batch 19 — subgraph rename (moved design doc)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/design/2026-06-13_subgraph_derivation_from_selection.md | current | n/a | 7 | 7 | 0 | 0 |
| **Totals** |  |  | **7** | **7** | **0** | **0** |

**Totals: 1 doc, 7 claims checked (7 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `92870eb4` (2026-09-12), working tree with the graphlet →
subgraph code rename applied but not yet committed. `archive_docs/` is
excluded.

This batch exists because the document moved. Its legacy record in
`snapshot_281_aggregate.json` (batch 12) is keyed by the old path
`mere_docs/design/2026-06-13_graphlet_derivation_from_selection.md` and stays
there as history; `legacy_corrections.json` can only correct counts, not
remap a path, so the new path takes a supplemental record here. The doc was
renamed on 2026-09-12 with `git mv` when the retired word *graphlet* left the
tree (TERMINOLOGY.md, retirement ruled 2026-09-05); its vocabulary was
updated to *subgraph* in the same pass and its content is otherwise the batch
12 text.

## mere_docs/design/2026-06-13_subgraph_derivation_from_selection.md

- disposition: current
- status line: "(none)" — accurate: n/a
- claims checked: 7 — holds: 7, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none

### Notes

- Re-verified against the renamed tree: the nine `SubgraphKind` shapes (Ego,
  Corridor, Component, Loop, Frontier, Facet, Session, Bridge,
  WorkbenchCorrespondence — `crates/forme/forme/src/subgraph.rs:78-88`);
  `ProjectionSource::{GraphDefault, GraphViewOverride, SelectionOverride}`
  (same file, `:102-106`); `EdgeProjectionSpec` (same file, `:118`);
  `SubgraphRef` (same file, `:18`); `SubgraphMemberDelta` (same file, `:130`);
  `SubgraphBinding::{UnlinkedSession, Linked, Branched}`;
  `assert_selected_relation` (`crates/canvas/canvas/src/selection.rs:149`).
- Inbound links repaired in the same pass: `DOC_README.md`,
  `mere_docs/design/2026-06-27_scope_model_reconciliation.md`,
  `mere_docs/design/2026-06-27_swatch_primitive_design.md`,
  `mere_docs/implementation_strategy/2026-06-13_scriptable_field_regions_plan.md`,
  `mere_docs/research/2026-06-13_edge_system_audit.md`,
  `mere_docs/research/2026-06-22_graph_projections_research.md`. The batch 12
  note's link inventory for sibling docs still holds under the new filename.
- The single cross-doc link the doc itself carries
  ([edge system audit](../research/2026-06-13_edge_system_audit.md)) resolves.
