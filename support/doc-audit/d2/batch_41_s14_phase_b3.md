# Batch 41 — S14 pass, phase B3 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-06-27_swatch_primitive_plan.md | historical-unmarked | no | 15 | 7 | 7 | 1 |
| mere_docs/implementation_strategy/2026-06-28_smolweb_host_integration_plan.md | historical-marked | no | 6 | 4 | 1 | 1 |
| mere_docs/implementation_strategy/2026-06-29_reticulum_transport_plan.md | current | no | 16 | 8 | 7 | 1 |
| mere_docs/implementation_strategy/2026-06-30_commitment_proof_interface_plan.md | current | yes | 8 | 5 | 1 | 2 |
| mere_docs/implementation_strategy/2026-07-01_event_log_timeline_plan.md | superseded | yes | 7 | 6 | 0 | 1 |
| mere_docs/implementation_strategy/2026-07-04_burn_wgpu_flip_plan.md | current | no | 10 | 8 | 2 | 0 |
| mere_docs/implementation_strategy/2026-07-06_orrery_graph_intelligence_plan.md | current | no | 15 | 7 | 7 | 1 |
| mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md | current | no | 17 | 11 | 4 | 2 |
| mere_docs/implementation_strategy/2026-07-20_overmap_sessions_graph_plan.md | current | yes | 8 | 8 | 0 | 0 |
| mere_docs/implementation_strategy/2026-07-21_family_repo_merges_plan.md | superseded | no | 8 | 1 | 5 | 2 |
| mere_docs/implementation_strategy/2026-07-21_projection_proofs_plan.md | current | no | 14 | 8 | 5 | 1 |
| mere_docs/implementation_strategy/2026-07-22_identity-vault-ssh-agent_plan.md | current | no | 9 | 6 | 3 | 0 |
| **Totals** |  |  | **133** | **79** | **42** | **12** |

**Totals: 12 docs, 133 claims checked (79 holds, 42 stale, 12 unverifiable), 23 contradictions; 9 status lines wrong.**

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

