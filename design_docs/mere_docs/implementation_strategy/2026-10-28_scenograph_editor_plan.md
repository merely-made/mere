# Scenograph Editor Plan

**Status:** 2026-10-28: cut from `2026-10-07_scenograph_editor_plan.md`, archived in `archive_docs/2026-10-28/`. Done: one history in Cambium and saves into the mere session (E1, E2, E2b), canvas pan, select and context menu (C1), options declared as data with rows built from the declaration (E3 to E5), the command home moved to Hest (C2), the swatch grid (S1), the first declarative presentation-rule slice (R1), the rule runner built (R2), the graph grammar's attention, seed, Partition, Sets, Mosaic and territory scenes, and read-only inspector and Forme drafts in Graphshell. Open: landing and adopting R2, linked swatches (S2), viewer backdrops (B1), primitive editor writes, headed qualification of every delivered draft, and the mapped-not-opened areas.

**For:** the editor surfaces Scenograph's authored scenes stand on, built in Graphshell's projection editor first. The lane was opened by Mark on 2026-10-07 ("wallpapers, props, canvas editor, node/edge/field style editors, scripting") and carries the balaur review's rulings A (generic history in Cambium) and B (options declared as data); ruling E is the dynamics plan's. Product direction lives in the cross-app design language record (`2026-08-23_projection_scenes_and_graph_native_platform.md`, section 9); the Codex handoff `research/2026-10-09_scenograph_codex_handoff.md` hands over S2, B1 and the dynamics slot.
Not here: the recipe's dynamics slot (F192 to F195), determinism rulings G8 and G10, field regions and field authoring (dynamics grammar plan); the Hest command architecture (C2 continuation, archived complete, residue to the platform qualification plan); the viewer's size budget and the Graphshell one-tree cutover (graphshell host plan); theme and typography (appearance plan); node facet dissolution (stack lift plan); resources, assertions and attribute storage (graph semantics plan, through its owner); the node body designer B3 (node body face model plan); Sets live tag-edit cache invalidation (projection grammar plan); grammar targets A3 stage two, A5 and Track F (projection grammar plan).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

Sources are lines of `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md`.

- History: one generic `History` over any cloneable snapshot (crate `backstep`); an entry holds the draft and the selected panel and never the workbench layout; working edits live in memory, durable edits go through the mere session's journaled undo (SE5, SE6, SE11). Lines 76-78, 88-92.
- Options are declared as data in scenograph, filled in by each catalog family and each `SolverCapability` (empty by default); an undeclared key is refused at compile before solving (SE7, SE8). Lines 80-82.
- Rule language: rhai, privilege by bindings; a shared scene's rules run with a narrower binding set under rhai's operation budget; maths reaches rules only through host functions on `libm`, never rhai's maths package; the seam is `ValueEngine` in Genet's `script-engine-api` (SE36 to SE42). Lines 150-160.
- Rule runner shape: a script is a `Condition::Script` inside the declarative rule set, evaluated through `ValueEngine` only, keeping the resolver, precedence, traces and hysteresis; fresh scope per evaluation; registering a name twice is refused; the running code and `script-engine-api` dependency sit behind the off-by-default `scripts` feature while the `Condition::Script` variant stays in the data model; a script that errors, overruns its budget or returns a non-bool leaves its rule unmatched with the reason in the trace; an absent fact reads as `()` (SE92 to SE100). Lines 322-387.
- Swatch: the grid is view only and editing inside a swatch arrives later as host intents (SE65); a reflection follows a view key or a sibling cell and a projection owns its recipe (SE58, SE66); scales are declared per axis, shared by default (SE56); dynamics in cells are settled, one live on focus (SE55); settled frames are seeded and run to the spec's stop on `libm` maths (SE70); the dynamics lane builds the recipe's dynamics slot and the grid consumes it (SE69). Lines 190-226.
- Linked swatches: the user-facing name is "appearance part" (SE88); the site's `mer3ly.facets` shelfmark key migrates, readers retaining the old value (reading, not ruled); each preview occurrence gets a separate transient body and an undisclosed physics channel is refused (SE89). Lines 258-269.
- Backdrop and attention: Reset restores the initial arrangement in relation to the backdrop; ambient context and props have separate visibility; visibility, collision, animation, picking and authored behaviour are independent choices; a backdrop may be an animated or interactive set; a collidable footprint is a physics obstacle whether or not it is visible (SE90). Selected nodes are foreground; deselecting an unpinned node returns it to the background without discarding it; a foreground pin is distinct from a position pin; no automatic hover retention or recency timeout (SE91). Reset refreshes to the seed (2026-10-16). Lines 270-312, 1288.
- Design responsibility (design language record, section 9.14, 2026-10-10): field behaviour and the node, link and field editors belong to the projection grammar lane; theme and typography to the Tabard lane; visible dynamics explanations and script and motion authoring to the dynamics lane. Lines 675-686.
- Graph semantics: anything touching resources and assertions goes through graph semantics' owner (SE74); node facets dissolve into attribute, resource and tag (SE75). Lines 234-236.
- Mosaic keeps its steady behaviour on a pannable infinite canvas; Partition is the viewport-bounded subdivision (Mark, 2026-10-16). Suggestions are derived graph members saved only once kept, with per-host suggestion settings off by default (2026-10-16). Lines 1289, 1294, 1306.
- The viewer's per-route wasm cap is 3.50 MiB (3,670,016 bytes), raised on Mark's call 2026-10-16. Lines 1304-1305.
- Draft delivery: drafts may be pushed straight to main for other machines to review (Mark, 2026-10-10); an unrun check stays unrun, and publishing a draft does not complete the plan's acceptance gate. Lines 908-920.

