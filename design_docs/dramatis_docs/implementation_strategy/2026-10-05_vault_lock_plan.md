# Vault Lock Plan

**Date**: 2026-10-05
**Status (2026-10-05)**: assessed; forks 1 to 15 ruled (rulings 1 to 16 in
§3), the rest wait for Mark. No code changed. Chatelaine P4 (CXF import) waits on this plan (chatelaine
rulings 64, 65).
**Scope**: the resident's secrets can be locked. While locked, no secret
material can be reached through the vault or the resident's derived keys.
Unlocking takes a user act. Every consumer (the SSH agent, castellan's item
store, OTP, the Secret Service, the projection) behaves as ruled while
locked.

**Related**:

- [chatelaine and CXF import plan](2026-10-01_chatelaine_cxf_plan.md):
  rulings 63 to 65 and the finding "the vault never locks".
- [dramatis tier architecture](../technical_architecture/2026-09-30_dramatis_tier_architecture.md):
  the tier's invariants (1, custody; 12, quarantine).
- [protocol architecture plan](../../mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md):
  §3.6 and §3.7 describe `UnlockTier` as custody tiers, and its line 349
  claims swap leakage is defended. Neither is true today.
- [persona wallet carry layer plan](../../mere_docs/implementation_strategy/2026-06-25_persona_wallet_carry_layer_plan.md):
  the "one unlock ladder" rule (:358-362), and Meerkat's 2026-07-04 "Lock
  now" (:788-792).

---

## 1. Why

Mark, 2026-10-04, on finding that the vault never locks: **"Hey wait, we
need to be able to lock the vault lmao. Otherwise it's just a big room"**.
Today a resident decrypts every secret at logon and keeps it until the
process exits.

## 2. What exists (assessed 2026-10-05)

A read-only lane (Opus) read the code at `de8214c7`. The load-bearing
claims were re-checked in code (marked *checked*); the lane's other claims
it marked measured or reading.

- **One unlock ladder.** `bootstrap.rs:35-61, 92-117` picks a passphrase
  from `PERSONAE_PASSPHRASE`, or otherwise AutoOs. Every caller goes
  through it: the agent bin, the CLI, djinn, graphshell, distillery and
  roster. On Windows, AutoOs unwraps a DPAPI-held random root with no
  prompt (`startup_unlock.rs:213-248`, `CryptUnprotectData` *checked*), so
  any process running as the user can open the vault at rest.
  `StartupUnlockMode::{AutoOs, Prompt, Locked}` is declared but the ladder
  never reads it; only pandect's wallet store does. `passphrase_root.rs`,
  the passphrase wrap over the same root, has no caller.
- **No lock surface.**
  - `IdentityVault { storage, current }` (`vault.rs:369-372`) has no lock
    or close method. Castellan's `PersonaeHost` sets
    `lock: VaultLockView::Unlocked` (`authority.rs:201`, *checked*), and
    nothing sets `Locked`.
  - The whole profile is one sealed record, so opening it decrypts every
    slot whatever its tier.
  - The storage keeps its own key: `SealedRecordStorage.key` is held, and
    `PassphraseEncryptedStorage` caches the Argon2 KEK. Dropping the
    profile alone is therefore not a lock; `switch_profile` reloads
    silently.
- **Residue.** `PlaintextProfile` is not zeroized (`profile_wire.rs:22-29`,
  *checked*), and `plaintext_to_slot` clones each payload, so each load and
  save frees an uncleared copy of the master seed and every slot. The DPAPI
  plaintext buffer is freed uncleared too.
- **The agent decodes per request** (`agent.rs:150-156`), and
  `certificate_for` copies the master seed into an `InMemoryProvider` on
  every listing (`agent.rs:164-167`). ssh-key zeroizes its private keys on
  drop. Under chatelaine ruling 63, ring's RSA key is built per signature
  and freed without clearing.
