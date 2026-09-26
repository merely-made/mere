# Graphshell on one Cambium tree

**Date:** 2026-09-25
**Status:** in progress, ruled 2026-09-25 (reservoir plan §7 items 39 and
40). Phases 1 and 2, accessibility in the browser and the file seam, were
done on 2026-09-26; phase 3, the canvas as a producer, is next.
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
