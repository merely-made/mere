# Djinn custodian

Status: H2 implemented. The custodian holds Castellan's resident host, the wallet, the credential catalog and the SSH agent, and a production Djinn reaches all of them through it: Djinn links no castellan and opens no vault. The H1 and H2 tests pass on macOS. Linux and Windows builds type-check; their qualification steps below are still to run on those hosts. Production enablement waits on those runs and on the gaps in the [plan](../../../design_docs/dramatis_docs/implementation_strategy/2026-10-21_custody_isolation_and_security_hardening_plan.md#h2--complete-authority-and-consumer-migration).

`djinn-custodian` is the independently launched process that opens Djinn's Castellan vault (custody isolation plan, §5). Only this process does the following:
- unlocks the vault and locks it;
- signs with its root or a root-derived key;
- derives epoch keys from the wallet and seals retained payloads;
- reviews and imports credentials;
- answers SSH agent requests;
- releases reviewed derived keys, each with the root's attestation.

Djinn and every other client go through [`custody-wire`](wire) over authenticated local IPC and never link castellan.

## Shape

- [`custody-wire`](wire) implements the §5.6 contract once:
  - the peer principal comes from the OS connection: `SO_PEERCRED`, `getpeereid`, the macOS audit-token code signature, or the named-pipe token SID; a hello carries only a version and a nonce;
  - a session opens with the server instance, lock generation, granted scopes and limits;
  - each operation is typed and names its request id and expected generation;
  - consent decisions are accepted only from the trusted channel, for one digest, request and generation;
  - outcomes are typed, and a lost connection is `Unknown`;
  - lock and revocation are monotonic facts.

  Frames keep the Graphshell door's wire format, and their buffers clear on drop. The client verifies the custodian's OS identity before it sends a hello. H2 adds typed operations for the following:
  - derived keys of the persona or wallet root, attestations, derived signatures and public keys;
  - the public read model, profile intents, passphrase change, SSH import and vault commands;
  - wallet epochs, station grants and device revocation;
  - retained payloads;
  - the credential catalog.
- [`policy`](src/policy.rs): installed, deny-by-default, per-principal grants for profile, operation, namespace and consent.
  - A principal matched only by the desktop user's uid or SID cannot sign or release unattended, or act as the consent channel, outside declared development mode.
  - The `djinn-door` rule names exactly the open profile's two door keys, so every other `mere.graphshell` delegation key stays sign-only.
  - Djinn's installed grants are one template, [`packaging/djinn-grants.json.in`](packaging/djinn-grants.json.in), with the consent filled in per OS.
- [`host`](src/host.rs): Djinn's real passphrase vault behind Castellan's `PersonaeHost`.
  - Every start is locked. The first unlock builds the host on the configured or remembered profile and claims the credential catalog; later unlocks reopen it. A changed identity is refused.
  - Nothing mints a profile, and an OS-sealed (DPAPI) vault must be migrated first.
- [`wallet`](src/wallet.rs) and [`credentials`](src/credentials.rs): the wallet's acts and the credential catalog's operations, moved from Djinn unchanged in effect. The epoch derivation is pinned by a test against the epoch read directly.
- [`agent`](src/agent.rs) (Unix): the SSH agent. Its socket admits only the owner's uid, and only once the vault has opened. Per-use approvals become consent petitions.
- [`core`](src/core.rs): the state machine without I/O, driven by [`driver`](src/driver.rs) over any listener. Lock, unlock and revocation each advance one generation, cancel pending petitions and broadcast a fact. A lock reports separately whether memory locked, the drain finished and the barrier persisted.
- [`delegation`](src/delegation.rs): the signing consumer for Graphshell (`mere.graphshell`). It decodes an insigne delegation certificate or revocation, rebuilds it byte-for-byte, then checks the audience, the issuer and the scope key before signing inside custody.
- [`platform`](src/platform): one launcher, peer check and confinement recipe per OS.
- [`fixture`](src/fixture.rs) (feature `fixture`, never installed): a custodian served from a test's thread over a new synthetic vault, for Djinn's test suites and receipt hosts.

The `synthetic` module is the 2026-10-22 core behind the signed macOS receiving fixture's C ABI (`native-host`), kept so that harness still builds.

