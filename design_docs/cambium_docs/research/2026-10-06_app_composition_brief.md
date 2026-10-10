# App composition brief: one Cambium application inside another

**Date:** 2026-10-06
**Status (2026-10-09):** research, ruled. AC1 to AC6 are ruled (§9,
"Rulings (2026-10-07)"): retained in-process sessions joined as AccessKit
subtrees through one Mere helper, E1a and E1b authorised, panics caught,
room reserved for forest mounts, and paint lists as the cross-process research
target. Turnstone's unusual-protocols lane owns the session-seam work and E1
(Turnstone U15). A read-only lane wrote the brief under fork U8 of that plan;
no code changed and nothing was built or run.

The initial assessment and the E1 progress below retain their source dates.
The [current reconciliation](#11-current-consumers-and-design-direction-2026-10-09)
links the design language, actual consumer progress, ambient-context proposal
and the separate Moot applet experiment. It changes no ownership or release bar.

Read against Mere `362c5d5a` (main moved during the read; no commit in that
window touched `crates/cambium`), Turnstone `032463ae` (the committed tree:
another session is repinning its working tree, which was not used), Knot
`210ee64b`, Woodshed `cefc903d`, wgpu-graft `95bab7e5`, wgpu-weld `b5cf0435`,
wgpu-scry `2c3ebd24`, netrender `0edc6077`, Genet `90c5ef50`, and the AccessKit
0.24.1 family Mere's lock resolves, read from the local cargo registry.

## 1. The question

Fork U8 of Turnstone's unusual-protocols browser plan
(`turnstone/design_docs/2026-10-06_unusual_protocols_browser_plan.md`, at
Turnstone `62b29eb`) asked how and when the stack's applications compose. The options were crate
panes and session projections beside the bar work, waiting until the bar is
met, or a live-pixel research lane as well. Mark:
**"start the live-pixel research lane but don't start integration yet... if
somehow the forest dom can accommodate as long as the app is built on
cambium... that would be pretty nice"**. Then: **"ah, the old tearout plan
might be archived now but it's also worth a look"**.

His wider aim, from the same plan's bar: "It would be really interesting if the
apps of the stack could kinda compose together… runtime or built… hmm. But
that sounds quite a tall order for a dude and an llm." He values Turnstone
"done right" at $1,000,000 and Woodshed and Knot at $100,000 each. The machine
has 32 GB of RAM.

The second round of that plan's rulings (Turnstone `032463ae`, U2) set the
weight this brief gives accessibility. Mark: **"i would hope we could meet the
accessibility bar on all platforms. i would prioritize that over pretty much
anything, because that's the difference between a browser that works and one
that doesn't for real folks, no matter the protocol."** It follows there that
AccessKit's adapters on Windows (UIA), macOS (NSAccessibility) and Linux
(AT-SPI), checked with Narrator or NVDA, VoiceOver and Orca, are the bar's
first requirement.

Stated plainly: how can one Cambium application's surface appear and work,
live, inside another (Knot or Woodshed inside Turnstone), and can the composed
whole reach a screen reader on all three operating systems as one tree, with
focus and actions working?

## 2. Families compared

Two families, as briefed, with the variants the code actually offers:

- **A. Live pixels from another process.** The guest keeps its own tree;
  the host gets pixels.
  - **A1.** Shared GPU textures, per operating system.
  - **A2.** Paint lists sent across the process boundary and rasterized by the
    host, a variant this read found (F7).
- **B. Composition at the tree level.** The host reads the guest's tree, laying
  it out, hit-testing it and projecting its accessibility.
  - **B1, view level.** The guest's view function is embedded in the host's
    view through a lens, in one runner and one document.
  - **B2, session.** The guest is a retained session in the host's process,
    with its own state and its own document. Turnstone works this way today.
  - **B3, forest root.** The guest's runner builds into the host window's one
    document, under a mount node. This is Mark's "forest dom" hope, extended to
    a second application.
  - **B4, cross-process session projection.** The guest serves a projection
    over a Graphshell session, and the host mounts it and returns intents.

The line between A and B is not "pixels or tree": Turnstone rasterizes every
pane into its own texture and composes them (F1). It is who holds the guest's
tree. Under B the host does, so layout, hit testing, caret geometry and
accessibility come from one place. Under A the guest does, and each of those
has to be carried across.

## 3. Findings

### F1. Turnstone composes panes as surfaces, and already hosts Cambium apps in-process

Turnstone is not a rootstock host. It depends on `cambium`, `cambium-winit`
and Genet's `genet-winit-host` (Turnstone `Cargo.toml`), not on
`cambium-rootstock` or `cambium-genet-winit-host`. "Turnstone places **one
composited surface per pane**" (`turnstone/src/panes/mod.rs`, lines 16 to 22).
Each surface is rasterized into its own target and composed at its placement
(`turnstone/src/shell/render.rs`, lines 984 to 993), and a surface from an
external producer enters the same list as `PlannedLayer::Imported` (lines 855
to 859).

Product surfaces arrive through the contributed-surface seam
(`turnstone/src/contributed_surface.rs`, lines 7 to 12). "Products own source
decoding and the concrete Cambium runner. Turnstone owns admission, retained
layout, viewport scrolling, hit testing, and the small amount of event routing
needed to get from a pane coordinate to a retained session." Four providers
are registered, among them Knot's document surface
(`turnstone/src/knot_document_surface.rs`, built from
`knot-editor/crates/knot-document/src/document_view.rs` lines 179 to 195) and
Redshank's compact dock (`woodshed/ports/redshank/surfaces/src/surface_api.rs`).
So family B2 is real: two Cambium applications from two other repositories
already run live inside Turnstone, each with its own state and document.

Turnstone uses the forest document too, but only for its chrome: every
window's chrome is a window-root of one `GenetMultiRunner` document
(`turnstone/src/chrome_view.rs`, lines 7 to 19 and 649 to 651).

### F2. The session contract erases a whole app, and stops short of four things

`RetainedSurfaceSession` (`crates/cambium/cambium/src/surface.rs`, lines 55 to
75) is an object-safe Cambium app: a descriptor, availability, its document and
root, focus, focus traversal, focusables, pointer capture, hit-target queries,
viewport sync and event dispatch. It was frozen as v1 with the descriptor
vocabulary on 2026-08-26, and "Changes are additive until a v2" (lines 7 to
11). By design it "does not perform layout, hit testing, scene conversion,
scrolling policy, accessibility hosting, or lifetime management for the host"
(lines 13 to 16). What it leaves out matters here:

- **No accessibility of its own.** The host projects the session's DOM.
- **No value or file events.** `ResolvedSurfaceEvent` (lines 45 to 53) has
  click, key, pointer, hover and wheel. The runner has `dispatch_value`
  (`crates/cambium/cambium/src/runner.rs`, line 1151), which a screen reader's
  SetValue reaches in rootstock hosts.
- **No state access.** Redshank works around it with a shared handle:
  "that trait carries no state access at all, so the two facts a host must
  exchange every frame ... travel through a shared `CompactDock` handle"
  (`surface_api.rs`, lines 1 to 15).
- **Focus traversal wraps inside the session.** The runner's traversal steps
  modulo the focusable count (`runner.rs`, around line 567), so Tab cycles
  inside a guest and never hands focus back to the host. For a keyboard or
  screen-reader user that is a trap at every composed boundary.

IME is not missing: `Key::Composition` carries preedit and commit
(`crates/cambium/cambium/src/key.rs`, lines 50 to 77), and a session receives it
as a key event. The candidate window's position comes from the laid-out caret,
which the host has under B (rootstock's `sync_ime_area`,
`crates/cambium/cambium-rootstock/src/frame.rs`, lines 406 to 430).

### F3. The forest document is one application's windows

Stack seams P2 landed on main at `40d7ae5e` (stack seams plan, §5). Its
rulings: one state with N lenses (S24), the forest document with a
window-root per window (S28), and leaves and producers in one shared registry,
painted per window (S29). In code:

- `GenetMultiRunner` owns one `State`, and "All projections share one
  `Logic`/`V` *type* ... heterogeneous window types would need boxing and have
  no consumer yet" (`crates/cambium/cambium/src/multi.rs`, lines 78 to 95).
- A dispatch in any window rebuilds every other projection
  (`rebuild_others`, lines 212 to 226, called from each dispatch).
- `MultiHost` lends the one runner, the shared host state and the hooks to
  each window's turn (`crates/cambium/cambium-rootstock/src/multi_host.rs`,
  lines 373 to 397 and 525 to 555). Its mutation router files each mutation
  under the window whose subtree it touched (lines 222 to 343).
