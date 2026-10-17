# Scenograph Editor Plan

**Date:** 2026-10-07
**Status (2026-10-10):** tracks E1 to E5 and C1 complete, each with headed checks in Chrome and Firefox (Safari not run); C2 landed (SE45 to SE51; F11 to F13), with headed checks in Chrome and Firefox; its label revision (SE52) landed with the web host's genet repin. S1, the swatch grid, landed 2026-10-09 (SE53 to SE83), its dynamics axis implemented in `e2543cee` (SE69, SE89; F200 to F202), with native/wasm fixture identity qualified against Genet `7422e906` and Vello `491c376c`; the handoff records browser qualification and its open Firefox gate. Rulings SE1 to SE91 (§1). Waiting: genet branch `value-engine` (SE39 to SE42) for Mark or genet's lane to land, then mere's rhai `ValueEngine` and the rule runner after a coordinated repin. *(2026-10-15: landed and repinned; rhai `ValueEngine` and the rule runner are track R2, built and unpushed.)* The Turnstone follow-up (SE32, SE35) is on Turnstone main at `d0775a1` (§3). Mapped, not opened: wallpapers and props, the style editors, authored motion (§3).
**Scope:** the editor foundation that Scenograph's editing surfaces stand on: one undo history in Cambium, and arrangement options declared as data. Carries out the balaur review's rulings A and B. The projection editor in Graphshell is the first consumer of both.

