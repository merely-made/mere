# Batch 50 — S14 pass, phase B12 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-08-09_burn_0_22_migration_plan.md | current | no | 19 | 15 | 3 | 1 |
| mere_docs/implementation_strategy/2026-08-09_mesh_host_lanes_plan.md | historical-marked | no | 13 | 10 | 2 | 1 |
| mere_docs/implementation_strategy/2026-08-12_eidetic_reorg_plan.md | historical-unmarked | no | 12 | 7 | 4 | 1 |
| mere_docs/implementation_strategy/2026-08-12_search_surface_wiring_plan.md | current | no | 14 | 11 | 3 | 0 |
| mere_docs/implementation_strategy/2026-08-20_device_resident_consolidation_plan.md | current | yes | 11 | 9 | 1 | 1 |
| mere_docs/implementation_strategy/2026-08-22_djinn_family_resident_services_plan.md | current | no | 12 | 10 | 2 | 0 |
| mere_docs/implementation_strategy/2026-08-28_derived_faces_plan.md | historical-marked | yes | 11 | 9 | 1 | 1 |
| mere_docs/implementation_strategy/2026-10-05_djinn_test_harness_plan.md | current | yes | 12 | 11 | 0 | 1 |
| mere_docs/technical_architecture/2026-07-08_generic_graph_substrate_plan.md | current | no | 10 | 5 | 5 | 0 |
| mere_docs/technical_architecture/2026-08-13_spatial_compute_plan.md | historical-marked | no | 12 | 7 | 5 | 0 |
| moothold_docs/implementation_strategy/2026-06-30_bounty_verification_economy_plan.md | current | yes | 5 | 4 | 1 | 0 |
| nematic_docs/implementation_strategy/2026-06-13_polyglot_block_resolver_plan.md | current | yes | 8 | 6 | 2 | 0 |
| **Totals** |  |  | **139** | **104** | **29** | **6** |

**Totals: 12 docs, 139 claims checked (104 holds, 29 stale, 6 unverifiable), 17 contradictions; 7 status lines wrong.**

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

Checked directly in this session: S16's merge `cec0b3a4` is an ancestor of the
base (burn migration). The verifier refuted nothing and found 7 items true
only in part (the burn plan's pin count, its post-S16 record, and S0-8, which
§13.13 did decide; the mesh plan's wgpu row, older than S16; Turnstone's esp,
transitive rather than absent; the spatial compute plan's commit attributions,
where the body already records the CubeCL move; the djinn plan's
`ReleaseRefV1` origin); those records are corrected.

## mere_docs/implementation_strategy/2026-08-09_burn_0_22_migration_plan.md

- disposition: current
- status line: "Status annotation (2026-09-27, ruling 378): needed crates.io downloads are authorized with versions/integrity recorded and Git pins preserved. The original offline failure in §13.14 remains evidence; §13.15 records resumption. The bounded guard checkpoint is in progress, with root/product manifests still on pre.2 and broader migration acceptance still pending." — accurate: no
- claims checked: 19 — holds: 15, stale: 3, unverifiable: 1

### Stale claims

- The first Status line (line 141) says root and product manifests are still on pre.2. At the base:
  - the root lock holds burn and burn-remote 0.22.0-pre.4 and cubecl-runtime 0.11.0-pre.4;
  - 33 manifest rows pin `=0.22.0-pre.4`, and two vendored patch manifests declare package version 0.22.0-pre.4;
  - no non-comment manifest row names pre.2.
  All five "Status"-labelled paragraphs (lines 141, 147, 156, 160, 228) describe 2026-09-27 or earlier. The plan's newest state sits in the unlabelled dated block above them (lines 3-139).
- That top block says "S16 is the coordinator's" (line 11), and its 2026-10-04 bullets say "S16 has not started" (lines 36, 53). S16 ran: cec0b3a4 ("Merge burn-pre4-repin: Burn/CubeCL pre.4 (S16, ruling 557)") and 07db35e2 are both ancestors of 535bca11. The plan records this in a 2026-10-05 annotation inside §13.44 (lines 4365-4370).
- S0-8 (lines 1707-1712) defers the parked pre.3 lane: "decide at S16". §13.13 (2026-09-27, lines 1816-1817) accepted "preserve the parked pre.3 branch, worktree and uncommitted work", but the revisit at S16 that line 1712 anticipated is not recorded. Branch `burn-pre3-repin` (276608d5, e1c0cb44, 31102555, 610a32c5, none merged) and worktree `Code/worktrees/mere-burn-pre3` still exist.

