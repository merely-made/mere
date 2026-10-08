# Batch 43 — S14 pass, phase B5 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-08-08_family_shared_identity_plan.md | historical-marked | yes | 21 | 18 | 3 | 0 |
| mere_docs/implementation_strategy/2026-08-10_castellan_otp_plan.md | historical-marked | yes | 19 | 16 | 3 | 0 |
| mere_docs/implementation_strategy/2026-08-10_crypto_generation_unification_plan.md | historical-marked | no | 13 | 11 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-10_dramatis_tier_plan.md | historical-marked | yes | 12 | 10 | 1 | 1 |
| mere_docs/implementation_strategy/2026-08-10_receipt_artifacts_replication_plan.md | historical-marked | no | 15 | 10 | 4 | 1 |
| mere_docs/implementation_strategy/2026-08-10_wallet_carry_foldin_plan.md | historical-marked | yes | 11 | 7 | 4 | 0 |
| mere_docs/implementation_strategy/2026-08-12_distillery_v0_plan.md | current | yes | 15 | 13 | 1 | 1 |
| mere_docs/implementation_strategy/2026-08-13_graph_behaviors_plan.md | current | yes | 11 | 9 | 1 | 1 |
| mere_docs/implementation_strategy/2026-08-14_castellan_keeper_founding_plan.md | historical-marked | yes | 11 | 9 | 1 | 1 |
| mere_docs/implementation_strategy/2026-08-15_projection_grammar_adoption_plan.md | current | yes | 10 | 7 | 2 | 1 |
| mere_docs/implementation_strategy/2026-08-22_license_sweep_plan.md | current | no | 16 | 12 | 3 | 1 |
| mere_docs/implementation_strategy/2026-08-22_scenograph_absorption_plan.md | historical-marked | yes | 7 | 6 | 1 | 0 |
| **Totals** |  |  | **161** | **128** | **26** | **7** |

**Totals: 12 docs, 161 claims checked (128 holds, 26 stale, 7 unverifiable), 22 contradictions; 3 status lines wrong.**

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

Checked directly in this session: `ports/djinn/Cargo.toml` pins
`sha2 = "0.10"` at the base (crypto generation); castellan's manifest is at
version 0.0.3 (castellan OTP); `SCORE_VERSION` is 5 (scenograph absorption);
and merge `0a8198ba`'s two parents carry inker's and verso-tile's notice files
while the merge carries neither (license sweep). The verifier refuted nothing
and found 6 items true only in part (the receipts plan's intake line order and
the device host, moved rather than deleted; athanor's dependencies; the
second DOC_README wording; the header count, 61 not 60; LICENSES.md's tinct
row), plus inker's notice-file loss, which the draft had put down to a ruling;
those records are corrected.

## mere_docs/implementation_strategy/2026-08-08_family_shared_identity_plan.md

- disposition: historical-marked
- status line: "Status: shared-identity adoption landed for Graphshell, Turnstone, Knot, Woodshed, and Hocket; Hocket's return-to-own-identity UI remains intentionally unbuilt." — accurate: yes
- claims checked: 21 — holds: 18, stale: 3, unverifiable: 0

### Stale claims

- The plan calls the module `session_runtime::shared_root`. At the base it is `pandect::shared_root` (crates/system/pandect/src/shared_root.rs:29 `MERE_ROOT`, :41 `shared_root`, :67 `adopt_legacy_identity`). The crate was renamed in 441e70f0 (2026-08-15).
- The Per-application table's Turnstone row says Knot's persona vault and device key come from `shared_root()` on both the hosted and the spawned path, and that `HostedKnot::PersonaVault` lost its `data_root` field. Turnstone d6c4bdc (2026-08-22, "Route persona Knot authoring through the resident") removed that variant. `enum HostedKnot` now has only `Directory` (turnstone src/knot_authoring.rs:482-484), and persona-vault mode goes through Graphshell's resident (`KnotHub::resident`).
- The plan says "`TURNSTONE_KNOT_PERSONA` is now the override, not a requirement". Turnstone now refuses it in persona-vault mode: "Graphshell owner settings select the resident Knot persona; remove TURNSTONE_KNOT_PERSONA from Turnstone" (src/knot_authoring.rs:830-833).

### Contradictions

