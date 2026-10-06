# Batch 45 — S14 pass, phase B7 (re-judged plans)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/testing/2026-07-07_headed_automation_plan.md | historical-unmarked | no | 9 | 6 | 3 | 0 |
| moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md | current | yes | 14 | 11 | 3 | 0 |
| moothold_docs/implementation_strategy/2026-08-31_flora_tulpa_standing_plan.md | historical-marked | no | 10 | 7 | 3 | 0 |
| nematic_docs/implementation_strategy/2026-06-27_native_smolweb_rendering_plan.md | historical-marked | yes | 10 | 7 | 2 | 1 |
| verso_docs/implementation_strategy/2026-06-23_genet_scrying_flipcarrier_plan.md | current | no | 12 | 9 | 3 | 0 |
| cambium_docs/implementation_strategy/2026-07-15_component_catalog_growth_plan.md | current | no | 13 | 11 | 2 | 0 |
| cambium_docs/implementation_strategy/2026-09-03_host_ui_zoom_plan.md | current | yes | 12 | 11 | 1 | 0 |
| dramatis_docs/implementation_strategy/2026-10-01_chatelaine_cxf_plan.md | current | yes | 9 | 9 | 0 | 0 |
| eidetic_docs/implementation_strategy/2026-06-12_eidetic_browsing_derivation_plan.md | historical-unmarked | no | 11 | 5 | 5 | 1 |
| mere_docs/implementation_strategy/2026-06-08_apparatus_pane_and_theme_switcher_plan.md | historical-unmarked | no | 8 | 4 | 4 | 0 |
| mere_docs/implementation_strategy/2026-06-09_shellbar_plan.md | historical-unmarked | no | 8 | 5 | 3 | 0 |
| mere_docs/implementation_strategy/2026-06-26_federation_interop_plan.md | current | yes | 6 | 3 | 3 | 0 |
| **Totals** |  |  | **122** | **88** | **32** | **2** |

**Totals: 12 docs, 122 claims checked (88 holds, 32 stale, 2 unverifiable), 13 contradictions; 7 status lines wrong.**

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

Checked directly in this session: no `.rs` file at the base names
`MEERKAT_SCENARIO` (headed automation); `a1551086` is an ancestor of the base
(FLORA, Tulpa and Standing); `crates/inker/inker/src/flip/orchestrator.rs`
exists, the verso-tile crate folded in (flipcarrier). The verifier found one
claim half refuted (`is_surface_engine` survives in genet's
`document-session-api`, re-exported by `inker::routing` and called by Pelt;
only `engine_pins` is gone) and 5 items true only in part (Gemot's ownership,
which keeps Tulpa, FLORA and the Standing lane; DOC_README's FLORA date, an
omission; the component catalog's brief reference; the browsing plan's E5,
only partly delivered, with corrected commits; the federation plan's schema
"registry"); those records are corrected.

## mere_docs/testing/2026-07-07_headed_automation_plan.md

- disposition: historical-unmarked
- status line: "Status: Assessment + the unification, now built and complete for the whole session. The "what ails it" fixes landed with the Slice 3 headed check; the `MEERKAT_SCENARIO` self-drive mode + shared scenario vocabulary landed 2026-07-08 (multi-window + settings verified headed), and the `navigate` + `key` verbs for flows outside the registry landed the same day (find verified headed). Only pointer gestures remain outside a scenario (Migration item 4)." — accurate: no
- claims checked: 9 — holds: 6, stale: 3, unverifiable: 0

### Stale claims

- The status line and the closing paragraph (l.167-172) say the scenario mode and the "one PS base (mk-harness) that only launches and collects" are built and complete. Meerkat left the workspace with all of that code in `c5f01064` (2026-07-18, "The funeral: meerkat leaves the workspace"), which is an ancestor of the base. `Code/testing/mere/scripts/mk-harness.ps1:70` still sets `$MK_EXE` to `target\debug\meerkat.exe`, which no longer builds.
- Four meerkat paths have no historical marker and are absent at the base: `render/paint.rs` (l.68), `scenario/runner.rs` (l.129), `settings.scn` (l.152) and `scenarios/find.scn` (l.164). No `MEERKAT_SCENARIO` symbol exists in any `.rs` file.
- Migration item 4 (l.165) says "the orphaned `C:\t\meerkat-target` build tree can still be deleted". That directory no longer exists.