- Every window-root is a direct child of the document (`multi.rs`, lines 132
  to 145), and `WindowDom` presents it as the whole page, so that a `:root`
  rule matches the window's top element
  (`crates/cambium/cambium-rootstock/src/window_dom.rs`, lines 20 to 27).
- The windows share one author sheet, with a generation counter
  (`crates/cambium/cambium-rootstock/src/host.rs`, `swap_sheet`, line 1508).

The forest is the substrate for one application's windows. It is not one for
several applications. Knot and Woodshed are rootstock hosts, but both use the
single-window `run` (`knot-editor/apps/desktop/src/lib.rs` line 32; the
`run(` call in `woodshed/crates/woodshed-genet/src/main.rs`).

Mounting a second application as another root would need:

- runners of different state types over one document (`RunnerTree::build_at`
  can mount at any node, but it is crate-private, `runner.rs` line 113);
- routing of hit tests, focus and mutations by mount;
- style scoping, since a guest under the host's window root inherits the
  host's sheet;
- namespaced leaf and producer keys;
- panic containment.

### F4. View-level embedding exists

Meristem carries Xilem's `lens`, `map_state` and `map_action`
(`crates/cambium/meristem/src/views/lens.rs`, lines 40 to 92;
`crates/cambium/meristem/src/views/map_state.rs`). Knot's document view and
sheet are exported for reuse (`KNOT_DOCUMENT_CSS`, `knot_document_view` in
`knot-editor/crates/knot-document/src/lib.rs`), and Woodshed's views are "View
fns + CSS sheets consumed by both hosts"
(`woodshed/crates/woodshed-views/src/lib.rs`). B1 needs no new mechanism. It
couples hardest: the host's state holds the guest's state, and the guest's
actions are mapped into the host's.

### F5. Producers are same-process and describe themselves thinly

`TextureProducer` renders on the host's own `wgpu::Device` and queue
(`crates/cambium/cambium-rootstock/src/producer.rs`, lines 68 to 79). Its
accessibility is `ProducerSemantics`, a slot role and name with one flat level
of `ProducerNode` children. Each child has a key, one of five roles (list, list
item, group, image, graphics object), a name, a rectangle and actions
(lines 127 to 203). There is no text, value, caret, selection or nesting.
`cambium-winit-a11y` lowers it to AccessKit nodes, with each action as a child
button (`crates/cambium/cambium-winit-a11y/src/lib.rs`, lines 290 to 351), and
the web host lowers it to ARIA (`crates/cambium/cambium-genet-web-host/src/mirror.rs`,
line 83 onward). That suits a graph canvas. It cannot describe a text editor.

### F6. Cross-process pixels: the host half exists, the guest half exists only on Windows, and freshness is hard

On the host side, Turnstone composes imported textures beside its own
(F1). Its Scry importer checks metadata and the exact fence, and waits on
every paint (`turnstone/docs/receipts/browser_scry_windows_20261005/README.md`).

The family's parts, by operating system:

| | Export from a renderer | Import into wgpu | Fence | Handle between processes |
|---|---|---|---|---|
| Windows | D3D12 `HEAP_FLAG_SHARED` resource and NT handle (`wgpu-graft/grafting/src/raw_gl/dx12.rs`, lines 10 to 11 and 76 to 118) | `import_dx12_shared_texture` (`wgpu-graft/grafting/src/dx12_shared_texture.rs`) | shared D3D12 fence from a wgpu device (`genet/components/genet-compositor/interop/windows_dx12.rs`; `wgpu-graft/grafting/src/sync_dx12.rs`) | Weld duplicates CEF's handle in `on_accelerated_paint` (`wgpu-weld/welding/src/windows_cef/mod.rs`, line 12); nothing brokers a handle between two of our own processes |
| Linux | none found from wgpu | DMABUF through Vulkan (`wgpu-graft/grafting/src/vulkan_dmabuf.rs`), proven only by an ignored round-trip test that needs three Vulkan extensions (`wgpu-graft/grafting/tests/dmabuf_roundtrip.rs`, lines 7 to 24) | Vulkan sync module | no fd passing found |
| macOS | none found | `MTLTexture` import in-process (`wgpu-graft/grafting/src/metal_texture_ref.rs`) | Metal sync module | no IOSurface or Mach port transfer found |

