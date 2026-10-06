# Batch 44 — S14 pass, phase B6 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-08-23_projection_receipts_plan.md | current | no | 15 | 13 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-24_doc_policy_consolidation_plan.md | current | yes | 13 | 11 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-25_browser_webrtc_carrier_plan.md | current | yes | 14 | 12 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-31_terminology_and_crate_folds_plan.md | current | yes | 17 | 13 | 3 | 1 |
| mere_docs/implementation_strategy/2026-09-02_autodiff_lora_trainer_plan.md | current | yes | 11 | 10 | 1 | 0 |
| mere_docs/implementation_strategy/2026-09-02_distillery_projection_walk_plan.md | current | no | 14 | 11 | 3 | 0 |
| mere_docs/implementation_strategy/2026-09-02_physics_catalog_plan.md | current | no | 9 | 8 | 1 | 0 |
| mere_docs/implementation_strategy/2026-09-02_platform_boundary_and_repository_topology_plan.md | current | yes | 13 | 12 | 1 | 0 |
| mere_docs/implementation_strategy/2026-09-11_recursive_query_experiments_plan.md | current | yes | 11 | 10 | 1 | 0 |
| mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md | current | no | 8 | 7 | 1 | 0 |
| mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md | current | no | 8 | 4 | 3 | 1 |
| mere_docs/technical_architecture/2026-08-14_tactile_tier_plan.md | current | yes | 6 | 6 | 0 | 0 |
| **Totals** |  |  | **139** | **117** | **20** | **2** |

**Totals: 12 docs, 139 claims checked (117 holds, 20 stale, 2 unverifiable), 15 contradictions; 5 status lines wrong.**

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

Checked directly in this session: G1's merge `39787d82` is an ancestor of the
base (dynamics grammar); `crates/conatus/seiche/src/laws/density.rs` exists
and `9b576c84` is "Merge density-cpu" (physics catalog). The verifier refuted
nothing and found 7 items true only in part (the DOC_POLICY line citations;
the terminology plan's "seven" consumer commits, plausible after all, and
Mesocosm `cff9b71`, checkable and stale; the physics catalog's landing merges;
the platform plan's genet pin, which main did carry for three days; the
reservoir's headed verdicts, which hold; the dynamics plan's G2 mention);
those records and their counts are corrected.

## mere_docs/implementation_strategy/2026-08-23_projection_receipts_plan.md

- disposition: current
- status line: "Status: active; Waves 1 and 2 are complete. Mer3ly is the headed first consumer and gazette Ledger the heterogeneous second; the Gazette port promotion and this plan landed at `0da3b8ba`. FT6, FT7, and FT8 closed 2026-08-25. Wave 3 remains explicitly gated." — accurate: no
- claims checked: 15 — holds: 13, stale: 2, unverifiable: 0

### Stale claims

- The status line (and DOC_README.md:93) says this plan landed at `0da3b8ba`. That commit (2026-08-24, "Found the Moot and Alembic ports; promote gazette to a port") does carry the gazette promotion and `ports/gazette/src/ledger.rs`, but not this plan. `git log --all -- '*projection_receipts_plan*'` shows the file was added by db9c613c (2026-08-24, a design_docs sweep).
- The banner at line 3 says "the graphlets crate is crates/graph/subgraph". That directory does not exist at base. Subgraph is now a module of mere, `crates/mere/src/subgraph.rs` (`SessionSubgraphs` at :39), folded in by 61894570 on 2026-09-23.

### Contradictions

- The status line dates FT6's close to 2026-08-25. §4 FT6 (line 182) and Progress (line 302) both say 2026-08-24.
- DOC_README.md:93 repeats the 0da3b8ba plan-landing claim.

### Recommended action

- Correct the status line: cite db9c613c for the plan (or drop that clause) and date FT6 2026-08-24. Amend the line-3 banner to name `crates/mere/src/subgraph.rs`. Fix DOC_README.md:93 to match.

### Notes