### Contradictions

- Internal: the status line says pointer gestures are "(Migration item 4)", but item 4 (l.165) is the `C:\t\meerkat-target` cleanup. Pointer gestures are only in the closing paragraph.
- DOC_README.md:444 says the plan "Proposes a `MEERKAT_SCENARIO` self-drive mode". The plan says that mode landed on 2026-07-08.

### Recommended action

- Add a dated banner saying meerkat and its scenario mode left on 2026-07-18 (`c5f01064`), and that live scenario work is now graphshell's `web_scenario.rs` and the `.scn` runners in `crates/cambium`.
- Fix the "Migration item 4" reference and drop item 4.
- Archive per DOC_POLICY §8, and correct the DOC_README entry.

### Notes

Checked: `c5f01064` ancestry, `git grep MEERKAT_SCENARIO`, `.cargo/` at the base (only `config.toml.example`, so the target-dir override is gone), `Code/testing/mere/scripts/` (`mk-harness.ps1`, `drive-s3c.ps1`), the `s3c-*.png` shots, the 138 files in `Code/testing/_archive/scripts`, and the absence of `C:\t\meerkat-target`. Batch 12 judged this plan historical-marked. I differ because the plan does not say anywhere that its subject was deleted: only three paths carry mechanical markers, and there is no banner.

## moothold_docs/implementation_strategy/2026-06-12_moot_object_m1_plan.md

- disposition: current
- status line: "Status (2026-09-06): Historical M1 landed as recorded below. The active continuation is [Community collections and author-offline publishing](#community-collections-and-author-offline-publishing-2026-09-04). Its same-machine live-peer process proof passed with stable-Persona binding and current Gemot command authority. Production publication/hosting records, historical authority proof, the Persona-to-device adapter, and a two-machine receipt remain open. The original M1 body preserves its dated vocabulary and ownership. Current owners are Gemot for community authority and recognition, Commons for shared graph operations, and Stickleback for accepted-operation replication. Historical references to flora as the artifact catalog mean fauna; FLORA is the separate federated adaptation lane. Possession of a moot id does not replace current admission and delegation checks." — accurate: yes
- claims checked: 14 — holds: 11, stale: 3, unverifiable: 0

### Stale claims

- l.347-349 says "The concurrent proof artifact is not yet committed" and marks the proof link `*(planned target)*`. At the base, `design_docs/moothold_docs/research/2026-09-05_author_offline_publication_proof.md` and `research/receipts/2026-09-05_author_offline_publication.json` both exist.
- l.328 calls `crates/moot/commons/examples/commons_practice_peer.rs` "Concurrent untracked" and marks it `*(planned target)*`. The file is tracked at the base.
- The plan's last word on P3a and P3c (l.456-460, 515-516) is that the captured-web consumer "has no production Fleece record resolver to consume yet" and that "P3c waits for both". Turnstone has since landed both:
  - source capture: `b4e69ce`, "capture explicit page sources with Fleece", 2026-09-09;
  - the collection consumer: `6b77e34`, "project local Fleece captures into places", 2026-09-09;
  - collection-scoped search: `aa515bd`, 2026-09-10.
  These are recorded in turnstone `design_docs/2026-08-28_page_capture_plan.md:407-436`, which calls P3a "in progress".

### Contradictions

- Internal: the status line (2026-09-06) does not mention P3b (`b99e532a`) or P3d (`6f1f73bf`), both landed 2026-09-09.
- Internal: "P3d follows the working consumer" (l.516), yet P3d is recorded as implemented while the plan records no P3c consumer.

### Recommended action

- Refresh the status line: add its date, P3b and P3d landed, and P3a/P3c progress in Turnstone with a pointer to Turnstone's page capture plan.
- Remove the stale `*(planned target)*` markers and the "not yet committed" sentence.

### Notes