## Running

```sh
djinn-custodian --config /etc/djinn-custodian/USER.json            # listeners from systemd/launchd
djinn-custodian init  --config PATH                                 # new vault; passphrase twice on stdin
djinn-custodian adopt --config PATH --from DJINN-VAULT-DIR          # copy and reopen to the same identity
```

`--socket PATH` binds the custody socket directly, plus `vault.agent_endpoint` when configured. It is refused outside development mode. Installers: [Linux](install-linux.sh), [macOS](install-macos.sh), [Windows](install-windows.ps1). Djinn finds an installed custodian through a client file the desktop user cannot write (`djinn::custodian::installed_path`); the file also names the agent socket for Djinn's status. A Djinn harness build (`passphrase-fd`) never reads the installed file. It reaches only a custodian named with `--custodian FILE`.

The installers point `vault.data_root` at the custodian's own state directory, so a wallet and catalog created there are the custodian's alone. An existing wallet or catalog in Djinn's data root is not moved (see the plan's wallet gap).

## Platform recipes and qualification

| OS | Launch and identity | Confinement before input | Verified here | Still to run on that host |
| --- | --- | --- | --- | --- |
| Linux | systemd system service `djinn-custodian@USER` under account `djinn-custodian-USER`. It is socket-activated: pid 1 owns the 0600 sockets `custody` and `agent` (`djinn-custodian-agent@USER.socket`), so clients expect uid 0. State lives in `/var/lib/djinn-custodian/USER`. | `no_new_privs`, not dumpable, `RLIMIT_CORE` 0, Landlock and a seccomp allowlist. Landlock ABI 6 is required in protected mode: state read-write, config read-only, no exec, no TCP, abstract sockets and signals scoped. polkit `auth_self` consent runs from a helper forked before confinement. The agent's runtime starts after confinement. | `x86_64-unknown-linux-gnu` type-check of the crate and its tests (`ring`'s C under a stub compiler; nothing linked) | Linux steps 1–8 below |
| macOS | launchd agent adopting its launchd sockets `custody` and `agent`. Both ends of the custody socket identify each other by team and identifier through the socket's audit token. | Deny-attach and no core files. Protected mode refuses to run without a valid hardened-runtime signature lacking `get-task-allow`. | Process, socket and H2 tests; the unsigned negative control for signed peers | Signed install and receiving run (macOS steps) |
| Windows | SCM service under `NT SERVICE\djinn-custodian` on pipe `\\.\pipe\djinn-custodian-<owner SID>`. The pipe's protected DACL admits only the owner, and the client requires the service SID as the pipe server. No agent pipe yet. | Extension points, dynamic code, and remote or low-label image loads disabled | `x86_64-pc-windows-gnu` type-check of the crate and its tests (as for Linux) | Windows steps below (o-pc) |

**Linux** (on the Fedora ThinkPad, kernel with Landlock ABI ≥ 6):
1. Build there: `cargo +1.99.1 test --locked -p djinn-custodian -p custody-wire`, then Djinn's `custodian_client`, `quarantine_review` and `cxf_import` tests. The process tests run unconfined there; confinement runs in development (logging) mode.
2. Install: `sudo ./install-linux.sh markik target/release/djinn-custodian ~/.local/share/djinn/vault`. Check that `systemctl status djinn-custodian@markik.socket djinn-custodian-agent@markik.socket` both listen.
3. Start once with `"development": true` in `/etc/djinn-custodian/markik.json`. Run the production `djinn` (default features) and exercise:
   - start and unlock, door admission and personal sync;
   - a Knot reopen after lock, a payload seal and a quarantine review;
   - `ssh-add -L` with `SSH_AUTH_SOCK=/run/djinn-custodian/markik-agent.sock`.

   Read `journalctl -k | grep audit` for any seccomp `SECCOMP_RET_LOG` hits, and add only the calls that were really needed. Then turn development off and repeat with the filter enforcing.
4. As `markik`, each of these must be denied:
   - reading `/var/lib/djinn-custodian/markik`;
   - `ptrace` on the custodian, and reading its `/proc/PID/mem`;
   - connecting to either socket as another user;
   - squatting a socket path;
   - replacing the units or the config.