These Mere commits are ancestors of base: 0da3b8ba, 7aa64240, 41ff2aba, 6cc014c4, 302bbe72. mer3ly 5410512 and 4c42847, and retinue 8cea8f9, are on their repos' origin/main. Symbols present: `CoordinatedSelection` (crates/chirograph/src/lib.rs:236), `ShelfmarkV1` (crates/incipit/src/lib.rs:79), `FrozenScene`, `LocalCarrier`, `MemoryTransport`, `sceno::Scene`, `ports/gazette/src/ledger.rs`, and retinue's `mixed_realization_uses_one_scene_focus_and_action_model`. The adoption plan records A2 closed (:331, :550). No FT9/FT10 work or field-data consumer exists anywhere, so everything is done except the gated wave 3.

## mere_docs/implementation_strategy/2026-08-24_doc_policy_consolidation_plan.md

- disposition: current
- status line: "Status: Complete 2026-09-06; phases A-D landed. A, B and C landed 2026-08-24. D1, the original D2 judgment pass and depth re-pass, and D4 completed 2026-09-02. D3 closed 2026-09-06 with the repository's self-testing audit at zero failing findings. The D2 identity closeout then preserved the 281-record aggregate, added 34 supplemental records, and proved coverage for all 298 active documents; 17 archived snapshot records remain as history. Intentional unresolved evidence and committed future targets carry exact occurrence-level annotations rather than weakening an authoritative document with a whole-file historical label. Decisions taken by Mark 2026-08-24 and 2026-09-02 remain recorded in §Decisions." — accurate: yes
- claims checked: 13 — holds: 11, stale: 2, unverifiable: 0

### Stale claims

- Lines 16-21 say genet and turquet are "Not committed" and that "genet's `design_docs/` is therefore still untracked, which is a live risk". The Open risk section (lines 609-616) says the same. Both were in fact committed the same day: genet's design_docs (policy, index and the eight docs) in genet 944949f5c86 on 2026-08-24, now on genet HEAD; turquet's policy in turquet b553c69 on 2026-08-24.
- The plan's last word is that the three roots inker_docs/, nematic_docs/ and verso_docs/ live in genet: C3 Progress (lines 314-316), and the Finding at line 542 ("genet/design_docs/ (8 docs …)"). They came back to mere on 2026-09-03 together with crates/inker and crates/nematic, under platform boundary P3. At base the three roots hold 9 docs. DOC_POLICY.md records the return (:191-204) and says "Track the work in the [doc policy consolidation plan]" (:212-213), but this plan never records it.

### Contradictions

- The 2026-09-02 Finding (line 672) says the checker "lives outside the tree (scratchpad)". The plan's own 2026-09-04 Progress entry says it was restored as `scripts/mere_doc_audit.py`.
- DOC_README.md:76 says, in the present tense, that the aggregate "is now durable beside 34 supplemental identity records" and that the gate "proves all 298". At base, support/doc-audit/d2/ holds batches 15-37, and 535bca11 itself claims coverage of 337 of 337.

### Recommended action

- Close the header paragraph and the Open-risk section, citing genet 944949f5c86 and turquet b553c69. Add a dated note that the three roots returned on 2026-09-03. Refresh the DOC_README.md:76 counts. Then archive per §8 (see Forks).

### Notes

Ancestors of base: db9c613c, bcb222ce, e620e8f6, 724b613d, c5f01064. Present at base: `scripts/mere_doc_audit.py`; `scripts/mere_doc_judgment_audit.py`, whose SNAPSHOT_SHA256 at :19 matches the plan's digest; `support/doc-audit/d2/`. No `crates/*/design_docs` remains, and archive_docs/2026-09-02_retired_plans/ holds 17 files. Sixteen repos under repos/ track a DOC_POLICY.md with `## Local addendum` at line 124. Mesocosm and paredros are not under repos/, so the plan's count of fifteen cannot be re-checked as written. smolweb/design_docs holds both repatriated docs. No archive checkpoint exists after 2026-09-02.

## mere_docs/implementation_strategy/2026-08-25_browser_webrtc_carrier_plan.md

