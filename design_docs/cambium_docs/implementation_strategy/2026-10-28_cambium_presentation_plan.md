# Cambium presentation: browser tenants, UA widgets, portable tiles and operator presence

**Status:** 2026-10-29: reconciled with the composition plan. Editable-component lifecycle, binding, scheduling and producer contracts now live there; its P0 identity repair is locally machine-qualified and P1–P6 remain proposed. This plan retains the browser-tenant, UA-widget, portable-tile and operator-presence residue cut on 2026-10-28. Browser presentation tenants are built and qualified within their recorded scopes; overlay roots P0–P2 retired with their substrate and P3–P6 remain unstarted. Turnstone already supplies the retained whole-pane owner used by the settings pilot; broader portable-tile and headed acceptance remain below. Operator presence still lacks a live host/source.

**For:** Cambium-hosted presentation shared by tenants and hosts: browser presentation routes, UA widgets and overlays, whole panes/tiles that keep identity across windows, and operator presence. The original directives and archived sources remain in Constraints and Phases. The related 2026-10-29 settings composition request is specified in the composition plan linked below.
Not here:
- The human assistive-technology validation gate for browser presentation tenants (automated ARIA-mirror checks do not count as human validation): `2026-10-28_platform_qualification_plan.md`.
- Woodshed's physical audio and other-platform qualification: Woodshed's own receipt, in its repository.
- The Genet render host's matching `without-classic-vello` feature: the Genet lane.
- Form submission, validation and autofocus behind the UA widgets: Genet proper, "separate, later".
- The co-op browsing lane that remote presence waits on: not in this set of plans.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

From the browser presentation tenants plan (`archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md`):
- The host owns the graphics context, frame schedule and presentation; renderer and content are tenants (:19). The first consumer is the viewer cone, Inspector and ordinary controls; other feature combinations need their own qualification (:20).
- Presentation choice is configurable (`auto`, `webgpu`, `webgl2`). A forced choice reports failure, and an ordinary draw error never silently switches backend (:22).
- The CPU dynamics route stays when compute is unavailable; an explicit request for unavailable compute reports its outcome (:23).
- Resource owner or appearance, content revision, dimensions, alpha and encoding, and context generation are tracked independently (:24).
- Required unsupported paint and resource imports produce attributed refusals, never omitted content. CPU and application state survive graphics loss, and old-generation handles and completions are rejected (:25-26).
- The viewer Wasm cap is 3,460,300 B (3.30 MiB, gzip6) per route artifact, R157 as amended; no cap is raised silently (:242, 296).
- Static-state backend comparisons gate at RMS 1, max 16 and alpha 0 at scale 1; baseline tolerances are declared before a run (:250, 285).
- The browser's asynchronous device-loss replacement boundary is preserved as a limit (:285).
- Portable composition's shadow-mask admission is a separate scope from the native one (:425).

From the overlay roots plan (`archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md`):
- Both directives are one engine capability: a view subtree mounted inside a document the view layer does not own, style-isolated from the page, laid out by the engine relative to a host node, painted in band, hit-tested first, with events routed to host app state (:63-67).
- Invariants for every slot kind: an own cascade scope; invisibility to page DOM APIs, which is the security boundary (autofill chips must not be page-readable); hit-testing resolves satellites before page content; satellites join the accessibility tree; satellite mutations classify through the same repaint-only and splice tiers (:146-152).
- The spec vocabularies are adopted outright, subset per phase: Anchor Positioning, Popover and the top layer, the Custom Highlight API, UA shadow roots (:254-258).
- ElementInternals is the checklist for the widget done-conditions and HTML-AAM names what each control feeds the accessibility tree; each widget phase records which obligations it does and does not yet meet. The widgets rebuild rendering and interaction, not the full HTML forms model (:40-46, 243-250).
- An anchor removed unmounts its satellite with an event to the host (:234-235). The runner batches per frame, not per state change (:239-242).