- **Keys derived from the vault and held elsewhere.**
  - `CastellanResident` holds the record and freshness keys for the
    process's life (`castellan/src/resident.rs:38-53`, *checked*), and
    every chatelaine item, OTP seed and Secret Service secret decrypts
    under them.
  - Distillery uses the persona's master keypair as its P2P transport
    identity (`distillery/src/installed.rs:265-268`, *checked*).
  - Knot's signing seed comes from pandect's startup-unlocked vault
    (`djinn/src/resident_knot.rs:74-81`).
  - The graphshell app opens its own vault.
- **`UnlockTier` was built as consent, not custody.** The approval broker
  treats PerUse as one approval per signature and ShortTtl as an approval
  cache (`signing.rs:339-366`), and the resident routes the agent through it
  (`authority.rs:191-194`, *checked*). Every tier is fully decrypted from
  the moment the vault opens. Its documentation (`vault.rs:38-43, 139-152`)
  says custody.
- **Secret Service** (castellan, test-served only): Lock and Unlock are
  label flips on sets that start empty, `Unlock` never prompts, and only
  `plain` sessions are offered.
- **The legacy `graphshell-device-host`** (source last at `15411a3c`) never
  locks either.
- **djinn senses idle** through `GetLastInputInfo` on Windows
  (`conditions.rs:521-540`). It reads unknown idle as "just touched", which
  for a lock would fail open. djinn restarts itself within 5 s of any exit
  (`install-windows.ps1:129-139`), and AutoOs then reopens the vault
  silently.
- **Memory locking and crash-dump exclusion are absent** in mere. `secrecy`
  provides neither, and `secrets`, `memsec` and `region` are not in the
  lock.
- **Prior art.** These are the lane's recall without the web; it gave a
  confidence for each.
  - OpenSSH `ssh-agent` gates signing behind `ssh-add -x`/`-X` (messages 22
    and 23) but keeps its keys decrypted. ssh-agent-lib 0.6.0 exposes
    `Session::lock` and `unlock`, unsupported by default.
  - Secret Service 0.2 has `Locked`, Prompt objects and `IsLocked`.
  - KeePassXC locks on idle, session lock and lid close; locking drops the
    database key, and quick unlock lasts until restart.
  - 1Password and Bitwarden lock on idle, system lock and sleep. Biometric
    unlock lasts only until restart, and 1Password's SSH agent prompts when
    locked.
  - The macOS Keychain has per-keychain idle and sleep locks.
  - Windows Credential Manager and DPAPI have no lock; user presence is
    Windows Hello.
  - The OS events: WTS session notifications, `WM_POWERBROADCAST`, and
    logind's Lock, Unlock and `PrepareForSleep` (zbus is in the lock).

## 3. Forks for Mark

Each comes with the lane's recommendation first. Mark's principle governs:
lean on standards and maintained libraries, and flag anything that would be
our own security mechanism.

1. **What "lock" covers.**
   - **Recommended:** every secret the resident exercises: the personae
     vault (profile and storage key) and `CastellanResident`'s record and
     freshness keys.
   - The personae vault only, which leaves chatelaine, OTP and the Secret
     Service open.
   - Also Distillery, Knot and pairing, which takes djinn offline.
2. **Distillery's transport identity is the master key.**
   - **Recommended:** it stays resident while sync runs, recorded as the
     master remaining in memory under lock; replacing it is its own plan.
   - Derive a separate transport key, which changes peer identity.
   - Stop sync on lock.
3. **Triggers.**
   - **Recommended:** an explicit intent, the OS session locking, suspend,
     and an idle timeout, each a per-device setting. Idle detection fails
     closed.
   - Explicit only.
   - Explicit plus idle.
4. **What lock does to memory.**
   - **Recommended:** drop the profile and the storage key, and fix the
     residue (zeroize `PlaintextProfile`, the DPAPI buffer, and the
     per-listing seed copy).
   - Drop only.
   - Add page locking and dump exclusion, which is our own mechanism (§5).
