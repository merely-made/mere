# Batch 81: Resident host plan restored from the archive tag, and the design's status corrected

Audit date: 2026-10-15. Scoped judgment of the restored implementation plan and the corrected
design against Mere main `90ce8f12` and the tag `archive/resident-current-main-20261010`
(peeled `fd1640d0`). Source read: `pandect` (`graph_session/residency.rs`,
`graph_session_transaction.rs`, `addressable_graph/{residency,write}.rs`), `ports/graphshell`
(`mere_host.rs`, `mere_host_resident.rs`), `ports/djinn/src/resident_mere.rs`, `chirograph`,
`pictograph`'s canvas view and `chartulary`'s stemma snapshot.

## mere_docs/implementation_strategy/2026-10-09_resident_host_implementation_plan.md

- disposition: current
- status line: "Restored onto main for the port ... no code has been ported" - accurate: yes
- claims checked: 24 - holds: 24, stale: 0, unverifiable: 0

### Stale claims

None in the 2026-10-15 block. The restored lane text below it is the lane's record on its own
branch: its "Tasks 1–6 are complete" status and its checked boxes do not describe main, and the
block says so. None of the lane's core names exists on main (grep over `crates` and `ports`).

### Contradictions

None with current rulings. The lane's Task 3 contract (apply, then store, with a retained
batch) differs from C1's `transact_change` (store, then install); the plan records that as
fork F1 for Mark rather than choosing.

### Recommended action

Take forks F1 and F2 to Mark before Task 2 or Task 3 start.

### Notes

The lane's file references to `web_session.rs` and `web_product.rs` name modules C1 retired;
they carry historical-citation annotations. The paths the port will create carry
planned-target annotations. The pump for the browser route is deferred to the Task 7 map.

## mere_docs/technical_architecture/2026-10-09_resident_host_design.md

- disposition: current
- status line: "Proposed for user review" - accurate: no
- claims checked: 6 - holds: 5, stale: 1, unverifiable: 0

### Stale claims

The original status line, "Proposed for user review", was stale; it is kept and corrected by a
dated note quoting the plan's approval.

### Contradictions

None.

### Recommended action

None beyond the plan's forks. Supersedes any earlier record for this document.

### Notes

The approval is quoted from the plan on the tag; the tag's own copy of this design carries the
approved status, which main never received.
