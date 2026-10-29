# Alembic owner workshop qualification receipts

Date: 2026-10-28. The active specification is
[the Alembic implementation plan](../../mere_docs/implementation_strategy/2026-10-28_alembic_implementation_plan.md).

## Source and scope

Worktrees: `Code/worktrees/mere-alembic-hosts-20261028` and
`Code/worktrees/turnstone-alembic-hosts-20261028`. Retained evidence is under
`/Users/markik/Code/output/alembic-hosts-20261028/`, with the earlier destination
attempts under `alembic-destination-20261028/` and receiving attempts under
`alembic-turnstone-20261028/`.

The first corpus is the authored public A/B/C example. A and B deliberately
share a title; B contains Unicode and identifies its version in the text.
Installations, subjects, grants, run namespaces and destination owners remain
independent. This is a deterministic workflow with no model invocation,
acquisition, credentials or private-source admission. Servitor remains in Eidetic.

## Initial completed integration gate

`combined-mere-tests-final.log` records the guarded Rust 1.99.1 gate:

```sh
PATH=/Users/markik/.cargo/bin:$PATH python3 /Users/markik/Code/tools/build_storage.py build \
  --lane alembic-hosts-20261028 --cwd /Users/markik/Code/worktrees/mere-alembic-hosts-20261028 \
  --reserve-gib 8 -- +1.99.1 test --locked --offline \
  -p mere-alembic -p mere-athanor -p pandect -p graphshell --no-default-features \
  --features mere-alembic/workshop-native,mere-athanor/workshop-worker,mere-athanor/workshop-destination,graphshell/alembic-workshop \
  --lib --bin graphshell-alembic --test alembic_workshop_process -j 4
```

| Target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Graphshell library | 290 | 0 | 6 |
| Graphshell native binary | 1 | 0 | 0 |
| Graphshell fresh-process workflow | 1 | 0 | 0 |
| Alembic library/native surface | 11 | 0 | 0 |
| Athanor library/driver/destination | 35 | 0 | 0 |
| Pandect library | 358 | 0 | 0 |
| Total | 696 | 0 | 6 |

The source was based on Mere `eb8abf630`. Its dependency lock was subsequently
saved in `39b598604`. The tested native preview used `pre` elements and checkmark
selection labels; the later paragraph/accessibility correction and its added
test require their own receiving gate. A later merge (`b809fbd3c`) incorporates
upstream lock updates and also requires requalification before publication.
These source distinctions must not be collapsed into a claim that the final
worktree or both products have passed every gate.

## Merged recovery and native-text gate

`merged-recovery-text-tests.log` records a later exit-0 run of the same guarded
four-package command above. It includes the upstream lock merge, explicit
installation lifecycle text, the paragraph accessibility correction and the
queued duplicate-Accept regression/fix.

| Target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| Graphshell library | 291 | 0 | 6 |
| Graphshell native binary | 1 | 0 | 0 |
| Graphshell fresh-process workflow | 1 | 0 | 0 |
| Alembic library/native surface | 12 | 0 | 0 |
| Athanor library/driver/destination | 35 | 0 | 0 |
| Pandect library | 358 | 0 | 0 |
| Total | 698 | 0 | 6 |

The resulting executable is frozen at
`graphshell-alembic-recovery-text-qualified`; its SHA-256 is
`2ed93bab65175f2a75b8a42426606f46eee36c78d001114b603b456e71ecdd57`.
`graphshell-recovery-text-binary.json` binds the executable, app bundle,
data root and relevant source-file hashes. The final native observation is
separate from the earlier executable's observation below.

The 8 GiB warm reservation follows the completed 20 GiB cold native-codegen
attempt in the same stable lane; the retained `warm-test-reservation.txt`
records the incremental scope. It is not inferred from a metadata-only check.

## Failures retained and corrected

- The destination suite first failed when replay-only node insertion supplied
  no shown Resource. Normal `AddNode` creation and an explicit staged-effect
  postcondition now cover that case.
- Revision tests caught a missing facet-sidecar distinction and a normalizer
  retaining synthesized visit timestamps. Current tests restore the actual
  sidecar, replace imported facets, preserve a sentinel visit value and verify
  the absent-visit case, equal-count divergence and cold reopen.