5. **What unlocking takes.**
   - **Recommended:** a passphrase (crypto-bound, through the existing
     `passphrase_root`) or OS consent (Windows Hello as a gate); a silent
     DPAPI re-unlock only if Mark wants the convenience.
   - The same ladder, silent: a gate only.
   - Passphrase only.
6. **At rest.** Under AutoOs any same-user process can open the vault at
   rest.
   - **Recommended:** keep AutoOs at startup, gate re-unlock behind a user
     act, and persist "locked" across restarts through
     `StartupUnlockMode::Locked`.
   - Prompt mode, dropping the DPAPI wrap.
   - Leave as is.
7. **`PERSONAE_PASSPHRASE` in the process environment**
   (`bootstrap.rs:54-61`).
   - **Recommended:** a resident with lock enabled never takes the
     passphrase from the environment; the CLI and tests keep it.
   - Clear it after startup.
   - Accept it.
8. **The agent while locked.**
   - **Recommended:** OpenSSH semantics: an empty identity list; sign, add
     and remove fail, with the reason logged.
   - List public keys and queue an unlock prompt (1Password-style).
   - An error.
9. **ssh-agent lock messages.**
   - **Recommended:** `ssh-add -x` locks the vault; `-X` is refused over
     the wire, so unlocking happens only on the resident's own surface.
   - Implement both with OpenSSH's lock-password hash, which is our own
     mechanism.
   - Leave both unsupported.
10. **Castellan's surfaces while locked.**
    - **Recommended:** typed `Locked` refusals. A Locked card with a
      portable Lock intent and a native-only Unlock; OTP refused; Secret
      Service collections report Locked, and `Unlock` returns a real
      Prompt.
    - Queue petitions until unlock.
    - Show nothing.
11. **What stays visible while locked.**
    - **Recommended:** a secret-free snapshot kept at lock time (labels and
      public keys).
    - Only "locked".
    - A separately keyed metadata index, which is a format change.
12. **`UnlockTier`.**
    - **Recommended:** rule that it means consent, which is what the broker
      enforces, and fix the docs; the vault-wide lock is the custody
      control.
    - Make the tiers custody, which needs per-slot sealing, a new at-rest
      format.
    - Also give the standalone `personae-agent` a confirmation path.
13. **How lock state reaches every holder.**
    - **Recommended:** state inside `IdentityVault` (accessors return
      `Locked`; a breaking API change) plus a broadcast from `PersonaeHost`
      that derived-key holders obey.
    - Vault state only.
    - Broadcast only.
14. **A persona switch while locked.**
    - **Recommended:** refused.
    - Switching unlocks the target.
15. **Pandect's wallet ladder** (Knot's seed).
    - **Recommended:** the lock also relocks the wallet stores djinn opened,
      for "one unlock ladder".
    - Personae only, with the gap recorded.
16. **Failed-unlock throttling.**
    - **Recommended:** rely on Argon2id's cost and add no counter of our
      own.
    - An OpenSSH-style growing delay.

Open questions:
- the default idle window (§3.6 used 15 minutes for ShortTtl);
- whether macOS is in scope (it has no AutoOs backend, so passphrase only);
- whether the standalone `personae-agent` is still a target;
- whether a resident lock should reach Turnstone, Knot and the graphshell
  app, which open the vault in their own processes;
- which sibling repos call `current_profile()` and `slot()`, whose
  signatures would change (a scope question for Mark);
- naming hibernation and pagefile exposure in a threat statement.

### Rulings

