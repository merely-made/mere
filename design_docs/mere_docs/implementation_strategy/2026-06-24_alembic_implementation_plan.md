# Alembic implementation plan: memory and the granted-agent workshop

**Status (2026-10-24):** workshop reconciliation and first two-host workflow
specified in [section 5](#5-workshop-reconciliation-and-first-workflow-2026-10-24);
implementation and acceptance remain open. Alembic/Athanor own the workshop
inside Distillery. Servitor remains in Eidetic, supplying admission and the
runtime-free run reducer; Armillary supplies execution mechanics. The first
workflow builds a reviewable evidence brief from selected retained passages in
Turnstone and Graphshell. Existing recall and furnace passes are implemented;
they do not establish a working workshop. Sections 1–4 retain the earlier
memory implementation record and its dated corrections.

**Earlier status (2026-10-06):** partially implemented. Slices A-C landed 2026-06-24, their host
halves in meerkat, retired with it 2026-07-18 (`c5f01064`); surviving library parts:
the codicil spine (`crates/system/pandect/src/graph_codicil.rs`) and the memory levels
(`crates/distillery/alembic/src/memory_levels.rs`). Slice D is half landed: the
forgetting (`d3893dd`, `95f3a20`) and consolidation passes live in `mere-athanor`
(`ports/distillery/athanor/src/lib.rs`, moved in `1bda73d5`). Merge landed as
`compose_graph_codicils` (`3e828a48`, 2026-06-30). Slice E's successor, the event log
timeline plan, was superseded on 2026-08-03 by the graph view curation plan
(`GraphJournal`). The LoRA lane was spun off to the local models harness brief. Open:
D's background scheduling, facet pass and Steward surfacing; the promote / demote UI,
editable settings, and the Timeline.

Earlier status (undated, 2026-09-05 reconcile): partially implemented: slices A-C landed; merge, promotion, event-log, LoRA, and settings/Timeline follow-ons remain deferred.

**Corrected 2026-10-06 (S14 pass):** engrams were renamed codicils on 2026-08-31
(`c51b9704`). The names this plan uses (`save_graph_engram`, `open_engram_as_session`,
`list_graph_engrams`, `GraphEngram`) are now `save_graph_codicil`,
`open_codicil_as_session` and `list_graph_codicils` in
`crates/system/pandect/src/graph_codicil.rs`, under the `mere.graph-snapshot/v2`
schema (v1 is read as legacy). Left unedited below as the historical record.

The [Alembic memory + engrams architecture](../technical_architecture/2026-06-09_alembic_memory_and_engrams.md)
is the memory design seed. This plan owns the workshop integration and its
first workflow; the [Servitor redesign](2026-08-13_graph_behaviors_plan.md#8-servitor-resident-and-run-redesign-2026-09-20)
owns reusable admission/run semantics. The [suite census](../../2026-08-22_turnstone_suite_composition_and_capability_census.md#74-granted-agent-and-automation-workshop-probable)
records the August workshop charter and September placement inside Distillery.

The Alembic is a multi-part subsystem (a 3-level memory model, graph engrams over eidetic, the docked
memory pane, the Athanor distillation daemon, and an event-log / Timeline). It is doc-only today, so
this is from-scratch, but the **spine already exists** and most of the early work is wiring.

## 1. Code reality (verified 2026-06-24)

**Exists:**

- **`GraphSnapshot`** — `Graph::to_snapshot` / `from_snapshot`
  ([graph-kernel](../../../crates/graph/graph-kernel/src/graph/snapshot/)), rkyv, already what
  session-runtime persists. The freeze/thaw primitive.
 - **The `Engram` envelope** — [eidetic-core](https://github.com/merely-made/eidetic/tree/main/crates/eidetic-core/src/engram.rs) *(historical citation)* <!-- doc-audit: historical-link -->:
  `schema` / `payload` / `content_hash` / `privacy` / `provenance` / `trust` / `bounds` /
  `envelope_version`, with `Engram::new`, `id() -> ManifestId`, `verify_integrity()`. Fully built.
- **The eidetic store, live in the host.** `eidetic::Store` (trait) + `eidetic_fjall::FjallStore`
  (`open(path)`) is **already opened in meerkat** (`main.rs`) and used today for deleted-node
  tombstones (`eidetic::record_deleted` / `list_deleted`, `node_ops.rs`) and the content store
  (`session-runtime/content_store.rs`). So engrams have a live home; the pattern to mirror is the
  `record_deleted` / `list_deleted` helper pair.
- **`armillary`** — the off-UI-thread actor framework (actor / message / pool), the shape Athanor is.
- **The peripheral-pane infra** — `frame::PaneContent` (Steward / Apparatus / Roster / Inspector /
  Gloss …), the list-pane render + scroll + a11y + the `Toggle*` registry commands. The template a
  new Alembic pane follows (Steward is the closest precedent: a docked list pane over a live store).

**Net-new:**

- The `mere.graph-snapshot/v1` schema def + typed `save_graph_engram` / `open_engram_as_session`
  helpers (the §3 freeze/thaw boundary).
- The Alembic pane (`PaneContent::Alembic` is aspirational, not yet in the enum).
- The 3-level memory model: short→long promotion (tag/bookmark), the configurable eviction policy,
  the Recent / Saved data.
- The Athanor daemon (an armillary actor) + its proposal-emission seam.
- The append-only `GraphMutation` event log + the Timeline (mostly net-new; the mutation *vocabulary*
  exists in `app/intents.rs`, the persisted log does not).

## 2. Phases (done-conditions, not dates)

### A — Graph-engram spine (the foundation)

Define `mere.graph-snapshot/v1`; add `save_graph_engram(store, graph, redaction)` (snapshot + redact +
serialize + `Engram::new` + write through the store) and `open_engram_as_session(store, id) -> Graph`
(read + verify + deserialize + `from_snapshot`), mirroring `record_deleted` / `list_deleted`. Wire a
host gesture: "Save graph as engram" (a registry command) and "Open engram" (thaw into a graph /
window, reusing the orrery at engram scope). Browsing is read-only; editing forks a thaw.

Done when a user freezes the focused graph to an engram, the engram persists (survives restart,
content-hashed, `verify_integrity` passes), and re-opening it thaws the same graph back.

**Landed 2026-06-24** (commits `af89808`, `4f51175`): `session-runtime/graph_engram` —
`save_graph_engram` / `save_graph_snapshot_engram` / `open_engram_as_session` / `list_graph_engrams`
over the eidetic typed-payload substrate (a `GraphEngram` newtype, orphan rule), `RedactionPolicy`
(conservative default strips thumbnails / favicons / session state). Host: `Command::SaveGraphEngram`
(palette + context menu + `>save_graph_engram`) freezes the focused graph to the private fjall store.
Verified: 6 tests incl. a real fjall save → close → reopen → thaw (survives restart). The thaw lib
primitive (`open_engram_as_session`) is built + tested; surfacing it as a **browse + click-to-open**
gesture lands in slice B (its UI home), so A is the freeze half + the open API, B is the open UX.

### B — The Alembic pane

Add `PaneContent::Alembic` + a docked list pane (Steward-shaped) with the three sections: **Recent**
(short-term), **Saved** (long-term), **Engrams** (the list from A, click → open in the orrery). A
`ToggleAlembic` registry command + the dock entry. Recent/Saved render from whatever C surfaces (until
C, they show a grounded subset: Recent = the nav/recent set, Saved = bookmarked/tagged nodes). The
context-binding toggle (global vs selection-filtered) per the dock contract.

Done when the Alembic pane opens from the palette / dock, lists engrams, and clicking one opens it.

**Corrected 2026-10-06 (S14 pass):** slice B landed in meerkat (`b029d83`, `7c6e5f4`; see
Progress) and was retired with it on 2026-07-18 (`c5f01064`). Turnstone registers a
`PaneContent::Alembic`; what it shows was not traced by the pass.

### C — The three memory levels

Promotion short→long as an affirmative act (tag / bookmark a node promotes it + its group); the
configurable eviction policy (by sessions or by days, user-overridable, never silent); free
demotion/deletion. The Recent (working set, eviction policy visible/editable) and Saved (durable kept
set) sections become real over this model.

Done when bookmarking/tagging promotes a node to Saved durably, the eviction policy is visible +
editable, and Recent evicts per policy.

**Spine landed 2026-06-24** (commit `9cacd41`): `session-runtime/memory_levels` is the pure read-model
(built spine-first, before the pane/settings wiring, because the chrome files are mid-churn under Mark's
shellbar work). `MemoryLevel` + `level_of`/`is_promoted` (a tag or pin promotes to long-term, per
decision #2); `EvictionPolicy` (KeepForever | KeepDays(n), default 30, with a visible `describe()`);
`evictable_short_term` (the ids a policy drops now, given host-supplied last-visit timing, only
short-term + dated-and-stale, promoted exempt, undated never dropped); `census` for real counts. 5
tests. Remaining (the **wiring**, deferred until the chrome churn settles): render Recent/Saved over
this model in `pane_data::alembic_items` (replacing B1's grounded proxies), surface + edit the policy
(persona settings, not the shellbar-dirty `settings_store`), a promote (keep) action from Recent rows
and demote from Saved, and the eviction pass itself (drop short-term content via `content_store`, the
forgetting Athanor/slice D will later run continuously). The **by-sessions** policy is deferred behind
a per-node last-session stamp the snapshot does not yet carry.

### D — Athanor, the distillation daemon

**Deferred Fleece intent (audited 2026-08-26):** the Alembic reservation does
not depend on Fleece and has no Article-consuming code. A future page engram
should supply the extracted `Article` to Athanor instead of raw page bytes, but
the dependency lands with that consumer rather than as a manifest receipt.

An armillary actor (steady-heat: throttled at idle, yields to foreground): forgetting (evict expired
short-term), consolidation (dedup by `content_hash`, relate version chains, maintain indices), facet
extraction (later), and **proposal emission** (it proposes engrams / GC / facets for the host to
apply; it never mutates graph truth or edits existing engrams — eidetic R0). Live passes surface in
Steward; its knobs in Apparatus config.

Done when Athanor runs in the background, evicts eligible short-term, emits consolidation proposals,
and stays inside R0 (no direct graph-truth mutation).

**Forgetting landed 2026-06-24** (commits `d3893dd` spine, `95f3a20` host wiring): the forgetting pass
in the R0 propose/apply shape. `session-runtime/athanor` is pure pass logic (`propose_forgetting` reads
a snapshot + timing, returns stale short-term urls; `apply_forgetting` drops their cached content via
`content_store::evict_content` → `FjallStore::delete_blob`); `node_ops::run_forgetting_pass` runs it
from the Alembic Recent section's "forget stale recent now" affordance and records the count. Drops
cached content only, never graph truth or engrams. This also closes **slice C's eviction tail** (Recent
evicts per policy). Remaining D: the **steady-heat armillary actor** that schedules this in the
background (vs the manual trigger), the **consolidation / facet** passes (consolidation is light —
graph engrams already dedup by content-addressing), and surfacing the live pass in **Steward** (today
the count lands in the Apparatus diagnostics buffer, not Steward's live-ops view).

**Corrected 2026-10-06 (S14 pass):** the consolidation pass landed 2026-07-01 (the athanor
steady-heat actor plan's P2), and both passes now live in `mere-athanor`
(`ports/distillery/athanor/src/lib.rs`: `propose_forgetting`, `apply_forgetting`,
`propose_consolidation`, `apply_consolidation`; moved in `1bda73d5`). D is not done:
its "runs in the background" condition is unmet, since the crate's module doc says
scheduling the passes "is still to come" and a 2026-09-02 ruling there gives the
scheduler to Djinn; the facet pass and Steward surfacing remain open.

**Reconciled 2026-10-24:** this plan owns D's remaining workshop integration.
The archived steady-heat plan is historical evidence. Section 5 specifies the
first manually invoked actor; automatic cadence and facet extraction remain
later extensions. Current host controls use the workshop's shared view model;
June's Steward/Apparatus placement is historical, not a dependency on retired
panes.

### E — Event log + Timeline (scoped for implementation, decision #5)

**Spun out 2026-07-01** to its own plan:
[event_log_timeline_plan](../../archive_docs/2026-10-06_superseded_plans/2026-07-01_event_log_timeline_plan.md) — read that doc for the current,
code-verified implementation spec (it corrects one finding here: `apply_graph_delta` is not the mutation
chokepoint this section implies; it has 2 real call sites, not a universal funnel). This section stays as
the historical decision record.

**Corrected 2026-10-06 (S14 pass):** that plan was itself superseded for implementation
on 2026-08-03 by the
[graph view curation and interaction plan](2026-08-03_graph_view_curation_and_interaction_plan.md):
`GraphJournal` and its attributed `CapturedDelta` supply the append-only replay
substrate proposed below as a new `GraphMutation` log.

The substrate undo/redo and the Timeline both ride. One append-only log of graph mutations, two
projections (eidetic R0): the **Alembic** current fold (slice C's live nodes / facets) and the
**Timeline** historical replay. This log is local short/long-term memory, not engrams (no
content-addressing, distillation, or federation); E5 ("Timeline to engram") is the one place a chosen
past state crosses into slice A.

**Code reality (verified 2026-06-24).** Exists: `GraphSnapshot` to/from (the checkpoint primitive);
`kernel::persistence::NodeAuditEventKind` (a per-node event taxonomy: TitleChanged / Tagged / Pinned /
Tombstoned / Restored / ..., serde + rkyv) as a partial mutation vocabulary; the **proven
 event-sourcing substrate** in `tessera` ([`gemot/src/tessera/store.rs`](https://github.com/merely-made/gemot/tree/main/crates/gemot/src/tessera/store.rs) *(historical citation)* <!-- doc-audit: historical-link -->,
a `TesseraStore` implementation) and `cable` (`murmuring/src/cable/{log_store,persistent_store}.rs`) to mirror;
the kernel graph mutators (`Graph::add_node` / `add_node_with_id` / `remove_node` + edge / field
methods) as the capture points; the `ViewIntent` sidecar (`session-runtime/view_intent_store.rs`:
`CameraSnapshot` + `HiddenRelationRecord`) as the seed for the view-intent stream;
`SharedNavigationMemory` (`graph/history.rs`), which is per-node **browse** history (where each node
went) and already snapshot-persisted, distinct from this **structural** log (how the graph changed).
Net-new: the `GraphMutation` event type, the persisted append-only log, the recording hook, the
view-intent stream (vs the single current sidecar), undo/redo, and the Timeline replay + scrubber.

**Sub-phases:**

- **E1 (the event type + log store).** A kernel `GraphMutation` enum (NodeSpawned {id, url, pos} /
  NodeRemoved / NodeMoved / EdgeAsserted {from, to, kind} / EdgeRetracted / FieldChanged, plus a
  `MetadataChanged(NodeAuditEventKind)` arm reusing the existing taxonomy), serde + rkyv like the rest
  of `persistence.rs`. A local append-only `GraphMutationLog` mirroring the tessera/cable `log_store`
  shape (append, iterate-from, truncate-after for redo invalidation), persisted as a per-session
  sidecar beside `graph.json`. Local memory, never an eidetic engram (R0).
- **E2 (recording + checkpoints).** Capture each mutation at one chokepoint (wrap the kernel mutators,
  or record at the host apply point so contributions and user gestures both flow through it; pick the
  single seam so nothing is missed). Interleave periodic `GraphSnapshot` checkpoints (the existing
  `session_graph_store::save`): cadence = every N mutations or on idle/close, whichever comes first, so
  replay is "load nearest checkpoint, replay events to T."
- **E3 (replay + the Timeline scrubber).** Fold to the nearest checkpoint, replay forward to time T.
  Surface = an **orrery scrubber / time-axis** (decision #6), not a docked pane; it replays over the
  live orrery (spawns, moves, projections playing out). Read-only: scrubbing never mutates truth.
- **E4 (undo/redo).** The same substrate at single-step granularity: undo reverses the last
  `GraphMutation` (each variant carries its inverse; NodeRemoved keeps the removed node's snapshot for
  re-spawn), redo re-applies; a new mutation after undo truncates the redo tail. Build the log once
  (E1/E2), get both undo/redo and the Timeline.
- **E5 (Timeline to engram).** Scrub to a past incarnation, then `save_graph_engram` that state (slice
  A) to pin it durably and shareably. The one crossing from the local log into the engram store.

**View-intent stream (decision #5 tail, resolved).** Promote the `ViewIntent` sidecar from a single
current state to an append stream of projection changes (camera, arrangement, hidden-relation
toggles), so the Timeline replays *arrangement* too, not only structure. Run it as a **parallel
stream the Timeline composes** (keyed by shared timestamps), not interleaved into the structural log,
so a structural undo and a view change stay independent.

**Open sub-decisions:** checkpoint cadence N (tune by measured replay cost in E3); whether field-layer
and coupling changes are first-class `GraphMutation` arms or a coarse `FieldChanged` (lean coarse
first); how far back replay stays cheap before a checkpoint is mandatory. Large enough that E may spin
to its own plan once C and D land; this section is its implementation spec.

## 3. Open decisions, resolved or deferred

All seven resolved with Mark 2026-06-24 (his calls in **bold ✓**).

| # | Decision | Resolution |
| --- | --- | --- |
| 7 | Graph-engram **redaction** | **✓ Mark: redact private info, granularly opt fields back in.** Built in slice A: `RedactionPolicy` strips thumbnails / favicons / session state (scroll + form drafts) by default, per-field opt-in via flags; structure + addresses + tags + classifications + provenance always kept. |
| 2 | Promote-in-place vs always-distil | **✓ Mark: tagging adds to long-term memory (retained) — promote-in-place.** A tag (or bookmark) keeps the node in long-term, raw retained; not a capture-to-distillate handoff. Lands in slice C. |
| 1 | **Merge semantics** (compose) | **✓ Mark: agreed — merge-by-identity, layer the context** (`import_provenance` is a `Vec`, so source records coexist); exposed as an Athanor per-compose option. Lands with the compose sub-feature (post-A/D); does not gate save/open. |
| 3 | Settings placement | **Mark asked "Apparatus or Steward? sync or async?" → answered:** Athanor splits by the §8 axis — its **config knobs go in Apparatus** (the at-rest plane: dedupe policy, eviction thresholds, steady-heat schedule), its **live passes surface in Steward** (the in-flight plane), its faults in Apparatus diagnostics. No new pane. It runs **async** — an armillary actor off the UI thread, never synchronous on render. Primary settings home = **Apparatus**. |
| 6 | Timeline surface | **✓ Mark: agreed — an orrery scrubber / time-axis**, not a docked pane (slice E). |
| 5 | Event-log shape | **Scoped for implementation (slice E expanded above, 2026-06-24).** A kernel `GraphMutation` event type building on `NodeAuditEventKind`; an append-only log mirroring the proven tessera/cable `LogStore`; checkpoint-interleaved replay (cadence every N mutations or idle/close); the view-intent promoted to a **parallel composed stream**; undo/redo on the same substrate; Timeline = orrery scrubber (#6). E1 type+store → E2 record+checkpoint → E3 replay+scrubber → E4 undo/redo → E5 Timeline-to-engram. |
| 4 | The lora lane | **Spun off 2026-06-24:** [local_models_harness_brief](../research/2026-06-24_local_models_harness_brief.md). The runtime + harness layer the geist brief and the tier-1 research defer: binds the LLM/adapter runtime seam (extend `intel/embed`'s `EmbeddingProvider` to `InferenceProvider` + `AdapterLoader`; Burn-wgpu wasm-reachable, native runtimes behind the seam), the armillary actor harness, the wasm/native split, and a no-training first slice. Architecture stays in the geist brief; marketplace + governance owned elsewhere. |

## 4. Slice A detail (the first build)

1. **Schema** — register `mere.graph-snapshot/v1` via eidetic's `schema_def`; payload = `rkyv(GraphSnapshot)`.
2. **`save_graph_engram(store, graph, redaction) -> Result<ManifestId>`** — `to_snapshot`, apply the
   redaction default (§3 #7), serialize, `Engram::new` with the schema + privacy (default local) +
   provenance + bounds, write through the store (mirror `record_deleted`). Lives where the eidetic
   helpers do, consumed by the host.
3. **`open_engram_as_session(store, id) -> Result<Graph>`** — load by id, `verify_integrity`,
   deserialize, `from_snapshot`. The host opens it as a new graph in the orrery at engram scope.
4. **Host gestures** — a `SaveGraphEngram` + `OpenEngram` registry command (palette / `>` verb), and
   the `>scene`-style wiring. (No pane yet; B adds the browse surface. A is verifiable headless + via
   the command echo: save, restart, open, the graph round-trips.)

Verification: a unit/integration test that `save → (drop) → open` round-trips a `Graph` through a temp
`FjallStore` with `verify_integrity` passing, plus a headed save/reopen once the command lands.

## 5. Workshop reconciliation and first workflow (2026-10-24)

### 5.1 Ownership and dependency boundaries

Alembic is the place to create, grant, run, observe, interrupt and dissolve a
helper. Its shared surfaces present memory and workshop state. Athanor owns
the workshop domain records and orchestration within Distillery, generalizing
its existing proposal-producing furnace passes. Target applications retain
their own truth and effect authority.

| Owner | Contract |
|---|---|
| Alembic | Shared workshop view model, controls and embeddable surfaces; recall remains independently selectable. |
| Athanor | Installed helper definitions/bindings, selected versions, goal and review policy, persisted lifecycle decisions, run journals and proposal orchestration. It uses Servitor's transitions rather than implementing another reducer. |
| Servitor, in Eidetic | Capability algebra, petitions/admission, binding revalidation and pure recorded-event run reduction. Generic procedure validation/evaluation contracts remain in its R2/R3 design. |
| Armillary | Actor/task execution, correlation transport, cancellation and observable joins. It supplies no permission, retry eligibility or external-effect completion. |
| Distillery / ESP | Device policy, compute placement and model execution; optional model calls remain subordinate operations of workshop runs. |
| Djinn | Compose Athanor as a service and supply process lifetime/scheduling. A host may embed the same domain implementation; no host invents a second workshop authority. |
| Eidetic / Chartulary | Storage, attributed graph history and retained artifacts under owner-defined schemas. |
| Source and destination owners | Authorize scoped reads; validate, commit and acknowledge product effects through their existing write paths. |

The host driver lives with Athanor's integration. It records intent, dispatches
admitted work through Armillary and feeds observations to `RunReducer`. The
reducer remains usable by synchronous scenarios without Armillary, Tokio, ESP
or Djinn. Merely sharing a repository would not impose runtime dependencies;
there is no source-home move or dependency repin in this plan reconciliation.
No new generic capability algebra, scheduler or actor runtime is required.

Procedure candidate validation belongs to the shared Servitor contract;
Athanor records the owner's selected revision and authorized adoption decision.
R2/R3 remain later extensions. A procedure never grants authority or supplies
its own evidence of success.

### 5.2 First workflow: an evidence brief from retained passages

**User result:** select two to four retained text passages, choose **Make
evidence brief**, inspect one proposed note with exact quotations and source
references, then explicitly **Add brief to this graph** or **Reject**. This
assembles research evidence without claiming semantic summarization. It uses
already retained bytes with acquisition disabled; no model is needed for the
first acceptance gate.

**Hosts:** Turnstone is the first embedding host because it already consumes
Servitor's run reducer and retained-source selection. Graphshell is the second,
using its retained-content and owned GraphSession paths. Both embed the same
Alembic controls and Athanor workflow implementation, with small product
adapters for selecting sources, admitting operations and committing results.
Two windows of one application do not satisfy this gate.

One versioned helper artifact is installed in each host-owned workspace. Each
installation has its own `ResidentId`, explicit subject/grants, binding
generation, run namespace and destination. Definition identity is shared;
installation identity and authority are not copied implicitly. The fixture
admits identical passage versions separately in both hosts and also includes
host-specific material to detect accidental cross-scope reads. This proves
reuse in two hosts; it does not claim cross-machine migration, shared live
execution or transfer of grants. A single host's cold reopen must preserve
that installation's identity, lifecycle and history.

**Inputs and bounds:** a captured ordered set of owner/session identity,
source/resource ID, immutable version or digest, passage selector and exact
UTF-8 bytes. The fixture uses three plain-text passages, including non-ASCII
text and repeated titles/URLs with different source identities. At most four
passages, 2 KiB each and 8 KiB total, enter the first workflow; oversized or
unreadable selection refuses visibly rather than silently truncating. These
are declared workflow limits, not new application-wide defaults. Read
authority is checked before collection and before resumed collection.

**Body:** a deterministic script or native decision driver validates the
references and emits a Djot note in selection order: source title, exact
quotation and an owner-routed source reference for each passage. It invents
no conclusions, paraphrases or citations. Djot remains the document format;
typed run/proposal metadata carries the identities a display link cannot.
Limits allow one context read, one body evaluation and one proposal emission.
The deterministic fixture supplies three decisions, two tool calls, zero
model tokens, 30,000 ms elapsed time and zero consecutive retryable failures;
measured time includes awaited work. The note body is capped at 16 KiB;
source labels and references have a separate 4 KiB aggregate metadata cap.
Review/apply is a separate bounded operation. These fixture limits and
observed usage appear in run details.

The public acceptance fixture is concrete; these source/version labels are
illustrative shorthand for the digests assigned to its actual retained bytes:

| Selection | Retained source | Exact selected passage |
|---|---|---|
| A, version 1 | Saved capture titled Field notes | A retained copy remains readable when its origin is offline. |
| B, version 2 | Editor clip also titled Field notes | The café note cites version 2 of its source. |
| C, version 1 | Local research note | Acceptance finishes after the destination confirms storage. |

The expected brief has three sections in A/B/C order, each with that title,
the complete quoted passage and the corresponding source/version reference.
B's selector is checked against UTF-8 bytes, including `é`. The source table
and preview make identical titles unambiguous. Changing B to version 3 after
proposal generation must not change B's quoted bytes or route its citation
to version 3. Acceptance creates a new note, leaving A/B/C untouched.

**Authority:** the helper may read only the selected versions and submit one
brief proposal to its owning workshop. It has no source mutation, acquisition,
credential-release or destination-write grant. After review, the human's
currently authorized destination operation creates the accepted note through
the target application's gate. Acceptance does not widen the helper's grants.
The commit retains both the proposing resident/run and the accepting human's
identity; the application determines its native author/provenance encoding.

The example input and operation descriptions here are illustrative contracts,
not implemented wire APIs. The existing app-host action envelope is an ABI,
not a ready-made semantic operation catalog. Both adapters must expose the
same versioned brief operations with checked arguments, scope, preconditions,
effect class and receipts. Palette/keyboard/automation presentations may
differ; all invoke those owners' same validation and commit paths.

### 5.3 Walkthrough and durable outcomes

1. **Install and scope.** The owner selects the helper revision and source
   set in the workshop. Admit its binding using existing stack identity and
   grants; never mint a new trust root from a readable projection. No timer
   or watch is enabled for this slice. One active run per installation is
   allowed; another invocation is refused with the existing run identified.
2. **Capture and propose.** Persist the run header, exact version references
   and accepted intent before dispatch. The worker receives selected bytes,
   not the underlying store. Persist its validated proposal artifact and
   result before exposing **Ready for review**. Worker completion alone does
   not establish proposal durability.
3. **Review.** Show exact inputs, note preview, grants, version, usage,
   refusals and source coverage. Citation activation asks the current source
   owner to open the recorded version; unavailable or unauthorized material
   stays unavailable. An edited source is shown as changed, without replacing
   the quoted version silently. The owner may reject or request a new run.
4. **Accept.** Capture a destination owner/graph and current revision witness;
   revalidate the proposal and human authority. Allocate a stable acceptance
   operation ID and persist intent before dispatch. The target owner creates
   exactly one retained note Resource and its graph presentation, with the
   proposal digest and provenance attached through its own mutation path.
   Both hosts owe this adapter; Turnstone's research-note chooser and
   Graphshell's generic graph editor do not already implement it.
5. **Acknowledge and reopen.** Only the destination's confirmed durable
   receipt yields **Added** and an openable result. An in-memory application,
   released port request or pending save stays pending with its actual
   disposition. Close the host and workshop owner, restart, inspect the run,
   open the brief and activate its exact retained citations with origins off.

The generation run may finish `Completed` when its proposal is durable. The
goal stays `AwaitingReview` until accepted or rejected; these are workshop
dispositions, not extra Servitor terminal variants. Acceptance is a separately
correlated domain operation with its own current authority, bounded deadline
and receipt; it does not append events to the terminal generation reducer.
The destination owns its operation ledger and recovery, rather than
representing the human as the helper's write-capable binding. A completed
generation must never be presented as a completed destination commit.

The destination must durably associate acceptance operation ID, proposal
digest, created resource identity and mutation result. Retry uses the same
operation ID; a different digest under that ID refuses. If that association
and the graph edit cannot share a transaction, the adapter must specify and
qualify its recovery path before this workflow can pass. After an uncertain
write, query that owner for the operation's receipt or reconcile under its
exclusive writer ownership. Do not regenerate the note or submit another
edit. Retain a workshop acceptance disposition of `Reconciling` when neither
completion nor absence can be proved; the completed generation run stays
terminal and unchanged.
Turnstone's `HandoffUntracked` default is insufficient for this path.

The first acceptance decision fixes one destination for the proposal and
durably assigns its operation ID once. Duplicate button activations,
reconnection and reopened review reuse that assignment; they cannot allocate
fresh operation IDs to evade deduplication. Another destination or an edited
brief requires a new explicit proposal/review identity. The fixture exercises
concurrent duplicate acceptance, not only retries from one client.

Pause stops further dispatch and requests cooperative worker shutdown;
Armillary's join reports execution status. Servitor retains uncertainty for
an active run; the destination ledger retains uncertain acceptance. Reopen
preserves explicit pause and revocation. Resume rechecks
binding, grants, source access and destination preconditions. Cancellation
cannot undo an accepted brief; a rejected or superseded proposal cannot later
be applied by a delayed reply. Dissolving a helper denies new work while
retaining unresolved effects and required receipts under the owner's policy.

Private passage copies, proposals and journals follow the selected owner's
sealed storage and lock policy. Lock prevents new reads/effects and clears
worker material as the actual custody contract requires; it cannot erase
uncertain effects. If no qualified sealer is supplied, refuse private inputs
rather than store them in clear. Public synthetic fixtures can qualify run
mechanics independently of production custody. Observations retain bounded
nonsecret references; they do not duplicate prompt/passage bodies.

### 5.4 Records and first implementation boundaries

Use related records rather than a universal work state machine: installed
binding and selected definition, goal/source set, Servitor run header/events,
immutable proposal, acceptance operation/receipt, and product journal entry.
References include owner/session namespace, run/step/attempt/generation and
artifact digests. A model invocation later references its existing Eidetic
`InferenceRunRecord`; a mesh job and Apparatus observation keep their own
identities and meanings. A content hash supplies integrity, not read authority.

Persist actionable version references and effect arguments securely enough
to reconcile without asking a model to reconstruct intent. The raw prompt
is not a reconciliation prerequisite. Retention may remove resolved content
under explicit policy; it must retain the unresolved operation's required
evidence or refuse further work when a configured bound is reached.

Keep shared implementation in the existing Alembic/Athanor homes. Add workshop
view/domain/driver modules when implementing; a new crate is not required.
Reuse Armillary task/lifecycle mechanisms and Servitor `admit`, `revalidate`
and `RunReducer`. Persist explicit attribution through awaited work instead
of carrying a mutable global author across async calls. Existing graph
writers remain sole owners; no second GraphSession writer is opened to apply
a brief. Automatic scheduling, model decisions, procedure learning, MCP/ACP,
remote execution and fleet migration have separate acceptance gates.

### 5.5 Feature targets and acceptance

| Target | Deliverable and done-condition |
|---|---|
| W0 — shared domain and deterministic workflow | Versioned inputs/proposals and persisted binding/run/review records; one reducer implementation, explicit owner clocks/budgets, and exact three-passage output. Plain Servitor consumers retain their current dependency boundary. |
| W1 — Turnstone owner adapter | Normal retained-source selection, shared workshop controls, real target gate/commit and fallible persistence acknowledgement. Accept/reject and citation routing work without acquisition. |
| W2 — Graphshell owner adapter | The identical helper body and workshop view/domain model operate through Graphshell's owner, with independent grants and IDs. No Turnstone workflow implementation or database reader is copied. |
| W3 — recovery and two-host receipt | Both hosts pass the matrix below on pinned source/dependency cuts. Headed evidence proves normal controls and product history; fresh-process evidence proves durable state. Publication and physical fleet behavior are reported separately. |

The mandatory matrix covers each host, not merely a shared unit fixture:

- Exact three-source order, Unicode quotation spans and versioned citations;
  empty, oversized, unauthorized and missing inputs; source title/URL collisions.
- A fourth source outside the grant cannot enter context. Changed versions,
  wrong owner/session and stale destination revision refuse or request review.
- Reject adds no note. Accept produces one durable note with proposing run
  and accepting actor attribution. Duplicate acceptance returns that receipt;
  a changed proposal under the same operation ID refuses.
- Kill before dispatch, after proposal persistence, after target application,
  and before acceptance-receipt persistence. Reopen yields no repeated edit,
  false success or discarded uncertainty; inability to recover blocks acceptance.
- Pause/resume, revoke/dissolve, lock where composed, budget exhaustion and
  delayed/wrong-generation replies. Source/authority changes during awaits
  are revalidated. Cooperative cancellation and observed join remain distinct.
- Cold reopen with source origins unavailable: same installation/run IDs,
  exact proposal/result, pending review or reconciliation, exact local reader
  activation, and unchanged source content. Two applications pass, with one
  shared body and view/domain implementation and their actual product adapters.

Fixtures use synthetic public retained content initially. Additional private
content rows require the supplied custody boundary; model-driven rows require
a real provider receipt and evidence of value beyond deterministic assembly.
No software two-host receipt establishes two physical machines or a general
retrieval-quality benchmark.

### 5.6 Findings (verified 2026-10-24)

- Alembic's [charter](../../../crates/distillery/alembic/README.md) already
  assigns identity/purpose, tools/models, histories, petitions and lifecycle
  controls to the workshop. Its [library](../../../crates/distillery/alembic/src/lib.rs)
  implements recall only; the workshop remains unbuilt.
- [Athanor](../../../ports/distillery/athanor/src/lib.rs) implements proposal
  passes for forgetting, consolidation and retirement. These are reusable
  domain precedents, not evidence of a scheduled workshop service.
- `repos/eidetic/crates/servitor/src/resident.rs` and `run.rs` supply admission
  and pure reduction. Its README records the 98-test promotion gate; that
  historical receipt does not qualify this new workflow.
- `repos/turnstone/src/resident_runs.rs` persists narrow run records and
  defaults to untracked external handoff. Its research-note selection in
  `repos/turnstone/src/app/research_note_arms.rs` binds an actual Knot document;
  it is not the planned retained-brief destination operation.
- Graphshell's [retained-content producer](../../../ports/graphshell/src/retained_content/producer.rs)
  and [Mere host](../../../ports/graphshell/src/mere_host.rs) provide source
  presentation and an owned graph writer. The accepted-note adapter and
  operation receipt remain implementation requirements.
- Armillary's independently selected actor/lifecycle library already supplies
  execution completion. Its API need not absorb Servitor, identity or graph
  dependencies to drive this workflow.

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

## Cross-references

- [Alembic memory + engrams](../technical_architecture/2026-06-09_alembic_memory_and_engrams.md) — the architecture seed this realizes.
- [peripheral panes architecture](../technical_architecture/2026-06-06_peripheral_panes_architecture.md) — the dock the Alembic pane (B) joins; the Apparatus/Steward axis (D's home).
- [memory tiers brief](../research/2026-05-11_memory_tiers_brief.md) — the prior two-tier model the seed refined to three levels (C).
- [statement kernel brief](../technical_architecture/2026-06-19_statement_kernel_brief.md) — short/long-term vs engram framing.
- eidetic ([eidetic-core](https://github.com/merely-made/eidetic/tree/main/crates/eidetic-core) over Muniment's [`fjall` backend](https://github.com/merely-made/eidetic/tree/main/crates/muniment/src/fjall_backend.rs), moved 2026-10-11), [armillary](https://github.com/merely-made/armillary/tree/main/crates/armillary), [graph-kernel snapshot](../../../crates/graph/graph-kernel/src/graph/snapshot/).

## Progress

- **2026-10-24:** reconciled the workshop with Servitor and Armillary, retaining
  Servitor in Eidetic. Specified the retained-passage evidence brief in
  Turnstone and Graphshell, exact owner operations/receipts, separate review
  state, recovery/refusal controls and two-host done-conditions. This is a
  documentation/design change; all W0–W3 implementation gates remain open.
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere
  535bca11, from the D2 record in support/doc-audit/d2/batch_40_s14_phase_b2.md: a
  dated status (A-C landed with host halves retired with meerkat, D half landed in
  `mere-athanor`, merge landed, E superseded), a codicil-rename banner, and slice D's
  ownership left open for this plan's lane. (This log runs newest first.)
- 2026-06-24: **Athanor forgetting + slice C eviction landed** (commits `d3893dd` spine, `95f3a20`
  host wiring). The R0 propose/apply forgetting pass (`athanor` module + `content_store::evict_content`
  over `FjallStore::delete_blob`), run from the Alembic Recent section's "forget stale recent now"
  affordance via `node_ops::run_forgetting_pass`; drops stale short-term cached content only, records
  the count. 3 spine tests + headed-verified (renders, runs, no crash). Closes slice C's eviction tail.
  Remaining D: the steady-heat armillary actor (background scheduling), consolidation/facet passes,
  and Steward live-op surfacing (the count currently lands in Apparatus diagnostics).
- 2026-06-24: **Slice C pane wiring landed** (commit `44a0dc8`, on top of Mark's stable-uncommitted
  tree, hunk-staged). The Alembic Recent/Saved now render the real model (Recent = recently-visited
  *untagged* short-term, Saved = *tagged* long-term with a count) and the eviction policy is visible
  ("evicting recent memory after 30 day(s)"). Correctness fix carried into the spine: `is_promoted` is
  tags-only — `is_pinned` is a physics position-pin, not a memory-keep (B1 conflated them). Headed-
  verified. Promotion works today via the existing Add-tag gesture (tags persist, so Saved is durable
  across sessions). Tail: in-pane one-click promote/demote, the eviction pass (forget stale short-term
  via `content_store`), and an editable (not just visible) policy in persona settings.
- 2026-06-24: **Slice C spine landed** (commit `9cacd41`): `session-runtime/memory_levels`, the pure
  read-model (level classification + eviction policy + evictable computation + census), 5 tests, built
  spine-first because the chrome/pane files (`pane_data`, `render`, `settings_store`, `menus`) are all
  dirty under Mark's concurrent shellbar work. The wiring (Recent/Saved over the model, the policy
  setting in persona settings, a promote/demote action, the eviction pass over `content_store`) is
  deferred until that churn settles. By-sessions policy deferred behind per-node session stamps.
- 2026-06-24: **Decision #4 spun off** to [local_models_harness_brief](../research/2026-06-24_local_models_harness_brief.md)
  (the local-models runtime + harness lane), DOC_README indexed. Code-grounded: `intel/embed`'s
  `EmbeddingProvider` is the live seam precedent the deferred LLM/adapter runtime extends; eidetic
  `ModelManifest` storage + Burn 0.21 (`aether`/`intel`) + the armillary actor framework are the
  substrate; the gap is the runtime binding + harness + wasm/native split, which the brief scopes
  without re-deriving the geist architecture or the compute marketplace.
- 2026-06-24: **Slice B done + event-log (#5) scoped for implementation.** Slice B landed in two
  commits: B1 (`b029d83`) the Alembic pane (Recent / Saved / Engrams, headed-verified listing a real
  engram), B2 (`7c6e5f4`) a clickable engram row thaws into an Orrery pane beside (headed-verified via
  a ShellCommand routing to `open_engram_beside`; ephemeral + read-only). Then expanded slice E into an
  implementation spec (decision #5): code-verified that `GraphMutation` is net-new but `NodeAuditEventKind`
  is a partial vocab, the **tessera/cable `LogStore`** is the event-sourcing substrate to mirror, the
  kernel `Graph` mutators are the capture seam, `ViewIntent` is the sidecar to grow into a parallel
  composed stream, and `SharedNavigationMemory` is browse-history (distinct from the structural log).
  Sub-phases E1–E5; resolved the #5 tail (view-intent = parallel stream) + the cadence/granularity leans.
- 2026-06-24: **All 7 open decisions resolved with Mark** (see §3). Agreed: merge-by-identity (#1),
  promote-in-place — tagging retains into long-term (#2), Timeline = orrery scrubber (#6), redact
  private + granular opt-in (#7, already built in A). Answered #3 (settings): Athanor knobs → Apparatus,
  live passes → Steward, runs async (armillary actor). Two follow-ups Mark requested: **#5** scope the
  event-log for implementation (expand slice E in-plan), and **#4** spin off a dedicated local-models +
  harness design doc (the LoRA / Distillery-trainer lane).
- 2026-06-24: **Slice A landed** (commits `af89808` spine, `4f51175` host gesture + persistence test).
  The freeze/thaw foundation: `session-runtime/graph_engram` over the eidetic typed-payload layer
  (`save_typed` / `load_typed` / `list_typed`), which turned out to be a richer substrate than the plan
  assumed (content-hashed, schema-tagged, integrity-verified, multi-schema — already tested), so the
  binding is a `GraphEngram` newtype + a `RedactionPolicy`, not hand-rolled blob plumbing. Host:
  `Command::SaveGraphEngram` (registry-native — palette, context menu, `>save_graph_engram`). 6 tests,
  incl. a real fjall close+reopen proving "survives restart." Refinement: A delivers the freeze gesture
  together with the `open_engram_as_session` lib primitive; the **thaw-into-the-running-orrery UX**
  (browse, click-to-open) moves to slice B where it has a pane to live in. Chose serde_json over the doc's rkyv (consistent with
  the live `graph.json`, sidesteps the rkyv-from-store alignment gotcha); rkyv compaction stays a noted
  later optimization.
- 2026-06-24: **Plan written; code-verified.** Confirmed the spine is live: `GraphSnapshot` to/from,
  the full `Engram` envelope, and the `eidetic::Store` / `FjallStore` **already opened in meerkat**
  (used for tombstones + the content store), so slice A is a graph-snapshot schema + save/open helpers
  over an existing store, not new infrastructure. Sequenced A (spine) → B (pane) → C (memory levels) →
  D (Athanor) → E (event log / Timeline); resolved the redaction default (#7) that gates A and the
  settings/Timeline-surface decisions (#3, #6), deferred merge / promote-model / event-log / lora to
  their phases. No code yet.
