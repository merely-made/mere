# System Diagnostics and Accessibility Plan

## Current design: shared observations, 2026-09-29

**Status:** design proposal. Semantic-selector integration is a separate active
implementation lane; the diagnostics slices below are not implemented by this
document. The June design and receipts are retained below as historical evidence.

Share bounded observations and causal references across applications, using their
existing state and event producers. Do not introduce a universal application-event
enum, replace domain journals, or rewrite the runtime. The first implementation
priorities are a shared bounded record store and correlated Mesquite receipt
attachments, qualified by two different consumers.

### Name and ownership

**Ruled by Mark, 2026-09-29: Apparatus is the shared diagnostics library, in the
existing `mere-apparatus` package (library `apparatus`).** Reuse that home for
bounded observations and inspection beneath product-owned views. The name and
crate home are settled; the storage contract and adapters below remain unimplemented.
The [September consolidation ruling](2026-09-23_crate_consolidation_plan.md)
requires components to remain modules and reserved homes to receive their actual
capabilities; two consumers must still qualify the shared contract and dependencies.

**Gloss takes the operational overview.** Turnstone's configurable sections can
combine its minimap and recent visits with downloads, background work, sync and
items needing attention. Trail retains browsing history, recall and recovery;
Gloss may compose summaries from those sources. Migrate Steward's useful content
and then retire its separate pane. Operational actions remain product-owned;
the current section activation vocabulary needs extending for controls such as
retry/cancel. This is an accepted direction, not a completed UI migration.

This supersedes the operational-status assignment in the July 18 taxonomy
(`turnstone/design_docs/2026-07-18_meerkat_harvest.md`). The existing Turnstone
Apparatus pane still analyzes selected graph-object facets and handling controls.
**Mark's follow-up assigns that object analysis to Inspector**, alongside its
inspection of documents and content within them, metadata and clipping. Inspector
shows the fields and controls appropriate to the selected subject; graph facets,
provenance and handling controls retain their product-owned sources and write
paths. Migrate those capabilities into Inspector before retiring the object-analysis
Apparatus pane. Application Settings remains separate. Both pane migrations are
pending; these rulings settle their destinations, not their implementation.

Mere's [Apparatus crate](../../../crates/domain/apparatus/src/lib.rs) still emits
empty diagnostic groups, without production consumers. That skeleton is not
evidence of a working shared diagnostics system.
[Alembic](../../../ports/distillery/alembic/src/lib.rs)
already owns Distillery recall/workshop scope; Eidetic owns retained artifacts;
[Armillary](../../../crates/armillary/README.md) owns actor execution and messaging.

Products retain state, permissions, operation completion, persistence, redaction
and actionable controls. Genet owns computed DOM accessibility semantics; hosts
own surface mappings and presentation revisions. Taproot selects/drives those
semantics; Mesquite owns scenario scheduling and aggregate receipts. Registry
owns channel descriptors, configuration and invariant vocabulary. The proposed
shared store observes these owners; it neither dispatches actions nor recomputes
their truth.

### Present supply and consumers

This is a source inventory, not a new test receipt. Cross-repository references
name explicit repository paths; links stay within Mere.

