# Scenograph Editor Plan

**Date:** 2026-10-07
**Status (2026-10-07):** in progress. Rulings SE1 to SE9 recorded (§1). E1, the generic history, landed on main 2026-10-07 (`de06e4f0`) and moves into the `edit-history` leaf crate (SE10, SE13). E2, undo in the projection editor, landed 2026-10-07 with its headed check. E2b, saving into the mere session (SE11, SE12, SE14 to SE22), landed 2026-10-07 with headed checks in Chrome and Firefox; Safari is open. E3 is next. Mark asked for drag-to-pan on the canvas as a separate objective.
**Scope:** the editor foundation that Scenograph's editing surfaces stand on: one undo history in Cambium, and arrangement options declared as data. Carries out the balaur review's rulings A and B. The projection editor in Graphshell is the first consumer of both.

Not in scope, mapped in §3 and opened by later rounds:
- the scripting comparison (SE3), which is the next assessment;
- wallpapers and props, the node, edge and field style editors, and authored motion.

**Related:**
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

**SE12, the save path (2026-10-07).** Question: the editor's web sink saves definitions to browser localStorage, outside any mere (`ports/graphshell/src/web_main.rs`, `BrowserProjectionSink`), against the 2026-09-23 ambiance ruling that apps keep their session data in a mere. Options: yes, as track E2b; yes, before E2; not now, recorded. Mark: **"Yes, as track E2b (Recommended)"**. *Follows:* track E2b.

**SE13, the crate's name (2026-10-07).** Question: the leaf crate's name; `edit-history`, `undo-history` and `cambium-history` were each checked free on crates.io. Mark: **"edit-history (Recommended)"**. *Follows:* `crates/cambium/edit-history`, `publish = false` until a publish is decided, so the name is chosen but not claimed.

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

## 2. Tracks

E1 and E2 carry ruling A; E3 to E5 carry ruling B. E2 needs E1; E4 needs E3; E5 needs E2 and E4. *Added 2026-10-07:* E2b carries SE11 and SE12, after E2.

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

*Annotation 2026-10-07 (SE10):* E1's `History` lives in `crates/cambium/edit-history`; the cambium crate depends on it and re-exports `History`, and `EditHistory` stays in `crates/cambium/cambium/src/editor.rs`.

## 3. Mapped, not opened

Each comes back as its own round of questions.
- **Scripting (SE3).** Compare every candidate, Rune included, through `script-engine-api` and embedded. F6 is the starting evidence.
- **Wallpapers and props.** The backdrop table (L2) is landed and consumed by Isometry, Woodshed and Graphshell. Seiche's living backdrop and scene bodies have no Mere or Turnstone host. The games wing's `MapDocument` carries a `props` layer (§4, F7).
- **Style editors.** Node: the body and face plan's B3 body designer is open. Edge: forme's `EdgeProjectionSpec`. Field: the scriptable field regions plan, where no host calls `add_field_at`. Appearance today is three strings (§4, F3).
- **Motion.** Balaur rulings C and D, gated on L5's authored front-end.

## 4. Findings

Verified 2026-10-07 against mere's origin unless named.

