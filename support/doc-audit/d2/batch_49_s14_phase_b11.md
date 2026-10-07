# Batch 49 — S14 pass, phase B11 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-06-30_runtime_mod_authoring_loop_plan.md | historical-unmarked | no | 12 | 6 | 6 | 0 |
| mere_docs/implementation_strategy/2026-07-02_meerkat_promotion_pass_plan.md | historical-marked | no | 12 | 9 | 3 | 0 |
| mere_docs/implementation_strategy/2026-07-03_archived_plan_tails_plan.md | current | yes | 23 | 15 | 8 | 0 |
| mere_docs/implementation_strategy/2026-07-05_inference_provider_plan.md | current | no | 10 | 7 | 3 | 0 |
| mere_docs/implementation_strategy/2026-07-05_overlay_roots_and_ua_widgets_plan.md | historical-unmarked | no | 9 | 4 | 5 | 0 |
| mere_docs/implementation_strategy/2026-07-05_theme_modes_plan.md | current | yes | 10 | 6 | 2 | 2 |
| mere_docs/implementation_strategy/2026-07-06_node_image_externalization_plan.md | current | no | 12 | 6 | 5 | 1 |
| mere_docs/implementation_strategy/2026-07-08_portable_tiles_plan.md | historical-unmarked | no | 5 | 2 | 3 | 0 |
| mere_docs/implementation_strategy/2026-07-09_virtualized_editor_plan.md | current | no | 9 | 6 | 3 | 0 |
| mere_docs/implementation_strategy/2026-07-22_graphshell_remote_projection_host_plan.md | current | no | 10 | 8 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-07_knot_publishing_protocol_plan.md | current | no | 10 | 8 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-09_browser_model_ceiling_probe_plan.md | current | yes | 10 | 9 | 1 | 0 |
| **Totals** |  |  | **132** | **86** | **43** | **3** |

**Totals: 12 docs, 132 claims checked (86 holds, 43 stale, 3 unverifiable), 22 contradictions; 9 status lines wrong.**

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

Checked directly in this session: athanor defines `propose_image_gc` and
`apply_image_gc` at the base (node image externalization, the verifier's
refutation); Cambium's `multi.rs` defines `push_forest_projection` (portable
tiles). The verifier refuted one omission (Phase 5's image GC landed in
athanor) and found the forest DOM's retirement overstated (only genet-layout's
spike went; the topology is live in Cambium and Turnstone), two misattributed
genet commits, the D2 probe's completeness overstated twice, the theme plan's
Tabard point narrower than drafted, and the knot "one immutable revision" note
checkable and stale; those records and their counts are corrected.

## mere_docs/implementation_strategy/2026-06-30_runtime_mod_authoring_loop_plan.md

- disposition: historical-unmarked
- status line: "Status: Planned." — accurate: no
- claims checked: 12 — holds: 6, stale: 6, unverifiable: 0

### Stale claims

- "`crates/meerkat/src/shell_eval.rs` is the privileged omnibar Rhai lane". The path is marked as a historical citation, but the sentence is in the present tense. `crates/meerkat` is absent at the base; Meerkat was removed in c5f01064 on 2026-07-18.
- "reads a frozen `ShellContext` and emits a `ShellOutcome`". `git grep` over crates, ports and apps finds no match at 535bca11.
- "registers one Rhai binding per `Command::ALL` verb". There is no `Command::ALL` in the tree.
- The `attach_script` / `script_event` / `detach_script` shell requests. Only a doc comment survives (crates/system/pandect/src/script_bindings_store.rs:12). The other two have no match.
- "Installed mods under `<mere_root>/mods/` can auto-bind as DocumentScripts". The helper `discover_wasm_mods_in_dir` (crates/system/registry/src/mod_loader/loader/free_fns.rs:149) has no production caller in Mere or Turnstone; it is re-exported at mod_loader.rs:57 and called only by tests (loader/tests.rs:108, :139). Nothing names a `<mere_root>/mods/` path.
- The Meerkat pane and the done-conditions ("without restarting Meerkat", "from one Meerkat pane") cannot be reached: the host is gone. None of the plan's objects exist (`install_mod`, `CommandModManifest`, `ModProject`, `meerkat.mod.*`). The only near match is the unrelated `unload_mod_with` in the registry mod_loader.

### Contradictions

