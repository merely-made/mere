# Batch 46 — S14 pass, phase B8 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-07-01_graph_write_path_migration_plan.md | historical-unmarked | no | 10 | 7 | 2 | 1 |
| mere_docs/implementation_strategy/2026-07-02_graph_delta_capture_apparatus_stats_plan.md | historical-unmarked | no | 10 | 6 | 4 | 0 |
| mere_docs/implementation_strategy/2026-07-06_intel_vector_index_burn_lift_plan.md | current | no | 8 | 6 | 2 | 0 |
| mere_docs/implementation_strategy/2026-07-12_deletion_retention_and_native_drop_plan.md | current | yes | 14 | 11 | 2 | 1 |
| mere_docs/implementation_strategy/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md | current | yes | 16 | 12 | 4 | 0 |
| mere_docs/implementation_strategy/2026-07-23_repo_consolidation_plan.md | historical-unmarked | no | 15 | 12 | 2 | 1 |
| mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md | current | no | 14 | 10 | 4 | 0 |
| mere_docs/implementation_strategy/2026-08-23_runtime_composition_acceptance_plan.md | current | no | 10 | 6 | 3 | 1 |
| mere_docs/implementation_strategy/2026-08-24_knot_shared_surface_and_port_contribution_plan.md | historical-unmarked | no | 9 | 6 | 3 | 0 |
| mere_docs/implementation_strategy/2026-09-25_graphshell_one_tree_plan.md | current | yes | 12 | 9 | 3 | 0 |
| mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md | current | no | 11 | 8 | 2 | 1 |
| cambium_docs/implementation_strategy/2026-05-27_serval_as_host_xilem_serval_plan.md | historical-marked | yes | 9 | 7 | 2 | 0 |
| **Totals** |  |  | **138** | **100** | **33** | **5** |

**Totals: 12 docs, 138 claims checked (100 holds, 33 stale, 5 unverifiable), 23 contradictions; 8 status lines wrong.**

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

Checked directly in this session: graph-kernel's `graph/mod.rs` reads
"ENFORCED as of the 2026-07-01 write-path migration" at the base (write
path); chatelaine P4a's `007fbe7c` is an ancestor of the base (device
pairing). The verifier refuted nothing and found 5 items true only in part
(the capture plan's replay test, which spot-checks fields; the deletion
plan's internal record, which still calls the fold a gap at l.855; the murm
plan's Phase A, where mesh's direct p2panda dependencies remain as the
status's second blocker; the burn lift's P2 commit; Turnstone's dependency
lines); those records are corrected.

## mere_docs/implementation_strategy/2026-07-01_graph_write_path_migration_plan.md

- disposition: historical-unmarked
- status line: "Status: In progress. Finishes the single-write-path boundary that `graph/mod.rs:328` declares but does not enforce ("graph topology mutators are crate-internal... other runtime/shell code paths should route through reducer intents"). Prerequisite hardening for [event_log_timeline_plan](2026-07-01_event_log_timeline_plan.md): once `apply_graph_delta` is the real funnel, slice E's recording hook instruments one function instead of ~45 mutator bodies." — accurate: no
- claims checked: 10 — holds: 7, stale: 2, unverifiable: 1

### Stale claims

- The status says "In progress" and that the boundary is declared but not enforced at `graph/mod.rs:328`. The tree shows the work done. The comment now sits at `crates/graph/graph-kernel/src/graph/mod.rs:412` and reads "ENFORCED as of the 2026-07-01 write-path migration". `get_node_mut` is `pub(crate)` (query.rs:108), and so is `get_edge_mut` (edge_ops.rs:436).
- Writer class 3 lists `set_node_lifecycle` as a `pub` transient exemption. No such fn exists in crates/graph at base. mod.rs:425-428 says webview runtime state left the kernel for the `BrowserNodeState` sidecar.

### Contradictions

- The status contradicts the plan's own Progress entry for 2026-07-01, which says "**Implemented.**"
- DOC_README.md:254 says "done 2026-07-01".
- The status calls this work prerequisite hardening for event_log_timeline_plan slice E. DOC_README.md:253 marks that plan superseded 2026-08-03. The funnel was instrumented by capture.rs (apply.rs:16 imports it) and journal.rs instead.

