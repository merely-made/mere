# Batch 40 — S14 pass, phase B2 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-06-13_scriptable_field_regions_plan.md | current | no | 21 | 16 | 5 | 0 |
| mere_docs/implementation_strategy/2026-06-17_unified_document_host_plan.md | historical-marked | yes | 10 | 6 | 3 | 1 |
| mere_docs/implementation_strategy/2026-06-18_node_representation_arrangement_plan.md | historical-marked | yes | 7 | 6 | 1 | 0 |
| mere_docs/implementation_strategy/2026-06-23_gloss_outline_lens_plan.md | current | no | 11 | 6 | 5 | 0 |
| mere_docs/implementation_strategy/2026-06-23_native_session_store_plan.md | current | no | 8 | 6 | 2 | 0 |
| mere_docs/implementation_strategy/2026-06-23_orrery_custom_layout_element_plan.md | historical-unmarked | yes | 9 | 5 | 4 | 0 |
| mere_docs/implementation_strategy/2026-06-24_alembic_implementation_plan.md | current | no | 12 | 6 | 4 | 2 |
| mere_docs/implementation_strategy/2026-06-24_meaningful_physics_signals_plan.md | current | yes | 13 | 10 | 3 | 0 |
| mere_docs/implementation_strategy/2026-06-24_tearout_gestures_plan.md | historical-marked | no | 7 | 4 | 3 | 0 |
| mere_docs/implementation_strategy/2026-06-25_athanor_steady_heat_actor_plan.md | historical-unmarked | no | 6 | 3 | 3 | 0 |
| mere_docs/implementation_strategy/2026-06-25_xilem_serval_control_adoption_plan.md | historical-unmarked | no | 6 | 3 | 3 | 0 |
| mere_docs/implementation_strategy/2026-06-26_illume_text_lexer_plan.md | historical-unmarked | no | 12 | 6 | 4 | 2 |
| **Totals** |  |  | **122** | **77** | **40** | **5** |

**Totals: 12 docs, 122 claims checked (77 holds, 40 stale, 5 unverifiable), 20 contradictions; 8 status lines wrong.**

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

Checked directly in this session: `set_physics_damping` is at pictograph
`canvas/input.rs:706` and `540935f2`, `50c23945` and `af775f24` are ancestors
of the base (field regions); `JarCookieProvider` appears only in documents,
never in code (native session store); illume's manifest declares
`license = "MPL-2.0"` (illume). The verifier refuted nothing and found 7 items
true only in part (the unified host's two citations, the gloss plan's scope
lens, genet's two test-only `CookieProvider` impls, the alembic plan's slice D,
which is half landed, and the physics-signals sources, two of which have live
equivalents); those records are corrected, and four status quotes it found
cut short are given in full.

## mere_docs/implementation_strategy/2026-06-13_scriptable_field_regions_plan.md

- disposition: current
- status line: "Status: partially implemented: movable and resizable field regions landed; physics-setting and further scripted-region work remains deferred." — accurate: no
- claims checked: 21 — holds: 16, stale: 5, unverifiable: 0

### Stale claims

- The status says physics settings remain deferred. They landed on 2026-06-14, after the plan's last entry (`9d46d934`):
  - runtime damping `540935f2` and `50c23945`, now `Canvas::set_physics_damping` (pictograph `input.rs:701-707`), driven live by Turnstone `src/app/session_lifecycle.rs:1159` and `ports/graphshell/src/canvas_physics.rs:934`;
  - per-field strength `af775f24`, now `set_field_strength` (`input.rs:603-631`) and kernel `set_field_coupling_strength` (`field_ops.rs:101-112`).
