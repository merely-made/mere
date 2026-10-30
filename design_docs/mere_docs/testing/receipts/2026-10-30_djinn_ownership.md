# Djinn ownership extraction, 2026-10-30

**Status:** Mere receiving gates passed against published suppliers. Independent
Djinn is published at Armillary `721e17d41a8f258073f440adbbb39ad145250868`.
External receiver publication is recorded in a follow-up receiving receipt.

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
