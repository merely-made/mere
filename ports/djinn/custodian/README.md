# Djinn custodian

Status: H1 portable cut on main. The custodian runs as its own process and serves the custody contract on macOS here; Linux and Windows builds type-check, with their qualification steps below still to run on those hosts. Djinn routes two consumers through it when one is installed. Every other vault use stays in Djinn until H2.

`djinn-custodian` is the independently launched process that opens Djinn's Castellan vault (custody isolation plan, §5). Only this process unlocks it, locks it, signs with its root or a root-derived key, and releases reviewed derived keys. Djinn and every other client go through [`custody-wire`](wire) over authenticated local IPC and never link castellan.

## Shape

- [`custody-wire`](wire) implements the §5.6 contract once:
  - the peer principal comes from the OS connection: `SO_PEERCRED`, `getpeereid`, the macOS audit-token code signature, or the named-pipe token SID; a hello carries only a version and a nonce;
  - a session opens with the server instance, lock generation, granted scopes and limits;
  - each operation is typed and names its request id and expected generation;
  - consent decisions are accepted only from the trusted channel, for one digest, request and generation;
  - outcomes are typed, and a lost connection is `Unknown`;
  - lock and revocation are monotonic facts.

  Frames keep the Graphshell door's wire format, and their buffers clear on drop. The client verifies the custodian's OS identity before it sends a hello.
- [`policy`](src/policy.rs): installed, deny-by-default, per-principal grants for profile, operation, namespace and consent. A principal matched only by the desktop user's uid or SID cannot sign or release unattended, or act as the consent channel, outside declared development mode.
- [`vault`](src/vault.rs): Djinn's real passphrase vault. Every start is locked. Unlock reopens the configured or remembered profile and refuses a changed identity. Nothing ever mints a profile, and an OS-sealed (DPAPI) vault must be migrated first.
- [`core`](src/core.rs): the state machine without I/O, driven by [`driver`](src/driver.rs) over any listener. Lock, unlock and revocation each advance one generation, cancel pending petitions and broadcast a fact. A lock reports separately whether memory locked, the drain finished and the barrier persisted.
- [`delegation`](src/delegation.rs): the H1 signing consumer. It decodes a Graphshell (`mere.graphshell`) insigne delegation certificate or revocation, rebuilds it byte-for-byte, then checks the audience, the issuer and the scope key before signing inside custody.
- [`platform`](src/platform): one launcher, peer check and confinement recipe per OS.

The `synthetic` module is the 2026-10-22 core behind the signed macOS receiving fixture's C ABI (`native-host`), kept so that harness still builds.

## Running

```sh
djinn-custodian --config /etc/djinn-custodian/USER.json            # listener from systemd/launchd
djinn-custodian init  --config PATH                                 # new vault; passphrase twice on stdin
djinn-custodian adopt --config PATH --from DJINN-VAULT-DIR          # copy and reopen to the same identity
```

`--socket PATH` binds a socket directly and is refused outside development mode. Installers: [Linux](install-linux.sh), [macOS](install-macos.sh), [Windows](install-windows.ps1). Djinn finds an installed custodian through a client file the desktop user cannot write (`djinn::custodian::installed_path`).

## Platform recipes and qualification

| OS | Launch and identity | Confinement before input | Verified here | Still to run on that host |
| --- | --- | --- | --- | --- |
| Linux | systemd system service `djinn-custodian@USER` under account `djinn-custodian-USER`. Socket-activated: pid 1 owns the 0600 socket, so clients expect uid 0. State lives in `/var/lib/djinn-custodian/USER`. | `no_new_privs`, not dumpable, `RLIMIT_CORE` 0, Landlock (ABI 6 required in protected mode: state read-write, config read-only, no exec, no TCP, abstract sockets and signals scoped) and a seccomp allowlist. polkit `auth_self` consent runs from a helper forked before confinement. | `x86_64-unknown-linux-gnu` type-check of the crate and its tests | Linux steps 1–6 below |
| macOS | launchd agent adopting its launchd socket `custody`. Both ends identify each other by team and identifier through the socket's audit token. | Deny-attach, no core files; protected mode refuses to run without a valid hardened-runtime signature lacking `get-task-allow` | Process and socket tests; the unsigned negative control for signed peers | Signed install and receiving run (macOS steps) |
| Windows | SCM service under `NT SERVICE\djinn-custodian` on pipe `\\.\pipe\djinn-custodian-<owner SID>`. The pipe's protected DACL admits only the owner, and the client requires the service SID as the pipe server. | Extension points, dynamic code, and remote or low-label image loads disabled | `x86_64-pc-windows-gnu` type-check of the crate and its tests | Windows steps below (o-pc) |

