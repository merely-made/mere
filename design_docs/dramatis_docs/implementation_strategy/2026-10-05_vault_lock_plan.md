# Vault Lock Plan

**Date**: 2026-10-05
**Status (2026-10-06)**: rulings 1 to 39 in §3; the threat statement is
still open. L1 (personae can lock) landed on `main` (`2556a20c`). L2 (every
consumer obeys) is next. The [dramatis repo plan](2026-10-06_dramatis_repo_plan.md)
moves this code later. Chatelaine P4 (CXF import) waits on this plan
(chatelaine rulings 64, 65).
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
- [dramatis repo plan](2026-10-06_dramatis_repo_plan.md): moves the
  vault's custody from personae into castellan and the tier out of mere;
  sequenced after this plan's L2 at the earliest.

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

**Ruling 17.** *How should djinn take on the CLI agent?* Options: djinn
hosts it, with the standalone bin and its three installers retired after
the pairing plan's D2 (the M4 moves then); an agent-only djinn mode; keep
both. Mark: **"djinn hosts it; retire after D2 (Recommended)"**. Follows:
the vault lock applies to one agent, djinn's.

**Ruling 18.** *djinn's test harness (each lane this week rebuilt its own
two-resident harness): when?* Options: its own plan, before the lock
build; the lock plan's first phase; later. Mark: **"Its own plan, before
the lock build (Recommended)"**. Follows: a shared djinn harness (spawning
residents, isolation roots, pipes, the installed-agent walls, receipts) is
assessed and built first; the lock's receipts are its first customer.

**Ruling 19.** *Failed-unlock throttling?* Options: Argon2id's cost only;
a growing delay of our own. Mark: **"Argon2id's cost only
(Recommended)"**.

**Ruling 20.** *The default idle window?* Options: 15, 5 or 30 minutes; off
by default. Mark: **"15 minutes (Recommended)"**. Follows: per device and
changeable.

**Ruling 21.** *Which unlock methods does the first build carry?* Options:
the passphrase plus Windows Hello; the passphrase only; every desktop
platform's method. Mark: **"Passphrase + Windows Hello (Recommended)"**.
Follows: other platforms' methods come later behind the same interface.

**Ruling 22.** *Is macOS in scope?* Options: yes, with the pairing plan's
D2; yes, now, through the standalone agent; Windows and Linux only. Mark:
**"Yes, with D2 (Recommended)"**.

**Ruling 23.** *Should a resident lock reach Turnstone, Knot and the
graphshell app, which open the vault in their own processes?* Options:
they follow the resident's broadcast and the persisted lock; record the
gap. Mark: **"They follow the resident (Recommended)"**. Follows: the work
reaches those repos; which of their calls break under ruling 13 is mapped
at the build's start.

*2026-10-06 annotation to ruling 15:* the dramatis repo plan's ruling D8
moves pandect's wallet secrets into castellan, so this lock reaches them
there once that lands. Until then ruling 15 stands as written. Its D18
finishes this plan first.

*2026-10-06 annotation to rulings 23, 35 and 36:* the dramatis repo plan's
ruling D5 has graphshell, Turnstone, woodshed, hocket and knot-editor call
djinn instead of opening the vault. Once that lands, "following the
resident" means djinn's answers say Locked; until then these rulings stand
as written.

**Ruling 24.** *Under lock, does Distillery's own transport key stay?*
Options: it stays and sync continues; it is dropped and sync pauses. Mark:
**"Stays; sync continues (Recommended)"**. Follows: the transport key only
identifies the device to peers and opens no secret.

Rulings 25 to 36 were asked on 2026-10-05 from L1's checkpoint A (§6),
after its claims were checked in code and its instrument rerun.

**Ruling 25.** *Where does the locked state live? Dropping the profile is
not a lock, because the storage keeps its own key and `switch_profile`
reloads silently.* Options: the vault drops its profile and the storage
drops its key, the storage trait gaining lock and unlock, so every record
call fails closed while locked (only personae implements the trait, so no
sibling breaks); the vault only, holding the storage optionally, with the
caller handing it back at unlock and so carrying the whole ladder. Mark:
**"Vault and storage both (Recommended)"**.

**Ruling 26.** *What shape do the secret accessors (`current_profile`,
`slot`) take while locked?* Options: `Result<_, Locked>`, as ruling 13
reads (about 25 production call sites in mere, Turnstone and graphshell
change); `Option`, which loses the reason; a guard
(`vault.unlocked()?`), with fewer signatures changing downstream. Mark:
**"Result<_, Locked> (Recommended)"**.

