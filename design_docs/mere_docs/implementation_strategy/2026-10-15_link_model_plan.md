# Link model: bounded shared reading and responsive-path probe

**Date:** 2026-10-15
**Status:** in progress. The bounded P1 reading and independent P5 CPU probe
are implemented and bounded-tested; phase acceptance remains open. Work began at `d4d2f88c4`; the combined source
commit `339a86c9e` is based on published `39035965a` after renderer adoption.

## Authority and scope

The [link model](../technical_architecture/2026-10-15_link_model.md) owns the
design, L1–L6, open rulings and phase order. This plan owns implementation
assignments and evidence. Preserve separately attributed assertions; derive
the pair link. An assertion's RDF reifier supplies addressability without a
stored pair entity. P1 does not introduce claim-editing UI or pair promotion.

Use one integration owner for the portable contract and persistence. The core
sequence remains P1 → P2 → P3 → P4 → P6. The independent P5 evaluator may
advance on fixed fixtures while the core progresses. Shared host rendering
and selection changes are serialized. P2–P4/P6 are later work, not silently
included in this first delivery.

For P2, Mark is considering whether two independent reasons from the same
asserter are separately withdrawable assertions or combined evidence on one
assertion. P1 preserves every existing statement ID and does not change the
current write-path deduplication rule while that question is pending.

## P1: shared link reading over current storage

### Ownership

- The P1 agent owns portable shapes in scenograph and a shared kernel reader
  in Pictograph's existing canvas bridge. Graphshell and the live Pictograph
  pair inspector consume that reader. The kernel keeps its graph-only
  dependency boundary. The integration owner reviews the shared API first.
- The exporter agent owns the linked-data every-outgoing-arc fix and its
  regression. It edits neither the shared shapes nor the consumers.
- The integration owner owns this plan, the index, judgment record and final
  review. Agents neither publish changes nor widen dependency manifests.

### Validation and done conditions

Cover canonical unordered link keys with directed members, parallel and
antiparallel buckets, self-links, both strata, lifted Resource appearances,
scope and asserter preservation, exact statement IDs, id-less legacy sidecars
and incomplete coverage. The source authority supplies public identity and
revision; an in-process graph generation is not a public revision. Missing
owner context is explicit and does not manufacture an actionable source.

Graphshell's existing inspector record fields remain equivalent, with additive
source/revision availability metadata, and the live
Pictograph pair inspection consumes the same read model rather than rescanning
the graph. Preserve existing controller/hide semantics until P4. Verify the
existing consumer tests as well as focused new controls. P1 is complete only
when both real consumers and the shared contract pass their affected gates.

The export regression must expose two parallel outgoing surface buckets and
an incoming-only relation. Export all outgoing assertions in their original
direction and scope; retain stable quad deduplication. Resource export keeps
its existing separate owner.

## P5: independent responsive-path fixture probe

### Ownership and outputs

The probe agent works in a separate worktree. It owns a pure bounded evaluator
in Numen beside the path/field vocabulary, focused CPU tests and an inspectable
example. No kernel, persistence, Graphshell or Pictograph host changes belong
to this probe. The canonical plan and index remain integration-owned.

### Validation and done conditions

Hold endpoints fixed. Exercise attractor, flow and vortex fields with field
response and declared compatible-link attraction independently enabled,
disabled and combined. Include incompatible links, endpoint/identity
preservation, deterministic replay and explicit finite work/memory limits.
Report configured inputs and resulting paths without calling an allocated
share a measured force. Use existing dependencies.

A working fixture is a probe result only. P5 remains incomplete until real
links feed a host adapter, the bounded Graphshell fixture runs, and reviewed
captures establish what the different responses actually make legible.

## Qualification and integration

Use locked/offline commands where cached inputs permit, one Cargo build job,
and isolated lane targets. Serialize the two P1 agents' builds. The probe has
its own target. Run focused tests before affected package checks; avoid
repeated broad builds while the shared API is moving. Recheck the exact
combined source after integration and record failures as well as passes.

Before commits, compare documentation audits against the pre-edit revision.
The initial judgment ledger had 13 missing records and a snapshot digest
mismatch. The updated upstream baseline `39035965a` has 14 missing records; this plan adds a covered record without relabeling that debt
as a passing global gate. Publish source and receipts with exact revisions;
keep fixture, package, host, native and persistence claims separate.

## Findings

- **2026-10-15:** semantic assertion IDs and per-assertion reifiers already
  exist (`crates/graph/graph-kernel/src/graph/edge_data.rs`,
  `crates/graph/linked-data/src/lib.rs`). No new reification ruling blocks P1.
- **2026-10-15:** Pictograph's canvas feature already composes kernel and
  scenograph; the kernel has no scenograph dependency. Its pair inspector is
  a live second consumer. The roster card builder lacks an in-tree production
  call site, so it alone would not prove the second-consumer gate.
- **2026-10-15:** `node_direct_quads` reads one bucket per outgoing neighbor;
  its fix must preserve outgoing direction rather than reinterpret incoming
  arcs. Numen's path declarations supply vocabulary, not a completed path
  evaluator.

## Progress

- **2026-10-15:** three `gpt-6-luna` agents assigned narrow seams: reification
  and export, shared reading and consumers, and the isolated P5 probe. Initial
  read-only audits completed; Cargo metadata resolves locked/offline. Code
  editing proceeds under the ownership above. No phase acceptance is claimed.

