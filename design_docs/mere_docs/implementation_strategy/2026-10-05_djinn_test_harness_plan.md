# djinn Test Harness Plan

**Date**: 2026-10-05
**Status (2026-10-05)**: assessed; all ten forks ruled (§3). Next: H1 to H3
in a lane. No code
changed. The vault lock plan's build waits on this harness (its ruling 18).
**Scope**: one shared, tested way to run djinn residents under test:
isolated, observed without scraping logs, stopped and restarted, held
behind enforced walls around the installed resident, and recorded as
receipts. It replaces the per-lane harnesses rebuilt in the week before.
It serves djinn's own receipts first, then the vault lock's (triggers,
restarts, the locked agent, Secret Service prompts) and the crates that
share djinn's patterns.

**Related**:

- [vault lock plan](../../dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md):
  ruling 18 puts this harness first; its receipts are the first customer.
- [device pairing by key plan](2026-10-02_device_pairing_by_key_plan.md): D1
  and D1b, whose two-resident harnesses this replaces.
- [djinn family resident services plan](2026-08-22_djinn_family_resident_services_plan.md).

---

## 1. Why

Mark, 2026-10-05: **"i have been wondering about our testing framework for
djinn too... and it's the daemon/cli thing for the stack, no? seems
important"**. In the week before, each lane rebuilt the same two-resident
machinery (D1, D1b, the restart traces, the iroh 1.3 timing runs, P4a's
ThinkPad receipt), and checked the installed resident by hand, by name or
by a pinned process id.

## 2. What exists (assessed 2026-10-05)

A read-only lane (Opus) read the code, the lanes' scripts and their logs.
Claims re-checked in code are marked *checked*.

- **The installed resident's process id changes.** The laptop booted at
  04:18 on 2026-10-05, and `graphshell-device-host.exe` is now PID 14756
  (*checked*), not the 53336 every brief named. P4a's receipt script, which
  pinned 53336, would now throw; five other scripts check by name only,
  which proves presence, not identity. The wall has to be whatever holds
  the installed executable and its pipes, captured when a run starts.
