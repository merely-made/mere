# Batch 60 - graph semantics follow-ons and initial host plan

Audit base: Mere a8889f60e, 2026-10-09, with local documentation updates. Original aggregate is preserved. This batch judges the three owned new documents; unrelated inherited audit findings remain outside its scope.

## mere_docs/implementation_strategy/2026-10-09_graph_semantics_followons_plan.md

- disposition: current
- status line: "In progress; RDF qualified and published, providers qualified locally, Turnstone under qualification, host plan approved and entering Task 1" - accurate: yes
- claims checked: 5 - holds: 5, stale: 0, unverifiable: 0

### Stale claims

- None.

### Contradictions

- None.

### Recommended action

- Keep implementation and publication status current as gates land.

### Notes

Checked supplier source and six-gate repair receipt; verified published main/follow-on tips a8889f60e; counted 108 owned manifest rows in the repin report; provider receipt and local commits establish completed provider gates; active Turnstone runner establishes qualification in progress. Future A2/C phases are done conditions, not capability claims.

## mere_docs/technical_architecture/2026-10-09_resident_host_design.md

- disposition: current
- status line: "Written spec and implementation plan approved by Mark; entering implementation" - accurate: yes
- claims checked: 5 - holds: 5, stale: 0, unverifiable: 0

### Stale claims

- None.

### Contradictions

- None.

### Recommended action

- Keep implementation and publication status current as gates land.

### Notes

User approval settles the complete-writer milestone. Existing GraphSession owns complete graph/history; ResidentGraph is immutable; edit_now validates candidate/replay before installation; current resident publication checkpoints then writes separately; current browser staging releases its borrow for I/O. Remaining sections define approved requirements and deferred contracts, not implemented capabilities.

## mere_docs/implementation_strategy/2026-10-09_resident_host_implementation_plan.md

- disposition: current
- status line: "Written spec and concrete plan approved by Mark; team implementation starts with Task 1" - accurate: yes
- claims checked: 15 - holds: 15, stale: 0, unverifiable: 0

### Stale claims

- None.

### Contradictions

- None.

### Recommended action

- Keep implementation and publication status current as gates land.

### Notes

Checked all 15 existing-path responsibility rows against source at a8889f60e, including NodeFacetStore export, SessionId re-export, ViewKey::new, node_is_pinned, Applied fields and private pending_with(&self). New signatures and tests are expressly proposed; six new paths carry planned-path annotations. Eight tasks cover the approved spec and five review-focus failures. No new capability or successful host gate is claimed.