- DOC_README.md:138 still describes "a Meerkat pane for model-assisted edit/check/build/load/run/reload" as live design.
- Two active plans cite this one as the authoring ergonomics they inherit: 2026-08-13_graph_behaviors_plan.md:23 and 2026-07-17_participant_gate_packs_plan.md:17. Both point at a plan whose host is gone.

### Recommended action

- Date the status line and mark the Meerkat grounding as historical (Meerkat removed 2026-07-18, c5f01064). Either retarget the Rhai/omnibar half and the pane onto a living host or archive the plan (see Forks). The Wasm half's grounding still holds.

### Notes

Checked paths at the base: crates/meerkat (absent), crates/script/rhai/src/lib.rs, crates/script/document-host (capabilities.rs:106 for net.fetch denied by default; net.rs), crates/script/wit/world.wit (two worlds, app-core and document-core). All four Related links resolve. The Wasm-side claims hold: DocumentScript, import gating before instantiation, and the follow-ons plan carrying the settings-lane grant UI (archive 2026-07-03, follow-ons plan lines 248, 275, 316).

## mere_docs/implementation_strategy/2026-07-02_meerkat_promotion_pass_plan.md

- disposition: historical-marked
- status line: "Status: P1/P2/P3-first-slice/P4/P5/P6/P7 promoted. P8's input-snapshot seam, first two domain moves, roster's pure helper layer, roster's explicit builder contract, and graphlet builder contract landed. `pane_data.rs` now has explicit local input structs and pure builders too, but the store-backed pane rows still remain in `meerkat`. The follow-up review on 2026-07-02 tightened P3/P4/P8 before code move. A later implementation pass on 2026-07-02 landed P1/P2/P5/P6/P7 and then re-ran the P3 seam check. The compile witness for P1/P2 is still blocked by unrelated workspace breakage (`session-runtime` and earlier `graph-kernel` errors), but the nearby imports settled enough to confirm the wider P3 contract shape below." — accurate: no
- claims checked: 12 — holds: 9, stale: 3, unverifiable: 0

### Stale claims

- "the store-backed pane rows still remain in `meerkat`". Meerkat was removed (c5f01064, 2026-07-18).
- "The compile witness for P1/P2 is still blocked by ... `session-runtime` ... errors". `session-runtime` was renamed `pandect` (441e70f0, 2026-08-15), and the crate being blocked no longer exists.
- P7: "theme editing's core into register-theme" is the plan's last word and carries no historical marker. register-theme was folded into `mere-registry` (3430ba2b, 2026-09-23), and the theme tree then moved to ports/tabard/src/theme/ (7f133433, 2026-09-24).

### Contradictions

- The status line is in the present tense, but the same document's 2026-09-05 note (line 22) says the mapping and compile blockers are "receipt-era context" and that current ownership follows the platform-boundary plan. That plan is itself marked "Complete 2026-09-06".

### Recommended action

- Rewrite the status line as dated history, for example "Historical: P1–P7 and the P8 slices landed 2026-07-02/03; Meerkat removed 2026-07-18; current ownership per the platform-boundary plan". Then decide whether to archive (see Forks).

### Notes

These exist at the base: crates/system/fetch, crates/crawl, crates/system/content-contract (lib.rs:106 and :178 for the message enums), crates/import/src/web_clip.rs, crates/shell/chrome/src/nav.rs (:27, :151) and suggest.rs, and crates/domain/apparatus. gloss and roster now live in crates/mere/src/gloss.rs (build_outline_snapshot at :105) and roster.rs (NodeRowInput at :333), folded in by 61894570 on 2026-09-23; those paths are marked historical inline, so they were not counted stale. Subgraph is crates/mere/src/subgraph.rs. All Relates-to links resolve.

## mere_docs/implementation_strategy/2026-07-03_archived_plan_tails_plan.md

- disposition: current
- status line: "Status: backlog holder. Each item below was explicitly deferred by a plan that is otherwise complete and now lives under [`archive_docs/`](../../archive_docs/), in the checkpoint folder named by the section it sits under. None of these gate anything today; pick up when the relevant lane is quiet. Items already tracked by an active plan are *not* repeated here." — accurate: yes
- claims checked: 23 — holds: 15, stale: 8, unverifiable: 0

### Stale claims

