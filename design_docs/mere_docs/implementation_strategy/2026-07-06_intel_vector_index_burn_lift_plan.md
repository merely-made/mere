# Intelligence-Tier Vector Index: the burn lift (and HNSW alternative)

**Date**: 2026-07-06
**Status (2026-10-06):** P1 to P3's library half landed; consumer routing not wired; P4
deferred. P1's batched-cosine kernel with ndarray and wgpu parity landed in `bb1b1608`
(2026-07-08). P2's crossover was measured in `98111f60` (2026-07-08) and is recorded as
`AFFINITY_GPU_MIN_ENTRIES` = 1024 and `SEARCH_GPU_MIN_ENTRIES` = 4096 (index_burn.rs:39,43).
P3's keyed accelerators `nearest_over_index` and `affinity_pairs_over_index` landed in
`c6ab781b` (2026-07-08; index_burn.rs:113,144). The code is in
`crates/intel/esp/src/embed/index_burn.rs`, behind esp's `index-burn` / `index-burn-wgpu`
features. Open: no crate enables `index-burn` and nothing outside index_burn.rs calls the
accelerators, so affinity, recall and canvas search do not yet route to them (the first
done condition; see P3). P4 (HNSW) stays deferred.
**Related**: [burn_utilization_brief](../research/2026-07-04_burn_utilization_brief.md)
(Lane 1 GPU findings, Lane 5 force pass), [orrery_graph_intelligence_plan](2026-07-06_orrery_graph_intelligence_plan.md)
(where the O(N²) affinity scan lives), `crates/intel/embed/src/index.rs` *(historical citation)* <!-- doc-audit: historical-path --> (the flat
index this lifts), `crates/intel/eidetic-search` + `crates/canvas/pictograph/src/canvas/canvas_search.rs`
(the other two consumers).

## The insight

The Lane 5 P4/P5 wiring runs `affinity_pairs`, which does a per-node
`VectorIndex::nearest` over a flat (dense, `O(N)`-per-query) index. Per node that
is `O(N)`, so the whole affinity signal is **`O(N²)`** cosine work on the CPU.

Two facts make this worth a dedicated plan rather than a buried "large graphs want
an actor" note:

1. **It is the same kernel shape as the Lane 5 force pass.** `aether::forces::repulsion`
   already computes an `O(N²)` all-pairs interaction as a burn tensor program
   (`[N,d]` broadcast against `[N,d]`, reduce), and Lane 5 P2 measured it beating
   naive CPU from ~7× at 1k to ~17× at 4k on burn-wgpu. All-pairs cosine is that
   exact program with a different reduction (dot / norms instead of inverse-square).
   So the affinity signal is not stuck at CPU `O(N²)`; it can be a burn kernel that
   already has a proven sibling in the codebase.

   **Corrected 2026-10-06 (S14 pass):** `aether` was renamed `quint` in `5b91b2ea`
   (2026-07-09), and quint was folded into its owners in `eae87153`. The sibling kernel,
   here and in Path A's "sibling to `aether::forces`", is now `repulsion` in
   `crates/conatus/seiche/src/tensor_forces.rs` (l.104).
2. **The flat index is a shared ceiling, not an arrangement-only one.** The same
   `embed::index::VectorIndex` backs:
   - **arrangement** (`affinity_pairs`, this session),
   - **recall** (`eidetic-search` fuses the vector half against the lexical half),
   - **canvas search** (`embed::canvas_search` / `field_bridge`, the query-similarity
     field over canvas space).
   `index.rs`'s own docs say "an HNSW-backed implementation will follow once scale
   becomes a real constraint." One lift raises all three consumers together.

So this is a single investment with three beneficiaries and a ready-made kernel
template. That is unusual leverage and the reason it earns its own plan.

## Two lift paths (not mutually exclusive)

### Path A — batched cosine on burn (the "brute force is fine on GPU" lift)

Keep the flat, exhaustive index, but compute the all-pairs / query-vs-all cosine as
a **backend-generic burn tensor program**, sibling to `aether::forces`:

- `embed::index_burn::top_k_batched<B: Backend>(queries: [Q,d], corpus: [N,d], k) -> [Q,k]`
  (indices + scores), L2-normalized cosine as a matmul `[Q,d] · [N,d]^T -> [Q,N]`
  then a top-k reduce.
- Gated `index-burn` / `index-burn-wgpu` exactly like `field-burn` / `bert` — the
  default embed build stays pure-Rust and burn-free (the lexical/CPU path).
