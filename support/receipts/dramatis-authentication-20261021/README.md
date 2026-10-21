# Dramatis local passphrase change — macOS qualification

Local authentication change through Dramatis, Djinn and Castellan, using two
public synthetic profiles in each of the wrapped-root and portable encrypted
profile formats. No personal vault, installed resident, or installed app was
changed. This qualifies macOS x86_64 on the native Cambium host, not other
platforms or a distribution release.

The tested implementation is `576061da39c91f0f210a2f473c5c04c85630d982`, including
current Mere main through `3bf2571918a9da6375e2cfeb29ff78c03c29b3d6`. Dramatis's
public, credential-free contract is pinned coherently at
`7d498c27df8076dacccb74ba4a05f863f9f48c76`. An earlier green native run was repeated
after concurrent stack changes were merged; these are the final rebuilt runs.
`build-inputs.json` records 1,027 repository inputs, 19 resolved Git suppliers,
the source digest reported by the app, toolchain and frozen executable hashes.
The later merge of `4232d56229428b7d3f6b1f7260b45d427618ddc4` adds only license headers in the uncompiled optional site-history module and its tests; all 1,027 frozen input hashes remain identical. The publication commit adds documentation and evidence only.

| Gate | Result |
| --- | --- |
| Dramatis facade library | 25 passed |
| Focused Castellan passphrase library tests | 26 passed; 259 outside the filter |
| Djinn authentication wire tests | 2 passed; headed host separately invoked |
| Dramatis retained interaction and state | 8 + 5 passed; 3 existing fixtures ignored |
| Mere port / Graphshell web dependency boundaries | Passed |
| Scripted headed flow, both formats | 16 captures; all four app/host pairs exited 0 |
| System-native masked prompts, both formats | User entered public values; 8 captures; all four app/host pairs exited 0 |
| Cold reopen, both providers and formats | Old rejected, new accepted, public identities equal; exact private seed/slot assertions passed |

The scripted provider exercises cancellation, wrong current passphrase,
mismatched confirmation, empty replacement, successful retry, acknowledgement
through background refresh and cold reopen through the ordinary retained
button, worker and custody route. The native provider substitutes
`SystemNativeIdentityUi` and the user's input in the three real masked dialogs.
Native prompt pixels were not captured; PNGs show the ordinary app before and
after that owner action. The fixture is an explicitly invoked ignored integration
test, not a production consent or credential bypass.

Storage tests additionally prove replacement failure leaves the old credential
and live key usable, later ordinary writes use the replacement key, legacy
portable files authenticate and migrate every profile, and wrapped-root change
leaves sealed profile/other record bytes unchanged. Wire tests refuse another
admitted app before prompting and refuse a vault locked during confirmation.
Only a completed replacement produces the success receipt. Connection failure
is unconfirmed, and refresh does not invent an acknowledgement.

Both storage formats share one encrypted-file replacement helper. It writes a
private same-directory temporary file, syncs it and renames last. This is a
cold-reopen and pre-rename failure qualification, not a power-loss or directory
metadata durability test. Forgotten-passphrase recovery, additional authentication
methods and changing other computers remain outside this local slice.

Reproduce from Mere with Rust 1.99.1 using the commands in `software-gates.json`.
Build `dramatis-ui` and the `djinn` integration test `passphrase_change`; the
compiler-artifact JSON supplies the test executable path. Then run
`python3 support/dramatis_auth_receipts.py --binary APP --host TEST --runtime NEW_PRIVATE_ROOT --output NEW_RECEIPT_ROOT`,
and repeat with `--native` and fresh paths. Each native format asks for the public
values `Old local fixture`, `New local fixture`, `New local fixture`. The controller
refuses existing output/runtime roots; the fixture refuses an existing seed
vault and requires its marker on reopen. Stores, sockets and executables are
excluded from this public receipt.
