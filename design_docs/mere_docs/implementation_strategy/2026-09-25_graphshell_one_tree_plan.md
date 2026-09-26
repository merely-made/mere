# Graphshell on one Cambium tree

**Date:** 2026-09-25
**Status:** plan, ruled 2026-09-25 (reservoir plan §7 items 39 and 40).
Nothing implemented. Phase 1 is first.
**Scope:** Graphshell's browser page becomes one retained Cambium tree. Its
HTML controls become Cambium components, its display-only Cambium chrome
joins them, pictograph's canvas renders into the tree as a texture producer,
and Cambium's web host takes over `web_gpu.rs`'s compositing. The reservoir
plan's V2b then places the mere view in that tree.

**Related:**
- [Reservoir plan](2026-09-23_reservoir_plan.md), V2b, whose Graphshell step
  waits on this plan.
- `crates/cambium/cambium-genet-web-host/src/a11y.rs`, the module doc that
  records the web host's accessibility gap.
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
    lines) holds about 100 controls, driven by `web.rs` (2,248 lines):
    - a mounted-sessions header;
    - a graph-controls bar (select, edit, pan, zoom);
    - an aside of six sections: browser history, add an object, the
      projection editor's seven tabs, find and arrange, scene and transfer,
      and the node detail.
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
  - Pelt's desktop workspace viewer and the scrying engine use it.
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
- **Five pages mount the view.** `index.html`, `embed.html`, `practice.html`,
  `practice-embed.html` and `co_op.html` all load it through `loader.js`.

## 3. Target shape

- **One mount per page.** Each page puts Graphshell's application on the page
  canvas with a single `mount` call.
- **One tree.** It holds the graph controls, the six sections and the chrome
  as Cambium components.
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
