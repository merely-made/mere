# Graph query layer plan (RDF / SPARQL / Oxigraph)

*Written before the 2026-09-05 retirement of graphlet (TERMINOLOGY.md): read graphlet as subgraph. Identifiers such as GraphletId, GraphletRef, and SessionGraphlets are now SubgraphId, SubgraphRef, and SessionSubgraphs, and the graphlets crate is crates/graph/subgraph (code renamed 2026-09-12).*

**Corrected 2026-10-06 (S14 pass):** that crate path no longer exists. The
subgraph crate was folded into mere in `61894570` (2026-09-23);
`SessionSubgraphs` is in `crates/mere/src/subgraph.rs`.

**Status (2026-10-06):** slice 1 landed and survives as linked-data's `query`
feature, rewired from Oxigraph onto `spareval` over `oxrdf::Dataset` in
`8e7ae82b` (2026-07-06), with `dataset_quads` as the projection it queries.
Slice 2, the `>sparql` omnibar verb, landed in meerkat on 2026-06-18, retired
with it 2026-07-18 (`c5f01064`); only the mere facade's `query` feature
remains (`crates/mere/Cargo.toml`). Backlog #5 landed under the petgraph-RDF
plan (`70278fed`), and #2 in part (statement-metadata reifiers). Open: #1, the
rest of #2, #3 (carried by the graph semantics plan's P4), #4, and #6 to #8.
The layer is SPARQL query over the focused graph, kernel-sourced and one-way
(the kernel stays truth; this is a derived, read-only view for interop and
exploration). None of the residual backlog blocks.

Parent / cross-refs:

- [interaction_model_spine](../../mere_docs/technical_architecture/2026-06-18_interaction_model_spine.md)
  §6 (made-semantic): `to_jsonld` export is the kernel-authoritative *broadcast*;
  this plan is the *query* facet of the same stage.
- [unified_document_host_plan](2026-06-17_unified_document_host_plan.md) — its Phase 2
  "semantic surface" payoff (emit kernel-sourced JSON-LD into the orrery DOM) is the
  view-legibility sibling of this query work; both build on `node_quads`.
- The JSON-LD ingest/export bridge this builds on is the (archived, completed)
  `2026-05-22_linked_data_ingest_export_plan` in
  [`archive_docs/2026-06-09_completed_plans/`](../2026-06-09_completed_plans).

---

## What shipped

### node_quads — the single kernel→RDF projection (substrate)

`pub fn node_quads(graph, key, node) -> Vec<oxrdf::Quad>` in
[`crates/graph/linked-data/src/lib.rs`]. The expanded and compacted JSON-LD
shapers (`node_object` / `compact_node_object`) now render from it instead of each
walking the graph, retiring the duplicated walk. Verified by the existing
linked-data goldens (21/21). `oxrdf 0.3` was already a direct dep; no new
dependency.

**Corrected 2026-10-06 (S14 pass):** `node_quads` is no longer the single
projection. `dataset_quads` (`crates/graph/linked-data/src/lib.rs`, line 532)
is now the canonical projection for query; `node_quads` (line 523) remains as
the default-graph-only input to the JSON-LD shapers.

### Slice 1 — the query capability (library)

`linked-data/src/query.rs`, behind an optional **`query` feature**:
`pub fn sparql(graph, &str) -> Result<QueryRows, String>` projects the focused
graph into a fresh in-memory Oxigraph store via `node_quads` (converted across the
oxrdf-version gap by `to_ox_quad`), runs the query, returns the solution rows.
Ephemeral (store built per call, dropped after). `oxigraph = { version = "0.5",
default-features = false }` so `Store::new()` is in-memory with **no RocksDB** —
keeps the wasm/PWA target viable. Verified: `sparql_selects_a_literal_and_an_edge_over_node_quads`
(literal + edge SELECT over the seed graph), 22/22 lib tests.

**Corrected 2026-10-06 (S14 pass):** slice 1 no longer builds an Oxigraph
store. `8e7ae82b` (2026-07-06, petgraph-RDF Phase 3) rewired `sparql` onto
`spareval` evaluating directly over an `oxrdf::Dataset`; Oxigraph remains only
as a test-side parity oracle (`crates/graph/linked-data/Cargo.toml`,
`crates/graph/linked-data/src/query.rs`).

### Slice 2 — the `>sparql` omnibar verb (host)

The command shell ([`crates/meerkat/src/shell_eval.rs` *(historical citation)* <!-- doc-audit: historical-path -->]) gained a `sparql_query`
field on `ShellOutcome` and an arg-bearing `sparql("…")` binding (mirroring the
`relate("…")` precedent: record into the outcome, don't mutate inline, since the
shell snapshot has no RDF graph); `complete()` ghosts the verb. The host drain
([`crates/meerkat/src/command_drain.rs` *(historical citation)* <!-- doc-audit: historical-path -->] `submit_omnibar_command` → `run_sparql_query`
→ `format_sparql_rows`) runs it over the focused graph and echoes a one-line
result. meerkat enables `linked-data/query`. Verified: full meerkat suite green
(64 lib + 94 bin), `shell_eval::tests::sparql_records_the_query_for_the_host_to_run`.

