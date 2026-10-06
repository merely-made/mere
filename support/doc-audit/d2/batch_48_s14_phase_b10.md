# Batch 48 — S14 pass, phase B10 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-06-18_petgraph_rdf_plan.md | current | no | 23 | 18 | 5 | 0 |
| mere_docs/implementation_strategy/2026-06-22_isometric_orrery_camera_plan.md | historical-unmarked | no | 15 | 11 | 4 | 0 |
| mere_docs/implementation_strategy/2026-06-22_physics_scenes_and_tangibility_plan.md | historical-unmarked | no | 18 | 13 | 5 | 0 |
| mere_docs/implementation_strategy/2026-06-23_browser_extension_companion_plan.md | superseded | no | 16 | 9 | 7 | 0 |
| mere_docs/implementation_strategy/2026-06-23_node_body_face_model_plan.md | current | no | 13 | 7 | 5 | 1 |
| mere_docs/implementation_strategy/2026-06-23_render_ladder_and_extraction_plan.md | historical-unmarked | no | 15 | 7 | 8 | 0 |
| mere_docs/implementation_strategy/2026-06-24_orrery_browser_lane_plan.md | historical-marked | no | 5 | 4 | 1 | 0 |
| mere_docs/implementation_strategy/2026-06-25_operator_presence_overlay_plan.md | current | yes | 11 | 6 | 4 | 1 |
| mere_docs/implementation_strategy/2026-06-25_persona_transport_unlinkability_plan.md | current | no | 12 | 6 | 6 | 0 |
| mere_docs/implementation_strategy/2026-06-25_persona_wallet_carry_layer_plan.md | current | no | 16 | 12 | 4 | 0 |
| mere_docs/implementation_strategy/2026-06-26_capture_provenance_consent_plan.md | historical-unmarked | no | 14 | 9 | 5 | 0 |
| mere_docs/implementation_strategy/2026-06-26_mcp_native_graph_plan.md | current | yes | 7 | 5 | 2 | 0 |
| **Totals** |  |  | **165** | **107** | **56** | **2** |

**Totals: 12 docs, 165 claims checked (107 holds, 56 stale, 2 unverifiable), 24 contradictions; 10 status lines wrong.**

Audit base: Mere `535bca11` (2026-10-05). Sibling repositories were read at
their HEADs that day, genet also at Mere's pin `bd3e8861`. `archive_docs/` is
excluded. No cargo command was run.

This batch belongs to the stack seams plan's S14 pass (rulings S14 and S33),
phase B: the plans the early-September snapshot judged, re-judged against the
tree. Each record here supersedes the snapshot's record for its plan (ruling
S34). A read-only subagent (opus) drafted the records; a second, independent
read-only subagent (opus) then tried to refute every stale claim and checked
every status quote, and its corrections are applied below; this session
re-checked a sample directly. The "holds" counts are the draft's. Nothing in
the plans changes here; corrections are phase C's, and plans found complete
are extracted and archived there (ruling S36).