**Linux** (on the Fedora ThinkPad, kernel with Landlock ABI ≥ 6):
1. Build there: `cargo +1.99.1 test --locked -p djinn-custodian -p custody-wire`. The process tests run unconfined there; confinement runs in development (logging) mode.
2. `sudo ./install-linux.sh markik target/release/djinn-custodian ~/.local/share/djinn/vault`, then `systemctl status djinn-custodian@markik.socket`.
3. Start once with `"development": true` in `/etc/djinn-custodian/markik.json`. Exercise unlock, release and sign through Djinn. Read `journalctl -k | grep audit` for any seccomp `SECCOMP_RET_LOG` hits and add only the calls that were really needed. Then turn development off and repeat with the filter enforcing.
4. As `markik`: try to read `/var/lib/djinn-custodian/markik`, `ptrace` and read `/proc/PID/mem` of the custodian, connect as another user, squat the socket path, and replace the unit or config. Each must be denied.
5. polkit: a release prompts once per generation and a delegation every time. Cancel must refuse. The prompt must name the requesting process.
6. Kill the custodian. Djinn must report the identity pending. After a restart the custodian must be locked with a fresh instance.

**macOS** (needs the user's signing step, from an account the receiving account cannot use, H1-F1): sign `djinn-custodian` with the team identity, hardened runtime and identifier `com.merely.djinn.custodian`. Sign Djinn as `com.merely.djinn`. Run `sudo ./install-macos.sh USER BINARY TEAM DJINN_TEAM`, log in as USER, and repeat steps 4–6 there: an unsigned or differently signed peer must be refused at both ends. The data-protection Keychain envelope and the native consent dialog remain the [signed fixture's](../../../testing/macos-castellan-custodian-h1/README.md) pieces and are not part of this binary.

**Windows** (o-pc): `cargo +1.99.1 test --locked -p djinn-custodian -p custody-wire`, then `.\install-windows.ps1 -User mark_ -Binary target\release\djinn-custodian.exe`. Check that `sc.exe qsidtype djinn-custodian` reports unrestricted and that `icacls` on the state directory admits only the service SID, SYSTEM and Administrators. As the user, the state files and the service configuration must be unreadable and unwritable. A second local account must not open the pipe, and a pipe served by any other process must be refused by Djinn. Consent is `deny` there until a Windows presence channel exists.

## H1 negative controls (CI, no GUI)

The negative controls are in [`src/tests.rs`](src/tests.rs), [`src/tests/driver_tests.rs`](src/tests/driver_tests.rs) and [`tests/process.rs`](tests/process.rs), with Djinn's side in [`ports/djinn/tests/custodian_h1.rs`](../tests/custodian_h1.rs). They cover:
- a wrong profile, an adjacent namespace, a mislabelled audience and an arbitrary-bytes signing oracle, each refused;
- forged approval fields, self-approval and petitions with the wrong digest, request or generation, each refused;
- a lock during a pending petition invalidates it, and a barrier failure degrades the lock without unlocking;
- a stale instance or generation is refused, and revocation closes sessions and refuses readmission;
- the handshake deadline, connection budgets and consent expiry;
- custodian loss is `Unknown`, and a restart is locked with a fresh instance;
- adoption keeps the identity and leaves the source untouched;
- a protected custodian refuses a self-bound socket;
- Djinn never falls back to its own vault when the custodian is locked or gone.
