# Vault Threat Statement

**Date:** 2026-10-08
**Status:** current. What the identity vault's lock defends, what it leaves
open, and what is known only by reading. Ruled into being by the
[vault lock plan](../implementation_strategy/2026-10-05_vault_lock_plan.md)'s
ruling 82; the lock's architecture-level promise is the
[dramatis tier architecture](2026-09-30_dramatis_tier_architecture.md)'s
invariant 13 (ruling 83). It replaces the
[protocol architecture plan](../../mere_docs/implementation_strategy/2026-05-05_protocol_architecture_plan.md)'s
§3.7 promise of "a future doc" for the vault's part of the threat model.
**Audit base:** Mere `526cbb3b` (2026-10-08), which carries the lock plan's
L1 to L3.

Each claim names its ruling or its code. A claim marked *read* was checked
in source; a claim marked *measured* has a test or receipt that fails when
it stops being true.

---

## 1. The adversary

- **Someone with the device while it is locked:** at the lock screen,
  asleep, or after a restart. This is the case the lock exists for.
- **Someone with the disk alone:** a stolen or imaged drive.
- **Another process of the same user:** it runs with the user's rights.
  It is defended against only where this statement says so.
- **Code inside the resident's process:** it is trusted (level 0 of the
  protocol plan's §3.7). Nothing here defends against it.

## 2. What the lock defends

**Locked means no secret is reachable** (rulings 1, 3 and 6; invariant 13).
A lock drops every secret the resident exercises:
- the personae vault's profile and storage key;
- castellan's record and freshness keys, in one shared cell, so every clone
  locks together (ruling 30);
- Knot's signing seed, by closing the Knot lane (rulings 43 and 48).

Every holder drops its keys before `lock()` returns: castellan's
`ResidentLock` runs synchronous holder hooks, then broadcasts the new state
(ruling 31, `ports/castellan/src/authority.rs`).

**The memory those secrets held is cleared, not just freed** (ruling 6).
*Measured* on the heap by a tracking allocator that fails on a positive
control:
- personae's vault (`crates/dramatis/personae/tests/no_residue.rs`);
- castellan's item store (`ports/castellan/tests/no_residue.rs`);
- Knot's seed through djinn (`ports/djinn/tests/knot_residue.rs`);
- mere's transport (`crates/murm/transport/tests/seed_residue.rs`, rulings
  51 and 52).

The residue was fixed for each source in turn:
- serde_json's reallocation copies, by `zeroizing_json` (ruling 34);
- argon2's working memory, which we supply and zeroize through its public
  API (ruling 33);
- knot-editor taking the seed by value (rulings 49 and 50).

**Four things lock it** (rulings 3 and 20), each one per device:
- an explicit intent (the Lock action, or `ssh-add -x`, per ruling 9);
- the OS session locking;
- suspend;
- 15 minutes idle, where an unknown idle reading counts as idle.

*Measured* on 2026-10-08 with real input (the lock plan's §6, L3
checkpoint E):
- **Fedora:** the session lock locked at once. Suspend locked 1.06 s
  before the kernel suspended, under a logind delay inhibitor (ruling 75).
- **Windows:** `Win+L` locked at once. On Modern Standby the session
  lock lands as standby begins (when sign-in after sleep is immediate). The
  suspend notification arrived 0.4 s and 5.1 s after lid close in two
  runs, each time before standby's next phase.

**The lock survives a restart** (ruling 5):
- A marker beside the vault's root, written on every lock, makes personae's
  startup loaders refuse an unattended open (ruling 32,
  `crates/dramatis/personae/src/startup_unlock.rs`).
- A resident restarted under it waits at its prompt. *Measured* by
  `ports/djinn/tests/locked_restart.rs`, whose control fails without the
  marker.

**Unlocking takes a user act on the resident's own surface:**
- **Windows:** Windows Hello, then the passphrase (rulings 4 and 21).
- **Linux:** the vault is a passphrase vault and starts locked (ruling 42).
- No route unlocks it over the wire. `ssh-add -X` is refused (ruling 9),
  and the Secret Service's `Unlock` returns a Prompt (ruling 10).
