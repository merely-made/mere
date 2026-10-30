# macOS Castellan custody integration fixture

Status: the frozen native01 fixture passed 49/49; this source cut adds the residue repair and has not been requalified as a new native binary; real consumers remain gated.

This harness links `djinn-custodian`'s synthetic core (`djinn_custodian::synthetic`, feature `native-host`) into the signed native custodian. The portable custodian process that Djinn now uses on every OS is described in the [crate README](../../ports/mere-resident/custodian/README.md); this fixture remains the signed launch, Keychain and native-dialog evidence for macOS. The harness replaces the earlier fixture's standalone CryptoKit root envelope with Castellan's real `SealedProfileStorage` and `IdentityVault`. The native11 fixture's 38/38 receiving result is recorded in the hardening plan; its receipts were retired with the security-audit worktree on 2026-10-26. Those historical results do not qualify this binary.

The custodian alone links Rust custody. It retains root-owned installation, a compiled installation path, SMAppService/launchd launch, the launch-parent constraint, Hardened Runtime, library validation, signed peers and its exact data-protection Keychain group. Workers remain App Sandbox XPC services without custody, blob or network access. Runtime remains the existing standard graphical `djinnh1` account without the signing key. Do not import that key, use `sudo -u`, recreate the account or migrate personal vaults.

## Receiving evidence

The 49 mandatory checks include all 38 earlier checks plus real Castellan root/derived signatures, forged client approval refusal, bounded scoped grants, adjacent scope refusal, lock-persistence errors and retries, locked cold restart after attacker removal of the marker, OS-confirmed unlock, revocation, exhaustion and a valid signed bundle copy rejected before key access. A missing binary, timeout, unavailable UI or arbitrary signing failure is not a pass.

The successful run needs five synthetic signing/grant approvals and two OS unlock authentications, including the final cold reopen. The custodian owns these dialogs and Deny is the default. Only approve dialogs for this synthetic run. No script auto-approves them. If Login Items approval is required, enable only **Djinn H1 Castellan Controller** and rerun. Denied or unavailable authentication leaves the gate closed; retain that result before preparing another fresh run.

The lock-error probe creates an empty directory at this run's `locked` path, checks two `vault-storage` refusals while memory remains locked, removes only its empty injected directory, and retries successfully. It then deletes the advisory marker as a same-user tampering probe. Existing vaults must still start locked and require native authentication. It touches no other directory.

## Build and prepare

From the repository root, using fresh output directories:

```sh
python3 testing/macos-castellan-custodian-h1/build_rust.py --target-dir /absolute/build-target --output /absolute/frozen-rust-build
python3 -I testing/macos-castellan-custodian-h1/run.py prepare --output /absolute/new-fixture --custodian-build /absolute/frozen-rust-build/build.json --profile /absolute/synthetic.provisionprofile --receiver-user djinnh1
python3 -I -m unittest discover -s testing/macos-castellan-custodian-h1 -p 'test_*.py'
```

The build freezes the actual library and Rust source hashes. Preparation refuses source drift, verifies exact signed identities and rejects a wrong identity using the installer's own command. It proves the attacker payload can be signed in the builder account without transferring signing material. Only the custodian links the Rust library; no fixture code runs as root.

Review and run the two generated administrator commands. They copy a narrow installer to a root-owned location, verify its digest before execution with `/usr/bin/python3 -I`, and atomically install only pinned synthetic code and public inputs. The existing account is reused. Root never executes receiving tools or writes that account's home.

Switch into `djinnh1` graphically. A run-specific launcher in `/Users/Shared` can bootstrap pinned inputs, run the experiment, pause for Login Items approval and export the public synthetic result. It is accessible after switching accounts. The runner refuses the signing UID, admin membership, a wrong home, sudo sessions and an absent graphical login. Export: `/Users/Shared/djinn-h1-<run-id>-results.json`.

## Qualification limits

Recompute the result with its frozen evaluator and check installed/payload hashes. A 49/49 receipt would qualify this synthetic macOS integration cut only. It does not migrate existing consumers, implement transactional migration, inventory every key copy, qualify product policy or prove sleep/logout/reboot/power-loss, dump/swap residue, signed updates or other platforms. A normal restart is not a power-loss test. The signing-account native10 escape remains valid: runtime signer separation is a deployment rule.

The original attacker root-decrypt probe targets the historical CryptoKit envelope. This integration still requires that the unrelated app obtain no wrapping key and have no signing authority; any obtained key fails the gate. The old decoder is not a positive control for Castellan's record codec. Real root/derived controls use Castellan and independent CryptoKit public-key verification.