- disposition: current
- status line: "Status: in progress. C0-C2 landed 2026-08-26. C3 landed 2026-08-28: the forced relay is physically proven over a TURN relay on a second machine, so the stop line is CLEARED. C4 landed 2026-09-02 (C4a 09-01, C4b 09-02): the real Graphshell web client mounts a native-owned projection over WebRTC in both surfaces, with the browser carrier profile marked physically proven in the remote projection plan. C5, public rendezvous, is the next phase." — accurate: yes
- claims checked: 14 — holds: 12, stale: 2, unverifiable: 0

### Stale claims

- Lines 932-934 say "mere's web manifest still pins genet at `eff0cb6`, so a clean checkout renders boxes". At base, ports/graphshell/web/Cargo.toml:48-59 pins genet bd3e8861 (2026-10-02), which contains the font fix 893ccb9b3d9 and the size fix 577e2471e97.
- Lines 1520-1521 say "the DOC_README index line for this plan still reads 'C4 open'". DOC_README.md:172 now reads "C4 landed 2026-09-02 …; C5 next".

### Contradictions

- none.

### Recommended action

- Mark both lines resolved, with a date.

### Notes

Present at base: crates/murm/webrtc-carrier; `TransportKind::WebRtc` and `IngressContext::webrtc` (crates/murm/transport/src/accepted.rs:50,111); `admit_accepted_session` and its WebRTC fixture test (ports/graphshell/src/carrier.rs:168,413); `InviteV1` (webrtc-carrier/src/invite.rs:195); `ReleaseRefV1` (crates/system/luggage/src/release.rs:62); `SessionCore` and `SessionDriver`; `serve_webrtc_join`; `serve_admitted`; the `c4_webrtc_host` bin; `RemoteLink` (web_remote.rs:49); `<graphshell-view>` and `mountGraphshell` (web/loader.js:397-403); `SharedLiveEndpoint`. The remote projection plan marks the browser profile proven (:288, :609). All five receipts are in Code/testing/mere/. No C5 artifacts exist in mere or mer3ly.

## mere_docs/implementation_strategy/2026-08-31_terminology_and_crate_folds_plan.md

- disposition: current
- status line: "Status: landed on `main` (2026-09-02). The stack was rebased from its `9a53c77a` base onto `origin/main` `f2924f08` on 2026-09-02, reconciled with the Djinn Distillery lane that arrived upstream in between, and every recorded gate was rerun green before the push. The seven external consumer commits follow it in their own repositories." — accurate: yes
- claims checked: 17 — holds: 13, stale: 3, unverifiable: 1

### Stale claims

- Progress (lines 132-134) cites 962333d1, 0f160ff0 and d5d5f9b9. None is an ancestor of base; they are the pre-rebase hashes. The commits that landed are c51b9704, 275448cd and eae87153, which sit between f2924f08 and base.
- Progress (lines 135-136) cites Turnstone b079e3f and Retinue 3c6ff79. Neither is on any branch now. The landed versions on origin/main are turnstone 030ba08 and retinue 5cc4c2d (both 2026-08-31, "Migrate … to muniment journals"). Isometry 0b17fe9 is on origin/main.

### Contradictions

- The status line says "seven external consumer commits", but Progress names only four. I found no list of seven in this plan, its receipt doc, DOC_README or the landing commits. Which seven is unverifiable, but the count is plausible: the sibling repositories' origin/main carry more than four 2026-08-31 consumer commits (woodshed `b5bd3b6`, mora `0416081`, isometry `92f8a692` and `de04c9e2`, Mesocosm `fc12da3e` and `096f740f`, beside turnstone `030ba08`, retinue `5cc4c2d` and isometry `0b17fe9f`). Mesocosm `cff9b71` is stale: Mesocosm's history lives in isometry (`archive/mesocosm/*`), `cff9b71` is absent there, and its landed equivalents are `fc12da3e` and `096f740f`.

### Recommended action

- Cite the landed Mere, Turnstone and Retinue hashes. Reconcile "seven" with the four named commits. Then archive per §8 (see Forks).

### Notes

