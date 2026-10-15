# Link model: bounded shared reading and responsive-path probe

**Date:** 2026-10-15
**Status:** in progress. Mark authorized orchestration with cheaper subagents.
P1 implementation and an independent P5 fixture probe are the first slice;
neither phase is complete. The source baseline is `d4d2f88c4`.

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
The baseline judgment ledger already has 13 missing records and a snapshot
digest mismatch; this plan adds a covered record without relabeling that debt
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
