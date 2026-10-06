# Batch 38 — dramatis repo plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| dramatis_docs/implementation_strategy/2026-10-06_dramatis_repo_plan.md | current | yes | 4 | 4 | 0 | 0 |
| **Totals** |  |  | **4** | **4** | **0** | **0** |

**Totals: 1 doc, 4 claims checked (4 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `ea169154` (2026-10-06). `archive_docs/` is excluded.

This batch exists because the plan is new.

## dramatis_docs/implementation_strategy/2026-10-06_dramatis_repo_plan.md

- disposition: current
- status line: "Status (2026-10-06): opened; rulings D1 to D5 (§3) set the shape. Nothing assessed in depth and nothing built. Next: a read-only assessment (§2's open list), then a round of forks for Mark. The build is sequenced against the vault lock plan, which is editing castellan and personae now." — accurate: yes
- claims checked: 4 — holds: 4, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none.

### Recommended action

- none for this record.

### Notes

The claims checked:
- castellan's size (3,626 lines over `ports/castellan/src/*.rs`) and the
  split of `view.rs` plus `projection.rs` (1,210 lines) and `authority.rs`
  (1,600 lines), by `wc -l`;
- the reference counts (62 code files, 48 doc files, 605 lines), by
  `git grep -i castellan`;
- crates.io ownership of castellan, chatelaine and dramatis (mark-ik), by
  the crates.io owners API on 2026-10-06;
- `DOC_POLICY.md`'s area-root invariant, in its local addendum.