- Lane0 item 6: "Barnes-Hut is one `add_force` line from live ... unwired at seiche_bridge.rs:58". It is live as `PhysicsLaw::Charge` ("charge.barnes-hut"; crates/canvas/pictograph/src/canvas/physics_catalog.rs:149 and :1110-1114). It landed in 37477457 on 2026-09-02, recorded in the active 2026-09-02_physics_catalog_plan.md:131.
- E3: "today any approved-capability `.wasm` in `mods/` auto-attaches". `discover_wasm_mods_in_dir` (free_fns.rs:149) has no production caller in Mere or Turnstone (tests only).
- The Meerkat-era tails in the 2026-07-03 and 2026-07-04 sections all name code that no longer exists in Mere, Turnstone or Genet components: paste into the find field (`handle_clipboard_shortcut`), the `submenu_hover` and other context-submenu items, the `favicon_data_uri` rename, and the `SecondaryForward`/`ProducerSurface` rename.
- "rkyv compaction of graph engrams ... for `GraphEngram`". `GraphEngram` is now `GraphCodicil` (c51b9704, 2026-08-31; crates/system/pandect/src/graph_codicil.rs:62).
- "Splitting `session-runtime`'s mixed concerns". The crate is now `pandect` (441e70f0, 2026-08-15).
- "graphlet_wiring (cross-plan leftovers owned by the active relational_browse plan)". That plan is archived (archive_docs/2026-08-06_completed_plans/2026-06-23_relational_browse_graphlet_plan.md).
- Roster model locations, a minor inaccuracy: `SubgraphSpec` is `forme::SubgraphSpec` (crates/forme/forme/src/subgraph.rs:69), not part of `mere::roster`, and `EdgeFamily` lives in graph-kernel (edge_taxonomy.rs:33), not canvas.

### Contradictions

- The Barnes-Hut tail (dated 2026-09-02) contradicts the active physics catalog plan, which landed the Charge law on the same day.
- The status says items tracked by an active plan are not repeated here, yet the relational_browse line points at an archived plan.
- The DOC_README.md:200 entry covers only the 2026-07-03 and 2026-07-04 passes. It omits the 2026-08-06 and 2026-09-02 sections.

### Recommended action

- Mark the Barnes-Hut item done (physics catalog P1, 37477457).
- Rename the GraphEngram and session-runtime references.
- Repoint graphlet_wiring at its archive, or extract its leftovers.
- Retire or mark the Meerkat-subject tails in the 2026-07-03/04 sections the way the 2026-09-02 pass treated deleted subjects.
- Widen the DOC_README entry to all four passes.

### Notes

These hold at the base:

- workbench.rs:182 and :189
- the five forme submodules
- PersonaSettings `menu_actions` and `command_usage`, now at persona_settings_store.rs:45 and :52, with no consumers
- SIZE_TIERS, now at canvas.rs:220
- spawn_named at armillary actor.rs:128
- ToastSpec at frame_model.rs:407-416
- servitor cap.rs:159 and grant.rs
- the lease.rs:25 kith note
- `Fetched` having no HTTP status (fetch/src/lib.rs:81)
- `.cwasm` support only in the loader
- merge_snapshots remaining in pandect
- the parallelism brief still lacking its note
- genet docs/2026-07-03_shell_paint_emission_raster_plan.md
- every cited link

Stickleback is now 0.1.1 and is still in-repo.

## mere_docs/implementation_strategy/2026-07-05_inference_provider_plan.md

- disposition: current
- status line: "Status: P0 (seam + stub), P1 (own decoder body incl. seeded temperature/top-p sampling; validated on the real TinyLlama checkpoint, 9.95 tok/s on wgpu vs 0.09 on ndarray), P2 (eidetic loading; corridor proven transparent), P3 (actor with cancellation), and the meerkat host wiring (`>ask` omnibar verb) all landed. P4's native measurement is landed; its headed-browser half is scoped in the D2 browser model ceiling probe." — accurate: no
- claims checked: 10 — holds: 7, stale: 3, unverifiable: 0

### Stale claims

