# Batch 95: hest command architecture plan

Audit date: 2026-10-20. Scoped judgment of a new plan against `crates/cambium/command-menu`,
`crates/cambium/cambium`, `crates/cambium/cambium-rootstock`, `crates/graph/graph-kernel`,
`crates/system/registry`, `crates/system/ux-events`, `crates/chirograph`, `crates/shell/chrome`
at Mere `2fead1143`, plus Turnstone's `action.rs` read from a clone of its `main`. Nothing was
built beyond publishing the `hest` 0.0.1 reservation (`cargo publish --dry-run` and publish passed).

## cambium_docs/implementation_strategy/2026-10-20_hest_command_architecture_plan.md

- disposition: current
- status line: "plan; nothing moved" - accurate: yes (only the name reservation is published)
- claims checked: 30 - holds: 29, stale: 0, unverifiable: 1

### Stale claims

None. Every `file:line` citation in the census table was read in the source at the cited
commit.

### Contradictions

None with current rulings. It cites SE45 to SE50 (the Scenograph editor plan) and defers the
references there to H1 rather than amending them.

### Recommended action

Take rulings 1 to 3 to Mark before H1. Run H1 only as a pure move, with the existing
`command_menu_compact`, `graph-kernel` action and `registry` input tests as the receipts.

### Notes

Unverifiable: whether Turnstone's `Action` at `action.rs:116` is unchanged on its current
`main` (the clone was read, not a fresh fetch). The count of five dependent manifests of
`command-menu` excludes frozen receipt harnesses under `ports/graphshell/docs/receipts`.
