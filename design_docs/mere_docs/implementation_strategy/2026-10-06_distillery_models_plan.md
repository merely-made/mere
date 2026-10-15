# Distillery models plan: the consolidated model, inference and mesh-host follow-ons

**Date:** 2026-10-06
**Status (2026-10-14):** M1 open, two of its three receipts have a stable
result (§3); M2 to M5 not started. Founded under ruling S63 of
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
[Eidetic development plan](../../eidetic_docs/implementation_strategy/2026-10-11_eidetic_development_plan.md#p4-build-recall-and-model-work-over-named-evidence)
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
  - *Burn Fusion remote panic: not yet reproduced or cleared on stable.* The
    prerelease panic (fused-matmul autotune/ordering, [lease-bound remote
    sessions](../technical_architecture/2026-08-10_lease_bound_remote_sessions.md))
    has no stable run, because the stable receipt above is plain WGPU only.
    Running the same fixture with the Fusion profile closes this item.
  - *`esp.train.peft-lora/v1` trainer mesh job: stable result recorded
    (next entry).* §13.47 records Distillery's 26 library tests passing with
    `remote,trainer-gpu,trainer-autodiff`, but the mesh-job receipt is the
    integration test `ports/distillery/tests/trainer.rs`
    (`each_trainer_arm_publishes_artifacts_and_a_strict_improvement_receipt`),
    which that list does not name. It uses synthetic weights and a built-in
    tokenizer, so the CPU arms need no GPU or model download.
  - *Browser ceiling D2a to D2c: no stable result.* §13.47 records the full
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
