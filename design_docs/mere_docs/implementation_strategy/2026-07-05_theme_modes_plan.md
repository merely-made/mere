# Theme Modes Plan (light / dark / high-contrast / custom)

**Date**: 2026-07-05
**Status**: T1–T5 implemented 2026-07-05 (see Progress; T5 shipped the declarative lane, rhai
graduation open). From Mark's theme-model decision (2026-07-05), unblocking the W3C adoption
plan's P3 host half.

**2026-09-13 extension:** Tabard's Lagrange artifact exporter is implemented;
native reader adapters and stock-client visual acceptance remain open below. The July
receipts describe their original host; they do not establish current Knot or
Turnstone settings, foreign exports or persistence.

**2026-10-09 design direction:** the [cross-app design language record](../../2026-08-23_projection_scenes_and_graph_native_platform.md#9-configurable-visual-and-interaction-language-2026-10-09)
records configurable themes with varied defaults, Emblem/Pictograph iconography,
and user-selected installed fonts with openly licensed bundled defaults. It
does not establish implementation or consumer adoption of those preferences.

**Related**: `repos/genet/docs/2026-07-05_w3c_mechanism_adoption_plan.md` (P3 engine half landed:
`IncrementalLayout::set_prefers_color_scheme`), `repos/tincture` *(historical citation)* <!-- doc-audit: historical-path --> (tinct seed-to-palette
derivation), `crates/meerkat/src/theme_sheets.rs` *(historical citation)* <!-- doc-audit: historical-path --> + `theme_edit.rs` (current sheet baking +
switch path).

## Design-language clarification (2026-10-09; research)

The [design language §9.8](../../2026-08-23_projection_scenes_and_graph_native_platform.md#98-voice-clarifications-and-research-boundaries-2026-10-09)
records Mark's agreement that theme values need a system defining how they are
used when data encoding, selection and activity compete. Fill, outline and
badge assignments are suggestions, not defaults chosen here. Research semantic
roles and configurable composition alongside projection style authoring; token
interchange alone does not decide these meanings.

Mark asks for granular font roles corresponding to the defaults, so a person
can choose installed fonts while retaining useful heading, size and text
patterns. Firefox's [font preferences](https://support.mozilla.org/en-US/kb/change-fonts-and-colors-websites-use)
are a checked precedent for serif/sans-serif/monospace choices, sizes and minimum
size. Those generic families do not themselves define Mere's UI, reading,
heading or label roles. The role schema, override precedence and remote/frozen
fidelity remain open; measurements must stay coherent with realized fonts.
This note does not change adoption order or claim controls in every consumer.

## Application adoption (2026-10-09)

**Status:** in progress. Mark selected this order: Pelt, Graphshell, Knot,
Turnstone, Woodshed, Redshank, Signalman. Isocosm is excluded while its own
migration proceeds. This is the implementation priority; the owner subsequently
asked to push integrations as they qualify. Implementation and discovery can
overlap, with each slice qualified before publication. Existing Woodshed adoption receives a
reconciliation pass rather than a second editor implementation.

### Shared foundation and current origin

Reconcile the published Tabard library/workshop, Cambium title bar and native
scene adapter with current Mere origin before consumer edits. Preserve newer
fetch, retained session, graph and domain changes. Keep primary checkouts'
unpublished commits intact, and fetch again before integration/push. Done when
the current resolved graph keeps one shared source identity, scoped foundation
checks pass and the port boundary remains one way. Historical native receipts
describe their original binaries; they do not qualify the new integration.

### Pelt durable application appearance

Add a reachable application appearance workflow using the shared workshop,
library and title-bar slots. Store application selection separately from
authored definitions, use normal host text/file/close seams, and preserve
controller/session and loaded-document authority. Custom authored stylesheets
remain authored stylesheets rather than being silently reduced to tokens.
Done when saved selection and mode survive a fresh process, editor preview
does not apply implicitly, invalid/corrupt saves remain recoverable, and native
wide/narrow rendering and same-window resize have actual nonblank captures.

### Remaining consumers in order

Graphshell qualifies its actual application presentation rather than mistaking
the native-messaging relay for a window. Its embedded projections inherit the
composing host's roles. Knot keeps writing/measure/font preferences and document
authority; Turnstone keeps settings, surface routing and reader precedence.
Woodshed preserves its accepted legacy themes and persona-owned selection.
Redshank and Signalman retain their product/domain contracts. Every host uses
shared authoring and theme resolution; no consumer grows another palette
engine, draft model, file format, renderer or caption-command queue. Done for
each when its reachable controls, durable selection, failure handling and
applicable rendered acceptance pass against its published dependency set.

### Findings and progress — 2026-10-09

- Mere's primary main has unpublished local commits and is behind the fetched
  origin. It was preserved. A separate worktree starts at origin
  `6708bfcd0`; the seven published Tabard commits were replayed cleanly there,
  retaining the intervening upstream changes and excluding unrelated local
  ambient-design history.
- The moved Tabard/workshop manifests require two lockfile references to adopt
  current Genet `15713014` rather than the older `965b64e2` used in historical
  acceptance. Full metadata resolves successfully with the current Genet source;
  the port and Graphshell web boundary checker passes.
- The shared theme resolver keeps the requested choice intact, reports missing
  theme/mode fallback, and distinguishes exact authored CSS from a derived
  palette. The workshop exposes validated definition intake and a saved-choice
  accessor that rejects unsaved or staged edits. Nine focused contract tests
  cover these embedding seams.
- The native host exposes an additive tool-window entry over a caller-owned
  event loop and existing render core. Its normal input, accessibility and
  close-policy pipeline remains the owner of the tool window's behavior.
- Hosts also supply transient protected export paths to the existing workshop
  writer. The shared identity comparison protects application settings and open
  documents before both creation and replacement, including aliases. This
  avoids each app duplicating export path normalization. All 17 interchange
  tests pass, including the two new host-file boundary regressions; this
  additive API was pushed and remote-verified as
  `d5679eba4aa1bc5e1a241c5df63d13490a5f46a7`.
- The existing choice file store gains an additive strict loader. Missing files
  keep the existing default, while malformed/non-UTF-8 preferences surface an
  error without changing bytes; legacy lines and authored custom-mode JSON keep
  their existing interpretation. All seven focused choice-store tests pass.
  The compatibility loader remains unchanged for existing consumers. Pelt and
  Signalman use this shared operation rather than duplicating file validation.
- Editing and saving the currently selected theme under the same identity must
  not bypass explicit application Apply. Each native adapter holds the applied
  presentation while authoring updates the library, refreshing it only after
  successful application selection or Apply; fresh launches resolve the saved
  definition normally. Mounted regression qualification is in progress.
- Current Cambium fields render semantic containers, so the workshop's old
  tag-based input/textarea CSS collapsed an empty stylesheet editor to zero
  height. Shared field classes restore its geometry. All 299 scoped Tabard,
  workshop, desktop and native-host tests pass against the fetched origin,
  including all six desktop usability tests. Two host documentation examples
  remain intentionally ignored. Pelt application/native qualification follows.
- A subsequent fetch found origin `7c0c12008`; its design-context and app
  composition documentation merged cleanly before consumer qualification.
- Origin advanced again to `4d8bd7037` with the Conatus tenant and its plan.
  The worktree fast-forwarded without changing application sources; the exact
  seven local application dependency rows were preserved over the upstream
  lockfile. Locked full metadata and the port/web boundary checker pass on
  that baseline.
- Shared foundation and current-source lock repair were pushed directly to
  origin main as `3567c8e937b0e2ec8e2c670cfaf4289d4403d216`. A clean committed
  snapshot independently passes locked full metadata and the port/web boundary
  checker. Consumer work remains separate from this publication.
- Canvas now accepts an already resolved Tinct palette for retained derived
  faces, reusing its existing role mapping and companion mixing. This lets web
  computed CSS roles and canonical Tabard profiles recolor existing face caches
  without changing deterministic face bytes or semantic node-state accents.
  All seven focused palette tests pass, including selected-mode and alpha
  preservation plus unchanged legacy seed mapping. This addition is ready for
  publication independently of the application adapters and was pushed as
  `7a2a851fb0134a3cab7e1c4161c1c46f371d829a`.
- Pelt's source adapter reaches the shared workshop through its real Theme
  drawer and mounts a tool window over the existing render core. Application
  choice and shared definitions have independent durable paths. Focused tests
  and fresh native receipts are in progress; this is not yet an acceptance claim.
- Pelt's 63 desktop tests pass. Its first current-source LaunchServices run
  times out before initial presentation with repeated `Occluded` acquisitions,
  so no editor capture or acceptance is claimed. The retained failed receipt
  motivates bounded first-presentation retries, initial activation after the
  accessibility reveal, and composition of the editor's existing idle policy
  into the parent event loop. That lifecycle correction is being requalified.
- The corrected Pelt source passes all 69 desktop tests, including application
  export-path protection and saving an edited active theme without implicitly
  applying it. Native presentation qualification remains open.
- The second Pelt LaunchServices run reaches the embedded workshop and captures
  Light and Dark but exceeds its scenario deadline: the driver did not owe the
  redraw that follows its first successful browser presentation. Failed receipts
  remain retained while that continuation is corrected. The shared launcher
  now recognizes the canonical macOS `/private` spelling of its unique copied
  executable during timeout cleanup; focused alias/argument/non-owned-process
  checks pass. This cleanup change cannot make a failed receipt pass.
- The final Pelt desktop suite passes all 70 tests, including real nested wheel
  delivery, scroll-aware accessibility bounds and Scroll Into View routing. The
  library drawer and builtin radio state are qualified in the production adapter.
- Fresh Pelt run 05 reaches the actual editor and captures all four modes, then
  stalls in the shared texture readback. Its retained process sample identifies
  `RenderCore::read_rgba8_texture` waiting indefinitely in Metal `Device.poll`.
  This is a shared capture issue; partial images do not qualify the workflow.
  Genet's owned pending readback is published and remote-verified as
  `7422e90613f9017e5bb790e3acb48f61776b2eda`. All four renderer/readback
  regressions pass on the AMD Radeon Pro Vega 56 / Metal discrete adapter,
  including padded rows, independent queued captures and cancellation.
  Rootstock starts the copy of the original presented frame; Mesquite polls
  completion on later host turns with a five-second deadline, retaining the
  original request and presentation identity. Synchronous callers wait only
  for their own copy submission with a bounded timeout. Current Genet's added
  form metadata notifications route through the owning Rootstock window.
  Mere adopts the whole active Genet host/render/DOM family and its required
  Boa/ICU lock update; its unchanged portable Knot dependency still carries
  older Fleece/layout API packages, without a second host or renderer.
  All 266 shared native tests pass (166 native host, 73 Rootstock, 27
  Mesquite), with four documentation examples intentionally ignored. All 19
  workshop interchange tests pass, including protection of entire owned
  directories, future generated files, symlink aliases and late replacement
  decisions. Directory protection uses the existing shared path identity
  handling and component boundaries; it does not enter authored definitions.
  A clean staged snapshot independently passes locked offline full metadata;
  the port and Graphshell web boundary checker passes. Fresh native app
  acceptance remains open.
- Another origin fetch found `421818710`, including the Moot capsule host for
  Graphshell. The worktree fast-forwarded and restored the application changes
  from a preserved stash. Both capsule and appearance module/state/frame seams
  remain; the tree stylesheet retains the capsule sheet on later appearance
  refreshes. Reconciled browser and metadata gates are in progress.
- Knot's initial adapter replaced the supplied sheet on unrelated dispatch,
  breaking existing scroll and focus invariants. Explicit appearance/editor
  refresh transitions repair that regression. Its full library now passes
  298 tests, with one existing diagnostic ignored; all 17 appearance tests,
  four preference tests, eight title-bar tests and eight existing scenario
  tests pass, as do all three bundled-font tests (338 tests total). Native
  qualification against the newly published capture stack remains open.
- Graphshell's application routes are the web full app and tree `app=local`;
  its native-messaging executable is a relay, not a settings window. The web
  adapter now consumes shared Tabard intake with separate application appearance
  persistence. Its six portable appearance regressions, locked default WASM
  build, viewer-only check and optional applet check pass on the current shared
  stack. Actual Chrome 152 / AMD Radeon Pro Vega 56 browser acceptance passes
  four canonical modes, exact authored CSS, custom mode intake, corrupt-input
  retention, fresh reload, embedded host/title/storage ownership, saved-tree
  mode controls and narrow layout. All 23 retained captures were inspected;
  browser/page/GPU error gates are clear and product storage remains unchanged
  by appearance selection. Evidence is retained under
  `tabard-app-receipts/2026-10-09/graphshell/visual-run-02.json` and
  `graphshell/main-controls-02/browser-report.json` in the family workspace.
  Embedded roots inherit composing-host roles without reading or writing the
  application preference. Optional Moot capsule lifecycle and styling survive
  later appearance refreshes.
- Native acceptance is paused after Pelt fresh runs 07 and 08 triggered GPU
  resets and WindowServer watchdog failures. The 01:46:41 October 10 kernel
  report identifies the fresh-08 Pelt process and its stalled Metal compute
  queue. Boot time remained October 2: this was a graphics-session reset, not
  a full reboot. Fresh-08 reproduces with sleep prevention active and records
  zero captures; no native application is visually qualified by this run.
  Renderer compute isolation and repair must precede another native attempt.
- October 10 renderer isolation: both the BBS Moot and canonical-domain lanes
  report their last headed browser checks ended near 00:12 Eastern, with no
  newly launched headed GPU runs at the later reset timestamps. Earlier demo
  pages may have remained open; those reports do not establish a shader cause.
  The canonical-domain lane also records later headless device-loss failures.
  Shared Vello `491c376c` retains the existing Radeon coarse fix, enables normal
  shader runtime checks and workgroup initialization, and validates fine
  command framing, forward jumps, clip depth and fill segment spans. All 31
  deliberate command dispatches, two coarse regressions, ten general renderer
  regressions, allocation recovery and repeated clip composition pass on the
  actual Radeon Pro Vega 56 / Metal adapter. The test image comparison now
  uses independent GPU bytes instead of comparing the CPU image with itself.
  The actual workshop Reader/CSS previews also pass 60 owned readbacks over
  four modes, authored CSS, wide/narrow dimensions and interleaved shared-core
  draws. Copy completion is at most 498 ms; exact background/text checks pass.
  Settled and isolated Reader comparisons permit at most 64 pixels differing
  by one byte per channel, with a CPU regression enforcing that bound.
  Twenty representative images were visually inspected. Evidence is retained
  under `tabard-app-receipts/2026-10-09/shared-renderer/offscreen-previews-03`.
  No new GPU reset report appeared during these diagnostics. The exact hung
  kernel remains unidentified, and native window acceptance remains pending;
  these offscreen results do not qualify application presentation or reopen.
  Each standalone application root must patch the maintained renderer triple
  explicitly: Cargo does not inherit Mere's root patches into consuming roots.
  Inspect the application's native host dependency closure before its receipt;
  an older unrelated Sprigging bridge may retain its existing Vello types.
- October 10 Pelt native qualification follows the renderer publication.
  The fixed binary passes fresh-10 and separate-process reopen-10 on Radeon
  Pro Vega 56 / Metal: 85 presented editor frames, nine nonblank editor captures
  and two application captures. All eleven PNGs were visually inspected.
  The real workflow authors all four modes and exact CSS, saves, resizes the
  same editor to 640×780, explicitly applies the choice in Pelt and restores
  that exact choice in a new process. Workspace assertions preserve controller
  identity, history, focus and aperture. No new GPU reset report appeared.
  Original evidence is retained under
  `tabard-app-receipts/2026-10-10/pelt/{fresh-10,reopen-10}`; the port's receipts
  preserve logs, presentation counts, hashes and review records. Fresh-09 was
  stopped prematurely as a precaution and is explicitly marked partial.
  Native acceptance now proceeds one application at a time; simultaneous
  browser/native GPU runs remain held while the other integrations qualify.
- Knot's fetched origin `802238cb` already has coherent Mere/Genet pins and
  semantic-field adaptation. An isolated application worktree preserves its
  primary checkout. Its shared-editor adapter extends local desktop preferences,
  leaving persona sync, document contents and writing controls under their
  current owners. Dependency publication and native qualification remain open.
- October 10 narrow native inspection found the workshop brand overlapping
  Undo/Redo/Save at 420 pixels. The shared workshop stylesheet now reserves
  the brand width and moves its action slot to a wrapping row below 480 pixels.
  Production `native_init` font/layout tests cover 360, 420 and 640 pixels with
  platform and explicit caption policies, checking nonoverlap, viewport bounds,
  pointer hit centres and Tab reachability. All 15 native-host library tests
  pass; the same focused test rejects the original stylesheet at 420 pixels.
  The windowless harness does not measure macOS traffic-light insets, so actual
  narrow window acceptance remains required. Knot and Redshank also expose
  intermittent missing title/control paint despite successful presentation and
  nonblank frames. Those receipts are rejected pending shared renderer/retained
  scene isolation; this geometry repair does not close that separate defect.

## Shared authoring and title-bar composition (2026-10-08)

**Decision:** expose the existing workshop as `crates/cambium/tabard-workshop`
and add reusable composition to existing Cambium/native-host crates. The
standalone desktop port remains the reference consumer. Turnstone, Woodshed,
Knot editor and Cleromancy are adoption candidates; this change does not claim
that their independent dependency pins or application settings are migrated.

**Context:** theme derivation and syntax/document/graph previews already use
Tinct, Illume and the existing document and graph components. The workshop
lived under a product port, and native title-bar views were assembled locally
by consumers even though the host already owned window behavior.

**Ownership:** Tabard keeps definitions, modes, libraries and interchange;
`tabard-workshop` keeps authoring state and its portable surface. Cambium owns
`title_bar`, its ornament/title/action/caption slots and inset-aware stylesheet.
The native host owns accessible caption adapters and `SceneProducer<T>` on its
existing render device. Hosts choose application appearance, reader precedence,
frame policy and native commands. Product identity is slot content and CSS,
not another palette or window-control engine.

**Alternatives:** copying the workshop or caption event handlers into each app
would multiply the authoring and platform behavior. A new desktop-chrome crate
would add a package for composition that fits existing ownership. Moving native
commands into portable workshop state would couple embedded editors to a
window. Callback adapters and view slots keep these seams additive.

**Consequences:** consumers can share the complete authoring model and choose
ornament content without replacing their command model. App-frame hosts mount
`TITLE_BAR_CSS`, provide native captions (macOS retains traffic lights), and
match the configured Maximize label to the Windows Snap label. Preview hosts
register retained scenes with distinct raster keys and current semantic
callbacks. Close still passes through each product's unsaved-work policy.

**Adoption work:** integrate one consumer at a time, starting with an existing
Cambium desktop host; preserve each app's settings, reader precedence and
identity. Turnstone's browser-specific adapter and custom calculator editor
remain separate follow-ups. Native Windows/Linux behavior needs platform
acceptance beyond the windowless caption tests. The shared-kit slice passes
285 windowless tests and the native authoring/narrow-layout scenario (five
nonblank captures). Fresh-process state assertions pass but native acquisition
can remain `Occluded`; a fresh empty-library control reproduces it. Preserve
this presentation boundary in the desktop receipts.

**Presentation lifecycle follow-up (2026-10-08):** Rootstock now exposes the
current redraw's existing `PresentedFrame` identity and clears it on an
unsuccessful attempt. Mesquite advances native scenario steps, settling,
frame limits and capture grace only after a new presentation; asynchronous
readbacks still receive every hook turn. A separate ten-second continuous
presentation deadline fails with presentation/redraw counts and cancels only
that lane's capture callbacks. Explicit windowless harness turns keep their
test clock. Validation passes 69 Rootstock tests, 24 Mesquite tests, 18 native
host library tests and four windowless capture/pairing regressions.

Fresh and saved-library native retries produce zero presentations and zero
captures, with 218 and 201 lane redraw turns respectively before the elapsed
deadline; an additional native-state diagnostic produces 129 turns. The owned
macOS window has valid geometry, is visible, can become key and is on the active
Space, but the application remains inactive and its native occlusion state
lacks the Visible bit that Metal checks before acquiring a drawable. A one-time
`focus_window` experiment did not change that result and was removed. The host
retains an owed redraw after initial reveal and opt-in native diagnostics.
Fresh-process presentation remains open at the activation/compositor boundary;
these failed receipts do not replace the earlier five successful captures.
[Concise native evidence and structured receipt projections](../../../ports/tabard/desktop/receipts/2026-10-08_stack/presentation_wait/README.md)
preserve the exact results without attributing the failure to authored state.

**macOS visual acceptance follow-up (2026-10-08):** the production binary from
`7d2a5f3cfe97174058368073d2bae809fffbf902` succeeds when launched as a normal
application through LaunchServices. No renderer, host or occlusion override was
needed. The reusable `scripts/run_macos_scenario.py` launches an isolated
temporary bundle and checks Mesquite's receipt in addition to the launcher's
exit. Three standalone lanes pass with 192 presentations and 15 nonblank
captures: four modes, shared reader/syntax/graph, keyboard seed edit/undo,
authored CSS, resizing and fresh-process reopen. All images were inspected.
[Complete acceptance artifacts](../../../ports/tabard/desktop/receipts/2026-10-08_stack/launchservices/README.md)
supersede the open macOS presentation claim for this launch path, while keeping
the failed background-child receipts. Woodshed has separately qualified its
embedded workshop with four seed/reopen lanes, 132 presentations and 14
nonblank images on its `2fa89ca61d75a9a03bf5e301eb658e696d74e44f` source.
Other consumers, Windows/Linux headed behavior, native OS decorations and live
screen-reader acceptance still require their own evidence.

## Tabard small-web adapters (2026-09-13 scope)

**Current code:** `Theme` in `crates/system/tabard/src/lib.rs` derives a Tinct palette and
emits DTCG color JSON, deterministic CSS custom properties, and
`lagrange_palette_txt()` with typed mapping diagnostics. It is a library. The recorded Pelt chrome/Reader preview
receipts prove consumer mappings, not installed user settings. In Mere,
`crates/system/document-lanes/src/smolweb.rs` accepts `SmolwebTheme::App` and an
explicit `DocumentStyleSheet`. Those are the native reader seams; protocol
parsers need no dependency on Tabard.

**Corrected 2026-10-06 (S14 pass):** `SmolwebTheme` is now defined in
`tabard::smolweb` and re-exported by document-lanes
(`crates/system/document-lanes/src/smolweb.rs:36`; `d16f055f`, 2026-09-24),
so the seam's type is Tabard's. Protocol parsers still need no Tabard
dependency: nematic has none.

Keep three targets explicit: application chrome, reader appearance, and an
author's published stylesheet. Changing a reader preference must not rewrite
source or publish it. Micron's authored colors remain source facts even when a
reader overrides their appearance. Gemtext needs no styling extension.

1. **Native reader adapter.** Map existing Tabard roles into the shared reader
   palette/style sheet, with Knot and Turnstone owning selection, persistence
   and precedence over site-derived defaults. Knot's current
   `knot-editor/apps/desktop/src/appearance.rs` derives its own Tinct palette; its consumer
   can reuse Tabard while keeping editor appearance separate from published CSS.
   Done when one theme is selected, previewed and restored in both apps, while
   document bytes, history, selection and viewport remain held; ordinary,
   visited/focused links and reader contrast overrides stay legible. Preserve
   unsupported roles explicitly rather than pretending every target uses them.
2. **Lagrange UI palette exporter.** Its
   [official help](https://raw.githubusercontent.com/skyjake/lagrange/dev/res/about/help.gmi)
   documents `palette.txt`, with Dark/Light sections and named RGB entries.
   This controls browser UI colors; document themes are separate. Link icons
   also use the UI palette, so test them against different page backgrounds.
   Tabard owns a deterministic exporter and an explicit semantic-role mapping,
   including diagnostics where roles collapse or have no representation.
   Done when exact golden files cover syntax and both modes and a stock Lagrange
   installation actually loads the exported file, including after restart.
   Preserve the documented neutral ordering and status-color meanings; do not
   claim CSS/DTCG import or control over a site's page theme. Exporting a file
   and installing it into an existing client profile are separate user actions.
   **Implementation receipt:** nine Tabard tests pass with `--offline --locked`,
   including both-mode golden output, neutral/accent ordering, reserved status
   meanings, alpha loss and unsupported-role diagnostics. The
   [v1.21.1 loader](https://raw.githubusercontent.com/skyjake/lagrange/v1.21.1/src/color.c)
   ignores `yellow` and `magenta` despite their presence in that release's
   [help](https://raw.githubusercontent.com/skyjake/lagrange/v1.21.1/res/about/help.gmi).
   The documented artifact retains those entries and reports
   `StockVersionIgnored` for each mode. Stock-client visual load/restart and
   link-icon contrast remain unqualified; this receipt does not close that gate.
3. **Geopard contract discovery.** Its
   [official README](https://raw.githubusercontent.com/ranfdev/Geopard/master/README.gemini)
   describes per-domain colors and a configuration directory but establishes
   no theme-import schema. An exporter stays uncommitted until public docs or
   controlled black-box observations establish a supported file contract and
   a stock-client import receipt. GTK/CSS implementation details are not that
   contract. This gate does not delay the other two adapters.

This extends the existing artifact and consumer boundaries. Font packs, arbitrary
stylesheet conversion and additional protocol-specific exporters need separate
consumer evidence before entering implementation. The shared Micron presentation
scope lives in the
[fidelity plan](../../nematic_docs/implementation_strategy/2026-07-01_smolweb_fidelity_plan.md#micron-completion-scope-2026-09-13).

**Ownership clarified 2026-10-07:** the September T1–T5 theme consolidation
is owned by [crate consolidation, C2a](2026-09-23_crate_consolidation_plan.md#c2a-filling-tabard).
That plan also records Tabard's move to `crates/system/tabard` to restore its
shared-library dependency direction. This plan owns theme modes and the
remaining consumer adapter and exporter acceptance work above.

## Appearance workshop (2026-10-07)

**Status (2026-10-08):** W1–W2 implemented; W3 authored-library persistence, portable
interchange, isolated stylesheet authoring and canonical-mode export parity implemented.
Custom-calculator authoring and Turnstone mounting remain open. Mark asked to start Tabard's larger authoring role,
with the browser's SC step 2 proceeding independently. The suite census owns
its product charter: an appearance workshop with authored themes and live
preview, shared by a standalone host and Turnstone.

- **W1 — Draft authoring model.** An isolated draft supports typed edits,
  undo/redo, discard, canonical-mode previews, and explicit commit to the theme
  registry. Built-ins require a fork; a commit refuses a destination collision
  or a concurrently changed/deleted source. Previewing does not activate a
  theme or rewrite reader content. Done when tests prove those boundaries.
- **W2 — Visible authoring surface.** Compose W1 into the reusable Cambium
  surface with seed controls, mode selection and representative chrome,
  reader, syntax and graph specimens. Mark authorized proceeding with a native
  standalone host and the reusable surface. Done when real-control retained
  tests and headed native captures show edits, modes, history and save/reopen.
- **W3 — Artifact fidelity and persistence.** Keep authored definitions
  distinct from exports and host appearance preferences; carry explicit modes,
  harmony and unsupported-role diagnostics through preview/export adapters.
  Done for the library when versioned authored definitions survive save/reload
  and stale/busy/failed writes cannot advance the editing save point. Current
  Legacy CSS/DTCG methods retain their documented normal-contrast profile. The
  workshop uses explicit-mode artifact APIs sharing its exact harmony, base
  palette and syntax derivation; selected stylesheet overrides/custom modes
  refuse a derived color export and can be preserved as authored theme JSON.

**Findings (2026-10-07):** `ThemeRegistry` already supports CRUD, but editing
through it writes immediately. `derive_from_def_for_mode` supplies canonical
previews; custom modes need a calculator or authored sheet and must not be
presented as a successful canonical fallback by the authoring model.

**Progress (2026-10-07):** W1 is implemented in
`crates/system/tabard/src/workshop.rs`. All 49 Tabard tests pass (34 unit,
10 artifact integration, 5 workshop integration). The new tests cover
non-mutating preview, built-in protection, concurrent-edit/deletion/collision
refusal, undo/redo branching, discard, and stylesheet/custom-mode handling.

The workshop package is `crates/cambium/tabard-workshop` (`tabard-workshop`),
with its thin desktop host in `ports/tabard/desktop`. The package moved out of
the product port on 2026-10-08 so sibling applications can embed the same
surface and authoring workflow. One retained state/view provides seed HSL controls, accent harmonies,
four canonical preview modes and chrome/reader/syntax/graph specimens. A
secondary or tertiary hue under locked harmony is explained rather than
offered as an ineffective control. Native name edits synchronize before
navigation and save guards. Previewing never changes the registry's active
appearance; built-ins are forked into user definitions.

`tabard::library::ThemeLibraryStore` persists version 1 authored definitions,
including mode sheets, separately from host appearance preferences. Save
validates and persists a candidate before replacing the live draft/registry;
failure retains edits and history. The store rejects corrupt/future files,
invalid definitions, external changes and cooperating busy writers. Read does
not create a missing library. External editors must honor the lock to exclude
all concurrent check/rename races.

The syntax specimen uses `tinct::derive_syntax_palette_with`: its surface is
the exact selected profile's surface and all syntax roles clear 4.5:1 at
normal contrast or 7:1 at high contrast. The existing syntax API and CSS/DTCG
export behavior remain compatible. Derived specimens explicitly disclose an
attached mode sheet; rendering/editing that sheet, custom calculators,
portable import/export controls and Turnstone mounting remain open.

**Initial surface validation (2026-10-07, macOS x86_64):** 55 shared Tabard tests, 8 retained
workshop tests, 4 desktop tests and 17 Tinct tests pass, plus Tinct's doctest.
The surface tests operate real pointer/keyboard controls and assert laid-out
specimen styles, all four profiles, history, native text synchronization,
save/reload, failed-write recovery and exact AccessKit names/roles/hit boxes.
The desktop wheel test reaches specimens in the stacked 640 × 780 layout.
This verifies the accessibility projection, not a live screen-reader session.

All three headed Mesquite scenarios exit successfully: workshop (8 captures),
fresh-process reopen (1), and narrow preview (3); no captured frame is blank.
The wide logical size is 1180 × 800 at 2× scale; the narrow size is 640 × 780.
Representative images and complete scenario receipts are retained in
[`ports/tabard/desktop/receipts/2026-10-07`](../../../ports/tabard/desktop/receipts/2026-10-07).
Initial/seed/dark frame digests are `7fbfa063e24fb8f2`, `f91bc08134eeb2aa` and
`fd2ddd39027ee59e`; fresh-process reopen is `d4ac3b00f6859e3b`, and the revealed
narrow specimen is `60601a374190fd4c`. The narrow headed lane uses normal
selector scroll-into-view; its wheel behavior is covered by the host test.

`check_port_boundaries.py` passes with both new packages. Strict Clippy with
`--no-deps` passes for `tabard-workshop`, `tabard-desktop` and Tinct. Broader
strict dependency linting remains blocked by existing Meristem type-complexity
and shared Tabard documentation/large-enum/filter-map lints. The documentation
judgment audit reports inherited snapshot-digest and browser-receipt coverage
errors (257/258 active documents); this slice creates no active design document.

### Shared-component specimens (2026-10-07)

**Status (2026-10-07):** implemented. Mark confirmed that the workshop should
exercise the existing stack and authorized extending existing crates as needed.
Cambium now exposes Illume-backed `code_styles`/`highlighted_code`, shared
read-only styled runs, local syntax palette CSS and explicit-mode syntax CSS.
The existing editor APIs retain their behavior; invalid UTF-8 style boundaries
are ignored by the common run builder. The workshop uses these APIs instead of
manually assigned token roles.

The reader extracts the checked-in HTML through Fleece, lowers it to the shared
Inker document and uses document-lanes/document-canvas for shaping and reflow.
Mode and seed changes preserve the source packet. The native producer uses the
host's existing render core/device and invalidates its retained texture when
the library replaces the reader instance. The graph uses Cambium's actual
`GraphCanvasSwatch` and Sprigging leaf, including pointer, hover, focus and
keyboard selection. Its lower-62-bit key follows the shared leaf/producer
namespace. `workshop_stylesheet()` composes the shared component rules.

Native inspection exposed a document-canvas font identity bug: regular and
bold faces in the same collection shared a blob ID and collapsed to the first
face. `FontInterner` now keys by both blob ID and collection index. Its new
regression proves separate faces and same-face clone deduplication; fresh
native reader images show a bold heading followed by regular body text.

**Validation:** Cambium's highlight-enabled suite passes 263 unit tests and its
compile-fail doctest (one existing editor doctest remains ignored). The workshop
passes 6 specimen unit tests and 12 retained surface/component tests; the desktop
passes 4 tests. The mounted tests prove real Rust lexer spans, all 16 syntax
variables in all four modes, stable reader source and real graph selection
without authored-theme mutation. Shared graph tests check paint/target geometry.
Strict Clippy with `--no-deps` passes for both Tabard packages; Cambium's broader
lint run succeeds with existing warnings outside the changed highlighting files.
Port boundaries and scoped formatting pass. Cambium's root module has inherited
ordering differences under rustfmt, left untouched.

The wider document-canvas suite reports 85 passes and one table-wrapping failure
(`normal_width_table_wraps_unbroken_link_inside_its_cell`). A controlled comparison
against the original `HEAD` font interner reproduces the same assertion; the
fixed interner's two identity tests pass. This inherited geometry failure remains
open. The documentation judgment audit retains its initial 257/258 coverage and
snapshot/browser-receipt errors; no active design document was added.

Before the font correction, all four native Mesquite scenarios pass: shared
components (9 captures), authoring (8), fresh-process reopen (1) and narrow (3),
with 181 scenario frames and no blank captures. These prove all four modes,
real graph selection, seed edit/undo, save/reopen and narrow composition. Fresh
font-corrected light/dark images confirm the typography fix. Subsequent full
recapture attempts hit surface occlusion before pending captures could present;
the failed receipts are retained and do not constitute passing scenario runs.
Representative images and receipts are retained in
[`ports/tabard/desktop/receipts/2026-10-07_shared_components`](../../../ports/tabard/desktop/receipts/2026-10-07_shared_components).

This is a bounded read-only reader appearance and selectable graph specimen.
Reader link activation/session accessibility, source editing, full graph
workspace behavior and host appearance activation remain separate capabilities.

### Standalone authoring and interchange (2026-10-08)

The existing surface now supports direct six-digit RGB entry alongside HSL;
Apply preserves authored alpha, invalid/incomplete text stays visible, and
Save/export/navigation cannot silently drop staged color input. The selected
canonical mode can become the theme's authored default in one undoable edit.

A separate application document uses the existing ScriptedDom/Livery cascade,
layout and paint translation. Exact authored mode CSS replaces the derived
sheet in that document; Apply, Clear, parser diagnostics and ordinary selectors
are available through Cambium's shared text input. Editor styles never enter
that document, and authored rules never enter the editor's cascade. Its scene
and the reader share one generic adapter on the native host's existing render
core/device. The typed reader, syntax and graph continue to show derived seed
appearance, with that boundary stated on the surface.

Portable theme JSON preserves authored fields, provenance, harmony and all
mode sheets. Import validates first and forks built-in/colliding identities;
it remains unpublished until Save. The shared crate owns this validation,
draft construction and atomic artifact writer. CSS and DTCG export use the
selected canonical mode and the same effective seeds/contrast/syntax profiles
as the preview. Arbitrary stylesheet overrides and custom calculators cannot
be represented as these derived color artifacts; exporting the full theme
preserves their source. Existing legacy exports retain their behavior.

The desktop chooses export destinations through its existing platform dialog
backend. An occupied destination requires an explicit replacement; export
captures the authored bytes before opening the chooser, never advances the
save point and cannot replace the active library, editor preferences or their
locks through lexical/symlink aliases. Delete validates a candidate library,
persists it, then changes the editor. Failed writes preserve draft/history and
registered definitions. Last selected saved theme and preview mode use the
existing host-choice store in a separate `.workshop.json` sidecar; they do not
activate an appearance in another host. Closing offers Save and close, Close
without saving, or Keep editing, including unfinished fields and unchanged
imported/copied definitions.

**Validation (macOS x86_64):** shared Tabard 70 tests, workshop 40 tests and
native desktop 9 tests pass. Retained acceptance uses actual control dispatch,
native text/file/close hooks and real temporary-file writes. It verifies exact
computed CSS and editor-cascade isolation, RGB/alpha/history, default flags,
import collisions, explicit replacement and protected destinations (including
missing files through parent symlinks and case aliases), transactional deletion,
choice restoration and invalid-input/failed-save recovery. The ordinary native
run composes the same close policy with its scenario lifecycle.

Headed authoring passes 95 frames / 5 captures, then a fresh process passes
18 frames / 1 capture; all six are nonblank. Native wide and narrow images,
authored fixture data and receipts are retained in
[`ports/tabard/desktop/receipts/2026-10-08_usable`](../../../ports/tabard/desktop/receipts/2026-10-08_usable).
The host file-routing suite passes 4 tests; strict Clippy with `--no-deps`
passes for both ports, and the shared crate completes with its existing warnings.
Port boundaries and scoped formatting pass. The documentation audit retains
its inherited digest/browser-receipt errors (257/258 active documents).
OS dialog panels and live screen-reader interaction were not automated.

Custom calculator creation/editing, Turnstone activation, editable content,
full graph workspaces and live screen-reader validation remain open.

## The model (decision record)

- A THEME is a set of tinct-style seed key colors the user picks (`tinct::Seeds` shape: brand
  triad, neutral, optional text overrides, functional hues).
- A MODE is a derivation profile applied to the current theme's seeds. Canonical modes: light,
  dark, high-contrast light, high-contrast dark. The derivation differs per mode: light modes
  derive lighter surface/text ladders, dark modes darker, high-contrast modes wider lightness
  separation and stronger borders/focus tiers. (tinct's `Seeds.dark: bool` is the degenerate
  two-mode version of this; mode generalizes it.)
- Granular override: any theme may attach a CUSTOM STYLESHEET per mode, used instead of the
  derived palette for that mode. Theme + mode resolve to either (derived palette -> generated
  sheet) or (custom sheet as-is).
- Custom modes: users can create new modes, i.e. a custom palette calculator that receives the
  tinct seeds and produces a stylesheet. Candidate execution lanes per the scripting doctrine:
  rhai (host-automation lane) or a declarative mapping first; pick when the phase lands.

## Engine mapping (what rides which mechanism)

- The light/dark pair WITHIN the current contrast level bakes as one fixed sheet: base rules =
  light palette, `@media (prefers-color-scheme: dark)` block = dark palette. Flipping
  light/dark is then the landed engine path (`set_prefers_color_scheme`): media re-evaluation
  over the persistent Stylist, session survives, no rebuild.
- Contrast level (normal vs high) SHOULD ride `prefers-contrast` the same way; stylo's servo
  Device currently exposes only `PrefersColorScheme` in `Device::new`, so investigate whether
  0.18 evaluates `prefers-contrast` and can be fed the preference. If yes: bake all four
  palettes into the one sheet (2x2 media blocks) and both axes flip cheaply. If no: contrast
  switch takes the sheet-swap path (today's rebuild), which is acceptable at its frequency.
- Custom modes and per-mode custom stylesheets are sheet swaps by definition (different rule
  sets, not different media applicability).
- OS-follow: a setting maps the system scheme (and, where the platform reports it, the system
  contrast preference) onto the mode; manual pick overrides. Follows the configurability
  doctrine: expose, do not hardcode.

**Corrected 2026-10-06 (S14 pass):** this section is historical. Its mechanism,
`IncrementalLayout::set_prefers_color_scheme` over Stylo's servo `Device`, no
longer exists in Mere or Genet's components: genet-layout and Stylo were
retired in genet `55c05d11759` (2026-08-21). The Related line's engine half and
the T2/T3 receipts below name the same mechanism; the 2026-09-13 note covers
the July receipts, not this one.

## Phases

### T1. Mode type + derivation profiles

- `Mode { Light, Dark, HcLight, HcDark, Custom(id) }` in the presentation layer; tinct grows
  per-mode derivation (replace `Seeds.dark: bool` at the call boundary with a profile:
  lightness ladder direction + contrast spread; tinct API change is upstream-first in
  `repos/tincture` *(historical citation)* <!-- doc-audit: historical-path --> since Woodshed/Strophe share it).
- **Done when** the four canonical modes derive distinct, sane palettes from one seed set in a
  tinct unit test (hc modes measurably wider text/surface contrast, e.g. an APCA/WCAG floor).

### T2. Bake the scheme pair + cheap flip (P3 host half)

- `theme_sheets::chrome_sheet` takes the active theme's LIGHT and DARK palettes (current
  contrast level) and emits one sheet: light values as base, dark values inside
  `@media (prefers-color-scheme: dark)`. Structural rules emitted once.
- `PaneSession::refresh` gains a scheme input: sheet unchanged + scheme changed calls
  `IncrementalLayout::set_prefers_color_scheme` instead of rebuilding; session creation seeds
  the scheme.
- The non-sheet theming (orrery palette, document palette, actor retheme, decoration caches)
  keys off the resolved mode palette exactly as today, on the same flip.
- **Done when** a light/dark mode flip re-themes the shell with `rebuild_us` absent from the
  capture (the `apply`-scale restyle only) and pixel output matches a from-scratch build of the
  same mode.

### T3. Contrast axis

- Resolve the stylo `prefers-contrast` question (see engine mapping). Either wire the second
  media axis (4-in-1 sheet) or route contrast switches through the sheet-swap path explicitly.
- **Done when** hc modes are pickable, derive via T1, and the chosen mechanism is recorded here
  with receipts.
- **RESOLVED 2026-07-05: sheet-swap path.** Stylo's servo-side media feature table at genet's
  pinned rev (8bde0e9, `style/servo/media_features.rs`) evaluates only `width / scan /
  resolution / device-pixel-ratio / -moz-device-pixel-ratio / prefers-color-scheme`;
  `prefers-contrast` exists gecko-side only. So the 2x2 4-in-1 sheet is not expressible on the
  servo Device today. Wired instead: `rebuild_chrome_sheet` bakes the pair AT the current
  contrast level (`(Light, Dark)` or `(HcLight, HcDark)`); picking a mode across contrast
  levels changes the sheet strings and takes the session-rebuild path (asserted in
  `chrome_sheet_bakes_the_scheme_pair_and_mode_flip_keeps_it_fixed`). Acceptable at contrast-
  switch frequency. Liftable later by adding `prefers-contrast` to the stylo servo table
  (fork territory) or an upstream stylo change.

### T4. Per-mode custom stylesheets

- Theme def gains optional per-mode sheet overrides; resolution order: custom sheet for
  (theme, mode) else derived palette sheet. Overrides participate in the media baking only when
  both scheme counterparts are derived; a custom sheet on either side of the pair forces the
  swap path for that theme (correctness first, optimize later).
- **Done when** a user-supplied dark sheet renders instead of the derived dark palette and
  survives restart (settings persistence).

### T5. Custom modes (calculator lane)

- A registered custom mode = name + calculator producing a stylesheet from `tinct::Seeds`.
  Start declarative if a mapping table covers the real cases; graduate to the rhai lane if
  authors need logic. Custom modes list alongside canonical ones in the mode picker.
- **Done when** a custom mode authored without rebuilding the app produces a working shell
  theme from the active seeds.

## Sequencing

T1 then T2 (T2 is the P3 host half and pays immediately). T3 after the stylo investigation.
T4 and T5 ride settings passes; T5 last.

## Progress

- 2026-07-05: decision recorded, plan written. Engine prerequisite (scheme flip) already landed
  genet-side.
- 2026-07-05: **T1 landed.** tinct (`repos/tincture` *(historical citation)* <!-- doc-audit: historical-path -->, v0.1.1 NOT YET PUBLISHED — Mark's call):
  `ModeProfile { dark, high_contrast }` with `LIGHT/DARK/HC_LIGHT/HC_DARK` consts +
  `derive_palette_with`; hc ladders push surfaces toward the extremes, text past them, and
  tighten the dim/disabled blend; `derive_palette(seeds)` unchanged as the degenerate form.
  Test `four_canonical_modes_derive_distinct_wider_hc_palettes` (7:1 floor + wider-than-normal
  assertions) green. Mere does NOT consume the new tinct API yet (crates.io pin at 0.1.0);
  register-theme derives per-mode through its existing `(dark, hc)` profile machinery:
  `theme::Mode { Light, Dark, HcLight, HcDark, Custom(id) }` (key/label/flags helpers),
  `seed::derive_from_def_for_mode`, `seed::default_mode_for_def`,
  `ThemeRegistry::mode_tokens`. Tests green (mode key roundtrip; four distinct palettes from
  one theme with the hc 7:1 gate; legacy-builtin default modes).
- 2026-07-05: **T2 landed.** `theme_sheets::bake_scheme_pair` (light rules base + dark-only
  rules in one `@media (prefers-color-scheme: dark)` block); `rebuild_chrome_sheet` bakes the
  pair at the current contrast level and refreshes a `chrome_theme_light/dark` token pair on
  `Presentation`; `gather_chrome_css` (roster/apparatus/utility/gloss pane CSS appended to the
  chrome sheet per frame) builds from the PAIR and pair-bakes too, so the chrome sheet identity
  is scheme-invariant. `PaneSession::refresh/scene` take `scheme_dark`: sheet unchanged +
  scheme changed rides `IncrementalLayout::set_prefers_color_scheme` (session + element scroll
  survive); a rebuild seeds the scheme after `new` (engine builds light-default — a
  `new`-with-scheme genet API would save that extra recascade, minor follow-up). Non-sheet
  lanes re-key off the mode tokens via `theme_edit::apply_resolved_tokens` (shared by
  `set_theme` / the new `set_mode`); the chrome base-raster cache folds the scheme into
  `chrome_base_sig`. Mode picker radios on the Appearance page (`mode:set:<key>`); persisted
  as `PersistedSettings::theme_mode`; boot restores it (unset re-seeds from the theme def, so
  the legacy four built-ins keep their meaning — `set_theme` re-seeds the mode the same way).
  Receipts: `chrome_sheet_bakes_the_scheme_pair_and_mode_flip_keeps_it_fixed` (sheet + token
  pair fixed across a scheme flip; hc pick changes them) and
  `scheme_flip_reuses_the_chrome_session_without_rebuild` (`rebuild == false` on the flip).
  meerkat bin suite 220 pass / 3 pre-existing fails (graph_delta_log, roster_view links_tab,
  wallet_pairing — fail at HEAD without these changes; meerkat LIB tests also red at HEAD in
  `ingest.rs`, concurrent-work skew).
  - CORRECTION (same day): the "list-pane pair-baking follow-up" is moot. The standalone
    `ViewPane` panes (RosterPane / ListPane / the settings-pane harness) are TEST harnesses
    only (`main.rs:134`); production roster / gloss / settings panes fold into the chrome
    document and are covered by the pair-baked `gather_chrome_css`. The remaining single-mode
    surfaces — the pelt tile CSS (`tile_sheet`, rebuilt when `pelt_theme != active tokens`) and
    the note-card band bake (`note_sheet`, stateless per re-raster) — are self-invalidating on
    a flip, same cost/behaviour as a theme switch. No further host work needed for the flip.
- 2026-07-05: **T3 resolved** — see the RESOLVED note in T3 (sheet-swap path; stylo servo
  Device has no `prefers-contrast` at the pinned rev).
- 2026-07-05: **T4 landed.** `ThemeDef.mode_sheets: BTreeMap<String, Vec<String>>` (keyed by
  `Mode::as_key`, `#[serde(default)]` so pre-T4 theme files parse; empty lists count as
  absent via `ThemeDef::mode_sheet`; forks carry the overrides). Resolution in
  `rebuild_chrome_sheet`: an override on either side of the scheme pair forces the swap path
  — the sheet is the ACTIVE mode's resolution (custom rules as-is, px-scaled, syntax rules
  appended; else the derived single-mode sheet); only a fully-derived pair bakes the cheap
  flip. Persistence is the theme file itself (`theme_store` serializes `ThemeDef`).
  Authoring surface today: hand-edit `<mere_root>/themes/<id>.json` (the mod-distribution
  path); a settings-lane editor can ride a later settings pass. Receipts:
  `mode_sheets_roundtrip_and_gate_on_non_empty` (register-theme),
  `per_mode_custom_sheet_overrides_the_derived_dark_sheet` (meerkat, the done-when),
  `theme_store::save_then_load_round_trips` extended with a mode-sheet entry (survives
  restart). Suite: 221 pass / same 3 pre-existing fails.
- 2026-07-05: **T5 landed, declarative lane** (the plan's own "start declarative" pick; the
  rhai graduation stays open for when authors need logic — the file shape below is the
  compatibility floor). A custom mode is `<mere_root>/modes/<id>.json`
  (`register_theme::mode_calc::CustomModeDef`): id + name + declared `(dark, high_contrast)`
  flags + a mapping table from every `ChromeTheme` role to a small OKLCH transform of one
  seed (`seed`, optional `l` / `c` / `rotate` degrees / `alpha`, or `on: true` for the
  contrast-picked text over the computed fill) — the same tinct maths as the built-in
  derivation, tiny and reviewable like theme files. Load: `mode_store::load_custom_modes` at
  boot (incomplete / malformed / duplicate files skipped + logged; completeness is proven by
  a seed-independent dry run). Resolution: custom modes are sheet swaps by definition —
  `rebuild_chrome_sheet` generates the sheet from the calculator's tokens (no baked pair);
  the non-sheet lanes (orrery / document palettes) derive canonically with the mode's
  declared flags, with the calculator's chrome overlaid (`set_mode` + boot mirror each
  other); `scheme_dark()` presents the declared scheme to the engine. Picker: customs list
  after the canonical four (`mode:set:custom:<id>`); persisted as `custom:<id>`, a missing
  file at boot falls back to the theme default. Failure posture: unknown id / failed eval is
  a logged no-op (pick) or canonical fallback (rebuild/boot) — a stale mode can't blank the
  shell. Receipts: `mode_calc` unit tests (transforms, `on`, rejection-by-name, JSON
  roundtrip), `mode_store` load test, and
  `custom_mode_file_produces_a_working_shell_theme` (the done-when: authored file → boots →
  listed → calculator palette renders → survives restart). Suite: 223 pass / same 3
  pre-existing fails; register-theme 21 pass.
- Remaining follow-ups: the rhai calculator lane (if declarative tables prove insufficient),
  an in-app editor surface for T4 per-mode sheets and T5 mode files (a later settings pass),
  the genet `new`-with-scheme micro-optimisation, and publishing tinct 0.1.1 (then
  optionally migrating register-theme's derivation onto `tinct::derive_palette_with`).
- 2026-07-05 (follow-ups pass): **tinct migration landed** — 0.1.1 published (Mark), pin bumped,
  `derive_token_set` bases on `derive_palette_with(seeds, ModeProfile)` (hc widens at the source;
  the stricter local hc branches stay). register-theme 22/22, meerkat theme tests green.
  **T4/T5 editors landed** (Appearance page): per-mode stylesheet rows for the active USER theme
  ("derived — copy to theme file" materializes the derived sheet into `mode_sheets` /
  "custom sheet — clear to derived"; built-ins show the fork hint); custom-mode management
  ("+ New custom mode (from current)" seeds a complete `CustomModeDef::template` file — valid for
  all four flag combos by test — plus per-mode Remove and "Reload modes from disk" for
  hand-edits; active removed mode falls back to the theme default). `mode_store` gained
  save/delete. The FILES stay the authoring surface (mod-distribution path); the editor
  materializes, removes, reloads. Receipts: `mode_sheet_editor_materializes_and_clears`,
  `custom_mode_editor_creates_removes_and_reloads`,
  `template_is_complete_and_valid_for_all_flag_combos`; suite 225 pass / same 3 pre-existing
  fails. Still open: genet `new`-with-scheme (deferred while moveBefore edits genet-layout)
  and the rhai calculator graduation.
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere 535bca11,
  from the D2 record in support/doc-audit/d2/batch_49_s14_phase_b11.md: the Engine-mapping
  section marked historical (genet-layout and Stylo retired in genet `55c05d11759`),
  `SmolwebTheme`'s move into `tabard::smolweb` (`d16f055f`) noted, and the Tabard scope's
  owning plan opened as a question.
