# Alembic implementation plan: memory levels, the granted-agent workshop and the image store

**Status:** 2026-10-28: cut from `2026-06-24_alembic_implementation_plan.md` and
`2026-07-06_node_image_externalization_plan.md`, archived in `archive_docs/2026-10-28/`. This
successor keeps the keyword `alembic-implementation`, so its Decisions stay as they were. Landed:
the codicil spine (slice A), the memory-level read model and manual forgetting (slice C's spine),
Athanor's forgetting and consolidation passes in `mere-athanor`, merge by identity
(`compose_graph_codicils`), the W0 shared workshop core (qualified 2026-10-25), and image-store
phases 1-3 with the bounded decoded cache and the orphan GC in library code. In progress: private-source storage and custody qualification (W0P), prioritized by Mark on 2026-10-28, then W1-W2 owner adapters and shared native controls. Open: W3 (the
two-host product and recovery qualification), slice C's promote, demote, editable-policy and
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
  and there is no source-home move or new crate (`:480-485`). Supplier lock updates
  for adopting the shared implementation stay scoped to that cohort.
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

### W0P. Private source and derived-state protection

Mark's D9 prioritizes this boundary before further workshop UI completion.
New private capture must seal the source, annotation, decoded recovery body and
sensitive request/recovery metadata. Legacy LocalOnly captures remain explicitly
unsealed; no real-store migration or implicit reclassification is part of this
slice. Tagged sealed identities and a separate namespace prevent collisions.

The complete workshop record, including source copies and proposals, is sealed
under live owner custody and an authenticated owner/installation context. CAS
compares the original stored ciphertext, not a newly encrypted equivalent.
Private accepted notes and operation requests must share the destination's
atomic snapshot/publication without putting clear private text in node bodies,
facets, diagnostics or ordinary exports. Mixed inputs inherit private status.
The human's accepted note remains readable through its owner after helper
retirement or journal removal; stable authenticated routing must not depend on
an active helper or widened grants.
Observed lock or custody loss invalidates the current generation, cancels and
joins work, clears transient private presentation, and refuses stale callbacks.
Authoritative custodian instance/generation fences each payload operation;
a boolean unlocked sample does not prove continuity. An already admitted write
can require reconciliation; lock is not a promise to undo it.

- Done when: synthetic private canaries survive exact-citation and cold-reopen
  flows through the current isolated custody process, while stored artifacts,
  journals, graph snapshots/sidecars and ordinary logs contain no cleartext
  canary. Lock/loss, changed generations, tamper, source revocation, failure and
  lost acknowledgement refuse or retain uncertainty without plaintext fallback.
- Needs: current PayloadSealer route and owner admission, the source actor,
  sealed workshop journal and private destination effect path. Existing frozen
  public receipts do not qualify this boundary.
- Rulings: alembic-implementation D9

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
owned GraphSession and retained body Resources) with independent grants and ids, and no Turnstone
workflow code or database reader is copied. Pandect's `GraphSession::transact_deltas_published`
joins journal changes and addressable graph publication, without an evidence-brief operation ledger
with the retained note Resource in the W0 cut. The owner adapter now uses a staged
operation transaction joining those records; this requires the product recovery qualification below.
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
ledger, Resource and graph transaction is the consequential acceptance boundary. Bounded shared and
Graphshell owner/process gates qualify that boundary; the full per-host matrix remains open.
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

- **D9, private-source support first (2026-10-28).** Mark: "Prioritize private-source support."
  This refines the ordering of the workshop slice: qualify sealed sources and
  derived private records before further UI completion. The already qualified
  public example remains bounded evidence. Existing unsealed captures are not
  silently migrated or described as sealed.

  Links: area Stack · draws on stack.1 · summary Private source and derived-state protection precede further workshop UI completion.

The ownership constraints and W1-W3 workflow above are the design
specified in response to Mark's request to reconcile the workshop with
Servitor and Armillary and flesh out a first workflow in two hosts. Their done-conditions do not authorize new standing
automation, credentials, grants or publication.

## Findings

- 2026-10-29: Graphshell's private receiving adapter retains source bodies,
  titles and UTF-8 ranges only in authenticated encrypted Resource facets.
  The graph's normal journal receives ciphertext deltas. Pandect's independent
  operation receipt must retain a commitment rather than the clear proposal;
  the encrypted operation and note remain in its existing atomic publication.
  This receiving implementation is still under compiler and host qualification.
- 2026-10-29: retiring a private owner must clear queued presentation actions as
  well as the visible proposal and reader. An unexecuted citation action carries
  a source title, so preserving it across a closed-owner DTO retains private
  presentation data even when the backend correctly refuses execution.
