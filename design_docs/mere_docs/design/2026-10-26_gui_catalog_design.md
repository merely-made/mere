# The GUI catalog: surfaces by concerns, and the loops between them

**Date:** 2026-10-26
**Status:** design record, 2026-10-26. Cues only: what the stack's existing
catalogs cover, what a complete catalog would have to add, and the loops a
catalog has to close. No ruling, no code, no work assigned. Amended the same
day with §6, where the catalogs disagree with each other and with the tree,
§7, Mark's rulings 1 to 5 on them, §8, his refinement of rulings 1 and 3: a
frame hosts one presentation, and what a mod contributes is a catalog axis, and
§9, his rulings 6 to 8 on the cues §8 left: a typed `contributes` list, a misfin
address opens its Comms conversation, and inker defines the embedded-application
declaration that a manifest carries. The §9 code landed 2026-10-27, read at
`10fe98b50`; §9 closes with what it leaves.

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
Read against each other and the tree, the catalogs also disagree (§6).

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
  diagnostic observations come from Registry (`crates/system/registry`)
  through Armillary's `apparatus-core`, and product views own their inspection
  UI. Cambium's detail panel and sectioned list name Apparatus rows beside
  Steward's, and Graphshell names Inspector and Roster. The dock contract the panes doc proposed exists twice
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

**A third axis, from §8: what a mod contributes.** A presentation, a pane
adapter, a pane, or a servitor. Every row above can be supplied by a
contribution of one of those kinds, and the catalog says which.

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

## 6. Where the catalogs disagree

Mark, 2026-10-26: "i bet a bunch of terminology and relevant plans and code
need updating" and "like, do those catalogs agree?" They do not. Read against
the tree at `d969028be`:

| Surface | What the sources say | The disagreement |
|---|---|---|
| panes | the panes doc: Gloss, Roster, Inspector, Index, Steward, Apparatus, Comms. TERMINOLOGY: roster, gloss, inspector, apparatus, comms. `SurfaceId`: those plus TreeSpine, NavigatorHost, WorkbenchPane. Kernel `ToolPaneState` (`pane.rs`, from April): Diagnostics, HistoryManager, AccessibilityInspector, FileTree, Settings. Built: Comms as a view-model crate, gloss as pictograph swatch geometry, Apparatus as a domain crate, Roster and Inspector named in Graphshell, Steward as rows in Cambium's detail panel | five lists; `ToolPaneState`'s five names appear in no catalog |
| Navigator and gloss | TERMINOLOGY ruled the Navigator is gloss expanded (2026-06-07) | `SurfaceId` carries both `GlossPane` and `NavigatorHost`; kernel `pane.rs`, chrome routing, forme's lens, `mere::glossary` and pictograph's gloss docs still say Navigator |
| dock contract | the panes doc proposes one (kind, open, side, width, focus, context binding) | it exists twice and is shared by neither: `DockState` and `DockSide` in `crates/shell/comms/src/pane.rs`, whose module doc says they graduate to a shared dock, and the kernel's `PanePresentationMode` |
| pane surfaces | `SurfaceId`'s doc says each surface is one variant and one emission site at its open and dismiss seams | four variants are emitted at real seams (CommandPalette, NodeFinder, ConfirmDialog, ContextMenu); every pane variant appears only in the descriptor table and ux-events' diagnostics map |
| workbench | TERMINOLOGY's tile entry, the `workbench` crate ("split-and-tab workspace organization") and Mark's list: a tile tree | the [composition spine](../technical_architecture/2026-05-21_mere_composition_spine.md), canonical for the arrangement ontology, makes the tree one of six projections and says the tree was masquerading as the ontology |
| cards | the summoning doc's family: preview, unvisited, connections swatch, object card | none of the four names is in code; Graphshell's card identifiers are `focus_card`, `PortableCard`, `receipt_card`, `card_offer`, `card_response`; Cambium's nearest is the detail popover |
| orrery | TERMINOLOGY: a form factor. `cartography::FormFactor`: Canvas, Orrery, Volvelle, Astroid, Minimap | code and TERMINOLOGY agree; TERMINOLOGY's survivors row still waited on a `Scope::Orrery` that no longer exists (corrected 2026-10-26) |
| swatch | TERMINOLOGY: a scope (node, subgraph, nested graph, mere) and a mode. `scenograph::Scope`: Node, Subgraph, Link, NestedGraph, Mere, Selection | agree; the entry omits Link and Selection |

The retired words, by where they survive (active docs exclude `archive_docs`):

