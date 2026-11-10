# Batch 131 — indexed dataset brief

Written 2026-11-09 against Mere main `719686b3b` (the adapter_scale example commit), the day the document was added.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
| --- | --- | --- | --- | --- | --- | --- |
| indexed_dataset_brief | current | yes | 8 | 8 | 0 | 0 |

## mere_docs/technical_architecture/2026-11-09_indexed_dataset_brief.md

- disposition: current
- status line: "design brief from a Mark session; D1 to D5 ruled the same day ... Implemented under the quad index plan: P1 and P2 landed 2026-11-10 ..." — accurate: yes
- claims checked: 8 — holds: 8, stale: 0, unverifiable: 0

### Stale claims

- none

### Contradictions

- none

### Recommended action

- none

### Notes

Checked at `719686b3b`: `crates/graph/linked-data/examples/adapter_scale.rs` exists with
the `[[example]]` entry in `crates/graph/linked-data/Cargo.toml` (`required-features = ["query"]`);
`GraphDataset` and `internal_quads_for_pattern` in `crates/graph/linked-data/src/query/dataset.rs`;
`dataset_quad_iter`, `dataset_quads_with_history` and `ASSERTION_HISTORY_GRAPH_IRI` in
`crates/graph/linked-data/src/lib.rs`; `sparql_with_history` in `crates/graph/linked-data/src/query.rs`;
`literal_handle_available` in `crates/graph/graph-kernel/src/graph/resource_properties.rs`;
the four query test files under `crates/graph/linked-data/src/query/`; the ledger's SPARQL 1.1
evaluation row (parser 230/244) and 1.2 evaluation row (64/65) in
`design_docs/mere_docs/testing/2026-11-09_conformance_ledger.md`. The measurement numbers are
the document's own run of 2026-11-09 and are not re-run here.