**Ruling 27.** *No passphrase-wrapped root exists anywhere yet
(`passphrase_root` has no caller), so a persisted lock with no Windows
Hello and no enrolled passphrase could never be undone.* Options: `lock()`
refuses unless at least one unlock method is available on the device; the
first enabling of lock enrols a passphrase. Mark: **"Refuse lock without
one (Recommended)"**.

**Ruling 28.** *Where does the Windows Hello gate live? `windows` 0.62.2 is
in the lock through djinn, and `UserConsentVerifier` needs only feature
flags.* Options: personae, behind a feature, so only personae can mint the
OS-presence token after Hello succeeds (one lock edge, no new package);
djinn, which needs a public constructor for the token, so any caller could
claim presence. Mark: **"In personae (Recommended)"**.

**Ruling 29.** *How is "locked" reported?* Options: a new
`IdentityError::Locked` variant, which breaks any exhaustive match (to be
checked in the build); a separate error type. Mark: **"New IdentityError
variant (Recommended)"**.

**Ruling 30.** *Castellan hands a clone of `SealedRecordStorage` to every
item store (`resident.rs:56`). Do the clones share one key cell?* Options:
yes, so locking any copy locks every copy (a change to what `Clone` means
there, documented); no, each holder locks its own. Mark: **"Yes, one shared
cell (Recommended)"**.

**Ruling 31.** *How does a lock reach the holders of vault-derived keys?*
Options: synchronous holder hooks, so `lock()` returns only after every
registered holder has dropped its keys, plus a `tokio` watch channel for
observers (the UI, the status route); the watch channel alone, which
leaves a window where castellan still holds its keys. Mark: **"Sync hooks +
watch (Recommended)"**.

**Ruling 32.** *Where is the persisted lock (ruling 5) checked at open?
hocket reads the DPAPI root through `startup_unlock` directly.* Options:
in `startup_unlock`'s root loaders, the one point hocket passes through
(advisory against same-user processes, which can read the DPAPI file
anyway; the threat statement says so); in `bootstrap` and `roster` only,
which hocket bypasses. Mark: **"In startup_unlock's loaders
(Recommended)"**.

**Ruling 33.** *argon2 0.5.3 frees its ~19 MiB of working memory uncleared
after deriving the passphrase key; the final blocks suffice to recompute
it (read in source, not measured).* Options: call argon2's own
`hash_password_into_with_memory` with a buffer we zeroize, enabling its
`zeroize` feature (a feature edge), and ledger the gap; ledger only. Mark:
**"Public API + ledger (Recommended)"**. Follows: upstream candidates
ledger item 9; argon2 0.6.0-rc.8 does not clear the memory either.

**Ruling 34.** *The lane's `zeroizing_json` (a serializer that sizes its
output first so nothing reallocates, and a serde visitor that clears each
buffer it outgrows) removes the residue serde_json's reallocation leaves.
It is memory hygiene, not crypto, and its bytes equal serde_json's.*
Options: keep it, guarded by the leak test and the equality test; drop it
and record the gap; keep it and ledger it. Mark: **"Keep it
(Recommended)"**.

**Ruling 35.** *Turnstone falls back to an unsealed seed or a
process-local key when its vault will not open (`identity.rs:125-135`), so
under a persisted lock it would quietly start as a different identity.*
Options: tell "locked" from "unavailable" and, while locked, run with the
identity pending and never use the fallback (kept for a missing vault);
keep the fallback and record that a lock re-roots Turnstone. Mark: **"Wait
for unlock (Recommended)"**.

