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

## Explicit private payload scopes for both product hosts

The actual native Graphshell process first refused at sealed startup. The
failure is retained in `20261029-graphshell-private-process-v1.json`: the
broker's payload policy admitted only the `turnstone` app label. The frozen
fixture itself passed process parentage, lock/unlock generation, unknown-input
redaction, custody-loss reaping and clean-exit checks in
`20261029-private-fixture-process-v1.json`. Its frozen SHA-256 is
`d52de97a089a1a74664283c6fb2f6e0651814ddaa42b16c12394785ff1f68839`.
The failed startup's sixteen regular retained files were separately scanned in
`20261029-graphshell-private-process-v1-no-clear.json`; no source canary was
stored and no plaintext fallback occurred. This is a negative observation,
not a successful private workflow receipt.

`20261029-two-app-payload-red.log` / `.exit` is a genuine pre-fix regression:
one test executed and failed at the required Graphshell seal operation with
`Refused(NotServed)`. The source and pre-fix policy are retained beside it.
The frozen red test executable is `20261029-two-app-payload-red-binary`,
SHA-256 `089682c13295ebd7fce49eb6b58e533d79662ba561599520138e87990eae66e4`.

The broker now explicitly admits the two app labels. The existing encoded
Turnstone AAD, `turnstone.retained-clips/custody-v1`, remains exact; Graphshell
uses `graphshell.retained-content/custody-v1`. Every other label remains
refused. The resulting `20261029-two-app-payload-green.log` / `.exit` passes
the named `custodian_client` and `custody_payload` targets: twelve passes,
zero failures, one existing ignored receipt helper. Both commands use the
unchanged warm 8 GiB reservation and retain source/lock hashes. The expanded
crypto test checks the Graphshell round trip, both cross-app decryption
refusals, compatibility with independently produced legacy Turnstone-scoped
wallet ciphertext, and the existing profile/master/hash/tamper/lock and
retained-epoch cases.

This proves domain separation for the admitted application label. It does not
authenticate an executable identity or qualify H1 installation. Private
ciphertext is not automatically portable between these scopes; an admitted
transfer/rewrap is separate. Generic legacy decoding of an absent custody
epoch remains unchanged, while the new Graphshell consumer requires an
authoritative epoch and sends its exact binding. Production custody and
real-consumer qualification flags remain false.

## Native private Graphshell receiving through separate custody processes

The repaired broker is source commit
`746a4d4e2d785fb22b467e90efc86039338d2d6f`. Its guarded native fixture build
passes in `20261029-private-fixture-build-v2.log` / `.exit`, using the unchanged
20 GiB reservation. The exact command is the earlier fixture compiler command
with `build` replacing `check`; HEAD, Cargo lock and changed source hashes are
retained in the matching `.head` and `.sha256` files. No supplier revision or
source identity was changed for the app-scope repair.

The APFS-cloned fixture executable is
`Code/output/alembic-hosts-20261028/private-workshop-custody-fixture-20261029-v2`,
SHA-256 `7e7ff63d16dc9b67453af8375cf2b4bfc25739e1566d4e1aa0b9ce22ac326e3f`.
`20261029-private-fixture-binary-v2.json` retains that build provenance.
The unchanged Graphshell consumer remains SHA-256
`54f052f2906520dd01c7a5c0c7c9fc266aeb7d3c610f53d064017df866e7979e`.

`20261029-private-fixture-process-v2.json` passes the fixture lifecycle smoke:
actual broker/custodian parentage, authoritative lock/unlock generation
advance, redaction of an unknown control input, acknowledged custody loss,
owned-child reaping and clean broker exit. Consumers open the broker from
independent native processes; the custodian service is a separate child and
the broker uses `RemoteHost`, with no in-process custody fallback on this path.

`20261029-graphshell-private-process-v2.json` passes the private product owner
flow using the same frozen executables and a new isolated root:

- Retain three sealed private sources, generate the brief and accept one note.
- Cold-reopen independent consumers with the same proposal digest, ordered
  source versions, original receipt and matching retained note/body digest.
- Refuse reading while custody is locked, then recover the same result through
  a fresh unlocked binding.
- Dissolve the helper and cold-open the same human-owned result, retaining
  exactly one added note.
- Refuse startup/reading after custodian process loss, without plaintext
  fallback.
- Scan all 33 regular persisted fixture/product files for the private source
  canary; it is absent. Probe stdout/stderr contain only hashes/state, with no
  clear canary.