This narrows the Turnstone plan's §4 line that "only D3D12 shared handles
exist, from Weld and Scry". Import halves exist for Linux and macOS too, but
the full export, import and fence set is Windows-only, and no platform has a
broker between two of the family's own processes.

The Scry receipt shows how hard this class is, even with a vendor runtime.
DOM assertions passed while the captured pixels were stale ("B's tab title
reports `clicks=2` but its captured body still shows `clicks=1`"), and reopened
tiles were blank. A supplier defect was found in Scry 0.7.1's two-slot capture
pool. Capture allocated a new texture per paint, and "CDP key/text dispatch is
distinct from physical keyboard and OS IME acceptance". Another session's
uncommitted receipt in Turnstone's working tree
(`docs/receipts/browser_supplier_integration_20261006/README.md`) reports
current input and scroll pixels passing natively after a qualification phase,
with "Supplementary Unicode and OS IME remain open". While a live capture
surface exists, Turnstone keeps redrawing to drive the capture mailbox
(`render.rs`, around line 861).

### F7. Paint lists already have a wire form

netrender's `PaintEnvelope` is the "Wire shape for transporting a `PaintList`
across IPC, fixture files, or any boundary"
(`netrender/paint_list_api/src/lib.rs`, lines 307 to 341). It is serde: the
commands, the viewport, a generation, and the font and image resources the
commands use. Mesquite already writes it as postcard sidecars
([Cambium architecture](../technical_architecture/2026-09-03_cambium_architecture.md),
Mesquite paragraph). So a guest process could send paint lists rather than
textures, with no GPU sharing and the same code on all three systems. What does
not travel as is:

- fonts and images, unless a cache protocol stops them being resent;
- leaves held as renderer-side retained fragments (`RetainedFragmentRef` is a
  renderer-allocated id);
- producers' external textures. The architecture doc says rasterizing a packet
  "with external GPU references still needs the separate producer
  import/capture contract".

### F8. Session projections compose across processes, with receipts

The Graphshell machinery carries projections and intents between processes.
The receipts are G3 (Turnstone's endpoint, `turnstone/src/remote_projection.rs`),
K2 (Knot's two-machine physical receipt, 2026-08-08) and I3h (a founder-held
Knot document edited live by place members through projection,
`turnstone/design_docs/2026-07-28_turnstone_place_port_plan.md`, T5c). Knot
authoring in Turnstone shows the hybrid that matters most here: "The background
hub owns the one endpoint process and all carrier traffic. Each visible
document owns a local Cambium editor. Keystrokes, selection, undo, IME,
highlighting, outline, folds, and preview therefore stay on the UI thread; only
Open, Save, and revision refresh cross the carrier"
(`turnstone/src/knot_authoring.rs`, lines 7 to 12). The presentation runs
in-process (B2) and the authority across the process boundary (B4).

The frozen realization turns a scene into navigable semantics for any host
(`crates/graphshell/graphshell-client/src/frozen.rs`, lines 6 to 24).

### F9. Accessibility: one composition primitive exists on all three platforms, and the stack has not used it

- **AccessKit 0.24.1 composes trees.** A `TreeId` names each tree, a graft
  node with `tree_id` set hosts a subtree, each `TreeUpdate` names its tree,
  and `ActionRequest` carries `target_tree` (`accesskit-0.24.1/src/lib.rs`:
  `TreeId` lines 737 to 750, the graft property 2181 to 2188, `TreeUpdate`
  2829 to 2880, `ActionRequest` 2949 to 2954). Subtree focus follows the graft
  chain ("For subtrees, this specifies which node has focus when the subtree
  itself is focused").
- **Every pinned adapter's consumer implements it.** `accesskit_windows`
  0.32.1 uses `accesskit_consumer` 0.35.0, `accesskit_unix` 0.21.1 uses
  `accesskit_atspi_common` 0.18.1 and so consumer 0.36.0, and `accesskit_macos`
  0.26.3 uses consumer 0.38.0. Each consumer's `tree.rs` keeps graft parents,
  and each `node.rs` treats a graft node's child as its subtree's root
  (consumer 0.35.0 `node.rs`, lines 106 to 157). Read from source only; no
  platform has run it.
- **No adapter takes another process's tree.** UIA's
  `GetEmbeddedFragmentRoots` returns null (`accesskit_windows-0.32.1/src/node.rs`,
  line 1062). The one AT-SPI `embed` call registers the application's own root
  with the registry (`accesskit_unix-0.21.1/src/atspi/bus.rs`, line 81). So
  composition happens in the host's own adapter or not at all, whichever
  family delivers the guest.
- **The stack publishes only the root tree.** Every `TreeUpdate` found in
  Mere's crates, Genet's components and Turnstone's source uses `TreeId::ROOT`. Turnstone joins its contributed
  panes by hashing paths into one id space and translating pane-local bounds
  (`turnstone/src/contributed_a11y.rs`, lines 7 to 12), and it routes actions
  back with an admission-generation check (lines 30 to 49).
- **Only Windows has been heard, and never a composed pane.** The 2026-08-20
  Narrator pass walked Turnstone's stitched tree through the Frozen Projection
  pane, "27 of 27" (`turnstone/design_docs/2026-08-20_screen_reader_pass_receipt.md`).
  That pane shows a session projection, so the one screen-reader receipt that
  touches composition is a B4 receipt on Windows. The same pass found that
  Narrator stops at nodes with no bounds. For contributed panes, "A manual
  Narrator walk remains the honest gate for claiming screen-reader coverage of
  a registered surface" (`turnstone/design_docs/2026-07-17_genet_probe_automatability_plan.md`,
  around line 310). No VoiceOver or Orca run of any stack application is
  recorded.
- **Custom actions are not read.** No pinned adapter reads AccessKit's
  `custom_actions`, so actions are lowered as child buttons (dynamics grammar
  plan, F65, which Mark ruled).

### F10. A crate pane joins the pin set, and the set is already spread

The heads read pin five Mere revisions between them:

- Turnstone, `3d1cdacc` (Genet `69a2383b`, Knot `92719898`, Woodshed
  `9e982b88`);
- Knot, `9310518b` (Genet `b1eb3af1`);
- Woodshed's root workspace, `8106c7c2`;
- Redshank's nested workspace, `bd5912fb`;
- Mere main itself, which pins Knot `ef89a186`.

Two Mere revisions in one graph give two copies of every type, so a guest
compiled into a host must build at the host's revisions. Mere and Knot already
move in lockstep for this reason. The frozen v1 contracts (`mere-surface-api`'s
descriptor, `crates/system/surface-api/surface.rs` line 11, and
`RetainedSurfaceSession`) narrow what can break. But a guest still links
Cambium and Genet themselves, so a breaking change moves every composed guest
first. A process boundary removes this: each side builds at its own revisions,
and the wire is the contract.

### F11. Nothing contains a guest's failure in-process

"A panic inside a turn leaves the lent runner in that window; nothing catches
it today" (stack seams plan, F20). Turnstone's contributed surfaces have an
`Unavailable` admission error (`contributed_surface.rs`, line 66) but no
failure state for a session that has already been admitted. A hang on the UI
thread stalls the host under every B variant except B4.

### F12. Memory and frame cost are unmeasured

No document records a per-process memory figure for any stack application.
What can be computed: a 1,600 by 1,000 physical-pixel pane is 6.4 MB per RGBA8
buffer, and a 3,840 by 2,160 window is 33.2 MB. Turnstone already spends one
such target per pane under B2 (F1). A1 adds the guest's own pool (two slots in
Scry, plus a new allocation per paint) and a second process with its own
device, glyph atlas, fonts and Genet. *Reading, not ruled:* 32 GB will not be
the binding limit for a handful of guests under any family. GPU memory on an
integrated GPU, frame pacing and power will be. With two windows in one
process, Win32 already starved one animating window until the multi-window
entry served both from its idle turn (stack seams F18). A cross-process frame
adds at least one frame between the guest's submit and the host's composition:
16.7 ms at 60 Hz.

## 4. What the archived tear-out plans still hold

The [tear-out composability plan](../../archive_docs/2026-07-04_completed_plans/2026-06-19_tearout_composability_plan.md)
and the [tear-out gestures plan](../../archive_docs/2026-10-06_completed_plans/2026-06-24_tearout_gestures_plan.md)
were written for meerkat, which was deleted on 2026-07-18. The
[tear-out operations brief](../../mere_docs/research/2026-05-11_tearout_operations_brief.md)
predates the host pivot too. Their code pointers are history. These ideas
still hold:

- **C2, the external-texture input bridge.** Its mechanism has a Cambium form.
  A producer's slot receives content-local pointer events through Genet's
  paint transform (Cambium architecture, "External viewport producers"). Its
  only intended consumer, "a live genet-rendered textured body", never
  shipped. The
  [native-surface compositing plan](../../archive_docs/2026-07-03_completed_plans/2026-06-19_native_surface_compositing_plan.md)
  ruled why, in its split of host surfaces by nature: genet-rendered content
  becomes "a **real DOM subtree** in the shell document (rides a11y,
  find-in-page, selection, true scroll)", and only genuinely external content
  becomes a texture. A Cambium guest is genet-rendered, so that split already
  places it in family B. Its finding 5 also holds for any cross-process guest:
  the host owns focus and forwards keys and IME by API, never through OS window
  focus, and places the IME window from the guest's reported caret.