**Ruling 36.** *Ruling 23 named Turnstone, Knot and the graphshell app; the
caller map also found hocket and woodshed, which open the storage or the
root directly. Should a live lock reach them?* Options: the same as ruling
23 (the resident's broadcast and the persisted lock), widening the work
into those repos; the persisted lock only, refusing their next open, with
the gap recorded. Mark: **"Same as ruling 23 (Recommended)"**.

Rulings 37 to 39 were asked on 2026-10-06 from L1's checkpoint B (§6).

**Ruling 37.** *chacha20poly1305 0.11's own `zeroize` feature is off in our
lock, so the cipher's internal key copy is not cleared on drop. The
measured leak was fixed by lending the key instead of copying it; the
feature alone did not fix that one.* Options: enable it too (the library's
own feature, like ruling 33; two feature edges, no new package); ledger
only. Mark: **"Enable it too (Recommended)"**.

**Ruling 38.** *After a restart under a persisted lock (L3), the resident
knows nothing about the persona: the only list of names and public keys is
inside the sealed profile.* Options: only the profile id until unlock (no
new file at rest); a public snapshot persisted beside the vault at lock
time (persona names and public keys at rest, outside the seal); the same
snapshot sealed with DPAPI, separately from the vault root. Mark: **"Only
the profile id (Recommended)"**. Follows: a resident that starts locked
shows a Locked card with no names or keys; ruling 11's snapshot is the
in-memory one kept at lock time.

**Ruling 39.** *Ruling 27 refuses `lock()` without an unlock method; Linux
has no Hello, and nothing can enroll a passphrase (the storage hides the
root `passphrase_root` wraps).* Options: a personae enrollment API plus a
djinn command that prompts on the terminal, never the environment (ruling
7), built in L2; the same plus a castellan enrollment card; Hello only
until later. Mark: **"personae API + djinn CLI (Recommended)"**.

Still open: a threat statement naming hibernation and the pagefile.

## 4. Phases

Drafted from the assessment; set once the forks are ruled.

- **L1 — personae can lock.** `IdentityVault::lock()` drops the profile and
  the storage key, and every accessor that reaches secret material returns
  `Locked`. Done when:
  - [x] each accessor has a test, and each test fails if its guard is
        removed;
  - [x] a no-residue instrument (a tracking allocator) finds no canary key
        live or freed uncleared after a lock, and fails on today's
        `profile_wire` clones as its positive control;
  - [x] after an unlock by the ruled method, slots are byte-identical and an
        Ed25519 signature verifies exactly as before, while a wrong
        credential leaves the vault locked. *(Passphrase measured; Windows
        Hello's prompt is Mark's attended step, its token path tested with
        a test-only constructor.)*
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
        *2026-10-05 correction:* the installed resident's PID changes
        on reboot (14756 that morning); the wall is its identity captured
        at each run's start (djinn test harness plan).

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

**2026-10-05, from L1's checkpoint A.**
- Zeroizing the structs alone left residue. `serde_json::to_vec` and
  serde's `Vec<u8>` visitor grow by reallocating, and each reallocation
  freed an uncleared copy (ruling 34 keeps the lane's fix).
- For L2: serde_json's scratch buffer for strings with escapes is also
  freed uncleared, which matters for castellan items holding secret strings.
- The instrument sees the heap only. Stack copies (moves of `[u8; 32]`, a
  documented `zeroize` limit) and DPAPI's `LocalAlloc` buffer are fixed by
  reading, not measured.
- argon2's working memory is freed uncleared in 0.5.3 and 0.6.0-rc.8
  (ruling 33).
- The persisted lock is advisory against same-user processes, which can
  read the DPAPI root file; this belongs in the threat statement.

## 6. Progress

**2026-10-05.** Assessed by a read-only lane (Opus); the load-bearing claims
re-checked in code. Nothing built. Next: Mark's rulings on §3.

**2026-10-05, L1 started.** The harness landed (`318b8f70`), so ruling 18's
order is met. An Opus lane builds L1 up to checkpoint A:
- the residue fixes of ruling 6;
- the tracking-allocator instrument, with its positive control failing on
  today's clones;
- the map of ruling 23 (every caller in mere and its siblings that ruling
  13's `Locked` return breaks);
- a proposed lock API.

It stops there; the API's choices come to Mark as forks before the
breaking change is built, Knot first.

**2026-10-05, L1 checkpoint A reached** (Opus lane, branch
`worktree-agent-a014d67042870a2b4`, base `24bfe7be`, not merged):
- `fea481a3` adds the no-residue instrument
  (`crates/dramatis/personae/tests/no_residue.rs`). It is a test-only
  tracking allocator with three canaries: the master seed, a slot payload,
  and a DPAPI root in a temp dir. It is red on purpose at that commit.
- `ffd3279b` adds the residue fixes:
  - `PlaintextProfile` and `PlaintextSlot` zeroize on drop, and
    `plaintext_to_slot` moves instead of cloning;
  - a crate-private `zeroizing_json`;
  - the passphrase storage's decrypted plaintext is `Zeroizing` at load,
    list and open;
  - the DPAPI buffer is cleared before `LocalFree`;
  - the agent's listing no longer copies the seed.
  No public API changed and the lock file is unchanged.
- **Verified in `mere-verify`:**
  - at `fea481a3` the instrument fails: the positive control finds what it
    should, then sealed storage shows 12 hits, passphrase storage 20 and
    the DPAPI root 1;
  - at `ffd3279b` all four scenarios are clean;
  - personae with all features passes 174 + 7 + 1, castellan 107 + 3 + 4
    + 1.
- **The caller map (ruling 23):**
  - mere: personae's agent, castellan's authority, Distillery's transport
    identity, djinn's resident (which opens the vault a second time for the
    Distillery lane), and graphshell's `GraphshellIdentity`.
  - Turnstone: `identity.rs`, with the fallback hazard of ruling 35.
  - hocket and woodshed reach the storage or the DPAPI root directly
    (ruling 36).
  - Knot has no production use of `IdentityVault`; its seed comes through
    pandect's wallet (ruling 15).
  - mer3ly, retinue, cleromancy and isometry are unaffected.
- The lane proposed the lock API; rulings 25 to 36 settle its forks. Next:
  L1's breaking change on the same branch, then L2.

**2026-10-06, L1 checkpoint B built** (same lane; `095c0423` adds the
personae lock API, `b7bcbdb0` adapts castellan and Distillery, not merged):
- The API follows rulings 25 to 30 and 33: `lock`, `unlock`, `is_locked`
  and `unlock_methods` on the vault and the storage trait;
  `IdentityError::Locked`; `PublicProfile` answering while locked; one
  shared key cell; `OsPresence` minted only by personae's Hello gate
  (feature `os-presence`).
- The lane also measured argon2's residue: three uncleared 19.9 MB blocks
  with argon2's own `hash_password_into`, none with ruling 33's fix.
- It found a stack copy of the passphrase KEK reaching the heap and fixed
  it by lending the key to the cipher rather than copying it out.
  *Reading, not ruled:* that is ruling 6's residue fix.
- No exhaustive match on `IdentityError` exists in mere or its siblings.
- Knot's pinned revision compiled against the branch: one error, identical
  at the base (pairing ruling 61's, fixed in Knot's repin).
- **Verified in `mere-verify`:**
  - personae 194 + 7 + 1, with the leak test clean in all six scenarios;
  - castellan 107 + 3 + 4 + 1, djinn 88, Distillery 24, graphshell 191;
  - the gate passed;
  - the lock file adds two feature edges and no package;
  - the installed resident was identical before and after.
- **Control A** (argon2's own call put back): the leak test failed with the
  three argon2 blocks.
- **Control B** (`authority.freshness.lock()` removed from
  `SealedRecordStorage::lock`): every test still passed. The freshness test
  checks that unlock wants the key back, not that lock dropped it, so
  ruling 1's freshness key could stay in memory under lock unnoticed.
  Today's code does drop it; the test is what is missing. Sent back to the
  lane.
- Next: the freshness test and ruling 37 on the same branch, then L1's
  merge.

**2026-10-06, L1 landed** (`2556a20c`, merging `fea481a3`, `ffd3279b`,
`095c0423`, `b7bcbdb0` and `f40a4d60`):
- **Checkpoint C, `f40a4d60`:**
  - a no-residue scenario for an authoritative record store (castellan's
    shape), with canary record and freshness keys;
  - a test that boxed and borrowed storages lock through their delegates;
  - ruling 37's cipher feature.
- **The lane's lock-body audit.** It removed every key-dropping line in
  every `lock()` (the record key, the freshness lock, the freshness
  ledger's key, the vault's profile and storage lock, both profile
  storages, the passphrase KEK, and the `&T` and `Box<T>` delegates). Each
  removal failed a test or a leak scenario.
- **Verified in `mere-verify`:**
  - my control B, rerun at `f40a4d60`, now fails: "live freshness key ...
    no-residue: FAILED";
  - personae 195 + 7 + 1 with seven leak scenarios clean, and castellan
    107 + 3 + 4 + 1;
  - the gate passed at `f40a4d60` and again on the merge with `main`
    `3a80b1fe`; the tree that landed is that verified merge.
- **The lock file** gains four feature edges (argon2, chacha20 and
  chacha20poly1305 zeroize; personae to windows) and no package.
- **Not verified here:**
  - a real Windows Hello prompt (Mark's step);
  - personae's all-features build on Linux (ring needs a cross compiler;
    chatelaine ruling 57 moves such checks to the ThinkPad);
  - stack copies, which the instrument cannot see.
- **For L2:**
  - passphrase enrolment (ruling 39);
  - lockable test storages (`InMemoryStorage` cannot lock);
  - serde_json's escape scratch buffer (§5);
  - a loader variant that takes the `OsPresence` proof, so the persisted
    lock (ruling 32, L3) does not block presence unlock;
  - ruling 38's constructor, which builds a vault without a decrypt (L3).

**2026-10-06, L2 started.** The dramatis repo plan's ruling D18 finishes this plan
first. An Opus lane builds L2 up to checkpoint A:
- the lock coordinator (rulings 13 and 31);
- castellan's typed refusals;
- the agent's OpenSSH semantics and `-x`/`-X`, with a harness receipt over an
  isolated pipe;
- passphrase enrolment (ruling 39), and lockable test storages.

The Secret Service is checkpoint B, on the ThinkPad.