| Owner and source | Existing evidence surface | Shared opportunity and limit |
|---|---|---|
| [UX events](../../../crates/system/ux-events/src/ux_observability.rs), [registry](../../../crates/system/registry/src/diagnostics/emit.rs) | Observers, probes, channel bridge, schemas and trace/message events | Reuse adapters; graph-specific action/node/surface types are not a universal app vocabulary. |
| [Cambium/Mesquite](../../../crates/cambium/mesquite/src/lib.rs), [receipt](../../../crates/cambium/mesquite/src/lane.rs) | Snapshots, string events, captures, failures, frame costs | Add optional typed attachments; CPU wall-time costs are not GPU timings. |
| Turnstone (`turnstone/src/observe.rs`), fanout (`turnstone/src/shell/effects.rs`) | Typed snapshot/AppEvent and 128-entry automation copy | Replace observation storage, preserve domain fold and trail; currently omission is unreported. |
| Knot (`knot-editor/apps/desktop/src/scenario.rs`), snapshot (`knot-editor/apps/desktop/src/workspace.rs`) | Mesquite document/format/dirty/appearance/message snapshot | Document authority and save/site acceptance remain product-owned; the current adapter is a state projection, not a causal outcome stream. |
| Woodshed (`woodshed/crates/woodshed-genet/src/scenario.rs`) | Stage/gesture snapshots, string events, drag metrics | Preserve arrangement and gesture meaning; capture-pending busy is not an operation model. |
| Redshank (`woodshed/ports/redshank/desktop/src/scenario.rs`) | Playback/transcript snapshots, seek requests, worker state | Workers report execution outcomes; desktop/session own stale-result acceptance and model application; persistence owns durable acknowledgment. |
| Cleromancy (`cleromancy/src/ui/native/scenario_driver.rs`) | Consultation/catalog state and durable first/reopen receipts | Preserve consultation authority and acceptance; useful worker-outcome pilot. |
| Mesocosm (`isometry/mesocosm/crates/mesocosm-genet/src/app/bench/probe.rs`), Eponym (`isometry/eponym/crates/eponym-client/src/bin/session/probe.rs`), Isomere host | Mesquite world snapshots, event cursors and cost observations | Keep world replay journals, rules and accepted-game-event history product-owned. |
| Hocket (`woodshed/ports/hocket/crates/hocket-genet/src/scenario.rs`) | Older local scenario/capture loop | Future Mesquite adoption; not a prerequisite for this design. |
| Signalman (`retinue/apps/signalman/src/observation.rs`), retention (`retinue/apps/signalman/src/observation/persistence.rs`) | Admission, source boot/sequence/gaps, bounded disk captures | Reuse limit/loss distinctions; radio schemas, authentication and physical receipts remain local. |

Historical [Graphshell watchdog design](../research/2026-05-17_graphshell_harvest_brief.md)
and Meerkat's bounded `HostObservability` are donors. Meerkat's cache survives in
git at `6f8e6903:crates/meerkat/src/observability/mod.rs`; its host was deleted in
`c5f01064`. Do not describe it as current Turnstone wiring. Current apps inspected
do not install the shared UX observer/registry sender as a stack-wide pipeline.

### Small contract over product payloads

Proposed `Observation<P>` carries schema version, run/source identity, monotonically
increasing record sequence, receipt time, optional source time, optional operation
identity and causal record reference, optional subject reference, and optional semantic
revision/presented-frame reference. `P` remains the producer's versioned, redacted
payload. References are scoped by run/source; sequence numbers are not globally
unique. Wall clocks do not establish causality. Keep source time, source boot,
host receipt time and monotonic elapsed time distinct; unavailable values stay
unavailable. Object, revision and causal links exist only when the producer supplies
them; absent correlation is explicit, never inferred from timing or matching text.
[Armillary correlation](../../../crates/armillary/src/message.rs)
can map into these references without making actors mandatory.

Redshank illustrates why these boundaries matter: its playback worker replaces
a latest-value snapshot and throttles position wakeups
(`woodshed/ports/redshank/playback/src/worker.rs`). `Desktop::poll` accepts or
rejects replies against current product state, while `Persistence::acknowledge`
separately advances durable revision (`woodshed/ports/redshank/desktop/src/main.rs`).
The pilot needs hooks at those actual reply, adjudication and acknowledgment
boundaries. A snapshot-difference adapter cannot recover every outcome; missing
pre-instrumentation coverage is not a counted dropped event. Reuse real request,
load and revision identities; an item ID alone is not an operation ID.

An event records an occurrence. A snapshot describes state at a revision. A
diagnostic is an observation or validation result, not a second authoritative
event journal. A state difference may be recorded as an observed change, but must
not invent a command or cause. Action accepted, operation started, terminal
outcome, state observed, and frame presented are distinct milestones. `act = true`
or `busy = false` proves neither success nor presentation. Rejected, failed,
cancelled and stale-discarded outcomes remain distinguishable. Terminal records
come from the operation owner; generation checks remain there. Stale targets
cannot be retargeted merely because a node identifier was reused.

Registry [descriptors](../../../crates/system/registry/src/diagnostics/descriptor/types.rs)
describe schemas, retention and sampling; they do not enforce buffering or validate
every payload. Its [current invariant matcher](../../../crates/system/registry/src/diagnostics/descriptor/registry.rs)
pairs starts and terminals FIFO per channel, so concurrent out-of-order operations
require correlation-keyed tracking rather than claiming this already exists.

### Bounds, loss and privacy

Configure count, accounted encoded bytes and maximum age at each store/queue
boundary; expose defaults as product settings/policy. Zero count/bytes or zero
retention age disables retention. Reject an oversized record before admission;
evict oldest records until all enabled bounds hold; age uses host monotonic time.
Bound producer payload construction and ingress too: a bounded ring behind an
unbounded sender is not bounded end to end. Define and test accounting for envelope
and payload bytes rather than claiming a precise allocator-memory ceiling.