**Ruling 1.** *What does "lock" cover?* Options: every secret the resident
exercises (the personae vault, profile and storage key, plus
`CastellanResident`'s record and freshness keys); the personae vault only;
everything, Distillery, Knot and pairing included. Mark: **"Every secret
the resident uses (Recommended)"**.

**Ruling 2.** *Distillery's P2P transport identity is the persona's master
keypair. What happens to it under lock?* Options: it stays while sync runs;
derive a transport key now; stop sync on lock. Mark: **"Derive a transport
key now"**. Follows: Distillery gets a key derived for transport, so the
master can leave memory on lock; its peer identity changes, and existing
peers must learn the new one. Whether the transport key itself stays
resident under lock is a follow-up question.

**Ruling 3.** *What locks the vault?* Options: an explicit intent, the OS
session locking, suspend and an idle timeout, each per device, with idle
failing closed; explicit only; explicit plus idle. Mark: **"Intent, OS
lock, sleep, idle (Recommended)"**.

**Ruling 4.** *What does unlocking take?* Options: a passphrase (through
`passphrase_root`) or Windows Hello as a gate, with silent DPAPI re-unlock
only by opt-in; passphrase only; silent, as today. Mark: **"1, and i'm sure
there are mac and/or linux and/or mobile integrations like biometrics and
passkeys and passwords and pins and whatnot"**. Follows: the passphrase or
OS presence, with no silent re-unlock unless opted into. *Reading, not
ruled:* one unlock interface with a method per platform (Windows Hello;
macOS Touch ID; Linux biometrics through the desktop's own stack; passkeys,
PINs and mobile biometrics later); which methods this plan builds first is
a follow-up question.

**Ruling 5.** *At rest, AutoOs lets any same-user process open the vault
with no prompt, and djinn restarts within 5 s of exiting, so a killed
locked resident would reopen silently.* Options: AutoOs at logon with the
lock persisting across restarts (`StartupUnlockMode::Locked`); always
prompt; leave as is. Mark: **"AutoOs at logon, lock persists
(Recommended)"**.

**Ruling 6.** *What does locking do to memory?* Options: drop the profile
and storage key and fix the residue (`PlaintextProfile`, the DPAPI buffer,
the per-listing seed copy) with `zeroize`, proven by a tracking-allocator
test; drop only; also page locking and dump exclusion. Mark: **"Drop keys,
fix residue (Recommended)"**.

**Ruling 7.** *`PERSONAE_PASSPHRASE` would leave the unlock credential in
the process environment.* Options: residents with lock enabled never read
it (the CLI and tests keep it); clear it after startup; accept it. Mark:
**"Residents never read it (Recommended)"**.

**Ruling 8.** *What does the SSH agent do while locked?* Options: OpenSSH's
behaviour (no identities listed; sign, add and remove fail, with the
reason logged); list and prompt; error everything. Mark: **"OpenSSH's
behaviour (Recommended)"**.

**Ruling 9.** *ssh-agent's lock messages (`ssh-add -x`/`-X`), which
ssh-agent-lib exposes unsupported by default?* Options: `-x` engages the
vault lock and `-X` is refused over the wire; both, OpenSSH-style, with a
lock-password hash of our own; neither. Mark: **"-x locks; -X refused
(Recommended)"**. Follows: unlocking happens only on the resident's own
surface.

**Ruling 10.** *What do castellan's surfaces do while locked?* Options:
typed `Locked` refusals (a Locked card with a portable Lock and a
native-only Unlock; OTP refused; Secret Service collections Locked, with
`Unlock` returning a real Prompt); queue until unlock; show nothing. Mark:
**"Typed Locked refusals (Recommended)"**.

**Ruling 11.** *What can be seen while locked?* Options: a secret-free
snapshot kept at lock time (labels and public keys); only "locked"; a
separately keyed metadata index. Mark: **"A secret-free snapshot
(Recommended)"**.

**Ruling 12.** *Is `UnlockTier` custody or consent?* Options: consent, with
the docs fixed and the vault-wide lock as the custody control; custody,
with per-slot sealing; consent plus a confirmation path in the standalone
`personae-agent`. Mark: **"Consent; fix the docs (Recommended)"**.

**Ruling 13.** *How does lock state reach every holder?* Options: state in
`IdentityVault` (accessors return `Locked`, which breaks `current_profile()`
and `slot()`, Knot first) plus a broadcast from `PersonaeHost` that
derived-key holders obey; vault state only; broadcast only. Mark: **"Vault
state plus a broadcast (Recommended)"**.

**Ruling 14.** *A persona switch while locked?* Options: refused; switching
unlocks the target. Mark: **"Refused (Recommended)"**.

**Ruling 15.** *Does the vault lock reach pandect's wallet store (Knot's
signing seed)?* Options: the lock also relocks the wallet stores djinn
opened ("one unlock ladder"); personae only. Mark: **"Lock relocks it too
(Recommended)"**. Follows: Knot's sync pauses while locked.