- The last Progress entry says "Still deferred to the physics menu: per-field strength … and the inertia/damping toggle". Same evidence. Per-field response (gather/repel/wall/dampen) is genuinely still absent.
- The line-3 banner says "the graphlets crate is crates/graph/subgraph". It was folded into `mere` as `crates/mere/src/subgraph.rs` (`61894570`), and `TERMINOLOGY.md:256` already says so.
- "The `orrery/arrangements` family … already arranges node subsets", plus the Design substrate. No arrangements package exists; the adapters are `crates/canvas/cartography/src/adapters/`.
- Findings and Design describe the rule script extending aether `FieldProjection::commit_to_graph` and gyre `CouplingForce::from_coupling`, in present tense. `commit_to_graph` left: `numen/src/projection.rs:63-67`. `from_coupling` became a pictograph bridge: `seiche_bridge.rs:86`. The historical markers cover only the links, not the prose.

### Contradictions

- `DOC_README.md:226` indexes the plan as "**planned**", against the status line and the Progress log.
- Line 16 assigns the scene-wide arrangement choice to the node-representation plan. That plan is superseded, and its arrangement axis went to the graph-signals plan, now archived.

### Recommended action

- Write a dated status:
  - landed: move/resize, the force well, rebuild-on-mutation, and physics settings (damping live; per-field strength API present but with no host driver since meerkat);
  - no host calls `add_field_at` (test-only: `node_minting.rs:174`), so a user cannot place a field today;
  - remaining: per-field response, and P2-P4.
- Fix the banner path.
- Mark or rename the gyre/aether/orrery substrate names.
- Update `DOC_README.md:226`.

### Notes

Checked:
- Commits `3c62b15 80c05fb 48067be b2ecaaf d43f3c1 bfd753a fada862 7445e70`: all on main.
- Pictograph `fields.rs` (`active_field` :98, `toggle_field_visible` :163, `center_on_field` :178, move/resize :21-344), `input.rs:557` `add_field_at`, `DEFAULT_FIELD_STRENGTH` 5000 (:38), `lifecycle.rs:357` `reconcile_derived`.
- Kernel `mod.rs:301-302` and `field_ops.rs`. numen `field_ast` / `rhai_bindings` (:115, :188). forme `EdgeProjectionSpec`. `PersistedField*`.
- No `FieldContext` / `show_edges` / `arrange` binding exists.

## mere_docs/implementation_strategy/2026-06-17_unified_document_host_plan.md

- disposition: historical-marked
- status line: "Status: closed 2026-06-23: Phase-1 core and its four pressing slices landed; Phase-2 tails are explicitly re-homed to successor plans." — accurate: yes
- claims checked: 10 — holds: 6, stale: 3, unverifiable: 1

### Stale claims

- Lines 6-7 and the closing entry call the plan "the still-current foundational record of the Phase-1 consolidation + orrery-as-element architecture". That architecture was meerkat's, deleted in `c5f01064`; `orrery_a11y_tree` has no code hit outside a comment in `crates/mere/src/glossary.rs`. Nothing in the tree implements the one-shell-document or the orrery element.
- The rename banner calls `OrreryGnode` / `gnode_view` / `.gnode` / `render_gnodes_as_dom` the "current names". `OrreryGnode` and `gnode_view` have 0 hits. Only `.gnode` (pictograph `canvas.rs:313`) and `render_gnodes_as_dom` (`canvas.rs:701`, `cartography.rs:139`) survive.
- The 2026-06-23 entry's last word on `mere-orrery`: "Left in place pending Mark's call on the crate's fate". It was renamed `glossary` on 2026-06-26 (`8d55f96`, `ffb1b5e`; `glossary.rs:18` and the gloss plan date it 06-23) and later folded into `mere` (`crates/mere/src/glossary.rs`, `61894570`).

### Contradictions

- `DOC_README.md:231` repeats the "still-current foundational record" framing.
- The thread map sends slice 5 to `layout_phase_split_probe_plan`, which sits in `archive_docs/2026-09-02_retired_plans/`: retired, not a live home.

### Recommended action

- Replace the "still-current" framing with "historical record; host deleted 2026-07-18".
- Correct the banner.
- Whether to archive is a fork.

### Notes

