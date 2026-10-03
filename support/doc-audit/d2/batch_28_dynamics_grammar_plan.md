# Batch 28 — dynamics grammar plan (new plan)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md | current | yes | 52 | 52 | 0 | 0 |
| **Totals** |  |  | **52** | **52** | **0** | **0** |

**Totals: 1 doc, 52 claims checked (52 holds, 0 stale, 0 unverifiable), 0 contradictions.**

Audit base: Mere `d3874ff3` (2026-10-02, main merged into
`dynamics-grammar-brief`), with this pass's edits in the working tree:
- the new plan;
- its `DOC_README.md` row, plus a sentence on the physics catalog plan's row
  and the dynamics grammar brief's row marking its forks ruled;
- in the physics catalog plan, a dated note closing P7 with a pointer here, a
  clause on its Status line, and a Progress entry;
- in the dynamics grammar brief, two dated annotations: the forks ruled, and
  the overwritten Density log.

Branches `density-cpu` (`c402d5e7`, a merge of main over `5f529c64`) and
`gpu-repulsion` (`60a990a5`) were read with `git log` and `git show`.
`archive_docs/` is excluded.

This batch exists because the plan is new. The rulings it carries were made
by Mark on 2026-10-01 and 2026-10-02 and are recorded first in the physics
catalog plan. The plan quotes them, so they are counted as quotation checks
against that record, not as claims about the tree.

## mere_docs/implementation_strategy/2026-10-02_dynamics_grammar_plan.md

- disposition: current
- status line: "Status (2026-10-02): planned. Written from the dynamics grammar brief and Mark's rulings of 2026-10-02; no track started. G1 comes first (ruled)." — accurate: yes
- claims checked: 52 — holds: 52, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- none. The plan corrects the brief's description of F9's rejected option
  (the full-force spread is a gradient in a group-size metric, class Em, and
  fails reciprocity rather than descent). It records the correction in its
  Findings, and the brief carries a dated pointer to it. The ruling is
  unaffected.

### Recommended action

- none for this record. The plan's §3 lists eight open questions, marked as
  readings, to return to Mark at the named tracks' checkpoints.

### Notes

The claims checked:

- **Rulings quoted**, 25:
  - the three grammar rulings (held, home, hypothesis);
  - F1 to F10;
  - the twelve input rulings in §1.2 (currencies, composition tier, meaning,
    embeddings, semantic field, Density with edges, overlays on Density,
    what earns a law id, home of GPU-tier laws, the picker, rapier's role,
    licensing).
  
  Each is checked against the physics catalog plan's §3 and §5 at `d3874ff3`.
- **Tree and branch facts**, 25 (the licence title and quote count as two):
  - `linlog.rs` lines 7–17;
  - `physics_catalog.rs` lines 62–65, 81–83, 160–165, 599 and 1087;
  - `strategy.rs` lines 520–616;
  - `index_burn.rs:144`;
  - `physics_board.rs:48`, with `PhysicsChoice` having no affinity field;
  - `product.rs` lines 216–243;
  - `anneal.rs` lines 66–91;
  - `sceno/score.rs` lines 107–120;
  - `seiche/lib.rs` lines 829–838;
  - the licence posture brief's title and "there are no exceptions";
  - `PhysicsLaw::overlay_refusal` on `density-cpu` (line 199);
  - the three branch heads;
  - `ports/graphshell/web/scenarios/physics_remote_board.scn`;
  - the P2 receipt's Anneal (energy ≤ 5) and Kinds (`energy >= 1` at 1 s)
    rows;
  - Density's ruled stop test;
  - `set_content_affinity` having no caller outside tests;
  - the density overlay probe's figures.
- **Physics catalog plan edits**, 2: the P7 closure's mapping of P7a–e to
  G1–G4, against P7's own text; the Status line.

Coverage before this pass was 309 of 329 active documents; the 20 uncovered
documents predate the dynamics grammar work and are not addressed here.
