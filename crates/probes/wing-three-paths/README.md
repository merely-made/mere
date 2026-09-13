<!--
Copyright 2026 Mark Alan Boykin
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
SPDX-License-Identifier: MPL-2.0
-->

# Three voxel presentation paths

An explicitly retained experiment for the
[wing presentation plan](https://github.com/merely-made/isometry/blob/main/mesocosm/design_docs/2026-09-11_orthographic_voxel_presentation_plan.md).
One orthographic scene compares A, retained planar fragments; B, content-cached
quarter-turn sprites; and C, Mesocosm resident geometry with a shared depth
attachment, composited through netrender. This is a standalone Rust/wgpu probe.
Genet document embedding, style mapping, picking and the specimen bench remain
separate consumers under the
[Genet integration plan](https://github.com/merely-made/genet/blob/main/design_docs/2026-09-12_css_3d_transforms_and_first_frame_plan.md).

## Running it

The manifest expects `mere`, `netrender` and `isometry` as sibling checkouts
under `Code/repos`. Mesocosm's `LiveBody` adapter must expose continuous
`yaw_radians`. From the `Code` directory, with a writable output directory:

```powershell
$env:CARGO_TARGET_DIR = Join-Path (Get-Location) 'repos/mere/crates/probes/wing-three-paths/target'
cargo test --manifest-path repos/mere/crates/probes/wing-three-paths/Cargo.toml --release
cargo build --manifest-path repos/mere/crates/probes/wing-three-paths/Cargo.toml --release
$env:L0C_OUT = 'C:/path/to/new-output/fixtures.json'
$env:L0C_BACKEND = 'vulkan'
$env:L0C_PATHS = 'abc'
$env:L0C_MESH_CACHE = '2048'
& repos/mere/crates/probes/wing-three-paths/target/release/wing-three-paths.exe
```

Use a fresh shell or clear earlier `L0C_*` values before reproducing fixed
fixtures. Set `CARGO_TARGET_DIR` explicitly if another workspace build uses it;
the executable above assumes the probe's own `target` directory.

| Environment control | Meaning |
| --- | --- |
| `L0C_OUT` | Output JSON path. Override the original machine-local default. |
| `L0C_PATHS` | Path order, default `abc`; `ca` and `ac` isolate the yaw order check. |
| `L0C_ONLY` | One case: `crossing`, `articulated`, `terrain_edit`, `clipped`, `material_change`, `continuous_yaw`; otherwise all six. |
| `L0C_BODIES` | Positive swarm population; unset keeps the small fixed fixtures. |
| `L0C_SHAPES` | Requested topology count, 1..65536; requires a swarm. Actual distinct counts are reported. |
| `L0C_SEED` | Decimal or `0x`-prefixed u64; default `0x5EED`. |
| `L0C_CHECK_SWARM` | `1` enables oracle checks for swarms of at most 32 bodies. |
| `L0C_MESH_CACHE` | C's resident mesh capacity, default 64; preflight includes terrain. |
| `L0C_BACKEND` | `vulkan` by default, `dx12`, or `all`; JSON records the actual adapter. |
| `L0C_DUMP` | When defined, writes sampled PPMs in `l0c_frames` beside the output JSON. |

The archived sweep sets 1,000 bodies, seed `0x5EED`, cache 2,048, shape counts
1/16/128/1000, and runs crossing and continuous yaw separately. The checked
swarm uses 8 bodies, 8 shapes, cache 64 and `L0C_CHECK_SWARM=1`.

## Recorded result, 2026-09-13

RTX 4060 Laptop GPU, Vulkan, NVIDIA 610.88, 1920x1080, three warmup and 60
measured frames. Frame time includes CPU build, encoding/submission and waiting
for GPU completion. It is not GPU timestamp duration. First draw excludes
bootstrap, renderer construction and workload generation. Other Rust builds
were active; these timings do not establish frame deadlines.

At 1,000 bodies, crossing uses the same poses on all three paths:

| Distinct body shapes | Live mesh keys | A median ms | B median ms | C median ms | B uploaded images | B image MiB |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| [1](receipts/2026-09-13/variety-1-crossing.json) | 6 | 13.18 | 9.53 | 2.09 | 12 | 5.42 |
| [16](receipts/2026-09-13/variety-16-crossing.json) | 21 | 14.51 | 9.48 | 2.44 | 24 | 8.68 |
| [128](receipts/2026-09-13/variety-128-crossing.json) | 133 | 12.86 | 9.02 | 2.95 | 136 | 39.12 |
| [1000](receipts/2026-09-13/variety-1000-crossing.json) | 1005 | 13.96 | 10.53 | 8.43 | 1008 | 276.10 |

The topology sampler removes seeded exterior torso cells while retaining the
interior and attachment planes. It tests a bounded surface-variation family,
not independently generated anatomies. Heads remain shared. Instance quads
grow from 360,072 to 374,488, while unique mesh quads grow from 432 to 278,584.
C groups by mesh, so draws rise from 6 to 1,005 per frame. Both reuse and
geometry complexity must accompany any performance comparison.

A and C now share continuous body poses. At one shared shape, the
[initial yaw run](receipts/2026-09-13/variety-1-continuous_yaw.json),
[C/A repeat](receipts/2026-09-13/yaw-order-ca.json) and
[A/C repeat](receipts/2026-09-13/yaw-order-ac.json) have matching workload digests:
A medians are 148.24/145.23/135.36 ms and C medians 2.43/2.17/2.14 ms.
The last C repeat has a 20.71 ms p95. B remains a quantized quality policy;
its yaw time is not an equivalent-speed comparison. With
[1,000 shapes](receipts/2026-09-13/variety-1000-continuous_yaw.json), C yaw is
7.99 ms median/9.01 ms p95; B reaches 819.64 MiB cumulative image uploads and
a 612.27 ms maximum while admitting newly requested facings. The other yaw
runs, [16](receipts/2026-09-13/variety-16-continuous_yaw.json) and
[128](receipts/2026-09-13/variety-128-continuous_yaw.json), are retained alongside them.

[Fixed fixtures](receipts/2026-09-13/fixtures.json) and the
[8-body/8-shape check](receipts/2026-09-13/variety-check.json) preserve sampled
oracle residuals. The colour threshold is 32/255; interior classification uses
four neighbours, and large swarms are unchecked cost workloads. C is not
declared pixel-exact. The [residual diagnosis](receipts/2026-09-13/diagnosis.md)
places all 1,637 coloured crossing residuals at frame 30 on coincident top
faces; one or two grey terrain pixels remain unassigned. Its raw
[foreground counts](receipts/2026-09-13/diagnosis_foreground.json) and
[tie classifications](receipts/2026-09-13/diagnosis_ties.json) are preserved.

## Source and archival boundary

[source-manifest.json](receipts/2026-09-13/source-manifest.json) records the
measured repository heads, dirty adapter files, exact source hashes, compiler
and executable hash. Ten probe CPU tests, 17 renderer library tests and the
Mesocosm all-features/all-targets workspace check passed for that receipt.
These are historical results, not a validation claim for later changes.

The nine Rust files were byte-identical to their recorded hashes before
archival. Their only archival edit is a prepended house MPL-2.0 header;
Cargo.toml additionally changes license metadata to MPL-2.0.
[archival-changes.json](receipts/2026-09-13/archival-changes.json) records both
hash sets. No algorithm changed and no timings were rerun for publication.
Raw JSON and the two small diagnostic crops are copied unchanged.

The full local receipt remains at
`Code/testing/wing/l0c_2026-09-13/`, including its README, rerun script, source
archive, original lockfile, adapter patch, logs and full frames. Those bulky
or machine-local files are not in this compact archive. Mere deliberately
does not track Cargo.lock, so a fresh dependency resolution is a new build;
byte-for-byte executable reproduction requires the original local source
archive/lock and the adapter state named in the manifest. Historical source
line numbers in the diagnosis precede the archival headers.

This directory remains covered by Mere's probe ignore rule. Preserve only
the intended source, README and compact receipt files with an explicit
`git add -f`; do not force-add Cargo.lock or target artifacts.
