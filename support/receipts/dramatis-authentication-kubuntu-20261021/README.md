# Dramatis local passphrase change — Kubuntu runtime qualification

Both wrapped-root and portable encrypted-profile vaults passed through the
ordinary Dramatis → Djinn → Castellan route on Ubuntu 26.04.1 LTS, x86_64,
using the existing KDE Wayland desktop and KDialog 25.12.3 on `s-pc`.
The source is Mere `a58b8014e14829356d1d013f4abb7051e1717111`; publication adds
only documentation and evidence. Personal vaults, installed apps and the older
primary checkout were preserved. Its sparse-checkout settings were unchanged.

This is an Ubuntu runtime qualification using the exact Fedora-built Linux
executables, not an Ubuntu compilation. The producer used Rust 1.99.1. Reusing
those binaries avoided a full build with limited available RAM and nearly full
swap; no user applications needed to be closed. Executable hashes, Ubuntu loader
resolution and repository source inputs were checked before use; executable and
source hashes were rechecked after acceptance.

The inherited [Fedora build manifest](../dramatis-authentication-fedora-20261021/build-inputs.json)
records the app, fixture host, 1,089 source inputs and 19 resolved Git suppliers.
`transfer.json` adds nine executable hashes and 38 extra test-source inputs:
1,127 distinct verified repository inputs, without duplicating the inherited
manifest. The app's reported build identity matches it. `preflight.json` records
the desktop, providers and Ubuntu dynamic dependencies.

| Gate | Result |
| --- | --- |
| Castellan passphrase tests | 26 passed |
| Djinn password-provider tests | 5 passed |
| Authentication wire tests | 2 passed; ignored hosts invoked separately |
| Fresh process without a graphical provider | 1 passed; actual custody refusal and preserved old credential/slots |
| Retained interaction and application state | 10 + 5 passed; 3 existing app fixtures ignored |
| Scripted headed flow, both formats | 16 captures; cancellation, wrong current value, mismatch, empty replacement, retry, acknowledgement and cold reopen |
| Real KDialog prompts, both formats | Human cancellation and successful retry; 10 captures |
| Four independent cold reopens | New accepted, old rejected, identities and private fixture slots preserved |

The replacement `New  local fixture * ' café` includes two spaces after `New`,
a literal glob, straight apostrophe and Unicode. Reopen authenticates those
exact bytes. Both native fixtures record `kdialog` with `graphic: true`.
Acknowledgements remained visible more than 14 seconds after success, exceeding the
three-second background refresh. All 16 final app/host processes exited zero
and were absent afterward. `post-run.json` retains original capture times,
hashes and primary-checkout preservation; `capture-review.json` records all
26 reviewed app captures. Native prompt pixels were not captured. An earlier
unattended native attempt was stopped at its first prompt and is excluded.

The tests ran again on Ubuntu with one test thread. The port dependency boundary
and normal app dependency tree are inherited unchanged from the verified Fedora
cohort; the boundary command was not rerun with Ubuntu's older installed Rust.
The [Fedora receipt](../dramatis-authentication-fedora-20261021/README.md) describes
the shared Linux input repair and retained feedback fix. The
[macOS receipt](../dramatis-authentication-20261021/README.md) describes the storage
contract and recovery limits. This does not qualify an Ubuntu build, installer,
power-loss recovery or another operating system.

To reproduce, build the producer cohort with Rust 1.99.1 and freeze app/host
with `support/dramatis_auth_freeze.py`. Transfer the exact executables listed
in `transfer.json`, verify their hashes and source inputs, then rerun the commands
in `software-gates.json` on Ubuntu. Use the shared
`support/dramatis_auth_receipts.py --desktop` controller with those frozen
`--binary` and `--host`, a fresh `--runtime` and `--output`, and the recorded
`VK_DRIVER_FILES`. Repeat with `--native --native-cancel` and new directories.
Each format asks for cancellation first, then the public current value
`Old local fixture`, the replacement above and its confirmation. Stores, sockets,
executables and SSH state are excluded from public artifacts.
