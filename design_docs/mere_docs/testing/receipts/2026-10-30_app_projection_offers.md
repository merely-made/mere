# Application projection offers, 2026-10-30

Source base: Mere `59d87099c`, after fetching and incorporating documentation-only
upstream changes over the qualified code base `56fbdc2e4`. This extends the reviewed custody/projection feature boundary.
The source hashes, commands, resource caps and verified log SHA-256 values are in
[qualification.json](../../../../support/receipts/app-projection-offers-20261030/qualification.json).
Retained executor logs live under `/home/markik/Code/testing/custody-contract-20261029`.

## Behavior

An application offers one route over the existing shared app handshake and typed
Call/Answer slots. Djinn admits it only under explicit owner policy; viewers have
separate permissions, with explicit denials preserved. Provider permission adds
no custody privilege. The host retains projection admission and authority checks.
The provider creates separate endpoint instances for viewers.

The native integration uses arbitrary provider/viewer labels and a real Knot
directory, without an app-specific Djinn registration. It reads a snapshot and
resource, checks provider/viewer denial, revokes live sessions, observes provider
disconnect and replacement, and checks raw idle-viewer EOF. A provider capacity
refusal leaves its existing viewer usable. The standalone Knot CLI intake is
pending publication of this supplier; this receipt does not yet qualify that binary.

## Verification

| Gate | Result |
| --- | --- |
| Endpoint native provider unit tests and exact wire fixtures | 35 + 4 passed |
| Graphshell host native library | 101 passed |
| Djinn door library, final frozen source | 56 passed |
| Djinn owner settings | 25 passed |
| Real app provider integration | 4 passed |
| Existing custody route, embedded Knot route, vault lock doors | 2 + 1 + 5 passed |
| Isolated feature consumers | Six tests passed, six expected missing-API checks |
| Portable door, offer DTOs and host on Wasm | Compile checks passed |
| Djinn default all-targets | Passed |
| Djinn no-default-features library | Passed |

These are selected suites, not a claim that the entire workspace passed.
All listed workspace tests were rerun against upstream’s new Dramatis/Murm cohort
and passed in `root-offer-upstream-requalification`. That combined run reached
the 600-second cap during the final all-targets check. Both remaining checks and
the refreshed provider fixture passed in `root-offer-upstream-remaining`; the
limits were unchanged. The subsequent `59d87099c` advance changed only documents.
The all-targets command passed inside `root-offer-consuming-targets`; its next
command requested an unsupported no-default-features `djinn` binary, which Cargo
refused because that target requires `personal-sync`. The corrected library gate,
settings tests and integration regressions passed in `root-offer-consuming-remaining`.
No source defect is inferred from that invalid test command.

The initial catalog/DTO reds were missing-API compile failures. The settings red
was an observed unknown-field rejection. The runtime offloading regression was
observed failing before the fix and passing afterward. Other added tests are
positive acceptance evidence, not claimed pre-change behavioral reds.

Native normal/build package-name cones: custody client 122, custody DTOs 113,
projection client 66, provider client 66, portable door 16, portable offers 36,
portable host 35. Wasm offers use 35 and portable host 34. Counts are dependency
names, not executable size. The custody cones exclude Chirograph/scenes/Hest and
graph products. Provider/projection direct edges exclude custody-only suppliers;
Personae remains for existing endpoint discovery. Compiled feature selections and
negative API checks are retained with the metadata. The new provider fixture pins
Dramatis `6aa0daa4`, matching this workspace; older fixtures retain their existing
locked revisions. The workspace lock changes only three stale source identities
(chatelaine, dramatis and gaz) to the already-resolved Dramatis cohort. Every
upstream registry dependency selection is retained; the edge audit is retained
in `testing/app-projection-offers-20261030/cohort-reconcile-edges.json`.

All heavy gates use the shared machine lock, MemoryMax=3G, zero swap, 150% CPU,
one Cargo job, one test thread, and the existing ten-minute per-run cap. Lock wait
failures and superseded preliminary runs remain retained and are not green gates.

## Review and limits

Independent review found work-ID ordering, queue-full cleanup, notice deadlines,
runtime blocking, explicit provider denial, idle viewer cancellation and malformed
reply withdrawal issues. These were fixed before the final gates. The focused
re-review found no remaining blocker. Cancellation drops partial frames with their
connections; cleanup is generation checked, so an old lease cannot remove a replacement.

Windows runtime, full GUI behavior and distributed transports are unqualified.
Wasm compilation qualifies portable contracts, not browser IPC. Already-executing
application side effects cannot be rolled back; synchronous provider work finishes
before its cancelled worker can drop the endpoint instances. Ordinary viewer grant
updates retain their previous next-connection semantics; provider-policy replacement
cancels active leased sessions. Embedded Knot lifecycle and background duties remain
in Djinn. This is projection registration, not the full app-manifest/custody migration.