- 2026-10-29: the first Turnstone private chooser uses admitted sealed saved
  passages whose full bodies fit the existing workshop passage bound. It does
  not truncate long documents or guess a quotation range. Opening the workshop
  must load the current admitted source projection without requiring an unrelated
  search action; passage-range authoring remains a separate interaction.

- 2026-10-28: the existing Turnstone LocalOnly source path calls unsealed
  Eidetic/Fleece saves and retains decoded recovery bytes and CaptureRequest
  metadata. LocalOnly is a distribution classification, not proof of encryption.
  Private workshop qualification must cover these copies as well as the journal,
  accepted-note body and operation ledger.
- 2026-10-28: the private destination retains encrypted operation evidence and
  note content in the same snapshot. An authenticated routing envelope lets the
  owner read a retained note by operation id without loading the helper journal;
  deletion does not recreate its surface. This new path still needs host and
  current-custodian qualification.
- 2026-10-28: sealed storage retains visible content addresses and stable routing
  identities. It protects new payload and request copies; it does not hide
  equality or confirmation through content hashes, remove pre-existing browser
  addresses/titles, migrate legacy LocalOnly data, or qualify RAM erasure.
- 2026-10-28: helper retirement, custody-binding expiry and retained encryption
  keys have different lifetimes. The result reader outlives the helper journal
  while the wallet still retains its seal epoch. Key retirement/rewrapping is
  not qualified by these tests; the current immutable operation-ledger guard
  compares stored ciphertext bytes, so rewrapping needs an explicitly admitted
  owner migration and must preserve the same operation evidence.
- 2026-10-28: private payload requests bind the custodian instance and generation,
  not only the profile's public identity and unlocked state. Profile/master
  transitions must invalidate pending consent and notify clients even if the
  profile later returns; exhausted authority must close rather than reuse a
  live generation. These new custodian paths remain in receiving qualification.
- 2026-10-28: the current custody payload route limits a clear operation to
  512 KiB. The new sealed workshop path uses a smaller explicit record bound;
  public retention bounds remain separate. Context must be authenticated inside
  the payload because the custody route selects a fixed application scope.

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

- 2026-10-28: the shared native surface is opt-in through Alembic's
  `workshop-surface` / `workshop-native` features. Athanor's `workshop_driver`
  persists before dispatch and pauses orphaned runs only after its owner proves
  exclusive ownership; worker return and destination acknowledgement stay distinct.
- 2026-10-28: `workshop_destination` stages the same note effect for Pandect and
  graph-snapshot owners. Pandect joins its independent operation ledger, journal
  and publication in one backend transaction. A graph-only owner stores the
  ledger in a separate unshown Resource, retains it across every save, and
  acknowledges only after atomic snapshot replacement. Generic graph APIs do
  not enforce that retention policy on their own.
- 2026-10-28: Turnstone's adapter must use its canonical Canvas/runtime writer;
  opening another GraphSession would create a competing owner. An exclusive
  session lease is acquired before loading the graph and held through writes.
  The initial corpus is authored public example text in both hosts; saved-source
  private bytes still refuse. Synthetic example principals do not qualify
  production Personae identity or custody.
- 2026-10-28: native Graphshell composes its owner on a service thread. Turnstone's
  drafted adapter executes the bounded helper off-thread, but journal I/O and
  graph replacement still run on the UI service turn. This responsiveness limit
  and the full per-host W3 matrix remain open until separately qualified.
- 2026-10-28: revision tokens include the authoritative node-facet sidecar, with
  `install_facets` replacing the defaults synthesized during snapshot reconstruction.
  Overlaying an absent visit facet would retain a new wall-clock visit and make
  unchanged stored work appear stale. Same-owner receipt reconciliation reloads
  the durable graph/history after a lost storage reply before acknowledging it.
- 2026-10-28: an exact durable receipt can coexist with a stale live owner after
  a committed write loses its acknowledgement. A queued duplicate Accept must
  reconcile that owner before returning success, while preserving any later
  publication generation (`MereHost::workshop_commit`). An old receipt alone
  does not prove the live graph can open its result.

## Progress

- 2026-10-28: cut from `2026-06-24_alembic_implementation_plan.md` and
  `2026-07-06_node_image_externalization_plan.md`; the Decisions section is copied unchanged;
  nothing new landed.
- 2026-10-28: added the opt-in shared native workshop, Athanor owner driver and
  destination operation contract, Pandect receipt reconciliation, and the native
  Graphshell owner entry. The public Graphshell workflow is qualified; Turnstone's
  adapter remains in receiving qualification and the full two-host matrix stays open.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- A JSON sidecar over fjall for short-term state and a `durable` flag for throwaway forks; the gate ("when branch operations land") never fired. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:229-232`
