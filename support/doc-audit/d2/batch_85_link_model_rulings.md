# Batch 85: Link model rulings L7–L13

Audit date: 2026-10-15. Re-judgment of the link model and its implementation plan after
Mark ruled O1, O4–O8 and the plan's pending same-asserter question, against Mere
`3968860b3`. No code claims were added; the new text records rulings and one phase (P7)
built on ruling 8's existing frozen-selection pattern.

## mere_docs/technical_architecture/2026-10-15_link_model.md

- disposition: current
- status line: "design with a bounded-tested P1 first slice and independent P5 CPU probe; phase acceptance remains open" - accurate: yes
- claims checked: 14 - holds: 14, stale: 0, unverifiable: 0

### Stale claims

None. §9 now lists no open rulings, and every former O-reference in §3, §6.1, §6.3, §7.2
and §8 points to its ruling.

### Contradictions

None. L13 extends ruling 1's dedup key with a reason key while keeping ruling 5's
version rule. L11 promotion reuses ruling 8 and mints no stored pair entity outside the
promoted Resource, so L1 stands.

### Recommended action

P2's owner specifies the reason key exactly, against L13, before changing dedup.

### Notes

Supersedes batch 84's record for this document.

## mere_docs/implementation_strategy/2026-10-15_link_model_plan.md

- disposition: current
- status line: "in progress. The bounded P1 reading and independent P5 CPU probe are implemented and bounded-tested; phase acceptance remains open." - accurate: yes
- claims checked: 4 - holds: 4, stale: 0, unverifiable: 0

### Stale claims

None. The pending same-asserter paragraph now records L13.

### Contradictions

None.

### Recommended action

Keep P7 out of the first delivery, as the plan states.

### Notes

Supersedes batch 84's record for this document.