- All on the base:
  - `155145cd`, `ee533436`, `499b892e`, `b99e532a` and `6f1f73bf` are ancestors.
  - `crates/moot/gemot/src/moot/` and `examples/moot-peer.rs` exist.
  - `moot/hosting/` appears only as a fixed scope in `examples/author-offline-publication.rs:263`, so production hosting is open.
  - `author-offline-publication.rs:1368` states that the device-key adapter is still needed.
  - No two-machine receipt exists; the control grep found the proof doc's own mention.
  - These symbols are present: `CaptureEvidenceV1` (`document-lanes/src/eidetic_bridge.rs:142`), `DocumentIndex<K>` (`eidetic-search/src/document.rs:67`), `Withdrawn` (`records/roster.rs:275`), `withdraw_share_for_identity` (`service.rs:809`), `Replica::accept` (`commons/src/lib.rs:955`), and `AvailabilityPolicy` plus `lease.rs` in mesh.
- Not recorded in the plan: the co-op lifecycle contract in `ports/moot/src/coop.rs` (`4837248d`, 2026-09-17), which is P4-adjacent.

## moothold_docs/implementation_strategy/2026-08-31_flora_tulpa_standing_plan.md

- disposition: historical-marked
- status line: "Status: landed on `main` (2026-09-02), after the integration branch completed on 2026-08-31. Gemot owns the social protocol, Distillery owns exact tensor execution, and the integrated signed, replicated, restart-durable receipt is green." — accurate: no
- claims checked: 10 — holds: 7, stale: 3, unverifiable: 0

### Stale claims

- Phase 1 renames Gemot's Standing domain, and the status line says "Gemot owns the social protocol". Since `a1551086` (2026-09-23, "Make mien the standing crate"), Standing's event grammar, ledger, wire, gate and store have lived in `crates/moot/mien`, and gemot depends on mien. No `standing` module remains under `crates/moot/gemot/src/moot/`. Gemot still holds Tulpa (`tulpa.rs`), FLORA (`flora.rs`) and the `gemot/standing/v1` lane (`lanes.rs:40`), so the status line is incomplete rather than wrong; Phase 1's rename of the Gemot domain is what is overtaken.
- Phase 1's done-condition says "The deprecated source aliases are visibly compatibility-only". `a1551086` deleted gemot's deprecated `tessera` source-compatibility module. Only the on-disk `tessera.redb` reader and the serde alias remain (`service.rs:65-69,128`).
- Progress (l.118-131) cites `244b66be`, `7fddd485`, `b806acf0` and `0738b709`. None is an ancestor of `535bca11` (`git branch -a --contains` lists nothing). Their main-line equivalents after the rebase are `b9cd2d30`, `3894c6a2` and `9342808c`.

### Contradictions

- DOC_README.md:614 says "complete 2026-08-31" and omits the plan's landing on main on 2026-09-02 (an omission: the integration branch did complete on 2026-08-31).
- Next door, not this doc: `crates/moot/mien/README.md` still says "No implementation yet; the working code is in `moothold`'s concord module and `gemot`'s standing store". That contradicts `a1551086`.

### Recommended action

- Add a dated annotation: Standing moved to mien and the tessera alias module was deleted (both `a1551086`).
- Map the four pre-rebase SHAs to their main-line commits.
- Then archive: every phase has landed.

### Notes

Checked: `f2924f08` and `77b3c3a2` are ancestors; `GEMOT_STANDING_LANE = "gemot/standing/v1"` (`lanes.rs:40`); the legacy readers; Tulpa's frozen `RecognitionContext` (`tulpa.rs:19,81`); `ports/distillery/src/flora.rs`; and that the receipt and the autodiff trainer plan paths exist. I could not re-run the receipt and took it as a dated record.

## nematic_docs/implementation_strategy/2026-06-27_native_smolweb_rendering_plan.md

- disposition: historical-marked
- status line: "Status: implementation complete (2026-06-27); Mere host integration and interactive eyeballing remain. Retained as a historical design and implementation record." — accurate: yes
- claims checked: 10 — holds: 7, stale: 2, unverifiable: 1

### Stale claims

- The 2026-08-03 home refinement (l.13-19) says rendering stays in "genet or cambium-nematic". That crate became the `cambium::nematic` module on 2026-09-24 (`crates/cambium/cambium/src/nematic.rs:5-12`). The views are `crates/cambium/cambium/src/nematic/views.rs:55,207,328`.
- l.267 says "errand (sibling repo `mark-ik/errand`)". Errand is now a Mere member at `crates/system/errand` (`Cargo.toml:151,350`).

