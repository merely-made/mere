# Dramatis local passphrase change — Fedora qualification

Fedora Linux 44 Workstation, x86_64, GNOME Wayland on the ThinkPad, using Zenity
4.2.2 and the native Cambium window. Both wrapped-root and portable encrypted
profile formats passed through Dramatis → Djinn → Castellan. This continues the
[macOS qualification](../dramatis-authentication-20261021/README.md); the storage
contract and remaining recovery limitations described there still apply.
Only public synthetic vaults were opened. Personal vaults, installed apps and
the installed resident were not changed.

The frozen implementation is `73b25d9ee71184a1a8e7bf53b774255815810727`, including
current main through `1dcb79cce851739a521c7e1bccd1f42a9cd9be2d`.
`build-inputs.json` records 1,089 repository inputs, 19 resolved Git suppliers,
Rust 1.99.1, app build identity and both executable hashes. The later
`9069774d7076283225db00b154bd36baf30bf501` adds a retained retry test and Linux
provider documentation; all 1,089 frozen inputs remain identical. Evidence and
publication changes do not alter those inputs.

| Gate | Result |
| --- | --- |
| Castellan passphrase tests | 26 passed; 259 outside the filter |
| Djinn password-provider tests | 5 passed; 230 outside the filter |
| Djinn authentication wire tests | 2 passed; 2 explicitly ignored hosts run separately |
| Fresh process without a graphical provider | 1 passed; actual custody route refused, old credential and slots preserved |
| Dramatis interaction and state, final retry regression included | 10 + 5 passed; 3 existing fixtures ignored |
| Port / Graphshell web dependency boundaries | Passed; normal app dependency tree excludes Castellan |
| Scripted headed, both formats | 16 captures; cancellation, wrong current value, mismatch, empty replacement, successful retry, acknowledgement and cold reopen |
| System-native masked dialogs, both formats | Human input, real cancellation and retry; 10 captures |
| Four independent cold reopens | New accepted, old rejected, public identities equal; private seed and credential-slot assertions passed |

The replacement fixture is `New  local fixture * ' café`, with two spaces after
`New`. Cold reopen authenticates with those exact bytes. Linux now invokes the
selected masked helper directly, avoiding the dependency's unquoted shell
`echo` pattern that collapses spaces and expands globs. The public negative
control is in `upstream-input-fidelity-control.json`. Console, unmasked and
unknown password providers are refused. This also protects the shared SSH
private-key passphrase prompt, whose code uses the same input helper.

Capture review found conditional feedback insertion disturbing retained heading
styling. Keeping the status live region present fixes that layout; interaction
regressions cover stable heading identity/geometry and retry after a presented
busy state. `capture-review.json` records every final PNG and its public state.
`post-run.json` records original capture times: acknowledgements survive over
4.7 seconds, exceeding the three-second refresh. All 16 final app/host processes
exited zero and were absent afterward. The remote primary checkout's HEAD and
tracked state were unchanged.

Initial preparation caught a lockfile normalization/source-digest mismatch and
an unsuitable heading selector in a regression test. The final freeze and
corrected tests passed. Earlier headed captures, an interrupted native run and
a diagnostic native change are excluded from the final counts. During the final
sealed native run, the operator reported a rejected current value, then retried
through the ordinary button successfully; that intermediate rejection has no
archived PNG and is not counted as an additional captured control. Native prompt
pixels were not captured; the archived PNGs show the ordinary app around the
human-operated dialogs. This is native development-host qualification, without
an installer, power-loss, Kubuntu, Windows or M4 qualification claim.

Reproduce the software commands in `software-gates.json`, then freeze the app
and ignored `passphrase_change` host with `support/dramatis_auth_freeze.py`.
Run the shared `support/dramatis_auth_receipts.py` with `--desktop`, frozen
`--binary` and `--host`, and fresh `--runtime` and `--output` directories. Repeat
with `--native --native-cancel`. In each native format cancel the first prompt,
then supply `Old local fixture`, the exact replacement above, and its
confirmation. The existing logged-in desktop must be unlocked. Linux requires
Zenity, Yad or KDialog; this receipt qualifies Zenity only. Vault stores, sockets,
executables and SSH state are excluded from the public artifacts.
