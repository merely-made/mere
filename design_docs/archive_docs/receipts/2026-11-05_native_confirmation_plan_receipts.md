# Native confirmation repair receipts

Owner: [native confirmation plan](../../mere_docs/implementation_strategy/2026-11-05_native_confirmation_plan.md).
UI evidence subcap: 200000 bytes inside the fleet rollout's record reserve.

## 2026-11-05: P0 and first source gates

The actual S-PC receiving attempt reached isolated custody, native vault unlock
and the selected weekly triggers. Application admission then logged
`granted=false` about 0.38 seconds after the unknown application connected. The
original resident was restored, original files and identity were verified, and
new custody remained installed but inactive. This is a failed receiving run,
not a successful rollout. The root-private event log was not inspected.

Private controller diagnostic: `testing/dramatis/fleet-policy/s-pc-receiving/receiving-diagnostic-01.json`,
2001 bytes, SHA-256 `2c7ebff7e81284dc95889b3d18909dce2db02cecb9321df8de5a19c49d774ee9`.
The Dramatis continuation receipts own the complete receiving/rollback account.

Source inspection found unsupported KDialog `--defaultno` and the legacy shell
wrapper's conversion of every failure into false. An installed S-PC offscreen
help probe exited 1 with `Unknown option 'defaultno'` without opening a dialog.
The same legacy path replaces quoted prompts with an invalid-message
placeholder; application names are always quoted by the host. Installed
KDialog help supports `--menu` and `--default`; installed Zenity help supports
list/radiolist, hidden/printed columns and `--no-markup`.

Primary references: [KDialog documentation](https://develop.kde.org/docs/administration/kdialog/)
and [KDialog source](https://github.com/KDE/kdialog/blob/master/src/kdialog.cpp).
The checked adapter uses fixed menu/radio response tags and direct process
arguments. A successful explicit No remains a real negative answer. Nonzero
exit, malformed output and missing graphical support return an error before
the host can write remembered owner policy. This does not infer the human's
answer from the old failed invocation or clear the preserved refusal.

O-PC, maintained target `C:\t\cargo-targets\mere`:

- `cargo test --locked -p light-file-dialog --lib`: **25 passed, 2 ignored**.
  Includes six checked command/result tests compiled portably on Windows.
  The two ignored Windows headed fixtures are not qualified by this run.
- `cargo test --locked -p mere-resident --test application_consent`:
  **4 passed**. Real Yes/No policy and route admission still pass; a native-UI
  error does not create a settings file, grant or remembered denial.

These are source/host tests on Windows. Linux process integration, the new
production resident, required local checks and a real desktop owner answer
remain pending. No S-PC recovery prompt was reopened by these tests.