- DOC_README.md line 190 says "landed 2026-08-08 for Turnstone and Woodshed; Hocket blocked on purpose" and "Hocket is a key rotation, not a wiring change". It also names `session_runtime::shared_root`. The plan's status line and its 2026-08-09 Hocket sections say Hocket is wired and the rotation was avoided.

### Recommended action

- Date the status line, as DOC_POLICY §8 requires.
- Add a dated note that Turnstone's Knot persona route moved to the resident in d6c4bdc, and that the crate is now `pandect`.
- Rewrite the DOC_README entry to match the status line.

### Notes

Checked at 535bca11: `personae::roster` functions in crates/dramatis/personae/src/roster.rs (now 12 tests); vault.rs:436 `switch_profile`; crates/dramatis/persona-picker (`roster_items` at :81); muniment backend.rs:162 Box impl; castellan authority.rs (`PersonaeHost`, `snapshot` at :486, profile intents); ports/graphshell/src/profile.rs:47; the persona_switch_receipt bin. Siblings: turnstone src/identity.rs:138 `open_chosen` and session_lifecycle.rs:44; knot-editor knot_sync_host.rs:25; woodshed woodshed-core sealed_backend.rs:57 and woodshed-genet storage.rs; hocket-genet identity.rs (Hocket has lived at woodshed/ports/hocket since 2026-09-09). No Hocket "return" path exists.

## mere_docs/implementation_strategy/2026-08-10_castellan_otp_plan.md

- disposition: historical-marked
- status line: "Status: C1 and C2 complete 2026-08-20, hardened and extended 2026-08-21 (resident lock and freshness ledger, Linux Secret Service, Steam Guard). Library-complete. Djinn now claims per-profile `CastellanResident` record and freshness custody; code presentation, admitted approval, Secret Service policy, credential replication between persona devices, and CXF import remain follow-on slices." — accurate: yes
- claims checked: 19 — holds: 16, stale: 3, unverifiable: 0

### Stale claims

- The 2026-08-20 C2a entry says `OtpItemStore` writes sealed records under `castellan/otp/v1/<persona>/<item>.json`. At the base, records live under `castellan/items/v1` (ports/castellan/src/items/records.rs:18) as chatelaine items (otp/item.rs:7-24). The chatelaine plan's P2 (3e4992ec) removed the `castellan/otp/v1` formats.
- "Where it lives" says chatelaine is "an empty reservation". crates/dramatis/chatelaine is now a real taxonomy crate (item.rs, otp.rs, disposition.rs), landed by chatelaine P1 to P3 (da3c50bc, 3e4992ec, ff68e86c). Castellan imports `OtpCodeStyle` and `OtpAlgorithm` from it.
- The 2026-08-21 entry says "The crate is still 0.0.2; no release followed C2". ports/castellan/Cargo.toml:3 is `version = "0.0.3"`, bumped in 5ca0d3a3 (2026-08-23).

### Contradictions

- DOC_README.md line 113 lists "hosting `CastellanResident` in a product (graphshell enables only `keeper`)" as open. The plan's 2026-08-22 entry and status line record that Djinn holds `CastellanResident` custody (ports/djinn/src/resident.rs:17, :40, :375).
- The plan does not point to the 2026-10-01 chatelaine CXF plan, which now owns OTP storage and CXF import.

### Recommended action

- Annotate C2a and "Where it lives" with dated pointers to chatelaine plan P2 and P3.
- Correct the version line.
- Update the DOC_README entry to name Djinn custody.

### Notes

Checked at 535bca11: the files under ports/castellan/src/otp; the struct definitions (`OtpItemStore` item.rs:126, `OtpReleaseGate` release.rs:226, `OtpCodeTile` tile.rs:28, `OtpAdmittedSession` admitted.rs:148, `CastellanResident` resident.rs:26, `serve` secret_service/dbus/mod.rs:102); the RFC vector constants in otp/tests.rs. The largest otp file is 584 lines. graphshell and djinn both use only castellan's `keeper` feature. Nothing outside castellan calls `serve` or uses `OtpCodeTile`.

## mere_docs/implementation_strategy/2026-08-10_crypto_generation_unification_plan.md

- disposition: historical-marked
- status line: "Status: DONE 2026-08-10 for every first-party manifest in mere" — accurate: no
- claims checked: 13 — holds: 11, stale: 2, unverifiable: 0