### Recommended action

- Rewrite the status as "landed 2026-07-01, enforced at graph/mod.rs:412". Annotate the event_log prerequisite as superseded by GraphJournal, and annotate class 3 for `set_node_lifecycle`.
- Record the unmet `apply.rs` split gotcha: the file is 1938 lines and there is no delta.rs. Extract or close it, then archive per §8.

### Notes

Checked the 16 GraphDelta variants and the HistoryStepped/FieldChanged/Applied results (apply.rs:328-482) and the four wrappers (apply.rs:1866-1933). Checked the `fixtures` feature (graph-kernel/Cargo.toml:20) and that all nine enabling manifests use it under [dev-dependencies] only. Checked the GraphFixtures escape hatches (fixtures.rs:70-71). Suites passing was not run.

## mere_docs/implementation_strategy/2026-07-02_graph_delta_capture_apparatus_stats_plan.md

- disposition: historical-unmarked
- status line: "Status: in progress." — accurate: no
- claims checked: 10 — holds: 6, stale: 4, unverifiable: 0

### Stale claims

- Status "in progress": the kernel half landed, and the host half was deleted with meerkat in c5f01064 (2026-07-18, "The funeral: meerkat leaves the workspace").
- Phase B's "env-gated postcard log writer (`MERE_GRAPH_DELTA_LOG=<dir>`)" is gone. The string appears in no code at 535bca11, only in this plan and support/doc-audit/d2/snapshot_281_aggregate.json. The session log's successor is GraphJournal (crates/graph/graph-kernel/src/graph/journal.rs).
- Phase C's meerkat `graph_delta_log` whole-snapshot oracle is gone: no `graph_delta_log` appears in any .rs file. Kernel replay tests remain: capture.rs:1138 spot-checks many fields and compares only the navigation snapshot (l.1624-1650), never a whole `GraphSnapshot`, and journal.rs:832 checks replay-matches-live. There is no canned regression log in the tree.
- Phase D's "Meerkat-side typed apparatus stat model" and its document/scene rows are gone. crates/domain/apparatus/src/lib.rs is a bounded observation store with placeholder sections and no table stats.

### Contradictions

- DOC_README.md:197 describes env-gated session logs and live per-table apparatus stats as current features, with no status.

### Recommended action

- Mark the plan historical. Phase A, the capture hook and the replay forms landed (capture.rs). Phase B/C-host/D were deleted with meerkat in c5f01064, and the session log is superseded by GraphJournal. Decide whether per-table stats get a new home, then archive and fix the DOC_README entry.

### Notes

Checked CapturedDelta and its postcard round-trip tests (capture.rs:868ff), the late replay variants (capture.rs:181-252) and `last_visited_ms` (node_facets.rs:36). Checked the content-contract stats types (content-contract/src/lib.rs:51-218). The removal came from `git log -S MERE_GRAPH_DELTA_LOG`.

## mere_docs/implementation_strategy/2026-07-06_intel_vector_index_burn_lift_plan.md

- disposition: current
- status line: "Status: scoped, not started. A cross-cutting scaling investment surfaced by the Lane 5 P5 wiring; deliberately its own plan because it lifts three consumers at once, not just arrangement." — accurate: no
- claims checked: 8 — holds: 6, stale: 2, unverifiable: 0

### Stale claims

- "Scoped, not started" is wrong; P1 to P3 work has landed:
  - P1 landed in bb1b1608 (2026-07-08, ndarray + wgpu parity).
  - P2's crossovers are recorded as AFFINITY_GPU_MIN_ENTRIES=1024 and SEARCH_GPU_MIN_ENTRIES=4096 (crates/intel/esp/src/embed/index_burn.rs:39,43).
  - P3's keyed accelerators `nearest_over_index` and `affinity_pairs_over_index` landed in c6ab781b (index_burn.rs:113,144).
  - The features exist at esp/Cargo.toml:80-81.
