# The quad index: implementation plan

**Date**: 2026-11-09
**Status**: plan, 2026-11-09; D1 to D3 ruled the same day; nothing implemented. Carries out the [indexed dataset brief](../technical_architecture/2026-11-09_indexed_dataset_brief.md) (D1 to D5, 2026-11-09) under the [engine split plan](2026-11-07_engine_split_plan.md)'s P5. Three phases, in order: the generic index in Eidetic, the engine seam in Mere, then measurement and refinement. Mark: "ok, orchestrate/do the implementation plan for the index" (2026-11-09).
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
- `QuadIndex::build(impl IntoIterator<Item = (Quad, QuadMeta)>) -> QuadIndex`: dedups, then sorts the six orderings of D3. `pattern(subject: Option<TermId>, predicate: Option<TermId>, object: Option<TermId>, graph: Option<Option<TermId>>, view: View) -> impl Iterator<Item = (&Quad, &QuadMeta)>` picks the ordering whose prefix the bound positions fill and reads one contiguous range; `View::Live` skips withdrawn rows, `View::History` keeps them (D5). `graph_names()` and `contains_graph(TermId)` read the graph-first orderings. `len()`, `memory_bytes()` for the measurement.
- Tests: every one of the sixteen bound shapes, both views, against a brute-force filter over a generated dataset with duplicates and several graphs; the default graph as `Some(None)`; `graph_names`; and one test that indexes `chartulary::rdf::to_quads` of a sample graph, tying the module to the crate's own projection.
- A short note `eidetic/design_docs/eidetic_docs/technical_architecture/2026-11-09_chartulary_g3_quads.md` in the G0 to G2 form, citing the Mere brief and this plan by path, with its `DOC_README.md` line.
- Done when: the tests pass; `cargo clippy --locked -p chartulary --all-targets` is clean on native and `cargo check -p chartulary --target wasm32-unknown-unknown` passes; one commit on `eidetic` main through the hooks, pushed; Mere's lock then moves by `cargo update -p chartulary`.
- Needs: nothing. Eidetic's `crates/chartulary` is 7,278 lines over 15 files today, none above 536.

## P2. The engine seam in Mere

In `crates/graph/linked-data/src/query/`, behind the existing `query` feature. `dataset.rs` (the borrowed adapter, 80 lines) is replaced; `query.rs` (210 lines) keeps its entry points.

- `IndexedDataset` (`query/indexed.rs`): a `Dictionary<oxrdf::Term>` and a `chartulary::quads::QuadIndex`. `from_quads(impl IntoIterator<Item = (oxrdf::Quad, QuadMeta)>)` for any dataset; `for_graph(&Graph)` built from `dataset_quad_iter` for held rows and the withdrawn reifiers that `dataset_quads_with_history` adds today, flagged withdrawn (D5; reifier metadata stored as tuples, plan D1). `is_current(&Graph)` compares the kernel's `revision`, `content_revision` and `semantic_observation_revision` captured at build, so a holder rebuilds only when one moved (D4). The engine hands out the handle; who holds it is the consumer's (plan D3).
- `impl QueryableDataset for &IndexedDataset` with `InternalTerm = TermId`. `internalize_term` is a dictionary lookup; a term the dictionary lacks maps to a reserved id that matches no row and externalizes back to the term it came from, so a constant in a query never panics. `externalize_term` resolves. The expression-term fast path (`externalize_expression_term`) is P3.
- Entry points: `sparql(graph, query)` builds `for_graph` once per call and evaluates; `sparql_on(&IndexedDataset, query, View)` evaluates over a held dataset; `sparql_with_history` becomes `sparql_on` with `View::History`; `sparql_resource_members` follows. The `dataset_quads_with_history` walk and `GraphDataset` are deleted (git keeps them).
- Conformance: `tests/w3c_conformance/sparql.rs` gains an `indexed` layer that loads each case's data through `IndexedDataset::from_quads` and evaluates with the same `spareval`; the ledger's SPARQL 1.1 and 1.2 query-evaluation rows gain its score beside the parser layer's 230/244 and 64/65.
- Instrument: `adapter_scale` replaces its materialized column with the indexed dataset and keeps the adapter column until the adapter is deleted, then reports indexed against the brief's §1 numbers; the table at 1k and 10k is appended to the brief's §1.
- Done when: the query tests in `src/query/` and the crate's other tests pass unchanged; at 10k nodes a point lookup is under 1 ms, the two-hop join under 50 ms, bound predicate and full scan within 2× the brief's materialized column, the build within 2× of 827 ms, and the index under 10 MB (brief §4); the conformance indexed layer scores no lower than the parser layer; `cargo clippy --locked -p mere-rdf --features query --all-targets` clean; `cargo check -p mere-rdf --features query --target wasm32-unknown-unknown` passes; one commit on Mere main through the hooks, pushed, with the ledger and the brief's §1 updated in it.
- Needs: P1 on `eidetic` main and Mere's lock moved to it.

## P3. Measure and refine

- The 100k row of the brief's §1, once the kernel's statement-id index has landed (its own session) and the build is linear.
- `externalize_expression_term` decoding literals straight to `ExpressionTerm`; measured on a `FILTER` over a numeric literal at 10k and 100k.
- Load, mutation, refresh and frame cost measured separately (engine split plan, Constraints). Incremental maintenance is opened as its own plan only if the refresh number at the scales the host sees demands it (D4); deriving reifier rows on read only if the memory number does (plan D1). Neither is started on a hunch.
- Done when: the brief's §1 holds the 100k row and the refresh numbers, and each refinement either has a number that justified it or a line saying it was not needed.
- Needs: P2; the kernel statement-id index session.

## Decisions

Ruled 2026-11-09, one round, before the plan was written.

- **D1. Reifier metadata rows are stored as tuples (Mark, 2026-11-09):** asked how the first cut holds the ~75% of tuples that are reifier metadata (six orderings at 100k nodes: 77 MB stored, 19 MB derived on read). "Store them as tuples (Recommended)". Every pattern over a reifier is a range read like any other; memory is measured in P2; deriving on read stays a P3 refinement if the number asks for it. Outcome: applies indexed-dataset D3 and D5.

  Links: area The stack · draws on indexed-dataset D3, indexed-dataset D5 · plans quad-index P2 · summary Reifier metadata quads are indexed as tuples in the first cut; deriving them on read waits on a memory number.

- **D2. The module is `chartulary::quads` (Mark, 2026-11-09):** against `chartulary::index` (recommended) and `chartulary::concordance`. "chartulary::quads". The module names the data; `Dictionary`, `TermId`, `QuadIndex` and `View` live inside it. Outcome: applies indexed-dataset D1.

  Links: area The stack · draws on indexed-dataset D1 · plans quad-index P1 · summary The generic dictionary and quad index are chartulary::quads.

- **D3. The engine hands out the handle; consumer wiring is the consumer's (Mark, 2026-11-09):** asked whether the plan wires mere-view, the one caller today, to hold the dataset per revision. "Engine handle only; consumer wiring is the consumer's (Recommended)". P2 ships `IndexedDataset::for_graph` and `is_current`; a consumer adopts it in its own receipt. Outcome: applies Stack 1 ("the second host's adoption is that host's receipt").

  Links: area The stack · draws on mere:stack.1, indexed-dataset D4 · plans quad-index P2 · summary The engine ships the held-dataset handle and its currency check; consumers wire it themselves.