- "its headed-browser half is scoped in the D2 browser model ceiling probe". D2 ran: that plan's status records D2a, the MiniLM D2b row, the D2c embedding matrix and one D2c decoder row complete (upper boundaries and GPU release unmeasured), with commits 3bdb66fc, dd215ebd and 45327c30 (2026-08-22), all ancestors of the base.
- "the meerkat host wiring (`>ask` omnibar verb) ... landed" is presented as current. Meerkat was removed (c5f01064). No host in Mere or Turnstone calls `spawn_inference_actor`; only the distillery probes use `DecoderProvider` directly.
- The plan never records the move into ESP. It names `crates/intel/infer` (marked historical) and `CannedProvider` throughout. The code is `esp::infer` (crates/intel/esp/src/infer/), and the stub is `StubInferenceProvider` (stub.rs:21), consolidated in 1283b4a8 on 2026-08-09.

### Contradictions

- DOC_README.md:359 says "implementation now in `esp::infer`" and "StubInferenceProvider". The plan body still says CannedProvider and the old crate.
- The status calls the browser half "scoped", but 2026-08-09_browser_model_ceiling_probe_plan.md reports D2a, the MiniLM D2b row, the D2c embedding matrix and one D2c decoder row complete.

### Recommended action

- Update the status: P0–P3 and P4-native landed in esp::infer; P4-browser extracted to D2, which ran through D2c on 2026-08-22; the Meerkat `>ask` host was removed with Meerkat on 2026-07-18.
- Note the CannedProvider → StubInferenceProvider rename.
- Consider archiving (see Forks).

### Notes

At the base: provider.rs:19, :37, :93, :135; decoder/provider.rs:35; decoder/mod.rs:91 (load_wgpu_provider); decoder/sample.rs; actor.rs:38 and :68, with Cancel documented at :16; tests/eidetic_corridor.rs and tests/tinyllama_real.rs. All five Related links resolve.

## mere_docs/implementation_strategy/2026-07-05_overlay_roots_and_ua_widgets_plan.md

- disposition: historical-unmarked
- status line: "Status: design/direction (with Mark). No code yet. Two directives, one substrate; this plan fixes the architecture and the build order." — accurate: no
- claims checked: 9 — holds: 4, stale: 5, unverifiable: 0

### Stale claims

- "No code yet". The plan's own Progress records P0 (both slots), P2 and P1 landing on 2026-07-05.
- The landed code has since been deleted with its substrate. Genet removed genet-layout's highlights.rs and overlays.rs, with Stylo and genet-layout, in 55c05d11759 (2026-08-21) (868c7b0cd07 removed unrelated xilem-serval files, now in Cambium), and the Meerkat P1 side went with c5f01064. There is no HighlightRegistry, OverlayRegistry, set_overlay or ContentLayout in Mere crates/ports or Genet components.
- "`BoxTree::graft_subtree` keeps a retained layout emittable". There is no match in Genet HEAD.
- "Stylo implements shadow trees; genet's cascade rides stylo". Stylo was retired in 55c05d11759.
- "The DocumentScript mirror seam (`handlers.rs:86`) is the enforcement point" points at Meerkat, which is gone.

### Contradictions

- The status line ("No code yet") contradicts the plan's own Progress entries.
- DOC_README.md:199 repeats "design/direction".

### Recommended action

- Replace the status with: historical, P0/P1/P2 landed 2026-07-05, substrate retired 2026-07-14 to 2026-08-21.
- Then archive with the UA-widget and feature-wave tails extracted, or rewrite onto Cambium and Genet's spec work (see Forks).

### Notes

These hold under new names: the control views (`xilem-serval` became Cambium; crates/cambium/cambium/src/controls/field.rs:124, toggle.rs:64, select.rs:76, slider.rs:68) and `host_pool` with its splice-safety test (crates/cambium/cambium/src/tags.rs:123 and :269). All five Mere cross-ref links and genet docs/2026-07-02_dom_mutation_capture_replay_plan.md resolve. Genet's only related live work is design_docs/2026-09-07_shadow_dom_plan.md (landed), which does not take up this plan.

## mere_docs/implementation_strategy/2026-07-05_theme_modes_plan.md

- disposition: current
- status line: "Status: T1–T5 implemented 2026-07-05 (see Progress; T5 shipped the declarative lane, rhai graduation open). From Mark's theme-model decision (2026-07-05), unblocking the W3C adoption plan's P3 host half." — accurate: yes
- claims checked: 10 — holds: 6, stale: 2, unverifiable: 2

### Stale claims

