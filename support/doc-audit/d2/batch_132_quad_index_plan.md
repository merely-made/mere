# Batch 132 — quad index plan

Written 2026-11-09 against Mere main `75e26f56e` (the indexed dataset brief's rulings commit), the day the document was added.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
| --- | --- | --- | --- | --- | --- | --- |
| quad_index_plan | current | yes | 7 | 7 | 0 | 0 |

## mere_docs/implementation_strategy/2026-11-09_quad_index_plan.md

- disposition: current
- status line: "plan, 2026-11-09; D1 to D3 ruled the same day; nothing implemented." — accurate: yes
- claims checked: 7 — holds: 7, stale: 0, unverifiable: 0

### Stale claims

- none

### Contradictions

- none

### Recommended action

- none

### Notes

Checked at `75e26f56e`: the indexed dataset brief exists with D1 to D5; `crates/graph/linked-data/src/query/dataset.rs`
is 80 lines and `query.rs` 210; the `query` feature in `crates/graph/linked-data/Cargo.toml`; the conformance
harness file `crates/graph/linked-data/tests/w3c_conformance/sparql.rs`; Mere's `chartulary` row names
`eidetic.git` on `branch = "main"`; Eidetic's `crates/chartulary/src` is 7,278 lines over 15 files, largest
536 (`stemma/mutations.rs`), with no Oxigraph dependency in its `Cargo.toml`; Eidetic's G0 to G2 notes exist
under `eidetic/design_docs/eidetic_docs/technical_architecture/`.
