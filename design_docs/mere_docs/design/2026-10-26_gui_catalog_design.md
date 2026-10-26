# The GUI catalog: surfaces by concerns, and the loops between them

**Date:** 2026-10-26
**Status:** design record, 2026-10-26. Cues only: what the stack's existing
catalogs cover, what a complete catalog would have to add, and the loops a
catalog has to close. No ruling, no code, no work assigned.

**Origin.** Mark, 2026-10-26, thinking aloud: "what a complete gui catalog
would look like... considering all these distinctions... graph/canvas, card,
pane, tile, workbench tile tree, history, persistence, actors, workers,
resources, engines, analysis and generation, networking, transports, resource
sharing, community, encryption, identity, asynchronicity, mesh computation...
but there are a lot of loops to tie up!" and "i probably forgot things too".

**Reading.** Most of that list is not GUI nouns. It is the set of concerns that
need a GUI face. The surfaces are the rows, the concerns are the columns, and
the catalog is the product of the two. The stack already holds six partial
catalogs, each one row or one column of that product, written from one
authority's point of view, and none cross-references another (§1). The loops
Mark names are the places where two of those catalogs claim the same cell (§4).

**Related:** the [composition spine](../technical_architecture/2026-05-21_mere_composition_spine.md)
(arrangement ontology), the
[interaction model spine](../technical_architecture/2026-06-18_interaction_model_spine.md)
(the pipeline), the
[design language record](../../2026-08-23_projection_scenes_and_graph_native_platform.md#9-configurable-visual-and-interaction-language-2026-10-09)
(selection, foreground, opening), the
[stack modularity map](../technical_architecture/2026-10-25_stack_modularity_map.md)
(which engines a build links), [TERMINOLOGY](../../TERMINOLOGY.md) (the nouns).

## 1. The catalogs that exist

Each is real and useful on its own axis. The gap is between them.

| Catalog | Axis | Where | What it enumerates |
|---|---|---|---|
| Cambium component catalog | widgets, executable | [component-catalog](../../cambium_docs/technical_architecture/component-catalog.md), `crates/cambium/cambium/examples/component_catalog.rs` | button through data grid; command surfaces as palette, picker and context menu with disabled reasons; overlay; detail popover; selection bars and filter chips; pane shell and `setting_row` with empty, error and unavailable states; graph-canvas swatch; paint leaves |
| Projection grammar catalog | graph visuals | [projection grammar catalog](../research/2026-08-15_projection_grammar_catalog.md) | readings and operators, marks, visual channels, guides and backdrops, relation forms, arrangement families, interaction and change over time |
| Frame-tree surfaces | roles, and panes by primary authority | [TERMINOLOGY, Surfaces](../../TERMINOLOGY.md#surfaces-frame-tree), consolidating the [frame taxonomy](2026-06-07_graph_roster_and_frame_taxonomy.md) §3 and the [peripheral panes](../technical_architecture/2026-06-06_peripheral_panes_architecture.md) dock | frame; pane as roster, gloss, inspector, apparatus, comms, plus Index and Steward from the panes doc, grouped there by graph, runtime or social authority; tile and the workbench tile tree; shellbar |
| Card family | summoned surfaces | [node card summoning](2026-07-01_node_card_summoning_design.md) §2 | preview, unvisited, connections swatch, object card |
| Capability-gate catalogue | refusals | [capability gate catalogue](../research/2026-05-14_capability_gate_catalogue_brief.md) | named gates, the policy chain (its chain superseded 2026-05-30 by the narrowing rule) |
| The kernel's own surface list | UX event sources | `crates/graph/graph-kernel/src/accessibility.rs`, `SurfaceId` | Omnibar, CommandPalette, NodeFinder, ContextMenu, ConfirmDialog, NodeCreate, FrameRename, StatusBar, TreeSpine, NavigatorHost, TilePane, CanvasPane, BaseLayer, RosterPane, InspectorPane, StewardPane, GlossPane, ApparatusPane, CommsPane, WorkbenchPane |

Four readings of the table:

- **The orrery is not a row.** Mark, 2026-10-26: "the orrery is just a
  projection of the graph." TERMINOLOGY narrowed it to a form factor on
  2026-07-26, a way a mere is rendered, and the grammar catalog already lists
  "orrery or node-link" as a projection family. The June frame taxonomy and
  the panes catalog were two readings of one frame tree, which TERMINOLOGY's
  Surfaces section already consolidates; the table cites that section and
  keeps the June docs as its sources.
- **The rows exist in different degrees.** `SurfaceId` names twenty surfaces
  because each is an event source for accessibility, not because each is
  built. As read on 2026-10-26, four of the twenty are emitted at real seams
  (CommandPalette, NodeFinder, ConfirmDialog, ContextMenu); the pane variants
  appear only in the kernel's descriptor table and ux-events' diagnostics map,
  with no open or dismiss seam. The panes themselves are built unevenly and
  outside the Graphshell port: Comms is a view-model crate
  (`crates/shell/comms/src/pane.rs`), gloss is swatch geometry in pictograph,
  Apparatus is a domain crate (`crates/domain/apparatus`) whose rows Cambium's
  detail panel and sectioned list name beside Steward's, and Graphshell names
  Inspector and Roster. The dock contract the panes doc proposed exists twice
  and is shared by neither: `DockState` and `DockSide` in the comms pane, whose
  module doc says they should graduate to a shared dock, and the kernel's
  `PanePresentationMode` (tiled, docked, floating, fullscreen) in
  `crates/graph/graph-kernel/src/pane.rs`. The card family lives in design docs
  and in Graphshell's 354 `card` mentions, none of which carry the family's four
  names; Cambium's "card" is a context for the summary body. A catalog that
  does not say *built, named, or designed* per row will be read as a claim that
  everything in it exists.
- **No catalog says which authority a surface projects.** The panes half of
  the frame-tree row comes closest, grouping panes by primary authority. The others enumerate shapes.
- **The doctrine has no GUI area.** `doctrine.md` has six Stack statements
  and none about surfaces, so there is nothing a GUI question can be filed
  under as *applies* or *refines*.

## 2. The matrix

**Rows: the surfaces.** The nouns TERMINOLOGY already fixes, plus the ones
the catalogs above add. Window; frame; pane; shellbar; tile and the workbench
tile tree; canvas (orrery and volvelle are its form factors, the gnode a
node's body); swatch;
card; overlay and popover; command surface (palette, picker, context menu);
chip; scope trail; settings page; status bar; dialog.

**Columns: the concerns.** None is a GUI thing on its own. History;
persistence; actors and workers; resources; engines; analysis and
generation; networking and transports; sharing and community; identity and
encryption; asynchrony; mesh computation. Plus the ones §3 adds.

**Each cell answers two questions.**

1. *Which authority answers it.* Not "the roster shows history" but "the
   roster shows the surface stratum's trail, owned by the session's journal."
2. *What the typed refusal looks like when it cannot.* A build that links no
   transport still has the pairing surface and the Comms pane; each has to say
   something. The modularity map already has the pattern: a Turnstone without
   `nomadnet` refuses a `destinationhex:/path` page with a typed message, and
   `TURNSTONE_OFFLINE=1` answers fetches with a typed refusal. The Cambium
   catalog already has the widget-level pattern: disabled reasons on commands,
   and empty, error and unavailable states on setting rows.

The second question is the
[stack modularity direction](../technical_architecture/2026-10-25_stack_modularity_map.md)
turned into GUI. Once transports and engines are opt-in, the catalog differs
per build profile, and a cell that does not name its refusal is a cell that
will panic, hide, or lie in the profile that lacks its engine.

## 3. What the list leaves out

Each item below is a column or a row the stack already has a record for and
Mark's list did not name. The citation is where the stack already says it.

- **Windows.** [One state, N windows](2026-07-05_one_state_n_windows_design.md)
  makes a window a projection of one app state, yet the panes doc says panes
  are per window. Window-local state is a column, not a footnote.
- **Swatches and the LOD ladder.** A swatch is a view instance over shared
  truth ([swatch primitive](2026-06-27_swatch_primitive_design.md)). The
  interaction spine's ladder is a glyph by default, a DOM subtree on focus, an
  underlay dot on cull, and "materialization is state", which puts the ladder
  in the persistence column.
- **Scope and the trail.** `ScopeNavigation<P, A>` in
  `crates/cambium/cambium/src/scope_navigation/` landed in Woodshed and Knot on
  2026-10-23 ([contextual navigation](../implementation_strategy/2026-10-23_contextual_navigation_plan.md)).
  Mark's 2026-10-25 description of settings reached as "context cards opened
  from the context menu as a peek-down scope" is the same mechanism.
- **Selection and focus as separate authorities.** The contextual navigation
  findings: Woodshed's presentation selection and playback ownership must not
  share an operation; Knot's restoration must validate the live source basis
  before applying a remembered heading. The design language record §9.2 adds
  foreground pins and background membership that survives deselection. Selection
  is its own column, with at least three owners.
- **Keeping levels and the ambient level.** Attention runs background,
  foreground, focus; keeping runs ambient, short-term, long-term, codicil, and
  every move records who made it ([ambiance](2026-09-23_ambiance_design.md) §9).
  Generated relations enter at a keeping level that "depends on the model and
  the purpose". So analysis and generation has a GUI face only through keeping.
- **Grants, leases, petitions, rings, custody.** The participant gate, watches,
  petitions ([facet signaling](../technical_architecture/2026-08-16_facet_signaling_and_control_loops.md)),
  the vault lock view (`dramatis::view::VaultLockView`, used by the door's
  custody client). Identity is not one column. It decides which cells render.
- **Asks and refusals.** Durable asks checkpoint at every token boundary
  ([resident inference](../implementation_strategy/2026-10-15_resident_inference_and_recall_plan.md)
  R8); disabled reasons; empty, error and unavailable states; typed offline
  refusals. These are one row today only in Cambium's catalog.
- **Accessibility and input.** The web host's a11y mirror
  (`crates/cambium/cambium-genet-web-host/src/a11y.rs`), `SurfaceId` as the
  event source list, AccessKit as the provider-side tree, and inker's
  `SurfaceProducer` / `WebSurface` with `capabilities()` as the input contract.
  Every row has to answer this column; only the Cambium catalog asserts it.
- **Settings as a stack capability.** Mark, 2026-10-25, during the modularity
  audit: settings should be unified as a stack capability with Woodshed's
  model as the baseline, presented as "panes, a full-screen tile, hierarchical
  tabs, or context cards". Woodshed's `AppSettings`
  (`woodshed/crates/woodshed-core/src/settings.rs`) splits portable preferences
  from host-local devices and transient state, with one owner per setting and a
  route per page. Cambium has `setting_row`; Mere has pandect's settings store.
  Nothing in Mere's docs records the ruling yet; this is its first citation.
- **Theme and ambiance.** Tabard owns theme definitions; each application owns
  its `appearance` record (stack lift R6); `tinct` is the colour crate; the
  attention foreground is a presentation role, not a plane.
- **History is at least five things.** Scope history (`ScopeNavigation`),
  edit history (`backstep`, the crate `edit-history` became), fold and
  checkpoint history (`sceno::Fold`, the one-tree plan), the journal with
  fork, branch and pick-off lineage (TERMINOLOGY), and deterministic replay
  ([wholesale snapshots](../../2026-07-31_deterministic_replay_brief.md)). The
  GUI shows all five as the one word *history*.
- **Resources against encounters.** A clip's source, captured version and
  projected appearance are distinct; two tabs showing the same page are two
  accessings of one resource ([family composition brief §8](../../2026-08-12_family_composition_thesis_brief.md#8-primary-data-model-clips-and-live-entities-2026-10-11),
  design language §9.3). The resources column has to show the encounter, not
  only the resource.
- **Embedding and tenants.** Graphshell as a viewer inside merelyllc.com,
  caller-owned wgpu contexts, browser WebGL2 presentation tenants
  ([family composition brief](../../2026-08-12_family_composition_thesis_brief.md)).
  The host is a row too, and it changes what every other row may assume.
- **Control density.** The one-tree plan records "control density and small
  glyphs remain usability work". A catalog without a density column repeats it.
- **Thin rather than absent.** Localization appears in eight active docs and
  i18n in one. Print and export could not be separated from "printing press"
  by search. Clipboard and drag are covered.

## 4. The loops

Each is a place where two catalogs, or two authorities, claim the same cell.
The evidence is cited; what would improve the stack follows it.

1. **Forme, platen, pictograph.** `platen` depends on `forme` and `workbench`
   (`crates/platen/platen/Cargo.toml`); Graphshell's `forme_session` and
   `forme_workspace` use `platen::TileLayout` and `TreeGeometry` and touch
   pictograph, so they cannot move into `forme`, and only the `mere` facade
   sees all four ([stack lift](../implementation_strategy/2026-10-14_graphshell_stack_lift_plan.md)
   §2 and R2). The tile tree's authority, its plan compiler and its canvas form
   a cycle at the center of the workbench row. R2 cuts it by putting workspace
   and session in `platen` behind a feature once the three canvas-touching
   pieces are out. Until then the workbench row has no single home to cite.
2. **Node faces.** One node has a gnode, a roster row, a tile tab, a card
   summoned about it, a swatch instance and, in the June design, a session
   chip. The representation-identity rule ties their colour and selection
   highlight. Nothing says which face owns selection when two are visible, and
   §9.2's foreground pins add a fourth owner. What would improve: one line per
   face in the catalog naming whether it may *initiate* selection or only
   *reflect* it.
3. **Window-local state.** One state dissolves sync, but dock state, focus,
   the rehearsal cursor and the attention aspect ("per view, never synced",
   TERMINOLOGY) are window-local on purpose. The catalog needs the column that
   says which rows are window-local by design, so that "nothing to sync" and
   "per window" stop reading as a contradiction.
4. **Settings about surfaces.** A pane's width is a setting, and settings are
   presented in panes, tiles and cards. Owner-per-setting breaks the loop only
   if the pane shell does not own its own width: the setting's owner is the
   dock, the pane renders it. The same holds for a tile's split ratio (Forme)
   and a window's size (the host).
5. **One async operation, three homes.** A fetch or a model job is a Steward
   row while running, its petition lives wherever asks live, and its result is
   a facet write that shows in the roster and on the canvas. Steward exists
   today as a `SurfaceId` and nothing else. Nothing names the operation across
   the three; the journal attributes the write but not the job. A job id that
   the ask, the Steward row and the facet all carry would close it.
6. **Engine opt-in against tile chrome.** Tile chrome gates on document
   capabilities (`DocumentCapabilities` in inker, read by `document-lanes`);
   capabilities depend on which engine is linked; linking is now a product
   feature. So the catalog differs per build profile. A record of the catalog
   that does not say which profile it describes is wrong in every other one.
7. **Identity without transport.** The modularity map says an offline Knot
   with no resident would have to replace the door with a local key provider.
   The vault lock view exists; what it unlocks when nothing is reachable is
   unanswered. The identity column's refusal cell is empty for the offline
   profile.
8. **Generation to keeping.** Resident inference produces relations; they
   enter at a keeping level; promotion is a recorded move; the surface that
   performs the move is unnamed. The reservoir plan lists promotion among its
   follow-ons. The analysis-and-generation column has a result cell and no
   action cell.

## 5. What ties the loops

Most of the eight close if a cell names an authority rather than a surface.
Five authorities cover every cell read above:

| Authority | Decides | Already recorded in |
|---|---|---|
| the node | what is shown | one node, atomic facets; the summoning model |
| the scope | which part, from where | contextual navigation; scope model reconciliation |
| the ring | who may see or act | participant gate, grants, petitions, watches |
| the keeping level | whether it is in focus, kept, or ambient | ambiance §9; graph semantics ruling 17 |
| the build profile | whether the engine behind it is linked | stack modularity map |

A surface is then a projection of a node set under a scope, a ring, a keeping
level and a profile. The catalog becomes a cross-reference between the six
lists in §1, not an eighth list, and a new surface is catalog-complete when
each of the five authorities has a line for it, including its refusal.

This record proposes nothing beyond that reading. A ruling would decide
whether the matrix is kept as one document, as a column in TERMINOLOGY's
aspect table, or as a doctrine area that the Stack statements lack.