- The 2026-09-13 scope says document-lanes' smolweb.rs "accepts `SmolwebTheme::App` ... protocol parsers need no dependency on Tabard". `SmolwebTheme` is now defined in `tabard::smolweb` and re-exported by document-lanes (crates/system/document-lanes/src/smolweb.rs:36; d16f055f, 2026-09-24): the seam's type is now Tabard's. "Protocol parsers need no dependency on Tabard" still holds, as nematic has no tabard dependency.
- The Related line and the Engine-mapping section rest on `IncrementalLayout::set_prefers_color_scheme` and the Stylo servo Device. Neither exists in Mere or Genet components; genet-layout and Stylo were retired in 55c05d11759. The 2026-09-13 note covers only the July receipts, not this mechanism.

### Contradictions

- DOC_README.md:68 omits the 2026-09-13 Tabard adapter scope and its open items.

### Recommended action

- Mark the Engine-mapping section historical.
- Note that SmolwebTheme moved into tabard::smolweb (d16f055f).
- Add the Tabard scope to the DOC_README entry.

### Notes

The successor code holds:

- tabard lib.rs:69 (Theme), :92 (mode_sheets) and :181 (lagrange_palette_txt)
- StockVersionIgnored (:418)
- tests/fixtures/lagrange_palette.txt
- theme/registry.rs:152 (Mode)
- theme/mode_calc.rs:33 (CustomModeDef; Rhai named only as a future lane at :19)
- tinct lib.rs:211 (ModeProfile)
- Knot apps/desktop/src/appearance.rs:7, which derives its own Tinct palette

The fidelity-plan anchor resolves. Unverifiable: whether native reader adapter 1 is closed (Turnstone now consumes `tabard::theme` ThemeChoice and Mode, src/settings_provider.rs:22, but no reader receipt could be checked read-only), and the stock-Lagrange load/restart gate.

## mere_docs/implementation_strategy/2026-07-06_node_image_externalization_plan.md

- disposition: current
- status line: "Status: Planned. Motivated by a measurement: the petgraph-RDF plan's Phase 4 footprint probe (`crates/probes/rdf-kernel-footprint/` *(historical citation)* <!-- doc-audit: historical-path -->, see that plan's Phase 4 gate note) found that at 50k nodes the kernel's live heap is 64% inline image bytes (`Node::thumbnail_png` + `Node::favicon_rgba`), dwarfing every other category and dwarfing what a term dictionary could reclaim (~1%). This plan moves that imagery out of the kernel into the durable content-addressed blob store the memory model already runs, leaving a small reference in the node. Target: live graph footprint at 50k nodes drops from ~553 MiB to the low 200s, with a bounded render-side image cache in place of every node holding its pixels forever." — accurate: no
- claims checked: 12 — holds: 6, stale: 5, unverifiable: 1

### Stale claims

- "Planned." Progress (2026-07-26) records Phases 2 and 3 landed, and the tree confirms it: crates/system/pandect/src/image_store.rs:39, :52, :61, :110; graph-kernel node.rs:56 and persistence.rs:169 (images map) and :263 (legacy_image_count); apply.rs:239 (SetNodeImage).
- "Phase 4 is seamed, not finished ... The map is unbounded". A byte-bounded LRU exists: `ResolvedImageCache` (crates/canvas/pictograph/src/canvas/resolved_image_cache.rs, 64 MiB default; canvas.rs:417; set_resolved_image_cache_limit_bytes at nodes.rs:65). It was added in 6d1187a7 on 2026-07-27, a commit whose subject is unrelated.
- "Not done: turnstone's ~28 read sites (... its build is broken until they are updated)". Turnstone HEAD uses the new model: src/session.rs:616 (ImageRole), src/browse.rs:582 (register_resolved_image) and :1062 (.favicon()).
- The Phase 4 done-condition asks for "the meerkat render-perf harness". That harness went with Meerkat (c5f01064).
- "Not done: … phase 5's orphan GC" (:391). Athanor carries the Phase 5 pass: `propose_image_gc` and `apply_image_gc` (ports/distillery/athanor/src/lib.rs:182, :198) and per-role reference forgetting (:232, :263), with tests at :574 and :635 that call `stored_image_hexes`; added in 6d1187a7 (2026-07-27). No host invokes it yet.

### Contradictions

- The status ("Planned.") contradicts the plan's own 2026-07-26 Progress.
- DOC_README.md:358 says "planned, not started".

### Recommended action