Checked directly in this session: `be99eadb` is an ancestor of the base and
the transport's `reticulum` feature is `["dep:retinue", "dep:hkdf",
"dep:sha2"]` (reticulum); eidetic-core's package name is `eidetic`, the
mere-eidetic rename reversed (family repo merges); `93ca150f` is an ancestor
(projection proofs). The verifier refuted nothing and found 6 items true only
in part (the swatch plan's two curation-plan mismatches, one dropped and one
recast as a framing tension; Turnstone's smolweb themes; the reticulum
decision gate; the gate packs plan's line citations, twice); those records are
corrected.

## mere_docs/implementation_strategy/2026-06-27_swatch_primitive_plan.md

- disposition: historical-unmarked
- status line: "Status (2026-07-01): P1, P2, and P3a/b/c landed — see Progress below. P2b (cartography re-layout, `Scope`/`SwatchInstance` unification) and the rest of P3's done condition (live projection toggle, frontier ghosts, contextual detectors, Linked/Astroid crystallize, chip-click crystallize) are deferred slices, still open. P4/P5 now have further downstream slices through the roster/orrery relation-cell work: 2026-07-01 closed the "gyre topology and springs remain endpoint-pair scoped" gap (springs are now per-visible-relation-cell — see the [roster detail cards plan](../../archive_docs/2026-09-02_retired_plans/2026-06-29_graph_object_roster_detail_cards_plan.md)'s 2026-07-01 entry), but P4/P5's full done conditions (per-cell edge *thickness*, one shared element-model edge renderer between the orrery and the connections swatch, and the P5 `GraphDefault < GraphViewOverride < SelectionOverride` layered stack) remain open. P6 did not land as scoped (see Progress 2026-07-05): the gloss minimap migrated Scene->DOM through a separate, parallel implementation, not this plan's component; P7 remains unstarted." — accurate: no
- claims checked: 15 — holds: 7, stale: 7, unverifiable: 1

### Stale claims

- P1 "landed" (host-generic `swatch_view<S>`): meerkat, including `swatch.rs`, was deleted in `c5f01064` (2026-07-18, "The funeral: meerkat leaves the workspace"). No `fn swatch_view` exists in Mere. Its successor is Turnstone's `src/swatch_pane/mod.rs`, one pane over Cambium `GraphCanvasSwatch` presets (Gloss-minimap and Overmap, `presets.rs:45,58`), which Isometry also uses.
- P2 connections swatch (`render/connections.rs`, `FocusCardKind::Connections`, `connections_swatch_view`, `connections_spec_from`): deleted in `c5f01064`. No definition exists in any repo under `repos/`.
- P3b gesture (`Shell::crystallize_selection`, `ContextAction::CrystallizeSelection`): deleted in `c5f01064`.
- P3c chip strip (`ConnectionsSpec.shape_chips`): deleted in `c5f01064`.
- P6 "shipped differently" via meerkat `gloss_view.rs` (`minimap_view` / `recent_view`): the file and both functions are gone. The 07-05 decision stands; the code it describes does not.
- P5 names `orrery::build::visible_relation_edges`: the function now lives in `pictograph::canvas::seiche_bridge`. `crates/canvas/pictograph/src/canvas/build.rs:170` records the move; the caller is `lifecycle.rs:373`.
- The 2026-09-12 vocabulary note says the live names are "SessionSubgraphs in crates/graph/subgraph". They moved to `crates/mere/src/subgraph.rs` in `61894570` (2026-09-23). `crates/graph` now holds only graph-kernel and linked-data.

### Contradictions

- `DOC_README.md:244` says "per-cell physics/thickness … and spring relaxation are not built". The plan's own P5 status (lines 214-219) and its 2026-07-01 Progress entry say both are built. The tree agrees with the plan (pictograph `build_tests.rs:67`).
- `2026-08-03_graph_view_curation_and_interaction_plan.md:529-531` makes two claims that don't match this plan:
  - A framing tension rather than a conflict: it says Cambium plus Isometry now give "a new, real second-consumer seam", while this plan's revisit trigger, "a third swatch consumer" (lines 476-478), is never recorded as met.
- Internal: the status header is dated 2026-07-01 but cites the 2026-07-05 P6 reconciliation.

### Recommended action

- Add a note under the status:
  - Gone with meerkat (`c5f01064`): P1, P2, the P3b gesture, P3c and the P6 gloss path.
  - Survivors: the classifier and `SessionSubgraphs` in `crates/mere/src/subgraph*`, and `visible_relation_edges` in pictograph.
  - Open P4/P5/P7: point them at the graph-view-curation plan and Cambium `GraphCanvasSwatch`, or extract them and archive (fork).
- Fix the vocabulary note's path.
- Correct the spring-relaxation clause in `DOC_README.md:244`.

### Notes

Checked the deletion in `c5f01064`. A grep across every repo under `repos/` for `swatch_view`, `connections_swatch_view`, `crystallize_selection` and `connections_spec_from` found only the Turnstone swatch pane's unrelated private `swatch_view`. Surviving pieces: `classifier.rs:30,110` (`classify`, `classify_selection`), `subgraph.rs:192` (`record_session`), and forme `subgraph.rs:102-110` (`ProjectionSource`, which is a projection-source enum, not an edge-visibility stack). All linked plans resolve. Per-cell edge thickness in pictograph was not checked.

## mere_docs/implementation_strategy/2026-06-28_smolweb_host_integration_plan.md

- disposition: historical-marked
- status line: "Status: P1–P3 landed 2026-06-28 (reconciled 2026-07-01; genet `1bbbfdb`, `0b7ca87`, `5c07ad5`; mere `476880b`, `0dd0c3e`, `3eed418`, `8dc3683`) — render, theme, scroll, link nav all wired, but compile-verified only, no headed run yet. The 2026-07-01 review left open items (see Open questions): theme hard-coded to `App` against the settled Site-default design, band-scroll cadence untested, and the trust-posture gap (owned by the smolweb fidelity plan's Workstream 2). P4 optional, unstarted. Separately, the scripted-live follow-on's `ResourceFetcher` trait mismatch at `content/actor.rs:33` was fixed 2026-07-01 (render-ladder plan's lane — see its progress log; `--features scripted` now compiles, 5/5 tests). The smolweb feature itself builds green." — accurate: no
- claims checked: 6 — holds: 4, stale: 1, unverifiable: 1

### Stale claims

- The status line presents the open items (App theme, band-scroll cadence, P4 "unstarted") as live, but their host is gone. Meerkat's `content/handlers.rs` and `content/actor.rs` were deleted in `c5f01064` (2026-07-18). The historical note (lines 15-17) says so; the status line does not. Turnstone's Reader and Micron lanes use `SmolwebTheme::System` (`src/shell/mod.rs:96`, `src/nomadnet.rs:224`); its eight engine-native smolweb lanes use `SmolwebTheme::default()`, which is `Site` (`shell/mod.rs:103`; tabard `smolweb.rs:25-26`), the settled Site-default design.

### Contradictions

- `DOC_README.md:281` still reads "planning (with Mark); meerkat code deferred to a clean window".

### Recommended action

- Fold the historical note into the status line: P1–P3 landed on main 2026-07-01; host removed in `c5f01064`; open items not carried forward.
- Update `DOC_README.md:281` to match.
- Archive if Mark agrees (fork).

### Notes

All five mere commits (`476880b`, `0dd0c3e`, `3eed418`, `8dc3683`, `737e0cd`) are ancestors of `535bca11`. All five genet commits (`1bbbfdb`, `0b7ca87`, `5c07ad5`, `1856486`, `5f50134`) are on genet main. Unverifiable: "landed 2026-06-28". Every cited commit carries author and committer dates of 2026-07-01. The linked nematic plans and the render-ladder plan resolve.

## mere_docs/implementation_strategy/2026-06-29_reticulum_transport_plan.md

- disposition: current
- status line: "Status: partial: P0/P1 and the bounded test row landed and verified; documentation and the decision gate remain open." — accurate: no
- claims checked: 16 — holds: 8, stale: 7, unverifiable: 1

### Stale claims

- **Beechat `reticulum` 0.1.0 as the backend.** This appears in the Goal (lines 11-12), in "the probe stays pinned to Beechat 0.1.0" (lines 59-61), and in Progress 2026-06-29 (`reticulum = "0.1"`). `be99eadb` (2026-07-15) replaced it with retinue:
  - The feature is now `reticulum = ["dep:retinue", "dep:hkdf", "dep:sha2"]` (`crates/murm/transport/Cargo.toml:17`).
  - The workspace pins retinue 0.2.0 at git rev `fa4f925` (`Cargo.toml:567-573`).
  - Retinue's v0 plan records "R5 — Mere adoption. DONE 2026-07-15" (`retinue/design_docs/2026-07-06_retinue_v0_plan.md:269`).
- **Identity mapping** (lines 100-105, both keys HKDF-derived): only the X25519 half is HKDF-derived now. The Ed25519 half is the Mere master seed itself (`reticulum_transport/keys.rs`, `derive_identity`), changed in `a69e4867` (2026-07-27).
- **Authenticated binding** `app_data = PeerID || signature` (lines 107-126): app data is now intentionally empty. The 96-byte form is read only as legacy (`announce.rs:38-51`, `:56-80`), changed in the same commit.
- **protoc required** (Progress 07-01, risk table): that was Beechat's `build.rs`. The retinue crate has no `build.rs` (only a firmware one), and no tonic/prost.
- **Retinue repo "dual MIT/Apache-2.0"** (line 65): the retinue README's License section says MPL-2.0.
- **"crates.io-only dep"**: Mere takes retinue by git rev (`Cargo.toml:573`).
- **"Decision gate remains open"**: the Direction section's trigger fired: Mere's own implementation replaced Beechat behind the same trait (`be99eadb`). Phase 3's decision is unrecorded; the feature is default-off, enabled by `commons-spine` only as a dev-dependency (`crates/moot/commons/Cargo.toml:40-44`) and by the `murm-direct-phy` probe as a normal one.

### Contradictions

- `DOC_README.md:256` says "active probe … using the Beechat Rust `reticulum` crate".
- The reference discipline (lines 50-54: treat the Python reference as a black-box oracle only) conflicts with retinue's current notice. `THIRD_PARTY_NOTICES.md`, section "Reticulum", reports a first limited review of RNS source on 2026-09-26 and says "Comparative review is the current scope".

### Recommended action

- Rewrite the status:
  - P0/P1 landed on Beechat 2026-07-01.
  - Backend replaced by retinue (`be99eadb`).
  - Binding revised (`a69e4867`).
  - Still open: a forged legacy-binding test, a README section on ALPN, announce and binding, and the DOC_README update.
- Mark the Goal, Findings and binding sections historical, or defer to retinue R5 and archive (fork).
- Point the reference-discipline section at retinue's notice instead of restating it.
- Update `DOC_README.md:256`.

### Notes

Checked:
- module wiring at `lib.rs:63-64` and `impl Transport for ReticulumTransport` at `reticulum_transport.rs:493`
- the five tests at `tests.rs:32,55,85,98,181`, which include the three the plan names
- no forged or mismatched binding test exists
- the README covers only the feature row (`README.md:35,83,108`)
- diffs of `be99eadb` and `a69e4867`

Unverifiable: whether retinue 0.0.1 was published to crates.io (no network).

## mere_docs/implementation_strategy/2026-06-30_commitment_proof_interface_plan.md

- disposition: current
- status line: "Status: Shared type interface landed 2026-07-13; proof backends remain planned." — accurate: yes
- claims checked: 8 — holds: 5, stale: 1, unverifiable: 2

### Stale claims

- The Decision code names `CommitmentDomain::TesseraReceipts` (line 37) and has no `DigestV1`. The crate has `StandingReceipts` with `#[serde(alias = "TesseraReceipts")]` (`crates/system/proofs/src/lib.rs:77-78`, after the 2026-08-31 terminology ruling `50b39f95`) and `DigestV1` (`lib.rs:61`). The samples are not labelled illustrative or compile-ready, which DOC_POLICY §8 requires.