Checked directly in this session: seiche depends on rapier2d at the base
(physics scenes); the record corrections below were checked against the
verifier's evidence. The verifier refuted one claim (Graphshell does have a
face picker, `web_product.rs:475-484`) and found 5 items true only in part
(the physics scenes' hosts, where mer3ly's repo-graph binds backdrop scenes;
the extension plan's `node_dom`; the render ladder's `EngineRoutePolicy`
path; the operator overlay's web-clip pieces, built but unhosted; the
wallet's `WalletEpochSealer` consumers, castellan among them), plus two line
citations; those records are corrected.

## mere_docs/implementation_strategy/2026-06-18_petgraph_rdf_plan.md

- disposition: current
- status line: "Status: Phase 1A partial. Direction decided from the feasibility research + a perf benchmark: petgraph stays the truth and the runtime; RDF is a lossless, on-demand projection plus a SPARQL query adapter, never a second held authority. The target is not "the kernel stores all RDF" but a defined Mere RDF projection profile: the content subgraph is losslessly projectable, the experience/runtime layer stays native. The current landed slices are the semantic statement-bucket backbone, typed literal fidelity on `NodeProperty`, and graph-scope fields on semantic statements / node properties with dataset-query visibility. Full named-graph JSON-LD shaping, statement metadata, and the direct SPARQL adapter are still ahead. The work is to (1) carry the profile in the content model (statement records in pair-local edge buckets, typed literals, named-graph scope, statement provenance), (2) prove losslessness with a round-trip test, (3) run SPARQL over the kernel via a `QueryableDataset` adapter, and (4) — only if footprint demands — move storage to an interned slotmap kernel. (Merges the projection-profile take + the statement-bucket revision, 2026-07-04.)" — accurate: no
- claims checked: 23 — holds: 18, stale: 5, unverifiable: 0

### Stale claims

- **Status says Phase 1A is partial and that statement metadata and the SPARQL adapter are still ahead.** The plan's own entries say otherwise: 2026-07-04 landed statement metadata, 2026-07-05 reads "Phase 1 closed + Phase 2 gate green", and 2026-07-06 reads "Phase 3 landed". The tree agrees:
  - `PersistedSemanticStatement` carries `provenance_iri` and `asserted_at_ms` (graph-kernel `persistence_edge.rs:173-186`).
  - The statement write API is at `edge_ops.rs:296/322/349`.
  - The round-trip gate is `linked-data/src/lib.rs:987`.
  - `sparql()` runs on spareval (`query.rs:50`).
- **Phase 1 body (lines 122-124) and Risks (299-301) say Phase 1 is not done until a federation-safe minting story is pinned.** It has landed: the id format `{unix_ms:012x}-{process_salt:016x}-{counter:016x}` is at graph-kernel `types.rs:125` (minted at `:133-138`), and `seed_statement_minter` is at `types.rs:102`.
- **Findings (271-272) say "ingest still ignores triple-term reifier input".** `ingest.rs:231` and `:336-341` lift `rdf:reifies`, `prov:wasAttributedTo` and `prov:generatedAtTime`.
- **The Phase 3 done-condition and the 2026-07-06 entry name a `>sparql` verb.** No such verb exists, and nothing outside `crates/graph/linked-data` calls `sparql(` in crates/ or ports/. The verb went with Meerkat.
- **Findings line 262 calls `crates/probes/rdf-kernel-footprint/` "Re-runnable (`cargo run --release`)".** The probe is gone. `crates/probes` holds only mesh-lexical-wasm, murm-direct-phy, pack-distribution and wing-three-paths. The path carries a historical marker, but "re-runnable" asserts it is present.

### Contradictions

- The Status line contradicts the plan's own Progress entries from 2026-07-04 to 2026-07-06.
- DOC_README.md:235 calls the plan "**planned**", against Progress (Phases 1-3 landed).
- Phase 3 (lines 208-213) says the shipped spareval-over-`dataset_quads` path "already satisfies the done-condition". The done-condition itself says "over the kernel directly". `query.rs:47-50` projects "into a fresh in-memory dataset per call". Graph semantics plan ruling 6 says SPARQL will be served by an adapter "instead of a dataset rebuilt per query", which implies the condition is not met.

### Recommended action

- **Update the status** to: Phases 1-3 landed 2026-07-04 to 06; Phase 4 gated and deferred on the 2026-07-06 footprint measurement; the resource-graph SPARQL adapter is carried by graph semantics plan P2.
- **Mark as overtaken:** the Phase 1 allocator text, the Risks minting text and the Findings reifier gap.
- **Drop "Re-runnable"**, restate the Phase 3 done-condition without `>sparql`, and fix DOC_README:235.
- **If archiving**, first extract the open items: the JSON-LD shaper gaps, the `skip_serializing_if` follow-up, the ExampleOf and Summarizes rows, the raw-IRI Semantic edge path, and dropping `dep:oxigraph`.

### Notes

- **Symbols checked at 535bca11:** `SemanticStatementSpec` (`edge_data.rs:132`), `all_semantic_sub_kinds` (`:389`), `vocabulary_alignment_quads` (`vocab.rs:128`), the Knot evidence-sense rows (`vocab.rs:85-102`, test `:198-201`), `serialize.rs:37-81`, `from_quads` (`ingest.rs:233`), the snapshot_size gate (`tests/mod.rs:18`, `snapshot_size.rs:104`), `EdgeContributionWire` (content-contract `lib.rs:1316-1328`), and linked-data `Cargo.toml:26-42`.
- **Still open, correctly stated:** `PersistedSemanticStatement` has `serde(default)` but no `skip_serializing_if`, so that follow-up is open. ExampleOf (`vocab.rs:117`) and Summarizes (`:108`) are unchanged, so "two further rows still open" holds.
- **Graph semantics P1** is on branch `graph-semantics` (`459cad84`), which is not an ancestor of the base.

## mere_docs/implementation_strategy/2026-06-22_isometric_orrery_camera_plan.md

- disposition: historical-unmarked
- status line: "Status: Planning (with Mark). The "2.5D isometric rung" of the [orrery physics environments research](../research/2026-06-22_orrery_physics_environments_research.md): the cheap, in-stack dimensional mode (an isometric, orbitable camera with fake height) over the existing 2D rapier physics, with no rapier3d and no new render lane. Full 3D stays the gated long pole in its own plan." — accurate: no
- claims checked: 15 — holds: 11, stale: 4, unverifiable: 0

### Stale claims

- **Status says "Planning (with Mark)".** The final entry (2026-06-24, `70a486f`) says "the plan is fully complete … Nothing deferred remains". `715203a` and `70a486f` are ancestors of the base. The camera is live in pictograph:
  - `Camera` and `to_screen` / `to_world` / `ground_transform` at `canvas/scene_paint.rs:91-147`;
  - `set_isometric` / `orbit_by` / `set_tilt` at `canvas/view.rs:101-128`;
  - `pick_at` / `set_alt` / `orbit_drag` at `canvas/input.rs:56-226`.
- **2026-06-23 entry: the `ToggleProjection` command, the palette entry "Projection (toggle 2.5D isometric)" and `>projection`.** All three have 0 hits in mere. The live equivalent is Turnstone `Action::ToggleIsometric`: `src/action.rs:280`, palette "Toggle isometric view" at `:660`, key `i` at `shell/keys.rs:444`, script name at `script.rs:277`.
- **yaw/tilt persistence (`camera_to_snapshot` / `snapshot_to_camera` / `snapshot_yaw_tilt`).** 0 hits in mere and in Turnstone. Pandect's `CameraSnapshot` (`view_intent_store.rs:80`) is a bare affine that nothing fills with yaw/tilt, and Turnstone keeps yaw/tilt per runtime only (`app/runtime_pool.rs:182`). "Nothing deferred remains" is therefore no longer true of the tree.
- **The standalone `orrery` bin keys (i, q/e, [/], h) used for every headed check.** The canvas-host binary was deleted in `47833f65` (2026-09-24, an ancestor of the base).

### Contradictions

- The Status line contradicts the plan's final Progress entry, which says the plan is complete.
- DOC_README.md:262 says "**planning (with Mark)**", against the final entry.

### Recommended action

- **Update the status** to: complete 2026-06-24 (`70a486f`); the camera lives in pictograph `canvas`; Turnstone hosts toggle, orbit, tilt, height and the Alt modifier; yaw/tilt persistence was lost with Meerkat.
- Fix DOC_README:262.
- **Archive per DOC_POLICY §8** after extracting the persistence gap to a live owner (a fork, listed below).

### Notes

- **Holds:** the four Code-header paths carry historical markers. `node_height` and `set_height_by_degree` are at `canvas/cartography.rs:634,645`. The camera tests are at `scene_paint.rs:550/567/585`.
- **Turnstone pushes `set_alt`** at `src/shell/events.rs:288`, so the in-app Alt+drag orbit survives in one host.
- All four related links resolve at the base.

## mere_docs/implementation_strategy/2026-06-22_physics_scenes_and_tangibility_plan.md

- disposition: historical-unmarked
- status line: "Status: Planning (with Mark). The build plan for the [orrery physics environments research](../research/2026-06-22_orrery_physics_environments_research.md): non-node physics scenes sharing the orrery's rapier world, the interactive/intangible tangibility lever, and the research's two features (living backdrop, interactive scene) plus liquid. The dimensional modes are owned by the [isometric orrery camera plan](2026-06-22_isometric_orrery_camera_plan.md); this plan is the scene *content and physics*, that one is the *view*." — accurate: no
- claims checked: 18 — holds: 13, stale: 5, unverifiable: 0

### Stale claims

- **Status says "Planning (with Mark)".** The final entry (2026-06-24) says "The physics-scenes arc is feature-complete". The engine is live:
  - seiche `scene_sim.rs:38-265`, `fluid_coupling.rs:26-47`, `field.rs:19-26`, `emitter.rs:50-61`;
  - `SCENE_BODY_CAP` at `lib.rs:252`;
  - pictograph `canvas/nodes.rs:19-217` and `canvas/ambient/mod.rs:47`.
- **The Code header marks "optional new `crates/orrery/scene`" as *(planned target)*.** The plan's own P3/P4b put the scene format inside seiche (`crates/conatus/seiche/src/scene_spec.rs`, with the 13 catalog scene functions in seiche). Nothing commits to creating that crate, so the marker should be historical.
- **The Meerkat Scene settings page (`pelt/scene`, `scene_settings.rs`, `apply_scene_key`, `06eae2b`).** 0 hits. No Mere or Turnstone host loads scenes (no `load_scene`, `SceneSpec`, `set_nodes_tangible` or `load_game_of_life` in ports/ or Turnstone src/); mer3ly's repo-graph does (`apply_backdrop`, `crates/repo-graph/src/lib.rs:1683-1705`, since `240b4f4`, 2026-08-12, loading seiche scenes and setting tangibility).
- **The `>scene <name>` verb and `load_named_scene` (`96b1ebc`).** 0 hits.
- **The orrery bin keys (1-9, 0, f, c/b/k/m, x, g, n, p, s, t) used for headed verification.** The binary was deleted in `47833f65`.

### Contradictions

- The Status line contradicts the final Progress entry ("feature-complete").
- The Code header lists `salva2d` (liquid). The plan's P4 says "salva is out", and P4c built its own PBF fluid.
- DOC_README.md:248 says "**planning (with Mark)**".

### Recommended action

- **Update the status** to: complete 2026-06-24; engine in seiche plus pictograph canvas; no Mere or Turnstone host binding since Meerkat was removed (mer3ly's repo-graph binds backdrop scenes).
- **Fix the header:** drop `salva2d`, and change the scene-crate marker to historical.
- **Before archiving**, extract the open follow-ons: host scene-picker and tangibility binding, bounds-drain, a fluid emitter, reuse of aether's `CouplingForce`, and re-applying tangibility on node add.

### Notes

- **Holds:**
  - the live Code-header path `crates/canvas/pictograph/src/canvas/scene_paint.rs`;
  - rapier2d 0.33 (seiche `Cargo.toml:19`);
  - `ContactShape` (`fluid.rs:109`);
  - `register_scene_sprite` (`nodes.rs:97`);
  - 16 cited commits, all ancestors (`5ad0bb8` through `f1f05dc`);
  - all six related links.
- **Not a successor:** `physics_catalog.rs` (2026-09-02 physics catalog plan) covers the graph's force laws, not backdrop scenes.

## mere_docs/implementation_strategy/2026-06-23_browser_extension_companion_plan.md

- disposition: superseded
- status line: "Status: Planning (with Mark). Net-new delivery target; no code yet. Node + delivery framing superseded 2026-06-24 by [orrery_browser_lane_plan](2026-06-24_orrery_browser_lane_plan.md) (capture-first, favicon-body nodes not "DOM cards", gloss sidebar + orrery discrete tab, baseline cross-browser, no-sync v1). The companion / smolweb / p2p / federation half below stands as the forward arc beyond that v1." — accurate: no
- claims checked: 16 — holds: 9, stale: 7, unverifiable: 0

### Stale claims

- **Status says "no code yet" and that the companion half "stands as the forward arc".**
  - The Graphshell reference host plan (`2026-07-27_graphshell_reference_host_plan.md:25-28`) "absorbs the live parts" of this plan, which "remain historical evidence".
  - An MV3 extension ships at `ports/graphshell/web/extension/` (`background.js`, `capture-model.js`, `install-native-host.ps1`), with its capture core at `ports/graphshell/src/capture.rs`.
- **§3 and Findings say `render_as_cards` "already" carries the DOM-card seam.** 0 hits in crates/ and ports/.
- **The per-node accessor list.** `node_representation` has 0 hits; the other five survive at pictograph `canvas/cartography.rs:65-112`.
- **The P0 gnode-pool symbols.** `node_layout` has 0 hits; `build_pool_dom` survives (`canvas/build.rs:222`), and `node_dom` remains only as the local name for its result (pictograph `lifecycle.rs:116,393`), so the genet-backed node pool still compiles in.
- **P0 cargo line `-p orrery`.** No `orrery` package exists.
- **Findings and P2: the capture substrate is "built and dormant … zero live callers".** `BrowsingMemory::record_traversal` has live callers at `ports/graphshell/src/capture.rs:482` and in Turnstone `src/trail_memory.rs`.
- **The keep set `gyre`, `aether`, `arrangements`.** No such packages exist; the successors are seiche, numen and the cartography adapters. Only `cartography` (`crates/canvas/cartography`) survives by name.

### Contradictions

- The status names orrery_browser_lane_plan as its successor. That plan is itself superseded (its 2026-07-27 banner), and the Graphshell plan absorbs both. This doc never names Graphshell.
- DOC_README.md:265 repeats "planning (with Mark)" and the stale `render_as_cards` / `orrery` core framing.

### Recommended action

- **Add a supersede pointer** to the Graphshell reference host plan (H4/H5), and set the status to: superseded 2026-07-27, kept as evidence.
- Fix DOC_README:265.
- It is an archive candidate (a fork, listed below).

### Notes

- **Holds:**
  - `crates/script/wit/world.wit`;
  - document-host `wasmtime = "45"` (`Cargo.toml:25`);
  - graph-kernel `description = "Portable …"` and `store = []` off by default (`Cargo.toml:12,24`);
  - `mere::glossary` (`crates/mere/src/glossary.rs:59,172`);
  - `kernel::permissions` (`lib.rs:34`) plus `Grant` (document-host `capabilities.rs:103`);
  - pandect `script_bindings_store.rs` and `settings_store.rs`;
  - nematic;
  - `SharedNavigationMemory` (`history.rs:121`);
  - all nine Related links.
- The Graphshell H5a/H5b receipts are at `ports/graphshell/docs/2026-07-28_h5*`.

## mere_docs/implementation_strategy/2026-06-23_node_body_face_model_plan.md

- disposition: current
- status line: "Status: Planning (with Mark). Successor to the representation half of the [node_representation_arrangement_plan](2026-06-18_node_representation_arrangement_plan.md), which is substantially complete (P0 cues, P1 per-node form, P2-static sprite faces + the sprite-alpha hull collider, P3/P4 done) and whose representation axis this plan re-bases. The arrangement half already spun out to [graph_signals_layer_plan](../../archive_docs/2026-08-20_completed_plans/2026-06-22_graph_signals_layer_plan.md) (Decision 7). This plan collects the still-open representation follow-ons so they are not orphaned." — accurate: no
- claims checked: 13 — holds: 7, stale: 5, unverifiable: 1

### Stale claims

- **Status says "Planning".** B0, B1 and B2 have landed:
  - **B0:** `node_collider` applies the hull "regardless of the face" (pictograph `canvas/lifecycle.rs:452-475`).
  - **B1:** `enum Face { Favicon, Derived, Sprite, Bare }`, with `Bare` "(Formerly `Shape`.)" (`canvas/types.rs:147-165`).
  - **B2:** `set_node_material` / `clear_node_material` over `seiche::NodeMaterial` (`canvas/cartography.rs:400-430`; seiche `node_body.rs:77`).
  - Materials and faces persist via `.with_materials` and `.with_faces` (`cartography.rs:48,55`).
- **The "Current state (code-verified 2026-06-23)" section still describes the coupling as present.** It names a `Representation` axis, the face pick `match representation`, and the Sprite-gated hull. No Progress entry records B0-B2. Pictograph has no `Representation` enum; the only one in the tree is unrelated (cambium `sceno/src/scene.rs:95`).
- **"Physical material is near-fixed … only `linear_damping` is per-node tunable."** The B2 setter above contradicts it.
- **2026-07-09 entry: "`FAVICON_INSET` 0.72 in orrery's lib.rs".** It is now `const FACE_INSET: f32 = 0.72` at pictograph `canvas.rs:266`.
- **The follow-on homes "so they are not orphaned" are archived.** settings_lane_consolidation_plan is in `archive_docs/2026-07-13_superseded_plans/`, and object_card_plan is in `archive_docs/2026-09-02_retired_plans/`. They were the owners of the node:<id> facet pane, scene-wide defaults, label density and the widget surface.

### Contradictions

- DOC_README.md:233 says "planning (with Mark)". It matches the status line, but both are contradicted by the tree. Progress never records B0-B2.

### Recommended action

- **Update the status** to: B0-B2 landed (pictograph canvas plus seiche); B3 shape editor and B4 open. Turnstone hosts sprite import with a hull (`src/app/node_arms.rs:1732-1743`); Graphshell has a face picker (`web_product.rs:475-484`, the "Representation" select in `web/component.html:312-313`); no host has a material picker.
- Rewrite "Current state" as dated history.
- Re-home the follow-ons whose owner plans are archived.

### Notes

- **Holds:**
  - B3 is genuinely open: the only hull API is `set_node_sprite_hull`, and there is no editor;
  - `Face::Sprite` is cover-fit;
  - the receipt `Code/testing/turnstone/images/2026-07-09_favicon_inset_frame.png` exists;
  - the Code-header markers;
  - eleven links resolve;
  - the seiche `NodeCollider` lowering.
- **Unverifiable:** B4's "live WebView form built, not blocked … in pelt tiles". `crates/inker/engines/scrying-engine` exists, but the tile path needs a runtime check.

## mere_docs/implementation_strategy/2026-06-23_render_ladder_and_extraction_plan.md

- disposition: historical-unmarked
- status line: "Status: substantially built (2026-07-01) — see Progress. Phase 1a (rung taxonomy), phase 2a-c (scripted render rung + external scripts + cookies), phase 3 (input → event bridge), and phase 4 slices 1-3 + wiring (genet-extract, Contribution path, headless-scripted + reader-mode extract, single-hop link materializer) all landed 2026-06-23/24. The `--features scripted` compile break (the `ResourceFetcher` trait mismatch at `content/actor.rs:33`) was fixed 2026-07-01 — see the Progress entry; the scripted feature builds and its 5 tests pass. 1b (picker surfacing), keyboard dispatch, interactive-region refinement, and the crawl frontier (V2 actor) remain open. Grounds the page-JS lane as a *rung*, not a static-path replacement, and adds the orthogonal analysis axis." — accurate: no
- claims checked: 15 — holds: 7, stale: 8, unverifiable: 0

### Stale claims

- **Status says "the scripted feature builds and its 5 tests pass".** Meerkat's `scripted` lane is gone: `build_scripted`, `ScriptFetcher`, `click_scripted`, `page_extract_contribution`, `harvest_links` and `materialize_links` all have 0 hits.
- **Status says the crawl frontier (V2 actor) remains open.** `crates/crawl` exists ("Host-neutral crawl frontier and bounded crawl runtime", `Cargo.toml:8`), with `Frontier` / `max_depth` (`frontier.rs:61`), robots, sitemap and a dedicated actor (`lib.rs:7-16`).
- **The rung selector `engine_pins`.** 0 hits; `EngineRoutePolicy` survives (`crates/inker/inker/src/routing.rs:76`).
- **The Phase 2 Meerkat content-actor scripted lane.** 0 hits.
- **Phase 3 `Constellation::click_scripted` / `is_scripted`.** 0 hits. `ContentCommand::ScriptedClick` survives only as a cfg-gated wire variant (content-contract `lib.rs:169-170`, `:432`) with no producer or consumer.
- **The Phase 4 `genet-extract` crate.** Genet has no such component; `PageExtract` is in fleece (genet `components/fleece/src/lib.rs:383`).
- **The Phase 4 Meerkat ingest wiring (`page_extract_contribution`, `contribution_from_page_extract`, `harvest_links`, `materialize_links`).** 0 hits. `ContentCommand::MaterializeLinks` is an orphan wire variant (content-contract `lib.rs:166`).
- **The line-3 banner says "the graphlets crate is crates/graph/subgraph".** It is `crates/mere/src/subgraph.rs`; `crates/graph` holds only graph-kernel and linked-data.

### Contradictions

- The status says "1b (picker surfacing) … remain open". The 2026-06-23 phase 2a entry says "Picker surfacing (1b) landed with it".
- DOC_README.md:271 says "partially integrated in Meerkat; remaining host/storage tails", against the status "substantially built".

### Recommended action

- **Rewrite the status:** the Meerkat integration left with Meerkat; the genet and fleece substrate survives; pelt's scripted viewers host the scripted rung (`ports/pelt/desktop/scripted_viewer.rs:54-63`); the crawl frontier landed in `crates/crawl`.
- Fix the banner and DOC_README:271.
- Whether to archive is a fork, listed below.

### Notes

- **Holds:**
  - `GenetRung`, `genet_rung` and `is_genet_rung` at genet `components/shared/document-session-api/src/engine_ids.rs:105,149,162`, re-exported by `crates/inker/inker/src/routing.rs:16`, test at `routing/tests.rs:303`;
  - genet `dispatch_event` (`genet-scripted/document.rs:408`), `click_at` (`:1060`) and `extract()` (`:525`);
  - fleece `extract_main_text` (`:822`);
  - `ScriptResourceFetcher` (`ports/pelt/desktop/lib.rs:131`, `scripted.rs:12`);
  - genet `docs/2026-05-12_genet_profile_ladder_plan.md`;
  - the four linked plans.

## mere_docs/implementation_strategy/2026-06-24_orrery_browser_lane_plan.md

- disposition: historical-marked
- status line: "Status: planning / design 2026-06-24. Supersedes the node-representation and delivery framing of the [browser_extension_companion_plan](2026-06-23_browser_extension_companion_plan.md): its "orrery-in-a-tab live DOM cards" P1 is replaced here by capture-first, favicon-body nodes, a gloss sidebar, and the orrery as a discrete surface. The companion / smolweb / p2p / federation half of that plan stands as the forward vision; this plan is the shippable v1 with no native sync." — accurate: no
- claims checked: 5 — holds: 4, stale: 1, unverifiable: 0

### Stale claims

- **The status calls this "planning / design" and "the shippable v1".** The product was superseded 2026-07-27, per the plan's own banner (lines 11-14) and the Graphshell plan (`:25-28`). The v1 extension shipped as Graphshell H5: `ports/graphshell/web/extension/`, receipt `ports/graphshell/docs/2026-07-28_h5a_browser_storage_capture_core_receipt.md`.

### Contradictions

- The Status paragraph still says planning / v1, while the banner two lines below says superseded.
- DOC_README.md:266 says "planning / design 2026-06-24" and gives no supersession.

### Recommended action

- **Fold the banner into the Status line:** "superseded 2026-07-27 by the Graphshell reference host plan; evidence only".
- Fix DOC_README:266.
- It is an archive candidate (a fork, listed below).

### Notes

- **Holds:** the supersession of the companion plan (that plan's status names this one); the Graphshell banner and the extension ownership; ten links resolve, including genet `docs/2026-06-24_nova_memory64_browser_lane_plan.md`.
- **Expected under historical-marked, not counted:** the interior's stale names (`eidetic-opfs`, `genet-extract` → fleece, the `orrery` package, meerkat). Graphshell used an IndexedDB `muniment::Backend`, not OPFS.

## mere_docs/implementation_strategy/2026-06-25_operator_presence_overlay_plan.md

- disposition: current
- status line: "Status: design (2026-06-25). A live, ephemeral, multi-scale focus-presence overlay: a colour-tagged ring around the node an operator is on, descending into the node to highlight the document section they are reading. One "operator" abstraction with three sources: an agent, a co-op guest, and you. The rendering half mostly exists; the new piece is a small ephemeral presence channel. No presence code yet." — accurate: yes
- claims checked: 11 — holds: 6, stale: 4, unverifiable: 1

### Stale claims

- **"The agent already has a focus (built)": `AgentObservation { focused_node … }`.** 0 hits in mere and Turnstone; the harness went with Meerkat. The link carries a historical marker, but the prose is present tense, and build-path step 1 ("buildable … plus the harness today") rests on it.
- **"The find overlay already highlights document regions at their rects (built)".** No find overlay exists in mere.
- **The web-clip inspector is "planned, not built".** Partly overtaken: its building blocks exist in `crates/import/src/web_clip.rs` (`web_clip_script` with an `elementFromPoint` picker, `:63,94`; `ClipRect`, `:33`; `write_clip_node` writing a `ClippedFrom` edge, `:293,346`; `build_clip_knot`, `:465`), but no host calls the picker or `write_clip_node`; Turnstone uses only the text-fragment half (`src/knot_authoring.rs:364,375`), and the hover-pick inspector itself is unbuilt.
- **"The scrying / verso-scry live-tile element primitives" are listed as built.** There is no verso-scry; only `crates/inker/engines/scrying-engine` exists.

### Contradictions

- none.

### Recommended action

- **Update "Why this is mostly already rendered" and "Exists vs gap":** the harness focus source and the find overlay are gone; the web-clip picker exists in `crates/import`; name a live agent-focus source.

### Notes

- **Holds:**
  - no `OperatorFocus` or `Presence` code anywhere;
  - `Overlay::{ClusterHalo, BridgeEmphasis}` (cartography `overlay.rs:18-30`) with a minimap consumer (`minimap.rs`);
  - the a11y projection (Turnstone `src/a11y.rs`);
  - `ProvenanceSubKind::ClippedFrom` (graph-kernel `edge_taxonomy.rs`);
  - moothold reciprocity (`moothold/src/reciprocity.rs`; concord lives in `crates/moot/mien`);
  - the djot and wallet links.
- **Unverifiable:** "remote presence waits on the co-op browsing lane". Graphshell has co-op receipts (`ports/graphshell/docs/receipts/co_op_*`), but I did not establish whether that lane is the precondition the plan means.

## mere_docs/implementation_strategy/2026-06-25_persona_transport_unlinkability_plan.md

- disposition: current
- status line: "Status: design (2026-06-25). Mode 1 + the relay-diversity half-measure are the near-term buildable plan; the own-device-cluster family is the core of the privacy story and is mostly a design gap, not an implementation gap; Nym and the metadata ceiling are a named research track. No code yet." — accurate: no
- claims checked: 12 — holds: 6, stale: 6, unverifiable: 0

### Stale claims

- **Status says "No code yet".** The device fabric that Sequencing step 2 calls for exists: `DeviceRoster` (personae `carry/mod.rs:350`), `DeviceExposure { HiddenClient (default), ExposedEgress }` (`carry/mod.rs:164-171`) and `ACTION_TRANSPORT_EGRESS` (`carry/scope.rs:45`). `PersonaManifest.egress` and per-persona endpoint routing are absent.
- **"There is no identity-level `DeviceRoster`".** Same evidence.
- **`crates/persona/identity` is marked *(planned target)*.** The crate is `crates/dramatis/personae`. The doc does not commit to creating that path, so the marker kind is wrong.
- **`crates/moot/gemot/src/tessera`, `persona_chain.rs` and `gate.rs` are marked planned.** The files exist at `crates/moot/mien/src/persona_chain.rs` and `crates/moot/mien/src/gate.rs` (`posting_threshold` at `:31`). Both the marker kind and the path are wrong.
- **"iroh 0.98 does not expose deliberate relay-through-node".** The transport pins `iroh = "1.0.3"` (`crates/murm/transport/Cargo.toml:51`). Whether 1.0.3 exposes it was not checked.
- **"Maps onto Mere's coalition/moothold federation tiers".** Coalition is retired: `TERMINOLOGY.md:152` says gemot was renamed from coalition 2026-07-30.

### Contradictions

- "What we have today" says no `DeviceRoster` exists. The wallet plan records it landing 2026-07-02, and this plan's own 2026-07-02 entry names it without saying it landed.
- DOC_README.md:255 says "no code yet".

### Recommended action

- Update the status and "What we have today" (device fabric landed via the wallet store).
- Convert the planned markers to historical markers with the current mien and personae paths.
- Replace "coalition" with gemot.
- Re-check the iroh relay API claim against 1.0.3.

### Notes

- **Holds:**
  - `crates/murm/transport/src/p2panda_transport.rs`;
  - `bind(master: &Ed25519Keypair, …)` (`:615`);
  - opt-in mDNS and random-walk builder calls (`:37-39`);
  - `PersonaId(pub uuid::Uuid)` (personae `lib.rs:120`);
  - the persona salt BLAKE3("persona" || uuid) (`carry/mod.rs:471-476`);
  - `posting_threshold` (mien `gate.rs:31`).
- **Not found:** `HostingCommitment` has 0 hits. It reads as a design reference, so it was not counted.

## mere_docs/implementation_strategy/2026-06-25_persona_wallet_carry_layer_plan.md

- disposition: current
- status line: "Status: storage slice, first host-adoption slice, typed signed-grant (2026-07-04) slice, remote-auth grant issuance slice, wrapped private-epoch crypto helper slice, pairing-transcript helper slice, pairing ticket/code helper slice, first Meerkat pairing-host slice, delegated-device response/SAS preview slice, enrollment-bundle slice plus delegatee enrollment-host/bootstrap-preservation slice, and the first `private.read` host/restore slice plus pairing-expiry/artifact-coherence hardening slice plus first capability-slot wiring slice plus first delegated-device revocation slice landed, plus the encrypted-vault design slice and the first local sealed-record unlock/migration slice plus the startup-unlock setting / locked-startup flow slice. Companion to the [persona_transport_unlinkability_plan](2026-06-25_persona_transport_unlinkability_plan.md). The wallet is "Layer 0", the carry layer everything else references. Most of what sits under it exists or is named in code; the identity-level and persona-level wallet manifest stores are now real in `session-runtime`, and Meerkat now seeds/loads them at startup and points `sync`/`comms` at the shared identity root. `identity/grants/<device_id>.cbor` now also has a typed signed envelope with canonical CBOR encode/decode, signing, verification, stable content-hash helpers, a remote-auth issuance helper that updates roster/index state coherently, XChaCha20-based wrap/unwrap helpers for private-epoch material, and a deterministic pairing-transcript helper that derives both the wrapping key and short auth string from a shared pairing secret plus device identities. The wallet layer also now has a typed pairing ticket/response seam for QR or manual-code transport, and Meerkat now has an artifact-based omnibar host seam that can mint remote-auth pairing tickets and accept a filled response artifact into grant issuance. That host seam is still manual/admin, but it now requests `identity.act` plus `private.read`, loads the delegator's current plaintext private epoch from a temporary per-persona bridge, wraps it into the grant, and exports an enrollment artifact the delegatee can install. Meerkat can now also materialize the delegated device side: it persists a local delegated-device identity bridge, caches the pairing ticket locally, writes a filled response artifact from a scanned ticket, previews the shared short auth string before grant issuance, and on install restores the signed grant, persona wallet manifests, roster enrollment, grant index, and the current plaintext private epoch against that local delegated-device identity. Remote-auth revocation can now also mark a delegated device revoked, clear its persona wallet slot grants, block new enrollment-bundle export, rotate future-write private epochs when that device had `private.read`, and refresh the remaining pairing-backed delegated grants with new wrapped epoch material for the rotated head. The identity seed, local delegated-device identity, owner-side wrapping-key bridge, and temporary persona epoch bridge now all have sealed-record migration paths under the new vault seam; the remaining gap is the actual PAKE/QR chrome, transport UI around that shared secret, per-persona encryption-at-rest and epoch-history usage beyond the current epoch, copy-mode export/import, and non-Windows startup unlock backends." — accurate: no
- claims checked: 16 — holds: 12, stale: 4, unverifiable: 0

### Stale claims

- **The status's Meerkat host slices** (Meerkat seeds/loads at startup, the pairing-host and delegatee-host seams, the `pelt/wallet` unlock page). Meerkat is gone; `pelt/wallet` and `pair_remote_auth` have 0 hits. Castellan now hosts grant issuance and revocation (`ports/castellan/src/reticulum/grant.rs:26,127`; `authority.rs:25,467`).
- **"Stores are now real in `session-runtime`", and the `session-runtime::wallet_store` / `wallet_grant` / `WalletEpochSealer` names.** They now live at `crates/system/pandect/src/wallet_store/`, `wallet_grant/` and `codicil_seal.rs:52`.
- **2026-07-08: "Gap #2 is now wired end to end".** The graph-engram sealed path (`save_graph_engram_sealed`, `open_engram_as_session_sealed`, `compose_graph_engrams_sealed`, Meerkat `export.rs` and `shell_load.rs`) has 0 hits. `WalletEpochSealer`'s live consumers are castellan (`payload_sealer`, `ports/castellan/src/authority.rs:448-452`, wrapping a backend in `sealed_storage.rs:28-35`) and pandect `wallet_sealed_backend.rs:22-58`.
- **The 2026-07-04 relock follow-on ("Lock now", the offline transition).** It went with Meerkat. The vault lock plan (`dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md` §1-2; it cites this plan at `:25-26`) finds "the vault never locks".

### Contradictions

- The status lists "per-persona encryption-at-rest" as a remaining gap. The 2026-07-08 entries say gap #2 is wired end to end.
- DOC_README.md:260 stops at the 2026-07-02 slices and says `master.seed` and the delegated identity are "temporary plaintext bridges". The plan's 2026-07-04 entries record their sealed-record migration (personae `sealed_record_storage.rs`).
- The vault lock plan's "the vault never locks" contradicts this plan's relock entry.

### Recommended action

- **Rewrite the status:**
  - crate paths are pandect and personae;
  - the host is castellan;
  - the Meerkat host slices are gone;
  - gap #2's host half survives as the pandect sealed backend, while graph-engram sealing went with Meerkat;
  - lock/unlock follow-through belongs to the vault lock plan.
- Fix DOC_README:260.

### Notes

- **Holds:**
  - pandect `wallet_store/mod.rs` and `wallet_grant/`;
  - personae `sealed_record_storage.rs`, `startup_unlock.rs` and `passphrase_root.rs`, with `load_passphrase_root` and `change_passphrase`;
  - eidetic `seal.rs`, `PayloadSealer` and `resolve_sealed_blob`;
  - `save_typed_sealed` / `load_typed_sealed` (eidetic `typed.rs`);
  - `WalletEpochSealer::for_persona` (`codicil_seal.rs:75`);
  - `StartupUnlockMode` persisted (pandect `device_settings_store.rs:299ff`);
  - `AutoOs` is Windows-only (`startup_unlock.rs:14-15`), so non-Windows backends are genuinely open;
  - `capability_slots` (carry `mod.rs`, `wallet_grant/enroll.rs`, `issue.rs`);
  - `DeviceRoster`;
  - the transport link.
- **At the base:** vault lock L1 (`2556a20c`) is not an ancestor.

## mere_docs/implementation_strategy/2026-06-26_capture_provenance_consent_plan.md

- disposition: historical-unmarked
- status line: "Status: C1 (live recorder) + C2 (candidate-context) built + runtime-verified (2026-06-26); the relational-browse V1 materializer trigger (`>materialize`) that lights C2 up is shipped + verified. C5 (page text into the index) is built + verified (`>recall`, `8b8b039`); C3's materialize / crawl half (harvested links record `ExtractedFrom` provenance) is built (`cdd2130`), and the web-clip case now writes `ClippedFrom` provenance from `>clip`. C4's membrane is live: the consent gate (`>capture`), retention (`390d74a`), forget (`>forget`, traces + index, `00a5331`), and federatability (the existing `PrivacyClass`) are built + verified. Remaining: forget's provenance-edge cleanup, C3's excerpt / summarize / generated-node provenance cases, and Phase 9 federation promotion/consumption. Created from the 2026-06-26 cross-cutting state audit (crawl / engram / knot / federation / models / graph / documentscript), which found that the left half of the browsing-data vision (browse, crawl, extract, local index) is largely built, the right half (distill, federate, tessera) is mostly designed, and the connective tissue between them, the live capture record, is owned by no plan and written by no running code." — accurate: no
- claims checked: 14 — holds: 9, stale: 5, unverifiable: 0

### Stale claims

- **The status says C1-C5 are built and verified through `>materialize`, `>recall`, `>capture`, `>forget` and `>clip`.** All five are Meerkat verbs and are gone. The live recorders are now Turnstone `src/trail_memory.rs` (capture plus recall) and Graphshell `ports/graphshell/src/capture.rs` (consented intake, `forget_url` at `:536`).
- **C1: `browse_capture.rs` tapped in `nav_sync.rs`.** 0 hits.
- **C2: candidates filled from the focused node's out-edges, plus the `>materialize` trigger.** `candidate_links` has 0 hits. Graphshell writes `candidates: Vec::new()` (`capture.rs:496`).
- **C4: a persisted `CaptureConsent` honoured by `record_browse_nav`.** 0 hits.
  - Turnstone's consent gate is a no-op stub (`trail_memory.rs:307-312`: "C4 replaces this body").
  - Graphshell has its own disabled-by-default `HistoryCapturePolicy` (`capture.rs:177-207`).
- **Findings: "`ContentNetFetcher` is a real backend over `fetch_page`".** It is gone. The seam is the `NetFetcher` trait (`crates/script/document-host/src/net.rs:25`), and `fetch_page` is at `crates/system/fetch/src/lib.rs:667`.

### Contradictions

- The plan says C4's consent membrane is live. Turnstone `trail_memory.rs:307-312`, which cites "capture plan C4", says the gate is a no-op until C4 lands.
- Findings still state, unmarked, that "the Provenance edge family has no live writer", that there is "No consent / … retention / forget anywhere", and that "eidetic-search still indexes only titles / URLs". Progress resolves all three (`cdd2130`, C4, `8b8b039`).
- DOC_README.md:277 says "planning …; no code yet", against the status "built + verified".

### Recommended action

- **Rewrite the status:** the Meerkat host was removed; the substrate survives; Turnstone and Graphshell are the current recorders.
- Mark the stale Findings resolved, and fix DOC_README:277.
- Which plan owns C4 now is a fork, listed below.

### Notes

- **Holds:**
  - `TraceEvent.candidates` (eidetic `browsing/mod.rs:143`);
  - `rebuild_with_text` (eidetic-search `index.rs:204`);
  - the `ExtractedFrom` writer (linked-data `ingest/apply.rs:173-185`) and `record_derivation` (`node_props.rs:360`);
  - `ClippedFrom` plus `build_clip_knot` (import `web_clip.rs:346`, `:465`);
  - `apply_quota` (`browsing/mod.rs:345`) and `retention_keep_n` (pandect `settings_store.rs:86`);
  - `forget_url` (`:368`);
  - `save_trace` stamps LocalOnly (`:180-191`);
  - 7 commits are ancestors;
  - 8 links resolve.

## mere_docs/implementation_strategy/2026-06-26_mcp_native_graph_plan.md

- disposition: current
- status line: "Status: Scoped, future, not yet. Captured from the [borrowed-ideas brief](../research/2026-06-25_borrowed_ideas_brief.md) at Mark's direction (scope both directions now, build later). Mere is agentic, so MCP (the Model Context Protocol) is the natural agent boundary: it makes external agents first-class citizens of the space and gives internal agents a standard way to reach out, both without bespoke glue. No work scheduled." — accurate: yes
- claims checked: 7 — holds: 5, stale: 2, unverifiable: 0

### Stale claims

- **"The agent's API is already the same `ActionRegistry` the user drives".** `ActionRegistry` appears only as a string literal (`crates/system/registry/src/mod_loader/loader/registry/registry_default.rs:71`). The linked command registry plan is retired (`archive_docs/2026-09-02_retired_plans/`).
- **"The agent harness already runs over `Command::ALL`".** 0 hits.

### Contradictions

- **Overlap with two other plans.** The Graphshell reference host plan's H9 (`:1403-1416`) schedules "an optional MCP adapter" over Graphshell's projection, query and intent grammar. The djinn family resident services plan's F2 (`:384-390`) makes MCP "an optional adapter" over djinn's authenticated agent door. Both differ from this plan's "No work scheduled" and from its command-registry substrate.

### Recommended action

- Re-anchor the expose half on the Graphshell intent grammar or the djinn agent door, or mark the plan superseded by H9/F2 (a fork, listed below).

### Notes

- **Holds:** no MCP code anywhere in the tree (grep over `*.toml` and the crates/ports `.rs` files); the capability-gate catalogue, federation_interop, borrowed-ideas, local-models-harness and archived document-script links; `ProvenanceSubKind` exists.
