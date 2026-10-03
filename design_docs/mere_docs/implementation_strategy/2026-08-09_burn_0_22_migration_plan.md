# Burn 0.22 Migration Plan

**2026-10-03 main reconciliation and P5 gates (§13.30):** main `a924f380`
and `f62581c7` are merged (`bcb57356`, `b73695da`), with conatus's P5 lanes
adapted to pre.4. Root and web graphs hold one pre.4 family and no Turso.
Native GPU gates pass, S13 (b) extrema passes headed, and the P5 2,000-node
web settle meets its bounds. Two stops are held as forks. S13 (c) still
leaves 10 allocations against zero (ruling 411's unanswered repair fork)
(answered by ruling 508).
The pre.4 wasm bundle also runs about fifty times slower per frame than
pre.2, because its static constructors re-run on each call. No push, main
promotion or downstream repin.

**2026-09-30 allocator diagnosis, ruling 411:** four `burn-cubecl` selectors
were retired locally at `124fc42b` after ruling 410's controls. Remote reclaim
then left ten active allocations / 5,323,776 bytes against zero. A bounded
diagnostic releases them after explicit GPU completion polling; this is not a
repaired lifecycle pass. The zero baseline remains. A patch-design fork now
asks where completion belongs. Browser extrema, migration acceptance, main
promotion and downstream repins remain held. See §13.29.

**2026-09-29 S13 stop:** native and patched browser comparisons pass, but
pristine upstream pre.4 also passes all ten graph and eleven embedding cases.
Stop rule 13.8(2) applies: carrying or retiring the patch awaits Mark's ruling.
Exact source restoration is verified; extrema, remote lifecycle and main
promotion remain held. See §13.26; earlier dated checkpoints retain their scope.

**2026-09-29 reconciliation verification:** the held pre.4 branch now includes
the accepted scroll/Insigne and semantic-projection sources through Mere
`99e44853`, using current Genet `19c20687`. Fresh graph, workspace, host,
remote-build, Wasm and affected numerical checks pass; see §13.25. Only Numen
carries its earlier execution proof. S13, migration acceptance and main
promotion remain held. Earlier dated entries retain their historical scope.


**2026-09-28 reconciliation checkpoint:** merge of committed main `5ce144ff`
into Lane M `a7c477e7` is prepared but uncommitted. Graph, workspace, Distillery,
Djinn, remote-fixture and web checks pass after three optional-field API
adoptions. A rootstock scroll assertion fails identically on untouched main;
acceptance and S13 remain held. See §13.24. This dated status supersedes the
earlier continuation banners without changing their historical words.


**2026-09-27 continuation:** remaining Conatus/ESP/Numen GPU and all eight S12 builds pass at the source-qualified `8d308572` checkpoint plus a remote-fixture API adapter; see §13.23. Current pins and seven prepared files are preserved; review precedes their commit.

**2026-09-27 carry update:** ruling 380 pre.2 repair is published; its bounded pre.4 carry passed direct/control, identity and full-force gates (§13.22). Broader migration acceptance remains open.
**2026-09-27 ruling 380:** the separate pre.2 alias-broadcast repair passed source/control/full-force review; pre.4 carry awaits release. See §13.21.
**Date**: 2026-08-09

**Localization annotation (2026-09-27):** the bounded displacement capture
finds the mismatch before later force arithmetic. Static comparison identifies
a missing broadcast reference in our existing aliased RHS view, present in
both patched pre.2 and pre.4. §13.20 records the candidate and evidence limits;
implementation remains stopped pending the repair-order decision.

**Baseline comparison annotation (2026-09-27):** the same hardened Seiche
release gate also fails on preserved pre.2 with the same two assertions and
reported error. §13.19 records the bounded comparison and exact restoration.
The observed failure predates migration; numerical acceptance remains open.

**Numerical stop annotation (2026-09-27, S9–S12):** the nine native and
19 wasm rows, Distillery, Djinn and portable whole-workspace checks pass.
Numen's release GPU suite passes, but Seiche's two GPU parity tests fail with
finite output under unchanged thresholds. The sequence stopped before Conatus
and ESP release gates. §13.18 records the exact failure, controls and resolved
nested locks; migration acceptance and main integration remain blocked.

**Continuation annotation (2026-09-27, S5–S8):** the coordinator accepted the
guard checkpoint and released the remaining patch rebases, exact consumer
pins, root lock and mechanical §13.4 adapters in the existing Lane M worktree.
The root production lock there is pre.4; main remains pre.2. Source review,
portable metadata and the Conatus resident compile pass. §13.17 records this
bounded checkpoint; broader matrices, nested lock migration, headed receipts,
two-peer lifecycle acceptance and main integration remain pending.

**Checkpoint annotation (2026-09-27, after ruling 378):** the pre.2 fixture
prerequisite and bounded pre.4 guard checkpoint are verified (§13.16).
Nine guard tests pass; removing the service comparison makes both service
tests fail, and byte restoration returns all nine to passing. Root/product
manifests and the production lock remain pre.2. S5–S18 and consumer
persistence-absence acceptance remain pending coordinator review.

**Status annotation (2026-09-27, ruling 378):** needed crates.io downloads
are authorized with versions/integrity recorded and Git pins preserved.
The original offline failure in §13.14 remains evidence; §13.15 records
resumption. The bounded guard checkpoint is in progress, with root/product
manifests still on pre.2 and broader migration acceptance still pending.

**Status annotation (2026-09-27, rulings 375–377):** production remains on
`0.22.0-pre.2`; no production manifest or lock has moved. Pre.4 will disable
CubeCL persistence with a manifest-only runtime patch and require the complete
Distillery gates. The allocation guard will compare six values, including
`Handle.service`, with a rejection control. Migration has not begun; the
coordinated baseline and first implementation checkpoint remain to be prepared.
§13.13 overrides the earlier S0 status and execution
paths below; their dated wording remains as evidence.

**Status (2026-09-27):** production is still `0.22.0-pre.2`. The pre.4
exact-pin repin is planned in §13, with S0 open and no manifest moved; the
pre.3 repin in §12 is superseded. The 2026-08-20 status follows.

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

- **2026-09-27 — Mark ruled the pre.4 repin (353–356, Isometry wing design
  record).** Burn `0.22.0-pre.4`, CubeCL `0.11.0-pre.4` and cubek
  `0.3.0-pre.4` were published together on 2026-09-22, still with no stable
  0.22. Exact pins supersede the parked pre.3 repin and D1. The
  `cubecl-runtime` patch retires, because pre.4 exposes allocation identity.
  turso, which pre.4 brings in through CubeCL's default `persistence`, is
  settled inside this migration. Lane H's scratch assessment found conatus
  the only Mere crate that fails (nine mechanical errors), and Distillery's
  lease tests passing on a rebased burn-remote. §13 is the execution plan; S0
  is open and no manifest has moved.

- **2026-10-03**: Mark's "Finish pre.4 now, no interim pin" (physics
  catalog plan, ruled 2026-10-02) is carried to its gates on this branch.
  §13.30 records two main merges, 169 checksum-verified downloads under
  ruling 378, the native, headed and graph gates, and two forks held: the
  ruling 411 repair, and the pre.4 wasm constructor cost.

## 12. Pre.3 repin execution plan (2026-09-16)

**Superseded 2026-09-27 by ruling 353** (pre.4, exact pins; §13). This
repin never merged. Its branch `burn-pre3-repin` stays parked with four
rebase commits (`276608d5`, `e1c0cb44`, `31102555`, `610a32c5`), which guide
§13's rebases, and D1's caret requirements no longer hold. The rest of this
section records how the pre.3 attempt was grounded and where it stopped.

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
  `C:\t\wasm-bindgen-0.2.122\wasm-bindgen-0.2.122-x86_64-pc-windows-msvc\wasm-bindgen.exe`,
  which reports `wasm-bindgen 0.2.122`. `burn-remote-0.22.0-pre.2.crate`
  (111,186 bytes, sha256 matching crates.io
  `fa8db206a100d838eeb100b8525ff8e3051958ee822fcfecd736fe58eddaeeaa`) extracted
  to `C:\t\burn-remote-0.22.0-pre.2\burn-remote-0.22.0-pre.2`, enabling the
  three-way burn-remote check. Both are under `C:\t`, which the 2026-09-15
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
throwaway crate under `C:\t\mere-burn-pre3-checks` with rustc 1.97.1; all four
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

### Phase B, first attempt and rulings (2026-09-16)

Phase B stopped at its first check, as designed. Main's local lock (sha256
`06a159dc...`, written 11:05:36) was copied as the baseline, and S1's
`--locked` check exited 101 on the branch and, in a throwaway worktree, on
pre.2 itself. Cause: main's lock was written under the gitignored
`.cargo/config.toml` redirects, so about 36 genet, netrender and retinue
packages are recorded as local paths, and four path-only unused patch rows
(outrider, postilion, radio-hand, tulle) appear. A copy Cargo rewrote once
offline changed no package version (burn, burn-cubecl, cubecl-runtime,
burn-remote, cubek-reduce, wgpu 30.0.1, libsqlite3-sys, tokio, iroh and
p2panda unchanged), moved those sources to git at the same pinned revisions
(genet `5ae30cad`, netrender `3961aca9`, retinue `85e716c7`), and passed
`--locked` at pre.2.

**Rulings (Mark, 2026-09-16).**

- **Baseline:** the normalized lock,
  `C:\t\mere-burn-pre3-baseline-normalized-pre2.lock`, sha256
  `6055022d3433481918b70a86243a8e3fca10a99bfc2c3a43c443d32916657f88`. It is
  what a clean checkout of committed Mere resolves, which is what the
  worktree builds.
- **Stop rule narrowed:** an unused-patch warning stops the repin only for
  `burn-cubecl`, `cubecl-runtime`, `burn-remote` or `cubek-reduce`. Pre-existing
  unused patches are recorded in the receipt as present before the repin.
- **Open defect, deferred until after the repin:** under the committed-state
  resolution, Mere's main already reports `patch was not used` for
  `boa_engine` and `boa_gc` (`Cargo.toml:788-789`, `mark-ik/boa` branch
  `genet`) and `iroh-mdns-address-lookup` (`Cargo.toml:756`, `=0.4.0` on
  `mark-ik/iroh-address-lookups` branch `mere`). Main believes it builds those
  forks and does not. Diagnosis waits so the repin's lock diff stays free of
  unrelated movement; it gets its own home when opened.

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

## 13. Pre.4 repin execution plan (2026-09-27)

**Later 2026-09-27 annotation:** read §13.13 before executing this section.
Rulings 375–377 settle persistence, the Distillery gate and the guard; the original
snapshot, questions and measurements below retain their words.

**Status (2026-09-27):** plan; S0 open. No manifest has moved. Branch
`burn-pre4-repin` in `Code/worktrees/mere-burn-pre4`, from `origin/main` at
`a464dc2a` with its upstream unset, carries only this section and the notes in
§11 and §12. Grounded by Lane H's scratch assessment of pre.4 (2026-09-26, no
repository edits) and a re-check against `a464dc2a`: registry sources for every
pre.2, pre.3 and pre.4 crate involved, the parked pre.3 branch read through git
only, `patch --dry-run` of each pre.3 delta onto pristine pre.4, and resolution
experiments in throwaway copies of the tree. Code here is illustrative.

### Rulings (Mark, 2026-09-27, Isometry wing design record)

- **353, "Pre.4, exact pins."** `=0.22.0-pre.4` for the burn family,
  `=0.11.0-pre.4` for cubecl, `=0.3.0-pre.4` for cubek. Supersedes the parked
  pre.3 repin and D1's caret requirements. The pre.3 branch's commits
  (`276608d5`, `e1c0cb44`, `31102555`, `610a32c5`) guide the rebases.
- **354, "No stopgap"** in Isometry's isometer-lens. It adapts when it repins to
  this repin's revision, not before (13.10).
- **355, "Retire it."** The `cubecl-runtime` patch retires. burn-cubecl's
  same-allocation guard compares pre.4's public, doc-hidden allocation id,
  `Handle.memory.descriptor().id`, directly. Knot's need to patch
  `cubecl-runtime` goes with it.
- **356, "Settle it in the migration."** Pre.4 brings turso in through
  CubeCL's default `persistence` feature. Whether that can be turned off is
  answered in 13.3; nothing about it is committed before Mark rules.

### 13.0 What changed since §12