### Stale claims

- The status line says every first-party manifest is on the 0.11 row. At the base, three first-party manifests pin `sha2 = "0.10"`:
  - ports/djinn/Cargo.toml:78, added in 37706afb on 2026-09-13
  - ports/pelt/desktop/Cargo.toml:186, commented "carries genet's 0.10 line"
  - ports/distillery/probe/session-fixture/Cargo.toml:21, added in aa121f03 on 2026-08-24
- The Residue section says "Every remaining `digest` 0.10 consumer is third-party transitive". The base Cargo.lock lists `djinn` 0.0.2 and `pelt-desktop` 0.2.0, both first-party, among sha2 0.10.9's dependents.

### Contradictions

- The doc policy consolidation plan (2026-08-24_doc_policy_consolidation_plan.md:414-415) records the session-fixture pin as "one code breach … against the ruled 0.11 row". This plan still says DONE.
- DOC_README.md line 103 repeats the "every remaining 0.10 consumer is third-party transitive" residue claim.

### Recommended action

- Add a dated note naming the three pins.
- Either reopen the plan to repin djinn and session-fixture, or record pelt's genet-line pin as an accepted exception under the crypto stack decision.
- Correct the DOC_README entry.

### Notes

The root workspace pins hold (Cargo.toml:583-584). Also checked: chacha20poly1305 0.11 in personae, eidetic-core and pandect; castellan hmac 0.13 and sha1 0.11; `KeyInit` at castellan otp/mod.rs:59; the pin test at personae seal.rs:126. The base lock still has misfin 0.0.4, ed25519-dalek 2.2.0 and 3.0.0, and argon2 0.5.3. "session-runtime" is now pandect.

## mere_docs/implementation_strategy/2026-08-10_dramatis_tier_plan.md

- disposition: historical-marked
- status line: "Status: ratified by Mark 2026-08-10; D1-D3 complete 2026-08-10; D4's wallet fold-in (2026-08-10) and credential port (castellan C1-C2, 2026-08-21) done; the `dramatis` facade was ruled real on 2026-10-01 (for repos outside mere) and is unbuilt" — accurate: yes
- claims checked: 12 — holds: 10, stale: 1, unverifiable: 1

### Stale claims

- The 2026-10-01 Progress entry is the plan's last word on chatelaine, and it says the rulings are "all unbuilt" and "chatelaine becomes a plain taxonomy crate". Chatelaine has since been built: P1 da3c50bc, P2 3e4992ec, P3 ff68e86c and P4a 007fbe7c are all ancestors of 535bca11.

### Contradictions

- DOC_README.md line 116 says "Only the empty `dramatis` facade reservation remains". It leaves out the plan's 2026-10-01 ruling (tier architecture ruling 8) that the facade is real.
- The status line gives castellan C1-C2 as done 2026-08-21. The OTP plan's status says complete 2026-08-20, hardened 2026-08-21. This is minor.

### Recommended action

- Append a dated note that chatelaine P1 to P4a landed (pointing to the chatelaine CXF plan).
- Give the facade item a plan home, then consider archiving.

### Notes

Checked at 535bca11: the crates/dramatis members (Cargo.toml:35-53); the gaz manifest's repository field; crates/dramatis/dramatis 0.0.2 with a doc-only lib.rs; ports/gazette; personae carry/refs.rs `CarryRef`; pandect codicil_seal.rs:52 `WalletEpochSealer`; the castellan `keeper` feature; tier architecture ruling 8 (:316, :396). Unverifiable: whether chatelaine 0.0.2 was republished (the manifest is at 0.0.2; crates.io not reachable offline). Post-base commits 3a80b1fe and dbad27d5 open a separate "dramatis repo plan"; they are outside this audit.

## mere_docs/implementation_strategy/2026-08-10_receipt_artifacts_replication_plan.md

- disposition: historical-marked
- status line: "Status: R0-R3 built 2026-08-10. R3 landed in the projection rather than turnstone, for the reason in §5; one gap named there. See §5." — accurate: no
- claims checked: 15 — holds: 10, stale: 4, unverifiable: 1

### Stale claims

