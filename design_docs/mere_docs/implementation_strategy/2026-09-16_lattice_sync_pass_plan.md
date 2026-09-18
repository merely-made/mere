# Lattice Sync Pass Plan

**Date**: 2026-09-16

**Status: plan, reviewed 2026-09-16.** The rulings below preserve the earlier
decision record. This review revises the execution and acceptance criteria;
it does not execute the migration or cleanup.

## Objective

Every repository under `Code/repos` resolves its owned git dependencies to current
source, builds from a clean checkout, is committed and pushed, and runs on Rust
1.98.1. Machine-local redirects stay, but stop leaking into committed locks. Stale
local state is removed.

**Done when:**

1. Every `merely-made/*` rev pin in every committed manifest equals its source's
   origin head as of this pass, or trails it only by the pass's own pin-only
   commits (the cycle rule below). One direct revision per source across the
   repository's maintained manifests; transitive cycle exceptions are recorded
   and checked for incompatible type identities.
2. Every supported stable development toolchain and owned CI selector uses
   `1.98.1`. Inventory explicit MSRV, firmware and purpose-specific nightly
   exceptions separately; do not rewrite them as ordinary stable pins.
3. The seven redirect repos carry `resolver.lockfile-path` locally, ignore
   `.cargo/local/` in git, and commit a lock with no path package from outside
   the repository.
4. Each supported workspace passes its recorded check command on 1.98.1 from a
   redirect-free checkout, with no new unexplained unused-patch row. The host
   default-feature gate is `cargo +1.98.1 check --workspace --locked`;
   target-specific workspaces have explicit commands and prerequisites.
5. Every completed migration commit is published and its tested revision is
   recorded. Active checkouts may await integration; that is recorded separately.
6. Local development resolves the intended sibling packages and leaves every
   committed lock byte-identical. Each independent workspace has its own local
   lock, and tracked setup instructions reproduce this on another machine.

Cleanup in P2 is a separate deliverable, not a prerequisite for portable builds.
Blocked repositories remain incomplete, even when independent ones finish.

## Two resolution modes

Committed manifests keep fetchable Git revisions for cross-repository
dependencies. Paths within the same repository remain normal. Committed locks
describe the portable graph; ignored Cargo patches and local locks describe the
developer's sibling graph. A local lock does not freeze the contents of a path
dependency, and a successful local build does not validate the portable graph.

The lock redirect protects a file, not dependency identity. Use `cargo metadata`
to verify the actual selected packages, versions, sources and manifest paths in
both modes. Do not infer provenance just from a package name in `Cargo.lock`,
which does not record the filesystem path of a path package.

## Rulings (Mark, 2026-09-16)

- **Redirects stay.** Only woodshed's dead entries go.
- **Local locks.** All seven redirect repos (mere, turnstone, genet, hocket,
  cleromancy, mer3ly, woodshed) set `resolver.lockfile-path` in their untracked
  config and commit a clean lock, including the four that ignore theirs today
  (mere, genet, cleromancy, turnstone).
- **Toolchain.** Everything moves to 1.98.1; the 294.1 MB download is approved.
- **Scope.** Every consumer, including the far-behind ones, now.
- **Cycle.** "Up to date" means at head, less the pass's own pin-only commits.
- **Burn pre.3 shelved.** Mere stays on pre.2 and Knot's lock returns to pre.2.
  The repin branch is kept as a pushed archive tag.
- **Cleanup.** Dead worktree records, merged local branches, unlinked
  `Code/worktrees` folders, Code-root leftovers and toolchain lint go.
  Renderling's uncommitted edits are committed to a branch and pushed to the fork.

## Findings (2026-09-16)

The original audit counts, pin lags and session states below are a dated
snapshot, not a live execution inventory. P0 must refresh them before mutation.

**Review verified:** Mere still ignores `Cargo.lock` at every depth in
`.gitignore`; `.cargo/config.toml.example` teaches redirects without a separate
lock. `ports/graphshell/web/Cargo.toml` and
`knot-editor/crates/knot-document/Cargo.toml` define independent workspaces.
`Code/.cargo/config.toml` exists (currently comments only). These are concrete
reasons to update setup examples, inventory nested roots and inspect inherited
configuration rather than treating a new worktree as isolation.

