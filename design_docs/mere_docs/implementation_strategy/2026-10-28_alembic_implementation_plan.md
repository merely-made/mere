# Alembic implementation plan: memory levels, the granted-agent workshop and the image store

**Status:** 2026-10-28: cut from `2026-06-24_alembic_implementation_plan.md` and
`2026-07-06_node_image_externalization_plan.md`, archived in `archive_docs/2026-10-28/`. This
successor keeps the keyword `alembic-implementation`, so its Decisions stay as they were. Landed:
the codicil spine (slice A), the memory-level read model and manual forgetting (slice C's spine),
Athanor's forgetting and consolidation passes in `mere-athanor`, merge by identity
(`compose_graph_codicils`), the W0 shared workshop core (qualified 2026-10-25), and image-store
phases 1-3 with the bounded decoded cache and the orphan GC in library code. Open: W1-W3 (the
two-host product adapters and the receipt), slice C's promote, demote, editable-policy and
by-sessions tails, slice D's scheduling, facet pass and surfacing, the image GC's host caller, Phase
4's closure and Phase 6's re-measure.

**For:** the Alembic as a memory and workshop subsystem: a three-level memory model, graph codicils
over eidetic, the Athanor daemon and the granted-agent workshop (create, grant, run, observe,
interrupt and dissolve a helper) whose first workflow is a reviewable evidence brief from retained
passages in Turnstone and Graphshell (Mark, 2026-10-24: reconcile the workshop with Servitor and
Armillary and flesh out a first workflow in two hosts); and moving node preview imagery out of the
kernel into content-addressed blobs that Athanor sweeps (inline image bytes were 64% of the kernel's
live heap at 50k nodes).
Not here: Servitor's admission and run semantics and the later R2 and R3 contracts
(`2026-10-28_graph_behaviors_plan.md`); scheduling Athanor's passes as a Djinn service
(`2026-10-28_family_update_resident_plan.md`); how the Distillery Chronicle shows Alembic work-runs
(`2026-10-28_projection_grammar_plan.md`, whose W5 waits on this plan's workshop emitting runs); the
event log and Timeline of old slice E, superseded by `GraphJournal` and the source-time scrubber
(`2026-10-28_graph_view_curation_plan.md`); the LoRA lane (the local models harness brief and
`2026-10-28_distillery_models_plan.md`); custody for private rows (`2026-10-28_custody_plan.md`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this plan (DOC_POLICY §8).
Evidence that matters is a test in the repository or a line in Findings.

## Constraints

- Ownership (`archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:276-306`): Alembic
  owns the shared workshop view model, controls and embeddable surfaces (recall stays selectable on
  its own); Athanor owns installed helper definitions and bindings, selected versions, goal and
  review policy, persisted lifecycle decisions, run journals and proposal orchestration, using
  Servitor's transitions and no second reducer; Servitor, in Eidetic, owns admission, revalidation
  and pure run reduction; Armillary executes and supplies no permission, retry eligibility or effect
  completion; Djinn composes Athanor and supplies process lifetime and scheduling, and no host
  invents a second workshop authority; source and destination owners authorize scoped reads and
  commit and acknowledge product effects. No second GraphSession writer is opened to apply a brief,
  and there is no source-home move, dependency repin or new crate (`:480-485`).
- Athanor proposes and never mutates graph truth or edits existing codicils (eidetic R0), and runs
  async off the UI thread (`:135-176,244-256`). Resolved with Mark on 2026-06-24: redaction strips
  thumbnails, favicons and session state by default with per-field opt-in, keeping structure,
  addresses, tags, classifications and provenance; a tag or bookmark promotes a node to long-term in
  place (decision #2). June's Steward and Apparatus placement is historical.
- First workflow (`:307-385,461-520`): acquisition disabled and no model for the first gate; at most
  four passages of 2 KiB each and 8 KiB total, a note body of at most 16 KiB, labels and references
  at most 4 KiB; one context read, one body evaluation, one proposal emission; the fixture limits
  are three decisions, two tool calls, zero model tokens, 30,000 ms and zero consecutive retryable
  failures; the helper holds no source-mutation, acquisition, credential or destination-write grant
  and acceptance does not widen its grants; the destination's gate creates the note and the commit
  keeps both the proposing run and the accepting human; a completed generation run is never
  presented as a completed destination commit.
- Acceptance recovery: the destination durably associates the acceptance operation id, proposal
  digest, created resource identity and mutation result; retry reuses the id and a different digest
  under it refuses; if that association and the graph edit cannot share a transaction, the adapter
  specifies and qualifies its recovery path first; after an uncertain write the owner is queried or
  reconciled, never regenerated, and `Reconciling` stays when neither completion nor absence is
  proved. Two windows of one application do not satisfy the two-host gate.
- Private passage copies, proposals and journals follow the owner's sealed storage and lock policy;
  with no qualified sealer private inputs are refused, never stored in clear. Retention removes
  resolved content only under explicit policy and keeps an unresolved operation's evidence or
  refuses further work at the bound (initially 32 runs and 1 MiB per installation). Fixtures start
  synthetic and public; model-driven rows need a real provider receipt and evidence of value beyond
  deterministic assembly.
- Image store
  (`archive_docs/2026-10-28/2026-07-06_node_image_externalization_plan.md:40-72,148-170,241-258,380-393`):
  preview imagery is experience, not truth; blobs are BLAKE3 content-addressed under
  `content/image/<hash>`; a node holds a role-keyed `ImageRef` map; authored `Node::body` stays
  inline; legacy journal variants stay readable, replaying to nothing; GC is mark-sweep against
  every live reference with a recheck at apply, favicons disposable, snapshots node-precious,
  promoted nodes keeping all imagery.

## Phases

### C. Memory levels: promote, demote and an editable policy
The model is a pure read model (`MemoryLevel`, `EvictionPolicy`, `evictable_short_term`, `census` in
`crates/distillery/alembic/src/memory_levels.rs`) and tagging already promotes. Open: render Recent
and Saved over it in the surviving hosts; an editable, not only visible, eviction policy in persona
settings; a promote action from Recent rows and a demote from Saved; and the by-sessions policy,
deferred behind a per-node last-session stamp the snapshot does not carry. The ownership repair
round of 2026-10-28 finds that `last_session_visited` has no writer, so `KeepSessions` never evicts,
and offers either to install the launch counter or to remove the session clock,
`PersonaSettings.session_count` and `KeepSessions`; Mark has not answered.
- Done when: bookmarking or tagging promotes a node to Saved durably, the eviction policy is visible
  and editable, and Recent evicts per policy.
- Needs: the session-clock fork (lane L1 of the graph semantics plan).
- Rulings: decision #2 (Constraints)
- From: `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:112-134,660-667`;
  `2026-10-20_ownership_repair_plan.md:440-448`

### D. Athanor: scheduling, facet pass and surfacing
The forgetting and consolidation passes exist (`propose_forgetting`, `apply_forgetting`,
`propose_consolidation`, `apply_consolidation` in `ports/distillery/athanor/src/lib.rs`). Open: the
steady-heat actor that runs them in the background (the crate's module doc says "still to come", and
its 2026-09-02 ruling gives the scheduler to Djinn), the facet pass, surfacing live passes, and
thresholds as settings. This plan owns D's remaining workshop integration; automatic cadence and
facet extraction stay later extensions, and a future page engram should supply Fleece's extracted
`Article` to Athanor when that consumer lands.
- Done when: Athanor runs in the background, evicts eligible short-term, emits consolidation
  proposals, and stays inside R0.
- Needs: Djinn composing Athanor as a service (the family update resident plan).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:135-176,297-300`;
  tails `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:610-618`

### W1. Turnstone owner adapter
Turnstone is the first embedding host, already consuming Servitor's run reducer and retained-source
selection. One versioned helper artifact is installed in each host-owned workspace with its own
`ResidentId`, subject and grants, binding generation, run namespace and destination. Both hosts must
expose the same versioned brief operations (checked arguments, scope, preconditions, effect class,
receipts); the app-host action envelope is an ABI, not a semantic catalog. Turnstone's research-note
chooser binds an actual Knot document and is not the destination operation, and its
`HandoffUntracked` default cannot supply the required atomic acceptance receipt.
- Done when: normal retained-source selection, shared workshop controls, a real target gate and
  commit and a fallible persistence acknowledgement work, and accept, reject and citation routing
  work without acquisition.
- Needs: W0 (landed); the accepted-note adapter and its recovery path specified and qualified.
- Rulings: alembic-implementation D8
- From: `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:379-384,407-435,486-491,540-544,575-582`

### W2. Graphshell owner adapter
The identical helper body and workshop view and domain model operate through Graphshell's owner (its
retained-content producer and owned GraphSession) with independent grants and ids, and no Turnstone
workflow code or database reader is copied. Pandect's `GraphSession::transact_deltas_published`
joins journal changes and addressable graph publication but not an evidence-brief operation ledger
with the retained note Resource, and Graphshell's generic graph editor does not implement the
accepted-note adapter.
- Done when: the identical helper body and workshop view and domain model operate through
  Graphshell's owner with independent grants and ids, and no Turnstone workflow implementation or
  database reader is copied.
- Needs: W0; the ledger-plus-Resource transaction or its specified recovery path.
- Rulings: alembic-implementation D8
- From: `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:379-384,407-409,492,527-531,575-582`

### W3. Recovery and two-host receipt
The mandatory matrix, per host and not only on the shared fixture: exact three-source order, Unicode
quotation spans and versioned citations; empty, oversized, unauthorized and missing inputs; title
and URL collisions; a fourth source outside the grant never entering context; changed versions,
wrong owner or session and a stale destination revision refusing or requesting review; reject adding
no note; accept producing one durable note with run and actor attributed; duplicate acceptance
returning its receipt and a changed proposal under one operation id refusing; kill before dispatch,
after proposal persistence, after target application and before acceptance-receipt persistence,
reopening with no repeated edit, false success or discarded uncertainty; pause, resume, revoke,
dissolve, lock, budget exhaustion and delayed or wrong-generation replies, cooperative cancellation
kept distinct from observed join; cold reopen with origins unavailable keeping ids, the exact
proposal and result, pending review or reconciliation, and unchanged source content. The destination
ledger, Resource and graph transaction is the consequential acceptance boundary and is unqualified.
Automatic scheduling, model decisions, procedure learning, MCP and ACP, remote execution, fleet
migration, a general retention service, body adoption and private storage qualification have
separate gates.
- Done when: both hosts pass the matrix on pinned source and dependency cuts; headed evidence proves
  normal controls and product history; fresh-process evidence proves durable state; publication and
  physical fleet behaviour are reported separately.
- Needs: W1, W2.
- Rulings: alembic-implementation D8
- From: `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:386-460,483-519,569-571,584-608`

### IM1. Image GC host caller (image Phase 5)
`propose_image_gc` and `apply_image_gc` (athanor `lib.rs:182,198`, with per-role reference
forgetting) exist in library code and no host invokes them. The sweep diffs the store's
`content/image/*` keys against hashes referenced across every live node, and a reference appearing
between proposal and apply must be safe (recheck at apply, or drop only blobs unreferenced across a
full snapshot). The sweep cadence stays visible, since the store grows until a sweep.
- Done when: a blob referenced by any node is never orphan-proposed; a blob no node references is
  proposed and dropped; a forgotten short-term node drops its favicon reference but keeps an
  un-refetchable snapshot; a promoted node keeps all imagery; re-running the pass is a safe no-op
  (the plan claims library code only, not host-level verification).
- Needs: a host caller, scheduled by Djinn per D.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-06_node_image_externalization_plan.md:3-11,241-262,308-318,421-429`

### IM2. Resolved-image cache: what closes Phase 4
`ResolvedImageCache` (64 MiB default) exists in pictograph. Phase 4's condition names a
no-regression render-performance harness that left with meerkat.
- Done when: visible cards render favicon, preview and snapshot identically to before; the
  steady-state card-raster path shows no regression; the cache respects its byte bound (an eviction
  test). The harness is gone: see the question below.
- Needs: a living render path to measure on.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-06_node_image_externalization_plan.md:208-239`

### IM3. Footprint re-measure (image Phase 6)
Re-run `crates/probes/rdf-kernel-footprint/` with images externalised.
- Done when: the probe confirms the images category falling from about 352 MiB to the reference size
  (about 2 MB of `ImageRef`s), the live truth footprint at 50k nodes moving from about 553 MiB into
  the low 200s, with the render cache accounted separately and boundedly, recorded here and in the
  footprint gate note that also bounds the interned-slotmap kernel (the graph semantics plan's
  tail).
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-06_node_image_externalization_plan.md:264-272`

### IM4. Image-store risks still to verify, and deferred knobs
Verify: wasm and OPFS resolution is async, so a render-time block would stall the frame (async
prefetch with a placeholder); back up `graph.json` before the first externalising save; cold-cache
latency across a pan over many nodes; the favicon capture's RGBA-to-PNG write at store time, which
could not be checked. Deferred knobs: the render cache bound as a visible memory-levels knob (no
plan owns it), PNG versus raw RGBA on disk only if decode cost shows, GC cadence, thumbnail capture
policy and which roles a node populates (upstream, capture-side), and per-lane `Snapshot(lane)`
roles, now a minimal default. Compaction of graph codicils with rkyv (override
`TypedPayload::serialize_to_bytes`, mind the aligned-buffer read gotcha) waits on measuring the size
win.
- Done when: not stated in the source for these items.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-06_node_image_externalization_plan.md:274-307,312-315,421-429`;
  `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:703-705`; tails
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:149-154`

## Open questions

- What does Turnstone's `PaneContent::Alembic` show, and which surfaces carry Recent and Saved
  today? From: `archive_docs/2026-10-28/2026-06-24_alembic_implementation_plan.md:108-110`
- Does the by-sessions eviction policy survive, or is it removed with the session clock (see phase
  C)? From: `2026-10-20_ownership_repair_plan.md:440-448`
- What closes image Phase 4 now that its harness is gone: restate the no-regression condition
  against a living render path (Turnstone's or pictograph's canvas), or close on the eviction test
  and the byte bound alone? From:
  `archive_docs/2026-10-28/2026-07-06_node_image_externalization_plan.md:223-239`

## Decisions — workshop reconciliation (2026-10-24)

- **D8, keep Servitor's present source home.** Mark: "leave Servitor where it
  is for now." Servitor remains in Eidetic. This applies the existing stack
  boundary while the workshop earns a second run consumer; no move to
  Armillary or split of authority/run crates is part of this plan.

  Links: area Stack · draws on stack.1 · summary Servitor remains in Eidetic while Alembic composes its admission and run contracts.

The ownership reconciliation and concrete workflow in section 5 are the design
specified in response to Mark's request to reconcile the workshop with
Servitor and Armillary and flesh out a first workflow in two hosts. They are
not a claim of implementation, acceptance or authorization for new standing
automation, credentials, grants or publication.

## Findings

- 2026-10-06: the codicil spine is `crates/system/pandect/src/graph_codicil.rs`
  (`save_graph_codicil`, `open_codicil_as_session`, `list_graph_codicils`, schema
  `mere.graph-snapshot/v2`, v1 read as legacy); merge by identity is `compose_graph_codicils`.
- 2026-10-24: Turnstone's `src/resident_runs.rs` persists narrow run records and defaults to
  untracked external handoff; Graphshell's `ports/graphshell/src/retained_content/producer.rs` and
  `mere_host.rs` supply source presentation and an owned writer.
- 2026-10-25: the W0 shared records and controls are in `crates/distillery/alembic/src/workshop.rs`,
  Athanor's workshop domain and native worker adapter in `ports/distillery/athanor/src/workshop.rs`,
  the furnace source unchanged in `src/furnace.rs` under a default `furnace` feature. The core's
  normal dependency graph excludes Armillary, Tokio, ESP, Burn and CubeCL, the UI and kernel, and
  the inference and cache crates, and it checks for wasm32 with defaults and Armillary disabled.
- 2026-07-27: the image store lives in `crates/system/pandect/src/image_store.rs`; the bounded cache
  in `crates/canvas/pictograph/src/canvas/resolved_image_cache.rs`
  (`set_resolved_image_cache_limit_bytes` at `nodes.rs:65`); the orphan GC in athanor; Turnstone's
  read sites use the role-keyed model.

## Progress

- 2026-10-28: cut from `2026-06-24_alembic_implementation_plan.md` and
  `2026-07-06_node_image_externalization_plan.md`; the Decisions section is copied unchanged;
  nothing new landed.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- A JSON sidecar over fjall for short-term state and a `durable` flag for throwaway forks; the gate ("when branch operations land") never fired. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:229-232`