- Driver tests cover post-await authority/source/time checks, correlation and
  captured-input identity, installation subject identity, orphan pause/recovery,
  lost admission replies, cancellation/join, and backend ownership through an
  unwind. Pandect tests cover real commit followed by a lost acknowledgement,
  stale in-memory repair, pending/validator refusal, current publication
  generation and retained validators/launch settings.
- Graphshell owner tests cover accept, reject, exact citations, changed-source
  refusal, lost acknowledgement, retained ledger after note deletion, writer
  exclusion and result routing that does not substitute an alias surface.

Earlier red output remains in `combined-mere-tests.log` and
`alembic-destination-20261028/combined-library-tests.log`.

`queued-accept-red-filtered.log` retains a later genuine regression: one
executed test fails after a real backend transaction commits and then loses
its acknowledgement. A queued duplicate Accept returns the stored receipt,
but the live graph still has two nodes where three are required. The exact
pre-fix source is retained as `mere_host_alembic.queued-accept-before-fix.rs`.
The correction routes an already stored matching receipt through canonical
owner reconciliation and preserves a later publication's generation. Its
combined receiving gate is separate from the earlier 696-pass gate.

## Native Graphshell observation

The tested executable was APFS-cloned and hash-verified at
`graphshell-alembic-qualified`; `graphshell-binary.json` records its SHA-256.
The local app bundle uses those exact bytes and a dedicated public data root.
Through the actual native controls: load the public corpus, generate, open B's
exact retained passage, accept, and open the resulting brief. Observed usage
was 3 decisions, 2 tool calls, 0 model tokens and 269 ms for that run; this is
one functional observation, not a performance benchmark.

Native close was followed by process-exit verification. A fresh process read
the headed store with four nodes, the original receipt and an exact proposal/
result match (`graphshell-headed-cold-reopen.json`), and the native app then
reopened with Added status. `headed-observation.json` records this bounded flow.

Native accessibility exposed the controls and version labels but omitted the
visible preview text. The failing observation is retained in
`native-accessibility-red.json`. The shared surface now uses paragraph elements
with preserved whitespace and a native text regression test. The later merged
gate passes that test, and the actual corrected native app exposes B's exact
Unicode passage and the accepted brief text through accessibility. It reopened
the earlier durable result and showed the original operation identity. The
actual window also renders ASCII selection marks and an explicit Helper active
label. These observations are retained in `native-recovery-text-observation.json`;
screenshots were observed through the native tool, with no local image file claimed.

## Receiving and remaining boundaries

Turnstone's action/window adapter, source allowlist, existing graph writer,
exclusive session lease and ledger retention are implemented but have not yet
passed a receiving check. Its older Eidetic pin lacked `Journal::prefix_digest`;
the receiving manifests now select the exact published Eidetic revision used
by this Mere cut. The temporary local supplier overlay has been removed and
the Turnstone config is byte-identical to its baseline. Its canonical lock
pins Mere `3d69d032e7f7516b50879ccc14d13971cba40773`, including AccessKit, and
the matching published Eidetic cohort. Rust 1.99.1 locked/offline metadata exits
0 after the upstream Turnstone lock merge. Its known Servo malformed-fixture
diagnostic is retained separately from that exit status.

`alembic-turnstone-20261028/receiving-cut/merged-identity-audit.json` checks 162
Git packages against both lock records and actual loaded checkout HEADs, with
no identity mismatch. Against current Turnstone upstream, unrelated package
records remain unchanged except the required new `genet-events` package at
the existing Genet revision and the root Alembic dependency. A broader Cargo
update candidate that refreshed unrelated branches was retained and rejected.
These metadata facts are not a compiler or product test result.

The local receiving source also routes citations through the exact persisted
proposal and owner, shares session-state cleanup between lease and load
refusal, and drops the workshop before App's writer leases. None of those
Turnstone regressions has executed. The pre-fix lease-refusal file, fixed file,
reversible patch and exact red/green re-entry steps are retained in
`alembic-turnstone-20261028/lease-refusal-refactor-reentry.md`.

