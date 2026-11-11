# The indexed dataset: the engine's query store (design brief)

**Date**: 2026-11-09
**Status**: design brief from a Mark session; D1 to D5 ruled the same day. Mark ruled "Let's measure, and do the indexed dataset after" (2026-11-09); §1 is the measurement, §2 to §5 the design as ruled, §6 the forks as they were asked, Decisions the answers. Implemented under the quad index plan: P1 and P2 landed 2026-11-10; the P2 table is in §1. The quadratic property append that the measurement also found is a separate session (Mark, 2026-11-09, "Separate session"), not this brief.
**Related**: the [engine split plan](../implementation_strategy/2026-11-07_engine_split_plan.md) (The engine's stages, Model and Query rows; Constraints, "Measure before the join moves"; P5, "Oxigraph's component crates stay upstream behind the engine's dataset and loader interfaces"), the [conformance ledger](../testing/2026-11-09_conformance_ledger.md) (SPARQL rows; the adapter layer "not scorable"), the [link model](2026-10-15_link_model.md) (what an assertion retains), the [statement kernel brief](2026-06-19_statement_kernel_brief.md) (the kernel as a statement store), the [linked-data platform lessons brief](../research/2026-11-06_linked_data_platform_lessons_brief.md) (the pack model that names predicates), doctrine Stack 1, Stack 3 and Stack 6.

## 1. The measurement

Instrument: `crates/graph/linked-data/examples/adapter_scale.rs` (package `mere-rdf`, release only, behind the `query` feature; landed on main 2026-11-09). It builds a chain graph of N nodes, each with one semantic statement to the next node and one literal property, which projects to 8 quads per node, and times four queries through today's adapter (`linked_data::query::sparql`) and through the same quads materialized into an `oxrdf::Dataset` and evaluated by the same `spareval`. Row counts agreed in every cell.

| nodes | quads | query | adapter | materialized `oxrdf::Dataset` | ratio |
|---|---|---|---|---|---|
| 1k | 8,000 | bound predicate, count | 26 ms | 0.9 ms | 30× |
| 1k | 8,000 | point lookup, one subject | 26 ms | 0.1 ms | 260× |
| 1k | 8,000 | two-hop join, count | 21.6 s | 2.8 ms | 7,700× |
| 1k | 8,000 | full scan, count | 44 ms | 3.4 ms | 13× |
| 10k | 80,000 | bound predicate, count | 228 ms | 5 ms | 45× |
| 10k | 80,000 | point lookup, one subject | 222 ms | 0.1 ms | 2,000× |
| 10k | 80,000 | two-hop join, count | not run (projected 40 min) | | |
| 10k | 80,000 | full scan, count | 434 ms | 40 ms | 11× |

Materializing the dataset cost 94 ms at 1k and 827 ms at 10k.

The cause is in `crates/graph/linked-data/src/query/dataset.rs`. `GraphDataset` implements `spareval::QueryableDataset` with `InternalTerm = Term` and an `internal_quads_for_pattern` that re-runs `dataset_quad_iter(graph)` over the whole graph for every pattern evaluation, cloning terms and deduplicating through a `HashSet`. A point lookup therefore costs the same as a predicate scan, and a join pays that scan once per binding: 1,000 bindings × 26 ms is the 21.6 s. The history entry point, `sparql_with_history`, already materializes an `oxrdf::Dataset` and so already runs at the right-hand column's speed; only the live path is slow.

The 100k scale was not reached because building the graph is itself quadratic. Per phase at 500 / 1k / 2k nodes: `add_node` 1 / 2 / 3 ms, `assert_semantic_statement` 3 / 8 / 14 ms, `append_node_properties` 150 / 920 / 3,236 ms, and 257 s at 10k. `literal_handle_available` (`crates/graph/graph-kernel/src/graph/resource_properties.rs`) walks every held literal and every semantic statement per append to check a statement id is unused, deserializing each resource's properties facet as it goes. That went to its own session and landed on Mere main on 2026-11-10 (`handle_index.rs`, a derived statement-handle index; `append_node_properties` 7 / 16 / 36 / 101 ms at 1k / 2k / 4k / 10k); the 100k row is below.

### 2026-11-10: the indexed dataset (quad index plan P2)

Same instrument, release, 1k and 10k only (100k waits on the kernel session). The borrowed adapter is deleted, so its column is gone. Columns: `sparql()`, which now builds an `IndexedDataset::for_graph` on every call; a held `IndexedDataset` built once and read with `sparql_on`; the same quads materialized into an `oxrdf::Dataset`, kept for parity. Row counts agreed in every cell across all three.

| nodes | quads | query | `sparql()` per call | held `IndexedDataset` | materialized `oxrdf::Dataset` |
|---|---|---|---|---|---|
| 1k | 8,000 | bound predicate, count | 35.8 ms | 0.26 ms | 3.8 ms |
| 1k | 8,000 | point lookup, one subject | 34.2 ms | 0.09 ms | 0.05 ms |
| 1k | 8,000 | two-hop join, count | 36.1 ms | 1.3 ms | 3.7 ms |
| 1k | 8,000 | full scan, count | 42.8 ms | 1.6 ms | 3.8 ms |
| 10k | 80,000 | bound predicate, count | 358 ms | 1.4 ms | 4.7 ms |
| 10k | 80,000 | point lookup, one subject | 359 ms | 0.09 ms | 0.05 ms |
| 10k | 80,000 | two-hop join, count | 370 ms | 9.6 ms | 30.6 ms |
| 10k | 80,000 | full scan, count | 411 ms | 7.6 ms | 38.2 ms |

Index build (`for_graph`: projection walk, interning, six orderings) 38 ms at 1k and 329 ms at 10k, against materialization's 85 ms and 923 ms in the same run. `memory_bytes()` (the quad index, not the dictionary's terms) 384,168 bytes at 1k and 3,840,168 at 10k. Against §4 at 10k: point lookup 0.09 ms (target under 1 ms), two-hop join 9.6 ms (under 50 ms), bound predicate 1.4 ms and full scan 7.6 ms (within 2× of 5 ms and 40 ms), build 329 ms (within 2× of 827 ms), index 3.8 MB (under 10 MB). All met. A `sparql()` call costs about one index build. A host that queries per frame holds the dataset and rebuilds on `is_current` (plan D3). Graph build in this run: 2.6 s at 1k and 266 s at 10k, nearly all `append_node_properties`, as above.