| Word | Active docs | Code files | In TERMINOLOGY's survivors table before today |
|---|---:|---:|---|
| meerkat | 85 | 32 | no |
| orrery | 77 | 29 | yes, stale row |
| engram | 46 | 4 | no |
| tessera | 41 | 0 | yes |
| verso | 35 | 16 | no |
| graphlet | 34 | 0 | the banner rule |
| gyre | 32 | 1 | no |
| Navigator | 31 | 7 | no |
| cabal | 18 | 9 | no, and `CabalId`, `CabalSink`, `MurmCabal` are live types |

The doc column is mostly age, and the policy already handles it: newer wins,
banners and historical markers mark the rest, and no sweep is wanted. The code
column is the cue. TERMINOLOGY's survivors table exists so nobody re-audits
those names, migrating them when a file is touched and never as a churn pass,
and most of these names were not in it, so each pass re-found them. The table
is refreshed today with every row above.

What would improve the stack, as cues:

- One pane list. `SurfaceId`, `ToolPaneState` and TERMINOLOGY's pane entry
  reconciled, with `ToolPaneState`'s five names either given catalog names or
  retired; Index and TreeSpine named as one thing or two.
- One dock contract, graduated from the comms pane to the shared place its
  module doc already names, with the kernel's presentation mode as its
  mobility half.
- Pane surfaces that emit. A pane variant in `SurfaceId` either has an open and
  dismiss seam or is not there until the pane is built.
- The workbench row saying what the spine says: the tile tree is the default
  desktop projection of a forme, not the workbench. Or the spine's claim
  narrowed, if the tree is the ontology after all.
- The card family with names in code, or the code's card vocabulary (focus,
  portable, receipt, offer, response) catalogued in the summoning doc beside
  the four it names.
- The cabal types renamed when the comms crate is next touched; the entry's
  2026-09-05 retirement renamed the term, not the types.

## 7. Rulings

