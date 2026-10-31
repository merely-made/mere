# Distillery models plan: the consolidated model, inference and mesh-host follow-ons

**Date:** 2026-10-06
**Status (2026-10-16):** M1's three receipts all have stable results and
the Fusion panic is gone (§3); the remote-autotune reclaim failure M1
surfaced needs an owner. M2 is qualified through resident inference I5; M3 to M5 remain open. Founded under ruling S63 of
the [stack seams plan](2026-10-04_stack_seams_plan.md), which consolidated the
open items of four plans archived on 2026-10-06 (ruling S58: "These four need
to be consolidated now that 0.22 is out"): the
[distillery v0 plan](../../archive_docs/2026-10-06_completed_plans/2026-08-12_distillery_v0_plan.md),
the [inference provider plan](../../archive_docs/2026-10-06_completed_plans/2026-07-05_inference_provider_plan.md),
the [browser model ceiling probe](../../archive_docs/2026-10-06_completed_plans/2026-08-09_browser_model_ceiling_probe_plan.md)
and the [mesh host lanes plan](../../archive_docs/2026-10-06_completed_plans/2026-08-09_mesh_host_lanes_plan.md).
Stable Burn 0.22.0, CubeCL 0.11.0 and Cubek 0.3.0 reached main on
2026-10-06 (burn 0.22 migration plan, §13.47), so the prerelease gate these
plans waited on has lifted.

**Owns:** what remains of Mere's local model lane after those four plans:
the inference surface's product half, Distillery's deferred operational
pieces, the browser model ceiling's unmeasured upper rows, and the mesh
host's deferred execution guarantees. It owns no Burn migration (the
[burn 0.22 migration plan](2026-08-09_burn_0_22_migration_plan.md) does) and
no training research beyond what a phase below names.

