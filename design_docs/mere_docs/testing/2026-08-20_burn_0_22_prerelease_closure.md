# Burn 0.22 prerelease closure receipt

**2026-10-06 S16 annotation (S14 pass):** S16 ran on 2026-10-05. Main merged
the pre.4 branch at `cec0b3a4` and was pushed, origin/main then at `07db35e2`
([migration plan](../implementation_strategy/2026-08-09_burn_0_22_migration_plan.md)
§13.44). Main's production row is now Burn `=0.22.0-pre.4`, CubeCL
`=0.11.0-pre.4` and Cubek `=0.3.0-pre.4`. The 2026-10-03 annotation's "Main
stays on pre.2 until S16", and the pre.2 production row in the 2026-08-20
Status and Remaining gate below, describe main before S16. The pre.2 receipts
stay historical, and stable publication remains gated.

**2026-10-03 S15 closure annotation:** on the pre.4 migration branch the
row is Burn `=0.22.0-pre.4`, CubeCL `=0.11.0-pre.4` and Cubek `=0.3.0-pre.4`,
exact-pinned in every consumer manifest, with one version per crate in the root
and graphshell-web graphs and no Turso or SQLite. Every S13 receipt passes on
that branch; the 2026-10-03 section below has the patch table and receipt
index. Main stays on pre.2 until S16, which waits on ruling 534's quiet GPU-on
A/B, and stable publication remains gated as before. Earlier annotations and
the dated text below keep their original scope.

**2026-09-30 allocator stop:** four local `burn-cubecl` selectors were retired
at `124fc42b` after ruling 410's controls. Remaining numerical/matrix/build
checks passed, but remote reclaim left ten allocations / 5,323,776 active bytes
against zero. Ruling 411 authorized bounded diagnosis. Explicit post-failure
GPU completion polling released active allocations; both diagnostic runs kept
the original failure, and source bytes/timestamps were restored. This is not a
repaired lifecycle pass. See migration plan §13.29 and
[the diagnosis receipt](2026-09-30_pre4_allocator_diagnosis.md).
The patch-design fork, zero-baseline requirement and remaining acceptance/main
promotion holds remain. Historical text below retains its original scope.

**2026-09-29 later S13 annotation:** the native comparison and all 21 patched
browser cases pass. Pristine upstream pre.4 also passes all 21 cases, with no
GPU errors. This triggers migration-plan §13.8(2), rather than satisfying the
expected-failure control. See §13.26 and the committed `2026-09-29_pre4_*`
browser receipts. Source bytes were restored exactly; patch retirement or
retention awaits Mark. Extrema, remote lifecycle and main promotion remain held.

**2026-09-29 pre.4 reconciliation annotation:** migration plan §13.25 records
the exact published-main integration through `99e44853` / Genet `19c20687`,
coherent locked graphs and fresh affected compile/runtime verification.
Conatus and ESP were rerun after dependency-source review invalidated their
initial carry assumption; only Numen carries unchanged-closure execution
evidence. This does not change the historical pre.2 production receipts below.
S13 (all ten graph plus eleven embedding cases, native comparison, unpatched
control, extrema and remote lifecycle), S15 closure and main promotion remain
held behind their own gates. See `pre4-semantic-reconcile` under
`Code/testing/mere/receipts/2026-09-29` for commands, logs and source seals.


**Date:** 2026-08-20

**Status:** The explicitly chosen `0.22.0-pre.2` migration remains the production
row and is green on native,
wasm compile, real WGPU, host-owned existing-device adoption, and clean
extracted-package verification. Burn `0.22.0-pre.3` is now published and has
passed the bounded source/package checks below; this is an audit, not a
production repin. Stable 0.22 is still unpublished, so publication remains
gated.

## Dependency repair

The first WGPU wasm check failed inside the published
`cubecl-runtime 0.11.0-pre.2`:

```text
error[E0433]: cannot find module or crate `wasm_bindgen_futures`
cubecl-runtime/src/tune/tuner.rs:485
```

The crate calls `wasm_bindgen_futures::spawn_local` on wasm but its published
manifest omitted the direct dependency. The next upstream checks also require
`cubecl-common`'s `serde` and `hash` features outside desktop targets. CubeCL
already fixed both facts in commits `bce4e489` and `7a2ee1c3`.

`support/patches/cubecl-runtime` is the registry source for exactly
`0.11.0-pre.2`. Only its Cargo manifests differ: they add the direct wasm
dependency and make the two required `cubecl-common` features unconditional.
The root patch records those commits and its deletion condition. A one-crate
Git overlay was rejected because Cargo followed the newer crate's path edges
into six newer CubeCL internals and split the prerelease graph.

