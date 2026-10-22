# Direct Canvas grouped replay — 2026-10-22

The flat Grouped fixture uses four stable nodes and two components resolved by the public Canvas channel API. It preserves an authored outer weight of 16.0 and nondefault roles. Two fresh native processes and two fresh Node-Wasm processes agree on all 25 witnesses (ticks 0–24); rebuilding from the exact saved recipe and original graph/layout agrees within each run.

Changing only the partition or inner law changes position bits at tick 1 in both runtimes. Unsupported `groups.bridges` refuses at `root.partition`; outer weights `1e300` and `1e-300` refuse at `root.outer.weight` in both runtimes. These also qualify the numeric admission boundary in compiled Wasm.

Witnesses cover actual groups-channel membership, role labels and public position bits. They do not expose velocities, clocks, checkpoint state, or qualify v4 persistence, browser UI, cross-OS replay or arbitrary Grouped compositions. The occurrence adapter retains its existing Grouped refusal. No production runtime or schema changed in this pass.

`replay-results.json` retains runtime versions, binary/input/source hashes and controls; `traces.tar.gz` retains outputs. `gates.json` and compressed logs record Rust 1.99.1 builds with debug0, incremental0 and four jobs. Full local evidence and source archive: `/Users/markik/Code/output/mere-seiche-three-passes-20261022/grouped-replay/matrix`. Reproduce with `support/seiche-replay/run.py --case grouped` using the final Cargo target directory.