The current compiler-cache cap is 300 GiB and the free-space floor is 100 GiB.
Repeated active-lane/admission refusals delayed the final preview gate and
prevented receiving attempts from reaching Cargo. The final Mere gate above
subsequently ran successfully; Turnstone remains separate. No cap increase has
been applied. The latest canonical Turnstone attempt was refused because the
300 GiB compiler-cache budget itself would be exceeded, before Cargo ran.
Source, native
window, OS containment, private custody, real Persona authority, provider/model,
remote fleet and full two-host W3 qualification remain separate claims.

For the eventual Turnstone native gate, its existing place-sharing `Resident`
recipe supports a fresh isolated test vault and private endpoints, with an
in-memory random passphrase passed on stdin. An existing Djinn lane artifact
is APFS-cloned and hash-verified as
`alembic-turnstone-20261028/djinn-native-public-fixture-candidate`; its candidate
manifest explicitly records missing historical build provenance. No daemon
was started, user vault accessed or model loaded. Actual protocol and native
qualification remain required before relying on that candidate.

The local build guard has a separately tested optimization: reuse an unchanged
preliminary inventory while retaining the fresh admission inventory. All 51
temporary-workspace Python tests pass (`guard-review/live-tests.log`); the
policy values and active-cache protections remain in force. A more aggressive
lock-name filter was rejected after an isolated race showed it could miss a
newly created profile lock. No real cache cleanup was performed by those tests.

## Documentation audit boundary

`doc-judgment-audit.log` reports 226 of 264 active documents covered, with an
aggregate-digest mismatch and 38 missing records. None of the missing paths is
the workshop plan or harness brief touched here. The audit script and snapshot
aggregate have byte-identical hashes to `origin/main`, recorded in
`doc-audit-baseline-comparison.json`. This is a retained baseline failure; the
workshop change does not repair or claim full documentation-audit coverage.


## Private-source priority and intermediate custody evidence

Mark prioritized private-source support on 2026-10-28 before further workshop
UI completion. The authorized workspace compiler-cache ceiling is now 400 GiB;
the 100 GiB free-space floor, age rules, active-cache protections and retained
receipts are unchanged. The workspace guard's 51 Python tests pass; policy
before/after and authorization evidence live under
`Code/output/alembic-hosts-20261028/cache-policy-400-20261028T204852Z/`.

This checkpoint does not qualify a private native workflow or production
custody. New sealed capture, host retirement and destination integrations are
still receiving work. No real-store migration or production installation was
performed.

- `private-core-destination-tests.log`: 40 Athanor tests passed before the later
  Resource-only reader and helper-journal-deletion test were added. It covers
  the authenticated sealer adapter, whole-record ciphertext CAS/reopen and the
  initial private destination. This is library/fixture evidence.
- `custody-payload-nested-red.log`: the provider payload-sealer call from Tokio
  executed and failed before the endpoint repair. Setup failures remain separate.
- `final-endpoint-tests.log` / `.exit`: 55 endpoint tests passed after the bounded
  startup deadline and owned stalled-watch shutdown changes. The production-only
  endpoint/Djinn library check passed in `final-production-check.log` / `.exit`.
  Both results precede the later custodian Core and wire repair below.
- `endpoint-prod-feature-tree-normal.log` and `djinn-prod-feature-tree.log`:
  normal/build dependency inspection excludes endpoint dev dependencies and
  Djinn's in-process custody; `door` supplies `tokio/macros` explicitly.
- `custodian-generation-red.log`: setup failure, no regression executed; the
  login shell selected non-rustup Cargo and rejected the toolchain argument.
- `custodian-generation-red-retry.log` / `.exit`: genuine pre-fix run, five
  executed tests, three passed and two failed. A profile switch away and back
  failed to advance generation; exhausted generation still allowed unlock.
  Initial regression sources are retained in `custodian-exhaustion/`.
- `final-private-libraries.log` / `.exit`: the post-fix custodian, wire and
  Athanor batch was refused before Cargo after waiting for build admission.
  It supplies no green result. Source now adds the profile-change fact, pending
  consent invalidation, and permanent exhausted admission with session closure.

