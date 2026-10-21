# Bounded binding and replay qualification

Two diagnostic examples share their implementation between a native executable
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
- A changed `dt` at tick 7 must change physical state at tick 7. Removing the
  disclosed relationship must change the occurrence position trace. Named
  channels and an incompatible arrangement must refuse with the expected path.
  Unit coverage also checks that refusal leaves an admitted live Canvas record,
  choice and role table intact.

Build the examples with the repository toolchain and lockfile. Keep debug
information and incremental compilation off. Set `CARGO_TARGET_DIR` to a
suitable shared cache if desired; the commands below use the default `target`.
The installed `wasm-bindgen` CLI must match the locked Rust package version.

```sh
export CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=4
cargo build --locked -p seiche --no-default-features --features serde --example binding_replay
cargo build --locked -p seiche --no-default-features --features serde --example binding_replay_wasm --target wasm32-unknown-unknown
cargo build --locked -p pictograph --no-default-features --features canvas --example occurrence_replay
cargo build --locked -p pictograph --no-default-features --features canvas --example occurrence_replay_wasm --target wasm32-unknown-unknown
python3 support/seiche-replay/run.py --target-dir target --output /tmp/seiche-replay
```

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
