# Distillery models: local models, the mesh host and compute-stack upkeep

**Status:** 2026-10-28: cut from `2026-10-06_distillery_models_plan.md` and the residue of `2026-08-09_burn_0_22_migration_plan.md`, archived in `archive_docs/2026-10-28/`. Stable results are recorded for M1's three receipts and the Burn Fusion remote MiniLM panic is gone, but M1 surfaced a remote-autotune owner-reclaim failure that still needs an owner (it is carried here). M2 is closed through resident inference I5. M3, M4 and M5 are open. The Burn migration is complete: stable Burn 0.22.1, CubeCL 0.11.1 and Cubek 0.3.1 are on Mere main with ESP; only its open list and deferred items are carried, as the compute-stack upkeep section below.

**For:** what remains of Mere's local model lane after the four plans archived on 2026-10-06 (distillery v0, inference provider, browser model ceiling probe, mesh host lanes), consolidated under Mark's stack seams rulings S58 ("These four need to be consolidated now that 0.22 is out") and S63: the inference surface's product half, Distillery's deferred operational pieces, the browser model ceiling's unmeasured upper rows, the mesh host's deferred execution guarantees; and, after the migration, keeping the Burn, CubeCL and Cubek cohort healthy.
Not here: the inference host and recall, `2026-10-28_resident_inference_plan.md` (M2 is closed there; the installed Djinn binary and physical second-device qualification for M2 are its IP phase); retained encounter and capture evidence, embedding compatibility and durable model-run records, owned by the Eidetic development plan P4 (Eidetic repository; one forcing host shares its records rather than a second inference-host implementation opening); Servitor, which stays in Eidetic, and the Alembic design drafts, `2026-10-28_alembic_implementation_plan.md`; the reveal finding (a selected first card stays partly scrolled off after Explain), reported to the projection grammar adoption owner, `2026-10-28_projection_grammar_plan.md`; Knot's Windows findings and Isometry's own repin, which belong to the Knot and Isocosm repositories.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

- **Ownership.** This plan owns no Burn migration beyond upkeep and no training research beyond what a phase below names. One forcing host is chosen and Eidetic P4's records are shared rather than a second inference-host implementation being opened. From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:18-31`.
- **Migration rules stay for any Burn move.** Preserve model math, public ESP contracts, feature names, default dependency floor and host ownership; keep CPU correctness anchors while WGPU APIs move; re-run native, wasm and real-device receipts and treat earlier numbers as historical; never enable Burn Remote default features as a side effect (they include client, server, iroh and websocket); never expose a Burn type from another Mere crate merely to make a test compile; never publish ESP or a compatibility shim from a prerelease migration. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:354-366`.
- **A vendor patch is a temporary compatibility fact.** Any required source fork or patch names its upstream commit, reason, removal condition and licence, and is not a new Mere-owned backend. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:368-370`.
- **Stop conditions.** Stop if model output or parity changes without an explained upstream numerical change, if default ESP pulls Burn or remote dependencies, or if the work requires ESP to own rendering or device policy. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:532-543`.
- **Burn Remote stays a separate lane.** It is separately versioned and verified as a resource adapter, and its four entrance conditions are satisfied on the chosen row; remote Fusion and autotune stay out of production defaults. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:519-531,572-583`.
- **No release without authorization.** An ESP semver release needs explicit authorization; no registry release was made for the 0.22.1 cohort, and no upstream issue, pull request or other communication has been authorized for the carried patches. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:495,1903-1906,5217-5222`.
- **A refreshed performance claim names its version and hardware.** From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:545-553`.

## Phases

### RA. Remote autotune owner reclaim
Remote autotune-only and fusion-with-autotune pass the first remote execution and the native control, then fail owner reclaim ("reclaim did not close the lease-bound session"): autotune over Burn Remote does not honour the lease-bound close. Fusion alone is cleared. The failure has no owner; it stays out of production defaults.
- Done when: not stated; needs an owner and a fix or a recorded reason.
- Needs: an owner named in this plan.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:5-6,205-210`; `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:572-575,583`

### M3. Distillery operations
Model manifest browsing and the streaming console as a Distillery surface; multi-device delivery by adopting `TransportCourier::for_mesh` where a `HostConfig` spans devices (every `HostConfig` keeps `NoCourier` today).
- Done when: a manifest can be browsed and a run's output streamed in the surface; a two-device run delivers its inputs through the courier, receipted.
- Needs: -
- Rulings: S58, S63
- From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:48-52,105-112`

### M4. Execution guarantees
Tolerant verification comparators (`registry::verify_output` returns `NotCheckable` for non-exact outputs), measured on the first non-exact remote resource; portable checkpoints carried by the blob lane so a job can resume elsewhere; physical GPU-allocation telemetry; a stacked-adapter row; and larger capability bounds from the ESP consolidation's D2 tail. Process and driver VRAM telemetry is not claimed for the Burn cohort either.
- Done when: a non-exact resource verifies with a measured tolerance and its measurement recorded; a checkpoint written on one device resumes on another; the telemetry reports allocation where the platform exposes it.
- Needs: the blob lane; a first non-exact remote resource.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:48-52,114-122`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:739-745`

