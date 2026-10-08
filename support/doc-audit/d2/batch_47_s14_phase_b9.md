# Batch 47 — S14 pass, phase B9 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| cambium_docs/implementation_strategy/2026-08-31_workbench_component_plan.md | current | no | 13 | 10 | 2 | 1 |
| dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md | current | yes | 16 | 16 | 0 | 0 |
| eidetic_docs/implementation_strategy/2026-08-22_redb_opfs_feasibility_plan.md | current | yes | 13 | 9 | 4 | 0 |
| inker_docs/implementation_strategy/2026-06-15_engine_picker_and_pluggability_plan.md | historical-unmarked | no | 10 | 7 | 3 | 0 |
| intel_docs/implementation_strategy/2026-07-08_index_burn_lift_plan.md | historical-marked | yes | 9 | 8 | 1 | 0 |
| mere_docs/implementation_strategy/2026-05-26_net_media_plan.md | current | yes | 7 | 7 | 0 | 0 |
| mere_docs/implementation_strategy/2026-06-03_host_p2p_wiring_plan.md | superseded | no | 6 | 5 | 1 | 0 |
| mere_docs/implementation_strategy/2026-06-05_comms_shell_plan.md | historical-unmarked | no | 9 | 8 | 1 | 0 |
| mere_docs/implementation_strategy/2026-06-05_node_navigation_lineage_wiring_plan.md | historical-marked | no | 7 | 5 | 2 | 0 |
| mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md | current | yes | 12 | 11 | 0 | 1 |
| mere_docs/implementation_strategy/2026-06-10_scrying_tile_plan.md | historical-unmarked | no | 9 | 7 | 2 | 0 |
| mere_docs/implementation_strategy/2026-06-18_graph_query_layer_plan.md | current | no | 14 | 7 | 7 | 0 |
| **Totals** |  |  | **125** | **100** | **23** | **2** |

**Totals: 12 docs, 125 claims checked (100 holds, 23 stale, 2 unverifiable), 18 contradictions; 7 status lines wrong.**

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

Checked directly in this session: graph-kernel's `history.rs` defines
`SharedNavigationMemory` at the base (node navigation lineage); `5cb55436` is
an ancestor (redb over OPFS). The verifier confirmed every listed
contradiction and found one stale claim true only in part (the engine picker:
`EngineRoutePolicy` has consumers beyond Pelt, among them Turnstone), plus a
miscounted commit list and a missed harness commit in the redb record and a
second stale item in the petgraph-RDF status; those records are corrected.
The vault lock record is pinned to the base; L1 merged after it
(`2556a20c`), so that plan's status has moved since.

## cambium_docs/implementation_strategy/2026-08-31_workbench_component_plan.md

- disposition: current
- status line: "Status: W5 opened (2026-09-04) — Turnstone's panes become tiles, ruled by Mark; S1 and S2 are committed in this Mere snapshot; S3 remains Turnstone's pin/adoption work. W1 through W4 are implemented and landed through coordinated Genet, Mere, and product branches. W4 has captured native Pelt acceptance and cancellation receipts, a headed Graphshell browser save/mutate/reload receipt, and a durable Woodshed open-lane consumer with full view and host receipts. The temporary `genet-host-api::tile` compatibility module is now removed. Pinned products that still need their own current-Genet port retain that work outside the shared component contract." — accurate: no
- claims checked: 13 — holds: 10, stale: 2, unverifiable: 1

### Stale claims