### 2026-11-10: 100k nodes, with the kernel's statement-id index landed

Same instrument and columns as above, release, one run of `adapter_scale 100000` (800,000 quads). Row counts agreed in every cell.

| nodes | quads | query | `sparql()` per call | held `IndexedDataset` | materialized `oxrdf::Dataset` |
|---|---|---|---|---|---|
| 100k | 800,000 | bound predicate, count | 2.92 s | 8.9 ms | 54 ms |
| 100k | 800,000 | point lookup, one subject | 3.09 s | 0.19 ms | 0.28 ms |
| 100k | 800,000 | two-hop join, count | 3.48 s | 61 ms | 205 ms |
| 100k | 800,000 | full scan, count | 2.93 s | 66 ms | 362 ms |

Index build 3.03 s; `memory_bytes()` 38,400,168 (48 bytes per quad, the six ordinal orderings plus the base row); materialization 8.64 s. The per-call `sparql()` column is the rebuild, about 3 s, which is why a consumer holds the dataset (plan D3) and why the rebuild-per-revision ruling (D4) says "then measure": at this scale a rebuild on every edit is not interactive, and the refresh numbers of P3 decide incremental maintenance.

Graph build 89.6 s: `add_node` 0.3 s, `append_node_properties` 1.03 s (linear now), `assert_semantic_statement` 88.2 s. The last is the next kernel finding: 7 / 20 / 62 / 376 ms at 1k / 2k / 4k / 10k, faster than linear, its cause unconfirmed when its session opened (Mark, 2026-11-10: "Open a session to fix it now").


### 2026-11-11: 100k again, with the kernel's pair lookup landed

`assert_semantic_statement` no longer scans every resource relation per call (Mere main `42f5a2f4f`; 6.8 / 13.1 / 35.0 / 69.3 ms at 1k / 2k / 4k / 10k, from 7 / 20 / 62 / 376 ms). One run of `adapter_scale 10000 100000`, release.