The library test separately qualifies reading after helper journal deletion;
this native flow dissolves the helper without deleting that journal. The native
process run does not establish headed controls, an existing consumer's live
window across lock/unlock, signed executable admission, real-store migration,
physical-platform behavior or H1 production qualification. Production custody
and real-consumer qualification flags remain false. Failed v1 evidence and
both fixture executables remain retained.

## Terminal retirement before queued native requests

The native Graphshell `after_wake` receiver used the ordinary `handled >= sent`
condition for every owner update. A terminal `owner_retired` DTO carrying an
older handled serial could therefore wait behind a newer queued UI request,
despite the shared Surface's correct one-way retirement behavior. The actual
native update-application seam is now extracted as `deliver_surface_update`;
its initial serial-only behavior was kept for the executed regression.

`20261029-native-retirement-serial-red.log` / `.exit` executes three native bin
tests: two controls pass and the retirement case fails. The test queues a
private source-bearing action and retained reading behind a newer sent serial,
then delivers terminal custody loss with an older handled serial. The failure
is the actual receiver suppressing that observation. A healthy stale-response
control passes. The frozen red harness is
`20261029-native-retirement-serial-red-binary`, SHA-256
`86f0d3c196c0b25dd8841b10664f5837913ecdbb743ca2e11e033c11143cd01d`;
its source snapshot and matching hashes are retained beside the log.

The receiver now admits terminal retirement regardless of the ordinary serial
condition. `20261029-native-retirement-serial-green.log` / `.exit` passes all
three bin tests and the existing fresh-process public recovery test. The
regression additionally proves clearing of the private reading, source choices
and queued action; a later serial-valid healthy DTO and another local source
activation cannot revive the retired window. Healthy stale-response suppression
still preserves the newer request and its busy state. No shared-library source
changed, so the prior library gate was not repeated.

```sh
PATH=/Users/markik/.cargo/bin:$PATH python3 /Users/markik/Code/tools/build_storage.py \
  --policy /Users/markik/Code/output/alembic-hosts-20261028/admission-wait-900-policy.json \
  build --lane alembic-hosts-20261028 \
  --cwd /Users/markik/Code/worktrees/mere-alembic-hosts-20261028 --reserve-gib 8 -- \
  +1.99.1 test --locked --offline -p graphshell --no-default-features \
  --features alembic-workshop-private --bin graphshell-alembic \
  --test alembic_workshop_process -j 4
```

The integration target rebuilt the normal executable. Its APFS-frozen copy is
`graphshell-alembic-private-20261029-retirement-v2`, SHA-256
`101b781efde0e02e9cc46c44763361e6c5226f1436d1523353dc22ea46816e62`.
`20261029-graphshell-retirement-v2-binary.json` binds the executable to the exact
native bin source hash and the green gate. The older frozen consumer, its app
bundle and existing runtime were not modified.

`20261029-graphshell-private-process-retirement-v2.json` passes the complete
earlier private process qualifier using this new consumer and the unchanged
fixture v2 (`7e7ff63d…`): three retained sealed sources, one accepted note,
matching cold proposal/source/receipt/note digests, refusal while locked,
fresh unlocked reading, retained reading after dissolution, and refusal after
custodian loss. All 33 regular persisted files and probe stdout/stderr remain
free of the private source canary. This process flow does not exercise the
native event loop; the executed receiving-seam regression supplies the ordering
proof, and headed live-window acceptance remains its separate gate. No H1 or
real-consumer qualification flag is lifted.


## Native private Graphshell controls and live custody retirement

The headed gate uses the same separate broker/custodian fixture v2 and an
isolated application bundle/root, with no default profile or model provider.
`20261029-graphshell-private-headed-v1.json` records the visible three-source
selection, brief generation, exact Unicode citation, accepted note and cold
reopen in a new native process after helper dissolution. Full accessibility
observations confirm that locking custody clears the source choices, proposal
and retained reading; unlocking does not revive the retired window.

After the receiving-order repair, the bundle's consumer was replaced with an
APFS clone of `graphshell-alembic-private-20261029-retirement-v2` and its hash
verified as `101b781efde0e02e9cc46c44763361e6c5226f1436d1523353dc22ea46816e62`.
`20261029-graphshell-private-headed-retirement-v2.json` records visible reading
of the retained Unicode note on that exact binary, then live custody lock at
generation 5 and unlock at generation 6. The complete accessibility tree no
longer exposes the private reading or source choices after lock, and remains
closed after unlock. All 33 regular retained fixture/product files and the
ordinary app log remain free of the private source canary. Closing the private
app was verified through process inventory. Earlier receipt and executable
paths remain preserved.

