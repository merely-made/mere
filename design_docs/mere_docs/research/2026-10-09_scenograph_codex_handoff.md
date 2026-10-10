# Scenograph handoff to Codex: S2, B1, and the dynamics slot

**Date:** 2026-10-09
**Status:** open: handed to the Codex agent by Mark (Scenograph editor plan, SE87).
**From:** the Scenograph editor lane (Claude). **For:** the Codex agent.

Three pieces of ruled work, each with done-conditions in its plan. Read the
plan sections first; they hold Mark's rulings verbatim, and those govern.

**Context refresh (2026-10-09):** Mere `6708bfcd0` includes the incoming
[design language §9](../../2026-08-23_projection_scenes_and_graph_native_platform.md#9-configurable-visual-and-interaction-language-2026-10-09).
The work order below stands. The [ambiance proposal](../design/2026-09-23_ambiance_design.md#10-proposal-focus-driven-relation-lenses-2026-10-07)
and [app composition reconciliation](../../cambium_docs/research/2026-10-06_app_composition_brief.md#11-current-consumers-and-design-direction-2026-10-09)
carry the wider site and cross-app context; neither is a receipt for these tracks.

**Voice continuation (2026-10-09):** [design language §9.8](../../2026-08-23_projection_scenes_and_graph_native_platform.md#98-voice-clarifications-and-research-boundaries-2026-10-09)
clarifies that deselection preserves edits, welcomes hover previews, expands
fields toward scoped projections and entry-triggered behavior, calls for
granular configurable font roles and meaningful visible dynamics, and records
the need to compose theme values with semantic channels. Owner plans carry
research inputs and candidate proofs. Existing work order and viewer ownership
stand; site Rulings 161–162 below replace R160. This is not qualification of the shared checkout's
concurrent dynamics implementation. The subsequent [primitive planning
direction, §9.9](../../2026-08-23_projection_scenes_and_graph_native_platform.md#99-plan-through-graph-primitives-2026-10-09)
asks for nodes, links and fields to be inspectable/configurable and connected
to dynamics. Graphshell is the reference host; the proposed terminology needs
reconciliation before a new proof is opened.

**Planning continuation (2026-10-09):** the editor plan's
[primitive and dynamics map](../implementation_strategy/2026-10-07_scenograph_editor_plan.md#primitive-and-dynamics-map-2026-10-09-source-backed-planning)
maps existing surface/Resource, relation and field/coupling identities to
authoring controls and dynamics. It records source gaps and proposed
done-conditions without taking over the active dynamics implementation or
opening a new track. The field plan corrects the earlier assumption that a
coupling selector already means membership in the field's extent.

## Ruled continuation, 2026-10-10

F200 to F202 are recorded verbatim in the dynamics grammar plan, committed
as `38cdebca`. The carrier, fixed-step runner, occurrence adapter and dynamics
matrix are committed as `e2543cee`; `40aac7fc` integrates upstream Mere
`6183006b`, preserving Tabard's host stylesheet alongside live dynamics.
That integration also adopts upstream's Genet `7422e906` pin. The subsequent
`601b6dd1` merge adopts Vello `491c376c`; `3d40d546` adds the upstream workshop
header and appearance receipts without moving the qualified physics sources.

- Direct coordinate and embedding families hold their encoded data axes;
  Timeline holds its continuous x position. Grid ranks, ordering, layers,
  buckets and ring slots remain free. With both axes encoded, F47 permits
  contact-only separation. A one-axis position-writing law refuses by path
  until an adapter can preserve that coordinate; unknown solvers likewise
  need a disclosed constraint adapter. Regions, relations and folds still
  require a broader scene-input adapter.
- A variant carries a preset or a complete spec. Naming both is refused (F201).
- The host's editable preview limit starts at 60 (F202). Capped cells visibly
  say "Step limit reached". The matrix pages across the complete axes;
  only its working cell advances. Paging does not rerun settled cells.
- The six focused native matrix tests pass, including the held-coordinate
  test and its moving-grid control. The pre-integration paged scenario passes
  34 steps / 55 frames in the in-app browser at 520×604 and 1280×720, and in
  standalone Chrome at 1037×583. These receipts precede the final stylesheet,
  refusal-card and newer-Genet qualification. Firefox remains open.

**Native and identity qualification:** the full native Graphshell web library
passes on Genet `7422e906` (194 passed, 5 ignored, before the final caption
changes). The six comparison tests pass after `72ce892d`, including the live
working-state label and unchanged snapshot states. Scenograph passes 16,
Scenomise 157, and Seiche 159 (10 ignored). Pictograph's full single-thread run
passes 381 with 18 ignored and one frame-timing failure under concurrent build
load. That speed test passes its targeted rerun with the original tolerance;
the loaded run is retained as `dynamics-pictograph-loaded-run.txt`.

`Code/testing/mere/scenograph-editor/receipts/dynamics-identity-ruled.json`
records all twelve presets twice per target at 0, 59, 60 and 4,000 steps:
native and Node wasm agree exactly, including float words and stop results.
Six laws change positions between 59 and 60 (the control). The direct-coordinate
probe agrees across targets and keeps its coordinates at 0 and 60. The receipt
names source `72ce892d`, both lock hashes, source and binary hashes, and Node
24.11.0; covered source stayed unchanged during the run. Its covered hashes
also match after the presentation-only `34539624` contrast correction, which
uses the existing on-tertiary theme role for selected captions. The final wasm
host builds on Genet `7422e906` and Vello `491c376c`. This supersedes the older
identity measurements below. Measured wasm runs average 3.85 s at 60 and
42.27 s at 4,000, including startup and concurrent system load; these are
three-occurrence fixture receipts, not a large-scene performance claim.

**Browser qualification:** the final 48-step scenario passes over 77 frames
with four captures in standalone Chrome at 1037×639 and the in-app Chromium
browser at 520×604 and 1440×900, with no errors or gate failures. The whole
captures were inspected: paging leaves captions readable, capped snapshots
say "Step limit reached", refusal cards name the missing input adapter, and
the applied working frame is brought into view with its current live state.
The selected captions now use the existing on-tertiary role; all six native
appearance tests pass after that correction. Final browser receipt names are
`chrome_dynamics-contrast-chrome-scenario-receipt.json`,
`chrome_dynamics-contrast-narrow-scenario-receipt.json`, and
`chrome_dynamics-contrast-desktop-scenario-receipt.json` in the receipts folder.
`dynamics-qualification.json` records their hashes, the browser package and
scenario hashes, source bases, the native results and the unchanged physics
source control. Firefox is unavailable in this session's browser control
inventory and retains its separate qualification gate. S1's final acceptance
and the physics coordinator's review remain open.

**Cleanup:** the receipt server is stopped, the temporary Node bindings are
removed, and the viewport override is reset. Receipts and the reproducible
probe remain in the existing testing directory. No isolated target, Cargo home
or worktree was created. The shared `C:/t/cargo-targets/mere` target and browser
package are retained for Mere's ongoing work and the open Firefox gate.

**B1 backend continuation (2026-10-10, draft):** the graph canvas now has a
portable backdrop binder with a separate static-obstacle set. Its resolved
geometry supplies both camera-aligned paint and contact; visibility and
collision stay independent. The binder preserves the previous layer on a
path-specific refusal. Unsupported contact paths, points and concave polygons
need a further footprint adapter. Living scene bodies, gravity and node
tangibility are retained when the obstacle set changes.

SE90 in the editor plan supersedes the proposed mode/control fork: reset is
an action, ambient context and props can coexist, and visibility is independent
of collision, animation, picking and authored behavior. The static binder is
one component of the set, not an interactive scenery implementation. SE91
resolves membership: selected or foreground-pinned nodes stay foregrounded;
after deselection, interacted nodes recede but remain until dismissed.
Untouched ambient suggestions may change with context. Reset's baseline and
the interaction boundary for retention remain open. R162's viewer ownership
has not been released.
No reserved viewer files have been edited by this continuation. Scene state,
viewer integration and Chrome/Firefox checks are still open. The retained
`backdrops-idle-overlap.txt` probe retains the short 120-step, idle-body
failure. Rapier caps correction at 3 world units per second. The
active-law/control check uses the canvas's existing 360-step settle budget;
larger/deeper or idle placements remain a qualification limit. No exclusion
tolerance or host settle budget changed. All four focused native binder tests
pass, and Seiche's library suite passes 161 with 10 ignored. The locked
standalone viewer wasm compile check passes on Genet `7422e906` and Vello
`10f01d6d`. This backend draft neither closes B1 nor refreshes the earlier
dynamics identity receipt.

B1/S2 continue through the viewer owner assigned by site R162. The concrete
next slices are: reconcile B1's reset/context/set state with the existing
owners under SE90, then viewer backdrop drawing, independent controls and the
tangible obstacle proof with its intangible control; then S2's clause selection, per-appearance
visibility, shelfmark mapping and scenotime replay. SE88's UI name is
"Appearance part"; a spelling for the replacement shelfmark key remains a
reading, not a separate ruling. The site's current selection semantics must
be preserved during that owner's cutover. F199's broader queue stays paused.

## The work, in order

1. **The dynamics slot** (dynamics grammar plan, `mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`, F192 to F195; asked for by the Scenograph plan's SE69).
   - F192: `scenograph::AuthoredProjectionDefinition` gains `dynamics: Option<DynamicsSlot { version, spec }>`, the spec as canonical JSON. scenograph stays on sceno and serde and keeps `Eq`; no seiche dependency. The binding host reads the spec through seiche and refuses by path.
   - F193: the recipe's `arrangement.kind` and the spec's `target.arrangement` must agree; binding refuses a mismatch, naming both paths.
   - F194: `ProjectionVariant` varies dynamics by a catalog preset id or a whole `DynamicsSlot`, one or the other (naming both is refused under F201).
   - F195: SE70's settle is a seiche function, `settle(spec, inputs, bound) -> positions`, every host calls, with a native-against-wasm identity receipt. The default step bound for a spec with no stop rule goes back to Mark as a measured number.
2. **The grid's dynamics axis** (Scenograph plan SE54, SE55, SE70, track S1's last open item). After the slot: `scenograph::swatch::AxisKind` gains `Dynamics`; the projection editor's comparison (`ports/graphshell/src/projection_compare.rs`) can vary it; settled cells call F195's settle; the focused cell runs live (SE55).
3. **B1, backdrops in the Graphshell viewer** (Scenograph plan §2 B1, SE86). Independent of the others.
4. **S2, linked swatches** (Scenograph plan §2 S2, SE84, SE85). The sandbox state was read directly at site `9ba3f03` and is recorded below. Refresh it against the site's current source before mapping, and put to Mark the new name for appearance-part selection (SE84) before writing it.

## Site state and design constraints, checked 2026-10-09

Canonical consumer record: `merelyllc.com/docs/2026-09-30_graphshell_site_canvas_plan.md`,
R158's SE84–SE86 amendment and R160. The checked implementation is
`merelyllc.com/assets/graph-sandbox.js`, especially `setSelectionClause`,
`selectFacet`, `dismissInstance`, `applyCoordinatedSelection` and `sceneState`.
These are existing consumer shapes to preserve, not proposed portable schemas:

- `selection = { resolution: "crossfilter", clauses: { ... } }`; each named
  clause has `role` and `targets: [{ kind: "node", id }]`. Spatial selection
  uses `focus`; matrix uses `brush`. Crossfilter intersects foreign clauses,
  excluding a view's own clause; no foreign clauses means unfiltered, whereas
  an empty intersection means no matched items. Scatter/deck currently select
  through the spatial clause, rather than having independent producers.
- Appearance addresses carry `{ view, source: { adapter, id }, facet }`;
  current parts include `node`, `cell`, `heading`, `summary` and `status`.
  `facets` is a selected-address list. A matrix-cell click addresses its
  column source's `cell` part; it is not an independent row/column pair identity.
- `instances` holds `{ instance: <appearance address>, visible: false }` for
  dismissed appearances. Dismissal preserves the source and its other
  appearances. Backdrop state is `{ kind, collidable }`; kind is `clear`,
  `ambient`, `props` or `field`. B1's ruled first cut still excludes field.
- The shelfmark deltas use `selection`, `mer3ly.instances`, `mer3ly.facets`
  and `mer3ly.backdrop`. The current site carries JSON text for these values.
  S2 must migrate the part-selection key under SE84's ruled future name and
  preserve the consumer's existing unknown-section/refusal behavior.

The design language §9.2 requires previously selected nodes to remain in the
background until dismissed, while untouched context can change with lenses.
This is attention/view curation; it is not durable keeping, a position pin or
source truth. S2's linked selection and appearance visibility are mechanisms
it can use, not a completed ambient reason controller. The new forme field
represents the recursive tile arrangement (§9.4); it does not settle B1's
deferred field physics or the layout-lock gesture.

Moot's capsule-library proof is now on published main (`f22205e4e`, followed by
`0792da6a9` and `3055ae5af` integration fixes), with an opt-in applet mount in
`web_tree.rs` and `web_tree/applet.rs`. Its canonical
continuation is the [Moot plan](../../moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md#capsule-library-composition-proof-2026-10-09).
Site Rulings 161–162 supersede R160: the nested branch adapts its groups to
`sceno::Fold`, and the site lane owns `web_tree*`, `host_dataset_view.rs` and
`web_dataset.rs` during cutover. The branch owner, Moot and the editor S2/B1
lane coordinate overlapping edits through that site lane. Native fold Rulings
163–166 belong to its separate S5 continuation. This handoff assigns none of
those implementations to the editor lane.

The nested branch's 2026-10-10 acceptance closes the portable Fold and combined
viewer slice for main integration. Source `8b8bd92b1` combines the committed
site cutover, Moot, shared capture polling/Genet `7422e906`, Tabard appearance,
maintained Vello `10f01d6d` and published Mere `0a3203f05` (including the
occurrence-dynamics work above). Both Cargo roots resolve the maintained
renderer/encoding/shader triple to the atlas repair, retaining the earlier
Radeon coarse and native-compute guards. The legacy compatibility encoding
remains separate.

All 254 shared scene tests, 141 affected CPU tests and seven serialized GPU
producer tests pass on this integration. Four unchanged exporter tests retain
their result. The locked viewer build, main-page/applet compile check, portable
metadata and header gate pass. Headed Chrome `152.0.7977.83` on the default
`amd` / `gcn-5` adapter passes nested/camera/flat/fold-history/gesture receipts,
all 15 site checkpoints and their keys, clean unknown-membership refusal,
a planted propagation control, narrow keyboard recovery and the plain viewer's
mount and actual page-wheel check. Actual captures were inspected; control
density and small glyphs remain usability concerns. The
[Fold receipt](../../../ports/graphshell/docs/receipts/host_dataset_folds.json)
records fresh source and bundle hashes, commands and capture paths, preserving
superseded device-loss and harness failures as history. Browser and receipt
server are stopped; the coordinated GPU slot is released.

Main subsequently advanced to `35ad305af` during the push. Its published
Dramatis/custody move and kernel component-copy repair are integrated, with
the separate viewer root aligned at `7d78b9960`. The scene and affected CPU
suites, both locked Wasm gates, metadata and header checks pass again. Renderer
and directly recorded viewer/fold/history/gesture/framing inputs are unchanged.
The receipt retains the headed source/bundle at `8b8bd92b1` and names the
post-publication CPU/build source separately; no further GPU run is claimed
after slot release.

This acceptance does not close native S5, saved ambient scenes, full compiled
Cargo closure or live-site adoption. R162's site owner continues those assigned
viewer edits; its uncommitted docs/scenarios and the primary checkout's five
unpublished commits were untouched. B1/S2 and the dynamics lane retain their
own done-conditions and acceptance boundaries above.

## Historical draft checkpoint, 2026-10-09

**Codex checkpoint (2026-10-09, implementation draft):** SE88 names the UI
concept "appearance part"; SE89 gives every occurrence a separate transient
physics body and refuses undisclosed channels. The carrier, preset/full-spec
variant binding, fixed-step settle loop, and Arrangement/Dynamics matrix are
being implemented. Native checks cover repeatability, a changed-bound control,
independent bodies for a repeated source, display refresh without restarting
motion, and one undo for a comparison pick. Native suites pass: Scenograph 16,
Scenomise 157, Seiche 159 (10 ignored), Pictograph with canvas 381 (18 ignored),
and Graphshell's no-default-features web library 187 (5 ignored). The full
browser host builds for wasm. The encoded-position interpretation, variant
precedence and measured default bound remain open, as do the browser and
readability qualification gates described below.
The broader F199 pause remains in effect.

**Final draft receipts:** `Code/testing/mere/scenograph-editor/receipts/dynamics-identity-final.json`
records two fresh runs per target for all twelve presets, with exact float-word
identity between native and Node wasm at bounds 0, 59, 60 and 4,000. The 59/60
control changes positions for six laws. It records source/binary/lock hashes;
source was unchanged during the probe. `dynamics_identity.py` beside the sink
server reproduces the probe after building the native example and generating
the Node wasm bindings. These are three-occurrence fixture receipts, not a
large-scene performance claim. The earlier `dynamics-identity.json` is superseded
because its attempted Anneal tick-demand change was removed.
The standalone web lock is ignored by Git and was regenerated against its
existing manifests after the stale lock could not satisfy Rapier's glamx
requirement. Its transitive resolution changed; no Genet manifest pin was
changed by this lane. The two lock hashes in the final receipt identify the
actual graphs used, rather than claiming a one-entry dependency update.

The corrected `projection_dynamics.scn` passes 27 steps over 46 frames in the
Codex in-app Chromium browser at 520×604 and 1440×900, with two captures per run
and no reported errors or gate failures. The sink labels this browser "chrome";
it is not a standalone Chrome receipt. Desktop headings are readable, but the
narrow matrix overlaps labels and the desktop footer covers part of the last
row. Capped-state wording is present in semantic buttons but needs visible
treatment. Standalone Chrome and Firefox remain unqualified. The receipt names
are `chrome_dynamics-final-scenario-receipt.json` and
`chrome_dynamics-desktop-scenario-receipt.json` in that receipts directory.

The final twelve-preset probe measured about 0.96 s per wasm run at 60 ticks and
14.79 s at 4,000; these include process/module startup and concurrent workspace
load. The earlier 0.30/13.84 s measurement is superseded. A configurable default
of 60 ticks was recommended from this receipt and installed under F202; a capped
cell is "Step limit reached", not "At rest".

**Stop-condition correction:** an attempted Anneal tick-demand change failed
the existing Seiche speed/settle contract and was removed. The occurrence
preview now reads the catalog's derived currencies. A kinematic or resident
law does not acquire an RMS-velocity rest claim; explicit schedule stops and
existing completion signals remain usable, otherwise the caller's step limit
ends the snapshot. Anneal's own completion is not qualified by this lane.
This does not change the shared Canvas runtime's ordinary settle budgets.

**Design-language reconciliation:** upstream §9.8/§9.9 keeps resource identity
separate from multiple appearances and makes deselection preserve edits.
The draft uses one body per occurrence and refreshes presentation without
restarting a reused placement. A footprint change rebuilds the preview,
because measurement is a physics input. Foreground attention, positional
pins and layout locks remain distinct. Fields and relationship explanations
do not install forces implicitly; their broader proofs remain with their
owners. Those three questions were resolved as F200 to F202 on 2026-10-10; the checkpoint above supersedes their pending state. Consumer qualification remains separate from implementation.

**Viewer ownership refresh:** the site source was refreshed against upstream
`87d7a3d`. Ruling 161 replaces the deferred grouping choice with folds; Ruling
162 assigns `web_tree*`, `host_dataset_view.rs` and `web_dataset.rs` to the site
lane for its cutover. B1/S2 edits to those files must go through that owner.
The one `host_dataset_view.rs` change in the dynamics draft is the required
`dynamics: None` initialization for the expanded recipe type. No new viewer
behavior has been installed. The site's upstream sandbox still carries
`mer3ly.facets`; SE88's key migration has not been applied.

**Upstream check, 2026-10-10:** fresh fetches report Mere `773a0dc2` and site
`87d7a3d` on `origin/main`. The viewer's Forme bridge `cf4f4c51` and reconciliation
`70fb3aa4` are published. The reserved viewer files in local main `d572b322`
match Mere upstream exactly, with no local edits at this checkpoint. The site
plan still reserves those files under R162; committed/pushed work does not
constitute an ownership release or proof that the cutover is complete. The
B1 backend `e386e827` is local and is not an ancestor of published main.

**SE90 source map:** portable `sceno::Backdrop` carries provenance, transforms,
geometry, visibility and collision, but no behavior or item-picking identity.
Pictograph's `AmbientSim` advances decorative simulations and paints them; it
does not mean contextual graph nodes and has no picking or reset contract.
Seiche's `SceneSpec` already supplies fixed/dynamic bodies, initial velocities,
sprites, joints and perpetual motion, with tangibility separate. These are
existing capabilities, not a unified portable set adapter. Graphshell's
"Restore arrangement" calls `Canvas::restore_arrangement`, which restores
stored placement and pauses physics without resetting scenic body, animation
or script state. Use these seams to define the next bounded slice rather than
turning the sandbox's mode labels into the scene model.

**Workspace rules for this continuation:** Mark's current workspace
instructions supersede the historical worktree/build-directory paragraph below.
Use main unless an actual concurrent collision requires isolation, and reuse
`C:/t/cargo-targets/mere`. No dynamics worktree or isolated Cargo home has been
created.
The temporary browser tab and receipt server were closed, and the generated
Node bindings were removed after the identity gate. Receipts and the reusable
probe remain under the existing testing directory. The shared Mere target and
the browser package remain available for the open qualification gates.

- Swatch types: `crates/cambium/scenes/scenograph/src/swatch.rs`. Composition: `crates/cambium/scenes/scenomise/src/facet.rs` (`compose_facet`), tests in `facet_tests.rs`. Subgraph specs: `crates/forme/curation` (published as `mere-curation`).
- The editor's grid: `ports/graphshell/src/projection_compare.rs` and `web_projection.rs` (`compare_targets`, `toggle_projection_compare`, `pick_projection_compare`); headed scenario `ports/graphshell/web/scenarios/projection_compare.scn`.
- The catalog's addition records: `mere_docs/research/2026-08-15_projection_grammar_catalog.md`, "Addition records". Each promoted primitive needs one.

## How this lane works (Mark's standing rules)

- **Forks go to Mark.** Any decision with more than one defensible answer becomes a question with evidence and options, recommendation first. Record his answer verbatim as the next unused numbered ruling in the owning plan and commit it the same turn. Check the current plan first: SE87 and F199 are the latest at this context refresh, so the next numbers are SE88 and F200. Mark anything beyond his words *Reading, not ruled*.
- **Evidence before claims.** Check code, counts and paths before writing them down; when evidence contradicts a ruling, reopen it with Mark.
- **Worktrees.** Work in a worktree off `origin/main`; push only your own commits as a fast-forward; never bare `git stash`. Each worktree builds in its own directory (F183): a gitignored `.cargo/config.toml` with `[build] build-dir = "C:/t/cargo-build/mere-<lane>"` and `[profile.dev] debug = 0`. At most three cargo builds at once.
- **Gates.** Tests pass, controls fail where they should, and a gate reruns only when its inputs moved. Headed checks run in Chrome and Firefox: build `ports/graphshell/web` for `wasm32-unknown-unknown`, `wasm-bindgen --target web --out-dir pkg`, serve with `Code/testing/mere/scenograph-editor/sink_server.py <port> <web dir>`, and open `index.html?scenario=scenarios/<name>.scn&sink=http://127.0.0.1:<port>/scenario-receipt`; receipts land in `Code/testing/mere/scenograph-editor/receipts/`. A hidden browser tab gets no frames: keep it visible. Look at the whole screenshot, not only the feature.
- **Docs.** New active docs need a `DOC_README.md` entry and a D2 record (`support/doc-audit/d2/`, checked by `scripts/mere_doc_judgment_audit.py`). Don't reformat whole files that were unformatted on main.
- **Clean up** each lane's worktree, branch and build directory when it lands.
