# The quad index: implementation plan

**Date**: 2026-11-09
**Status**: plan, 2026-11-09; D1 to D3 ruled the same day, D4 to D7 (the P1 lane's forks) on 2026-11-10; P1 landed on `eidetic` main `60f3d60` the same day (Mark: "Land and start P2"); D8 ruled and P2 landed on Mere main 2026-11-10 (Mark: "Land after the lock repair"); P3 open, its refresh and 100k rows measured 2026-11-12; P4 (incremental maintenance) opened 2026-11-12 by D9, after the engine split's P5 fold; its Eidetic half landed on `eidetic` main `1371574` (chartulary 0.3.1) and its Mere half on Mere main `f57371773`, with one source per node and per resource; **P4 is done, 2026-11-12** (D10): its done-conditions are checked under P4, with one edit at 100k nodes refreshing in 0.32 to 0.72 ms on the development machine and 1.4 to 1.7 ms on the quiet ThinkPad. Carries out the [indexed dataset brief](../technical_architecture/2026-11-09_indexed_dataset_brief.md) (D1 to D5, 2026-11-09) under the [engine split plan](2026-11-07_engine_split_plan.md)'s P5. Three phases, in order: the generic index in Eidetic, the engine seam in Mere, then measurement and refinement. Mark: "ok, orchestrate/do the implementation plan for the index" (2026-11-09).
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
- **Measured, not asserted.** The instrument is `crates/graph/graph-kernel/examples/adapter_scale.rs` (moved by engine split P5 L2 and L3); its table is appended to the brief's §1, not restated elsewhere.

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
  oracle in tests. How the sorted insert is done was left to a measurement at 800k quads: direct
  insert into each ordering if one edit (a statement's quad plus its reifier rows) costs under 1 ms,
  otherwise a small sorted overlay merged into the base past a threshold.
- Eidetic: `QuadIndex::apply(asserted, withdrawn)` in `chartulary::quads`, generic like the rest of
  the module. Done when a property test shows a random sequence of asserts and withdrawals applied
  in place reads the same as `QuadIndex::build` over the same final rows (every bound shape, both
  views, `graph_names`, `contains_graph`); the existing tests pass; one edit at 800k quads costs
  under 1 ms; chartulary's version is bumped (a lower bound, never `=x.y.z`).
- Mere: `IndexedDataset` gains a refresh that applies the kernel's captured deltas since the index's
  revision, and falls back to `for_graph` when deltas are unavailable or the store was replaced
  wholesale. Done when, after any edit sequence, the refreshed dataset answers every query in the
  existing test corpus the same as a fresh `for_graph`; `adapter_scale`'s `edit_scale` table shows
  refresh under 1 ms per edit at 100k nodes; the P5 baseline's refresh rows are rerun and recorded
  beside the old ones in the brief's §1.
- Needs: the engine split's P5 fold (closed on Mere main, `2fa594c1f`) and the kernel's withdraw
  scan fixed (`cf674aded`); both are in.
- Progress, 2026-11-12 (Eidetic half): `QuadIndex::apply` on the `eidetic` branch `quads/incr`,
  `9992ab7`, chartulary 0.3.1, not pushed. The base table is append-only after the first apply, so
  SPOG becomes a sixth ordinal then (a built index keeps reading its sorted base directly). A read
  picks each quad's least-rank visible row, so a withdrawal's flip moves no ordering. Measured at
  800k quads, release, on a machine shared with other builds: direct insert of an 8-row statement
  0.54 to 0.80 ms (median of 101, three runs), the withdrawal flip 0.003 ms, the first apply after
  a build 7 to 20 ms. The overlay stalled 77 to 82 ms per merge and doubled a wide read (scratch
  bench), so direct insert was taken. Vecs grow by a sixteenth: 38.4 MB built, 44.2 MB after edits
  (doubling had given 83.2 MB). 125 chartulary tests pass, 3 new; breaking the flip or the
  held-first choice fails the equality tests.
- Open before the Mere half, as a fork: Mere's projection does not withdraw in place. A withdrawn
  statement's quad and reifier rows leave their graph, and its reifiers reappear in the reserved
  history graph (`dataset_quads_with_history`). A node edit (title, property, removal) also drops
  quads outright. A held-flag flip alone therefore cannot make a refresh read the same as
  `for_graph`. The kernel captures a withdrawal as the pair's whole statement bucket
  (`capture_statement_bucket`), not as row changes, so the refresh has to re-project what an edit
  touched and diff it against the index.
- Progress, 2026-11-12 (the fork answered): removal as a third input to `apply`, tombstoned in the
  existing skip bitset until a rebuild compacts it, with `tombstones()` exposed and no threshold
  inside chartulary; the flip path stays. Coordinator, relaying: "take (A)"; a projection change
  "changes a history ruling and is not ours to make", and physical deletion "pays an insert's cost
  for nothing a tombstone does not give". Eidetic `6cdc567` on `quads/incr`: random removals added to
  the equality tests (breaking removal fails them), 126 chartulary tests; at 800k quads on a quiet run
  0.30 ms per 8-row assert and 0.40 ms to remove 8 held rows and add 8 withdrawn ones. Its push to
  `eidetic` main was refused by the session's permission check and waits on Mark.
- Open before the Mere refresh is written (stopped 2026-11-12): one node's held quads re-project
  alone (`node_dataset_quads` in `graph-kernel/src/rdf.rs`, which the full projection maps over every
  node), and one bucket's withdrawn reifiers do too (`push_withdrawn_reifiers`). A pair's held quads
  do not: they are re-projected only as part of the `from` node's quads. What has no answer yet is
  the baseline to diff against. The index cannot name the rows a node contributed: reifier rows are
  keyed by statement-id IRIs, and the reifier IRI of a retracted or removed statement survives only
  inside a triple-term object. Identical rows from two sources (a surface node and a resource with
  the same IRI) collapse into one row. The dataset receives deltas only from whoever holds the
  graph's single recorder slot, and nothing ties them to the revision it was built at.
- Progress, 2026-11-12 (those answered; coordinator, relaying): the baseline is "(i). The dataset
  keeps what each source (node or resource) contributed, with a count for rows emitted by more than
  one source"; deltas: "the holder passes them in. The dataset does not get a recorder slot and the
  kernel is not changed"; in-neighbours: "re-project them for a node removal or a node IRI change. If
  that turns out not to be mechanical, make those two edit kinds fall back"; and "make
  `node_dataset_quads` reachable for this with the narrowest visibility that works". Built on Mere
  `quads/incr`, unpushed. `RdfSource` gains `sources`, `source_quads` and `dependents`, all
  defaulting to "does not split". The kernel's `Graph` splits into one source per node
  (`node_dataset_quads` as it was, private, plus `push_withdrawn_reifiers` on its outgoing buckets)
  and one source for the whole resource projection, since a resource's quads read other resources.
  `IndexedDataset::for_graph` indexes by source and keeps each source's rows and a count for shared
  rows. `refresh(graph, touched)` re-projects the touched sources, plus the resource source whenever
  `ResourceGraph::version()` moved, and their in-neighbours when a node's IRI changed. It diffs, and
  removes a row from the index only when its count reaches zero. `kernel::rdf::touched_sources`
  derives the sources from captured deltas. Incremental: every node edit, relation edits (both
  ends), resource edits, and URL and navigation changes (in-neighbours re-projected). Fall back to
  `for_graph`: node removal (the graph no longer holds the edges that named it) and import records.
  Field and coupling edits touch nothing. Gap detection by the counters is not possible: over 400
  random edits the advance of `revision` plus `content_revision` matched the delta count for some edit
  kinds but not all (12 edits moved two counters for one delta), so the contract is in `refresh`'s doc
  comment and `verify_against_fresh` is the test check. Tests: random edit sequences, 12 seeds × 60
  batches, each refresh equal to a fresh `for_graph` in full quad sets for both views (620
  incremental, 75 fallbacks), with four corpus-shaped queries compared every 15 batches; indexing by
  source equals the whole walk; a row two sources emit; an IRI change through a containment edge; a
  missed source caught. Dropping removal, the shared counts or the in-neighbour step each fails them.
  618 `mere-kernel` tests pass. Measured with `adapter_scale 10000 100000`, release, quiet machine:
  the per-source record adds 2.30 MB beside the index's 3.84 MB at 10k nodes and 21.6 MB beside
  38.4 MB at 100k. Incremental refresh per edit is 142 ms after an assert and 144 ms after a
  withdrawal at 10k (rebuild 210 ms), and 1.47 s and 1.48 s at 100k (rebuild 2.27 s), so the 1 ms
  target is missed. Where the time goes (10k, probed): re-projecting the resource source, 81k rows
  (the statements and node properties live in the resource graph), costs 103 ms; interning it 43 ms;
  the diff 0.1 ms; `apply` 0.07 ms. Index build by source: 196 ms at 10k and 2.07 s at 100k.
  chartulary's requirement is now 0.3.1, served locally by an uncommitted patch to the Eidetic
  worktree until Eidetic's `6cdc567` and `1371574` reach main.
- Progress, 2026-11-12 (resources split; coordinator, relaying: "A third off a rebuild does not
  meet D4's purpose, so go on: split the resource projection into one source per resource"): each
  resource is its own source, `resource_source_quads`, built from the per-resource
  `resource_metadata::quads`, the per-edge `resource_edge_quads` over its outgoing edges, and
  `push_withdrawn_reifiers`. The whole-graph walk is unchanged for the serializers, and the "by
  source equals the whole walk" test still passes. The earlier reason for one resource source was
  wrong: a resource's quads read only its own facets. A tag concept's label lives on the concept
  itself, and is its identity (`tag_concept_iri` of owner and label; `define_tag_concept` refuses a
  new label). So no resource reads another, and `dependents` for a resource names its in-edges only
  for a canonical IRI change. `touched_sources` names resources from the deltas: a record edit its
  resource, a resource-edge edit both ends, showing a resource the surface and the resource. No
  delta kind lacked the ids. `ProjectionSource::Resources` (every resource) remains for anything
  unattributed, and is assumed when the resource graph's version moves with no resource named. That
  happened 0 times in 645 random-test refreshes. Still falling back to `for_graph`: node removal and
  import records. Tests rerun: equality in both views over the random sequences (621 incremental, 75
  fallbacks); attributing resource-edge deltas to one end only fails them, as do dropping removal,
  shared counts or dependents. `adapter_scale 10000 100000`, release, quiet machine, refresh per
  edit (median of 51): after an assert 0.057 ms at 10k and 0.47 ms at 100k; after a withdrawal 0.060
  and 0.32 ms; a title edit 0.014 and 0.092 ms; a re-tag (drop one tag, add another, the nearest edit
  to a label change) 0.17 and 0.72 ms; rebuilds 185 ms and 2.07 s. A URL edit measured 0.007 ms
  (it moved no rows in this instrument; the IRI-change path is proven by its test, not timed). The
  first refresh after a build reaches 6.2 ms at 100k (the index's first `apply` grows its vectors).
  The per-source record is 3.00 MB beside the 3.84 MB index at 10k and 27.2 MB beside 38.4 MB at 100k
  (+71%). Index build: 166 ms and 1.81 s. The 1 ms target is met at 100k for an assert, a
  withdrawal and a title edit.
- Progress, 2026-11-12 (landing review, three items): (1) `touched_sources` matches all 50
  `CapturedDelta` variants by name with no wildcard arm, so a new edit kind fails to compile until it
  is mapped. `refresh`'s doc comment now gives that as the reason to derive `touched` only through
  it, or to rebuild: the unchecked case is a caller building the list by hand. (2) Per-source
  ordinals instead of rows: checked first that an ordinal is stable under removals and asserts. Rows
  are appended and removed ones stay tombstoned. A withdrawal's flip and its withdrawn-twin drop are
  the two places it would not hold, and Mere never withdraws through `apply`. Built with a
  chartulary `ordinal_of` and `row`. The record fell from 3.00 to 1.72 MB at 10k (−43%) and from 27.2
  to 14.4 MB at 100k (−47%). Refresh at 100k stayed under 1 ms (0.57 ms assert, 0.41 ms withdraw,
  0.094 ms title, 0.74 ms re-tag). That is short of the "half or more" the review set, so it was
  reverted, and the 71% stands. The reason is the per-source map itself: about 11 MB of the 14.4 at
  100k is the hash map of 200k sources and the shared counts, not the rows. (3) The resource
  projection reads nothing from showing: `shown_resources` lives in the kernel graph, not the
  `ResourceGraph`. The random generator now both replaces and clears a shown resource (302 sets, 61
  clears over the run), and equality holds with no fallback on a clear. Tests: 618 `mere-kernel`
  tests; random sequences 637 incremental, 63 fallbacks; the four mutations still fail them.
- Landed, 2026-11-12: Eidetic main `1371574` (Mark approved the push), Mere main `f57371773`. The
  first reader reran the gates on a fresh resolve against Eidetic main: 621 `mere-kernel` lib tests
  with `query,bundled-contexts`, `mere` with `query` 28 unit, 2 inspector and 2 conformance tests,
  wasm32 with `query`, and one chartulary 0.3.1 in the graph.
- Done-conditions, 2026-11-12 (the sentences above stand as written; each is mapped to its evidence):
  - Eidetic, "a random sequence of asserts and withdrawals applied in place reads the same as
    `QuadIndex::build` over the same final rows": met, with removals added (126 chartulary tests;
    breaking the flip, removal or the held-first choice fails them). "One edit at 800k quads costs
    under 1 ms": met, 0.30 ms per 8-row assert and 0.40 ms per Mere-shaped withdrawal on a quiet run.
    "chartulary's version is bumped": 0.3.1, required as a lower bound.
  - Mere, "after any edit sequence, the refreshed dataset answers every query in the existing test
    corpus the same as a fresh `for_graph`": met through a stronger check. The random sequences
    compare the refreshed dataset's quads with a fresh build's in both views after every step, and
    two datasets holding the same quads in both views answer every query alike. 637 refreshed in
    place and 63 fell back, and four deliberate breakages fail the comparison.
  - Mere, "refresh under 1 ms per edit at 100k nodes": met on the development machine (0.47 ms after
    an assert, 0.32 after a withdrawal, 0.092 for a title, 0.72 for a re-tag). On the quiet ThinkPad
    it is 1.71, 1.44, 0.60 and 1.57 ms. Recorded as met with both machines, by Mark's ruling (D10).
  - Mere, "the P5 baseline's refresh rows are rerun and recorded beside the old ones in the brief's
    §1": met, "Quad index P4, incremental refresh" in the indexed dataset brief, on the same ThinkPad
    as the P5 tables.
  - Follow-ups, none blocking: a flat per-source record (dense source ids and offsets over one array,
    estimated 27.2 to about 9.6 MB at 100k), queued after the engine split's first P6 lanes; the
    batched merge from the prior-art survey (`Code/testing/engine-split/quad_index_prior_art.md`) if
    slow hardware or ten times the rows need it; and detecting a missed edit by the journal cursor.
    The kernel journal appends one entry per captured delta (`record_as`, with `live_cursor` as
    `next_seq`). A refresh that takes its deltas from the journal by cursor range, and stores the
    cursor it reached, therefore sees a gap with no change to what the kernel records. That holds
    wherever a journal exists, as in a pandect session, but not for a bare kernel `Graph`.

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

- **D10. P4's 1 ms condition is met with both machines recorded (Mark, 2026-11-12):** the condition
  says "refresh under 1 ms per edit at 100k nodes" and names no machine. The Mere half measured
  0.47 ms after an assert and 0.32 ms after a withdrawal on the development machine; the first
  reader's rerun on the quiet ThinkPad, the P5 tables' reference (Ryzen 3 4450U), measured 1.71 ms
  and 1.44 ms (a re-tag 1.57 ms, a title 0.60 ms), against a 2.65 s rebuild. The cost is the shift of
  six ordinal arrays of about 3 MB each, which meets the slower machine's memory bandwidth (at 10k it
  is 0.08 ms there). Put to Mark against holding P4 open for a batched-merge lane and against
  rewriting the bound per machine. Ruling: "Met; record both machines (Recommended)". Asked in the
  same round whether to accept that the per-source record slows the full rebuild, now only a fallback
  (on the ThinkPad at 100k, index build 2.24 to 2.49 s, rebuild after an edit 2.48 to 2.65 s, a
  rebuild then a two-hop join 2.56 to 2.92 s): "Accept, record it (Recommended)". The batched merge
  from the prior-art survey stays the named fix if slow hardware or ten times the rows ever need it.
  Outcome: applies quad-index D9; refines P4's done-condition reading.

  Links: area The stack · draws on quad-index D9 · plans quad-index P4 · summary Incremental refresh at 100k nodes is met at 0.3 to 0.7 ms on the development machine and recorded at 1.4 to 1.7 ms on the ThinkPad; the 7 to 14% slower fallback rebuild is accepted.