Live form:

```
>sparql("SELECT ?s ?o WHERE { ?s <https://schema.org/name> ?o }")
```

**Corrected 2026-10-06 (S14 pass):** slice 2 went with meerkat. Its host files
(`shell_eval.rs`, `command_drain.rs`) were deleted in `c5f01064` (2026-07-18),
no `sparql` call exists in Mere's ports, and only the mere facade's `query`
feature remains (`crates/mere/Cargo.toml`, line 53).

---

## Residual backlog (ranked by leverage; none blocking)

1. **Shared `mapping` module (cleanup).** Export (`node_quads`) and ingest
   (`ingest.rs`, already on `oxrdf` quads) should meet at one set of decisions,
   retiring `ingest.rs`'s duplicate `RDF_TYPE` const and its two pre-existing
   unused-import warnings. Small, tidy, low-risk.
2. **RDF-star edge metadata (fidelity — the substantive next RDF step).** Today
   only `Semantic` edges + curated literals project; edge provenance / durability
   (the `predicate` + `edge_data` fields) are dropped. RDF-star (Oxigraph's
   `rdf-12` feature) is the standards-correct home: `node_quads` emits quoted
   triples for the metadata, so export and query become lossless for edges. The
   one genuinely new RDF capability worth doing.
   **Corrected 2026-10-06 (S14 pass):** partly done.
   `push_statement_metadata_quads` emits `rdf:reifies` reifiers carrying the
   statement's label, provenance and asserted-at
   (`crates/graph/linked-data/src/lib.rs`, lines 204 to 240), and Oxigraph and
   oxrdf carry `rdf-12`.
3. **`CONSTRUCT` / `DESCRIBE` → graphlet.** `query.rs` currently returns an error
   for `QueryResults::Graph`. Wiring `CONSTRUCT` output into a derived subgraph
   ties SPARQL to graphlet-derivation (reveal latent structure as a real graphlet).
   **Amended 2026-10-04 (Mark, [graph semantics plan](../../mere_docs/implementation_strategy/2026-10-04_graph_semantics_plan.md)
   ruling 3):** a SPARQL query becomes a `Linked` subgraph spec, shared by spec and
   frozen on demand into a nested graph; that plan's P4 carries this item.
4. **Results pane.** Slice 2 echoes one line on the status bar; a tabular
   `ListPane`-style results surface is the richer UX (variables as columns).
5. **Turtle / N-Quads I/O.** Cheap interop win via `oxttl` (same ox* family),
   widening import/export beyond JSON-LD. A lot of linked data in the wild is
   Turtle.
   **Corrected 2026-10-06 (S14 pass):** done under the petgraph-RDF plan:
   `crates/graph/linked-data/src/serialize.rs` has `to_nquads`, `to_trig`,
   `from_nquads` and `from_trig` (`70278fed`, 2026-07-06).
6. **Semantic-surface JSON-LD-in-view (Path A payoff).** Emit kernel-sourced
   `<script type="application/ld+json">` per card once orrery nodes are DOM. Gated
   on unified-document-host Phase 2 (orrery-as-element), not pure RDF work.
7. **Synced-mirror store (perf).** The ephemeral per-query rebuild is fine for
   occasional queries; a long-lived store kept in sync on mutation is the
   follow-on only if query frequency demands it. One-way (kernel → store) per the
   two-natured rule.
8. **Federation / UPDATE / HDT (future tiers).** SPARQL `UPDATE` is deliberately
   out (the kernel is the one-way authority). Federated `SERVICE` query across
   moots and HDT (compact binary graph distribution) are federation-tier items.

PROV-O (for the edge provenance in #2) and SKOS (for tag hierarchies) are the
vocabularies to reach for if/when those land; Atomic Data is a design reference,
not a task.

## Progress

- **2026-06-18** — node_quads projection + slices 1 (linked-data `query` feature)
  and 2 (`>sparql` omnibar verb) shipped and verified green; slice-2 files
  warning-clean. Not committed (working tree). The build detour was a stale local
  `Cargo.lock`: piecemeal `cargo update -p {genet-layout, netrender}` left an
  incoherent partial state (a `windows` 0.61/0.62 split); a full fresh resolve
  (`rm Cargo.lock`) to the current owned-fork mains (genet `69431717`, netrender
  `c5e6400c`) restored a coherent set. Lesson, matching the workspace convention:
  for gitignored, branch-tracked owned forks, re-resolve fresh rather than bump
  pins one at a time.

  **Corrected 2026-10-06 (S14 pass):** the slice 1 code is committed and
  present at mere 535bca11 (`node_quads` in
  `crates/graph/linked-data/src/lib.rs`, `sparql` in
  `crates/graph/linked-data/src/query.rs`).
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at
  mere 535bca11, from the D2 record in
  support/doc-audit/d2/batch_47_s14_phase_b9.md: "What shipped" corrected for
  `spareval` and `dataset_quads`, slice 2 recorded as landed in meerkat and
  retired with it (`c5f01064`), backlog #2 marked partly and #5 wholly done
  under the petgraph-RDF plan, the dead subgraph path corrected, and the
  "not committed" note corrected.