From the portable tiles plan (`archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md`):
- A cross-tree key registry is not built speculatively; only if a specific view-heavy DOM-lane tile needs its state to survive a tear-out before the forest dom landed, and it has since landed (:108-113).
- The graph-scope gesture axis (leaf, branch, fork) is orthogonal to a tile's DOM identity (:31-34).
- The scroll carry is the recurring load-bearing part: one member-to-scroll handoff on window reassignment, built once and reused (:89, 142-144).

From the operator presence plan (`archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md`):
- Presence is live and disposable: it rides an ephemeral channel and is never folded into the event-DAG (:89-97).
- The recording mode is a published, visible constitutional parameter, consented to by joining and never a hidden capability; covert surveillance is the line not to cross (:134-140).
- Your own conduct and your agent's activity are separate visibility grants; by default a guest cannot watch your agent work (:141-146).
- An agent's presence is local by default; a guest's is opt-in per moot (:188-191).

## Relationship to the composition plan

The [Cambium composition plan](2026-10-29_cambium_composition_plan.md) is the
canonical home for editable component instances, lifecycle/binding rules,
settings composition persistence, selective updating and diagnostic producer
contracts. Its P0–P6 replaces the RC sequence previously drafted here.
Presentation keeps browser tenants, UA widgets, portable whole-pane/tile
movement and operator presence.

For Turnstone settings, the existing whole-pane runner and RetainedMount
remain this plan's boundary. Composition owns rows inside that runner through
a flat instance table; it does not allocate a host runner per row or rebuild
the workbench's window model. PT1 and composition P6 share the headed move,
scroll, focus/IME and accessibility qualification; each receipt states which
boundary it proves. Component retention alone does not prove physical
presentation. Diagnostic collection and exact-frame qualification remain in
the [diagnostics plan](../../mere_docs/implementation_strategy/2026-10-28_diagnostics_and_accessibility_plan.md).

## Phases

### V1. Viewer image loading
The bounded viewer census is zero images, so loading images in the actual viewer is outside the first acceptance. Root-paint image replacement and removal were qualified in the consumer-host fixture only.
- Done when: the source states none.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:250, 287`

### V2. Portable composition shadow masks
Native shadow-mask admission is done. Portable composition still refuses shadow masks, pending an actual viewer inventory, and keeps a separate admission and qualification scope.
- Done when: the source states none for the portable route.
- Needs: the viewer's actual paint inventory.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:282, 425, 474`

### V3. Qualification boundaries left separate
Native input, browser and platform coverage beyond the runs made, and broader label-layout polish stay separate boundaries. Related label overlap is graph-readability polish, not presentation adoption. Woodshed's physical audio is its owner's receipt (see Not here).
- Done when: the source states none.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:474, 501-502, 506-508`

### V4. Cleanup record
Closing Task 7 meant removing only owned worktrees after their commits were integrated or preserved, and recording cleanup and the remaining feature and manual gates. The plan states the intent with no completion line, and the Cargo target directory `C:/t/cargo-targets/mere/netrender-adoption` was retained because cleanup was blocked.
- Done when: the removal of the owned worktrees and the fate of the retained target directory are recorded.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:255, 289, 400-402`

### UA3. UA shadow slot and the cheap widgets (overlay roots P3)
`<details>` and `<summary>` (a pure toggle, no IME) and `<input type=checkbox|radio>` through the existing Cambium control views, mounted as a UA shadow root whose element box is the host box.
- Done when: a fetched page's checkbox toggles, reflects into the page-visible checked state, and the widget is invisible to page scripts except through the standard reflected attributes.
- Needs: open question 3 (the substrate); Cambium controls in `crates/cambium/cambium/src/controls/` and `host_pool` in `tags.rs`.
- Rulings: OQ-1 (adopt the spec vocabularies); the two directives.
- From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:197-201, 215-219`

### UA4. The `<select>` widget (overlay roots P4)
The control view in the shadow slot, with its popup as an overlay slot on the same element.
- Done when: a real page's select opens, picks, commits, and its popup overlays page content correctly inside a scrolled band.
- Needs: UA3; open question 3.
- Rulings: OQ-1.
- From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:202-205`

