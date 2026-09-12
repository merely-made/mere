# Recursive query experiments plan

**Status:** E1–E3 merged to main 2026-09-12 (`794a96bd`, `7ba1df59`, `ef64c1cc`); ascent dev-dependency and bench removed after recording the numbers; Q1/Q2 scoped below, awaiting Mark's ruling; the `graphlets` → subgraph rename unparked 2026-09-12 as its own commit.

## Why

Prompted by Tyler Hou's "Datalog go brrr" (https://tylerhou.com/posts/datalog-go-brrr/),
which argues graph traversals are relational recursive joins, and that the
representation-choice imperative code forces up front (adjacency list vs. edge
relation) is what a Datalog engine chooses for you, with semi-naive evaluation
and incremental view maintenance falling out.

Assessment against mere (2026-09-11, see Findings): mere already runs the
opposite bet deliberately. petgraph is truth, the relational view is an on-demand
projection (`chartulary::rdf`, `linked-data::sparql`), and the 2026-06-18
petgraph_rdf_plan measured relational-as-truth at roughly 11x memory and 18x
mutate cost. That decision is not reopened here. What the post does bite on is
narrower: two hand-rolled transitive closures of the same shape with very
different cost profiles, and derived state that is invalidated wholesale by a
single `u64` revision with no memoization. These three experiments probe that,
cheapest first, without touching the substrate decision.

## Scope

- E1: `canvas::fold_projection::collapse_descendants` (canvas crate only).
- E2: `graphlets::SessionGraphlets::reconcile_all` / `reconcile_delta`
  (graphlets crate only; no kernel change).
- E3: an `ascent` prototype of graphlet derivation, as a dev-dependency of
  graphlets with a timing harness on `std::time::Instant`. No criterion.

Out of scope: any change to `graph-kernel`'s write path or a relation delta
journal (a possible E2 phase 2, gated on E2's numbers); the substrate decision;
cross-application node reconciliation (a separate assessment).

## Phases

### E1 — fix the quadratic fold closure

`collapse_descendants` rescans `graph.relations()` (which fans out every edge
payload into `RelationView` rows) once per frontier node. That is naive
evaluation with no index: O(frontier × relations).

Done-conditions:
- The closure iterates the current node's incident edges (petgraph neighbors in
  the requested direction) and checks the family on the pair's payload, so cost
  is O(reachable × degree).
- A test with a chain or star hierarchy of a few hundred nodes plus unrelated
  noise relations asserts the member set is unchanged from the old walk.
- Existing canvas fold tests pass. `cargo test -p canvas` green.

### E2 — revision-gate graphlet reconciliation

`reconcile_all` re-derives every Linked graphlet's full member set on every
call, regardless of whether the graph changed.

Done-conditions:
- Each Linked graphlet records the kernel `Graph::revision()` it was last
  reconciled against; `reconcile_all` and the single-graphlet reconcile skip
  derivation when the revision is unchanged and return "no change".
- A forced path (or a revision reset) remains for callers that need a full
  re-derive, and persistence round-trips without serializing the revision as
  truth (it is a cache key, not content).
- Tests: unchanged graph → no derivation and no delta; mutated graph → delta as
  before. `cargo test -p graphlets` green.
- Findings record whether the kernel needs a relation delta journal for a
  true semi-naive expansion (E2 phase 2, not started here).

### E3 — ascent prototype of graphlet derivation

Done-conditions:
- `ascent` 0.8.x added as a dev-dependency of graphlets only.
- An `#[ignore]`d test builds synthetic graphs at three sizes (roughly 1k, 10k,
  100k relations, mixed families), extracts `relations()` rows as the
  extensional database, and derives Component and Ego(radius) membership with
  selector filtering as ascent rules.
- The test asserts set-equality against `derive_members` and prints wall-clock
  for both derivations plus the ascent load (EDB extraction) cost separately,
  since that load is the fair-comparison term.
- Findings record the numbers and a recommendation: rules layer above the
  kernel is worth a plan, or not.

## Scoped: two kernel selector-semantics questions (2026-09-12)

Scoped read-only after E1 surfaced them. Decisions are Mark's; nothing here is
implemented.

### Q1. `has_relation(Family)` vs `relations()` rows

Two read-side predicates decide whether an edge carries a family, and they
disagree for two of six families (`edge_payload.rs` `has_family` at ~207-228
vs `query.rs` `relation_rows` at ~38-95):

| Family | `has_family` | `relation_rows` | Diverges |
|---|---|---|---|
| Semantic | any statement, sub-kind, or predicate present | one row per statement with a `recognized_sub_kind` | yes |
| Traversal | sidecar present | events non-empty or `total_navigations > 0` | yes, in theory |
| Containment, Arrangement, Imported, Provenance | at least one sub-kind | one row per sub-kind | no |

Facts:
- An edge whose statements are all open-predicate (no recognized sub-kind) is
  Semantic to `has_relation` and invisible to `relations()`. That state is
  routine in production: JSON-LD ingest and inker link statements mint it via
  `assert_semantic_predicate` / `set_semantic_predicate`, it persists and
  round-trips (`snapshot/to.rs` ~151-175, `from.rs` ~185-226), and most
  canvas, intel and community test graphs are built entirely from it.
