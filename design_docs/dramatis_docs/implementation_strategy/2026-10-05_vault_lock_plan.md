# Vault Lock Plan

**Date**: 2026-10-05
**Status (2026-10-07)**: rulings 1 to 60 in §3; the threat statement is
still open. L1 landed (`2556a20c`). L2's checkpoints A (`7c588deb`) and B
(`ec1768ab`) landed. Still to come in L2: the Secret Service on the
ThinkPad, ruling 42 (Linux starts locked), ruling 44 (Distillery's
transport key) and the seed residue fixes (rulings 49 to 51). The
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

Still open: a threat statement naming hibernation and the pagefile.

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
- **Still open:** what to do about the 4 blocks, since `mere-p2panda-net`
  is our fork. Then ruling 42.
