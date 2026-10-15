# Coordinated Genet/Netrender receiving adoption, 2026-10-15

Status: qualified on implementation `16104bbd`, integrated over upstream `75a2e6ae`. All four graph audits, 336 native tests and all three wasm compilation gates pass. Initial `99ecfdee` logs retain their original tuple.

All four edited manifests select published Genet
`7ffe66208ca3059283ef8770de7d17aa8834ecc8` and Netrender
`f4f9c0d87e6a6030b61075a45118055dfe6a12df`. Direct Netrender dependencies
use the canonical `github.com/merely-made/netrender.git` URL. The temporary
Netrender Git-source patch tables are removed from the Mere workspace,
Graphshell viewer, standalone Genet web smoke and Distillery model probe.
The viewer enables the Classic opt-out through `genet-render-host`, which
now forwards it to Netrender. The existing Genet Fleece/layout compatibility
patch and required crates.io supplier patches retain their separate purpose.

## Source audit

[Integrated identities and lock hashes](integrated-identities.json) cover all four roots. The
[integrated lock movement](integrated-lock-movement.json) shows only Genet and Netrender source replacements in the workspace lock.
Standalone locks also retain the published Murm revision and Sceno version updates. The [standalone lock movement](standalone-lock-movement.json)
also records the older probe's existing `edit-history` to `backstep` workspace
rename, required Gloo timer edge and coordinated Vello provider update.
The ignored web-smoke lock is frozen as [web-smoke-resolved-lock.toml](web-smoke-resolved-lock.toml);
the other three locks are tracked beside their manifests.

## Gate results

| Gate | Result | Evidence |
|---|---|---|
| Four portable dependency graphs | One canonical Netrender family per root; Genet `7ffe66208ca` throughout | [Identities](integrated-identities.json) |
| Native shared hosts | 336 passed, zero failed, 11 ignored (four doctests) | [Raw log](resumed-integrated-native-hosts.log), [counts](integrated-native-counts.json) |
| Viewer WebGPU wasm | Passed, viewer features only | [Log](resumed-integrated-viewer-webgpu.log) |
| Viewer WebGL2 with forwarded Classic opt-out | Passed, viewer features only | [Log](resumed-integrated-viewer-webgl-opt-out.log) |
| Standalone web smoke wasm | Passed, including the default browser host/image cone | [Log](resumed-integrated-web-smoke.log) |

Each gate has an adjacent `.result.json` with the exact command and outcome.
The [integrated source basis](integrated-source-basis.json) records the Mere
implementation/base and six manifest/config hashes. The [final guards](integrated-final-guards.json)
confirm all four locks and all six source/config files stayed unchanged, with
no Rust source edits. Exact commands and environments are in the adjacent
`resumed-integrated-*.result.json` files.

The initial `99ecfdee` qualification remains in the original logs,
[identities](identities.json) and [source basis](source-basis.json). These are
historical evidence, separate from the integrated run above.
The first integrated native attempt was interrupted while waiting for the
shared artifact lock; [its result](integrated-native-hosts.result.json) records
zero tests executed. The successful rerun used a separate artifact directory
under the approved Mere target and reused the stable build cache.

Cleanup: `C:/t/cargo-targets/mere/netrender-adoption` remains owned by this
adoption gate. Its Cargo marker and lack of live process owners were verified;
automatic approval review blocked recursive removal with the reason
"blocked by policy". It holds generated artifacts only. The regular stable
Genet/Mere build caches remain reusable; no isolated Cargo home or source
worktree was created.

Publication integration: upstream `d4d2f88c` adds link-model documentation and
index changes only. Its index conflict was resolved by preserving that update
and the adoption entry. Comparison with the qualified tree confirms that
production source, manifests, configs and locks are unchanged.

## Qualification scope

The portable graph audit covers each of the four Cargo roots, requiring exactly
one source identity for each of the four renderer crates and the new Genet
revision on every resolved Genet Git package. It retains each lock hash and
selected package identities, with complete metadata-input hashes.

Consumer gates cover the native Rootstock, Mesquite and Genet winit host tests;
wasm viewer WebGPU and WebGL2/without-classic-vello configurations; and the
standalone web smoke compilation. Distillery's graph is audited; its inference
runtime is outside this supplier adoption. Browser runtime, physical input,
assistive technology and delivery-size packaging are not rerun here. Earlier
presentation/size receipts retain their original measured source tuples.

Commands run from `C:/Users/mark_/Code` with explicit manifests, published
suppliers and stable output under `C:/t/cargo-targets/mere`. This bypasses
ignored machine-local source redirects. The viewer's committed target flags
are passed explicitly for its wasm checks. Exact commands and outcomes are
retained in the gate result JSON files and byte-preserved logs.

Genet supplier qualification: `genet/design_docs/receipts/2026-10-15_netrender_adoption/README.md`.
The older local Windows Forms B snapshot was superseded by qualified Phase B
already on Genet main. Its temporary duplicate archive tag was removed locally
and remotely at Mark's clarification. Forms C and three-lanes worktrees were
not edited, merged or pruned.

Canonical execution plan: [browser presentation tenants](../../../implementation_strategy/2026-10-13_browser_presentation_tenants_plan.md#coordinated-supplier-adoption-2026-10-15).
