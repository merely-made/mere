# Djinn custodian

Status: H2 implemented. The custodian holds Castellan's resident host, the wallet, the credential catalog and the SSH agent, and a production Djinn reaches all of them through it: Djinn links no castellan, opens no vault and opens no wallet file. An existing wallet and catalog move into the custodian's state directory with `adopt --data-from`. The H1 and H2 tests pass on macOS. Linux and Windows builds type-check; their qualification steps below are still to run on those hosts. Everything Windows-specific (the consent helper, the OS-vault adoption and the agent pipe) is type-checked only: the Windows steps below, on o-pc, are its qualification. Production enablement waits on those runs and on the gaps in the [plan](../../../design_docs/archive_docs/2026-10-28/2026-10-21_custody_isolation_and_security_hardening_plan.md#h2--complete-authority-and-consumer-migration).

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
  - the wallet's public records, the persona index and the device roster, answerable while locked to a `snapshot` grant;
  - retained payloads;
  - the credential catalog.
- [`policy`](src/policy.rs): installed, deny-by-default, per-principal grants for profile, operation, namespace and consent.
  - A principal matched only by the desktop user's uid or SID cannot sign or release unattended, or act as the consent channel, outside declared development mode.
    The one exception is Windows' `windows-image` matcher, which pins the consent helper to the installed executable and allows it a status grant only.
  - The `djinn-door` rule names exactly the open profile's two door keys, so every other `mere.graphshell` delegation key stays sign-only.
  - Djinn's installed grants are one template, [`packaging/djinn-grants.json.in`](packaging/djinn-grants.json.in), with the consent filled in per OS.
- [`host`](src/host.rs): Djinn's real passphrase vault behind Castellan's `PersonaeHost`.
  - Every start is locked. The first unlock builds the host on the configured or remembered profile and claims the credential catalog; later unlocks reopen it. A changed identity is refused.
  - Nothing mints a profile, and an OS-sealed (DPAPI) vault must be migrated first (`adopt`, below).
- [`wallet`](src/wallet.rs) and [`credentials`](src/credentials.rs): the wallet's acts, its public records and the credential catalog's operations, moved from Djinn unchanged in effect. The epoch derivation is pinned by a test against the epoch read directly. [`wallet::adopt`](src/wallet/adopt.rs) lists the wallet's files and carries them into a data root: same relative paths, same bytes, proved to open to the same wallet before the source copies go.
- [`agent`](src/agent.rs) (Unix) and [`agent_pipe`](src/agent_pipe.rs) (Windows): the SSH agent. Its socket or pipe admits only the owner (uid, or the client process's token SID), and only once the vault has opened. Per-use approvals become consent petitions.
- [`helper`](src/helper.rs) and [`platform::hello`](src/platform/hello.rs) (Windows): the consent channel. `djinn-custodian consent` runs in the owner's session and puts each petition to the owner through Windows Hello; see the Windows row below.
- [`adoption`](src/adoption.rs): copies the profiles of Djinn's OS-rooted Windows vault into the custodian's passphrase vault.
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
djinn-custodian adopt --config PATH --data-from DJINN-DATA-ROOT     # carry the wallet and catalog, then remove the copies there
djinn-custodian wallet-files --data-root DIR                        # the wallet's and catalog's files, NUL-separated
djinn-custodian consent --client PATH                               # Windows only: the consent helper
```

`adopt` takes either kind of Djinn vault. A passphrase vault (`vault.json`) is copied as it is and keeps its passphrase on stdin. A Windows OS-rooted vault (`auto-unlock-root.json`, no `vault.json`) can be opened only by its owner's own logon, so `adopt` runs as the owner from an elevated prompt with the service stopped, refuses when the current user's SID is not `owner.windows_sid`, reads a new passphrase twice from stdin and writes every profile into a new passphrase vault, with the roster choice. It then proves the copy opens to the same profile and master public key. If Djinn left its persisted lock, unlock Djinn once first.

`--socket PATH` binds the custody socket directly, plus `vault.agent_endpoint` when configured. It is refused outside development mode. Installers: [Linux](install-linux.sh), [macOS](install-macos.sh), [Windows](install-windows.ps1). Djinn finds an installed custodian through a client file the desktop user cannot write (`djinn::custodian::installed_path`); the file also names the agent socket for Djinn's status. A Djinn harness build (`passphrase-fd`) never reads the installed file. It reaches only a custodian named with `--custodian FILE`.

The installers point `vault.data_root` at the custodian's own state directory, so a wallet and catalog created there are the custodian's alone, and Djinn gets the wallet's public records (the persona index and the device roster) through the `WalletPublic` operation rather than from a file.

`adopt --data-from DIR` moves an existing wallet and catalog out of Djinn's data root (the directory holding `identity/` and `castellan/`). `--from` and `--data-from` may be given together or alone. `--data-from` carries every file of the wallet, secret and public, and the catalog, byte for byte at the same relative paths. It checks that the copy holds the same seed, persona index, roster and manifests, then removes the copies in `DIR`. It refuses a root that already holds a wallet or catalog, an overlap, and any symlink. If a copy cannot be removed, or changed after it was carried, it stays, adopt names it and exits non-zero. Pass the owner's data root as the last installer argument: `install-linux.sh USER BINARY [DJINN-VAULT-DIR|-] [DJINN-DATA-ROOT]`, `install-macos.sh USER BINARY TEAM DJINN-TEAM [DJINN-DATA-ROOT]`.

## Platform recipes and qualification

| OS | Launch and identity | Confinement before input | Verified here | Still to run on that host |
| --- | --- | --- | --- | --- |
| Linux | systemd system service `djinn-custodian@USER` under account `djinn-custodian-USER`. It is socket-activated: pid 1 owns the 0600 sockets `custody` and `agent` (`djinn-custodian-agent@USER.socket`), so clients expect uid 0. State lives in `/var/lib/djinn-custodian/USER`. | `no_new_privs`, not dumpable, `RLIMIT_CORE` 0, Landlock and a seccomp allowlist. Landlock ABI 6 is required in protected mode: state read-write, config read-only, no exec, no TCP, abstract sockets and signals scoped. polkit `auth_self` consent runs from a helper forked before confinement. The agent's runtime starts after confinement. | `x86_64-unknown-linux-gnu` type-check of the crate and its tests (`ring`'s C under a stub compiler; nothing linked) | Linux steps 1–8 below |
| macOS | launchd agent adopting its launchd sockets `custody` and `agent`. Both ends of the custody socket identify each other by team and identifier through the socket's audit token. | Deny-attach and no core files. Protected mode refuses to run without a valid hardened-runtime signature lacking `get-task-allow`. | Process, socket and H2 tests; the unsigned negative control for signed peers | Signed install and receiving run (macOS steps) |
| Windows | SCM service under `NT SERVICE\djinn-custodian` on pipe `\\.\pipe\djinn-custodian-<owner SID>`. The pipe's protected DACL admits only the owner, and the client requires the service SID as the pipe server. The SSH agent has its own pipe, `\\.\pipe\djinn-custodian-<owner SID>-agent` (or `vault.agent_endpoint`), with the same DACL; a connection is admitted only from the owner's token and once the vault has opened. Consent comes from a helper (`djinn-custodian consent`) that the installer registers to run as the owner at logon. The policy matches it by the installed executable's path (`windows-image`) and lets it decide petitions and nothing else. It asks the owner through Windows Hello, and any unavailability, cancel or error is a refusal. | Extension points, dynamic code, and remote or low-label image loads disabled (the service only; the helper is not confined) | `x86_64-pc-windows-gnu` type-check of the crate and its tests (as for Linux). The policy, helper loop, reconnect and adoption copy run on macOS against Windows-shaped peers. | Windows steps below (o-pc) |

**Linux** (on the Fedora ThinkPad, kernel with Landlock ABI ≥ 6):
1. Build there: `cargo +1.99.1 test --locked -p djinn-custodian -p custody-wire`, then Djinn's `custodian_client`, `quarantine_review` and `cxf_import` tests. The process tests run unconfined there; confinement runs in development (logging) mode.
2. Install: `sudo ./install-linux.sh markik target/release/djinn-custodian ~/.local/share/djinn/vault ~/.local/share/mere` (the last argument is the directory holding `identity/` and `castellan/`: the family root `~/.local/share/mere`, or Djinn's own data root if `--data-root` named one; `djinn-custodian wallet-files --data-root DIR` lists what a directory holds. The owner's account pipes those files to the custodian's account, which removes its staged copy after proving them, then the owner removes the originals). Check that `systemctl status djinn-custodian@markik.socket djinn-custodian-agent@markik.socket` both listen.
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
8. Check the wallet at rest. Before the install, note Djinn's wallet public key, a persona UUID, and one epoch-derived result (a Knot reopen). After it:
   - Djinn's data root holds no `identity/` and no `castellan/`, and no `private-epoch-bridge.json` or `wallet.json` under `personas/*/`;
   - `/var/lib/djinn-custodian/markik/data` holds them, each file `0600` in `0700` directories, and step 4 denies `markik` reading it;
   - after the unlock, Djinn's status shows the same wallet public key and persona, the resident reservoir opens that persona (it takes the sole persona from the custodian), and the Knot reopen and payload unseal of step 3 still read the content made before the move;
   - `systemctl stop djinn-custodian@markik.socket` followed by a restart reproduces all of it from the custodian's files alone.

**macOS** (needs the user's signing step, from an account the receiving account cannot use, H1-F1):
1. Sign `djinn-custodian` with the team identity, the hardened runtime and identifier `com.merely.djinn.custodian`. Sign Djinn as `com.merely.djinn`.
2. Run `sudo ./install-macos.sh USER BINARY TEAM DJINN_TEAM` and log in as USER.
3. Repeat Linux steps 3, 4, 6 and 7 there. An unsigned or differently signed peer must be refused at both ends. To move an existing wallet, pass its data root as the installer's last argument. The custodian's state directory is the same account's as Djinn's, so that move changes who holds and uses the wallet, not who can open the file; the data-protection envelope below is what remains.
4. Per-use SSH keys are refused: the macOS config has no consent channel (`deny`).

The data-protection Keychain envelope and the native consent dialog remain the [signed fixture's](../../../testing/macos-castellan-custodian-h1/README.md) pieces and are not part of this binary.

**Windows** (o-pc):
1. Run `cargo +1.99.1 test --locked -p djinn-custodian -p custody-wire`, then `.\install-windows.ps1 -User mark_ -Binary target\release\djinn-custodian.exe`.
2. Check that `sc.exe qsidtype djinn-custodian` reports unrestricted, and that `icacls` on the state directory admits only the service SID, SYSTEM and Administrators.
3. As the user, the state files and the service configuration must be unreadable and unwritable. A second local account must not open either pipe, and Djinn must refuse a pipe served by any other process. Djinn must also connect at all: the service identifies a client by opening its process (`OpenProcess` for query rights, then its token and image path), which the owner's default process ACL may refuse to a virtual service account. If it does, `accept` fails and no client is admitted; the token SID can then be read by impersonating the client at identification level, but its image has no such route, so the helper's image pin needs another source.
4. Adopt Djinn's vault. Close Djinn, unlock it once, stop the service, and from an elevated prompt of the owner's own account run `djinn-custodian.exe adopt --config C:\ProgramData\djinn-custodian\custodian.json --from <Djinn vault dir>` with the new passphrase twice. It must print the profile and master Djinn showed before. Another account, or an administrator other than the owner, must be refused. Re-run the `icacls` line for the state directory, then `Start-Service djinn-custodian` and `Start-ScheduledTask -TaskName djinn-custodian-consent`.
5. Consent helper:
   - `Get-Process djinn-custodian` must show the helper running as the owner, not as SYSTEM or the service; it must come back after logoff and logon, and after the service restarts.
   - Unlock Djinn and use a grant: Windows Hello must prompt, naming the principal and the operation (the petition carries no requesting process). Approving must succeed once per generation; cancelling, a wrong PIN until retries run out, and Hello turned off must each refuse.
   - Copy the executable elsewhere and start it as the owner with `consent --client ...`: the custodian must not admit it as the channel (it says so on its stderr; started from a shell it keeps that console, started by the task it has none). Check that the Hello dialog comes to the front over a full-screen app, since its owner is a one-pixel tool window parked off the screen.
   - Count the prompts after an unlock and record them: that count is the cost of the same-user posture.
6. Agent: with `$env:SSH_AUTH_SOCK = '\\.\pipe\djinn-custodian-<owner SID>-agent'` (or `IdentityAgent` in `ssh_config`; Win32-OpenSSH's `ssh.exe` reads the variable as a pipe name), `ssh-add -L` must list the vault's keys once unlocked and nothing before. A per-use key must prompt Hello on each signature. A second account must not connect, and the agent must keep serving after a client disconnects.
7. Kill the service. Djinn must report the identity pending, and the helper must reconnect without a restart once the service is back.

The helper is not an OS-held identity. It is the same-user posture of a polkit authentication agent: the policy admits the owner's process whose executable is the installed one, which the installer leaves writable only by administrators. Another process of the owner can still attach to it, or start the executable under its own environment, so the helper must be checked at step 5 and recorded, not assumed. A Windows presence channel the owner's other processes cannot reach waits for the CNG envelope.

## Negative controls and H2 tests (CI, no GUI)

The controls are in these files:
- [`src/tests.rs`](src/tests.rs), [`src/tests/driver_tests.rs`](src/tests/driver_tests.rs), [`src/tests/h2_tests.rs`](src/tests/h2_tests.rs), [`src/tests/wallet_tests.rs`](src/tests/wallet_tests.rs) and [`tests/process.rs`](tests/process.rs);
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
- an adopted wallet is byte for byte the original: the seed's public key, the epoch derivations (public-only, bound and unbound), the persona index and the roster are identical across a cold reopen, the custodian's files are `0600` in `0700` directories, and nothing of the wallet or catalog stays in Djinn's root, though Djinn's own files do;
- an adoption that cannot proceed changes nothing: no wallet, an occupied custodian root, an overlap, a symlink, a copy that cannot be made, and a source file that changed after it was carried;
- the wallet's public records are disclosed over the wire while locked and only to a `snapshot` grant, with no seed or secret file in the answer, and a custodian process serves them from an adopted wallet;
- a protected custodian refuses a self-bound socket;
- Djinn never falls back to a vault of its own when the custodian is locked or gone;
- each H2 operation answers from the vault or wallet opened directly, and is refused without its grant or while locked;
- the epoch derivation is the one Djinn made;
- the door rule releases exactly the open profile's door keys, and Djinn's own door function agrees with it;
- per-use SSH signing is petitioned to the consent channel, and a denial reaches the waiting client at once;
- a `windows-image` consent channel is validated (absolute path, status grant only, never the account alone) and matched ahead of the account's own rule, and another image of the same user is an ordinary requester ([`src/tests.rs`](src/tests.rs));
- the Windows helper, over real sockets with Windows-shaped peers ([`src/helper/tests.rs`](src/helper/tests.rs)):
  - it is petitioned for each release, and neither the requester nor another image of the same user can decide it;
  - an approval releases, a refusal is final, and consent lasts a generation;
  - a session not admitted as the channel is not served, and a lost custodian ends the loop;
  - it connects again until admitted and reports a failure streak once;
- a sealed-record vault round-trips into a passphrase vault with its slots and roster choice, and an existing vault is never replaced ([`src/adoption.rs`](src/adoption.rs));
- Djinn's lock holders forget their keys on a custodian lock and re-derive through it on unlock;
- quarantine review and CXF import go through the custodian's catalog behind Djinn's native consent.