Expose retained range/bytes plus cumulative rejected, evicted, expired and dropped
counts. Independent cursors or immutable batches prevent competing destructive
drains; a slow cursor receives an explicit gap and next available sequence.
Neither missing records nor an empty disabled store means “nothing happened.”
The existing [RecordingObserver](../../../crates/system/ux-events/src/ux_observability.rs)
and [RecordingChannelSink](../../../crates/system/ux-events/src/ux_diagnostics.rs)
currently append indefinitely at capacity zero; the first slice must repair this
and test it. The registry emitter also silently drops without a receiver and uses
an unbounded sender; adapters must report their actual coverage/loss limits.

Products project and redact before buffering. Turnstone's credential-omitting
authentication observations provide a precedent, not proof that arbitrary URLs, titles or logs
are safe. Diagnostic collection does not enable screenshots, export or durable
capture. Each requires existing explicit product/run policy; secret-bearing raw
inputs must not be captured automatically. Retention, persistence and export are
separate policies. Export failure is visible and fails a requested receipt;
absence is never silently reported as success.

### Sequencing and acceptance

1. **Semantic parity first:** automated fixtures prove Taproot role/name matching
   uses the authoritative Genet computation while explicit text/class selectors
   retain their meanings. Exercise referenced/native labels, hidden/offscreen
   controls, disabled actions and disappeared/stale targets. Then native receipts
   identify the same object/revision across selector, accessibility action and
   diagnostic projection. A human Narrator/VoiceOver/Orca session is a separate
   acceptance level; neither unit tests nor native capture establishes it.
2. **Bounded records:** Turnstone's observation copy and one Mesquite worker app,
   preferably Redshank or Cleromancy, consume the same storage contract. Tests
   cover zero, byte/count/age overflow, oversized payload, slow independent readers,
   reset/new run, sequence gaps and concurrent out-of-order outcomes. Loss must be
   visible in both product inspection and receipts. Signalman informs the contract
   without migrating its evidence format.
3. **Correlated receipts:** attach optional records/loss summary to Mesquite's
   existing outcome, frame ordinal, capture and paint evidence. In two consumers,
   follow action dispatch through a real worker request/outcome to observed state
   and the exact presented frame. Prove rejected, failed, cancelled and stale
   paths, including capture/export failure. Existing product receipts and apps
   without diagnostic attachments remain compatible.

Done means owner-specific code and recorded gates satisfy those slices, with
native and human evidence labeled separately. This pass supplies the design only;
the bounded store and receipt attachment implementation follow later.

### Semantic implementation progress, 2026-09-29

**Status: in progress.** Genet published Taproot's projection-aware matcher at
`19c206873ab08ae227217892d9e74d0df18b349a`; all 25 Taproot library tests passed.
Mere's integration exposes the existing Genet document projection through
`AppCtx`, uses it in Mesquite's role/name selectors, and revalidates a retained
node after scrolling before dispatching its held click. Explicit class/text
matching retains its existing behavior. The shared host fixture compares
selection and accessibility activation of the same referenced-name button and
rejects targets hidden or renamed while a click waits.

Mere's isolated integration passed the standalone Graphshell Wasm check,
60 focused tests and the native smoke
(32 frames, three distinct nonblank captures, AccessKit installed with 15 nodes).
A raw-DOM negative control failed the referenced-name test, then all 21 scenario
tests passed with the exact source restored. Publication and integration with
concurrent Mere work remain pending; a human screen-reader follow-up is still
required. The current projection covers DOM semantics: Sprigging custom-leaf
contributions are subsequently merged by the native accessibility host. Its
revision is currently zero, and current DOM plus retained layout is not an
immutable presented-frame snapshot. Neither full semantic parity nor the later
correlated diagnostics acceptance is established by this slice.

The isolated integration subsequently merged the sealed Rootstock repair
`8eca3e4c` as `ced161f1`. Independent review confirmed both sides' source was
preserved. The combined Rootstock, winit host and Mesquite gate passed 199 tests
with no failures or skips; the native smoke repeated the same three capture
digests. The standalone Wasm check passed against the primary owner's accepted
web-lock baseline, with every package/version retained and only Taproot's new
document-session-api dependency added after mapping the Genet revision. Final
publication still waits for the concurrent main integration, including the
incoming Insigne migration; its broader acceptance is a separate owner's gate.

