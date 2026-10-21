# G4b2: bounded composition authoring and scene reopen

Qualified on 2026-10-21 from Mere base
`c66620cae016a856bd8cdae7dca189293bf295b6`, on branch
`codex/g4b2-composition-authoring-20261021`. This pass resumed the bounded G4b2 authoring slice after the
binding/replay qualification recorded in `../seiche-binding-replay-20261020/`.

## Result

Graphshell's current tree Inspector can author one two-law force Mix with
explicit shares, using the shared `physics_controls::TwoLawMix` action. Canvas's
existing `set_dynamics_spec` remains the admission/binding owner. The action
replaces the root while retaining channels, seed, bars, target, realization and
root overlays; there is no additional persistence format or schema bump.

Shares are force weights and need not sum to one. Nonfinite values, overflow and
nonzero underflow during runtime weight conversion refuse before mutation. The
existing grammar supplies the remaining path-bearing refusals, including a
kinematic law inside a force Mix. Source-only edits through Apply physics retain
the composition. Use single law explicitly replaces it, including when the
chosen law equals the Mix's first law.

The panel reads the canonical spec when reopening the supported two-plain-law
shape. Other authored shapes receive an explicit notice that the draft fields
are not readback. A Mix is shown as a custom profile. Explicit accessible names
on the share inputs were required by the actual headed test. A successful scene
reopen replaces the prior physics action status so that a restored Mix does not
retain the earlier single-law action message.

## Evidence

| Gate | Result |
| --- | --- |
| Pictograph Canvas library, current debug-off test binary | 581 passed, 18 ignored |
| Graphshell Web library, including actual v4 facet capture/restore test | 325 passed, 6 ignored |
| Shared physics-controls subset, included in the Canvas total | 9 passed |
| Native and Wasm replay examples | Built with locked/offline dependencies |
| Three-fixture native/Node-Wasm replay matrix | All process and runtime comparisons matched |
| Current Graphshell product/WebGPU Wasm bundle | Built successfully; exact package hashes retained |
| Headed compose/apply/save/refuse/replace/reopen/re-save | Passed on the current bundle |
| Deliberately wrong composition assertion after Apply mix | Failed at exactly the expected assertion |

The new production fixture uses Springs at 0.25 and Charge at 0.75 for 24 ticks,
with repeated source references, distinct occurrences/roles and a disclosed
relationship edge. The serialized DynamicsSlot is reopened and rebound from the
same initial graph and tick transcript. Fresh and reopened trajectories match in
each process. Changing either shares or a constituent law changes positions at
tick 1 in both native and Wasm processes. Invalid kinematic Mixes refuse at
`root.parts[1]`. Existing 600-tick binding and 12-tick occurrence fixtures remain
part of the matrix.

The headed scenario reads the actual saved v4 dynamics facet rather than treating
form labels as persistence proof. It checks that a source edit retains the root,
an invalid Anneal draft changes neither the live root nor the saved spec, an
explicit single-law replacement differs from the saved spec, and Reopen scene
restores the canonical spec, both authored shares, source choice and running
state. Re-saving preserves the authored spec. The final save operation reports
`saved`. This is a scene reopen inside the running page; it does not claim a
browser restart durability test. Bounded waits report continued activity because
physics was deliberately running; all state assertions and the operation result
passed.

`p4_tree_composition_control.scn` intentionally expects `none` after Apply mix.
Its only failure is `physics-composition ... got 'mix'`, confirming that the
headed runner exercised the action and detects a false composition assertion.
The result is retained as an expected negative control, not a product failure.

## Receipts and reproduction

- `qualification.json`: scope, settings, limits and primary preservation result.
- `software-gates.json` and `*.log.gz`: exact commands, exit results and logs.
- `replay-results.json` and `runtime-traces.tar.gz`: fixture inputs, witnesses,
  runtime hashes and controls.
- `headed-results.json.gz`: positive result and expected negative result from
  the current headed run.
- `browser-package.json`: raw Wasm and generated package hashes. The large
  package remains in the local evidence directory rather than this repository.
- `source-manifest.json.gz`: hashes of the relevant supplier and host sources,
  new fixtures/scenarios and separate workspace manifests/locks.
- `composition_reopened.png` and `composition_refused.png`: actual app captures.

Builds used Rust 1.99.1 on `x86_64-apple-darwin`, plus
`wasm32-unknown-unknown`, with `CARGO_PROFILE_DEV_DEBUG=0`,
`CARGO_PROFILE_TEST_DEBUG=0`, `CARGO_INCREMENTAL=0`, and four jobs. Cargo gates
used `--locked --offline`. The full Canvas run reused the exact binary built by
the focused Canvas gate; its build receipt is identified in `software-gates.json`.

Run the runtime matrix after building the examples with:

```sh
python3 support/seiche-replay/run.py --target-dir /path/to/target --output /path/to/results
```

For the browser gate, generate a Web package from the current standalone
`ports/graphshell/web` build, serve it with
`ports/graphshell/web/scenario_qualification_server.py --package /path/to/package
--output /path/to/results --port PORT`, and open
`tree.html?app=local&scenario=scenarios/p4_tree_composition_save_reopen.scn&sink=/scenario-receipt`.
Run `p4_tree_composition_control.scn` separately and expect its one disclosed
failure. Use a fresh local origin for an uncontaminated initial graph.

Full local evidence, including resolved diagnostic attempts, the generated
package and before/after preservation snapshots, is under
`/Users/markik/Code/output/mere-g4b2-composition-20261021`.
The dirty primary checkout's HEAD, porcelain status and tracked dirty-file
hashes match the initial snapshot. No primary files were staged, reset or
stashed, and no supplier pins or lockfiles changed in this pass.

## Qualification boundary

These are bounded fresh-start recipe replays on one native host and Node Wasm,
plus one current macOS browser/WebGPU authoring scenario. They do not qualify
evolving force/RNG checkpoints, cross-OS math, native UI, actor transcripts,
standalone Knot or downstream consumer repins. Raw-term execution, checkpoint
design, render dependency narrowing and a recursive/group/schedule editor remain
separate passes.
