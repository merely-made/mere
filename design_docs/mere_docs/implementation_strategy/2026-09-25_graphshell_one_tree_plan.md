# Graphshell on one Cambium tree

**Date:** 2026-09-25
**Status:** in progress, ruled 2026-09-25 (reservoir plan §7 items 39 and
40). Phases 1 and 2, accessibility in the browser and the file seam, were
done on 2026-09-26. Phase 3 has headed correctness receipts. On 2026-09-27
Mark approved proceeding to phase 4, with stack performance and live physics
explicitly open.
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
- [Platform boundary and repository topology plan](2026-09-02_platform_boundary_and_repository_topology_plan.md),
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
  alternative recorded both for later.
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

## 5. Stop rules

- Nothing a screen reader reaches today is lost: a control moves only once
  phase 1 projects its kind.
- The canvas's rendering, physics and picking stay pictograph's. This plan
  moves where the canvas is hosted, not what it draws.
- No Graphshell-only UI pieces. What Cambium lacks is added to Cambium.

## 6. Progress

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