**Final integration:** the published primary baseline `32edc2ad` was subsequently
merged, preserving the Insigne migration and Knot `855cb75d` adoption. All 199
Rootstock/winit-host/Mesquite tests and the standalone Wasm check passed again.
Both locks retain every incoming package/version after the Genet substitution;
the only added dependency edge is Taproot to document-session-api. This closes
the earlier pending source-integration and machine-verification gates. The
human AT check, custom-leaf parity and diagnostics implementation remain open.
The Apparatus name/crate home and Gloss operational overview are now ruled above;
the Gloss and Inspector pane migrations remain outstanding.

---

## Historical June design and implementation record

The following text records the retired Meerkat host and its June priorities.
Its status, ownership labels, future work and test claims are historical, not the
current plan. Preserve it as evidence; use the September design above for new work.

**Date**: 2026-06-08
**Status**: Planning. Follow-on to the apparatus pane/theme switcher pass.
**Related**: [apparatus pane + runtime theme switcher](2026-06-08_apparatus_pane_and_theme_switcher_plan.md), [frame tree in meerkat](../../archive_docs/2026-06-09_completed_plans/2026-06-08_frame_tree_in_meerkat_plan.md), [peripheral panes architecture](../technical_architecture/2026-06-06_peripheral_panes_architecture.md), [Graphshell harvest brief](../research/2026-05-17_graphshell_harvest_brief.md), [Graphshell docs full harvest](../research/2026-05-27_graphshell_docs_full_harvest.md), [spatial chrome IR brief](../../archive_docs/2026-06-09_pivot_superseded/2026-05-15_spatial_chrome_ir_brief.md), [spatial chrome modular adoption plan](../../archive_docs/2026-06-09_pivot_superseded/2026-05-15_spatial_chrome_modular_adoption_plan.md).

Build one host observability spine for **diagnostics, tracing, UX events,
UxTree/accessibility, probes, and agent harnesses**. The near-term user surface is
the **Apparatus** pane. The long-term consumers are tests, OS accessibility
bridges, and a Hermes/Burn-style agent harness that can drive the application
through typed observations and actions rather than raw pixel puppetry.

---

## Findings

### What is already live enough to use

- `crates/system/ux-events` is the right UX-event seam. It already defines
  observer/probe hooks, apparatus-facing diagnostics bridging, command-surface
  telemetry, and a stable place for user-facing event semantics distinct from
  low-level tracing.
- `registry::diagnostics` (`crates/system/registry/src/diagnostics.rs`) is useful as the channel catalog,
  descriptor/config layer, sampling policy, invariant store, and portable emit
  scaffold. It should configure and classify diagnostics; it should not become
  the live UI state store.
- `crates/forme/uxtree` already produces deterministic accessibility-shaped
  trees and can stitch subtrees into a `TreeUpdate`. This is the bridge between
  Mere's semantic surfaces and AccessKit.
- `crates/shell/frame` *(historical citation)* <!-- doc-audit: historical-path --> now gives meerkat a real pane tree. Its projected leaves
  are the natural unit for diagnostics, focus, accessibility bounds, and agent
  action targets.
- `crates/domain/apparatus` is a useful domain skeleton for an
  accessibility projection, but the current rendered pane belongs in `meerkat`
  like roster/gloss/comms. The domain crate should not be mistaken for a full UI.
- `crates/shell/chrome` still has valuable Graphshell-era shell semantics:
  toolbar, omnibar, command palette, focus authorities, host intents, routing
  hints, and the eventual `project_chrome(state) -> UxTree` direction.
- `registry::input` (`crates/system/registry/src/input.rs`) has useful action/binding/conflict
  vocabulary. The current keymap is in meerkat, but the registry gives Apparatus
  and Settings a future shape for shortcuts, command discoverability, and conflict
  diagnostics.
- The Graphshell harvest docs are useful for policy language: capability
  declaration, non-silent a11y degradation, schema-first diagnostics, watchdog
  invariants, per-frame UxTree snapshots, focus-region state, and probeable UX
  contracts.

### What should not be revived

- The old substrate-as-host plan is not the current plan. Genet/meerkat owns the
  host path now; Graphshell's substrate material is a source of concepts and OS
  plumbing warnings, not a host mandate.
- `register-viewer` / `register-renderer-types` should not become the canonical
  route selector for current meerkat content. The live content route is host
  dispatch plus `inker`/content actors. Renderer-registry events are still useful
  as diagnostic vocabulary once a second engine lane exists.