## Phases

### R1. Presentation rules in Graphshell
The declarative model (target, condition, effect; explicit precedence; hysteresis held by the host per rule and occurrence) is done, with a retained "Presentation rules" editor, exact-occurrence previews, a node ladder and relationship and dynamics inspectors delivered as drafts. Not adopted: Graphshell's live presentation consumers beyond the bounded face and selected-attribute ones.
- Done when: the headed renderer cases pass; hit-test and read cost of bound previews, long-context presentation, open-inspector traversal and live-context and rule costs are measured before large-graph adoption; exact occurrence adapters exist for sidecar-only and derived relations, with an execution-owner adapter and compiled-scene occurrence mapping; ForegroundPinned and Overview are mapped once their owners exist; node representation and detail measurement, compact content cards, snapshots, sprites and interactive representations arrive through their content and measurement owners; preview hysteresis is reconciled with live LOD state per view and occurrence; multi-tab IndexedDB races, conflict recovery, large-store cost and release performance meet the editor's earlier gates.
- Needs: dynamics owner for per-term or per-assertion contributions and installed field coupling snapshots; content and measurement owners.
- Rulings: SE36 to SE42.
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:397-427,2005-2015,2201-2208,2390-2397`

### R2. The rule runner: landing and adoption
`script-rhai` implements Genet's `ValueEngine` and `scenograph::presentation` carries `Condition::Script` and `RuleSet::resolve_with_scripts`; rule maths is bit-identical on native and wasm32 under tests in `crates/script/rhai`.
- Done when: after SE98's change `cargo check -p graphshell` passes, the branch is rebased, the touched tests re-run and it is pushed to main; and the not-done list is closed or ruled out: a script `Effect` (SE92 said "perhaps", not ruled in); Graphshell adoption (its rule editor offers a script condition, `BoundContext::resolve` supplies an engine, some host registers one); the per-trust-tier binding policy for shared scenes; region and field rules (field plan P4) and piccolo's `ValueEngine`; browser parity (the study host's JavaScript resolver understands the `script` condition); a release with `scripts` on once Genet publishes `script-engine-api`.
- Needs: Genet publishing `script-engine-api`; the dynamics plan's field regions for region and field rules.
- Rulings: SE92 to SE101.
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:388-391,428-480`

### S1. Swatch grid tails
The grid shows the recipe's families as cells in the projection editor with a live focused cell and a pick applied as one undo step.
- Done when: the dynamics axis (implemented, SE69 and SE89) passes its browser qualification including the open Firefox gate; settled frames with dynamics (SE70) land once the dynamics slot exists; the gloss pane is the second consumer, bringing reflections and scope axes on a native host (SE62).
- Needs: the dynamics plan's recipe slot and settle function; a native gloss host.
- Rulings: SE53 to SE70.
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:4,551-566,1287`

### S2. Linked swatches
A facet's cells act together so the mer3ly sandbox's scatter, deck and linked views move onto the stack: a shared selection of clauses per cell (crossfilter or highlight) routed as a host intent; appearance-part selection under its new name; per-cell visibility (the deck's dismiss) as a visibility delta; consumers are the projection editor's grid and the site's sandbox. Previously selected background membership must survive temporary coordinated-selection clauses.
- Done when: selection and visibility round-trip in a facet and replay through scenotime; a cell's hidden appearance leaves the source and the other cells untouched; the site's sandbox state maps onto it without loss; a headed check links two cells' selection in Chrome and Firefox.
- Needs: coordination with the viewer-file owner (the site's R162); the dynamics slot and settle work come first in the handed-over order (dynamics, grid axis, B1, S2); migration of `mer3ly.facets`.
- Rulings: SE84, SE85, SE87, SE88, SE89.
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:264,568-580,683-694,786-791`