Ancestors of base: f2924f08, 9a53c77a, 77b3c3a2. Present at base: `eidetic::Codicil` (crates/eidetic/eidetic-core/src/codicil.rs:50); the Journal re-exports (muniment/src/lib.rs:56); TrainingCorpus v2 plus the v1 reader (eidetic-core/src/models/training.rs:24-61); `graphshell.graph-codicil/v2` and the legacy engram tag (ports/graphshell/src/product.rs:37,40); pandect's `consolidated_engrams` alias (crates/system/pandect/src/manifest.rs:139); `chartulary::rdf`; the numen, seiche and hagiograph crates; `conatus::resident`; gemot's `moot::tulpa`. No scholia, quint, quint-shaders or codicil package or dependency remains. The receipt doc exists.

## mere_docs/implementation_strategy/2026-09-02_autodiff_lora_trainer_plan.md

- disposition: current
- status line: "Status: complete on `main` (2026-09-03). Assessment complete; Mark ruled D1–D3 on 2026-09-02, each on the recommended option; every phase landed 2026-09-03 and the receipts were rerun on the rebased tree and on the Fedora ThinkPad. The padding-with-mask follow-on and the burn `LoraAdapter` question stay open in Findings. Follow-on to the [distillery v0 plan](2026-08-12_distillery_v0_plan.md) (§9 trainer forcing, and the 2026-09-02 discrete-GPU trainer entry) and the [FLORA, Tulpa, and Standing plan](../../moothold_docs/implementation_strategy/2026-08-31_flora_tulpa_standing_plan.md)." — accurate: yes
- claims checked: 11 — holds: 10, stale: 1, unverifiable: 0

### Stale claims

- The "Done" section (lines 240-243) says "The autodiff trainer is the default arm in Djinn's lane configuration examples". That does not match what was built. ports/djinn/examples/ holds only resident_v1_fixture.rs, and the two trainer arms appear only in ports/djinn/tests/common/mod.rs:359 and :378.

### Contradictions

- The same Done clause conflicts with the plan's own Phase 3 entry (lines 283-285): "The lane configuration names only the device; the arm arrives inside the posted request."

### Recommended action

- Rewrite the Done clause as built: the arm is chosen per request and the tests run both arms. Move the padding-with-mask follow-on and the LoraAdapter question into a live plan before archiving; no other active doc carries them.

### Notes

3ce750f5 and 129734d1 are ancestors of base. a30381a2 exists but is not on main, which matches the plan's "pushed branch". Present at base: esp's `decoder-autodiff` feature (crates/intel/esp/Cargo.toml:74); `train_peft_lora_autodiff` (train_autodiff.rs:270) and the v1 constants; distillery's `TrainerSettings` (ports/distillery/src/trainer.rs:108) and implementation id `…esp-trainer/v2` (:73); `trainer-autodiff` features in distillery and djinn; djinn tests distillery_trainer.rs and distillery_trainer_gpu.rs. The cross-notes are in the distillery v0 plan (:1137) and the FLORA plan (:142).

## mere_docs/implementation_strategy/2026-09-02_distillery_projection_walk_plan.md

- disposition: current
- status line: "Status: W0 through W2 complete. W1's endpoint, admitted catalog seam, Graphshell mount, session-local resume by diff, detailed frozen table, machine-readable headless receipt, admitted `ResidentProjectionHost` carrier path, readable headed WebRTC fixture receipt, live Djinn-owned resident route, stable-topology authority diff, and live headed WebRTC receipt are green. W2's authored two-source recipe, authority-generation shelfmark, single-option variant, and headed binding receipt are green through 2026-09-08. Continuation across a fresh admission is a separate protocol question because it receives a fresh transcript-derived projection session (Progress). §2 was read at mere `77a3701f052` and corrected at `3ce750f5` by the W0 implementation, which read the code rather than this plan. On 2026-09-09 the host-neutral definition, validation, binding, variant, and deterministic-JSON contract moved into the `scenograph` authoring crate; Graphshell retains its editor UI, compiler, persistence, and Chronicle recipe." — accurate: no
- claims checked: 14 — holds: 11, stale: 3, unverifiable: 0

### Stale claims

