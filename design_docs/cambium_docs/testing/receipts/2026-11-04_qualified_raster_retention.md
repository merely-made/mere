# Qualified raster retention — bounded native qualification

**Status, 2026-11-05:** R2/R3/R5/R6/R7 retain bounded local qualification
and their exact evidence cuts. R7 identifies a query/draw CPU target without
an overall speedup claim; no COW or pooling is implemented. R8's Mere owner
cut on 81e passes 366 native tests, seven changed-example checks and both
standalone wasm checks. Current-main forward receiving, Mere publication and
Isocosm actual-cohort qualification remain separately tracked below. R8 is
active. Complete CPU decomposition, actual GPU duration, consistent 60 Hz,
R4 and broader physical/platform qualification remain open. Earlier receipt
sections retain their original cuts. The
[owner plan](../../implementation_strategy/2026-11-04_qualified_raster_retention_plan.md)
holds the accepted boundaries and active integration work.

Retained evidence is under
`Code/output/mesocosm-scene-composition/2026-11-04/raster-attribution/`.
Artifact names below are relative to that directory. The earlier successful
C4 trials remain under its parent directory and keep their own receipt in
`isocosm/mesocosm/design_docs/archive_docs/receipts/2026-11-04_scene_composition_plan_receipts.md`.

## 2026-11-04 — preserved attribution and failed native replays

The earlier successful C4 retained/component trials both have `ok:true` and
all six phases complete. They use the same frozen release executable,
explicit paused start, seed 9, population 256 and native bench inputs. Their
saved worlds are byte-identical after three manual native steps. All 266
eligible frames per run exceed the 16.7 ms CPU wall-time budget. Paused-frame
median/p95/p99 totals are 25.224/25.959/26.512 ms for retained rows and
25.386/26.201/26.616 ms for components. Those measurements establish neither
physical refresh deadlines nor physical ecology/gait acceptance.

In those successful paused phases, scene redraws, producer staging and
layout mutations/rebuilds are zero. The outer checked raster call is about
21.7 ms at median while the inner renderer timing is about 1.4 ms. These
spans have different scopes; their difference is not a GPU-duration measure.
The raw `retained-paused.json`, `components-paused.json`, timing files and
`mixed-comparison.json`/`mixed-summary.json` remain the C4 evidence.

A fresh `/usr/bin/sample` of the frozen old release executable records
3,441 main-thread samples, of which 2,784 fall under
`RenderCore::try_rasterize_scaled_for` → `wgpu::Device::poll` (about 80.9%
of that main-thread sample weight). Its retained call graph reaches wgpu
submission triage. Source inspection places the bounded synchronous native
quality wait at that boundary. Sample rows are inclusive and are not added
as independent times. `profile-exact.sample.txt` records macOS 15.8.1,
x86-64, sampled every millisecond.

That fresh run is **failed attribution evidence**: `profile-exact.json`
has `ok:false`, 368 redraws, zero accepted presented frames and the
10,000 ms native-presentation-stalled watchdog error. Its process exit is 1;
the sampling command exits 0. `surface-diagnostic.log` separately records
`acquire-unavailable` with `surface acquire skipped: Occluded`; its receipt
has 373 redraws and zero presented frames. These are not successful headed
performance fixtures.

The foreground app launch of the same frozen old binary also fails the
presentation watchdog: `foreground-baseline.json` has 389 redraws and zero
accepted frames. `foreground-baseline-open-exit.json` records successful
submission to the app launcher, not successful application presentation.
The executable is `bin/mesocosm-genet-release-v2` under the parent evidence
directory, SHA-256
`ab989213245e4b55b95aef3220691603c0bb207ebca537e506650aebe4c4126d`.
These failed fresh attempts do not replace or reclassify the earlier
successful C4 trials.

## 2026-11-04 — fixed SDK and bounded source overlays

The qualification starts with the selected SDK source, rather than replacing
complete packages with current owner heads:

| Supplier | Selected immutable source |
| --- | --- |
| Mere | `5431b896c8c277c4d8ce2cf33cdfd33692e82cd8` |
| Netrender | `11e208b84a981993df8077bb6025f8b3adbe2291` |
| Genet | `b7c0bef04613d854367d407b91c87a38b9b03b35` |
| Vello | `276b46648c08f3d2062e70559afa999842e1873c` |
| wgpu | `30.0.1` |

`sdk-before-overlays-Cargo.lock` has SHA-256
`459ce738160c9933d2cd075cf5aaa282a3bbab9a72e29d8364754a854cfdfb2f`.
`generate_sdk_overlays.py` copies complete pinned packages, then applies only
approved changed/new owner paths. Pinned manifests are normalized for
standalone resolution with the selected sibling Git identities and feature
declarations preserved. External literal test assets are retained. Owner
worktree heads are not supplier adoptions.

`overlays/corrected-03/overlay-manifest.json` records **22 approved source
paths across five packages**:

| Package | Approved source paths | Purpose |
| --- | ---: | --- |
| `cambium-rootstock` | 12 | Prepared producer witness, one qualified output per host/driver, presentation, recovery and controls |
| `mesquite` | 1 | Retained-raster observation in shared frame costs |
| `paint_list_api` | 5 | Complete content comparison and controls |
| `netrender` | 2 | Renderer-policy introspection and control |
| `genet-presenter` (`genet_render_host`) | 2 | Exclusive native admission/loss checks for cloning a qualified raster |

The corrected manifest SHA-256 is
`547917106e67970f927e183040951022eafe7cfd61e922bfe3e38bbcda7e3b95`.
The final read-only `standalone-lock-and-source-audit-04.json` reports
`static_audit_ok:true`, all approved owner/source copies equal, no captured
copy mismatches and no excess source deltas. Resolved locks are clean;
unsuccessful historical resolutions remain accounted for rather than being
counted as completed gates. Additional Iced and sparse Vello records resolve
existing immutable optional declarations. Executed Rootstock fingerprints
select only `default` and `image-decode`; Iced is not compiled.
`paint-api-preimage-test-extraction.json` separately accounts for the
preexisting move of inline tests into a child file, preserving the
production prefix and normalized test body.

Builds use the existing guarded `mesocosm-scene-composition` lane through
`run_guarded.py`, which sets absolute Rust 1.99.1 compiler/tool paths after
Python startup. Exact commands, patch arguments and exits are retained per
gate. These standalone offline package runs are distinct from a locked
receiving product build.

## 2026-11-04 — executed pre-fix control

The retained `baseline-02` overlay has the complete control and preparation
seams but no retained-output lookup in the native raster path. Its exact
actual-GPU control executes one test and fails with exit 101:

`presentation::wgpu::tests::repeated_checked_frame_reuses_output_and_matches_an_independent_cold_raster`

The expected failure is **`unchanged checked output was rasterized again`**.
Before that assertion, the initial red pixel, repeated pixels and an
independent cold raster agree, and the cold target is distinct. Thus this is
an executed repeated-raster failure after working texture controls, rather
than a compilation or fixture-admission failure.

- Log/command/exit: `baseline-02-standalone-control-v2.{log,command.json,exit.json}`.
- Proof: `baseline-02-control-proof.json`.
- Frozen executable: `bin/rootstock-pre-fix-control`, 98,206,592 bytes;
  SHA-256 `84040a83860457f32e17594404d3054244c1cb6bc5f17e91a2866a3c97178b39`.
- Baseline source manifest: `overlays/baseline-02/overlay-manifest.json`,
  SHA-256 `8e5d7cdd1ef54518ca17a4c3cb98d8e9df87fd19bb61e5f7a1ab1f9b8c0bfb43`.

## 2026-11-04 — corrected bounded controls

| Gate | Executed result | Evidence |
| --- | --- | --- |
| Corrected Rootstock library | **193 passed, 0 failed, 3 existing ignored**, exit 0, 108.97 s | `corrected-03-rootstock.log` and command/exit JSON |
| Paint API library | **14 passed, 0 failed, 0 ignored**, exit 0 | `corrected-03-paint-api.log` and command/exit JSON |
| Genet native reuse admission | **3 passed, 0 failed, 0 ignored**, 35 filtered, exit 0, 0.50 s; invoked with `--ignored --test-threads=1` | `corrected-03-native-admission.log` and command/exit JSON |
| Mesquite library | **42 passed, 0 failed, 0 ignored**, exit 0 | `corrected-03-mesquite.log` and command/exit JSON |
| Focused Netrender policy | **1 passed, 0 failed**, 83 filtered, exit 0; includes contended-lock and animated-overlay refusal | `renderer-policy-focused-v3.log` and command/exit JSON |
| Optimized offscreen workload | **1 passed**, 195 filtered, exit 0; all 32 retained requests reuse the checked texture | `corrected-03-release-workload.log` and command/exit JSON |
| Receiving Mesocosm release build | **exit 0**, fixed SDK plus five approved package overlays | `mesocosm-consumer-build.log`, command/exit JSON and `consumer-lock-audit.json` |

Rootstock's result includes **13 CPU cache controls and five actual-GPU
controls**. The CPU controls cover exact payload/resource/physical-producer
identity, publication-only changes, target/device/context changes, clearing,
portable and retained-fragment refusal, and conservative NaN misses. The GPU
controls establish repeated-output reuse against an independent cold raster,
changed inline pixels under a stable key, refusal during a pending native
batch, no publication for an unavailable surface, and the bounded unchanged
offscreen workload. That last control records 32 pairs at 1280×800 with
128 ordinary rectangles in a debug build. That debug result is separate from the optimized workload and headed
receiving measurements below.

The three existing ignores are two thousand-row layout measurements that
request release mode and the preexisting portable-refusal real-device cache
control. They are not counted as passed. The paint API run includes the six
new content/resource controls and eight existing controls.

The Genet gate explicitly executes all three ignored native reuse controls:
exclusive admission while cloning, refusal while a native batch is submitting
or pending, and actual device destruction during reuse followed by refusal
before later clone callbacks. Successful reuse submits no new raster or
quality-counter work; new rasters still use existing checked qualification.

The frozen corrected Rootstock executable is `bin/rootstock-qualified-debug`,
98,576,800 bytes, SHA-256
`f27f34804f06f8617fffb6c15b1ebe8141370f127436821a77c919e867760ecd`.
`corrected-03-rootstock-binary.json` records its identity and gate counts.

## 2026-11-04 — resolver refusal and remaining gates

The Netrender policy attempt encountered the unchanged dev-only
`netrender_text` declaration. Standalone resolution tried Git `main`
`2073c505` and refused at a `wesl-macros` conflict through optional
`vello_hybrid`/shader dependencies. **No source from that attempted new
Netrender/text revision compiled.** This is a failed resolver attempt, not
an adopted supplier cohort or a policy-test result.

The completed `renderer-policy-01` qualification-only retry preserves all
library/test source bytes and feature definitions, omitting only the unused
`netrender_text` dev dependency. Focused library source mentions that crate
only in four documentation links. Initial focused resolution still hit stale
0.4.4 shader-tool lock hints. The fixed Vello `276b466` shader manifest
explicitly requires `wesl`, `wesl-macros`, `wgsl-parse` and `wgsl-types` all at
`=0.4.0`; only those stale hints were removed from the focused test lock and
Cargo resolved/verified the required cached 0.4.0 packages. Records are in
`renderer-policy-lock-reselection.json` and
`renderer-policy-wesl-family-reselection.json`. The resulting policy test
passes. Production and receiving-product manifests/locks do not receive
these qualification-only adjustments. The attempted wrapper `cargo update`
was refused before execution; it is retained as setup evidence, not a gate.

## 2026-11-04 — optimized offscreen workload

This fixture has 128 ordinary rectangles at 1280×800, one admitted packet,
and 32 alternating forced-raster/reuse pairs on the same path/Core. It times
the raster helper including policy, payload comparison and admission. It
excludes DOM layout/translation, scene production, surface acquisition,
composition and presentation. Percentiles use nearest rank; p99 is the
maximum with 32 requests. No GPU-duration inference follows.