- `aether::forces::repulsion` is cited as the sibling kernel. aether was renamed quint in 5b91b2ea (2026-07-09), and quint was folded in eae87153 (2026-09-02). The function is now crates/conatus/seiche/src/tensor_forces.rs:104.

### Contradictions

- DOC_README.md:357 says "Path A landed"; the status says not started.
- DOC_README overstates Path A. No crate enables `index-burn` (only esp defines it), and nothing outside index_burn.rs calls the accelerators. Routing for affinity, recall and canvas search, the first done-condition, is not done.
- A stray closing code fence sits at line 117 with no opener.

### Recommended action

- New status: "P1 landed (bb1b1608), P2 crossover measured (98111f60), P3 accelerators and constants landed (c6ab781b), consumer routing not wired, P4 deferred". Repoint the aether path, correct DOC_README, and remove the stray fence.

### Notes

Checked the index_burn.rs header and API, and the P3 commit message ("Routing is the caller's one-line check"). Grepped for index-burn enables and for callers of the accelerators. The historical path esp/src/embed/index.rs:9 still carries the HNSW note.

## mere_docs/implementation_strategy/2026-07-12_deletion_retention_and_native_drop_plan.md

- disposition: current
- status line: "Status: Active plan." — accurate: yes
- claims checked: 14 — holds: 11, stale: 2, unverifiable: 1

### Stale claims

- The promotion note (l.9-10) lists "the Moot mapping" and "the constitution fold" as remaining, and l.662 says "the Moot mapping remain D5 work". Both landed before the 2026-07-26 note:
  - Moot aggregate drops are at crates/moot/gemot/src/moot/records/store.rs:414-479 (be99eadb, 2026-07-15).
  - The constitution fold is constitution/fold.rs:143 (by a4da5193, 2026-07-17).
- The §2 table (l.83) says "Meerkat selects redb", but meerkat was removed in c5f01064.

### Contradictions

- Inside the plan, the promotion note and l.662 disagree with l.492-494 (the fold) and l.856-863 (aggregate drops), which record them as landed; and l.855, in the same Progress list, still says "The live constitution log/fold remains the governance gap".
- DOC_README.md:156 lists "durable conversation reopen, drop-import view refresh, the Moot mapping ... live Moot constitution fold remain". The plan's l.660-661 and l.851-852 record redb reopen and imported-view refresh as landed.
- The plan does not mention the September group-key lane in stickleback: 71a767b8, 11f0d705 and 3d3ad81c, under coop_lifecycle_parity_plan. That lane bears on its "group authorization/key distribution" item. Whether it now feeds Murm's drop protector was not verifiable.

### Recommended action

- Rewrite the remaining-work list once and date the status. The list as the tree shows it: Iroh/Retinue carriage, binding group keys to the drop protector, a conversation retention checkpoint, reference tracing, and live peer command wiring.
- Cross-reference the group-key lane and fix DOC_README.

### Notes

These symbols exist at base: OperationProcessor (stickleback/src/processor.rs), DropProtector/MEREDRP (drop.rs), MERERCP (receipt.rs), DropExportSelector, MeshDropSelector, ConversationDropSelector, RetentionCheckpoint (mesh and gemot), GovernedCheckpointAuthority, MootGovernance and HydratedPayload. No DropId use exists outside stickleback and gemot, so Iroh/Retinue carriage is still open. The related links resolve.

## mere_docs/implementation_strategy/2026-07-12_murm_peer_runtime_and_moot_domain_plan.md

- disposition: current
- status line: "Status: Active plan." — accurate: yes
- claims checked: 16 — holds: 12, stale: 4, unverifiable: 0

### Stale claims