This establishes the native controls and live window gate on one Mac using
synthetic custody and fixture sources. It does not qualify installed executable
identity, real-profile migration, physical input/IME/accessibility hardware,
production custody, cross-machine transfer or a model inference provider. The
two application hosts remain a separate acceptance boundary: Turnstone's
private native receiving qualification is still open.


## Turnstone private receiving, regression repairs and native gate still open

Turnstone candidate `fcf8a0d06469aef5512b3e2688c532636f407284` is pushed to
`codex/alembic-hosts-20261028`. It receives Mere
`bd5ec7eb51eaafa409e64d8b8987b05ad0f24eca`: the source-aware audit advances
only the 62 Mere lock records and preserves all 1,599 nonselected records.
Locked offline metadata succeeds; all 162 Git packages match their actual
checkout revisions across 21 source identities. The receiving manifest and
local composition configuration are unchanged. These audits live in
`Code/output/alembic-turnstone-20261028/bd5-receiving-cut/`.

The session lease refusal regression has a genuine executed pre-fix failure
and repaired pass. The red left the departed session's Place state live after
writer admission failed. The repair routes both lease and load refusal through
the same cleanup. `private-source-test-receipt-20261029.md` distinguishes that
red from compiler setup failures and from four test-only capture fixture
adjustments: explicit custody installation and exact sidecar-committed crash
boundaries. Fresh-process recovery still checks Keep and the original capture
reference, then retries without duplicating the attachment.

A second real regression injects an acknowledgement error after the helper
journal transaction commits. The pre-fix run executes once and fails because
the durable note is absent from the live owner although review is Added.
`reconcile-install-before-ack-red-executed-20261029.log` and frozen artifact
`recovery-reconcile-red-artifact-20261029.json` retain that evidence. The actual
completion seam now installs the proven destination before its single helper
acknowledgement. Its regression passes on the final receiving harness.

The first native app pass used three loopback pages and ordinary Save source
and keep controls, with a separate custody broker/custodian and explicitly
isolated roots. The quiet workshop chooser admitted the sealed retained
versions of 91, 99 and 100 UTF-8 bytes. It generated a private proposal and
opened its exact Unicode source citation. That pass exposed two receiving
problems: an asynchronous workshop update did not request its own native
redraw, and accepting refused because reloading the same graph invented a
new revision. No note was accepted in that pass. The native receipt
`native-private-first-receive-red-20261029.json` retains the input identities
and boundaries; scanning all 37 ordinary persisted files and the app log found
no private source canary.

The redraw repair explicitly requests the auxiliary window's redraw. A third
executed regression demonstrates the revision defect through a real save/load:
`canonical-facets-red-executed-20261029.log` runs once and fails on an ordinary
live facet store. The loader now installs an existing modern canonical facet
file exactly, preserving absence, while genuine legacy columns keep their
existing migration and canonical precedence. The added migration control
checks canonical false over a legacy Keep label and stable repeat loading.

The final guarded command builds the normal app, library tests and binary tests
in one lane invocation, using the unchanged 20 GiB reservation:

```sh
PATH=/Users/markik/.cargo/bin:$PATH python3 /Users/markik/Code/tools/build_storage.py \
  --policy /Users/markik/Code/output/alembic-hosts-20261028/admission-wait-900-policy.json \
  build --lane alembic-turnstone-20261028 \
  --cwd /Users/markik/Code/worktrees/turnstone-alembic-hosts-20261028 --reserve-gib 20 -- \
  +1.99.1 build --lib --bin turnstone --tests --locked --offline --message-format=json -j 4
```

`canonical-facets-green-build-20261029.log` / `.exit` succeeds. Cargo identifies
the normal binary with `profile.test=false` and the separate library harness
with `profile.test=true`; both report `fresh=false`. APFS-cloned executables
and their hashes are bound by `private-receive-v2-artifacts-20261029.json`:

- Normal app: `turnstone-private-receive-v2-native-20261029-bd5`, SHA-256
  `b307d72faded793b0971eac3d533bf119f3ceaab2130dee83cda8a1f749da809`.
- Library harness: `turnstone-private-receive-v2-tests-20261029-bd5`, SHA-256
  `7035c0c0f67a24c4549051e4f3a81ce199e096189341bc768b47eb5199197ce3`.

`private-receive-v2-focused-20261029.json` records 103 distinct passing named
tests, with no failures or ignores: session persistence 21, session loading and
migration 16, Surface controls 4, workshop 8, source-capture persistence 26,
saved sources 11, custody binding 4, sealed capture 4, captured Place collection
6 and graph facet editing 3. This includes the abrupt fresh-process Fjall
capture tests. Test roots are explicit, with no scenario plaintext export.