- The status line says R3 landed in the projection rather than Turnstone, with one gap. The body's §6.1 to §6.4 record the first-party app door and the Turnstone lens landing between 2026-08-14 and 2026-08-16. Turnstone has src/device_receipts_pane.rs, src/device_receipts_service.rs, the `turnstone.device-receipts` pane (src/panes/registry.rs:31) and scenarios/device_receipts.scn. The named gap (a store-backed `bytes_for`) closed 2026-08-14: identity_endpoint.rs:47 `ResourceReader`, :53 64 MiB cache.
- The plan says `device_sync` hosts `spawn_receipt_intake` and `stage_captures` in graphshell. They now live in djinn: `stage_captures` at ports/djinn/src/personal_sync.rs:1064 and `spawn_receipt_intake` at :1115.
- §6.2 says "`graphshell_device_host` serves both doors … `--app-endpoint`". That binary moved (a git rename) into ports/djinn/src/bin/djinn.rs in 1a3dcf6f (2026-08-22). Djinn now serves the app door (ports/djinn/src/bin/djinn.rs:1009 `serve_app_broker`).
- §6.5's last word is that the Device Receipts pane "has no scroll container … a real gap". Turnstone 8382e75 (2026-08-16) added scrolling (`PaneScroll` at device_receipts_pane.rs:120), with scenarios/device_receipts_scroll.scn.

### Contradictions

- Inside the doc, the status line's "one gap" contradicts §5's closed read-through and §6's landed lens.
- DOC_README.md line 364 calls the plan "planned, not executed".

### Recommended action

- Rewrite the status line: R0-R3, the app door and the Turnstone lens landed; intake and doors moved to djinn.
- Either receipt R1's second-device condition or name it open.
- Fix the DOC_README entry.

### Notes

Checked at 535bca11: ports/graphshell/src/receipts/{ingest,manifest,intake,mod,card}.rs (`ingesting_twice_is_a_no_op` ingest.rs:320); bin/receipt_ingest.rs; native/{app_admission,app_broker,app_client,local_endpoint,local_session}.rs (`AllowedApps` is now `AllowedAppRoutes`); `LocalLink` at browser_carrier.rs:285; the tests at app_broker_tests.rs:82 and :372. Also genet scripts/remote-receipt.ps1 (`-Ingest*` and `-CargoProfile` parameters). Unverifiable: R1's "browsable on a second paired device" condition. No receipt in the plan shows it; §6.5 shows remote-run receipts ingested on one machine.

## mere_docs/implementation_strategy/2026-08-10_wallet_carry_foldin_plan.md

- disposition: historical-marked
- status line: "Status: COMPLETE 2026-08-10 (W0-W4). One question spun out; see W3." — accurate: yes
- claims checked: 11 — holds: 7, stale: 4, unverifiable: 0

### Stale claims

- W3 says the fixture `signed_device_grant_wire_format_is_pinned` asserts 612 bytes. The test no longer exists; 03552f46 (2026-08-14) removed it.
- W3 says `wallet_grant` split into fourteen modules, including `envelope`. pandect/src/wallet_grant at the base has no `envelope` module. It now has certificate.rs, trust.rs, migrate.rs and revocation.rs (the post-migration shape).
- The plan names `session-runtime::{wallet_store, wallet_grant}` and `engram_seal::WalletEpochSealer`. These are now pandect modules, with the sealer at crates/system/pandect/src/codicil_seal.rs:52 (crate renamed in 441e70f0).
- "Noted in passing" lists `athanor.rs` (840), `graph_engram.rs` (817) and `manifest_store.rs` (655) as over the ceiling. Athanor moved to ports/distillery/athanor in 1bda73d5. graph_engram is now graph_codicil.rs (838 lines). manifest_store.rs is 658 lines.

### Contradictions

- DOC_README.md line 114 presents the device-grant-as-`SignedDelegationCertificate` question as spun out and needing a brief. The plan records it answered 2026-08-11 and executed 2026-08-12.

### Recommended action

- Add one dated note covering the pandect rename, the certificate-shaped wallet_grant and the removed fixture.
- Archive the plan.
- Correct the DOC_README entry.

### Notes

Checked at 535bca11: personae carry/ (mod.rs, refs.rs, plus grant/ and scope.rs); `WALLET_SCHEMA_VERSION`, `persona_wallet_salt` and `derive_persona_chain_root` in carry/mod.rs; the `carry_ref_repr_matches_eidetic_hash_repr` test at pandect wallet_store/mod.rs:145; ciborium used directly (pandect Cargo.toml:34, wallet_grant/mod.rs:47); the wallet_store module tree. knot-editor still consumes the pandect adapter (startup.rs, knot_sync_host.rs).