- Phase A's status (l.230-233) says compatibility re-exports in `transport` and `mooting` keep the phase open. They are gone: mooting/src/lib.rs exports only recognition, and neither crate has `pub use stickleback`. Transport's was removed by 6d1187a7 (2026-07-27), mooting's earlier by be99eadb (2026-07-15). The done-conditions (l.243-249) now read as met, but the status's second named blocker remains: mesh still depends directly on p2panda-core, p2panda-store and p2panda-net (mesh/Cargo.toml:18, 30, 44).
- Phase C (l.327-328) says "Meerkat selects that redb backend", but meerkat was removed in c5f01064.
- Phase F and the 2026-07-26 finding say turnstone names no gemot/p2panda dependency and that the lane is vacuous. Turnstone's Cargo.toml (l.69-83) at HEAD c3b14cb depends directly on commons, transport, gemot, mere-moot, stickleback and muniment. p2panda appears only under [patch.crates-io] (l.318+), so the criterion holds, but not vacuously.
- The 2026-07-24 finding says "no surface reads the authorized view yet". Three sites read `authorized_fauna`: ports/moot/src/captured_web.rs:83, turnstone src/place/worker.rs:1918 and src/place/captured_collection.rs:160.

### Contradictions

- Progress for 2026-07-13 has the constitution-producer entry twice (l.1026-1032 and l.1037-1040).

### Recommended action

- Close Phase A citing 6d1187a7. Restate Phase F as non-vacuous, with turnstone's place worker as the consumer. Record the authorized-view readers, drop the meerkat mention and dedupe the Progress entry.

### Notes

- JoinedSpace and `leave()` are at stickleback/src/joined_space.rs:77,186. Session types are named only in joined_space.rs and synced_space.rs.
- The pinned test is at gemot records/store.rs:1216. 617ff648 is an ancestor.
- mesh/Cargo.toml has no mooting and lists transport as a dev-dependency only.
- moothold exists, and murmuring is absent.

## mere_docs/implementation_strategy/2026-07-23_repo_consolidation_plan.md

- disposition: historical-unmarked
- status line: "Status: ruled with Mark 2026-07-23; execution started same day. Later rulings: the bucket repo is named smolweb, and the errand spec crates (spartan/nex/guppy protocols) move into it, with gemini, titan, gopher, finger, plain, a shared TOFU/client-cert helper, and gemtext listed as trigger-gated later extractions from errand. Personae folds with a dependency-minimal wall and an explicit re-extraction trigger: the first external verifier, interop partner, or security-audit engagement pulls it back into a dedicated repo. Execution order is adjusted from the phase numbering: C4 runs first among code phases (active radio hardware work wants the merged workspace), C2 runs last (a live genet livery session holds uncommitted edits), consumer repoints happen only after each receiving repo is pushed green, and old repos are archived only at the very end, so no intermediate state leaves a consumer unresolvable. This plan supersedes the repository-boundary posture of the [Graphshell remote projection host plan](2026-07-22_graphshell_remote_projection_host_plan.md) sections 2, 3, and 9 where they conflict; that plan's protocol design, proof sequence, and receipts remain in force." — accurate: no
- claims checked: 15 — holds: 12, stale: 2, unverifiable: 1

### Stale claims

- The status says "execution started same day". Progress records C0-C6 done by 2026-07-24 ("C6 done"). The mere commits 73d8b1b1, 6a37de6b, 0086c9c3 and 4b8d875f are ancestors.
- "Still open" says "Still on 1.96.0: isometry, hocket, turnstone". isometry and turnstone now pin 1.98.1 in rust-toolchain.toml; only hocket is still on 1.96.0.

### Contradictions

- DOC_README.md:173 calls the plan a "historical execution record" that is partly superseded; the status still reads as in execution.
- C6 says "Archive this plan per DOC_POLICY on completion."
- The Cambium-into-genet placement is superseded by the in-doc 2026-09-02 boundary note (crates/cambium is in mere). That supersession is marked, not stale.

### Recommended action

- New status: "executed 2026-07-23/24 (C0-C6); topology superseded 2026-09-02". Trim "Still open" to hocket's toolchain and the graphshell-* publication, extract those, and archive.

### Notes

- Sibling receipts verified present:
  - retinue e071d08, ac2fe43, 97eef7d
  - genet ccb0b5d91df, 2a7c34c7b98, 890c4e86612, 9d366ffefd5
  - woodshed aa8026d, abdf524; hocket 7fe70e1; smolweb bcc70c7
