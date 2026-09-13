<!--
Copyright 2026 Mark Alan Boykin
This Source Code Form is subject to the terms of the Mozilla Public
License, v. 2.0. If a copy of the MPL was not distributed with this
file, You can obtain one at https://mozilla.org/MPL/2.0/.
SPDX-License-Identifier: MPL-2.0
-->

# L0c residual diagnosis, 2026-09-13

Archival note: the diagnostic text below is preserved from the local receipt.
Its source line numbers precede the archival license headers. This compact
archive includes the foreground/tie JSON and two representative crops;
the other named artifacts remain in the full local receipt described in the
[probe README](../../README.md).

Read-only CPU diagnosis of saved frames and current source. No renderer or probe
changes, GPU work, or benchmark reruns were performed for this note.

Every coloured interior mismatch in the sampled eight-body/eight-shape crossing
run lies on coincident body faces carrying different checker materials. This is
evidence of competing surfaces at equal geometric depth, not a continuous-parent
pose error. The remaining one or two grey terrain pixels are unassigned here.

| Swarm crossing frame | Foreground interior pixels | Bad interior pixels | Bad / foreground interior | Coloured pixels on tied faces | Grey terrain pixels |
| --- | ---: | ---: | ---: | ---: | ---: |
| 0 | 553489 | 223 | 0.040290% | 221 | 2 |
| 30 | 581752 | 1639 | 0.281735% | 1637 | 2 |
| 59 | 513345 | 211 | 0.041103% | 210 | 1 |

Foreground excludes the oracle's exact background RGB [15, 15, 20]. Interior
and mismatch use the probe's existing tests: four neighbours within 2 per colour
channel, mismatch above 32 in any channel. All bad interior pixels are foreground.

## World-plane evidence

The swarm constructs every body at y=2. Torso dimensions [6,8,6] put all torso
tops at world y=10; head dimensions [4,4,4], pivot [2,0,2] and attachment [3,8,3]
put all head tops at y=14. Bodies overlap freely. Reconstructing seeded positions
and the orthographic rays finds at least two coincident top faces at every
coloured mismatch. Both saved pixel colours match those faces' checker materials.

For example, crossing frame 30 pixel (751,285) lies at approximately
(5.995210,10,16.916081). Body 4 contributes torso material 3 and body 6 contributes
torso material 4 at that same point. Pixel (988,325) lies at approximately
(18.016473,14,16.523692), where body 1 contributes head material 6 and body 7
contributes head material 5. The head and torso material pairs account for all
1,637 coloured mismatches in this frame.

The original articulated fixture has the same kind of conflict at frame 30:
the tail has reached a half turn. Its end face coincides with the torso's +x face
at x=26, over y=7..9 and z=10..12. All 110 coloured mismatches lie within this
shared face. For example, pixel (1242,466) maps to approximately
(26,8.935596,11.203136). The torso uses material 4 with local +x shading; the
tail uses material 8 with local -x shading, rotated to face world +x.

Articulated frame 30 has 503,884 foreground-interior pixels and 112 bad interior
pixels: 0.022227%. The other two are the persistent grey pixels at (683,633) and
(817,832). The ordinary continuous-yaw frames have only those same two grey
residuals, with no coloured body mismatch in the sampled frames.

## Why the saved images can disagree

The oracle traverses chunks then bodies/parts and accepts strictly smaller depth
(`project.rs:231`, `project.rs:300`). LiveBodyRenderer groups draws by content
address and uses LessEqual (`live_body.rs:286`, `live_body.rs:447`). The two paths
also calculate and interpolate depth differently. Therefore the current test has
no common choice for equally deep, differently coloured faces; the exact share
attributable to comparison operator, draw order or floating-point evaluation was
not isolated. Merely changing LessEqual to Less would not establish equivalence.

This diagnosis does not require banning body overlap or declaring C pixel-exact.
Retain the raw mismatch counts and classify these tied-surface samples separately
from the one or two unresolved terrain pixels.

Source anchors are under `repos/mere/crates/probes/wing-three-paths/src/`:
`world.rs:289` (part dimensions/attachments), `world.rs:301` (tail),
`world.rs:374` (swarm construction), `world.rs:442` (tail turns),
`world.rs:469` (swarm stepping), `project.rs:344` (comparison).
Renderer anchors are under
`repos/isometry/mesocosm/crates/mesocosm-render/src/live_body.rs`.

## Artifacts

- `diagnosis_ties.json`: all sampled coloured residuals classified, with world-plane examples.
- `diagnosis_foreground.json`: counts, foreground denominators and colour pairs for swarm crossing, ordinary articulated and ordinary continuous yaw.
- `diagnosis_pixels.json`: mismatch regions and connected components for the swarm frames.
- `diagnosis_planes.json`: representative frame-30 ray intersections and reconstructed body origins.
- `diag_locations_f30.png`: swarm mismatch locations marked magenta.
- `diag_pair_f30_component0.png`: representative overlapping torso-top crop, oracle left and C right.
- `diag_articulated_f30.png`: torso/tail shared-face crop, oracle left and C right.