### Contradictions

- DOC_README.md:540-546 lists the plan as "planning (with Mark)". The plan's banner and status say implementation complete and historical.

### Recommended action

- Correct the DOC_README entry.
- Annotate the home refinement with the cambium-nematic and errand homes.
- Archive per §8 once the host-integration remainder is extracted or ruled (see Forks).

### Notes

- Holds:
  - `1b29cda` and `7845fee` are ancestors.
  - `Block::Table` exists (`inker/src/document.rs:263`).
  - All inker paths and cross-referenced docs exist, including the smolweb repo's home decision and Genet's archived knot-evaluation plan.
- The status line's "host integration remains" holds: nothing in Mere or Turnstone calls `gemtext_view`, `gopher_view` or `feed_view`. Turnstone, Pelt and Signalman read smolweb through `mere-document-lanes`' engine-native `EngineDocument` lane (`nematic.rs:10-11`; `document-lanes/Cargo.toml:17-19`).
- Unverifiable: the errand commits `8381620`, `3ce107f` and `439b729d`. They resolve neither in mere nor in smolweb.

## verso_docs/implementation_strategy/2026-06-23_genet_scrying_flipcarrier_plan.md

- disposition: current
- status line: "Status: Design resolved; the consolidated `verso-tile` crate is landed in Mere (2026-09-03), carrying the API, flip choreography, Scry receiver, and optional Genet donor. The v1 forward carry includes URL, scroll, and session; remaining host and flip-back work stays in the Progress ledger. Both charter prerequisites are done (verified in code 2026-06-23): P4 (the scry tile) and the inker picker (the engine-picker plan's Phases 0-3 — `engine_pins` routing through `EngineRoutePolicy`, `is_surface_engine`, the apparatus engine manager, and the per-node picker) both shipped 2026-06-15. Verso is unblocked. The picker already flips a node to `scrying.web` as a *stateless* engine-switch (a fresh WebView); verso is the state-carry layer that turns that switch into a flip. The former carrier/adapters next step is represented by `verso-tile`; its remaining host integration is tracked below rather than as a future crate split." — accurate: no
- claims checked: 12 — holds: 9, stale: 3, unverifiable: 0

### Stale claims

- The status says the `verso-tile` crate is landed in Mere. There is no verso-tile crate at the base. `7a726657` (2026-09-06) folded it into `crates/inker/inker/src/flip/` behind a `genet-donor` feature (`crates/inker/inker/Cargo.toml:30`; root `Cargo.toml:138` records the fold).
- Progress (l.240-260) says "the forward flip is live" through `meerkat::scrying_host` and `node_ops::toggle_focus_compat`. Both left with meerkat (`c5f01064`). Outside inker, only `crates/system/fetch/src/cookies_flip.rs` references `inker::flip`, so no host fires the flip.
- The status says the picker already flips a node via `engine_pins` and `is_surface_engine`. `engine_pins` and the per-node picker are gone; `is_surface_engine` survives in genet `document-session-api/src/engine_ids.rs:87` (at the pin `bd3e8861`), re-exported through `inker::routing` (`routing.rs:16`) and called by Pelt (`ports/pelt/core/src/workspace.rs:12,730`); `EngineRoutePolicy` is at `inker/src/routing.rs:76`.

### Contradictions

- DOC_README.md:582-592 says the plan is "Gated on the inker picker's Phase 4". The status says "Verso is unblocked".

### Recommended action

- Update the status:
  - verso-tile is folded into `inker::flip` (`7a726657`);
  - the meerkat host wiring and per-node picker are gone (`c5f01064`);
  - the flip is library-only until a host is wired.
- Settle and fix the DOC_README gating line.

### Notes

- In `crates/inker/inker/src/flip/`:
  - the traits are at `api.rs:145,158,166`;
  - `flip_forward` and `flip_back` are at `orchestrator.rs:84,97`;
  - `GenetDonor` is at `genet.rs:38`, and `ScryForward` with its `FlipReceiver` impl is in `scry.rs`.