The new `ProfileChanged` fact retains wire version 1. An older strict receiver
disconnects on that fact and must reconnect; current cohort tests must qualify
folding the new generation and closing authority on loss. This is a deliberate
compatibility boundary, not a claim that old decoders understand the new fact.

The added Djinn tests use the real custodian implementation over socket IPC,
served by a test thread over a synthetic vault, including the app broker and
provider path. They are not separate-process, native-product or H1 production
qualification. These tests have not executed at this checkpoint.

The private destination retains only encrypted content plus stable routing and
content identities. Its Resource-only reader no longer needs an active helper
or helper journal. The expanded test batch will additionally exercise reading
a saved private result after helper dissolution and journal deletion, using an
actual authenticated cipher. Content-hash equality/confirmation, existing browser
addresses/titles and RAM erasure remain outside this storage claim.


## Integrated private library and portability cut

The implementation checkpoint is `38af485fb`; the already inspected upstream
cut `0740ea10a` merged into `fa4493236ebb30f071a4846a3f6d3960c3724e3a`.
That merge changes Mesh's injected sync handle/dependency and documentation;
it selects no new supplier revision. Locked offline metadata passes.
The candidate is published on `codex/alembic-hosts-20261028` for receiving
validation; it is not a private-product or main-integration receipt.

`integrated-libraries.log` / `.exit` records the guarded combined gate on that
cut, with a warm 8 GiB reservation:

```sh
PATH=/usr/local/opt/rustup/bin:$PATH python3 /Users/markik/Code/tools/build_storage.py build \
  --lane alembic-hosts-20261028 --cwd /Users/markik/Code/worktrees/mere-alembic-hosts-20261028 \
  --reserve-gib 8 -- +1.99.1 test --locked --offline -p djinn-custodian -p custody-wire \
  -p mere-athanor --lib --no-default-features \
  --features mere-athanor/workshop-worker,mere-athanor/workshop-destination,mere-athanor/workshop-sealed -j 4
```

The command ran through `Code/tools/build_storage.py` in the existing
`alembic-hosts-20261028` lane. Its HEAD and source SHA-256s are retained in
`integrated-libraries.head` and `.sha256`.

| Target | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| custody-wire | 9 | 0 | 0 |
| djinn-custodian | 56 | 0 | 0 |
| Athanor | 41 | 0 | 0 |
| Total | 106 | 0 | 0 |

The expanded gate includes the profile round-trip fix, exhausted admission and
session closure, generation-fact handling, and reading the encrypted human-owned
result after helper dissolution and journal deletion.

`integrated-workshop-wasm.log` / `.exit` passes the workshop-only Athanor check
for `wasm32-unknown-unknown`, with defaults disabled and no worker or sealing
feature. `integrated-production.log` / `.exit` passes the endpoint/Djinn
production library check with endpoint `door` and Djinn `personal-sync,cxf`,
without test-only feature unification. Both reserve the original 6 GiB check
allowance, and retain HEAD/source hashes.

The named Djinn IPC integration has not executed: `integrated-djinn-ipc.log`
records a capacity refusal; `integrated-djinn-ipc-retry.log` records an admission
lock timeout. These are not failing tests. Later invocations use a validated
policy copy whose only difference is admission wait 300 to 900 seconds;
`admission-wait-900-receipt.json` binds it to the unchanged global 400 GiB
cache budget, 100 GiB floor and all cleanup/reservation protections. This
queue-wait change cannot admit a build that fails capacity or safety checks.

## Turnstone candidate receiving identity

`Code/output/alembic-turnstone-20261028/private-receiving-cut/` retains the
before lock/manifest/config, Cargo's incidental update, the restored lock and
identity audit. The selected 62 Mere package records point to `fa4493236`.
All 1,599 nonselected package records are unchanged, including the prior
Genet, Murm and Netrender cuts; Cargo's incidental moving-head refresh was
not adopted. Locked offline metadata passes and all 162 Git package checkout
HEADs match their lock source identities across 21 checkouts. The Cargo
configuration is unchanged. The receiving lock's SHA-256 is
`232f7dcc9ec825a4662aa74d63cdef04a5774c30a54acd829e163ea93ab72f22`.