### Contradictions

- Inside the plan:
  - line 11 says "S16 is the coordinator's", and line 4729 says "S16 remains the coordinator's";
  - the §13.44 annotation at line 4365 says "S16 ran".
- DOC_README.md line 361 leads with "2026-09-28: current-main reconciliation is prepared" and ends "Held: ... ruling 534's quiet GPU-on A/B and S16".
- mere_docs/testing/2026-08-20_burn_0_22_prerelease_closure.md lines 3-10 say "Main stays on pre.2 until S16". That breaks S15's own done-condition (§13.7: "no current-state doc says pre.2 is production"). The mesh host lanes plan breaks it too (record 2).

### Recommended action

- Add a dated current-status line at the top covering:
  - S16 ran (cec0b3a4, origin 07db35e2) and main is on pre.4;
  - S17, Knot's repin on knot-editor `mere-pre4-repin`, is in progress, and djinn's Knot pin waits on it;
  - S18, Isometry's repin, waits on Isometry's checkpoint 9;
  - stable 0.22 closure remains.
- Relabel the 2026-09-27 "Status" paragraphs as dated annotations, so the first `**Status` line is the current one.
- Record the S16-time revisit of S0-8 (its §13.13 answer preserved the lane), or put it back to Mark. Append S16 to §11 Progress.
- Update the DOC_README entry and the closure receipt's top annotation.

### Notes

Ancestors of the base: cec0b3a4, 07db35e2, 4b0713db, 344196c2 (main 9680306d), 06423cab, d101d7ff, 63345c17/64c917d5, 8f61b367/eba741c5, a924f380/f62581c7/bcb57356/b73695da, 124fc42b and 9d778fc5 (also in .git-blame-ignore-revs:7).

Also present at the base:
- rust-toolchain.toml pins 1.98.1, and the runners dot-source scripts/repo-toolchain.ps1;
- the getrandom rustflags are in all six tracked `.cargo/config.toml` files;
- `wasm-bindgen = "=0.2.129"` is in all six wasm manifests;
- `run_static_constructors_once` is in cambium-genet-web-host/src/start.rs;
- the root, probe and two repro locks all have wgpu 30.0.1, burn pre.4 and no turso;
- the root patch table has cubecl-runtime and burn-remote rows (Cargo.toml:677, 686).

Knot's `mere-pre4-repin` branch holds 54bb8cd, and the root still pins knot-editor 562353aa. Isometry still pins mere 32edc2ad, which predates cec0b3a4. Whether stable 0.22 is out is unverifiable offline: the local index cache (written 2026-09-28) lists only pre.1 to pre.4.

## mere_docs/implementation_strategy/2026-08-09_mesh_host_lanes_plan.md

- disposition: historical-marked
- status line: "Status: Implemented through the Burn 0.22.0-pre.2 production row. H0, H1, H2, and the lease-bound remote adapter have landed. Stable Burn 0.22 repinning and its clean package receipt remain release-gated." — accurate: no
- claims checked: 13 — holds: 10, stale: 2, unverifiable: 1

### Stale claims

- The status line and §5 ("The remote adapter uses this exact production prerelease row") call pre.2 production. Since cec0b3a4 (2026-10-05), main is on 0.22.0-pre.4, and the vendored burn-remote is version 0.22.0-pre.4 (support/patches/burn-remote/Cargo.toml:15).
- The 2026-08-20 Progress entry, the plan's last word on the graph, says "one wgpu 30.0.0 and one libsqlite3-sys 0.38.2". The root lock now has wgpu 30.0.1 and no libsqlite3-sys or turso entries (ruling 375's cubecl-runtime patch, Cargo.toml:673-677).

### Contradictions

- This status line breaks the burn plan's S15 done-condition (§13.7: "no current-state doc says pre.2 is production").
- DOC_README.md line 142 calls the plan "open". Its status says implemented, with only the external stable-Burn gate left.

### Recommended action

