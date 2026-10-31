# Graph view curation plan: folds, relation cells, source time, share records, the swatch and the gloss outline

**Status:** 2026-10-28: cut from `2026-08-03_graph_view_curation_and_interaction_plan.md`,
`2026-06-27_swatch_primitive_plan.md` and `2026-06-23_gloss_outline_lens_plan.md`, archived in
`archive_docs/2026-10-28/`. Curation's build order C0-C6 reads as complete on macOS (root-Canvas and
bounded-Swatch folds, Knot's catalog folds, source time with two real sources, mer3ly history, share
records), with a few unmet clauses. The swatch's meerkat halves left with meerkat (`c5f01064`); its
successor is Turnstone's swatch pane over Cambium's `GraphCanvasSwatch`. The gloss outline's library
survives and no host renders it. Open: the Collapse descendants step, remote relation targets, the
layered visibility stack, per-cell edge thickness and a shared edge renderer, the classifier's
remaining clauses, swatch templates, and the gloss host, nesting axis, signals and knot-ification.

**For:** one curation system for the graph in several hosts, not a universal graph rewrite: pull a
node, curate relation cells, fold a selection or an explicit hierarchy, scrub a source's history,
share a proven view (Mark, 2026-08-03); the swatch as the fourth graph primitive with
cells-as-edges, a non-propagating visibility stack and templates (Mark, 2026-06-27,
primitive-first); and the gloss outline as a djot projection that doubles as the seed of the first
notetaking feature (Mark, 2026-06-23). Graph truth, per-view choices, historical source state,
projection geometry and scene delivery keep separate owners.
Not here: Windows, Linux, assistive-technology and VTT acceptance (keyboard gestures, live
screen-reader interaction, recovery, Windows and Linux headed acceptance, and the Isocosm VTT
campaign consumer, whose Overmap has no fold controls) go to
`2026-10-28_platform_qualification_plan.md` and are not carried here; field-region edge visibility
as a contributor to the stack and the field runtime (`2026-10-28_dynamics_grammar_plan.md`); the
one-stroke link drawing and relation-cell identity as assertions (`2026-10-28_link_model_plan.md`);
the Graphshell protocol and reference host (`2026-10-28_graphshell_host_plan.md`); the Apparatus
library itself (`2026-10-28_diagnostics_and_accessibility_plan.md`); the lineage swatch and previous
and next controls that need this plan's gloss and swatch hosts
(`2026-10-28_graph_semantics_plan.md`).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this plan (DOC_POLICY §8).
Evidence that matters is a test in the repository or a line in Findings.

## Constraints

- Ownership
  (`archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:95-147`): durable
  graph history is `GraphJournal` or a source adapter; durable local curation (hidden cells, folds,
  pins, time cursor, camera, focus) is pandect's `view_intent_store`, called "view state" in prose,
  with no third type named `ViewIntent`; root physics stays Seiche plus Canvas; Sceno stays an
  output contract and a Scenograph contract bump needs a case two consumers cannot express through
  existing identities.
- Temporary highlight, durable per-view visibility and graph-truth mutation are three layers; the
  default stack stays `GraphDefault < GraphViewOverride < SelectionOverride`, a higher layer winning
  and never writing down; hiding relaxes the spring in that instance only and never drops a node out
  of a subgraph (Mark, 2026-06-27; curation `:171-194`; swatch `:221-232`).
- Source time, scene revision, arrangement and animation are different things. Scrubbing builds a
  temporary historical graph, never mutates the live graph and never writes the viewed cursor back
  as truth; Restore from here is a separate confirmable action (curation `:212-241`).
- A fold records a stable id, ordered members, source scope and boundary policy in view state, adds
  no source-truth group node, and saving a group into graph truth is a later explicit command;
  Collapse descendants needs an explicit relation family and direction and never infers hierarchy
  from connectivity (`:195-211`).
- Sharing: the live record is producer-owned and versioned and grows only from proven fields;
  secrets, private graph content, local paths and capability tokens never enter a URL; a recipient
  without source authority gets a refusal, not a redacted empty graph (`:242-272,429-446`).
