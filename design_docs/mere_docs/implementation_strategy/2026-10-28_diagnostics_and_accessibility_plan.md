# Diagnostics, observations and accessibility

**Status:** 2026-10-28: cut from `2026-06-08_system_diagnostics_and_accessibility_plan.md` and from the apparatus tail of `2026-09-23_crate_consolidation_plan.md`, archived in `archive_docs/2026-10-28/`. The September design is implemented and machine-qualified within stated limits: the bounded observation store, the optional Mesquite attachment, Turnstone's Gloss and Inspector migration, the Redshank and Knot worker pilots, shared selectors, current-target admission, producer identities, child, leaf and member lifetimes, native ID remapping, publication outcomes, cache membership and delayed host results. Open: owner-issued target lifetimes in deeper producers, provider provenance, authored-source lifetimes, guest ingress and native install recovery, a supplier-qualified semantic model with forward geometry, correlated action, worker and frame receipts, product collectors, the Inspector and Gloss gates, and the apparatus row of crate consolidation. The June design, built around the retired Meerkat host (D0 to D8), is history and is not carried.

**For:** sharing bounded observations and causal references across applications over their existing state and event producers, without a universal application-event enum, a replacement for domain journals or a runtime rewrite; and making semantics, actions and publication trustworthy for automation and assistive technology. Mark ruled on 2026-09-29 that Apparatus is the shared diagnostics library, that Gloss takes the operational overview and that Inspector takes object analysis.
Not here: human Narrator, VoiceOver and Orca runs and the human-accessibility checklist, headed AT and physical delivery, browser-runtime acceptance (a Wasm compile is not acceptance), native bridge acceptance on a real platform, the physical Narrator tear-out receipt, and the headed failure-and-recovery gate that must precede any native retry policy, all `2026-10-28_platform_qualification_plan.md`; the Hest plan's native, browser and AT acceptance residue, also platform qualification (its broader command declaration, invocation and result consolidation stays a question below, since that plan is complete and archived); boundary consolidation (execution and shutdown, the shared observation and semantic-action follow-up, the separate Castellan process design), `2026-10-28_family_promotion_plan.md`; custody-process gates, `2026-10-28_custody_plan.md`; fetch actor adoption, `2026-10-28_smolweb_fidelity_plan.md`; contributed-pane semantic automation and its scoped retained-layout and admission-generation work, Turnstone's pane registry plan (`turnstone/design_docs/2026-08-08_pane_registry_and_graph_panes_plan.md`); the meaning of link, field and assertion lifetimes, `2026-10-28_graph_semantics_plan.md`, `2026-10-28_link_model_plan.md` and `2026-10-28_graph_view_curation_plan.md`; the dramatis facade and the version baseline of crate consolidation, `2026-10-28_workspace_hygiene_plan.md`. The June design's D7 external agent transport for Hermes and Burn was never built and is not carried.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

Line numbers refer to `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md`.

