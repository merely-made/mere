# macOS H1 synthetic custody fixture

This fixture tests the macOS mechanisms selected in the
[custody hardening plan](../../design_docs/archive_docs/2026-10-28/2026-10-21_custody_isolation_and_security_hardening_plan.md).
It does not open a real vault or migrate a consumer. It uses a CryptoKit
AES-GCM envelope around a newly generated Ed25519 seed, rather than the
production Rust Castellan vault implementation.

## Recorded result

On macOS 15.8.1, native10 passed **34/35** receiving checks. The protected
installation, SMAppService/launchd activation, App Sandbox worker, peer identity,
ordinary-app Keychain denial and sealed process restart controls passed.
The remaining check failed: an unrelated same-user app could invoke codesign
unattended using the development account's existing signing key, mint a fixture
with the custodian identity, read the synthetic wrapping key and recover the
same vault root. The root-owned installed bundle did not prevent this attack.

**Production H1 is not qualified.** Native11 subsequently passed 38/38 checks
in the isolated runtime account, as described below. The fixture does not
change an existing signing key's ACL or export its private material.
The receiving results, exact source cuts and failed iterations were kept
outside git in `support/receipts/custody-h1-macos-20261022/` and retired with
the security-audit worktree on 2026-10-26. The hardening plan's §4.1 and §4.2
record the native10 and native11 results and the exported result's SHA-256.

## Separate-account receiving gate (native11, 38/38 passed)

The user authorized a new local standard account `djinnh1`, with the existing
signing key left in the signing account. The fresh native11 package contains no
Keychain, private signing key or vault seed. The synthetic root is generated
only when its custodian runs in the receiving account. The installer publishes
signed code and a hash-pinned public receiving payload under its root-owned run
directory. It never writes a user's home or executes receiving tools as root.

Preparation now includes a successful signing/identity verification on a
disposable copy of the exact unsigned attacker payload, while preserving the
receiving payload. The receiving runner requires this control, a pinned
manifest, the intended different UID/home, standard non-admin membership and a
graphical login session. It refuses a sudo session. The unrelated app also
attempts to open the signing account's default Keychain file without reading
bytes. ENOENT does not count as an access denial. All previous confinement,
key-access, launch and sealed-reopen gates remain required.

The measured runtime was djinnh1, UID 503, in a non-admin graphical login.
The unrelated app's signing attempt returned exit 1 with a missing-keychain-item
error (no timeout); it was denied a read-open of the signing account's default
Keychain file with EACCES. It retrieved no custody key and recovered no root.
The owner still signed and reopened the sealed root with a fresh instance/PID
and the same public key. Review recomputed all 38 passing checks from the
frozen runner and verified all 45 root-owned payload and eight source hashes.
The measured gate passes for the runtime account; native10's signing-account
escape remains valid. Production packages must preserve signer separation.

The preparation command is:

```sh
python3 -I testing/macos-custody-h1/run.py prepare --output /absolute/new/fixture --profile /absolute/synthetic.provisionprofile --receiver-user djinnh1
```

In the signing account, review and run its generated `SETUP-COMMANDS.txt`:
the first command creates `djinnh1` with sysadminctl and prompts locally for its
new password; the remaining two install the digest-pinned fixture. The account
is not made an administrator. This path neither imports keys nor changes the
existing signing key's ACL. Account creation/GUI login require the user's OS
authentication; preparation does not claim they have occurred.

Switch users and log in normally as `djinnh1`, keeping it a local account
without signing-key import or Apple ID setup. Open Terminal and follow the
installed run directory's `RECEIVER-COMMANDS.txt`. The root installer generates
the same instructions available beside the prepared package, so the new
account can read them without opening the signing account's home. The three
commands bootstrap fresh inputs under the receiver's home, run the protected
experiment, then export only the completed public synthetic result to the
run-specific `/Users/Shared/djinn-h1-<run-id>-results.json`. If background-item
approval is needed, approve the named synthetic fixture and repeat the run
command before exporting. Bootstrap and result export refuse existing outputs.

