# Production renderer acceptance — 2026-10-11

The stable Vello stack now has separate production source-capture, resource,
input/accessibility, native-presentation and orderly rehydration evidence.
GPU/native execution was explicitly handed off by Tabard and coordinated with
Moot. Only the coordinator built (two jobs) and launched GPU processes, one at
a time; three source/review lanes ran concurrently without adapters. The finite
GPU/native lane has been released and its owned sleep inhibitor stopped.

| Gate | Result | Evidence |
| --- | --- | --- |
| Intact actual workshop source capture | 12 lifecycle cases passed, widths 1280/420 | [capture control](checks/production-capture-final.log), `source-captures/*.json` |
| Registry admission and aggregate budget atomicity | Four CPU controls passed | [registry controls](checks/aggregate-registry-qualified.log) |
| Portable refusal with existing real GPU view | Passed, original view/stamp/owner retained | [real-device control](checks/portable-live-gpu-refusal.log) |
| Actual native-hook input, accessibility and application authority | Three controls passed | [host controls](checks/release-host-acceptance-fixed-slider.log) |
| Normalized slider assistive value route | Two controls passed | [slider controls](checks/slider-value-controls.log) |
| Full workshop CPU vs Sparse GPU | 12 accepted composition comparisons | [metrics](qualified-parity-metrics.json) |
| Actual producer CPU vs Sparse GPU | 24 independent scene comparisons accepted | Same metrics; complete per-case receipts in Netrender |
| Native presentation and resize continuity | RESULT ok, six distinct/nonblank captures, 53 presentations | [scenario receipt](native-presentation-qualified/scenario.done), [process](native-presentation-qualified/process.json), [pixel verification](native-presentation-qualified/capture-verification.json) |
| Orderly fresh-device and CPU-session rehydration | Exact full-frame RGBA after two boots, missing source refused atomically | [recovery receipt](device-rehydration-qualified/receipt.json), [control](checks/device-rehydration-qualified.log) |
| Classic three-renderer pixel parity | Open: curve/glyph edge thresholds exceeded | Netrender receipt and offline diagnosis |

Production capture covers cold, repeat, RGB edit preserving authored seed alpha
73, mode, resize and source rebind for each width. It preserves the existing
workshop draft/mode/dirty state and retains the full envelope, fonts, UI
commands, both bound producer scenes and two external draws. This specimen has
zero shadow requests. Font-bearing postcard captures and complete bundles stay
local; only metadata and rendered pixels are committed.

Callbacks stage owned source scenes. Aggregate source count, nested structure
and font/image bytes are checked across the complete borrowed plan before any
digest or content-stamp work. Only referenced sources move into the intact
capture; whole-envelope admission precedes cache/lifecycle commits and retained
source clones. The two-source negative control uses individually permitted
scenes whose combined structure exceeds the quota and verifies preservation of
accepted stamps, scenes, portable/GPU owners, active flags and retirement queue.
The default GPU-only callback refuses portable capture explicitly. Callback
preview layout caches may update; capture does not replace editor authority.

The input controls use the production native initialization/hooks and real
Cambium routing. They cover text selection, frame preparation, zoom/resize,
dirty-close cancellation, focus without activation, preview-mode activation,
normalized accessibility SetValue, undo, and actual preview Image semantics.
They found and fixed an actual slider omission: the control advertised its
value range but did not handle ValueEvent. The handler now accepts finite
values, clamps to 0–1, and preserves state on NaN/infinity.

The native scenario uses a fresh explicit library file through LaunchServices,
preserving draft, preview mode, focus and graph selection through 960×780 and
1280×1100 resizes. Each capture follows successful presentation. The first run
is retained as failed acceptance: its ambiguous Dark selector chose the library
entry, so its six captures/53 presentations did not establish a dark mode
change. The corrected scenario focuses exact `data-mode=dark` and activates it
with Space, preserving the mode assertion. Qualified
[dark](native-presentation-qualified/dark.png) and
[resized](native-presentation-qualified/resized.png) captures were visually
inspected; all six PNGs passed independent nonblank pixel-statistics checks.

The recovery control uses actual production reader CPU pixels and explicit
premultiplied-to-straight conversion, transparent target initialization, real
host upload/readback and five-second per-copy deadlines. It drops the first
core and all dependent handles before a second boot, verifies alpha-overlap
pixels and exact whole-frame equality, rehydrates a fresh CPU session from
owned sources, and preserves application state. Both 160×152 raw RGBA frames
have SHA256 `a307988075899ea251cf7b9f031b0ca775276bbe6ffbb7231ac9d7a7f45d5dd9`.
This establishes orderly teardown/recreation; `forced_driver_loss=false` and
`swapchain_presentation=false` remain explicit in that separate receipt.

CPU/Sparse GPU comparison preserves premultiplied encoded-sRGB RGBA, including
alpha, with original edge/interior/mean thresholds. Maximum full-composition
channel differences were 3 at edges and 2 in interiors; direct producers were
2/1. Classic curved border coverage and actual glyph edges exceed thresholds,
while producer interiors agree exactly. No commands, transparent pixels or
canonical curves were removed, and thresholds were not widened. Classic
remains the shipping host renderer; Sparse selection stays opt-in.

The diagnostic filename/metadata comparison found no new or changed matching
GPU/Kernel/WindowServer reports during the owned batch; see
[comparison](gpu-diagnostics-comparison.json). This is a bounded observation,
not a forced-reset or systemwide health qualification.

Browser execution, OS assistive-technology adapter interaction, native text
caret/selection pixel assertions and forced driver-loss recovery remain open.
The exact retained text selection is covered by the CPU hook test. The
[acceptance plan](../../../../../ports/tabard/desktop/scenarios/release_acceptance.plan.json)
and [native scenario](../../../../../ports/tabard/desktop/scenarios/release_presentation.scn)
retain reproducible commands and scope boundaries. Initial runtime checks used
the coherent local Netrender candidate patch. Git-only qualification now passes
for all five active metadata roots with one package identity per Netrender
crate, immutable source `ef5412d760a978584c8c1801e8a765c51aecd403`, Genet
`4bc75583865c2fb862b7acd7fe16dc28eb591042`, and the unchanged Vello provider.
See [pin qualification](pin-qualification.json). Published-source CPU reruns
pass the twelve-case production capture, three host controls, four registry
controls and two slider controls; [port boundaries](checks/mere-port-boundaries.log)
also pass. The default Graphshell Wasm library passes a Git-only locked
check; [compiler receipt](checks/graphshell-wasm-pinned-library.json). This
compilation does not qualify browser execution. The locked native Tabard
consumer check also passes; [compiler receipt](checks/tabard-pinned-native-check.json). The standalone web-smoke
workspace intentionally ignores its local lock; its resolved
[lock snapshot](genet-web-smoke.Cargo.lock) is preserved here. The global documentation audit in this isolated checkout reports
unrelated existing sibling/source-path findings; it reports no findings for
this new receipt, no index orphans, and no owned-source license-header failures. Raw Cargo logs preserve
their original whitespace and warnings.
