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
by this Mere cut. A coherent local supplier overlay is temporary and must be
removed before canonical Git-pin qualification. The scoped overlay resolution
now passes locked metadata: its AccessKit crates.io override selects the local
fork directly instead of leaving a Git-source patch unused. Retained attempts distinguish parse/setup failures, the
actual supplier mismatch, and guard refusals from executed product tests.

The current compiler-cache cap is 300 GiB and the free-space floor is 100 GiB.
Repeated active-lane/admission refusals delayed the final preview gate and
prevented receiving attempts from reaching Cargo. The final Mere gate above
subsequently ran successfully; Turnstone remains separate. No cap increase has
been applied. Source, native
window, OS containment, private custody, real Persona authority, provider/model,
remote fleet and full two-host W3 qualification remain separate claims.

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
