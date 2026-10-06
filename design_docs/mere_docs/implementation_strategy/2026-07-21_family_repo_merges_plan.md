# Family Repo Merges Plan (eidetic + conatus)

**Status (2026-10-06):** superseded by the
[repo consolidation plan](2026-07-23_repo_consolidation_plan.md). P1–P3
landed 2026-07-21 and the `mere-eidetic` rename and publish 2026-07-22; then
`6a37de6b` (2026-07-23, "Absorb nine component families into mere") made every
family crate a mere workspace path dependency, and `ca798151` (2026-09-18)
reversed the rename, so `crates/eidetic/eidetic-core/Cargo.toml` names the
package `eidetic` again. The P4 metadata follow-on is retired: the family
manifests already carry `repository = "https://github.com/merely-made/mere"`.
Open: whether donor GitHub archiving is still a task (see the open question
under P4).

2026-07-21. Ecosystem repo reorganization: group satellite single-crate repos
into family repos by concern, where the family is a true lockstep stack.
Decided with Mark in session; both merges executed same day.

## Plan

- **P1. conatus** (physics family): merge repos numen + quint + seiche into
  `mark-ik/conatus`, one workspace, histories preserved. **DONE.**
- **P2. eidetic** (durable-memory family): merge repos muniment + codicil +
  chartulary + scholia into `mark-ik/eidetic`, same shape. **DONE.**
- **P3. Consumer repoint**: mere, turnstone, hocket, isometry, woodshed,
  servitor onto the family repos (manifest URLs, path deps, local `.cargo`
  patches, lockfiles). **DONE** (this session; per-consumer detail below).