- **C3, cross-window pane resolution.** A pane is a view that resolves an
  authority by id, and edits propagate because two views resolve one
  authority. That is S24's one state with N lenses inside an application.
  Between applications the authority is the guest's session (B2) or its
  Graphshell session (B4). The camera lesson carries over too: view state
  belongs to the view, so a guest's scroll and viewport belong to the host's
  pane. `sync_viewport` and Turnstone's pane scroll already put them there.
- **C4, cross-graph composition with provenance.** The kernel's
  `copy_node_from` and `CopiedFrom` survive
  (`crates/graph/graph-kernel/src/graph/cross_graph.rs`, line 51). The
  stability principle carries over unchanged: a binding changes only on an
  affirmative act. *Reading, not ruled:* a composed pane is a leaf, a live view
  of the guest's own authority. Taking content from it into the host's graph is
  a fork, a copy with `CopiedFrom`, never a silent rebind.
- **The gestures plan's live parts** are Turnstone's leaf, branch and fork arms
  (`turnstone/src/action.rs`). `turnstone/src/panes/tearout.rs` is a 48-line
  payload that still names gpui. None of it composes applications.

## 5. Point by point

| | A1 shared textures | A2 paint lists | B1 view level | B2 session | B3 forest root | B4 projection |
|---|---|---|---|---|---|---|
| Exists today | host import and composition (Turnstone, Scry, Weld); export only on Windows | the wire form only | lens and map_state; views exported | yes, four providers in Turnstone | no; the forest serves one app | yes: G3, K2, I3h |
| Input | forwarded by IPC, content-local | forwarded by IPC | the host's own | the host resolves the target, the session dispatches | the host's own, routed by mount | intents |
| Focus and IME | protocol; the guest reports its caret | the same | native | per-session focus; traversal wraps (F2); IME native | needs per-mount arbitration | not applicable to generic drawing |
| Accessibility | the guest's whole AccessKit tree shipped and grafted (F9) | the same | automatic: one DOM | one tree by stitching or by AccessKit subtrees | automatic: one DOM | frozen semantics; Narrator-proven |
| State isolation | process | process | none: the guest's state is a field | per session; no state access (F2) | per runner, one document | process, plus grants |
| Crash isolation | process (fence waits need timeouts) | process | none | none today (F11) | none, and one shared document | process |
| Versions | each process its own | each process its own | the host's pin set | the host's pin set (F10) | the host's pin set | each process its own |
| Memory and frame | second device and pool; +1 frame | resources resent; +1 frame | none added | one target per pane | one layout per window, guest included | one process per app |
| Cost for a dude and an LLM | high, three OS lanes | medium to high | low | low | medium | medium per app |

The points that decide it, in prose:

**Input, focus and IME.** Under B the host has the laid-out tree, so hit
testing, caret geometry and the IME rectangle come from the same layout that
painted the pixels, and nothing goes stale. B2's one real gap is focus at
the edge (F2). Under A every one of these becomes a message, and the guest
must report its caret every time it moves. Scry needed a qualification phase
just for pixel freshness (F6).

**Accessibility.** This decides the comparison; §6 gives the verdict per
family.

**State and crash isolation.** B1 to B3 share the host's process, thread and
panic. Catching unwinding panics at every call into a session would contain
most failures, but not a hang, an abort or a guest that exhausts memory. A and
B4 get the operating system's isolation. A1 adds one hazard of its own: a
guest that dies mid-frame leaves a fence unsignalled, so every host wait needs
a timeout.