## mere_docs/implementation_strategy/2026-08-12_distillery_v0_plan.md

- disposition: current
- status line: "Status: D0 complete; D1 resident authority lifecycle complete. Its installed Personae/settings binding, configure/inspect binary, and read-only Cambium surface pass an exact-source focused Cargo gate, and Turnstone admits the surface as the contribution seam's second provider; operational host composition is ruled, built, and receipted in §10 — the Djinn lane runs a real mesh job from stated policy on `SystemClock` and closes clean, and as of 2026-09-02 reads physical memory on Windows, Linux and macOS alike, so its receipts are no longer Windows-gated — and the full workspace gate is closed green, leaving only the deferrals §10 records. D2's configured browser embedding matrix, first exact decoder row, lease-bound remote MiniLM row, and native ModelSession/PEFT LoRA row complete. Cooperative cancellation, explicit browser device teardown, fresh-worker recovery, exact remote reclaim ordering, CubeCL allocator cleanup, and real adapter numerical parity are proven. Driver-level release is proven for the supported plain remote profile. The immutable TrainingCorpus and EvalReport artifact contract carries the first local trainer receipt, and the trainer resource that receipt decided now runs as a real mesh job: `esp.train.peft-lora/v1` trains from a stored corpus's training partition, publishes the adapter blobs, manifest, and evaluation report into the composed Eidetic store, and commits an integer-only receipt in which the adapter strictly beats the unchanged baseline. Browser-level physical GPU allocation release remains unobservable through current browser APIs rather than an actionable implementation gate." — accurate: yes
- claims checked: 15 — holds: 13, stale: 1, unverifiable: 1

### Stale claims

- "Components founded, 2026-09-02" says Alembic and Athanor are "reservation stubs with no dependencies". At the base, ports/distillery/athanor/Cargo.toml depends on alembic, eidetic, kernel and pandect (euclid only as a dev-dependency), and alembic has src/memory_levels.rs (1bda73d5, 2026-09-23, "Move Athanor's passes and the memory levels into their named crates").

### Contradictions

- DOC_README.md line 144 says "Turnstone registration, operational host policy composition, the deterministic trainer fixture … and full workspace gate remain open", and calls the components "reservation stubs". The plan records all of those as closed (Turnstone 9d3a7d8; §10 Action, 2026-09-01; trainer_forcing.rs; the full workspace gate, 2026-09-01).
- The status line has no date of its own.

### Recommended action

- Add a dated note on Athanor and Alembic content.
- Date the status line.
- Rewrite the DOC_README entry.
- Decide whether the §8 and §10 deferrals go to another plan so this one can archive.

### Notes

Files present at 535bca11: ports/distillery/src/{authority,trainer,installed,surface}.rs and bin/distillery-installed.rs; crates/intel/esp/tests/trainer_forcing.rs; ports/djinn/src/{conditions,resident_distillery}.rs; ports/djinn/tests/distillery_{lane,trainer,trainer_gpu}.rs, with the `cfg(windows)` gates removed.

Symbols checked: `TRAINER_RESOURCE` v1 and the v2 implementation id (trainer.rs:56, :73); `DISTILLERY_MESH_SALT` (installed.rs:44); `mesh_lending`, `allowed_resources` and `accepted_checkpoints` in pandect device_settings_store.rs; `Host::facts` (now at ports/distillery/src/mesh_host/host.rs:197); `gpu_probe`; djinn's `trainer-gpu` and `trainer-autodiff` features.

Turnstone 9d3a7d8 is on HEAD. Unverifiable: the 2026-09-01 green workspace gate (needs cargo).

## mere_docs/implementation_strategy/2026-08-13_graph_behaviors_plan.md