- The kernel's own doc and test rule for side A: `edge_ops.rs` ~102-108 says an
  edge carrying only such a statement still reports Semantic, pinned by
  `tests/snapshot_basic.rs` ~37-56. `TERMINOLOGY.md` defines a link as a
  statement, which also reads as side A. No doc rules on the empty-Traversal
  case; it is unreachable from mere's writers (`push_traversal` always records
  metrics) but an edge in that state would be un-collectable because
  `is_empty()` tests `traversal.is_none()`.
- Side A consumers: `edge_matches_selectors` (so all subgraph derivation),
  `retract_relations` candidate selection, the persisted `families` vector in
  every snapshot, and the `edge_kind_labels` facet. Side B consumers: 19
  dependents across 15 files, including every canvas edge projection, the
  glossary metrics, `capture.rs` table stats, the journal replay
  `fingerprint`, and `graphlets/classifier.rs`. The sharpest split is inside
  the subgraph crate: derivation uses A, shape classification uses B.

Options:
1. Tighten `has_family` to row semantics. One file, but it flips persisted
   `families` vectors and facets, drops every open-predicate edge from
   derivation and from retraction candidates, and contradicts the pinned doc.
2. Widen the rows to match `has_family`: emit a Semantic row per statement with
   an open-predicate row kind, and a Traversal row on sidecar presence. Adds a
   `RelationKind` variant; no persisted-format change; touches all 19 side-B
   dependents, changes journal fingerprints, and about 20 canvas relation-count
   assertions. Matches the documented rule.
3. Leave both predicates and reimplement `edge_matches_selectors` on
   `relation_rows`, so derivation and projection agree while `has_relation`
   keeps its sidecar meaning for the write path. One file; changes subgraph
   member sets on open-predicate graphs only; no persisted change.

Recommendation: 2 if the statement-level reading in TERMINOLOGY.md is the
intended truth (it is the only option consistent with both the doc and the
pinned test); 3 if the intent is only to make the two read paths agree
cheaply. 1 contradicts the tree's own ruling.

### Q2. Antiparallel and parallel edges in `edge_matches_selectors`

- The substrate is a declared multigraph (`chartulary/src/graph.rs` ~110-114).
  There is no one-edge-per-pair invariant, only a per-direction convention each
  writer enforces with `find_edge_key(from, to).unwrap_or_else(connect)`.
  `find_edge_key` is directed and returns the first edge.
- Antiparallel pairs with different families arise by construction: URL
  containment derivation always asserts child-to-parent, while hyperlinks and
  traversals usually run parent-to-child; folder imports (container-to-member)
  meet derived containment (member-to-container) the same way. Traversal
  back-and-forth yields Semantic on one arc and Traversal-only on the other,
  pinned as separate edges by `tests/snapshot_basic.rs` ~352-388.
- `neighbors_undirected_sorted` collapses the pair to one visit, and
  `edge_matches_selectors` inspects only `find_edge_key(a,b)` or, failing
  that, `(b,a)`. So a Traversal-selected walk seeded at the hyperlink source
  never crosses the pair, and the result depends on which end the BFS reaches
  first. The only selector test uses a unidirectional chain and cannot catch
  it. Same-direction parallel edges are equally invisible to `find_edge_key`;
  `copy_component_from` (`cross_graph.rs` ~191-196) is the one writer that
  bypasses the guard, safe today only because both endpoints are fresh.

Options:
1. Scan every edge on the pair in both directions in `edge_matches_selectors`
   and match if any payload matches. `query.rs` only; strictly more edges
   followed, so member sets only grow; add a test mirroring
   `snapshot_basic.rs` ~352-388.
2. Add a kernel pair primitive (`edges_between_undirected(a, b)`) beside
   `find_edge_key` and route all read predicates through it. Two files, wider
   API surface, the prerequisite if the per-direction convention is ever
   dropped.
3. Canonicalize direction per pair at every writer and keep direction as
   statement metadata. Changes `edge_count`, `relations()` rows, the persisted
   edge list, and journal fingerprints, and needs a migration. Only if the
   pair rather than the arc is meant to be the unit.

Recommendation: 1 now, with 2 folded in if Q1 lands as option 2 and the row
helper is being reworked anyway. 3 is a substrate decision, not a bug fix.

## Findings

- 2026-09-11 (assessment). Two closures of the same shape:
  `graph-kernel/src/graph/query.rs` selector-filtered BFS (`component_members`
  / `ego_members`, neighbor iteration, fine) and
  `canvas/src/fold_projection.rs` `collapse_descendants` (full relations rescan
  per frontier node, quadratic). Neither is memoized.
- 2026-09-11. Invalidation is comparison on a single monotonic
  `Graph::revision`; canvas caches (community, bridges, affinity, strategy
  positions) and every Linked graphlet recompute wholesale on any topology
  change. salsa was considered and rejected for dependency-tracking weight
  (2026-06-03 actor constellation plan). No existing delta journal on the
  kernel; `apply_graph_delta` is the write funnel a journal would hang off.