Mark ruled 1 to 5 on 2026-10-26. Each was a choice among offered options; the
options and the evidence put to him are kept under each ruling, as the
[stack lift plan](../implementation_strategy/2026-10-14_graphshell_stack_lift_plan.md)
§5 does. Three of §6's rows needed no ruling: orrery (already reconciled),
swatch (TERMINOLOGY's entry now lists link and selection scopes), and the
cabal types (the survivor rule).

1. **The pane list. Ruled: TERMINOLOGY's list.** Roster, gloss, inspector,
   apparatus, comms, plus Steward once built. The kernel's `ToolPaneState`
   names fold: Diagnostics and AccessibilityInspector into apparatus, whose
   `SurfaceId` doc already lists diagnostics, probes and a11y status; Settings
   into the settings capability (Mark, 2026-10-25); HistoryManager into
   history; FileTree retired. It had three uses, all in chrome routing. Index
   and TreeSpine are in no canonical list, and TreeSpine leaves `SurfaceId`
   under ruling 2. Options set aside: the kernel lists as canonical, with
   TERMINOLOGY rewritten to them; product panes and tool surfaces kept apart
   as two kinds.
2. **What a `SurfaceId` variant means. Ruled: an emitting surface.** A variant
   needs an open and dismiss seam. Unbuilt pane variants and their
   accessibility descriptors leave until the pane is built; this record is the
   intended set. Nothing stored references them. Option set aside: keep every
   intended surface declared, and mark declared-not-emitting in the descriptor
   table.
3. **The workbench. Ruled: the surface that presents a forme.** The spine
   stands: a tile is a slot of node content in a projection, and the tile tree
   (splits and tabs) is the default desktop projection, not the workbench.
   TERMINOLOGY's tile entry is rewritten and a workbench entry added. Options
   set aside: the workbench as the tile tree, with the spine narrowed; deferring
   until a second projection is built.
4. **Cards. Ruled: kinds in the slot.** The family (preview, unvisited,
   connections swatch, object) names kinds of content the focus-card slot
   summons. `focus_card` is that slot, `PortableCard` is chirograph's small
   semantic card for sharing, `receipt_card` renders receipts. The summoning
   doc gains a line per code word; nothing renames. Options set aside: the code
   vocabulary as the live family; the four kinds becoming identifiers under the
   survivor rule.
5. **The dock contract. Ruled: Cambium's shell.** `crates/cambium/cambium/src/shell`
   owns dock state (open, side, width, focus, context binding) as the pane
   shell's contract, beside the overlay's edge placement. `DockState` and
   `DockSide` move out of the comms pane; the kernel keeps
   `PanePresentationMode` as the intent-level mobility word. Options set aside:
   the kernel's `pane.rs`; the chrome crate beside `ToolbarState`; leaving both
   until a second docked pane.

What the rulings leave in code, as cues: `SurfaceId` carries unbuilt pane
variants and `ToolPaneState` carries five names that now fold (kernel
`pane.rs`, `accessibility.rs`, ux-events' diagnostics map, chrome routing);
the dock contract still lives in the comms pane rather than Cambium's shell.
Both are mechanical under rulings 1, 2 and 5, and neither touches a stored
format.

## 8. Refinement of rulings 1 and 3: a frame hosts one presentation

Mark, 2026-10-26, after the rulings: "have we goofed on the panes? Missing
any... overlap... or the concept? It's kinda like tile with no tab, but also a
good embedding sidecar right? Does it make sense to have like redshank panes,
or knot, signalman...? And integrate content into the graph while using the
embedded app as a viewer? And make them available as selectable mods?" And,
on the first telling of this section: "it seems like enrichment, but also
there are a lot of legacy concepts in this telling that need consolidation to
avoid conflicting."

**What the tree says.** The kernel's `crates/graph/graph-kernel/src/pane.rs`
(April, pre-flip) defines a pane as any frame leaf hosting a "viewer", with
`TileRenderMode` naming Servo, Wry and an egui host as the ways pixels are
made. That is the "tile with no tab, embedding sidecar", in the words of its
day. The live words are TERMINOLOGY's: a **reader** (`inker::Engine`) parses
acquired content into the document model and document-canvas presents it; a
**surface producer** is a long-lived presentation driven by an inker host,
input in, frames out, which is what an embedded application is; the
interaction spine's three lanes classify how either reaches the screen. Knot
documents are already presented by nematic's Knot reader (`ENGINE_KNOT`,
`ENGINE_KNOT_DJOT`), and edits cross back through the shared source-edit
proposals. The Comms pane is an adapter model: a `ProtocolAdapter`
(`crates/shell/comms/src/adapter.rs`) presents one backend as conversations
and accepts a draft. Misfin is both a reader (`ENGINE_MISFIN`) and a comms
adapter, and nothing says which a node that references a mailbox opens into.
The mod manifest (`crates/system/registry/src/mod_loader/loader/types.rs`)
has `provides` as an untyped list of strings, `ModType` as Native or Wasm, and
`ModCapability` as Network, Filesystem, Identity, Clipboard, Exec: trust and
loading, with no word for what a mod adds to the GUI. Pelt already carries a
per-tile reader override (`route_overrides` in `ports/pelt/viewer.rs`).

**Ruled, refining 1 and 3.** A frame hosts one presentation: a reader's
document, or a surface producer. A tile is a frame presenting a node's
resource; a pane is a frame presenting an answer to one question about the
graph or the system. They differ by what is presented, not by whether an
application is embedded: a surface producer can be what either presents.
Ruling 1's pane list and ruling 3's workbench stand; this names the axis they
sit on.

**Contributions, a catalog axis.** What a pack or mod adds to the GUI is one
of four kinds:

| Kind | What it is | Existing seam | Example |
|---|---|---|---|
| presentation | a reader or a surface producer registered with inker, which a node's resource opens into (the design language record's "registered presentation") | inker's registry and routing; the engine flip; pelt's per-tile route override | Knot documents (a reader); the Knot editor or a playback application (a surface producer) |
| pane adapter | a feed into an existing pane | comms `ProtocolAdapter` | Signalman's LXMF path into Comms |
| pane | a new question about the graph or system | ruling 1's list; `SurfaceId` under ruling 2 | Steward, when built |
| servitor | a participant under a grant | the participant gate; a swatch over its nested graph is its inspection UI | a resident helper |

Knot, Signalman and Redshank therefore want a presentation, a pane adapter
and a presentation, not three panes. Redshank is read here only from its docs
(a playback piece with Web Annotation tests); if its annotations are kept,
they are link-model assertions. Selection falls to the settings capability for
per-mere defaults and to a per-node presentation override on the object card,
which pelt's per-tile override already shapes.