- **2026-10-15, shared reading:** the portable contract lives in scenograph;
  the one reader lives in Pictograph's existing kernel/scenograph bridge.
  Graphshell's link inspector and Pictograph's live pair/dynamics inspector
  consume it. Their independent pair expansions are removed; selector-based
  cells and hide behavior remain for P4. Legacy sidecars have no invented
  assertion IDs. Raw family summaries are opt-in for the existing Graphshell
  output so the live pair inspector does not copy traversal history.
- **2026-10-15, source disclosure:** both consumers disclose unavailable
  source authority/revision when given only a kernel graph. Graphshell adds
  `LinkInspection.source` and an `inspect_link_with_source` entry point that
  preserves supplied context. Production host binding is still open. An
  independent source review found no concrete regression in direction,
  multiplicity, lifting, member metadata, coverage or visibility semantics.
- **2026-10-15, exporter control:** the new parallel-outgoing/incoming-only
  fixture failed against the old `node_direct_quads` implementation, exposing
  the missing outgoing bucket. It passes with an adjacency-indexed
  `surface_outgoing_relation_buckets` read. Review rejected a whole-graph scan
  per node; the final helper walks only that node's outgoing arcs.
- **2026-10-15, responsive-path probe:** fixed endpoints, field response,
  normal-projected compatible-link attraction and local smoothing run without
  graph or persistence authority. Caller link/endpoint IDs survive unchanged.
  Compatibility uses declared keys, with geometric endpoint correspondence;
  it is not an inferred semantic equivalence or a resonance model.
  Bounds are 64 links, eight fields, 3–32 points per path, 48 iterations,
  256 expanded field-expression nodes, depth 16 and two million estimated
  evaluation work units. Input coordinates/scalars and per-step motion are
  bounded. Field evaluation and compatible attraction can each be disabled.
- **2026-10-15, probe review controls:** a registry cycle nested below scalar
  scaling is rejected before evaluation. Recursive cost accounting now rejects
  a six-level nested gradient/dot expression before finite-difference
  expansion exceeds the work cap. A geometry regression preserves response
  under endpoint relabeling and reversed input direction. The final isolated
  Numen suite passed **65 tests**, and `responsive_paths` printed attractor,
  flow and vortex path samples for field-only, compatible-only and combined
  response. This is CPU fixture evidence; it does not establish P5's visual
  or integrated Graphshell acceptance.

- **2026-10-15, integration setup:** the P1-focused checks passed Scenograph
  **2/2**, shared-reader **2/2** and Pictograph topology **8/8**. The first
  Graphshell web-feature test compile exposed two existing sibling Woodshed
  fixture includes unavailable in the worktree layout, plus a test macro
  import removed with the old grouping helper. The macro import is restored
  under `cfg(test)`. The final run uses the existing Woodshed fixture via a
  temporary sibling-repository symlink; it does not alter fixture bytes.
  The first combined offline check then stopped at the uncached newly pinned
  Genet revision. `cargo fetch --locked --target aarch64-apple-darwin`
  populated the cache before the final offline checks. Neither setup failure
  is recorded as a test pass.

## Combined check receipt and next review

The [compact check receipt](../../../support/receipts/link-model-20261015/checks.json)
pins source commit `339a86c9e52dcd7985c364ae7ff569fa88cc54e9`, upstream
`39035965aa5c98c368eff58bc404e9d150034e04`, each changed Rust source's
SHA-256, the lockfile and the unchanged sibling fixture. The final commands
ran locked/offline with one build job and the isolated P1 target.

| Check | Command after the common `cargo` prefix | Result |
|---|---|---|
| Core library suites | `test --locked --offline -j 1 -p mere-kernel -p mere-linked-data -p scenograph -p numen --lib` | Kernel 431, linked-data 75, Scenograph 27, Numen 65 passed |
| Pictograph canvas library | `test --locked --offline -j 1 -p pictograph --features canvas --lib` | 448 passed; 18 existing release/probe tests ignored |
| Graphshell link inspection, host CPU with web feature | `test --locked --offline -j 1 -p graphshell --no-default-features --features web --lib link_inspection` | 10 passed; 287 unrelated tests filtered |
| Responsive path example | `run --locked --offline -j 1 -p numen --example responsive_paths` | exited successfully; all nine field/compatibility configurations printed |

These are **1,056 passing tests** across the named suites. New and substantive
changed Rust files pass scoped formatting checks. Scenograph's root module
and Pictograph's canvas module retain their pre-existing formatting debt;
each differs from the baseline only by its intended module declaration.

The first slice is implemented and bounded-tested. Phase acceptance remains
open for receiving-host review: supply verified source binding/revision
through the reader and Graphshell entry point, exercise the ordinary consumer
inspection flows, and keep P5's real-link adapter and capture comparison
separate. P2's pending assertion/evidence choice is not treated as ruled.
No new snapshot format, withdrawal state, opened-link layout or fan retirement
is claimed by this delivery.

Documentation audits against upstream `39035965a` add one indexed and covered
plan, with **no new finding** in any active-doc audit category. Active/indexed
counts move **284/283 → 285/284**; judgment coverage moves
**270/284 → 271/285**. The judgment command still exits 1 for the same 14
uncovered upstream documents and snapshot digest mismatch. The 16 broken
relative links, 151 missing known-root paths, six stale historical annotations
and 61 unheaded owned sources are unchanged baseline debt, not passing global
gates. `git diff --check` passes for this delivery.