**Committed locks are clean today.** A read-only audit of every repository
compared each lock's path packages against the packages its tracked manifests
define. No committed lock records a path from outside its repository. The
untracked locks do: mere carries 37 such packages, genet 17, cleromancy 81.

**`resolver.lockfile-path` solves the lock collision on the toolchain already
pinned.** It was stabilized in Cargo 1.97, and the path must end in `Cargo.lock`.
Config paths resolve against the directory holding `.cargo`. The old
`--lockfile-path` CLI flag was removed in 1.95, which is why an earlier probe
came back negative. A scratch workspace proved the case with controls:

| Setup | Committed `Cargo.lock` | Local resolution |
| --- | --- | --- |
| `[patch]` redirect alone | rewritten with a path source (trap 9) | redirect |
| Redirect plus the snippet below | byte-identical | redirect, recorded in `.cargo/local/Cargo.lock` |
| No config (fresh checkout) | honoured with `--locked`, exit 0 | pins |

Cargo created `.cargo/local/` itself. A negative control, removing a package from
the local lock, made the local `--locked` build fail with exit 101, so the
local lock is really read. The snippet as verified:

```toml
[resolver]
lockfile-path = ".cargo/local/Cargo.lock"
```

Cargo older than 1.97 ignores the key and would rewrite the committed lock.
Hocket pins 1.96.0, so its toolchain moves in the same step as its config.