- Paths at base: ports/graphshell, scripts/check_port_boundaries.py, crates/{graphshell,scenograph,conatus,eidetic/muniment}.
- graphshell-* publication status was not checked (no registry access).

## mere_docs/implementation_strategy/2026-08-22_conatus_engine_plan.md

- disposition: current
- status line: "Status: active; body/runtime foundation, private-backend integrity, first profile-local resident body-position publication, and first product renderer tenant implemented; Nexus admission probe blocked at its upstream Windows shader build; scope corrected 2026-08-23. 2026-08-26: Mesocosm's runtime became the first product tactile consumer (terrarium picking over `BodyWorld`, Rapier private), quint's `ResidentChunk` join was proven with per-brick patches, tracer-validated read epochs, and allocator-observed bytes (V1b), and `conatus-brick` — the shared sparse-brick ABI both game vessels pin — advanced on `codex/conatus-brick-lift` to `bd8f0044`. 2026-09-26: `modulus`'s shrinking-retarget defect fixed and its atlas sized to the card by `AtlasLimits`, ruled by Mark (brick-atlas pass below). 2026-09-27: `modulus`'s `brick_dda` takes each voxel crossing afresh from the eye instead of accumulating it and clamps its first voxel into the pointer volume, and the same walk is public on the CPU as `BrickMap::trace`, all ruled by Mark (brick-traversal precision pass below); on branch `dda-precision`, not yet merged. 2026-09-28: body-binding shape and the T2 voxel-store lane carried from the wing's existing rulings into §1 and §2. Both are planned, not implemented; body bindings remain document-only under ruling 346, and T2 waits for the accepted pre.4 migration under ruling 363. This documentation pass neither implements nor certifies the separate query-refresh API." — accurate: no
- claims checked: 14 — holds: 10, stale: 4, unverifiable: 0

### Stale claims

- "On branch `dda-precision`, not yet merged" is wrong. a404cd48, 5e46956a and c3e054d6 are ancestors of 535bca11; they merged on 2026-09-27, before the 2026-09-28 status edit. `BrickMap::trace` is at modulus/src/trace.rs:59.
- The status, §3-4 and Immediate order items 4-5 treat the resident body-position publication and the renderer tenant as implemented. Both lived in isometry `crates/isometry-runtime` (15f5da2, 7d45c40), which isometry retired in 73a31409 (2026-09-27). The plan's own §1 (l.204, 219) acknowledges the retirement.
- Quint is treated as a live crate: Existing pieces l.94, §2 l.274/292, §3 l.353, §5 l.420 and Immediate order 4. quint was folded in eae87153 (2026-09-02):
  - resident moved to crates/conatus/conatus/src/resident/
  - eval and lower_burn moved to numen
  - forces moved to seiche/src/tensor_forces.rs
- Immediate order item 1 (make the Rapier boundary private) is still listed as to do. It was done at 339e8567 (2026-08-24).

### Contradictions

- The status names `conatus-brick`; the body names `modulus`. It was renamed in 33f9b6b6 (2026-08-28). This is a dated clause, noted only.

### Recommended action

- Drop "not yet merged". Mark the Isometry tenant and position plane as retired historical receipts. Rewrite the Quint references to their conatus/numen/seiche owners, tick item 1, then do the reconciliation with the runtime ledger that l.1036-1038 calls for.

### Notes

- Commits 406aafb2, 0ec498f0, 076d503e, ed0ef4aa, 5c8379d4, 5ce144ff, 339e8567 and bd8f0044 are ancestors.
- Schedule phases (schedule.rs:19-29) and the 2026-09-28 line references (engine.rs:166; world.rs:430/468/501/515; nisus lib.rs:97/143/234) match.
- No body-binding or query-refresh code exists. Isometry's tactile.rs still calls `step(1e-6)`.
- The pre.4 migration is still pending at base: burn plan status at l.141-156.

## mere_docs/implementation_strategy/2026-08-23_runtime_composition_acceptance_plan.md

