# Burn 0.22 Migration Plan

**Date**: 2026-08-09

**Status: PRERELEASE MIGRATION EXECUTED 2026-08-20; stable closure remains
release-gated.** On the 0.22.0-pre.2 row Mark chose, both production roots
migrated (esp then quint, the plan's order), the vendored
`support/patches/cubecl-wgpu` backport retired with it exactly as the probe
predicted, and the workspace carries one `wgpu` 30.0.0 and one
`libsqlite3-sys` 0.38.2 (rusqlite behind CubeCL's autotune cache, as the probe
also predicted: the workspace is no longer sqlite-free, and that is CubeCL's
own persistence, not a crossing of mere's storage boundary).

The API shift was larger than "churn": 0.22 removes the backend type
parameter entirely. `Tensor<B, D>` is `Tensor<const D>`, every `B: Backend`
generic dies, and devices become runtime values (`Device::ndarray()`,
`Device::wgpu(DeviceKind)`) with type-erased dispatch. esp's model stacks
came out *simpler* (the generics were plumbing). Three findings worth keeping:

- **Fusion is a default of standalone `burn-wgpu`**, and quint's resident
  interop must build on the non-fused backend (`burn_wgpu::Wgpu`) or
  `from_primitive` sees a `FusionTensor` where it expects a `CubeTensor`.
  quint deps `burn-wgpu` with `default-features = false`.
- **The raw-tensor bridge survives** behind `burn/extension`:
  `Tensor::from_primitive::<burn_wgpu::Wgpu>(cube_tensor)` and
  `try_into_primitive` replace 0.21's `TensorPrimitive::Float` wrapping.
- **cubecl 0.11 launches slices, not `Array`**: kernel params become `&[T]` /
  `&mut [T]`, `ArrayArg` becomes `BufferArg`, and `SharedMemory::<f32>::new`
  becomes `Shared::<[f32]>::new_slice`.

Receipts, all on this machine's real GPU in release: quint parity 4 (repulse,
node exclusion, scalar and vector lowering), resident chunk receipt 3 and
resident 4 (the raw-buffer bridge), esp vector-kernel parity, BERT
ndarray/wgpu parity. CPU suites: esp 173, quint 63. Prior 0.21 numbers are
historical per the migration rules.

**Closure receipts 2026-08-20:** all eleven ESP feature combinations and all
five Quint feature combinations now compile for `wasm32-unknown-unknown`,
including every WGPU row and Quint's resident `field-gpu` row. The run found
two real prerelease packaging gaps: `cubecl-runtime 0.11.0-pre.2` omitted its
direct `wasm-bindgen-futures` dependency and left `cubecl-common`'s `serde` /
`hash` features desktop-only. `support/patches/cubecl-runtime` is the unchanged
published source plus the manifest corrections already made upstream in
CubeCL commits `bce4e489` and `7a2ee1c3`; delete it when a release containing
those fixes replaces this row. The same run found and fixed Quint's own missing
`getrandom/wasm_js` feature edge under `field-burn`.

The existing-device boundary is re-proven by the seven release-mode Quint
receipts inherited by this migration: the host opens the wgpu
adapter/device/queue, `ResidentClient::init` registers those handles with
Burn/CubeCL, four resident kernel receipts execute, and three chunk receipts
prove the Burn tensor and raw-kernel view resolve to the same allocation. The
current shared tree also passed a fourth chunk receipt from the concurrent
resident-patch lane; it is evidence, but not part of this migration's change
set. The earlier phrase "cross-device receipt" was imprecise: this plan requires
host-owned existing-device adoption, not remote or cross-machine execution. The
detailed command ledger is in the
[prerelease closure receipt](../testing/2026-08-20_burn_0_22_prerelease_closure.md).
The same closure passed `cargo package -p esp` from a detached clean worktree;
Cargo built and verified the extracted package. Repeat that receipt after the
stable repin because dependency packaging is part of the release gate.

Stable Burn 0.22 is still unpublished as of the 2026-08-26 pre.3 audit,
so stable publication closure remains open. Remote-adapter acceptance no
longer shares that gate: Mark explicitly reopened the prerelease row, the
targeted close patch and Distillery adapter landed 2026-08-23, and a clean
two-peer MiniLM receipt now proves native WGPU numerics, active-request reclaim,
and fresh-session recovery. Stable closure must repin and repeat those receipts.

Original doc follows.


**Status**: Release-gated on stable 0.22 only. The second reason (the sqlite
conflict) cleared on 2026-08-16, verified by probe; see below. Original note
follows.

**Original status**: Release-gated, and the gate now has a second reason. [`burn`](https://crates.io/crates/burn)
and [`burn-remote`](https://crates.io/crates/burn-remote) are still only
`0.22.0-pre.2` as of 2026-08-16; production migration waits for stable 0.22
unless Mark explicitly reopens that gate. An isolated prerelease compatibility
probe is allowed.

**2026-08-16: Mark reopened the gate, and the prerelease turned out to be
unreachable anyway.** During the stack-wide wgpu 30 unification the gate was
put to him explicitly and he chose the prerelease row. Attempting it produced a
hard resolution failure that is nothing to do with release stability:

- on `cfg(any(windows, linux, macos, android))`, `cubecl-runtime 0.11.0-pre.2`
  depends on `cubecl-environment` with the `cache` feature **non-optionally**
  (a persistent autotune cache), which pulls `rusqlite ^0.40` and thus
  `libsqlite3-sys ^0.38`;
- `p2panda-store`'s `groups` and `encryption` features, which `gemot` needs,
  force its `sqlite` feature, which pulls `sqlx` and thus
  `libsqlite3-sys >=0.30.1, <0.38` — and no released sqlx, including 0.9.0,
  reaches 0.38;
- `libsqlite3-sys` declares `links = "sqlite3"`, so exactly one may exist in a
  graph.

Version selection cannot resolve that. The workspace took **the narrow CubeCL
backport instead**: `cubecl-wgpu` is the only crate in the whole burn 0.21 /
cubecl 0.10 stack that names wgpu, so it is vendored at
`support/patches/cubecl-wgpu` and moved to wgpu 30 (three call sites). Burn
stays at stable 0.21, which is also what this plan's own stop rule wanted.

This plan's B2/B3 work is therefore still pending and still release-gated. When
stable 0.22 lands, the sqlite conflict above must be re-checked before B0
starts: it is an independent blocker that will not clear just because the
release does.

**2026-08-16 (later the same day): the sqlite blocker is CLEARED, and it did
not take a release to clear it.** The re-check this paragraph asks for was run
as an isolated prerelease probe, and the conflict is gone from our side. The
[address book muniment port](../../archive_docs/2026-08-20_completed_plans/2026-08-16_address_book_muniment_port_plan.md)
took sqlx out of the workspace entirely, so `libsqlite3-sys` has no second
claimant. Note also that the diagnosis above was partly wrong: `p2panda-store`'s
`groups`/`encryption` features were never enabled here at all. The real enablers
were `p2panda-net/address_book` and `p2panda-store/default` via p2panda-sync and
p2panda-stream.

Probe result, with `burn = "0.22.0-pre.2"` on both roots and the vendored
`cubecl-wgpu` patch disabled:

- the workspace resolves (exit 0);
- with `esp/index-burn-wgpu` and `quint/field-burn-wgpu` actually enabled, the
  graph carries **exactly one `wgpu`, 30.0.0**, and **exactly one
  `libsqlite3-sys`, 0.38.2** (from `rusqlite`, via `cubecl-environment`'s
  still-non-optional `cache` feature);
- `cubecl-wgpu` comes from the registry at `0.11.0-pre.2`, so **the vendored
  backport at `support/patches/cubecl-wgpu` is no longer needed on this row**.

So the backport retires *with* this migration, not before it: dropping the patch
while staying on burn 0.21 would put wgpu 29 back in the graph. And the
migration is real code work, not a manifest bump. `cargo check -p quint
--features field-burn-wgpu` against 0.22.0-pre.2 fails on API churn:
`burn::backend` and `burn::tensor::backend` have moved, and `Tensor`'s rank
parameter changed (`type provided when a constant was expected`). That is B2/B3.

One thing the probe changes about the sqlite story: adopting 0.22 **reintroduces
an embedded SQLite** to the graph, as `rusqlite` behind CubeCL's autotune cache.
That is CubeCL's own persistence and does not cross mere's storage boundary, but
the workspace should not be described as sqlite-free afterwards.

The probe was reverted; the tree is back on burn 0.21 with the patch in place.

**Related**:
[`../research/2026-07-04_burn_utilization_brief.md`](../research/2026-07-04_burn_utilization_brief.md),
[`2026-07-04_burn_wgpu_flip_plan.md`](2026-07-04_burn_wgpu_flip_plan.md),
[`2026-06-30_mesh_lease_scheduler_plan.md`](../../archive_docs/2026-08-09_completed_plans/2026-06-30_mesh_lease_scheduler_plan.md),
[`2026-08-09_browser_model_ceiling_probe_plan.md`](2026-08-09_browser_model_ceiling_probe_plan.md),
[`2026-08-08_esp_consolidation_plan.md`](2026-08-08_esp_consolidation_plan.md)

This plan moves Mere from Burn 0.21 to stable 0.22 without combining the
dependency migration with remote execution, model-session design, or new
product behavior. Burn Remote becomes a later adapter only after M2/M3 prove
the resource and owner-reclaim seams.

---

## 1. Current dependency boundary

The live workspace has four direct Burn dependents:

- `esp`: the production BERT, vector-kernel, and llama-family model boundary;
- `quint`: the production field/tensor lowering boundary;
- ~~`mere-embed`~~: **removed 2026-08-10 (B1)**; and
- ~~`mere-eidetic-search`~~: **removed 2026-08-10 (B1)**.

The older docs' `aether` and Sibylla/Vates inventory is stale after crate and
ESP consolidation. The actual migration has two production roots, plus two
test/example leaks.

Burn 0.22.0-pre.2 declares Rust 1.95. The checkout currently uses Rust 1.97.1,
so MSRV is not the active gate. Release stability and backend/device API churn
are.

---

## 2. Migration rules

- Preserve model math, public ESP contracts, feature names, default dependency
  floor, and host ownership.
- Migrate one production Burn boundary at a time: ESP first, Quint second.
- Keep CPU correctness anchors while WGPU APIs move.
- Re-run native, wasm, and real-device receipts; prior 0.21 numbers become
  historical rather than silently carrying forward.
- Do not enable Burn Remote default features as a side effect. Its default
  includes client, server, iroh, and websocket.
- Do not expose a Burn 0.22 type from another Mere crate merely to make a test
  compile.
- Do not publish ESP or a compatibility shim from a prerelease migration.

Any required source fork or patch must name the upstream commit, reason, removal
condition, and license. A patch is a temporary compatibility fact rather than a
new Mere-owned backend.

---

## 3. B0: stable-release audit and baseline

When stable 0.22 appears:

1. Record exact versions and feature graphs for `burn`, WGPU support, tokenizer
   dependencies, and `burn-remote`.
2. Read the official migration notes and diff the used APIs: `Backend`,
   `BackendTypes`, `NdArray`, `Wgpu`, device construction/registration, tensor
   creation/readback, module parameter loading, and safetensors paths.
3. Confirm Burn's WGPU dependency still aligns with the workspace's wgpu stack
   or document the isolation boundary.
4. Re-run the 0.21 ESP feature/target matrix, CPU suite, real-WGPU parity, Quint
   tests, and representative performance commands as the baseline.
5. Capture `cargo tree` for every direct dependent and the default ESP tree.

Items 4-5 were captured **ahead of the release** on 2026-08-10, so nothing has
to be reconstructed once APIs have moved:
[Burn 0.21 baseline](../testing/2026-08-10_burn_0_21_baseline.md). It also
records what could *not* be run — the model-backed receipts, because
`MERE_MINILM_DIR` is unset on this machine — so the 0.22 comparison does not
mistake a never-green receipt for a regression. **The migration needs the
MiniLM and TinyLlama checkpoints present**: without them the suites prove
loading and shape, not arithmetic.

Stop if stable 0.22 removes a required browser or existing-device capability.
That becomes an upstream/fork decision before any manifest-wide bump.

---

## 4. B1: reduce accidental dependency fan-out

Before changing versions, remove direct Burn dependencies that exist only so a
consumer can spell an ESP backend type.

Prefer concrete ESP-owned CPU/WGPU construction functions returning the
provider seam over public re-exports of arbitrary Burn modules. Apply them to:

- `mere-embed/tests/bert_full_pipeline.rs`; and
- `mere-eidetic-search/examples/eidetic-recall.rs`.

The exact constructor must be proven by both existing consumers before it is
promoted. If the generic backend API remains materially useful outside tests,
keep the dependency and record why; do not invent a facade solely to reduce a
dependency count.

Done when `cargo metadata` shows only intentional production Burn boundaries,
or each remaining leaf dependency has a concrete consumer justification.

**Done 2026-08-10.** `cargo metadata` now shows two direct dependents, both
production: `esp` and `quint`. There were **three** consumers naming a Burn
type, not two — the third was `mere-embed`'s own lib doctest, which compiles
under `cargo test` and held the dependency open just as firmly. Replaced with
`esp::embed::bert::{load_cpu, load_wgpu, from_bytes_cpu}` returning
`Box<dyn EmbeddingProvider>`, plus `impl EmbeddingProvider for Box<dyn
EmbeddingProvider>` — without which a boxed provider cannot satisfy a generic
bound and a runtime backend choice still needs a Burn type. `BertEmbeddingProvider<B>`
stays public and justified: the constructors take the backend's *default*
device, and a host that must register an existing `wgpu` queue needs the generic
API. Full detail and the 0.21 numbers are in the
[baseline receipt](../testing/2026-08-10_burn_0_21_baseline.md).

---

## 5. B2: migrate ESP

Migrate ESP's shared Burn dependency and all three model families together so
one published crate never contains mixed Burn generations:

- `esp::embed::bert`;
- `esp::embed::index_burn`; and
- `esp::infer::decoder`.

Required receipts:

- empty/default dependency tree remains free of Burn and tokenizers;
- every ESP feature compiles individually on native and wasm;
- combined CPU and browser-WGPU feature sets compile;
- the merged CPU unit suite and Eidetic corridor pass;
- decoder, index, and BERT CPU/WGPU parity pass on real hardware;
- the real TinyLlama checkpoint produces reference-valid output and refreshed
  throughput numbers; and
- `cargo package -p esp` verifies the extracted package without relying on
  workspace patches.

Device initialization deserves its own receipt. Verify both ESP-owned device
creation and any existing-device registration used by a host. A compile through
generic `Wgpu` types is insufficient evidence for interop or queue policy.

No ESP release occurs until Quint and workspace consumers are known to resolve
one compatible Burn graph.

---

## 6. B3: migrate Quint

Quint is an independent, legitimate Burn boundary. Migrate its field lowering
after ESP is green rather than mixing model and field failures.

Required receipts:

- scalar/vector lowering and analytic-gradient tests;
- NdArray/WGPU parity;
- native and wasm feature checks;
- real-device field execution; and
- refreshed representative performance numbers, preserving the CPU-default
  conclusion unless resident/heavier workloads change it.

Gyre and other Burn-free consumers must remain Burn-free. The closure/field
seams survive the migration unchanged.

---

## 7. B4: workspace and publication closure

After both production roots pass:

- update `mere-embed` and Eidetic example consumers;
- inspect the workspace for mixed 0.21/0.22 graphs and accidental remote or
  training features;
- run the ESP package verification from a clean commit;
- update the feature/target matrix and old Burn footprint docs; and
- publish a semver-appropriate ESP release only with explicit authorization.

The deprecated Vates and Sibylla shims receive a version bump only if their ESP
requirement must change. They contain no independent Burn ownership.

---

## 8. Optional prerelease compatibility probe

While only 0.22 prereleases exist, a detached branch/worktree may answer
bounded questions against the newest prerelease:

- how much ESP and Quint source changes;
- whether WGPU and existing-device initialization still work;
- whether `burn-remote` can mount its iroh protocol on an application-owned
  Router; and
- how the authorizer callback maps to a mesh lease reference. (Answered on
  2026-08-10 without needing the probe, by reading the 0.22.0-pre.1 source:
  see [lease-bound remote sessions](../technical_architecture/2026-08-10_lease_bound_remote_sessions.md).
  Note that **`RemoteTicket` does not exist** — this plan and the host lanes
  plan both carried that name in error.)

This probe may use a narrow temporary patch. It must not merge into main,
publish crates, claim stable compatibility, or implement a second scheduler.
Keep only a short diff/receipt if the result changes the stable migration plan.

Burn Remote execution remains a separate lane. Its four entrance conditions
are now satisfied on the chosen prerelease row:

1. M2 has a real resource registry and namespace receipt;
2. M3 has cooperative cancellation and owner reclaim;
3. the explicitly authorized 0.22.0-pre.2 production row is migrated; and
4. the remote adapter can reuse the shared murm iroh endpoint without owning
   job authorization or lease policy.

---

## 9. Non-goals and stop rules

This plan does not add LoRA adapters, `ModelSession`, training, endpoint
inference, mesh economics, or browser product defaults.

Stop on any of these conditions:

- stable 0.22 is not published;
- a required feature exists only behind incompatible WGPU generations;
- model output/parity changes without an explained upstream numerical change;
- default ESP pulls Burn or remote dependencies; or
- the migration requires ESP to own rendering/device policy.

## 10. Done conditions

- One stable Burn 0.22 generation remains in the relevant workspace graph.
- ESP and Quint pass their native, wasm, CPU/WGPU, and real-device receipts.
- Existing-device and browser feature boundaries are re-proven.
- Test/example-only Burn dependencies are removed or justified.
- ESP packages from a clean commit.
- Burn Remote remains a separately versioned and verified resource adapter.
- All refreshed performance claims name the version and hardware.

## 11. Progress

- **2026-08-09**: scoped from the completed ESP consolidation ledger. Verified
  the registry still exposes only Burn/Burn Remote `0.22.0-pre.1`, the local
  toolchain satisfies the advertised Rust requirement, and the live dependency
  surface is ESP plus Quint with two test/example leaks. Separated stable
  migration, accidental-fan-out cleanup, and an optional disposable prerelease
  probe from the later Burn Remote adapter.
- **2026-08-12**: registry recheck found Burn and Burn Remote
  `0.22.0-pre.2`, still with no stable 0.22. The release gate is unchanged.
  Remote API claims elsewhere in the lane remain explicitly sourced to pre.1
  and need a source recheck against the chosen migration version.
- **2026-08-23**: the pre.2 remote source was re-audited; Mere's targeted
  pump-close patch, p2panda exact-ALPN seam, and Distillery lease adapter
  landed. A clean two-peer MiniLM receipt then matched ESP NdArray within
  `1.4901161e-7`, interrupted an in-flight 512-row request on owner reclaim,
  closed the session to zero, and recovered identical output under a new
  lease. Plain WGPU passes; Burn Fusion's remote MiniLM autotune/ordering panic
  is a separate backend sidequest. Stable repinning and package publication
  remain release-gated.
- **2026-08-26**: `0.22.0-pre.3` and the matching CubeCL/Cubek prerelease
  packages are published. A bounded isolated ESP source probe compiled the
  selected BERT/WGPU and index-WGPU rows through Burn, burn-wgpu, burn-cubecl,
  cubecl-runtime, burn-ir, and burn-pack with `-j 1`; temporary manifest pins
  were reverted. Pre.3 contains the two cubecl-runtime packaging fixes, so that
  patch is removable at repin. It does not contain the Mere-owned
  same-allocation burn-cubecl fix or targeted burn-remote pump-close/lifecycle
  control, so both remain required and must be rebased before any repin.
  Remote Fusion/autotune remains unsupported by the existing receipts. Pre.3's
  LoRA dtype/allocation and Burnpack streaming fixes are concrete follow-up
  probes, not production scope. A full Distillery pre.3 check was blocked
  before rustc by a shared genet checkout lock and remains open.

- **2026-09-16 — Mark authorized the `0.22.0-pre.3` repin now, superseding
  the stable-release gate** (ruling in the Knot predicates/inference plan's
  Track 2 thread). The trigger is Knot: its opt-in `embed-bert` feature
  resolved burn to pre.3, and Mark chose to move Mere to match rather than
  hold Knot back. Scope as recorded by the 2026-08-26 audit and the
  2026-09-16 recount: thirteen manifests request pre.2 (conatus, numen,
  seiche, esp, six probes, Distillery's probe crates); the `burn-cubecl`
  same-allocation patch and the `burn-remote` lease-bound close patch are
  still required on pre.3 and must be rebased onto pre.3 source with fresh
  receipts (headed same-allocation and BERT-width LayerNorm for
  `burn-cubecl`); the `cubecl-runtime` packaging patch retires because pre.3
  carries both upstream fixes; Distillery's pre.3 source compatibility is
  still unverified. Remote Fusion/autotune stays out of production defaults.
  The repin runs in an isolated worktree (`Code/worktrees/mere-burn-pre3`)
  with its own target directory, because live sessions build Mere's main
  tree. A grounded execution plan with done-conditions is appended below
  before any manifest moves.

## 12. Pre.3 repin execution plan (2026-09-16)

**Status:** S0 answered 2026-09-16; execution begins in the worktree. Main
has not moved. Grounded by a read-only assessment against the registry sources for
every pre.2 and pre.3 crate involved, with each source rebase checked by
`diff | patch --dry-run`.

### S0 rulings (Mark, 2026-09-16)

- **D1, caret requirements.** Manifests move to `0.22.0-pre.3`,
  `0.11.0-pre.3` and `0.3.0-pre.3` as caret requirements, like today's
  production rows. Consequence recorded, not overridden: a later
  `cargo update -p burn` in any consumer, Knot included, can move burn to
  pre.4 once published while the Mere patches stay pre.3, leaving them unused.
  The tripwire is the standing rule never to scroll past "patch was not used";
  every lock step below keeps that check in its done-condition. Loosening or
  tightening revisits at the stable 0.22.0 repin.
- **D2, `cubek-reduce` in scope.** Rebased with the other three.
- **D3, downloads permitted and done.** `wasm-bindgen-0.2.122-x86_64-pc-windows-msvc.tar.gz`
  (7,556,076 bytes, sha256 `03ba3056eb0c2ad2d0d30a0c0edefb590656a1e237596226804abb87b71b77ad`)
  from the wasm-bindgen GitHub release, extracted to
  `C:	\wasm-bindgen-0.2.122\wasm-bindgen-0.2.122-x86_64-pc-windows-msvc\wasm-bindgen.exe`,
  which reports `wasm-bindgen 0.2.122`. `burn-remote-0.22.0-pre.2.crate`
  (111,186 bytes, sha256 matching crates.io
  `fa8db206a100d838eeb100b8525ff8e3051958ee822fcfecd736fe58eddaeeaa`) extracted
  to `C:	urn-remote-0.22.0-pre.2urn-remote-0.22.0-pre.2`, enabling the
  three-way burn-remote check. Both are under `C:	`, which the 2026-09-15
  reclaim script clears after 24 idle hours; it runs only by hand, so do not
  run it during the repin.
- **D4, coordinate via idle notices.** At S16, message the busy Mere
  sessions, wait for each idle notice, then fast-forward push and tell them
  to pull.

### Phase A progress (2026-09-16)

S1 to S6 done in `Code/worktrees/mere-burn-pre3` on branch `burn-pre3-repin`,
based on `origin/main` at `7db4fe0a` rather than `d15619b4` because main had
moved; the branch's upstream is unset so it cannot be mistaken for main. No
stop rule fired and nothing is pushed.

| Patch | Against pristine pre.3 | Commit |
| --- | --- | --- |
| cubecl-runtime | 4 files, +55: two identity methods, one test, `[workspace]` | `276608d5` |
| burn-cubecl | 5 files, +90 / -3: three launcher blocks at offset 6, manifest tail | `e1c0cb44` |
| burn-remote | 11 files, +583 / -117; `tests/iroh.rs` absent | `31102555` |
| cubek-reduce | 5 files, +273 / -3: helper, two call sites, licenses | `610a32c5` |

cubecl-runtime committed before burn-cubecl so no commit leaves burn-cubecl
calling a missing helper. Each patch's standalone check could not run offline
(dev-dependencies absent from the cache, and burn-cubecl's upstream lock pins a
`cubecl-hip-sys` not cached), so each was compiled as a path dependency of a
throwaway crate under `C:	\mere-burn-pre3-checks` with rustc 1.97.1; all four
built, and `cargo tree` confirmed the worktree patches were used. Mere's two new
burn-remote session-close tests were not compiled; S9 and S10 cover them.

**The three-way burn-remote check closes risk 6.** With pristine pre.2 now on
disk, upstream changed only version pins, packaging metadata, its published
lock and tests between pre.2 and pre.3; all seven server, shared and transport
files Mere edits are byte-identical across the two, and Mere's pre.2 delta
applied to pre.3 with no fuzz or offset.

**Correction, blocking S7 and S8:** Mere has not committed `Cargo.lock` since
`ea9d3169` (2026-06-18, "stop committing Cargo.lock, track latest deps"), so
12.4's "the committed lock already uses genet rev=5ae30cad" and S8's
"regenerate the root lock" have no committed baseline, and S1's `--locked`
check cannot pass as written. The worktree has no lock. The main tree's local,
ignored lock was last written 2026-09-16 11:05 and holds burn pre.2. The
baseline is Mark's decision before S7.

Also found: the cubek-reduce patch carried LICENSE files and a whitespace trim
12.1 did not list (the trim is dropped, pre.3 replaced that file); Mere's
burn-remote tests still call `to_vec`, now deprecated, carried as warnings;
`support/patches/cubecl-wgpu` is a retired 0.10.0 archive nothing patches in,
left alone.

### 12.0 Corrections to the 2026-09-16 recorded scope

1. **The `cubecl-runtime` patch shrinks; it does not retire.** Pre.3 carries
   both packaging fixes (`cubecl-runtime-0.11.0-pre.3/Cargo.toml:98-104`,
   `:206-212`, `:228-229`). But the patch also adds
   `Handle::is_same_allocation` and `ManagedMemoryHandle::is_same_allocation`,
   which the `burn-cubecl` patch calls, and pre.3 exposes no public allocation
   identity (`memory_pool/handle.rs:8` private, `:167` and `:70`
   `pub(crate)`). The identity helpers stay.
2. **A fourth patch, `cubek-reduce`, is in scope.** Four Distillery probe
   manifests patch it, including the `burn_browser_embedding` repro that
   produces the same-allocation receipt. Pre.3 still builds infinity from
   literal bits (`cubek-reduce-0.3.0-pre.3/src/components/instructions/extrema.rs:24`,
   `:34`).
3. **Ten manifests request pre.2, not thirteen.** The six `crates/probes/*`
   crates are standalone workspaces on burn 0.21 (excluded at
   `Cargo.toml:175-176`) and are out of scope. Seven lockfiles regenerate.
4. **Knot must patch `cubecl-runtime` as well as `burn-cubecl`**, or Mere's
   `burn-cubecl` does not compile inside Knot (12.8).

### 12.1 The patches against pre.3

Upstream commits: `burn-cubecl` and `burn-remote` pre.3 `13f0a12b`;
`cubecl-runtime` pre.3 `b0d2e688`; `cubek-reduce` pre.3 `73743e34`.

**`burn-cubecl`.** Mere's delta: in `launch_binop`, `launch_binop_float` and
`launch_binop_int`, a block before `if lhs.can_mut_broadcast(&rhs)` that, when
both inputs share an allocation, creates a separate output, binds storage once
and passes `rhs.as_linear_view_alias(0)`; plus `[workspace]` and a
`cubecl-runtime` path patch in its manifest. Pre.3 added a zero-size early
return above the anchor in all three files; the anchors are otherwise
unchanged (`binary.rs:251`, `binary_float.rs:78`, `binary_int.rs:145`),
`tensor/base.rs` is byte-identical, and `dtype_to_storage_type` now returns
`ElemType` where `#[define(C)]` expects it. All three hunks apply at offset 6.
None of the fix is upstream. The receipt still exercises it: pre.3's default
`layer_norm` still calls `B::float_mul(centered.clone(), centered.clone())`
(`burn-backend-0.22.0-pre.3/src/backend/ops/modules/base.rs:859`).

**`cubecl-runtime`.** The `memory_pool/handle.rs` hunk applies cleanly. The
`server/handle.rs` hunk fails only on trailing context because pre.3 renamed
`Binding` to `BufferBinding` (`:73`); insert the method by hand after
`can_mut()` (`:69-71`), every field it reads still exists (`:10-21`). The
manifest change reduces to `[workspace]`. Add a `MERE-PATCH.md`.

**`burn-remote`.** Pre.2 source is not on disk (not in registry `src` or
`cache`; the first vendoring commit `d210519e` already held Mere's changes), so
the patch was diffed against pre.3 and every hunk accounted for: version
bumps; upstream test renames (`to_vec` to `try_to_vec` in `lib.rs` tests and
`tests/iroh.rs`); and Mere's lease-bound close work across `Cargo.toml`
(`server` adds `tokio/macros`, restored dev-dependencies, self `[patch]`,
`[workspace]`), `lib.rs` (visibility plus two tests), `server/mod.rs`,
`server/pump.rs`, `server/service/mod.rs`, `server/session.rs`,
`server/worker.rs`, `shared/mod.rs`, `transport/iroh/protocol.rs`. Upstream
changed only tests between pre.2 and pre.3; pre.3's `close()` still only
removes the map entry. Rebase: pristine pre.3, copy Mere's seven non-`lib.rs`
source files, apply Mere's `lib.rs` hunks keeping `try_to_vec`, re-apply the
four manifest edits with dev-dependencies at `=0.22.0-pre.3`. Residual risk:
an upstream change on exactly the lines Mere replaced would be hidden; low,
and the two-peer remote receipt is the behavioural check. A three-way check
would need downloading `burn-remote-0.22.0-pre.2.crate`.

**`cubek-reduce`.** Mere's delta: a `runtime_f32_from_bits` helper passing
bits through a mutable local at both identity sites, plus `[workspace]`. Hunk 1
fails only on pre.3's `type_of` to `elem_type_of` rename; hunk 2 applies with
fuzz. Re-apply by hand at pre.3 `extrema.rs:19-37`. The real risk is
behavioural: pre.3 rewrote CubeCL's IR and WGSL backend (`cubecl-opt` 30 to 12
files, `cubecl-wgpu` 30 to 42) and `extrema.rs` now uses `IsNanOp`. Only the
headed extrema receipt decides whether the trick still yields a runtime value.

### 12.2 Manifests and API

| File:line | Change |
| --- | --- |
| `crates/conatus/conatus/Cargo.toml:18-20` | cubecl to 0.11.0-pre.3; burn, burn-wgpu to 0.22.0-pre.3 |
| `crates/conatus/numen/Cargo.toml:20-21` | burn, burn-wgpu |
| `crates/conatus/seiche/Cargo.toml:34-35` | burn, burn-wgpu |
| `crates/intel/esp/Cargo.toml:25` (comment `:71`) | burn |
| `ports/distillery/Cargo.toml:36-38, 59` | burn-backend, burn-ir, burn-remote, burn-flex (exact pins) |
| `ports/distillery/probe/Cargo.toml:21` | burn |
| `ports/distillery/probe/native-fixture/Cargo.toml:12` | burn |
| `ports/distillery/probe/remote-fixture/Cargo.toml:12-14` | burn, burn-wgpu, cubecl |
| `ports/distillery/probe/repros/burn_browser_embedding/Cargo.toml:19, 27` | burn (twice) |
| `ports/distillery/probe/repros/cubek_browser_extrema/Cargo.toml:19` | burn; drop the `cubecl-runtime` row at `:27` (packaging-only) |

Lockfiles to regenerate: root, `probe`, `native-fixture`, `remote-fixture`,
`session-fixture`, and both repros. Root `[patch]` rows at `Cargo.toml:711,
718, 725` stay; rewrite their comments (`:706-725`).

**API.** No hard breaks found in what Mere uses. esp: `Device::{ndarray, wgpu}`,
`DeviceKind`, `autodiff()`, `into_data_async`, `into_scalar_async`, the
`burn::nn` layers and `burn::optim::{AdamConfig, GradientsParams::from_grads}`
unchanged. conatus: burn-wgpu `lib.rs` byte-identical; `WgpuSetup`,
`init_device`, `RuntimeOptions`, `CubeTensor::new_contiguous`,
`ComputeClient::{create_from_slice, get_resource, read_one}`,
`BufferArg::from_raw_parts`, `WgpuResource`, `Shared::new_slice` unchanged;
`burn::backend::wgpu` still exists through `burn-dispatch`. Distillery:
burn-remote's public API unchanged, burn-ir only gained items.

New deprecation warnings, not errors: `TensorData::to_vec` and `into_vec`
(about 60 call sites across esp, numen, seiche, Distillery), and
`Device::ndarray()` announcing burn-ndarray's removal. Do not switch the CPU
reference backend inside this repin; log it as a follow-up.

Not yet built on pre.3: esp's `decoder-lora`, `decoder-autodiff`,
`model-session`, `persistence` rows. Pre.3 changed burn-core's `module/lora`
and `param/*` and burn-optim's `state` and `visitor`.

MSRV stays 1.95, toolchain 1.97.1, one wgpu `^30`, one `libsqlite3-sys`.

### 12.3 Receipts and how they are produced

- **Same-allocation and BERT-width LayerNorm (headed).**
  `ports/distillery/probe/repros/burn_browser_embedding/run-repro.ps1`, built
  outside the checkout with `cargo build --locked --release --target
  wasm32-unknown-unknown`, wasm-bindgen CLI **0.2.122** required, served by
  `python -m http.server`; in headed Chromium run the graph cases or
  `window.burnEmbeddingRepro.run()`. Prior receipt
  `receipts/2026-08-22_binary_alias_iab.json`. Native control
  `shared_binary_and_layer_norm_pass_native_wgpu` (`src/lib.rs:674`).
- **Extrema (headed).** `repros/cubek_browser_extrema/run-repro.ps1`, then
  `window.cubekExtremaRepro.run()`. Prior receipt
  `receipts/2026-08-22_patched_iab.json`.
- **burn-remote lifecycle.** `ports/distillery/src/remote.rs` tests
  `a_live_lease_runs_on_the_shared_endpoint_and_reclaim_ends_the_client`
  (`:413`) and `distillery_closes_the_session_before_authoring_owner_reclaim`
  (`:606`); two-peer run `ports/distillery/probe/run-remote-minilm.ps1`.
  Prior receipt `receipts/2026-08-23_remote_minilm.json`.
- **Existing-device.** `cargo test -p conatus --release --features resident
  --test resident --test resident_chunk`: nine tests.
- **Tooling gap.** No wasm-bindgen 0.2.122 CLI on this machine;
  `~/.cargo/bin/wasm-bindgen.exe` is 0.2.126, which the scripts reject because
  versions from 0.2.123 break wgpu 30's `popErrorScope`
  (`probe/Cargo.toml:24-26`).

### 12.4 Isolation

- Worktree `Code/worktrees/mere-burn-pre3` on branch `burn-pre3-repin` from
  `d15619b4`. Mere's gitignored `.cargo/config.toml` source redirects are not
  inherited, so the worktree resolves exactly what is committed; the committed
  lock already uses genet `rev=5ae30cad`, whose checkout exists.
- Every step: `CARGO_TARGET_DIR=C:\t\mere-burn-pre3-target`,
  `CARGO_NET_OFFLINE=true`; scripts get `-TargetDir C:\t\mere-burn-pre3-<name>`.
- The 2026-08-26 Distillery check was blocked by a git fetch holding the
  package cache lock. Offline, Cargo never fetches git. Every pre.3 burn,
  cubecl and cubek crate in the graph is already cached, except
  `burn-communication`, used only by burn-remote's own websocket tests. Never
  run `cargo generate-lockfile` or a bare `cargo update`. Fallback: a copied
  `CARGO_HOME` under `C:\t`, offline.
- Models are gitignored; pass `-ModelDir` or `SIBYLLA_MINILM_DIR` pointing at
  the main tree's `models/all-MiniLM-L6-v2`.
- `-j 4` as a compromise with live sessions.

### 12.5 Steps and done-conditions

- **S0. Decisions (Mark).** D1 exact or caret pins; D2 `cubek-reduce` in
  scope; D3 permission to install wasm-bindgen-cli 0.2.122 and for any
  one-off fetch; D4 the merge window. *Done:* answered, and the 0.2.122 CLI
  prints its version.
- **S1. Worktree.** `git -C repos\mere worktree add -b burn-pre3-repin
  Code\worktrees\mere-burn-pre3 d15619b4`. *Done:* no `.cargo\config.toml` in
  the worktree; `cargo tree --offline --locked -p esp --features bert-wgpu -e
  normal --depth 1` exits 0 on the pre.2 baseline.
- **S2. Capture deltas.** `diff -u` pristine pre.2 against each patch into
  `C:\t\mere-burn-pre3-deltas`; burn-remote against pre.3. *Done:* four diffs
  matching 12.1.
- **S3. Rebase burn-cubecl.** *Done:* against pristine pre.3 the diff is three
  17-line insertions, the manifest tail and `MERE-PATCH.md`.
- **S4. Rebase cubecl-runtime to identity-only.** *Done:* diff is two methods,
  one test and `[workspace]`.
- **S5. Rebase burn-remote.** *Done:* diff lists only `Cargo.toml`,
  `Cargo.toml.orig`, `MERE-PATCH.md`, `src/lib.rs` and the seven server,
  shared and transport files; `tests/iroh.rs` absent.
- **S6. Rebase cubek-reduce.** *Done:* diff is one helper, two call sites and
  `[workspace]`.
- **S7. Move the manifests** and rewrite root `Cargo.toml:706-725` comments.
  *Done:* `git grep` for the three pre.2 versions in manifests matches only
  comments and `support/patches/*/Cargo.toml.orig`.
- **S8. Regenerate the root lock, offline and targeted:** `cargo update
  --offline -p burn -p burn-wgpu -p cubecl -p burn-backend -p burn-ir -p
  burn-remote -p burn-flex`. *Done:* no `pre.2"` in the lock; the three
  patched crates at pre.3 with no `source`; no "patch was not used"; the lock
  diff touches only the burn, cubecl and cubek families and their needs; one
  wgpu 30.x; esp with no default features has no burn and no tokenizers.
- **S9. Build and test.** Native: esp default, `bert,bert-validation`,
  `index-burn-wgpu`, `decoder-wgpu`, `decoder-lora,decoder-autodiff`; numen
  `field-burn`; seiche `tensor-burn`. GPU release: numen `field-burn-wgpu`,
  seiche `tensor-burn-wgpu`, conatus `resident`, esp BERT wgpu parity against
  real MiniLM. wasm, each `--target wasm32-unknown-unknown
  --no-default-features`: esp's eleven matrix rows; numen `""`, `field-burn`,
  `field-burn-wgpu`, `field-rhai`; seiche `""`, `tensor-burn`,
  `tensor-burn-wgpu`; conatus `resident`. *Done:* every suite passes, conatus
  nine with no missing-adapter lines, BERT wgpu parity under `2.0e-3`, every
  wasm row exits 0. Ambiguous failures rerun on a detached pre.2 worktree.
- **S10. Distillery.** `cargo check --offline -p distillery --all-targets
  --features remote,trainer-gpu,trainer-autodiff,flora`; `cargo test --offline
  -p distillery --features remote --lib remote::tests`; `cargo check --offline
  -p djinn --features trainer-gpu,trainer-autodiff`. *Done:* all exit 0, both
  lease tests pass. This is the missing Distillery pre.3 receipt.
- **S11. Whole workspace.** `cargo check --offline --workspace --all-targets`.
  *Done:* exit 0, or any failure is outside burn's graph and fails identically
  on `main`.
- **S12. Nested lockfiles** for `probe`, `native-fixture`, `remote-fixture`,
  `session-fixture` and both repros, offline and targeted. *Done:* no pre.2
  entries, no unused-patch warnings; `remote-fixture` keeps its older p2panda
  0.7.3 pins. Commit patches, manifests and locks before receipts so receipts
  record a clean commit.
- **S13. Receipts.**
  - (a) Same-allocation and LayerNorm, headed, into
    `receipts/<date>_binary_alias_pre3_iab.json`, plus the native control.
    *Done:* `passed: true`, eleven embedding controls pass, four graph cases
    `matches_expected: true`, `gpu_errors: []`.
  - (a') Unpatched control: temporarily drop only the `burn-cubecl` and
    `cubecl-runtime` rows, build into a separate target, record, restore.
    *Done:* `burn-unit-raw-mul-shared` false and both LayerNorm cases
    `output_matches_input_bits: true`, proving the patch is still needed.
  - (b) Extrema, headed. *Done:* four cases pass, `gpu_errors: []`.
  - (c) Two-peer remote run. *Done:* receipt `dirty=false`; 384 values match
    ESP native within the pre.2 receipt's tolerance (pre.2 recorded
    1.4901161e-7); the 512-row in-flight request fails on reclaim without
    hanging; CubeCL allocations and bytes return to baseline; a fresh lease
    reproduces the output. Fusion/autotune matrix not required.
  - (d) conatus and (e) esp wgpu parity outputs from S9, recorded.
- **S14. Commits on the branch:** one per patch; manifests and locks; receipt
  files; docs. No attribution trailers. Pushing is S16.
- **S15. Docs:** this plan's status lines and a progress entry; the closure
  doc's production row and patch table; `MERE-PATCH.md` in all four patches;
  esp's manifest comment; the probe and repro READMEs; the feature/target
  matrix. *Done:* no current-state doc says pre.2 is production.
- **S16. Merge back.** Rebase onto `origin/main`; on a `Cargo.lock`-only
  conflict take `origin/main`'s lock and rerun S8's targeted update; smoke
  `esp --features bert-wgpu`, `distillery --features remote`, conatus
  `resident_chunk`; fast-forward push to `main` with Mark's authorization in
  the D4 window; record the pushed revision R. Do not merge into the main
  working tree beneath a live session. *Done:* `origin/main` is R and a doc
  commit records it.
- **S17. Knot handoff** (12.8).

### 12.6 Stop rules (halt and return to Mark)

1. A patch needs different logic, not new context: pre.3 changed
   `can_mut_broadcast`, `Handle`'s fields, or `SessionManager`'s structure.
2. The unpatched control (S13 a') passes the shared-input cases: the patch
   may be removable, and carrying or dropping it is Mark's call.
3. A patched same-allocation or extrema case fails, or `gpu_errors` is
   non-empty.
4. The remote run hangs on reclaim, allocations miss baseline, or numbers
   differ from the pre.2 receipt without an upstream explanation.
5. Any numeric or parity result moves without an explained upstream change
   (repeats §9).
6. A compile fix would change an esp public contract, a feature name, device
   ownership, or the burn-remote patch API. Mechanical renames and
   deprecation warnings are not stops.
7. Lock regeneration moves wgpu off 30.x, adds a second burn, cubecl, wgpu or
   `libsqlite3-sys`, or moves unrelated majors (iroh, tokio, p2panda).
8. esp with no default features pulls burn or tokenizers.
9. Rebasing onto `main` conflicts in a manifest.
10. Cargo insists on a network git fetch while offline outside D3.

### 12.7 Risks, highest first

1. CubeCL's pre.3 IR and WGSL rewrite changes browser behaviour: the extrema
   trick may stop working, and the same-allocation defect may worsen or
   vanish. Only S13 a, a' and b decide it.
2. Knot carries a patch that is unused or does not compile, if
   `cubecl-runtime` is not patched with `burn-cubecl` or caret pins drift to
   pre.4 (D1).
3. Headed tooling: the 0.2.122 CLI is absent, newer CLIs break wgpu 30 error
   scopes, and a headed Chromium session is required.
4. Live sessions: cache locks, `Cargo.lock` conflicts at merge, and every
   session rebuilding the burn stack afterwards.
5. esp's LoRA and autodiff rows never built on pre.3.
6. burn-remote pre.2 source unavailable; the Mere-only finding rests on hunk
   accounting.
7. Deprecation noise and burn-ndarray's announced removal: follow-up.
8. Older mismatches (remote-fixture on p2panda 0.7.3) may surface.

### 12.8 Knot handoff

After S16, `origin/main` at R must contain `support/patches/burn-cubecl`
(`0.22.0-pre.3`) and `support/patches/cubecl-runtime` (`0.11.0-pre.3` with
the identity helpers). Knot needs both because `burn-cubecl` calls
`Handle::is_same_allocation`, and the `[patch]` inside `burn-cubecl`'s own
manifest is ignored when it is a dependency. Each patch declares
`[workspace]`, so Cargo loads it from a git checkout, and only these four
paths declare those package names. Knot's root `[patch.crates-io]` gains:

```toml
burn-cubecl    = { version = "0.22.0-pre.3", git = "https://github.com/merely-made/mere.git", rev = "<R>" }
cubecl-runtime = { version = "0.11.0-pre.3", git = "https://github.com/merely-made/mere.git", rev = "<R>" }
# only if Knot ever runs BERT on BrowserWebGpu:
# cubek-reduce = { version = "0.3.0-pre.3", git = "https://github.com/merely-made/mere.git", rev = "<R>" }
```

All of Knot's Mere pins move to R together. *Done in Knot:* no unused-patch
warning, and `cargo tree -p knot-editor --features embed-bert-wgpu -i
burn-cubecl` shows the git source at R. Note for Knot's plan: Knot is
desktop-only today and Mere's native WGPU control passes without the patch;
the patch protects a future browser build.

### 12.9 Effort

Mechanical: S1 to S5, S7, S8, S12, S14, S15, S17. About half a day of focused
work, mostly build waits. Investigation: S6 and S13 b (extrema on the new IR),
S9's LoRA and autodiff rows, S10 Distillery, S13 a and a' (headed tooling and
interpreting the control), S16 timing. Machine time roughly half a day at
`-j 4` into a cold target, a full day at `-j 1`. Total about one to one and a
half working days with no stop rule firing; add half a day to two days if the
extrema or same-allocation receipts behave differently on pre.3.