- A resident never reads `PERSONAE_PASSPHRASE` (ruling 7). A passphrase
  is handed over only through `--passphrase-fd` (ruling 65).

**Failed unlocks are slowed by Argon2id's cost alone** (ruling 19).

**The disk alone is not enough:**
- **Windows:** the vault is sealed under DPAPI, bound to the user's logon.
- **Linux:** the passphrase vault is sealed under Argon2id and
  ChaCha20-Poly1305.

## 3. What stays while locked, by ruling

- **Labels and public keys,** in a secret-free snapshot taken at lock time
  (ruling 11).
- **The door's session-signing key.** It is derived under its own name and
  only authenticates the resident to its apps (ruling 40).
- **Distillery's transport identity,** so sync continues (ruling 24).
  Until ruling 44's hard switch to a derived transport key lands, the
  running transport keeps the **persona's master keypair** in memory while
  locked. `transport_identity()` itself refuses while locked (*read*,
  `ports/distillery/src/installed.rs`). This is the largest gap the lock
  leaves today.
- **Pandect's wallets.** Knot's seed, which comes from one, is dropped
  when the Knot lane closes (ruling 43). The wallets themselves know no
  vault, so after a restart they reopen unattended until the dramatis repo
  plan's D8 moves them into castellan (ruling 81).

## 4. What the lock does not defend

- **The pagefile and the hibernation file.** While unlocked, any page
  holding a secret can be written to swap, and a copy written there
  survives the lock that clears the page in RAM. Ruling 6 chose not to
  lock pages or exclude dumps; a page-locking allocator would be a security
  mechanism of our own (the lock plan's §5). A hibernation that follows the
  suspend notification happens after the lock (§2). A hibernation without
  one, forced or by firmware policy, is unexamined.
- **Crash dumps** of an unlocked resident hold its secrets.
- **Other processes of the same user.**
  - On Windows (AutoOs), any of them can ask DPAPI to unseal the vault's
    root file without a prompt. The persisted lock is advisory against
    them: they can read the root anyway (rulings 5 and 32).
  - They can read the resident's memory while it is unlocked: on Windows
    through the user's own process access, and on Linux wherever Yama's
    `ptrace_scope` is 0, as it is on the Fedora ThinkPad (*read*,
    2026-10-08).
- **Code in the resident's process** (level 0). The protocol plan's §3.7
  sketches capability tokens, WASM and per-process isolation as later
  levels.
- **`UnlockTier` is consent, not custody** (ruling 12). Per-use and
  short-TTL slots are fully decrypted while the vault is unlocked. The
  tiers decide when the approval broker asks, not what is in memory.
- **Other CLIs and tests** still read `PERSONAE_PASSPHRASE` (ruling 7 kept
  them). The passphrase then sits in that process's environment.

## 5. Known only by reading

- **Stack copies of keys,** a documented `zeroize` limit that the
  instrument cannot see directly. They are not harmless. On 2026-10-08 a
  stale seed copy in dead stack reached a live heap block through an
  enum's uninitialized payload, and castellan's residue test caught it on
  Fedora (vault lock plan, ruling 90). The key now lives on the heap
  (`Ed25519Keypair` holds a `Box<SigningKey>`), so moving a keypair or a
  vault copies a pointer. Building a key still passes its seed through the
  stack once.
- DPAPI's `LocalAlloc` output buffer, which is cleared before `LocalFree`
  (`crates/dramatis/personae/src/startup_unlock.rs`).
- What iroh and p2panda-net leave per bind and close. Measured as
  unchanged by our transport (ruling 52), and recorded in the upstream
  ledger rather than fixed here.
- argon2's own frees, which are not cleared in 0.5.3 or 0.6.0-rc.8. We
  supply the memory instead (ruling 33; upstream ledger item 9).
