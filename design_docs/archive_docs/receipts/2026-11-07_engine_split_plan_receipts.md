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