1. **The root lock is tracked again.** `68f78873` (2026-09-20, "Separate local
   and portable Cargo graphs") made the root `Cargo.lock` the portable lock,
   gated by `cargo check --workspace --locked` and `python
   scripts/cargo_mode.py verify` (README, "Portable and local dependencies").
   §12's Phase A correction and the 2026-09-16 baseline ruling rested on there
   being no committed lock. The ruled file still exists unchanged
   (`C:\t\mere-burn-pre3-baseline-normalized-pre2.lock`, sha256 `6055022d…`,
   re-hashed 2026-09-27), but it fails `--locked` against `a464dc2a`'s
   manifests (exit 101): it records genet `5ae30cad`, netrender `3961aca9` and
   knot-editor `ec97d458`, all of which main has left. The committed lock passes
   the same command. S0-4.
2. **Toolchain 1.98.1** (`rust-toolchain.toml`). Pre.4's MSRV stays 1.95.
3. **The root patch rows moved:** `cubecl-runtime` to `Cargo.toml:663` (comment
   658–662), `burn-cubecl` to `:670` (664–669), `burn-remote` to `:677`
   (671–676).
4. **Knot never took 12.8, and D1's drift happened there.** Knot at `d265f03`
   (djinn's pin) and at Knot's `main` (`7d78141`) resolves `burn`,
   `burn-cubecl` 0.22.0-pre.3 and `cubecl-runtime` 0.11.0-pre.3 from the
   registry, and its root `[patch.crates-io]` names no burn crate. Exact pins
   close this: once Knot's mere pin passes this repin, its lock must take pre.4.
5. **remote-fixture has not resolved since `0db9d10d`** (2026-09-14, "Pin
   mere-p2panda-net 0.7.4"). That commit moved the root patch table and five
   member manifests. `crates/mesh/mesh/Cargo.toml:43` now requires
   `mere-p2panda-net = "=0.7.4"`, which exists only as the git tag the root
   table names (`Cargo.toml:692`). remote-fixture's own `[patch.crates-io]`
   still names tag `mere-p2panda-net-0.7.3` and has no `mere-p2panda-net` row,
   so it fails identically on pre.2 and pre.4 (Lane H's `rf-pre2.err`,
   `rf-pre4.err`). §12's S12 note that it "keeps its older p2panda 0.7.3 pins"
   was already stale when written, and S13 (c) could not have run on pre.3.
   S0-6.
6. **Five `C:\t` paths in §12 were byte-corrupted:** a TAB and a backspace had
   replaced `\t` and `\b` in D3, Phase A and Phase B. They are repaired in the
   commit that adds this section; no other §12 text changed.

### 13.1 Scope

Recounted at `a464dc2a` by `git grep` for the three pre.2 versions over every
tracked `Cargo.toml` outside `support/patches`, then checked against reverse
dependencies in a pre.4 scratch resolve (only `conatus`, `esp`, `numen` and
`seiche` depend on `burn`; only `distillery` on `burn-remote`). Ten manifests,
the same set as §12.0 item 3. Every burn, cubecl and cubek requirement in them
becomes exact; today only Distillery's four rows and remote-fixture's
`burn-wgpu` and `cubecl` are.

| File | Lines | Change |
| --- | --- | --- |
| `crates/conatus/conatus/Cargo.toml` | 18–20 | cubecl `=0.11.0-pre.4`; burn, burn-wgpu `=0.22.0-pre.4` |
| `crates/conatus/numen/Cargo.toml` | 20–21 | burn, burn-wgpu |
| `crates/conatus/seiche/Cargo.toml` | 34–35 | burn, burn-wgpu |
| `crates/intel/esp/Cargo.toml` | 25 (comment 70–73) | burn |
| `ports/distillery/Cargo.toml` | 35–37, 58 | burn-backend, burn-ir, burn-remote, burn-flex |
| `ports/distillery/probe/Cargo.toml` | 21; patch 36–39 | burn; drop the `cubecl-runtime` row (38) |
| `ports/distillery/probe/native-fixture/Cargo.toml` | 12 | burn |
| `ports/distillery/probe/remote-fixture/Cargo.toml` | 12–14; patch 31–40 | burn, burn-wgpu, cubecl; drop `cubecl-runtime` (34); p2panda rows per S0-6 |
| `ports/distillery/probe/repros/burn_browser_embedding/Cargo.toml` | 19, 27; patch 29–32 | burn twice; drop `cubecl-runtime` (31) |
| `ports/distillery/probe/repros/cubek_browser_extrema/Cargo.toml` | 19; patch 26–28 | burn; drop `cubecl-runtime` (27) |

Also: the root `Cargo.toml` loses the `cubecl-runtime` row and its comment
(658–663) and gets pre.4 comments on the other two rows (664–677);
`support/patches/burn-cubecl/Cargo.toml` loses its inner `[patch.crates-io]
cubecl-runtime` row (169); `support/patches/cubecl-runtime/` is deleted. The
S0-3 ruling may add one row.

Tracked lockfiles that regenerate: the root, `probe`, `remote-fixture`,
`session-fixture` and both repros. `session-fixture` names no burn crate; it
moves through esp. native-fixture's lock is untracked.
`support/patches/burn-cubecl/Cargo.lock` becomes upstream's pre.4 lock, and
`support/patches/cubecl-runtime/Cargo.lock` leaves with its directory.

Out of scope: the six burn 0.21 probes under `crates/probes`, which are
untracked local workspaces in the main checkout (the directory is gitignored,
and no tracked probe names burn); `support/patches/cubecl-wgpu`, the retired
0.10.0 archive nothing patches in (but see 13.3, option C); and Knot and
Isometry, which move after this lands (13.10).

### 13.2 The patches against pre.4

Upstream: Burn `d3c8d7c6` (burn-remote, burn-backend) and `9147c11a`
(burn-cubecl), CubeCL `9d1314c0`, cubek `648520b6`, all published 2026-09-22.
Each pre.3 delta is `diff -ruN --strip-trailing-cr` of pristine pre.3 against
the pre.3 branch's patch (checked-out files are CRLF under `core.autocrlf`),
dry-run onto pristine pre.4 with `patch --dry-run`. The deltas match §12's
Phase A: burn-cubecl 5 files, burn-remote 11, cubek-reduce 5, cubecl-runtime 4.

**`cubecl-runtime`: retires (355).** Pre.4 exposes every value the pre.3
helper compared. `Handle.memory`, `offset_start`, `offset_end` and `stream` are
public fields (`cubecl-runtime-0.11.0-pre.4/src/server/handle.rs:11-27`). The
size is `pub(crate)` but has a public accessor, `size()` (`:140`).
`ManagedMemoryHandle::descriptor()` (`memory_management/handle.rs:203`) and
`ManagedMemoryDescriptor.id` (`:46-49`, `PartialEq`) are public but
doc-hidden.
The packaging fixes have been upstream since pre.3. Delete the directory, the
root row, burn-cubecl's inner row and the four nested rows.

**`burn-cubecl`: hand rebase, same logic.** Pre.4 wraps `launch_binop` and
`launch_binop_float` in `in_memory_order([lhs, rhs], output_shape, |[lhs, rhs],
shape_out| { … })` (`kernel/binary.rs:228`, `kernel/binary_float.rs:58`),
which presents both operands permuted into memory order and permutes the
result back. `launch_binop_int` is not wrapped (`kernel/binary_int.rs:131`).
`launch_unchecked` lost its runtime parameter (`::<O>`, was `::<O, R>`). The
guard goes first in each `unsafe` block, ahead of `if
lhs.can_mut_broadcast(&rhs)` (`binary.rs:249`, `binary_float.rs:79`,
`binary_int.rs:156`), and acts on the presented operands inside the closure.
Permutation changes metadata, not handles. `Handle::is_same_allocation` becomes
a private helper in the patched crate that keeps the pre.3 semantics (S0-2):

```rust
// illustrative
fn same_view(a: &Handle, b: &Handle) -> bool {
    a.memory.descriptor().id == b.memory.descriptor().id
        && a.offset_start == b.offset_start
        && a.offset_end == b.offset_end
        && a.stream == b.stream
        && a.size() == b.size()
}
```

Allocation ids come from one process-wide counter
(`memory_management/handle.rs:224-231`), so equal ids already mean the same
service, and pre.4's new `Handle.service` is not compared (S0-1). The manifest
reduces to `[workspace]`. The defect path persists: pre.4's `layer_norm` still
squares `B::float_mul(centered.clone(), centered.clone())`
(`burn-backend-0.22.0-pre.4/src/backend/ops/modules/base.rs:993`, and `:191`).
Dry run: `Cargo.toml` applies at offset 42; the three launcher hunks fail, as
expected.

**`burn-remote`: diff rebase, never a file copy.** Upstream changed `lib.rs`,
`server/mod.rs`, `server/worker.rs` and `transport/iroh/protocol.rs` between
pre.3 and pre.4, along with the client files, `server/builder.rs`,
`shared/task.rs` and a new `server/logging.rs`. Copying Mere's pre.3 files would
silently revert that work. Mere's pre.3 delta dry-runs onto pristine pre.4 with
every source hunk applying (`lib.rs` at offset 37, `protocol.rs` at offset 5)
except `server/mod.rs`, where pre.4's `mod logging;` and `pub use
logging::ServerLogging;` moved the context. Re-insert Mere's two exports, `pub
use crate::shared::SessionId;` and `pub use session::ServedSession;`, by hand.
The manifest edits are re-derived on pre.4's manifest: `server` adds
`tokio/macros`, dev-dependencies are restored at `=0.22.0-pre.4`, plus the self
`[patch]` and `[workspace]`. The patch is still required: pre.4's `close()`
still only removes the map entry
(`burn-remote-0.22.0-pre.4/src/server/session.rs:214-221`). One new upstream
behaviour: `IrohRemoteProtocol::new` now calls
`burn_std::set_runtime_kind(RuntimeKind::Async)` (`transport/iroh/protocol.rs:91`),
so a session's tensor reads materialize eagerly. S13 (c)'s in-flight reclaim is
the behavioural check. Lane H's `patched/burn-remote` compiled and passed both
lease tests; S5 regenerates it in the worktree and compares with that candidate.

**`cubek-reduce`: carries.** Both `extrema.rs` hunks apply to pristine pre.4;
only the manifest tail (the version) is re-derived. Pre.4 still builds
±infinity from literal bits
(`cubek-reduce-0.3.0-pre.4/src/components/instructions/extrema.rs:24`, `:34`).
Whether the runtime-value trick survives pre.4's WGSL path is decided only by
the headed extrema receipt.

### 13.3 turso and `persistence` (356): the answer

**No consumer-side switch exists.** Every edge from burn, cubek and cubecl
into `cubecl-runtime`, `cubecl-server` and `cubecl-environment` sets
`default-features = false` (a scan of every cached pre.4 manifest), and so do
Mere's own rows. Pre.2's forcing target table is gone: the desktop edge to
`cubecl-environment` requests no features
(`cubecl-runtime-0.11.0-pre.4/Cargo.toml:215-217`). One edge turns
`persistence` on, inside CubeCL. `cubecl-wgpu` declares its optional
`cubecl-cpp` dependency without `default-features = false`
(`cubecl-wgpu-0.11.0-pre.4/Cargo.toml:114-117`), and its `std` feature names
`"cubecl-cpp/std"` without `?`, which enables that dependency:

`cubecl-wgpu/std` → `cubecl-cpp/default` → `cubecl-runtime/default` (directly
and through `cubecl-core/default`) → `cubecl-runtime/persistence` →
`cubecl-environment/persistence` → `dep:turso`.

Evidence: a pre.4 scratch resolve of `a464dc2a` with exact pins and the pre.2
patch rows set aside, traced per row with `cargo tree -e features -i turso`.
The positive control is the same instrument showing `libsqlite3-sys` under
`conatus/resident` on the committed pre.2 lock.

- Every WGPU row carries turso, through that chain and no other: esp
  `bert-wgpu`, `index-burn-wgpu`, `decoder-wgpu`; numen `field-burn-wgpu`;
  seiche `tensor-burn-wgpu`; conatus `resident`; Distillery
  `trainer-gpu,trainer-autodiff,flora`, with and without dev edges; djinn
  `trainer-gpu,trainer-autodiff`.
- No CPU row does: esp `bert`, `index-burn`, `decoder-lora,decoder-autodiff`;
  numen `field-burn`; seiche `tensor-burn`; Distillery `remote`.
- Nothing in Mere's graph names `cubecl/persistence`, `cubecl-server/default`
  or any other `…/persistence` directly.
- Lane H's wasm32 checks of `probe` and both repros compiled `turso` and
  `turso_core` for `wasm32-unknown-unknown` (fingerprints in its scratch
  target).

**Four options.** Every way to turn `persistence` off is a patch; each patch
shape was resolved in a scratch copy.

- **(A) Keep it.** No patch. turso 0.8.0-pre.13 stays, pinned by the tracked
  lock. The caret `^0.8.0-pre.8`
  (`cubecl-environment-0.11.0-pre.4/Cargo.toml:188-191`) moves only on an
  explicit `cargo update`, so stop rule 7 gains turso.
- **(B) `cubecl-runtime`: drop `persistence` from `default`.** A one-line
  manifest change. No build compiles turso and the lock no longer lists it: the
  lock falls from 1,710 to 1,655 packages (turso's nine crates and 46 transitive
  ones, among them `bindgen 0.69`, `io-uring`, `shuttle`, `antithesis_sdk` and
  `prost`), and the graph holds no SQLite of any kind. Cost: it re-vendors the
  crate 355 just retired, for a different reason. Knot must carry the same row
  or keep turso, because `[patch]` is not inherited. It rebases every
  prerelease.
- **(C) `cubecl-wgpu`: `default-features = false` on its `cubecl-cpp` edge.**
  One line. `cargo tree --workspace --all-features --target all -e
  normal,dev,build,features -i turso` then finds nothing, so no build compiles
  turso. The lock still lists it (1,710 packages): Cargo's lockfile resolver
  also pulls in optional dependencies named through weak `dep?/feature` edges
  (`cubecl-metal?/std` in `cubecl`'s `std`), and `cubecl-metal` has the same
  defaults-on `cubecl-cpp` edge. In the build it changes only `persistence`:
  the other `cubecl-runtime` defaults it drops (`channel-mpsc`,
  `channel-cell`) gate no code in pre.4, and `cubecl-common`'s and
  `cubecl-ir`'s defaults are requested anyway.
  `support/patches/cubecl-wgpu` is occupied by the 0.10.0 archive.
- **(D) Upstream.** The edge looks like an oversight, since every other
  internal CubeCL edge sets `default-features = false`. The durable fix is a PR
  adding it to `cubecl-wgpu`'s and `cubecl-metal`'s `cubecl-cpp` edges, or one
  making `persistence` opt-in. Mere carries A, B or C until a release has it.

**What autotune loses with `persistence` off:** memory across processes, not
within one. `persistence` gates three stores: the write-through store behind
the autotune cache, with its `TuneRecord` trials
(`cubecl-runtime-0.11.0-pre.4/src/tune/tune_cache.rs:112-120`,
`tune/mod.rs:38-39`); the throughput-probe cache (`throughput/cache.rs:21-27`);
and compiled-kernel caching (`cubecl-server`'s `compilation_cache`, native
only, used by CubeCL's SPIR-V path and not by Mere's WGSL path). Without it,
`Store` falls back to memory, and each process re-tunes each (kernel, shape,
device) key once, on first use.

The database lives at `<outermost Cargo.toml>/target/environment/default.db`,
chosen from the working directory rather than `CARGO_TARGET_DIR`; an installed
app falls back to the user cache directory
(`cubecl-environment-0.11.0-pre.4/src/persistence/root.rs`). In the browser the
store serves memory unless a page awaits
`cubecl_environment::environment::open()`, which is new in pre.4 and which no
Mere page calls, so there turso costs compile time and nothing else.

Pre.2 kept these stores in rusqlite, and did write on this machine. The only
such database here (`%LOCALAPPDATA%\cubecl\default.db`, written 2026-08-25 by
a process outside any workspace, WAL mode) holds two throughput-probe entries
(`throughput/0.11.0-pre.2/wgpu<wgsl>_dev0`) and no autotune entries.

**Measured 2026-09-27** on this machine (Windows 11, RTX 4060 Laptop GPU),
with release builds of the scratch pre.4 tree, wall time per process:

- **Only three rows turn autotune on at all:** conatus `resident`, numen
  `field-burn-wgpu` and seiche `tensor-burn-wgpu` name `burn-wgpu/autotune`.
  esp's WGPU rows and Distillery's trainer rows do not, because burn's `wgpu`
  feature leaves it off.
- **esp's `timing_bert_cpu_vs_gpu`** (MiniLM dimensions): with `persistence`
  on, 4.4–4.8 s per run whether `target/environment` was fresh or not; with
  option B's build, 4.6–4.9 s. The first run of each new binary took 7.1–7.2 s
  either way, and deleting the cache did not bring that back, so that cost is
  not CubeCL's. GPU forward times were identical, about 52 ms at batch 32 × seq
  128.
- **seiche's `timing_repulsion_cpu_vs_gpu`** (autotune on, `sum_dim`
  reductions): 3.9–4.4 s, cold or warm. CubeCL's decision log shows two keys
  tuned per process, with 9.0 ms and 5.6 ms of tuning steps. About 15 ms per
  process is everything autotune can lose here.
- **conatus `resident` and Isometry's resident-ground runs** (Lane H) left
  0-byte databases: nothing was stored.
- **On Windows, pre.4's `persistence` cannot work at all.** cubecl-environment
  opens turso with `experimental_multiprocess_wal(true)`
  (`cubecl-environment-0.11.0-pre.4/src/persistence/turso.rs:530`). turso's
  default Windows IO backend does not support shared-WAL coordination: it
  takes the trait default, `false` (`turso_core-0.8.0-pre.13/io/mod.rs:435`),
  and only Unix (`io/unix.rs:46`) and the opt-in `experimental_win_iocp`
  backend return `true`. So the open falls back to read-only, fails on the
  empty file ("no such table: meta"), and the store serves memory behind a
  warning. A scratch probe of `environment::store` reproduces this, and every
  0-byte `default.db` above is its leftover. Unix should persist; that was not
  verified here.

**Recommendation:** (B), with (D). No Mere row measured loses anything measurable
without `persistence`: at most about 15 ms of tuning per process, and on Windows
pre.4 already serves memory. Keeping it compiles turso's nine prerelease crates
and their dependencies into every WGPU build, native and wasm, and creates a
`default.db` wherever GPU code runs (an empty one, on Windows).

B is a one-line manifest patch with a plain removal condition: delete it when a
CubeCL release makes `persistence` opt-in or fixes the `cubecl-wgpu` edge. Its
cost is the tension with 355. S4 then rewrites `support/patches/cubecl-runtime`
instead of deleting it: a new `MERE-PATCH.md` and no identity helpers, while the
root row and the four nested rows stay. Knot needs the same row only if it
wants turso out of its own builds, which work either way.

(D) is an issue or PR on tracel-ai/cubecl, covering the defaults-on
`cubecl-cpp` edges and the Windows memory fallback; only Mark can file or
authorize it. If one more vendored crate weighs more than a prerelease database
in the graph, (A) is safe on this evidence: the lock pins turso, and on Windows
it never opens.

### 13.4 API changes

From Lane H's scratch diff, compile-checked there:

- **conatus** (nine errors; 16 changed lines and two now-unused `WgpuRuntime`
  imports, in `crates/conatus/conatus/src/resident.rs`,
  `crates/conatus/conatus/src/resident/chunk.rs` and
  `crates/conatus/conatus/tests/resident_chunk.rs`). `ComputeClient<WgpuRuntime>`
  becomes `cubecl::client::Client`. `launch_unchecked::<WgpuRuntime>` and
  `CubeTensor::<WgpuRuntime>` lose the parameter, and `new_contiguous` takes
  `device.clone().into()`. `WgpuRuntime::client(&device)` becomes
  `cubecl::Device::from(device.clone()).client()`. `get_resource(handle)` names
  the server:
  `get_resource::<cubecl::wgpu::WgpuServer<cubecl::wgpu::AutoCompiler>>(handle)`.
- **remote-fixture** (`ports/distillery/probe/remote-fixture/src/main.rs:92`).
  `AllocatorSnapshot::capture` uses both removed shapes,
  `<WgpuRuntime<AutoCompiler> as Runtime>::client(device)` and a `Result` from
  `memory_usage()`. It was not compiled, because resolution fails first (13.0
  item 5).
- **Isometry's isometer-lens**
  (`isometry/shared/isometer/crates/isometer-lens/examples/resident_ground.rs:422`,
  `:440`). `memory_usage()` returns `MemoryUsage`, not a `Result`, so both
  `.expect(..)` calls go. That change lands with Isometry's repin (354).
- **esp, numen, seiche, Distillery and djinn compile unchanged**, with
  deprecation warnings. Lane H's check logs count 110 for `Device::ndarray`,
  113 for `TensorData::to_vec` and 4 for `TrainerDevice::ndarray`. These are not
  stops; the follow-up is the same as §12.2's.
- **Not yet built on pre.4:** esp's `decoder-lora`, `decoder-autodiff`,
  `model-session` and `persistence` rows, and every wasm matrix row outside the
  Distillery probes. S9 builds them.

### 13.5 Receipts and how they are produced

- **(a) Same-allocation and BERT-width LayerNorm, headed.**
  `repros/burn_browser_embedding/run-repro.ps1 -WasmBindgen
  C:\t\wasm-bindgen-0.2.122\wasm-bindgen-0.2.122-x86_64-pc-windows-msvc\wasm-bindgen.exe
  -TargetDir C:\t\mere-burn-pre4-embedding`. It builds `--locked --release
  --target wasm32-unknown-unknown` and serves `web/` with `python -m
  http.server`. Open the page in headed Chromium and call
  `window.burnEmbeddingRepro.run()`. Save the JSON verbatim as
  `receipts/<date>_binary_alias_pre4_iab.json`, with the native control
  `shared_binary_and_layer_norm_pass_native_wgpu`. *Done:* `passed: true`,
  eleven embedding controls pass, four graph cases `matches_expected: true`,
  `gpu_errors: []`.
- **(a') Unpatched control.** Drop only the `burn-cubecl` row, build into a
  separate target, record, restore. *Done:* `burn-unit-raw-mul-shared` false
  and both LayerNorm cases `output_matches_input_bits: true`.
- **(b) Extrema, headed.** `repros/cubek_browser_extrema/run-repro.ps1`, then
  `window.cubekExtremaRepro.run()`. *Done:* four cases pass, `gpu_errors: []`.
- **(c) Two-peer remote run.** `ports/distillery/probe/run-remote-minilm.ps1
  -TargetDir C:\t\mere-burn-pre4-remote -ModelDir
  <main tree>\models\all-MiniLM-L6-v2`. It needs remote-fixture resolving
  (S0-6) and adapted (13.4). *Done:* receipt `dirty=false`; 384 values match
  ESP native within the pre.2 receipt's `1.4901161e-7`; the 512-row in-flight
  request fails on reclaim without hanging; CubeCL allocations and bytes return
  to baseline; a fresh lease reproduces the output. The Fusion/autotune matrix
  is not required.
- **(d) Existing device.** `cargo test -p conatus --release --features
  resident --test resident --test resident_chunk`. *Done:* nine pass in
  release; Lane H's nine were a debug run.
- **(e) esp WGPU parity.** `bert_sentence_parity_ndarray_wgpu`, and
  `real_minilm_fixture_wgpu` with `ESP_MINILM_DIR`. *Done:* under `2.0e-3`,
  and the fixture's first eight values within the validation tolerance.
- **Tooling.** The 0.2.122 CLI is on disk and prints `wasm-bindgen 0.2.122`
  (checked 2026-09-27). `~/.cargo/bin/wasm-bindgen` is now 0.2.127, which the
  scripts reject, because 0.2.123 and later break wgpu 30's `popErrorScope`
  (`ports/distillery/probe/Cargo.toml:24`). Pre.4 still names wgpu `^30`.
  The 2026-08-22 receipts record Chromium 151 on NVIDIA Lovelace; their `_iab`
  suffix presumably names the in-app browser. WebGPU compute and readback involve
  no canvas compositing, the Browser pane's known gap, so the pane can produce the
  receipt if it exposes a WebGPU adapter. Real Chromium through Claude in Chrome
  is the fallback.

### 13.6 Isolation

- Worktree `Code/worktrees/mere-burn-pre4`, branch `burn-pre4-repin`, from
  `origin/main` at `a464dc2a`, upstream unset. It has only
  `.cargo/config.toml.example`, so it resolves the portable graph; there is no
  local config to rewrite the lock.
- Every step uses `CARGO_TARGET_DIR=C:\t\mere-burn-pre4-target` and
  `CARGO_NET_OFFLINE=true`; scripts get `-TargetDir C:\t\mere-burn-pre4-<name>`.
  All pre.4 burn, cubecl and cubek crates are already in the cache. Never run
  `cargo generate-lockfile` or a bare `cargo update`.
- `-j 4`. Rust 1.98.1.
- With `persistence` on, every GPU test creates
  `target/environment/default.db` under the worktree, whatever
  `CARGO_TARGET_DIR` says; on Windows the file stays empty. `target/` is
  gitignored.
- Models are gitignored. Pass `-ModelDir` or `ESP_MINILM_DIR` pointing at the
  main tree's `models/all-MiniLM-L6-v2`, which is present.
- The reclaim script (`Code/testing/_scripts_reclaim_stale_2026-09-15.ps1`)
  deletes idle directories under `C:\t` and idle session temp folders, not loose
  files. The baseline lock is safe from it. `C:\t\wasm-bindgen-0.2.122` and
  `C:\t\burn-remote-0.22.0-pre.2` are directories, and so is Lane H's scratch.
  Do not run it during the repin. S2 rebuilds every delta from registry sources,
  so nothing here depends on Lane H's scratch surviving.

### 13.7 Steps and done-conditions

- **S0. Decisions (Mark).** 13.12. *Done:* each answered and recorded here
  under "S0 rulings" before any manifest moves.
- **S1. Worktree.** Done 2026-09-27. *Done:* no `.cargo\config.toml` or
  `config.local.toml` in the worktree; `cargo tree --offline --locked -p esp
  --features bert-wgpu -e normal --depth 1` exits 0 on the committed lock, with
  only the three pre-existing unused-patch warnings (`boa_engine`, `boa_gc`,
  `iroh-mdns-address-lookup`).
- **S2. Capture deltas.** `git archive 610a32c5 support/patches/…`, then
  `diff -ruN --strip-trailing-cr` against pristine pre.3 from the registry
  cache, into `C:\t\mere-burn-pre4-deltas`. *Done:* four deltas with the file
  counts in 13.2.
- **S3. Rebase burn-cubecl.** Pristine pre.4 plus the guard in the three
  launchers, the private helper, `[workspace]` and `MERE-PATCH.md`. *Done:*
  against pristine pre.4 the diff is three guard blocks, one helper, the
  manifest tail and `MERE-PATCH.md`; it builds as a path dependency of a
  throwaway crate (§12 Phase A's method), and `cargo tree` shows the patch used.
- **S4. Retire cubecl-runtime** (13.1). *Done:* `git grep
  "patches/cubecl-runtime"` matches no manifest.
- **S5. Rebase burn-remote.** Apply the pre.3 delta to pristine pre.4,
  hand-insert the two `server/mod.rs` exports and re-derive the manifest.
  *Done:* against pristine pre.4 the diff lists only `Cargo.toml`,
  `Cargo.toml.orig`, `MERE-PATCH.md`, `src/lib.rs` and the seven server,
  shared and transport files, and matches Lane H's candidate or each
  difference is explained.
- **S6. Rebase cubek-reduce.** *Done:* the diff is one helper, two call sites,
  `[workspace]`, the licenses and `MERE-PATCH.md`.
- **S7. Move the manifests** (13.1), apply the S0-3 ruling, and rewrite the
  root comments. *Done:* `git grep` for `0.22.0-pre.2`, `0.11.0-pre.2` and
  `0.3.0-pre.2` in manifests matches only comments and
  `support/patches/*/Cargo.toml.orig`, and every burn, cubecl and cubek
  requirement in the ten manifests begins with `=`.
- **S8. Regenerate the root lock, offline and targeted:** `cargo update
  --offline -p burn -p burn-wgpu -p cubecl -p burn-backend -p burn-ir -p
  burn-flex`. Do not pass `-p burn-remote`: the committed lock records it as a
  path package, so the spec fails, and the manifest change re-resolves it
  anyway. *Done:* no `pre.2"` in the lock; `burn-cubecl` and `burn-remote` at
  pre.4 with no `source`; no "patch was not used" for them; the lock diff
  touches only the burn, cubecl and cubek families and their needs, with turso
  as S0-3 rules; one wgpu 30.x; esp with no default features has no burn and no
  tokenizers; `python scripts/cargo_mode.py verify --metadata-only` passes. For
  reference, the scratch resolve changed 109 entries outside the three
  families, every one a new or dropped need of pre.4: turso's tree; CubeCL's
  CPU backend trading its MLIR tree (`tracel-mlir-*`) for a pliron/LLVM one;
  `sysinfo` 0.39; and rusqlite's tree leaving. wgpu 30.0.1, tokio 1.53.1,
  iroh 1.2.0 and the p2panda crates did not move.
- **S9. conatus and the build/test matrix.** Apply 13.4's conatus change.
  Native: esp default, `bert,bert-validation`, `index-burn-wgpu`,
  `decoder-wgpu`, `decoder-lora,decoder-autodiff`, `model-session`,
  `persistence`; numen `field-burn`; seiche `tensor-burn`. GPU release: numen
  `field-burn-wgpu`, seiche `tensor-burn-wgpu`, and receipts (d) and (e). wasm,
  each with `--target wasm32-unknown-unknown --no-default-features`: esp's
  eleven matrix rows; numen `""`, `field-burn`, `field-burn-wgpu`,
  `field-rhai`; seiche `""`, `tensor-burn`, `tensor-burn-wgpu`; conatus
  `resident`. *Done:* every suite passes, (d) and (e) meet their conditions,
  and every wasm row exits 0. Rerun ambiguous failures on a detached `a464dc2a`
  worktree.
- **S10. Distillery and djinn.** `cargo check --offline -p distillery
  --all-targets --features remote,trainer-gpu,trainer-autodiff,flora`; `cargo
  test --offline -p distillery --features remote --lib remote::tests`; `cargo
  check --offline -p djinn --features trainer-gpu,trainer-autodiff`. *Done:*
  all exit 0 and both lease tests pass. Lane H's scratch run passed, with
  djinn's knot-editor at `5ad3f67`; this run is at `d265f033`.
- **S11. Whole workspace.** `cargo check --offline --workspace --all-targets
  --locked`, then `python scripts/cargo_mode.py verify`. *Done:* both exit 0,
  or a failure lies outside burn's graph and fails identically on `a464dc2a`.
- **S12. Nested workspaces.** First apply S0-6 to remote-fixture's p2panda
  rows. Then update the five tracked nested locks offline and targeted, adapt
  remote-fixture's snapshot (13.4), and check each workspace with its own
  command: `probe` and both repros for `wasm32-unknown-unknown`, the fixtures
  natively. *Done:* no pre.2 entries, no unused-patch warnings, and every
  command exits 0. Commit patches, manifests and locks before receipts, so the
  receipts record a clean commit.
- **S13. Receipts** (13.5).
- **S14. Commits on the branch:** one per patch change (burn-cubecl, the
  cubecl-runtime retirement, burn-remote, cubek-reduce); remote-fixture's
  p2panda repair on its own; manifests and locks; the conatus change; receipts;
  docs. No attribution trailers. Pushing is S16.
- **S15. Docs.** This plan's status and progress. The closure doc's production
  row and patch table
  (`design_docs/mere_docs/testing/2026-08-20_burn_0_22_prerelease_closure.md`).
  `MERE-PATCH.md` in the three remaining patches and esp's manifest comment.
  **2026-09-28 checklist correction:** ruling 375 retains four patch documents,
  including the manifest-only persistence-off `cubecl-runtime` patch; the
  historical count of three and S14 retirement wording no longer govern.
  The probe and repro READMEs and the `UPSTREAM_ISSUE.md` files. The
  feature/target matrix
  (`design_docs/intel_docs/technical_architecture/2026-08-09_feature_target_matrix.md`).
  `DOC_README.md`'s lines for this plan and the closure doc; the plan's line
  has been stale since 2026-09-16 and still reads "pre.3 audited 2026-08-26".
  The file header's SQLite sentence, per S0-3. *Done:* no current-state doc
  says pre.2 is production, and `python scripts/mere_doc_audit.py` reports no
  new finding in the files touched.
- **S16. Merge back.** Rebase onto `origin/main`. The root lock is tracked now,
  so a conflict there is a real merge conflict: take `origin/main`'s lock, rerun
  S8's targeted update, then S8's done-condition. Smoke `esp --features
  bert-wgpu`, `distillery --features remote` and conatus `resident_chunk`.
  Fast-forward push to `main` with Mark's authorization in the S0-7 window, and
  record the pushed revision R. Do not merge into the main working tree beneath
  a live session. *Done:* `origin/main` is R, and a doc commit records it.
- **S17. Knot handoff** (13.10).
- **S18. Isometry handoff** (13.10).

### 13.8 Stop rules (halt and return to Mark)

1. A patch needs different logic, not new context: pre.4 changed
   `can_mut_broadcast`, `Handle`'s fields beyond the `service` field S0-1
   rules on, or `SessionManager`'s structure.
2. The unpatched control (S13 a') passes the shared-input cases: the patch may
   be removable, and carrying or dropping it is Mark's call.
3. A patched same-allocation or extrema case fails, or `gpu_errors` is
   non-empty.
4. The remote run hangs on reclaim, allocations miss baseline, or numbers
   differ from the pre.2 receipt without an upstream explanation.
5. Any numeric or parity result moves without an explained upstream change.
6. A compile fix would change an esp public contract, a feature name, device
   ownership or the burn-remote patch API. Mechanical renames and deprecation
   warnings are not stops.
7. Lock regeneration moves wgpu off 30.x; adds a second burn, cubecl, wgpu or
   turso; moves turso off 0.8.0-pre.13 or brings it back against the S0-3
   ruling; or moves unrelated majors (iroh, tokio, p2panda).
8. esp with no default features pulls burn or tokenizers.
9. Rebasing onto `main` conflicts in a manifest.
10. Cargo insists on a network fetch while offline, beyond what S0-9 allows.
11. "patch was not used" names `burn-cubecl`, `burn-remote`, `cubek-reduce`,
    or a patch row added under S0-3. The pre-existing `boa_engine`, `boa_gc` and
    `iroh-mdns-address-lookup` warnings are recorded, not stops (§12, Phase B
    ruling).

### 13.9 Risks, highest first

1. CubeCL's WGSL path changed again between pre.3 and pre.4: the extrema trick
   may stop yielding a runtime value, and the same-allocation defect may worsen
   or vanish. Only S13 a, a' and b decide it.
2. remote-fixture: the pre-existing p2panda failure, the pre.4 API change, and
   upstream's new `set_runtime_kind(Async)` all sit in front of S13 (c), the
   only behavioural check of the burn-remote rebase.
3. The guard now runs inside `in_memory_order`, on operands that may have been
   presented permuted. Identity is unaffected, because presentation clones
   handles, and the aliased operand keeps its own layout, as in pre.3. But no
   receipt exercises a permuted same-allocation pair: S13 (a) covers
   LayerNorm's logical-order shape only.
4. turso, if kept: a prerelease database engine compiled into every WGPU row,
   native and wasm, held in place only by the tracked lock, and inert on
   Windows (13.3).
5. Headed tooling: the 0.2.122 CLI is a directory the reclaim script can take,
   newer CLIs break wgpu 30's error scopes, and a headed Chromium session is
   required.
6. Live sessions: the tracked root lock turns S16 conflicts into real merges,
   and every session rebuilds the burn stack after it lands.
7. esp's LoRA and autodiff rows were built on neither pre.3 nor pre.4.
8. Deprecation noise, and burn-ndarray's announced removal: follow-up.

### 13.10 Knot and Isometry handoffs

**Knot (S17).** After S16, `origin/main` at R holds
`support/patches/burn-cubecl` at `0.22.0-pre.4` and no `cubecl-runtime`
patch. Knot moves every mere pin to R together and regenerates its lock; esp's
exact pin forces its burn family from pre.3 to pre.4. Nothing Knot calls
changes, because esp's source compiles unchanged on pre.4. djinn compiled
against the pre.4 tree in Lane H's scratch with knot-editor `5ad3f67`, and S10
repeats that at `d265f033`. So the knot-first procedure is not needed: mere
lands, then Knot repins. Knot needs no
`cubecl-runtime` row, ever. It needs a `burn-cubecl` row only if it runs BERT
on BrowserWebGpu; it is desktop-only today, and the native control passes
without the patch:

```toml
# only if Knot ever runs BERT on BrowserWebGpu:
# burn-cubecl  = { version = "=0.22.0-pre.4", git = "https://github.com/merely-made/mere.git", rev = "<R>" }
# cubek-reduce = { version = "=0.3.0-pre.4", git = "https://github.com/merely-made/mere.git", rev = "<R>" }
```

If S0-3 picks option B or C, Knot carries that row as well, or its WGPU builds
keep turso. *Done in Knot:* no unused-patch warning, and `cargo tree -p
knot-editor --features embed-bert-wgpu -i burn` shows only `0.22.0-pre.4`.

**Isometry (S18, ruling 354).** Isometry's isometer pins mere by rev
(`0418391f` on Isometry's `main`) and reaches conatus through git. After S16 it
moves that pin to R and, in the same commit, drops the two `.expect(..)` calls
(13.4). Nothing lands before that. *Done in Isometry:* isometer-lens's
`resident_ground` receipt passes on the RTX 4060. Lane H's scratch run passed
with `burn_raw_same_allocation: true` and `replay_matches: true`.

### 13.11 Effort

Mechanical: S1, S2, S4, S6–S8, the lock half of S12, S14, S15, S17 and S18.
Investigation: S3 (the guard in its new position), S5 (the hand-inserted
exports and the new runtime-kind behaviour), S9's wasm rows and esp's LoRA and
autodiff rows, remote-fixture's repair in S12, and S13 a, a', b and c.

Machine cost, measured on this machine at `-j 4` (Lane H's figures are debug
builds into warm targets): a cold release build of esp's `bert-wgpu` tests took 19m47s. A
second feature set on the same target (option B's patch, or seiche's
`tensor-burn-wgpu` tests) took about 17 minutes more, because the burn and
CubeCL families rebuild whenever their features change. Lane H's debug builds
took between 1m11s and 6m13s each. The plan needs roughly a dozen
such builds: the native matrix, the GPU release rows, the wasm matrix, five
nested workspaces, three headed wasm builds, the unpatched control, and the
remote fixture.

### 13.12 S0 questions

Each question needs an answer before S7 moves a manifest, except S0-8, which
can wait for S16.

- **S0-1. `Handle.service` and stop rule 1.** §12.6 rule 1 makes a change to
  `Handle`'s fields a stop, and pre.4's `Handle` gained `pub service:
  ServiceId` (`server/handle.rs:14-18`). The guard's logic does not change:
  allocation ids come from one process-wide counter, so equal ids already
  imply the same service. Options: (a) treat it as new context and proceed
  without comparing `service`; (b) proceed and compare `service` too, which is
  harmless today and robust if upstream ever makes ids per-service; (c) treat
  it as new logic, stop and redesign. **Recommend (a).**
- **S0-2. What 355's comparison covers.** 355 names the allocation id. Pre.3's
  helper also required equal `offset_start`, `offset_end`, `stream` and size,
  so it matched only the same view of one allocation. That still matters on
  pre.4: an aligned `slice` is a zero-copy view on the same handle at new
  offsets (`burn-cubecl-0.22.0-pre.4/src/kernel/index/slice.rs:34-47`), so an
  id-only guard would bind `x.slice(a) * x.slice(b)` once and compute lhs·lhs.
  Options: (a) the id plus the four view fields, all public in pre.4, which are
  the pre.3 semantics; (b) the id only, as worded. **Recommend (a).**
- **S0-3. turso (356).** Options A–D in 13.3: (A) keep `persistence`, with
  turso pinned by the tracked lock and stop rule 7 guarding it; (B) patch
  `cubecl-runtime`'s `default` (no turso in any build or in the lock; one
  re-vendored crate, which Knot must also carry); (C) patch `cubecl-wgpu`'s
  `cubecl-cpp` edge (no turso in any build, but still in the lock); (D) an
  upstream PR, alongside A, B or C. **Recommend (B) with (D).** No Mere row
  measured loses anything measurable, and on Windows pre.4's `persistence`
  already runs from memory. B cuts against 355's retirement of the
  `cubecl-runtime` directory; (A) is the no-patch alternative if that weighs
  more.
- **S0-4. The lock baseline for pre.4.** Options: (a) the committed portable
  lock at `a464dc2a` (blob sha256
  `9cf2136bfee2c08d63d4a9308131e26d5f81ec268490519cfeb48f777f2b62cf`; the CRLF
  working copy hashes to `5eb4eb0a…`), which passes `--locked` offline in the
  worktree; (b) the 2026-09-16 ruling's normalized pre.2 lock
  (`C:\t\mere-burn-pre3-baseline-normalized-pre2.lock`, sha256 `6055022d…`,
  still present), which fails `--locked` against today's manifests (exit 101).
  **Recommend (a)**, leaving the `C:\t` file as history.
- **S0-5. Distillery's scope in this repin.** Options: (a) full: S10, the five
  nested workspaces in S12, and S13 a, a', b and c; (b) S10 and the headed
  receipts (a, a', b), with the two-peer run (c) deferred until remote-fixture
  is repaired elsewhere; (c) S10 only. **Recommend (a):** (c) is the only
  behavioural check of the burn-remote rebase, and its prerequisite is small
  (S0-6).
- **S0-6. Where remote-fixture's pre-existing failure is fixed.** It has been
  broken since `0db9d10d` (2026-09-14), on pre.2 as well. Options: (a) on this
  branch, as its own commit ahead of the pre.4 manifest move: restate the root's
  eight `mere-p2panda-net-0.7.4` rows (`mere-p2panda-net`, `p2panda-auth`,
  `-core`, `-discovery`, `-encryption`, `-store`, `-stream`, `-sync`) in its
  `[patch.crates-io]`, and prove it resolves and builds on pre.2 first; (b) a
  separate fix on `main` first, which this branch rebases onto; (c) leave it,
  and skip S13 c. **Recommend (a).** The commit stays separable, so (b) remains
  possible at merge.
- **S0-7. The merge window.** Options: (a) §12's D4: at S16, message the busy
  Mere sessions, wait for each idle notice, fast-forward push, and tell them to
  pull; (b) a window Mark names. **Recommend (a).** With the root lock tracked,
  every session rebuilds the burn stack on its next pull.
- **S0-8. The parked pre.3 lane, at S16.** `Code/worktrees/mere-burn-pre3`
  holds another session's superseded uncommitted edits, and `burn-pre3-repin`
  has four unmerged commits that §12 records. Options: (a) leave both until
  Mark says otherwise; (b) after S16, remove the worktree and delete the branch;
  (c) keep the branch as archaeology and remove only the worktree. **Recommend
  (a) for now**, and decide at S16.
- **S0-9. Network during execution.** Every pre.4 burn, cubecl and cubek crate
  is already cached, and so is the 0.2.122 CLI; a nested workspace may still
  want an uncached crate. Options: (a) allow crates.io crate downloads when an
  offline step needs one, but never a git fetch; (b) stay strictly offline, so
  stop rule 10 halts on any download. **Recommend (a).**

### 13.13 Accepted scope and execution preparation (2026-09-27)

**Status:** documentation only; migration has not begun. The original plan
commit `02603e0903aeb1a6ed04675119be491b1e5d00e1` is integrated without
rewriting its history. The canonical rulings are 375–377 in
isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md. The questions,
options and joint answer are preserved here because they change this plan's
execution conditions.

**375, persistence; explicitly amends 355 and settles 356.** Asked:
"1. Should pre.4 retain GPU tuning results between app launches?"
Evidence as put: "Disabling persistence removes 55 packages, taking the
measured dependency count from 1,710 to 1,655. It requires retaining a small
vendor patch, amending ruling 355. The tested Windows configuration already
falls back to memory; persistence's benefit on Unix remains unmeasured."
Options, recommended first:

- A: "Disable persistence (recommended). Keep the manifest-only patch;
  tuning repeats when needed in each process. Retire the old identity helper."
- B: "Keep persistence. Remove the runtime patch as 355 intended; retain
  Turso and its dependencies."
- C: "Measure Unix first. Compare cold and warm launches before choosing."

**376, full Distillery acceptance.** Asked:
"2. How thoroughly should we verify Distillery during the migration?"
Evidence as put: "The full plan covers five nested workspaces and a two-peer
run checking output, cancellation, memory recovery and a fresh lease. The
remote fixture first needs its five outdated dependency patch rows aligned
with the root's eight current rows."
Options, recommended first:

- A: "Require the full set (recommended). Repair and verify the fixture on
  pre.2 first, then repeat the complete checks on pre.4."
- B: "Defer the two-peer run. Require compilation, lease tests and headed
  numerical checks; leave remote acceptance open."
- C: "Limit this pass to compilation and lease tests. Leave both headed
  numerical and remote acceptance open."

**Mark's answer to both, verbatim:** "I accept your two recommendations. Wow.
Much larger dependency count than I figured."

**377, the guard gains the service comparison.** Asked:
"How should we carry the GPU allocation guard into pre.4? It currently
compares five values: allocation ID, two slice offsets, stream and size.
Pre.4 adds a service ID, but normal allocations still receive globally unique
IDs, so a sixth comparison is currently redundant. The existing five must
remain: different slices can share an allocation, and treating them as
identical can produce the wrong calculation."
Options, recommended first:

- A: "Preserve the five comparisons; treat the new service field as context
  and proceed under the migration stop rule (recommended)."
- B: "Add service ID as a sixth comparison; also verify the new rejection
  case before proceeding."

**Mark's answer, verbatim:** "Add service ID as a sixth comparison; also
verify the new rejection case before proceeding."

Consequences for execution:

- **S0-3 / S4 / S7:** choose §13.3 B. Retain a freshly rebased
  `cubecl-runtime` with only the manifest change removing `persistence` from
  defaults; retire the identity helper. Keep the needed root and nested patch
  rows instead of deleting them. S4's earlier zero-row done-condition is
  superseded: compare against pristine pre.4 and prove the only functional
  runtime patch is this manifest change, with its provenance documented.
  S8 must show turso absent from the regenerated lock and build graph.
- The 1,710/1,655 counts are Lane H's lockfile resolution counts, including
  optional packages, not every application's compiled crate count. The 55
  comprise nine turso crates and 46 transitive packages. Disabling persistence
  loses cross-process autotune/throughput caching (and the compiled-kernel
  store on applicable native paths); within-process caches remain. The tested
  Windows setup already serves memory. Unix persistence and performance are
  unmeasured; the Windows result is not a cross-platform no-cost claim.
- **S0-5 / S0-6:** require S10, all five S12 nested workspaces, and all S13
  headed numerical/control and two-peer lifecycle receipts. Repair
  remote-fixture's patch table as a separate prerequisite commit and prove its
  pre.2 baseline before moving to pre.4. Commit placement and synchronization
  with main are coordination work; they do not weaken these gates.
- **S0-1 / S0-2 / S3, settled by 377:** preserve allocation id, both offsets,
  stream and size, and add equality of `Handle.service` as the sixth
  predicate when moving the guard into burn-cubecl. This explicitly disposes
  of the added-field stop rule for this change. §13.2's five-predicate sample
  remains its dated illustration, not the implementation specification.
  Verify an otherwise-matching pair is rejected when its service differs,
  alongside a same-service positive case and the distinct-slice controls;
  retain passing and deliberately failing evidence before proceeding.
- **Baseline refresh (S0-4):** coordinate onto current main before any
  migration build, preserve the dated normalized/portable-lock receipts, and
  declare the new source commit and committed portable-lock hash. This is a
  source-baseline refresh, not permission to change production dependencies.
  The read-only snapshot for this annotation is Mere `211b456e`, whose Genet
  pin is `34626a6c`, Knot pin `f14f9ef3`, and Netrender pin `c8c09f16`; Lane M's
  original base was `a464dc2a`, with Genet `0cf4f30b` and Knot `d265f033`.
  Verify the coordinated head again when execution opens; these are not new
  pinned execution receipts.
- **S0-7:** D4 already requires idle notices before integration. Ruling 363
  keeps traversal, pre.4 migration, then T2 in order. **S0-8:** preserve the
  parked pre.3 branch, worktree and uncommitted work. **S0-9:** remain offline;
  if a missing dependency needs a download, bring that bounded request back.
  The earlier D3 permitted two specific downloads, not unrestricted fetching.
- **Execution-path override:** §13.6's old target paths and per-receipt target
  variants are superseded. Use the stable `C:/t/cargo-targets/mere`, including
  scripts' `-TargetDir`, and coordinate its owner before Cargo work. Override
  any inherited target setting; use Rust 1.98.1 and four build jobs. Existing
  `Code/worktrees/mere-burn-pre4` is the Lane M checkout. Do not create the
  old §13.7 S2 scratch directory or throwaway source trees under arbitrary
  `C:/t` paths; keep retained deltas/receipts under approved `Code/testing`.

**Reading, not ruled:** retire the new manifest-only patch when upstream
makes persistence optional without the forcing edge. No upstream issue, PR,
or other communication was authorized by this answer. This documentation
integration runs no Cargo gate and changes no production manifest or lock.

### 13.14 First checkpoint, prepared source and offline stop (2026-09-27)

**Status:** prerequisite verified; S3/S4 prepared but uncompiled. No root
pre.4 manifest migration, production acceptance or S5–S18 continuation.

The documentation integration is Mere `844affce` (parents `211b456e` and
`02603e09`); Lane M fast-forwarded onto it before implementation. The clean
portable baseline's root lock working-byte SHA256 is
`da0822f516010218b3b163234d9826955add552002cffadf7250b5932add13d4`,
on Rust 1.98.1. Its offline locked `esp --features bert-wgpu` tree passed and
showed Burn pre.2. All Cargo work used `C:/t/cargo-targets/mere`, four jobs,
offline, after an owner check; the root manifest and lock remain unchanged.

**Remote-fixture prerequisite, commit `43c50fc6`:** the original offline
locked check reproduced the missing `mere-p2panda-net = "=0.7.4"` error.
Restating the root's eight p2panda patch rows resolved it. The nested lock
changed from 883 to 969 packages: 95 added identities and nine removed, while
the Burn/CubeCL/Cubek families remained pre.2. Current Distillery's path/GUI
dependencies explain this baseline closure; these additions are not a pre.4
migration cost. The `h2` change from 0.4.18 to 0.4.19 matches the root lock.
The auth/stream rows remain unused optional mirrors; no fake dependencies
were added to suppress their warnings.

The first complete `cargo build --manifest-path
ports/distillery/probe/remote-fixture/Cargo.toml --offline --locked` passed.
After its compiler exited, the exact root Vello patch was mirrored too. A
targeted update swapped Vello and vello_encoding 0.10.0 from the registry to
git `4354955e` and changed data-encoding-macro-internal 0.1.19's `syn` edge
from 3.0.4 to 1.0.109, allowed by its `>=1,<4` requirement and matching the
root lock. The total stayed 969. The affected locked build also passed.
These are compilation receipts; the two-peer lifecycle run remains a later
gate, and its script's limited `dirty=false` field is not graph provenance.

**Prepared S3/S4, lane commits `29646154` and `e9012d72`:** freshly copied published pre.4 sources carry the
manifest-only runtime default-feature change and a private burn-cubecl
`same_view` guard called by all three launchers. Nine authored unit tests
cover matching views, allocation identity, both service-identity axes,
separate slice offsets, None versus Some(0), stream and underlying size.
Static comparison finds no runtime Rust differences from pristine pre.4.
Burn-cubecl differs only in its manifest, provenance note, the three guards,
module declaration and new helper/tests. The standalone manifest's two new
dev-dependencies name the already-transitive common/environment crates for
constructing public test handles. Independent read-only review found no
remaining source correctness issue; this is not execution evidence.

**Actual stop:** `cargo update --manifest-path
support/patches/burn-cubecl/Cargo.toml --offline -p cubecl-runtime` exited
101 before compilation: no matching `cubecl-spirv` package in the offline
index. Cached cubecl-wgpu pre.4 requires exactly `0.11.0-pre.4` on an optional
edge; the normal Cargo index entry, archive and source directory are all
absent. The published lock records expected archive SHA256
`7c59ad637ad702bd1fd929b4e69ac560c9719dae8c31329c50cbf1fad5168b4b`.
No network request was made and no dependency edge was removed to bypass
resolution. The nine tests and deliberate missing-service negative control
have not run. Prepared vendor changes remain separate from the verified
pre.2 prerequisite; continuation requires the bounded network decision.

Full raw outputs and provenance are retained under
`Code/testing/mere/receipts/2026-09-27/burn-pre4`: `baseline.json`,
`baseline-gates.json`, both pre.2 build logs/JSON, the 496-file
`remote-fixture-pre2-aligned-source.json`, lock-delta classification,
`prepared-patch-source.json`, pristine diffs, published source hashes,
`guard-pre4-lock-update.log`/JSON and `missing-cubecl-spirv-cache.json`.
The stable target remains owned by Lane M for the pending checkpoint; no new
target, Cargo home or worktree was created.

### 13.15 Download authorization and guard execution (2026-09-27)

**Status:** bounded guard checkpoint in progress; production migration has
not been accepted. Canonical ruling 378 is in
isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md, committed there
as `a59189a`. The final question was "May I download the crates.io
dependencies needed for this pinned migration?" with the recommendation
"I recommend allowing that, recording downloads and preserving Git pins."
Mark answered verbatim: "Ok."

This settles S0-9 and supersedes §13.13's return-for-each-miss restriction:
download required crates.io dependencies for this pinned migration, record
versions and integrity evidence, and keep existing Git revisions fixed.
Unrelated upgrades and upstream communication remain outside this answer.
The working approach is Cargo-managed exact-package cache fills from outside
the workspace, then offline locked gates; the original failures remain.

The first resumed standalone lock resolution passed after exact cached-lock
packages were downloaded and verified. The gate's lock has no Git source
identities, before or after. Its helper tests run against the actual private
guard used by the launchers, not a copied predicate. Full commands, package
versions, expected/observed archive SHA256 values and outputs are retained
under the receipt directory named in §13.14, including
`guard-cache-resolution.json` and `guard-test-attempts.json`.

**Feature-scope qualification:** the standalone burn-cubecl default-feature
test explicitly enables cubecl-wgpu/default, then cubecl-server/default and
cubecl-runtime/persistence. Its compiled turso is therefore expected even
with persistence removed from the runtime's defaults. The retained
`guard-turso-feature-tree.log` exposes this chain. This guard test does not
prove turso absent from Mere's consumer feature matrix, whose burn/cubecl
edges disable defaults. The consumer graph and lock absence gate remains S8;
do not widen the runtime patch or claim its full acceptance from this test.

### 13.16 Verified guard checkpoint (2026-09-27)

**Status:** bounded S3/S4 source and guard acceptance verified, pending
coordinator review before S5–S18. Source commits remain on `burn-pre4-repin`:
fixture prerequisite `43c50fc6`, runtime `29646154`, guard `e9012d72`, and
tested standalone lock `65129b98`. Main receives documentation only.

On Rust 1.98.1, four jobs and the stable Mere target, the exact command was:

```text
cargo test --manifest-path support/patches/burn-cubecl/Cargo.toml --offline --locked --lib kernel::same_view::tests
```

The initial run passed all nine guard tests (21 unrelated tests filtered),
exit 0, compiling the actual patched crate and all three launchers. In one
supervised control sequence, removing only `&& a.service == b.service`
produced exit 101: the two service tests failed and the other seven passed.
The same-device/different-service-type failure rules out a device-only check.
The original source bytes were restored in `finally`, then the identical
command passed all nine again, exit 0. These are CPU predicate controls;
forged handles were never submitted to a GPU. Earlier cache failures are not
counted as fault controls.

Original/restored helper SHA256:
`de7f79a4c0eceacbbb5302de395ea26c7d093e8ef0b836cf755bd5e9dd89b127`.
The service-only mutant's SHA256:
`309a56e2541081839d0cc0de2c0615f3c996a209cc4342e1fc2f67ffb3e2ef30`.
The tested standalone lock stayed unchanged through all three runs, SHA256
`6e4c3d8c52be44fdc58e013b536395885a5e1c1efcd7c20228cd2742a8d173df`.
Independent review recomputed these hashes, checked the sole-line mutation,
and verified the failing and restored logs without rerunning the gate.

Fresh pristine-source comparison confirms every runtime Rust file is
unchanged from published pre.4; only its manifest and provenance note differ.
Burn-cubecl differs in its standalone manifest/lock, provenance note, three
guard blocks, module declaration and actual helper/tests. Unrelated upstream
formatting remains intact. The standalone lock moved from 524 to 523 package
identities (14 added, 15 removed); its exact classified delta is retained.
It has no Git source identities before or after. This is not the 1,710/1,655
historical consumer-lock measurement and does not prove consumer turso
absence; the default-feature qualification in §13.15 still applies.

Seven exact crates.io archives were fetched by Cargo and SHA256-checked
against the selected lock: cubecl-spirv 0.11.0-pre.4, tracel-ash
0.39.5+sdk1.4.357, pliron-spirv 0.15.0+sdk-1.4.357.0, tracel-rspirv
0.15.0+sdk-1.4.357.0, c2rust-bitfields 0.20.0,
c2rust-bitfields-derive 0.20.0 and ordered-float 4.6.0. No Git ref changed,
no upstream communication occurred, and every compiling/test gate ran offline
and locked after the concrete cache fills.

The existing external receipt directory retains full raw outputs:
`guard-test-attempt-2.log`, `guard-missing-service-control.log`,
`guard-restored-tests.log`, `guard-control-result.json`, original/mutant
source bytes, `verified-guard-source.json`, pristine diffs,
`guard-standalone-lock-delta.json`, `guard-tested.Cargo.lock`, and the package
download logs/integrity records. `guard-checkpoint-receipt.json` inventories
this completed checkpoint separately from the earlier uncompiled receipt.
No new target, Cargo home or worktree was created; Lane M retains the existing
worktree and stable target for the next reviewed slice.

### 13.17 S5–S8 bounded checkpoint (2026-09-27)

After root and independent acceptance of §13.16, the coordinator released
S5–S8 and the necessary mechanical §13.4 adapters. This section supersedes
the earlier pending-continuation status without altering those dated records.
Implementation is confined to `burn-pre4-repin`; main integration is not
authorized by this checkpoint.

**Patch rebases.** The preserved `610a32c5` pre.3 changes were compared with
pristine published pre.3 and applied to pristine pre.4. Burn Remote has 11
changed files against its 48-file published source: the two manifests,
provenance, and eight source files. Seven source deltas applied directly;
the two exports in `server/mod.rs` were inserted beside pre.4's new logging
export. The server feature keeps upstream `tracing-subscriber` and adds
`tokio/macros`. Cubek Reduce has five differences against its 93-file
inventory: the extrema helper/two call sites, manifest workspace, two retained
licenses and provenance. An independent reviewer recomputed both inventories
and found no source correctness issue. Upstream trailing whitespace in three
lines of Cubek's unit-test fixture is retained to keep its source delta exact.
The source rebase is not a headed extrema or remote lifecycle receipt.

**Pins and adapters.** All ten consumer manifests now use exact pre.4 family
requirements. The runtime patch remains under ruling 375. The three Conatus
files and remote-fixture allocator snapshot take §13.4's dynamic client and
plain memory-usage API, without changing numerical work or device ownership.
Conatus `--features resident --all-targets` compiles offline and locked,
exit 0. The nested snapshot has not yet been compiled against a pre.4 nested
lock. Nested and standalone Remote/Reduce locks retain their previous content
until their respective gates; root-lock acceptance does not cover them.

The focused Distillery `remote::tests` gate also passes offline and locked:
two tests pass, 24 are filtered out, exit 0. They exercise a live lease and
client termination on reclaim, and closing a session before authoring owner
reclaim. They use the Flex backend and do not replace the full WGPU two-peer
MiniLM lifecycle/allocator receipt required by ruling 376.

**Root graph.** The exact targeted S8 command resolves offline. The current
portable baseline has 1,648 packages and the resulting root lock 1,657:
92 identities added, 83 removed, 22 existing package blocks changed. This is
the live baseline, not Lane H's older package count. Every Git source identity
is unchanged; there is one wgpu 30.0.1 and no pre.2 family or Turso package.
The new non-family dependencies follow the pre.4 compiler/backend needs:
Pliron/LLVM and awint replace the MLIR closure, `buildid` serves the runtime,
and sysinfo advances with CubeCL CPU. The additional spin 0.12.3 and ureq
3.4.2 versions are required by pre.4 and its LLVM bundler; existing versions
remain for their other consumers. Existing-package edge convergence includes
Windows-sys 0.61.2, data-encoding-macro-internal's permitted syn 3.0.6 edge,
and feature-selected rand/HTTP edges. No existing package takes an unrelated
major-version upgrade, and no Git revision moves.

`cargo_mode.py verify --metadata-only` exits 0. The complete Distillery
`remote,trainer-gpu,trainer-autodiff,flora` and Conatus `resident` feature trees
both contain the patched runtime and omit Turso and runtime persistence.
ESP with defaults disabled contains neither Burn nor tokenizers. A single
detector invocation also detects Turso in the retained standalone-default
guard graph/lock, providing a positive control for the absence checks; it is
explicitly a fresh comparison of retained raw evidence, not a new standalone
Cargo run. Three tree commands exited 0 and retained complete logs/provenance
before their wrapper's console-only cp1252 rendering error; the wrapper now
uses UTF-8. No Cargo result or raw output was lost.

Full raw command output, source hashes, exact deltas and controls remain in
the external `Code/testing/mere/receipts/2026-09-27/burn-pre4` receipt directory:
`burn-remote-pre4-pristine-delta.json`, `cubek-reduce-pre4-pristine-delta.json`,
`s8-targeted-lock.json`, `s8-root-lock-delta.json`, `s8-graph-findings.json`,
`s8-absence-control.json`, `s8-portable-metadata.json` and
`s8-conatus-check.json`, with their raw logs. The root lock SHA-256 is
`2e85b0eac6c5f878d29c0a2487fb38c3767190926985dc3f5f4096814945f4b5`.
The only additional download was pristine Burn Remote pre.3 as a rebase
reference, with matching crates.io/archive checksum retained. It changes no
production pin. S9–S13 broader matrices and behavioral receipts remain subject
to the coordinator's next release.

`s5-distillery-lease-tests.json` retains the full focused-test log and source
provenance. `s8-final-source.json` explicitly distinguishes source hashes from
the two removed Cargo-cache metadata files in the Remote vendor directory;
the earlier gate inventories record those tracked deletions as null. No source
changed during graph review. `s8-checkpoint-receipt.json` inventories this
checkpoint separately from the guard receipt. The existing Lane M worktree
and stable Mere target remain owned by this ongoing migration.

Separable Lane M source commits: `e2f335d7` (Remote rebase), `ed513bcb`
(Reduce rebase), and `439a3585` (exact consumer pins, root lock and adapters).
All remain outside main pending the complete migration acceptance gates.

### 13.18 S9–S12 numerical stop and partial matrix (2026-09-27)

The coordinator accepted `5accdcb8` and released S9–S12 in the existing Lane M
worktree, with S13 still closed. This annotation supersedes the pending-state
summary in §13.17 only for the gates actually completed below. Main remains
pre.2; the lane retains the pre.4 root lock from §13.17 unchanged.

**Completed checks.** All nine native test commands and all 19 wasm matrix
rows pass. Native passing counts, per command rather than unique tests, are:
ESP default 92, BERT 158, index WGPU 99, decoder WGPU 136, decoder training
160, model-session 103, persistence 100, Numen 75 and Seiche 91. Existing
ignored tests remain separately enumerated in the raw logs. Distillery's full
`remote,trainer-gpu,trainer-autodiff,flora` check, Djinn's
`trainer-gpu,trainer-autodiff` check and `cargo_mode.py verify` all exit 0.
The latter includes the portable `cargo check --workspace --all-targets
--locked`; its 1,530 selected metadata packages are not the full lock's 1,657
package identities. The earlier two focused lease tests remain valid.

**Numerical stop.** Numen's release `field-burn-wgpu` suite passes 77 tests,
including both scalar and vector NdArray/WGPU parity tests; its timing test
is ignored. Seiche's release `tensor-burn-wgpu` suite exits 101: 85 pass,
two fail, one timing test is ignored. Both
`tensor_forces::tests_wgpu::parity_ndarray_wgpu` and
`node_exclusion_parity_ndarray_wgpu` fail. The latter reports x relative error
`1.9973466e0` against its unchanged `1.0e-3` limit. The former reports
`fx diverged` against its unchanged absolute `1.0e-3` limit. This is a finite
numerical mismatch, not an adapter skip or a non-finite-value rejection.
No cause or equivalence to the fresh pre.2 baseline is claimed yet.

Before this run, source review found that `f32::max` could hide NaN in ESP's
synthetic BERT and both Seiche parity tests; Seiche's `zip` could also truncate
unequal outputs. Prepared test-only helpers assert equal lengths and finite
values on both sides before the existing folds. Production algorithms and
tolerances are unchanged. Seiche's three new controls passed in the same run:
finite inputs accepted, NaN and both infinities rejected on either side, and
both unequal-length directions rejected. The six non-finite and two length
rejections are retained in stdout. The corresponding ESP helper/controls are
source-reviewed but have not been compiled or run yet. Earlier native/wasm
and workspace receipts precede these two test-only changes; no claim extends
the old receipts to the new ESP tests.

The GPU sequence stopped here under the numerical stop rule. Conatus's nine
resident tests, ESP's release BERT/MiniLM gates, all eight nested/standalone
builds and S13 headed/lifecycle receipts remain pending. Fresh pre.2 comparison
must preserve the lane's prepared changes and primary `reader.rs` WIP; neither
an arbitrary old snapshot nor an assumed baseline pass closes this failure.

**Resolved locks and selected graphs.** Targeted offline resolution passed
for all six nested workspaces and the two standalone patch crates. Counts are:

| Root | Before | After | Selected graph qualification |
|---|---:|---:|---|
| probe | 600 | 585 | Patched runtime; no Turso/persistence |
| native-fixture | no prior lock | 626 | NdArray only; no runtime/Turso selected |
| remote-fixture | 969 | 982 | Patched runtime; no Turso/persistence |
| session-fixture | 577 | 662 | NdArray only; no runtime/Turso selected |
| burn_browser_embedding | 570 | 552 | Patched runtime; no Turso/persistence |
| cubek_browser_extrema | 570 | 552 | Patched runtime; no Turso/persistence |
| standalone burn-remote | 679 | 761 | Upstream default/dev closure retains Turso |
| standalone cubek-reduce | 491 | 491 | Upstream default/dev closure retains Turso |

This is five tracked nested locks plus native-fixture's generated ignored lock,
with standalone locks inventoried separately. A dated clarification to §13.17:
the standalone Remote/Reduce starting locks were the published pre.4 lock bytes
copied during pristine-source rebasing, not the former pre.2 standalone locks.
Their previous-content wording must not be read as a pre.2 receipt.

All prior Git source identities remain fixed. Every root has one wgpu and no
pre.2 or duplicate Burn/CubeCL/Cubek family. Probe and both repros retain their
existing wgpu 30.0.0; the other roots use 30.0.1. CPU-only native/session roots
have no historical patch table and retain optional Turso lock entries; their
exact selected feature trees contain neither runtime nor Turso. One fresh
detector invocation checks all six selected trees against the retained
standalone-default positive graph, which detects Turso and three runtime
persistence feature occurrences. This proves selected-graph absence, not
universal absence from optional lock closures. Independent read-only review
recomputed all eight lock hashes, deltas, graph hashes and fixed Git identities.
Build acceptance remains pending.

Ruling 378 covered three needed registry cache fills: burn-communication
`0.22.0-pre.4`, thread-tree `0.3.3` and cubek-test-utils `0.3.0-pre.4`.
Cargo-managed downloads and archive checksums match their retained lock
provenance. Missing-cache exits are retained and are not negative controls.

**Evidence and ownership.** Full command, compiler, source and lock provenance
is external under `Code/testing/mere/receipts/2026-09-27/burn-pre4`, including
`s9-matrix-commands.json`, `s9-s12-matrix-summary.json`, `s9-gpu-numen.json`,
`s9-gpu-seiche.json`, `s9-seiche-numerical-stop.json`, the S10/S11 logs,
`s12-nested-lock-deltas.json`, before/after lock bytes and
`s12-feature-absence-control.json`. Each completed gate retains complete raw
stdout/stderr. Source maps are unchanged during the failed Seiche run; its log
SHA-256 is `1c6721bbcfd21eef52f9c6663a3ee461f01563133e90e6fd86de195b7828f52f`.
The source-reviewed test changes and five tracked nested locks remain prepared
WIP rather than accepted implementation. Existing Lane M worktree and stable
`C:/t/cargo-targets/mere` remain retained for this migration; the GPU slot is
released and no new worktree, target or Cargo home was created.

### 13.19 Fresh pre.2 Seiche comparison (2026-09-27)

The coordinator released one bounded diagnostic on primary `f4f61d6c`, using
only the identical Seiche finite/length test hunk from Lane M. The target
source was clean and matched baseline Git blob
`39b7e3d47f2a2a46e6ba5181c58d509740706ec4`. Primary LF line endings were
preserved; Lane M's CRLF bytes normalize to identical tested Rust source.
There were no active Cargo source redirects or competing compiler/test owner.
The pre.2 lock remained
`da0822f516010218b3b163234d9826955add552002cffadf7250b5932add13d4`.

The command matched the failed pre.4 gate exactly:
`cargo test --release --offline --locked -p seiche --features tensor-burn-wgpu
-- --test-threads=1 --nocapture`, with Rust 1.98.1, four jobs and the stable
Mere target. Both sources select discrete GPU 0; neither run sets WGPU backend
or adapter-name overrides. Current hardware/driver inventory is retained as
supporting provenance, not a new device-selection claim.

Pre.2 also exits 101 with **85 passed, two failed, one ignored**. The same two
GPU parity assertions fail, with node-exclusion x relative error
`1.9973466e0` against `1.0e-3`. All three finite/length controls pass, including
six non-finite and two unequal-length rejections. Thus the observed failure
exists in the preserved pre.2 baseline under the same checks; this does not
prove every output value identical across versions or close numerical
acceptance. Production algorithms and tolerances remain unchanged.

The wrapper restored the original Seiche bytes in `finally`. Every recorded
source hash matches its pre-run value, including the original root lock and
unrelated dirty `document-lanes/src/reader.rs`; primary status returned to
that reader WIP alone. No dependency download, source redirect or new target,
Cargo home or worktree was needed. No broader matrix or S13 gate resumed.

External evidence is `s9-seiche-pre2-baseline.log/.json`, its original/tested
source copies, patch, manifest/lock copies, adapter inventory and
`s9-seiche-pre2-pre4-comparison.json` in the existing receipt directory. The
pre.2 raw log SHA-256 is
`dc875500fdc44d8f13487d9742eb754f39afd5b8eff2c1084d7ccc6a4fb15a1a`.

**Proposed next diagnostic, not executed:** compare the common column-minus-row
broadcast displacement stage on NdArray and WGPU for the existing 257 positions
and an asymmetric three-point analytic control. Record complete arrays and
first mismatch before instrumenting later arithmetic/reduction stages. This
localizes the shared failure without changing either law or its thresholds.
The coordinator retains the next release decision.

### 13.20 Displacement capture and alias-layout localization (2026-09-27)

The coordinator released only the displacement-stage diagnostic and read-only
source localization. Lane M remained at `4673fe30` and root lock `2e85b0ea…`;
no new rendering closure from primary was absorbed. Temporary test code used
the same clone/reshape/consume subtraction expression, the existing 257-point
positions, and an asymmetric three-point analytic control. It recorded every
input, scalar expected, NdArray and WGPU value for both axes, with n² length
and finite checks. The prepared Seiche bytes were restored exactly afterward.

NdArray matches scalar expected values bit-for-bit in all four cases. For
three x coordinates `[-2, 1, 5]`, expected row-major differences are
`[0,-3,-7, 3,0,-4, 7,4,0]`; WGPU returns
`[0,-3,-7, 1,1,1, 5,5,5]`. Each three-point axis has six of nine mismatches.
For 257 points, x has 65,536 of 66,049 mismatches (first row 1, column 1),
and y has 65,792 (first row 1, column 0). All values are finite. The capture
test exits 0 because it records rather than asserts parity; this is not an
acceptance pass. The comparison parser independently rounds logged decimals
back to f32, checks host subtraction, and detects a planted single-value error
and a matching control in the same invocation. Independent review recomputed
all arrays, mismatches, hashes and restoration.

**Source finding.** Pre.4 `ops/tensor.rs::float_sub` delegates to
`numeric::sub`, then generic `launch_binop::<SubOp>`. Broadcast shape is
`[n,n]`; neither `[n,1]` nor `[1,n]` votes for a memory-order permutation, so
logical order remains. The ordinary contiguous reshape path updates metadata,
while cloning retains the shared handle and copies metadata. The same-view
predicate examines the handle's six identity/view fields, not tensor shape.

In all three patched same-view arms, output allocation and launch work count
already use the broadcast shape. The LHS receives
`into_linear_view_like(&output)`, but RHS receives
`as_linear_view_alias(0)`. `tensor/base.rs` constructs the latter with
`LinearViewLayoutLaunch::new()`, omitting a reference shape. CubeCL's linear
layout chooses broadcast coordinate mapping when a differing reference shape
is supplied; a contiguous RHS without it uses its own plain indexing.
Pristine pre.2/pre.4 fresh-output arms broadcast both inputs. Our patched
pre.2 has the same missing-reference alias construction as patched pre.4.
This identifies a source defect in the existing local alias branch, not in
the broadcast output allocation or the six-field identity predicate.

**Candidate, not implemented:** add an alias-aware broadcast-view helper that
combines `self.as_tensor_alias(input_pos)` with
`LinearViewLayoutLaunch::from_reference_shape(reference.shape())`, then use
it for RHS in all three same-view arms. Keep identity comparisons, separate
output allocation, zero-output handling, memory-order placement and numerical
thresholds unchanged. Regress actual generic subtraction, float atan2 and an
integer operation through their respective launchers, including same-buffer
broadcast, equal-shape alias and independent-allocation controls. Restoring the
old RHS layout must fail the new regression; unchanged full-force gates must
then pass before acceptance.

The diagnostic preserves the source-level clone/consume pattern but immediate
`into_data()` may change materialization/fusion relative to full force laws.
It does not directly observe handle IDs, reference counts or the executed
launcher. The source defect and observed pattern are strong localization,
not proof that every full-force failure is resolved by the candidate.
No later arithmetic stage or production correction was executed.

The repair-order question is pending: repair pre.2 separately and carry the
verified correction into pre.4, or repair only the migration lane. This entry
records no new ruling. Full raw capture/provenance, original/instrumented
source, four complete array files, comparator controls and source hashes are
external as `s9-displacement-*`, `s9-localization-pre2-*` and
`s9-alias-broadcast-source-localization.json` in the existing receipt directory.
The GPU slot is released; prepared lane changes and stable target are retained.

### 13.21 Ruling 380: repair pre.2 first (2026-09-27)

The user selected **A**, verbatim: `A`. The accepted option was: "Fix pre.2
separately, then carry the verified correction into pre.4 (recommended). The
existing bug gets its own tested commit." Authority is Isometry's canonical
wing design record, ruling 380, on commit `e6583a8`. This supersedes the pending
repair-order question in §13.20; the historical diagnostic qualification stands.

Implementation starts on primary Mere `be2e710a`, including its published
rendering closure. The older `f4f61d6c` diagnostic remains evidence for that
source only. Preserve concurrent `document-lanes/src/reader.rs`, Lane M's
prepared migration files, all five pre.2 allocation comparisons, separate output
allocation and existing numerical tolerances. The bounded correction adds an
alias view with the output reference shape and uses it in the three guarded
launchers. Nine direct, unfused cases cover subtraction, float atan2 and integer
XOR, each with broadcast alias, equal-shape alias and separate allocations.
Verify real handle identity, scalar expected values, output shape/length/finite
values, fresh output and preserved inputs. An old-layout-only fault must fail
regression, and the original full Seiche force gates must pass with the finite
and length checks retained. Independent review precedes the separate pre.2 fix
commit; carrying it to pre.4 and further migration gates await coordinator release.

Status: authorized, implementation and numerical acceptance pending. Reuse
`C:/t/cargo-targets/mere`, four jobs, and the existing external receipt directory.

#### Verified pre.2 checkpoint (2026-09-27, ruling 380)

The correction and tests passed independent source/receipt review and coordinator
acceptance before the separate source commit. `as_linear_view_alias_like` preserves
input-zero alias binding while supplying the output reference shape; exactly the
three guarded RHS calls use it. All five pre.2 identity comparisons, separate
output allocation, force formulas and four existing `1e-3` assertions are unchanged.

Nine direct private-launcher cases passed, with actual logical-view identity,
output allocation identity, shape, length, finite float values, scalar expected
values and input-preservation checks. Replacing only the new helper's
`from_reference_shape(reference.shape())` with `new()` caused exactly the three
broadcast-alias cases to fail and the other six to pass. Exact byte restoration
then passed all nine again. This final sequence includes the strengthened
allocation-identity assertions. Commands use the patched crate's standalone
manifest, release mode, `--no-default-features --features std,fusion --lib
kernel::alias_broadcast_tests -- --ignored --test-threads=1 --nocapture`. The
dev-only dependency is `burn-backend =0.22.0-pre.2` with `cubecl-wgpu,std`.

The current primary root command `cargo test --release --offline --locked -p
seiche --features tensor-burn-wgpu -- --test-threads=1 --nocapture` passed 87 unit
and 9 integration tests, with one ignored test. Both formerly failing force
comparisons executed and passed. The finite/length rejection controls passed.
After this run, only the direct fixture's fresh-allocation assertions and the
ordering of its `cfg(test)` module declaration changed; its final direct/control
sequence was rerun. Production operations and Seiche test bytes stayed identical.
Exact Seiche-time fixture/module copies were recovered and checked against the
recorded hashes, preserving this qualification.

Baseline is primary `be2e710a`, root lock SHA-256 `f00fbbfa8207f694e9d9d22edbe3a71d400de1c97a4483e1cf2ed3a0592acb08`.
The standalone lock remained byte-identical at 417 packages, SHA-256
`95771a4e7767f993df7f12aa4c18aac2e2d161751428a7f8bdecb0c9ab011a27`.
Reader WIP retained SHA-256 `69bf981ddc67688327a07488c6ff6485fd0f61c08169b0052a5e622b2ec0f42f`.
Five exact missing cached crates were downloaded under ruling 378 with archive
checksums recorded. Failed cache/fixture-compilation attempts remain in receipts
and are not counted as fault controls. Changed-file formatting was checked;
five comma suggestions reproduce on pristine HEAD and were preserved.

Full stdout, commands, compiler/device/process context, source hashes, old-layout
bytes and restoration are sealed by `pre2-repair-checkpoint.json` in the existing
external receipt directory. Other renderer/headed work could run concurrently:
these are correctness gates, with no exclusive-GPU or timing claim. The stable
Mere target remains shared and reusable. Lane M remains parked with its prepared
files; carrying this accepted correction into pre.4 awaits coordinator release.

### 13.22 Bounded pre.4 carry (2026-09-27)

The coordinator verified the published pre.2 commit `a016f86f9b47459425b6f10dded7982853b9bd6b`
and released only its alias-layout correction and direct regressions into existing
Lane M `1f5a7319`. This is not a merge or rebase onto primary's newer renderer
closure. Seven prepared files and current root/standalone lock bytes are retained.
The non-generic pre.4 tensor/launch APIs and CubeDevice are used; the fixture calls
the actual six-field `same_view` helper and checks allocation descriptor IDs.
Existing test-runtime dependencies suffice, so no manifest adaptation is needed.

Status: direct pass/fault/restoration, nine unchanged identity tests and the full
Seiche release gate are running or pending. The six-field helper remains exactly
`de7f79a4c0eceacbbb5302de395ea26c7d093e8ef0b836cf755bd5e9dd89b127`;
output allocation, zero-size returns, memory-order handling and tolerances are
unchanged. Independent review precedes any carry commit. Broader matrices, S13
and renderer reconciliation remain outside this release.

#### Completed bounded carry gates (2026-09-27)

The nine adapted direct launcher cases passed. Removing only the alias helper's
broadcast-reference construction produced exactly three broadcast-alias failures
with six passing controls; exact restoration passed all nine again. Generic
subtraction, float atan2 and integer XOR retain the pre.2 scalar expectations,
length/finite checks, output shape, input preservation and fresh-allocation checks.
No new manifests, lock updates, downloads, renderer pins or numerical thresholds
were needed. All six production identity predicates remain byte-identical.

All nine identity-helper tests also passed from the existing corrected-source
test binary while Cargo waited on another owner's active package-cache resolver.
The receipt records the binary hash, precise filter and build-source receipt.
This is historical binary evidence: the subsequent fault/restoration rebuild
replaced the executable. Source identity for the guard remained unchanged, and
the coordinator independently checked that binary before replacement.

The complete Seiche release command passed 87 unit and 9 integration tests, with
one ignored test, on the final corrected source. Both original GPU force parity
tests and finite/length rejection controls passed at their original tolerances.
Direct and full-force runs retained complete stdout, before/after source maps,
compiler, device and concurrent-process context. Correctness was checked with
possible unrelated renderer/headed work; no timing or residency claim is made.

Root lock remains `2e85b0eac6c5f878d29c0a2487fb38c3767190926985dc3f5f4096814945f4b5`;
standalone lock remains `6e4c3d8c52be44fdc58e013b536395885a5e1c1efcd7c20228cd2742a8d173df`.
All seven prepared WIP paths are byte-preserved, including the Seiche/ESP test
hardening and five nested locks. They are outside the carry's source commit.
Changed-file formatting was checked: seven old match-arm commas and the existing
`same_view` module ordering reproduce on HEAD and remain unchanged.

`pre4-carry-checkpoint.json` seals commands, raw logs, exact mutation/restoration,
fixture API adaptation, lock/source hashes and preservation checks in the existing
external receipt directory. This bounded checkpoint does not repeat or accept
the broader migration matrices, nested builds or S13. Keep the existing Lane M
worktree and stable Mere target for those separately released gates.

### 13.23 Remaining S9 GPU and S12 builds resumed (2026-09-27)

Following the user's "Let’s proceed," the coordinator released Conatus's nine
resident GPU cases, ESP synthetic/real-MiniLM parity and the eight prepared
nested/standalone builds at Lane M `8d308572`. Read the dated overrides to the
older target-path and baseline instructions: reuse `C:/t/cargo-targets/mere`,
four jobs, offline builds, and the exact existing closure. No new worktree,
Cargo home, rebase or renderer-pin absorption is part of this release. Primary
Mere's later capture-hook commit does not change this lane's source baseline.

Tests retain full output, compiler/source/lock context and model hashes. Native
GPU runs are correctness checks with possible unrelated device-host activity,
not timing receipts. Conatus adapter-skip messages must be detected explicitly;
ESP's real-model test is selected with `--ignored` and the existing external
MiniLM artifact. Preserve the finite/length controls and all current tolerances.
Stop on numerical failures or substantive forks. Return an independently reviewed
checkpoint before committing the seven prepared files or entering S13.

Status: running; previous native/wasm, Distillery/Djinn/workspace and Numen
receipts remain source-qualified as recorded. This release does not silently
repeat or upgrade them to final integration acceptance.

#### Focused refresh and first result (2026-09-27)

The coordinator also released a fresh Numen GPU suite on the corrected alias
source: its earlier 77-pass receipt predates the three production binary-path
changes. This is a focused behavioral refresh, not a repeat of the whole matrix.
Conatus's four resident and five chunk tests passed with the actual CubeCL kernel
marker and zero adapter-skip messages. The same log detector finds one planted
skip line; that is a parser control, not a forced adapter failure. Successful
tests do not print their numerical values, so the receipt claims their tested
bounds and lease/allocation assertions rather than invented measured errors.
See `s9-remaining-conatus.json`, its complete log and
`s9-remaining-conatus-execution-control.json` in the existing receipt directory.

ESP's release synthetic CPU/WGPU parity and three actual finite/length helper
controls passed (four tests; real-model and timing tests ignored in that run).
The real MiniLM fixture was then explicitly selected with `--ignored` and passed
with its 384-component finite output, original `1e-4` fixture/norm tolerance and
stage-trace assertions. Six local model files match the recorded hashes. These
are `s9-remaining-esp-synthetic` and `s9-remaining-esp-minilm` JSON/raw-log pairs;
the timing test was not run. All gate source maps are unchanged before/after.

The focused final-source Numen refresh passed 77 tests, including scalar and
vector GPU parity, with the timing test ignored. Receipt:
`s9-remaining-numen-alias.json` and its raw log. S12 build attempts now proceed
serially. Initial standalone Remote cache misses are retained separately;
ruling 378 cache fills use Cargo for exact locked crates.io versions and compare
downloaded archive checksums with the unchanged lockfile. They are not test
failures or deliberately broken correctness controls.

The remote fixture build exposed two mechanical pre.4 API adaptations: replace
the removed `WgpuDevice::DiscreteGpu(0)` constructor with
`WgpuDevice::new(WgpuDeviceKind::DiscreteGpu(0))`, and wrap the same cloned device
as `cubecl::Device::Wgpu` when mounting the server. Its emitted backend version
now truthfully says pre.4. The original and intermediate compiler failures,
exact source diff and before/after bytes are retained; the corrected build
`s12-remaining-remote-fixture-api2` passes. No device-selection policy, tolerance,
production algorithm, manifest or lock changed. This fixture-only difference
does not require repeating the already source-qualified production GPU gates.
File-only formatting checks reproduce an existing import-order suggestion on
the exact pre-edit bytes; it is preserved rather than mixed into this adapter.

#### Boundaries for the next checkpoint (2026-09-27)

Earlier native/wasm compile matrices and S10/S11 commands retain their recorded
source closure; their passing results do not silently verify the later alias
correction or primary's newer rendering pins. The fresh Seiche, Conatus, ESP,
Numen and S12 receipts identify their tested source separately. Before final
integration, a planned reconciliation with current primary must inspect exact
manifest/lock/source deltas, repeat S8 graph/identity/absence checks and S11
whole-workspace verification, and select affected S9/S10/nested consumer rows
from those actual deltas. Any changed numerical backend requires its affected
parity gates again. This paragraph identifies the review boundary; it does not
authorize a rebase or claim a new closure already passed.

S13 remains unreleased. Its older scripts' arbitrary/default target locations
do not override workspace hygiene: any later authorized run must explicitly
reuse `C:/t/cargo-targets/mere` and the pinned `wasm-bindgen` CLI `0.2.122`.
Headed, mutation and full two-peer lifecycle evidence remains pending.

#### Completed remaining S9/S12 checkpoint (2026-09-27)

All twelve commands exit 0: four GPU test commands (Conatus 9, ESP synthetic
and rejection controls 4, explicit real MiniLM 1, Numen 77) and eight builds.
The builds are standalone Remote/Reduce libraries; the probe and both browser
repros for Wasm; and native, remote and session fixtures for the host. The
complete commands and successful retry labels are in
`s12-remaining-build-commands.json` and `s12-remaining-build-results.json`.
`remaining-gates-summary.json` checks every raw log hash, exit, before/after
source map, current-source difference and preserved model/manifest/lock byte.

All eight nested/standalone manifests and locks match their earlier reviewed
resolution. The seven prepared paths and root lock remain byte-identical.
The only later source difference in the early GPU/build receipts is the
remote fixture adapter described above, with its own successful rebuild.
The four exact cache fills were `axum 0.8.9`, `axum-core 0.5.6`, `matchit 0.8.4`
and `serde_path_to_error 0.1.20`; each downloaded archive matches the unchanged
standalone Remote lock checksum. All failed cache and compiler attempts remain.

Keep graph qualifications intact: standalone Reduce defaults select Turso;
that is not a production persistence-absence gate. Production selected graphs
retain the earlier positive-control evidence. Probe and both repro locks use
WGPU 30.0.0; other reviewed roots use 30.0.1. Remote's unused `p2panda-auth` and
`p2panda-stream` patch warnings are the previously recorded mirrored optional
rows, not a reason to invent dependencies. Other gates retain their existing
upstream deprecation/dead-code warnings; no warning-free claim is made.

`remaining-s9-s12-checkpoint.json` seals this bounded result and the final
source/docs inventory under `Code/testing/mere/receipts/2026-09-27/burn-pre4`.
Independent review is requested before committing the seven prepared paths,
fixture adapter and owning docs. No new source acceptance, S13, renderer
reconciliation or main integration follows automatically. The existing Lane M
worktree remains needed; the stable Mere target is released to the waiting
consumer lane. No isolated Cargo home or alternate target was created.

### 13.24 Current-main reconciliation checkpoint (2026-09-28; held)

The accepted §13.23 checkpoint was committed and pushed as `a7c477e7`.
The next authorized slice merges exact committed main
`5ce144ffe58945746b7dabc21de499725219beaf`, retaining history and excluding
primary's raw `reader.rs` work. The merge is prepared in the existing Lane M
worktree and is not committed or accepted. Evidence lives under
`Code/testing/mere/receipts/2026-09-28/burn-pre4-reconcile`.

The overlapping pre.2 repair on main is already represented by the verified
pre.4 carry. All Lane M patch bytes, including the six-field identity helper
`de7f79a4…`, are preserved. Twelve incoming renderer, capture and publication
paths match committed main after newline normalization. The root manifest's
three-way semantic merge is checked leaf by leaf. Both dated documentation
histories are retained.

The root lock grows from 1,657 to 1,660 packages: 38 identities enter, 35 leave,
and 22 retained package blocks change. The new identities comprise 33 Genet,
four NetRender and registry `netrender-vello 0.10.1`; the latter replaces
0.10.0. Current workspace Genet is `7b48f94d` and NetRender `9607d16f`.
The old Genet Fleece/layout API identities remain under the unchanged Knot
pin, as they do on accepted main; they are not an extra workspace repin.
There is one root WGPU (30.0.1), 62 unique Burn/CubeCL/Cubek packages, and no
pre.2 family or root-lock Turso. Production Distillery, Conatus and ESP BERT
feature trees select the patched runtime without persistence or Turso. The
same detector finds Turso and three persistence occurrences in the fresh
standalone WGPU positive-control tree. The standalone default forward tree
alone did not expose runtime persistence and is not used as that control.

Numen, Seiche and Conatus complete lock cones are unchanged. Conatus's selected
feature tree is identical to its earlier accepted tree. ESP's changed optional
Genet nodes are absent from the selected BERT GPU graph; its minimal graph
still selects neither Burn nor tokenizers. The numerical implementation and
patch bytes are unchanged, so the prior source-qualified numerical receipts
remain applicable without repeating those runs. This is a selected-graph
qualification, not a claim that every feature combination is unchanged.

Two initial whole-workspace attempts exposed the new optional accessibility
description field. Lane M adopts exactly the Genet owner's committed
`mere-adoption.patch` in its 2026-09-26 accessible-name receipt: two
`description: None` reader initializers. One web-host blank test initializer
needs the same field. `api-adaptation-verification.json` proves the exact
three-line delta and owner patch identity. The primary reader's raw hash
`69bf981d…` remains untouched; it does not contain these fields. Earlier
wording that attributed the adoption to primary WIP was incorrect.
The focused reader all-targets check, all six web-host tests and changed-file
format check pass. The final whole-workspace `cargo_mode.py verify` passes.

Distillery's four-feature all-targets check, both remote lease tests, Djinn's
trainer check, and all 17 Mesquite tests pass. The affected remote fixture
build passes. Only its lock among the eight previous S12 roots contains the
changed renderer identities: exactly five Genet and one NetRender source
substitution, with the count unchanged at 982. The initial targeted solver
also moved `data-encoding-macro-internal`'s edge from `syn 3.0.4` to 1.0.109.
That unnecessary variant is retained externally, not accepted. The narrower
candidate preserves the original 3.0.4 edge and passes unchanged offline,
locked metadata. The earlier claim that 1.0.109 matched root was incorrect;
current root uses 3.0.6. All seven unaffected nested/standalone locks retain
their accepted bytes and qualifications.

The ignored Graphshell web lock is copied byte-for-byte from the accepted main
publication (`be471fff…`) after checking identical manifests. Fresh locked
metadata resolves 527 packages, including 18 current Genet packages and no
Burn family or outside path. The Wasm check passes with the recorded
`getrandom_backend="wasm_js"` configuration. The remote feature tree contains
the patched runtime and excludes persistence/Turso. Full command, compiler,
environment, source, lock and separate stdout/stderr evidence is retained.
Every command uses explicit Rust 1.98.1, four jobs, offline mode and
`C:/t/cargo-targets/mere`; no cache download, alternate target or home is added.

**Unresolved acceptance gate:** the complete rootstock suite reports 40 passes
and one failure in
`equal_hover_cascade_retains_geometry_text_generation_and_scroll`: the element
offset is `(0, 0)` instead of the unchanged expected `(0, 12)`. Both GPU
producer tests pass, but this does not waive the scroll failure. A focused run
on untouched exact primary `5ce144ff` reproduces the same assertion. Its full
before/after source and lock maps match, including primary reader raw bytes.
This locates the failure on the already-published main closure; it does not
make the behavior acceptable or prove a repair. No assertion, tolerance or
production behavior is changed. Owner diagnosis and a separately reviewed
repair remain pending.

`reconciliation-verification.json` verifies 21 passing commands, four preserved
failed commands, their log hashes and source stability. The failures are the
two earlier API compile attempts, the rootstock suite and its primary
comparison; they are not deliberate negative controls. The checkpoint seal
records final source/docs, locks and receipts. The stable target is released;
the existing Lane M worktree remains necessary for this held merge. A clean,
reviewed reconciliation commit must precede S13. No merge acceptance, S13
execution or integration follows from the independent passing gates.


### 13.25 Published-main reconciliation verification (2026-09-29)

The held checkpoint `387a8dd2` first received exact published main
`32edc2ad`. Its source/graph-only integration was independently reviewed and
committed locally as `0ea65fb6`, explicitly PRE-BUILD, NOT ACCEPTED, S13 HELD.
Exact published `99e448535985b7b8f4908c95fbedbba79bf76d03` then joined that
history before the final affected tests. Both merges preserved all 374
tracked pre.4 patch files, including the six-field identity guard
`de7f79a4…`; no pre.2 patch replacement or S13 fault edit occurred.

Evidence is split into immutable `pre4-before-metadata`,
`pre4-published-main-reconcile` and final `pre4-semantic-reconcile` folders
under `Code/testing/mere/receipts/2026-09-29`. Each gate records its exact
command, Rust 1.98.1 environment, separate logs, source/lock hashes and source
modification times. All 3,026 source/lock inputs remain stable during every
accepted gate. The stable `C:/t/cargo-targets/mere` target is reused with four
jobs and incremental compilation disabled; no isolated Cargo home is added.

All eleven locked metadata roots and the selected feature controls pass.
Unfiltered metadata contains 1,533 packages at root, 636 in Graphshell web
and 982 in the remote fixture; these are not browser payload sizes. Current
workspace Genet is `19c20687`, and the accepted Knot source is `855cb75d`.
The first integration added the approved host/Wasm 0.2.127 and Insigne graph
changes. The final semantic integration changes source identities and adds
only the selected Taproot → document-session-api edge in root and web.
Taproot also declares a dev-only genet-render dependency, which is absent
from those consumer resolutions. The web resolve graph matches accepted
primary while five inactive optional Burn declarations retain pre.4.

The remote lock retains all 982 package identities and its original
`data-encoding-macro-internal` → `syn 3.0.4` edge. It adds exactly three
required Insigne edges (Distillery, Mere Mesh and Pandect). The initial
source-only candidate failed locked metadata; the four-edge candidate also
failed because Mere Transport's Insigne dependency is dev-only here. Both
attempts and the bounded solver probe are retained. The accepted candidate
removes only that extra dev edge, preserves all other fields, and passes
locked metadata. Eight unaffected roots keep identical locks and graphs.

Every root's source/path detector rejects planted bad input. Production
Distillery, Conatus, ESP BERT, minimal ESP and remote feature trees exclude
Turso and runtime persistence. The same-run standalone WGPU control detects
Turso and three persistence occurrences. This is scoped dependency evidence,
not a blanket SQLite-free claim.

Fresh affected gates pass:

- Whole-workspace `cargo_mode.py verify`; Distillery all-targets with
  `remote,trainer-gpu,trainer-autodiff,flora`; both remote lease tests; Djinn
  all-target trainer check; Reader all-target check; remote-fixture build;
  and Graphshell Wasm all-target/all-feature check with
  `getrandom_backend="wasm_js"`.
- Rootstock, both native/web hosts and Mesquite: 205 passing tests and four
  existing ignored doctests, including all 44 Rootstock tests and 21 scenario tests, including the semantic cases. Pictograph canvas/Vello: 263 passing tests, including its
  headless crossing control.
- Seiche runtime: 12 default and eight no-default tests. Full release
  tensor/WGPU suite: 107 passing
  tests, with the existing timing comparison ignored. Both WGPU force parity
  tests and their finite-input rejection tests pass.
- Conatus release resident/resident-chunk: nine passing tests, with a positive
  GPU kernel marker and zero adapter-skip messages. The same skip detector
  finds its planted message.
- ESP release synthetic parity/controls: four passing tests and two existing
  ignored tests in that invocation. The explicit real-MiniLM test passes separately using the same six model-file
  hashes. No assertion or tolerance is widened.

**Carry correction:** unchanged crate files and dependency selection alone
did not prove unchanged dependency source. Conatus's lock closure includes
the changed Seiche runtime. ESP's selected test cone includes changed
Personae library source through Eidetic's default pack-signing path. The
initial carry audits are preserved as qualified attempts; both consumers
were freshly tested above. Only Numen carries earlier execution evidence:
its complete 584-package lock closure and local source closure are unchanged.
GPU runs establish correctness, without an exclusive-device or performance
claim. The local ESP MiniLM test does not satisfy S13's remote lifecycle gate.

**S13 checklist clarification, dated 2026-09-29:** the current browser fixture
contains ten graph cases (eight initial cases plus two LayerNorm cases), and
eleven embedding cases (eight initial cases plus three grouped cases).
Require all ten plus eleven; the earlier four-graph-case wording is
historical. The named unpatched raw shared multiply failure and both
LayerNorm input-bit-identity failure conditions remain unchanged. The remote
receipt must also report error no greater than the exact pre.2 baseline
`1.4901161193847656e-7`; its built-in `1e-4` test tolerance alone is insufficient.
S13 has not run at this checkpoint. Independent review and a clean
reconciliation commit still precede its release; migration acceptance, S15
closure and primary promotion remain separate gates.


### 13.26 Unpatched browser pass; retirement fork (2026-09-29)

The independently reviewed reconciliation was committed locally as
`6b52297f5852059735dff6073f20741e67ce1b81`, with clean sources and all 33
accepted graph/runtime gates. Nothing was promoted to main. S13 then ran
against that exact commit and source/config map.

The native shared-multiply/LayerNorm comparison passes exactly one qualified
test. An initial unqualified `--exact` selector ran zero tests; that attempt
is preserved and rejected as evidence. The corrected test and its source,
timestamp and output hashes were independently reviewed.

The visible in-app browser passes all ten graph and eleven embedding cases
with the backport, with `gpu_errors: []`. The unpatched build also passes all
21 cases with no GPU errors. Shared multiplication is correct; scalar and
independent-tensor multiplication pass in the same run; both LayerNorm
results differ from their input bits and match expected values. Its raw
result is byte-identical to the patched result. Thus the expected-failure
control is **not satisfied**, despite the successful computation.

Provenance was checked independently: only the `burn-cubecl` manifest patch
row and its registry source/checksum changed; all 552 package versions and
dependency edges remain. All 122 upstream files match the checksum-verified
registry archive. Patched and unpatched Wasm hashes differ. A fresh localhost
origin served the unpatched worker, generated JavaScript and Wasm with HTTP
200 after its build; the worker executes the fixture, without stored results.
The source/config/timestamp maps and served-asset hashes remained stable.
No stale-asset or source explanation was found.

Raw browser JSON, screenshots and a portable comparison manifest are under
`ports/distillery/probe/repros/burn_browser_embedding/receipts/`, named
`2026-09-29_pre4_*`. Full commands, logs, admission, registry comparison,
detector controls and restoration are in
`Code/testing/mere/receipts/2026-09-29/pre4-s13`.

Stop rule 13.8(2) now applies. Exact patched manifest/lock bytes and all
released source/config hashes were restored; both servers stopped and both
temporary browser tabs closed. Ignored generated browser assets still name
the unpatched build and must be rebuilt before a patched rerun.

**Reading, not ruled:** first compare historical unpatched pre.2 under this
same current browser. August's pre.2 receipt failed shared multiplication
and both LayerNorm cases. This new receipt does not distinguish a Burn/CubeCL
change from a browser/backend change, nor establish every guarded launcher
shape. The pending choice is to make that comparison first, retire the patch
on current evidence, or retain it as a precaution with an explicit control
amendment. No new ruling has been allocated. Extrema, remote lifecycle,
S15 closure, main promotion and downstream handoffs remain held.

### 13.27 Ruling 410: historical comparison, then conditional retirement (2026-09-29)

Mark answered the three-way question from §13.26: **"I suppose A, then B if we can"**.
Canonical ruling 410, with the question and all options, is recorded in Isometry's
`mesocosm/design_docs/2026-09-18_wing_design_plan.md`. A compares historical
unpatched pre.2 in the same browser; B retires the patch and completes the gates.
**Reading, not ruled:** retirement is authorized when the evidence supports it;
an unresolved numerical failure remains a stop, not permission to remove a guard.

The reconstructed historical dependency control reproduces the old failure in
the same current Codex IAB session where upstream pre.4 passes. Pre.2 has five
passing and five failing graph cases, all eleven embedding controls passing,
and no GPU errors. Shared multiplication fails, both LayerNorm outputs equal
their inputs bit-for-bit, and scalar/independent multiplication pass. Upstream
pre.4 passes all ten graph and eleven embedding cases with no GPU errors.
Both result auditors reject injected missing/error/wrong-case/name controls.

The current Rust/browser fixture bodies match `ac87a236` after license headers
and line-ending normalization. The historical lock and runtime/reduce patches
come from `28ed6a4f`: 570 packages, Burn/CubeCL pre.2 and wgpu 30.0.0. The August
receipt names that commit, but its committed fixture lacks the full graph
ladder it reports. This is therefore a reconstructed dependency control, not an
exact reconstruction of the August executable. Five missing crates.io archives
were fetched under ruling 378 and verified against the unchanged lock; no Git
source was fetched. All 178 historical patch files and 120 upstream burn-cubecl
files were verified. The failed first offline preparation was preserved and
restored before the successful attempt.

The pre.4 replay uses the exact previously verified upstream build assets from
`6b52297f`, with a fresh origin and successful worker/JS/Wasm HTTP requests.
Its raw result is byte-identical to the prior pre.4 run. Both runs use the same
IAB session; pre.4's 552-package row has wgpu 30.0.1. This establishes a tested
dependency-stack difference, not which upstream component fixed it.

Original source/config bytes were restored to clean `34d29269`; both temporary
servers and tabs were closed. Portable JSON, screenshots and the comparison
manifest are under the embedding fixture's `receipts/2026-09-29_*` paths.
External build, source, archive, server and restoration receipts are retained in
`Code/testing/mere/receipts/2026-09-29/pre4-s13/pre2-comparison-resolved`.

Before selector retirement, the embedding fixture now contains nine native
public-API controls for subtraction, atan2 and XOR, each with broadcast aliases,
same-shape aliases and separate inputs. They use explicit unfused CubeBackend
operations, real handles, fresh output and retained-input checks. The six-field
predicate verifies fixture construction; it does not assert upstream has our
guard. Only two already-selected direct dev-dependency edges were added to the
fixture lock. These controls are prepared, not yet accepted at this checkpoint.
All four selectors remain patched. Remaining S13, S15 and main promotion stay
gated; the old sealed release baseline is preserved unchanged.

### 13.28 Burn-CubeCL selector retirement under ruling 410 (2026-09-29)

The nine native public-API controls from §13.27 passed on both the patched and
pristine upstream pre.4 rows: 9 passed, 0 failed, 0 ignored each. All printed
numeric outputs are identical across rows. They cover subtraction, atan2 and
XOR for broadcast aliases, same-shape aliases and separate allocations, while
checking real-handle construction, fresh output and retained-input preservation.
Each receipt audit independently recomputes the outputs and rejects empty,
zero-test, wrong-name and corrupted-number controls. Source/config/mtime maps
remain stable during both commands; distinct test executables are recorded.
All 122 registry source files match the checksum-verified archive before and
after execution. Independent review accepts the results and exact restoration.

Ruling 410's conditional retirement now removes four selectors: root, browser
probe, remote fixture and embedding fixture. Four offline `--locked` metadata
gates select the exact upstream `burn-cubecl 0.22.0-pre.4` registry crate.
Each lock changes only that package's source/checksum; every existing version
and dependency edge remains. Root has 1,660 full package identities, including
four legitimate same-name/version pairs from distinct sources. They are kept.
Probe, remote and embedding locks retain 585, 982 and 552 entries respectively.

An earlier automatic-resolution attempt tried to deduplicate `spin 0.10.1`
into 0.12.3 through Pliron's broad requirement. It was rejected and restored.
The accepted change edits only the exact source/checksum and validates the
unchanged graph with `--locked`; no such unrelated dependency update was taken.

The vendored burn-cubecl source, standalone lock, licenses and historical tests
remain as provenance. Its former six-predicate guard is no longer selected by
these consumers. Ruling 377's service-identity rejection receipt remains valid
for that archived implementation; it is not claimed as upstream behavior.
The new test predicate only verifies fixture construction. Runtime persistence,
reduction and remote lifecycle patches retain their independent selectors.

Portable raw native results and the comparison manifest are in the embedding
fixture's `receipts/2026-09-29_launcher_*` files. External gates, source maps and
reviews remain in `Code/testing/mere/receipts/2026-09-29/pre4-s13` under
`launcher-retirement` and `selector-retirement`. This is a migration-branch
retirement, not main promotion. Affected-consumer runtime/build checks must now
use the upstream source. Remaining S13 extrema and plain two-peer lifecycle
receipts, S15 final documentation and S16 integration remain open.


### 13.29 Allocator stop and bounded diagnosis (2026-09-30)

The post-retirement source `124fc42bb4809dd27f9029d91e966915c97d5fd3`
passed the recorded numerical, affected matrix and build checks, including
remote release, extrema Wasm and bindgen. Build results do not stand for a
headed extrema run. The plain two-peer lifecycle exited 1 after 17.922 seconds,
without timing out: first reclaim left ten active allocations / 5,323,776 active
bytes against zero after 400 cleanup polls. Source/configuration, executable
and six model hashes stayed unchanged. Empty stdout means the strict prior
numerical ceiling and fresh-session recovery were not established. The
41,943,040 reserved bytes are cache accounting, not physical VRAM.
Stop rule 13.8(4) applies.

**Ruling 411:** Mark answered **"A!"** to bounded diagnosis with zero baseline
preserved and ownership or patch-design changes returned as forks. The exact
question/options/answer are in Isometry's wing design record, ruling 411,
published at `052ee05`. **Reading, not ruled:** acceptance and integration
remain held because the failed condition has not been repaired and reverified.

A fixture-only diagnostic preserved the original gate and error. Its control
reproduced ten allocations / 5,323,776 active bytes. Four additional samples
without explicit sync retained those amounts. Existing `client.sync().await`
returned `Ok` in 1.9222 ms; immediate and 100 ms later samples showed zero active
allocations/bytes. Subsequent cleanup reduced reserved bytes from 41,943,040 to
zero. Both processes deliberately exited 1, without timeout. Exact source
bytes/timestamps and clean status were restored. Independent review verified
source, executable/model/log hashes and rejected nine corrupted inputs.

**Reading, not ruled:** the observations support completion callbacks retaining
buffers after the worker's earlier wait. All-stream cleanup can submit work
after current-stream synchronization, while native WGPU polling needs an active
polling owner. The test does not identify every retained handle or rule out all
detached readback/transfer lifetime defects. The old pre.2 receipt did not
measure allocator counters, so this does not establish a pre.2 regression.

The pending patch-design fork recommends fixing the existing burn-remote close
path to await cleanup completion and propagate failures honestly. Alternatives
are a broader CubeCL completion-polling change or parking the migration. No
repair is selected by this annotation. A repair must retain zero-baseline and
strict numerical/recovery gates, preserve a second live lease's identity and
tensor values, and reject an injected synchronization failure. A device-wide
wait may wait on other queued work; isolated scheduling is not established.

See [the diagnosis receipt](../testing/2026-09-30_pre4_allocator_diagnosis.md)
for measurements and raw logs. Sealed external evidence remains under
`Code/testing/mere/receipts/2026-09-29/pre4-s13/allocator-diagnosis`.
The decoder and remote-native-reference verifier candidates and three main
reconciliation lock candidates remain unapplied. Browser extrema, S15 closure,
S16 promotion and downstream repins remain held. The existing migration worktree
is retained for this gate. The shared Mere target holds a diagnostic executable;
rebuild it before using it for acceptance. No new target or Cargo home was made.


### 13.30 Main reconciliation and the P5 gates on pre.4 (2026-10-02 to 03)

**Authority.** Mere's physics catalog plan records, in P5's findings, the
question whether conatus's caret `cubecl = "0.11.0-pre.2"` / Burn
`0.22.0-pre.2` should be pinned exactly. Mark replied "Wait. Those should be
bumped, no?", then chose "Finish pre.4 now, no interim pin" (main
`a924f380`). This lane takes pre.4 to its gates on this branch. Ruling 411
still governs the allocator gate, and its repair fork remains unanswered. No
merge to main, push or downstream repin follows from this entry.

**Merges.** `bcb57356` merges main `a924f380` (162 commits: Genet
`bd3e8861`, P5's conatus binning/exclusion kernels, seiche `gpu`, pictograph
`gpu`, mere and graphshell `canvas-gpu`, Density, pairing). Three conflicts
keep both sides: `.gitattributes`, seiche's manifest (main's conatus/wgpu
`gpu` rows with the branch's `=0.22.0-pre.4` pins) and conatus `chunk.rs`
(main's `ResidentClient` fields with the pre.4 client constructor). Weave
reordered entities in conatus `resident.rs` and `chunk.rs`: it moved `pub
mod binning;` and `from_wgpu`, dropped `chunk.rs`'s `std` imports and
misplaced its header. The first conatus check failed on that header.
Both files are rebuilt from main's blobs plus the branch's recorded pre.4
replacements. The root and conatus manifests and `DOC_README.md` equal a
plain three-way merge byte for byte. The root lock is the exact three-way
union of package identities, with no block changed on both sides.

`a147e6ad` gives main's new `binning/mod.rs` and `exclusion.rs` the same
mechanical renames as 13.4 (`Client`, launchers without the runtime
parameter): 9 and 19 lines. `cebcf94d` restates only the pre.4
persistence-off `cubecl-runtime` row in graphshell-web. Main's P5c lane had
restated the pre.2 `burn-cubecl` row there too; it goes, so the web graph
takes the registry `burn-cubecl` as the root does. *Reading, not ruled:*
ruling 410's retirement covers this fifth selector, which main added after
the retirement for the old pre.2 reason. The same commit moves the remote
fixture's lock with Genet `bd3e8861` through its path dependencies. That is
five Genet source substitutions, `genet-text` added, and two edges from
main's manifests: 982 to 983 packages, locked metadata passing.

At the coordinator's request, `b73695da` merges main `f62581c7`: Retinue
0.2.0 (`fa4f925`) at the root and in signalman, djinn's knot-site at
knot-editor `ea3e99e`, and two pairing-plan doc commits. Manifest and lock
equal a plain three-way merge. Locked metadata passes at the root, in
graphshell-web and in all five nested Distillery roots.

**Downloads under ruling 378.** The shared registry cache was emptied at
2026-10-02 18:00 by an outside process, and no pre.4 archive was cached.
Each fill was `cargo fetch --locked` against an unchanged lock. A recorder
listed every missing archive with the lock's checksum beforehand and hashed
each one afterwards. All 169 match: root 89 (the pre.4 Burn/CubeCL/Cubek
family plus Pliron/LLVM, awint, buildid, sysinfo 0.39.6, spin 0.12.3, ureq
3.4.2 and their needs), remote fixture 14, extrema repro 11, session fixture
55. The session fixture's 55 include its optional Turso lock entries, which
§13.18 records as lock-only for that CPU root. The recorder rejects a
planted wrong checksum. No Git fetch: every Git revision the merged locks
name, including Genet `bd3e8861`, knot-editor `ea3e99e` and Retinue
`fa4f925`, was already checked out.

**Graphs at `b73695da`.**

- **Root lock** (`114ab762…`, 1,662 packages): no pre.2. One version per
  Burn/CubeCL/Cubek crate. One wgpu (30.0.1). No Turso and no SQLite.
  iroh 1.2.0 and tokio 1.53.1 unmoved.
- **graphshell-web lock** (gitignored; exact bytes `b002ad3d…` retained in
  the receipt directory, 875 packages): resolved offline from main's
  recorded P5 lock `a915fa23` with only pre.4-forced moves. It holds the
  same pre.4 family as the root, from the registry except the path
  `cubecl-runtime`; one wgpu (30.0.0, as on main's web lock); no Turso. Its
  wasm feature tree shows no runtime `persistence`, while the same detector
  finds three in the retained §13.24 positive tree. Every other web-to-root
  version difference already existed on main. One compiled edge differs:
  Pliron 0.18.0, a normal dependency of pre.4 `cubecl-core`, takes spin
  0.12.3 in the web graph and 0.10.1 at the root.
- **ESP with no default features** has neither Burn nor tokenizers.
- `cargo_mode.py verify` passes both `--metadata-only` and full (1,535
  metadata packages).

**Gates.** Offline and locked, Rust 1.98.1, four jobs, no incremental, target
`C:/t/cargo-targets/mere/burn-pre4`. GPU logs are audited for zero
`no wgpu adapter` lines plus a positive adapter or CubeCL kernel marker; the
auditor rejects a planted skip, a missing marker and a failure.

| Gate | Result |
| --- | --- |
| conatus `resident --all-targets` check | pass |
| conatus release `--test resident --test exclusion --test resident_chunk` | 14 pass, 2 timing ignored, 0 skips |
| seiche release `gpu --test gpu_repulsion` | 3 pass, 1 timing ignored |
| seiche release `tensor-burn-wgpu` (13.21 force parity included) | 115 pass, 1 ignored |
| seiche `--lib` / `--no-default-features --lib` / `gpu --lib` | 96 / 92 / 96 (P5c: same) |
| pictograph `canvas --lib` | 270 (P5c: same) |
| pictograph `gpu --lib --test physics_device` | first attempt 272 + 1 fail; repeat 273 pass |
| mere `graph,canvas-gpu` and graphshell `canvas-gpu` checks | pass |
| cambium-genet-web-host | 8 pass (first attempt could not start rustc) |
| graphshell `web --lib` | 228 + 2 fail; main-equivalent cone, see below |
| ESP release synthetic parity + controls / real MiniLM | 4 pass, 2 ignored / 1 pass |
| Distillery four-feature check; both lease tests; Djinn trainer check | pass; 2 pass; pass (repeated after `f62581c7`) |
| graphshell-web wasm, `canvas-gpu`, dev, bindgen 0.2.127 | pass; bundle `db4ec90f…`, 110.8 MB (P5's pre.2 bundle: 85.2 MB) |

Numen's `field-burn-wgpu` cone is identity-unchanged across both merges, so
§13.25's 77-test receipt carries. The others were rerun because a selected
cone changed. The cone checker flags seiche's changed `gpu` cone as its
positive control. Every migration cone is identity-unchanged across the
`f62581c7` merge (djinn's control changes), so receipts taken at `cebcf94d`
carry to `b73695da`.

Three first attempts died on Windows `STATUS_DLL_INIT_FAILED` (0xC0000142)
starting `rustc` or `git` while more than twenty other sessions' compilers
held the CPU at 100%. ESP synthetic, cambium-genet-web-host and the
post-merge verify each passed on repeat. These are environmental, not
negative controls.

**Numbers against pre.2.** The conatus pairs pass prints the same relative
errors as P5's pre.2 receipt at 1k, 10k and 50k (worst 2.89e-6, 1.16e-5,
1.82e-5). The cells pass and the sign-flipped positive controls vary run to
run on one pre.4 binary: four runs gave cells worst 1.52e-5 to 2.20e-5 at
1k and 6.45e-5 to 1.05e-4 at 50k, and sign-flipped overlaps 442 to 544. The
pairs statistics stayed fixed. That fits the binning's atomic scatter order.
Pre.2's single samples fall inside these ranges except the 1k cells worst
error (1.11e-5), below all four pre.4 samples. One sample neither
establishes nor excludes a shift, and the test bound is 1e-3. Seiche's
lagged lane matches pre.2's lost-device receipt exactly. Its 2,000-node
device settle reads spread 1511.1 against 1511.0. Which steps fall back to
the CPU depends on machine load (stale 4 against 0) by design.

**Unresolved.**

- **pictograph `an_offloaded_canvas_uses_the_device_set_before_or_after_offload`**
  failed once with 0 device answers ("the actor never used the device").
  That was its gated run, the binary's first execution after a fresh build.
  Seventeen further executions pass (62 to 144 answers; P5c pre.2: 73/76),
  including the repeated gate. Cause not found; not claimed green.
- **graphshell `--features web --lib`** failures are
  `carrier::p2panda_murm_grant_is_refused_before_projection_bytes`
  ("projection accept timeout"; 3 of 5 when run alone) and
  `session_notices` polling-count tests (pass alone, fail under parallel
  load). The cone holds no Burn or CubeCL. Against main, it has no new
  package identity, and its only path differences are this branch's
  manifest pins and probe files. This binary is main's code; the failures
  are main's load-sensitive network/timing tests, outside this migration.

**S13 (b), extrema, headed: passes.** Built at `b73695da` with the pinned
wasm-bindgen 0.2.122, selecting the patched `cubek-reduce` and
`cubecl-runtime` and registry `burn-cubecl`. Run in a separate headed Chrome
154 instance, own profile, on NVIDIA Lovelace with the page visible. All
four cases match their expected bits, with `gpu_errors: []`. The JSON is
value-identical to the 2026-08-22 pre.2 receipt and is kept as
`ports/distillery/probe/repros/cubek_browser_extrema/receipts/2026-10-03_pre4_chrome.json`.

**S13 (c), two-peer lifecycle: still stops, unchanged.** The plain gate at
`b73695da` exits 1 in 17.8 s without timing out. After first reclaim it
leaves 10 active allocations / 5,323,776 bytes, reserved 41,943,040, against
the zero baseline, with stdout empty. That is §13.29's failure byte for
byte, so stop rule 13.8(4) still applies. Ruling 411's fixture-only
diagnostic was rerun; the retained candidate source was swapped in and its
original bytes and mtime restored and verified afterwards. The
completion-poll row reproduces §13.29: four samples without sync unchanged,
`client.sync().await` `Ok(())` in 2.2 ms, then zero active allocations and
bytes, and cleanup takes reserved to zero. New: the same diagnostic
executable's control row retained 6 allocations / 4,729,344 bytes, not 10.
The retained amount varies run to run. *Reading, not ruled:* that fits the
completion-ordering explanation, since it depends on how many completion
callbacks ran before the sample. The zero baseline is reached only through
the explicit completion wait. Where that wait belongs is the unanswered
fork in `repair-fork-pending-question.json`: (A) the burn-remote close
path, (B) general CubeCL polling, (C) park.

**P5 web receipts, headed.** Lane copies of P5's runners: own port 8823,
own Chrome profile, this worktree's page.

- **`p5_tree_gpu_settle_2000` at the web defaults (no `gpu_*` options):
  RESULT ok.** 413 of 418 steps on the device, spread 1,075, 0 overlaps,
  energy 424,849. P5's pre.2 receipt read 413 of 418, 1,075, 0 and 425,160.
- **CPU twin: RESULT ok.** Spread 1,071, energy 440,992.8 (pre.2 440,993.3).
- **Physics stage per frame (p50):** device 54.1 ms, CPU 68.3 ms. P5's pre.2
  figures were 4.9 ms and 89.3 ms.
- **Law receipts at the defaults:** 11 of 14 RESULT ok at the runner's
  300 s limit (springs, charge, energy, orbit, kinds, flock, sync, flow,
  anneal, still, profiles), taking 58 to 255 s against P5's 9 to 26 s.
  stress passes at 444 s under a 900 s limit. add and drag posted no
  receipt within 900 s (P5: 153 and 155 s), so they have no verdict.

The CPU path is not slower, and the device lane's web cost per frame is
about eleven times pre.2's. The web defaults rest on pre.2's crossover:
ruled third round, "Web N = 9, web threshold 400". Neither cause nor fix is
established here. Both bundles are dev builds whose CubeCL crates are
unoptimized (graphshell-web's dev overrides name only rapier, parry,
nalgebra, simba and seiche).

**Same-session A/B and the cause.** A pre.2 control bundle was built from a
`git archive` export of main `a924f380` in the session scratchpad with P5's
recorded web lock `a915fa23`. All its archives were cached; bundle
`c6b52d2d…`, 85.8 MB. Same runner, port, profile and options, alternating:

| | pre.2 | pre.4 |
| --- | --- | --- |
| GPU settle physics p50 | 3.1 ms | 55.5 ms |
| GPU settle frame interval p50 | 1,244 ms | 1,852 ms |
| GPU settle wall | 196 s | 293 s |
| GPU settle bounds | 413/418, 1,075, 0 overlaps | 413/418, 1,075, 0 overlaps |
| kinds wall | 8 s | 82 s |
| Idle tree, time to ready | 1.35 s | 3.2 s |
| Idle tree, frame p50, `gpu=off` too | 12.1 ms | 557 to 612 ms |

`gpu=off` does not help pre.4 (kinds 77 s; idle frames 558 ms), so the
device lane is not the cost. A 3 s DevTools CPU profile of the idle pre.4
page finds the main thread in wasm static constructors: `__wasm_call_ctors`
412 ms, `inventory::Registry::submit` 501 ms, Pliron's `inventory`
trait-cast and `InventoryWrapper` registrations 389 and 361 ms, a
`cubecl_ir::dialect::math … __ctor`, and the
`__externref_table_alloc/dealloc.command_export` wrappers 436 ms. The pre.2
profile has none of these: 19% idle, with livery style and layout on top.
*Reading, not ruled:* pre.4 `cubecl-core` depends on Pliron, which
registers through `inventory`, and cubecl-ir adds constructors of its own.
wasm-ld then links the module command-style, and its `.command_export`
wrappers run every constructor around each JS-to-wasm call, so every
registration is redone per call. Native code runs constructors once. The
probe and repro receipts are one-shot correctness runs and do not measure
this cost. Every web consumer of pre.4 CubeCL is affected, not only
`canvas-gpu`'s device lane. The fix is a build or patch-design choice and
returns as a fork.

**Held.** The ruling 411 repair fork (S13 (c)). The pre.4 wasm
constructor cost, returned as a fork; until it is settled, pre.4 cannot be
promoted to the web. S15 closure docs, S16 integration, and the Knot and
Isometry handoffs. Evidence: `Code/testing/mere/receipts/2026-10-02/burn-pre4`
(gate JSON/logs, audits, cone checks, fetch records, web-lock bytes,
remote/, extrema/, web/). The lane target is `C:/t/cargo-targets/mere/burn-pre4`,
and its remote fixture executable is the diagnostic build; rebuild it
before any acceptance run.

### 13.31 The two held forks ruled (2026-10-03)

Both of §13.30's held forks went to Mark on 2026-10-03, after the
coordinator re-read the gate and A/B records. They are Isometry wing
rulings 508 and 509 (`f702f0a`).

**Ruling 508: the allocator repair (ruling 411's pending fork).** Question: pre.4's
remote lifecycle gate holds 10 allocations (5,323,776 bytes) after reclaim
until an explicit `client.sync()`, which takes it to zero in 2.2 ms; where
does the repair go? Options: the burn-remote close path awaits cleanup
completion and propagates failures honestly; CubeCL's general completion
polling changes for every consumer; park the migration. Mark: **"burn-remote
close path"**. *Follows:* the repair keeps the zero-baseline gate and the
strict numerical and recovery gates, preserves a second live lease's
identity and tensor values, and rejects an injected synchronization
failure.

**Ruling 509: the wasm constructors.** Question: pre.4 frames take 557 ms against
pre.2's 12.1 ms, GPU on or off, because the module re-runs its static
constructors on every JS-to-wasm call (pliron's `inventory` registrations
via `cubecl-core`, upstream, not our patches); native is unaffected, and
the physics plan's P5 web defaults (N = 9, threshold 400) were ruled on
pre.2's frame times. Options: a bounded lane makes the constructors run
once (a different wasm link model, or a bindgen-side fix), proves it with
the same A/B, then re-measures P5's web crossover; the same lane plus an
upstream issue; vendor-patch the constructor sources for wasm; park web
promotion with native on pre.4. Mark: **"Bounded fix lane"**. *Follows:* no
upstream issue is filed; pre.4 is not promoted to the web, and main is not
merged, until the constructors run once and the A/B shows it.


### 13.32 Allocator repair (2026-10-03, ruling 508)

**Authority.** Ruling 508 (Isometry `f702f0a`, §13.31) puts the repair in
burn-remote's close path. Close waits for cleanup to complete and reports
failures honestly. Acceptance needs four things: the zero-baseline
lifecycle gate, the strict numerical and recovery gates, a second live
lease that keeps its identity and tensor values, and a rejected injected
synchronization failure that fails when the repair is removed. Ruling 509's
constructor lane is separate and owns the web build; nothing here touches
it.

**Where the patch lives.** The existing vendored
`support/patches/burn-remote` clearly fits. The close path being repaired is
Mere's already-patched targeted close. The root, the probe and the remote
fixture already select this source. Its `MERE-PATCH.md` already carries the
upstream commit, licence and removal condition that §2 requires. A mark-ik
fork would give one crate a second home. An upstream PR needs communication
that no ruling authorizes. This is not a new home, so no fork was raised.

**The repair, `88fd392f`.**

- `server/worker.rs`: after dropping the session's interpreter and running
  `memory_cleanup`, the worker waits for the device (`B::sync`), then runs
  `memory_cleanup` once more. The two existing pre-drop syncs no longer only
  log: every teardown failure is kept, and the worker's completion carries
  `Result<(), String>`.
- `server/session.rs`: `finish_session` turns a teardown error into
  `SessionCompletion::Failed`. The session stays registered and is never
  acknowledged clean, so `close_session` returns the error. Distillery's
  `close_run` already propagates it, so Distillery is unchanged.
- The wait is device-wide, as the 2026-09-30 review anticipated. It may
  also wait on other sessions' queued work; per-session isolated scheduling
  is not established.
- A `cfg(test)`-only hook, `worker::teardown_fault`, fails the post-release
  wait for one session. It exists only in burn-remote's own test build, and
  the public API is unchanged. `MERE-PATCH.md` records all of this and its
  removal condition.
- Changed files are rustfmt-clean under upstream's default style. Mere's
  family `rustfmt.toml` would reformat untouched upstream code, so it was not
  applied there.

**The fixture, `cf7103d8`.** Two changes:

- The reviewed 2026-09-29 verifier candidate (patch `368c183c` on source
  `cfa40321`) is applied unchanged: non-finite values are rejected and six
  CPU verifier tests cover it. Its four rustfmt suggestions are its own and
  are kept as reviewed.
- A second-live-lease stage runs after the zero-baseline reclaim and
  recovery stages. Two more jobs run concurrently on the same device. The
  host now allows three runs, for these two plus a re-grant of the first
  job; the earlier stages post one job at a time.
  - The kept lease executes, and one device sync from the fixture settles
    its baseline. The close under test gets no fixture sync.
  - The other lease executes, then its holder authors `LeaseRevokedByOwner`.
    The host loses that lease and cancels its run, and Distillery closes its
    sessions through burn-remote's targeted close.
  - The kept lease must not be disturbed. It must stay active with its one
    session and re-execute to bit-identical output, and the allocator must
    return to its baseline.
  - A final owner reclaim must reach zero. That reclaim does not repeat the
    stop-before-fact ordering assertion, which the first reclaim already
    proves.

A first development run failed exactly there: the fixture dropped the kept
provider before the shutdown reclaim, which closed its session first. That
was a fixture-design error, fixed before acceptance; the record is
`dev2-second-lease`.

**Acceptance at clean `cf7103d8`.** Offline and locked, Rust 1.98.1, four
jobs, lane targets under `C:/t/cargo-targets/mere/burn-pre4`.

- **Two-peer gate: exit 0 in 8.2 s, no timeout.** Executable `0984596f…`,
  the six model hashes match, sources stable.
  - Active allocations and bytes are 0 immediately after both owner
    reclaims (waits of 0.004 and 0.006 ms), against §13.29's 10
    allocations. The final cleanup also takes reserved bytes to 0; that is
    recorded, not gated.
  - All five numerical blocks read native error `1.4901161193847656e-7`,
    equal to the pre.2 ceiling. Recovery is bit-identical to the first run.
    The in-flight 512-row request fails on reclaim.
- **Second live lease.**

  | Point | Allocations / bytes |
  | --- | --- |
  | Kept baseline | 101 / 90,261,504 |
  | Both leases live | 202 / 180,523,008 |
  | Immediately after the close | 101 / 90,261,504 (0.013 ms) |
  | After the final reclaim | 0 / 0 |

  The kept lease was not disturbed, is still active with one session, and
  its re-run differs by 0. The closed lease ends with 0 sessions.
- **Auditor.** `audit_remote_repair.py` extends the 2026-09-29 auditor with
  the immediate-zero and second-lease requirements. It accepts receipt
  `ccc247ba…` against the 2026-08-23 baseline `56820c09…`, and rejects all 11
  planted faults: two weaker numerical results, three kinds of retained
  allocation, a disturbed kept lease, changed kept values, a lost kept
  session, a dirty receipt, a wrong head and an empty receipt. The receipt is
  kept as
  `ports/distillery/probe/receipts/2026-10-03_pre4_remote_minilm_repaired.json`.
- **Injected failure.** The unit test
  `server::session::teardown_tests::a_failed_teardown_sync_is_reported_and_never_acknowledged_clean`
  passes: an unarmed session closes `Ok` and leaves the registry, while an
  armed one fails with the injected error, stays registered, and fails a
  later `close_session`. Mutant A (post-release wait removed) and mutant B
  (failure discarded) each fail it; the restored source rebuilds and passes.
- **Other suites.** burn-remote's full lib suite passes 29 of 29 at default
  threading (twice) and its iroh tests 4 of 4. The fixture verifier passes
  6 of 6. Distillery's four-feature all-targets check, its two lease tests,
  Djinn's trainer check and `cargo_mode.py verify` pass. The tree is clean
  afterwards.

**Qualifications.**

- With `--test-threads=1`, burn-remote's upstream
  `tests::test_to_device_local_to_remote` fails. Its `Device::default()`
  resolves to a remote at `127.0.0.1:3000` under the `remote-websocket` dev
  feature, so it passes only while another test's server listens there. It
  fails identically alone on the pre-repair source (from a `git archive` of
  `f560c3b1`); this is not a repair result.
- Restoring a source file's older mtime after a mutation defeats Cargo's
  fingerprint and reruns the stale binary. The first mutant sequence's
  restored step failed that way; it is kept under `attempt-1-stale-restore`
  and is not counted. The repeat restores bytes with a fresh mtime. The same
  trap applied to §13.30's diagnostic restoration; this lane's next fixture
  edit forced the rebuild.
- The standalone burn-remote lock is gitignored by the root `Cargo.lock`
  rule. Its bytes (`c75388de…`) were unchanged by this work.

**Downloads under ruling 378.** Ten archives for the standalone burn-remote
lock, each checked against the lock's checksum, all matching: axum 0.8.9,
axum-core 0.5.6, burn-communication 0.22.0-pre.4, js-sys 0.3.105, matchit
0.8.4, ordered-float 4.6.0, serde_path_to_error 0.1.20, thread-tree 0.3.3,
wasm-bindgen-futures 0.4.78, web-sys 0.3.105. No Git fetch.

**Gate status.** S13 (c) passes on this branch. S13's (a)/(a') were
resolved by ruling 410's selector retirement; (b), (d) and (e) passed at
§13.30, and burn-remote is in none of their cones. Still open: S15 closure documents; S16 integration, which ruling 509
also holds until the wasm constructors run once and the A/B shows it; and the
Knot and Isometry handoffs. Evidence:
`Code/testing/mere/receipts/2026-10-02/burn-pre4/repair` (gate and audit
JSON, unit and mutant logs, fault controls, fetch record, development
runs).

### 13.33 wasm constructors (2026-10-03)

**Authority.** §13.31: Mark chose **"Bounded fix lane"** for the pre.4 wasm
constructors. *Follows:* no upstream issue is filed; pre.4 is not promoted to
the web, and main is not merged, until the constructors run once and the A/B
shows it. Vendor-patching the constructor sources (Pliron, `inventory`,
cubecl-ir) was the option not taken, and this lane touched none of them.
Branch `pre4-wasm-ctors` from `f560c3b1`. Burn-remote, the remote fixture and
ruling 411's gate are the allocator lane's and are untouched. No download:
the web, pre.2 and extrema locks each list 0 missing archives. No push or
merge.

**The cause, shown.** wasm-ld treats a module as a "command", called into
once per instance, when it is not relocatable, not position-independent, and
nothing in it calls or exports `__wasm_call_ctors`. It then wraps every
export in a `.command_export` function that runs every static constructor
first. inventory 0.3.24's docs and wasm-bindgen 0.2.127's source both quote
this test. rustc's link line for graphshell-web (`--print link-args`) meets
it: `rust-lld -flavor wasm`, `--no-entry`, 5,123 `--export`s,
`--gc-sections`, and no PIC, shared or relocatable flag, crt object,
`_initialize` or `__wasm_call_ctors`.

The raw pre.4 module's `__wasm_call_ctors` calls 8,166 constructors. All are
`inventory` `__ctor`s expanded from Pliron's registration macros: 7,305 in
cubecl-ir, 640 in cubecl-wgpu, 118 in Pliron, 55 in cubecl-core and 48 in
cubecl-opt. All 5,121 function exports point at wrappers that call it, then
the real function. After wasm-bindgen, 13 wrappers remain and 12 of the 34
exports reach them. The 12 include `__wbindgen_malloc`, `__wbindgen_realloc`
and `__wbindgen_free`, and `__externref_table_alloc` and `dealloc`, which the
glue calls for every string and every JS object it hands to wasm. P5's pre.2
bundle (`c6b52d2d`) has no `__wasm_call_ctors` and no wrapper. §13.30's
reading holds, with one correction: the constructors are not only Pliron's
own. They are Pliron's macros expanded in five crates.

*Positive control* (a zero-dependency crate, rustc 1.98.1, run in Node 24):
one `.init_array` constructor that counts its own runs.

| Variant | Wrapped exports | Constructor runs |
| --- | --- | --- |
| no constructor | 0 of 2 | 0 |
| constructor, default link | 2 of 2 | 4 after 3 calls and a read; 5 on the next read |
| `-C link-arg=--export=__wasm_call_ctors` | 0 of 3 | 0; 1 if the embedder calls the export |
| an exported function calls `__wasm_call_ctors` | 0 of 3 | 1 |

With wasm-bindgen 0.2.127 in the loop, the default link wraps all 74 raw
exports, and 3 `echo` calls run the constructor 12 times. The export-only
link argument leaves 0 runs, because the generated glue never calls the
export, so every `inventory` registry would be silently empty. A start
function that calls `__wasm_call_ctors` gives 1.

**The fix.** graphshell-web's `#[wasm_bindgen(start)]`
(`ports/graphshell/src/web.rs`) now calls `__wasm_call_ctors` before any
other Rust code runs, through `run_static_constructors` (five lines). The
link line is unchanged: the reference alone flips wasm-ld's test. The fixed
module still holds the 8,166 constructors, has no wrapper, and its only
caller of `__wasm_call_ctors` is `run_static_constructors`. The page has one
entry: `loader.js` → `init()` → `__wbindgen_start`, once per load.

**The count.** `wasm_ctor_probe.py` makes a probe copy of a bundle. It
appends an exported counter global incremented at the top of
`__wasm_call_ctors`, plus one glue line that reads it. Shipped bundles are
not modified. On the bindgen control the probe matches the crate's own
counter: 11 = 11 wrapped, 1 = 1 fixed. The counter is read at ready and
again after a 5 s frame window:

| Bundle | At ready | After the window |
| --- | --- | --- |
| pre.4 before the fix (`16ab7c2e`) | 5,441 | 15,413 (repeat: 18,737) |
| pre.4 after (`6d202d1b`), GPU on | 1 | 1 |
| pre.4 after, `gpu=off` | 1 | 1 |

**The A/B.** Three arms, one machine, back to back, alternating arm by arm:
pre.2 (`c6b52d2d`, the §13.30 export of main `a924f380`), pre.4 before the
fix and pre.4 after it. The scripts are the pre.4 lane's `measure_boot.py`,
`measure_profile.py` and `run-scenario.ps1`, copied with only the Chrome
profile and ports changed. Each arm serves its own pages. Two rounds ran
(3 and 5 repetitions GPU on, 2 and 5 with `gpu=off`); the table gives the
pooled medians of the repetitions. Other sessions loaded the machine
throughout (0 to 15 of their `rustc` processes at the samples, CPU 96% at
one), so pre.2's p50 ranges
12.1 to 30.3 ms where §13.30 read 12.1. Frame intervals fall on vsync
multiples (12.1, 18.2, 24.2 and 30.3 ms).

| Arm | GPU | n | Ready (ms) | Frame p50 | p95 | Max | p50 per repetition |
| --- | --- | --- | --- | --- | --- | --- | --- |
| pre.2 | on | 8 | 1,847 | 15.2 | 30.4 | 521 | 12.1 to 30.3 |
| pre.4 before | on | 8 | 4,530 | 812 | 949 | 949 | 612 to 1,121 |
| pre.4 after | on | 8 | 2,177 | 21.2 | 36.3 | 446 | 12.1 to 30.2 |
| pre.2 | off | 7 | 1,698 | 12.2 | 24.3 | 370 | 12.1 to 18.3 |
| pre.4 before | off | 7 | 4,187 | 691 | 861 | 861 | 600 to 915 |
| pre.4 after | off | 7 | 1,882 | 12.2 | 30.2 | 442 | 12.1 to 18.1 |

Against pre.2's §13.30 figure of 12.1 ms: pre.4 after the fix reads 12.2
with `gpu=off`, and 12.1 to 30.2 GPU on, with a median 21.2 against
pre.2's 15.2 in the same rounds. Its ready time is 2,177 / 1,882 ms
against 1,847 / 1,698; the wasm is 110.7 MB against 85.8 MB, and its
fetch takes 178 to 449 ms against 129 to 333. The `gpu=off` profiles:
pre.4 before spends 2,047 to 2,144 ms of 3.5 to 3.9 s in
`__wasm_call_ctors`, `inventory` submits and the wrappers. pre.4 after has
none of them in its top 30, and the same top entries as pre.2 (livery
style and layout). Three alternating GPU-on profile pairs attribute the
GPU-on median: busy main-thread time per 3 s is 2,059 to 2,771 ms for
pre.4 after against 2,094 to 2,264 for pre.2. The same crates lead both
(core, hashbrown, genet-livery, alloc, livery, read-fonts, genet-render),
and no CubeCL, Burn, Pliron or `inventory` frame appears in either. The
only steady difference is "(program)", 205 to 237 ms against 167 to 175.

| Scenario (same rounds) | pre.2 | pre.4 before | pre.4 after |
| --- | --- | --- | --- |
| 2,000-node settle, physics p50 / p95 / max (ms) | 6.3 / 7.5 / 8.3 | 81.9 / 115.2 / 163.3 | 6.8 / 8.6 / 9.0 |
| the same, frame interval p50 and wall | 2,055 ms, 370 s | 2,628 ms, 478 s | 2,125 ms, 350 s |
| the same, bounds | 413/418, 1,075, 0 | 413/418, 1,075, 0 | 413/418, 1,075, 0 |
| kinds wall, GPU on / off | 11 / 10 s | 86 / 78 s | 11 / 11 s |

**Receipts on the fixed bundle** (`6d202d1b`, headed Chrome 154, own
profile, NVIDIA Lovelace, page visible):

- **The 14 law receipts** at the web defaults and the runner's 300 s limit:
  **all RESULT ok**. springs 22 s, charge 20, stress 18, energy 22, orbit
  22, kinds 9, flock 20, sync 10, flow 22, anneal 23, still 16, profiles 19,
  add 150, drag 181. P5's pre.2 figures were 9 to 26 s, add 153 and drag
  155. Stress, add and drag no longer time out.
- **`p5_tree_gpu_settle_2000` at the web defaults: RESULT ok.** 413 of 418
  steps on the device, spread 1,075, 0 overlaps, energy 424,902, physics
  4.6 ms a frame (P5's pre.2: 413/418, 1,075, 0, 425,160, 4.9 ms). CPU twin:
  RESULT ok, spread 1,071, energy 440,992.8, 82.9 ms.
- **S13 (b), extrema: passes.** The repro is built as committed (release,
  wasm-bindgen 0.2.122), and its module has the same 8,166 constructors,
  with all 3,648 raw exports wrapped. A probe build adds the same
  constructor-once start (uncommitted; the source was restored and verified
  clean afterwards) and has no wrapper. Both pass all four cases with
  `gpu_errors: []`. Both receipts are value-identical to each other and to
  the pre.4 lane's 10-02 receipt.

**P5's web crossover, re-measured** with the third round's method on the
fixed bundle. The physics stage in ms a frame (p50), device steps of all
steps, and the newest answer's age. The pre.2 column is P5's sweep and
2,000-node runs (bundles `89b75bb4` and `3a82eca7`).

| Nodes | CPU | N = 3 | N = 9 | pre.2: CPU / N = 3 / N = 9 |
| --- | --- | --- | --- | --- |
| 128 | 0.4 | 1.3, 1/139, age 7 | 1.6, 134/139, age 6 | 0.3 / 1.2, 8/139 / 1.3, 134/139 |
| 256 | 1.0 | 1.8, 2/139, age 6 | 1.9, 134/139, age 6 | 1.1 / 1.8, 1/139 / 3.1, 134/139 |
| 512 | 3.7 | 6.8, 1/139, age 7 | 2.6, 137/139, age 6 | 4.5 / 4.0, 15/139 / 1.5, 134/139 |
| 1,000 | 17.2 | 26.0, 0/139, age 7 | 2.6, 134/139, age 6 | 16.1 / 17.8, 1/139 / 2.1, 134/139 |
| 2,000 | 82.9 | 89.0, 0/418, age 8 | 4.6 and 5.5, 413/418, age 6 | 89.3 / 94.3, 9/418 / 6.1 and 4.9, 413/418 |

The newest answer is six steps old at every size, as on pre.2. N = 3 keeps
the device on at most 2 of 139 steps (0 of 418 at 2,000, which fails the
receipt's device count, as on pre.2). N = 9 keeps it on for 134 to 137 of
139 steps and 413 of 418. The device at N = 9 is slower than the CPU at 128
and 256 nodes and faster from 512 up, so the crossover still lies between
256 and 512. A second N = 9 sample (the copied sweep's mislabelled rows,
below) reads 1.7, 2.0, 2.2 and 3.1 ms. *Reading, not ruled:* these numbers
do not move "Web N = 9, web threshold 400", so this is not returned as a
fork. At 512 nodes the device's margin is narrower than on pre.2 (2.2 to 2.6
against 3.7 ms, where pre.2 read 1.5 against 4.5).

*Method notes.* P5's sweep ran its N = 3 mode as `gpu_threshold=0` alone.
That meant N = 3 on bundle `89b75bb4`, whose default N was then 3. The
default is now 9, so the copied sweep's "gpu3" rows ran N = 9. They are
kept as the second N = 9 sample, and the N = 3 rows were rerun with
`gpu_max_stale_steps=3`. `p5_tree_gpu_settle_2000` has asserted
`gpu-threshold == 400` since the third round, so 2,000-node runs at
`gpu_threshold=0` fail that assertion alone. The table's 2,000-node N = 3
and N = 9 rows keep threshold 400 and set N explicitly.

**Held, returned as forks.** (1) The other pre.4 web cdylibs (Distillery's
model probe and the burn browser-embedding and extrema repros) are still
command-linked. The model probe records browser timings against configured
bounds. (2) The fix's form: the start-function call against an exported
`__wasm_call_ctors` plus a post-bindgen glue edit, or a shared helper.
(3) Whether the A/B above shows it, given the GPU-on median one vsync above
pre.2's while `gpu=off` matches. Evidence:
`Code/testing/mere/receipts/2026-10-03/pre4-ctors` (`inspect/`,
`control/`, `probe/`, `web/`, `ab/`). The lane target is
`C:/t/cargo-targets/mere/pre4-ctors`.

### 13.34 The constructor lane's three forks ruled (2026-10-03)

§13.33's three forks went to Mark on 2026-10-03, after the coordinator
re-read the fix, the run counts (5,441 at ready before, 1 after, GPU on and
off) and the pooled A/B. Their Isometry wing numbers are being assigned
with the wing session and are added here when known.

**The fix's form.** Question: graphshell-web's start function calls
`__wasm_call_ctors` (one run per page, link line unchanged); three other
pre.4 web modules still run their constructors on every call, and Knot's
and Isometry's web builds will once they take pre.4. Options: one helper in
a stack crate, called first from every web module's start, with a run-once
guard and a test counting exactly one constructor run; the per-module start
call as committed; the link arg plus a post-bindgen glue edit. Mark:
**"Shared stack helper"**. *Reading, not ruled:* the coordinator's reason
for recommending it was that a second run (for example a future
wasm-bindgen that also calls the constructors) would make each `inventory`
node point at itself, so iterating a registry would never end; the
one-run test is the guard against that.

**The other pre.4 web modules.** Question: Distillery's model probe (which
records browser timings against bounds) and two minimal repros (burn
browser embedding; extrema, 3,648 wrapped exports) are still
command-linked; S13(b) extrema passes either way. Options: the probe takes
the fix and the repros stay minimal; all three; none. Mark: **"Probe yes,
repros no"**.

**Whether the A/B shows it.** Question: pooled medians on a busy machine
(other sessions up to 96% CPU): GPU off, fixed pre.4 12.2 ms against pre.2
12.2 ms; GPU on, 21.2 ms against 15.2 ms, one vsync above, ranges
overlapping (12.1 to 30.2 against 12.1 to 30.3), the profiles showing no
pre.4 code; the 2,000-node settle 6.8 against 6.3 ms; P5's crossover still
between 256 and 512 nodes. Options: accept that ruling 509's condition is
met for graphshell-web; rerun the GPU-on boot A/B on a quieter machine
before promotion. Mark: **"Rerun GPU-on quieter"**. *Follows:* promotion
(S16) waits on a GPU-on A/B taken with the machine quiet. *Reading, not
ruled:* "quiet" is shown by recording CPU load beside each repetition; the
GPU-off result and the crossover stand.