| Optimized request | Median | p95 | p99 |
| --- | ---: | ---: | ---: |
| Forced checked raster | 37.9605 ms | 38.829 ms | 44.157 ms |
| Retained checked image | 0.003 ms | 0.004 ms | 0.017 ms |

`raster-workload-summary.json` retains both optimized and debug raw vectors
and frozen executables. The optimized control binary is
`bin/rootstock-qualified-release`, SHA-256
`b2b3087d2413ac71437b8d7cf75e5de35637903fcf393bc8807d1674f5ab0ab2`.

## 2026-11-04 — matched headed receiving

The release receiving executable is `bin/mesocosm-qualified-release`,
SHA-256 `b8c15514c82df0312f4274e191d1d66c2e0dba645bcdb0e55b4da75d2b4767a0`.
`consumer-binary.json` identifies its frozen copy. The receiving lock differs
from the fixed SDK by exactly five named Git-to-path package replacements;
all other records are unchanged. Canonical supplier pins remain untouched.

Later runs complete with **both old and new executables**, superseding the
fresh headed availability limitation for this receipt. The earlier occlusion
failures remain failed evidence. Their cause was not isolated, and successful
startup is not attributed to retention.

All four runs complete six phases, two captures, three manual native steps,
17 successful scene draws and four scene extractions, with zero projection
failures. New retained/components runs accept 349/350 frames; both current
baseline runs accept 349. Each has 266 eligible non-capture timing samples.
The WorldSave files are byte-identical across all four runs, SHA-256
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`;
the native state hash is `2f1d926b35f3314f`, living 263, selected organism 38,
part 1. This remains the coarse native bench: physical members/native poses
are zero, and it is not an embodied ecology/gait certification.

| All eligible host wall times | Median | p95 | p99 | Strictly over 16.7 ms |
| --- | ---: | ---: | ---: | ---: |
| Retained baseline | 25.699 ms | 30.698 ms | 43.861 ms | 266/266 |
| Retained receiving | 16.3735 ms | 37.009 ms | 47.995 ms | 21/266 |
| Components baseline | 25.659 ms | 30.978 ms | 41.501 ms | 266/266 |
| Components receiving | 16.334 ms | 37.806 ms | 48.750 ms | 25/266 |

Each receiving run reuses **251/266** eligible rasters, including all 62
paused-picture samples. Retained paused work excluding acquisition,
presentation and capture has median/p95/p99 **3.6005/3.784/3.896 ms**, down
from **25.853/26.861/27.501 ms**. Paused checked raster medians
22.0565/21.980 ms become zero new raster work on retained/components hits.
The new retained paused acquisition median is 12.7585 ms, so the remaining
16.374 ms host wall median includes substantial surface pacing/waiting.

At this R2 cut the changed-picture tail remained unresolved. Mixed p95/p99
increase in these trials; native-step maxima reach 55.915/56.661 ms. All 15 newly rasterized
requests per receiving run still use the synchronous checked path. The
measurements do not isolate GPU duration or the cause of the increased tail.
They justified the subsequent R3 receiving/producer lifecycle work, recorded
below; they did not establish consistent 60 Hz.

`headed-before-after-summary.json` retains raw-derived phase distributions,
strict `>16700us` and exact `3*duration>50000` counts, world identities and
per-run facts. Exact-60-Hz counts are 28/30 over budget in the new mixed
runs; these are CPU wall duration counts, not physical missed refreshes.
`receiving-backend-comparison.json` and
`current-baseline-backend-comparison.json` use the preserved C4 analyzer.
Commands, exits, receipts, timing files, WorldSaves and captures use the
`receiving-*` and `current-baseline-*` prefixes.

The retained before/after final PNGs and both receiving final PNGs are
byte-identical (SHA-256
`b0bb8bb38472939068afe2bd395dd9f2df0bf0452a298934845fb0cbf11cd082`).
The current components baseline final PNG differs; no all-four image identity
is claimed. `headed-capture-identities.json` records each image and capture
digest. The retained before/after captures were also visually reviewed.

## Qualification remaining after the R2 cut

- R3 subsequently qualifies Rootstock receiving of Genet's pending/accepted
  native batches, immutable displayed facts, producer submission capability,
  bounded coalescing, stale-work cancellation and capture/input publication;
  its executed evidence is appended below.
- Changed-picture throughput/tails and representative active worlds.
- Coordinated owner integration and product supplier-pin adoption; source
  remains in isolated lanes. The final R3 audit records canonical Netrender's
  separate clean advance without adopting it into qualification.
- Broader hardware/platform and physical presentation qualification.

GPU duration, end-to-end input-to-display latency, allocation/heap totals and
physical refresh deadlines are unmeasured. The sample header's process
footprint is not a heap-allocation measurement. Offscreen pixels and native
admission controls do not certify successful WSI presentation. Rendering
demand remains separate from Cosmograph's admitted simulation timing,
attention, handbacks, matter and replay authority.

## 2026-11-04 — R3 native frame receiving and final scheduling qualification

This R3 continuation records the generation-07 scheduling and Core-availability
repair: executed controls, successful bounded headed workloads and the final
CPU/cadence/pixel comparison. Consistent 60 Hz, broader hardware/platform and
supplier-adoption qualification remain open. R2 evidence remains unchanged
in the sibling `raster-attribution` directory.

All short artifact paths below are relative to
`Code/output/mesocosm-scene-composition/2026-11-04/native-frame-receiving/`.
The isolated owner lanes remain `Code/worktrees/mere-qualified-output`,
`genet-qualified-output`, `netrender-qualified-output` and the Isocosm
receiving lane `isocosm-scene-composition-validation`. These local results do
not integrate the owner changes into canonical product/supplier checkouts.
The separately observed canonical Netrender advance is recorded below.

### Source and fixed SDK

The fixed supplier commits are Mere
`5431b896c8c277c4d8ce2cf33cdfd33692e82cd8`, Genet
`b7c0bef04613d854367d407b91c87a38b9b03b35` and Netrender
`11e208b84a981993df8077bb6025f8b3adbe2291`. Seven SDK packages receive
copied, hashed source overlays: `cambium-desktop`, `cambium-rootstock`,
`cambium-a11y`, `mesquite`, `genet-presenter`, `netrender` and
`paint_list_api`. Each generation starts from the pinned package source and
copies its admitted owner paths; unrelated owner-HEAD differences are not
supplier updates. The guarded launcher uses the stable
`mesocosm-scene-composition` lane and the explicit Rust 1.99.1 toolchain.

| Immutable overlay manifest | SHA-256 | Qualification associated with it |
| --- | --- | --- |
| `overlays/receiving-pre-fix-01/overlay-manifest.json` | `7dcb5b7f4685151b19dfb2c89c41c7e42ca21d55242c2c7e156b61ab5e1b03f9` | The R2 presenter with the R3 common assertion added. |
| `overlays/r3-03/overlay-manifest.json` | `29e64633927a5eade3ccac04ab3c3d23ff87ce84264911c42d676887b3732bf5` | Rootstock, Mesquite and the Mesocosm receiving gates listed below. |
| `overlays/r3-05/overlay-manifest.json` | `8f9a176f964922c4afd66edf4d235b1c055b0af4d92a86ffbf3503925522ddaf` | Full default Rootstock gate after driver/capture policy receiving. |
| `overlays/r3-06/overlay-manifest.json` | `57f3322fab0c7378fbf29f6fcac4809d03f46ada63bc288a0f28ac2bfb87c439` | Pre-scheduling-06 Rootstock gate, including ignored GPU controls and the explicit incremental-group skip. |
| `overlays/r3-07/overlay-manifest.json` | `bf22b046dc54525927bb9e89d34626c00cbd65dae01cfbe5a942c30d6f8a6249` | Final scheduling/availability generation used by Rootstock, Genet and the generation-07 Mesocosm receiving gates. |

The Mesocosm receiving compile is recorded in
`receiving-runs/mesocosm-product-compile-03/{command.json,result.json,output.log}`.
It ran `test --release --offline -p mesocosm-genet --no-run -j 4` through the
guarded wrapper and exited 0. The receiving manifest stayed unchanged.
The source-aware lock comparison records fourteen changes: the removed
selected-Git record and added local-overlay record for each of the seven
packages above. It records zero Git records outside the original SDK source
identities. This is a seven-package source-overlay qualification, not a
moving-branch update or publication. The resulting lock SHA-256 is
`1ecb545b99546caee76d1251239f99350d84f978a5144513805654d7d82a5ba9`.

The generation-07 product compile is retained in
`receiving-runs/mesocosm-scheduling-compile-07/`. Its result also exits 0,
keeps the receiving manifest unchanged, records the same fourteen
Git-to-local-overlay identity changes, and records no Git identity outside
the original SDK. The resolved product lock again has SHA-256
`1ecb545b99546caee76d1251239f99350d84f978a5144513805654d7d82a5ba9`.

The product source and compatibility inventories are retained separately in
`r3-product-source-freeze-02.json` and
`r3-consumer-compatibility-freeze.json`. They identify preparation inputs;
the overlay and executed-binary identities identify each actual gate.

### Genuine before/after receiving control

The common actual-GPU control is
`native_receiving_control::redraw_returns_while_the_native_frame_is_receiving`.
The pre-fix overlay preserves the R2 Rootstock source except for its test
module/export. Its assertion body is byte-identical to the R3 control,
SHA-256 `826ff7050c5ebcd1ca165f45a5bf7e332eb7832cac207d4a53310c5e4f8e6c99`.
The old fixture omits only the then-unavailable `Surface::shared_core`
adapter method. The overlay manifest records that compatibility difference
and verifies the R2 source was unchanged after copying.

`receiving-pre-fix-control-01.log` executes one test and fails at the actual
redraw assertion: the synchronous R2 path returns idle instead of retaining
pending native acceptance. Its exit is 101, with 0 passed, 1 failed and 196
filtered. `pre-fix-binary.json` freezes
`bin/rootstock-receiving-pre-fix`, SHA-256
`0653733299045bb0d969a6daee08f9b37faf343ddab11ea74f48eba82302f599`.

`receiving-post-fix-control-02.log` executes that same control against R3:
1 passed, 0 failed, 0 ignored, 223 filtered; exit 0. Its command identifies
source generation `r3-02` and frozen binary
`bin/rootstock-receiving-02`, SHA-256
`56c2fcae730c7483a89e17a5938af333d14e7c9f1c1e49f3e4fa4bd6395219cd`.
The control also passes in the serialized R3-03 and R3-06 GPU gates. It
proves that ordinary redraw returns while the actual shared Core lease is
pending; its fixture is not a compositor/physical-display throughput test.

This is separate from R2's earlier unchanged-raster pre-failure and corrected
retention result. Those original artifacts and timing comparisons remain in
`Code/output/mesocosm-scene-composition/2026-11-04/raster-attribution/`.

### Executed gates at this cut

| Gate and retained log | Executed result | Scope |
| --- | --- | --- |
| `r3-rootstock-release-03.log` | 222 passed, 0 failed, 6 ignored; exit 0 | Full default Rootstock library gate for R3-03. |
| `receiving-native-gpu-03.log` | 6 passed, 0 failed, 0 ignored, 222 filtered; exit 0 | All six explicitly ignored Rootstock controls for that same source generation. |
| `r3-rootstock-release-05.log` | 223 passed, 0 failed, 9 ignored; exit 0 | Full default Rootstock library gate for R3-05, including the existing incremental group. |
| `r3-rootstock-final-06.log` (pre-scheduling-06) | 227 passed, 0 failed, 0 ignored, 5 filtered; exit 0 | `--include-ignored --skip owned_layout::incremental_tests --test-threads=1`. The unchanged five incremental controls are explicitly filtered here and passed in the preceding full R3-05 gate. |
| `r3-rootstock-scheduling-07.log` | 234 passed, 0 failed, 0 ignored, 5 filtered; exit 0 | Final generation-07 `--include-ignored --skip owned_layout::incremental_tests --test-threads=1`; includes actual GPU receiving, driver, terminal and two-host scheduling controls. The five unchanged incremental controls passed in full R3-05. |
| `r3-genet-native-07.log` | 22 passed, 0 failed, 0 ignored, 24 filtered; exit 0 | Genet native-frame and availability controls, with ignored actual-GPU controls included. |
| `runtime/isometer-all-06/output.log` | 73 passed, 0 failed, 0 ignored, 0 filtered; exit 0 | Full Isometer unit gate with ignored frozen body-query, terrain-query and staged-upscale GPU controls included. |
| `r3-a11y-focused-test-01.log` | 23 passed, 0 failed, 0 ignored; exit 0 | Focused `cambium-a11y --lib` bootstrap/consumer gate under the bounded dev-only manifest adjustment below. |
| `r3-mesquite-tests-03.log` | 47 passed, 0 failed, 0 ignored; exit 0 | Capture pairing, pending-attempt accounting and existing scenario-lane controls. |
| `runtime/mesocosm-default-03/output.log` | 78 passed, 0 failed, 6 ignored; exit 0 | Full default Mesocosm library gate. |
| `runtime/mesocosm-gpu-03/output.log` | 5 passed, 0 failed, 0 ignored, 79 filtered; exit 0 | Explicit actual-device Mesocosm tests; the ignored CPU component-performance diagnostic is deliberately skipped. |
| `runtime/mesocosm-world-save-03/output.log` | 1 passed, 0 failed, 0 ignored; exit 0 | `the_heads_writer_loads_in_the_core_and_watch_carries_on` WorldSave integration control. |
| `runtime/mesocosm-default-07/output.log` | 78 passed, 0 failed, 6 ignored; exit 0 | Final generation-07 full default Mesocosm library gate. |
| `runtime/mesocosm-staging-gpu-07/output.log` | 1 passed, 0 failed, 0 ignored, 83 filtered; exit 0 | Final generation-07 actual-device staged scene publication control. |
| `runtime/mesocosm-world-save-07/output.log` | 1 passed, 0 failed, 0 ignored; exit 0 | Final generation-07 WorldSave integration control. |

The R3-03 explicit Rootstock GPU gate covers the common receiving control,
actual device loss/cancellation, bound-capture refusal on unavailable
surfaces, resize/hidden cancellation, producer stage/publish/discard and
portable refusal preserving a live GPU image/stamp. The frozen binary is
`bin/rootstock-receiving-release-03`, SHA-256
`7832d2866834ca70695329c0ca7e1de7636be8e613f4c417ea50cb693d70af1d`.

R3-06 additionally executes the previously ignored driver policy/capture
controls and `actual_producer_shader_refusal_never_publishes_and_next_valid_batch_reaches_acquisition`.
The latter submits deliberately invalid WGSL under the actual GPU validation
scope: it refuses publication, then a valid batch reaches acquisition.
`driver_keeps_capture_a_pixels_and_stamp_after_receiving_b` checks preserved
capture A pixels/stamp after B; the custom-driver control verifies that a
shared native surface alone does not bypass custom presentation policy.
These are executed actual-device fixtures, not an OS presentation or input
latency measurement.

`mesocosm-test-binaries-03.json` records the frozen Mesocosm library binary
SHA-256 `a1ebfa9581e09d0141b986b9724d3a70994a7e3d4ff94e3e8614c29619338b95`
and WorldSave integration binary SHA-256
`d65f332606156f0a85e5923f9072ee2fef8972f0e4620b9230b70a61f1182f87`.
The five product GPU passes are the pending-scene publication control, H6
pixel/pick comparison, lit-body composition/skip, opaque chrome comparison
and actual present-device borrowing. The pending-scene control opens two
genuinely admitted native windows at different sites before capturing its
save, keeps A's queries/site/status while B is staged, promotes only B's
frozen input, and verifies save/replay preservation.

The three Isometer-only GPU controls are included in its executed 73-test
gate; they are separate from Mesocosm's five actual-device passes in R3-03.
Those five product GPU controls retain their source-identical original
passing evidence. Generation-07 re-executes the staged scene control against
the final suppliers, as well as the full default library and WorldSave gates;
it does not relabel the other four old passes as fresh generation-07 runs.
Counts above describe separate gates and overlapping source generations;
they are not added into a unique-test total.

Final frozen binary identities are retained in
`rootstock-scheduling-07-binary.json`, `genet-native-07-binary.json`,
`isometer-final-binary.json`, `a11y-focused-01-binary.json` and
`mesocosm-final-binaries-07.json`:

| Frozen executable | SHA-256 |
| --- | --- |
| `bin/rootstock-scheduling-07` | `75291f880774db1cba44c19709f02b20c9f8d6d912527815128776ab11216a0e` |
| `bin/genet-native-07` | `7e3b8df7c7b0566e654b5d5876f65c272d68e67b4378709b5c7d2f57e65df25e` |
| `bin/isometer-final-06` | `8a7dc06d94a7cec3e330540c99c1ecd763e267457281e41f15a20d7f17591fef` |
| `bin/a11y-focused-01` | `b28168e587d995e5a4d6d1605528dc60ea1d085acec81c56a045fe108c13508a` |
| `bin/mesocosm-receiving-final-07` | `7ba9714607bd7575cd9d1be1ce1c1f8f5a68a53997a83186d69cfb234e224a0d` |
| `bin/mesocosm-genet-tests-07` | `1f058288873085e45e4da5402456827daaa33ea3556c32a6bbe37f98f9cc6734` |
| `bin/mesocosm-world-save-tests-07` | `060f3f4d70f727b38e72e0865bfcdacb50a72e7a8dea5d72ca516ddf94da8375` |

The original standalone accessibility attempt, `r3-a11y-final-06.log`, did
not qualify its tests: an unused library-unit dev dependency, `uxtree`,
pulled `inker` and duplicate registry/Git `document-session-api` types from
an unqualified cached Genet cohort. No production-source repair is inferred
from that failure. The sibling `overlays/r3-a11y-focused-01/` preserves all
seven Rust source files, production dependencies, features and the original
SDK lock seed; its sole manifest adjustment omits the dev-only `uxtree`
table for `--lib`. Its complete manifest difference and unused-reference
proof are retained beside `preparation-proof.json`. The integration test
and example that use `uxtree` remain present and are outside this gate.

The focused overlay manifest has SHA-256
`2884e688aac502d700689f033442bd4256d33d110d12a407a6d4bc4ae40bd765`.
The offline resolution, fixed-source guard and parent admission are retained
in `r3-a11y-focused-resolve-01.log` and
`focused-a11y-parent-admission.json`. The latter names only the fixed-version
AccessKit Winit fixture and its target-specific iOS registry closure as
admitted additions; iOS is not compiled by this Mac test. The final focused
lock SHA-256 is
`c64a888c5042c2ce706c5bdf063c9047fe0833af5d09f474529ded9273c5c8d5`.
The SDK's Genet core remains `b7c0bef`; the failed attempt's cached moving
`66af` cohort and the owner root's separate `c0144` runtime records are not
adopted. The 23 unit passes establish the bounded bootstrap/consumer
fixture, not its omitted integration/example graph or physical OS
accessibility behavior.

### Received contracts and their limits

Rootstock receives Genet's existing native submission/completion/accepted
lifecycle through `Surface::shared_core`. One prepared packet owns its
paint, producer staging, capture requests, frozen observations and input
geometry while newer desired work coalesces. Publication rechecks source
guards, device/context, extent and scale before queue presentation. Replaced
sources, hidden/resize/loss states and shader refusal cannot commit the
packet. `presentation/native.rs`, `presentation/publication.rs` and
`producer/native.rs` in the copied Rootstock package own those seams.

`PresentationDriver::supports_native_receiving` defaults to false. Exposing
a native surface does not authorize bypassing a custom driver's policy.
The ordinary `WgpuDriver` opts into the common route and receives an opaque,
host-constructed `NativePresentedImage` with the accepted stamp, retaining
that same output for its own branded captures. It does not render a second
image or capture newer source state. The API and receiving implementation are
in Rootstock's `presentation/{frame,wgpu_driver}.rs`; portable/custom paths
retain their declared presentation route.

Accepted A and staged B have separate public producer metadata and query
ownership. Rootstock's `owned_layout/presented_input.rs` and
`input/presented.rs` retain A's hit/local/producer geometry and displayed
input scale; a removed displayed node is refused rather than retargeted to
B's node underneath. Geometry-dependent caret, drag, scroll and IME work is
withheld when the live layout/text cannot prove it still describes A.
`frame.rs::sync_ime_area` uses that displayed-layout proof. These source and
CPU-fixture controls do not certify physical IME placement or OS input timing.

Isometer's `query/snapshot.rs` owns body/implicit-surface/terrain queries,
camera and extent for the prepared image. Mesocosm's
`borrowed_section/producer/{staging,queries}.rs` keeps accepted geometry,
action revision and site together, including terrain picks. Its
`app/assembly/presentation.rs` freezes the committed subject, picture and
continuation reading before acceptance; it does not re-read mutable B state
when A completes. Retained-image publication can promote a captured newer
action context without another encode/image generation. A separate source
lifetime revokes reused numeric revisions/IDs. Initial query capture still
deep-copies CPU map and mesh data; only later snapshot clones share `Arc`.
Its initial copy cost belongs in submission CPU work, with no zero-copy or
speedup claim. VTT/Eponym and other legacy sources still need their R4
transactional adapters and declared fixtures.

### Completion scheduling and shared-Core availability

The first headed receiving attempt exposed a scheduling failure after the
basic receiving control already passed. It repeatedly turned an incomplete
candidate into another redraw, consuming cost samples while still accepting
logical pictures. Generation-07 separates three causes of work: application
wake, native completion, and shared-Core availability. These causes no longer
share a hook or a polling redraw loop.

Rootstock's `presentation/native/wake.rs` drains a native completion notice
directly into collection of an existing candidate. An OS redraw while a
candidate or Core waiter exists only records newer desired demand. It runs
no prepare or application/frame hooks and schedules no repeated redraw.
Winit's `redraw_turn.rs` applies each receiving window's queued visibility,
close, extent and scale commands before checking its packet, then runs the
same post-attempt accessibility/scenario epilogue for a collection attempt.
Application-owned wakes still run `AfterWake` independently. All receiving
windows are visited, including a window other than the app-work drainer.

When another host holds the Core, Genet's
`RenderCore::notify_native_frame_available` registers a one-shot waiter under
the same CPU mutex that retires native admission. This closes the Busy-to-idle
registration race without a timer or repeated GPU poll. Readiness is sticky
but grants no reservation: a notified host must retry normal admission. The
waiter holds only a weak gate reference; dropping it unregisters without
retaining Core/GPU state. Retirement marks all readiness bits before waking
outside the lock and GPU scopes, isolating a panicking callback. Already idle
or terminally unavailable state notifies immediately.

Rootstock retains the token until retry/cancellation, and only its readiness
can request the previously owed picture. An orphan completion, closed/hidden
host or unavailable surface creates no new picture demand. A reconfigured
surface requests one retry without publishing the invalidated packet.
`r3-rootstock-scheduling-07.log` executes the CPU wake separation controls,
two-host repeated collection/rearming, pending redraw coalescing and actual
unavailable/reconfigured/close controls. `r3-genet-native-07.log` executes
seven CPU availability controls and the actual busy-gate retirement control
alongside the existing native batch suite.

### Bounded headed runs and preserved failed evidence

`headed/receiving-hopped-retained-01/` remains a failed run: exit 1,
`ok:false`, 21,221 redraw attempts, 20,872 unpresented attempts and 349
accepted logical frames. Its reported error is `cost receipt exceeds 4096
samples; split the run into bounded phases`. It is not a successful timing
qualification, even though it reaches the same native world and accepts
pictures. The recorded failure motivated the scheduling/availability repair;
the original receipt, PNGs and cost samples remain intact.

All four subsequent `headed/*-02/` workload runs complete with exit 0,
`ok:true`, no watchdog, an unchanged frozen executable, no product errors and
no cost-budget overrun:

| Workload/backend | Logical frames | Redraw attempts | Unpresented attempts | Retained phase samples |
| --- | --- | --- | --- | --- |
| `baseline-retained-02` | 350 | 352 | 2 | 266 |
| `receiving-retained-02` | 349 | 700 | 351 | 532 |
| `baseline-components-02` | 350 | 353 | 3 | 266 |
| `receiving-components-02` | 349 | 699 | 350 | 532 |

Each receiving workload has 266 eligible phase pictures: 62 paused, 62
inspected, 21 camera, 40 inspector interaction, 19 native-step and 62
post-step pictures. Its 532 phase samples retain both submission/collection
attempts rather than disguising them as 532 logical frames. The whole-run
attempt counts also include work outside those bounded phases. The retained
receiving run has 350 native-pending attempts, the component receiving run
349. The baseline's older observer schema reports callback frames without
the new per-sample accepted-publication proof; cross-schema comparisons must
retain that qualification.

All four world artifacts are byte-identical, SHA-256
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`,
and all four use scenario SHA-256
`fbc18d99507d154a91b40d0131a78e045d5540eb801f7e7bce92790188f82a05`.
The older baseline executable is the frozen R2 binary, SHA-256
`b8c15514c82df0312f4274e191d1d66c2e0dba645bcdb0e55b4da75d2b4767a0`;
receiving uses the generation-07 executable identified above. Their
`command.json`, `exit.json`, `run.json`, timing, PNG and WorldSave artifacts
remain under each workload directory. The raw-derived comparison is retained in
`final-experiment-summary.json`, SHA-256
`995dbb68409c5d7b557bd6ffaf3f5b9974443561b9769b51f25af038a807d8f1`.
The per-backend artifacts are `retained-final-comparison.json`, SHA-256
`7e43cc07db9128a3d4d7fab52545f8c9b51f184aa4ddb35d83fc62b95ab51d89`,
and `components-final-comparison.json`, SHA-256
`235ea63022712ef403abbc92230fc14420c12980e65a866474937f4ebf691bf8`.

The headline is **paired profiled UI portions per logical picture**, including both
submission and collection accessibility passes. For all 266 receiving
pictures per backend, an independent raw check requires exactly one pending
submission followed by its adjacent accepted collection, with equal
`native_submit_us` and no leftover sample. Summing both `ui_attempt_us`
values equals accepted `total_us` plus the submission's `a11y_us` exactly.
The roughly 14.65 ms median accepted logical profile alone is incomplete for
this comparison. Per-picture work sums two UI turns; it is not a continuous
input latency or a physical refresh deadline.
The later R5 boundary audit clarifies this historical label: these fields
omit pre-admission, sealing/dispatch and outer callback bookkeeping. The
numbers remain paired profiled portions, not complete callbacks or the whole
event loop. R5 additionally leaves standalone/coalesced action-service cost
unavailable unless measured separately.

| Paired profiled UI portions per picture | Median | p95 | p99 | Strictly over 16.7 ms |
| --- | ---: | ---: | ---: | ---: |
| Retained R2 baseline | 16.371 ms | 37.846 ms | 47.781 ms | 26/266 |
| Retained R3 receiving | 16.3245 ms | 17.671 ms | 23.650 ms | 19/266 |
| Components R2 baseline | 16.376 ms | 36.186 ms | 47.306 ms | 23/266 |
| Components R3 receiving | 16.310 ms | 17.385 ms | 25.061 ms | 19/266 |

Timed-phase UI sums are 4.406741→4.054659 s for retained rows and
4.430038→4.052038 s for components. Strict exact-60-Hz interval counts
(`3*duration_us > 50000`) are 27→21 and 26→21 respectively. These are CPU
work budget counts, not physical missed-refresh observations. The new
accepted collection turns themselves all stay below 16.7 ms; eleven of 532
raw turns per backend exceed it on submission. Reporting those separate-turn
counts does not replace the 19/266 per-picture results above.

End-to-end native preparation-to-publication wall p99 remains 50.413 ms for
retained rows and 51.959 ms for components. The old binary has no equivalent
explicit wall-latency field. Callback interval p95/p99 changes
43.354/61.691→47.500/62.298 ms for retained rows and
43.001/55.236→48.308/64.033 ms for components. Those callback intervals
include scheduling/pacing and do not show a consistent-60-Hz result.
The worker qualification/queue, acquisition and scheduling contribution to
the native wall tail needs attribution; none of these fields is GPU duration.

Both scene crops `[24,120,2004,1500)` are decoded-RGBA exact, SHA-256
`cc64b84ceda2fddfe65233b8db7c5a86dd6938a59b8b9d7cfb26fcc63a56eb32`.
Each complete 2560×1600 image differs at one UI pixel outside that crop:
retained at `(273,40)`, components at `(2079,188)`, one RGB channel by one
8-bit level, with no alpha difference. Full-image equality is not claimed,
and the cause of the UI-edge difference is unproved. Native save, subject,
part and scenario agreement does not imply physical ecology/gait acceptance.

R3's lifecycle and bounded receiving fixture are locally qualified. These
results leave consistent 16.7 ms work, changed-frame wall tails, other
hardware/platforms and physical display/input qualification open.

### Timing and remaining qualification

Rootstock's `FrameProfile` and Mesquite's `cost.rs` distinguish redraw
attempts from logical accepted pictures. Raw records retain failed,
missing-profile and native-pending attempts. Eligible presented-frame
distributions exclude those attempts and capture frames. For an accepted
picture, `total_us` is its submission plus collection/publication time on
the UI thread; `native_latency_us` is preparation-to-presentation wall time,
including asynchronous qualification. Host CPU timings here are callback
wall-time spans, not process CPU-usage accounting; acquisition can still
wait. Neither quantity is a GPU-duration measurement.

All-attempt UI accounting counts a submission once: pending attempt
`total_us`, accepted `native_collect_us` plus the host's accessibility tail,
and legacy `total_us`. Structural counters can repeat on an accepted
profile, so all-attempt work totals sum only unique product cumulative
redraw/upload deltas. The qualified `cost.rs` includes CPU controls for this
distinction and for an all-pending phase refusing presented-frame eligibility.

On the opted-in native receiving path, changed-frame qualification is moved
off the redraw thread. This is not a claim that startup, device boot/recovery,
legacy/custom presentation or diagnostics never block.
Device/renderer initialization and explicit recovery retain their own work.
Mesocosm's opt-in `MESOCOSM_CAPTURE_MOTION` readback now runs only after
publication, but it remains a blocking GPU/file diagnostic and must stay off
ordinary performance fixtures. Retained source controls do not establish
GPU throughput, heap/allocation totals, old/new hardware coverage, physical
60 Hz, input-to-display latency or physical refresh deadlines. Queue
presentation is not compositor/display acknowledgement. Ecology, biological
rules, native clocks, attention admission and simulation replay authority
are unchanged by this presentation receiving pass.

### Follow-ons identified at the R3 cut

R3 identified duplicate submission/collection AT work, the approximately
50 ms native wall tail and initial query-copy cost. R5 below qualifies the AT
split and bounded timing attribution; it records the component saving and
absence of an overall profiled-duration improvement. R6 retains worker,
preseal and acquisition-pacing attribution. QuerySnapshot COW, R4 receivers
and conditional Didgeridoo bindings remain separately declared work.

### Preservation and review boundary

`canonical-preservation-audit.json` is a pre-final-doc audit against R3's
initial preservation snapshots. Mere, Genet and Isocosm retain their original
HEADs, prior source/dirty/untracked bytes and indexes apart from the listed
permitted task plan/index edits. No new canonical dirty/untracked path is
introduced at that cut. The overall preservation flag is **false**, solely
because canonical Netrender's clean checkout independently advances from
`72f54a71531f44335377ec21dec950cfd2b61def` to
`6364e5d9df6453af72e6742a63807ed74d4852c8` through two commits and twelve
tracked paths, including manifests and lock. This audit makes no attribution
and performs no reset/recovery. Exact committed deltas and before/after
manifest/lock bytes are retained under
`canonical-history/netrender-pre-final-doc-01/`.

That canonical advance is not adopted into the qualification. The isolated
Netrender owner stays at `72f54a71531f44335377ec21dec950cfd2b61def`, and the
original SDK manifest/lock hashes remain unchanged. Each combined R2/R3
review patch is based on its recorded isolated owner HEAD. The index
`review-patches/combined-r2-r3-pre-final-doc-01/index.json` identifies scoped
source/doc patches, exact owned new files, full dirty preservation and the
separate Isocosm qualification manifest/lock deltas. It does not reclassify
pre-existing ecology work or local qualification overlays as supplier
adoption. All four snapshots are stable and the read-only patch parser
coverage checks pass; no patch is applied, staged or committed. Later final
document updates may require a later audit cut.

## 2026-11-04 — R5 accessibility work and native wall tails — bounded qualification

**Status:** 2026-11-04: R5a's accessibility publication/action separation and
R5b's bounded native wall-tail attribution are locally qualified on the
fixed SDK. Both genuine before controls fail as expected, final owner and
receiving gates pass, and all four matched headed runs complete. AT's
profiled portions are lower; paired profiled UI duration does not improve
overall in this fixture. Consistent 60 Hz, deeper worker/preseal/acquisition
attribution, physical accessibility/display and broader hardware/platform
qualification remain open. R2/R3 qualification is retained.

All short artifact paths in this R5 section are relative to
`Code/output/mesocosm-scene-composition/2026-11-04/native-a11y-tails/`.
`final-status.json` is the concise evidence index, SHA-256
`ac1562154d0059bb4b2b1a32df8a4169d56e0e1986ddaab0f900bf3ad513e6ee`.

The owner plan is
`mere/design_docs/cambium_docs/implementation_strategy/2026-11-04_qualified_raster_retention_plan.md`,
R5a/R5b. Isocosm's product reference is C6 in
`isocosm/mesocosm/design_docs/2026-11-04_scene_composition_plan.md`.
This follow-on does not reopen the locally qualified R3 lifecycle or imply a
new supplier adoption. Detailed prior evidence remains in sibling
`native-frame-receiving/`, including `final-status.json` and
`final-experiment-summary.json`; original R2 evidence is unchanged.

### Preserved before arm

Use the frozen qualified R3-07 executable as the before arm, rather than
calling the earlier R2 binary the new baseline. Its SHA-256 is
`7ba9714607bd7575cd9d1be1ce1c1f8f5a68a53997a83186d69cfb234e224a0d`.
The associated overlay manifest SHA-256 is
`bf22b046dc54525927bb9e89d34626c00cbd65dae01cfbe5a942c30d6f8a6249`.
The prior raw-derived performance summary SHA-256 is
`995dbb68409c5d7b557bd6ffaf3f5b9974443561b9769b51f25af038a807d8f1`.
These identify prior evidence, not new R5 measurements.

The R3 fixture has six declared phases and 266 eligible pictures per backend.
Its profiled per-picture UI portions include submission and collection turns;
the
submission AT pass adds about 1.6 ms mean work per picture. Its native wall
p99 is approximately 50 ms and is not GPU duration. R5 measures publication/action separation and attributes the remaining tail.
The results below support the bounded split and attribution, without an
overall profiled-duration improvement or a tail-optimization claim.

`planning-01/preimages.json` preserves the four plan preimages.
`planning-01/plan-change-manifest.json` identifies the exact adopted planning
text, and `planning-01/source-assumptions.json` records the read-only source
inspection. The parent-owned eight-checkout initial preservation is under
`preservation/initial/index.json`. At that initial cut canonical Netrender
is at `6364e5d9d` with one preserved dirty WebGL path. The final source review
observes clean `b14e8452b`: its sole committed path has exactly those initial
working bytes. Thus HEAD/status changes while prior working bytes and
manifest/lock stay preserved. No attribution is inferred. The fixed SDK and
isolated qualified Netrender owner do not adopt that canonical advance.

### R5a source boundary and preserved contract

At the planning cut, Winit's `redraw_turn.rs::finish_redraw_attempt` calls
`sync_a11y` after both preparation/submission and collection. Rootstock's
`frame.rs::sync_a11y` combines publication gating, semantic projection and
typed request dispatch. The native adapter's `sync_inner` drains queued
actions through the prior published namespace before rebuilding/replacing
it. Root-only native bootstrap installs before revealing a hidden window;
its later-pending path preserves accepted routes. Those seams motivate the
split; source inspection is not executed qualification.

The received split lets actions drain without requiring a new tree or picture.
A request uses the publication it could have observed, retaining host and
contributor lifetime, typed action, focus/value, order and native admission
checks. Removed or stale accepted A targets refuse rather than retarget B.
Draining may create later render demand, but does not publish staged B or
claim another tree submission. An accepted semantic state publishes
coherently with its owned input/site/scale facts. Keep custom/legacy adapters
and explicit synchronous callers within their existing contracts.

### Executed gates and matched comparison

| Gate and retained log | Executed result | Scope |
| --- | --- | --- |
| `pre-a11y-tail-01.log` | 0 passed, 1 failed, 0 ignored, 239 filtered; exit 101, expected | Actual native-batch redundant-publication witness: 2 projections versus 1. |
| `pre-event-route-01.log` | 0 passed, 2 failed, 0 ignored, 21 filtered; exit 101, expected | Actual Winit CPU event path leaves the app at 0 instead of 1 and fails to consume hidden/closing queued gestures. |
| `genet-timing-tests-01.log` | 25 passed, 0 failed, 0 ignored, 24 filtered; exit 0 | Native batch/availability controls and three new timing CPU controls, including actual-device timing transport. |
| `rootstock-r5-01.log` | 246 passed, 0 failed, 0 ignored, 5 filtered; exit 0 | Intermediate generation 01 before idle-event integration correction. |
| `rootstock-r5-02.log` | 246 passed, 0 failed, 0 ignored, 5 filtered; exit 0 | Final optimized `--include-ignored --skip owned_layout::incremental_tests --test-threads=1`; five unchanged layout controls passed in R3 full-05. |
| `winit-r5-02.log` | 23 passed, 0 failed, 0 ignored, 0 filtered; exit 0 | Final Winit library gate, including both corrected actual `wake_turn` CPU controls. |
| `a11y-test-02.log` | 24 passed, 0 failed, 0 ignored, 0 filtered; exit 0 | Focused native adapter library unit gate; same source/features/production dependencies, unused `uxtree` dev table omitted for `--lib`. Integration/example and physical-reader qualification excluded. |
| `mesquite-test-02.log` | 49 passed, 0 failed, 0 ignored, 0 filtered; exit 0 | Final cost, pairing and scenario library controls, including timing availability and UI-accounting controls. |
| `receiving-runs/mesocosm-compile-02/output.log` | Build exit 0 | Final receiving compilation: manifest unchanged, fourteen selected Git/local-overlay record changes and zero Git identities outside the original fixed SDK. |
| `runtime/mesocosm-default-02/output.log` | 78 passed, 0 failed, 6 ignored, 0 filtered; exit 0 | Final default Mesocosm library gate; actual-device cases and CPU component diagnostic remain explicit ignores. |
| `runtime/mesocosm-staging-gpu-02/output.log` | 1 passed, 0 failed, 0 ignored, 83 filtered; exit 0 | Explicit final actual-device staged scene/query publication control. |
| `runtime/mesocosm-world-save-02/output.log` | 1 passed, 0 failed, 0 ignored, 0 filtered; exit 0 | Final native WorldSave load/watch-continuation integration control. |
| Matched retained/component before/after runs | Four exit-0, successful receipts; no watchdog or sample overrun | Same-session A-retained/B-retained/B-components/A-components order, 266 eligible pictures and 532 profiled attempt rows per run. |


The common test is
`native_a11y_tail_control::pending_submission_does_not_reproject_the_prior_accessibility_tree`.
The frozen R3-07 pre-overlay changes only its test module/export; the test body
is byte-identical. The sole API compatibility adjustment removes the new
fake adapter's drain override from the old trait implementation. Its prior
accepted image is explicitly a controlled CPU premise; the pending candidate
uses the actual native batch, without a fabricated swapchain/display claim.
`pre-fix-control-binary.json` identifies SHA-256
`ca3493925a89f597ee3e395d3e355ecb7657ae75aa85e60c3b77fbd293e96b10`.

`genet-binary.json` identifies the 25-test executable SHA-256
`2fd0bd0ec235ee881ca6a3f5dad96fd566d6cc46aeacba70ed1c83a00a5329c5`.
`rootstock-r5-01-binary.json` identifies SHA-256
`70c8bed5d0586599e83db9b76845a35ae0ad47ab89afb4554d29017edf39747f`.
Generation-01 overlay manifest SHA-256 is
`36fc1a1ac9bb02a53e349ef37342d944c5baaef52c5f175b75e8cdb42c0c9ca4`.
Generation-02 overlay manifest SHA-256 is
`0e318a1f2b502715587cb2512ef6dde7842c004222cec6b497e6c7f4f01afe55`.
The frozen event-route before control is `bin/event-route-pre-fix-01`,
SHA-256 `6862db4eb45ea0ad544b4054040ca84b1159f8f19b2499f7cab2529b5dc68a10`.
Its overlay derives from generation 01 and changes only the Winit test
module/export. The two-test source is byte-identical to generation 02,
SHA-256 `1cf0092563c0d961f6a51f0427e0ab3e19673c4b5f429930d0b9257a55bc05bd`,
with no API compatibility edit. It is a CPU event-source control, without a
native window, GPU or physical AT-delivery claim.

| Final frozen gate executable | SHA-256 |
| --- | --- |
| `bin/rootstock-r5-02` | `6701347facc248f78fab558b2a781df5334735fd7d8a361a5fed1bdcf0689993` |
| `bin/winit-r5-02` | `67bbd30520b4443dfd43ab90574fa7d5bfdb73af6e0b7be5421ea9a31b29975a` |
| `bin/a11y-test-02` | `6493e5808fd3c59a53b46f8da5226ea6c0bc028742e972cf56b85765530dbc81` |
| `bin/mesquite-test-02` | `6f6f93b40f85db0409287ae4ad794297fd1f741eb41b1c9082746053f7f47306` |

The receiving lock SHA-256 is
`1ecb545b99546caee76d1251239f99350d84f978a5144513805654d7d82a5ba9`.
`mesocosm-final-binaries-02.json` binds the source/receiving receipts and
frozen main executable `bin/mesocosm-a11y-tails-02`, SHA-256
`a39e7d9893c73b2f9b229dc75b927ead2d3054ecab1a6149b96510b1fc05b897`.
Its library test SHA-256 is
`f87a6d137b41d39be23e624ab863ace1ee4e493ec212bbc70e8c20760f081e8a`,
and WorldSave test SHA-256 is
`d9f6002f67b3525ed5f25c35a187699a5131ac0b0adbbc4c9832376493cfb5ca`.

The focused accessibility manifest retains the earlier bounded `uxtree`
unit-fixture adjustment; its source manifest SHA-256 is
`8542662bbe17a57d005bcca7bd4154a5ac41d579e0bdcff35618970416bf1e09`.
Only that declared `--lib` fixture is qualified, not the omitted dev graph
or an OS screen reader. Generations and overlapping gates stay separate;
counts are not added into a unique-test total. Owner/receiving correctness and the separately completed bounded headed
comparison do not establish physical display or general platform acceptance.

### Source review and corrected event-source boundary

The optional `Accessibility::drain_actions` uses prior published bindings;
its default `None` preserves the legacy combined fallback. Rootstock's
extracted `frame/accessibility.rs` retains per-request host/document and
contributor checks, current semantic visibility/disabled/action checks and
revalidation after each earlier callback. Opted-in hidden/closing/unavailable
hosts consume then reject gestures, so they cannot replay after reveal.
Bootstrap retains its root-only install-before-show/failure fallback.

Review found a generation-01 integration gap: the otherwise idle AT wake
still requested a redraw, causing a new candidate to prepare before action
service. Generation 02 adds `Host::take_a11y_wake` and consumes it in Winit's
per-window `presentation_wake_turn` after queued visibility/close commands,
before geometry refresh, native collection or new preparation. Actual model
changes create ordinary frame demand; duplicate notices and empty drains do
not. New CPU tests call the real Winit `wake_turn` with a bound published
button and assert no Frame/AfterFrame/AfterWake. That source has no further
concrete review finding. Both event-source controls now fail on the old
source and pass in the final 23-test Winit gate. CPU routing does not certify
OS reader delivery.

Native timing follows the same Job→Completion→AcceptedNativeFrame, with
scheduling, worker entry, qualified readiness and successful channel receipt
captured monotonically. Rootstock partitions preparation through publication
with cumulative rounded boundaries; the five intervals sum exactly to native
latency. The endpoint is before outer Winit AT work. Unavailable partitions
remain unavailable, including on the R3 baseline; zero-length measured spans
remain available. Worker polling/scopes are qualification wall time, not GPU
duration; old overlapping UI stage clocks are not added to that partition.

`review-deltas/r5-final-02/source-hash-manifest.json`, SHA-256
`105e4aac2a716a8941ecee3f9ee9694d35904ebdc86e7383804d6d6f926565d5`,
retains the final source-review cut: 19 admitted Mere source deltas, six
Genet deltas, no Isocosm product/isolated-Netrender source change, separate
qualification/doc patches, all eight stable preservation snapshots, and
verified unchanged R3 source/executable inputs. It verifies 366 final
source/assets/declared normalized manifests and 355 frozen R3 inputs.
Prior captured current bytes supply preimages first; newly dirty tracked
files use the initial HEAD, and new untracked files have absent preimages.
Source identity and review do not establish a passing runtime gate.

### Comparison and accounting contract

The four completed runs use the same paused native bench and six phases for
retained rows and the component projection under the fixed SDK. Scene, subject,
part, scenario, starting tick and admitted manual steps are matched; environment
and session metadata and ambient-load observations remain in each command. The prior expected WorldSave SHA-256 is
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`;
all four actual saves verify against that digest and their native state/history
checks. This is the coarse C4 fixture with zero physical members/poses, not
embodied ecology or gait qualification.

Bind attempts to their logical picture using actual submission/collection
identity and preserve orphan/refused/unpresented/missing-profile work.
The headline is the profiled UI portions of submission/poll/collection per
picture, including AT work inside those recorded spans. Report action-drain and
tree-publication work separately where observable and preserve their inclusive
AT total. Standalone/coalesced action-service turns run outside AfterFrame;
report their counts/time separately if measured, otherwise explicitly
unavailable, never zero. The native submit profile starts inside preparation: pre-admission work,
sealing/dispatch overhead and outer callback bookkeeping are not covered.
The ledger is neither complete callback work nor whole-event-loop work.
Report native wall latency, callback cadence, p50/p95/p99, strict `>16700us`
and exact-60-Hz CPU budget counts. Include raw attempt and
accepted-picture counts, scene/extraction/upload counters, source/binary
hashes, native saves and decoded full/crop pixel differences. A shorter
accepted callback cannot stand in for lower paired profiled work.

Preparation-to-publication wall time includes asynchronous qualification,
queue/scheduling and acquisition effects. Distinguish those boundaries before
attributing a tail, and do not add overlapping spans. Do not label wall time
GPU duration or callback cadence physical display deadlines. The measurements below retain the absence of an overall profiled-duration
improvement rather than treating the AT reduction alone as a speedup.

### Matched headed measurements

`native-tail-comparison-02.json`, SHA-256
`d227d89ca5f88379838a94366671a8df3eb9a7046429308883469eedd28f399b`,
contains the source/binary bindings, all raw-derived phase and paired-picture
statistics, exact new-only timing partitions, native saves, pixels and
ambient observations.
The compact `final-experiment-summary-02.json`, SHA-256
`b644417051277a455e63ed1a94e5dec5b150320dfd47f6006023389120711986`,
provides the concise result/reference/ambient comparison. All fresh A/B final
fields agree. Against historical R3, native/count/subject/stamp readings and
WorldSave bytes agree, while the presentation counter is 19 successful scene
draws in each fresh run versus 18 historically. Full historical field/image
identity is not claimed. `retained-comparison-02.json` and
`components-comparison-02.json` retain the ordinary comparison exports.
`plans/r5-final-02.status.json` records all four terminal runs. The run
commands and raw artifacts are under `headed/r5-final-02-*`.

The independent direct raw check in `independent-raw-check-02.json`, SHA-256
`6e5d4ab42034960651fef8233a7361eabf11e55755b94b88bf08265370dc59b8`,
checks adjacent submit/collect pairing, repeated submission identity, all
candidate timing sums, AT subsets and saved-world hashes without using the
exporter's arithmetic helpers. Each run has 266 eligible pictures and 532
profiled attempt rows. Each candidate has 266 exact native partitions and
532 rows whose publication/action subfields remain included in the AT total.
No failed/orphan attempt is removed to manufacture those sums.

There is **no overall paired profiled UI-duration improvement** in this
observation. AT's profiled sum is lower by about 44%, while acquisition
waiting grows by a similar amount. These are wall-span observations, not
process CPU-usage measurements or a controlled causal estimate.

| Paired profiled UI portions per picture | Median | p95 | p99 | Strictly over 16.7 ms | Sum over 266 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Retained R3 baseline | 16.270 ms | 16.975 ms | 22.960 ms | 16 | 4.015062 s |
| Retained R5 receiving | 16.2715 ms | 16.590 ms | 26.529 ms | 12 | 4.039222 s |
| Components R3 baseline | 16.2695 ms | 17.521 ms | 26.695 ms | 23 | 4.028974 s |
| Components R5 receiving | 16.2525 ms | 17.891 ms | 26.058 ms | 18 | 4.064153 s |

The summed changes are +0.60% for retained rows and +0.87% for components.
Exact-60-Hz profiled-budget counts (`3*duration_us > 50000`) are 17→12 and
24→19; neither set is physical missed-refresh evidence. AT paired sums are
874,008→490,681 µs and 892,215→498,830 µs. Their mean profiled portions per
picture are 3.285744→1.844665 ms and 3.354192→1.875301 ms. Steady acquisition
means move 8.8164→10.2648 ms and 8.7460→10.2789 ms; medians move
10.864→12.3995 ms and 10.7445→12.2975 ms. That opposing waiting duration
prevents an overall elapsed-profile improvement in these runs; it does not
prove a causal AT-to-acquisition relationship.

#### What the new native markers establish

Only R5 has the five-part native partition. The R3 before arm has no such
markers; its missing values remain unavailable, never zero. The partition
ends before the outer AT call, and all other stage spans can overlap it.
Quantiles from different fields must not be added into a synthetic packet.

| New-only native wall interval | Retained | Components |
| --- | ---: | ---: |
| Queue median | 0.014 ms | 0.014 ms |
| Queue maximum | 0.060 ms | 0.785 ms |
| Worker qualification p95 / p99 | 27.005 / 30.327 ms | 27.345 / 30.073 ms |
| Ready-to-collect median | 0.091 ms | 0.0895 ms |
| Ready-to-collect maximum | 0.192 ms | 0.247 ms |
| Post-collection median | 12.9525 ms | 12.8755 ms |

Worker wall time exceeds 16.7 ms for 15 pictures in each backend: four camera,
eight inspector-interaction and three native-step pictures, with none in the
steady picture phases. In this fixture the tens-of-milliseconds tail is in
checked worker qualification, not notification delivery. Worker time includes
polling, error scopes and quality/readback work; it does not isolate driver,
GPU execution, CPU or readback causes and is not GPU duration. End-to-end
native wall p99 remains 54.981 ms retained and 53.477 ms components.

For the same worst native-step packet, retained frame 256 has 61.490 ms =
30.274 preseal + 0.029 queue + 29.149 worker + 0.124 ready-to-collect +
1.914 post-collect. Components has 57.319 ms = 27.525 + 0.013 + 27.455 +
0.129 + 2.197 ms. These are the components of actual individual packets,
not a sum of unrelated quantiles. R6 needs worker/preseal attribution and
acquisition-pacing measurement while preserving checked qualification.

#### Native result, pixels and ambient conditions

All four native saves are byte-exact with the historical R3 digest given
above. Both scene crops `[24,120,2004,1500)` are exact, decoded RGBA SHA-256
`cc64b84ceda2fddfe65233b8db7c5a86dd6938a59b8b9d7cfb26fcc63a56eb32`.
The retained full 2560×1600 capture differs at `(2079,188)` by one RGB channel
level, with no alpha difference; the components full images are exact.
The cause of the isolated UI-edge difference is not established. Every arm
has the same 13 timed-phase scene redraws, 4,423,824 estimated mesh-upload
bytes and zero instance-upload bytes.

The protocol is one serial ABBA observation per source/backend on the same
hardware/session. Before/after load snapshots observe no compiler, but
ChromeTesting remains active at about one core of CPU and its GPU helper is
active; desktop helpers, Python and WindowServer also appear. This is not a
quiet-desktop or quiet-GPU fixture. One run per arm, material ambient load,
small per-phase tail samples and profiled-clock gaps limit causal or broader
performance conclusions. No claim of consistent 60 Hz follows.

### Limits and deferred work

The owner/receiving gates and bounded four-run comparison pass. R5a and R5b
are locally qualified; the measurements do not close the overall frame-time
target. Source remains isolated pending coordinated adoption. R4 VTT/Eponym adapters, other product
and hardware/platform fixtures, physical accessibility/input/display timing,
GPU duration and heap allocations retain their separate gates. C4 supplies no
queued-AT-request workload: action delivery is covered by the separate CPU
controls, not inferred from its ordinary pointer/keyboard scenarios. QuerySnapshot
initial body/map copying is a separate measured COW proposal, not part of an
unreported AT change. Ecology, biological rules, native clocks, attention
handbacks, authoritative storage and replay authority are unchanged.

## 2026-11-05 — R6 worker, preparation and acquisition attribution

R6 is implemented and locally qualified for the bounded fixed-SDK fixture.
Final owner/receiving gates and all four serial ABBA observations pass. This
qualifies the diagnostic transport and declared observations, with material
CPU coverage limits; it does not establish a complete CPU decomposition,
GPU duration, a causal speedup or consistent 60 Hz.

Evidence in this section is relative to
`Code/output/mesocosm-scene-composition/2026-11-04/native-worker-attribution/`.
`final-status.json` is the concise evidence index. Detailed analysis is
`native-worker-comparison-03.json`, SHA-256
`514a2c5d7aebf9dbf0b52120d048bf14d3eff8856c45b5fced3eab5fbbc0b005`;
`final-experiment-summary-03.json` is the compact export, SHA-256
`332ebbafd3015e86a222cfa69d708dff4fd977a3013f1da46630b5c5c30856e9`.
`independent-raw-check-03.json` independently checks raw packet identities,
saved submission provenance, wall/CPU availability, paired UI statistics and
all four native saves without importing the analysis implementation.

### Implementation and qualification identity

Netrender owns the shared same-thread CPU clock and provider observations;
Genet binds worker data to intrinsic batch/Core identities. Rootstock owns
synchronous preparation, capture, encode/seal and acquire/compose/present
scopes. Mesquite retains those facts alongside the existing profile ledger.
Current-thread POSIX runtime is tested on this macOS host. Other OS threads,
including driver threads, are outside that CPU counter. The reported 1ns
clock API resolution is not a physical-precision qualification.

Fresh owned Winit Cores enable CPU/quality diagnostics after boot; refusal
logs and cannot fail app boot. Borrowed/shared Cores are not reconfigured,
and missing owner-enabled observations stay unavailable. The additional GPU
feature request is default-off: `CAMBIUM_NATIVE_GPU_TIMING=1` adds optional
`TIMESTAMP_QUERY` under existing adapter/feature admission. Custom options
may already enable it; actual device features remain authority.

Only the SDK-selected `netrender-vello` at Vello revision
`276b46648c08f3d2062e70559afa999842e1873c` joins the isolated overlays. Its
quality hooks preserve the original poll, FIFO, retirement and typed-error
behavior. The optional GPU path brackets commands in the existing encoder
with its original single submission, excluding direct queue uploads and
later composition/presentation. It is not pure shader or whole-frame GPU
time. No new wgpu/core supplier cohort or unsafe Metal observer is adopted.

Final source is frozen in `overlays/r6-03/overlay-manifest.json`, SHA-256
`2403339879bd108933eefb5eca6fca2029e17618315190a48a7ddc9b6c5ef523`.
The candidate `bin/mesocosm-worker-attribution-03` SHA-256 is
`ea2eb26a0c2ee9163163a7084d82c6f3c1a08edcef49c397d616ec384e36c0c5`;
the before arm remains frozen R5 `bin/mesocosm-a11y-tails-02`, SHA-256
`a39e7d9893c73b2f9b229dc75b927ead2d3054ecab1a6149b96510b1fc05b897`.
The receiving graph has eight overlays/sixteen selected identity replacements,
zero new registry or foreign Git records and no common dependency-record
changes. Source audit verifies395 captured candidate inputs, 104 current
approved owner inputs and 366 frozen R5 inputs. This is local qualification,
not published supplier adoption.

| Final executed gate | Passed | Qualification details |
| --- | ---: | --- |
| Provider-01 |10| CPU metadata/quality controls, including corrected overflow history; no GPU-duration credit. |
| Net current-thread clock |3|84 filtered; exact availability/resolution rerun1 passes with86 filtered and overlaps this suite. |
| Genet native03 |55| Includes actual native pixels, validation and curve-quality refusal/recovery. Exact adapter inventory rerun1 passes with54 filtered and overlaps. |
| Rootstock03 |252| Five unchanged incremental controls filtered; prior full qualification covers them. |
| Winit03 |25| Includes GPU optional-request/default preservation. |
| Mesquite03 |51| Packet/current-call/availability and existing profile accounting. |
| Mesocosm default03 |78| Six ignored; explicit staging GPU control below executes separately. |
| Mesocosm staging03 |1| Actual-device staging/lifecycle;83 filtered. |
| WorldSave03 |1| Native save/replay preservation. |

All named final gates have zero failures. `validation-summary-03.json`,
SHA-256 `d2e7acefc01ded7f1e8fab7561a5825a62f8d97fa4a74611d5d912e0b8b08646`,
retains log hashes and qualification scope. The standalone provider resolves
one explicitly admitted inactive `wgpu-profiler 0.28.0` registry declaration;
its feature is disabled and no profiler is compiled. That addition is absent
from product receiving. Earlier resolver refusal, test-only constructor
compilation failure, locked-admission refusal and zero-test filter attempt
remain retained without passing-test credit.

The source review caught a GPU history marker that could expose an earlier
partial sum after a later overflow. The fixed inclusive expiration and its
sequence regression execute in the provider gate. This is a source-found
issue and a post-fix regression, not a claimed native pre-fix failing run.

Final03 also preserves R5's broad worker-ready endpoint, sampled after report
assembly/copies at completion construction and before channel publication.
The new CPU/wall observation ends earlier through the GPU metadata snapshot;
its excluded report finalization is bounded in wall time only, allowing for
cumulative microsecond rounding. Channel/wake/dispatch are not measured CPU,
and ready-to-collect is not pure notification delay. Historical r6-02's
shifted endpoint is superseded; its preimages and review remain retained.

### Bounded comparison and observed costs

The protocol is one A-retained/B-retained/B-components/A-components sequence
on the same paused C4 fixture and six phases. Every run exits 0 without the
watchdog. Each candidate has 266 accepted phase pictures and 532 retained
attempts, with 266 intrinsic packet identities and matching saved submission
facts. New CPU fields are unavailable in the R5 arms, never inferred zero.

| Paired recorded UI portions per picture | Median |p95|p99|Over16.7ms|Sum over266|
| --- | ---: | ---: | ---: | ---: | ---: |
| Fresh R5 retained |16.2445ms|16.899ms|24.951ms|17|4.031595s|
| R6 retained |16.2175ms|16.855ms|25.952ms|16|4.025358s|
| Fresh R5 components |16.2145ms|17.217ms|38.026ms|16|4.116756s|
| R6 components |16.193ms|16.682ms|25.593ms|13|4.036433s|

Observed sums change by−0.15% and−1.95%; these instrumented single-run
observations are not a causal optimization result. Exact60Hz profiled-budget
counts are17→16 and16→14. The component baseline includes an isolated74.402ms
worker-wall post-step packet. Do not generalize its absence in the candidate
into a source-caused tail improvement. Candidate native wall p99 remains
51.990ms/52.950ms. Thresholds are not physical missed-refresh measurements.

In each candidate,15 worker observations exceed16.7ms: camera4, inspector
interaction8 and native steps3. Their original `Device.poll(Wait)` calls
average27.414ms/27.723ms wall and0.331ms/0.307ms current-thread CPU. The
already-mapped quality range read/parse/unmap averages32.841µs/33.168µs CPU;
map request→callback wall averages27.555ms/27.854ms and overlaps progress.
This places the long call interval in bounded completion waiting rather than
mapped-counter CPU inspection. It does not identify GPU execution, other
thread driver work or the cause of descheduling/locking within those calls.

Across all 266 candidate acquisitions, means are10.502ms/10.554ms wall versus
56.904µs/57.063µs current-thread CPU. This is another off-CPU call residue,
not measured GPU time. Changed-picture preparation has real CPU cost: the
maximum producer spans reach17.976ms/18.744ms CPU. Mesocosm's producer scope
covers target setup, section/terrain/body configuration, drawing/submit and
accepted query capture. It does not isolate query copying, mesh work or
allocation individually. R7 should subdivide that producer/resource/query
scope before selecting copy-on-write storage or pooling.

### Material CPU and GPU availability limits

Worker wall partitions validate266/266 in both candidates, but complete
worker CPU partitions validate only17/16. The typed derived `other` CPU
exceeds its residual wall and is rejected in249/250 packets. Preparation
unscoped CPU is rejected266/266, submission unscoped264/266 and collection
unscoped255/256. Raw rejected counters remain present; valid CPU and residual
fields stay null. Whole observed intervals and named spans remain separate
valid measurements. There is **no complete UI or worker CPU decomposition**.

Nested sampling overhead has different coverage in wall and CPU windows and
is a source-grounded explanation to test for the derived-remainder problem;
it was not isolated experimentally. Fix measurement coverage before claiming
a complete decomposition. Do not clamp the counters or turn absent/no-call
classes into measured zeros. `quality_total`, `scope_wait` and map readiness
overlap other observations and cannot be added as independent work.

The sole enumerated adapter is AMD Radeon Pro Vega56 Metal and reports
`TIMESTAMP_QUERY:false`. Both candidates report `device_unsupported` for all
266 packets, with null GPU duration. Positive timestamp execution is
unqualified on this hardware. Actual-device pixel/quality tests and an API
feature path are not GPU-duration evidence. No canary substitutes for it.

### Native result, preservation and remaining qualification

All four saves are byte-exact with frozen R5, SHA-256
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`.
All eight final/mixed scene crops match frozen R5, decoded RGBA SHA-256
`cc64b84ceda2fddfe65233b8db7c5a86dd6938a59b8b9d7cfb26fcc63a56eb32`.
Fresh retained full images are exact; component full images differ at two
pixels by one channel level outside the scene crop, with no alpha difference.
Fresh pair final fields agree. Each run records13 timed scene redraws,
4,423,824 estimated mesh-upload bytes and zero instance-upload bytes. C4 has
zero physical native bodies; it does not qualify embodied ecology or gait.

The nine-repository source audit is
`review-deltas/r6-final-03/source-hash-manifest.json`, SHA-256
`33e30121f4509433054a5f5473a5ab48e4903640e27ed869a17a71d422ed3646`.
It uses the original eight-repository preservation plus the separately
retained clean exact-SDK provider witness. All new task source paths are
admitted. Canonical Netrender independently advances clean from b14e8452 to
9214a084 across a font-admission note and `vello_backends/sessions/text.rs`;
its manifests/lock remain unchanged. Committed before/after bytes and patch
are retained without attribution. Neither isolated Netrender nor the fixed
SDK adopts that advance; an all-canonical-HEADs-preserved claim is false.

Eight before/after ambient snapshots show no compiler, but Codex Renderer,
Python, WindowServer and desktop activity remain material, with load average
about2.17–4.18. Endpoint snapshots are not continuous load/GPU observations.
One observation per source/backend, instrumentation overhead and profile
scope gaps limit inference. Native UI call scopes exclude outer AT and
other callback/event-loop work; the paired profile accounts for its recorded
AT once and is not total process CPU. Standalone/coalesced action-only cost
remains outside that profile.

R6's bounded source/lifecycle/measurement qualification is complete with the
coverage limits above. Consistent60Hz, broader hardware/platform and physical
input/display/AT evidence remain open. R7 finer producer/resource/query
attribution precedes choosing a COW/pool optimization; complete CPU accounting
and a safe future GPU observability seam need separate work. R4 VTT/Eponym
receiving and conditional C5 Didgeridoo remain in their declared owners.
Ecology, rules, clocks, attention, authoritative storage and replay are
unchanged. No supplier commit/push or coordinated adoption is implied.

## 2026-11-05 — R7 producer, draw and immutable-query attribution

R7 is implemented and locally qualified for the bounded fixed-SDK fixture.
The new diagnostic controls, native resource/publication controls and all four
serial observations pass. The result identifies producer CPU targets; it
does not establish an overall speedup, a complete CPU decomposition, actual
GPU duration or consistent 60 Hz. No COW or target pool is implemented here.
Current-main integration is active separately under the owner plan's R8.

Evidence paths in this section are relative to
`Code/output/mesocosm-scene-composition/2026-11-04/producer-preparation/`.
`final-export-index-02.json` binds the final source, gates, execution policy
and measurements. Detailed `producer-preparation-comparison-02.json` has
SHA-256 `9d2aa84828e2a7936416df07f8cafaf0efafb4406875490fb42a47a783d24bd8`;
`final-experiment-summary-02.json` has SHA-256
`e6a1d209022682da60fc398b14a8af2d43f31de6c8566baa0cf52a8b4d71e6c7`.
`parent-raw-mechanism-check-02.json` independently differences adjacent raw
attempt counters without importing the exporter and confirms the named
CPU/call/resource results below.

### Source, transport and executed qualification

Mesocosm's product-local cumulative `main_scene` and `previews` costs carry
through optional `Product::cost_details` into Mesquite `Sample.product_work`.
The default remains `None`; the original observation API remains usable.
No Rootstock producer/registry API or simulation authority changes. A stable
cost epoch survives GPU retirement; source replacement starts a new epoch.
Each raw attempt retains its own immutable payload. Repeated cumulative
payload during collection is provenance, not another execution of its work.
The phase predecessor is retained, unavailable/missing/disappeared scopes
are explicit, and reset/regression cannot silently become a zero delta.

The five direct stages measure existing synchronous calls: explicit target
and section/configuration setup, scene drawing, encoder finish/queue submit,
fresh accepted-query capture and retained query cloning. `draw_work` includes
internal CPU preparation and resources; `resource_setup` is not an inventory
of every allocation. Queue-submit wall time does not measure GPU completion.
Capture statistics read the already captured query data rather than capture
it again. Dense-copy byte counters omit container overhead, nested meshes,
allocator capacity and total heap/GPU memory. Diagnostic transport and
sampling overhead remain in the candidate and are not all inside the named
profiled portions.

The staged path still gives candidate B a separate target while accepted A
retains its image, query data, site, camera and input routing. Collection
publishes only the captured candidate; retaining a frame clones its existing
query `Arc` and does not draw, allocate a target or take a fresh snapshot.
Counters describe attempted work and remain visible on refusal/discard;
their observation never publishes candidate facts early.

The before arm is immutable R6 `bin/mesocosm-worker-attribution-03`, SHA-256
`ea2eb26a0c2ee9163163a7084d82c6f3c1a08edcef49c397d616ec384e36c0c5`.
The candidate `bin/mesocosm-producer-preparation-02` has SHA-256
`cd11cfcf6fe546fb388a9c394b52120fe7b2f964255d088d0839e354033dc934`.
Its SDK overlay manifest is `overlays/r7-02/overlay-manifest.json`, SHA-256
`bc9c6a76b69a86323e5f19512abbf608dea5662ea8a7a3916dd9f8de6403c55c`;
the full receiving manifest has SHA-256
`3777c4048b24d10babffd7b6c897b82668d983eeaf007d3e2541bc261ba7383c`.
There are sixteen selected locked identity replacements, zero new registry
or foreign Git records, and zero changes to common dependency records.

| Final R7 executed gate | Passed | Scope |
| --- | ---: | --- |
| Mesquite | 54 | Optional transport, snapshots, phase predecessor and existing profile accounting. |
| Isometer | 73 | All unit controls, including the query snapshot/statistics seam. |
| Mesocosm default | 82 | Six ignored; the native staging control below executes explicitly. |
| Mesocosm staging | 1 | Actual-device resource/query/publication lifecycle; 87 filtered. |
| WorldSave | 1 | Native save/replay preservation. |

All named gates have zero failures. `validation-summary-02.json` binds their
logs and binary/source identities. Prior R6 owner gates are retained under
their exact source-reuse witnesses; they are not reported as newly executed
R7 suites. Earlier source/control corrections and generation01 remain
retained. The source review corrected sparse-counter tests that indexed
unperformed entries; exporter controls also preserve scope disappearance
instead of silently omitting its unavailable interval.

At qualification and immediately after the four headed runs, all 1,694
receiving files match their frozen copies. Later authorized updates to the
WT scene plan and its index are exactly two documented post-qualification
doc deltas. The source audit records those exceptions explicitly; it does
not claim all 1,694 live files remain identical after documentation adoption.
The original compiled copies and all earlier source/evidence cuts stay fixed.

### Measured producer work and resource counts

Assembly registers one viewport `SectionProducer` through one `LEAF` key.
The declared slot count is distinct from an observed registry count, and
the previews list counts diagnostic buckets. The C4 body scope is ten voxel
bodies, zero physical members and zero scene-physical bodies. Those ten
bodies are contents of the one scene, not ten registered producers.

Both candidate backends record 266 accepted phase pictures and 532 raw
attempts. Adjacent same-epoch raw differences yield 13 fresh Section draws
and query captures: camera 4, inspector interaction 6 and native steps 3.
The other 253 publications retain the query `Arc`; all 266 publish. These
13 fresh draws differ from the provider's 15 slow-worker packets.

| Direct measured CPU stage | Phase calls | Retained | Components |
| --- | ---: | ---: | ---: |
| Explicit resource setup | 26, two per fresh attempt | 2.295ms total; 0.177ms per fresh attempt | 2.367ms total; 0.182ms per fresh attempt |
| Draw work | 13 | 6.951ms mean; 17.024ms maximum | 7.277ms mean; 16.010ms maximum |
| Fresh query capture | 13 | 3.091ms mean; 4.920ms maximum | 2.860ms mean; 5.244ms maximum |
| Encoder finish and queue submit | 13 | 0.694ms mean | 0.737ms mean |
| Retained query clone | 253 | 0.751µs mean | 0.776µs mean |

Every executed call in these five new stages has an observed valid CPU
value: zero unavailable or rejected calls. This narrower result does not
repair the broader R6 UI/worker derived-remainder model. The raw complete
CPU fields, rejected counters and overlapping envelopes remain separate.
Unperformed stage entries are absent, not fabricated measured zeros.

The phases create 13 target textures and 26 views in each candidate while
creating zero Sections. Lifetime counters show one Section, 19 target
textures/38 views and 19 fresh draws/captures. Six fresh calls lie outside
the measured phases, including early candidates and capture work; these
are not all cold startup. The first-document interval includes three early
candidates, one Section creation and two discards. Phase-only counts must
not be substituted for the lifetime totals.

The 13 fresh snapshots account for 109,085,184 dense terrain payload bytes:
109,051,904 atlas bytes and 33,280 pointer bytes, or 8,391,168 per capture.
There is no phase ground rebind, but that counter does not prove the view
map is unchanged. Existing query `Arc` sharing begins after fresh capture;
it does not eliminate this initial copy. This is copied payload accounting,
not a measured heap-allocation or memory-bandwidth total.

### Matched result, contention and interpretation

The protocol executes exactly one A-retained/B-retained/B-components/
A-components sequence. All four runs exit 0 without the watchdog and have
the same phase counts and fresh-pair final fields. Recorded paired UI
portions sum 4.031290→4.039780s retained and 4.036371→4.040994s components,
changes of +0.21% and+0.11%. No overall UI improvement is established.

Execution explicitly admits external desktop/build contention for this
performance launch. Five of eight before/after snapshots observe compilers;
reported activity includes Zed, Cargo, desktop helpers and other processes,
with load average 4.27–4.65. This permission changes only performance-launch
conditions; it does not bypass Cargo storage guards. Endpoint snapshots are
not continuous CPU/GPU observations. The comparison supports bounded named
CPU/counter interpretation, not causal elapsed-time, quiet-desktop or
physical refresh claims.

All four native saves equal frozen R6 byte-for-byte, SHA-256
`f4648a3d62957a820ceae6cef1a48478b96b7a3d6b2deb01347270e00ce1207a`.
All eight final/mixed scene crops equal frozen R6, decoded RGBA SHA-256
`cc64b84ceda2fddfe65233b8db7c5a86dd6938a59b8b9d7cfb26fcc63a56eb32`.
Fresh component full images are exact; retained images differ at one pixel,
one channel level, outside the scene crop, without alpha differences.
Relative to historical R6, the successful-scene-draw presentation counter
is 19 fresh versus 18 historical; native state remains exact. Do not claim
every historical full-field value is identical.

The measured CPU target favors sharing immutable query data over pooling
target textures: initial query capture is materially larger than the small
explicit setup scope. Drawing remains the dominant named stage and still
contains unattributed internal preparation/resources. A future query change
needs actual-change/revision admission before mutable access; naïve
`Arc::make_mut` before an always-called refresh can copy on a `Current`
refresh while A holds the old map. A target pool must preserve accepted and
in-flight ownership. Neither optimization is implemented or qualified here,
and unavailable GPU timestamps cannot establish a pool's GPU benefit.

The genuine R7 initial→final working-source delta contains four Mesquite
source/test files and fourteen Isocosm product/query files; other qualified
owner source remains unchanged. The audit verifies 396 SDK inputs, 106 live
approved owner inputs and 395 frozen R6 inputs. Of the 1,694 receiving files,
1,692 remain live-byte-identical and exactly two authorized WT plan/index
changes are recorded after qualification. Standalone manifest/lock
normalizations remain separate qualification patches.

Canonical Netrender independently advances clean from 9214a084 to 0b93d99c
across `netrender-notes/2026-11-05_sparse_glyph_culling.md` and
`netrender/src/vello_backends/sessions/text.rs`. Its root/Net manifests and
lock remain unchanged. Both committed cuts and the patch are retained,
without attribution or adoption by the qualified 72f54a owner or fixed SDK.
An all-canonical-HEADs/bytes-unchanged claim is false; the other canonical
source and prior dirty work stay preserved. Fetches can also change remote
refs/behind counts, and new integration worktrees/source-object refs change
metadata inventories. Those intentional metadata updates do not imply a
source, index or canonical-HEAD mutation.

The qualified source remains isolated. Genuine initial→final review patches,
canonical preservation and explicit post-qualification doc changes are
retained in `review-deltas/r7-final-03/`. Main-source reconciliation uses
separate clean integration trees and exact fetched targets; its source,
dependencies, tests and publication require independent qualification.
Frozen R7 results do not qualify those newer-main ports.

R7's bounded attribution is complete. R8 main integration has priority,
followed by a bounded query-sharing experiment and deeper draw attribution.
Complete CPU decomposition, actual GPU duration, physical input/display/AT
evidence and consistent 60Hz across supported hardware/platforms remain open.
C4 has no physical native bodies and does not qualify embodied ecology.
R4 VTT/Eponym receiving and conditional C5 Didgeridoo retain their scope.
Ecology, rules, clocks, attention, native storage and replay are unchanged.

## 2026-11-05 — R8 current-main Mere owner receiving

This is new owner receiving evidence on published Mere base
`81e39365d7c1792b1840284d174325ceae8c9bf9` plus the bounded source port,
separate from the copied SDK used in R2–R7. The 92 staged Rust paths remain
byte exact through the current-main reconciliation. Dirty canonical source
and the frozen experimental worktrees remain protected. This section does
not certify the later Isocosm consumer or transfer historical performance
measurements to a new supplier tree.

The exact published prerequisites are:

| Owner | Published source | Owner qualification |
| --- | --- | --- |
| Selected Vello provider | `dfb9b53b315561887761deff319afed08cc33ff0` | 18 tests |
| Netrender | `f8811186cd82b0d4b6758b15c6bcecd415273d52` | 106 tests |
| Genet | `ceb5a9795a8f26ffe3c6fba25a55bfce3dd0a15d` | 66 tests |

Genet and Netrender use the canonical WWW source URLs and original-URL
bridges at these same fixed revisions. The root changes 38 existing selected
declarations and adds 39 reviewed G/N bridge rows. Its lock rekeys 47 selected
records while preserving all 1,797 other latest-main package records. The
only selected dependency changes remove netfetcher's now-dev-only websocket
edge and qualify the existing scripted-DOM regress edge to its fixed source.
No registry or foreign supplier update is selected by that alignment.

The two independent web manifests align 26 existing selectors. Graphshell's
root-specific bridge follows its actual lock roster; Smoke's bridge follows
its declared package closure and subsequent precise-source metadata audit.
Graphshell preserves all 1,097 nonselected records while rekeying 35 owned
identities. Its accepted resolution changes only the order of the same ten
unused markers; all 1,132 package records retain their logical payloads and
bytes. The resulting Graphshell lock SHA-256 is
`9358d59833e1858fc338685d430e92f7718c1e85dd0029395f957c480b075e48`.
Smoke originally has no lock. Its root-seeded offline proposal, locked
metadata and actual wasm check preserve that absence on restoration; no
new standing lock or moving supplier is adopted.

| Executed owner gate on the 81e cut | Result |
| --- | --- |
| Rootstock unit suite, including the existing incremental controls and all native controls | 257 passed; no failures, ignored or filtered tests |
| Rootstock `prepared_frame` integration | 6 passed |
| Mesquite unit suite | 54 passed |
| Cambium desktop host unit suite | 25 passed |
| Accessibility adapter unit suite | 24 passed |
| Complete native aggregate | **366 passed; 0 failed, 0 ignored, 0 filtered** |
| Changed native examples: `smoke`, `t4_receipt`, `harness`, `native_shared_recovery` | All four checked successfully |
| Changed wasm examples: `a11y_page`, `presentation_page`, `shared_recovery` | All three checked successfully on `wasm32-unknown-unknown` |
| Standalone Graphshell-web, full default wasm library | Check passed |
| Standalone Genet-web-smoke, actual wasm library | Check passed |

The wasm gates compile the real target-specific bodies, with the declared
WebGL/preview features where required; a native empty stub is not their
qualification. These are compilation and owner tests, not browser execution,
OS accessibility delivery or physical display measurements. No Rust fixes
were needed for this receiving cut.

Evidence is under
`Code/output/mesocosm-scene-composition/2026-11-04/producer-preparation/integration/mere-current-main-01/generations/current-81e-01/`:
`native-runtime-result.json`, the native/wasm example `result.json` files,
the two `web/*/wasm-check/result.json` files, the precise metadata audits and
`final-source-verification.json`. The source generation SHA-256 is
`0399d21388d85aaa1c323a2a3c78173f3d43f4c6b840bcd03af446c65c498ff4`;
the final source-verification receipt is
`9a9ad1168275639498be88c22f434149f3c6dca25f575fa3b24b814753a9b68e`.
That receipt confirms code/cohort inputs and complete staged entries stay
unchanged and Smoke's original no-lock state is restored. Raw index
cache-tree bytes are not claimed identical to the copied snapshot.

After this 81e cut, a fresh nonoverlapping main advance includes two
Graphshell source children outside the staged port. The task source/cohort
is preserved; the affected Graphshell recheck after the nonoverlapping current-main forward advance passes. This separate check
must not be silently counted as part of the original 81e gate. Its executed
receipt and the final source/publication manifests bind the actual published
owner state. Native gates are not broadened or repeated without a changed
compiled closure.

The forwarded receiving head is
`0aaaf07a09e3a15551b1f3514f47a95e590c9001`. Its affected Graphshell wasm
check passes, with 3,083 compiled inputs and scoped staged entries unchanged.
`integration/mere-current-main-01/generations/current-0aaaf-01/forward-reuse-proof.json` proves all 253
original non-documentation inputs remain exact and Graphshell is absent
from the native, changed-example and Smoke compiled closures. The exact
new-head result is `integration/mere-current-main-01/generations/current-0aaaf-01/graphshell-wasm/result.json`,
SHA-256 `3bd352c54316ad570341bcc5a41a763be076000f0587694f577d66d1ddfc8c8b`.
The final `integration/mere-current-main-01/qualification.json` has SHA-256
`e4a5eaed89e295ecba720550dd11dec31bac82e746a8df1e8aa3df91dbae0bbe`;
it binds the owner gates, fixed supplier sources, reuse proof and both
standalone wasm results. These paths use the same producer-preparation
evidence root as the owner receipts above.

At this qualification cut Mere publication is pending, and Isocosm's actual
fixed-M consumer qualification has not run. R8 therefore remains active.
The newly published Isocosm foreign-family baseline is preserved rather than
replaced by an arbitrary cohort; source/name/namespace mergers and affected
consumer gates remain explicit. New-main C4 saves and scene crops must use a
fresh within-cohort oracle, not an assumed historical R7 hash. R4 VTT/Eponym
staging, conditional Didgeridoo, query sharing, deeper draw attribution,
complete CPU/GPU decomposition, physical input/display/AT evidence and
consistent 60 Hz retain their existing qualification boundaries.