Turnstone's first check reached rustc and exposed host API/type mismatches.
Those diagnostics were repaired in source; subsequent admission refusals do
not qualify that repair. Host tests, native private controls, responsiveness,
lock/save races, the source picker and the full W3 matrix remain open.
Private binding requires the new authoritative-epoch resident; an older
resident without that capability leaves the private identity pending.


The helper-journal deletion test retains the wallet seal epoch. It does not
qualify key retirement or rewrapping. Binding generation changes revoke current
access without deleting retained encryption keys; the private operation ledger's
stored-ciphertext immutability guard requires an admitted migration before those
ciphertexts can be rewrapped. Helper dissolution alone is no evidence that the
human-owned result's encryption key is unused.


## Held consumer checkpoint and current capacity boundary

Turnstone's complete source checkpoint is `ce5b6ae9a667a6a65baebb9e3c340dd0c2bba3c2` on its isolated
`codex/alembic-hosts-20261028` branch. It is committed locally, not merged or
published. `private-source-checkpoint.json` records every owned changed file's
SHA-256 and the selected lock hash. The later source fixes include receipt
ownership, live-callback library admission/publication/read fences, stale-enable
coverage, and redacted library Debug output. These latest sources have not
completed a compiler or semantic test gate.

The compiler did execute in `check-fa449-scope-fixes.log` and
`check-fa449-policy900.log`, reporting host integration errors and then one
remaining ownership error. Those were fixed afterward. `check-fa449-fixes2.log`
and `check-fa449-final.log` record guard refusals before Cargo; unrelated rustc
processes are not evidence for this lane. The genuine lease-red procedure is
preserved in `lease-refusal-refactor-reentry.md`, using current targeted fixed
and red snapshots rather than overwriting later changes with a historical file.
The lease red still has not executed.

The remaining shared IPC attempt is also unexecuted. `integrated-djinn-ipc-wait900.log`
and `integrated-djinn-ipc-after-release.log` record capacity refusals. Read-only
verified lease snapshots briefly showed zero active reservations before a new
launch; other reservations appeared by admission. The 8 GiB test reservation,
400 GiB cache budget, 100 GiB floor and cleanup protections were not reduced or
bypassed. Main integration remains held for the required integration evidence.

## 2026-10-29 private owner receiving continuation

The selected dependency cut remains the earlier inspected cohort. New upstream
Mere and Turnstone supplier changes have been inspected, but adoption is held
for the requested exact-cut decision. None of the following results qualifies
those newer supplier cohorts or main integration.

The named Djinn integration gate finally executed. `20261029-djinn-ipc.log`
ran eleven client tests, with ten passes and a test-setup refusal: the fixture
correctly refused an ungranted root-public release before the payload assertion.
The test now uses the already-admitted status master and checks it against the
synthetic fixture vault, without adding a grant. The intervening
`20261029-djinn-ipc-fixed.log` stopped before tests because the new Graphshell
dev dependency required its lock entry. Offline metadata updated only the
Graphshell dependency list to include existing `chacha20poly1305 0.10.1`;
no package or supplier source changed.

`20261029-djinn-ipc-fixed-lock.log` / `.exit` passes: eleven client tests and
one payload test, zero failures and one existing ignored receipt helper. The
provider/app-broker path and authoritative generation/lock cycle execute here.
These sockets are served by test threads, not separate custody processes.

Graphshell now uses its canonical MereHost for encrypted source Resource
retention, the sealed workshop journal and private result publication. The
independent Pandect operation record stores a commitment rather than a clear
request. Private operation evidence and note Resources share the original
journal/publication transaction; its admission validator preserves retained
operation facets across normal writer/history candidates.

`20261029-graphshell-private-tests.log` records 294 passes, six existing ignores
and three failures. Two are genuine regressions: a retired-owner DTO preserved a
queued private source action, and the normal writer could remove encrypted
operation evidence. The other failure is a test-setup lifetime error: its helper
journal deletion handle was still open during reopen. The red executable and
source snapshots are frozen in `20261029-graphshell-private-red/`, with matching
binary SHA-256s. The fixed source makes owner retirement terminal in the UI,
clears queued actions, installs the graph admission guard and closes the test
handle before reopening.