- The status line says "§2 was read at mere `77a3701f052`". No such object exists in mere. 77a3701f052 is a genet commit (2026-09-02, "docs: assess unifying the two layout builders").
- Lines 591-593 (and DOC_README.md:145) say Turnstone can adopt the revision split only once its Mere and Knot pins advance together, and that "its checkout remains unchanged until then". Turnstone adopted it in dc7d227 (2026-09-09, on origin/main): src/knot_authoring.rs:31 and :139-153 use `PublicSourceRevision` and `RuntimeSourceBinding`.
- The W1 section (line 191) still gives as its "Current next step" carrying Djinn's live route through the browser/WebRTC door. That was closed on 2026-09-07 (Progress line 496; `admit_webrtc_catalog` at ports/graphshell/src/native/projection_host.rs:352).

### Contradictions

- The W1 phase header (line 178) still reads "software acceptance implemented 2026-09-06", and W2 has no landed marker, while the status line says W0–W2 are complete.
- DOC_README.md:145 repeats the Turnstone wait.

### Recommended action

- Correct the 77a3701f052 citation. Mark W1 and W2 as landed in §4 and drop the stale next step. Record Turnstone dc7d227 here and in DOC_README.md:145. Name W3 and W4 as the open work.

### Notes

Present at base: `ChronicleEndpoint`, `ChronicleObserver` and the `distillery.chronicle/v1` adapter (ports/distillery/src/chronicle.rs:44,139,227); `ChronicleMount` (ports/graphshell/src/distillery_w1.rs:57); `ServedWebRtcProjection`; scenograph's `PublicSourceRevision`, `RuntimeSourceBinding` and `AuthoredProjectionDefinition` (crates/cambium/scenes/scenograph/src/lib.rs:33,286,339); djinn's route and `chronicle_projection_host` (resident_distillery.rs:77,453); the receipt-host bin; tests distillery_chronicle_route, distillery_chronicle_host and c4_host_signaling; the receipt JSON; ports/distillery/{alembic,athanor}; Knot's `public_revision`. 3ce750f5 and 487e18a4 are ancestors of base. 0efc9ce5 (2026-09-09) is titled "Complete Distillery Chronicle projection walk". No Circuit (W3) code exists.

## mere_docs/implementation_strategy/2026-09-02_physics_catalog_plan.md

- disposition: current
- status line: "Status: in progress (P1 landed 2026-09-02; P1b, P2 on both hosts and P3 the remote board 2026-09-03; the runtime extraction 2026-09-04; P4 web half 2026-09-04, closing with the Graphshell tree port per the 2026-10-01 rulings in §5; P5a-c 2026-10-02: kernel, cell list, lagged seam, setters and the web tree at the third-round web defaults, receipts green, merged; then turnstone and P5d; P7 moved to the [dynamics grammar plan](2026-10-02_dynamics_grammar_plan.md) 2026-10-02)." — accurate: no
- claims checked: 9 — holds: 8, stale: 1, unverifiable: 0

### Stale claims

- The status line stops at 2026-10-02 and leaves out P6a. Density's CPU tier is on main at base: crates/conatus/seiche/src/laws/density.rs:149 and `PhysicsLaw::Density` in pictograph's physics_catalog.rs, with nine P6a rounds in Progress (lines 1826-2124), landed on main via `9b576c84` ("Merge density-cpu", 2026-10-04) and `fb12ce11` (`52ba81ba` and `9724a6d5` merge main into grammar-g7 and energy-frame). It also leaves out the Orbit retune (9ce5889f) and the Energy-frame merge (562488b0), both 2026-10-04.

### Contradictions

- DOC_README.md:170 says "…runtime extraction landed; P4 next". The tree and this plan have P4's web half landed (2026-09-04) and P5a–c landed (2026-10-02).

### Recommended action

- Add to the status line: P6a landed, P6b and P6c open, plus the Orbit and Energy retunes. Update DOC_README.md:170.

### Notes