- Mark, 2026-09-29: Apparatus is the shared diagnostics library; Gloss takes the operational overview (default minimap plus Downloads, configurable sections, read-only downloads for now; Steward's runtime pane retired); Inspector takes object analysis, content inside documents, metadata and clipping, and those capabilities migrate into Inspector before the object-analysis Apparatus pane retires; application Settings stays separate (lines 1480-1509).
- Components remain modules and reserved homes receive their actual capabilities; Apparatus remains a library (lines 1485-1488, 2326-2335).
- Products keep state, permissions, operation completion, persistence, redaction and actionable controls; Genet owns computed DOM accessibility semantics; hosts own surface mappings and presentation revisions; Taproot selects and drives; Mesquite owns scenario scheduling and aggregate receipts; Registry owns channel descriptors, configuration and invariant vocabulary. The shared store observes these owners and neither dispatches actions nor recomputes their truth (lines 1472-1476, 1519-1525).
- An observation carries schema version, run and source identity, a monotonic record sequence, receipt time, and optional source time, operation identity, causal and subject references and semantic revision or presented-frame reference; references are scoped by run and source; wall clocks do not establish causality; absent correlation is explicit, never inferred from timing or text. Action accepted, operation started, terminal outcome, state observed and frame presented are distinct milestones, and `act = true` or `busy = false` proves neither success nor presentation (lines 1551-1590).
- Bounds, loss and privacy: configure count, accounted encoded bytes and age at each store and queue; zero disables retention; reject an oversized record; evict oldest; independent cursors; gaps and loss are explicit, and an empty disabled store never means nothing happened; products project and redact before buffering; collection never turns on screenshots, export or durable capture; export failure is visible (lines 1592-1621).
- Acceptance levels stay separate: machine, native and human. Dispatch is not terminal success; a compiled Wasm host is not browser-runtime acceptance; native `submitted` and browser `mirrored` are not consumed or announced; `BridgeStatus` describes physical installation, not acceptance; queue and readback are not compositor or display acknowledgement; human participation cannot be inferred from automation (lines 229-241, 1799-1800, 1908-1910).
- Target lifetimes are never allocated from observation reads, and a frame epoch, an observed-description counter or a hand-rewritten subset of AccessKit node references is never substituted for owner-issued lives; a stale target cannot be retargeted because a node identifier was reused (lines 459-460, 678-680, 1582-1583). AccessKit owns typed node-reference traversal; Mere keeps no second list of relation properties (lines 281-285, 377-380).

## Phases

### M1. Custom and deeper producer ownership of target lifetimes
Bounded repairs are in: child tokens, member witnesses, queued owner and registration lifetimes, leaf and child lifetimes, and checked activation under a directly owned borrow. The latest lists leave custom and deeper producers: checked activation and action-ID reuse or reassignment in every custom and deeper shared producer (a trait default cannot make arbitrary interior state atomic), activation and child or action lifetime validation made atomic at the model owner, and legacy producers that still depend on authored nonreuse because an optional missing child token is not a lifetime.
- Done when: old requests are refused at their actual receivers across document, window, provider, child and leaf replacement while current requests and ordinary republication stay valid; tokens are chosen after auditing actual caller replacement and update patterns.
- Needs: M4 for native and browser ingress.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:43-45,135-145,181-199,243-258,333-337,674-681,1284-1285`

### M2. Provider provenance for delayed host results
Kernel input keys, and in-tree layout and StopReturn installation, are received. External raw-position and content or affinity providers still need dispatch-to-install request provenance, including same-member supersession and recipe, viewport and policy inputs; a direct-edit API is not a delayed-result lease. Graphshell app and session completion, and each fetch or provider request, are to be traced through exact owner and head admission, backend outcome and projection install. Broader provider request and policy admission is a separate migration.
- Done when: the provenance is carried at each external receiver, and each fetch or provider request traces through owner and head admission to projection install; the parallel fetch-actor work is qualified separately.
- Needs: fetch actor adoption (`2026-10-28_smolweb_fidelity_plan.md`).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:33-40,79-86`

### M3. Authored sources: lifetimes, rollback and checked transactions
Surface lifetimes do not settle source-specific ones. Link, field and assertion owners and authored curation (Fold and Forme rules) must establish their own replacement, rollback and replay rules; kernel transactions must be checked on install, and durable and history intake continue.
- Done when: link, field and assertion lifetimes, authored Fold and Forme rules, durable history and rollback, and checked transaction installation are received at their owners.
- Needs: the graph semantics, link model and graph view curation plans, which own the meanings.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:41-42,87-90`

### M4. Native publication: the AccessKit hook, guest ingress and install states
Native ROOT publication and ID remapping are implemented. The AccessKit hook is a local pinned 0.25.1 patch with upstream provenance under `support/patches/accesskit`, and Cargo patch tables do not propagate, so a standalone native consumer must declare the same patch at its own root. Open: promoting the AccessKit traversal API and coordinating patch and lock intake before external pins advance; guest compositor namespaces, grafts and trees (the publisher is ROOT-only and refuses them), install, reset and replay, and `SetTextSelection` ingress; `attach` refuses a different window handle and is idempotent for the same one, but has no explicit replacement protocol, and install success, permanent unsupported status, bounded recovery and window replacement must stay distinct states; separating adapter installation from OS traversal for the case where VoiceOver cannot enter the window tree; and Rootstock's intrinsic sizing and Path-B raster for custom leaves (`Leaf::measure` and `layout_dirty` are not received, and `RenderedLeaves::scenes` has only test consumers).
- Done when: each of these is received with its own qualification path; the headed halves (old platform events, partial-install recovery and a retry policy on a real failure) belong to platform qualification.
- Needs: M6 for external pins; platform qualification for headed receipts.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:142-145,187-199,243-258,316-337,382-385,498-503,599-603`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:236-239`

### M5. A supplier-qualified semantic model and forward geometry
Contributions are lowered once in shared composition, but the contributor vocabulary stays AccessKit-shaped, callbacks are seeded differently across paths (context-sensitive equivalence is unqualified), ARIA maps only known roles (arbitrary leaf roles, clipping and pixel parity are unqualified), and separate reads are not an atomic read and act transaction. Geometry uses Genet's content origin only: forward transforms, path clipping, scroll and shadow or slot ancestor placement need one supplier-owned mapping in Genet, and percentage padding is inherited. The advertised action set is routed only for Click, Focus and numeric SetValue; Increment, Decrement and ScrollIntoView are not routed.
- Done when: contributions live in one supplier-qualified model that preserves geometry, clipping, disabled and read-only behaviour, shadow trees and the real native and browser receivers; Genet supplies forward mapping (flattened paint-scope ancestry, transforms, scroll, explicit clipping) and native and browser lowering qualifies real producer pixels and input against semantic bounds; the wider action set routes through its owner with focus distinct from activation and routing acceptance distinct from terminal effects, and declined requests never claim success.
- Needs: Genet's forward geometry; an owner for broader action routing (see Open questions).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:757-774,881-883,1018-1049,1286-1293`

### M6. Supplier promotion and external intake
Sprigging 0.3.0 and Cambium 0.6.0 are prepared source versions, not registry publications. External product adoption (Woodshed's older Leaf callback among it), publication, and receiving against existing external pins are separate gates, and none is inferred from shared-host qualification.
- Done when: not stated beyond the separate gates.
- Needs: the release lane of `2026-10-28_workspace_hygiene_plan.md` for any publication.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:142-143,730-739,757-761`

### M7. Correlated receipts: action, worker and exact presented frame
Two complete action, worker and pixel causal receipts remain a later gate. Turnstone's custom compositor captures composed layer pixels with no sealed product or frame token: it needs an immutable reading at the real render boundary, a checked run, request and frame identity through readback, and deliberate failure, missing or mismatched token and delayed-readback controls; a failed requested capture must propagate into the scenario result, and `capture_composed` today computes success without applying it. Owner-reported terminal actor outcomes are to be received through their own qualified paths. Other worker families, cancelled and stale storage paths, exact frame correlation, built-in, custom and document surfaces, and custom-leaf semantic parity remain; whole-UI causal and revision authority is unavailable, and `sampled_at_lane_frame` names the observation point, not the captured pixel frame.
- Done when: in two consumers, action dispatch is followed through a real worker request and outcome to observed state and the exact presented frame; rejected, failed, cancelled and stale paths are proved, including capture and export failure; existing product receipts and apps without diagnostic attachments remain compatible.
- Needs: the product owners' own stale, cancellation and durability adjudication.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:144-145,198-199,1488,1638-1647,1666-1669,1703-1705,1775-1777,1899-1906`

### M8. Product collectors and installed resident diagnostics
The collector bridge and the resident lifecycle source are callable and fixture-wired, with Djinn's injected sender the one real caller. Products still choose redaction, sink lifetime and thread routing, drain scheduling, registry policy and invariants, and final-counter custody; a process-global OnceLock is not a per-window collector; projection callbacks must not panic. The lifecycle source covers only named resident events (unlock subevents, inference and workers, arbitrary tracing and mod-loader emitters are excluded), serializes with event-file IO, and keeps history and final stopped or disconnect evidence in memory only, so installed-process, abrupt-termination and final-receipt custody are unreceived. Registry catalog growth and producer allocation lie outside the bounded ingress contract, and installing a product pipeline, adding other real workers and supplying missing operation or semantic references need separate owner gates. Inference observations leave run identity, limits and export policy to consumers and do not bound the run table or token transport.
- Done when: not stated; the items are named open.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:1164-1172,1294-1300,1371,1398-1405,1612-1613,1896-1898`

### M9. Selector, input and invariant defects at their owners
Mesquite restates Taproot's private selector grammar in `wheel::parse_selector`, and key and input-text verbs take the narrower runner route (A961); product `act` and click booleans are dispatch-handling facts, not unique-target or terminal-success receipts; the Apparatus inspection skeleton and the real inspection coexist (A873); presentation ordinals still need their run, host and context scope. `descriptor::registry::legacy_violations` maps an invariant clock error to an empty violation vector, which must not be read as a clean inspection; the keyed API exposes the typed error and the legacy FIFO and public caller contract should carry it (A1155; no runtime reproduction claimed). The inference table assumes its mutex is never poisoned while calling owner journal callbacks, and a panicking journal can break that; panic and durable-recovery controls are owed (source candidate), as is journal recovery.
- Done when: each defect is repaired or ruled out at its owner with a regression.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:1173-1176,1301-1303,1390-1396,1407-1420`

### M10. Inspector and Gloss gates in Turnstone
Inspector's viewer and handling controls must supply semantic identity, retained bounds and enabled actions to automation and AT, rejecting old requests when the followed member switches or its source is replaced; rows without a live member expose disabled, no-action state. Viewer overrides write a member-keyed sidecar but find the live URL in the active legacy graph, so a cross-graph follower needs its own runtime lookup and respawn check. Inspector keyboard delivery is absent and needs real runner focus and key plumbing. Trail and active Gloss controls follow under their own owners; custom paint interiors need meaningful child projections beyond Sprigging's single-node fallback; Chrome menus, forms and review surfaces need action-route and focus checks; live documents need Turnstone to consume document-session-api's neutral projections and revision-guarded nonpointer actions, with within-document selection inspection. Gloss background and sync sections and retry or cancel actions are future extensions with their own gates (the downloads pane is a read-only projection, so Steward's per-row controls remain unbuilt).
- Done when: the controls pass composed-tree tests (producer IDs, retained bounds, clipping, read-only roles) and native restart checks, and each unavailable capability stays explicit.
- Needs: Turnstone's pane registry plan for contributed-pane automation.
- Rulings: Mark, 2026-09-29 (Gloss, Inspector).
- From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:1496-1497,1508-1509,1859-1894`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:253-262`

### M11. Tracing reach and quality
A tracing layer must never panic and per-span extension writes must be idempotent, a lesson worth a workspace note. In-tree engine and graph-kernel spans (T3), correlation (T4), and registry sampling with error-chain capture (T5) are re-scopable against Turnstone and Djinn; the armillary and registry-diagnostics halves survive (`spawn_named`, at `crates/armillary/src/actor.rs:128` in the Armillary repository).
- Done when: not stated.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:317-322`

### M12. The Apparatus row of crate consolidation
Crate consolidation's C2 table lists `mere-apparatus` with exact presented frame, broader worker qualification and human assistive-technology qualification still open, and C2's done-when (every row landed, ruled out by Mark, or found to have nothing misplaced, each landed move carrying its tests) is not met until this row and the dramatis row close. The first two pieces are M7; the human AT piece is platform qualification's. The dramatis row is the workspace hygiene plan's.
- Done when: the Apparatus row is landed, ruled out by Mark, or found to have nothing misplaced, with its tests.
- Needs: M7; platform qualification for the AT piece.
- Rulings: Mark, 2026-09-23 (reserved homes receive their capabilities; components are modules).
- From: `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:58,60-61`

## Open questions

- Which 2026-10-21 target-life gaps survive the later repairs? The earlier lists name originating native and browser requests across document, window and provider replacement, removed-and-reintroduced child and leaf keys, and a native initial-install branch that sets its installed guard after an error. The 2026-10-22 and 2026-10-23 sections call those lists historical for their bounded repairs (host-owner binding, late native IDs refused, failed first installation never commits the draft and leaves the installed flag false) without saying what is left, so the two accounts of install-failure state disagree. From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:306-312,445-460,567-608,662-687`
- Is Clip in or out of Inspector's first semantic route? `InspectorIntent::ClipToKnot` carries no member and both pointer drains call the globally focused-document helper, while availability is checked against the followed member. The choice is to bind the intent to its displayed member and revalidate at dispatch, or to leave Clip out; the old text says Mark "has been asked" and records no answer. From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:1876-1882`
- Where does Apparatus live? The ruling and the C2 table name the existing `mere-apparatus` package (library `apparatus`), the plan also says the Mere facade was removed and Registry owns the opt-in collector, and the tree has no `mere-apparatus` crate: `apparatus-core` is a git dependency on Armillary. From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:1480-1484,1511-1514`; `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:58`
- Who routes the wider action set now? The old plan sent broader command declaration, invocation and result consolidation to the Hest plan, which is complete and archived with its acceptance residue going to platform qualification, while Increment, Decrement and ScrollIntoView remain unrouted. From: `archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:766-769,1031-1034,1206-1207`
- Should per-table stats and a canned regression log be re-scoped into `crates/domain/apparatus`, or stay retired with meerkat, with `GraphJournal` and the graph view curation plan as successors? From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1067-1070`
- Should the settings overlay fold into Apparatus (the old A3), or be dropped because no host asked? From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:847-851`

## Findings

- 2026-10-28 (observed in the tree): there is no `mere-apparatus` crate and no `crates/domain/apparatus`; `apparatus-core` is pinned from the Armillary repository (`Cargo.toml:477`, `:914`) and used by Registry as an optional dependency (`crates/system/registry/Cargo.toml:26`), by Mesquite (`crates/cambium/mesquite/Cargo.toml:36`), and by `ports/distillery` and `ports/djinn`.
- 2026-10-23 (carried): `crates/cambium/cambium-winit-a11y/src/publication.rs` owns native IDs; Rootstock (`crates/cambium/cambium-rootstock`) owns authored identity; AccessKit owns typed reference traversal; the Genet bridge owns installation and replay. `a11y_projection` deliberately remains the DOM seed and current consumers use `semantic_projection` (`archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:299-304,1024-1026`).
- 2026-10-22 (carried): the note that latest-main receiving remains open (fetched main 50 commits ahead) is superseded by the main integration receiving that follows it (`archive_docs/2026-10-28/2026-06-08_system_diagnostics_and_accessibility_plan.md:392-409,462-466`).
- Where the evidence lives: each continuation's exact source and gates are in `support/family-promotion/diagnostics-*-qualification.json` and `-review.json`, and the focused September gates in `design_docs/mere_docs/testing/2026-09-29_apparatus_diagnostics_receipt.md`.

## Progress

- 2026-10-28: cut from `2026-06-08_system_diagnostics_and_accessibility_plan.md` and the apparatus row of `2026-09-23_crate_consolidation_plan.md`; nothing new landed.