### UA5. Text inputs (overlay roots P5)
`<input type=text>` and `<textarea>` through `text_field` in the shadow slot, with caret and IME through the same caret primitives the omnibar uses. The hard one, last.
- Done when: typing, selection, IME composition and paste work in a fetched form at parity with the omnibar.
- Needs: UA3, UA4; open question 3.
- Rulings: OQ-1.
- From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:206-209`

### UA6. The feature wave (overlay roots P6)
Link previews, annotation pins (statement-bucket consumers), autofill chips, and reader mode as a root-anchored overlay document. Each is a view and a state slice and lands as its own small plan citing this one. The first real feature also settles live placement of an overlay satellite into a window frame (which page node it anchors to, on-screen hit-testing), which was deferred to here. Overlay commands over the wasm content-worker wire are not in v1. Annotation data beyond rendering specs uses the W3C Web Annotation Data Model as an interchange projection of the statement-bucket model, when annotations want portability.
- Done when: each feature is its own small plan with its own done-condition.
- Needs: UA3 to UA5 for the widgets it relies on; open questions 3 to 7.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:210-213, 441-449, 422-424, 57-58, 289-293`

### PT1. Retained DOM-lane owners (portable tiles P1, re-grounded 2026-10-29)
For Turnstone settings, use the existing retained pane owner and `RetainedMount`. The [composition plan](2026-10-29_cambium_composition_plan.md) owns editable rows inside that runner through a flat table. The archived instruction to wrap the settings subtree in `PortableKeyed` is superseded for this consumer: a per-context nursery is not a shared cross-runner owner. Ordinary keyed children still use their local reconciliation contract. Other portable document-lane cards need a demonstrated owner before adoption. Handing over per-member textures for a frame-perfect handoff remains a nicety, not correctness.
- Done when: a cross-window move of a DOM-lane tile keeps its `NodeId` and scroll, with the target apply scoped rather than a full recompute, under test and a headed drive.
- Needs: composition P2–P3 for editable settings rows; P6 shares the remaining headed scroll and window-custody qualification. Existing whole-pane retention does not depend on building the new composition model. Open question 9 no longer blocks the settings pilot.
- Rulings: S51 (the portable tiles plan stays active against the tear-out gestures).
- From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:122-125, 93, 131-134`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1220`

### PT2. The trichotomy wired to the gestures (portable tiles P2)
The tear-out gesture (leaf, branch, fork) sets a tile's window assignment, note or rekey in `AppState`, and the portable machinery lowers each to the right DOM operation. The gesture tails of the archived tear-out plan are named against this phase: G5 copy and move on Turnstone, G6 cascade on Turnstone, and the ambiguous no-modifier drag, whose routed home (the notification subsystem plan) was retired on 2026-09-02.
- Done when: all three modes (leaf-move, sticky-note, rekey) produce the right survival behaviour from state alone, with no per-gesture DOM wiring.
- Needs: PT1; open question 9.
- Rulings: S51.
- From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:126-129, 71-82`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:997-1003`

### OP1. The ephemeral presence channel and the `OperatorFocus` overlay
The one new piece: a small stream per operator, `Presence { operator_id, colour, scale: Node | Element, target_ref, trail }`, emitted locally as an actor message or gossiped, rendered, expired and never reconciled into the durable log; and one more cartography `Overlay` variant, `OperatorFocus { operator, colour, node }`, which every consumer paints at its own positions.
- Done when: the source states none.
- Needs: open question 11.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:87-100, 170-172`

### OP2. Agent-local presence (build step 1)
Emit an `OperatorFocus` ring at node scale in the agent's colour, with a camera-follow option and a fading trail. No network. The plan calls it shippable before multiplayer exists and the more novel half. The agent focus source it assumed (`AgentObservation.focused_node` in the meerkat harness) is gone and no live agent-focus source is named in the tree.
- Done when: the source states none.
- Needs: OP1; a live agent-focus source (open question 11).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:149-153, 46-49`

