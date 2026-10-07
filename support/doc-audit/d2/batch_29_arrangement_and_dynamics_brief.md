# Batch 29 — arrangement and dynamics brief (new research brief)

| doc | disposition | status accurate | claims | holds | stale | unverifiable |
|---|---|---:|---:|---:|---:|---:|
| mere_docs/research/2026-10-03_arrangement_and_dynamics_brief.md | current | yes | 181 | 181 | 0 | 0 |
| **Totals** |  |  | **181** | **181** | **0** | **0** |

**Totals: 1 doc, 181 claims checked (181 holds, 0 stale, 0 unverifiable), 0 contradictions inside the doc.**

Audit base: Mere `63c47a59` (2026-10-03), branch `arrangement-dynamics-brief`. The working tree held this pass's edits:
- the new brief;
- its `DOC_README.md` entry at the head of the `mere_docs/research/` list.

Read only: isometry `eaf155e`; mer3ly `abafbaf`, which moved from `4d20e05` during the pass with only its site-canvas plan changing; the turnstone `proof3_physics` captures. The probe log `Code/testing/mere/arr-dyn/probe.log` was also read. No web sources; `archive_docs/` is excluded.

This batch exists because the document is new. The record was taken after the document was written. Its citations were checked against the tree the same day, and the off-by-one line numbers found in that check were corrected before the record was taken.

## mere_docs/research/2026-10-03_arrangement_and_dynamics_brief.md

- disposition: current
- status line: "Status (2026-10-03): assessment brief, complete; the forks in §8 wait for Mark. Docs only, no code changed. One measurement was taken with a temporary probe that was run once and not committed (§5)." — accurate: yes
- claims checked: 181 — holds: 181, stale: 0, unverifiable: 0

### Stale claims

- none.

### Contradictions

- None inside the brief. The brief records disagreements elsewhere as findings rather than resolving them:
  - The projection grammar catalog's Anchored policy ("returns toward home", line 64) disagrees with `sceno::Hold::Anchored`'s doc ("moving an anchored item is correct behaviour", `score.rs:86-88`). Fork A4.
  - The canvas's drag writes the drop point into the slot, while the physics board's drag returns the item toward its slot (`input.rs:376-389` against `physics_board.rs:256-265`). Fork A4.
  - The canvas docs say the host gates its per-frame strategy projection on `needs_strategy_recompute` (`strategy.rs:62-64`), and no port in mere calls it. Noted in §10.
  - `SavedSceneV1.arrangement_pull` has no serde default, while the five physics-catalog fields do (`product.rs:221-238`). A consequence in §7.

### Recommended action

- None for this record. The disagreements above are forks A4 and A1/§7 consequences in the brief, and they are Mark's to rule. The catalog's side is carried by the Projection grammar session after the ruling.

### Notes

The claims checked, grouped by where each holds:

- **Code citations, 94.** These are file:line citations in pictograph, seiche, scenomise, sceno, scenotime, scenograph and graphshell. Every one was re-read at the base, and the off-by-one cases were corrected before this record: `anchor_force.rs:30/34`, `score.rs:86-88`, `canvas_physics.rs:101-111`, `return_motion.rs:56-65`.
- **Document-line citations, 81.** These cover:
  - the projection proofs plan (360–445), the physics catalog plan (50–51, 102, 109–117, 265–270, 280–300, 428–433, 658–665, 775–779, 1133–1135, 1213–1222);
  - the dynamics grammar plan (18, 33–36, 74–78, 148, 160, 170, 177–181, 209–224, 245) and brief (83, 163–164, 173, 178, 291, 304–306, 315, 327, 415–416);
  - the projection grammar catalog (61–65, 217, 258–260, 264–277, 434), the adoption plan (194–197, 264–290, 826–886) and the seam doc (41–49, 70–81);
  - the P2 receipt (17, 50–53) and three scenario files.
- **Sibling repositories, 6 files.** In isometry: `overmap/scene.rs`. In mer3ly: `repo-graph/src/lib.rs`, `assets/graph-sandbox.js` and `docs/2026-09-30_graphshell_site_canvas_plan.md` (Rulings 1, 11 and 20, S5, the consume table at 774). In mere: `fold_projection.rs`.
- **The provenance finding.** Every Mark quotation the brief uses was matched to its record line. The attractor's introduction (proofs plan 393–413) and commits `82ac70fe` and `12267085` were read for attribution and carry none.

The probe's figures (§5) come from one log and are counted as that receipt, not as tree claims. The brief's §10 names four things it could not verify, and none is counted above:
- which reading the 2026-07-22 played capture shows;
- the practice workspace's receipts;
- whether a ruling on mer3ly's Anchored/Free choice exists outside a keyword search;
- other repositories' callers of `needs_strategy_recompute`.

The judgment audit's coverage before this pass was 310 of 330 active documents; the 20 uncovered documents predate this batch and are not addressed here.
