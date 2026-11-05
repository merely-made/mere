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

## 2026-11-05: P1 Linux host and required checks

Published source: `6f542e515aed71785ac3fee8aa9e90afca87e872`.
Line ceiling, Responsa, `cargo clippy --locked --all-targets` and dependency bans
all passed on O-PC. Unrelated shared documentation remained outside the commit.
The explicit vendored path also carries the checked API with Mere when a native
host receives it as a Git dependency; the root patch alone would not do that.

T-PC, maintained `target` and `target/build`, two Cargo jobs:

- Dialog library: **21 passed** on Linux, including the six command/result cases.
- Application consent: **4 passed** over the actual local door and owner policy.
- Native confirmation: **4 passed**, comprising three separate child-process
  scenarios and their helper entry. Actual `SystemNativeIdentityUi` reaches a
  synthetic KDialog executable. Backend failure writes no policy, explicit No
  remains a refusal, and explicit Yes grants only the reviewed declaration.
  The negative menu default and quoted application name survive the process
  boundary. This fixture neither opens a real dialog nor grants actual authority.

Before cleaning the temporary qualification patch, all eleven changed source/doc
blobs matched the published commit exactly. T-PC then fast-forwarded to that
commit with a clean checkout and unchanged lock. Its normal production resident
build is separate from the test feature graph. Production hash/interoperability,
real graphical consent and S-PC receiving remain pending.

## 2026-11-05: replacement production packet and resume preflight

T-PC's clean locked production build at the published source passed with features
`cxf`, `default`, `personal-sync`; its forbidden-feature/artifact list is empty.
The test graph's in-process custody and passphrase-FD features are absent. The
locked graph remained unchanged throughout. Replacement resident: 84862976 bytes,
SHA-256 `c5387a8854edfd3959f57fcb8b2cb9e71c75403fdba0584fb95775827d77174d`.
The qualified CLI remains 7509544 bytes, SHA-256
`23c66b92769a4f32b50e3a9e675afccd86fed9fef988c5bffef026128dd1205c`.

The fresh exact-binary interoperability/no-wallet battery passes **10/10** in
its unprivileged private user/mount namespace, exit zero. It verifies successful
CLI use, lock/re-unlock, custody-loss refusal and wallet/persona absence at each
named stage. Owned fixtures were removed. This is a synthetic runtime result,
separate from personal adoption, confinement and real graphical consent.

Private fleet records under `testing/dramatis/fleet-policy/s-pc-receiving`:

- `resident-release-build-02.json`: SHA-256
  `ecaf394f1f1cb6855422cdd8f41696cab918c437490c5290919309bd94405c05`.
- `production-no-wallet-04.json`: SHA-256
  `420dacec3c78b13eca2225f4ee31b0ff77dad12126cd62111cba5b59662b123d`.
- `replacement-qualification-01.json`: SHA-256
  `19c7264a6ace819864e4d78a013995930af40159a0dde5f9dde72d2b59e65b5b`.
- `resume/candidate.json`: SHA-256
  `3c494426892e98b4d1be3590443be8a084292242a4fab99aeb64b820718218e6`.

Compressed delivery was streamed T-PC to O-PC to S-PC in at most 1 MiB controller
buffers, with no persistent T/controller gzip copy. Both S inputs match exact
length/hash. Reallocating the record reserve to 1500000 bytes includes private
resume code/journals while preserving the total 160000000-byte peak cap; the
controlling raw-pair window is 155472520 bytes. The Dramatis lane owns delivery,
recovery and complete cross-host accounting.

The reviewed existing-state resume driver has twelve passing synthetic safety
checks. Its final source is
`05e30295351bc540d63e9b27afeb088711734a0f48195c0173616a3ff9a4a8ab`;
installed hash checks precede both native `--help` loaders and resident start.
Actual noninteractive Windows PowerShell 5 `-VerifyOnly` passes against the
staged S-PC packet without opening a window. Real target loaders, protected
recovery inspection, native owner answer and personal receiving remain open.

## 2026-11-05: native receiving stopped before corrected-resident start

S-PC's resume reached existing protected-state validation and installed binary
hash checks, then rolled back at an incorrect receiving-loader expectation.
The added driver required exit0 from `djinn --help`; pinned source and the exact
replacement independently show that this command prints static usage on stderr
and exits2. The unchanged vault CLI help exits0. The human traceback identifies
this exact pre-start command. All seven originals were restored and verified;
no corrected-resident startup or real application decision occurred. Dramatis
owns repairing that check against measured exit/length/output hashes and retaining
the failed run. The qualified replacement and locked graph remain unchanged.

## 2026-11-05: receiving help-contract correction qualified

The separate Dramatis continuation checks the measured exit and both output
lengths/digests. Ten actual T-PC binary/invalid-result cases and fourteen
synthetic recovery/input cases pass against driver
`d89ccb3d528750dfb8d72dca960086f67c6827bd259da5d8968c054f59a94c60`.
No production rebuild occurred. The unchanged pair is restaged under a distinct
resume-loader attempt, preserving both failed journals and original recovery.
Actual PS5 read-only preflight passes. The all-host record reserve is2000000
bytes for retained history and private code/journals, within the same160000000
peak cap; the controlling raw-pair window is155972520 bytes. Native vault and
real application admission remain pending.