- **P4. Follow-ons**:
  - **DONE 2026-07-22 — mere-eidetic rename + publish.** mere's
    `crates/eidetic/*` lane renamed to **mere-eidetic** (Mark's ruling: the
    eidetic name belongs to the family; the mere lane takes the prefix). Done
    the low-churn way so it did NOT ripple through the workspace: the published
    package names became `mere-eidetic` + `mere-eidetic-{fjall,https-fetcher,
    iroh-fetcher,search}`, but each crate keeps its old `[lib] name`
    (`eidetic`, `eidetic_fjall`, …) and mere's `[workspace.dependencies]` keep
    their `eidetic*` keys via `package = "mere-eidetic*"` — so all `use
    eidetic::` sites, consumer manifests, the `image-store-check` probe, and
    the `crates/eidetic/` directory are untouched. turnstone's two direct git
    deps + its gitignored `.cargo` patch keys updated the same way (patch keys
    are package names, so they became `mere-eidetic*`). **`mere-eidetic` 0.0.1
    published to crates.io** (mirrors the old `eidetic` 0.0.1). The bare
    **`eidetic` 0.0.1 stays as an orphaned reservation** — free for a future
    family facade. Companions were never published, so their rename just
    reserves the `mere-eidetic-*` names by manifest (not yet published).
    Verified: `mere-eidetic` lib 85 tests green, consumers + turnstone build.

    **Corrected 2026-10-06 (S14 pass):** the rename was reversed in
    `ca798151` (2026-09-18, "Rename mere-eidetic* packages to eidetic*");
    `crates/eidetic/eidetic-core/Cargo.toml` reads `name = "eidetic"`.
  - GitHub-archive the seven donor repos (numen, quint, seiche, muniment,
    codicil, chartulary, scholia) with tombstone READMEs pointing at the
    family repos; delete the local checkouts after. Left for Mark.

    **Open, raised by the S14 pass (2026-10-06):** is archiving the seven
    donor repos still a task, now that their crates live in mere
    (`6a37de6b`)? The pass could not check GitHub. Options: keep it as an open
    tail; record it as done or dropped.
  - Next `cargo publish` of each crate picks up the new `repository` field
    (update each crate's `Cargo.toml` repository URL at that time; not
    changed preemptively so the published metadata keeps matching the live
    published versions).

    **Corrected 2026-10-06 (S14 pass):** the manifests already say
    `repository = "https://github.com/merely-made/mere"`
    (`crates/eidetic/chartulary/Cargo.toml`,
    `crates/conatus/seiche/Cargo.toml`, `crates/servitor/Cargo.toml`), so this
    follow-on is retired.
  - crates.io `eidetic` (0.0.1, Mark's reservation) still carries the
    mere-lane description; reword at next publish.

## Findings

- **Bucket analysis** (the reorg's scope ruling): of 31 repos, only two
  satellite groups are true lockstep families. Everything else keeps its
  posture: engines (mere, genet, cambium, netrender) as-is; apps (turnstone,
  isometry, hocket, woodshed) as-is; deliberate standalones (wgpu-graft,
  wgpu-weld, wgpu-scry, misfin, wavicle, personae, armillary, netfetcher,
  vates, sibylla, servitor) as-is. **retinue + tulle + tucket + sennet
  stay separate repos: declined by Mark for legal reasons** (provenance
  boundary; MeshCore source-readable vs Meshtastic clean-room must remain
  independently auditable).

  **Corrected 2026-10-06 (S14 pass):** both postures changed on 2026-07-23.
  `6a37de6b` absorbed the standalones personae, armillary, vates, sibylla and
  servitor into mere, and retinue, tulle, tucket and sennet merged into one
  workspace (retinue's README, History section; mere's root `Cargo.toml` pins
  that one revision).
- **Names**: family repos take fresh names rather than the top crate's name.
  **eidetic** for the memory family (Mark: the name was always meant for the
  memory/recording/consolidating role). **conatus** for the physics family
  (cause-side word: the instantaneous striving that integration turns into
  motion, Leibniz; covers inertia/perseverance via Spinoza and goal-free
  motion like a seiche's oscillation). **telotaxis** stays banked for a
  future goal-seeking/steering layer; it names stimulus-steered goal-directed
  behavior only.
- All seven crate names plus bare `eidetic` were already Mark's on crates.io,
  so no claims were needed; `conatus` and `telotaxis` verified free 2026-07-21
  (conatus repo name used; neither crate name claimed yet).
- Merge mechanics: `git subtree add --prefix=crates/<name> <local-repo> main`
  preserved each crate's full history in the family repo. Intra-family deps
  converted to `path` + `version` (publishable). Member lockfiles were never
  tracked; each family has one workspace lock.
- Cargo resolves git deps by crate name inside a workspace repo, so consumer
  git URLs simply swap to the family URL; `branch`, `version`, and `features`
  fields survive unchanged.

## Progress

- conatus: founded, 3 subtree merges, path-linked, **98 tests green**
  (13 numen + 35 quint + 50 seiche), pushed to `mark-ik/conatus` (public).
- eidetic: founded, 4 subtree merges, path-linked, **70 tests green**,
  pushed to `mark-ik/eidetic` (public).
- mere: 14 git-dep lines across 10 manifests repointed to `eidetic.git`;
  `.cargo/config.toml` patches moved to family paths plus a new
  `[patch."…/eidetic.git"]` section; lock refreshed (zero references to old
  URLs); `mere-kernel` lib **273 tests green** on both family paths.

  **Corrected 2026-10-06 (S14 pass):** `6a37de6b` (2026-07-23) made every
  family crate a mere workspace path dependency; no `eidetic.git` or
  `conatus.git` reference remains. `crates/eidetic` now holds chartulary,
  eidetic-core, hagiograph and muniment, and `crates/conatus` holds conatus,
  modulus, nisus, numen and seiche (quint was folded into its owners in
  `eae87153`, 2026-08-31).
- Remaining consumers (turnstone, hocket, isometry, woodshed, servitor):
  repointed this session; see the commit trail in each repo.
- **2026-10-06 (S14 pass).** Status and claims corrected against the tree at mere 535bca11, from the D2 record in support/doc-audit/d2/batch_41_s14_phase_b3.md: the plan recorded as superseded by the repo consolidation plan, with the 2026-07-23 absorption (`6a37de6b`) and the `ca798151` rename reversal noted, the P4 metadata follow-on retired, and donor archiving raised as an open question.