P5 commits 438187cb, 13910c40, c4097a1e and af7d2b5f are ancestors of base. 89b75bb4 and 3a82eca7 are bundle ids, not commits. Present at base: `Exclusion` (conatus/src/resident/exclusion.rs:76); `resident::binning`; `LaggedRepulsion` and `LaggedLane` (seiche/src/lagged.rs:36,89); `PhysicsDevice` (seiche/src/gpu.rs:31); `set_physics_device` on Canvas, PhysicsBoard and RemoteBoard; mere's `canvas-gpu` (crates/mere/Cargo.toml:41). PhysicsLaw has 12 variants. Turnstone has `SetPhysicsLaw` (src/action.rs:268) but no `set_physics_device`, so "then turnstone and P5d" still holds. P4 correctly stays open behind the one-tree plan's phase 4, which is still in progress.

## mere_docs/implementation_strategy/2026-09-02_platform_boundary_and_repository_topology_plan.md

- disposition: current
- status line: "Status: Complete 2026-09-06; P0-P7 landed. Woodshed's P4 exception closed 2026-09-04 with single-source, software, and headed receipts. P6's public topology, hosted Pages, domain verification, certificate issuance, and HTTPS enforcement closed 2026-09-05. P7 closed 2026-09-06 when the documentation-policy D3 audit reached zero failing structural, link, path, and annotation findings. The 15 actionable D2 recommendations were reconciled in the [2026-09-06 remediation pass](../../../support/doc-audit/d2/remediation_2026-09-06.md); the 11 unverifiable claims remain explicit evidence requests. Vello V1-V4 are owned separately by Netrender's durable upstream-ask note." — accurate: yes
- claims checked: 13 — holds: 12, stale: 1, unverifiable: 0

### Stale claims

- The games-wing section (2026-09-09, near the end) says "Genet is pinned to its isolated adoption commit `3a7b5023…`". That commit exists only on genet's origin/wing-platform-alignment-20260909. The sentence was true when written and is stale now: main pinned `3a7b5023` from `6e453de7` (2026-09-09, the commit after `c4ef60eb`) until `8131d7a3` (2026-09-12) advanced every pin to `ec5281ef7fc`; the base pins `bd3e8861` (Cargo.toml:303-338; web/Cargo.toml:48), which lacks it. The Netrender c77b0be8 alignment does hold: base pins 9607d16f, which contains it.

### Contradictions

- The tail of the 2026-09-05 P6 Progress entry (lines 2786-2790) says GitHub "does not yet permit HTTPS enforcement". The status line and the P6 Status section (lines 488-491) say HTTPS was closed that day, and mer3ly's receipt (docs/receipts/org-transfer/2026-09-05_p6_platform_topology.md:58-78) confirms `https_enforced=true`.
- Progress has no P7 entry; P7's close appears only in the status line.
- P7's done-condition is "this plan contains only genuinely open work before archival". Instead the plan sits active and complete, with a 2026-09-09 follow-up appended after completion.

### Recommended action

- Correct the genet pin sentence. Annotate the 2026-09-05 tail with the HTTPS follow-up. Add a P7 Progress entry. Then archive per P7 and §8 (see Forks).

### Notes

Mere ancestors of base: 725bbf1a, cb3fd887, 91bf62c9, 13b64e30, d82afa17, c4ef60eb. On genet origin/main: a93189b1d7c, 75d3900f82e, ce79fd44a4d, 6d8daca939b, 115d348d. On woodshed origin/main: c5eaa7d6. knot-editor fcd004b6 exists. Genet's components contain no Cambium crate; its Cargo.toml:62 mentions Cambium only in a comment. Present in mere: crates/inker/knot-editor-host, ports/pelt, ports/tabard. The scenograph absorption plan carries the 2026-09-04 supersession note. The Vello note exists in netrender. The remediation receipt cites 15 recommendations and 11 unverifiable claims.

## mere_docs/implementation_strategy/2026-09-11_recursive_query_experiments_plan.md

- disposition: current
- status line: "Status: E1–E3 merged to main 2026-09-12 (`794a96bd`, `7ba1df59`, `ef64c1cc`); ascent dev-dependency and bench removed after recording the numbers; Q1/Q2 landed 2026-09-12 (`RelationKind::OpenPredicate`, Traversal row on presence, `edges_between_undirected` pair scan); the `graphlets` → `subgraph` crate rename landed 2026-09-12 (see Progress)." — accurate: yes
- claims checked: 11 — holds: 10, stale: 1, unverifiable: 0

