# Dramatis local passphrase change — Apple M4 qualification

Native passphrase changes and independent cold reopens are qualified for both
wrapped-root and portable encrypted-profile vaults on Apple M4 (`Mac16,3`,
arm64), macOS 26.5.1 build 25F80. The fixtures contain two synthetic personae and
private protocol slots. Personal vaults and installed applications were not
changed.

The human-input cohort uses source `cfbdaa88a464613c929b2b172c1c2c22866b5a27`.
Both formats exercised the ordinary app button, native cancellation, retry,
current/new/confirmation and successful acknowledgement. The graphical
`applescript` provider collected the exact public replacement
`New  local fixture * ' café`, including two spaces, punctuation and Unicode.
Native prompt pixels were not captured; app captures record feedback and controls.

The original portable scenario stopped presenting frames after its success
capture, before the later refresh capture. Its receipt reports failure and is
retained under `native-input/`. The cause is unconfirmed. Independent recovery
cold reopens then accepted the exact new bytes, rejected the old value and
preserved identities/private slots for both native-changed vaults.

Concurrent stack updates are integrated at `8d8f7baff76e9d484106134c213fd9c89316f6e1`. The app and host
were rebuilt on the M4, and the full scripted headed flow plus cold reopens of
byte-identical native-changed vault copies passed again. These current runs
collect no additional native input. `source-joins.json` binds the unchanged
authentication implementations to the earlier native evidence. Dramatis's public
contract remains pinned to current main `7d498c27df8076dacccb74ba4a05f863f9f48c76`.

| Gate | Result |
| --- | --- |
| Castellan passphrase implementation | 26 passing M4 tests reused from the unchanged component source |
| Djinn identity-provider implementation | 4 passing M4 tests reused from the unchanged component source |
| Current Djinn authentication wire | 2 passed; headed host separately invoked |
| Current Dramatis state/interaction | 15 passed; 3 existing fixtures ignored |
| Current artifact-tool regressions and port boundary | 5 Python tests and boundary passed |
| Current scripted headed flow, both formats | 16 reviewed app captures; two cold reopens |
| Current native-changed vault reopens | 2 reviewed app captures; two cold reopens, no input collected |

The scoped software total is 47 Rust tests (30 component tests reused, 17
current) plus five current Python regressions. Scripted headed checks cover
cancellation, rejected current value, mismatched confirmation, empty replacement,
successful retry and acknowledgement through background refresh. Each cold
reopen rejects the old value, accepts the replacement and preserves identities
and private slots. Current app/host processes and GUI jobs exited zero and were
absent afterward. The interrupted earlier portable presentation remains a
separate failed scenario; its partial checkpoints are labelled in the review.

`build-inputs.json` records 1111 tracked repository inputs,
15 generated build inputs and 19 resolved Git suppliers, with
arm64 Mach-O binary hashes and verified ad-hoc signatures. Tracked inputs match
the Git archive. Generated hashes were separately checked on the M4. The shared
freezer repair records Cargo-generated files separately from repository source,
including caches inside the checkout, while still refusing unexpected untracked
source. Two new regressions cover that distinction; three existing regressions cover
artifact parsing.
The first rejected freeze is retained in `native-input/freeze-initial.log.gz`.

The initial clean checkout was fast-forwarded for preparation. A later
concurrent checkout update was refused by headed preflight before prompting;
qualification moved to an isolated, exact-source worktree. `native-isolation.json`
records that boundary. Owned GUI jobs run in the unlocked console's `Aqua` domain
with SSH variables absent. Current test jobs use a process-scoped `caffeinate`
inhibitor. Rendering uses the native host default; no particular GPU backend is
asserted. Stores, keys, sockets and executables are excluded from this packet,
which is below the 100 MB receipt cap.

Post-publication cleanup removes owned GUI jobs, fixture/frozen copies and the
isolated M4 worktree while preserving concurrent primary work and its shared
Cargo cache. This qualifies the native development host. The
[Intel macOS receipt](../dramatis-authentication-20261021/README.md) records the
original storage and recovery boundaries. Reproduce using `software-gates.json`,
the archived GUI runner and shared `support/dramatis_auth_receipts.py`; use
`--native --native-cancel` with fresh fixtures for human input, and
`--native --reopen-only` for already changed, marker-owned synthetic vaults.