The final normal app cold-opens with `TURNSTONE_OFFLINE=1` after the owned
loopback origin was stopped. Expected typed acquisition refusals appear in its
log. Native inspection then times out for the app through path, bundle ID and
name; a fresh app registration reproduces it while Finder inspection succeeds.
The retained process sample is too sparse to establish a deadlock or root cause.
A preserved-layout/default-map diagnostic also times out; it does not qualify
the original restored Read layout. At that checkpoint, native acceptance, accepted-result cold reading and live
custody retirement remained open for Turnstone. The paused-scene continuation
below subsequently qualifies those bounded workshop conditions; the playing
scene and original restored Read layout remain separate limits. No production,
external-principal or H1 qualification flag is lifted.


## Turnstone paused-scene private native continuation

`native-private-paused-diagnostic-20261029.json` records a controlled fixture
change after the owned app exited: only the saved scene's `physics_paused`
boolean changed from false to true. The original facet bytes are retained and
every other JSON value is verified unchanged. Native inspection immediately
succeeds on the same frozen app, root and offline policy. This narrows the
observed inspection failure to the active scene/presentation condition; it does
not establish a root cause or qualify playing-scene performance. The earlier
layout diagnostic and original layout remain preserved.

The three sources were captured through the ordinary native Save source and
keep action on the earlier frozen normal app (`c2a3f116…`); the capture/storage
path did not change in the subsequent loader/redraw repair. The final app
(`b307d72f…`) cold-receives those sealed versions while the origin is stopped
and `TURNSTONE_OFFLINE=1`. Its quiet chooser again shows the exact 91/99/100-byte
versions. The prior unaccepted proposal is rejected, then a fresh helper run
finishes and displays its proposal without Pause/Resume or another refresh
control. No model provider is involved.

The final native controls qualify:

- Opening the exact Unicode saved citation while offline.
- Accepting one private brief, receiving the graph/receipt acknowledgement and
  reading the retained result Resource `0fd49e75-ce8c-5df6-9ccc-694700343593`.
- Live custody lock at generation 7 closing the private workshop; a fresh
  workshop-open attempt is refused with no source canary exposed in the full
  accessibility tree. Unlock at generation 8 does not revive the old window.
- A fresh binding recovering the same Added result, then helper dissolution
  leaving that result readable.
- Normal owner shutdown and a cold process change from PID 94663 to 95272;
  the dissolved helper's retained review reopens the same Resource and Unicode
  result, with the origin still absent.
- Actual custodian process loss retiring the private window and clearing the
  private canary from the full accessibility tree.

`native-private-v2-headed-20261029.json` retains these observations and the
final store scan. Exactly one accepted note remains: four graph nodes and five
Resources, including the separate operation Resource. All 38 regular files
under the isolated product root and the ordinary app log are free of the private
source canaries. The loopback connection is refused. The app, origin server,
broker and custodian are stopped and their owned process exits verified.

This qualifies the bounded private workshop in both application hosts on one
Mac, with a paused Turnstone scene and synthetic custody. It does not qualify
two physical hosts, an inference provider, authenticated external helper
principals, arbitrary live-scene performance, the original multi-Read restored
layout, private real-store migration or production custody. The complete W3
failure matrix and supplier integration remain open. The current integrated
main candidates introduce a newer Dramatis/Genet cohort; those dependency
identities are awaiting the requested explicit selection before adoption and
combined qualification. Candidate branches are published; main is not merged.


## Approved custody/renderer cohort integration and native requalification

Mark approved Dramatis `6711105e0284adb3e881fc9aebcc56f25000c1ca` and
Genet `565458ea3ab816940484079330eb165a878a22e0` on 2026-10-29. The latter
receives published `genet-taffy` 0.16.0, based on Taffy 0.15.1. Mere's tested
integration commit is `9ce25faf88810802342ad085ba92b607b3222afc`;
Turnstone's tested receiving commit is `78a43ab081b07fe7d8c8b0125cd3e565424d4530`.
Both preserve the existing `branch = "main"` selectors and exact lock revisions.
Servitor remains in Eidetic.

The Mere source audit checks 101 Git packages against actual checkout heads
and preserves all 1,736 external lock records from the selected upstream cut.
Turnstone checks 161 Git packages across 21 source identities. Its 69 selected
source replacements leave unrelated lock records unchanged. Locked offline
metadata succeeds in both hosts, with no local path overlay. Receipts are
`Code/output/alembic-hosts-20261028/cohort-merged-source-identity-audit-20261029.json`
and `Code/output/alembic-turnstone-hosts-20261028/merge-20261029/`.

