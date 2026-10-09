# Batch 58: the facet dissolution brief (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| mere_docs/research/2026-10-08_facet_dissolution_brief.md | current | yes | 9 | 9 | 0 | 0 |
| **Totals** |  |  | **9** | **9** | **0** | **0** |

**Totals: 1 doc, 9 claims checked (9 holds, 0 stale, 0 unverifiable), 0 contradictions; 0 status lines wrong.**

Audit base: Mere `f826bbd2` (2026-10-08), the commit that added the brief, with
its correction in the same batch. No cargo command was run. The Scenograph
editor lane that wrote the brief judged it the same day; an independent
re-judgment can supersede this record under ruling S34.

## mere_docs/research/2026-10-08_facet_dissolution_brief.md

- disposition: current
- status line: "**Status:** open: waiting for graph semantics' owner, to whom Mark hands it." — accurate: yes
- claims checked: 9 — holds: 9, stale: 0, unverifiable: 0

### Stale claims

None.

### Contradictions

None.

### Recommended action

Re-judge when graph semantics' owner answers its five questions, and supersede
this record.

### Notes

Each claim below was checked against the source in brackets, and holds:
- Mark's words for SE71 and SE75 [the Scenograph editor plan];
- the store is `chartulary::FacetStore<Uuid>` behind `node_facets.rs`, "JSON-shaped and unknown-forward at its boundary" [`crates/graph/graph-kernel/src/graph/node_facets.rs`];
- facets are set and removed through the delta spine [`apply.rs:261`, `:267`];
- pandect saves them as `facets.json` [`crates/system/pandect/src/facet_store.rs:54`];
- 3,166 occurrences in 141 Rust files [`git grep -i -o facet` over `*.rs` at origin/main, 2026-10-08];
- each declared key in the table exists as a constant or literal [`node_facets.rs`; the `*_FACET` constants; `Cap::facet("denizen.binding")` in `crates/servitor/src/cap.rs`; `personal_sync.rs:2031`];
- `facet_projection.rs` is the "PMEST facet projection" [its module doc];
- Gazette's contact facets use the source adapter `gazette.contact-facet/v1` [`ports/gazette/src/ledger.rs:19`];
- graph semantics rulings 18 and 19 concern the two node senses [the graph semantics plan's commit `62219dd1`].