The run also found a Mere-owned gap: Quint's `field-burn` feature reached
`getrandom 0.4` on wasm without activating `wasm_js`. `field-burn` now owns that
feature edge just as the WGPU extensions already did.

## Wasm compile matrix

Target: `wasm32-unknown-unknown`. Every configuration was checked separately
with `--no-default-features`.

ESP passed eleven rows:

- default and `actor`;
- `decoder` and `decoder-wgpu`;
- `index-burn` and `index-burn-wgpu`;
- `bert`, `bert-wgpu`, and `bert-validation`;
- combined CPU models: `decoder,index-burn,bert,bert-validation`;
- combined browser WGPU models:
  `decoder-wgpu,index-burn-wgpu,bert-wgpu,bert-validation`.

Quint passed five rows:

- default;
- `field-burn`;
- `field-burn-wgpu`;
- resident `field-gpu`;
- `field-rhai`.

These are compile receipts. Headed browser model execution remains the D2
browser-ceiling lane.

## Existing-device execution

Command:

```text
cargo test -p quint --release --features field-gpu \
  --test resident --test resident_chunk -- --nocapture
```

Result in the current shared tree: 8 passed, 0 failed, 0 ignored. The run did
not take either test's "no wgpu adapter" return path. Seven tests are the
existing migration surface; the fourth resident-chunk test belongs to a
concurrent stamped-patch lane and is not included in this closure commit.

- Four resident tests executed Quint-authored CubeCL kernels for force-law,
  settling, springs, and stamped position leases.
- Four resident-chunk tests proved Burn and raw views share the same CubeCL /
  wgpu allocation, exact integer planes stay exact, exported lease sizing is
  sound, and committed patches retain and restamp the allocation.

This is the migration's existing-device receipt: the test host creates the
adapter/device/queue, then Quint registers those exact handles with Burn and
CubeCL. It does not claim remote or cross-machine execution.

## Clean package receipt

From a detached clean worktree at the closure commit:

```text
cargo package -p esp
Packaged 55 files, 528.7KiB (138.6KiB compressed)
Finished `dev` profile ...
```

Cargo verified the extracted `esp 0.1.0` package rather than compiling the
workspace source in place.

## Remaining gate

The production row remains `0.22.0-pre.2` until a separately authorized stable
repin. The temporary CubeCL runtime patch should be deleted at repin because
pre.3 contains the two upstream packaging fixes above. The Mere-owned
same-allocation `burn-cubecl` patch and targeted `burn-remote` lifecycle patch
still need a source rebase and fresh receipts on whichever row is selected.

**2026-09-16:** Mark authorized the pre.3 repin, superseding the stable
gate above. The row is now `0.22.0-pre.3`; the two Mere-owned patches rebase
onto it. Execution is tracked in the Burn 0.22 migration plan's progress.

## 2026-08-26 pre.3 compatibility audit

The crates.io API reports the exact prerelease packages needed by the current
roots: `burn`, `burn-remote`, and `burn-cubecl` `0.22.0-pre.3`, plus
`cubecl`, `cubecl-runtime` `0.11.0-pre.3`, and `cubek` `0.3.0-pre.3`. The
published Burn source is the `v0.22.0-pre.3` release. The package is named
`burn-pack`, not `burnpack`.

An isolated ESP source probe copied the crate, removed only unrelated local
workspace/path dependencies and feature rows, then ran:

```text
cargo check --manifest-path C:\t\esp-pre3-probe-0826\Cargo.toml \
  --features "bert-validation bert-wgpu decoder-wgpu index-burn-wgpu" -j 1
Finished `dev` profile [unoptimized + debuginfo] target(s) in 16m 08s
```

The selected ESP model/WGPU rows compiled through Burn, burn-wgpu,
burn-cubecl, cubecl-runtime, burn-ir, burn-pack, and the pre.3 CubeCL graph.
This is a bounded source-compatibility receipt. It is not a production
manifest or full feature-matrix pass, and the temporary pins were reverted.

The corresponding temporary `cargo check -p distillery --features remote
-j 1` could not reach rustc: Cargo stopped while updating the shared `genet`
checkout because another process held its packfile. Until a clean Distillery
workspace check completes, Distillery pre.3 source compatibility remains
unverified. A dependency-only probe was not completed and contributes no
receipt.

Patch disposition against the pre.3 published sources:

