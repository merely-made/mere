# Batch 55: the postcard framing plan (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-06_postcard_framing_plan.md | current | yes | 6 | 6 | 0 | 0 |
| **Totals** |  |  | **6** | **6** | **0** | **0** |

**Totals: 1 doc, 6 claims checked (6 holds, 0 stale, 0 unverifiable), 0 contradictions; 0 status lines wrong.**

Audit base: Mere at the commit that adds the plan (2026-10-06). The
coordinator who wrote the plan judged it on the same day; an independent
re-judgment can supersede this record under ruling S34.

## mere_docs/implementation_strategy/2026-10-06_postcard_framing_plan.md

- disposition: current
- status line: "**Status:** plan, 2026-10-06. H1 waits on a crate name (checkpoint below)." — accurate: yes
- claims checked: 6 — holds: 6, stale: 0, unverifiable: 0

### Stale claims

None.

### Contradictions

None.

### Recommended action

Re-judge as each phase lands; H2 adds its inventory to Findings.

### Notes

Each claim below was checked against the source in brackets, and holds:
- wing-formats' `HEADER_LEN = 10`, 8-byte magics, `u16` versions and its
  `frame`/`unframe`/`peek` [`isometry/shared/wing-formats/src/lib.rs:16-62`];
- muniment's `REDO_MAGIC` [`crates/eidetic/muniment/src/directory_backend.rs:69`];
- the postcard call-site counts [`git grep` for `postcard::to_`/`from_`,
  tests excluded, in Mere, isometry and knot];
- the version-header name search and its result [`git grep` in each named
  crate's `src/`];
- F3, F4 and F6 [the data formats brief §5, and this session's questions and
  answers, 2026-10-06];
- ruling 621 of the wing design record (the repin onto stable)
  [`isometry/mesocosm/design_docs/2026-09-18_wing_design_plan.md`].
