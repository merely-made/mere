# Current renderer browser and open native gates — 2026-10-11

Current production Graphshell passes eight headed Chrome stages with twelve
actual Rootstock GPU readback captures. The source-bound build uses Mere base
ec1e65caed335010308566c8db51ad0e1e002799 plus the recorded renderer scenario,
Netrender ef5412d760a978584c8c1801e8a765c51aecd403, Genet
4bc75583865c2fb862b7acd7fe16dc28eb591042 and Vello provider
5862bba26c8a9f56f6a5fdc8ea6966b360a5d63a. This is the current product tree,
not an older cached package. Root built at two jobs with no incremental/debug
state and matched wasm-bindgen 0.2.129. The packaged JS/Wasm hashes and build
source fingerprint are in [identity-rerun.json](identity-rerun.json).

| Gate | Observed result |
| --- | --- |
| Production cold/repeat, rule draft/discard, zoom, selection, keyboard and details | Scenario passes; node/session/source geometry invariants pass within its actions |
| Four appearance modes | dark, light, hc_dark, hc_light; actual distinct mode captures and stored session preserved |
| Narrow and wide resize | 420x900 then 1440x1000; actual graph paint visible after ordinary drawer-close/Fit graph at narrow width |
| Fresh page on same isolated origin/context | IndexedDB reopened, same session and hc_light appearance; source graph produced again |
| Actual producer paint | Twelve PNGs pass an independent opaque IDLE-node color check; cold/repeat producer regions are exact |
| OS accessibility | Compiled Swift helper runs, but AX trust is false; no target tree query or action runs |
| Host device loss/reset | Architecture proposal only; production notification/replacement seam remains absent |
| Classic cross-renderer parity | Open; separate minimal Netrender curve/glyph diagnosis |

The [final browser qualification](browser-qualified/qualification.json),
[default adapter](browser-qualified/capability.json) and
[independent pixel/visual review](browser-qualified/root-pixel-review.json)
are authoritative. Chrome 152.0.7977.83 requested its normal default AMD/gcn-5
adapter; GPUAdapterInfo reports isFallbackAdapter=false. No unsafe WebGPU,
software or GPU-disabling flags were added. The context/profile was ephemeral.
The actual local CanvasProducer rendered; DOM controls were not rebuilt as a
replacement renderer. Browser inputs use existing production scenario/DOM
routing, not actual OS assistive-technology actions.

Pixel entropy is insufficient for producer proof: an open Graph tools drawer
covered the narrow graph in [a rejected run](browser-covered-graph-rejected/root-visual-review.json).
That frame had zero actual IDLE node pixels despite many UI colors. The final
source-bound guard requires at least 64 opaque pixels matching the actual
Canvas IDLE fill [54,92,156], within three channel levels, in the producer's
semantic crop. Application modes change face/backdrop colors but preserve this
activation accent. All final captures contain 6385–10450 qualifying pixels.
[Final narrow capture](browser-qualified/narrow/renderer_narrow.png) visibly
shows graph nodes after closing the drawer through its normal control and
fitting the graph. This qualifies that workflow, not visibility underneath an
open covering drawer or automatic camera fitting.

The [first failed run](browser-first-failure/qualification.json) attempted
Pause while the analytic layout was already paused. The scenario now asserts
that state, and the driver conditionally pauses only when the actual semantic
Pause control exists. A [second failure](browser-collapsed-panel-failure/qualification.json)
tried to select a hidden mode; the final driver opens Appearance through its
normal summary. These failed receipts remain preserved. The third run's raw
driver pass is superseded by root's rejecting visual review, not accepted.

The [scenario](../../../../../ports/graphshell/web/scenarios/renderer_acceptance.scn)
and preserved [driver](qualify.cjs) reproduce the browser lane; [preparation
recipe](browser-preparation.md) records build/server commands. Do not treat a
successful process exit or nonblank UI alone as pixel acceptance. Modes and
resize preserve stored-session identity; this receipt does not assert that
transient selection/camera survive fresh-page rebind. Nor does it prove browser
CPU/Classic parity. The browser GPU-only producer still explicitly refuses
portable source export; intact portable bundle export remains open.

The [OS trust receipt](accessibility-reset/swift-trust.json) reports trusted=false
and permission_unavailable_no_prompt_no_actions. The actual compiled helper
exited before selecting any target PID. Root neither prompted nor changed
OS privacy settings. The [prepared protocol](../../../../../crates/cambium/tabard-workshop/tests/os_accessibility_reset.plan.json)
and committed Swift/scenario source cover actual target identity, finite AX
queries/actions and truthful unsupported text operations. Actual AX execution
needs an identity with existing permission and a freshly built owned host.
Native text caret/selection pixel qualification remains separate.

The same protocol records the current shared RenderCore gap: no installed
loss callback, generation event or group replacement owner. Surface Lost
currently reconfigures the same device. A proposed monitor must register once
per shared device, wake all subscribers, reject stale notices before raster,
invalidate old resources and replace the shared core once while preserving
application authority. It is an architecture proposal, not tested recovery.
Device::destroy would qualify controlled destruction only; earlier orderly
two-device recreation and synthetic notifications cannot establish driver
reset recovery.

Both coordinating tasks explicitly released their reservations for this finite
batch. Only root built/launched runtime; source agents ran no adapters.
Owned browser contexts, loopback server and sleep inhibitor were stopped; the
GPU/native/browser lane and borrowed mere-s2s3 target were released. Borrowed
caches were preserved. Matching diagnostic filename metadata did not change.
That bounded observation is not systemwide health or forced-reset acceptance.

Netrender diagnosis and source controls are published at [a30c7db42](https://github.com/merely-made/netrender/blob/a30c7db42710c6f0cf38d8ed66ac876eb3a29802/netrender-notes/receipts/2026-10-11_classic_edge_gates/README.md).
The [owned documentation audit](checks/mere-doc-audit-summary.json) reports no
findings for these new files. Global existing source/path findings remain
recorded separately. Source whitespace checks pass; raw Cargo logs retain their
original bytes, including blank lines, for the recorded hashes.