**Versions.** In-process guests join the host's tested pin set, and today that
set spans five Mere revisions (F10). This is the strongest argument for a
process boundary. The accessibility channel narrows it, though: under A,
AccessKit's serde shape is the wire, so both sides must agree on AccessKit's
major version unless the host translates.

**Performance.** *Reading, not ruled:* B2 lays out each guest alone, so a
keystroke in Knot relayouts Knot's document and not Turnstone's. Under B3 the
mutation router's granularity is the window (F3), so the same keystroke
relayouts the host's whole window subtree. A pays a second process and at
least a frame of latency.

**Cost.** B2 already works. Its gaps are each a stack change of a few hundred
lines at most, plus headed screen-reader runs on three systems. Those runs are
the expensive part, and every family needs them. B3 is new work in Cambium and
again in Turnstone, which does not run on `MultiHost`. *Reading, not ruled:*
P2's one-app forest was estimated at 1,700 to 2,700 lines and landed in four
stages, so a multi-app forest is of that order. A1 is three operating-system
lanes plus a protocol, plus a freshness qualification like Scry's.

## 6. Accessibility, per family

The question: can a composed app's tree reach the screen reader as one
coherent tree, with focus and actions working, on all three operating systems?

- **B1 and B3: yes, by construction.** The guest's DOM is part of the host's
  document, so Genet projects one tree with real geometry, and the window's
  one adapter publishes it. They inherit whatever the single-app path achieves
  on each OS, which today is heard on Windows only.
- **B2: yes, through one host adapter.** The tree is joined either by
  stitching, as Turnstone does now, or by AccessKit subtrees, which all three
  pinned adapters' consumers implement (F9). Subtrees remove the id hashing,
  put each session's focus behind its graft node and route each action to its
  session by `target_tree`. Two gaps must close first. Tab traversal must hand
  focus back at a session's edge (F2), or every composed pane is a keyboard
  trap. And no screen reader has yet walked a contributed pane on any system.
- **B4: yes, for what the projection carries.** The frozen realization is the
  only composed surface ever heard by a screen reader (Narrator, 27 of 27).
  Bespoke interaction needs the guest's surface crate in the host, which makes
  its accessibility B2's.
- **A1 and A2: only if the guest ships its whole AccessKit tree.**
  `ProducerSemantics` cannot describe an editor (F5). No adapter accepts
  another process's tree, so the host must graft the shipped updates into its
  own (F9). That works in principle on all three systems. But it adds what B
  avoids: bounds mapped across the boundary (Narrator stops at nodes with no
  bounds), focus events crossing IPC, and a stale tree whenever the guest
  hangs. Family A needs B4's tree channel anyway, and adds pixels on top.

*Reading, not ruled:* under Mark's accessibility-first ruling, this ranks the
families B1 = B3 > B2 > B4 > A for screen-reader coherence. Only B2 and B4
also exist today.

## 7. Recommendation

1. **Make B2, the retained session with its own document, the stack's way of
   showing one Cambium application inside another.** It exists, two
   other-repository apps already run that way, and it keeps accessibility in
   the host's one adapter. Close its gaps in Mere, not in each host, so every
   host gets them:
   - compose each session's AccessKit tree as a subtree (`TreeId`, a graft
     node, actions routed by `target_tree`) through one shared helper. Turnstone's
     path-hash stitching (`contributed_a11y.rs`) then becomes a consumer of it;
   - hand focus back to the host when traversal leaves a session's first or
     last focusable;
   - add value and file events to the session's events;
   - catch unwinding panics at every call into a session, retire it to an
     "unavailable" surface and drop its subtree;
   - give each session its own namespace for leaf and producer keys. Woodshed's
     fretboard keys are fixed constants, `woodshed/crates/woodshed-views/src/fretboard_leaf.rs`
     lines 16 to 22, and the registry's rule is one key in one registry.
   Each change is additive to the frozen v1 contract.
2. **When an app's authority must live in its own process** (a device such as
   Woodshed's audio, a vault, a holder that serves others), use Knot
   authoring's hybrid. The presentation crate runs as a B2 session in the host,
   and the authority runs behind a Graphshell session (B4). Isolation and
   accessibility both survive. The cost is that the presentation crate joins
   the host's pin set.
3. **Reserve room for B3 now and build it when a consumer needs it.** A
   consumer here is a guest's content interleaved in the host's own layout, or
   a selection or a find that crosses the boundary. To reserve room: a session
   can be built at a mount node in a host's document, through a public form of
   Cambium's existing `build_at`; a guest's sheet carries a scope class, as
   Redshank's theme already does; and keys carry the namespace from point 1. A
   B2 session already moves between windows with its document intact, since
   its document belongs to no window. So tear-out does not need B3.
4. **Keep live pixels for content that is not Cambium** (Scry, Weld), as the
   2026-06-19 split by nature ruled. If cross-process bespoke Cambium surfaces
   are wanted later, to get a process boundary's version firewall, the research
   target is A2's paint lists with shipped AccessKit trees, not A1's shared
   textures. A2 needs no per-OS GPU work, and the accessibility channel is the
   same either way.

So the answer to Mark's hope is yes. The forest can take an app built on
Cambium, and it is already part of how the stack composes: Turnstone's chrome
is a forest, and B1 is tree composition in one document. But a second
application's best first home is its own document beside the host's, joined
in one accessibility tree. Mounting it inside the host's document is the
second step, worth reserving room for now.

## 8. The first experiment: E1, authorised under AC3

E1 tests the recommendation's riskiest claim: that two independent Cambium
sessions joined by AccessKit subtrees reach every screen reader as one tree,
with focus and actions working. It needs no other repository.

**Shape.** One host window and two guest sessions with distinct state types,
each with its own document, laid out and composed side by side. Guest A is an
editable text field with IME. Guest B is a list with buttons and one custom
leaf, whose key deliberately collides with a key of A's. Each session's
AccessKit tree is published as a subtree under a graft node in the host's root
tree. *Reading, not ruled:* the two documents' AccessKit ids will coincide,
as two fresh arenas allocate alike, so the subtrees' independent id spaces are
exercised for real. E1a checks it. **Corrected 2026-10-07:** they do not
coincide. Genet's `NodeId` carries a process-wide arena id, so in-process
sessions never share AccessKit ids. The subtrees still carry each session's
focus, route its actions and retire it whole, and coinciding ids do come
from other sources (a host's path-hashed nodes, another process's tree), so
E1a's tree half checks them with model guests.