- 17 cited mere commits are all on main. genet `a2d91ddc` and `39cb5b86` are on genet main.
- genet `8bde0e96` resolves in neither mere nor genet: unverifiable.
- All 15 relative links resolve.
- Seven active docs cite this plan.
- The plan is complete (closed), so DOC_POLICY §8 calls for archiving.

## mere_docs/implementation_strategy/2026-06-18_node_representation_arrangement_plan.md

- disposition: historical-marked
- status line: "Status: Superseded 2026-06-23 (kept in place, not relocated, as active siblings cite it). Substantially complete (P0 cues, P1 per-node form, P2-static sprite faces + the sprite-alpha hull collider, P3 + P4 done). The representation axis is re-based and continued by the [node_body_face_model_plan](2026-06-23_node_body_face_model_plan.md) (Body × Face, tile/shape as body presets, decoupled hull, per-node material, the generalized shape editor, and the corrected interactive/scripted-form feasibility). The arrangement axis is owned by [graph_signals_layer_plan](../../archive_docs/2026-08-20_completed_plans/2026-06-22_graph_signals_layer_plan.md) (Decision 7). Kept as the record of P0-P4; new representation work lands in the successor." — accurate: yes
- claims checked: 7 — holds: 6, stale: 1, unverifiable: 0

### Stale claims

- The code header "`crates/platen/` (cartography dispatch)" is unmarked. `crates/platen/platen/src/lib.rs:24-25` says that lane merged into the canvas crate. Cartography is `crates/canvas/cartography` plus pictograph.

### Contradictions

- "The arrangement axis is owned by graph_signals_layer_plan", but that plan was archived complete on 2026-08-20, so the axis has no active owner.
- `interaction_model_spine.md:82` still names this superseded plan as Owner of the represent stage.
- The successor `node_body_face_model_plan` still reads "Planning (with Mark)". That belongs to another batch's record.

### Recommended action

- Mark the platen header path historical.
- Archiving is a fork.

### Notes

Checked:
- Successor plan exists and names itself successor. Graph-signals link resolves to the archive.
- 15 active docs cite this plan.
- `seiche::NodeCollider::Hull` (`node_body.rs:34`).
- Pictograph `cartography_geometry` :17, `apply_cartography_sprites` :206, `set_node_sprite` :337.

The rename banner's pointer is marked historical and not counted. The plan is complete and superseded, so §8 calls for archiving.

## mere_docs/implementation_strategy/2026-06-23_gloss_outline_lens_plan.md

- disposition: current
- status line: "Status: partially implemented: P0, P1, P1a, and P2 caps landed; the remaining pluggable-lens work stays open." — accurate: no
- claims checked: 11 — holds: 6, stale: 5, unverifiable: 0

### Stale claims