- Flip-back has only mock `FlipBack` impls, which matches the stated remainder.
- Elsewhere:
  - `session_jar` is at `fetch/src/cookies.rs:20`;
  - Genet's `fdac70f2b10` and `3a35b5cc4aa` are on Genet's HEAD;
  - the charter and the native session store plan exist.

## cambium_docs/implementation_strategy/2026-07-15_component_catalog_growth_plan.md

- disposition: current
- status line: "Status: active. This replaces the proposed 2026-07-09 component-catalog plan still preserved in Genet. Cambium owns reusable view compositions; Sprigging owns portable paint leaves. Applications continue to own product policy and their CSS themes." — accurate: no
- claims checked: 13 — holds: 11, stale: 2, unverifiable: 0

### Stale claims

- C5 is marked "Landed 2026-07-17", but its done-condition (l.177-178) requires the summary body to be "reused by two applications". Only Isometry calls it (`crates/isometry-views/src/downtime.rs`). Woodshed, Turnstone and knot-editor do not; `mere-verify` mirrors mere.
- l.211-212 and 222-223 point to "the 2026-08-12 brief in the repository root docs". Mere's root `design_docs/` holds only an unrelated brief of that date (`2026-08-12_family_composition_thesis_brief.md`), which the bare reference would wrongly resolve to. The intended brief is genet's `docs/2026-08-12_meristem_scope_cut_and_component_contract_brief.md`, written before the plan moved.

### Contradictions

- Internal: the status says "active", but C1-C6 and the catalog refinements are all marked landed (last on 2026-08-12) and nothing is in progress. The one unmet done-condition, C5's two-application reuse, is not named.

### Recommended action

- Either mark C5 partially met (second application outstanding) or record a second consumer.
- Repoint the brief reference to genet's `docs/` path.
- Set the status to landed, or archive with C5's reuse extracted (see Forks).

### Notes

- Present at the base in `crates/cambium/cambium/src`:
  - `OverlaySurface` and `detail_popover`;
  - `CommandState`/`CommandItem` and `SelectionState`/`SelectionItem`;
  - the `Reorder*` types and `reorderable_list_with`;
  - `disclosure`, `accordion`, `tree_view` and `summary_body`;
  - `Component`, `component` and `COMPONENT_PROBE_ATTR` (`lib.rs:107`);
  - `setting_row` (over `genet-host-api`, `setting_row.rs:5`) and `on_focus`.
- Also present:
  - `GenetAppRunner` (`runner.rs:1074`);
  - the narrow and regular receipts in `design_docs/cambium_docs/testing/receipts/`;
  - `crates/cambium/sprigging`;
  - genet's `docs/2026-07-09_component_catalog_plan.md`.
- I did not verify C0's Genet hit-test guard.

## cambium_docs/implementation_strategy/2026-09-03_host_ui_zoom_plan.md

- disposition: current
- status line: "Status: in progress (2026-09-03); Z0 through Z4 are committed in this Mere snapshot, and Z5 was recorded as landed in Isometry with its design figure open (see Progress). Current consumer adoption remains a separate check. Founded when isometry's host migration exposed a panel laid out for 820 logical pixels on a display that offers 752." — accurate: yes
- claims checked: 12 — holds: 11, stale: 1, unverifiable: 0

### Stale claims

- l.398 says "Z5 landed in isometry (uncommitted)". It is committed: Isometry `7a468533` (2026-09-03, "Move the desktop host onto the shared Cambium host") adds `crates/isometry-genet/src/host_zoom.rs`.

### Contradictions

- none.

### Recommended action

- Annotate the Z5 entry with Isometry `7a468533`. Otherwise none.

### Notes

- In `cambium-rootstock/src/host.rs` and `cambium-genet-winit-host`:
  - `layout_scale`, `available_size`, `layout_point`, `ui_zoom`, `fit_design`, `fit_zoom`, `ladder_step`, `ZOOM_LADDER`, `set_ui_zoom`, `zoom_changed`, `AppFrameInsets::scaled` and `sync_app_frame_extents` are present;
  - the tests `zoom_one_is_the_identity`, `text_is_relaid_out_at_the_new_logical_width` and `app_frame_insets_are_css_pixels_and_carry_zoom` are present.