- disposition: current
- status line: "Status: W0 through W5 landed 2026-08-13, with a green headed receipt (`turnstone scenarios/behaviors_wake.scn`, captures under `Code/testing/turnstone/behaviors_wake`). Two follow-ups remain: the review row clips in the palette, and the clock, app-tier and budget slices have no headed receipt of their own. The cascade-frequency gap named 2026-08-16 closed at the actuation boundary on 2026-08-18. Originally open. Designed with Mark 2026-08-13 (the "neat lil ideas" conversation: one node triggering others nearby to refresh, summaries from connected nodes captured into a knot note, and the family of automations behind them)." — accurate: yes
- claims checked: 11 — holds: 9, stale: 1, unverifiable: 1

### Stale claims

- The "Redesign status, 2026-09-20" line, §8.6, §8.7 and the 2026-09-20 Progress entry say Cargo validation and the portable consumer pin are pending and the new tests "have not run under the Cargo hold". Both have since happened:
  - Mere 0e031fa5 (2026-09-22, an ancestor of 535bca11) committed servitor resident.rs and run.rs, "Verified on the tracked portable lock, Rust 1.98.1 … 1,029 tests pass".
  - Turnstone 5d2c910 swept in R1a/R1b and repinned mere to 0e031fa5 portably (cargo test 586 passed, 1 failed). f3f8e3d fixed that failure, and 21 resident tests pass.
  - Turnstone now pins mere bd5912fb, which contains 0e031fa5 and is an ancestor of the base.

### Contradictions

- Inside the doc, §8 opens with "Status: proposed design" while §8.6 and §8.7 describe implemented R1a and R1b sources. The W1b and W2 headings still read "MOSTLY LANDED" against a status line of "landed".
- DOC_README.md line 194 carries the stale "await executable validation and a portable dependency pin", and the agent_harness_brief entry at line 327 says the same in other words ("executable receipts and the portable consumer pin remain pending").

### Recommended action

- Update the Redesign status and §8.6 and §8.7 with the 0e031fa5, 5d2c910 and f3f8e3d receipts.
- Mark §8's own status as implemented through R1b.
- Update both DOC_README entries.

### Notes

Checked at 535bca11: crates/servitor/src/{watch,cascade,tick,deadband,resident,run}.rs; `Gate::petition_behavior` (gate.rs:315); TERMINOLOGY.md:160 `watch`; `cascade_budget` (pandect application_settings_store.rs:87); `derive_containment_for` used at pictograph canvas/input.rs:742. No `ProcedureGraph` types exist, so R2 is unbuilt. Turnstone has src/behaviors.rs, src/resident_admission.rs, src/resident_runs.rs and scenarios/behaviors_wake.scn, and no clock, app or budget scenario. Unverifiable: whether the review-row clipping was fixed (needs a headed run).

## mere_docs/implementation_strategy/2026-08-14_castellan_keeper_founding_plan.md

- disposition: historical-marked
- status line: "Status: complete: the keeper surface moved, its feature matrix and direct-consumer receipt passed, and the subsequent Titulus/Chirograph and wire-string corrections landed." — accurate: yes
- claims checked: 11 — holds: 9, stale: 1, unverifiable: 1

### Stale claims

- "Ruled 2026-08-15: both … The card vocabulary became titulus" is the plan's last word on titulus. Titulus was folded back into chirograph in 83feb122 (2026-09-23, recorded in 2026-09-23_crate_consolidation_plan.md). At the base there is no titulus crate, only crates/chirograph/src/titulus.rs. Castellan imports the cards from chirograph (ports/castellan/src/projection.rs:21), and chirograph is at 0.0.2.

### Contradictions

- Inside the doc, Decisions says the wire strings were "renamed the same day (2026-08-14)" while Follow-ups says "Done 2026-08-15 (`617ea210`)". The commit is dated 2026-08-15.
- DOC_README.md line 189 says "renaming the wire vocabulary is an open item". The plan records it done (617ea210).

### Recommended action

- Add a dated note on the titulus fold-back.
- Correct the DOC_README entry.
- Archive the plan.

### Notes

Checked at 535bca11: castellan view.rs, projection.rs and authority.rs behind the `keeper` feature; graphshell shims at src/identity.rs, src/identity_projection.rs and src/native/personae_host.rs; identity_endpoint.rs, native/identity_ui.rs and profile.rs kept in graphshell. `graphshell.identity.*` remains only in ports/graphshell/docs/receipts. persona_switch_receipt.rs:41 imports castellan; 617ea210 is an ancestor. Unverifiable: the crates.io publish of castellan 0.0.2 with `keeper`.