- The status says P1 and P1a landed. Both host halves died with meerkat (`c5f01064`): `gloss_outline_view` and `gloss_outline_a11y_tree` have 0 hits, and so does the Decision #4 apparatus "Graph" section (`apparatus_items`). No host renders the outline, and Turnstone has no use of `glossary::outline_*` or `mere::gloss`.
- "Remaining pluggable-lens work" leaves out P3 and P4, open per the Phases section (P2's scope lens is arguably inside it).
- Line 11, "today `mere-orrery`, to be renamed": renamed `glossary` (`8d55f96`), then folded into `mere::glossary` (`61894570`).
- The line-3 banner path `crates/graph/subgraph`: folded into `crates/mere/src/subgraph.rs`.
- Findings and P3 say the `intel/signals` producer is "unbuilt" and that P3 is gated on it. It landed 2026-06-24 as `mere-signals` (closing note of the archived graph_signals plan) and now lives in `crates/canvas/pictograph/src/signals/{importance,community}.rs` (`f590e45d`). P3's gate is open.

### Contradictions

- Line 7, "Planning (with Mark), 2026-06-23.", sits directly under the Status line.
- `DOC_README.md:263` says "planning (with Mark)".

### Recommended action

- Write a dated status saying what survives (`glossary.rs:59/134/172`; caps at `gloss.rs:36/39/73`), what was retired with meerkat, and that P3 is unblocked.
- Fix lines 7 and 11 and the banner.
- Update `DOC_README.md:263`.

### Notes

Checked:
- `8d55f96` and `ffb1b5e` on main.
- Kernel queries `query.rs:240/504/545/661/666`.
- djot engine: `nematic djot.rs:39`, `inker routing.rs:288`.
- All 9 relative links plus the nematic and genet cross-repo paths resolve.
- `crates/graph/linked-data` is still a separate crate.

## mere_docs/implementation_strategy/2026-06-23_native_session_store_plan.md

- disposition: current
- status line: "Status: Immediate HTTP/session work landed; 2026-07-04 reconciliation found the scripted-rung cookie wiring has also landed. Remaining native-session work is the JS-cookie persistence trigger, durable `localStorage` host backing, flip-back SESSION import, live multi-persona jar selection, and web-privacy refinements (`Partitioned` / top-level-site storage keys)." — accurate: no
- claims checked: 8 — holds: 6, stale: 2, unverifiable: 0

### Stale claims

- "The scripted-rung cookie wiring has also landed" (`JarCookieProvider`, the `genet.scripted` live consumer, `scripted_rung_document_cookie_reaches_the_jar`, plus the reconciliation's `content/actor.rs` and `content/mod.rs`). All 0 hits. No production `CookieProvider` impl exists in mere, genet or Turnstone (genet has two test-only impls, `genet-scripted/document.rs:1934` and `script-runtime-api/dom/tests.rs:1675`); mere has only a re-export at `ports/pelt/desktop/lib.rs:136`.
- The remaining item "JS-cookie persistence trigger" (flush on host event-loop drain) is done in Turnstone. `src/shell/events.rs:221-223` flushes "any cookie a fetch or a script set since the last drain" via `src/cookie_custody.rs` (`3671ad3`, 2026-09-22). No script writer exists, though.

### Contradictions

- `DOC_README.md:269` repeats the `JarCookieProvider` claim.
- The plan names meerkat as owner throughout. Turnstone now holds cookie custody and cites this plan for persona keying (`cookie_custody.rs:12-14`).

### Recommended action

- Rewrite the status:
  - HTTP and durable layers live in `crates/system/fetch`;
  - custody and drain-flush are in Turnstone;
  - no script `CookieProvider` is wired;
  - remaining: the script cookie provider, `StorageProvider` backing, flip-back import, the per-persona registry, and partition refinements.
- Fix `DOC_README.md:269`.

### Notes

Checked:
- `fetch/src/cookies.rs:20` `session_jar`, `cookies_flip.rs:18`, `cookies_persist.rs:114/184`.
- `inker/src/flip/api.rs:79-92`: Cookie with `same_site` / `expires` / `partitioned`.
- `2fa18ad` and `6bbe6f4` on main.
- netfetcher `7c22a65` and `514334c`, and genet `3cf326a` and `3ed0ed0`, all on genet main. Traits at `script-runtime-api dom/mod.rs:1105` and `platform.rs:258`.
- No `FlipBack` impl feeds the jar.
- The flipcarrier link resolves.

## mere_docs/implementation_strategy/2026-06-23_orrery_custom_layout_element_plan.md

- disposition: historical-unmarked
- status line: "Status: parked by design: no code landed; un-park only if host-driven transform setting becomes a performance or correctness problem." — accurate: yes
- claims checked: 9 — holds: 5, stale: 4, unverifiable: 0

### Stale claims

- "The interim holds": the focus ring and a11y bounds via `IncrementalLayout::accumulated_translate`. No definition exists in genet main or mere, and both consumers were meerkat's.
- "Today `orrery_element` is a host-positioned `<div>`…", plus the meerkat host-migration paragraph. 0 hits. The successor is pictograph's retained `node_document` (`.stage` / `.gnode`, `canvas.rs:312-319`), still positioned by a per-frame `transform: translate` (`frame.rs:377`).
- The `external_texture_key_of` precedent: no definition in genet main.
- The genet `incremental.rs:1232` regression guard: no `incremental.rs` in genet main.

### Contradictions

- none.

### Recommended action

- Either retarget the plan to pictograph's `node_document`, or retire it as dead. This is a fork.

### Notes

- genet `a2d91ddc` is on genet main; mere `7181206` is on main.
- `genet/docs/2026-05-27_genet_as_host_xilem_serval_plan.md` exists. `RepaintOnly` exists at `genet-scripted/capture.rs:116`.
- No custom-layout mode exists in genet or pictograph.
- Links resolve.
- `DOC_README.md:267` agrees with the plan.

## mere_docs/implementation_strategy/2026-06-24_alembic_implementation_plan.md

- disposition: current
- status line: "Status: partially implemented: slices A-C landed; merge, promotion, event-log, LoRA, and settings/Timeline follow-ons remain deferred." — accurate: no
- claims checked: 12 — holds: 6, stale: 4, unverifiable: 2

### Stale claims

- The status omits slice D's landed half: the forgetting (`d3893dd`, `95f3a20`) and consolidation passes, now in `mere-athanor` (`ports/distillery/athanor/src/lib.rs:116/141/321/372`, `1bda73d5`). D is not complete: its "runs in the background" done-condition is unmet (`lib.rs:30-32`: scheduling "still to come"), and the facet pass and Steward surfacing remain open.
- "Merge … deferred". `compose_graph_codicils` (`pandect graph_codicil.rs:339`, doc: "Alembic tail B7 / decision #1") landed in `3e828a48` (2026-06-30).
- "Event-log … deferred". Slice E's successor, `event_log_timeline_plan`, was itself superseded on 2026-08-03. `GraphJournal` (`journal.rs:178`) and `CapturedDelta` (`capture.rs:43`) supply the substrate.
- The engram→codicil rename (`c51b9704`) is unmarked. The plan's `save_graph_engram` / `open_engram_as_session` / `list_graph_engrams` / `GraphEngram` are now `save_graph_codicil` :194, `open_codicil_as_session` :276 and `list_graph_codicils` :323, and the schema is `mere.graph-snapshot/v2` (:39-40).

### Contradictions

- `DOC_README.md:250` says "A-D shipped" (the status says A-C) and lists merge, promote and event-log as deferred decisions, while §3 says all seven were resolved.

### Recommended action

- Write a dated status:
  - landed: A-C, D's forgetting and consolidation passes, and merge (B7); D's background scheduling, facet pass and Steward surfacing open;
  - host halves retired with meerkat;
  - event log superseded by graph-view curation;
  - remaining: promote/demote UI, settings, Timeline.
- Add a codicil-rename banner.
- Fix `DOC_README.md:250`.

### Notes

Checked:
- Commits `af89808 4f51175 9cacd41 44a0dc8 b029d83 7c6e5f4 d3893dd 95f3a20` all on main.
- `mere-alembic` `memory_levels.rs:40/47/149/177`.
- `alembic/README.md:43-45` (Fleece).
- `local_models_harness_brief` exists. Unmarked links resolve.

Unverifiable:
- Promotion UI: Turnstone registers `PaneContent::Alembic` (`panes/mod.rs:160`), but I did not trace what it shows.
- Timeline scrubber state.

## mere_docs/implementation_strategy/2026-06-24_meaningful_physics_signals_plan.md

- disposition: current
- status line: "Status: plan-only: seams and source signals were scoped, but no implementation slice has landed." — accurate: yes
- claims checked: 13 — holds: 10, stale: 3, unverifiable: 0

### Stale claims

- "Contract exists, producer pending", plus slice 4 waiting until graph_signals "produces `ImportanceWeights` / `AffinityScores`". The producer exists: `crates/canvas/pictograph/src/signals/{importance,community,affinity}.rs`, with `structural_affinity` at `affinity.rs:29` and the affinity spring at `canvas.rs:63-66`.
- "Consumed today by the `arrangements` layout adapters": no such crate; they are `crates/canvas/cartography/src/adapters/`.
- The "Real now" system and per-node sources, and "meerkat gathers it". The meerkat `ContentState` / `SyncStatus` / observability / steward sources are gone (`steward_rows` 0 hits); live equivalents exist for two of them (Turnstone `src/content.rs` `ContentStates`, and stickleback `synced_space.rs:64` `SyncStatus`, re-exported as `mesh::SyncStatus`), none for observability or steward. The links are marked historical, but the prose and slice 1's inputs are present tense.

### Contradictions

- `DOC_README.md:249` repeats "contract-only (producer owned by graph_signals)" and the meerkat signal inventory.

### Recommended action

- Say that the signals producer exists in pictograph.
- Re-ground slices 1-2 on the surviving hosts' sources (Turnstone or fetch).
- Fix the adapters path and `DOC_README.md:249`.

### Notes

Checked:
- No `AmbientMetrics` / `set_metrics`. The `AmbientSim` trait is at `ambient/mod.rs:47`.
- Kernel `query.rs:402/409/661/666`. Pictograph `set_node_favicon`, `set_node_states`, `set_node_material` :414, `apply_cartography_materials` :234, `apply_cartography_sizing` :179.
- seiche `set_node_materials` :142, `set_node_colliders` :110, `NodeMaterial` :77, `wants_continuous_tick`.
- cartography `IntelligenceSignals` (`signals.rs:24`).
- Links resolve.

## mere_docs/implementation_strategy/2026-06-24_tearout_gestures_plan.md

- disposition: historical-marked
- status line: "Status: Trichotomy + cross-graph copy/move + cascade DONE + driven (meerkat-era; G1 plumbing, G3 branch, G4 fork, G5 copy+move, G6 cascade via subgraph #3; subgraph #1 per-window focus also landed). Tile-tab leaf tear-out is now DONE + fresh-headed verified. The one substantial interactive item still open is the ambiguous no-modifier orrery drag path, which now cleanly belongs to the notification/toast subsystem plus the pin-vs-drag-out gesture split:" — accurate: no
- claims checked: 7 — holds: 4, stale: 3, unverifiable: 0

### Stale claims

- The line-3 note says "The live names are SessionSubgraphs in crates/graph/subgraph". Folded into `crates/mere/src/subgraph.rs` (`61894570`).
- The 2026-07-19 banner says "The gesture stack is UNWIRED on turnstone". G4-R is complete, and Turnstone wires:
  - leaf: `TearOutActivePane`, `src/action.rs:238`;
  - branch: `TearOutTile`, `:477-478`;
  - fork: `ForkNode` / `ForkFocusedNode`, `:487-491`;
  - all routed in `app/mod.rs:1327-1337`.
- The status line presents the deleted meerkat implementation as DONE, with one open item, as if it were the plan's status. The Turnstone state goes unstated: leaf, branch and fork are wired; G5 copy/move and G6 cascade were not found in Turnstone `src`.

### Contradictions

- The banner contradicts the G4-R section (COMPLETE 2026-07-20).
- The remaining "ambiguous drag" item is routed to the notification subsystem, whose plan is retired (`archive_docs/2026-09-02_retired_plans/2026-06-27_notification_subsystem_plan.md`).
- `DOC_README.md:216` describes the meerkat-era state as live and mentions neither the deletion nor G4-R.

### Recommended action

- Write a dated Turnstone-era status: G4-R done; leaf and branch arms live; G5 and G6 plus the ambiguous drag open.
- Fix the note path.
- Update `DOC_README.md:216`.

### Notes

Checked:
- mere `c9caf26` on main; turnstone `aa32a24` on HEAD.
- `ComponentCopy` (`cross_graph.rs:209`), `copy_component_from` :131, `copy_node_from` :51, `CopiedFrom`.
- `copy_node_facets` (`facet_store.rs:107`), `copy_scene_facets` (`scene_facets.rs:136`).
- `commit_positions_to_graph` survives only as a comment (`cartography.rs:71`).
- `scenarios/facet_fork.scn` exists.
- All 5 links resolve.

## mere_docs/implementation_strategy/2026-06-25_athanor_steady_heat_actor_plan.md

- disposition: historical-unmarked
- status line: "Status: P1 + P2 done (2026-07-01). Spun out of the [Alembic tail handoff](../../archive_docs/2026-06-30_completed_plans/2026-06-25_alembic_tail_and_audit_polish_handoff.md) B1 (slice D's remainder). Architecture: [alembic memory + engrams](../technical_architecture/2026-06-09_alembic_memory_and_engrams.md)." — accurate: no
- claims checked: 6 — holds: 3, stale: 3, unverifiable: 0

### Stale claims

- "P1 done". The idle cadence was meerkat's (`app_handler/idle_forgetting.rs`, `IDLE_GRACE`, `PASS_INTERVAL`: 0 hits).
  - Nothing schedules forgetting or consolidation today. mere-athanor `lib.rs:30-32` says "Scheduling them … is still to come".
  - Turnstone runs only `propose_retirement`, on session open (`recycle.rs:113-125`).
- "The pass logic — `session-runtime/athanor.rs`": now `ports/distillery/athanor/src/lib.rs` (`1bda73d5`).
- P3: a `spawn_athanor` armillary actor owning the cadence. This is overtaken by the ruling in athanor `lib.rs:22-29` (2026-09-02): "scheduled, not resident … Djinn contains the scheduler".

### Contradictions

- `DOC_README.md:224` says "P1 done 2026-06-30 … P2 waits on" the lineage, while the status says P1 and P2 are done.
- P3 contradicts the Djinn-scheduler ruling.

### Recommended action

- Status: P1 and P2 landed and P1 was retired with meerkat; the P2 logic lives in mere-athanor; scheduling is ruled to Djinn.
- Archiving or retargeting is a fork.
- Fix `DOC_README.md:224`.

### Notes

Checked:
- `propose_consolidation` :321, `apply_consolidation` :372, `CONSOLIDATION_CANDIDATE_CAP` 50 (:77), `SAME_MATERIAL_OVERLAP` 0.5 (:83).
- Compose records `upstream` (`graph_codicil.rs:333-336`).
- All links resolve.

## mere_docs/implementation_strategy/2026-06-25_xilem_serval_control_adoption_plan.md

- disposition: historical-unmarked
- status line: "Status: P1 + P2 done 2026-06-25 (genet + meerkat). P3 gated/open. From the [genet capability-misuse sweep](../../archive_docs/2026-07-03_completed_plans/2026-06-25_overlay_primitive_adoption_plan.md) (2026-06-25)." — accurate: no
- claims checked: 6 — holds: 3, stale: 3, unverifiable: 0

### Stale claims

- The P1 and P2 meerkat halves are gone: the `views.rs` button helper, `PaneItem` / `PaneAria` radio and switch rows, and `radio_and_switch_items_stamp_aria`. `PaneItem` has 0 hits; `c5f01064` is "the pane model leaves with it".
- "P3 gated/open" on "a lensed `PaneItem` path". The gate is moot: `PaneItem` is gone, and Turnstone already lenses `&mut RadioGroup` (`src/inspector_pane.rs:64,207`) with `aria-checked` (:93).
- "`xilem-serval` exports … `controls.rs:685`". The control set is cambium, now in mere:
  - `crates/cambium/cambium/src/controls/button.rs:19`;
  - `toggle.rs:64/77`;
  - `cambium/src/radio.rs:60`, `select.rs:76`, `slider.rs:68` (outside `controls/`);
  - `controls/field.rs:124`.

### Contradictions

- `DOC_README.md:222` says in present tense "meerkat uses only 1 of xilem-serval's 7 controls", against the status line.

### Recommended action

- Mark the plan historical: done in meerkat, host retired, P3 moot.
- Archiving is a fork.
- Fix `DOC_README.md:222`.

### Notes

- `OnClick::attr` survives in cambium `event.rs:131`.
- genet-render `a11y.rs:28` reads `role`; `:609-617` maps `aria-checked`.
- The overlay plan link resolves.
- The plan is complete apart from the moot P3, so §8 calls for archiving.

## mere_docs/implementation_strategy/2026-06-26_illume_text_lexer_plan.md

- disposition: historical-unmarked
- status line: "Status: Points 1-6 done and headed-verified. Point 7 part-shipped 2026-06-27 (tinct 0.1.0 + illume 0.0.1 published). Point 8 landed 2026-07-08: illume extracted to its own repo and the bridge dissolved into genet. illume is now a standalone public repo (`github.com/mark-ik/illume`, MIT OR Apache-2.0, edition 2024), and the `SyntaxKind → SyntaxRole → class` bridge that had lived host-side in `meerkat/knot_highlight.rs` moved *into* xilem-serval behind a `highlight` feature (optional illume + tinct deps). So the highlighter is no longer a per-host concern: any genet host flips `xilem-serval/highlight` and gets `highlighted_textarea` / `highlighted_text_field` / `syntax_css` for free (the omnibar, a note editor, a chat line, and — when genet gains a TUI backend — terminal text too). meerkat's bridge is deleted; Isometry (which already consumes xilem-serval) can adopt highlighting with a single feature flag. See the 2026-07-08 Progress entry. Captures the decisions and the build of the illume promotion across the 2026-06-26 / 27 / 07-08 sessions: the knot editor's highlight core promoted to a standalone sibling crate (illume), paired with tincture (published as tinct) for themed colours and genet's styled field for rendering, aimed at making mere/meerkat operable and legible from the omnibar." — accurate: no
- claims checked: 12 — holds: 6, stale: 4, unverifiable: 2

### Stale claims

- "illume is now a standalone public repo" and "mere consumes it by git dep + local override, the in-workspace copy deleted". It is a workspace path dep again (`Cargo.toml:311,611`). `crates/nematic/illume/Cargo.toml:6-9`: "Landed in mere 2026-09-03 (platform boundary plan, P3)".
- "MIT OR Apache-2.0": the manifest says `license = "MPL-2.0"` (`illume/Cargo.toml:5`).
- The bridge "moved into xilem-serval behind a `highlight` feature", "dissolved into genet". It is cambium, in mere: `crates/cambium/cambium/Cargo.toml:12`, `highlight.rs:147/154/163`.
- "mere sources tinct from crates.io via a `package = "tinct"` alias". tinct is a workspace path member since 2026-09-03 (`Cargo.toml:408-413`, `561-566`).

### Contradictions

- `DOC_README.md:276` says "planning; two pieces shipped … Remaining: illume rename …, extraction + publish", against the status line.

### Recommended action

- Write a dated status recording the 2026-09-03 re-homing of illume, tinct and cambium into mere, and the MPL-2.0 licence.
- Fix `DOC_README.md:276`.
- Remaining work: a stable release, plus the omnibar grammar gaps.

### Notes

Checked:
- mere `bcaf834 6d0a2ae 81d0c86 8e32f38 e271adc 3fdc64f` on main. genet `6a3ceace 3abaad8 ea5fdf33` on genet main.
- `knot-editor-host` is home at `crates/inker/knot-editor-host` (`Cargo.toml:310`).
- `crates/cambium/tinct` exists.
- 3 links resolve.

Unverifiable:
- the crates.io publications (no network);
- tincture commits `03661ce` and `cb33adc` (repo not local).
