# Mere Vello release adoption — 2026-10-11

**Status:** Historical receipt at the recorded source revisions. Later stack
repins require their own qualification.

Status: passed for the scoped consumer and CPU adoption. Consumer, Wasm target
and direct Pictograph adapter compilation passed on the final base. All twelve
refreshed production CPU comparisons passed. Root metadata and boundary reruns
passed after closing Pictograph's registry/Git device-package duplication.

Mere adopts the maintained Vello Classic 0.11 release and the qualified Genet /
Netrender consumers. The source base is
`e99add00413b97ede6566a16b33045fd302476ee`. The work initially used
`21bc504ab18dd4b3dca199fbffb6389a08d545df`; the intervening upstream changes
only affect comments/documentation, and the final checks use the refreshed base.
The qualified source/lock changes are committed as
`e3ed4d65d31b27c21aa8322a4546982173ed1d33`, with no source changes after checks.

| Source | Revision | Identity |
| --- | --- | --- |
| `https://github.com/mark-ik/vello` | `5862bba26c8a9f56f6a5fdc8ea6966b360a5d63a` | `netrender-vello`, `vello_encoding`, `vello_shaders` 0.11.0 |
| `https://github.com/merely-made/netrender.git` | `46606610cd387e2e6956d8a3ebbcf442eab210d9` | Netrender / PaintList 0.1.2 |
| `https://github.com/merely-made/genet.git` | `cec760f4ae6ce336823bc2be250d9ab4e35a8a47` | Qualified consumer; independent `genet-parley` / `genet-fontique` 0.10.0 |

Both workspace Scene aliases use the same Classic package and source. Sprigging
inherits that dependency with default features disabled. The root and four
standalone roots restate the coherent Classic / encoding / shader patch trio;
the obsolete standard Vello 0.10 tag patch is removed. Historical receipt
manifests remain intact.

## Evidence

[identity.json](identity.json) records the six changed manifest hashes, five
lockfile hashes, compact resolved identities, and the SHA-256 hashes of full
metadata artifacts kept under
`/Users/markik/Code/output/vello-release-adoption-20261011`. Full metadata JSON is
not duplicated into Git.

| Check | Result | Status |
| --- | --- | --- |
| Root full metadata | 1,611 packages | Passed |
| Graphshell web full metadata | 894 packages | Passed |
| Genet web smoke full metadata | 341 packages | Passed |
| Distillery probe full metadata | 793 packages | Passed |
| Distillery remote fixture full metadata | 990 packages | Passed |
| Port / Graphshell dependency boundaries | Passed, exit 0 | Passed |
| Native host consumer compilation | Three packages, native-host enabled | Passed |
| Wasm target compilation | Graphshell web, default features | Passed |
| Pictograph Vello adapter compilation | `vello` feature, library only | Passed |
| Refreshed production CPU frames | Twelve exact direct/retained comparisons | Passed |

Each recorded metadata graph selects one Classic 0.11 family and one `wgpu`
30.0.1 package. The first root snapshot also exposed a registry
`netrender_device` 0.1.2 alongside its qualified Git counterpart through
Pictograph's dev dependency. A root patch redirects that registry edge to the
qualified revision. The regenerated root metadata now contains exactly one
device-package identity; source and snapshot hashes reflect that repair.

Accepted check outputs are saved in [checks](checks). The CPU
[runtime receipt](production-cpu-frames/receipt.json) records pixel identity,
application-state preservation, and that GPU initialization never occurred.
The twelve PNGs are saved beside it. Eight frames are real production reader and
stylesheet components across light, dark, narrow and authored cases. Four are
compositions with an explicit opaque CPU snapshot of the production stylesheet
preview. They do not constitute the complete workshop producer-resource bundle.

The production CPU probe is a separate standalone workspace. Its actual build
`Cargo.lock`, manifest and unchanged twelve-frame probe source have their own
hashes in `identity.json`, separate from Mere's five lockfiles. That lock resolves
`vello_cpu` / `vello_common` 0.3.0 and `glifo` 0.4.0 at the same provider revision.
An optional `--locked` full metadata attempt requested additional lock
resolution and exited 101; its empty JSON is not used as evidence, and the
running build's lock was preserved. Probe provenance is taken directly from that
lock, with no full probe-metadata claim.

[probe/preparation.json](probe/preparation.json) is the unchanged pre-run snapshot;
its `prepared_not_executed` status describes preparation time. The runtime
receipt above records the subsequent successful execution. The saved
[probe manifest](probe/Cargo.toml) uses absolute paths to this checkout and local
font inputs, so it reproduces on this checkout rather than being a portable
fixture. Font-bearing `.scene` files are retained only in the local output
directory.

The compiled CPU proof executable is preserved locally at
`output/vello-release-adoption-20261011/production-cpu-probe/cambium-cpu-source-proof`,
SHA-256 `e3e27fb8f45e3f559b0d16204d309971dc0a746d71f2e31842744ee00913efd4`.
It is not included in Git. Root visually inspected the authored and narrow
composition PNGs; this is local image evidence, not host presentation.

## Acceptance limits

Metadata establishes package/source identity. Consumer compilation establishes
the selected host API compatibility. Neither establishes presented pixels.
These consumer metadata graphs select Classic; the CPU/GPU/glifo provider
release qualifications are separate evidence.

The CPU comparisons exercise production Tabard/Cambium sources on the new pins
and source expansion/re-lowering, without packet caching. They establish direct
versus retained equality on the new provider; they do not establish old/new or
cross-renderer pixel parity. Source admission through the Hybrid enum also does
not establish GPU execution.

GPU pixel parity, full producer external-resource replay, native or browser
presentation, input/accessibility acceptance, and driver-reset recovery require
their coordinated validation receipts. This directory does not claim those
gates passed.

## Source-header amendment (2026-10-11)

The retained probe source now carries its MPL-2.0 Exhibit A header. Runtime
code is unchanged. `identity.json` retains the qualified original source hash
and records the current header-added hash separately; prior compile and pixel
receipts continue to describe the original input bytes.