- Stop rules (`:508-531`): stop a shared extraction until two real consumers use the behaviour; stop
  a Sceno or Graphshell contract change until local proofs cannot express the need; stop historical
  relation playback at node activity if the extractor cannot prove the old relation from old inputs;
  stop a view action that changes graph-truth fingerprints; stop after each slice's receipt wall.
  Non-goals: reviving the archived Graphshell checkout, history or intent in Sceno, Scenotime
  revisions as cursors, applying today's relations backward through time, a Cambium-owned solver,
  one maximal share schema. Evidence levels (contract, truth, projection, interaction, headed,
  remote) are reported honestly; a green unit suite is not headed proof (`:469-506`).
- Swatch and gloss (`archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:12-47,257-277`;
  `archive_docs/2026-10-28/2026-06-23_gloss_outline_lens_plan.md:228-249`): the swatch model is
  canonical in the design record and is not re-derived; the P6 retrofit of the gloss minimap onto
  the swatch is declined and revisited only if a third swatch consumer appears; the outline is
  chrome-understood DOM; `outline_rows` and `outline_djot` stay uncapped and the pane truncates at
  the view layer by a live row budget plus a depth ceiling; gloss keeps bare scale metrics and the
  breakdown lives in apparatus; one gloss surface switches form factors, no split.

## Phases

### U1. Collapse descendants and the fold follow-ons
C3's selection fold is integrated in the root Canvas, the bounded Swatch and Knot's catalog; its
step 5, Collapse descendants over an explicit relation family and direction, has no landing claim,
and Knot does not infer document parenthood. Deferred: saving a group into graph truth, a user
setting that pins on release, Restore from here as a confirmable delta and engram action, and
dense-zoom relation bundling (selection resolving to the bundle first).
- Done when: fold and expand round-trip member identity and placement; internal cells return;
  boundary counts are exact; changing the selected hierarchy family changes the descendant set
  predictably; graph truth is byte-for-byte equivalent before and after view-state removal.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:160-161,191-193,209-210,239-240,350-368,700-701`

### U2. Remote relation targets and the shared drag machine
Extend Graphshell's item-only invocation target to a protocol-owned enum covering item, relation and
region once local proofs need remote edge actions, the endpoint mapping the epoch-local table id
back to its source cell; prove stale epoch or revision and unauthorized-intent receipts for edge,
fold and share actions. After two consumers use it, extract the shared drag state machine and remove
the Canvas duplicate or document why host coordinate conversion stays separate. The status is silent
on both. Sceno and Graphshell contract changes stop until two consumers cannot express the need.
- Done when: two relations on the same endpoints are separately picked, highlighted, hidden, shown
  and inspected in Canvas and Swatch; remote table-id round trips are proven before the protocol
  version changes; truth deletion stays a separately labelled command.
- Needs: the Graphshell host plan for the protocol change.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:144-147,316-318,336-348,483-484,511-512`

### U3. Source time beyond live-only
Joined clients and pre-origin checkpoints stay live-only until the session handshake carries a
verified origin plus a public log. `GraphJournal` is to be installed and persisted by a session host
(C4 owns the wiring; its unit tests are not a live-session proof). Human-readable journal
timestamps, if wanted, need a versioned envelope or side index, not a changed `CapturedDelta` or
journal wire format.
- Done when: not stated for joined clients; C4's condition, met for local sources, is that the same
  fixture scrubs through journal sequence and Git commit cursors, Radial, Timeline and one non-grid
  arrangement show the selected snapshot, switching arrangements keeps the cursor, Return to live
  resumes updates, and scrubbing leaves truth unchanged.
