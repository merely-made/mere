# Scenograph Editor Plan

**Date:** 2026-10-07
**Status (2026-10-07):** plan. Rulings SE1 to SE9 recorded (§1). Track E1, the generic history, is next; nothing is built yet.
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

## 2. Tracks

E1 and E2 carry ruling A; E3 to E5 carry ruling B. E2 needs E1; E4 needs E3; E5 needs E2 and E4.

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