- **Two patterns exist.** Spawning the real `djinn` binary with environment
  isolation (D1, D1b), and re-running the test binary as a child steered by
  environment variables (`mere_two_process.rs`, `reservoir_two_process.rs`,
  the transport's killed-peer test). `tests/common/mod.rs` is a Distillery
  fixture with no process, pipe or isolation helpers.
- **Duplicated pieces:**

  | Piece | Copies |
  |---|---|
  | isolated resident (environment, endpoints, roots, spawn, kill) | D1, D1b, two patched copies, P4a's script |
  | unique endpoint names | six places |
  | polling waits | five places |
  | re-exec child with an environment role | three places |
  | readiness | log scraping, pipe enumeration, route polling |
  | the installed-resident wall | five scripts by name, one by pinned PID |
  | load sampling (`rustc` count) | six scripts |
  | trace-filter source patches | two names, `DJINN_TRACE` and `DJINN_SCRATCH_LOG` |
  | controls made by patching source | three scripts |
  | run loops and tallies | five scripts plus `tally.py` |
  | event-relative timelines | three scripts |
  | stray-process checks | three scripts, by name |
  | `authorized_keys` before and after | P4a, twice |
  | receipt records | four formats |

- **What djinn exposes without logs:** the `device-directory-v1` route and
  `djinn-devices --json`, `--pairing-facts`, the agent on its receipt pipe,
  and pipe existence. Startup mode, protection, the listening ticket, peer
  state and the exit reason are in logs only. There is no ready signal, no
  graceful stop a child can receive on Windows besides killing, no event
  stream, and no log-filter flag (hence the two scratch patches).
- **Walls today.**
  - castellan refuses the standard agent pipe for a receipt listener on
    Windows only. Pipes are bound first-instance, so a collision fails
    rather than hijacks, except in the installed resident's 5 s restart
    gap.
  - On Unix, djinn's default agent endpoint is `$SSH_AUTH_SOCK`, and it
    deletes that socket file when a connect is refused
    (`djinn.rs:1001-1022`, *checked*). That is ordinary stale-socket
    cleanup, but a test run without its own endpoint, while the user's
    agent restarts, would take the user's socket.
  - `djinn-devices` has no endpoint flag and falls back to the installed
    app door.
  - The Secret Service's "disposable bus" rule is a doc comment only.
- **The lock removes today's unlock path.** Every subprocess test unlocks
  through `PERSONAE_PASSPHRASE`; vault lock rulings 7, 9 and 10 take that
  away from lock-enabled residents and make unlock native-only, and djinn
  wires the system unlock surface directly (`djinn.rs:815-838`).
- **Triggers have no seam.** djinn has no message loop. Idle is a free
  function and is unavailable off Windows. An injected clock exists only in
  Distillery and castellan's OTP. A real `Win+L` or suspend on the laptop
  stops everything running on it, the harness included.
- **Precedents to reuse:**
  - the Windows installer's structured `mere.djinn.windows-cutover/v1`
    record, which matches processes by executable path;
  - committed receipts under `mere_docs/testing/receipts/`;
  - the transport's compiled-in control switch (`without_connection_hook`)
    in place of source patches;
  - graphshell's `ScriptedUi`;
  - feature-gated test support in five crates.
- **Load.** D1's stopped-peer time was 74 to 85 s on a quiet machine and 103
  to 312 s under other sessions' builds, while assertions relative to a
  run's own events held under load.

## 3. Forks for Mark

Each comes with the lane's recommendation first.

1. **Where the harness lives.**
   - **Recommended:** a `djinn-testkit` crate (`publish = false`), a path
     dev-dependency that the other crates and repos sharing the pattern can
     use.
   - A module inside djinn's tests.
   - A `test-support` feature in djinn.
   - A receipt binary driven by scenario files.
2. **Language.**
   - **Recommended:** Rust for logic and records, with shell only to
     launch (for example `dbus-run-session -- cargo test`); Windows
     `ssh.exe` is called from Rust.
   - Rust plus committed scripts for remote runs and loops.
   - Per-lane scripts, as now.
3. **Timing under load.**
   - **Recommended:** assert relative to the run's own events, keep
     absolute times as wide bounds that are recorded, record a load sample
     in every receipt, offer an opt-in quiet-machine mode, and serialize
     live receipts with a machine-wide lock file.
   - A hard quiet gate.
   - Bounded retries, every attempt recorded.
   - Patience only.
4. **Real-machine receipts.**
   - **Recommended:** a remote runner for the ThinkPad and the Surface
     (bundle, separate worktree, run, fetch the record); macOS with D2; and
     the laptop's own `Win+L` and suspend as attended steps, where the
     harness arms, Mark acts and the harness records.
   - Local and Linux-over-SSH, with attended steps as checklists.
   - Manual throughout.
5. **The receipt record.**
   - **Recommended:** a versioned JSON record per run containing:
     - commit and binary hash;
     - machine and load;
     - the walls before and after;
     - timestamped steps;
     - assertions, expected against observed;
     - controls and whether each failed;
     - evidence hashes.

     Raw records stay outside the tree, since they hold test vaults, and a
     committed summary goes under `mere_docs/testing/receipts/`. A verifier
     recomputes the hashes.
   - A committed hand-written `RECEIPT.md` only.
   - Stdout lines, as now.
6. **What djinn exposes for observation.**
   - **Recommended:** production surfaces apps want too: an owner-only typed
     `resident-status-v1` route (startup mode, lock state, endpoints, the
     listening ticket, readiness) and a `--log-filter` flag, which retires
     both scratch patches.
   - Also a JSON-lines event file.
   - Structured logs, still scraped.
   - Nothing new.
7. **Injection for triggers, clock and unlock.**
   - **Recommended:** a trigger-source trait (session lock, suspend, idle,
     clock) and an unlock-surface trait in djinn, with the OS
     implementations in production; fakes in-process; and a test-only
     build of the binary that takes injected events and scripted unlock
     answers from a harness pipe.
   - In-process only.
   - Real OS events only.
8. **How the walls are enforced.**
   - **Recommended:** a guard plus resident-side refusals.
     - The guard captures the installed resident by executable path, PID,
       start time and pipe names (enumerated, never connected), and
       refuses any standard endpoint or any missing redirect. On Linux it
       refuses a bus it did not start. It tracks the processes it spawns,
       in a job object on Windows, never acting by name, and re-checks at
       the end.
     - Resident side: a non-installed djinn refuses the default endpoints,
       and castellan's wall extends to Unix `SSH_AUTH_SOCK`.
   - The guard only.
   - Convention, as now.
9. **Stopping and restarting.**
   - **Recommended:** both a kill (the crash case) and a graceful stop
     through an owner-only stop intent on the door; the lock must persist
     across both.
   - Kill plus a console Ctrl-Break.
   - Kill only.
10. **First customers.**
    - **Recommended:** D1 and D1b move onto the harness as its acceptance
      test, with outcomes unchanged; the lock receipts come next; the
      re-exec and transport helpers follow when they are touched.
    - The lock receipts only.
    - Everything now.

### Rulings

**Ruling 1.** *Where does the harness live?* Options: a `djinn-testkit`
crate; a module in djinn's tests; a `test-support` feature in djinn; a
receipt binary. Mark: **"A djinn feature"**. *Reading, not ruled:* djinn
depends on castellan and personae, so those crates cannot use it from
their own tests without a dependency cycle; their resident-level receipts
run in djinn's tests (put back to Mark the same day).

**Ruling 2.** *Language?* Options: Rust, with shell only to launch; Rust
plus committed scripts; per-lane scripts. Mark: **"Rust, shell only to
launch (Recommended)"**.

**Ruling 3.** *Timing under load?* Options: relative to the run's own
events with load recorded, wide absolute bounds, an opt-in quiet mode and
a machine-wide lock file; refuse while loaded; bounded retries. Mark:
**"Relative timing, record load (Recommended)"**.

**Ruling 4.** *Real-machine receipts?* Options: a remote runner for the
ThinkPad and the Surface, macOS with D2, and the laptop's `Win+L` and
suspend as attended steps; remote Linux with checklists here; all manual.
Mark: **"Remote runner + attended steps (Recommended)"**.

**Ruling 5.** *Ruling 1's consequence: djinn depends on castellan and
personae (`ports/djinn/Cargo.toml:48, 68`), so a djinn feature cannot be
used from their own tests without a dependency cycle. Is that fine?*
Options: fine, with resident-level receipts in djinn's tests; switch to
`djinn-testkit`. Mark: **"Switch to djinn-testkit"**. Follows: amends
ruling 1; the harness is an unpublished `djinn-testkit` crate usable as a
dev-dependency anywhere, castellan and personae included.

**Ruling 6.** *The receipt record?* Options: a versioned JSON record per run
with a committed summary under `mere_docs/testing/receipts/` and raw
records outside the tree under `C:	
eceipts`, rechecked by a verifier;
a hand-written `RECEIPT.md`; stdout lines. Mark: **"Versioned JSON +
committed summary (Recommended)"**.

**Ruling 7.** *What djinn exposes for observation?* Options: an owner-only
`resident-status-v1` route and a `--log-filter` flag; those plus a
JSON-lines event file; structured logs, still scraped. Mark: **"Also an
event file"**. Follows: the status route, the log-filter flag, and a
lifecycle event file.

**Ruling 8.** *How are the walls enforced?* Options: a guard plus
resident-side refusals; the guard only; convention. Mark: **"Guard +
resident refusals (Recommended)"**. Follows: the guard captures the
installed resident by executable path, PID, start time and enumerated pipe
names, refuses standard endpoints, missing redirects and a bus it did not
start, tracks only its own processes, and re-checks at the end; a
non-installed djinn refuses the default endpoints, and the receipt
listener's wall extends to Unix `SSH_AUTH_SOCK`.

**Ruling 9.** *Stopping and restarting?* Options: a kill and a graceful
stop through an owner-only stop intent on the door; a kill plus a console
Ctrl-Break; a kill only. Mark: **"Kill and a graceful stop
(Recommended)"**. Follows: the vault lock's persisted lock (its ruling 5)
must survive both.

**Ruling 10.** *Who uses it first?* Options: D1 and D1b as its acceptance
test with outcomes unchanged, then the lock receipts, other helpers when
touched; the lock receipts only; everything now. Mark: **"D1 and D1b, then
the lock (Recommended)"**.

## 4. Phases

Drafted from the assessment; set once the forks are ruled.

- **H1 — isolated resident.** Done when:
  - [ ] a spawn redirects every root and endpoint, and an incomplete list
        is refused;
  - [ ] readiness needs no log scraping;
  - [ ] kill, graceful stop and restart work on the same roots;
  - [ ] no orphans remain if the test process dies;
  - [ ] D1 and D1b run on the harness with unchanged outcomes, the lines
        removed from them counted.
- **H2 — walls.** Done when:
  - [ ] no pinned PID and no kill by name remain;
  - [ ] the installed identity and its pipes are recorded identical before
        and after every run;
  - [ ] each wall has a refusal test.
- **H3 — records.** Done when:
  - [ ] every run writes the versioned record;
  - [ ] a verifier recomputes its evidence hashes.
- **H4 — the lock's seams.** Done when:
  - [ ] the status route reports lock state and startup mode;
  - [ ] fake triggers and a fake clock work in-process, and scripted
        unlock in a subprocess;
  - [ ] the lock receipts fail against today's `main`: a restarted
        resident comes back unlocked, and the agent lists identities after
        "lock".
- **H5 — remote.** Done when:
  - [ ] a ThinkPad run under `dbus-run-session` produces the same record;
  - [ ] the `authorized_keys` step is a library step with its byte check;
  - [ ] attended Windows steps are supported.
- **H6 — load.** Done when:
  - [ ] every timing receipt records load;
  - [ ] under generated load the relative assertions still pass and the
        absolute ones are flagged.

Controls, each of which must fail where it should:

- **C1:** a spawn on the standard agent pipe, or with a redirect missing, is
  refused before it starts, and a changed installed identity fails the
  receipt.
- **C2:** a deliberately broken resident (never ready, exits after ready,
  lists identities while locked, comes back unlocked) is caught by name.
- **C3:** killing the test process mid-run leaves no spawned process alive.
- **C4:** canary timestamps under the real data roots and vault are
  unchanged by a run, and a deliberately unredirected variable is caught.
- **C5:** a tampered evidence file fails verification.
- **C6:** a Secret Service receipt without a harness-owned bus is refused.

## 5. Findings

**2026-10-05.** Recorded in §2. Dated corrections in other documents:
- the vault lock plan's L4 condition names PID 53336 (annotated there);
- the pairing and chatelaine plans' receipts name PID 53336 as that day's
  identity, which was true then and needs no change.

## 6. Progress

**2026-10-05.** Assessed by a read-only lane (Opus); the load-bearing claims
re-checked in code. Nothing built. Next: Mark's rulings on §3.