| nodes | graph build | `add_node` | `assert_semantic_statement` | `append_node_properties` |
|---|---|---|---|---|
| 10k | 0.14 s | 18 ms | 56 ms | 67 ms |
| 100k | 2.02 s | 0.34 s | 0.86 s | 0.82 s |

The 100k graph build was 89.6 s the day before and would have been hours on 2026-11-09. At 100k over the held index: point lookup 0.13 ms, bound predicate 6.7 ms, two-hop join 48 ms, full scan 57 ms; index build 1.93 s, 38.4 MB; materialization 6.47 s. The rebuild is now the same size as building the graph itself, so P3's refresh measurement starts from about 2 s per rebuild at this scale.

## 2. What the dataset is today

The dataset is a derived projection, not a store. `dataset_quad_iter` in `crates/graph/linked-data/src/lib.rs` walks surface nodes (direct quads: the node IRI as subject, properties as literals, scoped statements into named graphs by `GraphScope`, then RDF 1.2 reifier metadata per statement: asserter, assertion time, withdrawal), resource nodes (resource metadata) and resource edges (held assertions and their metadata). Terms are built as `oxrdf` strings on every walk. Named graphs are assertion scopes; the reserved history graph (`ASSERTION_HISTORY_GRAPH_IRI`) carries withdrawn reifiers only in the explicit history projection.

What the kernel already holds that an index can key on:

- **Durable identity.** A resource's id is a UUIDv5 of its canonical IRI (`chartulary::resource_id_from_canonical_iri`); the plan's Constraints call that "a durable binding, never a per-frame join".
- **Assertion records.** `chartulary::AssertionRecord { id, from, to, edge, asserter, span, basis }` with `AssertionSpan { first_asserted_ms, last_asserted_ms, withdrawn }`; `is_held` is `withdrawn.is_none()`. Scope lives on the statement (`GraphScope`).
- **Change counters.** `Graph` carries `revision`, `content_revision`, `semantic_observation_revision` and four `GraphInputKey`s advanced at the structural, content, site and visit mutation boundaries; a cached projection can be keyed on them without new plumbing.
- **A generic projection below Mere.** `chartulary::rdf` (the folded scholia crate) projects any chartulary graph's semantic ring to quads, JSON-LD and N-Quads, and names "named-graph scopes, RDF 1.2 reified statement metadata, vocabulary alignment, and SPARQL" as future work.

What the dataset must keep: the query tests in `crates/graph/linked-data/src/query/` (`tests.rs`, `coverage_tests.rs`, `resource_member_tests.rs`, `history_projection_tests.rs`), the coverage note on every result, resource-member and history semantics, and the ledger's SPARQL rows at the parser layer (1.1 evaluation 230/244, 1.2 evaluation 64/65). The ledger's adapter layer is "not scorable" today because `query::sparql` evaluates over a kernel `Graph` rather than a dataset the harness can load; the index is the chance to make it scorable.

## 3. The shape

**One dataset, held per revision, indexed, dictionary-encoded, fed to `spareval` through its own internal term.** The evaluator, optimizer, algebra and parsers stay upstream unchanged (P5); only the thing they evaluate over is ours.

- **Home (D1).** The generic part, a dictionary-plus-orderings quad index over ids, lives in Eidetic's `chartulary` with no Oxigraph dependency. Mere's engine crate wraps it: it supplies the terms from its facets and statements, owns the dictionary's external term type (`oxrdf`), and implements `spareval::QueryableDataset`. Eidetic indexes ids it is handed and never learns Mere's literal or reifier vocabulary.