- The diagnostics channel catalog is too broad to expose wholesale. Wire a small
  live subset first, then let orphan-channel reporting tell us what deserves a
  descriptor.
- Apparatus should not swallow Inspector, Gloss, Steward, or Settings. **Axis
  refined 2026-06-09** (see [Alembic memory + engrams](../technical_architecture/2026-06-09_alembic_memory_and_engrams.md)
  §8): Apparatus is the *at-rest* plane (system diagnostics, the recorded trace read
  after the fact, health snapshot, invariant violations, plus system/subsystem
  config), Steward is the *live* plane (the process/daemon monitor: running actors
  and async jobs you can act on). So live actor lifecycle belongs to Steward;
  Apparatus keeps the recorded actor faults + health snapshot + config. The
  node's graph facets/provenance/lineage belong to Roster at node scope; addressed
  content identity/provenance/parse state belongs to Inspector; graph commentary
  belongs to Gloss.

---

## Architecture

Add a bounded `HostObservability` store owned by `meerkat::App`.

It should collect:

- `DiagnosticRecord`: registry channel, severity, payload summary, span id,
  source, timestamp.
- `TraceRecord`: tracing target/name/level, active span, elapsed time, optional
  actor or pane id.
- `UxRecord`: semantic user event from `ux-events`, including surface, command,
  focus, pane, and result when known.
- `ActorRecord`: lifecycle events for fetch/sync/comms/content/agent actors.
- `ProbeRecord`: UX or invariant probe results, including pass/fail/degraded and
  the surface that failed.
- `A11ySnapshot`: root tree metadata, focus node, node count, missing label
  count, missing bounds count, and per-surface capability state.
- `AgentObservation`: compact, typed state for harnesses: focused surface,
  visible panes, enabled actions, selected graph node, pending diagnostics,
  current modal/palette, and accessible action targets.

Sources:

1. `tracing` subscriber layer for low-level spans/events.
2. `registry::diagnostics::emit` global sender for structured diagnostic events.
3. `ux-events` observers and `UxChannelObserver` for semantic UI events.
4. Meerkat actor inbox drains in `user_event` for actor lifecycle and faults.
5. Frame-tree/layout rebuilds for pane bounds, focus targets, and accessibility
   surface summaries.
6. UxTree projection passes after layout/render state changes.

Sinks:

1. Apparatus pane sections: Overview, Events, Tracing, Probes, Accessibility,
   Agent. (Live **Actors** lifecycle moved to the Steward pane per the 2026-06-09
   axis refinement; Apparatus keeps recorded actor faults, not the live monitor.)
2. Headless tests and smoke probes.
3. OS AccessKit adapter once internal tree quality is stable.
4. Hermes/Burn agent harness API.

The store is an observation cache only. It must never become the authority for
graph truth, frame layout, actor state, command dispatch, or accessibility
semantics.

---

## Initial Channel Set

Start with a small current-channel map rather than the full donor catalog:

- `meerkat.startup.started`, `meerkat.startup.succeeded`,
  `meerkat.startup.failed`
- `meerkat.frame.layout_changed`, `meerkat.frame.pane_summoned`,
  `meerkat.frame.pane_closed`, `meerkat.frame.divider_dragged`
- `meerkat.ui.action_dispatched`, `meerkat.ui.surface_opened`,
  `meerkat.ui.surface_dismissed`, `meerkat.ui.focus_changed`
- `meerkat.theme.activated`, mapped to the existing theme registry language
- `meerkat.actor.fetch.started/succeeded/failed`
- `meerkat.actor.sync.started/succeeded/failed`
- `meerkat.actor.comms.started/succeeded/failed`
- `meerkat.actor.content.respawned`, `meerkat.actor.content.failed`
- `meerkat.a11y.tree_built`, `meerkat.a11y.bounds_missing`,
  `meerkat.a11y.label_missing`, `meerkat.a11y.focus_missing`
- `meerkat.probe.failed`, `meerkat.probe.degraded`
- `meerkat.agent.spawned`, `meerkat.agent.intent_dropped`,
  `meerkat.agent.action_applied`

Use the Graphshell convention that long-running work emits
`started -> succeeded | failed`, with descriptors declaring timeout invariants.

---

## Phases

### D0 - inventory and mapping

- Map current meerkat events, panes, actors, and commands to the initial channel
  set above.
- Identify which donor channels are directly reusable, which need `meerkat.*`
  names, and which stay latent.
- Record accessibility capability state per surface: orrery, workbench, roster,
  apparatus, gloss, comms, modal/palette.