**Ruling 16** *(asked as: is the standalone `personae-agent` still a target,
with options dev tool, target with askpass later, retire).* Mark: **"hm.
would it make sense for djinn to take on those capabilities, so
personae-agent just becomes a djinn feature set/implementation/agent? i
have been wondering about our testing framework for djinn too... and it's
the daemon/cli thing for the stack, no? seems important"**. Follows: the
question is put back as djinn absorbing the agent, with djinn's test
harness as its own question. Evidence: `personae-agent` is a 210-line host
of personae's agent library with Windows, macOS (launchd) and Linux
installers; djinn's Windows installer already retires its task; on macOS
and Linux it remains the deployed agent until the pairing plan's D2.

## 4. Phases

Drafted from the assessment; set once the forks are ruled.

- **L1 — personae can lock.** `IdentityVault::lock()` drops the profile and
  the storage key, and every accessor that reaches secret material returns
  `Locked`. Done when:
  - [ ] each accessor has a test, and each test fails if its guard is
        removed;
  - [ ] a no-residue instrument (a tracking allocator) finds no canary key
        live or freed uncleared after a lock, and fails on today's
        `profile_wire` clones as its positive control;
  - [ ] after an unlock by the ruled method, slots are byte-identical and an
        Ed25519 signature verifies exactly as before, while a wrong
        credential leaves the vault locked.
- **L2 — every consumer obeys.** Done when:
  - [ ] over the isolated named pipe, the agent behaves as ruled while
        locked, and `ssh-add -x`/`-X` as ruled;
  - [ ] `CastellanResident` drops its keys, so items and the OTP gate return
        `Locked`;
  - [ ] the snapshot reports Locked, and Unlock is native-only;
  - [ ] Secret Service collections report Locked, `GetSecret(s)` refuses,
        and `Unlock` returns a Prompt, proven on the ThinkPad with
        `secret-tool` under a disposable bus.
- **L3 — triggers.** Done when:
  - [ ] each ruled trigger is proven, idle with an injected clock;
  - [ ] there are real receipts for Windows `Win+L` and suspend, and for
        Fedora `loginctl lock-session` and suspend;
  - [ ] an unknown idle fails closed;
  - [ ] a locked resident restarted by the launcher comes back as ruled.
- **L4 — docs and gates.** Done when:
  - [ ] `UnlockTier`'s docs, the protocol plan's §3.6 and §3.7, and the
        tier invariants match the rulings;
  - [ ] the gates pass, with Windows-only and Linux-only code each compiled
        on its own target;
  - [ ] PID 53336 is untouched; deployment is Mark's step.

## 5. Findings

**2026-10-05.** Things that would be our own security mechanism, flagged
under Mark's principle:
- an OpenSSH-style lock-password hash;
- binding a Windows Hello key to the vault key;
- a page-locking or zeroizing allocator;
- per-slot sealing tiers;
- an unlock-attempt counter.

The standards and libraries to lean on instead: the ssh-agent messages 22
and 23 through ssh-agent-lib; Secret Service 0.2's lock and Prompt
semantics; `zeroize` and `secrecy`, both in the lock; and the OS session,
power and presence APIs.

## 6. Progress

**2026-10-05.** Assessed by a read-only lane (Opus); the load-bearing claims
re-checked in code. Nothing built. Next: Mark's rulings on §3.