- Re-date the status: production is the pre.4 row since cec0b3a4, with burn-remote's close patch carried at pre.4. Stable closure belongs to the burn plan.
- Correct §5's present-tense pre.2 sentence.
- Archive or not is a fork (below).

### Notes

- Commits: 7fb07225 and 176c31e8 are ancestors. p2panda 9f2c2a01 ("net: admit exact external ALPN handlers") is in tag mere-p2panda-net-0.7.5, which the root pins (Cargo.toml:701). `accept_raw` is called at ports/distillery/src/remote.rs:203.
- The folded host: ports/distillery/src/mesh_host.rs plus mesh_host/{host,inflight,courier,sense}.rs. `tick` is at host.rs:288 and `still_held` at inflight.rs:65. There are 4+3+3 unit tests plus tests/mesh_host_{blob_delivery,supervised_reclaim}.rs, which matches the plan's 10 + 2.
- In crates/mesh/mesh: DeviceAttested, LeaseActivity, CheckpointError::LiveLease, collectable_blobs, live_leases and BlobCollected all exist. `verify_output` returns NotCheckable for non-ExactBytes outputs (registry.rs:241). mesh-peer uses `DevicePolicy::unsupervised` (examples/mesh-peer.rs:309).
- Whether stable Burn has been published is unverifiable offline.

## mere_docs/implementation_strategy/2026-08-12_eidetic_reorg_plan.md

- disposition: historical-unmarked
- status line: "Status: open; authorized by Mark 2026-08-12 ("you can reorg eidetic"). Execution is timed around the sibling sessions currently in mere's tree (distillery v0, moothold): the moves touch the workspace manifest, so they land as one commit when the tree quiets, with the search wiring plan's W4 consuming the new homes." — accurate: no
- claims checked: 12 — holds: 7, stale: 4, unverifiable: 1

### Stale claims

- The status line says the moves land later. The plan's own Progress says "2026-08-12 — executed", and the tree agrees:
  - crates/intel/esp/src/embed/persistence.rs exists behind `persistence = ["dep:eidetic"]` (esp Cargo.toml:78);
  - field_bridge.rs and canvas_search.rs are in crates/canvas/pictograph/src/canvas/;
  - no `mere-embed` package remains.
- E-R4 says "the fetchers stay crates". 254b23b6 (2026-09-23) folded them into eidetic-core as the features `https-fetcher` and `iroh-fetcher` (eidetic-core Cargo.toml:59-60; src/https_fetcher.rs, src/iroh_fetcher.rs).
- The 2026-08-12 entry chose to keep graphshell-web a gated workspace member "over moving it to exclude". Since 74b55236 (2026-08-24), the rejected arrangement is the one in the tree:
  - ports/graphshell/web/Cargo.toml:16 has its own `[workspace]`;
  - the root lists it under `exclude` (Cargo.toml:156-167);
  - it restates the root's patches.
- E-R3 says to "sweep its doc references", and done-condition 4 says the same. That is unmet for the census: design_docs/2026-08-10_leverage_census_brief.md:57 still reads `mere-embed` "Keep; wires in as W4".

### Contradictions

- Inside the plan: the status line says "open", and Progress says "executed".
- DOC_README.md line 111 still describes the superseded draft: `persistence` becoming eidetic-core's `vector` module behind `vector-index`, landing "as one commit when the tree quiets".

### Recommended action