- **Dictionary (D2).** Every term that appears in a quad is interned once: IRIs, blank or skolem nodes, literals with datatype and language. The dictionary maps a term to a dense `u32` minted when the index is built, and back. Nothing of the id is persisted; resource IRIs keep their durable UUID and statements their id, and the dense id is a projection of them with the revision's lifetime (Stack 6: identity is stable, representations are projections with their own lifetimes). Literals are interned by content.
- **Quads as id tuples, held and withdrawn (D5).** A quad is four ids, graph included (default graph is a reserved id), plus what a filter needs beside the triple: held or withdrawn, the asserter id, and the statement id for reifier lookups. The reifier metadata quads of today's projection are derived from that record rather than stored as extra tuples, so the live projection and the history projection are the same index read with and without the withdrawn rows; `dataset_quads_with_history` retires as a separate walk when the index lands.
- **Orderings (D3).** Six sorted tuple vectors: SPO, POS, OSP and their graph-first twins, which serve every pattern shape `spareval` asks for (`internal_quads_for_pattern` binds any subset of subject, predicate, object and graph). A pattern binds a prefix and reads one contiguous range; nothing scans. 7.7 MB at 80k quads.
- **The `spareval` seam.** `QueryableDataset` with `InternalTerm` = the dictionary id (a `u32` or `u64`: `Clone + Eq + Hash`, cheap to hash and compare, which is where today's `Term` clones cost). `internalize_term` is a dictionary lookup; a term the dictionary does not hold matches nothing. `externalize_term` is the reverse. The optional `externalize_expression_term` decodes a literal straight to the evaluator's typed `ExpressionTerm`, so a `FILTER` on a number or a date never rebuilds an `oxrdf` literal.
- **Refresh (D4).** Rebuilt per revision from the kernel's assertion records, keyed on the change counters above; the host holds the projection and reads it until the key moves. Incremental maintenance through the kernel's write paths is the second step, taken only if the rebuild cost shows at the scales the host sees (§4 says what to measure).
- **Writes do not go through the index.** SPARQL Update (ledger D65, "no update stage") would land through the kernel's assertion rules (separate assertions per asserter, withdrawal retained, no revival by a stale assert) and then refresh; the index is a read structure and never a second truth.

Nothing here forks a crate. The parsers, `spargebra`, `sparopt` and `spareval` are consumed as they are; `oxrdf` remains the external term type at the seam.

## 4. Done-conditions and budget

- The four queries of §1 run through the new dataset with the same row counts, at 1k, 10k and 100k nodes, and the table is appended to §1. Targets at 10k nodes: point lookup under 1 ms; two-hop join under 50 ms; bound predicate and full scan within 2× the materialized `oxrdf::Dataset` column.
- Index build at 10k nodes within 2× today's materialization (827 ms); memory for 80k quads under 10 MB across all orderings (six orderings of four 32-bit ids is 7.7 MB).
- Load, mutation, projection refresh and frame cost measured separately (plan Constraints), so the refresh fork in §6 is decided on numbers.
- The query tests, coverage note, resource-member and history behaviour pass unchanged; the ledger's SPARQL 1.1 and 1.2 evaluation rows gain an adapter-layer score run through this dataset.
- No new dependency; no fork of an Oxigraph component.

## 5. Where it bears on the doctrine

- **Stack 1.** "Try use, configure, compose and extend before implementing the gap." Use was tried: the borrowed adapter is the smallest thing that works and §1 is its cost. Compose is the shape chosen: the evaluator composed over our store. "A component is a module in its parent crate, not a crate of its own" and "a product's own crates are made when the abstraction earns one" are the first fork.
- **Stack 3.** "Seams stay clean even at some cost in speed" and "hot or wide stays native": a dataset read on every query is hot and wide, so it is native code behind the engine's dataset interface, not a pluggable component.
- **Stack 6.** Persist by durable identity, never a reusable slot: the dictionary id is never persisted; the UUID and the statement id are.
- **Engine split D1 and P5.** The query stage's store is the engine's; Oxigraph's components stay upstream behind it.

## 6. Forks

Asked as cards on 2026-11-09, in two rounds; the answers are the Decisions below, in the words given.

- **F1. Home.** (a) A module of the `mere` engine crate's query stage, where `linked-data/query` folds under P5. (b) Its own crate in Mere, taking a freed family name before a new word is spent (plan Constraints). (c) A generic index over chartulary's semantic ring in Eidetic's data root, beside `chartulary::rdf`, with Mere supplying the term mapping. Evidence: nothing outside the engine queries the dataset today; chartulary's own projection already lists SPARQL as future work; Stack 1 says a module until the abstraction earns a crate.
- **F2. Internal term.** (a) Dense ids minted per rebuild, smallest and simplest, no stability across revisions. (b) Durable ids (the resource UUID; a content hash for literals), stable across revisions, which incremental maintenance would need and which cost 128 bits per slot instead of 32.
- **F3. Orderings.** (a) Six (three triple orderings and their graph-first twins). (b) Three plus a per-graph partition. (c) Three, with a bound graph filtered on read. Evidence: `spareval` binds any subset of the four positions; the scopes are few and the triples many.
- **F4. Refresh.** (a) Rebuild per revision keyed on the kernel's change counters, then measure. (b) Incremental from the first cut, hooked where the kernel session's statement-id index hooks. Evidence: materialization today is 827 ms at 80k quads, which is the rebuild's ceiling to beat; the host's mutation rate is not yet measured.
- **F5. History.** (a) One index with held and withdrawn rows, live and history as reads of it, retiring `dataset_quads_with_history` as a separate walk. (b) Keep the separate history projection and index only the live view. Evidence: the history path is the one that is already fast, so (a) is about one structure rather than speed.

## Decisions

Ruled 2026-11-09, one session, each fork of §6 as a card.

- **D1. Store in Eidetic, seam in Mere (Mark, 2026-11-09):** F1. First round, against the module, own-crate and Eidetic options: "3 sounds right but i'm unsure. 1 is ok". Put back with the Eidetic option split, since `chartulary` names no Oxigraph crate (serde, petgraph, muniment, rkyv, slotmap, uuid; 7,853 lines) and only Mere knows its literal and reifier vocabulary: "Store in Eidetic, seam in Mere (Recommended)". `chartulary` gets a generic dictionary-plus-orderings quad index over ids with no Oxigraph dependency; Mere's engine crate wraps it, supplies the terms from its facets and statements, and implements `spareval::QueryableDataset`. Outcome: applies Stack 1 (compose before implementing; the abstraction that earned a home is the generic index, and it goes to the crate reserved for the generic graph) and Stack 3 (the Oxigraph seam stays in the engine).

  Links: area The stack · draws on mere:stack.1, mere:stack.3, engine-split D1 · plans engine-split P5 · summary The generic id-level quad index lives in chartulary; Mere's engine supplies the terms and the spareval seam.

- **D2. Dense ids per rebuild (Mark, 2026-11-09):** F2. "Dense ids per rebuild (Recommended)". The internal term is a `u32` minted when the index is built; nothing of it is persisted; the resource UUID and the statement id remain the durable identities. Outcome: applies Stack 6.

  Links: area The stack · draws on mere:stack.6 · plans engine-split P5 · summary The index's internal term is a dense id minted per rebuild; durable identity stays the UUID and the statement id.

- **D3. Six orderings (Mark, 2026-11-09):** F3. "Six: three plus graph-first twins (Recommended)". SPO, POS, OSP and their graph-first twins; every pattern shape reads one contiguous range; 7.7 MB at 80k quads.

  Links: area The stack · draws on indexed-dataset D2 · plans engine-split P5 · summary Six sorted orderings, three triple orderings and their graph-first twins, so every pattern reads one range.

- **D4. Rebuild per revision, then measure (Mark, 2026-11-09):** F4. "Rebuild per revision, then measure (Recommended)". The index is rebuilt from the assertion records when a kernel change counter moves and held until the next; incremental maintenance waits on the load, mutation and refresh numbers of §4. Outcome: applies the plan's Constraints, "Measure before the join moves".

  Links: area The stack · draws on engine-split D1 · plans engine-split P5 · summary The index is rebuilt per revision and held; incremental maintenance only if measured refresh cost demands it.

- **D5. One index, held and withdrawn rows (Mark, 2026-11-09):** F5. "One index, held and withdrawn rows (Recommended)". Live and history are reads of one index with and without withdrawn rows; `dataset_quads_with_history` retires as a separate walk when the index lands.

  Links: area The stack · draws on indexed-dataset D4 · plans engine-split P5 · summary Live and history are two reads of one index; the separate history walk retires.
