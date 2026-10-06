# Batch 33 — vault lock plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md | current | yes | 7 | 7 | 0 | 0 |
| **Totals** |  |  | **7** | **7** | **0** | **0** |

**Totals: 1 doc, 7 claims checked (7 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `b52edea7` (2026-10-05). `archive_docs/` is excluded.

This batch exists because the plan is new. Its other claims are a read-only
lane's, each marked in the plan by whether it was re-checked.

## dramatis_docs/implementation_strategy/2026-10-05_vault_lock_plan.md

- disposition: current
- status line: "Status (2026-10-05): all 16 forks and the follow-ups ruled (rulings 1 to 24 in §3); the threat statement is still open. The djinn test harness it waited on (ruling 18) landed (`318b8f70`). L1 is under way on a lane branch: checkpoint A (residue fixes, the no-residue instrument, the caller map of ruling 23, a proposed lock API). Nothing merged. Chatelaine P4 (CXF import) waits on this plan (chatelaine rulings 64, 65)." — accurate: yes
- claims checked: 7 — holds: 7, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none recorded here; the plan itself notes that the protocol plan's §3.6,
  §3.7 and swap claim (line 349) do not hold today.

### Recommended action

- none for this record.

### Notes

The claims checked, each marked *checked* in the plan: `CryptUnprotectData`
in `startup_unlock.rs`; `lock: VaultLockView::Unlocked` at castellan
`authority.rs:201`; the resident agent built with `from_shared_vault` and an
`ApprovalBroker` (`authority.rs:191-194`); `CastellanResident::claim` holding
the record and freshness keys (`castellan/src/resident.rs:38-53`);
Distillery's `transport_identity` returning the profile's master
(`distillery/src/installed.rs:265-268`); and `PlaintextProfile` holding the
master seed with no zeroize (`profile_wire.rs:22-29`).
