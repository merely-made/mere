# The quad index: implementation plan

**Date**: 2026-11-09
**Status**: plan, 2026-11-09; D1 to D3 ruled the same day, D4 to D7 (the P1 lane's forks) on 2026-11-10; P1 landed on `eidetic` main `60f3d60` the same day (Mark: "Land and start P2"); D8 ruled and P2 landed on Mere main 2026-11-10 (Mark: "Land after the lock repair"); P3 open, its refresh and 100k rows measured 2026-11-12; P4 (incremental maintenance) opened 2026-11-12 by D9, after the engine split's P5 fold. Carries out the [indexed dataset brief](../technical_architecture/2026-11-09_indexed_dataset_brief.md) (D1 to D5, 2026-11-09) under the [engine split plan](2026-11-07_engine_split_plan.md)'s P5. Three phases, in order: the generic index in Eidetic, the engine seam in Mere, then measurement and refinement. Mark: "ok, orchestrate/do the implementation plan for the index" (2026-11-09).
**Related**: the brief (the measurement in its §1, the shape in §3, the budget in §4); the [conformance ledger](../testing/2026-11-09_conformance_ledger.md) (SPARQL rows); Eidetic's chartulary notes (`eidetic/design_docs/eidetic_docs/technical_architecture/2026-07-08_chartulary_g0.md` to `g2`); doctrine Stack 1, Stack 3, Stack 6, Stack 7.

## What the brief ruled, carried here

- **D1.** The generic part, a dictionary and quad index over ids, lives in Eidetic's `chartulary` with no Oxigraph dependency; Mere's engine supplies the terms and the `spareval` seam.
- **D2.** The internal term is a dense `u32` minted per rebuild; the resource UUID and the statement id stay the durable identities; nothing of the dense id is persisted.
- **D3.** Six sorted orderings: SPO, POS, OSP and their graph-first twins.
- **D4.** Rebuilt per revision and held; incremental maintenance only when measured refresh cost demands it.
- **D5.** One index with held and withdrawn rows; live and history are two reads of it; `dataset_quads_with_history` retires as a separate walk.

## Constraints

- **Source discipline.** Written from the brief and this plan; no code copied from Oxigraph or any other store, no downloads, nothing from `Code/crates/prns`. Oxigraph's components are consumed as dependencies in Mere only and unchanged.
- **Family rows and locks.** Mere consumes `chartulary` from `eidetic.git` on `branch = "main"`; after P1 lands, Mere's lock moves by `cargo update -p chartulary` and nothing else is pinned or hand-edited.
- **Ceilings and hooks.** 600 physical lines per file in both repositories (Stack 7); Eidetic's hooks are line-ceiling and `cargo clippy --locked --all-targets`; Mere's add the responsa check and `cargo deny --locked check bans`. Never `--no-verify`. Plain commit messages, no attribution lines.
- **Lanes stop at forks.** A choice with more than one defensible answer comes back as a card with evidence before code commits to it. A lane verifies in its own worktree, one cargo build at a time, at most three tasks.
- **Measured, not asserted.** The instrument is `crates/graph/linked-data/examples/adapter_scale.rs`; its table is appended to the brief's §1, not restated elsewhere.

## P1. `chartulary::quads` in Eidetic

A new module of `crates/chartulary`, generic over the term type, with no new dependency. Split by responsibility under the ceiling: `quads/mod.rs` (the public surface and docs), `quads/dictionary.rs`, `quads/index.rs`, `quads/tests.rs`.

- `Dictionary<T: Hash + Eq + Clone>`: `intern(T) -> TermId`, `lookup(&T) -> Option<TermId>`, `resolve(TermId) -> &T`, `len()`. Append-only; a rebuild makes a new one (D2, D4). `TermId` is a `u32` newtype; one value is reserved for the default graph.
- `Quad { subject, predicate, object, graph: TermId }` plus `QuadMeta { held: bool, asserter: Option<TermId>, statement: Option<TermId> }`, the fields a filter needs beside the triple (brief §3).
- `QuadIndex::build(impl IntoIterator<Item = (Quad, QuadMeta)>) -> QuadIndex`: one base table of rows, every distinct (quad, meta) kept (D6), and the six orderings of D3 as sorted ordinals into it (D4). `TermId` is a `NonZeroU32` with `DEFAULT_GRAPH = u32::MAX` (D5). `pattern(subject: Option<TermId>, predicate: Option<TermId>, object: Option<TermId>, graph: Option<Option<TermId>>, view: View) -> impl Iterator<Item = (&Quad, &QuadMeta)>` picks the ordering whose prefix the bound positions fill and reads one contiguous range; `View::Live` skips withdrawn rows, `View::History` keeps them (brief D5); a quad carried by several rows is yielded once per view, a held row first (D6). `graph_names(View)` and `contains_graph(TermId, View)` read the graph-first orderings and omit a graph whose rows are all outside the view (D7). `len()`, `memory_bytes()` for the measurement.
- Tests: every one of the sixteen bound shapes, both views, against a brute-force filter over a generated dataset with duplicates and several graphs; the default graph as `Some(None)`; `graph_names`; and one test that indexes `chartulary::rdf::to_quads` of a sample graph, tying the module to the crate's own projection.
- A short note `eidetic/design_docs/eidetic_docs/technical_architecture/2026-11-09_chartulary_g3_quads.md` in the G0 to G2 form, citing the Mere brief and this plan by path, with its `DOC_README.md` line.
- Done when: the tests pass; `cargo clippy --locked -p chartulary --all-targets` is clean on native and `cargo check -p chartulary --target wasm32-unknown-unknown` passes; one commit on `eidetic` main through the hooks, pushed; Mere's lock then moves by `cargo update -p chartulary`.
- Needs: nothing. Eidetic's `crates/chartulary` is 7,278 lines over 15 files today, none above 536.
- **Landed 2026-11-10**, `eidetic` main `60f3d60`: `quads/mod.rs` 156, `dictionary.rs` 105, `index.rs` 263, `tests.rs` 405 lines; 122 chartulary tests pass, 10 new; `memory_bytes()` 3,840,200 at 80k generated quads; wasm check passes; G3 note `eidetic/design_docs/eidetic_docs/technical_architecture/2026-11-09_chartulary_g3_quads.md`. The lane's own calls, recorded there: `len()` counts stored rows, the default graph is never a named graph, and `TermId::from_u32`, `TermId::as_u32` and `Dictionary::get` exist for P2's unknown-term case. Mere's lock reached it in `a9153fa6d` ("Move to p2mere 0.10", 2026-11-10), which also took Murm's and Mere's own move from p2mere 0.9 to 0.10: moving the Eidetic packages alone had left two copies of `p2mere-core` and 29 errors in `pandect` and `murm-sync`, since Eidetic's `aa71322` had moved stickleback first. P2 started the same day.

## P2. The engine seam in Mere

In `crates/graph/linked-data/src/query/`, behind the existing `query` feature. `dataset.rs` (the borrowed adapter, 80 lines) is replaced; `query.rs` (210 lines) keeps its entry points.

- `IndexedDataset` (`query/indexed.rs`): a `Dictionary<oxrdf::Term>` and a `chartulary::quads::QuadIndex`. `from_quads(impl IntoIterator<Item = (oxrdf::Quad, QuadMeta)>)` for any dataset; `for_graph(&Graph)` built from `dataset_quad_iter` for held rows and the withdrawn reifiers that `dataset_quads_with_history` adds today, flagged withdrawn (D5; reifier metadata stored as tuples, plan D1). `is_current(&Graph)` compares the kernel's `revision`, `content_revision` and `semantic_observation_revision` captured at build, so a holder rebuilds only when one moved (D4). The engine hands out the handle; who holds it is the consumer's (plan D3).
- `impl QueryableDataset for &IndexedDataset` with `InternalTerm = TermId`. `internalize_term` is a dictionary lookup; a term the dictionary lacks maps (amended by D8: through a per-query dictionary, not one reserved id) to an id that matches no row and externalizes back to the term it came from, so a constant in a query never panics. `externalize_term` resolves. The expression-term fast path (`externalize_expression_term`) is P3.
- Entry points: `sparql(graph, query)` builds `for_graph` once per call and evaluates; `sparql_on(&IndexedDataset, query, View)` evaluates over a held dataset; `sparql_with_history` becomes `sparql_on` with `View::History`; `sparql_resource_members` follows. The `dataset_quads_with_history` walk and `GraphDataset` are deleted (git keeps them).
- Conformance: `tests/w3c_conformance/sparql.rs` gains an `indexed` layer that loads each case's data through `IndexedDataset::from_quads` and evaluates with the same `spareval`; the ledger's SPARQL 1.1 and 1.2 query-evaluation rows gain its score beside the parser layer's 230/244 and 64/65.
- Instrument: `adapter_scale` replaces its materialized column with the indexed dataset and keeps the adapter column until the adapter is deleted, then reports indexed against the brief's §1 numbers; the table at 1k and 10k is appended to the brief's §1.
- Done when: the query tests in `src/query/` and the crate's other tests pass unchanged; at 10k nodes a point lookup is under 1 ms, the two-hop join under 50 ms, bound predicate and full scan within 2× the brief's materialized column, the build within 2× of 827 ms, and the index under 10 MB (brief §4); the conformance indexed layer scores no lower than the parser layer; `cargo clippy --locked -p mere-rdf --features query --all-targets` clean; `cargo check -p mere-rdf --features query --target wasm32-unknown-unknown` passes; one commit on Mere main through the hooks, pushed, with the ledger and the brief's §1 updated in it.
- Needs: P1 on `eidetic` main and Mere's lock moved to it.
- **Followed up 2026-11-11** (Mark: "you can follow up on these"): `is_current` also compares the kernel's structural and content input keys, since two graphs can share the three counters; `sparql_resource_members_on` takes a held dataset; `mere-view`'s `SessionSubgraphs` holds one dataset across its reconcile paths, so saved queries at one graph revision share one build (the consumer's own wiring, per D3); and a second clippy hook builds `mere-rdf` and `mere-view` with the `query` feature, which no hook did when the lock broke it.
- **Landed 2026-11-10** on Mere main. At 10k nodes over a held dataset: point lookup 0.09 ms, two-hop join 9.6 ms, bound predicate 1.4 ms, full scan 7.6 ms, index build 329 ms, index 3.8 MB (the dictionary's terms not counted); `sparql()` per call, which rebuilds, is about 360 ms. Conformance indexed layer equals the parser layer (230/244 and 64/65). The table is in the brief's §1. `sparql_on` takes the coverage note as an argument, since coverage can change with no revision counter moving; `dataset_quads_with_history` stays for the serializers and only the query path left it; `QuadMeta`'s asserter and statement are unset in this cut. The landing regenerated `Cargo.lock` whole (Mark: "Regenerate the whole lock"): the p2mere 0.10 move had left spareval on two `digest` versions, so `--features query` did not build on main and no hook enabled the feature to see it.

## P3. Measure and refine

- The 100k row of the brief's §1, once the kernel's statement-id index has landed (its own session) and the build is linear.
- `externalize_expression_term` decoding literals straight to `ExpressionTerm`; measured on a `FILTER` over a numeric literal at 10k and 100k.
- Load, mutation, refresh and frame cost measured separately (engine split plan, Constraints). Incremental maintenance is opened as its own plan only if the refresh number at the scales the host sees demands it (D4); deriving reifier rows on read only if the memory number does (plan D1). Neither is started on a hunch.
- Done when: the brief's §1 holds the 100k row and the refresh numbers, and each refinement either has a number that justified it or a line saying it was not needed.
- Needs: P2; the kernel statement-id index session.
- Progress, 2026-11-12: the engine split's P5 baseline (pre-fold, Mere `0f641d525`, `adapter_scale`'s
  `edit_scale` table; brief §1, "P5 baseline (pre-fold)") holds the 100k row and the refresh numbers.
  Refresh after one edit is a full rebuild: 243 ms at 10k nodes, 2.61 s at 100k. Per-edit assert is
  0.004 and 0.010 ms, and `is_current` is under 1 µs. Withdraw is 0.12 ms at 10k and 5.0 ms at 100k
  (40x for 10x the nodes, a scan in the kernel's withdraw path, chased in its own lane). The index is
  38.4 MB at 100k with reifier rows stored, which answers the memory question for D1: deriving on read
  is not needed. That run was on a busy machine and is being repeated quietly. Open in P3:
  `externalize_expression_term` on a `FILTER` over a numeric literal. The refresh number opened P4 (D9).
- Progress, 2026-11-12 (withdraw scan): the scan was `Graph::semantic_statement_owner_keys`, which walked every
  Surface and Resource edge to find the buckets holding a statement id, and every withdrawal (and
  `update_assertion_record`) called it. It now reads the handle index the way `find_semantic_statement` does
  (indexed holders plus the dirty list, each checked, in scan order); no new structure, no persisted change.
  `edit_withdraw` median of 3 runs, release: 0.081 ms to 0.0011 ms at 10k nodes and 4.40 ms to 0.0010 ms at
  100k, flat in graph size. Machine busy only with this session's builds (none running during the runs). Test:
  `withdrawals_match_a_scan_and_a_state_rebuilt_from_scratch` in `handle_index_tests.rs`.

## P4. Incremental maintenance

- The held index applies the kernel's captured deltas between two revisions instead of rebuilding.
  The dictionary is append-only, so a new term takes the next dense id. An asserted quad is inserted
  into the six orderings (a binary search and a shift). A withdrawal flips the row's held flag, which
  the live view already skips, so nothing is removed and history stays one index (indexed-dataset D5).
  `is_current` becomes "caught up to this revision". A full rebuild stays as the fallback and as the
  oracle in tests.
- Done when: after any sequence of asserts and withdrawals, the incrementally maintained index equals
  a fresh `for_graph` (a property test over generated edit sequences); one edit at 100k nodes costs
  under 1 ms through the index; the P5 baseline's refresh rows are rerun and recorded beside the old
  ones.
- Needs: the engine split's P5 fold landed (it moves the code this touches); the kernel's withdraw
  scan fixed, since the kernel's per-edit cost is what a person then waits for.

## Decisions

D1 to D3 ruled 2026-11-09, one round, before the plan was written. D4 to D7 ruled 2026-11-10 on the P1 lane's forks, measured by the lane with a scratch bench before they were asked.

- **D1. Reifier metadata rows are stored as tuples (Mark, 2026-11-09):** asked how the first cut holds the ~75% of tuples that are reifier metadata (six orderings at 100k nodes: 77 MB stored, 19 MB derived on read). "Store them as tuples (Recommended)". Every pattern over a reifier is a range read like any other; memory is measured in P2; deriving on read stays a P3 refinement if the number asks for it. Outcome: applies indexed-dataset D3 and D5.

  Links: area The stack · draws on indexed-dataset D3, indexed-dataset D5 · plans quad-index P2 · summary Reifier metadata quads are indexed as tuples in the first cut; deriving them on read waits on a memory number.

- **D2. The module is `chartulary::quads` (Mark, 2026-11-09):** against `chartulary::index` (recommended) and `chartulary::concordance`. "chartulary::quads". The module names the data; `Dictionary`, `TermId`, `QuadIndex` and `View` live inside it. Outcome: applies indexed-dataset D1.

  Links: area The stack · draws on indexed-dataset D1 · plans quad-index P1 · summary The generic dictionary and quad index are chartulary::quads.

- **D3. The engine hands out the handle; consumer wiring is the consumer's (Mark, 2026-11-09):** asked whether the plan wires mere-view, the one caller today, to hold the dataset per revision. "Engine handle only; consumer wiring is the consumer's (Recommended)". P2 ships `IndexedDataset::for_graph` and `is_current`; a consumer adopts it in its own receipt. Outcome: applies Stack 1 ("the second host's adoption is that host's receipt").

  Links: area The stack · draws on mere:stack.1, indexed-dataset D4 · plans quad-index P2 · summary The engine ships the held-dataset handle and its currency check; consumers wire it themselves.

- **D4. Orderings are ordinals into one base table (Mark, 2026-11-10):** P1 F1. The lane benched both layouts: ordinals cost 4.48 MB at 80k quads and 44.8 MB at 800k, and a wide range through a non-base ordering runs 2 to 10x slower (200k rows at 800k: 3.2 ms against 0.31 ms); six copies of the four ids cost 11.2 MB at 80k once each row reaches its meta, over the brief's cap. "Ordinals into one base table (Recommended)". The slower wide range is about 16 ns per row, far under the evaluator's cost per solution. Outcome: applies indexed-dataset D3 within the brief's §4 budget.

  Links: area The stack · draws on indexed-dataset D3 · plans quad-index P1 · summary The six orderings are sorted ordinals into one base table of rows, 4.48 MB at 80k quads.

- **D5. TermId is a NonZeroU32 with the default graph at u32::MAX (Mark, 2026-11-10):** P1 F2, against reserving 0 or u32::MAX on a plain u32. "NonZeroU32, DEFAULT_GRAPH = u32::MAX (Recommended)". `Option<TermId>` takes the niche and is 4 bytes; `QuadMeta` falls from 20 to 12 bytes; ids start at 1. Outcome: applies indexed-dataset D2.

  Links: area The stack · draws on indexed-dataset D2 · plans quad-index P1 · summary TermId is a NonZeroU32; u32::MAX is the default graph; Option<TermId> is 4 bytes.

- **D6. Every (quad, meta) row is kept; a pattern yields each quad once per view, held first (Mark, 2026-11-10):** P1 F3, the same quad carried by two asserters or by one held and one withdrawn row, against one row per quad with held winning, and against yielding every row. "Keep every (quad, meta) row; pattern yields each quad once per view, held first (Recommended)". No asserter's record is dropped at build; the evaluator never sees a duplicate solution. Outcome: applies indexed-dataset D5 and quad-index D1.

  Links: area The stack · draws on indexed-dataset D5, quad-index D1 · plans quad-index P1 · summary Rows keep every asserter's record; a pattern read yields a quad once per view, a held row first.

- **D7. graph_names and contains_graph take the View (Mark, 2026-11-10):** P1 F4. Mere's reserved history graph holds only withdrawn rows, so without a view a live query would list it among the named graphs. "Take the View (Recommended)". Outcome: applies indexed-dataset D5.

  Links: area The stack · draws on indexed-dataset D5 · plans quad-index P1 · summary The graph listings take the view, so a graph whose rows are all withdrawn is absent from a live read.

- **D8. Unknown query terms go through a per-query dictionary (Mark, 2026-11-10):** the P2 lane showed the plan's "maps to a reserved id" cannot hold: `spareval` internalizes every computed value (BIND, aggregates, VALUES) and compares ids for joins and grouping, so one shared id would merge them all. It built a per-query overflow dictionary whose ids sit above the held dictionary's, with a test over VALUES and grouping on a computed term. Against a side table on the held dataset: "Accept the per-query dictionary (Recommended)". The held dataset stays immutable and nothing grows across queries. Outcome: refines P2's wording; applies indexed-dataset D2 and D4.

  Links: area The stack · draws on indexed-dataset D2, indexed-dataset D4 · plans quad-index P2 · summary Terms a query computes or names that the dataset lacks are interned in a per-query dictionary above the held ids.

- **D9. The refresh number opens incremental maintenance (first reader, 2026-11-12, listed to Mark):**
  the indexed-dataset brief's D4 left incremental maintenance for "only when measured refresh cost
  demands it", and P3 asked for the number at the scales the host sees. The P5 baseline answers it
  without waiting for a host's edit rate: one edit costs a 243 ms rebuild at 10k nodes, past the
  100 ms a person notices, and 10k nodes is a small browsing history; at 100k it is 2.6 s. So P4 is
  opened, scheduled after the engine split's P5 fold, with its equality test as the done-condition.
  The shape refines indexed-dataset D2 and D4 without reversing them: the append-only dictionary,
  ordered inserts and the held-flag flip, with a rebuild as fallback and oracle. Outcome: applies
  indexed-dataset D4; refines indexed-dataset D2.

  Links: area The stack · draws on indexed-dataset D2, indexed-dataset D4 · plans quad-index P3, quad-index P4 · summary A 243 ms rebuild per edit at 10k nodes opens incremental index maintenance, after the P5 fold, proved equal to a fresh rebuild.
