# Custody and projection dependency boundary

Status: Implemented and published; endpoint and unpatched broker qualification passed. External receiving checks are recorded separately.

This slice makes custody independently buildable inside the existing
`graphshell-endpoint` package. It does not settle the eventual public package
owner or name. It does not change Djinn's app admission or custody policy.

## Ownership and migration

`door::app_client::AppConnection` owns the shared native handshake, stream,
session and request counter. `door::custody_client` wraps it for custody.
`door::projection::{AppBrokerClient, AppRouteCarrier, DeviceCard}` holds the
projection operations, resource/card assembly and carrier adapter. The old
mixed client paths are removed; consumers migrate immediately.

The default feature set is empty. `projection` enables the root projection
contracts; `local`, `network` and `stdio` imply it. `door` enables portable
labels, route contracts, typed envelopes and synchronous bounded framing.
`door-native` adds the native connection and discovery; `door-serve` adds
listeners and serving tasks. `custody` enables native custody DTOs, including
its own Base64 dependency; `custody-client` enables their client/provider.
`projection-door` enables the projection adapter, and `profile` the custody
identity adapter. The portable host explicitly selects `projection` on its
endpoint dependency, independently of its native feature.

Native connection/custody modules and dependencies are excluded on Wasm.
The portable challenge type does not expose `fresh`; nonce generation belongs
to native serving. Blocking network transport remains native. Personae stays
as the existing socket discovery dependency; its fallback path is unchanged.

The two existing payload families occupy only the Request/Call and
Response/Notice/Answer slots of the generic envelopes. Schema names, endpoint
environment variables, socket paths, 1 MiB bounds, native-endian framing and
Windows busy-pipe handling are preserved. Custody branches before projection
carrier admission. App labels and a same-user transport are not executable
authentication. Djinn-to-custodian `custody-wire` remains a separate protocol.

## Error and buffer scope

JSON diagnostics retain category, line and column. Frame, application-client
and browser-carrier errors retain no Serde source error or input text. Fixed
fixtures cover envelope shapes; malformed enum sentinels and nonempty released
keys exercise Debug, Display and source-chain redaction, including unexpected
and mismatched answers. Released key containers keep their prior redaction
and zeroization behavior.

Owned frame body buffers are guarded before reads and serialization. Writes
use `serde_json::to_writer` into the existing guard. Tests observe wiping while
the allocation is still owned after success, decode/serialization errors,
partial reads, actual writer I/O failures and async cancellation. This guarantee
does not cover Serde scratch allocations, earlier allocations from Vec growth,
allocator copies, caller-owned payloads or kernel buffers.

## Qualification at the endpoint boundary

Source through `171bec593` (the last endpoint source change is `d5d5605e1`).
Linux x86_64, Rust 1.99.1; serialized machine gate, 3 GiB memory, zero swap,
150% CPU, one build job and one test thread. This is not an MSRV receipt.

`root-split-final-matrix-1` passed:

| Selection | Passing tests |
|---|---:|
| custody-client | 49 |
| projection-door | 34 |
| custody-client, projection-door, door-serve, profile | 70 |
| custody DTOs only | 26 |
| portable door | 19 |

These suites overlap; the rows are executions under different feature sets.
Five independent Cargo workspaces in `support/fixtures/custody-projection`
avoid workspace and dev-feature unification. Four run one behavioral test
each; the host fixture is compile-only. Expected missing-API compiler errors
prove custody excludes projection APIs, projection excludes custody clients,
and portable/DTO-only door builds exclude the native connection.

| Independent native consumer | Normal/build package names |
|---|---:|
| custody client | 122 |
| custody DTOs | 113 |
| projection client | 66 |
| portable door | 16 |
| portable Graphshell host | 35 |

Counts are target-filtered package names, not executable sizes. Custody's
closure excludes Chirograph, sceno, scenotime, Hest, Pandect, mere-kernel,
mere-alembic and Graphshell. The projection endpoint's direct edges exclude
custody DTO dependencies while retaining Personae discovery. The fixture
script retains feature sets, direct-edge provenance and expected compiler
failures as well as package names.

Portable door and host fixtures compile for Wasm (16 and 34 names). The endpoint
also compiles for Wasm with custody-client, projection-door, door-serve, profile
and network selected; native APIs are gated away. This is compilation evidence,
not browser IPC or Windows execution qualification.

The prior `root-split-native-1` also passed Djinn's library check and
station-custody all-targets check. The broker integration run against the old
locked Knot source failed at its removed `app_client::AppBrokerClient` import.
Receiver integration must use migrated Knot and remains pending below.

Retained logs and JSON receipts are on T-PC under
`/home/markik/Code/testing/custody-contract-20261029`:

- `root-split-final-matrix-1.log`: SHA-256 `e9a167f8cb31860d189086f3bdc2ddbf55dea60375f92b116e19749117903505`.
- `root-split-native-1.log`: SHA-256 `738f3420139ab4ee270047a3906bf7f940288a2c4bfb06ae18e63ee76e5fc500`.

## Review and receiving scope

Source review caught and repaired the portable host's missing unconditional
projection feature, custody DTOs' missing Base64 edge, and six Djinn Serde error
conversion failures. Writer failure and nonempty-key tests were strengthened
before the final matrix. Earlier failed and interrupted receipts remain retained.

Knot, Turnstone, Cleromancy and Woodshed/Hocket source migrations are prepared
in their own worktrees. Isocosm remains on its explicitly older fixed Mere
revision and is outside this receiving cohort. Historical receipt snapshots
are not rewritten. The public crate name remains `graphshell-endpoint`.

The main and web locks were unchanged through endpoint qualification. Receiving
lock review compares resolved dependency edges as well as package identities.
A temporary Cargo cohort update proposed unrelated registry edges and Murm/
NetRender revisions; that attempt was stopped and its lock discarded. The
replacement local test selects only migrated Knot and preserves the other
locked selections. Local overrides are verification aids, not publishable locks.

Windows runtime, full browser behavior and the previously bounded Distillery
fixture are not qualified by this receipt. The earlier first-cut web lock
regression was an introduced defect repaired upstream; its spareval-to-sha2
0.11 edge must remain intact.

## Final supplier and local cohort gates

`root-split-final-review` passed the same five endpoint configurations after
`1047cfc30` added observed wiping on sync and async JSON decode errors. It also
passed the full default `graphshell-web` locked Wasm check. Both repository
lockfiles remained byte-for-byte unchanged; spareval still selects sha2 0.11.0.
Log SHA-256: `91a2259dad6e2936afb5a80c523ee96fa4b2e52efdcb1ec0b1859040fdcbc89f`.

`root-split-local-cohort-2` selected migrated Knot source `77d163c` using a
temporary local patch. It passed 44 Djinn door tests (including the browser
JSON-wrapper sentinel), one custody-payload test, two custody-route tests and
five vault-lock tests. The explicit retained-process fixture remained ignored.
The new fresh custody connection while locked assertion executed successfully.
Log SHA-256: `3d309179d90a7716bb2fe00000be442948e3ba249c97793d4b9d23229db18744`.

This cohort's lock audit found only six Knot identities changing from Git to
local paths and one dependency owner outside those identities changing its
Knot edge. Other package identities and resolved edges were preserved. The
original lock was restored after the run. These results qualify this local
cohort; published receiver locks and the unpatched broker gate are still due.

## Upstream reconciliation before supplier publication

Upstream advanced to `54e09dd11` with documentation and audit artifacts only.
The reviewed series was rebased onto it. A direct comparison against retained
local branch `archive/custody-projection-reviewed-20261029` found no differences
in crates, ports, support, Cargo.toml or Cargo.lock. Review hashes above describe
the original series: `171bec593` corresponds to `66cdeb368`, `d5d5605e1` to
`972855420`, and `1047cfc30` to `3f344e2b4` in the rebased publication history.

## Published supplier cohort

The final unpatched broker gate receives Knot
`704084d064da886dd14cbe8ac3dd8758af105e1d`, Dramatis
`051e95a68011072f172b9ec4c0f04d4e5259df5b` and Murm
`1f601488f851c4e54672cd439b7c45728255ea65`. Knot's upstream `d400d0c`
introduced secure-publishing APIs requiring those Dramatis/Murm revisions;
its source and pins were preserved when the endpoint migration was rebased.
The endpoint supplier published to receivers is Mere
`c6290142866eefead385e728c5c5da24ac6f624a`.

`root-split-published-broker` passed without local overrides: 44 Djinn door
tests, one custody-payload test, two custody-route tests and five vault-lock
tests. The retained-process fixture remains explicitly ignored. This includes
the fresh custody connection while locked, unexpected/released-key responses,
and browser-wrapper JSON diagnostics. The gate used the same 3 GiB, zero-swap,
150 percent CPU limits. Log SHA-256:
`1e48a7b3cd19fdb8f326670558ec372d29e416c366b4ad7a6dc150981f2a436c`.

The root lock changes only the 26 Git package identities from those three
suppliers. Its resolved edge audit found 28 unchanged-identity owners whose
supplier targets advanced; no dependency target name or version changed.
The web lock is unchanged. Earlier local-cohort results above remain bounded
to their recorded sources; this published-cohort run supersedes their pending
unpatched broker gate.
