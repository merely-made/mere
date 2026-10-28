# Workspace hygiene: releases, dependency currency, portable graph, licenses

**Status:** 2026-10-28: cut from `2026-09-23_crate_consolidation_plan.md`, `2026-10-08_dependency_currency_plan.md`, `2026-09-16_lattice_sync_pass_plan.md`, `2026-08-22_license_sweep_plan.md`, `2026-08-10_crypto_generation_unification_plan.md` (complete; residue only) and the open cues of `2026-10-04_stack_seams_plan.md`, all archived in `archive_docs/2026-10-28/`. Landed: crate consolidation C1 to C4 and C2a, insigne phases A to D and chatelaine P1 to P3 (so C5's conditions are met); the crypto unification; netrender's dependency lane and the renderer cohort; lattice L1, L2 passes A and B, the 2026-09-23 consumer round, the portable workflow and a two-row unused-patch baseline; license P0, P1, P2, P4 (turnstone, hocket), P5, P6's clause, P7 except the retinue household, and the mere header gate. Open: the version baseline, the registry deletions and the dramatis facade; genet's held `dep-currency` branch and four mere dependency lanes; lattice L3, L5, L6, L7, cleanup, CI confirmation and the unused-patch causes; license P3 Isometry, P4 Woodshed and P7 retinue; and three stack-seams cues (F18/F20 measurements, the Eidetic critical pass, the render ladder rethink).

**For:** keeping the family's repositories buildable from a clean checkout and consistent in releases, versions, registry names, dependency currency and license headers. Opened by Mark's 2026-09-23 audit of the publishable crates (reserved crates receive their capabilities; components are modules), the 2026-09-16 lattice rulings, the 2026-10-08 refresh scope (mere, genet and netrender, the rest by propagation) and the 2026-08-22 license ruling (MPL-2.0 by default with correct provenance).
Not here: the mere-apparatus row of C2 (exact presented frame, worker qualification, human AT), which is `2026-10-28_diagnostics_and_accessibility_plan.md`; CXF import (chatelaine plan P4 onward), which is the chatelaine and CXF plan in the dramatis repository; the Burn cohort's residue, carried vendor patches and the ESP package receipt, which are `2026-10-28_distillery_models_plan.md`, and S63's consolidated ML plan, which that plan is; dynamics determinism and permitted actions (S3 to S6) and C3's libm figure at 5,000 bodies, which are `2026-10-28_dynamics_grammar_plan.md`; the lexical capability plan's L1, L3, L5 and Turnstone L4 (S42), which stay in `2026-09-16_lexical_capability_plan.md`; the MPL section 3.2 executable-form notice (one line in an About panel or installer), which is the packaging lane in `2026-10-28_family_update_resident_plan.md`; the Linebender text family and glyph-facing types, which are netrender's Vello lane (`netrender-notes/2026-10-08_vello_upstream_lane.md`); the 441 audit findings, which are routed by `2026-10-28_source_audit_repairs_plan.md`; the Linux and macOS headed runs of other plans' gates, which are `2026-10-28_platform_qualification_plan.md` (F18 and F20, phase X1, stay here).

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

Sources are cited by short name with line numbers into the archived copies: consolidation, currency, lattice, sweep, crypto, seams, tails.

**Releases and registry**
- Reserved crates exist to receive their capabilities; capability that grew elsewhere moves into its named home and no stub is left beside the kludge. A component is a module, folded into its parent crate; no new crate or repository for a component. The `sibylla` and `vates` shims and names are dead (Mark, 2026-09-23; consolidation 16-32).
- The version baseline is set once, on the consolidated crate set, then workspace versions run in lockstep from the first release (consolidation 30-32, 219-231). chatelaine publishes first because castellan depends on it (consolidation 423-424). Genet's forks (`genet-taffy` 0.14.0, `genet-parley` and `genet-fontique` 0.10.0) keep their upstream-derived numbers (consolidation 222-226).
- Registry deletions are done on the site by Mark and feed the Code-root crate inventory (consolidation 238-241).

**Portable graph (lattice)**
- Redirects stay; only woodshed's dead entries go. The seven redirect repos (mere, turnstone, genet, hocket, cleromancy, mer3ly, woodshed) commit a clean lock and keep a local lock under an ignored `.cargo/local/`; `scripts/cargo_mode.py local` supplies it and portable mode refuses a config carrying `resolver.lockfile-path` (lattice 18-42, 100-120).
- Cycle rule: "up to date" means at head less the pass's own pin-only commits; a pin-only change triggers no reciprocal repin; incompatible duplicate type identities stop the step, and a structural fix is a separate boundary change (lattice 26-27, 115, 515-520). A shared-interface crate to break repository cycles waits until a concrete type or ownership conflict names the boundary (lattice 173-178).
- Stop rules (lattice 569-579): a port needing a design choice; a lock committing an outside path package; a step adding an unused-patch row the pass did not inherit; a pin move splitting a crate into two revisions the step cannot unify; a refused push; a live session holding a needed file; Rust 1.98.1 changing a result no pin move explains (rerun on 1.97.1 and report).
- The host gate is `cargo check --workspace --all-targets --locked`; `cargo check` proves nothing about feature-gated or wasm code (lattice 73-77, 350-351). Porting stops and returns to Mark when it needs a design choice (lattice 503-509).
- Not this plan's: `mark-ik/*` fork rev pins (vello, boa) are Mark's to move (lattice 321-322); `crates/cargo-gpu`'s unpushed commit is local by design (lattice 319-320); duplicate global `safe.directory` entries are Mark's (lattice 467-468); build-output cleanup belongs to the disk-cleanup task (lattice 40-42).

**Dependency currency**
- Scope is mere, genet and netrender; every other repo changes by propagation from those two (D1, currency 41). This program's lanes fetch from crates.io only and log every new crate and version in the lock diff (D4, currency 47).
- Order is upstream first: netrender, then genet, then mere's own groups, one mere lane at a time (every mere lane rewrites `Cargo.lock`), mere taking genet and netrender only by repin, netrender first; provider before consumer and never a one-sided repin of the shared Netrender family; at most three lanes at once (D6, D8, currency 51, 55, 85-90).
- Linebender text (parley, parley_data, skrifa, read-fonts, harfrust) and glyph-facing types belong to netrender's Vello lane; taffy stays pinned at `0.11.0-experimental-cache-fix.3` (D8, D17, currency 37, 55, 73).
- Lane done-condition: every crate of the family at its current release, or the release the family agrees on, with the lock diff logged; the F138 gate (archived dynamics grammar plan) passes, meaning crates whose inputs moved are compiled and tested, receipts measuring what moved are rerun and any number that moves stops the lane for Mark; a family with a stored format (fjall) or a measured output (tokenizers, text shaping) comes to Mark as a fork before it lands (currency 94-96).
- Genet's stage: cadency keeps its error API with approximated location (D14); the nesting limit stays 75 and a WPT hit returns to Mark (D15); every digest 0.10 user in genet moves to the 0.11 line, and one that cannot move returns as a fork (D16); sha2 0.11 hex uses inline formatting with no new dependency (D18); the branch holds until the forms session commits, then merges once (D20); mere's accessibility family moves in one lane after genet lands (D21) (currency 59-81).

**License (sweep)**
- Everything Merely owns is MPL-2.0; third-party code keeps its provenance, and a file takes Exhibit A only if Mark wrote it. Exhibit A, never Exhibit B. House header is shape C with the notice `Mark Alan Boykin`; Servo-derived files keep Servo's bare Exhibit A (sweep 50-95).
- One `LICENSE` per repository; published crates ship SPDX plus Exhibit A and no license text file, because Cargo does not copy a root `LICENSE` into a member package (sweep 4-33, 90-95). Manifests say `MPL-2.0`. No functional change, no republish (a crate carries the new license at its next functional bump), never sweep a dirty tree (sweep 100-112).
- In already-MPL repositories P7 changes headers only, never a manifest, a `LICENSE` or a published version, and never before that repository's ledger is written (sweep 336-351).
- After `--apply`, diff every swept file against `HEAD` with header lines filtered out; a non-empty remainder is another lane's change and the file is held out (sweep 508). `relicense_headers.py --check` gates new owned sources in CI and the doc audit; the 2026-09-08 probe receipts are Frozen evidence (S79) and the two live sources named in receipt hash maps were swept (S82) (sweep 26-33; seams 848-862).

**Crypto and renames**
- First-party manifests sit on the sha2 and digest 0.11 row; a third-party transitive dependency on the 0.10 row is tolerable; moving genet's row moves the whole row. An orphan declaration is removed, not repinned (C1); rulings on this are recorded in the crypto plan, not as new S-numbers (C2) (crypto 60-144).
- Renames from S5, S6 and S8 (forme for the workbench arrangement, a world spec for seiche's `SceneSpec`, pandect's `ViewIntent` becoming the curation record) land in code when a file is touched, with persisted tags and file names still read; no churn pass (seams 324-355, 946-949). S10: pandect is part of Eidetic, the crates stay where they are (seams 362-368). S43 keeps this plan the owner of the lattice items (seams 702-708).

## Phases

### R1. The version baseline (C5)
Set genet 0.6 and Mere 0.4 once, on the consolidated crate set, with lockstep `version.workspace` from the first release. C5's conditions are met (insigne phases A to D, chatelaine P1 to P3 by 2026-10-02); the baseline is not set. The tails add one unchecked step: chatelaine's 0.0.2 republish could not be checked offline.
- Done when: genet and Mere are at their baseline numbers, workspace versions follow in lockstep from the first release, and chatelaine's 0.0.2 republish is checked.
- Needs: -
- Rulings: Mark 2026-09-23 (baseline), 2026-09-26 ("after B", met)
- From: `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:4-14,219-231,419-424`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1167-1169`

### R2. Registry names (C6)
Ten crates.io names have no crate behind them: `sibylla`, `vates`, `mere-capability`, `titulus`, `mere-resident`, `graphshell-stdio`, `graphshell-network`, `mere-canvas`, `mere-signals` and `cambium-nematic` (0.3.1). Mark deletes them on the site. Also open: the crates.io `eidetic` description reword (the last bullet of the family repo merges' P4).
- Done when: the ten names are deleted and the Code-root crate inventory shows it; the `eidetic` description is reworded.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:11-12,232-241`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:660`

### R3. The dramatis facade
A real facade crate re-exporting personae, insigne and gaz for repositories outside mere; ruled 2026-10-01, unbuilt, and not a condition of C5. The tails name the dramatis repository plan, whose ruling D1 makes dramatis the product, as its owner.
- Done when: the dramatis row is landed or ruled out, which is what C2's done-condition asks (every row landed, ruled out by Mark, or found to have nothing misplaced, each landed move carrying its tests); the apparatus row closes elsewhere.
- Needs: the dramatis repository plan's D1
- Rulings: Mark 2026-10-01
- From: `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:11-13,57,60-61,408-411`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1165-1166`

### D1. Genet's dependency stage lands
Genet's branch `dep-currency` carries accesskit 0.25, pollster, tokio-tungstenite, html5ever 0.40, cssparser 0.38, sha2 0.11 and the digest 0.11 line, unit-gated and held because genet's main gained the forms constraint-validation work, which adds `.local` and `.ns.as_ref()` sites html5ever 0.40 breaks. It waits for the forms session to commit, merges once converting every site, and returns for its push. The WPT after-pass (17 subsets, about 80 minutes at `--jobs 4`) runs once, combined, after cssparser, with moved files rerun on both runners; its result is not recorded as final. Landing the branch also moves genet's workspace sha2 row (fleece, genet-scripted), the crypto residue's genet item.
- Done when: the branch is merged with every site converted, unit-gated and returned for its push, and the combined WPT after-pass has a recorded result (a hit on the 75-block nesting limit returns to Mark).
- Needs: the genet forms session's commit; G2c's push before the after-pass
- Rulings: D10, D13 to D20
- From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:65,69,71,75,77,79,83`; `archive_docs/2026-10-28/2026-08-10_crypto_generation_unification_plan.md:98-103`

### D2. Mere's accessibility lane
One mere lane after genet lands: accesskit 0.25 and its adapters (accesskit_winit 0.34, accesskit_macos 0.27, accesskit_unix 0.24, accesskit_windows 0.35), accesskit_consumer to 0.39 with the exact pins in `cambium-winit-a11y` and `uxtree`, and mere's 29 `.local` and `.ns.as_ref()` sites in 11 files that html5ever 0.40 breaks, in the same commit as the genet repin because `genet-render::accesskit_tree` returns AccessKit 0.25 types.
- Done when: the lane done-condition holds (Constraints).
- Needs: D1
- Rulings: D3, D6, D21
- From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:29-30,45,81,83`

### D3. Mere's security-sensitive lane
argon2 0.5.3 to 0.6.0 (personae); minisign 0.7.9 to 0.10.0 and minisign-verify 0.2.5 to 0.3.0 (luggage); wasmtime and wasmtime-wasi 45 to 49 (app-host, document-host); zip 0.6.6 and 2.4.2 to 8.6.0 (muniment). sha2 for netrender is done. The argon2 move is a stored-format question (see Open questions).
- Done when: the lane done-condition holds.
- Needs: D1 (each mere lane takes the genet and netrender repins as they land)
- Rulings: D3, D6
- From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:22-27,45,51`

### D4. Mere's data and Meaning lane
fjall 2.11 to 3.1 (eidetic; the on-disk format changes), jsonschema 0.18 to 0.58 (eidetic), infer 0.19 to 0.22 (mere-kernel, mere-registry), tokenizers 0.20 to 0.23 (esp; can shift the pinned e5-base-v2 embeddings and P7's receipt).
- Done when: the lane done-condition holds; fjall and tokenizers come to Mark as a fork before landing.
- Needs: D1
- Rulings: D3, D6
- From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:31-35,96`

### D5. Mere's other lane
str0m 0.23 to 0.24, x11rb 0.13 to 0.14, dirs 5 and 6 to 7, pollster 0.4 to 1.0, tokio-tungstenite 0.29 to 0.30, windows-core 0.62 to 0.100.
- Done when: the lane done-condition holds.
- Needs: D1
- Rulings: D3, D6
- From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:36,45`

### D6. The rapier and parry lane, with mere's compatible updates
rapier2d and rapier3d 0.33 to 0.36 and parry 0.28 to 0.31.1 (seiche, conatus) on branch `rapier-036`, with mere's ten compatible updates folded into its lock change: encoding_rs, inventory, iroh-blobs, libc, scrying, thiserror, tokio, tokio-rustls, uuid and zeroize. The plan last says "moving"; its landing is not recorded.
- Done when: the lane done-condition holds, receipts that measure what moved are rerun and any number that moves stops the lane for Mark (F166 of the archived dynamics grammar plan).
- Needs: the dynamics grammar plan's F138 gate rule
- Rulings: D2
- From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:21,43,98-101`

### D7. Crypto residue
Items the crypto plan chose not to chase: `misfin` 0.0.4 pulls sha2 0.10 through its own published manifest (a smolweb-workspace commit and republish, held in stewardship); `ed25519-dalek` sits at 2.2.0 and 3.0.0 through p2panda's fork line and misfin (the archived 2026-07-15 iroh/p2panda bump plan records how much unifying is); genet's own sha2 row, which D1 moves. `argon2` stays 0.5.3 until 0.6 is stable (phase D3 holds the question), and `signature` 2 rides with `ssh-key` 0.6 rather than moving alone. Third-party transitive digest 0.10 consumers (p2panda-encryption, snow, ssh-key, sqlx, wasmtime, rsa, p256/384/521, elliptic-curve, bcrypt-pbkdf, oxrdf) are tolerated.
- Done when: no first-party manifest pins the 0.10 row and each remaining 0.10 consumer is third-party transitive (the decision doc's rule).
- Needs: D1 for genet's row
- Rulings: C1, C2
- From: `archive_docs/2026-10-28/2026-08-10_crypto_generation_unification_plan.md:60-106`

### L1. The remaining pin levels
Open levels of the lattice pass. L3: retinue, genet to head and mere to L2's head, porting across 79 and 221 commits. L5: mere pass B, pin-only, knot-editor unified to L4 (it is still split between the root and `ports/djinn`, which waits for its session to go idle) and retinue unified to L3. L6: woodshed (mere at L5, genet, netrender; unify three and three revisions), cleromancy (mere at L5, genet, turquet), mer3ly (mere at L5). L7 beyond turnstone: hocket (mere at L5, genet, netrender; its push waits on a broken origin) and isometry (cleromancy at L6, mere at L5 unified from three revisions, genet, netrender; after its live session goes idle). Turnstone, L1, L2 and the knot-editor consumer round are done.
- Done when: lattice done-conditions 1 to 6 hold for every inventory row (every pin at its source's head less the pass's own pin-only commits; supported workspaces on the recorded toolchain; each redirect repo's committed lock free of outside path packages; locked checks pass redirect-free with no new unused-patch row; completed migrations published; local resolution leaves committed locks byte-identical). Blocked rows keep their failure reason and prevent claiming the pass complete; independent rows may still land.
- Needs: L3 before L5's retinue unify; hocket's origin (Open questions)
- Rulings: lattice rulings of 2026-09-16, S43
- From: `archive_docs/2026-10-28/2026-09-16_lattice_sync_pass_plan.md:5-16,48-82,519-547`

### L2. Cleanup (P2)
Cleanup runs independently and must not delay the pins. Open items with no completion line: the `burn-pre3-repin` branch is still present and the tag `archive/burn-pre3-repin-20260916` is absent (the dirty diff and untracked files are saved with a restore recipe first); the Renderling branch `mark-ik/wgpu-30-comment-wrap`; woodshed's 21 dead config entries; worktree pruning in knot-editor, mere, retinue, turnstone and prns and the `smolweb-next-20260913` branches; the unlinked folders in `Code/worktrees`; Code-root leftovers; toolchain lint (`rustup override unset --nonexistent`, then uninstall 1.90.0 and 1.93.0 only after checking pins, CI, overrides and processes). The shelved mark on the Burn plan's pre.3 section is moot: that plan is archived whole and already calls its pre.3 repin superseded.
- Done when: the tag is listed on origin and the branch and worktree are gone; the Renderling branch exists on origin with a clean tree; no woodshed entry points at a missing path; no prunable records and the branches are gone; eligible folders and delete-list entries are gone with changed or active candidates listed as deferred and the kept list untouched; `rustup override list` shows no missing directory and neither toolchain is listed.
- Needs: -
- Rulings: lattice cleanup ruling of 2026-09-16
- From: `archive_docs/2026-10-28/2026-09-16_lattice_sync_pass_plan.md:14-16,390-468`; `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:176-178`

### L3. A fresh checkout builds
The portable workflow `.github/workflows/portable.yml` is published but had never passed: all 164 runs failed at the first gate step (a temp-path comparison on the Windows runner), repaired 2026-10-06, so the portable verify and the three WASI guest builds first run on the next push and whether they pass is unknown. Also open: mere's nested guest, wasm, port, vendored and fixture workspaces have no portable build gates (three WASI guests pass), and other nested ports and peripheral repositories are unqualified; the standalone roots `crates/cambium/examples/genet_web_smoke` and `ports/graphshell/web` stayed on the older genet pin, and the smoke root needed a moving genet refresh with no full renderer-prototype adoption receipt; three nested guest toolchain pins still name 1.93.0 and should align, tracked locks and locked build-script child commands included; Hocket, Woodshed's old Hocket port and Retinue's desktop app still request `genet-probe` and `parley` (now `genet-parley`) with no build receipt or dormancy designation; Hocket's toolchain pin is still 1.96.0.
- Done when: a clean clone needs only documented prerequisites, the optional local setup selects sibling sources and preserves the portable locks, both commands are recorded in tracked developer instructions and exercised once, and the workflow's later steps pass on CI.
- Needs: L1 for Hocket's toolchain step
- Rulings: lattice 2026-09-20 execution amendment
- From: `archive_docs/2026-10-28/2026-09-16_lattice_sync_pass_plan.md:28-31,168-172,179-183,549-559,606-607,631-640,672-674,693-709`; `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:541-549`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:865`

### L4. Unused-patch causes (P4)
Diagnose each surviving unused-patch row read-only (a version mismatch, a ref split, a dropped dependency, or a patch keyed to the wrong source), report to Mark and change nothing. Mere's baseline is two rows, `boa_engine` and `boa_gc` (their patches target crates.io while the graph selects direct Git pins); turnstone's `p2panda-stream` patch has no selected package.
- Done when: each row has a stated cause in Findings.
- Needs: L1
- Rulings: -
- From: `archive_docs/2026-10-28/2026-09-16_lattice_sync_pass_plan.md:11-12,142-144,561-567,643-644`

### S1. Isometry headers (license P3)
Workspace license line, two explicit lines, a single `LICENSE`, README license section, headers on about 123 owned sources and a `LICENSES.md`. The edition stays 2021; the migration is its own task. At the S14 audit the root was still `MIT OR Apache-2.0`.
- Done when: the P1 greps hold, `cargo check` on the workspace default members is green and `--audit` reports zero.
- Needs: the Isometry tree (see Open questions)
- Rulings: sweep invariants 1 to 10
- From: `archive_docs/2026-10-28/2026-08-22_license_sweep_plan.md:4-33,230-237`

### S2. Woodshed headers (license P4)
Headers on 74 or more sources (154 of 188 Rust files lack them), README license section, a single `LICENSE`; `woodshed-web` is already MPL and stays. The gate is a clean tree (47 files were in flight on 2026-08-22).
- Done when: the P1 greps hold, a sample `cargo check` is green and `--audit` reports zero.
- Needs: a clean Woodshed tree
- Rulings: sweep invariant 7
- From: `archive_docs/2026-10-28/2026-08-22_license_sweep_plan.md:4-14,239-246`

### S3. Retinue household headers (license P7)
Retinue, tulle, sennet and tucket carry no per-file notice (383 of 388 files bare in retinue); shape C headers only. `vendor/lora-phy` is excluded and tucket's MeshCore `NOTICE` is untouched; each repository's ledger precedes the tool.
- Done when: `git grep -L 'Mozilla Public' -- '*.rs'` returns only ledger paths, every owned file carries the copyright line, a sample `cargo check` is green and `--audit` reports zero unheaded owned sources.
- Needs: the retinue lane's tree
- Rulings: P7 ruling of 2026-08-22
- From: `archive_docs/2026-10-28/2026-08-22_license_sweep_plan.md:294-326,339-351,361-363`

### S4. License loose ends
Items the sweep left with no recorded outcome: the tool should refuse a dirty tree itself; genet's `--audit` exceeds two minutes and needs a longer budget or an index-based check; the tool's extension map lacks `.scn`, `.lua` and `.knot`; hocket's `cargo check` could not run because its gitignored config points at a deleted Codex worktree; the wavicle founding plan needs a dated note on the July convention; mesocosm keeps four license texts (the single-`LICENSE` rename waited on the promoted-library clause, retired 2026-09-03) and the same ruling made `paredros-identity` MPL-2.0 without a recorded manifest and header change; Knot's `LICENSES.md` is mere's 2026-08-27 ledger carried over by the extraction and names paths Knot does not have; mere's `LICENSES.md` still lists the inker and verso-tile notice pairs and the "not to be deleted" sentence that S40 amends; the posture brief's section 5 table is marked per phase; genet and netrender's cruft pass, asked 2026-09-03, has no recorded outcome; the 658 missing Exhibit A headers that moved with the promoted family trees were not repaired; gaz has no repository, and if it is to exist again that is a founding claimed by a real publish.
- Done when: each item has its outcome recorded in the repository concerned (the sources give no joint threshold).
- Needs: -
- Rulings: S40, 2026-09-03 promoted-library ruling
- From: `archive_docs/2026-10-28/2026-08-22_license_sweep_plan.md:262,273-292,380,490,496-500,505,507`; `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:5006-5007`; `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:161-165`

### X1. Multi-window pacing, measured off Windows (F18, F20)
P2 stage 4 serves each window that asked in one idle turn, drawing from the idle turn on Windows only. Open and unmeasured: two animating windows on X11, Wayland and macOS (F18); mixed-refresh monitors coupling windows to the slower (a 60 Hz and a 144 Hz window both drawing at 60); whether Win32's modal move and size loops still run the idle turn; whether an occluded but not minimized window's present blocks; and that a panic inside a turn leaves the lent runner in that window.
- Done when: the cases are measured on Mark's Linux and macOS machines and reported; S33 keeps F18 and F20 open until then.
- Needs: Mark's Linux and macOS machines
- Rulings: S26 (each window paces itself), S33
- From: `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:230-271,580,591-595,1156-1157`

### X2. The Eidetic critical pass (S59)
S59 archived four plans (Eidetic browsing derivation, redb over OPFS feasibility, the intel vector index burn lift's mere twin, receipt artifacts replication) for one critical pass that becomes a plan if warranted, with Eidetic's development taken up now that identity has a path; the reading is a new objective with its own assessment. Items it holds: E2's done-condition (Mark's run over a real exported history), E2's three questions (per-persona traces, the home of co-occurrence, the owner of the geist's fused query) and E5's remainder (W3's reports and corridor strip, W3 being the search surface wiring plan's); redb over OPFS adoption and crash-atomic creation (Safari and WKWebView not run), section 7.3 items 3, 5, 6 and 7, the Web Locks posture and whether adoption requires `Backend::transact` (the Eidetic development plan's P6 holds this decision as a checkpoint); no crate enables `index-burn`, so routing is open (Path A closed, or P3 open until callers route) and P4 HNSW (Path B) is deferred; receipt artifacts' R1 (a receipt browsable on a second paired device, with section 6.5's remote runs ingested on one machine).
- Done when: the pass has produced a plan, or a recorded finding that none is warranted.
- Needs: -
- Rulings: S59, S70
- From: `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:798,826,1019`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1360-1411`

### K1. Pandect files over the ceiling
`crates/system/pandect/src/graph_codicil.rs` (838 lines) and `manifest_store.rs` (658 lines) exceed the 600-line ceiling.
- Done when: both files are under the ceiling.
- Needs: -
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:772-773`

## Open questions

- **Genet's P2 hazard**: is it closed as moot (cambium has left genet and mere's copy carries `pointer.rs`) or kept open until a clean-checkout genet build is receipted? From: `archive_docs/2026-10-28/2026-08-22_license_sweep_plan.md:508-520`.
- **S44's completion**: the seams ledger gives the three `sha2` 0.10 repins no completion line, while the crypto plan records all three done on 2026-10-06 (djinn and the distillery fixture repinned, Pelt desktop's orphan removed). Confirm against the manifests. From: `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:710-716,1045-1048`; `archive_docs/2026-10-28/2026-08-10_crypto_generation_unification_plan.md:4-10,108-119`.
- **argon2 0.6**: the currency plan lists argon2 0.6.0 as the security lane's target, while the crypto plan holds 0.5.3 until 0.6 is stable and calls a password-hash change a stored-format change wanting its own migration note. Is 0.6 stable now, and does D3 go to Mark as a fork? From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:22-27,96`; `archive_docs/2026-10-28/2026-08-10_crypto_generation_unification_plan.md:104-106`.
- **Toolchain pin**: the lattice done-conditions and stop rules name Rust 1.98.1, while the audit's R3 and the family promotion's later receipts speak of 1.99.0 and 1.99.1. Which toolchain do the done-conditions mean now? From: `archive_docs/2026-10-28/2026-09-16_lattice_sync_pass_plan.md:58-59,113`; `archive_docs/2026-10-28/2026-10-16_full_source_audit_plan.md:67-89`; `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:743-745`.
- **The rapier lane's landing**: the currency plan says only that it is moving. Has `rapier-036` landed, with the ten compatible updates? From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:43,98-101`.
- **D9's note**: was the note to the owner of the workspace upstream refresh sent? There is no record. From: `archive_docs/2026-10-28/2026-10-08_dependency_currency_plan.md:57`.
- **Hocket and ringdown origins**: both fetch "Repository not found", and hocket's push waits on it. What is their origin now? From: `archive_docs/2026-10-28/2026-09-16_lattice_sync_pass_plan.md:318,541,576`.
- **Genet's forks' republish rule**: C5 lists the forks "with their own republish rule" that the plan does not state. Where is it? From: `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:222-226`.
- **Doc-audit baseline debt**: the family promotion plan reports the document-judgment check failing on 19 missing records and a snapshot digest constant, the source-edit plan 18 missing records and the same digest, and Hest makes no whole-tree doc-clean claim, while the seams S14 pass left both audits at exit 0 on 2026-10-06. Does the check still fail, and who owns the baseline? From: `archive_docs/2026-10-28/2026-10-19_family_promotion_plan.md:189-192`; `archive_docs/2026-10-28/2026-10-18_source_edit_proposal_plan.md:129-132`; `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:1013-1020`.
- **Which four `graphshell-*` crates are unpublished**: the tails name four, gated on the remote projection host's G5 to G7 (G6 and G7 received by the reference host plan), but C4 folded `graphshell-stdio`, `-local` and `-network` into `graphshell-endpoint`. What remains? From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:866-868`; `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:204-215`.
- **The seven donor GitHub repositories**: is archiving them still a task now that their crates live in Mere, or done or dropped? From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:655-658`.
- **Isometry's tree**: P3 sweeps "isometry" to MPL-2.0, but the isocosm repository now holds the isometry crates and its cut records `isocosm-vtt` at `MIT OR Apache-2.0` by its ruling 919 (`isocosm/design_docs/2026-10-28_campaign_retirement_plan.md`, F1). Which tree and license does P3 mean? From: `archive_docs/2026-10-28/2026-08-22_license_sweep_plan.md:8-10,230-237`.
- **Two atomic-write implementations**: pandect's `write_bytes_with_backup` and tabard's temp-and-rename writer (Pelt's, moved) are recorded, with unification left undone; Knot's settings write also still removes then renames (knot-editor's, no plan named). Unify? From: `archive_docs/2026-10-28/2026-09-23_crate_consolidation_plan.md:152-155`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:729-731`.
- **Does pandect move under `crates/eidetic/`?** S10 left it unruled. From: `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:362-368`.
- **The render ladder rethink**: Mark archived the render ladder with the note that it needs rethinking, and its tails (keyboard dispatch, interactive-region refinement, routing article `main_text` into the Eidetic corpus, a `pump()` before the scripted extract) wait on that rethink, not only on a host. From: `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:776,1019`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:963-973`.
- **F21's flake**: `carrier::tests::p2panda_murm_grant_is_refused_before_projection_bytes` timed out waiting 10 seconds in 2 of 5 runs under load and passed alone; read as a timing flake in a fixed timeout, recorded and not fixed (graphshell's carrier was outside the seams plan). Who fixes it? From: `archive_docs/2026-10-28/2026-10-04_stack_seams_plan.md:272-280`.
- **Forme's dead submodules**: `subgraph`, `lens`, `parity`, `pressure` and `reconciliation` remain in `crates/forme/forme/src/`; deleting them "wants a yes". From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:269-271`.
- **Orphans needing a code ruling**: `PersonaSettings.menu_actions` and `command_usage` (a round-trip test, no consumers) are reconsumed or dropped; `ToastSpec`, `ToastSeverity` and `FrameViewModel.toasts` have no producer or consumer. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:281-284,330-332`.
- **`apply.rs`**: 1,938 lines; is the split still wanted, or is it closed with the file whole? From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1060-1061`.
- **`mk-harness.ps1` launches `meerkat.exe`**: retire it beside the other retired drivers in `Code/testing/_archive/scripts/`, or repoint it at a current host's scenario runner (a question of substrate, since meerkat is gone). From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:832-835`.
- **Genet's `docs/` tree**: merging it into its doc tree was never scheduled. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1354-1355`.

## Findings

- 2026-10-07: the first build at each new genet revision stalls every cargo command on the machine for about fifteen minutes, because Cargo holds the package-cache lock while it checks out a git dependency; it is not a deadlock between an outer build and its nested guest builds. Each new test-binary path also raises a Windows Defender Firewall prompt that Mark clears by hand, a separate recurring cost (lattice 711-766).
- 2026-10-05: wgpu 30 compares a `Device` by a per-instance id and each `RenderCore::boot` makes its own instance, so devices from two boots compare equal. `cambium-rootstock/src/producer/registry.rs` (`prepare`) detects a changed device with `!=`; recovery from a lost device would need it to compare identity (F16, seams 213-222).
- 2026-10-05: on Windows winit delivers a frame on `WM_PAINT` and Win32 hands `WM_PAINT` to the first window needing paint, so an animating window starves another. Drawing from the idle turn is a Windows-only reading, not ruled; doing it everywhere would bypass Wayland's frame callbacks, under which a hidden surface's `Fifo` acquire can block (F18, seams 230-262).
- 2026-09-25: the portable gate checks all targets; without `--all-targets` it never built test or example code (lattice 679-685).
- 2026-10-06: Mere's unused-patch baseline is two rows, `boa_engine` and `boa_gc` (`Cargo.toml:734-735` then) (lattice 11-12, 142-144).
- 2026-08-27: Cargo does not copy a workspace-root `LICENSE` into a member package (verified on retinue), which is why published crates ship SPDX and Exhibit A with no license text file (sweep 382-410).

## Progress

- 2026-10-28: cut from the crate consolidation, dependency currency, lattice sync, license sweep and crypto generation plans and the open cues of the stack seams plan; nothing new landed.

## Received from the backlog holder, 2026-10-28

Items the archived-plan-tails plan held with no owner, given this one when the active plans were cut; each is as the holder recorded it, not re-verified.

- The substrate parallelism composition brief (`2026-06-21_substrate_parallelism_composition_brief.md`) still presents the parallel-cascade thesis as live although Stylo's retirement removed its mechanism; it needs a note. A new measurement would be a genet plan. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:294-300`
- Mooting adopting `eidetic-fjall` (a manifest change); meerkat's suite check belongs to whichever crate inherited the code. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:442-448`
- Splitting pandect's mixed concerns (the former session-runtime); only the settings slice is tracked and the rest is unowned. The reservoir's V2 journal may bear on it. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:452-460`
- Which seven consumer commits the terminology-and-crate-folds status means. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:801-803`
- The 2026-09-09 games-wing renderer adoption follow-up; Vello V1 to V4 are Netrender's upstream ask; eleven unverifiable claims against the 2026-09-06 remediation receipt. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:817-820`

## Backlog items declined on 2026-10-28

When the archived-plan-tails plan was emptied into the new set, these items were given no owner here: done, gone with their host, or another repository's. Each line says which.

- `favicon_data_uri` misnomer: the code went with meerkat. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:69`
- The S14 question on how to carry the four meerkat-subject tails: answered by this cut, which archives the holder whole; they are history. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:77-81`
- Document-lane find closed 2026-07-03; its cluster-map follow-on is the cambium presentation plan's (tails 185-190). From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:109-114`
- Paste into the find field: the code went with meerkat. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:115-120`
- Context-submenu tails: the code went with meerkat. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:124-133`
- `SecondaryForward` rename: the subject is gone. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:175-181`
- No gnode-pool debt; the residual is genet's shell paint emission raster plan's. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:197-202`
- No tails. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:204-213`
- The toolbar-clipping check against `workbench_tiling.rs` is Turnstone's. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:248-249`
- Nested scroll-into-view and the scrollbar thumb are genet's design docs'. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:311-313`
- irc_mod left no tail. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:386-387`
- Four genet seams with no caller are genet's to re-read against Turnstone. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:468-474`
- Hagiograph promotion and retelling: the design record is Isometry's isoscape family plan. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:506-510`
- Coop lifecycle items are carried by the suite census I3 and by Turnstone. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:526-536`
- Knot pin rule, Turnstone receipt of Livery clips and Knot search S0/S1 are knot-editor's copies. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:698-723`
- Splitting `keeper` is the dramatis repo plan's, which reads D14/D15 as making it unnecessary. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:779-785`
- The `mere-embed` leverage-census row was done in the 2026-10-06 pass. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:938-941`
- The remaining sibling repins are the dramatis repo plan's. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1129-1131`
- F0 is knot-editor's application workspace plan's; the pointer-capture gap is Turnstone's T lane. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1227-1230`
- Knot's Phase B spec and `knot-protocol` crate are knot-editor's copy. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1288-1294`
- Per-interface event subclasses and native `currentTarget` are genet's. From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1332-1336`
