# Bounded binding and replay qualification

Diagnostic examples share their implementation between a native executable
and a wasm-bindgen library. `run.py` executes each in fresh native processes and
fresh Node Wasm processes, compares the existing `state-witness` entries, and
retains the exact input and output documents. Node transports the Rust JSON as a
string; Python reads the 64-bit witness integers without JavaScript number
rounding.

- Seiche's `binding_replay.json` declares 600 fixed ticks, a seed, stable body
  labels and registration order. Its narrow diagnostic catalog admits one
  explicit spec for seeded Anneal plus a host-bound animated FlowAdvect field.
  Witnesses include position and velocity bits, field clock, sample time and
  applied displacement. This catalog is local to the example.
- Pictograph's `occurrence_replay.json` goes through the existing projection
  compiler, occurrence binding and Canvas catalog. Two occurrences share an
  exact source reference while retaining separate roles and transient bodies.
  Each accepted tick records public position bits, source identity and role
  labels. This adapter does not expose velocities; no diagnostic accessor was
  added to production code.
- Pictograph's `grouped_replay.json` uses the public Canvas with four stable
  nodes, two components, a flat Grouped root and nondefault roles. It records
  actual groups-channel membership, role labels and public position bits for
  ticks 0 through 24, then rebuilds from the exact serialized recipe and the
  original graph/layout. Changing only the partition or inner law must change
  position bits at tick 1. An unsupported partition and outer weights that
  overflow or underflow runtime f32 must refuse at their exact paths in both
  native and Wasm. This fixture does not widen the occurrence adapter contract.
- Pictograph's `schedule_replay.json` runs a two-stage Schedule: Charge for
  8 frames, capture as anchored, then Springs for 16 frames, capture as pinned.
  It records public position bits, roles, and `physics_schedule_stage()` for
  ticks 0 through 24, then reconstructs a fresh Canvas from the exact recipe
  read back before running and the original graph/layout. The runner compares
  two fresh native processes and two fresh Node-Wasm processes. Controls move
  the first stop by one frame, change its capture role, and require Canvas to
  refuse an unsupported nested partition at `root.stages[0].node.partition`.
  This is fresh-start recipe replay; it transfers no running physics state.
- A changed `dt` at tick 7 must change physical state at tick 7. Removing the
  disclosed relationship must change the occurrence position trace. Named
  channels and an incompatible arrangement must refuse with the expected path.
  Unit coverage also checks that refusal leaves an admitted live Canvas record,
  choice and role table intact.

Build the examples with the repository toolchain and lockfile. Keep debug
information and incremental compilation off. Set `CARGO_TARGET_DIR` to a
suitable shared cache if desired; the commands below use the default `target`.
Pass the final Cargo target directory to the runner, even when a separate
`build.build-dir` holds intermediate artifacts. The installed `wasm-bindgen` CLI must match the locked Rust package version.

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4
cargo build --locked -p seiche --no-default-features --features serde --example binding_replay
cargo build --locked -p seiche --no-default-features --features serde --example binding_replay_wasm --target wasm32-unknown-unknown
cargo build --locked -p pictograph --no-default-features --features canvas --example occurrence_replay
cargo build --locked -p pictograph --no-default-features --features canvas --example occurrence_replay_wasm --target wasm32-unknown-unknown
cargo build --locked -p pictograph --no-default-features --features canvas --example grouped_replay
cargo build --locked -p pictograph --no-default-features --features canvas --example grouped_replay_wasm --target wasm32-unknown-unknown
cargo build --locked -p pictograph --no-default-features --features canvas --example schedule_replay
cargo build --locked -p pictograph --no-default-features --features canvas --example schedule_replay_wasm --target wasm32-unknown-unknown
python3 support/seiche-replay/run.py --target-dir target --output /tmp/seiche-replay
```

Use `--case schedule` or `--case grouped` (or repeat `--case` with other case names) for a focused
run. Without `--case`, the runner checks every fixture.

The runner records mismatches as qualification failures and preserves the first
differing tick and witness label. Equal traces qualify these fresh-start
fixtures on the recorded host/runtime/pins. They do not establish complete
checkpoint restoration, actor replay, browser UI behavior, consumer adoption or
cross-OS bit equality. Raw execution, checkpoint design, world-RNG changes and
render dependency narrowing remain separate.

Related focused native tests:

```sh
cargo test --locked -p seiche --features serde --lib
cargo test --locked -p pictograph --no-default-features --features canvas --lib projection_dynamics
cargo test --locked -p pictograph --no-default-features --features canvas --lib physics_registry
```