- disposition: current
- status line: "Status: active; Conatus foundation, first product profile, first resident body-position proof, first product renderer tenant, and shared brick/DDA owner complete; host adoption, 3D realization, and other second-consumer gates open" — accurate: no
- claims checked: 10 — holds: 6, stale: 3, unverifiable: 1

### Stale claims

- C1 says "Implemented locally at Mesocosm commit `b112931`; remote integration open". b112931 is an ancestor of isometry main (HEAD 988b9929).
- Several places treat the Isometry profile and tenant as live code in `isometry-runtime`: the status, C2/C2a/C2b, the Isometry section (l.78-97), ledger rows l.52/58 and C3's premise ("host constructs the product profile"). Isometry retired the crate in 73a31409 (2026-09-27). 303e347, 15f5da2 and 7d45c40 remain only as history.
- Ledger row l.54 and stop rule l.189 ("`ResidentChunk` stays Quint-owned") are out of date. quint was folded in eae87153, and ResidentChunk is at crates/conatus/conatus/src/resident/chunk.rs.

### Contradictions

- DOC_README.md:107 says "DDA row's owner is selected (conatus-brick, on its branch)". The plan (l.55, l.226) says modulus landed on main 2026-09-02 (15ecaa5c).
- The conatus plan (l.1036-1038) says this ledger still needs reconciling with the body-binding and T2 rulings. There are no entries after 2026-08-26.

### Recommended action

- Reconcile the ledger:
  - Mark C2/C2a/C2b as historical, with isometry-runtime retired.
  - Restate C3 against current Isometry and close C1.
  - Move the Quint rows to their conatus/numen/seiche owners.
  - Add the body-binding and T2 gates.
- Fix DOC_README.

### Notes

5767563c and c382e734 are mere ancestors. The nisus and modulus renames were verified (efce7310, 33f9b6b6). Brief §7 exists at design_docs/2026-08-12_family_composition_thesis_brief.md:246. Whether nisus is "published" was not checked.

## mere_docs/implementation_strategy/2026-08-24_knot_shared_surface_and_port_contribution_plan.md

- disposition: historical-unmarked
- status line: "Status: in progress; current-origin G0 and the narrow `knot-document` package published; K0, the reusable Knot surface, desktop wrapper, and semantic host receipt implemented and independently green; Turnstone T0 consumer core and user invocation landed, including explicit read-only admission; generic accessibility landed; P0 is complete — Turnstone admits `distillery.installed.v1` through the existing registry with no provider-specific renderer arm, the full shell binary builds from published sources, and the contract is reduced and frozen at v1 (Genet `001448d55`, Turnstone `3f63671`); F0 is the next gated lane" — accurate: no
- claims checked: 9 — holds: 6, stale: 3, unverifiable: 0

### Stale claims

- "F0 is the next gated lane" of this plan is overtaken. knot-editor/design_docs/2026-08-24_knot_shared_surface_and_port_contribution_plan.md (repository note 2026-09-05) names knot-editor's 2026-09-05_knot_application_workspace_plan.md as F0's current authority, and calls the shared-surface plan a historical record.
- §3.1 and G0 say `genet-host-api` owns SurfaceDescriptor and that Cambium is under genet `components/cambium`. At base the descriptor is mere crates/system/surface-api/surface.rs:86, split out by 3e9f02f6 (2026-09-02), and `RetainedSurfaceSession` is crates/cambium/cambium/src/surface.rs:60.
- The P0 follow-through says descriptor literals in knot-document, distillery, Sky and the fixtures drop the removed fields "at next pin bumps". That is done: no `placement_hint` or `potential_capabilities` remains in any .rs in mere, knot-editor or turnstone.

### Contradictions

- Two divergent copies of this plan exist, in mere and in knot-editor, with different repository notes.
- DOC_README.md:77 says F0 "remains gated" in this plan; the knot-editor copy reassigns it.

### Recommended action

- Add a mere-side note pointing F0 to knot-editor's application workspace plan and the contracts to surface-api/cambium, and record the follow-through as done. Decide which copy is canonical. The open pointer-capture gap is turnstone T-lane work.

### Notes