`20261029-graphshell-private-fixed-tests.log` / `.exit` passes the same guarded
command, using the original warm 8 GiB reservation:

```sh
PATH=/Users/markik/.cargo/bin:$PATH python3 /Users/markik/Code/tools/build_storage.py \
  --policy /Users/markik/Code/output/alembic-hosts-20261028/admission-wait-900-policy.json \
  build --lane alembic-hosts-20261028 \
  --cwd /Users/markik/Code/worktrees/mere-alembic-hosts-20261028 --reserve-gib 8 -- \
  +1.99.1 test --locked --offline -p graphshell --no-default-features \
  --features alembic-workshop-private --lib --bins --test alembic_workshop_process -j 4
```

The library has 297 passes, zero failures and six pre-existing ignores; the
native service test and fresh-process public recovery test each pass. The
private owner tests use authenticated synthetic ChaCha20Poly1305 protection
and actual DirectoryBackend/GraphSession persistence. They cover cold reopen,
exact Unicode source reading, lost acknowledgement, helper dissolution and
journal deletion, Resource-only result reading, custody loss/terminal retirement,
wrong owner, source collision/range/tamper refusal and cleartext canary scans.
They do not yet qualify the current production custody route or headed private
controls.

`graphshell-alembic-private-20261029` is an APFS clone of the built executable,
SHA-256 `54f052f2906520dd01c7a5c0c7c9fc266aeb7d3c610f53d064017df866e7979e`.
`graphshell-private-20261029-binary.json` and
`20261029-graphshell-private-fixed-source.json` bind it to exact source hashes.
The explicitly enabled private CLI uses a supplied app-broker endpoint and
requires an authoritative unlocked custody epoch; an old resident or failed
connection cannot select plaintext storage. Its private probe prints only
hashes/state, unlike the historical public probe's readable public fixtures.

Turnstone's compiler check passed before the later sealed chooser and quiet
source-loading additions. Those additions and facet error propagation remain
unqualified. Its reserve-20 lease-red attempt was capacity-refused before Cargo;
the fixed source was restored byte-for-byte and no red test executed. The 400
GiB budget, 100 GiB floor and running-cache protections remain unchanged.
Production custody and real-consumer qualification flags remain false.

## Separate-process custody fixture compiler checkpoint

`ports/djinn/examples/private_workshop_fixture.rs`, enabled only by
`custody-receipt-host`, creates a new synthetic vault and wallet in a dedicated
custodian child. Its parent uses the real `RemoteHost` and serves the normal
app-broker route; receiving products are separate processes. Readiness contains
only process IDs, the explicit socket, a synthetic profile and a false production
qualification flag. Standard input supplies lock, unlock, status, custody-loss
and quit controls. Unknown commands are redacted, and normal exit or input EOF
stops and reaps the owned child. These are source facts, not executed lifecycle
evidence at this checkpoint.

The guarded compiler gate passes in
`Code/output/alembic-hosts-20261028/20261029-private-fixture-check.log` / `.exit`:

```sh
PATH=/Users/markik/.cargo/bin:$PATH python3 /Users/markik/Code/tools/build_storage.py \
  --policy /Users/markik/Code/output/alembic-hosts-20261028/admission-wait-900-policy.json \
  build --lane alembic-hosts-20261028 \
  --cwd /Users/markik/Code/worktrees/mere-alembic-hosts-20261028 --reserve-gib 6 -- \
  +1.99.1 check --locked --offline -p djinn --features custody-receipt-host \
  --example private_workshop_fixture -j 4
```

The retained HEAD/source hashes accompany that log. The native example build
has not run; its 20 GiB code-generation reservation remains unchanged. The
process smoke driver, `qualify-private-workshop-fixture.py` in that output
directory, is syntax-checked but unexecuted. It checks actual broker/custodian
parentage, lock/unlock generation changes, unknown-input redaction, custody loss
and owned-child reaping using a frozen executable. The checkpoint JSON records
these distinct gates. No current private product, installed identity, signing,
physical-platform or H1 production qualification follows from the compiler pass.