All compiler commands use the existing stable lanes and guarded launcher,
with the 400 GiB cache cap and 100 GiB free-space floor. The retained
`cohort-*-20261029.command.json`, `.log` and `.exit` files under
`Code/output/alembic-hosts-20261028/` record:

- Custody wire, custodian, Athanor and Pandect: 474 passing tests.
- Graphshell library, native workshop and public process gate: 308 passing,
  six existing ignored tests.
- Turnstone normal app, library and binary test builds; 103 distinct focused
  tests pass with no ignores, including source-capture fresh-process recovery,
  canonical facets, session ownership and acceptance reconciliation.
- Djinn custodian-client and payload IPC: 12 passing, one existing ignored
  helper. Total: 897 passing, zero failing and seven existing ignored tests.
- Normal separate-process custody fixture build, production endpoint/Djinn
  library check without the fixture feature, and portable Athanor workshop
  `wasm32-unknown-unknown` check all succeed.

Frozen normal executables are APFS clones, verified against the Cargo JSON
artifact identity and SHA-256, with `profile.test = false` and `fresh = false`:

- Graphshell: `graphshell-alembic-cohort-20261029`,
  `81c15109c6ebd5111815713d01ced6b17c852aa8d5d15255020965b00c615fe7`.
- Turnstone: `turnstone-cohort-native-20261029`,
  `757f76752d24a10d024d09cc51c5dfe45f743659ae1c1e74a981da5fd5d2f73f`.
- Separate broker/custodian fixture:
  `private-workshop-custody-fixture-cohort-20261029`,
  `5bb878a788ce280d3593af8e1a1131aa1c281b9b5a1e73a02d44668c2704385b`.

Graphshell's integrated private process gate passes seed, generate, acceptance,
exact cold proposal/source/receipt/result recovery, lock refusal, fresh-unlock
recovery, helper dissolution and actual custodian loss. All 33 regular files
are free of the private canary. The independent headed run generates and accepts
through the native controls, opens the Unicode citation, dissolves the helper,
and reopens the same retained storage witness in a different process. Its
Resource is `e53fe0b1-cca2-5328-a06f-b14920c38252`; the native displayed version
is the storage witness, distinct from the clear body digest. The headed root
and ordinary log scan covers 34 regular files.

Turnstone starts with a fresh isolated root and captures three loopback pages
through ordinary Save source and keep. Pause or resume physics and Save session
are ordinary product actions; no fixture metadata rewrite is used. After normal
shutdown the origin stops, `TURNSTONE_OFFLINE=1` is set explicitly, and a new
process restores all three original Read tabs. The quiet workshop admits exact
91/99/100-byte sealed versions, generates without a manual refresh, opens the
Unicode citation, accepts once and reads Resource
`7438c5d4-c94f-593c-8556-9853238b75e8`. That same result survives helper
dissolution and another independent offline process. Exactly one accepted note
remains, with four graph nodes and five Resources. The scan covers 38 regular files, including the ordinary app log, with no
source canary exposed.

On both visible native hosts, custody lock clears private text. Unlock does
not revive the retired window or binding; explicit reopening obtains a fresh
binding and recovers the accepted result. Actual custodian process loss then
clears both hosts again. All owned apps, the loopback origin, broker and
custodian exit; process inventory verifies closure. The two headed receipts
are `cohort-native-private-headed-20261029.json` under each stable lane's
`Code/output` directory; file inventories are retained beside them.

A final upstream fetch also receives the web-only spareval digest-edge repair
and its upstream qualification receipt, merged at
`b67f27481f9c2e5e0b60e91770b4e092abaabcbc`. That merge changes only the
separate web Cargo.lock and ownership-plan/receipt files. Native/workshop
source and the root lock remain byte-identical to the tested cut. The old
spareval full-web failure is therefore a historical result; this run's own
portable gate is the narrower Athanor workshop check.

This closes the selected supplier integration and bounded private native
receiving gates in two application hosts on one Mac, with a paused Turnstone
scene and synthetic custody. It does not close the full per-host W3 matrix,
playing-scene responsiveness, real-store migration, physical two-machine
transfer, inference-provider value, installed-executable authentication,
external helper principals or production custody. No H1 or production flag is
raised. Publication and final source-equivalence proofs are recorded separately
in `Code/output/alembic-hosts-20261028/cohort-publication-20261029.json`.