### B1. Backdrops in the Graphshell viewer
The viewer draws a scene's set with independently visible ambient context and props; Reset restores the initial arrangement; field waits until defined. The backend draft binds a scene's spaces and transforms for paint and static contact.
- Done when: acceptance covers independent visibility and collision, saved-state round trips and headed Chrome and Firefox checks; static obstacle acceptance alone does not establish animated or interactive scenery. Still to build: viewer drawing and controls, a disclosed collision edge, the tangible versus intangible exclusion control, saved scene state, the animated or interactive set slice against its existing owners, and the optional retain-working-selection setting (its UI, persistence and the boundary of a qualifying interaction). Concave contact polygons and contact paths or points refuse pending a footprint adapter; deeper or idle placements keep a qualification limit.
- Needs: the reset baseline defined across its state owners (arrangement baseline, body velocities, animation time, script state; `restore_arrangement` restores stored positions and pauses physics only); exact control names, off and pause semantics, persistence and legacy-mode migration through the viewer owner; interactive props need explicit entity, input and action wiring and animated or scripted reset its own behavioural qualification; the field backdrop needs a definition.
- Rulings: SE86, SE90, SE91.
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:270-312,582-627`

### G. Graph grammar tails
Open notes of the 2026-10-16 and 2026-10-17 landings. Nothing has been looked at in a headed browser for the attention and seed work. A reset while physics is playing jumps, since physics owns the bodies. Rings and drill-in are solver-side only (no canvas control picks Rings). The territory field has not been seen headed; meaning keys as domains, the soft edge and the hidden current are not built. The activity badge is not built, nor hiding inferred, same-resource and contains links (link appearances do not carry those kinds). No host reads pandect's arrangement facets back, positions included; adopting that store is the facet-convergence migration, left open. The design project's DECISIONS *Scene defaults* row and its brief section 3 still describe Mosaic as the workbench split and need the 2026-10-16 amendment.
- Done when: the sources state no wider condition than these notes being built, run headed or amended.
- Needs: the design project owner for the DECISIONS amendment.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:1288-1296,1301-1305`

### P. Primitive editor writes
Read-only inspection of access and Resource, link and exact assertion, and field and coupling, plus a bounded Forme arrangement draft with saved layout undo, are delivered as drafts. Writes remain planned and follow one pattern: Inspect, Edit, Preview, Apply or Discard. An edit binds to the inspected owner, reads a baseline, validates the full draft and re-resolves identity before apply; a deleted or changed target keeps the draft with an explanation and never retargets to another node at the same address; derived links route editing to their owner and disclose their rule and inputs. Exact-statement selection and member-to-Resource inspection paths come before the field read and write adapter.
- Done when: the walkthrough exposes the intended identity and shared effects; two accesses of one Resource stay distinct; a bundled link mark cannot edit the wrong assertion; one coupling edit leaves the others unchanged; malformed input commits nothing for a single-owner edit; apply, discard and undo with save and reopen preserve unrelated state; derived and undisclosed relations, hidden fields, changed or deleted targets and failed persistence are tested; keyboard reaches the same inspection and apply and discard actions. The proposed fixture steps 2 to 4 (authoring scene, inspected identities connected to the recipe, binding and runtime path, edit then deselect then undo then save and reopen) are not done.
- Needs: graph semantics' storage decision for arbitrary attributes on links and fields and its authorship and retraction contract for exact assertion mutation; the dynamics plan's field authoring draft; a scoped host transaction for one-coupling edits.
- Rulings: SE74, SE75.
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:708-728,766-834,836-1014`

### H. Headed qualification of the delivered drafts
No browser receipt qualifies the newer sources of the Forme bridge and draft, the access, link, field and relationship inspectors, the presentation-rule editor, the exact-occurrence previews and the node ladder. Native key and drag automation is unverified; Firefox and Safari were not run for the aesthetic study; the connected browser on S-PC has no WebGPU adapter, so runs belong to a WebGPU-capable browser on O-PC, the ThinkPad or a Mac. Inspector label and value spacing, focus and scroll follow-through, and the weak visual cue of a centre-drop stack preview remain open visual-review gates.
- Done when: the four Forme scenarios (`p4_tree_forme`, its reopen, `p4_tree_forme_draft`, its reopen) pass on one retained origin and profile with reviewed captures tied to the tested build, reopen really using retained storage, and focus and cancellation observations recorded; the other drafts' scenarios pass in Chrome and Firefox on fresh origins, with Safari recorded on a Mac if run.
- Needs: a WebGPU-capable machine.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:1052-1108,1168-1176,1213-1222,1311-1380,1448-1503,1586-1594,1760-1797,1875-1880,2028-2040`