- Set the status to: in progress, P1–P5 landed in library code (P4's bounded LRU and P5's athanor orphan GC in 6d1187a7, 2026-07-27; P5 not yet run by any host); P6 re-measure open.
- Restate the P4 done-condition against a living harness.
- Fix the DOC_README entry.

### Notes

P5's input helper is `stored_image_hexes` (image_store.rs:80), used by athanor's GC pass. crates/probes/rdf-kernel-footprint is untracked at the base; the plan marks it as a planned target. All seven cross-ref links resolve. Unverifiable: favicon capture's RGBA→PNG-at-store-time write site.

## mere_docs/implementation_strategy/2026-07-08_portable_tiles_plan.md

- disposition: historical-unmarked
- status line: "Status: Planning, downstream of the forest dom. Same-document cross-window moves are blocked on step 3 ([forest_dom_plan](../../archive_docs/2026-08-06_completed_plans/2026-07-08_forest_dom_plan.md)); this plan captures the target + the open questions so the shape is settled before step 3 lands." — accurate: no
- claims checked: 5 — holds: 2, stale: 3, unverifiable: 0

### Stale claims

- "blocked on step 3". The forest-dom plan is "LANDED 2026-07-18" (archived plan, line 4), and only genet-layout's ForestDom spike was retired (55c05d11759; "The retired `ForestDom` spike", crates/cambium/cambium/src/multi.rs:34): the forest-dom topology is live as Cambium's `GenetMultiRunner::push_forest_projection` (multi.rs:132), used by cambium-rootstock (multi_host.rs:75, :430) and Turnstone (chrome_view.rs:651, :665). So P1/P2 may be unblocked rather than moot.
- "This plan is the meerkat consumer", together with the pelt/`WindowView` lane split. Meerkat was removed (c5f01064).
- Progress says "none yet ... P0 ... can start". Turnstone delivers P0's outcome: `TearOutActivePane` keeps "its DOM, widget state, scroll ... untouched by the move" (turnstone src/action.rs:233-238), with the branch arm `TearOutTile` at :476-480.

### Contradictions

- The status says "blocked on step 3", while its own link points into archive_docs/2026-08-06_completed_plans.
- DOC_README.md:214 says P1/P2 "wait on the forest_dom_plan".

### Recommended action

- Restate the plan: its meerkat consumer is gone and P0's outcome is realized in Turnstone's tear-out (`TearOutActivePane`, and `tear_out_tile` at src/app/pane_arms.rs:345 with member-keyed content sessions at src/shell/lens.rs:232-250), while step 3 is live as Cambium's forest projection; whether P1/P2 proceed on it or the plan archives is a fork.

### Notes

These hold: PortableKeyed and the nursery survive in Cambium (context.rs:126 and :244; frisket.rs:39 keys `Slot::View` tiles by TileId). The four links resolve, including genet docs/2026-07-05_movebefore_dom_standard_plan.md.

## mere_docs/implementation_strategy/2026-07-09_virtualized_editor_plan.md

- disposition: current
- status line: "Status: partially implemented and next shared-editor work scoped. The rung-3 *infrastructure* (arrangement leaf + `VirtualWindow`) is built in Genet, and Cambium now has a read-only fold projection. This plan covers the coordinate map and projected focused-text path required before folding can become a live editor, plus the later gutter and large-file work. Grew out of the [djot editor plan](../../archive_docs/2026-08-06_completed_plans/2026-06-24_djot_editor_knot_nodes_plan.md) Phase 3 (folds), which is blocked on this." — accurate: no
- claims checked: 9 — holds: 6, stale: 3, unverifiable: 0

### Stale claims

- "infrastructure ... is built in Genet". It lives in Mere: crates/cambium/cambium/src/arrangement.rs:27, :41, :62 (placed, placed_with, arrangement) and crates/cambium/sprigging/src/arrange.rs:52 (VirtualWindow, formerly chisel). Cambium and Sprigging left Genet in ce79fd44a4d (2026-09-03).
- The 2026-09-13 Progress entry says the projection does not "provide a gutter/control", and the status calls the gutter "later". 0d341dee (2026-09-25) added `FoldProjection::lines()`, `rows(gutter)` and `FOLD_ROWS_CSS` (fold_projection.rs:196, :251, :37), described in its commit as "the gutter of the virtualized editor plan's P2, without the virtualization". Knot's folded source tile puts its fold controls there (knot-editor apps/desktop/src/document_folding.rs:12, :31, :253).
- The "Why" and P1 framing describe a textarea laid out by genet-layout, "No meerkat consumer", and P1 as the "first meerkat arrangement consumer". genet-layout and Meerkat are both gone; the textarea is Cambium's `styled_textarea` (styled_field.rs:190).