| Area | Result | Disposition |
| --- | --- | --- |
| `cubecl-runtime` wasm manifest fixes | Direct `wasm-bindgen-futures` and unconditional `cubecl-common` serde/hash requirements are present upstream | Remove the packaging patch at repin; rebase only Mere identity helpers if still required |
| `burn-cubecl` same-allocation binary launch | pre.3 has alias launch helpers but still chooses the mutable broadcast path before checking shared allocation | Keep and rebase the Mere patch; require headed same-allocation and LayerNorm receipts |
| `burn-remote` lifecycle | pre.3 exposes service close but `SessionManager::close` still only removes the map entry; no targeted pump drain/session reservation API is upstream | Keep and rebase the Mere lease-bound close patch |
| Remote Fusion/autotune | No pre.3 source evidence removes the existing remote allocation/ordering hazards; the pre.2 plain-WGPU result remains the supported lane | Keep Fusion/autotune out of production remote defaults; rerun only as a separately gated sidequest |

Pre.3 has concrete follow-up value for later work: the release notes include
LoRA persistent-allocation and explicit-dtype fixes, dtype preservation during
composition, and on-demand Burnpack tensor streaming. These justify a fresh
native LoRA receipt and a bounded `burn-pack` streaming probe when portable
checkpoint export becomes an active requirement. They do not change the
current ModelSession/ordinary PEFT claim or authorize a production repin.

## 2026-09-29: historical control supports conditional patch retirement

Ruling 410 authorizes the historical comparison, then retirement if supported.
Migration plan §13.27 records reconstructed pre.2 failing five graph cases while
all eleven embedding controls pass; upstream pre.4 passes all 21 cases in the
same current browser session. GPU errors are empty in both. Historical source
provenance is qualified explicitly; this does not identify a specific upstream
fix. Exact source restoration and fresh served-asset checks are recorded.

Nine native unfused launcher controls are now prepared in the embedding fixture
for patched/upstream comparison. Selectors remain patched and retirement,
remaining headed/lifecycle gates and main promotion are not accepted yet.

## 2026-09-29: burn-cubecl retirement on the migration branch

Ruling 410's condition is satisfied by the same-browser pre.2/pre.4 comparison
and nine native unfused launcher cases passing identically with and without
the patch. Plan §13.28 records independent source, numerical and fault-control
review. Four selectors now choose pristine registry pre.4, preserving all lock
versions/edges and the root's distinct-source duplicate name/version entries.
The vendored source and its six-field guard receipts remain historical evidence.

Three independent patch selectors remain: cubecl-runtime persistence policy,
cubek-reduce extrema handling and burn-remote lifecycle control. Earlier tables
retain their dated counts. Affected production checks, remaining S13 and main
promotion still require acceptance; this is not stable-release closure.

## 2026-10-03: the pre.4 row at S15

The branch's selected patches against published pre.4 (migration plan §13):

| Patch | Selected by | What it changes | Remove when |
| --- | --- | --- | --- |
| `cubecl-runtime` | root, graphshell-web, probe, remote fixture, both repros | manifest only: `persistence` leaves the default features (ruling 375) | upstream makes persistence opt-in or drops the defaults-on `cubecl-cpp` edge |
| `burn-remote` | root, remote fixture | lease-bound targeted close (§13.17); close waits for teardown completion and reports failures (ruling 508, §13.32) | an upstream release has equivalent session control and passes the lifecycle receipt |
| `cubek-reduce` | probe, extrema repro | extrema identities as runtime values, not literal infinity bits | a released row passes the headed extrema receipt without it |
| `burn-cubecl` | none | vendored pre.4 source kept as provenance; selectors retired under ruling 410 (§13.28, and graphshell-web's row in §13.30) | delete with the provenance it documents |

graphshell-web's start function runs the module's static constructors once
(ruling 509, §13.33). Ruling 532 moves that into one shared stack helper,
ruling 536 puts it in `cambium-genet-web-host`, and ruling 533 adds
Distillery's model probe. The probe cannot yet depend on that crate because
of conflicting exact `wasm-bindgen` pins (§13.35). The repros stay
command-linked.

Receipts on the branch:

| Gate | Result | Record |
| --- | --- | --- |
| S13 (a)/(a') same-allocation and LayerNorm | upstream pre.4 passes all 21; patch retired (ruling 410) | §13.26 to §13.28 |
| S13 (b) extrema, headed | four cases, `gpu_errors: []` | §13.30, §13.33 |
| S13 (c) two-peer lifecycle | zero baseline, strict ceiling, exact recovery, second live lease | §13.32 |
| S13 (d) existing device, conatus resident | pass in release | §13.30 |
| S13 (e) ESP WGPU parity and real MiniLM | pass | §13.30 |
| Native, wasm, Distillery, Djinn, workspace | pass | §13.25, §13.30 |
| P5 web settle and 14 law receipts | pass on the constructor-fixed bundle | §13.33 |

Stable 0.22 is still unpublished, so ESP publication and the stable closure
named in this receipt's first sections remain gated.
