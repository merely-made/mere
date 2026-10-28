# The djinn test harness: H4 remainder, H5, H6 and the audit's boundary repairs

**Status:** 2026-10-28: cut from `2026-10-05_djinn_test_harness_plan.md`, archived in `archive_docs/2026-10-28/`. The harness is the
unpublished `crates/system/djinn-testkit` crate plus djinn's observation surfaces (`resident-status-v1`,
`resident-control-v1`, `--events-file`, `--log-filter`, `--installed`). H1 (isolated resident), H2 (walls) and H3 (versioned
records with a verifier) landed, and the graceful stop with personal sync was fixed under ruling 11. H4 is partly met (the
status route reports lock state and startup mode). Open: H4's other conditions, H5 (remote runs), H6 (load), the controls
without recorded results, the audit's Q053 to Q055 and fork 7's missing ruling.

**For:** one shared, tested way to run djinn residents under test: isolated, observed without scraping logs, stopped and
restarted, held behind enforced walls around the installed resident, and recorded as receipts. It replaced the per-lane
two-resident harnesses rebuilt in the week of 2026-10-05 (Mark: "i have been wondering about our testing framework for djinn
too... and it's the daemon/cli thing for the stack, no? seems important") and serves djinn's own receipts first, then the
vault lock's and the crates that share djinn's patterns.
Not here:
- the full-source audit across the rest of Mere: `2026-10-28_source_audit_repairs_plan.md`;
- the pairing receipts (D1 and D1b) that run on this harness, and the legacy launcher's retirement at D2's switch (ruling
  12): `2026-10-28_device_pairing_plan.md`;
- the lock's receipts and the parked Secret Service design whose receipt needs a harness-owned bus (control C6):
  `2026-10-28_custody_plan.md`.

Receipts from this plan's runs go to `archive_docs/<date>/`, never into this
plan (DOC_POLICY §8). Evidence that matters is a test in the repository or a
line in Findings.

## Constraints

Numbers are the archived plan's rulings (`archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:197-281`); the others
are settled or landed. Each `Source:` is a line range in that file.
- **2, 5.** Rust for logic and records, with shell only to launch (for example `dbus-run-session -- cargo test`); the harness is
  an unpublished `djinn-testkit` crate usable as a dev-dependency anywhere, castellan and personae included (it depends on no
  djinn, castellan, personae or graphshell code, since those crates depend on each other). Source: `:199-226`.
- **3.** Assert relative to the run's own events and record load in every receipt; keep absolute times as wide recorded bounds;
  an opt-in quiet mode and a machine-wide lock file serialize live receipts. Source: `:210-213`.
- **4.** A remote runner for the ThinkPad and the Surface (bundle, separate worktree, run, fetch the record), macOS with D2,
  and the laptop's own `Win+L` and suspend as attended steps where the harness arms, Mark acts and the harness records.
  Source: `:215-218`.
- **6.** A versioned JSON record per run with a committed summary under `mere_docs/testing/receipts/`; raw records, which
  hold test vaults, stay outside the tree; a verifier recomputes the evidence hashes. Source: `:228-233`.
- **8.** A guard plus resident-side refusals: the guard captures the installed resident by executable path, PID, start time and
  enumerated (never connected) pipe names, refuses standard endpoints, missing redirects and a bus it did not start, tracks
  only its own processes (in a job object on Windows, never by name) and re-checks at the end; a non-installed djinn
  refuses the default endpoints and the receipt listener's wall covers Unix `SSH_AUTH_SOCK`. Source: `:241-248`.
- **9.** Both a kill (the crash case) and a graceful stop through an owner-only stop intent; the persisted lock survives both.
  Source: `:250-254`.
- **10.** D1 and D1b first, then the lock receipts; other re-exec and transport helpers move onto the harness when touched.
  Source: `:256-259`.
- **12.** The installed resident is told apart by the `--installed` flag the installer passes; manual runs need it or explicit
  endpoints. Source: `:267-272`.
- **13.** Stop is its own `resident-control-v1` route so granting status never grants stop; the guard's installed-locations list
  mirrors the installers' paths, is overridable and is empty on Unix; the crate sits under `crates/system`; status is read
  by running `djinn --resident-status`; receipts default to `%TEMP%\djinn-receipts`; the Linux test bus is left to H5. Source: `:274-281`.

## Phases

### H4. The lock's seams (remainder)
The status route reports lock state and startup mode. Open: the seam that lets lock receipts drive triggers, a clock and
unlock without real OS events, and receipts that fail against the old behaviour. A real `Win+L` or suspend on the
laptop stops everything running on it, the harness included, which is why the seam matters. Built with the vault lock (L1
to L4); see the open question on whether the lock's own receipts already meet these.
- Done when: fake triggers and a fake clock work in-process, and scripted unlock works in a subprocess; the lock receipts
  fail against the old behaviour: a restarted resident comes back unlocked, and the agent lists identities after "lock".
- Needs: fork 7's ruling
- Rulings: 9
- From: `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:164-171,305-315,417`

### H5. Remote
A ThinkPad run under `dbus-run-session` produces the same record (ruling 4 names the Surface as a second remote target). Djinn's own Linux cross-check cannot run on the
Windows laptop (ring's C build needs a cross compiler), so such checks go to the ThinkPad. Unix endpoint enumeration and
non-Linux Unix process identity are empty or stubbed today, so the closing assertions must not be reported as all-platform
protection; ruling 13's empty Unix installed-location list stays explicit until this phase.
- Done when: a ThinkPad run under `dbus-run-session` produces the same record; the `authorized_keys` step is a library step
  with its byte check; attended Windows steps are supported.
- Needs: the remote runner (ruling 4); macOS waits for pairing D2
- Rulings: 4, 8, 13
- From: `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:280,316-319,382-384,488-491`

### H6. Load
- Done when: every timing receipt records load; under generated load the relative assertions still pass and the absolute
  ones are flagged.
- Needs: -
- Rulings: 3
- From: `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:320-323`

### Controls C2 and C6
C2 (a deliberately broken resident, one that is never ready, exits after ready, lists identities while locked or comes back
unlocked, is caught by name) and C6 (a Secret Service receipt without a harness-owned bus is refused) have no recorded
result; the landing reports C1, C3, C4 and C5 only.
- Done when: each control fails where it should, with its result recorded.
- Needs: C6 needs a Linux test bus (H5)
- Rulings: -
- From: `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:325-336,376-377`

### Q053 to Q055. Boundary repairs (found by the 2026-10-16 audit, not repaired)
Q053: the guard reads case-sensitive environment keys, while Windows process environments merge names without regard to
case; lowercase aliases of `LOCALAPPDATA` and `SSH_AUTH_SOCK` pass the guard, and unknown variables are accepted despite the
module's stronger wording (role and askpass helpers add variables on purpose, so a blanket allowlist must account for
them). Q054: `Run::write` omits evidence whose hash read fails and ignores record and summary write errors, so a missing
requested file yields a passed record with zero evidence and a record that cannot be published still prints a passed
receipt and writes a passed summary; requested evidence, retained hashes, publication and outcome need one failure
contract. Q055: a resident name is joined directly to the run directory, so an absolute name selects an outside root and
`start` writes `lock.toml` there before the guard refuses. Further source candidates, not promoted to findings: assignment
to the process job happens after spawn; plan and evidence names enter paths without component admission; the binary hash
is taken at finalization, not launch.
- Done when: (from the findings' repair directions) the same effective environment that will reach Windows, case aliases
  included, is validated before spawning; a requested file that cannot be hashed or a record that cannot be written fails
  the run and prints no passed receipt; root and name ownership and the complete start are validated before any pre-spawn
  write; each repair has a regression.
- Needs: -
- Rulings: 8
- From: `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:432-438,440-469,471-481,493-496`

## Open questions

- Fork 7 has no ruling. The forks list a trigger-source trait (session lock, suspend, idle, clock), an unlock-surface trait
  in djinn with the OS implementations in production, fakes in-process, and a test-only build of the binary taking
  injected events and scripted unlock answers from a harness pipe (recommended), against in-process only or real OS events
  only. The rulings jump from fork 6 (ruling 7) to fork 8 (ruling 8), yet the status says all forks are ruled and H4 rests
  on this one. From: `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:4-7,164-171,305-315`
- Are H4's unticked boxes already met by the vault lock? Its L3 records each ruled trigger proven with an injected clock and
  "a locked resident restarted by the launcher comes back as ruled", and its gates ran the ignored `locked_restart`
  (whose control without a persisted lock fails) and `lock_agent` receipts on the harness. From:
  `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:305-315,417`; `archive_docs/2026-10-28/2026-10-05_vault_lock_plan.md:1176-1182,1937-1939`
- Is Q054 repaired? The ownership repair plan says it was already repaired on main and verified in the L6 worktree (the
  testkit tests passing), while this plan's 2026-10-16 audit entry lists it as not repaired. From: `2026-10-20_ownership_repair_plan.md:419,427-435`; `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:452-462`
- Where do the other re-exec and transport helpers (the `mere_two_process` and `reservoir_two_process` patterns, the
  transport's killed-peer test) move to? Ruling 10 says "when touched" and names no home. From:
  `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:192-193,486`

## Findings

- 2026-10-05: the installed resident's PID changes on reboot, so the wall is whatever holds the installed executable and its
  pipes, captured when a run starts. On Unix djinn's default agent endpoint is `$SSH_AUTH_SOCK` and it deletes that socket
  file when a connect is refused, so a test run without its own endpoint could take the user's socket. Source:
  `archive_docs/2026-10-28/2026-10-05_djinn_test_harness_plan.md:40-46,81-85`.
- 2026-10-06: D1b's second restart is the HyParView race (pairing ruling 72 waits for upstream): it opens only when a
  Neighbor message overtakes a Join, so one pass means that run did not hit it, not that it is gone. D1's stopped-peer time
  more than tripled under other sessions' builds while assertions relative to a run's own events held. Source: same file
  `:105-107,404-407`.
- 2026-10-16: the sharing claim is narrower than the harness's purpose. Resident and status wire readers avoid dependency
  cycles and are held to the producer by an integration test; D1, D1b and the lock receipts reuse the harness; other
  re-exec tests remain separate; `wait_ready` checks endpoint presence and reads status for the current pid rather than
  relying on a log. Source: same file `:483-491`.
- 2026-10-16: the Q053 to Q055 probes are public-API diagnostics against a freshly built testkit with empty installed-resident
  walls and generated decoys; they are not a repaired-harness gate or installed-resident acceptance. Source: same file `:471-481`.

## Progress

- 2026-10-28: cut from the djinn test harness plan; nothing new landed.