Done when the mapping is small enough to implement without a registry migration
and explicit enough that new channels do not appear ad hoc.

### D1 - HostObservability store and apparatus upgrade

- Add `meerkat/src/observability.rs` with bounded rings and summary counters.
- Feed current apparatus diagnostics from that store instead of hand-building a
  four-row snapshot.
- Expand Apparatus into sections: Overview, Events, Probes, Accessibility, Agent.
  (The live **Actors** view belongs to Steward per the 2026-06-09 refinement;
  Apparatus shows recorded actor faults, not the live monitor.)

Done when the current theme/apparatus pane shows live recent events and actor
state without changing app authority boundaries.

### D2 - UX events and probes

- Wire command palette, pane summon/close, theme switching, roster selection,
  frame divider drag, destructive confirmations, and focus changes through
  `ux-events`.
- Add `UxProbe`s for the first real risks: modal overlap, missing destructive
  confirmation, focus lost after pane close, and unavailable action shown as
  enabled.
- Bridge UX events into diagnostics through `UxChannelObserver`.

Done when common interactions produce both semantic UX records and diagnostic
records, and at least two probes fail in tests when the guard is deliberately
broken.

### D3 - diagnostics registry bridge

- Instantiate a diagnostics registry/config in meerkat and register the initial
  channel set.
- Install the global diagnostic sender and route emitted records into
  `HostObservability`.
- Surface descriptor state, sampling, orphan channels, and invariant violations
  in Apparatus.

Done when a missing `succeeded/failed` event after a `started` event becomes a
  visible invariant warning.

### D4 - tracing ring

- Add a tracing layer that records selected `meerkat`, actor, frame, render,
  diagnostics, and a11y spans into the bounded trace ring.
- Keep verbose targets opt-in and sampled.
- Link trace records to diagnostic records when span ids are available.

Done when Apparatus can answer "what just happened?" for a pane action or actor
failure without requiring terminal logs.

### D5 - UxTree snapshot and internal a11y audit

- Build one current app UxTree snapshot by stitching chrome/frame/pane/content
  roots after layout.
- Ensure every frame leaf has stable id, role, label, bounds, focusability, and
  enabled action metadata.
- Add an internal a11y audit pass: missing label, missing bounds, duplicate id,
  invisible focused node, action without command route.

Done when Apparatus can show tree health and focused node state, and tests can
  assert the tree contains root/window/frame/pane/content nodes with stable ids.

### D6 - OS AccessKit bridge

- Only after D5, emit AccessKit `TreeUpdate`s for the host window.
- Wire focus changes from app state to AccessKit focus.
- Preserve non-silent degradation: surfaces that cannot emit meaningful a11y must
  declare degraded/unavailable with an owner and exit criterion.

Done when a Windows screen reader can traverse the shell/panes at least as named
  regions, and unavailable surfaces are reported explicitly rather than silently.

### D7 - agent harness

- Define `AgentObservation` and `AgentAction` as typed data, not screenshot
  prompts: open pane, invoke command, select node, activate focused action,
  set theme, drag divider by semantic target, request content preview.
- Expose a harness loop that can read an observation, apply one action, and
  receive the resulting observation plus diagnostics.
- Let Hermes/Burn or a Gemma-class model sit behind that typed loop.

Done when a test agent can open Apparatus, switch themes, summon Roster, select a
  node, and report any a11y/probe failures without coordinate scripting.

### D8 - inspector/steward split

- Move content-level provenance, trust, parse diagnostics, and document structure
  into Inspector.
- Move async operation management and retries into Steward.
- Keep Apparatus as the aggregate system trace and health pane.

Done when Apparatus remains legible under real browsing load instead of becoming
  an undifferentiated debug dump.

---

## Agent Readiness Answer

A Gemma-class model could plausibly puppet the application **after D5-D7**, if
the harness exposes a typed observation/action protocol. It should not be asked
to operate the app through raw pixels first. The model needs:

- a compact tree of visible surfaces and enabled actions;
- stable ids for panes, commands, nodes, and focus targets;
- recent diagnostics and probe failures;
- clear action results, including blocked/degraded reasons;
- a small action vocabulary with no hidden side effects.

Without that, it will be brittle: it may click the right pixels in a demo, but it
will not understand pane ownership, disabled commands, focus traps, or degraded
accessibility states.

---

## First Slice

1. Add `meerkat/src/observability.rs` with bounded rings for diagnostics, UX
   events, traces, probes, actors, and a11y summaries.