**Content in the graph, the application as what presents it.** This is the
[family composition brief §8](../../2026-08-12_family_composition_thesis_brief.md#8-primary-data-model-clips-and-live-entities-2026-10-11)
direction, and three of its four seams exist: inker routes content type to
reader, resources are identified by canonical IRI, and source-edit proposals
carry edits back. The fourth, what an embedded application declares, is split
across inker's registry and producer capabilities, the mod manifest's trust
fields, and the resource-leaf draft, with no single contract.

**Examined critically, the same day.** The first telling of this section
leaned on four words that TERMINOLOGY has retired or never had, and they are
consolidated here rather than added: *viewer* (the kernel's April word and the
registry's unextracted "viewer registry") is *presentation*, a reader or a
surface producer; *engine* as a contribution kind is retired for readers
(survivors table) and lives only for inference and script engines, so the kind
is *presentation* and the example is a *reader*; `TileRenderMode`'s Servo and
Wry modes are the pre-flip classification, and the interaction spine's lanes
are the live one; the May gate name `engine.route_override` is not in the
tree, and pelt's `route_overrides` is. `PaneId`, `TileRenderMode` and
`PanePresentationMode` join the survivors table as the kernel's pane-as-frame
sense.

**What this leaves, as cues.** The mod manifest's `provides` has no vocabulary
for the four kinds. Misfin's double registration has no rule for which opens.
An embedded application's declaration (what it presents, what it exposes as
resource leaves and clips, what it accepts back, how it reaches the screen)
lives in three places. Ruled the same day in §9.

## 9. Rulings 6 to 8: the cues §8 left

Mark ruled 6 to 8 on 2026-10-26, each a choice among offered options, with the
evidence read from the tree at `8d69c9144` the same day and kept under each
ruling as §7 does.

6. **A mod's contributions. Ruled: a typed `contributes` list.** Evidence:
   `ModManifest::provides` (`crates/system/registry/src/mod_loader/loader/types.rs`)
   is a list of strings read for dependency order (`resolve_mod_load_order`),
   `is_capability_available` and `active_capability_ids`, and nothing else.
   The seven built-in manifests in `registry_default.rs` put two vocabularies
   in it: registry names that other mods require (`ProtocolRegistry`,
   `ViewerRegistry`, `IdentityRegistry`, `ActionRegistry`, `ControlPanel`,
   `DiagnosticsRegistry`) and contributed ids (`protocol:http`,
   `viewer:webview`). A typed vocabulary already exists one step later:
   `ModExtensionRecord` (ProtocolScheme, ViewerMime, ViewerExtension,
   ViewerCapabilities, Action, IndexProvider, Lens, Theme, WasmRuntime) is
   what activation returns, and only WasmRuntime is produced outside tests.
   The pack manifest (eidetic `PackManifest`) carries the trust ask as
   `requested_scopes` and its parts by role. Ruled: `contributes` is a typed
   field beside `provides`, naming each contribution by kind (presentation,
   pane adapter, pane, servitor) and id, in the TOML manifest
   (`DiskModManifest`) and the native registrations alike. `provides` and
   `requires` keep only dependency names; the `protocol:` and `viewer:` ids
   move into `contributes`. The records a mod returns at activation verify
   the declaration: a mod that registers what it did not declare is refused,
   a typed refusal in the catalog's sense. `requested_scopes` stays the trust
   ask. Options set aside: four reserved prefixes on the `provides` strings;
   deriving the kind from the activation records alone, which would leave
   install review and the settings capability's per-mere selection unable to
   show a contribution before the mod runs.
7. **A misfin address. Ruled: it opens the Comms conversation.** Evidence:
   Misfin is a send protocol, the request carries the message and the reply
   is a status, and nothing in mere's tree names the scheme on a fetch path;
   `document-lanes`' smolweb lane only renders a body it is handed, by scheme
   or by `message/x-misfin`. So inker's default `misfin` scheme rule
   (`crates/inker/inker/src/routing.rs`) and the registry's middlenet scheme
   list (`viewer.rs`) reach a reader with nothing to read. The comms adapter
   (`crates/shell/comms/src/misfin_adapter.rs`) presents one mailbox from the
   receive server's `MailboxStore`, one conversation per correspondent, and
   its bodies are gemtext. The kernel's `AddressKind` has no misfin variant.
   Ruled: a misfin address names a correspondent, not a document. Opening a
   node that carries one selects that conversation in Comms
   (`CommsPane::select`, opening the pane if it is closed). The reader keeps
   its `message/x-misfin` content-type route for a body a host already holds,
   as the adapter's messages are; the `misfin` scheme rule leaves inker's
   default policy and the registry's middlenet list. Options set aside: a
   reader tile with Comms reached only by its command; received messages as
   resource nodes with minted IRIs, a graph-model change the opening rule
   does not need.
8. **An embedded application's declaration. Ruled: inker defines it, the
   manifest carries it.** Evidence: the declaration's four parts live in
   three places. Inker holds how it reaches the screen and what it accepts:
   `SurfaceEngine` (id, spawn, `a11y_capability`), `WebSurface::capabilities`
   (cookie, script, pointer, document, drag-drop), and `EngineRoutePolicy`
   rules that map a scheme or content type to an engine id and a
   `SurfaceContractMode` (CompositedTexture, NativeOverlay, EmbeddedHost,
   Headless); three engine crates (graft, weld, scrying) implement
   `SurfaceEngine`, and hosts collect them in `SurfaceEngineRegistry`. The
   registry's viewer registry (`crates/system/registry/src/viewer.rs`:
   `ViewerCapability`, `ViewerRenderMode`, `ViewerRegistry`) declares the
   same things in the April words and has no caller outside its crate. Trust
   lives in the mod manifest's `ModCapability`, which document-host checks at
   activation, and in the pack's `requested_scopes`. What an application
   exposes is the composition brief's clip ruling 4 (inspect, annotate,
   reclip, open, play, edit and apply as separate operations) and the
   resource-leaf draft; what it accepts back is the shared source-edit crate
   (`crates/system/source-edit`, the 2026-10-18 plan). Neither registry nor
   inker depends on the other. Ruled: one declaration type in inker: what it
   presents (schemes, content types, extensions), how it reaches the screen
   (`SurfaceContractMode`, `A11yCapability`, producer capabilities), what it
   accepts back (source-edit proposals, commands), and which of the brief's
   operations it exposes. A manifest's presentation entry under `contributes`
   carries that type; built-ins construct it in code, as
   `EngineRoutePolicy::default` and the engine crates do now. The registry's
   viewer registry and the `ModExtensionRecord` viewer variants retire into
   it (survivors table). Trust stays with `ModCapability` and
   `requested_scopes`. Leaves and clips stay graph facts under the link model
   and the resource-leaf draft, which a presentation's operations act on;
   they are not fields of the declaration. Options set aside: the manifest as
   owner, which would have the registry crate define a schema inker cannot
   import; three owners named in TERMINOLOGY with nothing unified.

