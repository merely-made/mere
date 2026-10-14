# Graphshell on one Cambium tree

**Date:** 2026-09-25
**Status (2026-10-12):** in progress. The combined viewer qualified at `8b8bd92b1`
integrates published Mere `0a3203f05`, Genet `7422e906`, maintained Vello
`10f01d6d`, and the committed site cutover `bf626abd2`. Phases 1–2 are landed; phase 3 has its recorded headed producer
receipts and Mark's approval to proceed. Phase 4 remains unfinished: canvas
controls, elapsed-time physics, saved-graph editing and remote-session slices
have landed, but the complete page/control migration and acceptance wall have
not closed. The legacy `web_gpu.rs` and `component.html` remain. The reservoir's
mere panel is its V2b step 5, after this cutover. §2's current handoff names
the remaining gates; linked plans own later editor and dynamics status. The
reservoir's V3–V5 resident implementation has advanced independently; its
process fixtures do not close this browser cutover or mount the mere panel.
**Nested host-dataset continuation (2026-10-10):** landed through
`codex/nested-graph-view`. The portable `sceno::Fold` adapter required by site
R161 combines expandable membership, entry, relationship witnesses, retained
coordinates and checkpoint history with R162's committed viewer cutover.
The former `scenomise::grouping` module is removed. The source at `8b8bd92b1`
includes current occurrence dynamics and the atlas repair in both Cargo roots.
Its 254 shared scene tests, 141 affected CPU tests and seven serialized GPU
producer tests pass; four unchanged exporter tests retain their earlier result.
Both locked Wasm gates and the complete headed viewer run pass on the default
Radeon WebGPU adapter. The run covers nested/flat data, camera recovery,
fold/checkpoint history, all 15 site checkpoints and their keys, gestures,
a planted routing fault, clean unknown-group refusal, narrow keyboard focus,
and the plain embed's mount and real page scrolling. The
[source-bound Fold receipt](../../../ports/graphshell/docs/receipts/host_dataset_folds.json)
retains commands, source/bundle hashes, captures and superseded failures.
The coordinated GPU slot is released. Control density and small glyphs remain
usability work; native S5, saved-scene integration and the live site's viewer
replacement remain separate work.
**Post-qualification main sync (2026-10-10):** published main `35ad305af`
advanced during the first push. Its Dramatis/custody migration and kernel
component-copy repair are combined, with the standalone web root aligned at
`7d78b9960`. The 254 scene and 141 affected CPU tests and both Wasm gates are
renewed. Renderer and directly recorded hosted-viewer inputs are unchanged;
the headed receipts retain their exact `8b8bd92b1` source and tested bundle.
The post-Dramatis bundle is compile-checked, with no new GPU run after slot
release.

**Historical coordinated retained checkpoint (2026-10-10):** the local five-page tree
cutover is being reconciled with exact upstream `b726955ed`. Shared Fold,
history, gestures and Forme remain part of the combined page. Existing upstream
receipts below qualify their recorded sources, not this merge. The frozen local
attempt-3 bundle passed Find, Detail, Projection, Practice and History controls;
Scene remains an actual behavioral failure, and its passive diagnostic stopped
at setup before owner status was captured. Complete current-source live scenario
parity and legacy retirement remain open. The user authorized finishing this
cutover alongside the Turnstone accessibility and Pelt pool/input lanes.
The merged inventory retains the original 132 records and adds seven upstream
scenarios: 139 source scripts, provisionally 137 live and two historical presenter
comparisons. New dynamics and Forme DOM syntax is mapped to retained controls
with original source hashes and per-line custody. These counts and translations
are preparation, not a fresh parser, build or headed qualification result.

**First site presenter (2026-10-12):** the stable-face adjacent inspector,
Fold-preserved selection and direct graph commands/gestures are hardware-qualified
at `a1cb156f8`, with all nineteen combined product rows passing. Shared-main
publication landed at `007a4de6b`; live-site delivery and complete phase 4
remain open. The detailed source bases are below.

**Current reconciliation (2026-10-13):** both recorded source histories are
preserved in the merge of the local retained-owner work with `a454842ff`.
The counts and receipts above describe their original checkpoints. Fresh
current-source build and headed qualification remain pending for this union.
**Scope:** Graphshell's browser page becomes one retained Cambium tree. Its
HTML controls become Cambium components, its display-only Cambium chrome
joins them, pictograph's canvas renders into the tree as a texture producer,
and Cambium's web host takes over `web_gpu.rs`'s compositing. The reservoir
plan's V2b then places the mere view in that tree.

**Related:**
- [Reservoir plan](2026-09-23_reservoir_plan.md), V2b, whose Graphshell step
  waits on this plan.
- `crates/cambium/cambium-genet-web-host/src/a11y.rs`, whose module doc
  recorded the web host's accessibility gap and, since phase 1, describes the
  mirror.
- [Platform boundary and repository topology plan](../../archive_docs/2026-10-06_completed_plans/2026-09-02_platform_boundary_and_repository_topology_plan.md),
  whose `p2_cambium_h3_boot` receipt first showed the chrome and the graph
  scene through Genet on the web target.

## 1. Rulings

Mark, 2026-09-25, asked which framework was the better one to go with,
Cambium or Graphshell's custom WebGPU presenter, and whether Cambium should
become the custom one. He was told:
- the page is not a rival framework but three layers: a presenter, the
  graph scene, and a UI that is mostly hand-driven HTML around a small,
  display-only Cambium chrome;
- Cambium is the better framework for the UI, since it draws through the
  same Genet, netrender and WebGPU path and adds retained views, layout,
  input routing, focus, an accessibility tree, native hosts and the
  scenario lane;
- the direction runs the other way: the presenter folds into Cambium, with
  pictograph's scene kept as a renderer inside the tree.

He chose "One tree first": move Graphshell onto one Cambium tree within
V2b, with the mere view then one component in it. Told that the web host
projects no accessibility yet, he chose "Enablers first": the projection, a
file seam and a browser-proven producer come before any control moves, so
the page never loses accessibility. The alternatives were to start the move
with the projection built alongside, or to let the mere panel start the tree
and plan the full move after V2.

Phase 1's rulings, 2026-09-25 and 2026-09-26:
- **What the mirror is built from.** Asked which option was more standards
  compliant, Mark was told genet's neutral projection is. Its roles and
  states are read from the tree's own ARIA and HTML, so lowering it back to
  ARIA is close to an identity. AccessKit's roles follow Chromium's internal
  model, and no standard maps them back. He chose "Neutral, with a leaf
  bridge": the web host lowers `document_a11y_projection` to ARIA, and a
  leaf's role and label come from its AccessKit hook, mapped to ARIA's
  Graphics Module. The alternatives were a document-vocabulary hook on
  sprigging's Leaf trait, or AccessKit shared through rootstock.
- **Focus.** "DOM focus follows the tree": a node's element takes DOM focus
  when the application's focus moves while it holds the page's focus, and
  keys that reach the mirror go where the canvas's go. The alternative kept
  focus on the canvas with `aria-activedescendant`.
- **woodshed and Redshank.** "Bump woodshed, bump redshank": the phase's third
  condition is met by moving their mere pins, not by a patched local build.
- **A text field's name.** The test page showed genet naming a text field by
  what was typed in it. Mark chose to fix it in genet's projection over
  reshaping Cambium's field or only recording it.

Phase 2's rulings, 2026-09-26:
- **Which hosts answer.** "Both hosts now": the winit host answers the same
  request with the platform's open dialog, so one seam serves the desktop and
  the browser. The alternative scoped the phase to the web host and left
  desktop apps on their own dialogs.
- **The Chrome instrument.** "Claude in Chrome sets it": the test page runs in
  Mark's Chrome, driven through Claude in Chrome, whose upload tool sets a
  file on the chooser's input. Everything after the choice is exercised, but
  not the dialog. The alternatives were Mark choosing a file himself, the one
  path through the real dialog, or the injected event alone.

Phase 3's rulings, 2026-09-26:
- **The scene path.** "Lend the host renderer": `ProducerContext` gains the
  host's `RenderCore`, and the canvas's producer rasterizes pictograph's
  `Scene` under its own key, as `GpuPresenter` does today. The alternatives
  were a `SceneProducer` kind that rootstock rasterizes, or a second netrender
  renderer booted by the producer.
- **The proof page.** "A Graphshell page": a new page in
  `ports/graphshell/web` mounts a Cambium tree with the canvas as a producer.
  It is built into the same bundle, run by the same headed runner and lane,
  and takes `nodes` and `seed` parameters. The alternative was a Cambium
  web-host example.
- **Frame times.** "Add GPU timestamps": for each path and graph, while the
  layout moves and again idle, record rAF-to-rAF intervals (p50, p95, max),
  per-frame CPU time (p50, p95), the producer's render and stage
  microseconds, and WebGPU timestamp queries around the raster work. Every
  run records visibility and fails if the page was hidden. The alternatives
  were pacing and CPU work alone, or pacing only.
- **Timestamps' plumbing.** "Through NetrenderOptions": netrender's options
  gain `optional_features` and genet's `RenderCore` forwards them. The
  alternatives were an additive boot entry in genet alone, or dropping GPU
  timestamps.
- **The Knot copies.** The genet repin returns two old-genet copies, fleece
  and layout-dom-api, that knot-editor `5ad3f67` pins. Mark first chose
  "Knot session does it first"; the alternatives were keeping the copies
  until Knot moved, or repinning knot-editor from this session. The Knot
  session then found knot-editor cannot move first: knot-desktop broke,
  because its mere pin still supplies Cambium on the old genet. Mark ruled
  mere first, confirmed here: mere carries djinn's two copies until
  knot-editor repins its mere and genet together, and djinn's pin then moves.
- **wasm-bindgen.** "Install CLI 0.2.127 too": graphshell-web moves to
  `=0.2.127`, the family the web host and mere's lock use, and the installed
  CLI matches it. The alternatives were the same move on the 0.2.126 CLI, or
  loosening the web host's pin.
- **The lane.** "Lift the lane into rootstock": one host-neutral scenario
  lane in rootstock, each host supplying its capture and receipt parts, the
  winit host re-exporting it so Knot and mere-view keep their API. The tree
  page uses it, and phase 4 moves Graphshell's scenarios onto it. The
  alternatives were extending Graphshell's page lane with a tree probe, or a
  browser copy of the winit host's lane.
- **The build timed.** "Debug, shorter windows": the side-by-side runs on the
  headed runner's debug build, which optimizes only five physics crates, and
  the 2,000-node scenarios time 30-frame windows with 60 frames to settle,
  the same on both pages. The alternatives were a release build, or the dev
  build with pictograph, netrender, Vello, genet and Cambium optimized.

Phase 3 follow-up rulings, 2026-09-27:
- **The shared runner.** Mark approved Mesquite as the owner, superseding
  the unpublished rootstock lift above. The browser supplies asynchronous
  readback; Mesquite owns driving, captures, checks and completion.
- **Proceed to Cambium.** Mark accepts the proof as grounds to begin phase 4.
  This is architectural acceptance, not performance acceptance: Cambium,
  Mere and Genet still need work. The earlier requirement to wait for a
  ruling on the numbers is now satisfied.
- **Physics remains live work.** The recorded windows use paused analytic
  layouts. They prove neither simulation cost nor moving-graph behavior.
  Phase 4 must exercise live stepping, dragging while moving, pause/resume,
  and settling, with frame pacing and simulation cost recorded separately.
  Paused rendering already takes about 1.5 seconds at 2,000 nodes, so that
  cost must be investigated independently of physics.
- **Pause freezes visible positions.** Mark chose a separate Restore
  arrangement action. Pausing must not snap a relaxed graph back to stored
  arrangement slots. Resume starts from the held positions; preserving
  velocity is not established by this ruling.

Phase 4 physics-panel rulings, 2026-10-01. Mark asked for the physics law
picker to reach the tree. The inventory had placed it in step 3's
find/arrange/physics panel, and the newest phase-4 work sat unmerged on
`canvas-elapsed-host`, which edits the same tree files.
- **The base.** Asked what the panel should build on, Mark chose "Merge that
  branch to main first". `canvas-elapsed-host` (`74ee42ff`, its receipts
  headed-passing on 2026-09-30) merged into main as `3270cac2`, and the
  panel branches from there. The alternatives were branching off the
  unmerged branch, building on main and reconciling later, or waiting for
  the branch's owner.
- **Done means.** Mark chose "Panel + all 16, migrating what they need". The
  panel carries law, overlays, the three sources and profiles, and all 16
  physics-side scenarios pass headed on the tree: the eleven per-law
  receipts, `physics_profiles`, `physics_drag`, `physics_add`,
  `physics_remote_board` and `c4b1_live_board`. Whatever those scenarios
  need comes across with them. The alternatives were the panel plus the 13
  scenarios that need nothing else, or the panel alone with receipts later.
  *Reading, not ruled:* "what they need" includes the arrangement picker
  (every law scenario first selects `free`), an `add-node` lane verb, and the
  WebRTC remote session with its session switch and remote actions, which
  today live on the old page's `BrowserHost` (`web_remote.rs`).
- **Routes.** "Both fixture and app=local": one panel component on the
  default fixture route and on `tree.html?app=local`. The law choice is not
  persisted until saved-scene restoration crosses. The alternatives were
  `app=local` only, or the fixture only.
- **Execution.** Asked how to run it, Mark said: "Buddy, i like it sequential
  during design. You feel free to orchestrate using opus and/or sonnet."
  Design forks come back to him one round at a time; implementation runs as
  Opus/Sonnet lanes in their own worktrees, verified before they reach main.
- **Placement.** Told the tree has one toolbar row over a full-size canvas,
  that `app=local` floats its detail panel over the canvas, and that the old
  page stacks its tools in one "Graph tools" aside, Mark chose "Docked 'Graph
  tools' side region": a region beside the canvas whose first section is
  Arrangement + Physics, into which step 3's later panels stack. The
  alternatives were a tabbed side region, or selects inline in the toolbar.
- **Apply.** "Keep explicit Apply": Apply arrangement, Apply physics and
  Apply profile buttons as on the old page, one rebuild per apply in
  source → overlays → law order. The alternatives were applying on every
  change, or a hybrid with live law/profile and batched details.
- *Merge gate (2026-10-01):* on `3270cac2`, `cargo test --locked -p
  pictograph -p cambium-rootstock -p graphshell` exited 0. Native only; the
  wasm build and headed scenarios run in the panel lane.

Phase 4 physics-panel follow-up rulings, 2026-10-01, after the panel lane
(`6279ca31`) passed 13 of its 14 tree scenarios headed and the remote-session
assessment returned:
- **The drag release window.** The tree's one-frame-after-release checks
  flipped between runs (Stress ≤ 20 px read 21, 31 and a pass; Anneal ≤ 60
  read 61, 68, a pass, and 292 at one step per frame), while every 300-frame
  reclaim, hold and overlap check passed. The tree advances bounded elapsed
  time, up to three physics steps a frame; the old page took one. Mark chose
  "Measure in physics steps": the lane records the drop distance on the first
  stepped frame after release. The alternatives were pinning one step per
  frame, widening the tree's thresholds, or keeping the check on the old page
  only.
- **center-node.** Energy flings the dragged node off-screen, where the
  tree's real pointer path cannot press it. Mark chose "Keep center-node": a
  camera-only pan before each press, zoom unchanged. The alternatives were
  Fit graph, which changes zoom, or a non-pointer select-and-press.
- **When it merges.** The branch also carried the standalone web manifest's
  Genet repin from `69a2383b` to root's `b1eb3af1`, without which main's web
  build failed `--locked`. Mark chose "Now, drag pending": merged as
  `8ff96d2b`, drag following as its own commit. The alternatives were
  merging after the drag fix, or cherry-picking the repin alone.
- **The remote session's home.** Its op state machine (about 437 lines of
  `web_remote.rs`) is welded to the old page's `BrowserHost`, and its
  asynchronous pumps need shared ownership. Mark chose "Into
  graphshell-client": the op sequencing lifts beside `SessionDriver`, so
  native hosts get it too, and both browser pages consume it. The
  alternatives were a shared module in the web crate, or a tree-only copy.
- **Which remote link crosses.** The old page has two realizations, the
  in-process canary `FixtureEndpoint` and the WebRTC link; the two ruled
  scenarios use only WebRTC. Mark chose "WebRTC only": the canary stays on
  the old page until the H3/C4 scenarios move. The alternative carried both,
  keeping a `GraphshellApp` alive on the fixture route.