- These commits are present on their HEADs:
  - mere 709ae402, fb82d12f, 80c2f17c
  - genet 001448d55, bbd09062804, b8de627, 4e60f93, 1ac3727
  - turnstone 3f63671, 1e9dde1, a37e6a1, 0c105a7, 9d3a7d8, f3cb758
- surface.rs carries the v1 freeze header (l.11-17) and the reduced fields (l.86-91).
- Turnstone src/contributed_surface.rs:526-549 still ignores `pointer_capture` on move/up, so that gap holds.

## mere_docs/implementation_strategy/2026-09-25_graphshell_one_tree_plan.md

- disposition: current
- status line: "Status: in progress, ruled 2026-09-25 (reservoir plan §7 items 39 and 40). Phases 1 and 2, accessibility in the browser and the file seam, were done on 2026-09-26. Phase 3 has headed correctness receipts. On 2026-09-27 Mark approved proceeding to phase 4, with stack performance and live physics explicitly open." — accurate: yes
- claims checked: 12 — holds: 9, stale: 3, unverifiable: 0

### Stale claims

- 2026-09-30 says "Integration into main and retirement of this collision worktree remain pending". 650f8541 merged to main in 3270cac2 (2026-10-01).
- 2026-09-27 says a follow-up "removes the unpublished rootstock runner". That is done: rootstock has no `scenario` module and `ScenarioLane` is gone. The lane lives in crates/cambium/mesquite/src/lib.rs:55.
- §1 "Genet gaps" (2026-10-02) says "The lane neither pushes mere nor merges it to main; 'push mere' is still to do". genet-repin merged as 04a92ffe (2026-10-02), and 7a0950da is on origin/main.

### Contradictions

- DOC_README.md:288 says "phase 3, the canvas as a producer, next"; the plan is in phase 4.

### Recommended action

- Annotate the three overtaken items (3270cac2, mesquite, 04a92ffe) and update DOC_README to phase 4. Phase 4 stays open: ports/graphshell/src/web_gpu.rs and web/component.html still exist.

### Notes

- All 20 cited mere commits are ancestors, c19e1120 through de115df8.
- `open_file` is at cambium/src/file.rs:93; TOOLS_DOCK_MIN_WIDTH is 300+600 (web_tree.rs:62,67); `TextureProducer::semantics` is at rootstock/src/producer.rs:114.
- The receipt docs, the reservoir plan, batch_23 and genet-compatibility.md exist.

## mere_docs/implementation_strategy/2026-10-02_device_pairing_by_key_plan.md

- disposition: current
- status line: "Status (2026-10-04): in progress. Assessed and ruled by Mark from 2026-10-01 to 2026-10-04 (rulings 1 to 74 below). D1 landed (`4963b489`); D1b's mere fix (M1) landed (`177b927c`) and its fork fix (F1) shipped in the 0.7.5 repin, pushed 2026-10-04 (fork `1bec457e`, Knot `92367ec`, mere `031b3dcc`). `connected` follows the gossip overlay (ruling 31) and, off it, open connections (rulings 47 to 56). The overlay's gap after restarts has a ruled fix in iroh-gossip, held until its next release (rulings 64 to 72). Paused before D2 while chatelaine P4a runs (ruling 73)." — accurate: no
- claims checked: 11 — holds: 8, stale: 2, unverifiable: 1

### Stale claims

- "Paused before D2 while chatelaine P4a runs" is overtaken. Chatelaine P4a landed in 007fbe7c (2026-10-04), an ancestor of 535bca11. The chatelaine plan status (dramatis_docs/implementation_strategy/2026-10-01_chatelaine_cxf_plan.md:4-12) says P4 now waits for the vault lock.
- §7 Progress has one entry, 2026-10-02: "Nothing built. Next: Mark's go to start D1." D1 (4963b489) and M1 (177b927c) landed 2026-10-02, and the 0.7.5 lock (031b3dcc) landed 2026-10-04.

### Contradictions

- The status contradicts §7 Progress. The Progress log was never appended, which DOC_POLICY §8 requires. DOC_README.md:187 agrees with the status.

### Recommended action

