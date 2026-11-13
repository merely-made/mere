# Engine split plan receipts

Receipts for `mere_docs/implementation_strategy/2026-11-07_engine_split_plan.md` (DOC_POLICY §8).

## 2026-11-07: P0 drift audit (read-only, Sonnet subagent, no builds)

Spot-checked by the controller the same day: the backstep cycle (scenograph and scenomise depend on backstep; cambium on both), the registry features (only `mod-loader`, `observations` and `diagnostics` enabled anywhere), `pandect` depending on `tabard`, `ux-events` with no dependents, `mere-comms` absent from the tree and pinned from murm.git, and `linked-data/src/statements.rs` using `inker::EngineDocument` all hold.

# P0 drift audit: engine-side, Graphshell-set and elsewhere crates

Read-only audit, 2026-11-07, of `design_docs/mere_docs/implementation_strategy/2026-11-07_engine_split_plan.md`
(Crate dispositions, P0, D1, D10). Inker and Seiche sets are other lanes' work. Nothing was edited.

## Method and caveats

- **Lines** are Rust lines under `src`, tests included (a shell `find ... -name '*.rs' | xargs cat | wc -l`
  equivalent, `.claude/worktrees` and `target` excluded). `â€ ` = the lib lives at the crate root, not `src`
  (counted over the crate's own `.rs` files, no tests/examples). `*` = package is not in the Mere tree; counted
  from the graphshell.git checkout `~/.cargo/git/checkouts/graphshell-4e93238cfddf8326/40e8c8d` (`mere-comms`: the only
  copy, `.claude/worktrees/resident-port-821cb7`).
- **Dependents** come from parsing every `Cargo.toml` in Mere, turnstone, knot-editor, woodshed, cleromancy,
  isocosm (worktrees and `target` excluded), resolving `package =` renames and `workspace = true` aliases;
  normal, dev and build edges all count. Format: `N (first names, +rest) / external repo count`. A git or
  path dep of the same package name counts. Probes and fixtures are shown as `+N probe`.
- **Fit** is judged from `lib.rs` docs and the Cargo `description` against the modules and public items. Fork
  numbers (F1 ...) refer to the Forks list at the end.
- Package names, not directory names, are used throughout (`mere-kernel` is `crates/graph/graph-kernel`).

## Layering findings that bear on dispositions

Engine-side crates that depend on host-side or other-home crates today (each is an edge P3/P5 must cut):
`mere-kernel` to `numen` and `hest`; `mere-rdf` to `inker`; `pandect` to `tabard`, `hest`, `mere-curation`, `sceno`,
`chartulary`; `mere-registry` to `inker`, `tabard`, `tinct`, `hest`; `mere-fetch` to `pandect` and `inker`.
`seiche` and Isocosm depend on `state-witness`, which the plan keeps in Mere.

## Table 1: crates named in Crate dispositions (non-Inker, non-Seiche)

| Package | Path | Lines | Dependents: Mere tree / external | What still fits | Disposition of the rest | Name freed |
|---|---|---|---|---|---|---|
| `mere-kernel` | `crates/graph/graph-kernel` | 51160 | 16 (graphshell, mere, mere-athanor, mere-cartography, mere-chrome, +11) +2 probe / knot 1 | Resource half fits (graph/: journal, apply, revert, assertion_write, edge_data, edge_ops, predicate registry, resource, query). Root modules (6,389 lines: accessibility, actions, paint, overlay, pane, permissions, intents, host_event, host_toolkit, async_host, signal_router, address, verso_address, geometry) are host vocabulary. graph/ also carries ~4.9k of `legacy_*` migrations and surface items (node, display, chart, membership). Depends on `numen` (Seiche) and `hest`. | Split at the resource/surface seam (P5): resource half into `mere`; surface half into `incipit`; `legacy_*` retire when the migration window closes (Mark's call). The seam is not a module boundary; measure first (Constraints). | `mere-kernel` |
| `mere-rdf` | `crates/graph/linked-data` | 10794 | 2 (mere, mere-fleece) / none | Fits the engine and exceeds its description ("JSON-LD bridge"): codecs, ingest, reifier, SPARQL via spareval (`query` feature), vocab. Misfit: `statements.rs` (309) applies Inker `LinkStatement` into the graph, so the crate depends on `inker`; the graph-to-RDF projection is tied to the legacy graph. | Fold into `mere` (P5). `statements.rs` split at adapter (fork F2). Projection goes with the legacy snapshot adapter. | `mere-rdf` |
| `pandect` | `crates/system/pandect` | 27097 | 9 (graphshell, graphshell-host, graphshell-mere, graphshell-web, mere-athanor, +4) +1 probe / cleromancy 2, knot 1, turnstone 1 | Does not fit one job. Cargo description already lists five (session manifests, graph and view sidecars, worker declarations, graph-reference adapters, application settings); lib doc says "graphshell session runtime". See the Pandect section: 3 jobs, ~44 modules. | Split three ways (P4): reservoir to `catchment`, lifecycle stays `pandect`, third group to owners. Engine-side edges to cut: `tabard`, `hest`, `curation`, `sceno`. | none (spends `catchment`, ruled D9) |
| `mere-framing` | `crates/system/framing` | 257 | 1 (state-witness) / none | Fits: 8-byte magic + u16 version for stored/sent postcard records. One consumer in all six repos (`state-witness`). | Stays beside the engine. Fork F9: fold into `state-witness`. | `mere-framing` (if F9 merges) |
| `state-witness` | `crates/system/state-witness` | 521 | 2 (pictograph, seiche) / isocosm 2 | Fits: FNV-1a labelled witness, first-divergence report, tick trace. Consumers are `seiche` (Seiche repo after P2) and Isocosm, not the engine. | Stays; owner is shared substrate, not the engine (fork F9: Mere, Seiche or Eidetic). | none |
| `mere-facts` | `crates/domain/personal-facts` | 782 | 1 (mere-resident) / cleromancy 2 | Fits: persona facts domain; transforms candidates and validates complete graphs; Djinn owns files and grants. (dir `personal-facts`, package `mere-facts`.) | Stays beside the engine; admission policy stays with the domain owner. | none |
| `mere` | `crates/mere` | 6928 | 5 (graphshell, graphshell-blocks, graphshell-mere, graphshell-web, mere-view) +3 probe / cleromancy 2, turnstone 1 | Facade, to dissolve. Modules: roster 1,414; subgraph 1,783; query_view 1,619; gloss+glossary 820; navigation+trail 792; coupling_edit 208; clip 127; routing 110. | Dissolve (plan): navigation, trail, roster, subgraph to `incipit`; query_view to engine query or `mere-view` (fork F10); gloss, glossary, coupling_edit, routing, clip to the view crates or host adapters. The package name is re-spent on the engine crate. | none (name re-spent) |
| `incipit` | `crates/incipit` | 298 | 6 (gazette-ledger, graphshell-client-tests, mere-fleece, mere-resident, mere-view, +1) / turnstone 1 | Fits as ids only (GraphId, SessionId, ShelfmarkV1). Its doc cites `crawl` and `frisket` as consumers; `crates/crawl` is empty. | Receiver (D8): kernel surface half, navigation/trail/roster, `mere-forme`, `mere-curation`. Default feature stays ids-only (deps: serde, uuid). | none (receiver) |
| `mere-forme` | `crates/forme/forme` | 6689 | 4 (mere, mere-chrome, pictograph, platen) / none | Fits: arrangement authority (what a workbench is; a tree is one projection). Deps: `mere-kernel`, `mere-curation`. 4 dependents. | Merge into `incipit` (plan candidate), behind a feature. Fork F5: into `platen` instead. | `mere-forme` |
| `mere-curation` | `crates/forme/curation` | 585 | 5 (mere-forme, mere-view, pandect, pictograph, scenograph) / knot 1, turnstone 1, woodshed 1 | Fits: `SubgraphSpec` plus native fold record; 82-line lib, one 503-line module. Dep: serde only. | Merge into `incipit` (plan candidate). `scenograph` needs it dependency-light; incipit is too (serde, uuid). | `mere-curation` |
| `platen` | `crates/platen/platen` | 4817 | 4 (graphshell, graphshell-web, mere, mere-view) / knot 1 | Fits: compiles forme arrangements into tile trees, pane geometry, document-scene input. | Stays. | none |
| `uxtree` | `crates/forme/uxtree` | 1626 | 4 (cambium-a11y, graphshell-client-tests, mere, platen) / turnstone 1 | Fits: projects `inker` documents and `platen` workbenches into AccessKit nodes. Deps: inker, nematic. | Fork F6: merge into `platen` (its `accessibility.rs` exists) or `cambium-a11y`, or keep. | `uxtree` (if merged) |
| `mere-cartography` | `crates/canvas/cartography` | 5404 | 3 (mere-overmap, mere-view, pictograph) / turnstone 1 | Doc says contracts only, "strategies live in sibling crates". Actual: contracts plus six `LayoutStrategy` adapters (adapters/ 2,589; spiral_score 465). Consumers: `pictograph` (23 files), `mere-overmap` (3), `mere-view` (2). | Plan: merge into `pictograph` (frees one). Fork F1: merge with the canvas half instead. | `mere-cartography` or none (F1) |
| `pictograph` | `crates/canvas/pictograph` | 68184 | 3 (graphshell-host, mere, mere-view) / isocosm 1, knot 1, woodshed 1 | Faces fit (IconVG from content address; ~0.4k lib, vello 480). 63.8k of 68.2k lines (93%) is the graph canvas behind feature `canvas` ("graph truth through seiche physics, camera, node pool"); signals 3.1k behind `signals`. | Split at feature `canvas`: faces stay; canvas goes to a host crate (name: fork F1). | see F1 |
| `meristem` | `crates/cambium/meristem` | 3411 | 3 (cambium, cambium-desktop, cambium-rootstock) / cleromancy 1 | Fits: renderer-independent reactive diff and message core; deps tracing, kurbo, anymore. | Stays. | none |
| `cambium` | `crates/cambium/cambium` | 49550 | 20 (cambium-a11y, cambium-desktop, cambium-rootstock, cambium-web, cambium-winit, +15) +3 probe / cleromancy 1, isocosm 7, knot 3, turnstone 1, woodshed 9 | Toolkit fits; also holds graph-aware widgets: frisket (998), graph_canvas 428, atlas 592, forme_swatch 277, fold_projection 408, reading_find/toolbar, setting_row* (managed, session: ~2.2k with tests). Deps include `scenograph`, `scenomise`. | Stays. Receives `persona-picker`. `backstep` cannot merge here (cycle, F4). Graph-aware widgets are a later split, not a P0 merge. | none |
| `cambium-rootstock` | `crates/cambium/cambium-rootstock` | 31452 | 11 (cambium-a11y, cambium-desktop, cambium-web, graphshell, graphshell-blocks, +6) / isocosm 5, knot 1, turnstone 1, woodshed 3 | Fits: input, routing, layout, paint, a11y with no event source; feature `embedding` (egui, iced). | Stays. | none |
| `cambium-winit` | `crates/cambium/cambium-winit` | 288 | 3 (cambium-a11y, cambium-desktop, cambium-rootstock) / turnstone 1 | Fits; its doc says it is thin on purpose so a registry consumer can publish it. rootstock depends on it; desktop depends on rootstock. | Refuse merge (doc reason; into desktop is a cycle, into rootstock adds winit to the event-source-free stock). | none |
| `cambium-a11y` | `crates/cambium/cambium-winit-a11y` | 2178 | 2 (cambium-desktop, graphshell) +1 probe / none | Fits; split out of cambium-winit 2026-07-26 for publishability (its doc). | Stays. | none |
| `cambium-desktop` | `crates/cambium/cambium-genet-winit-host` | 5160 | 10 (cambium-web, dramatis-app, graphshell, graphshell-blocks, graphshell-mere, +5) / cleromancy 1, isocosm 5, knot 1, turnstone 1, woodshed 6 | Fits: single-root winit host over rootstock. | Stays. | none |
| `cambium-web` | `crates/cambium/cambium-genet-web-host` | 5151 | 2 (graphshell-web, tabard-workshop) / woodshed 2 | Fits: browser event source over rootstock. | Stays. | none |
| `workbench` | `crates/cambium/workbench` | 1856† | 12 (app-host, cambium, cambium-desktop, graphshell, graphshell-web, +7) / isocosm 1, knot 1, turnstone 1, woodshed 2 | Fits: TileTree split-and-tab reducer, floats, presentation. No deps in tree. Odd consumers: `mere-surface-api`, `settings-wit`. | Stays. | none |
| `hest` | `graphshell.git checkout 40e8c8d, crates/hest` | 1118* | 9 (cambium, graphshell, graphshell-mere, mere-kernel, mere-overmap, +4) / none | Fits: command ids, menus, palette sessions (declarations; hosts decide). Not in the Mere tree. Engine-side `mere-kernel` and `pandect` depend on it. | Stays (host); cut the engine-side edges. | none |
| `backstep` | `crates/cambium/backstep` | 270 | 4 (cambium, platen, scenograph, scenomise) +4 probe / knot 1 | Fits: snapshot undo/redo with keyed coalescing. Consumers: cambium, platen, scenograph, scenomise (+Knot). | Plan says merge into the Cambium crate it serves: that is a cycle (cambium depends on scenograph and scenomise, which depend on backstep). Fork F4: into `meristem`, or keep. | `backstep` (if F4 merges) |
| `mandrake` | `crates/cambium/mandrake` | 6154 | 8 (cambium-desktop, cambium-web, dramatis-app, graphshell-mere, graphshell-web, +3) / cleromancy 1, isocosm 4, knot 1, turnstone 1, woodshed 4 | Fits: scenario lane for a Cambium document host (frames, captures, receipts). | Stays. | none |
| `sprigging` | `crates/cambium/sprigging` | 3031 | 11 (cambium, cambium-a11y, cambium-desktop, cambium-rootstock, cambium-web, +6) / cleromancy 1, isocosm 4, knot 1, turnstone 1, woodshed 5 | Fits: retained custom-paint leaves and arrangement helpers. | Stays. | none |
| `tinct` | `crates/cambium/tinct` | 1476 | 6 (cambium, mere-registry, pelt-desktop, pictograph, tabard, +1) / cleromancy 1, isocosm 1, knot 1, woodshed 4 | Fits: OKLCH seed-to-palette plus syntax palette. | Stays. Fork F7: merge into `tabard` (its doc starts "Tinct seeds in"). | `tinct` (if F7 merges) |
| `persona-picker` | `crates/cambium/persona-picker` | 256 | 1 (dramatis-app) / woodshed 2 | Fits: one Cambium roster view for every app. Dep: `cambium` only. | Merge into `cambium` (plan): no cycle; all three consumers (dramatis-app, woodshed-genet, woodshed-views) already take cambium. | `persona-picker` |
| `mere-view` | `crates/cambium/mere-view` | 6132 | 4 (graphshell-mere, graphshell-web, mere-overmap, mere-resident) / cleromancy 1, knot 2 | Fits: mere sessions and graph as one Cambium component. | Stays; candidate receiver for `mere-overmap` (F3). | none |
| `mere-overmap` | `crates/cambium/mere-overmap` | 4148 | 4 (graphshell, graphshell-mere, graphshell-web, mere-resident) / cleromancy 1, knot 1 | Fits: remote mere route (wire vocabulary, materialization) filling the mere view. Feature `view` is default. | Fork F3: merge into `mere-view` (every external consumer, cleromancy and Knot, also takes mere-view) or keep (the Graphshell port serves the wire without the view). | `mere-overmap` (if F3 merges) |
| `scenograph` | `crates/cambium/scenes/scenograph` | 5541 | 9 (cambium, graphshell, graphshell-client-tests, graphshell-web, mere-resident, +4) +4 probe / cleromancy 2, knot 3, turnstone 1, woodshed 2 | Fits: host-neutral projection authoring model, no runtime or widget deps. | Stays. | none |
| `scenomise` | `crates/cambium/scenes/scenomise` | 15441 | 7 (cambium, gazette-ledger, graphshell, graphshell-client-tests, graphshell-web, +2) / cleromancy 1, isocosm 4, knot 4, turnstone 1, woodshed 4 | Fits: placement solvers realizing scores into arranged scenes. | Stays. | none |
| `tabard` | `crates/system/tabard` | 6112 | 9 (cambium-web, graphshell, graphshell-web, mere, mere-lanes, +4) / cleromancy 1, knot 1, turnstone 1, woodshed 5 | Fits: Tinct seeds to DTCG tokens and Livery CSS; includes smolweb palettes (239) and reading (346). Engine-side `pandect` and `mere-registry` depend on it. | Stays; cut engine-side edges (pandect `application_settings_store`). | none |
| `mere-chrome` | `crates/shell/chrome` | 2709 | 1 (ux-events) / turnstone 1 | Fits: toolbar, app menu, omnibar, frame view-models. No in-tree consumer except `ux-events`; Turnstone depends on it. | Stays; candidate receiver for `ux-events` (F8). | none |
| `mere-registry` | `crates/system/registry` | 11443 | 2 (document-host, mere-resident) / turnstone 1 | Does not fit: description lists "themes" (moved to tabard 2026-09-24). Eight registries, three consumed. See Registry section. | Dissolve to owners (plan candidate confirmed; destinations differ per registry). | `mere-registry` |
| `mere-surface-api` | `crates/system/surface-api` | 944† | 4 (cambium, cambium-a11y, pelt-desktop, settings-wit) / knot 1, turnstone 1, woodshed 3 | Fits: application host contracts above Genet (settings projection, surface admission). Cargo says "homed here until the platform boundary plan moves it". | Stays; receiver for `settings-wit`. | none |
| `settings-wit` | `crates/system/settings-wit` | 1448 | 2 (app-host, settings-guest) / none | Fits: revisioned settings over `mere:script` WIT (gated server, remote client). Consumers: `app-host`, `settings-guest` only. | Merge into `mere-surface-api` behind a `wit` feature (it implements that crate's `RevisionedSettingsProvider`). | `settings-wit` |
| `ux-events` | `crates/system/ux-events` | 2113 | 0 / none | Fits its description (chrome-event taxonomy). Zero dependents in all six repos; depends on `mere-chrome` and `mere-kernel`. | Fork F8: merge into `mere-chrome`, or retire. | `ux-events` |
| `mere-fetch` | `crates/system/fetch` | 6821 | 2 (graphshell-mere, mere-fleece) / cleromancy 1, knot 1, turnstone 1, woodshed 3 | Mixed. "Host-neutral fetch actor and session-cookie runtime" fits; two Eidetic blob adapters and the crawler request do not sit with the service. See Fetch section. | Stays as the host fetch service; `https_blob`, `iroh_blob` move out; the loader trait is new in the engine. | none |
| `graphshell-host` | `crates/graphshell/graphshell-host` | 12512 | 3 (graphshell, graphshell-web, mere-resident) +1 probe / cleromancy 1, isocosm 4, knot 1, turnstone 1 | Fits: admission, session loop, endpoint catalog, projection host, WebRTC carrier. Hosts serve; the engine does not. | Stays. | none |
| `graphshell-mcp` | `crates/graphshell/graphshell-mcp` | 948 | 1 (mere-resident) / none | Fits: stdio MCP server over an admitted route; decides nothing. | Stays. | none |
| `djinn-testkit` | `crates/system/djinn-testkit` | 3190 | 1 (mere-resident) / none | Fits its own job (harness for djinn residents) but that job is Djinn's. Sole consumer: `mere-resident`. | Fork F11: move to the Djinn repo, or stay. | `djinn-testkit` (if moved) |
| `script-wire` | `graphshell.git checkout 40e8c8d, crates/script-wire` | 577* | 2 (app-host, settings-wit) / none | Fits: wRPC value encoding and blocking transports. Not in the Mere tree. | Stays (host); distinct from the build-time generator. | none |
| `script-witgen` | `graphshell.git checkout 40e8c8d, crates/script-witgen` | 535* | 1 (settings-wit) / none | Fits: build-dependency generator of native WIT bindings. Not in the Mere tree. | Stays. | none |
| `app-host` | `crates/script/app-host` | 483 | 1 (mere-moot) / turnstone 1 | Fits: `app-core` component host, one envelope action. | Fork F12: one `script-host` crate for the three wasmtime hosts. | `document-host`, `block-host` (if F12) |
| `document-host` | `crates/script/document-host` | 1331 | 0 / none | Fits: DocumentScript host over a live ScriptedDom. Zero dependents anywhere; sole consumer of registry `mod-loader`. | Receives `mod-loader`; part of F12. | see F12 |
| `graphshell` | `ports/graphshell` | 104754 | 4 (graphshell-blocks, graphshell-mere, graphshell-web, mere-resident) +3 probe / cleromancy 1 | Doc: "presentation host and loopback acceptance view". Actual: 113 top-level modules; glue for other homes: alembic_workshop 1,609 + mere_host_alembic 519 + distillery_w1 577 (Distillery); moot_resources 905 (Moot); comms_open 117; personal_sync 4,695; coop 2,035; transfer 2,515; capture 2,557. | Stays as the host port. Split at adapters later: Distillery glue to the Distillery port, Moot glue to `mere-moot`. Not a P0 merge. | none |
| `mere-resident` | `ports/mere-resident` | 45018 | 0 / cleromancy 1 | Fits as "the concrete assembly" (resident_mere 7,181, personal_sync 1,344, moot_site 1,179, resident_inference 1,185, resident_distillery 1,519, micron_site 467). The resident split has its own plan. | Stays (plan). | none |
| `pelt` | `ports/pelt` | 1862† | 0 / none | Fits: reference browser port over Genet and Cambium (lib+main+viewer). Companions: `pelt-core` 6,378, `pelt-desktop` 20,329. | Stays. | none |
| `pelt-core` | `ports/pelt/core` | 6378 | 3 (graphshell, pelt, pelt-desktop) / turnstone 1 | Fits: window-neutral retained-session controller. | Stays. | none |
| `pelt-desktop` | `ports/pelt/desktop` | 20329† | 1 (pelt) / none | Fits: desktop shell and smoke harnesses for Pelt. | Stays. | none |
| `dramatis-app` | `ports/dramatis` | 769 | 0 / none | Fits: persona selection and quarantine review through Djinn (bin `dramatis-ui`). | Stays. | none |
| `gazette-ledger` | `ports/gazette-ledger` | 477 | 0 / none | Fits: contacts projected into a scene over chirograph, incipit, sceno. Zero dependents anywhere, not even the Graphshell port. | Fork F13: keep as a host leaf, or fold into the `graphshell` port. | `gazette-ledger` (if folded) |
| `mere-moot` | `ports/moot` | 5776 | 2 (graphshell, graphshell-web) +3 probe / turnstone 1 | Fits: Gemot community surface (murmurs, moots, coop). Feature-heavy; `gemot` is an optional dep. | Fork F14: move to the Gemot repo (it is Gemot's surface) or stay in the host. | `mere-moot` (if moved) |
| `mere-athanor` | `ports/distillery/athanor` | 5243 | 1 (graphshell) / turnstone 1 | Fits: authority half of the Alembic workshop; furnace graph passes already moved to `pandect::furnace` 2026-10-31. | Part of the Distillery pair (F15). | see F15 |
| `mere-alembic` | `crates/distillery/alembic` | 747 | 2 (graphshell, mere-athanor) / turnstone 1 | Fits: recall and workshop component of Distillery (its own description). Consumers: graphshell, athanor, Turnstone. No local Distillery checkout under `repos/`. | Plan: move to Distillery. Fork F15: first merge with `mere-athanor`. | `mere-alembic` |
| `mere-comms` | `.claude/worktrees/resident-port-821cb7/crates/shell/comms (only copy; gone from main)` | 2308* | 2 (graphshell, mere-moot) / none | Already moved: cycle-breaks P6 done 2026-11-06. Main has no `crates/shell/comms`; Mere pins `mere-comms` from `murm.git`. The local Murm checkout (1aebf9c, 2026-11-06 01:52) does not contain it either. | Nothing to move. Strike from the plan list; confirm the Murm remote carries it. | none |
| `chirograph` | `graphshell.git checkout 40e8c8d, crates/chirograph` | 4271* | 11 (gazette-ledger, graphshell, graphshell-client-tests, graphshell-door, graphshell-host, +6) +1 probe / cleromancy 2, isocosm 8, knot 1, turnstone 1 | Fits: carrier-neutral projection wire vocabulary. In graphshell.git; widely consumed (see Dependents). | Already in its home (D9). Nothing to do. | none |

## Table 2: in the Mere workspace but not named in Crate dispositions

The plan counts 62 non-probe crates; these were not placed. Same columns, same method.

| Package | Path | Lines | Dependents | What still fits | Disposition | Name freed |
|---|---|---|---|---|---|---|
| `mere-edit` | `crates/system/source-edit` | 383 | 0 / isocosm 1, knot 1 | Fits: revision-bound UTF-8 edit proposals; hosts keep the buffer. Not named in the plan. | Fork F16: Graphshell host contract (beside `mere-surface-api`) or Inker (Knot's edit path). | none |
| `graphshell-door` | `crates/graphshell/graphshell-door` | 1510 | 5 (graphshell, graphshell-mcp, graphshell-mere, mere-overmap, mere-resident) +3 probe / none | Fits: application door client, identity profile, projection provider; the half of the old endpoint that needs custody. Not named in the plan. | Stays host (cycle-breaks D9). | none |
| `block-host` | `crates/script/block-host` | 273 | 1 (graphshell-blocks) / none | Fits: `block-guest` component host. Not named. | Part of F12. | see F12 |
| `graphshell-blocks` | `ports/graphshell/blocks` | 581 | 0 / none | Block pane port (581). Not named; zero dependents. | Stays with the `graphshell` port. | none |
| `graphshell-mere` | `ports/graphshell/mere` | 4836 | 0 / none | Native mere viewer over the admitted route (4,836); zero dependents. Not named. | Stays with the `graphshell` port. | none |
| `tabard-workshop` | `ports/tabard/workshop` | 4421 | 2 (pelt-desktop, tabard-desktop) / cleromancy 1, knot 1, turnstone 1, woodshed 4 | Theme authoring surface (4,421). Not named. | Stays (port). | none |
| `tabard-desktop` | `ports/tabard/desktop` | 1029 | 0 / none | Standalone Tabard product (1,029). Not named. | Stays (port). | none |
| `graphshell-client-tests` | `crates/graphshell/graphshell-client-tests` | 9 | 0 / none | Test crate (9 lines of src). Not named. | Stays. | none |

## `mere-registry` (`crates/system/registry`, 11,443 lines; 8,977 non-test)

Eight registries behind eight features (`default` enables all but `observations`; the root workspace declares it
`default-features = false`). The Cargo description still says "and themes"; the theme registry left for tabard on
2026-09-24 and the viewer registry retired into Inker on 2026-10-27. Rule applied: a registry may carry several
owners' contracts but should make none of their decisions. **Consumers in all six repos: `mod-loader`
(document-host), `observations` and `diagnostics` (mere-resident; Turnstone enables `diagnostics`). Nothing
enables `input`, `knowledge`, `layout`, `lens` or `protocol`; a search for their public items finds only the
crate's own tests and two doc comments (`graph-kernel/src/pane.rs`, `tabard/src/theme/registry.rs`).**

| Registry (feature) | Lines (non-test) | Holds | Live consumer | Owner candidate | Makes the owner's decisions? |
|---|---|---|---|---|---|
| diagnostics | 3,520 | Event ingress, emit queue, correlation, invariants; 254 channel-name constants (`registry.*`, `mod.nostrcore`, `verse.sync`, `history.*`, `startup.*`) and a descriptor catalog (phase3a+3b, 872 lines) fixing `severity` per channel | Turnstone `reactive_trace`; mere-resident events | Pipeline: Graphshell host (both consumers are hosts), or the engine for stage diagnostics (F19). Channel names: each emitter | Yes. The catalog assigns severity and invariants to channels of other owners, several for retired subsystems (nostrcore, verse, viewer) |
| input | 963 | Key/mouse/pad bindings keyed by `hest::catalogue` ids, defaults (259), remap, conflicts; `from_egui` constructors | none | Graphshell host (hest is Graphshell's) | Yes: default keymap and conflict policy are the host's calls |
| knowledge | 474 | UDC tag validation, hierarchical distance, fuzzy search (nucleo), per-class colour hints; seed JSON under `assets/knowledge` | none | Engine vocabulary pack (UDC as a pack, P6); colour hints to the host | Yes for colour hints (presentation) and scoring; overlaps `mere-kernel` `resource_tags` / `validate_tag_concept` |
| layout | 1,056 | Canvas, workbench-surface and profile-registry primitives; `CanvasTopology/Navigation/Interaction/Style/PerformancePolicy`, conformance vocabulary | none | Canvas crate (F1) and `workbench`/`platen` for `SplitDirection` | Yes: canvas policy structs are canvas decisions |
| lens | 1,117 | Named bundles of physics + layout + theme + filters (`lens:default`); `GraphPhysicsTuning` (physics_config 156, physics 511) | none | Declared-lens plan (engine pack) for the bundle; physics tuning to Seiche; theme to tabard (F18) | Yes: a lens is a decision bundle over five owners |
| mod-loader | 1,399 | Discovery, manifest parsing, dependency resolution, rollback, WASM-runtime seam; contributes `inker::PresentationDeclaration` | `document-host` (sole) | Host script lane: move into `document-host` (F25) | It is the owner's own policy, not a registry of others: move, do not split |
| protocol | 266 | URI scheme to handler-id map and provider hook | none | Host fetch service (`mere-fetch` hard-codes `is_fetchable`: http, https, smolweb) or Inker (F20) | Yes: scheme routing is the fetch owner's decision |
| observations (`observations`, needs `diagnostics`) | 182 | Opt-in staging-to-history bridge over `apparatus-core`; says products own installation, scheduling, redaction | `mere-resident` (sole) | Its sole consumer, or the apparatus owner | No (it says so) |

Disposition (fork F17 covers the dead five): dissolve the crate. Live: diagnostics to the host, observations to
`mere-resident`, mod-loader to `document-host`. Dead: input, knowledge, layout, lens, protocol (3,876 lines) retire
now, or move to their owners as stubs. **Frees `mere-registry`.** Nothing else in the plan needs the name.

## `mere-fetch` (`crates/system/fetch`, 6,821 lines, 3,027 of them tests; lib name `fetch`)

Features (defaults marked): `actor`* (armillary, tokio macros), `smolweb`* (errand), `persona-cookies`* (eidetic,
pandect, serde, serde_json, pollster), `flip`* (inker), `https-blobs` (eidetic, ureq), `iroh-blobs` (eidetic,
transport). Always-on deps: genet-host-api, `mere-pageload`, netfetcher (WHATWG Fetch), tokio, url, zeroize,
rustls-pki-types, async-trait, futures-util, tracing. Dependents: graphshell-mere, mere-fleece; cleromancy
(no default features), Knot, Turnstone, Woodshed (redshank-cache, -desktop, -playback). **Nothing enables
`https-blobs` or `iroh-blobs`.**

| Part (lines) | Job | Owner |
|---|---|---|
| actor.rs 561 + executor 110, lib.rs 578 (FetchCommand/Update, content state, TOFU install), page.rs 220, policy.rs 80 (`ClientFetchPolicy` budgets), submission.rs 155 + smolweb_response.rs 186 (Titan/Spartan explicit writes) | Off-thread page and subresource acquisition, scheme routing, budgets | Host fetch service (stays) |
| cookies.rs 101, cookies_persist.rs 212 (persona-keyed, via `pandect` store), cookies_flip.rs 46 (cookies as `inker::flip` records) | Session jar and Inker engine-flip state | Host fetch service (stays); the persist edge needs the reservoir (`catchment`), not Pandect's lifecycle |
| handle.rs 589 + response 44: `Fetch` trait (`read_range`, `read_all(url, accept, limit)`, `read_into`), `Stores`, `NetFetch`, `Resources` | Byte and range reads for media and feeds (Woodshed) | Host fetch service; the one live `accept` caller is `graphshell-mere` with `text/html` |
| crawler.rs 418 | One admitted GET, streaming byte cap; redirects and HTTP failures returned as observed statuses; no cookies, cache or retry | Host implements it; its contract is the shape of the engine's loader interface |
| https_blob.rs 224, iroh_blob.rs 270 | `eidetic::BlobFetcher` adapters | Neither: Eidetic or the reservoir service; unused (retire or move to `catchment`) |

**RDF resolution policy.** There is none in `mere-fetch` or `mere-pageload` today: a search for turtle, rdf, ld+json,
sparql and Accept composition finds only the caller-supplied `accept` argument. Media type to codec lives in
`mere-rdf` ingest (`application/ld+json`) and `mere-fleece` `structured_data`. What the engine must own, new work in
`mere`: the loader trait (IRI to bytes, media type, retrieval provenance), the Accept list derived from the codec
set, redirect and 303 admission (today `crawler.rs` already returns redirects to the caller), and the retrieval-provenance
record. The host implements the trait with `crawler.rs`-style requests. Nothing needs extracting from `mere-fetch`
beyond the crawler's contract; the split is a new interface plus `crawler.rs` as its first implementer.

## `pandect` (`crates/system/pandect`, 27,097 lines, ~44 modules)

Split by lines: job 1 reservoir ~1.7k (6%); job 2 revision lifecycle ~16.4k (61%, 3.2k of it tests); job 3 ~8.7k
(32%); lib 254. **No replication code exists in Pandect** (no match for `replicat` in `src`); replication lives in
`mere-resident` `personal_sync` (1,344). Sealing is not in `reservoir.rs`: it is `PayloadSealer` use in
`graph_codicil` and the custody hook in `retained_text`. Mention counts are `pandect::<item>` uses across all six repos.

**Job 1: reservoir service (to `catchment`)**

| Module | Lines | Role |
|---|---|---|
| reservoir | 701 | Persona/domain catalog (`DomainId`, `MereId`, `MereRecord`, index slot), access (`MereApplicationAccess`), store lifetimes (`open_*_backend`, `ReservoirStore`) |
| slot_guard | 170 | Compare-and-replace over a muniment slot (5 mentions) |
| shared_root | 186 | Family-shared root and identity adoption (8) |
| retained_text | 353 | Reservoir-owned page text, custody hook (3) |
| image_store | 322 | Content-addressed node imagery, graph-aware retention (9). Fork F23: reservoir store or host cache |

**Job 2: graph-domain revision lifecycle (stays `pandect`)**

| Module | Lines | Role |
|---|---|---|
| graph_session | 5,593 | Session over one muniment store: baseline + journal, checkpoint, apply/undo/redo, `graph_at`, fork/branch begin; `MereSessions` mint/switch/trash. Also carries the view stream (`ViewKey`, `set_view_now`, `view_at`): fork F24 |
| graph_session_merge, snapshot_merge, merge_rules | 880, 1,311, 293 | Merge-in, codicil compose, shared rules |
| graph_session_transaction, graph_session_operation | 449, 243 | Stage and install with atomic acknowledgement; owner operation record |
| live_graph | 367 | Admission of observed live edits (11 mentions) |
| graph_codicil | 1,811 | Freeze/thaw to an Eidetic codicil; sealed save/load |
| addressable_graph | 1,158 | Recorded materializations by surface or resource address; residency |
| graph_placement (+ legacy receipt compat 240), session_graph_store | 789, 47 | Placement profile and effect cursor for replay; `graph.json` name (32 mentions) |
| tests (graph_session_*, merge, codicil P6, resource content) | ~3,208 | Travel with their modules |

**Job 3: sessions, manifests, sidecars, workers, settings, furnace (owner per module)**

| Module | Lines | Owner candidate |
|---|---|---|
| manifest | 457 | Split at the type: lineage, picked/merged/trash marks to job 2; `SessionPolicy`, `EngineProfileBinding`, `WorkerKind` to the host (fork F21; 25 mentions) |
| manifest_store | 658 | Host; legacy beside the muniment keys, 14 mentions, confirm before moving |
| view_intent_store, live_view | 602, 431 | Host (Graphshell views; streams sit beside the journal at `views/<app>/<view>`); 15 mentions |
| switcher_thumbnail | 558 | Host |
| cartography_geometry, saved_scene, scene_facets | 308, 268, 275 | Canvas crate (F1) |
| arrangement_facets | 784 | `incipit` (arrangement is workspace truth, D8) |
| web_facets, browser_node_state | 237, 359 | Application (Turnstone); 13 mentions |
| denizen_facets | 303 | `mere-moot`/Gemot or retire (legacy namespace; 12 mentions) |
| facet_store | 413 | Mechanism to the reservoir; namespaces to the owners above |
| session_service_runner | 340 | Host (worker orchestration; Djinn side) |
| script_bindings_store | 111 | `document-host` |
| settings_store, persona_settings_store, application_settings_store | 365, 260, 241 | Host, via `mere-surface-api` settings contract; the last imports `tabard::ThemeChoice` |
| notochord_policy_store | 167 | Custody (Dramatis/Notochord) |
| engine_profile_store | 339 | Inker engine selection, via the host |
| furnace, memory_levels | 842, 404 | Distillery (athanor says furnace passes moved here 2026-10-31; feature `furnace`); fork F22 |

## `crates/crawl` and `crates/import`

Both are empty directories (`crawl/src` is an empty folder; `import` has no children), dated 2026-10-11, the day
commit 92f391bfc moved crawl into Fleece and portable intake into Eidetic. Not workspace members; no manifest,
script, `line-ceiling.toml`, `deny.toml` or `.cargo/config.toml` entry references them, in Mere or the five other
repos. Remaining references are historical text: the plan (lines 144, 395), `design_docs/2026-05-24_external_deps_topology_brief.md`
(line 121), eleven archived plans, `support/doc-audit/d2/*`, three Graphshell receipt audit JSONs (`crates/import/src/history.rs`,
`web_clip.rs`), and two Knot archive plans. `incipit`'s lib doc still names `crawl` as a consumer. Safe to remove
(git tracks neither).

## Squish candidates, by names freed per 1,000 lines moved

Lines moved is the crate's size (retire = 0 lines).

1. **No lines moved:** retire the five dead registries (frees `mere-registry` with the three live moves, F17); retire
   `ux-events` if F8 says retire (frees `ux-events`); `mere-comms` is already gone from the tree.
2. `persona-picker` into `cambium`: 1 name / 256 lines = 3.9. No cycle. Confirmed.
3. `mere-framing` into `state-witness` (F9): 1 / 257 = 3.9. Fork.
4. `backstep` into `meristem` (F4): 1 / 270 = 3.7. The plan's target `cambium` is a cycle. Fork.
5. `gazette-ledger` into the `graphshell` port (F13): 1 / 477 = 2.1. Fork.
6. `mere-curation` into `incipit`: 1 / 585 = 1.7. Confirmed.
7. `mere-alembic` into `mere-athanor` before the Distillery move (F15): 1 / 747 = 1.3. Fork.
8. `document-host` + `block-host` into `app-host` (F12): 2 / 1,604 = 1.2. Fork (not in the plan's list).
9. `settings-wit` into `mere-surface-api` (feature `wit`): 1 / 1,448 = 0.7. Not in the plan's list; no external dependents.
10. `tinct` into `tabard` (F7): 1 / 1,476 = 0.7. Fork.
11. `uxtree` into `platen` (F6): 1 / 1,626 = 0.6. Fork.
12. `djinn-testkit` to the Djinn repo (F11): 1 / 3,190 = 0.3. Fork.
13. `mere-overmap` into `mere-view` (F3): 1 / 4,148 = 0.2. Fork.
14. `mere-cartography` into `pictograph` (plan) or the canvas (F1): 1 / 5,404 = 0.2.
15. `mere-moot` to Gemot (F14): 1 / 5,776 = 0.2. Fork.
16. `mere-forme` into `incipit` (plan; F5): 1 / 6,689 = 0.15.

Plan expected six freed names (forme, curation, cartography, backstep, persona-picker, registry). Audit result:
confirmed curation, persona-picker, registry; backstep fails as written (cycle); forme and cartography are forks.
Candidates the plan lacks: settings-wit, ux-events, the script hosts, gazette-ledger, framing, tinct, uxtree.
The only distinct purpose here that needs a name is the graph canvas after F1(b), which takes `cartography`; the
reservoir (`catchment`) and engine (`mere`) names are already ruled.

## Forks (two defensible answers)

- **F1. Pictograph, cartography and the canvas.** (a) Plan: merge `mere-cartography` into `pictograph`; frees one
  name, but pictograph stays 93% canvas (63.8k of 68.2k lines) under a face-deriver's name. (b) Split `pictograph` at feature
  `canvas`; the canvas half plus cartography's contracts and six adapters become one host crate named `cartography`;
  pictograph shrinks to faces (~1.3k lines + signals 3.1k). Frees none, fixes the drift, and the distinct purpose takes a
  freed name. Evidence: cartography has 23 `pictograph` files vs 3 `mere-overmap` and 2 `mere-view`; pictograph already
  contains `canvas/cartography.rs` and `cartography_scene.rs`; `canvas` is off by default.
- **F2. Where `mere-rdf/statements.rs` goes.** (a) Inker, which P1 lets depend on Mere's dataset surface, so the engine
  drops `inker`. (b) The host's legacy snapshot adapter (P5 sends the old projection there). Evidence: the file says
  it was relocated into this crate 2026-07-10 "so inker stays kernel-free"; P1/Constraints now allow the opposite edge.
- **F3. `mere-overmap` into `mere-view`.** Merge: Cleromancy and Knot, the only external consumers, take both; the
  `resident` build already enables `view`. Keep: the `graphshell` port uses the wire vocabulary without Cambium, and
  `view` is a feature today (4,148 lines; wire 588, route 517).
- **F4. `backstep`.** `cambium` is impossible (cambium depends on scenograph and scenomise, which depend on backstep).
  Into `meristem` (3.4k, tracing/kurbo only; frees one) vs keep (270 lines; platen, scenograph, scenomise use it
  without the toolkit; undo history is state, not diff).
- **F5. `mere-forme`.** Into `incipit` (plan; workspace truth, D8) vs into `platen` (compiles forme arrangements; its
  4 dependents are `mere`, `mere-chrome`, `pictograph`, `platen`). `incipit` must keep a ids-only default (serde, uuid);
  forme needs the kernel `Graph`'s workspace half, so it works only behind a feature.
- **F6. `uxtree`.** Into `platen` (`accessibility.rs` exists; platen already depends on it), into `cambium-a11y`
  (AccessKit host; merging forces platen onto the winit stack, so unlikely), or keep (a11y tree is its own artifact; 4 dependents).
- **F7. `tinct` into `tabard`.** tabard's doc starts "Tinct seeds in" and depends on nothing else; but `isomere` and
  `redshank-surfaces` use tinct without tabard, and cambium takes tinct for highlighting (`cambium -> tabard`
  adds an edge).
- **F8. `ux-events`.** Zero dependents in six repos and no Rust use of the crate name. Retire, or merge into `mere-chrome`
  (its upstream). Mark's "design for known futures" argues keeping it; the lack of any consumer argues retiring.
- **F9. `mere-framing` and `state-witness`.** One consumer for framing (state-witness) in all repos; the data-formats
  brief makes the header the universal rule. Merge frees a name; keep protects the doctrine's anchor. Owner: Mere,
  Seiche (its consumers `seiche` and Isocosm) or Eidetic.
- **F10. `mere/query_view` (1,619 lines).** Engine query module (it is "resource-owned query definitions and read-only
  projections of held strata") or `mere-view` (feature `query-editor` already imports it).
- **F11. `djinn-testkit`.** Its job is Djinn's (3,190 lines, one consumer, `mere-resident`); the plan keeps it with the host.
- **F12. Script hosts.** `app-host` (483), `document-host` (1,331), `block-host` (273) are wasmtime hosts over one WIT
  package, same engine version by comment; one `script-host` with features frees two names. Against: each has distinct
  consumers (Turnstone and Moot; none; graphshell-blocks) and distinct guest worlds.
- **F13. `gazette-ledger`.** 477 lines, zero dependents, not even the port. Keep as a host leaf (its doc explains why
  it left dramatis) or fold into `graphshell`.
- **F14. `mere-moot`.** Host port vs Gemot repo (its `gemot` dep is optional; Turnstone consumes it).
- **F15. `mere-alembic` and `mere-athanor`.** Move alembic to Distillery as planned, or merge it with athanor first
  (its own doc calls them a pair). Note there is no local Distillery checkout under `repos/`; only `distillery.git` is pinned for `esp`.
- **F16. `mere-edit` (383, unnamed).** Graphshell contract leaf vs Inker's edit path; consumers Knot and Isocosm only.
- **F17. Dead registries (input, knowledge, layout, lens, protocol; 3,876 lines).** Retire now (no consumer in six repos)
  vs move to owners as they arrive (design for known futures; lens and knowledge await the declared-lens and pack plans).
- **F18. Lens.** Engine pack (declared-lens plan) vs host presentation profile; its physics tuning belongs to Seiche either way.
- **F19. Diagnostics pipeline owner.** Host (both live consumers are hosts) vs engine (stage diagnostics, P6 SHACL
  results). The 254-name catalog is not a fork: it goes to emitters or retires.
- **F20. Protocol registry.** `mere-fetch` (it hard-codes scheme routing today) vs Inker (smolweb protocol family, D7).
- **F21. `pandect::manifest`.** Split lineage/marks (job 2) from policy/binding/worker kinds (host), or move whole to the host.
- **F22. `furnace` and `memory_levels`.** Distillery (its docs say so) vs reservoir (image GC and forgetting are store lifetimes).
- **F23. `image_store`.** Reservoir store (content-addressed, retention) vs host cache (graph-aware retention decisions).
- **F24. View stream inside `graph_session`.** `ViewKey`/`set_view_now`/`view_at` live in the lifecycle module; leave them
  (Alembic decision #5 keeps view streams beside the journal) or lift to the host's view sidecar crate.
- **F25. `mod-loader`.** Into `document-host` (sole consumer, 1,399 lines) vs a host `mods` crate for future consumers.

## 2026-11-07, P2 (Seiche) worktree proof

Lane branch `worktree-agent-a3b1be946717468ea`, base `78917b4fa`. No crate source or Mere
manifest changed. Windows 11, `cargo 1.99.1`, offline, `-j 4`, debug info and incremental off,
one build directory (the worktree's `target/`).

- **Standalone workspace.** A scratch workspace in the session temp directory listing only
  `conatus`, `modulus`, `nisus`, `numen`, `seiche` and `tenant` as path members, resolver 2,
  restating the inherited keys (licence, lints, `nisus` and `wgpu` workspace rows, the physics
  dev opt-level table), seeded with Mere's `Cargo.lock`. 847 packages resolve; every one has
  Mere's version and source except `cubek-matmul` 0.3.1, which Mere patches to
  `support/patches/cubek-matmul` and the standalone takes from the registry (reached only
  through the Burn features).
- `cargo check --workspace --all-targets`: pass, 0 warnings, 1 min 40 s.
  `--all-features`: pass, 2 min 03 s, 21 warnings, all the workspace `unsafe_code` lint on
  conatus's CubeCL launches, as in Mere.
- `cargo test --workspace` (default features): 372 passed, 0 failed, 11 ignored. conatus 44
  (19 lib, voxel 1, contacts 7, reads 11, refresh 6); modulus 26 (18 lib with 1 ignored, trace
  8); nisus 6; numen 65; seiche 224 (215 lib with 10 ignored, contact_to_fact 5,
  gpu_repulsion 0 without `gpu`, tactile 4); tenant 7.
- `cargo test -p seiche --features serde --lib`: 222 passed, 10 ignored. The binding replay
  receipt recorded 219 and 10 on 2026-10-20; seiche `src` since then adds 26 `#[test]` lines
  and removes 23 (mostly the 2026-11-04 file split), net 3.
- **Edges left in the standalone graph** (`cargo metadata`): `seiche` dev to `state-witness`
  (Mere, path), which brings `mere-framing` (Mere, path); `seiche` to `armillary` (family git,
  `branch = "main"`, optional, default feature `actor`); `tenant` to `kiss3d` (mark-ik fork,
  `rev = c362cd58`). Inside the set: `conatus` dev to `seiche`, `seiche` optional to
  `conatus`, `seiche` to `numen`, `conatus` optional to `nisus`.
- **Binding replay** (`support/seiche-replay`, receipt
  `Code/testing/mere/support/receipts/seiche-binding-replay-20261020`). `binding_replay` built
  from the standalone workspace (`--no-default-features --features serde`, exe SHA256
  `67c18280…`), fed the receipt's exact `binding-input.json` (SHA256 `b2058fd6…`) twice:
  600 ticks, 18,600 labelled witness entries, the two processes identical and both equal,
  entry for entry, to the recorded macOS native trace and the recorded Node Wasm trace;
  stdout byte-identical to the recorded `binding-native-1.stdout.json`. The dt control first
  diverges at tick 7, `position:amber/x-bits` 13859544967838438227 against
  8017892241343393635, state witness `clock:elapsed-f64-bits` 18376027366781248631 against
  5412665435615815106, as recorded; its perturbed trace equals the recorded one. The
  unsupported spec refuses at `spec`, exit 1. The Wasm half was not rerun.
- **Canvas.** In the worktree, `cargo check -p pictograph -p mere-cartography --features
  pictograph/canvas --all-targets --locked`: pass, 2 min 43 s, 105 warnings, all in
  pictograph, graph-kernel, tabard, pandect and scenomise, none in the spatial crates.
- **Isocosm.** Pinned today by `https://github.com/merely-made/mere.git`, `branch = "main"`,
  in Eponym's and Mesocosm's `[workspace.dependencies]` (and Isometer's lens, the parry-ground
  probe and Cosmograph/Isometer core for `state-witness`); both locks at `18c0f24a5`, whose
  six crates, `state-witness` and `mere-framing` are byte-identical to this branch. A scratch
  copy of Isocosm (no `.git`, no targets; the originals untouched) took a
  `[patch."https://github.com/merely-made/mere.git"]` for `conatus`, `modulus`, `nisus` and
  `tenant` pointing at the standalone workspace. Each resolves to one source (the path);
  `state-witness` and `mere-framing` stay on `mere.git` `18c0f24a5`. Eponym
  `cargo check --workspace --all-targets --locked`: pass, 4 min 20 s, warnings only in
  Isocosm's crates. Mesocosm: pass, 4 min 05 s, warnings only in Isocosm's crates. In both,
  the four patched crates compile from the standalone paths.
- **Not rerun (F138).** Pictograph's Density and F73 release receipts (62 figure lines): no
  input to them changed in this lane, Mere's lock and sources being untouched, and they cannot
  run from the standalone set, which holds no canvas.

## 2026-11-07: P1 lane (Inker), worktree branch `worktree-agent-a61f20e98db5486a2`

Base `78917b4fa`. All builds used `CARGO_PROFILE_DEV_DEBUG=0` and this worktree's `target/`
(`CARGO_TARGET_DIR` and `CARGO_BUILD_BUILD_DIR` set for the scratch workspaces).

Mere, touched crates:

- `cargo check --offline -p mere-fleece -p mere-fetch -p scenograph -p script-rhai --features
  mere-fleece/{crawl,clip,eidetic-bridge,portable-clip},mere-fetch/fleece-crawl,scenograph/scripts
  --tests`: green. `Cargo.lock` changed edges only: `mere-fleece` loses `incipit` and
  `mere-fetch` and gains `mere-pageload`; `mere-fetch` gains `mere-fleece`; `scenograph`
  gains `script-rhai`; `script-rhai` loses `scenograph`.
- Tests: `mere-fleece` (crawl, clip, eidetic-bridge, portable-clip) 51 lib + 1 + 6
  integration, the same lib count as before (the session test now runs under `crawl`, the
  single-hop test left); `mere-fetch --features fleece-crawl` `fleece_crawl` 1;
  `scenograph --features scripts --test script_rhai_rules` 9 (moved from script-rhai);
  `script-rhai` 33 lib + 3 `maths_bits`. All pass.
- `cargo check --offline --locked --workspace --all-targets --exclude mere-resident`: green,
  4m 34s. `mere-resident` is excluded because its dependency `knot-composition` (Knot
  `abffd2d`) names `scenomise::backdrops`, which `794a88c77` moved out of `scenomise`. The
  pre-commit clippy hook (`cargo clippy --locked --all-targets`) fails there for the same
  reason, before any P1 change.

Standalone Inker workspace (`%TEMP%/inker-p1-standalone`, generated by copying the twelve
crates at their Mere paths and writing a root manifest whose non-Inker Mere rows become
`mere.git` `branch = "main"` rows; lock seeded from Mere's):

- `cargo metadata --all-features`: Inker-local 12 packages; family sources genet 15,
  errand 9, eidetic 8, netrender 4, murm 3, dramatis 2, armillary 1, `mark-ik/vello` 3;
  `mere.git` 16 and graphshell.git 1, reached only through four edges:
  `knot-editor-host -> cambium`, `mere-lanes -> tabard`, `mere-fleece -> mere-rdf`
  (optional, `structured-data`), `script-rhai -> hest` (graphshell.git).
- File edge: `document-canvas/src/text/tests.rs` includes
  `crates/cambium/examples/genet_web_smoke/assets/Roboto-Regular.ttf`; a copy was placed at
  that path in the scratch workspace only.
- `cargo check --offline --workspace --all-targets --features mere-lanes/{netfetch,smolweb,
  retained-clips},mere-fleece/{crawl,clip,eidetic-bridge,portable-clip,retained-traversal,
  page-links,article},document-canvas/netrender`: green.
- `cargo test` with the same features: 761 passed, 1 failed:
  `mere-lanes` `remote::client_limits_tests::smolweb_queue_wait_uses_the_same_admission_pool_and_deadline`
  (100 ms deadline, run while another build compiled). Rerun alone three times: 3 of 3 pass.
  `welding-0-15` was left off: its build fetches the CEF distribution.

Consumers (scratch copies in `%TEMP%`, their own committed locks, a `[patch]` of the twelve
Inker packages to the standalone workspace, the post-move shape):

- Knot `cargo check --workspace`: green, 16.0s.
- Turnstone `cargo check -p turnstone`: green, 57.9s. `--workspace` fails in
  `cef-dll-sys`'s cmake build (member `sandbox_bootstrap_win` enables `weld`); that build
  script had downloaded the CEF distribution into this worktree's `target/` first.
- Both locks resolve each Inker package, `cambium` and `tabard` from exactly one source.
- Patching the Inker packages to the worktree instead gave two `cambium`s in both (the
  worktree crates resolve `cambium` by Mere path): the F2 edge seen as type identity.
  Patching every `mere.git` package to the worktree fails on Knot's own drift from Mere main
  (`scenomise::backdrops`, `sceno` split), unrelated to P1.

## 2026-11-07: Eidetic squish audit (read-only, Sonnet subagent, no builds)

# Eidetic squish audit (read-only, 2026-11-07)

Method: D10 of Mere's engine split plan. Read: `design_docs/DOC_README.md`, `DOC_POLICY.md` (no doctrine file; rulings live in the crate docs and plans), each crate's `lib.rs` doc and `Cargo.toml`. Rust lines are `cat src/**/*.rs | wc -l` (includes in-src tests; `target` excluded).

Counting caveats:
- "Ext. manifests" counts Cargo.toml files in the eight named repos (not `.claude`, `target`, `.cargo/local`) that name the package. It includes each repo's root workspace manifest, which carries `[patch]` and `[workspace.dependencies]` rows. Mere's counts are inflated by receipt harnesses and probes. Real consumers are named where I checked.
- Workspace members are 10 crates. `crates/responsa` and `crates/line-ceiling` are NOT members (each has its own `Cargo.lock`, no patch row).
- No Eidetic crate is depended on by Armillary (0 manifests for all).

## Crate table

| Package (path) | src lines | Dependents in Eidetic | Ext. manifests (repo:count) | Still fits original job | Disposition of the rest | Name freed |
|---|---|---|---|---|---|---|
| `muniment` (crates/muniment) | 8,044 | chartulary, eidetic, eidetic-search, stickleback, responsa | mere 18, isocosm 7, gemot 7, woodshed 7, knot 5, dramatis 5, cleromancy 2 | Fully: Backend seam, slots, blobs, journals, plus the redb, fjall, directory, zip, IndexedDB and atomic-file backends. All are one persistence floor behind features. | Keep. Absorbs `proofs` (fork F1). The `redb_cross_process_lock` bin is a helper, not a crate. | none |
| `chartulary` (crates/chartulary) | 7,853 | eidetic, servitor | mere 11, woodshed 6, isocosm 4, gemot 3, knot 2, cleromancy 2, dramatis 1 | Fully: generic `Graph<N,E>`, caps, the edit spine (`GraphLog`, commit), `rdf` (ex-scholia), `stemma` (lineage), taxonomy. Two earlier squishes (scholia and stemma as modules) already happened here. | Keep. Possible absorber of `servitor` (fork F3). `assertion.rs` and `canonical.rs` (URL canonicalization) are small and graph-adjacent; leave. | none |
| `eidetic` (crates/eidetic-core) | 13,544 | eidetic-search | mere 14, woodshed 5, isocosm 3, knot 3, gemot 3, dramatis 3, cleromancy 2 | Partly. The doc says "portable intake, typed retained content and memory". The generic layers fit: codicil (235), schema/schema_def/schema_facets (about 1,335), typed, manifest (668), seal, bundle, retention, pack (326). | The rest is Mere-product domain, not substrate: `browsing` 2,513 (BrowsingTrace, frecency), `import` 1,687 (browser bookmark/history parsers), `models` 2,266 (ML model manifests, embedding, inference, training), `records` 619 (assertions, query inputs), `content_store` 181 (URL-keyed page cache), `deleted` 236 (session-graph tombstones), `clip_profile` 138, `image_content` 90. About 7,700 lines (57%). Move to a Mere-side owner (fork F4); no name is freed. | none (a move, not a merge) |
| `eidetic-search` (crates/eidetic-search) | 4,076 | none | mere 4 (fleece, graphshell, moot), gemot 2 | Fits its own doc (in-tree BM25, candidates, fusion, `TrailIndex` from BrowsingTrace). But the only corpus is `eidetic::browsing`; the `DocumentIndex` seam is neutral. Lib name `eidetic_search`, bin `eidetic-recall`. | Merge into `eidetic` as a `search` module behind a feature (fork F2), or move with `browsing` to the Mere-side owner (F4). | `eidetic-search` (the `eidetic-recall` bin name goes too) |
| `stickleback` (crates/stickleback) | 11,572 | none | mere 8, gemot 7, knot 2, isocosm 2, cleromancy 2 | Fully, and it is the C5 home: signed-op store (`MunimentStore`), `OperationPolicy`, `CheckpointAuthority`, causal log, group crypto and lanes, epoch retention, native drops, inventory. Network sessions already left (T7, 2026-11-03). | Keep. Grammar and fold are not yet an interface here (see C5 note). Depends on `proofs`; if F1 merges into muniment the edge is stickleback to muniment, which already exists. | none |
| `mere-proofs` (crates/proofs, lib `proofs`) | 161 | stickleback | mere 8, knot 2, isocosm 2, cleromancy 2, gemot 1 | Fits: typed digests, commitments, `BlobRef`, proof envelopes. But it is one file and it is the content-address vocabulary muniment's `Hash` and `BlobStore` already own. | Merge into `muniment` as a `proofs` module (default). It has no Eidetic deps (blake3, serde, thiserror), so no cycle. | `mere-proofs` (and bare `proofs`) |
| `servitor` (crates/servitor) | 5,993 | none | mere 12, gemot 5, knot 2, cleromancy 2, isocosm 1 | Fits its own doc: capability algebra (`cap`), grants, the validating gate, delegation, watch, cascade, tick. It is the authority gate over chartulary commits; the only Eidetic dep is chartulary (plus insigne and identity). | Keep, or merge into chartulary (F3). Not a module of a neighbour by size; it is 76% of chartulary's size. | `servitor` only if F3 merges |
| `hagiograph` (crates/hagiograph) | 831 | none | isocosm 4 (cosmograph, mesocosm, wing-integration), knot 1 (knot-document), mere 3 (root, probes) | Fits: standing marks, feats, deep time. The doc rescoped it 2026-09-16 and states what it is not. | Candidate to merge with `impresa` (F5). No deps at all. | `hagiograph` or `impresa` |
| `impresa` (crates/impresa) | 979 | responsa | isocosm 1 (cosmograph) | Fits: validated association records (Impresa, KindSet, Stance). Record type and validation only. Its only consumers are cosmograph and responsa. | Merge with `hagiograph` (F5), or keep. | `impresa` or `hagiograph` |
| `luggage` (crates/luggage; lib `luggage-manifest`) | 2,848 | none | mere 7, knot 2, isocosm 2, woodshed 2, cleromancy 2, dramatis 1 | Fits its own doc (self-update, signed manifests, feeds), but it is not store substrate. A fork of cargo-packager-updater with zero Eidetic deps. The README table says "artifact distribution", which is an Eidetic claim. | Keep here, or move to a distribution owner (F6). No name is freed by a move. | none |
| `responsa` (crates/responsa) | 1,999 | none (needs impresa, muniment) | 0 | Fits its doc (cases to doctrine). A doc-tooling CLI, not a library for the stack. Not a workspace member. | Keep until a tools home exists; do not merge with `line-ceiling`. | none |
| `line-ceiling` (crates/line-ceiling) | 451 | none | 0 manifests (isocosm adopts it by tagged binary install) | Fits: one binary, one job. Not a workspace member. | Keep. | none |

Totals: 10 workspace crates plus 2 standalone, about 57,350 lines (eidetic 13,544; muniment 8,044; chartulary 7,853; stickleback 11,572; servitor 5,993; search 4,076; luggage 2,848; responsa 1,999; impresa 979; hagiograph 831; line-ceiling 451; proofs 161).

## Squish candidates (names freed per 1,000 lines moved)

| # | Merge | Lines moved | Names freed | Per 1k | Cycle check |
|---|---|---|---|---|---|
| 1 | `mere-proofs` into `muniment` (module `proofs`) | 161 | 1 (`mere-proofs`, lib `proofs`) | 6.2 | `proofs` depends on blake3, serde, thiserror only. Muniment already has blake3. Stickleback depends on both and keeps its muniment edge. No Eidetic crate gets a new upward edge. Ext. consumers (gemot, isocosm, knot, cleromancy) already take muniment or stickleback in the same trees except possibly gemot's proofs-only manifests; they would gain muniment's default features unless `proofs` is feature-split. Check gemot's `Cargo.toml` row. |
| 2 | `hagiograph` into `impresa` (or reverse) | 831 (or 979) | 1 | 1.2 (or 1.0) | Neither has any Eidetic dependency; responsa depends on impresa only. Both consumers are in isocosm (cosmograph takes both). Knot-document takes hagiograph only and would gain impresa's code (no new deps; both are serde-only). No cycle. |
| 3 | `eidetic-search` into `eidetic` (feature `search`) | 4,076 | 1 (plus the `eidetic-recall` bin and `eidetic_search` lib names) | 0.25 | Search depends on eidetic and muniment[fjall]; merging makes the dependency internal. Eidetic already depends on muniment. The fjall feature becomes an eidetic feature. No cycle. Disposition changes if F4 moves `browsing` out: then search goes with it. |
| 4 | `servitor` into `chartulary` (module `servitor` or `gate`) | 5,993 | 1 | 0.17 | Servitor depends on chartulary, insigne, identity. Chartulary would gain insigne and identity, which Eidetic already holds (eidetic-core and stickleback use identity). Neither upstream (muniment) depends on servitor. No cycle. Cost: chartulary's wasm and feature surface grows. |

Total if all four land: 4 names freed, 11,060 lines moved. Nothing else in Eidetic is a one-module crate of a neighbour. The three crates not in a merge (`luggage`, `responsa`, `line-ceiling`) are single-job and share no code with the rest.

## Forks (more than one defensible answer)

**F1. Where `proofs` goes: muniment, stickleback or stay.**
- Muniment: `Hash` and `BlobStore` already name blake3 content addresses; `BlobRef` and `Digest` are the same vocabulary. Eidetic-search and all Muniment consumers get it. Weighs against: `Digest` and proof envelopes are replication vocabulary, not storage.
- Stickleback: its only in-Eidetic user is `group_crypto.rs` and `store.rs`. Weighs against: gemot, isocosm, cleromancy and knot take `mere-proofs` in manifests where I did not confirm stickleback is also taken; a proofs-only consumer would pull p2panda.
- Evidence: 161 lines, 1 file; `rg proofs::` in stickleback hits `Digest::blake3`, `BlobRef`, `DigestAlg`; dependents 15 manifests outside Eidetic.

**F2. `eidetic-search`: merge into `eidetic`, merge with browsing elsewhere, or keep.**
- Merge: one crate for memory plus its recall; frees a name.
- Keep: search is 4,076 lines and carries its own wasm and OPFS argument (its doc: no mmap, no threads, no C); its `DocumentIndex` seam is neutral over any corpus; separate release cadence. Only 4 Mere manifests and 2 gemot manifests depend on it.
- Evidence: all of search's `eidetic::browsing` imports are in `index.rs`, `fusion.rs`, `candidates.rs`; `records` module uses `eidetic::records`.

**F3. `servitor` into `chartulary` or keep.**
- Merge: its sole Eidetic dependency is chartulary; "gate applies through chartulary's commit" makes it chartulary's authorization layer; frees a name.
- Keep: capability algebra (`cap`, `delegation`, `grant`) is authority policy, not graph data; 5,993 lines; used by 22 manifests, several (gemot, knot) that also take chartulary, but chartulary's own doc says it is app-agnostic and the gate is a different intention. D10 "a crate whose job is one module of a neighbour's" is arguable here: the gate is one layer over commit, but it carries its own algebra.

**F4. Where the Mere-product half of `eidetic-core` goes.**
- Options: stay (rename nothing); move `browsing`, `import`, `models`, `records`, `content_store`, `deleted`, `clip_profile`, `image_content` to Mere's graphshell/fleece side (three of their `use` sites are in graphshell and mere-resident tests); or split at the adapter, leaving schema/codicil/typed/seal/retention/pack in a lean `eidetic`.
- Evidence: external importers of `eidetic::models` are mere-resident tests and graphshell `goal_surface`; of `eidetic::import` and `eidetic::browsing`, graphshell `capture.rs`; of `eidetic::records`, graphshell `goal_surface/suggest.rs`. Eidetic-search is the only other consumer. This is the large mismatch against the `lib.rs` doc, but the doc itself still says "browsing memory, the domain the crate was named for", and the memory gates plan (P1-P4) is built on it. It frees no name and the plan is Mark's.

**F5. `hagiograph` with `impresa`.**
- Merge: both are zero-dependency isocosm-vocabulary "organs" of 831 and 979 lines; `impresa`'s only consumers are cosmograph and responsa; frees a name.
- Keep: history judgement (marks, feats, deep time) and association records (stance, kinds) are different jobs; hagiograph's doc lists boundaries it refuses. Knot-document takes hagiograph only. "Organ" is shared vocabulary, not shared intention.
- Evidence: Cargo.toml for each has only serde (hagiograph) / serde, serde_json, toml (impresa).

**F6. `luggage` stays or leaves Eidetic.**
- Stay: no other home exists; seven-plus repos take it; the Eidetic README claims "artifact distribution".
- Move: it does not store, replicate or gate anything; the Eidetic layering line is "store substrate, domain interface, pack header". A move frees no name; the only gain is a clearer Eidetic.

**F7. `responsa` and `line-ceiling` homes.** Both are non-members with their own locks and are developer tools, not libraries. Options: stay; move to a tools or house-style home (Mere's house-style plan owns stack-wide line-ceiling adoption). No name is freed either way.

## C5 and D10: what Eidetic must own, and does a crate exist

**C5 (Stickleback domain interface).** The contract says grammar, fold and policy are a Stickleback interface so non-graph domains (Knot) depend on Stickleback, not on Mere's graph crates.
- Present: `stickleback::OperationPolicy<E>` (admit: operation to `Admission`/`Reject`, `processor.rs:124`) and `CheckpointAuthority` (`authority.rs:16`). The lib doc says domains supply "grammar, addressing, authorization and deterministic materialization", but only the first two policy-side hooks are trait-shaped.
- Missing: a grammar/decode trait (body to typed edit) and a fold/materialization trait (deterministic apply over the causal log). `causal.rs` mentions "profiles choose their own event grammar and fold" and offers fold helpers (a projection fold around line 347), but no `Domain` trait. The M1 registry "holds Stickleback domain implementations", so it needs one.
- Crate: no new crate needed. It belongs in `stickleback` (a `domain` module). The graph grammar and fold stay in Pandect (C5). A separate "domain" crate would only recreate a name for one module.

**D10 (shared pack header beside `mere.pack/v1`).** Plan layering line puts "the pack header" in Eidetic; the lessons brief says one shared header in Eidetic embedded by each product's manifest.
- Present: `eidetic::pack` (326 lines, `PACK_SCHEMA = "mere.pack/v1"`, `PackManifest` {name, version, author, requested_scopes, parts}, signing binding, `verify_pack`). It is a manifest, not a separable header; no `PackHeader` type exists (`rg` for pack header in Eidetic: none).
- Isocosm's isometer-mesh profile claims a `mere.pack/v1` artifact with its own raw-bytes versioned header, so the version and refusal path is already a second shape.
- Crate: `eidetic` already holds the right home. A header split out of `PackManifest` (version, schema id, refusal) can sit in `pack.rs` or a small `pack` submodule beside it. If F4 slims `eidetic`, `pack` belongs on the stay side (schema, codicil, typed, seal). No new crate.

Both obligations land in existing crates; Eidetic should not spend a name on either.

## 2026-11-07: wgpu-* squish audit (read-only, Sonnet subagent, no builds)

# wgpu-* squish audit (read-only, 2026-11-07)

Scope: inside each of wgpu-cue, wgpu-graft, wgpu-scry, wgpu-weld (D7/D10 keep the repos separate).
Method: README, design_docs/DOC_README.md + DOC_POLICY.md, every Cargo.toml and lib.rs doc, `wc -l` over
`src` (no target), manifest greps in mere/genet/netrender/turnstone/isocosm/woodshed via `git ls-files`
(so `.claude/worktrees` and `target` are excluded). No cargo build run, no network.
Line counts are physical lines of `src/**/*.rs` only (tests inside src are counted).

Headline: the library crates are already one-intention-per-crate. The sprawl is (a) one leftover directory and
lib name, (b) demo and probe crates, (c) duplicated "native frame" and web-surface vocabulary between scrying
and welding that sits above the crate level. Real name-freeing is small; the larger prizes free duplicate type
names, not crate names.

External dependents key: M = mere, T = turnstone. genet, netrender, isocosm and woodshed have no manifest
dependency on any crate below (genet's one hit is a comment).

## wgpu-cue (no design_docs, no DOC_POLICY.md; docs/contract.md plus 17 dated receipts)

| Crate (package) | Path | src lines | Dependents in repo / outside | Published | Fits its job | Disposition | Name freed |
|---|---|---|---|---|---|---|---|
| `webcue` | crates/webcue | 1443 | wgpu-cue, cue-genet, cue-netrender, cue-scry, execution-proof / M workspace, M inker | yes, 0.1.2 | Wholly: renderer-free Surface control, protocol, frame slots, execution ledger | Keep (the wgpu-free boundary is the point; Fork F5) | none |
| `wgpu-cue` | crates/wgpu-cue | 841 | cue-genet, cue-netrender, cue-scry, execution-proof / M workspace, M graphshell, M pelt-core, T | yes, 0.1.3 | Wholly: Presenter plus Metal, Vulkan DMA-BUF and DX12 importers over `grafting` | Keep | none |
| `cue-netrender` (lib `wgpu_cue_netrender`) | adapters/netrender | 1878 | cue-genet, execution-proof / M workspace, M graphshell, M pelt-core | no | Mostly: retained renderer on the host device. cpu, hybrid, render_session, resource_table, scene_job are render-job machinery rather than a producer adapter | Keep (Fork F3) | none |
| `cue-genet` (lib `wgpu_cue_genet`) | adapters/genet | 515 | execution-proof / none | no | Wholly: a Genet document session as a Webcue Surface | Keep (Fork F3: merge into cue-netrender) | none |
| `cue-scry` (lib `wgpu_cue_scry`) | adapters/scry | 442 | none / none (macOS-only target deps; lib.rs is 8 lines, wk.rs 434) | no | Wholly, but WKWebView only | Keep (Fork F4: move into scrying) | none |
| `cue-execution-proof` | support/execution-proof | 7052 | none / none | no | A private proof harness: Metal/Vulkan/DX12 handoff, window receiver, render stream, two bins | Keep as support, not a product name (Fork F6) | none |
| `cue-registry-receiver-proof` (x2) | receipts/2026-10-22-release/registry-consumer, receipts/2026-10-23-receiving/registry-consumer | n/a | not workspace members / none | publish=false | Frozen receipt fixtures; the same package name appears twice | Keep as archive | none |

Notes: workspace.package.version is 0.1.2 and `wgpu-cue` overrides to 0.1.3. The README documents the five
library crates as distinct boundaries and matches the manifests. cue-genet has no manifest dependent outside the
repo; mere and turnstone reach Genet content by other paths (inker and pelt-core use webcue, wgpu-cue and
cue-netrender, not cue-genet).

## wgpu-graft (DOC_POLICY.md and DOC_README.md present)

| Crate (package) | Path | src lines | Dependents in repo / outside | Published | Fits its job | Disposition | Name freed |
|---|---|---|---|---|---|---|---|
| `grafting` | grafting | 8346 | grafting-servo, 9 demos / T (git rev 9f410f1); wgpu-cue, scrying, welding, cue-execution-proof (registry) | yes, 0.6.1 (2026-10-24) | Core: Vulkan/Metal/DX12 import, sync, GL. But `surfman_gl` (1839 lines) is Servo's GL layer and rides in the default features | Keep (Fork F2: move `surfman_gl` to grafting-servo) | none |
| `grafting-frame` | grafting-frame | 469 | grafting / T (=0.1.1), webcue | yes, 0.1.1 (2026-10-23) | Wholly: wgpu-free owned frame and fence custody | Keep (Fork F1: merge into grafting) | none |
| `grafting-servo` (dir `servo-wgpu-interop-adapter`, lib `servo_wgpu_interop_adapter`) | servo-wgpu-interop-adapter | 762 | 8 Servo demos / T (git; lib name used in src/shell/servo.rs); M graft-engine deliberately none | no (CHANGELOG: unpublished) | Wholly: Servo offscreen adapter | Keep; finish the rename (dir and lib name still the old name) | `servo-wgpu-interop-adapter` (dir and lib name) |
| `demo-support` | demo-support | 111 | 8 demos / none | no key | URL helper and ANGLE DLL build | Keep; absorb duplicated `keyutils.rs` | none |
| `demo-raw-gl` | demo-raw-gl | 557 | none | no key | Servo-free GL demo | Keep (the only non-Servo demo) | none |
| `demo-servo-winit` | demo-servo-winit | 1423 | none | no key | Reference gate demo | Keep | none |
| `demo-servo-iced` | demo-servo-iced | 1019 | none (excluded workspace, own lockfile, wgpu 28) | no key | iced demo | Keep (cannot join: web-sys pin) | none |
| `demo-servo-xilem`, `-blitz`, `-egui`, `-gpui`, `-bevy`, `-slint` | demo-servo-* | 866, 731, 679, 710, 406, 198 | none | no key | One GUI framework each, distinct dependency graphs | Keep; they cannot share a crate (different GUI crates and wgpu majors; Bevy and Iced are DX12-only) | none |
| `freetype-sys` shim, `yeslogic-fontconfig-sys` | patches/* | n/a | [patch.crates-io] only | no | Vendored patch crates; they hold no name of ours | Keep | none |

Notes: `keyutils.rs` is byte-identical in demo-servo-blitz, -winit and -xilem (331 lines each); egui (148),
gpui (224) and iced (322) differ. Folding the three identical copies into `demo-support` deletes 662 lines and
frees no name. Graft's demo manifests carry no `publish = false` (weld's do). README's "nine demos" matches.

## wgpu-scry (DOC_POLICY.md and DOC_README.md present)

| Crate (package) | Path | src lines | Dependents in repo / outside | Published | Fits its job | Disposition | Name freed |
|---|---|---|---|---|---|---|---|
| `scrying` | scrying | 28545 | all 6 demos / M scrying-engine (0.7.1 pin, reaches pelt-desktop), T via scrying-engine, cue-scry (git rev 6100df6, macOS) | yes, 0.7.2 (2026-10-07) | Wholly: system-webview producers. Modules: wkwebview 7760, wpe 4966, webview2 4946 + windows_capture 1005, webkit6 2741, webkitgtk 2536, native_frame 1771, lib 2016 | Keep: one crate, five backends as modules. `native_frame` is Fork F2b | none |
| `demo-win` | demo-win | 5895 | none | no key | Over its job: a 5.9k-line WebView2 probe with smokes, not "minimal" | Keep per OS (Fork F7) | none |
| `demo-mac` | demo-mac | 4753 | none | no key | Probe; main.rs alone is 4165 lines | Keep per OS (Fork F7) | none |
| `demo-linux` | demo-linux | 859 | none | no key | WebKitGTK 4.1 probe | Host of one merged Linux demo crate | none (host) |
| `demo-linux6` | demo-linux6 | 543 | none | no key | WebKitGTK 6.0 probe; same job as demo-linux | Merge into demo-linux as a second bin | `demo-linux6` |
| `demo-wpe` | demo-wpe | 240 | none | no key | WPE probe | Merge into demo-linux as a bin | `demo-wpe` |
| `demo-scrying-winit` | demo-scrying-winit | 169 | none | no key | Cross-platform winit smoke; two cfg lines | Merge into the same crate (rename it `demo-scrying`) as a bin | `demo-scrying-winit` |

Notes: the demos carry no `publish = false`, so a workspace publish would try to publish them.
`line-ceiling.toml` lists demo-linux, demo-mac, demo-win and others as size exceptions.

## wgpu-weld (DOC_POLICY.md and DOC_README.md present; PROJECT_DESCRIPTION.md named but not yet created)

| Crate (package) | Path | src lines | Dependents in repo / outside | Published | Fits its job | Disposition | Name freed |
|---|---|---|---|---|---|---|---|
| `welding` | welding | 13395 | 3 demos / M weld-engine (alias `welding-0-15`), T direct (0.15.0) | yes, 0.15.0 (2026-10-17 per the triplet plan; repo README still says 0.14.1, 2026-09-04) | Wholly: CEF OSR into wgpu. Modules: linux_cef 2138, macos_cef 2194, windows_cef 2288, native_frame 1761, surface 1653, runtime 601, plus cookies, permissions, downloads, drag, auth, ime, popup, profile, devtools, snapshot | Keep; `native_frame` is Fork F2b | none |
| `demo-weld-win` | demo-weld-win | 1922 | none | publish=false | Windows demo; cdylib plus 2 bins (direct, bundle) | Keep per OS (Fork F8) | none |
| `demo-weld-linux` | demo-weld-linux | 1995 | none | publish=false | Linux demo | Keep per OS (Fork F8) | none |
| `demo-weld-mac` | demo-weld-mac | 2174 | none | publish=false | macOS demo | Keep per OS (Fork F8) | none |

Notes: `testing/` holds ten HTML probes and no Rust.

## Squish candidates (ordered by names freed per 1,000 lines moved)

Dependency edges used for every check (arrow = "depends on"): webcue -> grafting-frame; wgpu-cue -> webcue,
grafting; cue-* -> webcue, wgpu-cue; cue-scry -> scrying -> grafting -> grafting-frame; welding -> grafting;
grafting-servo -> grafting. Demo crates are leaves: nothing depends on any of them (manifest grep, all seven repos).

1. Finish the `grafting-servo` rename. Frees `servo-wgpu-interop-adapter` (directory and lib name
   `servo_wgpu_interop_adapter`; the package name already changed, per CHANGELOG [Unreleased]). 1 name, 0 lines
   moved (a `git mv` plus the `use servo_wgpu_interop_adapter` paths in 8 demos; turnstone's
   src/shell/servo.rs also names the lib, outside this audit's scope). Ratio: unbounded.
   Cycle check: package name unchanged, so no edge changes.
2. wgpu-scry Linux and winit probes into one crate: fold demo-linux6 (543), demo-wpe (240) and
   demo-scrying-winit (169) into demo-linux as bins. 3 names / 952 lines = 3.2 per 1,000. Cycle check: all four
   are leaves depending only on scrying; no new edge. Caveat: `webkitgtk-fallback` (gtk 0.18) and `webkit6`
   (gtk4) features unify in one build, so the bins need `required-features` and scripts/test-linux.sh builds per bin.
3. wgpu-weld demos into one `demo-weld` with three bins: 2 names / about 4,170 lines moved (keeping the Windows
   crate as host) = 0.48 per 1,000. Cycle check: leaves on welding; no new edge. Caveat in Fork F8.
4. wgpu-scry demo-mac and demo-win into the same crate as OS bins: 2 more names / 10,648 lines = 0.19 per 1,000.
   Cycle check: leaves on scrying; target-gated deps (`windows`, `webview2-com`, objc2) already exist per manifest.
   Not recommended before the ceiling debt in line-ceiling.toml is split (Fork F7).
5. Hygiene, no name freed: three identical `keyutils.rs` into `demo-support` (662 lines deleted).

Nothing in wgpu-cue squishes: every crate is a distinct dependency boundary and no crate's job is a module of a
neighbour (the closest pair, cue-genet and cue-netrender, is Fork F3).

## Forks (two defensible answers each)

**F1. grafting-frame into grafting.** Merge: grafting already re-exports all of it (`pub use grafting_frame`);
frees `grafting-frame`, one fewer publish. Keep (my lean): grafting-frame has only target-gated `windows` and
`objc2` deps (469 lines, no wgpu), while `grafting` needs one of wgpu-28/29/30 and defaults to wgpu-29 plus
surfman (euclid, gleam, image, surfman). `webcue` (published, wgpu-major independent, used by mere inker)
depends on grafting-frame for exactly that reason, and turnstone pins `=0.1.1` directly. A merge forces wgpu or
a feature gate onto webcue. Frees 1 name for 469 lines and breaks the contract's stated independence.

**F2. grafting `surfman_gl` (1839 lines, default-on `surfman` feature) moves to grafting-servo.**
Move: surfman is Servo's GL layer; its users are grafting-servo, demo-servo-winit and build.rs only (turnstone,
scrying, welding, wgpu-cue all use default-features = false). Keeps `grafting` a native-handle importer.
Keep: the 0.6.1 CHANGELOG records Surfman 0.13 to 0.14 as the break that already cost a patch release, moving a
public module is a breaking change again, and `grafting::raw_gl` (1739 lines, GL, used by the Servo-free
demo-raw-gl) would then be the only GL left, so "GL producer family" may be the intended single intention.
Frees no crate name either way.

**F2b. `scrying::native_frame` (1771) and `welding::native_frame` (1761) versus `grafting`.** Both re-declare
`InteropBackend`, `NativeFrame`, `NativeFrameKind`, sync types and DMA-BUF/Metal/DX12 frame wrappers, and both
call `grafting::*` imports underneath (scrying native_frame/dmabuf.rs line 64; welding
native_frame/vulkan_dmabuf.rs line 153). `InteropBackend` is defined three times (scrying, welding, grafting).
Move into grafting: removes about 3.5k duplicated lines and two copies of the vocabulary without merging
repos. Keep: the shapes differ (scrying `DmaBufPlane` offset is u32, welding's is u64 plus size; scrying has
`TextureImporter` and `ProducerCapabilities`, welding has `CpuBitmap` and `PendingFrameSlot`), and each wraps
producer policy around an import; a shared type forces one shape on both producers. Frees duplicate type names,
no crate. It changes grafting's public API, so it is Mark's call.

**F3. cue-genet into cue-netrender.** Merge: cue-genet is 515 lines, built entirely on cue-netrender, and its
only dependent is the proof harness. Keep (my lean): cue-netrender's dependents (pelt-core, graphshell, the mere
workspace) take only the renderer; cue-genet adds genet-documents, genet-presenter and genet-host-api git deps at
one genet rev (de14e67) that they would then inherit. Also cue-netrender holds about 1,500 lines of render-job
machinery (cpu, hybrid, render_session, resource_table, scene_job); splitting that out would add a name, not
free one. Merge frees 1 name for 515 lines.

**F4. cue-scry into scrying (behind a `cue` feature).** Move: 442 lines, macOS-only, the other half of
scrying's WKWebView producer; frees `cue-scry`. Keep (my lean): scrying is published at 0.7.2 with its own
consumers and would gain webcue, wgpu-cue and through them document-session-api and accesskit, none of which it
has today; mere's inker places its adapters (scrying-engine, weld-engine, graft-engine) with the contract owner,
not the producer. Cycle check for the move: scrying -> webcue -> grafting-frame, and nothing in webcue or
wgpu-cue depends on scrying once cue-scry is gone, so no cycle.

**F5. webcue and wgpu-cue as one crate with a `wgpu` feature.** Merge: 2284 combined lines, frees 1 name.
Keep (my lean): webcue is deliberately "independent of the engine and the wgpu major" (README and lib doc),
inker depends on webcue only, and wgpu-cue pins `wgpu = 30.0.1` exactly. Merging changes a published 0.1 pair.

**F6. cue-execution-proof (7052 lines).** Keep: it is the instrument behind the 2026-10 and 2026-11 receipts
(features metal-handoff, vulkan-handoff, dx12-handoff, window-receiver); nothing depends on it. Retire or move
to a tests/examples layout: it is `publish = false` support, so the name is not a purpose a consumer sees, but
it also holds product-shaped code (render_stream 399, render_wire 134, process 193). Frees at most 1 name;
decide when the receipts are archived.

**F7. wgpu-scry: one demo crate for all six probes, or per OS.** One: frees 5 names (12,459 lines re-homed) and
one shared winit/wgpu host. Per OS (my lean, candidate 2 only): demo-mac (4753) and demo-win (5895) are large,
OS-specific (webview2-com plus the `windows` crate; objc2) and on the ceiling-exceptions list, so merging moves
debt before it is split.

**F8. wgpu-weld: three demos into one.** One `demo-weld`: frees 2 names. Three: demo-weld-win is a cdylib plus a
`bundle-demo-weld-win` bin using `cef` `build-util`; each OS demo has its own cfg-specific blit, present and
probe modules; they share file names (main, scripted, probe) but not content. Weak evidence either way.

**F9. scrying and welding (with graft's Servo lane) as one purpose.** For: both re-declare the same web-surface
vocabulary (`NavigationEvent`, `Cookie`, `SameSite`, `CursorShape`, `DragInput`, `KeyEvent`, `WebRequestId`;
scrying lib.rs 2016 lines, welding surface.rs 1653), carry an identical lib.rs wgpu alias header and fd-test
lock, a `native_frame` module and per-OS producers; Webcue `Surface` is a third contract already abstracting
them. Against (my lean): a different native dependency (OS webview libraries versus the CEF 154 bundle with
subprocess and sandbox setup), different cadence (scrying 0.7.2, welding 0.15.0), separate hardware CI, and D7
already ruled them separate. The squish available is shared vocabulary (F2b), not a repo merge.

## What each repository is for

wgpu-cue is the embedding contract: a renderer-free control protocol (`webcue`), host-device presentation
(`wgpu-cue`) and the adapters that make Genet or WKWebView content a Webcue Surface, so a host such as inker,
pelt or turnstone never names an engine. wgpu-graft is the importer: `grafting` turns a producer's native GPU
image (GL, DMA-BUF, IOSurface, DX12 shared handle) into a texture on the host's own wgpu device, with
`grafting-frame` as the wgpu-free custody layer, and the Servo adapter plus nine demos as its first producer and
consumers. wgpu-scry is the producer for the operating system's own webview (WebView2, WKWebView, WebKitGTK,
WPE), exposing frames, navigation, input and cookies through `scrying`. wgpu-weld is the producer for Chromium
through CEF accelerated off-screen rendering (`welding`), including the CEF subprocess, profile and sandbox
policy that has no counterpart in the other three.

## 2026-11-07: P1 lane under D15 to D17

`target/` and the consumer scratch copies had been cleared; rebuilt cold.

- `cargo check --offline -p mere-lanes -p illume -p knot-editor-host --features
  mere-lanes/{netfetch,smolweb,retained-clips},illume/markdown --tests`: green, 1m 08s.
- `cargo test` of the same: Illume 39 (gains the 2 lexer tests), `knot-editor-host` 7 (loses
  them), `mere-lanes` 91 lib (88 plus 3 reading-type tests) + 1 `smolweb_streaming_render`.
- `cargo check --offline --workspace --all-targets --exclude mere-resident --keep-going`:
  fails only in `tabard-workshop` (1 site, `src/reader.rs:134`) and `graphshell` (5 sites,
  `src/retained_content.rs` 299, 305, 306, 313, 315), lib and lib test. The workshop was
  converted at its edge; `cargo test -p tabard-workshop`: all suites pass (14, 4, 1, 4, 16, 1,
  1 with 1 ignored, 4, 6, 8, 7). The Graphshell port is the recorded D16 follow-up; its
  dependents were not checked past it.
- Standalone workspace regenerated (same generator, font copy placed again, lock seeded from
  Mere's). `cargo metadata --all-features`: Inker edges into Mere or Graphshell are
  `knot-editor-host -> cambium`, `mere-fleece -> mere-rdf` (optional), `mere-lanes -> tabard`
  (palette only), `script-rhai -> hest`; the other edges listed run from `mere.git` packages
  onward to graphshell.git (`hest`, `sceno`, `sceno-backdrops`).
- `cargo check --offline --workspace --all-targets` with the lanes, Fleece, canvas and
  `illume/markdown` features: green, 2m 19s. `cargo test --no-fail-fast` with the same: 765
  passed, 0 failed.
- Turnstone and Knot were not rechecked: under D16 Turnstone does not compile against these
  lanes until it converts, and Knot's `knot-document` is being repinned in its own lane.

## 2026-11-07: P1 lane after D15 and D19 (rebased onto main `4b99ce9f2`)

- The staged work was saved as a patch, the branch moved to main (its old head was an
  ancestor), and the patch reapplied cleanly; plan, receipts and lock were taken from main and
  the lane's edits redone.
- The smolweb palette moved into `mere-lanes` (`src/smolweb/palette.rs`, Tabard's module and
  its four tests, doc lines changed) over `tinct` from graphshell.git `branch = "main"`; the
  Tabard row left the lanes' manifest.
- `cargo check -p mere-lanes` (online once, to fetch graphshell `d80316db`): green. The lock
  then held graphshell.git at two revisions (ten packages at `40e8c8df`, `tinct` at
  `d80316db`) and offline Cargo stopped with an internal error ("failed to find tinct ... in
  path source"); `cargo update -p` on the ten graphshell packages moved them to `d80316db`,
  changing nothing else.
- `cargo check --offline --workspace --all-targets --keep-going`: fails only in `graphshell`
  (lib and lib test), the five D16 sites in `src/retained_content.rs` (299, 305, 306, 313,
  315). `mere-resident` checks now.
- `cargo test` of `mere-lanes` (netfetch, smolweb, retained-clips), `illume/markdown`,
  `knot-editor-host`, `tabard-workshop`: lanes 95 + 1, Illume 39, `knot-editor-host` 7,
  workshop suites all pass.
- Standalone workspace regenerated (37 inherited dependencies; rows rewritten to the Mere
  repository: `cambium`, `mere-rdf`). `cargo metadata --all-features`: Inker edges into Mere
  are `knot-editor-host -> cambium` and `mere-fleece -> mere-rdf` (optional); into Graphshell's
  contract root `mere-lanes -> tinct` and `script-rhai -> hest`; no Tabard. Check with the
  lanes, Fleece, canvas and `illume/markdown` features: green, 48.3s. `cargo test
  --no-fail-fast`: 769 passed, 0 failed.
- Graphshell port converted at its five D16 sites (option (a)). `cargo check -p graphshell
  --all-targets --offline`: green, 10m 07s. `cargo test -p graphshell --offline --lib --
  retained_content reading`: 19 passed, 1 ignored (needs a real wgpu adapter), 317 filtered,
  including `reading_controls_keep_held_accesses_and_search_independent`.

## 2026-11-08: D30 lane, chrome's frame model retired and the rest audited

Branch `p0/chrome-retire`, worktree `worktrees/mere-chrome-retire`, cut from main `a1a61df2f`.

- Consumer check before deleting: `grep -rn --include=*.rs -E
  '\b(FrameHostInput|FrameViewModel|FrameStep|HostEvent|DegradedReceiptSpec|HostIntent|host_intent|effects::)'
  crates ports` outside chrome finds no code that names chrome's types. The hits are the kernel's
  own `HostEvent` (`graph-kernel/src/host_event.rs`), winit-host's private `HostEvent`, a local
  `effects` module in the kernel's legacy migration, and doc comments only: kernel `geometry.rs`,
  `overlay.rs`, `graph/identity.rs`, `actions.rs`, `host_event.rs`, and ux-events
  `ux_observability.rs`, `ux_probes/productive_selection.rs`. Those comments now name deleted types
  and were left for the kernel's and ux-events' owners. Turnstone and Knot (read-only): no hit;
  Knot does not depend on `mere-chrome`.
- Deleted: `frame_model.rs` (483), `frame_model/settings.rs` (152), `frame_model/tests.rs` (167),
  `effects.rs` (82), `host_intent.rs` (139), 1,023 lines; plus a five-line test comment in
  `authorities.rs` pointing at `frame_model.rs`. The crate doc, README and manifest description
  were rewritten for what remains. Commit `df0141622` totals: 24 insertions, 1,052 deletions.
- Manifest: `hest` and `incipit` (with `arrangement`) removed, both used only by the retired
  modules. The lock drops exactly those two edges from `mere-chrome`; nothing else moved.
- `cargo check --offline -p mere-chrome -p ux-events -p graphshell`: green, 6m 23s (graphshell's
  nine warnings are its own). `cargo test --offline -p mere-chrome -p ux-events`: 44 and 40 passed.

### Audit of what remains (read-only; dispositions are a fork for Mark)

Searched `mere/crates`, `mere/ports`, `turnstone/src`, `knot-editor`, `isocosm`, `cleromancy`,
`woodshed`, `graphshell/crates` by module path (`chrome::<module>`) and by braced import
(`chrome::{...}`), excluding `target/` and `.claude/worktrees`. Only two manifests depend on
`mere-chrome`: ux-events and Turnstone. The braced hits (`cambium/src/workspace.rs`,
`mesocosm-genet`, `tabard::theme::chrome`) are local modules named `chrome`, unrelated.

| Module | Lines | Importers outside chrome | Own version elsewhere | Disposition under D10 |
| --- | --- | --- | --- | --- |
| `authorities` | 400 | none | none found | retire |
| `omnibar` | 398 | none (only `suggest`, inside chrome) | Turnstone's omnibar (`observe/omnibar_view.rs`, `shell_services.rs` `OmnibarConfig`/`OmnibarScope`, `a11y/omnibar_rows.rs`); no omnibar in the Graphshell port | retire |
| `suggest` | 326 | none | Turnstone's `observe/suggestion_line.rs` and its omnibar rows | retire |
| `toolbar` | 215 | none | Pelt core's own load and history state (`ports/pelt/core`) | retire |
| `routing` | 69 | ux-events, `command_surface_telemetry.rs:29` | none; built from kernel types (`GraphViewId`, `NodeKey`, `ToolPaneState`), and kernel `pane.rs:109` already documents it | move to ux-events (one consumer), or to the kernel beside the types it is made of |
| `nav` | 259 | Turnstone, `chrome::nav::History` at four sites (`app/mod.rs:219`, `app/fixtures.rs:124`, `app_boot.rs:107`, `app_adopt_session.rs:386`) | `History` is `backstep::nav::History` re-exported; Turnstone has its own classifier (`ui/normalize_address.rs`) and uses `genet_documents::resolve_href` | `History`: Turnstone repoints to `backstep` (one consumer, already its home); `NavTarget`, `classify`, `resolve_href`: retire |

- ux-events takes exactly one item: `chrome::routing::ToolSurfaceReturnTarget`. It takes nothing
  from `toolbar`; D30's evidence line naming `toolbar` as a ux-events import does not match the tree.
- No module has two consumers, so D10 keeps none. If the dispositions stand, chrome dissolves:
  `routing` moves (69 lines), Turnstone swaps one dependency row and four paths for `backstep`, and
  1,598 lines retire.
- Freed: the package name `mere-chrome` (0.0.1 from the workspace version, no `publish` field,
  not on crates.io: the API answers "crate `mere-chrome` does not exist") and the workspace row
  `chrome = { package = "mere-chrome", ... }` in Mere's root manifest.

## 2026-11-09: P3 slice 1 (D35 to D40), branch `es/p3-slice1` in six repositories

Every branch starts from that repository's `origin/main`. Mere's branch was rebased onto
`68ba84b1a`, whose new commits change only documents. History was imported the way
`sceno-backdrops` was: a `--no-local` clone of Mere at `e9e9b0aec`, `git filter-repo --path ...
--path-rename ...`, then `git merge --allow-unrelated-histories --no-ff`.

- **Eidetic** `22deb33` (import merge `b5467b5`, 4 commits of history): `mere-edit` at
  `crates/source-edit`, added as a workspace member. `cargo test -p mere-edit` ran 6 passed, the same
  as the 6 test attributes. The pre-commit hook (line ceiling, `cargo clippy --locked --all-targets`)
  passed.
- **Graphshell** `64813dff` (import merges `c58cb48c` with 173 commits of history, and `b8f240b6` for
  fixtures): `host/` holds tabard-workshop 4,448 lines, document-host 3,722, ux-events 2,183,
  graphshell-client-tests 9 and pelt-desktop 23,765 (lines of Rust in each crate's source).
  - The contract root: `cargo check --workspace --all-targets --locked` is green, and its `Cargo.lock`
    keeps sha256 `df67a47c...` before and after.
  - `host/`: `cargo check --workspace --all-targets --locked` is green. The lock was seeded from Mere's
    `Cargo.lock`, then resolved with mere.git at `72888bb8`. Its 987 packages hold 13 path packages:
    the 5 host crates and 8 contract crates.
  - Tests: `cargo test --workspace --no-fail-fast` in `host/`, against `cargo test --no-fail-fast` with
    `-p` for the same five crates in Mere at `e9e9b0aec`. Both runs give identical per-crate results:

    | crate | passed | failed | ignored |
    |---|---|---|---|
    | document-host | 56 | 0 | 3 |
    | graphshell-client-tests | 30 | 0 | 0 |
    | pelt-desktop | 85 | 0 | 0 |
    | tabard-workshop | 75 | 1 | 3 |
    | ux-events | 43 | 0 | 0 |

    That is 296 in each run. The one failure is the same in both:
    `release_host_acceptance::release_accessibility_focus_click_and_numeric_value_keep_separate_authority`,
    `assert_ne!` at line 134 with both sides `Srgb { r: 51, g: 102, b: 200, a: 255 }`.
  - Fixtures copied with history, because the moved tests read them by relative path:
    `ports/pelt/examples` (57 files), `crates/cambium/examples/genet_web_smoke/assets/Roboto-Regular.ttf`
    and `crates/cambium/scenes/scenomise/fixtures/coastal_map.json`, all under `host/` at the same
    subpaths. Mere keeps `ports/pelt/examples/Ahem.ttf`, which Cambium's tests read.
  - 15 line-ceiling exceptions moved to Graphshell's root `line-ceiling.toml` with a `host/` prefix.
- **Turnstone** (import merges `9b0319b8`, with 106 commits of history, and `c67b426a`, the Tabard
  README; manifest changes staged, not committed):
  - The `pelt` and `tabard-desktop` binaries are now members at `ports/pelt` and
    `ports/tabard/desktop`. The root's `[workspace.package]` gained edition, license, repository and
    authors, plus a `[workspace.dependencies]` table of 16 rows at the sources Turnstone already names.
    The `tabard-workshop` row moved from mere.git to graphshell.git.
  - Two line-ceiling exceptions moved here.
  - `cargo check --workspace --all-targets`, run with the overrides listed below, is green.
  - `cargo test -p pelt -p tabard-desktop`: pelt 4 passed; tabard-desktop 16 passed. The crate has 18
    test attributes, and 2 of them sit in an example that `cargo test` does not run.
- **Knot** (staged): 2 rows. `tabard-workshop` in the root workspace table moved from mere.git to
  graphshell.git, and `mere-edit` in `crates/knot-document` from mere.git to eidetic.git.
  `cargo check --workspace --all-targets`, with the overrides, is green.
- **Isocosm** (staged): 1 row, `mere-edit` in `crates/isocosm-system` from mere.git to eidetic.git.
  `cargo check --workspace --all-targets` is green. `cargo test -p isocosm-system --all-features` ran
  116 passed.
- **Overrides.** These are local and uncommitted, and stand in for revisions not yet pushed. They are
  passed as `cargo --config`:
  - `patch.'graphshell.git'` and `patch.crates-io` for the 11 contract crates, pointing at the
    Graphshell branch;
  - `patch.'graphshell.git'` for `tabard-workshop` and `pelt-desktop`, pointing at `host/`;
  - `patch.'eidetic.git'` for `mere-edit`;
  - for Turnstone, `patch.'knot-editor.git'` for knot-editor, knot-composition and knot-document,
    pointing at the Knot branch.

  The resolved locks were kept outside the repositories, and each repository's lock was restored to
  its `origin/main` copy, because none of these sources exists upstream yet.
- **Mere** `68ba84b1a`, everything staged, nothing committed (Mark: wait on `responsa`):
  - Removed members: source-edit, ux-events, document-host, graphshell-client-tests, gazette-ledger,
    tabard/desktop, tabard/workshop, pelt and pelt/desktop.
  - Removed rows: the workspace rows for ux-events, pelt-desktop, tabard-workshop and gazette-ledger,
    and the root's `[patch."mere.git"]` rows for mere-edit and tabard-workshop. Also the mere-edit
    patch row in `ports/graphshell/web/Cargo.toml` and two `.gitignore` guest-lock exceptions.
  - 18 line-ceiling exceptions left: 17 moved out with their files, and the stale
    `embedded_reservoir_validation.rs` entry, which Mark approved dropping.
  - `cargo check --workspace --all-targets` is green (4 min 57 s). The lock lost 125 packages (the 9
    moved or retired, and 116 only they reached) and gained none.
  - `gazette-ledger` retired (D39). The subject of its parity test, Gazette's three Ledger receipts
    reproduced through `scenomise::matrix`, was the retirement path for `ledger.rs` (mer3ly site
    canvas plan, Ruling 143). Its 3 unit tests and 3 parity tests retire with it. `scenomise`'s own 8
    matrix tests stay, including `value_cells_carry_both_axis_inputs`.
- **Checker (D35).**
  - `scripts/test_check_family_cycles.py`: 18 tests pass, 3 of them new: the two Graphshell roots as
    separate nodes, a host package named from below as a cycle, and a repository allowlist entry
    covering its roots.
  - `scripts/check_family_cycles.py --family-dir C:/t/p3-family` (this slice's six branches beside the
    other local checkouts): exit 0 once the stale `[[isolated]] murm/apps/signalman-desktop` (D17)
    entry is deleted. With it, the run exits 1 and names that entry as stale.
  - The edges include `graphshell/host -> mere` and `mere -> graphshell`, with no cycle between them.


## 2026-11-09: P0 conformance ledger, first run (branch `es/ledger`, worktree `C:/t/mere-es-ledger`)

Base: Mere main `31dc9107d`. Adapter: `crates/graph/linked-data` (package `mere-rdf`), last changed in `60622438`. Ledger: `mere_docs/testing/2026-11-09_conformance_ledger.md`. Harness: `crates/graph/linked-data/tests/w3c_conformance/` (`main.rs`, `manifest.rs`, `rdf.rs`, `sparql.rs`, `jsonld.rs`, `report.rs`; new; no manifest touched, no dependency added). No W3C manifest runner existed in Mere, Eidetic or the family (grep for `manifest.ttl`, `mf:entries`, `rdf-tests`, `json-ld-api` over every repository's `crates`, `tests` and `ports` found none).

**Suites** (plain `git clone` of `origin` into `C:/Users/mark_/Code/crates/<name>`, nothing else fetched):

| Clone | HEAD | Commit date | Branch | Size |
|---|---|---|---|---|
| `rdf-tests` | `202a74eb81f47bcad13bbafdd37eba69c361c1c2` | 2026-11-04 | main | 39 MB |
| `json-ld-api` | `55514738d854e56bb8bad4c53cda2f0e23be4a50` | 2026-10-07 | main | 43 MB |
| `data-shapes` | `9c102cbb5309fe2662cb6b608e4c17413df357d8` | 2026-11-08 | gh-pages | 37 MB |
| `rdf-canon` | `15619df2fda7a4ca88308733789b6774517f9638` | 2026-02-24 | main | 6.7 MB |

`data-shapes` clones onto `gh-pages` (its default branch); recorded as cloned.

**Commands** (Git Bash, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_BUILD_BUILD_DIR=C:/t/cargo-build/mere-es-ledger` by environment because `.cargo/config.toml` is tracked and cannot be a gitignored local file, `-j 4`, one cargo command at a time):

- `cargo check -j 4 -p mere-rdf --features query --test w3c_conformance`: clean (the only warnings are the existing `mere-kernel` ones).
- `cargo check -j 4 -p mere-rdf --tests` (no `query` feature): clean; the SPARQL part is `cfg(feature = "query")`.
- `W3C_SUITES_DIR=C:/Users/mark_/Code/crates cargo test -j 4 -p mere-rdf --features query --test w3c_conformance -- --include-ignored --nocapture`: `2 passed`; the run takes about 1 s. A second run after `rustfmt` gave byte-identical ROW, LEAF and FAIL lines.
- `W3C_SUITES_DIR=C:/nonexistent` the same command: prints `W3C suites absent ... skipping`, `2 passed`.
- Controls (test `instrument_controls`, runs by default): a bad N-Triples line is rejected, a good one parses to one quad, blank-node renaming is isomorphic and a real difference is not, `from_nquads` rejects and accepts as expected.

Two harness faults found and fixed during the first runs, kept here because they changed counts: (1) the SPARQL syntax tests were first run without a base IRI and 20 failed on relative IRIs; with the test's own address as base all 106 + 63 pass (the adapter's own call has no base, so the ledger says so); (2) a negative JSON-LD test passed whenever a context was unreachable, so rejections caused by a context outside the suite are now skipped, and the adapter's context cache holds every `.jsonld` file under `tests/`.

**Counts** (pass / fail / skip; `parser` = `oxttl`, `oxjsonld`, `spargebra` as the adapter configures them; `adapter` = `from_nquads`, `from_trig`, `from_jsonld_with_contexts_and_base_iri`):

| Suite, category | parser | adapter |
|---|---|---|
| rdf12 syntax (N-Triples 33/1/0, N-Quads 31/0/0, Turtle 77/0/0, TriG 38/0/0, RDF/XML 0/0/2) | 179 / 1 / 2 | 69 / 0 / 113 |
| rdf12 eval (Turtle 32, TriG 26, RDF/XML 0/0/29) | 58 / 0 / 29 | all 87 skipped |
| rdf12 c14n, entailment | 0 / 0 / 82, 0 / 0 / 29 | all skipped |
| rdf11 syntax | 539 / 0 / 40 | 286 / 15 / 278 |
| rdf11 eval | 288 / 0 / 126 | all 414 skipped |
| rdf11 entailment | 0 / 0 / 48 | all skipped |
| sparql11 query syntax | 106 / 0 / 0 | all skipped |
| sparql11 query eval | 0 / 0 / 247 | all skipped |
| sparql11 update syntax | 63 / 0 / 0 | all skipped |
| sparql11 update eval | 0 / 0 / 94 | all skipped |
| sparql12 syntax | 200 / 3 / 0 | all skipped |
| sparql12 eval (65 query, 3 update) | 0 / 0 / 68 | all skipped |
| jsonld toRdf (467 tests) | 439 / 10 / 18 | 441 / 1 / 25 |
| jsonld expand 386, compact 246, flatten 58, fromRdf 54, html 50, remote-doc 18 | all skipped | all skipped |

Entry counts per walked root: rdf12 380, rdf11 1,041, sparql11 query 353, sparql11 update 157, sparql12 271.

**Failing tests** (parser layer unless stated):

- rdf12: `ntriples-version-1` (`VERSION` directive in N-Triples rejected).
- rdf11 adapter, `from_trig`: 15 positive tests (`trig-syntax-number-01` to `-11`, `trig-syntax-datatypes-01` and `-02`, `trig-syntax-kw-01` and `-02`) rejected with "No scheme found in an absolute IRI": relative IRIs, and `from_trig` takes no base IRI. The same inputs pass at the parser layer with a base.
- sparql12 syntax: `nested-aggregate-functions`, `tripleterm-subject-03`, `tripleterm-subject-06` accepted; the suite says reject.
- jsonld toRdf, dataset not isomorphic to the expected output: `#t0122`, `#t0123`, `#tc013`, `#tdi02`, `#tdi04`, `#tdi05`, `#tdi06`, `#te062`, `#te091` (the `di` tests are the default-direction cases, which the adapter's `rdf-12` build answers with RDF 1.2 directional literals). `#tli12` rejected at both layers: base `http://invalid/<>/` is refused as an invalid IRI.

**Skip reasons**: RDF/XML (no parser in the adapter; `oxrdfxml` is in the lock but not a dependency of `mere-rdf`); C14N (no canonical output entry point); entailment (no stage); SPARQL query evaluation (comparison needs `sparesults`, not a dependency; the adapter's `query::sparql` runs SELECT and ASK over a kernel `Graph` only); update evaluation (no entry point); JSON-LD 1.0-only tests (11), `rdfDirection` (4), `expandContext` (1), `processingMode` 1.0 at the adapter (8); the other JSON-LD algorithms (no API). Not run: `data-shapes` (SHACL) and `rdf-canon` (RDFC-1.0), no engine stage yet.

## 2026-11-09 (later): ledger rerun with SPARQL query evaluation and RDF/XML (branch `es/ledger`, on Mere main `c480cefd9`)

Mark's rulings D61 to D63. Harness gained: `oxrdfxml` for RDF/XML (syntax and evaluation, RDF 1.1 and 1.2), `spareval` over an `oxrdf::Dataset` with expected results read through `sparesults` (SRX, SRJ, TSV; CONSTRUCT and DESCRIBE compared as graphs) for SPARQL query evaluation. Dev-dependencies of `mere-rdf` only: `oxrdfxml = "0.2"` (feature `rdf-12`) and `sparesults = "0.3"` (feature `sparql-12`).

**Lock diff** (`Cargo.lock`, +15 lines): one new package, `oxrdfxml` 0.2.5, resolved from crates.io (depends on `oxilangtag`, `oxiri`, `oxrdf`, `quick-xml` 0.37.5, `thiserror` 2.0.21, all already locked), plus the two dev-dependency edges on `mere-rdf` (`oxrdfxml`, `sparesults`). Nothing else new. `oxrdfxml` 0.2.4 pins `oxrdf =0.3.4`, which would have split the types; 0.2.5 uses the single locked `oxrdf` 0.3.5.

**Commands**: `cargo check -j 4 -p mere-rdf --features query --test w3c_conformance` (clean), then `W3C_SUITES_DIR=C:/Users/mark_/Code/crates W3C_VERBOSE=1 cargo test -j 4 -p mere-rdf --features query --test w3c_conformance -- --include-ignored --nocapture`: `2 passed`, 2 to 40 s depending on cache. Scoring: row order is not checked; numeric literals compare by value (integer, decimal, double, float), because the first pass compared lexical forms and failed 28 tests on `1` against `1.0`.

**Counts** (parser layer; pass / fail / skip): rdf12 syntax 181 / 1 / 0 (RDF/XML 2/0/0 included), rdf12 eval 85 / 2 / 0 (RDF/XML 27/2/0), rdf11 syntax 579 / 0 / 0 (RDF/XML 40/0/0), rdf11 eval 412 / 2 / 0 (RDF/XML 124/2/0), sparql11 query syntax 106 / 0 / 0, sparql11 query eval 230 / 14 / 3, sparql11 update syntax 63 / 0 / 0, sparql11 update eval 0 / 0 / 94, sparql12 syntax 200 / 3 / 0, sparql12 query eval 64 / 1 / 0, sparql12 update eval 0 / 0 / 3, jsonld toRdf 439 / 10 / 18. Adapter layer unchanged: rdf12 syntax 69 / 0 / 113, rdf11 syntax 286 / 15 / 278, jsonld toRdf 441 / 1 / 25; everything else skipped (no entry point).

**Failing tests by suite**
- rdf12: `ntriples-version-1`; RDF/XML `rdf12-xml-an-13`, `rdf12-xml-an-14` (not isomorphic).
- rdf11: RDF/XML `xml-canon-test001`, `xml-canon-test002` (not isomorphic); adapter `from_trig` as before (15 relative-IRI tests).
- sparql11 query eval (14): aggregates `agg-groupconcat-04`, `agg-groupconcat-06`; bindings `graph-from-named-1`, `-2`, `-3`, `values-construct-group-by` (rejected at parse: expected `UNDEF`); functions `bnode01`; negation `graph-minus`; property-path `zero_or_more_set_start`, `zero_or_more_set_end`, `zero_or_one_set_start`, `zero_or_one_set_end`; subquery `subquery15`, `subquery16`. Not verified as engine faults: the `graph-from-named`, `graph-minus`, property-path `set_*` and `subquery` cases involve named-graph data or FROM clauses, where the harness's graph naming and dataset loading may be at fault.
- sparql12: syntax `nested-aggregate-functions`, `tripleterm-subject-03`, `tripleterm-subject-06` (accepted); eval `select-variable-reuse` (rejected at parse).
- jsonld toRdf: as in the first run.

**Skips**: 3 sparql11 query-eval tests whose expected result is in a format the harness does not read; 94 + 3 update evaluation tests: `spareval` 0.2.9 evaluates queries, and its `PreparedDeleteInsertUpdate` covers DELETE/INSERT deltas only, so a full update run needs harness-written executor code for the rest (INSERT DATA, DELETE DATA, LOAD, CREATE, CLEAR, DROP, ADD, COPY, MOVE); returned to Mark as a fork.

## 2026-11-09 (last): controls for the SPARQL evaluation and RDF/XML fails

Rule (Mark): a failure is evidence only with a control. Base `c480cefd9` plus the previous ledger commit `d2b5488ba`.

**Harness check against the manifest semantics.** `qt:data` loads into the default graph, `qt:graphData` into a graph named by the file's address (`https://w3c.github.io/rdf-tests/<path>`), base IRIs are the test file addresses; all three were already so. One gap found and closed: FROM and FROM NAMED in a query did not load graphs the manifest data lacks. The harness now reads the query's dataset clause and, for a named IRI the data does not hold and that addresses a suite file, loads that file into the named graph. It changed no result: counts before and after are identical (sparql11 query eval 230 / 14 / 3, sparql12 query eval 64 / 1 / 0, RDF/XML eval 27 / 2 in 1.2 and 124 / 2 in 1.1), because no failing test relied on a FROM file. No harness fault was found behind any of the 19 fails.

**Controls** (`cargo test -j 4 -p mere-rdf --features query --test w3c_conformance engine_controls -- --nocapture`, with `W3C_SUITES_DIR` for the four RDF/XML cases; prints `CONTROL <name> MATCH|DIVERGES`, got, want). Positive controls match: a plain `GRAPH ?g` pattern returns g1 and g2, and the same pattern with `FROM NAMED ex:g1` returns g1 only. All 19 diverge from the expected result, each over inline data and a direct `spareval`, `spargebra` or `oxrdfxml` call, so all 19 are engine, none unconfirmed:
- `graph-from-named-1`: got four rows (`?g` unbound with x=1; g1/2; g2/3; unknown/4); want g1/1 and g1/2.
- `graph-from-named-2`: got g1, g2 and unknown each with x=1; want g1 only. `graph-from-named-3`: got x=1, 2, 3; want 1.
- `graph-minus`: got no rows; want `:a`.
- `zero_or_more_set_start`, `zero_or_more_set_end`, `zero_or_one_set_start`, `zero_or_one_set_end` over an empty dataset: got no rows; want the constant (`:o` or `:s`).
- `agg-groupconcat-04`, `agg-groupconcat-06`: ASK over GROUP_CONCAT of language-tagged strings: got false; want true.
- `subquery15`: got one row; want two. `subquery16`: got two rows, both `:s :o`; want the four rows `s1/o1`, `s1/o2`, `s2/o1`, `s2/o2`.
- `bnode01`: two distinct blank nodes across the four rows; want six.
- `values-construct-group-by`: `spargebra` rejects ("SELECT * is not authorized with GROUP BY"); want accepted. `select-variable-reuse`: rejected ("expression with a variable that is unbound"); want accepted.
- `xml-canon-test001`, `xml-canon-test002`, `rdf12-xml-an-13`, `rdf12-xml-an-14` (suite files): the only difference is the XMLLiteral value, `<br xmlns:rdf="..." xmlns:eg="..."></br>` against the expected `<br></br>`, and the rest of each graph, with reified triple terms, equals the expected one. `oxrdfxml` does not write the literal in exclusive canonical form.

Not controlled (unconfirmed, unchanged): the SPARQL and RDF/XML-independent syntax fails (`nested-aggregate-functions`, `tripleterm-subject-03`, `tripleterm-subject-06`), `ntriples-version-1`, and the JSON-LD toRdf fails.

Mark's F7 ruling (D65): SPARQL update evaluation, 97 tests, is "not run: no update stage" until P5 or P6.

## 2026-11-09: D56, mere-alembic into mere-athanor (branch `es/hostmerge`, worktree `C:/t/mere-es-hostmerge`)

Moved with history (git mv): alembic `workshop.rs` (193 lines), `workshop_surface.rs` (369), `workshop_native.rs` (120) to `ports/distillery/athanor/src/alembic/`; `lib.rs` (65), README and Cargo.toml dropped. Tests: 45 before (athanor 41, alembic 4), 45 after (`cargo test -p mere-athanor --all-features`: 45 passed). Checks: athanor `--all-targets` with no features, `workshop`, `workshop-surface`, `workshop-destination,workshop-sealed`, and all consumer features (`workshop-native,workshop-worker,workshop-sealed,workshop-destination`); graphshell `--all-targets --features alembic-workshop`. Lock diff: `mere-alembic` removed; athanor gains cambium, cambium-desktop, taproot; nothing else moved.

## 2026-11-09: P4 stage 1, catchment (branch `es/p4`, worktree `C:/t/mere-es-p4`)

Ruling F1 (Mark): "Generic over author". Moved with history (git mv) into `crates/system/catchment`:
`pandect/src/reservoir.rs` (701 lines) split into `catalog.rs` 175, `paths.rs` 119, `store.rs` 207 and
`store_tests.rs` 242, retiring its line-ceiling exception; `retained_text.rs` (353) split into
`retained_text.rs` 196 and `sealed.rs` 161; `shared_root.rs` 186; `tests/retained_text.rs` 183. New:
`lib.rs` 55, `tests/non_graph_domain.rs` 104 (a `music` mere and a Knot-style operation store opened
through the reservoir), `pandect/tests/reservoir_sessions.rs` 43 (the one reservoir test that needs
`MereSessions`). Pandect drops `dirs`, gains `catchment`, re-exports every old name and aliases
`ReservoirStore<B>` and `MereApplicationAccess` to `Author`. Consumer edits: none.

- `cargo tree -p catchment -e normal`: 135 packages; no `mere-kernel`, `pandect`, `incipit`, `tabard`,
  `hest`, `sceno` or Cambium crate. `chartulary` and `petgraph` arrive via `eidetic` and `sporran`.
- Lock: `catchment` added; pandect's entry gains `catchment` and drops `dirs 5.0.1`; nothing else moved.
- `cargo check --workspace --all-targets -j 4` (debug=0): green, 10m15s cold at `fe8c2d785`; green
  again in 1m14s after rebasing onto `62fc39ca6` (D56).
- Tests before: pandect 331 (299 unit, 32 integration). After: `cargo test -p catchment` 20 passed
  (14 unit, 2 non_graph_domain, 4 retained_text); `cargo test -p pandect` 313 passed, 1 ignored
  (284 unit, graph_residency 11, image_migration_failure 1, live_graph 9, reservoir_archive 2 + 1
  ignored, reservoir_sessions 1, resident_transaction 5). 331 kept or moved, 2 new.

## 2026-11-09: P4 D60, graph and session ids into catchment (branch `es/p4`, on Mere main `61636ef0f`)

Ruling F4 (Mark): "catchment". `GraphId` and `SessionId` (types, impls and the three id tests) moved
from `crates/incipit/src/lib.rs` (308 lines before, 220 after) to `crates/system/catchment/src/ids.rs`
(110). `incipit` gains `catchment` and `pub use catchment::{GraphId, SessionId}`. Pandect's
`incipit::{GraphId, SessionId}` imports repointed to catchment in `engine_profile_store`,
`graph_session`, `lib`, `manifest`, `manifest_store`, `session_graph_store` and
`session_service_runner`. Consumer edits: none (same names through the re-export).

- Lock: `incipit` gains `catchment`; nothing else moved.
- `cargo check --workspace --all-targets -j 4` (debug=0): green, 1m58s.
- Tests before: catchment 20, incipit 14, pandect 313 (+1 ignored). After (`cargo test -p catchment
  -p incipit -p pandect`): catchment 23 (17 unit, 2, 4), incipit 11, pandect 313 (+1 ignored).
- Pandect's remaining incipit uses: `live_view.rs` 15, 296; `view_intent_store.rs` 36, 398, 434, 439
  (curation folds, F3).
- First push refused: `5ef37f685` (eidetic row `features = ["product"]`, D50) landed first. Rebased
  onto it with no conflict (lock gains only `catchment` under incipit); workspace check green in
  1m56s and the same three crates' tests pass with the same counts.

## 2026-11-09: P4, catchment gated (D75 amended, "Gate catchment"; on Mere main `9cba35894`)

`catchment` default features: none (ids: serde, uuid `serde`+`v4`). `store` enables async-trait,
identity (personae), muniment (+redb, directory natively), eidetic, sporran, dirs, serde_json and
uuid `v5`; `non_graph_domain` and `retained_text` tests require it. pandect takes
`catchment = { workspace = true, features = ["store"] }`; incipit keeps the default. Lock unchanged.

- `cargo tree -p incipit -e normal --prefix none | sort -u`: 168 at `b67021f0c` (the coordinator's count); 12 now
  (catchment, cfg-if, getrandom 0.4.3, proc-macro2, quote, serde, serde_core, serde_derive, syn,
  unicode-ident, uuid, incipit); 11 with the catchment row removed, i.e. the pre-move set.
- `cargo tree -p catchment -e normal` (default): 11 packages, no eidetic, dramatis or crypto crate.
- `cargo package -p incipit --list`: ok. `cargo publish --dry-run --no-verify -p incipit`: fails,
  "no matching package named `catchment`" on crates.io. Same for catchment: fails, "`eidetic` with
  feature `product` but `eidetic` does not have that feature" (crates.io eidetic 0.0.3). Nothing
  published.
- `cargo check --workspace --all-targets`: green, 2m43s. Tests: `cargo test -p catchment` 3 (ids);
  `--features store` 17 + 2 + 4; `-p incipit` 11; `-p pandect` 313 (+1 ignored), unchanged.

## 2026-11-09: P4 D84, slot_guard into catchment (on Mere main `b5e679d90`)

`git mv crates/system/pandect/src/slot_guard.rs crates/system/catchment/src/slot_guard.rs` (170
lines, unchanged), behind `store`; `pandect::slot_guard` is `pub use catchment::slot_guard`, so
`ports/graphshell` (`block_editing/store.rs`, `presentation_store.rs`, `web_tree/forme.rs`,
`forme_session_lifecycle_tests.rs`) needs no edit. Lock unchanged.
- `cargo test -p catchment --features store`: 20 unit (17 + slot_guard's 3), 2, 4.
- `cargo check --workspace --all-targets`: green, 1m33s. `cargo test -p pandect`: 281 unit (284
  less slot_guard's 3) + 11 + 1 + 9 + 2 (+1 ignored) + 1 + 5 = 310.

## 2026-11-09: P4 D43/D77, the manifest's host fields become an extension slot

Ruling F2 (Mark): "Split by job". `GraphSessionManifest` (`pandect/src/manifest.rs`, 457 to 513
lines) replaces `active_workers`, `engine_profile` and `policy` with `extension:
serde_json::Map` (`#[serde(default, skip_serializing_if = "Map::is_empty")]`), `extension_get` and
`extension_set`. Serialization goes through `#[serde(remote = "Self")]` with hand-written impls; the
`Deserialize` folds the three legacy keys into the slot unless they hold their old defaults
(`[]`, `"PersonaScoped"`, `{"overrides":[]}`). Moved with history:
- `pandect/src/session_service_runner.rs` (340) to `ports/mere-resident/src/session_workers.rs`
  (434), with `WorkerKind`, `SessionPolicy`, `SessionPolicyOverride` from `manifest.rs` and
  `active_workers`/`set_active_workers`/`session_policy` over the slot.
- `pandect/src/engine_profile_store.rs` (339) to `crates/system/fetch/src/engine_profile.rs` (400),
  with `EngineProfileBinding` and `engine_profile_binding`/`set_engine_profile_binding`, behind a new
  default feature `engine-profiles = ["dep:pandect", "dep:serde"]`; fetch gains dev-deps
  `serde_json`, `uuid`.
- `PERSONAS_DIR` (sporran's) and `SESSIONS_DIR` now sit at pandect's root.

Consumers outside Mere naming any moved item or field: none (Turnstone, Cleromancy, Knot, Woodshed,
Graphshell checked). Lock: mere-fetch gains `uuid`.

- `cargo check --workspace --all-targets`: green, 1m14s; `cargo check -p mere-fetch
  --no-default-features`: green.
- Tests before / after: pandect unit 281 / 266 (less the runner's 8 and the store's 9, plus two
  shim tests), its integration tests unchanged (29 + 1 ignored); mere-fetch lib 66 / 76 (9 moved,
  one legacy-binding test); mere-resident lib 271 / 280 (8 moved, one extension test).
- mere-resident lib has two failures with and without this change, also at `fe8c2d785` before P4
  began: `resident_mere::tests::retained::nothing_is_kept_in_plaintext_when_no_custody_is_configured`
  (Windows os error 33 reading a locked `.redb`, `tests/retained.rs:244`) and
  `resident_mere::tests::the_dialable_host_serves_only_what_the_grant_and_the_owner_name`
  (`admitted: ActionNotOffered`, `resident_mere.rs:3109`). Not P4's; reported.





## 2026-11-09 (later): D54, subgraph and query_view into mere-view (branch `es/queryview`, worktree `C:/t/mere-es-hostmerge`)

Moved with history (git mv) into `crates/cambium/mere-view/src/`: `query.rs` (277) and `query/` (editor 248, editor/tests 154, projection 281, stroke 86, tests 573) = 1,619 lines; `subgraph.rs` (557) and `subgraph/` (classifier 265, query_tests 314, saved 159, tests 488) = 1,783 lines. Every file is under 600. Tests: 43 `#[test]` before and after in the moved files; at run time 42 under feature `query` (the 43rd is `cfg(not(feature = "query"))`). `cargo test -p mere-view --features query-editor`: lib 80 passed, `query_editor_host` 2 passed, `host_routing` 1 passed. `cargo test -p graphshell --features query-views --lib query_view`: 5 passed. Features: `subgraph` (incipit `arrangement`, serde), `query` (implies `subgraph`, linked-data/query), `query-editor` (implies `query`). Re-run after rebasing on cbf546caf: `--features subgraph --lib` 59 passed; `--features query-editor` lib 80, `query_editor_host` 2, `host_routing` 1. Checks: mere-view `--all-targets` with no features, `subgraph`, `query-editor`; graphshell `--all-targets --features query-views`; `cargo check --workspace --all-targets`. `--all-features` on mere-view fails on the `presentation_editor_host` test (`Snapshot` lacks `read_only`), which origin/main also fails. Lock diff: workspace-internal edges only (mere-view gains euclid, mere-rdf, serde; loses mere; graphshell gains mere-view).




## 2026-11-09: P4 D79, the settings theme stored opaque (on Mere main `8fb11dc0a`)

Ruling F5 (Mark): "Opaque theme, same bytes". New `pandect/src/stored_theme.rs` (85 lines):
`StoredTheme { theme_id, theme_mode: Option<String>, fonts: FontStacks }`, `FontStacks(Vec<(String,
Vec<String>)>)` serialized as a map in written order and skipped when every stack is empty.
`ApplicationSettings.theme` is `Option<StoredTheme>` (still flattened); `settings_store`'s legacy
migration builds one from the raw `theme_id`/`theme_mode`. pandect drops `tabard` (lock: pandect loses `tabard`, nothing else).

- Fixtures: generated by a throwaway test against the pre-D79 store (`ThemeChoice` with `Dark` and
  fonts `Code: [Iosevka, monospace]`, `Interface: [Inter]`; and `theme:light` with no mode), saved
  as `tests/fixtures/application_settings_theme_fonts.json` and `..._unmoded.json` (eol=lf in
  `.gitattributes`). `the_theme_keeps_the_bytes_tabards_choice_wrote` parses each and asserts
  `to_string_pretty` returns the fixture byte for byte.
- `cargo check --workspace --all-targets`: green, 1m28s. `cargo test -p pandect`: unit 266 to 267
  (one new test), integration unchanged.


## 2026-11-09: P4 D44/D78/D86, view state opaque in pandect; ViewIntent to incipit; live_view to the host

Rulings: F3 "incipit `view` feature" (D78), F7 "graphshell-host feature" (D86). Moved with history:
- `pandect/src/view_intent_store.rs` (602) to `crates/incipit/src/view.rs` (266) and
  `view/tests.rs` (348, 17 tests), behind `view = ["dep:serde_json", "dep:sceno", "dep:hest",
  "dep:muniment"]`; adds `VIEW_INTENT_SCHEMA = "mere.view-intent/v1"`; `kernel` dev-dependency for
  `relation_tag_round_trips_through_disk`. Its line-ceiling exception is removed.
- `pandect/src/live_view.rs` (431) to `crates/graphshell/graphshell-host/src/live_view.rs` (434, 3
  tests), behind `live-view = ["dep:incipit", "incipit/view", "dep:eidetic", "dep:serde"]`; schema id
  `mere.live-view/v1` unchanged.
- New `pandect/src/view_state.rs` (98, 3 tests). `ViewEntry.state`, `views`, `view`, `view_at`,
  `set_view`, `set_view_now`, `transact_view` and the codicil's view record take `ViewState`.
  pandect drops `incipit`, `sceno` and `hest`.
- Consumers: mere-overmap (`wire.rs`, `route_session.rs`, `operations.rs`), graphshell-mere
  (`connection_tests.rs`), graphshell port (`mere_host.rs` gains `encode_view`; `MereHost::view`
  returns an owned decoded `ViewIntent`; `mere_host_resident.rs`), mere-resident
  (`resident_mere.rs`); each takes `incipit` with `view`.

Gates: `cargo check --workspace --all-targets` green (1m40s); `cargo check -p graphshell --features
web --all-targets` green (3m45s). Tests before / after: pandect unit 267 / 250 (17 and 3 moved out,
3 new), integration unchanged; incipit 11 / 28 with `view`; graphshell-host lib 24 / 27 with
`live-view`; mere-overmap 17; graphshell-mere 46; graphshell `--features web --lib mere_host` 25;
mere-resident lib 280 with the same two pre-existing failures as at `fe8c2d785`.
`cargo tree -p pandect -e normal`: 158 packages; no incipit, tabard, sceno; hest via mere-kernel only.
- `ports/graphshell/web` (its own workspace): `cargo check --target wasm32-unknown-unknown
  --features product` first failed at `src/web_tree/product.rs:942` (`.cloned()` on the now-owned
  `MereHost::view` result); fixed, then green with `--all-targets`. Its lock takes only the edge
  changes (incipit gains hest, muniment, sceno; pandect loses hest, incipit, sceno, tabard).

## 2026-11-09: P4 D45/D47 preparation, the retention half made self-contained (no Eidetic edits)

New `crates/system/pandect/src/retention/`: `mod.rs` 29, `levels.rs` 279 (5 tests: 3 generic over a
plain record, the policy default and ladder moved from `memory_levels`), `forgetting.rs` 94
(`ForgetProposal`, `apply_forgetting`, one test moved from `furnace`), `retirement.rs` 134
(`RetirementProposal`, `propose_retirement`, `apply_retirement`, two tests moved from `furnace`).
Imports: `std`, `serde`, `eidetic` (`content_store`, `DeletedNode`, `purge_deleted`, `Result`,
`Store`); tests use `muniment::MemoryBackend`. No `kernel`, no `crate::`. `memory_levels.rs` 404 to
294 (graph adapter: `GraphNodeMemory` implements `MemoryRecord`; `is_promoted`, `level_of`,
`evictable_short_term`, `census` keep their signatures); `furnace.rs` 842 to 659, re-exporting the
moved names, so Cleromancy's `memory_levels::level_of` and Turnstone's `furnace::*_retirement` are
unchanged. D47: `image_store.rs` already calls `eidetic::image_content::{save_image_bytes,
load_image_bytes, delete_image_bytes, stored_image_hexes}`; nothing graph-free is left to move.

- Tests: pandect `--features furnace` unit 261 to 264; default unit 250 to 256 (retention's 8
  compile without `furnace`); integration unchanged.
- `cargo check --workspace --all-targets`: green, 1m23s.

## 2026-11-09: P4 D45/D87, retention moved to eidetic::memory

Eidetic 8e903bf (ff-pushed to main): `crates/eidetic-core/src/memory/{mod,levels,forgetting,retirement}.rs`,
copied from pandect's `retention/` with `eidetic::` paths rewritten to `crate::`; feature
`memory = ["content_store", "deleted"]`, added to `product`; README features and module rows.
`eidetic::retention` untouched. Eidetic hooks (line ceiling, clippy) passed.
- Eidetic: `cargo test -p eidetic` default 182 lib plus integration green (8 `memory::` tests among them);
  `--no-default-features --features memory --lib` 99 passed, the 8 `memory::` tests included.

Mere: pandect's `retention/` deleted (mod 29, levels 279, forgetting 94, retirement 134);
`lib.rs` has `pub use eidetic::memory as retention` (public path kept); `memory_levels` and
`furnace` re-export from `eidetic::memory`. Mere's eidetic row already carries `product`, so no
feature edit. Relock by revision substitution only: 12 eidetic.git rows in `Cargo.lock` and 8 in
`ports/graphshell/web/Cargo.lock`, 76481186 to 8e903bf1; `cargo metadata --locked` passes on both.
- `cargo check --workspace --all-targets --locked`: green.
- `cargo test -p pandect --features furnace --locked`: unit 264 to 256; default unit 256 to 248
  (the 8 moved tests leave); integration unchanged.
- `ports/graphshell/web`: `cargo check --target wasm32-unknown-unknown --locked`: green.

## 2026-11-12: the layout test per phase (P3 condition 3, P1 and P2 back-filled)

Read-only. Each lock read with `git show <rev>:<lock>` and parsed with Python's `tomllib`; for each
lock, the family git sources (`merely-made`, and the `mark-ik` forks) with their query and revision
and the package count from each, and the package names present from two or more sources where one is
a `merely-made` source. No build, no checkout. A repository counts toward the repin figure when its pin
or lock must move for the change to reach Turnstone; the owner is not counted. A family source held
once in Turnstone's lock, with `branch = "main"` and no `rev` row for it anywhere in its graph (no
`merely-made` `rev =` row in Turnstone's manifests at `c9efade0`), moves with one relock of Turnstone.

| Phase | Lock | Owner's source in it | Names from two sources (family) |
|---|---|---|---|
| before P1 | Turnstone `841c8117~1` | mere.git `28a0be22`, 53 packages (Inker still in Mere) | 3 (`servo-media-{player,streams,traits}`: Genet beside Servo git) |
| P1 | Turnstone `841c8117` | inker.git `436c8413`, 12 | 3, the same |
| P1 | Turnstone `6c4d278b` | inker.git `436c8413`, 11 | 3, the same |
| P1 | Knot `a58ba28` | inker.git `436c8413`, 8 | 0 |
| P2 | Turnstone `f3a455d3` (first lock with seiche.git) | seiche.git `f0e18dd9`, 3 | 3, the same |
| P2 | Turnstone `c24b5e0d`, `b12348d9` | seiche.git `f0e18dd9`, 3 | 3, the same |
| P2 | Knot `189487c1` | seiche.git `f0e18dd9`, 3 | 0 |
| before P3 | Turnstone `ea971f17` | mere.git `a9153fa6`, 33 (toolkit still in Mere) | 5 (`regress`, `servo-media-*` x3, `servo-xpath`: Genet beside crates.io) |
| P3 | Turnstone `c9efade0` | graphshell.git `aed944eb`, 47; mere.git `a9a125f2`, 5; emblem `e577e849`, 2; distillery `8908f7e4`, 4 | 5, the same names |
| before P3 | Knot `38ea372` | mere.git `3494d980`, 35 | 15 (crates.io beside family git) |
| P3 | Knot `ef69aef` | graphshell.git `893818f6`, 36 | 15, the same names |
| P3 | Graphshell `host/Cargo.lock` at `ed2dc6e6` | mere.git `325b0083`, 5 (contract root by path) | 1 (`regress`) |

At P1 and P2 both Turnstone's and Knot's locks also hold `netrender-vello` and `netrender-vello-encoding`
at two revisions of the `mark-ik/vello` fork (`77846139`, `dfb9b53b`); the P3 locks hold one. Every
owner source above is held once per lock, so each phase's figure is 1 (Turnstone). The family cycle
checker, `python scripts/check_family_cycles.py --remote`, run once the same day: exit 0, 35
repositories, 94 lock roots, 1,303 cross-repository manifest edges, resolved-cycle warnings inside 12
locks; heads graphshell `ed2dc6e68`, mere `08810f5d6`, turnstone `c9efade0e`, knot-editor `ef69aef0d`.

The line-ceiling recount for U6: `line-ceiling.toml` at `325b0083b` holds 151 exceptions, of which 133
name a path absent at the cutover `299d19b63` and 18 a path present; `299d19b63` and `08810f5d6` hold
those 18, none missing. At `0c4f551d6`, where U6's line was written, the file held 150, of which 132
name a path absent at `299d19b63`; U1 (`ea049e09a`) then replaced four pictograph rows with five
`mere-cartography` rows (the four moved files and `backdrop_paint.rs`).

## 2026-11-12: P5 baseline, pre-fold (Mere `0fad5b831` plus the instrument)

P0 recorded no performance baseline, so this run is the baseline P5's "measured against the P0 baseline" is met by (D141): the commit before the fold. The instrument is `crates/graph/linked-data/examples/adapter_scale.rs`, extended to time per-edit assert and withdraw, the refresh after one edit, and the engine's share of a frame; no engine code changed (Mere main's later `d12080e63` is docs only). Command, five times in sequence: `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_RELEASE_DEBUG=0 cargo build -j 4 --release -p mere-rdf --features query --example adapter_scale`, then `target/release/examples/adapter_scale.exe 10000 100000`. Machine: AMD Ryzen 9 7940HS, 16 threads, 31 GB, Windows 11 (10.0.26320); rustc 1.100.0 (0ef1bc386 2026-11-09), fresh resolve (no lock tracked). Machine state: not quiet. Two other lanes were building; total CPU before each run was 51, 37, 54, 46 and 25% (three one-second samples averaged), and 4, 7, 9, 4 and 2 cargo, rustc or link processes were alive after runs 1 to 5. The instrument is single-threaded, so the contention is memory and clock, not cores; the min..max columns show its size (about plus or minus 20% on the 10k rebuild, up to 2.7 times at the worst 100k rebuild). Every held-query row count agreed with the `sparql()` and materialized columns in every cell, and the instrument asserts `is_current` is false after an edit and true after the rebuild, and that the held point lookup and two-hop join return one row after every edit.

What an edit is: `assert_semantic_statement` of a statement with a predicate no edge carries yet, on a node pair with no statement (so it changes the graph; a repeat of a withdrawn statement does not, by the no-revival rule); withdraw is `withdraw_semantic_assertion` of the same handle. Refresh is `is_current` (stale), `IndexedDataset::for_graph`, `is_current` (fresh). A frame is `is_current` plus the coverage note plus one held query (steady), or `is_current` then rebuild then the query after one assert. All times in ms; each cell below is one run's median over its reps.

### Edit, refresh and frame (median of per-run medians, five runs)

| scale | measure | reps per run | run 1 | run 2 | run 3 | run 4 | run 5 | median | min..max |
|---|---|---|---|---|---|---|---|---|---|
| 10000 | edit_assert | 51 | 0.0037 | 0.0038 | 0.0042 | 0.004 | 0.0035 | 0.0038 | 0.0035..0.0042 |
| 10000 | edit_withdraw | 51 | 0.1452 | 0.096 | 0.1322 | 0.1169 | 0.0869 | 0.1169 | 0.0869..0.1452 |
| 10000 | is_current_stale_after_assert | 21 | 0 | 0 | 0 | 0 | 0 | 0 | 0..0 |
| 10000 | rebuild_after_assert | 21 | 276.686 | 227.222 | 242.83 | 265.219 | 206.19 | 242.83 | 206.19..276.686 |
| 10000 | is_current_fresh_after_assert | 21 | 0.0004 | 0.0004 | 0.0005 | 0.0003 | 0.0004 | 0.0004 | 0.0003..0.0005 |
| 10000 | refresh_total_after_assert | 21 | 276.687 | 227.223 | 242.831 | 265.219 | 206.19 | 242.831 | 206.19..276.687 |
| 10000 | is_current_stale_after_withdraw | 21 | 0.0002 | 0.0001 | 0.0001 | 0.0001 | 0.0001 | 0.0001 | 0.0001..0.0002 |
| 10000 | rebuild_after_withdraw | 21 | 283.668 | 227.161 | 232.56 | 255.939 | 213.524 | 232.56 | 213.524..283.668 |
| 10000 | is_current_fresh_after_withdraw | 21 | 0.0004 | 0.0004 | 0.0004 | 0.0004 | 0.0004 | 0.0004 | 0.0004..0.0004 |
| 10000 | refresh_total_after_withdraw | 21 | 283.668 | 227.161 | 232.561 | 255.939 | 213.524 | 232.561 | 213.524..283.668 |
| 10000 | frame_steady_point | 51 | 0.0112 | 0.0101 | 0.0117 | 0.011 | 0.0102 | 0.011 | 0.0101..0.0117 |
| 10000 | frame_after_edit_point | 21 | 282.338 | 259.682 | 228.806 | 258.109 | 209.523 | 258.109 | 209.523..282.338 |
| 10000 | frame_steady_two_hop | 51 | 6.5281 | 5.8487 | 5.2612 | 5.8267 | 5.0472 | 5.8267 | 5.0472..6.5281 |
| 10000 | frame_after_edit_two_hop | 21 | 303.432 | 284.252 | 229.884 | 262.404 | 210.29 | 262.404 | 210.29..303.432 |
| 100000 | edit_assert | 51 | 0.0065 | 0.01 | 0.0209 | 0.0039 | 0.0183 | 0.01 | 0.0039..0.0209 |
| 100000 | edit_withdraw | 51 | 6.8534 | 7.3068 | 5.0356 | 4.8686 | 4.8598 | 5.0356 | 4.8598..7.3068 |
| 100000 | is_current_stale_after_assert | 7 | 0.0001 | 0 | 0 | 0 | 0 | 0 | 0..0.0001 |
| 100000 | rebuild_after_assert | 7 | 3483.1 | 4622.48 | 2606.93 | 2446 | 2286.58 | 2606.93 | 2286.58..4622.48 |
| 100000 | is_current_fresh_after_assert | 7 | 0.0004 | 0.0004 | 0.0005 | 0.0004 | 0.0004 | 0.0004 | 0.0004..0.0005 |
| 100000 | refresh_total_after_assert | 7 | 3483.11 | 4622.48 | 2606.93 | 2446 | 2286.58 | 2606.93 | 2286.58..4622.48 |
| 100000 | is_current_stale_after_withdraw | 7 | 0.0002 | 0.0002 | 0.0002 | 0.0001 | 0.0001 | 0.0002 | 0.0001..0.0002 |
| 100000 | rebuild_after_withdraw | 7 | 3640.72 | 6329.03 | 2598.89 | 2265.89 | 2663.86 | 2663.86 | 2265.89..6329.03 |
| 100000 | is_current_fresh_after_withdraw | 7 | 0.0003 | 0.0004 | 0.0006 | 0.0003 | 0.0004 | 0.0004 | 0.0003..0.0006 |
| 100000 | refresh_total_after_withdraw | 7 | 3640.72 | 6329.03 | 2598.89 | 2265.89 | 2663.86 | 2663.86 | 2265.89..6329.03 |
| 100000 | frame_steady_point | 51 | 0.017 | 0.0191 | 0.0169 | 0.0098 | 0.0149 | 0.0169 | 0.0098..0.0191 |
| 100000 | frame_after_edit_point | 7 | 3419.61 | 6128.85 | 2669.13 | 2251.16 | 2418.06 | 2669.13 | 2251.16..6128.85 |
| 100000 | frame_steady_two_hop | 51 | 74.1903 | 126.437 | 57.0916 | 48.9403 | 71.8496 | 71.8496 | 48.9403..126.437 |
| 100000 | frame_after_edit_two_hop | 7 | 3069.96 | 6144.17 | 2866.72 | 2378.69 | 2440.63 | 2866.72 | 2378.69..6144.17 |

### Load and queries (median over five runs, min..max in brackets)

| scale | quads | graph build ms | index build ms | index bytes | materialize ms |
|---|---|---|---|---|---|
| 10000 | 80,000 | 185.92 (158.41..219.17) | 240.74 (200.87..252.67) | 3,840,168 | 621.71 (542.68..729.99) |
| 100000 | 800,000 | 2314.37 (1666.98..2803.41) | 3288.49 (1986.07..3736.93) | 38,400,168 | 8554.08 (6491.9..10784.6) |

| scale | query | `sparql()` per call ms | held `IndexedDataset` ms | materialized `oxrdf::Dataset` ms |
|---|---|---|---|---|
| 10000 | bound predicate | 250.46 (224.42..293.05) | 1.02 (0.82..1.38) | 3.8 (3.38..4.75) |
| 10000 | point lookup | 256.05 (226.22..293.16) | 0.08 (0.08..0.1) | 0.05 (0.04..0.06) |
| 10000 | two-hop join | 268.68 (213.24..314.49) | 6.4 (5.06..8.82) | 17.95 (16.24..21.81) |
| 10000 | full scan | 267.63 (214.73..278.91) | 5.97 (5.76..6.93) | 27.87 (26.58..32.71) |
| 100000 | bound predicate | 3056.27 (2259.24..3614.36) | 10.37 (7.07..11.85) | 45.94 (34.41..53.94) |
| 100000 | point lookup | 3271.83 (2271.79..4915.53) | 0.2 (0.18..0.24) | 0.05 (0.04..0.06) |
| 100000 | two-hop join | 3162.47 (2290.67..4698.08) | 53.7 (52.04..60.78) | 190.72 (175.58..257.36) |
| 100000 | full scan | 3141.47 (2338.57..4776.53) | 63.07 (59.7..74.94) | 345.69 (313.89..421.97) |

The 100k withdraw (5.0 ms, against 0.12 ms at 10k) is a kernel cost found by this run and not investigated here. The index build column is the first build after the graph; the warm rebuilds in the edit table are the refresh figures.

## 2026-11-12: P5 L3, the model seam (Mere `b36e9abf8` fixtures, `d2a6398c1` move)

Instrument: `crates/graph/graph-kernel/examples/join_scale.rs` (added with the move; the same source
ran from `crates/graph/engine/examples/` at the pre-move commit `b36e9abf8`). Each surface shows its own
resource and the resources form a `cites` ring. Release, debug info off, Windows 11, Ryzen 9 7940HS.
Each cell is the median of seven reps in one run, in milliseconds. Runs alternated: before 1, after 1,
before 2 (a throwaway worktree at `b36e9abf8`, deleted after), after 2. Before 1 ran while the machine
was busier; read run 2 against run 2.

| scale | phase | before run 1 | before run 2 | after run 1 | after run 2 |
|---|---|---|---|---|---|
| 10,000 | shown_resource_id+resource | 1.208 | 0.914 | 1.101 | 1.008 |
| 10,000 | surface_ids_showing_resource | 2.638 | 1.783 | 2.115 | 1.840 |
| 10,000 | resource_edges held | 0.058 | 0.036 | 0.047 | 0.038 |
| 10,000 | dataset_quads | 226.514 | 180.644 | 169.736 | 183.562 |
| 10,000 | for_graph | 162.209 | 129.190 | 129.573 | 139.013 |
| 10,000 | edit+is_current+rebuild | 159.716 | 132.022 | 121.927 | 133.713 |
| 100,000 | shown_resource_id+resource | 41.842 | 32.206 | 35.239 | 34.895 |
| 100,000 | surface_ids_showing_resource | 46.012 | 40.614 | 35.855 | 34.734 |
| 100,000 | resource_edges held | 2.813 | 2.715 | 2.118 | 2.037 |
| 100,000 | dataset_quads | 2966.216 | 2397.216 | 2267.295 | 2257.717 |
| 100,000 | for_graph | 1754.568 | 1415.989 | 1259.590 | 1413.587 |
| 100,000 | edit+is_current+rebuild | 1763.557 | 1350.096 | 1325.361 | 1352.418 |

Build (one pass each): `add_node` with resource binding 78.6 / 54.9 / 56.9 / 66.4 ms at 10k and
852.0 / 706.9 / 712.4 / 730.4 ms at 100k; resource asserts 32.8 / 29.5 / 36.0 / 36.4 ms and
489.9 / 404.6 / 374.2 / 404.2 ms (before 1, before 2, after 1, after 2). Counts agreed in every cell
(10k: 10,000 joins, 50,000 quads; 100k: 100,000 and 500,000).

Tests, by name: before, `mere` lib 93 and `mere-kernel` lib 537 (630); after, `mere` lib 14 and
`mere-kernel` lib 616 (630, `--features query,bundled-contexts`), the same 630 names. 90 engine tests
that build a kernel `Graph` moved to `kernel::rdf`'s test modules; 11 kernel model tests (edge data,
persisted statements and properties, minting, predicate strata) moved to `mere::model`. Integration
targets unchanged: `wire_fixtures` 5 + 1 ignored writer, `assertion_lifecycle_reopen` 1 + 1,
`frozen_link_reopen` 1 + 1, `w3c_conformance` 2 + 1, `mere-rdf` 1.

Ledger: `W3C_SUITES_DIR=C:/Users/mark_/Code/crates cargo test -p mere --features query --test
w3c_conformance -- --ignored --nocapture`, 44 s; all 49 `ROW` lines identical to L2's run.

Downstream, each from a throwaway worktree at origin/main (Graphshell `e919ae77`, Turnstone `0a4636ae`,
Cleromancy `30d3340`, Knot `090237c`), with `--config` naming a patch file that points `mere`,
`mere-kernel`, `mere-rdf`, `pandect`, `catchment` and `mere-facts` at the branch; every build showed the
patched packages from `C:/t/p5l3`. Graphshell `cargo check --manifest-path host/Cargo.toml --workspace
--all-targets`: first failed on pandect's `personal-sync` path (fixed in `0578ddadf`), then passed in
3m50s; `-p mere-view --features mere-view/query` passed. Turnstone `cargo check --workspace --all-targets`
(default features, CEF) passed in 9m07s. Knot `cargo check --workspace --all-targets` passed in 4m14s.
Cleromancy: `authority-guest` (its own workspace) passed; the root crate and `core --all-targets` fail,
and the same check without the patch (resolving Mere main `cc780c8d`) fails with the identical error set
(`SessionViewIntent.focus`, `SessionGraphV1.attention`, `DecoderDevice::ndarray`; `core`'s
`authority_corpus` test missing `mere_resident`, `eidetic`, `ostiary`, `uuid`); `core`'s library,
binaries and examples pass with the patch.

## 2026-11-12: P5 L4, format entry points and the engine layer (Mere `65f27a4a4` entry points, `d3ed0ece8` harness)

Ledger: `W3C_SUITES_DIR=C:/Users/mark_/Code/crates cargo test -p mere --features query --test
w3c_conformance -- --include-ignored --nocapture`, 3 passed in 24 s (suites `rdf-tests` `202a74e`,
`json-ld-api` `5551473`; the same as the ledger's starting point). 70 `ROW` lines: the 49 parser, adapter and
indexed lines equal L3's (compared with the counts recorded in the ledger and the P5 assessment: rdf12 syntax
parser 181/1/0 and adapter 69/0/113, eval 85/2/0; rdf11 syntax 579/0/0 and 286/15/278, eval 412/2/0; SPARQL
1.1 query-eval parser and indexed 230/14/3; SPARQL 1.2 64/1/0 and syntax 200/3/0; toRdf parser 439/10/18 and
adapter 441/1/25), plus 21 `engine` lines:

| ROW | pass | fail | skip |
|---|---|---|---|
| rdf12 syntax engine | 181 | 1 | 0 |
| rdf12 eval engine | 85 | 2 | 0 |
| rdf12 c14n engine | 0 | 0 | 82 |
| rdf12 entailment engine | 0 | 0 | 29 |
| rdf11 syntax engine | 579 | 0 | 0 |
| rdf11 eval engine | 412 | 2 | 0 |
| rdf11 entailment engine | 0 | 0 | 48 |
| sparql11-query query-eval engine | 230 | 14 | 3 |
| sparql11-query syntax engine | 0 | 0 | 106 |
| sparql11-update syntax engine | 0 | 0 | 63 |
| sparql11-update update-eval engine | 0 | 0 | 94 |
| sparql12 query-eval engine | 64 | 1 | 0 |
| sparql12 syntax engine | 0 | 0 | 203 |
| sparql12 update-eval engine | 0 | 0 | 3 |
| jsonld11 toRdf engine | 432 | 10 | 25 |
| jsonld11 compact, expand, flatten, fromRdf, html, remote-doc engine | 0 | 0 | 246, 386, 58, 54, 50, 18 |

Leaves that moved from the adapter layer: rdf11 TriG syntax 199/15/0 (adapter) to 214/0/0 (engine), TriG
evaluation 143/143, Turtle 168/168 and 145/145, N-Triples 70/70, RDF/XML syntax 40/40 and evaluation 124/126;
rdf12 N-Triples 33/34, Turtle 77/77 and 32/32, TriG 26/26 evaluation, RDF/XML 2/2 and 27/29. The verbose fail
lists of the parser and engine layers were diffed by test id: the engine's 30 fails are a subset of the
parser's 33 (the other 3 are SPARQL 1.2 syntax tests the engine layer does not score), so no case passes at
the parser layer and fails at the engine layer.

Tests: `cargo test -p mere` 14 lib tests (as L3), 19 with `--features query` (five new, in
`formats::parse`); harness 2 passed and 1 ignored with `query`, 1 and 1 without; `cargo test -p mere-kernel
--features query,bundled-contexts` lib 616 (as L3), `wire_fixtures` 5 + 1 ignored, `assertion_lifecycle_reopen`
1 + 1, `frozen_link_reopen` 1 + 1. `cargo check -p mere --target wasm32-unknown-unknown` passes with and
without `--features query`; `cargo check --workspace --all-targets` passes (1m20s); the pre-commit hooks pass
(clippy including `--all-targets` with `mere/query`), and `house doc-audit` and `house hooks --check` exit 0.

Downstream: Graphshell at origin/main `e919ae77`, a throwaway worktree (removed after), `cargo check -j 4
--config <patch file> -p mere-view --features mere-view/query` from `host/`, the patch file pointing `mere`,
`mere-kernel` and `mere-rdf` at the branch: passed in 1m08s, with no unused-patch warning for those three.
Callers of the entry points outside Mere: none (Graphshell, Turnstone, Cleromancy and Knot, `git grep` at
origin/main; Turnstone's one hit is a doc comment naming `from_jsonld`).

## 2026-11-12: P5 L5, the re-export window closes (Mere ungating `2b45c839a`, Graphshell `a377c352`)

Step 1, Mere `2b45c839a` (on main before Graphshell moved): `cargo test -p mere` 18 lib tests without `query`
(14 before: the four `formats::parse` tests that need no evaluator now run, the fifth keeps its `query` gate)
and 19 with; harness 1 passed and 1 ignored without `query`, 2 and 1 with; `cargo tree -p mere -e normal`
with no features: 0 lines naming `spareval`, `spargebra` or `oxsdatatypes`; `cargo check -p mere --target
wasm32-unknown-unknown` (RUSTFLAGS `--cfg getrandom_backend="wasm_js"`) passes with and without `query`;
`cargo check --workspace --all-targets` passes; `cargo test -p mere-kernel --features query,bundled-contexts`
lib 616 (as L4). Pre-commit hooks pass (clippy, bans, resolved record).

Graphshell `a377c352` (origin/main `e919ae77` before): from `host/`, `cargo check -p mere-view --features
mere-view/query` passes against Mere `2b45c839a`, one `mere` in the graph; `cargo check --workspace
--all-targets` passes (6m54s). The commit hook `cargo deny check bans` failed with a duplicate `dirs` (6.0.0
from `pelt-desktop`, 7.0.0 from `catchment`); a detached worktree at `e919ae77` fails the same way, so it is
not from this change. `pelt-desktop` moved to `dirs` 7 (its two calls, `config_dir` and `data_local_dir`, are
unchanged), `cargo deny check bans` passes, `cargo check -p pelt-desktop --features livery` passes, and the
hooks pass. Pushed to main.

Step 4, the shim deleted (`crates/graph/linked-data` and its workspace row; clippy hook now `-p mere -p
mere-kernel`, regenerated by `house hooks`; stale `mere-rdf` names in the harness comments and the
`getrandom` skip reasons fixed): `cargo check --workspace --all-targets` passes, and with `--all-features`
passes; wasm32 `cargo check -p mere` passes with and without `query`; `house hooks --check` exits 0.
Ledger: `W3C_SUITES_DIR=C:/Users/mark_/Code/crates cargo test -p mere --features query --test
w3c_conformance -- --include-ignored --nocapture`, 3 passed in 9 s: 70 `ROW` lines, each count equal to the
L4 table above (all 21 `engine` rows and the 49 parser, adapter and indexed rows); no layer lived only in the
shim.

Consumers: `git grep -E 'mere-rdf|linked-data|linked_data'` over `*.toml` at origin/main of Turnstone,
Cleromancy and Knot (`knot-editor`): Cleromancy and Knot none; Turnstone's `deny.toml` line 183 names `mere-rdf`
in a skip reason, a prose string, not a dependency. A sweep of every repo under `Code/repos` found the name
only in Graphshell (the row just repointed, and its `deny.toml`), in mer3ly's `content/specs.toml` prose, and
in Turnstone's `deny.toml`.

After both pushes: Graphshell's `cargo check -p mere-view --features mere-view/query` from `host/` with a fresh
resolve passes against Mere `f7ffaf16a` (one `mere` in the graph). `house cycles --family-dir
C:/Users/mark_/Code/repos --remote` exits 0 (35 repositories, 1,315 manifest edges, no manifest cycle; the
resolved-cycle lines are warnings, as before), and `house doc-audit` exits 0.

## 2026-11-12: P5 L6a, the headless resource inspector

`cargo run -p mere --example inspect --features query -- crates/graph/engine/examples/fixtures/history.trig
--query 'SELECT ?s ?o WHERE { GRAPH ?g { ?s <https://mere.computer/ns/rel#cites> ?o } } ORDER BY ?s ?o'`:

```
dataset: 43 rows, 64 terms

resources (2):
  https://example.test/alice
    https://mere.computer/ns/rel#cites https://example.test/bob [https://example.test/graph/notes]
    https://mere.computer/ns/rel#cites https://example.test/carol [https://example.test/graph/notes]
  https://example.test/bob
    https://mere.computer/ns/rel#cites https://example.test/carol [https://example.test/graph/imports]

asserters (2):
  urn:author:ada: 2 held, 0 withdrawn
  urn:author:grace: 1 held, 1 withdrawn

assertions (4):
  urn:author:ada  held  in https://example.test/graph/notes  urn:mere:statement:fx:one  reason: urn:method:manual-review v1.0; "read the paper"
  urn:author:ada  held  in https://example.test/graph/notes  urn:mere:statement:fx:two  reason: (none)
  urn:author:grace  held  in https://example.test/graph/imports  urn:mere:statement:fx:three  reason: urn:method:extractor v1.0
  urn:author:grace  withdrawn  in https://mere.computer/ns/graph#history  urn:mere:statement:fx:four  reason: urn:method:extractor v1.0; "retracted after recheck"

named graphs (4):
  https://example.test/graph/imports: 7 quads
  https://example.test/graph/notes: 13 quads
  https://mere.computer/ns/graph#history: 13 quads
  https://mere.computer/ns/graph#vocabulary: 10 quads

reason keys (2):
  urn:method:extractor: 2 assertions
  urn:method:manual-review: 1 assertions

query (3 rows): s | o
  https://example.test/alice | https://example.test/bob
  https://example.test/alice | https://example.test/carol
  https://example.test/bob | https://example.test/carol
```

Fixture: `to_trig_with_history` output of a four-statement kernel graph, generated once by a throwaway test in
`mere-kernel` (not committed) modelled on `explicit_history_roundtrips_lifecycle_basis_and_original_scope`.
`cargo test -p mere --features query`: lib 19, `inspect_example` 2, `w3c_conformance` 2 passed (1 ignored);
`cargo test -p mere` without `query`: lib 18 passed, the inspector test compiled out. wasm32 `cargo check -p
mere` passes with and without `query`; `cargo check --workspace --all-targets` passes.

## 2026-11-12: P5 L6b, content negotiation and Link headers

D142 row "Access: content negotiation and Link headers". Pure functions in `mere::acquire::negotiate`; no network, no HTTP client.

- `formats::RdfFormat` is `oxrdfio`'s enum (new normal dependency, `0.2`, `default-features = false`, `rdf-12`; resolved 0.2.7), re-exported; it covers Turtle, TriG, N-Triples, N-Quads, JSON-LD, RDF/XML and N3. `formats::parse::parse(format, bytes, base_iri, contexts)` dispatches to the `parse_*` entry points; N3 is an error.
- `negotiate`: `PARSED_FORMATS`, `AcceptPolicy` (formats and q-values as data; default Turtle and JSON-LD 1, TriG and N-Quads 0.9, N-Triples 0.8, RDF/XML 0.5; `narrowed_to`, `header_value`), `format_for_media_type`, `parse_content_type`, `parse_response`. `oxrdfio::from_media_type` is lenient (`application/json`, `text/plain`, `application/xml` map to JSON-LD, N-Triples, RDF/XML), so `format_for_media_type` refuses those three and N3: a dereferencer does not guess, and a bare `application/json` body needs a Link context. A non-UTF-8 charset is an unsupported type.
- `negotiate::link`: hand-written RFC 8288 parser (`parse_links`, `Link`) and `discover` (alternate with an RDF type, describedby, the JSON-LD context relation), targets resolved with `oxiri` against the response URL.
- Link parser: nothing in the tree fits, and the one widely used crate checked (`parse_link_header`, regex-based, keyed by `rel`) would add dependencies and lose repeated or multi-valued relations.
  The RFC 8288 section 3.5 examples are the tests.
- The inspector example picks its format with `RdfFormat::from_extension` and the dispatch; its test's unknown-extension case moved from `txt` (an oxrdfio alias for N-Triples) to `xyz`.

Gates: `cargo test -p mere` lib 27 passed; `--features query` lib 28, `inspect_example` 2, `w3c_conformance` 2 passed (1 ignored). Tests: Accept output, Content-Type dispatch for every format with parameters, the RFC examples, and Turtle body plus JSON-LD Link alternate end to end. wasm32 `cargo check -p mere` (with `--cfg getrandom_backend="wasm_js"`) passes with and without `query`; `cargo tree -p mere -e normal` shows no reqwest, ureq or hyper; `cargo check --workspace --all-targets` passes.

## 2026-11-12: P6 L1, the dataset reader and the JSON-LD processing mode (Mere `18178e5a0`, `1dcd5b49d`)

D143 F-R1 and F-R2. Spec and metric per item, as Mark asked (2026-11-12).

F-R1, `IndexedDataset::quads(View) -> impl Iterator<Item = oxrdf::Quad>` (`dataset.rs`; a full-scan `QuadIndex::pattern`, no `spareval`).
Ungated by it: the harness's RDF and JSON-LD engine layer (`dataset_of` reads through `quads(View::Live)`; `layers()` always lists `engine`) and the inspector (reifier, graph and resource sections read the live quads into a per-graph property map; `--query` is `cfg(feature = "query")` and an error without it; `required-features` dropped).
Metrics:
- Harness builds and runs its RDF and JSON-LD rows without `query`: yes, `cargo test -p mere --test w3c_conformance -- --ignored --nocapture`, 1.8 s.
- With `query`, every ROW line identical to before: `diff` of the baseline run (origin/main, stashed) against the F-R1 run shows only the timing line.
- Without `query` the RDF and JSON-LD ROW lines equal the `query` run's: `diff` empty.
- `cargo tree -p mere -e normal`: 0 lines naming `spareval` or `spargebra`.
- New tests: `dataset::tests::quads_round_trip_every_graph_kind_and_honour_the_view` (default, named, blank graph, a withdrawn row in both views), `parsed_rows_are_held_and_read_back_without_the_evaluator`, and the inspector's `a_query_without_the_feature_is_refused` (the inspector test file now compiles without `query`).

F-R2, `parse_jsonld(bytes, contexts, base_iri, options: JsonLdOptions)`, `JsonLdOptions { processing_mode: ProcessingMode }`, `ProcessingMode::{JsonLd1_0, JsonLd1_1}` (default 1.1), re-exported at the crate root. Spec: JSON-LD 1.1 Processing Algorithms and API, `processingMode`. `oxjsonld` 0.2.7 `with_processing_mode`.
Callers outside Mere: none (`git grep` for `parse_jsonld`, `parse_response`, `parse::parse` at `origin/main` of Graphshell `a377c352`, Turnstone `0a4636ae`, Cleromancy `2867b94`, Knot `090237c`, Dramatis, Eidetic, Genet, Isocosm: no hits; Turnstone's `pub use parse::parse` is its own module; a `mere::` grep finds each consumer's files, so the search reads). `formats::parse::parse` and `negotiate::parse_response` pass `JsonLdOptions::default()`.
Metric, the JSON-LD toRdf rows (`pass / fail / skip`):

| Layer | Before | After |
|---|---|---|
| parser (control) | 439 / 10 / 18 | 439 / 10 / 18 |
| engine | 432 / 10 / 25 | 439 / 10 / 18 |
| adapter | 441 / 1 / 25 | 441 / 1 / 25 (still 1.1 only) |

The engine and parser fail lists, from `W3C_VERBOSE=1`, are the same ten tests. Every other ROW line is unchanged. New test: `the_processing_mode_defaults_to_1_1_and_1_0_refuses_a_1_1_context`.

Gates: `cargo test -p mere` lib 30, `inspect_example` 3, `w3c_conformance` 1 (1 ignored); with `query` lib 31, `inspect_example` 3 (the query test replaces the refusal test), `w3c_conformance` 2; wasm32 `cargo check -p mere` with and without `query` (`RUSTFLAGS='--cfg getrandom_backend="wasm_js"'`); `cargo check --workspace --all-targets`; `cargo test -p mere-kernel --features query,bundled-contexts` 621 lib tests; commit hooks (clippy over `mere` and `mere-kernel`, dependency bans, line ceiling, decisions check) passed on both commits.

## 2026-11-13: P6 L2, RDFC-1.0 (Mere `<code>`, `<harness>`, `<docs>`)

D143 F-R7. Spec: W3C RDF Dataset Canonicalization (RDFC-1.0). Metric: the `rdf-canon` suite (`15619df2fda7`), 86 tests: 64 eval, 21 map, 1 negative.

Rulings (Mark), in his words, and what each did:
- 2026-11-12, on the work limit: "Dude. My priority is not bothering upstreams with fixes generated by ai. Patch, but presume I'm gonna say no; it's just a pita for overworked people. Why would i add to their suffering." The patch is carried in Mere; **no upstream report: no issue, no PR, no comment, nowhere.**
- 2026-11-13, on which implementation to carry: "Take and improve the more spec compliant of the two on the whole (beyond this one feature)". The two were `oxrdf` 0.3.5 `rdfc-10` and zkp-ld `rdf-canon` 0.15.3; the table is `Code/testing/engine-split/rdfc_comparison.md`. First reader's call on the weighing: `oxrdf` has one deviation in canonical output (repeated mentions collapsed, rare); `rdf-canon` has two on valid inputs (a quad naming one blank node twice is double counted, which is common; temporary ids ordered as strings, wrong from 11 ids).

Why the first plan stopped: `oxrdf` has no limit, and a blank-node-count limit cannot refuse the poison clique `test074c` (10 blank nodes; positive tests reach 19). A first run with a count limit (local, not landed) scored 64/64 eval, 21/21 map, 0/1 negative.

Comparison evidence (scratch crate, since removed): both implementations 64/64 eval and 21/21 map; `test074c` refused by `rdf-canon` in 0.28 s, never returned by `oxrdf` (over 2 minutes of CPU, killed). Differential fuzz of `oxrdf` against `rdf-canon` found 10,173 of 20,000 differing on dense datasets with self-mention quads, 0 once those were excluded for default graphs, literals and named graphs, and the repeated-mention and long-cycle differences above.

What `mere-rdfc` is: `crates/graph/rdfc`, `oxrdf` 0.3.5 `src/dataset.rs` lines 602-725, 727-796, 798-946, 949-1101, 1144-1192, 1996-2020 carried, MIT OR Apache-2.0, author Tpt, and seven marked changes (every mention kept; a budget on Hash N-Degree Quads calls and permutations; lazy permutations; own interner in place of the 568-line `interning.rs`; triple terms refused; entry points returning canonical N-Quads and the issued map; `sha2` 0.11). 890 lines in three files (`algorithm.rs` 525, `intern.rs` 140, `lib.rs` 136) plus 90 lines of tests.

Budget numbers: the largest positive test uses **468** Hash N-Degree Quads calls and **2,880** permutations; defaults **4,000** and **100,000**. `test074c` is refused after 4,000 calls: 0.4 s in a release build, 2.2 s in the debug harness run. Tests in `mere-rdfc`: self-mention counts once; twelve-cycle; a tied list of 14 (14! permutations, about 8.7e10, never materialized) trips the budget; triple terms refused; permutations are lazy and distinct. Tests in `mere`: dataset with issued map, SHA-384, triple term, clique refused.

Arbiter: zkp-ld `rdf-canon` 0.15.3 copied to a scratch crate with its two deviations fixed (the blank-node-to-quads map as a set, 4.4.3 step 2; temporary ids in issue order, step 5.3.1) and fuzzed against `mere-rdfc`: 16 modes x 20,000 random datasets (default graph; IRIs and literals; named graphs; blank-node graph names; repeated mentions in a second graph) and 48 cycle and chain shapes (n = 3 to 14). Result: 320,000 datasets, 18 differ, all in modes with blank-node graph names and all in datasets where an implementation is not invariant under relabelling its input (the unmodified `oxrdf` is not in 19 of 20,000; `mere-rdfc` in 19; fixed `rdf-canon` in 17); where both `mere-rdfc` and `rdf-canon` are invariant they agree on all 20,000. Of the 48 shapes, 1 differs (a 3-cycle with blank-node graph names).

The `rdf-canon` rows (`pass / fail / skip`):

| Type | Engine | Library (control, raw `oxrdf`) |
|---|---|---|
| eval | 64 / 0 / 0 | 64 / 0 / 0 |
| map | 21 / 0 / 0 | 21 / 0 / 0 |
| negative | 1 / 0 / 0 | 0 / 1 / 0 (`test074c`: still running after 10 s, no limit) |

Existing ROW lines: with and without `query`, the run with `rdf-canon` hidden and the run with it differ only by the `rdfc10` lines. The expected files of the suite are CRLF in this checkout (`core.autocrlf`; the git index is LF), and the harness normalizes them.

Gates: `cargo test -p mere` lib 34 (35 with `query`), `inspect_example` 3, `w3c_conformance` 1; `cargo test -p mere-rdfc` 5; ledger run with and without `query`, each ROW line equal to a run of the same binary with `rdf-canon` hidden, apart from the new `rdfc10` rows (the RDF and JSON-LD rows equal across `query`); `cargo check --workspace --all-targets`; `cargo test -p mere-kernel --features query,bundled-contexts` 621 lib tests; `cargo tree -d` shows `sha2` 0.10.9 and 0.11.1 and `digest` 0.10.7 and 0.11.3, the 0.10 pair held by `p2mere-encryption` under the existing `deny.toml` skips, 0.11 by `mere-rdfc` and the family, no new skip. wasm32 `cargo check` of `mere` (with and without `query`) and of `mere-rdfc` with `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'`: `mere-rdfc` needed the same `getrandom` 0.3 `wasm_js` target dependency `mere` carries (added); `mere` stops in `uuid` 1.29.2 (the workspace `uuid` has `v4` and no randomness source for wasm32, which this lane does not touch; not checked against `origin/main`) and checks clean, with and without `query`, with a temporary `js` feature on that workspace line, which was not kept.
