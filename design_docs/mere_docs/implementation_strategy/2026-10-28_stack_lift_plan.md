# Stack lift: Graphshell component extraction

**Status:** 2026-10-29: status corrected; cut on 2026-10-28 from `2026-10-14_graphshell_stack_lift_plan.md`, archived in `archive_docs/2026-10-28/`. R1 to R8 and the `Page` split are ruled. The `door` feature (P1), the serving half `graphshell-host` (P4), the Djinn broker move (P5), the option helpers (P3, Mere side), the rule-authoring and presentation controls (P7, Scenograph and Mere View) and the component and source-ownership slice are landed and published, and the five receiving repositories are cut over. P6's Knot second-host adoption is committed locally and qualified on 2026-10-29; it has not been pushed. Open: the dependency-cone baselines (P0), the remote overmap's Turnstone receipt (P2a), `personal_sync` and the facet brief (P8), the remaining `web_tree` views and owner completions (P9), Woodshed's dataset publication (W1, W3), the canvas lifts that wait on a second host, and the browser, Woodshed-runtime and physical-device qualification the slice did not claim.

**For:** extracting from the Graphshell port (about 100k lines) the parts other applications want without taking the whole port. It began when Mark noticed the reference host's size and asked what could be generalised; Knot, Cleromancy and Woodshed want its interactive capabilities, and Woodshed's own survey asks whether Mere extracts the canvas controls and editors into native-buildable crates with Woodshed as second native consumer. Each phase names a home, a prerequisite and a second-host receipt; a phase that lands code ends with that receipt, and a receipt in this repository's tests does not stand in for the other repository's build.
Not here: the Graphshell one-tree cutover and the reference host's own phases, `2026-10-28_graphshell_host_plan.md`; the facet dissolution brief's ruling and the resident-host port that replaces `product.rs` operations with Pandect `LiveCommand` values, `2026-10-28_graph_semantics_plan.md` (the brief's owner); the Eidetic development plan's P0 inventory, which takes `capture` (Eidetic repository); SE13, SE24, SE46 to SE51 and SE78 (gestures, command-menu verb ids), `2026-10-28_scenograph_editor_plan.md`; `appearance`, which stays in Graphshell (R6) with Tabard's division in `2026-10-28_appearance_plan.md`; custody isolation, `2026-10-28_custody_plan.md`; Woodshed's manifest repins, the Woodshed lane.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **R1:** whole-definition authoring state lives in `scenomise` behind an `authoring` feature (it cannot go in `scenograph`: a cycle). Its first adopter, Knot's recipe editor or Turnstone, is the receipt. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:370-381`.
- **R2:** Forme workspace and session live in `platen` behind a feature, after the three canvas-touching pieces are cut; `crates/forme/forme` is excluded by a cycle. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:382-386`.
- **R3 and R4:** the app-facing half is a `door` feature on `graphshell-endpoint`, no new crate; the serving half is the stack crate `graphshell-host` under `crates/graphshell/`; Djinn keeps only the brokers. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:387-402`.
- **R5:** `personal_sync` moves and adapts when the facet dissolution brief rules; L13 no longer waits on the brief. **R6:** `appearance` stays. **R7:** the remote overmap crate is `mere-overmap` under `crates/cambium/` beside `mere-view`. **R8:** Castellan is a component crate under Djinn, with no screens. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:403-413,414-429`.
- **The `Page` split:** each `web_tree` view becomes a Cambium component with narrow typed inputs and moves with the lift that owns its state (L5, L12, L11); no separate sub-state pass first. A control is deleted from the port in the same change that lifts it; a second host's adoption is evidence of reuse, never a reason to keep a port facade. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:430-434,1668-1670`.
- **Rulings after P5 (2026-10-15):** `view` and `sessions` lift into `graphshell-host`; the browser carrier and the identity endpoint pair move into Djinn; L13's replica, transfer and receipt types wait for the facet brief; `resident_mere_proxy` moves into `graphshell-endpoint`'s `door`; the C4 fixture returns to the port. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:1296-1303`.
- **Group 4 product truth is not lifted.** `mere_host`, `product`, `local_edit`, `local_intake`, `access`, `transfer` and `capture` stay in the port; only `access_inspection` and `link_inspection` (readers over `mere::kernel`) move first if L11 proceeds. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:329-347`.
- **Woodshed's constraints.** Consumers get client-side holders only (Woodshed never constructs `GraphshellApp`, `MereHost` or `MereSessions`); lifted model and reducer crates stay wasm-clean with native-only view pieces behind a feature; Woodshed, Hocket and Redshank repin to Mere main and this plan does not touch their manifests. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:566-586`.
- **Family source convention.** The family sits on `branch = "main"` with a committed lock and repin by `cargo update -p`; crates intentionally on `rev` are `wavicle`, `turquet`, `wgpu-graft` and the `mark-ik` forks. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:1452-1462,1516-1517`.

## Phases

### P0. Rulings and ledger: the baselines
R3 and R4 are ruled and the custody-scoping task is withdrawn as superseded by L1. What was never recorded is each consumer's `graphshell` dependency cone (a `cargo tree` per external consumer, not measurable offline because the git dependencies need network) and a rerun of the per-module `crate::` import graph.
- Done when: the baseline numbers are in Findings.
- Needs: network access to the consumers' git dependencies.
- Rulings: R3, R4
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:442-449,659-662,678-679`

### P2r. Canvas lifts that wait
Touch gestures (`touch_use`, `two_finger`) wait for host touch input in Cambium or the Genet desktop host; Mark chose to defer rather than wire synthetic events. L6 (`viewer_commands`) waits for a second host mapping the shared `command-menu` verb ids to canvas operations (Turnstone mapping `view:fit` and its siblings). L7 (`frame_budget`) waits for a second browser host. `canvas_physics` (1,104 lines) and `canvas_reader` have not moved; `canvas_reader` can follow L5 once a native second host needs a screen-reader producer. Woodshed's overview wheel is not driven natively, and `FaceAlignment` cannot see stale paint, so hosts should assert the painted camera against the live one.
- Done when: Turnstone maps `view:fit` and its siblings through the same function (L6); a second wasm host paces its canvas with `Period` (L7); the touch helpers' receipt exists once host touch input does.
- Needs: host touch input; a second browser host; the scenograph editor plan's SE48 to SE51 for the verb ids.
- Rulings: SE24 (scenograph editor plan: gestures belong in the pictograph canvas)
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:217-218,355-360,465-479,1033-1037,1083-1085`

### P2a. The remote overmap client: Turnstone's receipt
`mere-overmap` landed with its tests and both importers. Open is the Turnstone receipt: the Djinn binary starting `ResidentDialHost` (how a device grant becomes a delegation from the persona root, how a joined place's root is configured) and Turnstone's overmap pane mounting a remote mere over `LinePump`, over a session Turnstone admitted itself, showing sessions and graph. Registering a mere route on a carrier a remote device can dial is Djinn-side work the receipt needs; a dialled session has no application identity today and `AccessEndpoint::open` refuses one without it. The receipt waits on the transport note's rulings T4, T5 and T7 and a native stream-to-lines pump. Multi-route overmap composition is a design (node identity across meres, whose authority a merged view shows), not code. The archived overmap plan left stored-overmap promotion and cross-session edges (the Murm and Moot seam) without an owner.
- Done when: the crate builds with no `graphshell` dependency and its tests pass there; `graphshell_mere` and Djinn import it; Turnstone's overmap pane mounts a remote mere through it and shows sessions and graph; Turnstone's own `overmap.rs` keeps the local case.
- Needs: the transport agnosticism note's rulings T4, T5, T7 (`../technical_architecture/2026-10-15_transport_agnosticism_note.md`); a native stream-to-lines pump.
- Rulings: R7
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:273-328,480-495,1379-1384`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:632-633`

### P3r. Option helpers, host recipes and the authoring reducer: the other side
Landed on the Mere side: `scenomise::arrangement`, `scenomise::authoring` and Chronicle's recipe in `distillery::chronicle_recipe`. Knot's `composition/recipe/editor.rs` already imports `OptionFallback`, `arrangement_defaults`, `arrangement_options` and `option_fallback` from the shared arrangement module; its remaining functions supply host wording rather than duplicate the shared value resolution. `default_definition` (the Woodshed-flavoured practice recipe) stays in `projection_compile.rs` until Woodshed publishes the recipe with its dataset. The remaining L4b receipt is a second host adopting the whole-definition authoring reducer; this is separate from the already-adopted option helpers.
- Done when: Knot's `recipe/editor.rs` uses the shared helpers and its tests pass; Graphshell's editor uses the same; no `chronicle_definition` or `default_definition` remains in a shared path; a host other than Graphshell opens and saves a whole-definition draft.
- Needs: a receiving host for the whole-definition reducer; Woodshed's recipe publication (W1).
- Rulings: R1
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:225-235,370-374,496-507,890-892`

### W1, W3. Woodshed publishes its dataset
Woodshed publishes the practice host dataset; `practice_disclosure` loses the Woodshed parser; the cross-repo `include!` at `practice_workspace.rs:591` is replaced by the vendored fixture, whose provenance (which Woodshed commit produced it) becomes explicit. No progress entry exists.
- Done when: Graphshell's practice page loads `fixtures/woodshed-comparison.host-dataset.json` produced by Woodshed and `practice_disclosure.rs` loses the Woodshed parser (W1); the test passes in a worktree with no sibling `woodshed` checkout (W3).
- Needs: the Woodshed lane.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:241-252`

### P6. Forme: Knot second-host adoption (complete locally)
`platen::workspace::FormeSession` now drives Knot's actual workspace layout draft,
with View/menu-palette commands for Unlock, first-pane more/less space, Discard
and Apply. Knot keeps document sessions, focus, source undo and local persistence;
only layout state enters the shared session. Apply installs a draft after the
host acknowledges its preferences write; failures retain it. Changed tile
bindings and changed saved slots refuse stale application. Stable host bindings
allow different runtime tile allocation on reopen; scratch instances and changed
reading pins do not inherit another binding's saved layout. Unlock preserves the
host's focused document when saved active-tab geometry is restored.

The implementation is local Knot commit `2924a5f`. Its receiving tests cover the
shared session, real source Edit/Undo through layout Discard, focus, failed saves
and responsive compact-to-wide recovery. Native review covers explicit layout
authoring, a fresh process loading the same pinned-reading layout on Unlock, and
default compact presentation. The receiving record is
[`knot-editor/docs/receipts/2026-10-29_forme_workspace/README.md`](https://github.com/merely-made/knot-editor/blob/2924a5f1571f010bb9eb7692d6767117f3dff8ef/docs/receipts/2026-10-29_forme_workspace/README.md).

- Done when: the second host unlocks, drafts, discards and applies a forme through the lifted session. **Met by Knot.**
- Remaining UI scope: Frisket pointer resizing, tab drags, new splits, floats, layout-undo controls and automatic source-session reopening are separate follow-ups. Responsive presentation may constrain the visible fractions to protect the writing measure; it does not rewrite the saved canonical layout.
- Needs: no further second-host prerequisite for P6; publication remains separate.
- Rulings: R2
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:235-240,536-541,1733-1755`; Knot's receiving record above.

### P7r. Rule authoring: runtime adoption
Rule authoring and availability-aware resolution live in Scenograph and Mere View; an independent windowless host qualifies the shared boundary. Turnstone or Woodshed runtime adoption is a separate receipt, and graph occurrence binding, renderer support and inspection remain Graphshell adapters.
- Done when: Turnstone or Woodshed adopts the shared controls at runtime; not stated further.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:225-234,542-548,2047-2068`

### P8. `personal_sync` and the product operations
`personal_sync` (3,693 lines), `receipts` (1,368) and `transfer_offer` (363) are still in the port. The facet dissolution brief has been open since 2026-10-08, waiting on graph semantics' owner (the nudge is Mark's to give). `personal_sync` now imports its scene facet constants from `pandect`; what keeps it in the port is the rest of its product-facet reading. `validate_facet_name` refuses any name containing "seed", so a facet named like `arrangement.seed` cannot sync. Product operations follow Pandect live commands. The earlier Cleromancy `PersonalGraphEvent.basis` errors are historical: the later fresh-clone receiving round passed default, `personal-sync`, `graphshell-admission`, core checks and tests. That receipt does not close the remaining ownership work here.
- Done when: Cleromancy builds with no `graphshell` package; `product.rs` holds no operation Pandect now owns.
- Needs: the facet dissolution brief; the resident-host port (`2026-10-28_graph_semantics_plan.md`).
- Rulings: R5
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:267,414-420,549-555,1299-1301,1511-1515,1574-1576`

### P9. The `web_tree` views
Query editor, workbench editor and presentation controls are components. Open: documents, reading tiles and cards (`web_tree/blocks::Pane`, `reading::Reading`) mix document state with IndexedDB and rendering and move next to their owners, with load, save, export and play as effects; other owner operations keep separate completion paths; `SavedProduct` still serializes writes, the browser pumps adapters from its frame hook and the host owns the movable application and persistence task; the Graph-tools retry bridge and the query-specific completion queue are consolidation candidates; `GraphSession::edit_as` clones candidates (a performance claim needs its own measurement); real viewer dependency reduction means changing the transitive `graphshell-client` composition; saved-scene conversions elsewhere are not retired; constant-time inline-body change detection is unavailable because the kernel content revision does not cover body edits. Woodshed's adoption of these views changes its feature set (Mere's `mere-view` needs Pictograph `signals`).
- Done when: the remaining document, reading-tile and card views expose typed component inputs and effects, their port copies are removed, and a receiving host qualifies the adopted view lifecycle. The `Page` split ruling is already satisfied.
- Needs: P2r, P6 and P7r for the lifts that own each view's state.
- Rulings: the `Page` split
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:556-565,1614-1623,1673-1680,1702-1714,1766-1767,1780-1785,2042-2045`

### Q1. Runtime qualification and historical test failures
Browser interaction, Woodshed runtime adoption and physical-device acceptance are not qualified, and reading and document-tile lifecycle and the larger presentation and rules boundaries are separate from the completed slice. The archived run failures are historical observations, not a current failing-test inventory. Djinn now vendors the ESP fixtures under `ports/djinn/tests/fixtures/esp`; Woodshed's `validation/relationship-recipe/Cargo.toml` declares `tempfile`; the family composition brief records the Tabard `native_shared_recovery` compile repair. Knot's old `command_chrome` and `scroll_site` failures and the old `mere-mesh` example failures need reproduction against named current revisions before being listed as active blockers. No new full-family test run is claimed by this status correction. The vendored vello fork keeps its patch rows until the next vello repin. In Turnstone the flat law, overlay and source-form draft applies at once, and the speed pick is not in the saved scene in either host.
- Done when: not stated; owed.
- Needs: a headed browser, a Woodshed runtime, physical devices.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:9-11,1456-1459,1511-1517,1558-1564,1590-1592,2012-2017`