### Contradictions

- Done-condition 3 and the ownership sentence (lines 11-12) target kith grants. The kith plan was retired on 2026-09-02, "subject deleted or abandoned" (`DOC_README.md:148`).
- `DOC_README.md:139` carries no status and omits the 2026-07-13 landing.

### Recommended action

- Label the code samples illustrative and point at `crates/system/proofs/src/lib.rs`.
- Say in the status that done-conditions 1–3 are unbuilt.
- Settle the kith target (fork).
- Add a status to `DOC_README.md:139`.

### Notes

- `2fb78c43` (2026-07-13) adds `crates/system/proofs`; the types are at `lib.rs:17,26,59,72,87,116`.
- Retention binds a `BlobRef` plus a `StorageCheckpoints` commitment (`mesh/src/retention.rs:140`, `gemot/src/moot/records/retention.rs:148`).
- None of these exist in any `.rs` file: `InclusionProof`, `AppendProof`, `SetRelationProof`, `MootEpochHeader`, `RoleWitness`, `DelegationProof`.
- Unverifiable:
  - Done-condition 5 (p2panda wire docs): no such statement found; absence not proved.
  - Done-condition 4: `StorageChunks` is unused, so whether chunk and checkpoint-log commitments are "distinguished" is a judgment call.

## mere_docs/implementation_strategy/2026-07-01_event_log_timeline_plan.md