### M5. The browser's upper rows
Larger decoder and embedding rows, run only when a consumer names the model it needs; the persistent-storage posture (storage was denied as `best_effort` in the stable run). First runs on the stable decoder row are slower than on the prerelease and no gate covers first-run latency. Physical GPU-allocation release is unobservable through current browser APIs.
- Done when: a consumer-named row passes or fails with its boundary stated; the storage posture is ruled.
- Needs: a consumer naming a model.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:124-130,228-229`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:624-626,739-745`

### DL. Deferred lanes and trainer follow-ons
The `endpoint` backend (external OpenAI-compatible endpoints, lane 3); D1's render-versus-compute device policy, open "until the resident-data consumer forces it"; the communal lanes (geist, shared adapter engrams, group-scale compute), entrance-gated behind the local trainer receipt and portable checkpoints (the geist models and communal compute tiers briefs hold their design); the padding-with-mask follow-on of the autodiff LoRA trainer; and the question what the Burn `LoraAdapter` is, which this plan's parents never named.
- Done when: not stated; owed or declined.
- Needs: portable checkpoints (M4) and a local trainer receipt for the communal lanes.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:750-759,809-811`; `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:18-23`

### CU1. Compute-stack upkeep: Windows, Linux and headed acceptance
The stable cohort was qualified on macOS. Windows and Linux builds, the complete headed product acceptance matrix, process and driver VRAM telemetry, and the ignored external-model, performance and diagnostic rows are not claimed. The remote MiniLM receipt ran both endpoints on one Mac, not two physical machines. Knot-side Windows findings noted and not ruled: CRLF checkouts break seven `wordnet_import` tests, the default title-bar chrome hides toolbar buttons Knot's headed scenarios press, `source_beside_preview` crashes in genet-livery `atomic_basis.rs:288`, and Knot's `LICENSES.md` names paths Knot lacks.
- Done when: not stated; the Windows, Linux and headed matrix passes or states its boundary.
- Needs: Windows and Linux machines; two physical machines for the remote row.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:4995-5007,5093-5101,5214,5217-5222`

### CU2. Compute-stack upkeep: the ESP package receipt
The done condition "ESP packages from a clean commit" is satisfied by the 2026-08-20 `cargo package -p esp` run on a prerelease, and that run was to be repeated after the stable repin because dependency packaging is part of the release gate; no repeat is recorded after 0.22.
- Done when: `cargo package -p esp` from a clean commit passes on the stable cohort.
- Needs: nothing else; an ESP release is separate and needs explicit authorization.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:235-238,488-500,551`

### CU3. Compute-stack upkeep: carried vendor patches
Three patches are carried and must each name a removal condition: Burn Remote's owner session control and acknowledged teardown, Cubek reduction's infinity, NaN, top-k and tie fixes, and matmul's packed-Q4 memory fitting (the CubeCL runtime persistence opt-out is retired). No upstream issue or pull request is filed. Pristine released browser reduction behaviour was not measured. Also open: retiring the local rust-gpu fork by re-checking plain upstream; and Knot and Isometry handoffs, where BERT on BrowserWebGpu needs extra patches only if Knot ever runs BERT there.
- Done when: each patch has a recorded removal condition; the rust-gpu re-check is made.
- Needs: authorization for any upstream communication.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:370-372,1018-1051,5014-5022,5168-5171`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1296-1304`

### CU4. Compute-stack upkeep: Eponym and the model probe baseline
Eponym and its ambience probe remain unverified because the checkout lacks the renderer crate manifest (`crates/renderling/crates/renderling/Cargo.toml` under the code root) and its local Crabslab patch directories; their Conatus repins are prepared and no unrelated renderer source was substituted. The untouched model probe baseline lacks `LiveryPaintList::scrolled` at Rootstock line 758; the aligned cut builds and no scroll implementation was changed (a Genet and Rootstock owner question).
- Done when: Eponym's probe builds and passes against a restored renderer checkout; the probe baseline builds without the alignment.
- Needs: the renderer checkout; the Genet and Rootstock owner.
- Rulings: -
- From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:5094-5096,5177-5180`

## Open questions

- **Which phase or plan owns the remote autotune reclaim failure** that M1 surfaced ("needs an owner"). From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:5-6`
- **Reliability and reputation accounting** is out of scope "until a lane gives standing a purpose"; its kith lane retired on 2026-09-02, so it has no home. From: `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:132-133`; `archive_docs/2026-10-28/2026-07-03_archived_plan_tails_plan.md:1197-1206`
- **Has Isometry's own repin onto the stable cohort landed?** The Burn plan's S18 waited on a checkpoint that was superseded by Isocosm moving to stable, but Isometry's repin is not stated done. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:627,1652,1776`
- **Does the Burn Fusion remote MiniLM autotune panic status need correcting in the Burn plan's record?** The Burn plan calls it a separate backend sidequest; the distillery plan records it as gone on stable and replaced by the autotune reclaim failure above. From: `archive_docs/2026-10-28/2026-08-09_burn_0_22_migration_plan.md:572-575`; `archive_docs/2026-10-28/2026-10-06_distillery_models_plan.md:5-6`

## Findings

- 2026-10-28: the Fusion profile and the D2 rows were run on a GPU host only; a low-end host cannot build or run them (noted, not a plan item).
- 2026-10-28: tolerant comparators and portable checkpoints appeared in both Distillery's and the mesh host's deferrals and are one item each (M4).

## Progress

- 2026-10-28: cut from the distillery models plan and the Burn 0.22 migration plan's residue; nothing new landed.