Landed 2026-10-27, read at `10fe98b50`. Rulings 6 and 8 (`cebb68fa4`, merged
as `306fa68c7`): `inker::PresentationDeclaration`
(`crates/inker/inker/src/presentation.rs`) carries the four parts: `presents`
(schemes, content types, extensions), `reaches` (`SurfaceContractMode`,
accessibility, producer capabilities), `accepts` (source edits, commands) and
`operations`. `EngineRoutePolicy::default` is built from the declared
built-in readers, routing unchanged, and graft, weld and scrying carry their
id and accessibility in their declaration. `ModManifest` and
`DiskModManifest` carry a typed `contributes` (`loader/contributes.rs`:
presentation, pane adapter, pane, servitor); Verso's four ids left `provides`
and are one presentation; activation refuses a protocol, content-type,
extension or reach record the manifest did not declare
(`ModActivationError::undeclared`) and rolls back what the mod applied. The
registry's viewer registry is deleted with its `viewer`, `audio` and `pdf`
features, `ModExtensionRecord`'s viewer variants are
`PresentationContentType`, `PresentationExtension` and
`PresentationReaches`, and the registry takes `inker` under `mod-loader`.
Ruling 7 (`c499b708f`, `10fe98b50`): the `misfin` scheme rule is out of
inker's default policy, and the middlenet list went with the viewer registry;
`comms::misfin_correspondent` reads the correspondent from a misfin address
and `CommsPane::misfin_conversation` finds its loaded conversation;
graphshell's tree page owns a `CommsPane` (`web_tree/comms_pane.rs`), and its
three Forme open paths (`open_selected`, `open_retained`, `read_related`)
send a correspondent node to Comms through `graphshell::comms_open::open_node`
instead of a tile, with `mere-comms` an optional dependency under
`workspace-editor`.

What that leaves, as cues: verification runs one way, since native activation
returns no records, and Action, IndexProvider, Lens, Theme and WasmRuntime
records are not declarable; the built-in declarations list no operations and
accept nothing, because nothing in code exposes either, so the brief's
operations have a field and no type behind it; `ViewerRegistry` and
`mod:core-viewer` remain as dependency names so load order holds; the tree
page loads no inbox, since the misfin adapter reads a native mailbox store the
browser lacks, so every misfin open lands on the no-conversation notice until
a backend fills it; what opening does when no conversation is loaded (the pane
opens and keeps its selection) is unruled; the pelt workspace has no Comms
path; and whether a murm author key or an LXMF destination opens the way a
misfin address does is unruled.
