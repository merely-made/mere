# Batch 79: Graphshell stack lift plan

Audit date: 2026-10-14. Scoped judgment of a new plan against `ports/graphshell`,
`ports/djinn`, `crates/graphshell`, `crates/mere`, `crates/platen`, the scene crates and
`scripts/check_port_boundaries.py` at `0aade8668`, plus the Woodshed, Knot and Cleromancy
sources on this machine. Nothing was built; dependency cones were not measured.

## mere_docs/implementation_strategy/2026-10-14_graphshell_stack_lift_plan.md

- disposition: current
- status line: "plan; nothing moved" - accurate: yes
- claims checked: 49 - holds: 46, stale: 0, unverifiable: 3

### Stale claims

None. The document corrects a first-pass census and records each correction as a
dated finding; it amends no earlier document.

### Contradictions

None with current rulings. Finding 9 reports evidence beside the dramatis repo plan's
D15 and leaves that ruling standing.

### Recommended action

Take rulings R1 to R6 to Mark before the phases that name them; run P0 baselines.

### Notes

Unverifiable: dependency cones per consumer (git dependencies need network access),
Turnstone-side readiness (its source is not local), and whether a custody-scoping task
exists (none recorded in the doc tree). Line counts come from a script over the source
and include in-file tests.
