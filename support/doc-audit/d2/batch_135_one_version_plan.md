# Batch 135 — one-version plan

Written 2026-11-13 against Mere `origin/main` `198e1e967`, the day the document was added, at landing.

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
| --- | --- | --- | --- | --- | --- | --- |
| 2026-11-13_one_version_plan | current | yes | 6 | 4 | 1 | 1 |

## mere_docs/implementation_strategy/2026-11-13_one_version_plan.md

- disposition: current
- status line: "2026-11-13: plan. P1 has started outside Mere" — accurate: yes
- claims checked: 6 — holds: 4, stale: 1, unverifiable: 1

### Stale claims

- Corrected before landing: P5 said Mere carries patches under `support/patches` (rudof, burn, cubecl). `support/patches` is empty at `198e1e967`; corrected in the landing review commit.

### Contradictions

- none

### Recommended action

- none

### Notes

Checked by the first reader at landing. Holds: D144's text and commit (`6839c3138`); Mark's two card answers as quoted in D144; `mere-rdfc` staying in Mere (D144); Mere's root `[patch.crates-io]` rows (`Cargo.toml` from line 128) name only family crates. Unverifiable: the counts (263 skipped versions, 405 root holders and their classes), which come from `Code/testing/engine-split/skips_by_holder.md` and, as the plan says, from reason text and existing locks rather than a fresh resolve; P0's report is what verifies them.