- Commits and artifacts: `5e3d2d98` is an ancestor; the git note on genet `86019ea` is present; `Code/testing/genet/ui_zoom/` holds the receipts.
- Isometry:
  - `DESIGN_SIZE` is still `(1100, 820)` (`crates/isometry-genet/src/main.rs:158`);
  - Isometry's migration plan (l.178-193) leaves the 820-vs-1040 figure as Mark's call.
- Turnstone pulls in neither the Cambium host nor `HostOptions`.

## dramatis_docs/implementation_strategy/2026-10-01_chatelaine_cxf_plan.md

- disposition: current
- status line: "Status (2026-10-04): in progress. Shape ruled by Mark on 2026-10-01 (rulings 7 and 10 to 15 in the dramatis tier architecture; rulings 16 to 65 below). P0 met; P1 landed on `main` (`da3c50bc`); P2 landed (`3e4992ec`); P3 landed (`ff68e86c`), meeting the Mere 0.4 baseline's chatelaine condition. The review stop ended 2026-10-04 (ruling 51), and P4a landed (`007fbe7c`, rulings 51 to 63): the agent signs RSA through ring and ECDSA (P-256, P-384) beside Ed25519; held keys are never rewritten; unsignable keys and P-521 are refused at all three doors. P4 (CXF import) waits for the vault lock, in its own plan (rulings 64, 65)." — accurate: yes
- claims checked: 9 — holds: 9, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- Internal: P4a's four done-condition boxes (l.559-571) are still unchecked ([ ]), while the status and Progress (l.912) say P4a landed as `007fbe7c`. The box "ECDSA on the curves `ssh-key` supports" also conflicts with the P-521 refusal (ruling 55; `personae/src/ssh_sign.rs:16,45-47`).
- DOC_README.md:610: the dramatis tier architecture entry ends "and dramatis is the facade sibling repos pin; all unbuilt", although chatelaine's P1-P3 have landed. This doc's own DOC_README entry is consistent.

### Recommended action

- Tick the P4a boxes, amending the ECDSA box to record P-521's refusal under ruling 55.
- Separately, correct DOC_README's "all unbuilt" on the tier architecture entry.

### Notes

- Ancestors of the base:
  - phase merges `da3c50bc`, `3e4992ec`, `ff68e86c` and `007fbe7c`;
  - `f0141e3a`, `ff78acca` and `f7b31b9a`;
  - lane commits `71a91267`, `23e2b43b`, `e7c8acbd`, `0891771f`, `2d2a36c6` and `dfee134a`.