- `affinity_pairs` gets a burn fast-path above a crossover N (the L5 pattern: measure
  the crossover, route above it, keep the CPU path below where dispatch/readback
  dominate).

Wins where Path A is right: exact results, tiny code (one matmul + top-k), and it
reuses the L5 device-sharing story (D1). It stays `O(N²)` in FLOPs but on hardware
that eats `O(N²)` matmuls; the L5 numbers say that is a win from ~1k nodes.

### Path B — HNSW (the algorithmic lift)

An approximate-nearest-neighbour graph index (`O(log N)`-ish per query), CPU, no
burn. The right answer at large N where even a GPU `O(N²)` sweep stops paying, and
the answer for the browser/PWA target where GPU compute is least certain.

Path A and B compose: A is the cheap, exact, GPU-shaped lift that helps now (and is
one kernel); B is the algorithmic lift for the tail. A good sequencing is A first
(small, proven kernel shape, immediate three-consumer win at mid-N), B when a real
corpus makes `O(N²)`-anything the wrong asymptotics.

## Phases

### P1 — Batched-cosine burn kernel + parity
`embed::index_burn::top_k_batched`, backend-generic, `index-burn` / `index-burn-wgpu`
gated. ndarray↔wgpu parity test + a CPU-reference (naive flat `nearest`) correctness
anchor, the L5 `forces` test pattern. Done when batched top-k runs on both backends
matching the flat index's results.

### P2 — Crossover measurement across N and Q
Timing sweep (N, Q) CPU-flat vs GPU-batched, readback included, warmed, the L5
harness. Records where the batched GPU path beats the flat CPU scan for (a) all-pairs
(arrangement: Q=N) and (b) single/few queries (recall, canvas search: Q small). These
are different crossovers and both matter.

### P3 — Route the three consumers above the crossover
`affinity_pairs` (arrangement), the `eidetic-search` vector half (recall), and
`canvas_search` (query field) each gain the burn fast-path above their measured N,
behind the feature. Default build unchanged.

**Corrected 2026-10-06 (S14 pass):** P3 landed only its library half: the keyed
accelerators and the routing constants (`c6ab781b`, whose message says "Routing is the
caller's one-line check"). At mere 535bca11 only esp defines `index-burn`
(`crates/intel/esp/Cargo.toml`, l.80-81), no crate enables it, and nothing outside
index_burn.rs calls the accelerators, so none of the three consumers routes yet.

**Open, raised by the S14 pass (2026-10-06):** is Path A closed? Options: treat it as
closed, since the accelerators and constants exist and routing is each caller's one-line
check (the stance `c6ab781b` takes); keep P3 open until affinity, recall and canvas search
actually route above their crossovers.

### P4 (optional / later) — HNSW for the tail
Only if a real corpus makes the GPU `O(N²)` sweep the wrong shape. Pure-Rust, all
targets, the browser answer.

## Done conditions

- The affinity / recall / canvas-search paths can run their nearest-neighbour work on
  burn (Path A) with recorded crossover N per query-shape, default build still
  burn-free.
- The brief's Lane 5 "large-graph force pass" story gains its sibling: the *affinity
  computation itself* is a burn kernel, not only the physics force.

## Honest bounds

- **The flat CPU index is fine at current N.** This is a scaling investment, not a
  present bottleneck; typical graphs are far below any crossover. Sequence it when a
  consumer actually pushes N up (a large imported corpus, a full-graph re-embed),
  not speculatively.
- **Readback shape.** Like the L5 force pass, GPU-computed neighbours round-trip to
  the CPU (the index consumers are CPU-side). The win is compute throughput, not a
  zero-copy path, until/unless a resident-data consumer exists (the D1 question).
- **Path A is still `O(N²)` FLOPs.** It defers the algorithmic problem by throwing
  proven-cheap GPU matmul at it; it does not solve the asymptotics. That is Path B.
- **Browser target.** GPU compute in the browser is the least-certain leg (Lane 1's
  wasm-embed receipt is a named follow-on). Path B (HNSW, CPU) is the portable floor
  there.

## Progress

- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere
  535bca11, from the D2 record in support/doc-audit/d2/batch_46_s14_phase_b8.md: the
  status records P1 (`bb1b1608`), P2 (`98111f60`) and P3's accelerators (`c6ab781b`) as
  landed with consumer routing not wired, the `aether::forces::repulsion` citation is
  repointed to seiche, P3's routing is left as an open question, and a stray closing code
  fence is removed.