- **F1, the editor.** `ProjectionEditor` holds a draft, a panel and a `Workbench`, and reduces ten actions: `SelectPanel` and nine whole-field setters (`projection_editor.rs`, `EditorAction` and `reduce`). Its one effect is `save` through a `ProjectionDefinitionSink`. It has no undo. Graphshell's web app is its only host (`web_main.rs`, `web_projection.rs`).
- **F2, Cambium's history.** `EditHistory` (`crates/cambium/cambium/src/editor.rs`) is text-only: snapshots of text, caret and selection, a cap of 200, insert-run coalescing, no clock and no saved marker. Its users are `TextInput` (`controls/text_input/core.rs`, `command.rs`) and the crate's re-export. No crate in mere, Turnstone, Knot, Woodshed, Isometry, Genet or mer3ly names it. Cambium has no clock type. A second undo exists for graph truth, attributed and journal-based (`pandect::graph_session` `undo`, Graphshell's `mere_host`); the editor's draft history is a separate, local plane.
- **F3, the authored surface.** `Appearance` is `realization`, `title` and `theme`; `Encoding` is x, y, color and label; `Arrangement.options` is an open `BTreeMap<String, String>` (`crates/cambium/scenes/scenograph/src/lib.rs`).
- **F4, options.** Eleven families (`Family`, `FAMILIES` in `scenomise/src/catalog.rs`) read 26 distinct keys through seven readers (counted by literal call; E4's test finds any other): `finite`, `positive`, `count`, `depth`, `flag`, `choice`, `list`. `finish` refuses any key not read. A custom solver receives the options as a JSON object and judges them when it solves (`scenomise/src/projection.rs`). `SolverCapability` (`registry.rs`) carries no options. The only `Solver` implementations in mere, Isometry, Turnstone, Woodshed, Knot and mer3ly are test fixtures.
- **F5, the games wing.** Answering read-only (Isometric game engine architecture session), the wing took balaur's determinism recipe and labelled digest (wing rulings 603 to 611, 633 to 636, in `isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`) and none of the editor slices. Ruling 609 builds one labelled-digest crate in mere under G8.
- **F6, scripting.** Genet's `script-engine-api` (`components/script-engine-api/lib.rs`, 975 lines) is a JavaScript-shaped contract: realms, `WindowProxy` traps, reflectors carrying a DOM `NodeId`, host promises. Its backends are Nova, Boa and piccolo; piccolo's documents its deviations and does not yet honour step budgets. Outside it: mere's rhai crate, behind inker's `BlockEvaluator` and "deliberately *not* genet's full DOM-shaped `ScriptEngine` seam" (`crates/script/rhai/src/lib.rs`); mere's Wasmtime hosts on the `mere:script` WIT world (`crates/script/wit/world.wit`); and Isometry's piccolo, which `isometry-system` and `mesocosm-phenotype` depend on directly. No manifest in mere, Isometry, Turnstone, Woodshed or Knot names Rune. *Reading, not ruled:* values cross the seam as strings and node handles only, so scene logic may want a value-level seam beside it.
- **F7, the wing's stagecraft.** Isometry's `MapDocument` (`crates/isometry-core`) carries tile layers including `props`; appearance binds through CSS classes with tilesets as stylesheets (`crates/isometry-views`); the shared scene is `shared/isometer`.

## Progress

- **2026-10-07.** Plan written from the chat assessment; rulings SE1 to SE9 recorded. The balaur brief and the projection grammar handoff carry dated pointers here.
- **2026-10-07, E1 built on `sceno-editor-e1`, not merged.** `cambium::History<S, K>` in `crates/cambium/cambium/src/editor.rs`: key-and-window coalescing on a host-supplied millisecond time, a saved marker that shifts with the cap and is lost when its state falls off the cap or sat on a discarded redo stack. `EditHistory` is a newtype over `History<TextSnapshot>`, since `TextSnapshot` is crate-private; its API is unchanged and `TextInput` calls it as before. Eight new tests; `cargo test -p cambium --lib` passes, 257 tests. Two controls failed where they should: dropping the redo-discard reset, and dropping the cap shift, each fails one test. A first version of the redo-discard test passed under its control, because it never returned to the saved depth; it was fixed before the control was rerun. Found in passing: Cambium is not `cargo fmt`-clean at origin (`atlas.rs`, `graph_canvas.rs`, `lib.rs`, `workspace.rs`); `editor.rs` is.
- **2026-10-07, E1 landed.** Mark: **"Merge E1, then E2 (Recommended)"** (options: that; E2 on the same branch first; stop). Rebased on origin with no change, `cargo test -p cambium --lib` rerun (257 pass) and `cargo check -p graphshell` clean, then fast-forwarded to main at `de06e4f0`.
- **2026-10-07, SE10 to SE13; `edit-history` founded.** E2's first step found Graphshell's native build has no Cambium (46 crates to add), and Mark ruled a leaf crate (SE10), two layers over Eidetic (SE11), saves into the mere as E2b (SE12), and the name (SE13). `History` and its eight tests moved to `crates/cambium/edit-history`; cambium re-exports it. `cargo test -p edit-history -p cambium --lib`: 8 and 249 pass, the same 257 as before. Note: cambium is `publish = true` and now depends on an unpublished crate, so publishing cambium needs `edit-history` published first.
- **2026-10-07, E2 landed.** `ProjectionEditor` keeps an `edit_history::History` of (draft, panel), keyed by the action's variant, with a 400 ms window (`COALESCE_WINDOW_MS`) on the host's clock (`reduce` now takes `now_ms`; the web host passes `js_sys::Date::now()`). An edit that leaves the draft unchanged records nothing; `save` marks clean, and the executable path's own save calls `mark_saved`. The web host gains `undo-projection` and `redo-projection` commands, Undo and Redo buttons disabled when there is nothing to step, and an unsaved-changes line. Five new tests; `cargo test -p graphshell --lib projection_editor` passes 15. Control: with the panel return removed, one fails. The `graphshell-web` wasm check passes. Headed: `projection_undo.scn` in Chrome (tab visible) passed 36 steps over 147 frames; capture and result in `Code/testing/mere/scenograph-editor/`. Its first run failed one assertion of the scenario's own: a click on the disabled Undo button does nothing, so the scenario now asserts the button disabled and runs the command. The new line first rendered at body size and now shares the status lines' style. Findings for later: two fields that send the same action (`source.authority` and `source.domain` both send `SetSource`) coalesce if edited within the window, *reading, not ruled*, acceptable; the executable path (`save_live_projection`) and the sink both write localStorage, which E2b replaces; `wasm-bindgen-cli` on this machine is now 0.2.129 (Mark: "Replace the global CLI"), matching the lock.
- **2026-10-07, E2b landed.** pandect: `undo_now`, `redo_now` (a revert journaled, not stored; the async `undo` and `redo` now call them, unchanged in behaviour) and `has_unstored`. MereHost: `prepare_store`, `store`, public `staged`, `undo_now`, `redo_now`, `has_unstored`; `Staged` is public with `write` and `is_empty`. Graphshell: `save_projection` and `saved_projection` (one change through `graphshell.projection-editor`, node at `mere://projection/<id>`, facet `graphshell.projection-definition/v1`, value scenomise's `ProjectionSnapshot`), and `kept_summary`. The web host: a `web_session` module whose frame-pump step writes pending changes through a clone of the store and wakes the pump with `graphshell-wake`; a `data-session-store` token (`stored`, `pending`, `writing`, `failed`, and `unstored` for changes nobody asked to store) with `data-session-store-error`; both editor savers write the session and localStorage is gone from the editor; Undo save and Redo save in the editor; Undo change and Redo change pills in the top bar, drawn in the canvas chrome with hit targets beside Local and Remote. Tests: pandect 304 pass (with `has_unstored` checks); Graphshell's library with `web` 261 pass; three new product tests (reopen from the store through the split write, Undo save reaching only its channel and not Graphshell's change, kept parts named); control: saving through `graphshell` fails two of them. Headed, Chrome: `projection_session.scn` 45 steps ok, `projection_session_reopen.scn` on a fresh page ok, control on another origin with nothing saved fails both assertions, `projection_authoring.scn` 59 ok and `projection_reopen.scn` ok through the session, a real mouse click on the drawn Undo change pill undoes and Redo change redoes. Headed, Firefox 157: `projection_session.scn` 45 steps ok and the fresh-page reopen ok, read through a receipt sink (`Code/testing/mere/scenograph-editor/sink_server.py`, receipts beside it). Safari is not run. Findings: confirmed at runtime that a browser scene save reports "Scene saved" and is gone after a reload (`unknown address mere://scene/graphshell-h3`), because nothing stores it; per SE16 this is next door, now visible as `data-session-store="unstored"`, and a later editor save stores it along with its own. The page's `data-*` tokens live on `<graphshell-view>`, not `<body>`. Existing editor definitions in localStorage are not migrated.