### M. Mixed-content scene and embedded Forme study
A proposal, not opened: one scene with document, image and video material, related nodes and a Forme matching two side-by-side page accesses (identities recorded; a captured face distinguishable from a live surface); a comparison of appearance treatments and representation choices with a target, condition, effect sketch that exposes inputs, precedence, fallback and eligibility; and the tile-to-field bridge (hover subdivisions, selection, focus, moving the outer field, unlocking its internal layout).
- Done when (candidate, not recorded): treatment changes preserve source and access identity and manual placement; font or face changes update measured extents; returning to a tile reuses its session; moving the field preserves a locked local split; selection and deselection preserve edits; unlocking a layout round-trips through undo and save and reopen; each displayed rule and overlapping-field contribution is explained; compact and detail thresholds do not oscillate because their own representation changes the measured size.
- Needs: the dynamics qualification then B1 then S2 order and the viewer-file ownership hold first.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:1016-1050`

### N. Mapped, not opened
Each returns as its own round of questions and none is a track. Wallpapers and props: Seiche's living backdrop and scene bodies have no Mere or Turnstone host; the games wing's `MapDocument` carries a `props` layer. Style editors: the node body designer (B3) is open in its own plan, the edge editor starts from forme's `EdgeProjectionSpec`, no host calls `add_field_at` for fields, and appearance is still three strings; style authoring must separate data encoding, interaction state and theme values. Authored motion: balaur rulings C (our own easing set) and D (a rotation mode per `TransitionSpec`) wait on expansion lane L5's authored front-end, and lane L4 waits on a consumer. Candidate proofs recorded: select and edit a subject, preview another context, deselect with edits surviving; two linked appearances keep independent visibility and distinguish selection from data encoding.
- Done when: not opened; no done-condition is stated.
- Needs: lane L5's authored front-end; a consumer for L4; the node body face model plan for B3.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:48,61-62,668-706`

## Open questions

- Does Turnstone move its command-menu store from its session sidecar (`ViewIntentV1.command_menu`) onto the shared Hest type? Mark's to rule; SE45 was a reading, not ruled. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:166,659-664`
- Is R2 on main? The plan, its status and its track text still say built and unpushed, while SE101 ordered check, rebase and push. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:13-23,388-391,430`
- B1's text says the reset baseline must be defined first; the 2026-10-16 entry says the seed ruling answers it. Does the seed ruling close that prerequisite (the other state owners, body velocities, animation time and script state, are not covered by it)? From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:582-627,1288`
- Does the saved seed also travel with the scene recipe in the shelfmark? Today it travels with the saved scene and the personal arrangement facets only. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:1288,1295`
- Primitive forks to return before code: whether the narrow edge vocabulary replaces or only labels current relations; where arbitrary attributes on links and fields belong; how incompatible overlapping placement, projection and behaviour rules compose; the field trigger, attribute placement and terminology choices. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:708-728,786-790`
- Is a form submission one change or several? `save_metadata` commits title and tags before parsing the optional JSON facet, so invalid JSON fails after a partial change; no repair or new undo semantics is claimed. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:821-834,905-935`
- Style authoring forks: channel assignments and precedence, hover-preview gestures and touch equivalents, and preview commit and history behaviour. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:673-706`
- The retain-working-selection setting: its UI, persistence and the exact boundary of a qualifying interaction. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:270-312`

## Findings

- 2026-10-15: Genet's `ValueEngine` landed in `script-engine-api` and mere moved all 30 Genet dependencies onto it; rhai's engine is `RhaiValueEngine` in `crates/script/rhai`, the runner in `crates/cambium/scenes/scenograph/src/presentation.rs`. rhai's built-in maths gave different bits on native Windows than in the browser while `libm` gave identical bits on both targets; the in-tree test is `crates/script/rhai/tests/maths_bits.rs` with `wasm_maths_bits.mjs`. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:13-23,428-480,1110-1139`
- 2026-10-09: pictograph's Seiche bridge (`crates/canvas/pictograph/src/canvas/seiche_bridge.rs`) makes a body per kernel surface node and springs per visible relation cell, so springs carry endpoint pairs and cannot expose exact assertion provenance; coupling conversion resolves a selector to a snapshot of node keys and does not enforce a field's extent or detect entry and exit; an open response IRI is preserved by numen but ignored by the force integrator. These are gaps for the primitive editor loop. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:754-765`
- 2026-10-10: the B1 backend (`Canvas::set_projection_backdrops`) leaves residual overlap in a 120-step probe because Rapier caps correction at 3 world units per second; the active-law and control check uses the canvas's 360-step settle budget, and the exclusion tolerance and host settle budget are unchanged. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:612-627`
- 2026-10-07: the only `Solver` implementations in the repositories surveyed are test fixtures; an unknown option on a custom solver is refused at compile. From: `archive_docs/2026-10-28/2026-10-07_scenograph_editor_plan.md:1110-1139`

## Progress

- 2026-10-28: cut from `2026-10-07_scenograph_editor_plan.md`; nothing new landed.