- Mark the status landed (2026-08-12).
- Fix the census row and the DOC_README entry.
- Add dated notes for 254b23b6 (fetchers) and 74b55236 (graphshell-web's own workspace).
- Archive (fork below).

### Notes

Also checked: crates/intel/eidetic-search/Cargo.toml:40 gives the eidetic-recall example `esp` with the `persistence` feature. No manifest names `mere-embed`, and no stray `embed::` crate paths remain. `hex_digest` is at ports/graphshell/src/transfer.rs:788, and `#![cfg(target_arch = "wasm32")]` at ports/graphshell/src/web.rs:21. The search plan's W4 names the new homes. The esp persistence test count (6) is unverifiable without cargo.

## mere_docs/implementation_strategy/2026-08-12_search_surface_wiring_plan.md

- disposition: current
- status line: "Status: open. W4's lexical n-gram input probe, W5's deterministic fusion probe, and the V3 tokenized-URL repair completed 2026-08-31. Turnstone's live trail-fusion caller and private captured-trail evaluation harness are implemented on paired feature branches as of 2026-09-02. W4 canvas host wiring remains open. W5 has an executable promotion gate, but the active profile does not yet admit a real training/held-out selection. Spun out of the [leverage census](../../2026-08-10_leverage_census_brief.md) (step 2), and carries the census's audit answer for `mere-embed` inside it." — accurate: no
- claims checked: 14 — holds: 11, stale: 3, unverifiable: 0

### Stale claims

- The status says the caller and harness are "implemented on paired feature branches". Turnstone 7d6e348 and 57faef4 are on turnstone main and origin/main, and Mere 7b6ced78 is an ancestor of the base.
- The status's last word on W5 ("an executable promotion gate ... does not yet admit ... selection") is overtaken by the plan's own 2026-09-08 entry, which retired the hashed vector lane from recall. Turnstone has no direct esp dependency (it reaches its lock transitively through pictograph, mere-mesh and knot-editor) and no `phrase_order`/`phrase_influence` settings. The status never mentions W6a to W6e, the engine swap or the candidate index, all recorded as built.
- The 2026-09-07 entry says these commits are "unpushed pending the family re-pin": turnstone c863b8e and 696bc2f, mere 5231ae3e, 7a5b7e19 and fc0c9e8c. All are on main and origin/main in both repos.

### Contradictions

- Inside the plan: §2's W5 slice ("fuse() merges W2's lexical ranking with W4's vector ranking in the omnibar") clashes with the 2026-09-08 retirement.
- DOC_README.md line 112 says "a learned-vector baseline remain[s] open". The plan records the MiniLM baseline as complete on 2026-09-01. The entry also omits W6 and the retirement.

### Recommended action

- Rewrite the status, dated:
  - W1, W2, W6a to W6e and the engine swap have landed on both mains;
  - the vector lane was retired from recall on 2026-09-08;
  - W4 canvas wiring is open;
  - W5 needs a ruling (fork below).
- Update the DOC_README entry.

### Notes

- Turnstone: 539dacc, f22f61f, 7d6e348, 57faef4, c863b8e and 696bc2f are on main and origin/main. Its design_docs/2026-09-02_trail_recall_evaluation_plan.md exists.
- Mere ancestors: 7b6ced78, 5231ae3e, 7a5b7e19, fc0c9e8c and d82afa17.
- Present at the base:
  - eidetic-core src/browsing/{frecency,page,text}.rs;
  - eidetic-search src/{tokenize,bm25,candidates,fusion}.rs, with no tantivy dependency;
  - spec.rs:48 `FIELDS_V4` and `engine_version`;
  - esp `embed_sparse` and `SPARSE_INDEX_SCHEMA_REF` (persistence.rs:75);
  - scripts/firefox_history_export.py.
- canvas_search and field_bridge are referenced only by pictograph/src/canvas.rs, so W4 is unwired, as the plan says.

## mere_docs/implementation_strategy/2026-08-20_device_resident_consolidation_plan.md

- disposition: current
- status line: "Status: R1 through R5, C1 through C4, V1 automated, the Turnstone headed edit/close/restart slice, and the product-neutral resident extraction are complete. Physical two-device and the remaining standalone/evidence-headed receipts remain open." — accurate: yes
- claims checked: 11 — holds: 9, stale: 1, unverifiable: 1

### Stale claims

- §6 says the standalone sync-status part is open "because the repository has no standalone Knot executable yet". That blocker is gone, though the receipt may still be open:
  - Knot's sources left mere on 2026-09-04 (root Cargo.toml:95-96);
  - knot-editor ships a `knot` desktop binary (apps/desktop/Cargo.toml `[[bin]] name = "knot"`, added in 4434584, 2026-09-01).

### Contradictions

- DOC_README.md line 166 says "approved direction, implementation open 2026-08-20". The status marks R1 to R5 and C1 to C4 complete, with djinn as the composition root.

### Recommended action

- Add a dated note: `KnotResidentSource` and `KnotSpaceAuthoritySnapshot` now live in knot-editor (crates/knot-editor/src/authority.rs, endpoint.rs), and Knot has a standalone binary. Name where the remaining standalone and evidence-open receipts get produced.
- Refresh the DOC_README entry.

### Notes

- Ancestors: 91f1297e, 7655a625, f13ebf09, 72b7fdb2, 71a4b81f, 90a1a66d, 0f0a8006, 228213fe and 4565d040. Turnstone 1464043, d6c4bdc, 02772e0, 952f0df and 8637abe are on turnstone main, and genet 9d3f2bd3031 exists.
- Symbols present:
  - `PortableContentRefV1` (crates/chirograph/src/titulus.rs);
  - `P2pandaOverlayHost` (crates/murm/transport/src/p2panda_host.rs);
  - `BlobLease` (crates/murm/transport/src/blobs.rs);
  - `AppRouteCarrier` (ports/graphshell/src/native/app_client.rs).
- djinn uses `distillery::lifecycle` (resident.rs:18). Graphshell has no Knot dependency, and `graphshell_device_host` appears nowhere.
- Whether the C1-consumer receipt has been run is unverifiable.

## mere_docs/implementation_strategy/2026-08-22_djinn_family_resident_services_plan.md

- disposition: current
- status line: "Status: planned, gated on the `djinn 0.0.2` release" — accurate: no
- claims checked: 12 — holds: 10, stale: 2, unverifiable: 0

### Stale claims

- The status says the whole plan is planned and gated. §13's ordinary Gemini publication service was implemented and receipted on 2026-09-13 without that gate (ports/djinn/src/resident_site.rs, src/bin/djinn_site.rs). §13 itself says "Status: ordinary saved-snapshot service implemented".
- §4 A4 describes `ReleaseRefV1` as future work. It exists at crates/system/luggage/src/release.rs:62 (landed in d10e04da, 2026-09-01) and is used by crates/murm/webrtc-carrier/src/invite.rs. `ReleaseOfferV1` and the versioned manifest are absent, so A4 is partly done and unrecorded.

### Contradictions

- Inside the plan: the top status says "planned", and §13's status says "implemented".
- DOC_README.md line 167 repeats "planned after the `djinn 0.0.2` registry release" and omits §13.
- The Gazette row says "unbuilt". ports/gazette exists and reports "Built today: ... WebFinger (RFC 7033) resolution and unverified Gaz address intake" (Cargo.toml:7). Only the djinn composition is unbuilt: djinn has no gazette dependency.

### Recommended action

- Re-date the status: the entry gate still governs Phases A to G, §13's local service is implemented, and A4's `ReleaseRefV1` has landed.
- Word the Gazette row as "djinn composition unbuilt".
- Update DOC_README.

### Notes

- The entry gate holds: djinn is version 0.0.2, and its knot-site dependency is git-only with no version (ports/djinn/Cargo.toml:60), so `cargo package` would refuse it.
- A1 is not done: Hocket's `UpdatePolicy`, `UpdateChannel` and `decide` are still in woodshed ports/hocket/crates/hocket-genet/src/update.rs:46, 98, 454, and none are in Luggage.
- Present: DjinnResident, resident_knot.rs, resident_blobs.rs and crates/system/errand/src/serve. knot-editor 6b68405 is on main with `PublishedSnapshotV1` (crates/knot-site/src/lib.rs:375). Mere b5750a96 is an ancestor.
- No notifications crate and no installed-app inventory exist.

## mere_docs/implementation_strategy/2026-08-28_derived_faces_plan.md

- disposition: historical-marked
- status line: "Status: D1 published as `pictograph` 0.1.0; 0.2.0 publishes D2's bridge and D4's derivation grammar on 2026-09-04. D3 landed as a Mere integration on 2026-09-02 and D4 default-scale legibility landed on 2026-09-03. Editing remains deferred. The original gate was emblem's encoder (`repos/emblem/design_docs/2026-08-28_encoder_plan.md`), whose E1–E4 all landed and shipped as emblem 0.2.0 on 2026-08-29." — accurate: yes
- claims checked: 11 — holds: 9, stale: 1, unverifiable: 1

### Stale claims

- §4 describes `pictograph` as the face generator beside Canvas ("Canvas keeps its own portable sink"; "Canvas grows the Derived face arm"). f590e45d (2026-09-24, "Fold canvas and mere-signals into pictograph") moved Canvas into pictograph/src/canvas/. The manifest still reads `version = "0.2.0"` (Cargo.toml:3), so the in-tree 0.2.0 no longer matches the published 0.2.0 that the status describes.

### Contradictions

- none.

### Recommended action

- Add a dated note on the canvas fold.
- D1 to D4 have landed and only editing (§7) is deferred. Extract §7 and the §8 branding-gap finding, then archive (fork below).

### Notes

- In pictograph:
  - Cargo.toml has `vello = ["dep:netrender_vello"]`, and emblem is pinned at "0.2.0" (root Cargo.toml:203);
  - lib.rs:64 has `DERIVATION_VERSION: u32 = 3`;
  - src/canvas/types.rs:142-156 has `Face::Derived` with its default/override docs;
  - src/canvas/tests/node_face.rs exists.
- ports/graphshell/web/scenarios/d3_derived_faces_detail.scn exists.
- Ancestors: 42731a59, 1e59c0b7, d42fd1fb, 5a4f990c, db85375e, be007322, 81c86f29 and dc9dd8ca.
- Receipt directories exist: Code/testing/mere/derived-faces/d4_81c86f29 and scenarios/graphshell-web/d4_small_lod_81c86f29. The emblem encoder plan exists.
- Registry publication of 0.1.0 and 0.2.0 is unverifiable offline.

## mere_docs/implementation_strategy/2026-10-05_djinn_test_harness_plan.md

- disposition: current
- status line: "Status (2026-10-05): all forks ruled (§3, rulings 1 to 13). H1 to H3 landed on `main` (`318b8f70`), the graceful stop fixed (ruling 11). H4 to H6 open; H4 is built with the vault lock (its ruling 18)." — accurate: yes
- claims checked: 12 — holds: 11, stale: 0, unverifiable: 1

### Stale claims

- none.

### Contradictions

- none. One nuance: H4's first box ("the status route reports lock state and startup mode") is unticked, but resident_status.rs already has a lock-state field (fixed at Unlocked "until the vault lock exists") and `StartupUnlockV1`. H4 starts part-built.

### Recommended action

- none for this record.

### Notes

- Merges: 318b8f70 merges 45216587, and 38ed523a and c55faaff are ancestors. The mere-verify merges 6ed24c46 and 935e10ce exist but are not on main, as the plan says.
- Present at the base:
  - `ResidentTasks` (ports/graphshell/src/native/tasks.rs:28);
  - crates/system/djinn-testkit, with `publish = false` and no djinn, castellan or personae dependency;
  - both routes in ports/djinn/src/resident_status.rs;
  - the flags in src/bin/djinn.rs, and `--installed` in install-windows.ps1;
  - `mere.djinn.receipt/v1` in receipt.rs;
  - the SSH_AUTH_SOCK refusal in ports/castellan/src/authority.rs.
- The vault lock and pairing plans exist.
- The D1/D1b runtime outcomes are unverifiable without running them.

## mere_docs/technical_architecture/2026-07-08_generic_graph_substrate_plan.md

- disposition: current
- status line: "Status: planning. Decisions below locked with Mark 2026-07-08; the substrate crate name is the one open pick. Home is mere's design_docs because mere is the donor of the model and the largest eventual consumer, but the substrate itself will be a standalone repo (the muniment/codicil pattern), not a mere crate." — accurate: no
- claims checked: 10 — holds: 5, stale: 5, unverifiable: 0

### Stale claims

- "planning": the plan's own 2026-08-08 section says G0 to G2 landed and G3 is met in substance. chartulary 0.2.2 ships caps, graph, container, taxonomy, edit, spine, commit, facet, content_class, nested and stemma.
- "the substrate crate name is the one open pick": §1.4 and §8 decided `chartulary`, and §9 says the name is resolved.
- "a standalone repo ..., not a mere crate": it is crates/eidetic/chartulary inside mere. Woodshed consumes it from mere.git (woodshed-core/Cargo.toml:37), and there is no repos/chartulary.
- §2 and §5 make codicil the edit-log crate, and the 2026-08-08 section says "GraphLog over codicil". `GraphLog` journals into `muniment::Journal` (spine.rs:7, 34). "codicil" now names Eidetic's renamed engrams (c51b9704, 2026-08-31; eidetic-core/src/codicil.rs).
- "`scholia` ships `to_jsonld`, `to_nquads`, and `to_quads`": scholia was folded into `chartulary::rdf` (275448cd, 2026-08-31; rdf.rs:120, 144, 188).

### Contradictions

- Inside the plan: the status line ("name open", "planning") clashes with §8/§9 and the 2026-08-08 rung check.
- DOC_README.md line 302 says "planning", leaves G2 and later rungs open, and describes "codicil (edit log spine)" and a separate scholia.

### Recommended action

- Rewrite the status, dated:
  - G0 to G2 have landed;
  - G3 is credited, pending a ruling on its reworded clause;
  - G4 has only its export half;
  - G5 has begun from the top;
  - the home is crates/eidetic/chartulary.
- Replace codicil and scholia with muniment `Journal` and `chartulary::rdf` wherever they name current mechanisms.
- Update DOC_README.
- Note that the plan sits under technical_architecture, though DOC_POLICY §8 puts plans in implementation_strategy.

### Notes

The G4 claim holds: rdf.rs lines 7-25 list losslessness, compact JSON-LD and SPARQL as future work. Woodshed's `PracticeHistory` is over `Stemma<String, (), String, Engagement>` (woodshed-core/src/history.rs:99), and woodshed-graph exists. Turnstone uses `chartulary::FacetId` and `AcceptAll` (src/app/node_arms.rs:282-284). The mere crates that depend on chartulary are graph-kernel, pandect, graphshell, commons, gemot, servitor, pictograph, seiche, eidetic-core, alembic and athanor.

## mere_docs/technical_architecture/2026-08-13_spatial_compute_plan.md

- disposition: historical-marked
- status line: "Status: complete 2026-08-13; amended 2026-08-14 (§0.5: lanes are program shapes, resident-views law). P1 through P4 all decided and their work landed. P4 is a narrowing rather than an extraction (see below), with its one live hazard closed in code. Every open item is closed: slot stability ruled, the kernels promoted into `quint::resident` with the rust-gpu carriage working, and the windowed run presented. What remains is not this plan's: the promotion trigger for the lease (a shipped producer and consumer) and the renderling shader-edit wall, both with named watch conditions. The GPU architecture for conatus and its render consumers, ratified by Mark from the conatus discussion of 2026-08-13 (the projection ruling and its amendment in [the field system extraction doc](2026-05-30_field_system_extraction.md) are this plan's ground). Gates P1 to P4 below." — accurate: no
- claims checked: 12 — holds: 7, stale: 5, unverifiable: 0

### Stale claims

- The status and P2's closing paragraph say the kernels were "promoted into `quint::resident` with the rust-gpu carriage working", via `quint_shaders.spv` and `PASSTHROUGH_SHADERS`. That is gone:
  - the kernels migrated to CubeCL on 2026-08-16, which the plan body records (lines 116-135): `9cf7b3e5` removed `PASSTHROUGH_SHADERS` and the WGSL fallback, and `a9982673` recorded it in quint-shaders' README;
  - eae87153 (2026-08-31) moved resident.rs to crates/conatus/conatus/src/resident.rs (feature `resident`) and deleted quint-shaders and the .spv; so the stale text is the status line, the P2 paragraph (313-321) and the `quint::resident` path;
  - no .spv and no `PASSTHROUGH_SHADERS` remain anywhere.
- The status keeps "the renderling shader-edit wall" as a watch condition. The plan's own 2026-09-15 annotation says renderling is axed.
- §0's "The row is currently split, and that is a live break (2026-08-15)" is the plan's last word on wgpu. The root now has one row, wgpu 30.0.1 (Cargo.toml:513-524).
- P2, P3 and P4 cite `paredros/probes/ambience-lease`.
  - Paredros became Eponym (isometry 9410128b, 2026-09-24), and the probe is now isometry/eponym/probes/ambience-lease.
  - The P3 receipt moved to Code/testing/eponym/from-repos-testing-2026-09/.
- "Full detail in `quint-shaders/README.md`" points at a file deleted in eae87153.

### Contradictions

- Inside the plan: the status's renderling watch condition clashes with the 2026-09-15 annotation.
- DOC_README.md line 297 says "founded and completed 2026-08-18" and that P1 to P3 landed 2026-08-18. The plan dates them 2026-08-13.

### Recommended action

- Add a dated annotation:
  - the resident kernels are CubeCL in `conatus::resident` (a9982673, eae87153);
  - the rust-gpu carriage is retired here;
  - the renderling watch is moot;
  - the wgpu row is unified on 30.0.1.
- Mark the paredros paths historical or repoint them to eponym.
- Fix the DOC_README dates.
- Archive vs architecture role is a fork (below).

### Notes

- Seiche: `mod propose` (crates/conatus/seiche/src/lib.rs:194) and tests/contact_to_fact.rs exist.
- crates/probes/resident-graph is absent, and its citation is already marked historical.
- No `SpatialBufferLease` exists, consistent with P4's narrowing.
- netrender 1ce733be6 is on HEAD, with a `greedy` field on its tenant needs (netrender_device/src/core.rs:90).
- The rust-gpu fork branch `mark-ik/prerelease-version-gate` exists.
- Code/testing/mere/p2_resident_graph.png exists, and so does the field system extraction doc.

## moothold_docs/implementation_strategy/2026-06-30_bounty_verification_economy_plan.md

- disposition: current
- status line: "Status: Planned outer-ring architecture." — accurate: yes
- claims checked: 5 — holds: 4, stale: 1, unverifiable: 0

### Stale claims

- The Boundary, Ledgers and Epoch Receipts sections treat "Tessera" as the live standing ledger. It is now mien's `Standing` (crates/moot/mien/src/ledger.rs and others). Gemot reads Tessera only as legacy: `tessera.redb` and the `tessera_operations` alias (crates/moot/gemot/src/moot/service.rs:62-69, 128). DOC_README calls this "historical Tessera wording", but the plan itself is unmarked.

### Contradictions

- none.

### Recommended action

- Add a dated vocabulary note (Tessera is now Standing) and date the status line.

### Notes

- None of the plan's proposed types exist: BountyEvent, ResultSpec, MootEpochHeader, WorkReceipt, RoleWitness, VerificationTier, BountyId, CreditAmount, CommitmentLapsed. So "Planned" holds and no done-condition is met.
- All five Related links resolve.
- The typed commitments it relies on exist: `proofs::Commitment` and `proofs::Digest` (crates/system/proofs/src/lib.rs:26, 87).
- The mesh crate carries no market types.

## nematic_docs/implementation_strategy/2026-06-13_polyglot_block_resolver_plan.md

- disposition: current
- status line: "Status: Planned. Extends the knot evaluation + export plan (`genet/design_docs/archive_docs/2026-09-02/2026-06-12_knot_evaluation_export_plan.md`) (K1 transclude, K2 eval, K5 export, all landed) with the forward architecture: a single registry that resolves *any* fenced block by its tag, and the new resolver kinds that registry makes pluggable." — accurate: yes
- claims checked: 8 — holds: 6, stale: 2, unverifiable: 0

### Stale claims

- "Rhai + Lua backends" / "a working menu (Rhai + Lua)": only `RhaiEvaluator` implements `BlockEvaluator` (crates/script/rhai/src/lib.rs:85). 6de9a486's own message says the piccolo Lua path was "demonstrated by a local probe".
- "the existing `register-mod-loader`/extism seam": register-mod-loader was folded into the `registry` crate (3430ba2b, 2026-09-23; `WasmModRuntime` in registry/src/mod_loader.rs). No manifest depends on extism. The only `WasmModRuntime` implementation is crates/script/document-host/src/runtime.rs, on wasmtime 45.

### Contradictions

- none.

### Recommended action

- Correct the backend menu and P3's seam name with a dated note, and date the status line.

### Notes

- P0 is unbuilt: no `BlockHandler` and no `resolve_blocks`.
- The three passes exist: `expand_fenced_blocks` (crates/nematic/nematic/src/knot/expand.rs), and `resolve_transclusions` and `evaluate_blocks` in inker. Each keeps its own `BlockProvenanceMap` (transclude.rs:95, evaluate.rs:183), which is the duplication P0 targets.
- The archived K plan exists in genet. crates/platen/platen and the smolweb fidelity plan exist.