- disposition: superseded
- status line: "Status: Superseded for implementation; retained as a historical decision record. Spun out of [alembic_implementation_plan](2026-06-24_alembic_implementation_plan.md) §E, per that plan's own flag ("large enough that E may spin to its own plan once C and D land"). Slices A-D have since landed (C's by-sessions eviction and D's Athanor P1/P2 both shipped 2026-06-30/07-01); that made E ready when this historical plan was drafted, before the later replay and pointer substrates superseded its implementation." — accurate: yes
- claims checked: 7 — holds: 6, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- "Slices A-D have since landed" disagrees with the alembic plan's own status, "slices A-C landed" (`2026-06-24_alembic_implementation_plan.md:3`). `DOC_README.md:250` says "A-D shipped".
- Internal: the 2026-07-01 Progress entry (redirect recording into each mutator's body; "Not started") contradicts the body's correction that `apply_graph_delta` is the single chokepoint (lines 47-55, 128-136).

### Recommended action

- Add a dated Progress line recording the chokepoint correction and the 2026-08-03 supersession. Archiving is a fork.

### Notes

The supersession is recorded at `graph_view_curation` plan lines 519-523. `GraphJournal` is at `graph-kernel/src/graph/journal.rs:178`, `CapturedDelta` at `capture.rs:43`, and Cambium `pointer_capture` at `cambium-genet-winit-host/src/harness.rs:365`. The linked plans and the architecture doc resolve. Unverifiable: slice D (Athanor P1/P2).

## mere_docs/implementation_strategy/2026-07-04_burn_wgpu_flip_plan.md

- disposition: current
- status line: "Status: P0-P3 landed and measured; P4 aether wasm receipt green (2026-07-05). embed's wasm receipt is its own follow-on slice (getrandom-0.3-via-ahash + the tokenizers/onig C dependency); see Findings." — accurate: no
- claims checked: 10 — holds: 8, stale: 2, unverifiable: 0

### Stale claims

- "embed's wasm receipt is its own follow-on slice", plus Findings "P4 embed receipt deferred": the ESP consolidation's E0 (`1283b4a8`, 2026-08-09) closed it.
  - Wasm builds use tokenizers' `unstable_wasm` path (`crates/intel/esp/Cargo.toml:46-47`); Oniguruma is native-only (`:43-44`).
  - `bert-wgpu` passes on `wasm32-unknown-unknown` (`intel_docs/technical_architecture/2026-08-09_feature_target_matrix.md:38,41`).
- The D1 versions (burn 0.21, cubecl-wgpu 0.10, `wgpu = "29"`) are out of date. The tree is on burn `=0.22.0-pre.4` (`esp/Cargo.toml:25`, `numen/Cargo.toml:20-21`), cubecl-wgpu 0.11.0-pre.4 and wgpu 30.0.1 (`Cargo.toml:524`). There is still a single wgpu in `Cargo.lock`, so the unification conclusion holds.

### Contradictions

- The ESP plan's E0 (`2026-08-08_esp_consolidation_plan.md:103-112,332-347`) treats earlier wasm receipts as historical and re-ran them; this plan still calls embed wasm open.
- `DOC_README.md:362` has no status.

### Recommended action

- Change the status to complete: P0–P4 done; embed wasm closed by ESP E0; crates renamed embed→esp and aether→numen; burn 0.22 belongs to the burn 0.22 migration plan.
- Archive (fork).

### Notes

- Present: `bert-wgpu` (`esp/Cargo.toml:83`) and `field-burn-wgpu` (`numen/Cargo.toml:38`), with getrandom 0.4 `wasm_js` and uuid `js` (`numen/Cargo.toml:32-33`).
- Test files: `numen/src/lower_burn/tests_wgpu.rs` and `esp/src/embed/bert/wgpu_parity.rs`.
- Wiring: `eidetic-recall` takes `--backend wgpu` (`eidetic-search/examples/eidetic-recall.rs:211-215`), and the `OpaqueBlob` raw-bytes override is at `eidetic-core/src/models/mod.rs:85`.
- The timing tables were not re-run.

## mere_docs/implementation_strategy/2026-07-06_orrery_graph_intelligence_plan.md

- disposition: current
- status line: "Status: P1-P6 landed (2026-07-06). Force pass (P1-P3) + semantic-arrangement bridge (P4) + live meerkat content-affinity wiring (P5) + blended affinity and content-text enrichment (P6) are all in and tested. Open: the `semantic-embeddings` BERT-provider upgrade (a separate slice), the off-thread embedding actor (raw-body text + the intel-index lift), and the P3 live force-pass injection (held — niche)." — accurate: no
- claims checked: 15 — holds: 7, stale: 7, unverifiable: 1

### Stale claims

- "gyre stays burn-free: the force pass lives in aether" and `aether::forces::repulsion` / `repulsion_reference`: these moved to `seiche::tensor_forces` (`crates/conatus/seiche/src/tensor_forces.rs:104,224`) in `eae87153` (2026-08-31). They sit behind optional `tensor-burn` features (`seiche/Cargo.toml:38-39,54-55`).
- `RepulsionSolver` as "a plain `Fn(&[f32],&[f32],f32,f32)->(Vec,Vec)`": it is now `Arc<dyn Fn(&[f32], &[f32], RepulsionRequest) -> Result<RepulsionForces, RepulsionSolverError>>` (`seiche/src/lib.rs:397-401`).
- `aether::forces::repulsion_wgpu`: renamed `repulsion_wgpu_roundtrip` and documented as a staging helper (`tensor_forces.rs:11-20,168`).
- "P3 live force-pass injection (held — niche)": already done. Pictograph's `Canvas` and `PhysicsBoard` take a host `PhysicsDevice` with lagged GPU repulsion (`pictograph/src/canvas/physics_device.rs`, `c57e820b`, 2026-10-02, physics catalog P5c).
- "live meerkat content-affinity wiring (P5)": the meerkat driver was deleted in `c5f01064`. `set_content_affinity` (pictograph `strategy.rs:503`) has no caller in Mere or Turnstone.
- "`StubEmbeddingProvider` (deprecated alias kept)": no `HashedEmbeddingProvider` alias exists in code.
- "intel-index lift" listed as open: Path A has landed (`esp/src/embed/index_burn.rs:144`; `DOC_README.md:357`).

### Contradictions

- `DOC_README.md:356` repeats the same stale claims.
- `2026-09-02_physics_catalog_plan.md:356` (P5c) records the GPU host wiring that this plan lists as held.

### Recommended action

- Rewrite the status:
  - P1–P4 and P6 survive under new homes: seiche, esp, pictograph.
  - P5's driver was removed in `c5f01064`.
  - The P3 injection was superseded by physics catalog P5c.
  - The index lift has landed.
- Extract BERT content affinity and the off-thread actor, then archive (fork).
- Update `DOC_README.md:356`.

### Notes

Checked:
- `set_repulsion_solver` at `seiche/src/lib.rs:659`
- `AffinitySpring` at `seiche/src/affinity_force.rs:56`
- `affinity_pairs` at `esp/src/embed/affinity.rs:33`
- `LexicalEmbeddingProvider` at `esp/src/embed/lexical.rs:79`
- `StubEmbeddingProvider` at `esp/src/embed/stub.rs:36`
- `AffinityBlend` and `blend_affinity_pairs` at pictograph `canvas.rs:75,93`, and `set_affinity_blend` at `strategy.rs:517`
- a grep across Mere and Turnstone for `ContentArrangement` and `set_lagged_repulsion`

Unverifiable: the BERT upgrade, since its host is gone.

## mere_docs/implementation_strategy/2026-07-17_participant_gate_packs_plan.md

- disposition: current
- status line: "Status: partially implemented: the attributed gate, nested-graph substrate, and physical transfer receipts landed; pack schemas and broader consumer admission remain open." — accurate: no
- claims checked: 17 — holds: 11, stale: 4, unverifiable: 2

### Stale claims

- "pack schemas … remain open": B4 is complete. `mere.pack/v1`, `PackManifest` and `verify_pack` are at `crates/eidetic/eidetic-core/src/pack.rs:86,145` (`a3a246a8`, 2026-07-22).
- Line 141 says the binding is a sidecar and that "mere's web Node does not implement" `GraphBearing` (line 139's "does not implement GraphBearing yet" is stale in that word only). The kernel `Node` does implement it (`graph-kernel/src/graph/chart.rs:68`), with `SetNodeNested` (`apply.rs:253`) and `bear_nested` (`node_props.rs:82`), per the plan's own 2026-07-22 ruling. The sidecar was removed in `10084b3`.
- §5 "B5 is blocked on R4 completion" (line 112): the plan's own 2026-07-22 Progress and `5a33157f` record the pack pair proven, including over RF (`pack-distribution/src/bin/rf_pack_pair.rs`; `Code/testing/mere/rf_pack_pair/run2.log` exists).
- `session_runtime::denizen_facets`: session-runtime was renamed pandect in `441e70f0`. The module is at `crates/system/pandect/src/lib.rs:69`.

### Contradictions

- `DOC_README.md:195` still says "design (with Mark)" and that distribution is "gated on R4".
- Internal: the B3 build-order bullet (line 144) has no done marker, though Progress says B3 is complete. Line 141 contradicts the 2026-07-22 containment ruling.

### Recommended action

- Change the status to: B0–B4 complete; B5 pair and curation proven over RF. Still open:
  - the B5 tessera receipt on a live moot
  - revocation (OQ3)
  - the mere-native meadowcap layer
- Annotate lines 112, 139, 141 and 144.
- Fix the pandect path.
- Update `DOC_README.md:195`.

### Notes

- **Commits:** all mere commits (`953bf09`, `10084b3`, `e54ca8cf`, `a3a246a8`, `5a33157f`, `a4da519`, plus chartulary `2ced0fb`/`3361f0e` and servitor `1af0c91`) are ancestors of the base. Chartulary and servitor now live in-tree. Turnstone `8b3ad31` and `9727081` are on its HEAD.
- **Symbols:**
  - `commit_batch` (`commit.rs:166`), `Container.nested` (`container.rs:79`), `AttributedDelta` (`journal.rs:168`)
  - servitor `Gate` (`gate.rs:222`)
  - `MootAuthority` (`typed_authorization.rs:125`)
  - `Grant::from_authority` (`capabilities.rs:121`)
  - `world app-core` (`world.wit:168`)
  - turnstone `ring.rs:44,110,299`
- **Unverifiable:**
  - "broader consumer admission" is never defined in the plan.
  - The B5 tessera residue was not found; absence not proved.

## mere_docs/implementation_strategy/2026-07-20_overmap_sessions_graph_plan.md

- disposition: current
- status line: "Status: RUNGS COMPLETE 2026-07-20 (O0-O3 landed; held items dispositioned below). Executes the overmap ruling (Mark, 2026-07-19 — recorded in the [node-dissolution facets plan](../../archive_docs/2026-08-06_completed_plans/2026-07-18_node_dissolution_facets_plan.md), "The overmap" section): sessions are container nodes in a graph one level up; fork is node lineage at that level; the switcher becomes a graph view." — accurate: yes
- claims checked: 8 — holds: 8, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record. The plan is complete, so archiving is a fork.

### Notes

- **Commits:** mere `3f85112` adds `list_trash` / `restore_from_trash` (`pandect/src/manifest_store.rs:332,369`). Turnstone `058e6aa` and `4df6c56` are on its HEAD.
- **Turnstone code:**
  - `src/overmap.rs:55` is still a pure derivation.
  - `heal_nil_graph_ids` is at `session_lifecycle.rs:57`.
  - `PaneContent::Overmap` is at `panes/mod.rs:198`.
  - `RecoverSession` and `TrashSession` are at `action.rs:560,1082`.
- **Receipts:** both overmap `.scn` scenario files exist.
- **Docs:** `DOC_README.md:176` agrees.

## mere_docs/implementation_strategy/2026-07-21_family_repo_merges_plan.md

- disposition: superseded
- status line: "Status: P1-P3 and the `mere-eidetic` publish follow-on landed; donor GitHub archiving and later published-metadata refreshes remain pending." — accurate: no
- claims checked: 8 — holds: 1, stale: 5, unverifiable: 2

### Stale claims

- "mere: 14 git-dep lines … repointed to `eidetic.git`": `6a37de6b` (2026-07-23, "Absorb nine component families into mere") made every family crate a workspace path dependency (`Cargo.toml:192,226,427,487`). No `eidetic.git` or `conatus.git` reference remains.
- "mere-eidetic rename": reversed in `ca798151` (2026-09-18). `crates/eidetic/eidetic-core/Cargo.toml:2` reads `name = "eidetic"`.
- "Next publish picks up the [family] repository field": the manifests already say `repository = "https://github.com/merely-made/mere"` (`chartulary/Cargo.toml:10`, `seiche/Cargo.toml:7`, `servitor/Cargo.toml:8`).
- "retinue + tulle + tucket + sennet stay separate repos": they merged into one workspace on 2026-07-23 (retinue README, History section; mere `Cargo.toml:570-572`).
- "deliberate standalones (… personae, armillary, vates, sibylla, servitor) as-is": all were absorbed into mere by `6a37de6b`.

### Contradictions

- `DOC_README.md:213` still lists "open follow-on: … renames to mere-eidetic".
- `2026-07-23_repo_consolidation_plan.md:38` ("Mere is the platform; the family repos … remain its components") supersedes this plan's topology, but neither plan names the other.

### Recommended action

- Mark this plan superseded by the repo consolidation plan and record the `ca798151` reversal.
- Retire the P4 metadata follow-on.
- Update `DOC_README.md:213`.
- Archive. Whether the donor-repo archiving task is still live is a fork.

### Notes

Read the commit messages of `6a37de6b` and `ca798151`. `crates/eidetic` now holds chartulary, eidetic-core, hagiograph and muniment; `crates/conatus` holds conatus, modulus, nisus, numen, quint-shaders and seiche. Unverifiable: the mere-eidetic 0.0.1 publication and the donor GitHub archiving.

## mere_docs/implementation_strategy/2026-07-21_projection_proofs_plan.md

- disposition: current
- status line: "Status: P3/P4/P5 code slices are landed in the working tree. P3 has a green two-process headed receipt, including score restore after restart. Live radio facts remain correctly deferred because no radio product exposes them. Executes the five-proof sequence from the [projection_engine_prior_art_brief](../research/2026-07-21_projection_engine_prior_art_brief.md) §9. Proof 1 landed same-day. The scenograph family repo is founded ([mark-ik/scenograph](https://github.com/mark-ik/scenograph), commit `5a730e1`: `sceno` core / `scenomise` choreography / `scenotime` runtime / `scenograph` facade, MIT/Apache ed2024, name-holding; crates.io publication is Mark's step)." — accurate: no
- claims checked: 14 — holds: 8, stale: 5, unverifiable: 1

### Stale claims

- "landed in the working tree": the work is committed (`f78ea81e`, 2026-07-22; `93ca150f`, 2026-07-26, "Projection proofs are done").
- "scenograph family repo … crates.io publication is Mark's step": the family was absorbed into mere by `6a37de6b`, then moved to `crates/cambium/scenes/*` by `3e4d5098`. The 0.0.3 release plan is marked complete (`2026-07-24_scenograph_0_0_3_release_plan.md:3`).
- "display names live in two tables — `arrangements::registry` and `CANVAS_LAYOUT_STRATEGIES`": the arrangements crate was absorbed in `cc40c24f`. Its successor, the scenomise catalog, shows "L-system" and "Kanban" (`crates/cambium/scenes/scenomise/src/catalog.rs:200,202`). That disagrees with the ratified Fractal and Columns, which `CANVAS_LAYOUT_STRATEGIES` carries (`pictograph/src/canvas/cartography_scene.rs:146-160`).
- "Still open: a checkmarked layout picker (palette-only today)": Turnstone's Arrange pane exists (`src/arrange_pane.rs`, `f7a5388`, 2026-09-03).
- The 2026-10-03 annotation says "the code is unchanged until it rules". F18–F20 were ruled the same day (`2026-10-02_dynamics_grammar_plan.md:88-92`), and `270172db` (17:37 that day) removed `set_arrangement_pull` and `sync_anchor_force`.

### Contradictions

- `93ca150f` and `DOC_README.md:158` ("P1-P5 landed") disagree with "in the working tree".
- Internal: lines 173-176 still call the picker work owed, while the Findings at lines 99-103 mark it done. The P2 bullet in the Plan section has no done marker.

### Recommended action

- Change the status to: P1–P5 complete on main (`93ca150f`); scenes now at `crates/cambium/scenes`. Still open:
  - live radio facts
  - the naming drift between the two display-name tables
  - the arrangement pull, now under the dynamics grammar plan
- Fix the 10-03 annotation.
- Reconcile the catalog names (fork) and archive (fork).

### Notes

- **Symbols:** `restore_projection_score` (`strategy.rs:263`), `set_physics_paused` (`input.rs:649`).
- **Tests:** `score_and_physics.rs:122,155`.
- **Scene family:** sceno `Score` (`score.rs:71`), scenomise `solve` / `relax`, `coastal_map.json`, `AnchorSpring` (`anchor_force.rs:43`).
- **Isometry:** `Overmap` no longer has a `layout` function; `overmap_positions_relaxed` is at `isometry-views/src/overmap/scene.rs:72`.
- **Receipts:** the proof1 PNGs exist.
- **Turnstone:** `palette_actions` reads `CANVAS_LAYOUT_STRATEGIES` (`action.rs:626,722`).
- **Unverifiable:** "no radio product exposes them". Retinue's position-disclosure plan has PD1/PD2 implemented in software (`retinue 2026-09-01_position_disclosure_plan.md:4-7`).

## mere_docs/implementation_strategy/2026-07-22_identity-vault-ssh-agent_plan.md

- disposition: current
- status line: "Status: V1-V3 landed and the personae fold executed; V4 is demand-driven and V5 sync remains deferred." — accurate: no
- claims checked: 9 — holds: 6, stale: 3, unverifiable: 0

### Stale claims

- "V4 is demand-driven": the passwords and TOTP that V4 names now have a taxonomy under the chatelaine plan (`dramatis_docs/implementation_strategy/2026-10-01_chatelaine_cxf_plan.md:4-12`). Its P1–P3 have landed (`da3c50bc`, `3e4992ec`, `ff68e86c`).
- The CLI is described as "gated on a new `ssh` feature": `personae-vault` now requires `agent` (`crates/dramatis/personae/Cargo.toml:27-30`, `4c4ce3bc`, 2026-10-04).
- The resident-host adoption is written as pending: castellan already hosts `VaultAgent` with approval (`ports/castellan/src/authority.rs:26,184-238`, `b53ba480`, 2026-08-14).

### Contradictions

- `2026-07-22_graphshell_remote_projection_host_plan.md:544-552` (G8, folded into H4) took over the resident-host ruling.
- `DOC_README.md:124` has no status.
- The header of `install-agent-windows.ps1` still says "Until then".

### Recommended action

- Rewrite the status:
  - V1–V3 landed; fold executed.
  - Host in castellan (`b53ba480`).
  - Item types continue under chatelaine.
  - V5 deferred.
- Fix the CLI feature note.
- Decide whether the interim install scripts retire (fork).
- Add a status to `DOC_README.md:124`.

### Notes

Checked the following in `crates/dramatis/personae`:
- `SealedProfileStorage` (`sealed_profile_storage.rs:59,104`) and `profile_wire.rs`
- `VaultAgent` (`agent.rs:77`), `bootstrap.rs`, `ssh_slot.rs`, `device_loss_note` (`vault.rs:119`)
- the `personae-agent` and `personae-vault` bins (`Cargo.toml:19-30`)

The only `IdentityStorage` implementations are Passphrase, Sealed, InMemory, `&T` and `Box<T>`; none is replicated, so V5 is still deferred. The linked docs resolve.