- **Drawing the remote board.** Today 86 lines of Graphshell code
  (`remote_scene`, over `ProjectionLayoutView` and `Satisfaction`) draw it on
  the old presenter path. Mark chose "Pictograph board scene": pictograph
  gains a generic board scene, growing the footprint and backdrop types it
  lacks. The alternatives were a second producer leaf drawing the moved
  Graphshell code, or the canvas producer switching its content.
- **Where the remote controls show.** Mark chose "Section in Graph tools": a
  "Remote session" section in the docked region holds the session switch,
  the active-session line, the actions group (one button per intent) and the
  action status, on both routes. The alternatives were a detail surface on
  both routes opened by invoke-action, or a region shown only while remote.
- **Routes for the remote scenarios.** Mark chose "Both routes": they run
  over `?signal=` on the fixture route and on `app=local`. The alternatives
  were the fixture route only, or `app=local` only.
- **Anneal overrides a held node.** With the release window in physics steps
  (`4344f138`), ten laws read at most 8.2 px; Anneal read 255.2 and 190.8 px
  in two of three runs. Verified cause: `Anneal::apply`
  (`seiche/src/laws/anneal.rs:153-157`) calls `set_translation` on every body
  whose move it accepts, including the one `pin` made kinematic
  (`seiche/src/lib.rs:837`). Mark chose "Seiche: skip non-dynamic bodies":
  laws that write positions leave kinematic bodies alone, with a test that
  fails without the fix. The alternatives were re-seating the body on release
  in pictograph, or exempting Anneal from the check.
- **The session switch.** "Two aria-pressed buttons", as on the old page. The
  alternatives were Cambium tabs or a radio group.
- **Remote scope.** Told the two scenarios need no draft form, reconnect,
  disconnect or nudge UI, Mark chose "Carry their UI now": their logic lifts
  into graphshell-client and their tree UI is built in the same slice, with
  `c4b3_reconnect` ported too. The alternative deferred their UI.
- **What busy means.** Mark chose "Moving counts only when local is shown":
  a scenario `wait` holds for a pending capture, a remote operation in
  flight, or the local canvas moving while the local session is shown. The
  alternative kept the rule and accepted wait-timeouts under restless laws.
- **FlowAdvect.** The Anneal audit (`df4ede41`) found `FlowAdvect` in
  `seiche/src/coupling_force.rs` also calls `set_translation`, so it could
  override a pinned body. Mark chose "Fix it the same way": skip
  non-dynamic bodies, with a test that fails without it. The alternative
  recorded it only.
- **Narrow viewports.** The Graph tools region was a fixed 300 px. Mark
  chose "Collapse below a width": below a breakpoint it becomes a toggle
  button opening the region as an overlay; above it, docked. The
  alternatives were stacking it under the canvas, or always docked with a
  resize handle.
- **The storage status line.** On `app=local` the floating "IndexedDB
  reopened · persistent" line covered a node. Mark chose "Into Graph
  tools": a storage line in the docked region. The alternatives were the
  header status line, or leaving it.
- **Push.** Mark chose "Push now" for main, carrying this session's merges
  and rulings with other sessions' local commits already on it.
- **The board on the tree.** After checkpoint 1 (`840c63de`, `acc8920c`,
  `02a58aad`: the old page delegates to `graphshell_client::remote` and draws
  through pictograph's `BoardScene`; `c4b1_live_board`,
  `physics_remote_board` and `c4b3_reconnect` pass unchanged), Mark chose
  "One leaf, producer picks the scene": the tree's one canvas leaf
  rasterizes the canvas scene or the board scene by the session switch. Both
  scenes are pictograph's, so this is not the rejected option of switching
  Graphshell's own drawing. The alternative was a second leaf.
- **Board margins.** "Symmetric, e.g. 24 px" on the tree; the old page keeps
  its 50/50/116/64 fit. The alternative was the old page's fit on both.
- **The panel lane's two calls.** "Keep both": the narrow-width "Graph
  tools" toggle sits at the end of the controls row, and the breakpoint is
  900 px (the 300 px region plus a 600 px minimum canvas). The alternatives
  were a toggle over the canvas, or another breakpoint.
- **Graph tools overflow** (2026-10-01, after the remote lane's tree slice,
  `e9d95554`, passed all six headed runs). On `app=local` at 1400×900 the
  region ran past the window: the Link buttons on the bottom edge, the action
  status about 25 px below it, an open draft about 160 px more. Mark chose
  "Both": each section becomes a collapsible Cambium disclosure inside a
  region that scrolls vertically. The alternatives were scrolling alone, or
  disclosures alone.
- **Proving the draft form.** The live fixture's two intents take no input,
  so the draft form had never run headed. Mark chose "Third bounded intent in
  the fixture": `LiveEndpoint` gains a bounded intent, a tree draft scenario
  runs it, and the old page's three remote scenarios are re-run with the
  third button present. The alternatives were a separate flagged endpoint,
  or native tests only.
- **The board's card labels.** The tree's board drew bare rectangles, where
  the old page names each card ("Job 1 · Card 1") in DOM chrome, and the
  cards touched the leaf's top edge. Mark chose "Both": pictograph's
  `BoardScene` paints each card's title, the board projects one named node
  per card into the accessibility mirror, and the Remote session section
  lists the titles too; the top margin is fixed. The alternatives were
  painted titles with mirror names alone, or a list alone.
- **Per-card mirror nodes** (2026-10-02, after `5d77aeb2` passed the draft,
  overflow and title work headed). The mirror lets a custom leaf fill one
  node with no children, and the board is a `TextureProducer`. Mark chose
  "Producers expose semantics": `TextureProducer` gains a defaulted
  semantics method (role, name, child nodes with rectangles) that the web
  mirror writes under the leaf, changing `Accessibility::sync` for the winit
  host too. The alternatives were children on sprigging's `Leaf` beside the
  producer, or Graphshell overlaying named items.
- **Remote session starts collapsed.** Mark chose "Remote starts collapsed":
  the Remote session section is closed until a remote link exists. The
  alternatives were both sections open (as built), or the switch moved
  outside the disclosure.
- **Genet gaps.** Genet's Cambium UA sheet lacks `[hidden] { display: none }`
  (Cambium set `display: none` itself on closed panels), and Livery ignored
  `text-align: center` for card titles. Mark chose "Fix in Genet now,
  separate lane": add the UA rule and drop Cambium's workaround, honour
  `text-align` in Livery's standalone layout, and repin mere. The
  alternative recorded both for later. Asked what the fix branches from,
  since mere pins Genet `b1eb3af1`, which lived only on the unmerged
  `codex/knot-text-paint` (one commit ahead of Genet main `69a2383b`), Mark
  chose "Land codex/knot-text-paint on main first": verified, it
  fast-forwards Genet main, and the fix branches from there. The
  alternatives were branching from the pin, or from the older main.
  *Outcome and follow-up (2026-10-02):* Genet main fast-forwarded to
  `b1eb3af1971` after 627 genet-livery passes; the `[hidden]` rule landed as
  `4ac56bbbe0b` on `fix/hidden-ua-text-align` (fails without, passes with;
  628 + 124 + 40 passes). Text-align turned out general: every
  non-wrapping line ignores it, because Livery breaks it as one unbounded
  Parley line (the K3p contract) and Parley aligns within the longest line.
  A Livery-only re-break wraps no-wrap lines at inline boxes (1 line to 5);
  a prototype Parley patch adding an alignment width (`84701e00dd3`) fixes
  it. Mark said: "Is it possible to reach spec correctness while treating
  parley like a lib (much as we're doing with taffy…?) and building our
  implementations…? I lean to b but would accept c if the rationale were
  proven out by prototyping b", and on patching genet-parley: "I would
  prefer a livery only solution. So what's the path to css conformation
  look like and necessitate?" Open: an assessment of a Livery-owned inline
  layout over Parley used as a library, with option B (inline boxes
  honouring no-wrap) prototyped, comes back to him. On pushes, once
  text-align lands: "Push Genet main + fix, repin mere, push mere".
  *Ruled (2026-10-02), after three prototypes on one 14-directory WPT
  reftest run:* L (Livery-only per-line shift, no Parley patch) and C (a
  Parley alignment-width seam) each fix `text-align-end-015`, `-017` and
  `text-align-start-014` with no regressions; B (Parley inline boxes honour
  no-wrap) regresses `line-breaking-031`/`-032` and cannot be right without
  changing Parley's public `InlineBox`. Mark chose "L, if we can address the
  gaps later. if not, C"; L's gaps (a pre-existing break inside a no-wrap
  span at an inline box, and no-wrap `justify-all`) are both inside Stage 1's
  scope, so L lands. On the path to conformance, "Plan Stage 1 now": a Genet
  plan doc for a Livery-owned line breaker over Parley's shaped clusters,
  Parley used as a library as taffy is, gated on that WPT set. The
  alternatives were patching case by case, or upstream first.
  *Carried out (2026-10-02), on `genet-repin`:* Genet main is pushed at
  `bd3e8861b29`. Mere repins every Genet row to it (`7a0950da`), drops
  Cambium's inline `display: none` (`1182f642`), and centres the board
  titles with no style change (`de115df8`). §6 has the evidence. The lane
  neither pushes mere nor merges it to main; "push mere" is still to do.
  *Annotation, 2026-10-06 (S14 pass):* done. `genet-repin` merged to main
  as `04a92ffe` on 2026-10-02, and it and `7a0950da` are on origin/main.
- *Reading, not ruled:* the remote scenarios follow the panel lane's
  precedent of tree copies (`p4_tree_*`) with the originals kept for the old
  page, and the physics panel stays operable during a remote session, since
  `physics_remote_board` applies Orbit while remote.

The [phase-4 migration inventory](2026-09-27_graphshell_tree_migration_inventory.md)
maps the actual application state, controls, shared-input gaps and remaining
page/scenario obligations. The fixture toolbar is the first slice, not the
completed application migration.

## 2. Findings (verified 2026-09-25)

- **The page has three layers.**
  - `ports/graphshell/src/web_gpu.rs` (300 lines): `GpuPresenter` composites
    two netrender scenes onto the page's WebGPU canvas each frame, the graph
    content and the chrome.
  - Pictograph's canvas draws the graph, with physics, lenses and
    arrangements, as the content scene (`ports/graphshell/src/web.rs`).
  - `ports/graphshell/src/web_view.rs` (353 lines) builds the chrome as a
    retained Cambium view in a `ScriptedDom` that Genet paints. It is rebuilt
    from a `ChromeModel` on each change and takes no input.
  - The interactive UI is HTML. `ports/graphshell/web/component.html` (337
    lines) holds 89 form controls and a link, driven by `web.rs` (2,248 lines):
    - a mounted-sessions header;
    - a graph-controls bar (select, edit, pan, zoom);
    - an aside of five sections: browser history, add an object, the
      projection editor's seven tabs, find and arrange, and scene and
      transfer;
    - the node detail, a surface of its own after the aside. (The control
      count and this list were corrected on 2026-09-26.)
- **Cambium runs in a browser.** `cambium_genet_web_host::mount` puts an
  application on a canvas and feeds it DOM input. `woodshed-web` and
  Redshank's web port use it.
- **Its accessibility projects nothing yet, on purpose.** The module doc of
  `cambium-genet-web-host/src/a11y.rs` explains why. To a screen reader a
  canvas is one opaque graphic, and the DOM twin of `cambium-winit-a11y` is
  unbuilt. `cambium-winit-a11y` is 336 lines: `project_tree` turns the
  scripted DOM and layout into an AccessKit `TreeUpdate`. Rootstock's
  `Accessibility::sync` already hands a host the tree.
- **There is no file seam.** The web host has no file-open path, and the
  page's `#gs-file-input` needs one.
- **A canvas can be a producer.**
  - Rootstock's `TextureProducer` (`cambium-rootstock/src/producer.rs`)
    renders an application-owned texture into a custom-leaf slot, after
    layout and before paint. The application owns the scene state, the
    rendering and the picking.
  - Isometry's crates use it: isomere's host, isometer and eponym's client.
    Pelt's workspace and the scrying engine implement inker's
    `SurfaceProducer` instead, a different trait (corrected 2026-09-26).
  - The web host's frame loop calls rootstock's shared `redraw`, which
    prepares producers, so the path exists in a browser. Nothing has run it
    there.
- **The page's scenario lane targets HTML.**
  - `ports/graphshell/src/web_scenario.rs` (768 lines) runs taproot's grammar
    plus browser verbs over CSS selectors (`dom click`, `type`, `assert dom`,
    `assert attr`).
  - Its generic `click <selector>` misses, because the page keeps no Cambium
    surface.
  - There are 35 scenario files under `ports/graphshell/web/scenarios/`, and
    29 recorded run directories under `testing/mere/scenarios/graphshell-web/`.
- **Three Genet traits the mere view's proof met**, each a trap for the move
  (the [receipt](../testing/2026-09-25_mere_view_headed_receipt.md), verified
  2026-09-25 at netrender `aba7d837`):
  - `::before` and `::after` content drew nothing on the cells tried;
  - `text-align` did not apply inside the fixed-width box of an absolutely
    placed span, so text sat at the box's left;
  - a box with `overflow: hidden` around a self-clipping child and a
    positioned sibling blanked the whole frame.

  *Annotation (2026-10-02):* the `text-align` trait was a non-wrapping
  line. The labels' `.graph-canvas-swatch-label` set `white-space: nowrap`,
  and Genet `6fca091dc26` fixes that case.
  - A probe of the label style before `53f625fc` (absolute, 160 px wide,
    nowrap), left, centre and right, put all three at the box's left edge
    under Genet `b1eb3af1971`.
  - Under `bd3e8861b29` the centre label had gaps of 56.1 and 58.3 px, and
    the right label ended 2.2 px from the right edge.
  - `text-overflow: ellipsis` was not exercised.
  - The labels still anchor by edge, as `53f625fc` made them, unchanged
    here.

  A Cambium tree also has no `html` or `body`, so a page's base styles go
  on `:root`.
- **The Browser pane runs a mount while hidden** (verified 2026-09-26).
  WebGPU has an adapter there, and a canvas sized in CSS pixels lays out,
  but `requestAnimationFrame` never ticks and Chromium dispatches no focus
  events. `read_page` with the `all` filter reads the accessibility tree. So
  the mirror is written at mount and on a reader's action, not only on a
  frame.
- **Cambium's checkbox names itself "Checkbox"** unless its author sets
  `aria-label` (`cambium/src/controls/toggle.rs`), so unlabelled checkboxes
  all read alike.
- **woodshed-web predates `Init`'s `fonts` and `images`.** It builds `Init`
  with three fields at its pinned mere `691f9a0b`, so its pin bump needs the
  other two, a change of its own.
- **Five pages mount the view.** `index.html`, `embed.html`, `practice.html`,
  `practice-embed.html` and `co_op.html` all load it through `loader.js`.
- **A hidden Chrome tab draws no frames** (verified 2026-09-26). Through
  phase 2's run, the test page's tab in Mark's Chrome read
  `document.visibilityState` "hidden". A chosen file's answer waited 13
  seconds undelivered, since the web host's frame loop runs on
  `requestAnimationFrame` (`schedule_frames` in
  `cambium-genet-web-host/src/mount.rs`). A screenshot request timed out
  after 30 seconds but made Chrome run a frame, and the answer landed on it.
  Scripts and the mirror read throughout. Phase 3's frame times need the
  window in front.
- **Pictograph hands over a scene; a producer returns a texture** (verified
  2026-09-26). `Canvas::frame(w, h)` returns a netrender `Scene` and whether
  the layout moves (`pictograph/src/canvas/frame.rs`), and `GpuPresenter`
  rasterizes it with genet's `RenderCore`. A `TextureProducer` gets only the
  host's device and queue, built in one place
  (`cambium-rootstock/src/producer/registry.rs`), while the web host's
  `WebSurface` holds the same kind of `RenderCore`. Input and picking are
  pictograph's own: `pointer_down`, `pointer_up`, `wheel` and
  `node_at_screen` (`pictograph/src/canvas/input.rs`).
- **GPU timestamps need a device feature and pass markers** (verified
  2026-09-26). This machine's Chrome 153, on an NVIDIA Lovelace adapter,
  offers `timestamp-query`. Genet's `RenderCore` asked for no optional
  features until genet `0cf4f30ba0f`. WebGPU writes timestamps only at pass
  boundaries, and netrender's passes take none, so the GPU span is bracketed
  by marker passes around the raster work and includes any GPU idle between
  submissions.
- **The headed runner shows its page** (verified 2026-09-26).
  `run-graphshell-web-scenario.ps1` launches the same Chrome with a profile
  of its own in a new 1,400 by 900 window. Before phase 3 nothing timed
  Graphshell's frames, and nothing generated a large graph.