### Contradictions

- DOC_README.md:69 says "design, pre-build ... Genet's existing virtual-window infrastructure", which contradicts the status ("partially implemented") and the code's location.

### Recommended action

- Restate the infrastructure location (Cambium arrangement plus Sprigging VirtualWindow, both in Mere).
- Add a Progress entry for 0d341dee (P2 gutter rows without virtualization).
- Update the DOC_README entry.

### Notes

These hold: fold_projection.rs:125 (2676c5e9); EditableFoldProjection and the Rootstock focused-text projection are still absent (rootstock has only an a11y projection, host.rs:770); illume tree.rs:127 (folds); Knot's read-only fold reading. The djot plan archive link, the illume lexer plan and genet docs/2026-07-08_chisel_widget_catalog.md resolve.

## mere_docs/implementation_strategy/2026-07-22_graphshell_remote_projection_host_plan.md

- disposition: current
- status line: "Status: the local G0 boundary is sealed; G1's loopback presentation, G2's diff/resume/persistence, and G3's real Turnstone endpoint proofs are complete as of 2026-07-22. The Graphshell workspace is published on the existing `mark-ik/graphshell` repository; its retired browser donor remains available in the same Git history. G4, the already-proven Isometry projection, is next. Graphshell is ruled as the Merely family's remote projection host. It is neither the projection engine nor Mere's internal chrome layer. This plan remains the cross-repository roadmap; Graphshell's README owns the live package boundary." — accurate: no
- claims checked: 10 — holds: 8, stale: 2, unverifiable: 0

### Stale claims

- "G4, the already-proven Isometry projection, is next". §8 records G4 "Implemented locally 2026-07-22" (line 494; ports/graphshell/src/bin/g4_sessions.rs). G5a–G5f are complete as of 2026-07-29 (lines 700-949; ports/graphshell/docs/2026-07-29_h6a_*.md and h6b_*.md). The browser carrier profile was physically proven on 2026-09-02 (line 609). Next is G6.
- "The Graphshell workspace is published on the existing `mark-ik/graphshell` repository". The crates live in Mere (crates/chirograph, crates/graphshell/{graphshell-client,graphshell-endpoint}, ports/graphshell), per the 2026-07-23 consolidation.

### Contradictions

- The status line contradicts the header's own "Superseded in part 2026-07-23 ... the five crates move into the mere repository".
- The status line contradicts §8, where G4 and G5 are recorded as done.

### Recommended action

- Update the status to: G0–G5 complete (G5f 2026-07-29), browser carrier profile proven 2026-09-02, G6 next; crates in Mere since 2026-07-23; G8 folded into H4 of the reference host plan.

### Notes

These hold: notochord handshake.rs:312 (AdmittedPrincipal), graphshell admission.rs:71 and :111, carrier.rs:141, profile.rs:89, chirograph lib.rs:1170 (SessionStatus), and receipts/g1_loopback.html. All companion and amendment links resolve (reference host, repo consolidation, prior-art brief, projection proofs, participant gate, layer map, Murm plan, WebRTC carrier plan and probe). No active doc records G6 or G7 as done.

## mere_docs/implementation_strategy/2026-08-07_knot_publishing_protocol_plan.md

- disposition: current
- status line: "Status: Phase A implemented and physically receipted, including a public-client renewal on 2026-08-19. Direction remains A then B (§4). The existing K2 Graphshell projection rehearsal is useful precedent, not this protocol. Phase B has not begun: the Phase-A grammar needs real product use and an intended independent implementer before it is promoted into a compatibility commitment." — accurate: no
- claims checked: 10 — holds: 8, stale: 2, unverifiable: 0

### Stale claims

- "Phase B has not begun" (status) and "§9 Status: Not entered". Mere's own active 2026-08-08_knot_mark_read_adapter.md (DOC_README.md:72) says it "records the Phase B choice from 2026-08-07_knot_publishing_protocol_plan.md". It carries the compatibility table, choosing option 1, a bounded Mark read adapter, and reports it implemented: knot-editor crates/knot-editor/src/mark.rs:36 (`MARK_ALPN`).

