# Bounded Seiche binding/replay qualification

Qualified against Mere base `a1f860d03210f0434a046aa46ee6351b24980feb` plus
the source bytes identified in `source-manifest.json.gz`. All Cargo gates used
Rust 1.99.1, locked offline resolution, debug information off, incremental
compilation off, and four jobs.

The native tests passed: 219 Seiche tests (10 ignored), six occurrence-binding
tests, and three channel-registry tests. The new halt/resume regression compares
each resumed position/velocity bit witness and field time against the matching
uninterrupted tick, with a one-tick-short control. The live Canvas refusal check
preserves the running record, choice, role table, positions, tick count and clock.

Both replay examples built for native x86_64 macOS and
`wasm32-unknown-unknown`, and both Wasm libraries executed in Node v26.12.0.
For each fixture, two fresh native processes and two fresh Node Wasm processes
produced identical labelled traces, including native/Wasm comparison:

| Fixture | Accepted ticks | Witnesses | Observations |
| --- | ---: | ---: | --- |
| Seeded Anneal and animated FlowAdvect | 600 | 600 | Position/velocity bits, field clock, sample time and applied displacement |
| Admitted occurrence/Canvas binding | 12 | 13 | Initial state plus each tick; occurrence roles, exact source identities and public position bits |

The physical controls diverged at the same point in native and Wasm: the changed
`dt` at tick 7 first changed `position:amber/x-bits` at tick 7; removing the
disclosed spring relationship first changed `position-bits:b` at tick 1.
Unsupported diagnostic specs, occurrence channels and the incompatible
arrangement refused in both runtimes. The occurrence adapter's role witness
honors a spec default and occurrence overrides; graph-source item-role inputs
remain outside this diagnostic fixture adapter.

`replay-results.json` records comparisons, controls, runtime versions and binary
hashes, including the transformed Wasm bytes actually executed.
`runtime-traces.tar.gz` retains the exact serialized inputs, outputs and refusal
stderr. `software-gates.json` and compressed logs retain the build/test commands
and outcomes. `qualification.json` identifies the complete local source archive,
preservation receipt and qualification limits. Full local artifacts are under
`/Users/markik/Code/output/mere-binding-replay-qualification-20261020`.
The primary checkout's HEAD, status and tracked dirty-file hashes were preserved.

These receipts establish fresh-start replay for these fixtures on macOS 15.8.1
x86_64 and Node's Wasm runtime. They do not establish checkpoint restoration,
cross-OS bit equality, browser UI behavior, complete actor replay or consumer
adoption. Raw-term execution, checkpoint design, world-RNG changes and render
dependency narrowing remain separate. Reproduction commands are in
[`support/seiche-replay/README.md`](../../seiche-replay/README.md).