- Needs: the session handshake carrying a verified origin plus public log.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:17-18,75-77,370-396`

### U4. Visibility stack, per-cell thickness and the shared edge renderer
Build the layered `GraphDefault < GraphViewOverride < SelectionOverride` stack: wire the hidden-cell
set through it and persist the per-pane override. Make edge thickness per cell (uniform per fan lane
today) and have the orrery and the connections swatch share one element-model edge renderer,
re-targeted to Canvas and `GraphCanvasSwatch`. Related backlog: per-family edge toggles and a
per-family visibility UI after the consumer audit (pandect already persists `hidden_relations`),
sub-kind selector editing, and a relation-kind picker (every `assert_selected_relation` caller
passes `UserGrouped`).
- Done when: hiding a cell in one instance relaxes its spring there and nowhere else; the cell stays
  drawn and live in every other instance, in a selection and in a Linked subgraph; a `GraphDefault`
  hide applies as the base everywhere; subgraph membership is unaffected by any hide; edge thickness
  is per cell and the two hosts share the render.
- Needs: the open question on the link model's one-stroke rule; the dynamics grammar plan for
  field-region visibility.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:197-256`;
  `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:324-329`; tails
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:253-262,336-341,563-567`

### U5. Classifier remainder and swatch templates
The shape classifier survives (`classify`, `classify_selection`); the swatch's remaining P3 clauses
were written against the meerkat swatch and have no host: the live projection toggle, frontier
ghosts, Linked-derivation crystallize (Component and Ego) and the Astroid option, chip-click
crystallize, and the contextual detectors (Bridge, Facet, Frontier, WorkbenchCorrespondence).
Templates (P7) are a named preset over (scope, layout, lens, projection, mode) with a lock,
applicability conditions and per-persona persistence in `PersonaSettings`, three verbs on unlocked
swatches (copy the orrery's config, reset to default, apply a saved template), and built-ins node
editor, minimap and connections. The Scope, `SwatchInstance` and element-model unification (P2b)
waits for a third consumer.
- Done when: P3, the connections swatch names the dominant shape, re-derives it live as the
  projection toggles, ghosts the one-hop frontier and crystallizes the selection to a Session or
  Linked subgraph; P7, a swatch instantiates a named template and defaults to its config, lock
  toggles with reset-to-default always available, the three verbs work on unlocked swatches,
  templates persist per persona across graphs, and a locked or bound template renders a
  purpose-built UI over its own subgraph.
- Needs: Turnstone's swatch pane over `GraphCanvasSwatch` as the host.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:12-26,104-105,138-141,155-195,287-303,516-518`

### U6. Gloss outline host and lens
The glossary library survives (`outline_rows`, `outline_djot`, `graph_metrics`, with the row cap and
depth ceiling in `gloss.rs`); no host renders the outline and Turnstone does not use it. P1 is the
host DOM section (rows route a select-by-URL action and take node colour and selection) and P1a its
accessibility wiring in the hand-built per-pane way. P2 makes the nesting axis configurable (URL
structure by default; explicit Containment, an Arrangement sub-kind or a Semantic family through
`forme::ProjectionLens`) and the scope pickable (full graph, active selection, subgraph). P3 feeds
importance and community from the pictograph signals producer with degree and component fallback. P4
opens the outline as an editable djot knot, drag-to-reorder and promote writing back through
`assert_relation` with typed provenance. Decision 4's split stands: gloss keeps bare scale metrics
(nodes, edges, components) and the full breakdown (family histogram, orphans, largest component)
belongs to a new Apparatus "Graph" section, whose host left with meerkat. Deferred: the
outline-versus-swatch form-factor toggle (gloss design G3), relation-driven outlines (a P2 pluggable
lens), other read-side textual projections, and the three-way auto-size, moot unless a host returns.
- Done when: P1, opening the gloss shows the live outline and counts and clicking a row focuses the
  node, headed-verified; P1a, a select-by-URL action against an outline row's URL focuses the node
  as a click does; P2, the outline re-nests by a chosen edge family, re-scopes, and its visible row
  count tracks the pane's resize live; P3, importance and community appear when signals are present,
  fallback otherwise; P4, the outline opens as an editable note and a reorder persists as a
  containment or arrangement edge.