## mere_docs/implementation_strategy/2026-08-15_projection_grammar_adoption_plan.md

- disposition: current
- status line: "Status: active: the executable Graphshell authoring proof landed 2026-09-04; A5 and the remaining portable-grammar questions stay open." — accurate: yes
- claims checked: 10 — holds: 7, stale: 2, unverifiable: 1

### Stale claims

- The 2026-10-05 "one arrangement catalog" note says Graphshell maps the families by hand in `arrangement_for` and `placement_for` at ports/graphshell/src/projection_compile.rs lines 666-691. At the base the file is 432 lines with no such functions, and it re-exports `scenomise::projection::*`. The compiler moved in c79bb8c2.
- The 2026-10-05 "P1 under way" note says "Built on branch `stack-seams-p1`; nothing merges before Mark's review." P1 landed on main at 1633be0c (recorded in add54925). Both are ancestors of 535bca11, and the compiler now takes host `ItemSizes` (projection_compile.rs `ProjectionCompiler::new(ItemSizes { card })`).

### Contradictions

- Inside the doc there are three status lines: the top one (2026-09-04), the 2026-10-05 section's own, and "Status (reconciled 2026-09-01)" at line 330 under a displaced "Date: 2026-08-15" header. The top line does not reflect the 2026-10-05 continuation.
- The S1 note at lines 191-198 says Graphshell maps by hand. Line 11 says the compiler has moved to scenomise.
- DOC_README.md line 164 describes only the 2026-09-01 reconciliation.

### Recommended action

- Fold the 2026-10-05 continuation and P1's landing into the top status line.
- Mark the S1 hand-mapping note as superseded by c79bb8c2 and 1633be0c.
- Refresh the DOC_README entry.

### Notes

Ancestors of 535bca11: bd119a69d, c79bb8c2, 5011e2f9, 302bbe72. Files present: crates/cambium/scenes/scenomise/src/projection.rs; ports/graphshell/web/fixtures/woodshed-stage.json; projection_authoring.scn and projection_reopen.scn; docs/receipts/projection_authoring_receipt.json; scenograph 0.0.4. Turnstone 648bf19 and Retinue 8cea8f9 exist. Knot e4cf739c and 9d0b955f are on knot-editor origin/main (local main is 9 behind). Unverifiable: Woodshed c55461cc, dad3f624 and c92e7c96 are absent from the local woodshed clone, and there was no network.

## mere_docs/implementation_strategy/2026-08-22_license_sweep_plan.md

- disposition: current
- status line: "Status: P0 and P1 landed 2026-08-27; P2 (genet), P4 turnstone and hocket, P5 mora and wavicle, and P7 mesocosm, paredros, netrender, wgpu-graft, wgpu-scry and wgpu-weld landed 2026-09-03; P6 clause and layout landed 2026-09-03; P3 isometry, P4 woodshed and P7 retinue wait on their lanes' dirty trees; P5 gaz has no repository to sweep (see 2026-09-03); one P2 hazard open in genet (see 2026-09-03). mere is MPL-2.0 by default with correct provenance — receipt in §6's Progress. Both P0 confirmations were settled 2026-08-22 (header shape C, the notice `Mark Alan Boykin`, no exceptions); P0's tooling and ledger were built 2026-08-27, and its two open verifications are answered there. Two rulings were taken during the work: `crates/system/luggage` carries MPL-2.0 with the Tauri/CrabNebula notices retained, and published crates ship no license text file (root `LICENSE` only), which struck one P1 done-condition. The remaining gate for each later phase is unchanged: a clean tree in that repository." — accurate: no
- claims checked: 16 — holds: 12, stale: 3, unverifiable: 1

### Stale claims