- **The S3 pin gate (status line 15; :357-360).** The plan says S3 starts once Turnstone adopts a Mere revision containing S1/S2, and that Turnstone's pin "is not verifiable from this repository". In fact Turnstone pins `workbench` at Mere `bd5912fb` (turnstone `Cargo.toml:229`), and `9408681b` (S2c) is an ancestor of `bd5912fb`. The pin half is done. The adoption half has not started: there is no `workbench::Workspace` or `cambium::workspace` use in turnstone `src/`, and `src/panes/legacy_bridge.rs` and the `SpaceBlueprint` references remain.
- **The rustfmt finding (:327).** It says "mere has no `rustfmt.toml` … which style wins is Mark's call." `rustfmt.toml` was added in `3575a1af` (2026-09-04), the canonical policy with `match_block_trailing_comma = true` (Genet's style).

### Contradictions

- DOC_README.md:463 calls this "plan, W1-W4 landed; W5 opened". It does not record that S1/S2 landed, so it lags the plan's own status.

### Recommended action

- Restate S3 as "pin satisfied (`bd5912fb` ⊇ `9408681b`); Turnstone's A4/S3 adoption not started", and drop "not verifiable from this repository".
- Mark the rustfmt finding resolved by `3575a1af`.

### Notes

`dc5ff2b0`, `f0c7966b`, `9408681b`, `565285020c`, `7323c703`, `f01dd191`, `86eb4331`, `49f9b99e`, `2f85051245f` and `4d68c465e58` are all ancestors of the base. These exist: `crates/cambium/workbench/{float.rs,lib.rs}` (`Workspace`, `FloatingTile`, `FloatEvent`, serde feature), `cambium/src/workspace.rs` (`workspace_view`, `WorkspaceModel`, `composited_slots`), and `tabs.rs` (`tab_bar_view`, `tab_strip_closable`). `platen::TileLayout` and `ports/graphshell/src/projection_editor.rs` exist. There is exactly one `name = "workbench"` package, and no `genet_host_api::tile` use in code. The owed follow-ups are still open: `TabBarNames` and the `frisket-*` tokens remain. The physics catalog plan's P4 does not record a W5 dependency, so that claim is unverifiable.

## dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md

- disposition: current
- status line: "Status (2026-10-05): rulings 1 to 36 in §3; the threat statement is still open. The djinn test harness it waited on (ruling 18) landed (`318b8f70`). L1's checkpoint A is built and verified on a lane branch (residue fixes, the no-residue instrument, the caller map); rulings 25 to 36 settle the lock API, which is next. Nothing merged. Chatelaine P4 (CXF import) waits on this plan (chatelaine rulings 64, 65)." — accurate: yes
- claims checked: 16 — holds: 16, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- §4 still says "Drafted from the assessment; set once the forks are ruled." Rulings 1–36 are ruled and L1 is being built. This is a minor wording lag.

### Recommended action

- none for this record. Note that HEAD's `f9dba869` already supersedes this status: L1 merged as `2556a20c` after the base.

### Notes

- `318b8f70`, `24bfe7be` and `de8214c7` are ancestors of the base.
- `fea481a3` (parent `24bfe7be`) and `ffd3279b` exist and are not ancestors, which fits "nothing merged". The branch name `worktree-agent-a014d67042870a2b4` no longer exists locally.
- `095c0423` (the lock API) is dated after the base.
- Code citations checked at the base:
  - `vault.rs:369-372`, `profile_wire.rs:22-29` (no zeroize), `castellan/src/resident.rs:38-53` and `distillery/src/installed.rs:265-268` hold.
  - `authority.rs:201` held at `de8214c7`; at the base the line is :224.
  - `passphrase_root` has no caller outside its module and lib.rs.
  - `bin/personae-agent.rs` is 210 lines.
  - `install-windows.ps1` loops with `WScript.Sleep 5000`.
- Cross-doc claims: chatelaine rulings 64/65 exist, protocol plan line 349 mentions swap leakage, and the djinn test harness plan exists.

## eidetic_docs/implementation_strategy/2026-08-22_redb_opfs_feasibility_plan.md

- disposition: current
- status line: "Status: Two-engine, single-threaded feasibility strongly evidenced; adoption and crash-atomic creation remain open. Lanes 1–6 pass on native, Chromium 151 and Firefox 153 (lane 7) from a byte-identical harness. Safari and WKWebView are not run. Production muniment is unchanged." — accurate: yes
- claims checked: 13 — holds: 9, stale: 4, unverifiable: 0

### Stale claims

- **"The fifth- and sixth-pass changes are uncommitted" (:20; state note :1320).** They were committed in `5cb55436` (2026-08-23, "Make the OPFS probe's benchmark and contract gates honest", ASCII contract, fail-closed gate, `settled && classification_ok`, paired control). It is an ancestor of the base.
- **The probe is untracked, and committing it is still Mark's call (§3 item 6 :143-149, §5.3b :379, §7.3 :1458).** The probe is tracked: twelve commits touch `ports/muniment-opfs-probe/` (`cc40c24f`, `5cb55436`, `10bcce1b`, `e008c0cc`, then `db9c613c`, `e9aadb4a`, `810864ee`, `b38986c2`, `82d020c0`, `9d778fc5`, `b84197a7`, `06423cab`). §6's own state note says it landed in `cc40c24f`.
- **"This plan lives in muniment's in-crate `design_docs/`" (§3 item 5 :140).** It lives in `design_docs/eidetic_docs/implementation_strategy/` (moved by `db9c613c`).
- **"wasm-bindgen pinned to 0.2.126" (§3 item 3 :130).** `ports/muniment-opfs-probe/Cargo.toml:46` is `=0.2.129` (`82d020c0`, 2026-10-04). The harness changed after the receipts, first on 2026-08-27 (`e9aadb4a`, licence headers on every harness file), then `810864ee`, `b38986c2`, `82d020c0`, `9d778fc5`, `b84197a7` and `06423cab`. The receipts last changed in `e008c0cc` (2026-08-23), so their `probe_source_sha256` names a superseded harness.

### Contradictions

- §3 item 6 ("I have not committed anything") contradicts §6's state note ("The probe landed in `cc40c24f`").
- §7.3 opens with "Chromium feasibility is proven", while §7.1 and the status line say two engines.

### Recommended action

- Mark the fifth/sixth-pass fixes committed in `5cb55436`.
- Retire the "commit the probe" question; it is committed.
- Correct the plan's location and the wasm-bindgen pin.
- State that the receipts pin the 2026-08-23 harness, not the current one.

### Notes

- Muniment still uses `redb = { version = "2" }` and has no OPFS backend.
- Receipts: `chromium_full`, `firefox`, `webkit_unsupported` and the superseded receipt exist. Both engine receipts show 707 fault trials and `unopenable_stubs: 0`.
- `Backend` still has no `get_many`. It gained `transact`, defaulting to `NotTransactional`, in `9ba9f790` (2026-09-09). The probe's adapters do not implement it, which matters for adoption.

## inker_docs/implementation_strategy/2026-06-15_engine_picker_and_pluggability_plan.md

- disposition: historical-unmarked
- status line: "Status: Phases 0–3 shipped + verified (route → activate → manage → pick): meerkat now routes both altitudes through `EngineRoutePolicy` (0a/0b), and the activation model (global default + per-session override), the apparatus engine manager, and the per-node picker all landed. Remaining: Phase 4 (no-handler UX + local-file sniff), Phase 2b (per-host overrides + per-session toggle), Phase 5 (verso flip), and the register-viewer harvest. *(The §2 "Findings" below are the point-in-time pre-Phase-0 state; the "gap" it names is now closed, see the Progress log.)* Page capture P1 (2026-08-30): landed at the Inker boundary. The shared contract now has host-minted request ids, viewport-only requests, typed PNG outputs, explicit unknown CSS facts, and a correlated hosted completion event. No engine declares capture support yet; adapters remain honestly unsupported." — accurate: no
- claims checked: 10 — holds: 7, stale: 3, unverifiable: 0

### Stale claims

- **"meerkat now routes both altitudes through `EngineRoutePolicy`" (:5).** Meerkat was removed in `c5f01064`. `is_surface_engine`'s only code consumer is Pelt (`ports/pelt/core/src/workspace.rs:12,730`); `EngineRoutePolicy` is also consumed by `crates/import/src/web_clip.rs:166,366`, the `mere::routing::route_policy()` facade (`crates/mere/src/routing.rs:43`) and Turnstone (`src/shell/mod.rs:226-227,504,862`).
- **The activation model, engine manager and per-node picker "all landed" (status).** That code went with meerkat. `EngineActivation`, `engine_pins` and `route_document_engine` have zero hits in code at the base. Only pandect's `disabled_engines` settings field survives (`settings_store.rs:112`, `application_settings_store.rs:66`), with no consumer outside pandect.
- **Phase 5's weld/graft listed as remaining (:199-200).** `GraftEngine` (`crates/inker/engines/graft-engine/src/engine.rs:52`) and `WeldEngine` (`weld-engine/src/engine.rs:51`) implement `SurfaceEngine` at the base (added `f5c3d9cb`).

### Contradictions

- DOC_README.md:505-512 repeats "Phases 0–3 shipped + verified".
- The scrying tile plan calls the `ScryingTileEngine`/`ProducerFactory` registry fold-in "the inker-picker plan's Phase 0". This plan says Phase 0 is done and calls that fold-in the "Phase-5 companion" (:200).

### Recommended action

- Mark the meerkat-era arc historical (removed in `c5f01064`).
- Either re-home Phases 2b, 4 and 5 onto Pelt or another live host, or extract them and archive.
- Record that the graft and weld `SurfaceEngine` impls exist.

### Notes

- The meerkat commits `d4a1350`, `1966183`, `e90825e`, `b4706c6`, `c5f63d8`, `a7e609e`, `0adca6e` and `06b6ac7` are ancestors of the base.
- These exist: `routing.rs:117` `route_filtered`, `:86` `per_host_overrides`, `is_surface_engine` (re-exported via `document_session_api::engine_ids`), `PageCaptureRequest`/`PageCaptureCompleted`, and `ScryingTileEngine` (`scrying-engine/src/engine.rs:54`).
- `capture_snapshot_png` has no hits. `request_page_capture` has only the default impl (`surface_engine.rs:1062`), and `route_degraded` has no hit in code.

## intel_docs/implementation_strategy/2026-07-08_index_burn_lift_plan.md

- disposition: historical-marked
- status line: "Status: P1 (kernel + parity) + P2 (crossover measured, see Findings) + P3 (keyed GPU accelerators over `VectorIndex` + routing thresholds) landed. The burn-lift is complete for the flat index; HNSW is the separate algorithmic path." — accurate: yes
- claims checked: 9 — holds: 8, stale: 1, unverifiable: 0

### Stale claims

- **The generic-over-backend signatures (:33 `cosine_top_k<B: Backend>`, :61 `nearest_over_index<K, B>`, :63 `affinity_pairs_over_index<K, B>`).** At the base, `crates/intel/esp/src/embed/index_burn.rs:53,113,144` take `device: &Device` with no `B` parameter. The Burn 0.22 migration `6ce399ea` (2026-08-20) changed this, after the "moved unchanged" date of 2026-08-09.

### Contradictions

- The mere-side `mere_docs/implementation_strategy/2026-07-06_intel_vector_index_burn_lift_plan.md:4` says "scoped, not started", while this plan (and DOC_README.md:357 for that plan) says Path A landed.

### Recommended action

- Update the signatures, or add a note that Burn 0.22 removed the backend generic.
- Complete: DOC_POLICY §8 calls for archiving. Reconcile the mere-side twin in the same step.

### Notes

`index_burn.rs` exists in esp, landed by `1283b4a8` and touched by `6ce399ea`. Features `index-burn`/`index-burn-wgpu` are at esp `Cargo.toml:80-81`. The thresholds 1024/4096 are at :39,:43. The listed tests exist, including `parity_ndarray_wgpu` and `crossover_cpu_flat_vs_gpu_batched`. There are no sibylla or vates crates in the tree (marked by the "Historical home" banner).

## mere_docs/implementation_strategy/2026-05-26_net_media_plan.md

- disposition: current
- status line: "Status: plan-only; no `net-media` crate exists. Media tracks, session integration, and decode remain unimplemented after browser WebRTC data-channel carriage moved to Murm transport." — accurate: yes
- claims checked: 7 — holds: 7, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

- There is no net-media crate in mere or under `repos/`. Mere's lock has no symphonia, rav1d or cros-codecs; rav1e is present, likely via image avif, which is unrelated.
- The browser WebRTC carrier plan and the archived netfetcher plan exist.
- `woodshed/ports/redshank/playback/Cargo.toml` uses symphonia, firewheel and `servo-media-player`.
- The family composition thesis brief (:498) records the media split.

## mere_docs/implementation_strategy/2026-06-03_host_p2p_wiring_plan.md

- disposition: superseded
- status line: "Status: Draft (for review, no code yet). The detailed elaboration of the [modular integration plan](2026-06-02_modular_integration_plan.md)'s S5 (comms surface + cheap p2p win): how the proven p2p substrate (transport + murm's `SyncedCabal` + tessera's `SyncedMoot`) wires into the meerkat host loop." — accurate: no
- claims checked: 6 — holds: 5, stale: 1, unverifiable: 0

### Stale claims

- **"Draft (for review, no code yet)" (:4).** The plan's own Progress records S5.0, S5.1 and S5.2 landed (2026-06-03 to 06-05). That host code (`crates/meerkat/src/sync.rs`) was then deleted in `c5f01064`.

### Contradictions

- The status line contradicts the plan's own Progress log.
- DOC_README.md:201 ("S5.0–S5.2 shipped, S5.3 → comms shell") does not mention the 2026-09-05 supersession by the Murm/Moot plan.

### Recommended action

- Restate the status as superseded by the Murm/Moot plan, with S5.0–S5.2 landed then removed with meerkat and S5.3 moved to the comms shell plan.
- Archive candidate.

### Notes

The Murm/Moot plan, modular integration plan, archived tessera and p2panda spike plans and the persona brief all exist. `P2pandaTransport::ticket`/`add_peer_ticket` are at `crates/murm/transport/src/p2panda_transport.rs:889,897`. `two_moots_converge_bootstrapped_by_ticket` is now in `crates/moot/mien/src/sync.rs:205`. The module names in Findings #2 are also out of date: `murm_replication` is now `stickleback::SyncedSpace` and `gemot::tessera` is a legacy alias. The 2026-09-05 note covers those, so they are not counted stale.

## mere_docs/implementation_strategy/2026-06-05_comms_shell_plan.md

- disposition: historical-unmarked
- status line: "Status: Largely implemented (the `shell/comms` domain crate and the meerkat pane exist; see Progress). The realization of Mere's comms surface in meerkat: a docked peripheral pane that surfaces unified communications (misfin mail + murm cabals, with room for mooting protocols), rendered through the same domain → host pattern as the chrome. It closes the [modular integration plan](2026-06-02_modular_integration_plan.md)'s gap #7 ("murm/moot unsurfaced") and its S5 (comms surface), and is the on-screen form of the inherited `COMMS_AS_APPLETS` family the [protocol architecture plan](2026-05-05_protocol_architecture_plan.md) names." — accurate: no
- claims checked: 9 — holds: 8, stale: 1, unverifiable: 0

### Stale claims

- **"the meerkat pane exist[s]" (:4).** The pane (`crates/meerkat/src/comms_host.rs` and the views) was deleted in `c5f01064`. No manifest at the base depends on `mere-comms` beyond the workspace entry (`Cargo.toml:273`). Turnstone's build log reports the `mere-comms` patch as unused.

### Contradictions

- DOC_README.md:202 "(Largely implemented.)" carries the same claim.
- `crates/shell/comms/Cargo.toml:7` still says "Surfaced by the meerkat comms pane".

### Recommended action

- Mark P6 removed with meerkat.
- Decide the comms crate's host, or archive with the remaining live pieces (misfin server socket, networked murm, send path) extracted.

### Notes

`crates/shell/comms` holds `ProtocolAdapter` (`adapter.rs:57`), `Comms`, `CommsPane` (`pane.rs:85`) and the misfin/murm adapters. errand is in-tree (`crates/system/errand`, `misfin_send`, README:56). misfin is crates.io 0.0.4; its local registry source has `deterministic_identity`, `identity_salt` and `MailboxStore` (`mailbox.rs:92`). murm's cabal `history`/`subscribe` are at `cabal.rs:277,337`. The linked contact identity brief and peripheral panes architecture exist.

## mere_docs/implementation_strategy/2026-06-05_node_navigation_lineage_wiring_plan.md

- disposition: historical-marked
- status line: "Status: Implementation plan — pre-build" — accurate: no
- claims checked: 7 — holds: 5, stale: 2, unverifiable: 0

### Stale claims

- **"pre-build" (:6).** Its Progress records the (b) anchor migration landed on 2026-06-06. `SharedNavigationMemory` is at `crates/graph/graph-kernel/src/graph/history.rs:121`, `Graph.nav` at `mod.rs:307`, `node_can_back`/`node_current_url` at `mod.rs:595,609`, and `node_history_back/forward` at `apply.rs:1924,1933`.
- **The header says the graphlets crate is `crates/graph/subgraph` (:3).** That crate was folded into mere in `61894570` (2026-09-23). `SessionSubgraphs` is at `crates/mere/src/subgraph.rs:39`.

### Contradictions

- The 2026-09-05 historical note ("rationale, not a current implementation map") sits beside the 2026-10-04 graph semantics ruling 2 amendment, which treats §1's model as live.
- DOC_README.md:203 still describes a "per-node nav-lineage substrate", which the shared memory replaced.

### Recommended action

- Restate the status as historical, with (b) landed 2026-06-06 and the meerkat host phases removed.
- Fix the subgraph path.
- Consider moving §1's live model into the graph semantics plan.

### Notes

`navigation_memory` survives only as an ignored legacy field (`persistence.rs:250`). `EdgeFamily` has 6 variants (`edge_taxonomy.rs:33-40`). Graph semantics plan ruling 2 exists (:119). node-lineage has been retired for stemma (graph-kernel `Cargo.toml:30-35`), which is marked as a historical citation.

## mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md

- disposition: current
- status line: "Status: first diagnostics implementation qualified by focused Mere tests. The bounded Apparatus core, optional Mesquite attachment and zero-capacity UX recorder repairs are implemented. Turnstone's separate redacted observation copy and Gloss/Inspector migration were published at Turnstone `d6b62adb`. The combined consumer cohort is now published at Turnstone `b2ead70a448948a1e8a9bde10b34f01524119606`, with optional Diagnostics inspection and contributed semantic automation/platform actions qualified. Redshank's real persistence worker now qualifies dispatch, execution and save-reply correlation, including failed IO and retry. Its later presentation adapter is requalified and published at Woodshed `cefc903`, with 68 desktop tests, strict Clippy, Wasm compilation and paired/default native captures. The earlier early-durability negative control remains qualified. Turnstone now passes 635 workspace tests with nine ignores, five exact optional participant checks and five reviewed native Sky/Diagnostics images. Its preceding 612-test migration cohort retains its original source identities. The September 30 continuation below separates shared presentation stamps, consumer readings, remaining whole-application causal links and human accessibility acceptance. The June design and receipts are historical evidence." — accurate: yes
- claims checked: 12 — holds: 11, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- The "Semantic implementation progress, 2026-09-29" subsection still says "Status: in progress" (:260). Its own "Final integration" paragraph closes its source-integration gates, and the 2026-09-30 table qualifies contributed semantics.

### Recommended action

- Mark the 2026-09-29 semantic subsection's status as closed or superseded by the 2026-09-30 section.

### Notes

- Mere `ca2351b3`, `bd5912fb`, `32edc2ad`, `ced161f1` and `8eca3e4c` are ancestors of the base.
- Sibling commits are in their HEADs: turnstone `d6b62adb` and `b2ead70`; woodshed `cefc903`, `a57085b` and `752c920e`; genet `19c20687` and `69a2383b`; knot `3dfb70b`, `855cb75d` and `c92ad044`.
- Symbols: `ObservationStore` (`apparatus/src/observation.rs:250`), `inspect_batch`/`project_inspection` (`inspection.rs:54,180`), and `Product::diagnostic_attachment` / `sampled_at_lane_frame` in mesquite.
- The receipts and `Code/testing/turnstone/{contributed-semantics,diagnostic-inspection/final}` exist.
- The Clip finding still holds: `InspectorIntent::ClipToKnot` carries no member (turnstone `src/inspector_pane.rs:32`).
- Unverifiable: whether Mark answered the next-slice scope question.

## mere_docs/implementation_strategy/2026-06-10_scrying_tile_plan.md

- disposition: historical-unmarked
- status line: "Status: reconciled to code 2026-06-23; X1 shipped, X2 input core shipped (chrome round-trip still open), X3 multi-tile lifecycle shipped (durable `compat_mode` not yet the source of truth), X4 untouched (Windows-only). Shipped via a session-local `engine_pins` map + a host-concrete producer pool (`meerkat/src/scrying_host.rs`), not the `ScryingTileEngine` / `ProducerFactory` registry seam, which has zero meerkat consumers — the Findings below predicted this; folding the pin into `inker::routing` and the producer into the registry is the inker-picker plan's Phase 0. The phase bodies record original intent; the Progress log carries shipped reality and the two display-model pivots that postdate the 2026-06-11 entry." — accurate: no
- claims checked: 9 — holds: 7, stale: 2, unverifiable: 0

### Stale claims

- **X1–X3 shipped via `meerkat/src/scrying_host.rs` (:8).** That file was deleted in `c5f01064`. Scrying surfaces are now hosted by Pelt (`ports/pelt/desktop/scrying_receipt.rs`, `dx12_surface.rs`).
- **graph-kernel `Node` carries `compat_mode` (node.rs:115) with `SetNodeCompatMode` (:53); X3's durable `compat_mode` is still open.** `compat_mode` left the kernel in `ebd92b89` (2026-07-09). It now lives in `crates/system/pandect/src/browser_node_state.rs:66`.

### Contradictions

- DOC_README.md:135 says the host spawns producers "via the existing scrying-engine factory seam". The plan says it shipped not through that seam.
- The plan names the registry fold-in as the picker's Phase 0; the picker plan calls Phase 0 done and the fold-in a Phase-5 companion.

### Recommended action

- Mark the meerkat-era record historical.
- Either re-home the open items (X2 chrome round-trip, registry fold-in, X4) onto Pelt, or extract and archive.
- Point `compat_mode` at pandect.

### Notes

`ScryingTileEngine`/`ProducerFactory` still have zero consumers outside scrying-engine; only the graft/weld test factories exist. netrender has `compose_external_texture`/`ExternalTexturePlacement` (`external_texture.rs:245`). wgpu-scry has `new_offscreen`, `new_attached` and `force_restart_capture`. The integration plan, verso charter and archived scrying plan exist.

## mere_docs/implementation_strategy/2026-06-18_graph_query_layer_plan.md

- disposition: current
- status line: "Status: slices 1+2 shipped and verified. SPARQL query over the focused graph, kernel-sourced and one-way (the kernel stays truth; this is a derived, read-only view for interop and exploration). A residual backlog follows; none of it blocks." — accurate: no
- claims checked: 14 — holds: 7, stale: 7, unverifiable: 0

### Stale claims

- **Slice 2 (the `>sparql` omnibar verb) shipped (status).** The host files `shell_eval.rs` and `command_drain.rs` were deleted in `c5f01064`. No `sparql` call exists in `ports/`; only the mere facade's `query` feature remains (`crates/mere/Cargo.toml:53`).
- **"node_quads — the single kernel→RDF projection" (:25).** `dataset_quads` (`linked-data/src/lib.rs:532`) is now the canonical projection for query. `node_quads` (:523) is default-graph-only JSON-LD input.
- **Slice 1 projects into "a fresh in-memory Oxigraph store" via `to_ox_quad` (:38).** It was rewired onto `spareval` over `oxrdf::Dataset` in `8e7ae82b` (2026-07-06). Oxigraph is now a test-only parity oracle (linked-data `Cargo.toml:23-30`, `query.rs:1-25`).
- **Backlog #2: "Today only Semantic edges + curated literals project; edge provenance… dropped" (:70-71).** `push_statement_metadata_quads` emits `rdf:reifies` reifiers with label, provenance and asserted-at (`lib.rs:204-240`). Oxigraph and oxrdf carry `rdf-12`.
- **Backlog #5: Turtle/N-Quads I/O open (:84).** `serialize.rs` has `to_nquads`/`to_trig`/`from_nquads`/`from_trig` (`70278fed`, 2026-07-06, under the petgraph-RDF plan).
- **The header says the graphlets crate is `crates/graph/subgraph` (:3).** It was folded into mere in `61894570`.
- **"Not committed (working tree)" (:106).** The code is committed and present at the base (`lib.rs:523`, `query.rs:50`, test `lib.rs:1142`).

### Contradictions

- DOC_README.md:234 repeats the Oxigraph store, the omnibar verb, "RDF-star … the substantive next step" and Turtle I/O as backlog.
- The petgraph-RDF plan (`2026-06-18_petgraph_rdf_plan.md`) says it "extends and partly subsumes" this plan, and its own status still lists statement metadata and "the direct SPARQL adapter" as ahead, though `lib.rs:204-240` emits the metadata and `8e7ae82b` rewired query onto spareval.

### Recommended action

- Rewrite "What shipped" for spareval and `dataset_quads`.
- Mark slice 2 removed with meerkat.
- Strike backlog #2 (partly) and #5 as done under the petgraph-RDF plan.
- Fix the subgraph path.

### Notes

`QueryResults::Graph` still errors (`query.rs:83`). Backlog #1 is still open: `ingest.rs:37` duplicates `RDF_TYPE` and there is no mapping module. Graph semantics ruling 3 exists (:132). The interaction model spine, the unified document host plan and the archived ingest/export plan exist.