**E1a, windowless, on any machine.** The updates feed `accesskit_consumer`'s
tree, the layer every pinned adapter uses. Done when:

1. the consumer tree holds every node of the host, A and B, in host reading
   order;
2. with host focus on A's graft and A's focus on its field, the effective
   focus is A's field, and Tab past A's last focusable lands on B's first;
3. a Click request with `target_tree` set to B reaches B's button and changes
   B's count, while the node with the same id in A is untouched;
4. B's button bounds equal its own rectangle offset by B's pane origin;
5. a panic injected in B's dispatch is caught, B becomes unavailable, the next
   update removes B's graft, and A and the host continue, with no dangling
   nodes;
6. the colliding leaf keys paint each guest's own leaf.

Controls, each of which must fail:

- both sessions published into the root tree with their raw ids: nodes
  collide or vanish;
- the graft node omitted: the consumer refuses the subtree (consumer
  `tree.rs`, around line 123);
- today's wrapping traversal: Tab stays inside A;
- no panic catch: the run fails;
- no key namespace: one guest paints the other's leaf.

**E1b, headed, three operating systems.** An operator listens with Narrator
or NVDA on Windows, VoiceOver on an iMac, and Orca on Fedora (Wayland) and Mint
(X11). Done when, on each:

1. the walk reaches host chrome, A's field and B's list to its last item, with
   names read;
2. typing in A is echoed;
3. B's button, activated by the screen reader's default action, changes B's
   visible count;
4. Tab cycles host, A, B and host;
5. composing with a CJK IME in A puts the candidate window at A's caret.

Positive controls: the same build with A's graft omitted, where the operator
confirms A's field cannot be reached, which proves the walk detects an
absence; and on Windows, a node without bounds, which must stop the walk as it
did on 2026-08-20.

**Recorded, no bar:** the host process's memory with no guest, with A, and
with A and B.

