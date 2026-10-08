# Licenses in this repository

**This repository: MPL-2.0.** Every file Mark wrote carries Exhibit A and the
SPDX tag `MPL-2.0`, per the
[license posture brief](design_docs/2026-08-22_license_posture_brief.md) of
2026-08-22, except the frozen evidence below, which carries it through this
ledger and the root licence. The full text is in [`LICENSE`](LICENSE).

This file is the provenance ledger. It is the authority for what the relicense
tool skips: `scripts/relicense_headers.py` reads the backtick-quoted paths in
the first column of the Retained licenses and Frozen evidence tables and never
touches them. Provenance comes before license — a file gets Exhibit A only if
Mark wrote it.

## Retained licenses

Third-party code keeps its own license and its own notices. Nothing here is
relicensed, and nothing here receives a Merely copyright line.

| Path | License | Upstream | Notice files |
|---|---|---|---|
| `support/patches/cubecl-runtime` | MIT OR Apache-2.0 | [tracel-ai/cubecl](https://github.com/tracel-ai/cubecl) | `LICENSE-MIT`, `LICENSE-APACHE` in-tree |
| `support/patches/cubecl-wgpu` | MIT OR Apache-2.0 | [tracel-ai/cubecl](https://github.com/tracel-ai/cubecl) | `LICENSE-MIT`, `LICENSE-APACHE` in-tree |
| `support/patches/cubek-reduce` | MIT OR Apache-2.0 | [tracel-ai/cubecl](https://github.com/tracel-ai/cubecl) | `LICENSE-MIT`, `LICENSE-APACHE` in-tree |
| `support/patches/burn-cubecl` | MIT OR Apache-2.0 | [tracel-ai/burn](https://github.com/tracel-ai/burn) | upstream's |
| `support/patches/burn-remote` | MIT OR Apache-2.0 | [tracel-ai/burn](https://github.com/tracel-ai/burn) | upstream's |
| `ports/pelt/examples/resources` | MPL-2.0 | [servo/servo](https://github.com/servo/servo) (`resources/`), by way of genet | Servo's |
| `crates/conatus/conatus/src/resident/binning` | Apache-2.0 | [dimforge/nexus](https://github.com/dimforge/nexus) at `1cfbd76`, by Sébastien Crozet / Dimforge | `LICENSE-APACHE` in-tree; port and change notice in `mod.rs` |
| `crates/canvas/pictograph/src/canvas/tests/data/arxiv_topics.tsv` | CC0-1.0 (arXiv metadata) | [arXiv API](https://export.arxiv.org/api/query), terms at <https://info.arxiv.org/help/api/tou.html> | `arxiv_topics.provenance.json` beside it: the twelve requests, the licence quote, the window, the selection |

`ports/pelt/examples/resources` holds two Servo logo images that Pelt's
example documents reference by URLs that climb to the repository root and
descend again. They came with Pelt from genet on 2026-09-03, sat at the
repository root until later the same day, and are Servo's work under the same
MPL-2.0, so they keep Servo's copyright and take no Merely line; genet's own
ledger carries the identical row. The directory's `README.md` is Mark's and
carries the house header.

384 tracked files. These are vendored patch trees consumed through
`[patch]`; they are upstream's work carrying upstream's terms.

`crates/conatus/conatus/src/resident/binning` is a close hand port, to CubeCL,
of Nexus's exclusive scan (`src_rbd_shaders/utils/prefix_sum.rs` and its
staging in `src_rbd/utils/prefix_sum.rs`) and of the count / cursor / finalize
pattern of its MPM particle sort (`src_mpm_shaders/grid/sort.rs`), made on
2026-10-02 for the physics catalog plan's P5a cell list. Nexus declares `MIT
OR Apache-2.0` but ships only the Apache-2.0 text, so the port is treated as
Apache-2.0 per the 2026-10-01 licensing ruling; the module header names the
upstream, the author and the changes, and the license text sits beside it. The
cell-walking force kernel that reads the bins (`kernels::exclude_cells`) and
the host code that calls them are Mark's and stay MPL-2.0.

`crates/canvas/pictograph/src/canvas/tests/data/arxiv_topics.tsv` is test
data for the Meaning channel's purity receipt (dynamics grammar plan, F42 and
F43): 900 arXiv ids, titles and primary categories, 150 in each of six,
fetched by the coordinator on 2026-10-03 with each request approved by Mark.
arXiv says of this metadata: "You are free to use descriptive metadata about
arXiv e-prints under the terms of the Creative Commons Universal (CC0 1.0)
Public Domain Declaration." CC0 asks for no notice; the row and the
provenance note record where the bytes came from (SHA-256 `942a7716…`).

## Frozen evidence

Mark's own sources whose bytes a recorded receipt pins by SHA-256. They are
MPL-2.0 like everything else he wrote, but a per-file header would change the
bytes and break the receipt, so they carry none. Exhibit A allows the notice
to sit where a recipient would look for it when a per-file notice is not
desirable; for these paths that place is the root `LICENSE` and this ledger.
The tool skips them, as it skips Retained licenses.

| Path | Pinned by | Ruled |
|---|---|---|
| `design_docs/mere_docs/testing/receipts/2026-09-08_stack_pillar_probes` | `artifact-sha256.json` at its root and in `custody-backend/`, and the arena's run JSONs; its `.gitattributes` keeps the bytes so the digests survive checkout | 2026-10-06, ruling S79 of the [stack seams plan](design_docs/mere_docs/implementation_strategy/2026-10-04_stack_seams_plan.md) |

A receipt that does not pin its sources' bytes is not frozen evidence: its
sources take the header like any other (so the 2026-09-20 resource resolution
probes do, under the same ruling).

## Derivatives carrying MPL-2.0 with an upstream notice retained

These are **not** skipped. Each file receives Exhibit A and Mark's copyright
line, and every upstream copyright line above it is kept verbatim. Apply with
`--retain-notice`, which preserves foreign copyright lines while replacing
Mark's own.

| Path | Upstream | Notices kept |
|---|---|---|
| `crates/system/luggage` | [cargo-packager-updater](https://github.com/crabnebula-dev/cargo-packager), MIT OR Apache-2.0 | `Copyright 2019-2023 Tauri Programme within The Commons Conservancy`; `Copyright 2023-2023 CrabNebula Ltd.` |

Ruled 2026-08-27, on the brief's substantial-derivative precedent: tucket keeps
MeshCore's MIT notice, and cambium and meristem go MPL-2.0 with the Apache
notice retained. Both MIT and Apache-2.0 permit relicensing a derivative so
long as the notice travels with it. luggage is published (0.1.0, MIT OR
Apache-2.0); that version keeps its grant permanently and MPL-2.0 ships at its
next functional bump, per the sweep plan's invariant 8.

**This section is deliberately not the skip list.** The tool reads only the
`## Retained licenses` and `## Frozen evidence` tables above. Adding a path
here documents a disposition; it does not exempt the path from receiving a
header.

## Published grants that arrived with a crate

A published version keeps the grant it shipped with, permanently; MPL-2.0
reaches it at its next functional bump, per the sweep plan's invariant 8. That
rule already governs `crates/system/luggage` above. It governs the
engine-management layer too, which arrived from genet on 2026-09-03 under the
[platform boundary plan](design_docs/archive_docs/2026-10-06_completed_plans/2026-09-02_platform_boundary_and_repository_topology_plan.md)'s
P3, after genet's own 2026-08-27 ruling had already put every source in these
crates under Exhibit A and every manifest on `MPL-2.0`.

| Crate | Path | Published version's grant | In-tree notice files |
|---|---|---|---|
| `inker` | `crates/inker/inker` | 0.1.1, MIT OR Apache-2.0 | — (the pair carried over from `verso-tile` 0.1.0 was dropped by merge `0a8198ba`, 2026-09-06, and stays removed under ruling S40) |
| `document-canvas` | `crates/inker/document-canvas` | 0.1.0 | — |
| `nematic` | `crates/nematic/nematic` | 0.1.1 | — |
| `illume` | `crates/nematic/illume` | 0.0.2 | — |
| `errand` | `crates/system/errand` | 0.3.4, MIT OR Apache-2.0 | — |
| `tinct` | `crates/cambium/tinct` | 0.1.2, MIT OR Apache-2.0 | — (deleted by Mark's ruling, `642ca2d7`, 2026-09-24) |

Each arrived from genet 2026-09-03 under its own published grant; relicensing
is the sweep plan's call at the next bump. Nothing here was relicensed on the
move, and the four `LICENSE-MIT` / `LICENSE-APACHE` files that `inker` (for the
folded `verso-tile`) and `tinct` carry are **not** to be deleted: they are the notice text for versions
already on crates.io. This is the same disposition genet's own ledger records,
which names `sprigging` 0.2.1, `illume` 0.0.2, `errand` 0.3.4, `tinct` 0.1.2
and `inker` 0.1.1 explicitly and keeps the ruling as the precedent the sweep was
decided on.

**Amended 2026-10-06** (stack seams plan, ruling S40): both notice-file pairs
are gone. tinct's was deleted by Mark's ruling (`642ca2d7`, 2026-09-24);
inker's was dropped by merge `0a8198ba` (2026-09-06) and stays deleted, since
inker is MPL-2.0 like the rest of the workspace. The versions already on
crates.io keep the grant they were published under.

The three engine adapters — `scrying-engine`, `graft-engine` and `weld-engine`
— are `publish = false` and have no published version, so nothing is held open
for them.

This section, like the one above it, is **not** the skip list. Every source in
these crates carries Exhibit A already; the audit reports 0 without it.

## Exceptions under the fork/vendor criterion

**None.** The brief's §4 test — a crate stays MIT OR Apache-2.0 only when a
third party would need to *modify or vendor* it rather than merely link it —
admits nothing in this repository. `illume`, `buckram`, `errand` and `tinct`
were each proposed and declined on 2026-08-22.

If one is ever granted, its manifest says `MIT OR Apache-2.0` explicitly with a
comment naming the brief, and it is listed here. `illume`, `errand` and `tinct`
now live in this repository; the decline stands, and what they do carry is the
published-grant disposition in the section above, which is a different thing.

## How to add a file from elsewhere

1. Do not delete or rewrite the upstream copyright or license notice, ever.
2. Add its path to **Retained licenses** above with its license, upstream URL,
   and where its notice text lives. The tool then skips it automatically.
3. If it is a substantial derivative rather than a verbatim import, the brief's
   rule is MPL-2.0 on the derivative *with the upstream notice retained* —
   record it here with that disposition so the distinction is not lost.
4. Never add `license-file` to an owned manifest; the field is for retained
   third-party crates only.
5. Re-run `python scripts/relicense_headers.py --audit` and confirm the owned
   source count moved by exactly what you expected.

## A note on the discovery grep

The sweep plan's invariant 1 lists `Copyright (c)` among its discovery
patterns. That form is not universal: `crates/system/luggage` writes bare
`Copyright 2019-2023 <holder>` with no parenthesised `(c)`, and was missed by
the plan's own pattern set on 2026-08-27. Discovery should grep for
`Copyright` unqualified, then read the hits.