**Coordination (2026-10-11):** the
[Eidetic development plan](https://github.com/merely-made/eidetic/blob/main/design_docs/eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md#p4-build-recall-and-model-work-over-named-evidence)
owns retained encounter/capture evidence, embedding compatibility and durable
model-run records. Its P4 supplies the evidence contract for M2/M3 here; this
plan keeps the inference host, work surface and supervised execution. Choose one
forcing host and share its records rather than opening a second inference-host
implementation. This amendment is planning only and renews no execution receipt.

## 1. Findings carried in

Each item names the archived plan it comes from; that plan's text is the
evidence and history.

- **Stable Burn.** §13.47 records ESP, Conatus and Distillery
  (`remote,trainer-gpu,trainer-autodiff`) compiling and passing on 0.22.0.
  The mesh host lanes plan's last gate was this closure; whether its
  receipts (the lease-bound remote MiniLM row, the trainer mesh job) have
  been re-run on stable is not recorded there. (mesh host lanes, distillery
  v0)
- **Inference has no host.** P0 to P4 landed in `esp::infer`; the meerkat
  `>ask` host retired 2026-07-18, and nothing in Mere or Turnstone calls
  `spawn_inference_actor`. The 2026-07-06 follow-ups, a per-model prompt
  template and an answer card, lost their host with it. (inference provider)
- **Distillery's deferrals.** Model manifest browsing, a streaming console,
  portable remote checkpoints, tolerant verification comparators, a real
  stacked-adapter row, and multi-device delivery (`TransportCourier::for_mesh`
  is proven but every `HostConfig` keeps `NoCourier`). (distillery v0 §8,
  §10)
- **The browser ceiling's open rows.** D2a to D2c's configured rows passed
  on 2026-08-22; larger, consumer-gated decoder and embedding rows (the upper
  model boundaries) are unmeasured, physical GPU-allocation release is
  unobservable through current browser APIs, and the persistent-storage
  posture is open. (browser model ceiling probe)
- **The mesh host's deferred guarantees.** Tolerant verification comparators
  (`registry::verify_output` returns `NotCheckable` for non-exact outputs),
  portable checkpoints (resuming elsewhere needs the blob lane to carry the
  checkpoint), reliability and reputation accounting (its kith lane retired
  2026-09-02, so it has no home), and two 2026-08-23 sidequests: physical
  GPU-allocation telemetry and the Burn Fusion remote MiniLM panic. (mesh
  host lanes §7)

Tolerant comparators and portable checkpoints appear in both Distillery's and
the mesh host's deferrals; they are one item each here.

## 2. Phases

Each phase stops at its done-conditions; none has a date.

### M1. Stable receipts

Re-run the receipts the archived plans recorded on the prerelease rows, on
stable 0.22.0: Distillery's lease-bound remote MiniLM row, the
`esp.train.peft-lora/v1` trainer mesh job, and the browser ceiling's D2a to
D2c rows.

Done when: each named receipt has a stable-row result recorded here with its
commit, or a recorded reason it cannot run; the Burn Fusion remote MiniLM
panic is either reproduced on stable (and owned by a phase below) or recorded
as gone.

### M2. A host for inference

Choose the host that consumes `esp::infer` (a Turnstone surface, a Graphshell
surface, or a djinn resident service) and wire `spawn_inference_actor` there,
with the per-model prompt template and an answer-card surface.

Done when: one host sends a prompt through the actor and renders the answer,
with cancellation, headed; the prompt template is per-model data, not code.

**2026-10-15:** Mark chose djinn's resident as the host, with recall over
retained material as the forcing consumer. The
[resident inference and recall plan](2026-10-15_resident_inference_and_recall_plan.md)
carries M2; its I5 closes this phase. **2026-10-16: closed.** The native
Turnstone pane asks through Djinn's admitted route against retained text and
the pinned GPU model, streams progress, pauses/resumes, cancels another ask,
and reopens the answer and exact source after both processes restart with the
origin offline. The I5 receipt in that plan records the source cuts, binary
hashes and six native frames. Djinn is test-owned in this gate; its installed
binary and physical second-device qualification stay separate.

### M3. Distillery operations

Model manifest browsing and the streaming console as a Distillery surface;
multi-device delivery by adopting `TransportCourier::for_mesh` where a
`HostConfig` spans devices.

Done when: a manifest can be browsed and a run's output streamed in the
surface; a two-device run delivers its inputs through the courier, receipted.

### M4. Execution guarantees

Tolerant verification comparators, measured on the first non-exact remote
resource; portable checkpoints carried by the blob lane; physical
GPU-allocation telemetry; a stacked-adapter row.

Done when: a non-exact resource verifies with a measured tolerance and its
measurement recorded; a checkpoint written on one device resumes on another;
the telemetry reports allocation where the platform exposes it.

### M5. The browser's upper rows

Larger decoder and embedding rows, run only when a consumer names the model
it needs; the persistent-storage posture.

Done when: a consumer-named row passes or fails with its boundary stated; the
storage posture is ruled.

Reliability and reputation accounting stays out of scope until a lane gives
standing a purpose; it is recorded in the archived plan tails backlog.

## 3. Progress

- **2026-10-06.** Founded by the S14 pass (stack seams plan, rulings S58 and
  S63) from the four archived plans' open items; nothing started.
- **2026-10-14, M1 status audit.** Read from the record, except the trainer
  row, which was run on low-end hardware (next entry).
  - *Lease-bound remote MiniLM: stable result recorded.* The burn 0.22
    migration plan's §13.47 records the two-peer real-weight receipt passing
    on stable 0.22.0 at `d346d945`:
    `ports/distillery/probe/receipts/2026-10-06_stable_remote_minilm.json`.
    It covers cancellation, fresh-session recovery, single-lease close and
    allocator release. Its `backend_profile` is `plain` with `fusion`,
    `autotune` and `fusion_autotune` all false.
  - *Burn Fusion remote panic: gone on stable (2026-10-15 entry).* The
    prerelease panic (fused-matmul autotune/ordering, [lease-bound remote
    sessions](../technical_architecture/2026-08-10_lease_bound_remote_sessions.md))
    had no stable run, because the stable receipt above is plain WGPU only.
  - *`esp.train.peft-lora/v1` trainer mesh job: stable result recorded
    (next entry).* §13.47 records Distillery's 26 library tests passing with
    `remote,trainer-gpu,trainer-autodiff`, but the mesh-job receipt is the
    integration test `ports/distillery/tests/trainer.rs`
    (`each_trainer_arm_publishes_artifacts_and_a_strict_improvement_receipt`),
    which that list does not name. It uses synthetic weights and a built-in
    tokenizer, so the CPU arms need no GPU or model download.
  - *Browser ceiling D2a to D2c: stable result recorded (2026-10-15
    entry).* §13.47 records the full
    Distillery browser probe building for `wasm32-unknown-unknown`, and the
    headed Burn embedding/graph (21) and Cubek extrema (4) reproducer cases
    passing (`ports/distillery/probe/repros/*/receipts/2026-10-06_stable_iab.json`).
    Those are the reproducers, not the D2 rows, which were not re-run.
  - *Where the other two can run.* The Fusion profile needs a GPU-backed
    WGPU server and D2 needs headed Chromium; neither ran on the low-end
    machine below (integrated Radeon, no PowerShell or wasm tooling).
- **2026-10-14, M1 trainer row on low-end hardware.** Stable Burn 0.22.0 at
  `23fe8064`, Linux, Rust 1.98.1, 8-thread Ryzen 5 Surface laptop with
  5.8 GiB RAM and integrated Radeon (Raven), CPU only.
  `cargo test --locked -p distillery --features trainer,trainer-autodiff
  --test trainer`: 3 passed in 22 s, 71 MB peak RSS. Both arms published
  their artifacts with a strict improvement at `RankingAt{3}`:
  `esp-trainer-v0` (finite difference) baseline 0/6 vs adapter 4/6, and
  `esp-trainer-v1-autodiff` baseline 0/6 vs adapter 3/6.
  - *Running is cheap; building is the low-end limit.* The cold build is
    655 crates. With desktop applications open (about 600 MiB free) it was
    killed by the system out-of-memory killer twice, at 772 MB with two jobs
    and at 660 MB with one job compiling `noq-proto`. With about 2.2 GiB
    free it finished in 4 min 40 s with two jobs, debug info and
    incremental compilation off, peaking at 1.7 GB. One `rustc` on iroh's
    QUIC crate (`noq-proto`) did not fit in about 600 MiB on its own.
- **2026-10-15, M1 Fusion and D2 rows on O-PC.** Stable Burn 0.22.0 at
  `8757016c` plus the fixture repair below (receipts record
  `dirty: true` for it), Windows 11, Rust 1.98.1, NVIDIA RTX 4060 Laptop
  (8 GiB) beside a Radeon 780M. Receipts in `ports/distillery/probe/receipts/`
  with the `2026-10-15_stable_` prefix.
  - *Remote fixture repair.* Every remote row first failed at
    `first-lease-active` with "the server's authorizer rejected the
    credential". The fixture still bound both transports with the master
    keypair; since `49451b30` (vault lock rulings 92 and 93) a peer must
    connect as its derived mesh author key. The fixture now binds through
    `P2pandaTransport::builder_for` with `derived_keypair(MESH_AUTHOR_SALT)`,
    as Distillery's own tests do. Its `Cargo.lock` was also stale on this
    branch: genet moves to the root's `7a8c5b8c` and the unused dramatis
    `insigne` and `personae` rows drop; no Burn, iroh or p2panda row changes.
  - *Remote MiniLM matrix* (`run-remote-minilm.ps1 -Matrix`, server on
    `DiscreteGpu(0)`, wgpu's automatic graphics API): all four local rows
    pass (plain, fusion, autotune, fusion with autotune). Remote plain and
    remote fusion-only pass every stage: lease, owner reclaim of an
    in-flight 512-row request, fresh-session recovery, second live lease and
    allocator release. Fusion-only max error is `1.411e-7` against native
    and `1.639e-7` against the browser reference; recovery is bit-identical.
    `remote_minilm_fusion.json`, `remote_minilm_matrix.json`.
  - *Remote autotune: new failure, not the old panic.* Remote autotune-only
    and fusion-with-autotune pass the first remote execution and the native
    control, then fail owner reclaim: "reclaim did not close the
    lease-bound session", with the host in `AwaitingStop` for 401 steps.
    Fusion is cleared; autotune over Burn Remote does not yet honour the
    lease-bound close. Remote autotune stays out of production defaults.
  - *D2c embedding matrix* (headed Chrome 155, WebGPU on the RTX 4060,
    `browser_matrix.json`): all four rows pass, BGE Micro v2 (34.8 MB F16)
    through E5-base-v2 (438.0 MB F32). Every cold and warm output is finite
    and unit norm, repeats within and across workers, and is within
    `1.42e-7` of its reference (prerelease `1.416e-7`). Integrity reopen,
    termination at `executing` with zero late messages, and WebGPU error
    scopes pass for every row. Frame p95 stayed near 6.2 ms against the
    33.4 ms bound, with 23 over-bound intervals (prerelease 41). The largest
    was 1,357.6 ms during BGE Micro's cold first execution, the page's first
    model (prerelease largest 175.8 ms). Persistent storage was again
    denied (`best_effort`).
  - *D2c decoder row* (`browser_decoder.json`): SmolLM2-135M-Instruct
    (269.1 MB BF16) passes. All eight tokens match the reference exactly
    and cold and warm outputs are identical. Cancellation after one fragment
    is acknowledged in 206.8 ms with zero later fragments. `GPUDevice.destroy()`
    reports no errors, and a fresh worker recovers the exact output. Repeat
    runs hold 22.2 to 23.6 tokens/s (prerelease 25.5 to 26.2). First runs
    are slower than on the prerelease: first token 2,577 ms cold and 725 ms
    warm, against 364 and 279 ms. No gate covers first-run latency.

---

*Archived 2026-10-28.* Cut, with the other active plans, into a new set: what was still open, deferred or a follow-on here now lives in `mere_docs/implementation_strategy/2026-10-28_distillery_models_plan.md`, which cites this file by line. This copy is history: its status lines, receipts and claims are as they stood, not current.
