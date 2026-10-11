# Projection Scenes and the Graph-Native Application Platform

*Written before the 2026-09-05 retirement of graphlet (TERMINOLOGY.md): read graphlet as subgraph. Identifiers such as GraphletId, GraphletRef, and SessionGraphlets are now SubgraphId, SubgraphRef, and SessionSubgraphs, and the graphlets crate is crates/graph/subgraph (code renamed 2026-09-12).*

**Date:** 2026-08-23  
**Status:** direction, discussed with Mark 2026-08-23; no code task opened here  
**Scope:** sharpen the projection-scene test, record the scene-catalog cuts, and
derive the capabilities Mere, Scenograph, Cambium, and Genet need to extend the
web platform with graph-native application behavior.

**2026-10-09 continuation:** [§9](#9-configurable-visual-and-interaction-language-2026-10-09)
records Mark's cross-app visual and interaction direction: selection and
ambient context, the forme as a field, predictable dynamics, configurable
themes and fonts, and Emblem/Pictograph iconography. It is a design record;
implementation and consumer adoption remain with the owning plans.

**Related:**

- [projection grammar catalog](mere_docs/research/2026-08-15_projection_grammar_catalog.md)
- [Scenograph content catalog](mere_docs/research/2026-08-18_scenograph_content_catalog.md)
- [shelfmark format note](mere_docs/technical_architecture/2026-08-16_shelfmark_format_note.md)
- [family composition thesis](2026-08-12_family_composition_thesis_brief.md)
- [Turnstone suite composition and capability census](2026-08-22_turnstone_suite_composition_and_capability_census.md)
- Genet `components/cambium/ARCHITECTURE.md` *(historical citation)* <!-- doc-audit: historical-path -->
- Genet `docs/2026-08-12_meristem_scope_cut_and_component_contract_brief.md`
- Genet `docs/2026-08-14_web_platform_host_contract_plan.md`

**2026-08-23 follow-on:** the §8 receipts are scheduled in the
[projection receipts plan](archive_docs/2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md):
wave 1 is Matrix and coordination with mer3ly as first consumer and the
gazette Ledger as heterogeneous second; the field receipts stay gated there.

## 1. Ruling

A scene is a reusable projection regime, not a renamed dataset, application
surface, arrangement, query, or item representation. It composes a reading,
encodings, an arrangement, relation forms, guides or backdrops, and interaction
policy toward one legible purpose. It must still read true against a second,
heterogeneous dataset.

The scene inventory is therefore expected to be small. Fifty names are easy to
produce by changing product nouns or one projection lever. That does not yield
fifty scenes. The useful work is to find the smaller set of representations in
which source entities and relations become categorically different projected
objects.

Before applying the scene test, identify which projection layer owns the
candidate's novelty: authority, reading, encoding, arrangement, relation form,
guide or backdrop, composition, interaction, realization, or the total scene
recipe. If the novelty is exhausted by one lower layer, the candidate belongs
there even when the resulting application surface is distinctive. This
pre-filter rejects readings and composition capabilities wearing scene names.

For every candidate that survives the layer pre-filter, answer:

1. Which authority elements and values does it read?
2. Which selection, grouping, aggregation, traversal, temporal, spatial, or
   statistical derivation does it perform?
3. What does an entity become in this scene?
4. What does a relation become?
5. What geometric law determines position, size, path, containment, or region?
6. Which guides, scales, fields, spaces, or backdrops are required?
7. How does every projected or derived object map back to source authority?
8. Which intent changes source truth, reading parameters, projection state, or
   view state?
9. What second dataset proves that the recipe is not an ontology skin?

Changing graphlet depth, product vocabulary, styling, backdrop, or one
arrangement lever produces a configuration or variant unless the change also
alters the representation's governing purpose and source-to-scene mapping.

## 2. Current scene cut

This is a catalog judgment, not a portable-contract commitment. Two statuses
must remain independent:

1. **Categorical status:** whether a recipe is a distinct scene family.
2. **Contract evidence:** whether a consumer has forced its portable meaning
   and the required promotion receipts have landed.

This direction note may change categorical status without advancing a contract
gate. In the table, **no promotion asserted** means exactly that: the row makes
no new claim about implementation, consumer proof, or portability.

### Categorically distinct scene families

| Scene | Governing representation | Contract evidence named here |
| --- | --- | --- |
| **Orrery** | Entities become spatial bodies; relations become selectable routes; topology governs the view. | No promotion asserted. |
| **Mosaic** | Entities become adjacent media tiles; adjacency carries kinship and the collection becomes the ground. | No promotion asserted. |
| **Atlas** | Entities become markers in a referenced coordinate system; routes, ranges, regions, and geographic context retain real-world meaning. | No promotion asserted. |
| **Tabletop** | Entities become tangible pieces on an authored, collidable ground; placement and zones carry the composition. | No promotion asserted. |
| **Timeline** | Entities become events, spans, samples, or trails on a temporal axis; order, interval, concurrency, and change govern the reading. | No promotion asserted. |
| **Matrix** | Two readings become axes; relations, values, similarities, or deltas become addressable cells. | Two-reading receipts are proposed below. |
| **Partition** | A hierarchy becomes nested, value-bearing area; an entity is an enclosure or share of a whole. | No promotion asserted. |
| **Delta** | Descent topology governs the scene: stable identities become version, branch, membership, and reconciliation structures across epochs. If a retained base scene merely acquires change marks, that is the Diff operator below. | No promotion asserted. |
| **Calendar** | Temporal facts are grouped by recurring units on a semantic temporal backdrop; entries or aggregates inhabit the cells. | No promotion asserted. |
| **Scatter** | One source entity may produce several marks; quantitative values determine position while other facets determine size, color, form, labels, or uncertainty. | No promotion asserted. |
| **Profile** | Selected facets become comparable axes or panels; an entity becomes a multi-value profile. | Gazette is a candidate consumer; no promotion asserted. |
| **Setscape** | Membership becomes containment and overlap; sets become regions and entities inhabit intersections. | No promotion asserted. |
| **Deck** | One graphlet is repeated across configurable facet or metadata panels, with linked identity and declared shared or independent scales. | No promotion asserted. |
| **Territory** | Positioned sites and a winner rule partition a space; an entity becomes a site with a categorical region. | No promotion asserted. |
| **Contour** | A scalar field becomes bands, isolines, extrema, or raster; entities act as samples, anchors, or sources. | No promotion asserted. |
| **Current** | A vector field becomes arrows, streamlines, sources, sinks, and vortices. | A simulator, radio model, or system emulator is only a candidate consumer. |
| **Cartogram** | Named regions deform by quantitative values while preserving selected adjacency or recognizability constraints. | No forcing consumer is cited here. |
| **Volume** | Values and entities inhabit a three-dimensional space through slices, voxels, isosurfaces, landmarks, or nested volumes. | Portable depth semantics remain unproven here. |
| **Distribution** | Samples become contributor-aware bins, densities, quantiles, outliers, or intervals governed by a statistical distribution. | No named consumer is cited here. |
| **Simplex** | Values constrained to one total place an entity in a simplex; barycentric position carries the composition. | No forcing dataset is cited here. |
| **Provenance** | Sources, transformations, confidence, authorship, and freshness become an inspectable derivation structure. | No promotion asserted. |
| **Document** | Entities become passages, figures, annotations, or embedded surfaces in narrative order; relations become references, quotations, and transclusions. | No promotion asserted. |

### Candidates whose categorical status remains unresolved

| Candidate | Current judgment |
| --- | --- |
| **Topological network** | It may be Orrery over typed network facts rather than a separate total regime. Bearer, capacity, and route constraints must change more than the dataset vocabulary to separate it. |
| **Streams** | Earns separation from Timeline only if its shared axis may be ordinal, quantitative, or procedural; stream membership sets one axis and cross-stream relations remain explicit. |
| **Rosette** | Knot has landed poem and lyric proofs, but the 2026-08-23 review reopens whether the recipe is categorically more than polar placement plus chord encoding. A non-prosodic transfer would sharpen that judgment. |
| **Phase** | It may be Scatter plus a temporal trajectory. It earns scene status only if state-space navigation and dynamical structure govern the entire representation. |
| **Incidence** | An n-ary relation becoming a hub, region, or connector may be a relation form rather than a scene. A complete regime must be shown. |
| **Alignment** | Ordered sequences become parallel rows with correspondence columns, gaps, substitutions, and repeats. It remains unclear whether this is a total scene or a Matrix or Streams configuration. |

### Collapsed or reclassified proposals

- **Chronicle** is Timeline with era bands, causal arcs, and replay controls.
- **Circuit** is Topological Network unless typed ports and orthogonal routing
  are source-significant constraints. Without those facts it is an abstract
  network skin.
- **Loom** becomes Streams only after the non-temporal shared-axis rule is
  demonstrated.
- **Spotlight** is a graphlet-producing N-order search combined with radial or
  focus-and-context arrangement.
- **Fog** and **Grove** are last-visited, unvisited, freshness, and tending
  readings or metadata channels. They do not establish scene identity.
- **Ledger** is Matrix with entities on one axis and selected facets on the
  other.
- **Canopy** merely renames a hierarchy reading plus tree arrangement.
- **Itinerary**, ordered selection, path selection, rank, and lasso are graph
  utilities that produce or order a scope.
- **Sequence** and **Statechart** remain ordinary readings of appropriately
  typed data until a product forces a more general projection law.
- **Pulse** and **Feed** are promising entity representations. They enrich a
  node with recent or live data rather than defining a scene.
- **Comparison** is a Matrix flavor at another scope.
- **Diff** is a comparison operator over a retained base scene. It aligns stable
  identities across two epochs and produces addition, removal, move, and value
  marks without replacing the base scene's governing representation.
- **Tag lattice** is a maximal-shared-tag derivation followed by a layered-DAG
  arrangement. Its derived groups may be projected through Setscape,
  Partition, Matrix, or another scene without becoming authoritative nodes.
- **Matryoshka** names nested scene composition: portals, coordinate spaces,
  and authority scopes. A product may use the name for a complete recipe, but
  the reusable novelty belongs to platform composition until such a recipe
  survives the scene test.
- **Neighborhood** is a semantically inferred graphlet. The embedding that
  produces it is a reading or arrangement, not scene identity.
- Node-level, graphlet-level, mere-level, and moot-level lineage are Delta at
  different scopes.

## 3. Matrix is a family over two readings

Matrix is categorically important because neither axis must be the entire
graph. Each axis is an independently produced reading or graphlet.

Initial flavors:

| Axes | Cell meaning |
| --- | --- |
| graphlet A x graphlet A | relation identity, multiplicity, family, direction, strength, or absence |
| graphlet A x graphlet B | cross-scope relations, matching, coverage, or transfer |
| entities x selected facets | exact values, missingness, validation, or difference; the Ledger form |
| entities x scenes/readings | the same source identities compared across projections |
| entities x epochs | presence, value, relation, or representation delta |
| selected entities x selected entities | an ad hoc comparison produced by selection tools |

A cell is a projected object with its own instance identity. It must cite the
source relation, value, contributor set, or derivation that produced it. Picking
a cell may inspect the projection result or route an authorized intent to the
underlying sources; those targets are distinct.

## 4. One source may have several projected instances

The projection grammar already permits one source object to have multiple
projected instances. This becomes a platform-level rule rather than an edge
case.

One entity may appear simultaneously as a Scatter point, Matrix heading, Deck
card, uncertainty mark, legend specimen, Document fragment, or selected-detail
instance. The instances share source identity while retaining separate scene,
space, geometry, representation, and hit identity.

Required behavior:

- source selection can emphasize every visible instance;
- hover and keyboard focus may remain instance-local;
- an instance identifies the facet, relation, or derivation it represents;
- source actions and projection actions remain distinguishable;
- accessibility groups or cross-references repeated appearances rather than
  presenting unrelated duplicates;
- remote and frozen realizations preserve the same source mapping;
- disappearance of one instance does not imply removal of source authority.

This rule is load-bearing for Matrix, Scatter, Deck, comparison and Diff marks,
and Document scenes.

## 5. Graph utilities and enriched graph elements

Several rejected scenes are useful capabilities at a different layer.

### Scope-producing utilities

Search, N-order traversal, lasso, box selection, ordered selection, path
selection, brushing, tag intersection, maximal-shared-tag derivation, and set
operations should produce a stable graphlet or equivalent derived scope with
provenance. A scene consumes that scope without caring how it was produced.

The useful split is:

- Mere owns membership, ordering, derivation, reconciliation, and source
  identity;
- Cambium owns reusable pointer, keyboard, focus, cancellation, and
  announcement behavior;
- the application owns the command that creates, saves, shares, or mutates a
  graphlet;
- a scene owns only how the scope is projected.

### Entity representations

Feed, pulse, meter, badge, portrait, snapshot, and live surface are entity
representations. A scene may request them according to data and level of
detail. They should not require separate scene classes.

An enriched representation may cite several facets and expose several derived
marks while retaining the entity's source identity. High-frequency rendering
may use a Sprigging leaf; semantic structure, values, and available actions
must remain available to DOM, accessibility, and automation paths.

### Nested scene composition

A scene may contain, link to, or open another scene without nested composition
becoming the outer scene's identity. The reusable capability must carry:

- portal and nested-space identity;
- the inner scene's coordinate, focus, and navigation boundary;
- explicit authority scope and authorization at the crossing;
- source-to-instance mapping across the boundary;
- accessible entry, exit, naming, and alternate realization;
- deterministic remote and frozen behavior.

This capability supports Deck, Document, Matrix drill-through, and a possible
Matryoshka product recipe. It belongs to the platform rather than to one named
scene.

### 2026-09-05 review: working surfaces and leaf precedents

Mark proposes leaves as places where graph exploration becomes a working
surface: a snapshot/context card, document in a workbench, or nested scene.
This is a presentation interpretation under review, not a new terminal graph
entity or an implemented universal host contract. Projection eligibility follows
the [agreed requirements/facets basis](mere_docs/research/2026-08-15_projection_grammar_catalog.md#projection-requirements-and-disclosed-facets-2026-09-05).

There are three relevant precedents:

- The [June 2 integration model](archive_docs/2026-10-06_superseded_plans/2026-06-02_modular_integration_plan.md#1-the-architecture-a-graph-rooted-projection-model)
  explicitly calls a detached tile a leaf retaining its graph binding. Preserve
  that source custody without requiring every application to have a spatial
  graph as its privileged visible root.
- The [July 1 summoning model](mere_docs/design/2026-07-01_node_card_summoning_design.md)
  distinguishes a visible node instance from cards about its source. Its
  snapshot audit corrects the earlier assumption that re-rendering cached
  content reproduces a captured live viewport. Capture identity, source
  revision, viewport, freshness, and retention remain separate concerns.
- The [Chisel design](cambium_docs/implementation_strategy/2026-07-07_chisel_widget_leaf_design.md)
  uses leaf to mean a custom-paint element inside a host's layout/input/paint
  machinery. That is a realization mechanism. A working surface may contain
  such leaves, ordinary document content, or another scene.

The [June 7 card attempt](archive_docs/2026-06-09_completed_plans/2026-06-07_card_system_and_staging_plan.md)
also names a concrete failure: focus activated the node, so returning to the
graph reactivated the previous tile. Its two-stage card separated preview
from activation. Preserve that distinction when eligibility or increased
detail offers a live surface; focus, zoom, or availability alone should not
silently start an application session.

The June 22 research's rule that a source keeps one chosen form across all
projections is superseded for this purpose by section 4 above: separate
instances share source identity while retaining their own representation and
focus. Current Sceno expresses source/instance identity, representation slots,
and nested coordinate spaces. Those types support the direction; they do not
alone prove nested application focus, navigation, execution authority, or
resource lifecycle.

Current Forme also separates a stored, graph-bound `FormeDocument` from an
implicit identity view over graph membership. Cambium Workbench's `Tile` is a
handle onto host-resolved `ContentSource`, including an open domain lane.
These existing distinctions are preferable starting points to another durable
"leaf node" wrapper. A custom surface need not pretend to be a web document.

A useful next consumer check is one subject shown as a context card, snapshot,
and opened document/scene. Selection should coordinate; focus should enter and
leave the surface predictably; dismissing or detaching a surface must preserve
source authority; stale/unsupported representations must explain their state.
Moving the outer surface should reuse its content, and inactive live content
needs an explicit resource budget. This defines a possible proof, not a new
implementation commitment. The existing practice-workspace receipt proves
retained moving faces, not this entire lifecycle.

## 6. Field scenes remain distinct

Territory and Contour are related but not interchangeable.

- **Territory** evaluates a categorical winner, such as nearest, strongest, or
  highest-priority site. Every point belongs to one result region; boundaries
  arise where candidates tie.
- **Contour** evaluates a scalar value. Thresholds produce bands or isolines;
  points may be high, low, missing, or outside the supported extent without
  belonging to a winning entity.
- **Current** evaluates a vector. Direction and magnitude produce arrows,
  streamlines, sources, sinks, and vortices.

The three may share field evaluation, rasters, legends, region geometry, and
GPU resources. Their projected objects and questions differ.

## 7. Platform consequences

The web platform remains one realization system. The Merely stack adds durable
graph identity, projection, coordinated views, fields, local-first authority,
and remote composition beside it. Those capabilities should enter at their
own layers rather than being disguised as DOM concepts.

### Mere

Mere should own graph-browser-specific orchestration:

- graphlet and reading algebra for search, traversal, selection, brushing,
  grouping, tags, paths, and epoch scopes;
- source-to-instance and source-to-derived-object bindings;
- contributor and derivation provenance;
- linked selection facts and comparison state across scenes, while preserving
  the producing view for every selection;
- temporal metadata, epoch queries, lineage, and diff scopes;
- authority-scope transitions for nested scenes and portals;
- field providers and mappings from graph facts into field samples;
- saved scene recipes and active `ViewIntent`, separate from graph truth;
- intent routing that distinguishes source, reading, projection, and view-state
  targets.

Mere does not need one universal scene implementation. It needs the graph-aware
bindings and coordination that let several realizations remain views of the
same authority.

The current portable selection noun is `chirograph::Selection`, which carries
only `source` and `targets`. The coordinated-view direction here identifies the
forcing condition for a future resolution strategy, but does not advance A2's
gate by itself. Union, intersection, and crossfilter remain unadopted until two
coordinated views actually require and prove them.

### Scenograph

Scenograph remains product-free. The scene exercise identifies contract
pressure rather than an implementation queue:

- several instances may cite one source;
- derived marks need identity, values, contributor provenance, and semantic
  descriptions;
- scales, axes, legends, annotations, units, thresholds, and missing-value
  behavior need portable meaning when a proof forces them;
- Matrix needs two independent readings and addressable cells;
- Deck needs nested spaces with declared shared or independent scales;
- nested scenes need portable portal, space, source-mapping, and authority-scope
  semantics;
- relation identity must survive routes, cells, adjacency, containment,
  ribbons, order, and alignment;
- backdrops and fields need explicit visibility, hit, collision, extent, and
  provenance policy;
- epochs and diffs must preserve instance identity sufficiently for linked
  interaction.

Each addition still requires a forcing consumer, a heterogeneous second
consumer, deterministic carriage, and an accessible frozen realization.
Scenograph owns the portable structural meaning of a guide or nested-space
declaration as supplied by an adapter; the application still owns domain
meaning and authority. Scenograph does not own widgets, input behavior, focus
policy, or platform accessibility objects.

### Cambium

Cambium should own reusable interaction and semantic composition over Genet
elements and custom leaves:

- coordinated source selection across repeated instances;
- single, multiple, range, lasso, box, ordered, and brush selection behavior;
- a virtualized, keyboard-navigable Matrix with semantic row, column, and cell
  targets;
- Deck or panel composition with shared selection and declared scale sharing;
- interactive and accessible realizations of Scenograph-provided scales, axes,
  legends, thresholds, units, and filters;
- timeline scrubbers and before/after comparison controls;
- scene hosting with unified focus, pointer capture, overlays, and typed events;
- portal entry and exit, focus restoration, navigation, and announcements across
  nested scene boundaries;
- keyboard equivalents and announcements for direct manipulation, pinning,
  region selection, and cell activation;
- accessible table, tree, or long-form alternates for dense or spatial scenes.

The existing Cambium component rule continues to fit: props in, retained local
interaction state, typed events out. Applications lower events into their own
actions and effects. Cambium does not acquire graph authority or product
policy.

### Genet

Genet continues to own DOM, CSS, style, layout, paint, input, accessibility,
browser behavior, and platform realization. The graph-native platform adds
pressure at existing neutral seams:

- custom leaves and DOM content share one focus and pointer-capture model;
- semantic scene targets remain addressable through AccessKit, scenarios,
  genet-probe, and automation;
- DOM, scene paint, fields, video, web content, and three-dimensional surfaces
  share the wgpu device and explicit composition order;
- CSS and theme values can style scene realizations without becoming graph
  facts;
- print, snapshot, and frozen export can request a semantic alternate rather
  than capturing pixels alone;
- capability reporting names support or degradation for picking, fields,
  nested spaces, live surfaces, motion, depth, accessibility, and frozen output.

Backend-specific behavior remains behind the web-platform host contract.
Graph-native semantics do not become pseudo-web standards merely because Genet
realizes them.

### Shelfmarks and composed projections

Shelfmark v1 names one authority and one projection. Matrix introduces a
declared contract gap: its two readings may come from different authorities,
and repeated source instances may require authored delta sections to address a
particular projected instance. This direction note does not alter the v1
envelope. The [shelfmark format note](mere_docs/technical_architecture/2026-08-16_shelfmark_format_note.md)
owns the eventual citation shape after a forcing proof determines it.

Catalog collapse also cannot silently reinterpret an identifier already
written to a citation. If a former scene name was emitted as a reading,
arrangement, or other registry id, a resolver must either preserve its meaning
through a versioned alias or report the incompatibility. A catalog judgment by
itself neither proves wire exposure nor authorizes an alias.

## 8. Acceptance receipts

The [projection receipts plan](archive_docs/2026-10-06_completed_plans/2026-08-23_projection_receipts_plan.md)
owns completion evidence and receipt status; the list below records the
original acceptance shape.

The following receipts would prove the platform shape without requiring every
scene in the catalog:

1. **Repeated source identity:** one source entity appears as a Scatter point,
   Matrix heading, and Deck card. Selecting any appearance emphasizes the
   others; focus remains instance-local; every appearance exposes facet or
   derivation provenance.
2. **Two-reading Matrix:** two independently produced graphlets form the axes.
   Cells preserve relation identity or contributor provenance, round-trip
   through Graphshell, and admit an accessible table realization.
3. **Derived-mark integrity:** a Distribution or Contour scene emits derived
   marks that remain selectable, name their values and contributors, and never
   masquerade as source nodes.
4. **Mixed realization:** one scene combines DOM controls, Sprigging or GPU
   marks, accessible semantic targets, and automation addressing without two
   focus or action models.
5. **Local, remote, and frozen parity:** one scene runs interactively in a
   local host, crosses a Graphshell session to a viewer without source access,
   and freezes into a navigable semantic document or table.
6. **Field distinction:** the same sample dataset produces Territory and
   Contour scenes whose categorical and scalar meanings remain distinct through
   rendering, picking, legends, and accessible output.
7. **View-state authority:** scope, filters, selected facets, arrangement
   constraints, backdrop, and camera save and restore without entering graph
   truth; an authorized source edit still travels through the application.
8. **Coordinated-view selection:** two views over one authority contribute
   selections whose combination rule is explicit, deterministic, serialized,
   and removable. This is the receipt that may open A2's resolution half; the
   direction note alone does not.
9. **Composed citation and compatibility:** a two-reading Matrix whose axes use
   different authorities round-trips through a shelfmark-compatible citation,
   preserves any instance-scoped authored delta, and retains checkability for
   every required input. A previously emitted registry id for a collapsed name
   either reconstitutes with its original meaning or produces an explicit
   incompatibility report rather than silently selecting a newer recipe.

These receipts would demonstrate a graph-native application platform extending
the web platform while preserving the authority and ownership boundaries of
Mere, Scenograph, Cambium, Genet, and their applications.

## 9. Configurable visual and interaction language (2026-10-09)

**Status (2026-10-09):** recorded from Mark's *Explore Mere’s design language*
conversation, at his request to record, commit and push it. This section is
the shared home for that discussion across Mere and its apps. It extends this
platform direction record; it does not assign new implementation tracks or
change another plan's progress.

The decisions below paraphrase Mark unless presented as quotations.
*Inherited semantics* identifies an existing ruling; *checked precedent*
identifies existing mechanisms; *suggestion, not ruled* identifies an option
the discussion left open. Mark described the discussion as brainstorming, so
these are product direction and constraints, not a complete interaction spec.

### 9.1 Familiarity and the two working surfaces

The game reference was Rimworld, especially the familiarity people acquire
when an interface lets them interact with content, scenes and worlds. The
desired qualities are control over placement, customization, predictable
behavior, and keeping relevant things within reach without making everything
demand attention. Familiarity comes from learning what an entity does and
how the environment responds to its data and relationships.

The workbench, Frisket and forme cater to the web platform and to presenting
all kinds of content: recursively splittable tiles can present addressed
content, media, documents, attributes and resources. The dataspace explores
and contextualizes that content and its relationships. They are complementary
ways to work with the same domain, with connections between their arrangements.

*Clarified by Mark after the first study, 2026-10-09:* "I guess i was thinking
in terms of a force directed node graph, but that works too", followed by
"This is not a bad thing to have either, though. I can appreciate that both
are possible." The dataspace study should therefore lead with a force-directed
graph, while keeping the deliberate workbench/forme view as another useful
presentation of the same content. This is a clarification of the study's
emphasis, not a requirement that every application or lens use forces.

Dataspace presentation can vary with the data and the chosen view: canvas,
strata, planes, dimensionality, wallpaper, scenes and props; nodes, links and
fields; different arrangements and dynamics. Background, foreground and
overlay describe presentation roles here, not a ruling that every scene has
exactly three physical or geometric planes.

### 9.2 Selection, foreground and ambient context

The foreground is the current selection, together with nodes explicitly
pinned there. Selecting three nodes updates the HUD/GUI to reflect those
three: controls, context, and the background's presentation, membership or
scope can respond. Deselecting them returns the view to its preceding context.

Mark:

> things you've selected return to the background when you deselect them,
> but remain in the background until dismissed. what you don't interact with
> streams past with changing context, in the periphery.

Interaction therefore makes a difference to background membership. Previously
selected nodes remain available until dismissed; untouched peripheral content
can change as context changes. A lens chooses which ambient context is
relevant. This is how things stay in reach while attention moves elsewhere.

A foreground pin keeps a node informing the view while the person selects
other things. Deselecting a pinned node does not remove its foreground role.
The exact handling of view edits made during a selection, and the lifetime of
these choices across scenes or reopened sessions, need a later specification.

*Inherited semantics:* the [ambiance record](mere_docs/design/2026-09-23_ambiance_design.md)
separates attention from keeping. This selection behavior describes view
curation; it does not equate foreground pins or retained background membership
with long-term memory, nor silently change the ambient/short-term/long-term/
codicil keeping levels.

### 9.3 Opening, activity and resource identity

Opening follows the node's registered presentation. An engine that presents
through the workbench can open a tile or focus an existing one; related nodes
with open tiles are also represented in the workbench. Other presentations,
such as an applet, can unfold in situ or pop out into an overlay. The dataspace
should make activity legible: inactive, active/open, and possibly doing work
passively in the background. The final state names and indicators remain open.
Selection, activity and where something is presented are separate concerns.

Mark's example for identity was two browser tabs both showing Google: they
are two accessings of the content, even when they share the same resource.

*Inherited semantics:* the [graph semantics plan](mere_docs/implementation_strategy/2026-10-04_graph_semantics_plan.md),
especially rulings 6 and 19 and its placement table, already distinguishes
resources from surfaces. Resource identity and content statements are shared;
surfaces retain independent traversal, arrangement and layout containment,
and record which resource they show. The canvas can lift resource content
links onto surfaces. Thus a resource need not appear as a second visible node
in the familiar surface view; an appropriate lens may expose the resource
graph directly. This discussion adopts that boundary. It does not reduce
resources to addresses or contingent metadata, or merge independent accesses
because they currently show the same resource.

### 9.4 The forme as a field

A particular field can be called the **forme**: it represents and configures
the workbench's recursive tile arrangement in the dataspace. Its subregions
correspond to tiles and their grouping/nesting. The arrangement of active
nodes' tiles should be persisted and represented, so a connected subgraph can
influence the forme while its field makes the working arrangement legible.

Grouping and nesting are necessities. Relative placement is desirable;
relative size is useful when configurable. Nodes can be anchored to positions
derived from the forme arrangement, moved as needed, or position-pinned when
movement is undesirable, such as when presenting video or other media.

Layout manipulation should be an explicit gesture. Mark suggested locking
and unlocking the field: when unlocked, moving the field/subregions, or
explicitly treating a node as a tile's tab/handle, can expose drop-region
previews of the proposed split or position. Ordinary node interaction should
not inadvertently become a tile rearrangement gesture. The exact gesture and
whether a layout lock also pauses dynamics were not settled.

*Clarified 2026-10-10:* the [agreed forme draft session, §9.11](#911-forme-draft-session-2026-10-10),
now specifies unlock, node-as-tile-handle previews, scoped undo, discard and
lock-and-apply. Whether the layout lock also pauses dynamics remains open.

Keep these controls distinguishable:

| Control | Intended role |
|---|---|
| Foreground pin | Keep a node contributing to the current context while selection changes. |
| Position anchor | Give a node a target, including one derived from the forme arrangement. |
| Position pin | Keep its position fixed against dynamics. |
| Forme layout lock | Guard the explicit editing of the workbench arrangement. |

The [dynamics grammar plan](mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md)
owns the existing arrangement roles and their realization. This record gives
the interaction intent; it does not declare a new dynamics role or storage
schema.

The field's regions exist independently of their visual treatment. Mark
preferred a quiet outer region with subregions revealed on hover or selection,
and also valued permanently visible recursive boundaries. Both should be
themeable presentations of the same structure. HUD placement is configurable:
panes, overlays and other arrangements are valid options. A dockable summary
and local overlays were an assistant suggestion, not a chosen universal default.

### 9.5 Dynamics and inspectable links

Dynamics should follow the nature of the entities and their data, so the
result becomes predictable as the person learns that data. Directly handled
things should remain available and behave consistently; related peripheral
things can be more fungible until selected. Dynamics can reveal a relationship
before the person understands it, and inspection should explain what produced
the relationship rather than leaving motion as the only evidence.

Links should also explicate derived relationships beyond those covered by the
existing families. A selection's context menu can choose which families matter;
hovering a choice can show a swatch of what applying it would look like for
that selection. Scripts, derived-link search, Eidetic history and search
engines were named as possible ways to discover or produce context.

Mark preferred the natural word **link** and floated using *link* and *edge*
to distinguish explicit from derived/computed relationships. That split was
not decided. Using *link* generally in the UI with authored/computed/inferred
provenance was an assistant suggestion. The grammar's identifiers and
relationship semantics remain with their owning records until that choice is
made. Likewise, preview behavior does not establish a new truth mutation,
undo policy or persistence policy by itself.

### 9.6 Configurable appearance, iconography and fonts

Mark, correcting the idea of one palette:

> Have you seen tabard? One palette is not the vibe. Configurable, that is.
> Maybe some defaults like woodshed used to have.

The shared design language therefore includes configurable presentation and
useful defaults. No single palette or quiet visual treatment is mandated
across the apps. Defaults can provide coherent starting arrangements; users
must be able to customize them.

*Checked precedent:* [Tabard's theme model](../crates/system/tabard/src/lib.rs) carries
seed-based definitions, modes and custom mode sheets; its
[registry](../crates/system/tabard/src/theme/registry.rs) distinguishes built-ins from
user themes and supports forking a user copy. Woodshed's
`woodshed/crates/woodshed-views/src/theme.rs` at
`e08bf4b9875a7efb02b1c17bf9fac56708712cee` has Slate, Ember, Light, Dusk,
Meadow and Parchment definitions represented as built-in Tabard themes. These
are precedents for variety, not a ruling to impose those six defaults on every
app. The [theme modes plan](mere_docs/implementation_strategy/2026-07-05_theme_modes_plan.md)
owns its implementation history and remaining consumer work.

Mark also named **Emblem and Pictograph for iconography**. The existing roles
fit together: [Emblem](https://crates.io/crates/emblem) encodes/decodes compact
IconVG vector graphics, including icons and glyphs; palette references permit
retheming. [Pictograph](../crates/canvas/pictograph/README.md) derives
deterministic node faces from content addresses, with palette slots and
coarse/full levels of detail, using Emblem's encoding. Authored symbols and
derived entity faces can share the rendering and theme mechanisms. This does
not claim that Emblem supplies a finished authored icon collection.

Mark:

> Fonts, whatever fonts you have on your computer. I would like people to be
> able to configure that too. You can use a nice arrangement of open source
> options for defaults.

Installed fonts should be selectable by the person; curated openly licensed
font combinations provide defaults. Role-based choices for UI, reading,
source and labels, with size/weight/spacing controls and user overrides, were
assistant suggestions, not a ruled settings schema. The defaults are not a
requirement to use one family throughout the stack. As a checked precedent,
`knot-editor/apps/desktop/assets/fonts/README.md` and
`knot-editor/apps/desktop/src/fonts.rs` record bundled IBM Plex Mono and
Source Serif 4, with SIL Open Font License notices. They demonstrate an
existing openly licensed combination, not the default chosen here for Mere.

### 9.7 Open choices and handoff

Later implementation work should resolve the choices left open: link/edge
terminology; exact HUD defaults and activity indicators; font preference
roles and platform realization; selection restoration after intervening view
edits; the lifetime of pins and retained background membership; family-choice
persistence and swatch commit/undo behavior; and whether forme layout locking
affects dynamics. No implementation completion is inferred from this record.

| Consumer | Direction to carry into its existing work |
|---|---|
| Cambium/workbench/forme | Recursive regions, explicit layout manipulation, drop previews, persistent arrangement, configurable HUD placement. |
| Graph semantics | Preserve resource/surface identity; keep attention, activity and presentation distinct. |
| Scenograph/projections | Selection-shaped context, lens-defined ambiance, themeable field regions, and previews scoped to the selection. |
| Dynamics | Meaningful and explainable relationships, stable direct interaction, distinct anchors and position pins. |
| Tabard and appearance consumers | Configurable themes, varied defaults, installed-font choice with openly licensed defaults. |
| Iconography consumers | Emblem for vector graphics and Pictograph for derived node faces, integrated with theme palettes. |

The [Scenograph editor plan](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md),
[Cambium architecture](cambium_docs/technical_architecture/2026-09-03_cambium_architecture.md),
and the other owning plans linked above remain the implementation homes.
Apps can cite this section as the shared design direction rather than copying
it into separate competing records.

**Consumer reconciliation (2026-10-09):** the [ambient relation-lens proposal](mere_docs/design/2026-09-23_ambiance_design.md#10-proposal-focus-driven-relation-lenses-2026-10-07)
applies §9.2 to temporary lens reasons and previously selected background
membership, while preserving the keeping axis. The [app composition brief](cambium_docs/research/2026-10-06_app_composition_brief.md#11-current-consumers-and-design-direction-2026-10-09)
links retained sessions, owner-served projections and the separately granted
Moot applet experiment. The [Scenograph handoff](mere_docs/research/2026-10-09_scenograph_codex_handoff.md)
records current site state and overlapping viewer ownership. These are
implementation context and proposals; they do not settle the open choices
above or infer consumer completion from this direction record.

### 9.8 Voice clarifications and research boundaries (2026-10-09)

Recorded at Mark's request after discussing the six criticisms of §9. These
clarifications amend the earlier wording without turning brainstorming
examples into a finished specification or new implementation assignments.

1. **Theme values and their use.** Mark agreed that color carries too many
   meanings and that Tabard needs a token/presentation system for applying
   authored themes. Data encodings, selection and activity need distinguishable
   treatments. *Suggestion, not ruled:* category fill, selection outline and an
   activity badge, with configurable assignments and conflict previews. Exact
   token roles, defaults and precedence remain open.
2. **Selection and inspection.** Mark agreed that selection carries too much
   behavioral weight and some responsibilities should move to explicit
   interactions. Hover previews are welcome; touch needs separate exploration.
   The HUD and GUI are still being defined. Old Meerkat can supply precedents,
   but copying its interface is not the chosen solution. Which actions establish
   selection or retained background membership remains open; the existing rule
   for previously selected material stands.
3. **Deselecting preserves edits.** Changes made while selected "still count".
   Returning to the preceding context means an unselected view, not restoration
   of an earlier snapshot. This resolves §9.2 and §9.7's question about whether
   deselection discards intervening edits. *Reading, not ruled:* context should
   be recomputed against current edited state. Exact presentation and lifetime
   across scene changes or reopening remain open.
4. **Fields participate in the scene.** A forme arrangement should coexist
   with other subgraphs and arrangements without taking over the whole scene.
   Fields should be capable of applying arrangement, dynamics or a projection
   to affected material. Scripting includes general-purpose behavior and
   invoking a graph-held applet when a node enters a field. Mark's feed example
   lets overflow enter a field and dissolve or move into the background; these
   are examples, not a retention or deletion policy. Field overlap, membership
   and entry semantics, nested layout composition and action authority need
   research. Foreground pins, positional controls and layout locks keep their
   distinct purposes; locking acquires no implicit scene-wide override.
5. **Fonts retain configurable roles.** Defaults should expose granular font
   roles so people can choose fonts while retaining useful heading, size and
   text patterns. Installed fonts and alternative defaults stand. Browser font
   preferences are a precedent, not a complete role schema. Font changes must
   be considered with measurement and arrangement; exact remote/frozen fidelity
   remains open.
6. **Show dynamics through meaningful forms.** Mark wants visible accounts of
   forces, especially simultaneous contributions. Vectors, shape and edge
   treatments are candidates; arbitrary particle effects or more color alone
   do not satisfy the intent. Nodes, edges and fields participate in a cohesive
   scene. A relation's appearance may explain associated dynamics, but relation
   membership does not automatically install a force. Exact marks, reveal
   controls and contribution attribution remain open.

The [Scenograph editor plan](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md)
carries selection/preview and scene-authoring consequences; the
[field regions plan](mere_docs/implementation_strategy/2026-06-13_scriptable_field_regions_plan.md)
carries scoped behavior and composition questions; the
[dynamics grammar plan](mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md)
carries force explanation and constraints; the
[theme modes plan](mere_docs/implementation_strategy/2026-07-05_theme_modes_plan.md)
carries theme application and font-role research. Existing work order and
viewer ownership holds stand. This pass records direction, not runtime proof.
Tabard's §9.6 source links were corrected to its current system-crate home.

### 9.9 Plan through graph primitives (2026-10-09)

Mark's follow-up asks to plan through **nodes, links and fields**, represent
all three in the graph, then connect them to dynamics. Their presentation,
inspection, attributes and behavior should form a cohesive authoring model.
This is direction for planning, not a requirement to turn every primitive
into the same ordinary node representation.

Mark proposes **link** for ordinary visible relationships and **edge**
specifically for the association between a resource node and the normal
visible "content node". The associated resource normally stays out of the
typical graph view and carries metadata including RDF type information.
Both resource and content nodes have their own attributes. This advances
§9.5's terminology discussion; exact adoption still needs reconciliation
with the graph-semantics owner and existing data contracts.

*Checked boundary:* graph semantics already distinguishes `ResourceNode`
from `SurfaceNode`, with `Node` retained as a compatibility alias. Multiple
independent surfaces may show one resource. "Content node" therefore needs
an explicit mapping before renaming types or describing the resource as
owned exclusively by one appearance. Existing edge handles also address
Surface-Surface and Resource-Resource assertions; the proposed narrower
word cannot silently change their meaning. "Attributes" here is Mark's
broad description; the pending attribute/resource/tag dissolution still
needs an ownership mapping for each stored fact.

*Planning proposal, not ruled:* identify each primitive's stable address,
owned facts and attributes, permitted edits, scene appearances and explicit
dynamics participation. Link identity and explanatory appearance should
remain usable even when no force is attached. A field's rules and membership
need an inspectable representation as well as its geometry. Shared source
attributes stay with their owner while per-view choices stay with the view.

The [Scenograph editor plan](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md)
carries the proposed reference-host proof and the
[facet dissolution brief](mere_docs/research/2026-10-08_facet_dissolution_brief.md)
retains its graph-semantics ownership. Graphshell is the reference host for
this proof. The transcript's "GraphQL" spelling was interpreted too literally
in the initial planning record and corrected on 2026-10-09. Storage mapping
and dynamics participation remain planning questions; this record does not
claim an implementation of them.

### 9.10 Presentation rules agreed (2026-10-09)

Mark, on §9's follow-up discussion and the six examples below:

> I think those are all wonderful graph interactions, your targets and
> conditions and presentation decisions, they all make immediate sense to me.
> Target->condition->effect, rhai, etc. all agreed. Proceed

Agreed direction: author presentation as **target → condition → effect**, with
explicit precedence, fallbacks and explanations of which rules matched.
Representation decisions precede measurement and arrangement; user placement
and pins survive presentation changes. The examples Mark accepted are:

| Target and condition | Presentation decision |
|---|---|
| Image-bearing node in an overview | Thumbnail with its identity marker. |
| Selected node | Configured attributes and controls. |
| Node below a screen-size threshold | Compact face, with a separate threshold for restoring detail. |
| Membership link in a hierarchy scene | Containment. |
| Derived link being inspected | Its rule, inputs and explanation. |
| Unlocked forme field | Subregions and applicable drop previews. |

The interactive authoring surface is target → condition → effect with a live
swatch; Rhai serves more involved rules, under the Scenograph editor plan's
SE36–SE42 binding and dependency rulings. A first aesthetic study uses the
same mixed-content scene across three editable starting treatments (paper
and ink, workshop, night), independently of arrangement and representation.
These are study presets, not prescribed palettes for every application.

The [Scenograph editor plan](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md)
owns track R1 for the shared declarative model and first interactive study.
Production host adoption and the Rhai runner are separately qualified there.

### 9.11 Forme draft session (2026-10-10)

**Agreed interaction; Graphshell adapter implemented, headed qualification pending.**
The [editor plan's draft progress](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md#workbench-forme-draft-2026-10-10)
records its scoped native/build checks; other-host adoption remains separate.
Mark described unlocked nodes
as tile handles: dragging a handle previews the corresponding tile region,
split and nesting; dragging another node into the forme adds it to the proposed
arrangement. Locking agrees to commit those changes. Asked how to discard the
draft, Mark accepted a per-forme draft and undo history with an explicit
discard action: "Alright, I'm down!"

| Action | Effect |
|---|---|
| Unlock | Start an editing draft from this forme's committed arrangement and projection geometry. |
| Undo / redo | Step through arrangement gestures within this draft. |
| Discard changes | Drop the draft, return to the committed arrangement and lock the forme. |
| Lock and apply | Commit the draft as one arrangement change, undoable afterward, then lock the forme. |

Both the graph field and the tiled workbench can preview the same draft.
Pending changes must be visible. The saved arrangement remains unchanged
until apply; hiding handles or ordinary node selection is not a commit gesture.
Discard abandons the editing session, while undo reverses individual gestures.

The draft covers membership, grouping, splits, nesting, proportions and local
placement. Semantic arrangement facts stay in Forme; geometric details stay
in their projection state. It does not snapshot document contents, playback,
resource metadata or unrelated graph edits. Discarding a node's proposed
membership leaves the node and its content accesses intact. Dropping the
draft also drops its private history; future unlocking starts from the
committed state rather than undoing into a previous editing session.

Undo routing follows the active editor. Arrangement gestures use this forme's
draft history, typing in a document uses its document history, and committed
arrangement changes use the owning host's saved-change history. A position
pin, foreground pin and dynamics controls retain their separate meanings.
This interaction settles how layout edits are drafted and accepted; it does
not require another force-composition model or prescribe a scene's appearance.

The [editor plan](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md#forme-draft-follow-through-2026-10-10-agreed-interaction)
maps the shared history and layout substrate to implementation steps and
qualification. The first Graphshell adapter now implements the draft and saved
arrangement undo, with its interactive check pending; this agreement and source
qualification do not substitute for a headed runtime receipt.

### 9.12 Mixed-content scene and embedded forme study (2026-10-10)

**Proposal forwarded by Mark from the design-language agent; not a new
ruling or implementation receipt.** The presentation-rule direction was
separately accepted in §9.10; this subsection proposes its richer study and host
integration. Use one small scene containing a
document, images, a playing video, related nodes and a forme with two
side-by-side webpage accesses. Dress the same material in several editable
treatments. Paper and ink, a warm workshop and a luminous night are candidate
presets, not a decision about Mere's palette. Compare typography, silhouettes,
surfaces, borders, shadows, textures, backdrop, props and motion together.
Recognizable identity, readable selection and continued reachability are
part of the study, including Emblem and Pictograph identity cues.

The proposed authoring breakdown is reading; representation; encoding;
arrangement and dynamics; composition; appearance and interaction. It helps
discuss independently editable choices, but is not a replacement recipe
schema. In the [projection grammar catalog](mere_docs/research/2026-08-15_projection_grammar_catalog.md),
representation choices belong to encoding; arrangement produces the scene,
and realization supplies its measured content. A node may become an icon,
card, thumbnail, live pane or nested view while retaining its source identity.
A link may become a connector, adjacency, containment or a matrix cell. A
field may disclose its boundary, subdivisions or influence. These are
presentation alternatives; a drawn region alone does not implement field
membership or behavior.

**Accepted rule direction; proposed study:** target → condition → effect, with a live
swatch and an explanation of the matched rule, precedence and fallback.
Examples include thumbnail faces for image-bearing nodes, configured details
on selection, containment for membership, provenance for an inspected derived
link and drop previews for an unlocked forme. Compact/detail transitions need
distinct entry and exit thresholds to avoid flickering near a size boundary;
the relevant size and behavior after measurement remain design questions.
Choose representation before measuring its footprint and arranging it.
Paint-only changes can avoid a new layout; fonts, geometry and representation
changes must invalidate affected measurements. Preserve authored placement
and pins through that recomputation, and explain incompatible constraints.

[Mosaic's core](https://idl.uw.edu/mosaic/core/) is a reference for coordinated
selections and parameters. [Vega-Lite conditions](https://vega.github.io/vega-lite/docs/condition.html)
are a reference for predicate/selection-driven visual encodings with fallbacks.
They do not specify Mere's whole-representation dispatch or field actions.
In particular, Vega-Lite's default treatment of an empty selection is not
automatically Mere's deselection behavior. A presentation rule matching a
node does not grant permission to run a graph-held applet.

**The embedded forme:** opening webpages A and B establishes independent
content accesses associated with their resources. Returning to the graph
shows their forme as a field whose member placements correspond to the
existing split. Hover reveals its subdivisions; ordinary selection changes
context, while explicit open/focus returns to the existing tile and session.
Unlocking exposes layout handles and drop previews. Accepted edits should
update the owning workbench state and re-project both presentations.
Deselection preserves those edits (§9.8).

There are two coordinate levels: the forme in the surrounding scene, and
members inside the forme. Moving the outer field can carry its members while
their local composition stays fixed. An anchor, position pin, foreground pin
and layout lock remain distinct. Overlap with another field must disclose
which placement constraints and dynamics apply; this proposal does not decide
that precedence or whether a layout lock pauses motion.

**Checked substrate at published Mere `7bfb293d`:**
[Forme's arrangement](../crates/forme/forme/src/arrangement.rs) is a semantic
graph; it deliberately excludes coordinates and split ratios. Geometry belongs
to projection state. [Platen's workbench](../crates/platen/platen/src/workbench.rs)
already projects its recursive layout into Genet's `TileTree`, with host
events applied back to the owning layout. The missing bridge is from those
existing identities, geometry and document sessions into the scene's forme
presentation, not a second authoritative tile tree. The current
[Scenomise compiler](../crates/cambium/scenes/scenomise/src/projection.rs)
emits card representations; the richer mixed-content realization and generic
presentation-rule editor still need integration. Graphshell supplies graph
context, Pelt retained document viewing, and Turnstone their browsing
composition, as the [suite census](2026-08-22_turnstone_suite_composition_and_capability_census.md)
describes. The composition is direction, not evidence of this bridge shipping.

The [editor plan](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md#mixed-content-study-2026-10-10-proposal-not-opened)
records candidate steps and done-conditions. A study matrix may compare theme
on one axis with representation or arrangement on another; the shipped grid's
axis contract is not expanded by this proposal. Keep source material and access
identities stable across comparisons, disclose live versus captured faces,
and evaluate appearance and behavior together.

### 9.13 Backdrop, context, props and reset (2026-10-10)

**Mark's correction to the backdrop cut.** The earlier Clear / Ambient /
Props / Field mode list mixes different concerns. Mark prefers Clear as a
reset action: return the scene to its initial arrangement relative to its
backdrop. Contextual ambient nodes and props are classes or roles of entities
within a scene, with independently configurable presentation and behavior.
This revises the consumer direction behind SE86, not the earlier mechanical
backdrop receipts.

| Concern | Direction |
|---|---|
| Reset | An action that restores an arrangement relative to the scene's backdrop. It is not another scenery mode. |
| Ambient content | Contextual propositions about what is relevant to foreground entities; its display can be turned off. |
| Props | Scene entities with authored attributes, presentation or scripted behavior, including objects relevant to contextualizing the content. Their display can be turned off. |
| Backdrop | The scene's set, potentially with depth, animation and interaction, rather than only static wallpaper. |
| Physical participation | Collision, forces and other behavior remain separately configured; being a prop or ambient entity does not decide tangibility. |

Static wallpaper, animated scenery, interactive scenery and scenery with
scripted entities are possible treatments. This is capability direction, not
a mandatory ladder, four mutually exclusive modes or a claim of a shipped
interactive-backdrop runtime. Turning off contextual content or props does
not by itself specify collision or simulation participation. Define those
controls by the effect they change rather than infer behavior from a class
name. Context discovery, paint visibility, input interaction and physical
participation are distinct questions.

Mark describes the foreground as pinned, currently selected or recently
interacted-with entities. This adds recent interaction to the foreground
criteria in §9.2. Its duration and transition back to retained background
membership remain to be designed; it does not repeal the previously selected
material's availability until dismissed. Contextual relevance does not imply
that an entity must be noninteractive, intangible or stored permanently.

**Checked portable substrate at published Mere `773a0dc2`:**
[Backdrop](../crates/cambium/scenes/sceno/src/scene.rs) already carries
independent `visible` and `collidable` flags. It deliberately stays outside
ordinary item picking. The canvas binder inspected separately at local main
`d572b322` preserves that distinction, including invisible obstacles; its B1
draft record is not a claim that this binder exists in the published baseline.
Interactive props need a connection to scene entities and their existing
input/action authority; painting a backdrop does not establish that connection.
[AmbientSim](../crates/canvas/pictograph/src/canvas/ambient/mod.rs) is decorative
animation, distinct from the contextual ambient nodes described here. Its
advance/paint seam is not the interactive scenery or relation-lens contract.

**Remaining design:** identify the reset baseline and what it restores.
Resetting arrangement coordinates, rewinding animation and resetting a
scripted world's state are separate effects. Their combination, camera and
pin handling, undo and interaction with an open forme draft are not decided
by renaming Clear to Reset. Depth and richer scenery also require a forcing
consumer; today's portable 2D transform/footprint contract does not establish
a complete spatial scene implementation.

The [editor's B1 follow-through](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md#b1--backdrops-in-the-graphshell-viewer-se86),
[dynamics plan](mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md#backdrop-control-correction-2026-10-10)
and [ambiance continuation](mere_docs/design/2026-09-23_ambiance_design.md#backdrop-and-context-distinction-2026-10-10)
carry this correction. The shared design is recorded here; controls, legacy
mode migration and richer input/runtime behavior remain implementation and
design work with their existing owners.

### 9.14 Design work ownership (2026-10-10)

**Mark's allocation of the remaining design work:**

| Work | Owner and scope |
|---|---|
| Scene authoring and presentation rules | A research lane; no individual owner named in this allocation. Carry forward the accepted rule direction and R1 evidence while researching broader authoring. |
| Theme and typography roles | Tabard agent on q-pc. Editable theme definitions, semantic roles and typography customization. |
| Selection, inspection and ambient context | Projection grammar agent, this conversation. Fundamentals are established; concentrate on remaining integration and concrete controls rather than another foundational design round. |
| Field authoring and behavior | Projection grammar agent, this conversation. Membership, extent, placement, influence and projection, including the workbench/forme correspondence. |
| Node, link and field editors | Projection grammar agent, this conversation. Inspect and edit the intended entity or occurrence through its existing authority. |
| Visible dynamics explanations | Dynamics agent. Disclose applicable contributions, constraints and their effects through inspectable visual explanations. |
| Script and motion authoring | Dynamics agent. Scripted behavior and authored motion, including the runtime side of field actions. |

This allocation does not reopen the established distinction between selection,
inspection and explicit open/focus. Deselection preserves edits; attention is
separate from keeping, and foreground pins are separate from position pins.
The [ambiance design](mere_docs/design/2026-09-23_ambiance_design.md) and
Scenograph's selection rulings remain the starting contracts. Residual gesture,
retention and linked-appearance integration should resolve specific gaps.

For fields and primitive editors, the next design pass should make the
inspect/configure loop concrete: which identity is edited, which members are
affected, which placement constraints apply, and how overlapping fields are
explained. Ordinary additive forces already compose; incompatible placement
or state writers need explicit treatment. The agreed forme draft in §9.11
supplies unlock, scoped undo/redo, discard and lock-and-apply behavior.

Field authoring owns how a user selects members and configures a field's
effects. The dynamics lane owns force execution, visual explanations and the
script/motion authoring seam; their interface must describe targets, events
and effects consistently. Theme values and font roles come from Tabard.
Graphshell remains the reference host. Existing source authority, workbench
arrangement ownership and site viewer ownership are preserved. This is a
design allocation, not a new runtime receipt or a dispatch to another agent.

### 9.15 Field and primitive editor draft (2026-10-10)

Mark asked the projection grammar lane to proceed with its assigned design
work. The [field authoring draft](mere_docs/implementation_strategy/2026-06-13_scriptable_field_regions_plan.md#field-authoring-draft-2026-10-10)
specifies member/extent, placement, influence, projection, action and appearance
controls. The [primitive editor draft](mere_docs/implementation_strategy/2026-10-07_scenograph_editor_plan.md#primitive-editor-draft-2026-10-10)
specifies inspect/edit flows for content accesses, Resources, scene appearances,
link bundles/source records and fields. These are reviewable proposals, not
additional rulings. The established selection fundamentals and agreed forme
draft behavior remain their starting contracts.

Source at published Mere `86f2a3b8` supports a bounded workbench/forme bridge
and existing-field inspection, with headed acceptance pending. It also exposes
two editor gaps: overlapping field picking follows hash-map paint order, and
the field-level strength shortcut reads one coupling but writes all of them.
The drafts propose explicit edit targets and per-effect controls. They preserve
the difference between declared arrangement membership, occurrence-local
spatial eligibility and influence weight, and between an inspected link family
and an exact source assertion.

Open checkpoints are the spatial-membership test, incompatible exact-placement
policy, membership-event causes and field/forme storage association. Refusing a
conflicting placement edit and reporting initial matches without firing entry
actions are recommendations pending rulings. Theme roles, presentation-rule
precedence, visual force explanations and script execution remain with their
assigned lanes. No runtime source or qualification was changed by this pass.