### OP3. Into-node highlight (build step 2)
Descend to the focused node's document and highlight the focused accessibility element at its rect. The find overlay's rect-highlight path this step reused is gone; the accessibility projection survives in Turnstone.
- Done when: the source states none.
- Needs: OP2; a rect-highlight path (open question 11).
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:154-156, 55-57`

### OP4. The web-clip inspector as the local-cursor source (build step 3)
Build the hover-pick inspector as the local-cursor source of the same element-highlight primitive, so the inspector and presence share code. The building blocks exist (`web_clip_script` with its `elementFromPoint` picker, `ClipRect` and semantic clip lowering in `crates/fleece/src/clip.rs`; `write_clip_node` in `crates/mere/src/clip.rs`) but no host calls them; the inspector UI is unbuilt.
- Done when: the source states none.
- Needs: OP3.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:157-158, 61-85, 179-180`

### OP5. Co-op guests (build step 4)
The same `OperatorFocus` overlay fed by a remote operator's presence over the ephemeral gossip channel, fanned out across windows so a torn-out window shows the same rings. Remote presence waits on the co-op browsing lane.
- Done when: the source states none.
- Needs: OP1; the co-op browsing lane.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:159-162, 172`

## Open questions

1. The viewer-level task boxes: Task 1 and Tasks 3 to 7 of the tenants plan stayed unticked although its Progress reports them done (Task 2 is ticked). Reconcile the record, or drop the boxes? From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:53-69, 130-255, 277-296`
2. Recorded as options, not tasks: slim Genet main's browser-capture work in the viewer; align genet-parley onto one read-fonts; the viewer size levers. Do any become tasks? From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:296-297, 307, 382`
3. The substrate for UA3 to UA6 is gone: genet-layout's slots, Stylo's cascade and meerkat's host seam. Genet's shadow DOM plan (2026-09-07) does not take this plan up. Rewrite P3 to P6 onto Cambium and Genet's shadow DOM and spec work, or keep them as a historical design record? From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:215-219`
4. The page-script invisibility enforcement point (the DocumentScript mirror seam in meerkat's content actor) no longer exists, so the security boundary for autofill chips is unplaced. Where does it go? From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:223-230`
5. Satellite content must stay out of engram and clip capture (a clip under an annotation pin must not include the pin). Where is that filter? From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:259-261`
6. Can a DocumentScript mod request an overlay root? Host-only in v1; the capability shape rides the mod permissions model later. From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:262-264`
7. Do the smolweb and document lanes use the same slot machinery over `DocumentRenderPacket`, or chrome-DOM overlays as today? Deferred until the Genet-lane version proves the model. From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:265-267`
8. P2's two unclosed items name code that retired: the visual confirmation of engine-painted find fills, and deleting the match-rect compositing (which waited on that and a snapshot-lane decision). Does either still apply? From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:356-364`
9. Re-grounded for settings on 2026-10-29: Turnstone retains the pane runner outside placement, and Cambium's `RetainedMount` moves its owner within a shared document. The composition plan uses a flat row-owner table within that retained pane, rather than choosing a new whole-application lane model. Broader document-card and graph gesture adoption remains PT2 work. Historical question from: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:43-69, 131-134, 25-29`
10. P0 (surface-lane portability and scroll carry) is claimed delivered by Turnstone's `TearOutActivePane`, but the member scroll carry was not re-verified in this plan. Is a headed check still owed? From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:4-15, 89, 117-121, 151-154`
11. Which host and which sources carry operator presence? The agent-focus source and the find overlay went with meerkat. Surviving: the cartography `Overlay` channel (`ClusterHalo`, `BridgeEmphasis`, with a minimap consumer), the accessibility projection in Turnstone, and `ProvenanceSubKind::ClippedFrom`. From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:44-57, 174-180`
12. Presence design questions the plan left open: stacked rings or a blended colour for several operators on one node; trail length and decay, and whether the trail is a transient comet or the agent's durable operation nodes; element-reference stability across re-layout (anchor on the accessibility or DOM node, not a pixel rect); the set of recording modes and the per-tier default (orrery ephemeral; a large moot the constitution's call); whether the constitution must make the recording mode visible to members (lean yes); how the observe-activity capability is delegated and revoked (meadowcap now, Biscuit later; per role or per grant); and whether a charter may require agent transparency, with agent actions in the record private by default. From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:184-204`
13. The durable recording spectrum (public immutable record, private record, one-way glass) as a moderation and adjudication substrate has no build step and no home beyond the constitution, the capability layer and the event-DAG; adjudication plugging into tessera concord and reciprocity in moothold is unplaced. Where do they live? From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:102-146, 128-133`

## Findings

- 2026-10-29 (source inspection): Turnstone's settings pane owns its runner outside frame placement, and Cambium's retained mount parks/reparents its live DOM inside one document. Whole-pane movement is an existing boundary; editable-row retention and selective updates are distinct proposed work in the composition plan. The retained-mount tests are source evidence, not a new headed qualification run.
- 2026-10-13: `TextureProducer::portable_scene`, `ProducedScene`, `ProducerRegistry::plan_portable` and `admit_portable`, and `Host::capture_portable` exist; Netrender's `vello_backends/sessions.rs` owns CPU and Hybrid image and text admission, which direct WebGL reuses. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:260-262`
- 2026-10-16: Netrender generates shadow-mask keys from `0xFFFF_0000_0000_0000` and retains image overrides across frames. Rootstock keeps physical producer allocation below that band and attributes masks by ownership and producer draw boundaries; moving masks into ordinary inline-image keys would let an old override capture a later inline image. The whole scene is validated before anything is rewritten. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:408-425`
- 2026-10-15: the viewer ships per-route artifacts (`pkg/webgpu/`, `pkg/webgl2/`); Auto probes for an actual WebGPU device and remounts on WebGL2 if the WebGPU artifact refuses at boot. The Classic Vello opt-out is a compile error off wasm32. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:295-296`
- 2026-10-17: the extent walker excludes descendant content behind nested hidden, clip, auto and scroll boundaries while keeping each boundary's border box and visible overflow; the document-scroll observation reports unavailable when no layout exists. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:481-488`
- 2026-10-06: `PortableKeyed` and the nursery live in `crates/cambium/cambium/src/context.rs`; Frisket keys `Slot::View` tiles by `TileId` (`frisket.rs`); `GenetMultiRunner::push_forest_projection` (`multi.rs`) is used by cambium-rootstock and Turnstone; Turnstone's `TearOutActivePane` keeps the pane's DOM, widget state and scroll untouched by the move. From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:4-15, 25-29, 151-154`
- 2026-10-06: the overlay roots' surviving library parts are the controls (`crates/cambium/cambium/src/controls/`, `select.rs`, `slider.rs`) and `host_pool` with its splice-safety test (`tags.rs`); no `HighlightRegistry`, `OverlayRegistry`, `set_overlay` or `ContentLayout` remains in Mere or Genet's components. From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:4-12, 118-123`

## Progress

- 2026-10-28: cut from the browser presentation tenants, overlay roots and UA widgets, portable tiles and operator presence plans; nothing new landed.
- 2026-10-29: drafted the retained-component design here, then reconciled it into the canonical composition plan. Removed the duplicate contracts and phase sequence; PT1 retains whole-pane movement and shares qualification with composition P6. Implementation, runtime qualification and publication remain open.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- The `ElementSplice` move primitive (P4), linear key lookup against a hash index, and the duplicate-key policy; gated on the ui_polish finding 5. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:142-145`
- Retained text rendering fidelity: image-only inline links, per-band link re-harvest caching, and the glyph-to-character cluster map on `GlyphRun` that the document-lane find left as its follow-on (also the tails at 109-114). From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:185-190`
- The registry-as-one-seam thesis and the searchable-menu design S1 to S3; a configurable menu is absent. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:275-280`
- Per-window multi-DPI (forest_dom F4), deferred on purpose until a real multi-monitor case. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:434-438`
- The shellbar-outside-the-frame-tree ruling as a host-agnostic design note, and the F2.3 persona chip. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:857-859`
- Second-application reuse of `summary_body` (component catalog C5); the catalog's Decisions and promotion bar need a standing home. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1247-1250`