5. polkit consent:
   - each grant Djinn uses prompts once per generation, and a guest delegation prompts every time;
   - a per-use SSH key prompts on each signature;
   - cancel must refuse;
   - each prompt must name the requesting process.

   Count the prompts after an unlock and record them; that count is the cost of the same-user posture.
6. Kill the custodian. Djinn must report the identity pending, with its holders locked and its doors refusing new sessions. After a restart the custodian must be locked with a fresh instance.
7. Confirm that `djinn` (default features) does not link castellan: `cargo tree -p djinn -e normal -i castellan` must fail.
8. Leave the wallet at rest as it is. The plan's wallet gap decides when an existing wallet moves.

**macOS** (needs the user's signing step, from an account the receiving account cannot use, H1-F1):
1. Sign `djinn-custodian` with the team identity, the hardened runtime and identifier `com.merely.djinn.custodian`. Sign Djinn as `com.merely.djinn`.
2. Run `sudo ./install-macos.sh USER BINARY TEAM DJINN_TEAM` and log in as USER.
3. Repeat Linux steps 3, 4, 6 and 7 there. An unsigned or differently signed peer must be refused at both ends.
4. Per-use SSH keys are refused: the macOS config has no consent channel (`deny`).

The data-protection Keychain envelope and the native consent dialog remain the [signed fixture's](../../../testing/macos-castellan-custodian-h1/README.md) pieces and are not part of this binary.

**Windows** (o-pc):
1. Run `cargo +1.99.1 test --locked -p djinn-custodian -p custody-wire`, then `.\install-windows.ps1 -User mark_ -Binary target\release\djinn-custodian.exe`.
2. Check that `sc.exe qsidtype djinn-custodian` reports unrestricted, and that `icacls` on the state directory admits only the service SID, SYSTEM and Administrators.
3. As the user, the state files and the service configuration must be unreadable and unwritable. A second local account must not open the pipe, and Djinn must refuse a pipe served by any other process.

Djinn's grants are installed, but every one needs consent. The config is `deny` until a Windows presence channel exists, so each grant is refused there. The DPAPI vault Djinn uses on Windows must first be migrated to the passphrase vault, and no agent pipe is served. A Windows Djinn with this custodian therefore cannot run its consumers yet.

## Negative controls and H2 tests (CI, no GUI)

The controls are in these files:
- [`src/tests.rs`](src/tests.rs), [`src/tests/driver_tests.rs`](src/tests/driver_tests.rs), [`src/tests/h2_tests.rs`](src/tests/h2_tests.rs) and [`tests/process.rs`](tests/process.rs);
- on Djinn's side, [`custodian_client`](../tests/custodian_client.rs), [`quarantine_review`](../tests/quarantine_review.rs) and [`cxf_import`](../tests/cxf_import.rs);
- [`custodian_resident`](../tests/custodian_resident.rs), which runs the real resident and is ignored by default.

They cover:
- a wrong profile, an adjacent namespace, a mislabelled audience and an arbitrary-bytes signing oracle, each refused;
- forged approval fields, self-approval and petitions with the wrong digest, request or generation, each refused;
- a lock during a pending petition invalidates it, and a barrier failure degrades the lock without unlocking;
- a stale instance or generation is refused, and revocation closes sessions and refuses readmission;
- the handshake deadline, connection budgets and consent expiry;
- custodian loss is `Unknown`, and a restart is locked with a fresh instance;
- adoption keeps the identity and leaves the source untouched;
- a protected custodian refuses a self-bound socket;
- Djinn never falls back to a vault of its own when the custodian is locked or gone;
- each H2 operation answers from the vault or wallet opened directly, and is refused without its grant or while locked;
- the epoch derivation is the one Djinn made;
- the door rule releases exactly the open profile's door keys, and Djinn's own door function agrees with it;
- per-use SSH signing is petitioned to the consent channel, and a denial reaches the waiting client at once;
- Djinn's lock holders forget their keys on a custodian lock and re-derive through it on unlock;
- quarantine review and CXF import go through the custodian's catalog behind Djinn's native consent.