- Append Progress entries for D1, D1b M1/F1, the repin and the pause. Restate what now gates D2.

### Notes

- Fork 1bec457e and tag mere-p2panda-net-0.7.5 exist in crates/p2panda; Knot 92367ec exists. Mere pins 0.7.5 at Cargo.toml:701, with no iroh-gossip patch (ruling 72).
- Code references match: carry/mod.rs:350, settings.rs:468, announce.rs empty app data. The D1 route is at ports/djinn/src/bin/djinn_devices.rs and resident_devices.rs.
- There are no D2 unit files. The §4 corrections are present in the SSH CA and reachability plans.
- iroh-gossip's release state was not checked (no network).

## cambium_docs/implementation_strategy/2026-05-27_serval_as_host_xilem_serval_plan.md

- disposition: historical-marked
- status line: "Status (updated 2026-09-06): historical extraction plan; its current implementation is Cambium over Genet. The public backend names are `GenetAppRunner`, `GenetCtx`, and `GenetElement`; `Serval*` below is the pre-extraction vocabulary retained only as deprecated compatibility aliases. The implementation remains strong through Stages 0-7, with the previously named host-backend blockers landed. The plan scopes using serval as the application host (chrome and content rendered by one engine), and the reactive authoring layer that requires. The finding held: that layer is mostly *reuse* of `xilem_core` (a third backend beside Masonry and `xilem_web`), not a from-scratch Dioxus-style framework. The full loop — `xilem_core` diff → serval DOM → layout → paint → netrender → present, with input routed back through serval's hit-test + faithful xilem message dispatch — is validated on screen (`pelt-live-counter`), serval the sole engine. Stages 0 through 7 have since added component composition, keyboard/focus, capture-phase events, overlays + inline style, erased views, scrolling, z-index Tier 1, `DOM → AccessKit` emission, a real caret-aware text field with selection + clipboard, select, radio, textarea, and complete IME. Pointer-drag (`pointerdown`/`move`/`up` + capture), slider, Tab/Shift+Tab focus traversal, and clip-aware scroll hit-testing are also present in the current tree and covered by focused tests. Sibling to the [scripted render loop](#relationship-to-existing-docs): both share that native dispatch substrate, which wires serval's *existing* hit-test query into event routing rather than building a new one. Per-stage status and commits are in [Staging](#staging)." — accurate: yes
- claims checked: 9 — holds: 7, stale: 2, unverifiable: 0

### Stale claims

- l.357 says `hit_test_is_clip_and_scroll_aware` "pins the behavior". That test was added in genet 9e78d1a92e0 (2026-06-01) and removed in 55c05d11759 (2026-08-21, "Retire Stylo and the incumbent layout cone"). Equivalent coverage is in genet components/genet-livery/src/layout/tests.rs:7678 and tests/form_control_hit.rs:289.
- Stage 3 (l.306-308) says "Still open: More pointer events — pointermove/pointerup/wheel". These are done: crates/cambium/cambium/src/pointer.rs and slider.rs exist, and Stage 7 (l.381-384) says so.

### Contradictions

- Inside the plan, Stage 3's still-open item contradicts Stage 7 and the status.
- genet/docs/2026-05-27_genet_as_host_xilem_serval_plan.md is a second copy with the older non-historical status. DOC_README.md:465 ("strong through Stages 0-7, landed") echoes that copy rather than this file's "historical extraction plan".

### Recommended action

- Annotate the removed test name and the Stage 3 item. Make DOC_README say historical. Decide on archiving and on the genet duplicate.

### Notes

- Genet* types: context.rs:71, pod.rs:29, runner.rs:1074. Deprecated Serval* aliases: cambium/src/lib.rs:226-239. Tab traversal tests: tests.rs:1230-1272.
- Sampled genet stage commits exist (cc4b30a, ef4c026, e10a3211f82, 42d9d04c4e0, 1ea6ab4c153). pelt-live was retired in genet b108fb509ca; the status cites it only as history.
- The cross-repo links use relative `../../../../genet/docs/` paths, which DOC_POLICY §5 discourages, but they resolve.