**2026-10-17 Pool follow-up:** the [projection-grammar plan's Pool mechanism audit](2026-08-15_projection_grammar_adoption_plan.md#pool-mechanism-audit-reuse-and-integration-boundaries-2026-10-17) examines a dedicated editable Swatch over a container node's mutable nested graph as a consumer of SE65's host-authorized interaction-intent direction. Existing nested storage, bounded Swatch callbacks, hierarchy projection and durable session history are present; their Pool ownership/history connection and presentation policies remain open. This follow-up does not expand the S1 grid's historical view-only qualification.

**2026-10-09 addition:** R1's first declarative slice is implemented and qualified: the cross-app design conversation
agreed target → condition → effect authoring, the six presentation examples,
explicit precedence and match explanations, and an interactive aesthetic study.
The Rhai runner still waits on SE39–SE42's coordinated Genet repin. The
declarative model and study do not reopen S2/B1 or claim their completion.

**2026-10-15 annotation (the repin happened):** the coordinated Genet repin
that SE42 waited for landed as a side effect of the WebGL/viewer work: mere
`0bb5facf` moves all 30 Genet dependencies in the root `Cargo.toml` to Genet
`ae3d9f5`, which carries `ValueEngine` (Genet `f0a91f7`, "Land ValueEngine, the
value-level script contract beside ScriptEngine") in `script-engine-api`. The
trait is the one branch `value-engine` held (`27477d3e`; F10); only the crate's
`rust-version` line differs. SE42's wait is over, and the status line's
"Waiting" clause and the paragraph above keep their original words as the
2026-10-09 and 2026-10-10 record. When the annotation was written nothing in
mere used `ValueEngine`; track [R2](#r2--the-rule-runner-se92-to-se97)
(2026-10-15) now carries rhai's implementation and the rule runner.

**2026-10-10 continuation:** the bounded Graphshell Forme draft and saved layout
undo are implemented in `69f17908`, with native/build qualification and an open
[visual test handoff](#forme-visual-test-handoff-2026-10-10). The next primitive
editor design stages exact-owner inspection before writes; general field
authoring and the unresolved storage/composition policies remain planned.

**2026-10-11 continuation:** access/Resource and
[link/assertion inspection](#link-and-exact-assertion-inspector-draft-2026-10-11)
now have read-only local-product drafts. The
[field/coupling draft](2026-06-13_scriptable_field_regions_plan.md#field-inspection-draft-2026-10-11)
adds exact records and overlap discovery. The
[declarative rule editor](#declarative-presentation-rule-editor-draft-2026-10-11)
adds scoped authoring, explicit decision preview and saved history; primitive
writes, live presentation adapters and derived-link owner explanations retain
their existing gates.

Not in scope, mapped in §3 and opened by later rounds:
- the scripting comparison (SE3), which is the next assessment;
- wallpapers and props, the node, edge and field style editors, and authored motion.

**Related:**
- [cross-app design language record, 2026-10-09](../../2026-08-23_projection_scenes_and_graph_native_platform.md#9-configurable-visual-and-interaction-language-2026-10-09): selection-shaped context and swatches, themeable forme regions, configurable appearance; §9.10 records the agreed presentation rules carried by R1.
- [balaur review brief](../research/2026-10-06_balaur_review_brief.md): rulings A to E and the findings (§3.2, §3.3) this plan builds on.
- [Scenograph expansion brief](../research/2026-08-10_scenograph_expansion_brief.md): lanes L1 to L5; L2 (backdrops) is landed, L4 and L5 wait on consumers.
- [projection grammar adoption plan](2026-08-15_projection_grammar_adoption_plan.md) and [catalog](../research/2026-08-15_projection_grammar_catalog.md): the grammar the editor authors. The same lane owns both documents and this plan.
- [dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md): G8 and G10 carry balaur's ruling E. Its labelled-digest crate (games wing ruling 609) is the one to share if an editor ever needs a document digest.

*Reading, not ruled* marks this plan's own inference. Everything else is a ruling, a quotation, or a file and line.

## 1. Rulings

### 1.1 Inherited (2026-10-06)

From the balaur review brief §1, quoted there verbatim:
- **A, where whole-document history lives.** Mark: **"Generic, in Cambium (Recommended)"**. One history over any cloneable document, timed by the host's clock, so no editor builds its own.
- **B, how arrangement options are declared.** Mark: **"Declared as data (Recommended)"**. Each catalog family and `SolverCapability` declares its options in a closed type set that matches today's seven readers; both refusal and the editor's rows come from that one declaration.
- **C and D, motion,** wait on lane L5's authored front-end: our own easing set, and a rotation mode per `TransitionSpec` (short arc by default; as authored, clockwise, counter-clockwise).
- **E** is in the dynamics grammar plan (G8, G10).

### 1.2 This plan's rulings (2026-10-07)

Mark opened the lane: "i'm most curious about scenograph, honestly. wallpapers, props, canvas editor, node/edge/field style editors, scripting... tons of possibility." The options below list the recommendation first, as the questions were put.

**SE1, the assessment's form.** Question: what form the assessment of the Scenograph editor takes. Options: a brief in mere; straight to a plan; keep it in chat. Mark: **"Keep it in chat"**. *Follows:* the assessment ran in chat with the evidence in §4. SE9 then chose this plan as the record.

**SE2, the first area.** Question: which area opens first and goes deepest. Options: the editor foundation (A and B); wallpapers and props; style editors; scripting. Mark: **"Editor foundation (Recommended)"**. *Follows:* this plan's tracks.

**SE3, scripting.** Question: how Scenograph scripting is treated, given four engines in the family (rhai, Wasmtime/WIT, piccolo Lua, genet's `script-engine-api`). Options: compare and rule later; through `script-engine-api`; rhai; Wasm components. Mark: **"compare all four and more, through script-engine-api and/or embedded. hell, i think the only one that doesn't go through script engine api is probably rhai, right? we thought about rune too. i'm ok with them going through the neutral seam provided there are no major drawbacks; i figured that was just for the backends that might need to manipulate the dom"**. *Follows:* a comparison of every candidate, Rune included, through the seam and embedded, comes back as a fork before any scripting is built. The answer given to his question is in §4, F6.

**SE4, the owner.** Question: who carries the lane. Options: this session; a new session. Mark: **"This session (Recommended)"**. *Follows:* the projection grammar lane carries this plan beside the catalog and the adoption plan; its handoff carries a dated note.

**SE5, the history and `EditHistory`.** Question: how the generic history relates to Cambium's text-only `EditHistory`. Options: generalize it; a new type beside it. Mark: **"Generalize EditHistory (Recommended)"**. *Follows:* one `History<S>` over any cloneable snapshot, with coalescing by edit key within a window on the host's clock and a saved marker for the unsaved-changes flag. The text field's history becomes one instance of it, its insert-run coalescing one policy.

**SE6, what an entry holds.** Question: what one entry in the projection editor's history holds. Options: the draft and the selected panel; the draft only; the draft, the panel and the layout. Mark: **"Draft + selected panel (Recommended)"**. *Follows:* undo restores the definition and returns to the panel where the change was made. The workbench layout is never recorded.

**SE7, where the declaration type lives.** Question: where the option-declaration type lives. Options: the type in scenograph; everything in scenomise. Mark: **"Type in scenograph (Recommended)"**. *Follows:* the type sits in scenograph beside `Arrangement.options`, which it describes. Each family in scenomise and each `SolverCapability` fills it in.

**SE8, custom solvers.** Question: whether a registered solver must declare its options. Options: required, empty by default; optional. Mark: **"Required, empty by default (Recommended)"**. *Follows:* `SolverCapability` gains an options list. A solver that declares none takes none, and an undeclared key is refused when the projection compiles, before solving, as the built-in families' are.

**SE9, the record and what follows.** Question: where the rulings are recorded and what comes next. Options: a plan doc, then build; into the balaur brief only; stay in chat. Mark: **"Plan doc, then build (Recommended)"**. *Follows:* this plan. E1 is built first, in a worktree off origin, and verified there before anything reaches main.

**SE10, where the history lives (2026-10-07).** Question: E1 put `History` in the cambium crate, but Graphshell's projection editor also compiles under the native features, where Cambium is absent; depending on cambium adds 46 crates to the native build (vello, html5ever, `genet-scripted-dom`, a font stack), and the web build already has it. Options: a leaf crate in the Cambium family; Graphshell takes cambium; undo only under web. Mark: **"Leaf crate in Cambium family (Recommended)"**. *Follows:* `History` moves to a dependency-free crate under `crates/cambium/`, as `workbench` is. Cambium depends on it and re-exports `History`, so `EditHistory` and E1's API stay where they are. Graphshell depends on the leaf. This amends where SE5's history lives, not what it is.

**SE11, the history and Eidetic (2026-10-07).** Mark asked: "Or wait, shouldn't that in some way rely on eidetic?" Evidence put with the question: pandect's `GraphSession` already keeps each author's undo and redo, rebuilt from a muniment journal, durable, and keeping the parts another author changed since (`crates/system/pandect/src/graph_session.rs`, `undo`, `redo`, `Reverted`); the editor's working edits coalesce within a 400 ms window. Options: two layers; the draft journaled in the mere; in memory, kept on close. Mark: **"Two layers (Recommended)"**. *Follows:* working edits use the in-memory `History`, a widget's working memory like a text field's undo. Saving writes the definition into the mere session, where saved changes get pandect's durable, attributed undo. Nothing at keystroke level reaches the journal.

**SE12, the save path (2026-10-07).** Question: the editor's web sink saves definitions to browser localStorage, outside any mere ([web_main.rs at the pre-cutover revision](https://github.com/merely-made/mere/blob/e84cfbd95b1660046d646580a8707af020a06e4b/ports/graphshell/src/web_main.rs), `BrowserProjectionSink`), against the 2026-09-23 ambiance ruling that apps keep their session data in a mere. Options: yes, as track E2b; yes, before E2; not now, recorded. Mark: **"Yes, as track E2b (Recommended)"**. *Follows:* track E2b.

**SE13, the crate's name (2026-10-07).** Question: the leaf crate's name; `edit-history`, `undo-history` and `cambium-history` were each checked free on crates.io. Mark: **"edit-history (Recommended)"**. *Follows:* `crates/cambium/edit-history`, `publish = false` until a publish is decided, so the name is chosen but not claimed. *Renamed 2026-10-15 (Mark): the crate is `backstep`, at `crates/cambium/backstep`, free on crates.io; it stays a standalone crate (folded into Cambium it would make the scene crates depend on the UI toolkit, and scenotime is the wrong layer for a text field). Still `publish = false`, now publishable. Later sections keep the old name where they record what was said then.*

**SE14 to SE17, E2b's shape (2026-10-07).** Mark said "Proceed" to E2b; these were put with the evidence: Graphshell saves scenes as a node at an address carrying a JSON facet (`save_product_scene`, `graphshell.saved-scene/v3`), a journaled edit pandect can undo; the editor has two localStorage writers, the plain sink and the executable practice path (definition plus selected occurrence), which `projection_authoring.scn` and `projection_reopen.scn` reopen through; the browser host calls `MereHost::persist` only at open, and `GraphSession::edit_now` holds an edit "not stored until the next flush", so a browser save appears never to reach IndexedDB (read in code, not seen at runtime); the web UI calls no session undo.
- **SE14, where a definition lives.** Options: a node and facet, like scenes; a view intent; a facet on the session item. Mark: **"Node + facet, like scenes (Recommended)"**. *Follows:* a node at `mere://projection/<definition id>` carrying a `graphshell.projection-definition/v1` facet.
- **SE15, which savers move.** Options: both paths; the plain sink only. Mark: **"Both paths (Recommended)"**. *Follows:* the executable path's facet carries the definition and the selected occurrence together; both scenarios reopen from the session, and localStorage leaves the editor.
- **SE16, flushing.** Options: persist after each save; a general flush policy first. Mark: **"Persist after each save (Recommended)"**. *Follows:* a projection save persists the session at once. Whether scene saves are lost on reload is checked at runtime and reported as next door, not fixed here.
- **SE17, proving undo.** Options: a native test only; a session undo control. Mark: **"Add a session undo control"**. *Follows:* the web host gains session Undo and Redo over `MereHost::undo` and `redo`, beside the editor's own, and the headed check uses it.

**SE18 to SE21, scoping SE17 (2026-10-07).** Mark, on SE17: "Hmmm. That last one… we should scope". Evidence put: pandect's undo is per author, an author is a persona plus a channel (`via`), and every change the web host makes goes through `"graphshell"` (addresses, file metadata, scene saves, detail edits, and now projection saves), while extension captures come through their own channel; so `undo("graphshell")` reverses Graphshell's latest change of any kind. `Reverted.kept` names parts another author changed since.
- **SE18, what it reaches.** Options: editor saves only, through their own channel; all of Graphshell's changes; both, as two controls. Mark: **"Both, as two controls"**. *Follows:* projection saves go through their own channel, `graphshell.projection-editor`, and the editor offers Undo save and Redo save over that channel alone; a host-wide session Undo and Redo reaches `"graphshell"`'s changes.
- **SE19, where it sits.** Options: in the editor, named apart; in Graphshell's chrome. Mark: **"In Graphshell's chrome"**. *Reading, not ruled:* this places the host-wide pair, since SE18's two-control option put Undo save in the editor.
- **SE20, the open draft.** Options: reload it as a draft step; reload fresh; leave it alone. Mark: **"Reload it as a draft step (Recommended)"**. *Follows:* after an undo or redo of a save, the editor takes the stored definition as one step on its own history and reads clean against the store.
- **SE21, kept parts.** Options: the status names them; not surfaced yet. Mark: **"Status names them (Recommended)"**. *Follows:* the status says what was kept and who changed it.

**SE22, session writes in the browser (2026-10-07).** Evidence put: `BrowserHost` lives in an `Rc<RefCell<…>>` that every handler and frame borrows, while `persist`, `undo` and `redo` await IndexedDB, so a borrow held across them panics on the next event; this is why browser scene saves never persist. pandect already splits a store into `pending(at)` and `stored(pending)`, and `MereHost` has a crate-private `stage`/`staged` for capture; undo and redo have no split. Options: split prepare, write and commit; hold the borrow and guard callers; a session worker task. Mark: **"Split prepare / write / commit (Recommended)"**. *Follows:* under the borrow, edit or revert in memory and take the pending batch; without it, write through a clone of the backend; under it again, mark the batch stored. pandect gains `undo_now` and `redo_now` (a revert journaled but not stored, like `edit_now`), `MereHost` gains public prepare and commit, and one write is in flight at a time.

*Note 2026-10-07, on SE22.* Mark: "Ehhh… i could also be persuaded to 3… but if 1 holds cross platform and browser engine hosts, then sure", and "if I find it to not work for chromium browsers, or firefox, or safari… eh". Evidence given: every muniment backend (memory, directory, redb, zip, IndexedDB) clones to a shared handle and `apply` takes `&self`, so a batch written through a clone lands in the same store everywhere; the split is synchronous pandect plus any `Backend`, nothing wasm-specific; native hosts keep the async path. Headed results are in Progress (E2b). A worker task (option 3) can still be layered on the same calls.

### 1.3 The canvas and command rulings (2026-10-07)

Mark, after E2b: "lemme also suggest we make it easier to move around the canvas by implementing drag canvas to pan". Evidence put: pictograph's canvas, shared by Graphshell's page, its tree page and Turnstone, pans on a middle-drag with momentum, pans on the wheel and zooms on Ctrl+wheel, moves a node on a left-drag, and starts a rectangle select on a left-drag over empty space.

**SE23, the gesture.** Options: configurable, pan by default; pan with Shift+drag selecting; Space+drag pans. Mark: **"left. right drag can be rectangle select, no?"**. *Follows:* a left-drag on empty canvas pans; a right-drag selects.

**SE24, where.** Options: in the pictograph canvas; Graphshell's page only. Mark: **"In the pictograph canvas (Recommended)"**. *Follows:* every host of the canvas gets it, touch included.

**SE25, the record.** Options: before E3 as its own plan; before E3 as a track in this plan; after E3. Mark: **"Before E3, a track in this plan"**. *Follows:* track C1.

**SE26, right-click.** Evidence put: a browser opens its native context menu on right-press on macOS and Linux and on release on Windows, before a drag is known; the canvas has no menu of its own. Options: suppress the native menu on the canvas; suppress only after a drag; right-drag with a modifier. Mark: **"Oh, we should replace the native context menu with ours, right? So right click: context menu, right click + drag, rectangle select"**. *Follows:* the canvas suppresses the native menu and opens Graphshell's own on a right-click that does not move past the click slop; a right-drag past it selects.

**SE27, the cut.** Options: pan and right-drag now, the menu next; all in one track. Mark: **"All in one track"**.

**SE28, the menu's content.** Mark: **"Commands, I suppose. Be nice if the right click gave you the context menu with the command palette's search bar at the top, then default + recently used commands under that. And if people could remove any of the commands and add new ones, at will, that would be great"**. *Follows:* the context menu is the command surface with its search field, then default commands, then recent ones, and its contents are the person's to change.

**SE29, the command set.** Evidence put: Cambium's `command_surface` is the live, generic one (string ids, labels, shortcuts, depth-one submenus, disabled reasons), and filters by query only as a palette; Graphshell's page dispatches 43 string commands; graph-kernel's `actions` (a closed `ActionId` enum with categories and reserved recency and pin keys) and `chrome::command_palette` (a session with query, scope and a Tier-1 category cursor) survive from the egui shell with no live host. Four questions, each answered with the recommendation:
- where the set lives: **"Generic, in Cambium (Recommended)"**: hosts register commands, and the set keeps defaults, recents and the person's adds and removes, for the menu and the palette alike;
- where recents and changes are kept: **"In the mere session (Recommended)"**;
- what adding means: **"Any registered command (Recommended)"**; user-written commands wait for the scripting comparison (SE3);
- whether the menu follows the cursor: **"Yes, node commands first (Recommended)"**; search always covers every command. *Correction 2026-10-08:* "43 string commands" is 49 dispatched, 32 registered (F12).

**SE30, the meerkat model.** Mark: **"That is one of the few things we did correctly in the meerkat era… historic, but worth reviving"**. *Reading, not ruled:* this is the donor command model the harvest brief kept (`2026-05-17_graphshell_harvest_brief.md`, "Command-context-rank policy", "Context-aware action visibility per surface type", "Disabled-action visibility with precondition tooltips") with what survives in code (`kernel::actions` categories and its recency and pin keys, `chrome::command_palette`'s scopes); the Cambium set revives it over open string ids rather than the closed enum.

Mark, on SE30: **"Uhhh, just to say, remember the scope of meerkat was different than cambium and graphshell. So adaptation is needed"**. *Follows:* the concepts carry (commands ranked by where the menu opened, commands per surface, disabled reasons shown, recents and pins); meerkat's scope does not (workbench and pane scopes, the closed enum of browser actions).

**SE31, where the menu's state is kept.** Evidence put: graph truth is journaled and on per-author undo stacks, so a facet would write a change on every command use; the view plane (`set_view`) is per channel and view name, logged but off the undo stacks, though `ViewIntent` was a fixed struct for canvas panes and `set_view` async. Options: the view plane; a facet on its own channel; split, with recents per device. Mark's first answer, "A facet, own channel", was a slip ("Run that last one back, i accidentally button mashed while typing"); asked again, Mark: **"View plane (Recommended)"**. *Follows:* Graphshell's view `commands` holds a `CommandMenuView` (added, removed, recent) on `ViewIntent`; `set_view` gains the synchronous `set_view_now`.

**SE32, Turnstone.** Mark: "We should apply this to turnstone, or harvest from it…". Evidence put: Turnstone's right-click on a graph pane selects the node under it and opens the omnibar's `>` command lane (`src/shell/input.rs`); its catalog (`src/app/palette.rs`, `available_actions`) puts contextual rows ahead of the static registry in one composition read by palette, snapshot and automation; it has no recents or adds and removes, and it takes the right press before the canvas. Options: harvest now, apply next; apply in this track; harvest only. Mark: **"Harvest now, apply next (Recommended)"**. *Follows:* the Cambium set takes Turnstone's ordering rule; a follow-up in Turnstone moves its catalog onto the set and opens its palette on right release, so right-drag selects there too.

**SE33, how the page draws the menu.** Evidence put: Graphshell's top bar and nav are painted by a Cambium view under transparent DOM hit targets, while its panels are ordinary DOM; Cambium's `command_surface` renders a Genet DOM. Options: page DOM like its panels; Cambium's `command_surface`, painted. Mark: **"Page DOM, like its panels (Recommended)"**. *Follows:* the page renders the rows; Cambium supplies the set and the order.

**SE34, the order after C1 (2026-10-07).** Evidence put: S1 (a versioned host dataset envelope, `scenomise::host_dataset`, `pub(crate)` on the relationship helpers) was checked file by file against E4 and does not collide. Options: E3 and E4 beside S1; the Turnstone follow-up first; pause E3 and E4 until S1 lands. Mark: **"Pause E3/E4 until S1 lands"**. *Follows:* the Turnstone follow-up runs meanwhile.

**SE35, who carries the Turnstone follow-up (2026-10-07).** Evidence put: Turnstone pins 51 mere dependencies at `f1d169c`, before C1, set that morning by a coordinated browser family repin with Windows and Linux qualification gates (Turnstone `abb349c`, `e00869c`, its unusual-protocols plan's S0); that lane was active the same day. Options: brief Turnstone's lane; this lane does it under their gates; wait. Mark: **"Brief Turnstone's lane (Recommended)"**. *Follows:* the brief in §3; nothing is written in Turnstone by this lane.

### 1.4 The scripting rulings (2026-10-07)

Mark chose the scripting comparison (SE3) while E3 and E4 wait. Evidence put: the family's lanes are placed (graph behaviors plan §1: rhai for privileged local automation, piccolo Lua for sandboxed participant bodies, Wasm components for portable untrusted mods, with the browser's jco path designed in `crates/script/document-host` but not built; JavaScript through genet's seam for page scripts); Scenograph's concrete need is region and scene rules that run on events and emit declarative effects Rust evaluates (field regions plan P4, numen's field AST), so they must run native and in the browser, be deterministic, and be safe when shared; rhai 1.26.1 (September 2026) is in mere with numen's `field-rhai` bindings; piccolo's upstream last released June 2024, genet runs Mark's fork and Isometry crates.io 0.3; Rune 0.14.2 (May 2026) and Starlark 0.14.2 (June 2026, deterministic and hermetic by design, wasm unverified) are new to the family.

**SE36, the rule language.** Options: rhai with privilege by bindings; by trust tier; Starlark; decide after probes. Mark: **"rhai, privilege by bindings (Recommended)"**. *Follows:* scene and region rules are rhai whoever wrote them; a shared scene's rules run with a narrower binding set under rhai's operation budget, with maths routed through libm for G8.

**SE37, the seam.** Options: a small value seam in mere; extend `script-engine-api`; embed directly. Mark: **"scope 2, if unreasonable then 1, failing that then 3"**. *Follows:* extending genet's `script-engine-api` with a value layer is assessed first; if unreasonable, a value seam in mere's `crates/script`; failing that, rhai is embedded directly.

**SE38, probes.** Options (several): Starlark; Rune; an rhai baseline; none. Mark: **"Starlark,Rune,rhai baseline"**. *Follows:* a scratch probe outside mere's tree builds each native and for wasm32 and measures binary size, a budgeted rule's time, and float determinism across the two.

**SE39, the seam, scoped (2026-10-07).** Evidence put: genet's `ScriptEngine` is JavaScript-shaped (realms, `WindowProxy` traps, reflectors, host promises, `CallCx` callbacks), so a value layer is a sibling trait in `script-engine-api`, not a widening of `ScriptEngine`; the crate already calls itself an engine-neutral contract and has a non-JS backend (piccolo); mere pins genet at `965b64e2` in 28 dependencies, so the trait lands in genet first and mere follows at a coordinated repin, each time it changes. Options: option 2 as scoped; unreasonable, so option 1; option 3. Mark: **"Yes, option 2 as scoped (Recommended)"**. *Follows:* a `ValueEngine` trait in genet's `script-engine-api` (neutral values, evaluation under the crate's `Budget`, host functions over neutral values, globals), landed through genet's lanes; rhai implements it in mere's `crates/script/rhai`; Scenograph's rule runner depends on the trait only; piccolo's backend may add it.

**SE40, rule maths (2026-10-07).** Evidence put: the probes (§4, F8): rhai's built-in maths gave different bits on native Windows than in the browser, while `libm` gave identical bits in all three engines on both targets. Mark asked: "i feel like we had a particular stack answer for this... perhaps motivated by dynamics? have we used libm or alternatives elsewhere in the stack?" and "or burn". Answered: stack seams S4 ("Ordered, declared, cross-platform") routes seiche's transcendental maths through `libm`, G8 carries it with a lint, and glamx is built with `libm` and `scalar-math`; numen lowers its field algebra to Burn for batch throughput, not bit-reproducible across backends, and Burn's outputs are stamped signals, not replay truth. Options: libm per S4; rules build field expressions; both. Mark: **"libm, per S4 (Recommended)"**. *Follows:* rules reach maths only through host functions on `libm`; rhai's built-in float maths package is not loaded.

**SE41 and SE42, landing SE39 (2026-10-07).** Evidence put: genet has no `CLAUDE.md`; its main moves daily (the Streams lane touched `script-engine-api` that day) and its local checkout held others' unpushed commits; the addition is a new module, not an edit to `ScriptEngine`. Options for who writes and lands it: this lane writes it and Mark or genet's lane lands it; this lane writes and lands it; brief genet's lane. Mark: **"I write it, you or genet's lane land it (Recommended)"**. Options for mere's side: wait for the repin; start against the branch. Mark: **"Wait for the repin (Recommended)"**. *Follows:* genet branch `value-engine`, nothing pushed to genet main by this lane; rhai's implementation and the rule runner start in mere after a coordinated repin brings the trait in.

**SE43, declared defaults (2026-10-08).** The site session's S1 landed on mere main at `f67f5080`, so E3 and E4 resumed (SE34). Evidence put: the families' defaults are of four kinds: constants (`rotation` 0, `invert_y` false), an enum's default (`curve`), "auto" where absence lets the family choose (`depth`, `subdivisions`), and measured from the items (Grid's `cell_width` is the largest item's width, Timeline's `axis_length` the count times the pitch). Options: static, plus resolve on demand; words only; resolved only. Mark: **"Static, plus resolve on demand (Recommended)"**. *Follows:* a declaration's default is a value, "auto", a measured default in plain words, or empty; a family resolves the numbers for a given set of items.

**SE44, options when the arrangement changes (2026-10-08).** Evidence put: switching Grid to Spiral left `columns: 3` in the draft; Spiral does not declare it, so compile refused the projection, while no row showed it. Options: drop them, Undo brings them back; show them as rows to remove; keep them and carry them back. Mark: **"Drop them, undo brings them back (Recommended)"**. *Follows:* a switch to a known arrangement keeps only the options it declares, as one step on the editor's history; while the id names nothing known (being typed), every option is kept.

**SE45, one store for the command menu's choices (2026-10-08).** Evidence put: Turnstone keeps the kept and dropped commands in its own session view sidecar (`ViewIntentV1.command_menu`); Graphshell keeps them in pandect's `CommandMenuView`; both are views, not graph truth, as SE31 asks, but two stores hold the same thing. Options: leave it and note it (recommended: Turnstone's view is its own storage and it does not run on pandect's `GraphSession`); brief Turnstone to converge on `CommandMenuView` inside its sidecar; hoist the type out of pandect into Cambium beside `CommandSet`, with Graphshell and Turnstone repointing. Mark: **"Brief to converge and houst"**. *Follows:* both: the type is hoisted out of pandect into the Cambium family, and Turnstone's lane is briefed to store that type. *Reading, not ruled:* "houst" is "hoist". *Found after the answer:* cambium's `CommandChoices` already has the same three fields as `CommandMenuView` (`added`, `removed`, `recent`) without serde; the cambium crate has no serde dependency, and pandect does not depend on cambium (SE10 measured cambium at 46 crates for a build without it). Where the hoisted type lives is put back as SE46.

**SE46, where the hoisted type lives, and its name (2026-10-08).** Question 1: cambium's `CommandChoices` matches `CommandMenuView` field for field but has no serde, cambium has no serde dependency, and pandect does not depend on cambium (46 crates, SE10); both serialize to the same field names, so stored sessions keep loading. Options: a leaf crate holding the choices only (recommended); a leaf crate holding the whole set (`Command`, `CommandChoices`, `CommandSet`, with cambium keeping the drawing as `CommandItem`); pandect takes cambium. Mark: **"Think about it this way: what apps shouldn't have this capability? Commands feel pretty fundamental. Are choices (1) sufficient?"** Question 2, the crate's name. Options: `command-choices`; `command-set`; `command-menu`. Mark: **"command-menu"**. *Follows:* the crate is `command-menu`. Question 1 is answered by evidence and put back. *Reading, not ruled:* choices alone are not sufficient. Without the set, a host with commands but no cambium has to rebuild the composition (context first, then kept, then recent, with a query searching everything) to use the stored choices, and nine of the ten ports under `ports/` do not depend on cambium, nor does Graphshell's native build. Earlier portable runs at the same thing exist: graph-kernel's `ActionId` (`crates/graph/graph-kernel/src/actions.rs`), a closed enum of about 130 actions read by ux-events' diagnostics and probes, and mere-chrome's `CommandPaletteSession` (`crates/shell/chrome/src/command_palette.rs` *(historical citation)* <!-- doc-audit: historical-path -->, 332 lines), which no crate in mere's tree depends on. *Correction 2026-10-08:* "about 130 actions" is 68 (F12).

**SE47, what `command-menu` holds, where, and the older runs (2026-10-08).** Question 1: what goes in. Options: the whole set (recommended: `Command`, `CommandChoices` with serde, `CommandSet`, with cambium keeping the drawing as `CommandItem`); the set plus a palette session (the in-progress query, cursor and scope that mere-chrome's `CommandPaletteSession` held for the egui-era host), so a host only draws; the choices only. Mark: **"Set plus palette session"**. Question 2: where. Options: `crates/cambium/` beside `edit-history` (recommended); `crates/system/` beside pandect. Mark: **"crates/cambium/ (Recommended)"**. Question 3: graph-kernel's `ActionId` and mere-chrome's `CommandPaletteSession`. Options: note them and map them for later (recommended); fold them in during the hoist; leave them be. Mark: **"Note, map, then fold in depending on the results. We don't need duplicate commands or anything"**. *Follows:* track C2 (§2). `crates/cambium/command-menu` holds the command set, the stored choices and a palette session; cambium re-exports it and keeps the drawing; pandect's `CommandMenuView` becomes the crate's choices type. The two older runs are mapped against it first, and what folds in follows from the map, with no command defined twice.

**SE48, folding in by the map (2026-10-08).** Evidence put: F11. Question 1, one identity for a command. Options: shared ids in an open set (recommended: `command-menu` defines namespaced string ids for the verbs hosts share, hosts register those and add their own, `ActionId` keeps a typed enum for ux-events with keys from the shared ids); `ActionId` as the catalogue; retire `ActionId`. Mark: **"i think 1 makes sense with an audited actionid catalog for ux-events (we can't leave all those stale enums in there, it needs to adapt to current and future ux for the stack)"**. Question 2, the palette session. Options: shared plus context (recommended: query, a selected row with a wrapping step, focus the search, the context it opened over, expanded to all; position and drawing are the host's; mere-chrome's search scope stays out); shared plus a scope field; shared only. Mark: **"Shared plus context (Recommended)"**. Question 3, the older runs. Options: retire both in C2 (recommended: `CommandPaletteSession`, with `SearchPaletteScope` if nothing else reads it, and graph-kernel's two category persist keys); deprecate and retire later. Mark: **"Retire both in C2 (Recommended)"**. *Follows:* `command-menu` holds the shared verb ids; Graphshell and Turnstone register them for those verbs. `ActionId` is audited against the stack's current and coming surfaces before its keys move onto the shared ids: stale variants go, and what the audit finds missing comes back as a round. The palette session is as recommended. The two older runs are removed in C2.

**SE49, the audited catalogue (2026-10-08).** Evidence put: F12. Question 1, what `ActionId` keeps. Options: live, backed and planned, 28 once `node:delete` and `node:mark_tombstone` merge, dropping the 30 stale and 9 uncertain, the loose live ones renamed to today's verb (recommended); also keep the uncertain, 37; live and planned only, 12. Mark: **"1, and i am also willing to revise any and all commands to suit the stack properly."** Question 2, which host verbs get a shared id. Options: shared or watched verbs, a catalogue id when a second host offers the verb or ux-events reasons about it (recommended); every current host verb, about 150; only what ux-events watches. Mark: **"Shared or watched verbs (Recommended)"**. Question 3, `registry`'s `input::action_id`. Options: key bindings on the shared ids, its constants go (recommended); retire the input module; leave it. Mark: **"Key bindings on the shared ids (Recommended)"**. *Follows:* the catalogue keeps 28 verbs and gains the shared host verbs; registry's default bindings key on the catalogue's ids. *Reading, not ruled:* "revise any and all commands" frees the catalogue's ids, labels and namespaces, and the hosts' own ids, from their current spellings; the revised catalogue is drafted as a table and put back before any id moves.

**SE50, the catalogue's ids, the orphan bindings, old choices (2026-10-08).** Evidence put: F13. Question 1, the catalogue. Options: accept the draft, 33 ids in eleven namespaces named for what each acts on, `ActionId` keeping a typed variant per id with keys from `command-menu` (recommended); keep `ActionId`'s spellings for surviving verbs; namespace by surface. Mark: **"Accept the draft (Recommended)"**. Question 2, the 13 registry bindings with no shared verb. Options: drop all 13 (recommended); keep the ten host-only ones as host-owned ids. Mark: **"Drop all 13 (Recommended)"**. Question 3, choices stored under old host ids. Options: let them fall away, the set already ignoring unknown ids (recommended); alias old ids on load. Mark: **"Let them fall away (Recommended)"**. *Follows:* `command-menu` carries the 33 ids and their labels; `ActionId` becomes 33 variants keyed from them; registry's defaults keep the 21 bindings that land on catalogue ids and its own action-id constants go; Graphshell registers the catalogue ids for the verbs it shares; Turnstone is briefed to do the same at its repin; no alias tables.

**SE51, the Enter bindings, the button copy, landing C2 (2026-10-08).** Evidence put: the registry binds two more defaults than SE50 counted (36, not 34), Enter to `toolbar:submit` in the omnibar and to `graph:view_confirm` in the graph view, neither a shared command; Graphshell's menu shows catalogue labels while its buttons keep their copy ("Add address", "Undo change", "Redo change", "Edit selected object", "Pause physics"); C2 verified on its branch. Question 1, the Enter bindings. Options: keep them as the registry's only own ids, input-level actions and not commands (recommended); drop them; add them to the catalogue. Mark: **"Keep as input actions (Recommended)"**. Question 2, the button copy. Options: buttons use catalogue labels (recommended); buttons keep their copy; revise the catalogue labels. Mark: **"Revise the catalogue labels"**. Question 3, landing. Options: land now, then brief Turnstone (recommended); answer the above first. Mark: **"Land now, then brief Turnstone (Recommended)"**. *Follows:* the two Enter actions stay in `registry::input::action_id`; the labels the buttons say better are drafted into the catalogue and put back; C2 lands, and Turnstone's lane is briefed.

**SE52, the revised labels (2026-10-08).** Evidence put: the catalogue's labels beside Graphshell's button copy and Turnstone's wording. Question 1, `session:undo` and `session:redo`. Options: Undo change / Redo change (recommended: says what is undone, and reads clearly beside the projection undos); keep Undo / Redo. Mark: **"Undo change / Redo change (Recommended)"**. Question 2, `physics:toggle`. Options: Pause or resume physics, one fixed label with the button's state text a documented exception (recommended); labels by state; keep Play or pause physics. Mark: **"Pause or resume physics (Recommended)"**. Question 3, `node:edit`. Options: Edit selected node (recommended); Edit selected object; keep Edit node. Mark: **"Edit selected node (Recommended)"**. Question 4, `node:new` and the page's buttons. Options: keep New node, buttons for shared commands take the catalogue label (recommended); Add node, buttons follow; keep New node, buttons keep their copy. Mark: **"Keep New node; buttons follow (Recommended)"**. *Follows:* the four labels change in `command-menu`'s catalogue; Graphshell's shared-command buttons and painted pills show the catalogue label, held to it by a test; the physics button's state text is the exception. Built on branch `c2-labels` (`cddf72ac`), held off main until the Graphshell web host builds on main again (C2 Progress, the genet pin split).

### 1.5 The recipe grid rulings (2026-10-08)

Mark, after the two-reading matrix landed: "i wonder if it's possible to do a matrix of more than two arrangements, scenes, dynamics, idk. we have a compositional recipe method for building each, right?" Then: "open a round on the recipe grid". Evidence put: scenograph's `AuthoredProjectionDefinition` is a recipe of sections (sources, reading, encoding, arrangement, interaction, appearance) and `ProjectionVariant` v1 varies arrangement options only; dynamics is not a section, it composes per canvas in pictograph (`PhysicsComposition`, profiles); the projection grammar catalog lists faceted small multiples as a contract gap ("One score carries nested spaces and stable shared scales"; open item 4, "declared shared or independent domains"); conatus's `BodyWorld` is not a singleton, many at once unmeasured.

**SE53, what the grid is for.** Options: an editor tool built on a portable facet composition (recommended); an editor tool only; published small multiples. Mark: **"1. sounds like a swatch, to me. which we will need, as scoped or embedded mere reflections or projections"**. *Follows:* the grid's cells are swatches, and the swatch is needed beyond the grid, as scoped or embedded reflections or projections of a mere. *Reading, not ruled:* TERMINOLOGY's **swatch** is "a compact graph-canvas projection embedded in a pane", mirroring the main view or projecting through its own lens; *reflection* reads as the mirroring mode and *projection* as the own-lens mode; the scope model decision record (`mere_docs/design/2026-06-27_scope_model_reconciliation.md` §4) already names a "swatch preset" (the WHAT projection plus the HOW form factor), which is close to a recipe. Put back as SE57 onward.

**SE54, the axes.** Options: arrangement, then dynamics, the recipe gaining a dynamics section in its own round (recommended); arrangement only; any section; dynamics host-side. Mark: **"Arrangement, then dynamics (Recommended)"**.

**SE55, dynamics in cells.** Options: settled, one live on focus (recommended); settled only; all live. Mark: **"Settled, one live on focus (Recommended)"**.

**SE56, scales.** Options: declared per axis, shared by default (recommended); always shared; always independent. Mark: **"Declared per axis (Recommended)"**.

**SE57, what a swatch is made of.** Evidence put: cambium's `GraphCanvasSwatch` is a widget fed a host-built `GraphCanvasSubgraph`, outside the recipe and scene contract; the scope model record names a "swatch preset". Options: a portable swatch over a recipe, a scope, a mode and a recipe with its variant, composed by a facet, cambium's widget becoming one realization (recommended); the grid composes scenes and the swatch stays a widget. Mark: **"Portable swatch over a recipe (Recommended)"**.

**SE58, reflection and projection.** Options: a reflection follows another view's recipe and camera live, a projection carries its own recipe and variant, one grid may mix them (recommended); a reflection is the mere's data as authored, a projection applies a recipe. Mark: **"Reflection follows, projection owns (Recommended)"**.

**SE59, what may differ between cells.** Options: one scope, recipe axes (recommended); scope can be an axis too. Mark: **"Scope can be an axis too"**. *Follows:* a grid's axes may be scope as well as arrangement and then dynamics (SE54).

**SE60, the first home.** Options: Graphshell's projection editor (recommended); a gloss pane; an embedded mere on the site. Mark: **"1 and 2 are clear consumers"**. *Follows:* the projection editor and the gloss pane are both named consumers. *Reading, not ruled:* in the catalog's terms they are the first forcing consumer and the second heterogeneous one, and their order is put back.

Mark, with these: **"i am, of course, open to changing the concept to suit the stack as it is, not as it was months ago. but it seems a useful concept that needs restatement in the current terms."** *Follows:* TERMINOLOGY's **swatch** (wording ruled 2026-07-17) is restated in current terms and put back before it changes.

**SE61, the swatch restated.** Evidence put: TERMINOLOGY's entry as worded 2026-07-17: "a compact graph-canvas projection embedded in a pane: a scoped rendering of a graph or nested graph, either mirroring the main view (a minimap) or projecting through its own lens (independent layout, scope, or overlays; the gloss is a pane containing a swatch). A representation, never an identity: gnodes render in an orrery or swatch, while the graph itself lives in the kernel. A swatch over a servitor's nested graph is that servitor's inspection UI. Wording ruled 2026-07-17" Options: the draft (a scope and a mode; a projection owns a recipe that compiles to a scene, a reflection follows another view live; a portable fact a gloss shows, a facet composes and an embed carries; a representation, never an identity) (recommended); a shorter form keeping the old core; rename it. Mark: **"The draft as written (Recommended)"**. *Follows:* TERMINOLOGY's **swatch** carries the draft, and the nested graph entry's "a canvas representation" becomes "a representation".

**SE62, the consumers' order.** Options: the projection editor first, the gloss pane second (recommended); the gloss first. Mark: **"Editor first, gloss second (Recommended)"**. *Follows:* the projection editor's swatch grid opens the facet proof in the Graphshell web host; the gloss pane is the second heterogeneous consumer, bringing reflections and scope axes on a native host. Next: the portable swatch's shape and the facet contract, with the catalog's addition record, as their own round.

### 1.6 The swatch's shape (2026-10-08)

Mark: "proceed!" Evidence put: the June gloss design (`mere_docs/design/2026-06-07_gloss_navigator_design.md` §2a, §2b) already made the swatch "the portable, embeddable primitive configured by (scope, layout, lens, mode, filters)", with variants as points in that space, a view layer and an optional edit layer, embedded in facet panes, menus, djot notes and orrery cards, rendered as chrome-understood DOM; in current terms layout is the recipe's `Arrangement`, filters its `Reading`, the lens most of `Encoding` and `Appearance`, variants `ProjectionVariant`, the geometry a sceno `Scene`; scope has no portable type, forme has `SubgraphSpec` (nine kinds, anchors, selectors); sceno's `Scene` nests `Space`s; pandect names a view by `ViewKey`; June's "mode" is not SE58's.

**SE63, scope.** Options: a tagged scope over existing ids, `Node`, `Subgraph(SubgraphSpec)`, `NestedGraph`, `Mere`, reusing forme's spec, Reading still deciding how it is read (recommended); scope folds into Reading; an opaque host scope. Mark: **"A tagged scope over existing ids (Recommended)"**.

**SE64, the facet.** Options: one scene, a space per cell, axis labels as items, scales recorded per axis, through scenotime and the frozen and remote readers unchanged (recommended); separate scenes laid out by the host. Mark: **"One scene, a space per cell (Recommended)"**.

**SE65, the edit layer.** Options: view now, editing inside a swatch later as interaction intents through host authority (recommended); carry a view/edit flag now. Mark: **"View now, edit as an intent later (Recommended)"**.

**SE66, what a reflection follows.** Options: a pandect `ViewKey` or a sibling swatch in the same facet (recommended); only a view key. Mark: **"A view key, or a sibling cell (Recommended)"**.

**SE67, where `SubgraphSpec` lives.** Evidence put: forme pulls taffy, petgraph and uuid; scenograph depends on sceno and serde only; the site's Ruling 153 plans a view-curation leaf crate for `FoldRecord` in the native phase. Options: a serde-only leaf, the view-curation crate founded now by this lane in coordination with the site and graph-semantics' owner, forme re-exporting (recommended); into sceno; scenograph depends on forme. Mark: **"A serde-only leaf, forme re-exports (Recommended)"**.

**SE68, the types' home.** Options: Swatch, Facet and Scope in scenograph beside the recipe, a facet compiled by scenomise (recommended); all in scenomise. Mark: **"Types in scenograph, compile in scenomise (Recommended)"**.

**SE69, the recipe's dynamics slot.** Evidence put: the dynamics grammar plan's F21 ruled "Two slots, one binding" (a recipe names an arrangement and, optionally, a dynamics recipe); seiche's `DynamicsSpec` is the portable spec (F4); the authored recipe has no dynamics slot; seiche has no serde and F104 settles its carrier. Options: the dynamics lane builds the slot and the grid consumes it (recommended); this lane builds it to F21. Mark: **"Dynamics lane builds it, grid consumes (Recommended)"**. *Follows:* the physics coordinator is briefed; the grid's dynamics axis waits for the slot. *Annotation 2026-10-09:* the dynamics grammar plan ruled the slot's shape as F192 to F195 (mere `f89b229b`): an opaque, versioned `DynamicsSlot { version, spec }` on the recipe, scenograph keeping only sceno and serde (F192); the recipe's arrangement and the spec's target must agree or binding refuses (F193); a variant varies dynamics by a catalog preset id or a whole slot (F194); SE70's settle is a seiche function every host calls, with a native-against-wasm receipt, its default step bound to come back as a measured number (F195). The dynamics lane builds it; the grid takes the axis when it lands.

**SE70, settled frames.** Options: seeded, run from the spec's seed to its law's stop rule or a fixed step bound on libm maths (SE40), the settled positions written into the cell's scene (recommended); the host settles, best effort. Mark: **"Seeded, run to the spec's stop (Recommended)"**.

**SE71, "facet" (2026-10-08).** Mark asked: "wait wait, define a facet for me?" The answer given: the grid that lays swatches out along axes with a scale rule per axis, dataviz's facet (Vega-Lite, Observable Plot; small multiples, Cleveland's trellis), which collides with node facets (namespaced per-node data, 3,166 occurrences in 141 Rust files, stored formats `facets.json`, `gazette.facets/v1` and pandect's facet stores), `SubgraphKind::Facet` and the June design's facet pane. Mark: "hey, if this is a term of art, then i think that beats my improvised use of facet". Put back with the finding that node facets fit information science's faceted classification, also a term of art. Options: the grid takes "trellis", node facets keep facet (recommended); the grid keeps facet, node facets keep it as a code-identifier survivor with a new word in user copy; the grid keeps facet and node facets are renamed fully, stored formats migrating with old names readable. Mark: **"3, because i think the faceted classification we were doing came from pmest and that's not what this is now, so the dataviz facet has stronger provenance. we might as well call node facets anything. but have suggestions?"** *Follows:* the grid is the facet; node facets are renamed everywhere, the stored formats migrating with the old names still readable; the new name is put back. *Reading, not ruled:* the kernel's `facet_projection.rs` ("PMEST facet projection") is the classification sense Mark retires, and stored facets ("arbitrary, namespaced" per-node data, `apply.rs:261`) are what is renamed.

**SE72, node facets' new name (2026-10-08).** Evidence put: node facets are a node's namespaced data records set through the delta spine; the candidates' counts in mere's Rust: attribute 517 in 157 files, property 318, characteristic 0, extension 588. Options: attribute, the entity-attribute-value model's term (recommended); property; characteristic; extension. Mark: **"attribute... but lemme run something weird by ya. what about tag? isn't that exactly data on an entity? i think the resource node concept seems to cover a lot of what we needed facets for anyway. maybe an expanded tag system and resources is enough."** *Follows:* put back with an inventory of the 25 or so declared facet keys: claims about what a thing is (`semantic.*`, `chartulary.class`, `provenance.*`), labels (`presentation.tags`), how a node was met or is viewed (`arrangement.*`, `visit.history`, the access and browser histories, `pinned-projection`), and whole app documents at a node (`saved-scene/v4`, `projection-definition/v1`, `content/v1`, `local-file/v1`, the transfers, `receipt.*`, gazette's contact record, the relationship recipe, `denizen.binding`, `personae.vault-root/v1`).

**SE73, the curation crate's name.** Options: curation (recommended); view-curation; from the word list. Mark: **"curation (Recommended)"**. *Follows:* `crates/.../curation`, serde only, `publish = false`, holding `SubgraphSpec` and `SubgraphKind` now and `FoldRecord` in the native phase.

**SE74, graph-semantics' owner.** Options: this lane coordinates; resume it as its own lane; someone else holds it. Mark: **"the codex agent on the thinkpad is doing graph-semantics."** *Follows:* the native-phase coordination of the site's Ruling 152, and anything touching resources and assertions (SE72's put-back among them), goes through that agent, by way of Mark or the graph semantics plan, since this session cannot message it.

**SE75, dissolving node facets (2026-10-08).** Evidence put: SE72's inventory; claims fit resources and their assertions, labels are tags already, view state and whole app documents are versioned structured JSON that tags cannot carry without becoming a document store. Options: rename to attribute now and redistribute separately (recommended); redesign first and rename what is left; tags take everything. Mark: **"2; we could use attribute as an abstraction belonging to or characteristic of an entity sounds like view state to me. attribute, resource, tag... dissolving into those three sounds good to me"**. *Follows:* node facets dissolve into three: **attribute** (view state, what belongs to or characterizes an entity), **resource** (claims about what a thing is, through graph semantics' assertions), and **tag** (labels). The rename waits on that design; meanwhile the swatch grid's facet coexists with node facets in code. The design goes to graph-semantics' owner (SE74) as `mere_docs/research/2026-10-08_facet_dissolution_brief.md`, handed over by Mark. *Reading, not ruled:* whole app documents at a node (`saved-scene/v4`, `projection-definition/v1`, `content/v1`, `local-file/v1`, the transfers, `receipt.*`, contact records) become resources, as content the node shows; the brief puts this as its first question.

**SE76, the curation crate's place (2026-10-09).** Mark: "Go" (S1). Evidence put: its types come from forme (`SubgraphSpec`, later `FoldRecord`); forme's family holds forme and uxtree, the graph family the kernel and linked-data. Options: `crates/forme/curation` (recommended); `crates/graph/curation` *(historical citation)* <!-- doc-audit: historical-path -->. Mark: **"crates/forme/curation (Recommended)"**.

**SE77, its package name.** Options: `curation`, plain, as edit-history and command-menu (recommended); `mere-curation` with lib `curation`, as mere-forme and mere-kernel. Mark: **"mere-curation, lib curation"**.

**SE78, publishing it.** Evidence put, against SE73's `publish = false`: `mere-forme` is `publish = true` and on crates.io at 0.1.0, in pandect's dependency closure, so its next publish needs curation published first; `mere-curation` was free. Options: publishable, released with forme's next publish (recommended); claim the name now, confirming the crate's contents first; keep `publish = false`. Mark: **"Claim the name now"**, then **"I authorize the publish"**. *Follows:* `mere-curation` 0.1.0 is published with `SubgraphSpec` and `SubgraphKind`, the spec gaining `PartialEq` and `Eq`, and SE73's `publish = false` is amended. *Reading, not ruled:* `SubgraphKind::Facet` keeps its name until the facet dissolution design (SE75) reaches it.

**SE79, what fills the editor's grid (2026-10-09).** Evidence put: the projection editor edits one draft with no saved variants; scenomise has eleven built-in arrangement families. Options: the families, the working draft first as a reflection (recommended); authored variants; families now, variants next. Mark: **"Arrangement families, working draft first (Recommended)"**.

**SE80, where it appears.** Options: in the preview, toggled Preview | Compare (recommended); under the arrangement rows; on the main canvas. Mark: **"In the preview, toggled (Recommended)"**.

**SE81, picking a cell.** Options: apply it as one undo step through E2's history, the grid staying open (recommended); focus, then apply with a button. Mark: **"Apply it, one undo step (Recommended)"**.

**SE82, a scope axis in the editor.** Options: arrangement only in the editor, scope arriving with the gloss (recommended); both now. Mark: **"Both now"**. *Found after the answer:* the editor compiles a host-supplied `ProjectionDataset` (source, revision, fields, occurrences; no relationships), three occurrences in the practice fixture; every `Scope` variant names graph structure the dataset lacks, and `SubgraphKind` has no plain selection kind. Put back as SE83.

**SE83, the editor's scope rows.** Evidence put: SE82's finding. Options: two rows, every occurrence and the occurrences selected, `Scope` gaining a `Selection` of explicit ids that the gloss can reuse, the selection row hidden until something is selected (recommended); defer scope to the gloss; give the editor graph datasets first. Mark: **"Whole set and the selection (Recommended)"**. *Follows:* `Scope::Selection(Vec<String>)`; the editor's selection is one occurrence today, so its row shows that one until multiple selection exists.

### 1.7 The sandbox features (2026-10-09)

The mer3ly site session relayed its Ruling 158 (Mark: "Promote where a home exists"): the `/repos/` graph sandbox's scatter, deck and facets map onto the swatch grid and its field and tangible backdrops onto L2's backdrops, each a stack capability this lane owns, the sandbox keeping them until they land. Evidence put, read from mer3ly's `assets/graph-sandbox.js`: scatter and deck are further appearances of the same sources (`buildRepeatedAppearances`), swatches under other recipes; the sandbox's "facets" are selected parts of one appearance (`{view, source, facet}`, facet one of cell, heading, summary or status), saved in the shelfmark as `mer3ly.facets`, the sense SE71 retired; its views are linked by a crossfilter selection and the deck dismisses one appearance as an instance delta (`visible: false`); S1's grid is view only (SE65); sceno's `Backdrop` carries a kind and `collidable`, pictograph paints kinds and collidable edges, the Graphshell viewer offers no backdrop mode, and "field" has no definition (the catalog's field rasters wait on a field consumer).

**SE84, the site's facets.** Options: into a linked-selection track, renamed, the shelfmark key migrating (recommended); drop them; keep them site-local. Mark: **"Into a linked-selection track (Recommended)"**. *Follows:* appearance-part selection joins S2 under a new name, put to Mark there.

**SE85, linked swatches.** Options: track S2, a facet's cells sharing a selection and hiding one appearance per cell, as host intents (recommended); selection only; not now. Mark: **"Track S2: linked swatches (Recommended)"**.

**SE86, backdrops.** Options: track B1, the Graphshell viewer drawing a scene's backdrops with a mode control and tangible backdrops as physics obstacles, clear, ambient and props now and field once defined (recommended); all four now; fold into the swatch work. Mark: **"Track B1, field later (Recommended)"**.

**SE87, handing the next work to Codex (2026-10-09).** Mark: "could we hand it to codex? IDK, it's friday and i've got 25% usage left ;_;" Options: S2 and B1 (recommended); the dynamics slot (F192 to F195); all three. Mark: **"All three"**. *Follows:* `mere_docs/research/2026-10-09_scenograph_codex_handoff.md` hands the dynamics slot, the grid's dynamics axis, B1 and S2 to the Codex agent, in that order; the physics coordinator is told the slot moved.

**SE88, the appearance-part name (2026-10-09).** Asked: "For linked swatches, what should a selection of an appearance part be called in the UI? The handoff leaves this name open." Options: Appearance part (recommended), Facet, Part. Mark: **"Appearance part (Recommended)"**. *Follows:* the user-facing name is "appearance part". *Reading, not ruled:* the migrated shelfmark section can use `mer3ly.appearance_parts`; readers should retain the old `mer3ly.facets` value when migrating.

**SE89, physics bodies follow occurrences (2026-10-09).** Asked: "The practice grid has source occurrences, while Canvas physics uses graph nodes. Should each occurrence get its own preview body, with unavailable physics channels refused until an adapter supplies them? This preserves separate appearances of the same source." Options: Separate body per occurrence (recommended), Limit dynamics to graph-backed datasets. Mark: **"Separate body per occurrence (Recommended)"**. *Follows:* each preview occurrence has a separate transient body; an undisclosed physics channel is refused. The source identity remains the dataset's exact source reference. *Reading, not ruled:* a private per-cell controller may give each body a deterministic runtime UUID without writing those identities into the source dataset.

**SE90, reset, context and scenery (2026-10-10).** Mark corrected the proposed
Clear / Ambient / Props control, which had made visibility and collision
consequences of the selected mode. His answer, verbatim:

> i feel like clear is better as reset, like resetting the scene to its initial arrangement w/rt your backdrop (which should be able to be interactive, or at least animated, right? that in my mind makes it more of the backdrop of a scene's set, or almost diorama-esque, compared to a static wallpaper), but ambient content and props are both more like classes of entity in the scene; ambient nodes are like propositions for what's relevant to the entities in the foreground, which are either pinned, currently selected, or recently interacted with. you can turn that off, of course, but that's not the same thing as saying they do or don't interact with the scene. and props, likewise, can be turned off; you could have a static wallpaper, an animated wallpaper, an interactive wallpaper, or an interactive wallpaper that contains entities with scripted behavior, authored attributes, or presentation relevant to the backdrop, aside from but relevant to contextualizing the nodes in the foreground and the ambient nodes in the background in the backdrop and scene. but that's not the same as saying props are tangible or not. you have the wrong cut for what matters on that front.

*Follows:* SE86's three-mode wording is amended. Reset is an action restoring
the initial arrangement in relation to the backdrop. Ambient context and props
can coexist and have separate visibility choices. Neither role nor visibility
determines collision, animation, picking or authored behavior. The backdrop
may be a scene's animated or interactive set; static geometry is one supported
part, not the complete backdrop model. F199's pause and R162 ownership remain.

*Reading, not ruled:* keep contextual graph occurrences separate from scenic
entities even when both have a visible presentation. Reuse existing scene,
simulation and interaction owners before adding a portable behavior carrier.
The reset boundary still needs a concrete definition: arrangement baseline,
body velocities, animation time and script state are distinct state owners.
The current `restore_arrangement` restores stored positions and pauses physics;
it does not reset a whole animated set. The foreground/background membership
question raised here is resolved by SE91 below. No recency cutoff is chosen.
Field remains deferred; interactive/scripted scenery has not been
implemented by this ruling.

**SE91, recede but retain after interaction (2026-10-10).** Mark asked whether
interacted nodes should remain foregrounded or recede into the background but
persist when focus shifts. Recommendation put: selected nodes occupy the
foreground, including a working multi-selection; after deselection, previously
interacted nodes persist in the background until dismissed; explicit foreground
pins retain their foreground role across focus changes. Untouched ambient
suggestions may change with context. Recency can affect emphasis without
retaining foreground status. Mark: **"Hm. Alright."**

*Follows:* this is the default promotion/demotion behavior and reconciles SE90
with §9.2. Moving focus within a still-selected working group does not by itself
demote its selected members. Deselecting an unpinned node returns it to the
background without discarding it. Attention/presentation changes do not
implicitly change collision, animation, scripts or durable keeping levels;
a foreground pin remains distinct from a position pin. The recommendation also
offered an optional setting to retain the working selection across context
changes. Its UI, persistence and the exact boundary of qualifying interaction
remain to be specified. No automatic hover retention or recency timeout is
introduced. This ruling records behavior, not viewer implementation.

### 1.8 The rule runner's rulings (2026-10-15)

Evidence put after the repin (see the 2026-10-15 annotation above): the landed
`ValueEngine` matches branch `value-engine`; `crates/script/rhai` (264 lines)
has `base_engine()` over rhai's standard packages, which load the float-maths
package SE40 forbids, and no `script-engine-api` or `libm` dependency; this
plan carries no rule-runner task list. Mark answered six questions in a row.

**SE92, the runner's shape.** Options: (a) rhai expressions inside the declarative
`RuleSet`, as a script `Condition` and perhaps a script `Effect`, keeping the
resolver, precedence, traces and hysteresis; (b) whole rules as rhai scripts that
return effects; (c) region and field rules first (field plan P4). Mark: **"a"**.
*Follows:* the runner is a `Condition::Script` in `scenograph::presentation`,
evaluated through `ValueEngine` only (SE39). A script `Effect` is not ruled in
and is not built here. *Reading, not ruled:* a script that fails, runs past its
budget or returns a non-bool leaves its rule unmatched with the reason in the
trace, so one bad rule does not stop the rest; and `resolve` without an engine
refuses a set that carries a script condition.
*Annotation 2026-10-15:* SE99 rules the failure reading; SE98 puts the running
of scripts, and the `script-engine-api` dependency, behind a cargo feature.

**SE93, dependencies.** Question: may `libm = "0.2"` join the workspace
dependencies and `script-rhai`, and `script-engine-api` join `script-rhai`
(neither downloads). Mark: **"yes"**. *Follows:* both are added; `libm` without
default features. *Reading, not ruled:* `scenograph` also takes
`script-engine-api` (the trait only; no rhai).

**SE94, the maths host functions.** Options: the probe's exp, sin, cos, sqrt, pow;
those plus tan, atan2, ln, floor, abs, min and max, all on `libm`. Mark:
**"sure"** (to the recommended larger set). *Follows:* `script-rhai` registers
that set on request; nothing loads rhai's maths package (SE40).

**SE95, scope across evaluations.** Options: persist `let` bindings between
`eval` calls; a fresh scope per eval with only globals persisting. Mark:
**"fresh scope"**. *Follows:* a rule cannot leak state into the next.

**SE96, registering a name twice.** Options: refuse; replace. Mark: **"refuse"**.
*Follows:* `register` returns `ValueError::Host` for a name already registered.

**SE97, where the determinism test lives.** Options: (a) in tree, a native test
that hashes `to_bits()` against a stored constant, with the wasm32 run a script
invoked under Node; (b) keep the probe harness outside the tree. Mark: **"a"**.
*Follows:* the test, the example it shares code with and the script live under
`crates/script/rhai`; any `getrandom` setting wasm32 needs stays in that
crate's own build, not the root manifest.

**Review of the first build (2026-10-15).** Four more questions went to Mark
with the built track R2. The coordinating session relayed his chosen options
verbatim; the other options are as put there.

**SE98, publishing.** Evidence put: `scenograph` gained a git dependency on
`script-engine-api`, which a crates.io release of `scenograph` cannot carry
until Genet publishes that crate. Options: accept the block; feature-gate the
dependency. Mark: **"Feature-gate it"**. *Follows:* `script-engine-api` is an
optional dependency of `scenograph`, behind the cargo feature `scripts`, off by
default; `RuleSet::resolve_with_scripts` and the evaluation code exist only with
it, and `script-rhai` enables it for its tests. *Reading, not ruled:* the
`Condition::Script` variant itself stays in the data model under every build,
because an enum variant behind a feature changes what an exhaustive `match` in a
dependent crate must cover with whichever crate in the graph unified the feature
on; without the feature `resolve` refuses a set carrying one, and `validate` and
serde treat it as any other condition.

**SE99, a script that fails.** Options: leave the rule unmatched with the reason
in the trace; fail the whole resolve. Mark: **"Rule unmatched, traced
(Recommended)"**. *Follows:* a script that errors, runs past its budget or
returns a non-bool leaves its rule unmatched, with the reason in the trace. This
turns SE92's reading into a ruling.

**SE100, a missing fact.** Options: keep rhai's behavior, which mirrors `Not`;
make a missing fact an error. Mark: **"Keep, mirrors Not (Recommended)"**.
*Follows:* an absent fact reads as `()` and `facts.weight > 2.0` is false;
presence is checked with `"weight" in facts`.

**SE101, push.** Options: check Graphshell, then push; push now; hold. Mark:
**"Check graphshell, then push (Recommended)"**. *Follows:* after SE98's change,
`cargo check -p graphshell`, then rebase, re-run the touched tests and push to
main.

## 2. Tracks

E1 and E2 carry ruling A; E3 to E5 carry ruling B. E2 needs E1; E4 needs E3; E5 needs E2 and E4. *Added 2026-10-07:* E2b carries SE11 and SE12, after E2.

### R1 — presentation rules and the first aesthetic study (2026-10-09)

Mark: "Target->condition->effect, rhai, etc. all agreed. Proceed". The
[design language record §9.10](../../2026-08-23_projection_scenes_and_graph_native_platform.md#910-presentation-rules-agreed-2026-10-09)
owns the product direction and the six accepted examples.

Scope for this first slice:
- A host-neutral, serializable rule set in Scenograph, using the existing
  `sceno::Representation` slots. Conditions read disclosed scalar facts and
  occurrence-local view state; effects resolve presentation without source edits.
- Explicit higher-priority-first property resolution, with later authored
  rules winning equal priorities. Explain matches, missing inputs and overridden
  effects. Validate the whole set before applying it.
- Compact/detail screen-size hysteresis with prior match state held by the host
  per rule and occurrence. Resolve representations before measurement; no rule
  here moves an item, changes a pin, or conflates foreground and position pins.
- A mixed-content interactive study, with independent editable theme and
  arrangement choices, node selection, retained background membership, pinning,
  forme guides, link inspection, rule editing and match explanations.

Done when meaningful tests cover precedence, missing facts, malformed sets,
occurrence identity, hysteresis and round-trip storage; the browser study
agrees with the Rust resolver on shared fixtures; and headed checks verify rule
changes, themes, selection/deselection, pins and narrow-screen layout.

Rhai remains the rule language for the production scene/region runner under
SE36–SE42. This first declarative slice adds no replacement scripting seam.
The study exercises the model locally; its browser resolver is a study host,
not production Graphshell adoption. Broader style editors and live resource
rendering remain with their existing owners.

### R2 — the rule runner (SE92 to SE97)

**Status (2026-10-15):** implemented on branch `rhai-valueengine`. Based on mere
`90ce8f12` (Genet `ae3d9f5`). Mark's SE92 to SE101 rulings (§1.8) fix the shape;
this track is their done-conditions.

Scope:
- `script-rhai` implements Genet's `ValueEngine` as `RhaiValueEngine`, a raw rhai
  engine with only the language, iterator, array and map packages (SE39, SE40),
  the sandbox of `base_engine()` re-applied from one shared function, a fresh
  scope per evaluation (SE95) and a refusal for a name registered twice (SE96).
- `register_libm_maths` supplies `exp`, `ln`, `sin`, `cos`, `tan`, `atan2`, `sqrt`,
  `pow`, `floor`, `abs`, `min` and `max` on `libm` (SE94).
- `scenograph::presentation` gains `Condition::Script`, always present as data,
  and, behind the `scripts` feature (SE98), `RuleSet::resolve_with_scripts`
  (SE92). Scenograph depends on the trait only. `resolve` without an engine, or
  without the feature, refuses a set carrying a script condition.

Done when, and what holds it:
- Engine behavior: values cross both ways, including nested maps and
  non-finite floats; built-in maths is absent, with a positive control that the
  same call resolves once registered; a runaway is `ValueError::Budget` and the
  engine survives; an empty budget runs nothing; failures come back as script,
  host, value and budget errors. `crates/script/rhai/src/value.rs`, 17 tests; 16 were
  seen failing against a stub first and the 17th (the sandbox) passed against it.
- Rule maths is bit-identical on native and wasm32: `crates/script/rhai/tests/maths_bits.rs`
  holds the native build to `028dc38fdcbf650d` (F8's recorded prefix) and
  `crates/script/rhai/tests/wasm_maths_bits.mjs` builds
  `crates/script/rhai/examples/maths_bits.rs` for `wasm32-unknown-unknown`, runs it under
  Node 24 (V8) and compares against the same constant, with controls that the
  module ran the rule.
- The runner, both ways (SE98): `crates/cambium/scenes/scenograph/src/presentation.rs` with the
  feature off (25 tests, two of them new: refusal without an engine, validation
  and round trip) and on (29: four more on a scripted stand-in engine, for
  precedence, the globals a script sees, failure isolation and composition under
  `All`/`Any`/`Not`) and `crates/script/rhai/tests/scenograph_rules.rs` (8 tests on real rhai: facts and
  states, a missing fact, hysteresis written in the script, host maths, a
  narrower binding set, a runaway, no state between occurrences, repeatability).

Not done here, and not claimed:
- A script `Effect` (SE92 said "perhaps"; not ruled in).
- Graphshell adoption: its rule editor's vocabulary offers no script condition, and
  `BoundContext::resolve` calls the engine-less `resolve`, so a set with a script
  is refused there with an issue. No host registers an engine yet.
- Which bindings a shared scene's rules get (SE36's "narrower binding set"). The
  mechanism is the engine's registrations, tested with and without maths; the
  policy per trust tier is not decided.
- Region and field rules (the field plan's P4) and piccolo's `ValueEngine`.
- Browser parity: the study host's JavaScript resolver does not know the
  `script` condition.
- Publishing is unblocked by SE98: the default `scenograph` build has no
  `script-engine-api` in its tree (`cargo tree -p scenograph -e normal`). A
  release with the `scripts` feature on waits for Genet to publish the crate.

### E1 — one history in Cambium (A, SE5)

Generalize `EditHistory` (`crates/cambium/cambium/src/editor.rs`) into a history over any `Clone` snapshot.
- Two stacks with a depth cap, as today.
- Two coalescing policies: the text field's insert run (today's behaviour), and an edit key held within a window measured on a time the host passes in. Cambium keeps no clock of its own (§4, F2).
- A saved marker: the host marks the current position saved, and the history reports dirty when the position differs. Undoing back to the save reads clean. A saved entry that falls off the cap leaves the history dirty until the next save.
- `TextInput` moves onto the generic type. *Reading, not ruled:* the window's length is a parameter the host sets, with balaur's 400 ms as the default.

Done when:
- `TextInput`'s existing undo and redo tests pass unchanged;
- new tests cover: coalescing within the window, a break on a new key, a break when the window lapses, clean after undoing back to the save, and dirty after the saved entry falls off the cap;
- `cargo test -p cambium` passes. Nothing outside Cambium uses `EditHistory` (§4, F2), so no consumer changes.

### E2 — undo in the projection editor (A, SE6)

`ProjectionEditor` (`ports/graphshell/src/projection_editor.rs`) keeps a `History` of (draft, selected panel).
- Every reduce that returns `Changed` records before it mutates, keyed by the action's field. `SelectPanel` and workbench gestures record nothing.
- Undo and redo restore the draft and the panel. The caller recompiles the preview, as it already does after every edit.
- `save` marks the history saved; the editor exposes dirty.
- A fresh `ProjectionEditor` (a new or loaded draft) starts a fresh history.

Done when:
- Graphshell's tests cover undo, redo, the panel return, coalesced field edits, and dirty across a save;
- Graphshell's web host binds undo and redo through its command surface and shows dirty;
- a headed check in real Chromium (not the browser pane) edits, undoes, redoes and saves, with the capture under `Code/testing/mere/`.

### E2b — saves go into the mere session (SE11, SE12)

The web host's `ProjectionDefinitionSink` writes the definition into the mere session through Graphshell's `MereHost` (pandect), in place of browser localStorage, and reads it back from there on load. Saved changes are then attributed, durable and undoable through the session.

Done when:
- the localStorage sink is gone, and a saved definition reads back from the session after a reload;
- a session undo after a save restores the previous saved definition, attributed to its author;
- a headed check in real Chromium saves, reloads and finds the definition.

*Amended 2026-10-07 (SE14 to SE17):* the definition lives at `mere://projection/<id>` under a `graphshell.projection-definition/v1` facet; the executable path moves too, its facet holding the selected occurrence; each save persists at once; and the web host gains session Undo and Redo, which the headed check drives. *Amended again (SE18 to SE21):* two controls, the editor's Undo save over the `graphshell.projection-editor` channel and a host-wide pair in Graphshell's chrome; an undone or redone save reloads the draft as one step; kept parts are named in the status.

### C1 — drag to pan, right-drag select, the command context menu (SE23 to SE30)

Built and landed together (SE27).
- **Canvas** (`crates/canvas/pictograph`): a left press on empty canvas becomes a pan once it moves past the click slop, with the middle-drag's momentum; a bare left click there keeps today's edge pick or clear. A right press begins a rectangle select that commits past the slop; a right click within it asks the host for its menu.
- **Command set** (Cambium): registered commands with ids, labels, categories and the surfaces they belong to; defaults, a recency ring, and the person's adds and removes; ranked by where the menu was summoned. The context menu takes the search field the palette has.
- **Graphshell**: registers its page commands; a right-click opens the menu at the cursor, node commands first on a node; recents and changes are kept in the mere session; the native menu never shows on the canvas.

Done when:
- the canvas tests cover the pan, its momentum, the bare click, the right-drag select and the right click;
- the command set's tests cover ranking, recents, adds and removes, and search over every command;
- a headed check drags to pan, right-drags to select, and right-clicks to a menu whose search finds a command, in Chrome and Firefox.

### C2 — `command-menu`, one home for commands (SE45 to SE47)

- **The `ActionId` audit (SE48).** Each of the 68 against Graphshell web's commands, Turnstone's catalogue and ux-events' readers: live, stale, or missing; the result is a round before any variant moves.
- **Map first.** graph-kernel's `ActionId` and its readers, mere-chrome's `CommandPaletteSession`, Turnstone's palette state and Graphshell's web menu state, each set against the set and the session: what each carries, what overlaps, what would be defined twice. What folds in, and how, comes back as a round.
- **The crate** (`crates/cambium/command-menu`, serde its only dependency): `Command`, `CommandChoices` (serde, the same field names as `CommandMenuView`), `CommandSet` with `menu` returning commands, and a palette session (query, cursor and whatever else the map shows a palette needs). cambium re-exports it and keeps the conversion to `CommandItem`.
- **pandect** stores `CommandChoices` where `CommandMenuView` was; Graphshell repoints.
- **Turnstone** is briefed to store the same type in its sidecar and read its palette state through the session, at its next coordinated repin.

Done when:
- the set's five tests and the session's tests pass in the new crate, and cambium's tests pass unchanged in count;
- a session stored before the change, with `CommandMenuView`, loads unchanged (a test over its JSON);
- no command is defined in two places, by the map's own list;
- the Graphshell web scenarios `canvas_commands.scn` and `canvas_commands_reopen.scn` pass headed in Chrome and Firefox.

### S1 — the swatch grid (SE53 to SE70)

The projection editor's grid of swatches, the catalog's facet proof (first forcing consumer; the gloss pane is the second, SE62).
- **The leaf crate (SE67).** `SubgraphSpec` and `SubgraphKind` move out of forme into a serde-only view-curation crate; forme re-exports them, so no caller changes. Its name comes back as a question first.
- **The types (SE63 to SE66, SE68).** In scenograph: `Scope` (`Node`, `Subgraph(SubgraphSpec)`, `NestedGraph`, `Mere`), `Swatch` (a scope, and either a projection's recipe and variant or a reflection naming a `ViewKey` or a sibling cell), `Facet` (axes of scope and arrangement, each declaring shared or independent scales, shared by default). View only; editing waits for an intent.
- **The compile (SE64, SE68).** scenomise turns a facet into one `Scene`, a `Space` per cell, axis labels as items, scales recorded per axis; it passes scenotime and the frozen and remote readers unchanged.
- **The editor's grid.** Graphshell's arrangement panel shows the recipe's variants as cells, settled (SE55, SE70) with the focused cell live; picking a cell applies it through E2's history.
- **Dynamics axis.** Waits on the dynamics lane's recipe slot (SE69).

Done when:
- the leaf crate builds alone and forme's callers compile unchanged;
- a facet over two arrangements and two scopes compiles to one scene that round-trips through serde and replays through scenotime, its frozen form listing every cell;
- shared and independent scales differ in a test where they should, and a control fails when a cell escapes its declared scale;
- a settled cell is identical across two runs and across native and wasm;
- the catalog carries the facet's and the swatch's addition record;
- a headed check in Chrome and Firefox shows the grid, focuses a cell live, and applies a variant with undo.

### S2 — linked swatches (SE84, SE85)

*Ownership checkpoint, 2026-10-09:* the site's R162 now assigns the viewer
files to the site lane for its cutover. S2 and B1 coordinate through that
owner; the dynamics editor draft is not a qualification of either track.

A facet's cells act together, so the mer3ly sandbox's scatter, deck and linked views move onto the stack.
- **Shared selection.** A facet carries a selection of clauses per cell (crossfilter or highlight, as the site's `selection.clauses`), which every cell reads; picking routes as a host intent.
- **Appearance-part selection.** Selecting a part of one appearance (the site's cell, heading, summary, status) is a clause of its own, under a new name put to Mark; the site's `mer3ly.facets` shelfmark key migrates to it.
- **Per-cell visibility.** A cell may hide one appearance while its source stays (the deck's dismiss), as a visibility delta on that cell.
- **Consumers.** The projection editor's grid and the site's sandbox (scatter and deck as cells).

Done when: the selection and visibility round-trip in a facet and replay through scenotime; a cell's hidden appearance leaves the source and the other cells untouched; the site's sandbox state maps onto it without loss; a headed check links two cells' selection in Chrome and Firefox.

### B1 — backdrops in the Graphshell viewer (SE86, amended by SE90)

The viewer draws a scene's set, with independently visible ambient context
and props. Reset restores the initial arrangement in relation to that set.
Visibility, collision, animation, picking and authored behavior are separate
choices; ambient and props are not mutually exclusive modes. Field waits until
defined. A collidable backdrop footprint is an obstacle to physics, regardless
of its visibility or scene role.

The former three-mode done-condition is superseded. The bounded static backend
still needs viewer drawing, a disclosed collision edge and the tangible versus
intangible exclusion control. SE91 resolves foreground/background retention.
Before viewer controls land, resolve the reset baseline and the interaction
boundary for retention, then define the saved scene state
and animated/interactive set slice against their existing owners. Acceptance
must cover independent visibility and collision, saved-state round trips and
headed Chrome and Firefox checks. Static obstacle acceptance alone does not
establish animated or interactive scenery.

**Cross-lane control design:** [design language §9.13](../../2026-08-23_projection_scenes_and_graph_native_platform.md#913-backdrop-context-props-and-reset-2026-10-10)
records the same correction. Separate reset, contextual-content display,
prop display, scenery behavior and physical participation. Field effects may
coexist with scenery and contextual content. Exact control names, off/pause
semantics, persistence and legacy-mode migration go through the viewer owner.
Define the reset baseline and its effects before implementation; the label
Clear alone does not authorize state rewind. Interactive props need explicit
entity/input/action wiring; animated or scripted reset needs its own behavioral
qualification. Existing paint/contact receipts qualify only the minimum binder.

**B1 backend draft (2026-10-10):** `Canvas::set_projection_backdrops` resolves
a scene's spaces and transforms for graph-canvas paint and static contact.
Unknown appearance kinds use the existing stable kind paint. Invisible
collidable geometry remains an obstacle. Concave contact polygons and
contact paths/points refuse by path pending a footprint adapter; a refusal
retains the previous bound layer. The separate Seiche obstacle set leaves
living scenery, gravity and node tangibility intact. This is implementation
work, not B1 acceptance. Viewer controls, scene-state persistence and headed checks
remain open under R162's site ownership. SE90 supersedes the pending mode
fork and the sandbox's three-mode cut; the backend does not implement that
expanded set or determine contextual membership.
The retained short 120-step probe leaves residual overlap. Rapier caps
correction at 3 world units per second; the active-law/control check uses
the canvas's existing 360-step settle budget. All four focused native
binder tests pass, and Seiche's library suite passes 161 with 10 ignored.
Larger/deeper or idle placements retain an explicit qualification limit;
the exclusion tolerance and host settle budget are unchanged.

### E3 — the option declaration type (B, SE7)

A declaration type in `crates/cambium/scenes/scenograph`: per option, its key, one of the seven kinds (`finite`, `positive`, `count`, `depth`, `flag`, `choice` with its names, `list`), its default, and a label for the editor.

Done when the type serializes deterministically, round-trips, and has tests. No reader uses it yet.

### E4 — families and solvers declare (B, SE8)

- Each of the eleven families in `scenomise/src/catalog.rs` declares its options as data.
- `Options` checks every read against the family's declaration, and `finish`'s refusal comes from it.
- `SolverCapability` gains an options list, empty by default. A custom solver's undeclared key is refused at compile, before solving. The test solvers in `registry.rs` and `projection_catalog_tests.rs` are the only implementations (§4, F4).

Done when:
- a test fails if any family reads a key it does not declare, or declares one it never reads;
- refusal messages are unchanged for the built-in families;
- a custom solver's undeclared key is refused at compile;
- `cargo test -p scenomise -p scenograph` passes.

### E5 — rows from the declaration (B)

Graphshell's arrangement panel builds one row per declared option of the chosen family or solver: its kind, its default and, for `choice`, its names. Edits go through E2's history.

Done when the rows match the declaration for all eleven families and a test solver, and a headed check in real Chromium shows a family's rows and a refusal.

*Annotation 2026-10-07 (SE10):* E1's `History` lives in `crates/cambium/backstep` (then `edit-history`); the cambium crate depends on it and re-exports `History`, and `EditHistory` stays in `crates/cambium/cambium/src/editor.rs`.

## 3. Mapped, not opened

**The Turnstone brief (SE32, SE35), sent to its unusual-protocols lane 2026-10-07.** To take at its next coordinated repin past mere `16c1ef8d`:
- **What C1 offers.** `cambium::CommandSet` (register commands with an id, a label and a category; `menu(choices, context, query)` puts the context's commands first, then the kept ones, then recent ones; a query searches every command; `record_use`, `add`, `remove`). pictograph's canvas pans on a left-drag over empty canvas, selects on a right-drag, and leaves a `ContextRequest` on a right click within the slop (`take_context_request`). pandect's `ViewIntent` has `commands: Option<CommandMenuView>`, and `set_view_now` stores a view in the next batch.
- **What Mark ruled for this.** SE28: the right click gives the command palette's search at the top, then defaults and recent commands, with any command removable and addable. SE29: a generic set, kept in the mere session, adding means any registered command, node commands first. SE31: kept as a view, not graph truth. SE32: Turnstone's rule that contextual rows lead stays, and is now the set's.
- **The change in Turnstone.** `available_actions` (`src/app/palette.rs`) registers into, or is read through, the set, keeping its contextual-first order and its single composition for palette, snapshot and automation. The `>` lane shows kept and recent commands with keep and drop. `src/shell/input.rs` opens the palette on a right release that `take_context_request` reports, rather than on the press, so a right-drag reaches the canvas as a select.

*Added 2026-10-08:* taken. Turnstone's unusual-protocols lane landed it on Turnstone main at `d0775a1`, on mere `3ded2cd7` (its SC step 4 repin, past `16c1ef8d`; checked against Turnstone's `Cargo.toml` and lock). The `>` lane reads `available_actions` through `cambium::CommandSet`, with the rows composed ahead of the static registry as the `context` category, so SE32's order holds and one composition still serves the snapshot and the runner. Mark's Turnstone choices, as the lane reports them: a bare `>` shows context, kept and recent, then "All commands…" for the whole catalog; the defaults are the browsing eight; keep and drop by a per-row control and Ctrl+D. A right press on a graph pane goes to the canvas, and the palette opens on `take_context_request` after selecting the node. Rows and their Keep and Drop now reach Turnstone's accessibility tree, and keep and drop are host-only in its ring model. macOS gates green apart from four tests already failing on main there; no headed Windows run yet. Details: Turnstone's unusual-protocols plan, Progress for 2026-10-08.

*Reading, not ruled:* the choices live in Turnstone's own session view sidecar (`ViewIntentV1.command_menu`), not pandect's `CommandMenuView`. That keeps SE31 (a view, not graph truth) but is a second store for the same thing; whether Turnstone moves onto pandect's type is Mark's to rule.


Each comes back as its own round of questions.
- **Scripting (SE3).** Compare every candidate, Rune included, through `script-engine-api` and embedded. F6 is the starting evidence.
- **Wallpapers and props.** The backdrop table (L2) is landed and consumed by Isometry, Woodshed and Graphshell. Seiche's living backdrop and scene bodies have no Mere or Turnstone host. The games wing's `MapDocument` carries a `props` layer (§4, F7).
- **Style editors.** Node: the body and face plan's B3 body designer is open. Edge: forme's `EdgeProjectionSpec`. Field: the scriptable field regions plan, where no host calls `add_field_at`. Appearance today is three strings (§4, F3).
- **Motion.** Balaur rulings C and D, gated on L5's authored front-end.

### Design-language follow-through (2026-10-09; research, not opened)

**Ownership allocation, 2026-10-10:** the
[design language §9.14](../../2026-08-23_projection_scenes_and_graph_native_platform.md#914-design-work-ownership-2026-10-10)
assigns field behavior and node/link/field editors to the projection grammar
agent in Mark's current conversation. Selection, inspection and ambient
context remain there, with their fundamentals established and residual work
focused on integration. Scene authoring and presentation rules are a research
lane; this does not revoke R1's accepted direction or recorded implementation.
Theme and typography belong to the Tabard agent on q-pc. Visible dynamics
explanations and script/motion authoring belong to the dynamics agent.
The mapped items below retain their individual implementation gates; this
allocation names design responsibility rather than opening every track.

Mark's [voice clarifications, design language §9.8](../../2026-08-23_projection_scenes_and_graph_native_platform.md#98-voice-clarifications-and-research-boundaries-2026-10-09)
extend the mapped editor concerns. Deselection preserves edits and returns to
an unselected view; it is not undo or snapshot restoration. Selection,
inspection, opening and contextual previews need explicit responsibilities
before assigning them to a HUD. Hover previews are welcomed; exact gestures
and touch equivalents remain open. S2 must preserve previously selected
background membership independently of temporary coordinated-selection clauses.

Style authoring needs to distinguish data encoding, interaction state and
theme values; channel assignments and precedence remain open. Node, link and
field appearances should support meaningful dynamics explanations. Fields may
scope a projection as well as placement and motion, with the behavior model
researched in the field plan. These additions do not open a general style
editor, script runner or HUD redesign, or reorder SE87's work.

*Candidate proof, not opened:* select and edit a subject, preview another
context, then deselect; edits survive while selection-driven context ends.
Two linked appearances preserve independent visibility and distinguish
selection from their data encoding. Exact preview commit/history behavior
still returns as a fork before implementation.

### Planning through nodes, links and fields (2026-10-09)

Mark's [primitive planning direction, §9.9](../../2026-08-23_projection_scenes_and_graph_native_platform.md#99-plan-through-graph-primitives-2026-10-09)
asks to represent nodes, links and fields in the graph and connect each to
dynamics. Plan the same inspect/configure loop for all three, retaining each
primitive's own authority and identity rather than equating every primitive
with a rendered content node. The proposed link/edge and content/resource
vocabulary must be reconciled with graph semantics before code or stored
formats change. Graphshell is the reference host for the proposed
primitive-authoring proof.

*Candidate proof, not opened:* a visible content node exposes its own
attributes and its resource association; two independent accesses to one
resource retain their identities. A link is selectable and inspectable with
its attributes/provenance and any explicit dynamics contribution. A field
exposes its region, affected members and rules. Edit through each existing
owner, save/reopen, and inspect an overlapping-field case without changing
unrelated scene behavior. Scene appearance/selection and motion explanation
must trace to those identities. Field trigger, attribute placement and
terminology choices precede implementation; this records a candidate, not an
additional SE track or replacement for the current B1/S2 sequence.

#### Primitive and dynamics map (2026-10-09; source-backed planning)

Checked against published Mere `b4e818b4`. This extends the candidate above;
it is research and a proposed sequence, not a new SE ruling. Graph primitives
already have several storage and presentation forms. Representing them in the
graph means making their identities and relationships accessible to authoring;
it does not yet choose whether each must also become an ordinary content node.

| Primitive | Existing identity and authority | Proposed authoring and dynamics connection |
|---|---|---|
| Visible content node | `SurfaceNode` carries a `Container`; its UUID differs from the canonical Resource UUID it shows. `shown_resource_id` records that association. | Inspect surface-owned settings separately from Resource content and assertions. Bind each scene appearance to its chosen body and arrangement role; changing one appearance must not implicitly move every appearance of the Resource. |
| Resource node | `ResourceNode` owns immutable canonical identity; graph resource content, properties, tags and assertions are accessed separately. Several surfaces may show it. | Expose metadata through the content node's inspection path and permit an explicit Resource appearance. A metadata Resource does not acquire a simulation body merely because it exists. |
| Link | `RosterSubject` distinguishes a link bundle from a selected relation cell. `RelationKey` distinguishes surface and Resource relations; projected relations may include parallel records and self-loops. | Inspect the exact source assertion and provenance behind a displayed link. Disclose when one mark groups several assertions. An enabled spring or other contribution must trace back through the binding to those assertions. |
| Field | Numen's `FieldId`, definition, extent and lifecycle are graph-held; a separate `CouplingId` names the field, selector, response and strength. | Inspect/edit the region and its couplings, show intended members separately from evaluated targets, and explain each recognized response. A region appearance and a field's simulation participation remain separate choices. |

Source anchors: [surface substrate](../../../crates/graph/graph-kernel/src/graph/node.rs),
[Resource association](../../../crates/graph/graph-kernel/src/graph/resource.rs),
[relation reads](../../../crates/graph/graph-kernel/src/graph/relation_read.rs),
[roster subjects and cards](../../../crates/mere/src/roster.rs),
[field](../../../crates/conatus/numen/src/field.rs) and
[coupling](../../../crates/conatus/numen/src/coupling.rs).
The roster already offers nodes, links and fields in one model; this inspection
does not establish that Graphshell exposes every roster action today.

**Existing dynamics connection.** Pictograph's
[Seiche bridge](../../../crates/canvas/pictograph/src/canvas/seiche_bridge.rs)
currently creates a body per kernel surface node, springs per visible relation
cell and forces from graph couplings. Springs carry endpoint pairs, so this
bridge alone does not expose exact assertion provenance to a force inspector.
Coupling conversion resolves a selector to a snapshot of node keys and copies
the field definition and strength. It does not independently enforce the
field's extent or detect entry/exit. An open response IRI is preserved by numen
but is ignored by the force integrator until its owning consumer recognizes it.
These are concrete gaps for the proposed authoring loop, not evidence that the
expanded field behavior is implemented.

**Proposed sequence and done-conditions:**

1. Inventory the existing node, link and field cards in Graphshell and map each
   editable value to graph/session authority, recipe authority or view state.
   Done: every proposed control names its identity, storage owner and edit route;
   absent controls are recorded rather than inferred from the roster model.
2. Prepare one small authoring scene: two surfaces of one Resource, parallel
   assertions between a pair, one self-link, a forme and a second overlapping
   field. Done: each appearance resolves to its source, grouped links disclose
   their members, and each field discloses its region, selector and responses.
   This fixture is proposed; geometry and overlap policy are still to be chosen.
3. Connect inspected identities to the existing recipe/binding/runtime path.
   Done: two simultaneous contributions and one arrangement constraint can be
   traced to their source and spec path; disabling one contribution changes only
   its declared effect. A removed or missing target cannot silently retarget.
4. Exercise edit, deselect, undo and save/reopen through their existing owners.
   Done: edits survive deselection, shared Resource changes reach both surfaces,
   appearance settings remain independent, and undo/reopen retain exact identities.
   Native/Wasm dynamics and browser receipts follow the existing lane gates.

Before implementation, return the actual forks: whether the proposed narrow
*edge* vocabulary replaces or merely labels current relations; where arbitrary
attributes on links/fields belong; and how incompatible overlapping placement,
projection and behavior rules compose. A shared inspect/configure interaction
can be researched before those storage and composition choices are ruled.
The dynamics slot and settle work, then the grid dynamics axis, B1 and S2 retain
their existing order and active-owner boundaries.

#### Graphshell control inventory (2026-10-10; planning step 1)

Checked against published Mere `6183006b`, independently of the dynamics draft
being qualified in the primary checkout. This is source inspection of the
browser reference host, not a new browser receipt or native-host qualification.

| Control | Present browser route | Authority and missing work |
|---|---|---|
| Create a content node | `node:new` calls `create_address`, then refreshes the Canvas. | MereHost's recorded graph edit. A graph member is not a preview occurrence. |
| Edit title and tags | `node:edit` opens detail; `save-metadata` calls `edit_node`. | The title delta addresses the Surface. Tag deltas route to the shown Resource and retain assertion authorship. Shared Resource changes can affect several surfaces. |
| Edit a named JSON value | The same form optionally calls `set_product_facet`. | A separate journaled Surface-facet edit. This is not a general Resource, link or field attribute editor, nor the pending facet-dissolution design. |
| Add a link | `add-relation` resolves a target by URL and calls `assert_product_relation` for one of five editable kinds. | A recorded assertion through MereHost. The form does not address an individual statement handle, its provenance or a link-owned dynamics rule. Selecting the pair selects its nodes. |
| Inspect link bundles and cells | Shared roster has `LinkCard` and distinct bundle/cell subjects. | No Graphshell source calls these builders or consumes their subjects. Wiring an exact source assertion remains work. |
| Inspect and configure fields | Shared roster has field detail and visibility/strength intents. Canvas has placement, strength and visibility methods. | No Graphshell source consumes these roster intents or calls `add_field_at`/`set_field_strength`. Canvas's field writes mutate its graph; a host control must route durable changes through the authoritative session before refreshing the view. |
| Configure placement and motion | Browser arrangement, physics and role controls affect the Canvas; saved scenes retain the settings. | Distinct from a graph primitive's attributes. The pending occurrence-preview adapter is not assumed to supply field or link authoring. |

Sources: [historical browser product dispatch and form](https://github.com/merely-made/mere/blob/e84cfbd95b1660046d646580a8707af020a06e4b/ports/graphshell/src/web_product.rs),
[recorded product edits](../../../ports/graphshell/src/product.rs),
[MereHost edit and undo routes](../../../ports/graphshell/src/mere_host.rs),
[tag routing](../../../crates/graph/graph-kernel/src/graph/resource_content.rs),
[shared roster](../../../crates/mere/src/roster.rs),
[Canvas field writes](../../../crates/canvas/pictograph/src/canvas/input.rs) and
[field visibility](../../../crates/canvas/pictograph/src/canvas/fields.rs).
Hiding a field is presentation-only; its coupling continues to exist. Field
rule/extent cards currently describe values and select the field; they do not
edit the rule, script or extent. Their script/template rows say not configured.

**Edit boundary to resolve in the authoring proof:** `save_metadata` commits
title/tags before parsing and writing the optional JSON facet. An invalid JSON
value can therefore report failure after title/tags have changed. Two successful
calls also make separate recorded edits. The proposed authoring loop must state
whether a form submission is one change or several, validate the full draft
before committing, and show which owner each value changes. This is a source
finding; no repair or new undo semantics is claimed in this documentation pass.

Step 1's inventory is complete at this source. Before opening step 2, prepare
the explicit member-to-Resource inspection and exact-statement selection paths,
then the field read/write adapter. Reuse the established session for writes and
separate presentation intents. Arbitrary attribute storage, primitive node
encoding and overlap policy remain the existing forks. The active dynamics
qualification and B1/S2 owner boundaries retain their priority.

#### Primitive editor draft (2026-10-10)

**Design proposal; no new runtime track or receipt.** Mark assigned node, link
and field editors to the projection grammar agent and asked it to proceed.
The shared [allocation](../../2026-08-23_projection_scenes_and_graph_native_platform.md#914-design-work-ownership-2026-10-10)
preserves Tabard's appearance roles, the dynamics lane's explanations and
script/motion authoring, and established selection fundamentals.

Propose one inspect/edit pattern reached from a scene appearance or roster
entry. The inspector identifies the target in ordinary language before offering
writes. Inspecting something need not replace the working selection or open its
content. An explicit Edit action starts a draft for the chosen owner. Source
facts, this access and this appearance are distinguishable scopes, with their
shared effects explained rather than exposed as raw implementation labels.

| Target | Inspect | Proposed editing boundary |
|---|---|---|
| Content access / Surface | Title, content access, associated Resource and other appearances. | Surface-owned values address this access; Resource facts have their own edit action and disclose shared use. Content editing opens its existing document editor. |
| Scene occurrence | Its face, applicable rules, local placement and dynamics role. | Edit this appearance or its named recipe through the scene owner. A foreground pin remains separate from position pin, anchor and forme lock. |
| Link mark or bundle | Endpoints, contributing relations and provenance; explain whether one mark represents several facts. | Select the intended source record before a source edit. A relation family/subkind filter is not exact statement identity. Derived links expose their inputs/rule and route editing to their owner. |
| Field | Identity, members, extent, placement, couplings and scoped presentation. | Separate field values, individual coupling values, scene visibility and workbench arrangement writes. Use the field plan's draft. |

**Source findings at published Mere `86f2a3b8`:**
[RosterSubject](../../../crates/mere/src/roster.rs) already distinguishes Node,
LinkBundle, RelationCell and Field. Its
[RelationSelector](../../../crates/graph/graph-kernel/src/graph/edge_taxonomy.rs)
selects families/subkinds; a RelationCell is not necessarily one assertion.
[RelationKey](../../../crates/graph/graph-kernel/src/graph/identity.rs) identifies
the Surface/Resource relation stratum. The exact-record adapter must retain
the source's statement/provenance identity rather than infer it from endpoints.
The existing [product edits](../../../ports/graphshell/src/product.rs) address
Surface titles, shown-Resource tags and relation assertions; arbitrary
link/field attributes remain a storage-design question. The proposed narrow
link/edge vocabulary is not adopted as a schema rename by this editor draft.

The [current forme host](../../../ports/graphshell/src/web_tree/forme.rs) and
[saved workspace](../../../ports/graphshell/src/forme_workspace.rs) already
provide field inspection and bounded workbench projection. This supersedes
the earlier inventory's no-field-inspection-caller finding for that retained
host, without claiming shared roster-card adoption or a general field editor.
Existing workspace records hold arrangement, geometry, bounds, lock and
visibility; the [draft adapter recorded in Progress](#workbench-forme-draft-2026-10-10)
now adds private draft history and saved layout undo, with headed checks pending.

**Proposed edit flow:** Inspect → Edit → Preview → Apply / Discard changes.
Use the agreed Unlock → Lock and apply wording for a forme. Show pending state
and the exact affected owner. Validate all authored values before applying a
single-owner form; its successful apply should be one logical saved change.
The current metadata form commits title/tags before parsing an optional JSON
facet, so it does not meet that proposed behavior. Multi-owner edits need an
explicit compound adapter or separate named applies; do not promise atomic
saved undo merely because the controls share an inspector. Failed apply retains
the draft and states which changes, if any, committed. Discard changes leaves
content and other owners' committed edits intact.

**First walkthrough:** inspect two accesses to one Resource; change one access's
title, then explicitly inspect the shared Resource. Inspect a bundled link and
choose its actual source record. Inspect two overlapping fields and tune one
named coupling. Finally edit a forme through its scoped draft and discard or
apply it. This keeps arbitrary attribute storage and scripting out of the first
editor proof until their owners supply the necessary contracts.

Done when the walkthrough exposes the intended identity and shared effects;
two accesses remain distinct; a bundled mark cannot silently edit the wrong
assertion; one coupling edit leaves the others unchanged; malformed input
commits nothing for the promised single-owner edit; and apply/discard/undo plus
save/reopen preserve unrelated state. Test derived and undisclosed relations,
hidden fields, changed/deleted targets and failed persistence. Keyboard access
must reach the same inspection and apply/discard actions. The
[field authoring draft](2026-06-13_scriptable_field_regions_plan.md#field-authoring-draft-2026-10-10)
owns membership, overlap checkpoints and the field-specific walkthrough.

#### Implementation draft delivery (2026-10-10)

**Status:** delivery workflow authorized; candidates below are recommended,
not claimed implementations. Mark authorizes drafting existing plans and
pushing directly to main so agents on the other machines have code to review,
test and improve. A PR is optional. Host testing may follow the draft rather
than block its delivery. Each implementation pass records its scope, source
revision, checks actually performed, unfinished behavior and concrete next
review/test steps in the owning plan. An unrun check remains unrun; publishing
a draft does not complete the plan's acceptance gate.

The most useful candidates at source `559a3e90` are:

| Candidate | Bounded draft to publish | Review and continuation |
|---|---|---|
| Access / Resource inspector | Use recorded associations to show this access, its shared Resource and other held accesses, through read-only retained controls. | Verify separate access identities, incomplete-residency disclosure, selection/focus and keyboard behavior. Then add the exact owner's editing draft. |
| Link bundle inspector | Expand a displayed pair into its directed source records, keeping relation stratum, exact assertion handle and available provenance. | Exercise parallel assertions, self-links and lifted Resource links. Exact-record editing follows its authorship/write contract. |
| Field inspector and overlap discovery | List exact fields and individual couplings; expose all matching region candidates and keep the chosen target stable. | Check hidden/inactive inspection, deterministic candidate lists and pointer behavior. One-coupling editing follows a scoped host transaction. |
| Presentation-rule editor | Bind controls for target → condition → effect to the existing declarative resolver; preview and explain matches, missing inputs and overrides. | Review swatches, precedence, hysteresis, measurement and owning persistence. This slice can exercise the existing rule model while Rhai awaits its runner seam. |
| Mixed-content authoring fixture | Compose distinct content accesses, a forme, parallel links and another field into one inspectable Graphshell example. | Other machines can pressure-test the assembled interactions and expose layout/identity integration faults. Its fixture does not decide overlap precedence or applet execution. |

Recommended order: access/Resource inspection, link inspection, then field
inspection/overlap discovery. These expose the identities the later editors
need and can be delivered without choosing new attribute storage. A rule-editor
draft is the next appearance-authoring candidate, scoped to the implemented
[declarative model](../../../crates/cambium/scenes/scenograph/src/presentation.rs).
The mixed fixture becomes more useful as these inspection paths arrive.
Tabard's theme/font work, the dynamics lane's active runtime work and the
unresolved field event/composition choices retain their named owners.

#### Identity inspection follow-through (2026-10-10; next design steps)

**Status (2026-10-11):** access/Resource, link/assertion and field/coupling
inspection implemented as drafts; write adapters remain planned. Mark asked this agent to
keep working on plans while agents on O-PC, the ThinkPad and the Macs perform
visual tests. The implemented forme draft remains the bounded runtime proof. This sequence prepares the primitive-editor walkthrough without choosing
the outstanding attribute-storage or field-composition policies.

**Implementation continuation:** the access/Resource read-only adapter is
described in [Progress](#accessresource-inspector-draft-2026-10-10), followed by
the [link/assertion draft](#link-and-exact-assertion-inspector-draft-2026-10-11).
The [field/coupling draft](2026-06-13_scriptable_field_regions_plan.md#field-inspection-draft-2026-10-11)
adds deterministic boundary discovery and independent Forme-owner reads. Edit
adapters retain their planned status. The link draft reads held source records;
rule-derived display marks still need their owning explanation adapter.

Source checked at published Mere `69f17908`:

- [Resource reads](../../../crates/graph/graph-kernel/src/graph/resource.rs)
  expose `shown_resource_id`, `resource_record` and
  `surface_ids_showing_resource`. The inspector can disclose the recorded shared
  owner and its held accesses without grouping accesses by equal URL.
- [Projected relation reads](../../../crates/graph/graph-kernel/src/graph/relation_read.rs)
  retain `RelationKey::Surface` or `RelationKey::Resource` for parallel records
  and self-links. One visual pair can therefore disclose several owning records.
  [Assertion lookup](../../../crates/graph/graph-kernel/src/graph/assertion_write.rs)
  resolves a held `statement_id` to its recorded store. A statement's scope,
  provenance, predicate and assertion time are available in
  [SemanticStatement](../../../crates/graph/graph-kernel/src/graph/edge_data.rs).
- The [roster](../../../crates/mere/src/roster.rs) carries member pairs and
  family/subkind selectors in link rows, but does not carry the exact statement
  identity or a Resource inspection subject. Its existing editable flag alone
  cannot authorize an exact-record edit. These are adapter gaps, not evidence
  that the kernel lacks assertion identity.

**Proposed sequence and done-conditions:**

1. **Inspect the access and its shared source.** Resolve a clicked appearance
   to its graph member, then read its recorded Resource association. Show this
   access's title and address, the shared source's facts, and the other held
   accesses. Inspection keeps the working selection and open content sessions
   intact. A missing association is disclosed; an equal address is not enough
   to invent one. Done: two surfaces showing one Resource remain distinct, a
   changed shown Resource refreshes the inspector, and incomplete residency is
   labelled rather than presented as a complete list of all accesses.
2. **Expand a link into its source records.** A bundle first lists its directed
   endpoints and contributing records, including self-links and parallel facts.
   Retain the source stratum and statement handle wherever one exists. A
   family/selector row may group records, but must lead to an exact record before
   editing. Non-statement relations retain their actual owning data; do not mint
   a statement merely to fit this inspector. Derived marks disclose the rule
   and inputs through their owner, with missing provenance shown. Done: the
   walkthrough distinguishes two assertions of the same predicate and a
   Resource relation lifted onto two different access pairs. Reopen resolves
   stable identities rather than storing local edge indexes as durable targets.
3. **Bind an edit to the inspected owner.** Begin with separate access metadata
   and shared-source editing actions. Read and retain a baseline for the chosen
   owner, validate the full draft, and re-resolve its identity before apply.
   A deleted or changed target keeps the draft with an explanation; it cannot
   retarget to another node at the same address. Exact assertion mutation needs
   its existing authorship/retraction contract, and general Resource attributes
   need graph semantics' storage decision before those controls open. Done:
   malformed input changes nothing in the promised single-owner operation,
   discard preserves other owners, and saved undo names its scope. The current
   sequential title/tag/facet form is not treated as this compound operation.
4. **Join field inspection to the same interaction.** Use the
   [field adapter sequence](2026-06-13_scriptable_field_regions_plan.md#field-inspection-adapter-sequence-2026-10-10)
   for exact field/coupling subjects and overlap discovery. A forme opens its
   implemented arrangement draft; a Numen field opens its own inspection.
   Done: the same scene can explain both owners without fabricating an
   association between them or invoking an action during preview.

The first deliverable is read-only inspection with identity receipts. Subsequent
writes must prove their owning transaction and save/reopen path before adding
Apply. Keyboard inspection reaches the same subjects as pointer inspection.
Tabard owns appearance roles; dynamics owns effect explanations and execution.
The existing dynamics qualification → B1 → S2 order remains with that lane.

#### Mixed-content study (2026-10-10; proposal, not opened)

Mark forwarded the design-language agent's proposed
[mixed-content scene and embedded forme study, §9.12](../../2026-08-23_projection_scenes_and_graph_native_platform.md#912-mixed-content-scene-and-embedded-forme-study-2026-10-10).
It joins the existing style-editor and composition research; it creates no
SE ruling or additional active track. The candidate theme axis is not an
implemented facet axis. The existing dynamics qualification → B1 → S2 order
and viewer-file ownership holds stand.

*Reading, not ruled:* prepare the study in three reviewable parts:

1. Establish one scene with document, image and video material, related nodes
   and a forme corresponding to two existing side-by-side webpage accesses.
   Record source, occurrence, arrangement-member, tile and session identities.
   A captured face must be distinguishable from a working live surface.
2. Compare editable appearance treatments and representation/arrangement
   choices. Retain semantic font roles and selection cues. Separate paint-only
   updates from footprint changes requiring measurement and placement.
   A target → condition → effect sketch should expose matching inputs,
   precedence, fallback and eligibility before becoming a rule schema.
3. Exercise the tile-to-field bridge: hover subdivisions, selection, explicit
   focus, moving the outer field and deliberately unlocking its internal
   layout. Apply accepted tile edits to their owning state and refresh both
   projections. Keep the semantic arrangement, projection geometry and
   document-session ownership distinct.

Candidate done-conditions: changing treatments preserves source/access
identity and manual placement; font or face changes update measured extents;
returning to the tile reuses its session; moving the field preserves a locked
local split; selection and deselection preserve edits; unlocking a layout
change round-trips through undo and save/reopen. Explain each displayed rule
and overlapping-field contribution. Compact/detail thresholds must not
oscillate merely because their own representation changes the measured size.
These are acceptance proposals, not recorded receipts. SE36's Rhai choice
does not establish that the pending rule runner or style editors exist.

#### Forme draft follow-through (2026-10-10; agreed interaction)

**Status (2026-10-10):** interaction agreed; the bounded Graphshell adapter is
implemented with native/build checks recorded in [Progress](#workbench-forme-draft-2026-10-10).
Headed host qualification and adoption by other hosts remain pending.
Mark accepted the [forme draft session, design language §9.11](../../2026-08-23_projection_scenes_and_graph_native_platform.md#911-forme-draft-session-2026-10-10):
unlock starts a private draft; nodes act as tile handles with split/drop
previews; adding a node changes draft membership; undo/redo traverses draft
gestures; Discard changes drops the draft and locks; Lock and apply commits
one arrangement change and locks. Both graph and tiled presentations preview
the draft. This records the accepted model without declaring a new SE track
or changing another lane's assignment.

**Source findings at published Mere `e95326dc`:**

- [FormeDocument](../../../crates/forme/forme/src/forme_document.rs) addresses
  the owning forme and graph and stores its semantic arrangement separately
  from geometry and host persistence policy.
- [Platen's TileLayout](../../../crates/platen/platen/src/workbench.rs) is
  cloneable and already supplies membership, stack/split and fraction edits.
  Its [bridge](../../../crates/platen/platen/src/workbench/bridge.rs) persists
  the semantic arrangement and tree geometry together and reconstructs the
  working layout. `to_arrangement` constructs fresh local arrangement-node
  IDs; a persistent field bridge must deliberately retain/map its owning
  identities rather than treat each preview conversion as a new forme.
- Cambium's dependency-free [History](../../../crates/cambium/backstep/src/lib.rs)
  already records snapshots, coalesces gestures and provides undo/redo. Its
  draft history is separate from the host's durable saved-change history.
  The existing projection editor applies that separation to recipe edits;
  it does not provide this forme editing session or its persistence adapter.

**Implementation steps and done-conditions:**

1. **Scoped draft model.** Bind each editing session to the existing forme
   identity, semantic arrangement and projection geometry; reuse `History`
   for arrangement snapshots. Coalesce a drag into one draft step. Discard
   removes only this draft/history; another forme and unrelated source edits
   survive. Done when membership and nested geometry undo/redo independently,
   an added node survives discard, and reopening starts at committed state.
2. **Shared preview and handles.** Feed graph-field regions and tiled
   presentation from that draft using existing tile mutations. Make pending
   state, Discard changes and Lock and apply visible; route undo by editor
   focus. Done when node handles show meaningful split/drop previews, both
   presentations agree and document editing retains its own undo routing.
3. **Apply and saved undo.** Send the semantic arrangement and projection
   geometry through the owning host's persistence boundary as one logical
   arrangement edit. Keep the draft available if apply fails. Done when
   successful apply locks, saved undo/redo restores both arrangement and
   geometry, and save/reopen retains member/access identity without replacing
   document sessions. Discard must not create a saved-change history entry.

The Graphshell adapter's scoped source checks are recorded in Progress; headed
verification of these done-conditions remains pending. Draft membership and tile
geometry are distinct from node content or resource attributes. SE6 still
excludes the projection editor's own UI furniture from recipe undo; this
forme draft edits the user's arrangement and does not amend SE6. Existing
dynamics qualification, B1/S2 and viewer ownership remain with their owners.

## 4. Findings

Verified 2026-10-07 against mere's origin unless named.

- **F1, the editor.** `ProjectionEditor` holds a draft, a panel and a `Workbench`, and reduces ten actions: `SelectPanel` and nine whole-field setters (`projection_editor.rs`, `EditorAction` and `reduce`). Its one effect is `save` through a `ProjectionDefinitionSink`. It has no undo. Graphshell's web app is its only host (`web_main.rs`, `web_projection.rs`).
- **F2, Cambium's history.** `EditHistory` (`crates/cambium/cambium/src/editor.rs`) is text-only: snapshots of text, caret and selection, a cap of 200, insert-run coalescing, no clock and no saved marker. Its users are `TextInput` (`controls/text_input/core.rs`, `command.rs`) and the crate's re-export. No crate in mere, Turnstone, Knot, Woodshed, Isometry, Genet or mer3ly names it. Cambium has no clock type. A second undo exists for graph truth, attributed and journal-based (`pandect::graph_session` `undo`, Graphshell's `mere_host`); the editor's draft history is a separate, local plane.
- **F3, the authored surface.** `Appearance` is `realization`, `title` and `theme`; `Encoding` is x, y, color and label; `Arrangement.options` is an open `BTreeMap<String, String>` (`crates/cambium/scenes/scenograph/src/lib.rs`).
- **F4, options.** Eleven families (`Family`, `FAMILIES` in `scenomise/src/catalog.rs`) read 26 distinct keys through seven readers (counted by literal call; E4's test finds any other): `finite`, `positive`, `count`, `depth`, `flag`, `choice`, `list`. `finish` refuses any key not read. A custom solver receives the options as a JSON object and judges them when it solves (`scenomise/src/projection.rs`). `SolverCapability` (`registry.rs`) carries no options. The only `Solver` implementations in mere, Isometry, Turnstone, Woodshed, Knot and mer3ly are test fixtures.
- **F5, the games wing.** Answering read-only (Isometric game engine architecture session), the wing took balaur's determinism recipe and labelled digest (wing rulings 603 to 611, 633 to 636, in `isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`) and none of the editor slices. Ruling 609 builds one labelled-digest crate in mere under G8.
- **F6, scripting.** Genet's `script-engine-api` (`components/script-engine-api/lib.rs`, 975 lines) is a JavaScript-shaped contract: realms, `WindowProxy` traps, reflectors carrying a DOM `NodeId`, host promises. Its backends are Nova, Boa and piccolo; piccolo's documents its deviations and does not yet honour step budgets. Outside it: mere's rhai crate, behind inker's `BlockEvaluator` and "deliberately *not* genet's full DOM-shaped `ScriptEngine` seam" (`crates/script/rhai/src/lib.rs`); mere's Wasmtime hosts on the `mere:script` WIT world (`crates/script/wit/world.wit`); and Isometry's piccolo, which `isometry-system` and `mesocosm-phenotype` depend on directly. No manifest in mere, Isometry, Turnstone, Woodshed or Knot names Rune. *Reading, not ruled:* values cross the seam as strings and node handles only, so scene logic may want a value-level seam beside it.
- **F7, the wing's stagecraft.** Isometry's `MapDocument` (`repos/isometry/crates/isometry-core`) carries tile layers including `props`; appearance binds through CSS classes with tilesets as stylesheets (`repos/isometry/crates/isometry-views`); the shared scene is `shared/isometer`.

- **F8, the engine probes (SE38, 2026-10-07).** Scratch at `Code/testing/mere/scripting-probes` (outside the tree): one rule (a Gaussian falloff with `exp`, `sin`, `cos`, `sqrt` and a power over 10,000 generated positions), hashed by FNV-1a over the float bits; release builds at `opt-level = "s"` with LTO; wasm32 run under Node 24 (V8). Rune's float module has `sqrt` and `powf` but no transcendentals, and Starlark has no maths library, so both take host functions on `libm`; rhai ran on its built-in maths and on `libm`.

  | Engine | Native median | wasm32 median | Hash, native / wasm32 | wasm gzip |
  |---|---|---|---|---|
  | rhai, built-in maths | 32.8 ms | 74.5 ms | `d149150d…` / `028dc38f…` | 0.49 MB |
  | rhai, `libm` | 35.8 ms | 47.7 ms | `028dc38f…` both | 0.49 MB |
  | Rune 0.14.2, `libm` | 14.5 ms | 38.7 ms | `028dc38f…` both | 0.98 MB |
  | Starlark 0.14.2, `libm` | 12.2 ms | 27.4 ms | `028dc38f…` both | 1.27 MB |

  Medians of five runs, each including the engine's setup and compile. Every budget stopped a runaway on both targets: rhai's operation limit, Rune's `budget::with`, Starlark's `set_max_tick_count` (calls and loop back-edges; it also caps heap). The wasm32 builds needed `getrandom`'s `wasm_js` backend: rhai and Rune through `ahash` (0.3, also the `--cfg getrandom_backend`), and Starlark through `pagable` → `sorted_vector_map` → `quickcheck` → `rand` (0.4), a property-testing crate in its normal dependency tree. Starlark's `range` takes a 32-bit int. *Reading, not ruled:* rhai is the slowest by two to three times but the smallest, and rules run on events, so its speed does not argue against SE36.
- **F9, next door (2026-10-07).** numen's CPU field evaluator (`crates/conatus/numen/src/eval.rs`) uses std `.exp()` and `.sqrt()`, so field maths that feeds seiche's couplings is not yet on `libm` under S4; G8's lint is the dynamics lane's to apply.

- **F10, the trait fits rhai (2026-10-07).** genet branch `value-engine` at `27477d3e` (off genet main `161b1a89`): `components/script-engine-api/value.rs` *(historical citation)* <!-- doc-audit: historical-path --> adds `ScriptValue` (unit, bool, int, float, string, list, ordered map), `HostFunction` (`Arc<dyn Fn(&[ScriptValue]) -> Result<ScriptValue, String> + Send + Sync>`), `ValueError` (budget, script, host, value) and `ValueEngine` (`set_global`, `register`, `eval` under `Budget`); three tests through a toy backend, including use behind `dyn ValueEngine`. A scratch rhai implementation (`Code/testing/mere/scripting-probes/probe-value-rhai`, outside both trees) loads rhai's core, array and map packages but not its maths package, registers `libm` maths through the trait, and reproduced the probe rule's hash `028dc38f…` exactly; `sin(1.0)` without a host function fails as "Function not found", and a runaway returns `ValueError::Budget`.
- **F11, the command map (C2, 2026-10-08).** Read against mere main `81b6ad78` and Turnstone main `d0775a1`.
  - *The person's choices, three times.* cambium's `CommandChoices`, pandect's `CommandMenuView` (`view_intent_store.rs`) and Turnstone's `CommandMenuV1` (`src/session.rs`) are the same three lists, `added`, `removed`, `recent`; the two stored ones both carry `#[serde(default)]` on each, so one type in `command-menu` reads what either wrote. graph-kernel adds a fourth, narrower run: `CATEGORY_RECENCY_PERSIST_KEY` and `CATEGORY_PIN_ORDER_PERSIST_KEY`, recency and pins for its four categories.
  - *A palette's in-progress state, three times.* mere-chrome's `CommandPaletteSession` (`crates/shell/chrome/src/command_palette.rs` *(historical citation)* <!-- doc-audit: historical-path -->): query, a four-way search scope (current target, active pane, active graph, workbench), a selected index with a wrapping `step_selection`, a focus-the-search flag, an egui frame flag, and a selected `ActionCategory`; nothing in mere or Turnstone reads it. Graphshell web's `OpenMenu` (`web_commands.rs`): where it opened, its context, query, a redraw flag, a focus-the-search flag. Turnstone's `OmnibarState` (`src/ui.rs`): the command lane is the omnibar's text after `>`, its `selected` row and `all_commands` (the "All commands…" expansion); the rest is the address bar's. Shared by all three: query, selected row, focus the search. Shared by two: the context it opened over, an expansion or tier. Host-only: position, redraw, frame flags, the address bar.
  - *Commands defined twice.* graph-kernel's `ActionId` (`crates/graph/graph-kernel/src/actions.rs`, 588 lines): a closed enum of 68 actions with namespaced keys (`graph:fit`), labels, short labels and four categories, read by ux-events (diagnostics, observability, the destructive-action probe) and by mere-chrome's and shell-state's `HostIntent::Action`. No live host registers it: Graphshell web registers 32 commands under its own kebab ids, and Turnstone registers its composed rows under their labels, which its automation runner resolves. The same verbs appear under different ids: `graph:fit` and `fit-content`, `graph:toggle_physics` and `toggle-physics`, `persistence:undo`/`redo` and `session-undo`/`redo`, and Turnstone's Fit view. Turnstone reads mere-chrome only for history navigation (3 files).


## Progress

- **R1, force-directed dataspace clarification (2026-10-09).** Mark clarified
  that the first aesthetic study emphasized the forme more than he had intended:
  he had pictured a force-directed node graph, and then explicitly appreciated
  having both views. The study now starts with the force-directed dataspace and
  retains Workbench / forme, Neighborhood and Hierarchy. The same occurrence
  identities, rules, selection, foreground pins and position pins survive view
  changes; each arrangement keeps its own coordinates. Ambient nodes are
  represented within the dataspace under the same lens and retention policy.
  The forme field marks its four active surfaces within the graph.

  The bounded study solver uses family-specific springs, repulsion, overlap
  avoidance and damping. Link-family controls change which disclosed
  relationships the study recipe binds to attraction; relation membership
  alone does not install a force. It settles for a finite number of
  deterministic steps; selection and theme changes do not start perpetual
  motion. Position pins are exact constraints, while the current selection is
  temporarily held during context-driven settling. Foreground pins affect
  attention and context without imposing a position constraint. This is a local
  preview, not adoption or replacement of the production dynamics grammar.

  The resolver parity checks still pass all 116 cases. Six force checks cover
  repeatability under reordered input, exact pins, finite measured bounds,
  different family attraction, removal of a relationship and separation of
  overlapping free bodies. Headed Chromium checks verified that removing shared
  resource attraction moves free nodes while preserving pinned media, and that
  selection and foreground pins survive switches to the workbench and back.
  Foreground-pinned nodes remained free to move under changed link forces;
  theme changes preserved geometry, and the 320-pixel layout had no horizontal
  overflow. Native key automation, like drag automation, could not complete
  through the browser adapter; arrow-key arrangement is implemented but was
  not verified with a headed input action. Stored geometry is restored without
  an automatic re-solve when the host echoes saved state.

- **R1, presentation rules and aesthetic study (2026-10-09).**
  [`scenograph::presentation`](../../../crates/cambium/scenes/scenograph/src/presentation.rs)
  implements the versioned target → condition → effect model with atomic
  validation, per-property priority and authored-order tie breaking, match
  explanations, and occurrence-local screen-size hysteresis. The existing
  `sceno::Representation` vocabulary is reused. Hosts retain source authority,
  selection, pins, measurement and placement; the resolver returns decisions.
  [`scene-rules.html`](../../../support/design-studies/scene-rules.html) is the
  first mixed-content study, with Paper and ink, Workshop and Night as editable
  starting treatments, independent arrangement choices, retained ambient
  context, distinct foreground and position pins, forme guides, link provenance,
  and rule editing. Two surfaces access one resource independently. The local
  five-second media sample uses native video controls. Its identity marks are
  monochrome previews of actual Pictograph `params_of` masks (derivation v3),
  not a browser Emblem decoder or the full Pictograph LOD renderer. Forme
  quadrants and alternative link marks are study representations, not a new
  production tiling or dynamics implementation.

  **Qualification:** an isolated workspace containing the unchanged manifests
  and complete source of `sceno`, `scenograph` and `curation` passed 57 unit tests
  and one documentation test. Seven new resolver tests cover priority and ties,
  independent accesses, missing/wrong-type facts, hysteresis, malformed sets,
  persisted rules, and complexity limits. This is scoped source qualification;
  the sparse checkout's full workspace and its lockfile were not qualified.
  Build the `presentation_rules` example, then run
  `node scripts/mere_scene_study_check.mjs <path-to-presentation_rules>`.
  The [source qualification receipt](../../../support/design-studies/source-qualification.json)
  identifies the exact copied files and isolated lockfile. The
  [parity receipt](../../../support/design-studies/scene-rules-receipt.json)
  records 116 identical resolutions and four invalid sets refused by both
  hosts. `python3 scripts/mere_scene_study_pack.py` reproducibly embeds the
  canonical fixture, face masks and media in the fragment.

  Headed checks in the Codex Chromium browser verified rule changes, invalid
  edits retaining the prior valid presentation, threshold hold/exit, pinned
  media, three-node selection, foreground persistence after deselection, theme
  changes preserving coordinates, retained context across lenses, unlocked
  guides and a disabled drag handle on position-pinned media. Playing media
  continued across a theme change and selection. Responsive checks
  at 1024 and 320 pixels verified the narrow flow layout without horizontal
  overflow. Native drag automation could not run because the browser adapter
  rejected fractional iframe coordinates; drop exchange and pinned-occupant
  refusal are implemented but have not had a headed gesture check. Firefox and
  Safari were not run for this study. The browser resolver is a study host;
  production Rhai execution remains gated on SE39–SE42 and no Graphshell
  adoption is claimed. The [mechanical documentation delta](../../../support/design-studies/doc-audit-delta.json)
  added no findings and repaired two moved Tabard references; existing audit
  findings remain, including the D2 snapshot digest and coverage failures.

- **F12, the `ActionId` audit (C2, SE48, 2026-10-08).** Each of the 68 variants against Graphshell web's commands, Turnstone's catalogue (`d0775a1`) and the readers; gathered by a read-only agent, its load-bearing claims rechecked here. *Live* (a host offers the verb today), 12: `node:new`, `node:delete`, `node:edit_tags`, `node:mark_tombstone` (the same recoverable delete as `node:delete`; Turnstone's recycle bin, and the owner's "no tombstones"), `graph:fit`, `graph:toggle_physics`, `persistence:undo`, `persistence:redo`, `persistence:save_graph` (Turnstone's Save session; Graphshell's Save scene saves view state), `workbench:settings_pane`, and loosely `workbench:toggle_workbench_overlay` (now summon a pane) and `workbench:open_history_manager` (now the Trail). *Backed* (working code, no command), 16: `node:pin_toggle`, `node:pin_selected`, `node:unpin_selected`, `node:open_split`, `node:detach_to_split`, `node:move_to_active_pane`, `node:render_auto`, `node:import_webfinger`, `node:resolve_activitypub`, `node:add_to_frame` (a `frame-member` relation), `edge:connect_pair`, `edge:connect_both`, `edge:remove_user`, `graph:physics_config`, `workbench:command_palette_open`, `import:bookmarks_from_file`. *Planned*, 1: `node:resolve_nip05`. *Uncertain*, 9: `node:remove_from_subgraph`, `node:refresh_person_identity`, `node:copy_url`, `node:copy_title`, `graph:fit_subgraph`, `graph:cycle_focus_region`, `persistence:save_snapshot`, `persistence:restore_session`, `persistence:restore_latest_graph`. *Stale*, 30: the eight `frame:*`, the four layout-lock and tile-group `workbench:*`, the three workflows, `node:new_as_tab`, `node:choose_frame`, `node:add_connected_to_frame`, `node:open_frame`, `node:open_neighbors`, `node:open_connected`, `node:warm_select`, `node:resolve_matrix` (Matrix is SKIP in the standards survey), `node:render_webview`, `node:render_wry`, `graph:toggle_overview_plane`, `graph:toggle_ghost_nodes`, `workbench:radial_menu_open`, `persistence:open_hub`, `workbench:settings_overlay`. *Readers:* ux-events and `HostIntent` carry any `ActionId`; nothing outside `#[cfg(test)]` builds `UxEvent::ActionDispatched` or `HostIntent::Action` (checked); the one variant named in non-test code is `NodeMarkTombstone`, in `DestructiveActionGateProbe::iced_default`, which only tests call, for an iced host not in the tree. *Missing* (verbs hosts offer with no `ActionId`): navigation (Back, Forward, Reload, Stop), canvas zoom and pan, camera and view toggles, the physics catalogue rows, panes and windows, sessions, feeds, places, document find and capture, participants, the projection editor's ten, Graphshell's unregistered product edits. *A third vocabulary:* `registry`'s `input::action_id` (`crates/system/registry/src/input.rs`), 46 string constants for keybinding defaults, partly the same verbs under other keys (`graph:node_new` beside `node:new`, `workbench:undo` beside `persistence:undo`), plus zoom, select-all, reheat and help; nothing outside the registry crate, nor Turnstone, reads it (checked). *Corrections:* SE29's "43 string commands": Graphshell's page dispatches 49 named ids, 32 of them registered; SE46's and SE47's "about 130 actions": 68 (a line count was misread as a variant count).
- **F13, the draft catalogue (C2, SE49, put to Mark 2026-10-08; accepted as SE50).** 33 ids in eleven namespaces named for what each acts on, in the stack's words (TERMINOLOGY: *relation* in user copy where the kernel says edge; panes, the workbench and the Trail as surfaces; *frame* is a frame-tree leaf, so the old `frame:*` set stays stale). 27 come from `ActionId`'s 28 with `node:add_to_frame` folded into `relation:add`; six are new shared verbs (zoom in and out, navigation).
  | id | label | from |
  |---|---|---|
  | `node:new` | New node | `node:new`; registry `graph:node_new`; Graphshell `add-address` |
  | `node:delete` | Delete node | `node:delete` and `node:mark_tombstone`, merged; registry `graph:node_delete`; Turnstone Delete node (recoverable) |
  | `node:edit` | Edit node | `node:edit_tags`; registry `graph:node_edit_tags`; Graphshell `open-detail` |
  | `node:pin` | Pin | `node:pin_selected`; registry |
  | `node:unpin` | Unpin | `node:unpin_selected`; registry |
  | `node:pin_toggle` | Pin or unpin | `node:pin_toggle`; registry |
  | `node:viewer_auto` | Choose viewer automatically | `node:render_auto`; Turnstone `SetViewerOverride` with no viewer |
  | `relation:add` | Add relation | `edge:connect_pair` and `node:add_to_frame` (a `frame-member` relation), merged; registry; Graphshell `add-relation` |
  | `relation:add_both` | Add relation both ways | `edge:connect_both`; registry |
  | `relation:retract` | Retract relation | `edge:remove_user`; registry; kernel `retract_relation` |
  | `identity:webfinger` | Import from WebFinger | `node:import_webfinger`; gazette `fetch_import` |
  | `identity:activitypub` | Import ActivityPub actor | `node:resolve_activitypub`; gazette intake |
  | `identity:nip05` | Resolve NIP-05 | `node:resolve_nip05` (planned) |
  | `workbench:split_beside` | Open beside | `node:open_split`; Turnstone `WorkbenchSplitBeside` |
  | `workbench:split_out` | Split out | `node:detach_to_split`; Turnstone `WorkbenchSplitOut` |
  | `workbench:stack_onto` | Stack onto | `node:move_to_active_pane`; Turnstone `WorkbenchStackOnto` |
  | `view:fit` | Fit to view | `graph:fit`; Graphshell `fit-content`; Turnstone Fit view |
  | `view:zoom_in` | Zoom in | registry `graph:zoom_in`; Graphshell `zoom-in` (new, shared) |
  | `view:zoom_out` | Zoom out | registry `graph:zoom_out`; Graphshell `zoom-out` (new, shared) |
  | `physics:toggle` | Play or pause physics | `graph:toggle_physics`; registry; Graphshell `toggle-physics`; Turnstone Play/pause physics |
  | `physics:settings` | Physics settings | `graph:physics_config`; registry `workbench:open_physics_settings`; Graphshell `apply-physics`; Turnstone's Physics rows |
  | `session:undo` | Undo | `persistence:undo`; registry `workbench:undo`; Graphshell `session-undo` |
  | `session:redo` | Redo | `persistence:redo`; registry `workbench:redo`; Graphshell `session-redo` |
  | `session:save` | Save session | `persistence:save_graph`; Turnstone Save session (Graphshell's Save scene saves view state, and stays its own) |
  | `pane:settings` | Open Settings pane | `workbench:settings_pane`; Turnstone |
  | `pane:trail` | Open Trail pane | `workbench:open_history_manager`; registry; Turnstone |
  | `pane:workbench` | Open Workbench pane | `workbench:toggle_workbench_overlay`; registry; Turnstone |
  | `palette:open` | Open command palette | `workbench:command_palette_open`; registry; Turnstone's `>`, Graphshell's menu |
  | `import:bookmarks` | Import bookmarks | `import:bookmarks_from_file`; `crates/import` parser |
  | `nav:back` | Back | registry `toolbar:navigate_back`; Turnstone; mere-chrome's history (new, shared) |
  | `nav:forward` | Forward | registry `toolbar:navigate_forward`; Turnstone (new, shared) |
  | `nav:reload` | Reload | registry `toolbar:navigate_reload`; Turnstone (new, shared) |
  | `nav:stop` | Stop loading | Turnstone (new, shared) |
  Registry: 21 of its 34 default bindings land on these ids. The other 13 have no shared verb: `graph:select_all`, `graph:select_visible`, `graph:clear`, `graph:reheat_physics`, `graph:zoom_reset`, the two fit locks, `workbench:help_open`, `workbench:open_camera_controls`, `workbench:toggle_semantic_tab_group`, and the stale or dropped `graph:toggle_overview_plane`, `graph:radial_menu_open` and `graph:cycle_focus_region`. Of its 46 constants, 4 are binding ids and 42 action ids; the 8 action ids no default binds are `toolbar:submit`, `graph:view_confirm` and the six `radial_menu:*`.
- **F14, the value engine on real rhai (R2, 2026-10-15).** `crates/script/rhai/src/value.rs` against Genet `ae3d9f5`.
  - *The trait is the branch's.* `ae3d9f5`'s `value.rs` differs from `27477d3e` only by a `rust-version` line in the crate manifest (F10).
  - *The probe's two sharp edges.* It took each host-function argument out of its slot (`mem::take`), which would empty the caller's own variable under a method-style call (`a.first()`); the engine now copies, and a test pins it. And it reported a host function's refusal as a script error; the engine now carries the `ValueError` through rhai's runtime error, so `Host` comes back as `Host`.
  - *Rhai reads an absent map key as `()`, and `() > 2.0` is `false`.* A rule on `facts.weight` for a fact the occurrence lacks is a quiet non-match, as the declarative conditions say "fact: missing", and `!(facts.weight > 2.0)` matches, as `Not` does there. A script wanting the fact to exist asks `"weight" in facts`.
  - *The identical bits.* The rule hashes to `028dc38fdcbf650d` natively (Windows, `libm` 0.2.16, no default features) and under Node 24 on wasm32, which is F8's recorded `028dc38f…` for rhai on `libm`. The wasm build needs `--cfg getrandom_backend="wasm_js"` (getrandom 0.3 through rhai's `ahash`), which the runner passes and the root manifest does not carry; `wasm-bindgen` 0.2.129 matches the lock.
  - *No new packages.* The lock gained six dependency edges (`script-engine-api`, `libm`, `scenograph`, `getrandom` 0.3, `wasm-bindgen`) and no packages; every build ran `--offline`.
- **2026-10-07.** Plan written from the chat assessment; rulings SE1 to SE9 recorded. The balaur brief and the projection grammar handoff carry dated pointers here.
- **2026-10-07, E1 built on `sceno-editor-e1`, not merged.** `cambium::History<S, K>` in `crates/cambium/cambium/src/editor.rs`: key-and-window coalescing on a host-supplied millisecond time, a saved marker that shifts with the cap and is lost when its state falls off the cap or sat on a discarded redo stack. `EditHistory` is a newtype over `History<TextSnapshot>`, since `TextSnapshot` is crate-private; its API is unchanged and `TextInput` calls it as before. Eight new tests; `cargo test -p cambium --lib` passes, 257 tests. Two controls failed where they should: dropping the redo-discard reset, and dropping the cap shift, each fails one test. A first version of the redo-discard test passed under its control, because it never returned to the saved depth; it was fixed before the control was rerun. Found in passing: Cambium is not `cargo fmt`-clean at origin (`atlas.rs`, `graph_canvas.rs`, `lib.rs`, `workspace.rs`); `editor.rs` is.
- **2026-10-07, E1 landed.** Mark: **"Merge E1, then E2 (Recommended)"** (options: that; E2 on the same branch first; stop). Rebased on origin with no change, `cargo test -p cambium --lib` rerun (257 pass) and `cargo check -p graphshell` clean, then fast-forwarded to main at `de06e4f0`.
- **2026-10-07, SE10 to SE13; `edit-history` founded.** E2's first step found Graphshell's native build has no Cambium (46 crates to add), and Mark ruled a leaf crate (SE10), two layers over Eidetic (SE11), saves into the mere as E2b (SE12), and the name (SE13). `History` and its eight tests moved to `crates/cambium/edit-history`; cambium re-exports it. `cargo test -p edit-history -p cambium --lib`: 8 and 249 pass, the same 257 as before. Note: cambium is `publish = true` and now depends on an unpublished crate, so publishing cambium needs `edit-history` published first.
- **2026-10-07, E2 landed.** `ProjectionEditor` keeps an `edit_history::History` of (draft, panel), keyed by the action's variant, with a 400 ms window (`COALESCE_WINDOW_MS`) on the host's clock (`reduce` now takes `now_ms`; the web host passes `js_sys::Date::now()`). An edit that leaves the draft unchanged records nothing; `save` marks clean, and the executable path's own save calls `mark_saved`. The web host gains `undo-projection` and `redo-projection` commands, Undo and Redo buttons disabled when there is nothing to step, and an unsaved-changes line. Five new tests; `cargo test -p graphshell --lib projection_editor` passes 15. Control: with the panel return removed, one fails. The `graphshell-web` wasm check passes. Headed: `projection_undo.scn` in Chrome (tab visible) passed 36 steps over 147 frames; capture and result in `Code/testing/mere/scenograph-editor/`. Its first run failed one assertion of the scenario's own: a click on the disabled Undo button does nothing, so the scenario now asserts the button disabled and runs the command. The new line first rendered at body size and now shares the status lines' style. Findings for later: two fields that send the same action (`source.authority` and `source.domain` both send `SetSource`) coalesce if edited within the window, *reading, not ruled*, acceptable; the executable path (`save_live_projection`) and the sink both write localStorage, which E2b replaces; `wasm-bindgen-cli` on this machine is now 0.2.129 (Mark: "Replace the global CLI"), matching the lock.
- **2026-10-07, E2b landed.** pandect: `undo_now`, `redo_now` (a revert journaled, not stored; the async `undo` and `redo` now call them, unchanged in behaviour) and `has_unstored`. MereHost: `prepare_store`, `store`, public `staged`, `undo_now`, `redo_now`, `has_unstored`; `Staged` is public with `write` and `is_empty`. Graphshell: `save_projection` and `saved_projection` (one change through `graphshell.projection-editor`, node at `mere://projection/<id>`, facet `graphshell.projection-definition/v1`, value scenomise's `ProjectionSnapshot`), and `kept_summary`. The web host: a `web_session` module whose frame-pump step writes pending changes through a clone of the store and wakes the pump with `graphshell-wake`; a `data-session-store` token (`stored`, `pending`, `writing`, `failed`, and `unstored` for changes nobody asked to store) with `data-session-store-error`; both editor savers write the session and localStorage is gone from the editor; Undo save and Redo save in the editor; Undo change and Redo change pills in the top bar, drawn in the canvas chrome with hit targets beside Local and Remote. Tests: pandect 304 pass (with `has_unstored` checks); Graphshell's library with `web` 261 pass; three new product tests (reopen from the store through the split write, Undo save reaching only its channel and not Graphshell's change, kept parts named); control: saving through `graphshell` fails two of them. Headed, Chrome: `projection_session.scn` 45 steps ok, `projection_session_reopen.scn` on a fresh page ok, control on another origin with nothing saved fails both assertions, `projection_authoring.scn` 59 ok and `projection_reopen.scn` ok through the session, a real mouse click on the drawn Undo change pill undoes and Redo change redoes. Headed, Firefox 157: `projection_session.scn` 45 steps ok and the fresh-page reopen ok, read through a receipt sink (`Code/testing/mere/scenograph-editor/sink_server.py`, receipts beside it). Safari is not run. Findings: confirmed at runtime that a browser scene save reports "Scene saved" and is gone after a reload (`unknown address mere://scene/graphshell-h3`), because nothing stores it; per SE16 this is next door, now visible as `data-session-store="unstored"`, and a later editor save stores it along with its own. The page's `data-*` tokens live on `<graphshell-view>`, not `<body>`. Existing editor definitions in localStorage are not migrated.
- **2026-10-07, C1 landed.** Canvas (`pictograph`): a left press on empty canvas becomes a pan past the click slop, the canvas staying under the hand from the press, with the middle-drag's momentum; a bare click keeps the edge pick or clear; a right press starts the marquee, which selects past the slop; a right release within it leaves a `ContextRequest` (where, and the node under it) for the host (`take_context_request`). Five tests in `canvas/tests/pointer_gestures.rs`; the canvas suite passes 315 with them; control: an unreachable right-drag threshold fails one. Cambium: `command_set.rs` (`Command`, `CommandChoices`, `CommandSet`): the surface's commands lead, then the kept ones (defaults less removed, plus added), then recent ones, each once; a query searches every command by label, category and id; recents capped at eight. Five tests; control: without de-duplication two fail. pandect: `CommandMenuView` on `ViewIntent` (absent from older views), `set_view_now`, and views carried in `Pending` and `has_unstored`; the async `set_view` is now `set_view_now` plus a store. 305 tests pass, one new. MereHost: `view`, `set_view_now`, and re-exports of `CommandMenuView` and `ViewIntent`. Graphshell page: `web_commands.rs` registers 32 commands under node, canvas, session and projection, eight kept by default; the canvas never shows the native menu; a right click selects the node under it and opens the menu there, kept inside the window, search focused; each row runs its command or keeps or drops it; choices and recents go to the `graphshell`/`commands` view and the session store. The scenario lane gains `pointer <down|move|up> <button> <x> <y>`, `mark-camera` with `camera-dx` and `camera-dy`, and a `command-menu` token. Headed: `canvas_commands.scn` (48 steps: left-drag pan of at least 90 by 40, right-drag opens no menu, right click opens it, search finds a command not kept, keeping it is stored, it shows on reopening, running a row closes the menu) and `canvas_commands_reopen.scn` on a fresh page (the kept command came back from the session, then is dropped) pass in Chrome and in Firefox 157. With real input in Chrome, a right click opens the menu and a 120 by 80 left-drag moved the camera by exactly that. The first real right click showed two faults, fixed and rerun: the menu ran past the window's edge, and on empty canvas it listed every canvas command ahead of the kept ones, against SE29 ("on empty canvas the defaults and recents only"). Findings: in the narrow layout (under 720 px) the top bar's DOM hit targets, Local and Remote included, sit apart from the pills the chrome paints, a mismatch older than this track that the new pills share; a scenario that keeps state across runs needs its reopen run to restore it; a hidden or background browser tab stalls a run until it is shown (the sink receives nothing until then). Turnstone is not changed (SE32).
- **2026-10-07, C1 rebased onto 356a832c (dynamics grammar G3 landed meanwhile).** weave merged `Canvas` (G3 added `physics_composition` and `schedule`, C1 added `empty_press` and `context_request`); both sides' fields and initializers are present. On the rebased tree pandect passes 305, Graphshell 261, Cambium 254, the wasm check is clean, and the canvas suite passed 325 three times running. Its first run reported one failure (324 passed) whose name was not captured, and it did not recur; recorded here rather than called clean. The mer3ly site session's S1 (a versioned host dataset envelope in a new `scenomise::host_dataset`, `pub(crate)` on the relationship helpers) was checked against E4 and does not collide; which of S1 and E4 goes first is Mark's.
- **2026-10-07, the scripting comparison (SE3) ran.** Rulings SE36 to SE40; the probes are F8. Next: SE39's `ValueEngine` in genet, then rhai's implementation in mere after the repin.
- **2026-10-07, SE39 written.** `ValueEngine` is on genet branch `value-engine` (`27477d3e`), pushed as a branch only, for Mark or genet's lane to land (SE41); mere waits for the repin (SE42).
- **2026-10-08, E3 and E4 built.** E3: `scenograph::options` (`OptionSpec` with key, label, kind and default; `OptionKind`, the seven readers' kinds, with `refusal` in their words; `OptionDefault`: value, measured, auto, empty), deterministic JSON, two tests. E4: each of the eleven families declares its options (`Family::options`), its choice tables now named constants shared by the readers and the declaration; `Options` checks each read against the declaration (a debug assertion) and records the default it uses, so `Family::resolved_defaults(measure)` reports the numbers; `finish` refuses what the declaration does not name, with the same messages as before; `SolverCapability` gains `options`, empty by default and absent from older JSON; `validate_arrangement` refuses a registered solver's undeclared key and a value of the wrong kind before solving. The test solver `Line` now declares its `step`. Three catalog tests (each family reads exactly what it declares; every option left out resolves, matching declared values; an undeclared key is refused) and one compile test for solvers. scenomise passes 119 and scenograph 10; Graphshell still checks. Controls: dropping LSystem's `rotation` from its declaration fails three tests, and removing the solver refusal fails one. rustfmt was run over `catalog.rs`, which also normalized nine hunks that were unformatted on main before this change. E5, the editor's rows, is next.
- **2026-10-08, E5 built.** The graphshell library gains `arrangement_options` (a family's declaration, else a registered solver's), `arrangement_defaults` (resolved numbers for given items), `option_placeholder`, `with_option` (blank leaves the option out) and `with_kind` (SE44), with four tests; the projection editor's tests pass 19. The page's new `web_options.rs` draws one row per declared option in the arrangement panel: number-like inputs, a select for a choice and for a flag (default, true, false), text for a list; the placeholder is the default, with a measured default's number and words when the executable dataset gives items to measure and its words alone otherwise; a row's refusal shows beneath it in `OptionKind::refusal`'s words, the compiler's own. Rows rebuild when the arrangement changes and otherwise update without disturbing a focused row; edits go through `SetArrangement`, so E2's undo covers them. Headed: `projection_options.scn` (28 steps: Grid's measured placeholders, `columns` 0 refused and the projection not executable, 3 accepted, a switch to Spiral rebuilds the rows and a choice applies, an unknown id says so) passes in Firefox 157 and Chrome. Findings: in the narrow layout the rows' placeholders are cut by the panel's width, and the editor's button row (seven buttons since E2 and E2b) is crowded. Tracks E1 to E5 are complete.
- **2026-10-15, R2 built: rhai implements `ValueEngine`, and scripts are a rule condition.** Mark answered six questions (SE92 to SE97). `RhaiValueEngine` and `register_libm_maths` in `script-rhai`; `Condition::Script` and `RuleSet::resolve_with_scripts` in `scenograph::presentation`; the determinism check on native and wasm32 (F14). `script-rhai` passes 26 library tests, 3 in `maths_bits` and 8 in `scenograph_rules`; `scenograph` passes 25 with `scripts` off and 29 with it on; the wasm32 run under Node matches the native hash. Mark then ruled SE98 to SE101 (§1.8): the script-running code and the `script-engine-api` dependency moved behind the `scripts` feature, off by default. Open items are listed under R2.
- **2026-10-08, the Turnstone follow-up landed in Turnstone.** At `d0775a1`, by its unusual-protocols lane (§3, dated note). Nothing here moved; the plan stops listing it as waiting.
- **2026-10-08, C2 landed.** `crates/cambium/command-menu` holds `Command`, `CommandChoices` (serde; pandect's `CommandMenuView` is now this type, and a stored view reads back unchanged, with a control), `CommandSet` (`menu` returns commands; cambium draws them as `CommandItem`), `MenuSession`, and `catalogue`: the 33 shared ids of F13 with one label each. graph-kernel's `ActionId` is their typed form, 33 variants held one for one against the catalogue; its unread categories, input modes and persist keys went. mere-chrome's `CommandPaletteSession`, `CommandAuthorityMut`, `FrameViewModel.command_palette` and shell-state's re-export were removed. registry's default bindings name catalogue ids (21), 13 were dropped, and the two Enter actions stay (SE51). Graphshell registers its eight shared verbs under catalogue ids with its own surfaces. Tests: command-menu 13, cambium 251, mere-kernel 296, ux-events 38, mere-chrome 63, registry 134, pandect 309. Headed in Chrome: canvas_commands and its reopen, projection_session, h3_boot, role_controls, d3_derived_faces_detail, physics_framing_control, face_zoom, physics_still; in Firefox: canvas_commands and its reopen. Findings: physics_framing_control counts nodes, so it needs a fresh store after projection_session adds one; `graph::revert::tests::a_removed_node_comes_back_whole` flaked once on a wall-clock millisecond (3 of 3 green alone; offered as its own task); after rebasing onto main, the Graphshell web host does not build on main itself: `632c1d29` repinned genet to `e84f9c7f` in the root manifest and left `ports/graphshell/web/Cargo.toml` at `965b64e2`, so two taproots meet (reported, not fixed here).
- **2026-10-08, SE52 built, held.** Branch `c2-labels` (`cddf72ac`): the four labels, Graphshell's buttons and pills on the catalogue, and [button_labels.rs at the pre-cutover revision](https://github.com/merely-made/mere/blob/e84cfbd95b1660046d646580a8707af020a06e4b/ports/graphshell/tests/button_labels.rs) with its control. command-menu 13 and button_labels 2 pass; the wasm build and the headed check wait on main's genet pin split.
- **2026-10-08, the web host's genet pin, and SE52 landed.** Mark: "fix the web genet pin yourself". `ports/graphshell/web/Cargo.toml`'s seven genet pins move from `965b64e2` to `e84f9c7f`, the root's since `632c1d29`; the lock resolves one genet, netrender and taproot, and the host builds for wasm. SE52's labels landed on top. Headed on that build: Chrome, h3_boot and canvas_commands with its reopen; Firefox, canvas_commands with its reopen (the sink's log shows the fresh wasm fetched). Finding: on `e84f9c7f` some node bodies draw as circles where `965b64e2` drew squares, on the same stored data (a control build of `9e482f3a` showed squares). pictograph asks for them: `.gnode-representation-glyph` and `.gnode-circle` set `border-radius: 50%` (`crates/canvas/pictograph/src/canvas/build.rs`). *Reading, not ruled:* the newer genet honours a radius the older one dropped. Next door, not fixed: `ports/distillery/probe/Cargo.toml` still pins genet `965b64e2`.
  *Annotation 2026-10-08:* on the circles, Mark: **"that's also cool, if that's a fix, i can accept that"**. On the distillery pin, Mark: **"fix the distillery pin too"**: its three `[patch]` rows move to `e84f9c7f`; the probe checks for wasm32 and its committed lock resolves one genet and one taproot, also picking up `command-menu` and `edit-history` through cambium and genet's `wuff` in place of `fontsan-woff2`. Its real-model receipts were not rerun: they measure inference, which the repin does not reach, and need the fetched model matrix.
- **2026-10-09, S1 landed: the swatch grid in the projection editor.** `mere-curation` 0.1.0 published (SE78) with `SubgraphSpec` and `SubgraphKind`, forme re-exporting; scenograph's `Scope` (with `Selection`, SE83), `Swatch`, `Facet`; scenomise's `compose_facet`; graphshell's `projection_compare` and the editor's Preview | Compare, the working draft first and every other family that compiles, over the whole set and the selection, a pick applying the family as one undo step. Tests: mere-curation 1, scenograph 13, scenomise 157 (8 facet), graphshell lib 212 (4 compare), graphshell-client facet_frozen 1. Headed: `projection_compare.scn` in Chrome and Firefox; the first runs failed at the pick (buttons kept stale ids when a pick renamed cells at an equal count, so the page stopped updating), fixed by keying the rebuild on the target ids; a screenshot found frames offset half a cell from their cards (a rect footprint centres on its item), fixed in `compose_facet` with a test that every cell's items lie inside its frame; headings now keep a readable size. Open: settled frames with dynamics (SE70) wait on the dynamics lane's slot (SE69); the gloss pane is the second consumer (SE62).
- **2026-10-16, graph grammar attention and seed built and wired into Graphshell, not committed.** From the Merely Design System's graph-as-GUI bundle (DECISIONS rows *Graph grammar*, *Reset and seed*; design project `design_docs/2026-10-16_graph_as_gui_brief.md`). `scenograph::attention`: `AttentionBook<K>` holds SE91's forward / retained / ambient plus suggestions: deselection retains until dismissed, a foreground pin holds forward and is never a position pin, suggestions yield to the person's attention; `ViewState` gains `Retained` and `Suggested`. `scenograph::seed`: `identity_seed` (FNV-1a and SplitMix64 rejection sampling, no trigonometry, so every device agrees), `SeedBook` (Set seed, restore recipe), `reset_moves` (held occurrences stay), `SeedMove::at`, `RESET_EASE_MS` = 1050. This answers B1's reset baseline with the 2026-10-16 ruling: Reset refreshes to the seed; the seed is the saved one, else the active arrangement, else the identity seed. Pictograph's `Canvas` follows `selected` (synced before every selection change and each frame) and adds `member_attention`, `member_foreground_pinned`, `set_member_foreground_pin`, `toggle_selection_foreground_pin`, `dismiss_retained_member`, `dismiss_all_retained`, `set_suggested_members`, `members_with_attention`, `set_seed`, `restore_recipe_seed` and `reset_to_seed`. While paused a reset queues the whole placement as a `StopReturn`, which now carries `duration_ms`, so the host's existing return transition eases it over 1050 ms (stops keep scenotime's 520 ms default). `CanvasCommand` and `FitHost` gain `ResetToSeed`, `SetSeed`, `RestoreRecipeSeed`, `DismissRetained`, `ToggleForegroundPin`. Graphshell: the binder supplies `foreground_pinned`, `retained` and `suggested` (`ViewInputs::set_attention`) at both access call sites, and its note now names only overview as unadapted; the viewer command set and web toolbar carry local ids `reset-to-seed`, `set-seed`, `restore-recipe-seed`, `foreground-pin`, `dismiss-retained` (labels Reset, Set seed, Restore recipe seed, Foreground pin, Dismiss retained), not catalogue ids, pending naming through the viewer owner. Open: nothing marks suggestions yet (no suggestion source); a reset while playing jumps, since physics owns the bodies; whether a saved seed travels with the recipe (shelfmark) stays open, so `SeedBook` is view-local. Tests: scenograph 42 (7 attention and 5 seed new), pictograph `canvas::tests::grammar` 8 and the canvas suite, graphshell lib 327, rhai rules 8; `graphshell-web` checks for wasm32; `mere_scene_study_check.mjs` still agrees (116 contexts, 4 refusals, 6 force checks). Headed checks not run.
  *Annotation 2026-10-16:* on scene scope, Mark: **"mosaic shouldn't be limited to the viewport, but should exist on an infinite canvas too. Same behavior, just pannable, organizable, resizable. Partition being subdivisions of the viewport makes more sense."** Mosaic keeps its steady behaviour (tiles by weight, lift, shift-lift, drag tiles and groups, never reflows) on the pannable canvas; Partition is the viewport-bounded subdivision. The design project's DECISIONS *Scene defaults* row and brief §3 still describe Mosaic as the workbench split and need this amendment there.
- **2026-10-16, the graph grammar's bounded scenes: Partition, Sets and Provenance.** From the design project's turn 5 (`explorations/graph-as-gui/More Scenes.html`), under Mark's scope ruling above (Partition subdivides the viewport). sceno gains `Partition` (bounds, measured `whole`, `PartitionForm::{Tiles, Rings}`, gap) and `Setscape` (bounds, sets) in `subdivision.rs`, and `Arrangement::{Partition, Setscape}`. scenomise solves both as bounded families that also size items and draw regions (`families::Shape`, applied in `solve` like Hulls' cells): Partition is a squarified treemap nested one level by group with the unaccounted remainder as its own region, or angular sectors in Rings; Sets lays out fixed circles from the set count alone (pairwise overlap up to four, a ring beyond) and places each item by hashed rejection sampling inside exactly its sets, core or rim by weight. Both join the catalog (`partition`, `sets`; 13 families). Cartography's `Projection` carries `regions`; `PartitionAdapter` (`partition.default`, sized by `weight.degree` plus one) and `SetsAdapter` (`sets.default`, the four most common sets, the focus in the core) fill the target size. Pictograph offers Partition (by site), Sets (by tag) and Provenance (Kanban columns from derivation depth over projected provenance links, sources left, unrelated last), stores the arrangement's regions and paints them under the nodes; Free and Settled clear them. Orrery is the existing `radial.default` (rings by graph distance from the focus, unreachable on an outer ring) and Timeline and Matrix already existed. Goldens: the 22 earlier arrangements unchanged, six new recorded. Open: Mosaic on the infinite canvas; Rings and drill-in are solver-side only (no canvas control picks Rings yet); a restored scene keeps positions but not regions; tag edits do not invalidate a cached Sets layout. Tests: scenomise 191, cartography 57, pictograph 484, graphshell 327; `graphshell-web` checks for wasm32.
- **2026-10-16, Mosaic on the infinite canvas.** Per Mark's scope ruling ("Same behavior, just pannable, organizable, resizable"). `sceno::Mosaic` (origin, `unit_area` per unit of weight, aspect, gap, label band, `group_order`) and `Arrangement::Mosaic`; scenomise's Mosaic family grows its extent from the total weight rather than any viewport and splits groups, then members, by weighted balanced bisection in their given order (the specimen's workbench splits), emitting tile footprints, one region per tile and one per group; catalog id `mosaic` (14 families). `cartography::MosaicAdapter` (`mosaic.default`) weighs by `weight.degree` plus one times the person's size factor and leads each group with the person's order. Pictograph's "Mosaic (by site)" keeps a view-local `MosaicState`: dropping a dragged tile on another files it into that group just before it, on a group appends it, dragging a group's label band onto another moves the group before it, and Grow tile / Shrink tile (`CanvasCommand::{GrowTiles, ShrinkTiles}`, Graphshell's `tile-grow`, `tile-shrink`) scale the selection by a quarter within 0.25 to 8. Selection lifts a tile (an ochre outline drawn last, a little larger) and never moves one: two canvases framed alike, one selected, place identically. A new graph clears the arrangement; it is not yet saved with a scene. Goldens: the 28 earlier unchanged, two recorded. Tests: scenomise 195, cartography 57, pictograph 496, graphshell 333; `graphshell-web` checks for wasm32.
- **2026-10-16, the territory field.** The specimen's 2a, which the scene rulings keep as "the base scene language": each domain claims the ground nearest its nodes, tiered by how clearly it belongs (the margin by which the nearest domain beats the runner-up, in units of two body radii: under one tier 1, under two tier 2, else tier 3; `--g-tier-1…3` alphas 0.06, 0.11, 0.17). Pictograph's `territory.rs` computes it as a field over wherever the nodes are, so it works under forces and under any arrangement: a 48-column world grid over the visible nodes grown by six body radii, cells run-merged per row, domains as sites in sorted order coloured by the canvas's categorical palette, drawn as the layer just above the background. Not drawn past 2000 nodes. `Canvas::set_territory_field`, `CanvasCommand::ToggleTerritory`, Graphshell's `territory-field` command and Territory button; Graphshell's web canvases start with it on (DECISIONS, Scene defaults: Graphshell Territory). Meaning keys as domains, the soft edge and the hidden current are not built. Tests: pictograph 499, graphshell 333; `graphshell-web` checks for wasm32; not yet looked at in a headed browser.
- **2026-10-16, the grammar's node and link presentation.** DECISIONS *Graph grammar*, on when a host asks (`Canvas::set_graph_grammar`; Graphshell's web canvases turn it on). Forward nodes (selected or foreground-pinned) are bodies: round, ringed, lifted (`gnode-body`). Everything else is a square site filled with its category, its site's colour from the palette territory and community rings share (`gnode-site gnode-cat-N`). Retained nodes carry a dashed underline, suggestions are the smallest mark (0.45 of a face, faded), and captions show only for forward nodes, their neighbours by projected relation, retained and hovered nodes (`gnode-quiet` hides the rest). Links stay quiet (a hairline at 0.22 of their alpha, no marks) until they touch a forward or hovered node. Activation colours give way to category under the grammar, per the superseded 2026-08-29 rule; the activity badge is not built, nor hiding inferred, same-resource and contains links (link appearances do not yet carry those kinds). Tests: pictograph 503, graphshell 333; `graphshell-web` checks for wasm32.
- **2026-10-16, per-host suggestion settings, off by default.** Mark: **"Per host suggestion settings, off by default."** `scenograph::suggestion::SuggestionSettings` (on, at once, things kept track of, mark: dot, ring, glyph, face or word, sources) with `for_host` carrying DECISIONS Open 8's proposed pace and mark for each host, every one off. `budget` tapers over the last tenth of the tracked number and as the plate fills (rounded), and `admit` takes the offered candidates in order within it and the allowed sources; the tracked number never limits content. Pictograph keeps the settings and the last offer: `offer_suggestions` (and `set_suggested_members`, now an offer from no particular source) admit through them, a settings change re-admits, `plate_fill` is the faces' area over the viewport, and a suggestion carries `gnode-mark-<mark>`. `CanvasCommand::ToggleSuggestions`; Graphshell's `suggestions` command and button, its settings saved in the browser under `graphshell.suggestions` and overlaid on its host defaults at start. No suggestion source exists yet, so nothing is offered. Tests: scenograph 47, pictograph 507, graphshell 333; `graphshell-web` checks for wasm32.
- **2026-10-16, the saved seed and the arrangement's regions travel with a scene.** `CartographyGeometry` gains a serde-defaulted `seed` (`None` is the recipe seed) and `regions` (`SavedRegion`: contour, label, member ids), captured by `Canvas::cartography_geometry` and restored by `Canvas::apply_cartography_scene_state`, which Graphshell's `restore_scene` now calls after positions land. `CartographyGeometry::retain_members` filters every member-keyed field, and Graphshell's export filter uses it, so a narrowed scene keeps the new fields (and any later ones). An older scene loads with neither. Whether a saved seed also travels with the scene recipe (shelfmark) stays open; pandect's personal scene facets do not carry these yet. Tests: pictograph 508 (one relationship-topology test failed once in a full run and passed alone and in two reruns), graphshell 333; `graphshell-web` checks for wasm32.
- **2026-10-16, the person's Mosaic arrangement travels with a scene.** `CartographyGeometry` gains a serde-defaulted `mosaic` (`SavedMosaic`: tiles moved out of their site's group, the group order, the member order and the size factors, by member id), captured only when the person arranged something and restored into the canvas's `MosaicState` by `apply_cartography_scene_state`; `retain_members` filters it. Re-picking Mosaic on a restored scene re-projects the same mosaic (tested region for region). Tests: pictograph 509, graphshell 333; `graphshell-web` checks for wasm32.
- **2026-10-16, theme roles, fonts and the grammar's categories follow the theme.** Landed the Merely design system's theme and font change for Graphshell: tinct 0.2.0 adds `derive_gated_roles` (accent, focus, success, danger, border, border-strong, five category colours, each mixed toward white or black only where it misses 4.5:1 for text or 3:1 for marks; tested against the design system's hex values); tabard 0.0.2 publishes them as `--tabard-color-*` (success and danger gated), takes per-theme `CategoryOverrides` (still gated), and adds the built-in Merely and Graphshell night themes; Graphshell opens on Graphshell night, and `styles.css` reads Tabard roles with its old colours as fallbacks. Fonts: IBM Plex Sans replaces GraphshellSans and Plex Mono is the code face, under the OFL; the viewer build carries only Plex Sans (the same gzipped size as the face it replaces) and Mono rides in the product. Newsreader, the reading face, is named first in the reading font list but not yet in the tree. Fixes over the pasted diff: the "working" capture status stays ink rather than the success colour; the backdrop pane's second font load. The grammar's sites and the territory field now take the theme's category colours: Graphshell samples `category-1…5` (falling back to the triad and dim text for an authored sheet) and passes them to `Canvas::set_category_colors`; sites carry the colour inline, bodies and bare dot or ring suggestions do not, and an empty set restores the canvas's eight. Tests: tinct and tabard 114, pictograph 510, graphshell 333 plus the first-run theme; the workspace checks. The viewer (`--no-default-features`) wasm build was already failing in `promoted_links.rs` and still does.
- **2026-10-16, after the theme handoff.** Checked what the design system's handoff left open: tabard's `tests/theme.rs` (10, run with Genet), `pelt-desktop`, and the pictograph and graphshell examples all build or pass. `backdrop_receipt` (which lends only Roboto) passes and its frames render text, so board titles resolve through the generic family. Plex Mono's provenance: the landed file is IBM Plex Mono Regular, Version 2.3 (1033 glyphs), copied from Woodshed's `assets/fonts/ibm-plex-mono/`, whose README records it as an unchanged copy of github.com/IBM/plex with its family name and OFL notice kept; the patch's file was Version 2.005, and either is IBM's release. Mono loads only in the product build, by intent (the viewer's size budget). The pictograph title sheets now say that no host publishes `--tabard-font-*` into those surfaces, so the fallback is the face that renders. Newsreader stays open for Mark.
- **2026-10-16, Graphshell on both seed sets (DECISIONS Open 2).** Ran the web build in the browser on `theme:graphshell_night` (dark, the first run) and `theme:merely` (light and dark): the page, panes, controls, rule and node faces follow each theme, and the picker lists all six built-ins. Three surfaces did not, now fixed: the appearance panel's theme-import field kept Genet's default dark `Field` fill (the host sheet now themes `.tree-appearance` fields); the "mere" brand label used Cambium's `--workspace-accent` (the primary fill) as text, near-invisible oxblood on Merely dark, and now reads `--tabard-color-accent`; `.action-choice.selected` and `.remote-card-kicker` likewise read the gated accent rather than primary. `styles.css` has no hard-coded colour left; the Rust sheets for the applet library and the tree root still carry literal colours under the host's themed overrides. No screenshot receipt was kept (Mark's standing rule: evidence is tests and short findings).
- **2026-10-16, Newsreader is the reading face.** Mark: ship it. The 16 pt, weight 400 instance of Google Fonts' `Newsreader[opsz,wght].ttf` (google/fonts `ofl/newsreader`, 451,664 bytes, sha256 `8a08d13f…`), cut with fontTools 4.67 `varLib.instancer opsz=16 wght=400 --update-name-table`: `Newsreader-Regular.ttf`, family Newsreader, 118,280 bytes, with `OFL-Newsreader.txt` (no reserved font name, so the instance keeps the name). The reading host lends it first and defaults to it; the page loads it in the product build beside Plex Mono, so the viewer still carries Plex Sans only; `--workspace-font-reading` is `Newsreader, serif`; the extension packaging copies it and its notice. DECISIONS Host fonts is met. Checked in the browser: a retained document's body reads in Newsreader. The same check found Cambium's `instrument.css` colouring the selected pane title and state with `--workspace-accent` (the primary fill), near-invisible on Merely dark; a new `--workspace-accent-text` reads Tabard's gated accent (falling back to primary) for those two text rules, fills unchanged.
- **2026-10-16, the grammar state in the personal workspace's facets.** Pandect's arrangement facets carry the saved seed (`arrangement.seed`, per node, `None` clears it), the regions (`arrangement.regions`) and the Mosaic arrangement (`arrangement.mosaic`), the last two on the scene's container node as the `scene.*` facets are; malformed data falls back to empty for that group only, and a facet set written before them still loads. `ArrangementRegion` and `ArrangementMosaic` mirror pictograph's `SavedRegion` and `SavedMosaic` (pandect does not depend on pictograph). No host yet reads arrangement facets back into a canvas; the one that does should call `Canvas::apply_cartography_scene_state` after positions, as Graphshell's `local_scene.rs` does, and keep the container id in `retain_present_nodes`' set. Tests: pandect 352.
- **2026-10-16, Graphshell's Rust sheets read Tabard roles.** The tree page's `SHEET` (`web_tree.rs`) and the applet library's (`web_tree/applet.rs`) now write every literal colour as `var(--tabard-color-<role>, <old literal>)`, by what it does: grounds bg/surface/surface-2/surface-hover, text and text-dim, hairlines border, control outlines border-strong, the pressed tools switch tertiary with on-tertiary text, refusal edges danger. Nothing changes until a theme publishes. Graphshell has no hard-coded theme colour left. Tests: graphshell 334; both web builds check.
- **2026-10-16, Tabard's font roles reach pictograph's surfaces.** `BoardText::set_font_roles` (and `BackdropSurface::set_font_roles`, delegating) put the host's declarations in a `:root` rule ahead of the label sheets; each title is its own document root, so `var(--tabard-font-interface, …)` resolves. Empty, the default, leaves the sheets as authored. Graphshell passes `Appearance::font_roles()` (the resolved theme's typography declarations, empty when the theme sets none) before each backdrop paint and each remote-board paint, the path `backdrop_theme()` already takes, so an appearance change follows on the next frame. A family the host does not lend still falls to the lent faces. Tests: pictograph 512, graphshell 335.
- **2026-10-16, Graphshell's viewer builds again, and is over its cap.** The viewer (`--no-default-features`) had failed since `611e484` (promoted links name Cambium's link-member target under the `web` feature, which did not enable Cambium). `web` now enables `dep:cambium` (the web crate links it unconditionally, so the viewer pays nothing) and the viewer's inert `SavedProduct` gains `initial_fixture_positions`, `restore_canvas_link_hides`, `link_visibility_state` and `assertion_visibility_state`. `scripts/build-viewer.py`: WebGPU wasm 3,596,340 B gzip6 against the 3,460,300 cap (over by 136,040); WebGL2 3,453,975 (6,325 under); glue, mount and history pass. Baseline: `a4c1814c8` (the commit before `611e484`) with the same four stubs measures WebGPU 3,535,472 (already 75,172 over) and WebGL2 3,283,427, so the overage predates the break; the 45 commits since, across several lanes, added 60,868 to WebGPU and 170,548 to WebGL2, which now has 6,325 of headroom left. Mark's call the same day: raise the cap rather than slim the viewer. The Wasm cap is now 3,670,016 B (3.50 MiB) per route artifact (the figure is mine), leaving WebGPU 73,676 and WebGL2 216,041 of headroom.
- **2026-10-17, a kept link arrives linked, and the viewer is under its new cap.** Keeping a suggested link now writes the address and the hyperlink from the page it came from as one owner change (`MereHost::create_linked_address`, `local_intake::create_link`); an address already held only gains the link, and a page the person does not hold falls back to a plain address. Reading the arrangement facets back needs nothing in Graphshell: its saved scene carries the seed, regions and Mosaic through `CartographyGeometry`, and `restore_scene` hands them to `apply_cartography_scene_state`. No host uses pandect's facet store yet, positions included; adopting it is the facet-convergence migration, left open. Viewer at `a17e5bb97` against the 3,670,016 cap: WebGPU 3,549,457 B gzip6, WebGL2 3,336,403; every cap passes.
- **2026-10-16, suggestions from a page's unvisited links.** Mark: suggestions are derived graph members, saved only once kept. Fleece gains `page_links::html_links` (feature `page-links`, wasm-safe; the crawl reuses the feature): a page's links resolved against its address, deduplicated, links to itself dropped. Pictograph's `link_suggestions.rs`: `Canvas::offer_page_links(page, links)` records a page's links (skipping held ones); only as many as the suggestion settings admit are minted, each a **derived member** with a stable id from its URL, seeded by `identity_seed` around its page and linked from it, offered under the `links` source; turning suggestions off removes them. Selecting one keeps it (`take_kept_links`). Derived members are excluded from `cartography_geometry` (new chainable `retaining_members`), from the kept count, and from Graphshell's scene capture and `sync_canvas_intake`; a host reinstall drops them and the next frame mints them again. `set_graph` now keeps the host's suggestion settings and the link offers (it had reset them with the node-keyed grammar state). Graphshell: the workbench offers an opened page's links once per page while suggestions are on, and the product writes each kept link through the add-address intake (`create_address`, then the usual persist and reinstall), one at a time. The kept member's hyperlink from its page is not yet written to the owner. Tests: pictograph 514, graphshell 335, fleece page-links; both web builds check.


### Workbench Forme bridge (2026-10-10)

**Status:** implemented; native/build qualification passes; headed qualification
is pending. Mark: "alright, let's do it", asking to apply the
saved-workbench field to Graphshell. This opens the bounded bridge previously
proposed in the design-language discussion; general overlapping field behavior,
Rhai execution and other panes retain their own targets.

- Canonical membership is Forme's Arrangement; nesting, split shares and active
  tabs are Platen's TreeGeometry. Graphshell saves their pair and the field's
  extent under a graph-session key through Muniment. Surface UUIDs identify
  accesses, even where Resources are shared.
- The retained `tree.html?app=local` page presents reading tiles through Cambium
  Frisket and projects the same active cells into a quiet rectangular forme
  region in its Canvas. Hover, selection and unlocked editing reveal cells.
- Forme-local holds preserve the tile arrangement without pausing unrelated
  physics. Explicit item pins take precedence. The boundary's visibility and
  the layout-edit lock are independent of these holds.
- Existing graph-held Numen fields get an inspect/locate/hide section. A forme
  projection does not fabricate a scalar field, coupling or semantic link.

Done when nested/ratio/tab geometry and identities survive save/reopen,
removed accesses reconcile, malformed records refuse atomically, tile regions
hold while other bodies remain free, explicit pins win, and the retained
Graphshell page passes a headed open-two/return/hover/rearrange/reopen check.
Graphshell's reading tiles are its address/metadata view; live browser-engine
content continues to belong to Pelt in Turnstone.

**Implementation and qualification (2026-10-10):** the bridge, reading panes, existing-field
inspector, explicit placement previews, per-split size controls and atomic
workspace save/load are implemented. Previews update both projections; cancel
restores the saved arrangement and reports cancellation. The shared host resolves
hover handlers and delivers their local coordinates on ordinary motion; captured
drags keep owning their gesture. A graph swap releases Forme holds before reusing
node keys.

After integrating published main `e95326dc`, twenty native checks pass in an
isolated qualification crate importing the tracked sources: seven presentation
rule tests, seven workspace tests, five public Canvas integration tests and one
real-layout host hover test, including the stronger graph-switch regression.
The native rule example also agrees with the study model across 116 contexts;
four invalid rule sets refuse and six force checks pass. This is source/model
qualification, not a headed Graphshell receipt.

The integrated compact release product wasm build, viewer-only wasm check and
full default-feature wasm check pass. The uncapped debug `wasm-bindgen` packaging
step had exhausted the machine's remaining memory. The release dependency tree
was rebuilt with size optimization, no debug symbols, no LTO and one Cargo job,
inside a 1 GiB resident-memory scope with no swap. The current input wasm is
21,593,299 bytes, versus the earlier 88,386,325-byte debug input. Matching
`wasm-bindgen` 0.2.129 packages it under a 1280 MiB scope: maximum RSS 1,062,400
KiB, no swap, exit 0. The full-feature check required a 1536 MiB scope, reduced
debug information for `cubecl-ir`, and a guard stopping only the check's child
processes if available memory fell below 256 MiB; it completed without a guard
stop. These are per-command limits, not machine-wide settings.
Qualification logs are in `Code/testing/mere/forme-{tests-integrated,
rule-parity-integrated,build-integrated,viewer-integrated,
default-integrated-guard,bindgen-integrated}.log` on S-PC. Integration preserves
main's page/canvas gesture ownership and binds the reading tiles to its
configurable appearance roles.

The headed open/return/hover/edit/reopen receipt remains pending. The Codex
in-app browser on this machine fails before scenario startup with
`netrender wgpu boot failed: could not request a wgpu adapter` and
`webgpu found no adapters`. A visible-browser reload reproduced that failure;
the final integrated bundle reproduces it in a fresh tab as well.
This is a graphics-startup blocker after packaging succeeded; no interactive
Graphshell receipt or screenshot has been claimed. Run the paired scenarios in
a WebGPU-capable browser to complete qualification. The portable source scenarios
are `p4_tree_forme.scn` and
`p4_tree_forme_reopen.scn` in `ports/graphshell/web/scenarios/`.

The first bridge uses a layout-edit gate and immediate saves for accepted
gestures; move previews have their own apply/cancel. The subsequently agreed
[whole-forme draft and undo model](#forme-draft-follow-through-2026-10-10-agreed-interaction)
was the follow-through target at that checkpoint. It is now implemented in the
separate adapter below; the bridge checks above do not qualify that later work.

### Workbench Forme draft (2026-10-10)

Mark selected **"Implement Forme draft/undo (Recommended)"** after the first
bridge was published and this machine's in-app browser could not provide
WebGPU. This opens the scoped draft/history follow-through for that Graphshell
adapter, without opening generic field editors or changing another host.

The host now keeps a committed `FormeWorkspace` and an optional private draft.
Unlock clones the committed arrangement/geometry; layout gestures, including
membership and outer-region placement, edit the draft with Cambium `History`.
Graph regions and reading tiles use the same draft and temporary drop preview.
The Forme id, arrangement root and existing member-intent ids survive conversion
from the working tile tree; tile ids still address distinct content accesses.

- Unlocked graph nodes act as tile handles after four pixels of movement. Edge
  drops split; central drops tab. Another source node can be added this way.
  Release outside the field and Escape cancel the temporary preview. A completed
  drop is one draft history step; mere selection does not save an arrangement.
- Draft undo/redo and Discard changes are visible. Discard drops the private
  history and locks at the committed state, leaving source nodes and content
  intact. A new unlock starts fresh. Pending changes have a visible status.
- Lock and apply writes the canonical workspace and its saved layout history in
  one record. An unfinished tile preview must be applied or cancelled first.
  While writing, session mutations wait. A failed write retains the draft and
  its history, and the committed record/cursor stay unchanged for retry.
- A successful apply locks and becomes one saved arrangement undo step. Saved
  undo/redo survives reopen; its snapshots contain semantic arrangement, tree
  geometry and region bounds, with a cap of 32 steps and a 4 MiB record limit.
  Node content, resources, source edits, boundary visibility and document
  sessions are outside those snapshots. Removed graph members reconcile in
  every reachable saved layout, so undo cannot resurrect removed source nodes.
- Keyboard undo is scoped to an open graph Forme draft, Forme controls and the
  reading workbench. A locked graph view does not capture saved-Forme shortcuts;
  saved-layout undo remains available in the Forme controls and workbench.
  It does not intercept keys at the whole-page/document level or override an
  already handled editor key. Existing text/editor histories retain ownership.
  Divider samples can coalesce within 400 ms; discrete size buttons and completed
  drops break that run. General tiled-divider drag plumbing remains host-owned.

**Qualification:** 28 native checks pass: eight new draft/history cases plus the
previous twenty presentation, workspace, Canvas and real-layout hover checks.
They exercise nested geometry and membership undo/redo, stable identities,
preview cancellation, source preservation, discard/new-unlock, injected write
failure and retry, one saved step per applied draft, reopened saved undo/redo,
independent visibility, removed-member reconciliation and malformed-history
refusal. Product and default-feature wasm checks pass; viewer-only checking
also passes. The compact release product builds and `wasm-bindgen` 0.2.129
packages its 21,718,200-byte wasm under the 1280 MiB cap (maximum RSS 1,070,348
KiB, no swap, exit 0). The [portable qualification summary](../../../ports/graphshell/docs/receipts/forme_draft_20261010/qualification.json)
records source hashes, counts, build scope and the open browser gate. Logs are
`Code/testing/mere/forme-draft-{native,product-check,default-final,viewer-check,
release,bindgen}.log` on S-PC. Browser compilation is source
qualification; no interactive receipt is claimed while WebGPU is unavailable.

The existing `p4_tree_forme.scn` now checks that draft edits leave committed
geometry/bounds unchanged until Lock and apply. The added
`p4_tree_forme_draft.scn` and `p4_tree_forme_draft_reopen.scn` extend the same
local-origin sequence with real host pointer handle drags, draft undo/redo,
added membership/discard, saved undo and reopened redo. Those headed scenarios
remain pending on a WebGPU-capable browser.

#### Forme visual test handoff (2026-10-10)

**Status:** ready for another machine to run; no host or browser pass claimed.
Mark assigned this machine continued planning and expects agents on O-PC, the
ThinkPad or the Macs to test. This record supplies the shared handoff through
the repository; it does not claim that another chat received or accepted work.
The implementation to qualify is Mere `69f1790834a0de8885a4ab58f907343b09d2d0a6`.
If testing a later main, record that revision and distinguish intervening source
changes from this receipt's hashes.

Build the [web host](../../../ports/graphshell/web/Cargo.toml) with `product`
enabled, package with the lock's `wasm-bindgen` 0.2.129, and serve the web
directory with its generated `pkg` beside `tree.html`. The scoped source build
used `--no-default-features --features product`; default-feature build checks
also passed. The viewer-only build has no local-product adapter. The committed
[qualification summary](../../../ports/graphshell/docs/receipts/forme_draft_20261010/qualification.json)
is portable evidence of source checks; its S-PC log paths and build outputs are
local artifacts, not prerequisites on another machine.

Use a dedicated browser profile/origin with empty Graphshell IndexedDB for the
first run, then retain that same origin, profile and storage for this sequence.
Open a fresh page for every row; do not clear storage between rows. Keep the tab
visible and wait for completion before navigating onward.

| Order | Route relative to the served web directory | Purpose |
|---|---|---|
| 1 | `tree.html?app=local&scenario=scenarios/p4_tree_forme.scn` | Create two accesses, hover subdivisions, preview/edit and apply the arrangement. |
| 2 | `tree.html?app=local&scenario=scenarios/p4_tree_forme_reopen.scn` | Reopen committed layout and focus an existing access. |
| 3 | `tree.html?app=local&scenario=scenarios/p4_tree_forme_draft.scn` | Real host pointer drops, draft undo/redo/discard, membership and saved undo. |
| 4 | `tree.html?app=local&scenario=scenarios/p4_tree_forme_draft_reopen.scn` | Reopen the saved redo cursor and verify stable forme identity. |

The [loader](../../../ports/graphshell/web/loader.js) accepts an additional
`sink` query parameter naming a receipt endpoint; it posts the result and
captures to `/scenario-receipt` and live progress to `/scenario-progress`.
Existing host test harnesses can collect these, or collect the scenario result
and named captures from the page. Use separate clean profiles/origins for each
browser's four-run sequence. Record browser/version, OS, viewport, WebGPU boot
result, source revision, feature set and generated wasm hash with each receipt.
Chrome and Firefox checks are the existing headed targets; record Safari on a
Mac if run, keeping unsupported boot separate from an interaction failure.

Review the captures as well as scenario assertions: graph regions and tiles
should agree, hover guides should remain legible, and drop previews should be
distinct from committed layout. Add real pointer and keyboard checks for
Escape/outside-release cancellation, click-without-drag selection, undo in an
open draft, saved undo in Forme controls, and another editor retaining its own
undo. The scripted lane queues host pointer input; it does not qualify every
physical-device/focus path. Failure/retry already has an injected native test;
do not claim a browser persistence-failure receipt unless it is exercised there.

Done: all four runs and reviewed captures are tied to the tested build, reopen
really uses retained storage, and focus/cancellation observations are recorded.
Append results here, link portable receipts, and leave any failed or unrun gate
explicit. A fresh rerun after a partial failure starts with a new dedicated
origin/profile because the sequence intentionally changes retained state.

#### Planning and remote-machine qualification (2026-10-10)

Mark: "you keep working plans and the other agents on o-pc, the thinkpad, or the
macs will test it." The identity inspection follow-through and field inspection
adapter sequence now name available read APIs, missing roster subjects and the
write gates for shared sources, exact assertions and individual couplings.
The forme handoff records its published revision, product-feature requirement,
`app=local` route, four fresh-page runs on one retained origin/profile, capture
review and extra physical-input/focus checks. The field plan's earlier pending
forme-draft wording is reconciled with the implemented adapter. All are plan
updates; no new runtime source, browser receipt or agent dispatch is claimed.

#### Draft delivery authorization (2026-10-10)

Mark proposed publishing implementation drafts directly for the other agents to
review, test and improve, including when this machine cannot run their host
checks. The delivery workflow and five bounded candidates above record that
authorization. This pass changes documentation only; no candidate implementation
or new runtime qualification is claimed. The canonical working principles now
preserve direct-main draft delivery and explicit check status for future passes.

#### Access/Resource inspector draft (2026-10-10)

**Status:** implemented draft for other-machine review and host testing. Mark
approved starting the candidate sequence ("hell yeah, go ahead"). This pass
implements the access/Resource inspector first, based on `d541416e`; it does
not complete the broader primitive-editor walkthrough.

The portable [inspection model](../../../ports/graphshell/src/access_inspection.rs)
reads the owner's graph by Surface/member UUID and follows its recorded shown
Resource association. It returns the access's title/address/identity, shared
Resource identity/address/tags and held facts, and other accesses held here.
Known coverage limits are scoped to the target Resource and its held accesses,
while global limits remain visible. Even an empty coverage note never claims
the access list is globally complete. Facts retain unknown keys and values;
no attribute storage or schema rename is introduced.

The [retained inspector](../../../ports/graphshell/src/web_tree/inspection.rs)
is available on `tree.html?app=local`. Select an object and activate **Inspect
selected access**. Its panel appears in Graph tools, opening the narrow-window
tools overlay when necessary. **Inspect access: …** beside another held access
changes only the inspection target. Selecting another graph object does not
retarget the open inspector; **Close access inspection** clears inspection.
These controls neither open content nor change workbench membership. Existing
detail editing retains its selection and draft. Reads use the local owner,
not its Canvas clone or unsaved detail values; the chosen target is retained
while an in-flight save temporarily makes that owner unavailable. A removed
target reports absence rather than looking up another object at the same URL.

The panel composes existing Cambium detail sections and buttons with the host's
current appearance sheet. Generic Resource facts are a first inspection
presentation; semantic labels, large-value disclosure and the fuller attribute
editor remain review work. Inspector state is temporary view state, not saved
graph truth. Hosted datasets, generated graphs and other hosts do not yet mount
this local-product panel. Link inspection and general Resource writes remain
the next adapters, with their existing ownership gates.

**Qualification:** six model tests pass through the bounded native harness,
importing the tracked inspection source and actual Mere graph dependencies.
They cover distinct accesses sharing a Resource, immutable inspection, missing
association without address-based inference, changed live bindings, scoped
coverage, shared-fact/snapshot roundtrip and removed-target refusal. This is a
Graph snapshot roundtrip, not an IndexedDB reopen receipt. The product wasm
check passes from `ports/graphshell/web` with its committed target flags and
`--no-default-features --features product`. Both checks used one Cargo job, a
1280 MiB process-scope cap, no scope swap and the 256 MiB available-memory guard.
Formatting and whitespace checks pass. The
[qualification summary](../../../ports/graphshell/docs/receipts/access_inspection_20261010/qualification.json)
records exact source hashes and local log references. No new release bundle,
default/viewer check, browser run or capture is claimed; headed verification
remains with the other machines.

**Review and host-test handoff:**

- Build the web host with `product` (as in the Forme handoff) and run
  `tree.html?app=local&scenario=scenarios/p4_tree_access_inspection.scn` in a
  visible tab on a dedicated origin/profile. This
  [scenario](../../../ports/graphshell/web/scenarios/p4_tree_access_inspection.scn)
  checks independent selection/inspection and unchanged Forme identity and
  membership, with two named captures. It does not prove two accesses sharing
  one Resource in the browser; that case has a native fixture and needs a host
  fixture/receipt as the mixed-content example develops.
- Review long addresses/facts, empty and missing associations, known coverage
  disclosures, two equal-address accesses with distinct titles, and changing a
  held access's shown Resource. Check keyboard activation and focus in both
  docked tools and the narrow overlay, including an open detail editor.
- Confirm another-access inspection leaves graph selection, camera, content
  sessions and Forme layout alone. Test source saving/failure and target removal
  while inspection remains open. Append actual results and tested source/build
  identity here; the added observation fields are instruments, not receipts.

#### Forme identity and validation fixes (2026-10-11)

**Status:** source fixes with native qualification and a passed product
Wasm host check. Mark asked to fix the review findings and inspect nearby paths.
The source began at `1081987c` and was rebased onto published `c397f33b`; this pass
preserves the existing scoped draft, saved layout history and v1 record format.

- The shared arrangement matcher reuses a member-intent ID only when that
  member occurs once in both arrangements and the ID is free or already belongs
  to that occurrence. Repeated appearances keep their supplied occurrence IDs.
  This prevents both appearance collapse and overwriting a node rebound to
  another kind. It does not invent correspondence between rebuilt duplicates.
- Workspace validation checks positive, finite projected cell extents in both
  relative and world coordinates. Positive split shares alone were insufficient:
  normalization or coordinate rounding could produce an empty tile region.
- Divider events require exactly one share per split child. Too few or too many
  shares previously changed only the children matched by a zip. Invalid events
  now leave the workspace unchanged.
- Workbench reading tiles use the shared Resource content-tag accessor, retaining
  its legacy fallback for unassociated accesses. Titles and authored bodies
  remain access-specific. This follows the existing access/Resource boundary.

Three native identity regressions and three workspace regressions failed before
these fixes. Afterward all 120 Forme library tests and 19 Graphshell workspace/
session tests pass. The product web host also passes its locked, offline Wasm
check. The added session case checks that collapsed geometry cannot enter draft
history and that an invalid saved undo state is refused
without rewriting its retained bytes. Workspace tests now live in a child
module, including the existing stable-ID and tab-placement cases.

The [qualification summary](../../../ports/graphshell/docs/receipts/forme_fixes_20261011/qualification.json)
records exact source hashes, commands and check status. The earlier `69f17908`
receipt remains historical evidence; it does not qualify these changed sources.
Run the existing four-page retained-origin Forme sequence on the new build and
review pointer previews, saved undo/reopen and shared Resource tags in separate
accesses. No browser pass, capture, release package or physical-input receipt is
claimed by this pass.


#### Link and exact-assertion inspector draft (2026-10-11)

**Status:** implemented read-only draft, based on published `ebfa5be8`, under
Mark's direct-main draft-delivery authorization, then rebased onto `3d21c352`
before delivery. It preserves the Forme fixes and scene/dynamics work now on
main. The integrated primary-model reconciliation changes documentation only;
qualified runtime/scenario hashes remain unchanged. Field inspection and overlap discovery
remain the next bounded adapter; this pass introduces no primitive writes.

The portable [link model](../../../ports/graphshell/src/link_inspection.rs)
resolves a bundle by two access UUIDs and expands both directed pairs, visiting
an access self-pair once. Each contributing edge bucket retains its displayed
direction, owning Access/Resource stratum, stable source endpoint identities,
exact semantic statements and actual family sidecars. Parallel buckets are
retained; deterministic content ordering does not assign them durable identities.
One Resource relation lifted onto distinct access pairs shows the same source
owner and statement handles with different displayed endpoints. A Resource
self-link lifted between two distinct accesses contributes once per displayed
direction. Missing recorded associations and relevant known coverage limits are
disclosed; held records never imply global completeness.

Each assertion retains its handle, predicate, recognized kind, label, scope,
asserter and assertion time. Exact inspection re-resolves the held handle in
its recorded store, independent of current predicate placement or access binding.
If an access navigates away from a Resource, its bundle refreshes while an
already inspected Resource assertion remains inspectable. Retraction or access
removal reports absence, without substituting a same-predicate assertion or an
equal-address access. Non-statement sidecars retain their real owning values,
including session arrangements, traversal events and navigation aggregates;
no statement handles are manufactured for them. Family summaries are labelled
as grouping, separate from exact assertions.

The [retained controls](../../../ports/graphshell/src/web_tree/link_inspection.rs)
are mounted in Graph tools on `tree.html?app=local`. Select an access, activate
**Inspect selected access**, then **Inspect links: …** in its held-link list.
**Inspect assertion N** retains that row's exact statement handle; N is a
presentation ordinal only. Both endpoint directions and Resource source
identities are visible. Graph selection changes do not retarget either panel.
Closing access inspection leaves link inspection open; closing the exact
assertion leaves its bundle open; closing the bundle clears both link targets.
These actions change temporary view state without opening content, changing
selection, moving the camera or editing Forme membership/layout. Reads use the
acknowledged local owner, retaining targets during its in-flight save.

**Scope and findings:** Canvas currently exposes selected-link presence, but
not a stable public bundle subject for this host. The access inspector's incoming
and outgoing held-link list provides the initial keyboard/pointer entry path;
direct canvas-link activation and shared roster subjects remain follow-through.
The list covers held access neighbors, including lifts through recorded
Resource associations. A Resource with no held access cannot appear as an
adjacent displayed access. This is not a general Resource graph browser.

Rule-derived display marks are not yet connected to this adapter. A held
`AgentDerived` assertion shows its stored attribution and explicitly reports
that its rule and inputs are not held in that record; an asserter is not a rule
explanation. Other derived marks still need their execution owner's explanation
adapter. Unknown-legacy attribution remains the recorded marker. Generic JSON
sidecar values, long handles, disclosure sizing and large-neighborhood costs
need presentation review. Other hosts and generated datasets are not mounted.
Exact-record mutation still requires the existing authorship/retraction contract.

**Qualification:** nine actual-source native model tests pass. They cover
same-predicate assertions with distinct handles/attribution, lifted Resource
ownership across access pairs, both directions and strata, parallel buckets,
self-links, non-statement session data, mixed family/traversal data, exact lookup
after rebinding/retraction, missing targets and associations, scoped coverage,
and stable bundle/assertion Graph-snapshot roundtrip. The initial external
harness needed adjustment for main's extracted Forme test module; the dedicated
link harness imports the tracked link/access sources and actual Mere graph
implementation. The Graph-snapshot check does not qualify IndexedDB reopening.
The product wasm check passes with the web host's committed target flags and
`--no-default-features --features product`; exact scope is recorded in the
[check summary](../../../ports/graphshell/docs/receipts/link_inspection_20261011/qualification.json).
Native and product checks run sequentially with one Cargo job, a 1280 MiB scope
cap, no scope swap and the 256 MiB available-memory guard. No release bundle,
default/viewer check, browser run or capture is claimed for this draft.

**Review and host-test handoff:**

- Build with `product` and run
  `tree.html?app=local&scenario=scenarios/p4_tree_link_inspection.scn` in a visible
  tab on a dedicated origin/profile seeded with the reference fixture. The
  [scenario](../../../ports/graphshell/web/scenarios/p4_tree_link_inspection.scn)
  inspects the existing webpage → file citation, opens its exact assertion,
  changes selection, closes each inspector independently and checks unchanged
  Forme identity/membership. Its two named captures and observation fields are
  instruments; the scenario has not been run here.
- Add host cases for two same-predicate assertions, parallel source buckets,
  opposite directions, self-links, session sidecars and a Resource relation
  lifted through two distinct access pairs. Review source/displayed endpoint
  labels and unknown/missing attribution. The native cases do not substitute
  for browser receipts of those interactions.
- Check keyboard activation and focus in docked tools and the narrow overlay,
  long records, another open detail draft, in-flight source saves/failure,
  access rebinding/removal and exact-statement retraction. Confirm inspection
  does not change selection, camera, content sessions or Forme arrangement.
  Append actual outcomes and source/build/browser identity here. Review the
  unqualified list/record materialization costs before large-graph adoption.


#### Forme browser qualification (2026-10-11)

Mark authorized browser qualification of the review fixes. The first Chrome
run at `ebfa5be8` failed on four unsupported `dom click` steps. At `4a47b488`,
reading-tile move and preview controls gained tile-specific accessible names;
the scenario uses the standard retained host click path, including scrolling,
hit testing and actual callbacks. The failed receipt is preserved.

The intermediate sequence passed editing, reopen and saved-history reopen;
the draft case failed on an unnecessary "Return to Mere" click because fresh
pages already show Mere. Its behavior assertions passed. That failed receipt
is preserved, and `c33534bf` fixes the setup. Capture review also found graph
controls obscuring the first workbench tab. `c433519f` places those controls
above the tiles and makes scenario waits ignore hidden-canvas motion while
waiting for Forme saves. `c33534bf` changes only the script, with identical
compiled Rust sources to `c433519f`.

The locked, offline product Wasm dev build and viewer-only Wasm check pass.
Packaging uses `wasm-bindgen 0.2.129` after stripping debug information, with a
separate package directory. The loopback-only qualification server collects the
existing scenario results, semantic receipts and PNGs. A clean worktree excludes
concurrent dependency WIP from the build. Source/build hashes and both earlier
attempts are in the [browser qualification record](../../../ports/graphshell/docs/receipts/forme_browser_20261011/qualification.json).

Chrome final sequence: **all four cases pass at `c33534bf`**, with nine reviewed
2074×1166 PNGs at a 1037×583 viewport and DPR 2. The new
`127.0.0.1:9871` origin was verified with empty IndexedDB. Each case opens in a
fresh page while retaining that origin. Preview/cancel/apply, draft undo/redo
and discard, source preservation, placement during live dynamics, saved history
and both reopens pass without scenario or runtime gate failures. The workbench
tab-title obstruction is fixed. The center-drop stack preview has a weak visual
cue because its bounds stay the same and sidebar feedback may be below the
viewport; preview-feedback design remains open. Release-step NaN diagnostic
sentinels are not movement-distance or physics qualification.

Upstream link inspection was merged at `f2bda7cf`; its product Wasm integration
check passes. This does not extend the browser receipt to that newer source.

Firefox is blocked: its Windows Computer Use controller refused browser control
because browser URL policy enforcement is unsupported. No further native input
was attempted after that refusal. Firefox acceptance remains open. This dev
build does not qualify release performance, physical touch/focus behavior or
other hosts. The browser fixture shows Resource tags on two different resources;
it does not qualify shared-Resource identity across multiple accesses. The
native 120-library/19-adapter receipts above remain their recorded checks,
not new reruns in this browser pass.


#### Field inspection continuation (2026-10-11)

The [field owner](2026-06-13_scriptable_field_regions_plan.md#field-inspection-draft-2026-10-11)
records the implemented read-only field/coupling model, retained roster, exact
identity lookup and all-candidate region discovery. Forme reads its current
arrangement owner without synthesizing a Numen field. Selection and discovery
do not retarget inspection implicitly; visibility is separate from force effects
and selector matches are separate from boundary hits. Ten native model checks
and the integrated product wasm check pass. The opt-in isolated fixture and
two-page browser handoff remain unrun here. The other agent's Forme Chrome
receipt above qualifies its recorded source, not this new inspector. Primitive
writes, derived-link explanation and the field composition/event policies
retain their existing gates; the [declarative rule editor draft](#declarative-presentation-rule-editor-draft-2026-10-11)
continues the bounded appearance-authoring sequence.


#### Declarative presentation rule editor draft (2026-10-11)

**Status:** implemented draft for review and headed qualification. Mark's
"proceed! next uuup?" continues direct-main delivery. Based on published Mere
`a1acb6b9`, including the current Vello/Genet release adoption and documentation
repair; no previous Forme or study browser receipt qualifies this new editor.

The [portable authoring session](../../../ports/graphshell/src/presentation_editor.rs)
reuses `scenograph::presentation` version 1. It does not introduce a second
resolver or open Rhai adoption. Each rule retains its ID, label, enabled flag,
signed priority, occurrence target, condition and effects. The retained
[Graph tools editor](../../../ports/graphshell/src/web_tree/presentation.rs) is
available through **Presentation rules** on `tree.html?app=local`. **Begin rule
draft** opens private authoring. Add node/link/field rule buttons supply valid
starting rows; the target, condition and effects use the existing model's JSON
vocabulary, with separate ID, label and priority controls. Reordering is
semantic: later authored rules win equal-priority property ties. Disabled rules
still validate. Unknown input properties are refused instead of silently lost;
the shared resolver's compatibility policy remains unchanged.

**Draft interaction:** text controls hold editable buffers. **Update rule
draft**, **Preview rule decisions**, switching rows and **Apply rules** record
the current row before their operation. Malformed input remains undoable but
cannot preview or save. **Undo rule draft** first records pending input, then
undoes one draft step. Undo/redo clears the active row, requiring an explicit
choice before further row edits; it never selects or opens graph content. Redo
refuses unstaged input rather than dropping it. **Discard rule draft** drops
only this editor's buffers and history. Closing the panel retains its draft.
Text caret/IME/local text undo remains Cambium's existing input path; the rule
buttons own whole-set history. No graph-wide keyboard undo is intercepted.

**Decision preview:** node, link and field samples are clearly identified inputs,
editable through the context JSON control. All facts, states, occurrence ID,
screen size and previous matches are disclosed. Resolve shows each winning
property's effect, rule ID and priority, and every rule's match explanation and
winning properties. A changed buffer/context hides the stale result until
Preview runs again. An absent winning property uses the host's base presentation.
Preview is pure and does not carry hidden hysteresis state between occurrences,
samples, committed sets or drafts; prior matches are supplied explicitly.
At this authoring checkpoint the preview has no live graph-to-context adapter
or Canvas consumer. The [exact-occurrence and Forme guide continuation](#exact-occurrence-previews-and-live-forme-guides-draft-2026-10-11)
below adds disclosed bindings and the first supported overlay effect; faces,
containment, provenance and drop zones retain their renderer gates.

**Persistence:** rules, generation and up to 32 saved undo/redo steps commit in
one Muniment workspace record, `graphshell/presentation-rules/v1/<graph-session-uuid>`.
The graph session identifies the workspace scope; graph/source records, selection,
content sessions, theme preferences and Forme layout are absent from snapshots.
The record and malformed draft each have a 1 MiB bound; the resolver's 256-rule
and condition-depth/count limits remain. Every saved history entry, version and
scope is validated before reopening. Apply creates one saved undo step, freezes
rule changes while in flight and changes the committed version only after store
acknowledgment. Failure retains the draft and saved version for retry. Saved
undo/redo requires no open draft and survives reopening.

The prepared save uses `Backend::transact` to compare the exact previously loaded
bytes and replace the record atomically. A stale workspace refuses with a named
explanation, leaving the newer saved record and local draft intact. There is no
racy fallback for a backend without transactions. After a conflict, this first
UI requires reopening/reconciling explicitly; an in-panel merge or draft export
is not implemented. The current IndexedDB transaction adapter snapshots its
whole object store, so storage size and transaction cost remain a backend/host
qualification gate, not a claim of bounded total-store work.

**Qualification and handoff:** the [source receipt](../../../ports/graphshell/docs/receipts/presentation_rules_20261011/qualification.json)
records the actual-source native harness and locked product wasm check (both
pass). Replay the native checks with
`cargo test --locked --manifest-path ports/graphshell/docs/receipts/presentation_rules_20261011/harness/Cargo.toml --lib`.
Nine portable
session tests cover malformed/unknown inputs and disabled-rule validity,
priority/order undo, explicit occurrence-local hysteresis, acknowledged saves,
failure retention, stale-write refusal, graph-scoped MemoryBackend reopen and
saved history validation/limits. Native tests qualify this portable module with
an isolated lockfile, not the complete Graphshell native host or IndexedDB.
The integrated product check uses the web host's current locked dependencies
and committed wasm configuration; default/viewer and release bundles were not
built in this pass.
Browser checks remain pending on O-PC, the ThinkPad or a Mac with a connected
WebGPU-capable browser. The existing connected S-PC browser has no WebGPU adapter.

Use a **fresh dedicated profile/origin** on `tree.html?app=local` for
[`p4_tree_presentation_rules.scn`](../../../ports/graphshell/web/scenarios/p4_tree_presentation_rules.scn),
then a **fresh page/process on the same retained origin/profile** for
[`p4_tree_presentation_rules_reopen.scn`](../../../ports/graphshell/web/scenarios/p4_tree_presentation_rules_reopen.scn).
Use a normal-motion profile (the scripts pause physics explicitly); scenario
waits now include the rule-store write. The first requires an absent rule record; it exercises all three primitive kinds,
preview precedence/reorder/undo, invalid apply refusal, acknowledged apply,
saved undo/redo and discard. The second exercises stored history and decisions
on reopen. Both check graph membership, graph session and Forme access count
remain unchanged. Their two captures are planned, not generated or reviewed here.
Run at normal and narrow widths; review scrolling, long JSON input, caret/IME,
Tab/keyboard activation, disabled controls, source-editor undo independence and
Tabard theme changes. Physical input, fresh-origin negative control, actual
IndexedDB failure/conflict injection, multi-tab races, large-record costs,
release performance and other hosts remain separate acceptance gates.

**Next done-condition at this checkpoint:** bind preview inputs to stable
projected occurrences and an explicit disclosed fact/state adapter; realize
supported effects before measurement/placement, explain unsupported ones, and
qualify scene changes with source, selection, pins, manual placement and live
tile sessions preserved. The bounded continuation below implements these
bindings and Forme guides; its remaining acceptance gates stay open. This
bounded authoring draft does not complete primitive writes, field action/script
execution, theme/font authoring, scene composition or Rhai SE39–SE42 adoption.


#### Exact-occurrence previews and live Forme guides draft (2026-10-11)

**Status:** implemented draft, portable native and product wasm checks passed;
headed acceptance remains open. Mark's continuing “proceed!” authorizes direct
main delivery and review/testing handoff. Published `e56f7a3f` is the integration
base. Fetches before implementation and delivery found no newer upstream
adoption; earlier Forme Chrome receipts remain limited to their recorded source.

The [binding adapter](../../../ports/graphshell/src/presentation_binding.rs)
reads current held owners and receives disclosed state from one Canvas view.
**Use inspected access**, **Use inspected assertion** and **Use inspected field**
explicitly bind a preview. Selection does not retarget it. Missing/departed or
rebound targets produce an explanation and invalidate the previous preview;
there is no equal-address fallback. A held assertion must still contribute to
the exact displayed access pair, even if its Resource owner remains inspectable
elsewhere. Sidecar-only families do not acquire synthetic assertion handles.

Occurrence IDs are `access:<surface-uuid>`, `numen:<field-uuid>`,
`forme:<forme-uuid>` and `assertion:<JSON of canonical access pair and assertion
handle>`. The same Resource and assertion can supply facts to distinct access
or relation occurrences. Local node/edge indexes and URLs are not durable
occurrence IDs. Numen and Forme remain separate owners even with equal UUID
payloads. The read model does not write graph facts or evaluate field ASTs.

**Disclosed inputs:** accesses expose held label/address, optional MIME and its
known image/video/audio/text category, recorded Resource identity/address and
scalar Resource facets under `resource.<key>`. Complex facet values are omitted;
missing metadata is not a completeness claim. Access Selected and PositionPinned
come from this Canvas; Forme holds are not position pins. The optional screen
size uses the measured body hit rectangle's smaller dimension, including the
existing occlusion behavior; no first-frame measurement is invented. Held
assertions expose predicate, actual `Semantic` family, scope, provenance and
source/displayed endpoints; `derived` is not fabricated. Numen exposes name,
lifecycle, definition/extent kinds and this view's boundary visibility. Forme
exposes label, role, lock, visibility, member count and actual boundary hover
when presented. LayoutUnlocked belongs to the Forme owner. Inspected identifies
the retained exact inspector target, independently of graph selection.

The JSON report includes the occurrence, context, available states and limits.
Unknown state differs from known false. Rules that reference unavailable state
or measurement are skipped with a trace explanation, including references
under `Not` or `Any`. Other valid rules can still win. Original whole-set
validation runs before this host filter; malformed or disabled rules are not
silently repaired. ForegroundPinned, Overview, exact assertion selection and
field selection have no adapter yet. Child selection is not promoted to a
selected field. Missing facts retain the shared resolver's existing semantics.
Bound previews use an empty disclosed prior-match set, without hidden hysteresis.
Manual samples/context editing retain their explicit-input behavior; copying a
bound context to manual input stops tracking the owner. Changes to a bound input
invalidate its decision result; Preview must run again. Reads pause while the
editor is closed and refresh when reopened.

**First live effect:** **Enable saved rules in this view** opts into acknowledged
saved rules, including acknowledged saved Undo/Redo. The switch is view-local,
defaults off on a fresh page, and is not stored in graph or rule history. Private
drafts and manual previews never affect the Canvas. Live evaluation reads the
current exact Forme owner independently of the chosen preview target. It caches
resolution by generation and disclosed inputs; the frame installs its effect
before Canvas composition. An unavailable owner restores base guides with an
explanation. `RegionGuides` affects only Forme subdivisions. The [Canvas override](../../../crates/canvas/pictograph/src/canvas/forme_region.rs)
is scoped to the Forme UUID; absent/mismatched overrides use the existing
hover/member-selection/unlocked behavior. A hidden Forme stays hidden and the
outer outline remains. Bounds, cells, locks, selection, explicit pins, manual
placement, source assertions and live tile sessions keep their owners. Whole
graph replacement clears the override. Unsupported winning effects are
disclosed as decisions with no live renderer here; they do not silently invoke
a lower-priority effect or mutate a Numen field.

**Qualification:** the [source-hashed receipt](../../../ports/graphshell/docs/receipts/presentation_live_20261011/qualification.json)
records **31 native tests passed**: seven new binding tests, nine retained
editor/session tests and fifteen actual access/link inspection tests. The small
standalone graph-only harness imports checked-in modules and locks its own
resolved dependencies; it does not qualify the complete native host, Canvas
unit suite or IndexedDB. Replay with
`cargo test --locked --manifest-path ports/graphshell/docs/receipts/presentation_live_20261011/harness/Cargo.toml --lib`.
The current locked product wasm check passes with the same eighteen existing
web warnings. One compiler ran at a time under the 1280 MiB/zero-swap scope and
free-memory guard. Default/viewer/release bundles were not built. A Canvas
regression test is added for show/hide/base restoration, wrong-ID isolation,
hidden boundaries, selection/position/pin preservation and graph replacement;
its native execution remains a handoff gate. Run
`cargo test --locked -p pictograph --features canvas --lib canvas::forme_region::tests`
on a suitable native host before marking that gate passed.

**Browser handoff:** use a fresh dedicated origin/profile and normal motion on
`tree.html?app=local&fixture=field-inspection` for
[`p4_tree_presentation_live.scn`](../../../ports/graphshell/web/scenarios/p4_tree_presentation_live.scn),
then a fresh page/process retaining that origin for
[`p4_tree_presentation_live_reopen.scn`](../../../ports/graphshell/web/scenarios/p4_tree_presentation_live_reopen.scn).
The fixture retains its isolated store. The first checks exact access retention
across selection, private draft isolation, bound Forme/Numen decisions, saved
opt-in, acknowledged Undo/Redo, independent boundary visibility and base
restoration; the second checks stored rules/history and the view switch's
fresh-page default. Both preserve source/session and Forme layout identities.
Four captures are planned, not generated/reviewed. The connected S-PC browser
still lacks WebGPU. Review actual subdivisions, unsupported-effect explanations,
scrolling/narrow width, keyboard/IME, theme changes, physical input, tile session
continuity and pin/manual-placement invariants on a compatible connected
browser. Existing authoring scenarios remain useful regressions; separate fresh
origins are required because they expect an absent rule record.

**Remaining done-conditions:** qualify the renderer and headed cases above;
measure bound-preview hit-test/read costs and long-context presentation before
large-graph adoption; add exact occurrence adapters for sidecar-only and derived
relations; map ForegroundPinned/Overview when their owners exist; add supported
node representation/detail measurement and relation/provenance/drop-preview
consumers before claiming broad effect adoption. Reconcile preview hysteresis
with future live LOD state per view/occurrence. Multi-tab IndexedDB races,
conflict recovery, large-store costs and release performance retain the previous
editor gates. Scene composition, theme/font authoring, general primitive writes,
field actions/scripts, other hosts and Rhai SE39–SE42 remain separate work.

#### Forme drop feedback and field browser qualification (2026-10-11)

Mark authorized the Forme feedback fix followed by Chrome field-inspector
qualification. The Forme session now returns the drop action and original
target bounds together with its candidate. The graph renders a target-local
outline and action label: a half-region for an edge split, or a tab-header
cue for a center stack. The cue is temporary, takes no pointer input and
clears on acceptance, Escape, outside release or draft reset. Zone changes
rebuild the retained cue; motion within a zone keeps pointer capture. The
existing scoped draft, Lock and apply, discard and saved history retain their
authority.

The product Wasm build, viewer-only Wasm check and nine native Forme session
cases pass. All six Chrome scenarios pass: Forme arrangement/reopen,
draft/reopen and field inspection/reopen. Fifteen successful captures were
reviewed. The drag cases verify Escape then release and valid-target to outside
release without changing the draft. Field inspection verifies exact identities,
overlapping candidates, hidden boundaries, retired records, independent Forme
reads and retained IndexedDB reopen. Inspector label/value spacing and focus
or scroll follow-through remain open; functional acceptance does not close
those visual review gates. The browser source is `eb804d07`, with the scenario
key correction at `eac1524f`; subsequent integrations do not inherit this pass.
An initial unsunk launch was closed without a qualification claim. A collected
drag run found lowercase `key escape` unsupported by the scenario runner;
the scenario now uses its existing `key Escape` route. That failed run is
retained alongside the corrected sequence in the
[qualification record](../../../ports/graphshell/docs/receipts/forme_feedback_field_browser_20261011/qualification.json).

The integration at `9d8ea699` retains upstream presentation controls and live
Forme guides. Its product and viewer-only Wasm checks pass separately; Chrome
was not rerun on that newer source. The canonical primary documentation audit
has no index or current-link findings. Its remaining upstream findings are
recorded in the qualification summary. The temporary browser package is removed;
peer documentation work is preserved.

#### Inspector, presentation and exact strength qualification (2026-10-11)

The next authorized slices add shared fact-row spacing to the local inspector
stylesheet and one-shot focus/reveal for an explicitly chosen field or coupling.
Those detail headings participate in the retained keyboard order. Requests are
processed after dispatch and in the frame hook, so keyboard/programmatic
activation receives the same next-layout reveal. Discovery and ordinary graph
selection do not retarget inspection. A page-local `?motion=normal|reduced`
override follows an explicit root motion attribute and precedes the browser
preference. Motion preference and the analytic arrangement's paused physics
are separate; the tracked qualification checks both preconditions explicitly.

Ten Chrome scenarios pass, with eighteen final captures reviewed: field
inspection/reopen and 420px keyboard reading; declarative rule authoring/reopen;
exact-occurrence live Forme guides/reopen; and exact strength draft/save/reopen
plus a final visible-value/draft capture. The rule cases establish invalid-draft
refusal, decision traces, acknowledged saved undo/redo, opt-in guide adoption,
independent boundary visibility and restored base guides. Unsupported Numen
effects are disclosed rather than applied. The source-hashed
[qualification](../../../ports/graphshell/docs/receipts/field_editor_presentation_browser_20261011/qualification.json)
binds each page to its actual package/source cut. Fresh pages retain their own
origin's IndexedDB; fresh native/browser processes are not inferred.

Five native Forme renderer tests pass, including guide/base restoration,
wrong-identity isolation and pin/placement preservation. Five exact-coupling
tests pass, including failed/nontransactional storage, competing saves,
equal-ID foreign graph refusal and scoped history with unrelated-edit
preservation. The final product build and viewer-only check pass. Older portable
model and full-host receipts are not new reruns. Functional plus capture
acceptance does not qualify IME, physical touch/focus, other browsers, release
performance, theme/font changes or large-context costs. Rule-editor JSON input
and missing symbol-font fallback remain visual follow-through; the new strength
preview uses plain words for its old/new values.

The [field owner](2026-06-13_scriptable_field_regions_plan.md#field-inspection-adapter-sequence-2026-10-10)
records the graph-session-bound strength draft and staged graph/journal
transaction. Its acknowledged update refreshes the portable local client and
the exact Canvas coupling, preserving geometry and Forme identity. It opens
strength only; selector/response/definition editing, primitive associations and
entry/exit scripts keep their gates. The composition walkthrough separates
arrangement, additive influence and presentation scope without ruling the
remaining spatial or incompatible-placement policies.

Failed parser/precondition runs and the earlier narrow capture that focused
without revealing are retained. The corrected narrow capture shows the actual
exact-coupling details, not just a successful model assertion. Generated browser
packages and owned processes are removed after recording; the shared Mere Cargo
target is reused and concurrent documentation work is preserved.
#### Relationship and dynamics inspector draft (2026-10-11)

**Status:** implemented bounded draft; portable source checks and integrated
product wasm check passed. Mark chose the relationship/dynamics inspector next
and said “Sure. Go ahead”. Delivery continues directly to main. Integration
base was published `ec1e65ca`, with the later renderer/Knot viewport receipts
through `578c8e36` integrated before delivery. Runtime sources and dependencies
in this check cone are unchanged by those documentation/test-receipt commits.
Upstream Forme target-local feedback, Chrome
field inspection/reopen receipts and the shared occurrence-relationship dynamics
adapter are preserved. Those receipts retain their own source/platform limits
and do not qualify this new UI. The dynamics owner's production Knot consumer
review remains its own adoption and qualification record.

**Findings:** Graphshell's local Canvas can disclose the authored dynamics
record, its current running stage, controller topology inputs and placement
constraints. It has no measured per-assertion force vectors or installed
coupling-target snapshot readout. The projection relationship controller is
available to compiled-scene consumers; it is not implicitly installed in the
local graph view. Held `AgentDerived` assertions still have the existing exact
inspector's explicit missing-rule/input explanation. This draft does not invent
derived display marks, execution evidence or a new force law.

The [Canvas report](../../../crates/canvas/pictograph/src/canvas/dynamics_inspection.rs)
reads an exact access UUID or displayed pair without stepping the controller.
It separates the current record (including an authored schedule retained after
its run) from the current stage, carrying errors for undisclosed channel/partition cases.
The record is Canvas readback, not a storage acknowledgment or source-history
snapshot. The law label is a flat picker summary; the full stage remains the explanation
for mixtures/groupings. Overlays, pause state, arrangement and topology source
are disclosed. Inspection does not change source, selection, velocity, pose,
role, pin, Forme lock/geometry, tile sessions or controller lifetime.

The [shared topology seam](../../../crates/canvas/pictograph/src/canvas/relationship_topology.rs)
now supplies both the existing controller bridge and its inspector. It retains
every projected relation row, hidden-cell filtering and parallel multiplicity.
The selector conversion also remains shared with edge painting/picking. A pair
report groups rows by displayed direction, source stratum/endpoints and actual
relation selector; exact assertion handles are deduplicated within that group.
Grouping is explanatory, not a new durable source handle. Surface and Resource
owners, repeated accesses, both directions, parallel buckets, open predicates
and self-links remain distinct where the current input seam distinguishes them.
Missing accesses refuse; equal addresses do not retarget inspection.

A native probe corrected an assumed one-row-per-subkind expectation: current
`relation_rows` in the kernel emits each semantic statement as a separate row,
including two assertions with the same predicate. The report preserves that
existing behavior. Input-row count and assertion count are shown with their
own meaning; neither is a measured force count or strength. Hiding removes the
same row slots the controller consumes. A disclosed occurrence controller
reports its own input-edge count and does not claim a kernel-cell mapping.
No topology weighting or assertion-to-force policy was changed by this lane.

**Placement disclosure:** each target reports its role policy and item/group
sources, explicit position pin, installed arrangement pin, arrangement-slot
availability, configured live anchor target/stiffness, axis locks, Forme owner
and current hold, and pointer drag. Axis-lock policy is read through the same
helper used to install it. A Forme hold remains independent of boundary
visibility; an explicit pin takes priority. Role policy without a slot is not
claimed as an installed arrangement pin. An anchor target is configuration,
not a sampled contribution; Forme or other constraints can govern its body.
Attention/foreground pins are not mapped onto these position constraints.

**Interaction:** [Dynamics inspector](../../../ports/graphshell/src/web_tree/dynamics_inspection.rs)
lives in Graph tools. **Explain selected dynamics** binds one selected access;
**Explain this access dynamics** binds the exact retained access inspector target;
**Explain pair dynamics** binds the exact link inspector pair. Subsequent
selection or source-inspector closure does not retarget or close it. Changes
in current view inputs refresh its report; **Close dynamics inspection** clears
only this transient inspector. Pair groups provide **Inspect dynamics input
assertion** buttons to explicitly open the existing exact source owner, preserving
the dynamics target and selection. Derivation metadata remains sourced from that
owner, with absence shown. There are no inspector write controls or persisted
inspector snapshots. JSON stage/record blocks wrap in the existing themed pane.

**Qualification:** the [source receipt](../../../ports/graphshell/docs/receipts/relationship_dynamics_20261011/qualification.json)
records **38 portable native checks passed**: seven new topology tests and the
31 retained binding/editor/access/link tests. The locked graph-only harness
imports the actual shared helper and checked-in model sources; it does not
compile the complete native Canvas or exercise IndexedDB. Replay with
`cargo test --locked --offline --manifest-path ports/graphshell/docs/receipts/relationship_dynamics_20261011/harness/Cargo.toml --lib`.
The locked offline product wasm check passes with the same eighteen existing
web warnings. One compiler ran at a time under the established 1280 MiB,
zero-swap and free-memory guard. Canvas report regressions are added for
read-only graph/selection/pose/pause, Forme-versus-explicit-pin behavior and
missing exact members; their native execution remains open. On a suitable host,
run `cargo test --locked -p pictograph --features canvas --lib canvas::dynamics_inspection`.
Default/viewer and release bundles were not built in this pass.

**Headed handoff:** use a fresh dedicated origin/profile with normal motion on
`tree.html?app=local` for
[`p4_tree_dynamics_inspection.scn`](../../../ports/graphshell/web/scenarios/p4_tree_dynamics_inspection.scn),
then a fresh page/process retaining that origin for
[`p4_tree_dynamics_inspection_reopen.scn`](../../../ports/graphshell/web/scenarios/p4_tree_dynamics_inspection_reopen.scn).
They exercise independent exact targets, live pause changes, exact assertion
routing, Forme holds despite hidden boundaries, source-inspector closure and
reopen without persisted inspector state. Four captures are planned, none
produced/reviewed here. S-PC's connected browser still lacks WebGPU. Review
labels/values, long JSON, keyboard/focus, scrolling and narrow widths; qualify
explicit pin/manual placement, hidden relation controls, stage transitions,
live tile continuity and applicable native/Firefox hosts separately.

**Remaining done-conditions (after the Windows continuation below):** measure
open-inspector graph traversal/grouping and long-record costs
before large-graph adoption; add an execution-owner adapter for actual derived
relationships and compiled-scene occurrence mapping; add measured per-term or
per-assertion contributions only through their dynamics owner. Installed field
coupling snapshots, foreground-pin/overview adapters and full node presentation
remain separate. No script/field execution, primitive writes, broad scene
composition or force-policy adoption is completed by this read-only draft.

#### Relationship inspection integration and native checks (2026-10-11)

Merged upstream `6a196227` into local main as `208e134e`, preserving the exact
coupling-strength editor and both inspector styles. Existing uncommitted
documentation was restored, reconciled and subsequently recorded by its owner.
The three actual Canvas report tests now pass on Windows. Eight actual Canvas
topology tests pass, including a new mixed-kind Resource self-link projected
between repeated accesses in both directions, with 67 retained input rows.

Assertion readback now uses the typed relation handle directly and visits each
bucket/kind/displayed direction once. It still counts every projected row and
deduplicates only assertion handles. This removes repeated bucket searches and
same-kind statement scans; full-graph projection and long-record refresh costs
remain unmeasured. The original seven topology cases preserve hidden cells,
parallel buckets, source strata, exact targets and open predicates.

The combined product browser host builds with locked offline dependencies.
The root native gate initially required an offline lock reconciliation for four
already-existing Dramatis dependencies; it upgraded no packages. That lock and
the concurrent Dramatis source changes remain with their owner. Native tests
reuse `C:\t\cargo-targets\mere`; no isolated build tree or Cargo home is created.
The [Windows receipt](../../../ports/graphshell/docs/receipts/relationship_dynamics_20261011/windows/qualification.json)
records source hashes and separate native/browser outcomes.

Both incoming scenario scripts now explicitly open Graph tools before revealing
the inspector controls and assert the seeded arrangement's paused state. The
first script closes the tools overlay and centres its pointer target before
selecting another access; it asserts the acknowledged address before creating
the Forme. The earlier occluded click retained Reference notes instead of I2P
port notes, which the reopen assertion correctly rejected. Capture reveals use
the existing detail classes rather than an absent DOM heading-role attribute.
Earlier incomplete and failed attempts are retained. The strengthened fresh-origin
script passes with I2P port notes actually retained. Its fresh-page retained-origin
case also passes: the inspector begins closed, binds that exact access and reports
its restored Forme owner/hold without changing source or Forme geometry. Chrome
154 on Windows, WebGPU/Nvidia, 615 by 664 logical pixels at scale 2 supplies five
reviewed captures. Pair source strata/handles and reopened access identity,
placement constraints and Forme ownership are readable in the scrolled views.
The Forme graph capture records composition, not every inspector value in one
frame. The final placement capture reveals past the hold so its value is visible.
The root graph is cropped by the narrow two-pane layout; full graph visibility,
keyboard/focus at 420px, long-record costs, hidden-relation controls, stage
transitions, live tile continuity, native headed and Firefox remain separate.
No force vector, contribution attribution or missing derivation owner is invented.

#### Field editor delivery integration (2026-10-11)

The field editor lane integrated remote Moot work `f197cadf` and the newer
relationship-inspection source `2fc4024a` at `be6c09a1`. Locked offline product
and viewer-only Wasm checks both pass from `ports/graphshell/web`, using the
shared stable Cargo target. These compile checks are recorded separately in the
[field editor receipt](../../../ports/graphshell/docs/receipts/field_editor_presentation_browser_20261011/qualification.json).
Its ten passing Chrome cases and eighteen reviewed captures remain bound to
their named package cuts; the newer integrated source does not inherit those
browser results. The temporary integration checkout preserves concurrent
Dramatis packaging work in the primary checkout and is removed after delivery.


#### Node ladder and inspector continuation draft (2026-10-11)

**Status:** implemented bounded draft; portable native and product wasm checks
passed; headed acceptance remains open. Mark requested “Both. Plus pull”,
continuing relationship/dynamics inspection and the node presentation ladder.
An explicit `git pull --ff-only origin main` reported the existing `04a48c2d`
checkout up to date. During delivery, source and qualification work through
`3162f302` was integrated, including exact coupling edits, browser/focus receipts
and bounded assertion readback. The two append-only document conflicts were
reconciled while preserving both lanes. Final Knot completion/refusal
documentation `e6192a18` is also retained; runtime checks remain applicable.
Delivery continues directly to main. No prior Forme,
renderer or Knot receipt qualifies this new face/attribute consumer.

The [node rule runtime](../../../ports/graphshell/src/node_presentation.rs)
reuses the saved version-1 presentation rules and their disclosed occurrence
contexts. **Add node ladder defaults** appends four editable rules to the
private draft: ordinary Canvas face, compact Glyph at a 40/56 logical-pixel
hysteresis band, selected ordinary face plus attributes, and the positional-pin
face guard. Priorities are 0/10/20/30 respectively. Users can edit targets,
conditions, values, thresholds and priorities before applying. The existing
acknowledged Apply/Undo/Redo, stale-write refusal, bounds and private draft
history remain the persistence contract. Adding defaults does not apply them.

**Supported presentation:** saved-rule view opt-in now also drives node
`Representation::Glyph` and `Card` through an independent
[Canvas face layer](../../../crates/canvas/pictograph/src/canvas/node_presentation.rs).
Glyph uses the existing terse icon/caption treatment; Card is the existing
ordinary Canvas face, not a newly embedded content summary. The layer is keyed
by exact access UUID, validates the current local view epoch atomically and
leaves the analytic projection score intact. Whole graph replacement clears
faces and increments that view epoch. Disabling rules, undoing to an empty set,
leaving a scope lens or refusing a context returns to the base score/Canvas
presentation. It never substitutes a shared Resource identity or URL.

Face changes retain the existing resolved body dimensions, geometry, collision
and placement owners. They do not restart dynamics, modify pins, change Forme
holds/layout or replace tile/content sessions. Full content-card measurement,
media/snapshot textures and interactive panes need their own renderer adapters.
A winning Snapshot/Sprite/LivePane/open request is disclosed as unsupported by
this rule consumer; no lower-priority representation is silently substituted.
Other supported properties can still win independently. Existing legacy
projection-score face treatments remain the base behavior.

**Attributes and explanation:** `Details` controls a separate current-selection
attribute lane in Graph tools. It shows held scalar facts for selected accesses,
including label/address, MIME and recorded Resource metadata; complex values and
tags remain in the exact source inspector, reachable explicitly. It does not
retarget or hide that inspector. [Retained node controls](../../../ports/graphshell/src/web_tree/node_presentation.rs)
report supported-face counts, refusal reasons and unsupported requests.
The dynamics inspector now reports each exact target's face treatment, rule
face override and the body-size input, and can explicitly bind that access in
Presentation rules. Selection, motion inspection and rule preview remain
independent. Neither these controls nor rule matching opens content implicitly.

**Stable input and hysteresis:** the Canvas exposes its resolved logical body
footprint multiplied by camera zoom before choosing the face treatment. This
is the same width used by its frame composer, rather than the prior preview's
occlusion-sensitive body-hit rectangle. Caption hiding, selection or partial
occlusion therefore cannot feed back as a rung-size change. Both bound preview
and live evaluation disclose this footprint convention. Pure previews still
use explicit/empty prior matches; live matches belong to one view epoch, saved
rule generation and exact access occurrence. Graph replacement, saved-rule
changes, disable/refusal or departure from the current lens reset/prune that
history. Caller-supplied prior matches are not imported into live state. Equal
addresses and shared Resources do not share hysteresis.

The access fact adapter now reads the recorded binding and scalar Resource
facts directly; constructing a sibling-access roster/coverage inspection for
every live node was unnecessary and would repeat whole-graph work. Inspector
read models and their coverage semantics remain unchanged. Unknown states and
missing measurements retain the explicit refusal behavior, including under
negation; foreground attention pins are not inferred from positional pins.

**Bounds:** this first consumer requires at most 128 individually visible
accesses and 256 KiB of serialized contexts. Exceeding either refuses the node
layer with an explanation and restores base faces. It does not partially apply
a subset. Forme guide consumption remains independent. The runtime reuses
unchanged inputs/rules and stores only current occurrence matches/decisions;
scene, source and content sessions are absent from these transient snapshots.
Open-context collection and large rule-set costs still need performance
measurement. No release or large-graph frame-budget claim is made.

**Qualification:** the [source receipt](../../../ports/graphshell/docs/receipts/node_ladder_20261011/qualification.json)
records 47 portable native checks passing: eight new node-runtime tests plus
the retained 39 binding/editor/topology/inspection tests. Cases include repeated
Resource occurrences, independent hysteresis, selection versus position pins,
view/generation reset, departure/reentry, unsupported winners, unknown negated
state, immutable inputs and atomic bounds/identity/rule refusal. Replay with
`cargo test --locked --offline --manifest-path ports/graphshell/docs/receipts/node_ladder_20261011/harness/Cargo.toml --lib`.
The integrated product wasm check passes with eighteen existing web warnings.
One compiler ran at a time under the existing 1280 MiB/zero-swap and free-memory
guard. Default/viewer/release packages were not built.

Native Canvas node regressions additionally cover source/pose/selection/pin/
footprint preservation, atomic unsupported/missing-member refusal and old-epoch
rejection. The existing dynamics-inspector cases are included in the native
qualification attempt. Its first offline run could not load an uncached
upstream P2P patch; fetching locked sources left manifests and lockfiles intact.
The full-debug native compile then reached the scope's 1342177280-byte peak
and was terminated with `Result=oom-kill` before tests ran. The retry disables
Pictograph debug symbols while retaining that memory cap. Three inspector and
three new face-layer tests pass; the latest integrated-source reruns are recorded
in the receipt. This is six scoped native cases, not an all-green Canvas suite.

**Headed handoff:** on a fresh dedicated normal-motion origin/profile and a
logical 860×720 viewport, run
[`p4_tree_node_ladder.scn`](../../../ports/graphshell/web/scenarios/p4_tree_node_ladder.scn)
on `tree.html?app=local`, followed by a fresh page/process retaining that origin
for [`p4_tree_node_ladder_reopen.scn`](../../../ports/graphshell/web/scenarios/p4_tree_node_ladder_reopen.scn).
The fixture deliberately uses large compact thresholds to avoid dependence on
viewport-fit zoom; the scripts explicitly open the collapsed Graph tools pane.
Native cases qualify the ordinary hysteresis band separately.
Four captures are planned, none generated/reviewed here. Scenarios cover private
draft isolation, saved opt-in, selected attributes, independent dynamics targets,
unsupported Snapshot precedence, acknowledged saved undo, disable/base restore
and retained rules with a fresh transient view. They preserve graph/session and
Forme membership/geometry/locks. Review actual glyph/caption/attribute rendering,
normal zoom thresholds, pinned nodes, physical input, themes/fonts, narrow
scroll/focus, tile continuity and other hosts separately. S-PC's connected
browser still lacks WebGPU.

**Remaining done-conditions:** complete headed acceptance;
measure live-context/rule costs; add actual compact content cards, snapshots,
sprites and interactive representations through their content/measurement
owners; extend exact occurrence adapters, attention states and measured force/
derivation evidence through their existing owners. This draft completes only
the bounded face/selected-attribute consumers and inspector integration. It
introduces no force policy, source/script/field execution, primitive writes or
broad scene-composition completion.
