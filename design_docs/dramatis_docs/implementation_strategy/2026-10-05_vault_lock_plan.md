# Vault Lock Plan

**Date**: 2026-10-05
**Status (2026-10-08)**: rulings 1 to 95 in §3. L1 to L4 landed (L3
as `79fbbeb7`, its attended receipts as `303b5097`; L4 on 2026-10-08, §6);
deployment is Mark's step. Not yet carried out: ruling 44's transport-key hard switch,
so Distillery keeps the master keypair while locked, and pandect's wallets
until D8 (ruling 81). The [vault threat statement](../technical_architecture/2026-10-08_vault_threat_statement.md)
says what the lock defends and leaves open. The
[dramatis repo plan](2026-10-06_dramatis_repo_plan.md) moves this code
later. Chatelaine P4 (CXF import) waits on this plan (chatelaine rulings
64, 65).
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
- [persona wallet carry layer plan](../../archive_docs/2026-10-06_completed_plans/2026-06-25_persona_wallet_carry_layer_plan.md):
  the "one unlock ladder" rule (:358-362), and Meerkat's 2026-07-04 "Lock
  now" (:788-792).
- [vault threat statement](../technical_architecture/2026-10-08_vault_threat_statement.md):
  what the lock defends, what stays while locked, and what it does not
  defend (ruling 82).
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

Rulings 40 to 45 were asked on 2026-10-06 from L2's checkpoint A (§6).

**Ruling 40.** *While locked, djinn's own doors close: each app session's
signer is derived from the vault, so even the status and stop routes
cannot be reached.* Options: the doors' session-signing key is a
namespaced derived key the lock leaves in place, as ruling 24 does for the
transport key (it authenticates the resident to apps and opens nothing);
the status and stop routes admit sessions without it; accept it and
observe a locked resident through its event file. Mark: **"Door key stays
(Recommended)"**.

**Ruling 41.** *How is a running djinn unlocked? The unlock call exists,
but nothing in djinn reaches it.* Options: a native Windows Hello or
passphrase prompt in the resident (ruling 21), so the credential never
crosses a pipe; a `djinn --unlock` command that sends the passphrase over
the owner-only control route; both. Mark: **"Both"**. Follows: both are
built. The command's passphrase crosses a local pipe, so it should sit
only on the owner-only control route. *Reading, not ruled.*

**Ruling 42.** *Linux has no OS-held root. Its vault is the passphrase
vault, which can already lock and unlock by passphrase, but the resident
takes that passphrase from `PERSONAE_PASSPHRASE`, which ruling 7 forbids
for a resident that locks.* Options: the Linux resident starts locked and
waits for the same native prompt (a terminal prompt when headless); an
AutoOs backend for Linux (the kernel keyring, or systemd-creds bound to
the TPM; the desktop keyring is awkward because castellan is the Secret
Service there); the environment for now, as an exception. Mark: **"Starts
locked, prompt (Recommended)"**.