### Stale claims

- The Scope, done-conditions and Progress (line 311) name the `mere-subgraph` package and the `canvas` crate (`cargo test -p mere-subgraph`, `-p canvas`). At base, subgraph is the module `crates/mere/src/subgraph.rs` (folded by 61894570, 2026-09-23). Canvas is now `pictograph` (`collapse_descendants` at crates/canvas/pictograph/src/canvas/fold_projection.rs:139, folded by f590e45d, 2026-09-24). No banner records either fold.

### Contradictions

- none.

### Recommended action

- Add a dated rename note. Move E2 phase 2 (the relation delta journal) into a live plan. Then archive per §8 (see Forks).

### Notes

794a96bd, 7ba1df59 and ef64c1cc are ancestors of base. No ascent dependency or ascent_bench.rs remains. Present at base: `RelationKind::OpenPredicate` (edge_taxonomy.rs:383; query.rs:45); `edges_between_undirected` (edge_ops.rs:475); `outgoing_relations` and `incoming_relations` (query.rs:182,193); `force_reconcile_all` (crates/mere/src/subgraph.rs:112); roster's "Predicate" label (roster.rs:784); `.claude/` in .gitignore:121; the timestamp zeroing (fold_and_source_time.rs:251); all containment sub-kinds restored (snapshot/from.rs:352-363); `ArrangementKind::Supernode`.

## mere_docs/implementation_strategy/2026-09-23_reservoir_plan.md

- disposition: current
- status line: "Status: in progress. V1 is complete and on main: the pandect index, wallet-persona resolution, djinn's reservoir lane and route, and a real two-process receipt. It reached origin with `5364dfa0` on 2026-09-24. V2's shape was ruled on 2026-09-23 and 2026-09-24 (§7). Steps 1 to 3 (muniment, graph-kernel, pandect) landed on 2026-09-24 and reached origin on 2026-09-25; step 3b, undo with exact replay, landed on 2026-09-25 and reached origin the same day. Step 4, `MereHost` on `GraphSession` in Graphshell, landed and reached origin on 2026-09-25, and a browser receipt the same day shows it running in Chromium over IndexedDB (§7 item 29); the receipt's scenario verdicts await a headed run. Step 5, djinn's routes, landed on 2026-09-25 with its two-process receipt, meeting V2's done-conditions (§8), and reached origin the same day. V2b, the mere view, was assessed and ruled the same day (§7 items 34 to 40): Graphshell first moves onto one Cambium tree, in its own plan. V2b's steps 1 to 3, the component, its headed proof and the route adapter, landed and reached origin the same day. Step 4, Graphshell on one Cambium tree, is under way: its plan's phases 1 and 2, accessibility in the browser and the file seam, were done on 2026-09-26, and phase 3, the canvas as a producer, is next." — accurate: no
- claims checked: 8 — holds: 7, stale: 1, unverifiable: 0

### Stale claims

- The status line says "phase 3, the canvas as a producer, is next". The one-tree plan's own status at base (2026-09-25_graphshell_one_tree_plan.md:4-8) says phase 3 has headed correctness receipts and that Mark approved phase 4 on 2026-09-27. Its Progress records the phase-4 physics-panel lane on 2026-10-01 (:790).

### Contradictions

- none. The DOC_README entry is less specific and consistent. The step-4 browser receipt's "scenario verdicts await a headed run" holds at the base: `ports/graphshell/docs/2026-09-25_reservoir_step4_browser_session_receipt.md:77-78` still says so, unchanged since `5cb6e65b`, and plan :1180 records the wait.

### Recommended action

- Update step 4's position, or point to the one-tree plan's status rather than restating it. Record the headed verdicts, or keep them explicitly open.

### Notes

5364dfa0 is an ancestor of base and on origin/main; c19e1120 and 685c830e are ancestors. Present at base: crates/cambium/mere-view; ports/graphshell/src/{mere_host,mere_route,session_item}.rs; `GraphSession` (pandect/src/graph_session.rs:336); `MereHost` (mere_host.rs:153); ports/djinn/tests/mere_two_process.rs. Graphshell has only the opt-in `mere-route` feature and no panel in web_tree.rs, so V2b step 5 is open, as the plan implies. The cleromancy divination plan exists.

## mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md

- disposition: current
- status line: "Status (2026-10-03): G1 done on branch `grammar-g1`, ready to merge: the declarations, the instruments, F7's relabel and exponent, and F10's measurement and change, with the four questions G1 returned ruled as F14 to F17 (§1.1) and carried out. G2 starts once G1 is merged; G3 to G6 wait for the arrangement design (F11, F17). Written from the [dynamics grammar brief](../research/2026-10-02_dynamics_grammar_brief.md) and Mark's rulings of 2026-10-02." — accurate: no
- claims checked: 8 — holds: 4, stale: 3, unverifiable: 1

### Stale claims

- The status line says G1 is "ready to merge". It merged to main at 39787d82 on 2026-10-03, and seiche/src/terms.rs is present at base.
- The status line says "G2 starts once G1 is merged". G1 is merged, and G2 has been under way since 2026-10-03 on branch grammar-g2: first commit 6d112683, head 6b4039d0, 18 commits not in base. The plan on main has no G2 progress or status entry (G9's shared-files list, :496-497, names the grammar-g2 lane).
- Progress last calls G7 (line 446, "unmerged, at 270172db") and G9 (lines 466 and 576) unmerged. G7 merged at 2d4b1ee9 (2026-10-04) and G9 at c1cd69ff (2026-10-05). The status line names neither.

### Contradictions

- DOC_README.md:171 repeats "ready to merge … G2 is next". Unverifiable: "G3 to G6 wait for the arrangement design (F11, F17)". G7, the arrangement roles track (F18–F30), merged on 2026-10-04; whether that clears the gate needs a ruling.

### Recommended action

- Rewrite the status line: G1 merged (39787d82), G7 merged (2d4b1ee9), G9 merged (c1cd69ff), G2 in progress on grammar-g2, G8 open, G3–G6 per ruling. Fix DOC_README.md:171.

### Notes

All of these are ancestors of base: f1691793, 66058f61, 270172db, 4d602ad3, 52ba81ba, b9c849f5, 85525373, d8441788, 60176e38, 9fd45b01, 90fcad2f. Branches grammar-g1, -g7 and -g9 are merged; grammar-g2 is not. `sceno::Hold` has Seeded, Anchored and Pinned (sceno/src/score.rs:167-173). The physics plan's P7 closes with a pointer here (lines 1043-1050). All related links resolve.

## mere_docs/technical_architecture/2026-08-14_tactile_tier_plan.md

- disposition: current
- status line: "Status: founded 2026-08-14; T1-T3 landed the same day. The tactile half of the two-tier ruling (field system extraction doc, amendments of 2026-08-13): CPU rapier, the bodies a hand manipulates, the source of commitment events. Its doctrine is already ruled and this plan builds the vocabulary on it:" — accurate: yes
- claims checked: 6 — holds: 6, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- This is a plan filed under technical_architecture/, but DOC_POLICY §8 puts plans in `implementation_strategy/`. D1 flagged it on 2026-09-02 (doc policy plan, lines 628-629), and it is still unchanged.
- T5, "the mere profile" (a physics profile as data), overlaps the physics catalog's named profiles and the dynamics grammar plan's G4 `DynamicsSpec` saved in `SavedSceneV1`. Neither of those plans references this one.

### Recommended action

- Move it to implementation_strategy/ and repair the inbound link in DOC_README.md (:306, the only one), or archive it with T4 and T5 extracted (see Forks).

### Notes

Present at base: `NodeMaterial` with `gravity_scale`, and the `node_material` read-back (crates/conatus/seiche/src/node_body.rs:77-186); `Kinds(u16)` and the sieve in seiche/src/sift.rs:35; `supports_of` and `containments_of` (seiche/src/propose.rs:41,72); seiche/tests/tactile.rs. No piles or union-find query exists, so T4 is open. The last edit was 133989c1 (the 2026-09-05 audit).
