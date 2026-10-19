# Counted endpoint topology and shared dynamics inspection

Qualified on 2026-10-19 against baseline
`eacbb82dc3e84f3c07af7d0b4047ad701fdd7629`. Compiled product, automated gates
and accepted native seed use `cfbf750df45a1975c14eaedfd4082510150598b8`.
The accepted reopen scenario is committed at
`e5437a382e0fd42335efd846545246697c3417f1`. The only intervening change opens
the saved promoted Resource before asking its disclosure snapshots; no
compiled product or dependency source changed. The exact diff is recorded in
`qualification-input-continuity.json`.

Pictograph now supplies endpoint dynamics and inspection through one eligible
relation-row reader, preserving exact assertion identity, canonical stratum
and displayed aliases. Seiche stores adjacent equal ordered endpoint pairs
as counted runs in simulation, layout view and actor handoff. Original ordered
slices are expanded transiently at the existing force and diagnostic
boundaries. There is no persistent second expanded cache. Adjacent compression
does not reorder contributions or replace repeated additions with a multiplied
force. Inspector counts disclose current inputs, not measured force.

## Automated qualification

| Gate | Result | Process record |
| --- | --- | --- |
| Seiche | 220 passed; 10 ignored | `seiche-final-2.json` |
| Pictograph | 580 passed; 19 ignored | `canvas-final-2.json` |
| Graphshell | 317 passed; 6 ignored | `graphshell-final.json` |
| Product Wasm / presentation-webgpu | passed | `wasm-final.json` |
| Matching wasm-bindgen 0.2.129 | passed | `bindgen-final.json` |
| Product-disabled presentation-webgpu viewer | passed | `viewer-final.json` |
| Port boundaries | passed | `boundaries-final.json` |

Every process record includes exact argv, working directory, committed source,
clean pre-run status, actual exit and log hash. Tests compare independently
supplied original slices with counted topology for force, velocity and
position bits over 120 ticks across spring, LinLog, Magnetic, Boids, Anneal,
Kuramoto, DegreeRepulsion and HubGravity. They also cover actor command parity,
neighbor multiplicity, query order, exact-reason visibility, canonical stratum
hides, alias expansion and explicit empty occurrence mode.

## Native browser and persistence

Native Chrome 155 ran the product Wasm through the BrowserWebGpu shared host
adapter on a loopback-only server, in a retained profile/origin. The environment
probe records the browser, Metal/WebGPU availability, initial empty database
inventory, viewport and DPR. Five seed captures and one fresh-process reopen
capture were inspected by `/root` and `/root/ordinary_path_acceptance`.
`browser-review.json` records image hashes and individual observations.

| Capture | Observed result |
| --- | --- |
| `p8_pair_topology_initial` | Pair held 3 / visible 3 / hidden 0; controller inputs 6 |
| `p8_pair_topology_exact_reason_hidden` | Pair held 3 / visible 2 / hidden 1; source facts and revision unchanged |
| `p8_promoted_source_before_withdrawal` | Restored pair frozen as a Resource; live comparison unchanged |
| `p8_promoted_copy_after_withdrawal` | Frozen assertions survive; live comparison changes after withdrawal |
| `p8_pair_topology_after_withdrawal` | Pair held 2 / visible 2 / hidden 0; independent sibling survives |
| `p8_counted_topology_fresh_reopen` | Stored graph and frozen copy reopen; pair remains held 2 / visible 2 / hidden 0 |

Both accepted scenarios report success, no page errors and no renderer gate
failures. Pair storage has two ordered runs in this fixture because alias
rows separate matching pairs in the original input order. A run count is not
a distinct-link count. Source facts and runtime graph revision match exactly
between post-withdrawal seed completion and reopened completion
(`source-continuity.json`). The isolated fixture reports no public source
revision, so this is not a public revision continuity claim.

Chrome PID 50511 completed the seed and exited. The corrected reopen ran in
PID 51673 and exited. `browser-processes.json` includes both empty process-list
observations, plus the separately retained failed reopen iteration. The
loopback server was intentionally stopped after browser exit; its observed
Ctrl-C exit is 1 (`server-process.json`).

## Recheck and preserved iterations

The full qualification folder, including the 187 MB Wasm package and frozen
site, is retained at:

```
/Users/markik/Code/testing/mere/counted-link-topology-20261019
```

From the Mere root, verify that package and the committed receipt inputs:

```
python3 support/receipts/link-model-counted-topology-20261019/verify.py /Users/markik/Code/testing/mere/counted-link-topology-20261019
```

Run the verifier from this Mere Git checkout, with both recorded commits
available. It reuses the ordinary-path receipt's existing command, hash,
scenario, PNG and source-comparison checks. It additionally checks totals,
six named image reviews, exact scenario-input hashes, the isolated input
correction against Git (including committed scenario bytes) and distinct
native processes. Generated JS/Wasm are not committed;
running the verifier with this receipt directory alone must reject the missing
package. No build or native pass is inferred from metadata alone.

`iterations/index.json` names the preserved nonaccepted attempts. The first
candidate expected one pair storage run even though source order yields two,
and put the summary below a long report. The final source corrects both. A
first reopen asked for details before opening their Resource; its result,
capture and exact input remain in `iterations/reopen-cfbf-disclosure-not-opened`.
The corrected scenario opens the Resource and passed in another fresh process.
The full earlier candidate package and failed command records remain in the
external qualification folder.

## Compatibility and remaining bounds

Existing slice-based custom force APIs, `sync_edges` and the legacy actor
command remain available. The new `PhysicsCommand::SyncEdgeTopology` enum
variant requires downstream exhaustive matches to handle it. Freshly fetched
Woodshed main `b1c9dfd05a75614919e8b7188e2346a6d5f60a43` has no direct
`PhysicsCommand` or `ForceContext` references (`woodshed-compatibility.json`).
This is source inspection, not a Woodshed build, repin or receiving-host
qualification. Woodshed, Hocket and Knot source were not changed.

The four-node native fixture qualifies the readable contribution summary,
alias/stratum distinction, visibility, withdrawal, frozen-copy survival and
reopen. The lower viewport remains black empty space; the relevant content is
unobscured. Density, broad zoom/layout behavior, frame-time improvement,
arbitrary weights, globally aggregated springs, measured per-assertion forces,
path-to-node feedback and resonance are outside this receipt. The ignored
`decay_progress` write surface and Knot checkpoint confidentiality remain
separate backlog items.

Repository documentation audits retain their existing nonzero findings;
`doc-audit-comparison.json` records the before/after comparison. These are not
reported as clean audits.
