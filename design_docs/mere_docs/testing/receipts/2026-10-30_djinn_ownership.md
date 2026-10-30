# Djinn ownership extraction, 2026-10-30

**Status:** ownership extraction and consumer migrations published. The table below
records exact qualified commits. Subsequent unrelated Mere graph work is preserved
in this receipt commit; these gates describe the pinned ownership cohort.

## Ownership

The independent `djinn` package lives beside `armillary` and `apparatus-core`.
It owns the finite application envelope, bounded framing, native connection,
selected custody clients, shared local nonce binding, application admission
mechanism and resident task/service shutdown orchestration. Its default features
are empty. Armillary itself gains no product dependency.

Mere's existing `ports/djinn` package is now `mere-resident`; executable names
stay unchanged. It constructs Knot, graph/reservoir, Distillery/inference,
published-site and other product services. It supplies route policy and custody
authority, preserves the provider leases and projection validators, and closes
its resources after Djinn has stopped services and joined tracked children.
Custodian/Castellan remains a separate protected authority using custody-wire.

Graphshell endpoint keeps projection clients/carriers, cards/resources/notices,
provider adaptation and the Mere proxy. Neutral imports now go directly to
Djinn. Identity views go directly to Dramatis, including tests and external
receivers; the convenience identity re-export is removed.

The implementation plan and supplier evidence live in
`armillary/design_docs/armillary_docs/implementation_strategy/2026-10-30_djinn_ownership.md`
and `armillary/support/receipts/djinn-ownership-20261030.json`.
The new extraction follows the separately published provider-offer delivery:
[provider offer receipt](2026-10-30_app_projection_offers.md).

## Verification

| Boundary | Observed result |
| --- | --- |
| Independent Djinn native suite | 70 tests passed |
| Independent portable/native/custody DTO/custody client/resident fixtures | Five nonzero tests, four negative API checks, portable Wasm passed |
| Graphshell endpoint projection/profile/provider and exact wire fixtures | 11 + 4 + 1 tests passed |
| Graphshell native host | 101 tests passed |
| Mere resident door | 56 tests passed |
| Provider offers, separate custodian, custody route, embedded Knot route, vault lock doors | 4 + 11 + 2 + 1 + 5 tests passed |
| Mere resident all-targets and no-default library | Passed against the isolated source cohort |
| Published-pin Mere resident and station-custody all-targets | Passed |
| Published-pin provider/custody/Knot/vault integration suites | 12 tests passed |
| Independent combined resident/client fixture | One test passed without workspace feature unification |
| Published-pin adapter feature matrix | Six fixture tests, negative API checks and portable Wasm checks passed |
| Graphshell MCP door, Overmap resident, Dramatis app and Graphshell Mere adapter | All-target checks passed |

These are focused suites. The combined local regression run reached its
600-second limit during the subsequent station-custody check; all preceding
commands above recorded success. Its timeout remains retained. Published-pin
receiving checks and the independent combined-client fixture have separate
receipts; a copied lock or successful metadata resolution is not a build result.

Independent supplier normal/build package-name cones, including each fixture:
portable door 16, native connection 61, custody DTOs 113, custody client 122,
resident lifetime 18. They contain no Mere, Knot, Graphshell, scenes, Hest or
Distillery product packages. Personae remains in native discovery for the existing
fallback socket path; this is an explicit residual edge. Direct dependency edges
and feature selections are retained with the isolated metadata.

## Review and preserved behavior

Independent review caught two extraction regressions before delivery: product
maintenance could be cancelled during a lock/refresh action, and a session's
surface snapshot moved past challenge/connect. The fixes preserve completion of
a selected maintenance action and the original pre-challenge snapshot boundary;
focused rereview found no further issue. A separate consuming check caught two
test imports still using the moved browser-specific accept method. The provider
runner tests now use a real local connection handshake, without exposing a
production constructor only for tests.

The frame limit remains 1 MiB, native-endian framing and endpoint/env/schema
strings are unchanged, and JSON errors retain safe metadata only. Existing owned
frame buffers are guarded before serialization/read. That guarantee excludes
Serde scratch, allocator copies, caller payloads and kernel buffers. Custody
remains available while locked. Application labels remain self-declared within
the same-user transport boundary, not executable authentication. Host grants,
mandatory projection admission, epoch/cache invalidation and provider lease
ordering/cancellation remain in the exercised paths.

Windows runtime, browser IPC and full graphical operation are not qualified by
these tests. Portable Wasm checks cover portable contracts. The relationship fixture has an upstream Inker document-session type mismatch
outside the changed source; the baseline and migrated metadata have the same two conflicting dependency
identities. Baseline compilation has not been independently observed. It remains an unqualified receiving target; Woodshed and Hocket's actual all-target builds
against published Djinn passed.

## Retained evidence

The [structured qualification receipt](../../../../support/receipts/djinn-ownership-20261030/qualification.json) records source hashes, commands, log SHA-256 values,
adapter feature cones, actual Git checkout heads and the final lock dependency
edge audit. Registry selections are unchanged. Root lock changes are limited to
the new owner/package identity, Armillary and Knot source intake, and the eight
consumers whose direct edges changed. Heavy checks used the shared machine lock,
3 GiB memory cap, zero swap, one Cargo job, one test thread and 600-second
execution limit. The retained timeout remains a failed overall gate, with its
completed subcommands identified separately.

## Published receiving cohort

| Repository / packages | Published commit | Receiving result |
| --- | --- | --- |
| Armillary / independent Djinn | `721e17d41a8f258073f440adbbb39ad145250868` | Native and isolated feature fixtures passed |
| Mere / mere-resident and Graphshell adapters | `675540865430958085da67155188be9ca1aa37ff` | Focused tests and consuming targets passed |
| Knot | `055e1f4e54e8f2539673d6c9ed2ab273478a1c0d` | Four provider tests and peers check passed |
| Woodshed / Hocket | `b790b3c1f9c4a08a599e5ad746c39134ee22810a` | Both all-target checks passed |
| Turnstone | `720f59f885545b93d81070958ac92ce755b99bad` | Authoring-enabled all-target check passed |
| Cleromancy | `6ec7e34774d9e12bcf5f21969ec883dd059479bd` | Default all-target check; four resident and eight domain tests passed with default features disabled |

The [receiving receipt](../../../../support/receipts/djinn-ownership-20261030/receiving.json)
records commands, log hashes, source hashes and retained audit locations. These
final receiver gates use published Git suppliers without local path overrides.
The earlier default-feature Cleromancy focused test reached its 600-second limit
while compiling; no test ran in that attempt. Its failed receipt is retained.

Mere's root lock retains Knot's published implementation commit `072ff2ca`; the
final Knot receiving commit changes only its lock and an unused patch entry.
Turnstone and Cleromancy lock the final Knot commit. Their existing Genet and Netrender cohorts were retained. Dramatis resolves
consistently to their existing `051e95a6` cohort, including correction of split
lock identities in Knot; Murm intake is required by Knot's current
transport-security API. Stale unused Genet Taffy patch entries were
removed where they prevented resolution; the selected registry package stayed
unchanged. Isolated worktrees and failed gate logs remain preserved.

Cleromancy's allocator lock hash changed from `01d3f528` to `565458ea`, but
baseline and final locked metadata both resolve the actual `565458ea` checkout.
The package subtree tree ID is identical at both commits:
`0cdb33d47adf5e74dfc4d0f9e0b2609598c67e78`. This is a corrected stale lock
identity, not an allocator implementation upgrade; the receiving receipt names
the retained baseline/final audit.
