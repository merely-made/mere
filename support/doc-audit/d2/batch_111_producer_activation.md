# Batch 111: Producer activation and child lifetimes

## mere_docs/implementation_strategy/2026-06-08_system_diagnostics_and_accessibility_plan.md

- disposition: current
- status line: "bounded checked activation implemented; automated receiving qualified within the boundaries below" - accurate: yes
- claims checked: 14 - holds: 14, stale: 0, unverifiable: 0

### Stale claims

Expired weak child tokens could compare equal to cached copies. Liveness now
refuses full publication and queued/direct admission. Raw requests freeze entry
context once; previously published/queued lives are preserved into activation.
Checked dispatch validates the admitted child token at the producer owner.
Rc adapters keep one mutable producer borrow; retained ContentProducer validates
and dispatches under its ContentSlot borrow. Stale/declined actions do not invoke
after-dispatch hooks or move focus as though dispatched.

### Contradictions

A trait default cannot make a deeper shared model atomic, and None child tokens
still require authored nonreuse. Action ID reassignment needs a new child life
or owner-checked revision. Dispatch is not terminal actor completion. Headless
host tests and compilation are not physical platform/AT acceptance.

### Recommended action

Receive a mutation-time member incarnation/removal witness in Canvas ingestion.
The separate red reproduction removes/recreates the same member in one closure
and demonstrates the old Pin reaching its replacement. Preserve ordinary live
keys rather than blanket invalidation. Then receive deeper custom owners and
action lifetimes, supplier intake, guest/platform and full-leaf paths in turn.

### Notes

The known Canvas failure is retained as an unapplied patch plus failed receipt,
separate from passing acceptance gates. Temporary test source was restored;
the public Pictograph key comment now states the actual reconciliation boundary.
Existing separate-call Canvas controls pass. Exact gates, source and boundaries
are in support/family-promotion/diagnostics-activation-qualification.json.
The lane caps retained evidence at 10 MB and reuses the stable Mere target.
Original staged work and concurrent receipt edits are preserved.
