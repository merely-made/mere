# Native confirmation repair

**Status, 2026-11-05: in progress.** S-PC's isolated custody receiving reached
native unlock and the selected weekly lock settings, then rolled back after an
application-confirmation backend failure was persisted as an owner refusal.
Mere owns this platform-dialog repair. Dramatis's fleet rollout owns recovery,
installed-binary acceptance and the later root-trust steps.

This implements the native answer boundary required by D25 of the
[capability door plan](2026-10-30_capability_door_plan.md). That plan owns
application admission, declarations and remembered owner decisions. This plan
owns how a native confirmation produces an answer or a failure. Its
[receipts](../../archive_docs/receipts/2026-11-05_native_confirmation_plan_receipts.md)
separate source tests, installed backend facts, candidate builds and real desktop
acceptance. Fleet continuation: `repos/dramatis/design_docs/2026-11-05_fleet_custody_rollout_plan.md`.

## Required behavior and ownership

A successful explicit affirmative choice may grant the reviewed declaration.
A successful explicit negative choice may persist the owner's refusal. Closing,
canceling or failing to create a dialog supplies neither decision. Those paths
refuse the current request and leave remembered application policy unchanged.
The existing owner settings and custody authority remain the decision owners.

Preserve the existing negative Linux default. Display the actual authority-owned
prompt, including quotes, Unicode and line breaks. Prompt text is data passed as
native process arguments. It must not become a shell program or be replaced by
an invalid-text placeholder. Unsupported graphical backends return an explicit
failure; a console fallback cannot answer an application-admission question.

The shared `light-file-dialog` patch owns checked Linux confirmation results.
Mere's `SystemNativeIdentityUi` maps failures into its existing native-UI failure
seam for application, signing and quarantine confirmations. Existing callers of
the infallible message-box API may keep its conservative refusal fallback.
Windows and macOS dialog implementations retain their separate qualification.
This work adds no admission grant, executable pin, identity or key machinery.

## P0. Reproduce and preserve

Read the exact candidate source, installed backend help and public receiving
diagnostics. Keep the failed native run and remembered refusal intact. Do not
repeat unlock or install actions merely to rediscover a backend parser error.

**Done when:** the unsupported option and failure-to-refusal conversion are
reproduced without opening a permission dialog; the successful vault/trigger
steps and subsequent rollback are recorded with their actual evidence boundary.
Do not infer that a human chose No from the old boolean return alone.

## P1. Checked native choice

Add a checked Linux message-box path using supported graphical choice controls.
For KDialog, a native choice list with a selected negative entry returns an
explicit fixed response token only on successful acceptance. This avoids the
unsupported `--defaultno` option and separates an explicit No from parser,
display, spawn or cancellation failures. Compatible Zenity-family choice lists
follow the same response contract. Single-button notices remain single-button
acknowledgements. Use direct argument vectors and validate exit status plus the
bounded set of expected answer tokens.

Expose the checked result to native identity confirmations. Backend failures
must propagate before `OwnerConsent` writes either `applications` or
`refused_applications`. Preserve the real Yes and No paths and their exact
declaration scope. Never repair receiving by installing a synthetic fixture grant
or deleting a remembered denial without an owner-authorized recovery decision.

**Done when:** focused tests cover affirmative/negative answers, close/cancel,
backend failure, invalid response, negative defaults, literal prompt transport
and unchanged owner policy on native-UI error. The vendored patch's provenance
describes the added behavior. Native headed acceptance remains separate.

## P2. Candidate qualification

Use the maintained Mere targets and unchanged locked dependency graph. Qualify
the shared primitive and host admission paths on Linux, and compile the host
change on Windows where practical. Build a new production resident with exact
source/features/hash and retain the prior candidate source/hash and failed-run receipts as evidence.
Repeat only the binary interoperability and no-wallet gates affected by the new
resident; custody confinement and the unchanged CLI retain their own receipts.

**Done when:** focused tests and required checks pass, actual installed backend
arguments are supported, and the replacement resident has an explicit receiving
packet. A successful CLI help probe is not a native owner-consent result.

## P3. Resume the existing S-PC recovery

The failed first installation left protected adopted state and recovery while
restoring the original resident. The receiving lane must resume that reviewed
state, rather than invoke first-install/adoption again. Keep the preserved
refusal and failed-run receipts as history. The new native answer must come from
the corrected owner surface. Apply the existing weekly policy and retain the
explicit I2P, remote-consent and approval-only-opening boundaries.

**Done when:** identity, catalog/agent continuity, real owner admission, weekly
settings and the named native receiving checks pass. Cold reopen, fresh outbound
SSH and root-trust setup are reported independently. All human inputs stay in
the one target-local handoff, with an Enter gate before mutations and no short
human-response deadline.

## Retention

UI receipts have a 200000-byte subcap within the fleet rollout's 2000000-byte
record reserve; the shared receiving peak remains 160000000 bytes. Candidate,
recovery and compressed delivery copies stay in that shared count. Reuse stable
Cargo targets/build directories, preserve unrelated WIP, and remove only verified
generated delivery/cache files after their owners have exited.

## Findings

- 2026-11-05: the candidate's KDialog adapter appends unsupported `--defaultno`
  for a negative default and converts every failed command into false. The host
  then persists false as an owner refusal. An installed offscreen help probe
  reproduces the parser error without any consent interaction.
- 2026-11-05: the legacy Unix message-box path also replaces quoted text with an
  invalid-message placeholder; application confirmation always quotes its name.
  Literal argument transport is required to display the reviewed grant.

## Progress

- 2026-11-05: reproduced the native backend defect and reviewed the rollback
  evidence. Shared dialog/host repair belongs to this lane; the existing
  auth-controls chat retains S-PC's single native recovery handoff.
- 2026-11-05: P1 source qualification passed: Windows library/host tests,
  Linux library/door/native-process integration and all required local hooks.
  The published fix is `6f542e515`; T-PC's clean production build is in progress.
  P2 installed-binary gates and P3 real owner admission remain open.
- 2026-11-05: the clean production resident and unchanged CLI passed the exact
  ten-check interoperability/no-wallet battery. Verified compressed inputs are
  staged on S-PC by bounded streaming, with no T/controller archive copies.
  The final resume driver and read-only PS5 preflight pass; native receiving,
  target loader acceptance and real owner choice remain open in P3.