- The P1 done-conditions and the status line's "mere is MPL-2.0 by default with correct provenance" (zero unheaded owned sources) no longer hold. `git grep -L 'Mozilla Public' 535bca11 -- '*.rs'` finds 61 sources outside the ledger paths without Exhibit A (most carry a short `Copyright` + `SPDX MPL-2.0` header, which the tool's audit does not accept): 54 under crates/ and ports/, for example ports/graphshell/src/web_tree/*.rs, crates/system/registry/src/diagnostics/*, crates/intel/reference-data/src/*, crates/cambium/mesquite/src/*, and 7 probe sources under design_docs/mere_docs/testing/receipts. Files added since P1 lack the header.
- The P6 2026-09-03 note says "`tinct` and `verso-tile` still carry `LICENSE-MIT`/`LICENSE-APACHE` … Open: whether the two notice-file pairs are removed". tinct's pair was deleted by Mark's ruling in 642ca2d7 (2026-09-24). inker, which absorbed verso-tile, carries no notice files at the base, but not by a ruling: merge 0a8198ba (2026-09-06) dropped them (its first parent has `crates/inker/inker/LICENSE-*`, its second `crates/inker/verso-tile/LICENSE-*`, the merge neither), while LICENSES.md says those files are "not to be deleted".
- "One P2 hazard open in genet" (cambium's `PointerButton` re-export without pointer.rs) is overtaken. Genet HEAD 90c5ef507db no longer has components/cambium, only support/name-claims/cambium. In mere, crates/cambium/cambium/src/pointer.rs exists and lib.rs:162 re-exports `PointerButton`.

### Contradictions

- DOC_README.md line 165 says "planned 2026-08-22, confirmations settled, P0 tooling not started". The plan records P0 to P2 and P4 to P7 landed.
- mere's LICENSES.md says inker and tinct carry in-tree `LICENSE-MIT` and `LICENSE-APACHE` (rows at :84 and :89) and that the four files are "not to be deleted" (:93-94). All four are absent at the base (that file is outside this plan, but the plan points at it).

### Recommended action

- Correct the status line: mark the genet hazard moot, and record the 60-file header drift with a re-run of `scripts/relicense_headers.py` on a clean tree.
- Close the P6 notice-file question for tinct with 642ca2d7; inker's loss in merge 0a8198ba needs Mark's call (restore or accept).
- Fix the DOC_README entry and LICENSES.md's inker and tinct rows and its "not to be deleted" sentence.

### Notes

Checked at 535bca11: the root `LICENSE` and `LICENSES.md`, the tool scripts/relicense_headers.py, and workspace `license = "MPL-2.0"` (Cargo.toml:172). The only notice files outside support/patches are ledgered: conatus binning, meristem and mora-cmudict. Sibling commits on HEAD: genet 957926e4e8a; mora 741e18f; hocket 9a0fe17; turnstone ed46e0c; wavicle 14be097; the wgpu-graft, -scry and -weld commits; netrender f8c3485 and 5e8b2b9. mesocosm and paredros commits are in Code/archive/wing-consolidation-20260909, now isometry/mesocosm, still with shape-C headers and the retired-clause note at LICENSES.md:17-20. The waits hold: isometry root is `MIT OR Apache-2.0`; 154 of woodshed's 188 .rs files lack headers; retinue is unheaded. Unverifiable: that merely-made/gaz is absent on GitHub.

## mere_docs/implementation_strategy/2026-08-22_scenograph_absorption_plan.md

- disposition: historical-marked
- status line: "Status: complete — S1–S7 landed; `arrangements` is deleted and `mer3ly` names one engine. Committed and pushed as mere `cc40c24f`, with `mer3ly` `54fd8e2` repinned to mere `330eee98`." — accurate: yes
- claims checked: 7 — holds: 6, stale: 1, unverifiable: 0

### Stale claims

- S7 says "`SCORE_VERSION` 4 still applies to them". At the base it is `SCORE_VERSION: u16 = 5` (crates/cambium/scenes/sceno/src/score.rs:42), bumped in 4d602ad3 (2026-10-04).

### Contradictions

- Inside the doc, S7 says mer3ly's remaining mere deps still name rev 546991d and must be repinned. The status line records the repin to 330eee98, and mer3ly now pins d82afa17.

### Recommended action

- Add a dated note on Score 5.
- Archive as a complete historical receipt, as the header already describes it.

### Notes

cc40c24f and 330eee98 are ancestors of the base, and crates/canvas/arrangements is gone. Also checked: `Arrangement::Custom` (score.rs:264); the tests `v3_score_loads_with_disclosure_fields_absent` (score.rs:813), `a_hold_outranks_the_arrangement_in_every_family` and `every_family_places_every_item` (scenomise solve.rs:375, :416); scenomise registry.rs; scenograph is now the authoring crate. mer3ly 54fd8e2 exists, and repo-graph has no `arrangements` dependency.