The receiving run has **38 mandatory checks**. A passing signer boundary needs
the authorized custodian to sign/reopen successfully, the unrelated app to be
unable to use the signing authority, no recovered synthetic key/root, and the
signing-account Keychain file to be inaccessible. A timeout, malformed attack
payload or wrong account cannot qualify. Native11 passed the measured H1-F1
runtime gate. Real consumers remain gated by production Castellan, independent
consent/policy, durable-lock repairs and lifecycle integration. The receiving
process reported successful item deletion and service unregister; both recorded
custodian PIDs are absent. Builder-side cross-account service queries are
denied and are not independent proof of unregistration.

## Roles and boundaries

- The controller is an admitted fixture client. It pins the custodian identity
  before sending the synthetic challenge and rejects adjacent operation scope.
- The custodian runs through the OS service manager, has a launchd parent
  constraint, and uses only its own provisioning-authorized data protection
  Keychain group. Its listener pins the controller's identity. Its signing
  operation has a fixed fixture namespace.
- The worker is a separate App Sandbox XPC service with no vault, blob, network
  or custody-group entitlement. It receives selected input. Direct/symlink
  file, network, inherited-descriptor and live task-port probes have functional
  controls.
- Separately signed same-team/different-ID and ad-hoc apps test ordinary
  same-UID access. A fake server knows the public run ID; an unpinned client is
  its capture control. The unrelated signing-oracle app signs only the prepared
  attacker fixture and reports root recovery by public-key equality.

## Reproduction

Requires macOS 14+, Command Line Tools, an available compatible Developer ID
identity, and a provisioning profile authorizing the dedicated synthetic
custodian App ID/group. The runner defaults identify the signing workstation
used for the recorded experiment; supply the correct `--identity` and `--team`
for a different authorized setup. Never reuse a production access group.

From the repository root:

```sh
python3 -m unittest discover -s testing/macos-custody-h1 -p 'test_*.py' -v
python3 testing/macos-custody-h1/run.py prepare --output /absolute/new/fixture --profile /absolute/synthetic.provisionprofile
```

Preparation compiles and signs the roles, records source/artifact hashes, and
runs the installer's exact signature requirements plus a wrong-identity control.
Review `manifest.json`, `install_fixture.py`, and the generated
`INSTALL-COMMANDS.txt`. The two generated administrator commands copy a narrow
installer to a root-owned path and verify its digest before executing it with
an isolated trusted interpreter. The installer copies only hash-pinned bundle
files, rejects symlink traversal and publishes a verified private staging tree
atomically. No fixture executable runs as root.

After that synthetic-only installation:

```sh
python3 testing/macos-custody-h1/run.py run-protected --fixture /absolute/new/fixture
```

If SMAppService reports approval required, approve the named synthetic fixture
in macOS Login Items and rerun the same command. Status 3 denotes pending
registration, status 2 a completed receiving run with failed security checks,
and status 0 all explicit checks passing. Native10 intentionally returned 2
because signing authority was available to the attacker.

Each receiving experiment needs a fresh prepared directory. The runner refuses
to overwrite a result or accept modified prepared artifacts. The signing attack
changes its prepared attacker app; the receiving run deletes its exact
synthetic Keychain item. Do not run `run-local` on a package intended for a
later protected run. Local launchctl experiments establish launch origin and
individual primitives, while leaving protected installation unqualified.

## Evidence and cleanup

Missing files, unavailable services, timeouts, missing owner entitlements and
arbitrary codesign failures do not count as isolation passes. The 24 evaluator
and installer tests exercise these false-positive cases, symlink/path escape,
and digest verification before installer execution. The separate-account
extension adds 18 context/package/evidence counterexample tests, for 42 current
tests. Native signature checks also run during each preparation.

The receiving runner deletes only its own synthetic Keychain item, removes its
temporary fake/probe jobs and unregisters its SMAppService. Native10's item
deletion and unregister succeeded, and all three job identities were checked
absent afterward. Its root-owned bundle and root-owned installer remain for
review; synthetic state, binaries and profile stay outside versioned source.
Receipts contain hashes, public keys, signatures and bounded observations,
without raw vault seeds or wrapping keys.

The measured task_for_pid denial is one memory primitive. The DYLD test uses a
separate hardened launchd probe with a working unhardened control. A process
restart is not a sleep/logout/reboot, crash-durability, rollback or residue test.
The bounded NSXPC fixture is not a production custody protocol. Native consent,
principal/generation policy, production vault migration, C04/C07 repairs,
resource limits and a scoped continuous consumer still require their own H1
receiving evidence.