- `f8734195` and `eb79b36e` are worktree verification merges and are correctly not on main.
- At the base:
  - `chatelaine/src/kind.rs:57` carries `#[non_exhaustive]`, and `disposition.rs:35` is present;
  - castellan has `ItemStore` (`items.rs:138`) and `SecretServiceStore { items, limits }` (`secret_service/store.rs:238-241`);
  - `OtpItem`, `castellan/otp/v1` and `castellan/secret-service/v1` are gone;
  - personae has no vault lock method, and castellan hard-codes `VaultLockView::Unlocked` (`authority.rs:224`; the plan's l.940 cites :201, an older line number).
- After the base, HEAD records vault-lock L1 landing (`2556a20c`, not an ancestor of the base), so P4's gate may move soon.

## eidetic_docs/implementation_strategy/2026-06-12_eidetic_browsing_derivation_plan.md

- disposition: historical-unmarked
- status line: "Status: Active. This plan activates two of the [deferred phases](2026-06-09_eidetic_deferred_phases_plan.md) — Phase 8 (browsing memory) in full, and Phase 9's producer half native-first — and sequences them into the user-value arc Mark named: *a user derives useful information from their own browsing, for themselves*. The deferred-phases plan stays the umbrella for what this plan does not pull in (Phase 7 / OPFS, the wasm probe, and Phase 9's moot-consume half: `EngramDirectory`, merge policy, defensive ingestion)." — accurate: no
- claims checked: 11 — holds: 5, stale: 5, unverifiable: 1

### Stale claims

- "Active": E1-E4 all landed on 2026-06-12 per the plan's own Progress. The only open item is E2's real-export run; E5 is partly delivered elsewhere (next bullet but one).
- E3 and E4 are specified on tantivy: the `tantivy::Directory` produce path, the tantivy format version, and tantivy's aggregations (l.90-110, 119). `7a5b7e19` (2026-09-07, "in-tree BM25 with one tokenizer replaces tantivy") removed tantivy. The `eidetic-search/Cargo.toml` description now reads "an in-tree BM25 index ... stored-column reports".
- E5 is "gated; do not start" (l.126-130). E5's omnibar recall shipped in Turnstone under the search surface wiring plan's W2 (`18cf209`, 2026-08-12; `fuse_many` from `501d87a`, 2026-09-07): `src/trail_memory.rs:28-54` uses `BrowsingMemory`, `TrailIndex` and `fuse_many`. W3's reports and the corridor strip are not delivered, and that plan's status is "open". Explicit page sources are now captured through Fleece (P3a, `b4e69ce`), but the trail and recall corpus still carries url and title only (search wiring plan l.55-61; W6 open).
- "the vector half in `intel/embed`" (l.42): no embed crate exists. The vector index lives in `crates/intel/esp/src/embed/`.
- "live navigation memory in `node-lineage`" (l.39-40): the bridge now reads `chartulary::stemma`, aliased as `GraphMemorySnapshot` (`eidetic-core/src/browsing/lineage.rs:21`).

### Contradictions

- DOC_README.md:608 repeats "active ... shell surfacing gated on the reshape".

### Recommended action

- Update the status to landed (E1-E4, 2026-06-12).
- Annotate the tantivy → in-tree BM25, embed → esp and node-lineage → stemma changes.
- Add a pointer for E5 to `mere_docs/implementation_strategy/2026-08-12_search_surface_wiring_plan.md`.
- Archive after extracting E2's real-export run and the open questions, and update DOC_README.

### Notes

- Holds:
  - `eidetic-core/src/browsing/mod.rs:65-345`: `bootstrap_browsing_schema`, `save_trace`, `BrowsingMemory`, `record_traversal`, `recent_corridor`, `nodes_visited_in_window`, `co_occurrence`, `apply_quota`;
  - `lineage.rs:53` `project_lineage`;
  - `eidetic-search`'s `SearchIndexSpec`, `TrailIndex`, `fuse`, `top_domains`, `visits_histogram` and `examples/eidetic-recall.rs`;
  - `crates/import/src/history.rs`.
- Unverifiable: E2's done-condition, Mark's run over a real export.
- Batch 13 judged this plan current; the tantivy replacement came after that snapshot.

## mere_docs/implementation_strategy/2026-06-08_apparatus_pane_and_theme_switcher_plan.md

- disposition: historical-unmarked
- status line: "Status: Planning → building. Greenlit by Mark as the next arc after F1 (the frame tree). The first settings/system consumer of the frame-tree substrate." — accurate: no
- claims checked: 8 — holds: 4, stale: 4, unverifiable: 0

### Stale claims

- "Planning → building": the plan's own Progress records A1 and A2 landed on 2026-06-08. The meerkat app they landed in was removed (`c5f01064`, 2026-07-18).
- The `register-theme` crate (l.6, 16) is now `tabard::theme` (`ports/tabard/src/theme/registry.rs:72-75`, `chrome.rs` `mere_darker`).
- `session-runtime::settings_store` (l.21) is now `crates/system/pandect/src/settings_store.rs`. No session-runtime path exists at the base.
- The meerkat pane and orrery wiring are absent at the base: `PaneContent::Apparatus`, the Ctrl+, summon, `toggle_pane`, and `Orrery::set_palette` (the only `set_palette` is in `pictograph/src/canvas/cartography.rs`). Frisket's `PaneContent` moved to merecat in `c5f01064`.

### Contradictions

- DOC_README.md:205 says "A1 + A2 shipped ... A3 + per-theme HC node fills remain", but the subject they would land in is gone.

### Recommended action

- Mark historical with a banner (meerkat removed in `c5f01064`).
- Point the theme half to `2026-07-05_theme_modes_plan.md`, which does not itself claim to replace this plan.
- Archive. Either extract A3 and the HC node fills or record them as dropped.

### Notes

Holds: `ThemeRegistry` has four themes, including `THEME_ID_HIGH_CONTRAST` (tabard `registry.rs:72-75`); `PersistedSettings.theme_id` is in pandect; `ChromeTheme::mere_darker` is in tabard; the frame-tree archive and taxonomy docs exist. The `apparatus` crate's "uxtree skeleton" finding was true when written; the crate now holds bounded diagnostic observations.

## mere_docs/implementation_strategy/2026-06-09_shellbar_plan.md

- disposition: historical-unmarked
- status line: "Status: In progress." — accurate: no
- claims checked: 8 — holds: 5, stale: 3, unverifiable: 0

### Stale claims

- "In progress": F2.1 and F2.2 landed on 2026-06-09 per Progress. Their host was removed in `c5f01064`.
- The F2.1 steps name `meerkat/src/{command,lib,shellbar,main,render,views,frame_ops}.rs` (l.28-47) with no historical marker; only the Related line marks `crates/meerkat/`. `SHELLBAR_THICKNESS`, `ToggleApparatus` and `shellbar_rect` are absent at the base.
- R2 says `RosterRow` gains `edges: Vec<EdgeRow>` (l.86-87). The surviving `RosterRow` (`crates/mere/src/roster.rs:138-147`) has no `edges` field; links are a separate `LinkRow` (:150).

### Contradictions

- none.

### Recommended action

- Mark historical (meerkat removed) and archive.
- Extract the "shellbar outside the frame tree; its edge is a window preference" ruling, and F2.3's pointer to multi-graph MG4, if either still applies to a current host.

### Notes

Holds: `crates/system/pandect/src/settings_store.rs` carries `shellbar_edge`, and `ShellbarEdge` is in `application_settings_store.rs`; R1/R3 roster facets and sections survive in `crates/mere/src/roster.rs` (title, url, content_type, tags, section_header; `content_bucket` :641, `relation_kind_label` :778); the taxonomy doc and the multi-graph activation plan exist.

## mere_docs/implementation_strategy/2026-06-26_federation_interop_plan.md

- disposition: current
- status line: "Status: Scoped, not started. Two federation-interop mechanisms borrowed from the [borrowed-ideas brief](../research/2026-06-25_borrowed_ideas_brief.md), scoped now (ahead of the knot-editor resume) because both are load-bearing for federation and painful to retrofit once peers are exchanging data. Implementation deferred." — accurate: yes
- claims checked: 6 — holds: 3, stale: 3, unverifiable: 0

### Stale claims

- §2's gap, "No federation access-control primitive: nothing says 'peer X may read subgraph S, revocably,' without a server", no longer holds. Other lanes built a form of it:
  - `servitor::cap` and `grant` (`cap.rs:240` `Mode`, ordered Read < Write < Delegate; `grant.rs:26` `Grant` with expiry; `README.md:100-103` leaves room for a later Meadowcap-shaped provider);
  - Gemot's capability-scoped `MootAuthorizationRequest` (`gemot/src/moot/service.rs:190-200`);
  - Commons' encrypted graph profile with group-key epochs and parking (`commons/src/encrypted.rs:7-22`; `af674f30`, `3d3ad81c`).
  No borrow (Meadowcap, UCAN or Keyhive) has been chosen.
- The engram vocabulary and placement (l.16-18, 28-29): engrams became codicils in `c51b9704` (2026-08-31). The schema vocabulary (`SchemaRef`, pointing at a schema codicil) is in `eidetic-core/src/schema.rs`, though no schema registry type exists, and "alembic" now names Distillery's recall component (`ports/distillery/alembic/README.md`).
- "dovetails with Tessera as the trust receipt" (l.69): Tessera is now Standing (`gemot/standing/v1`, code in `crates/moot/mien`).

### Contradictions

- none.

### Recommended action

- Revise §2 to say what servitor, Gemot and Commons now provide, and narrow §2 to the remaining borrow choice, or rule it superseded (see Forks).
- Rename engram → codicil and Tessera → Standing, and repoint the schema-registry home.

### Notes

The lens half is genuinely unstarted: `git grep -i "cambria\|SchemaLens\|lens_chain"` finds nothing relevant. `crates/system/registry/src/lens.rs`'s `LensRegistry` is a graph-presentation preset registry, unrelated to schema lenses. All eight cross-referenced docs exist at the base.
