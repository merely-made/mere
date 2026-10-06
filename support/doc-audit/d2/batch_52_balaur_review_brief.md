# Batch 52: the balaur review brief (new document)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---|---:|---:|---:|---:|
| mere_docs/research/2026-10-06_balaur_review_brief.md | current | yes | 10 | 9 | 0 | 1 |
| **Totals** |  |  | **10** | **9** | **0** | **1** |

**Totals: 1 doc, 10 claims checked (9 holds, 0 stale, 1 unverifiable), 0 contradictions; 0 status lines wrong.**

Audit base: Mere `ed8b67e8` (2026-10-06), the commit that added the brief.
Balaur was read at `de0df794eee4eea223ed7efd31461044c71999f0`, its HEAD that
day, through its README, `ARCHITECTURE.md` and `LICENSE` on GitHub. No cargo
command was run. The coordinator who wrote the brief judged it on the same
day; an independent re-judgment can supersede this record under ruling S34.

## mere_docs/research/2026-10-06_balaur_review_brief.md

- disposition: current
- status line: "**Status:** open. The review lane writes its findings into §3." — accurate: yes
- claims checked: 10 — holds: 9, stale: 0, unverifiable: 1

### Stale claims

None.

### Contradictions

None.

### Recommended action

When the review lane writes §3, re-judge the brief and supersede this
record. Check §3's file and line citations against balaur
`de0df794eee4eea223ed7efd31461044c71999f0`, and check that §3 quotes no more
than short phrases from balaur's source.

### Notes

Each claim below was checked against the source in brackets, and holds:
- the description quote [README];
- MIT [LICENSE];
- the copyright line "Copyright (c) 2026 Sébastien Crozet, Dragos Daian" [LICENSE];
- the `.claude/` folder and `CLAUDE.md` [the repository's top-level listing];
- the nexus binning path and its ledger row [`LICENSES.md`, the
  `crates/conatus/conatus/src/resident/binning` row, Apache-2.0, "by
  Sébastien Crozet / Dimforge"];
- the composition of kiss3d 0.46, hecs 0.11, rapier 0.35, egui and taffy
  [`ARCHITECTURE.md`];
- the six slices' descriptions [`ARCHITECTURE.md`];
- Mark's two rulings [this session's questions and answers, 2026-10-06];
- the rapier `enhanced-determinism` timings at 500, 2,000 and 5,000 bodies
  [`Code/testing/mere/rapier-determinism/probe.log`, rapier2d 0.33].

Unverifiable: that Crozet is the author of rapier, parry and kiss3d as well as
nexus. The ledger names him for nexus only, and no source for the other three
was read for this record.