The [Cargo configuration reference](https://doc.rust-lang.org/stable/cargo/reference/config.html#resolverlockfile-path)
confirms the 1.97 minimum. The snippet above is for one workspace. If inherited
by an independent nested workspace, its fixed path still names the same lock.
Give each nested workspace a distinct ignored lock and an explicit configuration
or launcher. Cargo discovers config from the invocation directory and its
ancestors, so `--manifest-path` alone does not select a nested config. Verify
both the documented shell command and editor invocation before rollout.

**Review probe, Cargo 1.97.1:** two dependency-free workspaces, one nested,
inherited the snippet above. Running `generate-lockfile --offline` first at the
root and then in the nested workspace replaced the root's local lock with the
nested package; no nested local lock was created. This confirms that the
single-workspace scratch test does not establish safe nested use. Reproduce
this control, then prove distinct locks under the chosen local setup on 1.98.1.

**Toolchain.** Stable 1.98.1 (manifest date 2026-09-03). The component set
matches 1.97.1's install, 294.1 MB compressed from `static.rust-lang.org/dist`:

| Component | MB |
| --- | --- |
| rustc | 71.3 |
| llvm-tools | 45.7 |
| rust-std, x86_64-unknown-linux-gnu | 30.7 |
| rust-std, aarch64-apple-darwin | 29.8 |
| rust-std, wasm32-wasip2 | 24.5 |
| rust-docs | 24.2 |
| rust-std, x86_64-pc-windows-msvc | 23.0 |
| rust-std, wasm32-unknown-unknown | 22.4 |
| cargo | 10.3 |
| rust-src | 5.7 |
| clippy | 4.0 |
| rustfmt | 2.6 |

1.97.1's install occupies 2,213 MB. Ten repos pin 1.97.1, hocket pins 1.96.0, and
cleromancy uses the default stable.

**Pin snapshot.** Every owned rev pin in committed manifests. Lag is measured
against the source's origin head in its local clone, and "code" means the lag
touches `.rs`, `.toml`, `.wgsl` or `.js` files.

| Consumer | Source | Revisions pinned (lag) |
| --- | --- | --- |
| genet | netrender | two: 1 code, 44 code |
| mere | genet | 8 code |
| mere | netrender | 1 code |
| mere | wgpu-weld | 2 code |
| mere | retinue | two: 6 code, 67 code |
| mere | knot-editor | two: 26 code (root), 44 code (`ports/djinn`) |
| knot-editor | mere | 4, docs only |
| knot-editor | genet | 8 code |
| knot-editor | retinue | 4 code |
| retinue | mere | two: 79 code, 221 code |
| retinue | genet | 160 code |
| turnstone | mere | 46 code |
| turnstone | knot-editor | 25 code |
| turnstone | turquet | 10 code |
| turnstone | genet | 8 code |
| turnstone | netrender | 1 code |
| cleromancy | mere | 46 code |
| cleromancy | genet | 8 code |
| cleromancy | turquet | 5 code |
| mer3ly | mere | 221 code |
| woodshed | mere | three: 46, 162, 221 code |
| woodshed | genet | three: 8, 160, 160 code |
| woodshed | netrender | 44 code |
| hocket | mere | 221 code |
| hocket | genet | 160 code |
| hocket | netrender | 44 code |
| isometry | mere | three: 23, 46, 681 code |
| isometry | genet | 8 code |
| isometry | netrender | two: 1, 3 code, plus a `branch = "main"` dep |
| isometry | cleromancy | 0 |

Pins already at head (retinue in turnstone, smolweb, wgpu-graft, wgpu-weld in
turnstone, woodshed, vano, genet's boa fork) are omitted. Branch and tag pins
(p2panda, vello, swarm-discovery, arboard, piccolo, wavicle, woodshed in hocket,
boa and iroh-address-lookups in mere) track their refs and are out of scope.

**Cycles.** mere pins knot-editor and retinue, and both pin mere. turnstone pins
all three. No pass can leave every pin at head, hence the cycle rule.

**Other facts.**

- Unused-patch rows in current locks:
  - mere: boa_engine, boa_gc, genet-scripted-dom, iroh-mdns-address-lookup
  - knot-editor and turnstone: p2panda-stream
  - knot-document (nested): genet-taffy
- Woodshed's config has 21 dead entries of 43 (19 distinct paths): 16 under a
  deleted `Documents\Codex` worktree, and mere crates since removed or moved
  (`conatus/quint`, `eidetic/codicil`, `eidetic/scholia`).
- Hocket's config has 1 dead path of 40. It is not in the rulings, so it stays.
- `hocket` and `ringdown` fetch with "Repository not found" from origin.
- `crates/cargo-gpu` holds one unpushed commit pointing spirv-builder at the
  local rust-gpu fork. It is local by design and stays unpushed.
- Rev pins on `mark-ik/*` forks (isometry, retinue and netrender on vello; genet
  on boa) are Mark's to move and are left alone.
- Peer sessions are live in mere (38 uncommitted paths, builds running),
  isometry (27 uncommitted paths) and knot-editor (moved its mere pin at 17:02).
  Mere's `ports/djinn/Cargo.toml` has been dirty since 2026-09-15.

## Method

Use main directly when it is idle and the edit set can be isolated. Create
`Code/worktrees/sync-<repo>` only to avoid an actual collision, recording its
base and ownership, with `CARGO_TARGET_DIR=C:\t\sync-<repo>`. Keep build outputs
outside the checkout. Use a disposable checkout of the candidate revision for
portable validation; do not disable redirects in a live development tree.

A worktree omits untracked repo config but still inherits ancestor config,
Cargo-home config and environment overrides. Run from the intended workspace
directory, inspect both `config` and `config.toml` and any includes, and use an
isolated Cargo home or a verified redirect-free one. Clear resolver/path/source
overrides for the validation process. Preserve documented build requirements
(linkers, SDKs and required flags); move missing portable requirements into
tracked setup rather than quietly copying machine-specific config.

Each level lands as a bounded commit; Mere needs at least passes A and B.
Coordinate files with active sessions before editing their local config or
integrating commits. Recheck origin before pushing: if it advanced, integrate
and rerun affected gates without force-pushing. Nothing is pulled into a main
tree beneath an active session. The 2026-09-15
reclaim script is not run during the pass, because the work lives under `C:\t`.

`cargo check` covers default features on the host target. It proves nothing
about feature-gated or wasm code, and the pass claims no more than that.

## P0. Snapshot and notify

Record every `merely-made/*` source's origin head as the target set in Progress.
List live sessions and message those working in repositories the pass will touch.

Fetch each source first; a cached remote-tracking ref is not a fresh head
receipt. Record fetch failures as blocked. Freeze the target set rather than
chasing unrelated commits throughout the pass; update it only for recorded
prerequisites or this pass's published commits.

Build a finite inventory of repositories and independently resolved workspace
roots, including nested/excluded ports. For each row record its manifest, lock
tracking policy, toolchain/CI selectors, build command, platform prerequisites,
redirect config, expected local lock, and baseline unused patches. Historical
fixtures and vendored workspaces need explicit classification, not blind repins.
Every repository in scope needs a row, even if it requires no pin change.

*Done:* refreshed target set and complete workspace inventory recorded; active
ownership recorded and relevant sessions notified before their files change.

## P1. Tooling

1. Install 1.98.1 with the component set above.
2. Once the relevant session is idle, back up the config and existing locks.
   For each independently resolved workspace in the seven redirect repos, seed
   its distinct local lock from its current lock if present. Do not overwrite an
   existing local lock on rerun. Add the redirect only after confirming the
   command/editor uses Cargo 1.97 or newer. Hocket waits for its P3 toolchain
   integration; changing a toolchain only in a worktree does not update main.
3. Verify root and nested workspace commands resolve different local locks.
   Compare hashes of committed locks before/after, including locks currently
   ignored by git. `git status` alone cannot detect changes to ignored locks.

*Done:* `rustc +1.98.1 -V` reports 1.98.1 with the full component list. In six
repos local metadata identifies the intended sibling packages and leaves
portable lock hashes unchanged. Record Hocket as pending until its P3 step.

## P2. Cleanup

Run independently after P0; cleanup must not delay P3. Before each removal,
refresh worktree registration, dirty/untracked state, unpushed commits and active
process/session use. Resolve the exact absolute path and verify containment in
the named cleanup directory, including junction targets. Preserve changed or
active candidates and record them as deferred. A missing worktree registration
is not evidence that its files are disposable.

1. **Burn branch.** Tag mere's `burn-pre3-repin` head `610a32c5` as
   `archive/burn-pre3-repin-20260916` and push the tag after verifying that head.
   A tag preserves committed content only: separately save the dirty diff and
   untracked files with a restore recipe before removing the worktree and branch.
   Verify the saved content. Mark the pre.3 section
   of the Burn 0.22 migration plan shelved.
   *Done:* `git ls-remote --tags origin` lists the tag; the branch and worktree
   are gone.
2. **Renderling.** Commit the 26 uncommitted files, a rustfmt comment-wrapping
   pass, to a new branch `mark-ik/wgpu-30-comment-wrap` off `mark-ik/wgpu-30`, and
   push it to the fork.
   *Done:* the branch exists on origin; the tree is clean.
3. **Woodshed.** Remove the 21 dead entries from its untracked config.
   *Done:* no entry points at a missing path.
4. **Worktree records and branches.** Run `git worktree prune` in knot-editor,
   mere, retinue, turnstone and prns. Delete the local, never-pushed, fully
   merged `smolweb-next-20260913` branch in knot-editor, mere and turnstone.
   *Done:* no prunable records; the branches are gone.
5. **Unlinked worktree folders.** Delete these from `Code/worktrees`:
   - genet-docs-k6-ortet-20260906
   - genet-fleece-w3c
   - genet-network-runtime
   - genet-ortet-browser-fetch-20260906
   - genet-ortet-o3-20260906
   - genet-ortet-o5b-20260908
   - genet-ortet-o5b-20260908-b
   - genet-webgl-present-20260906
   - genet-woodshed-probe
   - ortet-receipt-audit
   - turnstone-p3-integration
   - webgl-livery
   - knot-editor-clean-4434584, a clone that is clean with nothing unpushed

   *Done:* eligible named candidates are gone; changed or active candidates are
   listed as deferred. Unlisted worktrees remain untouched.
6. **Code root.** Delete generated leftovers:
   - empty `cargo-homes`, `.targets`, `.codex-worktrees`, `.codex-targets`,
     `.codex-target`, `.codex-runner-setup`
   - `.cargo-row17`, `.cargo-mere-w3c-adapter`, `.cargo-cambium-range-v2`,
     `.cargo-check-logs`, `.cargo-homes`, `cargo-home-mere-w1-20260907`,
     `cargo-home-mere-w1-isolated`
   - `tmp` (four clean clones, nothing unpushed)
   - `_tmp_p2panda_073` (a clean clone)
   - `.workspace-proof-practice-reducer`
   - `.woodshed-release-verification-01a057ac`
   - `distillery-chronicle-final.out` and `.err`,
     `distillery-observer-test.log` and `.err`,
     `target-appearance-reader-test.log`, `target-appearance-turnstone-check.log`

   Kept as content, caches or live data:
   - `readme-proposals`, `readme-clips`, `experiments`, `validation`, `work`,
     `scry-shots`, `src`
   - `ebe.txt`, `misfin_spec.gmi`, `g4_burrow_run.png`, `loop0031.wav`,
     `temp-stickleback-store.rs`
   - `target/proofs`, `cef-cache`, `.venv`
   - `.tmp` (written within a day)
   - `webgl-livery2` (187,603 uncommitted entries)
   - `artifacts`, `scratch`, `archive`, `targets`, `membackup`, `morning-briefs`
     and the tool config folders

   *Done:* eligible delete-list entries are gone; deferrals are recorded and the
   kept list is untouched.
7. **Toolchain lint.** Run `rustup override unset --nonexistent`, then uninstall
   1.90.0 and 1.93.0 only after checking current pins, CI, overrides, installed
   tooling and active processes; absence from repository pins alone is insufficient.
   *Done:* `rustup override list` shows no missing directory; neither toolchain
   is listed.

Not done: global git config holds duplicate `safe.directory` entries. They are
harmless, and that setting is a security control, so it is left for Mark.

## P3. Pins, toolchain and locks, bottom-up

**Per-repository recipe.** Steps 1 to 6 happen in the repository's worktree.

1. Set the supported stable toolchain and matching owned CI selectors to
   `1.98.1`, preserving the inventoried explicit exceptions.
2. Move every in-scope `merely-made/*` rev pin in maintained manifests, nested
   workspaces included, to the target named for that level.
3. For a redirect repo, ignore local locks at every inventoried workspace root.
   Replace blanket lock-ignore rules with explicit tracking of supported
   portable locks; keep any fixture exclusions explicit. Update tracked config
   examples and setup instructions with the minimum Cargo version, separate
   local locks, configurable sibling locations and commands for both modes.
   For Hocket, install its P1 config only after main's toolchain is integrated.
4. Update the lock with a targeted `cargo update -p` for crates from moved
   sources, or generate it where none is committed yet. Sweep every nested lock
   that resolves a changed manifest. Re-run the lock audit: it must
   report no outside path package. Inspect tracked manifests for sibling paths
   too, including target-specific dependencies and patches. Confirm every
   source-less metadata package has a manifest inside the repository and that
   its required files are tracked. Git/registry packages live in Cargo's cache
   and are classified by source, not rejected for being outside the checkout.
5. Run the inventoried locked check for every supported workspace, plus
   `cargo metadata --locked --format-version 1` for source identity. Record
   `--all-targets` without gating on it. Hash committed locks before and after;
   prove Cargo read those locks rather than a redirected one. Record candidate
   SHA, command, working directory, toolchain, config inputs, exit code and log.
6. Compare unused-patch rows with the Findings list; a new one stops the step.
7. Commit, coordinate, push, and integrate into main once idle. Run a final
   locked check from the published revision with redirects absent. Recheck
   local mode after integration; local lock changes are expected when pins move,
   but portable locks must stay unchanged.

**Porting.** Far-behind repositories will fail step 5. A port proceeds when the
fix follows mechanically from the upstream change: a rename, a moved path, or a
signature whose replacement the upstream commit names. It stops and returns to
Mark when it needs a design choice, such as a removed capability with no
replacement, a changed meaning, or anything touching product behaviour. A
stopped repository stays in its worktree, and every step that consumes it waits.

**Levels.** Each level pins the published, tested heads left by the levels
before it, using P0's frozen heads for unchanged sources. Complete the inventory
for sources absent from this list before starting; this list alone does not
cover the stated every-repository scope.

Repository cycles can leave old Git source identities reachable transitively.
The pin-only exception permits finite history, not arbitrary duplicate type
families. Inspect the resolved graph at L5 and downstream: direct-pin alignment
does not prove transitive unification. Record unavoidable historical sources;
stop on incompatible duplicate identities. A structural fix, if needed, is a
separate boundary change, not repeated repinning toward an impossible fixed point.

- **L0.** Toolchain-only commits in wgpu-graft, wgpu-scry and wgpu-weld, which
  are sources with no owned pins.
- **L1.** genet: netrender unified to head.
- **L2.** mere, pass A: genet, netrender and wgpu-weld to head. Knot-editor and
  retinue pins are left for L5. The clean lock is committed.
- **L3.** retinue: genet to head, mere to L2's head (porting across 79 and 221
  commits).
- **L4.** knot-editor: mere to L2, retinue to L3, genet to head. The burn family
  returns to pre.2 in both locks, and root `[patch.crates-io]` gains
  `burn-cubecl` and `cubecl-runtime` from mere's git at the same revision, which
  carries the pre.2 patches.
- **L5.** mere, pass B, pin-only: knot-editor unified to L4 (root and
  `ports/djinn`), retinue unified to L3. `ports/djinn` waits for its session to
  go idle. After L5, knot-editor and retinue trail mere only by this commit.
- **L6.** woodshed (mere at L5, genet, netrender; unify three and three
  revisions), cleromancy (mere at L5, genet, turquet), mer3ly (mere at L5).
- **L7.**
  - turnstone: mere at L5, knot-editor at L4, retinue at L3, woodshed at L6,
    genet, netrender, turquet, wgpu-graft, wgpu-weld, smolweb
  - hocket: mere at L5, genet, netrender; its push waits on the broken remote
  - isometry: cleromancy at L6, mere at L5 unified from three revisions, genet,
    netrender unified; after its live session goes idle

*Done:* done-conditions 1 to 6 hold for every inventory row. Blocked rows retain
their failure reason and prevent claiming the full pass complete; independent
rows may still land.

## P3 follow-through. Keep fresh checkouts working

Add a redirect-free CI check for the inventoried supported workspaces, using
committed locks and the recorded toolchain. Include the manifest/metadata source
audit and lock hash check so a future local-path leak fails visibly. Document a
local setup command that refuses unsupported Cargo versions before resolution
and can be rerun without overwriting local configuration or locks.

*Done:* a clean clone needs only documented prerequisites, while the optional
local setup selects sibling sources and preserves the portable locks. Both
commands are recorded in tracked developer instructions and exercised once.

## P4. Unused patches

Once the pins have moved, diagnose each surviving unused-patch row read-only: a
version mismatch, a ref split, a dependency dropped, or a patch keyed to the
wrong source. Report to Mark; change nothing.

*Done:* each row has a stated cause in Findings.

## Stop rules

1. A port needs a design choice (P3, Porting).
2. A lock would commit a path package from outside its repository.
3. A step adds an unused-patch row the pass did not inherit.
4. Moving a pin splits a crate into two revisions in one graph that the step
   cannot unify.
5. A push is refused, for example hocket's broken origin.
6. A live session holds a file the step must change and does not go idle.
7. Rust 1.98.1 changes a build result in a way no pin move explains: rerun that
   repository on 1.97.1 and report.

## Progress

- **2026-09-16:** plan written from Mark's rulings; P0 not started.
- **2026-09-16 review:** clarified portable versus local resolution, nested lock
  isolation, inherited config, lock tracking and CI, frozen source receipts,
  cycle identity checks and cleanup preservation. Reviewed local manifests,
  config examples and ignore rules; a Cargo 1.97.1 scratch probe reproduced the
  inherited nested-lock collision. No migration or cleanup executed.
