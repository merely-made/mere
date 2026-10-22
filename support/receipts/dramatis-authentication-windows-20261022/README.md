# Dramatis local passphrase change — Windows qualification

Wrapped-root and portable encrypted-profile vaults passed real masked Win32
prompts on Windows 11 Home Insider Preview 10.0.26220, x86_64 (`o-pc`). Human
cancellation, retry, current/new/confirmation and independent cold reopens ran
at Mere `2eb8aa09340f7f64739349c7ee08c05902f10f87`. The public replacement
`New  local fixture * ' café` includes two spaces, a literal glob, straight
apostrophe and Unicode; cold reopen authenticates those exact bytes.

The first attempt exposed a shared-library defect: light-file-dialog 3.21.3
registered its Windows password class for each prompt, treating registration
failure after the first prompt as cancellation. The workspace patch registers
that class once per process. Its regression creates and cancels three hidden
password windows. [UPSTREAM.json](../../patches/light-file-dialog/UPSTREAM.json)
and `vendor-verification.json` retain the published crate checksum and original
file hashes; only Windows GUI source differs. Unix implementation bytes are
unchanged. `initial-failure/` retains the earlier incomplete run, separately
from passing evidence.

Reading, governed Scroll and Knot checkpoint changes landed during qualification. They are
integrated at `12d9efff8fde93de1586434afbc718ce5eee51ad`; `build-inputs.json` records `1117`
repository inputs and the freshly rebuilt app/host. Affected custody/app tests and
scripted headed acceptance passed again. The byte-identical Castellan and
identity-provider gates were reused from their recorded component source cut. Copies of the same native-changed
vaults then passed additional headed cold reopens with the current binaries.
`source-joins.json` verifies 107 byte-identical authentication inputs across
the source cuts, including native UI, custody, storage, endpoints and Dramatis
source. Human prompt receipts remain explicitly scoped to their recorded
native source; current integration does not claim another human input round.

The later publication integration at `96a7d76d7089247b432623f3c7135493e38f9b4e`
adds a disabled optional NomadNet hosting feature. Only two recorded inputs
differ: Cargo.lock and Djinn’s manifest. `source-equivalence.json` verifies
1,115 unchanged inputs, unchanged active dependency trees, and the exact
optional-only metadata differences. Binaries and headed receipts remain bound
to their actual build source; that metadata integration did not rebuild them.

| Gate | Result |
| --- | --- |
| Castellan passphrase tests | 25 passed; unchanged component reuse |
| Djinn provider and custody-wire tests | 4 reused + 2 current passed |
| Current Dramatis state/interaction tests | 15 passed; 3 existing fixtures ignored |
| Current Windows password-window lifecycle regression | 1 passed |
| Artifact-tool regressions and port boundary | 3 Python tests and boundary passed |
| Human native prompts, both formats | 10 reviewed captures; cancellation, retry, acknowledgement and two cold reopens |
| Current scripted headed flow, both formats | 16 reviewed captures; cancellation, wrong current value, mismatch, empty replacement, retry, acknowledgement and two cold reopens |
| Current reopen of native-changed vault copies | 2 reviewed captures; two further cold reopens, no input collected |

All six retained cold reopens rejected the old value, accepted the new one,
and preserved both master identities and private fixture slots. Acknowledgements
survived the three-second refresh, with measured capture times retained.
The 20 retained passing app/host processes exited zero and were absent afterward.
Native prompt pixels were not captured. The native provider reports
`windows_wchar`, `graphic: true`; headed runs used interactive session 1 and the
unlocked `Default` desktop. Rendering used the native host default backend;
no particular GPU/backend is asserted.

`native-qualification/` freezes the human-run cohort. Root software logs,
`scripted/` and `native-cold/` cover current integration. The isolated Windows
checkout avoided concurrent primary changes; primary HEAD/status did change
independently, so the receipt does not claim byte-for-byte primary preservation.
Personal vaults and installed applications were not changed. Stores, keys,
sockets and executables are excluded. Retained public bytes are below the
100 MB lane cap. Post-publication cleanup removes temporary fixture stores, frozen copies, the
scheduled task and Windows worktree; it preserves the shared Cargo cache.

Reproduce with Rust 1.99.1 and `core.autocrlf=false`, using the commands in `software-gates.json`, capturing
compiler artifact JSON during the custody test, then freeze with
`support/dramatis_auth_freeze.py`. Run `support/dramatis_auth_receipts.py` against
fresh owned runtime/output directories; add `--native --native-cancel` for the
human batch. `--native --reopen-only` reopens copies of already changed,
marker-owned fixtures without collecting input. This qualifies a Windows
native development host, not an installer or forgotten-passphrase recovery;
the [macOS receipt](../dramatis-authentication-20261021/README.md) records storage
and recovery boundaries.
