# Fresh Schedule replay — 2026-10-23

The two-stage direct Canvas fixture runs Charge for 8 fixed frames, captures as Anchored, then runs Springs for 16 fixed frames and captures as Pinned. Four stable UUIDs, original positions, a fixed seed, and item role overrides are retained. Its exact authored DynamicsSpec is read back immediately after binding, before any frame runs; save/reopen reconstructs a fresh Canvas from that recipe and the original graph/layout. No running physics state is transferred.

Two fresh native processes and two fresh Node-Wasm processes agree on all 25 witnesses (ticks 0–24): public position bits, roles, and schedule-stage readback. Each reconstruction from the saved recipe agrees with its original trace. The first transition is tick 8, completion tick 24. No new runtime API or persisted grammar changed.

Executed controls in both runtimes:

- Changing only the first stop from 8 to 9 frames shifts the transition from tick 8 to tick 9. The unchanged second 16-frame stage remains active at tick 24.
- Changing only the first capture to Pinned changes roles at tick 8 and subsequent position bits at tick 9.
- An unsupported `groups.bridges` partition nested in the first stage reaches Canvas binding and refuses at `root.stages[0].node.partition`.

The native and Wasm builds passed, as did all 10 focused native Schedule library tests. Rust 1.99.1, debug information off, incremental compilation off, four jobs. Runtime: macOS 15.8.1 x86_64, Node v26.12.0 x64, wasm-bindgen 0.2.129. The native build initially waited for another active Cargo cache user; that process was preserved.

`replay-results.json` binds 311 source files, inputs, native/Wasm binaries, glue, runtime versions, comparisons, and controls. All source hashes were rechecked after execution. `traces.tar.gz` retains fixture input and each runtime's output/refusal stderr. Control inputs are deterministically derived by the source-pinned runner. `gates.json` and compressed logs retain exact commands and exits. Full local source archive and evidence: `/Users/markik/Code/output/mere-next-crop-implementation/schedule/matrix`.

Reproduce with `support/seiche-replay/run.py --case schedule`, using the final Cargo target directory. Parent implementation commit: 80027e687.

Limits: this qualifies fresh-start recipe replay on the recorded native host and Node-Wasm runtime. It does not qualify checkpoint restoration, browser restart/UI durability, velocities/clocks, cross-OS equality, raw-term execution, or render dependency narrowing.
