# Cambium presentation: browser tenants, UA widgets, portable tiles and operator presence

**Status:** 2026-10-28: cut from `2026-10-13_browser_presentation_tenants_plan.md`, `2026-07-05_overlay_roots_and_ua_widgets_plan.md`, `2026-07-08_portable_tiles_plan.md` and `2026-06-25_operator_presence_overlay_plan.md`, archived in `archive_docs/2026-10-28/`. The browser presentation tenants are built and qualified (coordinated supplier adoption 2026-10-15, native shadow-mask admission 2026-10-16, document scroll boundary adoption 2026-10-17); what is open there is a short residue. Overlay roots landed P0 to P2 and then retired with their substrate; P3 to P6 were never started. Portable tiles and operator presence are plan-only and their hosts are gone.

**For:** Cambium-hosted presentation shared by tenants and hosts: the viewer on direct WebGL2 when WebGPU is unavailable, browser features rebuilt as UA widgets and overlay views, tiles that keep their identity across windows, and a live presence overlay for operators (an agent, a guest, yourself). It was opened by Mark's two 2026-07-05 directives (overlay roots; UA widgets rebuilt from the Cambium control views), the 2026-10-13 browser WebGL2 presentation tenants design, step 4 of the one-state, N-windows design (portable tiles), and the 2026-06-25 presence idea.
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

### PT1. DOM-lane tiles as `PortableKeyed` children (portable tiles P1)
Wrap portable shell-DOM subtrees (start with a settings tile or a document-lane card) in `PortableKeyed`, keyed by member. A float view child through `PortableKeyed` is a tail of the archived workbench component plan, and belongs here. Handing over per-member textures for a frame-perfect handoff was a nicety, not correctness.
- Done when: a cross-window move of a DOM-lane tile keeps its `NodeId` and scroll, with the target apply scoped rather than a full recompute, under test and a headed drive.
- Needs: the lane split settled (open question 9); a host (Turnstone and Cambium replace the retired meerkat shell).
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
9. Which lane model do portable tiles take: the two-lane story (DOM-lane tiles by `PortableKeyed`, surface-lane tiles by member-to-window reassignment) or the workbench migrating into the shell DOM? The premise (meerkat's `WindowView`, Pelt's `TileShell`) is gone, and the plan needs re-grounding on Cambium and Turnstone. From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:43-69, 131-134, 25-29`
10. P0 (surface-lane portability and scroll carry) is claimed delivered by Turnstone's `TearOutActivePane`, but the member scroll carry was not re-verified in this plan. Is a headed check still owed? From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:4-15, 89, 117-121, 151-154`
11. Which host and which sources carry operator presence? The agent-focus source and the find overlay went with meerkat. Surviving: the cartography `Overlay` channel (`ClusterHalo`, `BridgeEmphasis`, with a minimap consumer), the accessibility projection in Turnstone, and `ProvenanceSubKind::ClippedFrom`. From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:44-57, 174-180`
12. Presence design questions the plan left open: stacked rings or a blended colour for several operators on one node; trail length and decay, and whether the trail is a transient comet or the agent's durable operation nodes; element-reference stability across re-layout (anchor on the accessibility or DOM node, not a pixel rect); the set of recording modes and the per-tier default (orrery ephemeral; a large moot the constitution's call); whether the constitution must make the recording mode visible to members (lean yes); how the observe-activity capability is delegated and revoked (meadowcap now, Biscuit later; per role or per grant); and whether a charter may require agent transparency, with agent actions in the record private by default. From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:184-204`
13. The durable recording spectrum (public immutable record, private record, one-way glass) as a moderation and adjudication substrate has no build step and no home beyond the constitution, the capability layer and the event-DAG; adjudication plugging into tessera concord and reciprocity in moothold is unplaced. Where do they live? From: `archive_docs/2026-10-28/2026-06-25_operator_presence_overlay_plan.md:102-146, 128-133`

## Findings

- 2026-10-13: `TextureProducer::portable_scene`, `ProducedScene`, `ProducerRegistry::plan_portable` and `admit_portable`, and `Host::capture_portable` exist; Netrender's `vello_backends/sessions.rs` owns CPU and Hybrid image and text admission, which direct WebGL reuses. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:260-262`
- 2026-10-16: Netrender generates shadow-mask keys from `0xFFFF_0000_0000_0000` and retains image overrides across frames. Rootstock keeps physical producer allocation below that band and attributes masks by ownership and producer draw boundaries; moving masks into ordinary inline-image keys would let an old override capture a later inline image. The whole scene is validated before anything is rewritten. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:408-425`
- 2026-10-15: the viewer ships per-route artifacts (`pkg/webgpu/`, `pkg/webgl2/`); Auto probes for an actual WebGPU device and remounts on WebGL2 if the WebGPU artifact refuses at boot. The Classic Vello opt-out is a compile error off wasm32. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:295-296`
- 2026-10-17: the extent walker excludes descendant content behind nested hidden, clip, auto and scroll boundaries while keeping each boundary's border box and visible overflow; the document-scroll observation reports unavailable when no layout exists. From: `archive_docs/2026-10-28/2026-10-13_browser_presentation_tenants_plan.md:481-488`
- 2026-10-06: `PortableKeyed` and the nursery live in `crates/cambium/cambium/src/context.rs`; Frisket keys `Slot::View` tiles by `TileId` (`frisket.rs`); `GenetMultiRunner::push_forest_projection` (`multi.rs`) is used by cambium-rootstock and Turnstone; Turnstone's `TearOutActivePane` keeps the pane's DOM, widget state and scroll untouched by the move. From: `archive_docs/2026-10-28/2026-07-08_portable_tiles_plan.md:4-15, 25-29, 151-154`
- 2026-10-06: the overlay roots' surviving library parts are the controls (`crates/cambium/cambium/src/controls/`, `select.rs`, `slider.rs`) and `host_pool` with its splice-safety test (`tags.rs`); no `HighlightRegistry`, `OverlayRegistry`, `set_overlay` or `ContentLayout` remains in Mere or Genet's components. From: `archive_docs/2026-10-28/2026-07-05_overlay_roots_and_ua_widgets_plan.md:4-12, 118-123`

## Progress

- 2026-10-28: cut from the browser presentation tenants, overlay roots and UA widgets, portable tiles and operator presence plans; nothing new landed.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- The `ElementSplice` move primitive (P4), linear key lookup against a hash index, and the duplicate-key policy; gated on the ui_polish finding 5. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:142-145`
- Retained text rendering fidelity: image-only inline links, per-band link re-harvest caching, and the glyph-to-character cluster map on `GlyphRun` that the document-lane find left as its follow-on (also the tails at 109-114). From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:185-190`
- The registry-as-one-seam thesis and the searchable-menu design S1 to S3; a configurable menu is absent. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:275-280`
- Per-window multi-DPI (forest_dom F4), deferred on purpose until a real multi-monitor case. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:434-438`
- The shellbar-outside-the-frame-tree ruling as a host-agnostic design note, and the F2.3 persona chip. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:857-859`
- Second-application reuse of `summary_body` (component catalog C5); the catalog's Decisions and promotion bar need a standing home. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1247-1250`