### Current cutover and reservoir handoff (2026-10-08)

Source and bounded intake acceptance rechecked at Mere `da466027a`, based on
`79fbbeb75` (Genet `15713014`, including the current vault-lock stack). The
resident wave published earlier as `9403b90a7`; the 2026-10-07 refresh used
`cd3ebf26d`. The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
records this slice's fresh wide/narrow browser checks and native regression
gates. They do not replace the complete phase-4 scenario or performance wall.

- **Available:** the accessibility mirror, shared file seam, canvas producer,
  shared canvas commands and toolbar, captured dragging, pause/restore,
  elapsed-time physics, saved Title/Tags editing, durable address/file intake
  through the existing owner and shared Cambium chooser, and remote-session
  controls. Intake acknowledges persistence before selecting/opening the new
  member; rejected writes retain a retryable pending member. Repeated addresses
  or byte-identical files reuse their identity. The panel now follows the
  wrapped toolbar, and current text-field sizing is checked at both widths.
  Their scope and limits remain in the linked receipts and the
  [migration inventory](2026-09-27_graphshell_tree_migration_inventory.md).
- **Still required for phase 4:** complete the remaining product controls;
  make every required page mount the single tree; retire the old presenter
  and component page; rerun the live scenario inventory with full-frame capture
  review and accessibility checks under the done-conditions in §4. Performance
  results must name their revision, graph, motion state, dimensions and machine
  load; old debug medians are not a current responsiveness verdict.
- **Reservoir boundary:** this is V2b step 4. The common mere view and route
  adapter exist, but Graphshell's mounted session panel and its lifecycle/
  graph synchronization proof are V2b step 5 in the
  [reservoir plan](2026-09-23_reservoir_plan.md). Completing a remote board or
  a scene editor does not prove that panel exists. That plan now records
  integrated archive, grant enforcement and client-first embedded-owner
  contracts with process fixtures; product adaptation follows its own gates.
- **Shared follow-ons:** the
  [Scenograph editor plan](2026-10-07_scenograph_editor_plan.md) owns shared
  history, session saves and the later pan/selection/context-menu work. The
  [dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md) owns declarations,
  compositions, channel registration, portable dynamics persistence and replay.
  Their progress does not close this plan's full page-migration wall, and
  does not establish Cleromancy adoption.

The next bounded product target is the remaining detail actions (facets,
relations, handlers and representation), followed by find/arrange and the
other inventory panels. For each implementation pass, select its unfinished
target and scenario/page gate. Record its result here and in the inventory, then
hand the mounted-panel acceptance back to the reservoir plan. Keep V3–V5's
independent resident/storage work visible there instead of treating the whole
reservoir as one browser migration.

## 3. Target shape

- **One mount per page.** Each page puts Graphshell's application on the page
  canvas with a single `mount` call.
- **One tree.** It holds the graph controls, the aside's sections, the node
  detail and the chrome as Cambium components.
- **The graph is a producer** in its slot. Pointer, wheel and keyboard input
  reach it, and picking stays pictograph's.
- **The web host mirrors the tree to DOM**, with ARIA roles, names, states and
  boxes, and it opens files.
- **What retires:** `web_gpu.rs` and `component.html`.
- **Scenarios** use the generic Cambium verbs.

A control missing from Cambium is added to Cambium and used from there, never
built inside Graphshell.

## 4. Phases

1. **Accessibility in the browser.** The web host projects rootstock's
   accessibility tree into DOM elements with ARIA roles, names, states and
   boxes. It keeps them in step each frame, and routes a reader's focus and
   activation back into the tree.
   *Done when:*
   - on a mounted Cambium test page, the browser's accessibility tree, read
     through the Browser pane's `read_page`, lists each button, text field,
     select, checkbox, tab and list item with its role and name;
   - focusing or activating one there acts in the tree;
   - `woodshed-web` and Redshank's web port gain the projection without a
     change of their own.
2. **Opening files.** The web host gains a file-open seam. A component asks,
   the host opens the browser's file chooser, and the chosen file's name and
   bytes come back as an event.
   *Done when:*
   - a Cambium test page opens a chosen file in headed Chrome;
   - a scenario supplies a file through the same event without a dialog.
3. **The canvas as a producer.** Pictograph's scene renders through a
   `TextureProducer` in a mounted tree, with pointer, wheel and keyboard
   input routed to it.
   *Done when:*
   - in headed Chrome, the fixture and a generated 2,000-node graph both
     draw inside a Cambium tree;
   - a click picks the node under it;
   - frame times are recorded side by side with today's `GpuPresenter` on
     the same machine;
   - Mark rules on those numbers before phase 4 begins.
4. **The move.** The controls, the chrome and the canvas become one tree.
   `web_gpu.rs` and `component.html` retire. `loader.js` mounts through the
   web host on every page, and the scenario lane moves to the generic verbs.
   *Done when:*
   - every scenario under `ports/graphshell/web/scenarios/` passes headed on
     the tree;
   - each recorded run under `testing/mere/scenarios/graphshell-web/` that
     still describes live behaviour is re-run, with its captures reviewed
     whole-frame, and the rest are marked historical;
   - `read_page` lists every control with its role and name;
   - all five pages mount.

The mere view's panel is the reservoir plan's V2b step 5, one component in
this tree.

### Additional target: expandable host groups (2026-10-08)

The host chooses a parent-to-member relationship kind through `?grouping=` or
the root's `data-grouping-kind`. `scenomise::host_dataset::folds` validates
this as a forest and lowers the closed frontier to portable `sceno::Fold`
facts; dependency relationships never infer ownership. Visibility comes from
`Scene::fold_effect`, with expansion choices in `FoldViewState`. Closed
ancestors fold the union of their descendants and suppress inner facts, so
active folds never overlap. Entering unfolds the ancestor path without
overwriting remembered expansion choices.
Graphshell consumes the result in its retained tree. No S1 schema extension,
stored graph mutation, or second renderer is introduced. This first slice
uses group nodes and membership edges, not nested frame geometry.

*Done when:*
- Multiple groups independently expand, including repository → workspace →
  crate nesting; collapse restores their original identities.
- Enter and breadcrumb controls retain external dependency endpoints while
  excluding unrelated material. Membership and dependencies stay distinct.
- Summary edges expose all original relationship ids, explanations and
  provenance; internal dependencies remain inspectable without fake loops.
- Opening a group preserves remembered coordinates, surviving selection and
  the viewport. This first slice explicitly pauses physics; it does not retain
  moving simulation velocities or per-item physics constraints across a swap.
- A named Cargo metadata exporter supplies declarations with package, alias,
  optional, dependency-kind, target and source coverage. It excludes absent
  endpoints and machine-local paths, and refuses missing requested packages.
- Shared tests cover a second hierarchy kind, ambiguous ownership and cycles;
  the headed web proof covers controls and the existing ungrouped S1 route.

The bounded fixture contains Mere, Genet and Woodshed repository groups and
selected real member crates. It is a manifest-declaration slice, not a claim
that current snapshot versions compile together or a full Cargo closure.
The S1 byte and compiler budgets remain in force. Larger resolved-build
datasets, lens reason accounting, scenes saved/reopened with group state,
nested frame layout follow after this bounded proof. Portable folds are now
part of this target under Ruling 161; saved/reopened scenes remain later work.

Input generation is reproducible: capture each named workspace with
`cargo metadata --locked --no-deps --format-version 1`, then pass the files
to `scripts/export_grouped_cargo.py` using `--workspace repo=metadata.json`,
`--authority`, `--output`, and optionally repeated `--package repo/package`.
Excluded standalone workspaces can use a named `repo/workspace` scope. The
source revision is a hash of the public disclosure, not a guessed Git revision.
Open `tree.html?dataset=fixtures/grouped-manifest-components.json&grouping=contains`.

### Hosted-view camera recovery (2026-10-09)

An explicit camera action follows the bounded nested-group proof. Expanding and
entering groups continue to retain the viewport; the visitor can choose **Fit
visible graph** or **Fit selection** to recover current content. Pictograph owns
the bounds and projection; Graphshell supplies the retained controls. Framing
does not restore an arrangement, move nodes, change scope or selection, or write
a portable scene fold. It stops camera following and pan inertia while retaining
yaw and tilt. Empty/hidden selection disables its control; an empty view is a
no-op. Legacy non-hosted Fit keeps its existing layout-follow behavior.
The keyboard graph leaf also advertises its existing focusability to Genet.
Rootstock reports current runner focus to accessibility independently of its
hover/restyle cache, so the browser mirror follows graph input even without a
hover event. The disabled control is excluded from retained keyboard traversal,
and its callback also guards the current selection predicate.

*Done when:* shared tests cover scoped outliers, multiple selections, the fold
summary, projected yaw/tilt and empty/nonfinite input; a headed hosted receipt
pans content offscreen and restores it through the retained action, then frames
selection without moving nodes. The grouped and flat S1 pixel gates still pass,
and keyboard activation and a 420px viewport reach the new controls.
Tab skips the disabled action, reaches it after selection, and graph input
transfers DOM focus into the graph before traversal returns to the controls.

### Graph gestures and contextual commands (2026-10-10; implementation started 2026-10-11)

After reviewing the combined viewer captures, Mark asked for most controls to
be integrated into graph interaction: drag the background to pan, use the
wheel to zoom by default, and make the mapping configurable. Scale/depth is
central to detail, context and rendering in this view; document-style wheel
panning is not its useful default. Other actions can be graph affordances or
commands through the context menu with the palette's search field.

