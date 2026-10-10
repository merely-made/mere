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

## The work, in order

1. **The dynamics slot** (dynamics grammar plan, `mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md`, F192 to F195; asked for by the Scenograph plan's SE69).
   - F192: `scenograph::AuthoredProjectionDefinition` gains `dynamics: Option<DynamicsSlot { version, spec }>`, the spec as canonical JSON. scenograph stays on sceno and serde and keeps `Eq`; no seiche dependency. The binding host reads the spec through seiche and refuses by path.
   - F193: the recipe's `arrangement.kind` and the spec's `target.arrangement` must agree; binding refuses a mismatch, naming both paths.
   - F194: `ProjectionVariant` varies dynamics by a catalog preset id or a whole `DynamicsSlot`, one or the other (naming both is refused, a reading not yet ruled: confirm with Mark).
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

The nested branch's 2026-10-10 checkpoint combines the committed site cutover,
Moot, shared capture polling/Genet `7422e906`, published Tabard appearance and
the guarded renderer publication through `5cea11408` (published main
`11236fd4f`, Vello `491c376c`). The guard preserves the earlier Radeon coarse
repair. Its 141 affected CPU tests, locked viewer build and main-page/applet
Wasm check pass after the renderer update; the unchanged shared scene and
exporter inputs retain their 254 and four passing tests. Main landing remains
pending: the iMac's confirmed Radeon reset and
WindowServer incident has put GPU/capture runs on hold, and seven Rootstock
GPU fixtures failed to obtain a device. The
[Fold receipt](../../../ports/graphshell/docs/receipts/host_dataset_folds.json)
records current hashes, historical partial receipts and the exact remaining
headed checks. The site lane's uncommitted docs/scenarios remain its own work.

## Where things are

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