2. Replace `apparatus_diagnostics()` with an observability snapshot and render
   recent events in the Apparatus pane.
3. Emit `ux-events` for theme switch, apparatus summon, roster summon, pane close,
   frame divider drag, and focus changes.
4. Register the first `meerkat.*` diagnostic descriptors and install the sender.
5. Add tests for:
   - theme switch emits UX + diagnostic records;
   - apparatus summon/close preserves focus;
   - missing label/bounds appears in the a11y audit;
   - `started` without terminal event trips an invariant.

---

## Progress

- 2026-06-08: Plan written. Grounded in the live `meerkat`, `ux-events`,
  `registry::diagnostics`, `uxtree`, `frame`, `chrome`, `registry::input`, and
  Graphshell harvest docs. No code yet.
- 2026-06-08: **D1/D2 seed landed.** Added `meerkat::observability` as a bounded
  host-local observation cache, expanded the shared `SurfaceId` vocabulary for
  Roster / Gloss / Apparatus / Comms / Workbench panes, and wired pane
  open/close events through `ux-events` into diagnostic records. Apparatus now
  renders Overview, UX Events, Actors, Accessibility, Diagnostics, and Probes
  sections from the snapshot. Actor events are recorded from the kernel inbox
  drain for fetch, sync, content respawns, and comms updates. A coarse a11y
  summary records visible surfaces and explicitly marks the OS AccessKit bridge
  degraded until the real bridge lands. Full `registry::diagnostics` descriptor
  registry, tracing layer, and probe execution remain D3/D4 follow-ons.
- 2026-06-08: **D3 landed + D4 seed.** `HostObservability` now owns a local
  `DiagnosticsRegistry`, registers the first `meerkat.*` and pane UX channel
  descriptors, and runs every diagnostic through registry sampling/orphan
  tracking/invariant observation before it enters the Apparatus ring. Startup,
  fetch, and comms use the `started -> succeeded | failed` convention, with
  completion invariants registered for startup/fetch/comms. Apparatus now shows a
  Registry section (registered count, orphan channels, invariant violations) and
  a Tracing section. `meerkat` installs the portable
  `register_diagnostics::emit` sender and drains `DiagnosticEvent`s into the same
  cache, so extracted registries can feed Apparatus without depending on meerkat.
  Remaining D4 work is a real `tracing` subscriber layer over host spans.
- 2026-06-08: **D4 landed + D5 internal seed.** `meerkat` now installs a
  `tracing-subscriber` layer that mirrors selected `meerkat`, `frame`, and
  `uxtree` spans/events into the same portable diagnostics receiver used by
  `register_diagnostics::emit`, so Apparatus can show recent host trace activity
  without tailing terminal logs. The a11y refresh now builds an internal
  AccessKit-shaped `uxtree` snapshot by stitching a host window root, a chrome
  root, and the live `frame::project_frame` subtree. Apparatus reports tree root,
  focus node, node count, missing label/description count, missing bounds count,
  duplicate ids, and audit findings. The OS bridge remains explicitly degraded;
  the next D5/D6 work is attaching real bounds/content subtrees and then pushing
  `TreeUpdate`s through an AccessKit platform adapter.
- 2026-06-08: **D5 bounds/content slice landed.** The internal a11y snapshot now
  uses `frame::project_frame_with` to attach available domain subtrees under
  frame leaves: `mere-orrery` for the graph pane, `workbench` for the tiled
  workbench, the apparatus skeleton for Apparatus/System panes, and stable
  generic roots for Roster/Gloss/Comms/Tile/Custom panes. Meerkat stamps the
  host-computed frame root, pane leaf, and pane-content-root bounds into the
  AccessKit-shaped tree after projection, preserving the frame crate's
  geometry-free ownership. Focus now resolves to the active frame leaf when
  chrome does not own focus. Unit tests cover frame leaf stable ids, host bounds
  attachment, and a11y audit focus/bounds failures. Remaining D5 work is richer
  descendant bounds and pane-specific subtrees for Roster/Comms/Gloss content.
- 2026-06-08: **D5 pane subtree slice landed.** Roster, Gloss, and Comms now
  project pane-specific internal `UxTree` subtrees instead of generic content
  roots. Roster exposes member rows as list items and reuses row hit-test bounds
  when the host has rendered them; Gloss exposes graph nodes as link-like items
  with URL values and focused-node state; Comms exposes conversation list items,
  selected thread messages, and the draft text input. This gives Apparatus and
  harness code a real semantic surface to inspect before any OS AccessKit bridge
  is enabled. Remaining D5/D6 work is making descendant bounds stable across the
  first render for every pane type, then emitting platform `TreeUpdate`s through
  an AccessKit adapter.