- Needs: a living host (question below); the signals producer at
  `crates/canvas/pictograph/src/signals/`.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-06-23_gloss_outline_lens_plan.md:5-13,108-110,146-148,171-180,186-249,355-368`

## Open questions

- Curation C6 completed while C3's Swatch proof was pending, against the sequencing rule; later
  entries supply the Swatch proof but the fork is not marked resolved. Accept the out-of-order
  completion and amend the rule, or reopen C3's Swatch gate? From:
  `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:464-467`
- The status is silent on C0, C1 and C2, while the swatch plan says curation C2 claims the layered
  visibility stack. Is the stack built? Is `GraphJournal` installed and persisted by a session host,
  or only reached through adapters? From:
  `archive_docs/2026-10-28/2026-08-03_graph_view_curation_and_interaction_plan.md:75-77,324-329`;
  `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:246-249`
- The swatch's P4 draws each cell as its own fanned line with per-cell thickness; the link model
  rules one stroke per link and retires the fan. Do per-cell thickness and the shared fan renderer
  still apply? From: `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:197-219`; link
  model L4 (`technical_architecture/2026-10-15_link_model.md:191-197`)
- True parallel edge instances were deferred to an archived multigraph storage plan; the link
  model's separately attributed assertions may answer them. Unruled. From:
  `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:307-308`; tails
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:336-341`
- Where does the latent staging relation live, a gloss-owned subgraph store or a latent kernel edge
  family, and is the default chain or bus? Explicitly Mark's call, unanswered. From: tails
  `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:243-247`
- The per-table stats and canned regression log question is the diagnostics and accessibility plan's (`2026-10-28_diagnostics_and_accessibility_plan.md`).
- The swatch's open design questions: the element model's shape (lean: adapt over `Projection` and
  `MinimapDescriptor`), `FocusCardKind` versus a general embedder (lean: new kind), the conditions
  vocabulary for templates, per-cell fan geometry legibility before a collapse and expand
  affordance, edit-layer reach (`GraphArrangeEdit` versus the orrery's direct manipulation), and the
  classifier's ranking strength (a unified number, likely a setting). Two are written against
  meerkat and may be moot. From:
  `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:350-365`
- The swatch's third-consumer trigger for P6 was never recorded as met, though Cambium and Isometry
  now give a second-consumer seam; the P5 hide and show spring relaxation and the P4 fan and pick on
  a multi-relation pair were never headed-verified. From:
  `archive_docs/2026-10-28/2026-06-27_swatch_primitive_plan.md:282-285,485-486`
- Gloss: which host renders the outline, which edge families earn a lens, and is outline order its
  own Arrangement sub-kind (a projections-research question)? From:
  `archive_docs/2026-10-28/2026-06-23_gloss_outline_lens_plan.md:9-13,234-235,365-368`

## Findings

- 2026-10-06: the durable `ViewIntent` and its store live in pandect
  (`crates/system/pandect/src/view_intent_store.rs`, `pandect::ViewIntent`);
  `cartography::ViewIntent` is the immutable per-projection request.
- 2026-10-06: surviving swatch library parts are the shape classifier
  (`crates/mere/src/subgraph/classifier.rs`), `SessionSubgraphs::record_session`
  (`crates/mere/src/subgraph.rs`) and the per-visible-cell spring feed `visible_relation_edges`
  (`crates/canvas/pictograph/src/canvas/seiche_bridge.rs`); the glossary lives at
  `crates/mere/src/glossary.rs` with `cap_outline_rows` and `MAX_OUTLINE_DEPTH` in
  `crates/mere/src/gloss.rs`.
- 2026-10-15: fold records are curation-owned (v2 with a v1 migration), several folds may be active
  in the Canvas, boundary routing runs over the full fold set, and a scope-aware fold never hides a
  partially scoped member; exact empty scopes stay empty (`scope_to_exact_members`). 2026-10-16:
  `mere-view` accepts host-owned native `FoldRecord`s and Knot persists them in its catalog-root
  preferences, independent of source truth.

## Progress

- 2026-10-28: cut from the three sources named in Status; nothing new landed.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- The object card's Widget, Preset and type-scoped-card model and its P1 to P4. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:288-290`
- The canvas text-scaling policy (fixed, zoom-scaled, or hybrid with an LOD floor) and whether Ctrl+zoom reaches content; the paint-list ask is genet's. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:362-365`
- Yaw and tilt persistence for the isometric orrery camera; no host writes it. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:896`