**Ruling 43.** *Ruling 15 (the lock also relocks Knot's seed) can be done
now without pandect changes.* Options: djinn closes the Knot lane on lock
and reopens it on unlock, with the seed's residue measured; wait for the
dramatis repo plan's D8. Mark: **"Close/reopen now (Recommended)"**.
Follows: Knot's sync pauses while locked.

**Ruling 44** *(amends how rulings 2 and 24 are carried out).* *Deriving
Distillery's transport key changes the device's network id, and peers
find a device's address by its master key.* Options: a separate plan
where peers accept both identities for a window; one coordinated switch
across Mark's devices; reopen ruling 2 and keep the master resident. Mark:
**"Hard switch"**. Follows: no window; every device switches in one
update. Peers outside Mark's devices learn the new transport identity
afresh. Until it lands, Distillery keeps the master in memory while
locked.

**Ruling 45.** *The lane's smaller calls.* Each was offered for unticking:
- the standalone agent refuses `-x`;
- a second `-x` fails, as in OpenSSH;
- enrolment never mints or replaces a root;
- while locked, only the vault card acts (pending approval cards lose
  their buttons; revocation and the wallet sealer are refused).

Mark kept all four: **"Standalone agent refuses -x, Second -x fails,
Enrolment never mints, Locked: only Unlock acts"**.

Rulings 46 to 48 were asked on 2026-10-06 from L2 checkpoint B's forks.
The lane stopped before building rulings 40, 41a and 43.

**Ruling 46** *(how ruling 40 is carried out).* *An admitted door session
needs two vault-derived keys. The delegation signer is already namespaced
to the door (`mere.graphshell` plus the local network id). notochord's
session signer is derived under one global salt that every remote hello
uses (`notochord/src/handshake.rs:41`, accepted alone at `:253`), so
keeping it live while locked keeps the persona's network login live.*
Options:
- notochord also accepts a session signer derived under a salt bound to
  the local network, used only by the local door, while remote hellos
  keep the global salt, which locks (additive; old peers and Knot
  unaffected; the subject stays the persona);
- keep the global signer;
- the door speaks as its own derived identity, which changes every app's
  subject.

Mark: **"Network-bound signer (Recommended)"**. Follows: the door holds a
restricted provider. It answers the master public key and derives or
attests only the door's two salts; every other salt returns Locked.

**Ruling 47** *(how ruling 41's native half is triggered).* Options:
- the Locked card's native-only Unlock reaches the resident's own UI, as
  SSH import does: Windows Hello first, then a passphrase box, the
  credential never leaving the resident (a browser wire enum gains one
  variant);
- `djinn --unlock --native` asks the resident to show its prompt;
- both.

Mark: **"Both"**.

**Ruling 48** *(when ruling 43's Knot lane closes).* Options: djinn's run
loop closes it when the watch reports Locked (a short logged window after
`lock()` returns; the route switches to Locked refusals and live Knot
sessions are cut; it reopens on unlock); a synchronous holder that blocks
on the close (no window, but the agent's lock call waits on Knot's network
shutdown, and it needs a multi-thread runtime). Mark: **"Close right after
lock (Recommended)"**.

**Ruling 49.** *When djinn closes the Knot lane on lock, no live copy of
Knot's signing seed remains, but 19 to 20 copies are freed uncleared,
because knot-editor's own functions (`author`, `KnotSyncHost::open`) take
the seed by value into async code. knot-editor is Mark's repo, so this is
not an upstream ledger item.* Options:
- fix it in knot-editor (the seed borrowed or zeroizing, landing in Knot
  first under the lockstep, with djinn's residue test made strict, and any
  copies inside p2panda to the upstream ledger);
- fix it with the dramatis plan's D8;
- record only.

Mark: **"Fix in knot-editor (Recommended)"**.

**Ruling 50** *(the base for ruling 49's Knot fix).* *Twenty-two
knot-editor functions take the seed by value. The burn coordinator's Knot
lane is mid-flight on knot-editor (`mere-p1-adapt`, `7bea433`), and burn
plan 13.46 holds djinn's next Knot pin move for that head.* Options:
- branch from the Knot lane's P1 head once it lands, so the seed fix is
  the next Knot commit and djinn's repin after theirs carries it;
- branch from origin/main `54bb8cd` now and have the Knot lane merge it;
- hand the change list to their lane.

Mark: **"After their head (Recommended)"**.

**Ruling 51** *(mere's transport).* *mere's own transport
(`crates/murm/transport`: `builder_from_seed`, `bind_seed`) also takes the
seed by value, and Knot calls it, so Knot's fix takes effect only once the
transport is fixed. iroh alone leaves 2 freed blocks per bind and close;
p2panda-net's share cannot be measured until the transport is fixed.*
Options: fix the transport now (mere work, an additive borrowing API beside
the old one), re-measure, and ledger whatever iroh and p2panda-net still
leave; fix it with the Knot fix later. Mark: **"Fix transport first
(Recommended)"**.

Rulings 52 to 54 were asked on 2026-10-06 from ruling 51's transport fix.

**Ruling 52.** *After the transport fix, the transport leaves exactly what
p2panda-net and iroh leave alone, except an intermittent 392-byte block.
On close the transport asks p2panda's endpoint actor for the iroh
endpoint, and the boxed message carries stale stack bytes from bind.*
Options: keep the iroh endpoint handle from bind so close sends no actor
message (and ledger the cause); ledger only. Mark: **"Keep the handle
(Recommended)"**.

**Ruling 53.** *The g5, h6 and h7 receipt binaries still pass their test
seeds (hashed from environment variables, never the vault) by value into
`InMemoryProvider`.* Options: leave them, noted; fix them too. Mark:
**"Leave them (Recommended)"**.

**Ruling 54.** *With ruling 52 built, the 392 and 1824 blocks are gone
from transport runs (0 of 30; p2panda-net alone shows them in 6 to 23 of
30). Four other transport calls (`endpoint_addr`, `peers`, `peer_ticket`,
`peer_paths`) still ask the actor on every call, and the residue test
does not cover them.* Options: move all four onto the kept handle (`peers`
loses its "could not ask" branch); close only. Mark: **"Move all four
(Recommended)"**.

**Ruling 55** *(how Knot reaches the transport for ruling 49).* *Knot's
head `ef89a18` has the P1 work and is djinn's pin, but Knot pins mere at
`e0cea3e0`, before the transport's borrowing entry points, and Knot's
repins have been the genet chain's.* Options: no Knot repin (Knot lends
the seed through its own functions and passes it by value only at the one
synchronous call into the transport, which boxes and clears it at once);
repin Knot's mere rows to `83b06806` or later and call the borrowing entry
directly, telling the genet chain. Mark: **"Repin Knot's mere rows"**.
Follows: the genet-chain session and the burn coordinator were told on
2026-10-06, before any edit.

**Ruling 56** *(the order under Mark's three-tasks rule).* Options:
- the Knot fix, then ruling 42 (no hardware), then the Secret Service on
  the ThinkPad together with 42's Linux proof, then L3;
- the Knot fix, then L3 (finishing Windows first);
- pause after the Knot fix.

Mark: **"Knot, then ruling 42 (Recommended)"**.

**Ruling 57** *(amends ruling 55's means; asked 2026-10-07).* *The repin
`f68af0d` (every mere row to `ea74604b`, with S77's preview change) was
never pushed. Meanwhile Knot's `0096591` repinned all 35 mere rows to
`57b4893d`, 34 commits past `ea74604b`, moved genet to `965b64e`, and
carries its own S77 fallback; Knot's main is `ae3352e`. The uncommitted
seed fix (21 files) shares one file with what origin changed.* Options:
drop `f68af0d` and move the seed fix onto `ae3352e`, verified by the strict
residue test before any push; rebase `f68af0d` anyway (moving the pins
back); pause Knot. Mark: **"Drop f68af0d, rebase fix (Recommended)"**.
Follows: ruling 55's precondition (Knot's mere rows at or past `83b06806`)
is met by `0096591`.

**Ruling 58** *(mere's Knot pin).* *mere's workspace and djinn pin Knot
`ef89a186`.* Options: one repin, to the Knot commit carrying the seed fix;
repin to `ae3352e` now and again later. Mark: **"Once, after seed fix
(Recommended)"**.

**Ruling 59** *(the sync host's verdict).* *On `ae3352e` the strict paths
fail (`author` leaves 4256) and with the fix pass. The sync host, judged
against iroh alone and p2panda-net alone, still shows 7 sizes with the fix
(8 without). Six are identical in both arms, and Knot now lends the seed
only into the transport builder. So those copies come from below Knot,
when the store joins gossip and sync, which neither baseline does.* Options:
- a no-Knot baseline that binds the overlay host and joins as Knot's store
  does, the fix committed only if the sync host passes against it and the
  control still fails;
- commit now, with the sync host reporting rather than failing;
- name each block from allocation backtraces first.

Mark: **"Baseline the join (Recommended)"**.

**Ruling 60** *(mDNS residue).* *`P2pandaHostPolicy::default()` turns mDNS
on (Active). The overlay host alone leaves 4 seed copies beyond the
baselines with it on (472, 568, 784, 2424) and none with it off, in every
app that binds through the overlay host. mere-transport's
`seed_residue.rs` binds without the policy, so it never sees them.*
Options:
- a vault lock item, with `seed_residue.rs` gaining a default-policy run
  and the copies attributed (iroh, p2panda-net, or Mere's wiring);
- ledger only;
- defer to L3.

Mark: **"Vault lock item + test (Recommended)"**.

**Ruling 61** *(p2panda-net's mDNS copies; asked 2026-10-08).* *Ruling 60's
measurement puts the 4 blocks in p2panda-net's mDNS layer: our
`mere-p2panda-net` 0.7.5 fork, with mDNS on by default. iroh's own lookup
leaves none, and Mere's wiring adds none.* Options:
- ledger only, as item 11's other p2panda-net blocks are;
- mDNS off in the default host policy;
- trace them, then carry a minimal patch in the fork.

Mark: **"Ledger only (Recommended)"**. Follows: ruling 42 is next (ruling
56's order).

Rulings 62 to 65 were asked on 2026-10-08 from ruling 42's assessment (§6).

**Ruling 62** *(ruling 42's scope).* *Ruling 42 names Linux. Ruling 7
forbids the environment passphrase for any resident that locks, and the
Windows test harness uses it.* Options:
- every resident on the passphrase vault starts locked: Linux always,
  Windows when it uses that vault, while DPAPI Windows still auto-unlocks;
- Linux only, keeping the Windows environment path as a recorded exception.

Mark: **"Every passphrase vault (Recommended)"**. Follows: djinn no longer
reads `PERSONAE_PASSPHRASE`, and the path is tested here on the passphrase
vault.

**Ruling 63** *(the first unlock).* *A resident admits door sessions only
with the door keys, which come from the vault (rulings 40 and 46). Before
its first unlock it has none, so `djinn --unlock` cannot reach it.*
Options:
- the resident's own prompt at start (the native box where the desktop
  has a dialog provider, else its terminal), with no doors until it
  succeeds, and a cancel exiting;
- the same plus an owner-only pre-door unlock socket;
- the resident's own prompt, with a cancel re-showing it (with backoff)
  instead of exiting.

Mark: **"Prompt, and re-prompt on cancel"**. *Reading, not ruled:*
- a cancel or a wrong passphrase re-prompts;
- with no dialog provider and no terminal there is nothing to re-show, so
  the resident exits with an error naming both.

**Ruling 64** *(first run).* *No vault exists, and nothing creates a
passphrase vault without the environment (`--enroll-passphrase` enrols
only over the sealed DPAPI vault).* Options: a `djinn --create-vault`
terminal command; the start prompt creates it. Mark: **"The start prompt
creates it"**. *Reading, not ruled:* it asks twice, and a mismatch
re-prompts.

**Ruling 65** *(the harness).* *With the environment gone, the harness
cannot type into `rpassword`, which reads only the console or tty.*
Options:
- `--passphrase-fd N` (GnuPG's convention), compiled only under a test or
  receipt feature;
- the same in release builds;
- the environment kept, harness only.

Mark: **"Passphrase over an fd (Recommended)"**.

**Ruling 66** *(what "starts locked" builds; asked 2026-10-08).* *A vault
object opened locked would have no master public key, but
`IdentityProvider::master_public_key()` is infallible, with about 200
calls. Under ruling 63 nothing reaches the vault before the first unlock.*
Options:
- wait before the vault: the resident prompts before building any
  storage key, profile, door or lane, then opens the passphrase vault as
  today; `open` already rejects a wrong passphrase and creates a missing
  vault;
- a locked vault object, with personae's `open_locked` constructors and a
  fallible master key.

Mark: **"Wait before the vault (Recommended)"**. Follows: personae is
unchanged, and the first done-condition of ruling 42's build is
superseded (§6, 2026-10-08). *Reading, not ruled:* without the
environment, djinn picks the passphrase vault when the platform has no OS
root, when `--passphrase-fd` is given, or when the vault directory holds
`vault.json`; otherwise DPAPI. The installed resident's directory holds
only `auto-unlock-root.json` and `profiles`, so it stays on DPAPI.

Rulings 67 to 69 were asked on 2026-10-08 from the Secret Service's
assessment (§6).

**Ruling 67** *(the Secret Service's Prompt).* *While the vault is locked,
a client's `Unlock` gets a Prompt object (ruling 10).* Options: the
Prompt shows the resident's native unlock (ruling 47's prompt), handed in
by the resident, with a cancel completing as dismissed; the Prompt shows
nothing and completes when the vault is unlocked by any route. Mark:
**"Show the native unlock (Recommended)"**.

**Ruling 68** *(a client's `Lock`).* Options:
- any `Lock` engages the whole vault lock, as `ssh-add -x` does (ruling 9);
- per-object flips under the vault lock;
- refused.

Mark: **"Locks the whole vault (Recommended)"**.

**Ruling 69** *(who serves it).* *gnome-keyring already owns
`org.freedesktop.secrets` on the ThinkPad.* Options: test-served for this
checkpoint, proven under a disposable bus, with djinn's wiring and the
hand-over from gnome-keyring as later items; djinn serves it now behind an
owner setting that is off by default. Mark: **"Test-served for now
(Recommended)"**.

**Ruling 70** *(the Linux residue; asked 2026-10-08).* *On Linux, personae's
`no_residue` finds the sealed root key in a 564-byte block freed uncleared
during `save` (3 of 3 runs; clean on Windows at the same commit).* Options:
trace it now, with ruling 42's Linux proof waiting; ruling 42's proof
first; record it and go on to L3. Mark: **"Trace it now (Recommended)"**.

Rulings 71 and 72 were asked on 2026-10-08 from ruling 70's trace (§6).

**Ruling 71** *(the slot table).* *On lock the vault frees its profile's
slot `HashMap` uncleared. Values moved into it carry stale stack bytes in
their padding, the master seed among them: Linux, 3 of 4 runs, depending
on layout.* Options:
- the slots move to `Vec` storage that `zeroize` wipes whole, padding and
  spare capacity included, on drop and on growth;
- clear only at lock;
- ledger it.

Mark: **"Zeroizing Vec storage (Recommended)"**.

**Ruling 72** *(the test fixture).* *The original Linux hit was
`no_residue`'s own canary profile, built inside the measured window. Its
`HashMap` caught half the root key from the test's stack copy.* Options:
build the fixtures unarmed and drop them after disarming, as the
passphrase scenario does; keep them armed. Mark: **"Build fixtures unarmed
(Recommended)"**.

Rulings 73 to 78 were asked on 2026-10-08 from L3's assessment (§6).

**Ruling 73** *(the Linux idle source).* *On the ThinkPad, GNOME's
`org.gnome.Mutter.IdleMonitor` read the idle time exactly (237 s). logind's
`IdleHint` is never set there, because GNOME's `idle-delay` is 0, so a
logind-only reading would say "active" forever.* Options:
- Mutter's monitor, else logind's `IdleSinceHint`, else unknown;
- logind only;
- Mutter only.

Mark: **"Mutter, then logind (Recommended)"**.

**Ruling 74** *(what "idle fails closed" does, ruling 3).* Options:
- unknown counts as idle, so the vault locks once a full window passes
  with no reading of activity;
- lock at the first unreadable sample;
- idle locking reported unavailable on a device with no source.

Mark: **"Unknown counts as idle (Recommended)"**.

**Ruling 75** *(suspend).* Options: lock before sleep (Windows' suspend
notification; on Linux a logind delay inhibitor); lock on resume. Mark:
**"Before sleep (Recommended)"**.

**Ruling 76** *(amends ruling 38: a resident restarted under the persisted
lock).* *Before its first unlock a resident admits no door session (the
door keys come from the vault), so ruling 38's Locked card cannot be
served.* Options:
- it waits before the vault at its native prompt (Hello, then the
  passphrase box), as ruling 66 does, serving nothing until unlocked;
- the door keys sealed under DPAPI apart from the root;
- it exits and the launcher stops restarting it.

Mark: **"Wait at its prompt (Recommended)"**.

**Ruling 77** *(where the per-device lock settings live).* Options:
- a person-edited `lock.toml` in djinn's app directory, per device,
  read at start and watched, with an absent or malformed file meaning the
  defaults (all on, 15 minutes), never "no locking";
- a section of each profile's owner settings JSON;
- the TOML file plus a castellan card intent.

Mark: **"Device lock.toml (Recommended)"**.

**Ruling 78** *(the real receipts).* Options: build every trigger with
injected signals and clock and prove it in tests, then one attended
session for `Win+L`, `loginctl lock-session` and suspend on both machines;
run the ThinkPad's lock-session receipt unattended while building. Mark:
**"Build first, one attended run (Recommended)"**.

*2026-10-08 annotation to ruling 78:* Mark: **"Do the lock and sleep after
this crop of runs. Don't interrupt anything, please."** The attended
receipts wait for his word that the other sessions' runs are done.

**Ruling 79** *(where the persisted lock's marker lives).* *The loaders see
only a root file's path. The vault's root, pandect's wallet roots (Knot's
and Retinue's seeds) and signalman's station roots all pass through
them.* Options:
- one marker per user, beside the default vault, which every identity
  AutoOs root obeys;
- a marker beside each root;
- one per vault directory, with pandect's loaders told the vault
  directory.

Mark: **"One per user (Recommended)"**.

**Ruling 80** *(amends ruling 79; asked the same day).* Mark: **"Would one
per vault be such a big refactor?"**
- **Measured:**
  - for the vault, nothing: its root sits in the vault directory, so a
    marker beside it is free;
  - pandect's wallets have their own roots under an app's data root and
    know no vault. Ten public wallet functions take only `data_root`, and
    `load_identity_seed` has four callers in mere besides Knot's.
- **Options:**
  - per vault by a middle path: the marker beside the vault's root, and
    each wallet checking the marker of the vault its own settings name
    (the default vault when none is named), with no signature changes;
  - per user, as ruled;
  - per vault, threading the directory through pandect's signatures
    across mere, Knot and Retinue.

Mark: **"Per vault, middle path (Recommended)"**. Follows: signalman's
station roots are no vault's and pass untouched.

**Ruling 81** *(amends ruling 80's pandect half).* Mark: **"So wait, is
pandect gonna need vault awareness?"**, then **"Or d8 will handle it?"**
Options:
- leave pandect vault-unaware and record the wallets' unattended reopen
  after a restart as a gap that D8 (the wallet's secrets into castellan,
  in DR-B) closes;
- the small check now, removed again when D8 lands.

Mark: **"Leave it to D8 (Recommended)"**.

Rulings 82 and 83 were asked on 2026-10-08, at L4's start.

**Ruling 82.** *Where should the vault's threat statement live? It would
state what the lock defends, what it leaves open (the pagefile and
hibernation copies made while unlocked, crash dumps, same-user processes,
in-process mods), and what is fixed only by reading. The protocol plan's
§3.7 promises "a future doc under technical_architecture".* Options:
- its own doc in dramatis' `technical_architecture/`, linked from the tier
  architecture and §3.7, moving with dramatis at the split;
- a section in the tier architecture;
- a section in this plan;
- left open past L4.

Mark: **"Own doc in dramatis (Recommended)"**. Follows:
[the vault threat statement](../technical_architecture/2026-10-08_vault_threat_statement.md).

**Ruling 83.** *Should the dramatis tier architecture gain an invariant for
the lock? Its §4 lists 12, each with where it is enforced, and breaking
one comes to Mark first.* Options:
- add invariant 13: while locked no secret is reachable, and unlocking
  takes a user act on the resident's own surface, with its four
  enforcement points;
- annotate invariant 1 only;
- neither.

Mark: **"Add invariant 13 (Recommended)"**.

Rulings 84 to 87 were asked on 2026-10-08 from L4's follow-up of the
failures and open items left beside this plan, scoped read-only first.
Each fix lands in code another plan owns; this record holds the rulings,
and the owners' plans get dated pointers.

**Ruling 84** *(the reservoir plan's V5 code).* *`embedded_reservoir_two_process`
and `embedded_reservoir_validation` fail on Windows with error 231 ("All
pipe instances are busy"). The listener keeps one waiting pipe instance
and makes the next only after a connect (`local_endpoint.rs:77-84`), and
`connect_local` (`:36-41`) opens once with no retry. `EmbeddedOwner::start`
probes with a connect and then connects for real at once, inside that
window. It is a product race any two close clients can hit, and probably
never green on Windows.* Options:
- retry in `connect_local`, briefly and bounded, as tokio's docs advise;
- the retry plus spare waiting instances;
- spare instances only;
- retry in the reservoir only.

Mark: **"Retry in connect_local (Recommended)"**.

**Ruling 85** *(the djinn test harness plan's code).* *Once in a long run,
`djinn --stop-resident` exited with failure though the resident stopped.
The stop intent raises the stop before its reply is written
(`resident_status.rs:470-472`), so the shutdown can cancel the reply or
exit before the CLI reads it. The testkit keeps no record of the stop
command's output.* Options:
- the resident raises the stop only once the reply is flushed, and the
  testkit records the stop command's output;
- the stop after the reply only;
- the testkit judges "left" by the `stopping` event and the exit;
- the CLI treats a dropped connection after the stop as accepted.

Mark: **"Stop after reply + record (Recommended)"**.

**Ruling 86** *(personae's tests; no live plan).* *`authoritative_opening_is_exclusive_until_every_clone_drops`
flakes on Linux: the last claim finds "authority is already held". The lock
is `flock`, which a forked child holds until it execs, and the sibling
test `..._across_processes` spawns a child (inferred, not traced).* Options:
- serialize the two tests;
- retry the final claim;
- record only.

Mark: **"Serialize the two tests (Recommended)"**.

**Ruling 87** *(amends ruling 69's "later items").* *Serving the Secret
Service for real on Linux is eight work items, with open questions: how
the name is taken from gnome-keyring, what serves before the first unlock,
gnome-keyring's existing items, and the Flatpak portal backend. The Linux
`personae-agent` keeps its passphrase in gnome-keyring today.* Options:
- after pairing D2, with the questions asked then;
- now, behind an owner setting that is off by default;
- scoped as its own plan.

Mark: **"Scope it as its own plan"**.

**Ruling 88** *(how ruling 85's product half is built).* *The reply is
written by graphshell's generic app-door loop (`app_broker.rs:351-366`),
after the session's server task has run the intent, so nothing fires
"after the reply". djinn makes a control endpoint per session
(`resident_status.rs:419`), and the session ends once the reply is written
and the asking CLI closes.* Options:
- the stop intent marks the session, and the stop is raised when it ends
  or after 2 s, whichever is first (djinn only);
- at session end only;
- a generic post-reply hook in graphshell's broker.

Mark: **"At session end + 2 s fallback (Recommended)"**.

**Ruling 89** *(carries ruling 7 into the libraries).* *`InstalledAuthority::open`
(distillery) and `GraphshellIdentity::load` / `load_selected` (graphshell)
read `PERSONAE_PASSPHRASE` themselves. Their only callers are the
`distillery-installed` CLI and one graphshell test, and no sibling repo
calls them. A smoke script also sets the variable for a binary that never
opens the vault.* Options:
- remove the env-reading entry points now; the CLI reads the environment
  itself and passes an explicit `Unlock`, and the dead script line goes;
- leave them until D5 and D2;
- a debug assertion only.

Mark: **"Remove them now (Recommended)"**.

**Ruling 90** *(amends ruling 6's reach to the stack; asked from L4's gate).*
*castellan's residue test failed on Fedora under djinn's unified features,
3 of 3: a live 56-byte block held the persona's master seed after the
lock. The block is castellan's `AgentListenerView` (`authority.rs:445`).
Its `StandaloneRetained` variant leaves a 24-byte payload uninitialized,
and that payload was copied from a stack slot holding a stale seed copy.
`Profile` holds the key inline, so every by-value move of `IdentityVault`
(`with_profile`, `Mutex::new`) leaves a bitwise copy in dead stack: 25 to
29 at that point in every build, measured with gdb watchpoints. djinn's
features only change codegen, which decided whether a copy landed under the
payload.* Options:
- box the key (`Ed25519Keypair` holds `Box<SigningKey>`; a private
  field, so no API change), so moves copy a pointer; re-measure, keep the
  test strict, and name stack residue and its path into the heap in the
  threat statement;
- box the profile in the vault instead, plus the same record;
- record only.

Mark: **"Box the key + record (Recommended)"**.

**Ruling 91** *(amends ruling 42; asked 2026-10-09 beside the Secret
Service plan's SS8).* *Ruling 42 started Linux locked, partly because the
desktop keyring was awkward while castellan meant to be the Secret
Service, which SS5 then SS8 removed. gnome-keyring is unlocked by the login
password through PAM and stays unlocked all session.* Options: keep ruling
42 (Linux asks once per login); keep djinn's root in the OS keyring, so it
auto-unlocks at login like Windows; ask again at D2. Mark: **"Root in the
OS keyring"**. Follows: Linux gets an auto-unlock root held in the
desktop's Secret Service. It is built after its own assessment, which
covers the client library, the persisted lock and the threat statement's
Linux at-rest line.

Rulings 92 to 95 were asked on 2026-10-09 from ruling 44's assessment
(§6), before its build (dramatis repo plan D35).

**Ruling 92** *(which key is Distillery's transport identity).* Options:
reuse the mesh author key (`MESH_AUTHOR_SALT`), so the author becomes the
address as personal sync and Knot already do, and the directory's
author-to-master indirection goes away; a per-mesh transport salt, with a
new master-signed attestation; one fixed transport salt, with a new
attestation. Mark: **"Reuse the mesh author key (Recommended)"**.

**Ruling 93** *(the landing's scope).* *Only tests exercise the
address-by-master code; production never reaches a peer.* Options: the
derivation plus the directory, courier and remote checks now, with a
negative control that a master-bound transport is refused; the derivation
only. Mark: **"Derivation and lookups now (Recommended)"**.

**Ruling 94** *(rebinding while locked).* Options: no rebind while
locked, since the lane drops its derived key after binding; the lane holds
the derived key for its whole life to rebind. Mark: **"No rebind while
locked (Recommended)"**.

**Ruling 95** *(the master-taking `P2pandaTransport::builder`).* Options:
retire it from production, which binds only from a derived key, keeping it
for tests and probes with a check against production use; keep it. Mark:
**"Retire it from production (Recommended)"**.

Still open: a threat statement naming hibernation and the pagefile.
*2026-10-08:* closed by ruling 82.

## 4. Phases

Drafted from the assessment; set on 2026-10-05 once rulings 1 to 24 were
made, and carried out since under the later rulings.

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
  - [x] over the isolated named pipe, the agent behaves as ruled while
        locked, and `ssh-add -x`/`-X` as ruled;
  - [x] `CastellanResident` drops its keys, so items and the OTP gate return
        `Locked`;
  - [x] the snapshot reports Locked, and Unlock is native-only;
  - [x] Secret Service collections report Locked, `GetSecret(s)` refuses,
        and `Unlock` returns a Prompt, proven on the ThinkPad with
        `secret-tool` under a disposable bus. *(2026-10-08, `aac67a85`;
        §6.)*
- **L3 — triggers.** Done when:
  - [x] each ruled trigger is proven, idle with an injected clock;
  - [x] there are real receipts for Windows `Win+L` and suspend, and for
        Fedora `loginctl lock-session` and suspend;
  - [x] an unknown idle fails closed;
  - [x] a locked resident restarted by the launcher comes back as ruled.
        *(2026-10-08, `79fbbeb7`; §6, L3 checkpoint E.)*
- **L4 — docs and gates.** Done when:
  - [x] `UnlockTier`'s docs, the protocol plan's §3.6 and §3.7, and the
        tier invariants match the rulings;
  - [x] the gates pass, with Windows-only and Linux-only code each compiled
        on its own target;
  - [x] PID 53336 is untouched; deployment is Mark's step.
        *(2026-10-08; §6, L4.)*
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

**2026-10-06, L2 checkpoint A landed** (`7c588deb`, merging `b173f766` and
`fcd82877`):
- **What it built:**
  - the resident lock coordinator (rulings 13 and 31): the vault first,
    then every holder, then the watch channel; a holder that fails to
    re-derive relocks everything;
  - `CastellanResident`'s lock holder;
  - castellan's typed refusals (ruling 10): items, the OTP gate,
    revocation, the wallet sealer, and switching (ruling 14);
  - the Locked card, with only a native Unlock, and the kept snapshot
    (ruling 11);
  - the agent's OpenSSH semantics: `ssh-add -x` locks the vault, and `-X`
    is refused (rulings 8 and 9);
  - passphrase enrolment through personae and `djinn --enroll-passphrase`
    (ruling 39);
  - a castellan no-residue scenario, which shares the tracker with
    personae's.
- **The lane's guard removals:** the holder hook, the agent's sign guard,
  the `-X` refusal, revocation's guard and the snapshot's locked branch.
  Each failed a test.
- **The lane's receipt:** `20261006T091745Z-7115a66e` used the system's
  OpenSSH (`ssh-add`, `ssh-keygen -Y sign`) over an isolated pipe. All 15
  assertions passed, the record verified, and the installed resident was
  identical.
- **Verified in `mere-verify` at `fcd82877`:**
  - my two controls each failed a named test: no relock when a holder
    fails (`a_holder_that_cannot_rederive_relocks_everything`), and a
    holder joining a locked vault keeping its keys
    (`a_holder_joining_a_locked_vault_locks`);
  - personae 205 + 7 + 1, castellan 120 + 3 + 4 + 1, djinn 90 plus its
    integration tests, testkit 6 + 5;
  - the receipt again (`20261006T095312Z-774abc32`, passed, 4 evidence
    files verified), and the harness's live tests;
  - the gate, on the merge with `main` `e8115a16`;
  - the installed resident identical.
- **The lock** gains djinn's edge to `rpassword`, already present. Knot's
  pinned revision shows the same single `knot-desktop` failure as the base
  (two genet copies), so the branch adds nothing.
- **Not verified:**
  - the wire sign refusal against the real binary (OpenSSH never sends a
    sign request when nothing is listed; proven in castellan's in-process
    pipe test);
  - a real Hello prompt;
  - personae and castellan on Linux with all features;
  - serde_json's escape buffer (§5);
  - ruling 7 (the receipt's resident still opens with the environment
    passphrase, for ruling 42 to remove).
- Rulings 40 to 45 settle its forks.

**2026-10-06, L2 checkpoint B built** (branch `l2b`: `4f81b3c4`,
`b7843783`, `8553bba5`, `a599c49d`; not merged):
- **What it built:**
  - `djinn --unlock` over the control route (41b);
  - the door's two retained keys through a restricted provider, with
    notochord accepting a signer bound to the network (46);
  - native unlock by the card's `UnlockVault` and by `djinn --unlock
    --native`, Hello first and then a passphrase box (47);
  - the Knot lane closed by a gate when the watch reports Locked, and
    reopened on unlock (48).
- **The lane's receipt** (`20261006T175358Z-c0d7a28c`, 23 assertions):
  lock over the wire, the status route reads Locked (H4's first condition),
  `-X` refused, a wrong `--unlock` stays locked, the right one lists the
  same identities and a signature checks, `-x` relocks, and a graceful stop
  while locked exits 0.
- **Verified in `mere-verify` at `a599c49d`:**
  - personae 207 + 7 + 1, castellan 120 + 3 + 4 + 1, notochord 17 + 16 +
    13, graphshell's 191 library tests (which do not compile in the lane's
    worktree), djinn with every integration test (five door tests
    included), and testkit;
  - the receipt again (`20261006T182410Z-e247d160`, verified), the
    harness's live tests and the gate;
  - the installed resident identical;
  - my control 1 (a closed Knot gate still accepting opens) failed
    `route_reopens_over_joined_sync…`;
  - my control 2 (the native unlock intent accepting a payload) failed
    nothing. That behaviour has no test, so it went back to the lane.
- **Finding:** the Knot seed's freed copies (ruling 49).
- **Finding:** a resident whose door was never used before it locked has
  no door keys until its first unlock. That is the shape ruling 42's
  resident, which starts locked, will have.

**2026-10-06, L2 checkpoint B landed** (`ec1768ab`, merging `l2b` through
`7555a36d`):
- `7555a36d` adds `the_native_unlock_refuses_any_payload`.
- **My control 2, rerun:** the native unlock intent accepting a payload
  now fails that test.
- **Verified:** djinn's library tests (91), and the gate on the merge with
  `main` `9b53f744`. The tree that landed is that merge.
- **The seed residue, measured by the lane** (freed uncleared blocks, no
  live copies):
  - Knot's `author`: 1;
  - an iroh endpoint bind and close alone: 2;
  - mere's `P2pandaTransport` bind: 7, or 11 with gossip;
  - `KnotSyncHost::open` and close: 19;
  - p2panda's `SigningKey` and iroh's `SecretKey` clear themselves.
- **Reading, not measured:** a `[u8; 32]` taken by value into an async
  function leaves its copy in the future, and `Zeroizing` alone does not
  stop moves copying. The fix is borrowing, or one boxed `Zeroizing`,
  across every await.
- **Next under rulings 50 and 51:**
  - the transport first, in mere;
  - then Knot's 22 functions on the Knot lane's P1 head;
  - djinn's strict residue test lands with the repin, judged against
    iroh's own baseline in the same process.

**2026-10-06, ruling 51's transport fix built** (branch `transport-seed`:
`ed741806`, then `01b4f632` for ruling 52; not merged):
- **The change:**
  - the builder holds the seed as one `Box<Zeroizing<[u8; 32]>>`;
  - new borrowing entry points (`builder_from_seed_ref`, `bind_seed_ref`);
  - the by-value ones are kept for Knot and clear their own copy;
  - p2panda's handles and spawns are boxed with no await between;
  - mere's callers moved to the borrowing path.
- **Measured** (freed uncleared blocks holding the seed, one process): the
  transport's own block (6080 bytes, its task frame) is gone. After the
  fix the transport leaves exactly p2panda-net's own set. Ruling 52
  removed the intermittent 392 and 1824 blocks (0 of 30 runs).
- **`mere-transport/tests/seed_residue.rs`** fails on any block size found
  in every transport run and in no baseline run. The baselines are iroh
  alone and p2panda-net alone, nested exactly as the transport is.
- **Verified in `mere-verify` at `ed741806`:**
  - my control (the held seed without `Zeroizing`) failed with "the
    transport's own blocks {(false, 32)}";
  - the residue test passed 3 of 3;
  - the transport tests and the gate passed;
  - graphshell's library tests passed 191 on a rerun. One carrier test
    timed out once under load, then passed 3 of 3 on the branch and 3 of
    3 on `main` (a load flake, not a regression).
- **What iroh and p2panda-net leave** is in the upstream candidates ledger
  (items 10 and 11).
- **Next:** ruling 54 on the same branch, merged onto `main` `ebfb490a`,
  which moved djinn's Knot pin to `ef89a18`. Then verification and the
  merge.

**2026-10-06, a tail inherited from the S14 archive pass** (recorded in the
[archived plan tails plan](../../mere_docs/implementation_strategy/2026-07-03_archived_plan_tails_plan.md), "2026-10-06 archive pass"): lock and
unlock follow-through, and non-Windows startup unlock backends, from the
[persona wallet carry layer plan](../../archive_docs/2026-10-06_completed_plans/2026-06-25_persona_wallet_carry_layer_plan.md).
- Lock and unlock follow-through is this plan.
- Non-Windows startup is already ruled:
  - Linux starts locked and waits for the native or terminal prompt, with
    no OS-held root (ruling 42);
  - macOS joins with the pairing plan's D2 (ruling 22).

**2026-10-06, the transport's seed path landed** (`83b06806`, merging
`ed741806`, `01b4f632`, `de91e5cf` and `d636a125`; rulings 51, 52 and 54):
- After bind, nothing in the transport asks p2panda's actor for iroh's
  endpoint.
- With `endpoint_addr` reverted to the actor path (the lane's
  measurement), 392 came back in 11 of 15 gossip runs. That makes it a
  control for the measurement, not for the test's verdict, because 392 is
  in p2panda-net's own baseline.
- **Verified in `mere-verify`:**
  - at `d636a125`: the transport suite 3 of 3; the dialling test that
    failed once for the lane, 5 of 5; the residue test; graphshell 191;
    djinn 91; the gate;
  - the gate again on the merge with `main` `b2cabfca`. The tree that
    landed is that merge.
- Knot `ef89a18` compiles against it with no errors (the lane's lockstep
  run).
- **Ruling 50's precondition is met.** The Knot lane's P1 work is in
  `ef89a18`, which is Knot's origin/main and djinn's pin. Knot's own mere
  rows are at `e0cea3e0`, which predates the borrowing entry points.

**2026-10-07, the Knot repin overtaken.**
- **The Knot work under ruling 55.** It produced `f68af0d` in
  `worktrees/knot-seed`, on Knot `6cb57f1`: every mere row moved to
  `ea74604b`, with S77. Its gates passed except knot-document's tests,
  which never finished.
- **The lane that wrote the seed fix died** on a usage limit, partway
  through its control. It left 21 uncommitted files and, in
  `worktrees/mere-knot-seed`, the strict `knot_residue.rs` and djinn's
  baseline rows. All of this is unverified.
- **Mark's "Repin knot and push"** was already done on Knot origin by
  `0096591` (at mere `57b4893d`, genet `965b64e`) before it was pushed from
  here.
- **Rulings 57 and 58:** `f68af0d` retired unpushed, the fix moves onto
  `ae3352e`, and mere repins once after the fix.

**2026-10-07, the seed fix on `ae3352e`, measured.**
- **Knot.** The fix replayed cleanly onto `ae3352e` (branch
  `seed-borrow-ae3352e` in `worktrees/knot-seed`). `cargo check --workspace
  --all-targets` passes.
- **djinn.** `resident_knot.rs` took the seed by value. It now holds it in
  `Zeroizing` across its awaits and lends it to Knot, and its test passes.
- **The instrument.** djinn's `knot_residue`, run against a by-path patch
  of each Knot tree on mere `973a7fc1` (WIP `727100bf` in
  `worktrees/mere-knot-seed`):
  - **positive control:** passes in both arms;
  - **`author`:** 4256 left on `ae3352e`, 0 with the fix;
  - **source and session, capture retention:** 0 in both arms;
  - **sync host, beyond iroh, p2panda-net and the overlay host with
    mDNS:** 536, 632, 920, 1016, 1680, 2272, 7616 and 7632 on `ae3352e`;
    the same six and 7568 with the fix.
- **Diagnostic runs** (marked in the test): the overlay host alone leaves
  nothing beyond the baselines with mDNS off, and 4 sizes with it on
  (ruling 60).
- **Rulings 59 and 60.** Next: the join baseline.

**2026-10-07, ruling 49 landed in Knot and Mere.**
- **The join baseline (ruling 59)** binds the overlay host under the default
  policy and calls Knot's `store.join` with the seed held by the test. It
  accounts for the six mid-size blocks exactly.
- **The last block was the run's own future.** In every run, the one large
  block matched that run's future size exactly: iroh alone 7720, the join
  6904, the sync host 7568. That copy sits in the run's outer frame, whose
  size changes with nesting, so the test compares outer frames by presence,
  not by size. The overlay host alone (future 2264) has no copy in its
  outer frame, so the copy comes from the join, awaited inline, below Knot.
- **Verdicts.** Against `ae3352e` the test fails: `author` leaves 4256, and
  the sync host leaves 7632 beside its 7616 outer frame. Against the fix it
  passes. *Reading, not ruled:* this outer-frame rule implements ruling 59.
  A Knot copy held only in its own outer frame could not be told apart from
  the join's copy; the strict paths still catch a seed taken by value.
- **Knot `eabd443`** is the fix rebased onto Knot `14cd06e`, which repins
  every Mere row to `f1d169c7`; pushed with Mark's OK.
  - Gates: the workspace check, `knot-editor`, `knot-desktop` and
    `knot-document` tests (33 suites, 565 passed).
  - One `knot-editor` lib test hung once in an earlier full run (6 hours,
    14 s of CPU). It passed alone and in a full rerun (149 in 17 s), so it
    is recorded as a one-off.
- **Mere repins Knot once (ruling 58)**: `knot-editor`, `knot-document`
  and djinn's `knot-site` are at `eabd4434`.
  - Gates: `cargo_mode.py verify`, and djinn's tests (20 suites, 116
    passed, `knot_residue` included, with no patch).
  - A first run hit a rustc out-of-memory on the shared machine; the
    rerun used `-j 4`.
- **Next:** ruling 60's default-policy run in mere-transport's
  `seed_residue.rs`, then ruling 42 (ruling 56's order).

**2026-10-08, ruling 60: the mDNS copies are p2panda-net's.**
- **mere-transport's `seed_residue.rs`** gains a third shape. The transport
  is built through `P2pandaHostPolicy::default()`, gossip on, and judged
  against p2panda-net alone with `MdnsDiscovery` (Active) spawned in the
  transport's order. It passes: the transport adds nothing beyond
  p2panda-net in any of the three shapes.
- **Attribution, measured and reported:**
  - iroh's own `MdnsAddressLookup`, built from the public id without
    p2panda, adds nothing beyond iroh alone;
  - p2panda-net's mDNS layer adds exactly the 4 blocks (472, 568, 784,
    2424).

  So they are p2panda-net's, in its `MdnsActor` layer, and not Mere's
  wiring. They are recorded in the upstream candidates ledger, item 11.
  The mechanism (ractor boxing a stack that still held key bytes) is
  inferred, not traced.
- **Dev-dependency:** `iroh-mdns-address-lookup = "=0.6.0"`, the version
  p2panda-net already locks. `cargo_mode.py verify` passes.
- **Ruling 61:** ledger only. Next: ruling 42.

**2026-10-08, ruling 42 assessed** (at `19de6eab`).
- **What exists:**
  - the passphrase vault locks and unlocks;
  - Linux has the native prompt: graphshell's `SystemNativeIdentityUi`
    uses `light-file-dialog`'s password box, through whatever graphical
    dialog provider the desktop offers;
  - `djinn --unlock` and `--native` (rulings 41 and 47);
  - the door keys captured at first unlock (ruling 46).
- **What is missing:**
  - **Nothing opens locked.** `PassphraseEncryptedStorage::open` and
    `IdentityVault::{open, with_profile}` all need the secret.
  - **djinn's `run()` does everything after an unlocked open:** the
    profile load, the door keys, a second vault open in
    `DjinnResident::open` (which re-reads the environment), and the Knot,
    Distillery, reservoir and sync lanes.
  - **No door before the first unlock.** `admit_local_client` asks the
    door for its keys before it admits anything, so the first unlock can
    only be the resident's own prompt (ruling 63).
  - **No first-run path off DPAPI.** `--enroll-passphrase` opens the
    sealed vault only (ruling 64).
  - **The environment in the harness.** djinn-testkit sets
    `PERSONAE_PASSPHRASE` (ruling 65).
- **Not in this scope:** `personae-agent` and `personae-vault` still use
  `Unlock::from_env()`. The standalone agent is Linux's deployed agent
  until the pairing plan's D2.
- **The build. Done when:**
  - [ ] personae opens a passphrase vault locked (salt only, profile id
        only). Every guard refuses until a passphrase unlock loads the
        profile; a wrong passphrase stays locked; tests cover each.
        *2026-10-08: superseded by ruling 66. The resident waits before
        building the vault, so personae is unchanged.*
  - [x] djinn's resident path reads no passphrase from the environment
        (measured by search). A control resident given
        `PERSONAE_PASSPHRASE` and nothing else stays locked.
        *2026-10-08: the search is clean for djinn and djinn-testkit. The
        control resident is not run: on Windows the native box is always
        available, so it would open a real dialog on the desktop mid-test.
        The live tests stand in for it: they pass with no environment.*
  - [x] A passphrase-vault resident starts locked, prompts, re-prompts on
        a cancel or a wrong passphrase, and serves its doors only after
        the unlock. The door keys, lanes and second open follow the
        unlock. *2026-10-08: the prompt logic by scripted unit tests; a
        live resident's events run `started, waiting-for-unlock,
        vault-created, listening, ready`. The real native box and terminal
        are Mark's attended step.*
  - [x] With no vault, the start prompt asks twice and creates it.
  - [x] `--passphrase-fd` exists only under the test or receipt feature,
        and the harness, its receipts and djinn's tests use it. *A plain
        build answers "unknown argument: --passphrase-fd".*
  - [ ] Gates: personae, castellan and djinn tests, the receipt, and
        `cargo_mode.py verify`. The Linux build and runtime proof go to
        the ThinkPad with the Secret Service (ruling 56).

**2026-10-08, ruling 42 built** (`55ff58e4`, branch `start-locked`).
- **What changed:**
  - djinn's new `startup_vault` chooses the vault and holds the prompt
    loop;
  - `run()` and the pairing commands open through it;
  - `--passphrase-fd 0` sits behind the `passphrase-fd` feature, which the
    tests turn on through djinn's dev-dependency on itself;
  - djinn-testkit hands the passphrase over on standard input;
  - graphshell's `NativeIdentityUi` gains `ask_vault_passphrase(message)`;
  - personae gains a named constant, `PASSPHRASE_VAULT_FILE`, and no other
    change.
- ***Readings, not ruled:***
  - any refused open asks again, with the reason shown, rather than matching
    personae's "incorrect passphrase" string;
  - only fd 0 is read, since Windows has no other inherited descriptor
    numbers;
  - the passphrase is kept in zeroizing memory only for Distillery's second
    open of the same directory.
- **Verified in the worktree:**
  - djinn's tests, startup_vault's 8 included; personae, djinn-testkit and
    castellan (32 suites, 346 passed);
  - graphshell's tests compile;
  - the live tests: `harness` 4 of 4, `lock_agent` and the two-resident
    directory test;
  - `cargo_mode.py verify`.
- **Finding, not this change's:** `mdns_first_contact_two_instance` fails
  at "a restarted: contact within 90s", 3 of 3 runs on the branch and 1 of
  1 on its base `19de6eab`, at the same step. It belongs to the device
  pairing plan's D1b receipts.
- **Still open:**
  - the Linux build and runtime proof on the ThinkPad, with the Secret
    Service (ruling 56);
  - the real native box and terminal prompt, which are Mark's attended
    step;
  - outside djinn, `Unlock::from_env()` remains in `personae-agent`,
    `personae-vault`, `distillery-installed`, graphshell's `profile.rs`
    and its web-extension smoke host.

**2026-10-08, the Secret Service assessed** (L2 checkpoint B, at
`18404a4c`).
- **What exists:**
  - castellan's D-Bus server (about 1,460 lines, Linux-only);
  - the store under it refuses while the resident is locked;
  - `secret_service_linux.rs` drives `secret-tool` (store, lookup, clear)
    under `dbus-run-session`.
- **What is missing** (§2's survey still holds):
  - `Locked` is a label flip on sets that start empty, not the vault's
    state;
  - `Unlock` never prompts;
  - no lock or unlock change reaches D-Bus;
  - nothing serves it in production.
- *Reading, not ruled:* ruling 11's secret-free snapshot covers the Secret
  Service's metadata (collections, labels, lookup attributes and content
  types, which the specification treats as not secret). So a locked search
  still answers and reports its items locked, and the client's `Unlock`
  meets ruling 67's prompt. Under ruling 66 a Linux resident builds nothing
  before its first unlock, so a snapshot exists whenever the service is
  served.
- **The ThinkPad.** It is at `192.168.4.32`, with ED25519 key
  `SHA256:9kM6RpW0UxjYmEdg5ngHw1JL8J7Hm7B8QXXJGKWkB7o`, as recorded. Rust
  1.98.1, `secret-tool`, `dbus-run-session`, `gdbus` and `zenity` are there.
  Another session's checkout and builds are left alone; this work runs in
  a worktree of its own.
- **The build. Done when:**
  - [x] collections and items report `Locked` exactly when the vault is
        locked, and a lock or unlock emits the property change;
  - [x] while locked, a search answers from the snapshot with every item
        locked, `GetSecrets` returns nothing, and `Item.GetSecret` fails
        `IsLocked`;
  - [x] `Unlock` while locked returns a Prompt. Its `Prompt()` runs the
        handed-in unlock: success completes with the unlocked objects, and
        a cancel completes as dismissed with the vault still locked;
  - [x] a client `Lock` of any object locks the vault;
  - [x] proven on the ThinkPad under `dbus-run-session`: `secret-tool
        lookup` on a locked vault brings up the scripted prompt and
        returns the secret, and with a cancel returns nothing. The
        properties and refusals are read with `gdbus`. The existing
        receipt still passes. *2026-10-08: the reads use the receipt's
        own bus connection instead of `gdbus`. Each `gdbus` call is a new
        connection, so it cannot hold the transfer session `GetSecret`
        needs.*
  - [x] castellan's tests pass on Windows and Linux.

**2026-10-08, the Secret Service built and proven** (`aac67a85`, branch
`secret-lock`; checkpoint B's Secret Service).
- **What changed:**
  - castellan's store gains `MetadataSnapshot`, ruling 11's snapshot for
    the Secret Service, held in memory and never written;
  - `serve()` takes the host's `SecretServiceVault` (is locked, a watch,
    lock, the native unlock prompt);
  - `Locked` follows it, and a watcher announces each change as
    `PropertiesChanged` on every object;
  - while locked, reads answer from the snapshot;
  - `Unlock` returns a Prompt object (`prompt.rs`);
  - a client `Lock` of any collection, alias or item locks the vault.
- ***Reading, not ruled:*** the snapshot is retaken at serve, at each
  unlock and after each write made through the service. An edit made
  through another surface while unlocked shows in the locked view after
  the next of those.
- **Verified on the ThinkPad (Fedora 44, under `dbus-run-session`):**
  - the new receipt: Lock, the announcement, the locked reads, the
    refusals, a cancelled prompt and an unlocking one through
    `secret-tool lookup`;
  - the existing store, lookup and clear receipt;
  - castellan with all features (5 suites, 127 passed);
  - **control:** `locked()` hard-wired to false fails the receipt at the
    collection's `Locked`. The first draft read that property through a
    caching proxy, which answered from the very signal under test; every
    read is uncached now.
- **Verified on Windows:** castellan by default (67) and with
  `secret-service` (18), and `cargo_mode.py verify`.
- **Fixed in passing:** personae's `ssh_ca_live` (`#![cfg(unix)]`, behind
  `ssh`) had not compiled since `7926d3a8`, which moved the proofs to
  insigne; it lacked `use personae::delegation::Issue`.
- **Finding, Linux only:** personae's `no_residue` fails "sealed vault
  locked". The sealed root key, raw, is in a 564-byte block freed
  uncleared, allocated during `save`. This happened in 3 of 3 runs on the
  ThinkPad. On Windows, at the same commit, every scenario is clean. L1's
  instrument has not been run on Linux before, so L1's residue condition
  holds on Windows only. Not yet traced.
- **Still open:** djinn's wiring and the hand-over from gnome-keyring
  (ruling 69); ruling 42's Linux runtime proof; the Linux residue above.

**2026-10-08, ruling 70's trace.**
- **Method:** on the ThinkPad, a temporary trap (`int3`) in the tracker's
  allocator fired at the 564-byte allocation during `save`, and `gdb`
  printed its stack. A temporary hex dump showed the block's contents.
  Both were reverted.
- **The block** was the `HashMap` table of `no_residue`'s own
  `canary_profile()`, allocated inside the measured window. Beside heap
  and stack pointers it held 16 bytes of the root key. The key was the
  test's own stack local, passed by value to `open_with_key` and picked up
  through padding when slot values were moved into the table.
- **Building the fixture unarmed** clears "sealed vault locked" (7 of 7
  runs). It then shows "passphrase vault locked" failing in 3 of 4 runs:
  the master seed in a 564-byte block allocated at open and freed at lock.
  That is the vault's own slot table, by the same mechanism, in personae's
  code. Rulings 71 and 72 settle both.

**2026-10-08, rulings 71 and 72 built** (`8eab9fcf`).
- **`Profile::slots` is a `SlotMap`.** It is a small vector whose whole
  buffer, padding and spare capacity included, is zeroed (with `zeroize`,
  volatile) on drop, after a removal, and before an outgrown buffer is
  freed. It keeps the `HashMap` methods callers use, and personae, castellan,
  pandect, graphshell and djinn compile unchanged. The three loaders build it
  with the slot count up front, so loading never grows it.
- **`no_residue`'s `sealed_lock`** builds its canary profile unarmed and
  drops it after disarming.
- **Verified on Linux (ThinkPad):**
  - `no_residue` clean 10 of 10 (with only the fixture fix: 1 of 4);
  - personae with all features (207, three full runs) and castellan with all
    features.
- **Verified on Windows:** personae and castellan with all features, `no_residue`
  clean (10 suites, 345 passed).
- **So L1's no-residue condition now holds on Linux as well as Windows.**
- **Seen in passing, neither from this change:**
  - personae's `authoritative_opening_is_exclusive_until_every_clone_drops`
    failed once in four full Linux runs ("authority is already held").
    It passed 5 of 5 alone and the next three full runs. It is probably a
    child process from the cross-process sibling test briefly holding the
    lock across fork; not traced.
  - djinn's `embedded_reservoir_two_process`, added today (`df0e0804`),
    fails 3 tests on Windows with "All pipe instances are busy" (os error
    231). It fails the same way at `origin/main` without this change, as a
    control. It belongs to that lane.
- **Next:** ruling 42's Linux runtime proof on the ThinkPad, then L3.

**2026-10-08, ruling 42 proven on Linux** (ThinkPad, Fedora 44; djinn
built plain at `fc3da34c`).
- **The plain build** answers "unknown argument: --passphrase-fd", as on
  Windows.
- **Terminal path.** Each run used isolated roots and endpoints, as
  djinn-testkit isolates them. A small Python pty driver typed each answer
  only when its prompt appeared, since the ThinkPad has no `script(1)`:
  - **no vault:** `started, waiting-for-unlock, vault-created, listening,
    ready`. The terminal asked "No identity vault yet. Choose a passphrase
    for a new one." then "Type the new vault passphrase again.", and
    `vault.json` was created;
  - **a wrong passphrase, then the right one:** `started,
    waiting-for-unlock, unlock-refused, unlocked-at-start, listening,
    ready`, with the reason shown before asking again. The status route
    reported `startup_unlock` and `protection` as `passphrase`;
  - **environment control** (`PERSONAE_PASSPHRASE` set, nothing typed): it
    stays at `started, waiting-for-unlock`, the terminal waiting, never
    ready.
  - Each resident stopped through its own door.
- **Native path** (Mark at the ThinkPad). The resident ran with no
  terminal, only the desktop's display (`DISPLAY=:0` and the Xwayland
  authority). GNOME's passphrase box came up. Eleven refused attempts were
  each answered by the box again, then the right passphrase gave
  `unlocked-at-start, listening, ready`, and later `stopping, stopped`. The
  box's own Cancel was not exercised natively; the scripted unit test
  covers it.
- **Testing note:** with `XDG_RUNTIME_DIR` redirected to a scratch root,
  GTK's box started the document portal's FUSE mount and `gvfsd-fuse` in
  it. Both outlive the resident and must be unmounted and stopped
  afterwards; they were.
- **L2's last condition** (the Secret Service on the ThinkPad) and this
  proof, which ruling 56 pairs with it, are done. Next: L3, the triggers.

**2026-10-08, L3 assessed** (at `2dff736e`).
- **What exists:**
  - the explicit intent (castellan's lock intent and `ssh-add -x`);
  - djinn's Windows idle probe (`conditions.rs`, `GetLastInputInfo`; Linux
    reads nothing);
  - `StartupUnlockMode::Locked` as an unused variant.
- **What does not exist:** no session-lock or suspend listener, no idle
  trigger, no persisted lock, and no lock settings.
- **On the ThinkPad:**
  - logind offers the session's `Lock` signal and `LockedHint`, plus
    `PrepareForSleep` and `Inhibit` (a delay lock lets a process lock
    before sleep);
  - GNOME's idle monitor reads idle exactly; logind's `IdleHint` never
    moves (ruling 73).
- **On Windows:** `Win+L` arrives as a WTS session notification (it needs a
  message-only window), and suspend through
  `PowerRegisterSuspendResumeNotification` (no window).
- **The build, in checkpoints:**
  - **A, the persisted lock (rulings 5, 32, 76):**
    - a marker beside the vault written at lock (no secret, advisory);
    - `startup_unlock`'s loaders refuse while it is present;
    - a loader that takes `OsPresence` passes;
    - an unlock clears it;
    - a sealed vault opens locked (ruling 38's constructor);
    - djinn's DPAPI start waits at its prompt under the marker.
  - **B, the triggers' core:**
    - `lock.toml` (ruling 77);
    - the idle rule with an injected clock (rulings 20, 74);
    - a dispatcher that locks on session lock and before sleep (ruling 75),
      from injected signals.
  - **C, Windows sources:** idle, `Win+L` and suspend.
  - **D, Linux sources:** Mutter, then logind, for idle; logind's `Lock`;
    `PrepareForSleep` under a delay inhibitor.
  - **E, the attended run (ruling 78).**
- **L3 done when** (§4, plus):
  - [x] the idle rule fires after the window, resets on activity, and
        counts unknown as idle (unit tests with an injected clock);
  - [x] settings absent or malformed mean the defaults; each trigger can
        be turned off;
  - [x] the dispatcher locks on an injected session lock and before an
        injected suspend acknowledges;
  - [x] a resident locked, then killed, comes back waiting at its prompt
        and opens only on an unlock (a receipt), while one never locked
        auto-unlocks as before;
  - [x] the attended receipts: Windows `Win+L` and suspend, Fedora
        `loginctl lock-session` and suspend.

**2026-10-08, L3 checkpoints A to D built** (`ed1de0f7`, branch `l3`).
- **A, the persisted lock:**
  - personae: `persist_lock`, `clear_persisted_lock` and `lock_persisted`
    (the marker `locked` beside the vault's root);
  - the loaders refuse under it, and a loader that takes `OsPresence`
    passes;
  - `SealedProfileStorage::open_locked`;
  - castellan: `ResidentLock` writes the marker on every lock, `ssh-add -x`
    included, and clears it on unlock, but only for a host made with
    `with_persisted_lock`, which only djinn's resident is;
  - djinn: a DPAPI start under the marker waits at its prompt (Hello, then
    the passphrase) and clears it.
  - *Reading, not ruled:* a handed-over passphrase (`--passphrase-fd`) now
    selects the passphrase vault only where no OS-rooted vault exists yet;
    beside one, it answers the persisted lock (refines ruling 66's
    reading).
- **B, the policy:** `lock_triggers`: `LockSettings` from `lock.toml`, the
  `IdleRule`, `Triggers`, and a `TriggerHost` that locks synchronously and
  does nothing while locked.
- **C, Windows:**
  - `Win+L` through WTS session notifications to a message-only window;
  - suspend through `PowerRegisterSuspendResumeNotification`, whose
    callback locks before it returns;
  - idle from `GetLastInputInfo`.
- **D, Linux:**
  - logind's session `Lock`, found through `User.Display`;
  - `PrepareForSleep` under a delay inhibitor, released after the lock and
    taken again on wake;
  - idle from Mutter's monitor, else logind's `IdleSinceHint` (0 means
    unknown).
- **Test residents** start with every trigger off unless the test writes
  its own `lock.toml`, because they read the real session's input and
  lock.
- **Verified on Windows:**
  - personae (211), castellan (122 and its suites), djinn's unit tests
    (the new 7 included);
  - `locked_restart`, which passes; its control (no persisted lock) fails
    at "the lock persists beside the vault";
  - `lock_agent` and `harness`.
  - `harness`'s graceful-stop test was refused a stop once in a long serial
    run, then passed 2 of 2 alone; it is intermittent.
  - The reservoir two-process failures are the ones already on main.
- **Verified on Linux (ThinkPad):**
  - djinn compiles with no warnings of its own; the triggers' 7 and
    personae's persisted-lock tests pass;
  - a scratch resident with its triggers on read `lock.toml`, found the
    graphical session, and held "djinn · sleep · Lock the identity vault
    before sleep · delay" in `systemd-inhibit --list`, with no warnings.
- **Still to do: E**, the attended receipts (`Win+L` and suspend on
  Windows; `loginctl lock-session` and suspend on Fedora). They wait for
  Mark's word (ruling 78's annotation).

**2026-10-08, L3 checkpoint E, the attended receipts.** Mark's word:
**"let's do it"**. djinn built from mere main (`463c8d40`, which carries
L3's `79fbbeb7`) on both machines.
- **Method:** two scratch residents per machine, each with its own vault
  and a `lock.toml`. A has only `session_lock` on and B only `suspend`, so
  each leg's other resident is its control. A lock is read from the
  events file (`lock-trigger` with its reason, then `locked`) and from the
  marker `vault/locked`.
- **Fedora (ThinkPad, GNOME on Wayland, session 8):**
  - `loginctl lock-session 8` (over ssh): A locked, reason `session-lock`,
    marker written; B stayed open.
  - Suspend (Mark's hands; polkit refuses `systemctl suspend` from an ssh
    session): B locked, reason `suspend`, at 21:17:49.389.
    `systemd-suspend` started at 21:17:50.451, and the kernel entered
    suspend at 21:17:50.493, so the lock came 1.06 s ahead of the sleep.
  - After the wake, both residents had taken their delay inhibitors again.
- **Windows (Modern Standby, S0 low-power idle; sign-in after sleep is
  immediate, `DelayLockInterval` 0):**
  - `Win+L`: A locked, reason `session-lock`, at 21:38:44.594, marker
    written; B stayed open.
  - Lid closed, A already locked: Kernel-Power 506 (entering Modern
    Standby) at 21:38:54.492. B locked, reason `suspend`, at 21:38:59.557,
    5.1 s later, and 0.1 s before the next standby phase (566, 1 to 2, at
    21:38:59.654).
  - Lid closed with no `Win+L`, fresh residents: 506 at 22:08:43.332.
    B locked, `suspend`, at 22:08:43.718; A locked, `session-lock`, at
    22:08:43.737, because Windows locks the session as standby begins. The
    next phase (566, 6 to 7) came at 22:08:43.982.
- **Findings:**
  - On Modern Standby, `PBT_APMSUSPEND` arrives when standby leaves its
    first phase, not at lid close: 0.4 s once and 5.1 s once in these
    runs. Under the defaults, the session lock lands at standby entry, so
    there is no gap. A resident with only `suspend` on can stay unlocked
    for those seconds with the screen off. *Reading, not ruled:* left
    as is, since the default settings cover it.
  - The Linux source takes its sleep delay whatever `suspend` says, so a
    resident with `suspend` off still lists an inhibitor; it lets go as
    soon as the signal arrives, and a settings reload needs no new one.
    *Reading, not ruled:* harmless, left as is.
- L3 is done; the scratch residents are stopped and their directories
  removed.

**2026-10-08, L4 done: the docs, the gates and what the gates found.**
- **Docs** (rulings 12, 82 and 83):
  - `UnlockTier`'s docs in `personae/src/vault.rs` say consent: when the
    approval broker asks, with every slot decrypted while unlocked;
  - the protocol plan's §3.6 and §3.7 carry dated annotations on what
    holds now;
  - the tier architecture gains invariant 13 with its four enforcement
    points;
  - the [vault threat statement](../technical_architecture/2026-10-08_vault_threat_statement.md)
    is new.
- **The gates** ran on mere `526cbb3b` plus this work, in fresh worktrees.
  - **Windows:** personae, castellan and djinn give 477 passed across 37
    targets. The ignored `harness`, `lock_agent` and `locked_restart`
    receipts pass. The installed resident (PID 14756, started 2026-10-05
    05:06) is the same before and after every run.
  - **Fedora:** the same crates give 464 passed across 35 targets,
    compiling the logind and Mutter sources on their own target.
    `secret_service_linux` passes 2 of 2 under `dbus-run-session`. On the
    user's own bus it fails with `NameTaken`, since gnome-keyring owns
    the name.
- **What the gates found, and what was done** (rulings 84 to 90):
  - **The first Linux gate failed castellan's residue test.** Only under
    djinn's unified features, 3 of 3: a live block held the master seed
    after the lock. The cause was stack residue: every by-value move of
    `IdentityVault` left a seed copy in dead stack, and an enum's
    uninitialized payload carried one into a heap block (ruling 90). The
    key is now boxed, and the test is clean 3 of 3 on Fedora and clean on
    Windows, with its positive control firing each time.
  - **The reservoir fixtures** failed on Windows with error 231.
    `connect_local` now retries a busy pipe (ruling 84), and both
    fixtures pass. On the same commit without the fix, both fail.
  - **The graceful stop** is raised when the asking session ends, or
    after 2 s (rulings 85 and 88), and the testkit records the stop
    command's output. The new unit test covers both paths, and `harness`
    passes 5 of 5.
  - **personae's two authority tests** take turns (ruling 86).
  - **The env-reading library openers are gone** (ruling 89).
    `distillery-installed` reads the environment itself, and the smoke
    script's dead `PERSONAE_PASSPHRASE` line is removed.
  - **The Secret Service's real serving** has its own
    [plan](2026-10-08_secret_service_plan.md) (ruling 87).
  - **Scoped, and left with their owners:** the mDNS restart failure is
    the pairing plan's ruling 36, waiting on iroh-gossip (ruling 72).
- **Still open, by ruling:**
  - ruling 44's transport-key hard switch, so Distillery keeps the master
    keypair while locked;
  - pandect's wallets until D8 (ruling 81).

**2026-10-09, L4's Windows gate rerun in its own build directory** (the
dynamics grammar plan's F183).
- **Why:** F183 withdrew F145's shared `C:/t/cargo-build/mere`. Its
  freshness check runs by modification time, so one worktree can build
  with another tree's artifacts. L4's two Windows gate runs, and the
  djinn binary for L3 checkpoint E's Windows legs, were built through that
  directory.
- **The rerun:** on main `17519aa4`, which carries L4, in a fresh
  worktree with its own `C:/t/cargo-build/mere-l4recheck`, built cold.
  Personae, castellan and djinn give 477 passed across 37 targets, the same
  as before. The ignored `locked_restart`, `harness` and `lock_agent`
  receipts pass. The installed resident (PID 14756) is unchanged.
- **The Fedora gates and receipts stand.** They built in each worktree's
  own `target/`.
- **Checkpoint E's Windows legs** ran a binary of unproven provenance.
  Their events matched L3's code, and rerunning them with an isolated
  build waits for Mark's hands.
  - *Rerun 2026-10-09 (Mark: "let's do it now"),* with djinn built in
    that worktree's own directory and the same two residents.
  - `Win+L` locked A at 00:27:12.881, reason `session-lock`, with its
    marker; B stayed open.
  - Lid closed with no `Win+L` on a fresh pair: Modern Standby began at
    00:28:01.716 (Kernel-Power 506). B locked, `suspend`, at 00:28:01.827,
    and A locked, `session-lock`, at 00:28:02.139, both with markers.
    Standby's next phase (566, 9 to 10) came at 00:28:02.357.
  - E's Windows legs hold on an isolated build.

**2026-10-09, ruling 44 assessed** (Opus, read-only, at `5df06f80`; the
headline checked in code).
- **The hard switch breaks no live peer relationship.**
  - Each vault generates its own master (`bootstrap.rs:138`), so two of
    Mark's devices never share a Distillery mesh.
  - The production lane binds gossip and blobs only, with no discovery
    (`installed.rs:319-322`).
  - djinn keeps `NoCourier` until a second device (`resident_distillery.rs:322-326`).
  - Nothing in production announces on the mesh.
  - So the mixed-version window ruling 44 worried about matters only once
    multi-device Distillery is wired.
- **Who keeps the master after a lock:** p2panda-net's `Endpoint`, its
  gossip and discovery handles, and iroh's endpoint secret, held through
  `ResidentDistillery` in `DjinnResident`, which ruling 24 keeps open.
- **Address by master** exists only in code that tests exercise:
  - `DeviceDirectory` (`crates/mesh/mesh/src/directory.rs`);
  - the courier's `PeerID::from_bytes(master)`;
  - `remote.rs`'s `master_of` checks.
- **Not affected:** device pairing (per-graph derived node ids) and Knot
  (its own derived seed).
- **Rulings 92 to 95 settle the build.** Its checkpoints:
  - A, the derivation;
  - B, the lookups, with a master-bound negative control;
  - C, a residue test that fails first on the current tree;
  - D, the gates with the lane on and locked by the real trigger;
  - E, the records.