- The repository note's "consumed by Djinn and Turnstone from one immutable revision": three knot-editor revisions are pinned (Djinn 562353aa with knot-site ea3e99ef; Turnstone 3dfb70b0).
### Contradictions

- This plan's §9 and status contradict the Mark adapter doc on whether Phase B's decision gate has been passed.
- Duplicate document: Knot Editor carries its own copy, knot-editor/design_docs/2026-08-07_knot_publishing_protocol_plan.md. The two differ only in the repository note and the audit markers, and Knot's copy calls itself "a historical integration record". This bears on DOC_POLICY §2 (shared material lives once) and §4 (docs live with the subject's repo).

### Recommended action

- Reconcile the Phase B wording with the Mark adapter, either "decision gate passed (adapter, 2026-08-08); spec and knot-protocol not entered" or by marking the adapter out of sequence.
- Choose one canonical copy and cite it from the other (see Forks); the Mark adapter doc is duplicated in knot-editor/design_docs too.

### Notes

These hold: notochord authority.rs:32 (RetainedAuthority), consumed at graphshell lifecycle.rs:46 and :109; 162be7a9 is an ancestor of the base (2026-08-12); knot-editor crates/knot-editor/src/publish.rs, publish_wire.rs, publish_carrier.rs, publish_host.rs, examples/knot_publish_peer.rs, and publish_client.rs:98 (fetch_published_document); Knot's 2026-08-19 receipt (b54ba97); Turnstone's PUBLISHING pane (src/app/tests.rs:3074); murm transport lib.rs:56 (noise) and notochord.rs:66 (into_session); no knot-protocol crate anywhere. "Consumed by Djinn and Turnstone from one immutable revision" is stale: Djinn uses the workspace knot-editor at 562353aa (Cargo.toml:243) plus knot-site at ea3e99ef, and Turnstone pins 3dfb70b0 (Cargo.toml:102).

## mere_docs/implementation_strategy/2026-08-09_browser_model_ceiling_probe_plan.md

- disposition: current
- status line: "Status: D2a, the MiniLM D2b embedding row, D2c's configured embedding matrix, and one exact D2c decoder row are complete. Headed Chromium passes four cold/cancel/warm embedding rows from a 34.8 MB F16 BGE artifact through a 438.0 MB F32 E5-base artifact, plus streamed SmolLM2 generation, cooperative token-boundary cancellation, explicit `GPUDevice.destroy()`, and exact recovery in a fresh worker from a 269.1 MB BF16 artifact. The upper model boundaries and physical GPU-allocation release remain unmeasured. This track remains independent of the personal mesh and Burn Remote." — accurate: yes
- claims checked: 10 — holds: 9, stale: 1, unverifiable: 0

### Stale claims

- 2026-08-22: "Mere carries that narrow guard in vendored `burn-cubecl` plus a logical-allocation identity helper in its existing `cubecl-runtime` patch. Burn and CubeCL main still contain the susceptible source path". At the base, Cargo.toml:678-681 says the burn-cubecl selector "was retired under ruling 410 ... current consumers select the registry crate", and Cargo.toml:673-677 says the cubecl-runtime patch source "is pristine upstream". The burn 0.22 migration plan §13.28 (2026-09-29) records the retirement after upstream pre.4 passed.

### Contradictions

- DOC_README.md:360 says "scoped, independent evidence lane", which contradicts the status (D2a–D2c complete).
- DOC_README.md:437 (the receipt entry) gives only "BrowserWebGpu failed the numerical gate", which this plan's 2026-08-22 recovery overtakes.

### Recommended action

- Add a dated note that the guard was retired under ruling 410 (burn migration §13.28; the four rows and the SmolLM2 row re-passed on pre.4).
- Update the DOC_README entry.
- Decide on archiving (see Forks).

### Notes

3bdb66fc, dd215ebd and 45327c30 are ancestors of the base. These exist: ports/distillery/probe, the 2026-08-21 receipt, the esp_consolidation plan, the intel feature_target_matrix, and ports/graphshell/docs/2026-08-06_browser_storage_persistence_receipt.md. The ESP fixes are present: split-half rotary (attention.rs:31), async readback (generate.rs:54), BF16/F16 decode (tensors.rs:12-29).