**Progress (2026-10-07).** E1a passes on branch `turnstone-session-seam`
(Turnstone U15's lane), windowless, against `accesskit_consumer` 0.35, 0.36
and 0.38, the consumers under the Windows, AT-SPI and macOS adapters:

- *Tree half,* `crates/forme/uxtree/tests/e1a_subtrees.rs`, 27 checks: the
  join itself with model guests whose ids coincide, including one focus move
  into a new guest and between two held guests, and both controls failing as
  they must.
- *Session half,* `crates/cambium/cambium-winit-a11y/tests/e1a_sessions.rs`,
  18 checks: two `RunnerSurfaceSession`s in `ContainedSession`, laid out and
  projected on their own; done-conditions 1 to 6 above, with a shared leaf
  registry as the control for 6.
- *Built for it:* `uxtree::graft` (the AC2 helper), runner and session focus
  exits, `ContainedSession`, per-session leaf registries (Turnstone U17), and
  a public `OwnedLayout::new`.
- *Found:* `Grafts` must order a frame's updates so that focus entering a
  guest is one move, not a stop at the guest's stale focus; and the id
  reading above does not hold for in-process sessions.

E1b, the headed walks, is next and needs Mark at each machine. Its probe is
`crates/cambium/cambium-winit-a11y/examples/e1b_two_sessions.rs` (in Mere:
`Code/testing/mere/` is not reachable from the lane's cloud session). It
publishes a host menu and two `ContainedSession` panes, joined by
`uxtree::graft`, through AccessKit's own winit adapter. `--omit-a` and
`--unboxed` are the positive controls; `--guests` and the printed memory
are the recorded measure; `--self-check` reads the joined tree through the
AT-SPI consumer without a window. Self-check on Linux (debug, windowless):
reading order host menu, A's field, B's list to its last item; Tab cycles
A's field, B's button, host menu; a reader's Click raises B's count; with
`--omit-a` A is unreachable. Resident memory 13 MiB with no guest, 24 MiB
with one, 25 to 26 MiB with two.

*Found for hosts on Genet's winit bridge (Turnstone, Pelt):*
`genet_winit_host::AccessKitBridge` drops an action request's
`target_tree`, and answers activation with the latest update, which under
subtrees is a guest's and is refused as an initial tree. Both block AC2 in
those hosts until Genet's bridge carries the tree id and keeps the host's
initial tree apart; `Composition::initial_host` supplies the latter.

**Where it lives** is part of fork AC3. *Reading, not ruled:* E1a fits as
tests beside the shared helper in Mere. E1b fits as a small binary under
`Code/testing/mere/`, where the stack seams probes live.

## 9. Forks for Mark

Each fork stands alone. The recommendation is listed first.

**AC1. How should one Cambium application appear and work inside another?**
Turnstone already runs Knot's document editor and Redshank's dock in-process,
each as a retained session with its own state and document. Turnstone lays
them out, hit-tests them and joins them into its accessibility tree. The
forest document (stack seams P2) holds one application's windows over one
state type, not several applications. Live pixels from another process need
per-OS GPU sharing (export, import and fences exist together only on Windows),
an IPC protocol, and the guest's whole accessibility tree shipped across as
well.

- **(a)** Same process, each guest a retained session with its own document.
  An app whose authority must stay in its own process keeps it there over a
  Graphshell session, while its presentation crate runs in the host, as Knot
  authoring does now. Live pixels stay for content that is not Cambium (Scry,
  Weld). Commits the stack to closing the session seam's gaps (AC2 to AC4).
  Every composed repository joins the host's tested pin set, which today spans
  five Mere revisions.
- (b) Same process, mounted inside the host window's one forest document.
  Commits to new multi-application forest work in Cambium and a separate
  implementation in Turnstone, whose shell is not rootstock's. Also commits to
  stylesheet scoping, and a guest's edit relayouts the host's window.
- (c) Live pixels from another process as the general mechanism. Commits to
  three per-OS GPU-sharing lanes and to a protocol for frames, input, IME,
  caret, focus and accessibility trees. Gains a process boundary between
  versions.

**AC2. How are the accessibility trees of composed applications joined?**
AccessKit 0.24.1 joins independent trees as subtrees: a tree id per tree, a
graft node in the parent, and actions addressed by tree. The consumer layer
under each pinned adapter implements it (Windows 0.32.1, macOS 0.26.3, AT-SPI
0.18.1). No adapter accepts another process's tree. The stack publishes only
the root tree today. Turnstone joins its contributed panes by hashing paths
into one id space, and no screen reader has walked a contributed pane on any
system.

- **(a)** AccessKit subtrees, through one shared helper in Mere that every host
  uses (Turnstone's panes, rootstock windows, session projections).
- (b) Move Turnstone's path-hash stitching into Mere as the shared helper,
  unchanged.
- (c) Leave each host to join its own trees.

**AC3. Should the first experiment (E1) run, and where?** E1 shows two Cambium
sessions joined as AccessKit subtrees in one window. In its windowless half
(E1a), the consumer layer must show one tree, focus handed across, actions
routed per session and a contained panic, each with a failing control. Its
headed half (E1b) needs a person listening with Narrator or NVDA, VoiceOver
and Orca on a Windows machine, an iMac, Fedora and Mint. Accessibility on all
three systems is Turnstone's first bar requirement, and so far only Windows
Narrator has ever walked a stack application.

- **(a)** Authorise E1a and E1b. E1a goes as tests in a Mere worktree beside
  the shared helper, and E1b as a probe under `Code/testing/mere/`. Windows
  first, then macOS, then Linux.
- (b) E1a only now. The headed walks join Turnstone's accessibility stage
  (S8).
- (c) Not now.

**AC4. How is an in-process guest's failure contained?** A panic in a host turn
is caught nowhere today (stack seams F20), and an admitted session has no
failure state. Catching contains only panics that unwind. A separate process
also contains hangs, aborts and runaway memory.

- **(a)** Catch unwinding panics at every call into a guest session, retire the
  session to an "unavailable" surface and drop its accessibility subtree. A
  hang or an abort still takes the host down.
- (b) Require a separate process for any guest built outside the host's own
  repository. Such guests must then use session projections.
- (c) No containment.

**AC5. Should the stack reserve room now for mounting a guest inside a host's
document?** It would cost three additive changes and no host-side work: a
session can be built at a mount node in a host's document (a public form of
Cambium's internal `build_at`), a guest's stylesheet carries a scope class, and
leaf and producer keys carry a per-session namespace. A session already moves
between windows with its document intact, so tear-out does not need the mount.
Its distinct use is a guest's content interleaved in the host's own layout, or
a selection or find that crosses the boundary.

- **(a)** Reserve the room now, and build the mount when a consumer needs it.
- (b) Build the mount now, beside the session work.
- (c) Nothing until a consumer exists.

**AC6. If a bespoke Cambium surface is later wanted from another process, which
transport does research target?** The reason to want one is that each
application could then build at its own revisions. netrender's paint lists
already have a wire form meant for IPC. They need no GPU sharing and behave
the same on all three systems, but fonts and images must be cached across the
boundary, and custom leaves and producers do not travel as they are. Shared
textures need export, handle transfer and fences per system; only Windows has
those parts. Either transport needs the guest's AccessKit tree shipped and
grafted into the host's tree.

- **(a)** Paint lists plus AccessKit trees. Research only, after AC1's path is
  proven.
- (b) Shared GPU textures plus AccessKit trees.
- (c) Neither. Across processes, the stack composes by session projection
  only.

### Rulings (2026-10-07)

Put to Mark by the Turnstone unusual-protocols lane, as written above, with
notes on where AC2 and AC3 meet that plan's stages SC (one browsing
controller in `pelt-core`) and S8 (page accessibility).

- **AC1.** Mark: **"1, but there is a utility to 2 that has yet to be
  articulated. for now, 1 is good"**. *Follows:* (a), retained in-process
  sessions with authority behind a Graphshell session where it must stay in
  its own process. The forest mount keeps a use Mark has yet to articulate;
  AC5 keeps room for it.
- **AC2.** Mark: **"One mere subtree helper (Recommended)"**: (a). Turnstone's
  page accessibility (S8) and Pelt's combined tree, which Turnstone adopts
  through SC, both use the helper, and Turnstone's path-hash stitching
  becomes a consumer of it.
- **AC3.** Mark: **"E1a and E1b (Recommended)"**: (a). E1a as tests in Mere
  beside the helper, E1b as a probe under `Code/testing/mere/`, Windows, then
  macOS, then Linux.
- **AC4.** Mark: **"Catch panics (Recommended)"**: (a).
- **AC5.** Mark: **"Reserve room now (Recommended)"**: (a).
- **AC6.** Mark: **"Paint lists + a11y trees (Recommended)"**: (a), research
  only, after AC1's path is proven.
- **Owner (Turnstone U15).** Mark: **"This lane (Recommended)"**: the
  Turnstone unusual-protocols lane builds the helper, focus handback, panic
  containment, key namespaces, AC5's reservations and E1, beside its
  `pelt-core` work (Turnstone U11). Record:
  `turnstone/design_docs/2026-10-06_unusual_protocols_browser_plan.md`, §7.
- **Leaf keys (Turnstone U17).** Asked once the lane found that no
  contributed session hands leaves to a host yet: per-session registries,
  namespaced keys in one host registry, or both. Mark: **"Per-session
  registries (Recommended)"**. *Follows:* this refines AC5's "keys carry a
  per-session namespace" for retained sessions. Each session owns its leaf
  registry through the seam and keys stay as written; a namespace returns
  only if forest mounts come to share one registry. Producers stay host-side
  for now: `ProducerRegistry` lives in `cambium-rootstock`, above the seam,
  and holds the host's device.

## 10. Cross-references

- [Stack seams plan](../../mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md):
  P2 and rulings S23 to S31; F16 to F20.
- [One state, N windows design](../../mere_docs/design/2026-07-05_one_state_n_windows_design.md).
- [Cambium architecture](../technical_architecture/2026-09-03_cambium_architecture.md):
  external viewport producers; Mesquite paint sidecars.
- [Dynamics grammar plan](../../mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md):
  G9, F65 (child buttons for actions).
- [Tear-out composability plan](../../archive_docs/2026-07-04_completed_plans/2026-06-19_tearout_composability_plan.md),
  [tear-out gestures plan](../../archive_docs/2026-10-06_completed_plans/2026-06-24_tearout_gestures_plan.md),
  [tear-out operations brief](../../mere_docs/research/2026-05-11_tearout_operations_brief.md).
- [Native-surface compositing plan](../../archive_docs/2026-07-03_completed_plans/2026-06-19_native_surface_compositing_plan.md):
  the split of host surfaces by nature; finding 5 on input by API.
- [Port GUI composition brief](../../mere_docs/research/2026-09-02_port_gui_composition_and_comparable_stacks_brief.md)
  and [family composition thesis](../../2026-08-12_family_composition_thesis_brief.md)
  §3: Scenograph is "present in every application with its own graph",
  holding a lens and never truth. It is the standing example of a component
  every host compiles in, which makes it a B1 or B2 citizen under this brief.
- [Knot shared surface and port contribution plan (archived)](../../archive_docs/2026-10-06_completed_plans/2026-08-24_knot_shared_surface_and_port_contribution_plan.md):
  the contribution contract frozen at v1.
- [Postcard framing plan](../../mere_docs/implementation_strategy/2026-10-06_postcard_framing_plan.md):
  the framing a cross-process wire would use.
- Turnstone: `turnstone/design_docs/2026-10-06_unusual_protocols_browser_plan.md`
  (§4, S11, U2, U8); `turnstone/design_docs/2026-08-20_screen_reader_pass_receipt.md`;
  `turnstone/docs/receipts/browser_scry_windows_20261005/README.md`.

## 11. Current consumers and design direction (2026-10-09)

**Status:** read-only reconciliation against Mere `6708bfcd0`, Turnstone
`25f85bd`, site `9ba3f03` and the available Tabard and Moot chats. Recorded
qualifications were read; no runtime or physical assistive-technology gate was
rerun. Historical findings above describe the earlier assessed sources.

The [design language](../../2026-08-23_projection_scenes_and_graph_native_platform.md#9-configurable-visual-and-interaction-language-2026-10-09)
joins the workbench and dataspace as presentations of the same domain. Opening
may focus a retained tile, unfold an applet or use an overlay. Selection,
activity and presentation remain independent. Its forme field represents the
recursive workbench arrangement; that direction does not require every embedded
surface to become a graph node or every guest to share the host's document.
AC1's retained sessions and AC5's reserved forest mount remain as ruled.

Three existing paths can participate, with different ownership boundaries:

| Path | Existing authority and qualification boundary |
| --- | --- |
| Retained Cambium session | The guest keeps its state/document; the host admits it and composes input, focus, leaves and accessibility through the shared seam. AC1–AC5 and E1 remain the governing choices. Tabard's Woodshed adoption is a concrete native embedding precedent; its visual receipt does not complete E1b's screen-reader walks. |
| Owner-served Graphshell projection | Domain/session authority remains with its owner; the host presents disclosed state and returns authorized intents. The browser carrier and reservoir plans own delivery and access. A projection does not transfer custody merely because it opens beside a local session. |
| Verified Wasm applet | The [Moot capsule-library proof](../../moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#capsule-library-composition-proof-2026-10-09) exercises a signed WIT component over host-disclosed data and local execution grants in native and browser example hosts. Its Graphshell mount was published on main at `3055ae5af` later on 2026-10-09; independent local readings are the bounded continuation recorded in that plan. This catalogue/reader host dialect is not a universal Cambium surface ABI or a release-qualified embedded product. |

The Moot example separates Gemot collection/contribution/hosting authority,
signed content, participant retention and local applet execution. Its browser
host reads a disclosed snapshot; it does not replay all Gemot operations.
The host rechecks proposals against disclosure and current grants, and owns
presentation and mount lifetime. Those boundaries complement AC1's trusted
in-process presentation crates; they do not establish equivalent isolation or
input/accessibility behavior. Canonical proof details and checks remain in the
Moot plan rather than being copied here.

Turnstone `25f85bd` records the current SC/S8 continuation. Its own AccessKit
adapter preserves tree-aware requests and ordered guest batches; the earlier
unconditional Genet-bridge prerequisite therefore does not block Turnstone
source integration. The pinned Genet bridge still needs the repair for hosts
using it, including standalone Pelt. Live document projections/actions,
contributed-tree migration, shared scenario observation and physical S8/E1b
acceptance remain open. The qualified controller-pumping cut advances retained
clocks while suppressing hidden redraw; SC6's engine-aware pool and repeated-
placement contract remain separate. Read the consumer's plan and handoff before
resuming its work, rather than reconstructing ownership from this brief's
historical findings.

For the site, [ambient relation lenses](../../mere_docs/design/2026-09-23_ambiance_design.md#10-proposal-focus-driven-relation-lenses-2026-10-07)
need disclosed identities, reasons and coverage across all these presentations;
they do not need arbitrary app execution to show dependencies. Portable article
content opened in Knot or Turnstone can be a bounded composition proof before
a full browser application is delivered. A static site, a browser-local session,
an extension sidecar and a native-owned attached session have separate storage,
capability and lifecycle boundaries. Existing owner plans retain those choices:
`merelyllc.com/docs/2026-09-30_graphshell_site_canvas_plan.md` and
`turnstone/design_docs/2026-10-06_unusual_protocols_browser_plan.md`.

### 11.1 Renderer choice for composed applications (2026-10-10)

**Status:** source assessment and upstream review; no CPU/Hybrid application
parity or automatic fallback is qualified. The pinned Netrender `9607d16f`
exposes optional `vello-cpu` and `vello-hybrid` adapters. Its shared sparse
lowering admits geometry, gradients, transforms, clips and layers, but refuses
image/pattern hydration, glyph resources and retained fragments. Graphshell's
current host uses Classic; merely enabling the optional features cannot render
its full application scenes. These are integration limits, not limits of the
upstream renderers. The three coherent Classic patch crates (`netrender-vello`,
`vello_encoding`, `vello_shaders`) are distinct from the three renderer choices.

[Upstream's current renderer guidance](https://github.com/linebender/vello#choose-a-renderer)
identifies CPU as its most mature software renderer and Vello GPU as a CPU path
preprocessor with GPU raster/compositing that does not require compute shaders.
The original compute renderer remains experimental under `research/`. Our
pinned sparse GPU adapter still uses the Hybrid name. This source assessment
does not update pins or establish compatibility with current upstream APIs.

The bounded follow-up is to hydrate the same Scene's text, image and retained
resources for the optional adapters, then compare a real Cambium application
through CPU output and the independent sparse GPU path. Explicit backend
admission must refuse unsupported scene operations rather than omit content.
Qualify fonts, images, clips, composition, resize and input/accessibility against
that shared scene before routing ordinary applications to another backend.
CPU raster output can provide an independent visual control and export path;
host upload/presentation and recovery after a driver reset need separate
qualification. Backend selection must preserve guest/session authority, drafts
and selection; it must not turn a rendering failure into a new applet grant.