- 2026-09-11. SPARQL property paths over the projection already give
  declarative transitive closure today, but `dataset_quads` rebuilds per query,
  so it is not incremental.
- 2026-09-11. The post's headline speedups cite Doop/Souffle-class engines
  with adaptive planners. Embeddable Rust Datalogs (ascent, crepe) do
  semi-naive with fixed join order and no adaptivity; mere's graph is also
  non-monotonic (containment relations are retracted and re-derived), which
  pushes true incremental maintenance into differential-dataflow territory.
- 2026-09-11 (E1). No directed per-node relation iterator existed in the
  kernel, and `EdgePayload::has_relation(Family)` does not match `relations()`
  row semantics: Semantic rows require a recognized sub-kind and Traversal rows
  require events or metrics, which `has_relation` ignores. E1 therefore added
  read-only `Graph::outgoing_relations` / `incoming_relations` (the per-edge
  row expansion factored out of `relations()`) rather than the pair check.
  **Open:** the kernel's own `component_members` / `ego_members` filter via
  `edge_matches_selectors`, which uses `has_relation`, so subgraph derivation
  and the `relations()` projection disagree on which Semantic and Traversal
  edges count. Unchanged here; needs a ruling on which is truth.
- 2026-09-11 (E1). `edge_matches_selectors` checks
  `find_edge_key(a,b).or_else(|| find_edge_key(b,a))`, so an antiparallel pair
  with different families (a→b Semantic, b→a Containment) is judged by the
  first edge only. Possible kernel bug; unchanged.
- 2026-09-11 (E2). A true semi-naive expansion needs a revision-tagged journal
  of asserted relations. `apply_graph_delta` is not the sole write path
  (`assert_relation`, `remove_node`, edge_ops mutate directly), so a complete
  journal must hang off `bump_revision`. Additions are monotone and cheap to
  expand from; any removal can disconnect an arbitrary subset of a component,
  so a delta containing a removal still needs a full re-walk absent support
  counts or delete-and-rederive. Not started.
- 2026-09-11 (E3). ascent 0.8.1 never beats the hand-rolled walk on any of the
  six shapes at any size, and the gap grows with graph size: at 100k
  relations, Component[Containment] is 1.0 ms hand-rolled vs 11 ms ascent
  query (34x cold including EDB extraction); Ego r=2 is 0.006 ms vs 11 ms.
  Cause: ascent's `run()` takes each relation's total index into delta, so
  every run is a full re-derivation, O(|EDB|) regardless of answer size. EDB
  extraction alone (13 ms at 100k) is 35–40% of the cold total and already 6x
  the slowest hand-rolled query. Member sets equal in all 18 cases. Ego radius
  expressed cleanly as a `Dual<u8>` lattice; undirected walk needs each rule
  twice and doubles index memory. Six new crates in the dev tree, ~1 s of
  macro codegen. Full table in the E3 commit body.
- 2026-09-11 (E3 recommendation). Not worth a plan for seed-anchored,
  answer-bounded shapes. If a rules layer is ever revisited, measure a
  whole-graph query set (all-pairs, global closure, cross-cutting
  classification) sharing one EDB pass, and note that non-monotonic
  incremental maintenance is differential-dataflow territory, not ascent.
- 2026-09-11 (environment). The mere checkout's gitignored
  `.cargo/config.toml` patches `servo-paint` to `../genet/components/paint`,
  which no longer exists in the sibling genet checkout; every worktree failed
  resolution until a worktree-local override was written. Machine-local, not
  committed.
- 2026-09-11 (test). `source_time_canvas_scrubs_every_canvas_arrangement_without_rewriting_live_truth`
  in mere-canvas is flaky (about one run in five) because it compares two
  snapshots that embed wall-clock seconds. Unchanged.

## Progress

- 2026-09-11. Plan written; E1, E2, E3 dispatched to parallel worktree
  branches. Results to be recorded here.
- 2026-09-11. E1 landed: `collapse_descendants` walks incident edges
  (O(reachable × degree)); two tests including a brute-force equality check
  on a 600-node hierarchy under ~4,800 noise relations; kernel gained two
  read-only relation iterators. E2 landed: per-subgraph reconciled revision
  (serde-skipped), gate in `reconcile_delta`, `force_reconcile_all`, four
  tests, no public signature changes. E3 landed: `ascent` dev-dependency and
  an ignored bench; recommendation negative. All three on worktree branches,
  not yet on main. Open items above: `has_relation` vs `relations()` semantics,
  antiparallel selector check, the `graphlets` → subgraph crate rename
  (TERMINOLOGY.md, parked by Mark 2026-09-11).
- 2026-09-12. Merged E1, E2, E3 to main. Dropped the `ascent` dev-dependency
  and `tests/ascent_bench.rs` (finding recorded above; six crates out of the
  dev tree). Fixed the flaky canvas source-time test by zeroing
  `timestamp_secs` before comparing snapshots. Gitignored `.claude/` (agent
  worktrees). Scoped Q1 and Q2 with options and recommendations; ruling
  pending. Rename of the `graphlets` crate to the subgraph vocabulary started
  as a separate commit.