## Open questions

- **P9 completion criteria.** The Page split was ruled on 2026-10-15; another ruling note is not a prerequisite. Remaining work is component extraction with explicit owner completions and receiving-host qualification. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:556-565,1668-1680`
- **Multi-route overmap composition.** What identifies a node across meres and whose authority a merged view shows; no code moves until this is designed. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:313-316`
- **Who owns stored-overmap promotion and cross-session edges** (the Murm and Moot seam), left "here" by the archived overmap plan. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:632-633`
- **Murm and I2P.** The Murm desktop result does not extend to the I2P runtime. From: `archive_docs/2026-10-28/2026-10-14_graphshell_stack_lift_plan.md:2008-2010`

## Findings

- 2026-10-29: Knot now consumes the shared Platen workspace session in production layout commands. Its actual source-edit history and focus remain host-owned; native and receiving tests distinguish canonical layout from responsive presentation. See `knot-editor/apps/desktop/src/workspace_forme.rs`, `knot-editor/apps/desktop/src/documents.rs` and `knot-editor/apps/desktop/tests/workspace_forme.rs`.

- 2026-10-29: the historical ESP fixture paths have been replaced in Djinn tests by vendored `ports/djinn/tests/fixtures/esp` inputs; Woodshed's relationship-recipe manifest includes `tempfile`. Historical failure receipts remain archived and do not establish current test status.

- 2026-10-28: the app-facing half of the port is the `door` feature on `graphshell-endpoint`; Signalman and `ports/dramatis` build without the `graphshell` package. Knot's recipe editor and the shared option helpers are siblings, not copies.
- 2026-10-28: the `web_tree` views are page-shaped; twelve of its files call no browser API.
- 2026-10-28: `capture` (consented browser-history intake into Mere and Eidetic memory) is not Turnstone's today and waits on the Eidetic plan's P0 inventory.
- 2026-10-28: `scripts/check_port_boundaries.py` does not check dependency direction between ports, so Djinn depending on `graphshell` passes it.

## Progress

- 2026-10-28: cut from the stack lift plan; nothing new landed.

- 2026-10-29: corrected stale receiving-failure claims and the already-satisfied P9 ruling prerequisite; runtime qualification remains open.

- 2026-10-29: source-checked Knot's existing option-helper adoption; retained L4b as the separate whole-definition reducer receipt.

- 2026-10-29: P6 second-host adoption completed locally in Knot (`2924a5f`), with coordinated source review, receiving tests and bounded native/fresh-process qualification; no push.
