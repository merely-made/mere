# Batch 122: Cambium composition rulings

## cambium_docs/implementation_strategy/2026-10-29_cambium_composition_plan.md

- disposition: current
- status line: "2026-10-29: P0 locally qualified; P1 in progress." - accurate: yes
- claims checked: 11 - holds: 11, stale: 0, unverifiable: 0

### Stale claims

None. The two findings added with the rulings were read on 2026-10-29:
upstream Xilem through the GitHub API (linebender/xilem main `66c8893f1`,
mark-ik/xilem branches), Murm's plan at origin/main `31281d62514`, Mere
`crates/system/surface-api/settings/revisioned.rs` at `01fce8a78`, and
Turnstone `src/launch.rs` at origin/main. The Masonry IME behavior is taken
from the text of linebender/xilem#1692, not from Masonry source.

### Contradictions

None. D6 (conflicts caught at Apply) and D7 (dirty rows cannot be rebound)
replace the live-conflict and three-way rebind text; the binding table,
rebind rules, first-consumer paragraph and reconciled-choices row now agree.
D14 cut P4–P6 and the selective-update and diagnostics design to outlines;
the producer table, Meristem section and change-ownership table stay because
P1's event vocabulary and the original request depend on them.

### Recommended action

Keep. D4 needs a check of whether Genet's host can force-commit an IME
composition before P2 implements moves; it is listed under Remaining
implementation questions.

### Notes

Mark answered fourteen question cards on 2026-10-29; D4 replaced the proposed
default and the other thirteen confirmed theirs. He also asked whether
upstream Xilem, Murm's secure sessions and live interaction plan or the Xilem
fork would undo assumptions here; the findings record that none does, with
two cues for a future remote writer (idempotent redelivery, redacted remote
causes).