- 2026-06-09: **D6 Windows AccessKit bridge landed.** Meerkat now factors the
  internal a11y projection into a reusable `TreeUpdate` path and installs a
  Windows `accesskit_windows::SubclassingAdapter` before the winit window is
  shown. The same stitched `uxtree` snapshot feeds Apparatus and the OS bridge;
  render, resize, and focus changes refresh the tree and push updates when the
  adapter is active. Non-Windows platforms remain explicitly degraded for this
  slice rather than silently pretending to have an OS bridge. Remaining D6 work
  is richer action handling, platform bridges beyond Windows, and visual/manual
  screen-reader verification.
- 2026-06-09: **D7 typed agent harness landed.** Meerkat now has a feature-gated
  in-process `agent_harness` module with typed `AgentObservation`,
  `AgentAction`, and one-step `AgentStep` results. The harness reads visible
  frame surfaces, active theme, focused node, enabled action descriptors,
  diagnostics/probes, and the same a11y snapshot Apparatus sees, then routes
  actions through existing host methods for opening panes, switching themes,
  invoking commands, selecting nodes by URL, and requesting focused-node
  previews. Blocked actions emit `meerkat.agent.intent_dropped`; applied actions
  emit `meerkat.agent.action_applied`. The first tests cover opening Apparatus,
  switching themes, summoning Roster/Comms, selecting nodes, and reporting
  blocked selections without coordinate scripting. Remaining D7 work is a stable
  external transport for Hermes/Burn, richer semantic divider targets, and a
  narrower public schema once the action vocabulary settles.
- 2026-06-09: **D6 AccessKit action routing landed.** The Windows bridge now
  queues `ActionRequest`s and wakes the winit loop instead of dropping them.
  Meerkat drains those requests on the kernel thread, resolves target node ids
  through a host-owned action route table generated with the a11y tree, and maps
  graph/gloss links plus roster rows to semantic node selection. Routed
  `Click`/`Focus` actions record `meerkat.agent.action_applied`; unsupported or
  stale target ids record `meerkat.agent.intent_dropped`. Remaining D6 work is
  platform bridges beyond Windows and real manual screen-reader verification.
- 2026-06-09: **D6 non-Windows bridge scaffolding and verification checklist
  landed.** Meerkat now depends on the workspace `accesskit_macos` and
  `accesskit_unix` adapters and installs the macOS AppKit subclass adapter or
  Unix AT-SPI adapter behind the same host-local snapshot/action queue used on
  Windows. Focus updates are forwarded through the platform bridge where the
  adapter exposes that hook. The manual screen-reader pass is captured in
  [2026-06-09_accesskit_screen_reader_verification.md](../../archive_docs/2026-09-02_retired_plans/2026-06-09_accesskit_screen_reader_verification.md):
  Narrator, VoiceOver, and Orca still need real local runs before D6 can be
  called screen-reader verified.
- 2026-06-09: **D8 pane split seed landed.** `frame::PaneContent` now has
  first-class `Inspector` and `Steward` variants, with matching UX surface ids,
  diagnostics channels, AccessKit descriptors, command-palette verbs, agent
  actions, a11y projection labels, and simple rendered placeholder panes.
  Inspector currently summarizes the focused node, node count, and content
  state; Steward summarizes active actors, sync chip state, and tab cap.
  Remaining D8 work is moving real content provenance/trust/parse/document
  structure into Inspector and real async operation retry/cancel/pin controls
  into Steward while keeping Apparatus as the at-rest aggregate trace/config pane.
- 2026-06-09: **D8 Inspector content + Steward operation pass landed.**
  Inspector now reads the focused graph node identity, durable import
  provenance, classifications, pinned/compat/viewer metadata, fetch state,
  content type/body size, parser lane, trust/provenance, parse diagnostics,
  outgoing links, and document-structure counts. Nematic-routed content is
  inspected through `EngineDocument`; HTML reports a Genet lane structure
  summary until Genet exposes a full document semantic tree. Steward now lists
  live content operations from the constellation and exposes host-routed retry,
  stop, and background-pin hooks through command-palette verbs and typed agent
  actions. Remaining D8 work is making those hooks clickable in the Steward pane
  UI itself, adding richer per-operation history/error affordances, and migrating
  node facets / lineage out of the focused-node Inspector into the Roster's
  node-scoped facet view.