**Baseline findings (before this continuation):** background left-drag already pans in Pictograph
(`crates/canvas/pictograph/src/canvas/input.rs`, SE23–24 in the
[Scenograph editor plan](2026-10-07_scenograph_editor_plan.md#13-the-canvas-and-command-rulings-2026-10-07)).
The viewer's `canvas_gestures.rs` and `web_tree/gestures.rs` deliberately pass
plain wheel events to the outer page and reserve Ctrl/Meta wheel for zoom.
The older main page's searchable context menu uses Cambium's shared command
set, but `web_commands` is gated by `main-page` in `ports/graphshell/src/web.rs`.
The viewer still has the toolbar in `web_tree/controls.rs` and does not consume
the canvas's context-menu request. Acceptance of those retained controls did
not establish the intended final interaction design.

**Next implementation target:** wheel over the graph zooms about the pointer;
background dragging pans. Scrolling outside the graph remains the page's.
Expose the wheel mapping as a view/host preference, with graph pan and outer
page scroll as explicit alternatives. This revises the earlier default for
graph interaction; the recorded page-scroll receipt remains evidence of the
previous policy. Touch mappings require their own acceptance rather than an
inferred change from the mouse ruling.

**Proposed integration:** bring the shared command set and searchable context
surface into the retained viewer without enabling the main-page product cone.
Register commands for what the viewer supports. Right-click and keyboard
invocation expose actions for the current node, link, group or background,
with unavailable actions explaining their preconditions. Fit, restore,
physics, arrangements, lenses, folds and relationship evidence can use this
surface. Group disclosure and selection actions can also be offered beside
their graph subjects; parameter editing opens an inspector when needed.
Representation detail follows the chosen scene/LOD policy, while membership,
scope and lens state remain owned by their existing contracts. This is an
interaction proposal, not a new geometry or materialization implementation.

**Done when:** a headed run verifies pointer-anchored wheel zoom without page
scroll, background drag pan, node drag, configurable wheel alternatives and
outer-page scrolling. Right-click opens commands; right-drag still selects
under SE26. Keyboard invocation, search, disabled reasons, selection context,
Escape and focus return work through the accessible retained tree. Wide and
narrow captures show readable graph content. Remove redundant always-visible
navigation buttons only after those routes provide equivalent access; the
existing toolbar remains until this target passes.

**Implementation scope (2026-10-11):** Mark authorized continuing after an
upstream refresh. Work starts from published `f8d8c1a79`, preserving the new
Forme draft and access/link inspection adapters. This slice carries the
viewer-supported camera, wheel preference and fold/evidence commands through
Cambium's command model and retained text/button components. It does not
implement new lens providers, field editors or scene materialization. CPU and
Wasm checks precede a coordinated GPU/browser handoff; redundant toolbar
navigation is removed only after command/gesture parity is demonstrated.
The continuation subsequently integrates published main through `e99add004`,
including Fleece/Eidetic, the current Dramatis lockstep and the exact field
inspector. The inspector's pointer and Escape handlers remain in their owner
module; this command slice does not replace its inspection contract.
The capture bridge also reconciles a completed rectangle selection with the
retained selection/detail state. Ordinary header space passes wheel events to
the embedding page; actual tools, relation lists and the command menu keep
their own scrolling.

**Validation at `685d33567` (before the presenter review sync):** the focused command, gesture, canvas-control and host
dataset CPU suites pass (32 tests); the locked/offline viewer build and the
main-page/applets Wasm check pass. The source-bound browser candidate is
prepared under `Code/output/graphshell-viewer-commands-20261011/`. No new
headed, GPU or visual acceptance is claimed. Toolbar navigation remains
present while the serialized browser handoff is pending. The read-only viewer
keeps menu choices only for its mounted lifetime; saved-product command-view
persistence remains a migration target under SE31, not a new storage lane.

### Presenter flow review (2026-10-11)

**Status:** source-backed review authorized by Mark after the command candidate.
Review basis: published `ec1e65cae`, combined with that candidate at `2bb2e7d72`.
The flow and first-consumer recommendation below are proposals for implementation,
not additional design rulings or a runtime receipt. The shared
[design language](../../2026-08-23_projection_scenes_and_graph_native_platform.md#9-configurable-visual-and-interaction-language-2026-10-09)
owns the established selection, retention and presentation semantics. The
[editor plan](2026-10-07_scenograph_editor_plan.md#r1--presentation-rules-and-the-first-aesthetic-study-2026-10-09)
owns R1 and its authoring/runtime qualification; the
[ambiance proposal](../design/2026-09-23_ambiance_design.md#10-proposal-focus-driven-relation-lenses-2026-10-07)
owns the context-lens continuation. This review connects those contracts to
the site consumer without reallocating their owners.

**Findings.** Upstream has advanced beyond the earlier study-only assessment.
The [presentation editor](../../../ports/graphshell/src/presentation_editor.rs)
now has saved rule drafts and history. Its
[exact-occurrence binder](../../../ports/graphshell/src/presentation_binding.rs)
reads held access, assertion, Numen and Forme owners; changing selection does
not silently retarget an explicitly bound preview. The retained
[product adapter](../../../ports/graphshell/src/web_tree/presentation.rs)
can opt into acknowledged saved rules and apply `RegionGuides` to one exact
Forme before canvas composition. Unsupported winning effects remain explained
decisions; a private draft does not affect the canvas. These are implemented
drafts with their own CPU/Wasm evidence and pending headed/native gates, not
complete production presentation or site adoption.

Four concrete gaps shape the next consumer:

- The read-only hosted viewer does not mount that product-only adapter. Its
  facts come from `HostDatasetV1`; it needs a disclosure adapter, not a product
  store or synthetic held assertions. Keep source binding, disclosed occurrence
  ID, relationship witnesses and revision together; Canvas node keys and the
  generated `urn:host-dataset:` address are transport handles, not new authority.
- The Canvas [representation branch](../../../crates/canvas/pictograph/src/canvas/frame.rs)
  currently changes face classes within the same measured footprint. `LivePane`
  there is a framed face request, not embedded interactive document content.
  R1's `Representation` and `Details` effects precede measurement; automatic
  representation/footprint adaptation is still an integration target.
- The product preview measures the smaller dimension of `node_body_rect`, an
  actual hit-test body that can shrink under occlusion. Bound previews start
  with an empty prior-match set. That is disclosed preview input, not a stable
  automatic LOD contract or per-occurrence live hysteresis implementation.
- [Fold application](../../../ports/graphshell/src/host_dataset_view.rs) remembers
  positions and the viewport, pauses physics, replaces the visible graph, and
  restores one focused URL. It does not preserve the complete selected comparison
  set across the swap. Hidden comparison subjects therefore need explicit
  occurrence-based view state and a policy separate from current Canvas focus.

**Ownership and composition.** Keep the one-tree architecture. The host supplies
disclosed data, capability grants, lifetime and source/view persistence. Scene
compilation and R1 resolve occurrence presentation; measurement/placement and
Pictograph supply geometry, dynamics, picking and the graph scene. Cambium owns
retained controls, focus, input routing and accessibility; its host and
Genet/Netrender composite and present the content. One retained tree can contain
several render surfaces. A future live document face keeps its engine/session
owner and needs explicit nested focus, clipping, input, suspension and capture
contracts. Opening a live access, rendering a captured clip and showing a face
remain separate capabilities. This review adds no universal presenter crate or
second workbench/session owner.

The host also needs a caption, viewport and chrome-composition contract: the
reference page's fixture heading/status and full tool inventory should not
become mandatory site chrome. Keep the command entry discoverable, with
contextual inspection and deeper tools reachable on demand. A recorded
[product inspector capture](../../../ports/graphshell/docs/receipts/forme_feedback_field_browser_20261011/chrome/final/p4_tree_field_inspection/exact_scalar_coupling.png)
was visually reviewed here: its useful exact-owner evidence is dominated by
raw UUID/JSON rows and navigation controls. This is existing upstream evidence
at its receipt's source, not a capture of the combined command candidate.
The proposed site inspector should lead with project name, summary and
relationship/coverage explanation, then expose exact IDs and source witnesses
through a reachable detail surface. Preserve that evidence while improving
the reading hierarchy; no automatic inspector should obscure the initial view.

**Candidate site flow.** Use a real disclosed repository/member-crate scene.
Labels, coverage and relationship evidence remain reachable in a list or table.
Keep mouse, keyboard, command and accessible routes equivalent.

| Transition | Presentation result to implement | Invariants / current gap |
| --- | --- | --- |
| Frozen view → interaction | Load the bounded viewer on use; retain the frozen reading on refusal or unsupported WebGPU. | Site R157's first-load and asset caps remain. No eager product cone or promised live document face. |
| Select project A | Mark A and show supported attributes/actions in an adjacent inspector. Enabled lenses disclose context with subject and source reasons. | Selection, inspection and opening are separate. Lens discovery is not yet implemented by this command candidate. |
| Add project B | Present the union once per disclosed semantic identity, retaining both reasons; offer explained shared/different readings only within known coverage. | Retain an occurrence-based comparison set independently of the single inspector and keyboard focus. Equal names do not merge packages or independent accesses. |
| Inspect a contextual item or edge | Explain the exact occurrence or contributing relationship without changing comparison roots. | Explore from here is explicit. Unsupported/absent evidence remains visible as such. |
| Zoom | Change camera scale and a supported face's detail, preserving subjects, lens scope, source identity and placement. | Decide a stable logical-pixel reference measurement and hold prior rule matches per view/occurrence; zoom does not silently fetch a wider graph. |
| Expand / enter / return | Use portable Folds and keep the viewport and surviving coordinates. | Preserve the comparison set through hidden/visible occurrences; move or return inspector focus deliberately if its target disappears. |
| Deselect A | Remove A's active lens reasons; previously interacted-with A recedes into retained background until dismissed. | Retain context still justified by B or explicit view retention. Do not erase authored edits or equate view retention with durable keeping. |
| Open | Invoke the host's disclosed operation, such as opening the project's article. | Do not create a second document session or infer permission from a representation rule. |

The adjacent inspector is the recommended first site treatment, keeping the
selected graph face's footprint stable while the richer measurement adapter is
developed. It is a bounded starting treatment, not a restriction on R1's cards,
thumbnails, nested scenes or live panes. Theme/typography values remain with
Tabard; source data remains with the site exporter.

**Options assessed.** Finishing gestures alone improves navigation but leaves
selection/context/representation transitions unspecified. A new universal
presenter would overlap the existing scene, UI and document owners before a
forcing consumer proves its API. The recommended path is a bounded host-dataset
adapter to the existing rule model, with explicit supported effects and a
single end-to-end flow. Its costs are occurrence-state reconciliation, geometry
invalidation and additional acceptance; its advantage is exercising the shared
model within the viewer's delivery boundary.

**Feature targets and acceptance.** Complete the review before toolbar removal;
then qualify independently useful command/gesture work on its actual source.
For the next presentation implementation:

1. **Disclosed occurrence state:** bind real host-dataset facts, distinguish
   unavailable from known false, reconcile selection/inspection through Fold
   and checkpoint changes, and reject stale results from an old binding/revision.
   Test two selected projects, one hidden subject, one departed occurrence and
   two distinct accesses to one Resource. Record the state lifetime/save policy;
   no second storage path is implied by this review.
2. **Supported presentation:** apply the existing resolver before measurement;
   expose matched/overridden rules and unsupported winning effects. Test a
   supported paint-only effect separately from footprint changes. Establish a
   stable reference-size input whose own representation change cannot oscillate
   the condition; exercise enter/hold/exit thresholds, missing measurements,
   occlusion and independent matches for two occurrences of the same source.
3. **Coherent realization:** painting, picking, command targets and accessible
   boxes use the same occurrence geometry and update generation. Preserve authored
   placement, pins, viewport, source facts and document sessions; disclose
   incompatible constraints. Keep camera fit, restore arrangement, reset,
   foreground retention, position pins and layout locks distinct actions.
4. **Host and delivery proof:** review wide/narrow and reduced-motion captures,
   keyboard/search/focus return, the equivalent list and frozen fallback. Before
   performance acceptance, name the actual graph, motion state, source and
   supplier revisions, logical/physical dimensions, DPR, adapter and machine
   load. Measure composition/raster, measurement, physics and frame pacing
   separately; a small fixture or old debug median does not qualify Cargo scale.

Headed/GPU/native checks require the coordinated validation slot. The existing
command bundle's hashes and earlier receipts keep their original basis after
this upstream sync; fresh CPU/Wasm integration checks do not renew their visual
qualification. Scene-rule adoption, the site's live cutover and full phase 4
remain separately open.

**Review integration checks:** at `2bb2e7d72`, the 32 command/gesture/control/host
tests and 16 presentation-binding/editor tests pass (48 total). Locked/offline
Wasm checks pass for viewer, product/applets and main-page/applets. The merge
retains Forme drop feedback, exact inspector handlers and all three text-focus
routes (commands, presentation editor and product). These are compile/model
checks, not a new bundle or headed acceptance. The review and browser inventory
are recorded under `Code/output/graphshell-presenter-review-20261011/`; the
complete citation audit is unchanged, and D2 retains its two pre-existing
baseline errors. No fresh GPU/browser/native-host run was performed.

### First site presenter (2026-10-11)

**Status:** implementation in progress, authorized by Mark: "stable project
faces, an adjacent inspector, and selection preserved through folds. Coordinate
the browser checks, then land it when those pass." The review's broader
automatic LOD, live content and context-lens targets remain later work. This
slice is a read-only host-dataset viewer treatment, not the complete live-site
sandbox retirement or phase-4 page migration.

The `inspector` presentation profile is selected by `data-presenter` or the
`presenter` query option. A host supplies its caption, summary field and link
field roles. The plain `viewer.html` consumes the actual current exported site dataset
with that profile and links to a separately retained historical capture. The reference fixture/product routes retain their existing
chrome and owners. In this profile the graph keeps its faces and footprint;
Cambium mounts the inspector beside it, or stacks a scrollable inspector under
the graph at narrow widths. Commands, fit and deeper tools remain reachable;
Fold controls and the equivalent project list are disclosed in the inspector.
The initial view has no selected project or automatic detail popup.

The [host presenter model](../../../ports/graphshell/src/host_presenter.rs)
reads validated disclosure occurrences, scalar values and relationship witnesses.
Its selection is keyed by occurrence ID within one source binding, independent
of visible Canvas members. A Fold projection changes that visible subset;
hidden selected subjects remain named and removable in the inspector, and regain
their Canvas selection on return. Inspecting another subject does not replace
the comparison set. Explicit clear removes the complete view selection. A
checkpoint preserves surviving IDs; a departed occurrence is removed from this
view with a notice. A different authority is refused before model mutation.
These choices last for the mount only; there is no new storage mechanism or
claim that a view selection performs durable keeping.

The first R1 adapter supplies exact scalar facts and known Selected/Inspected
state to an adjacent-details rule. It introduces no automatic screen-size,
foreground-pin or representation change. Unsupported navigation remains text;
only disclosed HTTP(S) addresses without credentials become navigation actions.
The inspector leads with the label, summary and disclosed relationship scope;
exact IDs, scalar fields and full witness provenance remain under source details.
Equal labels or shared sources do not merge occurrences; duplicate labels are
disambiguated in selection controls. The existing Fold adapter also preserves
every surviving visible selected member instead of just one focused URL.

**Done when:** CPU tests cover hidden/returning subjects, multiple visible
selections through the real Fold adapter, shared-source occurrences, explicit
departure/authority refusal, source immutability and navigation refusal. The
locked viewer and product gates pass. On the assigned hardware browser, run
`hosted_presenter.scn`, `hosted_presenter_history.scn` and the command/gesture
regressions, including real pointer and keyboard input, accessible selection
and inspector controls, wide/narrow captures, page/inspector scrolling and a
planted selection-loss control. Check the actual site disclosure, a fresh
mount's empty selection, link activation and clean unknown-profile refusal.
Keep source/bundle hashes and update the final acceptance before direct main
integration. The source owner coordinates the finite browser slot with the
renderer lane; no concurrent headed/GPU launch is implied by this plan.

**Candidate checks:** 54 focused CPU tests pass, including five new presenter
cases and the real Fold multi-selection regression. Locked/offline viewer and
product/applets debug builds and the main-page/applets Wasm check pass. Related
inspection preserves comparison subjects; removing a selection returns focus
through the retained graph route. The two packages and prepared browser drivers
live under `Code/output/graphshell-site-presenter-20261011/`. They are candidates,
not headed acceptance. The citation audit passes; existing D2 digest/receipt
coverage gaps are retained separately. Main landing waits for the source-bound
browser run and capture review.

**Qualification continuation (2026-10-12):** the candidate at `41a064ea0`
remains old-pin/unqualified. The renderer coordinator requires the final
coherent Genet/Netrender/provider/Fleece/Knot/Mere family before browser
acceptance; interim supplier heads are not that basis. Merge its published
Mere revision, renew the affected gates and packages, and take only its explicit
finite runtime release. At that old-pin qualification stage no headed/GPU launch had occurred.
The dev package's function-name custom section was removed with
`wasm-bindgen --remove-name-section`; all other Wasm sections are byte-identical,
reducing viewer transfer from 344.7 MB to 43.6 MB for the dev checks.

The old-pin release size gate also exposes an inherited delivery gap. With
P0's `s`/thin-LTO/symbol-strip/one-codegen-unit settings, published `04a48c2d6`
is 6,186,727 B gzip and this presenter/command continuation is 6,271,451 B
(+84,724 B). Both exceed R157's 3.25 MiB viewer cap. Glue, mount and history
fit their individual caps. These are source-bound measurements in
`baseline-size-comparison.json` and `release-size.json` under the candidate
output, not current-coherent-family or live-site delivery qualification. Keep
the cap unchanged: shared presenter integration after browser acceptance does
not authorize deploying an oversized viewer or retiring the site's fallback.

**Published-source reconciliation (2026-10-12):** `f39f6ab0b` combines this
presenter with published Mere `d46b0f5ca`, preserving the coupling editor,
node-presentation ladder and their inspector/text-input focus owners. All 67
focused CPU tests pass (the previous 54 plus eight node-presentation and five
coupling-edit cases); locked/offline viewer and product/applets debug builds
and the main-page/applets Wasm check pass on this combined source. The finite
browser batch also includes both new owners' saved-state reopen scenarios in
fresh, separate profiles. This remains an old-pin build candidate awaiting the
renderer coordinator's coherent family and explicit runtime release; no headed
acceptance or main landing is implied. The combined-source records are
`merged-source-cpu/gates.json`, `merged-source-wasm/gates.json` and the updated
`candidate.json` in the existing output directory. The earlier release-size
measurements remain bound to their recorded source commits.

**Final supplier rebuild (2026-10-12):** published Mere `631355a2f` is
integrated at compiled source `91f46146d`, with Genet `2c7ef73b`, Netrender
`c74b7cb2`, provider `2eae6243` and Knot `3af93add`. All 67 focused CPU tests,
viewer/product debug builds and main-page checking pass again; locked metadata
in both consumer roots contains only the expected supplier revisions. The
[source-bound candidate receipt](../../../ports/graphshell/docs/receipts/host_presenter_20261012/qualification.json)
records that source/package basis with false browser/GPU/visual/main flags;
those historical flags do not describe the later partial runs below.
The renderer explicitly offered and this lane accepted a finite Chrome window
of up to 30 minutes after rebuilding and confirming an unlocked console.
At that rebuild checkpoint `CGSSessionScreenIsLocked` reported true, so the
browser window remained unused and Tabard typography remained queued. Each prepared driver
checks console unlock before Chromium launch; main integration still requires
the browser checks and capture review. No toolbar/parity acceptance is inferred
from compilation.

The release viewer on this final family measures 6,378,552 B gzip against
R157's unchanged 3,407,872 B cap. Glue (18,730 B), mount (588 B) and history
(6,254 B) fit their respective gzip limits. Live-site delivery remains gated
independently. The citation/source audit passes. D2 initially had seven
findings; the renderer owner reconciled its three new document records at
`e0dae9c9`, a documentation-only descendant integrated here with all recorded
source and lockfile byte hashes unchanged. Four older D2 findings remain, and
this plan's incremental record remains present.


**Hardware continuation and current disclosure (2026-10-12):** the first
complete presenter batch, `presenter-browser-r8`, ran headed Chrome on the
AMD `gcn-5` adapter with compiled source `4a090af1a`. All eleven cases passed:
Fold and checkpoint selection, keyboard and narrow inspector access, independent
related inspection, clear/focus return, the planted selection-loss control,
unknown-profile refusal, omission of undisclosed historical links, and actual
Enter activation of a current disclosed project URL. Wide and plain-page
captures were inspected. This is earlier-source presenter proof, not final
combined command/product acceptance or main landing.

The exact public site's current `scenomise.host-dataset/v1` export is now a
separate 66,623-byte fixture, SHA-256 `69b322357acaa034c47887cbce560815f089a63304582fd8fd803c5145cb3380`,
with 22 repositories and authority-authored summaries, relationship scope and
project/GitHub URLs. The old thin history stays unchanged and exposes no
invented links. The inspector's supported named region and placement of subject
details before the expanded roster were corrected during browser review.

The physical command batch found three real input seams: DOM wheel direction
needed translation into the native canvas convention; DOM F10 needed its named
key mapping; and removing a focused accessibility proxy needed to retain page
focus ownership before mutation. These repairs are `659274653`, `aae20fb9f`
and `7cd9a985e`. The last headed attempt confirmed that physical Shift+F10
reaches the graph and opens command search after dismissal; the full batch
still awaited completion after fixing an asynchronous harness wait. A later
console lock stopped the guard before Chrome launch. That finite runtime slot
was released, with no owned browser/GPU/server left running.

Current source `8ad44a5c7` integrates main `050e7d542`, including the typography
and native-recovery owners' publications. All 68 focused CPU tests, viewer/product debug builds and main-page checking
pass on that combined source. Its renewed packages and release measurement
remain separate from browser acceptance while the human unlock is pending.
The final command/gesture and fresh-profile saved-product matrix, a renewed
presenter batch on the final combined package, and visual review remain the
main-landing gates. Earlier-source browser records retain their own hashes;
no acceptance is transferred from the other owners' native receipts. The release measurement on `8ad44a5c7` is 6,398,022 B gzip against the
unchanged 3,407,872 B cap. Glue (18,728 B), mount (588 B) and history
(6,254 B) pass their individual caps. This records the existing live-delivery
gate separately from shared presenter landing. The browser slot is released
while the console remains locked; no owned browser, server, build or inhibitor
is left running. The feature branch is reviewable; main integration awaits
the remaining hardware checks.


**Field-runtime main reconciliation (2026-10-12 evening):** Mark authorized
push/merge after the state overview. Published Mere `8568f42fd` adds F203–F205
field clocks, extent/selector membership and actor delivery. It merges cleanly
at `abaed67c1`, preserving the presenter, Fold, wheel and accessibility repairs.
The renewed batch covers the 68 focused Graphshell cases plus four actual
Canvas field-runtime integration cases, then viewer/product debug builds and
main-page checking. All 72 CPU cases and three Wasm gates pass on this combined source; both
fresh debug packages are ready. The earlier source-bound runtime results remain
unchanged. The console is now unlocked and the lane has requested a fresh
serialized browser window; final physical browser checks remain unexecuted. Main landing keeps the requested
browser acceptance condition. The last release-size measurement remains bound
to `8ad44a5c7`; it is not transferred to the new field-runtime source.


**Renewed presenter and command acceptance (2026-10-12 evening):** the finite
hardware window ran on compiled `77374c788`, with current main `8568f42fd`
and the coordinated supplier family unchanged. The command footer exposed a
real browser-mirror naming defect: options supplied their names as parent text
beside the same child text. `1fc5f9bc1` names the option and its disabled reason;
`77374c788` lowers `ListBoxOption` names through `aria-label`. A real retained-DOM
regression verifies exact name, description/disabled state, no duplicated parent
text and preserved child/source structure. Eleven mirror CPU tests and all
three Wasm gates pass. The earlier 72 model/Canvas tests retain their
`abaed67c1` basis; their directly covered source is unchanged by these two
accessibility-only repairs (83 CPU cases across the recorded inputs).

Same-bundle `commands-browser-r10` passes fixture/grouped commands and Fold
page/history checks. `commands-browser-r12` completes the physical/embedding
batch on AMD `gcn-5`, with zero page errors: actual Ctrl+K/Shift+F10 and footer
keys; external page focus preservation; right click/right-box selection;
background pan; pointer-anchored wheel zoom; node drag; precise workspace Fold
commands; kept-command and narrow-popup access; real zoom/page/outside scrolling;
Chrome's exact option name, disabled reason and state; unknown-wheel refusal;
and the expected planted wheel-control failure. Earlier failures are retained.
The harness used quiescence `wait` where it needed a fixed physical observation
window; `settle 180` corrects that timing. Its shortened Enter query selected the
outer mere Fold; the restored exact inner workspace target keeps the original
nine-node assertion. Neither correction lowers an assertion or changes source.

`presenter-browser-r9` renews all eleven presenter cases on the same frozen
source/package, with zero page errors. Reviewed final captures cover the plain
22-project page, wide/narrow inspector, independent related inspection and
wide/narrow command menus. The window was released before its 01:29:35 UTC
limit with no owned browser/GPU/server/build/inhibitor left active. The remaining
hardware gate is the nineteen-scenario saved-product/fresh-profile same-origin
matrix, queued for a separate explicitly coordinated window. Main landing
remains pending that gate. R157's unchanged delivery cap independently blocks
live-site cutover; the latest recorded release measurement remains tied to
`8ad44a5c7`, rather than being transferred to this source. The runtime coordinator
confirmed this shared-main versus live-delivery distinction.

**Final presenter qualification and retired-hover repair (2026-10-12):**
compiled `a1cb156f8` combines current main `8568f42fd` with the shared browser
name/focus repairs and a narrow Rootstock lifecycle correction. The product
run exposed a cached hover target retired by a Forme subtree replacement.
The complete primary stack is pointer movement → hover dispatch → local
coordinates → Genet lookup of the retired node; subsequent frame borrow
errors are a cascade after the Wasm abort. This is reproduced by the CPU
regression before the fix. Leave/Enter dispatch now rejects retired targets
before coordinate lookup, including a target retired by the preceding Leave
handler. Genet's strict lookup remains unchanged. Four lifecycle cases pass,
including Move replacing its own subtree; the test-only descendant is
`a4f09a372`. Two host accessibility/focus tests and all three Wasm gates pass
on the repaired runtime. Together with the earlier unchanged model/Canvas and
mirror bases, 89 CPU cases pass across the explicitly recorded sources; this
is not one newly executed full suite on HEAD.

The final frozen viewer passes all eleven `presenter-browser-r10` cases and
all 24 command/scenario/setup/physical/embedding rows in
`commands-browser-r13`, including the expected planted faults. Both run headed
Chrome 152 on AMD `gcn-5`, with zero page errors. The original nineteen-row
product matrix also passes, recorded by `product-matrix.json`: twelve rows in
r6, dynamics and coupling pairs in r7, the ladder pair in r8, and the actual
Title-focus Ctrl+K/Escape case in r10. Each logical pair uses a fresh profile;
the Forme four-row sequence and each reopen pair retain their own origin.
All accepted runs have zero page errors. Original scenario assertions remain
unchanged. Failed setup receipts are kept: the dynamics overlay requires its
accepted 615×608 viewport; the ladder requires actual Play activation before
its original Pause step on the intentionally paused seed; and the custom
shortcut check now compares the real mirrored Title values across physical
keys, rather than an unavailable cross-run scenario checkpoint or native-input
API. Exact source/package/driver hashes bind each continuation.

Reviewed captures show the plain 22-project page, wide/narrow inspector,
independent related inspection, menus and product reading/editing controls.
The existing small graph glyphs stay stable; this bounded presenter does not
claim final overview typography, automatic LOD or large-graph performance.
The source-bound receipt now qualifies shared-main publication. All owned
Chrome, HTTP and inhibitor resources were closed and explicitly released at
02:37:55 UTC, before the agreed 02:42:45 UTC cutoff. Shared main publication is confirmed at `007a4de6b`. The final merge of
upstream `abc8e5ad5` changes only documentation; all recorded runtime source,
manifest and lockfile byte hashes remain identical to the qualified basis. Live-site deployment, R157's unchanged size cap,
native/touch qualification and complete one-tree phase 4 remain separate.
The latest release measurement stays bound to `8ad44a5c7`; no new release size
is inferred from these debug packages.

### Viewer delivery budget (2026-10-13; published)

Mark authorized reducing the viewer below R157's unchanged 3,407,872-byte
(3.25 MiB) gzip limit so the site can replace its sandbox. Work starts on
`codex/viewer-size-budget` from published `ea9239a3f`, preserving the accepted
presenter, commands, Fold/history and explicit unsupported-product/remote routes.
The first load remains frozen under the site's separate 64 KiB cap.

**Target:** remove capabilities outside the read-only cone, select a reproducible
shipping profile, enforce all asset caps, and qualify the exact candidate before
site adoption. No cap increase, new source authority or fork of the site's data.
Platen's document producer is now a default-on `document-layout` capability;
its pane plans, geometry and structural accessibility build without it.
Graphshell's native and product hosts keep document layout, while the viewer
selects the portable model. This prevents an unused document producer from
activating Parley's dictionary segmentation throughout the viewer cone. The
text engine and dictionary data are not patched. Consumer feature inspection
must retain the full document capability in the product build.

**Exploratory evidence:** a fresh no-defaults baseline is 6,416,541 B gzip.
Five ICU dictionary blobs are found byte-for-byte in its data segment; the
feature path is Graphshell web → Platen → document-canvas → Parley complex
scripts. Separating that producer reduces the same-profile package to
3,910,153 B gzip, still 502,281 B over budget. Fifty portable Platen tests and
54 default-feature tests pass, including document scene tests. `wasm-opt`
`-Os`, `-Oz` and converged `-Oz` each worsen compressed size, with the failed
measurements retained. A separate `viewer` profile inherits release, uses `z`
and full LTO, and leaves the established full-page release profile unchanged;
its first exploratory package is 3,383,277 B gzip9 and 3,398,093 B gzip6.
Both fit narrowly. The normal shipping package at `cdba4c969` measures
3,379,962 B gzip9 and 3,396,363 B gzip6, with 11,509 B spare at gzip6.
Glue, mount and history also pass their individual caps. Portable/default
Platen tests, the two size-gate controls and all four Wasm feature builds pass.

That exact package passed 24 command cases, eleven presenter cases, all
seventeen physics receipts, four browser pointer/keyboard assertions and three
clean product/remote refusals on headed Chrome 152 with AMD gcn-5, with no page
errors. The final 21-node synthetic diagnostic has 16.7–16.8 ms median display
intervals across three windows; the last window includes an 82.6 ms maximum.
This is a count-matched diagnostic, not the site's actual data or a broad
performance guarantee. Existing missing Arabic/CJK/Thai glyphs occur in both
baseline and reduced packages; disclosed strings agree, Latin/Greek remain
readable, and mixed-script line breaks differ. No expanded font repertoire or
identical complex-script segmentation is claimed.

The original timeout/focus receipts are retained. Density needed 420 seconds
for its 7,400-frame scenario; presenter checks wait for selected-state/link
projection; the keyboard scenario needs twelve Tabs through the current panel
(the old eleven-Tab recipe fails on both baseline and candidate). Assertions
remain unchanged. Browser, server and inhibitor resources were explicitly
closed at 07:19:09 UTC on 13 October. Receipts are under
`ports/graphshell/docs/receipts/viewer_budget_20261012`.

Subsequent main `0d1dc1927` adds retained Pelt content and Moot controls.
Reconciliation preserves these capabilities in native defaults and web product
builds through an explicit `retained-content` feature, keeping the read-only
viewer free of their document producer. The standalone web workspace also
restates the root p2panda compatibility patches, which Cargo does not inherit.
The reconciled runtime (`3f57971e0`, packaged at comment-only successor
`47199232d`) measures 3,303,044 B gzip6 and 3,289,510 B gzip9, leaving
104,828 B at the stricter measured level. Raw Wasm is 9,474,800 B. Glue is
19,147 B gzip6; mount 588 B and history 6,704 B. All four caps and all four
Wasm configurations pass, with product complex-script support present and
viewer support absent. The exact rebuilt package independently passes all 24 command, eleven
presenter, seventeen physics and four browser pointer/keyboard checks, Unicode
comparison and the three expected unsupported-product assertion failures with
no page errors. The new complete 21-node live frame windows have 16.6–16.8 ms median,
at most 18.1 ms p95 and 28.3 ms maximum intervals. The paused windows reach
18.7 ms p95. These complete frame windows differ from the separate rolling
120-sample interval arrays; this remains a bounded diagnostic. Chrome 152 uses AMD gcn-5. All four driver processes exit
zero, and the browser/server/inhibitor are closed at 07:44:12 UTC on 13 October.
The earlier package and failed controls remain independently preserved.
Source/asset/driver/served/auxiliary hashes are frozen in the candidate ledger.
Main publication is confirmed at `425b04ceb`. The publication commit changes
only receipts and documentation relative to the frozen runtime. Live-site
cutover remains separate.

The builder enforces all four interaction caps at gzip6 and gzip9. The portable
workflow invokes it in a separate Linux delivery job and retains package/report.
GitHub's repository permission check reports Actions disabled on 13 October;
remote CI execution is not claimed. The site's separate first-load cap and
static/mobile/non-WebGPU/failure reading remain site-cutover gates.

**Done when:** the normal shipping build and exact deployment recipe fit each
R157 cap with source/profile/feature/package hashes and an automated regression
gate; portable/default Platen, viewer/product/main-page compile and relevant
shared tests pass; the exact optimized viewer receives a serialized hardware
browser check for presenter, controls, accessibility, Fold/history, physics and
clean product/remote refusals. Compare reading and text behavior on the actual
site disclosure and representative Unicode inputs. Preserve static/mobile and
non-WebGPU reading, first-interaction loading, source validation and failure
fallbacks during site replacement. The final source is reconciled with upstream
before publication; exploratory measurements do not transfer acceptance.

## 5. Stop rules

- Nothing a screen reader reaches today is lost: a control moves only once
  phase 1 projects its kind.
- The canvas's rendering, physics and picking stay pictograph's. This plan
  moves where the canvas is hosted, not what it draws.
- No Graphshell-only UI pieces. What Cambium lacks is added to Cambium.

## 6. Progress

- **2026-10-12 — first site presenter and direct controls landed:** shared main
  and `codex/nested-graph-view` both confirm `007a4de6b`, containing the final
  presenter, command/accessibility/input repairs, retired-hover fix and full
  qualification evidence. Runtime acceptance is source-bound to `a1cb156f8`;
  the later test-only and documentation descendants change no runtime inputs.
  All nineteen product rows, eleven presenter cases and the full command pass
  are accepted, with preserved failed setups and expected planted controls.
  The [landing receipt](../../../ports/graphshell/docs/receipts/host_presenter_20261012/landing.json)
  records exact remote refs. No live-site deployment, full phase 4 completion
  or release-size improvement is claimed.

- **2026-10-10 — concurrent Dramatis publication combined before main push:**
  the qualified branch push succeeded, while main correctly rejected a stale
  fast-forward after `35ad305af` published. Integrated that migration without
  conflicts. DR-D moves identity packages to published Dramatis `c1d62076`;
  the separate web root needs the root's Muniment patch and a renewed lock,
  removing unused password-custody packages (`7d78b9960`). A cold offline
  cache initially lacked the new Knot pin; locked metadata fetched it before
  successful offline checks. All 254 shared scene and 141 affected CPU tests,
  both locked Wasm gates, portable metadata and headers pass again. The renderer,
  Rootstock and directly recorded viewer/fold/history/gesture/framing sources,
  fixtures and assets are unchanged. Headed/GPU results remain explicitly
  qualified at `8b8bd92b1`; the new bundle is compile-checked. No GPU work was
  launched after releasing the slot. The receipt distinguishes these bases.

- **2026-10-10 — combined Fold/viewer acceptance closed for main landing:**
  integrated the published image-atlas repair (Mere `3808c5a22`, Vello
  `10f01d6d`) at `07bc7af58`, aligned the independent web Cargo root at
  `2fdeff569`, then combined current main `0a3203f05` and its occurrence
  dynamics. The standalone lockfile needed one `scenograph` dependency entry
  for pictograph (`8b8bd92b1`); no external dependency version changed.
  - **Renewed gates:** 254 shared scene tests (including ten Fold cases),
    141 affected CPU tests and all seven Rootstock GPU producer tests pass.
    The latter ran one exact case per process on the assigned GPU slot,
    with a 90-second bound and stop on failure. The viewer build and
    main-page/applet compile check both pass locked and offline. The unchanged
    exporter retains four passing tests. Portable-lock metadata and the owned
    source header gate pass. The exact atlas reproduction is recorded in
    `LICENSES.md` under the existing frozen-evidence rule, with its original
    source digest verified; the recorded reproduction bytes are unchanged.
  - **Headed acceptance:** Chrome `152.0.7977.83`, adapter `amd` / `gcn-5`,
    passes all nested/camera/flat/fold-history/gesture scenarios and all 15 site
    checkpoint buttons with Home, End, arrows and Page keys. Unknown membership
    is refused visibly without page errors. The planted propagation fault
    fails exactly the intended assertion. Tab skips the disabled selection
    action, reaches it after selection and preserves graph focus; both camera
    recovery actions work at 420 pixels. The plain embed has no loader or
    scenario/receipt globals, mounts only the viewer, keeps its checkpoint
    controls and lets a real wheel scroll the page from 0 to 240 pixels.
  - **Evidence and limits:** the fresh bundle and actual readbacks/screenshots
    are recorded in the Fold receipt. Its old device-loss and partial browser
    failures remain historical evidence. The final harness corrected reversed
    Page-key expectations, an already-paused history precondition and the
    graph reader's group selector; no acceptance assertion was dropped.
    Current controls are still dense and some graph glyphs small. This closes
    the bounded hosted Fold/history slice, not the whole one-tree migration,
    full compiled Cargo closure, performance budget, saved ambient state,
    native S5 or live-site adoption. Site-owner WIP and the primary checkout's
    five unpublished commits are untouched. Browser and receipt server are
    stopped and the GPU slot is released. Earlier hold entries below describe
    superseded checkpoints.

- **2026-10-10 — source sync before the explicit GPU-slot handoff:** merged
  main `3eda5d278` at `4c18d597d`, including native workshop narrow-header
  layout and the renderer lane's Graphshell appearance receipts. Viewer
  source/assets, renderer pins/locks, Rootstock, Mesquite and shared scene
  inputs are unchanged, so their gates retain the checked source bases in
  the Fold receipt. The coordinator still holds all headed/headless GPU runs
  pending a single-slot handoff after Turnstone acceptance. Its cooperative
  wait repair belongs to Turnstone; no new shared-renderer fix is reported.
  Upstream appearance receipts do not close this branch's combined Fold and
  history acceptance. This lane has launched no GPU or native capture.

- **2026-10-10 — published Moot/Pelt continuation combined:** merged main
  `4e1811578` at `5884b94a0`, including Moot's Commons conversation adapter
  and Pelt's durable appearance/native acceptance record. This merge changes
  no viewer source or renderer pin. Both locked Wasm gates, 22 host/history
  tests and six capsule tests pass on the combined source, along with
  portable-lock metadata and header verification. Other retained gate inputs
  are unchanged. The receipt names the source base for each renewed or
  retained CPU gate. Pelt's separate application acceptance does not close
  this lane's headed viewer acceptance; the coordinated GPU hold continues.

- **2026-10-10 — published guarded renderer integrated; GPU hold continues:**
  merged published Mere `11236fd4f` at `5cea11408`. Both Cargo roots and their
  lockfiles now select maintained `netrender-vello`, its encoding and shaders
  at `491c376c`. Git ancestry confirms that this retains the earlier Radeon
  coarse traversal repair `865cbf419`; the resolved renderer dependencies also
  use the guarded source. The existing compatibility `vello` tag keeps its
  separate encoding entry, so this is not a claim that every encoding package
  in the workspace has one source.
  - **Renewed checks:** 141 affected CPU tests pass with this pin, including
    all 67 CPU Rootstock tests; its seven GPU fixtures are explicitly filtered
    out. Both locked Wasm gates and portable-lock/header checks pass. The 254
    shared scene tests and four exporter tests retain their previous results:
    their source/dependency inputs did not change in this renderer publication.
    No prior result log was overwritten by the renewed run. The
    [Fold receipt](../../../ports/graphshell/docs/receipts/host_dataset_folds.json)
    carries the new source hashes, commands and separate evidence paths.
  - **Acceptance boundary:** the renderer lane's
    [theme-modes record](2026-07-05_theme_modes_plan.md) reports real
    Radeon/Metal command guards, renderer regressions and 60 workshop preview
    readbacks. Those diagnostics do not qualify the combined Graphshell
    viewer. The coordinator continues the GPU hold during serialized native
    application acceptance; this lane launched no GPU or native capture for
    the integration. The historical seven device-loss failures and the final
    headed viewer checks remain outstanding, and main landing remains pending.

- **2026-10-10 — CPU/build qualification complete; GPU acceptance held:**
  incorporated published Mere `b513994ba` (shared capture polling and Genet
  `7422e906`) at `178779e2c`, then `6183006ba` (shared Tabard appearance)
  at `6b0798a95`. The appearance merge retains both palette application and
  reduced-motion pause. On these combined sources, 395 Rust tests and four
  exporter tests pass: 254 shared scene tests, 22 host/history tests, six
  gesture tests, six framing tests, 67 CPU Rootstock tests, 27 Mesquite tests,
  one legacy-control test, six applet tests and six appearance tests. The
  locked standalone viewer build, main-page/applet Wasm check, portable-lock
  metadata verification and owned-source header gate pass. The latter needed
  three Exhibit A comment lines in the published browser appearance adapter
  (`2ef6e3207`); no executable behavior changed.
  - **GPU boundary:** the full Rootstock suite ended at 01:51:43 Eastern with
    67 passed and seven GPU producer fixtures failing with "Parent device is
    lost". The CPU rerun explicitly excludes those fixtures and does not close
    them. A separate native-acceptance lane supplied the 01:46:41 Radeon reset
    report and WindowServer watchdog failure: FirstPendingCB identifies Pelt's
    acceptance `product` PID 43570. This identifies a pending submission,
    not a proven root cause. No headed GPU or native capture is being launched
    by this lane while that incident is investigated.
  - **Remaining acceptance:** regenerate bindgen output and refresh served
    assets, then run the combined nested/camera/flat/history/gesture captures,
    all 15 site checkpoints with Home/End/Page keys, planted gesture routing
    control, plain viewer mount, real page wheel, and narrow keyboard/focus
    checks on current sources. Historical headed receipts below do not qualify
    the new Genet pin or appearance integration. The branch is not yet landed
    on main; site-lane WIP and primary main's five unpublished commits remain
    untouched. See the
    [source-bound pending receipt](../../../ports/graphshell/docs/receipts/host_dataset_folds.json)
    for commands, counts, hashes, incident evidence and local receipt paths.

- **2026-10-10 — earlier combined viewer qualification:** native checks
  passed on the integrated site/Moot sources. The first headed Fold/history
  receipt passes. The gesture receipt exposed an inherited probe bug:
  `gesture_steps::pointer` read `cancelBubble` after dispatch, when the browser
  has reset its propagation flags. The probe now observes delivery at the
  host's canvas; an injected routing fault will test that it can fail.
  The site walkthrough passed all checkpoint button steps but exposed the
  shared scenario driver's missing Home/End/Page key names. Mesquite now
  dispatches those existing named keys, matching the history slider's handlers.
  Viewer and main-page/app-host checks passed after this probe change; the
  final headed rerun remains held as recorded above.
  Portable metadata/lock provenance passes. The CI header gate also caught
  the published macOS scenario runner's missing Exhibit A notice; adding the
  standard owned-source header makes the gate and its planted controls pass.

- **2026-10-09 — Rulings 161–162 adaptation, in progress:** incorporated
  published Mere `3567c8e93` and site cutover baseline `7c0c12008` without
  rewriting the pushed branch. The only merge conflict was the root lockfile;
  resolution retains current main's dependencies and the qualified renderer
  source. The former `scenomise::grouping` module is removed. The host adapter
  now emits `sceno::Fold` facts with exact instance membership, a member
  stand-in, the explicit descendants rule, source label and boundary counts.
  Viewer disclosure derives from the portable fold effect and retains original
  relationship witnesses. The complete source scene remains available beside
  the viewer graph, including hidden instances. Tests and headed receipts are
  being renewed on the combined sources. The site's committed checkpoint
  `bf626abd2` is merged at `25e356782`, followed by published main `421818710`
  (including Moot) at `c8ac18ba9`. Uncommitted site-lane docs and scenarios
  remain untouched. Checkpoint changes rebuild the same Fold reading: surviving
  group choices and coordinates carry forward, choices naming removed groups
  clear, and each fresh disclosure reapplies change marks once. A checkpoint
  without the requested membership kind refuses without replacing the previous
  view. The changes list remains complete even when occurrences are folded.

- **2026-10-09 — hosted camera recovery and live accessibility focus:** merged
  published main `dafacc25c` at `6a7c7d8fa`, retaining both the grouping and
  current history/facet modules. Hosted controls now offer one-shot **Fit
  visible graph** and **Fit selection** through shared Pictograph framing.
  They retain node placement, selection, scope, yaw and tilt; successful framing
  clears pan inertia and camera following. The existing non-hosted **Fit graph**
  continues to resume following. Selection fit excludes hidden/folded members,
  and its retained control is unavailable without a finite visible selection.
  - **Shared focus repair:** graph input already changed retained focus, but the
    leaf did not advertise that capability to Genet. After adding `tabindex`, a
    combined pointer/keyboard receipt still failed: Rootstock accessibility sync
    forwarded `last_focus`, a hover/restyle cache. Sync now reports current runner
    focus through the current `WindowDom` while leaving that cache intact. The
    native regression changes and clears focus without hover and checks both the
    reported ID and projected focused node.
  - **Passed native/build gates:** six framing tests cover scoped outliers,
    multiple selection, fold-summary bounds, yaw/tilt, nonfinite/empty input and
    narrow singleton centering. All 73 Rootstock library tests, eight host-input
    tests, the existing zoom-command test, 162 Scenomise tests and four exporter
    tests pass. The locked standalone viewer builds, and `main-page` checks
    without GPU physics. Rootstock and both wasm gates were rerun after the
    shared focus fix.
  - **Passed headed proof:** Chrome `152.0.7977.83` on the default AMD `gcn-5`
    WebGPU adapter passes the three original group captures, two new recovery
    captures and the original flat S1 capture, without shader hooks. The recovery
    scenario pans the entered nine-node view offscreen, frames it, then centers
    selection within one pixel. Positions, picked occurrence, node/edge counts,
    camera and zoom satisfy the recorded assertions, including 60-frame holds.
    Unknown grouping is refused without a page error. The original combined
    keyboard sequence now transfers DOM focus into the graph, skips the disabled
    action, reaches the enabled action and activates both fits at 420×800. No
    focus-transition workaround is used. Final wide and narrow captures were
    inspected.
  - **Evidence and boundary:**
    `ports/graphshell/docs/receipts/hosted_camera_recovery.json` records source,
    debug-bundle and PNG hashes; local logs/captures are under
    `Code/output/nested-graph-viewer/receipts-camera-recovery/`. The outside-view
    assertion measures node centres, not all labels or relationship extents.
    The normal 0.1 zoom floor still limits very large graphs. Dense group/toolbar
    chrome, tiny glyphs and on-canvas labels need further usability work. This
    closes the bounded recovery target, not saved scenes, ambient lens reason
    accounting, full Cargo performance, release transfer budget or site cutover.
- **2026-10-08 — Radeon coarse-command repair:** merged current main
  `fbc149bd9` at `26f4c19a0`, retaining the new intake controls and coherent
  Genet `15713014` pins. On the default Radeon Pro Vega 56 adapter, the old
  coarse loop processes one entry from sampled bitmap slices containing four
  entries, leaving an opening clip without subsequent drawing or closure.
  Binning and path bounds are valid. The maintained Vello fix `865cbf41`
  traverses slices explicitly, then consumes each set bit in ascending order.
  Mere and standalone Graphshell web patch Classic, encoding and shaders to
  that one immutable source, preserving 0.10 APIs and buffer recovery.
  - **Source-built headed proof:** the original nested-group and flat-dataset
    scenarios pass all four strict captures on the default Radeon adapter,
    without shader hooks or a software adapter. Keyboard expansion, entry and
    breadcrumbs pass, and unknown grouping refuses without a page error.
    Wide and 420×800 captures display graph nodes, edges and retained controls.
  - **Provider gates:** two new native CPU/GPU regressions cover empty slices,
    bit 31, later-draw order and clip closure across slices. Nine existing
    comparisons and allocation-count buffer recovery also pass: 12 tests,
    with the existing long-running large-bin comparison still ignored.
  - **Immutable consumer gates:** the Git-pinned locked viewer build and the
    locked `main-page` check without GPU physics pass; all eight host-input
    tests pass. The hardware browser run repeats all four captures and keyboard
    gates from that bundle. Substituting only the old coarse shader restores
    the failed pixel gate without GPU/page errors. The linked JSON receipt
    retains the initial failures and software run, then records the repaired
    hardware acceptance with source, bundle and capture hashes.
  - **Boundary:** this repairs the qualified Chrome/Radeon rendering failure;
    it does not identify the exact Tint or driver instruction. It is a 0.10
    maintenance fix, not adoption of upstream Vello 0.11. Preserved-camera
    offscreen content, narrow-screen control density, saved scene state,
    large Cargo closure/performance and live-site cutover remain open.
- **2026-10-08 — current-main qualification:** nested-group branch merged
  published main `42d6f17f6` at `7ec9c4503`. The standalone web manifest now
  agrees with the root's Genet revision `e84f9c7f9`; its old explicit pins did
  not inherit the root repin. Repeated checks passed: 125 Scenomise tests,
  eight host-input tests, four exporter tests, the viewer wasm build and
  `main-page` product/remote compile check without GPU physics. Headed
  grouped/flat scenarios and keyboard activation retain their semantic
  passes, but all four strict pixel captures remain blank. A direct WebGPU
  clear control displays correctly in the same Chrome setup. Instrumentation
  records compute dispatches, nonzero render dimensions and a zero bump
  failure counter; sampled intermediate scene/staging texture centres are
  transparent before the final composition. The exact renderer cause remains
  open. These diagnostic samples are not a rendering or performance receipt.
  - **Default-adapter boundary:** the same unmodified page displays on
    Chrome's SwiftShader WebGPU adapter. Chromium 151 also remains blank on
    the default adapter. The machine has a Radeon Pro Vega 56; three pinned
    native Netrender controls pass, including a clipped rectangle. Sampled
    input has balanced begin/end tags and GPU clip fixups, while hardware
    coarse commands leave an open clip. Temporary output controls display
    pixels and dimensions, narrowing the issue to compute rendering without
    identifying the exact compiler or shader fault. No shader substitution
    is shipped. On the corrected SwiftShader WebGPU launch, the original
    grouped scenario passes all three strict pixel captures and the flat S1
    scenario passes its capture. Keyboard expansion, entry and breadcrumbs
    pass; an unknown grouping kind refuses without a page error. The expanded,
    entered, collapsed and 420×800 narrow screenshots were inspected. They
    display the graph and retained controls, but the preserved camera leaves
    some nodes offscreen and controls take substantial narrow-screen space.
    This software acceptance, with six PNG hashes in the linked receipt,
    cannot close the default-adapter gate, site usability or performance
    budget. The unmodified code qualified is `0051258e4`.
- **2026-10-08 — expandable host groups:** implemented on
  `codex/nested-graph-view`, based on main `4c796590c`. The shared Scenomise
  forest projection preserves source occurrence identities and dependency
  witnesses. Graphshell's host-input controller retains current coordinates,
  viewport and surviving single focus; controls independently expand groups,
  enter them, expose external context and navigate breadcrumbs. Changes pause
  physics. The Cargo declaration exporter produced a bounded Mere/Genet/
  Woodshed fixture with 18 occurrences, 34 relationships and 51,298 raw bytes.
  - **Passed:** all 125 Scenomise tests, including six grouping tests; eight
    Graphshell host-input tests, including a real Canvas swap with changed
    coordinates and camera; four exporter tests. The wasm viewer builds, and
    the `main-page` product/remote configuration checks without GPU physics.
  - **Browser semantics passed, capture gate failed:** headed Chrome at
    1440×1000 ran every assertion in `nested_host_groups.scn` and the flat
    `tree_host_dataset.scn`. Both receipts fail only their capture detail
    checks: all captures are blank. No page errors or gate failures occur.
    A five-second warm-up did not resolve it; the software-renderer attempt
    timed out before ready. Retained DOM keyboard activation and breadcrumbs
    pass, and an unknown membership kind refuses with an alert and no page
    error. This establishes disclosure/control behavior, not visible graph
    rendering or layout usability. The flat route is the same built viewer
    without grouping, not a separately rebuilt historical binary.
  - **Evidence:** `ports/graphshell/docs/receipts/nested_host_groups.json`
    summarizes the bounded tests and source hashes. Full runtime captures and
    receipts remain in the local `Code/output/nested-graph-viewer/receipts/`
    directory; the later software-adapter run is kept separately in
    `receipts-swiftshader/`. Default-adapter visual qualification, saved scene
    adoption, portable folds, nested frame geometry, full Cargo
    closure/performance and the live site replacement remain open. This commit
    is not a deploy or a size-budget gate.

- 2026-09-25: plan written from the assessment in §2 and Mark's rulings in §1.
  The reservoir plan records them as §7 items 39 and 40.
- 2026-09-25: the mere view's headed proof met three Genet traits the move
  will meet too; §2 records them.
- 2026-09-26: phase 1's mirror landed on `reservoir-v2` (`c19e1120`), with
  genet's text-field fix (genet `1b62fd0b218`) and mere's repin to it
  (`685c830e`).
  - `cambium_rootstock::document_projection` gives a frame's layout as
    genet's neutral projection. `cambium-genet-web-host::mirror` lowers it to
    ARIA, and `a11y` writes one element per node over the canvas, changing
    only what moved. The canvas is `aria-hidden`, and the mirror is a region
    named for the application.
  - The example `a11y_page` mounts one of each control. In the Browser pane,
    hidden, `read_page` listed the region "Accessibility page" and in it the
    heading, button "Press", textbox "Name", checkbox "Subscribe", combobox
    "Colour", tablist "Sections" with tabs One, Two and Three, list items
    Alpha, Beta and Gamma, and the status line.
  - Clicks on mirror elements pressed the button, ticked the checkbox, picked
    tab Two and chose Green, and the mirror showed each before the click
    returned. A focus moved Cambium's focus to the field, and typed text
    reached it; the hidden pane fired no focus events, so that path took a
    dispatched `focusin`.
  - Typing "Hi" first renamed the field "Hi", with an empty value: genet
    named every element from its own text before its label, and read a text
    control's value only from a `value` attribute, while Cambium's field
    holds its text as children. After the fix and repin, the field keeps the
    name "Name" and reads "Hi" as its value.
  - The mirror's six native tests lay the page out through the winit
    harness and check each control's role, name and states, and each box
    against where layout painted it.
  - Left for the phase: woodshed-web and Redshank's web port, by their pin
    bumps.
- 2026-09-26: phase 1 is done. woodshed moved to mere `149b8053` and genet
  `1b62fd0b218` (woodshed `a0910d5`, Redshank `d721a80`).
  - Redshank needed no change of its own. woodshed needed four, all drift
    from its pin being 335 commits old: two patch-table version
    requirements, the obsolete `parley` patch row, the `genet-probe` →
    `taproot` rename, and `Init`'s `fonts` and `images`.
  - In the Browser pane, woodshed-web's mirror, the region "Woodshed",
    reads its S0 sheet's text. Redshank's web port lists its tablist, tabs,
    status, panel buttons, Seek slider, transport buttons and Capture
    group. Choosing Library through the mirror switched Redshank's
    destination.
  - The pages were built from the bump worktree beside test pages under
    `Code/testing/cambium/` with fixed-size canvases, since the hidden
    pane's viewport is 0 by 0.
- 2026-09-26: phase 2 is done: the file seam, on both hosts.
  - A component wraps its control in `cambium::open_file(child, requested,
    filter, handler)`. When `requested` turns true, the tree files a
    `FileRequest` for that node, as `request_focus` files focus. The answer
    comes back to the node as a `FileEvent` holding each file's name, media
    type, modified time and bytes.
  - Rootstock gives a request to the host's `FileChooser` after dispatch and
    delivers answers at the start of the next frame. A host with no chooser
    answers with no files, and `AppCtx.files` lets a hook swap the chooser.
  - The web host's `WebFileChooser` keeps a hidden file input beside the
    canvas. A request clicks it, `change` reads the chosen files' bytes and
    answers, and `cancel` answers with none. The winit host's
    `DialogFileChooser` opens the platform dialog through
    `light-file-dialog`, which Graphshell already uses.
  - The scenario lane parks a request and answers it with `file <path>`,
    relative to the scenario, or `file cancel`. A `file` step with nothing
    waiting fails. The winit host's four `files` tests and three new scenario
    tests cover the seam.
  - In Mark's Chrome, through Claude in Chrome, the test page's "Open a file"
    button was pressed through the mirror from page script, with
    `navigator.userActivation.isActive` false, where the browser's rule is to
    show no chooser. The upload tool set `chrome-proof.txt` on the input, and
    the status line then read "File: chrome-proof.txt (42 bytes)", the
    file's size. A second request took `second-run.txt` (11 bytes) the same
    way. The Browser pane had shown the same path earlier with `hello.txt`
    set from script.
  - Both done-conditions are met, the first by the ruled instrument, which
    leaves the dialog itself unexercised.
- 2026-09-26: the D2 audit of this plan and the mere view's receipt
  (`support/doc-audit/d2/batch_23_one_tree_plan_and_receipt.md`) found five
  stale claims, four here and one in the receipt, and corrected them.
- 2026-09-26: phase 3 was assessed and ruled (§1). For its GPU timestamps,
  netrender `c8c09f16b` adds `NetrenderOptions::optional_features`, and
  genet `0cf4f30ba0f` forwards it into `RenderCore`'s boot, with a test that
  fails when the forwarding is removed. Mere repinned to both.
- 2026-09-26: mere `0418391f` on main carries genet `0cf4f30ba0f` and
  netrender `c8c09f16b`, with djinn's two interim copies until knot-editor
  moves.
- 2026-09-26: the scenario lane moved into rootstock
  (`cambium_rootstock::scenario`): `ScenarioLane<A, H>`, with a `LaneHost`
  half per host. The winit host keeps `LaneConfig` and wraps the lane over
  `NativeLane` under its old name, so Knot and mere-view build unchanged, and
  its nine scenario and four file tests pass. The web host gains
  `capture_into` and `PendingFrame`, an asynchronous frame readback, and
  `WebLane`, the browser's half. `ProducerContext` gains `core`, the host's
  `RenderCore`. graphshell-web takes the web host and rootstock on
  wasm-bindgen 0.2.127, with the CLI installed to match.
- 2026-09-27: the phase-3 worktree now selects published `netrender-vello`
  0.10.1 through netrender `9607d16f1` and Genet `92b249af5b2`, in both
  the root and standalone web manifests and locks. The locked wasm build
  (`CARGO_PROFILE_DEV_DEBUG=0`, `getrandom_backend="wasm_js"`) and
  wasm-bindgen 0.2.127 bundle generation passed. The receipt and artifact
  hashes are in `Code/testing/mere/mere-reservoir-web-vello-repin-build.json`.
  The clean main-line repin was pushed with approval as `815279cf`.
  The earlier blank 2,000-node timings are invalid.
- 2026-09-27: headed reruns exposed two phase-3 faults. The tree cached its
  incomplete first texture when the analytic layout was paused, preventing
  Vello's asynchronous buffer recovery. It now rasterizes continuously, as
  the presenter does. Setup also fitted the camera before buffered analytic
  positions reached the canvas view; `fit_to_content` now publishes those
  paused positions before finding bounds, with a regression test.
  `tree_recovery/p3_tree_health` draws the full 2,000-node graph and picks
  node 0 through the host pointer path. The full tree scenario then passed.
  A generic nonblank screenshot check had counted the heading as content;
  captures must also be inspected for the graph itself.
- 2026-09-27: pictograph culls offscreen underlay and node paint before
  scene lowering. Crossing edges survive even with both endpoints outside,
  and captions use their own bounds. Transform stacks and uncertain
  filter/shadow/fragment extents are retained. DOM layout and physics are
  unchanged. The GPU comparison produces identical pixels before and after
  culling and detects a deliberately removed crossing edge. All 254 unit
  tests and two existing GPU tests pass; the standalone wasm build passes.
  An event-driven producer still needs a renderer-completion signal before
  caching textures; the continuously rendered comparison page does not
  establish that contract.
  The analytic layout is paused on both pages: the timing windows are
  startup and steady rendering, not moving physics. Earlier receipts retain
  their misleading `moving`/`idle` labels as historical evidence.
- 2026-09-27: all four final headed scenarios pass, with every capture
  checked for visible graph content and both tree picks passing. Steady
  frame interval medians are 10.3 ms (presenter fixture), 24.1 ms (tree
  fixture), 1493.8 ms (presenter 2,000 nodes), and 1553.3 ms (tree 2,000
  nodes). These are debug, paused-layout measurements. The tree uses 2x
  device pixels while the presenter uses 1x, and the tree fixture has pacing
  outliers. The [full receipt](../testing/2026-09-27_graphshell_producer_receipt.md)
  records p95, dimensions, artifact hashes and limitations.
  Current main uses Mesquite for shared scenario execution. The follow-up
  integrates its lane with browser asynchronous captures and removes the
  unpublished rootstock runner.
  *Annotation, 2026-10-06 (S14 pass):* done. Merge `8c04c96a` (2026-09-27,
  "Merge main and run the browser proof through Mesquite") removed
  cambium-rootstock's `scenario` module, and the lane is Mesquite's `Lane`
  (`crates/cambium/mesquite`, unified in `8fce5365`).
  Mark has since approved Mesquite and proceeding to phase 4, with
  performance and live physics explicitly open (see the follow-up rulings).
- 2026-09-27: the first phase-4 slice adds shared canvas commands and a
  Cambium toolbar, captured dragging, graph-scoped keys, position-freezing
  pause and explicit arrangement restoration. Its headed scenario passes
  with named accessibility controls and inspected captures. Native tests
  cover held motion, membership changes and actor snapshot barriers.
  Equal DOM writes were invalidating retained layout every paused frame;
  skipping them reduces the 2,000-node paused median from 1603.3 to 107.6 ms
  in the instrumented dev comparison. Moving graphs remain slow: 512 nodes
  take 872 ms, primarily in DOM restyle/layout, while physics takes 2.6 ms.
  The live 2,000-node run timed out. The
  [controls and physics receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
  records the evidence and open gates. This does not complete phase 4.
- 2026-09-27: approved Genet motion commit `27d20d3f` is pushed and the
  worktree repins to it. All 298 targeted native tests, the locked wasm build
  and the headed controls scenario pass. Quiet live medians fall from
  141.7 to 96.0 ms (128 nodes) and 872.0 to 595.0 ms (512 nodes), with the
  remaining cost dominated by mutation/restyle. Genet `f2e2850f` separately
  removes repeated batch preparation; 302 tests and a deliberate failing
  work-count control support it. It is now pushed with approval and adopted.
  The controls/physics receipt records exact pins, artifact hashes and
  qualifications. After adoption, live medians are 73.8 ms at 128 nodes and
  251.4 ms at 512 nodes. The 2,000-node live run now completes with a visible
  graph at 1365.0 ms median; acceptable responsiveness remains open. All 403
  native tests, the locked wasm build and headed controls pass. Seiche's
  bounded elapsed-time core is implemented and tested; Canvas/browser host
  adoption remains pending, distinct from these rendering measurements.

- 2026-09-27: bounded Genet text-bounds publication is in progress. Root and
  standalone Graphshell web manifests select accepted Genet `7b48f94d7a7`,
  preserving NetRender `9607d16`, netrender-vello 0.10.1, wgpu 30 and pre.2.
  This takes the owner-verified text/inline-decoration fixes and retained
  motion/restyle source, not a new phase-3 policy. Portable dependency
  classification and focused consumer checks precede publication; see
  `design_docs/cambium_docs/technical_architecture/genet-compatibility.md`.
- 2026-09-28: the bounded publication gates above pass with published Genet
  `7b48f94d`: native host/Cambium/Sprigging all-target checks and standalone
  Graphshell web Wasm check. Locked metadata and deliberately faulty source
  controls qualify 1,524 native packages and 527 unfiltered web packages;
  only the accepted Genet revision and its exact documents/text edge change.
  Native wgpu 30.0.1 and web wgpu 30.0.0 retain their respective prior locks.
  NetRender/Vello and pre.2 remain unchanged. This is compile/source evidence;
  the headed work above remains open. Exact raw/provenance and interrupted
  checkout-preparation qualifications are linked from the compatibility note.
- 2026-09-28: integrate `reservoir-v2` with main `5ce144ff`, keeping accepted
  Genet `7b48f94d` (which includes the motion/restyle fixes). Mesquite retains
  both asynchronous browser readback and main's paired paint-envelope capture.
  All 403 native tests, the locked wasm build and headed controls pass on the
  combined tree. The three control captures are byte-identical to the
  inspected restyle-adoption captures. Vello compilation was concurrent, so
  this is functional evidence, not a new performance measurement. Logs,
  bundle and hashes are under `Code/testing/mere/reservoir-integration*`.
  The earlier worktree's held phase-3/4 changes are now integrated; product
  migration, large-graph responsiveness and elapsed-time host adoption remain
  open. The active primary checkout is left untouched for its current owner.
- 2026-09-29: the tree producer adopts bounded elapsed-time physics through
  Canvas and host timestamps. Configurable caps, discarded-time telemetry,
  idle/pause/reseed resets and visibility suspension now connect the tested
  Seiche core to the browser. All 266 Pictograph tests and 41 Rootstock tests
  pass; one pre-existing scroll-retention failure reproduces on unchanged
  Rootstock sources. The locked wasm build and both headed controls/elapsed
  scenarios pass, with visible graph captures. Concurrent builds exclude a
  performance comparison. Actual browser hide/show remains a headed gate.
  The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
  records the exact scope and evidence. Product migration and large-graph
  responsiveness remain open. The primary checkout is untouched.
- 2026-09-29: rebase the elapsed host slice onto published `ca2351b3`, keeping
  Genet `19c20687`, shared text boundaries and Apparatus/Mesquite work. All
  578 targeted library tests and 24 native scenario tests pass, including the
  formerly failing scroll check; the offline locked Wasm build passes. Add
  `tree.html?app=local` for the existing saved graph and a retained Title/Tags
  editor, with typed storage/retry and metadata-only canvas refresh. Headed
  verification is open: the test window is hidden and Windows Computer Use
  stopped because it could not verify its URL for policy enforcement. The
  mounted accessibility mirror is observed, but no save/reopen, visibility
  resume or new performance receipt is claimed. The controls receipt records
  this distinction; the primary checkout is untouched.
  Four focused storage/editor tests also pass, for 606 native tests in total.
  The reviewed source is published on `canvas-elapsed-host`; integration into
  main and worktree retirement await the headed gates.
- 2026-09-30: merge published main `da2940b6` into the elapsed-host branch at
  `650f8541`, preserving newer Gaz, Apparatus and generated-text work and
  adopting Genet `c5470fcb` with its published standalone web lock. All 578
  targeted library, 25 winit scenario and four local-editor tests pass, for
  607 native tests; the offline locked Wasm build passes. Headed saved edit
  and independent reopening retain the same session/member and edited
  metadata. Controls, elapsed physics and an initially hidden real background
  tab's first resume pass. Moving-graph hide/show and the intentional
  hidden-timing rejection remain pending. The new 512-node live diagnostic
  completes; 2,000 nodes are still running at this checkpoint. The
  [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
  records artifact hashes and measurements. Integration into main and
  retirement of this collision worktree remain pending; the shared primary
  checkout is untouched.
  *Annotation, 2026-10-06 (S14 pass):* the 2,000-node run had finished
  before this entry was committed (`108647cb`, 07:26): its raw receipt,
  `Code/testing/mere/scenarios/graphshell-web/tree_final_live_2000/`, written
  at 02:42, records state ok over 14 frames, interval p50 2217.3 ms and p95
  2486.2 ms. The controls receipt now records it. Integration landed:
  `650f8541` merged to main in `3270cac2` on 2026-10-01. The collision
  worktree, `Code/worktrees/mere-canvas-elapsed` at `74ee42ff`, still exists,
  so its retirement is still pending.
- 2026-09-30: Mark finds the current physics slightly laggy but acceptable,
  and points out that layout, physics law and scale can change the result.
  Record this separately from timings. A comparison must identify the
  layout/law, graph size and density, visible count and elapsed-step settings;
  the present generated-graph receipts do not settle all configurations.
- 2026-10-01: the physics panel lane (`tree-physics-panel`, from `d91a49f0`)
  builds the docked "Graph tools" region with its "Arrangement and physics"
  section, on both the fixture and `app=local` routes.
  - The typed Apply actions live in `graphshell::canvas_physics`, and the old
    page's form handlers now call them. Pictograph's
    `Canvas::set_physics_choice` gives one rebuild per apply, in source →
    overlays → law order.
  - The standalone web manifest's Genet pins follow root's `b1eb3af1`, which
    main `c6707958` had missed.
  - Native gates pass 261 (merge gate), 228 (graphshell `web` lib) and 260
    (pictograph `canvas` lib), and the locked wasm build passes.
  - Headed on the tree: the 11 per-law receipts, profiles and add pass, and
    Springs and profiles pass on `app=local`. A positive control fails as it
    should.
  - `p4_tree_physics_drag` fails its one-frame release-window threshold
    intermittently (Stress 21–31 against ≤ 20; Anneal 61–292 against ≤ 60).
    Its reclaim and hold assertions pass. This is returned as a fork.
  - The region's width at narrow viewports is not ruled.
  - The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
    records the evidence. The remote board scenarios belong to the separate
    remote-session slice.
- 2026-10-01: following the "Measure in physics steps" ruling, the tree lane
  records the release window on the first frame after a release that executed
  physics steps (`drag-return-step`, `drag-return-steps`).
  `p4_tree_physics_drag` reads it, with the thresholds unchanged.
  - In three headed runs, ten laws read 0.0–8.2 px after one or two steps.
  - Anneal read 255.2, 190.8 and 0.0 px after one step, against ≤ 60.
  - In the 190.8 px run the node sat 48.1 px from where it was pressed, at
    zoom 1.00. Anneal moves a body at most 80 px a tick, so this is a
    snap-back, not walk noise. A likely cause is that Anneal writes every
    body's translation, the held one included, so the body never follows
    the drag. This is not isolated.
  - Per the ruling, the threshold is unchanged and the case returns to Mark
    as a fork. A positive control fails the new observation as it should.
    The receipt doc records the numbers.
- 2026-10-01: Mark ruled "Seiche: skip non-dynamic bodies".
  - `Anneal::apply` now leaves non-dynamic (kinematic) bodies unmoved; they
    still count in the energy as neighbours. An audit of every law,
    overlay and Hold found no other position write. The rest use
    `add_force`, which rapier 0.33 applies only to dynamic bodies, or
    `set_linvel`, which it ignores on kinematic position-based ones.
  - Outside that audit, `CouplingForce`'s FlowAdvect response
    (`coupling_force.rs`) has the same `set_translation` pattern. It is
    noted, not changed.
  - A new seiche test pins a node under Anneal. Without the fix it fails,
    with the pinned body at (58.4, 93.9) against a target of (400, −300).
    With the fix it passes.
  - Seiche passes 98 tests with default features and 94 without; pictograph
    `canvas` passes 260. The wasm bundle builds.
  - `p4_tree_physics_drag` passes headed three runs in a row. Anneal's
    first-step reading is 0.7, 0.0 and 0.0 px, and the 300-frame checks pass.
    `p4_tree_physics_anneal` passes.
- 2026-10-01: the three follow-ups ruled at `ea604bf4` are carried out.
  - **FlowAdvect.** `CouplingForce` FlowAdvect skips non-dynamic bodies. Its
    new test failed without the fix, holding the pinned body at (120, 0)
    against a target of (−200, 50), and passes with it. Seiche passes 99
    tests with default features and 95 without.
  - **Narrow viewports.** `TOOLS_DOCK_MIN_WIDTH` in `web_tree.rs` is 900
    logical px: the 300 px region plus a 600 px minimum canvas. A centred
    node dragged 220 px needs 476 px, and the fitted fixture spans about
    320 px.
    - Below that width the region is not rendered. A "Graph tools" toggle
      with `aria-expanded` appears at the end of the Graph controls row and
      opens the region as an absolute overlay over the canvas row.
    - *Reading, not ruled:* the toggle sits in the controls row rather than
      over the canvas, so the open overlay never covers its own toggle.
  - **Storage line.** On `app=local` the storage line now leads the Graph
    tools region as a status named "Storage: …". The floating line on the
    canvas shows only save feedback.
  - **Structure.** `physics.rs` now returns only its section, and
    `web_tree.rs` builds the region around it, so another section can join.
  - **Results.** Graphshell's `web` lib passes 228 tests and the wasm build
    passes. Headed, `p4_tree_physics_springs`, `_springs` on `app=local` and
    the new `p4_tree_tools_narrow` (700 px window) pass, all inspected
    whole-frame. The mirror lists button "Graph tools" (`aria-expanded`
    false) and the "Storage" status.
- 2026-10-01: the remote-session slice (`tree-remote-session`, from
  `f7c5873c`, with main `f4e4726c` merged) carries out the remote-session
  rulings.
  - The op sequencing lives in `graphshell_client::remote`, and the old
    page delegates to it. The WebRTC transport is shared by both pages.
    Pictograph draws the board as a `BoardScene`. The tree's one canvas leaf
    paints it with 24 px margins while "Remote mount" is pressed.
  - Graph tools gains a "Remote session" section: the switch, the
    active-session line, one described button per intent, the draft form,
    the link's Disconnect, Reconnect and Nudge host, and the action status.
  - Cambium's web-host mirror now writes `aria-description`, which it had
    dropped.
  - Native gates pass: graphshell-client 59, the merge gate 262. The wasm
    build passes.
  - Headed, the old page passes the three originals unchanged. The tree
    copies pass on the fixture route and on `app=local`, and a positive
    control fails.
  - Two forks are open. On `app=local` at 1400 by 900 the region overflows
    the window, clipping the Link group and the action status. The live
    fixture has no intent with inputs, so the draft form is untested headed.
  - The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
    records the evidence.
- 2026-10-02: the overflow, draft-proof and card-label rulings
  (`bbc89994`) are carried out on `tree-remote-session`.
  - Graph tools sections are Cambium disclosures in a region that scrolls.
    Cambium's closed panels now also carry `display: none`, because Genet's
    UA sheet has no `[hidden]` rule.
  - `LiveEndpoint` has a bounded "Append a coloured card". An incomplete
    draft is refused locally rather than failing the session.
  - Pictograph paints card titles in the page's font. The section lists
    them, and the tree's board frames the cards' edges. The old page keeps
    its framing.
  - Gates pass: the merge gate 263, Cambium 244, pictograph 269. The wasm
    build passes.
  - Headed, `p4_tree_remote_draft` and the three tree copies pass on both
    routes. The old page's three originals pass with the third action.
  - Open as a fork: one mirror node per board card. Cambium's leaf
    semantics have no children.
  - The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
    records the evidence.
- 2026-10-02: the per-card mirror and collapsed-remote rulings
  (`7248e74d`) are carried out on `tree-remote-session`.
  - `TextureProducer::semantics` lets a producer name its slot and the
    things drawn in it. Both the web mirror and the winit a11y host write
    them, and `Accessibility::sync` takes the producer registry.
  - The board's slot is list "Remote board · N cards", with one named item
    per card at its painted rectangle. Mirror boxes match the captures on
    both routes.
  - Remote session starts closed and opens with the link.
  - Gates pass: Cambium 244, winit host 153, merge gate 263. The wasm build
    passes.
  - Headed, the four tree scenarios and the new `p4_tree_remote_absent`
    pass on both routes, and the old page's three pass. A positive control
    fails.
  - The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
    records the evidence.
- 2026-10-02: the "Genet gaps" ruling is carried out on `genet-repin`
  (from main `f02d9d35`).
  - **Repin** (`7a0950da`). Every Genet row moves from `b1eb3af1971` to
    `bd3e8861b29`: 28 in the root manifest and 7 in the standalone web
    manifest. A whole-tree sweep found no other manifest naming the old
    SHA; `genet_web_smoke` keeps its documented older pin. Both locks move
    their Genet entries and nothing else.
  - Each gate crate's graph resolves one Genet revision. The workspace's
    second, Genet `34626a6c` through djinn's `knot-editor` `855cb75d` pin,
    predates this change.
  - **`[hidden]`** (`1182f642`). Cambium's closed panels carry `hidden`
    alone again. New rootstock layout tests check that a closed disclosure,
    accordion panel or tree group has no box. At the old pin they failed
    without the inline style (18 px boxes), and at the new one they pass.
  - **Titles** (`de115df8`). No style change was needed. A pictograph test
    checks titles are centred in their cards; it failed at the old pin with
    a 6 px left gap.
  - §2's `text-align` finding was a nowrap case, and `6fca091dc26` fixes
    it (annotated there).
  - Gates pass:
    - Cambium 245, rootstock 55, web host 8, winit host 153;
    - pictograph `canvas` 270, graphshell `web` 230, graphshell-client 59,
      and the merge gate 266;
    - `cargo check --workspace --all-targets`.

    The wasm build passes.
  - Headed on `tree.html`, five scenarios pass: `p4_tree_physics_springs`,
    `p4_tree_tools_narrow` (700 px), `p4_tree_remote_absent`, and over the
    fixture `p4_tree_c4b1_live_board` and `p4_tree_remote_draft`.
  - The closed-section capture is pixel-identical to the tree-remote lane's.
    The board captures differ from it only in the title band, and the titles
    are centred.
  - Not run: the `app=local` route, and the old page's originals.
  - The [controls receipt](../testing/2026-09-27_graphshell_controls_physics_receipt.md)
    records the evidence.

- 2026-10-07: source/history refresh at Mere `cd3ebf26d`. Dated the status and
  added the current cutover/remaining-gates map, preserving all historical
  native/browser and performance limits. The reservoir panel remains V2b step
  5; shared editor and dynamics status is linked to its owning plans. No new
  runtime verification or phase-completion claim.
- 2026-10-08: bounded Genet livery lookup repair adopted at published Genet
  `15713014e2e23b887360471552f75f60684f5384`. All 38 current Genet rows move
  from `e84f9c7f9aec23320c539784961d1465f8a53a9b` across the root, standalone
  Graphshell web, and Distillery probe manifests; root and probe locks are
  refreshed. Root still resolves the separately pinned Knot Genet `965b64e`
  `fleece` and `layout-dom-api` packages as a second source; the optional Knot
  pin is intentionally unchanged under ruling 58.
  Locked Cambium library (251), Mesquite library (19), and rootstock
  `no-default-features,image-decode` checks pass. This is dependency adoption
  evidence, not a headed/browser or feature-completion claim; command logs are
  under `/Users/markik/Code/testing/genet-boundary-20261008.R85RvP/evidence/`.
  All three commands use the isolated worktree based on `daf08a3f` and
  `CARGO_TARGET_DIR=/Users/markik/Code/testing/genet-boundary-20261008.R85RvP/mere-target`:
  `cargo test --locked -p cambium --lib`,
  `cargo test --locked -p mesquite --lib`, and
  `cargo check --locked -p cambium-rootstock --no-default-features --features image-decode`.
  Final gates rerun after unrelated resolver-edge cleanup; no probe compile or
  Graphshell web build is claimed. Genet's wider gate retains three failures
  reproduced on its untouched base (two float cases and one table-glyph case);
  consumer checks do not establish a wholly green upstream suite.

- 2026-10-09: full cutover resumed with explicit parallel source ownership and
  one low-priority build queue in the existing cache. The current scenario
  inventory reconciles 119 original scripts and 16 archived names to four
  byte-identical imported scripts, with two historical presenter-only scripts
  separated from live acceptance. New tests cover the remaining retained
  detail, find, projection, scene and practice controls. Reopen scenarios keep
  their same-origin persistence prerequisites rather than using empty profiles.
  Initial detail-action RED exposes only its missing typed owner API; the
  implemented owner passes eight native tests, including saved facet/relation
  reopening and refused-write retry. Corrected canvas-cancel RED reaches six
  missing-method references, and saved-scene RED reaches its missing owner
  module. Their implementations still await GREEN. These bounded tests do not
  close the Wasm, five-page, live-inventory or performance gates.

- 2026-10-09 continued: canvas cancellation passes its two native regressions. Projection-owner RED reaches only the missing module; scene GREEN awaits a corrected typed diagnostic formatter and repeat. The standalone full web development graph resolves 683 packages with no selected Mere/Genet git duplicates. The live inventory is 126 scripts plus two historical comparisons; intentional negative diagnostics retain explicit source-bound expected predicates rather than being silently removed or counted as acceptance passes. Fresh Wasm, all five public pages and the complete live receipt wall remain open.

## Native retained-content continuation — 2026-10-13

**Status:** implemented; native and browser qualification recorded. Turnstone
`d5d4737` adopts the published Mere `0d1dc1927859ff1aa2a43dd88a04391bc1086fd8`
baseline with normal locked dependency resolution and no source overrides.
This continuation follows Mark's native-stack ruling in the
[suite census](../../2026-08-22_turnstone_suite_composition_and_capability_census.md).

### Target and done-conditions

Compose Pelt/Genet in Graphshell's existing Frisket Workbench; lift selected-clip
presentation into shared document lanes and adopt that factory in Turnstone.
Done means retained source bytes reach real document sessions and the host's
existing device, selected clips survive store reopening, hidden/retired views
refuse input, and browser captures show actual content after an asynchronous
source read. Consumer qualification must distinguish source overrides from a
published dependency pin.

### Findings

- Shared source identity and clip verification belong in document lanes;
  admission, custody, arrangements and link decisions stay with the application.
- Frisket rebuilds the previous fill callback. A callback that reads mutable
  present state can reconstruct the previous view with the wrong concrete type.
  The Workbench now captures immutable appearance snapshots before building it.
- A custom leaf must preserve `display:block` when replacing its inline style;
  otherwise the retained source exists but its paint box is empty.
- Producer keys must fit rootstock's 62-bit namespace. The browser scenario
  now asserts actual registrations, not only source sessions. Earlier runs
  that passed while the producer was absent are preserved as rejected evidence.
- Browser-hosted document sessions need the same held font supply as the chrome.
  The native factory accepts explicit font bytes and optional display-family
  selection; Graphshell's browser reading view selects its bundled Roboto.
  This presentation choice leaves the owner-held source unchanged. It does not
  qualify original font fidelity or arbitrary web-font acquisition.
- The core source inventory refuses ambient file/network acquisition. A restored
  file fallback is retained as a failing negative control in the receipt.

### Progress and remaining gates

The shared native factory, retained clip session, portable content host/input
view/producer, browser store admission and Workbench mounting are implemented.
Evidence is under `ports/graphshell/docs/receipts/retained_content_20261012/`.
The Windows consumer check passes with one Mere source and one Genet revision;
its clip (10) and pane-store (5) tests pass. Shared document-canvas tests (85),
Graphshell retained-content tests (9), shared clip tests (2), merged Moot
resource tests (42), and admitted native resource-mount tests (8) pass.
Browser captures show actual retained HTML, selected UTF-8 text and voice-note
controls; reopening reads the saved IndexedDB store without reimporting clips.
Chrome accepts actual keyboard and pointer playback from the retained WAV;
pointer Stop is observed after the host frame settles. The scripted gesture-free
attempt is correctly refused. The shared accessibility mirror cannot scroll away
from the canvas or recursively enter an FnMut focus trampoline. Retained body
clicks also stop at the content boundary before an enclosing tile can replace
the input target. Physical audibility is not claimed.
The latest Turnstone draft `7d62a5a` supplies Tabard themes and Interface fonts
to the shared pane frame. That adoption retains its existing acceptance scope.
This bounded native content path does not close physical assistive-technology,
protected personal custody, all media adapters or cross-platform acceptance.
Graphshell's selected-clip export adapter now serializes the verified selected
body as a PortableClip package and hands it to the browser with the package MIME
and content-derived filename. Thirteen native retained-content/export tests pass,
including exact owner/session/source guards and usable short-panel allocation.
A trusted browser click reaches the handoff status; a completed browser download
path remains unobserved. See the source-bound continuation in
`ports/graphshell/docs/receipts/retained_content_20261012/clip-export-browser/`.

Turnstone A4/S3 now implements in-house Component bodies in the shared retained
forest and stages native producer images at their exact appearance apertures.
Source-first withdrawal parks the actual runner until remount. Eleven shared
controls cover eager removal, GC, focus/state continuity, owner teardown,
foreign-DOM refusal and explicit stock-tab focus requests. The focused Genet
19947 receipt remains distinct from final supplier qualification. Native headed
composition qualification is in progress. Physical
input/accessibility, monitor DPI transitions and performance remain separate
gates. External engine builds and simultaneous engine loading remain separately
qualified consumers.

The final b6/6a shared model gates pass on the published Genet and Netrender
suppliers. The Pelt scheduling continuation observes actual pending work and
session identities while preserving prepared input/a11y routes; all 25 pool
tests pass. Future timer completion is independent of the script-visible settled
handshake. Hosts retain deadline/wake authority and admit redraws through the
current visible placement. Turnstone's actual headed acceptance and final Mere
pin remain distinct from these shared controls.

### Shared browsing presenter candidate (2026-10-13)

At Mark's request, `data-presenter="browse"` opts into a quieter shared viewer
presentation. The existing inspector profile is unchanged. Browsing keeps one
44-pixel Graph menu trigger backed by the same command catalogue as right-click,
Ctrl/Command+K and Shift+F10. Project details precede disclosed selection
management; history starts closed, with a command to reveal it and a separate
raw checkpoint-source disclosure. These choices change presentation only:
source authority, selection identity through folds and graph gestures stay shared.

The candidate compiles for the no-default-features Wasm viewer. It is not yet
browser-qualified or adopted by the site. Required acceptance includes physical
context-menu and keyboard input, focus restoration, touch input at a compact
viewport, optional history, and native no-script site disclosures. Measure the
new package against the existing delivery caps and coordinate a fresh bounded
runtime lease before headed acceptance; existing package evidence does not
qualify this runtime. Keep the deployed row until those checks pass.

The first complete desktop browsing case and native no-script/mobile/tablet
reading cases passed locally. Touch Fit also passed, including geometry and
focus restoration. The subsequent Graph tools tap failed: the test used the
accessibility mirror's full item bounds without intersecting the painted menu
clip. The item was below that clip, so this does not establish a shared-host
pointer dispatch failure. R3/R4 failed receipts remain preserved; neither is
full qualification.

The browsing menu continuation uses bounded two-command pages, with Previous
and Next commands, Close before the list, and the existing shared search and
keyboard selection controlling the same page. It does not introduce app-specific
event dispatch or change the browser host. The next touch gate must page through
the real visible controls, activate Graph tools, dismiss by touch, and restore
focus; every tap target must be inside both the browser viewport and painted
menu clip. A rebuilt package and fresh size/browser checks are required. The
small project faces at the fitted scale remain a separate visual limitation.

R5 on provider `56859e30292405e682f28f3841b76e4f141acf73` (including main's
Genet 561 font/recovery integration) passes the full headed site suite and all
13 viewer cases: context/keyboard commands, page navigation, emulated-touch
Fit/Next/Previous/tools/Close with focus return and clipped visible targets,
no-script/static reading and startup/failure fallbacks. Five command tests and
39 site library tests pass; rebuilt Wasm gzip6 is 3,280,020 bytes against
3,407,872. The source-bound seal is
`6114468a012717f72ab13f652bd43a085ece8f98498bc81b4afec7e06a6c9e61`.
The site records package/capture/closure evidence under
`docs/receipts/site_browsing_20261013/`. Emulated touch does not qualify physical
phones, performance or another renderer. The earlier R3/R4 failures remain
failed runs, with touch Fit distinguished from the later off-clip tools tap.
Publication is separate from this acceptance; a rebuilt later source needs its
own package qualification. Fitted project faces remain a visual limitation.
